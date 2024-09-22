use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{Expression, Node, Program, Statement},
    object::{Environment, Object},
};

use miette::{Result, Severity};

pub fn eval(node: Node, env: &Rc<RefCell<Environment>>) -> Result<Rc<Object>> {
    match node {
        Node::Program(program) => eval_program(&program, env),
        Node::Statement(stmt) => eval_statement(&stmt, env),
        Node::Expression(expr) => eval_expression(&expr, env),
    }
}

fn eval_program(program: &Program, env: &Rc<RefCell<Environment>>) -> Result<Rc<Object>> {
    let mut result = Rc::new(Object::Null);
    for stmt in program.statements() {
        result = eval_statement(stmt, env)?;

        // TODO return the inner of ReturnValue ???
        if let Object::ReturnValue(_) = *result {
            return Ok(result);
        };
    }
    Ok(result)
}

fn eval_statement(statement: &Statement, env: &Rc<RefCell<Environment>>) -> Result<Rc<Object>> {
    match statement {
        Statement::Let { token, name, value } => {
            let val = eval_expression(value, env)?;
            let mut borrow_env = env.as_ref().borrow_mut();
            borrow_env.set(name.into(), val);
            Ok(Rc::new(Object::Null))
        }
        Statement::Return { token, value } => {
            let val = eval_expression(value, env)?;
            Ok(Rc::new(Object::ReturnValue(val)))
        }
        Statement::Expr(expr) => Ok(eval_expression(expr, env)?),
    }
}

fn eval_expression(expression: &Expression, env: &Rc<RefCell<Environment>>) -> Result<Rc<Object>> {
    match expression {
        Expression::IntegerLiteral(i) => Ok(Rc::new(Object::Integer(*i))),
        Expression::Boolean(b) => Ok(Rc::new(Object::Boolean(*b))),
        Expression::Ident(identifier) => {
            let name = identifier.value();
            let env = env.as_ref().borrow();
            match env.get(name) {
                Some(val) => Ok(Rc::clone(&val)),
                None => Err(miette::miette!("identifier not found: {}", name,)),
            }
        }
        Expression::Prefix {
            token,
            operator,
            right,
        } => {
            let right_obj = eval_expression(right, env)?;
            eval_prefix_expression(operator, &right_obj)
        }
        Expression::Infix {
            token,
            operator,
            left,
            right,
        } => {
            let left_obj = eval_expression(left, env)?;
            let right_obj = eval_expression(right, env)?;
            eval_infix_expression(operator, &left_obj, &right_obj)
        }
        Expression::If {
            condition,
            consequence,
            alternative,
        } => {
            let condition = eval_expression(condition, env)?;
            match is_truthy(&condition) {
                true => eval_program(consequence, env),
                false => {
                    if let Some(alt) = alternative {
                        eval_program(alt, env)
                    } else {
                        Ok(Rc::new(Object::Null))
                    }
                }
            }
        }
        Expression::FunctionLiteral { parameters, body } => Ok(Rc::new(Object::Function {
            parameters: parameters.clone(),
            body: body.clone(),
            env: Rc::clone(env),
        })),
        Expression::Call {
            function,
            arguments,
        } => {
            let func = eval_expression(function, env)?;
            let args = eval_expressions(arguments, env)?;
            apply_function(func, args)
        }
        Expression::StringLiteral(s) => Ok(Rc::new(Object::String(s.into()))),
    }
}

fn eval_prefix_expression(operator: &str, right: &Object) -> Result<Rc<Object>> {
    match operator {
        "!" => {
            let res = match right {
                Object::Boolean(true) => false,
                Object::Boolean(false) => true,
                Object::Null => true,
                _ => false,
            };
            Ok(Rc::new(Object::Boolean(res)))
        }
        "-" => match right {
            Object::Integer(i) => Ok(Rc::new(Object::Integer(-i))),
            _ => Err(miette::miette!(
                severity = Severity::Error,
                //code = "expected::rparen",
                //help = "always close your parens",
                //labels = vec![LabeledSpan::at_offset(6, "here")],
                //url = "https://example.com",
                "unknown operator: -{}",
                right.r#type()
            )),
        },
        _ => Err(miette::miette!(
            severity = Severity::Error,
            //code = "expected::rparen",
            //help = "always close your parens",
            //labels = vec![LabeledSpan::at_offset(6, "here")],
            //url = "https://example.com",
            "unknown operator: {}{}",
            operator,
            right.r#type()
        )),
    }
}

fn eval_infix_expression(operator: &str, left: &Object, right: &Object) -> Result<Rc<Object>> {
    if right.r#type() != left.r#type() {
        return Err(miette::miette!(
            severity = Severity::Error,
            //code = "expected::rparen",
            //help = "always close your parens",
            //labels = vec![LabeledSpan::at_offset(6, "here")],
            //url = "https://example.com",
            "type mismatch: {} {} {}",
            left.r#type(),
            operator,
            right.r#type(),
        ));
    }

    match (left, operator, right) {
        (Object::Integer(l), "+", Object::Integer(r)) => Ok(Rc::new(Object::Integer(l + r))),
        (Object::Integer(l), "-", Object::Integer(r)) => Ok(Rc::new(Object::Integer(l - r))),
        (Object::Integer(l), "*", Object::Integer(r)) => Ok(Rc::new(Object::Integer(l * r))),
        (Object::Integer(l), "/", Object::Integer(r)) => Ok(Rc::new(Object::Integer(l / r))),

        (Object::Integer(l), "<", Object::Integer(r)) => Ok(Rc::new(Object::Boolean(l < r))),
        (Object::Integer(l), ">", Object::Integer(r)) => Ok(Rc::new(Object::Boolean(l > r))),
        (Object::Integer(l), "==", Object::Integer(r)) => Ok(Rc::new(Object::Boolean(l == r))),
        (Object::Integer(l), "!=", Object::Integer(r)) => Ok(Rc::new(Object::Boolean(l != r))),

        (Object::Boolean(l), "==", Object::Boolean(r)) => Ok(Rc::new(Object::Boolean(l == r))),
        (Object::Boolean(l), "!=", Object::Boolean(r)) => Ok(Rc::new(Object::Boolean(l != r))),

        (Object::String(l), "+", Object::String(r)) => {
            Ok(Rc::new(Object::String(format!("{}{}", l, r))))
        }
        _ => Err(miette::miette!(
            severity = Severity::Error,
            //code = "expected::rparen",
            //help = "always close your parens",
            //labels = vec![LabeledSpan::at_offset(6, "here")],
            //url = "https://example.com",
            "unknown operator: {} {} {}",
            left.r#type(),
            operator,
            right.r#type(),
        )),
    }
}

fn eval_expressions(
    expressions: &[Expression],
    env: &Rc<RefCell<Environment>>,
) -> Result<Vec<Rc<Object>>> {
    let mut result = Vec::new();
    for exp in expressions {
        let evaluated = eval_expression(exp, env)?;
        result.push(evaluated);
    }
    Ok(result)
}

fn apply_function(func: Rc<Object>, args: Vec<Rc<Object>>) -> Result<Rc<Object>> {
    match func.as_ref() {
        Object::Function {
            parameters,
            body,
            env,
        } => {
            let extended_env = {
                let mut new_env = Environment::new_enclosed(Rc::clone(env));
                for (param_idx, param) in parameters.iter().enumerate() {
                    new_env.set(param.value().into(), Rc::clone(&args[param_idx]));
                }
                new_env
            };
            let extended_env = Rc::new(RefCell::new(extended_env));
            let evaluated = eval_program(body, &extended_env)?;
            match evaluated.as_ref() {
                Object::ReturnValue(rc) => Ok(Rc::clone(rc)),
                _ => Ok(evaluated),
            }
        }
        _ => Err(miette::miette!("not a function: {}", func.r#type())),
    }
}

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Null => false,
        Object::Boolean(b) => *b,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::Identifier,
        lexer::Lexer,
        parser::Parser,
        token::{Token, TokenKind},
    };

    use super::*;

    fn test_eval(input: &str) -> Result<Rc<Object>> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let environment = Rc::new(RefCell::new(Environment::new()));
        eval(Node::Program(parser.parse_program()), &environment)
    }

    #[test]
    fn test_eval_integer_expression() {
        assert_eq!(test_eval("5").unwrap(), Rc::new(Object::Integer(5)));
        assert_eq!(test_eval("10").unwrap(), Rc::new(Object::Integer(10)));

        assert_eq!(test_eval("-5").unwrap(), Rc::new(Object::Integer(-5)));
        assert_eq!(test_eval("-10").unwrap(), Rc::new(Object::Integer(-10)));

        assert_eq!(
            test_eval("5 + 5 + 5 + 5 - 10").unwrap(),
            Rc::new(Object::Integer(10))
        );
        assert_eq!(
            test_eval("2 * 2 * 2 * 2 * 2").unwrap(),
            Rc::new(Object::Integer(32))
        );
        assert_eq!(
            test_eval("-50 + 100 + -50").unwrap(),
            Rc::new(Object::Integer(0))
        );
        assert_eq!(
            test_eval("5 * 2 + 10").unwrap(),
            Rc::new(Object::Integer(20))
        );
        assert_eq!(
            test_eval("5 + 2 * 10").unwrap(),
            Rc::new(Object::Integer(25))
        );
        assert_eq!(
            test_eval("20 + 2 * -10").unwrap(),
            Rc::new(Object::Integer(0))
        );
        assert_eq!(
            test_eval("50 / 2 * 2 + 10").unwrap(),
            Rc::new(Object::Integer(60))
        );
        assert_eq!(
            test_eval("2 * (5 + 10)").unwrap(),
            Rc::new(Object::Integer(30))
        );
        assert_eq!(
            test_eval("3 * 3 * 3 + 10").unwrap(),
            Rc::new(Object::Integer(37))
        );
        assert_eq!(
            test_eval("3 * (3 * 3) + 10").unwrap(),
            Rc::new(Object::Integer(37))
        );
        assert_eq!(
            test_eval("(5 + 10 * 2 + 15 / 3) * 2 + -10").unwrap(),
            Rc::new(Object::Integer(50))
        );
    }

    #[test]
    fn test_eval_boolean_expression() {
        assert_eq!(test_eval("true").unwrap(), Rc::new(Object::Boolean(true)));
        assert_eq!(test_eval("false").unwrap(), Rc::new(Object::Boolean(false)));
        assert_eq!(test_eval("1 < 2").unwrap(), Rc::new(Object::Boolean(true)));
        assert_eq!(test_eval("1 > 2").unwrap(), Rc::new(Object::Boolean(false)));
        assert_eq!(test_eval("1 < 1").unwrap(), Rc::new(Object::Boolean(false)));
        assert_eq!(test_eval("1 > 1").unwrap(), Rc::new(Object::Boolean(false)));
        assert_eq!(test_eval("1 == 1").unwrap(), Rc::new(Object::Boolean(true)));
        assert_eq!(
            test_eval("1 != 1").unwrap(),
            Rc::new(Object::Boolean(false))
        );
        assert_eq!(
            test_eval("1 == 2").unwrap(),
            Rc::new(Object::Boolean(false))
        );
        assert_eq!(test_eval("1 != 2").unwrap(), Rc::new(Object::Boolean(true)));
        assert_eq!(
            test_eval("true == true").unwrap(),
            Rc::new(Object::Boolean(true))
        );
        assert_eq!(
            test_eval("false == false").unwrap(),
            Rc::new(Object::Boolean(true))
        );
        assert_eq!(
            test_eval("true == false").unwrap(),
            Rc::new(Object::Boolean(false))
        );
        assert_eq!(
            test_eval("true != false").unwrap(),
            Rc::new(Object::Boolean(true))
        );
        assert_eq!(
            test_eval("false != true").unwrap(),
            Rc::new(Object::Boolean(true))
        );
        assert_eq!(
            test_eval("(1 < 2) == true").unwrap(),
            Rc::new(Object::Boolean(true))
        );
        assert_eq!(
            test_eval("(1 < 2) == false").unwrap(),
            Rc::new(Object::Boolean(false))
        );
        assert_eq!(
            test_eval("(1 > 2) == true").unwrap(),
            Rc::new(Object::Boolean(false))
        );
        assert_eq!(
            test_eval("(1 > 2) == false").unwrap(),
            Rc::new(Object::Boolean(true))
        );
    }

    #[test]
    fn test_bang_operator() {
        assert_eq!(test_eval("!true").unwrap(), Rc::new(Object::Boolean(false)));
        assert_eq!(test_eval("!false").unwrap(), Rc::new(Object::Boolean(true)));
        assert_eq!(test_eval("!5").unwrap(), Rc::new(Object::Boolean(false)));
        assert_eq!(test_eval("!!true").unwrap(), Rc::new(Object::Boolean(true)));
        assert_eq!(
            test_eval("!!false").unwrap(),
            Rc::new(Object::Boolean(false))
        );
        assert_eq!(test_eval("!!5").unwrap(), Rc::new(Object::Boolean(true)));
    }

    #[test]
    fn test_minus_operator() {
        assert_eq!(test_eval("5").unwrap(), Rc::new(Object::Integer(5)));
        assert_eq!(test_eval("10").unwrap(), Rc::new(Object::Integer(10)));
        assert_eq!(test_eval("-5").unwrap(), Rc::new(Object::Integer(-5)));
        assert_eq!(test_eval("-10").unwrap(), Rc::new(Object::Integer(-10)));
    }

    #[test]
    fn test_if_else_expression() {
        assert_eq!(
            test_eval("if (true) { 10 }").unwrap(),
            Rc::new(Object::Integer(10))
        );
        assert_eq!(
            test_eval("if (false) { 10 }").unwrap(),
            Rc::new(Object::Null)
        );
        assert_eq!(
            test_eval("if (1) { 10 }").unwrap(),
            Rc::new(Object::Integer(10))
        );
        assert_eq!(
            test_eval("if (1 < 2) { 10 }").unwrap(),
            Rc::new(Object::Integer(10))
        );
        assert_eq!(
            test_eval("if (1 > 2) { 10 }").unwrap(),
            Rc::new(Object::Null)
        );
        assert_eq!(
            test_eval("if (1 > 2) { 10 } else { 20 }").unwrap(),
            Rc::new(Object::Integer(20))
        );
        assert_eq!(
            test_eval("if (1 < 2) { 10 } else { 20 }").unwrap(),
            Rc::new(Object::Integer(10))
        );
    }

    #[test]
    fn test_return_statement() {
        let expected = Rc::new(Object::ReturnValue(Rc::new(Object::Integer(10))));
        assert_eq!(test_eval("return 10;").unwrap(), expected);
        assert_eq!(test_eval("return 10; 9;").unwrap(), expected);
        assert_eq!(test_eval("return 2 * 5; 9;").unwrap(), expected);
        assert_eq!(test_eval("9; return 2 * 5; 9;").unwrap(), expected);

        assert_eq!(
            test_eval(
                "
if (10 > 1) {
    if (10 > 1) {
        return 10;
    }
    return 1;
}"
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn test_error_handling() {
        match test_eval("5 + true;") {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e.to_string(), "type mismatch: INTEGER + BOOLEAN"),
        }

        match test_eval("5 + true; 5;") {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e.to_string(), "type mismatch: INTEGER + BOOLEAN"),
        }

        match test_eval("-true") {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e.to_string(), "unknown operator: -BOOLEAN"),
        }

        match test_eval("true + false") {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e.to_string(), "unknown operator: BOOLEAN + BOOLEAN"),
        }

        match test_eval("if (10 > 1) { if (10 > 1) {return true + false;} return 1; }") {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e.to_string(), "unknown operator: BOOLEAN + BOOLEAN"),
        }

        match test_eval("foobar") {
            Ok(_) => unreachable!(),
            Err(e) => assert_eq!(e.to_string(), "identifier not found: foobar"),
        }
    }

    #[test]
    fn test_let_statements() {
        assert_eq!(
            test_eval("let a = 5; a;").unwrap(),
            Rc::new(Object::Integer(5))
        );
        assert_eq!(
            test_eval("let a = 5 * 5; a;").unwrap(),
            Rc::new(Object::Integer(25))
        );
        assert_eq!(
            test_eval("let a = 5; let b = a; b;").unwrap(),
            Rc::new(Object::Integer(5))
        );
        assert_eq!(
            test_eval("let a = 5; let b = a; let c = a + b + 5; c;").unwrap(),
            Rc::new(Object::Integer(15))
        );
    }

    #[test]
    fn test_function_object() {
        let input = "fn(x) { x + 2; };";
        let mut body = Program::new();
        body.push(Statement::Expr(Expression::Infix {
            token: Token::new(TokenKind::Plus, 10, 10),
            operator: "+".into(),
            left: Box::new(Expression::Ident(Identifier::new("x".to_string()))),
            right: Box::new(Expression::IntegerLiteral(2)),
        }));
        let environment = Environment::new();
        let env = Rc::new(RefCell::new(environment));

        assert_eq!(
            test_eval(input).unwrap(),
            Rc::new(Object::Function {
                parameters: vec![Identifier::new("x".into())],
                body,
                env,
            })
        );
    }

    #[test]
    fn test_function_application() {
        assert_eq!(
            test_eval("let identity = fn(x) { x; }; identity(5);").unwrap(),
            Rc::new(Object::Integer(5))
        );
        assert_eq!(
            test_eval("let identity = fn(x) { return x; }; identity(5);").unwrap(),
            Rc::new(Object::Integer(5))
        );
        assert_eq!(
            test_eval("let double = fn(x) { x * 2; }; double(5);").unwrap(),
            Rc::new(Object::Integer(10))
        );
        assert_eq!(
            test_eval("let add = fn(x, y) { x + y; }; add(5, 5);").unwrap(),
            Rc::new(Object::Integer(10))
        );
        assert_eq!(
            test_eval("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));").unwrap(),
            Rc::new(Object::Integer(20))
        );
        assert_eq!(
            test_eval("fn(x) { x; }(5)").unwrap(),
            Rc::new(Object::Integer(5))
        );
    }

    #[test]
    fn test_closures() {
        let input = "
let newAdder = fn(x)
    { fn(y) {
        x + y
    };
};

let addTwo = newAdder(2);
addTwo(2);
";
        assert_eq!(test_eval(input).unwrap(), Rc::new(Object::Integer(4)));
    }

    #[test]
    fn test_string_literal() {
        let input = r#""Hello World!""#;
        assert_eq!(
            test_eval(input).unwrap(),
            Rc::new(Object::String("Hello World!".into()))
        );
    }

    #[test]
    fn test_string_concatenation() {
        let input = r#""Hello" + " " + "World!""#;
        assert_eq!(
            test_eval(input).unwrap(),
            Rc::new(Object::String("Hello World!".into()))
        );
    }
}
