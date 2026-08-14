use crate::config::Config;
use crate::generator::GeneratorAction;
use crate::logger::Logger;
use crate::store::{StoreAction, VoucherStore};
use crate::voucher::Voucher;
use openssl::ssl::{SslConnector, SslMethod};
use postgres::NoTls;
use postgres_openssl::MakeTlsConnector;
use std::env;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio_postgres::{Client, connect};
use uuid::Uuid;

struct DbConfig {
    client: Client,
    batch_size: usize,
}

impl DbConfig {
    async fn new(config: &Config) -> Self {
        // Set variable from either the env variable or config
        let db_url = env::var("DATABASE_URL");
        let db_url = db_url
            .as_deref()
            .unwrap_or_else(|_| config.db_url.as_deref().expect("database url not set"));

        let db_url = url::Url::parse(db_url).expect("Failed to parse url");
        let batch_size = config.db_batch_size.unwrap_or(10000);

        if let Some(cert_file) = &config.db_cert {
            let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
            builder.set_ca_file(cert_file).unwrap();

            let connector = MakeTlsConnector::new(builder.build());
            let (client, connection) = connect(&db_url.to_string(), connector).await.unwrap();

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            return DbConfig { client, batch_size };
        }

        let (client, connection) = connect(&db_url.to_string(), NoTls).await.unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        DbConfig { client, batch_size }
    }
}

pub struct DbStore {
    group: Uuid,
    group_col_name: String,
    db: DbConfig,
    total_vouchers: usize,
    saved_vouchers: usize,
}

impl DbStore {
    pub async fn new(app_config: &Config) -> Self {
        DbStore {
            db: DbConfig::new(app_config).await,
            group: app_config.group_id,
            group_col_name: app_config.db_group_col_name.clone(),
            total_vouchers: app_config.no_of_vouchers,
            saved_vouchers: 0,
        }
    }

    async fn insert_many(&mut self, vouchers: Vec<Voucher>) -> Vec<Voucher> {
        let client = &mut self.db.client;
        let mut pins = Vec::new();
        let mut serials = Vec::new();

        for item in &vouchers {
            let (pin, serial) = item.get();

            pins.push(pin);
            serials.push(serial);
        }

        let query = format!(
            "INSERT INTO public.vouchers (pin, serial, {group_col_name})
            SELECT pin, serial, $3::uuid
            FROM UNNEST($1::text[], $2::text[]) AS v(pin, serial)
            ON CONFLICT DO NOTHING
            RETURNING pin, serial;",
            group_col_name = self.group_col_name
        );

        let query = query.as_str();
        let rows_affected = client.query(query, &[&pins, &serials, &self.group]).await;

        if let Err(e) = &rows_affected {
            println!("query error: {:?}", e)
        }

        let mut vouchers = Vec::new();

        if let Ok(rows) = rows_affected {
            for row in rows {
                vouchers.push(Voucher::new(row.get(0), row.get(1)));
            }
        }

        vouchers
    }

    async fn write(&mut self, batch: Vec<Voucher>) {
        let saved_vouchers = self.insert_many(batch).await;
        let total_vouchers = saved_vouchers.len();

        self.saved_vouchers += total_vouchers;
    }
}

impl VoucherStore for DbStore {
    fn start(
        store: Arc<Mutex<Self>>,
        generator: mpsc::Sender<GeneratorAction>,
    ) -> (mpsc::Sender<StoreAction>, JoinHandle<()>) {
        let (send, mut rx) = mpsc::channel(1_000_000);

        let stream = tokio::spawn(async move {
            let db_batch_size = store.lock().await.db.batch_size;
            let total_vouchers = store.lock().await.total_vouchers;

            loop {
                let mut vouchers = vec![];
                let mut save_voucher_futures = vec![];

                while let Some(action) = rx.recv().await {
                    match action {
                        StoreAction::Save(voucher) => {
                            vouchers.push(voucher);
                        }
                        StoreAction::Stop => {
                            break;
                        }
                    }

                    if vouchers.len() >= db_batch_size {
                        let save_store = store.clone();
                        save_voucher_futures.push(tokio::spawn(async move {
                            save_store.lock().await.write(vouchers).await;
                        }));

                        vouchers = vec![];
                    }
                }

                if vouchers.len() > 0 {
                    let save_store = store.clone();
                    save_voucher_futures.push(tokio::spawn(async move {
                        save_store.lock().await.write(vouchers).await;
                    }));
                }

                // ensure all promises are resolved
                for save_voucher_future in save_voucher_futures {
                    save_voucher_future.await.unwrap();
                }

                let saved_vouchers = store.lock().await.saved_vouchers;
                let to_generate = total_vouchers - saved_vouchers;

                if to_generate != 0 {
                    let _ = generator.send(GeneratorAction::Generate(to_generate)).await;
                    continue;
                }

                println!("completed: saved {} vouchers", saved_vouchers);
                break;
            }
        });

        (send, stream)
    }
}
