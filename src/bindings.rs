#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl std::fmt::Debug for SG_AnimationNodeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SG_AnimationNodeInfo")
            .field("name", &self.name())
            .field("type", &self.type_)
            .field("channel_count", &self.channel_count)
            .finish()
    }
}

impl SG_AnimationNodeInfo {
    pub fn name(&self) -> String {
        std::ffi::CStr::from_bytes_until_nul(unsafe {
            &*(&self.name as *const [i8] as *const [u8])
        })
        .ok()
        .and_then(|s| s.to_str().ok())
        .unwrap_or("Unknown")
        .to_owned()
    }
}
