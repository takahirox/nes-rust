use joypad;

pub trait Input {
	fn get_input(&mut self) -> Option<(joypad::Button, joypad::Event)>;
}
