use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::video::WindowContext;

pub struct Display {
	pixels: [u8; 256 * 240 * 3],
	texture: Texture<'static>,
	renderer: Canvas<Window>
}

impl Display {
	pub fn new(sdl: Sdl) -> Self {
		let video_subsystem = sdl.video().unwrap();

		let mut window_builder = video_subsystem.window(
			"nes-rust",
			256,
			240
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
				256,
				240
			)
			.unwrap();

		Display {
			pixels: [0; 256 * 240 * 3],
			texture: texture,
			renderer: renderer
		}
	}

	pub fn render_pixel(&mut self, x: u16, y: u16, c: u32) {
		let r = ((c >> 16) & 0xff) as u8;
		let g = ((c >> 8) & 0xff) as u8;
		let b = (c & 0xff) as u8;
		let base_index = (y as u32 * 256 + x as u32) * 3;
		// Is this memory layout, BGR, correct?
		self.pixels[(base_index + 2) as usize] = r;
		self.pixels[(base_index + 1) as usize] = g;
		self.pixels[(base_index + 0) as usize] = b;
	}

	pub fn update_screen(&mut self) {
		self.texture
			.update(None, &self.pixels, 256 * 3)
			.unwrap();
		self.renderer.clear();
		self.renderer.copy(&self.texture, None, None);
		self.renderer.present();
	}
}
