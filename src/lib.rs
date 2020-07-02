pub mod register;
pub mod cpu;
pub mod ppu;
pub mod apu;
pub mod rom;
pub mod memory;
pub mod mapper;
pub mod button;
pub mod joypad;
pub mod input;
pub mod audio;
pub mod display;
pub mod default_input;
pub mod default_audio;
pub mod default_display;

use cpu::Cpu;
use rom::Rom;
use button::Button;
use input::Input;
use display::Display;
use audio::Audio;

/// NES emulator.
///
/// You need to implement [`input::Input`](./input/trait.Input.html),
/// [`display::Display`](./display/trait.Display.html), and
/// [`audio::Audio`](./audio/trait.Audio.html) traits for your platform
/// specific Input/Output.
///
/// ```ignore
/// // Create Nes with your platform specific Input/Output
/// let input = Box::new(MyInput::new());
/// let display = Box::new(MyDisplay::new());
/// let audio = Box::new(MyAudio::new());
/// let mut nes = Nes::new(input, display, audio);
///
/// // Load and set Rom from rom image binary
/// let filename = &args[1];
/// let mut file = File::open(filename)?;
/// let mut contents = vec![];
/// file.read_to_end(&mut contents)?;
/// let rom = Rom::new(contents);
/// nes.set_rom(rom);
///
/// // Go!
/// nes.bootup();
/// loop {
///   nes.step_frame();
///   std::thread::sleep(Duration::from_millis(1));
/// }
/// ```
///
/// Refer to [`cli/src/main.rs`](https://github.com/takahirox/nes-rust/blob/master/cli/src/main.rs)
/// as more concrete sample code.
pub struct Nes {
	cpu: Cpu
}

impl Nes {
	/// Creates a new `Nes`.
    /// You need to pass [`input::Input`](./input/trait.Input.html),
    /// [`display::Display`](./display/trait.Display.html), and
    /// [`audio::Audio`](./audio/trait.Audio.html) traits for your platform
    /// specific Input/Output.
    ///
    /// # Arguments
    /// * `input` For pad input
    /// * `display` For screen output
    /// * `audio` For audio output
	pub fn new(input: Box<dyn Input>, display: Box<dyn Display>,
		audio: Box<dyn Audio>) -> Self {
		Nes {
			cpu: Cpu::new(
				input,
				display,
				audio
			)
		}
	}

	/// Sets up NES rom
	///
	/// # Arguments
	/// * `rom`
	pub fn set_rom(&mut self, rom: Rom) {
		self.cpu.set_rom(rom);
	}

	/// Boots up
	pub fn bootup(&mut self) {
		self.cpu.bootup();
	}

	/// Resets
	pub fn reset(&mut self) {
		self.cpu.reset();
	}

	/// Executes a CPU cycle
	pub fn step(&mut self) {
		self.cpu.step();
	}

	/// Executes a PPU (screen refresh) frame
	pub fn step_frame(&mut self) {
		self.cpu.step_frame();
	}

	/// Copies RGB pixels of screen to passed pixels.
	/// The length and result should be specific to `display` passed via the constructor.
	///
	/// # Arguments
	/// * `pixels`
	pub fn copy_pixels(&self, pixels: &mut [u8]) {
		self.cpu.get_ppu().get_display().copy_to_rgba_pixels(pixels);
	}

	/// Copies audio buffer to passed buffer.
	/// The length and result should be specific to `audio` passed via the constructor.
	///
	/// # Arguments
	/// * `buffer`
	pub fn copy_sample_buffer(&mut self, buffer: &mut [f32]) {
		self.cpu.get_mut_apu().get_mut_audio().copy_sample_buffer(buffer);
	}

	/// Presses a pad button
	///
	/// # Arguments
	/// * `button`
	pub fn press_button(&mut self, button: Button) {
		self.cpu.get_mut_input().press(button);
	}

	/// Releases a pad button
	///
	/// # Arguments
	/// * `buffer`
	pub fn release_button(&mut self, button: Button) {
		self.cpu.get_mut_input().release(button);
	}
}
