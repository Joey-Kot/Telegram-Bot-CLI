use std::path::PathBuf;

use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct FilePart {
    pub field_name: String,
    pub path: PathBuf,
}

impl FilePart {
    pub fn new(field_name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            field_name: field_name.into(),
            path,
        }
    }
}

#[derive(Debug)]
pub struct RequestSpec {
    pub method: &'static str,
    pub fields: Map<String, Value>,
    pub files: Vec<FilePart>,
}

impl RequestSpec {
    pub fn new(method: &'static str) -> Self {
        Self {
            method,
            fields: Map::new(),
            files: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: impl Into<String>, value: Value) {
        self.fields.insert(name.into(), value);
    }

    pub fn insert_string(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.insert(name, Value::String(value.into()));
    }

    pub fn insert_i64(&mut self, name: impl Into<String>, value: i64) {
        self.insert(name, Value::Number(value.into()));
    }

    pub fn insert_u32(&mut self, name: impl Into<String>, value: u32) {
        self.insert(name, Value::Number(value.into()));
    }

    pub fn insert_true(&mut self, name: impl Into<String>, enabled: bool) {
        if enabled {
            self.insert(name, Value::Bool(true));
        }
    }

    pub fn add_file(&mut self, field_name: impl Into<String>, path: PathBuf) {
        self.files.push(FilePart::new(field_name, path));
    }
}
