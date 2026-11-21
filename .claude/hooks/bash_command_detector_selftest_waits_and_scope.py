# 检出钩子 bash-command-detector.sh 的 python 部分之一：自证函数 selftest_in_waits_and_scope：等待循环、放出追踪、看门狗、写范围、终止进程、扫脚本、就地改脚本的样本（selftest_in 把现场拆成两个函数，局部变量经 shared 交接）。
# 入口 bash-command-detector.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 DETECTOR_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import json
import os
import shutil
import subprocess
import tempfile
from datetime import datetime, timezone
from __main__ import DETECTOR_BREAK_VARIABLE, DETECTOR_PART_NAMES, SELFTEST_MARK_CONTENT, SELFTEST_MARK_VARIABLE, detector_break_is_set
from bash_command_detector_hooks_and_overwrites import (
    GIT_LS_FILES_RUNS, record, three_way_legs_library, unwaited_ampersand_refusal, wait_loop_refusal, writer_locks_library,
)
from bash_command_detector_literals_and_signals import (
    LEDGER_REGISTRY_PATH, bash_scope_refusal, compile_first_refusal, process_signal_refusal, repository_in_place_edit_refusal,
    repository_in_place_edit_verdict, repository_root_of, results_overwrite_refusal, script_in_place_refusal, service_source_refusal,
    work_root_removal_roots,
)
from bash_command_detector_scope_and_index import (
    COMPILE_THEN_SWAP_SCRIPT, GUARD_OFF_ITEM, GUARD_OFF_WITH_OTHERS_ITEM, JUDGING_NOTICES, LIBRARY_BREAK_VARIABLES, SHELL_WORDS_PART_NAMES,
    WRITE_SCOPE_TABLE_NAME, agent_has_definition, claude_launch_refusal, detaching_refusal, foreground_timeout_refusal, shell_words,
    watchdog_rejection, whole_target_build_refusal, work_roots,
)



def selftest_in_waits_and_scope(shared):
    hook_dir, work = shared["hook_dir"], shared["work"]
    detections = os.path.join(work, "detections.jsonl")
    # 弄坏开关只在自证里认：标记文件写进环境，自证起的子进程（入口、--scan-scripts）带着它才认调用方设的开关值
    with open(os.path.join(work, "selftest-mark"), "w", encoding="utf-8") as mark_handle:
        mark_handle.write(SELFTEST_MARK_CONTENT + "\n")
    os.environ[SELFTEST_MARK_VARIABLE] = os.path.join(work, "selftest-mark")
    # 自证在临时假仓里起的钩子拷着 lib_work_root.py，它照所在仓找 multi-host.env、假仓里没有：数据根取这一侧（真仓）算出的，交给自证起的每个钩子子进程，
    # 不然工作根在子进程里取不到，判得与这一侧不同（只在统一任务服务带了 SINGLEFS_DATA_ROOT 的那一趟才绿）
    if work_roots and not os.environ.get(work_roots.DATA_ROOT_VARIABLE):
        try:
            os.environ[work_roots.DATA_ROOT_VARIABLE] = work_roots.shared_data_root()
        except work_roots.DataRootUnconfigured:
            pass
    shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-empty")   # 系统级启动文件不读本机的 /etc（三方 d137-gov-r1 第三段那一组另换样本目录）
    cases = [
        ("普通命令", "cargo test --release", 0),
        ("带 timeout 的等待循环", "timeout 600 bash -c 'until grep -q done log; do sleep 10; done'", 0),
        ("while read 读文件", 'while read -r line; do echo "$line"; done < list.txt', 0),
        ("没有 timeout 的 until 等待", 'until grep -q "^exit=" log; do sleep 10; done', 1),
        ("按模式找进程归上游钩子，这里不记", "pgrep -f e154-binary", 0),
        ("写进笔记的 heredoc 正文里是一段没超时的等待循环", "cat > note.md <<'EOF'\nuntil grep -q done log; do sleep 10; done\nEOF", 0),
        ("喂给 bash 的 heredoc 里真是一段没超时的等待循环", "bash <<'EOF'\nuntil grep -q done log; do sleep 10; done\nEOF", 1),
        ("heredoc 之后的命令里是没超时的等待循环", "cat > note.md <<EOF\n说明\nEOF\nuntil grep -q done log; do sleep 10; done", 1),
        ("后台起的 nohup … & 加 disown", "nohup nice -n 19 bash gate.sh > gate.log 2>&1 & echo $!; disown", 1, True),
        ("后台起的命令末尾单独一个 &", "bash cache-keepalive.sh > keepalive.log 2>&1 &", 1, True),
        ("后台起的命令只有 2>&1、|& 与 &&", "cargo build 2>&1 | tail -3 && echo ok; make |& tee out", 0, True),
        ("后台起的普通命令", "bash cache-keepalive.sh", 0, True),
        ("后台起的并行加 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", 0, True),
        ("后台起的 & 在 wait 之后", "wait; bash late.sh &", 1, True),
        ("前台命令里的 & 不算（不是 run_in_background）", "sleep 1 & wait", 0),
        ('bg-notify-r1 X01', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo "started pid $!"', 1, True),
        ('bg-notify-r1 X02', '(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &)', 1, True),
        ('bg-notify-r1 X03', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo "gate.sh started as pid $!; I will wait for the notification"', 1, True),
        ('bg-notify-r1 X04', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & bash scripts/hosts/cache-keepalive.sh & wait $!', 1, True),
        ('bg-notify-r1 X05', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & bash scripts/hosts/cache-keepalive.sh & wait -n', 1, True),
        ('bg-notify-r1 X06', '(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &); wait', 1, True),
        ('bg-notify-r1 X07', 'pid=$(nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & echo $!); echo "gate pid $pid"; wait "$pid"', 1, True),
        ('bg-notify-r1 X08', 'coproc GATE { nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1; }', 1, True),
        ('bg-notify-r1 X09', "tmux new-session -d -s gate 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1'", 1, True),
        ('bg-notify-r1 X10', "screen -dmS gate bash -c 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1'", 1, True),
        ('bg-notify-r1 X11', 'systemd-run --user --collect --unit gate-run nice -n 19 bash .claude/scripts/gate.sh --staged', 1, True),
        ('bg-notify-r1 X12', "nice -n 19 python3 - > /tmp/claude-1000/gate-triage-commit-0918/rerun.log 2>&1 <<'EOF' &\nimport subprocess\np = subprocess.Popen(['bash', 'scripts/experiment-apparatus/replay.sh'])\np.wait()\nEOF", 1, True),
        ('bg-notify-r1 X13', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 &', 0, False),
        ('bg-notify-r1 F01', 'nohup nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1', 0, True),
        ('bg-notify-r1 F02（第三种不记；它的轮询循环没有 timeout，记的是第二种）', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & pid=$!; while kill -0 "$pid" 2>/dev/null; do sleep 20; done; tail -3 /tmp/claude-1000/gate-triage-commit-0918/gate-run.log', 1, True),
        ('bg-notify-r1 F03', 'nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1 & tail --pid=$! -f /tmp/claude-1000/gate-triage-commit-0918/gate-run.log > /dev/null', 0, True),
        ('bg-notify-r1 F04', "nice -n 19 python3 - <<'EOF' > /tmp/claude-1000/gate-triage-commit-0918/tally.txt\nimport json\nrows = [json.loads(line) for line in open('/tmp/claude-1000/gate-triage-commit-0918/events.jsonl')]\nprint(sum(1 for row in rows if row['flags'] & 0x4))\nEOF", 0, True),
        ('bg-notify-r1 F05', "cat > /tmp/claude-1000/impl-probe/copy/crates/singlefs-verification/tests/probe.rs <<'EOF'\nfn digest(bytes: &[u8]) -> u64 { bytes.iter().map(|&b| b as u64).sum() }\n#[test]\nfn probe() { assert_eq!(digest(&[1, 2]), 3); }\nEOF\ncd /tmp/claude-1000/impl-probe/copy && nice -n 19 cargo test --release --test probe 2>&1 | tail -20", 0, True),
        # 引号里当数据写的 & 不算（一百三十七节件 1b 改成与 ④ 同一套数法之后；弄坏开关 background-ampersand-by-regex 下照旧误记、这一格红）
        ('bg-notify-r1 F06：sed 替换串引号里的 & 不记', "nice -n 19 cargo test --release 2>&1 | sed -E 's/^test (.*) \\.\\.\\. FAILED$/!! &/' > /tmp/claude-1000/impl-probe/test.log", 0, True),
        ('bg-notify-r1 F07', 'setsid -w nice -n 19 bash .claude/scripts/gate.sh --staged > /tmp/claude-1000/gate-triage-commit-0918/gate-run.log 2>&1', 0, True),
        # 不带 --wait 的 setsid 是进程组组长时会 fork、自己马上退出（一百三十七节件 1b 第 5 条；弄坏开关 bare-setsid-unrecorded 下前两格红）
        ("一百三十七：裸 setsid 不带 --wait，照记一条", "setsid bash long.sh > long.log 2>&1 < /dev/null", 1, True),
        ("一百三十七：内存包装里的 setsid 不带 --wait，照记一条", "bash scripts/hosts/run-with-memory-cap.sh 8G setsid bash x.sh", 1, True),
        ("一百三十七：内存包装里的 setsid --wait 不记", "bash scripts/hosts/run-with-memory-cap.sh 8G setsid --wait bash x.sh", 0, True),
        ("setsid -f 放后台", "setsid -f bash long.sh > long.log 2>&1 < /dev/null", 1, True),
        ("setsid 跟着的 tail -f 不是 setsid 的选项", "setsid -w tail -f log.txt", 0, True),
        # 每个 & 记了 $!、之后逐个 wait "$pN"：与 ④ 同一套数法，不记（一百三十七节件 1b 第 5 条主 agent 的原样；弄坏开关 background-ampersand-by-regex 下红）
        ("一百三十七：两个子 shell 各记 $!、逐个 wait \"$pN\" 不记",
         '( bash a.sh > o1 2>&1; echo "rc=$?" >> o1 ) & p1=$!; ( bash b.sh > o2 2>&1 ) & p2=$!; wait $p1; wait $p2', 0, True),
    ]
    results = []
    for label, command, want, *background in cases:
        tool_input = {"command": command, "run_in_background": bool(background and background[0])}
        written = record({"tool_name": "Bash", "session_id": "s", "tool_input": tool_input}, detections)
        results.append((label, want, written))
    # 起看门狗：(说明, 命令, run_in_background, 该不该拒绝)
    watchdog_cases = [
        ("前台起、输出丢进 /dev/null 还带 &", "bash scripts/agents/watch.sh a1 > /dev/null 2>&1 &", False, 1),
        ("run_in_background 起、只写 watch.sh", "bash scripts/agents/watch.sh a1", True, 0),
        ("run_in_background 起却又带 &", "bash scripts/agents/watch.sh a1 &", True, 1),
        ("前台 grep 把 watch.sh 当参数", "grep -n watch.sh records/x.md", False, 0),
        ("前台 agent-watch.py report 不是看门狗", "python3 scripts/agents/agent-watch.py report --session-dir x", False, 0),
        ("前台起、什么都没加", "bash scripts/agents/watch.sh a1,a2", False, 1),
        ("前台起 agent-watch.py watch", "python3 scripts/agents/agent-watch.py watch --agents a1", False, 1),
        ("run_in_background 起 agent-watch.py watch", "python3 scripts/agents/agent-watch.py watch --agents a1", True, 0),
        ("run_in_background 起、带 --ack 与 cd", "cd ~/code/singlefs && bash scripts/agents/watch.sh --ack a1:上下文过大 a1", True, 0),
        ("run_in_background 起却挂 nohup", "nohup bash scripts/agents/watch.sh a1", True, 1),
        ("run_in_background 起却挂 setsid", "setsid bash scripts/agents/watch.sh --processes", True, 1),
        ("run_in_background 起却跟 disown", "bash scripts/agents/watch.sh a1; disown", True, 1),
        ("run_in_background 起却把输出丢进 /dev/null", "bash scripts/agents/watch.sh a1 > /dev/null", True, 1),
        ("前台起、写在 bash -c 里", "bash -c 'nice -n 19 ./scripts/agents/watch.sh a1'", False, 1),
        ("前台起、包在 capped.sh 里", "bash scripts/hosts/capped.sh 4 bash scripts/agents/watch.sh a1", False, 1),
        ("run_in_background 起、包在 capped.sh 里", "bash scripts/hosts/capped.sh 4 bash scripts/agents/watch.sh a1", True, 0),
        ("前台 watch.sh --selftest 不盯", "bash scripts/agents/watch.sh --selftest", False, 0),
        ("前台 watch.sh --report 不盯", "bash scripts/agents/watch.sh --report", False, 0),
        ("前台 bash -n 只查语法", "bash -n scripts/agents/watch.sh", False, 0),
        ("前台读看门狗输出", "tail -5 /tmp/claude-1000/x/tasks/b1.output; cat scripts/agents/watch.sh | head", False, 0),
        ("前台 echo 引号里当数据写", 'echo "bash scripts/agents/watch.sh a1 &"', False, 0),
        ("前台写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\nbash scripts/agents/watch.sh a1 > /dev/null 2>&1 &\nEOF", False, 0),
        ("前台注释里写的", "ls  # bash scripts/agents/watch.sh a1 &", False, 0),
        ("前台 diff <(git show …watch.sh) watch.sh：进程替换之后的词是 diff 的参数", "diff <(git show HEAD:scripts/agents/watch.sh) scripts/agents/watch.sh", False, 0),
        # 一百三十七节件 8b：/dev/null 只算看门狗所在那一段的（watchdog-null-redirect-whole-command 下该放行的两格红，照拒的四格 allow-watchdog-misuse 下红）
        ("run_in_background 起、前一段 grep 2>/dev/null、后一段 watch.sh 放行", "grep -n x records/a.md 2>/dev/null; bash scripts/agents/watch.sh a1", True, 0),
        ("run_in_background 起、前一段 grep 2>/dev/null、watch.sh 自己也丢进 /dev/null 照拒", "grep -n x records/a.md 2>/dev/null; bash scripts/agents/watch.sh a1 > /dev/null", True, 1),
        ("run_in_background 起、bash -c 整条丢进 /dev/null 照拒", "bash -c 'bash scripts/agents/watch.sh a1' > /dev/null", True, 1),
        ("run_in_background 起、花括号整组丢进 /dev/null 照拒", "{ bash scripts/agents/watch.sh a1; } > /dev/null", True, 1),
        ("run_in_background 起、管道后段把标准输出丢进 /dev/null 照拒", "bash scripts/agents/watch.sh a1 | grep x > /dev/null", True, 1),
        ("run_in_background 起、管道后段只丢 2 号放行", "bash scripts/agents/watch.sh a1 | grep x 2>/dev/null", True, 0),
    ]
    for label, command, background, want in watchdog_cases:
        results.append((f"看门狗:{label}", want, 1 if watchdog_rejection(command, background)[0] else 0))
    # 前台没超时的等待循环：(说明, 命令, run_in_background, 该不该拒绝)
    wait_loop_cases = [
        ("前台 until … sleep 没超时", "until grep -q x f; do sleep 10; done", False, 1),
        ("同一条套上 timeout 60", "timeout 60 bash -c 'until grep -q x f; do sleep 10; done'", False, 0),
        ("同一条走 run_in_background", "until grep -q x f; do sleep 10; done", True, 0),
        ("run_in_background 里轮询后台任务的输出文件", "until [ -s /tmp/claude-1000/-home-x/abc/tasks/b1x2.output ]; do sleep 20; done; tail -5 /tmp/claude-1000/-home-x/abc/tasks/b1x2.output", True, 1),
        ("run_in_background 里轮询后台任务输出、外面套了 timeout 也拒", "timeout 600 bash -c 'until grep -q done tasks/b9.output; do sleep 5; done'", True, 1),
        ("run_in_background 里读一次后台任务输出、不轮询", "tail -5 /tmp/x/tasks/b1.output", True, 0),
        ("前台按 pid 轮询的 while kill -0", 'while kill -0 "$pid" 2>/dev/null; do sleep 20; done; tail -3 log', False, 1),
        ("前台 timeout 只在循环条件里", "until timeout 5 grep -q x f; do sleep 10; done", False, 1),
        ("前台多行写的 while true", "while true\ndo\n  grep -q x f && break\n  sleep 5\ndone", False, 1),
        ("前台 sleep 在循环条件里", "while sleep 5; do grep -q x f && break; done", False, 1),
        ("前台喂给 bash 的 heredoc 里的循环", "bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 1),
        ("timeout 60 bash 的 heredoc 里的循环", "timeout 60 bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 0),
        ("capped.sh 与 nice 包着的 timeout", "nice -n 19 capped.sh 4 timeout 600 bash -c 'until grep -q x f; do sleep 10; done'", False, 0),
        ("前台 bash -c 里没有 timeout", "nice -n 19 bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        ("前台 capped.sh 包着的 bash -c 里没有 timeout", "bash scripts/hosts/capped.sh 4 bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        ("前台 run-with-memory-cap.sh 包着的 bash -c 里没有 timeout", "bash scripts/hosts/run-with-memory-cap.sh 4G bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        ("前台命令替换里的循环", "state=$(until grep -q x f; do sleep 1; done; echo ok)", False, 1),
        ("前台 if 里的循环", "if true; then while ! grep -q x f; do sleep 1; done; fi", False, 1),
        ("前台 echo 引号里当数据写", 'echo "until grep -q x f; do sleep 10; done"', False, 0),
        ("前台 grep 的参数里写着循环", "grep -n 'until .*; do sleep' notes.md", False, 0),
        ("前台写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 0),
        ("前台注释里写的", "ls  # until grep -q x f; do sleep 10; done", False, 0),
        ("前台 while read 没有 sleep", 'while read -r line; do echo "$line"; done < list.txt', False, 0),
        ("前台 sleep 在循环外面", 'while read -r line; do echo "$line"; done < list.txt; sleep 5', False, 0),
        ("前台 for 循环", "for attempt in 1 2 3; do grep -q x f && break; sleep 1; done", False, 0),
        ("前台单独的 sleep", "sleep 5 && tail -3 log", False, 0),
        ("前台 echo 的参数里有 while、后面才 sleep", "echo while done; sleep 1", False, 0),
        ("前台循环体里套 for、sleep 在 for 之后", "until grep -q x f; do for part in a b; do echo $part; done; sleep 1; done", False, 1),
        # time 关键字带 -p（LIB_SHELL_WORDS_BREAK=time-options-as-command 下这一格红）
        ("前台 time -p 包着的 bash -c 里没有 timeout", "time -p bash -c 'until grep -q x f; do sleep 10; done'", False, 1),
        # heredoc 挂在没收尾的命令替换、进程替换里的 bash 上（LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen 下前两格红）
        ("前台 $(bash <<EOF) 里的循环", "x=$(bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n)", False, 1),
        ("前台 <(bash <<EOF) 里的循环", "diff <(bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n) y", False, 1),
        ("$(timeout 60 bash <<EOF) 里的循环", "x=$(timeout 60 bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n)", False, 0),
        ("$(cat <<EOF) 喂的是数据", "x=$(cat <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n)", False, 0),
        # 同一行里先有双引号包着、已收尾的 $(…)，再接顶层的 bash <<EOF（LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed 下这两格红）
        ('前台 cd "$(…)" 之后 bash <<EOF 里的循环', "cd \"$(git rev-parse --show-toplevel)\" && bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 1),
        ('前台 echo "$(date)" | bash <<EOF 里的循环', "echo \"$(date)\" | bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF", False, 1),
        # ANSI-C 引号 $'…' 里的 \' 不收尾，之后没收尾的 $( 里的 bash <<EOF 照认（LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen 下这一格红）
        ("前台 $'a\\'b' 之后 $(bash <<EOF) 里的循环", "echo $'a\\'b'; x=$(bash <<'EOF'\nuntil grep -q x f; do sleep 10; done\nEOF\n)", False, 1),
    ]
    for label, command, background, want in wait_loop_cases:
        results.append((f"等待循环:{label}", want, 1 if wait_loop_refusal(command, background) else 0))
    # ⑬ 项目子 agent 在 Bash 里起 claude 会话：(说明, agent 类型, 命令, 该不该拒)；弄坏开关 allow-subagent-claude-launch 与 disable-check 下「拒」的八格红
    claude_cases = [
        ("子 agent 命令位置上的 claude -p", "implementation-writer", "claude -p '接着做'", 1),
        ("子 agent 经 \"$CLAUDE_CODE_EXECPATH\" 起", "clerk", '"$CLAUDE_CODE_EXECPATH" -p x', 1),
        ("子 agent 经 ${CLAUDE_CODE_EXECPATH} 起", "clerk", "${CLAUDE_CODE_EXECPATH} --print x", 1),
        ("子 agent 起以 /claude 结尾的可执行文件", "tooling-writer", "~/.local/bin/claude -p x", 1),
        ("子 agent nice 包着起 claude", "implementation-writer", "nice -n 19 claude -p x", 1),
        ("子 agent bash -c 里起 claude", "implementation-writer", "bash -c 'claude -p x'", 1),
        ("子 agent 命令替换里起 claude", "implementation-writer", "out=$(claude -p x)", 1),
        ("子 agent 喂给 bash 的 heredoc 里起 claude", "implementation-writer", "bash <<'EOF'\nclaude -p x\nEOF", 1),
        ("子 agent command -v claude 只查名字", "implementation-writer", "command -v claude", 0),
        ("子 agent grep 参数里的 claude", "implementation-writer", "grep -n claude records/x.md", 0),
        ("子 agent echo 引号里写的 claude -p", "implementation-writer", "echo 'claude -p x'", 0),
        ("子 agent 写进文件的 heredoc 正文里的 claude", "implementation-writer", "cat > /tmp/x.sh <<'EOF'\nclaude -p x\nEOF", 0),
        ("子 agent 起名字里带 claude 的别的程序", "implementation-writer", "claude-code-helper --version", 0),
        ("主 agent 起 claude 不判", None, "claude -p x", 0),
    ]
    for label, agent, command, want in claude_cases:
        results.append((f"起 claude:{label}", want, 1 if claude_launch_refusal(command, agent) else 0))
    # ⑬ shell 启动时先读的文件里起 claude（一百三十七节件 8f、8g）：弄坏开关 startup-file-claude-unread 与 LIB_SHELL_WORDS_BREAK=startup-files-unread 下「拒」的格红，
    # 各格另在 LIB_SHELL_WORDS_BREAK 的 declared-exports-unseen、interactive-startup-files-unread、zsh-startup-files-unread、executed-script-startup-unread 下红（逐格见文件头开关清单）
    startup_directory = tempfile.mkdtemp()
    try:
        for startup_name, startup_text in (("startup-claude.sh", "claude -p x\n"), ("startup-light.sh", "echo hi\n"),
                                           # 直接执行的脚本先读的启动文件（件 8g）：#! 指到 bash、sh、python 的脚本；交互的 bash 读 HOME 下的 .bashrc，zsh 读 ZDOTDIR 下的 .zshenv
                                           ("exec-bash.sh", "#!/bin/bash\necho hi\n"), ("exec-sh.sh", "#!/bin/sh\necho hi\n"), ("exec-python.py", "#!/usr/bin/env python3\nprint(1)\n"),
                                           ("home-claude/.bashrc", "claude -p x\n"), ("home-light/.bashrc", "echo hi\n"), ("zdot-claude/.zshenv", "claude -p x\n")):
            os.makedirs(os.path.dirname(os.path.join(startup_directory, startup_name)), exist_ok=True)
            with open(os.path.join(startup_directory, startup_name), "w", encoding="utf-8") as handle:
                handle.write(startup_text)
        startup_claude = os.path.join(startup_directory, "startup-claude.sh")
        for label, agent, command, want in (
                ("子 agent BASH_ENV 指到的脚本里起 claude", "implementation-writer", f"BASH_ENV={startup_claude} bash -c true", 1),
                ("子 agent env BASH_ENV=… 起 bash，脚本里起 claude", "implementation-writer", f"env BASH_ENV={startup_claude} bash -c true", 1),
                ("子 agent 前面 export BASH_ENV 再起 bash，脚本里起 claude", "implementation-writer", f"export BASH_ENV={startup_claude}; bash -c true", 1),
                ("子 agent ENV 指到的脚本里起 claude，sh 起的", "implementation-writer", f"ENV={startup_claude} sh -c true", 1),
                ("子 agent BASH_ENV 写相对路径，按命令开头的当前目录解", "implementation-writer", "BASH_ENV=startup-claude.sh bash -c true", 1),
                ("子 agent 直接执行 #!/bin/bash 的脚本，BASH_ENV 指到的脚本里起 claude", "implementation-writer", f"BASH_ENV={startup_claude} ./exec-bash.sh", 1),
                ("子 agent 直接执行 #!/bin/sh 的脚本，ENV 指到的脚本里起 claude", "implementation-writer", f"ENV={startup_claude} ./exec-sh.sh", 1),
                ("子 agent declare -x BASH_ENV 之后起 bash，脚本里起 claude", "implementation-writer", f"declare -x BASH_ENV={startup_claude}; bash -c true", 1),
                ("子 agent typeset -x BASH_ENV 之后起 bash，脚本里起 claude", "implementation-writer", f"typeset -x BASH_ENV={startup_claude}; bash -c true", 1),
                ("子 agent 先赋值后 export BASH_ENV，再起 bash，脚本里起 claude", "implementation-writer", f"BASH_ENV={startup_claude}; export BASH_ENV; bash -c true", 1),
                ("子 agent bash -i 读 HOME 下的 .bashrc 里起 claude", "implementation-writer", f"HOME={startup_directory}/home-claude bash -i -c true", 1),
                ("子 agent bash --rcfile 文件 -i，文件里起 claude", "implementation-writer", f"bash --rcfile {startup_claude} -i -c true", 1),
                ("子 agent sh -i 读 ENV 指到的脚本里起 claude", "implementation-writer", f"ENV={startup_claude} sh -i -c true", 1),
                ("子 agent zsh 读 ZDOTDIR 下的 .zshenv 里起 claude", "implementation-writer", f"ZDOTDIR={startup_directory}/zdot-claude zsh -c true", 1),
                ("子 agent export -n 撤掉 BASH_ENV 之后起 bash", "implementation-writer", f"export BASH_ENV={startup_claude}; export -n BASH_ENV; bash -c true", 0),
                ("子 agent bash -i 不读 BASH_ENV，.bashrc 里没有 claude", "implementation-writer", f"BASH_ENV={startup_claude} HOME={startup_directory}/home-light bash -i -c true", 0),
                ("子 agent zsh -f 不读启动文件", "implementation-writer", f"ZDOTDIR={startup_directory}/zdot-claude zsh -f -c true", 0),
                ("子 agent 直接执行 #!/bin/sh 的脚本不读 BASH_ENV", "implementation-writer", f"BASH_ENV={startup_claude} ./exec-sh.sh", 0),
                ("子 agent 直接执行 python 脚本不读 BASH_ENV", "implementation-writer", f"BASH_ENV={startup_claude} ./exec-python.py", 0),
                ("子 agent BASH_ENV 指到的脚本里没有 claude", "implementation-writer", f"BASH_ENV={startup_directory}/startup-light.sh bash -c true", 0),
                ("子 agent ENV 指到的脚本，bash 不带 --posix 不读", "implementation-writer", f"ENV={startup_claude} bash -c true", 0),
                ("子 agent bash -n 只查语法，不读 BASH_ENV", "implementation-writer", f"BASH_ENV={startup_claude} bash -n -c true", 0),
                ("子 agent BASH_ENV 指到的文件不在", "implementation-writer", f"BASH_ENV={startup_directory}/no-such.sh bash -c true", 0),
                ("子 agent BASH_ENV 是命令替换拼的，这一道不判（由 project-code-guard.sh 照认不出拒）", "implementation-writer", f"BASH_ENV=$(echo {startup_claude}) bash -c true", 0),
                ("主 agent BASH_ENV 指到的脚本里起 claude 不判", None, f"BASH_ENV={startup_claude} bash -c true", 0)):
            results.append((f"起 claude:一百三十七：{label}", want, 1 if claude_launch_refusal(command, agent, startup_directory) else 0))
    finally:
        shutil.rmtree(startup_directory, ignore_errors=True)
    # ⑬ 登录 shell 的 profile（LIB_SHELL_WORDS_BREAK=login-profiles-unread）、系统级启动文件（system-startup-files-unread；
    # SYSTEM_STARTUP_DIRECTORY 换成样本目录）、别的程序里起的 claude（LIB_SHELL_WORDS_BREAK=embedded-launches-unseen 与 embedded-launch-claude-unread）、
    # bash x.sh 起的脚本正文里起 claude（script-body-claude-unread）；「拒」的格在各自的开关下红（逐格见文件头开关清单）
    d137_directory = tempfile.mkdtemp()
    try:
        for d137_name, d137_text in (
                                   ('home-claude-login/.bash_profile', 'claude -p x\n'),
                                   ('home-claude-profile/.profile', 'claude -p x\n'),
                                   ('home-light-login/.bash_profile', 'echo hi\n'),
                                   ('home-light/.bashrc', 'echo hi\n'),
                                   ('system-claude/profile', 'claude -p x\n'),
                                   ('system-claude/bash.bashrc', 'claude -p x\n'),
                                   ('claude-body.sh', '#!/usr/bin/env bash\nclaude -p x\n'),
                                   ('claude-free.sh', '#!/usr/bin/env bash\necho hi\ncommand -v claude\n'),
                                   ('mk-claude/Makefile', 'all:\n\tclaude -p x\n'),
                                   ('prog-claude.awk', 'BEGIN { system("claude -p x") }\n'),
):
            os.makedirs(os.path.dirname(os.path.join(d137_directory, d137_name)), exist_ok=True)
            with open(os.path.join(d137_directory, d137_name), "w", encoding="utf-8") as handle:
                handle.write(d137_text)
        os.chmod(os.path.join(d137_directory, "claude-body.sh"), 0o755)
        for label, agent, command, want, system in (
                ('d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile 里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-claude-login bash -l -c true', 1, 'system-empty'),
                ('d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile 里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-claude-login exec -l bash -c true', 1, 'system-empty'),
                ('d137 已知没认：登录 sh -l 读 HOME 下的 .profile 里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-claude-profile sh -l -c true', 1, 'system-empty'),
                ('d137 已知没认：登录 bash -l 读的 profile 里没有 claude', 'implementation-writer', f'HOME={d137_directory}/home-light-login bash -l -c true', 0, 'system-empty'),
                ('d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录）里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-light-login bash -l -c true', 1, 'system-claude'),
                ('d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录）里起 claude', 'implementation-writer', f'HOME={d137_directory}/home-light bash -i -c true', 1, 'system-claude'),
                ('d137 已知没认：系统级 不登录不交互的 bash -c 不读 /etc 下的', 'implementation-writer', 'bash -c true', 0, 'system-claude'),
                ('d137 H9：python -c 里 os.system 起 claude', 'implementation-writer', 'python3 -c \'import os; os.system("claude -p x")\'', 1, 'system-empty'),
                ('d137 H9：perl -e 里 system 起 claude', 'implementation-writer', 'perl -e \'system("claude -p x")\'', 1, 'system-empty'),
                ('d137 H9：gawk 的 system() 起 claude', 'implementation-writer', 'gawk \'BEGIN{system("claude -p x")}\'', 1, 'system-empty'),
                ('d137 H9：sed 的 e 命令起 claude', 'implementation-writer', "sed -n '1e claude -p x' f", 1, 'system-empty'),
                ('d137 H9：git -c 别名 ! 起 claude', 'implementation-writer', "git -c alias.x='!claude -p x' x", 1, 'system-empty'),
                ('d137 H9：tclsh 读 here-string 里 exec 起 claude', 'implementation-writer', "tclsh <<< 'exec claude -p x'", 1, 'system-empty'),
                ('d137 H9：make 读 Makefile 配方起 claude', 'implementation-writer', f'make -C {d137_directory}/mk-claude', 1, 'system-empty'),
                ('d137 H9：awk -f 程序文件里 system() 起 claude', 'implementation-writer', f'awk -f {d137_directory}/prog-claude.awk f', 1, 'system-empty'),
                ('d137 H9：python -c 不起命令', 'implementation-writer', "python3 -c 'print(1)'", 0, 'system-empty'),
                ("d137 H9：awk '{print $1}' 不起命令", 'implementation-writer', "awk '{print $1}' f", 0, 'system-empty'),
                ('d137 已知没认：bash x.sh 起的脚本正文里起 claude', 'implementation-writer', f'bash {d137_directory}/claude-body.sh', 1, 'system-empty'),
                ('d137 已知没认：source 读进来的脚本正文里起 claude', 'implementation-writer', f'source {d137_directory}/claude-body.sh', 1, 'system-empty'),
                ('d137 已知没认：直接执行的 #!/usr/bin/env bash 脚本正文里起 claude', 'implementation-writer', f'{d137_directory}/claude-body.sh', 1, 'system-empty'),
                ('d137 已知没认：同一条命令里 heredoc 写出再 bash 起，正文里起 claude', 'implementation-writer', f"cat > {d137_directory}/hd137.sh <<'EOF'\nclaude -p x\nEOF\nbash {d137_directory}/hd137.sh", 1, 'system-empty'),
                ('d137 已知没认：脚本正文里只有 command -v claude，不拒', 'implementation-writer', f'bash {d137_directory}/claude-free.sh', 0, 'system-empty'),
                ('d137 已知没认：主 agent bash x.sh 不判', None, f'bash {d137_directory}/claude-body.sh', 0, 'system-empty'),
):
            shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(d137_directory, system)
            results.append((f"起 claude:{label}", want, 1 if claude_launch_refusal(command, agent, d137_directory) else 0))
    finally:
        shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-empty")
        shutil.rmtree(d137_directory, ignore_errors=True)
    # ㉑ 前台 Bash 调用的 timeout 超过 240000 毫秒拒（主 agent 与子 agent 一样；弄坏开关 allow-long-foreground-timeout 下头四格红）
    for label, hook_payload, want in (
            ("主 agent 前台 timeout 600000 毫秒拒", {"tool_input": {"command": "ls", "timeout": 600000}}, 1),
            ("主 agent 前台 timeout 240001 毫秒拒", {"tool_input": {"command": "ls", "timeout": 240001}}, 1),
            ("子 agent 前台 timeout 600000 毫秒拒", {"agent_type": "implementation-writer", "tool_input": {"command": "ls", "timeout": 600000}}, 1),
            ("主 agent 前台 timeout 写成字符串 300000 也拒", {"tool_input": {"command": "ls", "timeout": "300000"}}, 1),
            ("主 agent 前台 timeout 正好 240000 毫秒放行", {"tool_input": {"command": "ls", "timeout": 240000}}, 0),
            ("子 agent 前台 timeout 正好 240000 毫秒放行", {"agent_type": "implementation-writer", "tool_input": {"command": "ls", "timeout": 240000}}, 0),
            ("主 agent 前台没写 timeout 放行", {"tool_input": {"command": "ls"}}, 0),
            ("主 agent run_in_background 带 timeout 600000 放行", {"tool_input": {"command": "ls", "timeout": 600000, "run_in_background": True}}, 0),
            ("子 agent run_in_background 的 timeout 不在这一道（⑩ 拒）",
             {"agent_type": "clerk", "tool_input": {"command": "ls", "timeout": 600000, "run_in_background": True}}, 0),
            ("主 agent timeout 写成 true 放行", {"tool_input": {"command": "ls", "timeout": True}}, 0),
            ("主 agent timeout 写成不是数的字符串放行", {"tool_input": {"command": "ls", "timeout": "soon"}}, 0)):
        results.append((f"前台超时:{label}", want, 0 if foreground_timeout_refusal(hook_payload) is None else 1))
    # ㉓ 交给任务服务的 cargo build 点了全部目标拒（判法不看调用方，主 agent 与子 agent 一样）：(说明, 命令, 该不该拒)；起点是这一份钩子所在的仓根
    # （task_server_submission 只认本仓的客户端）。弄坏开关 allow-whole-target-build 下「拒」的格红，whole-target-full-gate-unseen 下带 --full-gate 的两格红
    whole_target_root = repository_root_of(hook_dir)
    for label, command, want in (
            ("submit 交 cargo build --all-targets 拒", "task-server/task submit -- cargo build -p singlefs-verification --offline --all-targets", 1),
            ("submit 交 cargo build --tests 拒", "task-server/task submit -- cargo build -p singlefs-verification --tests", 1),
            ("submit 交 cargo build --benches 拒", "task-server/task submit -- cargo build -p singlefs-verification --benches", 1),
            ("submit 交 cargo build --examples 拒", "task-server/task submit -- cargo build -p singlefs-verification --examples", 1),
            ("submit 交 cargo build --workspace 拒", "task-server/task submit -- cargo build --workspace", 1),
            ("submit 交 cargo build --all 拒", "task-server/task submit -- cargo build --all", 1),
            ("submit 交 cargo b --all-targets（内建别名）拒", "task-server/task submit -- cargo b --all-targets", 1),
            ("范围声明写在前面、--cwd 点名目录的 submit 照拒", "SINGLEFS_RUN_SCOPE='x' task-server/task submit --cwd crates -- cargo build --all-targets", 1),
            ("submit 之后 env 前缀里的 cargo build 照拒", "task-server/task submit -- env CARGO_INCREMENTAL=0 cargo build --all-targets", 1),
            ("submit 之后 bash -c 里 cd 再 cargo build 照拒", "task-server/task submit -- bash -c 'cd crates && cargo build --tests'", 1),
            ("bash -c 里的 submit 照拒", "bash -c 'task-server/task submit -- cargo build --all-targets'", 1),
            ("&& 连着的后一段 submit 照拒", "cd . && task-server/task submit -- cargo build --all-targets", 1),
            ("只带 --whole-package-confirmed 的 submit 照拒", "task-server/task submit --whole-package-confirmed /sample/confirmed -- cargo build --all-targets", 1),
            ("--whole-package-confirmed 的值写成 --full-gate 不算带了 --full-gate，照拒",
             "task-server/task submit --whole-package-confirmed --full-gate -- cargo build --all-targets", 1),
            ("submit 交 cargo build -p 点名包放行", "task-server/task submit -- cargo build -p singlefs-verification --offline", 0),
            ("submit 交 cargo clippy --all-targets 放行（只查不链接）", "task-server/task submit -- cargo clippy -p singlefs-verification --all-targets", 0),
            ("submit 带 --full-gate 交 cargo build --all-targets 放行", "task-server/task submit --full-gate -- cargo build --workspace --all-targets", 0),
            ("--cwd 写在 --full-gate 前面也认，放行", "task-server/task submit --cwd crates --full-gate -- cargo build --all-targets", 0),
            ("submit 交合入后验证脚本放行（脚本正文不读）",
             "task-server/task submit -- bash scripts/gate/merge-and-verify.sh --item singlefs-verification:tests/sample.rs /sample/patches", 0),
            ("submit 交 cargo test --tests 不归这一道，放行", "task-server/task submit -- cargo test -p singlefs-format --tests", 0),
            ("没经 submit 直接起的 cargo build --all-targets 不归这一道，放行", "cargo build --all-targets", 0),
            ("echo 引号里当数据写的放行", "echo 'task-server/task submit -- cargo build --all-targets'", 0),
            ("别处同名而盘上没有的 task 不认，放行", "/sample/elsewhere/task-server/task submit -- cargo build --all-targets", 0)):
        results.append((f"全部目标构建:{label}", want, 1 if whole_target_build_refusal(command, whole_target_root) else 0))
    # ⑧ 有 Edit 工具的项目子 agent 在 Bash 里就地改仓内文件：(说明, agent 类型, 命令, 该不该拒)
    edit_repository = tempfile.mkdtemp()
    # 工作根下的样本路径不看弄坏开关（lib_work_root.py 的 configured_work_root）；~ 写法在工作根落在家目录下时才有
    new_root = work_roots.configured_work_root() if work_roots else "/nonexistent-work-root"
    home = os.path.expanduser("~")
    home_new_root = "~" + new_root[len(home):] if new_root.startswith(home + "/") else new_root
    try:
        os.makedirs(os.path.join(edit_repository, ".claude", "agents"))
        open(os.path.join(edit_repository, ".claude", "agents", "experiment-runner.md"), "w").write("---\ntools: Read, Edit, Write, Bash\n---\n")
        open(os.path.join(edit_repository, ".claude", "agents", "sweep.md"), "w").write("---\ntools: Read, Bash\n---\n")
        repository_edit_cases = [
            ("执行员 sed -i 改仓内源文件", "experiment-runner", "sed -i 's/a/b/' scripts/experiment-apparatus/e1.rs", 1),
            ("执行员 python open(仓内路径, 'w')", "experiment-runner", "python3 -c \"open('crates/x/src/lib.rs', 'w').write('x')\"", 1),
            ("执行员 python Path(仓内绝对路径).write_text", "experiment-runner",
             f"python3 - <<'EOF'\nfrom pathlib import Path\nPath('{edit_repository}/.claude/kb/x.md').write_text('y')\nEOF", 1),
            ("执行员 sed -i 改草稿目录", "experiment-runner", "sed -i 's/a/b/' /tmp/claude-1000/e/notes.md", 0),
            ("执行员 python 写草稿目录", "experiment-runner", "python3 -c \"open('/tmp/claude-1000/e/x.txt', 'w').write('x')\"", 0),
            ("执行员 sed -i 改工作根下的草稿目录", "experiment-runner", f"sed -i 's/a/b/' {new_root}/e/notes.md", 0),
            ("执行员 sed -i 改工作根下的草稿目录（~ 写法）", "experiment-runner", f"sed -i 's/a/b/' {home_new_root}/e/notes.md", 0),
            ("执行员 sed -i 改工作根同名前缀的兄弟目录", "experiment-runner", f"sed -i 's/a/b/' {new_root}-other/notes.md", 1),
            ("执行员 python 只读仓内文件", "experiment-runner", "python3 -c \"print(open('crates/x/src/lib.rs').read())\"", 0),
            # os.replace / os.rename / shutil.move 挪进仓里也是就地改，scripts/ 起头的同算仓内（弄坏开关 python-move-into-repository-unseen 下前五格红）
            ("执行员 python os.replace 把草稿挪进 scripts/ 下的入库装置", "experiment-runner",
             f"python3 -c \"import os; os.replace('{new_root}/e/e1.py', 'scripts/experiment-apparatus/e1.py')\"", 1),
            ("执行员 python os.rename 挪进知识库（源是变量）", "experiment-runner", "python3 -c \"import os; os.rename(draft, '.claude/kb/a.md')\"", 1),
            ("执行员 python shutil.move 挪进仓根起的绝对路径", "experiment-runner",
             f"python3 - <<'EOF'\nimport shutil\nshutil.move('{new_root}/e/x.rs', '{edit_repository}/crates/x/src/lib.rs')\nEOF", 1),
            ("执行员 python shutil.move 用 dst= 挪进 scripts/", "experiment-runner", "python3 -c \"import shutil; shutil.move(source, dst='scripts/kb/x.py')\"", 1),
            ("执行员 python open(scripts/ 下的文件, 'w')", "experiment-runner", "python3 -c \"open('scripts/kb/x.py', 'w').write('x')\"", 1),
            ("执行员 python os.replace 的目标在草稿目录放行", "experiment-runner",
             f"python3 -c \"import os; os.replace('{new_root}/e/x.tmp', '{new_root}/e/x.py')\"", 0),
            ("执行员 python os.replace 把仓内文件挪进草稿目录放行", "experiment-runner",
             f"python3 -c \"import os; os.replace('scripts/kb/x.py', '{new_root}/e/x.py')\"", 0),
            # python 一半的相对路径从 cd 之后的目录算：cd 进草稿目录里的仓副本再写 crates/…、scripts/… 放行（弄坏开关 python-relative-path-ignores-cd 下前三格红）
            ("cd 到草稿目录里的仓副本之后 python 相对路径写 crates/ 放行", "experiment-runner",
             f"cd {new_root}/e/copy && python3 -c \"open('crates/x/src/lib.rs', 'w').write('x')\"", 0),
            ("cd 到草稿目录里的仓副本之后 python 相对路径挪进 scripts/ 放行", "experiment-runner",
             f"cd {new_root}/e/copy && python3 -c \"import os; os.replace('{new_root}/e/x.tmp', 'scripts/kb/x.py')\"", 0),
            ("cd 到草稿目录里的仓副本之后喂给 python 的 heredoc 相对路径写 scripts/ 放行", "experiment-runner",
             f"cd {new_root}/e/copy && python3 - <<'EOF'\nopen('scripts/kb/x.py', 'w').write('x')\nEOF", 0),
            ("cd 到仓根之后 python 相对路径写 crates/ 照拒", "experiment-runner",
             f"cd {edit_repository} && python3 -c \"open('crates/x/src/lib.rs', 'w').write('x')\"", 1),
            ("cd 到算不出的目录之后 python 相对路径写 crates/ 照旧按仓内判", "experiment-runner",
             "cd \"$WHERE\" && python3 -c \"open('crates/x/src/lib.rs', 'w').write('x')\"", 1),
            ("cd 到草稿目录里的仓副本之后 python 写仓根起的绝对路径照拒", "experiment-runner",
             f"cd {new_root}/e/copy && python3 -c \"open('{edit_repository}/crates/x/src/lib.rs', 'w').write('x')\"", 1),
            # heredoc 正文只在喂给 shell 或 python 时才是代码（弄坏开关 heredoc-body-judged-as-commands 下，前三格是 ⑧ 的 python 判法在整条文本上跑而红）
            ("往草稿目录 cat 写脚本，heredoc 正文里的 python open(仓内路径, 'w') 是数据", "experiment-runner",
             f"cat > {new_root}/e/x.py <<'EOF'\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 0),
            ("往草稿目录 cat 写笔记，heredoc 正文里的 python Path(仓内路径).write_text 是数据", "experiment-runner",
             f"cat > {new_root}/e/n.md <<'EOF'\nPath('records/x.md').write_text('y')\nEOF", 0),
            ("引号里 echo 的 open(仓内路径, 'w') 文字是数据", "experiment-runner", "echo \"open('.claude/kb/a.md', 'w')\"", 0),
            ("喂给 python 的 heredoc 里 open(仓内路径, 'w') 照拒", "experiment-runner", "python3 - <<'EOF'\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
            ("喂给 bash 的 heredoc 里的 sed -i 照拒", "experiment-runner", "bash <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 1),
            ("cat 写草稿脚本的 heredoc 正文里的 sed -i 是数据", "experiment-runner", f"cat > {new_root}/e/x.sh <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
            ("bash -c 里的 python3 -c 写仓内文件照拒", "experiment-runner", "bash -c \"python3 -c \\\"open('.claude/kb/a.md', 'w')\\\"\"", 1),
            ("没有 Edit 的回扫员不判", "sweep", "sed -i 's/a/b/' research/x.md", 0),
            ("主 agent 不判", None, "sed -i 's/a/b/' research/x.md", 0),
            # sed -i 的相对路径从 cd 之后的目录算（弄坏开关 sed-in-place-ignores-cd 下，前四格红；共用模块的 LIB_SHELL_WORDS_BREAK=double-slash-kept 下 // 开头写的那一格红）
            ("cd 到工作根下的草稿目录之后 sed -i 相对路径放行", "experiment-runner", f"cd {new_root}/e && sed -i 's/a/b/' x.py", 0),
            ("cd 到 /tmp/ 下的目录之后 sed -i 相对路径放行", "experiment-runner", "cd /tmp/claude-1000/e && sed -i 's/a/b/' x.py", 0),
            ("cd 到 // 开头写的 /tmp/ 下的目录之后 sed -i 相对路径放行", "experiment-runner", "cd //tmp/claude-1000/e && sed -i 's/a/b/' x.py", 0),
            ("cd 到前面赋过值的草稿目录之后 sed -i 相对路径放行", "experiment-runner", f"W={new_root}/e; cd $W && sed -i 's/a/b/' x.py", 0),
            ("cd 到仓内目录之后 sed -i 相对路径照拒", "experiment-runner", f"cd {edit_repository}/records && sed -i 's/a/b/' x.md", 1),
            ("不 cd 的仓内相对路径 sed -i 照拒", "experiment-runner", "sed -i 's/a/b/' records/x.md", 1),
            ("cd 到算不出的目录之后 sed -i 相对路径照旧按仓内判", "experiment-runner", "cd \"$WHERE\" && sed -i 's/a/b/' x.md", 1),
            ("先 cd 到草稿目录、再 cd 回仓内之后 sed -i 相对路径照拒", "experiment-runner",
             f"cd {new_root}/e && cd {edit_repository}/records && sed -i 's/a/b/' x.md", 1),
            ("cd 到草稿目录之后 sed -i 的相对路径用 .. 跳出工作根照拒", "experiment-runner", f"cd {new_root}/e && sed -i 's/a/b/' ../../../x.md", 1),
            ("cd 到草稿目录之后 sed -i 的目标带通配，照旧按仓内判", "experiment-runner", f"cd {new_root}/e && sed -i 's/a/b/' x*.py", 1),
            # 经管道喂给 python、bash 的 heredoc 正文是代码，喂给 grep 的是数据（弄坏开关 heredoc-pipe-fed-as-data 下前五格红）
            ("管道喂给 python3 - 的 heredoc 里 open(仓内路径, 'w') 照拒", "experiment-runner",
             "cat <<'EOF' | python3 -\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
            ("管道喂给 python3（不带 -）的 heredoc 里 open(仓内路径, 'w') 照拒", "experiment-runner",
             "cat <<'EOF' | python3\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
            ("管道喂给 python3 - 的 heredoc 里 Path(仓内路径).write_text 照拒", "experiment-runner",
             "cat <<'EOF' | python3 -\nfrom pathlib import Path\nPath('records/x.md').write_text('y')\nEOF", 1),
            ("echo 的文字管道喂给 python3 - 照拒", "experiment-runner", "echo \"open('.claude/kb/a.md', 'w')\" | python3 -", 1),
            ("管道喂给 bash 的 heredoc 里 sed -i 仓内文件照拒", "experiment-runner", "cat <<'EOF' | bash\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 1),
            ("管道喂给 grep 的 heredoc 正文是数据", "experiment-runner",
             "cat <<'EOF' | grep x\nopen('.claude/kb/a.md', 'w').write('x')\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
            ("echo 的文字管道喂给 grep 是数据", "experiment-runner", "echo \"open('.claude/kb/a.md', 'w')\" | grep x", 0),
            ("管道喂给 python3 脚本文件的 heredoc 正文是数据（程序从文件读）", "experiment-runner",
             "cat <<'EOF' | python3 x.py\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 0),
        ]
        for label, agent, command, want in repository_edit_cases:
            results.append((f"就地改仓内文件:{label}", want, 1 if repository_in_place_edit_refusal(command, agent, edit_repository) else 0))
        # 命令开头的当前目录（hook 输入里的 cwd）：sed -i 的相对路径从那里算（弄坏开关 sed-in-place-ignores-cd 下第一格红）
        for label, command, start_directory, want in (
                ("命令开头的当前目录在草稿目录里，sed -i 相对路径放行", "sed -i 's/a/b/' x.py", f"{new_root}/e", 0),
                ("命令开头的当前目录在仓内，sed -i 相对路径照拒", "sed -i 's/a/b/' x.py", f"{edit_repository}/records", 1)):
            results.append((f"就地改仓内文件:{label}", want,
                            1 if repository_in_place_edit_refusal(command, "experiment-runner", edit_repository, start_directory) else 0))
        # ⑧ 认 perl -i：开关簇里的 -i（合写、带备份后缀的都算），-Ilib、-MList::Util 簇里的 i 不算，只读的 perl -ne 不拦
        #   （弄坏开关 perl-in-place-unseen 下前八格红；perl-switch-cluster-by-regex 下 -I、-M 那两格红）
        perl_cases = [
            ("perl -pi -e 改仓内知识库", "perl -pi -e 's/a/b/' records/x.md", 1),
            ("perl -i -pe 改入库装置", "perl -i -pe 's/a/b/' scripts/experiment-apparatus/e1.rs", 1),
            ("perl -i.bak 带备份后缀", "perl -i.bak -pe 's/a/b/' .claude/kb/a.md", 1),
            ("perl -pi.bak -e 合写带备份后缀", "perl -pi.bak -e 's/a/b/' crates/x/src/lib.rs", 1),
            ("perl -pie 是备份后缀为 e 的 -i，后面的文件照拒", "perl -pie 's/a/b/' records/x.md", 1),
            ("perl -lpi 合写照拒", "perl -lpi -e 's/a/b/' records/x.md", 1),
            ("perl -0777 -i -pe 照拒", "perl -0777 -i -pe 's/a/b/' records/x.md", 1),
            ("bash -c 里的 perl -pi 照拒", "bash -c \"perl -pi -e 's/a/b/' records/x.md\"", 1),
            ("perl -ne 只读不拦", "perl -ne 'print if /x/' records/x.md", 0),
            ("perl -Ilib -ne 只读不拦（-I 簇里的 i 不是 -i）", "perl -Ilib -ne 'print' records/x.md", 0),
            ("perl -MList::Util=sum -ne 只读不拦（-M 簇里的 i 不是 -i）", "perl -MList::Util=sum -ne 'print' records/x.md", 0),
            ("perl -pi 改工作根下的草稿目录不拦", f"perl -pi -e 's/a/b/' {new_root}/e/notes.md", 0),
            ("perl -pi 改 /tmp/ 下的文件不拦", "perl -pi -e 's/a/b/' /tmp/claude-1000/e/x.md", 0),
            ("cd 到工作根下的草稿目录之后 perl -pi 相对路径不拦", f"cd {new_root}/e && perl -pi -e 's/a/b/' x.md", 0),
        ]
        for label, command, want in perl_cases:
            results.append((f"就地改仓内文件:{label}", want, 1 if repository_in_place_edit_refusal(command, "experiment-runner", edit_repository) else 0))
        for label, agent in (("主 agent 的 perl -pi 不判", None), ("没有 Edit 的回扫员 perl -pi 不判", "sweep")):
            results.append((f"就地改仓内文件:{label}", 0,
                            1 if repository_in_place_edit_refusal("perl -pi -e 's/a/b/' records/x.md", agent, edit_repository) else 0))
        # ⑧ 读 python3 起的、落在工作根下的脚本正文，照同一套 python 判法判；自证里读脚本的根换成临时目录 script_root
        #   （弄坏开关 python-script-body-unread 下「照拒」五格与「记一条检出」两格红；python-script-any-location-read 下仓里的脚本那一格红；
        #   python-script-unliteral-write-unnoted 下「记一条检出」两格红；python-move-into-repository-unseen 下起 fix_rs.py 的三格与 os.replace 那一格红；
        #   python-relative-path-ignores-cd 下 cd 进草稿目录起脚本那一格红）
        script_root = os.path.join(work, "in-place-scripts")
        os.makedirs(script_root)
        os.makedirs(os.path.join(edit_repository, "scripts", "kb"), exist_ok=True)
        script_bodies = {
            "fix_rs.py": "text = open('scripts/experiment-apparatus/e1.rs').read()\nopen('scripts/experiment-apparatus/e1.rs', 'w').write(text.replace('a', 'b'))\n",
            "fix_kb_absolute.py": f"from pathlib import Path\nPath('{edit_repository}/.claude/kb/x.md').write_text('y')\n",
            "move_in.py": "import os\nos.replace('draft.tmp', 'scripts/kb/x.py')\n",
            "write_scratch.py": f"open('{script_root}/out.txt', 'w').write('x')\n",
            "write_argv.py": "import sys\nopen(sys.argv[1], 'w').write('x')\n",
            "write_joined.py": "import os\ntarget = os.path.join(os.environ['BASE'], 'x.rs')\nwith open(target, 'w') as handle:\n    handle.write('x')\n",
            "read_only.py": "print(open('scripts/experiment-apparatus/e1.rs').read())\n",
        }
        for script_name, script_body in script_bodies.items():
            with open(os.path.join(script_root, script_name), "w", encoding="utf-8") as script_handle:
                script_handle.write(script_body)
        with open(os.path.join(edit_repository, "scripts", "kb", "writer.py"), "w", encoding="utf-8") as script_handle:
            script_handle.write("open('records/x.md', 'w').write('x')\n")
        script_cases = [
            ("python3 起草稿目录里的脚本按字面量改入库装置照拒", "experiment-runner", f"python3 {script_root}/fix_rs.py", 1),
            ("python3 起草稿目录里的脚本按仓根起的绝对路径写知识库照拒", "experiment-runner", f"python3 {script_root}/fix_kb_absolute.py", 1),
            ("python3 起草稿目录里的脚本用 os.replace 挪进 scripts/ 照拒", "experiment-runner", f"python3 {script_root}/move_in.py", 1),
            ("nice 前缀、带选项与参数起草稿目录里的脚本照拒", "experiment-runner", f"nice python3 -u {script_root}/fix_rs.py --dry", 1),
            ("bash -c 里起草稿目录里的脚本照拒", "experiment-runner", f"bash -c 'python3 {script_root}/fix_rs.py'", 1),
            ("cd 进草稿目录之后起脚本，相对路径落在草稿目录放行", "experiment-runner", f"cd {script_root} && python3 fix_rs.py", 0),
            ("脚本按字面量写工作根下的文件放行", "experiment-runner", f"python3 {script_root}/write_scratch.py", 0),
            ("脚本只读仓内文件放行", "experiment-runner", f"python3 {script_root}/read_only.py", 0),
            ("脚本的目标取自 sys.argv 判不出，放行", "experiment-runner", f"python3 {script_root}/write_argv.py records/x.md", 0),
            ("仓里的脚本不读（定点写入脚本这类）", "experiment-runner", "python3 scripts/kb/writer.py", 0),
            ("脚本文件不在放行", "experiment-runner", f"python3 {script_root}/missing.py", 0),
            ("主 agent 起同一份脚本不判", None, f"python3 {script_root}/fix_rs.py", 0),
            ("没有 Edit 的回扫员起同一份脚本不判", "sweep", f"python3 {script_root}/fix_rs.py", 0),
        ]
        for label, agent, command, want in script_cases:
            results.append((f"就地改仓内文件:{label}", want,
                            1 if repository_in_place_edit_refusal(command, agent, edit_repository, script_roots=(script_root,)) else 0))
        for label, command, want in (
                ("脚本的目标取自 sys.argv，记一条检出", f"python3 {script_root}/write_argv.py records/x.md", 1),
                ("脚本的目标是拼出来的变量，记一条检出", f"python3 {script_root}/write_joined.py", 1),
                ("脚本只读仓内文件不记检出", f"python3 {script_root}/read_only.py", 0),
                ("脚本按字面量写工作根下的文件不记检出", f"python3 {script_root}/write_scratch.py", 0)):
            results.append((f"就地改仓内文件:{label}", want,
                            len(repository_in_place_edit_verdict(command, "experiment-runner", edit_repository, script_roots=(script_root,)).noted)))
        # ⑨ 先编后换：experiment-runner 在 Bash 里写主工作区 crates/ 下的 .rs：(说明, agent 类型, 命令, 该不该拒)
        harness_bin = os.path.join(edit_repository, "crates", "singlefs-verification", "src", "bin")
        os.makedirs(harness_bin)
        open(os.path.join(harness_bin, "e161_x.rs"), "w").write("fn main() {}\n")
        open(os.path.join(edit_repository, "crates", "mutations.tsv"), "w").write("# 表头\n")
        bin_path = "crates/singlefs-verification/src/bin/e161_x.rs"
        draft_path = f"{new_root}/runner-x/e161_x.rs"
        compile_first_cases = [
            ("执行员 cp 草稿副本盖主工作区的入库装置", "experiment-runner", f"cp {draft_path} {bin_path}", 1),
            ("执行员 cat > 新建入库装置", "experiment-runner", "cat > crates/singlefs-verification/src/bin/e999_new.rs <<'EOF'\nfn main() {}\nEOF", 1),
            ("执行员 >> 追加", "experiment-runner", f"echo '// x' >> {bin_path}", 1),
            ("执行员 tee -a", "experiment-runner", f"echo x | tee -a {bin_path}", 1),
            ("执行员 mv 草稿副本进 bin 目录", "experiment-runner", f"mv {draft_path} crates/singlefs-verification/src/bin/", 1),
            ("执行员 cd 进 bin 目录之后 cp 到 .", "experiment-runner", f"cd crates/singlefs-verification/src/bin && cp {draft_path} .", 1),
            ("执行员 sed -i", "experiment-runner", f"sed -i 's/a/b/' {bin_path}", 1),
            ("执行员 perl -pi -e", "experiment-runner", f"perl -pi -e 's/a/b/' {bin_path}", 1),
            ("执行员 rsync", "experiment-runner", f"rsync -a {draft_path} {bin_path}", 1),
            ("执行员 rm 入库装置", "experiment-runner", f"rm {bin_path}", 1),
            ("执行员 mv 把入库装置挪走", "experiment-runner", f"mv {bin_path} {new_root}/runner-x/", 1),
            ("执行员 bash -c 里 cp", "experiment-runner", f"bash -c 'cp {draft_path} {bin_path}'", 1),
            ("执行员变量里的目录", "experiment-runner", f"d=crates/singlefs-verification/src/bin; cp {draft_path} \"$d/e161_x.rs\"", 1),
            ("执行员目标算不出、对得上已有的入库装置", "experiment-runner", f"cp {draft_path} \"$DIR\"/crates/singlefs-verification/src/bin/e161_x.rs", 1),
            ("执行员 python shutil.copy", "experiment-runner",
             f"python3 -c \"import shutil; shutil.copy('{draft_path}', '{bin_path}')\"", 1),
            ("执行员 python open(…, 'a')", "experiment-runner", f"python3 -c \"open('{bin_path}', 'a').write('x')\"", 1),
            ("执行员 python os.replace", "experiment-runner", f"python3 -c \"import os; os.replace('{draft_path}', '{bin_path}')\"", 1),
            ("执行员 python 的 for 逐个取元组，open(循环变量, 'a') 写到入库装置", "experiment-runner",
             f"python3 -c \"for p in ('{draft_path}', '{bin_path}'): open(p, 'a').write('x')\"", 1),
            ("执行员经先编后换脚本换进来放行", "experiment-runner",
             f"bash scripts/hosts/capped.sh 16 python3 {COMPILE_THEN_SWAP_SCRIPT} {draft_path} {bin_path} --scratch {new_root}/runner-x", 0),
            ("执行员把主工作区那份拷进草稿目录放行", "experiment-runner", f"cp {bin_path} {new_root}/runner-x/", 0),
            ("执行员在草稿目录里改放行", "experiment-runner", f"sed -i 's/a/b/' {draft_path}", 0),
            ("执行员追加 crates/mutations.tsv 放行", "experiment-runner", "printf 'x\\n' >> crates/mutations.tsv", 0),
            ("执行员 grep 读入库装置放行", "experiment-runner", f"grep -n main {bin_path}", 0),
            ("执行员 python 只读入库装置放行", "experiment-runner", f"python3 -c \"print(open('{bin_path}').read())\"", 0),
            ("执行员引号里当数据写的放行", "experiment-runner", f"echo 'cp {draft_path} {bin_path}'", 0),
            ("实现员同样的 cp 放行", "implementation-writer", f"cp {draft_path} {bin_path}", 0),
            ("主 agent 同样的 cp 放行", None, f"cp {draft_path} {bin_path}", 0),
        ]
        for label, agent, command, want in compile_first_cases:
            results.append((f"先编后换:{label}", want, 1 if compile_first_refusal(command, agent, edit_repository) else 0))
        # 相对路径从 hook 输入的 cwd 起算，同 ⑧ ⑫（BASH_COMMAND_DETECTOR_BREAK=compile-first-ignores-cwd 下第一格红）：
        # cwd 是草稿目录里的仓副本时 crates/… 写的是副本；cwd 在主工作区的子目录里时相对路径从那里算
        copy_root = edit_repository + "-copy"
        results.append(("先编后换:执行员 cwd 在仓副本里、相对路径写 crates/ 下的 .rs 放行", 0,
                        1 if compile_first_refusal(f"cp {draft_path} {bin_path}", "experiment-runner", edit_repository, copy_root) else 0))
        results.append(("先编后换:执行员 cwd 在主工作区的 crates/ 里、相对路径写 .rs 拒", 1,
                        1 if compile_first_refusal(f"cp {draft_path} singlefs-verification/src/bin/e161_x.rs", "experiment-runner", edit_repository,
                                                   os.path.join(edit_repository, "crates")) else 0))
        # ⑭ 工具员在 Bash 里改两个服务 src/ 下已存在的文件与已存在的 Cargo.toml、Cargo.lock：(说明, agent 类型, 命令, 该不该拒)
        #   弄坏开关 allow-service-source-write 下拒的格全红；service-source-overwrite-only 下只认整份覆盖，追加、sed -i、replace-once.py、rsync 那几格红；
        #   service-source-lexical-only 下经符号链接写的那一格红；scope-known-scripts-unseen 下 replace-once.py 那两格红（定点写脚本同一份认法）；
        #   service-source-backup-kept、-directory-copy-unseen、-link-unseen、-hardlink-inode-unseen、-extract-unseen、-removal-unseen、-unlink-find-unseen、-rsync-removal-unseen、
        #   -compressor-unseen、-in-place-tools-unseen、-read-write-redirect-unseen、-unzip-unseen、-python-delete-unseen、-python-in-place-unseen、-stdout-extract-judged、
        #   -patch-operand-ignored、scope-swap-spec-out-unseen 各红下面各自注释点名的那几格
        for service_file, service_text in (("prune-server/src/protocol.rs", "pub enum Request {}\n"), ("prune-server/src/bin/serve.rs", "fn main() {}\n"),
                                           ("gpu-verification-server/src/lib.rs", "pub fn run() {}\n"), ("prune-server/Cargo.toml", "[package]\n"),
                                           ("prune-server/Cargo.lock", "version = 3\n"), ("gpu-verification-server/Cargo.toml", "[package]\n"),
                                           ("prune-server/target/debug/prune_server", "binary\n")):
            os.makedirs(os.path.dirname(os.path.join(edit_repository, service_file)), exist_ok=True)
            with open(os.path.join(edit_repository, service_file), "w", encoding="utf-8") as handle:
                handle.write(service_text)
        service_draft = f"{new_root}/tooling-x/protocol.rs"
        service_copy = os.path.join(work, "service-source-copy")   # 草稿目录里的仓副本：同名文件在主工作区之外
        os.makedirs(os.path.join(service_copy, "prune-server", "src"))
        with open(os.path.join(service_copy, "prune-server", "src", "protocol.rs"), "w", encoding="utf-8") as handle:
            handle.write("pub enum Request {}\n")
        service_links = os.path.join(work, "service-source-links")   # 仓外的目录里放一个指到 src/ 下已存在文件的符号链接
        os.makedirs(service_links)
        os.symlink(os.path.join(edit_repository, "prune-server", "src", "protocol.rs"), os.path.join(service_links, "protocol-link.rs"))
        service_tree = os.path.join(work, "service-source-tree")   # 草稿目录里的小目录树：src/ 里的文件名与服务源码相同，fresh/ 里的文件服务源码里没有
        for tree_file in ("src/protocol.rs", "src/bin/serve.rs", "fresh/brand_new.rs"):
            os.makedirs(os.path.dirname(os.path.join(service_tree, tree_file)), exist_ok=True)
            with open(os.path.join(service_tree, tree_file), "w", encoding="utf-8") as handle:
                handle.write("// draft\n")
        service_hard_source = os.path.join(edit_repository, "prune-server", "src", "bin", "serve.rs")
        service_hard_link = os.path.join(service_links, "serve-hard.rs")   # 与 prune-server/src/bin/serve.rs 是同一个文件的硬链接；临时目录不在同一个文件系统上时建在仓里
        try:
            os.link(service_hard_source, service_hard_link)
        except OSError:
            service_hard_link = os.path.join(edit_repository, "serve-hard.rs")
            os.link(service_hard_source, service_hard_link)
        service_unrelated_link = os.path.join(service_links, "unrelated-second-name.rs")   # 与服务源码无关、有两个名字的文件
        with open(os.path.join(service_links, "unrelated.rs"), "w", encoding="utf-8") as handle:
            handle.write("// unrelated\n")
        os.link(os.path.join(service_links, "unrelated.rs"), service_unrelated_link)
        service_source_cases = [
            ("工具员 cat >> 追加已存在的 prune-server/src/protocol.rs", "tooling-writer", "cat >> prune-server/src/protocol.rs <<'EOF2'\n// probe\nEOF2", 1),
            ("工具员用 replace-once.py 改已存在的 prune-server/src/protocol.rs", "tooling-writer",
             "python3 scripts/kb/replace-once.py prune-server/src/protocol.rs 'pub ' 'pub '", 1),
            ("工具员 cp 整份盖已存在的 prune-server/src/protocol.rs", "tooling-writer", f"cp {service_draft} prune-server/src/protocol.rs", 1),
            ("工具员 tee 盖 gpu-verification-server/src/ 下已存在的文件", "tooling-writer", "echo x | tee gpu-verification-server/src/lib.rs", 1),
            ("工具员 sed -i 改 src/ 子目录里已存在的 prune-server/src/bin/serve.rs", "tooling-writer", "sed -i 's/a/b/' prune-server/src/bin/serve.rs", 1),
            ("工具员 cp 进 prune-server/src/ 目录、文件名撞上", "tooling-writer", f"cp {service_draft} prune-server/src/", 1),
            ("工具员 cd 进 src 之后追加", "tooling-writer", "cd prune-server/src && echo x >> protocol.rs", 1),
            ("工具员 mv 草稿里的文件盖 prune-server/src/protocol.rs", "tooling-writer", f"mv {service_draft} prune-server/src/protocol.rs", 1),
            ("工具员 bash -c 里 cp", "tooling-writer", f"bash -c 'cp {service_draft} prune-server/src/protocol.rs'", 1),
            ("工具员前面赋过值的变量里的目录", "tooling-writer", f"d=prune-server/src; cp {service_draft} \"$d/protocol.rs\"", 1),
            ("工具员 python open(…, 'a')", "tooling-writer", "python3 -c \"open('prune-server/src/protocol.rs', 'a').write('x')\"", 1),
            ("工具员 python shutil.copy", "tooling-writer", f"python3 -c \"import shutil; shutil.copy('{service_draft}', 'prune-server/src/protocol.rs')\"", 1),
            ("工具员 python 的 for 逐个取元组，open(循环变量, 'w') 盖 prune-server/src/protocol.rs", "tooling-writer",
             f"python3 -c \"for p in ('{new_root}/tooling-x/notes.txt', 'prune-server/src/protocol.rs'): open(p, 'w').write('x')\"", 1),
            ("工具员经符号链接 cp 写 src/ 下已存在的文件", "tooling-writer", f"cp {service_draft} {service_links}/protocol-link.rs", 1),
            ("工具员 cat >> 追加已存在的 prune-server/Cargo.toml", "tooling-writer", "cat >> prune-server/Cargo.toml <<'EOF2'\n# probe\nEOF2", 1),
            ("工具员用 replace-once.py 改已存在的 gpu-verification-server/Cargo.toml", "tooling-writer",
             "python3 scripts/kb/replace-once.py gpu-verification-server/Cargo.toml '[package]' '[package]'", 1),
            ("工具员 cp 盖已存在的 prune-server/Cargo.lock", "tooling-writer", f"cp {service_draft} prune-server/Cargo.lock", 1),
            ("工具员 sed -i 改已存在的 gpu-verification-server/Cargo.toml", "tooling-writer", "sed -i 's/a/b/' gpu-verification-server/Cargo.toml", 1),
            ("工具员新建 prune-server/src/ 下的文件放行", "tooling-writer", "cat > prune-server/src/new_module.rs <<'EOF'\npub fn x() {}\nEOF", 0),
            ("工具员新建 gpu-verification-server/Cargo.lock 放行", "tooling-writer", "cat > gpu-verification-server/Cargo.lock <<'EOF'\nversion = 3\nEOF", 0),
            ("工具员写草稿目录里的同名文件放行", "tooling-writer", f"cp {service_draft} {new_root}/tooling-x/prune-server/src/protocol.rs", 0),
            ("工具员 cwd 在仓副本里、相对路径写同名文件放行", "tooling-writer", f"cp {service_draft} prune-server/src/protocol.rs", 0),
            ("工具员改 prune-server/target/ 下的文件放行", "tooling-writer", f"cp {service_draft} prune-server/target/debug/prune_server", 0),
            ("工具员 grep 读 src/ 下的文件放行", "tooling-writer", "grep -n pub prune-server/src/protocol.rs", 0),
            ("工具员把 src/ 下的文件拷出去放行", "tooling-writer", f"cp prune-server/src/protocol.rs {new_root}/tooling-x/", 0),
            ("工具员 python 只读 src/ 下的文件放行", "tooling-writer", "python3 -c \"print(open('prune-server/src/protocol.rs').read())\"", 0),
            ("工具员 replace-once.py --dry-run 不写放行", "tooling-writer", "python3 scripts/kb/replace-once.py --dry-run prune-server/src/protocol.rs a b", 0),
            ("工具员引号里当数据写的放行", "tooling-writer", f"echo 'cp {service_draft} prune-server/src/protocol.rs'", 0),
            ("主 agent 同样的追加放行", None, "cat >> prune-server/src/protocol.rs <<'EOF2'\n// probe\nEOF2", 0),
            ("实现员同样的追加不归这一道", "implementation-writer", "cat >> prune-server/src/protocol.rs <<'EOF2'\n// probe\nEOF2", 0),
        ]
        # 带备份的拷与挪（旧文件换了名字留底，这个名字照样是新写进去的）：弄坏开关 service-source-backup-kept 下带 -b、--backup 的三格红
        service_source_cases += [
            ("工具员 cp -b 带备份盖已存在的 prune-server/src/protocol.rs", "tooling-writer", f"cp -b {service_draft} prune-server/src/protocol.rs", 1),
            ("工具员 install -b 带备份盖已存在的 prune-server/src/protocol.rs", "tooling-writer", f"install -b -m 644 {service_draft} prune-server/src/protocol.rs", 1),
            ("工具员 mv --backup=numbered 盖已存在的 prune-server/src/protocol.rs", "tooling-writer", f"mv --backup=numbered {service_draft} prune-server/src/protocol.rs", 1),
            ("工具员 install 不带 -b 盖已存在的 prune-server/src/protocol.rs", "tooling-writer", f"install -m 644 {service_draft} prune-server/src/protocol.rs", 1),
            ("工具员 cp -n 不盖已存在的 prune-server/src/protocol.rs 放行", "tooling-writer", f"cp -n {service_draft} prune-server/src/protocol.rs", 0),
            ("工具员 cp -b 写草稿目录里的同名文件放行", "tooling-writer", f"cp -b {service_draft} {new_root}/tooling-x/prune-server/src/protocol.rs", 0),
        ]
        # 源是目录的 cp -r、rsync（落点算成目录）：逐份对目标目录里已有的服务源码；弄坏开关 service-source-directory-copy-unseen 下前五格红，
        # service-source-overwrite-only 下两条 rsync 的格红（rsync 的目标只在认全部写法时认）
        service_source_cases += [
            ("工具员 cp -r 草稿/src/. 进 prune-server/src/（逐份盖已存在的源码）", "tooling-writer", f"cp -r {service_tree}/src/. prune-server/src/", 1),
            ("工具员 cp -r 草稿/src 进 prune-server/（落点是 prune-server/src）", "tooling-writer", f"cp -r {service_tree}/src prune-server/", 1),
            ("工具员 rsync -a 草稿/src/ 进 prune-server/src/（源带结尾的 /）", "tooling-writer", f"rsync -a {service_tree}/src/ prune-server/src/", 1),
            ("工具员 rsync -a 草稿/src 进 prune-server/（源不带结尾的 /，落点是 prune-server/src）", "tooling-writer", f"rsync -a {service_tree}/src prune-server/", 1),
            ("工具员 cp -rT 草稿/src 盖 prune-server/src", "tooling-writer", f"cp -rT {service_tree}/src prune-server/src", 1),
            ("工具员 cp -r 草稿/src/* 进 prune-server/src/（通配的源）", "tooling-writer", f"cp -r {service_tree}/src/* prune-server/src/", 1),
            ("工具员 cp -r 草稿/src 进 prune-server/src/（落点是 prune-server/src/src，服务里没有）放行", "tooling-writer", f"cp -r {service_tree}/src prune-server/src/", 0),
            ("工具员 rsync -a 草稿/src 进 prune-server/src/（落点是 prune-server/src/src，服务里没有）放行", "tooling-writer", f"rsync -a {service_tree}/src prune-server/src/", 0),
            ("工具员 cp -r 草稿/fresh/. 进 prune-server/src/（里面只有服务里没有的新文件）放行", "tooling-writer", f"cp -r {service_tree}/fresh/. prune-server/src/", 0),
            ("工具员 cp -rn 草稿/src/. 进 prune-server/src/（不盖已有的）放行", "tooling-writer", f"cp -rn {service_tree}/src/. prune-server/src/", 0),
            ("工具员 cp -r 草稿/src/. 进仓副本里的 prune-server/src/ 放行", "tooling-writer", f"cp -r {service_tree}/src/. {service_copy}/prune-server/src/", 0),
        ]
        # ln 与硬链接：把已有的源码换成链接、给已有的源码建硬链接（往硬链接写就是往源码写）、往已经是服务源码硬链接的文件追加；
        # 弄坏开关 service-source-link-unseen 下 ln 与 cp -l 的四格红，service-source-hardlink-inode-unseen 下往硬链接追加那一格红，service-source-overwrite-only 下追加那一格也红
        service_source_cases += [
            ("工具员 ln -sf 把已存在的 prune-server/src/protocol.rs 换成指向草稿的链接", "tooling-writer", f"ln -sf {service_draft} prune-server/src/protocol.rs", 1),
            ("工具员 ln 给 prune-server/src/bin/serve.rs 建硬链接（源是服务源码）", "tooling-writer", f"ln prune-server/src/bin/serve.rs {new_root}/tooling-x/serve-hard.rs", 1),
            ("工具员 cp -l 给 prune-server/src/bin/serve.rs 建硬链接", "tooling-writer", f"cp -l prune-server/src/bin/serve.rs {new_root}/tooling-x/", 1),
            ("工具员 cp -al 给整个 prune-server/src/ 建硬链接树", "tooling-writer", f"cp -al prune-server/src {new_root}/tooling-x/src-linked", 1),
            ("工具员往与服务源码是同一个文件的硬链接追加", "tooling-writer", f"printf 'x' >> {service_hard_link}", 1),
            ("工具员 ln -s 在 prune-server/src/ 里新建符号链接放行", "tooling-writer", f"ln -s {service_draft} prune-server/src/new_link.rs", 0),
            ("工具员 ln -s 建指向 prune-server/src/bin/serve.rs 的符号链接放行", "tooling-writer", f"ln -s prune-server/src/bin/serve.rs {new_root}/tooling-x/serve-link.rs", 0),
            ("工具员 ln 给不是服务源码的文件建硬链接放行", "tooling-writer", f"ln scripts/gate/tool.py {new_root}/tooling-x/tool.py", 0),
            ("工具员往与服务源码无关、有两个名字的文件追加放行", "tooling-writer", f"printf 'x' >> {service_unrelated_link}", 0),
        ]
        # 解包与打补丁（包里、补丁里写了什么不读，按往哪个目录里写判）：弄坏开关 service-source-extract-unseen 下拒的八格红
        service_source_cases += [
            ("工具员 tar -xf 解进 prune-server", "tooling-writer", f"tar -xf {service_draft}.tar -C prune-server", 1),
            ("工具员 tar xzf（不带横线）解进 prune-server/src", "tooling-writer", f"tar xzf {service_draft}.tgz -C prune-server/src", 1),
            ("工具员 tar --extract --directory= 解进 gpu-verification-server", "tooling-writer", f"tar --extract --file {service_draft}.tar --directory=gpu-verification-server", 1),
            ("工具员 cd 进 prune-server/src 之后 tar -xf", "tooling-writer", f"cd prune-server/src && tar -xf {service_draft}.tar", 1),
            ("工具员 tar -xf 不带 -C、当前目录是仓根", "tooling-writer", f"tar -xf {service_draft}.tar", 1),
            ("工具员 patch -p1 在仓根打补丁", "tooling-writer", f"patch -p1 -i {service_draft}.patch", 1),
            ("工具员 patch -d prune-server 打补丁", "tooling-writer", f"patch -d prune-server -p1 -i {service_draft}.patch", 1),
            ("工具员 git apply 在仓根打补丁", "tooling-writer", f"git apply {service_draft}.patch", 1),
            ("工具员 tar -xf 解进草稿目录放行", "tooling-writer", f"tar -xf {service_draft}.tar -C {new_root}/tooling-x", 0),
            ("工具员 tar -tf 只列不解放行", "tooling-writer", f"tar -tf {service_draft}.tar", 0),
            ("工具员 tar -czf 打包 prune-server/src 放行", "tooling-writer", f"tar -czf {new_root}/tooling-x/src.tgz prune-server/src", 0),
            ("工具员 patch --dry-run 放行", "tooling-writer", f"patch --dry-run -p1 -i {service_draft}.patch", 0),
            ("工具员 patch -d 草稿目录放行", "tooling-writer", f"patch -d {new_root}/tooling-x -p1 -i {service_draft}.patch", 0),
            ("工具员 git apply --check 放行", "tooling-writer", f"git apply --check {service_draft}.patch", 0),
            ("工具员 git apply --cached 放行", "tooling-writer", f"git apply --cached {service_draft}.patch", 0),
            ("工具员 git -C 仓副本 apply 放行", "tooling-writer", f"git -C {service_copy} apply {service_draft}.patch", 0),
            ("主 agent 同样的 tar -xf 放行", None, f"tar -xf {service_draft}.tar -C prune-server", 0),
        ]
        # rm 与 mv 挪走已有的服务源码（之后同名新建，第十一道与这一道都当新文件放）：弄坏开关 service-source-removal-unseen 下拒的六格与真实入口里 rm 那一格（「stdin:工具员 rm 已存在的两个服务 src/ 下的文件拒绝」）红
        service_source_cases += [
            ("工具员 rm 已存在的 prune-server/src/protocol.rs", "tooling-writer", "rm prune-server/src/protocol.rs", 1),
            ("工具员 rm -f 已存在的 gpu-verification-server/Cargo.toml", "tooling-writer", "rm -f gpu-verification-server/Cargo.toml", 1),
            ("工具员 rm -r 整个 prune-server/src", "tooling-writer", "rm -r prune-server/src", 1),
            ("工具员 rm 通配 prune-server/src/*.rs", "tooling-writer", "rm prune-server/src/*.rs", 1),
            ("工具员 mv 挪走已存在的 prune-server/src/protocol.rs", "tooling-writer", f"mv prune-server/src/protocol.rs {new_root}/tooling-x/", 1),
            ("工具员 mv 挪走整个 prune-server/src", "tooling-writer", f"mv prune-server/src {new_root}/tooling-x/src-old", 1),
            ("工具员 rm 不存在的 prune-server/src/not_there.rs 放行", "tooling-writer", "rm prune-server/src/not_there.rs", 0),
            ("工具员 rm -r prune-server/target 放行（里面没有服务源码）", "tooling-writer", "rm -r prune-server/target", 0),
            ("工具员 rm 草稿目录里的文件放行", "tooling-writer", f"rm {new_root}/tooling-x/notes.txt", 0),
            ("工具员 mv 草稿目录里的文件放行", "tooling-writer", f"mv {new_root}/tooling-x/a.rs {new_root}/tooling-x/b.rs", 0),
            ("主 agent rm 已存在的 prune-server/src/protocol.rs 放行", None, "rm prune-server/src/protocol.rs", 0),
        ]
        # unlink、find -delete、find -exec 里删找到的路径：弄坏开关 service-source-unlink-find-unseen 下拒的三格红
        service_source_cases += [
            ("工具员 unlink 已存在的 prune-server/src/protocol.rs", "tooling-writer", "unlink prune-server/src/protocol.rs", 1),
            ("工具员 find -delete 删 prune-server/src 下已存在的文件", "tooling-writer", "find prune-server/src -name protocol.rs -delete", 1),
            ("工具员 find -exec rm 删 prune-server/src 下已存在的文件", "tooling-writer", "find prune-server/src -name protocol.rs -exec rm {} +", 1),
            ("工具员 unlink 不存在的 prune-server/src/not_there.rs 放行", "tooling-writer", "unlink prune-server/src/not_there.rs", 0),
            ("工具员 find 只列 prune-server/src 放行", "tooling-writer", "find prune-server/src -name protocol.rs", 0),
            ("工具员 find -delete 草稿目录放行", "tooling-writer", f"find {new_root}/tooling-x -name notes.txt -delete", 0),
            ("工具员 find -exec grep 读 prune-server/src 放行", "tooling-writer", "find prune-server/src -name protocol.rs -exec grep -l pub {} +", 0),
            ("工具员 find -exec cp 把 prune-server/src 下的文件拷出去放行", "tooling-writer", f"find prune-server/src -name protocol.rs -exec cp {{}} {new_root}/tooling-x/ ';'", 0),
        ]
        # rsync --delete（目的地里源里没有的文件被删）与 --remove-source-files（拷完删源）：弄坏开关 service-source-rsync-removal-unseen 下拒的两格红
        service_source_cases += [
            ("工具员 rsync -a --delete 空目录 进 prune-server/src/", "tooling-writer", f"rsync -a --delete {service_tree}/empty/ prune-server/src/", 1),
            ("工具员 rsync --remove-source-files 把 prune-server/src/protocol.rs 拷出去", "tooling-writer",
             f"rsync -a --remove-source-files prune-server/src/protocol.rs {new_root}/tooling-x/", 1),
            ("工具员 rsync -a --delete 进草稿目录放行", "tooling-writer", f"rsync -a --delete {service_tree}/src/ {new_root}/tooling-x/", 0),
            ("工具员 rsync -an --delete 只演练放行", "tooling-writer", f"rsync -an --delete {service_tree}/empty/ prune-server/src/", 0),
            ("工具员 rsync --delay-updates 不是 --delete 放行", "tooling-writer", f"rsync -a --delay-updates {service_tree}/empty/ prune-server/src/", 0),
        ]
        # gzip 一类把原文件换成压缩或解压后的文件：弄坏开关 service-source-compressor-unseen 下拒的三格红
        service_source_cases += [
            ("工具员 gzip 把 prune-server/src/protocol.rs 换成压缩文件", "tooling-writer", "gzip prune-server/src/protocol.rs", 1),
            ("工具员 bzip2 -f 把 prune-server/src/bin/serve.rs 换成压缩文件", "tooling-writer", "bzip2 -f prune-server/src/bin/serve.rs", 1),
            ("工具员 zstd --rm 压完删 prune-server/src/protocol.rs", "tooling-writer", "zstd --rm prune-server/src/protocol.rs", 1),
            ("工具员 gzip -k 留着原文件放行", "tooling-writer", "gzip -k prune-server/src/protocol.rs", 0),
            ("工具员 gzip -c 写标准输出放行", "tooling-writer", "gzip -c prune-server/src/protocol.rs", 0),
            ("工具员 zstd 默认留着原文件放行", "tooling-writer", "zstd prune-server/src/protocol.rs", 0),
            ("工具员 gzip 草稿目录里的文件放行", "tooling-writer", f"gzip {new_root}/tooling-x/notes.txt", 0),
        ]
        # shred、sort -o、gawk -i inplace 在原 inode 上整份改写：弄坏开关 service-source-in-place-tools-unseen 下拒的三格红
        service_source_cases += [
            ("工具员 shred 改写 prune-server/src/protocol.rs", "tooling-writer", "shred -n 1 prune-server/src/protocol.rs", 1),
            ("工具员 sort -o 排完写回 prune-server/src/protocol.rs", "tooling-writer", "sort -o prune-server/src/protocol.rs prune-server/src/protocol.rs", 1),
            ("工具员 gawk -i inplace 就地改 prune-server/src/protocol.rs", "tooling-writer", "gawk -i inplace '{print}' prune-server/src/protocol.rs", 1),
            ("工具员 sort 只读 prune-server/src/protocol.rs 放行", "tooling-writer", "sort prune-server/src/protocol.rs", 0),
            ("工具员 sort -o 写草稿目录放行", "tooling-writer", f"sort -o {new_root}/tooling-x/sorted.txt prune-server/src/protocol.rs", 0),
            ("工具员 gawk 不带 -i inplace 放行", "tooling-writer", "gawk '{print}' prune-server/src/protocol.rs", 0),
            ("工具员 gawk -i inplace 草稿目录里的文件放行", "tooling-writer", f"gawk -i inplace '{{print}}' {new_root}/tooling-x/notes.txt", 0),
        ]
        # `<>` 读写重定向（不截断、在原 inode 上从头改写）：弄坏开关 service-source-read-write-redirect-unseen 下拒的一格红
        service_source_cases += [
            ("工具员 1<> 在原 inode 上改写 prune-server/src/protocol.rs", "tooling-writer", "printf 'x' 1<> prune-server/src/protocol.rs", 1),
            ("工具员 1<> 写草稿目录里的文件放行", "tooling-writer", f"printf 'x' 1<> {new_root}/tooling-x/notes.txt", 0),
            ("工具员 < 只读 prune-server/src/protocol.rs 放行", "tooling-writer", "cat < prune-server/src/protocol.rs", 0),
        ]
        # unzip 解进服务源码目录或包着它的上级（同 tar -x 按往哪个目录里写判）：弄坏开关 service-source-unzip-unseen 下拒的两格红
        service_source_cases += [
            ("工具员 unzip -d 解进 prune-server/src", "tooling-writer", f"unzip -o {service_draft}.zip -d prune-server/src", 1),
            ("工具员 unzip 不带 -d、当前目录是仓根", "tooling-writer", f"unzip -o {service_draft}.zip", 1),
            ("工具员 unzip -d 解进草稿目录放行", "tooling-writer", f"unzip -o {service_draft}.zip -d {new_root}/tooling-x", 0),
            ("工具员 unzip -l 只列放行", "tooling-writer", f"unzip -l {service_draft}.zip", 0),
            ("工具员 unzip -p 解到标准输出放行", "tooling-writer", f"unzip -p {service_draft}.zip protocol.rs", 0),
        ]
        # python 的删除与挪走（os.remove、os.unlink、Path.unlink、shutil.rmtree、os.rename 与 shutil.move 的源）：弄坏开关 service-source-python-delete-unseen 下拒的四格红
        service_source_cases += [
            ("工具员 python os.remove 删 prune-server/src/protocol.rs", "tooling-writer", "python3 -c \"import os; os.remove('prune-server/src/protocol.rs')\"", 1),
            ("工具员 python os.rename 把 prune-server/src/protocol.rs 挪走", "tooling-writer",
             f"python3 -c \"import os; os.rename('prune-server/src/protocol.rs', '{new_root}/tooling-x/moved.rs')\"", 1),
            ("工具员 python shutil.rmtree 删整个 prune-server/src", "tooling-writer", "python3 -c \"import shutil; shutil.rmtree('prune-server/src')\"", 1),
            ("工具员 python Path.unlink 删 prune-server/src/protocol.rs", "tooling-writer",
             "python3 -c \"from pathlib import Path; Path('prune-server/src/protocol.rs').unlink()\"", 1),
            ("工具员 python os.remove 草稿目录里的文件放行", "tooling-writer", f"python3 -c \"import os; os.remove('{new_root}/tooling-x/notes.txt')\"", 0),
            ("工具员 python os.rename 草稿目录里的文件放行", "tooling-writer",
             f"python3 -c \"import os; os.rename('{new_root}/tooling-x/a.rs', '{new_root}/tooling-x/b.rs')\"", 0),
        ]
        # python 带写标志的 os.open 与 shutil.copytree(…, dirs_exist_ok=True) 往已有目录逐份盖：弄坏开关 service-source-python-in-place-unseen 下拒的两格红
        service_source_cases += [
            ("工具员 python os.open 带 O_WRONLY 改 prune-server/src/protocol.rs", "tooling-writer",
             "python3 -c \"import os; f = os.open('prune-server/src/protocol.rs', os.O_WRONLY); os.write(f, b'x')\"", 1),
            ("工具员 python shutil.copytree(dirs_exist_ok=True) 逐份盖 prune-server/src", "tooling-writer",
             f"python3 -c \"import shutil; shutil.copytree('{service_tree}/src', 'prune-server/src', dirs_exist_ok=True)\"", 1),
            ("工具员 python os.open 只读放行", "tooling-writer", "python3 -c \"import os; os.open('prune-server/src/protocol.rs', os.O_RDONLY)\"", 0),
            ("工具员 python shutil.copytree 不带 dirs_exist_ok 放行", "tooling-writer",
             f"python3 -c \"import shutil; shutil.copytree('{service_tree}/src', 'prune-server/src')\"", 0),
            ("工具员 python shutil.copytree(dirs_exist_ok=True) 里只有服务里没有的新文件放行", "tooling-writer",
             f"python3 -c \"import shutil; shutil.copytree('{service_tree}/fresh', 'prune-server/src', dirs_exist_ok=True)\"", 0),
        ]
        # 解到标准输出的 tar（-O、--to-stdout）与带位置参数原文件的 patch 只写点名的那一份：弄坏开关 service-source-stdout-extract-judged 下前三格、
        # service-source-patch-operand-ignored 下紧跟着的头两格红（按目录判成了往仓根写；第三格 -d 给的就是草稿目录，照目录判也放行）；
        # patch 的原文件就是服务源码的拒，service-source-extract-unseen 下那一格红
        service_source_cases += [
            ("工具员 tar -xOf 解到标准输出放行", "tooling-writer", f"tar -xOf {service_draft}.tar protocol.rs", 0),
            ("工具员 tar xOf（不带横线）解到标准输出放行", "tooling-writer", f"tar xOf {service_draft}.tar protocol.rs", 0),
            ("工具员 tar --to-stdout 解到标准输出放行", "tooling-writer", f"tar -xf {service_draft}.tar --to-stdout protocol.rs", 0),
            ("工具员 patch 原文件 补丁、原文件在草稿目录放行", "tooling-writer", f"patch {new_root}/tooling-x/write-guard.sh {service_draft}.patch", 0),
            ("工具员 patch -i 补丁 原文件、原文件在草稿目录放行", "tooling-writer", f"patch -i {service_draft}.patch {new_root}/tooling-x/write-guard.sh", 0),
            ("工具员 patch -d 草稿目录 -i 补丁 原文件放行", "tooling-writer", f"patch -d {new_root}/tooling-x -i {service_draft}.patch write-guard.sh", 0),
            ("工具员 patch 原文件是已存在的 prune-server/src/protocol.rs", "tooling-writer", f"patch prune-server/src/protocol.rs {service_draft}.patch", 1),
        ]
        # make-swap-spec.py 的 --out（⑫ 与 ⑭ 同一份认法）：弄坏开关 scope-swap-spec-out-unseen 下红
        service_source_cases += [
            ("工具员 make-swap-spec.py 的 --out 写已存在的 prune-server/src/protocol.rs", "tooling-writer",
             "python3 scripts/gate/make-swap-spec.py --base a --new b --out prune-server/src/protocol.rs scripts/tool.py", 1),
            ("工具员 make-swap-spec.py 的 --out 写草稿目录放行", "tooling-writer",
             f"python3 scripts/gate/make-swap-spec.py --base a --new b --out {new_root}/tooling-x/spec.json scripts/tool.py", 0),
        ]
        # 已有门禁的样本档目录（判法与写闸第十一道那一半同一份，lib_write_scope.existing_gate_of_fixture）：临时仓里放一道门禁 stage.sh 与它的一份样本，
        # 另有一个还没有门禁阶段的样本目录；弄坏开关 gate-fixtures-unseen 与 allow-service-source-write 下拒的八格红，service-source-removal-unseen 下 rm 两格红，
        # service-source-extract-unseen 下 git apply 那一格红
        for fixture_file, fixture_text in ((".claude/gate.d/stage.sh", "#!/usr/bin/env bash\n"), (".claude/gate.d/fixtures/stage.sh/stage-red/expect", "exit=1\n"),
                                           (".claude/gate.d/fixtures/new-stage.sh/new-red/expect", "exit=1\n")):
            os.makedirs(os.path.dirname(os.path.join(edit_repository, fixture_file)), exist_ok=True)
            with open(os.path.join(edit_repository, fixture_file), "w", encoding="utf-8") as handle:
                handle.write(fixture_text)
        service_source_cases += [
            ("工具员 cp 盖已有门禁的样本", "tooling-writer", f"cp {service_draft} .claude/gate.d/fixtures/stage.sh/stage-red/expect", 1),
            ("工具员 > 新建已有门禁样本档目录里的文件", "tooling-writer", "printf 'exit=0\\n' > .claude/gate.d/fixtures/stage.sh/stage-new-red/expect", 1),
            ("工具员 tee 写已有门禁的样本", "tooling-writer", "echo 'exit=0' | tee .claude/gate.d/fixtures/stage.sh/stage-red/expect", 1),
            ("工具员 mv 草稿里的样本进已有门禁的样本档目录", "tooling-writer", f"mv {service_draft} .claude/gate.d/fixtures/stage.sh/", 1),
            ("工具员 cp -r 草稿样本目录进已有门禁的样本档目录", "tooling-writer", f"cp -r {service_tree}/src .claude/gate.d/fixtures/stage.sh/", 1),
            ("工具员 git apply 在样本档根里打补丁", "tooling-writer", f"cd .claude/gate.d/fixtures && git apply -p2 {service_draft}.patch", 1),
            ("工具员 rm 已有门禁的样本", "tooling-writer", "rm .claude/gate.d/fixtures/stage.sh/stage-red/expect", 1),
            ("工具员 rm -r 整个已有门禁的样本档目录", "tooling-writer", "rm -r .claude/gate.d/fixtures/stage.sh", 1),
            ("工具员写还没有门禁阶段的样本目录放行", "tooling-writer", "printf 'exit=0\\n' > .claude/gate.d/fixtures/new-stage.sh/new-red/expect", 0),
            ("工具员写样本档根里直接放的文件放行", "tooling-writer", "printf 'x\\n' > .claude/gate.d/fixtures/notes.txt", 0),
            ("工具员写草稿目录里的 fixtures-new 放行", "tooling-writer",
             f"mkdir -p {new_root}/tooling-x/fixtures-new && cp {service_draft} {new_root}/tooling-x/fixtures-new/expect", 0),
            ("工具员 git apply --check 在仓根核样本补丁放行", "tooling-writer", f"git apply -p2 --check {service_draft}.patch", 0),
            ("主 agent 写已有门禁的样本放行", None, f"cp {service_draft} .claude/gate.d/fixtures/stage.sh/stage-red/expect", 0),
        ]
        for label, agent, command, want in service_source_cases:
            results.append((f"服务源码:{label}", want,
                            1 if service_source_refusal(command, agent, edit_repository, service_copy if "cwd 在仓副本里" in label else None) else 0))
        # scripts/hosts/two-logs.sh <名> -- <命令> 包着的命令照原判法判（wrapped_commands 剥它，认法在 lib_project_code_runs.py 的 command_under_two_logs）：(说明, 判法, 命令, 该不该拒)
        two_logs_cases = [
            ("包着的前台没超时的等待循环拒", "等待循环", "bash scripts/hosts/two-logs.sh /tmp/n -- bash -c 'until grep -q x f; do sleep 10; done'", 1),
            ("--quiet 包着的前台没超时的等待循环拒", "等待循环", "bash scripts/hosts/two-logs.sh --quiet /tmp/n -- bash -c 'until grep -q x f; do sleep 10; done'", 1),
            ("包着的 kill 负进程号拒", "终止进程", "bash scripts/hosts/two-logs.sh /tmp/n -- kill -TERM -1234", 1),
            ("包着的 setsid -f 拒", "放出追踪", "bash scripts/hosts/two-logs.sh /tmp/n -- setsid -f bash long.sh", 1),
            ("包着的 cp 盖主工作区入库装置拒（先编后换）", "先编后换", f"bash scripts/hosts/two-logs.sh /tmp/n -- cp {draft_path} {bin_path}", 1),
            ("包着的普通命令放行（等待循环）", "等待循环", "bash scripts/hosts/two-logs.sh /tmp/n -- bash gate.sh", 0),
            ("包着的 timeout 等待循环放行", "等待循环", "bash scripts/hosts/two-logs.sh /tmp/n -- timeout 60 bash -c 'until grep -q x f; do sleep 10; done'", 0),
            ("--from-log 不跑命令放行（终止进程）", "终止进程", "bash scripts/hosts/two-logs.sh --from-log /tmp/n", 0),
            ("别处的同名拷贝不剥（终止进程）", "终止进程", "bash /tmp/elsewhere/two-logs.sh /tmp/n -- kill -TERM -1234", 0),
        ]
        two_logs_judges = {
            "等待循环": lambda command: wait_loop_refusal(command, False),
            "终止进程": process_signal_refusal,
            "放出追踪": detaching_refusal,
            "先编后换": lambda command: compile_first_refusal(command, "experiment-runner", edit_repository),
        }
        for label, judged, command, want in two_logs_cases:
            results.append((f"两道包装:{label}", want, 1 if two_logs_judges[judged](command) else 0))
    finally:
        shutil.rmtree(edit_repository, ignore_errors=True)
    # ⑫ 有写范围的项目子 agent 在 Bash 里写写范围之外的仓内文件：临时仓里放 clerk、kb-scribe、reporter（表里没有行）的定义和一张写范围表，
    #   同目录放一份拷贝出来的 hook 与它读的库，真实入口那几格（stdin:）也在这个仓里起；(说明, agent 类型, 命令, 该不该拒)
    scope_repository = os.path.realpath(os.path.join(work, "scope-repository"))
    scope_hooks = os.path.join(scope_repository, ".claude", "hooks")
    os.makedirs(scope_hooks)
    os.makedirs(os.path.join(scope_repository, ".claude", "agents"))
    for library_name in (("bash-command-detector.sh",) + DETECTOR_PART_NAMES + ("lib_shell_words.py",) + SHELL_WORDS_PART_NAMES
                         + ("lib_project_code_runs.py", "lib_project_code_runs_bootstrap.py", "lib_project_code_runs_base.py", "lib_project_code_runs_launchers.py", "lib_project_code_runs_selftest.py", "lib_write_scope.py", "lib_work_root.py", "lib_writer_lock.py", "lib_three_way_legs.py")):
        shutil.copy(os.path.join(hook_dir, library_name), scope_hooks)
    for agent_name in ("clerk", "kb-scribe", "reporter"):
        open(os.path.join(scope_repository, ".claude", "agents", f"{agent_name}.md"), "w").write("---\ntools: Read, Edit, Write, Bash\n---\n")
    scope_table = os.path.join(scope_hooks, WRITE_SCOPE_TABLE_NAME)
    open(scope_table, "w").write("clerk\trecords/**\t样本\nclerk\tresearch/prompts/**\t样本\nclerk\t$SINGLEFS_WORK_ROOT/**\t样本\n"
                                 "kb-scribe\t.claude/kb/**\t样本\nkb-scribe\t$SINGLEFS_WORK_ROOT/**\t样本\n")
    open(os.path.join(scope_repository, "spec-out.json"), "w").write(json.dumps([{"file": "records/x.md", "old": "a", "new": "b"}, {"file": ".claude/kb/x.md", "old": "a", "new": "b"}]))
    open(os.path.join(scope_repository, "spec-in.json"), "w").write(json.dumps([{"file": "records/x.md", "old": "a", "new": "b"}, {"file": "research/prompts/y.md", "old": "a", "new": "b"}]))
    scope_cases = [
        ("clerk 用 replace-once.py 写知识库（现场：越出写范围）", "clerk", "python3 scripts/kb/replace-once.py .claude/kb/checks-owed.md a b", 1),
        ("clerk 直接执行 replace-once.py 写知识库", "clerk", "scripts/kb/replace-once.py .claude/kb/checks-owed.md a b", 1),
        ("clerk 用 replace-once.py 写自己写范围里的 records", "clerk", "python3 scripts/kb/replace-once.py records/x.md a b", 0),
        ("clerk 用 replace-once.py 写工作根下自己的草稿目录", "clerk", f"python3 scripts/kb/replace-once.py {new_root}/clerk-x/notes.md a b", 0),
        ("clerk 先 cd 到 records 再用相对路径写知识库", "clerk", "cd records && python3 ../scripts/kb/replace-once.py ../.claude/kb/x.md a b", 1),
        ("kb-scribe 用 replace-once.py 写知识库放行", "kb-scribe", "python3 scripts/kb/replace-once.py .claude/kb/x.md a b", 0),
        ("kb-scribe 用 replace-once.py 写 records 拒", "kb-scribe", "python3 scripts/kb/replace-once.py records/x.md a b", 1),
        ("clerk 用 insert-row.py 写知识库", "clerk", "python3 scripts/kb/insert-row.py .claude/kb/checks-owed.md --after '^# a' --line 'x'", 1),
        ("clerk 的 insert-row.py --line 值里的路径不当目标", "clerk", "python3 scripts/kb/insert-row.py records/x.md --after '^# a' --line '.claude/kb/y.md'", 0),
        ("clerk 的 insert-row.py --dry-run 不写", "clerk", "python3 scripts/kb/insert-row.py .claude/kb/checks-owed.md --after a --line x --dry-run", 0),
        ("clerk 的 insert-row.py --absent-literal 写在文件前面，文件照认、写自己写范围里的 records 放行", "clerk", "python3 scripts/kb/insert-row.py --absent-literal row-a records/x.md --after '^# a' --line row-a", 0),
        ("clerk 的 replace-once.py --selftest 不写", "clerk", "python3 scripts/kb/replace-once.py --selftest", 0),
        ("clerk 用 make-swap-spec.py 的 --out 写知识库", "clerk", "python3 scripts/gate/make-swap-spec.py --base a --new b --out .claude/kb/x.json scripts/tool.py", 1),
        ("clerk 用 make-swap-spec.py 的 --out 写工作根下自己的草稿目录放行", "clerk",
         f"python3 scripts/gate/make-swap-spec.py --base a --new b --out {new_root}/clerk-x/spec.json scripts/tool.py", 0),
        ("clerk 用 replace-batch.py，规格点名知识库", "clerk", "python3 scripts/kb/replace-batch.py spec-out.json", 1),
        ("clerk 用 replace-batch.py，规格点名的都在写范围里", "clerk", "python3 scripts/kb/replace-batch.py spec-in.json", 0),
        ("clerk 的 replace-batch.py --dry-run 不写", "clerk", "python3 scripts/kb/replace-batch.py --dry-run spec-out.json", 0),
        # 脚本路径放在前面赋过值的变量里：代入之后照认（弄坏开关 scope-script-variable-unexpanded 下前四格红）
        ("clerk 脚本路径在变量里，用 replace-once.py 写知识库", "clerk", "S=scripts/kb/replace-once.py; python3 $S .claude/kb/x.md a b", 1),
        ("clerk 脚本路径在变量里，用 insert-row.py 写知识库", "clerk", "W=scripts/kb/insert-row.py; python3 \"$W\" .claude/kb/checks-owed.md --after a --line x", 1),
        ("clerk 脚本路径在变量里，用 replace-batch.py、规格点名知识库", "clerk", "B=scripts/kb/replace-batch.py; python3 ${B} spec-out.json", 1),
        ("clerk 直接执行变量里的 replace-once.py 写知识库", "clerk", "S=scripts/kb/replace-once.py; $S .claude/kb/x.md a b", 1),
        ("clerk 脚本路径在变量里，写自己写范围里的 records 放行", "clerk", "S=scripts/kb/replace-once.py; python3 $S records/x.md a b", 0),
        ("clerk 脚本路径在变量里、带 --dry-run 不写", "clerk", "S=scripts/kb/replace-once.py; python3 $S .claude/kb/x.md a b --dry-run", 0),
        ("clerk sed -i 改 crates", "clerk", "sed -i 's/a/b/' crates/x/src/lib.rs", 1),
        ("clerk sed -i 改 records 放行", "clerk", "sed -i 's/a/b/' records/x.md", 0),
        ("clerk 重定向 > 写 crates", "clerk", "echo x > crates/x/src/lib.rs", 1),
        ("clerk 重定向 >> 写知识库", "clerk", "echo x >> .claude/kb/a.md", 1),
        ("clerk tee 写 crates", "clerk", "echo x | tee crates/x/src/lib.rs", 1),
        ("clerk tee -a 写知识库", "clerk", "echo x | tee -a .claude/kb/a.md", 1),
        ("clerk cp 到知识库", "clerk", "cp records/a.md .claude/kb/a.md", 1),
        ("clerk cp 进知识库目录", "clerk", "cp records/a.md .claude/kb/", 1),
        ("clerk mv 到 crates", "clerk", "mv records/a.md crates/a.md", 1),
        ("clerk cp 到 records 放行", "clerk", "cp /tmp/a.md records/a.md", 0),
        ("clerk heredoc 写 records 放行", "clerk", "cat > records/ok.md <<'EOF'\nx\nEOF", 0),
        ("clerk heredoc 写知识库", "clerk", "cat > .claude/kb/ok.md <<'EOF'\nx\nEOF", 1),
        ("clerk 先 cd 进知识库目录再重定向", "clerk", "cd .claude/kb && echo x > a.md", 1),
        ("clerk bash -c 里的写", "clerk", "bash -c 'echo x > .claude/kb/a.md'", 1),
        ("clerk 变量里的目录（前面赋过值）", "clerk", "d=.claude/kb; echo x > $d/a.md", 1),
        ("clerk python open(…, 'w') 写知识库", "clerk", "python3 -c \"open('.claude/kb/x.md', 'w').write('x')\"", 1),
        ("clerk 写 /dev/null、2>&1 与 /tmp 放行", "clerk", "ls > /dev/null 2>&1; echo x > /tmp/y.out", 0),
        ("clerk 写工作根下的草稿目录放行", "clerk", f"echo x > {new_root}/clerk-x/a.md", 0),
        # heredoc 正文只在喂给 shell（bash、sh）或 python 时才是代码；喂给 cat、tee 的是数据，但它们的目标照判（弄坏开关 heredoc-body-judged-as-commands 下红）
        ("clerk 往草稿脚本 cat 写的 heredoc 正文里有 sed -i 知识库：正文是数据，放行", "clerk",
         f"cat > {new_root}/clerk-x/x.sh <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
        ("clerk 往草稿脚本 cat 写的 heredoc 正文里有重定向和 cp 到知识库：放行", "clerk",
         f"cat > {new_root}/clerk-x/x.sh <<'EOF'\necho x > .claude/kb/a.md\ncp a.md .claude/kb/b.md\nEOF", 0),
        ("clerk 往草稿脚本 tee 写的 heredoc 正文里有 sed -i 知识库：放行", "clerk",
         f"tee {new_root}/clerk-x/x.sh <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
        ("clerk 喂给 bash 的 heredoc 里同样一句 sed -i 知识库照拒", "clerk", "bash <<'EOF'\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 1),
        ("clerk 喂给 bash 的 heredoc 里的重定向写知识库照拒", "clerk", "bash <<'EOF'\necho x > .claude/kb/a.md\nEOF", 1),
        ("clerk 喂给 python 的 heredoc 里 open(知识库, 'w') 照拒", "clerk", "python3 - <<'EOF'\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 喂给 python 的 heredoc 里写自己写范围里的 records 放行", "clerk", "python3 - <<'EOF'\nopen('records/a.md', 'w').write('x')\nEOF", 0),
        # python 的 for 逐个取字面元组：按每个候选判，有一个落在写范围外就拒（弄坏开关 python-loop-variable-unexpanded 下红）
        ("clerk 喂给 python 的 heredoc 里 for 逐个取元组，候选里有一个落在写范围外（知识库）照拒", "clerk",
         "python3 - <<'EOF'\nfor f in ('records/a.md', '.claude/kb/a.md'):\n    open(f, 'w').write('x')\nEOF", 1),
        ("clerk cat > 知识库 <<EOF 的目标本身照拒（正文是数据，目标不是）", "clerk", "cat > .claude/kb/a.md <<'EOF'\nx\nEOF", 1),
        ("clerk tee 知识库 <<EOF 的目标本身照拒", "clerk", "tee .claude/kb/a.md <<'EOF'\nx\nEOF", 1),
        ("clerk 只读知识库放行", "clerk", "cat .claude/kb/a.md | grep x", 0),
        ("clerk 引号里当数据写的放行", "clerk", "echo 'echo x > .claude/kb/a.md'", 0),
        ("clerk 经 two-logs.sh 包着写知识库", "clerk", "bash scripts/hosts/two-logs.sh /tmp/n -- tee .claude/kb/a.md", 1),
        ("clerk 的目标在没赋过值的变量里：拒", "clerk", "python3 scripts/kb/replace-once.py \"$TARGET\" a b", 1),
        ("clerk 重定向目标带通配：拒", "clerk", "echo x > .claude/kb/*.md", 1),
        ("clerk cd 到没赋过值的变量目录之后写相对路径：拒", "clerk", "cd \"$WHERE\" && echo x > a.md", 1),
        ("clerk cd 到没赋过值的变量目录之后写绝对路径的知识库：照拒", "clerk", f"cd \"$WHERE\" && echo x > {scope_repository}/.claude/kb/a.md", 1),
        # 经管道喂给 python、bash 的 heredoc 正文是代码，喂给 grep、python 脚本文件的是数据（弄坏开关 heredoc-pipe-fed-as-data 下带「照拒」的红）
        ("clerk 管道喂给 python3 - 的 heredoc 里 open(知识库, 'w') 照拒", "clerk", "cat <<'EOF' | python3 -\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 管道喂给 python3（不带 -）的 heredoc 里 open(知识库, 'w') 照拒", "clerk", "cat <<'EOF' | python3\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 管道喂给 python3 - 的 heredoc 里写自己写范围里的 records 放行", "clerk", "cat <<'EOF' | python3 -\nopen('records/a.md', 'w').write('x')\nEOF", 0),
        ("clerk cd 到 records 之后管道喂给 python3 - 的 heredoc 里相对路径写知识库照拒", "clerk",
         "cd records && cat <<'EOF' | python3 -\nopen('../.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 管道喂给 bash 的 heredoc 里 sed -i 知识库照拒", "clerk", "cat <<'EOF' | bash\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 1),
        ("clerk 管道喂给 bash -s 的 heredoc 里重定向写知识库照拒", "clerk", "cat <<'EOF' | bash -s\necho x > .claude/kb/a.md\nEOF", 1),
        ("clerk echo 的文字管道喂给 python3 - 写知识库照拒", "clerk", "echo \"open('.claude/kb/a.md', 'w').write('x')\" | python3 -", 1),
        ("clerk printf 的文字管道喂给 python3 - 写知识库照拒", "clerk", "printf '%s\\n' \"open('.claude/kb/a.md', 'w')\" | python3 -", 1),
        ("clerk 自己的 heredoc 盖过管道：echo | python3 - <<EOF 里写知识库照拒", "clerk", "echo x | python3 - <<'EOF'\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 1),
        ("clerk 管道喂给 grep 的 heredoc 正文是数据，放行", "clerk",
         "cat <<'EOF' | grep x\necho x > .claude/kb/a.md\nopen('.claude/kb/a.md', 'w')\nsed -i 's/a/b/' .claude/kb/a.md\nEOF", 0),
        ("clerk echo 的文字管道喂给 grep 是数据，放行", "clerk", "echo \"open('.claude/kb/a.md', 'w')\" | grep x", 0),
        ("clerk 管道喂给 python3 脚本文件的 heredoc 正文是数据，放行", "clerk", "cat <<'EOF' | python3 x.py\nopen('.claude/kb/a.md', 'w').write('x')\nEOF", 0),
        # cd 到写死的目录之后 $PWD 按那个目录展开（弄坏开关 pwd-variable-unexpanded 下红）；mktemp -d 赋的变量落在 ${TMPDIR:-/tmp} 下（仓外）
        ("clerk cd 到知识库目录之后写 $PWD/… 照拒", "clerk", "cd .claude/kb && echo x > $PWD/a.md", 1),
        ("clerk cd 到知识库目录之后 cp 进 $PWD/… 照拒", "clerk", "cd .claude/kb && cp /tmp/a.md $PWD/b.md", 1),
        ("clerk cd 到 records 之后写 $PWD/… 放行", "clerk", "cd records && echo x > $PWD/a.md", 0),
        ("clerk mktemp -d 建的变量目录里写放行", "clerk", "W=$(mktemp -d) && cp -a records/a.md $W/", 0),
        ("clerk mktemp -d 之后 cd 进去再写相对路径放行", "clerk", "W=$(mktemp -d) && cd \"$W\" && echo x > a.md", 0),
        ("主 agent 写知识库放行", None, "python3 scripts/kb/replace-once.py .claude/kb/x.md a b", 0),
        # 没有项目定义的 agent 只许写工作根（弄坏开关 builtin-agents-unjudged 下「拒」的六格红；会话临时目录根下那两格另在 SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted 下红）
        ("没有项目定义的内置 agent 写仓里拒", "Explore", "echo x > crates/x.rs", 1),
        ("没有项目定义的内置 agent 写仓外别处拒", "general-purpose", "echo x > /var/tmp/singlefs-builtin-elsewhere/a.md", 1),
        ("没有项目定义的内置 agent 写会话临时目录根拒（scratchpad 之外）", "general-purpose", f"echo x > /tmp/claude-{os.getuid()}/singlefs-builtin-x/a.md", 1),
        ("没有项目定义的内置 agent 写会话 scratchpad 拒", "Explore", f"echo x > /tmp/claude-{os.getuid()}/-home-sample/selftest-session/scratchpad/a.md", 1),
        ("没有项目定义的内置 agent 用 replace-once.py 写仓里拒", "general-purpose", "python3 scripts/kb/replace-once.py records/x.md a b", 1),
        ("定义已删的旧实例写仓里拒", "gate-triage", "cp /tmp/a.md records/a.md", 1),
        ("没有项目定义的内置 agent 写工作根放行", "Explore", f"echo x > {new_root}/explore-x/a.md", 0),
        ("没有项目定义的内置 agent 写 /dev/null、2>&1 与只读放行", "Explore", "grep -rn x . > /dev/null 2>&1; cat records/a.md | head -3", 0),
        ("表里没有行的项目定义不判", "reporter", "echo x > crates/x.rs", 0),
        # eval 的代码与启动器里面那条照判，xargs、find -exec 读进来的落点算不出照拒（一百三十七节件 1b 第 4 条；弄坏开关 scope-launcher-writes-unseen 下「拒」的五格与下面工作根那一段的 xargs 一格红）
        ("一百三十七：clerk eval 字面代码里 sed -i 知识库，拒", "clerk", "eval 'sed -i s/a/b/ .claude/kb/a.md'", 1),
        ("一百三十七：clerk find -exec sed -i 读进来的路径，落点算不出，拒", "clerk", "find .claude/kb -name a.md -exec sed -i s/a/b/ {} ';'", 1),
        ("一百三十七：clerk echo 喂给 xargs sed -i，落点算不出，拒", "clerk", "echo .claude/kb/a.md | xargs sed -i s/a/b/", 1),
        ("一百三十七：clerk xargs sed -i 写范围里的 records 也拒（落点算不出）", "clerk", "echo records/a.md | xargs sed -i s/a/b/", 1),
        ("一百三十七：clerk find -exec cp 到写死的知识库路径，拒", "clerk", "find records -name a.md -exec cp {} .claude/kb/b.md ';'", 1),
        ("一百三十七：clerk eval 字面代码写 records 放行", "clerk", "eval 'echo x > records/a.md'", 0),
        ("一百三十七：clerk xargs 起只读的 grep 放行", "clerk", "echo records/a.md | xargs grep -l x", 0),
        ("一百三十七：clerk find -exec cp 到写死的 records 路径放行", "clerk", "find records -name a.md -exec cp {} records/b.md ';'", 0),
    ]
    for label, agent, command, want in scope_cases:
        case_verdict = bash_scope_refusal(command, agent, scope_repository, scope_repository, scope_table)
        results.append((f"Bash写范围:{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
    # ⑫ 只带 Read、Bash 的八类定义（experiment-designer、mutation-triage、prior-art、sweep、three-way-attack、three-way-defense、three-way-forward、three-way-materials）
    #   也在写范围表里有行：读这一份钩子所在仓的真表与真定义，往仓里写它们定义的写范围之外的拒，写范围之内的放行。
    #   弄坏开关 scope-read-only-definitions-unjudged 下 tools 里没有 Write、Edit 的定义的行读成没有，「拒」的八格红；(说明, agent 类型, 命令, 该不该拒)
    read_only_scope_root = repository_root_of(hook_dir)
    read_only_scope_table = os.path.join(hook_dir, WRITE_SCOPE_TABLE_NAME)
    read_only_scope_cases = [
        ("experiment-designer 往实验页的索引写，拒", "experiment-designer", "echo x >> .claude/kb/experiments.md", 1),
        ("experiment-designer 往跑前登记追加，放行", "experiment-designer", "echo x >> research/prompts/e999-preregistration.md", 0),
        ("experiment-designer 写重跑登记，放行", "experiment-designer", "echo x > research/prompts/e999-r2-prereg.md", 0),
        ("mutation-triage 往 crates/mutations.tsv 追加，拒", "mutation-triage", "echo x >> crates/mutations.tsv", 1),
        ("prior-art 用 sed -i 改 prior-art 的 kb 页，拒", "prior-art", "sed -i 's/a/b/' .claude/kb/prior-art.md", 1),
        ("sweep 用 sed -i 改被搜到的 records 文件，拒", "sweep", "sed -i 's/a/b/' records/x.md", 1),
        ("three-way-attack 写 crates，拒", "three-way-attack", "echo x > crates/x.rs", 1),
        ("three-way-attack 写别的轮的背景材料，拒", "three-way-attack", "echo x > research/prompts/x-background.md", 1),
        ("three-way-attack 写报告，放行", "three-way-attack", "echo x > research/prompts/x-opus-output.md", 0),
        ("three-way-attack 往模型目录里拷脚本，放行", "three-way-attack", "cp /tmp/a.sh research/prompts/x-opus-model/tools/rerun.sh", 0),
        ("three-way-defense 写 crates，拒", "three-way-defense", "echo x > crates/x.rs", 1),
        ("three-way-defense 写报告，放行", "three-way-defense", "echo x > research/prompts/x-sonnet-output.md", 0),
        ("three-way-forward 写 records，拒", "three-way-forward", "echo x > records/x.md", 1),
        ("three-way-forward 写报告，放行", "three-way-forward", "echo x > research/prompts/x-sonnet-output.md", 0),
        ("three-way-materials 改正文，拒", "three-way-materials", "echo x >> research/prompts/_x-body.md", 1),
        ("three-way-materials 写小节清单，放行", "three-way-materials", "cat > research/prompts/_x-checklist.md <<'EOF'\nx\nEOF", 0),
        ("three-way-materials 写开工快照的清单，放行", "three-way-materials", "echo x > research/prompts/x-snapshot/kb-sha256.txt", 0),
    ]
    read_only_scope_not_run = []   # hook 所在的仓里没有那个定义（仓副本没带 .claude/agents/）的格不判，成功句里报「本次未跑」
    for label, agent, command, want in read_only_scope_cases:
        if not agent_has_definition(read_only_scope_root, agent):
            read_only_scope_not_run.append(f"Bash写范围:只读定义:{label}（hook 所在的仓里没有 {agent} 的定义）")
            continue
        case_verdict = bash_scope_refusal(command, agent, read_only_scope_root, read_only_scope_root, read_only_scope_table)
        results.append((f"Bash写范围:只读定义:{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
    # ⑫ 只带 Read、Bash 的定义写工作根下的草稿目录也照写者锁判（writer_lock_refusal，lib_writer_lock.writer_verdict 与写闸第九道同一份）：持锁的是别的子 agent 的拒，自己持锁的、没有锁的放行；
    #   弄坏开关 scope-writer-lock-unchecked 下「拒」的那一格红；hook 所在的仓里没有 prior-art 的定义、写者锁库读不到时不判，成功句里报「本次未跑」
    read_only_lock_locks = writer_locks_library()
    if agent_has_definition(read_only_scope_root, "prior-art") and read_only_lock_locks is not None and work_roots is not None:
        os.makedirs(work_roots.configured_work_root(), exist_ok=True)
        read_only_lock_scratch = tempfile.mkdtemp(prefix="bash-command-detector-selftest-lock-", dir=work_roots.configured_work_root())
        try:
            for lock_name, lock_holder in (("held-by-other", "agent-other"), ("held-by-self", "agent-self")):
                lock_directory = os.path.join(read_only_lock_scratch, lock_name)
                read_only_lock = read_only_lock_locks.new_lock("s-own", f"自证 {lock_holder}", "prior-art", os.path.join(work, "no-transcript.jsonl"))
                read_only_lock_locks.create_lock(lock_directory, read_only_lock)
                read_only_lock_locks.write_exclusively(read_only_lock_locks.claim_path(lock_directory, read_only_lock), {"agent_id": lock_holder, "claimed_at": 0})
            os.makedirs(os.path.join(read_only_lock_scratch, "unlocked"))
            unclaimed_lock = read_only_lock_locks.new_lock("s-own", "自证 unclaimed", "prior-art", os.path.join(work, "no-transcript.jsonl"))   # 有锁、没有认领，派发回执读不到
            read_only_lock_locks.create_lock(os.path.join(read_only_lock_scratch, "unclaimed-receipt-missing"), unclaimed_lock)
            read_only_lock_identity = {"agent_id": "agent-self", "session_id": "s-own", "transcript_path": os.path.join(work, "no-transcript.jsonl")}
            for label, lock_name, want in (("别的子 agent 持写者锁的草稿目录，拒", "held-by-other", 1), ("自己持写者锁的草稿目录，放行", "held-by-self", 0),
                                           ("没有写者锁的草稿目录，放行", "unlocked", 0)):
                case_verdict = bash_scope_refusal(f"echo x > {read_only_lock_scratch}/{lock_name}/a.md", "prior-art", read_only_scope_root, read_only_scope_root,
                                                  read_only_scope_table, read_only_lock_identity)
                results.append((f"Bash写范围:只读定义:写者锁:prior-art 写{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
            # 有锁、没有认领、派发回执读不到（unclaimed）：放行、不认领，并记一条检出（lib_writer_lock.py 文件头）；弄坏开关 scope-unclaimed-receipt-unrecorded 下检出那一项红
            notices_before_lock = len(JUDGING_NOTICES)
            unclaimed_verdict = bash_scope_refusal(f"echo x > {read_only_lock_scratch}/unclaimed-receipt-missing/a.md", "prior-art", read_only_scope_root, read_only_scope_root,
                                                   read_only_scope_table, read_only_lock_identity)
            results.append(("Bash写范围:只读定义:写者锁:prior-art 写有锁没认领、派发回执读不到的草稿目录，放行并记一条检出", (0, 1, 0),
                            (1 if unclaimed_verdict.out_of_scope or unclaimed_verdict.undeterminable else 0, len(JUDGING_NOTICES) - notices_before_lock,
                             len([name for name in os.listdir(os.path.join(read_only_lock_scratch, "unclaimed-receipt-missing")) if name.startswith(read_only_lock_locks.CLAIM_PREFIX)]))))
            del JUDGING_NOTICES[notices_before_lock:]
        finally:
            shutil.rmtree(read_only_lock_scratch, ignore_errors=True)
        # 只有归属文件 .owner、没有写者锁的草稿目录：.owner 写的会话不是这个会话的拒（scripts/hosts/work-directory-ownership.sh 的三行格式），自己会话的、归属文件写得不对的、没有归属文件的放行；
        # 弄坏开关 scope-owner-only-directory-unjudged 下「别的会话」那一格红
        owner_directories = {}
        try:
            for owner_name, owner_text in (("other-session", "session: s-other\ncreated: 2026-11-19\npurpose: 自证：别的会话的\n"), ("own-session", "session: s-own\ncreated: 2026-11-19\npurpose: 自证：本会话的\n"),
                                           ("malformed", "session: s-other\n"), ("no-owner", None)):
                owner_directories[owner_name] = tempfile.mkdtemp(prefix=f"bash-command-detector-selftest-owner-{owner_name}-", dir=work_roots.configured_work_root())
                if owner_text is not None:
                    with open(os.path.join(owner_directories[owner_name], ".owner"), "w", encoding="utf-8") as owner_handle:
                        owner_handle.write(owner_text)
            for label, owner_name, want in (("只有 .owner（别的会话的）、没有写者锁的草稿目录，拒", "other-session", 1), ("只有 .owner（这个会话的）、没有写者锁的草稿目录，放行", "own-session", 0),
                                            (".owner 写得不对（只有一行）的草稿目录，放行", "malformed", 0), ("没有 .owner 也没有写者锁的目录，放行", "no-owner", 0)):
                case_verdict = bash_scope_refusal(f"echo x > {owner_directories[owner_name]}/a.md", "prior-art", read_only_scope_root, read_only_scope_root,
                                                  read_only_scope_table, read_only_lock_identity)
                results.append((f"Bash写范围:只读定义:写者锁:prior-art 写{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
        finally:
            for owner_directory in owner_directories.values():
                shutil.rmtree(owner_directory, ignore_errors=True)
    else:
        read_only_scope_not_run.append("Bash写范围:只读定义:写者锁（hook 所在的仓里没有 prior-art 的定义，或写者锁库读不到）")
    # ⑫ 三方腿（frontmatter 写了 leg: three-way）往 research/prompts/ 写只许落在这一轮自己的报告与模型目录：轮名取派发回执里提示的行首「报告：research/prompts/<轮>-<模型>-output.md」那一行
    #   （写范围表只认形态，别的轮的报告与模型目录也在形态里）；第二次同立场腿（-s2）的报告与模型目录各有自己的行；回执读不到的放行并记一条检出。
    #   弄坏开关 scope-leg-round-unchecked 下「拒」与「记检出」那几格红；用这一份钩子所在的仓里的真表与真定义，没有就不判、成功句里报「本次未跑」
    leg_types = ("three-way-attack", "three-way-defense", "three-way-forward")
    leg_library = three_way_legs_library()
    if leg_library is not None and all(agent_has_definition(read_only_scope_root, leg_type) and leg_type in leg_library.three_way_leg_types(read_only_scope_root) for leg_type in leg_types):
        leg_transcript = os.path.join(work, "leg-transcript.jsonl")
        with open(leg_transcript, "w", encoding="utf-8") as leg_handle:
            for leg_agent, leg_report in (("agent-leg-attack", "research/prompts/leg-r3-opus-output.md"), ("agent-leg-attack-s2", "research/prompts/leg-r3-opus-s2-output.md"),
                                          ("agent-leg-defense", "research/prompts/leg-r3-sonnet-output.md"), ("agent-leg-forward", "research/prompts/leg-r3-sonnet-output.md")):
                leg_handle.write(json.dumps({"timestamp": "2026-01-01T00:00:00Z", "toolUseResult": {  # clock-times:allow 自证构造合成会话记录的样例时间戳
                    "status": "async_launched", "agentId": leg_agent, "prompt": f"草稿目录：~/x/\n报告：{leg_report}\n禁读清单：别的腿的报告\n"}}, ensure_ascii=False) + "\n")
        leg_cases = [
            ("攻方写这一轮自己的报告，放行", "three-way-attack", "agent-leg-attack", "echo x > research/prompts/leg-r3-opus-output.md", 0, 0),
            ("攻方写这一轮自己的模型目录里的文件，放行", "three-way-attack", "agent-leg-attack", "echo x > research/prompts/leg-r3-opus-model/tools/rerun.sh", 0, 0),
            ("攻方写更早一轮的报告，拒", "three-way-attack", "agent-leg-attack", "echo x > research/prompts/leg-r1-opus-output.md", 1, 0),
            ("攻方写更早一轮的模型目录里的文件，拒", "three-way-attack", "agent-leg-attack", "echo x > research/prompts/leg-r2-opus-model/planted.py", 1, 0),
            ("第二次同立场的攻方写自己的 -s2 报告，放行", "three-way-attack", "agent-leg-attack-s2", "echo x > research/prompts/leg-r3-opus-s2-output.md", 0, 0),
            ("第二次同立场的攻方写自己的 -s2 模型目录里的文件，放行", "three-way-attack", "agent-leg-attack-s2", "echo x > research/prompts/leg-r3-opus-s2-model/probe.py", 0, 0),
            ("第二次同立场的攻方写第一次那条腿的报告，拒", "three-way-attack", "agent-leg-attack-s2", "echo x > research/prompts/leg-r3-opus-output.md", 1, 0),
            ("辩方写这一轮自己的报告，放行", "three-way-defense", "agent-leg-defense", "echo x > research/prompts/leg-r3-sonnet-output.md", 0, 0),
            ("辩方写别的轮的报告，拒", "three-way-defense", "agent-leg-defense", "echo x > research/prompts/leg-r1-sonnet-output.md", 1, 0),
            ("正推腿写别的轮的报告，拒", "three-way-forward", "agent-leg-forward", "echo x > research/prompts/leg-r2-sonnet-output.md", 1, 0),
            ("派发回执读不到的攻方写别的轮的报告：只照写范围表放行，记一条检出", "three-way-attack", "agent-leg-unlaunched", "echo x > research/prompts/leg-r1-opus-output.md", 0, 1),
        ]
        for label, leg_type, leg_agent, command, want_refused, want_notices in leg_cases:
            notices_before_leg = len(JUDGING_NOTICES)
            case_verdict = bash_scope_refusal(command, leg_type, read_only_scope_root, read_only_scope_root, read_only_scope_table,
                                              {"agent_id": leg_agent, "session_id": "s-leg", "transcript_path": leg_transcript})
            results.append((f"Bash写范围:三方腿:{label}", (want_refused, want_notices),
                            (1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0, len(JUDGING_NOTICES) - notices_before_leg)))
            del JUDGING_NOTICES[notices_before_leg:]
    else:
        read_only_scope_not_run.append("Bash写范围:三方腿（hook 所在的仓里三方腿的定义没有写 leg: three-way，或三方腿类型库读不到）")
    # ⑫ 认 ledger-append-section.py：它写的是仓根下登记表 current 那一份账本。另起一个假仓（不往 scope_repository 里建 records/：上面几格 cd records 的判法看目录在不在），
    #   放 clerk、kb-scribe 的定义、登记表与账本，写范围表用上面那一张；仓外再放一份带登记表的副本给 --root 指。(说明, agent 类型, 命令, 该不该拒)
    #   弄坏开关 scope-ledger-append-unseen 下「拒」的三格红
    ledger_scope_repository = os.path.realpath(os.path.join(work, "ledger-scope-repository"))
    ledger_scope_copy = os.path.realpath(os.path.join(work, "ledger-scope-copy"))
    for ledger_scope_root in (ledger_scope_repository, ledger_scope_copy):
        os.makedirs(os.path.join(ledger_scope_root, ".claude", "agents"))
        os.makedirs(os.path.join(ledger_scope_root, ".claude", "gate.d"))
        os.makedirs(os.path.join(ledger_scope_root, "records"))
        with open(os.path.join(ledger_scope_root, LEDGER_REGISTRY_PATH), "w", encoding="utf-8") as handle:
            handle.write("current\trecords/ledger.md\t-\t样本里的现行账本，自证用\n")
        with open(os.path.join(ledger_scope_root, "records", "ledger.md"), "w", encoding="utf-8") as handle:
            handle.write("# 账本\n")
    for agent_name in ("clerk", "kb-scribe"):
        with open(os.path.join(ledger_scope_repository, ".claude", "agents", f"{agent_name}.md"), "w", encoding="utf-8") as handle:
            handle.write("---\ntools: Read, Edit, Write, Bash\n---\n")
    ledger_append_command = "python3 scripts/kb/ledger-append-section.py --title x --body-file /tmp/body.md"
    ledger_scope_cases = [
        ("写范围里没有 records/ 的 kb-scribe 起 ledger-append-section.py 写现行账本，拒", "kb-scribe", ledger_append_command, 1),
        ("kb-scribe 直接执行 ledger-append-section.py 写现行账本，拒", "kb-scribe", "scripts/kb/ledger-append-section.py --title x --body-file /tmp/body.md", 1),
        ("kb-scribe 的 ledger-append-section.py --root 写成没赋过值的变量，落点算不出，拒", "kb-scribe",
         "python3 scripts/kb/ledger-append-section.py --root \"$LEDGER_ROOT_UNSET\" --title x --body-file /tmp/body.md", 1),
        ("写范围有 records/ 的 clerk 起 ledger-append-section.py 放行", "clerk", ledger_append_command, 0),
        ("主 agent 起 ledger-append-section.py 放行", None, ledger_append_command, 0),
        ("kb-scribe 的 ledger-append-section.py --dry-run 不写，放行", "kb-scribe", ledger_append_command + " --dry-run", 0),
        ("kb-scribe 的 ledger-append-section.py --root= 点名仓外的副本，放行", "kb-scribe",
         f"python3 scripts/kb/ledger-append-section.py --root={ledger_scope_copy} --title x --body-file /tmp/body.md", 0),
    ]
    for label, agent, command, want in ledger_scope_cases:
        case_verdict = bash_scope_refusal(command, agent, ledger_scope_repository, ledger_scope_repository, scope_table)
        results.append((f"Bash写范围:{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
    # ⑫ 认 append-record.py：它写的是第一个不是选项的参数 <记录>（--root 的值不算）。用上面那个假仓（有 records/ledger.md、clerk 与 kb-scribe 的定义）与同一张写范围表；
    #   弄坏开关 scope-record-append-unseen 下「拒」的五格红，scope-record-append-root-as-target 下 --root 写在记录前面那一格红。(说明, agent 类型, 命令, 该不该拒)
    record_append_command = "python3 scripts/kb/append-record.py records/ledger.md /tmp/text.md"
    record_append_scope_cases = [
        ("写范围里没有 records/ 的 kb-scribe 起 append-record.py 往记录追加，拒", "kb-scribe", record_append_command, 1),
        ("kb-scribe 直接执行 append-record.py 往记录追加，拒", "kb-scribe", "scripts/kb/append-record.py records/ledger.md /tmp/text.md", 1),
        ("kb-scribe cd 进 records 之后按相对路径起 append-record.py，拒", "kb-scribe", "cd records && python3 ../scripts/kb/append-record.py ledger.md -", 1),
        ("kb-scribe 的 append-record.py 先写 --root 再写记录，落点照第一个位置参数认，拒", "kb-scribe",
         f"python3 scripts/kb/append-record.py --root {ledger_scope_copy} records/ledger.md /tmp/text.md", 1),
        ("kb-scribe 的 append-record.py 记录写成没赋过值的变量，落点算不出，拒", "kb-scribe",
         "python3 scripts/kb/append-record.py \"$RECORD_UNSET\" /tmp/text.md", 1),
        ("写范围有 records/ 的 clerk 起 append-record.py 放行", "clerk", record_append_command, 0),
        ("主 agent 起 append-record.py 放行", None, record_append_command, 0),
        ("kb-scribe 的 append-record.py --dry-run 不写，放行", "kb-scribe", record_append_command + " --dry-run", 0),
        ("kb-scribe 的 append-record.py --selftest 不写，放行", "kb-scribe", "python3 scripts/kb/append-record.py --selftest", 0),
        ("kb-scribe 的 append-record.py 记录在仓外的副本里，不判", "kb-scribe",
         f"python3 scripts/kb/append-record.py {ledger_scope_copy}/records/ledger.md /tmp/text.md", 0),
    ]
    for label, agent, command, want in record_append_scope_cases:
        case_verdict = bash_scope_refusal(command, agent, ledger_scope_repository, ledger_scope_repository, scope_table)
        results.append((f"Bash写范围:{label}", want, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
    # 钩子进程自己的当前目录在工作根下时，sed -i 展开不了的目标照算不出拒（一百三十七节件 1b；弄坏开关 sed-unknown-target-resolved-from-cwd 下这两格红）
    if work_roots is not None and os.path.isdir(work_roots.work_root()):
        process_directory = os.getcwd()
        os.chdir(work_roots.work_root())
        try:
            for label, command in (("一百三十七：钩子进程在工作根下，clerk sed -i 没赋过值的变量照拒", "sed -i s/a/b/ $F"),
                                   ("一百三十七：钩子进程在工作根下，clerk xargs sed -i 照拒", "echo records/a.md | xargs sed -i s/a/b/")):
                case_verdict = bash_scope_refusal(command, "clerk", scope_repository, scope_repository, scope_table)
                results.append((f"Bash写范围:{label}", 1, 1 if case_verdict.out_of_scope or case_verdict.undeterminable else 0))
        finally:
            os.chdir(process_directory)
    # 目标里有变量、命令替换、通配的：落点算得出的照判，算不出的拒（弄坏开关 scope-undeterminable-only-noted 下「拒」那几格变成「记」、红）；(说明, agent 类型, 命令, 结局：拒 / 放行 / 记)
    def scope_outcome(command, agent="clerk", identity=None):
        verdict = bash_scope_refusal(command, agent, scope_repository, scope_repository, scope_table, identity)
        return "拒" if verdict.out_of_scope or verdict.undeterminable else "记" if verdict.noted else "放行"
    landing_directory = os.path.join(work, "landing")          # 仓外的目录里放一个指到仓内知识库的符号链接：`<链接>/../x` 照 shell 逐段解开回到仓里
    os.makedirs(os.path.join(scope_repository, ".claude", "kb", "sub"), exist_ok=True)
    os.makedirs(landing_directory, exist_ok=True)
    os.symlink(os.path.join(scope_repository, ".claude", "kb", "sub"), os.path.join(landing_directory, "kb-link"))
    climb_back = os.path.relpath(os.path.join(scope_repository, ".claude", "kb", "a.md"), os.path.join(new_root, "clerk-x"))   # 从草稿目录爬回仓里知识库的 ../…
    # 仓里的链接：范围内的指到仓里范围外（records/kb-link → .claude/kb）、范围内的指回范围内、范围外的指回范围内、范围外的指到仓外
    os.makedirs(os.path.join(scope_repository, "records", "sub"), exist_ok=True)
    os.symlink(os.path.join(scope_repository, ".claude", "kb"), os.path.join(scope_repository, "records", "kb-link"))
    os.symlink(os.path.join(scope_repository, "records", "sub"), os.path.join(scope_repository, "records", "back-link"))
    os.symlink(os.path.join(scope_repository, "records"), os.path.join(scope_repository, ".claude", "kb", "records-link"))
    os.symlink(landing_directory, os.path.join(scope_repository, ".claude", "kb", "outside-link"))
    scope_dynamic_cases = [
        # python3 起的脚本词代入之后还算不出、第一个参数落在本仓里的，照算不出拒（弄坏开关 scope-script-variable-unexpanded 下前三格变成「放行」、
        # scope-undeterminable-only-noted 下变成「记」，都红）；第一个参数在仓外、带 --dry-run、$HOME 下的脚本不判
        ("python3 起没赋过值的变量里的脚本、第一个参数落在知识库，拒", "clerk", "python3 \"$WRITER\" .claude/kb/x.md a b", "拒"),
        ("python3 起没赋过值的变量里的脚本、第一个参数落在写范围里的 records 也拒", "clerk", "python3 $WRITER records/x.md a b", "拒"),
        ("python3 起命令替换给的脚本、第一个参数是仓根起的绝对路径，拒", "clerk", f"python3 $(ls scripts/kb/*.py | head -1) {scope_repository}/.claude/kb/x.md a b", "拒"),
        ("python3 起没赋过值的变量里的脚本、第一个参数在仓外，放行", "clerk", "python3 \"$WRITER\" /tmp/x.md a b", "放行"),
        ("python3 起没赋过值的变量里的脚本、带 --dry-run，放行", "clerk", "python3 \"$WRITER\" .claude/kb/x.md a b --dry-run", "放行"),
        ("python3 起 $HOME 下的脚本不当算不出，放行", "clerk", "python3 $HOME/tools/report.py records/x.md", "放行"),
        ("目标在没赋过值的变量里，拒", "clerk", "python3 scripts/kb/replace-once.py \"$TARGET\" a b", "拒"),
        ("重定向目标带通配，拒", "clerk", "echo x > .claude/kb/*.md", "拒"),
        ("replace-batch.py 规格读不到，拒", "clerk", "python3 scripts/kb/replace-batch.py no-such-spec.json", "拒"),
        ("cd 到没赋过值的变量目录之后写相对路径，拒", "clerk", "cd \"$WHERE\" && echo x > a.md", "拒"),
        ("cd 算不出之后 python 写相对路径，拒", "clerk", "cd \"$WHERE\" && python3 -c \"open('a.md', 'w').write('x')\"", "拒"),
        # 例：for 列表写死，但目标里有命令替换 $(echo $f | tr / _)
        ("检出原文：mkdir -p <草稿目录>/originals && for f in …; do cp -p \"$f\" \"<草稿目录>/originals/$(echo $f | tr / _)\"; done，拒", "clerk",
         f"mkdir -p {new_root}/m4-per-run-build-dirs/originals && for f in scripts/gate/merge-and-verify.sh scripts/mutation/crates-mutation-rows.py "
         f"scripts/crash/crash-point-replay.sh; do cp -p \"$f\" \"{new_root}/m4-per-run-build-dirs/originals/$(echo $f | tr / _)\"; done", "拒"),
        ("检出原文：S=<草稿目录> && mkdir -p $S/originals && for f in …; do cp -- \"$f\" \"$S/originals/$(echo $f | tr / _)\"; done，拒", "clerk",
         f"S={new_root}/m4-completion-cleanup-hooks && mkdir -p $S/originals && for f in .claude/settings.json .claude/hooks/selftest-registry.tsv "
         f".claude/agent-common.md; do cp -- \"$f\" \"$S/originals/$(echo $f | tr / _)\"; done", "拒"),
        # 写死的词的循环变量逐个取值（弄坏开关 scope-loop-values-unknown 下这几格红）
        ("for b in 写死的词; do … > <草稿目录>/breaks/$b.out，放行", "clerk", f"for b in one two three; do echo x > {new_root}/clerk-x/breaks/$b.out; done", "放行"),
        ("cd 到写死的范围内目录之后 for 循环（列表是写死的名字）里写 $d/…，放行", "clerk", "cd records && for d in a b; do echo x > $d/.gate-cells; done", "放行"),
        ("for f in 写死的仓内路径; do cp $f <草稿目录>/; done，放行", "clerk", f"for f in records/a.md .claude/kb/b.md; do cp \"$f\" {new_root}/clerk-x/; done", "放行"),
        ("S=<草稿目录> 之后 for f in 写死的词 写 $S/$f.out，放行", "clerk", f"S={new_root}/clerk-x && for f in a b; do echo x > \"$S/$f.out\"; done", "放行"),
        # 取值里的 .. 与 / 照逐个落点判（弄坏开关 scope-loop-landings-trusted 下这几格红）
        ("for 的列表里有爬回仓里的 ..，落在写范围外，拒", "clerk", f"for f in ok {climb_back}; do echo x > {new_root}/clerk-x/$f; done", "拒"),
        ("for 的列表里有 .. 段、落点回到写范围之外，拒", "clerk", "cd records && for d in a ../b; do echo x > $d/a.md; done", "拒"),
        ("for 的列表里有写范围外的仓内路径，cp 写到它，拒", "clerk", "for f in records/a.md .claude/kb/b.md; do cp /tmp/x \"$f\"; done", "拒"),
        # 变量来源不明的：静态前缀在哪都拒（静态前缀在仓外或范围里也拒：变量的值可以带 /.. 爬回仓里）
        ("f 来源不明的 > <草稿目录>/out-$f，拒", "clerk", f"echo x > {new_root}/clerk-x/out-$f", "拒"),
        ("变量来源不明、静态前缀在工作根下（仓外），拒", "clerk", f"echo x > {new_root}/clerk-x/breaks/$b.out", "拒"),
        ("变量来源不明、静态前缀在 /tmp 下（仓外），拒", "clerk", "echo x > /tmp/claude-1000/s/scratchpad/$f.out", "拒"),
        ("变量来源不明、静态前缀已在写范围里，拒", "clerk", "echo x > records/n-$f.md", "拒"),
        ("命令替换在最后一段、静态前缀已在写范围里，拒", "clerk", "echo x > records/n-$(date +%F).md", "拒"),
        ("cp 进范围内目录、源名算不出，拒", "clerk", "cp $SRC records/", "拒"),
        ("cd 到写死的范围内目录之后写带来源不明变量的相对路径，拒", "clerk", "cd records && echo x > gz-$k/.gate-cells", "拒"),
        ("cd 到写死的范围内目录之后写变量开头的相对路径，拒", "clerk", "cd records && echo x > $k/.gate-cells", "拒"),
        ("for 循环的列表里有算不出的词，拒", "clerk", "cd records && for d in $(ls); do echo x > $d/a.md; done", "拒"),
        ("通配目标在范围内目录下，拒", "clerk", "echo x > records/*.md", "拒"),
        ("静态前缀在仓内、写范围之外，拒", "clerk", "echo x > .claude/kb/$x.md", "拒"),
        ("目标以变量开头，拒", "clerk", "echo x > $REPO/.claude/kb/$x.md", "拒"),
        ("变量单独占一段，拒", "clerk", "echo x > records/$x", "拒"),
        ("变量之后有字面的 .. 段，拒", "clerk", "echo x > records/n-$x/../../.claude/kb/a.md", "拒"),
        ("仓外目录下变量单独占一段，拒", "clerk", f"echo x > {new_root}/clerk-x/$f", "拒"),
        ("cd 到写死的范围外目录之后写带变量的相对路径，拒", "clerk", "cd .claude/kb && echo x > gz-$k/a.md", "拒"),
        ("cd 到算不出的目录之后写带变量的相对路径，拒", "clerk", "cd \"$WHERE\" && echo x > gz-$k/a.md", "拒"),
        ("cd 进循环变量拼的目录之后写相对路径，拒", "clerk", f"cd {new_root}/clerk-x && for d in a b; do cd $d; echo x > y; done", "拒"),
        ("目标明确的放行", "clerk", "echo x > records/a.md", "放行"),
        ("目标明确、写范围之外的拒", "clerk", "echo x > crates/a.rs", "拒"),
        # 落点照 shell 逐段解开符号链接（弄坏开关 scope-landing-lexical-only 下这一格红）
        ("仓外的符号链接指到仓内知识库，<链接>/../x 回到仓里，拒", "clerk", f"echo x > {landing_directory}/kb-link/../x.md", "拒"),
        # 仓里的链接按解开之后的落点判，与写闸同一份判法（弄坏开关 scope-landing-lexical-only 下除「范围内的链接指回范围内」之外这几格红）
        ("范围内的链接指到仓里范围外的目录，经它 > 写，拒", "clerk", "echo x > records/kb-link/a.md", "拒"),
        ("范围内的链接指到仓里范围外的目录，经它 cp 写，拒", "clerk", "cp /tmp/a.md records/kb-link/a.md", "拒"),
        ("范围内的链接指到仓里范围外的目录，经它 replace-once.py 写，拒", "clerk", "python3 scripts/kb/replace-once.py records/kb-link/a.md a b", "拒"),
        ("范围内的链接指到仓里范围外的目录，cp 进链接（目标是目录），拒", "clerk", "cp /tmp/a.md records/kb-link/", "拒"),
        ("范围内的链接指回范围内，经它写，放行", "clerk", "echo x > records/back-link/a.md", "放行"),
        ("范围外的链接指回范围内，经它写，放行", "clerk", "echo x > .claude/kb/records-link/a.md", "放行"),
        ("范围外的链接指到仓外，经它写的落点在仓外，不判", "clerk", "echo x > .claude/kb/outside-link/a.md", "放行"),
        ("$HOME 照 hook 的环境展开，落在仓外放行", "clerk", "echo x > $HOME/bash-command-detector-selftest-probe.out", "放行"),
        # $PWD：前面有算得出的 cd 时按那个目录展开（弄坏开关 pwd-variable-unexpanded 下前四格红）；没有 cd、cd 算不出的拒
        ("cd 到写死的草稿目录之后写 $PWD/… 不记", "clerk", f"cd {new_root}/clerk-x && echo x > $PWD/a.md", "放行"),
        ("cd 到写死的范围内目录之后写 $PWD/… 不记", "clerk", "cd records && echo x > $PWD/a.md", "放行"),
        ("cd 到写死的草稿目录之后 cp 进 $PWD/子目录/ 不记", "clerk", f"cd {new_root}/clerk-x && cp a.md $PWD/sub/", "放行"),
        ("cd 到写死的草稿目录之后 T=$PWD/x 再写 $T/… 不记", "clerk", f"cd {new_root}/clerk-x && T=$PWD/x && echo x > $T/a.md", "放行"),
        ("没有 cd 的 $PWD，拒", "clerk", "echo x > $PWD/a.md", "拒"),
        ("cd 到算不出的目录之后 $PWD，拒", "clerk", "cd \"$WHERE\" && echo x > $PWD/a.md", "拒"),
        ("先 cd 到写死的目录、再 cd 到算不出的目录，$PWD 作废，拒", "clerk", "cd records && cd \"$WHERE\" && echo x > $PWD/a.md", "拒"),
        ("cd 到写死的草稿目录之后 $PWD/子目录/$p，$p 来源不明，拒", "clerk", f"cd {new_root}/clerk-x && cp a.md $PWD/sub/$p", "拒"),
        # mktemp -d 赋的变量当作落在 ${TMPDIR:-/tmp} 下（弄坏开关 mktemp-directory-unknown 下前六格红）；别的写法的 mktemp 拒
        ("W=$(mktemp -d) 之后 cp 进 $W/ 不记", "clerk", "W=$(mktemp -d) && cp -a records/a.md $W/", "放行"),
        ("W=`mktemp -d`（反引号）之后写 $W/… 不记", "clerk", "W=`mktemp -d`; echo x > $W/a.md", "放行"),
        ("export W=$(mktemp -d) 之后写 $W/… 不记", "clerk", "export W=$(mktemp -d); echo x > $W/a.md", "放行"),
        ("mktemp -d 之后 cd 进去写相对路径不记", "clerk", "W=$(mktemp -d) && cd \"$W\" && echo x > a.md", "放行"),
        ("mktemp -d -t 带模板不记", "clerk", "W=$(mktemp -d -t gate.XXXXXX) && echo x > $W/a.md", "放行"),
        ("mktemp --directory 不记", "clerk", "W=$(mktemp --directory) && echo x > $W/a.md", "放行"),
        ("mktemp -d -p 指的目录算不出，拒", "clerk", "W=$(mktemp -d -p \"$BASE\") && echo x > $W/a.md", "拒"),
        ("mktemp 不带 -d 建的是文件，拒", "clerk", "W=$(mktemp) && echo x > $W/a.md", "拒"),
        ("mktemp -d 的模板在当前目录里，拒", "clerk", "W=$(mktemp -d work.XXXXXX) && echo x > $W/a.md", "拒"),
        # 循环变量读的是什么算不出（while read 读文件，for 的列表来自变量）：拒，前面赋过的同名变量也作废
        ("while read 的循环变量，拒", "clerk", f"while read f; do cp a.md {new_root}/clerk-x/tree/$f; done < files.txt", "拒"),
        ("for 的列表来自变量，拒", "clerk", f"for f in $FILES; do cp a.md {new_root}/clerk-x/tree/$f; done", "拒"),
        ("前面赋过写死值的变量又被 for $列表 改写，拒", "clerk", f"f=ok; for f in $FILES; do cp a.md {new_root}/clerk-x/tree/$f; done", "拒"),
        ("主 agent 的目标算不出也不判", None, "echo x > $F", "放行"),
    ]
    for label, agent, command, want in scope_dynamic_cases:
        results.append((f"Bash写范围:{label}", want, scope_outcome(command, agent)))
    # mktemp -d 的模板在这个子 agent 认领的草稿目录里（拆分提案第九十节 ⑰）：工作根下建两个第一层目录，一个的归属文件写会话 s-own（own/ 的锁认领给 agent-own，
    #   other/ 的锁认领给 agent-other，receipt/ 的锁没有认领文件、靠派发回执认），另一个的归属文件写别的会话（锁认领给 agent-own）；
    #   弄坏开关 mktemp-scratch-template-unknown 下「不记」的四格与 stdin 那一格红，mktemp-template-owner-unchecked 下「别的子 agent」「别的会话」「派发回执对不上」三格红
    locks = writer_locks_library()
    owned_scratch, foreign_scratch = None, None
    try:
        os.makedirs(new_root, exist_ok=True)
        owned_scratch = tempfile.mkdtemp(prefix="bash-command-detector-selftest-", dir=new_root)
        foreign_scratch = tempfile.mkdtemp(prefix="bash-command-detector-selftest-", dir=new_root)
        own_directory, other_directory = os.path.join(owned_scratch, "own"), os.path.join(owned_scratch, "other")
        for first_level, session in ((owned_scratch, "s-own"), (foreign_scratch, "s-foreign")):
            open(os.path.join(first_level, ".owner"), "w", encoding="utf-8").write(f"session: {session}\ncreated: 2026-10-22\npurpose: 自证\n")
        for directory, holder in ((own_directory, "agent-own"), (other_directory, "agent-other"), (foreign_scratch, "agent-own")):
            lock = locks.new_lock("s-own", f"自证 {holder}", "clerk", os.path.join(work, "no-transcript.jsonl"))
            locks.create_lock(directory, lock)
            locks.write_exclusively(locks.claim_path(directory, lock), {"agent_id": holder, "claimed_at": 0})
        identity = {"agent_id": "agent-own", "session_id": "s-own", "transcript_path": os.path.join(work, "no-transcript.jsonl")}
        # 还没有认领文件的那把锁：主会话记录里 agent-receipt 的派发回执（提示与锁对得上、时刻不早于锁）认它
        receipt_directory, receipt_transcript = os.path.join(owned_scratch, "receipt"), os.path.join(work, "receipt-session.jsonl")
        receipt_lock = locks.new_lock("s-own", "自证 receipt", "clerk", receipt_transcript)
        locks.create_lock(receipt_directory, receipt_lock)
        with open(receipt_transcript, "w", encoding="utf-8") as handle:
            handle.write(json.dumps({"timestamp": datetime.fromtimestamp(receipt_lock["dispatched_at"] + 1, timezone.utc).isoformat(),
                                     "toolUseResult": {"status": "async_launched", "agentId": "agent-receipt", "prompt": "自证 receipt"}}, ensure_ascii=False) + "\n")
        receipt_identity = {"agent_id": "agent-receipt", "session_id": "s-own", "transcript_path": receipt_transcript}
        scratch_mktemp_cases = [
            ("模板在自己认领的草稿目录里：T=$(mktemp -d $W/名.XXXXXX) 之后 cp 进 $T/ 不记", identity,
             f"W={own_directory}; T=$(mktemp -d $W/rs.XXXXXX); cp -a records/a.md $T/", "放行"),
            ("模板在自己认领的草稿目录里：直接写进 $(mktemp -d …)/文件 不记", identity, f"echo x > $(mktemp -d {own_directory}/real.XXXXXX)/cell.py", "放行"),
            ("cd 进自己认领的草稿目录之后模板写相对路径不记", identity, f"cd {own_directory} && T=$(mktemp -d runs.XXXXXX) && echo x > $T/a.md", "放行"),
            ("还没有认领文件、派发回执对得上那把锁：模板在它的草稿目录里不记", receipt_identity, f"T=$(mktemp -d {receipt_directory}/rs.XXXXXX); echo x > $T/a.md", "放行"),
            ("派发回执对不上那把锁（别的子 agent）：拒", {**receipt_identity, "agent_id": "agent-stranger"}, f"T=$(mktemp -d {receipt_directory}/rs.XXXXXX); echo x > $T/a.md", "拒"),
            ("模板在别的子 agent 认领的草稿目录里，拒", identity, f"T=$(mktemp -d {other_directory}/rs.XXXXXX); cp -a records/a.md $T/", "拒"),
            ("模板目录的归属文件是别的会话，拒", identity, f"T=$(mktemp -d {foreign_scratch}/rs.XXXXXX); echo x > $T/a.md", "拒"),
            ("模板在 /tmp 下（不是草稿目录），拒", identity, "T=$(mktemp -d /tmp/probe.XXXXXX); echo x > $T/a.md", "拒"),
            ("hook 输入里没有 agent_id，模板在草稿目录里也拒", None, f"W={own_directory}; T=$(mktemp -d $W/rs.XXXXXX); cp -a records/a.md $T/", "拒"),
            # 拆分提案第九十节 ㉕：-p、--tmpdir= 与套在 $(cd … && pwd -P) 里的形状
            ("-p 指到自己认领的草稿目录：T=$(mktemp -d -p \"$W\") 之后写进 $T/ 不记", identity, f"W={own_directory}; T=$(mktemp -d -p \"$W\"); echo x > $T/a.md", "放行"),
            ("--tmpdir= 指到自己认领的草稿目录（带模板名）：cp 进 $T/ 不记", identity,
             f"T=$(mktemp -d --tmpdir={own_directory} rs.XXXXXX); cp -a records/a.md $T/", "放行"),
            ("cd 进自己认领的草稿目录之后 -p . 不记", identity, f"cd {own_directory} && T=$(mktemp -d -p .) && echo x > $T/a.md", "放行"),
            ("直接写进 $(mktemp -d -p …)/文件 不记", identity, f"echo x > $(mktemp -d -p {own_directory})/cell.py", "放行"),
            ("不带值的 --tmpdir：W=$(mktemp -d --tmpdir) 之后写进 $W/ 不记", identity, "W=$(mktemp -d --tmpdir) && echo x > $W/a.md", "放行"),
            ("-p 指到别的子 agent 认领的草稿目录，拒", identity, f"T=$(mktemp -d -p {other_directory}); cp -a records/a.md $T/", "拒"),
            ("--tmpdir= 指到别的会话的草稿目录，拒", identity, f"T=$(mktemp -d --tmpdir={foreign_scratch}); echo x > $T/a.md", "拒"),
            ("-p 指到 /tmp（不是草稿目录），拒", identity, "T=$(mktemp -d -p /tmp); echo x > $T/a.md", "拒"),
            ("-p 指到仓里的目录，拒", identity, "T=$(mktemp -d -p records); echo x > $T/a.md", "拒"),
            ("hook 输入里没有 agent_id，-p 指到草稿目录也拒", None, f"T=$(mktemp -d -p {own_directory}); echo x > $T/a.md", "拒"),
            ("套一层 cd 加 pwd -P：T=$(cd $(mktemp -d -p \"$W\") && pwd -P) 之后写进 $T/ 不记", identity,
             f"W={own_directory}; T=$(cd $(mktemp -d -p \"$W\") && pwd -P); echo x > $T/a.md", "放行"),
            ("套一层、不带 -p：T=$(cd $(mktemp -d) && pwd -P) 之后写进 $T/ 不记", identity, "T=$(cd $(mktemp -d) && pwd -P); echo x > $T/a.md", "放行"),
            ("套一层、反引号写法：T=`cd $(mktemp -d -p …) && pwd` 之后写进 $T/ 不记", identity,
             f"T=`cd $(mktemp -d -p {own_directory}) && pwd`; echo x > $T/a.md", "放行"),
            ("套一层、直接写进 $(cd $(mktemp -d -p …) && pwd -P)/文件 不记", identity,
             f"echo x > $(cd $(mktemp -d -p {own_directory}) && pwd -P)/cell.py", "放行"),
            ("套一层、-p 指到别的子 agent 认领的草稿目录，拒", identity,
             f"T=$(cd $(mktemp -d -p {other_directory}) && pwd -P); echo x > $T/a.md", "拒"),
            ("套一层、cd 之后接的不是 pwd，拒", identity, "T=$(cd $(mktemp -d) && echo /tmp); echo x > $T/a.md", "拒"),
        ]
        for label, case_identity, command, want in scratch_mktemp_cases:
            results.append((f"Bash写范围:{label}", want, scope_outcome(command, "clerk", case_identity)))
        before_scratch_noted = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
        scratch_payload = {"tool_name": "Bash", "session_id": "s-own", "agent_id": "agent-own", "agent_type": "clerk", "transcript_path": identity["transcript_path"],
                           "tool_input": {"command": f"T=$(mktemp -d {own_directory}/rs.XXXXXX); cp -a records/a.md $T/; echo x > $(mktemp -d {own_directory}/real.XXXXXX)/cell.py",
                                          "run_in_background": False}}
        scratch_entry = subprocess.run(["bash", os.path.join(scope_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                                       env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections), input=json.dumps(scratch_payload))
        results.append(("stdin:模板在自己认领的草稿目录里的 mktemp -d 放行（退出码 0）、不记检出", (0, 0),
                        (scratch_entry.returncode, (sum(1 for _ in open(detections)) if os.path.exists(detections) else 0) - before_scratch_noted)))
    finally:
        for directory in (owned_scratch, foreign_scratch):
            if directory:
                shutil.rmtree(directory, ignore_errors=True)
    # 把活放出追踪的写法：(说明, 命令, 该不该拒绝)；前台、run_in_background 一样拒，detaching_refusal 不看 run_in_background
    detaching_cases = [
        ("现场：& disown 之后又 & wait", "nice -n 19 bash long.sh > long.log 2>&1 & disown 2>/dev/null || true & wait", 1),
        ("nohup … &", "nohup bash long.sh > long.log 2>&1 &", 1),
        ("nice 包着的 nohup … &", "nice -n 19 nohup bash long.sh > long.log 2>&1 & echo $!", 1),
        ("coproc", "coproc GATE { bash gate.sh; }", 1),
        ("setsid -f", "setsid -f bash long.sh > long.log 2>&1 < /dev/null", 1),
        ("setsid --fork", "setsid --fork bash long.sh", 1),
        ("tmux new-session -d", "tmux new-session -d -s gate 'bash gate.sh'", 1),
        ("screen -dmS", "screen -dmS gate bash -c 'bash gate.sh'", 1),
        ("systemd-run 不等结束", "systemd-run --user --collect --unit gate-run bash gate.sh", 1),
        ("bash -c 里的 & disown", "bash -c 'bash long.sh & disown'", 1),
        ("命令替换里的 nohup … &", "pid=$(nohup bash long.sh > long.log 2>&1 & echo $!)", 1),
        ("capped.sh 包着的 setsid -f", "bash scripts/hosts/capped.sh 4 setsid -f bash long.sh", 1),
        ("喂给 bash 的 heredoc 里的 disown", "bash <<'EOF'\nbash long.sh &\ndisown\nEOF", 1),
        ("nohup 不带 &", "nohup bash long.sh > long.log 2>&1", 0),
        ("裸 setsid 不 fork", "setsid bash long.sh > long.log 2>&1 < /dev/null", 0),
        ("setsid 跟着的 tail -f", "setsid tail -f log.txt", 0),
        ("systemd-run --wait", "systemd-run --user --wait bash gate.sh", 0),
        ("tmux ls", "tmux ls", 0),
        ("并行加 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", 0),
        ("echo 引号里当数据写", 'echo "nohup bash x.sh & disown"', 0),
        ("grep 的参数里写着 disown", "grep -n 'disown\\|coproc' notes.md", 0),
        ("写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\nnohup bash long.sh &\ndisown\nEOF", 0),
        ("注释里写的", "ls  # nohup bash long.sh & disown", 0),
    ]
    for label, command, want in detaching_cases:
        results.append((f"放出追踪:{label}", want, 1 if detaching_refusal(command) else 0))
    # run_in_background 里后面没有 wait 的单独 &：(说明, 命令, run_in_background, 该不该拒绝)
    unwaited_ampersand_cases = [
        ("结尾单独一个 &", "bash long.sh > long.log 2>&1 &", True, 1),
        ("& 之后接 echo $!", "bash long.sh > long.log 2>&1 & echo $!", True, 1),
        ("& 在 capped.sh 前缀的那条命令上", "bash scripts/hosts/capped.sh 4 nice -n 19 cargo test --release > test.log 2>&1 &", True, 1),
        ("& 在 capped.sh 包着的 bash -c 里", "bash scripts/hosts/capped.sh 4 bash -c 'cargo test --release > test.log 2>&1 &'", True, 1),
        ("& 在 wait 之后", "wait; bash late.sh &", True, 1),
        ("圆括号子 shell 里的 &、wait 在外面", "(bash long.sh > long.log 2>&1 &); wait", True, 1),
        ("& 之后的 wait 在另一对圆括号里", "bash long.sh & (wait)", True, 1),
        ("命令替换里的 &、wait 在外面", 'pid=$(bash long.sh > long.log 2>&1 & echo $!); wait "$pid"', True, 1),
        ("bash -c 里的 &、wait 在外面", "bash -c 'bash long.sh &'; wait", True, 1),
        ("喂给 bash 的 heredoc 正文里的 &", "bash <<'EOF'\nbash long.sh > long.log 2>&1 &\nEOF", True, 1),
        ("喂给 python 的 heredoc 挂在 & 前面", "python3 - > rerun.log 2>&1 <<'EOF' &\nimport subprocess\nEOF", True, 1),
        ("& 之后只 wait -n", "bash a.sh & bash b.sh & wait -n", True, 1),
        ("& 之后按 pid 轮询（tail --pid）算等到了（与检出 ② 同一套数法）", "bash long.sh > long.log 2>&1 & tail --pid=$! -f /dev/null", True, 0),
        ("整个循环放后台", "for part in a b; do bash $part.sh; done &", True, 1),
        ("一对圆括号整组放后台", "(bash a.sh; bash b.sh) > both.log 2>&1 &", True, 1),
        ('& 之后 wait "$pid"', 'bash long.sh > long.log 2>&1 & pid=$!; wait "$pid"', True, 0),
        ("&&", "cargo build --release && cargo test --release", True, 0),
        ("2>&1 重定向", "cargo test --release > test.log 2>&1", True, 0),
        ("|& 与 &>", "make |& tee out.txt; cargo test &> test.log", True, 0),
        ("并行加不带参数的 wait", "bash a.sh > a.log 2>&1 & bash b.sh > b.log 2>&1 & wait", True, 0),
        ("循环里放后台、循环之后 wait", "for part in a b; do bash $part.sh & done; wait", True, 0),
        ("bash -c 里 & 加 wait", "bash -c 'bash a.sh & bash b.sh & wait'", True, 0),
        ("圆括号里 & 加 wait", "(bash a.sh & wait) > both.log", True, 0),
        ("if 里 & 之后 wait", "if true; then bash a.sh & wait; fi", True, 0),
        ("数组赋值的括号不打乱圆括号组", "parts=(a b); bash a.sh & wait", True, 0),
        ("几个 & 只 wait 一个 pid：剩下的照拒", "bash a.sh & bash b.sh & wait $!", True, 1),
        ("引号里当数据写的 &", "echo 'bash long.sh &' > note.txt", True, 0),
        ("写进文件的 heredoc 正文里的 &", "cat > note.md <<'EOF'\nbash long.sh &\nEOF", True, 0),
        ("注释里的 &", "ls  # bash long.sh &", True, 0),
        ("前台：结尾单独一个 &", "bash long.sh > long.log 2>&1 &", False, 0),
        ("前台：& 之后接 echo $!", "bash long.sh > long.log 2>&1 & echo $!", False, 0),
    ]
    for label, command, background, want in unwaited_ampersand_cases:
        results.append((f"单独的 &:{label}", want, 1 if unwaited_ampersand_refusal(command, background) else 0))
    # ⑥ 终止进程：(说明, 命令, 该不该拒绝)；写死的进程号对着一张造出来的进程表判，这条命令的会话是 claude 1000（祖先 1001 → 1000 → 900 → 800 → 700 → 1）。
    # 每条命令只拿来判，一条都不执行
    fake_table = {
        1: {"parent": 0, "name": "systemd", "command_line": "/sbin/init"},
        700: {"parent": 1, "name": "sshd", "command_line": "sshd: /usr/sbin/sshd -D"},
        800: {"parent": 700, "name": "sshd-session", "command_line": "sshd-session: fy5090"},
        900: {"parent": 800, "name": "MainThread", "command_line": "/home/u/.vscode-server/cli/servers/Stable/server/node --dns-result-order=ipv4first"},
        1000: {"parent": 900, "name": "claude", "command_line": "/home/u/.vscode-server/extensions/anthropic.claude-code/native-binary/claude"},
        1001: {"parent": 1000, "name": "bash", "command_line": "/bin/bash -c source snapshot.sh"},
        1100: {"parent": 1001, "name": "cargo", "command_line": "cargo test -p singlefs-core --lib"},
        2000: {"parent": 900, "name": "claude", "command_line": "/home/u/.vscode-server/extensions/anthropic.claude-code/native-binary/claude"},
        2100: {"parent": 2000, "name": "bash", "command_line": "/bin/bash -c bash gate.sh"},
        2101: {"parent": 2100, "name": "cargo", "command_line": "cargo test --release"},
        3000: {"parent": 1, "name": "python3", "command_line": "python3 -m selftest_model_server.entrypoints.api_server --model x"},
        4000: {"parent": 1, "name": "sleep", "command_line": "sleep 600"},
    }
    fake_reader = lambda: fake_table  # noqa: E731
    # 本地模型服务的单元与进程标记走自证自己的私有配置（不读本机那一份）；跑完这一段换回原来的环境
    private_config = os.path.join(work, "private.env")
    with open(private_config, "w", encoding="utf-8") as handle:
        handle.write("# 自证用的私有配置\nOTHER_KEY=selftest-other\nLOCAL_MODEL_SERVICE_UNITS=selftest-model-server, other-model.service\n"
                     "LOCAL_MODEL_SERVICE_PROCESS_MARKERS='selftest_model_server'\n")
    private_config_before = os.environ.get("BASH_COMMAND_DETECTOR_PRIVATE_CONFIG")
    os.environ["BASH_COMMAND_DETECTOR_PRIVATE_CONFIG"] = private_config
    process_signal_cases = [
        ("kill -- 负进程号（进程组）", "kill -- -1234", 1),
        ("kill -TERM 负进程号", "kill -TERM -1234", 1),
        ("kill -s TERM 负进程号", "kill -s TERM -1234", 1),
        ("自己的任务的进程组 -\"$job\"", 'kill -INT -- -"$job"', 1),
        ("kill 0", "kill 0", 1),
        ("kill -TERM 0", "kill -TERM 0", 1),
        ("kill -9 -1（全部）", "kill -9 -1", 1),
        ("kill $PPID（Claude 会话本身）", "kill -TERM $PPID", 1),
        ("kill $(pgrep …)", "kill -TERM $(pgrep cargo)", 1),
        ("kill $(jobs -p)", "kill $(jobs -p)", 1),
        ("kill `pidof sshd`", "kill -HUP `pidof sshd`", 1),
        ("kill 数组整批展开", 'kill "${worker_pids[@]}"', 1),
        ("kill \"$@\"", 'kill "$@"', 1),
        ("kill 一次两个", 'kill "$first" "$second"', 1),
        ("kill %1 %2", "kill %1 %2", 1),
        ("kill 带通配", "kill /proc/[0-9]*", 1),
        ("for 循环里 kill", 'for pid in $(cat pids.txt); do kill "$pid"; done', 1),
        ("while read cgroup.procs 里 kill（现场那一段）",
         'while read -r process_id; do [[ "$process_id" != "$BASHPID" ]] && kill -TERM "$process_id" 2>/dev/null; done < /sys/fs/cgroup/user.slice/user-1000.slice/session-1.scope/cgroup.procs', 1),
        ("多行循环里 kill", 'for pid in "${pids[@]}"\ndo\n  kill -TERM "$pid"\ndone', 1),
        ("cgroup.procs 交给 xargs kill", "cat /sys/fs/cgroup/x/cgroup.procs | xargs kill", 1),
        ("xargs -n1 kill", "xargs -n1 kill < pids.txt", 1),
        ("pgrep 交给 xargs -r kill -9", "pgrep -u fy5090 node | xargs -r kill -9", 1),
        ("find -exec kill", "find /proc -maxdepth 1 -name '[0-9]*' -exec kill {} +", 1),
        ("pkill 按名字", "pkill cargo", 1),
        ("pkill -x claude", "pkill -x claude", 1),
        ("killall", "killall node", 1),
        ("fuser -k", "fuser -k 8200/tcp", 1),
        ("先从 cgroup.procs 挑一个再 kill", 'p=$(head -1 /sys/fs/cgroup/x/cgroup.procs); kill -TERM "$p"', 1),
        ("先 pgrep 挑一个再 kill", 'p=$(pgrep -n node); kill "$p"', 1),
        ("python -c 推导式里读 cgroup.procs 再 os.kill",
         "python3 -c 'import os; [os.kill(int(p), 15) for p in open(\"/sys/fs/cgroup/x/cgroup.procs\")]'", 1),
        ("喂给 python 的 heredoc 里 os.killpg", "python3 - <<'EOF'\nimport os, signal\nos.killpg(os.getpgid(0), signal.SIGTERM)\nEOF", 1),
        ("python -c os.kill(0, …)", "python3 -c 'import os; os.kill(0, 15)'", 1),
        ("systemctl stop ssh", "systemctl stop ssh", 1),
        ("sudo systemctl restart sshd", "sudo systemctl restart sshd", 1),
        ("systemctl kill session-1.scope", "systemctl kill session-1.scope", 1),
        ("systemctl stop user@1000.service", "systemctl stop user@1000.service", 1),
        ("systemctl set-property user.slice", "systemctl set-property user.slice MemoryMax=1G", 1),
        ("systemctl stop 私有配置登记的本地模型服务（不写后缀）", "systemctl stop selftest-model-server", 1),
        ("systemctl restart 私有配置登记的第二个单元", "systemctl restart other-model.service", 1),
        ("systemctl disable --now ssh", "systemctl disable --now ssh", 1),
        ("systemctl --user stop 带通配 singlefs-*", "systemctl --user stop 'singlefs-*'", 1),
        ("systemctl --user kill 带通配 *.scope", "systemctl --user kill '*.scope'", 1),
        ("systemctl --user stop 命令替换列出来的一批", "systemctl --user stop $(systemctl --user list-units --plain 'singlefs*' | awk '{print $1}')", 1),
        ("systemctl isolate", "systemctl isolate multi-user.target", 1),
        ("systemctl --user exit", "systemctl --user exit", 1),
        ("systemctl reboot", "systemctl reboot", 1),
        ("loginctl terminate-session", "loginctl terminate-session 1", 1),
        ("loginctl kill-user", "loginctl kill-user fy5090", 1),
        ("往 cgroup.kill 写", "echo 1 > /sys/fs/cgroup/user.slice/user-1000.slice/session-1.scope/cgroup.kill", 1),
        ("tee 进 cgroup.freeze", "echo 1 | tee /sys/fs/cgroup/x/cgroup.freeze", 1),
        ("sudo reboot", "sudo reboot", 1),
        ("shutdown -h now", "shutdown -h now", 1),
        ("写死的进程号是 sshd", "kill -TERM 700", 1),
        ("写死的进程号是 VSCode 服务端", "kill 900", 1),
        ("写死的进程号是这个会话自己的 Claude", "kill -9 1000", 1),
        ("写死的进程号是另一个 Claude 会话", "kill 2000", 1),
        ("写死的进程号是另一个 Claude 会话起的 cargo", "kill 2101", 1),
        ("写死的进程号是私有配置登记的本地模型服务", "kill 3000", 1),
        ("kill -HUP 1", "kill -HUP 1", 1),
        ("proc.py stop 写死的 sshd", "python3 .claude/singlefs-ai-sop/scripts/proc.py stop 700", 1),
        ("proc.py stop 一次两个", "python3 .claude/singlefs-ai-sop/scripts/proc.py stop 1100 4000", 1),
        ("循环里 proc.py stop", 'for p in 1100 4000; do python3 .claude/singlefs-ai-sop/scripts/proc.py stop "$p"; done', 1),
        ("bash -c 里 kill 进程组", "bash -c 'kill -TERM -- -$$'", 1),
        ("nice 包着 kill -9 -1", "nice -n 19 kill -9 -1", 1),
        ("喂给 bash 的 heredoc 里 kill 0", "bash <<'EOF'\nkill 0\nEOF", 1),
        ("timeout 包着 kill 负进程号", "timeout 5 kill -TERM -4000", 1),
        ("run-with-memory-cap.sh 包着 pkill", "bash scripts/hosts/run-with-memory-cap.sh 1G pkill cargo", 1),
        ("shell 函数与变量里的脚本包着的 bash -c 里 kill 0", 'in_context bash "$target" 64M bash -c \'kill 0\'', 1),
        ("shell 函数包着的 python -c 里 os.killpg", "in_context python3 -c 'import os; os.killpg(1234, 15)'", 1),
        # 自己用新会话起的子进程的进程组放行，这四格照拒（OWN_GROUP_TARGET 与 SESSION_CHILD_CALLS 两段注释）：
        # own-session-group-session-unchecked 下「不是 setsid 起的」「没开新会话的」两格红，own-session-group-first-binding-only 下「改写过」「改绑过」两格红
        ("不是 setsid 起的后台任务的进程组 -\"$leader\"", 'sleep 30 & leader=$!; kill -TERM -- -"$leader"', 1),
        ("setsid 起的后台任务的进程号又被改写过 -\"$leader\"", 'setsid --wait sleep 30 & leader=$!; leader="$other_group"; kill -TERM -- -"$leader"', 1),
        ("python -c 对没开新会话的子进程的进程组 os.killpg",
         "python3 -c 'import os, subprocess; child = subprocess.Popen([\"sleep\", \"30\"]); os.killpg(child.pid, 15)'", 1),
        ("python -c 子进程的名字又被改绑过的 os.killpg",
         "python3 -c 'import os, subprocess; child = subprocess.Popen([\"sleep\", \"30\"], start_new_session=True); child = other; os.killpg(child.pid, 15)'", 1),
        # 放行的两格在 own-session-group-unrecognized 下红
        ("setsid 起的后台任务的进程组 -\"$leader\"", 'setsid --wait sleep 30 & leader=$!; kill -TERM -- -"$leader"', 0),
        ("python -c 对 start_new_session=True 起的子进程的进程组 os.killpg 与 os.kill(-组号)",
         "python3 -c 'import os, subprocess; child = subprocess.Popen([\"sleep\", \"30\"], start_new_session=True); os.killpg(os.getpgid(child.pid), 15); os.kill(-child.pid, 9)'", 0),
        ("点名一个 kill \"$!\"", 'kill "$!"', 0),
        ("点名一个 kill -TERM \"$pid\"", 'kill -TERM "$pid"', 0),
        ("点名一个 kill -9 $worker", "kill -9 $worker", 0),
        ("点名一个任务号 kill %1", "kill %1", 0),
        ("点名一个任务号 kill %%", "kill %%", 0),
        ("点名一个任务号 kill -INT %2", "kill -INT %2", 0),
        ("kill -0 探活", 'kill -0 "$pid"', 0),
        ("kill -s 0 探活写死的进程号", "kill -s 0 700", 0),
        ("kill -l", "kill -l 15", 0),
        ("写死的进程号是这个会话起的 cargo", "kill 1100", 0),
        ("写死的进程号是没人认领的孤儿", "kill 4000", 0),
        ("写死的进程号不存在", "kill 99999999", 0),
        ("proc.py stop 一个写死的", "python3 .claude/singlefs-ai-sop/scripts/proc.py stop 1100", 0),
        ("proc.py stop 一个变量带 --grace", 'python3 .claude/singlefs-ai-sop/scripts/proc.py stop "$pid" --grace 5', 0),
        ("按 pid 轮询的 while kill -0", 'while kill -0 "$pid" 2>/dev/null; do sleep 5; done', 0),
        ("先核进程再停一个", '[[ "$(ps -p "$pid" -o args=)" == *mutate* ]] && kill "$pid"', 0),
        ("grep 参数里的 kill 字样", "grep -n 'kill -TERM -1' notes.md", 0),
        ("echo 里的 pkill", "echo 'pkill cargo'", 0),
        ("写进文件的 heredoc 正文", "cat > note.md <<'EOF'\nkill 0\nfor p in $(pgrep x); do kill $p; done\nEOF", 0),
        ("git commit 说明里写 kill 0", "git commit -m 'kill 0 的拒绝'", 0),
        ("systemctl --user stop 自己的 slice", "systemctl --user stop singlefs_memory_selftest_123_1.slice", 0),
        ("systemctl status sshd", "systemctl --user status sshd; systemctl status selftest-model-server", 0),
        ("systemctl stop 私有配置里只在别的键下出现的单元", "systemctl stop selftest-other", 0),
        ("systemctl show 与 set-property 自己的 slice", "systemctl --user show -p ControlGroup --value singlefs-heavy.slice && systemctl --user set-property --runtime singlefs-heavy.slice MemoryMax=40G", 0),
        ("systemctl list-units 带通配", "systemctl --user list-units 'singlefs*'", 0),
        ("systemctl reset-failed 自己的 scope", "systemctl --user reset-failed singlefs-memory-cap-1-2.scope", 0),
        ("loginctl list-sessions", "loginctl list-sessions; loginctl show-session 1", 0),
        ("只读 cgroup.procs", "cat /sys/fs/cgroup/user.slice/user-1000.slice/session-1.scope/cgroup.procs", 0),
        ("只读 cgroup.kill", "cat /sys/fs/cgroup/x/cgroup.kill", 0),
        ("只 pgrep 不发信号", "pgrep -u fy5090 cargo; ps -o pid,args --ppid 1100", 0),
        ("python -c os.kill 探活", "python3 -c 'import os; os.kill(1100, 0)'", 0),
        ("python -c 停一个 pid 文件里的", "python3 -c 'import os,signal; os.kill(int(open(\"worker.pid\").read()), signal.SIGTERM)'", 0),
        ("timeout -s KILL 包着命令", "timeout -s KILL 60 cargo test -p singlefs-core --lib", 0),
        ("shutdown -c", "shutdown -c", 0),
    ]
    for label, command, want in process_signal_cases:
        results.append((f"终止进程:{label}", want, 1 if process_signal_refusal(command, fake_reader, 1001) else 0))
    # 进程表按需读：只有判到 kill / proc.py stop 的写死的进程号才读，读一次；BASH_COMMAND_DETECTOR_BREAK=always-read-process-table（命令里有数字就先读）下
    # 「没有终止词的带数字命令不读进程表」一格红
    table_reads = []
    def counting_reader():
        table_reads.append(1)
        return fake_table
    def reads_for(*commands):
        del table_reads[:]
        for one_command in commands:
            process_signal_refusal(one_command, counting_reader, 1001)
        return len(table_reads)
    results.append(("终止进程:没有终止词的带数字命令不读进程表（git log -1、sleep 2、--threads 8）", 0,
                    reads_for("git log -1", "sleep 2", "cargo test -p singlefs-core --threads 8")))
    results.append(("终止进程:kill 写死的进程号读一次进程表", 1, reads_for("kill 4000")))
    results.append(("终止进程:proc.py stop 写死的进程号读一次进程表", 1, reads_for("python3 .claude/singlefs-ai-sop/scripts/proc.py stop 4000")))
    results.append(("终止进程:一条命令里几处写死的进程号共读一次进程表", 1, reads_for("kill 4000; kill 1100")))
    results.append(("终止进程:kill 的目标不是写死的进程号不读进程表", 0, reads_for('kill "$pid"')))
    # 私有配置不在、没有那两个键：本地模型服务那一项不判、不报错，别的照判
    with open(os.path.join(work, "private-keyless.env"), "w", encoding="utf-8") as handle:
        handle.write("# 自证用的私有配置，没有那两个键\nOTHER_KEY=selftest-model-server selftest_model_server\n")
    for private_case, private_path in (("私有配置文件不在", os.path.join(work, "private-absent.env")),
                                       ("私有配置里没有那两个键", os.path.join(work, "private-keyless.env"))):
        os.environ["BASH_COMMAND_DETECTOR_PRIVATE_CONFIG"] = private_path
        for label, command, want in (("systemctl stop 没登记的单元放行", "systemctl stop selftest-model-server", 0),
                                     ("写死的进程号没登记标记放行", "kill 3000", 0),
                                     ("systemctl stop ssh 照拒", "systemctl stop ssh", 1)):
            results.append((f"终止进程:{private_case}:{label}", want, 1 if process_signal_refusal(command, fake_reader, 1001) else 0))
    if private_config_before is None:
        os.environ.pop("BASH_COMMAND_DETECTOR_PRIVATE_CONFIG", None)
    else:
        os.environ["BASH_COMMAND_DETECTOR_PRIVATE_CONFIG"] = private_config_before
    # 扫脚本（门禁 code-tooling 的 research-gate-lint 格那一路）：临时目录里造脚本，走 --scan-scripts 真实入口，看红绿与报的行号
    scan_root = os.path.join(work, "scan-root")
    scan_scripts_directory = os.path.join(scan_root, "scripts")
    os.makedirs(scan_scripts_directory)
    os.makedirs(os.path.join(scan_root, ".claude"))
    scan_files = {
        "group.sh": '#!/usr/bin/env bash\nset -m\nsleep 30 &\njob=$!\nkill -INT -- -"$job"\n',
        "loop.sh": '#!/usr/bin/env bash\nfor pid in "${worker_pids[@]}"; do\n  kill -TERM "$pid"\ndone\n',
        "storm-guarded.sh": ('#!/usr/bin/env bash\ngroup="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"\n'
                             'case "$group" in */singlefs-memory-cap-*.scope) ;; *) exit 0 ;; esac\n'
                             'while read -r process_id; do kill -TERM "$process_id"; done < "$group/cgroup.procs"  # process-safety:own-scope 上一行 case 核过是自己开的 scope\n'),
        "storm-unguarded.sh": ('#!/usr/bin/env bash\ngroup="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"\n'
                               'while read -r process_id; do kill -TERM "$process_id"; done < "$group/cgroup.procs"  # process-safety:own-scope 说是核过了但没核\n'),
        "wrong-kind.sh": '#!/usr/bin/env bash\n# singlefs-memory-cap-*.scope\nkill -TERM -- -"$job"  # process-safety:own-scope 进程组没有例外\n',
        "stale-annotation.sh": '#!/usr/bin/env bash\n# singlefs-memory-cap-*.scope\necho hi  # process-safety:own-scope 这一行没有发信号\n',
        "clean.sh": ('#!/usr/bin/env bash\nsleep 30 &\nworker=$!\nkill -0 "$worker" && kill -TERM "$worker"\n'
                     'while kill -0 "$worker" 2>/dev/null; do sleep 1; done\npython3 .claude/singlefs-ai-sop/scripts/proc.py stop "$worker"\n'
                     'for slice in "${slices[@]}"; do systemctl --user stop "$slice"; done\n'
                     "python3 - <<'EOF'\nimport os\nos.kill(int(open('worker.pid').read()), 15)\nEOF\n"),
        "bad.py": "import os, signal\nfor pid in pids:\n    os.kill(pid, signal.SIGKILL)\nos.killpg(0, signal.SIGTERM)\n",
        "good.py": "import os\nos.kill(pid, 0)\nprocess.kill()\n",
        "docs.sh": "#!/usr/bin/env bash\n# 那一行写 `# process-safety:own-scope <怎么核的>`\ncat > note.md <<'EOF'\nkill 0  # process-safety:own-scope 写进文件的正文里的不算\nEOF\n",
        "docs.py": 'NOTE = "kill 0  # process-safety:own-scope 写在字符串里的不算"\n',
    }
    for file_name, content in scan_files.items():
        with open(os.path.join(scan_scripts_directory, file_name), "w", encoding="utf-8") as handle:
            handle.write(content)
    def scan(root):
        return subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--scan-scripts", root, os.path.join(root, "scripts")],
                              capture_output=True, text=True)
    scanned = scan(scan_root)
    results.append(("扫脚本:有红就退出码 1", 1, scanned.returncode))
    for label, marker in [("自己任务的进程组报在第 5 行", "scripts/group.sh:5"), ("循环里 kill 报在第 3 行", "scripts/loop.sh:3"),
                          ("标了 own-scope 而没核 scope", "scripts/storm-unguarded.sh:3"), ("进程组标 own-scope 不放行", "scripts/wrong-kind.sh:3"),
                          ("不起作用的 own-scope 标注", "scripts/stale-annotation.sh:3"), ("python 循环里 os.kill", "scripts/bad.py:3"),
                          ("python os.killpg", "scripts/bad.py:4")]:
        results.append((f"扫脚本:{label}", 1, int(f"✗ {marker} " in scanned.stdout)))
    results.append(("扫脚本:核过 scope 又标了的、点名一个的、探活的、python 探活的都不报", 0,
                    int(any(f"scripts/{name}" in scanned.stdout for name in ("storm-guarded.sh", "clean.sh", "good.py", "docs.sh", "docs.py")))))
    results.append(("扫脚本:拒绝带出路", 1, int("→ 怎么办：只停自己起的、点名的那一个" in scanned.stdout)))
    # 自己用新会话起的子进程的进程组（OWN_GROUP_TARGET 与 SESSION_CHILD_CALLS 两段注释）：另一个扫描根，上面那一组格数的脚本个数不动。
    # own-session-group-unrecognized 下「不报」那一格红；own-session-group-session-unchecked 下「没用 setsid」「setsid --fork」「没开新会话」三格红；
    # own-session-group-first-binding-only 下两格「改写过」「改绑过」红
    group_scan_root = os.path.join(work, "scan-group-root")
    os.makedirs(os.path.join(group_scan_root, "scripts"))
    os.makedirs(os.path.join(group_scan_root, ".claude"))
    group_scan_files = {
        "own-group.sh": ('#!/usr/bin/env bash\nsetsid --wait sleep 30 &\nleader=$!\nkill -TERM -- -"$leader"\nsleep 1\nkill -KILL -- -"${leader}"\n'),
        "own-group.py": ("import asyncio\nimport os\nimport signal\nimport subprocess\n\n\n"
                         "def start_worker(command):\n    return subprocess.Popen(command, start_new_session=True)\n\n\n"
                         "def stop_direct(command):\n    process = subprocess.Popen(command, start_new_session=True)\n"
                         "    os.killpg(process.pid, signal.SIGTERM)\n    os.killpg(os.getpgid(process.pid), signal.SIGKILL)\n\n\n"
                         "def stop_through_helper(command):\n    worker = start_worker(command)\n    group = os.getpgid(worker.pid)\n"
                         "    for attempt in range(3):\n        os.killpg(group, signal.SIGTERM)\n    os.kill(-worker.pid, signal.SIGKILL)\n\n\n"
                         "def stop_in_context(command):\n    with subprocess.Popen(command, process_group=0) as runner:\n"
                         "        def stop_later():\n            os.killpg(runner.pid, signal.SIGTERM)\n        stop_later()\n\n\n"
                         "async def stop_asynchronous(command):\n    child = await asyncio.create_subprocess_exec(*command, start_new_session=True)\n"
                         "    os.killpg(child.pid, signal.SIGTERM)\n"),
        "other-group.sh": ('#!/usr/bin/env bash\nsleep 30 &\nleader=$!\nkill -TERM -- -"$leader"\n'
                           'setsid --wait sleep 30 &\nleader=$!\nleader="$other_group"\nkill -INT -- -"$leader"\n'
                           'setsid --fork sleep 30 &\nforked=$!\nkill -HUP -- -"$forked"\nkill -TERM -- -1234\n'),
        "other-group.py": ("import os\nimport signal\nimport subprocess\n\n\n"
                           "def stop_given(process):\n    os.killpg(process.pid, signal.SIGTERM)\n\n\n"
                           "def stop_plain(command):\n    process = subprocess.Popen(command)\n    os.killpg(process.pid, signal.SIGTERM)\n\n\n"
                           "def stop_rebound(command, other):\n    process = subprocess.Popen(command, start_new_session=True)\n    process = other\n"
                           "    os.killpg(process.pid, signal.SIGTERM)\n\n\n"
                           "def stop_own_group():\n    os.killpg(os.getpgid(0), signal.SIGTERM)\n\n\n"
                           "def stop_picked():\n    group = int(open(\"/sys/fs/cgroup/x/cgroup.procs\").readline())\n    os.killpg(group, signal.SIGTERM)\n\n\n"
                           "class Runner:\n    def stop(self):\n        os.killpg(self.process.pid, signal.SIGTERM)\n\n\n"
                           "os.kill(-1, signal.SIGTERM)\n"),
    }
    for file_name, content in group_scan_files.items():
        with open(os.path.join(group_scan_root, "scripts", file_name), "w", encoding="utf-8") as handle:
            handle.write(content)
    group_scanned = scan(group_scan_root)
    results.append(("扫脚本:进程组:有红就退出码 1", 1, group_scanned.returncode))
    results.append(("扫脚本:进程组:自己用 setsid 或 start_new_session=True 起的子进程的组不报", 0,
                    int(any(f"scripts/{name}" in group_scanned.stdout for name in ("own-group.sh", "own-group.py")))))
    for label, marker in [("没用 setsid 起的后台任务的组", "scripts/other-group.sh:4"), ("setsid 起的进程号又被改写过", "scripts/other-group.sh:8"),
                          ("setsid --fork 起的", "scripts/other-group.sh:11"), ("负的写死号", "scripts/other-group.sh:12"),
                          ("python 参数传进来的子进程", "scripts/other-group.py:7"), ("python 没开新会话的子进程", "scripts/other-group.py:12"),
                          ("python 子进程的名字又被改绑过", "scripts/other-group.py:18"), ("python os.getpgid(0)", "scripts/other-group.py:22"),
                          ("python 从 cgroup.procs 读来的组", "scripts/other-group.py:27"), ("python 属性上的子进程 self.process", "scripts/other-group.py:32"),
                          ("python os.kill(-1, …)", "scripts/other-group.py:35")]:
        results.append((f"扫脚本:进程组:{label}", 1, int(f"✗ {marker} " in group_scanned.stdout)))
    # 弄坏开关只在自证里认：平常跑（不带标记）设了就退 2 并点名变量；自证起的子进程（带标记）照认。break-honored-outside-selftest 设着时探针只写它一项，第一格红
    unmarked_environment = {key: value for key, value in os.environ.items() if key != SELFTEST_MARK_VARIABLE}
    probe_items = GUARD_OFF_ITEM if detector_break_is_set(GUARD_OFF_ITEM) else "disable-check"   # 关守卫项只在单独一项时认：开关设着时探针只写它一项
    plain_run = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                               env=dict(unmarked_environment, **{DETECTOR_BREAK_VARIABLE: probe_items, "AGENT_HOOK_DETECTIONS": os.path.join(work, "probe-detections.jsonl")}))
    results.append(("平常跑设了弄坏开关：退 2 并点名只在自证里认", (2, True), (plain_run.returncode, DETECTOR_BREAK_VARIABLE in plain_run.stderr and "自证" in plain_run.stderr)))
    # 它导入的共用库的弄坏开关变量（LIBRARY_BREAK_VARIABLES）平常设了同样退 2：探针不带 BASH_COMMAND_DETECTOR_BREAK；关守卫项设着时探针只写它一项（守卫关着，库的变量也放过去），这一格红
    library_plain_environment = {key: value for key, value in unmarked_environment.items() if key != DETECTOR_BREAK_VARIABLE}
    library_plain_environment[LIBRARY_BREAK_VARIABLES[0]] = "entry-form-unchecked"
    if detector_break_is_set(GUARD_OFF_ITEM):
        library_plain_environment[DETECTOR_BREAK_VARIABLE] = GUARD_OFF_ITEM
    library_plain_run = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                       env=dict(library_plain_environment, AGENT_HOOK_DETECTIONS=os.path.join(work, "probe-detections.jsonl")))
    results.append(("平常跑设了共用库的弄坏开关 LIB_CODE_DISCIPLINE_LINTS_BREAK：退 2 并点名只在自证里认", (2, True),
                    (library_plain_run.returncode, LIBRARY_BREAK_VARIABLES[0] in library_plain_run.stderr and "自证" in library_plain_run.stderr)))
    marked_run = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh")], capture_output=True, text=True,
                                input=json.dumps({"tool_name": "Bash", "tool_input": {"command": 'until grep -q "^exit=" log; do sleep 10; done', "run_in_background": False}}),
                                env=dict(os.environ, **{DETECTOR_BREAK_VARIABLE: "disable-check", "AGENT_HOOK_DETECTIONS": os.path.join(work, "probe-detections.jsonl")}))
    results.append(("自证起的子进程（带标记）照认弄坏开关", 0, marked_run.returncode))
    # 关守卫项与别的项同写：平常跑与带标记的子进程都退 2、点名它只在单独一项时认；开关 break-honored-outside-selftest-with-others 设着时带标记那一格的开关里也带上它（同写照认），那一格红
    combined_items = f"{GUARD_OFF_ITEM},disable-check"
    combined_plain = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                    env=dict(unmarked_environment, **{DETECTOR_BREAK_VARIABLE: combined_items, "AGENT_HOOK_DETECTIONS": os.path.join(work, "probe-detections.jsonl")}))
    results.append(("关守卫项与别的项同写：平常跑退 2 并点名它只在单独一项时认", (2, True),
                    (combined_plain.returncode, GUARD_OFF_ITEM in combined_plain.stderr and "唯一一项" in combined_plain.stderr)))
    lenient_items = combined_items + (f",{GUARD_OFF_WITH_OTHERS_ITEM}" if detector_break_is_set(GUARD_OFF_WITH_OTHERS_ITEM) else "")
    combined_marked = subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                     env=dict(os.environ, **{DETECTOR_BREAK_VARIABLE: lenient_items, "AGENT_HOOK_DETECTIONS": os.path.join(work, "probe-detections.jsonl")}))
    results.append(("关守卫项与别的项同写：带标记的子进程里也退 2 并点名它", (2, True),
                    (combined_marked.returncode, GUARD_OFF_ITEM in combined_marked.stderr and "唯一一项" in combined_marked.stderr)))
    pending_path = os.path.join(scan_root, ".claude", "process-safety-pending")
    for file_name in ("storm-unguarded.sh", "wrong-kind.sh", "stale-annotation.sh"):
        os.remove(os.path.join(scan_scripts_directory, file_name))
    with open(pending_path, "w", encoding="utf-8") as handle:
        handle.write("# 待改\nscripts/group.sh  # 自检：别人正在改\nscripts/loop.sh  # 自检：别人正在改\nscripts/bad.py  # 自检：别人正在改\n")
    pending_green = scan(scan_root)
    results.append(("扫脚本:红的都登记进待改清单就退出码 0", 0, pending_green.returncode))
    results.append(("扫脚本:成功行报查了几个、放行了哪一处、逐个列出没判的", 1,
                    int("查了 8 个脚本（.sh 5 个、.py 3 个）" in pending_green.stdout and "scripts/storm-guarded.sh:4" in pending_green.stdout
                        and "scripts/group.sh（scripts/group.sh:5）" in pending_green.stdout)))
    with open(pending_path, "a", encoding="utf-8") as handle:
        handle.write("scripts/clean.sh  # 自检：干净的也登记了\nscripts/gone.sh  # 自检：指向不存在的文件\n")
    pending_stale = scan(scan_root)
    results.append(("扫脚本:待改清单里一处都没排到的与指向不存在的判红", 1,
                    int(pending_stale.returncode == 1 and "clean.sh 里一处要放行的都没有" in pending_stale.stdout and "gone.sh 指向不存在的文件" in pending_stale.stdout)))
    # 待改清单里的行所在的目录这一趟没扫（拆分提案第九十节 ㉝ d）：只扫 scripts/ 这一层，清单另有一行落在 scripts/deeper/ 下；没扫就不知道它还有没有要放行的，判本次未判
    os.makedirs(os.path.join(scan_scripts_directory, "deeper"))
    with open(os.path.join(scan_scripts_directory, "deeper", "killer.sh"), "w", encoding="utf-8") as handle:
        handle.write('#!/usr/bin/env bash\nfor pid in "${worker_pids[@]}"; do\n  kill -TERM "$pid"\ndone\n')
    with open(pending_path, "w", encoding="utf-8") as handle:
        handle.write("# 待改\nscripts/group.sh  # 自检：别人正在改\nscripts/loop.sh  # 自检：别人正在改\nscripts/bad.py  # 自检：别人正在改\n"
                     "scripts/deeper/killer.sh  # 自检：别人正在改，这一趟没扫它所在的目录\n")
    not_run_marker = os.path.join(work, "not-run-lines")
    open(not_run_marker, "w").close()
    def scan_directories(directories, extra_environment=None):
        return subprocess.run(["bash", os.path.join(hook_dir, "bash-command-detector.sh"), "--scan-scripts", scan_root, *directories],
                              capture_output=True, text=True, env=dict(os.environ, **(extra_environment or {})))
    outside_scanned = scan_directories([scan_scripts_directory], {"GATE_NOT_RUN_FILE": not_run_marker})
    results.append(("扫脚本:待改清单里的行所在的目录这一趟没扫：退出码 0、报本次未判", (0, True, False),
                    (outside_scanned.returncode,
                     "本次未判" in outside_scanned.stdout and "scripts/deeper/killer.sh" in outside_scanned.stdout,
                     "scripts/deeper/killer.sh 里一处要放行的都没有" in outside_scanned.stdout)))
    results.append(("扫脚本:待改清单里的行所在的目录这一趟没扫：往 GATE_NOT_RUN_FILE 报一行", 1,
                    sum(1 for line in open(not_run_marker, encoding="utf-8") if "scripts/deeper/killer.sh" in line and "本次未判" in line)))
    both_scanned = scan_directories([scan_scripts_directory, os.path.join(scan_scripts_directory, "deeper")])
    results.append(("扫脚本:待改清单里的行所在的目录这一趟也扫了：照常列进没判的、不报本次未判", (0, True, False),
                    (both_scanned.returncode, "scripts/deeper/killer.sh（scripts/deeper/killer.sh:3）" in both_scanned.stdout, "本次未判" in both_scanned.stdout)))
    # 整份覆盖 research/results/ 下未跟踪的产物：在临时 git 仓里造已跟踪与未跟踪的文件，判定与真实入口都指着它，不碰真仓
    repository = os.path.realpath(os.path.join(work, "repository"))
    os.makedirs(os.path.join(repository, "research", "results", "sub"))
    elsewhere = os.path.join(work, "elsewhere")        # 仓外同样的相对路径：cd 过去之后写的是它，不是仓里那份
    sources = os.path.join(work, "sources")            # cp 进目录时的源，文件名与仓里未跟踪的那份相同
    not_a_repository = os.path.join(work, "not-a-repository")
    for directory in (os.path.join(elsewhere, "research", "results"), sources, os.path.join(not_a_repository, "research", "results")):
        os.makedirs(directory)
    for path in (*(os.path.join(repository, relative) for relative in (
                     "research/results/untracked.out", "research/results/tracked.out", "research/results/1",
                     "research/results/sub/deep.out", "scratch.log")),
                 os.path.join(elsewhere, "research", "results", "untracked.out"), os.path.join(sources, "untracked.out"),
                 os.path.join(work, "outside.log"), os.path.join(not_a_repository, "research", "results", "untracked.out")):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("旧字节\n")
    git_ready = (subprocess.run(["git", "init", "-q", repository], capture_output=True).returncode == 0
                 and subprocess.run(["git", "-C", repository, "add", "research/results/tracked.out"], capture_output=True).returncode == 0)
    results.append(("覆盖产物:临时 git 仓建得起来（git init、git add）", 1, int(git_ready)))
    outside = os.path.join(work, "outside.log")
    # (说明, 命令, 该不该拒绝)；目标按临时仓的根解析
    results_overwrite_cases = [
        ("> 覆盖未跟踪的产物", "python3 x.py > research/results/untracked.out", 1),
        ("tee 不带 -a", "cargo run --release | tee research/results/untracked.out", 1),
        ("cp 的目标（现场原句的形态）", "cp /tmp/claude-1000/e158-s9/config-evidence-runs/yi-ring-retained.out research/results/untracked.out", 1),
        ("bash -c 里的 >", "bash -c 'python3 x.py > research/results/untracked.out'", 1),
        (">|", "python3 x.py >| research/results/untracked.out", 1),
        ("&>", "cargo test &> research/results/untracked.out", 1),
        ("2> 同样截断", "cargo test 2> research/results/untracked.out", 1),
        (">& 后面跟文件名", "cargo test >& research/results/untracked.out", 1),
        ("tee -i 不是追加", "make | tee -i research/results/untracked.out", 1),
        ("tee 写两份、其中一份是未跟踪的", "make | tee research/results/fresh.out research/results/untracked.out", 1),
        ("mv 的目标", "mv /tmp/fresh.out research/results/untracked.out", 1),
        ("install 的目标", "install -m 644 fresh.out research/results/untracked.out", 1),
        ("cp 进已有目录、源的文件名撞上", f"cp {sources}/untracked.out research/results/", 1),
        ("cp -t 目录", f"cp -t research/results {sources}/untracked.out", 1),
        ("cp 源里的通配按这一刻展开", f"cp {sources}/*.out research/results", 1),
        ("cd 进去之后写相对路径", "cd research/results && python3 x.py > untracked.out", 1),
        ("cd 进子目录之后写", "cd research/results/sub && echo > deep.out", 1),
        ("绝对路径", f"echo > {repository}/research/results/untracked.out", 1),
        ("喂给 bash 的 heredoc 正文里的 >", "bash <<'EOF'\npython3 x.py > research/results/untracked.out\nEOF", 1),
        ("命令替换里的 >", "lines=$(python3 x.py > research/results/untracked.out; echo done)", 1),
        ("cat 把 heredoc 写进未跟踪的产物", "cat > research/results/untracked.out <<'EOF'\n一行\nEOF", 1),
        ("花括号整组重定向", "{ echo a; echo b; } > research/results/untracked.out", 1),
        ("圆括号整组重定向", "(echo a; echo b) > research/results/untracked.out", 1),
        ("timeout、capped.sh 包着的 bash -c 整条重定向", "timeout 60 bash scripts/hosts/capped.sh 4 bash -c 'cargo run' > research/results/untracked.out", 1),
        ("前面赋过值的变量代入", 'out=research/results/untracked.out; python3 x.py > "$out"', 1),
        ("export 过的变量代入", 'export OUT=research/results/untracked.out; python3 x.py > "${OUT}"', 1),
        ("cd 到变量路径算不出来、按仓库根解析", 'cd "$scratch" && python3 x.py > research/results/untracked.out', 1),
        ("cd $HOME/… 算得出来、按那个目录解析（仓外，放行）", 'cd "$HOME/no-such-directory-for-detector-selftest" && python3 x.py > research/results/untracked.out', 0),
        ("进程替换里的 tee", "make > >(tee research/results/untracked.out)", 1),
        ("整个循环重定向", "for arm in a b; do python3 x.py $arm; done > research/results/untracked.out", 1),
        ("sudo 包着的 tee", "echo x | sudo tee research/results/untracked.out", 1),
        ("git add 的是另一份，这一份照拒", "git add research/results/tracked.out && python3 x.py > research/results/untracked.out", 1),
        ("dd of= 覆盖未跟踪的产物", "dd if=/tmp/new.out of=research/results/untracked.out", 1),
        ("dd conv=notrunc 覆盖未跟踪的产物", "dd if=/tmp/new.out of=research/results/untracked.out conv=notrunc", 1),
        ("truncate -s 0 覆盖未跟踪的产物", "truncate -s 0 research/results/untracked.out", 1),
        ("truncate 选项连写覆盖未跟踪产物", "truncate -cs 0 research/results/sub/deep.out", 1),
        ("dd of= 写已跟踪的文件（git 兜得住）", "dd if=/tmp/new.out of=research/results/tracked.out", 0),
        ("truncate 写已跟踪的文件（git 兜得住）", "truncate -s 0 research/results/tracked.out", 0),
        ("dd of= 写新文件名", "dd if=/tmp/new.out of=research/results/untracked-2026-09-25.out", 0),
        ("dd of= 目标在 research/results/ 外", "dd if=/tmp/new.out of=scratch.log", 0),
        ("同一条命令里先 git add 再 dd 覆盖", "git add research/results/untracked.out && dd if=/tmp/new.out of=research/results/untracked.out", 0),
        ("引号里当数据写的 dd", "echo 'dd if=/tmp/new.out of=research/results/untracked.out'", 0),
        (">> 追加", "python3 x.py >> research/results/untracked.out", 0),
        ("&>> 追加", "cargo test &>> research/results/untracked.out", 0),
        ("写新文件名", "python3 x.py > research/results/untracked-2026-09-25.out", 0),
        ("写已跟踪的文件（git 兜得住）", "python3 x.py > research/results/tracked.out", 0),
        ("/tmp 目标（已存在、不在任何仓里）", f"python3 x.py > {outside}", 0),
        ("仓里 research/results/ 外面的未跟踪文件", "python3 x.py > scratch.log", 0),
        ("2>&1 不是写一个叫 1 的文件（research/results/1 未跟踪地存在）", "cd research/results && python3 x.py >> untracked.out 2>&1", 0),
        ("tee -a", "make | tee -a research/results/untracked.out", 0),
        ("tee --append", "make | tee --append research/results/untracked.out", 0),
        ("cp -n 不盖已有的", "cp -n fresh.out research/results/untracked.out", 0),
        ("cp --backup 旧的留成备份", "cp --backup=numbered fresh.out research/results/untracked.out", 0),
        ("install -d 建目录", "install -d research/results/untracked.out", 0),
        ("cp 的源是产物、目标在外面", "cp research/results/untracked.out /tmp/copy.out", 0),
        ("读产物、写别处", "sort < research/results/untracked.out > /tmp/sorted.out", 0),
        ("cd 到仓外之后同样的相对路径", f"cd {elsewhere} && python3 x.py > research/results/untracked.out", 0),
        ("同一条命令里先挪到带日期的旧名再写", "mv research/results/untracked.out research/results/untracked-2026-09-24-old.out && python3 x.py > research/results/untracked.out", 0),
        ("同一条命令里先 git add 再写", "git add research/results/untracked.out && python3 x.py > research/results/untracked.out", 0),
        ("引号里当数据写", "echo 'python3 x.py > research/results/untracked.out'", 0),
        ("写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\npython3 x.py > research/results/untracked.out\nEOF", 0),
        ("注释里写的", "ls  # > research/results/untracked.out", 0),
        ("目标里有算不出的变量：不拦（只记检出）", "for arm in a b; do python3 x.py > research/results/untracked$arm.out; done", 0),
        ("目标里有命令替换、对不上已有文件", "python3 x.py > research/results/untracked-$(date +%F).out", 0),
    ]
    for label, command, want in results_overwrite_cases:
        results.append((f"覆盖产物:{label}", want, 1 if git_ready and results_overwrite_refusal(command, repository)[0] else 0))
    # 一条命令里几个确定的目标同时判：各自点名，且 git ls-files 只起一次（BASH_COMMAND_DETECTOR_BREAK=per-path-git-query 一份一份查，后两格的次数红）
    def refused_and_runs(command):
        GIT_LS_FILES_RUNS[0] = 0
        refused_here = results_overwrite_refusal(command, repository)[0] if git_ready else []
        return refused_here, GIT_LS_FILES_RUNS[0]
    two_refused, two_runs = refused_and_runs("make | tee research/results/untracked.out research/results/sub/deep.out")
    mixed_refused, mixed_runs = refused_and_runs("make | tee research/results/tracked.out research/results/untracked.out research/results/sub/deep.out research/results/fresh.out")
    results.append(("覆盖产物:一条命令里两份未跟踪的目标各自点名", ["tee research/results/untracked.out", "tee research/results/sub/deep.out"], two_refused))
    results.append(("覆盖产物:一条命令里两份未跟踪的目标同时判，git ls-files 只起一次", 1, two_runs))
    results.append(("覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起同时判，只点名未跟踪的", ["tee research/results/untracked.out", "tee research/results/sub/deep.out"], mixed_refused))
    results.append(("覆盖产物:已跟踪的、未跟踪的、不存在的目标混在一起，git ls-files 只起一次", 1, mixed_runs))
    # 只记检出、不拦：(说明, 命令, 仓库根, 该不该记)
    results_noted_cases = [
        ("算不出的变量对得上一份未跟踪的产物", "for arm in a b; do python3 x.py > research/results/untracked$arm.out; done", repository, 1),
        ("变量在前、research/results/ 在后", 'python3 x.py > "$CLAUDE_PROJECT_DIR"/research/results/untracked.out', repository, 1),
        ("算不出的段对不上任何已有文件（一定是新文件名）", "for arm in a b; do python3 x.py > research/results/e-$arm.out; done", repository, 0),
        ("命令替换当日期、对不上已有文件", "python3 x.py > research/results/untracked-$(date +%F).out", repository, 0),
        ("对得上的只有已跟踪的", "for arm in a b; do python3 x.py > research/results/tracked$arm.out; done", repository, 0),
        ("git 判不了（不是 git 仓）只记不拦", "python3 x.py > research/results/untracked.out", not_a_repository, 1),
        ("拒了的不另记", "python3 x.py > research/results/untracked.out", repository, 0),
    ]
    for label, command, root, want in results_noted_cases:
        refused_here, noted_here = results_overwrite_refusal(command, root) if git_ready else ([], [])
        results.append((f"覆盖产物检出:{label}", want, int(bool(noted_here))))
    results.append(("覆盖产物检出:git 判不了的那一条不拦", 0, int(bool(results_overwrite_refusal("python3 x.py > research/results/untracked.out", not_a_repository)[0]))))
    # ⑦ 在原 inode 上改已有脚本：在临时仓里造 .sh、.py、带执行位没后缀的、指向脚本的符号链接与不是脚本的文件，scratch 根换成临时目录；只判不执行
    scratch_root = os.path.join(work, "scratch-root")
    os.makedirs(os.path.join(repository, "scripts"))
    os.makedirs(os.path.join(repository, "bin"))
    os.makedirs(os.path.join(scratch_root, "probe"))
    loop_hooks = os.path.join(scratch_root, "new", ".claude", "hooks")   # python 循环变量那几格：scratch 根下一棵新版树里的两份已有脚本
    os.makedirs(loop_hooks)
    for loop_script in ("bash_command_detector_selftest_waits_and_scope.py", "bash-command-detector.sh"):
        with open(os.path.join(loop_hooks, loop_script), "w", encoding="utf-8") as handle:
            handle.write("echo 旧内容\n")
    for relative, mode in (("scripts/running.sh", 0o755), ("scripts/tool.py", 0o644), ("bin/runner", 0o755),
                           ("notes.md", 0o644), ("data.txt", 0o644)):
        with open(os.path.join(repository, relative), "w", encoding="utf-8") as handle:
            handle.write("echo 旧内容\n")
        os.chmod(os.path.join(repository, relative), mode)
    os.symlink("running.sh", os.path.join(repository, "scripts", "link.sh"))
    for path in (os.path.join(scratch_root, "probe", "step.sh"), os.path.join(work, "outside.sh")):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("echo 旧内容\n")
        os.chmod(path, 0o755)
    running = "scripts/running.sh"
    # (说明, 命令, 该不该拒绝)；目标按临时仓的根解析
    script_in_place_cases = [
        ("> 覆盖已有的 .sh", f"echo x > {running}", 1),
        (">|", f"echo x >| {running}", 1),
        ("&>", f"make &> {running}", 1),
        ("2> 同样截断", f"make 2> {running}", 1),
        (">& 后面跟文件名", f"make >& {running}", 1),
        ("cat > 旧 <<EOF", f"cat > {running} <<'EOF'\n#!/usr/bin/env bash\necho 新内容\nEOF", 1),
        ("tee 不带 -a", f"echo x | tee {running}", 1),
        ("tee -i 不是追加", f"echo x | tee -i {running}", 1),
        ("cp 新 旧", f"cp /tmp/new.sh {running}", 1),
        ("cp -f", f"cp -f /tmp/new.sh {running}", 1),
        ("cp -a", f"cp -a /tmp/new.sh {running}", 1),
        ("cp 进已有目录、文件名撞上", f"cp {sources}/running.sh scripts/", 1),
        ("cp -t 目录", f"cp -t scripts {sources}/running.sh", 1),
        ("dd of=", f"dd if=/tmp/new.sh of={running}", 1),
        ("dd conv=notrunc", f"dd if=/tmp/new.sh of={running} conv=notrunc", 1),
        ("truncate -s 0", f"truncate -s 0 {running}", 1),
        ("truncate -cs 0（选项连写）", "truncate -cs 0 scripts/tool.py", 1),
        ("写已有的 .py", "echo x > scripts/tool.py", 1),
        ("带执行位、没有后缀的", "cp /tmp/new bin/runner", 1),
        ("经符号链接写它指向的脚本", "echo x > scripts/link.sh", 1),
        ("scratch 根下的脚本", f"cp /tmp/new.sh {scratch_root}/probe/step.sh", 1),
        ("cd 进去之后写相对路径", "cd scripts && echo x > running.sh", 1),
        ("bash -c 里的 >", f"bash -c 'echo x > {running}'", 1),
        ("喂给 bash 的 heredoc 正文里的 cp", f"bash <<'EOF'\ncp /tmp/new.sh {running}\nEOF", 1),
        ("命令替换里的 >", f"lines=$(echo x > {running}; echo done)", 1),
        ("前面赋过值的变量代入", f'target={running}; cp /tmp/new.sh "$target"', 1),
        ("nice、timeout 包着的 cp", f"timeout 60 nice -n 19 cp /tmp/new.sh {running}", 1),
        ("python3 -c 的 open(…, 'w')", f"python3 -c \"open('{running}', 'w').write('x')\"", 1),
        ("python3 -c 的 open(…, 'r+')", f"python3 -c \"open('{running}', 'r+').write('x')\"", 1),
        ("python3 -c 的 open(…, mode='wb')", "python3 -c \"open('scripts/tool.py', mode='wb').write(b'x')\"", 1),
        ("喂给 python 的 heredoc：只赋过一次的名字", f"python3 - <<'EOF'\npath = '{running}'\nwith open(path, 'w') as handle:\n    handle.write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：Path(…) / '…' 的 write_text", "python3 - <<'EOF'\nfrom pathlib import Path\n(Path('scripts') / 'tool.py').write_text('x')\nEOF", 1),
        ("喂给 python 的 heredoc：pathlib.Path(…).write_bytes", f"python3 <<'EOF'\nimport pathlib\npathlib.Path('{running}').write_bytes(b'x')\nEOF", 1),
        ("喂给 python 的 heredoc：Path(…).open('w')", f"python3 - <<'EOF'\nimport pathlib\nwith pathlib.Path('{running}').open('w') as handle:\n    handle.write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：os.path.join 与 io.open", "python3 - <<'EOF'\nimport io, os\nio.open(os.path.join('scripts', 'tool.py'), 'w')\nEOF", 1),
        ("喂给 python 的 heredoc：shutil.copyfile", f"python3 - <<'EOF'\nimport shutil\nshutil.copyfile('/tmp/new.sh', '{running}')\nEOF", 1),
        ("喂给 python 的 heredoc：shutil.copy 进已有目录", "python3 - <<'EOF'\nimport shutil\nshutil.copy('/tmp/running.sh', 'scripts')\nEOF", 1),
        ("cd 之后喂给 python 的相对路径", "cd scripts && python3 - <<'EOF'\nopen('running.sh', 'w').write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：for 逐个取元组里只赋过一次的名字，open(循环变量, 'w') 盖 scratch 根下的脚本",
         f"cd {scratch_root}/new && python3 - <<'EOF'\np='{loop_hooks}/bash_command_detector_selftest_waits_and_scope.py'\nq='{loop_hooks}/bash-command-detector.sh'\n"
         "for f in (p,q):\n    t=open(f,encoding='utf-8').read()\n    open(f,'w',encoding='utf-8').write(t)\nEOF", 1),
        ("喂给 python 的 heredoc：for 逐个取列表字面量，候选里有一个是已有脚本", f"python3 - <<'EOF'\nfor f in ['notes.md', '{running}']:\n    open(f, 'w').write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：with open(os.path.join(…, 循环变量), 'w') 嵌在 for 的循环体里",
         "python3 - <<'EOF'\nimport os\nfor name in ('running.sh', 'tool.py'):\n    with open(os.path.join('scripts', name), 'w') as handle:\n        handle.write('x')\nEOF", 1),
        # 经只赋过一次的名字引到循环变量（python_literal_path 的 bound 往赋值里带、referenced_loop_names 往赋值里找），与循环体里的条件不看、绑定次数不数 except … as
        ("喂给 python 的 heredoc：循环体里 target = os.path.join(…, 循环变量) 只赋过一次，open(target, 'w') 经它引到循环变量",
         "python3 - <<'EOF'\nimport os\nfor name in ('notes.md', 'running.sh'):\n    target = os.path.join('scripts', name)\n    open(target, 'w').write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：循环体里先判 os.path.exists 只写新文件，条件不看、候选里有已有脚本照拒",
         f"python3 - <<'EOF'\nimport os\nfor f in ('{running}', 'scripts/loop-new-step.sh'):\n    if os.path.exists(f):\n        continue\n    open(f, 'w').write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：循环体里只写 .md 后缀，条件不看、候选里有已有脚本照拒",
         f"python3 - <<'EOF'\nfor f in ('notes.md', '{running}'):\n    if f.endswith('.md'):\n        open(f, 'w').write('x')\nEOF", 1),
        ("喂给 python 的 heredoc：循环变量另在 except … as 里绑定（不数进绑定次数），照按候选拒",
         f"python3 - <<'EOF'\nfor f in ('notes.md', '{running}'):\n    try:\n        open(f, 'w').write('x')\n    except OSError as f:\n        pass\nEOF", 1),
        ("bash -c 里的 python3 -c", f"bash -c \"python3 -c \\\"open('{running}', 'w')\\\"\"", 1),
        (">> 追加", f"echo x >> {running}", 0),
        ("&>> 追加", f"make &>> {running}", 0),
        ("tee -a", f"echo x | tee -a {running}", 0),
        ("python 的 'a' 追加", f"python3 -c \"open('{running}', 'a').write('x')\"", 0),
        ("python 的 'x' 只建新文件", f"python3 -c \"open('{running}', 'x')\"", 0),
        ("python 读", f"python3 -c \"print(open('{running}').read())\"", 0),
        ("写新文件名", "echo x > scripts/new-step.sh", 0),
        ("写到同目录临时文件再 mv 换上", f"cat > scripts/.running.sh.new <<'EOF'\necho 新内容\nEOF\nchmod 755 scripts/.running.sh.new && mv scripts/.running.sh.new {running}", 0),
        ("cp 成隐藏文件、chmod、mv 换上", f"cp /tmp/new.sh scripts/.running.sh.installing && chmod 755 scripts/.running.sh.installing && mv scripts/.running.sh.installing {running}", 0),
        ("mv 新 旧", f"mv /tmp/new.sh {running}", 0),
        ("install 换 inode", f"install -m 755 /tmp/new.sh {running}", 0),
        ("cp --remove-destination 换 inode", f"cp --remove-destination /tmp/new.sh {running}", 0),
        ("cp -l -f 换成硬链接", f"cp -l -f /tmp/new.sh {running}", 0),
        ("cp -n 不盖已有的", f"cp -n /tmp/new.sh {running}", 0),
        ("cp -b 旧的留成备份", f"cp -b /tmp/new.sh {running}", 0),
        ("sed -i 换 inode", f"sed -i 's/旧/新/' {running}", 0),
        ("同一条命令里先 rm 再写", f"rm {running} && echo x > {running}", 0),
        ("同一条命令里先挪走再写", f"mv {running} {running}.old && echo x > {running}", 0),
        ("python 写到临时文件再 os.replace", f"python3 - <<'EOF'\nimport os\nopen('scripts/.running.sh.new', 'w').write('x')\nos.replace('scripts/.running.sh.new', '{running}')\nEOF", 0),
        ("写已有的非脚本文件（.md）", "echo x > notes.md", 0),
        ("写已有的、不带执行位的 .txt", "echo x > data.txt", 0),
        ("仓外、scratch 根外的脚本", f"echo x > {work}/outside.sh", 0),
        ("读脚本、写别处", f"cat {running} > {work}/copy.txt", 0),
        ("cp 脚本出去", f"cp {running} {work}/copy.sh", 0),
        ("引号里当数据写", f"echo 'cp /tmp/new.sh {running}'", 0),
        ("写进笔记的 heredoc 正文", f"cat > {work}/note.md <<'EOF'\necho x > {running}\nEOF", 0),
        ("注释里写的", f"ls  # cp /tmp/new.sh {running}", 0),
        ("目标里有算不出的变量（判不到）", 'cp /tmp/new.sh "$target"', 0),
        ("python 的路径从 sys.argv 来（判不到）", f"python3 -c \"import sys; open(sys.argv[1], 'w')\" {running}", 0),
        ("python 的名字赋过两次（判不到）", f"python3 - <<'EOF'\npath = 'notes.md'\npath = '{running}'\nopen(path, 'w')\nEOF", 0),
        ("python 的 for 逐个取元组，候选都不是脚本", "python3 - <<'EOF'\nfor f in ('notes.md', 'data.txt'):\n    open(f, 'w').write('x')\nEOF", 0),
        ("python 的 for 逐个取元组，候选是新文件名与不是脚本的文件",
         "python3 - <<'EOF'\nfrom pathlib import Path\nfor f in ('notes.md', 'scripts/loop-new-step.sh'):\n    Path(f).write_text('x')\nEOF", 0),
        ("python 的 for 元组里有一个元素算不出（判不到）", f"python3 - <<'EOF'\nimport sys\nfor f in (sys.argv[1], '{running}'):\n    open(f, 'w')\nEOF", 0),
        ("python 的循环变量另有一次赋值（判不到）", f"python3 - <<'EOF'\nf = 'notes.md'\nfor f in ('{running}',):\n    open(f, 'w')\nEOF", 0),
        ("python 的 for 取推导式（判不到）", f"python3 - <<'EOF'\nfor f in [item for item in ('{running}',)]:\n    open(f, 'w')\nEOF", 0),
        ("python 的循环变量在循环体之外用（判不到）", f"python3 - <<'EOF'\nfor f in ('{running}',):\n    pass\nopen(f, 'w')\nEOF", 0),
        ("python 的循环写在推导式里（判不到）", f"python3 - <<'EOF'\n[open(f, 'w').write('x') for f in ('notes.md', '{running}')]\nEOF", 0),
        ("python 的 lambda 参数经 map 逐个取元组（判不到）", f"python3 - <<'EOF'\nlist(map(lambda f: open(f, 'w').write('x'), ('notes.md', '{running}')))\nEOF", 0),
        ("python 跑脚本文件、喂给它的 heredoc 是数据", f"python3 tool.py <<'EOF'\nopen('{running}', 'w')\nEOF", 0),
        ("一百三十七：rsync --inplace 盖已有脚本（rsync-inplace-unseen 下红）", f"rsync --inplace /tmp/new.sh {running}", 1),
        ("一百三十七：rsync 不带 --inplace 换 inode", f"rsync /tmp/new.sh {running}", 0),
    ]
    for label, command, want in script_in_place_cases:
        results.append((f"就地改脚本:{label}", want, 1 if script_in_place_refusal(command, repository, scratch_root) else 0))
    results.append(("就地改脚本:拒绝时报出写法与相对仓库根的路径", 1,
                    int(script_in_place_refusal(f"cp /tmp/new.sh {running}", repository, scratch_root) == [f"cp {running}"])))
    results.append(("就地改脚本:python 的 for 逐个取值时，拒绝点名是已有脚本的那个候选、不点名别的候选", 1,
                    int(script_in_place_refusal(f"python3 - <<'EOF'\nfor f in ['notes.md', '{running}']:\n    open(f, 'w').write('x')\nEOF",
                                                repository, scratch_root) == [f"python open(…, 'w') {running}"])))
    # 不给 scratch_root 时认的是工作根与会话临时目录根（SCRIPT_SCRATCH_ROOTS）：几处各放一份已有的脚本，自己建、自己删；
    # 会话 scratchpad 那一份照 Claude Code 的布局 <会话临时目录根>/<项目目录名>/<会话号>/scratchpad/ 放（项目目录名那一层是自己建的临时目录）
    session_temporary_root = work_roots.claude_session_temporary_root() if work_roots else None
    for label, root, inner in (("工作根下的脚本", work_roots.configured_work_root() if work_roots else None, ""),
                               ("会话临时目录根下的脚本", session_temporary_root, ""),
                               ("会话 scratchpad 里的脚本", session_temporary_root, os.path.join("selftest-session", "scratchpad"))):
        refused = 0
        if root is not None:
            os.makedirs(root, exist_ok=True)
            probe_directory = tempfile.mkdtemp(prefix="-home-bash-command-detector-selftest-", dir=root)
            try:
                os.makedirs(os.path.join(probe_directory, inner), exist_ok=True)
                probe = os.path.join(probe_directory, inner, "step.sh")
                with open(probe, "w", encoding="utf-8") as handle:
                    handle.write("echo 旧内容\n")
                refused = int(bool(script_in_place_refusal(f"cp /tmp/new.sh {probe}", repository)))
            finally:
                shutil.rmtree(probe_directory, ignore_errors=True)
        results.append((f"就地改脚本:{label}", 1, refused))
    results.append(("就地改脚本:⑤ 现在认 dd、truncate 覆盖未跟踪产物（与 ⑦ 共用同一段 overwrite_steps，不抄第二份）", 1,
                    int(bool(results_overwrite_refusal("dd if=x of=research/results/untracked.out; truncate -s 0 research/results/untracked.out",
                                                       repository)[0])) if git_ready else 1))
    results.append(("就地改脚本:⑤ 仍不认 python 的写法（覆盖未跟踪产物；rm 另见下一条，那条测的是 PreservingStep 不是 form）", 0,
                    int(bool(results_overwrite_refusal("python3 -c \"open('research/results/untracked.out', 'w')\"", repository)[0])) if git_ready else 0))
    results.append(("就地改脚本:rm 掉再写未跟踪产物，⑤ 照拒（rm 没留住旧字节）", 1,
                    int(bool(results_overwrite_refusal("rm research/results/untracked.out && echo x > research/results/untracked.out", repository)[0])) if git_ready else 1))
    # 走真实入口：把 hook 与共用切词模块拷进临时仓的 .claude/hooks/，仓库根取它往上两级
    repository_hooks = os.path.join(repository, ".claude", "hooks")
    os.makedirs(repository_hooks)
    for library_name in ("bash-command-detector.sh",) + DETECTOR_PART_NAMES + ("lib_shell_words.py",) + SHELL_WORDS_PART_NAMES:
        shutil.copy(os.path.join(hook_dir, library_name), repository_hooks)
    for project_code_runs_file in ("lib_project_code_runs.py", "lib_project_code_runs_bootstrap.py", "lib_project_code_runs_base.py", "lib_project_code_runs_launchers.py", "lib_project_code_runs_selftest.py"):   # 剥 /usr/bin/time、flock 这类启动器用它（command_under_launcher）；入口按自己所在目录 import 部分模块
        shutil.copy(os.path.join(hook_dir, project_code_runs_file), repository_hooks)
    # 工作根（⑦ ⑪ 认工作根）与写者锁两份库同样拷进去：缺了，钩子子进程就记一条「读不到」的检出、工作根相关的判法也取不到；
    # 写范围库不拷：这个仓不放写范围表，⑫ 在它上面不判（写范围的入口格在另搭的 scope_repository 上判）
    for library_name in ("lib_work_root.py", "lib_writer_lock.py"):
        shutil.copy(os.path.join(hook_dir, library_name), repository_hooks)
    def through_entry(command, run_in_background=False):
        before_entry = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
        completed = subprocess.run(["bash", os.path.join(repository_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
                                   env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
                                   input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
                                       "command": command, "run_in_background": run_in_background}}))
        after_entry = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
        return completed, after_entry - before_entry
    scene, scene_recorded = through_entry("cp /tmp/claude-1000/e158-s9/config-evidence-runs/yi-ring-retained.out research/results/untracked.out")
    results.append(("stdin:cp 覆盖未跟踪的产物拒绝（退出码 2）", 2, scene.returncode))
    results.append(("stdin:拒绝时 stderr 写了出路（按日期另存、先 git add 或挪到带日期的旧名）", 1,
                    int("✗" in scene.stderr and "→" in scene.stderr and "research/results/untracked.out" in scene.stderr
                        and "按日期另存一个新文件名，旧的留着；确实要换掉旧产物，先 `git add` 它或者把它挪到带日期的旧名" in scene.stderr)))
    results.append(("stdin:拒了的不记检出", 0, scene_recorded))
    background_overwrite = through_entry("python3 x.py > research/results/untracked.out", True)[0]
    results.append(("stdin:run_in_background 里的 > 覆盖同样拒绝（退出码 2）", 2, background_overwrite.returncode))
    appended, appended_recorded = through_entry("python3 x.py >> research/results/untracked.out")
    results.append(("stdin:>> 追加放行（退出码 0）、不记检出", (0, 0), (appended.returncode, appended_recorded)))
    unresolved, unresolved_recorded = through_entry("for arm in a b; do python3 x.py > research/results/untracked$arm.out; done")
    results.append(("stdin:目标算不出、对得上未跟踪产物的放行（退出码 0）并记一条检出", (0, 1), (unresolved.returncode, unresolved_recorded)))
    # 走真实入口：在原 inode 上改临时仓里的已有脚本拒绝（退出码 2）、stderr 给出 mv 换上与定点改两条出路、不记检出；写到临时文件再 mv 换上的放行
    in_place, in_place_recorded = through_entry(f"cp /tmp/new.sh {running}")
    results.append(("stdin:cp 覆盖已有脚本拒绝（退出码 2）", 2, in_place.returncode))
    results.append(("stdin:拒绝时 stderr 写了出路（临时文件再 mv 换上、replace-once.py / insert-row.py 定点改）", 1,
                    int("✗" in in_place.stderr and "→" in in_place.stderr and f"cp {running}" in in_place.stderr
                        and "写到同目录临时文件再 `mv` 换上" in in_place.stderr and "scripts/kb/replace-once.py" in in_place.stderr)))
    results.append(("stdin:就地改脚本拒了的不记检出", 0, in_place_recorded))
    # python 的 for 循环体里带条件只写新文件：条件不看、照拒，拒绝说明给这一形能照做的出路（Edit 工具或写临时文件再 mv）
    guarded_loop = through_entry(f"python3 - <<'EOF'\nimport os\nfor f in ('{running}', 'scripts/loop-new-step.sh'):\n"
                                 "    if os.path.exists(f):\n        continue\n    open(f, 'w').write('x')\nEOF")[0]
    results.append(("stdin:python 的 for 循环体里先判 os.path.exists 只写新文件照拒（退出码 2），stderr 写了这一形的出路（条件不看、Edit 工具或写临时文件再 mv）", (2, True),
                    (guarded_loop.returncode, all(fragment in guarded_loop.stderr for fragment in (
                        f"python open(…, 'w') {running}", "不看循环体里的条件", "Edit 工具", "写临时文件再 `mv`")))))
    python_in_place = through_entry(f"python3 - <<'EOF'\nopen('{running}', 'w').write('x')\nEOF", True)[0]
    results.append(("stdin:run_in_background 里喂给 python 的 open(…, 'w') 同样拒绝（退出码 2）", 2, python_in_place.returncode))
    # ⑪ rm -r 通删工作根：(说明, 命令, 该不该拒)；样本路径照配置的工作根造（不看弄坏开关），会话临时目录根是 /tmp/claude-<uid>
    removal_root = work_root_removal_roots()[0]
    removal_home = os.path.expanduser("~")
    removal_tilde = "~" + removal_root[len(removal_home):] if removal_root.startswith(removal_home + "/") else removal_root
    removal_dollar = "$HOME" + removal_root[len(removal_home):] if removal_root.startswith(removal_home + "/") else removal_root
    removal_session_temporary_root = f"/tmp/claude-{os.getuid()}"
    removal_cases = [
        ("工作根下通配（~ 写法）", f"rm -rf {removal_tilde}/*", 1),
        ("工作根下通配（$HOME 写法、引号里外）", f'rm -rf "{removal_dollar}/"*', 1),
        ("工作根本身", f"rm -rf {removal_root}", 1),
        ("工作根本身带结尾的 /", f"rm -rf {removal_tilde}/", 1),
        ("会话临时目录根下通配", f"rm -rf {removal_session_temporary_root}/*", 1),
        ("rm -fr", f"rm -fr {removal_root}/*", 1),
        ("rm -r -f 分开写", f"rm -r -f {removal_root}/*", 1),
        ("rm --recursive --force", f"rm --recursive --force {removal_root}/*", 1),
        ("rm -r 不带 -f", f"rm -r {removal_root}/*", 1),
        ("工作根下第一层部分通配", f"rm -rf {removal_root}/owed-*", 1),
        ("工作根下第一层通配再往下", f"rm -rf {removal_root}/*/target", 1),
        ("cd 进工作根之后 rm -rf *", f"cd {removal_tilde} && rm -rf *", 1),
        # cd 的目录写在变量里：同一段命令里前面赋过的变量与 $HOME 展开之后再判（共用模块 lib_shell_words.py 的 expand_known_variables）
        ("D=… 之后 cd $D 进工作根再 rm -rf *", f"D={removal_root}; cd $D && rm -rf *", 1),
        ("cd $HOME/… 进工作根再 rm -r 第一层部分通配", f"cd {removal_dollar} && rm -r owed-*", 1),
        ("cd 进写在变量里的工作根下具体目录再 rm -rf * 放行", f"D={removal_root}/owed-x; cd $D && rm -rf *", 0),
        ("bash -c 里", f"bash -c 'rm -rf {removal_root}/*'", 1),
        ("nice、sudo 前缀", f"nice -n 19 sudo rm -rf {removal_root}/*", 1),
        ("工作根的上级", f"rm -rf {os.path.dirname(removal_root)}", 1),
        ("$SINGLEFS_WORK_ROOT 写法", 'rm -rf "$SINGLEFS_WORK_ROOT"/*', 1),
        ("/tmp/* 盖住会话临时目录根", "rm -rf /tmp/*", 1),
        # 探针串拆成两段相邻字面量拼起来：探针要的就是「变量路径不带空值守卫」这个写法，整串写在源码里会被 shell 纪律检查当成本脚本自己的 rm -rf
        ("前面 export 过的变量代入", f'export W={removal_root}; rm -rf "$' 'W"/*', 1),
        ("选项写在操作数后面", f"rm {removal_root}/* -rf", 1),
        ("喂给 bash 的 heredoc 里", f"bash <<'EOF'\nrm -rf {removal_root}/*\nEOF", 1),
        ("工作根下一个具体目录放行", f"rm -rf {removal_tilde}/owed-c792-work-ownership", 0),
        ("具体目录里面的通配放行", f"rm -rf {removal_root}/owed-x/copy-*", 0),
        ("别处的 rm -rf 通配放行", "rm -rf /tmp/singlefs-mutation-x/*", 0),
        ("仓里的相对路径放行", "rm -rf target/debug/incremental", 0),
        ("不带 -r 的 rm 放行", f"rm -f {removal_root}/*.tmp", 0),
        ("echo 引号里当数据写的放行", f"echo 'rm -rf {removal_root}/*'", 0),
        ("grep 参数里的放行", "grep -n 'rm -rf' .claude/hooks/bash-command-detector.sh", 0),
        ("算不出的变量放行", 'rm -rf "$d"/*', 0),
        ("同名前缀的兄弟目录放行", f"rm -rf {removal_root}-other/*", 0),
        ("会话临时目录根下一个具体目录放行", f"rm -rf {removal_session_temporary_root}/owed-x", 0),
        ("写进文件的 heredoc 正文放行", f"cat > note.md <<'EOF'\nrm -rf {removal_root}/*\nEOF", 0),
        ("家目录通配不碰点目录放行", "rm -rf ~/*", 0),
    ]
    local_names = locals()
    shared.update({name: local_names[name] for name in ("background", "command", "detections", "handle", "hook_dir", "hook_payload", "label", "line", "name", "new_root", "read_only_scope_not_run", "relative", "removal_cases", "removal_root", "repository", "repository_hooks", "results", "running", "scope_hooks", "scope_repository", "through_entry", "value", "want", "want_notices", "whole_target_root", "work") if name in local_names})
