#![allow(dead_code)]

use bindings::{SG_SampleRate, SG_SampleType};
use std::{thread, time::Duration};

mod bindings;
mod context;
mod error;
mod player;

const CHARACTER_DATA: &[u8] = include_bytes!("../deps/Jonesy.k");
const ALGORITHM_DATA: &[u8] = include_bytes!("../deps/algorithms_SGCom2.k");

fn main() {
    let ctx = context::initialize(CHARACTER_DATA.to_vec(), ALGORITHM_DATA.to_vec()).unwrap();
    let player = ctx
        .add_player(SG_SampleType::SG_SAMPLE_PCM16, SG_SampleRate::SG_RATE_16KHZ)
        .unwrap();
    let samples_10ms = player.sample_rate().to_rate() / (1000 / 10); // Samples per 10ms
    let mut buffer: Vec<u16> = vec![0; samples_10ms as usize];

    loop {
        buffer.fill(0);
        player.add_input_pcm16(&mut buffer).unwrap();
        dbg!(&player);
        let output = player.process(Duration::from_millis(10)).unwrap();
        dbg!(output);
        thread::sleep(Duration::from_millis(10));
    }
}
