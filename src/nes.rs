use cpu::Cpu;
use ppu::Ppu;
use apu::Apu;
use rom::Rom;
use joypad::Joypad;
use std::cell::RefCell;
use std::rc::Rc;
use input::Input;
use display::Display;
use audio::Audio;
use std::time::Duration;

pub struct Nes {
	cpu: Cpu
}

impl Nes {
	pub fn new(input: Box<Input>, display: Box<Display>, audio: Box<Audio>) -> Self {
		Nes {
			cpu: Cpu::new(
				Joypad::new(input),
				Ppu::new(display),
				Apu::new(audio)
			)
		}
	}

	pub fn set_rom(&mut self, rom: Rc<RefCell<Rom>>) {
		self.cpu.set_rom(rom.clone());
	}

	pub fn run(&mut self) {
		self.cpu.bootup();
		while true {
			self.step_frame();
			// @TODO: Fix sleep duration time
			// @TODO: timer should depend on platform
			//        (For example we use requestAnimationFrame on WASM + Web)
			//        so should we move it out from this class?
			std::thread::sleep(Duration::from_millis(2));
		}
	}

	fn step(&mut self) {
		self.cpu.step();
	}

	fn step_frame(&mut self) {
		self.cpu.step_frame();
	}
}
