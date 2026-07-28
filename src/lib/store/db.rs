use crate::config::Config;
use crate::store::VoucherStore;
use crate::voucher::Voucher;
use crate::writer::Writer;
use openssl::ssl::{SslConnector, SslMethod};
use postgres::Client;
use postgres::NoTls;
use postgres_openssl::MakeTlsConnector;
use std::env;
use uuid::Uuid;

struct DbConfig {
    client: Client,
    batch_size: usize,
}

impl DbConfig {
    fn new(config: &Config) -> Self {
        // Set variable from either the env variable or config
        let db_url = env::var("DATABASE_URL");
        let db_url = db_url
            .as_deref()
            .unwrap_or(config.db_url.as_deref().expect("database url not set"));

        let mut db_url = url::Url::parse(db_url).expect("Failed to parse url");
        db_url.set_query(None);

        let client = if let Some(cert_file) = &config.db_cert {
            let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();

            builder.set_ca_file(cert_file).unwrap();

            let connector = MakeTlsConnector::new(builder.build());

            Client::connect(&db_url.to_string(), connector).unwrap()
        } else {
            Client::connect(&db_url.to_string(), NoTls).unwrap()
        };

        let batch_size = config.db_batch_size.unwrap_or(10000);

        DbConfig { client, batch_size }
    }
}

pub struct DbStore {
    group: Uuid,
    config: DbConfig,
}

impl DbStore {
    pub fn init(app_config: &Config) -> Self {
        let config = DbConfig::new(app_config);
        let group = app_config.group_id;

        DbStore { group, config }
    }

    fn insert_many(&mut self, vouchers: Vec<Voucher>) -> Vec<Voucher> {
        let client = &mut self.config.client;
        let mut pins = Vec::new();
        let mut serials = Vec::new();

        for item in &vouchers {
            let (pin, serial) = item.get();

            pins.push(pin);
            serials.push(serial);
        }

        let query = "
            INSERT INTO public.vouchers (pin, serial, group_id)
            SELECT pin, serial, $3::uuid
            FROM UNNEST($1::text[], $2::text[]) AS v(pin, serial)
            ON CONFLICT (group_id, serial) DO NOTHING
            RETURNING pin, serial;
        ";

        let rows_affected = client.query(query, &[&pins, &serials, &self.group]);

        if let Err(e) = &rows_affected {
            println!("{:?}", e)
        }

        let mut vouchers = Vec::new();

        if let Ok(rows) = rows_affected {
            for row in rows {
                vouchers.push(Voucher::new(row.get(0), row.get(1)));
            }
        }

        println!("inserted {} rows", vouchers.len());

        vouchers
    }

    fn write<TR: Writer>(&mut self, batch: Vec<Voucher>, writer: Option<&mut TR>) {
        let saved_vouchers = self.insert_many(batch);

        if let Some(w) = writer {
            for voucher in saved_vouchers {
                w.write(voucher);
            }
        }
    }
}

impl VoucherStore for DbStore {
    fn save<T: Iterator<Item = Voucher>, TR: Writer>(
        &mut self,
        vouchers: &mut T,
        mut writer: Option<TR>,
    ) {
        let mut batch = Vec::new();

        while let Some(voucher) = vouchers.next() {
            if batch.len() == self.config.batch_size {
                self.write(batch, writer.as_mut());
                batch = Vec::new();
            }

            batch.push(voucher);
        }

        if !batch.is_empty() {
            self.write(batch, writer.as_mut());
        }
    }
}
