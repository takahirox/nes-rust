pub const SCREEN_WIDTH: u32 = 256;
pub const SCREEN_HEIGHT: u32 = 240;
pub const PIXEL_BYTES: u32 = 3;
pub const PIXELS_CAPACITY: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * PIXEL_BYTES as usize;

pub trait Display {
	fn render_pixel(&mut self, x: u16, y: u16, c: u32);
	fn update_screen(&mut self);
	fn copy_pixels(&self, pixels: &mut [u8; PIXELS_CAPACITY]);
}
