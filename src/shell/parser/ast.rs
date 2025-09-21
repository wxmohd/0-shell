#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub background: bool,
}
