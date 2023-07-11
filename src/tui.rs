use std::io::{self, *};

pub struct TUI {
    title: &'static str,
    todo_list: Vec<String>,
    at_line: usize,
    state: TUIState,
    controls: Vec<(&'static str, &'static str)>
}

impl TUI {
    pub fn new() -> Self {
        Self {
            title: "TODO-List",
            todo_list: Vec::new(),
            at_line: 0,
            state: TUIState::Home { info: None },
            controls: vec![("Esc", "Exit"), ("A", "Add"), ("D", "Remove"), ("M", "Move"), ("\u{f0e7a}", "Scroll"), ("S", "Save"), ("L", "Load")]
        }
    }
    pub fn draw(&mut self, size: (usize, usize)) {
        use std::fmt::Write;
        let mut out = String::new();
        
        match &mut self.state {
            TUIState::Home { .. } | TUIState::BeforeTextbox(_) => {
                let title_padding = size.0 - self.title.len() >> 1;
                let title_padding = (title_padding, size.0-self.title.len()-title_padding);

                writeln!(&mut out, "\x1b[2J\x1b[H\x1b[?25l\x1b[0;46m{}\x1b[0;1;97m{}\x1b[46m{}\x1b[0m",
                    " ".repeat(title_padding.0), self.title, " ".repeat(title_padding.1)
                ).unwrap();

                let mut list_items = 0;
                for (i, el) in self.todo_list.iter().enumerate() {
                    if self.at_line + size.1 < i+4 {
                        break
                    } else if self.at_line > i {
                        continue
                    }

                    writeln!(&mut out, "\x1b[1;93m{:4} |\x1b[0m {}", i+1, el).unwrap();
                    list_items += 1;
                }

                write!(&mut out, "\x1b[{}B", size.1 - list_items - 2).unwrap();
                match &self.state {
                    TUIState::Home { info } => {
                        if info.is_none() {
                            let mut len = 0;
                            for el in self.controls.iter() {
                                let this = format!("\x1b[0;1;48;5;239;97m {} \x1b[0;97m {} \x1b[0m", el.0, el.1);
                                len += console::strip_ansi_codes(&this).len();
                                if len-2 > size.0 {
                                    break;
                                }
                                write!(&mut out, "{}", this).unwrap();
                            }
                        } else {
                            write!(&mut out, "\x1b[0;1;96m[i]\x1b[0m {}", info.unwrap()).unwrap()
                        }
                    }
                    TUIState::BeforeTextbox(n) => {
                        write!(&mut out, "\x1b[?25h\x1b[0;4m{}\x1b[0m\x1b[0G", " ".repeat(size.0)).unwrap();
                        self.state = *(*n).to_owned()
                    },
                    _ => unreachable!()
                }
            },
            TUIState::NewItem       { current, cursor_pos } |
            TUIState::RemoveItem    { current, cursor_pos } |
            TUIState::MoveItem      { current, cursor_pos } |
            TUIState::Save          { cursor_pos, current } |
            TUIState::Load          { cursor_pos, current } => {
                write!(&mut out, "\x1b[?25h\x1b[0G\x1b[0;4m{current}{}\x1b[0m\x1b[{}G", " ".repeat(size.0.max(current.len())-current.len()), *cursor_pos+1).unwrap();
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
                TUIState::Home { info } => {
                    *info = None;
                    match key {
                        Key::Escape => {
                            print!("\x1b[?25h\x1b[2J\x1b[H");
                            std::io::stdout().flush().unwrap();
                            std::process::exit(0);
                        },
                        Key::Char(c) => {
                            match c.to_ascii_lowercase() {
                                'a' => self.state = TUIState::BeforeTextbox(Box::new(TUIState::NewItem      { cursor_pos: 0, current: String::new() })),
                                'd' => self.state = TUIState::BeforeTextbox(Box::new(TUIState::RemoveItem   { cursor_pos: 0, current: String::new() })),
                                'm' => self.state = TUIState::BeforeTextbox(Box::new(TUIState::MoveItem     { cursor_pos: 0, current: String::new() })),
                                's' => self.state = TUIState::BeforeTextbox(Box::new(TUIState::Save         { cursor_pos: 0, current: String::new() })),
                                'l' => self.state = TUIState::BeforeTextbox(Box::new(TUIState::Load         { cursor_pos: 0, current: String::new() })),
                                _   => print!("\x07")
                            }
                        },
                        Key::ArrowUp    => self.at_line = self.at_line.checked_sub(1).unwrap_or_else(|| {print!("\x07"); 0}),
                        Key::ArrowDown  => {
                            self.at_line = self.at_line.checked_add(1).unwrap_or_else(|| {print!("\x07"); self.at_line});
                            if self.at_line >= self.todo_list.len() {
                                print!("\x07");
                                self.at_line -= 1;
                            }
                        },
                        Key::ArrowLeft  => self.at_line = 0,
                        Key::ArrowRight => self.at_line = self.todo_list.len().checked_sub(1).unwrap_or(0),
                        _ => print!("\x07")
                    }
                },
                TUIState::NewItem       { cursor_pos, current } |
                TUIState::RemoveItem    { cursor_pos, current } |
                TUIState::MoveItem      { cursor_pos, current } |
                TUIState::Save          { cursor_pos, current } |
                TUIState::Load          { cursor_pos, current } => {
                    match key {
                        Key::Escape => self.state = TUIState::Home { info: None },
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
                                        self.todo_list.push(
                                            current.to_owned().replace("*", "\x1b[0;1;93;7m*\x1b[0m")
                                        )
                                    },
                                    TUIState::RemoveItem { current, .. } => {
                                        match current.parse::<usize>() {
                                            Ok(v) => {
                                                if v > self.todo_list.len() || v == 0 { print!("\x07"); } else {
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
                                                    Some(v) => if *v > self.todo_list.len() || *v == 0 { print!("\x07"); self.draw_auto(); return }
                                                }
                                            }

                                            let was = self.todo_list.remove(current[0].unwrap()-1);
                                            self.todo_list.insert(current[1].unwrap()-1, was);
                                        }
                                    },
                                    TUIState::Save { current, .. } => {
                                        std::fs::write(current.to_owned()+".ftms", self.as_bytes()).unwrap()
                                    },
                                    TUIState::Load { current, .. } => {
                                        let data = std::fs::read(current.to_owned()+".ftms");
                                        if data.is_err() {
                                            self.state = TUIState::Home { info: Some("\x1b[0;1;91mCan't find file\x1b[0m") };
                                            self.draw_auto();
                                            return
                                        }
                                        let new = Self::from_bytes(&data.unwrap());
                                        if new.is_none() {
                                            self.state = TUIState::Home { info: Some("\x1b[0;1;91mFile format error\x1b[0m") };
                                            self.draw_auto();
                                            return
                                        }
                                        *self = new.unwrap()
                                    },
                                    _ => unreachable!()
                                }
                            }
                            self.state = TUIState::Home { info: None };
                        },
                        Key::ArrowLeft  => *cursor_pos = cursor_pos.checked_sub(1).unwrap_or_else(|| {print!("\x07"); 0}),
                        Key::ArrowRight => {
                            *cursor_pos = cursor_pos.checked_add(1).unwrap_or(*cursor_pos);
                            if *cursor_pos > current.len() {
                                *cursor_pos -= 1;
                                print!("\x07")
                            }
                        },
                        Key::Home   => *cursor_pos = 0,
                        Key::End    => *cursor_pos = current.len(),
                        _ => print!("\x07")
                    }
                },
                _ => unreachable!()
            }
            self.draw_auto();
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let len = self.todo_list.len() as u32;
        let mut buf = vec![0xFE, 0xDC, 0x00, (len>>24) as u8, (len>>16) as u8, (len>>8) as u8, len as u8];
        for i in self.todo_list.iter() {
            buf.extend(i.as_bytes());
            buf.push(0);
        }
        buf
    }

    pub fn from_bytes(buf: &Vec<u8>) -> Option<Self> {
        let mut a = Self::new();
        if buf.len() < 7 {
            return None
        }
        if buf[0] != 0xFE || buf[1] != 0xDC || buf[2] != 0 {
            return None
        }
        let reads = ((buf[3] as u32) << 24) |
                    ((buf[4] as u32) << 16) |
                    ((buf[5] as u32) <<  8) |
                    buf[6] as u32;
        let mut pos = 7;
        for _ in 0..reads {
            let mut this = Vec::new();
            while buf[pos] != 0 {
                this.push(buf[pos]);
                pos += 1;
            }
            pos += 1;

            a.todo_list.push(String::from_utf8(this).unwrap());
        }

        Some(a)
    }
}

#[derive(Clone)]
pub enum TUIState {
    Home {
        info: Option<&'static str>
    },
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

    Save {
        cursor_pos: usize,
        current: String
    },

    Load {
        cursor_pos: usize,
        current: String
    },
}
