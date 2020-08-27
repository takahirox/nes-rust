extern crate nes_rust;
extern crate sdl2;

mod sdl2_input;
mod sdl2_display;
mod sdl2_audio;

use std::fs::File;
use std::io::Read;
use std::time::Duration;
use std::env;

use nes_rust::Nes;
use nes_rust::rom::Rom;

use sdl2_input::Sdl2Input;
use sdl2_display::Sdl2Display;
use sdl2_audio::Sdl2Audio;

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
	let rom = Rom::new(contents);
	assert_eq!(rom.valid(), true);

	let sdl = sdl2::init().unwrap();
	let event_pump = sdl.event_pump().unwrap();
	let audio_subsystem = sdl.audio().unwrap();
	let input = Box::new(Sdl2Input::new(event_pump));
	let display = Box::new(Sdl2Display::new(sdl));
	let audio = Box::new(Sdl2Audio::new(audio_subsystem));
	let mut nes = Nes::new(input, display, audio);
	nes.set_rom(rom);

	nes.bootup();
	loop {
		nes.step_frame();
		if !nes.is_power_on() {
			break;
		}
		// @TODO: Fix sleep duration time
		std::thread::sleep(Duration::from_millis(1));
	}
	Ok(())
}
