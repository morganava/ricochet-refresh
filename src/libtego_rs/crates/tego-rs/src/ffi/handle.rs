// standard
use std::ffi::c_void;
use std::fmt;

// extern
use anyhow::bail;

// crate
use crate::ffi::*;

pub struct Handle<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> {
    // least significant bits hold the tag
    // most significant bits hold the value
    raw: usize,
    _ffi_struct: std::marker::PhantomData<FFI_STRUCT>,
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> Handle<FFI_STRUCT, TAG_BITS, TAG> {
    const MAX_VALUE: usize = usize::MAX >> TAG_BITS;
    const TAG_MASK: usize = (1usize << TAG_BITS) - 1usize;

    pub fn new(value: usize) -> Self {
        assert!(value <= Self::MAX_VALUE);

        let raw = (value << TAG_BITS) | TAG;
        Self {
            raw,
            _ffi_struct: Default::default(),
        }
    }

    pub fn value(&self) -> usize {
        self.raw >> TAG_BITS
    }

    pub fn tag(&self) -> usize {
        self.raw & Self::TAG_MASK
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> fmt::Display
    for Handle<FFI_STRUCT, TAG_BITS, TAG>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let htype = std::any::type_name::<FFI_STRUCT>();
        let value = self.value();
        let tag = self.tag();
        write!(
            f,
            "Handle( type: {htype}, value: {value:#x}, tag: {tag:#x} )"
        )
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> Copy
    for Handle<FFI_STRUCT, TAG_BITS, TAG>
{
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> Clone
    for Handle<FFI_STRUCT, TAG_BITS, TAG>
{
    fn clone(&self) -> Self {
        Self {
            raw: self.raw,
            _ffi_struct: Default::default(),
        }
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> PartialEq
    for Handle<FFI_STRUCT, TAG_BITS, TAG>
{
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> Eq for Handle<FFI_STRUCT, TAG_BITS, TAG> {}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> Ord
    for Handle<FFI_STRUCT, TAG_BITS, TAG>
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> PartialOrd
    for Handle<FFI_STRUCT, TAG_BITS, TAG>
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> TryFrom<*const FFI_STRUCT>
    for Handle<FFI_STRUCT, TAG_BITS, TAG>
{
    type Error = anyhow::Error;

    fn try_from(pointer: *const FFI_STRUCT) -> Result<Self, Self::Error> {
        let raw = pointer as usize;
        let tag = raw & Self::TAG_MASK;
        if tag != TAG {
            bail!(
                "not a {} pointer: {:?}",
                std::any::type_name::<FFI_STRUCT>(),
                pointer as *const c_void
            );
        }

        Ok(Self {
            raw,
            _ffi_struct: Default::default(),
        })
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> TryFrom<*mut FFI_STRUCT>
    for Handle<FFI_STRUCT, TAG_BITS, TAG>
{
    type Error = anyhow::Error;

    fn try_from(pointer: *mut FFI_STRUCT) -> Result<Self, Self::Error> {
        let raw = pointer as usize;
        let tag = raw & Self::TAG_MASK;
        if tag != TAG {
            bail!(
                "not a {} pointer: {:?}",
                std::any::type_name::<FFI_STRUCT>(),
                pointer as *const c_void
            );
        }

        Ok(Self {
            raw,
            _ffi_struct: Default::default(),
        })
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> From<Handle<FFI_STRUCT, TAG_BITS, TAG>>
    for *const FFI_STRUCT
{
    fn from(value: Handle<FFI_STRUCT, TAG_BITS, TAG>) -> Self {
        value.raw as *const FFI_STRUCT
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> From<Handle<FFI_STRUCT, TAG_BITS, TAG>>
    for *mut FFI_STRUCT
{
    fn from(value: Handle<FFI_STRUCT, TAG_BITS, TAG>) -> Self {
        value.raw as *mut FFI_STRUCT
    }
}

impl<FFI_STRUCT, const TAG_BITS: usize, const TAG: usize> From<Handle<FFI_STRUCT, TAG_BITS, TAG>>
    for usize
{
    fn from(value: Handle<FFI_STRUCT, TAG_BITS, TAG>) -> Self {
        value.raw
    }
}
