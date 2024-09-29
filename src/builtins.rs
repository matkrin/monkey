use std::{cell::LazyCell, collections::HashMap, rc::Rc};
use miette::Result;

use crate::object::Object;

pub const BUILTINS: LazyCell<HashMap<String, Rc<Object>>> = LazyCell::new(|| {
    let mut b = HashMap::new();
    b.insert("len".into(), Rc::new(Object::Builtin(len)));
    b
});

fn len(args: Vec<Rc<Object>>) -> Result<Rc<Object>> {
    if args.len() != 1 {
        return Err(miette::miette!("wrong number of arguments. got={}, want = 1", args.len()));
    }
    let arg = args[0].as_ref();
    match arg {
        Object::String(s) => Ok(Rc::new(Object::Integer(s.chars().count() as isize))),
        _ => Err(miette::miette!("argument to `len` not supported, got {}", arg)),
    }
}
