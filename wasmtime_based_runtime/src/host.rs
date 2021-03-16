#[cfg(feature = "enable_wasi")]
use wasmtime_wasi::{Wasi, WasiCtx};

use manager::Manager;

use std::collections::HashMap;

use wasmtime::*;

use crate::global::{
    access_immutable_memory, wasm_malloc, wasm_memory, wasm_table, write_error, InstanceInfo,
    DLERROR, GOT_FUNC, GOT_MEM, INSTANCES, LINKER, MAIN, MALLOC, MANAGER, MEMORY, TABLE,
};
use crate::leb128::Dylink;

pub type OurResult<T> = Result<T, Box<dyn std::error::Error>>;

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($($arg:tt)+) => (
        #[cfg(feature = "use_log")]
        log::info!($($arg)+);
        #[cfg(feature = "use_println")]
        std::println!($($arg)+);
    )
}

#[macro_export(local_inner_macros)]
macro_rules! error {
    ($($arg:tt)+) => (
        #[cfg(feature = "use_log")]
        log::error!($($arg)+);
        #[cfg(feature = "use_println")]
        std::println!($($arg)+);
    )
}

fn insert_function(
    table: Table,
    func: Func,
    function_name: String,
    got: &mut HashMap<String, u32>,
) -> Result<u32, Trap> {
    if table.size() == 0 {
        // prevent return of 0 index
        table.grow(1, Val::FuncRef(None))?;
    }
    table.grow(1, func.into())?;
    got.insert(function_name, table.size() - 1);
    Ok(table.size() - 1)
}

pub fn dlerror(store: &Store) -> Func {
    Func::wrap(&store, || -> u32 {
        DLERROR.with(|error_ref| {
            let mut error = error_ref.borrow_mut();
            if !error.occurred {
                0
            } else {
                error.occurred = false;
                error.position as u32
            }
        })
    })
}

fn link_binaries(
    binaries: &Vec<(manager::Dependency, Vec<u8>)>,
    linker: &mut Linker,
) -> Result<u32, Trap> {
    let mut loaded_module_ids = HashMap::new();
    INSTANCES.with(|i| {
        let instances = (*i).lock().unwrap();
        for instance_info in instances.iter() {
            loaded_module_ids.insert(instance_info.name.clone(), instance_info.instance.clone());
        }
    });

    for (dependency, binary) in binaries {
        if let Some(instance) = loaded_module_ids.get(&dependency.id) {
            continue;
        }
        let info = Dylink::check_dylink(&binary)?;

        let table = wasm_table()?;
        let table_base = table.grow(info.table_size, Val::FuncRef(None))?;

        let memory_base = wasm_malloc(info.mem_size)? as i32;
        let memory_base_global = Global::new(
            linker.store(),
            GlobalType::new(ValType::I32, Mutability::Const),
            memory_base.into(),
        )?;
        linker.define("env", "__memory_base", memory_base_global)?;

        let table_base_global = Global::new(
            linker.store(),
            GlobalType::new(ValType::I32, Mutability::Const),
            (table_base as i32).into(),
        )?;
        linker.define("env", "__table_base", table_base_global)?;

        linker.define(
            "host",
            "dlopen",
            dlopen(linker.store(), dependency.id.clone()),
        )?;

        let module = Module::from_binary(linker.store().engine(), &binary)?;

        let mut globals_func = HashMap::new();
        let mut globals_mem = HashMap::new();

        for import in module.imports() {
            if import.module() == "GOT.func" {
                let global = Global::new(
                    linker.store(),
                    GlobalType::new(ValType::I32, Mutability::Var),
                    0.into(),
                )?;

                globals_func.insert(import.name(), global.clone());
                linker.define("GOT.func", import.name(), global)?;
            } else if import.module() == "GOT.mem" && import.ty().global().is_some() {
                let global = Global::new(
                    linker.store(),
                    GlobalType::new(ValType::I32, Mutability::Var),
                    0.into(),
                )?;

                globals_mem.insert(import.name(), global.clone());
                linker.define("GOT.mem", import.name(), global)?;
            } else if import.module() == "env" {
                let exter = linker.get_one_by_name(import.module(), import.name());
                if exter.is_err() {
                    let exter = INSTANCES.with(|instance_ref| {
                        let instances = (*instance_ref).lock().unwrap();
                        for instance_info in &*instances {
                            let export = instance_info.instance.get_export(import.name());
                            if export.is_some() {
                                return export;
                            }
                        }
                        None
                    });

                    if let Some(item) = exter {
                        linker.define("env", import.name(), item)?;
                    } else {
                        return Err(Trap::new(format!(
                            "Couldn't find {}::{}",
                            import.module(),
                            import.name()
                        )));
                    }
                }
            }
        }

        let instance = linker.instantiate(&module).or_else(|err| {
            Err(Trap::new(format!(
                "Unable to instantiate `{}` module: {}",
                dependency.id, err
            )))
        })?;
        linker.instance(&dependency.id, &instance)?;

        GOT_MEM.with(|got_ref| -> Result<(), Trap> {
            let mut got = got_ref.borrow_mut();
            for (name, global) in globals_mem {
                let memory_index: Result<i32, Trap> = match got.get(name) {
                    Some(index) => Ok(*index),
                    None => {
                        // didnt find from GOT
                        let index = {
                            // find from the currently instantiated instance
                            if let Some(offset_global) = instance.get_global(name) {
                                let index = memory_base + offset_global.get().i32()
                                    .ok_or_else(|| Trap::new(format!(
                                        "global {} in {} is not i32",
                                        name, dependency.id
                                    )))?;
                                index
                            } else {
                                // find from any existing instance
                                let index_option: Option<i32> = INSTANCES.with(|instance_ref| {
                                    let instances = (*instance_ref).lock().unwrap();
                                    for instance_info in &*instances {
                                        if let Some(offset_global) = instance_info.instance.get_global(name)
                                        {
                                            if let Some(offset) = offset_global.get().i32() {
                                                return Ok(Some(instance_info.memory_base + offset));
                                            } else {
                                                return Err(Trap::new(format!(
                                                    "global {} in {} is not i32",
                                                    name, dependency.id
                                                )));
                                            }
                                        }
                                    }
                                    Ok(None)
                                })?;

                                if let Some(index) = index_option {
                                    index
                                } else {
                                    return Err(Trap::new(format!(
                                        "Couldn't find GOT.mem.{}",
                                        name
                                    )));
                                }
                            }
                        };
                        got.insert(String::from(name), index);
                        Ok(index)
                    }
                };

                global.set(Val::I32(memory_index? as i32))?;
            }
            Ok(())
        })?;

        GOT_FUNC.with(|got_ref| -> Result<(), Trap> {
            let mut got = got_ref.borrow_mut();
            for (name, global) in globals_func {
                let function_index: Result<u32, Trap> = match got.get(name) {
                    Some(index) => Ok(*index),
                    None => {
                        let func = || -> Result<Func, Trap> {
                            let get_func = || {
                                for (_, item_name, value) in linker.iter() {
                                    if let Some(func) = value.into_func() {
                                        if item_name == name {
                                            return Some(func);
                                        }
                                    }
                                }
                                None
                            };

                            if let Some(func) = get_func() {
                                Ok(func)
                            } else {
                                return Err(Trap::new(format!("Couldn't find GOT.func.{}", name)));
                            }
                        }()?;

                        let index =
                            insert_function(table.clone(), func, name.to_string(), &mut got)?;

                        Ok(index)
                    }
                };

                global.set(Val::I32(function_index? as i32))?;
            }
            Ok(())
        })?;

        if let Some(post_instantiate) = instance.get_func("__post_instantiate") {
            post_instantiate.call(&[])?;
        }

        let len = INSTANCES.with(|i| {
            let mut instances = (*i).lock().unwrap();
            instances.push(InstanceInfo {
                name: dependency.id.clone(),
                instance: instance,
                dylink: Some(info),
                memory_base,
                table_base: Some(table_base),
            });
            instances.len()
        });
    }

    // The module we load should be the last one we loaded in current implementation
    let handle = INSTANCES.with(|i| {
        let instances = (*i).lock().unwrap();
        instances.len()
    });

    Ok(handle as u32)
}

pub fn dlopen(store: &Store, caller_module: String) -> Func {
    Func::wrap(&store, move |_: Caller<'_>, id_ptr: i32| -> u32 {
        let ret = LINKER.with(|m| -> Result<u32, Trap> {
            let mut guard = m.lock().unwrap();
            let a = guard.as_mut();
            let linker = a.unwrap();

            let memory = wasm_memory()?;
            let id = unsafe { access_immutable_memory(&memory, id_ptr)? };
            let dependency = MANAGER.with(|m| {
                let mut manager = m.borrow_mut();
                match manager.as_mut() {
                    Some(manager) => manager.resolve_id(&caller_module, &id),
                    None => unreachable!(),
                }
            })?;
            let module_identifier = dependency.id.clone();

            if let Some(index) = INSTANCES.with(|i| {
                let instances = (*i).lock().unwrap();
                for (i, instance_info) in instances.iter().enumerate() {
                    if instance_info.name == module_identifier {
                        return Some(i);
                    }
                }
                None
            }) {
                return Ok((index + 1) as u32);
            }

            let handle_result = {
                MANAGER.with(|m| {
                    let mut manager = m.borrow_mut();
                    let binaries = match manager.as_mut() {
                        Some(manager) => manager.load(dependency),
                        None => unreachable!(),
                    };
                    let handle = link_binaries(&binaries?, linker);
                    handle
                })
            };
            handle_result
        });

        if let Err(err) = ret {
            error!("{}", &err.to_string());
            write_error(&err.to_string());
            return 0;
        }

        ret.unwrap()
    })
}

pub fn dlsym(store: &Store) -> Func {
    Func::wrap(
        &store,
        |_: Caller<'_>, handle: i32, function_name: i32| -> u32 {
            let ret = || -> Result<u32, Trap> {
                let memory = wasm_memory()?;
                let function_name = unsafe { access_immutable_memory(&memory, function_name)? };
                if let Some(i) = GOT_FUNC.with(|got_ref| -> Option<u32> {
                    let got = got_ref.borrow();
                    if let Some(index) = got.get(&function_name) {
                        return Some(*index);
                    }

                    None
                }) {
                    return Ok(i);
                };

                let table = wasm_table()?;
                let side_instance = INSTANCES.with(|i| {
                    let mut guard = (*i).lock().unwrap();
                    match guard.get_mut((handle - 1) as usize) {
                        Some(instance_info) => Ok(instance_info.instance.clone()),
                        None => Err(Trap::new(format!("Instance with handle {} not found", handle))),
                    }
                })?;

                let func = side_instance.get_func(&function_name).ok_or_else(|| {
                    Trap::new("The item is either not a function or was not found (dlsym)")
                })?;

                GOT_FUNC.with(|got_ref| {
                    let mut got = got_ref.borrow_mut();

                    insert_function(table, func, function_name, &mut got)
                })
            }();

            if let Err(err) = ret {
                error!("{}", &err.to_string());
                write_error(&err.to_string());
                return 0;
            }

            ret.unwrap()
        },
    )
}

pub fn get_platform_string() -> &'static str {
    #[cfg(target_os = "android")]
    {
        return "aarch64_android";
    }
    #[cfg(not(target_os = "android"))]
    {
        #[cfg(target_arch = "aarch64")]
        return "aarch64";

        #[cfg(target_arch = "x86_64")]
        return "x86_64";
    }
}

pub fn fopen_and_read(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, filename: i32, length: i32| -> u32 {
        let ret = || -> Result<u32, Trap> {
            let memory = wasm_memory()?;
            let filename = unsafe { access_immutable_memory(&memory, filename)? };

            let contents = std::fs::read(filename).map_err(|e| Trap::new(e.to_string()))?;
            let pointer = wasm_malloc(contents.len() as u32)?;
            unsafe {
                &memory.data_unchecked_mut()[pointer as usize..pointer as usize + contents.len()]
                    .copy_from_slice(&contents)
            };

            let len = contents.len() as u32;

            unsafe {
                memory.data_unchecked_mut()[length as usize..(length + 4) as usize]
                    .copy_from_slice(&len.to_le_bytes());
            }

            Ok(pointer)
        }();

        if let Err(err) = ret {
            write_error(&err.to_string());
            return 0;
        }

        ret.unwrap()
    })
}

pub fn write_file(store: &Store) -> Func {
    Func::wrap(
        &store,
        |_: Caller<'_>, filename: i32, buffer: i32, length: i32| -> u32 {
            let ret = LINKER.with(|m| -> Result<u32, Trap> {
                let mut guard = m.lock().unwrap();
                let a = guard.as_mut();
                let linker = a.unwrap();

                let memory = linker
                    .get_one_by_name("env", "memory")?
                    .into_memory()
                    .ok_or_else(|| {
                        Trap::new("The external is not a func found in linker (dlsym)")
                    })?;

                let data: &[u8] = unsafe {
                    &memory.data_unchecked()[buffer as usize..(buffer + length) as usize]
                };

                let filename = unsafe { access_immutable_memory(&memory, filename) }?;

                std::fs::write(std::path::Path::new(&filename), data)
                    .map_err(|e| Trap::new(e.to_string()))?;

                return Ok(1);
            });

            if let Err(err) = ret {
                write_error(&err.to_string());
                return 0;
            }

            return ret.unwrap();
        },
    )
}

pub fn run_main() {
    info!("Going to run the module");
    MAIN.with(|entrypoint| -> Result<(), Trap> {
        let mut entrypoint = entrypoint.borrow_mut();
        let result = entrypoint.as_mut().unwrap().call(&[]);
        if result.is_ok() {
            info!("RESULT: {:#?}", result.unwrap());
        } else {
            let err = result.unwrap_err();
            if let Some(trap) = err.downcast_ref::<Trap>() {
                if let Some(status) = trap.i32_exit_status() {
                    info!("RESULT: {:#?}", status);
                    return Ok(());
                }
            }
            info!("RESULT: {:#?}", err);
        }
        Ok(())
    })
    .unwrap();
}

pub fn create_linker(main_module_name: &str) -> Result<Linker, Trap> {
    let store = Store::default();
    let mut linker = Linker::new(&store);
    linker.allow_shadowing(true);

    #[cfg(feature = "enable_wasi")]
    {
        let wasi = Wasi::new(&store, WasiCtx::new(std::env::args()).expect("Wasi Error"));
        wasi.add_to_linker(&mut linker)?;
    }

    // Emscripten specific import that can be a no-op
    // https://github.com/emscripten-core/emscripten/issues/11954#issuecomment-675632031
    linker.func("env", "emscripten_notify_memory_growth", |index: i32| {
        // info!("Grew memory index {}", index);
    })?;

    linker.define(
        "host",
        "dlopen",
        dlopen(&store, main_module_name.to_string()),
    )?;
    linker.define("host", "dlsym", dlsym(&store))?;
    linker.define("host", "dlerror", dlerror(&store))?;
    linker.define("host", "fopen_and_read", fopen_and_read(&store))?;
    linker.define("host", "write_file", write_file(&store))?;

    Ok(linker)
}

pub fn init_logging() {
    #[cfg(feature = "use_log")]
    {
        #[cfg(feature = "desktop_build")]
        {
            use simplelog::*;
            SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();
        }
        #[cfg(feature = "android_build")]
        {
            android_logger::init_once(
                android_logger::Config::default().with_min_level(log::Level::Info),
            );
        }
    }
}

pub fn initialize(
    main_metadata: &str,
    main_module_name: &str,
    cache_path: &str,
    attributes: Vec<String>,
    mut linker: Linker,
    store: Store,
) -> OurResult<()> {
    info!("cache_path: {}", cache_path);
    let mut manager = Manager::new(
        &main_metadata,
        attributes,
        &Some(format!("{}/cache", cache_path)),
    )?;

    let main_module = Module::from_binary(store.engine(), &manager.load_main(main_module_name)?)?;
    MANAGER.with(|m| {
        let mut option_manager = m.borrow_mut();
        *option_manager = Some(manager);
    });

    for import in main_module.imports() {
        if import.module() == "env" {
            if import.name() == "table" {
                // Create table if one required
                if let ExternType::Table(t) = import.ty() {
                    let table_ty = TableType::new(ValType::FuncRef, t.limits().clone());
                    let table = Table::new(&store, table_ty, Val::FuncRef(None))?;
                    linker.define("env", "table", table)?;
                }
            } else if import.name() == "memory" {
                // Create memory if one required
                if let ExternType::Memory(m) = import.ty() {
                    let memory_ty = MemoryType::new(m.limits().clone());
                    let memory = Memory::new(&store, memory_ty);
                    linker.define("env", "memory", memory)?;
                }
            }
        }
    }
    info!("Should be initalizing,");

    let main_instance = linker.instantiate(&main_module)?;
    linker.instance(&main_module_name, &main_instance)?;

    let memory = main_instance
        .get_memory("memory")
        .ok_or(Trap::new("A memory is not exported (main)"))?;
    MEMORY.with(|v| {
        v.replace(Some(memory.clone()));
    });

    let table = main_instance
        .get_table("__indirect_function_table")
        .or(main_instance.get_table("table"))
        .ok_or(Trap::new("A table is not exported (main)"))?;
    TABLE.with(|v| {
        v.replace(Some(table.clone()));
    });

    let malloc = linker
        .get_one_by_name(main_module_name, "malloc")
        .or(Err(Trap::new("The malloc not exported (main)")))?
        .into_func()
        .ok_or(Trap::new("The malloc export is not a function (main)"))?;
    MALLOC.with(|v| {
        v.replace(Some(malloc.clone()));
    });

    DLERROR.with(|error_ref| -> Result<(), Trap> {
        let mut error = error_ref.borrow_mut();
        error.memory = Some(memory.clone());
        error.position = wasm_malloc(error.size as u32)? as usize;
        Ok(())
    })?;

    INSTANCES.with(|i| {
        let mut instances = (*i).lock().unwrap();
        instances.push(InstanceInfo {
            name: main_module_name.to_string(),
            instance: main_instance.clone(),
            dylink: None,
            memory_base: 0, // Any offsets are relative to 0 for main module
            table_base: None,
        });
    });

    let entrypoint = linker.get_default(main_module_name).or_else(|err| {
        Err(Trap::new(format!(
            "No main function found (neither _start nor __original_main): {}",
            err.to_string()
        )))
    })?;

    LINKER.with(|m| {
        *m.lock().unwrap() = Some(linker);
    });

    MAIN.with(|rc| {
        *rc.borrow_mut() = Some(entrypoint);
    });
    Ok(())
}
