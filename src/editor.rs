use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::config::Paths;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CcssConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
}

/// 读取 ccss 配置，文件不存在或解析失败返回默认值
pub fn load_config(paths: &Paths) -> CcssConfig {
    let path = paths.ccss_config_file();
    if !path.exists() {
        return CcssConfig::default();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            eprintln!(
                "{} Failed to parse config file: {e}, using defaults.",
                "Warning:".yellow()
            );
            CcssConfig::default()
        }),
        Err(_) => CcssConfig::default(),
    }
}

/// 保存 ccss 配置到文件
pub fn save_config(paths: &Paths, config: &CcssConfig) -> Result<(), String> {
    let dir = paths.profiles_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create profiles directory: {e}"))?;
    }
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    fs::write(paths.ccss_config_file(), content)
        .map_err(|e| format!("Failed to write config: {e}"))?;
    Ok(())
}

/// 检测可用的编辑器，优先级: ccss config > EDITOR env > VISUAL env > 自动检测
pub fn detect_editor(config: &CcssConfig) -> Option<String> {
    // 1. ccss 配置
    if let Some(ref cmd) = config.editor {
        let binary = cmd.split_whitespace().next().unwrap_or(cmd);
        if which::which(binary).is_ok() {
            return Some(cmd.clone());
        }
    }

    // 2. EDITOR 环境变量
    if let Ok(editor) = std::env::var("EDITOR")
        && !editor.is_empty()
    {
        return Some(editor);
    }

    // 3. VISUAL 环境变量
    if let Ok(visual) = std::env::var("VISUAL")
        && !visual.is_empty()
    {
        return Some(visual);
    }

    // 4. 自动检测常见编辑器
    for name in &["vim", "nano", "code"] {
        if which::which(name).is_ok() {
            return Some(name.to_string());
        }
    }

    None
}

/// 启动编辑器打开指定文件
pub fn launch_editor(path: &PathBuf, config: &CcssConfig) -> Result<(), String> {
    let editor_cmd = detect_editor(config).ok_or_else(|| {
        "No editor found. Set one with:\n  ccss config editor <command>\nOr set the EDITOR environment variable.".to_string()
    })?;

    let parts: Vec<&str> = editor_cmd.split_whitespace().collect();
    let binary = parts[0];
    let args = &parts[1..];

    let binary_path = which::which(binary)
        .map_err(|e| format!("Editor '{}' not found in PATH: {e}", binary))?;

    let status = Command::new(&binary_path)
        .args(args)
        .arg(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to launch editor '{}': {e}", editor_cmd))?;

    if !status.success() {
        return Err(format!(
            "Editor '{}' exited with error.",
            editor_cmd
        ));
    }

    Ok(())
}

/// 处理 `ccss config show`
pub fn config_show(paths: &Paths) -> Result<(), String> {
    let config = load_config(paths);
    println!(
        "Editor: {}",
        config.editor.as_deref().unwrap_or("(not set)")
    );
    println!();
    match detect_editor(&config) {
        Some(editor) => println!("Resolved editor: {editor}"),
        None => println!("{}", "No editor detected.".yellow()),
    }
    Ok(())
}

/// 处理 `ccss config editor [command]`
pub fn config_editor(paths: &Paths, command: Option<&str>) -> Result<(), String> {
    match command {
        None => {
            let config = load_config(paths);
            match &config.editor {
                Some(e) => println!("Configured editor: {e}"),
                None => println!("Editor not configured."),
            }
            match detect_editor(&config) {
                Some(e) => println!("Resolved editor: {e}"),
                None => println!("{}", "No editor detected.".yellow()),
            }
        }
        Some(cmd) => {
            let binary = cmd.split_whitespace().next().unwrap_or(cmd);
            if which::which(binary).is_err() {
                return Err(format!(
                    "Editor '{}' not found in PATH. Please verify the command.",
                    binary
                ));
            }
            let mut config = load_config(paths);
            config.editor = Some(cmd.to_string());
            save_config(paths, &config)?;
            println!("Editor set to: {}", cmd.green());
        }
    }
    Ok(())
}
