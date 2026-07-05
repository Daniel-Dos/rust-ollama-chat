//! Exibição do banner ASCII de inicialização no terminal.
//!
//! O banner é carregado de um arquivo externo (`banner.txt`) e exibido
//! com cores ANSI (laranja para a arte, cinza para metadados).

/// Exibe o banner ASCII e informações de versão/autor no terminal.
///
/// O banner é lido de [`banner.txt`](banner.txt) e exibido com cor laranja
/// (ANSI 208). A versão e o autor aparecem em cinza escuro (ANSI 90).
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
