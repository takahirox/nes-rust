extern crate wasm_bindgen;
extern crate nes_rust;

mod wasm_audio;
mod wasm_display;
mod wasm_input;

use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use nes_rust::Nes;
use nes_rust::rom::Rom;
use nes_rust::display::PIXELS_CAPACITY;
use nes_rust::audio::BUFFER_CAPACITY;
use nes_rust::button;

use wasm_audio::WasmAudio;
use wasm_display::WasmDisplay;
use wasm_input::WasmInput;

// @TODO: Reuse button::Button instead of defining Button here

#[wasm_bindgen]
pub enum Button {
	Poweroff,
	Reset,
	Select,
	Start,
	Joypad1A,
	Joypad1B,
	Joypad1Up,
	Joypad1Down,
	Joypad1Left,
	Joypad1Right,
	Joypad2A,
	Joypad2B,
	Joypad2Up,
	Joypad2Down,
	Joypad2Left,
	Joypad2Right
}

fn to_button_internal(button: Button) -> button::Button {
	match button {
		Button::Poweroff => button::Button::Poweroff,
		Button::Reset => button::Button::Reset,
		Button::Select => button::Button::Select,
		Button::Start => button::Button::Start,
		Button::Joypad1A => button::Button::Joypad1A,
		Button::Joypad1B => button::Button::Joypad1B,
		Button::Joypad1Up => button::Button::Joypad1Up,
		Button::Joypad1Down => button::Button::Joypad1Down,
		Button::Joypad1Left => button::Button::Joypad1Left,
		Button::Joypad1Right => button::Button::Joypad1Right,
		Button::Joypad2A => button::Button::Joypad2A,
		Button::Joypad2B => button::Button::Joypad2B,
		Button::Joypad2Up => button::Button::Joypad2Up,
		Button::Joypad2Down => button::Button::Joypad2Down,
		Button::Joypad2Left => button::Button::Joypad2Left,
		Button::Joypad2Right => button::Button::Joypad2Right
	}
}

#[wasm_bindgen]
pub struct WasmNes {
	nes: Nes,
	pixels: [u8; PIXELS_CAPACITY],
	sample_buffer: [f32; BUFFER_CAPACITY]
}

#[wasm_bindgen]
impl WasmNes {
	pub fn new() -> Self {
		let input = Box::new(WasmInput::new());
		let display = Box::new(WasmDisplay::new());
		let audio = Box::new(WasmAudio::new());
		let nes = Nes::new(input, display, audio);

		WasmNes {
			nes: nes,
			pixels: [0; PIXELS_CAPACITY],
			sample_buffer: [0.0; BUFFER_CAPACITY]
		}
	}

	pub fn set_rom(&mut self, contents: Vec<u8>) {
		let rom = Rc::new(RefCell::new(Rom::new(contents)));
		self.nes.set_rom(rom);
	}

	pub fn bootup(&mut self) {
		self.nes.bootup();
	}

	pub fn reset(&mut self) {
		self.nes.reset();
	}

	pub fn step_frame(&mut self) {
		self.nes.step_frame();
	}

	pub fn update_pixels(&mut self) {
		self.nes.copy_pixels(&mut self.pixels);
	}

	pub fn pixels_ptr(&self) -> *const u8 {
		self.pixels.as_ptr()
	}

	pub fn update_sample_buffer(&mut self) {
		self.nes.copy_sample_buffer(&mut self.sample_buffer);
	}

	pub fn sample_buffer_ptr(&self) -> *const f32 {
		self.sample_buffer.as_ptr()
	}

	pub fn press_button(&mut self, button: Button) {
		self.nes.press_button(to_button_internal(button));
	}

	pub fn release_button(&mut self, button: Button) {
		self.nes.release_button(to_button_internal(button));
	}
}
