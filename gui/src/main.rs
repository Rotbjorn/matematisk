#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use matex_gui::MatexApp;


#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(960.0, 540.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Matex GUI",
        options,
        Box::new(|_cc| Box::new(MatexApp::default())),
    )?;
    Ok(())
}


#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "canvas-id", 
            web_options, 
            Box::new(|_cc| Box::new(MatexApp::default()))
        ).await.expect("Failed to initialise eframe");
    });
}