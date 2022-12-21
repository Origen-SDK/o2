mod session_group;
mod session_store;

pub use session_group::SessionGroup;
pub use session_store::SessionStore;

use crate::file::FilePermissions;
use crate::Result;
use indexmap::IndexMap;
use std::path::Path;
use std::sync::MutexGuard;

// TODO make this lazy static?
lazy_static! {
    static ref DEFAULT_FILE_PERMISSIONS: FilePermissions = FilePermissions::GroupWritable;
}

pub fn sessions() -> MutexGuard<'static, Sessions> {
    crate::SESSIONS.lock().unwrap()
}

pub fn with_session_group<F, T>(
    name: &str,
    sessions: Option<MutexGuard<Sessions>>,
    mut f: F,
) -> Result<T>
where
    F: FnMut(&SessionGroup, &MutexGuard<Sessions>) -> Result<T>,
{
    match sessions {
        Some(s) => Ok(f(s.require_group(name)?, &s)?),
        None => {
            let s = super::sessions::sessions();
            Ok(f(s.require_group(name)?, &s)?)
        }
    }
}

pub struct Sessions {
    groups: IndexMap<String, SessionGroup>,
    standalones: IndexMap<String, SessionStore>,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            groups: IndexMap::new(),
            standalones: IndexMap::new(),
        }
    }

    pub fn add_group(
        &mut self,
        group_name: &str,
        root: &Path,
        file_permissions: Option<FilePermissions>,
    ) -> Result<&mut SessionGroup> {
        if let Some(sg) = self.groups.get(group_name) {
            bail!(&format!(
                "Session group '{}' has already been added (path: '{}')",
                group_name,
                sg.path().display()
            ));
        }
        self.groups.insert(
            group_name.to_string(),
            SessionGroup::new(group_name, root, file_permissions)?,
        );
        Ok(self.groups.get_mut(group_name).unwrap())
    }

    pub fn add_standalone(
        &mut self,
        name: &str,
        root: &Path,
        file_permissions: Option<FilePermissions>,
    ) -> Result<&SessionStore> {
        if let Some(s) = self.standalones.get(name) {
            bail!(&format!(
                "Standalone session '{}' has already been added (path: '{}')",
                name,
                s.path().display()
            ));
        }
        self.standalones.insert(
            name.to_string(),
            SessionStore::new(name, root, None, file_permissions)?,
        );
        Ok(self.standalones.get(name).unwrap())
    }

    pub fn get_group(&self, group_name: &str) -> Option<&SessionGroup> {
        self.groups.get(group_name)
    }

    pub fn get_mut_group(&mut self, group_name: &str) -> Option<&mut SessionGroup> {
        self.groups.get_mut(group_name)
    }

    // TESTS_NEEDED
    pub fn ensure_group(
        &mut self,
        name: &str,
        root: &Path,
        file_permissions: Option<FilePermissions>,
    ) -> Result<bool> {
        if self.groups.contains_key(name) {
            Ok(false)
        } else {
            self.add_group(name, root, file_permissions)?;
            Ok(true)
        }
    }

    pub fn require_group(&self, group: &str) -> Result<&SessionGroup> {
        match self.groups.get(group) {
            Some(s) => Ok(s),
            None => bail!("Session group {} has not been created yet!", group),
        }
    }

    pub fn require_mut_group(&mut self, group: &str) -> Result<&mut SessionGroup> {
        match self.groups.get_mut(group) {
            Some(s) => Ok(s),
            None => bail!("Session group {} has not been created yet!", group),
        }
    }

    pub fn get_standalone(&self, name: &str) -> Option<&SessionStore> {
        self.standalones.get(name)
    }

    pub fn get_mut_standalone(&mut self, name: &str) -> Option<&mut SessionStore> {
        self.standalones.get_mut(name)
    }

    pub fn require_standalone(&self, name: &str) -> Result<&SessionStore> {
        match self.standalones.get(name) {
            Some(s) => Ok(s),
            None => bail!("Standalone session {} has not been created yet!", name),
        }
    }

    pub fn require_mut_standalone(&mut self, name: &str) -> Result<&mut SessionStore> {
        match self.standalones.get_mut(name) {
            Some(s) => Ok(s),
            None => bail!("Standalone session {} has not been created yet!", name),
        }
    }

    pub fn groups(&self) -> &IndexMap<String, SessionGroup> {
        &self.groups
    }

    pub fn standalones(&self) -> &IndexMap<String, SessionStore> {
        &self.standalones
    }

    pub fn delete_group(&mut self, name: &str) -> Result<bool> {
        Ok(match self.groups.remove(name) {
            Some(mut g) => {
                g.clean()?;
                true
            }
            None => false,
        })
    }

    pub fn delete_standalone(&mut self, name: &str) -> Result<bool> {
        Ok(match self.standalones.remove(name) {
            Some(s) => {
                s.remove_file()?;
                true
            }
            None => false,
        })
    }

    // TESTS_NEEDED
    pub fn refresh(&self) -> Result<()> {
        for (_, g) in self.groups.iter() {
            g.refresh()?;
        }
        for (_, s) in self.standalones.iter() {
            s.refresh()?;
        }
        Ok(())
    }

    // TESTS_NEEDED
    pub fn clean(&mut self) -> Result<()> {
        for (_, g) in self.groups.iter_mut() {
            g.clean()?;
        }
        for (_, s) in self.standalones.iter() {
            s.remove_file()?;
        }
        self.unload()
    }

    // TESTS_NEEDED
    pub fn unload(&mut self) -> Result<()> {
        self.groups = IndexMap::new();
        self.standalones = IndexMap::new();
        Ok(())
    }
}

#[cfg(all(test, not(origen_skip_frontend_tests)))]
mod tests {
    use crate::current_func;
    use crate::framework::sessions::{SessionStore, Sessions};
    use num_bigint::BigInt;
    use std::path::PathBuf;

    lazy_static! {
        static ref TEST_SESSION_DIR: PathBuf = {
            let mut f = std::env::current_dir().unwrap();
            f.pop();
            f.pop();
            f.push("python/origen_metal/tmp/pytest/.session_rust");
            f
        };
    }

    fn posture_session<'a>(sessions: &'a mut Sessions, name: &str) -> &'a SessionStore {
        let s = sessions
            .add_standalone(name, &TEST_SESSION_DIR, None)
            .unwrap();
        s.remove_file().unwrap();
        s
    }

    fn store_from_frontend(name: &str, vals: Vec<(&str, &str)>) {
        let mut cmd = format!(
            "om.sessions.add_standalone('{}', r'{}'); ",
            name,
            &TEST_SESSION_DIR.display()
        );

        let stores = vals
            .iter()
            .map(|v| {
                format!(
                    "om.sessions.standalone('{}').store('{}', {})",
                    name, v.0, v.1
                )
            })
            .collect::<Vec<String>>();
        cmd.push_str(&stores.join("; "));
        crate::tests::run_python(&cmd).unwrap();
    }

    #[test]
    fn test_shared_session_string() {
        let mut s = crate::sessions();
        let session = s
            .add_standalone("rust_test_session", &TEST_SESSION_DIR, None)
            .unwrap();
        session.remove_file().unwrap();
        assert_eq!(session.retrieve("rust_test").unwrap(), None);

        let mut cmd = format!(
            "s = om.sessions.add_standalone('rust_test_session', r'{}'); ",
            &TEST_SESSION_DIR.display()
        );
        cmd.push_str("print(s.path); ");
        cmd.push_str("s.store('rust_test', 'test_str')");

        crate::tests::run_python(&cmd).unwrap();
        session.refresh().unwrap();
        assert_eq!(session.retrieve("rust_test").unwrap().is_some(), true);
        assert_eq!(
            session.retrieve("rust_test").unwrap(),
            Some(crate::TypedValue::String("test_str".to_string()))
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
