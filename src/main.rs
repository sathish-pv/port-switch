use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use port_switch::{start_backend, BackEndConfig};

fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 700.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    let (tx, rx): (Sender<BackEndConfig>, Receiver<BackEndConfig>) = channel();

    let _handle = thread::Builder::new()
        .name("port_switch_back_end".to_string())
        .spawn(|| start_backend(rx));

    eframe::run_native(
        "Port switch",
        native_options,
        Box::new(|cc| Ok(Box::new(port_switch::App::new(cc, tx)))),
    )
}
