#[summer_boot_macro::main]
async fn main() {
    async { println!("Hello world"); }.await
}