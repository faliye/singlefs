import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
CAP = cases_a.CAP; RB = cases_a.RB
# ---------- 切词边角：花括号、重定向、引号、替换、heredoc，正当的放行 ----------
edge = [
 ('M01', "find scripts -name '*.sh' -exec grep -l cargo {} + | head -3", ['brace_w']),
 ('M02', "ls scripts | xargs -I{} echo {}", ['brace_w']),
 ('M03', "awk '{print $1}' README.md | head -2", ['brace_w']),
 ('M04', "echo {} {a} {a,b a{b \\{a,b\\} \"{a,b}\" '{a,b}'", ['brace_w']),
 ('M05', "git log --format='{%h}' -3", ['brace_w']),
 ('M06', "echo {,} a{,}", ['brace_w']),
 ('M07', "sed 's/{a,b}/x/' README.md | head -2", ['brace_w']),
 ('M08', "echo `date +%F` $(pwd) `pwd`", ['redir']),
 ('M09', "echo $'a\\tb' $\"x\"", ['redir']),
 ('M10', "exec 3>{D}/m10.txt; echo hi >&3; exec 3>&-", ['redir']),
 ('M11', "ls &>{D}/m11.txt; ls >|{D}/m11b.txt; ls 2>>{D}/m11c.txt", ['redir']),
 ('M12', "cat <<-EOF\n\tindent $(date +%F)\n\tEOF", ['heredoc_strip']),
 ('M13', "cat <<EOF\nvalue: $(echo hi) and `pwd`\nEOF", ['heredoc_strip']),
 ('M14', "cat <<'A' > {D}/m14a.txt; cat <<'B' > {D}/m14b.txt\none\nA\ntwo\nB", ['heredoc_strip']),
 ('M15', "cat <<'EOF' | bash\nls\nEOF", ['stdin_shell']),
 ('M16', "echo \"a & b\"; echo 'x && y'", ['b_amp_fg']),
 ('M17', "x=$(sleep 1; echo hi); echo $x", ['b_loop']),
 ('M18', "ln -sf ../fx/light.sh {D}/m18-link.sh; ln -s /usr/bin/ls {D}/m18b; ln fx/light.sh {D}/m18c", ['redir']),
 ('M19', "install -m 755 fx/light.sh {D}/m19.sh; cp -t {D} fx/light.sh fx/heavy-all.sh", ['b_scope_land']),
 ('M20', "A=1 B=2 env C=3 ls; export D1=1 D2=2; declare -x D3=3", ['env_opts']),
 ('M21', "cmd=ls; $cmd -la", ['invisible']),
 ('M22', "ls ${NOPE:-.}", ['brace_w']),
 ('M23', "printf '%s\\n' a b | while read -r l; do echo $l; done", ['b_loop_ok']),
 ('M24', "git log --grep=kill | head -2; echo kill 1 2; ls kill 2>/dev/null", ['b_kill']),
 ('M25', "pgrep -x nothing_here_xyz | head -2; ps aux | head -2", ['b_kill_pkill']),
 ('M26', "kill -s TERM \"$pid\" 2>/dev/null || true", ['b_kill_ok']),
 ('M27', "kill -SIGTERM \"$pid\"; kill -- \"$pid\"; kill -9 \"$!\"", ['b_kill_ok']),
 ('M28', "kill \"${pids[0]}\"", ['b_kill_ok']),
 ('M29', "timeout -s KILL 5 sleep 1", ['b_kill']),
 ('M30', "rm -rf ./*.tmp; rm -rf ~/.local/share/singlefs/work/some-dir/; rm -rf /tmp/mydir-*", ['b_rm_other','b_rm_ok']),
 ('M31', "rm -rf \"$UNSET_DIR\"", ['b_rm_other']),
 ('M32', "herd7 -version; herd7 --version", ['herd7']),
 ('M33', "qemu-img info disk.img 2>/dev/null; qemu-img create {D}/m33.img 1M", ['qemu']),
 ('M34', "python3 - <<'EOF'\nprint(1 & 2)\nEOF", ['b_amp','heredoc_strip']),
 ('M35', "curl -s 'http://localhost:1/x?a=1&b=2' | head -1", ['b_amp']),
 ('M36', "sed 's/a/&b/' README.md | head -1", ['b_amp']),
]
for n, cmd, cites in edge:
    for ident in (M, IW, TF):
        C(f'{n}-{ident[:3]}', ident, cmd, 0, 0, cites)
# 后台里引号里的 & 不拒（主 agent）
for n, cmd in [('M40', 'echo "a & b"'), ('M41', "curl -s 'http://localhost:1/x?a=1&b=2'"), ('M42', "sed 's/a/&b/' README.md"), ('M43', "python3 - <<'EOF'\nprint(1 & 2)\nEOF"), ('M44', 'ls &>/dev/null')]:
    C(f'{n}-M', M, cmd, 0, 0, ['b_amp'], bg=True)
# 轻型三条：子 agent 要经内存包装
CPR = 'scripts/crash/crash-point-replay.sh'
C('M50-IW', IW, 'bash ' + CPR + ' --quick', 2, 0, ['light','memcap'], cwd='repo')
C('M51-IW', IW, 'bash scripts/hosts/run-with-memory-cap.sh 4G bash ' + CPR + ' --quick --local-only', 0, 0, ['light','light_who'], cwd='repo')
C('M52-IW', IW, 'bash scripts/hosts/run-with-memory-cap.sh 4G bash ' + CPR + ' --quick --full', 2, 0, ['light_main'], cwd='repo')
C('M53-IW', IW, 'bash scripts/hosts/run-with-memory-cap.sh 4G bash ' + CPR + ' --quick --item foo', 0, 0, ['light'], cwd='repo')
C('M54-M', M, 'bash ' + CPR + ' --quick --local-only', 0, 0, ['light_main'], cwd='repo')
C('M55-IW', IW, 'bash scripts/hosts/run-with-memory-cap.sh 4G bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick .', 0, 0, ['light','light_who'], cwd='repo')
C('M56-IW', IW, 'bash scripts/hosts/run-with-memory-cap.sh 4G bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick --full', 2, 0, ['light_main'], cwd='repo')
C('M57-M', M, 'bash ' + CPR + ' --quick --full', 2, 0, ['light_main','prefix_need'], cwd='repo')
C('M58-M', M, 'LIGHT=1 CRASH_POINT_REPLAY_BREAK=x bash ' + CPR + ' --quick', 2, 0, ['light_main'], cwd='repo', note='设了弄坏开关变量就不认轻型')
