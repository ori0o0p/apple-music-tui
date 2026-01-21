//! JXA (JavaScript for Automation) 통신 모듈
//! macOS Music.app을 osascript를 통해 제어합니다.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

/// 플레이어 상태
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PlayerState {
    Playing,
    Paused,
    #[default]
    Stopped,
}

impl From<&str> for PlayerState {
    fn from(s: &str) -> Self {
        match s {
            "playing" => PlayerState::Playing,
            "paused" => PlayerState::Paused,
            _ => PlayerState::Stopped,
        }
    }
}

/// 현재 재생 중인 트랙 정보
#[derive(Debug, Clone, Default)]
pub struct TrackInfo {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    pub player_position: f64,
    pub state: PlayerState,
}

/// JXA 스크립트 실행 결과를 파싱하기 위한 구조체
#[derive(Deserialize)]
struct RawTrackInfo {
    name: String,
    artist: String,
    album: String,
    duration: f64,
    #[serde(rename = "playerPosition")]
    player_position: f64,
    state: String,
}

/// JXA 스크립트를 실행하고 결과를 반환합니다.
#[cfg(target_os = "macos")]
fn run_jxa(script: &str) -> Result<String> {
    let output = Command::new("osascript")
        .arg("-l")
        .arg("JavaScript")
        .arg("-e")
        .arg(script)
        .output()
        .context("osascript 실행 실패")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("JXA 스크립트 실패: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(not(target_os = "macos"))]
fn run_jxa(_script: &str) -> Result<String> {
    anyhow::bail!("이 앱은 macOS에서만 실행됩니다.")
}

/// Music.app이 실행 중인지 확인
pub fn is_music_running() -> bool {
    let script = r#"
        Application('System Events').processes.whose({name: 'Music'}).length > 0
    "#;
    run_jxa(script).map(|r| r == "true").unwrap_or(false)
}

/// Music.app 실행 (백그라운드)
pub fn launch_music() -> Result<()> {
    run_jxa("Application('Music').activate()")?;
    // 잠시 대기 후 백그라운드로
    std::thread::sleep(std::time::Duration::from_millis(500));
    run_jxa(r#"
        Application('System Events').processes.byName('Music').windows[0].buttons[0].click()
    "#).ok(); // 창 닫기 시도 (실패해도 무시)
    Ok(())
}

/// Music.app 초기화 - 앱이 실행되지 않았으면 실행
pub fn ensure_music_ready() -> Result<()> {
    if !is_music_running() {
        launch_music()?;
    }
    Ok(())
}

/// 라이브러리에서 재생 시작 (stopped 상태에서 호출)
pub fn start_playback() -> Result<()> {
    let script = r#"
        const music = Application('Music');
        // 라이브러리 플레이리스트에서 첫 번째 곡 재생
        try {
            const library = music.libraryPlaylists[0];
            if (library && library.tracks.length > 0) {
                library.tracks[0].play();
                "ok";
            } else {
                "no_tracks";
            }
        } catch(e) {
            "error";
        }
    "#;
    run_jxa(script)?;
    Ok(())
}

/// 재생/일시정지 토글 (stopped면 재생 시작)
pub fn play_pause() -> Result<()> {
    let script = r#"
        const music = Application('Music');
        if (music.playerState() === 'stopped') {
            // stopped 상태면 라이브러리에서 재생 시작
            try {
                const library = music.libraryPlaylists[0];
                if (library && library.tracks.length > 0) {
                    library.tracks[0].play();
                }
            } catch(e) {}
        } else {
            music.playpause();
        }
    "#;
    run_jxa(script)?;
    Ok(())
}

/// 다음 곡으로 이동
pub fn next_track() -> Result<()> {
    run_jxa("Application('Music').nextTrack()")?;
    Ok(())
}

/// 이전 곡으로 이동
pub fn previous_track() -> Result<()> {
    run_jxa("Application('Music').previousTrack()")?;
    Ok(())
}

/// 볼륨 설정 (0-100)
pub fn set_volume(level: u8) -> Result<()> {
    let level = level.min(100);
    run_jxa(&format!("Application('Music').soundVolume = {}", level))?;
    Ok(())
}

/// 현재 볼륨 가져오기
pub fn get_volume() -> Result<u8> {
    let result = run_jxa("Application('Music').soundVolume()")?;
    result.parse().context("볼륨 파싱 실패")
}

/// 현재 재생 중인 트랙 정보 가져오기
pub fn get_current_track() -> Result<TrackInfo> {
    let script = r#"
        const music = Application("Music");
        const state = music.playerState();
        if (state === "stopped") {
            JSON.stringify({
                name: "",
                artist: "",
                album: "",
                duration: 0,
                playerPosition: 0,
                state: "stopped"
            });
        } else {
            const track = music.currentTrack();
            JSON.stringify({
                name: track.name(),
                artist: track.artist(),
                album: track.album(),
                duration: track.duration(),
                playerPosition: music.playerPosition(),
                state: state
            });
        }
    "#;

    let result = run_jxa(script)?;
    let raw: RawTrackInfo = serde_json::from_str(&result).context("트랙 정보 파싱 실패")?;

    Ok(TrackInfo {
        name: raw.name,
        artist: raw.artist,
        album: raw.album,
        duration: raw.duration,
        player_position: raw.player_position,
        state: PlayerState::from(raw.state.as_str()),
    })
}

/// 현재 트랙의 아트워크를 iTunes Search API로 가져와 임시 파일에 저장합니다.
/// 아트워크가 없거나 가져올 수 없으면 None을 반환합니다.
pub fn get_artwork_path() -> Result<Option<PathBuf>> {
    // 먼저 현재 트랙 정보 가져오기
    let track = get_current_track()?;
    
    if track.name.is_empty() || track.artist.is_empty() {
        return Ok(None);
    }

    // iTunes Search API로 아트워크 URL 검색
    let search_term = format!("{} {}", track.artist, track.album);
    let encoded_term = urlencoding(&search_term);
    let api_url = format!(
        "https://itunes.apple.com/search?term={}&entity=album&limit=1",
        encoded_term
    );

    // curl로 API 호출
    let output = std::process::Command::new("curl")
        .args(["-s", &api_url])
        .output()
        .context("curl 실행 실패")?;

    if !output.status.success() {
        return Ok(None);
    }

    let response = String::from_utf8_lossy(&output.stdout);
    
    // JSON에서 artworkUrl100 추출
    if let Some(artwork_url) = extract_artwork_url(&response) {
        // 100x100을 600x600으로 변경하여 고해상도 이미지 가져오기
        let hires_url = artwork_url.replace("100x100", "600x600");
        
        // 이미지 다운로드
        let temp_path = std::env::temp_dir().join("apple_music_tui_artwork.jpg");
        let download = std::process::Command::new("curl")
            .args(["-s", "-o", temp_path.to_str().unwrap(), &hires_url])
            .output()
            .context("아트워크 다운로드 실패")?;

        if download.status.success() && temp_path.exists() {
            return Ok(Some(temp_path));
        }
    }

    Ok(None)
}

/// URL 인코딩 (간단한 구현)
fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

/// JSON 응답에서 artworkUrl100 추출
fn extract_artwork_url(json: &str) -> Option<String> {
    // "artworkUrl100":"URL" 패턴 찾기
    let marker = "\"artworkUrl100\":\"";
    if let Some(start) = json.find(marker) {
        let start_idx = start + marker.len();
        if let Some(end) = json[start_idx..].find('"') {
            return Some(json[start_idx..start_idx + end].to_string());
        }
    }
    None
}


/// 검색 결과
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub id: String, // persistentID
}

/// 라이브러리 검색
pub fn search_library(query: &str) -> Result<Vec<SearchResult>> {
    // 따옴표 escaping
    let safe_query = query.replace('"', "\\\"");
    
    let script = format!(r#"
        const music = Application("Music");
        const library = music.libraryPlaylists[0];
        
        try {{
            // 검색 수행
            const results = music.search(library, {{for: "{safe_query}"}});
            
            // 결과 매핑 (최대 20개까지만)
            let output = [];
            const limit = Math.min(results.length, 20);
            
            for (let i = 0; i < limit; i++) {{
                const track = results[i];
                output.push({{
                    name: track.name(),
                    artist: track.artist(),
                    album: track.album(),
                    id: track.persistentID()
                }});
            }}
            
            JSON.stringify(output);
        }} catch(e) {{
            JSON.stringify([]);
        }}
    "#);

    let result = run_jxa(&script)?;
    let search_results: Vec<SearchResult> = serde_json::from_str(&result).unwrap_or_default();
    
    Ok(search_results)
}


/// Apple Music 카탈로그 검색 (iTunes Search API)
pub fn search_apple_music(query: &str) -> Result<Vec<SearchResult>> {
    let encoded_query = urlencoding(query);
    let url = format!("https://itunes.apple.com/search?term={}&entity=song&limit=20&country=US", encoded_query); // country=KR? US가 안전

    let output = std::process::Command::new("curl")
        .args(["-s", &url])
        .output()
        .context("curl 실행 실패")?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let response = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap_or(serde_json::json!({}));
    
    let mut results = Vec::new();
    
    if let Some(items) = json["results"].as_array() {
        for item in items {
            let name = item["trackName"].as_str().unwrap_or("Unknown").to_string();
            let artist = item["artistName"].as_str().unwrap_or("Unknown").to_string();
            let album = item["collectionName"].as_str().unwrap_or("Unknown").to_string();
            
            // trackViewUrl 또는 ID 조합
            // 재생을 위해서는 music:// 스킴 사용
            // 예: https://music.apple.com/us/album/omg/1659513441?i=1659513445
            // -> music://music.apple.com/us/album/omg/1659513441?i=1659513445
            
            let track_view_url = item["trackViewUrl"].as_str().unwrap_or("");
            let id = if !track_view_url.is_empty() {
                track_view_url.replace("https://", "music://")
            } else {
                // URL이 없으면 ID로 조합 시도 (collectionId, trackId)
                let collection_id = item["collectionId"].as_u64().unwrap_or(0);
                let track_id = item["trackId"].as_u64().unwrap_or(0);
                format!("music://music.apple.com/song/{}?i={}", collection_id, track_id)
            };

            results.push(SearchResult {
                name,
                artist,
                album,
                id,
            });
        }
    }

    Ok(results)
}

/// 트랙 재생 (ID 또는 Apple Music URL)
pub fn play_track_by_id(id: &str) -> Result<()> {
    if id.starts_with("music://") {
        // Apple Music URL이면 open -g 명령어로 백그라운드 실행 시도
        std::process::Command::new("open")
            .arg("-g")
            .arg(id)
            .output()
            .context("open -g 실행 실패")?;
            
        // URL 로딩 대기 후 재생 시도
        // 별도 스레드에서 실행하여 UI 블로킹 방지
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            // 재생 명령 전송 (현재 컨텍스트 재생)
            let _ = run_jxa("Application('Music').play()");
        });
    } else {
        // 로컬 라이브러리 ID면 JXA로 재생
        let script = format!(r#"
            const music = Application("Music");
            try {{
                const library = music.libraryPlaylists[0];
                const tracks = library.tracks.whose({{persistentID: "{id}"}});
                
                if (tracks.length > 0) {{
                    tracks[0].play();
                }}
            }} catch(e) {{}}
        "#);
        
        run_jxa(&script)?;
    }
    Ok(())
}

