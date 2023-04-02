use egui::{RichText, Ui};
use matex_common::{
    function::{Function, Parameter},
    util::SymbolTable,
};
use matex_compiler::cas::backend::{
    format::{NormalFormatter, ValueFormatter},
    value::RuntimeVal,
};

pub struct Inspector {}

impl Inspector {
    pub fn ui(ui: &mut Ui, functions: &SymbolTable<Function>, variables: &SymbolTable<RuntimeVal>) {
        ui.collapsing(RichText::new("Functions").heading(), |ui| {
            egui::Grid::new("func_grid")
                .num_columns(1)
                .spacing(egui::vec2(0.0, 4.0))
                .striped(true)
                .show(ui, |ui| {
                    for (_, value) in functions {
                        let Function { name, params, body } = value;
                        let params_string = params
                            .iter()
                            .map(|param| param.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let function_string = format!("{}({})", name, params_string);
                        ui.collapsing(function_string, |ui| {
                            for Parameter { name, type_name } in params {
                                ui.label(format!("{name}: {type_name}"));
                            }
                            ui.label(format!("body: {:?}", body));
                        });
                        ui.end_row();
                    }
                });
        });
        ui.separator();
        ui.collapsing(RichText::new("Variables").heading(), |ui| {
            egui::Grid::new("var_grid")
                .num_columns(2)
                .spacing(egui::vec2(10.0, 4.0))
                .striped(true)
                .show(ui, |ui| {
                    for (name, value) in variables {
                        ui.label(name);
                        ui.label(NormalFormatter::format(value));
                        ui.end_row();
                    }
                });
        });
    }
}
