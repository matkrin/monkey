use std::io;
use std::io::Write;

use lexer::Lexer;
use token::Token;

mod ast;
mod lexer;
mod parser;
mod token;

const PROMPT: &str = "monkey❯";



fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();

    'outer: loop {
        input.clear();
        write!(stdout, "{} ", PROMPT).expect("Failed writing to stdout");
        io::stdout().flush().expect("Failed to flush stdout");

        stdin
            .read_line(&mut input)
            .expect("Failed to read line from stdin");

        let mut lexer = Lexer::new(&input);

        loop {
            match lexer.next_token() {
                Token::Eof => continue 'outer,
                t => writeln!(stdout, "{}", t).expect("Failed to write to stdout"),
            }
        }
    }
}
