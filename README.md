# Summer Boot

<a href="https://app.bors.tech/repositories/45710"><img src="https://bors.tech/images/badge_small.svg" alt="Bors enabled"></a>
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fsummer-os%2Fsummer-boot.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fsummer-os%2Fsummer-boot?ref=badge_shield)

The next generation decentralized web framework allows users to manage and share their own data. 
It will be a wide area and cross regional web framework.

```rust
summer_boot::log Logger started
summer_boot::log 
    _____                                       ____              _   
   / ____|                                     |  _ \            | |  
  | (___  _   _ _ __ ___  _ __ ___   ___ _ __  | |_) | ___   ___ | |_ 
   \___ \| | | | '_ ` _ \| '_ ` _ \ / _ \ '__| |  _ < / _ \ / _ \| __|
   ____) | |_| | | | | | | | | | | |  __/ |    | |_) | (_) | (_) | |_ 
  |_____/ \__,_|_| |_| |_|_| |_| |_|\___|_|    |____/ \___/ \___/ \__|
                                                                      
  :: Summer Boot Version::             (0.1.0)                                                                    
 
summer_boot::web2::server::server Server listening on http://127.0.0.1:8080
```

## Quick Start

Cargo.toml:
```rust
summer-boot = "0.1.3"
```

Add resuorce configuration file to src directory

src/resources/application.yml
```yml
profiles:
  active: test

```
src/resources/application-test.yml
```yml
server:
  port: 7798
  context_path: /
```

src/main.rs
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
    summer_boot::run();
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
