use crate::context::{ContextType, DecodeContext, EncodeContext};
use crate::frame::AvFrame;
use crate::packet::AvPacket;
pub use crate::sys::AVCodecID as AvCodecId;
pub use crate::sys::AVPixelFormat as AvPixelFormat;
use crate::sys::{
    avcodec_alloc_context3, avcodec_find_decoder, avcodec_find_encoder, avcodec_open2,
    avcodec_parameters_alloc, avcodec_parameters_copy, avcodec_parameters_free,
    avcodec_parameters_from_context, avcodec_parameters_to_context, avcodec_receive_frame,
    avcodec_receive_packet, avcodec_send_frame, avcodec_send_packet, AVCodec, AVCodecContext,
    AVCodecParameters, AV_CODEC_FLAG_4MV, AV_CODEC_FLAG_AC_PRED, AV_CODEC_FLAG_BITEXACT,
    AV_CODEC_FLAG_CLOSED_GOP, AV_CODEC_FLAG_DROPCHANGED, AV_CODEC_FLAG_GLOBAL_HEADER,
    AV_CODEC_FLAG_GRAY, AV_CODEC_FLAG_INTERLACED_DCT, AV_CODEC_FLAG_INTERLACED_ME,
    AV_CODEC_FLAG_LOOP_FILTER, AV_CODEC_FLAG_LOW_DELAY, AV_CODEC_FLAG_OUTPUT_CORRUPT,
    AV_CODEC_FLAG_PASS1, AV_CODEC_FLAG_PASS2, AV_CODEC_FLAG_PSNR, AV_CODEC_FLAG_QPEL,
    AV_CODEC_FLAG_QSCALE, AV_CODEC_FLAG_UNALIGNED,
};
use crate::util::property;
use crate::util::AvRational;
use crate::{AvMediaType, AvOwnable, AvOwned, AvResult};
use bitflags::bitflags;
use std::marker::PhantomData;
use std::os::raw::{c_int, c_uint};
use std::{mem, ptr};

bitflags! {
    pub struct CodecFlags: u32 {
        const UNALIGNED = AV_CODEC_FLAG_UNALIGNED;
        const QSCALE = AV_CODEC_FLAG_QSCALE;
        const _4MV = AV_CODEC_FLAG_4MV;
        const OUTPUT_CORRUPT = AV_CODEC_FLAG_OUTPUT_CORRUPT;
        const QPEL = AV_CODEC_FLAG_QPEL;
        const DROP_CHANGED = AV_CODEC_FLAG_DROPCHANGED;
        const PASS1 = AV_CODEC_FLAG_PASS1;
        const PASS2 = AV_CODEC_FLAG_PASS2;
        const LOOP_FILTER = AV_CODEC_FLAG_LOOP_FILTER;
        const GRAY = AV_CODEC_FLAG_GRAY;
        const PSNR = AV_CODEC_FLAG_PSNR;
        const INTERLACED_DCT = AV_CODEC_FLAG_INTERLACED_DCT;
        const LOW_DELAY = AV_CODEC_FLAG_LOW_DELAY;
        const GLOBAL_HEADER = AV_CODEC_FLAG_GLOBAL_HEADER;
        const BIT_EXACT = AV_CODEC_FLAG_BITEXACT;
        const AC_PRED = AV_CODEC_FLAG_AC_PRED;
        const INTERLACED_ME = AV_CODEC_FLAG_INTERLACED_ME;
        const CLOSED_GOP = AV_CODEC_FLAG_CLOSED_GOP;
    }
}

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

    pub fn codec_tag(&self) -> u32 {
        unsafe { (*self.params).codec_tag }
    }

    pub fn set_codec_tag(&mut self, value: u32) {
        unsafe {
            (*self.params).codec_tag = value;
        }
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

    pub fn pix_fmt(&self) -> AvPixelFormat {
        unsafe { mem::transmute((*self.params).format as i32) }
    }

    pub fn set_pix_fmt(&self, value: AvPixelFormat) {
        unsafe {
            (*self.params).format = value as i32;
        }
    }

    pub fn copy_from(&mut self, src: &AvCodecParameters) -> AvResult<()> {
        unsafe {
            let result = avcodec_parameters_copy(self.params, src.params);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn copy_from_context<T: ContextType>(&mut self, src: &AvCodecContext<T>) -> AvResult<()> {
        unsafe {
            let result = avcodec_parameters_from_context(self.params, src.ptr);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
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

impl<T> AvCodec<T> {
    pub fn pix_fmts(&self) -> Vec<AvPixelFormat> {
        unsafe {
            let mut ptr: *const AvPixelFormat = (*self.codec).pix_fmts;
            let mut values = Vec::new();

            if !ptr.is_null() {
                loop {
                    let value = *ptr;
                    if value == AvPixelFormat::None {
                        break;
                    }

                    values.push(value);
                    ptr = ptr.offset(1);
                }
            }

            values
        }
    }
}

pub struct AvCodecContext<T: ContextType> {
    ptr: *mut AVCodecContext,
    _type: PhantomData<T>,
}

impl<T: ContextType> AvCodecContext<T> {
    pub fn alloc(codec: Option<&AvCodec<T>>) -> Option<Self> {
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

    pub fn use_parameters(&mut self, params: &AvCodecParameters) -> AvResult<()> {
        unsafe {
            let result = avcodec_parameters_to_context(self.ptr, params.params);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn open(&mut self, codec: Option<&AvCodec<T>>) -> AvResult<()> {
        unsafe {
            let codec = match codec {
                None => ptr::null(),
                Some(val) => val.codec,
            };

            let result = avcodec_open2(self.ptr, codec, ptr::null_mut());
            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    property!(codec_type, AvMediaType);
    property!(codec_id, AvCodecId);
    property!(codec_tag, c_uint);

    pub fn codec(&self) -> AvCodec<T> {
        unsafe {
            let codec = (*self.ptr).codec;
            AvCodec {
                codec,
                _ctx: Default::default(),
            }
        }
    }

    property!(bit_rate, i64);

    pub fn time_base(&self) -> AvRational {
        unsafe { AvRational::from((*self.ptr).time_base) }
    }

    pub fn set_time_base(&mut self, value: AvRational) {
        unsafe {
            (*self.ptr).flags = 1;
            (*self.ptr).time_base = value.into();
        }
    }

    property!(width, c_int);
    property!(height, c_int);

    pub fn sample_aspect_ratio(&self) -> AvRational {
        unsafe { AvRational::from((*self.ptr).sample_aspect_ratio) }
    }

    pub fn set_sample_aspect_ratio(&mut self, value: AvRational) {
        unsafe {
            (*self.ptr).sample_aspect_ratio = value.into();
        }
    }

    property!(pix_fmt, AvPixelFormat);

    pub fn framerate(&self) -> AvRational {
        unsafe { AvRational::from((*self.ptr).framerate) }
    }

    pub fn set_framerate(&mut self, value: AvRational) {
        unsafe {
            (*self.ptr).framerate = value.into();
        }
    }

    pub fn flags(&self) -> CodecFlags {
        unsafe { CodecFlags::from_bits_unchecked((*self.ptr).flags as u32) }
    }

    pub fn set_flags(&mut self, value: CodecFlags) {
        unsafe {
            (*self.ptr).flags = value.bits as c_int;
        }
    }
}

impl AvCodecContext<DecodeContext> {
    pub fn send_packet(&mut self, packet: &AvPacket) -> AvResult<()> {
        unsafe {
            let result = avcodec_send_packet(self.ptr, packet.ptr);
            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn receive_frame(&mut self, target: &AvFrame) -> AvResult<()> {
        unsafe {
            let result = avcodec_receive_frame(self.ptr, target.ptr);
            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }
}

impl AvCodecContext<EncodeContext> {
    pub fn send_frame(&mut self, frame: Option<&AvFrame>) -> AvResult<()> {
        unsafe {
            let frame = match frame {
                None => ptr::null(),
                Some(value) => value.ptr,
            };

            let result = avcodec_send_frame(self.ptr, frame);
            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn receive_packet(&mut self, target: &AvPacket) -> AvResult<()> {
        unsafe {
            let result = avcodec_receive_packet(self.ptr, target.ptr);
            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }
}
