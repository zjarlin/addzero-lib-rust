#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageChannel {
    pub slug: &'static str,
    pub title: &'static str,
    pub distribution: &'static str,
    pub description: &'static str,
    pub rule: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageAsset {
    pub slug: &'static str,
    pub channel_slug: &'static str,
    pub software_title: &'static str,
    pub package_name: &'static str,
    pub version: &'static str,
    pub platform: &'static str,
    pub format: &'static str,
    pub status: &'static str,
    pub source: &'static str,
    pub install_target: &'static str,
    pub checksum_state: &'static str,
    pub relation: &'static str,
    pub note: &'static str,
}

pub const PACKAGE_CHANNELS: &[PackageChannel] = &[
    PackageChannel {
        slug: "desktop",
        title: "桌面安装器",
        distribution: "DMG / PKG / App",
        description: "面向 macOS 工作站的图形化安装包，强调人工校验、版本回滚和入口统一。",
        rule: "软件对象和安装包分开记：这里登记的是可分发文件，不是软件概念本身。",
    },
    PackageChannel {
        slug: "cli",
        title: "CLI 二进制",
        distribution: "tar.gz / zip / brew",
        description: "面向命令行和基础设施的可执行分发物，优先记录平台架构与来源渠道。",
        rule: "CLI 包要保留平台、架构、拉取来源和 checksum 状态，避免把下载链接当成唯一事实源。",
    },
    PackageChannel {
        slug: "bundles",
        title: "知识附件包",
        distribution: "zip / tgz / assets",
        description: "随知识条目一起纳入的附件、演示资源和辅助安装素材。",
        rule: "附件包用于支撑知识条目复现，不和软件安装器混在同一个分发组里。",
    },
];

pub const PACKAGE_ASSETS: &[PackageAsset] = &[
    PackageAsset {
        slug: "cursor-macos",
        channel_slug: "desktop",
        software_title: "Cursor",
        package_name: "cursor-macos-universal.dmg",
        version: "0.52.x",
        platform: "macOS / universal",
        format: "DMG",
        status: "已纳入",
        source: "官网发布页 + MinIO / msc-aio 归档",
        install_target: "/Applications/Cursor.app",
        checksum_state: "待补 SHA256",
        relation: "关联软件对象：Cursor IDE / Agent 协作入口",
        note: "作为桌面开发主入口，后续可补自动校验和历史版本回滚。",
    },
    PackageAsset {
        slug: "docker-desktop",
        channel_slug: "desktop",
        software_title: "Docker Desktop",
        package_name: "docker-desktop-arm64.dmg",
        version: "4.40.x",
        platform: "macOS / arm64",
        format: "DMG",
        status: "待更新",
        source: "Docker 官方分发页",
        install_target: "/Applications/Docker.app",
        checksum_state: "待补 SHA256",
        relation: "关联软件对象：本地容器编排",
        note: "需要和运行时版本、镜像缓存策略一起记录，避免只看到软件名看不到安装介质。",
    },
    PackageAsset {
        slug: "obsidian",
        channel_slug: "desktop",
        software_title: "Obsidian",
        package_name: "obsidian-1.9.x-universal.dmg",
        version: "1.9.x",
        platform: "macOS / universal",
        format: "DMG",
        status: "已纳入",
        source: "官网发布页",
        install_target: "/Applications/Obsidian.app",
        checksum_state: "已校验",
        relation: "关联软件对象：笔记编辑与知识沉淀",
        note: "和知识库场景强相关，适合作为笔记编辑器安装器示例。",
    },
    PackageAsset {
        slug: "cloudflared-arm64",
        channel_slug: "cli",
        software_title: "cloudflared",
        package_name: "cloudflared-darwin-arm64.tgz",
        version: "2026.4",
        platform: "macOS / arm64",
        format: "tar.gz",
        status: "已纳入",
        source: "Cloudflare release archive",
        install_target: "~/.local/bin/cloudflared",
        checksum_state: "已校验",
        relation: "关联软件对象：隧道与公网访问",
        note: "适合作为 CLI 包的标准样例，来源与落地目标都比较明确。",
    },
    PackageAsset {
        slug: "rustup-init",
        channel_slug: "cli",
        software_title: "Rust Toolchain",
        package_name: "rustup-init-aarch64-apple-darwin",
        version: "stable-2026-04",
        platform: "macOS / arm64",
        format: "binary",
        status: "已纳入",
        source: "static.rust-lang.org",
        install_target: "~/.cargo/bin/rustup",
        checksum_state: "已校验",
        relation: "关联软件对象：Rust 构建链路",
        note: "它不是业务软件，但确实是需要管理的安装资产。",
    },
    PackageAsset {
        slug: "node-v22",
        channel_slug: "cli",
        software_title: "Node.js",
        package_name: "node-v22.x-darwin-arm64.tar.gz",
        version: "22.x",
        platform: "macOS / arm64",
        format: "tar.gz",
        status: "待更新",
        source: "nodejs.org dist",
        install_target: "~/.local/node/v22",
        checksum_state: "待补 SHA256",
        relation: "关联软件对象：前端构建与脚本运行时",
        note: "和 pnpm、前端构建脚本一起构成知识前端与后台的依赖面。",
    },
    PackageAsset {
        slug: "knowledge-graph-demo",
        channel_slug: "bundles",
        software_title: "知识图谱 Demo",
        package_name: "kg-demo-assets.zip",
        version: "2026-04-28",
        platform: "Cross-platform",
        format: "ZIP",
        status: "整理中",
        source: "演示资源打包后上传至 MinIO / msc-aio",
        install_target: "~/Downloads/kg-demo-assets",
        checksum_state: "待补 SHA256",
        relation: "关联知识条目：知识图谱概览演示素材",
        note: "用来承载截图、示例数据和演示附件，不挤占软件安装器分组。",
    },
];

pub fn package_channel(slug: &str) -> Option<&'static PackageChannel> {
    PACKAGE_CHANNELS.iter().find(|channel| channel.slug == slug)
}

pub fn package_asset(slug: &str) -> Option<&'static PackageAsset> {
    PACKAGE_ASSETS.iter().find(|asset| asset.slug == slug)
}

pub fn package_assets(channel_slug: &str) -> impl Iterator<Item = &'static PackageAsset> {
    PACKAGE_ASSETS
        .iter()
        .filter(move |asset| asset.channel_slug == channel_slug)
}

pub fn first_package_asset_slug_for_channel(channel_slug: &str) -> Option<&'static str> {
    package_assets(channel_slug).map(|asset| asset.slug).next()
}

pub fn package_asset_count(channel_slug: &str) -> usize {
    package_assets(channel_slug).count()
}

pub fn total_package_assets() -> usize {
    PACKAGE_ASSETS.len()
}

pub fn total_package_channels() -> usize {
    PACKAGE_CHANNELS.len()
}

pub fn package_pending_checks() -> usize {
    PACKAGE_ASSETS
        .iter()
        .filter(|asset| asset.checksum_state != "已校验")
        .count()
}

pub fn package_platform_count() -> usize {
    let mut seen = Vec::new();

    for asset in PACKAGE_ASSETS {
        if !seen.contains(&asset.platform) {
            seen.push(asset.platform);
        }
    }

    seen.len()
}
