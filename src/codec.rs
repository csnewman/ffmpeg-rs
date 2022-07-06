use crate::context::{ContextType, DecodeContext, EncodeContext};
use crate::sys::{
    avcodec_alloc_context3, avcodec_find_decoder, avcodec_find_encoder, avcodec_open2,
    avcodec_parameters_alloc, avcodec_parameters_copy, avcodec_parameters_free,
    avcodec_parameters_to_context, AVCodec, AVCodecContext, AVCodecParameters,
};
use crate::{wrap_error, AvError, AvMediaType, AvOwnable, AvOwned};
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::ptr;

pub use crate::sys::AVCodecID as AvCodecId;

pub struct AvCodecParameters {
    pub(crate) params: *mut AVCodecParameters,
}

impl AvCodecParameters {
    pub fn alloc() -> AvOwned<Self> {
        unsafe {
            let params = avcodec_parameters_alloc();

            AvOwned::wrap(Self { params })
        }
    }

    pub fn codec_type(&self) -> AvMediaType {
        unsafe { (*self.params).codec_type }
    }

    pub fn codec_id(&self) -> AvCodecId {
        unsafe { (*self.params).codec_id }
    }

    pub fn width(&self) -> c_int {
        unsafe { (*self.params).width }
    }

    pub fn height(&self) -> c_int {
        unsafe { (*self.params).height }
    }

    pub fn sample_rate(&self) -> c_int {
        unsafe { (*self.params).sample_rate }
    }

    pub fn copy_from(&mut self, src: &AvCodecParameters) -> Result<(), AvError> {
        unsafe {
            let result = avcodec_parameters_copy(self.params, src.params);

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
            }
        }
    }
}

impl AvOwnable for AvCodecParameters {
    fn drop(&mut self) {
        unsafe { avcodec_parameters_free(&mut self.params) }
    }
}

#[derive(Copy, Clone)]
pub struct AvCodec<T> {
    codec: *const AVCodec,
    _ctx: PhantomData<T>,
}

impl AvCodec<DecodeContext> {
    pub fn find_decoder(id: AvCodecId) -> Option<Self> {
        unsafe {
            let codec = avcodec_find_decoder(id);
            if codec.is_null() {
                None
            } else {
                Some(Self {
                    codec,
                    _ctx: Default::default(),
                })
            }
        }
    }
}

impl AvCodec<EncodeContext> {
    pub fn find_encoder(id: AvCodecId) -> Option<Self> {
        unsafe {
            let codec = avcodec_find_encoder(id);
            if codec.is_null() {
                None
            } else {
                Some(Self {
                    codec,
                    _ctx: Default::default(),
                })
            }
        }
    }
}

pub struct AvCodecContext<T: ContextType> {
    ptr: *mut AVCodecContext,
    _type: PhantomData<T>,
}

impl<T: ContextType> AvCodecContext<T> {
    pub fn alloc(codec: Option<AvCodec<T>>) -> Option<Self> {
        unsafe {
            let codec = match codec {
                None => ptr::null(),
                Some(val) => val.codec,
            };

            let ctx = avcodec_alloc_context3(codec);
            if ctx.is_null() {
                None
            } else {
                Some(Self {
                    ptr: ctx,
                    _type: Default::default(),
                })
            }
        }
    }

    pub fn use_parameters(&mut self, params: &AvCodecParameters) -> Result<(), AvError> {
        unsafe {
            let result = avcodec_parameters_to_context(self.ptr, params.params);

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
            }
        }
    }

    pub fn open(&mut self, codec: Option<AvCodec<T>>) -> Result<(), AvError> {
        unsafe {
            let codec = match codec {
                None => ptr::null(),
                Some(val) => val.codec,
            };

            let result = avcodec_open2(self.ptr, codec, ptr::null_mut());
            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
            }
        }
    }

    pub fn codec_type(&self) -> AvMediaType {
        unsafe { (*self.ptr).codec_type }
    }

    pub fn codec_id(&self) -> AvCodecId {
        unsafe { (*self.ptr).codec_id }
    }

    pub fn codec(&self) -> AvCodec<T> {
        unsafe {
            let codec = (*self.ptr).codec;
            AvCodec {
                codec,
                _ctx: Default::default(),
            }
        }
    }
}
