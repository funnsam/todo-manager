mod tui;

use std::io::Write;

fn main() {
    ctrlc::set_handler(move || {
        print!("\x1b[?25h\x1b[2J\x1b[H");
        std::io::stdout().flush().unwrap();
        std::process::exit(0)
    }).unwrap();

    let mut instance = tui::TUI::new();
    instance.draw_auto();
    loop {
        instance.update();
    }
}
