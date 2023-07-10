mod tui;

fn main() {
    ctrlc::set_handler(move || {}).unwrap();

    let mut instance = tui::TUI::new();
    instance.draw_auto();
    loop {
        instance.update();
    }
}
