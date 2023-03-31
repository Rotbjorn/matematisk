use std::cell::RefCell;
use std::rc::Rc;

use matex_common::{function::Function, util::SymbolTable};

use super::value::RuntimeVal;

#[derive(Default, Debug)]
pub struct Environment {
    pub scopes: Vec<Rc<RefCell<Scope>>>,
}

impl Environment {
    pub fn get_scope(&self) -> Rc<RefCell<Scope>> {
        let Some(scope) = self.scopes.last() else {
            panic!("No scope?!");
        };
        Rc::clone(scope)
    }
    pub fn push_scope(&mut self, scope: Rc<RefCell<Scope>>) {
        self.scopes.push(scope);
    }

    pub fn pop_scope(&mut self) {
        let _ = self.scopes.pop();
    }
}

#[derive(Default, Debug)]
pub struct Scope {
    pub functions: SymbolTable<Function>,
    pub variables: SymbolTable<RuntimeVal>,
}
