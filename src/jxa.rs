//! JXA (JavaScript for Automation) 통신 모듈
//! macOS Music.app을 osascript를 통해 제어합니다.

use anyhow::{Context, Result};
use serde::Deserialize;
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
