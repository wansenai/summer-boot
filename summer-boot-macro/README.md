# Summer Boot Macro

Used to write all macros of summer boot

## Getting Started

```rust
summer-boot-macro = "1.4.1"
```

## Macro description

### 1. Macro attribute summer_boot::main

This macro is mainly used to start asynchronous methods and create a new instance of summer boot 

```rust
#[summer_boot::main]
async fn main() {
    async { println!("Hello world"); }.await
}
```

### 2. Macro attribute summer_boot::auto_scan

This macro is mainly used to automatically scan the workspace or under a single project API, 
and to automatically complete the scanning of YML configuration files under the resource directory

```rust
#[summer_boot::auto_scan]
async fn main() {
    summer_boot::run();
}
```
