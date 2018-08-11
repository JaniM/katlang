extern crate clap;
extern crate itertools;

mod interpreter;
mod parser;
mod spec;

use clap::{App, Arg};
use interpreter::{CatValue::VStack, ExecFrame, Interpreter};
use parser::Parser;
use std::time::Instant;

fn print_frame(frame: ExecFrame, depth: usize) {
    println!(
        ">  {: <40} {} | Stack before: {: <40} | Stack after: {}",
        format!("{}{:?}", " ".repeat(depth * 2), frame.command),
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
    let matches = App::new("Catlang")
        .version("1.0")
        .author("Jani Mustonen <janijohannes@kapsi.fi>")
        .about("A simple concatenative golf language")
        .arg(
            Arg::with_name("code")
                .short("c")
                .long("code")
                .value_name("CODE")
                .help("Executes a string directly")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("trace")
                .short("t")
                .help("Traces the entire execution"),
        )
        .get_matches();
    let code = matches.value_of("code").unwrap_or("\"Hello, world!\"P");
    let trace = matches.is_present("trace");

    let mut parser = Parser::new();
    parser.parse(code)?;
    println!("{:?}", parser.commands.clone());
    let mut interpreter = Interpreter::new(trace);
    let now = Instant::now();
    if trace {
        for command in parser.commands {
            interpreter.execute_single(&command)?;
            for frame in interpreter.exec_frames.drain(0..) {
                print_frame(frame, 0);
            }
        }
    } else {
        interpreter.execute(parser.commands.into_iter())?;
    }
    interpreter.pop().map(|v| {
        println!("{}", v.stringify());
    });
    let elapsed = now.elapsed();
    println!("{} s {} µs", elapsed.as_secs(), elapsed.subsec_micros());
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => println!("Error: {}", e),
    }
}
