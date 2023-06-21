use matex_common::{function::Function, util::SymbolTable};

use super::value::RunVal;

type Intrinsic = fn(&Vec<RunVal>) -> RunVal;

#[derive(Default)]
pub struct Environment {
    pub scopes: Vec<Scope>,
    pub constants: SymbolTable<RunVal>,
    pub intrinsics: SymbolTable<Intrinsic>,
}

impl Environment {
    pub fn set_variable(&mut self, name: &str, value: RunVal) {
        self.get_scope_mut()
            .variables
            .insert(name.to_string(), value);
    }

    pub fn remove_variable(&mut self, name: &str) {
        self.get_scope_mut().variables.remove(name);
    }

    pub fn set_func(&mut self, name: &str, func: Function) {
        self.get_scope_mut()
            .functions
            .insert(name.to_string(), func);
    }

    pub fn remove_func(&mut self, name: &str) {
        self.get_scope_mut().functions.remove(name);
    }
}

impl Environment {
    pub fn get_scope_mut(&mut self) -> &mut Scope {
        let Some(scope) = self.scopes.last_mut() else {
            panic!("No scope?!");
        };
        scope
    }

    pub fn get_scope(&self) -> &Scope {
        let Some(scope) = self.scopes.last() else {
            panic!("No scope?!");
        };
        scope
    }

    pub(crate) fn push_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    pub(crate) fn pop_scope(&mut self) {
        let _ = self.scopes.pop();
    }

    pub(crate) fn get_intrinsic(&self, name: &str) -> Option<&Intrinsic> {
        return self.intrinsics.get(name);
    }

    pub(crate) fn get_function(&self, name: &str) -> Option<&Function> {
        for scope in self.scopes.iter().rev() {
            if let Some(function) = scope.functions.get(name) {
                return Some(function);
            }
        }
        None
    }

    pub(crate) fn get_variable(&self, name: &str) -> Option<&RunVal> {
        if let Some(variable) = self.get_scope().variables.get(name) {
            Some(variable)
        } else {
            self.constants.get(name)
        }
    }
}

#[derive(Default, Debug)]
pub struct Scope {
    pub functions: SymbolTable<Function>,
    pub variables: SymbolTable<RunVal>,
}
