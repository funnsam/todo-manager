use std::io::{self, *};

pub struct TUI {
    title: String,
    todo_list: Vec<String>,
    at_line: usize,
    state: TUIState,
    controls: Vec<(String, String)>
}

impl TUI {
    pub fn new() -> Self {
        let mut todo_list = vec!["test item".to_owned(); 100];
        todo_list.push("last".to_owned());
        Self {
            title: "TODO-List".to_owned(),
            todo_list,
            at_line: 0,
            state: TUIState::Home,
            controls: vec![("A".to_owned(), "Redraw".to_owned()); 3]
        }
    }
    pub fn draw(&self, size: (usize, usize)) {
        print!("\x1b[?25l\x1b[2J\x1b[H");

        let title_padding = size.0 - self.title.len() >> 1;
        let title_padding = (title_padding, size.0-self.title.len()-title_padding);

        println!("\x1b[0;1;97;44m{}{}{}\x1b[0m", " ".repeat(title_padding.0), self.title, " ".repeat(title_padding.1));

        for (i, el) in self.todo_list.iter().enumerate() {
            if self.at_line + size.1 - 3 < i {
                break
            } else if self.at_line > i {
                continue
            }

            println!("\x1b[1;93m{:4} |\x1b[0m {}", i+1, el);
        }

        for el in self.controls.iter() {
            print!("\x1b[0;1;100;97m {} \x1b[0;104;97m {} \x1b[0m ", el.0, el.1);
        }
        io::stdout().flush().unwrap()
    }

    pub fn update(&mut self) {
        let size = termsize::get().map(|size| (size.cols as usize, size.rows as usize)).unwrap();

        let stdin = console::Term::buffered_stdout();
        if let Ok(c) = stdin.read_char() {
            println!("{:?}", c);
            self.draw(size);
        }
    }
}

pub enum TUIState {
    Home, NewItem
}
