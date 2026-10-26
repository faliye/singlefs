import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
RB = cases_a.RB; CAP = cases_a.CAP
# 主工作区里子 agent 起 cargo：包装次序两种都应放行
C('N01-IW', IW, 'bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-harness', 0, 0, ['target_ok','ac_cap'], cwd='repo')
C('N02-IW', IW, 'bash scripts/hosts/run-build-directory.sh bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness', 0, 0, ['target_ok','ac_cap'], cwd='repo')
C('N03-IW', IW, 'bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-harness', 0, 0, ['target_ok','ac_cap'], cwd='repo')
C('N04-IW', IW, 'CARGO_TARGET_DIR={D}/tgt bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness', 0, 0, ['target_ok','ac_cap'], cwd='repo')
C('N05-IW', IW, 'bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness', 2, 0, ['target','ac_cap'], cwd='repo', note='主工作区里默认编进 <仓根>/target')
# 注释、引号、中文
more = [
 ('N10', "# 先看一眼\nls # don't do this 'x\necho ok", ['redir']),
 ('N11', "echo \"it's\" # comment 'x", ['redir']),
 ('N12', "grep -rn '「重型」' .claude/rules | head -3", ['redir']),
 ('N13', 'git commit --dry-run -m "feat: x" -m "$(cat <<\'EOF\'\n细节\nEOF\n)"', ['heredoc_strip']),
 ('N14', 'time (cargo fmt --check)', ['cmdpos']),
 ('N15', '! grep -q zzz README.md && echo none', ['cmdpos']),
 ('N16', 'files=(a b c); for f in "${files[@]}"; do echo $f; done; files+=(d)', ['cmdpos']),
 ('N19', 'CMD="cargo test --all"; echo "$CMD"', ['invisible']),
 ('N20', 'grep x <<< "cargo test --all"', ['stdin_shell']),
 ('N21', '[[ -f README.md ]] || echo none', ['brace_w']),
 ('N22', 'exec > >(tee {D}/n22.log) 2>&1; echo hi', ['redir']),
 ('N23', 'if grep -q herd7 README.md; then echo has; fi', ['cmdpos']),
 ('N24', 'echo herd7 qemu-system-x86_64 mutate.sh gate.sh', ['cmdpos']),
 ('N25', 'ls herd7 qemu-system-x86_64 scripts/mutation/mutate.sh 2>/dev/null', ['cmdpos']),
 ('N26', 'grep -c "kill" README.md; man kill 2>/dev/null | head -1', ['b_kill']),
 ('N27', 'echo $(( 1 & 2 ))', ['b_amp']),
]
for n, cmd, cites in more:
    for ident in (M, IW, TF):
        C(f'{n}-{ident[:3]}', ident, cmd, 0, 0, cites, cwd='repo' if ident == M else 'copy')
