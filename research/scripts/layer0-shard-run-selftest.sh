#!/usr/bin/env bash
# research/scripts/layer0-shard-run.sh --selftest 的本体：不碰第二台。拷一份仓进临时目录、git init，登记一条 shard=across-machines 的用例，
# 「第二台」是本机上的另一个目录（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1），本机两个进程各跑一片，走驱动脚本同一条路：
# 工具链与输入指纹两边比、清场与复原（回读）、两片、拷账本、merge、判、写 54 号那一格全绿标记。
# 默认用假 cargo（不编译、不跑用例）：登记的是临时仓里新写的一条标了 #[ignore] 的替身用例，三趟都不带 --include-ignored，不算重型；
# 假 cargo 分片跑时往进度目录写账本、merge 时核两份账本在不在、记的输入指纹对不对，打与层 0 同形的计数行与带 shards= 的线程行。
# 带 SINGLEFS_HEAVY_TESTS（commit 或 user-request）时换真 cargo：登记 crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
# 里不标 ignore 的小流用例（第一条流按甲二展开 29 个状态），release 下真编真跑；不带时成功行写明真 cargo 那一趟本次未跑。逐格核：
#   ① 不分片跑那条用例打的计数行，与双机分片 merge 之后记进全绿标记的那一行逐字相同；标记作数；清场复原了；第二台这一趟的目录删了
#   ② --merged-log（门禁 54 号调的那一条）：merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数
#   ③ 「第二台」的 rustc -Vv 第一行不同 ⇒ 拒（工具链不同），标记不动
#   ④ 「第二台」那一片没写账本 ⇒ 判红，删这批输入那一格标记
#   ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒（退 1），出路指到 multi-host.env.example
#   ⑥ 没登记 shard=across-machines 的用例 ⇒ 拒
#   ⑦ 发现日志：--merged-log 时三趟各有一份，merge 那一份是 <日志文件>.findings.tsv（一节、begin 行不带 shard=、有 summary），
#      两片的在各自日志旁边（begin 行带 shard=0/2、shard=1/2），第二台上那一份拷回之后删了，三份路径都打进驱动的输出；单独跑时三份在 common-dir 下这一趟的目录里
# 假 cargo 设了 SINGLEFS_LAYER0_FINDINGS_FILE 就往里追加一节（行格式照 crates/singlefs-checker-tier/src/crash.rs 的发现日志），标准输出另打一行 LAYER0_FINDINGS。
#   ⑧ 第二台那一片经 research/scripts/run-with-memory-cap.sh 起（上限是配置的 PEER_MEMORY_CAP）：假 cargo 在 1/2 那一片记下自己的 cgroup，
#      要在 singlefs-memory-cap- 那个 scope 里；驱动的输出里那一行写着经它起；配置缺 PEER_MEMORY_CAP、写成不带单位的数，配置判法拒
#   ⑨ 树里没有规范副本 .claude/singlefs-ai-sop/（HEAD + 暂存区的 worktree 就是这样：它在 .gitignore 里）：树是临时仓的一个 orphan worktree，
#      「第二台」那棵树 git init 过、找不回主仓；驱动在 ② 把驱动所在主仓（临时仓）的规范副本拷过去，第二台算得出同一个输入指纹，判绿，这一趟的树删了
#   ⑩ ③ 之后失败（清场之后回读没过）⇒ 退 1，「第二台」上这一趟的树照 ⑦ 删掉，两片的进度目录留着
#   ⑪ 本机设了 RUN_WITH_MEMORY_CAP_RESERVE=10G ⇒ 第二台那一条 run-with-memory-cap.sh 带同一个值（假 cargo 在 1/2 那一片记下它）；没设（① 那一趟）就不带
#   ⑫ 设了 SINGLEFS_LAYER0_PEER_MEMORY_CAP=2G ⇒ 覆盖配置的 PEER_MEMORY_CAP（第二台那一片的 scope memory.max 是 2G）；没设（① 那一趟）照配置的 1G；
#      设成不带单位的数 ⇒ 拒（退 1）
#   ⑬ 两片跑到一半给驱动（只给它自己，kill "$!"）发 TERM ⇒ 驱动先停两片（假 cargo 在两片里记下的进程号都不在了），再复原、回读、
#      删「第二台」这一趟的树，退 143；两片的进度目录还在
#   ⑭ 双机开关（定义在 research/scripts/layer0-shard-configuration-check.sh）：配置齐、「第二台」连得上、写了 ENABLE_ACROSS_MACHINES=1 ⇒ 判法退 0、报「双机：开，GPU：关」，
#      --emit-assignments 打 ENABLE_ACROSS_MACHINES=1 与没写的 ENABLE_GPU=0；去掉那一行（没写开关）⇒ 判法报「双机：关」退 3（不是「不能用」的报错），
#      --emit-assignments 同样退 3、不打赋值
#   ⑮ 开关写了 0、1 之外的值（ENABLE_ACROSS_MACHINES=yes、ENABLE_GPU=2）⇒ 判不过（退 1），说出是哪个键
# 弄坏开关（配置判法的）MULTI_HOST_CONFIGURATION_BREAK=switch-off-is-on 打开时 ⑭ 判错，switch-value-unchecked 打开时 ⑮ 判错。
# 弄坏开关（驱动脚本的）LAYER0_SHARD_RUN_BREAK=no-peer-findings-copy、no-merge-findings 各自打开时 ⑦ 判错，peer-without-memory-cap 打开时 ⑧ 判错，
# no-sop-copy 打开时 ⑨ 判错，no-peer-cleanup-on-failure 打开时 ⑩ 判错。
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
rsync -a --exclude target --exclude .git --exclude multi-host.env "$repository_root/" "$copy/"
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
# 驱动先 cargo test --no-run --message-format=json 取测试可执行文件、再直接执行它：可执行文件是指回这份脚本的符号链接 stand-in-test-binary，
# 按被叫的名字认出是在当测试二进制跑，照 test 那一支走（参数是 libtest 的那几个）。
if [[ "$(basename "$0")" == stand-in-test-binary ]]; then set -- test "$@"; fi
if [[ "${1:-}" == -V ]]; then echo "cargo 0.0.0-layer0-shard-selftest"; exit 0; fi
[[ "${1:-}" == test ]] || { echo "假 cargo 只认 -V 与 test：$*"; exit 101; }
if [[ " $* " == *" --no-run "* ]]; then
  test_target="" previous_argument=""
  for argument in "$@"; do [[ "$previous_argument" == --test ]] && test_target="$argument"; previous_argument="$argument"; done
  printf '{"reason":"compiler-artifact","target":{"name":"%s","kind":["test"]},"manifest_path":"%s/Cargo.toml","executable":"%s"}\n' \
    "$test_target" "$PWD" "$(dirname "$0")/stand-in-test-binary"
  exit 0
fi
count_line="LAYER0_SHARDED states=29 closed_form=29 violations=0 exhaustive=true"
passed="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s"
write_findings_section() { # write_findings_section <begin 行多出来的字段>：设了 SINGLEFS_LAYER0_FINDINGS_FILE 就追加一节（0 个签名），标准输出打 LAYER0_FINDINGS
  if [[ -n "${SINGLEFS_LAYER0_FINDINGS_FILE:-}" ]]; then
    printf 'layer0_findings_begin\tformat=1\tstream=stand-in\tstates=29%s\nlayer0_findings_summary\tsignatures=0\tred_states=0\tstates=29\tstates_by_finding=none\n' "$1" \
      >> "$SINGLEFS_LAYER0_FINDINGS_FILE" || exit 101
  fi
  echo "LAYER0_FINDINGS signatures=0 red_states=0 states=29"
}
case "${SINGLEFS_LAYER0_SHARD-}" in
  "")
    write_findings_section ""
    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=2 configured_worker_threads=2 worker_threads_source=environment_variable resumed_slices=0 freshly_run_slices=4 progress_file_after_completion=none elapsed_seconds=0.1"
    printf '%s\n%s\n' "$count_line" "$passed" ;;
  0/2|1/2)
    [[ -n "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}" && -n "${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "分片跑没设进度目录或输入指纹"; exit 101; }
    if [[ -n "${LAYER0_SHARD_SELFTEST_STDOUT_KINDS:-}" ]]; then
      # ⑭：记下这一片的标准输出指向哪里（普通文件的路径，还是 pipe:[…]）
      echo "shard=$SINGLEFS_LAYER0_SHARD stdout=$(readlink "/proc/$$/fd/1")" >> "$LAYER0_SHARD_SELFTEST_STDOUT_KINDS"
    fi
    if [[ "$SINGLEFS_LAYER0_SHARD" == 1/2 && -n "${LAYER0_SHARD_SELFTEST_PEER_CGROUP:-}" ]]; then
      own_cgroup="$(sed -n 's/^0:://p' /proc/self/cgroup)"
      { cat /proc/self/cgroup; printf 'memory.max=%s\nreserve=%s\n' "$(cat "/sys/fs/cgroup$own_cgroup/memory.max" 2>/dev/null)" "${RUN_WITH_MEMORY_CAP_RESERVE-unset}"; } >> "$LAYER0_SHARD_SELFTEST_PEER_CGROUP"
    fi
    if [[ -n "${LAYER0_SHARD_SELFTEST_HOLD_FILE:-}" ]]; then
      # ⑬：记下自己的进程号，停在这里等驱动收到 TERM 来停；至多 60 秒，hold 文件被删（自证放走）就退 101
      echo "shard=$SINGLEFS_LAYER0_SHARD pid=$$" >> "$LAYER0_SHARD_SELFTEST_HOLD_FILE"
      for ((held_rounds = 0; held_rounds < 600; held_rounds++)); do
        [[ -e "$LAYER0_SHARD_SELFTEST_HOLD_FILE" ]] || exit 101
        sleep 0.1
      done
      exit 101
    fi
    shard_index="${SINGLEFS_LAYER0_SHARD%/2}"
    ledger="$SINGLEFS_LAYER0_PROGRESS_DIRECTORY/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
    mkdir -p "$SINGLEFS_LAYER0_PROGRESS_DIRECTORY" && printf 'input_fingerprint=%s\n' "$SINGLEFS_LAYER0_INPUT_FINGERPRINT" > "$ledger" || exit 101
    write_findings_section "$(printf '\tshard=%s\tshard_states=15' "$SINGLEFS_LAYER0_SHARD")"
    printf 'LAYER0_PROGRESS slice=1/2 shard=%s\nLAYER0_SHARD mode=run shard=%s ledger=%s\n%s\n' "$SINGLEFS_LAYER0_SHARD" "$SINGLEFS_LAYER0_SHARD" "$ledger" "$passed" ;;
  merge/2)
    [[ -z "${SINGLEFS_LAYER0_START_OVER+set}" ]] || { echo "merge 那一趟带了 SINGLEFS_LAYER0_START_OVER"; exit 101; }
    for shard_index in 0 1; do
      ledger="${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
      [[ "$(cat "$ledger" 2>/dev/null)" == "input_fingerprint=${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "merge：第 $shard_index 片的账本缺或输入指纹不同（$ledger）"; exit 101; }
    done
    echo "LAYER0_SHARD mode=merge shards=2 ledgers=${SINGLEFS_LAYER0_PROGRESS_DIRECTORY}"
    write_findings_section ""
    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=4 configured_worker_threads=4 worker_threads_source=shard_ledgers resumed_slices=0 freshly_run_slices=4 shards=2 shard_worker_threads=2,2 shard_configured_worker_threads=2,2 shard_worker_threads_sources=environment_variable,environment_variable shard_available_parallelism=2,2 shard_resumed_slices=0,0 shard_freshly_run_slices=2,2 shard_elapsed_milliseconds=10,10 progress_file_after_completion=kept elapsed_seconds=0.1"
    printf '%s\n%s\n' "$count_line" "$passed" ;;
  *) echo "假 cargo 认不出 SINGLEFS_LAYER0_SHARD=${SINGLEFS_LAYER0_SHARD}"; exit 101 ;;
esac
STAND_IN_CARGO
  chmod +x "$selftest_cargo"
  ln -s cargo "$work/stand-in-bin/stand-in-test-binary"
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
PEER_MEMORY_CAP=1G
ENABLE_ACROSS_MACHINES=1
QUIESCE_STOP_COMMAND=touch $work/quiesced
QUIESCE_STOPPED_CHECK_COMMAND=test -e $work/quiesced
QUIESCE_START_COMMAND=rm -f $work/quiesced
QUIESCE_STARTED_CHECK_COMMAND=test ! -e $work/quiesced
CONFIGURATION
}
write_configuration "$work/multi-host.env" "$selftest_bin_directory"
export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env"
export SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1
# 三趟 cargo test 都不带 --include-ignored：真 cargo 那条小流用例不标 ignore（同一个测试目标里标了 ignore 的 golden 子进程用例不进过滤范围）；
# 假 cargo 的替身用例标了 ignore，不带它就不算重型
export SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0
unset SINGLEFS_LAYER0_SHARD SINGLEFS_LAYER0_PROGRESS_DIRECTORY SINGLEFS_LAYER0_INPUT_FINGERPRINT SINGLEFS_LAYER0_START_OVER
# ⑪ ⑫ 之外的各格都是「没设」：余量与第二台上限的覆盖开关不从调用方漏进来
unset RUN_WITH_MEMORY_CAP_RESERVE SINGLEFS_LAYER0_PEER_MEMORY_CAP
driver="$copy/research/scripts/layer0-shard-run.sh"
admission_module="$copy/research/scripts/admission.py"

# 不分片跑一次那条用例：计数行是 merge 之后要逐字相同的那一行
(cd "$copy" && cargo test --release -p singlefs-harness --test "$selftest_target" -- --exact "$selftest_function" --nocapture) > "$work/unsharded.log" 2>&1
unsharded_exit=$?
unsharded_count_line="$(grep '^LAYER0_SHARDED ' "$work/unsharded.log")"
expect "不分片跑登记的那条用例：退 0，恰好一行 LAYER0_SHARDED 计数行" "$([[ $unsharded_exit == 0 && $(grep -c '^LAYER0_SHARDED ' "$work/unsharded.log") == 1 ]]; echo $?)" \
  "退 $unsharded_exit；日志尾部：$(tail -5 "$work/unsharded.log" | tr '\n' '|')"

# ① 单独跑：两片、merge、判、写标记（假 cargo 在第二台那一片记下自己的 cgroup，⑧ 核它在内存包装的 scope 里）
export LAYER0_SHARD_SELFTEST_PEER_CGROUP="$work/peer-cgroup"
LAYER0_SHARD_SELFTEST_STDOUT_KINDS="$work/stdout-kinds" bash "$driver" "$selftest_case_key" "$copy" > "$work/standalone.log" 2>&1
standalone_exit=$?
# ⑭ 两片的标准输出都是文件、不是管道：读输出的一方（显示用的 tail、第二台的 ssh）断了，测试进程照样写文件，
#    不会因写断掉的管道 panic、连带删掉进度文件（2026-09-28 预演第二台那一片续跑读回 0 片的成因）
stdout_kind_problems=()
for shard_name in 0/2 1/2; do
  stdout_target="$(grep "^shard=$shard_name " "$work/stdout-kinds" 2>/dev/null | head -1 | sed 's/.* stdout=//')"
  if [[ -z "$stdout_target" ]]; then stdout_kind_problems+=("第 $shard_name 片没记下标准输出")
  elif [[ "$stdout_target" != /* ]]; then stdout_kind_problems+=("第 $shard_name 片的标准输出是 $stdout_target，不是文件（管道记成 pipe:[…]；第二台那一份文件跑完随 ⑦ 删了，只看记下的是不是路径）")
  fi
done
expect "⑭ 两片的测试进程标准输出都直接写文件、不经管道" "$( (( ${#stdout_kind_problems[@]} == 0 )); echo $?)" \
  "$(IFS='；'; echo "${stdout_kind_problems[*]}")；记下的：$(tr '\n' '|' < "$work/stdout-kinds" 2>/dev/null)"
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

# ⑧ 第二台那一片经内存包装起
peer_cap_problems=()
grep -q '第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 bash research/scripts/run-with-memory-cap.sh 1G 起' "$work/standalone.log" \
  || peer_cap_problems+=("驱动的输出里 ④ 那一行没写第二台那一片经 run-with-memory-cap.sh 1G 起")
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]] && ! grep -q 'singlefs-memory-cap-' "$work/peer-cgroup" 2>/dev/null; then
  peer_cap_problems+=("假 cargo 在第二台那一片记下的 cgroup 不在 singlefs-memory-cap- 的 scope 里：$(tr '\n' ' ' < "$work/peer-cgroup" 2>/dev/null)")
fi
expect "⑧ 第二台那一片经 research/scripts/run-with-memory-cap.sh 起（上限是配置的 PEER_MEMORY_CAP=1G）" "${#peer_cap_problems[@]}" \
  "$(IFS='；'; echo "${peer_cap_problems[*]}")"
unset LAYER0_SHARD_SELFTEST_PEER_CGROUP

# ② --merged-log：门禁 54 号调的那一条
bash "$driver" --merged-log "$selftest_case_key" "$copy" "$fingerprint" "$work/merged.log" > "$work/merged-driver.log" 2>&1
merged_exit=$?
expect "② --merged-log：退 0，merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数" \
  "$([[ $merged_exit == 0 ]] && grep -q 'LAYER0_SHARD mode=merge shards=2 ' "$work/merged.log" && [[ "$(grep '^LAYER0_SHARDED ' "$work/merged.log")" == "$unsharded_count_line" ]]; echo $?)" \
  "退 $merged_exit；驱动脚本输出尾部：$(tail -10 "$work/merged-driver.log" | tr '\n' '|')"

# ⑦ 发现日志：merge 那一份是 <日志文件>.findings.tsv（54 号读的那一份），两片的在各自日志旁边，第二台上那一份拷回之后删了，三份路径都打进输出
findings_problems=()
merge_findings="$work/merged.log.findings.tsv"
local_findings="$work/merged.log.shard-0-of-2.log.findings.tsv"
fetched_findings="$work/merged.log.shard-1-of-2.log.findings.tsv"
[[ "$(grep -c '^layer0_findings_begin' "$merge_findings" 2>/dev/null)" == 1 && "$(grep -c '^layer0_findings_summary' "$merge_findings" 2>/dev/null)" == 1 ]] \
  || findings_problems+=("merge 那一份 $merge_findings 不是恰好一节带 summary")
if grep -q $'^layer0_findings_begin\t.*shard=' "$merge_findings" 2>/dev/null; then findings_problems+=("merge 那一份的 begin 行带了 shard="); fi
grep -q $'^layer0_findings_begin\t.*\tshard=0/2\t' "$local_findings" 2>/dev/null || findings_problems+=("本机那一片的 $local_findings 没有 shard=0/2 那一节")
grep -q $'^layer0_findings_begin\t.*\tshard=1/2\t' "$fetched_findings" 2>/dev/null || findings_problems+=("第二台那一片没拷回到 $fetched_findings（或没有 shard=1/2 那一节）")
leftover_peer_findings="$(find "$work/peer/runs" -maxdepth 1 -name '*.findings.tsv' 2>/dev/null)"
[[ -z "$leftover_peer_findings" ]] || findings_problems+=("第二台上的发现日志没删：$leftover_peer_findings")
for findings_path in "$local_findings" "$fetched_findings" "$merge_findings"; do
  grep -qF -- "$findings_path" "$work/merged-driver.log" || findings_problems+=("驱动的输出里没打 $findings_path")
done
standalone_findings_count="$(find "$(git -C "$copy" rev-parse --path-format=absolute --git-common-dir)/singlefs-layer0-logs" -name '*.findings.tsv' -size +0 2>/dev/null | grep -c .)"
[[ "$standalone_findings_count" == 3 ]] || findings_problems+=("单独跑那一趟 common-dir 下 singlefs-layer0-logs/ 里非空的发现日志是 ${standalone_findings_count} 份，不是 3 份")
expect "⑦ 发现日志：merge 那一份在 <日志文件>.findings.tsv（一节、不带 shard=、有 summary），两片的在各自日志旁边（shard=0/2、1/2），第二台上那一份拷回后删了，三份路径都打进输出；单独跑时三份在 common-dir 下" \
  "${#findings_problems[@]}" "$(IFS='；'; echo "${findings_problems[*]}")"

# ③ 「第二台」的 rustc -Vv 第一行不同
mkdir -p "$work/other-toolchain"
printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$selftest_cargo" > "$work/other-toolchain/cargo"
printf '#!/usr/bin/env bash\nif [[ "${1:-}" == -Vv ]]; then %q -Vv | sed "1s/\$/ (another build)/"; else exec %q "$@"; fi\n' "$real_rustc" "$real_rustc" > "$work/other-toolchain/rustc"
chmod +x "$work/other-toolchain/cargo" "$work/other-toolchain/rustc"
write_configuration "$work/other-toolchain.env" "$work/other-toolchain"
SINGLEFS_MULTI_HOST_CONFIG="$work/other-toolchain.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/other-toolchain.log" 2>&1
toolchain_exit=$?
python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
toolchain_marker_exit=$?
expect "③ 「第二台」rustc -Vv 第一行不同 ⇒ 拒（退 1，说工具链不同），这批输入那一格标记不动" \
  "$([[ $toolchain_exit == 1 && $toolchain_marker_exit == 0 ]] && grep -q '两台的工具链不同' "$work/other-toolchain.log"; echo $?)" \
  "退 $toolchain_exit，标记核 $toolchain_marker_exit；输出：$(tail -4 "$work/other-toolchain.log" | tr '\n' '|')"

# ④ 「第二台」那一片退 0 却没写账本
mkdir -p "$work/no-ledger"
# 驱动先 cargo test --no-run 取可执行文件、再直接执行它：--no-run 交出这一格自己的「不写账本」测试二进制
cat > "$work/no-ledger/cargo" <<NO_LEDGER_CARGO
#!/usr/bin/env bash
if [[ "\${1:-}" == test && " \$* " == *" --no-run "* ]]; then
  printf '{"reason":"compiler-artifact","target":{"name":"%s","kind":["test"]},"manifest_path":"%s/Cargo.toml","executable":"%s"}\n' $(printf '%q' "$selftest_target") "\$PWD" $(printf '%q' "$work/no-ledger/stand-in-test-binary")
  exit 0
fi
if [[ "\${1:-}" == test ]]; then echo "LAYER0_SHARD mode=run shard=1/2 （自证：这一片不写账本）"; exit 0; fi
exec $(printf '%q' "$selftest_cargo") "\$@"
NO_LEDGER_CARGO
printf '#!/usr/bin/env bash\necho "LAYER0_SHARD mode=run shard=1/2 （自证：这一片不写账本）"\nexit 0\n' > "$work/no-ledger/stand-in-test-binary"
printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$real_rustc" > "$work/no-ledger/rustc"
chmod +x "$work/no-ledger/cargo" "$work/no-ledger/rustc" "$work/no-ledger/stand-in-test-binary"
write_configuration "$work/no-ledger.env" "$work/no-ledger"
git_common_directory="$(git -C "$copy" rev-parse --path-format=absolute --git-common-dir)"
rm -f -- "${git_common_directory:?}/singlefs-layer0-progress/$fingerprint"/layer0-shard-*.tally "${work:?}/peer/progress/$fingerprint"/layer0-shard-*.tally
SINGLEFS_MULTI_HOST_CONFIG="$work/no-ledger.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-ledger.log" 2>&1
no_ledger_exit=$?
python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
no_ledger_marker_exit=$?
expect "④ 「第二台」那一片没写账本 ⇒ 判红（退 1，说两片的账本不是各恰好一份），删这批输入那一格标记" \
  "$([[ $no_ledger_exit == 1 && $no_ledger_marker_exit == 1 ]] && grep -q '两片的账本不是各恰好一份' "$work/no-ledger.log"; echo $?)" \
  "退 $no_ledger_exit，标记核 $no_ledger_marker_exit；输出：$(tail -4 "$work/no-ledger.log" | tr '\n' '|')"

# ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒
grep -v '^PEER_CARGO_BIN_DIRECTORY=' "$work/multi-host.env" > "$work/missing-key.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/missing-key.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/missing-key.log" 2>&1
missing_key_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/no-such-configuration.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-configuration.log" 2>&1
no_configuration_exit=$?
expect "⑤ 配置缺键 PEER_CARGO_BIN_DIRECTORY、配置文件不在 ⇒ 配置判法各拒（退 1，说双机分片不能用），出路指到 multi-host.env.example" \
  "$([[ $missing_key_exit == 1 && $no_configuration_exit == 1 ]] && grep -q '双机分片不能用' "$work/missing-key.log" && grep -q '双机分片不能用' "$work/no-configuration.log" \
     && grep -q 'multi-host.env.example' "$work/missing-key.log" && grep -q 'multi-host.env.example' "$work/no-configuration.log"; echo $?)" \
  "缺键退 $missing_key_exit（$(tail -3 "$work/missing-key.log" | tr '\n' '|')），不在退 $no_configuration_exit（$(tail -3 "$work/no-configuration.log" | tr '\n' '|')）"

# ⑧ 配置缺 PEER_MEMORY_CAP、写成不带单位的数 ⇒ 配置判法拒
grep -v '^PEER_MEMORY_CAP=' "$work/multi-host.env" > "$work/missing-memory-cap.env"
sed 's/^PEER_MEMORY_CAP=.*/PEER_MEMORY_CAP=16/' "$work/multi-host.env" > "$work/unitless-memory-cap.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/missing-memory-cap.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/missing-memory-cap.log" 2>&1
missing_memory_cap_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/unitless-memory-cap.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/unitless-memory-cap.log" 2>&1
unitless_memory_cap_exit=$?
expect "⑧ 配置缺 PEER_MEMORY_CAP、写成不带单位的 16 ⇒ 配置判法各拒（退 1），说出是哪个键" \
  "$([[ $missing_memory_cap_exit == 1 && $unitless_memory_cap_exit == 1 ]] && grep -q '缺键 PEER_MEMORY_CAP' "$work/missing-memory-cap.log" \
     && grep -q 'PEER_MEMORY_CAP 要写第二台那一片的内存上限' "$work/unitless-memory-cap.log"; echo $?)" \
  "缺键退 $missing_memory_cap_exit（$(tail -3 "$work/missing-memory-cap.log" | tr '\n' '|')），不带单位退 $unitless_memory_cap_exit（$(tail -3 "$work/unitless-memory-cap.log" | tr '\n' '|')）"

# ⑭ 双机开关：写了 1 ⇒ 开；没写 ⇒ 双机：关（退 3，不是判不过）
configuration_check="$copy/research/scripts/layer0-shard-configuration-check.sh"
SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" bash "$configuration_check" "$copy" > "$work/switch-on.log" 2>&1
switch_on_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" bash "$configuration_check" --emit-assignments "$copy" > "$work/switch-on-emit.log" 2>&1
switch_on_emit_exit=$?
grep -v '^ENABLE_ACROSS_MACHINES=' "$work/multi-host.env" > "$work/switch-absent.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/switch-absent.env" bash "$configuration_check" "$copy" > "$work/switch-absent.log" 2>&1
switch_absent_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/switch-absent.env" bash "$configuration_check" --emit-assignments "$copy" > "$work/switch-absent-emit.log" 2>&1
switch_absent_emit_exit=$?
expect "⑭ 写了 ENABLE_ACROSS_MACHINES=1 ⇒ 退 0、双机：开，GPU：关，赋值里 ENABLE_GPU=0；没写开关 ⇒ 判法报「双机：关」退 3（不是判不过），--emit-assignments 同样退 3、不打赋值" \
  "$([[ $switch_on_exit == 0 && $switch_on_emit_exit == 0 && $switch_absent_exit == 3 && $switch_absent_emit_exit == 3 ]] \
     && grep -q '^双机：开，GPU：关；' "$work/switch-on.log" && grep -qx 'ENABLE_ACROSS_MACHINES=1' "$work/switch-on-emit.log" && grep -qx 'ENABLE_GPU=0' "$work/switch-on-emit.log" \
     && grep -q '^双机：关' "$work/switch-absent.log" && grep -q 'ENABLE_ACROSS_MACHINES=1' "$work/switch-absent.log" && ! grep -q '不能用' "$work/switch-absent.log" \
     && ! grep -q '^PEER_SSH_HOST=' "$work/switch-absent-emit.log"; echo $?)" \
  "写了 1：退 $switch_on_exit / emit 退 $switch_on_emit_exit（$(head -1 "$work/switch-on.log")）；没写：退 $switch_absent_exit / emit 退 $switch_absent_emit_exit（$(tr '\n' '|' < "$work/switch-absent.log")）"

# ⑮ 开关写了 0、1 之外的值 ⇒ 判不过
sed 's/^ENABLE_ACROSS_MACHINES=.*/ENABLE_ACROSS_MACHINES=yes/' "$work/multi-host.env" > "$work/switch-other.env"
{ cat "$work/multi-host.env"; echo ENABLE_GPU=2; } > "$work/gpu-other.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/switch-other.env" bash "$configuration_check" "$copy" > "$work/switch-other.log" 2>&1
switch_other_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/gpu-other.env" bash "$configuration_check" "$copy" > "$work/gpu-other.log" 2>&1
gpu_other_exit=$?
expect "⑮ 开关写了 0、1 之外的值（ENABLE_ACROSS_MACHINES=yes、ENABLE_GPU=2）⇒ 判不过（退 1），说出是哪个键" \
  "$([[ $switch_other_exit == 1 && $gpu_other_exit == 1 ]] && grep -q 'ENABLE_ACROSS_MACHINES 只许写 0 或 1' "$work/switch-other.log" \
     && grep -q 'ENABLE_GPU 只许写 0 或 1' "$work/gpu-other.log"; echo $?)" \
  "ENABLE_ACROSS_MACHINES=yes 退 $switch_other_exit（$(tail -2 "$work/switch-other.log" | tr '\n' '|')），ENABLE_GPU=2 退 $gpu_other_exit（$(tail -2 "$work/gpu-other.log" | tr '\n' '|')）"

# ⑥ 没登记 shard=across-machines 的用例
bash "$driver" crash-case:c561-sigma-full "$copy" > "$work/not-shardable.log" 2>&1
not_shardable_exit=$?
expect "⑥ 没登记 shard=across-machines 的用例 ⇒ 拒（退 1），出路是单机跑" \
  "$([[ $not_shardable_exit == 1 ]] && grep -q '没登记 shard=across-machines' "$work/not-shardable.log"; echo $?)" \
  "退 $not_shardable_exit；输出：$(tail -3 "$work/not-shardable.log" | tr '\n' '|')"

# ⑨ 树里没有规范副本、「第二台」那棵树找不回主仓：树是临时仓的 orphan worktree（git common-dir 指回临时仓，本机的准入模块从临时仓找得到规范副本），
# 文件按硬链接从临时仓铺过去、不带规范副本；驱动用这棵树里的那一份（与门禁 54 号 --full 调 $ROOT/research/scripts/layer0-shard-run.sh 同形）
tree_without_sop_copy="$work/tree-without-sop-copy"
peer_runs_before_sop_cell="$(ls -A "$work/peer/runs" 2>/dev/null)"
if git -C "$copy" worktree add -q --orphan -b layer0-shard-selftest-tree "$tree_without_sop_copy" > "$work/worktree-add.log" 2>&1 \
    && rsync -a --link-dest="$copy" --exclude target --exclude .git --exclude /.claude/singlefs-ai-sop "$copy/" "$tree_without_sop_copy/" >> "$work/worktree-add.log" 2>&1; then
  bash "$tree_without_sop_copy/research/scripts/layer0-shard-run.sh" "$selftest_case_key" "$tree_without_sop_copy" > "$work/without-sop-copy.log" 2>&1
  without_sop_copy_exit=$?
else
  without_sop_copy_exit="worktree-not-built（$(tail -3 "$work/worktree-add.log" | tr '\n' '|')）"
fi
expect "⑨ 树里没有规范副本、「第二台」那棵树找不回主仓 ⇒ 驱动在 ② 把驱动所在主仓的规范副本拷过去，第二台算得出同一个输入指纹，判绿，这一趟的树删了" \
  "$([[ $without_sop_copy_exit == 0 && ! -e "$tree_without_sop_copy/.claude/singlefs-ai-sop" && -z "$peer_runs_before_sop_cell" && -z "$(ls -A "$work/peer/runs" 2>/dev/null)" ]] \
     && grep -qF "规范副本 $copy/.claude/singlefs-ai-sop 拷到第二台" "$work/without-sop-copy.log" && grep -q '③ 两台的输入指纹相同' "$work/without-sop-copy.log"; echo $?)" \
  "退 $without_sop_copy_exit；跑之前第二台 runs/ 下「${peer_runs_before_sop_cell//$'\n'/ }」，跑之后「$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')」；输出尾部：$(tail -6 "$work/without-sop-copy.log" 2>/dev/null | tr '\n' '|')"

# ⑩ ③ 之后失败：清场之后回读没过（QUIESCE_STOPPED_CHECK_COMMAND=false）；第二台上这一趟的树照 ⑦ 删掉，进度目录留着
sed 's/^QUIESCE_STOPPED_CHECK_COMMAND=.*/QUIESCE_STOPPED_CHECK_COMMAND=false/' "$work/multi-host.env" > "$work/quiesce-check-fails.env"
peer_runs_before_cleanup_cell="$(ls -A "$work/peer/runs" 2>/dev/null)"
SINGLEFS_MULTI_HOST_CONFIG="$work/quiesce-check-fails.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/quiesce-check-fails.log" 2>&1
quiesce_check_fails_exit=$?
expect "⑩ ③ 之后失败（清场之后回读没过）⇒ 退 1，「第二台」上这一趟的树照 ⑦ 删掉，两片的进度目录留着" \
  "$([[ $quiesce_check_fails_exit == 1 && "$(ls -A "$work/peer/runs" 2>/dev/null)" == "$peer_runs_before_cleanup_cell" && -d "$work/peer/progress/$fingerprint" ]] \
     && grep -q '清场之后回读没过' "$work/quiesce-check-fails.log" && grep -q '⑦ 删掉第二台上这一趟的树与编译目录' "$work/quiesce-check-fails.log"; echo $?)" \
  "退 $quiesce_check_fails_exit；跑之前第二台 runs/ 下「${peer_runs_before_cleanup_cell//$'\n'/ }」，跑之后「$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')」；进度目录 $work/peer/progress/$fingerprint $([[ -d "$work/peer/progress/$fingerprint" ]] && echo 在 || echo 不在)；输出尾部：$(tail -6 "$work/quiesce-check-fails.log" | tr '\n' '|')"

# ⑪ ⑫ 余量与第二台上限的覆盖：设了就带给第二台那一条包装 / 覆盖配置；没设的情形核 ① 那一趟（假 cargo 记下的在 $work/peer-cgroup）
export LAYER0_SHARD_SELFTEST_PEER_CGROUP="$work/peer-cgroup-overridden"
RUN_WITH_MEMORY_CAP_RESERVE=10G SINGLEFS_LAYER0_PEER_MEMORY_CAP=2G bash "$driver" "$selftest_case_key" "$copy" > "$work/overridden.log" 2>&1
overridden_exit=$?
SINGLEFS_LAYER0_PEER_MEMORY_CAP=2 bash "$driver" "$selftest_case_key" "$copy" > "$work/unitless-peer-cap.log" 2>&1
unitless_peer_cap_exit=$?
unset LAYER0_SHARD_SELFTEST_PEER_CGROUP
reserve_problems=()
[[ $overridden_exit == 0 ]] || reserve_problems+=("设了两样那一趟退 $overridden_exit：$(tail -4 "$work/overridden.log" | tr '\n' '|')")
grep -q '第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 RUN_WITH_MEMORY_CAP_RESERVE=10G bash research/scripts/run-with-memory-cap.sh ' "$work/overridden.log" \
  || reserve_problems+=("设了余量那一趟，驱动 ④ 那一行没写第二台经 RUN_WITH_MEMORY_CAP_RESERVE=10G 起包装")
grep -q '第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 bash research/scripts/run-with-memory-cap.sh ' "$work/standalone.log" \
  || reserve_problems+=("没设余量的 ① 那一趟，驱动 ④ 那一行带了余量（或没经包装）")
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  grep -qx 'reserve=10G' "$work/peer-cgroup-overridden" 2>/dev/null || reserve_problems+=("设了余量那一趟，第二台那一片记下的：$(grep '^reserve=' "$work/peer-cgroup-overridden" 2>/dev/null)")
  grep -qx 'reserve=unset' "$work/peer-cgroup" 2>/dev/null || reserve_problems+=("没设余量的 ① 那一趟，第二台那一片记下的：$(grep '^reserve=' "$work/peer-cgroup" 2>/dev/null)")
fi
expect "⑪ 本机设了 RUN_WITH_MEMORY_CAP_RESERVE=10G ⇒ 第二台那一条 run-with-memory-cap.sh 带同一个值；没设就不带" "${#reserve_problems[@]}" \
  "$(IFS='；'; echo "${reserve_problems[*]}")"
peer_cap_override_problems=()
grep -q '第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 RUN_WITH_MEMORY_CAP_RESERVE=10G bash research/scripts/run-with-memory-cap.sh 2G 起' "$work/overridden.log" \
  || peer_cap_override_problems+=("设了 SINGLEFS_LAYER0_PEER_MEMORY_CAP=2G 那一趟，驱动 ④ 那一行上限不是 2G")
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  grep -qx 'memory.max=2147483648' "$work/peer-cgroup-overridden" 2>/dev/null \
    || peer_cap_override_problems+=("设了 2G 那一趟，第二台那一片的 scope：$(grep '^memory.max=' "$work/peer-cgroup-overridden" 2>/dev/null)")
  grep -qx 'memory.max=1073741824' "$work/peer-cgroup" 2>/dev/null \
    || peer_cap_override_problems+=("没设的 ① 那一趟，第二台那一片的 scope 不是配置的 1G：$(grep '^memory.max=' "$work/peer-cgroup" 2>/dev/null)")
fi
[[ $unitless_peer_cap_exit == 1 ]] && grep -q 'SINGLEFS_LAYER0_PEER_MEMORY_CAP 要写第二台那一片的内存上限' "$work/unitless-peer-cap.log" \
  || peer_cap_override_problems+=("SINGLEFS_LAYER0_PEER_MEMORY_CAP=2（不带单位）没被拒：退 $unitless_peer_cap_exit，$(tail -3 "$work/unitless-peer-cap.log" | tr '\n' '|')")
expect "⑫ 设了 SINGLEFS_LAYER0_PEER_MEMORY_CAP=2G ⇒ 覆盖配置的 PEER_MEMORY_CAP（第二台那一片的 scope 是 2G）；没设照配置的 1G；不带单位的拒" \
  "${#peer_cap_override_problems[@]}" "$(IFS='；'; echo "${peer_cap_override_problems[*]}")"

# ⑬ 两片跑到一半给驱动发 TERM（假 cargo 在两片里各记下进程号、停着等）；只给驱动自己发（kill "$!"），不发给它的子进程
hold_file="$work/shards-held"
: > "$hold_file"
peer_runs_before_interrupt="$(ls -A "$work/peer/runs" 2>/dev/null)"
export LAYER0_SHARD_SELFTEST_HOLD_FILE="$hold_file"
bash "$driver" "$selftest_case_key" "$copy" > "$work/interrupted.log" 2>&1 &
interrupted_driver=$!
unset LAYER0_SHARD_SELFTEST_HOLD_FILE
for ((waited_rounds = 0; waited_rounds < 600; waited_rounds++)); do
  [[ "$(grep -c '^shard=' "$hold_file")" == 2 ]] && break
  sleep 0.1
done
mapfile -t held_processes < <(sed -n 's/^shard=.* pid=//p' "$hold_file")
kill -TERM "$interrupted_driver"
wait "$interrupted_driver"
interrupted_exit=$?
still_running=()
for held_process in "${held_processes[@]}"; do
  held_state="$(ps -o stat= -p "$held_process" 2>/dev/null)"
  [[ -z "$held_state" || "$held_state" == Z* ]] || still_running+=("$held_process")
done
rm -f -- "$hold_file"  # 没停掉的两片看到它不在就自己退
interrupt_problems=()
[[ $interrupted_exit == 143 ]] || interrupt_problems+=("驱动退 $interrupted_exit，不是 143")
[[ ${#held_processes[@]} == 2 ]] || interrupt_problems+=("发 TERM 之前两片没都停在那里（记下 ${#held_processes[@]} 个进程号）")
(( ${#still_running[@]} == 0 )) || interrupt_problems+=("TERM 之后两片还在跑：${still_running[*]}")
grep -q '复原之后回读过了' "$work/interrupted.log" || interrupt_problems+=("没复原回读")
[[ ! -e "$work/quiesced" ]] || interrupt_problems+=("清场标记还在（没复原）")
[[ "$(ls -A "$work/peer/runs" 2>/dev/null)" == "$peer_runs_before_interrupt" ]] || interrupt_problems+=("第二台 runs/ 下多了：$(ls -A "$work/peer/runs" | tr '\n' ' ')")
[[ -d "$work/peer/progress/$fingerprint" && -d "$git_common_directory/singlefs-layer0-progress/$fingerprint" ]] || interrupt_problems+=("两片的进度目录不在了")
expect "⑬ 两片跑到一半给驱动发 TERM ⇒ 先停两片，再复原、回读、删第二台这一趟的树，退 143，进度目录还在" "${#interrupt_problems[@]}" \
  "$(IFS='；'; echo "${interrupt_problems[*]}")；驱动输出尾部：$(tail -8 "$work/interrupted.log" | tr '\n' '|')"

if (( failures > 0 )); then
  echo "  ✗ layer0-shard-run.sh 自证没过：${failures} 格判错（共 ${checked} 格）"
  echo "     → 怎么办：照上面每一格的说明改驱动脚本（research/scripts/layer0-shard-run.sh）或分片那一段（crates/singlefs-checker-tier/src/layer0_progress.rs、crash.rs）"
  exit 1
fi
echo "  ✓ layer0-shard-run.sh 自证通过：${checked} 格都对（${cargo_mode}；第二台是本机上的另一个目录，没碰真的第二台）"
