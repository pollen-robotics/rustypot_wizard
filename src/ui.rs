use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Gauge, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::app::{App, FocusedPane, Hit, HitZone, Mode, SetupField};
use crate::registers::{decode_value, Access, Brand, Protocol, Reg, RegType, COMMON_BAUDRATES};

const ACCENT: Color = Color::Cyan;
const HEADER_BG: Color = Color::Rgb(30, 60, 110);
const ROW_HL: Color = Color::Rgb(20, 80, 140);

fn add_zone(app: &App, rect: Rect, hit: Hit) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    app.hits.borrow_mut().push(HitZone { rect, hit });
}

pub fn draw(f: &mut Frame, app: &App) {
    app.hits.borrow_mut().clear();
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_title(f, chunks[0]);
    match app.mode {
        Mode::Setup => draw_setup(f, chunks[1], app),
        Mode::Main => draw_main(f, chunks[1], app),
    }
    draw_status(f, chunks[2], app);

    if app.editing.is_some() {
        draw_edit_modal(f, area, app);
    }
}

fn draw_title(f: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(
            " Rustypot Wizard ",
            Style::default().fg(Color::White).bg(HEADER_BG).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "Dynamixel & Feetech configuration TUI",
            Style::default().fg(Color::Gray),
        ),
    ]);
    f.render_widget(Paragraph::new(title), area);
}

fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    let hints = match app.mode {
        Mode::Setup => " Tab/↑↓ field  ←→ change  Enter scan  F5 refresh ports  q quit ",
        Mode::Main => {
            if app.editing.is_some() {
                " Edit: type number, Enter commit, Esc cancel "
            } else {
                if app.scan.is_some() {
                    " Scanning…  s/Esc stop  q quit "
                } else {
                    " Tab focus  ↑↓ select  s rescan  r reread  e edit  Esc setup  q quit "
                }
            }
        }
    };
    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", app.status),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ),
        Span::raw(" "),
        Span::styled(hints, Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

// ---------------- Setup ----------------
fn draw_setup(f: &mut Frame, area: Rect, app: &App) {
    // Center a panel
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(14),
            Constraint::Percentage(20),
        ])
        .split(area);
    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(60),
            Constraint::Percentage(20),
        ])
        .split(v[1]);
    let panel = h[1];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .title(Span::styled(
            " Connection ",
            Style::default().fg(Color::White).bg(HEADER_BG).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(panel);
    f.render_widget(block, panel);

    let fields: [(&str, String); 5] = [
        ("Brand", brand_label(app.brand).to_string()),
        ("Port", port_label(app)),
        ("Baudrate", format!("{} bps", app.current_baud())),
        ("Protocol", protocol_label(app.protocol).to_string()),
        ("Scan IDs", format!("0..={}", app.scan_max)),
    ];

    let rows: Vec<Row> = fields
        .iter()
        .enumerate()
        .map(|(i, (label, value))| {
            let selected = i == app.setup_focus;
            let arrow = if selected { "▶" } else { " " };
            let style = if selected {
                Style::default().fg(Color::White).bg(ROW_HL).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Row::new(vec![
                Cell::from(arrow).style(Style::default().fg(ACCENT)),
                Cell::from(*label).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from(value.clone()).style(style),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(2), Constraint::Length(14), Constraint::Min(20)],
    );
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(2)])
        .split(inner);
    f.render_widget(table, layout[0]);

    // Hit zones for each setup row (placed inside layout[0]).
    for i in 0..fields.len() {
        let row_rect = Rect {
            x: layout[0].x,
            y: layout[0].y + i as u16,
            width: layout[0].width,
            height: 1,
        };
        add_zone(app, row_rect, Hit::SetupField(i));
    }

    let field_help = match app.current_field() {
        SetupField::Brand => "Pick the protocol family — Dynamixel or Feetech.",
        SetupField::Port => "Use ←→ to cycle detected serial ports. F5 to rescan.",
        SetupField::Baud => "Use ←→ to cycle baud rates.",
        SetupField::Protocol => "Dynamixel V1 (AX/MX classic, Feetech) or V2 (X-series).",
        SetupField::ScanRange => "Maximum motor ID to ping during scan (0..=N).",
    };
    let hint = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(field_help, Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [ Connect & Scan ]  ",
                Style::default()
                    .fg(Color::White)
                    .bg(HEADER_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("(Enter)", Style::default().fg(Color::Gray)),
        ]),
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(hint, layout[1]);
    // Approximate button rect: 3rd line of the hint area, columns 0..22.
    if layout[1].height >= 4 {
        let btn = Rect {
            x: layout[1].x,
            y: layout[1].y + 3,
            width: 22.min(layout[1].width),
            height: 1,
        };
        add_zone(app, btn, Hit::Connect);
    }
}

fn brand_label(b: Brand) -> &'static str {
    match b {
        Brand::Dynamixel => "Dynamixel",
        Brand::Feetech => "Feetech",
    }
}

fn protocol_label(p: Protocol) -> &'static str {
    match p {
        Protocol::V1 => "V1",
        Protocol::V2 => "V2",
    }
}

fn port_label(app: &App) -> String {
    if app.ports.is_empty() {
        "<no port detected>".into()
    } else {
        format!("{} ({}/{})", app.ports[app.port_idx], app.port_idx + 1, app.ports.len())
    }
}

// ---------------- Main ----------------
fn draw_main(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(22),
            Constraint::Percentage(56),
            Constraint::Percentage(22),
        ])
        .split(area);

    draw_motor_panel(f, cols[0], app);
    draw_register_panel(f, cols[1], app);
    draw_detail_panel(f, cols[2], app);
}

fn draw_motor_panel(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == FocusedPane::Motors;

    let block = bordered_block(" Motors ", focused);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let scanning = app.scan.is_some();
    let inner = if scanning {
        let v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(inner);
        draw_scan_gauge(f, v[0], app);
        v[1]
    } else {
        inner
    };

    let port_label = match app.current_port() {
        Some(p) => format!("{} @ {}", p, app.current_baud()),
        None => "<no port>".into(),
    };
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from(vec![
        Span::styled("▼ ", Style::default().fg(ACCENT)),
        Span::styled(port_label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ])));
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{} / {}", brand_label(app.brand), protocol_label(app.protocol)),
            Style::default().fg(Color::Gray),
        ),
    ])));
    items.push(ListItem::new(""));
    if app.motors.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  (no motors found — press s)",
            Style::default().fg(Color::DarkGray),
        ))));
    } else {
        for m in &app.motors {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  ● ", Style::default().fg(Color::Green)),
                Span::raw(m.display()),
            ])));
        }
    }

    // Hit zones — motors start at row index 3 (port/brand/spacer above).
    let motor_row_y0 = inner.y + 3;
    for (i, _) in app.motors.iter().enumerate() {
        let y = motor_row_y0 + i as u16;
        if y >= inner.y + inner.height {
            break;
        }
        add_zone(
            app,
            Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            },
            Hit::MotorIdx(i),
        );
    }

    let mut state = ListState::default();
    if !app.motors.is_empty() {
        state.select(Some(3 + app.motor_idx));
    }
    let list = List::new(items)
        .highlight_style(Style::default().bg(ROW_HL).add_modifier(Modifier::BOLD));
    f.render_stateful_widget(list, inner, &mut state);
}

fn draw_scan_gauge(f: &mut Frame, area: Rect, app: &App) {
    let Some(scan) = app.scan.as_ref() else { return };
    add_zone(app, area, Hit::StartOrStopScan);
    let label = format!(
        "scan {}/{} · {} found · [S] stop",
        scan.next_id.min(scan.max as u16 + 1),
        scan.max as u16 + 1,
        app.motors.len()
    );
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(ACCENT).bg(Color::Rgb(20, 20, 28)))
        .ratio(scan.ratio())
        .label(Span::styled(
            label,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(gauge, area);
}

fn draw_register_panel(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == FocusedPane::Registers;
    let title = match app.selected_motor().and_then(|m| m.model) {
        Some(m) => format!(" Registers — {} ", m.name),
        None => " Registers ".to_string(),
    };

    let regs = app.current_regs();
    let header_style = Style::default().fg(Color::White).bg(HEADER_BG).add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("Addr").style(header_style),
        Cell::from("Item").style(header_style),
        Cell::from("Decimal").style(header_style),
        Cell::from("Hex").style(header_style),
        Cell::from("Type").style(header_style),
        Cell::from("RW").style(header_style),
    ])
    .height(1);

    let rows: Vec<Row> = regs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let value = app.reg_values.get(&r.addr);
            let (dec, hex) = match value {
                Some(Ok(bytes)) => {
                    let v = decode_value(bytes, r.ty);
                    let hex = match r.ty.len() {
                        1 => format!("0x{:02X}", v as u8),
                        2 => format!("0x{:04X}", v as u16),
                        _ => format!("0x{:08X}", v as u32),
                    };
                    (v.to_string(), hex)
                }
                Some(Err(_)) => ("err".into(), "—".into()),
                None => ("…".into(), "…".into()),
            };

            let row_style = if app.reg_idx == i && focused {
                Style::default().fg(Color::White).bg(ROW_HL).add_modifier(Modifier::BOLD)
            } else if i % 2 == 0 {
                Style::default().fg(Color::Gray).bg(Color::Rgb(20, 20, 28))
            } else {
                Style::default().fg(Color::Gray)
            };
            Row::new(vec![
                Cell::from(format!("{}", r.addr)),
                Cell::from(r.name),
                Cell::from(dec),
                Cell::from(hex),
                Cell::from(type_label(r.ty)),
                Cell::from(access_label(r.access)),
            ])
            .style(row_style)
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(6),
        Constraint::Length(4),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(bordered_block(&title, focused));

    let mut state = TableState::default();
    state.select(Some(app.reg_idx));
    f.render_stateful_widget(table, area, &mut state);

    // Hit zones for register rows. Layout: top border (1) + header (1) = 2 rows
    // before the body, then bottom border (1) at the end.
    let body_y0 = area.y + 2;
    let body_h = area.height.saturating_sub(3);
    let visible = body_h as usize;
    let start = app.reg_idx.saturating_sub(visible.saturating_sub(1));
    for (offset, i) in (start..start + visible).enumerate() {
        if i >= regs.len() {
            break;
        }
        let y = body_y0 + offset as u16;
        if y >= area.y + area.height - 1 {
            break;
        }
        add_zone(
            app,
            Rect {
                x: area.x + 1,
                y,
                width: area.width.saturating_sub(2),
                height: 1,
            },
            Hit::RegIdx(i),
        );
    }
}

fn draw_detail_panel(f: &mut Frame, area: Rect, app: &App) {
    let title = match app.selected_motor().and_then(|m| m.model) {
        Some(m) => format!(" Control — {} ", m.name),
        None => " Control ".to_string(),
    };
    let block = bordered_block(&title, false);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(motor) = app.selected_motor() else {
        let p = Paragraph::new(Line::from(Span::styled(
            "No motor selected.",
            Style::default().fg(Color::DarkGray),
        )));
        f.render_widget(p, inner);
        return;
    };

    let ctl = app.motor_control();
    let deg_per_count = motor.model.map(|m| m.deg_per_count).unwrap_or(360.0 / 4096.0);

    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // motor header
            Constraint::Length(8),  // live readings
            Constraint::Length(2),  // torque toggle
            Constraint::Length(5),  // goal slider
            Constraint::Min(2),     // hints
        ])
        .split(inner);

    // Motor header
    let header = Line::from(vec![
        Span::styled(
            format!(" ID {} ", motor.id),
            Style::default().fg(Color::White).bg(HEADER_BG).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            match motor.model {
                Some(m) => m.name.to_string(),
                None => format!("Unknown ({:?})", motor.model_number),
            },
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(header), v[0]);

    // Live readings
    let mut lines: Vec<Line> = Vec::new();
    let row = |label: &str, value: String, unit: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("{:<11}", label), Style::default().fg(Color::DarkGray)),
            Span::styled(value, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {}", unit), Style::default().fg(Color::Gray)),
        ])
    };
    let read = |reg: Option<Reg>| -> Option<i64> { reg.and_then(|r| app.cached(r)) };

    if let Some(p) = read(ctl.present_position) {
        let deg = (p as f64) * deg_per_count;
        lines.push(row(
            "Position",
            format!("{:>7.2}°", deg),
            &format!("(raw {})", p),
        ));
    } else {
        lines.push(row("Position", "—".into(), ""));
    }
    if let Some(v) = read(ctl.present_velocity) {
        lines.push(row("Velocity", format!("{}", v), "raw"));
    }
    if let Some(c) = read(ctl.present_current) {
        lines.push(row("Current", format!("{}", c), ""));
    }
    if let Some(l) = read(ctl.present_load) {
        lines.push(row("Load", format!("{}", l), ""));
    }
    if let Some(volt) = read(ctl.present_voltage) {
        // Most servos report voltage in 0.1 V units.
        let scale = match ctl.present_voltage.map(|r| r.ty) {
            Some(RegType::U16) => 0.1,
            Some(RegType::U8) => 0.1,
            _ => 1.0,
        };
        lines.push(row(
            "Voltage",
            format!("{:>5.2}", volt as f64 * scale),
            "V",
        ));
    }
    if let Some(t) = read(ctl.present_temperature) {
        lines.push(row("Temp", format!("{}", t), "°C"));
    }
    if let Some(m) = read(ctl.moving) {
        lines.push(row(
            "Moving",
            if m != 0 { "yes".into() } else { "no".into() },
            "",
        ));
    }
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        v[1],
    );

    // Torque toggle
    let torque_state = read(ctl.torque_enable);
    let torque_line = match torque_state {
        Some(t) => {
            let on = t != 0;
            Line::from(vec![
                Span::styled("Torque  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if on { " ON  " } else { " OFF " },
                    if on {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Rgb(120, 30, 30))
                            .add_modifier(Modifier::BOLD)
                    },
                ),
                Span::styled("   [t] toggle", Style::default().fg(Color::Gray)),
            ])
        }
        None => Line::from(Span::styled(
            "Torque  (n/a)",
            Style::default().fg(Color::DarkGray),
        )),
    };
    f.render_widget(Paragraph::new(torque_line), v[2]);
    add_zone(app, v[2], Hit::ToggleTorque);

    // Goal slider
    if let Some(goal_reg) = ctl.goal_position {
        let bounds = app.position_bounds().unwrap_or((0, 4095));
        let goal = read(Some(goal_reg)).unwrap_or(bounds.0);
        let pos = read(ctl.present_position).unwrap_or(goal);
        let span = (bounds.1 - bounds.0).max(1) as f64;
        let goal_ratio = ((goal - bounds.0) as f64 / span).clamp(0.0, 1.0);
        let pos_deg = (pos as f64) * deg_per_count;
        let goal_deg = (goal as f64) * deg_per_count;

        let header = Line::from(vec![
            Span::styled("Goal Position  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>7.2}°", goal_deg),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  (raw {} / {}…{})", goal, bounds.0, bounds.1),
                Style::default().fg(Color::Gray),
            ),
        ]);
        let cur_line = Line::from(vec![
            Span::styled("Present        ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>7.2}°", pos_deg),
                Style::default().fg(Color::White),
            ),
            Span::styled(format!("  (raw {})", pos), Style::default().fg(Color::Gray)),
        ]);
        let inner_v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
            .split(v[3]);
        f.render_widget(Paragraph::new(header), inner_v[0]);
        f.render_widget(Paragraph::new(cur_line), inner_v[1]);
        let gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(ACCENT)
                    .bg(Color::Rgb(20, 20, 28))
                    .add_modifier(Modifier::BOLD),
            )
            .ratio(goal_ratio)
            .label(Span::styled(
                format!("{}", goal),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
        f.render_widget(gauge, inner_v[2]);
        add_zone(app, inner_v[0], Hit::EditGoal);
        add_zone(app, inner_v[2], Hit::GoalSlider);
    } else {
        f.render_widget(
            Paragraph::new(Span::styled(
                "Goal position: n/a",
                Style::default().fg(Color::DarkGray),
            )),
            v[3],
        );
    }

    // Hints
    let hints = Paragraph::new(vec![
        Line::from(Span::styled(
            "[t] torque   [h/l] ±1   [H/L] ±50   [g] set goal",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "[r] re-read register   [e] edit register",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(hints, v[4]);
}

fn draw_edit_modal(f: &mut Frame, area: Rect, app: &App) {
    let Some(edit) = app.editing.as_ref() else { return };
    let reg = app
        .current_regs()
        .iter()
        .find(|r| r.addr == edit.addr)
        .copied();

    let w = 50.min(area.width.saturating_sub(4));
    let h = 7.min(area.height.saturating_sub(4));
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let popup = Rect { x, y, width: w, height: h };
    f.render_widget(Clear, popup);

    let title = match reg {
        Some(r) => format!(" Edit — {} (addr {}) ", r.name, r.addr),
        None => " Edit ".into(),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .title(Span::styled(
            title,
            Style::default().fg(Color::White).bg(HEADER_BG).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let p = Paragraph::new(vec![
        Line::from(Span::styled(
            "Type a decimal value:",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(edit.buffer.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("_", Style::default().fg(ACCENT)),
        ]),
    ])
    .alignment(Alignment::Left);
    f.render_widget(p, inner);
}

fn bordered_block(title: &str, focused: bool) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if focused { ACCENT } else { Color::DarkGray }))
        .title(Span::styled(
            title.to_string(),
            Style::default().fg(Color::White).bg(HEADER_BG).add_modifier(Modifier::BOLD),
        ))
}

fn type_label(t: RegType) -> &'static str {
    match t {
        RegType::U8 => "u8",
        RegType::U16 => "u16",
        RegType::U32 => "u32",
        RegType::I8 => "i8",
        RegType::I16 => "i16",
        RegType::I32 => "i32",
        RegType::Bool => "bool",
    }
}
fn access_label(a: Access) -> &'static str {
    match a {
        Access::R => "R",
        Access::Rw => "RW",
    }
}

#[allow(dead_code)]
fn _silence(_: &[u32]) {
    let _ = COMMON_BAUDRATES;
}
