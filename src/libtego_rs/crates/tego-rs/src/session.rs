// standard
use std::collections::{BTreeMap, BTreeSet};

// extern
use anyhow::{Context as AnyhowContext, Result};
use rico_profile::v4::profile::{Profile, User, UserType};

// internal
use crate::context::UserHandle;
use crate::macros::*;

pub(crate) type SessionHandle = i64;

pub(crate) struct Session {
    profile: Profile,
    users: BTreeMap<UserHandle, User>,
    owner: UserHandle,
    allowed_users: BTreeSet<UserHandle>,
    pending_users: BTreeSet<UserHandle>,
    requesting_users: BTreeSet<UserHandle>,
    rejected_users: BTreeSet<UserHandle>,
    blocked_users: BTreeSet<UserHandle>,
}

impl Session {
    pub fn new(profile: Profile) -> Result<Self> {
        let mut users: BTreeMap<UserHandle, User> = Default::default();
        let mut owner: Option<UserHandle> = None;
        let mut allowed_users: BTreeSet<UserHandle> = Default::default();
        let mut pending_users: BTreeSet<UserHandle> = Default::default();
        let mut requesting_users: BTreeSet<UserHandle> = Default::default();
        let mut rejected_users: BTreeSet<UserHandle> = Default::default();
        let mut blocked_users: BTreeSet<UserHandle> = Default::default();

        for (user, user_handle) in profile.get_users()? {
            let user_handle = user_handle.0;
            let user_type = user.user_type;
            if let Some(_) = users.insert(user_handle, user) {
                unreachable!("Profile should not have duplicate UserHandles in users table");
            }

            let _ = match user_type {
                UserType::Owner => {
                    assert!(owner.is_none(), "Profile should only have one Owner");
                    owner = Some(user_handle);
                    false
                }
                UserType::Allowed => allowed_users.insert(user_handle),
                UserType::Pending => pending_users.insert(user_handle),
                UserType::Requesting => requesting_users.insert(user_handle),
                UserType::Rejected => rejected_users.insert(user_handle),
                UserType::Blocked => blocked_users.insert(user_handle),
            };
        }

        let owner = owner.context("Profile must have an owner")?;

        Ok(Session {
            profile,
            users,
            owner,
            allowed_users,
            pending_users,
            requesting_users,
            rejected_users,
            blocked_users,
        })
    }

    pub fn get_users(&self) -> &BTreeMap<UserHandle, User> {
        &self.users
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
