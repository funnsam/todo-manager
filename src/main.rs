mod tui;

fn main() {
    ctrlc::set_handler(move || {
        use std::io::Write;
        print!("\x07");
        std::io::stdout().flush().unwrap();
    }).unwrap();

    let mut instance = tui::TUI::new();
    instance.draw_auto();
    loop {
        instance.update();
    }
}
