use dotenvy::dotenv;
use generate_vouchers::config::Config;
use generate_vouchers::generator::Generator;
use generate_vouchers::logger::file::FileLogger;
use generate_vouchers::store::db::DbStore;
use generate_vouchers::store::memory::MemoryStore;
use std::env;

fn main() {
    dotenv().ok();

    let args = env::args();
    let config = Config::from_args(args);

    // stores
    let mut db_store = DbStore::init(&config);
    let mut memory_store = MemoryStore::init(&config);

    // logger
    let mut logger = FileLogger::new(&config);

    Generator::generate(&config, &mut db_store, &mut logger);
}
