use display::{
	Display,
	PIXEL_BYTES,
	PIXELS_CAPACITY,
	SCREEN_HEIGHT,
	SCREEN_WIDTH
};

pub struct DefaultDisplay {
	pixels: Vec<u8>
}

impl DefaultDisplay {
	pub fn new() -> Self {
		DefaultDisplay {
			pixels: vec![0; PIXELS_CAPACITY]
		}
	}
}

impl Display for DefaultDisplay {
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

	fn vblank(&mut self) {
	}

	fn copy_to_rgba_pixels(&self, pixels: &mut [u8]) {
		for y in 0..SCREEN_HEIGHT {
			for x in 0..SCREEN_WIDTH {
				let base_index = (y * SCREEN_WIDTH + x) as usize;
				pixels[base_index * 4 + 0] = self.pixels[base_index * 3 + 0];
				pixels[base_index * 4 + 1] = self.pixels[base_index * 3 + 1];
				pixels[base_index * 4 + 2] = self.pixels[base_index * 3 + 2];
				pixels[base_index * 4 + 3] = 255;
			}
		}
	}
}
