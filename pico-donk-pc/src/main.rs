use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ChannelCount, Sample, SampleRate};
use pico_donk_core::cst::Sample as PicoSample;
use pico_donk_core::helpers::SinCos;
use pico_donk_core::Song;

const CHANNELS: ChannelCount = 2;
const SAMPLE_RATE: SampleRate = SampleRate { 0: 44100 };

fn main() -> ! {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No audio output device available.");
    let supported_config = device
        .supported_output_configs()
        .expect("Error while querying audio configs")
        .find(|c| {
            c.channels() == CHANNELS
                && c.min_sample_rate() <= SAMPLE_RATE
                && c.max_sample_rate() >= SAMPLE_RATE
        })
        .expect("Could not find suitable audio config")
        .with_sample_rate(SAMPLE_RATE);

    let mut song = Song::new();

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let config = supported_config.into();
    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in output.chunks_mut((CHANNELS * 2) as usize) {
                    let value = Sample::from(&song.get_sample());
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
        )
        .expect("Unable to create stream");

    stream.play().expect("Unable to play stream");

    loop {
        let mut input_text = String::new();
        match std::io::stdin().read_line(&mut input_text) {
            Ok(_) => match input_text.trim().parse::<PicoSample>() {
                Ok(sample) => {
                    let first = ((sample.cos().to_num::<f64>() - 1.) * 2.) - 1.;
                    let second = PicoSample::from_num(
                        (sample.to_num::<f64>() * (std::f64::consts::PI * 2.)).cos(),
                    )
                    .to_num::<f64>();
                    println!("{} vs {}, difference {}", first, second, second - first);
                }
                Err(e) => eprintln!("{}", e),
            },
            Err(e) => eprintln!("{}", e),
        };
    }
}
