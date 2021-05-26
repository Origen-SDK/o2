use super::user::User;
use crate::Result;
use indexmap::IndexMap;

pub struct Users {
    users: IndexMap<String, User>,
    current_id: String,
    // initial_id: String,
}

impl Users {
    pub fn current_user(&self) -> Result<&User> {
        Ok(self.users.get(&self.current_id).unwrap())
    }

    pub fn current_user_mut(&mut self) -> Result<&mut User> {
        Ok(self.users.get_mut(&self.current_id).unwrap())
    }

    pub fn current_user_id(&self) -> Result<String> {
        Ok(self.current_id.clone())
    }

    pub fn user(&self, u: &str) -> Result<&User> {
        if let Some(user) = self.users.get(u).as_ref() {
            Ok(&user)
        } else {
            error!("No user '{}' has been added", u)
        }
    }

    pub fn user_mut(&mut self, u: &str) -> Result<&mut User> {
        if let Some(user) = self.users.get_mut(u) {
            Ok(user)
        } else {
            error!("No user '{}' has been added", u)
        }
    }

    pub fn users(&self) -> &IndexMap<String, User> {
        &self.users
    }

    pub fn add(&mut self, id: &str) -> Result<()> {
        if self.users.contains_key(id) {
            error!("User '{}' has already been added", id)
        } else {
            self.users.insert(id.to_string(), User::new(id));
            Ok(())
        }
    }
}

impl Default for Users {
    fn default() -> Self {
        let u = User::current();
        let id = u.id().to_string();
        let users = Self {
            users: {
                let mut i = IndexMap::new();
                i.insert(id.clone(), u);
                i
            },
            current_id: id.clone(),
            // initial_id: id
        };
        users
    }
}
