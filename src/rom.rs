use memory::Memory;
use mapper::{Mapper, MapperFactory};

pub struct Rom {
	header: RomHeader,
	memory: Memory,
	mapper: Box<dyn Mapper>
}

pub static HEADER_SIZE: usize = 16;

pub enum Mirrorings {
	SingleScreen,
	Horizontal,
	Vertical,
	FourScreen
}

impl Rom {
	pub fn new(data: Vec<u8>) -> Self {
		let header = RomHeader::new(data[0..HEADER_SIZE].to_vec());
		let mapper = MapperFactory::create(&header);
		Rom {
			header: header,
			memory: Memory::new(data[HEADER_SIZE..].to_vec()),
			mapper: mapper
		}
	}

	/**
	 * CPU memory address:
	 * 0x0000 - 0x1FFF: Character ROM access
	 * 0x8000 - 0xFFFF: Program ROM access
	 *
	 * To access wide range ROM data with limited CPU memory address space
	 * Mapper maps CPU memory address to ROM's.
	 * In general writing control registers in Mapper via .store() switches bank.
	 */
	pub fn load(&self, address: u32) -> u8 {
		let mut address_in_rom = 0 as u32;
		if address < 0x2000 {
			// load from character rom
			address_in_rom += self.header.prg_rom_bank_num() as u32 * 0x4000;
			address_in_rom += self.mapper.map_for_chr_rom(address);
		} else {
			address_in_rom += self.mapper.map(address);
		}
		self.memory.load(address_in_rom)
	}

	pub fn load_without_mapping(&self, address: u32) -> u8 {
		self.memory.load(address)
	}

	/**
	 * In general writing with ROM address space updates control registers in Mapper.
	 */
	pub fn store(&mut self, address: u32, value: u8) {
		self.mapper.store(address, value);
	}

	pub fn valid(&self) -> bool {
		self.header.is_nes()
	}

	pub fn has_chr_rom(&self) -> bool {
		self.header.has_chr_rom()
	}

	pub fn mirroring_type(&self) -> Mirrorings {
		match self.mapper.has_mirroring_type() {
			true => self.mapper.mirroring_type(),
			false => self.header.mirroring_type()
		}
	}

	// @TODO: MMC3Mapper specific. Should this method be here?
	pub fn irq_interrupted(&mut self) -> bool {
		self.mapper.drive_irq_counter()
	}
}

// @TODO: Cache
pub struct RomHeader {
	data: Vec<u8>
}

impl RomHeader {
	fn new(vec: Vec<u8>) -> Self {
		let mut header = RomHeader {
			data: Vec::new()
		};
		for i in 0..HEADER_SIZE {
			header.data.push(vec[i]);
		}
		header
	}

	fn load(&self, address: u32) -> u8 {
		self.data[address as usize]
	}

	fn is_nes(&self) -> bool {
		if self.signature() == "NES" && self.magic_number() == 0x1a {
			return true;
		}
		false
	}

	fn signature(&self) -> String {
		let mut vec = Vec::new();
		for i in 0..3 as u32 {
			vec.push(self.load(i));
		}
		String::from_utf8(vec).unwrap()
	}

	fn magic_number(&self) -> u8 {
		self.load(3)
	}

	pub fn prg_rom_bank_num(&self) -> u8 {
		self.load(4)
	}

	pub fn chr_rom_bank_num(&self) -> u8 {
		self.load(5)
	}

	fn has_chr_rom(&self) -> bool {
		self.chr_rom_bank_num() > 0
	}

	fn control_byte1(&self) -> u8 {
		self.load(6)
	}

	fn control_byte2(&self) -> u8 {
		self.load(7)
	}

	fn _ram_bank_num(&self) -> u8 {
		self.load(8)
	}

	fn _unused_field(&self) -> u64 {
		let mut value = 0 as u64;
		for i in 0..7 as u32 {
			value = (value << 8) | self.load(9 + i) as u64;
		}
		value
	}

	fn extract_bits(&self, value: u8, offset: u8, size: u8) -> u8 {
		(value >> offset) & ((1 << size) - 1)
	}

	fn mirroring_type(&self) -> Mirrorings {
		match self.four_screen_mirroring() {
			true => Mirrorings::FourScreen,
			false => match self.extract_bits(self.control_byte1(), 0, 1) {
				0 => Mirrorings::Horizontal,
				_ /* 1 */ => Mirrorings::Vertical
			}
		}
	}

	fn _is_horizontal_mirroring(&self) -> bool {
		match self.mirroring_type() {
			Mirrorings::Horizontal => true,
			_ => false
		}
	}

	fn _battery_backed_ram(&self) -> u8 {
		self.extract_bits(self.control_byte1(), 1, 1)
	}

	fn _trainer_512_bytes(&self) -> u8 {
		self.extract_bits(self.control_byte1(), 2, 1)
	}

	fn four_screen_mirroring(&self) -> bool {
		self.extract_bits(self.control_byte1(), 3, 1) == 1
	}

	pub fn mapper_num(&self) -> u8 {
		let lower_bits = self.extract_bits(self.control_byte1(), 4, 4);
		let higher_bits = self.extract_bits(self.control_byte2(), 4, 4);
		(higher_bits << 4) | lower_bits
	}
}

#[cfg(test)]
mod tests_rom {
	use super::*;

	#[test]
	fn initialize() {
		let r = Rom::new(vec![0; 17]);
	}

	#[test]
	fn load() {
		let r = Rom::new(vec![0; 17]);
		assert_eq!(0, r.load(0));
	}

	#[test]
	fn store() {
		let mut r = Rom::new(vec![0; 17]);
		r.store(0, 0);
	}

	#[test]
	fn valid() {
		let r = Rom::new(vec![0; 64]);
		assert_eq!(false, r.valid());
		let mut v = vec![0; 64];
		v[0] = 0x4e; // N
		v[1] = 0x45; // E
		v[2] = 0x53; // S
		v[3] = 0x1a; // magic number
		let r2 = Rom::new(v);
		assert_eq!(true, r2.valid());
	}
}
