use spec::CatCommand;
use std::io::{self, BufRead};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum CatValue {
    VInteger(i64),
    VString(String),
}
use self::CatValue::*;

impl CatValue {
    pub fn stringify(&self) -> String {
        match self {
            VInteger(v) => v.to_string(),
            VString(v) => v.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Interpreter {
    main_stack: Vec<CatValue>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { main_stack: vec![] }
    }

    pub fn execute(&mut self, commands: impl Iterator<Item = CatCommand>) -> Result<(), String> {
        for command in commands {
            self.execute_single(command)?;
        }
        Ok(())
    }

    fn execute_single(&mut self, command: CatCommand) -> Result<(), &str> {
        #[allow(unreachable_patterns)]
        match command {
            CatCommand::CreateString(v) => self.push(VString(v)),
            CatCommand::CreateInteger(v) => self.push(VInteger(v)),
            CatCommand::ReadLine => {
                let mut line = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut line).unwrap();
                line.pop();
                self.push(VString(line));
            }
            CatCommand::WriteLine => {
                println!("{}", self.pop().ok_or("Pop from empty stack")?.stringify());
            }
            CatCommand::Add => self.run_add()?,
            _ => return Err("Unimplemented command"),
        };
        Ok(())
    }

    fn push(&mut self, val: CatValue) {
        self.main_stack.push(val)
    }

    fn pop(&mut self) -> Option<CatValue> {
        self.main_stack.pop()
    }

    fn run_add(&mut self) -> Result<(), &str> {
        match self.pop() {
            Some(VInteger(v1)) => match self.pop() {
                Some(VInteger(v2)) => self.push(VInteger(v2 + v1)),
                Some(VString(v2)) => self.push(VString(v2 + &v1.to_string())),
                None => return Err("Adding from stack of depth 1"),
            },
            Some(VString(v1)) => match self.pop() {
                Some(VInteger(v2)) => self.push(VString(v2.to_string() + &v1)),
                Some(VString(v2)) => self.push(VString(v2 + &v1)),
                None => return Err("Adding from stack of depth 1"),
            },
            None => return Err("Adding from stack of depth 0"),
        }
        Ok(())
    }
}
