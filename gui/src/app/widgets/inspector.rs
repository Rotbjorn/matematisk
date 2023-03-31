use egui::Ui;
use matex_common::{util::SymbolTable, function::Function};
use matex_compiler::cas::backend::{format::{NormalFormatter, ValueFormatter}, value::RuntimeVal};

pub struct Inspector {

}

impl Inspector {
    pub fn ui(ui: &mut Ui, functions: &SymbolTable<Function>, variables: &SymbolTable<RuntimeVal>) {
        ui.collapsing("Functions", |ui| {
            for (key, value) in functions {
                ui.label(format!("{}: {:?}", key, value));
            }
        });
        ui.separator();
        ui.collapsing("Variables", |ui| {
            for (key, value) in variables {
                ui.label(format!("{}: {}", key, NormalFormatter::format(value)));
            }
        });
    }
}