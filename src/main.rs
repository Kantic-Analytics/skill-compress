fn main() {
    if let Err(error) = skill_compress::run() {
        eprintln!("error: {}", error.message);
        std::process::exit(error.exit_code);
    }
}
