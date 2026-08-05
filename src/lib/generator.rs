use crate::config::Config;
use crate::logger::Logger;
use crate::store::{StoreAction, VoucherStore};
use crate::voucher::Voucher;
use chrono::offset;
use rand::TryRngCore;
use rand::rngs::OsRng;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

const PIN_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const SERIAL_ALPHABET: &[u8] = b"0123456789";

pub struct Generator {}

#[derive(Debug)]
pub enum GeneratorAction {
    Generate(usize),
    Stop,
}

fn validate_voucher_pin(pin: &str) -> bool {
    let mut char_iter = pin.chars().into_iter();

    let mut has_alpha = false;
    let mut has_numeric = false;

    while let Some(first) = char_iter.next() {
        if let Some(second) = char_iter.next() {
            // prevent consecutive characters
            if second == first {
                return false;
            }

            if !has_alpha && (first.is_alphabetic() | second.is_alphabetic()) {
                has_alpha = true;
            }

            if !has_numeric && (first.is_numeric() | second.is_numeric()) {
                has_numeric = true;
            }
        }
    }

    has_alpha & has_numeric
}

fn generate_random(rng: &mut OsRng, chars: &[u8], size: usize) -> String {
    let mut pin = String::with_capacity(size);
    let max_allowed = u8::MAX - chars.len() as u8;

    while pin.len() < size {
        let mut byte = [0u8; 1];

        rng.try_fill_bytes(&mut byte)
            .expect("failed to read secure random bytes from OS");

        if byte[0] < max_allowed {
            let index = byte[0] as usize % chars.len();
            pin.push(chars[index] as char);
        }
    }

    pin
}

impl Generator {
    pub fn new<T: VoucherStore>(config: &Config, store: T) -> JoinHandle<()> {
        let (send, mut rx) = mpsc::channel(5);
        let store = Arc::new(Mutex::new(store));

        let (store_sender, store_stream) = T::start(store.clone(), send.clone());
        let no_of_vouchers = config.no_of_vouchers;

        println!("[{}] Generator started at", offset::Local::now());

        let stream = tokio::spawn(async move {
            Generator::generate(no_of_vouchers, store_sender.clone()).await;
            store_sender.clone().send(StoreAction::Stop).await.unwrap();

            while let Some(action) = rx.recv().await {
                match action {
                    GeneratorAction::Generate(no_of_vouchers) => {
                        Generator::generate(no_of_vouchers, store_sender.clone()).await;
                        store_sender.clone().send(StoreAction::Stop).await.unwrap();
                    }
                    GeneratorAction::Stop => {
                        break;
                    }
                }
            }

            println!("[{}] Generator ended at", offset::Local::now());
        });

        let combined = tokio::spawn(async move {
            let _ = tokio::join!(store_stream, stream);
        });

        combined
    }

    pub async fn generate(total_vouchers: usize, sender: mpsc::Sender<StoreAction>) {
        let mut rng = OsRng;
        let mut total_generated = 0;

        loop {
            let serial = generate_random(&mut rng, SERIAL_ALPHABET, 20);

            let pin = {
                let mut generated;

                loop {
                    generated = generate_random(&mut rng, PIN_ALPHABET, 8);

                    if validate_voucher_pin(&generated) {
                        break;
                    }
                }

                generated
            };

            total_generated += 1;

            sender
                .send(StoreAction::Save(Voucher::new(
                    pin.as_str(),
                    serial.as_str(),
                )))
                .await
                .unwrap();

            if total_generated == total_vouchers {
                break;
            }
        }
    }

    // pub fn generate<VS: VoucherStore, W: Logger>(config: &Config, sender: mpsc::Sender<Voucher>) {
    //     let mut rng = OsRng;
    //     let mut vouchers_generated = 0;
    //     let mut vouchers = Vec::with_capacity(config.batch_size);
    //     let mut total_created = 0;
    //
    //     loop {
    //         let serial = generate_random(&mut rng, SERIAL_ALPHABET, 20);
    //
    //         let pin = {
    //             let mut generated;
    //
    //             loop {
    //                 generated = generate_random(&mut rng, PIN_ALPHABET, config.pin_size);
    //
    //                 if validate_voucher_pin(&generated) {
    //                     break;
    //                 }
    //             }
    //
    //             generated
    //         };
    //
    //         vouchers.push(Voucher::new(&pin, &serial));
    //         vouchers_generated += 1;
    //
    //         if vouchers_generated == (config.no_of_vouchers - total_created)
    //             || (vouchers_generated != 0 && vouchers_generated % config.batch_size == 0)
    //         {
    //             println!(
    //                 "[{}] Generated {} vouchers, attempting to save...",
    //                 offset::Local::now(),
    //                 vouchers_generated,
    //             );
    //
    //             let saved_vouchers = store.save(&mut vouchers.into_iter(), logger);
    //             total_created += saved_vouchers;
    //
    //             vouchers = Vec::with_capacity(config.batch_size);
    //             vouchers_generated = 0;
    //
    //             println!(
    //                 "[{}] Total generated: {}",
    //                 offset::Local::now(),
    //                 total_created,
    //             );
    //
    //             if total_created == config.no_of_vouchers {
    //                 break;
    //             }
    //         }
    //     }
    // }
}
