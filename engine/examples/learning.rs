#[derive(Debug)]
struct Organism {
    energy: f64,
    age: u32,
}

impl Organism {
    fn new() -> Self {
        Self {
            energy: 100.0,
            age: 0,
        }
    }

    fn update(&mut self) {
        self.energy -= 12.0;
        self.age += 1;
    }

    fn is_alive(&self) -> bool {
        self.energy > 0.0
    }
}

fn main() {
    let mut organism = Organism::new();

    while organism.is_alive() {
        organism.update();

        println!("Age: {} | Energy: {:.2}", organism.age, organism.energy);
    }

    println!("Organism died!");
}
