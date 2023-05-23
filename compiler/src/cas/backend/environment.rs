use matex_common::{function::Function, util::SymbolTable};

use super::value::RunVal;


#[derive(Default, Debug)]
pub struct Environment {
    pub scopes: Vec<Scope>,
}

impl Environment {
    pub fn get_scope(&mut self) -> &mut Scope {
        let Some(scope) = self.scopes.last_mut() else {
            panic!("No scope?!");
        };
        scope
    }
    pub fn push_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    pub fn pop_scope(&mut self) {
        let _ = self.scopes.pop();
    }
    pub fn get_function(&mut self, name: &str) -> Option<&Function> {
        for scope in self.scopes.iter().rev() {
            if let Some(function) = scope.functions.get(name) {
                return Some(function);
            }
        }
        return None;
    }
}

#[derive(Default, Debug)]
pub struct Scope {
    pub functions: SymbolTable<Function>,
    pub variables: SymbolTable<RunVal>,
}
