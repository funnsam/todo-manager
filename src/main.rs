mod tui;

fn main() {
    let mut instance = tui::TUI::new();
    instance.draw_auto();
    loop {
        instance.update();
    }
}
