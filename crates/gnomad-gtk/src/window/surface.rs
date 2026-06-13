use gdk4::Surface;
use gtk4::prelude::*;
use gtk4::ApplicationWindow;

/// GDK surface for a realized GTK window.
pub fn window_surface(window: &ApplicationWindow) -> Option<Surface> {
    window.surface()
}
