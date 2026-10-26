import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
CAP = cases_a.CAP; SC = cases_a.SC
RB = cases_a.RB
# ---------- 文件头写明要拒的各类，造最常见的写法 ----------
# 一、各类重型：子 agent 一律拒（带 / 不带前缀、经不经包装），主 agent 不带前缀拒
classes = [
 ('D01', 'cargo test --all', ['full','sub_refuse']),
 ('D02', 'cargo test --workspace', ['full','sub_refuse']),
 ('D03', 'cargo test -p singlefs-checker-tier', ['checker','sub_refuse']),
 ('D04', 'cd crates/singlefs-checker-tier && cargo test', ['checker','sub_refuse']),
 ('D05', 'cargo test -p singlefs-checker-tier --test crash_enumeration_alpha', ['checker','sub_refuse']),
 ('D06', 'cargo test -p singlefs-background-gpu --features tests-on-a-real-gpu', ['gpu','sub_refuse']),
 ('D07', 'bash .claude/gate.d/checker-tier-qemu-device-streams.sh', ['qemu','sub_refuse']),
 ('D08', 'qemu-system-x86_64 -m 512 -nographic', ['qemu','sub_refuse']),
 ('D09', 'bash scripts/devices/vm-bench.sh', ['qemu','sub_refuse']),
 ('D10', 'bash .claude/gate.d/checker-tier-lkmm.sh', ['herd7','sub_refuse']),
 ('D11', 'herd7 foo.litmus', ['herd7','sub_refuse']),
 ('D12', 'bash .claude/scripts/lkmm.sh', ['herd7','sub_refuse']),
 ('D13', 'bash scripts/mutation/mutate.sh crates/mutations.tsv', ['mut','sub_refuse']),
 ('D14', 'bash .claude/gate.d/checker-tier-crates-mutation-replay.sh', ['mut','sub_refuse']),
 ('D15', 'bash scripts/experiment-apparatus/replay.sh', ['replay','sub_refuse']),
 ('D16', 'bash scripts/experiment-apparatus/replay.sh E100', ['replay','sub_refuse']),
 ('D17', 'bash scripts/experiment-apparatus/replay.sh --admission-only', ['replay','sub_refuse']),
 ('D18', 'bash scripts/gate/gate-staged.sh', ['gate','sub_refuse']),
 ('D19', 'bash .claude/scripts/check.sh', ['full','sub_refuse']),
 ('D20', 'bash scripts/devices/e152-run.sh', ['e152','sub_refuse']),
 ('D21', 'python3 scripts/mutation/broad-mutation.py', ['broad','sub_refuse']),
 ('D22', 'python3 scripts/mutation/broad-mutation.py --full', ['broad','sub_refuse']),
 ('D23', 'bash .claude/gate.d/checker-tier-formal-verification.sh', ['verus','sub_refuse']),
 ('D24', 'bash scripts/crash/crash-amplification.sh', ['amp','sub_refuse']),
 ('D25', 'bash scripts/crash/crash-amplification.sh --item foo', ['amp','sub_refuse']),
 ('D26', 'bash scripts/crash/crash-point-replay.sh', ['cpr','sub_refuse']),
 ('D27', 'bash scripts/crash/crash-point-replay.sh --full', ['cpr','sub_refuse']),
 ('D28', 'bash scripts/crash/crash-point-replay.sh --item foo', ['cpr','sub_refuse']),
 ('D29', 'bash scripts/devices/build-patched-qemu.sh', ['qemubuild','sub_refuse']),
 ('D30', 'bash .claude/gate.d/checker-tier-research-build-and-replay.sh', ['gate_list','sub_refuse']),
 ('D31', 'bash .claude/gate.d/doc-text.sh', ['commit_checks','sub_refuse']),
 ('D32', 'bash .claude/singlefs-ai-sop/scripts/doc-lint.sh', ['commit_checks','sub_refuse']),
 ('D33', 'bash .claude/singlefs-ai-sop/scripts/gate-lint.sh', ['commit_checks','sub_refuse']),
 ('D34', 'python3 scripts/gate/verus-proof-check.py proof', ['verus','sub_refuse']),
 ('D35', 'cargo test -p singlefs-harness --workspace', ['full','sub_refuse']),
]
for n, cmd, cites in classes:
    # 子 agent 不带前缀
    C(f'{n}-IW', IW, cmd, 2, 0, cites)
    # 子 agent 带前缀仍拒
    C(f'{n}p-IW', IW, 'SINGLEFS_HEAVY_TESTS=commit ' + cmd, 2, 0, cites + ['prefix_need'])
    # 子 agent 经内存包装 + capped 仍拒
    C(f'{n}w-ER', ER, 'bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G ' + cmd, 2, 0, cites)
    # 主 agent 不带前缀拒（cargo 的经 run-build-directory 包，隔开编译目录那一道）
    mcmd = (cases_a.RB + cmd) if cmd.startswith(('cargo','cd ')) else cmd
    if cmd.startswith('cd '):
        mcmd = cmd.replace('cargo test', cases_a.RB + 'cargo test')
    C(f'{n}-M', 'main', mcmd, 2, 0, cites + ['prefix_need','main_all'])
    # 主 agent 带前缀放行
    C(f'{n}p-M', 'main', 'SINGLEFS_HEAVY_TESTS=commit ' + mcmd if not cmd.startswith('cd ') else mcmd.replace('bash scripts/hosts/run-build-directory.sh', 'SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-build-directory.sh'), 0 if n not in ('D06',) else 2, 0, cites + ['prefix_need','main_all'] + (['gpu_quiesce'] if n=='D06' else []))
# 二、别的值的前缀
for v in ['1','true','']:
    C(f'D40-{v or "empty"}', 'main', f'SINGLEFS_HEAVY_TESTS={v} ' + cases_a.RB + 'cargo test --all', 2, 0, ['prefix_need'])
C('D41', 'main', 'SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/capped.sh 31 ' + cases_a.RB + 'cargo test --all', 0, 0, ['prefix_wrap'])
C('D42', 'main', 'SINGLEFS_HEAVY_TESTS=Commit ' + cases_a.RB + 'cargo test --all', 2, 0, ['prefix_need'])
C('D43', 'main', 'export SINGLEFS_HEAVY_TESTS=commit\n' + cases_a.RB + 'cargo test --all', 0, 0, ['prefix_forms'])
C('D44', 'main', cases_a.RB + 'cargo test --all\nexport SINGLEFS_HEAVY_TESTS=commit', 2, 0, ['prefix_forms'], note='export 写在后面，前面那条没带')
C('D45', 'main', 'SINGLEFS_HEAVY_TESTS=commit true; ' + cases_a.RB + 'cargo test --all', 2, 0, ['prefix_forms'], note='前缀只属于 true 那一条')
# 三、经各种包装/间接起法仍拒（子 agent，不带前缀，cwd 副本）
wraps = [
 ('E01', 'nice -n 19 cargo test --all', ['cmdpos','sub_refuse']),
 ('E02', 'timeout 600 cargo test --all', ['prefix_wrap','sub_refuse']),
 ('E03', 'env FOO=1 cargo test --all', ['prefix_forms','sub_refuse']),
 ('E04', 'time cargo test --all', ['sub_refuse']),
 ('E05', '/usr/bin/time -v cargo test --all', ['launchers','sub_refuse']),
 ('E06', "bash -c 'cargo test --all'", ['prefix_wrap','sub_refuse']),
 ('E07', 'bash <<\'EOF\'\ncargo test --all\nEOF', ['stdin_shell','sub_refuse']),
 ('E08', "bash <<< 'cargo test --all'", ['stdin_shell','sub_refuse']),
 ('E09', "printf 'cargo test --workspace\\n' | bash", ['stdin_shell','sub_refuse']),
 ('E10', "echo 'cargo test --workspace' | sh", ['stdin_shell','sub_refuse']),
 ('E11', 'bash fx/heavy-all.sh', ['script_read','sub_refuse']),
 ('E12', './fx/heavy-all.sh', ['script_read','sub_refuse']),
 ('E13', 'source fx/heavy-all.sh', ['script_read','sub_refuse']),
 ('E14', '. fx/heavy-all.sh', ['script_read','sub_refuse']),
 ('E15', "cat > {D}/h.sh <<'EOF'\ncargo test --all\nEOF\nbash {D}/h.sh", ['written','sub_refuse']),
 ('E16', '( cargo test --all )', ['sub_refuse']),
 ('E17', '{ cargo test --all; }', ['sub_refuse']),
 ('E18', 'cd crates && cargo test --workspace', ['full','sub_refuse']),
 ('E19', 'cargo test --all 2>&1 | tail -5', ['full','sub_refuse']),
 ('E20', 'true && cargo test --workspace; echo rc=$?', ['full','sub_refuse']),
 ('E21', 'X=1; cargo test --all', ['full','sub_refuse']),
 ('E22', 'echo $(cargo test --all)', ['cmdpos','sub_refuse']),
 ('E23', 'bash scripts/hosts/capped.sh 31 cargo test --all', ['prefix_wrap','sub_refuse']),
 ('E24', 'cargo test --all -- --list', ['list_opt','full']),
 ('E25', 'cargo t --workspace', ['checker','full']),
 ('E26', 'cargo nextest run --workspace', ['checker','full']),
 ('E27', 'for i in 1 2; do cargo test --all; done', ['sub_refuse']),
 ('E28', 'mutate.{sh,x} crates/mutations.tsv', ['brace']),
 ('E29', 'bash scripts/mutation/mutat?.sh crates/mutations.tsv', ['glob']),
 ('E30', 'bash .claude/gate.d/checker-tier-crash-point-replay.{sh,}', ['brace']),
 ('E31', 'cargo test --all # comment', ['full']),
 ('E32', 'cargo   test\t--workspace', ['full']),
 ('E33', 'cargo "test" --all', ['full']),
 ('E34', "'cargo' test --all", ['full']),
 ('E35', 'env -S "cargo test --all"', ['env_opts','sub_refuse']),
 ('E36', 'env -C crates cargo test --workspace', ['env_opts','full']),
 ('E37', 'flock /tmp/l.lock cargo test --all', ['launchers']),
 ('E38', 'systemd-run --user --scope cargo test --all', ['launchers']),
]
for n, cmd, cites in wraps:
    C(f'{n}-IW', IW, cmd, 2, 0, cites)
    if n in ('E01','E06','E11','E16','E23'):
        C(f'{n}-M', 'main', cmd.replace('cargo test', cases_a.RB + 'cargo test') if n not in ('E11','E12') else cmd, 2, 0, cites + ['prefix_need'])
C('E50-M', 'main', 'SINGLEFS_HEAVY_TESTS=commit bash fx/heavy-all.sh', 0, 0, ['script_read','prefix_wrap'])
C('E51-M', 'main', 'bash fx/heavy-all.sh', 2, 0, ['script_read','prefix_need'])
C('E52-M', 'main', 'SINGLEFS_HEAVY_TESTS=commit bash fx/uses-commit.sh', 0, 0, ['script_read','commit_checks'])
C('E53-IW', IW, 'bash fx/uses-commit.sh', 2, 0, ['script_read','commit_checks'])
