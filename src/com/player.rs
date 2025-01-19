use bevy::log::warn;

use super::{
    bindings::{
        SG_AdvanceOutput, SG_Error, SG_GetOutputAnimation, SG_Input, SG_InputTraits,
        SG_OutputTraits, SG_STDLN_DestroyTransceiver, SG_SampleRate, SG_SampleType,
        SG_TransceiverPtr,
    },
    context::AnimationNodeInfo,
    error::Result,
};
use std::sync::{Arc, Mutex};

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
    queued_buffer: Mutex<AudioQueue>,
}

unsafe impl Send for PlayerImpl {}
unsafe impl Sync for PlayerImpl {}

#[derive(Debug)]
struct AudioQueue {
    buffer: AudioBuffer,
    sample_rate: SG_SampleRate,
}

#[derive(Debug)]
enum AudioBuffer {
    PCM8(Vec<i8>),
    PCM16(Vec<i16>),
    PCM32(Vec<i32>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
}

impl AudioQueue {
    pub fn new(sample_rate: SG_SampleRate, sample_type: SG_SampleType) -> Self {
        Self {
            buffer: match sample_type {
                SG_SampleType::SG_SAMPLE_PCM8 => AudioBuffer::PCM8(Vec::new()),
                SG_SampleType::SG_SAMPLE_PCM16 => AudioBuffer::PCM16(Vec::new()),
                SG_SampleType::SG_SAMPLE_PCM32 => AudioBuffer::PCM32(Vec::new()),
                SG_SampleType::SG_SAMPLE_FLOAT32 => AudioBuffer::Float32(Vec::new()),
                SG_SampleType::SG_SAMPLE_FLOAT64 => AudioBuffer::Float64(Vec::new()),
            },
            sample_rate,
        }
    }

    // 10ms worth of samples
    fn buffer_capacity(&self) -> usize {
        (self.sample_rate.to_rate() / (1000 / 10)) as usize
    }

    fn add_data<T>(vec: &mut Vec<T>, buffer: &[T], capacity: usize) -> Option<Vec<T>>
    where
        T: Copy,
    {
        vec.extend_from_slice(buffer);
        if vec.len() >= capacity {
            let new_vec = vec.split_off(capacity);
            let ret = std::mem::replace(vec, new_vec);
            if vec.len() > capacity {
                warn!("Added data is larger than the buffer capacity");
            }
            Some(ret)
        } else {
            None
        }
    }

    // Might return a Vec<i8> if it's time to flush the buffer
    pub fn add_pcm8(&mut self, buffer: &[i8]) -> Result<Option<Vec<i8>>> {
        let capacity = self.buffer_capacity();

        let vec = match &mut self.buffer {
            AudioBuffer::PCM8(vec) => vec,
            _ => return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into()),
        };

        Ok(Self::add_data(vec, buffer, capacity))
    }

    pub fn add_pcm16(&mut self, buffer: &[i16]) -> Result<Option<Vec<i16>>> {
        let capacity = self.buffer_capacity();

        let vec = match &mut self.buffer {
            AudioBuffer::PCM16(vec) => vec,
            _ => return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into()),
        };

        Ok(Self::add_data(vec, buffer, capacity))
    }

    pub fn add_pcm32(&mut self, buffer: &[i32]) -> Result<Option<Vec<i32>>> {
        let capacity = self.buffer_capacity();

        let vec = match &mut self.buffer {
            AudioBuffer::PCM32(vec) => vec,
            _ => return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into()),
        };

        Ok(Self::add_data(vec, buffer, capacity))
    }

    pub fn add_float32(&mut self, buffer: &[f32]) -> Result<Option<Vec<f32>>> {
        let capacity = self.buffer_capacity();

        let vec = match &mut self.buffer {
            AudioBuffer::Float32(vec) => vec,
            _ => return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into()),
        };

        Ok(Self::add_data(vec, buffer, capacity))
    }

    pub fn add_float64(&mut self, buffer: &[f64]) -> Result<Option<Vec<f64>>> {
        let capacity = self.buffer_capacity();

        let vec = match &mut self.buffer {
            AudioBuffer::Float64(vec) => vec,
            _ => return Err(SG_Error::SG_ERROR_INVALID_INPUT_TRAITS.into()),
        };

        Ok(Self::add_data(vec, buffer, capacity))
    }
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
                queued_buffer: Mutex::new(AudioQueue::new(
                    input_traits.sample_rate,
                    input_traits.sample_type,
                )),
            }),
        }
    }

    pub fn add_input_pcm8(&self, buffer: &[i8]) -> Result<()> {
        if let Some(mut data) = self.imp.queued_buffer.lock().unwrap().add_pcm8(buffer)? {
            unsafe {
                SG_Input(
                    self.imp.transceiver,
                    data.as_mut_ptr().cast(),
                    data.len() as u32,
                    std::ptr::null_mut(),
                )
            }
            .into_result()?
        }
        Ok(())
    }

    pub fn add_input_pcm16(&self, buffer: &[i16]) -> Result<()> {
        if let Some(mut data) = self.imp.queued_buffer.lock().unwrap().add_pcm16(buffer)? {
            unsafe {
                SG_Input(
                    self.imp.transceiver,
                    data.as_mut_ptr().cast(),
                    data.len() as u32,
                    std::ptr::null_mut(),
                )
            }
            .into_result()?
        }
        Ok(())
    }

    pub fn add_input_pcm32(&self, buffer: &[i32]) -> Result<()> {
        if let Some(mut data) = self.imp.queued_buffer.lock().unwrap().add_pcm32(buffer)? {
            unsafe {
                SG_Input(
                    self.imp.transceiver,
                    data.as_mut_ptr().cast(),
                    data.len() as u32,
                    std::ptr::null_mut(),
                )
            }
            .into_result()?
        }
        Ok(())
    }

    pub fn add_input_float32(&self, buffer: &[f32]) -> Result<()> {
        if let Some(mut data) = self.imp.queued_buffer.lock().unwrap().add_float32(buffer)? {
            unsafe {
                SG_Input(
                    self.imp.transceiver,
                    data.as_mut_ptr().cast(),
                    data.len() as u32,
                    std::ptr::null_mut(),
                )
            }
            .into_result()?
        }
        Ok(())
    }

    pub fn add_input_float64(&self, buffer: &[f64]) -> Result<()> {
        if let Some(mut data) = self.imp.queued_buffer.lock().unwrap().add_float64(buffer)? {
            unsafe {
                SG_Input(
                    self.imp.transceiver,
                    data.as_mut_ptr().cast(),
                    data.len() as u32,
                    std::ptr::null_mut(),
                )
            }
            .into_result()?
        }
        Ok(())
    }

    pub fn process(&self, delta: std::time::Duration) -> Result<Vec<Vec<f32>>> {
        unsafe { SG_AdvanceOutput(self.imp.transceiver, delta.as_secs_f32() * 1000.0) }
            .into_result()?;

        let mut ret = Vec::with_capacity(self.imp.nodes.len());

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
                ret.push(Vec::new());
            } else {
                ret.push(
                    unsafe {
                        std::slice::from_raw_parts(animation_data, node.imp.channel_count as usize)
                    }
                    .to_vec(),
                );
            }
        }

        Ok(ret)
    }

    pub fn processed_names(&self) -> Vec<(String, Vec<String>)> {
        self.imp
            .nodes
            .iter()
            .map(|node| (node.imp.name(), node.channel_names.clone()))
            .collect()
    }

    pub fn animation_info(&self) -> &Vec<AnimationNodeInfo> {
        &self.imp.nodes
    }

    pub fn sample_rate(&self) -> SG_SampleRate {
        self.imp.input_traits.sample_rate
    }

    // pub fn set_sample_rate(&self, sample_rate: SG_SampleRate) -> Result<()> {
    //     let mut new_traits = self.imp.input_traits.clone();
    //     new_traits.sample_rate = sample_rate;
    //     unsafe { SG_UpdateInputTraits(&mut new_traits, self.imp.transceiver) }.into_result()
    // }
}

impl Drop for PlayerImpl {
    fn drop(&mut self) {
        unsafe { SG_STDLN_DestroyTransceiver(self.transceiver) };
    }
}
