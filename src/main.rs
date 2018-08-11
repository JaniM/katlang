extern crate itertools;

mod interpreter;
mod parser;
mod spec;

use interpreter::{CatValue::VStack, ExecFrame, Interpreter};
use parser::Parser;
use std::env;

fn print_frame(frame: ExecFrame, depth: usize) {
    println!(
        "> {} {: <30} {}{} | Stack before: {: <40} | Stack after: {}",
        " ".repeat(depth * 2),
        format!("{:?}", frame.command),
        " ".repeat(10 - depth * 2),
        if frame.reading { "(read)" } else { "      " },
        {
            let v = VStack(frame.stack_before).debug_stringify();
            if v.len() > 37 {
                format!(
                    "...{}",
                    v.chars()
                        .rev()
                        .take(37)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>()
                )
            } else {
                v
            }
        },
        VStack(frame.stack_after).debug_stringify(),
    );
    frame
        .inner_frames
        .into_iter()
        .for_each(|f| print_frame(f, depth + 1));
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut parser = Parser::new();
    parser.parse(&args[1])?;
    let mut interpreter = Interpreter::new();
    for command in parser.commands.iter().cloned() {
        interpreter.execute_single(command.clone())?;
        for frame in interpreter.exec_frames.drain(0..) {
            print_frame(frame, 0);
        }
    }
    interpreter.pop().map(|v| {
        v.auto_do(|v| -> Result<(), ()> {
            println!("{}", v.stringify());
            Ok(())
        })
    });
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
