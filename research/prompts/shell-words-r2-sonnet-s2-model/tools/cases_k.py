import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
# 定义与共用约束里点名的辅助脚本，按文件头判据应放行的用法
helpers = [
 ('K01', 'python3 scripts/agents/copy-originals.py --help', ['b_scope_unknown']),
 ('K02', 'python3 scripts/hosts/compile-then-swap.py --help', ['b_compile_ok']),
 ('K03', 'python3 scripts/gate/rerun-failed-tests.py --help', ['ac_cap_order']),
 ('K04', 'python3 scripts/mutation/crates-mutation-rows.py plan', ['cmdpos']),
 ('K05', 'bash scripts/hosts/two-logs.sh --quiet mylog -- ls', ['ac_rc']),
 ('K06', 'bash scripts/agents/remind.sh --help', ['cmdpos']),
 ('K07', 'python3 scripts/agents/dispatch-prompt.py --help', ['cmdpos']),
 ('K08', 'python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list', ['commit_checks']),
 ('K09', 'bash scripts/mutation/prove-red.sh --help', ['cmdpos']),
 ('K10', 'bash scripts/hosts/multi-host-run.sh --help', ['ac_ps']),
 ('K11', 'python3 .claude/singlefs-ai-sop/scripts/proc.py stop 999999', ['ac_proc']),
 ('K12', 'bash scripts/hosts/detached-run.sh list', ['ac_detached']),
 ('K13', 'ps -o pid,ppid,args --ppid 1 | head -3', ['ac_proc']),
]
for n, cmd, cites in helpers:
    for ident in (M, IW, ER, TF):
        C(f'{n}-{ident[:3]}', ident, cmd, 0, 0, cites)
