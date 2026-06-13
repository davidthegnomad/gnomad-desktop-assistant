use gdk4::prelude::*;
use gtk4::prelude::*;
use gtk4::{gdk, ApplicationWindow};

use super::above::{apply_floating_above, clear_above};
use super::dims::{
    DisplayMode, MAX_HEIGHT, MAX_WIDTH, MIN_HEIGHT, MIN_WIDTH, PANEL_HEIGHT, PANEL_WIDTH,
};
use super::panel_anchor::{apply_panel_position, TrayAnchor};

/// Applies native GTK window policies per display mode (no WebKit shell).
pub struct WindowManager {
    window: ApplicationWindow,
    mode: DisplayMode,
    tray_anchor: Option<TrayAnchor>,
}

impl WindowManager {
    pub fn new(window: ApplicationWindow) -> Self {
        let mgr = Self {
            window,
            mode: DisplayMode::Panel,
            tray_anchor: None,
        };
        mgr.attach_resize_limits();
        mgr
    }

    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }

    pub fn mode(&self) -> DisplayMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: DisplayMode) {
        self.mode = mode;
        self.apply_mode();
    }

    pub fn set_tray_anchor(&mut self, anchor: Option<TrayAnchor>) {
        self.tray_anchor = anchor;
    }

    pub fn toggle_visibility(&mut self) {
        if self.window.is_visible() {
            self.hide();
        } else {
            self.show();
        }
    }

    pub fn show(&mut self) {
        self.apply_mode();
        self.raise_and_present();
    }

    pub fn show_in_mode(&mut self, mode: DisplayMode) {
        self.mode = mode;
        self.apply_mode();
        self.raise_and_present();
    }

    pub fn raise_and_present(&mut self) {
        self.window.present();
        self.window.set_focus_visible(true);
        if self.mode == DisplayMode::Floating {
            apply_floating_above(&self.window);
        }
    }

    pub fn toggle_windowed(&mut self) {
        if self.window.is_visible() && self.mode == DisplayMode::Windowed {
            self.hide();
        } else {
            self.show_in_mode(DisplayMode::Windowed);
        }
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn is_visible(&self) -> bool {
        self.window.is_visible()
    }

    pub fn bind_hide_on_close(&self) {
        self.window.connect_close_request(|win| {
            win.set_visible(false);
            gtk4::glib::Propagation::Stop
        });
    }

    fn attach_resize_limits(&self) {
        let window = self.window.clone();
        window.connect_notify_local(Some("width"), move |w, _| {
            clamp_size(w);
        });
        window.connect_notify_local(Some("height"), move |w, _| {
            clamp_size(w);
        });
    }

    fn apply_mode(&mut self) {
        let window = &self.window;

        if window.is_maximized() {
            window.unmaximize();
        }
        if window.is_fullscreen() {
            window.unfullscreen();
        }

        match self.mode {
            DisplayMode::Panel => self.apply_panel(),
            DisplayMode::Floating => self.apply_floating(),
            DisplayMode::Windowed => self.apply_windowed(),
            DisplayMode::Fullscreen => self.apply_fullscreen(),
        }
    }

    fn apply_panel(&mut self) {
        clear_above(&self.window);
        let window = &self.window;
        window.set_title(Some("Gnomad"));
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_size_request(PANEL_WIDTH, PANEL_HEIGHT);
        window.set_default_size(PANEL_WIDTH, PANEL_HEIGHT);
        self.apply_toplevel_layout(false);
        apply_panel_position(window, self.tray_anchor);
    }

    fn apply_floating(&mut self) {
        let window = &self.window;
        let (w, h) = DisplayMode::Floating.default_size();
        window.set_title(Some("Gnomad"));
        window.set_decorated(true);
        window.set_resizable(true);
        window.set_size_request(MIN_WIDTH, MIN_HEIGHT);
        window.set_default_size(w, h);
        self.apply_toplevel_layout(true);
    }

    fn apply_windowed(&mut self) {
        clear_above(&self.window);
        let window = &self.window;
        let (w, h) = DisplayMode::Windowed.default_size();
        window.set_title(Some("Gnomad"));
        window.set_decorated(true);
        window.set_resizable(true);
        window.set_size_request(MIN_WIDTH, MIN_HEIGHT);
        window.set_default_size(w, h);
        self.apply_toplevel_layout(true);
    }

    fn apply_fullscreen(&mut self) {
        let window = &self.window;
        window.set_title(Some("Gnomad"));
        window.set_decorated(true);
        window.set_resizable(true);
        window.set_size_request(MIN_WIDTH, MIN_HEIGHT);
        window.fullscreen();
    }

    fn apply_toplevel_layout(&self, resizable: bool) {
        if let Some(toplevel) = self.toplevel() {
            let layout = gdk::ToplevelLayout::new();
            layout.set_resizable(resizable);
            toplevel.present(&layout);
        }
    }

    fn toplevel(&self) -> Option<gdk::Toplevel> {
        super::surface::window_surface(&self.window)
            .and_then(|s| s.downcast::<gdk::Toplevel>().ok())
    }
}

fn clamp_size(window: &ApplicationWindow) {
    if !window.is_visible() || window.is_fullscreen() {
        return;
    }
    let w = window.width();
    let h = window.height();
    if w > MAX_WIDTH || h > MAX_HEIGHT {
        window.set_default_size(w.min(MAX_WIDTH), h.min(MAX_HEIGHT));
        window.set_size_request(MIN_WIDTH, MIN_HEIGHT);
    }
}
