#!/usr/bin/env bash
# admission: always 被 scripts/experiment-apparatus/replay.sh 在同目录 source 进去的那一段，判的是此刻的仓与这一趟的参数，上一次的结论不替这一次作保
# run-condition: none 不单独执行：由 replay.sh 的 source 载入，环境要求同 replay.sh（文件头「GPU」「编译目录」）
# replay-inputs-and-build.sh：replay.sh 的一段（拷副本、驱动要读的输入、编研究二进制、GPU 开关与特性集、复跑表一行的准入键）。只定义函数与载入时赋值的全局量，不改 shell 选项。
# 这一段按 replay.sh 里 source 它的那一行的位置载入。文件头的用法、复跑表、弄坏开关表、退出码都在 replay.sh。
# clip_text <字节数>：标准输入每行取前 <字节数> 个字节，再丢掉被切在中间的半个字符，输出总是合法的 UTF-8（GNU 的 cut -c 与 cut -b 都按字节切，三个字节的汉字会被切断；给人看的失败行与日志摘录用它截）
clip_text() { cut -b "1-$1" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null; }

# 照写死的清单拷 crates/ 去编译的副本（只剩 replay_copy_closure_into 的弄坏开关那一支）都带上每台默认重活线程数的登记文件
# .claude/heavy-thread-default：harness 包经 include_str! 在编译时读它（crates/singlefs-verification/src/heavy_threads.rs），副本里没有就编不过。
# 拷 crates 的那一行在同一行里接着调它（replay_selftest 的 copy-lines-with-registry 那一格静态核）。在仓根调（脚本开头已 cd 到仓根）。
# 弄坏开关 REPLAY_BREAK=thread-registry-not-copied：什么都不拷。
copy_thread_registry_into() { # <副本目录>
  [[ "$REPLAY_BROKEN" == thread-registry-not-copied ]] && return 0
  mkdir -p "$1/.claude" && cp -- .claude/heavy-thread-default "$1/.claude/heavy-thread-default"
}

# 拷今天这棵树去编译的副本（e168_patched_tree 的副本、driver_e171_s4_comparator 的冻结树）：范围取 scripts/hosts/lib_repository_copy.py 的共用清单与闭包
# （path 依赖的 prune-server/、编译期读的 .claude/heavy-thread-default、运行期起的脚本都在里面），经 scripts/hosts/minimal-copy.py 拷，这里不写清单；
# 它打的两行进 stderr（驱动的 stdout 是产物）。在仓根调。副本里起命令的，多机配置照 replay_configuration_for_the_copy 交。
# 弄坏开关 REPLAY_BREAK=copy-list-fixed：照写死的那几样拷（共用清单里的四样加登记文件，没有 prune-server/）。
replay_copy_closure_into() { # <副本目录>
  if [[ "$REPLAY_BROKEN" == copy-list-fixed ]]; then
    rsync -a --exclude target Cargo.toml Cargo.lock .cargo crates "$1/" && copy_thread_registry_into "$1"
    return
  fi
  python3 scripts/hosts/minimal-copy.py . "$1" >&2
}

# 副本里起命令用的多机配置（scripts/hosts/lib_repository_copy.py configuration-path：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它，没设取主工作区那一份）：
# 副本是新 init 的仓、不带 multi-host.env，判法照副本的位置找不到配置。打一行绝对路径。在仓根调。弄坏开关 REPLAY_BREAK=copy-configuration-unset：什么都不打（副本里不交）。
replay_configuration_for_the_copy() {
  [[ "$REPLAY_BROKEN" == copy-configuration-unset ]] && return 0
  python3 scripts/hosts/lib_repository_copy.py configuration-path .
}

# 取回归档的留存产物：research/results/ 底下的留存产物按「每次提交删上一次的实验记录」归档进了版本库，不在树里。git log --all --diff-filter=D 找最近一次删它的提交，
# git show <提交>^:research/results/<名> 读回删之前那一版。成功往标准输出打提交的前 12 位、内容写进 <取回到的文件>；找不到提交、读不出、读出来是空的返回 1。
# 弄坏开关 REPLAY_BREAK=archived-restore-off：一律取不回（改成不取的样子）。
replay_archived_product_copy() { # <research/results 底下的产物名> <取回到的文件>
  local name="$1" target="$2" commit
  [[ "$REPLAY_BROKEN" == archived-restore-off ]] && return 1
  commit="$(git log --all --diff-filter=D --format=%H -1 -- "research/results/$name" 2>/dev/null)" && [[ -n "$commit" ]] || return 1
  git show "$commit^:research/results/$name" >"$target" 2>/dev/null && [[ -s "$target" ]] || return 1
  printf '%s\n' "${commit:0:12}"
}
# 驱动开跑前核它要读的输入。research/results/ 底下缺的先从 git 历史取回、放回原处（记进 $OUT_DIR/restored-archived-products.list，replay_finish 收尾删）；
# 取不回、或不在 research/results/ 底下（冻结根、导出根）的缺了，每缺一样往 stderr 打一行「✗ 输入不在 test=<这一行> …」，返回 1，这一行记跑不了；都在返回 0。
# 驱动写成 replay_inputs_present <这一行> <缺了怎么办> <路径…> || return $?，放在编与跑之前。
# 弄坏开关 REPLAY_BREAK=inputs-not-checked：不核、照起（缺了在装置或比对器里跑不了，或静默少判）。
replay_inputs_present() { # <这一行> <缺了怎么办> <路径…>
  local row="$1" remedy="$2" path missing=0 commit temporary
  shift 2
  [[ "$REPLAY_BROKEN" == inputs-not-checked ]] && return 0
  for path in "$@"; do
    [[ -n "$path" && -e "$path" ]] && continue
    if [[ "$path" == research/results/* ]]; then
      temporary="$path.restoring.$BASHPID"
      if commit="$(replay_archived_product_copy "${path#research/results/}" "$temporary")" && mv -n -- "$temporary" "$path" && [[ ! -e "$temporary" ]]; then
        printf '%s\n' "$path" >>"${OUT_DIR:?}/restored-archived-products.list"
        printf '  · %s 要读的 %s 不在树里，从 git 历史取回（git show %s^:%s），本趟收尾删\n' "$row" "$path" "$commit" "$path" >&2
        continue
      fi
      rm -f -- "$temporary"
      [[ -e "$path" ]] && continue
    fi
    printf '  ✗ 输入不在 test=%s %s 不在（树里没有、git 历史里也取不回），这一行记跑不了；%s\n' "$row" "${path:-（没给路径）}" "$remedy" >&2
    missing=$((missing + 1))
  done
  (( missing == 0 ))
}
# 冻结根与导出根的固定位置：工作根（.claude/hooks/lib_work_root.py）下的 replay-frozen-roots/<名>。工作根在多机配置的共享根下，两台读同一个绝对路径；
# 根的重建办法在各驱动的出路里。取不到工作根返回 1。
replay_frozen_root() { # <名>
  local work_root
  work_root="$(python3 "$REPLAY_REPOSITORY_ROOT/.claude/hooks/lib_work_root.py" 2>/dev/null)" && [[ -n "$work_root" ]] || return 1
  printf '%s/replay-frozen-roots/%s\n' "$work_root" "$1"
}
# 取回那一句的出路（树里与 git 历史里都没有时给人看）
REPLAY_ARCHIVED_REMEDY='先从 git 历史取回：git log --all --diff-filter=D --name-only -- "*<文件名>" 找到删它的提交，git show <提交>^:<路径> 放回原处；历史里也没有就要重录这份产物'
# 装置 main 与单测按字面 concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/<名>") 读别的实验的留存产物（scripts/experiment-apparatus/lib.rs 的 ARCHIVED_PRODUCT_NOT_RUN_MARKER 那一段）：
# 有行要现跑时，把这些名字里不在树里的从 git 历史取回放回原处（replay_inputs_present，本趟收尾删）。取不回的这里不判：读它的装置自己退 77，那一行记跑不了。
replay_restore_referenced_products() {
  local name
  while IFS= read -r name; do
    [[ -n "$name" && ! -e "research/results/$name" ]] || continue
    replay_inputs_present "装置读的留存产物" "" "research/results/$name" 2>/dev/null || true
  done < <(grep -rhoE 'CARGO_MANIFEST_DIR"\), *"/\.\./\.\./research/results/[A-Za-z0-9_.-]+"' scripts/experiment-apparatus --include='*.rs' 2>/dev/null | sed -E 's#.*/research/results/##; s#"$##' | sort -u)
}

# 开跑之前编一次 scripts/experiment-apparatus/ 的装置（在 scripts/experiment-apparatus/ 里跑）：输出落进 $1（构建日志，以 .log 结尾；经 scripts/hosts/two-logs.sh 出两份：
# $1 是全量，同名的 .problems.log 只收 error、warning: 与它们的位置行、非 0 退出码，行前带全量日志里的行号）。失败时打第一处 error 那一行、日志末尾 20 行、
# 日志路径、错误警告日志路径与一条单独重现的命令，返回 2；成功什么都不打。弄坏开关 REPLAY_BREAK=build-log-single：只写一份全量日志。
build_research_binaries() { # <构建日志> [只编这几个 bin…]（不给就整编 research 工作区）
  local log="$1" build_exit bin_options=() name
  shift
  for name in "$@"; do bin_options+=(--bin "$name"); done
  if [[ "$REPLAY_BROKEN" == build-output-dropped ]]; then   # 弄坏开关：输出丢进 /dev/null、红了只说一句
    cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml >/dev/null 2>&1 || { echo "replay: 构建失败" >&2; return 2; }
    return 0
  fi
  if [[ "$REPLAY_BROKEN" == build-log-single ]]; then
    if cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml ${bin_options[@]+"${bin_options[@]}"} >"$log" 2>&1; then return 0; else build_exit=$?; fi
  elif bash scripts/hosts/two-logs.sh --quiet "${log%.log}" -- cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml ${bin_options[@]+"${bin_options[@]}"}; then
    return 0
  else
    build_exit=$?
  fi
  echo "replay: 构建失败（cargo build 退 $build_exit），一条实验都没跑" >&2
  grep -m1 -E '^error(\[|:)' -- "$log" | sed 's/^/  第一处错：/' >&2
  echo "  构建日志末尾 20 行：" >&2
  tail -n 20 -- "$log" | sed 's/^/    /' >&2
  echo "  整份构建日志：$log" >&2
  echo "  错误警告日志：${log%.log}.problems.log（只收 error、warning: 与位置行，行前带全量日志里的行号，先看它）" >&2
  echo "  → 怎么办：照日志里第一处 error 改；单独重现：(cd $PWD && cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml${bin_options[*]:+ ${bin_options[*]}})" >&2
  return 2
}


# replay_row_driver_leases_its_own_card <复跑表第 2 列>：这一行的驱动在上面那张登记里返回 0。弄坏开关 REPLAY_BREAK=driver-leased-rows-held-by-replay 下一律返回 1。
replay_row_driver_leases_its_own_card() {
  [[ "$REPLAY_BROKEN" != driver-leased-rows-held-by-replay && "$1" == @* && " ${REPLAY_DRIVERS_LEASING_THEIR_OWN_CARDS[*]} " == *" ${1#@} "* ]]
}

# 配置开了 GPU 时，起 实验包装置（cargo run … -p singlefs-implementation-experiments --bin <装置>）带 --features gpu 编（文件头「GPU」）：
# 结果放进全局数组 CHECKER_TIER_FEATURES（空数组就是不带），不经命令替换带出来。多机配置怎么找、键怎么读只在
# scripts/crash/crash-amplification-lib.sh 一处（crash_amplification_multi_host_configuration_file、crash_amplification_configuration_value），这里 source 它不抄第二份。
# 返回 0：配置没开 GPU（文件不在、没写 ENABLE_GPU、写 0），或开了而 gpu 特性编得过（cargo build --release -p singlefs-verification --features gpu --lib，输出落进 $1）。
# 返回 78（打 ✗ 与 → 出路）：配置读不出（SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在、文件不可读、读的时候出错、ENABLE_GPU 写了 0、1 之外的值），
# 或开了 GPU 而 gpu 特性编不过。不悄悄退回不带特性：那样装置会把判段当没编进 gpu 特性、报本次未跑或 panic，复跑结论不是配置要的那一份。
# 弄坏开关 REPLAY_BREAK=gpu-features-dropped：读配置、编也编，但开了 GPU 也不带特性。
resolve_checker_tier_features() { # <gpu 编译日志>：先读开关、开着再试编（两步分开的写法见 read_gpu_switch、prepare_checker_tier_features）
  read_gpu_switch "$1" || return $?
  prepare_checker_tier_features "$1"
}

# 读多机配置的 ENABLE_GPU：结果放进全局 REPLAY_GPU_SWITCH（0 或 1）与 REPLAY_GPU_CONFIGURATION_FILE，不编。配置读不出返回 78（打 ✗ 与 → 出路）。
read_gpu_switch() { # <gpu 编译日志>（读配置出错时把那几行落在 <日志>.configuration）
  local log="$1" configuration_file switch="" reading_exit
  REPLAY_GPU_SWITCH=0
  REPLAY_GPU_CONFIGURATION_FILE=""
  CHECKER_TIER_FEATURES=()
  # 只拷了这一份脚本的树（自证造的临时仓）里没有那份库：那里也没有多机配置，ENABLE_GPU 算关，不读库
  if [[ ! -r scripts/crash/crash-amplification-lib.sh && -z "${SINGLEFS_MULTI_HOST_CONFIG:-}" && ! -e multi-host.env ]]; then
    return 0
  fi
  # shellcheck source=/dev/null
  if ! source scripts/crash/crash-amplification-lib.sh; then
    echo "  ✗ replay: 读不了 scripts/crash/crash-amplification-lib.sh（读多机配置的共用库），不知道配置开没开 GPU，一条实验都没跑" >&2
    echo "  → 怎么办：从仓根起（bash scripts/experiment-apparatus/replay.sh），或补回那份库" >&2
    return 78
  fi
  configuration_file="$(crash_amplification_multi_host_configuration_file "$PWD")"
  if [[ ! -e "$configuration_file" ]]; then
    if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
      echo "  ✗ replay: SINGLEFS_MULTI_HOST_CONFIG 点名的多机配置 $configuration_file 不在，读不出 ENABLE_GPU，一条实验都没跑" >&2
      echo "  → 怎么办：把 SINGLEFS_MULTI_HOST_CONFIG 指到在的文件，或不设它（取主工作树根的 multi-host.env；那份不在就是两个开关都关）" >&2
      return 78
    fi
    return 0   # 没有多机配置：ENABLE_GPU 算关（crash-amplification-lib.sh 文件头「配置」）
  fi
  if [[ ! -r "$configuration_file" ]]; then
    echo "  ✗ replay: 多机配置 $configuration_file 不可读，读不出 ENABLE_GPU，一条实验都没跑" >&2
    echo "  → 怎么办：修好文件权限（ls -l $configuration_file），再跑" >&2
    return 78
  fi
  if switch="$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU 2>"$log.configuration")"; then reading_exit=0; else reading_exit=$?; fi
  if (( reading_exit != 0 )); then
    echo "  ✗ replay: 读多机配置 $configuration_file 的 ENABLE_GPU 出错（退 $reading_exit）：$(head -n 3 -- "$log.configuration" 2>/dev/null | tr "\n" " ")，一条实验都没跑" >&2
    echo "  → 怎么办：看那份配置是不是 UTF-8 文本；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_configuration_value \"\$1\" ENABLE_GPU' x $configuration_file" >&2
    return 78
  fi
  case "$switch" in
    ""|0) return 0 ;;
    1) REPLAY_GPU_SWITCH=1; REPLAY_GPU_CONFIGURATION_FILE="$configuration_file"; return 0 ;;
    *)
      echo "  ✗ replay: 多机配置 $configuration_file 的 ENABLE_GPU 写成了「$switch」，只认 0、1，读不出开没开 GPU，一条实验都没跑" >&2
      echo "  → 怎么办：改成 ENABLE_GPU=1 或 ENABLE_GPU=0（写法见仓根 multi-host.env.example）" >&2
      return 78 ;;
  esac
}

# 开了 GPU（read_gpu_switch 读出 1）时试编 gpu 特性，设 CHECKER_TIER_FEATURES；没开一个字都不变。编不过返回 78。
prepare_checker_tier_features() { # <gpu 编译日志>
  local log="$1" configuration_file="${REPLAY_GPU_CONFIGURATION_FILE:-}" build_exit
  CHECKER_TIER_FEATURES=()
  [[ "${REPLAY_GPU_SWITCH:-0}" == 1 ]] || return 0
  if [[ "$REPLAY_BROKEN" == build-log-single ]]; then   # 弄坏开关：只写一份全量日志
    if cargo build --release -p singlefs-verification --features gpu --lib >"$log" 2>&1; then build_exit=0; else build_exit=$?; fi
  elif bash scripts/hosts/two-logs.sh --quiet "${log%.log}" -- cargo build --release -p singlefs-verification --features gpu --lib; then
    build_exit=0
  else
    build_exit=$?
  fi
  if (( build_exit == 0 )); then
    [[ "$REPLAY_BROKEN" == gpu-features-dropped ]] || CHECKER_TIER_FEATURES=(--features gpu)
    return 0
  fi
  echo "  ✗ replay: 多机配置 $configuration_file 开了 GPU（ENABLE_GPU=1），验证包带 --features gpu 编不过（cargo build 退 $build_exit），一条实验都没跑；不退回不带 gpu 特性去跑" >&2
  grep -m1 -E '^error(\[|:)' -- "$log" | sed 's/^/  第一处错：/' >&2
  echo "  编译日志末尾 20 行：" >&2
  tail -n 20 -- "$log" | sed 's/^/    /' >&2
  echo "  整份编译日志：$log" >&2
  echo "  错误警告日志：${log%.log}.problems.log（只收 error、warning: 与位置行，行前带全量日志里的行号，先看它）" >&2
  echo "  → 怎么办：照日志里第一处 error 修好 gpu 特性的编译；单独重现：(cd $PWD && cargo build --release -p singlefs-verification --features gpu --lib)；确实不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
  return 78
}

# 复跑表这一行问准入用哪个键：admission.py replay-row-keys 给的（按前三列 实验号|二进制|参数 查 ADMISSION_ROW_KEY），结果放进 ROW_ADMISSION_KEY
# （不经命令替换带出来）。弄坏开关 REPLAY_BREAK=row-key-by-experiment 下一律用实验号。
admission_key_of_row() {
  if [[ "$REPLAY_BROKEN" == row-key-by-experiment ]]; then
    ROW_ADMISSION_KEY="$1"
  else
    ROW_ADMISSION_KEY="${ADMISSION_ROW_KEY["$1|$2|$3"]-$1}"
  fi
}

