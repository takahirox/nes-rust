extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

mod register;
mod memory;
mod mapper;
mod rom;
mod cpu;
mod ppu;
mod apu;
mod joypad;
mod nes;
mod display;
mod audio;
mod input;
mod wasm_audio;
mod wasm_display;
mod wasm_input;

use rom::Rom;
use nes::Nes;
use display::PIXELS_CAPACITY;
use audio::BUFFER_CAPACITY;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_audio::WasmAudio;
use wasm_display::WasmDisplay;
use wasm_input::WasmInput;

// @TODO: Reuse joypad::Button instead of defining Button here

#[wasm_bindgen]
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

fn to_joypad_button(button: Button) -> joypad::Button {
	match button {
		Button::A => joypad::Button::A,
		Button::B => joypad::Button::B,
		Button::Select => joypad::Button::Select,
		Button::Start => joypad::Button::Start,
		Button::Up => joypad::Button::Up,
		Button::Down => joypad::Button::Down,
		Button::Left => joypad::Button::Left,
		Button::Right => joypad::Button::Right
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
	pub fn new(contents: Vec<u8>) -> Self {
		let rom = Rc::new(RefCell::new(Rom::new(contents)));
		let input = Box::new(WasmInput::new());
		let display = Box::new(WasmDisplay::new());
		let audio = Box::new(WasmAudio::new());
		let mut nes = Nes::new(input, display, audio);
		nes.set_rom(rom);

		WasmNes {
			nes: nes,
			pixels: [0; PIXELS_CAPACITY],
			sample_buffer: [0.0; BUFFER_CAPACITY]
		}
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
		self.nes.press_button(to_joypad_button(button));
	}

	pub fn release_button(&mut self, button: Button) {
		self.nes.release_button(to_joypad_button(button));
	}
}
