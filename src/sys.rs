#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::approx_constant)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::upper_case_acronyms)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! FFERRTAG {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        ($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24)
    };
}

pub const AVERROR_BSF_NOT_FOUND: u32 = FFERRTAG!(0xF8, b'B', b'S', b'F');
pub const AVERROR_BUG: u32 = FFERRTAG!('B', 'U', 'G', '!');
pub const AVERROR_BUFFER_TOO_SMALL: u32 = FFERRTAG!('B', 'U', 'F', 'S');
pub const AVERROR_DECODER_NOT_FOUND: u32 = FFERRTAG!(0xF8, 'D', 'E', 'C');
pub const AVERROR_DEMUXER_NOT_FOUND: u32 = FFERRTAG!(0xF8, 'D', 'E', 'M');
pub const AVERROR_ENCODER_NOT_FOUND: u32 = FFERRTAG!(0xF8, 'E', 'N', 'C');
pub const AVERROR_EOF: u32 = FFERRTAG!('E', 'O', 'F', ' ');
pub const AVERROR_EXIT: u32 = FFERRTAG!('E', 'X', 'I', 'T');
pub const AVERROR_EXTERNAL: u32 = FFERRTAG!('E', 'X', 'T', ' ');
pub const AVERROR_FILTER_NOT_FOUND: u32 = FFERRTAG!(0xF8, 'F', 'I', 'L');
pub const AVERROR_INVALIDDATA: u32 = FFERRTAG!('I', 'N', 'D', 'A');
pub const AVERROR_MUXER_NOT_FOUND: u32 = FFERRTAG!(0xF8, 'M', 'U', 'X');
pub const AVERROR_OPTION_NOT_FOUND: u32 = FFERRTAG!(0xF8, 'O', 'P', 'T');
pub const AVERROR_PATCHWELCOME: u32 = FFERRTAG!('P', 'A', 'W', 'E');
pub const AVERROR_PROTOCOL_NOT_FOUND: u32 = FFERRTAG!(0xF8, 'P', 'R', 'O');
pub const AVERROR_STREAM_NOT_FOUND: u32 = FFERRTAG!(0xF8, 'S', 'T', 'R');
pub const AVERROR_BUG2: u32 = FFERRTAG!('B', 'U', 'G', ' ');
pub const AVERROR_UNKNOWN: u32 = FFERRTAG!('U', 'N', 'K', 'N');
pub const AVERROR_EXPERIMENTAL: u32 = 0x2bb2afa8;
pub const AVERROR_INPUT_CHANGED: u32 = 0x636e6701;
pub const AVERROR_OUTPUT_CHANGED: u32 = 0x636e6702;
/* HTTP & RTSP errors */
pub const AVERROR_HTTP_BAD_REQUEST: u32 = FFERRTAG!(0xF8, '4', '0', '0');
pub const AVERROR_HTTP_UNAUTHORIZED: u32 = FFERRTAG!(0xF8, '4', '0', '1');
pub const AVERROR_HTTP_FORBIDDEN: u32 = FFERRTAG!(0xF8, '4', '0', '3');
pub const AVERROR_HTTP_NOT_FOUND: u32 = FFERRTAG!(0xF8, '4', '0', '4');
pub const AVERROR_HTTP_OTHER_4XX: u32 = FFERRTAG!(0xF8, '4', 'X', 'X');
pub const AVERROR_HTTP_SERVER_ERROR: u32 = FFERRTAG!(0xF8, '5', 'X', 'X');
