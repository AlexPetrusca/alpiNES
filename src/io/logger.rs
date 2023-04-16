use std::fs::File;
use std::io::Write;

#[macro_export]
macro_rules! logln {
    ($dst:expr, $($arg:tt)*) => {
        $dst.logln(&format_args!($($arg)*).to_string())
    }
}

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(filepath: &str) -> Self {
        Self {
            file: File::create(filepath).unwrap()
        }
    }

    pub fn log(&mut self, text: &str) {
        self.file.write(text.as_ref()).unwrap();
    }

    pub fn logln(&mut self, text: &str) {
        self.file.write(text.as_ref()).unwrap();
        self.file.write("\n".as_ref()).unwrap();
    }
}