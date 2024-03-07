use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_pgdatadiff::diff::diff_ops::Differ;
use rust_pgdatadiff::diff::diff_payload::DiffPayload;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Print the version")]
    Version,
    Diff {
        /// postgres://postgres:postgres@localhost:5438/example
        first_db: String,
        /// postgres://postgres:postgres@localhost:5439/example
        second_db: String,
        /// Only compare data, exclude sequences
        #[arg(long, default_value_t = false, required = false)]
        only_tables: bool,
        /// Only compare sequences, exclude data
        #[arg(long, default_value_t = false, required = false)]
        only_sequences: bool,
        /// Do a quick test based on counts alone
        #[arg(long, default_value_t = false, required = false)]
        only_count: bool,
        /// The chunk size when comparing data
        #[arg(long, default_value_t = 10000, required = false)]
        chunk_size: i64,
        /// Max connections for Postgres pool
        #[arg(long, default_value_t = 100, required = false)]
        max_connections: i64,
        /// Tables included in the comparison
        #[arg(short, long, value_delimiter = ',', num_args = 0.., required = false, conflicts_with = "exclude_tables")]
        include_tables: Vec<String>,
        /// Tables excluded from the comparison
        #[arg(short, long, value_delimiter = ',', num_args = 0.., required = false, conflicts_with = "include_tables")]
        exclude_tables: Vec<String>,
        /// Schema name
        #[arg(long, default_value = "public", required = false)]
        schema_name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Version => {
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Diff {
            first_db,
            second_db,
            only_tables,
            only_sequences,
            only_count,
            chunk_size,
            max_connections,
            include_tables,
            exclude_tables,
            schema_name,
        } => {
            let payload = DiffPayload::new(
                first_db.clone(),
                second_db.clone(),
                *only_tables,
                *only_sequences,
                *only_count,
                *chunk_size,
                *max_connections,
                include_tables.to_vec(),
                exclude_tables.to_vec(),
                schema_name.clone(),
            );
            let _ = Differ::diff_dbs(payload).await;
            Ok(())
        }
    }
}
