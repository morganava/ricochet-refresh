// standard
use std::collections::BTreeMap;

// extern
use anyhow::{Context, Result};

// crate
use crate::ffi::handle::*;

pub struct ObjectMap<T, FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> {
    map: Option<BTreeMap<Handle<FFI_STRUCT, TAG_BITS, TAG>, T>>,
    counter: usize,
}

#[allow(clippy::new_without_default)]
impl<T, FFI_STRUCT, const TAG_BITS: usize, const TAG: usize>
    ObjectMap<T, FFI_STRUCT, TAG_BITS, TAG>
{
    pub const fn new() -> Self {
        Self {
            map: None,
            counter: 1usize,
        }
    }

    fn increment_counter(&mut self) -> usize {
        let key = self.counter;
        self.counter += 1usize;
        key
    }

    pub fn insert(&mut self, value: T) -> Handle<FFI_STRUCT, TAG_BITS, TAG> {
        let key_raw = self.increment_counter();
        let key: Handle<FFI_STRUCT, TAG_BITS, TAG> = Handle::new(key_raw);
        let map = self.map.get_or_insert_default();

        map.insert(key, value);
        key
    }

    pub fn remove(&mut self, key: &Handle<FFI_STRUCT, TAG_BITS, TAG>) -> Result<T> {
        let result = match &mut self.map {
            Some(map) => map.remove(key),
            None => None,
        };
        result.context(format!("no object in map for handle {key}"))
    }

    pub fn get(&self, key: &Handle<FFI_STRUCT, TAG_BITS, TAG>) -> Result<&T> {
        let result = match &self.map {
            Some(map) => map.get(key),
            None => None,
        };
        result.context(format!("no object in map for handle {key}"))
    }

    pub fn get_mut(&mut self, key: &Handle<FFI_STRUCT, TAG_BITS, TAG>) -> Result<&mut T> {
        let result = match &mut self.map {
            Some(map) => map.get_mut(key),
            None => None,
        };
        result.context(format!("no object in map for handle {key}"))
    }
}
