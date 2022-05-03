use std::fs::{File};
use std::io::{Read, Write};

use syscall::{O_ACCMODE, O_RDONLY, O_TRUNC, O_WRONLY, Packet};
use syscall::scheme::SchemeMut;
use syscall::error::Result;
use handle::HandleMap;
use crate::handle::OpenMode;
use crate::storage::{MemStorage, Storage};

mod handle;
mod storage;

struct ConfigScheme {
    //storage_file: File,
    handles: HandleMap,

    storage: MemStorage,
}

impl ConfigScheme {
    fn new(/*storage_path: File*/) -> ConfigScheme {
        ConfigScheme {
            //storage_file: storage_path,
            handles: HandleMap::new(),
            storage: MemStorage::new(),
        }
    }
}

fn main() {
    //let options = OpenOptions::new().create(true).write(true);
    //let storage_file = File::open("file:/config.f", )
    //    .expect("Failed to access config.f");
    let mut scheme = ConfigScheme::new(/*storage_file*/);

    println!("Creating scheme.");
    let mut handler = File::create(":config")
        .expect("Failed to create the config scheme");

    let mut packet = Packet::default();

    println!("Starting loop");
    loop {
        // Wait for the kernel to send us requests
        let read_bytes = handler.read(&mut packet)
            .expect("config: failed to read event from vec scheme handler");

        println!("Read bytes");
        if read_bytes == 0 {
            // Exit cleanly
            break;
        }

        // Scheme::handle passes off the info from the packet to the individual
        // scheme methods and writes back to it any information returned by
        // those methods.
        println!("handling");
        scheme.handle(&mut packet);

        println!("writing.");
        handler.write(&packet)
            .expect("config: failed to write response to config scheme handler");
    }
}

impl SchemeMut for ConfigScheme {
    fn open(&mut self, path: &str, flags: usize, uid: u32, _gid: u32) -> Result<usize> {
        println!("open {}", path);
        let v = flags & O_ACCMODE;
        let open_mode = match v {
            _ if v == O_RDONLY => OpenMode::Read,
            _ if v == O_WRONLY => OpenMode::Write,
            _ => OpenMode::ReadWrite,
        };
        let handle = self.handles.open_handle(path, uid, open_mode)?;
        if flags & O_TRUNC == O_TRUNC {
            println!("truncating..");
            self.storage.trunc(handle)?;
        }
        Ok(handle.get_id())
    }

    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        let handle = self.handles.get_handle_mut(id)?;
        self.storage.read(handle, buf)
    }

    // Simply push any bytes we are given to self.vec
    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        println!("write {}", id);
        let handle = self.handles.get_handle_mut(id)?;
        self.storage.write(handle, buf)
    }

    fn close(&mut self, id: usize) -> Result<usize> {
        println!("close");
        self.handles.close(id)
    }
}
