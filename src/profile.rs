use std::fs;
use std::path::PathBuf;

use colored::Colorize;

use crate::config::Paths;

/// 列出所有 profile，标记当前激活的
pub fn list(paths: &Paths) -> Result<(), String> {
    let profiles_dir = paths.profiles_dir();
    if !profiles_dir.exists() {
        println!("No profiles found. Run `ccss add <name>` to create one.");
        return Ok(());
    }

    let active = current_name(paths);

    let mut profiles: Vec<String> = fs::read_dir(&profiles_dir)
        .map_err(|e| format!("Failed to read profiles directory: {e}"))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                return None;
            }
            if entry.path().is_dir() {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    if profiles.is_empty() {
        println!("No profiles found. Run `ccss add <name>` to create one.");
        return Ok(());
    }

    profiles.sort();

    for name in &profiles {
        if active.as_deref() == Some(name) {
            println!("* {}", name.green().bold());
        } else {
            println!("  {name}");
        }
    }

    Ok(())
}

/// 获取当前激活的 profile 名
pub fn current_name(paths: &Paths) -> Option<String> {
    let active_file = paths.active_file();
    fs::read_to_string(&active_file).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// 显示当前 profile
pub fn current(paths: &Paths) -> Result<(), String> {
    match current_name(paths) {
        Some(name) => println!("{name}"),
        None => println!("{}", "(none)".dimmed()),
    }
    Ok(())
}

/// 切换到指定 profile
/// 默认：复制到当前项目 .claude/settings.local.json
/// global=true：复制到 ~/.claude/settings.json
pub fn use_profile(paths: &Paths, name: &str, global: bool) -> Result<(), String> {
    let profile_dir = paths.profile_dir(name);
    if !profile_dir.exists() {
        return Err(format!("Profile \"{name}\" does not exist. Run `ccss add {name}` to create it."));
    }

    let profile_settings = paths.profile_global_settings(name);
    if !profile_settings.exists() {
        return Err(format!("Profile \"{name}\" has no settings.json."));
    }

    if global {
        // 全局模式：覆盖 ~/.claude/settings.json
        // 先保存当前配置到旧 profile
        if let Some(old_name) = current_name(paths) {
            save_current_to_profile(paths, &old_name)?;
        }
        copy_file(&profile_settings, &paths.global_settings())?;
    } else {
        // 项目模式：复制到 <project>/.claude/settings.local.json
        let project_claude_dir = paths.project_claude_dir();
        if !project_claude_dir.exists() {
            fs::create_dir_all(&project_claude_dir)
                .map_err(|e| format!("Failed to create .claude directory: {e}"))?;
        }
        copy_file(&profile_settings, &paths.project_settings())?;
    }

    // 更新 .active
    fs::write(paths.active_file(), name)
        .map_err(|e| format!("Failed to update active profile: {e}"))?;

    if global {
        println!("Switched to profile \"{}\" (global)", name.green().bold());
    } else {
        println!("Switched to profile \"{}\" (project: {})", name.green().bold(), paths.project_settings().display());
    }
    Ok(())
}

/// 从当前配置创建新 profile，或创建空 profile
pub fn add(paths: &Paths, name: &str, empty: bool) -> Result<(), String> {
    let profile_dir = paths.profile_dir(name);
    if profile_dir.exists() {
        return Err(format!("Profile \"{name}\" already exists. Remove it first or use a different name."));
    }

    fs::create_dir_all(&profile_dir)
        .map_err(|e| format!("Failed to create profile directory: {e}"))?;

    if empty {
        // 创建空 JSON
        let empty_json = "{\n}\n";
        fs::write(paths.profile_global_settings(name), empty_json)
            .map_err(|e| format!("Failed to create empty settings: {e}"))?;
        println!("Created empty profile \"{}\".", name.green());
    } else {
        let global_settings = paths.global_settings();
        if !global_settings.exists() {
            // 清理已创建的目录
            let _ = fs::remove_dir(&profile_dir);
            return Err("No ~/.claude/settings.json found. Is Claude Code configured?".to_string());
        }

        copy_file(&global_settings, &paths.profile_global_settings(name))?;

        println!("Created profile \"{}\" from current settings.", name.green());
    }

    Ok(())
}

/// 删除指定 profile
pub fn remove(paths: &Paths, name: &str, yes: bool) -> Result<(), String> {
    let profile_dir = paths.profile_dir(name);
    if !profile_dir.exists() {
        return Err(format!("Profile \"{name}\" does not exist."));
    }

    if !yes {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(format!("Remove profile \"{}\"?", name))
            .default(false)
            .interact()
            .map_err(|e| format!("Failed to read confirmation: {e}"))?;
        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    fs::remove_dir_all(&profile_dir)
        .map_err(|e| format!("Failed to remove profile: {e}"))?;

    // 如果删除的是当前激活的，清除 .active
    if current_name(paths).as_deref() == Some(name) {
        let _ = fs::write(paths.active_file(), "");
    }

    println!("Removed profile \"{}\".", name.green());
    Ok(())
}

/// 显示指定 profile 的配置内容
pub fn show(paths: &Paths, name: &str) -> Result<(), String> {
    let path = paths.profile_global_settings(name);

    if !path.exists() {
        return Err(format!("Profile \"{name}\" has no settings.json."));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {e}"))?;

    // 格式化 JSON
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON: {e}"))?;
    let formatted = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to format JSON: {e}"))?;

    println!("{formatted}");
    Ok(())
}

/// 用编辑器打开 profile 配置
pub fn edit(paths: &Paths, name: &str) -> Result<(), String> {
    let path = paths.profile_global_settings(name);
    if !path.exists() {
        return Err(format!("Profile \"{name}\" has no settings.json."));
    }

    let content_before = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {e}"))?;

    edit::edit_file(&path)
        .map_err(|e| format!("Failed to open editor: {e}"))?;

    // 编辑后校验 JSON
    let content_after = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings after edit: {e}"))?;

    if let Err(e) = serde_json::from_str::<serde_json::Value>(&content_after) {
        // 恢复原始内容
        fs::write(&path, &content_before)
            .map_err(|e2| format!("Invalid JSON after edit: {e}\nAlso failed to restore backup: {e2}"))?;
        return Err(format!("Invalid JSON after edit, restored original. Error: {e}"));
    }

    println!("Profile \"{}\" updated.", name.green());
    Ok(())
}

/// 对比当前配置与指定 profile 的差异
pub fn diff(paths: &Paths, name: &str) -> Result<(), String> {
    let profile_settings = paths.profile_global_settings(name);
    if !profile_settings.exists() {
        return Err(format!("Profile \"{name}\" has no settings.json."));
    }

    let current_path = paths.global_settings();
    if !current_path.exists() {
        return Err("No current settings.json found.".to_string());
    }

    let current_content = fs::read_to_string(&current_path)
        .map_err(|e| format!("Failed to read current settings: {e}"))?;
    let profile_content = fs::read_to_string(&profile_settings)
        .map_err(|e| format!("Failed to read profile settings: {e}"))?;

    // 格式化两边 JSON 方便对比
    let current_json = format_json(&current_content)?;
    let profile_json = format_json(&profile_content)?;

    print_diff(&current_json, &profile_json, "current", name);
    Ok(())
}

// --- 内部辅助函数 ---

fn save_current_to_profile(paths: &Paths, name: &str) -> Result<(), String> {
    let profile_dir = paths.profile_dir(name);
    if !profile_dir.exists() {
        return Ok(());
    }

    let global_settings = paths.global_settings();
    if global_settings.exists() {
        copy_file(&global_settings, &paths.profile_global_settings(name))?;
    }

    Ok(())
}

fn copy_file(from: &PathBuf, to: &PathBuf) -> Result<(), String> {
    fs::copy(from, to)
        .map_err(|e| format!("Failed to copy {} to {}: {e}", from.display(), to.display()))?;
    Ok(())
}

fn format_json(content: &str) -> Result<String, String> {
    let json: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("Invalid JSON: {e}"))?;
    serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to format JSON: {e}"))
}

fn print_diff(left: &str, right: &str, _left_label: &str, _right_label: &str) {
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(left, right);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };

        let line = change.to_string_lossy();
        let formatted = match change.tag() {
            ChangeTag::Delete => line.red().to_string(),
            ChangeTag::Insert => line.green().to_string(),
            ChangeTag::Equal => line.to_string(),
        };

        print!("{sign}{formatted}");
    }
}
