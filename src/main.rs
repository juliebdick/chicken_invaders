#![no_std]
#![no_main]

use lazy_static::lazy_static;
use spin::Mutex;
use pc_keyboard::DecodedKey;
use pluggable_interrupt_os::HandlerTable;
use pluggable_interrupt_os::vga_buffer::clear_screen;
use chicken_invaders::Game;
use crossbeam::atomic::AtomicCell;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .startup(startup)
        .start()
}

lazy_static! {
    static ref GAME: Mutex<Game> = Mutex::new(Game::new());
    static ref LAST_KEY: AtomicCell<Option<DecodedKey>> = AtomicCell::new(None);
}

fn tick() {
    let mut this_game = GAME.lock();
    match LAST_KEY.swap(None) {
        None => {}
        Some(key) => this_game.key(key)
    }
    this_game.tick();
    //Game::tick(&mut GAME.lock());
}

fn key(key: DecodedKey) {
    LAST_KEY.store(Some(key));
    GAME.lock().key(key);
}

fn startup() {
    clear_screen();
}