mod memory;
mod space_invaders_memory;
mod cpu;
mod conditions;

fn main() {
    let memory = Box::new(space_invaders_memory::SpaceInvadersMemory::new());
    let c = cpu::Cpu::new(memory);
    println!("{}", c);
}
