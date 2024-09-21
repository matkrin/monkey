use crate::{
    ast::{Expression, Node, Program, Statement},
    object::Object,
};

use miette::Result;

pub fn eval(node: Node) -> Result<Object> {
    match node {
        Node::Program(program) => eval_program(program),
        Node::Statement(stmt) => eval_statement(stmt),
        Node::Expression(expr) => eval_expression(&expr),
    }
}

fn eval_program(program: Program) -> Result<Object> {
    let mut result = Object::Null;
    for stmt in program.statements() {
        result = eval(Node::Statement(stmt))?;
    }
    Ok(result)
}

fn eval_statement(statement: Statement) -> Result<Object> {
    match statement {
        Statement::Let { token, name, value } => todo!(),
        Statement::Return { token, value } => todo!(),
        Statement::Expr(expr) => eval_expression(&expr),
    }
}

fn eval_expression(expression: &Expression) -> Result<Object> {
    match expression {
        Expression::IntegerLiteral(i) => Ok(Object::Integer(*i)),
        Expression::Boolean(b) => Ok(Object::Boolean(*b)),
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
        } => todo!(),
        Expression::If {
            condition,
            consequence,
            alternative,
        } => todo!(),
        Expression::FunctionLiteral { parameters, body } => todo!(),
        Expression::Call {
            function,
            arguments,
        } => todo!(),
    }
}

fn eval_prefix_expression(operator: &str, right: &Object) -> Result<Object> {
    let res = match operator {
        "!" => {
            let res = match right {
                Object::Boolean(true) => false,
                Object::Boolean(false) => true,
                Object::Null => true,
                _ => false,
            };
            Object::Boolean(res)
        }
        "-" => match right {
            Object::Integer(i) => Object::Integer(-i),
            _ => Object::Null,
        },
        _ =>Object::Null
    };

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    fn test_eval(input: &str) -> Result<Object> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        eval(Node::Program(parser.parse_program()))
    }

    #[test]
    fn test_eval_integer_expression() {
        assert_eq!(test_eval("5").unwrap(), Object::Integer(5));
        assert_eq!(test_eval("10").unwrap(), Object::Integer(10));
    }

    #[test]
    fn test_eval_boolean_expression() {
        assert_eq!(test_eval("true").unwrap(), Object::Boolean(true));
        assert_eq!(test_eval("false").unwrap(), Object::Boolean(false));
    }

    #[test]
    fn test_bang_operator() {
        assert_eq!(test_eval("!true").unwrap(), Object::Boolean(false));
        assert_eq!(test_eval("!false").unwrap(), Object::Boolean(true));
        assert_eq!(test_eval("!5").unwrap(), Object::Boolean(false));
        assert_eq!(test_eval("!!true").unwrap(), Object::Boolean(true));
        assert_eq!(test_eval("!!false").unwrap(), Object::Boolean(false));
        assert_eq!(test_eval("!!5").unwrap(), Object::Boolean(true));
    }

    #[test]
    fn test_minus_operator() {
        assert_eq!(test_eval("5").unwrap(), Object::Integer(5));
        assert_eq!(test_eval("10").unwrap(), Object::Integer(10));
        assert_eq!(test_eval("-5").unwrap(), Object::Integer(-5));
        assert_eq!(test_eval("-10").unwrap(), Object::Integer(-10));
    }
}
