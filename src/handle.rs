use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use syscall::{EBADF, EBUSY, Error};

pub struct NodeHandle {
    path: String,
    id: usize,
    mode: OpenMode,
    uid: u32,
    seek: usize,
    // TODO: Options, read write etc.
}

impl NodeHandle {
    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_seek(&self) -> usize {
        self.seek
    }

    pub fn set_seek(&mut self, seek: usize) {
        self.seek = seek;
    }

    pub fn advance_seek(&mut self, amt: usize) {
        self.seek += amt;
    }
}

impl NodeHandle {
    pub fn new(path: String, id: usize, mode: OpenMode, uid: u32) -> Self {
        Self {
            path,
            id,
            mode,
            uid,
            seek: 0
        }
    }
}

pub struct HandleMap {
    id_map: HashMap<usize, String>,
    handle_paths: HashMap<String, Vec<NodeHandle>>,
    next_id: AtomicUsize
}

#[derive(PartialEq, Eq)]
pub enum OpenMode {
    Read,
    Write,
    ReadWrite,
}

impl HandleMap {
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            handle_paths: HashMap::new(),
            next_id: AtomicUsize::new(1)
        }
    }

    pub fn open_handle(&mut self, path: &str, uid: u32, open_mode: OpenMode) -> syscall::Result<&mut NodeHandle> {
        let path = path.trim_matches('/');
        // Check if parent nodes are being written to
        for (i, _) in path.char_indices().filter(|(_i, c)| *c == '/').into_iter() {
            let key = &path[0..i];
            println!("Checking key {}", key);
            if let Some(handles) = self.handle_paths.get(key) {
                if !handles.is_empty() && (open_mode == OpenMode::Write || open_mode == OpenMode::ReadWrite) {
                    eprintln!("Cannot open for writing - Not the only one writing.!");
                    return Err(Error::new(EBUSY));
                }
            }
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let handle = NodeHandle::new(path.to_owned(), id, open_mode, uid);
        self.id_map.insert(id, path.to_owned());
        let vec = self.handle_paths.entry(path.to_owned()).or_insert_with(|| Vec::new());
        vec.push(handle);
        Ok(vec.last_mut().unwrap())
    }

    pub fn get_handle_mut(&mut self, id: usize) -> syscall::Result<&mut NodeHandle> {
        if let Some(path) = self.id_map.get(&id) {
            Ok(self.handle_paths.get_mut(path).expect("Should exist.").iter_mut().find(|h| h.id == id).expect("Should exist 2"))
        }
        else {
            Err(Error::new(EBADF))
        }
    }

    pub fn close(&mut self, id: usize) -> syscall::Result<usize> {
        if let Some(handle_path) = self.id_map.remove(&id) {
            let vec = self.handle_paths.get_mut(&handle_path).expect("Handle path should be in the handle paths map.");
            let (idx, _h) = vec.iter().enumerate().find(|(_i, h)| h.id == id).expect("Should have found the handle in the vec");
            vec.swap_remove(idx);
            Ok(0)
        }
        else {
            Err(Error::new(EBADF))
        }
    }
}