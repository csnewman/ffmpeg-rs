use crate::sys::{
    av_dict_count, av_dict_free, av_dict_set, AVDictionary, AV_DICT_APPEND, AV_DICT_DONT_OVERWRITE,
    AV_DICT_IGNORE_SUFFIX, AV_DICT_MATCH_CASE, AV_DICT_MULTIKEY,
};
use crate::util::map_to_cstr;
use crate::{wrap_error, AvError, AvOwnable, AvOwned};
use bitflags::bitflags;
use std::ptr;

bitflags! {
    pub struct DictInsertFlags: u32 {
        const DONT_OVERWRITE = AV_DICT_DONT_OVERWRITE;
        const APPEND = AV_DICT_APPEND;
        const MULTIKEY = AV_DICT_MULTIKEY;
    }

    pub struct DictFetchFlags: u32 {
        const MATCH_CASE = AV_DICT_MATCH_CASE;
        const IGNORE_SUFFIX = AV_DICT_IGNORE_SUFFIX;
    }
}

pub struct AvDictionary {
    pub(crate) ptr: *mut AVDictionary,
}

impl AvDictionary {
    pub fn empty() -> AvOwned<AvDictionary> {
        AvOwned::wrap(Self {
            ptr: ptr::null_mut(),
        })
    }

    pub fn count(&self) -> usize {
        unsafe { av_dict_count(self.ptr) as usize }
    }

    pub fn set(&mut self, key: &str, value: &str, flags: DictInsertFlags) -> Result<(), AvError> {
        unsafe {
            let key = map_to_cstr(key);
            let value = map_to_cstr(value);
            let result = av_dict_set(
                &mut self.ptr,
                key.as_ptr(),
                value.as_ptr(),
                flags.bits as i32,
            );

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
            }
        }
    }
}

impl AvOwnable for AvDictionary {
    fn drop(&mut self) {
        unsafe { av_dict_free(&mut self.ptr) }
    }
}
