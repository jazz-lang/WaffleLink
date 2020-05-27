use parking_lot::Mutex;

fn main() {
    println!("{}", std::mem::size_of::<Mutex<i32>>());
}
