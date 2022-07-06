use crate::sys::{av_frame_alloc, av_frame_free, AVFrame};

pub struct AvFrame {
    ptr: *mut AVFrame,
}

impl AvFrame {
    pub fn alloc() -> Option<Self> {
        unsafe {
            let frame = av_frame_alloc();
            if frame.is_null() {
                None
            } else {
                Some(Self { ptr: frame })
            }
        }
    }
}

impl Drop for AvFrame {
    fn drop(&mut self) {
        unsafe {
            av_frame_free(&mut self.ptr);
        }
    }
}
