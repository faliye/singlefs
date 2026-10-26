import cases_a
CASES = []
def C(i, ident, cmd, h, d, cites, **opt):
    CASES.append(dict(id=i, ident=ident, cmd=cmd, h=h, d=d, cites=cites, opt=opt))
IW, ER, TF, CL, INV, TW, GP, TA, MT = cases_a.IW, cases_a.ER, cases_a.TF, cases_a.CL, cases_a.INV, cases_a.TW, cases_a.GP, cases_a.TA, cases_a.MT
CAP = cases_a.CAP; SC = cases_a.SC
RUNNERS = [IW, ER, INV, TW, MT]   # 会编译测试的身份
# ---------- 子 agent 起编译与测试的正当写法（cwd 是仓副本） ----------
specs = [
 ('C01', SC + 'bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test t', ['ac_cap_order','ac_harness']),
 ('C02', CAP + 'cargo test -p singlefs-core --lib', ['ac_cap','ac_capped']),
 ('C03', 'cargo build -p singlefs-core', ['ac_nobuild']),
 ('C04', 'cargo clippy -p singlefs-core -- -D warnings', ['ac_nobuild']),
 ('C05', 'cargo fmt --all', ['ac_nobuild']),
 ('C06', 'cargo test --no-run -p singlefs-harness', ['memcap_what']),
 ('C07', 'cargo check --workspace', ['ac_nobuild','full']),
 ('C08', CAP + 'cargo test -p singlefs-checker --lib', ['ac_cap','checker']),
 ('C09', CAP + 'cargo run --release --bin e168_example -- --threads 31', ['ac_cap']),
 ('C10', 'bash scripts/hosts/run-with-memory-cap.sh 4G target/release/e168_example --threads 31', ['ac_cap']),
 ('C11', 'nice -n 19 ' + CAP + 'cargo test -p singlefs-harness > {D}/t.txt 2>&1; echo rc=$?; tail -n 20 {D}/t.txt', ['ac_rc','ac_cap']),
 ('C12', "SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=崩溃点重放轻型; 范围=1; 为什么=自测' bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/crash/crash-point-replay.sh --quick", ['ac_light3','light_who']),
 ('C13', "SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=形式化轻型; 范围=1; 为什么=自测' bash scripts/hosts/run-with-memory-cap.sh 4G bash .claude/gate.d/checker-tier-formal-verification.sh --quick .", ['ac_light3','light_who']),
 ('C14', "SINGLEFS_RUN_SCOPE='种类=quick; 跑什么=形式化轻型一项; 范围=1; 为什么=自测' bash scripts/hosts/run-with-memory-cap.sh 4G bash .claude/gate.d/checker-tier-formal-verification.sh --quick --item foo", ['light','light_who']),
 ('C15', 'verus --version', ['verus_direct']),
 ('C16', 'python3 scripts/gate/verus-proof-check.py list-items', ['verus_readonly']),
 ('C17', 'python3 scripts/gate/verus-proof-check.py tool-versions', ['verus_readonly']),
 ('C18', 'bash .claude/hooks/heavy-test-guard.sh --selftest', ['selftest','selftest_light']),
 ('C19', 'bash .claude/hooks/bash-command-detector.sh --selftest', ['selftest','selftest_light']),
 ('C20', 'bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness -- --list', ['list_opt','ac_cap']),
 ('C21', 'bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G bash fx/light.sh', ['script_read','ac_cap']),
 ('C22', 'python3 scripts/mutation/broad-mutation.py --list-items', ['broad_light']),
 ('C23', 'bash scripts/crash/crash-point-replay.sh --list-items', ['cpr_list']),
 ('C24', 'bash scripts/experiment-apparatus/replay.sh --list-items', ['replay']),
 ('C25', 'bash .claude/gate.d/checker-tier-crash-point-replay.sh --list-items', ['gate_list']),
 ('C26', 'bash scripts/crash/crash-amplification.sh --list-items', ['amp_list']),
 ('C27', 'cargo test --no-run -p singlefs-checker-tier', ['checker_nobuild']),
 ('C28', 'cargo build -p singlefs-checker-tier --release', ['checker_nobuild']),
 ('C29', "bash -c 'cargo build -p singlefs-core'", ['bash_c','ac_nobuild']),
 ('C30', 'cd crates/singlefs-core && cargo build', ['ac_nobuild']),
 ('C31', 'env CARGO_TARGET_DIR={D}/tgt cargo build -p singlefs-core', ['target_ok']),
 ('C32', 'bash scripts/hosts/run-build-directory.sh cargo build -p singlefs-core', ['target_ok']),
 ('C33', CAP + 'bash fx/light.sh', ['ac_cap','script_read']),
]
REPO_ONLY = {'C12', 'C23', 'C24'}   # 入口脚本按名字、按本仓位置判；别处拷贝照文件头读正文，另列 c 后缀的对照
for n, cmd, cites in specs:
    for ident in RUNNERS:
        if n in REPO_ONLY:
            C(f'{n}-{ident[:3]}', ident, cmd, 0, 0, cites, cwd='repo')
            C(f'{n}c-{ident[:3]}', ident, cmd, 2, 0, cites + ['own_copy'], cwd='copy', note='仓副本里的入口脚本不是本仓那一份，文件头写明正文照读')
        else:
            C(f'{n}-{ident[:3]}', ident, cmd, 0, 0, cites)

# 身份专属
C('C40-TF', TF, 'bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 4G bash {D}/model/tools/rerun.sh --repo {D}/repo --frozen {D}/frozen', 0, 0, ['ac_cap','script_read'])
C('C41-TA', TA, 'cargo build -p singlefs-core --manifest-path {D}/proto/Cargo.toml', 0, 0, ['ac_nobuild'])
C('C42-CL', CL, 'cp {D}/a.md records/b.md', 0, 0, ['b_scope_land'])
C('C43-TW', TW, 'cp {D}/h.sh .claude/hooks/new-hook.sh', 0, 0, ['b_scope_land'])
C('C44-IW', IW, 'cp {D}/x.rs crates/singlefs-core/src/x.rs', 0, 0, ['b_compile_ok','b_scope_land'])
C('C45-INV', INV, 'echo x > crates/singlefs-core/src/y.rs', 0, 0, ['b_scope_land'])
C('C46-ER', ER, 'cp crates/singlefs-core/src/lib.rs {D}/lib.rs.bak', 0, 0, ['b_compile_ok'])
C('C47-ER', ER, 'echo "x" >> crates/mutations.tsv', 0, 0, ['b_compile_ok'])
C('C48-ER', ER, 'echo x > research/results/e999-out.txt', 0, 0, ['b_scope_land'])
C('C49-IW', IW, 'echo x > /tmp/foo.txt', 0, 0, ['b_scope_land'])
C('C50-GP', GP, 'echo x > {D}/o.txt', 0, 0, ['b_scope_builtin'])
C('C51-GP', GP, 'ls 2>/dev/null; echo x > /dev/null', 0, 0, ['b_scope_builtin'])
C('C52-CL', CL, 'ls 2>/dev/null', 0, 0, ['b_scope_land'])
C('C53-TF', TF, 'timeout 60 bash -c "until grep -q ok {D}/log; do sleep 1; done"', 0, 0, ['b_loop_timeout'], timeout=200000)
C('C54-TF', TF, 'sleep 2; echo done', 0, 0, ['b_timeout'], timeout=240000)
C('C55-TF', TF, 'sleep 1', 0, 0, ['ac_timeout','b_timeout'], timeout=300000, note='超 240000 毫秒只记检出，不拒')
C('C56-IW', IW, 'bash scripts/hosts/detached-run.sh list', 0, 0, ['ac_detached'])
C('C57-IW', IW, 'bash scripts/hosts/detached-run.sh watch', 0, 0, ['ac_detached'])
C('C58-TF', TF, 'git reset HEAD -- b.md', 0, 0, ['b_index_ok'], cwd='staged')
C('C59-TF', TF, 'git restore --staged -- b.md', 0, 0, ['b_index_ok'], cwd='staged')
C('C60-IW', IW, 'git reset b.md', 0, 0, ['b_index_ok'], cwd='staged')
C('C61-IW', IW, 'git reset --soft HEAD~1', 0, 0, ['b_index_ok'], cwd='staged')
C('C62-IW', IW, 'git reset', 0, 0, ['b_index_ok'], cwd='empty')
C('C63-IW', IW, 'git restore --staged .', 0, 0, ['b_index_ok'], cwd='empty')
C('C64-IW', IW, 'sleep 1', 0, 0, ['b_bg_sub'])
