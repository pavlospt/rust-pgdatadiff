use deadpool_postgres::Pool;

/// This structure is holding 2 Postgres DB pools.
/// These will be used to query both the source and the destination databases.
pub struct DBClients {
    first_db_pool: Pool,
    second_db_pool: Pool,
}

impl DBClients {
    pub fn new(first_db_pool: Pool, second_db_pool: Pool) -> Self {
        Self {
            first_db_pool,
            second_db_pool,
        }
    }

    pub fn first_db_pool(&self) -> Pool {
        self.first_db_pool.clone()
    }

    pub fn second_db_pool(&self) -> Pool {
        self.second_db_pool.clone()
    }
}
