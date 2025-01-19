use super::{
    bindings::{
        SG_AnimationNodeInfo, SG_AnimationType, SG_Error, SG_GetAnimationChannelName,
        SG_GetAnimationNodeInfo, SG_GetOutputTraits, SG_GetVersionNumber, SG_GetVersionString,
        SG_Initialize, SG_InputTraits, SG_OutputDataType, SG_OutputTraits,
        SG_STDLN_CreateTransceiver, SG_SampleRate, SG_SampleType, SG_SetIntensity, SG_Shutdown,
        SG_TransceiverPtr, ALGORITHM_DATA, CHARACTER_DATA,
    },
    error::{Error, Result},
    player::Player,
};
use std::{ffi::c_char, fmt::Debug, sync::LazyLock};

static CONTEXT: LazyLock<Result<SGContext>> =
    LazyLock::new(|| SGContext::new(CHARACTER_DATA.to_vec(), ALGORITHM_DATA.to_vec()));

pub fn get() -> Result<&'static SGContext> {
    CONTEXT.as_ref().map_err(|e| *e)
}

pub struct SGContext {
    character_data: Vec<u8>,
    algorithm_data: Vec<u8>,
}

#[derive(Debug)]
pub struct AnimationNodeInfo {
    pub(super) imp: SG_AnimationNodeInfo,
    pub(super) channel_names: Vec<String>,
}

static INITIALIZE_CODE: LazyLock<Result<()>> =
    LazyLock::new(|| unsafe { SG_Initialize() }.into_result());

impl SGContext {
    fn new(character_data: Vec<u8>, algorithm_data: Vec<u8>) -> Result<Self> {
        (*INITIALIZE_CODE)?;

        Ok(Self {
            character_data,
            algorithm_data,
        })
    }

    pub fn add_player(
        &self,
        sample_type: SG_SampleType,
        sample_rate: SG_SampleRate,
    ) -> Result<Player> {
        let mut input_traits = SG_InputTraits {
            sample_type,
            sample_rate,
            user_sample_size: 0,
        };

        let mut algorithm_data = self.algorithm_data.clone();
        let mut character_data = self.character_data.clone();
        let mut transceiver: SG_TransceiverPtr = std::ptr::null_mut();
        unsafe {
            SG_STDLN_CreateTransceiver(
                algorithm_data.as_mut_ptr(),
                algorithm_data.len(),
                character_data.as_mut_ptr(),
                character_data.len(),
                (&mut input_traits) as *mut SG_InputTraits,
                SG_OutputDataType::SG_OUTPUT_ANIMATION,
                None,
                SG_AnimationType::SG_ANIM_CONTROL,
                1,
                1.0,
                0.0,
                &mut transceiver as *mut *mut _,
            )
        }
        .into_result()?;

        let mut output_traits = SG_OutputTraits::default();
        unsafe { SG_GetOutputTraits(transceiver, 0, &mut output_traits) }.into_result()?;

        unsafe { SG_SetIntensity(transceiver, 1.0) }.into_result()?;

        if output_traits.anim_node_count == 0 {
            // No animation data found
            return Err(Error::from(SG_Error::SG_ERROR_INVALID_ANIMATION_NODE));
        }

        let mut nodes = Vec::with_capacity(output_traits.anim_node_count as usize);
        for i in 0..output_traits.anim_node_count {
            let mut node_info = SG_AnimationNodeInfo::default();
            unsafe { SG_GetAnimationNodeInfo(transceiver, 0, i, &mut node_info) }.into_result()?;

            let mut channel_names = Vec::with_capacity(node_info.channel_count as usize);
            for j in 0..node_info.channel_count {
                let channel_name: &mut [c_char] = &mut [0; 1024];
                unsafe {
                    SG_GetAnimationChannelName(
                        transceiver,
                        0,
                        node_info.name.as_mut_ptr(),
                        j,
                        channel_name.as_mut_ptr(),
                        1024,
                    )
                }
                .into_result()?;
                channel_names.push(
                    std::ffi::CStr::from_bytes_until_nul(unsafe {
                        &*(channel_name as *const [i8] as *const [u8])
                    })
                    .map_err(|_| Error::from(SG_Error::SG_ERROR_INVALID_ANIMATION_CHANNEL))?
                    .to_str()
                    .map_err(|_| Error::from(SG_Error::SG_ERROR_INVALID_ANIMATION_CHANNEL))?
                    .to_owned(),
                );
            }

            nodes.push(AnimationNodeInfo {
                imp: node_info,
                channel_names,
            });
        }

        Ok(Player::new(transceiver, input_traits, output_traits, nodes))
    }

    pub fn version() -> String {
        let version = unsafe { SG_GetVersionString() };
        unsafe { std::ffi::CStr::from_ptr(version) }
            .to_str()
            .unwrap_or_else(|_| "Unknown")
            .to_owned()
    }

    pub fn version_number() -> u32 {
        unsafe { SG_GetVersionNumber() }
    }
}

impl SG_SampleRate {
    pub fn to_rate(self) -> i32 {
        match self {
            SG_SampleRate::SG_RATE_8KHZ => 8000,
            SG_SampleRate::SG_RATE_12KHZ => 12000,
            SG_SampleRate::SG_RATE_16KHZ => 16000,
            SG_SampleRate::SG_RATE_24KHZ => 24000,
            SG_SampleRate::SG_RATE_32KHZ => 32000,
            SG_SampleRate::SG_RATE_48KHZ => 48000,
        }
    }

    pub fn from_rate(rate: i32) -> Option<Self> {
        match rate {
            8000 => Some(SG_SampleRate::SG_RATE_8KHZ),
            12000 => Some(SG_SampleRate::SG_RATE_12KHZ),
            16000 => Some(SG_SampleRate::SG_RATE_16KHZ),
            24000 => Some(SG_SampleRate::SG_RATE_24KHZ),
            32000 => Some(SG_SampleRate::SG_RATE_32KHZ),
            48000 => Some(SG_SampleRate::SG_RATE_48KHZ),
            _ => None,
        }
    }
}

impl Drop for SGContext {
    fn drop(&mut self) {
        unsafe { SG_Shutdown() };
    }
}
