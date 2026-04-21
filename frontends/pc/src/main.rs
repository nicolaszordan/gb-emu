use gb::GameBoy;

fn main() {
    let mut gb = GameBoy::new();
    loop {
        gb.step().unwrap();
    }
}
