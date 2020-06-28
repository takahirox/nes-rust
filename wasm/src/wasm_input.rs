use std::collections::VecDeque;

use nes_rust::input::Input;
use nes_rust::button;

pub struct WasmInput {
	events: VecDeque<(button::Button, button::Event)>
}

impl WasmInput {
	pub fn new() -> Self {
		WasmInput {
			events: VecDeque::<(button::Button, button::Event)>::new()
		}
	}
}

impl Input for WasmInput {
	fn get_input(&mut self) -> Option<(button::Button, button::Event)> {
		match self.events.len() > 0 {
			true => self.events.pop_front(),
			false => None
		}
	}

	fn press(&mut self, button: button::Button) {
		self.events.push_back((button, button::Event::Press));
	}

	fn release(&mut self, button: button::Button) {
		self.events.push_back((button, button::Event::Release));
	}
}
