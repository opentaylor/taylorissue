use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

// ---- JsonFile ----

pub struct JsonFile {
    path: PathBuf,
}

impl JsonFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Value {
        match fs::read_to_string(&self.path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or(Value::Null),
            Err(_) => Value::Null,
        }
    }

    pub fn save(&self, data: &Value) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(data).unwrap_or_default();
        let _ = fs::write(&self.path, content);
    }

    pub fn changed(&self, data: &Value) -> bool {
        let current = self.load();
        &current != data
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

// ---- JsonlFile ----

pub struct JsonlFile {
    path: PathBuf,
}

impl JsonlFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn append(&self, data: &Value) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let line = serde_json::to_string(data).unwrap_or_default();
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .unwrap();
        writeln!(file, "{}", line).unwrap();
    }

    pub fn read_all(&self) -> Vec<Value> {
        match fs::read_to_string(&self.path) {
            Ok(content) => content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn read_tail(&self, n: usize) -> Vec<Value> {
        let all = self.read_all();
        if all.len() <= n {
            all
        } else {
            all[all.len() - n..].to_vec()
        }
    }

    pub fn clear(&self) {
        let _ = fs::write(&self.path, "");
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

// ---- DirStore ----

pub struct DirStore {
    root: PathBuf,
}

impl DirStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn ensure(&self) -> &Self {
        let _ = fs::create_dir_all(&self.root);
        self
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn list(&self) -> Vec<String> {
        match fs::read_dir(&self.root) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn read(&self, name: &str) -> Option<String> {
        fs::read_to_string(self.root.join(name)).ok()
    }

    pub fn write(&self, name: &str, content: &str) {
        self.ensure();
        let _ = fs::write(self.root.join(name), content);
    }

    pub fn delete(&self, name: &str) -> bool {
        fs::remove_file(self.root.join(name)).is_ok()
    }

    pub fn json(&self, name: &str) -> JsonFile {
        JsonFile::new(self.root.join(name))
    }

    pub fn jsonl(&self, name: &str) -> JsonlFile {
        JsonlFile::new(self.root.join(name))
    }

    pub fn sub(&self, name: &str) -> DirStore {
        DirStore::new(self.root.join(name))
    }

    pub fn exists(&self, name: &str) -> bool {
        self.root.join(name).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn test_json_file_load_save() {
        let dir = temp_dir();
        let file = JsonFile::new(dir.path().join("test.json"));
        assert_eq!(file.load(), Value::Null);

        let data = serde_json::json!({"key": "value"});
        file.save(&data);
        assert_eq!(file.load(), data);
    }

    #[test]
    fn test_json_file_changed() {
        let dir = temp_dir();
        let file = JsonFile::new(dir.path().join("test.json"));
        let data = serde_json::json!({"key": "value"});
        file.save(&data);
        assert!(!file.changed(&data));
        assert!(file.changed(&serde_json::json!({"key": "other"})));
    }

    #[test]
    fn test_jsonl_file_append_read() {
        let dir = temp_dir();
        let file = JsonlFile::new(dir.path().join("test.jsonl"));

        file.append(&serde_json::json!({"a": 1}));
        file.append(&serde_json::json!({"b": 2}));

        let entries = file.read_all();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["a"], 1);
        assert_eq!(entries[1]["b"], 2);
    }

    #[test]
    fn test_jsonl_file_read_tail() {
        let dir = temp_dir();
        let file = JsonlFile::new(dir.path().join("test.jsonl"));
        for i in 0..5 {
            file.append(&serde_json::json!({"i": i}));
        }
        let tail = file.read_tail(2);
        assert_eq!(tail.len(), 2);
        assert_eq!(tail[0]["i"], 3);
        assert_eq!(tail[1]["i"], 4);
    }

    #[test]
    fn test_jsonl_file_clear() {
        let dir = temp_dir();
        let file = JsonlFile::new(dir.path().join("test.jsonl"));
        file.append(&serde_json::json!({"a": 1}));
        file.clear();
        assert!(file.read_all().is_empty());
    }

    #[test]
    fn test_dir_store_basic() {
        let dir = temp_dir();
        let store = DirStore::new(dir.path().join("store"));
        store.ensure();
        assert!(store.path().exists());

        store.write("test.txt", "hello");
        assert_eq!(store.read("test.txt"), Some("hello".to_string()));
        assert!(store.exists("test.txt"));

        let list = store.list();
        assert!(list.contains(&"test.txt".to_string()));

        assert!(store.delete("test.txt"));
        assert!(!store.exists("test.txt"));
    }

    #[test]
    fn test_dir_store_json() {
        let dir = temp_dir();
        let store = DirStore::new(dir.path().join("store"));
        store.ensure();

        let jf = store.json("data.json");
        jf.save(&serde_json::json!({"x": 42}));
        assert_eq!(jf.load()["x"], 42);
    }

    #[test]
    fn test_dir_store_jsonl() {
        let dir = temp_dir();
        let store = DirStore::new(dir.path().join("store"));
        store.ensure();

        let jl = store.jsonl("log.jsonl");
        jl.append(&serde_json::json!({"msg": "hello"}));
        let entries = jl.read_all();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_dir_store_sub() {
        let dir = temp_dir();
        let store = DirStore::new(dir.path().join("store"));
        let sub = store.sub("child");
        sub.ensure();
        assert!(sub.path().exists());
    }
}
