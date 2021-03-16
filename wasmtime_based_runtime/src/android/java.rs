use std::cell::RefCell;

use jni::{
    objects::{JObject, JValue},
    JNIEnv,
};

use jni::{objects::GlobalRef, JavaVM};

use crate::host::OurResult;

thread_local!(
    pub static ENV: RefCell<Option<JavaVM>> = RefCell::new(None);
    pub static CLASS: RefCell<Option<GlobalRef>> = RefCell::new(None);
);

pub fn dealloc_jvalue_obj(env: &JNIEnv, obj: JValue) {
    if let JValue::Object(obj) = obj {
        env.delete_local_ref(obj).unwrap();
    }
}

pub fn get_metadata_bytes(env: &JNIEnv, class: &JObject, module_name: &str) -> OurResult<Vec<u8>> {
    let module_name = env.new_string(module_name)?;
    let arg = JValue::Object(module_name.into());
    let byte_array =
        match env.call_method(*class, "loadMetadata", "(Ljava/lang/String;)[B", &[arg])? {
            JValue::Object(byte_array) => byte_array,
            _ => panic!("Unexpected return value"),
        };
    dealloc_jvalue_obj(env, arg);
    Ok(env.convert_byte_array(*byte_array)?)
}

pub fn create_text_view(env: &JNIEnv, class: &JObject, text: &str) -> i32 {
    let text = env
        .new_string(text)
        .expect("Couldn't create a java string!");
    let arg = JValue::Object(text.into());
    match env
        .call_method(*class, "createTextView", "(Ljava/lang/String;)I", &[arg])
        .unwrap()
    {
        JValue::Int(i) => {
            dealloc_jvalue_obj(env, arg);
            i
        }
        _ => panic!("Unexpected return value"),
    }
}

pub fn remove_text_view(env: &JNIEnv, class: &JObject, id: i32) {
    let id = JValue::Int(id);
    match env
        .call_method(*class, "removeTextView", "(I)V", &[id])
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}

pub fn modify_text_view(env: &JNIEnv, class: &JObject, id: i32, text: &str) {
    let text = env
        .new_string(text)
        .expect("Couldn't create a java string!");
    let text = JValue::Object(text.into());
    let id = JValue::Int(id);
    match env
        .call_method(
            *class,
            "modifyTextView",
            "(ILjava/lang/String;)V",
            &[id, text],
        )
        .unwrap()
    {
        JValue::Void => dealloc_jvalue_obj(env, text),
        _ => panic!("Unexpected return value"),
    }
}

pub fn create_button(env: &JNIEnv, class: &JObject, label: &str) -> i32 {
    let label = env
        .new_string(label)
        .expect("Couldn't create a java string!");
    let label = JValue::Object(label.into());
    match env
        .call_method(*class, "createButton", "(Ljava/lang/String;)I", &[label])
        .unwrap()
    {
        JValue::Int(i) => {
            dealloc_jvalue_obj(env, label);
            i
        }
        _ => panic!("Unexpected return value"),
    }
}

pub fn create_canvas(env: &JNIEnv, class: &JObject, width: i32, height: i32) -> i32 {
    let width = JValue::Int(width);
    let height = JValue::Int(height);
    match env
        .call_method(*class, "createCanvas", "(II)I", &[width, height])
        .unwrap()
    {
        JValue::Int(i) => i,
        _ => panic!("Unexpected return value"),
    }
}

pub fn create_bitmap(env: &JNIEnv, class: &JObject, width: i32, height: i32) -> i32 {
    let width = JValue::Int(width);
    let height = JValue::Int(height);
    match env
        .call_method(*class, "createBitmap", "(II)I", &[width, height])
        .unwrap()
    {
        JValue::Int(i) => i,
        _ => panic!("Unexpected return value"),
    }
}

pub fn modify_bitmap(env: &JNIEnv, class: &JObject, id: i32, x: i32, y: i32, color: i32) {
    let id = JValue::Int(id);
    let x = JValue::Int(x);
    let y = JValue::Int(y);
    let color = JValue::Int(color);
    match env
        .call_method(*class, "modifyBitmap", "(IIII)V", &[id, x, y, color])
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}

pub fn bitmap_set_position(env: &JNIEnv, class: &JObject, bitmap_id: i32, left: i32, top: i32) {
    let bitmap_id = JValue::Int(bitmap_id);
    let left = JValue::Int(left);
    let top = JValue::Int(top);
    match env
        .call_method(
            *class,
            "bitmapSetPosition",
            "(III)V",
            &[bitmap_id, left, top],
        )
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}

pub fn bitmap_set_z_index(env: &JNIEnv, class: &JObject, bitmap_id: i32, z_index: i32) {
    let bitmap_id = JValue::Int(bitmap_id);
    let z_index = JValue::Int(z_index);
    match env
        .call_method(*class, "bitmapSetZIndex", "(II)V", &[bitmap_id, z_index])
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}

pub fn canvas_add_bitmap(env: &JNIEnv, class: &JObject, canvas_id: i32, bitmap_id: i32) {
    let canvas_id = JValue::Int(canvas_id);
    let bitmap_id = JValue::Int(bitmap_id);
    match env
        .call_method(*class, "canvasAddBitmap", "(II)V", &[canvas_id, bitmap_id])
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}

pub fn canvas_redraw(env: &JNIEnv, class: &JObject, canvas_id: i32) {
    let canvas_id = JValue::Int(canvas_id);
    match env
        .call_method(*class, "canvasRedraw", "(I)V", &[canvas_id])
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}

pub fn canvas_remove_bitmap(env: &JNIEnv, class: &JObject, canvas_id: i32, bitmap_id: i32) {
    let canvas_id = JValue::Int(canvas_id);
    let bitmap_id = JValue::Int(bitmap_id);
    match env
        .call_method(
            *class,
            "canvasRemoveBitmap",
            "(II)V",
            &[canvas_id, bitmap_id],
        )
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}

pub fn canvas_delete_bitmap(env: &JNIEnv, class: &JObject, bitmap_id: i32) {
    let bitmap_id = JValue::Int(bitmap_id);
    match env
        .call_method(*class, "canvasDeleteBitmap", "(I)V", &[bitmap_id])
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}

pub fn create_text(env: &JNIEnv, class: &JObject, text: &str, color: i32, text_size: i32) -> i32 {
    let text = env
        .new_string(text)
        .expect("Couldn't create a java string!");
    let text = JValue::Object(text.into());
    let color = JValue::Int(color);
    let text_size = JValue::Int(text_size);
    match env
        .call_method(
            *class,
            "createText",
            "(Ljava/lang/String;II)I",
            &[text, color, text_size],
        )
        .unwrap()
    {
        JValue::Int(i) => i,
        _ => panic!("Unexpected return value"),
    }
}

pub fn set_text(env: &JNIEnv, class: &JObject, text_id: i32, text: &str) {
    let text_id = JValue::Int(text_id);
    let text = env
        .new_string(text)
        .expect("Couldn't create a java string!");
    let text = JValue::Object(text.into());
    match env
        .call_method(
            *class,
            "setText",
            "(ILjava/lang/String;)V",
            &[text_id, text],
        )
        .unwrap()
    {
        JValue::Void => (),
        _ => panic!("Unexpected return value"),
    }
}
