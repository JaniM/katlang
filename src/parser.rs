use spec::CatCommand;
use std::iter::Peekable;

#[derive(Debug)]
pub struct Parser {
    pub commands: Vec<CatCommand>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { commands: vec![] }
    }

    pub fn parse(&mut self, text: &str) -> Result<(), String> {
        let mut chars = text.chars().peekable();
        loop {
            let c = if let Some(c) = chars.peek() {
                *c
            } else {
                return Ok(());
            };
            if c.is_whitespace() {
                chars.next();
            } else if c == '"' {
                chars.next();
                self.read_string(&mut chars);
            } else if c.is_digit(10) {
                self.read_digit(&mut chars)
            } else if self.read_command(&mut chars) {
            } else {
                return Err("Unexpected character: ".to_owned() + &c.to_string());
            }
        }
    }

    fn read_digit<I: Iterator<Item = char>>(&mut self, chars: &mut Peekable<I>) {
        let mut digits: Vec<i64> = vec![];
        loop {
            let c = if let Some(c) = chars.peek() {
                *c
            } else {
                break;
            };
            if let Some(d) = c.to_digit(10) {
                digits.push(d as i64);
                chars.next();
            } else {
                break;
            }
        }
        let num = digits
            .iter()
            .enumerate()
            .map(|(i, v)| 10i64.pow((digits.len() - i - 1) as u32) * v)
            .sum();
        self.commands.push(CatCommand::CreateInteger(num))
    }

    fn read_string<I: Iterator<Item = char>>(&mut self, chars: &mut Peekable<I>) {
        let mut buffer: Vec<char> = vec![];
        loop {
            let mut c = if let Some(c) = chars.next() {
                c
            } else {
                break;
            };
            if c == '\\' {
                c = if let Some(c) = chars.next() {
                    c
                } else {
                    break;
                };
            } else if c == '"' {
                break;
            }
            buffer.push(c);
        }
        self.commands
            .push(CatCommand::CreateString(buffer.into_iter().collect()));
    }

    fn read_command<I: Iterator<Item = char>>(&mut self, chars: &mut Peekable<I>) -> bool {
        let c = if let Some(c) = chars.peek() {
            *c
        } else {
            return false;
        };
        let cmd = match c {
            '+' => CatCommand::Add,
            'R' => CatCommand::ReadLine,
            'P' => CatCommand::WriteLine,
            _ => return false,
        };
        chars.next();
        self.commands.push(cmd);
        return true;
    }
}
