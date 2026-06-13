# CLAUDE.md

<!-- autoskills:start -->

由 `autoskills` 生成的摘要。请查看 `.claude/skills` 中的完整文件。

## 可访问性 (a11y)

根据 WCAG 2.2 指南审核并改进 Web 可访问性。当用户要求“改进可访问性”、“a11y 审计”、“WCAG 合规”、“屏幕阅读器支持”、“键盘导航”或“提高可访问性”时使用此技能。

- `.claude/skills/accessibility/SKILL.md`
- `.claude/skills/accessibility/references/A11Y-PATTERNS.md`：针对常见可访问性需求的实用、可复制粘贴模式。每个模式都是自包含的，并链接到主 [SKILL.md](../SKILL.md)。
- `.claude/skills/accessibility/references/WCAG.md`

## 设计思维

创建具有高设计质量的标志性生产级前端界面。用户请求构建 Web 组件、页面、工件、海报或应用程序时使用此技能（示例包括网站、着陆页、仪表盘、React 组件、HTML/CSS 布局或样式设计等）。

- `.claude/skills/frontend-design/SKILL.md`

## Rust 最佳实践

>

- `.claude/skills/rust-best-practices/SKILL.md`
- `.claude/skills/rust-best-practices/references/chapter_01.md`：Rust 的所有权系统鼓励使用 **借用**（`&T`），而不是 **克隆**（`T.clone()`）。
- `.claude/skills/rust-best-practices/references/chapter_02.md`：请确保已随 Rust 编译器安装 `cargo clippy`，在终端中运行 `cargo clippy -V`，对于 Rust 项目你应该看到类似 `clippy 0.1.86 (05f9846f89 2025-03-31)` 的版本信息。如果终端未显示 clippy 版本，请运行以下命令 `rustup update && r...`
- `.claude/skills/rust-best-practices/references/chapter_03.md`：性能工作的 **黄金法则**：
- `.claude/skills/rust-best-practices/references/chapter_04.md`：Rust 强制执行严格的错误处理方法，但 _如何_ 处理它们决定了你的代码是否感觉符合人体工程学、一致且安全，而不是晦涩难懂。这一章深入探讨了跨库和二进制文件建模与管理易失败操作的最佳实践。
- `.claude/skills/rust-best-practices/references/chapter_05.md`：在 Rust 中，和许多其他语言一样，测试通常展示函数的正确用法。如果测试清晰且有针对性，它往往比阅读函数体更有帮助，结合其他测试时，它们可作为活文档。
- `.claude/skills/rust-best-practices/references/chapter_06.md`：Rust 允许你通过两种方式处理多态代码：_ **泛型 / 静态分发**：编译时按每个使用单态化。_ **特征对象 / 动态分发**：运行时 vtable，单一实现。
- `.claude/skills/rust-best-practices/references/chapter_07.md`：在编译时对状态建模，通过使非法状态不可表示来防止错误。它利用 Rust 的泛型和类型系统创建只有在满足特定条件时才能达到的子类型，从而使某些操作在编译时变为非法。
- `.claude/skills/rust-best-practices/references/chapter_08.md`：当代码无法清晰表达时，使用 `//` 注释（双斜杠），例如：_ **安全保证**，其中一些可以通过代码条件更好地表达。_ **解决方法** 或 **优化**。\* 旧版或 **平台特定** 的行为。它们中的某些可以通过 `#...` 表达。
- `.claude/skills/rust-best-practices/references/chapter_09.md`：许多高级语言隐藏了内存管理，通常通过 **按值传递**（复制数据）或 **按引用传递**（引用共享数据）来处理，而无需关心分配、堆、栈、所有权和生命周期，所有这些都由垃圾回收器或虚拟机委派处理。这里是一个比较 o...

## SEO 优化

优化搜索引擎可见性和排名。当用户要求“改进 SEO”、“优化搜索”、“修复元标签”、“添加结构化数据”、“网站地图优化”或“搜索引擎优化”时使用此技能。

- `.claude/skills/seo/SKILL.md`

<!-- autoskills:end -->
