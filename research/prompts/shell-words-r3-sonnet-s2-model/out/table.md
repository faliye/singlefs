| # | 命令原样 | 角色 | 该判 | 钩子判（重型闸/检出钩子退出码） | 归类 | 文件头写明看不见/接受 |
|---|---|---|---|---|---|---|
| 1 | `cat <<'EOF' \| tee -a {U}/w.sh >/dev/null↵cd {C} && cargo test --workspace↵EOF↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 2 | `cat <<'EOF' \| tee {U}/w.sh >/dev/null↵cd {C} && cargo test --workspace↵EOF↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 3 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵cat {U}/w.sh &> {U}/w2.sh; bash {U}/w2.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 4 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵cat {U}/w.sh >> {U}/w2.sh; bash {U}/w2.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 5 | `cat {U}/s.sh &> {U}/cat.sh; bash {U}/cat.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 6 | `cat {U}/s.sh >& {U}/cat.sh; bash {U}/cat.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 7 | `cat {U}/s.sh >> {U}/cat.sh; bash {U}/cat.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 8 | `cat {U}/s.sh \| tee -a {U}/t.sh >/dev/null; bash {U}/t.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 9 | `cat {U}/s.sh \| tee {U}/t.sh >/dev/null; bash {U}/t.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A1 脚本拷出用 >> / &> / >& / tee 再执行 | 是 |
| 10 | `bash -s < {U}/s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 11 | `bash < {U}/s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 12 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵bash -s < {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 13 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵cat {U}/w.sh \| bash` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 14 | `cat {U}/s.sh \| bash` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 15 | `sh -s < {U}/s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 16 | `tee {U}/t.sh < {U}/s.sh >/dev/null; bash {U}/t.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A2 标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash） | 是 |
| 17 | `cat > {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵echo {U}/w.sh \| xargs bash` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 18 | `echo FOO=1 \| xargs env cargo test --workspace` | 主带前缀、主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 19 | `echo FOO=1 \| xargs env cargo test -p singlefs-harness --test t1` | 子 | 拒（H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 20 | `echo extra \| bash {R}/scripts/hosts/run-with-memory-cap.sh 4G xargs cargo test --workspace` | 主带前缀、主无前缀、子 | 拒（H1） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 21 | `echo extra \| xargs -I{} cargo test --workspace` | 主带前缀、主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 22 | `echo extra \| xargs -I{} cargo test -p singlefs-harness --test t1` | 子 | 拒（H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 23 | `echo extra \| xargs -I{} sh -c 'cargo test --workspace'` | 主带前缀、主无前缀、子 | 拒（H1） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 24 | `echo extra \| xargs -I{} sh -c 'cargo test -p singlefs-harness --test t1'` | 子 | 拒（H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 25 | `echo extra \| xargs cargo test --workspace` | 主带前缀、主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 26 | `echo extra \| xargs cargo test -p singlefs-harness --test t1` | 子 | 拒（H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 27 | `echo hi \| xargs claude -p` | 子 | 拒（D13） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 28 | `echo {U}/s.sh \| xargs bash` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 29 | `printf '%s\0' cargo test --workspace \| xargs -0 echo >/dev/null; printf '%s\0' cargo test --workspace \| xargs -0 -n3 env` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A3 xargs 起的命令/脚本（含 xargs env、xargs bash） | 是 |
| 30 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G doas cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 31 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G find . -maxdepth 0 -exec cargo test --workspace \;` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 32 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G numactl -N0 cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 33 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G script -qec 'cargo test --workspace' /dev/null` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 34 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G setpriv --reuid=$(id -u) cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 35 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G ssh localhost 'cargo test --workspace'` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 36 | `bash {R}/scripts/hosts/run-with-memory-cap.sh 4G unshare --user cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | A4 名单外启动器：script | 否 |
| 37 | `find . -maxdepth 0 -exec cargo test --workspace \;` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 38 | `find . -maxdepth 0 -exec cargo test -p singlefs-harness --test t1 \;` | 子 | 拒（H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 39 | `find . -maxdepth 0 -execdir cargo test --workspace \;` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 40 | `find . -maxdepth 0 -execdir cargo test -p singlefs-harness --test t1 \;` | 子 | 拒（H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 41 | `find {U} -name 's.sh' -exec bash {} \;` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A4 名单外启动器：find | 否 |
| 42 | `ssh localhost 'cargo test --workspace'` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | A4 名单外启动器：ssh | 否 |
| 43 | `ssh localhost 'cargo test -p singlefs-harness --test t1'` | 子 | 拒（H2） | 放（0/0） | A4 名单外启动器：ssh | 否 |
| 44 | `ssh localhost 'claude -p hi'` | 子 | 拒（D13） | 放（0/0） | A4 名单外启动器：ssh | 否 |
| 45 | `echo hi > '{D}/o.{a,b}'` | 子(Edit) | 放（-） | 拒（0/2） | B2 重定向目标花括号展开成几个词 | 否 |
| 46 | `echo hi > {D}/o.{a,b}` | 子(Edit) | 放（-） | 拒（0/2） | B2 重定向目标花括号展开成几个词 | 否 |
| 47 | `echo hi > {D}/o{1..2}.txt` | 子(Edit) | 放（-） | 拒（0/2） | B2 重定向目标花括号展开成几个词 | 否 |
| 48 | `function run { "$@"; }; run cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 49 | `m() { bash {R}/scripts/hosts/run-with-memory-cap.sh 4G "$@"; }; m cargo test --workspace` | 主无前缀、子 | 拒（H1） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 50 | `r() { env "$@"; }; r cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 51 | `run() { "$@"; }; run cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 52 | `run_logged() { local l=$1; shift; "$@" > "$l" 2>&1; }; run_logged {D}/g.log cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 53 | `w() { nice "$@"; }; w cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | C 函数当启动器（run_logged 一类） | 否 |
| 54 | `bash -c "$(cat {U}/s.sh)"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 55 | `bash -c "$(printf '%s' 'cargo test --workspace')"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 56 | `bash <(echo 'cargo test --workspace')` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 57 | `c='cargo test --workspace'; eval "$c"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 eval 拼出命令 | 是 |
| 58 | `eval "$(cat {U}/s.sh)"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 eval 拼出命令 | 是 |
| 59 | `source <(cat {U}/s.sh)` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 60 | `source <(echo 'cargo test --workspace')` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z1 进程替换 / 命令替换里拼出命令或脚本 | 是 |
| 61 | `i=0; while [ $i -lt 3 ]; do sleep 0.1; i=$((i+1)); done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 62 | `sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 63 | `until [ -f {U}/ready ]; do sleep 1; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 64 | `while ! [ -f {U}/ready ]; do sleep 1; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 65 | `while read -r l; do sleep 0.1; done <<< foo` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 66 | `while sleep 0.1 && false; do :; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 67 | `while true; do sleep 0.2; break; done` | 主、子 | 放（-） | 拒（0/2） | Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询） | 是 |
| 68 | `sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1` | 主(bg) | 拒（D3/4） | 放（0/0） | Z11 两个 & 只等其中一个（wait $p1），另一个留下 | 是 |
| 69 | `sleep 1 & wait -n` | 主(bg) | 放（-） | 拒（0/2） | Z12 bg 里 `& … wait -n`（单个作业，真等到了）按没等算 | 是 |
| 70 | `sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done` | 主(bg) | 放（-） | 拒（0/2） | Z13 bg 里 `tail --pid` / `kill -0` 轮询等它，文件头写了算等，实际仍按没等拒 | 相反 |
| 71 | `sleep 2 & tail --pid=$! -f /dev/null` | 主(bg) | 放（-） | 拒（0/2） | Z13 bg 里 `tail --pid` / `kill -0` 轮询等它，文件头写了算等，实际仍按没等拒 | 相反 |
| 72 | `w() { wait; }; sleep 1 & w` | 主(bg) | 放（-） | 拒（0/2） | Z14 wait 写在函数里、函数被调用，认不出 | 否 |
| 73 | `F=--workspace; cargo test $F` | 主无前缀 | 拒（H1） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 74 | `P=singlefs-checker-tier; cargo test -p "$P"` | 主无前缀 | 拒（H1） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 75 | `P=singlefs-checker-tier; cargo test -p $P` | 主无前缀 | 拒（H1） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 76 | `cargo $(echo test --workspace)` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 77 | `cargo $(echo test) --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 78 | `t={U}/w.sh; cat > $t <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵bash $t` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出 | 否 |
| 79 | `bash -c '$0 test --workspace' cargo` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令 | 否 |
| 80 | `bash -c 'cargo "$1" "$2"' _ test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令 | 否 |
| 81 | `bash -c 'cargo "$@"' _ test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令 | 否 |
| 82 | `sh -c 'exec "$@"' sh cargo test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令 | 否 |
| 83 | `"$(command -v cargo)" test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 命令词由命令替换 / set -- 算出 | 是 |
| 84 | `$(which cargo) test --workspace` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 命令词由命令替换 / set -- 算出 | 是 |
| 85 | `a=(cargo test --workspace); "${a[@]}"` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径） | 是 |
| 86 | `bash $(echo {U}/s.sh)` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径） | 是 |
| 87 | `c='cargo test --workspace'; $c` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径） | 是 |
| 88 | `c=claude; $c -p hi` | 子 | 拒（D13） | 放（0/0） | Z2 变量里拼出命令词 | 是 |
| 89 | `for f in {U}/s.sh; do bash $f; done` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径） | 是 |
| 90 | `set -- cargo test --workspace; "$@"` | 主无前缀、子 | 拒（H1） | 放（0/0） | Z2 命令词由命令替换 / set -- 算出 | 是 |
| 91 | `x=reset; git $x` | 主、子 | 拒（G15） | 放（0/0） | Z2 变量里拼出命令词（git $x） | 是 |
| 92 | `PATH={U} s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z3 命令词不带斜杠、经 PATH 找到的脚本 | 是 |
| 93 | `dd if={U}/s.sh of={U}/dd.sh 2>/dev/null; bash {U}/dd.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 94 | `dd of={U}/w.sh 2>/dev/null <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 95 | `echo 'cd {C} && cargo test --workspace' > {U}/w.sh; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 96 | `echo 'cd {C} && cargo test --workspace' >> {U}/w.sh; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 97 | `echo -e 'cd {C} && cargo test --workspace' > {U}/w.sh; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 98 | `head -n 9 {U}/s.sh > {U}/h.sh; bash {U}/h.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 99 | `install -m755 /dev/stdin {U}/w.sh <<'EOF'↵cd {C} && cargo test --workspace↵EOF↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 100 | `printf '%s\n' 'cd {C} && cargo test --workspace' > {U}/w.sh; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 101 | `python3 - <<'PY'↵open('{U}/w.sh','w').write('cd {C} && cargo test --workspace')↵PY↵bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 102 | `python3 -c "open('{U}/w.sh','w').write('cd {C} && cargo test --workspace')"; bash {U}/w.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 103 | `sed 's/x/x/' {U}/s.sh > {U}/se.sh; bash {U}/se.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行 | 是 |
| 104 | `case a in b) cargo test --workspace;; esac` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 105 | `f() { cargo test --workspace; }; echo done` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 106 | `f() { while true; do sleep 1; done; }; echo ok` | 主、子 | 放（-） | 拒（0/2） | Z5 没被调用的函数体按字面判（循环） | 否 |
| 107 | `false && cargo test --workspace` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 108 | `for i in; do cargo test --workspace; done` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 109 | `if false; then cargo test --workspace; fi` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 110 | `until true; do cargo test --workspace; done` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 111 | `while false; do cargo test --workspace; done` | 主无前缀、子 | 放（-） | 拒（2/0） | Z5 永远不执行的分支 / 没被调用的函数体按字面拒 | 否 |
| 112 | `bash <<< 'claude -p hi'` | 子 | 拒（D13） | 放（0/0） | Z6 ⑬ claude：here-string / env -S 里写 claude 认不出（①与重型闸对同一写法认得） | 否 |
| 113 | `env -S 'claude -p hi'` | 子 | 拒（D13） | 放（0/0） | Z6 ⑬ claude：here-string / env -S 里写 claude 认不出（①与重型闸对同一写法认得） | 否 |
| 114 | `bash {R}/scripts/agents/watch.sh \| tee /dev/null` | 主(bg) | 拒（W1） | 放（0/0） | Z7 ① 看门狗输出经 `| tee /dev/null` 丢掉 | 否 |
| 115 | `echo '# x' > {U}/n.sh; rsync --inplace {U}/n.sh scripts/hosts/capped.sh` | 主 | 拒（W7） | 放（0/0） | Z8 ⑦ 没点名 rsync --inplace（同 inode 改脚本） | 否 |
| 116 | `bash {U}/*s.sh` | 主无前缀、子 | 拒（H1; H2） | 放（0/0） | Z9 bash 的脚本路径参数里带通配，没按 bash 展开去读 | 是 |
