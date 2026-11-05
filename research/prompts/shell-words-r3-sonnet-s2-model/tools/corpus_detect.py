"""家族 D：检出钩子的几道。每条写 (名字, 命令, 放出追踪的机制, 之后没人等的单独 &)。
手标的依据是 bash-command-detector.sh 文件头 ①–⑮ 与 .claude/agent-common.md「执行前拒绝的写法」；
真 bash 只用来核"命令跑完后是不是真有进程留下"（left_behind），标的和观测对不上的在报告里单列。"""
from corpus_base import add

# 名字, 命令, mech3（③ 点名的放出追踪写法）, unwaited（单独 & 之后同一层没有等它的 wait）
BG = [
    ("plain-amp", "sleep 2 &", False, True), ("amp-wait", "sleep 1 & wait", False, False),
    ("amp-wait-pid", "sleep 1 & p=$!; wait $p", False, False), ("amp-wait-bang", "sleep 1 & wait $!", False, False),
    ("two-amp-wait-one", "sleep 1 & p1=$!; sleep 2 & p2=$!; wait $p1", False, True),
    ("two-amp-wait-all", "sleep 1 & sleep 1 & wait", False, False), ("amp-wait-n", "sleep 1 & wait -n", False, False),
    ("amp-subshell-wait", "sleep 2 & (wait)", False, True), ("bashc-amp-then-wait", "bash -c 'sleep 2 &'; wait", False, True),
    ("bashc-amp-inner-wait", "bash -c 'sleep 1 & wait'", False, False), ("group-amp", "{ sleep 2; echo x; } &", False, True),
    ("group-amp-wait", "{ sleep 1; echo x; } & wait", False, False), ("amp-devnull", "sleep 2 >/dev/null 2>&1 &", False, True),
    ("nohup-amp", "nohup sleep 2 &", True, True), ("nohup-fg", "nohup sleep 1", False, False), ("disown", "sleep 2 & disown", True, True),
    ("disown-h", "sleep 2 & disown -h", True, True), ("setsid-f", "setsid -f sleep 2", True, False), ("setsid-fork", "setsid --fork sleep 2", True, False),
    ("coproc", "coproc sleep 2", True, False), ("coproc-named", "coproc X { sleep 2; }", True, False),
    ("tmux", "tmux new-session -d 'sleep 2'", True, False), ("screen", "screen -dm sleep 2", True, False),
    ("systemd-run", "systemd-run --user sleep 2", True, False), ("systemd-run-wait", "systemd-run --wait sleep 1", False, False),
    ("systemd-run-pipe", "systemd-run --pipe sleep 1", False, False), ("systemd-run-scope", "systemd-run --scope sleep 1", False, False),
    ("subshell-amp", "( sleep 2 & )", False, True), ("cmdsub-amp", "echo $(sleep 2 &)", False, True),
    ("for-amp-wait", "for i in 1 2; do sleep 1 & done; wait", False, False), ("for-amp-nowait", "for i in 1 2; do sleep 2 & done", False, True),
    ("amp-echo-pid", "sleep 2 & echo $!", False, True), ("amp-newline-wait", "sleep 1 &\nwait", False, False),
    ("andand-not-bg", "sleep 1 && echo ok", False, False), ("amp-redirect-not-bg", "sleep 1 &>/dev/null", False, False),
    ("gtamp-not-bg", "sleep 1 >& /dev/null", False, False), ("pipe-amp-not-bg", "sleep 1 |& cat", False, False),
    ("amp-in-quotes", "echo 'a & b'", False, False), ("amp-in-url", "curl -s 'http://x/?a=1&b=2' >/dev/null 2>&1 || true", False, False),
    ("sed-amp", "echo a | sed 's/a/&&/'", False, False), ("timeout-amp-wait", "timeout 5 sleep 1 & wait", False, False),
    ("timeout-amp", "timeout 5 sleep 2 &", False, True), ("wait-tail-pid", "sleep 2 & tail --pid=$! -f /dev/null", False, False),
    ("wait-kill0-poll", "sleep 1 & p=$!; while kill -0 $p 2>/dev/null; do sleep 0.2; done", False, False),
    ("bashc-nohup", "bash -c 'nohup sleep 2 &'", True, True), ("env-nohup-amp", "env A=1 nohup sleep 2 &", True, True),
    ("capped-amp", "bash {R}/scripts/hosts/capped.sh 31 sleep 2 &", False, True),
    ("cmd-amp-then-tail-wait", "sleep 1 & tail -n1 /dev/null; wait", False, False),
    ("amp-then-wait-in-if", "sleep 1 & if true; then wait; fi", False, False),
    ("amp-then-wait-in-fn", "w() { wait; }; sleep 1 & w", False, False),
    ("heredoc-bash-amp", "bash <<'EOF'\nsleep 2 &\nEOF", False, True),
    ("heredoc-bash-amp-wait", "bash <<'EOF'\nsleep 1 &\nwait\nEOF", False, False),
    ("heredoc-data-amp", "cat <<'EOF'\nsleep 2 &\nEOF", False, False),
    ("py-heredoc-amp", "python3 - <<'PY'\nprint(1 & 2)\nPY", False, False),
]
for name, cmd, mech3, unwaited in BG:
    left = mech3 or unwaited
    for bg in (False, True):
        for role in ("main", "sub"):
            want = []
            if role == "sub" and bg:
                want = [("D10", "子 agent 用 run_in_background（⑩）")]
            elif mech3 or (bg and unwaited):
                want = [("D3/4", "放出追踪（③）" if mech3 else "bg 里单独 & 之后没有等它的 wait（④）")]
            add(f"D-bg{int(bg)}-{name}", cmd, roles=(role,), bg=bg, cwd="R", want=want, prefix_variants=False,
                note=f"手标；依据 detector 文件头 ② ③ ④ 与 agent-common 拒绝写法；left_behind={left}")

LOOPS = [  # 名字, 命令, 真会一直转（前台 timeout 才叫没停）, setup
    ("until-never", "until [ -f {U}/never ]; do sleep 1; done", True, ""), ("while-true", "while true; do sleep 1; done", True, ""),
    ("while-colon", "while :; do sleep 1; done", True, ""), ("until-false", "until false; do sleep 1; done", True, ""),
    ("while-notf", "while [ ! -f {U}/never ]; do sleep 1; done", True, ""), ("brace-loop", "{ while true; do sleep 1; done; }", True, ""),
    ("subshell-loop", "(while true; do sleep 1; done)", True, ""), ("bashc-loop", "bash -c 'while true; do sleep 1; done'", True, ""),
    ("timeout-bashc-loop", "timeout 2 bash -c 'while true; do sleep 1; done'", False, ""), ("timeout-capped-loop", "bash {R}/scripts/hosts/capped.sh 31 timeout 2 bash -c 'until false; do sleep 1; done'", False, ""),
    ("fn-loop-called", "f() { while true; do sleep 1; done; }; f", True, ""), ("fn-loop-notcalled", "f() { while true; do sleep 1; done; }; echo ok", False, ""),
    ("loop-comment", "echo ok # while true; do sleep 1; done", False, ""), ("loop-echo-data", "echo 'while true; do sleep 1; done'", False, ""),
    ("loop-heredoc-data", "cat <<'EOF'\nwhile true; do sleep 1; done\nEOF", False, ""),
    ("loop-heredoc-bash", "bash <<'EOF'\nwhile true; do sleep 1; done\nEOF", True, ""),
    ("loop-py-data", "python3 - <<'PY'\nprint('while true; do sleep 1; done')\nPY", False, ""),
    ("loop-break", "while true; do sleep 0.2; break; done", False, ""), ("loop-counter", "i=0; while [ $i -lt 3 ]; do sleep 0.1; i=$((i+1)); done", False, ""),
    ("until-file-exists", "until [ -f {U}/ready ]; do sleep 1; done", False, "touch {U}/ready"), ("while-notf-exists", "while ! [ -f {U}/ready ]; do sleep 1; done", False, "touch {U}/ready"),
    ("until-grep-timeout-inside", "until timeout 5 grep -q x /dev/null; do sleep 1; done", True, ""),
    ("while-sleep-cond", "while sleep 1; do :; done", True, ""), ("for-sleep", "for i in 1 2 3; do sleep 0.1; done", False, ""),
    ("while-read-sleep", "while read -r l; do sleep 0.1; done <<< foo", False, ""), ("while-read-nosleep", "while read -r l; do :; done <<< foo", False, ""),
    ("loop-multiline", "until [ -f {U}/never ]\ndo\n  sleep 1\ndone", True, ""), ("loop-bgfile", "until grep -q rc= /tmp/claude-1000/x/tasks/zz.output 2>/dev/null; do sleep 1; done", True, ""),
    ("loop-no-sleep", "until [ -f {U}/ready ]; do :; done", False, "touch {U}/ready"),
    ("loop-sleep-in-cond-only", "while sleep 0.1 && false; do :; done", False, ""),
]
for name, cmd, forever, setup in LOOPS:
    for bg in (False, True):
        for role in ("main", "sub"):
            if role == "sub" and bg:
                want = [("D10", "子 agent 用 run_in_background（⑩）")]
            elif not bg and forever:
                want = [("D2", "前台没有超时的等待循环（②）")]
            elif bg and "tasks/" in cmd and forever:
                want = [("D2", "bg 里轮询本会话后台任务输出文件（②）")]
            else:
                want = []
            add(f"D-loop{int(bg)}-{name}", cmd, roles=(role,), bg=bg, cwd="R", want=want, setup=setup, prefix_variants=False, timeout=8,
                note=f"手标；依据 detector ②；真会一直转={forever}")
