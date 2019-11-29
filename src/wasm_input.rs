use input::Input;
use joypad;
use std::collections::VecDeque;

pub struct WasmInput {
	events: VecDeque<(joypad::Button, joypad::Event)>
}

impl WasmInput {
	pub fn new() -> Self {
		WasmInput {
			events: VecDeque::<(joypad::Button, joypad::Event)>::new()
		}
	}

	pub fn press(&mut self, button: joypad::Button) {
		self.events.push_back((button, joypad::Event::Press));
	}

	pub fn release(&mut self, button: joypad::Button) {
		self.events.push_back((button, joypad::Event::Release));
	}
}

impl Input for WasmInput {
	fn get_input(&mut self) -> Option<(joypad::Button, joypad::Event)> {
		match self.events.len() > 0 {
			true => self.events.pop_front(),
			false => None
		}
	}
}
