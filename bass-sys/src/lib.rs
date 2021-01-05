#[macro_use]
mod macros;

pub mod constants;
pub mod types;

pub use crate::types::*;

extern_log! {
    pub fn BASS_ErrorGetCode() -> c_int;

    pub fn BASS_GetConfig(option: DWORD) -> DWORD;

    pub fn BASS_ChannelGetDevice(handle: DWORD) -> DWORD;
    pub fn BASS_ChannelSetDevice(handle: DWORD, device: DWORD) -> BOOL;
    pub fn BASS_ChannelPlay(handle: DWORD, restart: BOOL) -> BOOL;
    pub fn BASS_ChannelPause(handle: DWORD) -> BOOL;
    pub fn BASS_ChannelGetLength(handle: DWORD, mode: DWORD) -> QWORD;
    pub fn BASS_ChannelGetPosition(handle: DWORD, mode: DWORD) -> QWORD;
    pub fn BASS_ChannelBytes2Seconds(handle: DWORD, pos: QWORD) -> f64;

    pub fn BASS_GetDevice() -> DWORD;
    pub fn BASS_GetDeviceInfo(device: DWORD, info: *mut BASS_DEVICEINFO) -> BOOL;
    pub fn BASS_Init(device: c_int, freq: DWORD, flags: DWORD, win: *const c_void, clsid: *const c_void) -> BOOL;
    pub fn BASS_Free() -> BOOL;
    pub fn BASS_Pause() -> BOOL;
    pub fn BASS_Start() -> BOOL;
    pub fn BASS_GetVolume() -> c_float;

    pub fn BASS_StreamCreate(
        freq: DWORD,
        chans: DWORD,
        flags: DWORD,
        proc: STREAMPROC,
        user: *mut c_void,
    ) -> HSTREAM;
    pub fn BASS_StreamCreateFile(
        mem: BOOL,
        file: *mut c_void,
        offset: QWORD,
        length: QWORD,
        flags: DWORD,
    ) -> HSTREAM;
    pub fn BASS_StreamFree(handle: HSTREAM) -> BOOL;

    pub fn BASS_SampleGetInfo(handle: HSAMPLE, info: *mut BASS_SAMPLE) -> BOOL;
}
