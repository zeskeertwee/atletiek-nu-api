mod app;
mod windows;
mod async_resource;

use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    pretty_env_logger::init();

    let native_options = NativeOptions::default();
    eframe::run_native(
        "Atletiek-nu-api GUI",
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}
