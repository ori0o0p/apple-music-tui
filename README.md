# 🎵 Apple Music TUI

macOS Music.app을 터미널에서 제어하는 TUI (Terminal User Interface) 리모트 컨트롤러

![macOS](https://img.shields.io/badge/macOS-only-blue)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)

## 설치

```bash
git clone https://github.com/ori0o0p/apple-music-tui.git
cd apple-music-tui
cargo build --release
```

## 사용법

1. **Music.app**을 백그라운드에서 실행
2. 터미널에서 앱 실행:

```bash
cargo run --release
```

## 키 바인딩

| 키 | 동작 |
|---|---|
| `Space` | 재생 / 일시정지 |
| `←` / `h` | 이전 곡 |
| `→` / `l` | 다음 곡 |
| `↑` / `k` | 볼륨 증가 (+5) |
| `↓` / `j` | 볼륨 감소 (-5) |
| `q` / `Esc` | 종료 |

## 요구사항

- **macOS** (Music.app 사용)
- **Rust 1.70+**
- Music.app이 백그라운드에서 실행 중이어야 함

## 기술 스택

- [ratatui](https://github.com/ratatui/ratatui) - TUI 프레임워크
- [tokio](https://tokio.rs/) - 비동기 런타임
- [crossterm](https://github.com/crossterm-rs/crossterm) - 터미널 제어
- **JXA** (JavaScript for Automation) - Music.app 통신

## 라이선스

MIT
