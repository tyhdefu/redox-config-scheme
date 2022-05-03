use std::cmp::min;
use std::collections::HashMap;
use crate::handle::NodeHandle;

pub trait Storage {
    fn read(&self, handle: &mut NodeHandle, buf: &mut [u8]) -> syscall::Result<usize>;

    fn write(&mut self, handle: &mut NodeHandle, buf: &[u8]) -> syscall::Result<usize>;

    fn trunc(&mut self, handle: &mut NodeHandle) -> syscall::Result<()>;
}

pub struct MemStorage {
    data: HashMap<String, String>,
}

impl MemStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Storage for MemStorage {
    fn read(&self, handle: &mut NodeHandle, buf: &mut [u8]) -> syscall::Result<usize> {
        println!("read id {}", handle.get_id());

        let opt_data = self.data.get(handle.get_path());
        println!("Data {:?}", opt_data);
        let amt = opt_data.map(|d| {
            if handle.get_seek() >= d.len() {
                return 0;
            }
            let amount_read = min(buf.len(), d.len() - handle.get_seek());
            let as_bytes = d.as_bytes();
            buf[0..amount_read].copy_from_slice(&as_bytes[0..amount_read]);
            handle.advance_seek(amount_read);
            amount_read
        }).unwrap_or(0);
        println!("Read count: {}", amt);
        Ok(amt)
    }

    fn write(&mut self, handle: &mut NodeHandle, buf: &[u8]) -> syscall::Result<usize> {
        let string = self.data.entry(handle.get_path().to_owned())
            .or_insert(String::from(""));

        string.push_str(&String::from_utf8_lossy(buf));

        Ok(buf.len())
    }

    fn trunc(&mut self, handle: &mut NodeHandle) -> syscall::Result<()> {
        self.data.get_mut(handle.get_path()).map(|s| s.clear());
        Ok(())
    }
}