extern crate wasm_bindgen;
extern crate nes_rust;

use wasm_bindgen::prelude::*;

use nes_rust::Nes;
use nes_rust::rom::Rom;
use nes_rust::button;
use nes_rust::default_input::DefaultInput;
use nes_rust::default_audio::DefaultAudio;
use nes_rust::default_display::DefaultDisplay;

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

/// `WasmNes` is an interface between user JavaScript code and
/// WebAssembly NES emulator. The following code is example
/// JavaScript user code.
///
/// ```ignore
/// // Create NES
/// const nes = WasmNes.new();
///
/// // Load Rom
/// nes.set_rom(new Uint8Array(romArrayBuffer));
///
/// // Set up Audio
/// const audioContext = AudioContext || webkitAudioContext;
/// const bufferLength = 4096;
/// const context = new audioContext({sampleRate: 44100});
/// const scriptProcessor = context.createScriptProcessor(bufferLength, 0, 1);
/// scriptProcessor.onaudioprocess = e => {
///   const data = e.outputBuffer.getChannelData(0);
///   nes.update_sample_buffer(data);
/// };
/// scriptProcessor.connect(context.destination);
///
/// // Set up screen resources
/// const width = 256;
/// const height = 240;
/// const canvas = document.createElement('canvas');
/// const ctx = canvas.getContext('2d');
/// const imageData = ctx.createImageData(width, height);
/// const pixels = new Uint8Array(imageData.data.buffer);
///
/// // animation frame loop
/// const stepFrame = () => {
///   requestAnimationFrame(stepFrame);
///   // Run emulator until screen is refreshed
///   nes.step_frame();
///   // Load screen pixels and render to canvas
///   nes.update_pixels(pixels);
///   ctx.putImageData(imageData, 0, 0);
/// };
///
/// // Go!
/// nes.bootup();
/// stepFrame();
/// ```
#[wasm_bindgen]
pub struct WasmNes {
	nes: Nes
}

#[wasm_bindgen]
impl WasmNes {
	/// Creates a `WasmNes`
	pub fn new() -> Self {
		let input = Box::new(DefaultInput::new());
		let display = Box::new(DefaultDisplay::new());
		let audio = Box::new(DefaultAudio::new());
		let nes = Nes::new(input, display, audio);
		WasmNes {
			nes: nes
		}
	}

	/// Sets up NES rom
	///
	/// # Arguments
	/// * `rom` Rom image binary `Uint8Array`
	pub fn set_rom(&mut self, contents: Vec<u8>) {
		self.nes.set_rom(Rom::new(contents));
	}

	/// Boots up
	pub fn bootup(&mut self) {
		self.nes.bootup();
	}

	/// Resets
	pub fn reset(&mut self) {
		self.nes.reset();
	}

	/// Executes a CPU cycle
	pub fn step(&mut self) {
		self.nes.step();
	}

	/// Executes a PPU (screen refresh) frame
	pub fn step_frame(&mut self) {
		self.nes.step_frame();
	}

	/// Copies RGB pixels of screen to passed RGBA pixels.
	/// The RGBA pixels length should be
	/// 245760 = 256(width) * 240(height) * 4(RGBA).
	/// A channel will be filled with 255(opaque).
	///
	/// # Arguments
	/// * `pixels` RGBA pixels `Uint8Array` or `Uint8ClampedArray`
	pub fn update_pixels(&mut self, pixels: &mut [u8]) {
		self.nes.copy_pixels(pixels);
	}

	/// Copies audio buffer to passed `Float32Array` buffer.
	/// The length should be 4096.
	///
	/// # Arguments
	/// * `buffer` Audio buffer `Float32Array`
	pub fn update_sample_buffer(&mut self, buffer: &mut [f32]) {
		self.nes.copy_sample_buffer(buffer);
	}

	/// Presses a pad button
	///
	/// # Arguments
	/// * `button`
	pub fn press_button(&mut self, button: Button) {
		self.nes.press_button(to_button_internal(button));
	}

	/// Releases a pad button
	///
	/// # Arguments
	/// * `buffer`
	pub fn release_button(&mut self, button: Button) {
		self.nes.release_button(to_button_internal(button));
	}
}
