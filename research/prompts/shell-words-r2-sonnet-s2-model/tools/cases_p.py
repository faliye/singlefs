import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
# ---------- 派发提示指定的喂法：cwd 就是草稿目录本身 ----------
dr = dict(cwd='draft')
for ident in (M, IW, TF):
    s = ident[:3]
    C(f'Q01-{s}', ident, 'ls -la', 0, 0, ['ac_read'], **dr)
    C(f'Q02-{s}', ident, 'git status --short | head -3', 0, 0, ['ac_read'], **dr)
    C(f'Q03-{s}', ident, "bash scripts/hosts/work-directory-ownership.sh claim {D} --purpose 'x'", 0, 0, ['cmdpos'], **dr)
    C(f'Q04-{s}', ident, 'cargo test --all', 2, 0, ['full','sub_refuse'], **dr)
    C(f'Q05-{s}', ident, "cd \"$(git rev-parse --show-toplevel)\" && bash <<'EOF'\ncargo test --all\nEOF", 2, 0, ['stdin_shell','sub_refuse'], **dr)
    C(f'Q06-{s}', ident, 'env =1 cargo test --all', 2, 0, ['env_opts','full'], **dr)
    C(f'Q07-{s}', ident, '{cargo,} test --all', 2, 0, ['brace_w','full'], **dr)
    C(f'Q08-{s}', ident, 'cargo test -p singlefs-harness -- --list', 2 if ident != M else 0, 0, ['list_opt','memcap'], **dr)
    C(f'Q09-{s}', ident, CAP_ := 'bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness', 0, 0, ['ac_cap'], **dr)
    C(f'Q10-{s}', ident, 'SINGLEFS_HEAVY_TESTS=commit cargo test --all', 0 if ident == M else 2, 0, ['prefix_need','sub_refuse'], **dr)
