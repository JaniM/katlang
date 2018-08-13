use itertools::Itertools;
use spec::CatCommand;
use std::io::{self, BufRead, Write};
use std::mem;

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
            VStack(v) => format!("[{}]", v.iter().map(|x| x.debug_stringify()).join(" ")),
            VCommand(v) => format!("{:?}", v),
        }
    }

    pub fn debug_stringify(&self) -> String {
        match self {
            VInteger(v) => v.to_string(),
            VString(v) => format!(
                "\"{}\"",
                v.chars()
                    .map(|c| if c == '\n' {
                        "\\n".to_owned()
                    } else {
                        c.to_string()
                    })
                    .join("")
            ),
            VStack(v) => format!("[{}]", v.iter().map(|x| x.debug_stringify()).join(" ")),
            VCommand(v) => format!("{:?}", v),
        }
    }

    pub fn auto_map<E>(
        self,
        mut func: impl FnMut(CatValue) -> Result<CatValue, E>,
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

    pub fn auto_map_ref<E>(
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

    #[allow(dead_code)]
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
    pub side_stack: Vec<CatValue>,
    block_stack: Option<Vec<CatValue>>,
    block_depth: i64,
    collect_frame_pos: usize,
    trace: bool,
}

impl Interpreter {
    pub fn new(trace: bool) -> Interpreter {
        Interpreter {
            main_stack: Vec::new(),
            side_stack: Vec::new(),
            block_stack: None,
            block_depth: 0,
            exec_frames: vec![],
            collect_frame_pos: 0,
            trace: trace,
        }
    }

    #[allow(dead_code)]
    pub fn execute<'a>(
        &mut self,
        commands: impl Iterator<Item = &'a CatCommand>,
    ) -> Result<(), String> {
        for command in commands {
            self.execute_single(command)?;
        }
        Ok(())
    }

    pub fn execute_single(&mut self, command: &CatCommand) -> Result<(), String> {
        if self.trace {
            let stack_before = self.main_stack.clone();
            let frame_len = self.exec_frames.len();

            self.execute_single_f(command)?;

            let inner_frames = self.exec_frames.drain(frame_len..).collect();

            self.exec_frames.push(ExecFrame {
                stack_before: stack_before,
                stack_after: self.main_stack.clone(),
                reading: self.block_stack.is_some(),
                command: command.clone(),
                inner_frames: inner_frames,
            });
        } else {
            self.execute_single_f(command)?;
        }
        Ok(())
    }

    pub fn execute_single_f(&mut self, command: &CatCommand) -> Result<(), String> {
        let block_done = if let Some(ref mut bl) = self.block_stack {
            match command {
                CatCommand::StartBlock => {
                    bl.push(VCommand(CatCommand::StartBlock));
                    self.block_depth += 1;
                    return Ok(());
                }
                CatCommand::CloseBlock => {
                    self.block_depth -= 1;
                    if self.block_depth == 0 {
                        true
                    } else {
                        bl.push(VCommand(CatCommand::CloseBlock));
                        return Ok(());
                    }
                }
                cmd => {
                    bl.push(VCommand(cmd.clone()));
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
        match command {
            CatCommand::StartBlock => {
                self.block_stack = Some(Vec::new());
                self.block_depth += 1;
            }
            CatCommand::CloseBlock => {
                return Err("Closing outside a block".to_owned());
            }
            CatCommand::CreateString(v) => self.push(VString(v.clone())),
            &CatCommand::CreateInteger(v) => self.push(VInteger(v)),
            CatCommand::CreateCommand(v) => self.push(VCommand(*v.clone())),
            CatCommand::ReadLine => {
                io::stdout().flush().ok().expect("Could not flush stdout");
                let mut line = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut line).unwrap();
                line.pop();
                self.push(VString(line));
            }
            CatCommand::WriteLine => {
                println!("{}", self.pop_res()?.stringify());
            }
            CatCommand::Write => {
                print!("{}", self.pop_res()?.stringify());
            }
            CatCommand::Add => self.run_add()?,
            CatCommand::Multiply => {
                let v1 = self.pop_res()?;
                let v2 = self.pop_res()?;
                let val = v1.auto_map(|v1| match v1 {
                    VInteger(v1) => v2.auto_map_ref(|v2| match v2 {
                        VInteger(v2) => Ok(VInteger(v1 * v2)),
                        x => Err(format!("Not a multiplicative type: {:?}", x)),
                    }),
                    x => Err(format!("Not a multiplicative type: {:?}", x)),
                })?;
                self.push(val);
            }
            CatCommand::Execute => {
                let val = self.pop_res()?;
                self.execute_value(&val)?;
            }
            CatCommand::ExecuteScoped => {
                let val = self.pop_res()?;
                self.collect_frame(|this| -> Result<(), String> {
                    this.execute_value(&val)?;
                    Ok(())
                })?;
            }
            CatCommand::Map => {
                let func = self.pop_res()?;
                let values = match self.pop_res()? {
                    VStack(v) => v,
                    VString(v) => v.chars().map(|c| VString(c.to_string())).collect(),
                    _ => return Err("Map parameter isn't a stack or a string".to_owned()),
                };
                let mut results = Vec::new();
                for val in values {
                    self.collect_frame(|this| -> Result<(), String> {
                        this.push(val);
                        this.execute_value(&func)?;
                        Ok(())
                    })?;
                    let result = self.pop_res()?;
                    if let VStack(mut s) = result {
                        if s.len() == 1 {
                            results.push(s.swap_remove(0));
                        } else {
                            results.push(VStack(s));
                        }
                    } else {
                        results.push(result);
                    }
                }
                self.push(VStack(results));
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
                    self.execute_value(&func)?;
                }
            }
            CatCommand::Repeat => {
                let func = self.pop_res()?;
                let count = self.pop_res()?;
                count.auto_do(|count| match count {
                    VInteger(count) => {
                        for _ in 0..*count {
                            self.execute_value(&func)?;
                        }
                        Ok(())
                    }
                    VString(count) => {
                        for _ in 0..count.parse().map_err(|_| "Not a number")? {
                            self.execute_value(&func)?;
                        }
                        Ok(())
                    }
                    x => Err(format!("Not an integer: {:?}", x)),
                })?;
            }
            CatCommand::Split => {
                let separator = match self.pop_res()? {
                    VString(v) => v,
                    _ => return Err("Split parameter isn't a string".to_owned()),
                };
                let val = self.main_stack.last_mut().ok_or_else(|| "Empty stack")?;
                *val = val.auto_map_ref(|x| match x {
                    VString(v) => Ok(VStack(
                        v.split(&separator).map(|x| VString(x.to_owned())).collect(),
                    )),
                    _ => Err("Split parameter isn't a string".to_owned()),
                })?;
            }
            CatCommand::Join => {
                let separator = match self.pop_res()? {
                    VString(v) => v,
                    _ => return Err("Join parameter isn't a string".to_owned()),
                };
                let val = self.main_stack.last_mut().ok_or_else(|| "Empty stack")?;
                *val = match val {
                    VStack(ref v) => VString(v.iter().map(|x| x.stringify()).join(&separator)),
                    _ => return Err("Join parameter isn't a stack".to_owned()),
                };
            }
            CatCommand::ToInteger => {
                let val = self.main_stack.last_mut().ok_or_else(|| "Empty stack")?;
                *val = val.auto_map_ref(|x| match x {
                    VInteger(v) => Ok(VInteger(*v)),
                    VString(v) => Ok(VInteger(
                        v.parse()
                            .map_err(|_| "String doesn't represent an integer")?,
                    )),
                    _ => Err("Can't convert value to string".to_owned()),
                })?;
            }
            CatCommand::Range => {
                let count = self.main_stack.last_mut().ok_or_else(|| "Empty stack")?;
                *count = count.auto_map_ref(|end| match end {
                    VInteger(end) => Ok(VStack((1..end + 1).map(VInteger).collect())),
                    _ => Err("Range requires integer parameters"),
                })?;
            }
            CatCommand::Duplicate => {
                let item = self.copy_nth(0)?;
                self.push(item);
            }
            CatCommand::DuplicateSecond => {
                let item = self.copy_nth(1)?;
                self.push(item);
                self.swap(1, 0)?;
            }
            CatCommand::Drop => {
                self.pop_res()?;
            }
            &CatCommand::Rotate(n) => {
                for i in 0..n {
                    self.swap(n - i - 1, 0)?;
                }
            }
            CatCommand::PushSide => {
                let item = self.copy_nth(0)?;
                self.side_stack.push(item);
            }
            CatCommand::PopSide => {
                let item = self
                    .side_stack
                    .pop()
                    .ok_or_else(|| "Pop from empty side stack")?;
                self.push(item);
            }
            CatCommand::ConsumeSide => {
                let mut new_stack = Vec::new();
                mem::swap(&mut new_stack, &mut self.side_stack);
                self.push(VStack(new_stack));
            }
        };
        Ok(())
    }

    fn execute_value(&mut self, value: &CatValue) -> Result<(), String> {
        match value {
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
        }
        Ok(())
    }

    fn collect_frame<S, E>(
        &mut self,
        func: impl FnOnce(&mut Self) -> Result<S, E>,
    ) -> Result<S, E> {
        let orig_pos = self.collect_frame_pos;
        self.collect_frame_pos = self.main_stack.len() + 1;
        let out = func(self)?;
        let result = self
            .main_stack
            .drain(self.collect_frame_pos - 1..)
            .collect();
        self.collect_frame_pos = orig_pos;
        self.push(VStack(result));
        return Ok(out);
    }

    fn push(&mut self, val: CatValue) {
        self.main_stack.push(val)
    }

    pub fn pop(&mut self) -> Option<CatValue> {
        if self.main_stack.len() < self.collect_frame_pos {
            self.collect_frame_pos = self.main_stack.len();
        }
        self.main_stack.pop()
    }

    fn pop_res(&mut self) -> Result<CatValue, &'static str> {
        self.pop().ok_or("Pop from an empty stack")
    }

    fn copy_nth(&mut self, n: usize) -> Result<CatValue, &'static str> {
        if self.main_stack.len() <= n {
            Err("pop from an empty stack")
        } else {
            Ok(self.main_stack[self.main_stack.len() - n - 1].clone())
        }
    }

    fn swap(&mut self, n1: usize, n2: usize) -> Result<(), &'static str> {
        let len = self.main_stack.len();
        if len <= n1 || len <= n2 {
            Err("pop from an empty stack")
        } else {
            Ok(self.main_stack.swap(len - n1 - 1, len - n2 - 1))
        }
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
        let result = v1.auto_map(move |v1| match v1 {
            VInteger(v1) => v2.auto_map_ref(|v2| match v2 {
                VInteger(v2) => Ok(VInteger(v2 + v1)),
                VString(v2) => Ok(VString(v2.clone() + &v1.to_string())),
                x => Err(format!("Can't add int and {:?}", x)),
            }),
            VString(v1) => v2.auto_map_ref(|v2| match v2 {
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
