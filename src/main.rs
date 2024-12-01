// use wasapi::*;
// use std::fs::File;
// use std::io::Write;
use cpal::{ Device, Sample };
use cpal::traits::{ StreamTrait, HostTrait, DeviceTrait };
use dasp_sample::ToSample;
use std::sync::{ Arc, Mutex };
use std::thread;
// use hound;
/// capture_audio_output - capture the audio stream from the default audio output device
///
/// sets up an input stream for the wave_reader in the appropriate format (f32/i16/u16)
///

fn play_test_audio(device: cpal::Device) {
    let sample_rate = 44100;
    let duration_secs = 5;
    let samples: Vec<f32> = (0..sample_rate * duration_secs)
        .map(|i| {
            let t = (i as f32) / (sample_rate as f32);
            (2.0 * std::f32::consts::PI * 440.0 * t).sin() // 440 Hz sine wave
        })
        .collect();

    let config = device.default_output_config().unwrap();
    let err_fn = |err| eprintln!("Stream error: {}", err);

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for (i, sample) in data.iter_mut().enumerate() {
                *sample = samples[i % samples.len()];
            }
        },
        err_fn,
        None
    );

    match stream {
        Ok(s) => {
            s.play().expect("Failed to play stream");
            std::thread::sleep(std::time::Duration::from_secs(duration_secs));
        }
        Err(err) => eprintln!("Failed to build stream: {}", err),
    }
}

fn play_audio_output(buffer: Arc<Mutex<Vec<f32>>>, devices: Vec<cpal::Device>, sample_rate: u32) {
    let mut handles = vec![];
    // let samples = Arc::new(f32_samples.clone());
    // let playback_duration = (f32_samples.len() as f32) / (sample_rate as f32);

    for device in devices {
        let buffer = Arc::clone(&buffer);
        // let samples = Arc::clone(&samples);
        let handle = thread::spawn(move || {
            let device_name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
            let config = match device.default_output_config() {
                Ok(cfg) => cfg,
                Err(err) => {
                    eprintln!("Failed to get default output config for {}: {}", device_name, err);
                    return;
                }
            };

            // println!("Default output config: {:?}", config);
            let clone_device_name = device_name.clone();
            let err_fn = move |err| eprintln!("Error on {}: {}", clone_device_name, err);

            let mut sample_clock = 0;

            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => {
                    device.build_output_stream(
                        &config.into(),
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            let mut buffer = buffer.lock().unwrap();
                            let len = buffer.len();

                            // Process samples in chunks
                            if len >= data.len() {
                                data.copy_from_slice(&buffer[..data.len()]);
                                buffer.drain(..data.len()); // Remove the used samples
                            } else {
                                data[..len].copy_from_slice(&buffer); // Use available samples
                                data[len..].fill(0.0); // Fill the rest with silence
                                buffer.clear();
                            }
                        },
                        err_fn,
                        None
                    )
                }
                _ => {
                    eprintln!("Unsupported sample format on {}", device_name);
                    return;
                }
            };

            if let Ok(stream) = stream {
                stream.play().expect("Failed to play stream");
                // println!("Playing samples with f32 format on {}", device_name);

                // Keep the thread running so the stream can play
                println!("Playing audio for 0.02 seconds on device {}", device_name); // playback_duration,
                std::thread::sleep(std::time::Duration::from_secs_f32(10.0));
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
}

fn wave_reader<T>(
    samples: &[T],
    f32_samples: &mut Vec<f32>,
    devices: Arc<Vec<Device>>,
    sample_rate: u32
)
    where T: Sample + ToSample<f32>
{
    f32_samples.clear();
    f32_samples.extend(samples.iter().map(|x: &T| T::to_sample::<f32>(*x)));
    // println!("writing to file");
    // println!("First 10 samples: {:?}", &f32_samples[..10]);
    // play_audio_output(f32_samples, devices.to_vec(), sample_rate);
    // let spec = hound::WavSpec {
    //     channels: 2,
    //     sample_rate: 44100,
    //     bits_per_sample: 32,
    //     sample_format: hound::SampleFormat::Float,
    // };

    // let mut writer = OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .append(true)
    //     .open("output_audio.wav")
    //     .expect("Failed to open WAV file");

    // for sample in f32_samples.iter() {
    //     writer.write_all(&sample.to_ne_bytes()).expect("Failed to write sample");
    // }
}
fn log(message: String) {
    println!("{}", message);
}
fn capture_output_audio(
    device: &cpal::Device,
    buffer: Arc<Mutex<Vec<f32>>>
) -> Option<cpal::Stream> {
    log(
        format!(
            "Capturing audio from: {}",
            device.name().expect("Could not get default audio device name")
        )
    );
    let mut f32_samples: Vec<f32> = Vec::with_capacity(16384);
    let audio_cfg = device.default_output_config().expect("No default output config found");
    let sample_rate = audio_cfg.sample_rate().0;
    log(format!("Default audio {:?}", audio_cfg));

    let stream = match
        device.build_input_stream(
            &audio_cfg.config(),

            move |data: &[f32], _: &cpal::InputCallbackInfo|
                // wave_reader::<f32>(data, &mut f32_samples.clone(), devices.clone(), sample_rate),
                {
                    // println!("adding to buffer");
                    let mut local_data: Vec<f32> = Vec::with_capacity(data.len());
                    local_data.extend_from_slice(data);

                    // Append local_data to buffer
                    let mut buffer = buffer.lock().unwrap();
                    buffer.extend(local_data);
                },
            capture_err_fn,
            None
        )
    {
        Ok(stream) => Some(stream),
        Err(e) => {
            log(format!("Error capturing f32 audio stream: {}", e));

            None
        }
    };

    stream

    // match audio_cfg.sample_format() {
    //     cpal::SampleFormat::F32 =>

    //     // cpal::SampleFormat::I16 => {
    //     //     match
    //     //         device.build_input_stream(
    //     //             &audio_cfg.config(),
    //     //             move |data, _: &_| wave_reader::<i16>(data),
    //     //             capture_err_fn,
    //     //             None
    //     //         )
    //     //     {
    //     //         Ok(stream) => Some(stream),
    //     //         Err(e) => {
    //     //             log(format!("Error capturing i16 audio stream: {}", e));
    //     //             None
    //     //         }
    //     //     }
    //     // }
    //     // cpal::SampleFormat::U16 => {
    //     //     match
    //     //         device.build_input_stream(
    //     //             &audio_cfg.config(),
    //     //             move |data, _: &_| wave_reader::<u16>(data),
    //     //             capture_err_fn,
    //     //             None
    //     //         )
    //     //     {
    //     //         Ok(stream) => Some(stream),
    //     //         Err(e) => {
    //     //             log(format!("Error capturing u16 audio stream: {}", e));
    //     //             None
    //     //         }
    //     //     }
    //     // }
    // }
}

/// capture_err_fn - called whan it's impossible to build an audio input stream
fn capture_err_fn(err: cpal::StreamError) {
    log(format!("Error {} building audio input stream", err));
}

fn main() {
    let host = cpal::default_host();
    let temp_devices = host.output_devices().expect("No output devices found");
    let device = host.default_output_device().expect("No output device found");
    //remove default_device from devices
    let devices = temp_devices
        .into_iter()
        .filter(|x| x.name().unwrap() != device.name().unwrap())
        .collect::<Vec<Device>>();
    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let cloned = Arc::clone(&buffer);
    let stream = capture_output_audio(&device, cloned).expect("No stream found");
    stream.play().expect("Failed to play stream");

    play_audio_output(buffer, devices, 44100);
    // play_test_audio(device);
    std::thread::sleep(std::time::Duration::from_secs(5));
}
