//
// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     let connect_string = "host=localhost user=postgres password=password dbname=weather-bot";
//     let (client, connection) = tokio_postgres::connect(connect_string, NoTls).await?;
//
//     tokio::spawn(async move {
//         if let Err(e) = connection.await {
//             eprint!("Connection error: {}", e);
//         }
//     });
//
//     let rows = client
//         .query("SELECT login FROM Users WHERE name = $1::TEXT", &[&"Roman"])
//         .await?;
//
//     let value: &str = rows[0].get(0);
//     println!("query value: {}", value);
//     assert_eq!(value, "rastler");
//
//     Ok(())
// }
use db::{Db, PostgresDb};
use tokio_postgres::{Error, NoTls};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let postgres = PostgresDb {
        host: String::from("localhost"),
        port: 5432,
        db: String::from("weather-bot"),
        user: String::from("postgres"),
        password: String::from("password"),
    };

    postgres.connect().await?;

    Ok(())
}
