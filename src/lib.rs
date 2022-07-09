use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_int;
use thiserror::Error;

pub mod codec;
pub mod context;
pub mod dict;
pub mod format;
pub mod frame;
pub mod io;
pub mod packet;
pub mod stream;
mod sys;
pub mod util;

pub use sys::AVMediaType as AvMediaType;

pub type AvResult<T> = Result<T, AvError>;

#[derive(Error, Debug)]
pub enum AvError {
    #[error("Misc IO Error `{0}`")]
    MiscIoError(#[from] std::io::Error),
    #[error("Resource temporarily unavailable")]
    Again,
    #[error("Device or resource busy")]
    Busy,
    #[error("Connection refused")]
    ConnectionRefused,
    #[error("Connection timed out")]
    ConnectionTimeOut,
    #[error("Numerical argument out of domain")]
    NumericalDomain,
    #[error("File too large")]
    FileTooLarge,
    #[error("Invalid argument")]
    InvalidArgument,
    #[error("No such device")]
    NoSuchDevice,
    #[error("No such file/directory")]
    NoSuchFile,
    #[error("Not implemented")]
    NotImplemented,
    #[error("Protocol not supported")]
    ProtocolUnsupported,
    #[error("Bitstream filter not found")]
    BsfNotFound,
    #[error("Internal bug")]
    Bug,
    #[error("Buffer too small")]
    BufferTooSmall,
    #[error("Decoder not found")]
    DecoderNotFound,
    #[error("Demuxer not found")]
    DemuxerNotFound,
    #[error("Encoder not found")]
    EncoderNotFound,
    #[error("End of file")]
    EndOfFile,
    #[error("Immediate exit was requested")]
    ExitRequested,
    #[error("Generic error in an external library")]
    ExternalError,
    #[error("Filter not found")]
    FilterNotFound,
    #[error("Invalid data found when processing input")]
    InvalidData,
    #[error("Muxer not found")]
    MuxerNotFound,
    #[error("Option not found")]
    OptionNotFound,
    #[error("Not yet implemented in FFmpeg, patches welcome")]
    PatchWelcome,
    #[error("Protocol not found")]
    ProtocolNotFound,
    #[error("Stream not found")]
    StreamNotFound,
    #[error("Internal bug")]
    Bug2,
    #[error("Unknown error, typically from an external library")]
    Unknown,
    #[error("Requested feature is flagged experimental")]
    Experimental,
    #[error("Input changed between calls. Reconfiguration is required")]
    InputChanged,
    #[error("Output changed between calls. Reconfiguration is required")]
    OutputChanged,
    #[error("Http 400")]
    HttpBadRequest,
    #[error("Http 401")]
    HttpUnauthorized,
    #[error("Http 403")]
    HttpForbidden,
    #[error("Http 404")]
    HttpNotFound,
    #[error("Http 4XX")]
    HttpOther400,
    #[error("Http 5XX")]
    HttpServerError,
}

impl From<c_int> for AvError {
    fn from(value: c_int) -> Self {
        if value >= 0 {
            panic!("Unexpected error value {}", value);
        }

        match (-value) as u32 {
            sys::EAGAIN => AvError::Again,
            sys::EBUSY => AvError::Busy,
            sys::ECONNREFUSED => AvError::ConnectionRefused,
            sys::ETIMEDOUT => AvError::ConnectionTimeOut,
            sys::EDOM => AvError::NumericalDomain,
            sys::EFBIG => AvError::FileTooLarge,
            sys::EINVAL => AvError::InvalidArgument,
            sys::ENODEV => AvError::NoSuchDevice,
            sys::ENOENT => AvError::NoSuchFile,
            sys::ENOSYS => AvError::NotImplemented,
            sys::EPROTONOSUPPORT => AvError::ProtocolUnsupported,
            sys::AVERROR_BSF_NOT_FOUND => AvError::BsfNotFound,
            sys::AVERROR_BUG => AvError::Bug,
            sys::AVERROR_BUFFER_TOO_SMALL => AvError::BufferTooSmall,
            sys::AVERROR_DECODER_NOT_FOUND => AvError::DecoderNotFound,
            sys::AVERROR_DEMUXER_NOT_FOUND => AvError::DemuxerNotFound,
            sys::AVERROR_ENCODER_NOT_FOUND => AvError::EncoderNotFound,
            sys::AVERROR_EOF => AvError::EndOfFile,
            sys::AVERROR_EXIT => AvError::ExitRequested,
            sys::AVERROR_EXTERNAL => AvError::ExternalError,
            sys::AVERROR_FILTER_NOT_FOUND => AvError::FilterNotFound,
            sys::AVERROR_INVALIDDATA => AvError::InvalidData,
            sys::AVERROR_MUXER_NOT_FOUND => AvError::MuxerNotFound,
            sys::AVERROR_OPTION_NOT_FOUND => AvError::OptionNotFound,
            sys::AVERROR_PATCHWELCOME => AvError::PatchWelcome,
            sys::AVERROR_PROTOCOL_NOT_FOUND => AvError::ProtocolNotFound,
            sys::AVERROR_STREAM_NOT_FOUND => AvError::StreamNotFound,
            sys::AVERROR_BUG2 => AvError::Bug2,
            sys::AVERROR_UNKNOWN => AvError::Unknown,
            sys::AVERROR_EXPERIMENTAL => AvError::Experimental,
            sys::AVERROR_INPUT_CHANGED => AvError::InputChanged,
            sys::AVERROR_OUTPUT_CHANGED => AvError::OutputChanged,
            sys::AVERROR_HTTP_BAD_REQUEST => AvError::HttpBadRequest,
            sys::AVERROR_HTTP_UNAUTHORIZED => AvError::HttpUnauthorized,
            sys::AVERROR_HTTP_FORBIDDEN => AvError::HttpForbidden,
            sys::AVERROR_HTTP_NOT_FOUND => AvError::HttpNotFound,
            sys::AVERROR_HTTP_OTHER_4XX => AvError::HttpOther400,
            sys::AVERROR_HTTP_SERVER_ERROR => AvError::HttpServerError,
            _ => AvError::MiscIoError(std::io::Error::from_raw_os_error(value)),
        }
    }
}

pub trait AvOwnable {
    fn drop(&mut self);
}

pub struct AvOwned<T: AvOwnable> {
    inner: T,
}

impl<T: AvOwnable> AvOwned<T> {
    pub fn wrap(value: T) -> Self {
        Self { inner: value }
    }
}

impl<T: AvOwnable> Deref for AvOwned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AvOwnable> DerefMut for AvOwned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: AvOwnable> Drop for AvOwned<T> {
    fn drop(&mut self) {
        T::drop(self)
    }
}

pub struct AvBorrow<'a, S: 'a, T> {
    _src: PhantomData<&'a S>,
    value: T,
}

impl<'a, S: 'a, T> AvBorrow<'a, S, T> {
    pub fn wrap(_src: &'a S, value: T) -> Self {
        Self {
            _src: Default::default(),
            value,
        }
    }
}

impl<'a, S: 'a, T> Deref for AvBorrow<'a, S, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub struct AvBorrowMut<'a, S: 'a, T> {
    _src: PhantomData<&'a mut S>,
    value: T,
}

impl<'a, S: 'a, T> AvBorrowMut<'a, S, T> {
    pub fn wrap(_src: &'a mut S, value: T) -> Self {
        Self {
            _src: Default::default(),
            value,
        }
    }
}

impl<'a, S: 'a, T> Deref for AvBorrowMut<'a, S, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, S: 'a, T> DerefMut for AvBorrowMut<'a, S, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
