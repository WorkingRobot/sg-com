use crate::com::{self, SGContext, SG_SampleRate, SG_SampleType};
use bevy::{log, prelude::*};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, SampleRate, StreamConfig,
};
use crossbeam_deque::Worker;
use std::sync::Mutex;

pub struct FacialAnimPlugin;

impl Plugin for FacialAnimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FacialAnim::new());
        app.add_systems(PreUpdate, process_data);
    }
}

#[derive(Resource)]
pub struct FacialAnim {
    context: &'static SGContext,
    player: com::Player,
    stream: Mutex<SendStream>,
    out_stream: Mutex<SendStream>,
    pub processed_data: Option<Vec<Vec<f32>>>,
    pub names: Vec<(String, Vec<String>)>,
}

struct SendStream(cpal::Stream);

unsafe impl Send for SendStream {}

impl FacialAnim {
    pub fn new() -> Self {
        let ctx = com::context().expect("Failed to initialize SG_Com");

        let host = cpal::default_host();
        let input = host
            .default_input_device()
            .expect("No input device available");
        let input_config = input
            .default_input_config()
            .expect("No default input config available");

        log::info!("Input device: {}", input.name().unwrap());
        log::info!("Input config: {:?}", input_config);

        let output = host
            .default_output_device()
            .expect("No output device available");

        let com_sample_type = match input_config.sample_format() {
            SampleFormat::I8 => SG_SampleType::SG_SAMPLE_PCM8,
            SampleFormat::I16 => SG_SampleType::SG_SAMPLE_PCM16,
            SampleFormat::I32 => SG_SampleType::SG_SAMPLE_PCM32,
            SampleFormat::F32 => SG_SampleType::SG_SAMPLE_FLOAT32,
            SampleFormat::F64 => SG_SampleType::SG_SAMPLE_FLOAT64,
            _ => panic!("Unsupported sample format"),
        };
        let com_sample_rate = SG_SampleRate::from_rate(input_config.sample_rate().0 as i32)
            .expect("Unsupported sample rate");
        let player = ctx
            .add_player(com_sample_type, com_sample_rate)
            .expect("Failed to add player");

        let err_fn = move |err| {
            log::error!("Input stream error: {}", err);
        };
        let stream_player = player.clone();
        let stream_config: StreamConfig = input_config.clone().into();

        let producer = Worker::<f32>::new_lifo();

        let consumer = producer.stealer();
        let output_config = StreamConfig {
            channels: 2,
            sample_rate: SampleRate(44100),
            buffer_size: cpal::BufferSize::Default,
        };
        let s = output
            .build_output_stream(
                &output_config,
                move |data: &mut [f32], _| {
                    let sample_count = data.len() / 2;
                    let data_to_take = sample_count * 48000 / 44100;
                    let mut ret = Vec::with_capacity(data_to_take);
                    for _ in 0..data_to_take {
                        let v = match consumer.steal() {
                            crossbeam_deque::Steal::Success(d) => d,
                            _ => 0.0,
                        };
                        ret.push(v);
                    }
                    let idx_dist = sample_count as f32 / data_to_take as f32;
                    for i in 0..sample_count {
                        let idx = (i as f32 / idx_dist) as usize;
                        data[i * 2] = ret[idx];
                        data[i * 2 + 1] = ret[idx];
                    }
                },
                err_fn,
                None,
            )
            .expect("Failed to build output stream");

        let stream = match com_sample_type {
            SG_SampleType::SG_SAMPLE_PCM8 => input.build_input_stream(
                &stream_config,
                move |data: &[i8], _| {
                    stream_player
                        .add_input_pcm8(&mut data.to_vec())
                        .expect("Failed to add input");
                },
                err_fn,
                None,
            ),
            SG_SampleType::SG_SAMPLE_PCM16 => input.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    stream_player
                        .add_input_pcm16(&mut data.to_vec())
                        .expect("Failed to add input");
                },
                err_fn,
                None,
            ),
            SG_SampleType::SG_SAMPLE_PCM32 => input.build_input_stream(
                &stream_config,
                move |data: &[i32], _| {
                    stream_player
                        .add_input_pcm32(&mut data.to_vec())
                        .expect("Failed to add input");
                },
                err_fn,
                None,
            ),
            SG_SampleType::SG_SAMPLE_FLOAT32 => input.build_input_stream(
                &stream_config,
                move |data: &[f32], _| {
                    for s in data {
                        producer.push(*s);
                    }
                    stream_player
                        .add_input_float32(&mut data.to_vec())
                        .expect("Failed to add input");
                },
                err_fn,
                None,
            ),
            SG_SampleType::SG_SAMPLE_FLOAT64 => input.build_input_stream(
                &stream_config,
                move |data: &[f64], _| {
                    stream_player
                        .add_input_float64(&mut data.to_vec())
                        .expect("Failed to add input");
                },
                err_fn,
                None,
            ),
        }
        .expect("Failed to build input stream");

        let ret = Self {
            context: ctx,
            names: player.processed_names(),
            player,
            stream: Mutex::new(SendStream(stream)),
            out_stream: Mutex::new(SendStream(s)),
            processed_data: None,
        };

        ret
    }
}

fn process_data(mut anim: ResMut<FacialAnim>, time: Res<Time>, mut started_capturing: Local<bool>) {
    if !*started_capturing {
        anim.stream
            .lock()
            .unwrap()
            .0
            .play()
            .expect("Failed to begin microphone capture");

        anim.out_stream
            .lock()
            .unwrap()
            .0
            .play()
            .expect("Failed to begin speaker output");

        *started_capturing = true;
        return;
    }
    let output = anim.player.process(time.delta()).unwrap();
    anim.processed_data = Some(output);
}
