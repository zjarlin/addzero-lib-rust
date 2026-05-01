#[derive(Clone, Debug)]
pub(crate) struct Download {
    pub(crate) url: String,
    pub(crate) filename: String,
    pub(crate) version: Option<u8>,
}

impl Download {
    pub(crate) fn new(url: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            filename: filename.into(),
            version: None,
        }
    }

    pub(crate) fn versioned(
        version: u8,
        url: impl Into<String>,
        filename: impl Into<String>,
    ) -> Self {
        Self {
            url: url.into(),
            filename: filename.into(),
            version: Some(version),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArchFamily {
    X64,
    Arm,
    Loong,
}

pub(crate) fn arch_family() -> ArchFamily {
    let arch = std::env::consts::ARCH.to_ascii_lowercase();
    if arch.contains("aarch64") || arch.contains("arm") {
        ArchFamily::Arm
    } else if arch.contains("loong") {
        ArchFamily::Loong
    } else {
        ArchFamily::X64
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OneClickSpec {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
}

pub(crate) const ONE_CLICK_SPECS: &[OneClickSpec] = &[
    OneClickSpec {
        name: "all",
        description: "按旧 AutoInit 任务集合逐项执行",
    },
    OneClickSpec {
        name: "env-scripts",
        description: "写入 dotfiles 环境脚本能力",
    },
    OneClickSpec {
        name: "git",
        description: "安装/检查 Git 并提示配置身份",
    },
    OneClickSpec {
        name: "node",
        description: "安装/检查 Node.js 和前端包管理器",
    },
    OneClickSpec {
        name: "jdk",
        description: "安装/检查 JDK 17",
    },
    OneClickSpec {
        name: "pnpm",
        description: "安装 pnpm 并运行 pnpm setup",
    },
    OneClickSpec {
        name: "pkg",
        description: "安装当前平台包管理器",
    },
    OneClickSpec {
        name: "pkg-manager",
        description: "安装配置中的默认软件包",
    },
    OneClickSpec {
        name: "graalvm",
        description: "下载 GraalVM 到 ~/.config/df/useful",
    },
    OneClickSpec {
        name: "final-shell",
        description: "下载并打开 FinalShell 安装器",
    },
    OneClickSpec {
        name: "idea",
        description: "配置 IntelliJ IDEA XDG 路径",
    },
    OneClickSpec {
        name: "zulu-jdk",
        description: "下载 Zulu JDK 8/17/21 并设置 JAVA_HOME",
    },
    OneClickSpec {
        name: "powershell",
        description: "执行 Windows 系统优化项",
    },
    OneClickSpec {
        name: "powershell-env",
        description: "初始化 PowerShell profile 和执行策略",
    },
    OneClickSpec {
        name: "quark",
        description: "下载夸克网盘客户端安装包",
    },
    OneClickSpec {
        name: "docker",
        description: "运行 linuxmirrors Docker 初始化脚本",
    },
    OneClickSpec {
        name: "lazyvim",
        description: "初始化 LazyVim starter",
    },
    OneClickSpec {
        name: "homebrew",
        description: "初始化 Homebrew",
    },
    OneClickSpec {
        name: "ohmyzsh",
        description: "初始化 Oh My Zsh",
    },
    OneClickSpec {
        name: "macos",
        description: "执行 macOS defaults 优化",
    },
    OneClickSpec {
        name: "enable-all-sources",
        description: "macOS 开启所有来源",
    },
    OneClickSpec {
        name: "keji",
        description: "运行 keji 面板脚本",
    },
];
