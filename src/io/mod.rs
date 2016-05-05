pub mod port;
mod pic;
mod pci;
mod ata;
pub mod keyboard;
pub mod timer;

pub use io::ata::Ata;
pub use io::port::{Io, Port};
pub use io::pic::Pics;
pub use io::keyboard::{Keyboard, KeyEvent};
pub use io::timer::handle_timer_interrupt;

pub static mut PICS: Pics = unsafe { Pics::new() };
pub static mut KEYBOARD: Keyboard = Keyboard::new();
pub static mut ATA: Ata = unsafe { Ata::new() };

pub fn init_io() {
	unsafe {
		PICS.init();
		self::timer::init_timer();
		//Enable interrupts
		asm!("sti");
		KEYBOARD.init_keyboard();
		self::pci::init_pci();
		ATA.init_ata();
	}
}
