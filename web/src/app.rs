use eframe::App;
use egui::{TextEdit, Ui};
use matex_compiler::cas::{
    backend::{
        format::{NormalFormatter, ValueFormatter},
        runtime::Runtime,
    },
    frontend::{lexer::Lexer, parser::Parser},
};

pub struct MatexApp {
    source: String,

    executed: Vec<(String, String)>,
    runtime: Runtime,
}

impl MatexApp {
    fn render_executions(&self, ui: &mut Ui) {
        for (source, output) in &self.executed {
            ui.label("i> ".to_owned() + source);
            ui.label("o>".to_owned() + output);
            ui.separator();
        }
    }
}

impl Default for MatexApp {
    fn default() -> Self {
        Self {
            source: "".to_owned(),
            executed: Vec::new(),
            runtime: Runtime::new(),
        }
    }
}

impl App for MatexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("REPL").show(ctx, |ui| {
            self.render_executions(ui);
            /*
            let mut visuals = Visuals::default();
            visuals.panel_fill = Color32::DARK_RED;
            ctx.set_visuals(visuals);
            */
            TextEdit::multiline(&mut self.source).show(ui);

            if ui.button("Run").clicked() {
                if let Ok(program) = Parser::new(Lexer::new(&self.source).collect()).parse() {
                    let last_value = self.runtime.run(&program);
                    self.executed.push((
                        self.source.clone(),
                        format!("{}", NormalFormatter::format(&last_value)),
                    ));
                    self.source.clear();
                }
            }

            /*
            let sin: PlotPoints = (0..1000).map(|i| {
                let x = i as f64 * 0.1;
                [x, x.sin()]
            }).collect();
            let line = Line::new(sin);
            let line = line.stroke((32.0, Color32::GOLD));


            Plot::new("test-plot").show_background(false).show(ui, |plot_ui| {
                plot_ui.line(line)
            });
            */
        });

        egui::Window::new("Environment").show(ctx, |ui| {
            ui.label("Functions:");
            for (key, value) in &self.runtime.environment.get_scope().borrow().functions {
                ui.label(format!("{}: {:?}", key, value));
            }
            ui.separator();
            ui.label("Variables:");
            for (key, value) in &self.runtime.environment.get_scope().borrow().variables {
                ui.label(format!("{}: {}", key, NormalFormatter::format(value)));
            }
        });
    }
}
