use ratatui::{prelude::*, widgets::*};
use ratatui::{
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;
use crate::appdata::WindowPhase;
use crate::kill::KillSignal;
use crate::numbers::PercentFormatterExt;
use crate::sysinfo::ProcessStat;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    render_left(app, frame, layout[0]);
    render_right(app, frame, layout[1]);
}

fn render_left(app: &mut App, frame: &mut Frame, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(3),
            Constraint::Max(3),
            Constraint::Min(5),
        ])
        .split(area);

    render_info_panel(app, frame, layout[0]);
    render_filter_panel(app, frame, layout[1]);
    render_proc_list(app, frame, layout[2]);

    if app.window_phase == WindowPhase::SignalPick {
        render_signal_panel(app, frame);
    }
}

fn render_right(app: &mut App, frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new(app.format_sys_stats())
        .block(
            Block::default()
                .title("System")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left);

    frame.render_widget(widget, area);
}

fn render_info_panel(_app: &mut App, frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new(
        "Press `Esc`, `Ctrl-C` or `q` to exit. `/` to filter processes. `F5` to refresh. `Enter` to confirm selection"
    )
    .block(
        Block::default()
            .title("PSycho KILLer")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
    .style(Style::default().fg(Color::LightRed))
    .alignment(Alignment::Center);

    frame.render_widget(widget, area);
}

fn render_filter_panel(app: &mut App, frame: &mut Frame, area: Rect) {
    let p_text = match app.window_phase {
        WindowPhase::Browse => app.filter_text.clone(),
        WindowPhase::ProcessFilter => format!("{}\u{2588}", app.filter_text), // cursor block
        WindowPhase::SignalPick => app.filter_text.clone(),
    };
    let panel_color = match app.window_phase {
        WindowPhase::ProcessFilter => Color::LightYellow,
        _ => Color::White,
    };

    let widget = Paragraph::new(p_text)
        .block(
            Block::default()
                .title("Filter")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(panel_color))
        .alignment(Alignment::Left);

    frame.render_widget(widget, area);
}

fn render_proc_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let rows: Vec<Row> = app
        .filtered_processes
        .iter()
        .map(|it: &ProcessStat| {
            Row::new(vec![
                format!("[{}]", it.pid),
                it.display_name.clone(),
                it.cpu_usage.format_percent_0(),
                it.memory_usage.format_percent_1(),
            ])
        })
        .collect();
    let widths = [
        Constraint::Percentage(5),
        Constraint::Percentage(75),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
    ];
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec!["PID", "Name", "CPU", "MEM"])
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title("Running Processes")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    frame.render_stateful_widget(table, area, &mut app.proc_list_table_state);
}

fn render_signal_panel(app: &mut App, frame: &mut Frame) {
    let list_items: Vec<ListItem> = app
        .known_signals
        .iter()
        .map(|it: &KillSignal| ListItem::new(it.name))
        .collect();
    let mut list_state = ListState::default().with_selected(Some(app.signal_cursor));
    let widget = List::new(list_items)
        .block(
            Block::default()
                .title("Kill command")
                .borders(Borders::ALL)
                .bg(Color::DarkGray),
        )
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let height = app.known_signals.len() as u16 + 2;
    let width: u16 = app
        .known_signals
        .iter()
        .map(|it: &KillSignal| it.name.len() as u16)
        .max()
        .unwrap_or(0)
        + 8;
    let area = centered_rect(width, height, frame.size());
    let buffer = frame.buffer_mut();
    Clear.render(area, buffer);
    frame.render_stateful_widget(widget, area, &mut list_state);
}

fn centered_rect(w: u16, h: u16, r: Rect) -> Rect {
    let x_gap = (r.width as i16 - w as i16).max(0) / 2;
    let y_gap = (r.height as i16 - h as i16).max(0) / 2;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(y_gap as u16),
            Constraint::Length(h),
            Constraint::Min(y_gap as u16),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(x_gap as u16),
            Constraint::Length(w),
            Constraint::Min(x_gap as u16),
        ])
        .split(popup_layout[1])[1]
}
