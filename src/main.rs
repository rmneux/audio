use std::sync::Mutex;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

static DO_NOT_MOVE_THIS: Mutex<Option<DoNotMoveThis>> = Mutex::new(None);

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into(), &DO_NOT_MOVE_THIS),
        _ => panic!(),
    }
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    do_not_move_this: &'static Mutex<Option<DoNotMoveThis>>,
) -> anyhow::Result<()>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let sample_clock = 0f32;

    *DO_NOT_MOVE_THIS.lock().unwrap() = DoNotMoveThis {
        sample_rate,
        channels,
        sample_clock,
    }
    .into();

    let stream = device.build_output_stream(
        config,
        |data: &mut [T], _| {
            do_not_move_this
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .write_data(data);
        },
        |err| eprintln!("{}", err),
        None,
    )?;

    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

/// Oscillator that is not moved into the data callback closure
#[derive(Debug)]
pub struct DoNotMoveThis {
    sample_rate: f32,
    channels: usize,
    sample_clock: f32,
}

impl DoNotMoveThis {
    pub fn next_value(&mut self) -> f32 {
        self.sample_clock = (self.sample_clock + 1.0) % self.sample_rate;
        (self.sample_clock * 440.0 * 2.0 * std::f32::consts::PI / self.sample_rate).sin()
    }
    pub fn write_data<T>(&mut self, output: &mut [T])
    where
        T: cpal::Sample + cpal::FromSample<f32>,
    {
        for frame in output.chunks_mut(self.channels) {
            let value: T = T::from_sample(self.next_value());
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
}
