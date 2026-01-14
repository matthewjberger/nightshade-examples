use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_host_protocol::{BackendEvent, FrontendCommand};

pub fn send_command(cmd: FrontendCommand) {
    if let Some(data) = cmd.to_base64() {
        let window = web_sys::window().unwrap();
        if let Ok(ipc) = js_sys::Reflect::get(&window, &JsValue::from_str("ipc")) {
            if let Some(f) = js_sys::Reflect::get(&ipc, &JsValue::from_str("postMessage"))
                .ok()
                .and_then(|f| f.dyn_ref::<js_sys::Function>().cloned())
            {
                let _ = f.call1(&ipc, &JsValue::from_str(&data));
            }
        }
    }
}

pub fn set_backend_handler(handler: impl Fn(BackendEvent) + 'static) {
    let window = web_sys::window().unwrap();
    let connected = Rc::new(Cell::new(false));
    let connected_for_handler = connected.clone();

    let closure = Closure::wrap(Box::new(move |data: String| {
        if let Some(event) = BackendEvent::from_base64(&data) {
            if matches!(event, BackendEvent::Connected) {
                connected_for_handler.set(true);
            }
            handler(event);
        }
    }) as Box<dyn Fn(String)>);

    let _ = js_sys::Reflect::set(&window, &JsValue::from_str("__h"), closure.as_ref());
    closure.forget();

    let poll = Closure::wrap(Box::new(move || {
        if !connected.get() {
            send_command(FrontendCommand::Ready);
        }
    }) as Box<dyn Fn()>);

    let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
        poll.as_ref().unchecked_ref(),
        50,
    );
    poll.forget();
}
