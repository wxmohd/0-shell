use super::{ast::SimpleCommand, lexer::tokenize};

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub background: bool,
}

// Split by ';', detect trailing '&' per command, then tokenize.
pub fn parse_line(line: &str) -> Vec<ParsedCommand> {
    let mut out = Vec::new();
    for part in line.split(';') {
        let mut s = part.trim();
        if s.is_empty() { continue; }

        let mut background = false;
        if s.ends_with('&') {
            background = true;
            s = s.trim_end_matches('&').trim_end();
        }

        let tokens = tokenize(s);
        if tokens.is_empty() { continue; }
        let cmd = tokens[0].clone();
        let args = tokens[1..].to_vec();

        let _ast = SimpleCommand { cmd: cmd.clone(), args: args.clone(), background };
        out.push(ParsedCommand { cmd, args, background });
    }
    out
}
