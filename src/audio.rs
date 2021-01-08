use std::ffi::CString;
use std::path::Path;
use std::ptr;

use anyhow::Result;
use bass::constants::*;

pub struct AudioEngine {
    current_device: AudioDevice,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let current_device = AudioDevice::init_default()?;
        Ok(AudioEngine { current_device })
    }

    pub fn play(&self, sound: &Sound) {
        let handle = sound.handle();
        unsafe { bass::BASS_ChannelPlay(handle, 0) };
    }

    pub fn pause(&self, sound: &Sound) {
        let handle = sound.handle();
        unsafe { bass::BASS_ChannelPause(handle) };
    }
}

pub struct AudioDevice {
    id: i32,
}

impl AudioDevice {
    pub fn init_default() -> Result<Self> {
        Self::init(-1)
    }

    pub fn init(id: i32) -> Result<Self> {
        let result = unsafe { bass::BASS_Init(id, 44100, 0, ptr::null(), ptr::null()) };
        if result != 1 {
            bail!("initialization failed");
        }
        Ok(AudioDevice { id })
    }
}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        unsafe { bass::BASS_Free() };
    }
}

pub struct Sound {
    handle: u32,
}

impl Sound {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let path_str = path.to_str().unwrap();
        let path_cstr = CString::new(path_str.as_bytes()).unwrap();
        let handle =
            unsafe { bass::BASS_StreamCreateFile(0, path_cstr.into_raw() as *mut _, 0, 0, 0) };

        Ok(Sound { handle })
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }

    pub fn position(&self) -> Result<f64> {
        let time = unsafe {
            let pos_bytes = bass::BASS_ChannelGetPosition(self.handle, BASS_POS_BYTE);
            bass::BASS_ChannelBytes2Seconds(self.handle, pos_bytes)
        };
        Ok(time)
    }

    pub fn set_position(&self, pos: f64) -> Result<()> {
        unsafe {
            let pos_bytes = bass::BASS_ChannelSeconds2Bytes(self.handle, pos);
            bass::BASS_ChannelSetPosition(self.handle, pos_bytes, BASS_POS_BYTE);
        }
        Ok(())
    }
}
