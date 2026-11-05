# G1 样本第二批：heavy-test-guard.sh，主 agent 前缀规矩与子 agent 的正当命令。
from cases_heavy import add, CASES
T = "TREE"
add("M01", "主agent", "cargo test --all", 2, "主 agent 不带前缀：提交时才跑", agent=None)
add("M02", "主agent", "SINGLEFS_HEAVY_TESTS=commit cargo test --all", 0, "主 agent 带前缀放行", agent=None)
add("M03", "主agent", "$c", 2, "主 agent 不带前缀，认不出照拒", agent=None, truth=False)
add("M04", "主agent", "SINGLEFS_HEAVY_TESTS=commit $c", 0, "主 agent 带前缀，认不出放行（用户口径：同重型那张表）", agent=None, truth=False)
add("M05", "主agent", "SINGLEFS_HEAVY_TESTS=commit; cargo test --all", 2, "分号隔开的赋值不进 cargo 的环境（真 bash）", agent=None)
add("M06", "主agent", "export SINGLEFS_HEAVY_TESTS=commit; cargo test --all", 0, "export 过的进 cargo 的环境", agent=None)
add("M07", "主agent", "SINGLEFS_HEAVY_TESTS=commit echo x | xargs cargo test --workspace", 2, "前缀只落在管道前一段", agent=None)
add("M08", "主agent", "env -i SINGLEFS_HEAVY_TESTS=commit cargo test --all", 0, "env -i 之后重新给的变量带得进去", agent=None, truth=False)
add("M09", "主agent", "SINGLEFS_HEAVY_TESTS=commit env -i cargo test --all", 2, "env -i 清掉前缀", agent=None)
add("M10", "主agent", "SINGLEFS_HEAVY_TESTS=commit true; cargo test --all", 2, "前缀只管第一条", agent=None)
add("M11", "主agent", "SINGLEFS_HEAVY_TESTS=commit bash -c 'cargo test --all'", 0, "前缀带进 bash -c", agent=None)
add("M12", "主agent", "SINGLEFS_HEAVY_TESTS=x cargo test --all", 2, "前缀值不对", agent=None)
add("M13", "主agent", "SINGLEFS_HEAVY_TESTS=commit; export SINGLEFS_HEAVY_TESTS; cargo test --all", 0, "先赋后 export", agent=None)
add("M14", "主agent", "SINGLEFS_HEAVY_TESTS=commit BASH_ENV=bashenv.sh bash -c true", 0, "带前缀：启动文件里的 cargo test 放行", agent=None, truth=False)
add("M15", "主agent", "BASH_ENV=bashenv.sh bash -c true", 2, "主 agent 不带前缀：BASH_ENV 指到的脚本里是 cargo test --all", agent=None)
add("M16", "主agent", "x=$(case a in a) cargo test --all;; esac)", 2, "主 agent 不带前缀：命令替换里不加括号的 case", agent=None)
add("M17", "主agent", "coproc cargo test --all", 2, "主 agent 不带前缀：coproc", agent=None)
add("M18", "主agent", "set -- test --all; cargo \"$@\"", 2, "主 agent 不带前缀：位置参数拼旗标", agent=None)
for n, c in enumerate([
    "ls -la", "git status", "git diff HEAD -- .claude/agent-common.md", "grep -rn heavy .claude/hooks | head", "sed -n 1,10p CLAUDE.md",
    "python3 -I scripts/agents/cite-check.py /dev/null", "for f in a b; do echo $f; done", "x=$(date +%F); echo $x", "TZ=Asia/Tokyo date +%F", "sha256sum CLAUDE.md",
    "ps -o pid,args -u \"$(id -u)\" | head -3", "du -sh .claude", "nice -n 19 python3 -c 'print(1)'", "timeout 10 bash -c 'echo hi'", "command -v cargo",
    "[ -x \"$(command -v cargo)\" ] && echo ok", "cat <<'EOF' > /dev/null\ncargo test --all\nEOF", "echo 'cargo test --all' > /dev/null", "grep -n 'cargo test' CLAUDE.md",
    "echo \"cargo test --all\"", "cat CLAUDE.md | wc -l", "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib",
    "bash scripts/hosts/run-with-memory-cap.sh 4G bash -c 'cargo test -p singlefs-harness --lib'", "cargo fmt --all -- --check", "cargo build -p singlefs-harness",
    "cargo test --no-run -p singlefs-harness", "find . -name '*.rs' -exec grep -l foo {} \;", "find . -name '*.md' | xargs grep -l foo", "echo x | xargs echo",
    "bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib",
    "bash -n scripts/hosts/capped.sh", "python3 scripts/kb/quote-kb.py --selftest > /dev/null 2>&1; echo rc=$?",
    "f() { echo hi; }; f", "a=1; b=2; echo $((a+b))", "while read -r l; do echo \"$l\"; done < CLAUDE.md", "if [ -f CLAUDE.md ]; then echo y; fi",
    "case $(uname) in Linux) echo l;; esac", "x=$(case a in a) echo A;; esac); echo $x", "x=$(echo hi # )\necho there); echo $x", "echo {a,b}", "echo \"{a,b}\"", "echo '{cargo,test,--all}'",
    "'{cargo,test,--all}'",
    ], 1):
    add("A%02d" % n, "正当命令", c, 0, "这条本身不起重活", truth=False, cwd=T)
