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

fn powershell_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn posix_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
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
}
