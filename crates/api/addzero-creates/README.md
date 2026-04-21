# addzero-creates

把 `addzero-lib-jvm/lib/tool-jvm/network-call` 里适合公开、适合沉淀为 Rust 工具库的常见 HTTP API，统一收口成一个可复用的创建器 crate。

其中音乐领域能力已经独立抽到 `addzero-music`，这里保留兼容创建入口，避免上层调用一次性断裂。

当前已落下这些常见能力：

- Maven Central 检索与文件下载
- 网易云音乐搜索与歌词查询
- Suno 音乐生成任务
- 天眼查普通接口
- 天眼查华为云签名接口
- mail.tm 临时邮箱

## 添加依赖

如果你在这个 workspace 里直接使用：

```toml
[dependencies]
addzero-creates = { path = "../../api/addzero-creates" }
```

如果你从仓库外部引用当前本地 checkout：

```toml
[dependencies]
addzero-creates = { path = "/absolute/path/to/addzero-lib-rust/crates/api/addzero-creates" }
```

## 基础用法

```rust
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let maven = Creates::maven_central()?;
    let latest = maven.get_latest_version("com.google.guava", "guava")?;
    println!("latest guava version: {latest:?}");

    let music = Creates::music_search()?;
    let songs = music.search_songs("晴天", 5, 0)?;
    println!("songs: {}", songs.len());

    let temp_mail = Creates::temp_mail()?;
    let mailbox = temp_mail.create_mailbox_and_login("demo", 12)?;
    println!("mailbox: {}", mailbox.address);

    Ok(())
}
```

## Maven Central

```rust
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::maven_central()?;

    let artifacts = api.search_by_group_id("com.google.guava", 5)?;
    for artifact in artifacts {
        println!(
            "{}:{} -> {:?}",
            artifact.group_id,
            artifact.artifact_id,
            artifact.resolved_version()
        );
    }

    let pom = api.download_file(
        "com.google.guava",
        "guava",
        "33.2.1-jre",
        "guava-33.2.1-jre.pom",
    )?;
    println!("downloaded {} bytes", pom.len());
    Ok(())
}
```

已封装的方法：

- `search_by_group_id`
- `search_by_artifact_id`
- `search_by_coordinates`
- `search_all_versions`
- `search_by_full_coordinates`
- `search_by_class_name`
- `search_by_fully_qualified_class_name`
- `search_by_sha1`
- `search_by_tag`
- `search_by_keyword`
- `get_latest_version`
- `get_latest_version_by_group_id`
- `download_file`

## 网易云音乐搜索

```rust
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::music_search()?;

    let songs = api.search_songs("晴天", 5, 0)?;
    let artists = api.search_artists("周杰伦", 3, 0)?;
    let lyric = api.get_lyric(186016)?;

    println!("songs: {}", songs.len());
    println!("artists: {}", artists.len());
    println!("lyric: {:?}", lyric.lrc.and_then(|item| item.lyric));
    Ok(())
}
```

已封装的方法：

- `search`
- `search_songs`
- `search_artists`
- `search_albums`
- `search_playlists`
- `get_lyric`
- `get_song_detail`
- `search_by_song_and_artist`
- `search_by_lyric`
- `get_lyric_by_song_name`
- `get_lyrics_by_fragment`

## Suno

```rust
use addzero_creates::{Creates, SunoMusicRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::suno("your-suno-token")?;

    let task_id = api.generate_music(&SunoMusicRequest {
        prompt: "写一首中文电子流行歌".to_owned(),
        title: Some("电子晚风".to_owned()),
        tags: Some("electropop, chinese".to_owned()),
        ..Default::default()
    })?;

    let task = api.wait_for_completion(task_id)?;
    println!("audio url: {:?}", task.audio_url);
    Ok(())
}
```

已封装的方法：

- `generate_music`
- `generate_lyrics`
- `concat_songs`
- `fetch_task`
- `batch_fetch_tasks`
- `wait_for_completion`
- `wait_for_completion_with`
- `wait_for_batch_completion`
- `wait_for_batch_completion_with`

## 天眼查

```rust
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::tianyancha("your-authorization", "your-x-auth-token")?;

    let search = api.search_company("河南中洛佳科技有限公司", 1, 10, "0")?;
    let detail = api.get_base_info(3398690435)?;

    println!("search total: {:?}", search.company_total);
    println!("detail name: {}", detail.name);
    Ok(())
}
```

已封装的方法：

- `search_company`
- `get_base_info`

## 天眼查华为云签名版

这个版本会在 Rust 侧真实生成 `SDK-HMAC-SHA256` 签名头，而不是简单拼 URL。

```rust
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::tianyancha_huawei("your-ak", "your-sk")?;
    let result = api.search_companies("洛阳古城机械有限公司", 1, 10)?;
    println!("records: {}", result.company_list.len());
    Ok(())
}
```

已封装的方法：

- `search_companies`

## Temp Mail

```rust
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::temp_mail()?;

    let mailbox = api.create_mailbox_and_login("az", 12)?;
    println!("address: {}", mailbox.address);

    let messages = api.list_messages(&mailbox.token, 1)?;
    for message in messages {
        println!("message: {} {}", message.id, message.subject);
    }

    Ok(())
}
```

已封装的方法：

- `get_domains`
- `create_mailbox_and_login`
- `create_account`
- `create_token`
- `list_messages`
- `get_message`

## 自定义配置

```rust
use std::time::Duration;
use addzero_creates::{ApiConfig, Creates};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ApiConfig::builder("https://api.vectorengine.ai")
        .connect_timeout(Duration::from_secs(5))
        .request_timeout(Duration::from_secs(15))
        .user_agent("my-app/1.0.0")
        .build()?;

    let api = Creates::suno_with_config("your-suno-token", config)?;
    let lyrics = api.generate_lyrics("写一段中文民谣歌词")?;
    println!("lyrics: {lyrics}");
    Ok(())
}
```

## 测试覆盖

当前 crate 已用本地 fake server 覆盖这些真实请求流程：

- Maven 检索与下载
- 网易云音乐搜索、歌词、详情与二次过滤
- Suno 创建任务、查任务与轮询完成
- 天眼查 header 注入
- 天眼查华为云查询参数与签名头生成

## 范围说明

这次没有直接把 `network-call` 全量照搬到 Rust：

- 浏览器自动化、支付、私有供应商接入这类模块，不适合在当前仓库做一层“看起来统一、实际不可复用”的硬迁移
- `addzero-api-weather` 这类依赖特定站点抓取细节和 cookie 的实现，稳定性不足，不适合作为通用公开 API 默认暴露
- 后续如果确认某个接口长期稳定，再按 crate 继续补进 `addzero-creates`
