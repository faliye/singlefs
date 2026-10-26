import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
CAP = cases_a.CAP
M = 'main'
# 补：--list 只列用例、经包装时不算重型
C('E24c-IW', IW, CAP + 'cargo test --all -- --list', 0, 0, ['list_opt','ac_cap'])
C('E24c-M', M, cases_a.RB + 'cargo test --all -- --list', 0, 0, ['list_opt'])
# 主工作区里直接起会编的 cargo：主 agent 与子 agent 一样拒；包一层或仓外目录放行
for ident in (M, IW):
    C(f'T01-{ident[:3]}', ident, 'cargo build', 0 if False else 0, 0, ['target'], cwd='repo', note='目标：拒（重型闸返回 2）')
CASES[-2]['h'] = 2; CASES[-1]['h'] = 2
for n, cmd, exp in [
 ('T02', 'cargo test -p singlefs-harness', 2), ('T03', 'cargo clippy --workspace', 2), ('T04', 'cargo check', 2),
 ('T05', 'cd crates/singlefs-core && cargo build', 2), ('T06', 'cargo build --target-dir {D}/tgt', 0),
 ('T07', 'CARGO_TARGET_DIR={D}/tgt cargo build', 0), ('T08', 'bash scripts/hosts/run-build-directory.sh cargo build', 0),
 ('T09', 'cargo fmt --all', 0), ('T10', 'cargo metadata --no-deps', 0), ('T11', 'cargo tree', 0), ('T12', 'cargo clean', 0),
 ('T13', 'CARGO_TARGET_DIR=target cargo build', 2), ('T14', 'cargo doc --no-deps', 2), ('T15', 'cargo bench --no-run', 2),
 ('T16', 'cargo run --bin foo', 2),
]:
    for ident in (M, IW):
        # 子 agent 在主工作区里 cargo run/test 还要内存包装：这里只看编译目录那一道的口径，统一用包装测子 agent
        c = cmd
        C(f'{n}-{ident[:3]}', ident, c, exp, 0, ['target','target_ok'], cwd='repo')

# ---------- 检测器：七种拒绝 ----------
# ① 看门狗
C('F01-M', M, 'bash scripts/agents/watch.sh', 0, 2, ['b_watch','ac_watch'])
C('F02-M', M, 'bash scripts/agents/watch.sh', 0, 0, ['b_watch','ac_bg'], bg=True)
C('F03-M', M, 'bash scripts/agents/watch.sh --processes', 0, 0, ['b_watch'], bg=True)
C('F04-M', M, 'nohup bash scripts/agents/watch.sh &', 0, 2, ['b_watch_bad'], bg=True)
C('F05-M', M, 'bash scripts/agents/watch.sh > /dev/null 2>&1', 0, 2, ['b_watch_bad'], bg=True)
C('F06-M', M, 'python3 scripts/agents/agent-watch.py watch', 0, 2, ['b_watch','ac_watch'])
C('F07-M', M, 'python3 scripts/agents/agent-watch.py watch', 0, 0, ['b_watch'], bg=True)
C('F08-M', M, 'bash scripts/agents/watch.sh --selftest', 0, 0, ['b_watch'])
C('F09-M', M, 'bash scripts/agents/watch.sh --report', 0, 0, ['b_watch'])
C('F10-M', M, 'grep -n watch.sh .claude/main-agent.md', 0, 0, ['b_watch'])
C('F11-M', M, 'cat research/x.log | tail -5', 0, 0, ['b_watch'])
# ② 前台等待循环
for ident in (M, IW):
    s = ident[:3]
    C(f'G01-{s}', ident, "until grep -q '^rc=' log.txt; do sleep 10; done", 0, 2, ['b_loop','ac_loop'])
    C(f'G02-{s}', ident, 'while true; do sleep 1; done', 0, 2, ['b_loop'])
    C(f'G03-{s}', ident, "timeout 60 bash -c 'while true; do sleep 1; done'", 0, 0, ['b_loop_timeout'])
    C(f'G04-{s}', ident, 'until timeout 5 grep -q ok log.txt; do sleep 10; done', 0, 2, ['b_loop_inner'])
    C(f'G05-{s}', ident, 'for i in 1 2 3; do sleep 1; done', 0, 0, ['b_loop_ok'])
    C(f'G06-{s}', ident, 'while read -r line; do echo "$line"; done < fx/light.sh', 0, 0, ['b_loop_ok'])
    C(f'G07-{s}', ident, 'n=0; while [ $n -lt 5 ]; do n=$((n+1)); sleep 1; done', 0, 2, ['b_loop_ok'])
    C(f'G08-{s}', ident, 'n=0\nwhile [ $n -lt 5 ]\ndo\n  n=$((n+1))\n  sleep 1\ndone', 0, 2, ['b_loop'])
    C(f'G09-{s}', ident, "echo 'until x; do sleep 1; done'", 0, 0, ['b_loop'])
    C(f'G10-{s}', ident, "cat > {D}/loop.sh <<'EOF'\nuntil x; do sleep 1; done\nEOF", 0, 0, ['b_loop'])
    C(f'G11-{s}', ident, 'timeout 100 bash scripts/hosts/capped.sh 31 bash -c "while true; do sleep 1; done"', 0, 0, ['b_loop_timeout'])
C('G12-M', M, "until grep -q ok log.txt; do sleep 10; done", 0, 0, ['b_loop_bg'], bg=True)
C('G13-M', M, "until grep -q done /tmp/claude-1000/x/tasks/abc123.output; do sleep 5; done", 0, 2, ['b_loop_bg'], bg=True)
# ③ 放出追踪
for ident in (M, IW):
    s = ident[:3]
    C(f'H01-{s}', ident, 'setsid -f bash fx/light.sh', 0, 2, ['b_detach'])
    C(f'H02-{s}', ident, 'bash fx/light.sh & disown', 0, 2, ['b_detach'])
    C(f'H03-{s}', ident, 'nohup bash fx/light.sh > {D}/n.log 2>&1 &', 0, 2, ['b_detach'])
    C(f'H04-{s}', ident, "tmux new -d 'sleep 100'", 0, 2, ['b_detach'])
    C(f'H05-{s}', ident, 'screen -dm sleep 100', 0, 2, ['b_detach'])
    C(f'H06-{s}', ident, 'systemd-run --user bash fx/light.sh', 0, 2, ['b_detach'])
    C(f'H07-{s}', ident, 'coproc sleep 5', 0, 2, ['b_detach'])
    C(f'H08-{s}', ident, 'nohup bash fx/light.sh', 0, 0, ['b_detach_ok'])
    C(f'H09-{s}', ident, 'systemd-run --user --wait bash fx/light.sh', 0, 0, ['b_detach_ok'])
    C(f'H10-{s}', ident, 'setsid bash fx/light.sh', 0, 0, ['b_detach_ok'])
    C(f'H11-{s}', ident, "echo 'setsid -f x; disown'", 0, 0, ['b_detach_ok'])
    C(f'H12-{s}', ident, "bash -c 'setsid -f sleep 5'", 0, 2, ['b_detach'])
C('H20-M', M, 'bash scripts/hosts/detached-run.sh start --wall-clock within-2h --label a -- bash fx/light.sh', 0, 0, ['b_detach','ac_detached'])
C('H21-IW', IW, 'bash scripts/hosts/detached-run.sh start --wall-clock within-2h --label a -- bash fx/light.sh', 0, 2, ['b_detach_start','ac_detached'])
C('H22-IW', IW, 'bash scripts/hosts/detached-run.sh stop a', 0, 2, ['b_detach_start'])
C('H23-M', M, 'bash scripts/hosts/detached-run.sh start --wall-clock within-2h --label a -- setsid -f bash fx/light.sh', 0, 2, ['b_detach'])
# ④ run_in_background 里的单独 &
for ident in (M, IW):
    s = ident[:3]
    bgok = (ident == M)   # 子 agent 用 run_in_background 先被 ⑩ 拒
    C(f'J01-{s}', ident, 'sleep 100 &', 0, 2, ['b_amp'], bg=True)
    C(f'J02-{s}', ident, 'sleep 100 & wait', 0, 2 if not bgok else 0, ['b_amp','b_bg_sub'], bg=True)
    C(f'J03-{s}', ident, 'bash fx/light.sh > {D}/o.log 2>&1 &', 0, 2, ['b_amp'], bg=True)
    C(f'J04-{s}', ident, 'sleep 5 &', 0, 0, ['b_amp_fg'])
    C(f'J05-{s}', ident, 'a=1 & b=2 & wait', 0, 2 if not bgok else 0, ['b_amp'], bg=True)
    C(f'J06-{s}', ident, '(sleep 1 &); wait', 0, 2, ['b_amp_paren'], bg=True)
    C(f'J07-{s}', ident, 'sleep 10 & tail --pid=$! -f /dev/null', 0, 2, ['b_amp_pid'], bg=True)
    C(f'J08-{s}', ident, 'true && echo ok', 0, 2 if not bgok else 0, ['b_amp'], bg=True)
    C(f'J09-{s}', ident, 'ls >&2; ls &> {D}/o.txt; ls |& cat', 0, 2 if not bgok else 0, ['b_amp'], bg=True)
    C(f'J10-{s}', ident, 'sleep 1 & pid=$!; wait "$pid"', 0, 2 if not bgok else 0, ['b_amp'], bg=True)
# ⑩ 子 agent 的 run_in_background
for ident in (IW, ER, TF, CL, GP):
    C(f'K01-{ident[:3]}', ident, 'ls', 0, 2, ['b_bg_sub','ac_sub_bg'], bg=True)
    C(f'K02-{ident[:3]}', ident, 'ls', 0, 0, ['b_bg_sub'])
C('K03-M', M, 'ls', 0, 0, ['b_bg_main','ac_bg'], bg=True)
