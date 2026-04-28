# ccss - Claude Code Settings Switcher

管理 Claude Code 配置文件的命令行工具，支持创建、切换、对比多个 settings.json 配置方案。

## 安装

```bash
cargo install --path .
```

## 核心概念

- **Profile**：一套完整的 `settings.json` 配置快照，存储在 `~/.claude/profiles/<name>/` 下
- **项目模式**（默认）：切换时将 profile 复制到当前项目的 `.claude/settings.local.json`
- **全局模式**（`-g`）：切换时将 profile 复制到 `~/.claude/settings.json`
- 每次切换前，**当前配置会自动保存回上一个 profile**，不会丢失修改

## 命令

```bash
# 列出所有 profile（* 标记当前激活）
ccss list

# 显示当前激活的 profile 名称
ccss current

# 从当前 ~/.claude/settings.json 创建 profile
ccss add <name>

# 创建空 profile
ccss add <name> --empty

# 切换到指定 profile（项目模式，写入 .claude/settings.local.json）
ccss use <name>

# 切换到指定 profile（全局模式，写入 ~/.claude/settings.json）
ccss use <name> --global

# 查看 profile 内容（格式化 JSON）
ccss show <name>

# 用编辑器编辑 profile（退出后自动校验 JSON）
ccss edit <name>

# 对比当前 ~/.claude/settings.json 与 profile 的差异
ccss diff <name>

# 删除 profile
ccss remove <name>
ccss remove <name> --yes   # 跳过确认
```

## 文件布局

```
~/.claude/
├── settings.json              # 全局配置
├── profiles/
│   ├── .active                # 记录当前激活的 profile 名称
│   ├── coding/                # profile "coding"
│   │   └── settings.json
│   └── minimal/               # profile "minimal"
│       └── settings.json

<project>/.claude/
└── settings.local.json        # 项目级覆盖（ccss use 的默认输出）
```

## 典型工作流

```bash
# 1. 首次使用：把当前配置存为 profile
ccss add coding

# 2. 创建一个最小化配置用于资源受限场景
ccss add minimal --empty
ccss edit minimal             # 编辑写入精简配置

# 3. 在项目中使用 coding profile
ccss use coding

# 4. 临时切换到 minimal
ccss use minimal

# 5. 切回 coding（当前 minimal 的配置自动保存回 minimal profile）
ccss use coding
```
