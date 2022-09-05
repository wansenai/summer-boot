use serde::Deserialize;
use summer_boot::{Request, Result};

#[derive(Debug, Deserialize)]
struct User {
    name: String,
    age: u16,
}

#[summer_boot::auto_scan]
#[summer_boot::main]
async fn main() {
    summer_boot::run();
}

#[summer_boot::post("/test/api")]
async fn test_api(mut req: Request<()>) -> Result {
    let User { name, age } = req.body_json().await?;
    Ok(format!("Hello, {}!  {} years old", name, age).into())
}
