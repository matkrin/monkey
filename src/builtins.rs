use std::{cell::LazyCell, collections::HashMap, rc::Rc};
use miette::Result;

use crate::object::Object;

pub const BUILTINS: LazyCell<HashMap<String, Rc<Object>>> = LazyCell::new(|| {
    let mut b = HashMap::new();
    b.insert("len".into(), Rc::new(Object::Builtin(len)));
    b.insert("first".into(), Rc::new(Object::Builtin(first)));
    b.insert("last".into(), Rc::new(Object::Builtin(last)));
    b.insert("rest".into(), Rc::new(Object::Builtin(rest)));
    b.insert("push".into(), Rc::new(Object::Builtin(push)));
    b
});

fn len(args: Vec<Rc<Object>>) -> Result<Rc<Object>> {
    if args.len() != 1 {
        return Err(miette::miette!("wrong number of arguments. got={}, want = 1", args.len()));
    }
    let arg = args[0].as_ref();
    match arg {
        Object::String(s) => Ok(Rc::new(Object::Integer(s.chars().count() as isize))),
        Object::Array(v) => Ok(Rc::new(Object::Integer(v.len() as isize))),
        _ => Err(miette::miette!("argument to `len` not supported, got {}", arg)),
    }
}

fn first(args: Vec<Rc<Object>>) -> Result<Rc<Object>> {
    if args.len() != 1 {
        return Err(miette::miette!("wrong number of arguments. got={}, want = 1", args.len()));
    }
    let arg = args[0].as_ref();
    match arg {
        Object::Array(v) => {
            if !v.is_empty() {
                return Ok(Rc::clone(&v[0]));
            }
            Ok(Rc::new(Object::Null))
    }
        _ => Err(miette::miette!("argument to `first` must be ARRAY, got {}", arg)),
    }
}

fn last(args: Vec<Rc<Object>>) -> Result<Rc<Object>> {
    if args.len() != 1 {
        return Err(miette::miette!("wrong number of arguments. got={}, want = 1", args.len()));
    }
    let arg = args[0].as_ref();
    match arg {
        Object::Array(v) => {
            if !v.is_empty() {
                return Ok(Rc::clone(v.last().unwrap()));
            }
            Ok(Rc::new(Object::Null))
    }
        _ => Err(miette::miette!("argument to `first` must be ARRAY, got {}", arg)),
    }
}

fn rest(args: Vec<Rc<Object>>) -> Result<Rc<Object>> {
    if args.len() != 1 {
        return Err(miette::miette!("wrong number of arguments. got={}, want = 1", args.len()));
    }
    let arg = args[0].as_ref();
    match arg {
        Object::Array(v) => {
            if !v.is_empty() {
                let new_elements = v[1..v.len()].to_vec();
                return Ok(Rc::new(Object::Array(new_elements)));
            }
            Ok(Rc::new(Object::Null))
    }
        _ => Err(miette::miette!("argument to `rest` must be ARRAY, got {}", arg.r#type())),
    }
}

fn push(args: Vec<Rc<Object>>) -> Result<Rc<Object>> {
    if args.len() != 2 {
        return Err(miette::miette!("wrong number of arguments. got={}, want = 2", args.len()));
    }
    let arg = args[0].as_ref();
    match arg {
        Object::Array(v) => {
            let mut new_elements = v.clone();
            new_elements.push(Rc::clone(&args[1]));
            Ok(Rc::new(Object::Array(new_elements)))
    }
        _ => Err(miette::miette!("argument to `push` must be ARRAY, got {}", arg.r#type())),
    }
}

