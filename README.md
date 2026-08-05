# generate-vouchers

A CLI tool that generates unique voucher `pin` + `serial` pairs and writes them out in
batched CSV files (with optional Postgres persistence).

- **Pin**: random string drawn from `ABCDEFGHJKLMNPQRSTUVWXYZ23456789` (ambiguous
  characters `I`, `O`, `0`, `1` are excluded). Each pin is required to contain at
  least one letter and one digit, and rejects immediately-repeated characters.
- **Serial**: random 20-digit numeric string.

Pins and serials are deduplicated in memory before being written out, so every
generated voucher is unique.

## Requirements

- Rust (2024 edition) / Cargo
- OpenSSL is vendored via the `openssl` crate, so no system OpenSSL install is required
- Optional: a Postgres database if you want to persist vouchers via `DATABASE_URL`

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release -- [OPTIONS]
```

or, after building:

```bash
./target/release/generate-vouchers [OPTIONS]
```

### Example

Generate 100,000 vouchers, in batches of 10,000, written to `./out/`:

```bash
cargo run --release -- -v 100000 -b 10000 -o out/vouchers
```

This produces `out/vouchers-1.csv`, `out/vouchers-2.csv`, … each containing a
`pin, serial` header followed by up to `--batch` rows.

## CLI options

| Flag | Alias | Description | Default |
|---|---|---|---|
| `--vouchers <n>` | `-v` | Number of vouchers to generate | `1` |
| `--pin-size <n>` | | Pin length | `6` |
| `--group <uuid>` | `-g` | Group ID attached to generated vouchers | random UUID v4 |
| `--batch <n>` | `-b` | Number of vouchers per output batch (drives both output-file rotation and progress logging) | `1000000` |
| `--batch-start <n>` | | Starting batch number, used to number/resume output files | `1` |
| `--output <path>` | `-o` | Output file prefix. Files are written as `<prefix>-<batch>.csv` | `.codes/<group_id>` |
| `--db-url <url>` | | Postgres connection string used to persist vouchers | none (falls back to `DATABASE_URL` env var) |
| `--db-cert <path>` | | Path to a CA cert file, enables TLS when connecting to the database | none (connects without TLS) |
| `--db-batch <n>` | | Batch size for database insert statements | `10000` |
| `--db-group-column <name>` | | Name of the group-id column in the `vouchers` table | `group_id` |

Unrecognized flags are silently ignored.

### Database connection

`DATABASE_URL` can also be set via a `.env` file in the project root (loaded
automatically on startup via `dotenvy`):

```
DATABASE_URL="postgresql://user:password@host:5432/dbname"
```

The `DATABASE_URL` env var takes precedence over `--db-url` when both are present.

## Output

Each output CSV has the header `pin, serial` followed by one row per voucher:

```
pin, serial
AB23CD45, 00000000000000000001
...
```

Files roll over to a new batch (`<prefix>-<batch+1>.csv`) every `--batch` rows.

## Known limitations (current branch: `feat/async-implementation`)

- The Postgres store (`DbStore`) and Redis store (`store/redis.rs`) exist in the
  codebase but are not currently wired up in `main.rs` — only the in-memory store
  (`MemoryStore`) is active, meaning vouchers are only ever written to CSV, not to
  a database, when you run the binary as-is.
