use crate::compiler::chunk::Chunk;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Varint(pub u32);

impl Varint {
    /// Writes a u32 as a varint (1–4 bytes).
    /// Returns the number of bytes written.
    pub fn write_bytes(&self, chunk: &mut Chunk, line: i32) -> usize {
        let mut written = 0;
        let mut value = self.0;
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;

            if value != 0 {
                byte |= 0x80; // continuation bit
            }

            chunk.write(byte, line);
            written += 1;

            if value == 0 {
                break;
            }
        }

        written
    }

    /// Reads a varint starting at `offset`.
    /// Returns (value, bytes_read).
    pub fn read_bytes(chunk: &Chunk, offset: usize) -> (u32, usize) {
        let mut result = 0u32;
        let mut shift = 0;
        let mut bytes_read = 0;

        loop {
            let byte = chunk.code[offset + bytes_read];
            bytes_read += 1;

            result |= ((byte & 0x7F) as u32) << shift;

            if byte & 0x80 == 0 {
                break;
            }

            shift += 7;
            debug_assert!(shift <= 28, "varint too large");
        }

        (result, bytes_read)
    }
}
