# Rust PGDataDiff

`rust-pgdatadiff` is a re-write of the Python version of [pgdatadiff](https://github.com/dmarkey/pgdatadiff)

## What makes it different?

* It is schema aware right from the get-go, as when we had to use the original
  `pgdatadiff` we ended up having different schemas that we needed to perform checks on.

* It runs DB operations in a parallel fashion,
  making it at least 3x faster in comparison to the original `pgdatadiff` which performs the checks sequentially.

* It is written in Rust, which means that it is memory safe and has a very low overhead.

* It provides both a library and a client, which means that you can use it as a standalone tool
  and in your own projects.

_The benchmarks below are based on DBs with 5 tables and 1M rows each. The results are as follows:_

## Python (sequential)
![python-timings](images/python.png)

## Rust (parallel)
![rust-timings](images/rust.png)

# Installation (Client)

In case you want to use this as a client you can install it through `cargo`:

```shell
cargo install rust-pgdatadiff-client
```

# Installation (Library)

In case you want to use this as a library you can add it to your `Cargo.toml`:

```shell
cargo add rust-pgdatadiff
```

or

```toml
[dependencies]
rust-pgdatadiff = "0.1.0"
```

# Usage (Client)

```
Usage: rust-pgdatadiff-client diff [OPTIONS] <FIRST_DB> <SECOND_DB>

Arguments:
  <FIRST_DB>   postgres://postgres:postgres@localhost:5438/example
  <SECOND_DB>  postgres://postgres:postgres@localhost:5439/example

Options:
      --only-tables                           Only compare data, exclude sequences
      --only-sequences                        Only compare sequences, exclude data
      --only-count                            Do a quick test based on counts alone
      --chunk-size <CHUNK_SIZE>               The chunk size when comparing data [default: 10000]
      --max-connections <MAX_CONNECTIONS>     Max connections for Postgres pool [default: 100]
  -i, --include-tables [<INCLUDE_TABLES>...]  Tables included in the comparison
  -e, --exclude-tables [<EXCLUDE_TABLES>...]  Tables excluded from the comparison
      --schema-name <SCHEMA_NAME>             Schema name [default: public]
  -h, --help                                  Print help
  -V, --version                               Print version
```

# Usage (Library)

```rust
use rust_pgdatadiff::diff::diff_ops::Differ;
use rust_pgdatadiff::diff::diff_payload::DiffPayload;

#[tokio::main]
async fn main() -> Result<()> {
  let first_db = "postgres://postgres:postgres@localhost:5438/example";
  let second_db = "postgres://postgres:postgres@localhost:5439/example";
  
  let payload = DiffPayload::new(
    first_db.to_owned(),
    second_db.to_owned(),
    *only_data,
    *only_sequences,
    *only_count,
    *chunk_size,
    *max_connections,
    included_tables.to_vec(),
    schema_name.clone(),
  );
  let diff_result = Differ::diff_dbs(payload).await;
  // Handle `diff_result` in any way it fits your use case
  Ok(())
}
```

# Examples

You can spin up two databases already prefilled with data through Docker Compose.

```shell
docker compose up --build
```

Prefilled databases include a considerable amount of data + rows, so you can run benchmarks against them to check the
performance of it. You can modify a few of the generated data in order to see it in action.

You can find an example of using it as a library in the [`examples`](./examples) directory.

Run the example with the following command, after Docker Compose has started:

```shell
cargo run --example example_diff diff \
  "postgresql://localhost:5438?dbname=example&user=postgres&password=postgres" \
  "postgresql://localhost:5439?dbname=example&user=postgres&password=postgres"
```

You can also enable Rust related logs by exporting the following:

```shell
export RUST_LOG=rust_pgdatadiff=info
```

Switching from `info` to `debug` will give you more detailed logs. Also since we are utilizing
`sqlx` under the hood, you can enable `sqlx` logs by exporting the following:

```shell
export RUST_LOG=rust_pgdatadiff=info,sqlx=debug
```

# Authors

* [Pavlos-Petros Tournaris](https://github.com/pavlospt)
* [Nikolaos Nikitas](https://github.com/nikoshet)
