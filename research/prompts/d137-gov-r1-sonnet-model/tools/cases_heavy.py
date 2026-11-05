# G1 样本：heavy-test-guard.sh。want=2 该拒 / 0 该放（依据写在 why：用户口径或真 bash 行为）；truth=True 表示同时拿真 bash + cargo 垫片量「这条命令真起没起 cargo test」。
W = "implementation-writer"
CASES = []
def add(i, fam, cmd, want, why, agent=W, truth=True, env=None, cwd=None):
    CASES.append(dict(id=i, fam=fam, cmd=cmd, want=want, why=why, agent=agent, truth=truth, env=env or {}, cwd=cwd))

H = "cargo test --all"
# ---- U 认不出的一律拒（用户口径）：子 agent，每条真 bash 里都真起 cargo test --all
add("U01", "认不出", "c='cargo test --all'; $c", 2, "命令词写在变量里、展开出来夹空白，且是 cargo test --all")
add("U02", "认不出", "c=cargo; $c test --all", 2, "命令词写在变量里，前面赋过字面值，展开后是 cargo test --all")
add("U03", "认不出", "$(which cargo) test --all", 2, "命令词是命令替换")
add("U04", "认不出", "set -- cargo test --all; \"$@\"", 2, "命令词是位置参数")
add("U05", "认不出", "bash <(echo cargo test --all)", 2, "bash 起进程替换")
add("U06", "认不出", "source <(echo cargo test --all)", 2, "source 进程替换")
add("U07", "认不出", "bash -c \"$(cat f.txt)\"", 2, "bash -c 的参数是命令替换")
add("U08", "认不出", "c='cargo test --all'; eval \"$c\"", 2, "eval 变量")
add("U09", "认不出", "for f in heavy.sh; do bash $f; done", 2, "循环变量作脚本路径；真 bash 起 heavy.sh")
add("U10", "认不出", "cargo $(echo test --all)", 2, "cargo 子命令是命令替换")
add("U11", "认不出", "bash -c 'cargo \"$@\"' _ test --all", 2, "位置参数拼 cargo 参数")
add("U12", "认不出", "F=--all; cargo test $F", 2, "变量拼 cargo 旗标")
add("U13", "认不出", "cargo test ${X:---all}", 2, "默认值展开的旗标")
add("U14", "认不出", "PATH=pathdir:$PATH evil.sh", 2, "设了 PATH 的命令词，找到的脚本里是 cargo test --all")
add("U15", "认不出", "bash g*.sh", 2, "bash 起通配路径，排头一个是 g1.sh")
add("U16", "认不出", "cat f.txt | bash", 2, "管道喂给 shell 的标准输入不是字面")
add("U17", "认不出", "bash -s < f.txt", 2, "bash -s 读盘上文件")
add("U18", "认不出", "BASH_ENV=bashenv.sh bash -c true", 2, "BASH_ENV 指到的文件被执行")
add("U19", "认不出", "ENV=bashenv.sh sh -c true", 2, "ENV 偏严：dash 非交互其实不读（见 truth）；用户口径偏严", truth=True)
add("U20", "认不出", "echo x | xargs bash", 2, "xargs 接 bash")
add("U21", "认不出", "find . -maxdepth 0 -exec bash heavy.sh \;", 2, "find -exec 起 bash 脚本（字面路径，真起）")
add("U22", "认不出", "run() { \"$@\"; }; run cargo test --all", 2, "脚本里自己的转发函数")
add("U23", "认不出", "echo 'cargo test --all' > h2.sh; bash h2.sh", 2, "写出再起的脚本（字面）")
add("U24", "认不出", "$(echo echo) 'cargo test --all' > h3.sh; bash h3.sh", 2, "前面一条认不出的命令写过，后面执行（③）")
add("U25", "认不出", "BASH_ENV=$(echo bashenv.sh) bash -c true", 2, "启动文件路径解不出")
add("U26", "认不出", "./execd.sh", 2, "直接执行 #!/bin/bash 脚本，正文是 cargo test --all")
# 三种静默漏拒（改成拒）
add("S01", "三种静默", "x=$(case a in a) cargo test --all;; esac)", 2, "命令替换里不加括号的 case 模式（用户定改成拒）")
add("S02", "三种静默", "x=$(echo hi # )\ncargo test --all)", 2, "命令替换里注释里的 )（用户定改成拒）")
add("S03", "三种静默", "coproc cargo test --all", 2, "coproc（用户定改成拒）")
add("S04", "三种静默", "coproc X { cargo test --all; }", 2, "coproc 命名形式")
add("S05", "三种静默", "set -- test --all; cargo \"$@\"", 2, "Z15 位置参数拼 cargo 旗标")
add("S06", "三种静默", "F=--all; cargo test $F", 2, "Z16 变量拼 cargo 旗标")
add("S07", "三种静默", "x=$(case a in (a) cargo test --all;; esac)", 2, "带括号的 case 模式（改前就认）")
# 字面写法对照：本该拒
for n, c in enumerate([
    "cargo test --all", "cargo test --workspace", "'cargo' test --all", "\"cargo\" \"test\" \"--all\"", "\\cargo test --all",
    "c\\argo test --all", "ca\"\"rgo test --all", "$'cargo' test --all", "cargo $'test' --all", "cargo test --al\"l\"",
    "{cargo,test,--all}", "cargo test --{all,x}", "cargo {test,test} --all", "command cargo test --all", "builtin command cargo test --all",
    "exec cargo test --all", "env cargo test --all", "env -i PATH=bin cargo test --all", "env -S 'cargo test --all'", "nice -n 19 cargo test --all",
    "timeout 5 cargo test --all", "time cargo test --all", "time -p cargo test --all", "stdbuf -o0 cargo test --all", "taskset -c 0 cargo test --all",
    "flock lockfile cargo test --all", "setsid --wait cargo test --all", "ionice -c3 cargo test --all", "chrt -o 0 cargo test --all",
    "bash -c 'cargo test --all'", "sh -c \"cargo test --all\"", "bash -lc 'cargo test --all'", "dash -c 'cargo test --all'", "eval 'cargo test --all'",
    "eval \"cargo test --all\"", "bash <<< 'cargo test --all'", "echo 'cargo test --all' | bash", "echo 'cargo test --all' | sh -s", "bash <<EOF\ncargo test --all\nEOF",
    "( cargo test --all )", "{ cargo test --all; }", "if true; then cargo test --all; fi", "true && cargo test --all", "false || cargo test --all",
    "! cargo test --all", "while true; do cargo test --all; break; done", "for i in 1; do cargo test --all; done", "case x in x) cargo test --all;; esac",
    "[[ -n x ]] && cargo test --all", "until cargo test --all; do :; done", "echo $(cargo test --all)", "echo `cargo test --all`", "cat <(cargo test --all)",
    "x=$(cargo test --all)", "echo \"$(cargo test --all)\"", "[ -n \"$(cargo test --all)\" ]", "diff <(cargo test --all) <(true)", "A=1 cargo test --all",
    "A=$(date) cargo test --all", ">/dev/null cargo test --all", "2>&1 cargo test --all", "cargo test --all >/dev/null 2>&1", "cargo +nightly test --all",
    "cargo t --all", "./bin/cargo test --all", "bin/cargo test --all", "cd bin && ./cargo test --all", "f() { cargo test --all; }; f",
    "function f { cargo test --all; }; f", "f() { cargo test --all; } && f", "find . -maxdepth 0 -exec cargo test --all \;", "find . -maxdepth 0 -execdir cargo test --all ';'",
    "echo x | xargs -I{} cargo test --all", "xargs -a f.txt cargo test", "nohup cargo test --all >/dev/null 2>&1 &", "(cargo test --all) &", "cargo test --all &",
    ], 1):
    add("L%02d" % n, "字面对照", c, 2, "字面起 cargo test（子 agent 不经内存包装、全量）", truth=True)
