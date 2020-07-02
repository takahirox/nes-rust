use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::video::WindowContext;

use nes_rust::display::Display;
use nes_rust::display::SCREEN_WIDTH;
use nes_rust::display::SCREEN_HEIGHT;
use nes_rust::display::PIXEL_BYTES;
use nes_rust::display::PIXELS_CAPACITY;

pub struct Sdl2Display {
	pixels: [u8; PIXELS_CAPACITY],
	texture: Texture<'static>,
	renderer: Canvas<Window>
}

impl Sdl2Display {
	pub fn new(sdl: Sdl) -> Self {
		let video_subsystem = sdl.video().unwrap();

		let mut window_builder = video_subsystem.window(
			"nes-rust",
			SCREEN_WIDTH,
			SCREEN_HEIGHT
		);
		let window = window_builder.position_centered().build().unwrap();

		let renderer = window
			.into_canvas()
			.accelerated()
			.present_vsync()
			.build()
			.unwrap();
		let texture_creator = renderer.texture_creator();
		let texture_creator_pointer = &texture_creator as *const TextureCreator<WindowContext>;
		let texture = unsafe { &*texture_creator_pointer }
			.create_texture(
				PixelFormatEnum::RGB24,
				TextureAccess::Streaming,
				SCREEN_WIDTH,
				SCREEN_HEIGHT
			)
			.unwrap();

		Sdl2Display {
			pixels: [0; PIXELS_CAPACITY],
			texture: texture,
			renderer: renderer
		}
	}
}

impl Display for Sdl2Display {
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
		self.texture
			.update(None, &self.pixels, SCREEN_WIDTH as usize * PIXEL_BYTES as usize)
			.unwrap();
		self.renderer.clear();
		match self.renderer.copy(&self.texture, None, None) {
			Ok(()) => {},
			Err(_e) => {} // @TODO: Error handling
		};
		self.renderer.present();
	}

	fn copy_to_rgba_pixels(&self, _pixels: &mut [u8]) {
	}
}
