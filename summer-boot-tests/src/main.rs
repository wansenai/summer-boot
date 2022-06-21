use serde::Deserialize;
use summer_boot::{Request, Result};
use summer_boot::log;

#[derive(Debug, Deserialize)]
struct User {
    name: String,
    age: u16,
}

#[summer_boot::auto_scan]
#[summer_boot::main]
async fn main() {
    log::start();
    let mut app = summer_boot::new();
    app.listen("127.0.0.1:8080").await.unwrap();
    log::info!("请求完成");
}

#[summer_boot::post("/test/api")]
async fn test_api(mut req: Request<()>) -> Result {
    let User { name, age } = req.body_json().await?;
    Ok(format!("Hello, {}!  {} years old", name, age).into())
}
