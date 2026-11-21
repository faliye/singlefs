#!/usr/bin/env bash
# 合入后验证 merge-and-verify.sh 的函数库：补丁的核对、cargo 命令与 check.sh 的读法、测试目标的挑选、每一步的起法（run_step）、验证包那一步。
# 被 source，只定义函数、不改 shell 选项；读写入口脚本的全局变量（同一个 shell）。

# submit_command_text <带不带整包确认：1 或 0> <命令词…>：task-server/task submit 起这几个词的整条（词照 %q 写）；头一个参数是 1 的在 submit 与 -- 之间带
#   --whole-package-confirmed <整包确认文件>（文件头「只重跑的命令点名」：不点项的那几条）。弄坏开关 rerun-confirmation-dropped：一律不带
submit_command_text() {
  local confirmed="$1" text
  shift
  text="$(printf '%q ' "$@")"
  if [[ "$confirmed" == 1 && "${MERGE_AND_VERIFY_BREAK:-}" != "rerun-confirmation-dropped" ]]; then
    printf '%q submit --whole-package-confirmed %s -- %s' "$root/task-server/task" "$WHOLE_PACKAGE_CONFIRMATION_PLACEHOLDER" "${text% }"
  else
    printf '%q submit -- %s' "$root/task-server/task" "${text% }"
  fi
}

# current_tree_patch_way_out <日志目录> <补丁目录>…：--skip-apply 核不过时出路里「按此刻树现算补丁、再拿它 --skip-apply」那一句（文件头「--skip-apply 先核补丁打过没有」）。
# 打出两条整命令：先交 scripts/gate/patch-of-the-current-tree.py（照给的次序，--out 在这一趟的日志目录下，--tree 是要验的树），出了目录再拿它 --skip-apply 验：
#   后一条带原命令点的 --item（文件头「只重跑的命令点名」），原命令没点项的 submit 带 --whole-package-confirmed。弄坏开关 way-out-without-current-tree-patch：什么都不打。
current_tree_patch_way_out() {
  local out_directory="$1/current-tree-patch" make_text verify_text item range_text="1 份补丁" confirmed=1
  local -a verify_words=(bash "$root/scripts/gate/merge-and-verify.sh" --skip-apply "$out_directory")
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "way-out-without-current-tree-patch" ]] || return 0
  for item in ${NAMED_ITEMS[@]+"${NAMED_ITEMS[@]}"}; do verify_words+=(--item "$item"); done
  if ((${#NAMED_ITEMS[@]})); then range_text="${#NAMED_ITEMS[@]} 项"; confirmed=0; fi
  make_text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/patch-of-the-current-tree.py" --tree "$SOURCE_TREE" --out "$out_directory" "${@:2}")"
  verify_text="$(submit_command_text "$confirmed" "${verify_words[@]}")"
  printf "；补丁是三方合并打进去的、撤回了一半、或主工作区上另有别人的改动而倒不掉的，这两条都走不通：先 %s ，等它退 0、出了新补丁目录，再 SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证按此刻树现算的补丁目录; 范围=%s; 为什么=原补丁在此刻树上倒不掉，改验按此刻树现算的补丁' %s %s（原命令里别的参数照旧带上）" \
    "${make_text% }" "$range_text" "$verify_text" "$( ((confirmed)) && printf '%s' "$WHOLE_PACKAGE_CONFIRMATION_NOTE")"
}

# check_patches_applied <日志目录>：--skip-apply 时核给的每份 crates.patch 都已打在要验的树上（文件头「--skip-apply 先核补丁打过没有」）；核不过退 1、不编。
check_patches_applied() {
  local logs="$1" dir patch checked_count=0 unapplied_text=""
  local -a unapplied=() present=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-apply-unchecked" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-apply-unchecked：--skip-apply 时不核补丁打过没有"; return 0
  fi
  if ((${#CHECK_PATCH_DIRS[@]} == 0)); then echo "  - 补丁打没打本次未核：带了 --skip-apply 而没给补丁目录"; return 0; fi
  mkdir -p "$logs" || die "建不了日志目录 $logs" "看工作根在不在、可不可写" 1
  for dir in "${CHECK_PATCH_DIRS[@]}"; do
    if [[ ! -d "$dir" ]]; then unapplied+=("$dir（目录不在）"); continue; fi
    patch="$dir/crates.patch"
    [[ -s "$patch" ]] || continue
    present+=("$dir")
  done
  if ((${#unapplied[@]})); then
    unapplied_text="$(printf '%s、' "${unapplied[@]}")"
    die "带了 --skip-apply，给的补丁目录有 ${#unapplied[@]} 个不在，核不了打没打在要验的树 $tree 上：${unapplied_text%、}；一个都没编" \
      "补丁目录写对再来；没打过的那几份去掉 --skip-apply 让这里打（打过的与没打过的分两趟）" 1
  fi
  checked_count=${#present[@]}
  if ((checked_count == 0)); then echo "  - 补丁打没打本次未核：给的 ${#CHECK_PATCH_DIRS[@]} 个补丁目录里都没有 crates.patch"; return 0; fi
  # 弄坏开关 reverse-check-each-alone：在要验的树上逐份单独 git apply --check --reverse（同一批两份补丁改到同一行时前一份反着核不过，误判没打）
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "reverse-check-each-alone" ]]; then
    for dir in "${present[@]}"; do
      git -C "$tree" apply --check --reverse "$dir/crates.patch" > "$logs/reverse-check-$(basename "$dir").log" 2>&1 \
        || unapplied+=("$dir（日志 $logs/reverse-check-$(basename "$dir").log）")
    done
    if ((${#unapplied[@]})); then
      unapplied_text="$(printf '%s、' "${unapplied[@]}")"
      die "带了 --skip-apply，给的补丁有 ${#unapplied[@]} 份没打在要验的树 $tree 上（弄坏开关 reverse-check-each-alone：逐份单独反着核）：${unapplied_text%、}；一个都没编" \
        "没打过的那几份去掉 --skip-apply 让这里打" 1
    fi
    echo "  核过 $checked_count 份补丁都已打在树上（弄坏开关 reverse-check-each-alone：逐份单独 git apply --check --reverse）"
    return 0
  fi
  reverse_check_in_order "$logs" "${present[@]}"
  case $? in
    0) ;;
    1) die "带了 --skip-apply，照给的次序倒着逐份 git apply --reverse，倒到 $REVERSE_CHECK_FAILED 倒不掉：这一份没打在要验的树 $tree 上，或给的次序与打的次序不同（它前面还有 $REVERSE_CHECK_SKIPPED 份没往下核）；一个都没编" \
         "没打过的那几份去掉 --skip-apply 让这里打（打过的与没打过的分两趟）；都打过而次序给错了的，照打的次序重给补丁目录$(current_tree_patch_way_out "$logs" "${CHECK_PATCH_DIRS[@]}")" 1 ;;
    *) die "带了 --skip-apply，核补丁打没打用的临时副本没建成：$REVERSE_CHECK_FAILED；一个都没编" "看 ${TMPDIR:-/tmp} 在不在、可不可写、有没有空间，再重跑同一条" 1 ;;
  esac
  echo "  核过 $checked_count 份补丁都已打在树上（临时副本里照给的次序倒着逐份 git apply --reverse，全部倒得掉）"
}

# reverse_check_in_order <日志目录> <补丁目录>…：--skip-apply 核补丁打没打（文件头「--skip-apply 先核补丁打过没有」）。在 ${TMPDIR:-/tmp} 下新建一份临时副本
# （git init 的空仓，只拷要验的树里这几份 crates.patch 点到的文件），照给的次序倒着逐份 git apply --reverse：后打的先倒，同一批两份补丁改到同一行也倒得掉。
# 全部倒得掉退 0；有一份倒不掉退 1，那一份的补丁目录与日志进 REVERSE_CHECK_FAILED、它前面没往下核的份数进 REVERSE_CHECK_SKIPPED；副本建不成退 2（原因进 REVERSE_CHECK_FAILED）。
# 副本用完就删（三种结局都删）；REVERSE_CHECK_KEEP=1 的调用方（带 --skip-apply 那一趟的变异锚点核，skip_apply_anchor_check 要拿倒回原样的文件当补丁之前的树）在全部倒得掉（退 0）时
# 副本不删、路径进 REVERSE_CHECK_COPY，由调用方删；要验的树一个字节不动。
reverse_check_in_order() {
  local logs="$1" copy dir path index log copy_log="$1/reverse-check-copy.log"
  local -a patch_dirs=("${@:2}") touched=()
  REVERSE_CHECK_FAILED=""; REVERSE_CHECK_SKIPPED=0; REVERSE_CHECK_COPY=""
  if ! copy="$(mktemp -d "${TMPDIR:-/tmp}/merge-and-verify-reverse-check.XXXXXX")"; then
    REVERSE_CHECK_FAILED="${TMPDIR:-/tmp} 下建不了临时目录"; return 2
  fi
  # 补丁点到的路径（新旧两侧、Git 引号、重命名与拷贝）：与重编核对同一份读法（lib_rebuilt_packages.py 的 collect_paths），这里不另写
  if ! python3 - "$REBUILT" "${patch_dirs[@]}" > "$copy_log.paths" 2> "$copy_log" <<'PY_REVERSE_CHECK_PATHS'
import importlib.util, sys
library, *patch_directories = sys.argv[1:]
specification = importlib.util.spec_from_file_location("lib_rebuilt_packages_for_reverse_check", library)
rebuilt = importlib.util.module_from_spec(specification)
specification.loader.exec_module(rebuilt)
for path in rebuilt.collect_paths(patch_directories, []):
    print(path)
PY_REVERSE_CHECK_PATHS
  then
    REVERSE_CHECK_FAILED="读不出补丁点到的路径（日志 $copy_log）"; rm -rf -- "${copy:?}"; return 2
  fi
  mapfile -t touched < "$copy_log.paths"
  if ! git init -q "$copy" >> "$copy_log" 2>&1; then
    REVERSE_CHECK_FAILED="临时副本 $copy 里 git init 不成（日志 $copy_log）"; rm -rf -- "${copy:?}"; return 2
  fi
  for path in ${touched[@]+"${touched[@]}"}; do
    # 绝对路径与带 .. 的路径不拷（git apply 自己也拒它们，倒不掉照判）；树里没有的不拷（新建的文件倒着打要删它、不在就倒不掉）
    [[ "$path" != /* && "/$path/" != */../* ]] || continue
    [[ -e "$tree/$path" || -L "$tree/$path" ]] || continue
    # 不保留属性（--no-preserve=all）：有的临时目录不认保留权限，git apply 只看内容与可执行位，可执行位随建文件带过去
    local preserve="--no-preserve=all"; [[ "${MERGE_AND_VERIFY_BREAK:-}" == "reverse-check-copy-preserves" ]] && preserve="-p"
    if ! (cd "$tree" && cp --parents -P "$preserve" -- "$path" "$copy/") >> "$copy_log" 2>&1; then
      REVERSE_CHECK_FAILED="$path 拷不进临时副本 $copy（日志 $copy_log）"; rm -rf -- "${copy:?}"; return 2
    fi
  done
  for (( index = ${#patch_dirs[@]} - 1; index >= 0; index-- )); do
    dir="${patch_dirs[index]}"; log="$logs/reverse-check-$(basename "$dir").log"
    if ! git -C "$copy" apply --reverse "$dir/crates.patch" > "$log" 2>&1; then
      REVERSE_CHECK_FAILED="$dir（日志 $log）"; REVERSE_CHECK_SKIPPED=$index
      rm -rf -- "${copy:?}"; return 1
    fi
  done
  if [[ "${REVERSE_CHECK_KEEP:-0}" == 1 ]]; then REVERSE_CHECK_COPY="$copy"; else rm -rf -- "${copy:?}"; fi
  return 0
}

# clippy_step_name <特性组合>：这个组合那一步的步名（日志 <日志目录>/<步名>.log）：clippy- 接组合去掉开头的 -、别的非字母数字换成 -
clippy_step_name() {
  local text="$1"
  while [[ "$text" == -* ]]; do text="${text#-}"; done
  printf 'clippy-%s' "${text//[^A-Za-z0-9]/-}"
}

# read_cargo_command <check.sh> <子命令>：文件头「cargo 命令现读」。check.sh 里命令位置上写 cargo <子命令> 或 run_cargo <子命令>（前面可带 if）的行恰好一条，
# 子命令之后到第一个 |、;、\ 之前的词（2>&1 不算）进 CARGO_COMMAND_WORDS。找不到、不止一条、词里带 $、引号、括号这类这里不会展开的写法
# （clippy 那一行的两个占位词除外）返回 1，原因进 CHECK_SH_PROBLEM，原因里写这一条哪一行、哪个词、怎么办。
# 弄坏开关 cargo-wrapper-fixed：只认 run_cargo 那一种包装；cargo-wrapper-unrecognized：只认不带包装的 cargo。
read_cargo_command() {
  local file="$1" subcommand="$2" line line_number=0 hits=0 hit_text="" hit_number=0 wrapper_pattern='run_cargo|cargo' word line_re
  local -a words=()
  CARGO_COMMAND_WORDS=()
  case "${MERGE_AND_VERIFY_BREAK:-}" in
    cargo-wrapper-fixed) wrapper_pattern='run_cargo' ;;
    cargo-wrapper-unrecognized) wrapper_pattern='cargo' ;;
    *) ;;
  esac
  line_re='^[[:space:]]*(if[[:space:]]+)?('"$wrapper_pattern"')[[:space:]]+'"$subcommand"'([[:space:]]+([^|;\\]*))?([|;\\].*)?$'
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_number=$((line_number + 1))
    if [[ "$line" =~ $line_re ]]; then
      hits=$((hits + 1)); hit_number=$line_number; hit_text+="第 $line_number 行「${line#"${line%%[![:space:]]*}"}」；"
      read -r -a words <<< "${BASH_REMATCH[4]}"
    fi
  done < "$file"
  if ((hits == 0)); then
    CHECK_SH_PROBLEM="$file 里找不到 cargo $subcommand 那一条命令（命令位置上写 cargo $subcommand 或 run_cargo $subcommand 的行，数到 0 条）：上游把它换成了别的写法（另一个包装函数、存在变量里的命令名）；先在这份脚本的 read_cargo_command 里加这一种写法的识别，只加识别、不放宽核对"
    return 1
  fi
  if ((hits != 1)); then
    CHECK_SH_PROBLEM="$file 里 cargo $subcommand 命令写了 $hits 条（${hit_text%；}）：这里每一步只对一条，先决定上游多出来的那几条各对应这里的哪一步"
    return 1
  fi
  for word in ${words[@]+"${words[@]}"}; do
    [[ "$word" == '2>&1' ]] && continue
    if [[ "$subcommand" == clippy && ( "$word" == "$CLIPPY_FEATURES_PLACEHOLDER" || "$word" == "$CLIPPY_LINTS_PLACEHOLDER" ) ]]; then CARGO_COMMAND_WORDS+=("$word"); continue; fi
    case "$word" in
      *[\$\"\'\`\(\)\{\}\<\>\&\*\?\~]*)
        CHECK_SH_PROBLEM="$file 第 $hit_number 行的 cargo $subcommand 命令里有这里不会展开的词「$word」（带 \$、引号、括号这类）：先决定它在合入后验证里对应什么值，再在这份脚本的 read_cargo_command 里加这个词的处理"
        return 1 ;;
      *) ;;
    esac
    CARGO_COMMAND_WORDS+=("$word")
  done
  return 0
}

# read_check_sh <check.sh>：现读编码纪律那张 lint 表进 DISCIPLINE_LINTS、缺省的特性组合进 CLIPPY_COMBINATIONS、特性组合登记的相对路径进 CLIPPY_REGISTRATION，
# 现读 fmt、clippy、build、test 四条 cargo 命令的参数进 CARGO_FMT_WORDS、CARGO_CLIPPY_WORDS、CARGO_BUILD_WORDS、CARGO_TEST_FLAGS，
# 核 clippy 按组合循环的三行结构；读不出、对不上返回 1，原因进 CHECK_SH_PROBLEM。
read_check_sh() {
  local file="$1" line in_table=0 i command_text hits rest combination combination_lines step seen word narrowing features_count lints_count scope_count
  local combinations_line_re='^CLIPPY_FEATURE_COMBINATIONS=\((.*)\)[[:space:]]*$' quoted_word_re='^[[:space:]]*"([^"]*)"(.*)$'
  local -a words=() steps=()
  DISCIPLINE_LINTS=(); CLIPPY_COMBINATIONS=(); CLIPPY_REGISTRATION=""; CHECK_SH_PROBLEM=""
  CARGO_FMT_WORDS=(); CARGO_CLIPPY_WORDS=(); CARGO_BUILD_WORDS=(); CARGO_TEST_FLAGS=()
  [[ -f "$file" ]] || { CHECK_SH_PROBLEM="$file 不在"; return 1; }
  while IFS= read -r line || [[ -n "$line" ]]; do
    if ((in_table == 1)); then
      if [[ "$line" =~ ^\)[[:space:]]*$ ]]; then in_table=2; break; fi
      line="${line%%#*}"
      read -r -a words <<< "$line"
      DISCIPLINE_LINTS+=(${words[@]+"${words[@]}"})
    elif [[ "$line" =~ ^CODE_DISCIPLINE_LINTS=\([[:space:]]*$ ]]; then
      in_table=1
    fi
  done < "$file"
  if ((in_table != 2)); then CHECK_SH_PROBLEM="$file 里找不到 CODE_DISCIPLINE_LINTS=( … ) 那一段（起止各占一行）"; return 1; fi
  if ((${#DISCIPLINE_LINTS[@]} == 0 || ${#DISCIPLINE_LINTS[@]} % 2 != 0)); then
    CHECK_SH_PROBLEM="$file 的 CODE_DISCIPLINE_LINTS 读出 ${#DISCIPLINE_LINTS[@]} 个词，不是一条一对的「-D clippy::<名>」"; return 1
  fi
  for ((i = 0; i < ${#DISCIPLINE_LINTS[@]}; i += 2)); do
    if [[ ! "${DISCIPLINE_LINTS[i]}" =~ ^-[DWAF]$ || ! "${DISCIPLINE_LINTS[i+1]}" =~ ^clippy::[a-z0-9_]+$ ]]; then
      CHECK_SH_PROBLEM="$file 的 CODE_DISCIPLINE_LINTS 第 $((i / 2 + 1)) 条读成「${DISCIPLINE_LINTS[i]} ${DISCIPLINE_LINTS[i+1]}」，不是「-D clippy::<名>」的形状"; return 1
    fi
  done
  for command_text in "${CHECK_SH_STRUCTURE_LINES[@]}"; do
    hits="$(grep -cF -- "$command_text" "$file")"
    if [[ "$hits" != 1 ]]; then CHECK_SH_PROBLEM="$file 里「$command_text」出现 ${hits:-0} 次，不是 1 次：上游改了 clippy 按特性组合循环的写法，这一步要跟着改"; return 1; fi
  done
  # 四条 cargo 命令的参数现读（文件头「cargo 命令现读」）
  read_cargo_command "$file" fmt || return 1
  CARGO_FMT_WORDS=(${CARGO_COMMAND_WORDS[@]+"${CARGO_COMMAND_WORDS[@]}"})
  read_cargo_command "$file" build || return 1
  CARGO_BUILD_WORDS=(${CARGO_COMMAND_WORDS[@]+"${CARGO_COMMAND_WORDS[@]}"})
  read_cargo_command "$file" clippy || return 1
  CARGO_CLIPPY_WORDS=(${CARGO_COMMAND_WORDS[@]+"${CARGO_COMMAND_WORDS[@]}"})
  features_count=0; lints_count=0
  for word in ${CARGO_CLIPPY_WORDS[@]+"${CARGO_CLIPPY_WORDS[@]}"}; do
    [[ "$word" != "$CLIPPY_FEATURES_PLACEHOLDER" ]] || features_count=$((features_count + 1))
    [[ "$word" != "$CLIPPY_LINTS_PLACEHOLDER" ]] || lints_count=$((lints_count + 1))
  done
  if ((features_count != 1 || lints_count != 1)); then
    CHECK_SH_PROBLEM="$file 的 cargo clippy 命令里特性参数的占位词 $CLIPPY_FEATURES_PLACEHOLDER 出现 $features_count 次、编码纪律 lint 表的占位词 $CLIPPY_LINTS_PLACEHOLDER 出现 $lints_count 次，各要恰好 1 次：少了就是 clippy 没按特性组合跑、或没带 lint 表，与这里不是同一套"; return 1
  fi
  read_cargo_command "$file" test || return 1
  CARGO_TEST_FLAGS=(); scope_count=0
  for word in ${CARGO_COMMAND_WORDS[@]+"${CARGO_COMMAND_WORDS[@]}"}; do
    if [[ "$word" == --all || "$word" == --workspace ]]; then scope_count=$((scope_count + 1)); continue; fi
    for narrowing in "${TEST_NARROWING_FLAGS[@]}"; do
      if [[ "$word" == "$narrowing" ]]; then CHECK_SH_PROBLEM="$file 的 cargo test 命令带了收窄目标的标志 $word：这里的测试步自己按包、按测试目标挑，收窄了就不是同一批用例，先决定怎么对应"; return 1; fi
    done
    if [[ ! "$word" =~ ^--[a-z][a-z0-9-]*$ ]]; then
      CHECK_SH_PROBLEM="$file 的 cargo test 命令里有这里认不出的词「$word」（只认 --all 或 --workspace，加上不带值的 --标志）：先决定它在合入后验证里对应什么，再在这份脚本的 read_check_sh 里加"; return 1
    fi
    CARGO_TEST_FLAGS+=("$word")
  done
  if ((scope_count != 1)); then
    CHECK_SH_PROBLEM="$file 的 cargo test 命令里 --all 或 --workspace 出现 $scope_count 次，不是 1 次：不是整个工作区的测试，这里按「工作区成员减掉 验证包」挑包，对不上"; return 1
  fi
  # 缺省的特性组合（文件头 clippy 一段）：行首那一行 CLIPPY_FEATURE_COMBINATIONS=("…" …)，括号里只许双引号括起的词
  combination_lines="$(grep -c '^CLIPPY_FEATURE_COMBINATIONS=(' "$file")"
  if [[ "$combination_lines" != 1 ]]; then CHECK_SH_PROBLEM="$file 里行首的 CLIPPY_FEATURE_COMBINATIONS=( 出现 ${combination_lines:-0} 次，不是 1 次：上游改了缺省特性组合的写法"; return 1; fi
  line="$(grep '^CLIPPY_FEATURE_COMBINATIONS=(' "$file")"
  if [[ ! "$line" =~ $combinations_line_re ]]; then CHECK_SH_PROBLEM="$file 的缺省特性组合那一行认不出（要写在一行里、以右括号收尾）：$line"; return 1; fi
  rest="${BASH_REMATCH[1]}"
  while [[ "$rest" =~ $quoted_word_re ]]; do CLIPPY_COMBINATIONS+=("${BASH_REMATCH[1]}"); rest="${BASH_REMATCH[2]}"; done
  if [[ -n "${rest//[[:space:]]/}" ]]; then CHECK_SH_PROBLEM="$file 的缺省特性组合那一行有认不出的写法「${rest}」（每个组合用双引号括起）：$line"; return 1; fi
  for combination in ${CLIPPY_COMBINATIONS[@]+"${CLIPPY_COMBINATIONS[@]}"}; do
    if [[ "$combination" != default && "$combination" != -* ]]; then CHECK_SH_PROBLEM="$file 的缺省特性组合「$combination」认不出（只认 default 与以 - 起头的特性参数）"; return 1; fi
    step="$(clippy_step_name "$combination")"
    for seen in ${steps[@]+"${steps[@]}"}; do
      if [[ "$seen" == "$step" ]]; then CHECK_SH_PROBLEM="$file 的缺省特性组合里有两个组合的步名都是 $step（日志名会撞）"; return 1; fi
    done
    steps+=("$step")
  done
  # 弄坏开关 clippy-combinations-unchecked：不到两个也照收
  if ((${#CLIPPY_COMBINATIONS[@]} < 2)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "clippy-combinations-unchecked" ]]; then
    CHECK_SH_PROBLEM="$file 的缺省特性组合只读出 ${#CLIPPY_COMBINATIONS[@]} 个（${CLIPPY_COMBINATIONS[*]-}），不到两个：check.sh 退回了单趟 clippy"; return 1
  fi
  # 特性组合登记的相对路径：上游 lib.sh 的 CLIPPY_FEATURE_COMBINATIONS_FILE
  hits="$(grep -c '^CLIPPY_FEATURE_COMBINATIONS_FILE=' "$CHECK_SH_LIBRARY" 2>/dev/null)"
  CLIPPY_REGISTRATION="$(sed -n 's/^CLIPPY_FEATURE_COMBINATIONS_FILE=//p' "$CHECK_SH_LIBRARY" 2>/dev/null)"
  if [[ "$hits" != 1 || -z "$CLIPPY_REGISTRATION" ]]; then
    CHECK_SH_PROBLEM="$CHECK_SH_LIBRARY 里读不出唯一一行 CLIPPY_FEATURE_COMBINATIONS_FILE=<路径>（出现 ${hits:-0} 次）：算不出 check.sh 认的特性组合登记在哪"; return 1
  fi
  return 0
}

# verification_member_present <树>：工作区成员逐个写出目录、各有 [package] name，验证包在成员里（文件头 test 一段：认不出验证包就排不掉登记的崩溃枚举用例）；
#   不是返回 1，原因进 MEMBERS_PROBLEM。弄坏开关 no-verification-package-guard：成员里找不到验证包也照往下验
verification_member_present() {
  local listing
  MEMBERS_PROBLEM=""
  if ! listing="$(python3 - "$1" "$VERIFICATION_PACKAGE" "${MERGE_AND_VERIFY_BREAK:-}" <<'PY_MEMBERS' 2>&1
import os, sys, tomllib
tree, wanted, broken = sys.argv[1], sys.argv[2], sys.argv[3]
def fail(text):
    print(text); sys.exit(1)
try:
    with open(os.path.join(tree, "Cargo.toml"), "rb") as handle:
        members = tomllib.load(handle).get("workspace", {}).get("members")
except (OSError, tomllib.TOMLDecodeError) as error:
    fail(f"读不了 {tree}/Cargo.toml：{error}")
if not isinstance(members, list) or not members:
    fail(f"{tree}/Cargo.toml 没有 [workspace] members")
names = []
for member in members:
    if not isinstance(member, str) or any(mark in member for mark in "*?["):
        fail(f"成员「{member}」写成了通配或不是字符串，这里只认逐个写出的目录")
    try:
        with open(os.path.join(tree, member, "Cargo.toml"), "rb") as handle:
            name = tomllib.load(handle).get("package", {}).get("name")
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"读不了成员 {member} 的 Cargo.toml：{error}")
    if not isinstance(name, str) or not name:
        fail(f"成员 {member} 的 Cargo.toml 没有 [package] name")
    names.append(name)
if wanted not in names and broken != "no-verification-package-guard":
    fail(f"工作区成员（{'、'.join(names)}）里没有 {wanted}：验证包改了名或挪了，认不出它就排不掉登记的崩溃枚举用例")
PY_MEMBERS
  )"; then
    MEMBERS_PROBLEM="$listing"; return 1
  fi
  return 0
}

# select_test_targets <树> [<补丁动到的文件打补丁之前的原样目录>]：挑测试目标（文件头「挑测试目标」，判法是 MERGE_AND_VERIFY_ITEMS_PROGRAM 的 targets 一段）：挑出的项进 ALL_ITEM_LINES 与 ITEM_REASONS，
#   包目录下有测试属性的进 TESTED_PACKAGES，要特性的进 REQUIRED_FEATURE_TARGETS，判不出的进 TARGET_REFUSALS（带 --unresolved-to-commit-gate 又点了 --item 的真跑那一趟改进 LEFT_TO_COMMIT，留给提交时整轮门禁），报一句的进 TARGET_NOTES；读不了返回 1、原因进 TARGET_PROBLEM。
#   第二个参数（真跑那一趟给 <日志目录>/merge-baseline，经环境变量 MERGE_AND_VERIFY_BASELINE_ROOT 交给 lib_merge_verify_formal_tables.py）是必红世界表打补丁之前的原样，FORMAL_ITEMS 只收相对它新加或改过的行；不给的行都算新加
#   弄坏开关 whole-package-tests、fallback-runs-whole-workspace 在 targets 那一段里认
select_test_targets() {
  local listing kind first second third fourth
  ALL_ITEM_LINES=(); ITEM_REASONS=(); TESTED_PACKAGES=(); REQUIRED_FEATURE_TARGETS=(); TARGET_REFUSALS=(); LEFT_TO_COMMIT=(); TARGET_NOTES=(); FORMAL_ITEMS=(); TARGET_PROBLEM=""
  if ! listing="$(MERGE_AND_VERIFY_BASELINE_ROOT="${2:-}" python3 - targets "$1" "$RERUN_FAILED_TESTS" "$REBUILT" "${MERGE_AND_VERIFY_BREAK:-}" ${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"} \
       <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM" 2>&1)"; then
    TARGET_PROBLEM="$(tail -n 3 <<<"$listing" | tr '\n' ' ')"; return 1
  fi
  while IFS=$'\t' read -r kind first second third fourth; do
    case "$kind" in
      REFUSE) TARGET_REFUSALS+=("$first") ;;
      ITEM) ALL_ITEM_LINES+=("$first"$'\t'"$second"$'\t'"$third"); ITEM_REASONS+=("$first"$'\t'"$fourth") ;;
      TESTED) TESTED_PACKAGES+=("$first") ;;
      REQUIRES) REQUIRED_FEATURE_TARGETS+=("$first"$'\t'"$second"$'\t'"$third"$'\t'"$fourth") ;;
      NOTE) TARGET_NOTES+=("$first") ;;
      FORMAL) FORMAL_ITEMS+=("$first"$'\t'"$second") ;;
    esac
  done <<<"$listing"
  # 带了 --unresolved-to-commit-gate 又点了 --item 的真跑那一趟（unresolved_left_active）：判不出的几处移进 LEFT_TO_COMMIT、不挡测试那一步与验证包那一步，落得到的项照留
  if ((${#TARGET_REFUSALS[@]})) && unresolved_left_active; then
    LEFT_TO_COMMIT=("${TARGET_REFUSALS[@]}"); TARGET_REFUSALS=()
  fi
  # 有判不出的：真跑那一趟不跑测试那一步与验证包那一步，挑出的几样一概不留（verify 照 TARGET_REFUSALS 判）；只有 --list-items 留着落得到的照列（list_items_and_exit）。
  #   弄坏开关 list-items-refusal-drops-items 下列项时也清掉（一项都不列）
  if ((${#TARGET_REFUSALS[@]})) && { ((LIST_ITEMS == 0)) || [[ "${MERGE_AND_VERIFY_BREAK:-}" == list-items-refusal-drops-items ]]; }; then
    ALL_ITEM_LINES=(); ITEM_REASONS=(); TESTED_PACKAGES=(); REQUIRED_FEATURE_TARGETS=(); TARGET_NOTES=()
  fi
  return 0
}

# report_target_selection：挑出的项与为什么、报一句的、要特性的逐行打（文件头「挑测试目标」）；判不出的不在这里打（verify 里打红句与出路）
report_target_selection() {
  local line
  local -a fields=()
  ((${#TARGET_REFUSALS[@]})) && return 0
  echo "  挑测试目标：${#ALL_ITEM_LINES[@]} 项（补丁改到的与变异行点名的；不跑整个包，也不带上依赖它们的包）"
  for line in ${ITEM_REASONS[@]+"${ITEM_REASONS[@]}"}; do printf '      %s：%s\n' "${line%%$'\t'*}" "${line#*$'\t'}"; done   # gate-lint:detail
  for line in ${REQUIRED_FEATURE_TARGETS[@]+"${REQUIRED_FEATURE_TARGETS[@]}"}; do
    IFS=$'\t' read -r -a fields <<<"$line"
    printf '      %s 的 %s 要特性 %s（%s）：测试那一步不跑，产品 GPU 包的交验证包那一步\n' "${fields[0]}" "${fields[1]}" "${fields[2]}" "${fields[3]}"   # gate-lint:detail
  done
  for line in ${TARGET_NOTES[@]+"${TARGET_NOTES[@]}"}; do printf '      · %s\n' "$line"; done   # gate-lint:detail
  report_left_to_commit "  判不出改到哪个测试目标的 ${#LEFT_TO_COMMIT[@]} 处不跑、留给提交时的整轮门禁（带了 $UNRESOLVED_OPTION、点了 --item：测试那一步与验证包那一步只跑点名的项）："
  return 0
}

# report_left_to_commit <标题行>：留给提交时整轮门禁的几处（LEFT_TO_COMMIT，文件头「判不出的留给提交时整轮门禁」）先打标题行、再逐条一行「留给提交时整轮门禁：<原因>」（原因里点名文件）；没有的不打。
#   弄坏开关 unresolved-lines-dropped：不打
report_left_to_commit() {
  local word
  ((${#LEFT_TO_COMMIT[@]})) || return 0
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != unresolved-lines-dropped ]] || return 0
  echo "$1"
  for word in "${LEFT_TO_COMMIT[@]}"; do echo "      留给提交时整轮门禁：$word"; done   # gate-lint:detail
}

# anchor_check_new_breaks <日志目录> <快照> [<原样目录> <路径单子> <补丁目录>]：变异锚点核（文件头「先验后合」）：合进主工作区的那一份补丁目录（不带 --skip-apply 的那一趟是 <日志目录>/merge-patch）
#   算好之后，判这一批补丁合起来让哪些变异行新红了（别批变异行的锚点被补丁弄断：原文在文件里原来恰好命中 1 次、打完 0 次或多次；补丁之前就红的不算）。判法是 apply-writer-patch.py 的 static_check_report，这里不另写：
#   MERGE_ANCHOR_CHECK_PROGRAM 只摆出补丁之前的树、调它、打结果。后三个参数不给的取不带 --skip-apply 的那一趟的 merge-baseline、merge-baseline.paths、merge-patch；
#   带 --skip-apply 的那一趟由 skip_apply_anchor_check 交它倒出来的那三份。红或没做成返回 1，点名的行与出路在标准输出与 <日志目录>/anchor-check.log：日志里新红全数都有，
#   标准输出只列前 ANCHOR_CHECK_SHOWN_LINES 行并写明一共几处；补丁目录里什么都没有的不做；自证里只在带 ANCHOR_CHECK_SAMPLE_MARK 标记的样本树上做。
#   弄坏开关 anchor-check-skipped：一进来就放行；anchor-check-baseline-ignored：static_check_report 不拿补丁之前的树当基线（经 APPLY_WRITER_PATCH_BREAK=static-check-baseline-ignored）；
#   anchor-check-release-row-number-kept：RELEASE 行连行号一起比（经 APPLY_WRITER_PATCH_BREAK=static-check-release-row-number-kept）；
#   anchor-mutation-only-skipped：没有 crates.patch、只改变异表的补丁不做核；anchor-way-out-delete-dropped：出路不写锚着的代码被整段删掉的改写进 mutations-delete.txt；
#   anchor-way-out-keep-copy-dropped：没留快照的那一趟出路不写带 MERGE_AND_VERIFY_KEEP_COPY=1 重交
anchor_check_new_breaks() {
  local logs="$1" snapshot="$2" baseline="${3:-$1/$MERGE_BASELINE_NAME}" paths_file="${4:-$1/$MERGE_BASELINE_PATHS_NAME}" merge_patch="${5:-$1/$MERGE_PATCH_NAME}" rc total listed
  if inside_selftest && [[ ! -e "$snapshot/$ANCHOR_CHECK_SAMPLE_MARK" ]]; then return 0; fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "anchor-check-skipped" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=anchor-check-skipped：变异锚点核一进来就放行"; return 0
  fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "anchor-mutation-only-skipped" && ! -s "$merge_patch/crates.patch" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=anchor-mutation-only-skipped：没有 crates.patch、只改变异表的补丁不做变异锚点核"; return 0
  fi
  if [[ ! -s "$merge_patch/crates.patch" && ! -s "$merge_patch/mutations-append.tsv" && ! -s "$merge_patch/mutations-replacements.tsv" && ! -s "$merge_patch/mutations-delete.txt" ]]; then
    echo "  · 变异锚点核不做：这一批补丁合起来没有改动"; return 0
  fi
  echo "  · 变异锚点核开始（补丁之前与补丁之后两趟 static-check 先后跑：真仓上一趟约 400–740 秒、内存峰值约 15 GiB；日志 $logs/anchor-check.log）"
  (
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != "anchor-check-baseline-ignored" ]] || export APPLY_WRITER_PATCH_BREAK=static-check-baseline-ignored
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != "anchor-check-release-row-number-kept" ]] || export APPLY_WRITER_PATCH_BREAK=static-check-release-row-number-kept
    python3 - "$APPLY_WRITER_PATCH_LIBRARY" "$snapshot" "$baseline" "$paths_file" "$merge_patch" <<<"$MERGE_ANCHOR_CHECK_PROGRAM"
  ) > "$logs/anchor-check.log" 2>&1; rc=$?
  if ((rc == 0)); then
    echo "  ✓ 变异锚点核过：这一批补丁没有让变异行新红"
    sed 's/^/      /' "$logs/anchor-check.log"   # gate-lint:detail
    return 0
  fi
  echo "  ✗ 变异锚点核红（退 $rc；日志 $logs/anchor-check.log）：这一批补丁合起来让原来没红的变异行红了，或核没做成"
  sed 's/^/      /' "$logs/anchor-check.log" | head -n "$ANCHOR_CHECK_SHOWN_LINES"   # gate-lint:detail
  total="$(sed -n 's/^✗ 补丁打上之后变异表有 \([0-9][0-9]*\) 处.*/\1/p' "$logs/anchor-check.log" | head -n 1)"; listed="$(wc -l < "$logs/anchor-check.log")"
  if [[ -n "$total" ]]; then
    echo "  · 新红一共 $total 处，全部在 $logs/anchor-check.log$( ((listed > ANCHOR_CHECK_SHOWN_LINES)) && echo "（上面只列了日志 $listed 行里的前 $ANCHOR_CHECK_SHOWN_LINES 行）")"
  fi
  if ((skip_apply)); then
    echo "  → 怎么办：上面点名的行（ANCHOR：原文在文件里原来恰好命中 1 次、现在 0 次或多次）是这一批补丁新弄断的（补丁已在要验的树上，补丁之前的树是倒出来的，补丁之前就红的行不在里面）：以要验的树此刻的代码为准，"
    echo "     对每一行重取原文与替换文，直接改 crates/mutations.tsv 里那几行，锚着的代码整段没了、那一行已没有意义的删掉那一行，改完照原命令再验；上面写着没做成或判不了的，是核自己没跑成：照那几行与日志修好 static-check 读的变异表、登记表与拷贝范围，或补丁倒不回补丁之前的原因，再重验"
    return 1
  fi
  echo "  → 怎么办：上面点名的行（ANCHOR：原文在文件里原来恰好命中 1 次、现在 0 次或多次）是这一批补丁新弄断的，补丁之前就红的行不在里面：以这一趟留下的快照 $snapshot 为基线，对每一行照补丁之后的代码重取原文与替换文，"
  echo "     重出 mutations-replacements.tsv 里那几行（前六段是新行、后六段是快照里那一行的原样），接在这一批补丁目录之后整批再交；上面写着没做成或判不了的，是核自己没跑成：照那几行与日志修好 static-check 读的变异表、登记表与拷贝范围，再整批重交"
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "anchor-way-out-delete-dropped" ]]; then
    echo "     锚着的代码被这一批整段删掉、那一行已没有意义的：改写进 mutations-delete.txt（每行是快照里那一行的原样），同样接在这一批补丁目录之后整批再交"
  fi
  if [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" != 1 && "${MERGE_AND_VERIFY_BREAK:-}" != "anchor-way-out-keep-copy-dropped" ]]; then
    echo "     这一趟没留快照（没设 MERGE_AND_VERIFY_KEEP_COPY=1），上面的快照收尾已删：照原命令在 task-server/task submit -- 之后加 env MERGE_AND_VERIFY_KEEP_COPY=1 整条重交，拿重交那一趟留下的快照当基线重出那几行"
  fi
  return 1
}

# skip_apply_anchor_check <日志目录> <快照>：带 --skip-apply 的那一趟的变异锚点核（文件头「先验后合」变异锚点核）：补丁已在要验的树上，没有 merge-patch，这里照样核。
#   补丁之前的树由 skip_apply_anchor_baseline 倒出来（原样进 <日志目录>/anchor-skip-baseline，动到的路径进 anchor-skip-baseline.paths）；补丁之后是快照此刻，
#   合起来的补丁照「原样 → 快照此刻」由 MERGE_PATCH_PROGRAM 现算进 anchor-skip-patch，再交 anchor_check_new_breaks。补丁目录里什么都没有的不做。
#   倒不回补丁之前、合起来的补丁算不出，都记红、点名原因与日志 <日志目录>/anchor-skip-check.log。红或没做成返回 1。自证里只在带 ANCHOR_CHECK_SAMPLE_MARK 标记的样本树上做。
#   没给补丁目录的不核（没有补丁就摆不出补丁之前的树），打一行「变异锚点核本次未核」点名为什么，不记红。
#   弄坏开关 anchor-skip-apply-unchecked：一进来就放行；anchor-skip-no-directories-silent：没给补丁目录的那一趟不打「本次未核」那一行；
#   anchor-skip-none-reason-dropped：那一行只写「变异锚点核本次未核」、不写为什么；anchor-skip-none-records-red：打了那一行之后也记一项红；
#   anchor-skip-irreversible-passes：倒不回补丁之前、合起来的补丁算不出的照放行
skip_apply_anchor_check() {
  local logs="$1" snapshot="$2" log="$1/anchor-skip-check.log" baseline="$1/anchor-skip-baseline" paths_file="$1/anchor-skip-baseline.paths" patch_out="$1/anchor-skip-patch"
  local dir has_patch_files=0
  if inside_selftest && [[ ! -e "$snapshot/$ANCHOR_CHECK_SAMPLE_MARK" ]]; then return 0; fi
  if ((${#CHECK_PATCH_DIRS[@]} == 0)); then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "anchor-skip-none-reason-dropped" ]]; then
      echo "  · 变异锚点核本次未核"
    elif [[ "${MERGE_AND_VERIFY_BREAK:-}" != "anchor-skip-no-directories-silent" ]]; then
      echo "  · 变异锚点核本次未核：带 --skip-apply 没给补丁目录，不知道补丁改了哪些文件与变异行，摆不出补丁之前的树；要核就把补丁目录一起给上"
    fi
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != "anchor-skip-none-records-red" ]] || return 1
    return 0
  fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "anchor-skip-apply-unchecked" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=anchor-skip-apply-unchecked：带 --skip-apply 的那一趟不做变异锚点核"; return 0
  fi
  for dir in "${CHECK_PATCH_DIRS[@]}"; do
    [[ -s "$dir/crates.patch" || -s "$dir/mutations-append.tsv" || -s "$dir/mutations-replacements.tsv" || -s "$dir/mutations-delete.txt" ]] && has_patch_files=1
  done
  if ((has_patch_files == 0)); then
    echo "  · 变异锚点核不做：带 --skip-apply 给的补丁目录里没有 crates.patch，也没有变异表文件"; return 0
  fi
  rm -rf -- "${baseline:?}" "${paths_file:?}" "${patch_out:?}"; : > "$log"
  if ! skip_apply_anchor_baseline "$logs" "$snapshot" "$log" "$baseline" "$paths_file" \
     || ! python3 - "$REBUILT" "$snapshot" "$baseline" "$paths_file" "$patch_out" "${CHECK_PATCH_DIRS[@]}" <<<"$MERGE_PATCH_PROGRAM" >> "$log" 2>&1; then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "anchor-skip-irreversible-passes" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=anchor-skip-irreversible-passes：补丁之前的树倒不出来也放行"; return 0
    fi
    echo "  ✗ 变异锚点核没做成（${ANCHOR_SKIP_PROBLEM:-合起来的补丁算不出：$(tail -n 1 "$log")}；日志 $log）：补丁已在要验的树上，核要先倒出补丁之前的树"
    echo "  → 怎么办：照日志里的原因修；补丁之后又被别的改动动过、倒不回补丁之前的，去掉 --skip-apply 把补丁交给这里打（先验后合），或按要验的树此刻的样子重做补丁目录再验"
    return 1
  fi
  anchor_check_new_breaks "$logs" "$snapshot" "$baseline" "$paths_file" "$patch_out"
}

# skip_apply_anchor_baseline <日志目录> <快照> <日志> <原样目录> <路径单子>：带 --skip-apply 的那一趟摆出补丁之前的原样（文件头「先验后合」变异锚点核）。
#   crates.patch 点到的文件：reverse_check_in_order（check_patches_applied 核补丁打过没有的那一份倒打；REVERSE_CHECK_KEEP=1 留着倒完的临时副本）倒回原样，
#   MERGE_BASELINE_PROGRAM 把它们存进原样目录、路径单子写动到的路径（有变异表文件的另加 crates/mutations.tsv）；变异表照各份补丁目录里的变异表文件倒着摆回补丁之前
#   （MERGE_TABLE_REVERSE_PROGRAM，补丁目录给倒着的次序）。做成返回 0；倒不回返回 1，原因进 ANCHOR_SKIP_PROBLEM。临时副本用完就删。
#   弄坏开关 anchor-skip-table-not-reversed：变异表不倒回（补丁之前的表就是补丁之后的表）
skip_apply_anchor_baseline() {
  local logs="$1" snapshot="$2" log="$3" baseline="$4" paths_file="$5" dir index reversed_tree="" reverse_rc has_mutation_files=0
  local -a code_dirs=() reversed_dirs=()
  ANCHOR_SKIP_PROBLEM=""
  for dir in "${CHECK_PATCH_DIRS[@]}"; do
    [[ -s "$dir/crates.patch" ]] && code_dirs+=("$dir")
    [[ -s "$dir/mutations-append.tsv" || -s "$dir/mutations-replacements.tsv" || -s "$dir/mutations-delete.txt" ]] && has_mutation_files=1
  done
  if ((${#code_dirs[@]})); then
    REVERSE_CHECK_KEEP=1; reverse_check_in_order "$logs" "${code_dirs[@]}"; reverse_rc=$?; REVERSE_CHECK_KEEP=0
    if ((reverse_rc == 0)); then reversed_tree="$REVERSE_CHECK_COPY"; else ANCHOR_SKIP_PROBLEM="补丁倒不回补丁之前：$REVERSE_CHECK_FAILED"; fi
  elif ! reversed_tree="$(mktemp -d "${TMPDIR:-/tmp}/merge-and-verify-anchor-skip.XXXXXX")"; then
    ANCHOR_SKIP_PROBLEM="${TMPDIR:-/tmp} 下建不了临时目录"
  fi
  if [[ -z "$ANCHOR_SKIP_PROBLEM" ]] && ! python3 - "$REBUILT" "$reversed_tree" "$baseline" "$paths_file" "${CHECK_PATCH_DIRS[@]}" <<<"$MERGE_BASELINE_PROGRAM" >> "$log" 2>&1; then
    ANCHOR_SKIP_PROBLEM="存不成补丁之前的原样"
  fi
  if [[ -z "$ANCHOR_SKIP_PROBLEM" ]] && ((has_mutation_files)); then
    mkdir -p "$baseline/crates"
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "anchor-skip-table-not-reversed" ]]; then
      cp -- "$snapshot/crates/mutations.tsv" "$baseline/crates/mutations.tsv" >> "$log" 2>&1 || ANCHOR_SKIP_PROBLEM="变异表拷不出来"
    else
      for (( index = ${#CHECK_PATCH_DIRS[@]} - 1; index >= 0; index-- )); do reversed_dirs+=("${CHECK_PATCH_DIRS[index]}"); done
      python3 - "$snapshot/crates/mutations.tsv" "$baseline/crates/mutations.tsv" "${reversed_dirs[@]}" <<<"$MERGE_TABLE_REVERSE_PROGRAM" >> "$log" 2>&1 || ANCHOR_SKIP_PROBLEM="$(tail -n 1 "$log")"
    fi
  fi
  [[ -z "$reversed_tree" ]] || rm -rf -- "${reversed_tree:?}"
  [[ -z "$ANCHOR_SKIP_PROBLEM" ]]
}

run_step() {   # $1 日志文件，其余是命令：在这一整条里面直接起（不套内存包装与 capped.sh，上限与线程由起这一整条的服务定，文件头「上限、线程、机器」）
  local log="$1" rc
  shift
  # 进度行（文件头「进度行」）：命令跑着的时候每个间隔往标准输出打一行。弄坏开关 progress-lines-dropped：不起
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "progress-lines-dropped" ]]; then
    progress_ticker "$$" "$log" "$(basename -- "${log%.log}")" &
    PROGRESS_TICKER_PID=$!
  fi
  # 每一步的输出出两份日志（scripts/hosts/two-logs.sh --quiet：$log 是全量，同名的 .problems.log 只收错误与警告，行前带全量日志里的行号；不转打，汇总行点名它们）。
  # 弄坏开关 logs-single：只写一份全量日志
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "logs-single" ]]; then "$@" > "$log" 2>&1; rc=$?
  else bash "$TWO_LOGS" --quiet "${log%.log}" -- "$@"; rc=$?; fi
  stop_progress_ticker
  return "$rc"
}

# crash_expand_assignments：缩崩溃域交出去的「变量=值」填进 CRASH_EXPAND_ASSIGNMENTS（没给 CRASH_EXPAND_UP_TO 就是空）：放量与崩溃枚举用例的引擎读的那个
#   （CRASH_EXPAND_UP_TO_VARIABLE）与验证包里直接调判器的对拍读的那个（CRASH_JUDGE_EXPAND_UP_TO_VARIABLE）两个都在、同一个值；交给门禁、验证包目标的 cargo test、
#   算键的请求与只重跑的命令的四处都经它取，值的来历（参数、环境里已设的、估算回落）不影响。弄坏开关 judge-expand-variable-not-passed：只交引擎读的那一个
crash_expand_assignments() {
  CRASH_EXPAND_ASSIGNMENTS=()
  [[ -n "$CRASH_EXPAND_UP_TO" ]] || return 0
  CRASH_EXPAND_ASSIGNMENTS=("$CRASH_EXPAND_UP_TO_VARIABLE=$CRASH_EXPAND_UP_TO")
  [[ "${MERGE_AND_VERIFY_BREAK:-}" == judge-expand-variable-not-passed ]] || CRASH_EXPAND_ASSIGNMENTS+=("$CRASH_JUDGE_EXPAND_UP_TO_VARIABLE=$CRASH_EXPAND_UP_TO")
}

# verification_rerun_command <交给门禁的参数…>：只重跑验证包那一步红的那几项的整条命令（文件头「验证包那一步」；起主工作区那一份门禁、交给它的根是 VERIFICATION_RERUN_ROOT）：
#   范围声明写在 task-server/task 前面（种类 items，范围数是 --check 点名的格数加 --item 的项数，同 .claude/hooks/run-scope-guard.sh 文件头「门禁阶段」），
#   命令经 task-server/task submit -- 交（不套包装、不写上限与线程，文件头「上限、线程、机器」）；缩过域的那一趟带同一个展开深度的两个变量（crash_expand_assignments；只给带 --skip-apply 的那一趟用，env 接在 submit -- 之后：变量不在服务的白名单里，写在命令里原样跑；不带 --skip-apply 的合入后验证写 --crash-expand-up-to，merge_rerun_command）。
#   不带 --item 而点了产品 GPU 包的真卡对拍那一格的（那一格不收点名）submit 带 --whole-package-confirmed、命令后面接那一句出路（文件头「只重跑的命令点名」）。
#   弄坏开关 rerun-command-unscaled：缩过域的那一趟也不带变量；rerun-command-wrapped：套 capped.sh 与内存包装、不经 task-server；rerun-whole-cells：打整格（三格、不带 --item）
verification_rerun_command() {
  local word cells="" previous="" cell_count item_count text note="" confirmed=0 why="合入后验证这几项红了，修完只重跑它们：键相同的照复用，只现判红的那几条与改后键变了的"
  local -a arguments=("$@") scale_words=() items=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "rerun-whole-cells" ]]; then arguments=(--check "$WHOLE_VERIFICATION_CELLS"); fi
  for word in "${arguments[@]}"; do
    [[ "$previous" == --check ]] && cells="$word"
    [[ "$previous" == --item ]] && items+=("$word")
    previous="$word"
  done
  crash_expand_assignments
  if [[ -n "$CRASH_EXPAND_UP_TO" && "${MERGE_AND_VERIFY_BREAK:-}" != "rerun-command-unscaled" ]]; then
    scale_words=(env "${CRASH_EXPAND_ASSIGNMENTS[@]}")
    why+="（缩过域：展开深度上限 $CRASH_EXPAND_UP_TO，不算门禁通过）"
  fi
  cell_count="$(tr ',' '\n' <<< "$cells" | grep -c .)"; item_count=${#items[@]}
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "rerun-command-wrapped" ]]; then
    text="$(printf '%q ' bash "$root/scripts/hosts/capped.sh" "$THREADS" bash "$root/scripts/hosts/run-with-memory-cap.sh" 36G bash "$VERIFICATION_RERUN_STAGE" "${arguments[@]}" "$VERIFICATION_RERUN_ROOT")"
  else
    if ((item_count == 0)) && [[ ",$cells," == *",$PRODUCT_GPU_CELL,"* ]]; then confirmed=1; note="$WHOLE_PACKAGE_CONFIRMATION_NOTE"; fi
    text="$(submit_command_text "$confirmed" ${scale_words[@]+"${scale_words[@]}"} bash "$VERIFICATION_RERUN_STAGE" "${arguments[@]}" "$VERIFICATION_RERUN_ROOT")"
  fi
  printf "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crash-point-replay.sh 的 %s 格%s; 范围=%s 项（%s 格加点名的 %s 项）; 为什么=%s' %s%s" \
    "${cells//,/、}" "$( ((item_count)) && printf '点名的 %s' "${items[*]}")" "$((cell_count + item_count))" "$cell_count" "$item_count" "$why" "${text% }" "$note"
}

# verification_red_test_names <日志>：那一道日志里红的测试函数名，逐行、去重。认两种写法：cargo 的「test <名> ... FAILED」，
# 与那一道逐个红的测试目标抄下的「FAILED <名>」（汇总行「FAILED. N passed」不算）；只认前一种会把几条红数成 1 条
# 弄坏开关 verification-step-failed-lines-ignored：只认前一种
verification_red_test_names() {
  {
    sed -n 's/^ *test \(.*\) \.\.\. FAILED$/\1/p' "$1"
    [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-step-failed-lines-ignored" ]] || sed -n 's/^ *FAILED \([^ .][^ ]*\)$/\1/p' "$1"
  } | LC_ALL=C sort -u
}

# verification_routes <门禁 --list-items 的日志，没有写 ->：这一趟跑到的验证包目标里要另走门禁的几样（文件头「验证包那一步」）：
#   崩溃枚举用例进 VERIFICATION_CASE_ITEMS（crash-case:<用例名>），挂特性的进 VERIFICATION_FEATURE_ITEMS（feature-gated:<文件名>、feature-gated:lib）；
#   产品 GPU 包要特性的目标置 VERIFICATION_PRODUCT_GPU，别的包要特性的目标进 VERIFICATION_UNROUTED。
#   弄坏开关 feature-gated-unrouted：挂特性的一样都不交；product-gpu-unrouted：产品 GPU 包的也当别的包
verification_routes() {
  local listing="$1" line item rest package selector key value target product_gpu_package=""
  local -a targets=() fields=()
  VERIFICATION_CASE_ITEMS=(); VERIFICATION_FEATURE_ITEMS=(); VERIFICATION_PRODUCT_GPU=0; VERIFICATION_UNROUTED=(); VERIFICATION_PRODUCT_GPU_WHY=""
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    item="${line%%$'\t'*}"; rest="${line#*$'\t'}"; package="${rest%%$'\t'*}"; selector="${rest#*$'\t'}"
    [[ "$package" == "$VERIFICATION_PACKAGE" ]] && targets+=("$selector")
  done
  if [[ "$listing" != - ]]; then
    while IFS=$'\t' read -r key value; do
      case "$key" in
        crash-case:*)
          target="${value#*:}"; target="${target%%:*}"
          if word_in_list "--test $target" ${targets[@]+"${targets[@]}"} || { [[ "$target" == lib ]] && word_in_list --lib ${targets[@]+"${targets[@]}"}; }; then
            VERIFICATION_CASE_ITEMS+=("$key")
          fi ;;
        feature-gated:lib) word_in_list --lib ${targets[@]+"${targets[@]}"} && VERIFICATION_FEATURE_ITEMS+=("$key") ;;
        feature-gated:*.rs) target="${key#feature-gated:}"; target="${target%.rs}"; [[ "${MERGE_AND_VERIFY_BREAK:-}" == feature-gated-directory-target-unrouted ]] || target="${target%/main}"
                            word_in_list "--test $target" ${targets[@]+"${targets[@]}"} && VERIFICATION_FEATURE_ITEMS+=("$key") ;;
      esac
    done < "$listing"
  fi
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != feature-gated-unrouted ]] || VERIFICATION_FEATURE_ITEMS=()
  ((${#REQUIRED_FEATURE_TARGETS[@]})) || return 0
  product_gpu_package="$(sed -n 's/^PRODUCT_GPU_PACKAGE_NAME="\([^"$`]*\)".*/\1/p' "$VERIFICATION_STAGE" 2>/dev/null | head -n 1)"
  for line in "${REQUIRED_FEATURE_TARGETS[@]}"; do
    IFS=$'\t' read -r -a fields <<<"$line"
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" != product-gpu-unrouted ]] && { [[ -z "$product_gpu_package" ]] || [[ "${fields[0]}" == "$product_gpu_package" ]]; }; then
      VERIFICATION_PRODUCT_GPU=1
      VERIFICATION_PRODUCT_GPU_WHY="${fields[0]} 的 ${fields[1]}（要特性 ${fields[2]}；${fields[3]}）$([[ -z "$product_gpu_package" ]] && echo "；$(basename "$VERIFICATION_STAGE") 里读不出 PRODUCT_GPU_PACKAGE_NAME=\"<包名>\" 那一行，照起")"
    else
      VERIFICATION_UNROUTED+=("${fields[0]} 的 ${fields[1]}（要特性 ${fields[2]}）")
    fi
  done
  return 0
}

# verification_call <说明> <过了要读到的行：blocks 或 functions> <日志> <交给门禁的参数…>：验证包那一步起门禁的一次（文件头「验证包那一步」），
#   根是 VERIFICATION_CALL_ROOT。过交 0（现判、复用、拒算加进 VERIFICATION_JUDGED、VERIFICATION_REUSED、VERIFICATION_REFUSED，崩溃状态块那一行进 VERIFICATION_BLOCKS），
#   本次未跑交 77，红交 1（打红的测试函数、红的项与只重跑红的那几项的命令）。
#   弄坏开关 verification-step-sample-exit-anywhere：退 3 不看根是不是样本根一律算过；verification-step-red-ignored：红了也报过；
#   card-handed-to-the-gate、start-card-stripped 弄坏交给门禁的卡（文件头「显卡」）
verification_call() {
  local label="$1" expect="$2" log="$3" rc counts judged reused refused passes block_text seen=0 name word previous=""
  shift 3
  local -a arguments=("$@") red_tests=() red_items=() named=() rerun_arguments=()
  echo "  验证包那一步（$label）：起 $(basename "$VERIFICATION_STAGE") ${arguments[*]}，只现判键对不上的、键相同的复用${CRASH_EXPAND_UP_TO:+；展开深度上限缩到 $CRASH_EXPAND_UP_TO（缩过域、不算门禁通过）}…"
  rm -f -- "$log"
  # 卡（文件头「显卡」）：环境里的 GPU_CARDS_BY_PRIORITY 照原样交给门禁，上卡的那几段归门禁照它文件头「显卡」办；这里不领、不挑、不设、不 -u。
  #   弄坏开关的 -u 写在「变量=值」之前（env 见到头一个「变量=值」之后就不再认选项）
  local -a card_environment=()
  case "${MERGE_AND_VERIFY_BREAK:-}" in
    start-card-stripped) card_environment=(-u GPU_CARDS_BY_PRIORITY) ;;
    card-handed-to-the-gate) [[ -n "${GPU_CARDS_BY_PRIORITY:-}" ]] || card_environment=("GPU_CARDS_BY_PRIORITY=card-handed-to-the-gate:0:8G") ;;
  esac
  run_step "$log" env -u MERGE_AND_VERIFY_BREAK ${card_environment[@]+"${card_environment[@]}"} ${VERIFICATION_SCALE_WORDS[@]+"${VERIFICATION_SCALE_WORDS[@]}"} \
    bash "$VERIFICATION_STAGE" "${arguments[@]}" "$VERIFICATION_CALL_ROOT"; rc=$?
  if ((rc == 77)); then
    VERIFICATION_NOT_RUN+=("$label")
    echo "  - 验证包那一步（$label）本次未跑：$(basename "$VERIFICATION_STAGE") 退 77（$(grep -E '^ *!' "$log" | tail -n 1 | sed 's/^ *//')），不记红、不算过"
    return 77
  fi
  counts="$(sed -nE 's/^ *· (feature-gated-tests |product-gpu-real-card-tests )?按条复用：.*现判 ([0-9]*)、复用 ([0-9]*)、拒算 ([0-9]*).*/\2 \3 \4/p' "$log")"
  read -r judged reused refused passes < <(awk 'NF { judged += $1; reused += $2; refused += $3; passes++ } END { print judged + 0, reused + 0, refused + 0, passes + 0 }' <<< "$counts")
  block_text="$(sed -nE 's/^ *✓ 崩溃状态块：现判 ([0-9]+)、复用 ([0-9]+)、拒算 ([0-9]+).*/现判 \1、复用 \2、拒算 \3/p' "$log" | head -n 1)"
  if [[ "$expect" == blocks ]]; then [[ -n "$block_text" ]] && seen=1; else ((passes > 0)) && seen=1; fi
  if { { ((rc == 0)) || { ((rc == 3)) && { [[ -d "$VERIFICATION_CALL_ROOT/.crash-point-replay-sample-tools" ]] || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-step-sample-exit-anywhere" ]]; }; }; } && ((seen)); } \
     || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-step-red-ignored" ]]; then
    VERIFICATION_JUDGED=$((VERIFICATION_JUDGED + judged)); VERIFICATION_REUSED=$((VERIFICATION_REUSED + reused)); VERIFICATION_REFUSED=$((VERIFICATION_REFUSED + refused))
    [[ -z "$block_text" ]] || VERIFICATION_BLOCKS="$block_text"
    echo "  ✓ 验证包那一步（$label）过（$(basename "$VERIFICATION_STAGE") 退 $rc$( ((rc == 3)) && echo '，样本档：假 cargo 走的流程')；$( [[ "$expect" == blocks ]] && echo "崩溃状态块 ${block_text:-读不到}" || echo "按条现判 $judged、复用 $reused、拒算 $refused，读到 $passes 趟按条复用的那一行")；日志 $log）"
    grep -m 1 '^ *· 逐格：' "$log" | sed 's/^ */    /'
    return 0
  fi
  mapfile -t red_tests < <(verification_red_test_names "$log")
  mapfile -t red_items < <(sed -nE 's/^ITEM_DONE item=([^ ]+) .* exit=1$/\1/p' "$log" | LC_ALL=C sort -u)
  for word in "${arguments[@]}"; do [[ "$previous" == --item ]] && named+=("$word"); previous="$word"; done
  # 只重跑红的：ITEM_DONE 里 exit=1 的项落在这一次点的项里就只点它们，认不出的点这一次的全部项（文件头「验证包那一步」）
  previous=""
  for word in "${arguments[@]}"; do
    if [[ "$word" == --item ]]; then previous=--item; continue; fi
    if [[ "$previous" == --item ]]; then previous=""; continue; fi
    rerun_arguments+=("$word")
  done
  local -a rerun_items=()
  for name in ${named[@]+"${named[@]}"}; do
    if ((${#red_items[@]})) && word_in_list "$name" "${red_items[@]}"; then rerun_items+=("$name"); fi
  done
  # 红的项一项都不在这一次点的项里（或没有 exit=1 的 ITEM_DONE）：点这一次的全部项（文件头「只重跑的命令点名」）。弄坏开关 verification-rerun-items-dropped：红的项认不出时一项都不点
  if ((${#rerun_items[@]} == 0)) && { ((${#red_items[@]} == 0)) || [[ "${MERGE_AND_VERIFY_BREAK:-}" != "verification-rerun-items-dropped" ]]; }; then
    rerun_items=(${named[@]+"${named[@]}"})
  fi
  for name in ${rerun_items[@]+"${rerun_items[@]}"}; do rerun_arguments+=(--item "$name"); done
  echo "  ✗ 验证包那一步（$label）红（$(basename "$VERIFICATION_STAGE") 退 $rc$( ((rc == 3)) && echo '：退 3 只在样本根上算过，交给它的根不是样本根')$( ((seen == 0)) && echo "，日志里读不到$([[ "$expect" == blocks ]] && echo '崩溃状态块' || echo '按条复用')那一行（现判、复用多少判不了）")；红的项：${red_items[*]:-认不出}；日志 $log、错误警告 ${log%.log}.problems.log）：FAILED 的测试函数 ${#red_tests[@]} 条"
  for name in ${red_tests[@]+"${red_tests[@]}"}; do echo "      $name"; done   # gate-lint:detail
  grep -E '^ *红的 |^ *error: test failed, to rerun pass' "$log" | sed 's/^ *//' | LC_ALL=C sort -u | head -n 40 | sed 's/^/      /'   # gate-lint:detail
  ((${#red_tests[@]})) || grep -E '^ *✗' "$log" | head -n 5 | sed 's/^ */      /'   # gate-lint:detail
  # 不带 --skip-apply 的那一趟（文件头「先验后合」）：补丁不在主工作区上，只重跑的是合入后验证。弄坏开关 verification-rerun-on-main-tree：照带 --skip-apply 的那一种打
  if ((skip_apply)) || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-rerun-on-main-tree" ]]; then
    echo "  → 怎么办：FAILED 的那几条测试函数并进写段的下一批派 implementation-writer 改；改完只重跑红的：$(verification_rerun_command "${rerun_arguments[@]}")（键相同的照复用：判绿记过库的不重跑，只现判上面红的这几条与改后键变了的）"
  else
    echo "  → 怎么办：FAILED 的那几条测试函数派 implementation-writer 改，副本从这一趟留下的快照建、修补以它为基线写（没留快照的照原命令带 env MERGE_AND_VERIFY_KEEP_COPY=1 重交一趟）；修补的补丁目录接在这一条的补丁目录之后，只重跑红的：$(verification_merge_rerun_command "${arguments[@]}")（补丁不在主工作区上，照这一趟的形态在快照上打、验；键相同的照复用：判绿记过库的不重跑，只现判上面红的这几条与改后键变了的）"
  fi
  return 1
}

# crash_domain_settings：崩溃域上限（文件头「崩溃域上限」）要的四样，一行一样打出来：放量家目录（取不到打 /nonexistent：运行记录读不到，要靠记录估的算不出）、
#   GPU 开关（0 / 1）、bindgen 参数、多机配置的路径（与入口 scripts/crash/crash-point-replay.sh 同一套读法）。放量那份库读不了就一行不打
crash_domain_settings() (
  source "$AMPLIFICATION_LIBRARY" > /dev/null 2>&1 || exit 1
  local configuration home gpu=0
  configuration="$(crash_amplification_multi_host_configuration_file "$VERIFICATION_CALL_ROOT")"
  home="$(crash_amplification_home_directory)" || home=/nonexistent
  if [[ "$(crash_amplification_configuration_value "$configuration" ENABLE_GPU)" == 1 ]]; then gpu=1; fi
  printf '%s\n' "$home" "$gpu" "$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS" "$configuration"
)

# verification_crash_domain <日志目录>：崩溃域上限（文件头「崩溃域上限」）。交 0 接着起（算得出 n 的填这个 n，估不出或算不出 n 的填 CRASH_DOMAIN_FALLBACK_DEPTH，都填进 CRASH_EXPAND_UP_TO），1 估算那一步没成（已打）。
#   读库程序的环境同入口的 test_functions_environment。弄坏开关 crash-domain-unbounded：不估；crash-domain-under-limit-narrowed：不超上限也缩到 1；
#   crash-domain-narrow-ignored：算得出 n 也不缩；crash-domain-narrow-note-dropped：缩了不打说明；crash-domain-known-depth-replaced：算得出 n 也取小值；
#   crash-domain-unknown-passed：估不出的也放行；crash-domain-fallback-note-dropped：取小值不打说明；crash-domain-fallback-depth-wrong：小值多取一层；crash-domain-program-failure-ignored：估算那一步没成也放行
verification_crash_domain() {
  local logs="$1" log="$1/verification-crash-domain.log" decision_file="$1/verification-crash-domain.decision" rc decision="" head="" variable
  local -a settings=() environment=(env -u MERGE_AND_VERIFY_BREAK -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE -u GPU_CARDS_BY_PRIORITY)
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "crash-domain-unbounded" ]] || return 0
  echo "  验证包那一步（崩溃域上限）：没给 --crash-expand-up-to，估这一趟点名的 ${#VERIFICATION_CASE_ITEMS[@]} 条崩溃枚举用例要现判多少崩溃状态…"
  mapfile -t settings < <(crash_domain_settings)
  rm -f -- "$log" "$decision_file"
  if ((${#settings[@]} == 4)); then
    for variable in $(compgen -e); do [[ "$variable" == SINGLEFS_CRASH_AMPLIFICATION_* ]] && environment+=(-u "$variable"); done
    # -u 一律写在「变量=值」之前：env 见到头一个「变量=值」之后就不再认选项
    environment+=("BINDGEN_EXTRA_CLANG_ARGS=${settings[2]}")
    [[ ! -f "${settings[3]}" ]] || environment+=("SINGLEFS_MULTI_HOST_CONFIG=$(cd "$(dirname "${settings[3]}")" && pwd)/$(basename "${settings[3]}")")
    run_step "$log" "${environment[@]}" python3 -c "$MERGE_AND_VERIFY_CRASH_DOMAIN_PROGRAM" "${MERGE_AND_VERIFY_CRASH_DOMAIN_LIBRARY:-$VERIFICATION_CALL_ROOT/scripts/crash/lib_crash_point_replay_items.py}" \
      "$VERIFICATION_CALL_ROOT" "${settings[0]}/runs" "${settings[1]}" "${MERGE_AND_VERIFY_BREAK:-}" "$decision_file" "${VERIFICATION_CASE_ITEMS[@]}"; rc=$?
  else
    rc=127
    echo "  ! 读不了放量那份库 $AMPLIFICATION_LIBRARY，估不了崩溃域" >> "$log"
  fi
  if ((rc == 0)) && [[ -f "$decision_file" ]]; then
    { IFS= read -r decision; IFS= read -r head; } < "$decision_file"
  fi
  case "$decision" in
    pass)
      echo "  · $head"
      tail -n +3 -- "$decision_file"   # gate-lint:detail
      if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "crash-domain-under-limit-narrowed" ]]; then CRASH_EXPAND_UP_TO=1; fi
      return 0 ;;
    "narrow "*)
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == "crash-domain-narrow-note-dropped" ]] || echo "  · $head"
      tail -n +3 -- "$decision_file"   # gate-lint:detail
      if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "crash-domain-known-depth-replaced" ]]; then CRASH_EXPAND_UP_TO="$CRASH_DOMAIN_FALLBACK_DEPTH"
      elif [[ "${MERGE_AND_VERIFY_BREAK:-}" != "crash-domain-narrow-ignored" ]]; then CRASH_EXPAND_UP_TO="${decision#narrow }"; fi
      return 0 ;;
    fallback)
      [[ "${MERGE_AND_VERIFY_BREAK:-}" != "crash-domain-unknown-passed" ]] || return 0
      CRASH_EXPAND_UP_TO="$CRASH_DOMAIN_FALLBACK_DEPTH"
      [[ "${MERGE_AND_VERIFY_BREAK:-}" != "crash-domain-fallback-depth-wrong" ]] || CRASH_EXPAND_UP_TO=$((CRASH_DOMAIN_FALLBACK_DEPTH + 1))
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == "crash-domain-fallback-note-dropped" ]] || echo "  · $head；展开深度上限取 $CRASH_EXPAND_UP_TO 接着跑（这个值是估不出时合入后验证固定缩到的深度，等同给 --crash-expand-up-to $CRASH_DOMAIN_FALLBACK_DEPTH；缩过域、不算门禁通过；要全量给不小于引擎展开上限常量的值，或留给提交时不指定范围的那一趟；日志 $log）"
      tail -n +3 -- "$decision_file"   # gate-lint:detail
      return 0 ;;
  esac
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "crash-domain-program-failure-ignored" ]] || return 0
  echo "  ✗ 验证包那一步没起：估崩溃域的那一步退 $rc、没交出判定（日志 $log、错误警告 ${log%.log}.problems.log）"
  echo "  → 怎么办：照那份日志修（scripts/crash/lib_crash_point_replay_items.py 与放量那份库读不读得了），再重跑同一条命令；等不及修就带 --crash-expand-up-to <n> 再交（给了值不估）"
  return 1
}

# verification_step <日志目录>：验证包那一步（文件头「验证包那一步」）。过与本次未跑交 0、红交 1；汇总行里的一句进 VERIFICATION_STATE，缩过域的进 VERIFICATION_SCALED。
#   弄坏开关 verification-step-skipped：不跑；whole-verification-cells：有要交的就起那一道的三格整格；crash-cases-whole-cell：崩溃枚举用例那一次不带 --crash-cases-only；
#   expand-up-to-not-passed：缩过域也不把变量交给那一道；scaled-run-unlabelled：缩过域的那一趟汇总里不写；
#   crash-domain-given-value-overridden：给了值也估
verification_step() {
  local logs="$1" listing="-" rc reds=0 item verification_targets=0 line joined
  local -a case_arguments=() feature_arguments=() parts=()
  VERIFICATION_CALL_ROOT="${MERGE_AND_VERIFY_VERIFICATION_ROOT:-$tree}"; VERIFICATION_RERUN_ROOT="${MERGE_AND_VERIFY_VERIFICATION_ROOT:-$SOURCE_TREE}"
  VERIFICATION_JUDGED=0; VERIFICATION_REUSED=0; VERIFICATION_REFUSED=0; VERIFICATION_BLOCKS=""; VERIFICATION_NOT_RUN=(); VERIFICATION_SCALE_WORDS=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-step-skipped" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=verification-step-skipped：不跑 验证包那一步"; return 0
  fi
  if ((${#TARGET_REFUSALS[@]})); then VERIFICATION_STATE="没跑（判不出测试目标，同测试那一步）"; return 0; fi
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    line="${line#*$'\t'}"; [[ "${line%%$'\t'*}" == "$VERIFICATION_PACKAGE" ]] && verification_targets=$((verification_targets + 1))
  done
  # 登记的崩溃枚举用例与挂特性的测试文件照那一道的 --list-items 现读（不起 cargo）
  if ((verification_targets)); then
    listing="$logs/verification-list-items.log"
    run_step "$listing" env -u MERGE_AND_VERIFY_BREAK bash "$VERIFICATION_STAGE" --list-items "$VERIFICATION_CALL_ROOT"; rc=$?
    if ((rc != 0)); then
      VERIFICATION_STATE="红（$(basename "$VERIFICATION_STAGE") --list-items 退 $rc）"
      echo "  ✗ 验证包那一步：$(basename "$VERIFICATION_STAGE") --list-items 退 $rc（日志 $listing、错误警告 ${listing%.log}.problems.log），读不出跑到的 $verification_targets 个验证包目标里登记的崩溃枚举用例与挂特性的测试文件"
      echo "  → 怎么办：照那份日志修（那一道的登记表、tests/ 里挂特性的文件读不读得了），再重跑同一条命令"
      return 1
    fi
  fi
  if item_mode && ((${#REQUIRED_FEATURE_TARGETS[@]})); then
    VERIFICATION_NOT_RUN+=("产品 GPU 包的真卡对拍（点了 --item）")
  fi
  verification_routes "$listing"
  ! item_mode || VERIFICATION_PRODUCT_GPU=0
  if ((${#VERIFICATION_CASE_ITEMS[@]} + ${#VERIFICATION_FEATURE_ITEMS[@]} + VERIFICATION_PRODUCT_GPU == 0)); then
    VERIFICATION_STATE="本次未跑（跑到的目标里没有登记的崩溃枚举用例与挂特性的测试文件，也没有产品 GPU 包的真卡对拍）"
    echo "  - 验证包那一步本次未跑：这一趟跑到的 $verification_targets 个验证包目标里没有登记的崩溃枚举用例与挂特性的测试文件，补丁也没改到产品 GPU 包的真卡对拍（不记红、不算过）${CRASH_EXPAND_UP_TO:+；给的展开深度上限 $CRASH_EXPAND_UP_TO 没交给门禁}${VERIFICATION_UNROUTED[*]:+；要特性、本次未跑的：${VERIFICATION_UNROUTED[*]}}"
    return 0
  fi
  for item in ${VERIFICATION_CASE_ITEMS[@]+"${VERIFICATION_CASE_ITEMS[@]}"}; do case_arguments+=(--item "$item"); done
  for item in ${VERIFICATION_FEATURE_ITEMS[@]+"${VERIFICATION_FEATURE_ITEMS[@]}"}; do feature_arguments+=(--item "$item"); done
  # 崩溃域上限（文件头「崩溃域上限」）：有崩溃枚举用例要交、又没给值才估；缩到多少填进 CRASH_EXPAND_UP_TO，下面那几次照带
  if ((${#case_arguments[@]})) && { [[ -z "$CRASH_EXPAND_UP_TO" ]] || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "crash-domain-given-value-overridden" ]]; }; then
    verification_crash_domain "$logs"; rc=$?
    if ((rc != 0)); then VERIFICATION_STATE="红（估崩溃域的那一步没成）"; return 1; fi
  fi
  crash_expand_assignments
  if [[ -n "$CRASH_EXPAND_UP_TO" && "${MERGE_AND_VERIFY_BREAK:-}" != "expand-up-to-not-passed" ]]; then
    VERIFICATION_SCALE_WORDS=("${CRASH_EXPAND_ASSIGNMENTS[@]}")
  fi
  # 上卡的那几段归门禁照它文件头「显卡」办（这一份文件头「显卡」），这里照次序起、不领卡
  local -a case_options=("$CRASH_CASES_ONLY_OPTION")
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "crash-cases-whole-cell" ]] || case_options=()
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "whole-verification-cells" ]]; then
    verification_call "三格整格（弄坏开关 whole-verification-cells）" functions "$logs/verification-step.log" --check "$WHOLE_VERIFICATION_CELLS"
    (($? == 1)) && reds=$((reds + 1))
  else
    if ((${#case_arguments[@]})); then
      verification_call "登记的崩溃枚举用例" blocks "$logs/verification-crash-cases.log" --check "$CRASH_CASE_CELL" ${case_options[@]+"${case_options[@]}"} "${case_arguments[@]}"
      (($? == 1)) && reds=$((reds + 1))
    fi
    if ((${#feature_arguments[@]})); then
      verification_call "挂特性的测试文件" functions "$logs/verification-feature-gated.log" --check "$FEATURE_GATED_CELL" "${feature_arguments[@]}"
      (($? == 1)) && reds=$((reds + 1))
    fi
    if ((VERIFICATION_PRODUCT_GPU)); then
      echo "  · 产品 GPU 包的真卡对拍：$VERIFICATION_PRODUCT_GPU_WHY"
      verification_call "产品 GPU 包的真卡对拍" functions "$logs/verification-product-gpu.log" --check "$PRODUCT_GPU_CELL"
      (($? == 1)) && reds=$((reds + 1))
    fi
  fi
  # 缩过域的那一趟汇总里写明（弄坏开关 scaled-run-unlabelled：不写）
  if [[ -n "$CRASH_EXPAND_UP_TO" && "${MERGE_AND_VERIFY_BREAK:-}" != "scaled-run-unlabelled" ]]; then VERIFICATION_SCALED=1; fi
  ((${#VERIFICATION_UNROUTED[@]})) && VERIFICATION_NOT_RUN+=("要特性的 ${VERIFICATION_UNROUTED[*]}")
  parts=("按条现判 $VERIFICATION_JUDGED、复用 $VERIFICATION_REUSED")
  [[ -z "$VERIFICATION_BLOCKS" ]] || parts+=("崩溃状态块 $VERIFICATION_BLOCKS")
  ((${#VERIFICATION_NOT_RUN[@]})) && parts+=("本次未跑：${VERIFICATION_NOT_RUN[*]}")
  joined="$(printf '%s；' "${parts[@]}")"; joined="${joined%；}"
  if ((reds)); then VERIFICATION_STATE="红（$reds 次门禁判红；$joined）"; return 1; fi
  VERIFICATION_STATE="过（$joined）"
  return 0
}
