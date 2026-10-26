import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
W = '~/.local/share/singlefs/work'
# ---------- 当数据写的危险字样不拒 ----------
data = [
 ('Z01', "cat > {D}/note.md <<'EOF'\n# 笔记\nuntil x; do sleep 1; done\nkill -9 -1\nrm -rf ~/.local/share/singlefs/work/*\nsetsid -f foo &\ncargo test --all\nbash .claude/gate.d/checker-tier-lkmm.sh\nEOF", ['b_loop','b_rm_other','b_detach_ok','heredoc_strip']),
 ('Z02', "python3 -c \"print('kill -9 -1; rm -rf /tmp/*')\"", ['b_rm_other','b_kill_ok']),
 ('Z03', "git commit -m 'kill -9 -1 and rm -rf /tmp/* were refused; cargo test --all is heavy'", ['b_rm_other','cmdpos']),
 ('Z04', "git log --oneline --grep='rm -rf' | head -3", ['b_rm_other','cmdpos']),
 ('Z05', "echo 'rm -rf ~/.local/share/singlefs/work/*' >> {D}/notes.md", ['b_rm_other','b_inode_ok']),
 ('Z06', "grep -n 'until' .claude/hooks/bash-command-detector.sh | head -3; grep -rn 'setsid -f' scripts | head -3; grep -n nohup .claude/main-agent.md | head", ['b_loop','b_detach_ok','cmdpos']),
 ('Z07', "awk '/while/ {print}' scripts/hosts/capped.sh | head -3", ['b_loop_ok']),
 ('Z08', 'git commit -m "$(cat <<\'EOF\'\nfix: 修一处\n\nuntil x; do sleep 1; done 不再拒\nEOF\n)"', ['heredoc_strip']),
 ('Z09', "gh pr create --title t --body \"$(cat <<'EOF'\n## summary\n- cargo test --all\nEOF\n)\"", ['heredoc_strip']),
 ('Z10', "echo 'claude -p x'; echo 'sleep 100 &'", ['b_claude_ok','b_amp_fg']),
 ('Z11', "sed -n '1,20p' .claude/gate.d/checker-tier-qemu-device-streams.sh | head", ['cmdpos']),
 ('Z12', "bash -n .claude/gate.d/checker-tier-crash-point-replay.sh && echo syntax-ok", ['n_syntax']),
 ('Z13', "for f in .claude/gate.d/checker-tier-*.sh; do bash -n \"$f\" || echo bad $f; done", ['n_syntax','glob']),
 ('Z14', "diff <(sed -n 1,5p README.md) <(sed -n 1,5p README.en.md)", ['redir']),
 ('Z15', "ps -o pid,args -u \"$(id -u)\" | head -5", ['ac_ps']),
 ('Z16', "kill -0 999999 && echo alive || echo gone", ['b_kill_loop']),
 ('Z17', "trap 'rm -f {D}/lock' EXIT; echo $$", ['b_rm_other']),
 ('Z18', f"rm -rf {W}/shell-words-r2-sonnet-s2/*", ['b_rm_ok']),
 ('Z19', "rsync -a --exclude target ./ {D}/copy-z19/ >/dev/null", ['b_scope_land']),
 ('Z20', "cp -r crates {D}/copy-z20/", ['b_scope_land']),
 ('Z21', "bash scripts/hosts/run-with-memory-cap.sh 4G bash -c 'cd {D} && python3 -c \"print(1)\"'", ['ac_cap']),
 ('Z22', "ls scripts/*/ | head; find scripts -name '*.sh' -newer README.md | head -3", ['glob']),
 ('Z23', "sleep 5 && ls", ['b_loop']),
 ('Z24', "wait", ['b_amp']),
 ('Z25', "cat .claude/gate.d/checker-tier-lkmm.sh | head -5; less -F scripts/mutation/mutate.sh | head -3", ['cmdpos']),
 ('Z26', "head -5 .claude/gate.d/checker-tier-crates-mutation-replay.sh; tail -5 scripts/crash/crash-point-replay.sh", ['cmdpos']),
 ('Z27', "stat scripts/devices/build-patched-qemu.sh; file scripts/devices/vm-bench.sh", ['cmdpos']),
 ('Z28', "bash scripts/hosts/capped.sh 31 python3 -I scripts/kb/kb-sections.py .claude/kb/decisions.md > {D}/o.txt 2>&1; echo rc=$?; tail -n 20 {D}/o.txt", ['ac_rc','ac_capped']),
 ('Z29', "TZ=Asia/Tokyo date +%F", ['cmdpos']),
 ('Z30', "ssh -V 2>&1 | head -1", ['cmdpos']),
]
for n, cmd, cites in data:
    for ident in (M, IW, TF, CL, GP):
        # 主 agent 的 Z19/Z20/Z28 命令照写（不涉及仓内写）
        C(f'{n}-{ident[:3]}', ident, cmd, 0, 0, cites, **({'cwd': 'repo'} if ident == M else {}))
