use crate::context::{ContextType, InputContext, OutputContext};
use crate::dict::AvDictionary;
use crate::io::AvIoContext;
use crate::packet::AvPacket;
use crate::stream::AvStream;
use crate::sys::{
    av_guess_format, av_interleaved_write_frame, av_read_frame, av_write_frame, av_write_trailer,
    avformat_alloc_output_context2, avformat_find_stream_info, avformat_init_output,
    avformat_new_stream, avformat_open_input, avformat_write_header, AVDictionary, AVFormatContext,
    AVIOContext, AVOutputFormat, AVFMT_ALLOW_FLUSH, AVFMT_GLOBALHEADER, AVFMT_NEEDNUMBER,
    AVFMT_NODIMENSIONS, AVFMT_NOFILE, AVFMT_NOSTREAMS, AVFMT_NOTIMESTAMPS, AVFMT_TS_NEGATIVE,
    AVFMT_TS_NONSTRICT, AVFMT_VARIABLE_FPS,
};
use crate::util::{cstr_optional_to_ptr, map_to_cstr_optional};
use crate::{AvBorrow, AvBorrowMut, AvResult};
use bitflags::bitflags;
use std::marker::PhantomData;
use std::ptr;

pub struct AvFormatContext<T: ContextType> {
    pub(crate) ptr: *mut AVFormatContext,
    _type: PhantomData<T>,
}

impl AvFormatContext<InputContext> {
    pub fn open_input(url: Option<&str>) -> AvResult<Self> {
        unsafe {
            let mut ctx = ptr::null_mut();
            let url = map_to_cstr_optional(url);

            let result = avformat_open_input(
                &mut ctx,
                cstr_optional_to_ptr(&url),
                ptr::null(),
                ptr::null_mut(),
            );

            match result {
                0 => Ok(Self {
                    ptr: ctx,
                    _type: Default::default(),
                }),
                val => Err(val.into()),
            }
        }
    }

    pub fn find_stream_info(&self, options: Option<&[&AvDictionary]>) -> AvResult<()> {
        unsafe {
            let options: Option<Vec<*mut AVDictionary>> = match options {
                None => None,
                Some(value) => Some(value.iter().map(|d| d.ptr).collect()),
            };

            let options_ptr = match options.as_ref() {
                None => ptr::null_mut(),
                Some(value) => value.as_ptr() as *mut *mut AVDictionary,
            };

            let result = avformat_find_stream_info(self.ptr, options_ptr);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn read_frame(&mut self, dest: &mut AvPacket) -> AvResult<()> {
        unsafe {
            let result = av_read_frame(self.ptr, dest.ptr);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }
}

impl<T: ContextType> Drop for AvFormatContext<T> {
    fn drop(&mut self) {
        T::drop_format_context(self);
    }
}

impl AvFormatContext<OutputContext> {
    pub fn alloc_output(
        oformat: Option<&AvOutputFormat>,
        format_name: Option<&str>,
        filename: Option<&str>,
    ) -> AvResult<Self> {
        unsafe {
            let mut ctx = ptr::null_mut();
            let oformat = match oformat {
                None => ptr::null(),
                Some(value) => value.ptr,
            };
            let format_name = map_to_cstr_optional(format_name);
            let filename = map_to_cstr_optional(filename);

            let result = avformat_alloc_output_context2(
                &mut ctx,
                oformat,
                cstr_optional_to_ptr(&format_name),
                cstr_optional_to_ptr(&filename),
            );

            match result {
                0 => Ok(Self {
                    ptr: ctx,
                    _type: Default::default(),
                }),
                val => Err(val.into()),
            }
        }
    }

    pub fn init_output(&mut self, options: Option<&mut AvDictionary>) -> AvResult<()> {
        unsafe {
            let options = match options {
                None => ptr::null_mut(),
                Some(value) => &mut value.ptr,
            };
            let result = avformat_init_output(self.ptr, options);

            if result.is_negative() {
                Err(result.into())
            } else {
                Ok(())
            }
        }
    }

    pub fn write_header(&mut self) -> AvResult<()> {
        unsafe {
            let result = avformat_write_header(self.ptr, ptr::null_mut());

            if result.is_negative() {
                Err(result.into())
            } else {
                Ok(())
            }
        }
    }

    pub fn interleaved_write_frame(&self, packet: &AvPacket) -> AvResult<()> {
        unsafe {
            let result = av_interleaved_write_frame(self.ptr, packet.ptr);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }

    pub fn write_frame(&self, packet: Option<&AvPacket>) -> AvResult<()> {
        unsafe {
            let packet = match packet {
                None => ptr::null_mut(),
                Some(value) => value.ptr,
            };

            let result = av_write_frame(self.ptr, packet);

            if result.is_negative() {
                Err(result.into())
            } else {
                Ok(())
            }
        }
    }

    pub fn write_trailer(&mut self) -> AvResult<()> {
        unsafe {
            let result = av_write_trailer(self.ptr);

            match result {
                0 => Ok(()),
                val => Err(val.into()),
            }
        }
    }
}

impl<T: ContextType> AvFormatContext<T> {
    pub fn nb_streams(&self) -> usize {
        unsafe { (*self.ptr).nb_streams as usize }
    }

    pub fn stream(&self, id: usize) -> Option<AvBorrow<Self, AvStream<T>>> {
        unsafe {
            if id >= self.nb_streams() {
                return None;
            }

            let stream = *(*self.ptr).streams.add(id);
            Some(AvBorrow::wrap(&self, AvStream::wrap(self.ptr, stream)))
        }
    }

    pub fn stream_mut(&mut self, id: usize) -> Option<AvBorrowMut<Self, AvStream<T>>> {
        unsafe {
            if id >= self.nb_streams() {
                return None;
            }

            let stream = *(*self.ptr).streams.add(id);
            Some(AvBorrowMut::wrap(self, AvStream::wrap(self.ptr, stream)))
        }
    }

    pub fn new_stream(&mut self) -> Option<AvBorrowMut<Self, AvStream<T>>> {
        unsafe {
            let stream = avformat_new_stream(self.ptr, ptr::null());
            if stream.is_null() {
                None
            } else {
                Some(AvBorrowMut::wrap(self, AvStream::wrap(self.ptr, stream)))
            }
        }
    }

    pub fn pb(&self) -> AvBorrow<Self, AvIoContext> {
        unsafe {
            let ptr = (&mut (*self.ptr).pb) as *mut *mut AVIOContext;

            AvBorrow::wrap(&self, AvIoContext { ptr })
        }
    }

    pub fn pb_mut(&mut self) -> AvBorrowMut<Self, AvIoContext> {
        unsafe {
            let ptr = (&mut (*self.ptr).pb) as *mut *mut AVIOContext;

            AvBorrowMut::wrap(self, AvIoContext { ptr })
        }
    }
}

bitflags! {
    pub struct OutputFlags: u32 {
        const NO_FILE = AVFMT_NOFILE;
        const NEED_NUMBER = AVFMT_NEEDNUMBER;
        const GLOBAL_HEADER = AVFMT_GLOBALHEADER;
        const NO_TIMESTAMPS = AVFMT_NOTIMESTAMPS;
        const VARIABLE_FPS = AVFMT_VARIABLE_FPS;
        const NO_DIMENSIONS = AVFMT_NODIMENSIONS;
        const NO_STREAMS = AVFMT_NOSTREAMS;
        const ALLOW_FLUSH = AVFMT_ALLOW_FLUSH;
        const TS_NON_STRICT = AVFMT_TS_NONSTRICT;
        const TS_NEGATIVE = AVFMT_TS_NEGATIVE;
    }
}

pub struct AvOutputFormat {
    ptr: *const AVOutputFormat,
}

impl AvOutputFormat {
    pub fn guess_format(
        short_name: Option<&str>,
        filename: Option<&str>,
        mime_type: Option<&str>,
    ) -> Option<Self> {
        unsafe {
            let short_name = map_to_cstr_optional(short_name);
            let filename = map_to_cstr_optional(filename);
            let mime_type = map_to_cstr_optional(mime_type);

            let format = av_guess_format(
                cstr_optional_to_ptr(&short_name),
                cstr_optional_to_ptr(&filename),
                cstr_optional_to_ptr(&mime_type),
            );

            if format.is_null() {
                None
            } else {
                Some(Self { ptr: format })
            }
        }
    }

    pub fn flags(&self) -> OutputFlags {
        unsafe { OutputFlags::from_bits_unchecked((*self.ptr).flags as u32) }
    }
}
