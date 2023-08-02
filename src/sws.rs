use crate::codec::AvPixelFormat;
use crate::frame::AvFrame;
use crate::sys::{
    sws_freeContext, sws_getContext, sws_scale_frame, SWS_AREA, SWS_BICUBIC, SWS_BICUBLIN,
    SWS_BILINEAR, SWS_FAST_BILINEAR, SWS_GAUSS, SWS_LANCZOS, SWS_POINT, SWS_SINC, SWS_SPLINE,
    SWS_X,
};
use crate::{sys, AvResult};
use bitflags::{bitflags, Flags};
use std::os::raw::c_int;
use std::ptr;

bitflags! {
    pub struct SwsFlags: u32 {
        const FAST_BILINEAR = SWS_FAST_BILINEAR;
        const BILINEAR = SWS_BILINEAR;
        const BICUBIC = SWS_BICUBIC;
        const X = SWS_X;
        const POINT = SWS_POINT;
        const AREA = SWS_AREA;
        const BICUBLIN = SWS_BICUBLIN;
        const GAUSS = SWS_GAUSS;
        const SINC = SWS_SINC;
        const LANCZOS = SWS_LANCZOS;
        const SPLINE = SWS_SPLINE;
    }
}

pub struct SwsContext {
    ptr: *mut sys::SwsContext,
}

impl SwsContext {
    pub fn get(
        srcW: c_int,
        srcH: c_int,
        srcFormat: AvPixelFormat,
        dstW: c_int,
        dstH: c_int,
        dstFormat: AvPixelFormat,
        flags: SwsFlags,
    ) -> Option<SwsContext> {
        unsafe {
            let ctx = sws_getContext(
                srcW,
                srcH,
                srcFormat,
                dstW,
                dstH,
                dstFormat,
                flags.bits() as c_int,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            );
            if ctx.is_null() {
                None
            } else {
                Some(Self { ptr: ctx })
            }
        }
    }

    pub fn scale_frame(&self, dst: &mut AvFrame, src: &AvFrame) -> AvResult<()> {
        unsafe {
            let result = sws_scale_frame(self.ptr, dst.ptr, src.ptr);
            if result.is_negative() {
                Err(result.into())
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for SwsContext {
    fn drop(&mut self) {
        unsafe { sws_freeContext(self.ptr) }
    }
}
