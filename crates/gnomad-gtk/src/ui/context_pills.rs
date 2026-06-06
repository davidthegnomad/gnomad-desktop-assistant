use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation};

#[cfg(target_os = "linux")]
use gnomad_core::platform::{format_context_block, gather_desktop_context, DesktopContext};

#[derive(Clone)]
pub struct ContextPills {
    pub root: GtkBox,
    window_label: Label,
    clipboard_label: Label,
    #[cfg(target_os = "linux")]
    cached: std::rc::Rc<std::cell::RefCell<DesktopContext>>,
}

impl ContextPills {
    pub fn new() -> Self {
        let root = GtkBox::new(Orientation::Horizontal, 8);
        root.add_css_class("context-pills");
        root.set_hexpand(true);
        root.set_margin_start(16);
        root.set_margin_end(16);
        root.set_margin_top(4);

        let window_pill = pill_container();
        let window_title = Label::new(Some("Window"));
        window_title.add_css_class("context-pill-title");
        window_pill.append(&window_title);
        let window_label = Label::new(Some("…"));
        window_label.add_css_class("context-pill-value");
        window_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        window_label.set_max_width_chars(28);
        window_pill.append(&window_label);
        root.append(&window_pill);

        let clipboard_pill = pill_container();
        let clip_title = Label::new(Some("Clipboard"));
        clip_title.add_css_class("context-pill-title");
        clipboard_pill.append(&clip_title);
        let clipboard_label = Label::new(Some("…"));
        clipboard_label.add_css_class("context-pill-value");
        clipboard_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        clipboard_label.set_max_width_chars(32);
        clipboard_pill.append(&clipboard_label);
        root.append(&clipboard_pill);

        #[cfg(target_os = "linux")]
        let cached = std::rc::Rc::new(std::cell::RefCell::new(DesktopContext {
            app_name: "…".into(),
            window_title: "…".into(),
            clipboard_preview: "…".into(),
        }));

        Self {
            root,
            window_label,
            clipboard_label,
            #[cfg(target_os = "linux")]
            cached,
        }
    }

    pub fn refresh(&self) {
        #[cfg(target_os = "linux")]
        {
            let ctx = gather_desktop_context(80);
            self.window_label
                .set_text(&format!("{} — {}", ctx.app_name, ctx.window_title));
            self.clipboard_label.set_text(&ctx.clipboard_preview);
            *self.cached.borrow_mut() = ctx;
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.window_label.set_text("Unavailable");
            self.clipboard_label.set_text("Unavailable");
        }
    }

    pub fn prompt_context(&self) -> String {
        #[cfg(target_os = "linux")]
        {
            format_context_block(&self.cached.borrow())
        }
        #[cfg(not(target_os = "linux"))]
        {
            String::new()
        }
    }
}

fn pill_container() -> GtkBox {
    let pill = GtkBox::new(Orientation::Horizontal, 6);
    pill.add_css_class("context-pill");
    pill
}
