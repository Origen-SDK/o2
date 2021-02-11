use super::file_utils::FilePermissions;
use crate::Metadata;
use crate::Result;
use std::convert::TryFrom;
use std::path::PathBuf;
use toml::map::Map;
use toml::Value;
use indexmap::IndexMap;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

static USER_PATH_OFFSET: &str = ".o2/.session";
static APP_PATH_OFFSET: &str = ".session";

#[derive(Serialize, Deserialize, Clone)]
pub enum Storeable {
    Val(Value),
    Serialized(Vec<u8>),
}

pub struct Sessions {
    pub app_session_root: Option<PathBuf>,
    pub user_session_root: PathBuf,
    pub app_sessions: HashMap<PathBuf, SessionStore>,
    pub user_sessions: HashMap<PathBuf, SessionStore>,
    // aux_sessions: Vec<SessionStore>,
    // default_app_session_permissions: Permissions,
    // default_user_session_permissions: Permissions,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            user_session_root: {
                let mut p = crate::core::user::current_home_dir().unwrap();
                p.push(USER_PATH_OFFSET);
                p
            },
            app_session_root: {
                if let Some(app) = crate::app() {
                    let mut p = app.root.clone();
                    p.push(APP_PATH_OFFSET);
                    Some(p)
                } else {
                    None
                }
            },
            app_sessions: HashMap::new(),
            user_sessions: HashMap::new(),
        }
    }

    // pub fn app_session(&mut self, session: Option<PathBuf>) -> Result<&SessionStore> {
    pub fn app_session(&mut self, session: Option<String>) -> Result<&mut SessionStore> {
        let mut path = self.app_session_root.as_ref().unwrap().clone();
        if let Some(s) = session {
            path.push(s);
        } else {
            path.push(crate::app().unwrap().name());
        }
        if !self.app_sessions.contains_key(&path) {
            self.create_app_session(path.clone())?;
        }
        Ok(self.app_sessions.get_mut(&path).unwrap())
    }

    pub fn user_session(&mut self, session: Option<String>) -> Result<&mut SessionStore> {
        let mut path = self.user_session_root.clone();
        if let Some(s) = session {
            path.push(s);
        } else {
            path.push(crate::core::user::get_current_id()?);
        }
        if !self.user_sessions.contains_key(&path) {
            self.create_user_session(path.clone())?;
        }
        Ok(self.user_sessions.get_mut(&path).unwrap())
    }

    pub fn create_app_session(&mut self, session_path: PathBuf) -> Result<()> {
        self.app_sessions.insert(
            session_path.clone(),
            SessionStore::new(session_path, true, FilePermissions::Group)?,
        );
        Ok(())
    }

    pub fn create_user_session(&mut self, session_path: PathBuf) -> Result<()> {
        self.user_sessions.insert(
            session_path.clone(),
            SessionStore::new(session_path, false, FilePermissions::Group)?,
        );
        Ok(())
    }

    pub fn get_app_session_root_string(&self) -> Result<String> {
        if let Some(root) = self.app_session_root.as_ref() {
            Ok(root.to_string_lossy().to_string())
        } else {
            error!("Attempted to get app session root but this hasn't been set yet")
        }
    }

    pub fn get_user_session_root_string(&self) -> Result<String> {
        Ok(self.user_session_root.to_string_lossy().to_string())
    }

    pub fn get_session(&self, path: PathBuf, is_app_session: bool) -> Result<&SessionStore> {
        if is_app_session {
            Ok(self.app_sessions.get(&path).unwrap())
        } else {
            Ok(self.user_sessions.get(&path).unwrap())
        }
    }

    pub fn get_mut_session(
        &mut self,
        path: PathBuf,
        is_app_session: bool,
    ) -> Result<&mut SessionStore> {
        if is_app_session {
            if !self.app_sessions.contains_key(&path) {
                self.create_app_session(path.clone())?;
            }
            Ok(self.app_sessions.get_mut(&path).unwrap())
        } else {
            if !self.user_sessions.contains_key(&path) {
                self.create_user_session(path.clone())?;
            }
            Ok(self.user_sessions.get_mut(&path).unwrap())
        }
    }

    pub fn clear_cache(&mut self) -> Result<()> {
        self.app_sessions.clear();
        self.user_sessions.clear();
        Ok(())
    }

    pub fn remove_files(&mut self) -> Result<()> {
        if let Some(p) = self.app_session_root.as_ref() {
            if p.exists() {
                std::fs::remove_dir_all(p)?;
            }
        }
        if self.user_session_root.exists() {
            std::fs::remove_dir_all(self.user_session_root.clone())?;
        }
        self.clear_cache()?;
        Ok(())
    }

    pub fn available_app_sessions(&self) -> Result<Vec<(String, PathBuf)>> {
        let mut retn = vec![];
        if let Some(app_path) = &self.app_session_root {
            for session in std::fs::read_dir(app_path)? {
                let session = session?;
                retn.push((session.file_name().into_string()?, session.path()));
            }
        }
        Ok(retn)
    }

    pub fn available_user_sessions(&self) -> Result<Vec<(String, PathBuf)>> {
        let mut retn = vec![];
        for session in std::fs::read_dir(&self.user_session_root)? {
            let session = session?;
            retn.push((session.file_name().into_string()?, session.path()));
        }
        Ok(retn)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionData {
    data: Map<String, Value>,
}

impl Default for SessionData {
    fn default() -> Self {
        Self { data: Map::new() }
    }
}

pub struct SessionStore {
    pub path: PathBuf,
    permissions: FilePermissions,
    data: SessionData,
    is_app_session: bool,
}

impl PartialEq for SessionStore {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.is_app_session == other.is_app_session
    }
}

impl SessionStore {
    // pub fn new_app_session() -> Result<Self> {
    //     // ...
    // }

    // pub fn new_user_session() -> Result<Self> {
    //     Self::new(origen::current_user!().home_dir, OSPermissions::Private)?
    // }

    pub fn new(
        path_to: PathBuf,
        is_app_session: bool,
        permissions: FilePermissions,
    ) -> Result<Self> {
        let mut s = Self {
            permissions: permissions,
            data: SessionData::default(),
            path: path_to,
            is_app_session: is_app_session,
        };
        s.refresh()?;
        Ok(s)
    }

    pub fn remove_file(&mut self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(self.path.clone())?;
        }
        self.refresh()?;
        Ok(())
    }

    pub fn name(&self) -> Result<String> {
        if let Some(p) = self.path.file_stem() {
            // to_string_lossy here will replace invalid unicode with ? symbols
            // however, in order to get this far the path would've been valid, so
            // this should be safe.
            // See: https://doc.rust-lang.org/std/ffi/struct.OsStr.html#method.to_string_lossy
            Ok(p.to_string_lossy().to_string())
        } else {
            crate::error!(
                "Problem occurred resolving session name. Expected a file stem in {:?}",
                self.path
            )
        }
    }

    pub fn store(&mut self, key: String, data: Metadata) -> Result<()> {
        self.data.data.insert(key, data.to_toml_value()?);
        self.write().unwrap();
        Ok(())
    }

    pub fn store_serialized(
        &mut self,
        key: String,
        bytes: &[u8],
        serializer: Option<String>,
        source: Option<String>,
    ) -> Result<()> {
        let metadata = Metadata::Serialized(bytes.to_vec(), serializer, source);
        self.store(key, metadata)
    }

    pub fn delete(&mut self, key: &str) -> Result<Option<Metadata>> {
        let value = self.data.data.remove(key);
        self.write().unwrap();
        if let Some(v) = value {
            Ok(Some(Metadata::try_from(&v)?))
        } else {
            Ok(None)
        }
    }

    pub fn retrieve(&mut self, key: &str) -> Result<Option<Metadata>> {
        let value = self.data.data.get(key);
        if let Some(v) = value {
            Ok(Some(Metadata::try_from(v)?))
        } else {
            Ok(None)
        }
    }

    pub fn retrieve_serialized(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        let stored = self.data.data.get(key);
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
                                        _ => return error!("Data at {} was not serialized!", key)
                                    }
                                }
                                Ok(Some(retn))
                            },
                            _ => return error!("Data at {} was not serialized!", key)
                        }
                    } else {
                        return error!(
                            "Expected data entry for {} in {:?}, but none was found",
                            key,
                            self.path
                        )
                    }
                }
                _ => return error!(
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

    pub fn refresh(&mut self) -> Result<()> {
        if let Some(d) = Self::read_toml(&self.path)? {
            self.data = d;
        } else {
            self.data = SessionData::default();
        }
        Ok(())
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
                return error!("Session located at {:?} does not appear to be a file", path);
            }
        } else {
            Ok(None)
        }
    }

    pub fn write(&mut self) -> Result<()> {
        log_trace!("Rewriting session '{:?}'", &self.path);
        if !self.path.parent().unwrap().exists() {
            std::fs::create_dir_all(format!("{}", self.path.parent().unwrap().display()))?;
        }
        let mut file = File::create(&self.path).unwrap();
        write!(file, "{}", toml::to_string(&self.data).unwrap()).unwrap();
        self.permissions.apply_to(&self.path, true)?;
        Ok(())
    }

    pub fn data(&self) -> Result<IndexMap<String, Metadata>> {
        let mut retn: IndexMap<String, Metadata> = IndexMap::new();
        for (k, v) in self.data.data.iter() {
            retn.insert(k.to_string(), Metadata::try_from(v)?);
        }
        Ok(retn)
    }

    pub fn len(&self) -> usize {
        self.data.data.len()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.data.data.keys().map( |k| k.as_str()).collect()
    }
}

#[cfg(all(test, not(origen_skip_frontend_tests)))]
mod tests {
    use crate::utility::session_store::{SessionStore, Sessions};
    use num_bigint::BigInt;

    /// Get the caller name. Taken from this SO answer:
    /// https://stackoverflow.com/a/63904992/8533619
    macro_rules! current_func {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);

            // Find and cut the rest of the path
            match &name[..name.len() - 3].rfind(':') {
                Some(pos) => &name[pos + 1..name.len() - 3],
                None => &name[..name.len() - 3],
            }
        }};
    }

    fn set_session_root(offset: &str) {
        let mut f = std::env::current_dir().unwrap();
        f.pop();
        f.pop();
        f.push("test_apps/python_app");
        f.push(offset);
        let mut s = crate::sessions();
        s.user_session_root = f;
        s.clear_cache().unwrap();
    }

    fn posture_session<'a>(sessions: &'a mut Sessions, name: &str) -> &'a mut SessionStore {
        // Update the root into the py-app's tmp area
        let mut f = std::env::current_dir().unwrap();
        f.pop();
        f.pop();
        f.push("test_apps/python_app/tmp/pytest/.session_rust");
        sessions.user_session_root = f;
        sessions.clear_cache().unwrap();
        let s = sessions.user_session(Some(name.to_string())).unwrap();
        s.remove_file().unwrap();
        s
    }

    fn store_from_frontend(name: &str, vals: Vec<(&str, &str)>) {
        let mut cmd = "origen.session_store.set_user_root(origen.app.root.joinpath('tmp/pytest/.session_rust')); ".to_string();
        cmd.push_str("print(origen.session_store.user_root()); ");
        cmd.push_str("origen.session_store.clear_cache(); ");

        let stores = vals
            .iter()
            .map(|v| {
                format!(
                    "origen.session_store.user_session('{}').store('{}', {})",
                    name, v.0, v.1
                )
            })
            .collect::<Vec<String>>();
        cmd.push_str(&stores.join("; "));
        crate::tests::run_python(&cmd).unwrap();
    }

    #[test]
    fn test_shared_session_string() {
        let offset = "tmp/pytest/.session_rust";
        set_session_root(offset);
        let mut s = crate::sessions();
        s.remove_files().unwrap();
        let session = s
            .user_session(Some("rust_test_session".to_string()))
            .unwrap();
        assert_eq!(session.retrieve("rust_test").unwrap(), None);

        let mut cmd = format!(
            "origen.session_store.set_user_root(origen.app.root.joinpath('{}')); ",
            offset
        );
        cmd.push_str("print(origen.session_store.user_root()); ");
        cmd.push_str("origen.session_store.clear_cache(); ");
        cmd.push_str(
            "origen.session_store.user_session('rust_test_session').store('rust_test', 'test_str')",
        );
        crate::tests::run_python(&cmd).unwrap();
        session.refresh().unwrap();
        assert_eq!(session.retrieve("rust_test").unwrap().is_some(), true);
        assert_eq!(
            session.retrieve("rust_test").unwrap(),
            Some(crate::Metadata::String("test_str".to_string()))
        );
        assert_eq!(
            session
                .retrieve("rust_test")
                .unwrap()
                .unwrap()
                .as_string()
                .unwrap(),
            "test_str".to_string()
        );
    }

    #[test]
    fn test_shared_session_bigint() {
        let mut s = crate::sessions();
        let session = posture_session(&mut s, current_func!());
        assert_eq!(session.retrieve("test_bigint").unwrap(), None);
        assert_eq!(session.retrieve("test_bigint_neg").unwrap(), None);
        store_from_frontend(
            current_func!(),
            vec![("test_bigint", "1000"), ("test_bigint_neg", "-1000")],
        );
        session.refresh().unwrap();
        assert_eq!(
            session
                .retrieve("test_bigint")
                .unwrap()
                .unwrap()
                .as_bigint()
                .unwrap(),
            BigInt::from(1000 as usize)
        );
        assert_eq!(
            session
                .retrieve("test_bigint_neg")
                .unwrap()
                .unwrap()
                .as_bigint()
                .unwrap(),
            BigInt::from(-1000)
        );
    }

    #[test]
    fn test_shared_session_bool() {
        let mut s = crate::sessions();
        let session = posture_session(&mut s, current_func!());
        assert_eq!(session.retrieve("test_true").unwrap(), None);
        assert_eq!(session.retrieve("test_false").unwrap(), None);
        store_from_frontend(
            current_func!(),
            vec![("test_true", "True"), ("test_false", "False")],
        );
        session.refresh().unwrap();
        assert_eq!(
            session
                .retrieve("test_true")
                .unwrap()
                .unwrap()
                .as_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            session
                .retrieve("test_false")
                .unwrap()
                .unwrap()
                .as_bool()
                .unwrap(),
            false
        );
    }

    #[test]
    fn test_shared_session_multiple_items() {
        let mut s = crate::sessions();
        let session = posture_session(&mut s, current_func!());
        assert_eq!(session.retrieve("test_str").unwrap(), None);
        assert_eq!(session.retrieve("test_bigint").unwrap(), None);
        assert_eq!(session.retrieve("test_bool").unwrap(), None);
        store_from_frontend(
            current_func!(),
            vec![
                ("test_str", "'hi'"),
                ("test_bigint", "0"),
                ("test_bool", "True"),
            ],
        );
        session.refresh().unwrap();
        assert_eq!(
            session
                .retrieve("test_str")
                .unwrap()
                .unwrap()
                .as_string()
                .unwrap(),
            "hi"
        );
        assert_eq!(
            session
                .retrieve("test_bigint")
                .unwrap()
                .unwrap()
                .as_bigint()
                .unwrap(),
            BigInt::from(0)
        );
        assert_eq!(
            session
                .retrieve("test_bool")
                .unwrap()
                .unwrap()
                .as_bool()
                .unwrap(),
            true
        );
    }

    #[test]
    fn test_shared_session_vector_of_stuff() {
        let mut s = crate::sessions();
        let session = posture_session(&mut s, current_func!());
        assert_eq!(session.retrieve("test_vec").unwrap(), None);
        store_from_frontend(
            current_func!(),
            vec![
                ("test_vec", "['hi', -1, 0, True, 6.022]"),
                ("test_vec2", "['hello', 2, -2, False, -6.022]"),
            ],
        );
        session.refresh().unwrap();
        let v = session
            .retrieve("test_vec")
            .unwrap()
            .unwrap()
            .as_vec()
            .unwrap();
        assert_eq!(v.len(), 5);
        assert_eq!(v[0].as_string().unwrap(), "hi");
        assert_eq!(v[1].as_bigint().unwrap(), BigInt::from(-1));
        assert_eq!(v[2].as_bigint().unwrap(), BigInt::from(0));
        assert_eq!(v[3].as_bool().unwrap(), true);
        assert_eq!(v[4].as_float().unwrap(), 6.022);

        let v = session
            .retrieve("test_vec2")
            .unwrap()
            .unwrap()
            .as_vec()
            .unwrap();
        assert_eq!(v[0].as_string().unwrap(), "hello");
        assert_eq!(v[1].as_bigint().unwrap(), BigInt::from(2));
        assert_eq!(v[2].as_bigint().unwrap(), BigInt::from(-2));
        assert_eq!(v[3].as_bool().unwrap(), false);
        assert_eq!(v[4].as_float().unwrap(), -6.022);
    }
}
