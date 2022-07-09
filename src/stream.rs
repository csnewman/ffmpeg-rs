use crate::codec::AvCodecParameters;
use crate::context::ContextType;
use crate::frame::AvFrame;
use crate::sys::{av_guess_frame_rate, AVFormatContext, AVStream};
use crate::util::AvRational;
use crate::{AvBorrow, AvBorrowMut};
use std::marker::PhantomData;
use std::ptr;

pub struct AvStream<T: ContextType> {
    format: *mut AVFormatContext,
    stream: *mut AVStream,
    _ctx: PhantomData<T>,
}

impl<T: ContextType> AvStream<T> {
    pub fn wrap(format: *mut AVFormatContext, stream: *mut AVStream) -> Self {
        Self {
            format,
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

    pub fn guess_frame_rate(&self, frame: Option<&AvFrame>) -> Option<AvRational> {
        unsafe {
            let frame = match frame {
                None => ptr::null_mut(),
                Some(value) => value.ptr,
            };

            let result = av_guess_frame_rate(self.format, self.stream, frame);

            if result.num == 0 && result.den == 1 {
                None
            } else {
                Some(result.into())
            }
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
