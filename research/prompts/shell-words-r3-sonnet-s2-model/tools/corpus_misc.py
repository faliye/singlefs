"""家族 M：claude 会话、detached-run、看门狗、kill、git 索引、research/results 覆盖、同 inode 改脚本、子 agent 写仓内文件。"""
from corpus_base import add

CAP = "bash {R}/scripts/hosts/capped.sh 31"
# ⑬ claude 会话（只拒子 agent）
for name, cmd in [
    ("plain", "claude -p hi"), ("abs", "~/.local/bin/claude -p hi"), ("execpath", "\"$CLAUDE_CODE_EXECPATH\" -p hi"),
    ("execpath-brace", "\"${CLAUDE_CODE_EXECPATH}\" -p hi"), ("nice", "nice claude -p hi"), ("env", "env A=1 claude -p hi"), ("timeout", "timeout 5 claude -p hi"),
    ("capped", CAP + " claude -p hi"), ("bashc", "bash -c 'claude -p hi'"), ("exec", "exec claude -p hi"), ("command", "command claude -p hi"),
    ("sudo", "sudo claude -p hi"), ("xargs", "echo hi | xargs claude -p"), ("ssh", "ssh localhost 'claude -p hi'"), ("var", "c=claude; $c -p hi"),
    ("herestring", "bash <<< 'claude -p hi'"), ("cmdsub", "echo $(claude -p hi)"), ("heredoc", "bash <<'EOF'\nclaude -p hi\nEOF"),
    ("pipe-stdin", "echo hi | claude -p"), ("amp", "claude -p hi & wait"), ("find-exec", "find . -maxdepth 0 -exec claude -p hi \\\;"),
    ("env-S", "env -S 'claude -p hi'"), ("env-i-abs", "env -i {R}/.claude/hooks/../../model/x/claude 2>/dev/null; claude -p"),
    ("command-v", "command -v claude"), ("which", "which claude"), ("grep-arg", "grep -rn claude /dev/null"), ("echo-data", "echo claude"),
    ("quoted-data", "echo 'claude -p hi'"), ("type", "type claude"), ("claude-ver-dir", "ls ~/.claude >/dev/null; echo claude"),
]:
    add(f"M-claude-{name}", cmd, roles=("sub", "main"), cwd="R", prefix_variants=False)

# detached-run.sh（子 agent 的 start / stop 拒，list / watch 放；主 agent 都放）
DET = "bash {R}/scripts/hosts/detached-run.sh"
for name, cmd in [
    ("start", DET + " start --wall-clock within-2h --label x -- sleep 1"), ("stop", DET + " stop x"), ("list", DET + " list"), ("watch", DET + " watch"),
    ("start-cap", CAP + " " + DET + " start --wall-clock within-2h --label x -- sleep 1"), ("start-nice", "nice " + DET + " start --wall-clock within-2h --label x -- sleep 1"),
    ("start-bashc", "bash -c '" + DET + " start --wall-clock within-2h --label x -- sleep 1'"), ("start-env", "env A=1 " + DET + " start --wall-clock within-2h --label x -- sleep 1"),
    ("start-var", "S={R}/scripts/hosts/detached-run.sh; bash $S start --wall-clock within-2h --label x -- sleep 1"),
    ("start-rel", "cd {R} && bash scripts/hosts/detached-run.sh start --wall-clock within-2h --label x -- sleep 1"),
    ("start-direct", "{R}/scripts/hosts/detached-run.sh start --wall-clock within-2h --label x -- sleep 1"),
    ("start-source", "source {R}/scripts/hosts/detached-run.sh start --wall-clock within-2h --label x -- sleep 1"),
]:
    add(f"M-detached-{name}", cmd, roles=("sub", "main"), cwd="R", prefix_variants=False)

# ① 看门狗：主 agent 才起；run_in_background 为真且命令里没有 &、nohup、disown、setsid、丢进 /dev/null
WATCH = "bash {R}/scripts/agents/watch.sh"
for name, cmd in [("plain", WATCH), ("amp", WATCH + " &"), ("nohup", "nohup " + WATCH), ("devnull", WATCH + " >/dev/null 2>&1"), ("disown", WATCH + " & disown"),
                  ("setsid", "setsid " + WATCH), ("processes", WATCH + " --processes"), ("selftest", WATCH + " --selftest"), ("report", WATCH + " --report"),
                  ("dry", WATCH + " --dry-run"), ("grep-data", "grep -n watch.sh /dev/null"), ("cat-data", "cat {R}/scripts/agents/watch.sh >/dev/null"),
                  ("bash-n", "bash -n {R}/scripts/agents/watch.sh"), ("pyw", "python3 {R}/scripts/agents/agent-watch.py watch"),
                  ("pyw-amp", "python3 {R}/scripts/agents/agent-watch.py watch &"), ("cap-watch", CAP + " " + WATCH), ("bashc", "bash -c '" + WATCH + "'"),
                  ("ack", WATCH + " --ack 1:x"), ("devnull-in", WATCH + " </dev/null"), ("tee-null", WATCH + " | tee /dev/null"),
                  ("devnull-stderr", WATCH + " 2>/dev/null"), ("exec", "exec " + WATCH), ("env", "env A=1 " + WATCH), ("sub-sh", "( " + WATCH + " )")]:
    for bg in (False, True):
        noise = any(k in cmd for k in (" &", "nohup", "disown", "setsid", ">/dev/null", "> /dev/null", "2>/dev/null", "| tee /dev/null")) and "grep" not in cmd and "cat " not in cmd
        silent = any(k in name for k in ("selftest", "report", "dry", "grep", "cat-data", "bash-n"))
        starts = not silent and "grep-data" != name
        want = [("W1", "看门狗没用 run_in_background 起，或带 &/nohup/disown/setsid/丢 /dev/null（①）")] if starts and (not bg or noise) else []
        add(f"M-watch{int(bg)}-{name}", cmd, roles=("main",), bg=bg, cwd="R", want=want, prefix_variants=False,
            note="手标；依据 detector ①；--selftest/--report/--dry-run 与 grep/cat/bash -n 不算起看门狗")

# ⑥ 终止进程（只放安全的几种进真 bash）
for name, cmd, bad in [("bang", "sleep 30 & kill $!", False), ("job", "sleep 30 & kill %1", False), ("pid-var", "sleep 30 & p=$!; kill -9 $p", False),
                       ("check", "kill -0 $$", False), ("zero", "kill 0", True), ("two", "sleep 30 & a=$!; sleep 30 & b=$!; kill $a $b", True),
                       ("neg-group", "sleep 30 & kill -- -$$", True), ("loop", "sleep 30 & for p in $(jobs -p); do kill $p; done", True),
                       ("cmdsub", "sleep 30 & kill $(jobs -p)", True), ("pgrep-sub", "sleep 30 & kill $(pgrep -P $$ sleep)", True)]:
    add(f"M-kill-{name}", cmd, roles=("main", "sub"), cwd="R", prefix_variants=False,
        want=[("K6", "终止写法不是点名一个自己起的进程号（⑥）")] if bad else [], note="手标；依据 agent-common ⑥；放行的是点名一个：kill \"$!\"、kill %1")

# ⑮ git 索引
GS = "echo a > {R}/a.txt; echo b > {R}/b.txt; git -C {R} add a.txt b.txt"
for name, cmd in [("bare", "git reset"), ("hard", "git reset --hard"), ("soft", "git reset --soft HEAD"), ("path", "git reset -- a.txt"), ("head-path", "git reset HEAD -- a.txt"),
                  ("restore-dot", "git restore --staged ."), ("restore-root", "git restore --staged :/"), ("restore-path", "git restore --staged a.txt"),
                  ("C-reset", "git -C {R} reset"), ("cd-reset", "cd {R} && git reset"), ("head", "git reset HEAD"), ("head-dot", "git reset HEAD -- ."), ("quiet", "git reset -q"),
                  ("mixed", "git reset --mixed"), ("restore-S-dot", "git restore -S ."), ("restore-staged-wt", "git restore --staged --worktree ."),
                  ("reset-var", "x=reset; git $x"),
                  ("bashc", "bash -c 'git reset'"), ("amp-and", "true && git reset")]:
    add(f"M-gitindex-{name}", cmd, roles=("main", "sub"), cwd="R", setup=GS, prefix_variants=False)

# ⑤ research/results 未跟踪产物
RS = "mkdir -p {R}/research/results; echo old > {R}/research/results/old.txt"
for name, cmd in [("gt", "echo new > research/results/old.txt"), ("append", "echo new >> research/results/old.txt"), ("clobber", "echo new >| research/results/old.txt"),
                  ("amp", "echo new &> research/results/old.txt"), ("tee", "echo new | tee research/results/old.txt >/dev/null"), ("tee-a", "echo new | tee -a research/results/old.txt >/dev/null"),
                  ("cp", "echo n > {U}/n.txt; cp {U}/n.txt research/results/old.txt"), ("mv", "echo n > {U}/n.txt; mv {U}/n.txt research/results/old.txt"),
                  ("dd", "echo new | dd of=research/results/old.txt 2>/dev/null"), ("truncate", "truncate -s0 research/results/old.txt"), ("newname", "echo new > research/results/new.txt"),
                  ("abs", "echo new > {R}/research/results/old.txt"), ("cd-rel", "cd research/results && echo new > old.txt"), ("var", "f=research/results/old.txt; echo new > $f"),
                  ("python", "python3 -c \"open('research/results/old.txt','w').write('new')\""), ("sed-i", "sed -i s/old/new/ research/results/old.txt"),
                  ("printf", "printf new > research/results/old.txt"), ("install", "echo n > {U}/n.txt; install {U}/n.txt research/results/old.txt"),
                  ("brace-group", "{ echo new; } > research/results/old.txt"), ("fd-redir", "echo new 1> research/results/old.txt"), ("exec-redir", "exec > research/results/old.txt; echo new")]:
    add(f"M-results-{name}", cmd, roles=("main",), cwd="R", setup=RS, stage=False, prefix_variants=False)

# ⑦ 同 inode 改已有脚本
for name, cmd in [("gt", "echo '# x' > scripts/hosts/capped.sh"), ("append", "echo '# x' >> scripts/hosts/capped.sh"), ("clobber", "echo '# x' >| scripts/hosts/capped.sh"),
                  ("amp", "echo '# x' &> scripts/hosts/capped.sh"), ("tee", "echo '# x' | tee scripts/hosts/capped.sh >/dev/null"), ("tee-a", "echo '# x' | tee -a scripts/hosts/capped.sh >/dev/null"),
                  ("cp", "echo '# x' > {U}/n.sh; cp {U}/n.sh scripts/hosts/capped.sh"), ("cp-remove", "echo '# x' > {U}/n.sh; cp --remove-destination {U}/n.sh scripts/hosts/capped.sh"),
                  ("mv", "echo '# x' > {U}/n.sh; mv {U}/n.sh scripts/hosts/capped.sh"), ("dd", "echo '# x' | dd of=scripts/hosts/capped.sh 2>/dev/null"), ("truncate", "truncate -s0 scripts/hosts/capped.sh"),
                  ("sed-i", "sed -i 's/exec/exec/' scripts/hosts/capped.sh"), ("python-w", "python3 -c \"open('scripts/hosts/capped.sh','w').write('x')\""),
                  ("python-a", "python3 -c \"open('scripts/hosts/capped.sh','a').write('x')\""), ("perl-i", "perl -pi -e 's/exec/exec/' scripts/hosts/capped.sh"),
                  ("install", "echo '# x' > {U}/n.sh; install -m755 {U}/n.sh scripts/hosts/capped.sh"), ("new-file", "echo '# x' > scripts/hosts/brand-new.sh"),
                  ("abs", "echo '# x' > {R}/scripts/hosts/capped.sh"), ("cd-rel", "cd scripts/hosts && echo '# x' > capped.sh"), ("var", "f=scripts/hosts/capped.sh; echo '# x' > $f"),
                  ("py-file", "echo '# x' > scripts/agents/agent-watch.py"), ("brace-group", "{ echo '# x'; } > scripts/hosts/capped.sh"), ("exec-redir", "exec > scripts/hosts/capped.sh; echo '# x'"),
                  ("sed-n-redirect", "sed -n p scripts/hosts/capped.sh > scripts/hosts/capped.sh"), ("ed", "printf 'a\\nx\\n.\\nw\\nq\\n' | ed -s scripts/hosts/capped.sh >/dev/null 2>&1"),
                  ("rsync-inplace", "echo '# x' > {U}/n.sh; rsync --inplace {U}/n.sh scripts/hosts/capped.sh"), ("cat-heredoc", "cat > scripts/hosts/capped.sh <<'EOF'\n# x\nEOF")]:
    add(f"M-inode-{name}", cmd, roles=("main",), cwd="R", prefix_variants=False)

# ⑧ ⑫ 子 agent 往仓内写
for name, cmd in [("sed-i-crates", "sed -i 's/a/b/' crates/singlefs-checker-tier/Cargo.toml"), ("gt-crates", "echo x > crates/new.rs"), ("append-crates", "echo x >> crates/singlefs-checker-tier/Cargo.toml"),
                  ("tee-scripts", "echo x | tee scripts/new.txt >/dev/null"), ("cp-scripts", "echo x > {W}/n.txt; cp {W}/n.txt scripts/new.txt"), ("mv-scripts", "echo x > {W}/n.txt; mv {W}/n.txt scripts/new.txt"),
                  ("python-crates", "python3 -c \"open('crates/new.rs','w').write('x')\""), ("python-path", "python3 -c \"from pathlib import Path; Path('crates/new.rs').write_text('x')\""),
                  ("python-replace", "echo x > {W}/n.txt; python3 -c \"import os; os.replace('{W}/n.txt','scripts/new.txt')\""),
                  ("draft-ok", "echo x > {W}/note.txt"), ("tmp-ok", "echo x > /tmp/claude-1000/zz-note.txt"), ("copy-ok", "cd {C} && echo x > new.rs"), ("copy-sed-ok", "cd {C} && sed -i 's/a/b/' Cargo.toml"),
                  ("copy-abs-ok", "echo x > {C}/new2.rs"), ("var-cd-copy", "d={C}; cd $d && echo x > new3.rs"), ("sed-i-draft-ok", "echo a > {W}/s.txt; sed -i 's/a/b/' {W}/s.txt"),
                  ("python-draft-ok", "python3 -c \"open('{W}/n2.txt','w').write('x')\""), ("heredoc-data-ok", "cat > {W}/{I}-n3.py <<'EOF'\nopen('crates/x.rs','w').write('x')\nEOF"),
                  ("sed-i-brace", "sed -i 's/a/b/' crates/{singlefs-checker-tier,zz}/Cargo.toml"), ("sed-i-glob", "sed -i 's/a/b/' crates/*/Cargo.toml"),
                  ("replace-once-draft", "echo a > {W}/r.txt; python3 {R}/scripts/kb/replace-once.py {W}/r.txt a b >/dev/null 2>&1; true")]:
    add(f"M-write-{name}", cmd, roles=("sub", "editor"), cwd="R", prefix_variants=False)
