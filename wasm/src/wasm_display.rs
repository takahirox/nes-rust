use nes_rust::display::{
	Display,
	PIXEL_BYTES,
	PIXELS_CAPACITY,
	SCREEN_WIDTH
};

pub struct WasmDisplay {
	pixels: Vec<u8>
}

impl WasmDisplay {
	pub fn new() -> Self {
		WasmDisplay {
			pixels: vec![0; PIXELS_CAPACITY]
		}
	}
}

impl Display for WasmDisplay {
	fn render_pixel(&mut self, x: u16, y: u16, c: u32) {
		let r = ((c >> 16) & 0xff) as u8;
		let g = ((c >> 8) & 0xff) as u8;
		let b = (c & 0xff) as u8;
		let base_index = (y as u32 * SCREEN_WIDTH + x as u32) * PIXEL_BYTES;
		// Is this memory layout, BGR, correct?
		self.pixels[(base_index + 2) as usize] = r;
		self.pixels[(base_index + 1) as usize] = g;
		self.pixels[(base_index + 0) as usize] = b;
	}

	fn update_screen(&mut self) {
	}

	fn copy_pixels(&self, pixels: &mut [u8; PIXELS_CAPACITY]) {
		for i in 0..self.pixels.len() {
			pixels[i] = self.pixels[i];
		}
	}
}
