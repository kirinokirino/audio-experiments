pub const DEVICES: &[&str] = &["default\0", "pipewire\0"];
pub const SAMPLE_RATE: u32 = 44100;
pub const CHANNELS: u16 = 2;
pub const PCM_BUFFER_SIZE: ::std::os::raw::c_ulong = 4096 / 8;
