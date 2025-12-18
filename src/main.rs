use chrono::offset;
use rand::Rng;
use std::collections::HashSet;
use std::{env, fs, io::Write};

fn _validate_voucher_pin(pin: &str) -> bool {
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
    let mut rng = rand::rng();

    let mut existing_pins = HashSet::new();
    let mut existing_serials = HashSet::new();
    let mut vouchers_generated = 0;

    let mut current_batch = 1;
    let args: Vec<String> = env::args().collect();
    let no_of_vouchers: &i32 = &args[1].clone().parse().expect("enter number of vouchers");
    let batch_size: &i32 = &args[2].clone().parse().expect("enter batch size");
    let output_file = &args[3];

    let mut file = fs::File::create(format!("{output_file}-{current_batch}.csv")).unwrap();
    file.write("pin, serial\n".as_bytes()).unwrap();

    loop {
        let serial: String = (0..20).map(|_| rng.random_range('0'..'9')).collect();
        let pin = {
            let mut generated;
            let mut rng = rand::rng();

            generated = String::new();

            loop {
                let generated_char: u8 = rng.random_range(0..10);
                let generated_char = (b'0' + generated_char) as char;

                if let Some(last_char) = generated.chars().last() {
                    if last_char == generated_char {
                        continue;
                    }
                }

                generated.push(generated_char);

                if generated.len() == 9 {
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
