use sdl2::audio::{AudioDevice, AudioCallback, AudioSpecDesired};
use sdl2::AudioSubsystem;

const BUFFER_CAPACITY: usize = 8192;
static mut buffer_index: usize = 0;
static mut buffer: [f32; BUFFER_CAPACITY] = [0.0; BUFFER_CAPACITY];

pub struct Audio {
	device: AudioDevice<NesAudioCallback>
}

struct NesAudioCallback {

}

impl AudioCallback for NesAudioCallback {
	type Channel = f32;

	fn callback(&mut self, buf: &mut [Self::Channel]) {
		// @TODO: Don't use unsafe
		unsafe {
			let mut i = 0;
			for b in buf.iter_mut() {
				if i >= BUFFER_CAPACITY || i >= buffer_index {
					break;
				}
				*b = buffer[i];
				i += 1;
			}
			buffer_index = 0;
		}
	}
}

impl Audio {
	pub fn new(subsystem: AudioSubsystem) -> Self {
		let spec = AudioSpecDesired {
			freq: Some(44100),
			channels: Some(1),
			samples: Some(4410)
		};
		Audio {
			device: subsystem.open_playback(None, &spec, |_| NesAudioCallback {}).unwrap(),
		}
	}

	pub fn resume(&self) {
		self.device.resume();
	}

	pub fn push(&mut self, value: f32) {
		// @TODO: Don't use unsafe
		unsafe {
			if buffer_index >= BUFFER_CAPACITY {
				return;
			}
			buffer[buffer_index] = value;
			buffer_index += 1;
		}
	}
}

