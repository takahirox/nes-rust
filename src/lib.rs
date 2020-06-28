pub mod register;
pub mod cpu;
pub mod ppu;
pub mod apu;
pub mod rom;
pub mod memory;
pub mod mapper;
pub mod button;
pub mod input;
pub mod joypad;
pub mod audio;
pub mod display;

use std::cell::RefCell;
use std::rc::Rc;

use cpu::Cpu;
use rom::Rom;
use button::Button;
use input::Input;
use display::Display;
use display::PIXELS_CAPACITY;
use audio::Audio;
use audio::BUFFER_CAPACITY;

pub struct Nes {
	cpu: Cpu
}

impl Nes {
	pub fn new(input: Box<dyn Input>, display: Box<dyn Display>, audio: Box<dyn Audio>) -> Self {
		Nes {
			cpu: Cpu::new(
				input,
				display,
				audio
			)
		}
	}

	pub fn set_rom(&mut self, rom: Rc<RefCell<Rom>>) {
		self.cpu.set_rom(rom.clone());
	}

	pub fn step(&mut self) {
		self.cpu.step();
	}

	pub fn bootup(&mut self) {
		self.cpu.bootup();
	}

	pub fn reset(&mut self) {
		self.cpu.reset();
	}

	pub fn step_frame(&mut self) {
		self.cpu.step_frame();
	}

	pub fn copy_pixels(&self, pixels: &mut [u8; PIXELS_CAPACITY]) {
		self.cpu.get_ppu().get_display().copy_pixels(pixels);
	}

	pub fn copy_sample_buffer(&mut self, buffer: &mut [f32; BUFFER_CAPACITY]) {
		self.cpu.get_mut_apu().get_mut_audio().copy_sample_buffer(buffer);
	}

	pub fn press_button(&mut self, button: Button) {
		self.cpu.get_mut_input().press(button);
	}

	pub fn release_button(&mut self, button: Button) {
		self.cpu.get_mut_input().release(button);
	}
}
