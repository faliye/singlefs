#!/usr/bin/env bash
# research/scripts/layer0-shard-run.sh --selftest 的本体：不碰第二台。拷一份仓进临时目录、git init，登记一条 shard=across-machines 的用例，
# 「第二台」是本机上的另一个目录（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1），本机两个进程各跑一片，走驱动脚本同一条路：
# 工具链与输入指纹两边比、清场与复原（回读）、两片、拷账本、merge、判、写 54 号那一格全绿标记。
# 默认用假 cargo（不编译、不跑用例）：登记的是临时仓里新写的一条标了 #[ignore] 的替身用例，三趟都不带 --include-ignored，不算重型；
# 假 cargo 分片跑时往进度目录写账本、merge 时核两份账本在不在、记的输入指纹对不对，打与层 0 同形的计数行与带 shards= 的线程行。
# 带 SINGLEFS_HEAVY_TESTS（commit 或 user-request）时换真 cargo：登记 crates/singlefs-harness/tests/crash_enumeration_sharded_across_processes.rs
# 里不标 ignore 的小流用例（第一条流按甲二展开 29 个状态），release 下真编真跑；不带时成功行写明真 cargo 那一趟本次未跑。逐格核：
#   ① 不分片跑那条用例打的计数行，与双机分片 merge 之后记进全绿标记的那一行逐字相同；标记作数；清场复原了；第二台这一趟的目录删了
#   ② --merged-log（门禁 54 号调的那一条）：merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数
#   ③ 「第二台」的 rustc -Vv 第一行不同 ⇒ 拒（工具链不同），标记不动
#   ④ 「第二台」那一片没写账本 ⇒ 判红，删这批输入那一格标记
#   ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒（退 1），出路指到 layer0-shard.env.example
#   ⑥ 没登记 shard=across-machines 的用例 ⇒ 拒
# 成功行报核了几格（现算）。
#
# admission: always 自证判的是这一刻的驱动脚本与仓，每次调都要现跑
# run-condition: command cargo rustc python3 rsync git nproc
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
selftest_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

repository_root="$(cd "$selftest_script_directory/../.." && pwd)"
work="$(mktemp -d)"
remove_work() {
  echo "  · 自证的临时目录 $work（$(du -sh "$work" 2>/dev/null | cut -f1)）删掉"
  rm -rf -- "${work:?}"
}
trap remove_work EXIT
checked=0
failures=0
expect() { # expect <名> <判据的退出码：0 为过> <不过时的细节>
  checked=$((checked + 1))
  if [[ "$2" == 0 ]]; then
    echo "  ✓ $1"
  else
    echo "  ✗ $1：$3"  # gate-lint:detail
    failures=$((failures + 1))
  fi
}

selftest_case_key="crash-case:sharded-selftest"
copy="$work/repository"
rsync -a --exclude target --exclude .git --exclude layer0-shard.env "$repository_root/" "$copy/"
git -C "$copy" init -q
real_rustc="$(command -v rustc)"
if [[ -n "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  cargo_mode="真 cargo（SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS}）"
  selftest_function="the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts"
  selftest_target="crash_enumeration_sharded_across_processes"
  selftest_cargo="$(command -v cargo)"
else
  cargo_mode="假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）"
  selftest_function="stand_in_for_the_driver_selftest"
  selftest_target="sharded_driver_selftest_stand_in"
  printf '#[test]\n#[ignore]\nfn %s() {}\n' "$selftest_function" > "$copy/crates/singlefs-harness/tests/$selftest_target.rs"
  mkdir -p "$work/stand-in-bin"
  selftest_cargo="$work/stand-in-bin/cargo"
  cat > "$selftest_cargo" <<'STAND_IN_CARGO'
#!/usr/bin/env bash
# 驱动脚本自证的假 cargo：-V 打版本；test 按 SINGLEFS_LAYER0_SHARD 分三种：不设打计数行与线程行，<i>/2 往进度目录写那一片的账本，
# merge/2 核两份账本在、记的输入指纹与这一趟的相同、START_OVER 没设，再打计数行与带 shards= 的线程行。别的一律退 101。
if [[ "${1:-}" == -V ]]; then echo "cargo 0.0.0-layer0-shard-selftest"; exit 0; fi
[[ "${1:-}" == test ]] || { echo "假 cargo 只认 -V 与 test：$*"; exit 101; }
count_line="LAYER0_SHARDED states=29 closed_form=29 violations=0 exhaustive=true"
passed="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s"
case "${SINGLEFS_LAYER0_SHARD-}" in
  "")
    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=2 configured_worker_threads=2 worker_threads_source=environment_variable resumed_slices=0 freshly_run_slices=4 progress_file_after_completion=none elapsed_seconds=0.1"
    printf '%s\n%s\n' "$count_line" "$passed" ;;
  0/2|1/2)
    [[ -n "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}" && -n "${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "分片跑没设进度目录或输入指纹"; exit 101; }
    shard_index="${SINGLEFS_LAYER0_SHARD%/2}"
    ledger="$SINGLEFS_LAYER0_PROGRESS_DIRECTORY/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
    mkdir -p "$SINGLEFS_LAYER0_PROGRESS_DIRECTORY" && printf 'input_fingerprint=%s\n' "$SINGLEFS_LAYER0_INPUT_FINGERPRINT" > "$ledger" || exit 101
    printf 'LAYER0_PROGRESS slice=1/2 shard=%s\nLAYER0_SHARD mode=run shard=%s ledger=%s\n%s\n' "$SINGLEFS_LAYER0_SHARD" "$SINGLEFS_LAYER0_SHARD" "$ledger" "$passed" ;;
  merge/2)
    [[ -z "${SINGLEFS_LAYER0_START_OVER+set}" ]] || { echo "merge 那一趟带了 SINGLEFS_LAYER0_START_OVER"; exit 101; }
    for shard_index in 0 1; do
      ledger="${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
      [[ "$(cat "$ledger" 2>/dev/null)" == "input_fingerprint=${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "merge：第 $shard_index 片的账本缺或输入指纹不同（$ledger）"; exit 101; }
    done
    echo "LAYER0_SHARD mode=merge shards=2 ledgers=${SINGLEFS_LAYER0_PROGRESS_DIRECTORY}"
    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=4 configured_worker_threads=4 worker_threads_source=shard_ledgers resumed_slices=0 freshly_run_slices=4 shards=2 shard_worker_threads=2,2 shard_configured_worker_threads=2,2 shard_worker_threads_sources=environment_variable,environment_variable shard_available_parallelism=2,2 shard_resumed_slices=0,0 shard_freshly_run_slices=2,2 shard_elapsed_milliseconds=10,10 progress_file_after_completion=kept elapsed_seconds=0.1"
    printf '%s\n%s\n' "$count_line" "$passed" ;;
  *) echo "假 cargo 认不出 SINGLEFS_LAYER0_SHARD=${SINGLEFS_LAYER0_SHARD}"; exit 101 ;;
esac
STAND_IN_CARGO
  chmod +x "$selftest_cargo"
  export PATH="$work/stand-in-bin:$PATH"
fi
printf '%s\t%s\t%s\t%s\n' "$selftest_case_key" "crates/ Cargo.toml Cargo.lock" \
  "test=singlefs-harness:$selftest_target:$selftest_function count-line=LAYER0_SHARDED exhaustive=LAYER0_SHARDED threads=LAYER0_SHARDED shard=across-machines" \
  "# 驱动脚本自证临时登记的用例" >> "$copy/.claude/gate.d/stage-inputs.tsv"
selftest_bin_directory="$(dirname "$selftest_cargo")"
write_configuration() { # write_configuration <文件> <PEER_CARGO_BIN_DIRECTORY>
  cat > "$1" <<CONFIGURATION
# 驱动脚本自证用的配置：第二台是本机上的 $work/peer
PEER_SSH_HOST=selftest-peer-is-this-machine
PEER_REPOSITORY_DIRECTORY=$work/peer
PEER_CARGO_BIN_DIRECTORY=$2
QUIESCE_STOP_COMMAND=touch $work/quiesced
QUIESCE_STOPPED_CHECK_COMMAND=test -e $work/quiesced
QUIESCE_START_COMMAND=rm -f $work/quiesced
QUIESCE_STARTED_CHECK_COMMAND=test ! -e $work/quiesced
CONFIGURATION
}
write_configuration "$work/layer0-shard.env" "$selftest_bin_directory"
export SINGLEFS_LAYER0_SHARD_CONFIG="$work/layer0-shard.env"
export SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1
# 三趟 cargo test 都不带 --include-ignored：真 cargo 那条小流用例不标 ignore（同一个测试目标里标了 ignore 的 golden 子进程用例不进过滤范围）；
# 假 cargo 的替身用例标了 ignore，不带它就不算重型
export SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0
unset SINGLEFS_LAYER0_SHARD SINGLEFS_LAYER0_PROGRESS_DIRECTORY SINGLEFS_LAYER0_INPUT_FINGERPRINT SINGLEFS_LAYER0_START_OVER
driver="$copy/research/scripts/layer0-shard-run.sh"
admission_module="$copy/research/scripts/admission.py"

# 不分片跑一次那条用例：计数行是 merge 之后要逐字相同的那一行
(cd "$copy" && cargo test --release -p singlefs-harness --test "$selftest_target" -- --exact "$selftest_function" --nocapture) > "$work/unsharded.log" 2>&1
unsharded_exit=$?
unsharded_count_line="$(grep '^LAYER0_SHARDED ' "$work/unsharded.log")"
expect "不分片跑登记的那条用例：退 0，恰好一行 LAYER0_SHARDED 计数行" "$([[ $unsharded_exit == 0 && $(grep -c '^LAYER0_SHARDED ' "$work/unsharded.log") == 1 ]]; echo $?)" \
  "退 $unsharded_exit；日志尾部：$(tail -5 "$work/unsharded.log" | tr '\n' '|')"

# ① 单独跑：两片、merge、判、写标记
bash "$driver" "$selftest_case_key" "$copy" > "$work/standalone.log" 2>&1
standalone_exit=$?
fingerprint="$(python3 "$admission_module" crash-case-manifest "$copy" "$selftest_case_key" "$work/manifest" \
  --judging-digest --toolchain --build-environment | cut -d' ' -f1)"
marker_output="$(python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest")"
marker_exit=$?
marker_count_line="$(grep '^LAYER0_SHARDED ' <<< "$marker_output")"
marker_path="$(head -1 <<< "$marker_output" | cut -d' ' -f2)"
expect "① 单独跑：退 0，两片各跑了（LAYER0_SHARD mode=run 0/2、1/2），这批输入那一格全绿标记作数" \
  "$([[ $standalone_exit == 0 && $marker_exit == 0 ]] && grep -q 'SINGLEFS_LAYER0_SHARD=0/2' "$work/standalone.log" && grep -q '④ 两片都跑完' "$work/standalone.log"; echo $?)" \
  "驱动脚本退 $standalone_exit，标记核 $marker_exit（$marker_output）；驱动脚本输出尾部：$(tail -15 "$work/standalone.log" | tr '\n' '|')"
expect "① 不分片的计数行与双机分片 merge 之后记进标记的那一行逐字相同" "$([[ -n "$unsharded_count_line" && "$marker_count_line" == "$unsharded_count_line" ]]; echo $?)" \
  "不分片「$unsharded_count_line」，标记里「$marker_count_line」"
expect "① merge 那一行按片报线程（shards=2、shard_worker_threads=），记进了标记" "$(grep -q '^parallel_finished=LAYER0_PARALLEL_FINISHED .*shards=2 shard_worker_threads=' "$marker_path"; echo $?)" \
  "标记 $marker_path 里：$(grep 'parallel_finished=' "$marker_path" 2>/dev/null)"
expect "① 清了场、跑完复原了（回读过），第二台这一趟的目录删了" \
  "$([[ ! -e "$work/quiesced" ]] && grep -q '清场之后回读过了' "$work/standalone.log" && grep -q '复原之后回读过了' "$work/standalone.log" \
     && [[ -z "$(ls -A "$work/peer/runs" 2>/dev/null)" ]]; echo $?)" \
  "清场标记还在：$([[ -e "$work/quiesced" ]] && echo 是 || echo 否)；第二台 runs/ 下：$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')"

# ② --merged-log：门禁 54 号调的那一条
bash "$driver" --merged-log "$selftest_case_key" "$copy" "$fingerprint" "$work/merged.log" > "$work/merged-driver.log" 2>&1
merged_exit=$?
expect "② --merged-log：退 0，merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数" \
  "$([[ $merged_exit == 0 ]] && grep -q 'LAYER0_SHARD mode=merge shards=2 ' "$work/merged.log" && [[ "$(grep '^LAYER0_SHARDED ' "$work/merged.log")" == "$unsharded_count_line" ]]; echo $?)" \
  "退 $merged_exit；驱动脚本输出尾部：$(tail -10 "$work/merged-driver.log" | tr '\n' '|')"

# ③ 「第二台」的 rustc -Vv 第一行不同
mkdir -p "$work/other-toolchain"
printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$selftest_cargo" > "$work/other-toolchain/cargo"
printf '#!/usr/bin/env bash\nif [[ "${1:-}" == -Vv ]]; then %q -Vv | sed "1s/\$/ (another build)/"; else exec %q "$@"; fi\n' "$real_rustc" "$real_rustc" > "$work/other-toolchain/rustc"
chmod +x "$work/other-toolchain/cargo" "$work/other-toolchain/rustc"
write_configuration "$work/other-toolchain.env" "$work/other-toolchain"
SINGLEFS_LAYER0_SHARD_CONFIG="$work/other-toolchain.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/other-toolchain.log" 2>&1
toolchain_exit=$?
python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
toolchain_marker_exit=$?
expect "③ 「第二台」rustc -Vv 第一行不同 ⇒ 拒（退 1，说工具链不同），这批输入那一格标记不动" \
  "$([[ $toolchain_exit == 1 && $toolchain_marker_exit == 0 ]] && grep -q '两台的工具链不同' "$work/other-toolchain.log"; echo $?)" \
  "退 $toolchain_exit，标记核 $toolchain_marker_exit；输出：$(tail -4 "$work/other-toolchain.log" | tr '\n' '|')"

# ④ 「第二台」那一片退 0 却没写账本
mkdir -p "$work/no-ledger"
printf '#!/usr/bin/env bash\nif [[ "${1:-}" == test ]]; then echo "LAYER0_SHARD mode=run shard=1/2 （自证：这一片不写账本）"; exit 0; fi\nexec %q "$@"\n' "$selftest_cargo" > "$work/no-ledger/cargo"
printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$real_rustc" > "$work/no-ledger/rustc"
chmod +x "$work/no-ledger/cargo" "$work/no-ledger/rustc"
write_configuration "$work/no-ledger.env" "$work/no-ledger"
git_common_directory="$(git -C "$copy" rev-parse --path-format=absolute --git-common-dir)"
rm -f -- "${git_common_directory:?}/singlefs-layer0-progress/$fingerprint"/layer0-shard-*.tally "${work:?}/peer/progress/$fingerprint"/layer0-shard-*.tally
SINGLEFS_LAYER0_SHARD_CONFIG="$work/no-ledger.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-ledger.log" 2>&1
no_ledger_exit=$?
python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
no_ledger_marker_exit=$?
expect "④ 「第二台」那一片没写账本 ⇒ 判红（退 1，说两片的账本不是各恰好一份），删这批输入那一格标记" \
  "$([[ $no_ledger_exit == 1 && $no_ledger_marker_exit == 1 ]] && grep -q '两片的账本不是各恰好一份' "$work/no-ledger.log"; echo $?)" \
  "退 $no_ledger_exit，标记核 $no_ledger_marker_exit；输出：$(tail -4 "$work/no-ledger.log" | tr '\n' '|')"

# ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒
grep -v '^PEER_CARGO_BIN_DIRECTORY=' "$work/layer0-shard.env" > "$work/missing-key.env"
SINGLEFS_LAYER0_SHARD_CONFIG="$work/missing-key.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/missing-key.log" 2>&1
missing_key_exit=$?
SINGLEFS_LAYER0_SHARD_CONFIG="$work/no-such-configuration.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-configuration.log" 2>&1
no_configuration_exit=$?
expect "⑤ 配置缺键 PEER_CARGO_BIN_DIRECTORY、配置文件不在 ⇒ 配置判法各拒（退 1，说双机分片不能用），出路指到 layer0-shard.env.example" \
  "$([[ $missing_key_exit == 1 && $no_configuration_exit == 1 ]] && grep -q '双机分片不能用' "$work/missing-key.log" && grep -q '双机分片不能用' "$work/no-configuration.log" \
     && grep -q 'layer0-shard.env.example' "$work/missing-key.log" && grep -q 'layer0-shard.env.example' "$work/no-configuration.log"; echo $?)" \
  "缺键退 $missing_key_exit（$(tail -3 "$work/missing-key.log" | tr '\n' '|')），不在退 $no_configuration_exit（$(tail -3 "$work/no-configuration.log" | tr '\n' '|')）"

# ⑥ 没登记 shard=across-machines 的用例
bash "$driver" crash-case:c561-sigma-full "$copy" > "$work/not-shardable.log" 2>&1
not_shardable_exit=$?
expect "⑥ 没登记 shard=across-machines 的用例 ⇒ 拒（退 1），出路是单机跑" \
  "$([[ $not_shardable_exit == 1 ]] && grep -q '没登记 shard=across-machines' "$work/not-shardable.log"; echo $?)" \
  "退 $not_shardable_exit；输出：$(tail -3 "$work/not-shardable.log" | tr '\n' '|')"

if (( failures > 0 )); then
  echo "  ✗ layer0-shard-run.sh 自证没过：${failures} 格判错（共 ${checked} 格）"
  echo "     → 怎么办：照上面每一格的说明改驱动脚本（research/scripts/layer0-shard-run.sh）或分片那一段（crates/singlefs-harness/src/layer0_progress.rs、crash.rs）"
  exit 1
fi
echo "  ✓ layer0-shard-run.sh 自证通过：${checked} 格都对（${cargo_mode}；第二台是本机上的另一个目录，没碰真的第二台）"
