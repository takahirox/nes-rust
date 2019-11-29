use audio::Audio;
use audio::BUFFER_CAPACITY;
use sdl2::audio::{AudioDevice, AudioCallback, AudioSpecDesired};
use sdl2::AudioSubsystem;

static mut buffer_index: usize = 0;
static mut buffer: [f32; BUFFER_CAPACITY] = [0.0; BUFFER_CAPACITY];
static mut previous_value: f32 = 0.0;

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
				*b = match index >= buffer_index {
					true => previous_value,
					false => buffer[index]
				};
				previous_value = *b;
				*b *= self.volume;
				index += 1;
			}
			// @TODO: Optimize
			index = 0;
			for i in buf.len()..buffer_index {
				buffer[index] = buffer[i];
				index += 1;
			}
			buffer_index = index;
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
	fn resume(&self) {
		self.device.resume();
	}

	fn push(&mut self, value: f32) {
		// @TODO: Don't use unsafe
		unsafe {
			if buffer_index >= BUFFER_CAPACITY {
				return;
			}
			buffer[buffer_index] = value;
			buffer_index += 1;
		}
	}

	fn copy_sample_buffer(&mut self, sample_buffer: &mut [f32; BUFFER_CAPACITY]) {
	}
}
