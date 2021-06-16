use embedded_storage::{ReadStorage, Storage};

pub struct FlashStorage {
    read_buf: Vec<u8>,
}

impl FlashStorage {
    pub fn new(read_buf: &[u8]) -> Self {
        Self {
            read_buf: read_buf.to_vec(),
        }
    }
}

impl ReadStorage for FlashStorage {
    type Error = probe_rs::Error;

    fn try_read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        // println!("o: {}, bl: {}", offset, bytes.len());
        let offset = offset as usize;
        if offset + bytes.len() >= self.capacity() {
            return Ok(());
        }

        let slice = &self.read_buf[offset..(offset + bytes.len())];
        bytes.copy_from_slice(slice);

        Ok(())
    }

    fn capacity(&self) -> usize {
        self.read_buf.len()
    }
}

impl Storage for FlashStorage {
    fn try_write(&mut self, _offset: u32, _bytes: &[u8]) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
