mod http_method;

#[summer_boot::auto_scan]
#[summer_boot::main]
async fn main() {
    summer_boot::run();
}