use interpreter::Interpreter;
use itertools::Itertools;
use parser::Parser;
use std::io::{stdin, stdout, Write};
use termion::{clear, cursor, event::Key, input::TermRead, raw::IntoRawMode, terminal_size};

pub fn run_term() -> Result<(), String> {
    let stdin = stdin();
    // Enter raw mode.
    let mut stdout = stdout().into_raw_mode().unwrap();
    writeln!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    write!(stdout, "{}Write code below:", cursor::Goto(1, 1)).unwrap();

    let mut cursorpos = 0;
    let mut code = String::new();

    write!(
        stdout,
        "{}{}> {}{}",
        cursor::Goto(1, 2),
        clear::CurrentLine,
        code,
        cursor::Show
    ).unwrap();

    stdout.flush().unwrap();
    for c in stdin.keys() {
        // Print the key we type...
        match c.unwrap() {
            // Exit.
            Key::Esc => break,
            Key::Char(c) => {
                code.insert(cursorpos, c);
                cursorpos += 1;
            }
            Key::Backspace => {
                if code.len() > 0 {
                    code.remove(cursorpos - 1);
                    cursorpos -= 1;
                }
            }
            Key::Alt(c) => println!("Alt-{}", c),
            Key::Ctrl(c) => println!("Ctrl-{}", c),
            Key::Left => {
                if cursorpos > 0 {
                    cursorpos -= 1;
                }
            }
            Key::Right => {
                if cursorpos < code.len() {
                    cursorpos += 1;
                }
            }
            Key::Up => println!("<up>"),
            Key::Down => println!("<down>"),
            _ => println!("Other"),
        }

        write!(
            stdout,
            "{}{}Write code below:",
            clear::All,
            cursor::Goto(1, 1)
        ).unwrap();

        write!(
            stdout,
            "{}{}> {}",
            cursor::Goto(1, 2),
            clear::CurrentLine,
            code.chars()
                .map(|c| if c == '\n' {
                    "\u{2424}".to_owned()
                } else {
                    c.to_string()
                })
                .join("")
        ).unwrap();

        let mut parser = Parser::new();
        match parser.parse(&code) {
            Ok(()) => {}
            Err(e) => {
                write!(
                    stdout,
                    "{}{}Parse error: {}",
                    cursor::Goto(1, 3),
                    clear::CurrentLine,
                    e
                ).unwrap();
            }
        }
        let mut interpreter = Interpreter::new(false);
        match interpreter.execute(parser.commands.iter()) {
            Ok(()) => {}
            Err(e) => {
                write!(
                    stdout,
                    "{}{}Execution error: {}",
                    cursor::Goto(1, 3),
                    clear::CurrentLine,
                    e
                ).unwrap();
            }
        }
        write!(
            stdout,
            "{}{:<40} | {:<40} | {:<40}",
            cursor::Goto(1, 4),
            "Commands",
            "Stack",
            "Side stack"
        ).unwrap();
        let (_width, height) = terminal_size().unwrap_or((80, 30));
        for i in 0..(height - 5) as usize {
            let cmd = parser.commands.get(i);
            let stack_item = interpreter.main_stack.get(i);
            let side_item = interpreter.side_stack.get(i);
            write!(
                stdout,
                "{}{:<40} | {:<40} | {:<40}",
                cursor::Goto(1, 5 + i as u16),
                cmd.map(|x| format!("{:?}", x)).unwrap_or("".to_owned()),
                stack_item
                    .map(|x| x.debug_stringify().chars().take(30).join(""))
                    .unwrap_or("".to_owned()),
                side_item
                    .map(|x| x.debug_stringify().chars().take(30).join(""))
                    .unwrap_or("".to_owned())
            ).unwrap()
        }

        write!(stdout, "{}", cursor::Goto(3 + cursorpos as u16, 2),).unwrap();
        stdout.flush().unwrap();
    }

    writeln!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();

    Ok(())
}
