use crate::config::Config;
use crate::logger::Logger;
use crate::voucher::Voucher;
use std::fmt::format;
use std::fs;
use std::io::Write;
use tokio::sync::mpsc::{self, Sender};
use tokio::task::JoinHandle;

struct FileWriter {
    file_name: String,
    current_file: fs::File,
    current_batch: usize,
    written: usize,
    batch_size: usize,
}

impl FileWriter {
    fn new(config: &Config) -> Self {
        let file_name = config
            .output_file
            .clone()
            .unwrap_or(format!(".codes/{}", config.group_id));

        let mut current_file =
            fs::File::create(format!("{file_name}-{}.csv", config.initial_batch)).unwrap();
        current_file.write("pin, serial\n".as_bytes()).unwrap();

        Self {
            file_name,
            current_file,
            written: 0,
            current_batch: config.initial_batch,
            batch_size: config.batch_size,
        }
    }

    fn log(&mut self, vouchers: Vec<Voucher>) {
        for voucher in vouchers {
            if self.written == self.batch_size {
                self.written = 0;
                self.current_batch += 1;

                self.current_file =
                    fs::File::create(format!("{}-{}.csv", self.file_name, self.current_batch))
                        .unwrap();

                self.current_file.write("pin, serial\n".as_bytes()).unwrap();
            }

            let (pin, serial) = voucher.get();

            self.current_file
                .write(format!("{}, {}\n", pin, serial).as_bytes())
                .unwrap();

            self.written += 1;
        }
    }
}

pub struct FileLogger {
    se: Sender<Voucher>,
}

impl FileLogger {
    pub fn new(config: &Config) -> (FileLogger, JoinHandle<()>) {
        let mut config = FileWriter::new(config);
        let (se, mut rx) = mpsc::channel(100);

        let stream = tokio::spawn(async move {
            let mut vouchers = vec![];

            while let Some(voucher) = rx.recv().await {
                vouchers.push(voucher);

                if vouchers.len() >= 10_000 {
                    config.log(vouchers);
                    vouchers = Vec::with_capacity(10_000);
                }
            }

            config.log(vouchers);
        });

        (FileLogger { se }, stream)
    }
}

impl Logger for FileLogger {
    fn log(&self, voucher: Voucher) {
        let sender = self.se.clone();

        tokio::spawn(async move {
            sender.send(voucher).await.unwrap();
        });
    }

    fn log_all(&self, vouchers: Vec<Voucher>) {
        for voucher in vouchers {
            self.log(voucher);
        }
    }
}
