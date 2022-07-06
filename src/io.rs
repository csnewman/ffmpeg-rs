use crate::sys::{
    avio_closep, avio_open, AVIOContext, AVIO_FLAG_DIRECT, AVIO_FLAG_NONBLOCK, AVIO_FLAG_READ,
    AVIO_FLAG_WRITE,
};
use crate::util::map_to_cstr;
use crate::{wrap_error, AvError};
use bitflags::bitflags;
use std::os::raw::c_int;

bitflags! {
    pub struct OpenFlags: u32 {
        const READ = AVIO_FLAG_READ;
        const WRITE = AVIO_FLAG_WRITE;
        const NONBLOCK = AVIO_FLAG_NONBLOCK;
        const DIRECT = AVIO_FLAG_DIRECT;
    }
}

pub struct AvIoContext {
    pub(crate) ptr: *const *mut AVIOContext,
}

impl AvIoContext {
    pub fn open(&mut self, url: &str, flags: OpenFlags) -> Result<(), AvError> {
        unsafe {
            let ptr = self.ptr as *mut *mut AVIOContext;
            let url = map_to_cstr(url);

            let result = avio_open(ptr, url.as_ptr(), flags.bits as c_int);

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
            }
        }
    }

    pub fn closep(&mut self) -> Result<(), AvError> {
        unsafe {
            let ptr = self.ptr as *mut *mut AVIOContext;
            let result = avio_closep(ptr);

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
            }
        }
    }
}
