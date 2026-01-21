//! Apple Music TUI Remote
//! macOS Music.app을 터미널에서 제어하는 TUI 앱

mod app;
mod events;
mod jxa;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // 터미널 초기화
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Music.app이 실행되지 않았으면 자동 실행
    let _ = jxa::ensure_music_ready();

    // 앱 상태 초기화
    let mut app = App::new();
    
    // 초기 상태 로드
    app.update();

    // 메인 루프
    let result = run_app(&mut terminal, &mut app).await;

    // 터미널 복원
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_secs(1);
    let mut last_tick = std::time::Instant::now();

    while app.running {
        // UI 렌더링
        terminal.draw(|frame| ui::render(frame, app))?;

        // 이벤트 폴링 (100ms timeout)
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_millis(100));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    events::handle_key_event(app, key);
                }
            }
        }

        // 1초마다 상태 업데이트
        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}
