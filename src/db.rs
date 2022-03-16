use rusty_leveldb::DB as LevelDB;
use std::env;
use std::fs;
use std::sync::Mutex;
// use tokio::sync::Mutex;

pub struct Database(Mutex<LevelDB>);
impl Database {
    pub fn open(name: &str) -> Self {
        let mut path = env::current_exe().unwrap().with_file_name("db");
        fs::create_dir_all(&path).unwrap();
        path.push(name);
        let db = LevelDB::open(path, rusty_leveldb::Options::default()).unwrap();
        Self(Mutex::new(db))
    }
    pub fn get(&self, key: &str) -> Option<String> {
        let mut db = self.0.lock().unwrap();
        db.get(key.as_bytes())
            .map(|s| String::from_utf8_lossy(&s).into())
    }
    pub fn put(&self, key: &str, value: &str) {
        let mut db = self.0.lock().unwrap();
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }
    pub fn flush(&self) {
        self.0.lock().unwrap().flush().unwrap();
    }
}
