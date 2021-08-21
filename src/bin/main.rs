//! A static site generator for photo galleries.

fn main() {
    if let Err(e) = gallery::run_on_args(std::env::args_os()) {
        println!("Error: {:?}", e);
    }
}
