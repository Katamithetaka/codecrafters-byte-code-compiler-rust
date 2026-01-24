use crate::{
    compiler::{
        varint::Varint,
    },
    expressions::Value,
};

#[derive(Debug)]
pub struct Chunk<'a> {
    pub code: Vec<u8>,
    pub constants: Vec<Value<&'a str>>,
    pub lines: Vec<(i32, usize)>,
}

impl<'a> Chunk<'a> {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            constants: vec![],
            lines: vec![],
        };
    }


    pub fn get_constant(&mut self, value: &Value<&'a str>) -> Option<Varint> {
        self.constants.iter().position(|e| e == value).map(|v| {
            return Varint(v as u32);
        })
    }

    pub fn add_constant(&mut self, value: Value<&'a str>) -> Varint {
        self.constants.push(value);
        return Varint((self.constants.len() - 1) as u32);
    }

    pub fn write(&mut self, byte: u8, line: i32) {
        let line_v = self.lines.last_mut();
        match line_v {
            Some(a) if a.0 == line => a.1 += 1,
            _ => self.lines.push((line, 1)),
        }

        self.code.push(byte);
    }


    pub fn get_line(&self, offset: usize) -> i32 {
        let mut i = 0;
        let mut line_index = 0;
        while i < offset {
            i += self.lines[line_index].1;
            if i >= offset {
                break;
            }
            if line_index + 1 >= self.lines.len() {
                break;
            }
            line_index += 1;
        }
        return self.lines[line_index].0;
    }
}

impl Default for Chunk<'_> {
    fn default() -> Self {
        Self::new()
    }
}
