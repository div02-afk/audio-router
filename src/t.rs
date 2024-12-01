use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let devices = host.output_devices()?;

    // Arc allows safe sharing of the `devices` iterator across threads
    let devices: Vec<_> = devices.collect();

    // Spawn a thread for each device
    let mut handles = vec![];
    for device in devices {
        let handle = thread::spawn(move || {
            let device_name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
            println!("Playing on device: {}", device_name);

            let config = match device.default_output_config() {
                Ok(cfg) => cfg,
                Err(err) => {
                    eprintln!("Failed to get default output config for {}: {}", device_name, err);
                    return;
                }
            };

            println!("Default output config: {:?}", config);
            let sample_rate = config.sample_rate().0 as f32;
            let mut sample_clock = 0f32;
            let frequency = 440.0; // A4 (440 Hz)
            let mut next_sample = move || {
                sample_clock = (sample_clock + 1.0) % sample_rate;
                (2.0 * PI * frequency * (sample_clock / sample_rate)).sin()
            };
            let clone_device_name = device_name.clone();
            let err_fn =move |err| eprintln!("Error on {}: {}", clone_device_name, err);

            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => device.build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        for sample in data.iter_mut() {
                            *sample = next_sample();
                        }
                    },
                    err_fn,
                    None,
                ),
                _ => {
                    eprintln!("Unsupported sample format on {}", device_name);
                    return;
                }
            };

            if let Ok(stream) = stream {
                stream.play().expect("Failed to play stream");
                println!("Playing sine wave with f32 format on {}", device_name);

                // Keep the thread running so the stream can play
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
