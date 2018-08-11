mod interpreter;
mod parser;
mod spec;

use interpreter::Interpreter;
use parser::Parser;
use std::env;

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut parser = Parser::new();
    parser.parse(&args[1])?;
    let mut interpreter = Interpreter::new();
    interpreter.execute(parser.commands.iter().cloned())?;
    println!("{:?}", parser);
    println!("{:?}", interpreter);
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}
