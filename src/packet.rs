use crate::sys::{
    av_packet_alloc, av_packet_rescale_ts, av_packet_unref, AVPacket, AV_PKT_FLAG_CORRUPT,
    AV_PKT_FLAG_DISCARD, AV_PKT_FLAG_DISPOSABLE, AV_PKT_FLAG_KEY, AV_PKT_FLAG_TRUSTED,
};
use crate::util::AvRational;
use bitflags::bitflags;

bitflags! {
    pub struct PacketFlags: u32 {
        const KEY = AV_PKT_FLAG_KEY;
        const CORRUPT = AV_PKT_FLAG_CORRUPT;
        const DISCARD = AV_PKT_FLAG_DISCARD;
        const TRUSTED = AV_PKT_FLAG_TRUSTED;
        const DISPOSABLE = AV_PKT_FLAG_DISPOSABLE;
    }
}

pub struct AvPacket {
    pub(crate) ptr: *mut AVPacket,
}

impl AvPacket {
    pub fn alloc() -> Option<Self> {
        unsafe {
            let packet = av_packet_alloc();
            if packet.is_null() {
                None
            } else {
                Some(Self { ptr: packet })
            }
        }
    }

    pub fn pts(&self) -> i64 {
        unsafe { (*self.ptr).pts }
    }

    pub fn dts(&self) -> i64 {
        unsafe { (*self.ptr).dts }
    }

    pub fn stream_index(&self) -> usize {
        unsafe { (*self.ptr).stream_index as usize }
    }

    pub fn flags(&self) -> PacketFlags {
        unsafe { PacketFlags::from_bits_retain((*self.ptr).flags as u32) }
    }

    pub fn duration(&self) -> i64 {
        unsafe { (*self.ptr).duration }
    }

    pub fn rescale_ts(&mut self, tb_src: AvRational, tb_dst: AvRational) {
        unsafe { av_packet_rescale_ts(self.ptr, tb_src.into(), tb_dst.into()) }
    }

    pub fn unref(&mut self) {
        unsafe { av_packet_unref(self.ptr) }
    }
}

impl Drop for AvPacket {
    fn drop(&mut self) {
        self.unref()
    }
}
