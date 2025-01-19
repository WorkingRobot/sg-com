use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex, OnceLock},
};

use bevy::{log, prelude::*};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, StreamConfig,
};
use hound::WavSpec;

use crate::com::{self, SGContext, SG_SampleRate, SG_SampleType};

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
    stream: OnceLock<Mutex<SendStream>>,
    pub processed_data: Option<Vec<Vec<f32>>>,
    pub names: Vec<(String, Vec<String>)>,
    wav_writer: Arc<Mutex<hound::WavWriter<BufWriter<File>>>>,
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

        let writer = hound::WavWriter::create(
            "output.wav",
            WavSpec {
                channels: 1,
                sample_rate: input_config.sample_rate().0,
                bits_per_sample: input_config.sample_format().sample_size() as u16 * 8,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .unwrap();

        let ret = Self {
            context: ctx,
            names: player.processed_names(),
            player,
            stream: OnceLock::default(),
            processed_data: None,
            wav_writer: Arc::new(Mutex::new(writer)),
        };

        let stream_writer = ret.wav_writer.clone();
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
                    {
                        let mut lock = stream_writer.lock().unwrap();
                        for sample in data {
                            lock.write_sample(*sample).unwrap();
                        }
                        lock.flush().unwrap();
                    }
                    info!("Wrote {} samples", data.len());

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

        let _ = ret.stream.set(Mutex::new(SendStream(stream)));

        ret
    }
}

fn process_data(mut anim: ResMut<FacialAnim>, time: Res<Time>, mut started_capturing: Local<bool>) {
    if !*started_capturing {
        anim.stream
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .0
            .play()
            .expect("Failed to begin microphone capture");

        *started_capturing = true;
        return;
    }
    let output = anim.player.process(time.delta()).unwrap();
    anim.processed_data = Some(output);
    info!("Processed");
}
