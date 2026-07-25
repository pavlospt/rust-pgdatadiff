use deadpool_postgres::{Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod, Runtime};
use deadpool_postgres::tokio_postgres::NoTls;
use rust_pgdatadiff::diff::diff_ops::Differ;
use rust_pgdatadiff::diff::diff_output::DiffOutput;
use rust_pgdatadiff::diff::diff_payload::DiffPayload;
use std::time::Instant;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, ContainerRequest, GenericImage, ImageExt,
};

const SETUP_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS product (
    product_id INT NOT NULL,
    name varchar(250) NOT NULL,
    PRIMARY KEY (product_id)
);
CREATE TABLE IF NOT EXISTS users (
    user_id INT NOT NULL,
    name varchar(250) NOT NULL,
    PRIMARY KEY (user_id)
);
CREATE TABLE IF NOT EXISTS country (
    country_id INT NOT NULL,
    country_name varchar(450) NOT NULL,
    PRIMARY KEY (country_id)
);
CREATE TABLE IF NOT EXISTS city (
    city_id INT NOT NULL,
    city_name varchar(450) NOT NULL,
    country_id INT NOT NULL,
    PRIMARY KEY (city_id),
    CONSTRAINT fk_country FOREIGN KEY (country_id) REFERENCES country(country_id)
);
CREATE SEQUENCE product_seq START WITH 1 INCREMENT BY 1;
CREATE SEQUENCE user_seq START WITH 1 INCREMENT BY 1;
CREATE SEQUENCE country_seq START WITH 1 INCREMENT BY 1;
"#;

const SEED_SQL: &str = r#"
INSERT INTO product SELECT id, concat('Product ', id) FROM GENERATE_SERIES(1, 100) as id;
INSERT INTO users SELECT id, concat('User ', id) FROM GENERATE_SERIES(1, 200) as id;
INSERT INTO country SELECT id, concat('Country ', id) FROM GENERATE_SERIES(1, 10) as id;
INSERT INTO city SELECT id, concat('City ', id), floor(random() * 10 + 1)::int FROM GENERATE_SERIES(1, 50) as id;
"#;

const DIFFERENCE_SQL: &str = r#"
INSERT INTO product VALUES (999, 'Extra product only in second DB');
DELETE FROM users WHERE user_id = 50;
"#;

fn postgres_image() -> ContainerRequest<GenericImage> {
    GenericImage::new("postgres", "17")
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_PASSWORD", "postgres")
}

async fn create_pool(port: u16) -> Pool {
    let url = format!("postgres://postgres:postgres@localhost:{port}/postgres");
    let mut cfg = Config::new();
    cfg.url = Some(url);
    cfg.application_name = Some("rust-pgdatadiff-test".into());
    cfg.pool = Some(PoolConfig::new(4));
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}

async fn setup_schema(pool: &Pool, sql: &str) {
    let client = pool.get().await.unwrap();
    client.batch_execute(sql).await.unwrap();
}

async fn start_container(
    image: ContainerRequest<GenericImage>,
) -> Option<ContainerAsync<GenericImage>> {
    match image.start().await {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Skipping integration test: Docker not available ({e})");
            None
        }
    }
}

async fn get_port(container: &ContainerAsync<GenericImage>) -> u16 {
    container.get_host_port_ipv4(5432).await.unwrap()
}

#[tokio::test]
async fn integration_diff_detects_row_count_differences() {
    let start = Instant::now();
    let (container1, container2) = tokio::join!(
        start_container(postgres_image()),
        start_container(postgres_image()),
    );

    let (Some(container1), Some(container2)) = (container1, container2) else {
        return;
    };

    let db1_port = get_port(&container1).await;
    let db2_port = get_port(&container2).await;
    let container_setup_ms = start.elapsed().as_millis();
    eprintln!("Containers started in {container_setup_ms}ms (db1:{db1_port}, db2:{db2_port})");

    let pool1 = create_pool(db1_port).await;
    let pool2 = create_pool(db2_port).await;

    setup_schema(&pool1, SETUP_SQL).await;
    setup_schema(&pool1, SEED_SQL).await;
    setup_schema(&pool2, SETUP_SQL).await;
    setup_schema(&pool2, SEED_SQL).await;

    // Introduce a difference in DB2: extra product + deleted user
    setup_schema(&pool2, DIFFERENCE_SQL).await;

    let db1_url = format!("postgres://postgres:postgres@localhost:{db1_port}/postgres");
    let db2_url = format!("postgres://postgres:postgres@localhost:{db2_port}/postgres");

    let payload = DiffPayload::builder()
        .first_db(&db1_url)
        .second_db(&db2_url)
        .only_tables(true)
        .only_sequences(false)
        .only_count(false)
        .chunk_size(50)
        .start_position(0)
        .max_connections(4)
        .include_tables(Vec::<String>::new())
        .exclude_tables(Vec::<String>::new())
        .schema_name("public")
        .accept_invalid_certs_first_db(false)
        .accept_invalid_certs_second_db(false)
        .build();

    let diff_start = Instant::now();
    let results = Differ::diff_dbs(payload).await.unwrap();
    let diff_ms = diff_start.elapsed().as_millis();
    eprintln!("Diff completed in {diff_ms}ms");

    let mut product_diff_found = false;
    let mut user_diff_found = false;

    for result in &results {
        if let DiffOutput::TableDiff(table_output) = result {
            let s = table_output.to_string().to_string();
            eprintln!("  {s}");
            if s.contains("product") && s.contains("100") && s.contains("101") {
                product_diff_found = true;
            }
            if s.contains("users") && s.contains("200") && s.contains("199") {
                user_diff_found = true;
            }
        }
    }

    assert!(
        product_diff_found,
        "Expected product row count diff (100 vs 101)"
    );
    assert!(
        user_diff_found,
        "Expected users row count diff (200 vs 199)"
    );
    assert!(diff_ms < 15_000, "Diff took {diff_ms}ms, expected < 15s");
}

#[tokio::test]
async fn integration_diff_identical_databases_no_diff() {
    let start = Instant::now();
    let (container1, container2) = tokio::join!(
        start_container(postgres_image()),
        start_container(postgres_image()),
    );

    let (Some(container1), Some(container2)) = (container1, container2) else {
        return;
    };

    let db1_port = get_port(&container1).await;
    let db2_port = get_port(&container2).await;
    let container_setup_ms = start.elapsed().as_millis();
    eprintln!("Containers started in {container_setup_ms}ms (db1:{db1_port}, db2:{db2_port})");

    let pool1 = create_pool(db1_port).await;
    let pool2 = create_pool(db2_port).await;

    setup_schema(&pool1, SETUP_SQL).await;
    setup_schema(&pool1, SEED_SQL).await;
    setup_schema(&pool2, SETUP_SQL).await;
    setup_schema(&pool2, SEED_SQL).await;

    let db1_url = format!("postgres://postgres:postgres@localhost:{db1_port}/postgres");
    let db2_url = format!("postgres://postgres:postgres@localhost:{db2_port}/postgres");

    let payload = DiffPayload::builder()
        .first_db(&db1_url)
        .second_db(&db2_url)
        .only_tables(true)
        .only_sequences(false)
        .only_count(false)
        .chunk_size(50)
        .start_position(0)
        .max_connections(4)
        .include_tables(Vec::<String>::new())
        .exclude_tables(Vec::<String>::new())
        .schema_name("public")
        .accept_invalid_certs_first_db(false)
        .accept_invalid_certs_second_db(false)
        .build();

    let diff_start = Instant::now();
    let results = Differ::diff_dbs(payload).await.unwrap();
    let diff_ms = diff_start.elapsed().as_millis();
    eprintln!("Diff completed in {diff_ms}ms");

    let mut diff_count = 0usize;
    for result in &results {
        if let DiffOutput::TableDiff(table_output) = result {
            let s = table_output.to_string().to_string();
            eprintln!("  {s}");
            if s.contains("difference") || s.contains("not exist") {
                diff_count += 1;
            }
        }
    }

    assert_eq!(diff_count, 0, "Expected no differences but found {diff_count}");
    assert!(diff_ms < 15_000, "Diff took {diff_ms}ms, expected < 15s");
}
