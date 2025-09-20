use super::{ast::SimpleCommand, lexer::tokenize};

/// For now: simple "command arg1 arg2 ..." with basic quotes.
pub fn parse_command(line: &str) -> Option<(String, Vec<String>)> {
    let tokens = tokenize(line);
    if tokens.is_empty() {
        return None;
    }
    let cmd = tokens[0].clone();
    let args = tokens[1..].to_vec();
    let _ast = SimpleCommand { cmd: cmd.clone(), args: args.clone() };
    Some((cmd, args))
}
