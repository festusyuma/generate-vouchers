use dotenvy::dotenv;
use generate_vouchers::config::Config;
use generate_vouchers::generator::Generator;
use generate_vouchers::store::db::DbStore;
use generate_vouchers::writer::file::FileWriter;
use std::env;

fn main() {
    dotenv().ok();

    let args = env::args();
    let config = Config::new(args);

    let store = DbStore::init(&config);
    let writer = FileWriter::new(&config);

    let mut generator = Generator::new(&config, store, writer);

    generator.generate(config.no_of_vouchers);
}
