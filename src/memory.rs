pub struct Memory {
	data: Vec<u8>
}

impl Memory {
	pub fn new(vec: Vec<u8>) -> Self {
		Memory{ data: vec }
	}

	pub fn clear(&mut self) {
		for i in 0..self.capacity() {
			self.data[i as usize] = 0;
		}
	}

	pub fn capacity(&self) -> u32 {
		self.data.len() as u32
	}

	pub fn load(&self, address: u32) -> u8 {
		self.data[address as usize]
	}

	pub fn store(&mut self, address: u32, value: u8) {
		self.data[address as usize] = value;
	}
}

#[cfg(test)]
mod tests_memory {
	use super::*;

	#[test]
	fn initialize() {
		let m = Memory::new(vec![0; 1]);
		assert_eq!(0, m.load(0));
	}

	#[test]
	fn clear() {
		let mut m = Memory::new(vec![0; 16]);
		for i in 0..m.capacity() {
			m.store(i, 1);
		}
		m.clear();
		for i in 0..m.capacity() {
			assert_eq!(0, m.load(i));
		}
	}

	#[test]
	fn capacity() {
		let m = Memory::new(vec![0; 16]);
		assert_eq!(0x10, m.capacity());
	}

	#[test]
	fn store_and_load() {
		let mut m = Memory::new(vec![0; 16]);
		assert_eq!(0, m.load(1));
		m.store(1, 1);
		assert_eq!(1, m.load(1));
		assert_eq!(0, m.load(2));
	}
}
