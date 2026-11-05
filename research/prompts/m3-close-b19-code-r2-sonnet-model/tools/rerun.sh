#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条经内存包装跑，工作目录随意）：
#   bash scripts/hosts/run-with-memory-cap.sh <上限> bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 scripts/hosts/run-with-memory-cap.sh（判是不是本仓）
#     --frozen  冻结副本（腿开工时拷出的那棵树）：要是一个目录
#     --work    新的草稿目录：不在、或是空目录；跑出来的全写在这里
#     --model   模型目录（原型、变异表、out/）：默认是本脚本的上一层——模型目录整份拷走时它跟着走；只拷这一份脚本时用它指过去
# 退出码：0 跑完；2 用法错（缺参数、仓根不像本仓、冻结副本不在、草稿目录已存在且非空、模型目录不在），stderr 列用法；
#         别的是 rerun_steps 里出错那一步的退出码。--selftest：0 全过，1 有格红。
#
# 准入与运行条件不写、不调 preflight：这份模板拷进模型目录、草稿目录跑，不从自身路径就找不到 .claude/scripts/preflight.sh；
# 环境要求（仓根、冻结副本、草稿目录）由 parse_arguments 自己判，不满足退 2。
#
# 弄坏开关（只给 --selftest 证红用）：
#   RERUN_TEMPLATE_BREAK=root-from-self-path  仓根改回从本脚本路径往上推四层（旧 rerun.sh 的写法），--repo 不认
#       「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」那一格红
#       连带红的格：「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条经 scripts/hosts/run-with-memory-cap.sh 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
      rerun.sh --selftest
USAGE
  exit 2
}

parse_arguments() {
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""
  while (($#)); do
    case "$1" in
      --repo|--frozen|--work|--model)
        (($# >= 2)) || refuse "$1 后面要跟一个路径"
        case "$1" in
          --repo) REPO_ROOT="$2" ;;
          --frozen) FROZEN="$2" ;;
          --work) WORK="$2" ;;
          --model) MODEL="$2" ;;
        esac
        shift 2 ;;
      *) refuse "认不出参数：$1" ;;
    esac
  done
  [[ -n "$REPO_ROOT" ]] || refuse "缺 --repo <仓根>"
  [[ -n "$FROZEN" ]] || refuse "缺 --frozen <冻结副本>"
  [[ -n "$WORK" ]] || refuse "缺 --work <新的草稿目录>"
  if [[ "${RERUN_TEMPLATE_BREAK:-}" == root-from-self-path ]]; then
    REPO_ROOT="$(dirname "${BASH_SOURCE[0]}")/../../../.."
  fi
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 scripts/hosts/run-with-memory-cap.sh）"
  REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
  [[ -d "$FROZEN" ]] || refuse "--frozen $FROZEN 不是目录"
  FROZEN="$(cd "$FROZEN" && pwd)"
  MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
  [[ -d "$MODEL" ]] || refuse "模型目录 $MODEL 不在（只拷了这一份脚本的，用 --model 指过去）"
  MODEL="$(cd "$MODEL" && pwd)"
  if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿要跑的步骤（辩方腿 m3-close-b19-code-r2-sonnet）──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引。两棵树：
#   $WORK/new  冻结副本的 crates 等，加本腿 tests/ 里五份探针（逐盘核在续做预演之前，第二轮修补之后的形态）
#   $WORK/old  new 反向打上 patches/r2-fix-as-given.diff（逐盘核在续做预演之后，第二轮修补之前的形态）
# 每一步输出落 $WORK/out/<名>.raw，筛出 PROBE、test、test result 三类行（去掉耗时、线程号，按行排序）与 $MODEL/out/<名>.txt 逐行比，差异打 DIFF、一致打 SAME。
# cargo 命令逐条写字面，整条脚本由外面的内存包装跑（不再各自包一层）。
filtered_lines_without_the_timing() {   # filtered_lines_without_the_timing <raw 文件>
  grep -E 'PROBE|^test |test result|panicked' "$1" | sed -E 's/finished in [0-9.]+s/finished in Ns/; s#\(/[^)]*\)##; s/ \([0-9]+\) panicked/ panicked/' | LC_ALL=C sort || true
}

compare_with_the_saved_output() {   # compare_with_the_saved_output <名>
  local name="$1"
  filtered_lines_without_the_timing "$WORK/out/$name.raw" > "$WORK/out/$name.txt"
  if diff -u "$MODEL/out/$name.txt" "$WORK/out/$name.txt" > "$WORK/out/$name.diff"; then
    echo "SAME name=$name lines=$(wc -l < "$WORK/out/$name.txt")"
  else
    echo "DIFF name=$name see=$WORK/out/$name.diff"
  fi
}

rerun_steps() {
  mkdir -p "$WORK/out"
  if [[ "${RERUN_TEMPLATE_SELFTEST_MARK:-}" == rerun-template-selftest ]]; then
    mkdir -p "$WORK/tree"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
    echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
    return 0
  fi
  mkdir -p "$WORK/new/.claude" "$WORK/old"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/new/"
  cp "$REPO_ROOT/.claude/heavy-thread-default" "$WORK/new/.claude/heavy-thread-default"
  cp "$MODEL"/tests/r2_probe_*.rs "$WORK/new/crates/singlefs-harness/tests/"
  mkdir -p "$WORK/old/.claude"
  rsync -a --exclude target "$WORK/new/" "$WORK/old/"
  (cd "$WORK/old" && patch -R -p1 < "$MODEL/patches/r2-fix-as-given.diff" > "$WORK/out/reverse-patch.txt" 2>&1)
  echo "name=trees new_mount_sha256=$(sha256sum "$WORK/new/crates/singlefs-core/src/mount.rs" | cut -c1-16) old_mount_sha256=$(sha256sum "$WORK/old/crates/singlefs-core/src/mount.rs" | cut -c1-16)"

  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r2_probe_u1_check_before_rehearsal; 范围=1 个测试目标; 为什么=复跑辩方腿 U1 探针，看逐盘核在续做预演之前时两样同时成立报哪一个' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test r2_probe_u1_check_before_rehearsal -- --nocapture --test-threads=1) > "$WORK/out/new-u1.raw" 2>&1 || true
  compare_with_the_saved_output new-u1
  (cd "$WORK/old" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r2_probe_u1_check_before_rehearsal; 范围=1 个测试目标; 为什么=复跑辩方腿 U1 探针在还原成修补之前次序的树上的结局，旧次序下该红两条' CARGO_TARGET_DIR="$WORK/target-old" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --no-fail-fast --test r2_probe_u1_check_before_rehearsal -- --nocapture --test-threads=1) > "$WORK/out/old-u1.raw" 2>&1 || true
  compare_with_the_saved_output old-u1
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r2_probe_u2_zeroed_slots_with_read_faults_in_the_rehearsal; 范围=1 个测试目标; 为什么=复跑辩方腿 U2 探针，看补写盘两槽叠上读故障时挂载怎么收尾' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test r2_probe_u2_zeroed_slots_with_read_faults_in_the_rehearsal -- --nocapture --test-threads=1) > "$WORK/out/new-u2.raw" 2>&1 || true
  compare_with_the_saved_output new-u2
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r2_probe_u4_failed_resend_then_resend; 范围=1 个测试目标; 为什么=复跑辩方腿 U4 探针，看冻结之后重发先失败再成功、之后进 discard 调用点发不发' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test r2_probe_u4_failed_resend_then_resend -- --nocapture --test-threads=1) > "$WORK/out/new-u4.raw" 2>&1 || true
  compare_with_the_saved_output new-u4
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r2_probe_u5_state_after_the_rollback_and_after_a_crash; 范围=1 个测试目标; 为什么=复跑辩方腿 U5 探针，看回卷之后与取号成功后崩溃之后的下一次挂载走哪一支' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test r2_probe_u5_state_after_the_rollback_and_after_a_crash -- --nocapture --test-threads=1) > "$WORK/out/new-u5.raw" 2>&1 || true
  compare_with_the_saved_output new-u5
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r2_probe_u6_instance_table_sweep_threshold_by_mounting_many_times; 范围=1 个测试目标; 为什么=复跑辩方腿 U6 探针，反复挂载看产品默认阈值 365 下清扫第一次触发的行数与链长' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test r2_probe_u6_instance_table_sweep_threshold_by_mounting_many_times -- --nocapture --test-threads=1) > "$WORK/out/new-u6.raw" 2>&1 || true
  compare_with_the_saved_output new-u6

  # 钉读序数的四个目标与第二轮修补带来的三个新用例（新树）、钉读序数的四个目标加 T4 用例（旧树，T4 用例该红）
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的 7 个测试目标（钉读序数的 4 个加第二轮修补带来的 3 个新用例）; 范围=7 个测试目标; 为什么=复跑辩方腿核逐盘核挪位对钉读序数的用例的影响与新用例在新树上的结局' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test writable_mount_acquisition_write_reads_the_system_configuration_slots_from_the_disk --test writable_mount_rehearsal_drops_bytes_failing_self_verification_from_the_shared_reads --test writable_mount_rehearsal_and_the_real_run_read_each_block_once --test writable_mount_reread_after_a_bad_read_reaches_the_disk_past_the_reads_shared_with_the_rehearsal --test writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device --test admission_floor_ceiling_entry_returns_no_valid_root_for_an_allocator_without_the_root_ring_table --test discard_after_the_frozen_publish_is_resent_reaches_the_segments_reclaimed_before_the_freeze) > "$WORK/out/new-pinned-and-new-tests.raw" 2>&1 || true
  compare_with_the_saved_output new-pinned-and-new-tests
  (cd "$WORK/old" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的 5 个测试目标（钉读序数的 4 个加 T4 用例）在还原成修补之前次序的树上; 范围=5 个测试目标; 为什么=复跑辩方腿看钉读序数的用例对逐盘核的次序是否敏感，以及 T4 新用例在旧次序下是否真红' CARGO_TARGET_DIR="$WORK/target-old" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --no-fail-fast --test writable_mount_acquisition_write_reads_the_system_configuration_slots_from_the_disk --test writable_mount_rehearsal_drops_bytes_failing_self_verification_from_the_shared_reads --test writable_mount_rehearsal_and_the_real_run_read_each_block_once --test writable_mount_reread_after_a_bad_read_reaches_the_disk_past_the_reads_shared_with_the_rehearsal --test writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device) > "$WORK/out/old-pinned-and-t4.raw" 2>&1 || true
  compare_with_the_saved_output old-pinned-and-t4
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot 整个目标; 范围=1 个测试目标; 为什么=复跑辩方腿核根记录改进 verus 块之后核心层写出的根字节被池级 checker 的独立解析器读回，以及 T1 变异的未变异对照' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot) > "$WORK/out/new-root-record-target.raw" 2>&1 || true
  compare_with_the_saved_output new-root-record-target
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-core 的 --lib 里 root_record 模块的单元测试; 范围=1 个测试目标; 为什么=复跑辩方腿核根记录 verus 块之后序列化往返的库内单测' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-core --lib root_record) > "$WORK/out/new-core-lib-root-record.raw" 2>&1 || true
  compare_with_the_saved_output new-core-lib-root-record

  # 两条变异：施加 → 跑那一条用例（应红）→ 反向还原
  (cd "$WORK/new" && patch -p1 < "$MODEL/patches/mutation-6144-tie-broken-backwards.patch" > "$WORK/out/mutation-6144-apply.txt" 2>&1)
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot 的 1 个测试 施加 mutations.tsv 第 6144 行; 范围=1 个测试; 为什么=复跑辩方腿 T1 的变异证红，钉值相同取槽最大时钉着平局规则的用例应当红' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot -- two_genesis_ring_slots_pinned_at_txg_zero_carry_the_lower_ring_slot_as_the_pinned_ring_slot --exact) > "$WORK/out/new-t1-mutation-6144.raw" 2>&1 || true
  compare_with_the_saved_output new-t1-mutation-6144
  (cd "$WORK/new" && patch -R -p1 < "$MODEL/patches/mutation-6144-tie-broken-backwards.patch" > "$WORK/out/mutation-6144-revert.txt" 2>&1)
  (cd "$WORK/new" && patch -p1 < "$MODEL/patches/mutation-6248-public-ceiling-entry-panics.patch" > "$WORK/out/mutation-6248-apply.txt" 2>&1)
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test admission_floor_ceiling_entry_returns_no_valid_root_for_an_allocator_without_the_root_ring_table 整个目标 施加 mutations.tsv 第 6248 行; 范围=1 个测试目标; 为什么=复跑辩方腿 T2 的变异证红，公开入口改回 panic 时第二轮补的用例应当红' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --no-fail-fast --test admission_floor_ceiling_entry_returns_no_valid_root_for_an_allocator_without_the_root_ring_table) > "$WORK/out/new-t2-mutation-6248.raw" 2>&1 || true
  compare_with_the_saved_output new-t2-mutation-6248
  (cd "$WORK/new" && patch -R -p1 < "$MODEL/patches/mutation-6248-public-ceiling-entry-panics.patch" > "$WORK/out/mutation-6248-revert.txt" 2>&1)
  (cd "$WORK/new" && patch -p1 < "$MODEL/patches/mutation-6155-ceiling-rereads-skipped.patch" > "$WORK/out/mutation-6155-apply.txt" 2>&1)
  (cd "$WORK/new" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test admission_floor_ceiling_with_every_root_ring_slot_unreadable_on_its_own_read 的 1 个测试 施加 mutations.tsv 第 6155 行; 范围=1 个测试; 为什么=复跑辩方腿 T2 的变异证红，断言挪走之后名字写红在已删断言的第 6155 行是否仍红' CARGO_TARGET_DIR="$WORK/target-new" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test admission_floor_ceiling_with_every_root_ring_slot_unreadable_on_its_own_read -- the_admission_ceiling_refuses_with_the_reread_refusal_when_every_root_ring_slot_reads_bad_only_on_its_own_read --exact) > "$WORK/out/new-t2-mutation-6155.raw" 2>&1 || true
  compare_with_the_saved_output new-t2-mutation-6155
  (cd "$WORK/new" && patch -R -p1 < "$MODEL/patches/mutation-6155-ceiling-rereads-skipped.patch" > "$WORK/out/mutation-6155-revert.txt" 2>&1)
  echo "name=mutations_reverted new_mount_sha256=$(sha256sum "$WORK/new/crates/singlefs-core/src/mount.rs" | cut -c1-16)"
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  export RERUN_TEMPLATE_SELFTEST_MARK="rerun-template-selftest"   # 自证里拷出去起的子进程（环境原样继承）靠它认：弄坏开关只在自证里认
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/scripts/hosts/run-with-memory-cap.sh"
  echo 冻结副本里的文件 > "$scratch/frozen/crates/marker.txt"
  echo 编译产物 > "$scratch/frozen/crates/target/junk"
  copy="$scratch/elsewhere/r1-opus-model/tools/rerun.sh"
  cp "${BASH_SOURCE[0]}" "$copy"

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1") \
      > "$scratch/run1.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 0 ]] || ! grep -qF "name=rerun_roots repo=$scratch/fake-repo frozen=$scratch/frozen model=$scratch/elsewhere/r1-opus-model " "$scratch/run1.out" \
      || [[ ! -f "$scratch/work1/tree/crates/marker.txt" || -e "$scratch/work1/tree/crates/target" ]]; then
    failures+=("拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 $rc：$(tail -n 4 "$scratch/run1.out")")
  fi

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy") > "$scratch/run2.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "用法：rerun.sh --repo" "$scratch/run2.out"; then
    failures+=("不传参应当退 2 并列出用法，实际退 $rc：$(tail -n 4 "$scratch/run2.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --work "$scratch/work3" > "$scratch/run3.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "缺 --frozen" "$scratch/run3.out"; then
    failures+=("缺 --frozen 应当退 2 并点名缺它，实际退 $rc：$(tail -n 4 "$scratch/run3.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/cwd" --frozen "$scratch/frozen" --work "$scratch/work4" > "$scratch/run4.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run4.out"; then
    failures+=("--repo 指到不是仓的目录应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run4.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1" > "$scratch/run5.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "已存在且非空" "$scratch/run5.out"; then
    failures+=("--work 指到已有输出的目录应当退 2、不在旧输出上接着写，实际退 $rc：$(tail -n 4 "$scratch/run5.out")")
  fi

  if ((${#failures[@]})); then
    local failure
    for failure in "${failures[@]}"; do
      echo "  ✗ 自检：$failure"   # gate-lint:detail
    done
    echo "    → 怎么办：看 parse_arguments() 与 rerun_steps()；RERUN_TEMPLATE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--work 非空都退 2 并说明"
  return 0
}

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（RERUN_TEMPLATE_SELFTEST_MARK 对得上）而变量非空，参数解析之前就退 2（不管值是什么）。
if [[ -n "${RERUN_TEMPLATE_BREAK:-}" && ! ( "$#" -eq 1 && "${1:-}" == --selftest ) && "${RERUN_TEMPLATE_SELFTEST_MARK:-}" != rerun-template-selftest ]]; then
  echo "  ✗ RERUN_TEMPLATE_BREAK=${RERUN_TEMPLATE_BREAK} 是弄坏开关，只在自证（bash <这份脚本> --selftest）里认，平常跑设了就拒，这一趟什么都没做" >&2
  echo "     → 怎么办：去掉 RERUN_TEMPLATE_BREAK 再跑；要证红就跑 RERUN_TEMPLATE_BREAK=<项> bash scripts/agents/three-way-rerun-template.sh --selftest" >&2
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
