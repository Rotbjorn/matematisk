use super::{runtime::Runtime, value::{RunType, RunVal}};

impl Runtime {
    pub fn add_standard_environment(&mut self) {
        const PI: f64 = std::f64::consts::PI;
        self.environment.constants.insert("PI".to_string(), RunVal::new(RunType::Number(PI)));
        self.environment.constants.insert("π".to_string(), RunVal::new(RunType::Number(PI)));

        self.environment.intrinsics.insert("sin".to_string(), |args| {
            let Some(arg) = args.first() else {
                panic!("No arguments passed???");
            };

            match arg.typ {
                RunType::Number(n) => RunVal::new(RunType::Number(n.sin())),
                _ => {
                    panic!("Not a number!!!");
                }
            }
        });

        self.environment.intrinsics.insert("cos".to_string(), |args| {
            let Some(arg) = args.first() else {
                panic!("No arguments passed???");
            };

            match arg.typ {
                RunType::Number(n) => RunVal::new(RunType::Number(n.cos())),
                _ => {
                    panic!("Not a number!!!");
                }
            }
        });
    }
}
