extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

mod register;
mod memory;
mod mapper;
mod rom;
mod cpu;
mod ppu;
mod apu;
mod button;
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
use input::Input;
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
	Joypad1_A,
	Joypad1_B,
	Joypad1_Up,
	Joypad1_Down,
	Joypad1_Left,
	Joypad1_Right,
	Joypad2_A,
	Joypad2_B,
	Joypad2_Up,
	Joypad2_Down,
	Joypad2_Left,
	Joypad2_Right
}

fn to_button_internal(button: Button) -> button::Button {
	match button {
		Button::Poweroff => button::Button::Poweroff,
		Button::Reset => button::Button::Reset,
		Button::Select => button::Button::Select,
		Button::Start => button::Button::Start,
		Button::Joypad1_A => button::Button::Joypad1_A,
		Button::Joypad1_B => button::Button::Joypad1_B,
		Button::Joypad1_Up => button::Button::Joypad1_Up,
		Button::Joypad1_Down => button::Button::Joypad1_Down,
		Button::Joypad1_Left => button::Button::Joypad1_Left,
		Button::Joypad1_Right => button::Button::Joypad1_Right,
		Button::Joypad2_A => button::Button::Joypad2_A,
		Button::Joypad2_B => button::Button::Joypad2_B,
		Button::Joypad2_Up => button::Button::Joypad2_Up,
		Button::Joypad2_Down => button::Button::Joypad2_Down,
		Button::Joypad2_Left => button::Button::Joypad2_Left,
		Button::Joypad2_Right => button::Button::Joypad2_Right
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
		self.nes.press_button(to_button_internal(button));
	}

	pub fn release_button(&mut self, button: Button) {
		self.nes.release_button(to_button_internal(button));
	}
}
