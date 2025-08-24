use bytes::*;
// this is a helper for the bytes
// if the decoder has a buffer typed bytes and a cursor typed usize
// this will be useful
// because the Bytes struct is a zero copy buffer
// clone it is cheap

pub trait BytesCursorHelper {
    fn inner_buffer(&self) -> Bytes;
    fn inner_cursor(&self) -> usize;
    fn set_inner_cursor(&mut self, cursor: usize);
    fn seek_next(&mut self, n: usize) {
        self.set_inner_cursor(self.inner_cursor() + n);
    }
    fn seek_start(&mut self) {
        self.set_inner_cursor(0);
    }
    fn seek_start_next(&mut self, n: usize) {
        self.set_inner_cursor(n);
    }
    fn seek_end(&mut self) {
        self.set_inner_cursor(self.inner_buffer().len());
    }
    fn seek_end_before(&mut self, n: usize) {
        self.set_inner_cursor(self.inner_buffer().len() - n);
    }
    fn read(&mut self, size: usize) -> Bytes {
        let cursor = self.inner_cursor();
        let buf = self.inner_buffer().slice(cursor..cursor + size);
        self.seek_next(size);
        buf
    }
    fn read_to_end(&mut self) -> Bytes {
        let cursor = self.inner_cursor();
        let buf = self.inner_buffer().slice(cursor..self.inner_buffer().len());
        self.seek_end();
        buf
    }
    fn read_sized<const SIZE: usize>(&mut self) -> [u8; SIZE] {
        use crate::internal::utils::SafeArrayConvert;
        let cursor = self.inner_cursor();
        let buf = self.inner_buffer().slice(cursor..cursor + SIZE);
        self.seek_next(SIZE);
        buf.to_vec()
            .try_into_array()
            .unwrap_or_else(|_| panic!("Buffer size mismatch"))
    }
}

#[derive(Clone, Default)]
pub struct EasyBytesWithCursor {
    pub buffer: Bytes,
    pub cursor: usize,
}

impl BytesCursorHelper for EasyBytesWithCursor {
    fn inner_buffer(&self) -> Bytes {
        self.buffer.clone()
    }
    fn inner_cursor(&self) -> usize {
        self.cursor
    }
    fn set_inner_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }
}

impl EasyBytesWithCursor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn create(buf: Bytes) -> Self {
        Self {
            buffer: buf,
            cursor: 0,
        }
    }
}
