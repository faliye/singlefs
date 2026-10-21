#!/usr/bin/env bash
# admission: always 每一次调都照此刻片结果库里哪几片做完了判还差什么，上一次跑完不替这一次作保
# run-condition: command cargo git
# E182（被抛弃根读不出的三条岔路）第一段（单盘择根段，跑前登记 research/prompts/e182-preregistration.md 5.4 与 5.5）的长活跑法：
#   装置是入库装置 crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs（bin e182_unreadable_abandoned_root_forks），
#   按「一条历史 × 它录制流里的一个段」切片判，片结果按复用键原子存进片结果库，片齐了由装置自己合并计数、打产物
#   research/results/e182-unreadable-abandoned-root-forks-single-device.out。
#
# long-run-shards: 一片 = 一条历史 × 它录制流里的一个段，另加整条流末尾那个全持久的状态为一片；历史 18 条（G默认四个起点与 G窄两个起点，各三族），片数由段数定，片之间互不依赖，装置 list-items 列历史名、片号按历史次序与段号排下来
# long-run-resume: 同一条命令重起就是续跑：每片判完原子写一个以复用键命名的文件进片结果库，重起先读库，键相同的片直接取、只补没判的，成功那句报「现判 N、复用 M」；复用键 = 判法入口按 singlefs-unit-graph 算的函数单元闭包指纹（去注释）加工具链行、历史名、段号、展开方式
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
#   多机的读法：本脚本只管一台。第二台由主 agent 照 --emit-hosts 读出的那一台，经 bash scripts/hosts/multi-host-run.sh --peer --fetch <片结果库目录> -- bash scripts/experiments/e182-single-device-long-run.sh --share 1/2
#   起另一半（本机同时跑 --share 0/2），两边各写各的片结果库；取回之后在本机 --import-library <取回的目录> 把片文件并进本机库，再不带参数跑一次本脚本：片全部取自库、合并出产物。
#   装置在一趟里不许换（欠账 C862，共用段落在 long-run-lib/binary-identity.sh）：开跑时把装置二进制拷一份进 $LOG_DIRECTORY/pinned-binary/<sha256>/（只读），装置按这份拷贝起，
#   不再 exec 仓根 target/release 下那一份（别的会话的 cargo 随时会把它换掉）；开跑那句报这份拷贝的 sha256。这一段的片是片结果库里按复用键命名的文件，复用键含判法入口的函数单元闭包指纹
#   与工具链行，哪一版代码判的由键管，所以本脚本不给库里的片另记二进制 sha256（记了会把「代码没变、二进制重编过」的片也作废，违背按键复用）；--list-items 只列名，照旧直接起仓根那份。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiments/e182-single-device-long-run.sh                          本机跑全部片并出产物（片结果库里已有的片直接取）
#   bash scripts/experiments/e182-single-device-long-run.sh --full                   全量：不读库、全部现判、照键写回
#   bash scripts/experiments/e182-single-device-long-run.sh --share <第几份>/<共几份>   只判片号对共几份取余等于第几份的片，写进片结果库，不打产物
#   bash scripts/experiments/e182-single-device-long-run.sh --import-library <目录>    把另一台取回的片文件并进本机片结果库（不覆盖已有的），不编、不跑
#   bash scripts/experiments/e182-single-device-long-run.sh --list-items             列可点名的历史名（要装置已编好，不编译）
#   bash scripts/experiments/e182-single-device-long-run.sh --selftest               假装置、假内存包装、假 cargo 在临时树里走一遍整库续跑、--full、--share、--import-library、产物名取号与完成标记（不起 cargo、编译）
# 环境：E182_MEMORY_CAP（装置的内存包装上限，默认 16G）、E182_VERDICT_LIBRARY（片结果库目录；没设：不分片取 <git 公共目录>/singlefs-unit-verdicts/e182-single-device，
#   分片取仓根下的 e182-shard-library-share-<第几份>-of-<共几份>，好让 multi-host-run.sh --fetch 取得回）、E182_LOG_DIRECTORY（进度日志目录，默认 ~/.local/share/singlefs/work/e182-long-run）。
# 产物名：research/results/e182-unreadable-abandoned-root-forks-single-device.out 已有时另存 …-single-device-<日期>.out，同一天再跑加 -r2、-r3，已有的一个不动。
# 退出码：0 做完（分片时是这一份的片判完）；1 装置跑失败或产物不齐；2 用法错；3 装置报停机（该停下交回主 agent 的，日志里 STOP: 那一行）；78 装置没编好（--list-items）。
#
# 自证的做法：把本脚本拷进临时树（scripts/experiments/ 下），树里放假的 scripts/hosts/capped.sh 与 run-with-memory-cap.sh（记一笔再 exec）、
#   假的 target/release/ 装置（按 E182_SHARD_SHARE 与 E182_VERDICT_LIBRARY 往片结果库写六个片文件、打现判/复用那一行、按 E182_SELFTEST_DEVICE 装停机/失败/产物不齐）、
#   PATH 最前面的假 cargo（E182_SELFTEST_CARGO=fail 退 101）、指向真仓 .claude 的符号链接（准入判法）；每一格各搭一棵树、各用各的日志目录，格与格不共用状态。
#   换不掉的一处：产物名里的日期用 TZ=Asia/Tokyo date +%F 取，自证里同样取，跨零点的那一刻会误红，重跑即可。
# 弄坏开关 E182_SINGLE_DEVICE_BREAK=<项>（只在 --selftest 起的那几趟里认：自证设 E182_SINGLE_DEVICE_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   share-format-unchecked        --share 的写法不核对                              打开后自证「--share 写坏退 2：abc」「…：1/」「…：1-2」三格红
#   missing-value-unchecked       --share、--import-library 后面缺值不拦             打开后自证「--share 缺值退 2」「--import-library 缺值退 2」红
#   unknown-argument-accepted     认不得的参数悄悄跳过                              打开后自证「认不得的参数退 2」红
#   list-items-binary-unchecked   --list-items 不核对装置在不在                      打开后自证「--list-items：装置没编好退 78」红
#   import-directory-unchecked    --import-library 点名的目录不在也不拦              打开后自证「--import-library 目录不在退 2」红
#   git-common-directory-unchecked 不在 git 工作树里也往下走（库落在当前目录）        打开后自证「--import-library 不在 git 里又没设库退 2」红
#   import-overwrites             并进片文件时覆盖已有的（cp -f）                    打开后自证「--import-library 并进片文件、已有的不覆盖、不编不跑」红
#   build-failure-ignored         装置编不过也往下跑                                打开后自证「装置编不过退 1、装置一次没起」红
#   device-failure-ignored        装置退非 0 也不当失败                              打开后自证「装置失败退 1 并说退几」红
#   stop-exit-ignored             装置退 3（停机）当普通失败（退 1）                  打开后自证「装置报停机退 3、STOP 行转出来、不打产物」红
#   complete-marker-ignored       产物末行对不上行数也算齐                           打开后自证「产物末行对不上行数退 1、不入库」红
#   share-not-passed              --share 不传给装置                                打开后自证「--share 0/2 只判自己那一份、不打产物」红
#   full-ignored                  --full 不传给装置                                 打开后自证「--full 不读库、全部现判」红
#   share-library-name-shared     各份共用同一个片结果库目录名                       打开后自证「--share 两份各写各的片结果库」「分片、并进、再不带参数跑：片全取自库、合并出产物」红
#   product-name-fixed            产物名不取号、一律用本名                           打开后自证「原样重起：六片全取自库」「产物本名已有：另存带日期的」「本名与带日期的都已有：加 -r2」「--full 不读库、全部现判」红
#   exec-root-binary              装置仍按仓根那个路径起、不按私有副本               打开后自证「装置在一趟里不许换」红（跑着的时候仓根那份被换，装置自报的 sha256 就不是开跑时那份）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：本脚本得留在仓里的 scripts/experiments/ 下（它按自己的位置找仓根）；挪走了就放回去再起" >&2; exit 2; }

BINARY="target/release/e182_unreadable_abandoned_root_forks"
PRODUCT_STEM="research/results/e182-unreadable-abandoned-root-forks-single-device"
MEMORY_CAP="${E182_MEMORY_CAP:-16G}"
LOG_DIRECTORY="${E182_LOG_DIRECTORY:-$HOME/.local/share/singlefs/work/e182-long-run}"

usage_error() {
  echo "  ✗ $1" >&2
  echo "     → 怎么办：见本脚本文件头「用法」" >&2
  exit 2
}

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E182_SINGLE_DEVICE_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E182_SINGLE_DEVICE_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${E182_SINGLE_DEVICE_SELFTEST_MARK:-}" || "$(cat -- "${E182_SINGLE_DEVICE_SELFTEST_MARK}" 2> /dev/null)" != e182-single-device-selftest ]]; then
    echo "  ✗ E182_SINGLE_DEVICE_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    echo "     → 怎么办：平常跑时不设它" >&2
    exit 2
  fi
fi

# 产物做完了：末行是 name=done emitted=N 且 N 等于行数。
is_complete_output() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  break_is complete-marker-ignored && return 0
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E7RESULT name=done emitted=$line_count" ]]
}

# 还没被占的产物名：先试没有日期的，已有就加日期，同一天再加 -r2、-r3……
next_free_product_path() {
  local today candidate revision
  today=$(TZ=Asia/Tokyo date +%F)
  candidate="$PRODUCT_STEM.out"
  break_is product-name-fixed && { echo "$candidate"; return; }
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  candidate="$PRODUCT_STEM-$today.out"
  [[ -e "$candidate" ]] || { echo "$candidate"; return; }
  revision=2
  while [[ -e "$PRODUCT_STEM-$today-r$revision.out" ]]; do revision=$((revision + 1)); done
  echo "$PRODUCT_STEM-$today-r$revision.out"
}

build_the_binary() {
  bash scripts/hosts/capped.sh default cargo build --release -p singlefs-checker-tier --bin e182_unreadable_abandoned_root_forks || break_is build-failure-ignored || {
    echo "  ✗ 装置编不过" >&2
    echo "     → 怎么办：照 cargo 打的第一处 error 改装置，再起本脚本" >&2
    exit 1
  }
}

# 自证：假装置、假内存包装、假 cargo 在每一格自己的临时树里走一遍（做法与换不掉的一处见文件头）。
run_selftest() {
  local work failures=0 cell_count=0 fakes real_root today cell exit_code swap_target original_sha
  unset E182_VERDICT_LIBRARY E182_MEMORY_CAP E182_LOG_DIRECTORY
  real_root="$PWD"
  today=$(TZ=Asia/Tokyo date +%F)
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes/bin"
  echo e182-single-device-selftest > "$work/mark"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
echo "device $* | library=${E182_VERDICT_LIBRARY:-unset} share=${E182_SHARD_SHARE:-unset}" >> "$E182_SELFTEST_CALLS"
echo "$(sha256sum -- "$0" | cut -d' ' -f1)" >> "$E182_SELFTEST_CALLS.self-sha"
if [[ "$1" == list-items ]]; then printf 'history_a\nhistory_b\n'; exit 0; fi
full=0
for argument in "$@"; do [[ "$argument" == --full ]] && full=1; done
case "${E182_SELFTEST_DEVICE:-ok}" in
  stop) echo "STOP: selftest stop reason" >&2; exit 3 ;;
  fail) echo "boom" >&2; exit 1 ;;
esac
library="${E182_VERDICT_LIBRARY:-.git/singlefs-unit-verdicts/e182-single-device}"
mkdir -p -- "$library"
share_index=0; share_total=1
if [[ -n "${E182_SHARD_SHARE:-}" ]]; then share_index=${E182_SHARD_SHARE%/*}; share_total=${E182_SHARD_SHARE#*/}; fi
fresh=0; reused=0
for index in 0 1 2 3 4 5; do
  ((index % share_total == share_index)) || continue
  shard_file="$library/shard-$index.shard"
  if [[ -f "$shard_file" && $full == 0 ]]; then reused=$((reused + 1)); else echo "judged shard $index by device $(sha256sum -- "$0" | cut -d' ' -f1)" > "$shard_file"; fresh=$((fresh + 1)); fi
done
echo "现判 $fresh、复用 $reused" >&2
if [[ -n "${E182_SHARD_SHARE:-}" ]]; then exit 0; fi
for index in 0 1 2 3 4 5; do printf 'E7RESULT name=row shard=%s\n' "$index"; done
emitted=7
if [[ "${E182_SELFTEST_DEVICE:-ok}" == truncated ]]; then emitted=99; fi
printf 'E7RESULT name=done emitted=%s\n' "$emitted"
FAKE
  cat > "$fakes/capped.sh" <<'FAKE'
#!/usr/bin/env bash
echo "capped $1" >> "$E182_SELFTEST_CALLS"
# E182_SELFTEST_SWAP_TARGET：起装置之前（脚本已拷过私有副本）把这个路径的装置换成另一份字节不同的，像别的会话的 cargo 把仓根那份换掉。
if [[ -n "${E182_SELFTEST_SWAP_TARGET:-}" && "$*" == *run-with-memory-cap* && ! -e "$E182_SELFTEST_SWAP_TARGET.swapped" ]]; then
  { cat -- "$E182_SELFTEST_SWAP_TARGET"; echo "# swapped by a build while the long run is going"; } > "$E182_SELFTEST_SWAP_TARGET.swap" \
    && chmod +x "$E182_SELFTEST_SWAP_TARGET.swap" && mv -f -- "$E182_SELFTEST_SWAP_TARGET.swap" "$E182_SELFTEST_SWAP_TARGET" && : > "$E182_SELFTEST_SWAP_TARGET.swapped"
fi
shift
exec "$@"
FAKE
  printf '%s\n' '#!/usr/bin/env bash' 'echo "cap $1" >> "$E182_SELFTEST_CALLS"' 'shift' 'exec "$@"' > "$fakes/run-with-memory-cap.sh"
  cat > "$fakes/bin/cargo" <<'FAKE'
#!/usr/bin/env bash
echo "cargo $*" >> "$E182_SELFTEST_CALLS"
if [[ "${E182_SELFTEST_CARGO:-ok}" == fail ]]; then echo "error: selftest compile error" >&2; exit 101; fi
exit 0
FAKE
  chmod +x "$fakes/device" "$fakes/capped.sh" "$fakes/run-with-memory-cap.sh" "$fakes/bin/cargo"
  # 一格的树：本脚本的拷贝、假的 scripts/hosts 两件、假装置、指向真仓 .claude 的符号链接、空的 git 仓。
  make_tree() {
    local tree="$1/tree"
    mkdir -p -- "$tree/scripts/experiments/long-run-lib" "$tree/scripts/hosts" "$tree/target/release" "$tree/research/results" "$1/logs"
    cp -- "$SCRIPT_PATH" "$tree/scripts/experiments/e182-single-device-long-run.sh"
    cp -- "$(dirname -- "$SCRIPT_PATH")/long-run-lib/binary-identity.sh" "$tree/scripts/experiments/long-run-lib/"
    cp -- "$fakes/capped.sh" "$fakes/run-with-memory-cap.sh" "$tree/scripts/hosts/"
    cp -- "$fakes/device" "$tree/target/release/e182_unreadable_abandoned_root_forks"
    ln -s -- "$real_root/.claude" "$tree/.claude"
    git -C "$tree" init -q > /dev/null 2>&1
  }
  # 在这一格的树里起脚本拷贝；这一格要设的环境变量写在 invoke 前面。
  invoke() {
    local cell="$1"
    shift
    ( cd -- "$cell/tree" && env PATH="$fakes/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E182_SELFTEST_CALLS="$cell/calls" \
        E182_LOG_DIRECTORY="$cell/logs" E182_SINGLE_DEVICE_SELFTEST_MARK="$work/mark" \
        bash "$cell/tree/scripts/experiments/e182-single-device-long-run.sh" "$@" > "$cell/out" 2>&1 )
  }
  count_calls() {
    local count
    count=$(grep -c -- "$2" "$1/calls" 2> /dev/null) || true
    echo "${count:-0}"
  }
  count_files() {
    find "$1" -type f 2> /dev/null | wc -l
  }
  expect() {
    local name="$1" condition="$2"
    cell_count=$((cell_count + 1))
    if eval "$condition"; then
      echo "  ✓ $name（第 $cell_count 格）"
    else
      echo "  ✗ $name：条件「$condition」不成立"
      [[ -f "$cell/out" ]] && tail -n 6 -- "$cell/out" | sed 's/^/      输出：/'
      echo "     → 怎么办：照上面这一格点名的条件和输出改本脚本，再单独起 bash scripts/experiments/e182-single-device-long-run.sh --selftest 到它退 0"
      failures=$((failures + 1))
    fi
  }
  local stem="research/results/e182-unreadable-abandoned-root-forks-single-device"
  local results="" first=0 second=0 bad_share shard_number
  cell="$work/fresh"; make_tree "$cell"
  invoke "$cell"; exit_code=$?
  expect "单机全新：构建一次、六片现判、产物按本名落进 research/results" '((exit_code == 0)) && grep -qF "产物：$stem.out（7 行）；现判 6、复用 0" "$cell/out" && (($(wc -l < "$cell/tree/$stem.out") == 7)) && (($(count_calls "$cell" "^cargo build --release -p singlefs-checker-tier --bin e182_unreadable_abandoned_root_forks") == 1)) && grep -qxF "device single-device | library=unset share=unset" "$cell/calls" && grep -qx "cap 16G" "$cell/calls"'
  cell="$work/restart"; make_tree "$cell"
  invoke "$cell"; first=$?
  invoke "$cell"; exit_code=$?
  expect "原样重起：六片全取自库" '((first == 0 && exit_code == 0)) && grep -qF "现判 0、复用 6" "$cell/out" && [[ -f "$cell/tree/$stem-$today.out" ]]'
  cell="$work/name-taken"; make_tree "$cell"
  echo old-product > "$cell/tree/$stem.out"
  invoke "$cell"; exit_code=$?
  expect "产物本名已有：另存带日期的" '((exit_code == 0)) && (($(wc -l < "$cell/tree/$stem-$today.out") == 7)) && [[ "$(cat -- "$cell/tree/$stem.out")" == old-product ]]'
  cell="$work/name-and-date-taken"; make_tree "$cell"
  echo old-product > "$cell/tree/$stem.out"; echo old-dated-product > "$cell/tree/$stem-$today.out"
  invoke "$cell"; exit_code=$?
  expect "本名与带日期的都已有：加 -r2" '((exit_code == 0)) && (($(wc -l < "$cell/tree/$stem-$today-r2.out") == 7)) && [[ "$(cat -- "$cell/tree/$stem.out")" == old-product ]] && [[ "$(cat -- "$cell/tree/$stem-$today.out")" == old-dated-product ]]'
  cell="$work/full"; make_tree "$cell"
  invoke "$cell"; first=$?
  invoke "$cell" --full; exit_code=$?
  expect "--full 不读库、全部现判" '((first == 0 && exit_code == 0)) && grep -qF "现判 6、复用 0" "$cell/out" && grep -qxF "device single-device --full | library=unset share=unset" "$cell/calls"'
  cell="$work/library-variable"; make_tree "$cell"
  E182_VERDICT_LIBRARY="$cell/library" invoke "$cell"; exit_code=$?
  expect "E182_VERDICT_LIBRARY 设了就照它传给装置" '((exit_code == 0)) && grep -qxF "device single-device | library=$cell/library share=unset" "$cell/calls" && (($(count_files "$cell/library") == 6))'
  cell="$work/share"; make_tree "$cell"
  invoke "$cell" --share 0/2; exit_code=$?
  expect "--share 0/2 只判自己那一份、不打产物" '((exit_code == 0)) && grep -qF "这一份（0/2）的片判完：现判 3、复用 0" "$cell/out" && grep -qF "share=0/2" "$cell/calls" && (($(count_files "$cell/tree/research/results") == 0))'
  cell="$work/share-libraries"; make_tree "$cell"
  invoke "$cell" --share 0/2; first=$?
  invoke "$cell" --share 1/2; second=$?
  expect "--share 两份各写各的片结果库" '((first == 0 && second == 0)) && [[ -d "$cell/tree/e182-shard-library-share-0-of-2" && -d "$cell/tree/e182-shard-library-share-1-of-2" ]] && grep -qF "library=e182-shard-library-share-0-of-2" "$cell/calls" && grep -qF "library=e182-shard-library-share-1-of-2" "$cell/calls"'
  for bad_share in abc 1/ 1-2; do
    cell="$work/bad-share-${bad_share//\//-slash}"; make_tree "$cell"
    invoke "$cell" --share "$bad_share"; exit_code=$?
    expect "--share 写坏退 2：$bad_share" '((exit_code == 2)) && grep -qF -e "--share 要写 第几份/共几份" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  done
  cell="$work/share-no-value"; make_tree "$cell"
  invoke "$cell" --share; exit_code=$?
  expect "--share 缺值退 2" '((exit_code == 2)) && grep -qF -e "--share 后面要写" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/import-no-value"; make_tree "$cell"
  invoke "$cell" --import-library; exit_code=$?
  expect "--import-library 缺值退 2" '((exit_code == 2)) && grep -qF -e "--import-library 后面要写" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/unknown-argument"; make_tree "$cell"
  invoke "$cell" --bogus; exit_code=$?
  expect "认不得的参数退 2" '((exit_code == 2)) && grep -qF "认不得的参数「--bogus」" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/import-missing-directory"; make_tree "$cell"
  invoke "$cell" --import-library "$cell/no-such-directory"; exit_code=$?
  expect "--import-library 目录不在退 2" '((exit_code == 2)) && grep -qF "点名的目录不在" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/import-without-git"; make_tree "$cell"
  rm -rf -- "${cell:?}/tree/.git"; mkdir -p -- "$cell/fetched"
  invoke "$cell" --import-library "$cell/fetched"; exit_code=$?
  expect "--import-library 不在 git 里又没设库退 2" '((exit_code == 2)) && grep -qF "不在 git 工作树里" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/import"; make_tree "$cell"
  mkdir -p -- "$cell/fetched/e182-shard-library-share-1-of-2" "$cell/tree/.git/singlefs-unit-verdicts/e182-single-device"
  for shard_number in 1 3 5; do echo "fetched shard $shard_number" > "$cell/fetched/e182-shard-library-share-1-of-2/shard-$shard_number.shard"; done
  echo kept > "$cell/tree/.git/singlefs-unit-verdicts/e182-single-device/shard-3.shard"
  invoke "$cell" --import-library "$cell/fetched"; exit_code=$?
  expect "--import-library 并进片文件、已有的不覆盖、不编不跑" '((exit_code == 0)) && grep -qF "并进片文件 2 个" "$cell/out" && [[ "$(cat -- "$cell/tree/.git/singlefs-unit-verdicts/e182-single-device/shard-3.shard")" == kept ]] && [[ -f "$cell/tree/.git/singlefs-unit-verdicts/e182-single-device/shard-1.shard" && -f "$cell/tree/.git/singlefs-unit-verdicts/e182-single-device/shard-5.shard" ]] && [[ ! -e "$cell/calls" ]]'
  cell="$work/shards-then-import"; make_tree "$cell"
  invoke "$cell" --share 0/2; first=$?
  invoke "$cell" --share 1/2; second=$?
  invoke "$cell" --import-library e182-shard-library-share-0-of-2
  invoke "$cell" --import-library e182-shard-library-share-1-of-2
  invoke "$cell"; exit_code=$?
  expect "分片、并进、再不带参数跑：片全取自库、合并出产物" '((first == 0 && second == 0 && exit_code == 0)) && grep -qF "现判 0、复用 6" "$cell/out" && (($(wc -l < "$cell/tree/$stem.out") == 7)) && (($(count_calls "$cell" "^device single-device") == 3))'
  cell="$work/build-failure"; make_tree "$cell"
  E182_SELFTEST_CARGO=fail invoke "$cell"; exit_code=$?
  expect "装置编不过退 1、装置一次没起" '((exit_code == 1)) && grep -qF "装置编不过" "$cell/out" && (($(count_calls "$cell" "^device") == 0))'
  cell="$work/device-stop"; make_tree "$cell"
  E182_SELFTEST_DEVICE=stop invoke "$cell"; exit_code=$?
  expect "装置报停机退 3、STOP 行转出来、不打产物" '((exit_code == 3)) && grep -qF "STOP: selftest stop reason" "$cell/out" && (($(count_files "$cell/tree/research/results") == 0))'
  cell="$work/device-failure"; make_tree "$cell"
  E182_SELFTEST_DEVICE=fail invoke "$cell"; exit_code=$?
  expect "装置失败退 1 并说退几" '((exit_code == 1)) && grep -qF "装置退 1（日志" "$cell/out" && (($(count_files "$cell/tree/research/results") == 0))'
  cell="$work/product-truncated"; make_tree "$cell"
  E182_SELFTEST_DEVICE=truncated invoke "$cell"; exit_code=$?
  expect "产物末行对不上行数退 1、不入库" '((exit_code == 1)) && grep -qF "产物不齐" "$cell/out" && (($(count_files "$cell/tree/research/results") == 0))'
  cell="$work/swap"; make_tree "$cell"
  swap_target="$cell/tree/target/release/e182_unreadable_abandoned_root_forks"
  original_sha=$(sha256sum -- "$swap_target" | cut -d' ' -f1)
  E182_SELFTEST_SWAP_TARGET="$swap_target" invoke "$cell"; exit_code=$?
  expect "装置在一趟里不许换：起装置之前仓根那份被换掉，装置自报的 sha256 与片里记的、开跑那句报的都是开跑时那份" '((exit_code == 0)) && [[ -e "$swap_target.swapped" ]] && [[ "$(sha256sum -- "$swap_target" | cut -d" " -f1)" != "$original_sha" ]] && [[ "$(sort -u "$cell/calls.self-sha")" == "$original_sha" ]] && grep -qF "装置 sha256 $original_sha（私有副本" "$cell/out" && (($(grep -l "by device $original_sha" "$cell"/tree/.git/singlefs-unit-verdicts/e182-single-device/shard-*.shard | wc -l) == 6))'
  cell="$work/list-without-binary"; make_tree "$cell"
  rm -f -- "$cell/tree/target/release/e182_unreadable_abandoned_root_forks"
  invoke "$cell" --list-items; exit_code=$?
  expect "--list-items：装置没编好退 78" '((exit_code == 78)) && grep -qF "装置还没编好" "$cell/out"'
  cell="$work/list"; make_tree "$cell"
  invoke "$cell" --list-items; exit_code=$?
  expect "--list-items：编好了经 4G 内存包装列名、不编译" '((exit_code == 0)) && grep -qx "history_a" "$cell/out" && grep -qx "cap 4G" "$cell/calls" && (($(count_calls "$cell" "^cargo") == 0))'
  rm -rf -- "${work:?}"
  if ((failures > 0)); then echo "e182-single-device-long-run.sh 自证：$failures 格红（共 $cell_count 格）"; return 1; fi
  echo "e182-single-device-long-run.sh 自证：$cell_count 格全绿"
  return 0
}

full=0
share=""
import_directory=""
list_items=0
selftest=0
while (($# > 0)); do
  case "$1" in
    --full) full=1; shift ;;
    --share) (($# >= 2)) || break_is missing-value-unchecked || usage_error "--share 后面要写 第几份/共几份"; share="$2"; shift 2 ;;
    --import-library) (($# >= 2)) || break_is missing-value-unchecked || usage_error "--import-library 后面要写目录"; import_directory="$2"; shift 2 ;;
    --list-items) list_items=1; shift ;;
    --selftest) selftest=1; shift ;;
    *) break_is unknown-argument-accepted && { shift; continue; }; usage_error "认不得的参数「$1」" ;;
  esac
done

if ((selftest)); then
  run_selftest
  exit $?
fi

if [[ -n "$share" && ! "$share" =~ ^[0-9]+/[0-9]+$ ]] && ! break_is share-format-unchecked; then usage_error "--share 要写 第几份/共几份（两个整数），给的是「$share」"; fi

if ((list_items)); then
  [[ -x "$BINARY" ]] || break_is list-items-binary-unchecked || { echo "  ✗ 装置还没编好：$BINARY" >&2; echo "     → 怎么办：先不带 --list-items 跑一次本脚本，或 cargo build --release -p singlefs-checker-tier --bin e182_unreadable_abandoned_root_forks" >&2; exit 78; }
  bash scripts/hosts/run-with-memory-cap.sh 4G "$BINARY" list-items
  exit $?
fi

library="${E182_VERDICT_LIBRARY:-}"
if [[ -z "$library" && -n "$share" ]]; then
  library="e182-shard-library-share-${share%/*}-of-${share#*/}"
  break_is share-library-name-shared && library="e182-shard-library-share-all"
fi

if [[ -n "$import_directory" ]]; then
  [[ -d "$import_directory" ]] || break_is import-directory-unchecked || usage_error "--import-library 点名的目录不在：$import_directory"
  if [[ -z "$library" ]]; then
    common=$(git rev-parse --git-common-dir) || { break_is git-common-directory-unchecked && common=.; } || { echo "  ✗ 不在 git 工作树里，设 E182_VERDICT_LIBRARY" >&2; echo "     → 怎么办：在 git 工作树里起本脚本，或把 E182_VERDICT_LIBRARY 设成片结果库的目录再起" >&2; exit 2; }
    library="$common/singlefs-unit-verdicts/e182-single-device"
  fi
  mkdir -p -- "$library" || exit 1
  imported=0
  while IFS= read -r -d '' shard_file; do
    # 已有同名的不覆盖，也不算并进来（cp -n 在 coreutils 9.2 起跳过时退 0，数不出跳过了几个）。
    if [[ -e "$library/${shard_file##*/}" ]] && ! break_is import-overwrites; then continue; fi
    cp -- "$shard_file" "$library/" && imported=$((imported + 1))
  done < <(find "$import_directory" -maxdepth 2 -name '*.shard' -print0)
  echo "并进片文件 $imported 个（已有的不覆盖）：$library"
  exit 0
fi

build_the_binary
mkdir -p -- "$LOG_DIRECTORY" || exit 1
run_stamp="$(TZ=Asia/Tokyo date +%F)-$$"
progress_log="$LOG_DIRECTORY/e182-single-device-$run_stamp.progress.log"
product_scratch="$LOG_DIRECTORY/e182-single-device-$run_stamp.product.out"
arguments=(single-device)
if ((full)) && ! break_is full-ignored; then arguments+=(--full); fi
environment=()
[[ -n "$library" ]] && environment+=("E182_VERDICT_LIBRARY=$library")
if [[ -n "$share" ]] && ! break_is share-not-passed; then environment+=("E182_SHARD_SHARE=$share"); fi

# 开跑时拷一份私有副本，装置按这份拷贝起（上面 cargo build 与这一步之间被换掉的，拷的就是换后那一份，sha256 是拷贝的）
binary_identity_pin "$BINARY" "$LOG_DIRECTORY/pinned-binary" || exit 1
launch_binary="$PINNED_BINARY"
break_is exec-root-binary && launch_binary="$BINARY"
echo "装置 sha256 $PINNED_BINARY_SHA256（私有副本 $PINNED_BINARY，源 $BINARY）"
binary_identity_assert_unchanged || exit 1
env ${environment[@]+"${environment[@]}"} bash scripts/hosts/capped.sh default \
  bash scripts/hosts/run-with-memory-cap.sh "$MEMORY_CAP" "$launch_binary" "${arguments[@]}" > "$product_scratch" 2> "$progress_log"
exit_code=$?
reuse_line=$(grep -m1 '^现判 ' -- "$progress_log")
if ((exit_code == 3)) && ! break_is stop-exit-ignored; then
  echo "  ✗ 装置报停机，该停下交回主 agent" >&2
  grep -m3 '^STOP: ' -- "$progress_log" >&2
  echo "     → 怎么办：把日志 $progress_log 里的 STOP 行交主 agent，不重跑" >&2
  exit 3
fi
if ((exit_code != 0)) && ! break_is device-failure-ignored; then
  echo "  ✗ 装置退 $exit_code（日志 $progress_log）" >&2
  tail -n 5 -- "$progress_log" >&2
  echo "     → 怎么办：看日志末尾；内存包装的退出码含义见 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」" >&2
  exit 1
fi
if [[ -n "$share" ]]; then
  echo "这一份（$share）的片判完：${reuse_line:-（日志里没有现判/复用那一行）}；片结果库：${library:-默认库}"
  exit 0
fi
if ! is_complete_output "$product_scratch"; then
  echo "  ✗ 产物不齐（末行不是 name=done emitted=行数）：$product_scratch" >&2
  echo "     → 怎么办：看日志 $progress_log；产物不入库" >&2
  exit 1
fi
destination=$(next_free_product_path)
mv -n -- "$product_scratch" "$destination" || exit 1
echo "产物：$destination（$(wc -l < "$destination") 行）；${reuse_line:-（日志里没有现判/复用那一行）}；进度日志：$progress_log"
