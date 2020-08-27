use sdl2::EventPump;
use sdl2::keyboard::Keycode;

use nes_rust::input::Input;
use nes_rust::button;

// @TODO: Be Configurable
fn keycode_to_button(key: Keycode) -> Option<button::Button> {
	match key {
		Keycode::Escape => Some(button::Button::Poweroff),
		// joypad1
		Keycode::Space => Some(button::Button::Start),
		Keycode::S => Some(button::Button::Select),
		Keycode::A => Some(button::Button::Joypad1A),
		Keycode::B => Some(button::Button::Joypad1B),
		Keycode::Up => Some(button::Button::Joypad1Up),
		Keycode::Down => Some(button::Button::Joypad1Down),
		Keycode::Left => Some(button::Button::Joypad1Left),
		Keycode::Right => Some(button::Button::Joypad1Right),
		// joypad2
		Keycode::X => Some(button::Button::Joypad2A),
		Keycode::Z => Some(button::Button::Joypad2B),
		Keycode::Num8 => Some(button::Button::Joypad2Up),
		Keycode::Num2 => Some(button::Button::Joypad2Down),
		Keycode::Num4 => Some(button::Button::Joypad2Left),
		Keycode::Num6 => Some(button::Button::Joypad2Right),
		_ => None
	}
}

pub struct Sdl2Input {
	event_pump: EventPump
}

impl Sdl2Input {
	pub fn new(event_pump: EventPump) -> Self {
		Sdl2Input {
			event_pump: event_pump
		}
	}
}

impl Input for Sdl2Input {
	fn get_input(&mut self) -> Option<(button::Button, button::Event)> {
		match self.event_pump.poll_event() {
			Some(ev) => {
				match ev {
					sdl2::event::Event::KeyDown {
						keycode: Some(key), ..
					} => {
						match keycode_to_button(key) {
							Some(button) => Some((button, button::Event::Press)),
							None => self.get_input()
						}
					},
					sdl2::event::Event::KeyUp {
						keycode: Some(key), ..
					} => {
						match keycode_to_button(key) {
							Some(button) => Some((button, button::Event::Release)),
							None => self.get_input()
						}
					},
					sdl2::event::Event::Quit { .. } => {
						Some((button::Button::Poweroff, button::Event::Press))
					},
					_ => self.get_input()
				}
			},
			None => None
		}
	}

	fn press(&mut self, _button: button::Button) {
		// Doesn't expect to be called
	}

	fn release(&mut self, _button: button::Button) {
		// Doesn't expect to be called
	}
}
