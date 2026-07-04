pub fn print_banner() {
    const BANNER: &str = include_str!("../../banner.txt");
    const VERSION: &str = "0.1.0";
    const AUTHOR: &str = "Daniel Dias";

    const ORANGE: &str = "\x1b[38;5;208m";
    const DIM: &str = "\x1b[90m";
    const RESET: &str = "\x1b[0m";

    // Print the ASCII art banner_text in orange, then version/author in dim
    print!("{}{}{}", ORANGE, BANNER, RESET);
    println!("{} v{} — {} {}", DIM, VERSION, AUTHOR, RESET);
}