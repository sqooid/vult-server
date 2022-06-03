mod config;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::parse_config::read_config("config.toml")?;
    println!("{:?}", &config);
    // let _rocket = rocket::build().mount("/", routes![index]).launch().await?;

    Ok(())
}
