use std::cell::RefCell;
use std::io::{self, BufRead, BufReader};
use std::io::{Read, Write};
use std::rc::Rc;

use monkey::Node;
use monkey::eval;
use monkey::Lexer;
use monkey::Environment;
use monkey::Parser;

const PROMPT: &str = "monkey❯";

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    start_repl(stdin, stdout);
}

fn start_repl(stdin: impl Read, mut stdout: impl Write) {
    let mut stdin = BufReader::new(stdin);
    let mut input = String::new();
    let environment = Rc::new(RefCell::new(Environment::new()));

    loop {
        input.clear();
        write!(stdout, "{} ", PROMPT).expect("Failed writing to stdout");
        io::stdout().flush().expect("Failed to flush stdout");

        stdin
            .read_line(&mut input)
            .expect("Failed to read line from stdin");

        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        match eval(Node::Program(program), &environment) {
            Ok(evaluated) => writeln!(stdout, "{}", evaluated).expect("Failed writing to stdout"),
            Err(e) => writeln!(stdout, "{:?}", e).expect("Failed writing to stdout"),
        };
    }
}
