use log::{Level, Log, Metadata, Record};
use wasm_bindgen::prelude::*;

pub static LOGGER: &'static dyn Log = &NativeLog;

struct NativeLog;

impl Log for NativeLog {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let message = format!("{}", record.args());

        match record.level() {
            Level::Error => console_error(&message),
            Level::Warn => console_warn(&message),
            Level::Info => console_info(&message),
            Level::Debug | Level::Trace => console_debug(&message),
        }
    }

    fn flush(&self) {}
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = debug)]
    fn console_debug(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = info)]
    fn console_info(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn console_warn(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(s: &str);
}
