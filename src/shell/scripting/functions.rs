use std::collections::HashMap;

/// Minimal placeholder for function definitions:
/// myfunc() { echo "hi"; }
#[derive(Default)]
pub struct FunctionTable {
    pub map: HashMap<String, String>, // name -> body (raw)
}

impl FunctionTable {
    pub fn define(&mut self, name: String, body: String) {
        self.map.insert(name, body);
    }
    pub fn get(&self, name: &str) -> Option<&str> {
        self.map.get(name).map(|s| s.as_str())
    }
}
