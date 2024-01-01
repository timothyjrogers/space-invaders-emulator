mod memory;
mod cpu;
mod conditions;

fn main() {
    let c = cpu::Cpu::new();
    println!("{}", c)
}
