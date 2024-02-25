mod cpu;
use cpu::CPU;

fn main() {
    let mut instance_cpu = CPU::new();

    instance_cpu.interpret(vec![0xa9]);
    println!("Hello, world!");
}
