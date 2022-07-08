use crate::context::{ContextType, InputContext, OutputContext};
use crate::dict::AvDictionary;
use crate::io::AvIoContext;
use crate::packet::AvPacket;
use crate::stream::AvStream;
use crate::sys::{
    av_guess_format, av_interleaved_write_frame, av_read_frame, av_write_frame, av_write_trailer,
    avformat_find_stream_info, avformat_init_output, avformat_new_stream, avformat_open_input,
    avformat_write_header, AVFormatContext, AVIOContext, AVOutputFormat,
};
use crate::util::{cstr_optional_to_ptr, map_to_cstr_optional};
use crate::{avformat_alloc_output_context2, wrap_error, AvBorrow, AvBorrowMut, AvError};
use std::marker::PhantomData;
use std::ptr;

pub struct AvFormatContext<T: ContextType> {
    pub(crate) ptr: *mut AVFormatContext,
    _type: PhantomData<T>,
}

impl AvFormatContext<InputContext> {
    pub fn open_input(url: Option<&str>) -> Result<Self, AvError> {
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
                val => Err(wrap_error(val)),
            }
        }
    }

    pub fn find_stream_info(&self) -> Result<(), AvError> {
        unsafe {
            let result = avformat_find_stream_info(self.ptr, ptr::null_mut());

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
            }
        }
    }

    pub fn read_frame(&mut self, dest: &mut AvPacket) -> Result<(), AvError> {
        unsafe {
            let result = av_read_frame(self.ptr, dest.ptr);

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
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
    ) -> Result<Self, AvError> {
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
                val => Err(wrap_error(val)),
            }
        }
    }

    pub fn init_output(&mut self, options: Option<&mut AvDictionary>) -> Result<(), AvError> {
        unsafe {
            let options = match options {
                None => ptr::null_mut(),
                Some(value) => &mut value.ptr,
            };
            let result = avformat_init_output(self.ptr, options);

            if result.is_negative() {
                Err(wrap_error(result))
            } else {
                Ok(())
            }
        }
    }

    pub fn write_header(&mut self) -> Result<(), AvError> {
        unsafe {
            let result = avformat_write_header(self.ptr, ptr::null_mut());

            if result.is_negative() {
                Err(wrap_error(result))
            } else {
                Ok(())
            }
        }
    }

    pub fn interleaved_write_frame(&self, packet: &AvPacket) -> Result<(), AvError> {
        unsafe {
            let result = av_interleaved_write_frame(self.ptr, packet.ptr);

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
            }
        }
    }

    pub fn write_frame(&self, packet: Option<&AvPacket>) -> Result<(), AvError> {
        unsafe {
            let packet = match packet {
                None => ptr::null_mut(),
                Some(value) => value.ptr,
            };

            let result = av_write_frame(self.ptr, packet);

            if result.is_negative() {
                Err(wrap_error(result))
            } else {
                Ok(())
            }
        }
    }

    pub fn write_trailer(&mut self) -> Result<(), AvError> {
        unsafe {
            let result = av_write_trailer(self.ptr);

            match result {
                0 => Ok(()),
                val => Err(wrap_error(val)),
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
            Some(AvBorrow::wrap(&self, AvStream::wrap(stream)))
        }
    }

    pub fn new_stream(&mut self) -> Option<AvStream<T>> {
        unsafe {
            let stream = avformat_new_stream(self.ptr, ptr::null());
            if stream.is_null() {
                None
            } else {
                Some(AvStream::wrap(stream))
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
}
