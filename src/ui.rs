//! UI 렌더링 모듈

use crate::app::App;
use crate::jxa::PlayerState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// UI 렌더링
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // 타이틀
            Constraint::Min(8),     // 트랙 정보
            Constraint::Length(3),  // 진행 바
            Constraint::Length(3),  // 볼륨 바
            Constraint::Length(3),  // 도움말
        ])
        .split(frame.area());

    render_title(frame, chunks[0]);
    render_track_info(frame, app, chunks[1]);
    render_progress_bar(frame, app, chunks[2]);
    render_volume_bar(frame, app, chunks[3]);
    render_help(frame, chunks[4]);
}

/// 타이틀 렌더링
fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("🎵 Apple Music Remote")
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

/// 트랙 정보 렌더링
fn render_track_info(frame: &mut Frame, app: &App, area: Rect) {
    let state_icon = match app.track.state {
        PlayerState::Playing => "▶ Playing",
        PlayerState::Paused => "⏸ Paused",
        PlayerState::Stopped => "⏹ Stopped",
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Title:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.track.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Artist: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.track.artist, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Album:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.track.album, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(state_icon, Style::default().fg(Color::Green)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Now Playing "));
    frame.render_widget(paragraph, area);
}

/// 진행 바 렌더링
fn render_progress_bar(frame: &mut Frame, app: &App, area: Rect) {
    let ratio = if app.track.duration > 0.0 {
        (app.track.player_position / app.track.duration).min(1.0)
    } else {
        0.0
    };

    let current = format_time(app.track.player_position);
    let total = format_time(app.track.duration);
    let label = format!("{} / {}", current, total);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(Color::Magenta))
        .ratio(ratio)
        .label(label);
    frame.render_widget(gauge, area);
}

/// 볼륨 바 렌더링
fn render_volume_bar(frame: &mut Frame, app: &App, area: Rect) {
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Volume "))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(app.volume as u16)
        .label(format!("{}%", app.volume));
    frame.render_widget(gauge, area);
}

/// 도움말 렌더링
fn render_help(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled(" ␣ ", Style::default().fg(Color::Yellow)),
        Span::raw("Play/Pause  "),
        Span::styled("←/→ ", Style::default().fg(Color::Yellow)),
        Span::raw("Prev/Next  "),
        Span::styled("↑/↓ ", Style::default().fg(Color::Yellow)),
        Span::raw("Volume  "),
        Span::styled("q ", Style::default().fg(Color::Red)),
        Span::raw("Quit"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, area);
}

/// 초를 mm:ss 형식으로 변환
fn format_time(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}
