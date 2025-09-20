/// Very small tokenizer that splits on whitespace but respects simple double quotes.
/// No escapes, no pipes/redirection yet.
pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            c if c.is_whitespace() && !in_quotes => {
                if !cur.is_empty() {
                    tokens.push(cur.clone());
                    cur.clear();
                }
            }
            _ => cur.push(ch),
        }
    }

    if !cur.is_empty() {
        tokens.push(cur);
    }

    tokens
}
