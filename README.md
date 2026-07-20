# code_enter

`code_enter` is a small Rust CLI for saving directory aliases and jumping to
deep paths quickly from your shell.

The executable cannot change the current directory of its parent shell by
itself, so `code_enter init` installs a small shell function named `enter`.
That function calls the executable, captures the path printed by `jp`, and then
runs `cd` / `Set-Location` in the current shell.

## Install the shell function

Build or download the `code_enter` executable, then run the command for your
shell.

Windows PowerShell:

```powershell
code_enter init powershell
```

Windows Command Prompt:

```cmd
code_enter.exe init cmd
```

macOS zsh:

```bash
code_enter init zsh
```

macOS/Linux bash:

```bash
code_enter init bash
```

Restart your terminal, or reload your shell profile. For `cmd.exe`, restart the
prompt so the updated user PATH is picked up.

If your profile already contains an `enter` function, `init` will stop instead
of overwriting it. To append the managed `code_enter` wrapper anyway:

```powershell
code_enter init powershell --force
```

## Usage

```powershell
enter add rust D:\Code\Rust
enter add tauri D:\Code\Client\tauri
enter list
enter jp rust
enter ed rust D:\Code\RustProjects
enter del tauri
```

Commands:

```text
add <alias> <path>   Add a new alias. Duplicate aliases are rejected.
ed <alias> <path>    Edit an alias. Missing aliases are added.
del <alias>          Delete an alias.
jp <alias>           Jump to an alias through the shell wrapper.
list                 Show all aliases.
init <shell>         Install the shell wrapper. Supports powershell, cmd, bash, zsh.
```

## Config File

Aliases are stored automatically in a user config file. You do not need to
create or move this file manually.

Default locations:

```text
Windows: %APPDATA%\code_enter\config.json
Linux:   $XDG_CONFIG_HOME/code_enter/config.json or ~/.config/code_enter/config.json
macOS:   ~/Library/Application Support/code_enter/config.json
```

For testing or custom setups, set `CODE_ENTER_CONFIG_DIR` to override the config
directory.
