use ratatui::{prelude::*, widgets::*};
use ratatui::{
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;
use crate::appdata::WindowFocus;
use crate::kill::KillSignal;
use crate::numbers::{format_duration, ClampNumExt, PercentFormatterExt};
use crate::strings::apply_scroll;
use crate::sysinfo::ProcessStat;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let w = area.width as f32;
    let r_width = 44.;
    let mut l_width = (w - r_width).clamp_min(w * 0.4);
    match app.window_focus {
        WindowFocus::ProcessFilter | WindowFocus::Browse | WindowFocus::SignalPick => {
            l_width = l_width.clamp_min(w * 0.75).clamp_min(58.).clamp_max(w * 0.9);
        }
        WindowFocus::SystemStats => {}
    }
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Min(l_width as u16), Constraint::Min(r_width as u16)])
        .split(area);

    render_left(app, frame, layout[0]);
    render_right(app, frame, layout[1]);

    if app.window_focus == WindowFocus::SignalPick {
        render_signal_panel(app, frame);
    }
    if app.info_message.is_some() {
        render_info_popup(app, frame);
    }
    if app.error_message.is_some() {
        render_error_popup(app, frame);
    }
}

fn render_left(app: &mut App, frame: &mut Frame, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Max(3), Constraint::Max(3), Constraint::Min(5)])
        .split(area);

    render_info_panel(app, frame, layout[0]);
    render_filter_panel(app, frame, layout[1]);
    render_proc_list(app, frame, layout[2]);
}

fn render_info_panel(_app: &mut App, frame: &mut Frame, area: Rect) {
    let p_text =
        "`Ctrl+F` to filter processes. `R` to refresh list. `S` to sort. `Enter` to confirm. `?` for more controls.";
    let widget = Paragraph::new(p_text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(format!("PSycho KILLer {}", VERSION))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::LightRed))
        .alignment(Alignment::Center);

    frame.render_widget(widget, area);
}

fn render_filter_panel(app: &mut App, frame: &mut Frame, area: Rect) {
    let p_text = match app.window_focus {
        WindowFocus::ProcessFilter => format!("{}\u{2588}", app.filter_text), // cursor block
        _ => app.filter_text.clone(),
    };
    let panel_color = match app.window_focus {
        WindowFocus::ProcessFilter => Color::LightYellow,
        _ => Color::White,
    };
    let mut title = Block::default().title("Filter (`Ctrl+F`)");
    if app.window_focus == WindowFocus::ProcessFilter {
        title = title.title_style(Style::new().bold());
    }

    let widget = Paragraph::new(p_text)
        .block(
            title
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
                it.pid.clone(),
                apply_scroll(&it.display_name, app.horizontal_scroll),
                format_duration(it.run_time),
                it.memory_usage.to_percent1(),
                it.format_cpu_usage(&app.previous_proc_stats.processes),
            ])
        })
        .collect();
    let col_pid_length: i32 = app
        .filtered_processes
        .iter()
        .map(|it| it.pid.to_string().len())
        .max()
        .unwrap_or(0) as i32;
    let w = area.width as i32;
    let uptime_col_w = 9;
    let mem_col_w = 5;
    let cpu_col_w = 6;
    let rest_width = (w - col_pid_length - uptime_col_w - mem_col_w - cpu_col_w - 4 - 2 - 2).clamp_min(3); // -4 for padding, -2 for cursor, -2 for borders
    let widths = [
        Constraint::Length(col_pid_length as u16), // PID
        Constraint::Min(rest_width as u16),        // Name
        Constraint::Max(uptime_col_w as u16),      // Uptime
        Constraint::Max(mem_col_w as u16),         // MEM
        Constraint::Max(cpu_col_w as u16),         // CPU
    ];
    let headers = match app.ordering {
        crate::appdata::Ordering::ByUptime => ["PID", "Name", "Uptime↓", "MEM", "CPU"],
        crate::appdata::Ordering::ByMemory => ["PID", "Name", "Uptime", "MEM↑", "CPU"],
        crate::appdata::Ordering::ByCpu => ["PID", "Name", "Uptime", "MEM", "CPU↑"],
    };
    let panel_color = match app.window_focus {
        WindowFocus::Browse => Color::LightYellow,
        _ => Color::White,
    };
    let mut title = Block::default().title("Running Processes");
    if app.window_focus == WindowFocus::Browse {
        title = title.title_style(Style::new().bold());
    }

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .header(Row::new(headers).style(Style::new().bold()).bottom_margin(1))
        .block(
            title
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(panel_color)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    frame.render_stateful_widget(table, area, &mut app.proc_list_table_state);
}

fn render_right(app: &mut App, frame: &mut Frame, area: Rect) {
    let panel_color = match app.window_focus {
        WindowFocus::SystemStats => Color::LightYellow,
        _ => Color::Yellow,
    };
    let mut title = Block::default().title("System");
    if app.window_focus == WindowFocus::SystemStats {
        title = title.title_style(Style::new().bold());
    }
    let widget = Paragraph::new(app.format_sys_stats())
        .wrap(Wrap { trim: true })
        .block(
            title
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(panel_color))
        .alignment(Alignment::Left);

    frame.render_widget(widget, area);
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

fn render_error_popup(app: &mut App, frame: &mut Frame) {
    if app.error_message.is_none() {
        return;
    }
    let error_message: String = app.error_message.clone().unwrap();

    let title = Block::default()
        .title("Error")
        .title_style(Style::new().bold())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .bg(Color::Red)
        .border_type(BorderType::Rounded);
    let error_window = Paragraph::new(error_message)
        .wrap(Wrap { trim: true })
        .block(title)
        .style(Style::default().fg(Color::White));
    let ok_label = Paragraph::new("OK")
        .style(Style::default().bold().fg(Color::LightRed).bg(Color::White))
        .alignment(Alignment::Center);

    let width: u16 = (frame.size().width as f32 * 0.75f32) as u16;
    let height: u16 = frame.size().height / 2;
    let area = centered_rect(width, height, frame.size());
    let ok_label_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - 2,
        width: area.width - 2,
        height: 1,
    };
    let buffer = frame.buffer_mut();
    Clear.render(area, buffer);
    frame.render_widget(error_window, area);
    frame.render_widget(ok_label, ok_label_area);
}

fn render_info_popup(app: &mut App, frame: &mut Frame) {
    if app.info_message.is_none() {
        return;
    }
    let info_message: String = app.info_message.clone().unwrap();

    let title = Block::default()
        .title("Info")
        .title_style(Style::new().bold())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .bg(Color::Blue)
        .border_type(BorderType::Rounded);
    let error_window = Paragraph::new(info_message)
        .wrap(Wrap { trim: true })
        .block(title)
        .style(Style::default().fg(Color::White));
    let ok_label = Paragraph::new("OK")
        .style(Style::default().bold().fg(Color::LightBlue).bg(Color::White))
        .alignment(Alignment::Center);

    let width: u16 = (frame.size().width as f32 * 0.75f32) as u16;
    let height: u16 = frame.size().height / 2;
    let area = centered_rect(width, height, frame.size());
    let ok_label_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - 2,
        width: area.width - 2,
        height: 1,
    };
    let buffer = frame.buffer_mut();
    Clear.render(area, buffer);
    frame.render_widget(error_window, area);
    frame.render_widget(ok_label, ok_label_area);
}

fn centered_rect(w: u16, h: u16, r: Rect) -> Rect {
    let x_gap = (r.width as i32 - w as i32).clamp_min(0) / 2;
    let y_gap = (r.height as i32 - h as i32).clamp_min(0) / 2;
    Rect {
        x: r.x + x_gap as u16,
        y: r.y + y_gap as u16,
        width: w,
        height: h,
    }
}
