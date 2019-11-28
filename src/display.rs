pub trait Display {
	fn render_pixel(&mut self, x: u16, y: u16, c: u32);
	fn update_screen(&mut self);
}
