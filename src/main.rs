use lexer::Lexer;

mod ast;
mod lexer;
mod parser;
mod token;

fn main() {
    let lexer = Lexer::new("");
    println!("Hello, world!");
}
