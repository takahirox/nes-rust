pub struct Register<T> {
	data: T
}

// @TODO: Combine Register<u8> with Register<u16>

impl Register<u8> {
	pub fn new() -> Register<u8> {
		Register{ data: 0 }
	}

	pub fn get_width(&self) -> u8 {
		8
	}

	pub fn load(&self) -> u8 {
		self.data
	}

	pub fn load_bit(&self, pos: u8) -> u8 {
		(self.data >> pos) & 1
	}

	pub fn load_bits(&self, offset: u8, width: u8) -> u8 {
		(self.data >> offset) & ((1 << width) - 1)
	}

	pub fn store(&mut self, value: u8) {
		self.data = value;
	}

	pub fn store_bit(&mut self, pos: u8, value: u8) {
		self.data = self.data & !(1 << pos) | ((value & 1) << pos);
	}

	pub fn store_bits(&mut self, offset: u8, width: u8, value: u8) {
		let mask = (1 << width) - 1;
		self.data = self.data & !(mask << offset) | ((value & mask) << offset);
	}

	pub fn clear(&mut self) {
		self.data = 0;
	}

	pub fn set_bit(&mut self, pos: u8) {
		self.store_bit(pos, 1);
	}

	pub fn clear_bit(&mut self, pos: u8) {
		self.store_bit(pos, 0);
	}

	pub fn is_bit_set(&self, pos: u8) -> bool {
		self.load_bit(pos) == 1
	}

	pub fn increment(&mut self) {
		self.add(1);
	}

	pub fn increment_by_2(&mut self) {
		self.add(2);
	}

	pub fn add(&mut self, value: u8) {
		self.data = self.data.wrapping_add(value);
	}

	pub fn decrement(&mut self) {
		self.sub(1);
	}

	pub fn decrement_by_2(&mut self) {
		self.sub(2);
	}

	pub fn sub(&mut self, value: u8) {
		self.data = self.data.wrapping_sub(value);
	}

	pub fn shift(&mut self, value: u8) -> u8 {
		let carry = self.load_bit(self.get_width() - 1);
		self.data = (self.data << 1) | (value & 1);
		carry
	}

	pub fn dump(&self) -> String {
		format!("0x{:02x}", self.data)
	}
}

impl Register<u16> {
	pub fn new() -> Register<u16> {
		Register{ data: 0 }
	}

	pub fn get_width(&self) -> u8 {
		16
	}

	pub fn load(&self) -> u16 {
		self.data
	}

	pub fn load_bit(&self, pos: u8) -> u8 {
		((self.data >> pos) & 1) as u8
	}

	pub fn load_bits(&self, offset: u8, width: u8) -> u16 {
		(self.data >> offset) & ((1 << width) - 1)
	}

	pub fn store(&mut self, value: u16) {
		self.data = value;
	}

	pub fn store_bit(&mut self, pos: u8, value: u8) {
		self.data = self.data & !(1 << pos) | ((value as u16 & 1) << pos);
	}

	pub fn store_bits(&mut self, offset: u8, width: u8, value: u16) {
		let mask = (1 << width) - 1;
		self.data = self.data & !(mask << offset) | ((value & mask) << offset);
	}

	pub fn clear(&mut self) {
		self.data = 0;
	}

	pub fn set_bit(&mut self, pos: u8) {
		self.store_bit(pos, 1);
	}

	pub fn clear_bit(&mut self, pos: u8) {
		self.store_bit(pos, 0);
	}

	pub fn is_bit_set(&self, pos: u8) -> bool {
		self.load_bit(pos) == 1
	}

	pub fn increment(&mut self) {
		self.add(1);
	}

	pub fn increment_by_2(&mut self) {
		self.add(2);
	}

	pub fn add(&mut self, value: u16) {
		self.data = self.data.wrapping_add(value);
	}

	pub fn decrement(&mut self) {
		self.sub(1);
	}

	pub fn decrement_by_2(&mut self) {
		self.sub(2);
	}

	pub fn sub(&mut self, value: u16) {
		self.data = self.data.wrapping_sub(value);
	}

	pub fn shift(&mut self, value: u8) -> u8 {
		let carry = self.load_bit(self.get_width() - 1);
		self.data = (self.data << 1) | (value as u16 & 1);
		carry
	}

	pub fn store_higher_byte(&mut self, value: u8) {
		self.data &= 0x00ff;
		self.data |= (value as u16) << 8;
	}

	pub fn store_lower_byte(&mut self, value: u8) {
		self.data &= 0xff00;
		self.data |= value as u16;
	}

	pub fn dump(&self) -> String {
		format!("0x{:04x}", self.data)
	}
}

#[cfg(test)]
mod tests_register_u8 {
	use super::*;

	#[test]
	fn initial_value() {
		let r = Register::<u8>::new();
		assert_eq!(0, r.data);
	}

	#[test]
	fn get_width() {
		let r = Register::<u8>::new();
		assert_eq!(8, r.get_width());
	}

	#[test]
	fn load() {
		let r = Register::<u8>::new();
		assert_eq!(0, r.load());
	}

	#[test]
	fn load_bit() {
		let mut r = Register::<u8>::new();
		r.store(2);
		assert_eq!(1, r.load_bit(1));
	}

	#[test]
	fn load_bits() {
		let mut r = Register::<u8>::new();
		r.store(2);
		assert_eq!(1, r.load_bits(1, 2));
	}

	#[test]
	fn store() {
		let mut r = Register::<u8>::new();
		r.store(0xFF);
		assert_eq!(0xFF, r.load());
		r.store(0);
		assert_eq!(0, r.load());
	}

	#[test]
	fn store_bit() {
		let mut r = Register::<u8>::new();
		r.store_bit(1, 1);
		assert_eq!(2, r.load());
		r.store(0xFF);
		r.store_bit(1, 0);
		assert_eq!(0xFD, r.load());
	}

	#[test]
	fn store_bits() {
		let mut r = Register::<u8>::new();
		r.store_bits(1, 2, 3);
		assert_eq!(6, r.load());
		r.store(0xFF);
		r.store_bits(1, 2, 0);
		assert_eq!(0xF9, r.load());
	}

	#[test]
	fn clear() {
		let mut r = Register::<u8>::new();
		r.store(0xFF);
		r.clear();
		assert_eq!(0, r.load());
	}

	#[test]
	fn set_bit() {
		let mut r = Register::<u8>::new();
		r.set_bit(1);
		assert_eq!(2, r.load());
	}

	#[test]
	fn clear_bit() {
		let mut r = Register::<u8>::new();
		r.store(0xFF);
		r.clear_bit(1);
		assert_eq!(0xFD, r.load());
	}

	#[test]
	fn is_bit_set() {
		let mut r = Register::<u8>::new();
		assert_eq!(false, r.is_bit_set(1));
		r.set_bit(1);
		assert_eq!(true, r.is_bit_set(1));
	}

	#[test]
	fn increment() {
		let mut r = Register::<u8>::new();
		r.increment();
		assert_eq!(1, r.load());
		r.store(0xFF);
		r.increment();
		assert_eq!(0, r.load());
	}

	#[test]
	fn increment_by_2() {
		let mut r = Register::<u8>::new();
		r.increment_by_2();
		assert_eq!(2, r.load());
		r.store(0xFF);
		r.increment_by_2();
		assert_eq!(1, r.load());
	}

	#[test]
	fn add() {
		let mut r = Register::<u8>::new();
		r.add(3);
		assert_eq!(3, r.load());
		r.store(0xFF);
		r.add(3);
		assert_eq!(2, r.load());
	}

	#[test]
	fn decrement() {
		let mut r = Register::<u8>::new();
		r.decrement();
		assert_eq!(0xFF, r.load());
		r.decrement();
		assert_eq!(0xFE, r.load());
	}

	#[test]
	fn decrement_by_2() {
		let mut r = Register::<u8>::new();
		r.decrement_by_2();
		assert_eq!(0xFE, r.load());
		r.decrement_by_2();
		assert_eq!(0xFC, r.load());
	}

	#[test]
	fn sub() {
		let mut r = Register::<u8>::new();
		r.sub(3);
		assert_eq!(0xFD, r.load());
		r.sub(3);
		assert_eq!(0xFA, r.load());
	}

	#[test]
	fn shift() {
		let mut r = Register::<u8>::new();
		assert_eq!(0, r.shift(1));
		assert_eq!(1, r.load());
		assert_eq!(0, r.shift(0));
		assert_eq!(2, r.load());
		r.store(0xFF);
		assert_eq!(1, r.shift(1));
		assert_eq!(0xFF, r.load());
		assert_eq!(1, r.shift(0));
		assert_eq!(0xFE, r.load());
	}

	#[test]
	fn dump() {
		let mut r = Register::<u8>::new();
		assert_eq!("0x00", r.dump());
		r.store(0xFF);
		assert_eq!("0xff", r.dump());
		r.store(0x8A);
		assert_eq!("0x8a", r.dump());
	}
}


#[cfg(test)]
mod tests_register_u16 {
	use super::*;

	#[test]
	fn initial_value() {
		let r = Register::<u16>::new();
		assert_eq!(0, r.data);
	}

	#[test]
	fn get_width() {
		let r = Register::<u16>::new();
		assert_eq!(16, r.get_width());
	}

	#[test]
	fn load() {
		let r = Register::<u16>::new();
		assert_eq!(0, r.load());
	}

	#[test]
	fn load_bit() {
		let mut r = Register::<u16>::new();
		r.store(2);
		assert_eq!(1, r.load_bit(1));
	}

	#[test]
	fn load_bits() {
		let mut r = Register::<u16>::new();
		r.store(2);
		assert_eq!(1, r.load_bits(1, 2));
	}

	#[test]
	fn store() {
		let mut r = Register::<u16>::new();
		r.store(0xFF);
		assert_eq!(0xFF, r.load());
		r.store(0);
		assert_eq!(0, r.load());
	}

	#[test]
	fn store_bit() {
		let mut r = Register::<u16>::new();
		r.store_bit(1, 1);
		assert_eq!(2, r.load());
		r.store(0xFF);
		r.store_bit(1, 0);
		assert_eq!(0xFD, r.load());
	}

	#[test]
	fn store_bits() {
		let mut r = Register::<u16>::new();
		r.store_bits(1, 2, 3);
		assert_eq!(6, r.load());
		r.store(0xFF);
		r.store_bits(1, 2, 0);
		assert_eq!(0xF9, r.load());
	}

	#[test]
	fn clear() {
		let mut r = Register::<u16>::new();
		r.store(0xFF);
		r.clear();
		assert_eq!(0, r.load());
	}

	#[test]
	fn set_bit() {
		let mut r = Register::<u16>::new();
		r.set_bit(1);
		assert_eq!(2, r.load());
	}

	#[test]
	fn clear_bit() {
		let mut r = Register::<u16>::new();
		r.store(0xFF);
		r.clear_bit(1);
		assert_eq!(0xFD, r.load());
	}

	#[test]
	fn is_bit_set() {
		let mut r = Register::<u16>::new();
		assert_eq!(false, r.is_bit_set(1));
		r.set_bit(1);
		assert_eq!(true, r.is_bit_set(1));
	}

	#[test]
	fn increment() {
		let mut r = Register::<u16>::new();
		r.increment();
		assert_eq!(1, r.load());
		r.store(0xFF);
		r.increment();
		assert_eq!(0x100, r.load());
		r.store(0xFFFF);
		r.increment();
		assert_eq!(0, r.load());
	}

	#[test]
	fn increment_by_2() {
		let mut r = Register::<u16>::new();
		r.increment_by_2();
		assert_eq!(2, r.load());
		r.store(0xFF);
		r.increment_by_2();
		assert_eq!(0x101, r.load());
		r.store(0xFFFF);
		r.increment_by_2();
		assert_eq!(1, r.load());
	}

	#[test]
	fn add() {
		let mut r = Register::<u16>::new();
		r.add(3);
		assert_eq!(3, r.load());
		r.store(0xFF);
		r.add(3);
		assert_eq!(0x102, r.load());
		r.store(0xFFFF);
		r.add(3);
		assert_eq!(2, r.load());
	}

	#[test]
	fn decrement() {
		let mut r = Register::<u16>::new();
		r.decrement();
		assert_eq!(0xFFFF, r.load());
		r.decrement();
		assert_eq!(0xFFFE, r.load());
	}

	#[test]
	fn decrement_by_2() {
		let mut r = Register::<u16>::new();
		r.decrement_by_2();
		assert_eq!(0xFFFE, r.load());
		r.decrement_by_2();
		assert_eq!(0xFFFC, r.load());
	}

	#[test]
	fn sub() {
		let mut r = Register::<u16>::new();
		r.sub(3);
		assert_eq!(0xFFFD, r.load());
		r.sub(3);
		assert_eq!(0xFFFA, r.load());
	}

	#[test]
	fn shift() {
		let mut r = Register::<u16>::new();
		assert_eq!(0, r.shift(1));
		assert_eq!(1, r.load());
		assert_eq!(0, r.shift(0));
		assert_eq!(2, r.load());
		r.store(0xFF);
		assert_eq!(0, r.shift(1));
		assert_eq!(0x1FF, r.load());
		assert_eq!(0, r.shift(0));
		assert_eq!(0x3FE, r.load());
		r.store(0xFFFF);
		assert_eq!(1, r.shift(1));
		assert_eq!(0xFFFF, r.load());
		assert_eq!(1, r.shift(0));
		assert_eq!(0xFFFE, r.load());
	}

	#[test]
	fn dump() {
		let mut r = Register::<u16>::new();
		assert_eq!("0x0000", r.dump());
		r.store(0xFF);
		assert_eq!("0x00ff", r.dump());
		r.store(0x8A);
		assert_eq!("0x008a", r.dump());
		r.store(0xFFFF);
		assert_eq!("0xffff", r.dump());
		r.store(0xA88A);
		assert_eq!("0xa88a", r.dump());
	}
}
