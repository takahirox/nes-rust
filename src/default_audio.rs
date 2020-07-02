use audio::{Audio, BUFFER_CAPACITY};

pub struct DefaultAudio {
	buffer_index: usize,
	buffer: [f32; BUFFER_CAPACITY],
	previous_value: f32
}

impl DefaultAudio {
	pub fn new() -> Self {
		DefaultAudio {
			buffer_index: 0,
			buffer: [0.0; BUFFER_CAPACITY],
			previous_value: 0.0
		}
	}
}

impl Audio for DefaultAudio {
	fn push(&mut self, value: f32) {
		if self.buffer_index >= BUFFER_CAPACITY {
			return;
		}
		self.buffer[self.buffer_index] = value;
		self.buffer_index += 1;
	}

	fn copy_sample_buffer(&mut self, sample_buffer: &mut [f32]) {
		// @TODO: Remove side effect?

		// @TODO: Remove magic number
		let client_sample_buffer_length = 4096;

		for i in 0..client_sample_buffer_length {
			sample_buffer[i] = match i >= self.buffer_index {
				true => self.previous_value,
				false => self.buffer[i]
			};
			self.previous_value = sample_buffer[i];
		}
		for i in client_sample_buffer_length..self.buffer.len() {
			self.buffer[i - client_sample_buffer_length] = self.buffer[i];
		}
		self.buffer_index = match self.buffer_index < client_sample_buffer_length {
			true => 0,
			false => self.buffer_index - client_sample_buffer_length
		};
	}
}
