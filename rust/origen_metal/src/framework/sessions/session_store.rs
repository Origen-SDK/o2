use crate::file::FilePermissions;
use crate::Result;
use crate::TypedValue;
use indexmap::IndexMap;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};
use toml::map::Map;
use toml::Value;

use std::fs::File;
use std::io::prelude::*;
use std::sync::RwLock;

use super::DEFAULT_FILE_PERMISSIONS;

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionData {
    data: Map<String, Value>,
}

impl Default for SessionData {
    fn default() -> Self {
        Self { data: Map::new() }
    }
}

#[derive(Debug)]
pub struct SessionStore {
    path: PathBuf,
    group: Option<String>,
    permissions: Option<FilePermissions>,
    data: RwLock<SessionData>,
}

impl PartialEq for SessionStore {
    // Two session stores are the same if they point to the same file and have
    // the same permissions.
    // Note: one may be "out-of-sync" and need refreshing and the compare will still return true
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self
                .permissions
                .as_ref()
                .unwrap_or(&DEFAULT_FILE_PERMISSIONS)
                == other
                    .permissions
                    .as_ref()
                    .unwrap_or(&DEFAULT_FILE_PERMISSIONS)
    }
}

impl SessionStore {
    pub fn new(
        name: &str,
        root: &Path,
        group: Option<String>,
        permissions: Option<FilePermissions>,
    ) -> Result<Self> {
        let s = Self {
            permissions: permissions.clone(),
            data: RwLock::new(SessionData::default()),
            path: {
                let mut r = root.to_path_buf();
                r.push(name);
                r
            },
            group: group,
        };
        s.refresh()?;
        Ok(s)
    }

    pub fn with_data<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&SessionData) -> Result<T>,
    {
        let data = self.data.read()?;
        func(&data)
    }

    pub fn with_mut_data<T, F>(&self, mut func: F) -> Result<T>
    where
        F: FnMut(&mut SessionData) -> Result<T>,
    {
        let mut data = self.data.write()?;
        func(&mut data)
    }

    pub fn remove_file(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(self.path.clone())?;
        }
        self.refresh()?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn group(&self) -> &Option<String> {
        &self.group
    }

    pub fn root(&self) -> &Path {
        self.path.parent().unwrap()
    }

    pub fn name(&self) -> Result<String> {
        Ok(self
            .path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()?)
    }

    pub fn store(&self, key: String, data: TypedValue) -> Result<()> {
        {
            let mut d = self.data.write()?;
            d.data.insert(key, data.to_toml_value()?);
        }
        self.write()?;
        Ok(())
    }

    pub fn store_serialized(
        &self,
        key: String,
        bytes: &[u8],
        serializer: Option<String>,
        source: Option<String>,
    ) -> Result<()> {
        let val = TypedValue::Serialized(bytes.to_vec(), serializer, source);
        self.store(key, val)
    }

    pub fn delete(&self, key: &str) -> Result<Option<TypedValue>> {
        let value = self.with_mut_data(|data| Ok(data.data.remove(key)))?;
        self.write().unwrap();
        if let Some(v) = value {
            Ok(Some(TypedValue::try_from(&v)?))
        } else {
            Ok(None)
        }
    }

    pub fn retrieve(&self, key: &str) -> Result<Option<TypedValue>> {
        self.with_data(|data| {
            let value = data.data.get(key);
            if let Some(v) = value {
                Ok(Some(TypedValue::try_from(v)?))
            } else {
                Ok(None)
            }
        })
    }

    pub fn retrieve_serialized(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let d = self.data.read()?;
        let stored = d.data.get(key);
        if let Some(map) = stored {
            match map {
                Value::Table(map) => {
                    if let Some(data) = map.get("data") {
                        match data {
                            Value::Array(vec) => {
                                let mut retn: Vec<u8> = vec![];
                                for byte in vec.iter() {
                                    match byte {
                                        Value::Integer(b) => {
                                            retn.push(*b as u8);
                                        },
                                        _ => bail!("Data at {} was not serialized!", key)
                                    }
                                }
                                Ok(Some(retn))
                            },
                            _ => bail!("Data at {} was not serialized!", key)
                        }
                    } else {
                        bail!(
                            "Expected data entry for {} in {:?}, but none was found",
                            key,
                            self.path
                        )
                    }
                }
                _ => bail!(
                    "Session data for {} in {:?} was not stored correctly. Expected a table, received {:?}",
                    key,
                    self.path,
                    map
                )
            }
        } else {
            Ok(None)
        }
    }

    pub fn refresh(&self) -> Result<()> {
        self.with_mut_data(|data| {
            if let Some(d) = Self::read_toml(&self.path)? {
                *data = d;
            } else {
                *data = SessionData::default();
            }
            Ok(())
        })
    }

    pub fn read_toml(path: &PathBuf) -> Result<Option<SessionData>> {
        if path.exists() {
            if path.is_file() {
                let mut file = File::open(path).unwrap();
                let mut buffer = String::new();
                file.read_to_string(&mut buffer).unwrap();
                // Accept an empty file. This will break the attempt to parse to a
                // SessionData, but is, IMO, a benign corner case.
                if buffer.len() == 0 {
                    Ok(None)
                } else {
                    Ok(Some(toml::from_str(&buffer).unwrap()))
                }
            } else {
                bail!("Session located at {:?} does not appear to be a file", path);
            }
        } else {
            Ok(None)
        }
    }

    pub fn write(&self) -> Result<()> {
        log_trace!("Rewriting session '{:?}'", &self.path);
        if !self.path.parent().unwrap().exists() {
            std::fs::create_dir_all(format!("{}", self.path.parent().unwrap().display()))?;
        }
        let mut file = File::create(&self.path).unwrap();
        write!(file, "{}", toml::to_string(&*self.data.read()?).unwrap()).unwrap();
        self.permissions
            .as_ref()
            .unwrap_or(&DEFAULT_FILE_PERMISSIONS)
            .apply_to(&self.path, true)?;
        Ok(())
    }

    pub fn data(&self) -> Result<IndexMap<String, TypedValue>> {
        self.with_data(|data| {
            let mut retn: IndexMap<String, TypedValue> = IndexMap::new();
            for (k, v) in data.data.iter() {
                retn.insert(k.to_string(), TypedValue::try_from(v)?);
            }
            Ok(retn)
        })
    }

    pub fn len(&self) -> Result<usize> {
        self.with_data(|data| Ok(data.data.len()))
    }

    pub fn keys(&self) -> Result<Vec<String>> {
        self.with_data(|data| Ok(data.data.keys().map(|k| k.to_string()).collect()))
    }
}
