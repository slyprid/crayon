#[derive(Debug, Default)]
pub struct Runtime {
    pub lines: Vec<String>,
    pub clear_requested: bool,
}

impl Runtime {
    pub fn cls(&mut self) {
        self.lines.clear();
        self.clear_requested = true;
    }

    pub fn print(&mut self, s: String) {
        self.lines.push(s);
    }
}