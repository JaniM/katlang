#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum CatCommand {
    CreateInteger(i64),
    CreateString(String),
    WriteLine,
    ReadLine,
    Add,
}
