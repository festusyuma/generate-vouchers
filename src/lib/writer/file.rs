use crate::config::Config;
use crate::voucher::Voucher;
use crate::writer::Writer;
use std::fs;
use std::io::Write;

pub struct FileWriter {
    file_name: String,
    current_file: fs::File,
    current_batch: usize,
    written: usize,
    batch_size: usize,
}

impl FileWriter {
    pub fn new(config: &Config) -> FileWriter {
        let file_name = config
            .output_file
            .clone()
            .unwrap_or(config.group_id.to_string());

        let mut current_file = fs::File::create(format!("{file_name}-1.csv")).unwrap();
        current_file.write("pin, serial\n".as_bytes()).unwrap();

        FileWriter {
            file_name,
            current_file,
            current_batch: 1,
            written: 0,
            batch_size: config.batch_size,
        }
    }
}

impl Writer for FileWriter {
    fn write(&mut self, voucher: Voucher) {
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
