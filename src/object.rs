use core::fmt;
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Integer(isize),
    Boolean(bool),
    Null,
    ReturnValue(Rc<Object>),
    Error(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Null => write!(f, "null"),
            Object::ReturnValue(x) => write!(f, "{}", x),
            Object::Error(e) => write!(f, "ERROR: {}", e),
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
            Object::Error(_) => "ERROR_OBJ".into(),
        }
    }
}

pub struct Environment {
    store: HashMap<String, Rc<Object>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Rc<Object>> {
        self.store.get(name)
    }

    pub fn set(&mut self, name: String, val: Rc<Object>) {
        self.store.insert(name, val);
    }
}
