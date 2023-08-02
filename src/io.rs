use crate::sys::{
    avio_close, avio_closep, avio_flush, avio_open, avio_open_dyn_buf, AVIOContext,
    AVIO_FLAG_DIRECT, AVIO_FLAG_NONBLOCK, AVIO_FLAG_READ, AVIO_FLAG_WRITE,
};
use crate::util::map_to_cstr;
use crate::AvResult;
use bitflags::{bitflags, Flags};
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
    pub fn open(&mut self, url: &str, flags: OpenFlags) -> AvResult<()> {
        unsafe {
            let ptr = self.ptr as *mut *mut AVIOContext;
            let url = map_to_cstr(url);

            let result = avio_open(ptr, url.as_ptr(), flags.bits() as c_int);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn open_dyn_buf(&mut self) -> AvResult<()> {
        unsafe {
            let ptr = self.ptr as *mut *mut AVIOContext;

            let result = avio_open_dyn_buf(ptr);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn close(&mut self) -> AvResult<()> {
        unsafe {
            let ptr = *self.ptr;
            let result = avio_close(ptr);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn closep(&mut self) -> AvResult<()> {
        unsafe {
            let ptr = self.ptr as *mut *mut AVIOContext;
            let result = avio_closep(ptr);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn flush(&mut self) {
        unsafe {
            let ptr = *self.ptr;
            avio_flush(ptr);
        }
    }
}
