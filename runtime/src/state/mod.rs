use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::CapsuleId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateSnapshot {
    pub capsule_id: CapsuleId,
    pub entries: Vec<(String, Vec<u8>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateError {
    Io(String),
    LockPoisoned,
    InvalidKey(String),
}

pub trait StateStore: Send + Sync {
    fn get(&self, capsule_id: &CapsuleId, key: &str) -> Result<Option<Vec<u8>>, StateError>;
    fn set(&self, capsule_id: &CapsuleId, key: &str, value: Vec<u8>) -> Result<(), StateError>;
    fn delete(&self, capsule_id: &CapsuleId, key: &str) -> Result<(), StateError>;
    fn list(&self, capsule_id: &CapsuleId) -> Result<Vec<String>, StateError>;
    fn snapshot(&self, capsule_id: &CapsuleId) -> Result<StateSnapshot, StateError>;
    fn restore(&self, snapshot: &StateSnapshot) -> Result<(), StateError>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryStateStore {
    entries: Arc<RwLock<HashMap<(CapsuleId, String), Vec<u8>>>>,
}

impl InMemoryStateStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl StateStore for InMemoryStateStore {
    fn get(&self, capsule_id: &CapsuleId, key: &str) -> Result<Option<Vec<u8>>, StateError> {
        let entries = self.entries.read().map_err(|_| StateError::LockPoisoned)?;
        Ok(entries.get(&(capsule_id.clone(), key.to_string())).cloned())
    }

    fn set(&self, capsule_id: &CapsuleId, key: &str, value: Vec<u8>) -> Result<(), StateError> {
        validate_key(key)?;
        let mut entries = self.entries.write().map_err(|_| StateError::LockPoisoned)?;
        entries.insert((capsule_id.clone(), key.to_string()), value);
        Ok(())
    }

    fn delete(&self, capsule_id: &CapsuleId, key: &str) -> Result<(), StateError> {
        let mut entries = self.entries.write().map_err(|_| StateError::LockPoisoned)?;
        entries.remove(&(capsule_id.clone(), key.to_string()));
        Ok(())
    }

    fn list(&self, capsule_id: &CapsuleId) -> Result<Vec<String>, StateError> {
        let entries = self.entries.read().map_err(|_| StateError::LockPoisoned)?;
        let mut keys: Vec<String> = entries
            .keys()
            .filter(|(id, _)| id == capsule_id)
            .map(|(_, key)| key.clone())
            .collect();
        keys.sort();
        Ok(keys)
    }

    fn snapshot(&self, capsule_id: &CapsuleId) -> Result<StateSnapshot, StateError> {
        let entries = self.entries.read().map_err(|_| StateError::LockPoisoned)?;
        let mut snapshot_entries: Vec<(String, Vec<u8>)> = entries
            .iter()
            .filter(|((id, _), _)| id == capsule_id)
            .map(|((_, key), value)| (key.clone(), value.clone()))
            .collect();
        snapshot_entries.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(StateSnapshot {
            capsule_id: capsule_id.clone(),
            entries: snapshot_entries,
        })
    }

    fn restore(&self, snapshot: &StateSnapshot) -> Result<(), StateError> {
        let mut entries = self.entries.write().map_err(|_| StateError::LockPoisoned)?;
        entries.retain(|(id, _), _| id != &snapshot.capsule_id);
        for (key, value) in &snapshot.entries {
            validate_key(key)?;
            entries.insert((snapshot.capsule_id.clone(), key.clone()), value.clone());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FileStateStore {
    root: PathBuf,
}

impl FileStateStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn default_root() -> PathBuf {
        std::env::var_os("SOVEREIGN_MESH_STATE_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|home| home.join(".sovereign-mesh").join("state"))
            })
            .or_else(|| {
                std::env::var_os("USERPROFILE")
                    .map(PathBuf::from)
                    .map(|home| home.join(".sovereign-mesh").join("state"))
            })
            .unwrap_or_else(|| PathBuf::from(".sovereign-mesh").join("state"))
    }

    fn capsule_dir(&self, capsule_id: &CapsuleId) -> PathBuf {
        self.root.join(sanitize_path_component(capsule_id))
    }

    fn key_path(&self, capsule_id: &CapsuleId, key: &str) -> Result<PathBuf, StateError> {
        validate_key(key)?;
        Ok(self.capsule_dir(capsule_id).join(sanitize_path_component(key)))
    }
}

impl Default for FileStateStore {
    fn default() -> Self {
        Self::new(Self::default_root())
    }
}

impl StateStore for FileStateStore {
    fn get(&self, capsule_id: &CapsuleId, key: &str) -> Result<Option<Vec<u8>>, StateError> {
        let path = self.key_path(capsule_id, key)?;
        if !path.exists() {
            return Ok(None);
        }
        fs::read(path).map(Some).map_err(map_io)
    }

    fn set(&self, capsule_id: &CapsuleId, key: &str, value: Vec<u8>) -> Result<(), StateError> {
        let path = self.key_path(capsule_id, key)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(map_io)?;
        }
        fs::write(path, value).map_err(map_io)
    }

    fn delete(&self, capsule_id: &CapsuleId, key: &str) -> Result<(), StateError> {
        let path = self.key_path(capsule_id, key)?;
        if path.exists() {
            fs::remove_file(path).map_err(map_io)?;
        }
        Ok(())
    }

    fn list(&self, capsule_id: &CapsuleId) -> Result<Vec<String>, StateError> {
        let dir = self.capsule_dir(capsule_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut keys = Vec::new();
        for entry in fs::read_dir(dir).map_err(map_io)? {
            let entry = entry.map_err(map_io)?;
            if entry.file_type().map_err(map_io)?.is_file() {
                keys.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        keys.sort();
        Ok(keys)
    }

    fn snapshot(&self, capsule_id: &CapsuleId) -> Result<StateSnapshot, StateError> {
        let mut entries = Vec::new();
        for key in self.list(capsule_id)? {
            if let Some(value) = self.get(capsule_id, &key)? {
                entries.push((key, value));
            }
        }
        Ok(StateSnapshot {
            capsule_id: capsule_id.clone(),
            entries,
        })
    }

    fn restore(&self, snapshot: &StateSnapshot) -> Result<(), StateError> {
        let dir = self.capsule_dir(&snapshot.capsule_id);
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(map_io)?;
        }
        fs::create_dir_all(&dir).map_err(map_io)?;
        for (key, value) in &snapshot.entries {
            self.set(&snapshot.capsule_id, key, value.clone())?;
        }
        Ok(())
    }
}

fn validate_key(key: &str) -> Result<(), StateError> {
    if key.trim().is_empty() || key.contains("..") || key.contains('/') || key.contains('\\') {
        return Err(StateError::InvalidKey(key.to_string()));
    }
    Ok(())
}

fn sanitize_path_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn map_io(error: std::io::Error) -> StateError {
    StateError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{FileStateStore, InMemoryStateStore, StateStore};

    #[test]
    fn in_memory_store_get_set_delete_list() {
        let store = InMemoryStateStore::new();
        let capsule = "capsule.test.v1".to_string();

        store.set(&capsule, "profile", b"abc".to_vec()).unwrap();
        assert_eq!(store.get(&capsule, "profile").unwrap(), Some(b"abc".to_vec()));
        assert_eq!(store.list(&capsule).unwrap(), vec!["profile"]);
        store.delete(&capsule, "profile").unwrap();
        assert_eq!(store.get(&capsule, "profile").unwrap(), None);
    }

    #[test]
    fn snapshot_restore_round_trip() {
        let store = InMemoryStateStore::new();
        let capsule = "capsule.test.v1".to_string();

        store.set(&capsule, "one", b"1".to_vec()).unwrap();
        let snapshot = store.snapshot(&capsule).unwrap();
        store.set(&capsule, "two", b"2".to_vec()).unwrap();
        store.restore(&snapshot).unwrap();

        assert_eq!(store.list(&capsule).unwrap(), vec!["one"]);
    }

    #[test]
    fn file_store_round_trip() {
        let root = std::env::temp_dir().join(format!(
            "sovereign-mesh-state-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let store = FileStateStore::new(&root);
        let capsule = "capsule.file.v1".to_string();

        store.set(&capsule, "blob", b"bytes".to_vec()).unwrap();
        assert_eq!(store.get(&capsule, "blob").unwrap(), Some(b"bytes".to_vec()));
        let snapshot = store.snapshot(&capsule).unwrap();
        store.delete(&capsule, "blob").unwrap();
        store.restore(&snapshot).unwrap();
        assert_eq!(store.get(&capsule, "blob").unwrap(), Some(b"bytes".to_vec()));

        let _ = std::fs::remove_dir_all(&root);
    }
}
