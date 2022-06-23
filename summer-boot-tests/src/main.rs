use serde::Deserialize;
use serde_json::Value;
use summer_boot::{Request, Result};

#[derive(Debug, Deserialize)]
struct User {
    name: String,
    age: u16,
}

#[summer_boot::main]
async fn main() {
  let mut Application = summer_boot::run();

  // auto scan 编译前 注入
  Application.at("/test/api").get(test_api);
  

  // 这个后面有 run方法自己走  试试放main宏里
  Application.listen("127.0.0.1:8080").await.unwrap();

}

#[summer_boot::post("/test/api")]
async fn test_api(mut req: Request<()>) -> Result {
    let User { name, age } = req.body_json().await?;
    Ok(format!("Hello, {}!  {} years old", name, age).into())
}
