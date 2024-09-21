use std::io::{self, BufReader, BufRead};
use std::io::{Read, Write};

use lexer::Lexer;
use parser::Parser;

mod ast;
mod lexer;
mod parser;
mod token;
mod object;

const PROMPT: &str = "monkey❯";



fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    start_repl(stdin, stdout);
}

fn start_repl(stdin: impl Read, mut stdout: impl Write) {
    let mut stdin = BufReader::new(stdin);
    let mut input = String::new();

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

        writeln!(stdout, "{}", program).expect("Failed writing to stdout");
    }

}
