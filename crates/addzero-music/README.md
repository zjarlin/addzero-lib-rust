# addzero-music

`addzero-music` is the standalone music-domain crate extracted from the old mixed `addzero-creates` surface.

It currently provides:

- Netease music search, detail, and lyric lookup
- Suno music generation, task fetch, and polling helpers

## Add Dependency

```toml
[dependencies]
addzero-music = { path = "/absolute/path/to/addzero-lib-rust/crates/addzero-music" }
```

## Basic Usage

```rust
use addzero_music::{Music, SunoMusicRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let netease = Music::netease()?;
    let songs = netease.search_songs("晴天", 5, 0)?;
    println!("songs: {}", songs.len());

    let suno = Music::suno("your-suno-token")?;
    let task_id = suno.generate_music(&SunoMusicRequest {
        prompt: "Write a bright city-pop song".to_owned(),
        ..Default::default()
    })?;
    println!("task id: {task_id}");
    Ok(())
}
```

## APIs

- `Music::netease`
- `Music::netease_with_config`
- `Music::suno`
- `Music::suno_with_config`
- `create_netease_api`
- `create_suno_api`
