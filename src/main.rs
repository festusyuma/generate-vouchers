use chrono::offset;
use dotenvy::dotenv;
use openssl::ssl::{SslConnector, SslMethod};
use postgres::{Client, NoTls};
use postgres_openssl::MakeTlsConnector;
use rand::Rng;
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashSet;
use std::{env, fs, io::Write};
use uuid::Uuid;

fn validate_voucher_pin(pin: &str) -> bool {
    let mut has_char = false;
    let mut has_digit = false;

    for c in pin.chars() {
        if !has_char {
            has_char = c.is_alphabetic()
        }

        if !has_digit {
            has_digit = c.is_numeric()
        }

        if has_digit & has_char {
            return true;
        }
    }

    false
}

fn main() {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let promo_id: &Uuid = &args[1].parse().expect("promo id is required");
    let no_of_vouchers: &i32 = &args[2].clone().parse().expect("enter number of vouchers");
    let batch_size: &i32 = &args[3].clone().parse().expect("enter batch size");
    let output_file: &str = &args[4];
    let cert_file = &args.get(5);

    let mut rng = rand::rng();

    let mut existing_pins = HashSet::new();
    let mut existing_serials = HashSet::new();
    let mut vouchers_generated = 0;

    // Access an environment variable
    let db_url_str = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let mut db_url = url::Url::parse(&db_url_str).expect("Failed to parse url");
    db_url.set_query(None);

    let mut db_client = if let Some(cert_file) = cert_file {
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_ca_file(cert_file).unwrap();

        let connector = MakeTlsConnector::new(builder.build());

        Client::connect(&db_url.to_string(), connector).unwrap()
    } else {
        Client::connect(&db_url.to_string(), NoTls).unwrap()
    };

    let mut last_pin: String = String::from("");
    let mut vouchers_loaded = 0;

    let query = db_client
        .prepare("SELECT pin, serial, promo_id FROM \"Voucher\" WHERE promo_id = $1 and pin > $2 ORDER BY pin LIMIT 100000")
        .expect("Error preparing query");

    loop {
        let items = db_client.query(&query, &[&promo_id, &last_pin]).unwrap();

        vouchers_loaded += items.len();
        last_pin = String::from("");

        for row in items {
            let pin: String = row.get(0);
            let serial: String = row.get(1);

            existing_pins.insert(pin.to_uppercase());
            existing_serials.insert(serial.to_uppercase());
            last_pin = pin;
        }

        if last_pin == "" {
            break;
        }

        println!("Loaded {vouchers_loaded} from DB")
    }

    let mut current_batch = 1;

    let mut file = fs::File::create(format!("{output_file}-{current_batch}.csv")).unwrap();
    file.write("pin, serial\n".as_bytes()).unwrap();

    loop {
        let serial: String = (0..20).map(|_| rng.random_range('0'..'9')).collect();
        let pin = {
            let mut generated;

            loop {
                generated = String::new();

                loop {
                    let generated_char = Alphanumeric
                        .sample_string(&mut rng, 1)
                        .to_uppercase()
                        .chars()
                        .last()
                        .expect("unable to generate character");

                    if generated.len() > 0 {
                        if generated.chars().last().unwrap() == generated_char {
                            continue;
                        }
                    }

                    generated.push(generated_char);

                    if generated.len() == 5 {
                        break;
                    }
                }

                if validate_voucher_pin(&generated) {
                    break;
                }
            }

            generated
        };

        if existing_pins.contains(&pin) || existing_serials.contains(&serial) {
            continue;
        }

        file.write(format!("{}, {}\n", pin, serial).as_bytes())
            .unwrap();

        vouchers_generated += 1;
        existing_pins.insert(pin);
        existing_serials.insert(serial);

        if &vouchers_generated == no_of_vouchers {
            break;
        }

        if &vouchers_generated % batch_size == 0 {
            current_batch += 1;

            println!(
                "Batch complete: Generated {} vouchers at {}",
                vouchers_generated,
                offset::Local::now()
            );

            file = fs::File::create(format!("{output_file}-{current_batch}.csv")).unwrap();
            file.write("pin, serial\n".as_bytes()).unwrap();
        }
    }
}
