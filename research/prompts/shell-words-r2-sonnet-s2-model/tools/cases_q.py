import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
M = 'main'
MC = 'bash scripts/hosts/run-with-memory-cap.sh 4G '
# ---------- 直接执行编出来的二进制、E152、通配 ----------
HASH = '0123456789abcdef'
C('R01-IW', IW, MC + f'target/debug/deps/crash_full_run_backlog-{HASH} --nocapture', 2, 0, ['checker','sub_refuse'])
C('R01-M', M, MC + f'target/debug/deps/crash_full_run_backlog-{HASH} --nocapture', 2, 0, ['checker','prefix_need'])
C('R01p-M', M, 'SINGLEFS_HEAVY_TESTS=commit ' + MC + f'target/debug/deps/crash_full_run_backlog-{HASH} --nocapture', 0, 0, ['checker','prefix_need'])
C('R02-IW', IW, f'target/debug/deps/crash_full_run_backlog-{HASH}', 2, 0, ['checker','memcap'], note='既是 checker 档，又没经内存包装')
C('R03-ER', ER, MC + 'target/release/e152-file-system-benchmark --help', 2, 0, ['e152','sub_refuse'])
C('R03-M', M, 'target/release/e152-file-system-benchmark --help', 2, 0, ['e152','prefix_need'])
C('R04-M', M, 'bash scripts/devices/e152-run.sh', 2, 0, ['e152','prefix_need'])
C('R04p-M', M, 'SINGLEFS_HEAVY_TESTS=user-request bash scripts/devices/e152-run.sh', 0, 0, ['e152','prefix_need'])
C('R05-IW', IW, 'bash scripts/mutation/mutate.[[:lower:]]h crates/mutations.tsv', 2, 0, ['glob','sub_refuse'])
C('R06-IW', IW, './scripts/mutation/mutat?.sh crates/mutations.tsv', 2, 0, ['glob','sub_refuse'])
C('R07-IW', IW, 'bash .claude/gate.d/checker-tier-crash-point-repl*.sh', 2, 0, ['glob','sub_refuse'])
C('R08-IW', IW, 'cargo run --release --bin e152-file-system-benchmark', 2, 0, ['e152','sub_refuse'])
C('R09-IW', IW, MC + 'cargo run --release --bin e168_example', 0, 0, ['ac_cap'])
C('R10-M', M, 'SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 4G bash -c "bash scripts/gate/gate-staged.sh"', 0, 0, ['prefix_wrap'])
C('R11-IW', IW, MC + 'bash -c "bash scripts/gate/gate-staged.sh"', 2, 0, ['gate','sub_refuse'])
C('R12-IW', IW, 'bash scripts/gate/gate-staged.sh --help', 2, 0, ['gate','sub_refuse'], note='按名字判，不看参数')
