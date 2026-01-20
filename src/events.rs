//! 이벤트 핸들링 모듈

use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

/// 키보드 이벤트 처리
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match key.code {
        // 재생/일시정지
        KeyCode::Char(' ') => app.toggle_play_pause(),
        
        // 이전 곡
        KeyCode::Left | KeyCode::Char('h') => app.previous_track(),
        
        // 다음 곡
        KeyCode::Right | KeyCode::Char('l') => app.next_track(),
        
        // 볼륨 증가
        KeyCode::Up | KeyCode::Char('k') => app.volume_up(),
        
        // 볼륨 감소
        KeyCode::Down | KeyCode::Char('j') => app.volume_down(),
        
        // 종료
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        
        _ => {}
    }
}
