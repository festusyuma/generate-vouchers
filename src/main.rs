use dotenvy::dotenv;
use generate_vouchers::config::Config;
use generate_vouchers::generator::Generator;
use generate_vouchers::logger::file::FileLogger;
// use generate_vouchers::store::db::DbStore;
use generate_vouchers::store::memory::MemoryStore;
use std::env;
use std::error::Error;
use tokio::join;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let args = env::args();
    let config = Config::from_args(args);

    let (logger, logger_stream) = FileLogger::new(&config);

    // let mut _db_store = DbStore::init(&config);
    let memory_store = MemoryStore::new(&config);
    let generator_stream = Generator::new(&config, memory_store);

    drop(logger);

    let _ = join!(logger_stream, generator_stream);

    Ok(())
}
