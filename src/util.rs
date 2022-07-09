use crate::sys::AVRational;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;

pub(crate) fn map_to_cstr(src: &str) -> CString {
    CString::new(src.as_bytes()).unwrap()
}

pub(crate) fn map_to_cstr_optional(src: Option<&str>) -> Option<CString> {
    src.map(|str| CString::new(str.as_bytes()).unwrap())
}

pub(crate) fn cstr_optional_to_ptr(src: &Option<CString>) -> *const c_char {
    src.as_ref().map_or(ptr::null(), |str| str.as_ptr())
}

#[derive(Debug, Copy, Clone)]
pub struct AvRational {
    pub num: c_int,
    pub den: c_int,
}

impl AvRational {
    pub fn inverse(&self) -> Self {
        Self {
            num: self.den,
            den: self.num,
        }
    }
}

impl From<AVRational> for AvRational {
    fn from(value: AVRational) -> Self {
        Self {
            num: value.num,
            den: value.den,
        }
    }
}

impl Into<AVRational> for AvRational {
    fn into(self) -> AVRational {
        AVRational {
            num: self.num,
            den: self.den,
        }
    }
}

macro_rules! property {
    ($name:ident, $ret:ty) => {
        paste::paste! {
            pub fn $name(&self) -> $ret {
                unsafe {
                    (*self.ptr).$name
                }
            }

            pub fn [<set_ $name>](&mut self, value: $ret) {
                unsafe {
                    (*self.ptr).$name = value;
                }
            }
        }
    };
}

pub(crate) use property;
