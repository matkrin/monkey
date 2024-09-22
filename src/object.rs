use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::{BlockStatement, Identifier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Integer(isize),
    Boolean(bool),
    Null,
    ReturnValue(Rc<Object>),
    Function {
        parameters: Vec<Identifier>,
        body: BlockStatement,
        env: Rc<RefCell<Environment>>,
    },
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Null => write!(f, "null"),
            Object::ReturnValue(x) => write!(f, "{}", x),
            Object::Function {
                parameters,
                body,
                env: _,
            } => {
                let mut params = Vec::new();
                for param in parameters {
                    params.push(param.to_string());
                }
                write!(f, "fn({}){{\n{}\n}}", params.join(", "), body)
            }
        }
    }
}

impl Object {
    pub fn r#type(&self) -> String {
        match self {
            Object::Integer(_) => "INTEGER".into(),
            Object::Boolean(_) => "BOOLEAN".into(),
            Object::Null => "NULL".into(),
            Object::ReturnValue(_) => "RETURN_VALUE".into(),
            Object::Function {
                parameters: _,
                body: _,
                env: _,
            } => "FUNCTION".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub store: HashMap<String, Rc<Object>>,
    pub outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        let mut env = Environment::new();
        env.outer = Some(outer);
        env
    }

    pub fn get(&self, name: &str) -> Option<Rc<Object>> {
        match self.store.get(name) {
            Some(obj) => Some(Rc::clone(obj)),
            None => match &self.outer {
                Some(outer_env) => {
                    let outer_environment = Rc::clone(outer_env);
                    let outer_obj = outer_environment.borrow().get(name)?;
                    Some(Rc::clone(&outer_obj))
                }
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, val: Rc<Object>) {
        self.store.insert(name, val);
    }
}
