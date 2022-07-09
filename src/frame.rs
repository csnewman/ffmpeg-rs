use crate::sys::{av_frame_alloc, av_frame_free, AVFrame};
use crate::util::property;

pub struct AvFrame {
    pub(crate) ptr: *mut AVFrame,
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

    property!(pts, i64);

    property!(best_effort_timestamp, i64);
}

impl Drop for AvFrame {
    fn drop(&mut self) {
        unsafe {
            av_frame_free(&mut self.ptr);
        }
    }
}
