use std::{io::{self, *}};

pub struct TUI {
    title: String,
    todo_list: Vec<String>,
    at_line: usize,
    state: TUIState,
    controls: Vec<(&'static str, &'static str)>
}

impl TUI {
    pub fn new() -> Self {
        ctrlc::set_handler(move || {
            print!("\x1b[?25h");
            io::stdout().flush().unwrap();
            std::process::exit(0)
        }).unwrap();

        let mut todo_list = vec!["test item".to_owned(); 100];
        todo_list.push("last".to_owned());
        Self {
            title: "TODO-List".to_owned(),
            todo_list,
            at_line: 0,
            state: TUIState::Home,
            controls: vec![("^C", "Exit"), ("A", "Add"), ("D", "Remove"), ("M", "Move"), ("⇅", "Scroll")]
        }
    }
    pub fn draw(&mut self, size: (usize, usize)) {
        use std::fmt::Write;
        let mut out = String::new();
        
        match &mut self.state {
            TUIState::Home | TUIState::BeforeTextbox(_) => {
                let title_padding = size.0 - self.title.len() >> 1;
                let title_padding = (title_padding, size.0-self.title.len()-title_padding);

                writeln!(&mut out, "\x1b[2J\x1b[H\x1b[?25l\x1b[0;46m{}\x1b[0;1;97m{}\x1b[46m{}\x1b[0m",
                    " ".repeat(title_padding.0), self.title, " ".repeat(title_padding.1)
                ).unwrap();

                let mut list_items = 0;
                for (i, el) in self.todo_list.iter().enumerate() {
                    if self.at_line + size.1 - 3 < i {
                        break
                    } else if self.at_line > i {
                        continue
                    }

                    writeln!(&mut out, "\x1b[1;93m{:4} |\x1b[0m {}", i+1, el).unwrap();
                    list_items += 1;
                }

                write!(&mut out, "\x1b[{}B", size.1 - list_items - 2).unwrap();
                match &self.state {
                    TUIState::Home => {
                        let mut len = 0;
                        for el in self.controls.iter() {
                            let this = format!("\x1b[0;1;100;97m {} \x1b[0;97m {} \x1b[0m", el.0, el.1);
                            len += console::strip_ansi_codes(&this).len();
                            if len-2 > size.0 {
                                break;
                            }
                            write!(&mut out, "{}", this).unwrap();
                        }
                    }
                    TUIState::BeforeTextbox(n) => {
                        write!(&mut out, "\x1b[?25h\x1b[0;4m{}\x1b[0m", " ".repeat(size.0)).unwrap();
                        self.state = *(*n).to_owned()
                    },
                    _ => unreachable!()
                }
            },
            TUIState::NewItem       { current, .. } |
            TUIState::RemoveItem    { current, .. } |
            TUIState::MoveItem      { current, .. } => {
                write!(&mut out, "\x1b[?25h\x1b[0G\x1b[0;4m{}{}\x1b[0m", current, " ".repeat(size.0-current.len())).unwrap();
            },
        }

        print!("{out}");
        io::stdout().flush().unwrap();
    }

    pub fn draw_auto(&mut self) {
        let size = termsize::get().map(|size| (size.cols as usize, size.rows as usize)).unwrap();
        self.draw(size);
    }

    pub fn update(&mut self) {
        let stdin = console::Term::buffered_stdout();
        if let Ok(key) = stdin.read_key() {
            use console::Key;
            match &mut self.state {
                TUIState::Home => {
                    match key {
                        Key::Char(c) => {
                            match c.to_ascii_lowercase() {
                                'a' => self.state = TUIState::BeforeTextbox(Box::new(TUIState::NewItem { cursor_pos: 0, current: String::new() })),
                                'd' => self.state = TUIState::BeforeTextbox(Box::new(TUIState::RemoveItem { cursor_pos: 0, current: String::new() })),
                                'm' => self.state = TUIState::BeforeTextbox(Box::new(TUIState::MoveItem { cursor_pos: 0, current: String::new() })),
                                _   => print!("\x07")
                            }
                        },
                        Key::ArrowUp    => self.at_line = self.at_line.checked_sub(1).unwrap_or_else(|| {print!("\x07"); 0}),
                        Key::ArrowDown  => self.at_line = self.at_line.checked_add(1).unwrap_or_else(|| {print!("\x07"); self.at_line}).min(self.todo_list.len()-1),
                        Key::ArrowLeft  => self.at_line = 0,
                        Key::ArrowRight => self.at_line = self.todo_list.len()-1,
                        _ => print!("\x07")
                    }
                },
                TUIState::NewItem       { cursor_pos, current } |
                TUIState::RemoveItem    { cursor_pos, current } |
                TUIState::MoveItem      { cursor_pos, current } => {
                    match key {
                        Key::Char(c) => {current.insert(*cursor_pos, c); *cursor_pos += 1},
                        Key::Backspace => {
                            *cursor_pos = cursor_pos.checked_sub(1).unwrap_or(0);
                            if *cursor_pos < current.len() {
                                current.remove(*cursor_pos);
                            } else {
                                print!("\x07")
                            }
                        },
                        Key::Enter => {
                            if !current.trim().is_empty() {
                                match &self.state {
                                    TUIState::NewItem { current, .. } => {
                                        self.todo_list.insert(self.todo_list.len().min(self.at_line), current.to_owned())
                                    },
                                    TUIState::RemoveItem { current, .. } => {
                                        match current.parse::<usize>() {
                                            Ok(v) => {
                                                if v > self.todo_list.len() { print!("\x07"); } else {
                                                    self.todo_list.remove(v-1);
                                                }
                                            },
                                            Err(_) => print!("\x07"),
                                        }
                                    },
                                    TUIState::MoveItem { current, .. } => {
                                        let current: Vec<&str> = current.split(';').collect();
                                        if current.len() != 2 {
                                            print!("\x07")
                                        } else {
                                            let current: Vec<Option<usize>> = current.iter().map(|a| a.parse().ok()).collect();
                                            for i in current.iter() {
                                                match i {
                                                    None => { print!("\x07"); self.draw_auto(); return },
                                                    Some(v) => if *v > self.todo_list.len() { print!("\x07"); self.draw_auto(); return }
                                                }
                                            }
                                            self.todo_list.swap(current[0].unwrap()-1, current[1].unwrap()-1)
                                        }
                                    }
                                    _ => unreachable!()
                                }
                            }
                            self.state = TUIState::Home;
                        },
                        _ => print!("\x07")
                    }
                },
                _ => unreachable!()
            }
            self.draw_auto();
        }
    }
}

#[derive(Clone)]
pub enum TUIState {
    Home,
    BeforeTextbox(Box<TUIState>),
    NewItem {
        cursor_pos: usize,
        current: String
    },

    RemoveItem {
        cursor_pos: usize,
        current: String
    },

    MoveItem {
        cursor_pos: usize,
        current: String
    },
}
