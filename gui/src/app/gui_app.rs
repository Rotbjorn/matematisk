use eframe::App;
use egui::{panel::Side, FontId, Label, RichText, TextEdit, Ui};
use matex_compiler::cas::{
    eval::{
        format::{NormalFormatter, ValueFormatter},
        runtime::Runtime,
    },
    syntax::{lexer::Lexer, parser::Parser},
};

use super::widgets::inspector::Inspector;

struct State {
    pane: ActivePane,
}

pub struct MatexApp {
    state: State,
    source: String,

    executed: Vec<(String, String)>,
    runtime: Runtime,
}

impl MatexApp {
    fn render_executions(&self, ui: &mut Ui) {
        for (source, output) in &self.executed {
            ui.add(Label::new(
                RichText::new("i>".to_owned() + source).font(FontId::monospace(12.0)),
            ));
            ui.add(Label::new(
                RichText::new("o>".to_owned() + output).font(FontId::monospace(12.0)),
            ));
            ui.separator();
        }
    }
}

impl Default for MatexApp {
    fn default() -> Self {
        Self {
            state: State {
                pane: ActivePane::Repl,
            },
            source: "".to_owned(),
            executed: Vec::new(),
            runtime: Runtime::new(),
        }
    }
}

#[derive(PartialEq)]
enum ActivePane {
    Repl,
    Example,
}

impl App for MatexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(Side::Right, "inspector").show(ctx, |ui| {
            let scope = self.runtime.environment.get_scope();
            Inspector::ui(ui, &scope.functions, &scope.variables);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.state.pane, ActivePane::Repl,  RichText::new("REPL").heading());
                ui.selectable_value(&mut self.state.pane, ActivePane::Example,  RichText::new("Example").heading());
            });

            match self.state.pane {
                ActivePane::Repl => {
                    self.render_executions(ui);

                    TextEdit::multiline(&mut self.source)
                        .code_editor()
                        .hint_text("Write some code")
                        .show(ui);

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
                },
                ActivePane::Example => {
                    TextEdit::multiline(&mut EXAMPLE_CODE.to_owned())
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .show(ui);


                    ui.separator();
                    ui.label(
                        r"Finns just nu inte möjlighet att göra kommentarer
Man får inte heller göra ett funktions anrop först (på en linje), för den tror att det är en funktionsdefinition
e.g. add(1, 2). Är enkelt att fixa men har inte gjort det än, är annars bara att spara funktionsvärdet i en variabel
så fungerar det: y = add(1, 2).
Som visat ovanför så går det också att skriva in symboler i funktioner och få tillbaka ett uttryck.
När jag lägger till funktioner som cos och sin, så måste sådana funktioner kunna uttryckas i uttryck, 
te.x. cos(x) måste förvaras till värdet på x är definierat.
Just nu gör typerna efter variablerna inget, men de är obligatoriska att ha med!

Tänker att nästa steg är bättre förenkling + ekvationslösning samt reaktiva variabler");
                },
            }
        });
    }
}

const EXAMPLE_CODE: &str = r"add(x: r, y: hund) = x + y
a = add(2, 3)
b = add(c, d)

y = l*f - l*2*f
simplify y
i = simplify y
i

h = simplify add(12*x, 13*x)
";
