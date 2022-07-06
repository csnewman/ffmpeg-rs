use crate::format::AvFormatContext;
use crate::sys::{avformat_close_input, avformat_free_context};

pub trait ContextType {
    fn drop_format_context(ctx: &mut AvFormatContext<Self>)
    where
        Self: Sized;
}

#[derive(Copy, Clone)]
pub struct OutputContext {}

impl ContextType for OutputContext {
    fn drop_format_context(ctx: &mut AvFormatContext<Self>) {
        unsafe {
            avformat_free_context(ctx.ptr);
        }
    }
}

#[derive(Copy, Clone)]
pub struct InputContext {}

impl ContextType for InputContext {
    fn drop_format_context(ctx: &mut AvFormatContext<Self>) {
        unsafe {
            avformat_close_input(&mut ctx.ptr);
        }
    }
}

#[derive(Copy, Clone)]
pub struct DecodeContext {}

impl ContextType for DecodeContext {
    fn drop_format_context(_ctx: &mut AvFormatContext<Self>)
    where
        Self: Sized,
    {
        panic!("Unexpected context")
    }
}

#[derive(Copy, Clone)]
pub struct EncodeContext {}

impl ContextType for EncodeContext {
    fn drop_format_context(_ctx: &mut AvFormatContext<Self>)
    where
        Self: Sized,
    {
        panic!("Unexpected context")
    }
}
