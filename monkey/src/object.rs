use core::fmt;
use miette::Result;
use std::{cell::RefCell, collections::HashMap, hash, rc::Rc};

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
    String(String),
    Builtin(fn(Vec<Rc<Object>>) -> Result<Rc<Object>>),
    Array(Vec<Rc<Object>>),
    Hash(HashMap<Rc<Object>, Rc<Object>>)
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
                let params: Vec<_> = parameters.iter().map(|param| param.to_string()).collect();
                write!(f, "fn({}){{\n{}\n}}", params.join(", "), body)
            }
            Object::String(s) => write!(f, "{}", s),
            Object::Builtin(_) => write!(f, "builtin function"),
            Object::Array(v) => {
                let elements: Vec<_> = v.iter().map(|it| it.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Object::Hash(map) => {
                let pairs: Vec<_> = map.iter().map(|(key, val)|  format!("{}: {}", key, val) ).collect();
                write!(f, "{{{}}}", pairs.join(", "))
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
            Object::String(_) => "STRING".into(),
            Object::Builtin(_) => "BUITLIN".into(),
            Object::Array(_) => "ARRAY".into(),
            Object::Hash(_) => "HASH".into(),
        }
    }

    pub fn is_hashable(&self) -> bool {
        matches!(
            self,
            Object::Integer(_) | Object::Boolean(_) | Object::String(_)
        )
    }
}

impl hash::Hash for Object {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::Integer(i) => i.hash(state),
            Object::Boolean(b) => b.hash(state),
            Object::String(s) => s.hash(state),
            _ => panic!("Only Integers, Booleans and Strings are allowed as keys in a map"),
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
