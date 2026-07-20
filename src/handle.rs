use crate::command::{Commands, Shell};
use crate::file_reader;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const START_MARKER: &str = "# >>> code_enter init >>>";
const END_MARKER: &str = "# <<< code_enter init <<<";

pub fn handle(cmd: &Commands) -> i32 {
    match run(cmd) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            1
        }
    }
}

fn run(cmd: &Commands) -> Result<(), String> {
    match cmd {
        Commands::Add { alias, path } => add(alias, path),
        Commands::Del { alias } => delete(alias),
        Commands::Ed { alias, path } => edit(alias, path),
        Commands::Jp { alias } => jump(alias),
        Commands::List => list(),
        Commands::Init { shell, force } => init(shell, *force),
    }
}

fn add(alias: &str, path: &str) -> Result<(), String> {
    let mut map = file_reader::load_map()?;
    file_reader::add_alias(&mut map, alias, path)?;
    file_reader::save_map(&map)?;

    println!("{alias} -> {path} added");
    Ok(())
}

fn delete(alias: &str) -> Result<(), String> {
    let mut map = file_reader::load_map()?;

    if file_reader::delete_alias(&mut map, alias) {
        file_reader::save_map(&map)?;
        println!("{alias} deleted");
        Ok(())
    } else {
        Err(format!("{alias} does not exist"))
    }
}

fn edit(alias: &str, path: &str) -> Result<(), String> {
    let mut map = file_reader::load_map()?;
    let existed = file_reader::edit_alias(&mut map, alias, path);
    file_reader::save_map(&map)?;

    if existed {
        println!("{alias} -> {path} updated");
    } else {
        println!("{alias} did not exist, added {alias} -> {path}");
    }

    Ok(())
}

fn jump(alias: &str) -> Result<(), String> {
    let map = file_reader::load_map()?;
    let route =
        file_reader::find_alias(&map, alias).ok_or_else(|| format!("{alias} does not exist"))?;

    println!("{}", route.path);
    Ok(())
}

fn list() -> Result<(), String> {
    let map = file_reader::load_map()?;

    if map.alias_path.is_empty() {
        println!("No aliases configured");
        return Ok(());
    }

    println!("Available aliases:");
    for route in map.alias_path {
        println!("{:<12} {}", route.alias, route.path);
    }

    Ok(())
}

fn init(shell: &Shell, force: bool) -> Result<(), String> {
    let exe_path =
        env::current_exe().map_err(|err| format!("Unable to locate current executable: {err}"))?;

    match shell {
        Shell::Powershell => init_powershell(&exe_path, force),
        Shell::Cmd => init_cmd(&exe_path, force),
        Shell::Bash => init_posix_shell(&exe_path, &bash_profile_path()?, force),
        Shell::Zsh => init_posix_shell(&exe_path, &zsh_profile_path()?, force),
    }
}

fn init_powershell(exe_path: &Path, force: bool) -> Result<(), String> {
    let profile_paths = powershell_profile_paths()?;
    let block = powershell_wrapper(exe_path);

    for profile_path in &profile_paths {
        write_managed_block(profile_path, &block, force)?;
        println!("Updated {}", profile_path.display());
    }

    Ok(())
}

fn init_posix_shell(exe_path: &Path, profile_path: &Path, force: bool) -> Result<(), String> {
    let block = posix_wrapper(exe_path);
    write_managed_block(profile_path, &block, force)?;
    println!("Updated {}", profile_path.display());
    Ok(())
}

fn init_cmd(exe_path: &Path, force: bool) -> Result<(), String> {
    if !cfg!(windows) {
        return Err("cmd init is only supported on Windows".to_string());
    }

    let exe_dir = exe_path.parent().ok_or_else(|| {
        format!(
            "Unable to locate parent directory of {}",
            exe_path.display()
        )
    })?;
    let wrapper_path = exe_dir.join("enter.bat");
    let wrapper = cmd_wrapper(exe_path);

    if wrapper_path.exists() {
        let content = fs::read_to_string(&wrapper_path)
            .map_err(|err| format!("Unable to read {}: {err}", wrapper_path.display()))?;
        if !content.contains("CODE_ENTER_EXE") && !force {
            return Err(format!(
                "{} already exists. Re-run with --force to overwrite it.",
                wrapper_path.display()
            ));
        }
    }

    fs::write(&wrapper_path, wrapper)
        .map_err(|err| format!("Unable to write {}: {err}", wrapper_path.display()))?;
    add_to_user_path(exe_dir)?;

    println!("Updated {}", wrapper_path.display());
    println!(
        "Added {} to the user PATH if it was not already present",
        exe_dir.display()
    );
    println!("Restart cmd.exe before using `enter` from a new prompt");
    Ok(())
}

fn write_managed_block(profile_path: &Path, block: &str, force: bool) -> Result<(), String> {
    if let Some(parent) = profile_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Unable to create {}: {err}", parent.display()))?;
    }

    let content = match fs::read_to_string(profile_path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(format!("Unable to read {}: {err}", profile_path.display())),
    };

    let managed_block = format!("{START_MARKER}\n{block}\n{END_MARKER}");
    let updated = if let Some(start) = content.find(START_MARKER) {
        let end = content[start..]
            .find(END_MARKER)
            .map(|end| start + end + END_MARKER.len())
            .ok_or_else(|| {
                format!(
                    "{} contains a code_enter start marker without an end marker",
                    profile_path.display()
                )
            })?;

        format!("{}{}{}", &content[..start], managed_block, &content[end..])
    } else {
        if content.contains("function enter") || content.contains("enter()") {
            if !force {
                return Err(format!(
                    "{} already contains an enter function. Re-run with --force to append the managed code_enter wrapper.",
                    profile_path.display()
                ));
            }
        }

        let separator = if content.trim().is_empty() {
            ""
        } else {
            "\n\n"
        };
        format!("{content}{separator}{managed_block}\n")
    };

    fs::write(profile_path, updated)
        .map_err(|err| format!("Unable to write {}: {err}", profile_path.display()))
}

fn powershell_profile_paths() -> Result<Vec<PathBuf>, String> {
    if cfg!(windows) {
        let documents = env::var("USERPROFILE")
            .map(PathBuf::from)
            .map_err(|_| "USERPROFILE is not set".to_string())?
            .join("Documents");

        return Ok(vec![
            documents
                .join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
            documents
                .join("WindowsPowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
        ]);
    }

    Ok(vec![home_dir()?
        .join(".config")
        .join("powershell")
        .join("Microsoft.PowerShell_profile.ps1")])
}

fn bash_profile_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".bashrc"))
}

fn zsh_profile_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".zshrc"))
}

fn home_dir() -> Result<PathBuf, String> {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map(PathBuf::from)
        .map_err(|_| "Neither HOME nor USERPROFILE is set".to_string())
}

fn powershell_wrapper(exe_path: &Path) -> String {
    format!(
        "function enter {{\n    $output = & '{}' @args\n    if ($args[0] -eq 'jp' -and $LASTEXITCODE -eq 0) {{\n        Set-Location $output\n    }} else {{\n        $output\n    }}\n}}",
        powershell_single_quote(&exe_path.to_string_lossy())
    )
}

fn posix_wrapper(exe_path: &Path) -> String {
    format!(
        "enter() {{\n    local output\n    output=\"$({} \"$@\")\"\n    local code_enter_status=$?\n    if [ \"$1\" = \"jp\" ] && [ $code_enter_status -eq 0 ]; then\n        cd \"$output\"\n    else\n        printf '%s\\n' \"$output\"\n        return $code_enter_status\n    fi\n}}",
        posix_single_quote(&exe_path.to_string_lossy())
    )
}

fn cmd_wrapper(exe_path: &Path) -> String {
    format!(
        "@echo off\r\nsetlocal\r\nset \"CODE_ENTER_EXE={}\"\r\nif /I \"%~1\"==\"jp\" (\r\n    for /f \"usebackq delims=\" %%I in (`\"%CODE_ENTER_EXE%\" %*`) do (\r\n        endlocal\r\n        cd /d \"%%I\"\r\n        goto :eof\r\n    )\r\n    endlocal\r\n    exit /b 1\r\n) else (\r\n    \"%CODE_ENTER_EXE%\" %*\r\n    exit /b %ERRORLEVEL%\r\n)\r\n",
        cmd_path(&exe_path.to_string_lossy())
    )
}

fn add_to_user_path(dir: &Path) -> Result<(), String> {
    let dir = dir.to_string_lossy();
    let current_path = user_path()?;

    if current_path
        .split(';')
        .any(|part| part.trim_matches('"').eq_ignore_ascii_case(&dir))
    {
        return Ok(());
    }

    let status = std::process::Command::new("setx")
        .arg("PATH")
        .arg(append_path_entry(&current_path, &dir))
        .status()
        .map_err(|err| format!("Unable to run setx: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("setx failed with status {status}"))
    }
}

fn user_path() -> Result<String, String> {
    let output = std::process::Command::new("reg")
        .args(["query", r"HKCU\Environment", "/v", "Path"])
        .output()
        .map_err(|err| format!("Unable to query user PATH: {err}"))?;

    if !output.status.success() {
        return Ok(String::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("Path ") || trimmed.starts_with("PATH ") {
            let mut parts = trimmed.split_whitespace();
            parts.next();
            parts.next();
            return Ok(parts.collect::<Vec<_>>().join(" "));
        }
    }

    Ok(String::new())
}

fn append_path_entry(current_path: &str, entry: &str) -> String {
    if current_path.trim().is_empty() {
        entry.to_string()
    } else {
        format!("{current_path};{entry}")
    }
}

fn powershell_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn posix_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn cmd_path(value: &str) -> String {
    value.replace('%', "%%")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_profile_path(test_name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        env::temp_dir()
            .join("code_enter_tests")
            .join(format!("{test_name}_{}_{}", std::process::id(), nonce))
            .join("profile")
    }

    #[test]
    fn write_managed_block_appends_to_empty_profile() {
        let path = temp_profile_path("write_managed_block_appends_to_empty_profile");

        write_managed_block(&path, "enter() {}", false).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains(START_MARKER));
        assert!(content.contains("enter() {}"));
        assert!(content.contains(END_MARKER));
    }

    #[test]
    fn write_managed_block_replaces_existing_managed_block() {
        let path = temp_profile_path("write_managed_block_replaces_existing_managed_block");

        write_managed_block(&path, "old", false).unwrap();
        write_managed_block(&path, "new", false).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(!content.contains("old"));
        assert!(content.contains("new"));
    }

    #[test]
    fn write_managed_block_rejects_existing_enter_without_force() {
        let path = temp_profile_path("write_managed_block_rejects_existing_enter_without_force");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "enter() {}\n").unwrap();

        let result = write_managed_block(&path, "new", false);

        assert!(result.is_err());
    }

    #[test]
    fn write_managed_block_appends_existing_enter_with_force() {
        let path = temp_profile_path("write_managed_block_appends_existing_enter_with_force");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "enter() {}\n").unwrap();

        write_managed_block(&path, "new", true).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("enter() {}"));
        assert!(content.contains("new"));
    }

    #[test]
    fn cmd_wrapper_uses_bat_cd_for_jump() {
        let wrapper = cmd_wrapper(Path::new(r"D:\Tools\code_enter.exe"));

        assert!(wrapper.contains("if /I \"%~1\"==\"jp\""));
        assert!(wrapper.contains("cd /d \"%%I\""));
        assert!(wrapper.contains(r#"set "CODE_ENTER_EXE=D:\Tools\code_enter.exe""#));
    }

    #[test]
    fn append_path_entry_handles_empty_and_existing_path() {
        assert_eq!(append_path_entry("", r"D:\Tools"), r"D:\Tools");
        assert_eq!(
            append_path_entry(r"C:\Windows", r"D:\Tools"),
            r"C:\Windows;D:\Tools"
        );
    }
}
