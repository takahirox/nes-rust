use register::Register;

// @TODO: Implement. Otherwise games relying on APU IRQ don't work

pub struct Apu {
	cycle: u32,
	step: u16,
	frame_irq_active: bool,
	dmc_irq_active: bool,
	pub irq_interrupted: bool
}

impl Apu {
	pub fn new() -> Self {
		Apu {
			cycle: 0,
			step: 0,
			frame_irq_active: false,
			dmc_irq_active: false,
			irq_interrupted: false
		}
	}

	pub fn step(&mut self) {
		self.cycle += 1;
	}

	pub fn load_register(&mut self, address: u16) -> u8 {
		match address {
			0x4015 => {
				// @TODO: Implement properly
				0xFF
			},
			_ => 0
		}
	}

	pub fn store_register(&mut self, address: u16, value: u8) {

	}
}
