use crate::{app, Result, ORIGEN_CONFIG};
use std::sync::MutexGuard;
use origen_metal as om;
use om::framework::sessions::{Sessions, SessionGroup, SessionStore};
use om::file::FilePermissions;
use std::path::PathBuf;

pub static DEFAULT_USER_PATH_OFFSET: &str = "./.o2/.session";
pub static DEFAULT_APP_PATH_OFFSET: &str = "./.session";
pub static USER_GROUP_NAME: &str = "__user__";
pub static APP_GROUP_NAME: &str = "__app__";
pub static USER_SESSIONS_FILE_PERMISSIONS: Option<FilePermissions> = Some(FilePermissions::Private);
pub static APP_SESSIONS_FILE_PERMISSIONS: Option<FilePermissions> = Some(FilePermissions::GroupWritable);

pub fn setup_sessions() -> om::Result<()> {
    log_trace!("Setting up user session...");
    let mut user_root;
    if let Some(r) = &ORIGEN_CONFIG.session__user_root {
        user_root = PathBuf::from(r);
    } else {
        user_root = crate::core::user::current_home_dir()?;
    }
    user_root.push(DEFAULT_USER_PATH_OFFSET);

    let mut sessions = om::sessions();
    let user_grp = sessions.add_group(USER_GROUP_NAME, &user_root, USER_SESSIONS_FILE_PERMISSIONS.clone())?;
    user_grp.add_session(&crate::core::user::get_current_id()?)?;

    if let Some(app) = crate::app() {
        log_trace!("Setting up application session...");
        let mut app_root;
        if let Some(r) = &app.config().app_session_root {
            app_root = PathBuf::from(r);
        } else {
            app_root = app.root.clone();
        }
        app_root.push(DEFAULT_APP_PATH_OFFSET);
        let app_grp = sessions.add_group(APP_GROUP_NAME, &app_root, APP_SESSIONS_FILE_PERMISSIONS.clone())?;
        app_grp.add_session(&app.name())?;
    }
    Ok(())
}

pub fn clean_sessions() -> om::Result<()> {
    {
        let mut sessions = om::sessions();
        sessions.clean()?;
    }
    setup_sessions()
}

pub fn with_user_session<T, F>(session: Option<String>, mut func: F) -> om::Result<T>
where
    F: FnMut(&SessionStore) -> Result<T>,
{
    let mut sessions = om::sessions();
    let grp = sessions.require_mut_group(USER_GROUP_NAME)?;
    let s = grp.get_or_add(session.as_ref().unwrap_or(&crate::core::user::get_current_id()?))?;
    Ok(func(s)?)
}

pub fn with_user_session_group<F, T>(sessions: Option<MutexGuard<Sessions>>, mut f: F) -> om::Result<T>
where
    F: FnMut(&SessionGroup, &MutexGuard<Sessions>) -> om::Result<T>,
{
    match sessions {
        Some(s) => {
            Ok(f(s.require_group(USER_GROUP_NAME)?, &s)?)
        },
        None => {
            let s = om::sessions();
            Ok(f(s.require_group(USER_GROUP_NAME)?, &s)?)
        }
    }
}

pub fn with_mut_user_session_group<F, T>(sessions: Option<MutexGuard<Sessions>>, mut f: F) -> om::Result<T>
where
    F: FnMut(&mut SessionGroup) -> om::Result<T>,
{
    match sessions {
        Some(mut s) => {
            Ok(f(s.require_mut_group(USER_GROUP_NAME)?)?)
        },
        None => {
            let mut s = om::sessions();
            Ok(f(s.require_mut_group(USER_GROUP_NAME)?)?)
        }
    }
}

pub fn with_app_session<T, F>(session: Option<String>, mut func: F) -> om::Result<T>
where
    F: FnMut(&SessionStore) -> Result<T>,
{
    if let Some(app) = app() {
        let mut sessions = om::sessions();
        let grp = sessions.require_mut_group(APP_GROUP_NAME)?;
        let s = grp.get_or_add(session.as_ref().unwrap_or(&app.name()))?;
        Ok(func(s)?)
    } else {
        om::bail!("No application is present! No app session is available!");
    }
}

pub fn with_app_session_group<F, T>(sessions: Option<MutexGuard<Sessions>>, mut f: F) -> om::Result<T>
where
    F: FnMut(&SessionGroup, &MutexGuard<Sessions>) -> om::Result<T>,
{
    if app().is_some() {
        match sessions {
            Some(s) => {
                Ok(f(s.require_group(APP_GROUP_NAME)?, &s)?)
            },
            None => {
                let s = om::sessions();
                Ok(f(s.require_group(APP_GROUP_NAME)?, &s)?)
            }
        }
    } else {
        om::bail!("No application is present! No app session is available!");
    }
}

pub fn with_mut_app_session_group<F, T>(sessions: Option<MutexGuard<Sessions>>, mut f: F) -> om::Result<T>
where
    F: FnMut(&mut SessionGroup) -> om::Result<T>,
{
    if app().is_some() {
        match sessions {
            Some(mut s) => {
                Ok(f(s.require_mut_group(APP_GROUP_NAME)?)?)
            },
            None => {
                let mut s = om::sessions();
                Ok(f(s.require_mut_group(APP_GROUP_NAME)?)?)
            }
        }
    } else {
        om::bail!("No application is present! No app session is available!");
    }
}
