use sdl2::audio::{AudioDevice, AudioCallback, AudioSpecDesired};
use sdl2::AudioSubsystem;

use nes_rust::audio::Audio;
use nes_rust::audio::BUFFER_CAPACITY;

static mut BUFFER_INDEX: usize = 0;
static mut BUFFER: [f32; BUFFER_CAPACITY] = [0.0; BUFFER_CAPACITY];
static mut PREVIOUS_VALUE: f32 = 0.0;

pub struct Sdl2Audio {
	device: AudioDevice<NesAudioCallback>
}

struct NesAudioCallback {
	volume: f32
}

impl AudioCallback for NesAudioCallback {
	type Channel = f32;

	fn callback(&mut self, buf: &mut [Self::Channel]) {
		// @TODO: Don't use unsafe
		unsafe {
			let mut index = 0;
			for b in buf.iter_mut() {
				*b = match index >= BUFFER_INDEX {
					true => PREVIOUS_VALUE,
					false => BUFFER[index]
				};
				PREVIOUS_VALUE = *b;
				*b *= self.volume;
				index += 1;
			}
			// @TODO: Optimize
			index = 0;
			for i in buf.len()..BUFFER_INDEX {
				BUFFER[index] = BUFFER[i];
				index += 1;
			}
			BUFFER_INDEX = index;
		}
	}
}

impl Sdl2Audio {
	pub fn new(subsystem: AudioSubsystem) -> Self {
		let spec = AudioSpecDesired {
			freq: Some(44100),
			channels: Some(1),
			samples: Some(4096)
		};
		Sdl2Audio {
			device: subsystem.open_playback(
				None,
				&spec,
				|_| NesAudioCallback {volume: 0.25}
			).unwrap()
		}
	}
}

impl Audio for Sdl2Audio {
	fn push(&mut self, value: f32) {
		// @TODO: Don't use unsafe
		unsafe {
			if BUFFER_INDEX >= BUFFER_CAPACITY {
				return;
			}
			BUFFER[BUFFER_INDEX] = value;
			BUFFER_INDEX += 1;
		}
		self.device.resume();
	}

	fn copy_sample_buffer(&mut self, _sample_buffer: &mut [f32]) {
	}
}
