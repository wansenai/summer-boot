use std::{
    fs::{self, File},
    io::{BufReader, Read},
};

#[test]
fn test() {
    //  let file = File::open("example/src/resources").expect("找不到文件");

    let file = File::open("example/src/resources/").is_ok();

    let file_name = "application.yml";
    let directories = vec!["src/resources", "spring-boot/src/resources", "dir3"];

    for directory in directories {
        let path = format!("{}/{}", directory, file_name);
        let file_exists = fs::metadata(&path).is_ok();

        if file_exists {
            println!("文件 {} 存在于目录 {}", file_name, directory);
        } else {
            println!("文件 {} 不存在于目录 {}", file_name, directory);
        }
    }
}
