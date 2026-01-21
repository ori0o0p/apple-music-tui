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

/// 재생/일시정지 토글
pub fn play_pause() -> Result<()> {
    run_jxa("Application('Music').playpause()")?;
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

