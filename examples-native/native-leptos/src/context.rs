use nightshade::prelude::{egui, window};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use web_host_protocol::{BackendEvent, FrontendCommand};
use wry::{
    Rect, WebView, WebViewBuilder,
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
};

const INIT_SCRIPT: &str = "window.onBackendMessage=function(d){window.__h&&window.__h(d)};";

fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(w, h).into(),
    }
}

pub struct WebviewContext {
    webview: Option<WebView>,
    bounds: (f64, f64, f64, f64),
    tx: Sender<FrontendCommand>,
    rx: Receiver<FrontendCommand>,
}

impl Default for WebviewContext {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            webview: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
            tx,
            rx,
        }
    }
}

impl WebviewContext {
    pub fn ensure_webview(&mut self, window: Arc<window::Window>, port: u16, r: egui::Rect) {
        let b = (
            r.min.x as f64,
            r.min.y as f64,
            r.width() as f64,
            r.height() as f64,
        );

        if let Some(wv) = &self.webview {
            if self.bounds != b {
                let _ = wv.set_bounds(rect(b.0, b.1, b.2, b.3));
                self.bounds = b;
            }
            return;
        }

        let tx = self.tx.clone();
        if let Ok(wv) = WebViewBuilder::new()
            .with_url(format!("http://127.0.0.1:{port}"))
            .with_bounds(rect(b.0, b.1, b.2, b.3))
            .with_navigation_handler(|_| true)
            .with_initialization_script(INIT_SCRIPT)
            .with_ipc_handler(move |r| {
                if let Some(c) = FrontendCommand::from_base64(r.body()) {
                    let _ = tx.send(c);
                }
            })
            .build_as_child(window.as_ref())
        {
            let _ = wv.set_visible(true);
            let _ = wv.focus();
            self.bounds = b;
            self.webview = Some(wv);
            let size = window.inner_size();
            let _ = window.request_inner_size(PhysicalSize::new(size.width + 1, size.height));
            let _ = window.request_inner_size(size);
        }
    }

    pub fn send(&self, event: BackendEvent) {
        if let (Some(wv), Some(d)) = (&self.webview, event.to_base64()) {
            let _ = wv.evaluate_script(&format!("window.onBackendMessage('{d}')"));
        }
    }

    pub fn drain_messages(&self) -> impl Iterator<Item = FrontendCommand> + '_ {
        self.rx.try_iter()
    }
}
