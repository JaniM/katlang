use itertools::Itertools;
use spec::CatCommand;
use std::io::{self, BufRead};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum CatValue {
    VInteger(i64),
    VString(String),
    VStack(Vec<CatValue>),
    VCommand(CatCommand),
}
use self::CatValue::*;

impl CatValue {
    pub fn stringify(&self) -> String {
        match self {
            VInteger(v) => v.to_string(),
            VString(v) => v.clone(),
            VStack(v) => format!("[{}]", v.iter().map(|x| x.stringify()).join(" ")),
            VCommand(v) => format!("{:?}", v),
        }
    }

    pub fn debug_stringify(&self) -> String {
        match self {
            VInteger(v) => v.to_string(),
            VString(v) => format!("\"{}\"", v),
            VStack(v) => format!("[{}]", v.iter().map(|x| x.debug_stringify()).join(", ")),
            VCommand(v) => format!("{:?}", v),
        }
    }

    pub fn auto_map<E>(
        &self,
        mut func: impl FnMut(&CatValue) -> Result<CatValue, E>,
    ) -> Result<CatValue, E> {
        match self {
            VStack(vec) => Ok(VStack(
                vec.into_iter()
                    .map(func)
                    .collect::<Result<Vec<CatValue>, _>>()?,
            )),
            x => func(x),
        }
    }

    pub fn auto_do<E>(&self, mut func: impl FnMut(&CatValue) -> Result<(), E>) -> Result<(), E> {
        match self {
            VStack(vec) => {
                vec.into_iter().map(func).collect::<Result<(), _>>()?;
                Ok(())
            }
            x => func(x),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecFrame {
    pub stack_before: Vec<CatValue>,
    pub stack_after: Vec<CatValue>,
    pub reading: bool,
    pub command: CatCommand,
    pub inner_frames: Vec<ExecFrame>,
}

#[derive(Debug)]
pub struct Interpreter {
    pub exec_frames: Vec<ExecFrame>,
    pub main_stack: Vec<CatValue>,
    block_stack: Option<Vec<CatValue>>,
    block_depth: i64,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            main_stack: vec![],
            block_stack: None,
            block_depth: 0,
            exec_frames: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn execute(&mut self, commands: impl Iterator<Item = CatCommand>) -> Result<(), String> {
        for command in commands {
            self.execute_single(command)?;
        }
        Ok(())
    }

    pub fn execute_single(&mut self, command: CatCommand) -> Result<(), String> {
        let stack_before = self.main_stack.clone();
        let frame_len = self.exec_frames.len();

        self.execute_single_f(command.clone())?;

        let inner_frames = self.exec_frames.drain(frame_len..).collect();

        self.exec_frames.push(ExecFrame {
            stack_before: stack_before,
            stack_after: self.main_stack.clone(),
            reading: self.block_stack.is_some(),
            command: command.clone(),
            inner_frames: inner_frames,
        });
        Ok(())
    }

    pub fn execute_single_f(&mut self, command: CatCommand) -> Result<(), String> {
        let block_done = if let Some(ref mut bl) = self.block_stack {
            match command {
                CatCommand::StartBlock => {
                    bl.push(VCommand(CatCommand::StartBlock));
                    self.block_depth += 1;
                    false
                }
                CatCommand::CloseBlock => {
                    self.block_depth -= 1;
                    self.block_depth == 0
                }
                cmd => {
                    bl.push(VCommand(cmd));
                    return Ok(());
                }
            }
        } else {
            false
        };
        if block_done {
            let bl = self.block_stack.take().unwrap();
            self.push(VStack(bl));
            return Ok(());
        }
        match &command {
            CatCommand::StartBlock => {
                self.block_stack = Some(Vec::new());
                self.block_depth += 1;
            }
            CatCommand::CloseBlock => {
                return Err("Closing outside a block".to_owned());
            }
            CatCommand::CreateString(v) => self.push(VString(v.clone())),
            CatCommand::CreateInteger(v) => self.push(VInteger(*v)),
            CatCommand::CreateCommand(v) => self.push(VCommand(*v.clone())),
            CatCommand::ReadLine => {
                let mut line = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut line).unwrap();
                line.pop();
                self.push(VString(line));
            }
            CatCommand::WriteLine => {
                println!("{}", self.pop_res()?.stringify());
            }
            CatCommand::Add => self.run_add()?,
            CatCommand::Execute => match self.pop_res()? {
                VStack(cmds) => {
                    for cmd in cmds {
                        match cmd {
                            VCommand(c) => self.execute_single(c)?,
                            _ => return Err("Executed stack has non-command values".to_owned()),
                        };
                    }
                }
                VCommand(cmd) => self.execute_single(cmd)?,
                _ => return Err("Can't execute".to_owned()),
            },
            CatCommand::ExecuteScoped => {
                let mut interp = Interpreter::new();
                let commands = match self.pop_res()? {
                    VStack(commands) => commands,
                    _ => return Err("Expected a stack".to_owned()),
                };
                for cmd in commands {
                    match cmd {
                        VCommand(c) => interp.execute_single(c)?,
                        _ => return Err("Executed stack has non-command values".to_owned()),
                    };
                }
                self.push(VStack(interp.main_stack));
            }
            CatCommand::Map => {
                let func = self.pop_res()?;
                let values = match self.pop_res()? {
                    VStack(v) => v,
                    VString(v) => v.chars().map(|c| VString(c.to_string())).collect(),
                    _ => return Err("Map parameter isn't a stack or a string".to_owned()),
                };
                self.collect_frame(|this| -> Result<(), String> {
                    for val in values {
                        this.push(val);
                        this.push(func.clone());
                        this.execute_single(CatCommand::Execute)?;
                    }
                    Ok(())
                })?;
            }
            CatCommand::ForEach => {
                let func = self.pop_res()?;
                let values = match self.pop_res()? {
                    VStack(v) => v,
                    VString(v) => v.chars().map(|c| VString(c.to_string())).collect(),
                    _ => return Err("Map parameter isn't a stack or a string".to_owned()),
                };
                for val in values {
                    self.push(val);
                    self.push(func.clone());
                    self.execute_single(CatCommand::Execute)?;
                }
            }
            CatCommand::Split => {
                let separator = match self.pop_res()? {
                    VString(v) => v,
                    _ => return Err("Split parameter isn't a string".to_owned()),
                };
                let values = self.pop_res()?.auto_map(|x| match x {
                    VString(v) => Ok(VStack(
                        v.split(&separator).map(|x| VString(x.to_owned())).collect(),
                    )),
                    _ => Err("Split parameter isn't a string".to_owned()),
                })?;
                self.push(values);
            }
            CatCommand::ToInteger => {
                let val = self.pop_res()?.auto_map(|x| match x {
                    VInteger(v) => Ok(VInteger(*v)),
                    VString(v) => Ok(VInteger(
                        v.parse()
                            .map_err(|_| "String doesn't represent an integer")?,
                    )),
                    _ => Err("Can't convert value to string".to_owned()),
                })?;
                self.push(val);
            }
        };
        Ok(())
    }

    fn collect_frame<S, E>(
        &mut self,
        func: impl FnOnce(&mut Self) -> Result<S, E>,
    ) -> Result<S, E> {
        let orig_len = self.main_stack.len();
        let out = func(self)?;
        let result = self.main_stack.drain(orig_len..).collect();
        self.push(VStack(result));
        return Ok(out);
    }

    fn push(&mut self, val: CatValue) {
        self.main_stack.push(val)
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) -> Option<CatValue> {
        self.main_stack.pop()
    }

    fn pop_res(&mut self) -> Result<CatValue, &str> {
        self.main_stack.pop().ok_or("Pop from an empty stack")
    }

    fn run_add(&mut self) -> Result<(), String> {
        let v1 = self.pop_res()?;
        let v2 = self.pop_res()?;
        if let VStack(v1) = &v1 {
            if let VStack(mut v2) = v2 {
                v2.extend_from_slice(v1);
                self.push(VStack(v2));
                return Ok(());
            }
        }
        let result = v1.auto_map(|v1| match v1 {
            VInteger(v1) => v2.auto_map(|v2| match v2 {
                VInteger(v2) => Ok(VInteger(v2 + v1)),
                VString(v2) => Ok(VString(v2.clone() + &v1.to_string())),
                x => Err(format!("Can't add int and {:?}", x)),
            }),
            VString(v1) => v2.auto_map(|v2| match v2 {
                VInteger(v2) => Ok(VString(v2.to_string() + &v1)),
                VString(v2) => Ok(VString(v2.clone() + &v1)),
                x => Err(format!("Can't add string and {:?}", x)),
            }),
            x => Err(format!("Can't add {:?}", x)),
        })?;
        self.push(result);
        Ok(())
    }
}
