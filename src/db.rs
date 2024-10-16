use tokio_postgres::{Config, Error, NoTls};

pub trait Db {
    async fn connect(&self) -> Result<(), Error>;
}

pub struct PostgresDb {
    pub host: String,
    pub port: u16, // 5432
    pub db: String,
    pub user: String,
    pub password: String,
}

impl Db for PostgresDb {
    async fn connect(&self) -> Result<(), Error> {
        println!("Connecting to the db");
        let mut config = Config::new();
        config.user(&self.user);
        config.password(&self.password);
        config.host(&self.host);
        config.port(self.port);
        config.dbname(&self.db);
        let (client, connection) = Config::connect(&config, NoTls).await?;

        Ok(())
    }
}
