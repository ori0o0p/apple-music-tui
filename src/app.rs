//! 앱 상태 관리 모듈

use crate::jxa::{self, PlayerState, TrackInfo, SearchResult};
use image::ImageReader;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

/// 애플리케이션 모드
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AppMode {
    #[default]
    Normal,
    SearchInput,
    SearchResults,
}

/// 애플리케이션 상태
pub struct App {
    /// 현재 재생 중인 트랙 정보
    pub track: TrackInfo,
    /// 현재 볼륨 (0-100)
    pub volume: u8,
    /// 앱 실행 상태
    pub running: bool,
    /// 현재 앱 모드
    pub mode: AppMode,
    
    /// 이미지 프로토콜 Picker (터미널 그래픽스 프로토콜 감지용)
    pub picker: Picker,
    /// 현재 아트워크 이미지 프로토콜 (렌더링용)
    pub artwork: Option<StatefulProtocol>,
    /// 마지막으로 로드한 트랙 이름 (변경 감지용)
    last_track_name: String,

    /// 검색 쿼리
    pub search_query: String,
    /// 검색 결과
    pub search_results: Vec<SearchResult>,
    /// 검색 결과 선택 인덱스
    pub search_result_index: usize,
}

impl App {
    /// 새로운 App 인스턴스 생성
    pub fn new() -> Self {
        // 터미널 그래픽스 프로토콜 감지 (실패 시 halfblocks 폴백)
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::from_fontsize((8, 16)));
        
        Self {
            track: TrackInfo::default(),
            volume: 50,
            running: true,
            mode: AppMode::Normal,
            picker,
            artwork: None,
            last_track_name: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_index: 0,
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
            // 트랙이 변경되었는지 확인
            let track_changed = track.name != self.last_track_name;
            self.track = track;
            
            // 트랙이 변경되었으면 아트워크 업데이트
            if track_changed {
                self.last_track_name = self.track.name.clone();
                self.update_artwork();
            }
        }
        if let Ok(vol) = jxa::get_volume() {
            self.volume = vol;
        }
    }

    /// 아트워크 업데이트
    fn update_artwork(&mut self) {
        self.artwork = None;
        
        if let Ok(Some(path)) = jxa::get_artwork_path() {
            if let Ok(reader) = ImageReader::open(&path) {
                if let Ok(dyn_img) = reader.decode() {
                    self.artwork = Some(self.picker.new_resize_protocol(dyn_img));
                }
            }
        }
    }

    /// 앱 종료
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// 재생 중인지 확인
    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        self.track.state == PlayerState::Playing
    }

    /// 검색 수행
    pub fn perform_search(&mut self) {
        if let Ok(results) = jxa::search_library(&self.search_query) {
            self.search_results = results;
            self.search_result_index = 0;
            if !self.search_results.is_empty() {
                self.mode = AppMode::SearchResults;
            } else {
                // 결과 없음 모드는 따로 없으므로 Input 모드 유지 또는 알림
            }
        }
    }

    /// 검색 결과 선택 위로 이동
    pub fn search_play_selection(&mut self) {
        if let Some(result) = self.search_results.get(self.search_result_index) {
            let _ = jxa::play_track_by_id(&result.id);
            // 재생 후 검색 모드 종료
            self.mode = AppMode::Normal;
            self.search_query.clear();
            self.search_results.clear();
        }
    }

    /// 검색 결과 선택 위로 이동
    pub fn search_select_prev(&mut self) {
        if self.search_result_index > 0 {
            self.search_result_index -= 1;
        }
    }

    /// 검색 결과 선택 아래로 이동
    pub fn search_select_next(&mut self) {
        if self.search_result_index < self.search_results.len().saturating_sub(1) {
            self.search_result_index += 1;
        }
    }
}

