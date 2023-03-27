use eframe::App;
use egui::{Ui, plot::{Plot, Line, PlotPoints}, Frame, Color32, Visuals, Stroke, TextEdit};
use matex_compiler::cas::{
    backend::runtime::Runtime,
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
            ui.label(source);
            ui.label(output);
            ui.separator();
        }
    }
}

impl Default for MatexApp {
    fn default() -> Self {
        Self {
            source: "".to_owned(),
            executed: Vec::new(),
            runtime: Runtime::default(),
        }
    }
}

impl App for MatexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::SidePanel::right("my-right-panel").default_width(300.0).show(ctx, |ui| {
            self.render_executions(ui);
            /*
            let mut visuals = Visuals::default();
            visuals.panel_fill = Color32::DARK_RED;
            ctx.set_visuals(visuals);
            */
            ui.text_edit_multiline(&mut self.source);

            let input = TextEdit::multiline(&mut self.source).

            if ui.button("Run").clicked() {
                if let Ok(program) = Parser::new(Lexer::new(&self.source).collect()).parse() {
                    let last_value = self.runtime.run(program);
                    self.executed
                        .push((self.source.clone(), format!("{:?}", last_value)));
                    self.source.clear();
                }
            }

            let sin: PlotPoints = (0..1000).map(|i| {
                let x = i as f64 * 0.1;
                [x, x.sin()]
            }).collect();
            let line = Line::new(sin);
            let line = line.stroke((32.0, Color32::GOLD));


            Plot::new("test-plot").show_background(false).show(ui, |plot_ui| {
                plot_ui.line(line)
            });

            /*
             ui.heading("My egui Application");
             ui.horizontal(|ui| {
                 let name_label = ui.label("Your name: ");
                 ui.text_edit_singleline(&mut self.name)
                     .labelled_by(name_label.id);
             });
             ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
             if ui.button("Click each year").clicked() {
                 self.age += 1;
             }
             ui.label(format!("Hello '{}', age {}", self.name, self.age));
            */
        });
    }
}
