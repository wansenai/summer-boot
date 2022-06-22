use serde::Deserialize;
use summer_boot::{Request, Result};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct User {
    name: String,
    age: u16,
}

#[summer_boot::main]
async fn main() {
    summer_boot::run().await.unwrap();
}

#[summer_boot::post("/test/api")]
async fn test_api(mut req: Request<()>) -> Result {
    let User { name, age } = req.body_json().await?;
    Ok(format!("Hello, {}!  {} years old", name, age).into())
}