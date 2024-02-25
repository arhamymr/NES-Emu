mod cpu;
mod sdl;

use cpu::CPU;
use sdl::SDL;

fn main() {
    let mut instance_cpu = CPU::new();
    SDL::new().run();

    instance_cpu.interpret(vec![0xa9, 0x05, 0x00]);
    println!("Hello, world!");
}
