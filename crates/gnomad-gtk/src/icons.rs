use std::sync::LazyLock;

use gdk4::Display;
use gnomad_core::config::paths::DataPaths;
use gtk4::prelude::*;
use gtk4::{IconTheme, Window};
use image::GenericImageView;
use ksni::Icon;

/// Freedesktop icon name (matches `com.gnomadstudio.gnomad.desktop`).
pub const ICON_NAME: &str = "com.gnomadstudio.gnomad";

const TRAY_1X: &[u8] = include_bytes!("../icons/tray-mushroom.png");
const TRAY_2X: &[u8] = include_bytes!("../icons/tray-mushroom@2x.png");
const APP_128: &[u8] = include_bytes!("../icons/128x128.png");

static TRAY_PIXMAPS: LazyLock<Vec<Icon>> = LazyLock::new(|| {
    vec![
        png_to_ksni_icon(TRAY_1X).expect("tray-mushroom.png"),
        png_to_ksni_icon(TRAY_2X).expect("tray-mushroom@2x.png"),
    ]
});

pub fn tray_icons() -> Vec<Icon> {
    TRAY_PIXMAPS.clone()
}

/// Install mushroom PNGs into the XDG icon theme search path for GTK / task switcher.
pub fn install_icon_theme(paths: &DataPaths) -> Result<(), String> {
    let base = paths.data_dir().join("icons");
    for (size, bytes) in [(22, TRAY_1X), (44, TRAY_2X), (128, APP_128)] {
        let dir = base.join(format!("hicolor/{size}x{size}/apps"));
        std::fs::create_dir_all(&dir).map_err(|e| format!("create icon dir: {e}"))?;
        std::fs::write(dir.join(format!("{ICON_NAME}.png")), bytes)
            .map_err(|e| format!("write icon: {e}"))?;
    }

    let display = Display::default().ok_or_else(|| "no GDK display".to_string())?;
    IconTheme::for_display(&display).add_search_path(&base);
    Window::set_default_icon_name(ICON_NAME);
    Ok(())
}

fn png_to_ksni_icon(bytes: &[u8]) -> Result<Icon, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("decode png: {e}"))?;
    let (width, height) = img.dimensions();
    let mut data = img.into_rgba8().into_vec();
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1); // RGBA → ARGB (ksni / network byte order)
    }
    Ok(Icon {
        width: width as i32,
        height: height as i32,
        data,
    })
}
