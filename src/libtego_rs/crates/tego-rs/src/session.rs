// standard
use std::collections::BTreeMap;

// extern
use anyhow::{Context as AnyhowContext, Result};
use rico_profile::v4::profile::{Profile, User, UserType};

// internal
use crate::context::UserHandle;
use crate::macros::*;

pub(crate) struct Session {
    profile: Profile,
    users: BTreeMap<UserHandle, User>,
}

impl Session {
    pub fn new(profile: Profile) -> Result<Self> {
        let mut users: BTreeMap<UserHandle, User> = Default::default();
        for (user, user_handle) in profile.get_users()? {
            if let Some(_) = users.insert(user_handle.0, user) {
                unreachable!("Profile should not have duplicate UserHandles in users table");
            }
        }
        log_info!("new session with {} users", users.len());
        Ok(Session { profile, users })
    }

    pub fn get_user_count(&self) -> usize {
        self.users.len()
    }

    pub fn get_user_handles(&self) -> Vec<UserHandle> {
        self.users.keys().cloned().collect()
    }

    fn get_user(&self, user_handle: UserHandle) -> Result<&User> {
        self.users
            .get(&user_handle)
            .context("No user with UserHandle '{user_handle}'")
    }

    pub fn get_user_type(&self, user_handle: UserHandle) -> Result<UserType> {
        Ok(self.get_user(user_handle)?.user_type)
    }

    pub fn get_user_nickname(&self, user_handle: UserHandle) -> Result<String> {
        Ok(self.get_user(user_handle)?.user_profile.nickname.clone())
    }

    pub fn get_user_pet_name(&self, user_handle: UserHandle) -> Result<Option<String>> {
        Ok(self.get_user(user_handle)?.user_profile.pet_name.clone())
    }
}
