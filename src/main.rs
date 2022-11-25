use std::io::Cursor;

use bevy::{window::WindowId, winit::WinitWindows};
use bevy_blackfriday::prelude::*;
use winit::window::Icon;

fn main() {
    let mut app = App::new();
    bevy_blackfriday::setup_app(&mut app);
    app.add_startup_system(set_window_icon);
    app.run();
}

fn set_window_icon(windows: NonSend<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();
    let icon_buf = Cursor::new(include_bytes!("../assets/textures/bevy.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
