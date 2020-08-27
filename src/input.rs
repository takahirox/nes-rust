use button::{Button, Event};

pub trait Input {
	fn get_input(&mut self) -> Option<(Button, Event)>;
	fn press(&mut self, button: Button);
	fn release(&mut self, button: Button);
}
