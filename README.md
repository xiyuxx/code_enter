<h1>INTRO</h1>
A cmd tool writing by rust used serde and clap crate.

It is used to quickly open some directories that are too deep</h2>

<h1>ATTENTION</h1>
<h3>
I'm new to Rust so there are some places I didn't handled well.
<br/>
If you are using or studying Rust, it seems not a big deal.
<br/>
<br/>
This tool needs three files:
the .exe(/target/debug), the lang_map.json and the goal_path.txt <br/>
Store them everywhere you like.
<br>
<del>the question is there are two const variables(Line 9 and 12) in the src/file_reader.rs  needs to be fit the path you store the three files manually (Fatal problem)
and run cargo build</del>   
<h2>Now you can put the program everywhere you like, it'll dynamically handle the path.</h2>
</h3>

<h1>USAGE</h1>
<h3>You should create a powershell function in $PROFILE to call it like this.</h3>


```powershell
# choose the function name you like!
function enter() {
    # the path you store the .exe file
    D:\Tool\enter\code_enter.exe $args
    if($args[0] -eq "jp"){
        $goal_path = "D:\Tool\enter\goal_path.txt"

        $goal = (Get-Content -Path $goal_path)[0]
        $ifjump = (Get-Content -Path $goal_path)[-1]

        if ($ifjump) {
            Set-Location $goal
        }
    }
}
```

<h3>Incidentally, Windows PowerShell does not have a configuration file by default, and you can directly follow the path and file name output by $PROFILE to create a new file. </h3>

``` powershell
 >> $PROFILE
C:\Users\UserName\Documents\PowerShell\Microsoft.PowerShell_profile.ps1
```

Then you can call it in powershell as follows

```powershell
enter list
```

Output will like this

```powershell
Available maps are as follows~
vue    : D:\Code\Client\vue\vue_code
rust   : D:\Code\Rust
tauri  : D:\Code\Client\tauri
cele   : D:\Games\Steam\steamapps\common\Celeste\Mods
```

It contains five subcommands, input -help to check them
<br/>
add and ed(means edit) require two args: alias and path
del(means delete) and jp(jump) require one arg: alias

```powershell
enter jp alias # jump to the goal path!
enter del alias
enter add alias path
enter ed alias path
```
