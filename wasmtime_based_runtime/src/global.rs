use manager::Manager;

use std::collections::HashMap;
use std::{cell::RefCell};
use std::{sync::Mutex};

use crate::leb128::Dylink;
use wasmtime::*;

pub struct ErrorInformation {
    pub size: usize,
    pub position: usize,
    pub occurred: bool,
    pub memory: Option<Memory>,
}

pub struct InstanceInfo {
    pub name: String,
    pub instance: Instance,
    pub dylink: Option<Dylink>,
    pub memory_base: i32,
    pub table_base: Option<u32>,
}

thread_local!(
    pub static MAIN: RefCell<Option<Func>> = RefCell::new(None);
    pub static MANAGER: RefCell<Option<Manager>> = RefCell::new(None);

    pub static INSTANCES: Mutex<Vec<InstanceInfo>> = Mutex::new(Vec::new());
    pub static LINKER: Mutex<Option<Linker>> = Mutex::new(None);
    pub static DLERROR: RefCell<ErrorInformation> = RefCell::new(ErrorInformation {
        size: 256,
        position: 0,
        occurred: false,
        memory: None,
    });

    pub static GOT_FUNC: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
    pub static GOT_MEM: RefCell<HashMap<String, i32>> = RefCell::new(HashMap::new());

    pub static MEMORY: RefCell<Option<Memory>> = RefCell::new(None);
    pub static TABLE: RefCell<Option<Table>> = RefCell::new(None);
    pub static MALLOC: RefCell<Option<Func>> = RefCell::new(None);
);

pub fn reset_globals() {
    MAIN.with(|rc| {
        let mut refmut = rc.borrow_mut();
        *refmut = None;
    });
    MANAGER.with(|rc| {
        let mut refmut = rc.borrow_mut();
        *refmut = None;
    });
    INSTANCES.with(|m_vec| {
        let mut vec = (*m_vec).lock().unwrap();
        *vec = Vec::new();
    });
    LINKER.with(|m_ol| {
        let mut ol = (*m_ol).lock().unwrap();
        *ol = None;
    });
    DLERROR.with(|rc| {
        let mut refmut = rc.borrow_mut();
        *refmut = ErrorInformation {
            size: 256,
            position: 0,
            occurred: false,
            memory: None,
        };
    });
    GOT_FUNC.with(|rc| {
        let mut refmut = rc.borrow_mut();
        *refmut = HashMap::new();
    });
    GOT_MEM.with(|rc| {
        let mut refmut = rc.borrow_mut();
        *refmut = HashMap::new();
    });
    MEMORY.with(|rc| {
        let mut refmut = rc.borrow_mut();
        *refmut = None;
    });
    TABLE.with(|rc| {
        let mut refmut = rc.borrow_mut();
        *refmut = None;
    });
    MALLOC.with(|rc| {
        let mut refmut = rc.borrow_mut();
        *refmut = None;
    });
}

pub fn wasm_malloc(size: u32) -> Result<u32, Trap> {
    MALLOC.with(|v| -> Result<u32, Trap> {
        let func = v.borrow();
        let malloc = func.clone().ok_or(Trap::new("MALLOC is None"))?;
        let malloc = malloc
            .get1::<u32, u32>()
            .or(Err(Trap::new("MALLOC signature doesnt match u32(u32)")))?;
        let result = malloc(size);
        Ok(result?)
    })
}

pub fn wasm_table() -> Result<Table, Trap> {
    TABLE.with(|v| -> Result<Table, Trap> {
        let table = v.borrow().clone().ok_or(Trap::new("TABLE is None"))?;
        Ok(table)
    })
}

pub fn wasm_memory() -> Result<Memory, Trap> {
    MEMORY.with(|v| -> Result<Memory, Trap> {
        let memory = v.borrow().clone().ok_or(Trap::new("MEMORY is None"))?;
        Ok(memory)
    })
}

pub unsafe fn access_immutable_memory(memory: &Memory, name: i32) -> Result<String, Trap> {
    let data: &[u8] = &memory.data_unchecked()[name as usize..];
    for (index, byte) in (&data).iter().enumerate() {
        if byte.to_owned() as char == '\0' {
            match std::str::from_utf8(&data[..index]) {
                Ok(s) => return Ok(s.to_string()),
                Err(e) => return Err(Trap::new(e.to_string())),
            }
        }
    }

    Err(Trap::new("Data has no '\0' byte"))
}

pub fn write_error(error_message: &str) {
    DLERROR.with(|error_ref| {
        let mut error = error_ref.borrow_mut();
        assert!(error.size >= 1);
        error.occurred = true;
        let memory = error.memory.clone().unwrap();
        let error_len = std::cmp::min(error.size, error_message.len());
        unsafe {
            memory.data_unchecked_mut()[error.position..error.position + error_len]
                .copy_from_slice(error_message[..error_len].as_bytes());
            memory.data_unchecked_mut()[error.position + error_len] = b'\0';
        }
    })
}
