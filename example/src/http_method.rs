use serde::Deserialize;
use summer_boot::{Request, Result};

#[derive(Debug, Deserialize)]       
struct User {
    name: String,
    age: u16,
}

#[summer_boot::get("/hello")]
pub async fn hello(mut _req: Request<()>) -> Result {
    Ok(format!("Hello, Summer Boot").into())
}

#[summer_boot::post("/user/getUserInfo")]
pub async fn test_api(mut req: Request<()>) -> Result {
    let User { name, age } = req.body_json().await?;
    Ok(format!("Hello, {}!  {} years old", name, age).into())
}