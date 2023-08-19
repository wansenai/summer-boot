<h1 align="center">Summer Boot</h1>
<div align="center">
 <strong>
  A web framework for Rust
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/summer-boot">
    <img src="https://img.shields.io/crates/v/summer-boot.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/summer-boot">
    <img src="https://img.shields.io/crates/d/summer-boot.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- bors -->
  <a href="https://app.bors.tech/repositories/45710">
    <img src="https://bors.tech/images/badge_small.svg"
      alt="ors enabled" />
  </a>
  <!-- fossa status -->
  <a href="https://app.fossa.com/projects/git%2Bgithub.com%2Fsummer-os%2Fsummer-boot?ref=badge_shield">
    <img src="https://app.fossa.com/api/projects/git%2Bgithub.com%2Fsummer-os%2Fsummer-boot.svg?type=shield"
      alt="ors enabled" />
  </a>
</div>

<br />

------
Encapsulating tide, combined with the design principles of spring boot.
will simplify web development and enable developers to focus more on business API development.

```rust
summer_boot::log Logger started
summer_boot::log 
    _____                                       ____              _   
   / ____|                                     |  _ \            | |  
  | (___  _   _ _ __ ___  _ __ ___   ___ _ __  | |_) | ___   ___ | |_ 
   \___ \| | | | '_ ` _ \| '_ ` _ \ / _ \ '__| |  _ < / _ \ / _ \| __|
   ____) | |_| | | | | | | | | | | |  __/ |    | |_) | (_) | (_) | |_ 
  |_____/ \__,_|_| |_| |_|_| |_| |_|\___|_|    |____/ \___/ \___/ \__|
                                                                      
  :: Summer Boot Version::             (1.0.3)                                                                    
 
summer_boot::server::server Server listening on http://127.0.0.1:8080
```

## Quick Start

Cargo.toml:
```rust
summer-boot = "1.0.3"
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
  port: 8080
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
