import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
R = dict(cwd='repo')
# ---------- 自证（--selftest）按登记表判 ----------
for ident in (M, IW, TF):
    s = ident[:3]
    C(f'T01-{s}', ident, 'python3 scripts/agents/agent-handover.py --selftest', 0, 0, ['selftest','selftest_light'], **R)
    C(f'T02-{s}', ident, 'bash scripts/hosts/run-with-memory-cap.sh 4G python3 scripts/agents/agent-handover.py --selftest', 0, 0, ['selftest_light'], **R)
C('T03-IW', IW, 'bash scripts/devices/dm-owned.sh --selftest', 2, 0, ['selftest','commit_checks'], **R, note='登记表里没有这一行')
C('T03-M', M, 'bash scripts/devices/dm-owned.sh --selftest', 2, 0, ['selftest','commit_checks'], **R)
C('T04-IW', IW, 'bash scripts/devices/vm-bench.sh --selftest', 2, 0, ['qemu','selftest'], **R)
C('T04-M', M, 'bash scripts/devices/vm-bench.sh --selftest', 2, 0, ['qemu','selftest'], **R)
C('T05-IW', IW, 'python3 scripts/agents/agent-handover.py --selftest --verbose', 2, 0, ['selftest_light','selftest'], **R, note='多带了别的参数，表外')
