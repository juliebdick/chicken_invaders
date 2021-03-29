## Chicken Invaders Game

This project was built off a template from https://github.com/gjf2a/pluggable_interrupt_os.

It demonstrates a simple interactive program that uses both keyboard and timer interrupts.  
When the user types a < or > arrow key, a cannon moves in that direction.
When the user types a spacebar key, a rocket is launched from the cannon.

The program logic is largely in `lib.rs`. The code in 
`main.rs` creates a `Mutex`-protected `Game` object. The keyboard and timer handlers
invoke the appropriate methods on the unlocked `Game` object.

Prior to building this example, be sure to install the following:
* [Qemu](https://www.qemu.org/)
* Nightly Rust:
  * `rustup default nightly`
* `llvm-tools-preview`:
  * `rustup component add llvm-tools-preview`
* The [bootimage](https://github.com/rust-osdev/bootimage) tool:
  * `cargo install bootimage`