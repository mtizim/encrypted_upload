use std::path::PathBuf;

#[derive(Clone)]
pub struct AppConfig {
    pub file_dir: PathBuf,
    pub max_file_size: u64,
}

impl AppConfig {
    pub fn new() -> AppConfig {
        let file_dir = PathBuf::from(match std::env::var("FILE_DIR") {
            Ok(val) => val,
            Err(_) => String::from("./files/"),
        })
        .canonicalize()
        .expect("Invalid path");

        if file_dir.is_file() {
            panic!("Expected dir, not file");
        }

        if !file_dir.exists() {
            panic!("Invalid path - dir does not exist")
        }

        let max_file_size = match std::env::var("MAX_FILE_SIZE") {
            Ok(val) => val.parse::<u64>().or(Err(())),
            Err(_) => Err(()),
        }
        .unwrap_or((1 << 20) * 100);
        AppConfig {
            file_dir,
            max_file_size,
        }
    }
}
