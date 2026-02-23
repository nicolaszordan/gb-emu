pub struct GameBoy {}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy {}
    }

    pub fn run(&mut self) {
        println!("starting gb emu...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gb_emu() {
        let mut gb = GameBoy::new();
        gb.run();
    }
}
