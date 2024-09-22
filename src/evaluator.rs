use std::rc::Rc;

use crate::{
    ast::{Expression, Node, Program, Statement},
    object::Object,
};

use miette::{Result, Severity};

pub fn eval(node: Node) -> Result<Rc<Object>> {
    match node {
        Node::Program(program) => eval_program(&program),
        Node::Statement(stmt) => eval_statement(&stmt),
        Node::Expression(expr) => eval_expression(&expr),
    }
}

fn eval_program(program: &Program) -> Result<Rc<Object>> {
    let mut result = Rc::new(Object::Null);
    for stmt in program.statements() {
        result = eval_statement(stmt)?;

        // TODO return the inner of ReturnValue ???
        if let Object::ReturnValue(_) = *result {
            return Ok(result);
        };
    }
    Ok(result)
}

fn eval_statement(statement: &Statement) -> Result<Rc<Object>> {
    match statement {
        Statement::Let { token, name, value } => todo!(),
        Statement::Return { token, value } => {
            let val = eval_expression(value)?;
            Ok(Rc::new(Object::ReturnValue(val)))
        }
        Statement::Expr(expr) => Ok(eval_expression(expr)?),
    }
}

fn eval_expression(expression: &Expression) -> Result<Rc<Object>> {
    match expression {
        Expression::IntegerLiteral(i) => Ok(Rc::new(Object::Integer(*i))),
        Expression::Boolean(b) => Ok(Rc::new(Object::Boolean(*b))),
        Expression::Ident(identifier) => todo!(),
        Expression::Prefix {
            token,
            operator,
            right,
        } => {
            let right_obj = eval_expression(right)?;
            eval_prefix_expression(operator, &right_obj)
        }
        Expression::Infix {
            token,
            operator,
            left,
            right,
        } => {
            let left_obj = eval_expression(left)?;
            let right_obj = eval_expression(right)?;
            eval_infix_expression(operator, &left_obj, &right_obj)
        }
        Expression::If {
            condition,
            consequence,
            alternative,
        } => {
            let condition = eval_expression(condition)?;
            match is_truthy(&condition) {
                true => eval_program(consequence),
                false => {
                    if let Some(alt) = alternative {
                        eval_program(alt)
                    } else {
                        Ok(Rc::new(Object::Null))
                    }
                }
            }
        }
        Expression::FunctionLiteral { parameters, body } => todo!(),
        Expression::Call {
            function,
            arguments,
        } => todo!(),
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

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Null => false,
        Object::Boolean(b) => *b,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    fn test_eval(input: &str) -> Result<Rc<Object>> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        eval(Node::Program(parser.parse_program()))
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
            Ok(_) => todo!(),
            Err(e) => assert_eq!(e.to_string(), "type mismatch: INTEGER + BOOLEAN"),
        }

        match test_eval("5 + true; 5;") {
            Ok(_) => todo!(),
            Err(e) => assert_eq!(e.to_string(), "type mismatch: INTEGER + BOOLEAN"),
        }

        match test_eval("-true") {
            Ok(_) => todo!(),
            Err(e) => assert_eq!(e.to_string(), "unknown operator: -BOOLEAN"),
        }

        match test_eval("true + false") {
            Ok(_) => todo!(),
            Err(e) => assert_eq!(e.to_string(), "unknown operator: BOOLEAN + BOOLEAN"),
        }

        match test_eval("if (10 > 1) { if (10 > 1) {return true + false;} return 1; }") {
            Ok(_) => todo!(),
            Err(e) => assert_eq!(e.to_string(), "unknown operator: BOOLEAN + BOOLEAN"),
        }
    }
}
