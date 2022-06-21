# Summer Boot

<a href="https://app.bors.tech/repositories/45710"><img src="https://bors.tech/images/badge_small.svg" alt="Bors enabled"></a>
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fsummer-os%2Fsummer-boot.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fsummer-os%2Fsummer-boot?ref=badge_shield)

The next generation decentralized web framework allows users to manage and share their own data. 
It will be a wide area and cross regional web framework.


## Quick Start

```rust
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
}

#[summer_boot::post("/test/api")]
async fn test_api(mut req: Request<()>) -> Result {
    let User { name, age } = req.body_json().await?;
    Ok(format!("Hello, {}!  {} years old", name, age).into())
}
```

## License

Licensed under either of

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fsummer-os%2Fsummer-boot.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fsummer-os%2Fsummer-boot?ref=badge_large)
