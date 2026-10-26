import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
KB = 'kb-scribe'
M = 'main'
R = dict(cwd='repo', wr=True)
# ⑫ 有写范围的项目子 agent 在 Bash 里写范围之外的仓内文件
C('U01-CL', CL, 'echo x > .claude/kb/foo.md', 0, 2, ['b_scope','b_scope_land'], **R)
C('U02-CL', CL, 'echo x > records/foo.md', 0, 0, ['b_scope_land'], **R)
C('U03-CL', CL, 'cp {D}/a.md records/b.md', 0, 0, ['b_scope_land'], **R)
C('U04-CL', CL, 'echo x > $UNSET/foo.md', 0, 2, ['b_scope_unknown'], **R)
C('U05-CL', CL, 'echo x > {D}/$(date +%s).txt', 0, 2, ['b_scope_unknown'], **R)
C('U06-CL', CL, 'echo x > {D}/*.txt', 0, 2, ['b_scope_unknown'], **R)
C('U07-CL', CL, 'echo x > {D}/{a,b}.txt', 0, 2, ['b_scope_unknown'], **R)
C('U08-CL', CL, 'echo x > $HOME/.local/share/singlefs/work/foo.txt', 0, 0, ['b_scope_land'], **R)
C('U09-CL', CL, 'while read f; do echo x > {D}/$f; done < {D}/list', 0, 2, ['b_scope_unknown'], **R)
C('U10-CL', CL, 'for f in $FILES; do echo x > {D}/$f; done', 0, 2, ['b_scope_unknown'], **R)
C('U11-CL', CL, 'cd records && echo x > foo.md', 0, 0, ['b_scope_land'], **R)
C('U12-IW', IW, 'cd records && echo x > foo.md', 0, 2, ['b_scope_land'], **R)
C('U13-IW', IW, 'echo x > records/foo.md', 0, 2, ['b_scope'], **R)
C('U14-IW', IW, 'echo ok > scripts/x.sh', 0, 2, ['b_scope'], **R)
C('U15-IW', IW, 'echo x > crates/singlefs-core/src/lib.rs', 0, 0, ['b_scope_land'], **R)
C('U16-TW', TW, 'echo x > crates/x.rs', 0, 2, ['b_scope'], **R)
C('U17-TW', TW, 'cp {D}/h.sh scripts/foo.sh', 0, 0, ['b_scope_land'], **R)
C('U18-INV', INV, 'echo x > crates/x.rs', 0, 0, ['b_scope_land'], **R)
C('U19-INV', INV, 'echo x > records/x.md', 0, 2, ['b_scope'], **R)
C('U20-GP', GP, 'echo x > records/x.md', 0, 2, ['b_scope_builtin'], **R)
C('U21-GP', GP, 'echo x > /etc/foo.conf', 0, 2, ['b_scope_builtin'], **R)
C('U22-GP', GP, 'echo x > /tmp/foo', 0, 2, ['b_scope_builtin'], **R)
C('U23-GP', GP, 'echo x > /dev/null; ls 2>/dev/null; echo x > /dev/stderr', 0, 0, ['b_scope_builtin'], **R)
C('U24-GP', GP, 'echo x > {D}/o.txt', 0, 0, ['b_scope_builtin'], **R)
C('U25-TF', TF, 'echo x > records/x.md', 0, 0, ['b_scope_none'], **R, note='three-way-forward 不在写范围表里')
C('U26-M', M, 'echo x > records/x.md', 0, 0, ['b_scope_none'], **R)
C('U27-CL', CL, 'd=$(mktemp -d); echo x > $d/f', 0, 0, ['b_scope_land'], **R)
C('U28-CL', CL, 'cd {D} && echo x > out.txt', 0, 0, ['b_scope_land'], **R)
C('U29-KB', KB, 'python3 scripts/kb/replace-once.py .claude/kb/decisions.md old new', 0, 0, ['b_scope_land'], **R)
C('U30-IW', IW, 'python3 scripts/kb/replace-once.py .claude/kb/decisions.md old new', 0, 2, ['b_scope'], **R)
C('U31-IW', IW, 'python3 scripts/kb/replace-once.py .claude/kb/decisions.md old new --dry-run', 0, 0, ['b_scope'], **R)
C('U32-CL', CL, 'tee {D}/log.txt < /dev/null', 0, 0, ['b_scope_land'], **R)
C('U33-TW', TW, 'echo x >> .claude/hooks/agent-write-scope.tsv', 0, 0, ['b_scope_land'], **R)
C('U34-ER', ER, 'echo x >> scripts/experiment-apparatus/replay.sh', 0, 0, ['b_scope_land'], **R)
C('U35-ER', ER, 'echo x > .claude/gate.d/doc-text.sh', 0, 2, ['b_scope'], **R)
# ⑬ 子 agent 起 claude 会话
for n, cmd, e in [
 ('V01', 'claude -p hi', 2), ('V02', '~/.local/bin/claude --version', 2), ('V03', 'nice claude -p x', 2),
 ('V04', "bash -c 'claude -p x'", 2), ('V05', 'command -v claude', 0), ('V06', 'which claude', 0),
 ('V07', 'grep -n claude README.md', 0), ('V08', 'echo claude', 0), ('V09', '"$CLAUDE_CODE_EXECPATH" -p x', 2),
 ('V10', 'timeout 60 claude -p x', 2), ('V11', "bash <<'EOF'\nclaude -p x\nEOF", 2),
 ('V12', "cat > {D}/c.sh <<'EOF'\nclaude -p x\nEOF", 0), ('V13', "echo 'claude -p x'", 0),
]:
    C(f'{n}-IW', IW, cmd, 0, e, ['b_claude','b_claude_ok','ac_claude'])
C('V20-M', M, 'claude --version', 0, 0, ['b_claude'])
# ⑮ 暂存区非空时清整个索引
for ident in (M, IW):
    s = ident[:3]
    for n, cmd, e, cw in [
     ('W01', 'git reset', 2, 'staged'), ('W02', 'git reset --hard', 2, 'staged'), ('W03', 'git reset --mixed HEAD', 2, 'staged'),
     ('W04', 'git restore --staged .', 2, 'staged'), ('W05', 'git restore -S :/', 2, 'staged'), ('W06', 'git reset HEAD', 2, 'staged'),
     ('W07', 'git reset -q', 2, 'staged'), ('W08', 'git reset HEAD -- b.md', 0, 'staged'), ('W09', 'git restore --staged -- b.md', 0, 'staged'),
     ('W10', 'git reset b.md', 0, 'staged'), ('W11', 'git reset --soft HEAD~1', 0, 'staged'),
     ('W12', 'git -C {D}/stagedrepo reset', 2, 'copy'), ('W13', 'git -C {D}/emptyrepo reset', 0, 'staged'),
     ('W14', 'git reset', 0, 'empty'), ('W15', 'git restore --staged .', 0, 'empty'),
     ('W16', 'git status --short; git diff --cached --stat', 0, 'staged'), ('W17', 'git add b.md', 0, 'staged'),
     ('W18', 'cd {D}/stagedrepo && git reset', 2, 'empty'), ('W19', 'git reset --keep', 2, 'staged'), ('W20', 'git restore --staged *', 2, 'staged'),
    ]:
        C(f'{n}-{s}', ident, cmd, 0, e, ['b_index','b_index_ok'], cwd=cw)
# ⑭ 主 agent 用 run_in_background 起命令（只核没有历史也没有长活声明的放行）
C('X01-M', M, 'SINGLEFS_HEAVY_TESTS=commit bash scripts/gate/gate-staged.sh', 0, 0, ['ac_bg'], bg=True, **R)
C('X02-M', M, 'bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label x -- bash fx/light.sh', 0, 0, ['ac_detached'], bg=True, **R)
# ⑫ 花括号：文件头把花括号展开列在「落点算不出」里
C('U40-IW', IW, 'tee records/{a,b}.md < /dev/null', 0, 2, ['b_scope_unknown','b_scope'], **R)
C('U41-CL', CL, 'tee .claude/kb/{a,b}.md < /dev/null', 0, 2, ['b_scope_unknown','b_scope'], **R)
C('U42-CL', CL, 'tee {D}/{a,b}.md < /dev/null', 0, 2, ['b_scope_unknown'], **R)
C('U44-CL', CL, 'tee records/{a,b}.md < /dev/null', 0, 2, ['b_scope_unknown'], **R, note='落点在范围内，但花括号')
