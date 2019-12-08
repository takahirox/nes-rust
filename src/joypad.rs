use button;
use register::Register;

const BUTTON_NUM: u8 = 8;

pub enum Button {
	A,
	B,
	Select,
	Start,
	Up,
	Down,
	Left,
	Right
}

fn button_index(button: Button) -> usize {
	match button {
		Button::A => 0,
		Button::B => 1,
		Button::Select => 2,
		Button::Start => 3,
		Button::Up => 4,
		Button::Down => 5,
		Button::Left => 6,
		Button::Right => 7
	}
}

pub struct Joypad {
	register: Register<u8>,
	latch: u8,
	current_button: u8,
	buttons: [bool; BUTTON_NUM as usize]
}

impl Joypad {
	pub fn new() -> Self {
		Joypad {
			register: Register::<u8>::new(),
			latch: 0,
			current_button: 0,
			buttons: [false; BUTTON_NUM as usize]
		}
	}

	pub fn handle_input(&mut self, button: Button, event: button::Event) {
		match event {
			button::Event::Press => self.press_button(button),
			button::Event::Release => self.release_button(button)
		};
	}

	pub fn load_register(&mut self) -> u8 {
		let button = match self.latch == 1 {
			true => 1,
			_ => {
				let value = self.current_button;
				self.current_button += 1;
				value
			}
		};

		match button >= BUTTON_NUM || self.buttons[button as usize] {
			true => 1,
			false => 0
		}
	}

	pub fn store_register(&mut self, mut value: u8) {
		self.register.store(value);
		value = value & 1;
		if value == 1 {
			self.current_button = 0;
		}
		self.latch = value;
	}

	pub fn press_button(&mut self, button: Button) {
		self.buttons[button_index(button)] = true;
	}

	pub fn release_button(&mut self, button: Button) {
		self.buttons[button_index(button)] = false;
	}
}
