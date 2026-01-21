//! UI 렌더링 모듈

use crate::app::{App, AppMode, SearchMode};
use crate::jxa::PlayerState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Clear, List, ListItem, ListState},
    Frame,
};
use ratatui_image::StatefulImage;
use unicode_width::UnicodeWidthStr;

/// UI 렌더링
pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // 타이틀
            Constraint::Min(14),    // 트랙 정보 + 아트워크 (더 크게)
            Constraint::Length(3),  // 진행 바
            Constraint::Length(3),  // 볼륨 바
            Constraint::Length(3),  // 도움말
        ])
        .split(frame.area());

    render_title(frame, chunks[0]);
    render_now_playing(frame, app, chunks[1]);
    render_progress_bar(frame, app, chunks[2]);
    render_volume_bar(frame, app, chunks[3]);
    render_help(frame, chunks[4], app.mode);

    // 검색 모드일 때 팝업 렌더링
    if app.mode == AppMode::SearchInput {
        render_search_input(frame, app);
    } else if app.mode == AppMode::SearchResults {
        render_search_results(frame, app);
    }
}

/// 타이틀 렌더링
fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("🎵 Apple Music Remote")
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

/// Now Playing 영역 렌더링 (아트워크 + 트랙 정보)
fn render_now_playing(frame: &mut Frame, app: &mut App, area: Rect) {
    // 전체 영역에 블록 그리기
    let block = Block::default().borders(Borders::ALL).title(" Now Playing ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // 아트워크 크기를 높이 기반으로 계산 (정사각형 유지)
    // 터미널 문자는 대략 가로:세로 = 1:2 비율이므로, 폭 = 높이 * 2
    let artwork_height = inner.height;
    let artwork_width = (artwork_height as u16).saturating_mul(2).min(inner.width / 2);

    // 내부를 좌우로 분할 (아트워크 : 정보)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(artwork_width), // 아트워크 영역 (반응형)
            Constraint::Min(25),               // 트랙 정보 영역
        ])
        .split(inner);

    // 아트워크 렌더링
    render_artwork(frame, app, content_chunks[0]);

    // 트랙 정보 렌더링
    render_track_info(frame, app, content_chunks[1]);
}

/// 아트워크 렌더링
fn render_artwork(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(ref mut protocol) = app.artwork {
        // 아트워크가 있으면 이미지 렌더링
        let image = StatefulImage::default();
        frame.render_stateful_widget(image, area, protocol);
    } else {
        // 아트워크가 없으면 플레이스홀더 표시
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from("    🎵"),
            Line::from(""),
            Line::from("  No Artwork"),
        ])
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        frame.render_widget(placeholder, area);
    }
}

/// 트랙 정보 렌더링
fn render_track_info(frame: &mut Frame, app: &App, area: Rect) {
    let state_icon = match app.track.state {
        PlayerState::Playing => "▶ Playing",
        PlayerState::Paused => "⏸ Paused",
        PlayerState::Stopped => "⏹ Stopped",
    };

    // Stopped 상태이고 트랙 정보가 없으면 안내 메시지 표시
    let text = if app.track.state == PlayerState::Stopped && app.track.name.is_empty() {
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
                Span::styled("Space", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to start playback", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(state_icon, Style::default().fg(Color::DarkGray)),
            ]),
        ]
    } else {
        vec![
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
        ]
    };

    let paragraph = Paragraph::new(text);
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
fn render_help(frame: &mut Frame, area: Rect, mode: AppMode) {
    let help_text = match mode {
        AppMode::Normal => vec![
            Span::styled(" ␣ ", Style::default().fg(Color::Yellow)),
            Span::raw("Play/Pause  "),
            Span::styled("←/→ ", Style::default().fg(Color::Yellow)),
            Span::raw("Prev/Next  "),
            Span::styled("↑/↓ ", Style::default().fg(Color::Yellow)),
            Span::raw("Volume  "),
            Span::styled("/ ", Style::default().fg(Color::Yellow)),
            Span::raw("Search  "),
            Span::styled("q ", Style::default().fg(Color::Red)),
            Span::raw("Quit"),
        ],
        AppMode::SearchInput => vec![
            Span::styled(" Enter ", Style::default().fg(Color::Yellow)),
            Span::raw("Search  "),
            Span::styled(" Tab ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch Source  "),
            Span::styled("Esc ", Style::default().fg(Color::Yellow)),
            Span::raw("Cancel"),
        ],
        AppMode::SearchResults => vec![
            Span::styled(" ↑/↓ ", Style::default().fg(Color::Yellow)),
            Span::raw("Move  "),
            Span::styled("Enter ", Style::default().fg(Color::Yellow)),
            Span::raw("Play  "),
            Span::styled("Esc ", Style::default().fg(Color::Yellow)),
            Span::raw("Cancel"),
        ],
    };

    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, area);
}

/// 검색 입력창 렌더링 (화면 중앙 팝업)
fn render_search_input(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, frame.area()); 
    let height = 3;
    let y_pos = area.y + (area.height - height) / 2;
    let input_area = Rect::new(area.x, y_pos, area.width, height);

    frame.render_widget(Clear, input_area); // 배경 지우기

    let title = match app.search_mode {
        SearchMode::Library => " Search Library (Tab to switch) ",
        SearchMode::AppleMusic => " Search Apple Music (Tab to switch) ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow));

    let input = Paragraph::new(app.search_query.as_str())
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(input, input_area);

    // 커서 표시 (width() 사용을 위해 unicode-width crate 필요)
    let cursor_x = input_area.x + 1 + app.search_query.width() as u16;
    let cursor_y = input_area.y + 1;
    frame.set_cursor(cursor_x, cursor_y); 
}

/// 검색 결과 리스트 렌더링 (화면 중앙 팝업)
fn render_search_results(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = app.search_results
        .iter()
        .map(|track| {
            let content = Line::from(vec![
                Span::styled(format!("{} - ", track.name), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} ", track.artist)),
                Span::styled(format!("({})", track.album), Style::default().fg(Color::DarkGray)),
            ]);
            ListItem::new(content)
        })
        .collect();

    // ListState 생성 (선택된 인덱스 설정)
    let mut state = ListState::default();
    state.select(Some(app.search_result_index));

    let title = match app.search_mode {
        SearchMode::Library => " Search Results (Library) ",
        SearchMode::AppleMusic => " Search Results (Apple Music) ",
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut state);
}

/// Helper: 화면 중앙에 특정 크기의 Rect 생성
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 초를 mm:ss 형식으로 변환
fn format_time(seconds: f64) -> String {
    let total_secs = seconds as u64;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}
