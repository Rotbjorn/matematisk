#[cfg(not(target_arch = "wasm32"))]
mod app;

#[cfg(not(target_arch = "wasm32"))]
use app::gui_app::MatexApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Matex",
        native_options,
        Box::new(|_cc| Box::<MatexApp>::default()),
    )?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
