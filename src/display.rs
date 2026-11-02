use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use crossterm::cursor::{Hide, MoveUp, RestorePosition, SavePosition, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{self, Clear, ClearType, DisableLineWrap, EnableLineWrap};
use crossterm::{execute, queue};

pub struct Display;

impl Display {
    /// Shows the menu below the prompt and returns the chosen index.
    /// Drawn on /dev/tty because stdout is captured by the shell.
    pub fn select(paths: &[&Path]) -> io::Result<Option<usize>> {
        let mut tty = OpenOptions::new().write(true).open("/dev/tty")?;
        terminal::enable_raw_mode()?;
        let result = run_menu(&mut tty, paths);
        terminal::disable_raw_mode()?;
        result
    }

    pub fn print(path: &Path) {
        println!("{}", path.display());
    }
}

fn run_menu(tty: &mut File, paths: &[&Path]) -> io::Result<Option<usize>> {
    let height = paths.len() as u16;
    let mut menu = Menu::new(paths.len());

    // Make room below the prompt (scrolls if it is at the bottom of the screen),
    // then remember where the user was typing. In raw mode "\n" keeps the column.
    execute!(tty, Hide, DisableLineWrap, Print("\n".repeat(paths.len())), MoveUp(height), SavePosition)?;
    draw(tty, paths, menu.selected)?;
    let choice = loop {
        let Event::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match menu.handle(key) {
            Action::Continue => draw(tty, paths, menu.selected)?,
            Action::Select(i) => break Some(i),
            Action::Cancel => break None,
        }
    };

    // Erase the menu and put the cursor back where the user was typing.
    execute!(tty, RestorePosition, Clear(ClearType::FromCursorDown), EnableLineWrap, Show)?;
    Ok(choice)
}

/// One line per path below the prompt, the selected one highlighted.
fn draw(tty: &mut File, paths: &[&Path], selected: usize) -> io::Result<()> {
    queue!(tty, RestorePosition)?;
    for (i, path) in paths.iter().enumerate() {
        // "\r\n" rather than MoveToNextLine: it scrolls when the prompt is at the bottom.
        queue!(tty, Print("\r\n"), Clear(ClearType::CurrentLine))?;
        if i == selected {
            queue!(tty, PrintStyledContent(label(i, path).reverse()))?;
        } else {
            queue!(tty, Print(label(i, path)))?;
        }
    }
    tty.flush()
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Continue,
    Select(usize),
    Cancel,
}

/// Navigation state of the menu, independent of the terminal.
pub struct Menu {
    pub selected: usize,
    len: usize,
}

impl Menu {
    pub fn new(len: usize) -> Menu {
        Menu { selected: 0, len }
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Cancel;
        }
        match key.code {
            KeyCode::Up => self.selected = (self.selected + self.len - 1) % self.len,
            KeyCode::Down | KeyCode::Tab => self.selected = (self.selected + 1) % self.len,
            KeyCode::Enter => return Action::Select(self.selected),
            KeyCode::Esc | KeyCode::Char('q') => return Action::Cancel,
            KeyCode::Char(c) => {
                if let Some(i) = digit_index(c).filter(|&i| i < self.len) {
                    return Action::Select(i);
                }
            }
            _ => {}
        }
        Action::Continue
    }
}

/// '1'..'9' -> 0..8, '0' -> 9 (the 10th line).
fn digit_index(c: char) -> Option<usize> {
    match c.to_digit(10)? {
        0 => Some(9),
        d => Some(d as usize - 1),
    }
}

/// "  3  /home/hane/projets"
pub fn label(index: usize, path: &Path) -> String {
    format!("{:>3}  {}", index + 1, path.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn arrows_and_tab_move_selection_with_wrap() {
        let mut menu = Menu::new(3);
        assert_eq!(menu.handle(key(KeyCode::Up)), Action::Continue);
        assert_eq!(menu.selected, 2);
        menu.handle(key(KeyCode::Down));
        assert_eq!(menu.selected, 0);
        menu.handle(key(KeyCode::Tab));
        assert_eq!(menu.selected, 1);
    }

    #[test]
    fn enter_selects_current() {
        let mut menu = Menu::new(3);
        menu.handle(key(KeyCode::Down));
        assert_eq!(menu.handle(key(KeyCode::Enter)), Action::Select(1));
    }

    #[test]
    fn digits_select_directly_and_zero_is_tenth() {
        let mut menu = Menu::new(10);
        assert_eq!(menu.handle(key(KeyCode::Char('2'))), Action::Select(1));
        assert_eq!(menu.handle(key(KeyCode::Char('0'))), Action::Select(9));
    }

    #[test]
    fn digit_out_of_range_is_ignored() {
        let mut menu = Menu::new(2);
        assert_eq!(menu.handle(key(KeyCode::Char('3'))), Action::Continue);
    }

    #[test]
    fn escape_q_and_ctrl_c_cancel() {
        let mut menu = Menu::new(2);
        assert_eq!(menu.handle(key(KeyCode::Esc)), Action::Cancel);
        assert_eq!(menu.handle(key(KeyCode::Char('q'))), Action::Cancel);
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(menu.handle(ctrl_c), Action::Cancel);
    }

    #[test]
    fn other_keys_are_ignored() {
        let mut menu = Menu::new(2);
        assert_eq!(menu.handle(key(KeyCode::Char('x'))), Action::Continue);
        assert_eq!(menu.selected, 0);
    }

    #[test]
    fn label_aligns_numbers() {
        assert_eq!(label(0, Path::new("/a")), "  1  /a");
        assert_eq!(label(9, Path::new("/b")), " 10  /b");
    }
}
