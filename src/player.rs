use crate::{
    bindings::{
        SG_AdvanceOutput, SG_Error, SG_GetOutputAnimation, SG_Input, SG_InputTraits,
        SG_OutputTraits, SG_STDLN_DestroyTransceiver, SG_SampleRate, SG_SampleType,
        SG_TransceiverPtr, SG_UpdateInputTraits,
    },
    context::AnimationNodeInfo,
    error::Result,
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Player {
    imp: Arc<PlayerImpl>,
}

#[derive(Debug)]
struct PlayerImpl {
    transceiver: SG_TransceiverPtr,
    input_traits: SG_InputTraits,
    output_traits: SG_OutputTraits,
    nodes: Vec<AnimationNodeInfo>,
}

impl Player {
    pub fn new(
        transceiver: SG_TransceiverPtr,
        input_traits: SG_InputTraits,
        output_traits: SG_OutputTraits,
        nodes: Vec<AnimationNodeInfo>,
    ) -> Self {
        Self {
            imp: Arc::new(PlayerImpl {
                transceiver,
                input_traits,
                output_traits,
                nodes,
            }),
        }
    }

    pub fn add_input_pcm8(&self, buffer: &mut [u8]) -> Result<()> {
        if self.imp.input_traits.sample_type != SG_SampleType::SG_SAMPLE_PCM8 {
            return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into());
        }
        unsafe {
            SG_Input(
                self.imp.transceiver,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
                std::ptr::null_mut(),
            )
        }
        .into_result()
    }

    pub fn add_input_pcm16(&self, buffer: &mut [u16]) -> Result<()> {
        if self.imp.input_traits.sample_type != SG_SampleType::SG_SAMPLE_PCM16 {
            return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into());
        }
        unsafe {
            SG_Input(
                self.imp.transceiver,
                buffer.as_mut_ptr().cast(),
                (buffer.len() / 2) as u32,
                std::ptr::null_mut(),
            )
        }
        .into_result()
    }

    pub fn add_input_pcm32(&self, buffer: &mut [i32]) -> Result<()> {
        if self.imp.input_traits.sample_type != SG_SampleType::SG_SAMPLE_PCM32 {
            return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into());
        }
        unsafe {
            SG_Input(
                self.imp.transceiver,
                buffer.as_mut_ptr().cast(),
                (buffer.len() / 4) as u32,
                std::ptr::null_mut(),
            )
        }
        .into_result()
    }

    pub fn add_input_float32(&self, buffer: &mut [f32]) -> Result<()> {
        if self.imp.input_traits.sample_type != SG_SampleType::SG_SAMPLE_FLOAT32 {
            return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into());
        }
        unsafe {
            SG_Input(
                self.imp.transceiver,
                buffer.as_mut_ptr().cast(),
                (buffer.len() / 4) as u32,
                std::ptr::null_mut(),
            )
        }
        .into_result()
    }

    pub fn add_input_float64(&self, buffer: &mut [f64]) -> Result<()> {
        if self.imp.input_traits.sample_type != SG_SampleType::SG_SAMPLE_FLOAT64 {
            return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into());
        }
        unsafe {
            SG_Input(
                self.imp.transceiver,
                buffer.as_mut_ptr().cast(),
                (buffer.len() / 8) as u32,
                std::ptr::null_mut(),
            )
        }
        .into_result()
    }

    pub fn process(
        &self,
        delta: std::time::Duration,
    ) -> Result<BTreeMap<String, BTreeMap<String, f32>>> {
        unsafe { SG_AdvanceOutput(self.imp.transceiver, delta.as_secs_f32() * 1000.0) }
            .into_result()?;

        let mut ret = BTreeMap::new();

        for node in &self.imp.nodes {
            if node.imp.channel_count == 0 {
                continue;
            }

            let mut animation_data: *mut f32 = std::ptr::null_mut();
            unsafe {
                SG_GetOutputAnimation(
                    self.imp.transceiver,
                    0,
                    node.imp.name.as_ptr(),
                    &mut animation_data,
                )
            }
            .into_result()?;
            if animation_data.is_null() {
                ret.insert(node.imp.name(), BTreeMap::new());
            } else {
                ret.insert(
                    node.imp.name(),
                    unsafe {
                        std::slice::from_raw_parts(animation_data, node.imp.channel_count as usize)
                    }
                    .iter()
                    .zip(node.channel_names.iter())
                    .map(|(v, n)| (n.clone(), *v))
                    .collect(),
                );
            }
        }

        Ok(ret)
    }

    pub fn animation_info(&self) -> &Vec<AnimationNodeInfo> {
        &self.imp.nodes
    }

    pub fn sample_rate(&self) -> SG_SampleRate {
        self.imp.input_traits.sample_rate
    }

    pub fn set_sample_rate(&self, sample_rate: SG_SampleRate) -> Result<()> {
        let mut new_traits = self.imp.input_traits.clone();
        new_traits.sample_rate = sample_rate;
        unsafe { SG_UpdateInputTraits(&mut new_traits, self.imp.transceiver) }.into_result()
    }
}

impl Drop for PlayerImpl {
    fn drop(&mut self) {
        unsafe { SG_STDLN_DestroyTransceiver(self.transceiver) };
    }
}
