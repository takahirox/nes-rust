use cpu::Cpu;
use ppu::Ppu;
use apu::Apu;
use rom::Rom;
use joypad::Joypad;
use std::cell::RefCell;
use std::rc::Rc;
use input::Input;
use display::Display;

pub struct Nes {
	cpu: Cpu
}

impl Nes {
	pub fn new(input: Input, display: Display) -> Self {
		Nes {
			cpu: Cpu::new(Joypad::new(input), Ppu::new(display), Apu::new())
		}
	}

	pub fn set_rom(&mut self, rom: Rc<RefCell<Rom>>) {
		self.cpu.set_rom(rom.clone());
	}

	pub fn run(&mut self) {
		self.cpu.bootup();
		while true {
			self.step();
		}
	}

	fn step(&mut self) {
		self.cpu.step();
	}
}
