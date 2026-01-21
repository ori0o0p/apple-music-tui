//! 이벤트 핸들링 모듈

use crate::app::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent};

/// 키보드 이벤트 처리
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match app.mode {
        AppMode::Normal => handle_normal_mode(app, key),
        AppMode::SearchInput => handle_search_input_mode(app, key),
        AppMode::SearchResults => handle_search_results_mode(app, key),
    }
}

/// 기본 모드 키 핸들링
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
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
        
        // 검색 모드 진입
        KeyCode::Char('/') => {
            app.mode = AppMode::SearchInput;
            app.search_query.clear();
        }

        // 종료
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        
        _ => {}
    }
}

/// 검색 입력 모드 키 핸들링
fn handle_search_input_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // 검색 수행
        KeyCode::Enter => app.perform_search(),
        
        // 취소
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
            app.search_query.clear();
        }
        
        // 백스페이스
        KeyCode::Backspace => {
            app.search_query.pop();
        }
        
        // 문자 입력
        KeyCode::Char(c) => {
            app.search_query.push(c);
        }
        
        _ => {}
    }
}

/// 검색 결과 선택 모드 키 핸들링
fn handle_search_results_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // 선택 및 재생
        KeyCode::Enter => app.search_play_selection(),
        
        // 취소
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
            app.search_query.clear();
            app.search_results.clear();
        }
        
        // 위로 이동
        KeyCode::Up | KeyCode::Char('k') => app.search_select_prev(),
        
        // 아래로 이동
        KeyCode::Down | KeyCode::Char('j') => app.search_select_next(),
        
        _ => {}
    }
}
