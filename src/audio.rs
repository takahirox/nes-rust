pub const BUFFER_CAPACITY: usize = 4096 * 2;

pub trait Audio {
	fn push(&mut self, value: f32);
	fn copy_sample_buffer(&mut self, sample_buffer: &mut [f32]);
}
