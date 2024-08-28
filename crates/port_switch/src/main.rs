use port_switch::DynamicProxy;

fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 700.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    let (proxy_handle, handle) = DynamicProxy::initiate()
        .map_err(|err| eframe::Error::AppCreation(Box::new(err)))?;

    eframe::run_native(
        "Port switch",
        native_options,
        Box::new(|cc| Ok(Box::new(port_switch::App::new(cc, proxy_handle)))),
    )?;
    handle.join().unwrap();
    Ok(())
}
