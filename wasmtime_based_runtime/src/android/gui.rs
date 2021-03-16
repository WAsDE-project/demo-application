use std::{cell::RefCell, collections::HashMap};

use wasmtime::*;

use crate::android::java;
use crate::android::java::{CLASS, ENV};
use crate::global::{access_immutable_memory};

thread_local! {
    pub static BUTTON_PRESSES: RefCell<HashMap<i32, u32>> = RefCell::new(HashMap::new());
    pub static ON_TICK_HANDLERS: RefCell<Vec<u32>> = RefCell::new(Vec::new());
}

fn get_string_from_wasm_memory(str_ptr: i32) -> String {
    use crate::global::wasm_memory;
    let ret = || -> Result<String, Trap> {
        let memory = wasm_memory()?;
        unsafe { access_immutable_memory(&memory, str_ptr) }
    }();
    match ret {
        Ok(string) => string,
        Err(e) => panic!("{}", e.to_string()),
    }
}

// pub fn create_text_view(ctx: &mut Ctx, text: WasmPtr<u8, Array>) -> i32 {
//     // Fetch the text for the textview from wasm memory
//     let memory = ctx.memory(0);
//     let text = text.get_utf8_string_with_nul(memory).unwrap();
//     // Get the java environment
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();
//     // Call java code.
//     java::create_text_view(&env, &class, text)
// }

pub fn create_text_view(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, text: i32| -> i32 {
        //Get the label text from memory
        let text = get_string_from_wasm_memory(text);

        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::create_text_view(&env, &class, &text)
            })
        })
    })
}

// pub fn modify_text_view(ctx: &mut Ctx, id: i32, text: WasmPtr<u8, Array>) {
//     // Fetch the text for the textview from wasm memory
//     let memory = ctx.memory(0);
//     let text = text.get_utf8_string_with_nul(memory).unwrap();
//     // Get the java environment
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();
//     // Call java code.
//     java::modify_text_view(&env, &class, id, text)
// }

pub fn modify_text_view(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, id, text: i32| {
        //Get the label text from memory
        let text = get_string_from_wasm_memory(text);

        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::modify_text_view(&env, &class, id, &text);
            })
        })
    })
}

// pub fn remove_text_view(id: i32) {
//     // Get the java environment
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();
//     // Call java code.
//     java::remove_text_view(&env, &class, id)
// }

pub fn remove_text_view(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, id| {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::remove_text_view(&env, &class, id);
            })
        })
    })
}

// pub fn create_button(ctx: &mut Ctx, label: WasmPtr<u8, Array>) -> i32 {
//     // Fetch the label text for the button
//     let memory = ctx.memory(0);
//     let label = label.get_utf8_string_with_nul(memory).unwrap();
//     // Get the java environment
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();
//     // Call java code.
//     java::create_button(&env, &class, label)
// }

pub fn create_button(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, label: i32| -> i32 {
        //Get the label text from memory
        let label = get_string_from_wasm_memory(label);

        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::create_button(&env, &class, &label)
            })
        })
    })
}

// pub fn register_on_click(ctx: &mut Ctx, id: i32, callback: u32) -> u32 {
//     let callback = TableIndex::new(callback as usize);
//     // Currently this doesn't map the table indexes to specific modules so this is an
//     // implementation for a single module project (which pacman is currently).
//     let mut press_map = &mut *BUTTON_PRESSES.lock().unwrap();
//     press_map.insert(id, callback);
//     0
// }

pub fn register_on_click(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, id, callback| -> i32 {
        BUTTON_PRESSES.with(|btn_prs_ref| {
            let mut press_map = btn_prs_ref.borrow_mut();
            press_map.insert(id, callback);
            0
        })
    })
}

// pub fn register_on_tick(callback: u32) {
//     let callback = TableIndex::new(callback as usize);
//     // Currently this doesn't map the table indexes to specific modules so this is an
//     // implementation for a single module project (which pacman is currently).
//     let vec = &mut *ON_TICK_HANDLERS.lock().unwrap();
//     vec.push(callback);
// }

pub fn register_on_tick(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, callback| {
        ON_TICK_HANDLERS.with(|on_tick_ref| {
            let mut vec = on_tick_ref.borrow_mut();
            vec.push(callback);
        })
    })
}

// // Create canvas to hold the bitmaps.
// pub fn create_canvas(width: i32, height: i32) -> i32 {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::create_canvas(&env, &class, width, height)
// }

pub fn create_canvas(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, width, height| -> i32 {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::create_canvas(&env, &class, width, height)
            })
        })
    })
}

// pub fn create_bitmap(width: i32, height: i32) -> i32 {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::create_bitmap(&env, &class, width, height)
// }

pub fn create_bitmap(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, width, height| -> i32 {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::create_bitmap(&env, &class, width, height)
            })
        })
    })
}

// pub fn modify_bitmap(id: i32, x: i32, y: i32, color: i32) {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::modify_bitmap(&env, &class, id, x, y, color)
// }

pub fn modify_bitmap(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, id, x, y, color| {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::modify_bitmap(&env, &class, id, x, y, color)
            })
        })
    })
}

// pub fn bitmap_set_position(bitmap_id: i32, left: i32, top: i32) {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::bitmap_set_position(&env, &class, bitmap_id, left, top)
// }

pub fn bitmap_set_position(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, id, left, top| {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::bitmap_set_position(&env, &class, id, left, top)
            })
        })
    })
}

// pub fn bitmap_set_z_index(bitmap_id: i32, z_index: i32) {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::bitmap_set_z_index(&env, &class, bitmap_id, z_index)
// }

pub fn bitmap_set_z_index(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, bitmap_id, z_index| {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::bitmap_set_z_index(&env, &class, bitmap_id, z_index)
            })
        })
    })
}

// pub fn canvas_add_bitmap(canvas_id: i32, bitmap_id: i32) {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::canvas_add_bitmap(&env, &class, canvas_id, bitmap_id)
// }

pub fn canvas_add_bitmap(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, canvas_id, bitmap_id| {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::canvas_add_bitmap(&env, &class, canvas_id, bitmap_id)
            })
        })
    })
}

// pub fn canvas_redraw(canvas_id: i32) {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();
//     // Get the class.
//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::canvas_redraw(&env, &class, canvas_id)
// }

pub fn canvas_redraw(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, canvas_id| {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::canvas_redraw(&env, &class, canvas_id)
            })
        })
    })
}

// pub fn canvas_remove_bitmap(canvas_id: i32, bitmap_id: i32) {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();

//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::canvas_remove_bitmap(&env, &class, canvas_id, bitmap_id);
// }

pub fn canvas_remove_bitmap(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, canvas_id, bitmap_id| {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::canvas_remove_bitmap(&env, &class, canvas_id, bitmap_id)
            })
        })
    })
}

// pub fn canvas_delete_bitmap(bitmap_id: i32) {
//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();

//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::canvas_delete_bitmap(&env, &class, bitmap_id);
// }

pub fn canvas_delete_bitmap(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, bitmap_id| {
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::canvas_delete_bitmap(&env, &class, bitmap_id)
            })
        })
    })
}

// pub fn create_text(ctx: &mut Ctx, text: WasmPtr<u8, Array>, color: i32, text_size: i32) -> i32 {
//     let memory = ctx.memory(0);
//     let text = text.get_utf8_string_with_nul(memory).unwrap();

//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();

//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::create_text(&env, &class, text, color, text_size)
// }

pub fn create_text(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, text, color, text_size| -> i32 {
        let text = get_string_from_wasm_memory(text);
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::create_text(&env, &class, &text, color, text_size)
            })
        })
    })
}

// pub fn set_text(ctx: &mut Ctx, text_id: i32, text: WasmPtr<u8, Array>) {
//     let memory = ctx.memory(0);
//     let text = text.get_utf8_string_with_nul(memory).unwrap();

//     let ovm = &*ENV.lock().unwrap();
//     let vm = ovm.as_ref().unwrap();
//     let env = vm.get_env().unwrap();

//     let o_class = &*CLASS.lock().unwrap();
//     let class_ref = o_class.as_ref().unwrap();
//     let class = class_ref.as_obj();

//     java::set_text(&env, &class, text_id, text);
// }

pub fn set_text(store: &Store) -> Func {
    Func::wrap(&store, |_: Caller<'_>, text_id, text| {
        let text = get_string_from_wasm_memory(text);
        //Get the java environment
        ENV.with(|vm_ref| {
            let e_brw = vm_ref.borrow();
            let env = e_brw.as_ref().unwrap().get_env().unwrap();
            //Get the class.
            CLASS.with(|class_ref| {
                let c_brw = class_ref.borrow();
                let class = c_brw.as_ref().unwrap().as_obj();
                java::set_text(&env, &class, text_id, &text)
            })
        })
    })
}
