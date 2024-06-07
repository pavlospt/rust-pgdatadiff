use anyhow::Result;

#[cfg(not(feature = "with-clap"))]
use inquire::{Confirm, Text};

#[cfg(feature = "with-clap")]
use clap::{Parser, Subcommand};
use rust_pgdatadiff::diff::diff_ops::Differ;
use rust_pgdatadiff::diff::diff_payload::DiffPayload;

#[cfg(feature = "with-clap")]
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "with-clap")]
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
        /// The start position for the comparison
        #[arg(long, default_value_t = 0, required = false)]
        start_position: i64,
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

#[cfg(feature = "with-clap")]
async fn main_clap() -> Result<()> {
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
            start_position,
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
                *start_position,
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

#[cfg(not(feature = "with-clap"))]
async fn main_inquire() -> Result<()> {
    let first_db = Text::new("First DB")
        .with_default("postgres://postgres:postgres@localhost:5438/example")
        .with_help_message("Enter the first database connection string")
        .prompt()?;
    let second_db = Text::new("Second DB")
        .with_default("postgres://postgres:postgres@localhost:5439/example")
        .with_help_message("Enter the first database connection string")
        .prompt()?;
    let only_tables = Confirm::new("Do you want to only compare tables?")
        .with_default(false)
        .with_help_message("By confirming this option, you will only compare tables")
        .prompt()?;
    let only_sequences = Confirm::new("Do you want to only compare sequences?")
        .with_default(false)
        .with_help_message("By confirming this option, you will only compare sequences")
        .prompt()?;
    let only_count = Confirm::new("Do you want to only count rows of tables?")
        .with_default(false)
        .with_help_message("By confirming this option, you will only row counts of tables")
        .prompt()?;
    let chunk_size = Text::new("Number of rows to compare (in batch)")
        .with_default("10000")
        .with_help_message("Enter the chunk size when comparing data")
        .prompt()?;
    let start_position = Text::new("Start position for the comparison")
        .with_default("0")
        .with_help_message("Enter the start position for the comparison")
        .prompt()?;
    let max_connections = Text::new("Number of DB connections to utilize")
        .with_default("100")
        .with_help_message("Enter the max connections for Postgres pool")
        .prompt()?;
    let include_tables = Text::new("Tables to include in the comparison")
        .with_default("")
        .with_help_message("Enter the tables to include in the comparison (comma separated)")
        .prompt()?;
    let exclude_tables = Text::new("Tables to exclude from the comparison")
        .with_default("")
        .with_help_message("Enter the tables to exclude from the comparison (comma separated)")
        .prompt()?;
    let schema_name = Text::new("DB schema name to compare")
        .with_default("public")
        .with_help_message("Enter the DB schema name to perform the comparison on")
        .prompt()?;

    let payload = DiffPayload::new(
        first_db,
        second_db,
        only_tables,
        only_sequences,
        only_count,
        chunk_size.parse::<i64>().unwrap(),
        start_position.parse::<i64>().unwrap(),
        max_connections.parse::<i64>().unwrap(),
        include_tables
            .split_whitespace()
            .flat_map(|t| t.split(','))
            .collect(),
        exclude_tables
            .split_whitespace()
            .flat_map(|t| t.split(','))
            .collect(),
        schema_name,
    );
    let _ = Differ::diff_dbs(payload).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    #[cfg(feature = "with-clap")]
    {
        _ = main_clap().await;
    }
    #[cfg(not(feature = "with-clap"))]
    {
        _ = main_inquire().await;
    }

    Ok(())
}
