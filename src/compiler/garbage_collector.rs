
use std::{cell::RefCell};

use crate::{prelude::Chunk, value::{GlobalFunction, Value}};

// A Gc handle is just an index - copying it is free
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gc(u32);

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
    pub name: Value,
    pub arguments_count: u8,
    pub chunk: &'static Chunk,  // chunk can stay Rc for now
    pub function_kind: FunctionKind,
}

#[derive(Debug, Clone)]
pub struct GcClass {
    pub name: Value,
    pub base_class: Option<Value>,
    pub constructor: Option<Value>,
    pub methods: Vec<(Value, Value)>,  // (name string Value, closure Value)
}

#[derive(Debug, Clone)]
pub struct GcInstance {
    pub class: Value,
    // Contains both user-set fields AND pre-bound method closures.
    // Methods are inserted at creation time so every field lookup is O(1)
    // with no class traversal needed at runtime.
    pub fields: hashbrown::HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy)]
pub struct GcClosure {
    pub class: Option<Value>,
    pub instance: Option<Value>,
    pub function: Value,       // index to a GcFunction
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
            objects: Vec::with_capacity(1024),
            marked: Vec::with_capacity(1024),
            threshold: 1024,
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
        let gc = match value {
            Value::String(gc)
            | Value::Function(gc)
            | Value::GlobalFunction(gc)
            | Value::Closure(gc)
            | Value::Class(gc)
            | Value::Instance(gc)
            | Value::Cell(gc) => gc,
            Value::Number(_) | Value::Null | Value::Boolean(_) => return,
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
                    let mut v = vec![c.function];
                    if let Some(cls) = c.class { v.push(cls); }
                    if let Some(inst) = c.instance { v.push(inst); }
                    v.push(Value::Function(c.upvalues)); // upvalues vec
                    v
                }
                HeapObject::Function(f) => vec![f.name],
                HeapObject::GlobalFunction(_) => vec![],
                HeapObject::ValueVec(vals) => vals.clone(),
                HeapObject::Class(cls) => {
                    let mut v = vec![cls.name];
                    if let Some(b) = cls.base_class { v.push(b); }
                    if let Some(c) = cls.constructor { v.push(c); }
                    for (name, method) in &cls.methods {
                        v.push(*name);
                        v.push(*method);
                    }
                    v
                }
                HeapObject::Instance(inst) => {
                    let mut v = vec![inst.class];
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

    // -----------------------------------------------------------------
    // Value resolution helpers
    // -----------------------------------------------------------------

    pub fn resolve_inner(&self, value: Value) -> ResolvedObject<'_> {
        match self.resolve(value) {
            ResolvedObject::Cell(ref_cell) => self.resolve_inner(*ref_cell.borrow()),
            value => value,
        }
    }

    pub fn resolve(&self, value: Value) -> ResolvedObject<'_> {
        match value {
            Value::Number(c) => ResolvedObject::Number(c),
            Value::String(gc) => match self.get(gc) {
                HeapObject::String(s) => ResolvedObject::String(s),
                _ => panic!("Expected a string!"),
            },
            Value::Null => ResolvedObject::Null,
            Value::Boolean(b) => ResolvedObject::Boolean(b),
            Value::Function(gc) => match self.get(gc) {
                HeapObject::Function(s) => ResolvedObject::Function(s),
                _ => panic!("Expected a function!"),
            },
            Value::GlobalFunction(global_function) => match self.get(global_function) {
                HeapObject::GlobalFunction(s) => ResolvedObject::GlobalFunction(s),
                _ => panic!("Expected a global function!"),
            },
            Value::Closure(gc) => match self.get(gc) {
                HeapObject::Closure(s) => ResolvedObject::Closure(s),
                _ => panic!("Expected a closure!"),
            },
            Value::Class(gc) => match self.get(gc) {
                HeapObject::Class(s) => ResolvedObject::Class(s),
                _ => panic!("Expected a class!"),
            },
            Value::Instance(gc) => match self.get(gc) {
                HeapObject::Instance(i) => ResolvedObject::Instance(i),
                _ => panic!("Expected a class instance!"),
            },
            Value::Cell(gc) => match self.get(gc) {
                HeapObject::Cell(c) => ResolvedObject::Cell(c),
                _ => panic!("Expected a value pointer!"),
            },
        }
    }

    pub fn to_string(&self, value: Value) -> String {
        match self.resolve_inner(value) {
            ResolvedObject::Closure(gc_closure) => self.to_string(gc_closure.function),
            ResolvedObject::Function(gc_function) => {
                let name = self.to_string(gc_function.name);
                format!("<fn {name}>")
            }
            ResolvedObject::GlobalFunction(global_function) => {
                format!("<fn {}>", global_function.name)
            }
            ResolvedObject::ValueVec(_) => String::new(),
            ResolvedObject::Class(gc_class) => self.to_string(gc_class.name),
            ResolvedObject::String(c) => c.to_string(),
            ResolvedObject::Number(c) => format!("{c}"),
            ResolvedObject::Null => "nil".to_string(),
            ResolvedObject::Boolean(f) => format!("{f}"),
            ResolvedObject::Instance(i) => {
                format!("{} instance", self.to_string(i.class))
            }
            ResolvedObject::Cell(_) => unreachable!(),
        }
    }

    pub fn is_truthy(&self, value: Value) -> bool {
        match self.resolve_inner(value) {
            ResolvedObject::Null => false,
            ResolvedObject::Boolean(v) => v,
            _ => true,
        }
    }

    pub fn set(&mut self, lhs: &mut Value, rhs: Value) {
        match self.resolve(*lhs) {
            ResolvedObject::Cell(ref_cell) => *ref_cell.borrow_mut() = rhs,
            _ => *lhs = rhs,
        }
    }

    pub fn copy_value(&self, value: Value) -> Value {
        match value {
            Value::Cell(c) => match self.get(c) {
                HeapObject::Cell(ref_cell) => *ref_cell.borrow(),
                _ => unreachable!(),
            },
            _ => value,
        }
    }

    pub fn alloc_string(&mut self, s: String) -> Value {
        Value::String(self.alloc(HeapObject::String(s)))
    }

    // -----------------------------------------------------------------
    // Class helpers
    // -----------------------------------------------------------------

    /// Return the string name of a class/closure/function value.
    pub fn name_of(&self, value: Value) -> String {
        match self.resolve_inner(value) {
            ResolvedObject::Function(f) => self.to_string(f.name),
            ResolvedObject::Closure(c) => self.name_of(c.function),
            ResolvedObject::Class(c) => self.to_string(c.name),
            _ => String::new(),
        }
    }

    /// Look up a method by name string on a class Gc handle.
    /// Searches the class's own methods vec; does NOT walk base classes.
    pub fn class_get_method(&self, class: Gc, name: &str) -> Option<Value> {
        match self.get(class) {
            HeapObject::Class(c) => {
                for (n, v) in &c.methods {
                    if self.to_string(*n) == name {
                        return Some(*v);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Returns true if a method with the given name exists on the class.
    pub fn class_has_method(&self, class: Gc, name: &str) -> bool {
        self.class_get_method(class, name).is_some()
    }

    pub fn equals(&mut self, lhs: Value, rhs: Value) -> bool {
        match (self.resolve_inner(lhs), self.resolve_inner(rhs)) {
            (ResolvedObject::Closure(lhs), ResolvedObject::Closure(rhs)) => self.equals(lhs.function, rhs.function),
            (ResolvedObject::Function(gc_function), ResolvedObject::Function(gc_closure)) => self.equals(gc_function.name, gc_closure.name),
            (ResolvedObject::GlobalFunction(global_function), ResolvedObject::GlobalFunction(gf)) => global_function.name == gf.name,
            (ResolvedObject::Class(gc_class), ResolvedObject::Class(gc_closure)) => self.equals(gc_class.name, gc_closure.name),
            (ResolvedObject::String(s), ResolvedObject::String(gc_closure)) => s == gc_closure,
            (ResolvedObject::Number(n), ResolvedObject::Number(rhs)) => n == rhs,
            (ResolvedObject::Null, ResolvedObject::Null) => true,
            (ResolvedObject::Boolean(lhs), ResolvedObject::Boolean(rhs)) => lhs == rhs,
            (ResolvedObject::Instance(_), ResolvedObject::Instance(_)) => false,
            _ => false
        }
    }

    /// Add (or overwrite) a named method on a class.
    pub fn class_add_method(&mut self, class: Gc, name: Value, closure: Value) {
        let name_str = self.to_string(name);
        match self.get_mut(class) {
            HeapObject::Class(c) => {

                c.methods.push((name, closure));
                // update constructor slot if this is "init"
                if name_str == "init" {
                    c.constructor = Some(closure);
                }
            }
            _ => panic!("Expected a class"),
        }
    }

    /// Copy all methods from `src` class onto `dst` class, skipping ones
    /// already defined on `dst`.  Also copies constructor if dst has none.
    pub fn class_inherit(&mut self, dst: Gc, src: Gc) {
        // Collect methods from src first to avoid borrow conflict
        let src_methods: Vec<(Value, Value)> = match self.get(src) {
            HeapObject::Class(c) => c.methods.clone(),
            _ => panic!("Expected a class"),
        };
        let src_constructor: Option<Value> = match self.get(src) {
            HeapObject::Class(c) => c.constructor,
            _ => None,
        };

        for (name, method) in src_methods {
            let name_str = self.to_string(name);
            if !self.class_has_method(dst, &name_str) {
                match self.get_mut(dst) {
                    HeapObject::Class(c) => {
                        c.methods.push((name, method));
                        if name_str == "init" && c.constructor.is_none() {
                            c.constructor = Some(method);
                        }
                    }
                    _ => panic!("Expected a class"),
                }
            }
        }

        // copy base_class pointer
        {
            match self.get_mut(dst) {
                HeapObject::Class(c) => c.base_class = Some(Value::Class(src)),
                _ => {}
            }
        }

        // copy constructor if dst doesn't have one
        if let (Some(ctor), true) = (src_constructor, {
            match self.get(dst) {
                HeapObject::Class(c) => c.constructor.is_none(),
                _ => false,
            }
        }) {
            match self.get_mut(dst) {
                HeapObject::Class(c) => c.constructor = Some(ctor),
                _ => {}
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
    pub fn instance_create(&mut self, class_val: Value) -> Value {
        // Gather all (name_str, closure_val) pairs from the class before
        // touching the heap mutably, to avoid borrow conflicts.
        let class_gc = match class_val {
            Value::Class(gc) => gc,
            _ => panic!("instance_create: expected a Class value"),
        };
        let methods: Vec<(String, Value)> = match self.get(class_gc) {
            HeapObject::Class(c) => c
                .methods
                .iter()
                .map(|(n, v)| (self.to_string(*n), *v))
                .collect(),
            _ => panic!("instance_create: class Gc does not point to a Class"),
        };

        // Allocate the instance with an empty field map first so we have a
        // stable Gc handle to pass to bind_method.
        let instance_gc = self.alloc(HeapObject::Instance(GcInstance {
            class: class_val,
            fields: hashbrown::HashMap::with_capacity(methods.len()),
        }));
        let instance_val = Value::Instance(instance_gc);

        // Bind every method and store it as a field.
        for (name, closure_val) in methods {
            let bound = self.bind_method(closure_val, instance_val);
            match self.get_mut(instance_gc) {
                HeapObject::Instance(i) => {
                    i.fields.insert(name, bound);
                }
                _ => unreachable!(),
            }
        }

        instance_val
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
    pub fn bind_method(&mut self, closure_val: Value, instance: Value) -> Value {
        let gc = match closure_val {
            Value::Closure(gc) => gc,
            _ => return closure_val,
        };
        // Copy the GcClosure fields out before we mutably borrow the heap
        let closure = match self.objects[gc.0 as usize].as_ref() {
            Some(HeapObject::Closure(c)) => *c,
            _ => return closure_val,
        };
        let bound = GcClosure { instance: Some(instance), ..closure };
        Value::Closure(self.alloc(HeapObject::Closure(bound)))
    }
}
