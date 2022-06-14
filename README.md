# Summer Boot

**(We ready to face the cruelty reality and long time , and it is we team dream to develop it)**

The next generation decentralized web framework allows users to manage and share their own data. 

It will be a wide area and cross regional web framework.


## Future API Example Show

```rust
use serde::Deserialize;
use summer_boot::{Request, Result};

#[derive(Debug, Deserialize)]
struct User {
    name: String,
    age: u16,
}

#[summer_boot::main]
async fn main() {
    let mut app = summer_boot::new();
    app.at("/test/api").post(test_api);
    app.listen("127.0.0.1:8080").await.unwrap();
}

async fn test_api(mut req: Request<()>) -> Result {
    let User { name, age } = req.body_json().await?;
    Ok(format!("Hello, {}!  {} years old", name, age).into())
}
```


```rust
#[web3]
#[SummerBootApplication]
async fn main() {
    SummerApplication::run();
}
```

```rust
#[web3]
#[SummerBootApplication]
async fn main() {
    SummerApplication::run();
}
```

```rust
#[ResutController]
#[RequestMapping("api")]
async trait Api{
    // do something
}
```
