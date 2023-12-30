use ratatui::{prelude::*, widgets::*};
use ratatui::{
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;
use crate::app::WindowPhase;
use crate::kill::KillSignal;
use crate::sysinfo::ProcessStat;

pub fn render(app: &mut App, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(3),
            Constraint::Max(3),
            Constraint::Percentage(80),
        ])
        .split(frame.size());

    render_info_pane(app, frame, layout[0]);
    render_filter_pane(app, frame, layout[1]);
    render_proc_list(app, frame, layout[2]);

    if app.window_phase == WindowPhase::SignalPick {
        render_signal_panel(app, frame);
    }
}

fn render_info_pane(_app: &mut App, frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new(format!("Press `Esc`, `Ctrl-C` or `q` to exit.\n"))
        .block(
            Block::default()
                .title("PSycho KILLer")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center);

    frame.render_widget(widget, area);
}

fn render_filter_pane(app: &mut App, frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new(format!("{}", app.filter_text))
        .block(
            Block::default()
                .title("Filter")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left);

    frame.render_widget(widget, area);
}

fn render_proc_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let list_items: Vec<ListItem> = app
        .filtered_processes
        .iter()
        .map(|it: &ProcessStat| ListItem::new(format!("[{}] {}", it.pid, it.display_name)))
        .collect();
    let mut list_state = ListState::default().with_selected(Some(app.process_cursor));
    let widget = List::new(list_items)
        .block(
            Block::default()
                .title("Running Processes")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(widget, area, &mut list_state);
}

fn render_signal_panel(app: &mut App, frame: &mut Frame) {
    let list_items: Vec<ListItem> = app.known_signals
        .iter()
        .map(|it: &KillSignal| ListItem::new(it.name))
        .collect();
    let mut list_state = ListState::default().with_selected(Some(app.signal_cursor));
    let widget = List::new(list_items)
        .block(
            Block::default()
                .title("KILL command")
                .borders(Borders::ALL)
                .bg(Color::DarkGray),
        )
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let area = centered_rect(40, 6, frame.size());
    let buffer = frame.buffer_mut();
    Clear.render(area, buffer);
    frame.render_stateful_widget(widget, area, &mut list_state);
}

fn centered_rect(x: u16, y: u16, r: Rect) -> Rect {
    let x_gap = (r.width as i16 - x as i16).max(0) / 2;
    let y_gap = (r.height as i16 - y as i16).max(0) / 2;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(y_gap as u16),
            Constraint::Min(y),
            Constraint::Length(y_gap as u16),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Max(x_gap as u16),
            Constraint::Min(x),
            Constraint::Max(x_gap as u16),
        ])
        .split(popup_layout[1])[1]
}
