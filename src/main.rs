mod config;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    config::parse_config()
    let _rocket = rocket::build().mount("/", routes![index]).launch().await?;

    Ok(())
}
