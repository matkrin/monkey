use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Integer(isize),
    Boolean(bool),
    Null,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Null => write!(f, "null"),
        }
    }
}

impl Object {
    pub fn r#type(&self) -> String {
        match self {
            Object::Integer(_) => "INTEGER".into(),
            Object::Boolean(_) => "BOOLEAN".into(),
            Object::Null => "NULL".into(),
        }
    }
}
