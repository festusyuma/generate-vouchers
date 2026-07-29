use std::str::FromStr;
use uuid::Uuid;

#[derive(Default)]
pub struct Config {
    pub group_id: Uuid,
    pub no_of_vouchers: usize,
    pub initial_batch: usize,
    pub batch_size: usize,
    pub output_file: Option<String>,

    pub db_batch_size: Option<usize>,
    pub db_url: Option<String>,
    pub db_cert: Option<String>,
}

fn unwrap_arg<Ta: Iterator<Item = String>, T: FromStr>(args: &mut Ta) -> Option<T> {
    args.next().and_then(|v| v.parse().ok())
}

impl Config {
    pub fn new<T: Iterator<Item = String>>(mut args: T) -> Self {
        let mut config = Config::default();
        let args = &mut args;

        config.batch_size = 1_000_000;
        config.initial_batch = 1;
        config.group_id = Uuid::new_v4();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--group" | "-g" => config.group_id = unwrap_arg(args).unwrap_or(Uuid::new_v4()),
                "--vouchers" | "-v" => {
                    config.no_of_vouchers =
                        unwrap_arg(args).expect("no of vouchers is required (e.g -v 100)")
                }
                "--batch" | "-b" => {
                    config.batch_size = unwrap_arg(args).unwrap_or(config.batch_size)
                }
                "--batch-start" => {
                    config.initial_batch = unwrap_arg(args).unwrap_or(config.initial_batch)
                }
                "--output" | "-o" => config.output_file = unwrap_arg(args),
                "--db-url" => config.db_url = unwrap_arg(args),
                "--db-cert" => config.db_cert = unwrap_arg(args),
                "--db-batch" => config.db_batch_size = unwrap_arg(args),
                _ => (),
            }
        }

        config
    }
}
