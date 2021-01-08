pub use std::os::raw::*;

pub type DWORD = c_uint;
pub type QWORD = c_ulong;
pub type BOOL = c_int;

pub type HSAMPLE = DWORD;
pub type HSTREAM = DWORD;

pub type STREAMPROC = extern "C" fn(HSTREAM, *mut c_void, DWORD, *mut c_void) -> DWORD;

#[repr(C)]
pub struct BASS_DEVICEINFO {
    pub name: *mut c_char,
    pub driver: *mut c_char,
    pub flags: DWORD,
}

#[repr(C)]
pub struct BASS_SAMPLE {
    /// Default sample rate.
    pub freq: DWORD,
    pub volume: c_float,
    pub pan: c_float,
    pub flags: DWORD,
    pub length: DWORD,
    pub max: DWORD,
    pub origres: DWORD,
    pub chans: DWORD,
    pub mingap: DWORD,
    pub mode3d: DWORD,
    pub mindist: c_float,
    pub maxdist: c_float,
    pub iangle: DWORD,
    pub oangle: DWORD,
    pub outvol: c_float,
    pub vam: DWORD,
    pub priority: DWORD,
}
