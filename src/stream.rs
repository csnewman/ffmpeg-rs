use crate::codec::AvCodecParameters;
use crate::context::ContextType;
use crate::format::AvFormatContext;
use crate::sys::AVStream;
use crate::util::AvRational;
use crate::{AvBorrow, AvBorrowMut};
use std::marker::PhantomData;

pub struct AvStream<T: ContextType> {
    stream: *mut AVStream,
    _ctx: PhantomData<AvFormatContext<T>>,
}

impl<T: ContextType> AvStream<T> {
    pub fn wrap(stream: *mut AVStream) -> Self {
        Self {
            stream,
            _ctx: Default::default(),
        }
    }

    pub fn codec_parameters(&self) -> AvBorrow<AvStream<T>, AvCodecParameters> {
        unsafe {
            AvBorrow::wrap(
                self,
                AvCodecParameters {
                    params: (*self.stream).codecpar,
                },
            )
        }
    }

    pub fn codec_parameters_mut(&mut self) -> AvBorrowMut<AvStream<T>, AvCodecParameters> {
        unsafe {
            AvBorrowMut::wrap(
                self,
                AvCodecParameters {
                    params: (*self.stream).codecpar,
                },
            )
        }
    }

    pub fn time_base(&self) -> AvRational {
        unsafe { AvRational::from((*self.stream).time_base) }
    }

    pub fn set_time_base(&mut self, value: AvRational) {
        unsafe {
            (*self.stream).time_base = value.into();
        }
    }
}
