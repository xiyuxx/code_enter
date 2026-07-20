# code_enter

`code_enter` 是一个用 Rust 写的小型命令行工具，用来给常用目录保存别名，并在终端里快速跳转到这些目录。

例如你可以把 `D:\Code\RustProjects` 记成 `rust`，之后直接执行：

```powershell
enter jp rust
```

就能跳转到对应目录。

## 为什么需要 init

命令行程序本身不能直接修改父级终端的当前目录。也就是说，`code_enter.exe` 这个子进程不能单独完成 `cd`。

所以本项目采用的方式是：

1. `code_enter jp <alias>` 查询别名，并把目标路径输出到 stdout。
2. shell 函数 `enter` 捕获这个路径。
3. `enter` 在当前终端里执行 `cd` / `Set-Location`。

`code_enter init` 会自动把这个 `enter` 函数写入你的 shell profile。

## 安装与初始化

先构建或下载 `code_enter` 可执行文件，然后根据你使用的终端运行下面其中一个命令。

PowerShell：

```powershell
code_enter init powershell
```

Bash：

```bash
code_enter init bash
```

Zsh：

```bash
code_enter init zsh
```

执行后重启终端，或者手动重新加载对应的 profile 文件。

如果你的 profile 里已经存在 `enter` 函数，`init` 会停止，避免直接覆盖你的配置。如果你确认要追加 `code_enter` 管理的 wrapper，可以使用：

```powershell
code_enter init powershell --force
```

## 使用方式

添加别名：

```powershell
enter add rust D:\Code\Rust
enter add tauri D:\Code\Client\tauri
```

查看所有别名：

```powershell
enter list
```

跳转目录：

```powershell
enter jp rust
```

修改别名：

```powershell
enter ed rust D:\Code\RustProjects
```

删除别名：

```powershell
enter del tauri
```

## 命令说明

```text
add <alias> <path>   添加新别名。如果别名已存在，会拒绝添加。
ed <alias> <path>    修改别名。如果别名不存在，会自动添加。
del <alias>          删除别名。
jp <alias>           跳转到别名对应的目录，需要通过 enter 函数使用。
list                 列出所有别名。
init <shell>         安装 enter 函数，支持 powershell、bash、zsh。
```

## 配置文件

别名会自动保存到用户配置目录，不需要手动创建或移动配置文件。

默认位置：

```text
Windows: %APPDATA%\code_enter\config.json
Linux:   $XDG_CONFIG_HOME/code_enter/config.json 或 ~/.config/code_enter/config.json
macOS:   ~/Library/Application Support/code_enter/config.json
```

配置文件内容大致如下：

```json
{
  "alias_path": [
    {
      "alias": "rust",
      "path": "D:\\Code\\Rust"
    }
  ]
}
```

如果你想在测试或特殊场景下指定配置目录，可以设置环境变量：

```powershell
$env:CODE_ENTER_CONFIG_DIR = "D:\Temp\code_enter"
```

## 设计变化

旧版本需要把可执行文件、`lang_map.json` 和 `goal_path.txt` 放在一起，并通过 `goal_path.txt` 把跳转目标传给 PowerShell。

现在不再需要 `goal_path.txt`：

- 跳转路径直接通过 stdout 返回。
- `enter` 函数根据退出码决定是否跳转。
- 配置文件由程序自动放到用户配置目录。

因此用户只需要关心一个可执行文件和一次 `init`。

## 开发

运行测试：

```bash
cargo test
```

格式化代码：

```bash
cargo fmt
```
