use register::Register;
use audio::Audio;

/*
 * Audio Processing Unit implementation. Consists of
 *   - Pulse 1/2 channel
 *   - Triangle channel
 *   - Noise channel
 *   - DMC channel
 * Refer to https://wiki.nesdev.com/w/index.php/APU
 */
pub struct Apu {
	cycle: u32,
	step: u16,

	// CPU memory mapped sub units/registers

	pulse1: ApuPulse, // 0x4000 - 0x4003
	pulse2: ApuPulse, // 0x4004 - 0x4007
	triangle: ApuTriangle, // 0x4008 - 0x400B
	noise: ApuNoise, // 0x400C - 0x400F
	dmc: ApuDmc, // 0x4010 - 0x4013
	status: Register<u8>, // 0x4015
	frame: ApuFrameRegister, // 0x4017

	sample_period: u32,
	frame_irq_active: bool,
	dmc_irq_active: bool,
	pub irq_interrupted: bool,

	audio: Box<dyn Audio>
}

static LENGTH_TABLE: [u8; 32] = [
	0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06,
	0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
	0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16,
	0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E
];

impl Apu {
	pub fn new(audio: Box<dyn Audio>) -> Self {
		Apu {
			cycle: 0,
			step: 0,
			pulse1: ApuPulse::new(ApuPulseChannel::Channel1),
			pulse2: ApuPulse::new(ApuPulseChannel::Channel2),
			triangle: ApuTriangle::new(),
			noise: ApuNoise::new(),
			dmc: ApuDmc::new(),
			status: Register::<u8>::new(),
			frame: ApuFrameRegister::new(),
			sample_period: 1764000 / 44100, // @TODO: Fix me
			frame_irq_active: false,
			dmc_irq_active: false,
			irq_interrupted: false,
			audio: audio
		}
	}

	pub fn bootup(&mut self) {
		self.status.store(0x00);
	}

	pub fn reset(&mut self) {
		self.status.store(0x00);
		// @TODO: Implement properly
	}

	pub fn get_mut_audio(&mut self) -> &mut Box<dyn Audio> {
		&mut self.audio
	}

	// Expects being called at CPU clock rate
	pub fn step(&mut self, dmc_sample_data: u8) {
		self.cycle += 1;

		// Samping at sample rate timing
		// @TODO Fix me, more precise timing

		if (self.cycle % self.sample_period) == 0 {
			self.sample();
		}

		// Timers
		// Clocked on every CPU cycles for triangle and
		// every two CPU cycles for others

		if (self.cycle % 2) == 0 {
			self.pulse1.drive_timer();
			self.pulse2.drive_timer();
			self.noise.drive_timer();
			// @TODO: Add note
			if self.dmc.drive_timer(dmc_sample_data) {
				self.dmc_irq_active = true;
			}
		}

		self.triangle.drive_timer();

		// 240Hz Frame sequencer
		// @TODO: Fix me, more precise timing

		if (self.cycle % 7457) == 0 {
			if self.frame.five_step_mode() {
				// Five-step sequence
				//
				// 0 1 2 3 4    function
				// -----------  -----------------------------
				// - - - - -    IRQ (if bit 6 is clear)
				// l - l - -    Length counter and sweep
				// e e e e -    Envelope and linear counter

				if self.step < 4 {
					self.pulse1.drive_envelope();
					self.pulse2.drive_envelope();
					self.triangle.drive_linear();
					self.noise.drive_envelope();
				}

				if self.step == 0 || self.step == 2 {
					self.pulse1.drive_length();
					self.pulse1.drive_sweep();
					self.pulse2.drive_length();
					self.pulse2.drive_sweep();
					self.triangle.drive_length();
					self.noise.drive_length()
				}

				self.step = (self.step + 1) % 5;
			} else {
				// Four-step sequence
				//
				// 0 1 2 3    function
				// ---------  -----------------------------
				// - - - f    IRQ (if bit 6 is clear)
				// - l - l    Length counter and sweep
				// e e e e    Envelope and linear counter

				self.pulse1.drive_envelope();
				self.pulse2.drive_envelope();
				self.triangle.drive_linear();
				self.noise.drive_envelope();

				if self.step == 1 || self.step == 3 {
					self.pulse1.drive_length();
					self.pulse1.drive_sweep();
					self.pulse2.drive_length();
					self.pulse2.drive_sweep();
					self.triangle.drive_length();
					self.noise.drive_length();
				}

				if self.step == 3 && !self.frame.irq_disabled() {
					self.frame_irq_active = true;
				}

				// Seems like keep invoking IRQ once frame IRQ flag is on
				// until IRQ flag is cleared or it's disabled...?

				if self.frame_irq_active && !self.frame.irq_disabled() {
					self.irq_interrupted = true;
				}

				self.step = (self.step + 1) % 4;
			}

			// @TODO: check sending IRQ timing
			if self.dmc_irq_active {
				self.irq_interrupted = true;
			}
		}
	}

	pub fn load_register(&mut self, address: u16) -> u8 {
		match address {
			0x4015 => {
				// Loading status register
				//
				// bit
				//   7: DMC interrupt
				//   6: Frame interrupt
				//   4: DMC remaining bytes > 0
				//   3: Noise length counter > 0
				//   2: Triangle length couter > 0
				//   1: Pulse2 length counter > 0
				//   0: Pulse1 length counter > 0

				let mut value = 0;

				if self.dmc_irq_active {
					value |= 0x80;
				}

				if self.frame_irq_active && !self.frame.irq_disabled() {
					value |= 0x40;
				}

				if self.dmc.remaining_bytes_counter > 0 {
					value |= 0x10;
				}

				if self.noise.length_counter > 0 {
					value |= 0x08;
				}

				if self.triangle.length_counter > 0 {
					value |= 0x04;
				}

				if self.pulse2.length_counter > 0 {
					value |= 0x02;
				}

				if self.pulse1.length_counter > 0 {
					value |= 0x01;
				}

				// Loading status register clears the frame IRQ flag

				self.frame_irq_active = false;

				value
			},
			_ => 0
		}
	}

	pub fn store_register(&mut self, address: u16, value: u8) {
		match address {
			0x4000..=0x4003 => self.pulse1.store_register(address, value),
			0x4004..=0x4007 => self.pulse2.store_register(address, value),
			0x4008..=0x400B => self.triangle.store_register(address, value),
			0x400C..=0x400F => self.noise.store_register(address, value),
			0x4010..=0x4013 => self.dmc.store_register(address, value),
			0x4015 => {
				// Storing status register
				//
				// bit: Enable(1) / Disable(0)
				//   4: DMC unit
				//   3: Noise unit
				//   2: Triangle unit
				//   1: Pulse2 unit
				//   0: Pulse1 unit
				//
				// Writing a zero to any of channel enables bits will
				// set its length counter/remaining bytes to zero.

				self.status.store(value);

				self.dmc.set_enable((value & 0x10) == 0x10);
				self.noise.set_enable((value & 0x8) == 0x8);
				self.triangle.set_enable((value & 0x4) == 0x4);
				self.pulse2.set_enable((value & 0x2) == 0x2);
				self.pulse1.set_enable((value & 0x1) == 0x1);

				// Storing status register clears the DMC interrupt flag

				self.dmc_irq_active = false;
			},
			0x4017 => {
				// Storing frame counter register
				self.frame.store(value);

				// If interrupt inhibit flag is set, the frame IRQ flag is cleared.

				if self.frame.irq_disabled() {
					self.frame_irq_active = false;
				}
			},
			_ => {}
		};
	}

	// See cpu.step() for what the following two methods are for
	// @TODO: A bit hacky. Simplify.

	pub fn dmc_needs_cpu_memory_data(&self) -> bool {
		(self.cycle % 2) == 1 && self.dmc.needs_cpu_memory_data()
	}

	pub fn dmc_sample_address(&self) -> u16 {
		self.dmc.address_counter
	}

	fn sample(&mut self) {
		// Calculates the audio output within the range of 0.0 to 1.0.
		// Refer to https://wiki.nesdev.com/w/index.php/APU_Mixer

		let pulse1 = self.pulse1.output() as f32;
		let pulse2 = self.pulse2.output() as f32;
		let triangle = self.triangle.output() as f32;
		let noise = self.noise.output() as f32;
		let dmc = self.dmc.output() as f32;

		let mut pulse_out = 0.0;
		let mut tnd_out = 0.0;

		if pulse1 != 0.0 || pulse2 != 0.0 {
			pulse_out = 95.88 / ((8128.0 / (pulse1 + pulse2)) + 100.0);
		}

		if triangle != 0.0 || noise != 0.0 || dmc != 0.0 {
			tnd_out = 159.79 / (1.0 / (triangle / 8227.0 + noise / 12241.0 + dmc / 22638.0) + 100.0);
		}

		self.audio.push(pulse_out + tnd_out);
	}
}

/**
 * Apu Pulse channel. Consists of
 *   - Timer
 *   - Length counter
 *   - Envelope
 *   - Sweep
 */
struct ApuPulse {
	channel: ApuPulseChannel,
	register0: Register<u8>, // 0x4000, 0x4004
	register1: Register<u8>, // 0x4001, 0x4005
	register2: Register<u8>, // 0x4002, 0x4006
	register3: Register<u8>, // 0x4003, 0x4007
	enabled: bool,

	timer_counter: u16,
	timer_period: u16,
	timer_sequence: u8,

	envelope_start_flag: bool,
	envelope_counter: u8,
	envelope_decay_level_counter: u8,

	length_counter: u8,

	sweep_reload_flag: bool,
	sweep_counter: u8
}

static DUTY_TABLE: [u8; 32] = [
  0, 1, 0, 0, 0, 0, 0, 0,
  0, 1, 1, 0, 0, 0, 0, 0,
  0, 1, 1, 1, 1, 0, 0, 0,
  1, 0, 0, 1, 1, 1, 1, 1
];

enum ApuPulseChannel {
	Channel1,
	Channel2
}

impl ApuPulse {
	fn new(channel: ApuPulseChannel) -> Self {
		ApuPulse {
			channel: channel,
			register0: Register::<u8>::new(),
			register1: Register::<u8>::new(),
			register2: Register::<u8>::new(),
			register3: Register::<u8>::new(),
			enabled: false,
			timer_counter: 0,
			timer_period: 0,
			timer_sequence: 0,
			envelope_start_flag: true,
			envelope_counter: 0,
			envelope_decay_level_counter: 0,
			length_counter: 0,
			sweep_reload_flag: false,
			sweep_counter: 0
		}
	}

	fn store_register(&mut self, address: u16, value: u8) {
		match address & 0x4003 {
			0x4000 => self.register0.store(value),
			0x4001 => {
				self.register1.store(value);
				self.sweep_reload_flag = true;
			},
			0x4002 => {
				self.register2.store(value);
				self.timer_period = self.timer();
			},
			0x4003 => {
				self.register3.store(value);

				// Side effects
				//   - If the enabled flag is set, the length counter is reloaded
				//   - The envelope is restarted
				//   - The sequencer is immediately restarted at the first value of the current
				//     sequence. The period divider is not reset.

				if self.enabled {
					self.length_counter = LENGTH_TABLE[self.length_counter_index()  as usize];
				}
				self.timer_period = self.timer();
				self.timer_sequence = 0;
				self.envelope_start_flag = true;
			},
			_ => {} // @TODO: Throw an error?
		};
	}

	fn set_enable(&mut self, enabled: bool) {
		self.enabled = enabled;

		// When the enabled bit is cleared (via $4015), the length counter is forced to 0

		if !enabled {
			self.length_counter = 0;
		}
	}

	fn drive_timer(&mut self) {
		if self.timer_counter > 0 {
			self.timer_counter -= 1;
		} else {
			self.timer_counter = self.timer_period;
			self.timer_sequence += 1;

			// 8-step sequencer
			if self.timer_sequence == 8 {
				self.timer_sequence = 0
			}
		}
	}

	fn drive_length(&mut self) {
		if !self.envelope_loop_enabled() && self.length_counter > 0 {
			self.length_counter -= 1;
		}
	}

	fn drive_envelope(&mut self) {
		if self.envelope_start_flag {
			self.envelope_counter = self.envelope_period();
			self.envelope_decay_level_counter = 0xF;
			self.envelope_start_flag = false;
			return;
		}

		if self.envelope_counter > 0 {
			self.envelope_counter -= 1;
		} else {
			self.envelope_counter = self.envelope_period();
			if self.envelope_decay_level_counter > 0 {
				self.envelope_decay_level_counter -= 1;
			} else if self.envelope_decay_level_counter == 0 && self.envelope_loop_enabled() {
				self.envelope_decay_level_counter = 0xF;
			}
		}
	}

	fn drive_sweep(&mut self) {
		if self.sweep_counter == 0 &&
			self.sweep_enabled() &&
			self.sweep_shift_amount() != 0 &&
			self.timer_period >= 8 &&
			self.timer_period <= 0x7FF {

			let change = self.timer_period >> self.sweep_shift_amount();

			// In negated mode, Pulse 1 adds the ones' complement while
			// Pulse 2 adds the twos' complement

			self.timer_period += match self.negated_sweep() {
				// @TODO: Fix me
				true => match self.channel {
					ApuPulseChannel::Channel1 => !change,
					ApuPulseChannel::Channel2 => !change + 1
				},
				false => change
			};
		}

		if self.sweep_reload_flag || self.sweep_counter == 0 {
			self.sweep_reload_flag = false;
			self.sweep_counter = self.sweep_period();
		} else {
			self.sweep_counter -= 1;
		}
	}

	fn output(&self) -> u8 {
		if self.length_counter == 0 ||
			self.timer_period < 8 ||
			self.timer_period > 0x7FF ||
			DUTY_TABLE[(self.duty() * 8 + self.timer_sequence) as usize] == 0 {
			return 0;
		}

		// 4-bit output
		0x0F & match self.envelope_disabled() {
			true => self.envelope_period(),
			false => self.envelope_decay_level_counter
		}
	}

	fn duty(&self) -> u8 {
		self.register0.load_bits(6, 2)
	}

	fn envelope_loop_enabled(&self) -> bool {
		self.register0.is_bit_set(5)
	}

	fn envelope_disabled(&self) -> bool {
		self.register0.is_bit_set(4)
	}

	fn envelope_period(&self) -> u8 {
		self.register0.load_bits(0, 4)
	}

	fn sweep_enabled(&self) -> bool {
		self.register1.is_bit_set(7)
	}

	fn sweep_period(&self) -> u8 {
		self.register1.load_bits(4, 3)
	}

	fn negated_sweep(&self) -> bool {
		self.register1.is_bit_set(3)
	}

	fn sweep_shift_amount(&self) -> u8 {
		self.register1.load_bits(0, 3)
	}

	fn timer_low(&self) -> u8 {
		self.register2.load()
	}

	fn timer_high(&self) -> u8 {
		self.register3.load_bits(0, 3)
	}

	fn timer(&self) -> u16 {
		((self.timer_high() as u16) << 8) | self.timer_low() as u16
	}

	fn length_counter_index(&self) -> u8 {
		self.register3.load_bits(3, 5)
	}
}

/*
 * Apu Triangle channel. Consists of
 *   - Timer
 *   - Length counter
 *   - Linear counter
 */
struct ApuTriangle {
	register0: Register<u8>, // 0x4008
	register1: Register<u8>, // 0x4009
	register2: Register<u8>, // 0x400A
	register3: Register<u8>, // 0x400B
	enabled: bool,

	timer_counter: u16,
	timer_sequence: u8,

	length_counter: u8,

	linear_reload_flag: bool,
	linear_counter: u8
}

static SEQUENCE_TABLE: [u8; 32] = [
  15, 14, 13, 12, 11, 10,  9,  8,
   7,  6,  5,  4,  3,  2,  1,  0,
   0,  1,  2,  3,  4,  5,  6,  7,
   8,  9, 10, 11, 12, 13, 14, 15
];

impl ApuTriangle {
	fn new() -> Self {
		ApuTriangle {
			register0: Register::<u8>::new(),
			register1: Register::<u8>::new(),
			register2: Register::<u8>::new(),
			register3: Register::<u8>::new(),
			enabled: false,
			timer_counter: 0,
			timer_sequence: 0,
			length_counter: 0,
			linear_reload_flag: false,
			linear_counter: 0
		}
	}

	fn store_register(&mut self, address: u16, value: u8) {
		match address {
			0x4008 => self.register0.store(value),
			0x4009 => self.register1.store(value),
			0x400A => self.register2.store(value),
			0x400B => {
				self.register3.store(value);

				// Side effects
				//   - If the enabled flag is set, the length counter is reloaded
				//   - Sets the linear counter reload flag
				//   - The sequencer is immediately restarted at the first value of the current
				//     sequence. The period divider is not reset.
				if self.enabled {
					self.length_counter = LENGTH_TABLE[self.length_counter_index() as usize];
				}

				self.linear_reload_flag = true;
			},
			_ => {} // @TODO: Throw an error?
		};
	}

	fn set_enable(&mut self, enabled: bool) {
		self.enabled = enabled;

		// When the enabled bit is cleared (via $4015), the length counter is forced to 0

		if !enabled {
			self.length_counter = 0;
		}
	}

	fn drive_timer(&mut self) {
		if self.timer_counter > 0 {
			self.timer_counter -= 1;
		} else {
			self.timer_counter = self.timer();

			// The sequencer is clocked by the timer as long as
			// both the linear counter and the length counter are nonzero.

			if self.length_counter > 0 && self.linear_counter > 0 {
				self.timer_sequence += 1;

				// 32-step sequencer

				if self.timer_sequence == 32 {
					self.timer_sequence = 0;
				}
			}
		}
	}

	fn drive_linear(&mut self) {
		if self.linear_reload_flag {
			self.linear_counter = self.linear_counter();
		} else if self.linear_counter > 0 {
			self.linear_counter -= 1;
		}

		if !self.length_counter_disabled() {
			self.linear_reload_flag = false;
		}
	}

	fn drive_length(&mut self) {
		if !self.length_counter_disabled() && self.length_counter > 0 {
			self.length_counter -= 1;
		}
	}

	fn output(&self) -> u8 {
		if !self.enabled ||
			self.length_counter == 0 ||
			self.linear_counter == 0 ||
			self.timer() < 2 {
			return 0;
		}

		// 4-bit output
		return SEQUENCE_TABLE[self.timer_sequence as usize] & 0xF;
	}

	fn linear_counter(&self) -> u8 {
		self.register0.load_bits(0, 7)
	}

	fn length_counter_disabled(&self) -> bool {
		self.register0.is_bit_set(7)
	}

	fn timer_low(&self) -> u8 {
		self.register2.load()
	}

	fn length_counter_index(&self) -> u8 {
		self.register3.load_bits(3, 5)
	}

	fn timer_high(&self) -> u8 {
		self.register3.load_bits(0, 3)
	}

	fn timer(&self) -> u16 {
		((self.timer_high() as u16) << 8) | self.timer_low() as u16
	}
}

/*
 * Apu Noise channel. Consists of
 *   - Timer
 *   - Length counter
 *   - Envelope
 *   - Linear feedback shift register
 */
struct ApuNoise {
	register0: Register<u8>,  // 0x400C
	register1: Register<u8>,  // 0x400D
	register2: Register<u8>,  // 0x400E
	register3: Register<u8>,  // 0x400F

	enabled: bool,

	timer_counter: u16,
	timer_period: u16,

	envelope_start_flag: bool,
	envelope_counter: u8,
	envelope_decay_level_counter: u8,

	length_counter: u8,

	shift_register: u16  // 15-bit register
}

static NOISE_TIMER_TABLE: [u16; 16] = [
	0x004, 0x008, 0x010, 0x020,
	0x040, 0x060, 0x080, 0x0A0,
	0x0CA, 0x0FE, 0x17C, 0x1FC,
	0x2FA, 0x3F8, 0x7F2, 0xFE4
];

impl ApuNoise {
	fn new() -> Self {
		ApuNoise {
			register0: Register::<u8>::new(),
			register1: Register::<u8>::new(),
			register2: Register::<u8>::new(),
			register3: Register::<u8>::new(),
			enabled: false,
			timer_counter: 0,
			timer_period: 0,
			envelope_start_flag: false,
			envelope_counter: 0,
			envelope_decay_level_counter: 0,
			length_counter: 0,
			shift_register: 1
		}
	}

	fn store_register(&mut self, address: u16, value: u8) {
		match address {
			0x400C => self.register0.store(value),
			0x400D => self.register1.store(value),
			0x400E => {
				self.register2.store(value);
				self.timer_period = NOISE_TIMER_TABLE[self.timer_index() as usize];
			},
			0x400F => {
				self.register3.store(value);

				// Side effects
				//   - If the enabled flag is set, the length counter is reloaded
				//   - The envelope is restarted

				if self.enabled {
					self.length_counter = LENGTH_TABLE[self.length_counter_index() as usize];
				}

				self.envelope_start_flag = true;
			},
			_ => {} // @TODO: Throw an error?
		};
	}

	fn set_enable(&mut self, enabled: bool) {
		self.enabled = enabled;

		// When the enabled bit is cleared (via $4015), the length counter is forced to 0

		if !enabled {
			self.length_counter = 0;
		}
	}


	fn drive_timer(&mut self) {
		if self.timer_counter > 0 {
			self.timer_counter -= 1;
		} else {
			self.timer_counter = self.timer_period;

			// Feedback is calculated as the exclusive-OR of bit 0
			// and another bit: bit 6 if Mode flag is set, otherwise bit 1.

			let feedback = (self.shift_register & 1) ^
				(((self.shift_register >> match self.is_random() { true => 6, false => 1 })) & 1);

			self.shift_register = ((feedback as u16) << 14) | (self.shift_register >> 1);
		}
	}

	fn drive_envelope(&mut self) {
		if self.envelope_start_flag {
			self.envelope_counter = self.envelope_period();
			self.envelope_decay_level_counter = 0xF;
			self.envelope_start_flag = false;
			return;
		}

		if self.envelope_counter > 0 {
			self.envelope_counter -= 1;
		} else {
			self.envelope_counter = self.envelope_period();

			if self.envelope_decay_level_counter > 0 {
				self.envelope_decay_level_counter -= 1;
			} else if self.envelope_decay_level_counter == 0 &&
				self.length_counter_disabled() {
				self.envelope_decay_level_counter = 0xF;
			}
		}
	}

	fn drive_length(&mut self) {
		if !self.length_counter_disabled() && self.length_counter > 0 {
			self.length_counter -= 1;
		}
	}

	fn output(&self) -> u8 {
		if self.length_counter == 0 ||
			(self.shift_register & 1) == 1 {
			return 0;
		}

		// 4-bit output
		0x0F & match self.envelope_disabled() {
			true => self.envelope_period(),
			false => self.envelope_decay_level_counter
		}
	}

	fn length_counter_disabled(&self) -> bool {
		self.register0.is_bit_set(5)
	}

	fn envelope_disabled(&self) -> bool {
		self.register0.is_bit_set(4)
	}

	fn envelope_period(&self) -> u8 {
		self.register0.load_bits(0, 4)
	}

	fn is_random(&self) -> bool {
		self.register2.is_bit_set(7)
	}

	fn timer_index(&self) -> u8 {
		self.register2.load_bits(0, 4)
	}

	fn length_counter_index(&self) -> u8 {
		self.register3.load_bits(3, 5)
	}
}

/*
 * Apu DMC channel. Consists of
 *   - Timer
 *   - Memory reader
 *   - Sample buffer
 *   - Output unit
 */
struct ApuDmc {
	register0: Register<u8>, // 0x4010
	register1: Register<u8>, // 0x4011
	register2: Register<u8>, // 0x4012
	register3: Register<u8>, // 0x4013

	enabled: bool,

	timer_period: u16,
	timer_counter: u16,

	delta_counter: u8,
	address_counter: u16,
	remaining_bytes_counter: u16,

	sample_buffer: u8,
	sample_buffer_is_empty: bool,

	shift_register: u8,
	remaining_bits_counter: u8,

	silence_flag: bool
}

static DMC_TIMER_TABLE: [u16; 16] = [
	0x1AC, 0x17C, 0x154, 0x140,
	0x11E, 0x0FE, 0x0E2, 0x0D6,
	0x0BE, 0x0A0, 0x08E, 0x080,
	0x06A, 0x054, 0x048, 0x036
];

impl ApuDmc {
	fn new() -> Self {
		ApuDmc {
			register0: Register::<u8>::new(),
			register1: Register::<u8>::new(),
			register2: Register::<u8>::new(),
			register3: Register::<u8>::new(),
			enabled: false,
			timer_period: 0,
			timer_counter: 0,
			delta_counter: 0,
			address_counter: 0,
			remaining_bytes_counter: 0,
			sample_buffer: 0,
			sample_buffer_is_empty: true,
			shift_register: 0,
			remaining_bits_counter: 0,
			silence_flag: true
		}
	}

	fn store_register(&mut self, address: u16, value: u8) {
		match address {
			0x4010 => {
				self.register0.store(value);
				self.timer_period = DMC_TIMER_TABLE[self.timer_index() as usize] >> 1;
			},
			0x4011 => {
				self.register1.store(value);
				self.delta_counter = self.delta_counter();
			},
			0x4012 => {
				self.register2.store(value);
				self.address_counter = ((self.sample_address() as u16) << 6) | 0xC000;
			},
			0x4013 => {
				self.register3.store(value);
				self.remaining_bytes_counter = ((self.sample_length() as u16) << 4) | 1;
			},
			_ => {} // @TODO
		}
	}

	fn set_enable(&mut self, enabled: bool) {
		self.enabled = enabled;

		// If DMC enable flag is set via 0x4015,
		// the DMC sample will be restarted only if its remaining bytes is 0.

		if enabled {
			if self.remaining_bytes_counter == 0 {
				self.start();
			}
		} else {
			self.remaining_bytes_counter = 0;
		}
	}

	fn start(&mut self) {
		self.delta_counter = self.delta_counter();
		self.address_counter = ((self.sample_address() as u16) << 6) | 0xC000;
		self.remaining_bytes_counter = ((self.sample_length() as u16) << 4) | 1;
	}

	// See cpu.step() for what this method is for
	// @TODO: Solution to remove this workaround
	fn needs_cpu_memory_data(&self) -> bool {
		self.timer_counter == 0 &&
			self.remaining_bytes_counter > 0 &&
			self.sample_buffer_is_empty
	}

	fn drive_timer(&mut self, sample_data: u8) -> bool {
		let mut irq_active = false;
		if self.timer_counter > 0 {
			self.timer_counter -= 1;
		} else {
			self.timer_counter = self.timer_period;

			// Memory reader

			if self.remaining_bytes_counter > 0 && self.sample_buffer_is_empty {
				self.sample_buffer = sample_data;

				// if address exceeds 0xFFFF, it is wrapped around to 0x8000.
				self.address_counter = match self.address_counter {
					0xFFFF => 0x8000,
					_ => self.address_counter + 1
				};

				self.sample_buffer_is_empty = false;

				// If the bytes remaining counter becomes zero
				//   - the sample is restarted if the loop flag is set
				//   - otherwise, the interrupt flag is set if IRQ enabled flag is set

				self.remaining_bytes_counter -= 1;

				if self.remaining_bytes_counter == 0 {
					if self.is_loop() {
						self.start();
					} else if self.irq_enabled() {
						irq_active = true;
					}
				}
			}

			// Output unit

			if self.remaining_bits_counter == 0 {
				self.remaining_bits_counter = 8;
				if self.sample_buffer_is_empty {
					self.silence_flag = true;
				} else {
					self.silence_flag = false;
					self.sample_buffer_is_empty = true;
					self.shift_register = self.sample_buffer;
					self.sample_buffer = 0;
				}
			}

			if !self.silence_flag {
				if (self.shift_register & 1) == 0 {
					if self.delta_counter > 1 {
						self.delta_counter -= 2;
					}
				} else {
					if self.delta_counter < 126 {
						self.delta_counter += 2;
					}
				}
			}

			// The bits-remaining counter is updated whenever the timer outputs a clock
			self.remaining_bits_counter -= 1;
			self.shift_register = self.shift_register >> 1;
		}

		irq_active
	}

	fn output(&self) -> u8 {
		// Seems like we should ignore enable bit set via 0x4015
		// (or no enable bit in DMC unit?)

		// if !self.enabled {
		//	return 0;
		// }

		if self.silence_flag {
			return 0;
		}

		// 7-bit output
		self.delta_counter & 0x7F
	}

	fn irq_enabled(&self) -> bool {
		self.register0.is_bit_set(7)
	}

	fn is_loop(&self) -> bool {
		self.register0.is_bit_set(6)
	}

	fn timer_index(&self) -> u8 {
		self.register0.load_bits(0, 4)
	}

	fn delta_counter(&self) -> u8 {
		self.register1.load_bits(0, 7)
	}

	fn sample_address(&self) -> u8 {
		self.register2.load()
	}

	fn sample_length(&self) -> u8 {
		self.register3.load()
	}
}

struct ApuFrameRegister {
	register: Register<u8>
}

impl ApuFrameRegister {
	fn new() -> Self {
		ApuFrameRegister {
			register: Register::<u8>::new()
		}
	}

	fn store(&mut self, value: u8) {
		self.register.store(value);
	}

	fn five_step_mode(&self) -> bool {
		self.register.is_bit_set(7)
	}

	fn irq_disabled(&mut self) -> bool {
		self.register.is_bit_set(6)
	}
}
