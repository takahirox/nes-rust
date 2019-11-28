use input::Input;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use joypad;

fn keycode_to_joypad_button(key: Keycode) -> Option<joypad::Button> {
	match key {
		Keycode::A => Some(joypad::Button::A),
		Keycode::B => Some(joypad::Button::B),
		Keycode::Space => Some(joypad::Button::Start),
		Keycode::S => Some(joypad::Button::Select),
		Keycode::Up => Some(joypad::Button::Up),
		Keycode::Down => Some(joypad::Button::Down),
		Keycode::Left => Some(joypad::Button::Left),
		Keycode::Right => Some(joypad::Button::Right),
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
	fn get_input(&mut self) -> Option<(joypad::Button, joypad::Event)> {
		match self.event_pump.poll_event() {
			Some(ev) => {
				match ev {
					sdl2::event::Event::KeyDown {
						keycode: Some(key), ..
					} => {
						match keycode_to_joypad_button(key) {
							Some(button) => Some((button, joypad::Event::Press)),
							None => self.get_input()
						}
					},
					sdl2::event::Event::KeyUp {
						keycode: Some(key), ..
					} => {
						match keycode_to_joypad_button(key) {
							Some(button) => Some((button, joypad::Event::Release)),
							None => self.get_input()
						}
					},
					_ => self.get_input()
				}
			},
			None => None
		}
	}
}
