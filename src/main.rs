mod app;
mod comm;
mod registers;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, FocusedPane, Mode};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let res = run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    res
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new();

    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, &app))?;

        let scanning = app.scan.is_some();
        let poll_ms = if scanning {
            0
        } else if app.mode == Mode::Main && app.editing.is_none() {
            50
        } else {
            150
        };
        if event::poll(Duration::from_millis(poll_ms))? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    handle_key(&mut app, key.code, key.modifiers);
                }
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::Down(MouseButton::Left)
                    | MouseEventKind::Drag(MouseButton::Left) => {
                        app.handle_click(m.column, m.row);
                    }
                    MouseEventKind::ScrollUp => match app.mode {
                        Mode::Setup => app.cycle_field(false),
                        Mode::Main => match app.focus {
                            FocusedPane::Motors => app.move_motor(-1),
                            FocusedPane::Registers => app.move_reg(-1),
                        },
                    },
                    MouseEventKind::ScrollDown => match app.mode {
                        Mode::Setup => app.cycle_field(true),
                        Mode::Main => match app.focus {
                            FocusedPane::Motors => app.move_motor(1),
                            FocusedPane::Registers => app.move_reg(1),
                        },
                    },
                    _ => {}
                },
                _ => {}
            }
        }
        if app.scan.is_some() {
            // Pump several pings per frame to keep the scan brisk while
            // still updating the UI as motors are discovered.
            for _ in 0..4 {
                if !app.tick_scan() {
                    break;
                }
            }
        } else if app.mode == Mode::Main {
            app.tick_live();
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    if app.editing.is_some() {
        handle_edit_key(app, code);
        return;
    }
    match app.mode {
        Mode::Setup => handle_setup_key(app, code, mods),
        Mode::Main => handle_main_key(app, code, mods),
    }
}

fn handle_setup_key(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        KeyCode::Tab | KeyCode::Down => app.cycle_field(true),
        KeyCode::BackTab | KeyCode::Up => app.cycle_field(false),
        KeyCode::Left => app.adjust_field(-1),
        KeyCode::Right => app.adjust_field(1),
        KeyCode::F(5) => {
            app.refresh_ports();
            app.status = format!("Found {} port(s).", app.ports.len());
        }
        KeyCode::Enter => app.connect_and_scan(),
        _ => {}
    }
}

fn handle_main_key(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        KeyCode::Esc => {
            if app.scan.is_some() {
                app.stop_scan();
            } else {
                app.mode = Mode::Setup;
                app.bus = None;
                app.status = "Disconnected.".into();
            }
        }
        KeyCode::Tab => {
            app.focus = match app.focus {
                FocusedPane::Motors => FocusedPane::Registers,
                FocusedPane::Registers => FocusedPane::Motors,
            };
        }
        KeyCode::Up => match app.focus {
            FocusedPane::Motors => app.move_motor(-1),
            FocusedPane::Registers => app.move_reg(-1),
        },
        KeyCode::Down => match app.focus {
            FocusedPane::Motors => app.move_motor(1),
            FocusedPane::Registers => app.move_reg(1),
        },
        KeyCode::PageUp => match app.focus {
            FocusedPane::Motors => app.move_motor(-5),
            FocusedPane::Registers => app.move_reg(-10),
        },
        KeyCode::PageDown => match app.focus {
            FocusedPane::Motors => app.move_motor(5),
            FocusedPane::Registers => app.move_reg(10),
        },
        KeyCode::Char('s') => {
            if app.scan.is_some() {
                app.stop_scan();
            } else {
                app.start_scan();
            }
        }
        KeyCode::Char('S') => app.stop_scan(),
        KeyCode::Char('r') => {
            app.read_selected_reg();
        }
        KeyCode::Char('R') => {
            app.read_all_regs_for_selected();
        }
        KeyCode::Char('e') | KeyCode::Enter => app.start_edit(),
        KeyCode::Char('t') | KeyCode::Char('T') => app.toggle_torque(),
        KeyCode::Char('h') => app.nudge_goal(-1),
        KeyCode::Char('l') => app.nudge_goal(1),
        KeyCode::Char('H') => app.nudge_goal(-50),
        KeyCode::Char('L') => app.nudge_goal(50),
        KeyCode::Char('g') | KeyCode::Char('G') => app.start_edit_goal(),
        _ => {}
    }
}

fn handle_edit_key(app: &mut App, code: KeyCode) {
    let Some(edit) = app.editing.as_mut() else {
        return;
    };
    match code {
        KeyCode::Esc => app.cancel_edit(),
        KeyCode::Enter => app.commit_edit(),
        KeyCode::Backspace => {
            edit.buffer.pop();
        }
        KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => edit.buffer.push(c),
        _ => {}
    }
}

#[allow(dead_code)]
fn _io_silence() {
    let _ = io::stdout();
}
