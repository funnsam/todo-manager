mod tui;

fn main() {
    let mut instance = tui::TUI::new();
    loop {
        instance.update();
    }
}
