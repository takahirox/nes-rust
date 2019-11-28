pub trait Audio {
	fn resume(&self);
	fn push(&mut self, value: f32);
}
