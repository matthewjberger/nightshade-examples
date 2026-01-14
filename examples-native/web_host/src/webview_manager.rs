use nightshade::prelude::{egui, window};
use std::collections::HashMap;
use std::sync::Arc;
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::{Rect, WebView, WebViewBuilder};

pub struct WebviewManager {
    webviews: HashMap<String, WebView>,
    webview_bounds: HashMap<String, (f64, f64, f64, f64)>,
    window: Option<Arc<window::Window>>,
}

impl Default for WebviewManager {
    fn default() -> Self {
        Self {
            webviews: HashMap::new(),
            webview_bounds: HashMap::new(),
            window: None,
        }
    }
}

impl WebviewManager {
    pub fn set_window(&mut self, window: Arc<window::Window>) {
        self.window = Some(window);
    }

    pub fn create_webview(&mut self, id: String, url: &str, rect: egui::Rect, _scale_factor: f32) {
        if self.webviews.contains_key(&id) {
            return;
        }

        let Some(window) = &self.window else {
            return;
        };

        let bounds = Rect {
            position: LogicalPosition::new(rect.min.x as f64, rect.min.y as f64).into(),
            size: LogicalSize::new(rect.width() as f64, rect.height() as f64).into(),
        };

        let webview_result = WebViewBuilder::new()
            .with_url(url)
            .with_bounds(bounds)
            .with_navigation_handler(|_| true)
            .build_as_child(window.as_ref());

        if let Ok(webview) = webview_result {
            let _ = webview.set_visible(true);
            self.webview_bounds.insert(
                id.clone(),
                (
                    rect.min.x as f64,
                    rect.min.y as f64,
                    rect.width() as f64,
                    rect.height() as f64,
                ),
            );
            self.webviews.insert(id, webview);
        }
    }

    pub fn update_position(&mut self, id: &str, rect: egui::Rect, _scale_factor: f32) {
        let Some(webview) = self.webviews.get(id) else {
            return;
        };

        let new_bounds = (
            rect.min.x as f64,
            rect.min.y as f64,
            rect.width() as f64,
            rect.height() as f64,
        );

        if let Some(old_bounds) = self.webview_bounds.get(id)
            && *old_bounds == new_bounds
        {
            return;
        }

        let bounds = Rect {
            position: LogicalPosition::new(new_bounds.0, new_bounds.1).into(),
            size: LogicalSize::new(new_bounds.2, new_bounds.3).into(),
        };

        let _ = webview.set_bounds(bounds);
        self.webview_bounds.insert(id.to_string(), new_bounds);
    }

    pub fn ensure_all_visible(&self) {
        for webview in self.webviews.values() {
            let _ = webview.set_visible(true);
        }
    }

    pub fn has_webview(&self, id: &str) -> bool {
        self.webviews.contains_key(id)
    }

    pub fn retain_only(&mut self, active_ids: &std::collections::HashSet<String>) {
        self.webviews.retain(|id, _| active_ids.contains(id));
        self.webview_bounds.retain(|id, _| active_ids.contains(id));
    }
}
