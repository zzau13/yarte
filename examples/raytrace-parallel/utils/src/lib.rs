use wasm_bindgen::prelude::*;

// TODO: Disable only in release
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => ($crate::error(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_logv {
    ($v:expr) => {
        $crate::logv($v)
    };
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn logv(x: &JsValue);
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}
