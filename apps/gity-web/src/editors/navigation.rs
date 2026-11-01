use super::{call, set};
use js_sys::{Array, Object};
use wasm_bindgen::JsValue;
pub(super) fn jump_to(editor: &JsValue, line: u32, column: u32, length: u32) {
    let position = Object::new();
    set(
        &position,
        "lineNumber",
        &JsValue::from_f64(line.max(1) as f64),
    );
    set(
        &position,
        "column",
        &JsValue::from_f64(column.max(1) as f64),
    );
    let _ = call(editor, "setPosition", &[position.into()]);
    let _ = call(
        editor,
        "revealLineInCenter",
        &[JsValue::from_f64(line.max(1) as f64)],
    );
    if length == 0 {
        return;
    }
    let range = Object::new();
    for (key, value) in [
        ("startLineNumber", line),
        ("endLineNumber", line),
        ("startColumn", column),
        ("endColumn", column + length),
    ] {
        set(&range, key, &JsValue::from_f64(value.max(1) as f64));
    }
    let options = Object::new();
    set(
        &options,
        "className",
        &JsValue::from_str("search-jump-match-decoration"),
    );
    let mark = Object::new();
    set(&mark, "range", &range);
    set(&mark, "options", &options);
    let marks = Array::new();
    marks.push(&mark);
    let _ = call(
        editor,
        "deltaDecorations",
        &[Array::new().into(), marks.into()],
    );
}
