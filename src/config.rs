use std::path::PathBuf;

/// Claude Code 配置相关的路径解析
pub struct Paths {
    claude_dir: PathBuf,
    project_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Self {
        let home = dirs_home();
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            claude_dir: home.join(".claude"),
            project_dir: cwd,
        }
    }

    /// ~/.claude/settings.json
    pub fn global_settings(&self) -> PathBuf {
        self.claude_dir.join("settings.json")
    }

    /// <project>/.claude/settings.local.json（项目级覆盖）
    pub fn project_settings(&self) -> PathBuf {
        self.project_dir.join(".claude").join("settings.local.json")
    }

    /// <project>/.claude/
    pub fn project_claude_dir(&self) -> PathBuf {
        self.project_dir.join(".claude")
    }

    /// ~/.claude/profiles/
    pub fn profiles_dir(&self) -> PathBuf {
        self.claude_dir.join("profiles")
    }

    /// ~/.claude/profiles/.active
    pub fn active_file(&self) -> PathBuf {
        self.profiles_dir().join(".active")
    }

    /// ~/.claude/profiles/<name>/
    pub fn profile_dir(&self, name: &str) -> PathBuf {
        self.profiles_dir().join(name)
    }

    /// ~/.claude/profiles/<name>/settings.json
    pub fn profile_global_settings(&self, name: &str) -> PathBuf {
        self.profile_dir(name).join("settings.json")
    }

    /// ~/.claude/profiles/config.json (ccss 自身配置)
    pub fn ccss_config_file(&self) -> PathBuf {
        self.profiles_dir().join("config.json")
    }
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}
