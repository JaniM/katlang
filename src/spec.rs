#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum CatCommand {
    StartBlock,
    CloseBlock,
    CreateInteger(i64),
    CreateString(String),
    CreateCommand(Box<CatCommand>),
    WriteLine,
    ReadLine,
    Add,
    Execute,
    ExecuteScoped,
    Map,
    ForEach,
    Split,
    ToInteger,
}
