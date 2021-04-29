use crate::errors::MemoryReservationError;

#[derive(Debug)]
pub struct MemoryHandle {
    start: usize,
    len: usize,
    pos: usize,
}

impl MemoryHandle {
    pub fn new(start: usize, len: usize) -> MemoryHandle {
        MemoryHandle { start, len, pos: 0 }
    }

    pub fn allocate(
        &mut self,
        len: usize,
        _alignment: u32,
    ) -> Result<usize, MemoryReservationError> {
        //TODO alignment
        let segment_start = self.start + self.pos;
        //TODO overflow
        self.pos += len;
        Ok(segment_start)
    }
}
