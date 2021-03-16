pub mod global;
pub mod host;
pub mod leb128;

#[allow(non_snake_case)]
#[cfg(feature = "android_build")]
pub mod android {

    mod gui;
    mod java;

    use jni::{
        objects::{JClass, JObject, JString},
        JNIEnv,
    };

    use host::OurResult;
    use java::{CLASS, ENV};

    use super::*;

    fn initialize(javaENV: &JNIEnv, callback: &JObject, cache_path: String) -> OurResult<()> {
        info!("getting metadata bytes");
        let metadata_bytes: Vec<u8> =
            java::get_metadata_bytes(&javaENV, &callback, "android_metadata.json")?;
        let metadata: String = String::from_utf8(metadata_bytes)?;

        info!("initializing");
        let mut linker = host::create_linker("android")?;
        let store = linker.store().clone();

        linker.define("host", "CreateTextView", gui::create_text_view(&store))?;
        linker.define("host", "ModifyTextView", gui::modify_text_view(&store))?;
        linker.define("host", "RemoveTextView", gui::remove_text_view(&store))?;
        linker.define("host", "CreateButton", gui::create_button(&store))?;
        linker.define("host", "RegisterOnClick", gui::register_on_click(&store))?;
        linker.define("host", "RegisterOnTick", gui::register_on_tick(&store))?;
        linker.define("host", "CreateCanvas", gui::create_canvas(&store))?;
        linker.define("host", "CreateBitmap", gui::create_bitmap(&store))?;
        linker.define("host", "ModifyBitmap", gui::modify_bitmap(&store))?;
        linker.define(
            "host",
            "BitmapSetPosition",
            gui::bitmap_set_position(&store),
        )?;
        linker.define("host", "BitmapSetZIndex", gui::bitmap_set_z_index(&store))?;
        linker.define("host", "CanvasAddBitmap", gui::canvas_add_bitmap(&store))?;
        linker.define("host", "CanvasRedraw", gui::canvas_redraw(&store))?;
        linker.define(
            "host",
            "CanvasRemoveBitmap",
            gui::canvas_remove_bitmap(&store),
        )?;
        linker.define(
            "host",
            "CanvasDeleteBitmap",
            gui::canvas_delete_bitmap(&store),
        )?;
        linker.define("host", "CreateText", gui::create_text(&store))?;
        linker.define("host", "SetText", gui::set_text(&store))?;

        host::initialize(
            &metadata,
            "android",
            &cache_path,
            vec!["aarch64".to_string(), "android".to_string()],
            linker,
            store,
        )?;

        let class = javaENV.new_global_ref(*callback).unwrap();
        CLASS.with(|rc| *rc.borrow_mut() = Some(class));
        let vm = javaENV.get_java_vm().unwrap();
        ENV.with(|rc| {
            *rc.borrow_mut() = Some(vm);
        });
        Ok(())
    }

    #[no_mangle]
    pub unsafe extern "C" fn Java_com_hy_wasmandroid_Wasm_JNIInitializeRuntime(
        javaENV: JNIEnv,
        _: JClass,
        callback: JObject,
        cacheDir: JString,
    ) {
        host::init_logging();

        std::panic::set_hook(Box::new(|panic_info| {
            error!("ERR: {}", panic_info.to_string());
        }));
        let cache_path: String = javaENV
            .get_string(cacheDir)
            .expect("JNI error: Couldn't convert the data path string.")
            .into();

        if let Err(e) = initialize(&javaENV, &callback, cache_path) {
            javaENV.throw(e.to_string()).unwrap();
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn Java_com_hy_wasmandroid_Wasm_JNIRunMainWASM(
        _javaENV: JNIEnv,
        _: JClass,
    ) {
        host::run_main();
    }

    #[no_mangle]
    pub unsafe extern "C" fn Java_com_hy_wasmandroid_GuiContext_JNIButtonPress(
        _javaEnv: JNIEnv,
        _: JClass,
        id: i32,
    ) -> i32 {
        use global::wasm_table;
        use gui::BUTTON_PRESSES;
        BUTTON_PRESSES.with(|rc| {
            let press_map = rc.borrow();
            match press_map.get(&id) {
                Some(callback) => {
                    let table = wasm_table().unwrap();
                    let callback_f = table.get(*callback).unwrap();
                    let f = callback_f.funcref().unwrap().unwrap();
                    f.call(&[]).unwrap();
                    0
                }
                None => -1,
            }
        })
    }

    #[no_mangle]
    pub unsafe extern "C" fn Java_com_hy_wasmandroid_Wasm_JNIOnTick(_javaENV: JNIEnv, _: JClass) {
        use global::wasm_table;
        use gui::ON_TICK_HANDLERS;
        ON_TICK_HANDLERS.with(|rc| {
            let vec = rc.borrow();
            let table = wasm_table().unwrap();
            for callback in vec.iter() {
                let callback_f = table.get(*callback).unwrap();
                let f = callback_f.funcref().unwrap().unwrap();
                f.call(&[]).unwrap();
            }
        })
    }
}
