use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use serde_json::{from_str as json_from_str, to_string_pretty};
use serde_yaml::from_str as yaml_from_str;
use std::path::Path;
use std::{
    fs::{self, read_to_string},
    io::Read,
    ops::Add,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalConfig {
    pub mysql: Option<Mysql>,
    pub server: Option<Server>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mysql {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
    pub db: String,
    pub pool_min_idle: u64,
    pub pool_max_open: u64,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub port: u32,
    pub context_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct Profiles {
    pub active: String,
}

#[derive(Serialize, Deserialize)]
pub struct EnvConfig {
    pub profiles: Profiles,
}

#[derive(Debug, Deserialize)]
struct ConfWorkSpace {
    workspace: Option<Member>,
    package: Option<Name>,
}

/// 匹配workspace下的member数组格式
#[derive(Debug, Deserialize)]
struct Member {
    members: Option<Vec<String>>,
}

/// 匹配package下的name字段
#[derive(Debug, Deserialize)]
struct Name {
    name: String,
}

///
/// 判断是workspace还是project
///
fn check_project_workspace() -> String {
    let mut types: String = String::new();

    // 找到需要扫描的路径
    let mut cargo_toml = fs::File::open("Cargo.toml").expect("2222");
    let mut content = String::new();
    cargo_toml.read_to_string(&mut content).expect("3333");

    // 根据包类型分别处理
    if let Ok(conf_work_space) = toml::from_str::<ConfWorkSpace>(&content) {
        if let Some(workspace) = conf_work_space.workspace {
            types.push_str("workspace");
        } else {
            types.push_str("project");
        }
    }
    types
}

///
/// 获取toml package_name
///
fn get_package_name() -> String {
    let mut cargo_toml = fs::File::open("Cargo.toml").expect("4444");
    let mut content = String::new();
    cargo_toml.read_to_string(&mut content).expect("5555");

    let mut projects = Vec::<String>::new();

    if let Ok(conf_work_space) = toml::from_str::<ConfWorkSpace>(&content) {
        if let Some(workspace) = conf_work_space.workspace {
            if let Some(members) = workspace.members {
                for member in members {
                    projects.push(format!("{}/src/resources/application.yml", member));
                    for project in &projects {
                        let check = fs::metadata(project).is_ok();
                        if check == true {
                            return member;
                        }
                    }
                }
            }
        } else if projects.len() == 0 {
            if let Some(package) = conf_work_space.package {
                return package.name;
            }
        }
    }
    // report error
    String::from("_")
}

///
/// 加载环境配置
///
pub fn load_env_conf() -> Option<EnvConfig> {
    let mut path = String::new();
    let types = check_project_workspace();

    if types.eq("workspace") {
        let package_name = get_package_name();
        path = format!("{}/src/resources/application.yml", package_name);
    } else if types.eq("project") {
        path = format!("src/resources/application.yml");
    }

    let schema = yaml_from_str::<RootSchema>(&read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "Error loading configuration file {}, please check the configuration!",
            &path
        )
    }));
    return match schema {
        Ok(json) => {
            let data =
                to_string_pretty(&json).expect("resources/application.yml file data error！");
            let p: EnvConfig =
                json_from_str(&*data).expect("Failed to transfer JSON data to EnvConfig object！");
            return Some(p);
        }
        Err(err) => {
            println!("{}", err);
            None
        }
    };
}

///
/// 根据环境配置加载全局配置
///
/// action  dev 开始环境 test 测试环境 prod 生产环境
///
pub fn load_global_config(action: String) -> Option<GlobalConfig> {
    let mut path = String::new();
    let types = check_project_workspace();

    if types.eq("workspace") {
        let package_name = get_package_name();
        path = format!("{}/src/resources/application-{}.yml", package_name, &action);
    } else if types.eq("project") {
        path = format!("src/resources/application-{}.yml", &action);
    }

    let schema = yaml_from_str::<RootSchema>(&read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "Error loading configuration file {}, please check the configuration!",
            &path
        )
    }));
    return match schema {
        Ok(json) => {
            let data = to_string_pretty(&json).unwrap_or_else(|_| {
                panic!(
                    "{} file data error！, please check the configuration!",
                    path
                )
            });
            let p = json_from_str(&*data)
                .expect("Failed to transfer JSON data to BriefProConfig object！");
            return Some(p);
        }
        Err(err) => {
            println!("{}", err);
            None
        }
    };
}

///
/// 先加载环境配置 在根据当前加载的环境 去加载相应的信息
///
pub fn load_conf() -> Option<GlobalConfig> {
    if let Some(init) = load_env_conf() {
        return load_global_config(init.profiles.active);
    }
    None
}
