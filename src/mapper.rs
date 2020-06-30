pub struct MapperFactory;
use rom::Mirrorings;
use rom::RomHeader;
use register::Register;

impl MapperFactory {
	pub fn create(header: &RomHeader) -> Box<dyn Mapper> {
		match header.mapper_num() {
			0 => Box::new(NRomMapper::new(header)),
			1 => Box::new(MMC1Mapper::new(header)),
			2 => Box::new(UNRomMapper::new(header)),
			3 => Box::new(CNRomMapper::new()),
			4 => Box::new(MMC3Mapper::new(header)),
			_ => panic!("Unsupported mapper {}", header.mapper_num())
		}
	}
}

pub trait Mapper {
	// Maps 0x8000 - 0xFFFF to the program rom address
	fn map(&self, address: u32) -> u32;

	// Maps 0x0000 - 0x1FFF to the character rom address
	fn map_for_chr_rom(&self, address: u32) -> u32;

	// Writes control register inside in general
	fn store(&mut self, address: u32, value: u8);

	fn has_mirroring_type(&self) -> bool;

	fn mirroring_type(&self) -> Mirrorings;

	// @TODO: MMC3Mapper specific. Should this method be here?
	fn drive_irq_counter(&mut self) -> bool;
}

pub struct NRomMapper {
	program_bank_num: u8
}

impl NRomMapper {
	fn new(header: &RomHeader) -> Self {
		NRomMapper {
			program_bank_num: header.prg_rom_bank_num()
		}
	}
}

impl Mapper for NRomMapper {
	/**
	 * if program_bank_num == 1:
	 * 	0x8000 - 0xBFFF: 0x0000 - 0x3FFF
	 *	0xC000 - 0xFFFF: 0x0000 - 0x3FFF
	 * else:
	 * 	0x8000 - 0xFFFF: 0x0000 - 0x7FFF
	 */
	fn map(&self, mut address: u32) -> u32 {
		if self.program_bank_num == 1 && address >= 0xC000 {
			address -= 0x4000;
		}
		address - 0x8000
	}

	/**
	 * 0x0000 - 0x1FFF: 0x0000 - 0x1FFF
	 */
	fn map_for_chr_rom(&self, address: u32) -> u32 {
		address
	}

	/**
	 * Nothing to do
	 */
	fn store(&mut self, _address: u32, _value: u8) {
		// throw exception?
	}

	fn has_mirroring_type(&self) -> bool {
		false
	}

	fn mirroring_type(&self) -> Mirrorings {
		Mirrorings::SingleScreen // dummy
	}

	fn drive_irq_counter(&mut self) -> bool {
		false
	}
}

pub struct MMC1Mapper {
	program_bank_num: u8,
	control_register: Register<u8>,
	chr_bank0_register: Register<u8>,
	chr_bank1_register: Register<u8>,
	prg_bank_register: Register<u8>,
	latch: Register<u8>,
	register_write_count: u32
}

impl MMC1Mapper {
	fn new(header: &RomHeader) -> Self {
		let mut control_register = Register::<u8>::new();
		control_register.store(0x0C);
		MMC1Mapper {
			program_bank_num: header.prg_rom_bank_num(),
			control_register: control_register,
			chr_bank0_register: Register::<u8>::new(),
			chr_bank1_register: Register::<u8>::new(),
			prg_bank_register: Register::<u8>::new(),
			latch: Register::<u8>::new(),
			register_write_count: 0
		}
	}
}

impl Mapper for MMC1Mapper {
	fn map(&self, address: u32) -> u32 {
		let bank: u32;
		let mut offset = address & 0x3FFF;
		let bank_num = self.prg_bank_register.load() as u32 & 0x0F;

		match self.control_register.load_bits(2, 2) {
			0 | 1 => {
				// switch 32KB at 0x8000, ignoring low bit of bank number
				// TODO: Fix me
				offset = offset | (address & 0x4000);
				bank = bank_num & 0x0E;
			},
			2 => {
				// fix first bank at 0x8000 and switch 16KB bank at 0xC000
				bank = match address < 0xC000 {
					true => 0,
					false => bank_num
				};
			},
			_ /*3*/ => {
				// fix last bank at 0xC000 and switch 16KB bank at 0x8000
				bank = match address >= 0xC000 {
					true => self.program_bank_num as u32 - 1,
					false => bank_num
				};
			}
		};
		bank * 0x4000 + offset
	}

	fn map_for_chr_rom(&self, address: u32) -> u32 {
		let bank: u32;
		let mut offset = address & 0x0FFF;
		if self.control_register.load_bit(4) == 0 {
			// switch 8KB at a time
			bank = self.chr_bank0_register.load() as u32 & 0x1E;
			offset = offset | (address & 0x1000);
		} else {
			// switch two separate 4KB banks
			bank = match address < 0x1000 {
				true => self.chr_bank0_register.load(),
				false => self.chr_bank1_register.load()
			} as u32 & 0x1f;
		}
		bank * 0x1000 + offset
	}

	fn store(&mut self, address: u32, value: u8) {
		if (value & 0x80) != 0 {
			self.register_write_count = 0;
			self.latch.clear();
			if (address & 0x6000) == 0 {
				self.control_register.store_bits(2, 2, 3);
			}
		} else {
			self.latch.store(((value & 1) << 4) | (self.latch.load() >> 1));
			self.register_write_count += 1;

			if self.register_write_count >= 5 {
				let val = self.latch.load();
				match address & 0x6000 {
					0x0000 => self.control_register.store(val),
					0x2000 => self.chr_bank0_register.store(val),
					0x4000 => self.chr_bank1_register.store(val),
					_ /*0x6000*/ => self.prg_bank_register.store(val)
				};
				self.register_write_count = 0;
				self.latch.clear();
			}
		}
	}

	fn has_mirroring_type(&self) -> bool {
		true
	}

	fn mirroring_type(&self) -> Mirrorings {
		match self.control_register.load_bits(0, 2) {
			0 | 1 => Mirrorings::SingleScreen,
			2 => Mirrorings::Vertical,
			_ /*3*/ => Mirrorings::Horizontal
		}
	}

	fn drive_irq_counter(&mut self) -> bool {
		false
	}
}

struct UNRomMapper {
	program_bank_num: u8,
	register: Register<u8>
}

impl UNRomMapper {
	fn new(header: &RomHeader) -> Self {
		UNRomMapper {
			program_bank_num: header.prg_rom_bank_num(),
			register: Register::<u8>::new()
		}
	}
}

impl Mapper for UNRomMapper {
	fn map(&self, address: u32) -> u32 {
		let bank = match address < 0xC000 {
			true => self.register.load(),
			false => self.program_bank_num - 1
		} as u32;
		let offset = address & 0x3FFF;
		0x4000 * bank + offset
	}

	fn map_for_chr_rom(&self, address: u32) -> u32 {
		address
	}

	fn store(&mut self, _address: u32, value: u8) {
		self.register.store(value & 0xF);
	}

	fn has_mirroring_type(&self) -> bool {
		false
	}

	fn mirroring_type(&self) -> Mirrorings {
		Mirrorings::SingleScreen // dummy
	}

	fn drive_irq_counter(&mut self) -> bool {
		false
	}
}

struct CNRomMapper {
	register: Register<u8>
}

impl CNRomMapper {
	fn new() -> Self {
		CNRomMapper {
			register: Register::<u8>::new()
		}
	}
}

impl Mapper for CNRomMapper {
	fn map(&self, address: u32) -> u32 {
		address - 0x8000
	}

	fn map_for_chr_rom(&self, address: u32) -> u32 {
		self.register.load() as u32 * 0x2000 + (address & 0x1FFF)
	}

	fn store(&mut self, _address: u32, value: u8) {
		self.register.store(value & 0xF);
	}

	fn has_mirroring_type(&self) -> bool {
		false
	}

	fn mirroring_type(&self) -> Mirrorings {
		Mirrorings::SingleScreen // dummy
	}

	fn drive_irq_counter(&mut self) -> bool {
		false
	}
}

struct MMC3Mapper {
	program_bank_num: u8,
	character_bank_num: u8,
	register0: Register<u8>,
	register1: Register<u8>,
	register2: Register<u8>,
	register3: Register<u8>,
	register4: Register<u8>,
	register5: Register<u8>,
	register6: Register<u8>,
	register7: Register<u8>,
	program_register0: Register<u8>,
	program_register1: Register<u8>,
	character_register0: Register<u8>,
	character_register1: Register<u8>,
	character_register2: Register<u8>,
	character_register3: Register<u8>,
	character_register4: Register<u8>,
	character_register5: Register<u8>,
	irq_counter: u8,
	irq_counter_reload: bool,
	irq_enabled: bool
}

impl MMC3Mapper {
	fn new(header: &RomHeader) -> Self {
		MMC3Mapper {
			program_bank_num: header.prg_rom_bank_num(),
			character_bank_num: header.chr_rom_bank_num(),
			register0: Register::<u8>::new(),
			register1: Register::<u8>::new(),
			register2: Register::<u8>::new(),
			register3: Register::<u8>::new(),
			register4: Register::<u8>::new(),
			register5: Register::<u8>::new(),
			register6: Register::<u8>::new(),
			register7: Register::<u8>::new(),
			program_register0: Register::<u8>::new(),
			program_register1: Register::<u8>::new(),
			character_register0: Register::<u8>::new(),
			character_register1: Register::<u8>::new(),
			character_register2: Register::<u8>::new(),
			character_register3: Register::<u8>::new(),
			character_register4: Register::<u8>::new(),
			character_register5: Register::<u8>::new(),
			irq_counter: 0,
			irq_counter_reload: false,
			irq_enabled: true
		}
	}
}

impl Mapper for MMC3Mapper {
	fn map(&self, address: u32) -> u32 {
		let bank = match address {
			0x8000..=0x9FFF => match self.register0.is_bit_set(6) {
				true => self.program_bank_num * 2 - 2,
				false => self.program_register0.load()
			},
			0xA000..=0xBFFF => self.program_register1.load(),
			0xC000..=0xDFFF => match self.register0.is_bit_set(6) {
				true => self.program_register0.load(),
				false => self.program_bank_num * 2 - 2
			},
			_ => self.program_bank_num * 2 - 1
		};
		// I couldn't in the spec but it seems that
		// we need to wrap 2k bank with 4k program_bank_num
		((bank as u32) % ((self.program_bank_num as u32) * 2)) * 0x2000 + (address & 0x1FFF)
	}

	fn map_for_chr_rom(&self, address: u32) -> u32 {
		let bank = match self.register0.is_bit_set(7) {
			true => match address & 0x1FFF {
				0x0000..=0x03FF => self.character_register2.load(),
				0x0400..=0x07FF => self.character_register3.load(),
				0x0800..=0x0BFF => self.character_register4.load(),
				0x0C00..=0x0FFF => self.character_register5.load(),
				0x1000..=0x13FF => self.character_register0.load() & 0xFE,
				0x1400..=0x17FF => self.character_register0.load() | 1,
				0x1800..=0x1BFF => self.character_register1.load() & 0xFE,
				_ => self.character_register1.load() | 1
			},
			false => match address & 0x1FFF {
				0x0000..=0x03FF => self.character_register0.load() & 0xFE,
				0x0400..=0x07FF => self.character_register0.load() | 1,
				0x0800..=0x0BFF => self.character_register1.load() & 0xFE,
				0x0C00..=0x0FFF => self.character_register1.load() | 1,
				0x1000..=0x13FF => self.character_register2.load(),
				0x1400..=0x17FF => self.character_register3.load(),
				0x1800..=0x1BFF => self.character_register4.load(),
				_ => self.character_register5.load()
			}
		};
		// I couldn't in the spec but it seems that
		// we need to wrap 0.4k bank with 4k character_bank_num
		((bank as u32) % ((self.character_bank_num as u32) * 8)) * 0x400 + (address & 0x3FF)
	}

	fn store(&mut self, address: u32, value: u8) {
		match address {
			0x8000..=0x9FFF => match (address & 1) == 0 {
				true => self.register0.store(value),
				false => {
					self.register1.store(value);
					match self.register0.load_bits(0, 3) {
						0 => self.character_register0.store(value & 0xFE),
						1 => self.character_register1.store(value & 0xFE),
						2 => self.character_register2.store(value),
						3 => self.character_register3.store(value),
						4 => self.character_register4.store(value),
						5 => self.character_register5.store(value),
						6 => self.program_register0.store(value & 0x3F),
						_ => self.program_register1.store(value & 0x3F)
					};
				}
			},
			0xA000..=0xBFFF => match (address & 1) == 0 {
				true => self.register2.store(value),
				false => self.register3.store(value)
			},
			0xC000..=0xDFFF => {
				match (address & 1) == 0 {
					true => self.register4.store(value),
					false => self.register5.store(value)
				};
				self.irq_counter_reload = true;
			},
			_ => match (address & 1) == 0 {
				true => {
					self.register6.store(value);
					self.irq_enabled = false;
				},
				false => {
					self.register7.store(value);
					self.irq_enabled = true;
				}
			}
		};
	}

	fn has_mirroring_type(&self) -> bool {
		true
	}

	fn mirroring_type(&self) -> Mirrorings {
		match self.register2.is_bit_set(0) {
			true => Mirrorings::Horizontal,
			false => Mirrorings::Vertical
		}
	}

	fn drive_irq_counter(&mut self) -> bool {
		match self.irq_counter_reload {
			true => {
				self.irq_counter = self.register4.load();
				self.irq_counter_reload = false;
				false
			},
			false => match self.irq_enabled {
				true => match self.irq_counter > 0 {
					true => {
						self.irq_counter -= 1;
						match self.irq_counter == 0 {
							true => {
								self.irq_counter_reload = true;
								true
							}
							false => false
						}
					},
					false => false
				},
				false => false
			}
		}
	}
}

#[cfg(test)]
mod tests_nrom_mapper {
	use super::*;

	#[test]
	fn initialize() {
		NRomMapper{program_bank_num: 1};
	}

	#[test]
	fn map_with_program_bank_num_1() {
		let m = NRomMapper{program_bank_num: 1};
		assert_eq!(0x0000, m.map(0x8000));
		assert_eq!(0x3FFF, m.map(0xBFFF));
		assert_eq!(0x0000, m.map(0xC000));
		assert_eq!(0x3FFF, m.map(0xFFFF));
	}

	#[test]
	fn map_with_program_bank_num_2() {
		let m = NRomMapper{program_bank_num: 2};
		assert_eq!(0x0000, m.map(0x8000));
		assert_eq!(0x3FFF, m.map(0xBFFF));
		assert_eq!(0x4000, m.map(0xC000));
		assert_eq!(0x7FFF, m.map(0xFFFF));
	}

	#[test]
	fn map_for_chr_rom() {
		let m = NRomMapper{program_bank_num: 1};
		assert_eq!(0x0000, m.map_for_chr_rom(0x0000));
		assert_eq!(0x1FFF, m.map_for_chr_rom(0x1FFF));
	}
}
