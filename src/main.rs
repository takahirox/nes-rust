mod register;
mod memory;
mod mapper;
mod rom;
mod cpu;
mod ppu;
mod apu;
mod input;
mod joypad;
mod display;
mod nes;
extern crate sdl2;

use std::env;
use nes::Nes;
use rom::Rom;
use std::fs::File;
use std::io::Read;
use std::cell::RefCell;
use std::rc::Rc;
use input::Input;
use display::Display;

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();

	if args.len() < 2 {
		// @TODO: throw error
		return Ok(());
	}

	let filename = &args[1];
	let mut file = File::open(filename)?;
	let mut contents = vec![];
	file.read_to_end(&mut contents)?;
	let rom = Rc::new(RefCell::new(Rom::new(contents)));
	assert_eq!(rom.borrow().valid(), true);

	let sdl = sdl2::init().unwrap();
	let event_pump = sdl.event_pump().unwrap();
	let input = Input::new(event_pump);
	let display = Display::new(sdl);
	let mut nes = Nes::new(input, display);
	nes.set_rom(rom);
	nes.run();
	Ok(())
}
