//! 앱 상태 관리 모듈

use crate::jxa::{self, PlayerState, TrackInfo};

/// 애플리케이션 상태
pub struct App {
    /// 현재 재생 중인 트랙 정보
    pub track: TrackInfo,
    /// 현재 볼륨 (0-100)
    pub volume: u8,
    /// 앱 실행 상태
    pub running: bool,
}

impl App {
    /// 새로운 App 인스턴스 생성
    pub fn new() -> Self {
        Self {
            track: TrackInfo::default(),
            volume: 50,
            running: true,
        }
    }

    /// 재생/일시정지 토글
    pub fn toggle_play_pause(&mut self) {
        let _ = jxa::play_pause();
    }

    /// 다음 곡
    pub fn next_track(&mut self) {
        let _ = jxa::next_track();
    }

    /// 이전 곡
    pub fn previous_track(&mut self) {
        let _ = jxa::previous_track();
    }

    /// 볼륨 증가
    pub fn volume_up(&mut self) {
        self.volume = (self.volume + 5).min(100);
        let _ = jxa::set_volume(self.volume);
    }

    /// 볼륨 감소
    pub fn volume_down(&mut self) {
        self.volume = self.volume.saturating_sub(5);
        let _ = jxa::set_volume(self.volume);
    }

    /// 트랙 정보 업데이트 (폴링)
    pub fn update(&mut self) {
        if let Ok(track) = jxa::get_current_track() {
            self.track = track;
        }
        if let Ok(vol) = jxa::get_volume() {
            self.volume = vol;
        }
    }

    /// 앱 종료
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// 재생 중인지 확인
    pub fn is_playing(&self) -> bool {
        self.track.state == PlayerState::Playing
    }
}
