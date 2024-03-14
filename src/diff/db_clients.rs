use sqlx::{Pool, Postgres};

/// This structure is holding 2 Postgres DB pools.
/// These will be used to query both the source and the destination databases.
pub struct DBClients {
    first_db_pool: Pool<Postgres>,
    second_db_pool: Pool<Postgres>,
}

impl DBClients {
    pub fn new(first_db_pool: Pool<Postgres>, second_db_pool: Pool<Postgres>) -> Self {
        Self {
            first_db_pool,
            second_db_pool,
        }
    }

    pub fn first_db_pool(&self) -> Pool<Postgres> {
        self.first_db_pool.clone()
    }

    pub fn second_db_pool(&self) -> Pool<Postgres> {
        self.second_db_pool.clone()
    }
}
