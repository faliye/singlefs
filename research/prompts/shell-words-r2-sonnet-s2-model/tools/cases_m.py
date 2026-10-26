import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
KB = 'kb-scribe'
M = 'main'
R = dict(cwd='repo')
# ---------- 主 agent / 共用约束里照抄的起法 ----------
C('P01-M', M, 'bash scripts/crash/crash-point-replay.sh --list-items --with-keys', 2, 0, ['cpr','cpr_list','ma_prefix'], **R, note='不是「单独的 --list-items」')
C('P02-M', M, 'SINGLEFS_HEAVY_TESTS=commit bash scripts/crash/crash-point-replay.sh --list-items --with-keys', 0, 0, ['cpr','prefix_need'], **R)
C('P03-M', M, 'python3 scripts/agents/agent-cost.py usage --since 2026-10-01', 0, 0, ['cmdpos'], **R)
C('P04-M', M, 'python3 scripts/agents/agent-cost.py outcome --since 2026-10-01 --until 2026-10-26 --summary --out {D}/outcome.txt', 0, 0, ['cmdpos'], **R)
for ident in (M, KB, IW):
    C(f'P05-{ident[:3]}', ident, "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=写回' bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync", 0, 0, ['gate_list'], **R)
    C(f'P06-{ident[:3]}', ident, "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; 范围=4 格; 为什么=交回前核自己改动里的命名' bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names .", 0, 0, ['commit_selfcheck'], **R)
    C(f'P07-{ident[:3]}', ident, "bash .claude/singlefs-ai-sop/scripts/naming-lint.sh .", 0, 0, ['commit_selfcheck'], **R)
C('P08-IW', IW, "bash .claude/gate.d/code-source-discipline.sh --check vague-names,doc-text .", 2, 0, ['commit_selfcheck'], **R, note='点名里有别的格，照旧拒')
for ident in (M, IW, INV):
    s = ident[:3]
    C(f'P10-{s}', ident, 'python3 scripts/gate/rerun-failed-tests.py {D}/test.log -p singlefs-harness', 0, 0, ['ac_cap_order'], **R)
    C(f'P11-{s}', ident, 'python3 -m py_compile scripts/hosts/compile-then-swap.py; bash -n scripts/hosts/capped.sh', 0, 0, ['n_syntax'], **R)
    C(f'P12-{s}', ident, 'python3 scripts/kb/stale-candidates.py --check-facts', 0, 0, ['cmdpos'], **R)
    C(f'P13-{s}', ident, 'python3 scripts/kb/batch-target.py', 0, 0, ['cmdpos'], **R)
    C(f'P14-{s}', ident, 'python3 scripts/agents/three-way-materials.py r1', 0, 0, ['cmdpos'], **R)
    C(f'P15-{s}', ident, 'git log --all --diff-filter=D --name-only -- "*shell-words*"; git show HEAD^:README.md | head -2', 0, 0, ['cmdpos'], **R)
    C(f'P16-{s}', ident, 'python3 .claude/singlefs-ai-sop/scripts/proc.py wait 999999 --timeout 5', 0, 0, ['ac_proc'], **R)
    C(f'P17-{s}', ident, 'python3 scripts/agents/copy-originals.py copy {D} README.md', 0, 0, ['b_scope_unknown'], **R, wr=True)
C('P20-M', M, 'python3 scripts/gate/stage-mine.py', 0, 0, ['cmdpos'], **R)
C('P21-M', M, 'bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts', 0, 0, ['cmdpos'], **R)
C('P22-M', M, 'bash scripts/hosts/detached-run.sh long-run-check -- bash fx/light.sh', 0, 0, ['ac_detached'], **R)
C('P23-M', M, "SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=门禁 checker-tier-crash-point-replay.sh 的全量（--full）; 范围=整轮; 为什么=用户要求' SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/checker-tier-crash-point-replay.sh --full .", 0, 0, ['prefix_need'], **R)
C('P24-M', M, 'SINGLEFS_HEAVY_TESTS=commit ' + cases_a.RB + 'cargo test --release -p singlefs-checker-tier --lib --tests', 0, 0, ['prefix_need','ac_build'], **R)
C('P25-M', M, 'SINGLEFS_HEAVY_TESTS=commit bash scripts/crash/gpu-quiesce-run.sh -- ' + cases_a.RB + 'cargo test -p singlefs-background-gpu --features tests-on-a-real-gpu -- --test-threads 1', 0, 0, ['gpu','gpu_quiesce'], **R)
C('P26-M', M, 'bash scripts/hosts/multi-host-run.sh --peer --help', 0, 0, ['ac_ps'], **R)
C('P27-IW', IW, "bash scripts/hosts/run-with-memory-cap.sh 4G bash {D}/own-script.sh", 0, 0, ['ac_cap'], **R, note='脚本还不在：.sh 按「不存在」放行并记检出')
C('P28-IW', IW, "cat > {D}/own-script.sh <<'EOF'\ncargo build -p singlefs-core\nEOF\nbash scripts/hosts/run-with-memory-cap.sh 4G bash {D}/own-script.sh", 0, 0, ['ac_cap','written'], **R)
C('P29-IW', IW, "cat > {D}/own-test.sh <<'EOF'\ncargo test -p singlefs-harness\nEOF\nbash scripts/hosts/run-with-memory-cap.sh 4G bash {D}/own-test.sh", 0, 0, ['ac_cap','written'], **R, note='整条脚本经内存包装跑：脚本里的 cargo test 算在包装里')
C('P30-IW', IW, "cat > {D}/own-test2.sh <<'EOF'\ncargo test -p singlefs-harness\nEOF\nbash {D}/own-test2.sh", 2, 0, ['ac_cap','memcap'], **R, note='脚本里的 cargo test 没经内存包装')
