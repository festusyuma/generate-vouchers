use crate::config::Config;
use crate::logger::Logger;
use crate::voucher::Voucher;
use std::fs;
use std::io::Write;

pub struct FileLogger {
    file_name: String,
    current_file: fs::File,
    current_batch: usize,
    written: usize,
    batch_size: usize,
}

impl FileLogger {
    pub fn new(config: &Config) -> FileLogger {
        let file_name = config
            .output_file
            .clone()
            .unwrap_or(config.group_id.to_string());

        let mut current_file =
            fs::File::create(format!("{file_name}-{}.csv", config.initial_batch)).unwrap();
        current_file.write("pin, serial\n".as_bytes()).unwrap();

        FileLogger {
            file_name,
            current_file,
            written: 0,
            current_batch: config.initial_batch,
            batch_size: config.batch_size,
        }
    }
}

impl Logger for FileLogger {
    fn log(&mut self, voucher: Voucher) {
        if self.written == self.batch_size {
            self.written = 0;
            self.current_batch += 1;

            self.current_file =
                fs::File::create(format!("{}-{}.csv", self.file_name, self.current_batch)).unwrap();

            self.current_file.write("pin, serial\n".as_bytes()).unwrap();
        }

        let (pin, serial) = voucher.get();

        self.current_file
            .write(format!("{}, {}\n", pin, serial).as_bytes())
            .unwrap();

        self.written += 1;
    }
}
