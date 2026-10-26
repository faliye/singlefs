import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
OTHERS = ['kb-scribe', 'sweep', 'prior-art', 'three-way-materials', 'three-way-defense', 'experiment-designer']
# ---------- 其余项目子 agent 各一组：只读命令放行，重型与 claude 会话与后台照拒 ----------
for ident in OTHERS:
    s = ident
    C(f'S01-{s}', ident, 'ls -la; grep -n cargo README.md | head -3; sha256sum README.md', 0, 0, ['ac_read'])
    C(f'S02-{s}', ident, "nice -n 19 python3 scripts/kb/kb-sections.py .claude/kb/decisions.md > {D}/s.txt 2>&1; echo rc=$?; tail -n 20 {D}/s.txt", 0, 0, ['ac_rc'])
    C(f'S03-{s}', ident, 'cargo test --all', 2, 0, ['full','sub_refuse'])
    C(f'S04-{s}', ident, 'bash .claude/gate.d/checker-tier-lkmm.sh', 2, 0, ['herd7','sub_refuse'])
    C(f'S05-{s}', ident, 'claude -p hi', 0, 2, ['b_claude'])
    C(f'S06-{s}', ident, 'ls', 0, 2, ['b_bg_sub'], bg=True)
    C(f'S07-{s}', ident, 'until grep -q ok log.txt; do sleep 5; done', 0, 2, ['b_loop'])
    C(f'S08-{s}', ident, "cp {D}/a.md {D}/b.md", 0, 0, ['b_scope_land'])
    C(f'S09-{s}', ident, "bash scripts/hosts/work-directory-ownership.sh claim {D} --purpose 'x'", 0, 0, ['cmdpos'])
