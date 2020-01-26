


pub struct Formatter {
    indent: String,
    code: String
}



impl Formatter {
    pub fn new() -> Self {
        Formatter {
            indent: String::new(),
            code: String::new()
        }
    }

    pub fn add_line(&mut self, line: &str) {
        self.code += &self.indent;
        self.code += line;
        self.code += "\n"
    }

    pub fn indent(&mut self) {
        self.indent += "    ";
    }
    pub fn unindent(&mut self) {
        if self.indent.len() >= 4 {
            self.indent = (&self.indent[..(self.indent.len() - 4)]).to_string();
        }
    }

    pub fn get_code(self) -> String {
        self.code
    }
}