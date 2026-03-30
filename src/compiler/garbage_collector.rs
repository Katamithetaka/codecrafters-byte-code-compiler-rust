
use std::{cell::RefCell};

use crate::{prelude::Chunk, value::{GlobalFunction, Value}};

// A Gc handle is just an index - copying it is free
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gc(pub u32);

// Everything that can live on the heap
#[derive(Debug)]
pub enum HeapObject {
    Closure(GcClosure),
    Function(GcFunction),
    GlobalFunction(GlobalFunction),
    ValueVec(Vec<Value>),
    Class(GcClass),
    String(String),
    Instance(GcInstance),
    Cell(RefCell<Value>),
    Chunk(&'static Chunk)
}

#[derive(Debug)]
pub enum ResolvedObject<'a> {
    Closure(&'a GcClosure),
    Function(&'a GcFunction),
    GlobalFunction(&'a GlobalFunction),
    ValueVec(&'a [Value]),
    Class(&'a GcClass),
    String(&'a str),
    Number(f64),
    Null,
    Boolean(bool),
    Instance(&'a GcInstance),
    Cell(&'a RefCell<Value>)
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionKind {
    Function, Method {
        is_derived: bool
    }
}

// Stripped down versions of your existing types
// that use Gc handles instead of Rc
#[derive(Debug, Clone)]
pub struct GcFunction {
    pub name: Gc,
    pub arguments_count: u8,
    pub chunk: &'static Chunk,  // chunk can stay Rc for now
    pub function_kind: FunctionKind,
}

#[derive(Debug, Clone)]
pub struct GcClass {
    pub name: Gc,
    pub base_class: Gc,
    pub constructor: Gc,
    pub methods: Vec<(Gc, Gc)>,  // (name string Value, closure Value)
}

#[derive(Debug, Clone)]
pub struct GcInstance {
    pub class: Gc,
    // Contains both user-set fields AND pre-bound method closures.
    // Methods are inserted at creation time so every field lookup is O(1)
    // with no class traversal needed at runtime.
    pub fields: hashbrown::HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy)]
pub struct GcClosure {
    pub class: Gc,
    pub instance: Gc,
    pub function: Gc,       // index to a GcFunction
    pub upvalues: Gc,       // index to an Upvalues
    pub function_kind: FunctionKind,
}

pub struct Heap {
    pub objects: Vec<Option<HeapObject>>,  // None = freed slot
    pub marked: Vec<bool>,
    pub threshold: usize,
    pub free_slots: Vec<u32>,  // recycled indices from freed objects
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: Vec::with_capacity(4096),
            marked: Vec::with_capacity(4096),
            threshold: 4096,
            free_slots: Vec::new(),
        }
    }

    // -----------------------------------------------------------------
    // Allocation
    // -----------------------------------------------------------------

    /// Allocate an object on the heap, reusing a freed slot when available.
    /// If the live object count has reached `threshold`, a collection is
    /// triggered first using the provided set of GC roots.
    pub fn alloc_with_roots(&mut self, obj: HeapObject, roots: &[Value]) -> Gc {
        let live = self.objects.len() - self.free_slots.len();
        if live >= self.threshold {
            self.collect(roots);
            // After collection grow the threshold so we don't collect every
            // single allocation when the heap is legitimately large.
            self.threshold = (self.live_count() * 2).max(1024);
        }
        self.alloc(obj)
    }

    /// Low-level alloc — no GC triggered. Prefer `alloc_with_roots`.
    pub fn alloc(&mut self, obj: HeapObject) -> Gc {
        if let Some(slot) = self.free_slots.pop() {
            self.objects[slot as usize] = Some(obj);
            self.marked[slot as usize] = false;
            return Gc(slot);
        }

        let index = self.objects.len() as u32;
        self.objects.push(Some(obj));
        self.marked.push(false);
        Gc(index)
    }

    pub fn live_count(&self) -> usize {
        self.objects.len() - self.free_slots.len()
    }

    // -----------------------------------------------------------------
    // Object access
    // -----------------------------------------------------------------

    pub fn get(&self, handle: Gc) -> &HeapObject {
        self.objects[handle.0 as usize].as_ref().unwrap()
    }

    pub fn get_mut(&mut self, handle: Gc) -> &mut HeapObject {
        self.objects[handle.0 as usize].as_mut().unwrap()
    }

    // -----------------------------------------------------------------
    // Mark-and-sweep garbage collection
    // -----------------------------------------------------------------

    /// Run a mark-and-sweep collection starting from the provided roots.
    pub fn collect(&mut self, roots: &[Value]) {
        // --- Mark phase ---
        for marked in self.marked.iter_mut() {
            *marked = false;
        }
        for &root in roots {
            self.mark(root);
        }

        // --- Sweep phase ---
        for i in 0..self.objects.len() {
            if self.objects[i].is_some() && !self.marked[i] {
                self.objects[i] = None;
                self.free_slots.push(i as u32);
            }
        }
    }

    fn mark(&mut self, value: Value) {
        let gc = match value.is_number() || value.is_null() || value.is_bool() {
            true => return,
            false => value.unwrap_gc()
        };

        let idx = gc.0 as usize;
        if idx >= self.marked.len() || self.marked[idx] {
            return;
        }
        self.marked[idx] = true;

        // Trace children — we need to collect them first to avoid borrow issues
        let children = self.children_of(gc);
        for child in children {
            self.mark(child);
        }
    }

    /// Collect all Values directly reachable from a heap object.
    fn children_of(&self, gc: Gc) -> Vec<Value> {
        match self.objects[gc.0 as usize].as_ref() {
            None => vec![],
            Some(obj) => match obj {
                HeapObject::Closure(c) => {
                    let mut v = vec![Value::function(c.function)];
                    if let Some(cls) = c.class.as_option() { v.push(Value::class(cls)); }
                    if let Some(inst) = c.instance.as_option() { v.push(Value::instance(inst)); }
                    v.push(Value::function(c.upvalues)); // upvalues vec
                    v
                }
                HeapObject::Function(f) => vec![Value::string(f.name)],
                HeapObject::GlobalFunction(_) => vec![],
                HeapObject::ValueVec(vals) => vals.clone(),
                HeapObject::Class(cls) => {
                    let mut v = vec![Value::string(cls.name)];
                    if let Some(b) = cls.base_class.as_option() { v.push(Value::class(b)); }
                    if let Some(c) = cls.constructor.as_option() { v.push(Value::closure(c)); }
                    for (name, method) in &cls.methods {
                        v.push(Value::string(*name));
                        v.push(Value::closure(*method));
                    }
                    v
                }
                HeapObject::Instance(inst) => {
                    let mut v = vec![Value::class(inst.class)];
                    for val in inst.fields.values() {
                        v.push(*val);
                    }
                    v
                }
                HeapObject::String(_) => vec![],
                HeapObject::Cell(cell) => vec![*cell.borrow()],
                HeapObject::Chunk(_) => vec![]
            },
        }
    }

    pub fn resolve_cell(&self, gc: Gc) -> Option<&RefCell<Value>> {
        if let HeapObject::Cell(ref_cell) = self.get(gc) {
            Some(&ref_cell)
        } else {
            None
        }
    }

    pub fn resolve_class(&self, gc: Gc) -> Option<&GcClass> {
        if let HeapObject::Class(ref_cell) = self.get(gc) {
            Some(&ref_cell)
        } else {
            None
        }
    }


    pub fn resolve_instance(&self, gc: Gc) -> Option<&GcInstance> {
        if let HeapObject::Instance(ref_cell) = self.get(gc) {
            Some(&ref_cell)
        } else {
            None
        }
    }

    pub fn resolve_function(&self, gc: Gc) -> Option<&GcFunction> {
        if let HeapObject::Function(ref_cell) = self.get(gc) {
            Some(ref_cell)
        } else {
            None
        }
    }

    pub fn resolve_global_function(&self, gc: Gc) -> Option<&GlobalFunction> {
        if let HeapObject::GlobalFunction(ref_cell) = self.get(gc) {
            Some(ref_cell)
        } else {
            None
        }
    }

    pub fn resolve_closure(&self, gc: Gc) -> Option<&GcClosure> {
        if let HeapObject::Closure(ref_cell) = self.get(gc) {
            Some(ref_cell)
        } else {
            None
        }
    }

    pub fn resolve_string(&self, gc: Gc) -> Option<&str> {
        if let HeapObject::String(ref_cell) = self.get(gc) {
            Some(ref_cell)
        } else {
            None
        }
    }

    pub fn resolve_value_vec(&self, gc: Gc) -> Option<&[Value]> {
        if let HeapObject::ValueVec(ref_cell) = self.get(gc) {
            Some(ref_cell)
        } else {
            None
        }
    }


    // -----------------------------------------------------------------
    // Value resolution helpers
    // -----------------------------------------------------------------

    pub fn resolve_inner(&self, value: Value) -> Option<ResolvedObject<'_>> {
        match self.resolve(value) {
            Some(ResolvedObject::Cell(ref_cell)) => self.resolve_inner(*ref_cell.borrow()),
            value => value,
        }
    }

    pub fn resolve(&self, value: Value) -> Option<ResolvedObject<'_>> {

        match Some(()) {
            _ if value.is_number() => Some(ResolvedObject::Number(value.as_number())),
            _ if value.is_null() => Some(ResolvedObject::Null),
            _ if value.is_bool() => Some(ResolvedObject::Boolean(value.as_bool())),
            _ if value.is_cell() => self.resolve_cell(value.unwrap_gc()).map(ResolvedObject::Cell),
            _ if value.is_class() => self.resolve_class(value.unwrap_gc()).map(ResolvedObject::Class),
            _ if value.is_instance() => self.resolve_instance(value.unwrap_gc()).map(ResolvedObject::Instance),
            _ if value.is_function() => self.resolve_function(value.unwrap_gc()).map(ResolvedObject::Function),
            _ if value.is_string() => self.resolve_string(value.unwrap_gc()).map(ResolvedObject::String),
            _ if value.is_global_function() => self.resolve_global_function(value.unwrap_gc()).map(ResolvedObject::GlobalFunction),
            _ if value.is_closure() => self.resolve_closure(value.unwrap_gc()).map(ResolvedObject::Closure),
            _ => None
        }
    }

    pub fn to_string(&self, value: Value) -> Option<String> {
         self.resolve_inner(value).and_then(|c| match c {
            ResolvedObject::Closure(gc_closure) => self.to_string(Value::function(gc_closure.function)),
            ResolvedObject::Function(gc_function) => {
                self.to_string(Value::string(gc_function.name)).map(|name| format!("<fn {name}>"))
            }
            ResolvedObject::GlobalFunction(global_function) => {
                Some(format!("<fn {}>", global_function.name))
            }
            ResolvedObject::ValueVec(_) => Some(String::new()),
            ResolvedObject::Class(gc_class) => self.to_string(Value::string(gc_class.name)),
            ResolvedObject::String(c) => Some(c.to_string()),
            ResolvedObject::Number(c) => Some(format!("{c}")),
            ResolvedObject::Null => Some("nil".to_string()),
            ResolvedObject::Boolean(f) => Some(format!("{f}")),
            ResolvedObject::Instance(i) => {
                self.to_string(Value::class(i.class)).map(|c| format!("{} instance", c))
            }
            ResolvedObject::Cell(_) => unreachable!(),
        })
    }

    pub fn is_truthy(&self, value: Value) -> Option<bool> {
        self.resolve_inner(value).and_then(|v| match v {
            ResolvedObject::Null => Some(false),
            ResolvedObject::Boolean(v) => Some(v),
            _ => Some(true),
        })
    }

    pub fn set(&mut self, lhs: &mut Value, rhs: Value) {
        match self.resolve(*lhs).unwrap() {
            ResolvedObject::Cell(ref_cell) => *ref_cell.borrow_mut() = rhs,
            _ => *lhs = rhs,
        }
    }

    pub fn copy_value(&self, value: Value) -> Option<Value> {
        match value {
            _ if value.is_cell() => { self.resolve_cell(value.unwrap_gc()).map(|c| *c.borrow())}
            _ => Some(value),
        }
    }

    pub fn alloc_string(&mut self, s: String) -> Value {
        Value::string(self.alloc(HeapObject::String(s)))
    }

    // -----------------------------------------------------------------
    // Class helpers
    // -----------------------------------------------------------------

    /// Return the string name of a class/closure/function value.
    pub fn name_of(&self, value: Value) -> Option<String> {
        self.resolve_inner(value).and_then(|c|  match c {
            ResolvedObject::Function(f) => self.to_string(Value::string(f.name)),
            ResolvedObject::Closure(c) => self.name_of(Value::function(c.function)),
            ResolvedObject::Class(c) => self.to_string(Value::string(c.name)),
            _ => None,
        })
    }

    /// Look up a method by name string on a class Gc handle.
    /// Searches the class's own methods vec; does NOT walk base classes.
    pub fn class_get_method(&self, class: Gc, name: &str) -> Gc {
        self.resolve_class(class).and_then(|c|  {
            for (n, v) in &c.methods {
                if self.to_string(Value::string(*n)).unwrap() == name {
                    return Some(*v);
                }
            }

            None
        }).unwrap_or_default()
    }

    /// Returns true if a method with the given name exists on the class.
    pub fn class_has_method(&self, class: Gc, name: &str) -> bool {
        self.class_get_method(class, name).is_some()
    }

    pub fn equals(&mut self, lhs: Value, rhs: Value) -> bool {
        match (self.resolve_inner(lhs), self.resolve_inner(rhs)) {
            (Some(ResolvedObject::Closure(lhs)), Some(ResolvedObject::Closure(rhs))) => self.equals(Value::function(lhs.function), Value::function(rhs.function)),
            (Some(ResolvedObject::Function(gc_function)), Some(ResolvedObject::Function(gc_closure))) => self.equals(Value::string(gc_function.name), Value::string(gc_closure.name)),
            (Some(ResolvedObject::GlobalFunction(global_function)), Some(ResolvedObject::GlobalFunction(gf))) => global_function.name == gf.name,
            (Some(ResolvedObject::Class(gc_class)), Some(ResolvedObject::Class(gc_closure))) => self.equals(Value::string(gc_class.name), Value::string(gc_closure.name)),
            (Some(ResolvedObject::String(s)), Some(ResolvedObject::String(gc_closure))) => s == gc_closure,
            (Some(ResolvedObject::Number(n)), Some(ResolvedObject::Number(rhs))) => n == rhs,
            (Some(ResolvedObject::Null), Some(ResolvedObject::Null)) => true,
            (Some(ResolvedObject::Boolean(lhs)), Some(ResolvedObject::Boolean(rhs))) => lhs == rhs,
            (Some(ResolvedObject::Instance(_)), Some(ResolvedObject::Instance(_))) => false,
            _ => false
        }
    }

    /// Add (or overwrite) a named method on a class.
    pub fn class_add_method(&mut self, class: Gc, name: Value, closure: Value) {
        let name_str = self.to_string(name).unwrap();
        match self.get_mut(class) {
            HeapObject::Class(c) => {

                c.methods.push((name.unwrap_gc(), closure.unwrap_gc()));
                // update constructor slot if this is "init"
                if name_str == "init" {
                    c.constructor = closure.unwrap_gc();
                }
            }
            _ => panic!("Expected a class"),
        }
    }

    /// Copy all methods from `src` class onto `dst` class, skipping ones
    /// already defined on `dst`.  Also copies constructor if dst has none.
    pub fn class_inherit(&mut self, dst: Gc, src: Gc) {
        // Collect methods from src first to avoid borrow conflict
        let src_methods: Vec<(Gc, Gc)> = match self.get(src) {
            HeapObject::Class(c) => c.methods.clone(),
            _ => panic!("Expected a class"),
        };
        let src_constructor: Gc = match self.get(src) {
            HeapObject::Class(c) => c.constructor,
            _ => Gc::NONE,
        };

        for (name, method) in src_methods {
            let name_str = self.to_string(Value::string(name)).unwrap();
            if !self.class_has_method(dst, &name_str) {
                match self.get_mut(dst) {
                    HeapObject::Class(c) => {
                        c.methods.push((name, method));
                        if name_str == "init" && c.constructor.is_none() {
                            c.constructor = method;
                        }
                    }
                    _ => panic!("Expected a class"),
                }
            }
        }

        // copy base_class pointer
        {
            match self.get_mut(dst) {
                HeapObject::Class(c) => c.base_class = src,
                _ => {}
            }
        }

        // copy constructor if dst doesn't have one
        if !src_constructor.is_none() {
            if {
                match self.get(dst) {
                    HeapObject::Class(c) => c.constructor.is_none(),
                    _ => false,
                }
            } {
                match self.get_mut(dst) {
                    HeapObject::Class(c) => c.constructor = src_constructor,
                    _ => {}
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // Instance helpers
    // -----------------------------------------------------------------

    /// Create a new instance of `class_val`, pre-populating `fields` with
    /// every method from the class (and its ancestors) already bound to the
    /// fresh instance.  User-written fields added later simply overwrite the
    /// method entries with the same name.
    pub fn instance_create(&mut self, class_val: Gc) -> Option<Value> {
        // Gather all (name_str, closure_val) pairs from the class before
        // touching the heap mutably, to avoid borrow conflicts.

        let methods: Vec<(String, Gc)> =
            self.resolve_class(class_val)?
                .methods
                .iter()
                .map(|(n, v)| (self.to_string(Value::string(*n)).unwrap(), *v))
                .collect();


        // Allocate the instance with an empty field map first so we have a
        // stable Gc handle to pass to bind_method.
        let instance_gc = self.alloc(HeapObject::Instance(GcInstance {
            class: class_val,
            fields: hashbrown::HashMap::with_capacity(methods.len()),
        }));


        // Bind every method and store it as a field.
        for (name, closure_val) in methods {
            let bound = self.bind_method(closure_val, instance_gc)?;
            match self.get_mut(instance_gc) {
                HeapObject::Instance(i) => {
                    i.fields.insert(name, bound);
                }
                _ => return None,
            }
        }

        Some(Value::instance(instance_gc))
    }

    /// Simple field read — O(1), no class traversal.
    /// Methods were pre-bound into fields at construction time.
    pub fn instance_get(&self, instance: Gc, name: &str) -> Option<Value> {
        match self.get(instance) {
            HeapObject::Instance(i) => i.fields.get(name).copied(),
            _ => panic!("Expected an instance"),
        }
    }

    pub fn instance_set_field(&mut self, instance: Gc, name: String, value: Value) {
        match self.get_mut(instance) {
            HeapObject::Instance(i) => {
                i.fields.insert(name, value);
            }
            _ => panic!("Expected an instance"),
        }
    }

    /// Wrap a closure value with a bound instance, producing a new Closure
    /// heap object whose `instance` field is set.
    pub fn bind_method(&mut self, closure_val: Gc, instance: Gc) -> Option<Value> {
        let gc = closure_val;
        let closure = self.resolve_closure(closure_val)?;
        let bound = GcClosure { instance: instance, ..*closure };
        Some(Value::closure(self.alloc(HeapObject::Closure(bound))))
    }
}




impl Gc {
    pub const NONE: Gc = Gc(u32::MAX);

    pub fn is_none(self) -> bool { self == Self::NONE }
    pub fn is_some(self) -> bool { !self.is_none() }
    pub fn as_option(self) -> Option<Gc> {
        if self.is_none() { None } else { Some(self) }
    }
}

impl Default for Gc {
    fn default() -> Self {
        Gc::NONE
    }
}
