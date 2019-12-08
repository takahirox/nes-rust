use button;

pub trait Input {
	fn get_input(&mut self) -> Option<(button::Button, button::Event)>;
	// The following two are WASM specific methods
	// @TODO: Remove
	fn press(&mut self, button: button::Button);
	fn release(&mut self, button: button::Button);
}
