### Z1 进程替换 / 命令替换里拼出命令或脚本（14 个角色×写法，7 条命令）

文件头怎么说：是（heavy-test-guard.sh:195/182「喂给 shell 的不是字面」）

- `source <(echo 'cargo test --workspace')`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `bash <(echo 'cargo test --workspace')`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `bash -c "$(printf '%s' 'cargo test --workspace')"`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `c='cargo test --workspace'; eval "$c"`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- 同子类另有 3 条，见判错表

### Z10 真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询）（14 个角色×写法，7 条命令）

文件头怎么说：是（detector:109「带计数上限的 while 循环机器分不出上限，照拒」；其余同一原理）

- `i=0; while [ $i -lt 3 ]; do sleep 0.1; i=$((i+1)); done`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `while ! [ -f {U}/ready ]; do sleep 1; done`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `while read -r l; do sleep 0.1; done <<< foo`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- 同子类另有 3 条，见判错表

### Z11 两个 & 只等其中一个（wait $p1），另一个留下（1 个角色×写法，1 条命令）

文件头怎么说：是（detector 文件头 ② 末「之后的 wait "$pid" 只等了其中一个」只记不拒）

- `sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1`（主(bg)）：该判**拒**；真 bash：没有桩被起；残留进程 sleep 2；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z12 bg 里 `& … wait -n`（单个作业，真等到了）按没等算（1 个角色×写法，1 条命令）

文件头怎么说：是（detector:29「`wait -n` 不算」）

- `sleep 1 & wait -n`（主(bg)）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2

### Z13 bg 里 `tail --pid` / `kill -0` 轮询等它，文件头写了算等，实际仍按没等拒（2 个角色×写法，2 条命令）

文件头怎么说：相反（detector:29「或之后有 `tail --pid` / `while|until … kill -0` 轮询」）

- `sleep 2 & tail --pid=$! -f /dev/null`（主(bg)）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done`（主(bg)）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2

### Z14 wait 写在函数里、函数被调用，认不出（1 个角色×写法，1 条命令）

文件头怎么说：否

- `w() { wait; }; sleep 1 & w`（主(bg)）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2

### Z15 cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出（9 个角色×写法，6 条命令）

文件头怎么说：否（文件头只写命令词与脚本路径）

- `cargo $(echo test --workspace)`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `P=singlefs-checker-tier; cargo test -p $P`（主无前缀）：该判**拒**；真 bash：cargo test -p singlefs-checker-tier（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `F=--workspace; cargo test $F`（主无前缀）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `P=singlefs-checker-tier; cargo test -p "$P"`（主无前缀）：该判**拒**；真 bash：cargo test -p singlefs-checker-tier（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- 同子类另有 2 条，见判错表

### Z16 bash -c / sh -c 的脚本里用位置参数 $0 $1 "$@" 拼出命令（8 个角色×写法，4 条命令）

文件头怎么说：否

- `bash -c 'cargo "$@"' _ test --workspace`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `sh -c 'exec "$@"' sh cargo test --workspace`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `bash -c '$0 test --workspace' cargo`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `bash -c 'cargo "$1" "$2"' _ test --workspace`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z2 变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径）（17 个角色×写法，9 条命令）

文件头怎么说：是（heavy-test-guard.sh:195「变量里拼出来的命令与脚本路径」）

- `a=(cargo test --workspace); "${a[@]}"`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `c='cargo test --workspace'; $c`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `set -- cargo test --workspace; "$@"`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `"$(command -v cargo)" test --workspace`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- 同子类另有 5 条，见判错表

### Z3 命令词不带斜杠、经 PATH 找到的脚本（2 个角色×写法，1 条命令）

文件头怎么说：是（heavy-test-guard.sh:199「命令词不带斜杠、从 PATH 里找到的脚本」）

- `PATH={U} s.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z4 脚本用 dd / sed / head / printf / echo / install / python 写出再执行（22 个角色×写法，11 条命令）

文件头怎么说：是（heavy-test-guard.sh:176「认不出的写法（printf、echo、python 写文件」；dd/sed/head/install 没点名，同一原理）

- `sed 's/x/x/' {U}/s.sh > {U}/se.sh; bash {U}/se.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `echo 'cd {C} && cargo test --workspace' >> {U}/w.sh; bash {U}/w.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `python3 - <<'PY'↵open('{U}/w.sh','w').write('cd {C} && cargo test --workspace')↵PY↵bash {U}/w.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `head -n 9 {U}/s.sh > {U}/h.sh; bash {U}/h.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- 同子类另有 7 条，见判错表

### Z5 没被调用的函数体按字面判（循环）（16 个角色×写法，8 条命令）

文件头怎么说：否

- `f() { while true; do sleep 1; done; }; echo ok`（主、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=0 detector=2
- `false && cargo test --workspace`（主无前缀、子）：该判**放**；真 bash：没有桩被起；bash 退出码 1；钩子退出码 heavy=2 detector=0
- `until true; do cargo test --workspace; done`（主无前缀、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=2 detector=0
- `f() { cargo test --workspace; }; echo done`（主无前缀、子）：该判**放**；真 bash：没有桩被起；bash 退出码 0；钩子退出码 heavy=2 detector=0
- 同子类另有 4 条，见判错表

### Z6 ⑬ claude：here-string / env -S 里写 claude 认不出（①与重型闸对同一写法认得）（2 个角色×写法，2 条命令）

文件头怎么说：否（detector ① 写了 here-string 字面照命令判；⑬ 那一条只写了 heredoc）

- `bash <<< 'claude -p hi'`（子）：该判**拒**；真 bash：claude -p hi（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0
- `env -S 'claude -p hi'`（子）：该判**拒**；真 bash：claude -p hi（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z7 ① 看门狗输出经 `| tee /dev/null` 丢掉（1 个角色×写法，1 条命令）

文件头怎么说：否

- `bash {R}/scripts/agents/watch.sh | tee /dev/null`（主(bg)）：该判**拒**；真 bash：scripts/agents/watch.sh （前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z8 ⑦ 没点名 rsync --inplace（同 inode 改脚本）（1 个角色×写法，1 条命令）

文件头怎么说：否

- `echo '# x' > {U}/n.sh; rsync --inplace {U}/n.sh scripts/hosts/capped.sh`（主）：该判**拒**；真 bash：没有桩被起；仓内改动 scripts/hosts/capped.sh；bash 退出码 0；钩子退出码 heavy=0 detector=0

### Z9 bash 的脚本路径参数里带通配，没按 bash 展开去读（2 个角色×写法，1 条命令）

文件头怎么说：是（heavy-test-guard.sh:195「通配与花括号写出来的脚本里的正文」）

- `bash {U}/*s.sh`（主无前缀、子）：该判**拒**；真 bash：cargo test --workspace（前缀=无，内存包装=无）；bash 退出码 0；钩子退出码 heavy=0 detector=0

