#!/usr/bin/env bash
# admission: always 每一次调都照此刻产物目录里哪几片做完了判还差什么，上一次跑完不替这一次作保
# run-condition: command cargo python3 nice
# E10（老化：结构抗老化 vs 事后整理） 第 1 次跑的第一段（跑前登记 research/prompts/e10-r1-prereg.md 第五节「每段跑法」）的分片跑法：
#   装置 scripts/experiment-apparatus/e10_aging_compaction.rs（bin e10-aging-compaction）按（取样点, S）切片跑，片齐了用装置的 --merge 拼成
#   与不带参数跑逐字节相同的产物 research/results/e10-aging-compaction-<日期>.out（日期取 E10_RUN_DATE，默认 2026-10-13，这一段开跑的那天）。
#
# long-run-shards: 一片 =（取样点, S），七个取样点 × S 三档共二十一片；片名 <取样点>-s<S>，片产物 research/results/e10-aging-compaction-<日期>/<片名>.out；片的次序就是全跑的正式跑次序，装置的 --merge 逐条核次序与条数
# long-run-resume: 同一条命令重起只跑缺的片：片文件在、末行是 name=done emitted=N 且 N 等于行数、片旁 <片名>.out.binary-sha256 记的装置 sha256 是这一趟的才算做完、跳过（记的不同或没有记录的片整份移到 <E10_LOG_DIRECTORY>/void-<日期>-<进程号>/ 再现跑，逐片报「作废：<片名>（装置 sha256 不同）」）；每片先写进临时目录，核过收尾行、记下装置 sha256 才换名进片目录，跑到一半被打断的片不留在片目录里
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
#   多机的读法：每台线程是 scripts/hosts/heavy-threads.sh 的 heavy_thread_default（E10_THREADS 设了就用它）；第二台照 bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts 读出的那一台，
#   由主 agent 经 bash scripts/hosts/multi-host-run.sh --peer --fetch <片目录（片文件连同 <片名>.out.binary-sha256）> -- bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --shards <片名,…> 起另一半、取回之后在本机再跑一次本脚本合并
#   装置在一趟里不许换，每片记下打它的是哪一版装置（欠账 C862，共用段落在 long-run-lib/binary-identity.sh）：开跑时把装置程序文件拷一份进 $E10_LOG_DIRECTORY/pinned-binary/<sha256>/（只读），
#   每片、合并都按这份拷贝起，不再 exec 编译目录里那一份（别的会话的 cargo 随时会把它换掉）；每片做完先在片旁写 <片名>.out.binary-sha256 再换名进片目录；续跑只沿用记的 sha256 相同的片；
#   合并之前再核二十一片记的 sha256 都是这一趟的，不同就拒、不出产物。第二台要用与本机同一份装置：把本机私有副本经 multi-host-run.sh 的 --send 送过去，第二台设 E10_BINARY 指到送到的文件
#   （不再自己 cargo build）、E10_SHIPPED_BINARY_SHA256 写本机算的 sha256（核送到的文件相同才往下，不同退 1；并在 PATH 最前面放 python3 垫片，让装置的准入判法找得到第二台这棵树里的 preflight.py 与源文件）；
#   第二台自己编的另一份 sha256 一般不同，取回的片会被本机当成别的版本作废。老的私有副本留着不删（一个 sha256 一个目录），确认不再续跑那一版之后手删。
#
# 用法（在仓里任何目录起都行，脚本自己回到仓根）：
#   bash scripts/experiment-apparatus/e10-aging-compaction-run.sh                 跑缺的片；二十一片齐了合并出产物（产物已在就只核、不重写）
#   bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --shards a,b    只跑点名的片，不合并
#   bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --list-items    逐行列片名与做没做完，不编、不跑
#   bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --selftest      假装置、假内存包装、假 cargo 在临时树里走一遍续跑、换装置作废、跑着换二进制、合并前核 sha256（不起 cargo、不碰真装置）
# 环境：E10_THREADS（装置的线程数，没设取登记的每台默认重活线程数）、E10_MEMORY_CAP（每片的内存包装上限，默认 auto：照峰值表现算）、E10_RUN_DATE（产物日期）、
#   E10_LOG_DIRECTORY（装置私有副本与作废的片放这里，默认 ~/.local/share/singlefs/work/e10-run）、E10_BINARY（装置程序文件；设了就不 cargo build，用它）、E10_SHIPPED_BINARY_SHA256（见上，与 E10_BINARY 合用）。
# 退出码：0 二十一片齐、产物在；1 有片跑失败（那一片不留）或合并失败；2 用法错；3 片还没齐（--shards 跑完的正常结局）。
#
# 弄坏开关 E10_LONG_RUN_BREAK=<项，逗号分隔>（只在 --selftest 起的那几趟里认：自证设 E10_LONG_RUN_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   exec-root-binary      每片仍按编译目录那个路径起、不按私有副本               打开后自证「装置在一趟里不许换」红（跑着的时候仓根那份被换，后起的片自报的 sha256 就不是开跑时那份）
#   stale-binary-reused   沿用只看片末行、不看片旁记的装置 sha256               打开后自证「旧装置打的片作废」红
#   merge-sha-unchecked   合并之前不核各片记的装置 sha256                      打开后自证「合并之前各片记的装置 sha256 不同：拒」红
#   done-marker-ignored   片输出末行不齐也当做完                                打开后自证「末行不齐的片输出不进片目录」红
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
source "$(dirname "${BASH_SOURCE[0]}")/../hosts/heavy-threads.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
source "$(dirname "${BASH_SOURCE[0]}")/long-run-lib/binary-identity.sh"
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根" >&2; echo "     → 怎么办：从仓根照文件头的用法起：bash scripts/experiment-apparatus/e10-aging-compaction-run.sh" >&2; exit 2; }

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E10_LONG_RUN_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E10_LONG_RUN_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${E10_LONG_RUN_SELFTEST_MARK:-}" || "$(cat -- "${E10_LONG_RUN_SELFTEST_MARK}" 2> /dev/null)" != e10-long-run-selftest ]]; then
    echo "  ✗ E10_LONG_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
    echo "     → 怎么办：平常跑时不设它" >&2
    exit 2
  fi
fi

RUN_DATE="${E10_RUN_DATE:-2026-10-13}"
SHARD_DIRECTORY="research/results/e10-aging-compaction-$RUN_DATE"
PRODUCT="research/results/e10-aging-compaction-$RUN_DATE.out"
BINARY="${E10_BINARY:-scripts/experiment-apparatus/target/release/e10-aging-compaction}"
LOG_DIRECTORY="${E10_LOG_DIRECTORY:-$HOME/.local/share/singlefs/work/e10-run}"
DEVICE_SOURCE_PATH="scripts/experiment-apparatus/e10_aging_compaction.rs"
MEMORY_CAP="${E10_MEMORY_CAP:-auto}"
POINTS=(main disk_1gib no_remount delete_units snapshots_10 leaf_203 base_6sc)
ROOT_RING_SLOTS_PER_REGION_CHOICES=(4 8 16)

all_shard_names() {
  local point root_ring_slots_per_region
  for point in "${POINTS[@]}"; do
    for root_ring_slots_per_region in "${ROOT_RING_SLOTS_PER_REGION_CHOICES[@]}"; do
      echo "$point-s$root_ring_slots_per_region"
    done
  done
}

# 一份片（或产物）做完了：末行是 name=done emitted=N 且 N 等于行数。
is_complete_output() {
  local file="$1" line_count last_line
  [[ -f "$file" ]] || return 1
  break_is done-marker-ignored && return 0
  line_count=$(wc -l < "$file")
  last_line=$(tail -n 1 -- "$file")
  [[ "$last_line" == "E7RESULT name=done emitted=$line_count" ]]
}

# 装置的线程数：E10_THREADS 设了用它，没设取登记的每台默认重活线程数（heavy_thread_default，拒的时候它在 stderr 写原因与出路）。
thread_count() {
  if [[ -n "${E10_THREADS:-}" ]]; then
    echo "$E10_THREADS"
    return
  fi
  heavy_thread_default
}

run_shard() {
  local name="$1" point="${1%-s*}" root_ring_slots_per_region="${1##*-s}" temporary started finished exit_code
  temporary=$(mktemp -d)
  binary_identity_assert_unchanged || return 1
  started=$(date +%s)
  E10_THREADS="$THREADS" bash scripts/hosts/run-with-memory-cap.sh "$MEMORY_CAP" nice -n 19 "$LAUNCH_BINARY" \
    --point "$point" --root-ring-slots-per-region "$root_ring_slots_per_region" > "$temporary/out" 2> "$temporary/err"
  exit_code=$?
  finished=$(date +%s)
  if [[ "$exit_code" == 0 ]] && is_complete_output "$temporary/out" && [[ ! -e "$SHARD_DIRECTORY/$name.out" ]]; then
    # 先在片旁记下打它的装置 sha256（这份私有副本的），再换名进片目录：有片就一定有记录
    binary_identity_stamp_write "$SHARD_DIRECTORY/$name.out" || return 1
    mv -- "$temporary/out" "$SHARD_DIRECTORY/$name.out"
    echo "片 $name：退 0，$((finished - started)) 秒，$(wc -l < "$SHARD_DIRECTORY/$name.out") 行"
    rm -rf -- "$temporary"
    return 0
  fi
  echo "片 $name：退 $exit_code，$((finished - started)) 秒，没进片目录（错误输出留在 $temporary/err）" >&2
  return 1
}

# 自证：假装置、假内存包装、假 cargo 在每一格自己的临时树里走一遍（本脚本与共用库拷进树里，树里 .claude 指向真仓的）；不起 cargo、不碰真装置。
run_selftest() {
  local work failures=0 cell_count=0 fakes real_root cell exit_code first_exit old_sha original_sha swap_target product shard_count=21
  unset E10_BINARY E10_SHIPPED_BINARY_SHA256 E10_LOG_DIRECTORY E10_MEMORY_CAP
  real_root="$PWD"
  work=$(mktemp -d) || return 1
  fakes="$work/fakes"
  mkdir -p -- "$fakes/bin"
  echo e10-long-run-selftest > "$work/mark"
  product="research/results/e10-aging-compaction-2099-01-01.out"
  cat > "$fakes/device" <<'FAKE'
#!/usr/bin/env bash
# 假装置：--point <点> --root-ring-slots-per-region <S> 打一行 row 和收尾行；--merge <片…> 打 merged 和收尾行。
self_sha=$(sha256sum -- "$0" | cut -d' ' -f1)
echo "$*" >> "$E10_SELFTEST_CALLS"
echo "$* $self_sha" >> "$E10_SELFTEST_CALLS.self-sha"
if [[ "$1" == --merge ]]; then
  shift
  printf 'E7RESULT name=merged shards=%s\nE7RESULT name=done emitted=2\n' "$#"
  exit 0
fi
case "${E10_SELFTEST_DEVICE:-ok}" in
  fail) echo "boom" >&2; exit 1 ;;
  truncated) printf 'E7RESULT name=row\nE7RESULT name=done emitted=99\n'; exit 0 ;;
esac
# E10_SELFTEST_TAMPER_STAMP：跑最后一片的时候改写别处一片的装置记录，像别的趟用另一份装置写了同一个片目录。
if [[ -n "${E10_SELFTEST_TAMPER_STAMP:-}" && "$2" == base_6sc && "$4" == 16 ]]; then
  printf '%s\n' "$(printf 'f%.0s' {1..64})" > "$E10_SELFTEST_TAMPER_STAMP"
fi
printf 'E7RESULT name=row point=%s s=%s device=%s\nE7RESULT name=done emitted=2\n' "$2" "$4" "$self_sha"
FAKE
  cat > "$fakes/run-with-memory-cap.sh" <<'FAKE'
#!/usr/bin/env bash
echo "cap $1" >> "$E10_SELFTEST_CALLS.wrappers"
# E10_SELFTEST_SWAP_TARGET：起装置之前（脚本已拷过私有副本）把这个路径的装置换成另一份字节不同的，像别的会话的 cargo 把编译目录里那份换掉。
if [[ -n "${E10_SELFTEST_SWAP_TARGET:-}" && "$*" == *"nice -n 19"* && ! -e "$E10_SELFTEST_SWAP_TARGET.swapped" ]]; then
  { cat -- "$E10_SELFTEST_SWAP_TARGET"; echo "# swapped by a build while the long run is going"; } > "$E10_SELFTEST_SWAP_TARGET.swap" \
    && chmod +x "$E10_SELFTEST_SWAP_TARGET.swap" && mv -f -- "$E10_SELFTEST_SWAP_TARGET.swap" "$E10_SELFTEST_SWAP_TARGET" && : > "$E10_SELFTEST_SWAP_TARGET.swapped"
fi
shift
exec "$@"
FAKE
  cat > "$fakes/bin/cargo" <<'FAKE'
#!/usr/bin/env bash
echo "cargo $*" >> "$E10_SELFTEST_CALLS.cargo"
exit 0
FAKE
  chmod +x "$fakes/device" "$fakes/run-with-memory-cap.sh" "$fakes/bin/cargo"
  # 一格的树：本脚本与共用库的拷贝、假的 run-with-memory-cap.sh、真的线程取法、默认位置上的假装置、指向真仓 .claude 的符号链接、空的 git 仓。
  make_tree() {
    local tree="$1/tree"
    mkdir -p -- "$tree/scripts/experiment-apparatus/long-run-lib" "$tree/scripts/experiment-apparatus/target/release" "$tree/scripts/hosts" "$tree/research/results" "$1/logs" "$1/shipped"
    cp -- "$SCRIPT_PATH" "$tree/scripts/experiment-apparatus/e10-aging-compaction-run.sh"
    cp -- "$(dirname -- "$SCRIPT_PATH")/long-run-lib/binary-identity.sh" "$tree/scripts/experiment-apparatus/long-run-lib/"
    cp -- "$fakes/run-with-memory-cap.sh" "$real_root/scripts/hosts/heavy-threads.sh" "$tree/scripts/hosts/"
    cp -- "$fakes/device" "$tree/scripts/experiment-apparatus/target/release/e10-aging-compaction"
    cp -- "$fakes/device" "$1/shipped/e10-aging-compaction"
    ln -s -- "$real_root/.claude" "$tree/.claude"
    git -C "$tree" init -q > /dev/null 2>&1
  }
  invoke() {
    local cell="$1"
    shift
    ( cd -- "$cell/tree" && env PATH="$fakes/bin:$PATH" GIT_CEILING_DIRECTORIES="$work" E10_SELFTEST_CALLS="$cell/calls" E10_THREADS=3 E10_RUN_DATE=2099-01-01 \
        E10_LOG_DIRECTORY="$cell/logs" E10_LONG_RUN_SELFTEST_MARK="$work/mark" \
        bash "$cell/tree/scripts/experiment-apparatus/e10-aging-compaction-run.sh" "$@" > "$cell/out" 2>&1 )
  }
  expect() {
    local name="$1" condition="$2"
    cell_count=$((cell_count + 1))
    if eval "$condition"; then
      echo "  ✓ $name（第 $cell_count 格）"
    else
      echo "  ✗ $name：条件「$condition」不成立"
      [[ -f "$cell/out" ]] && tail -n 6 -- "$cell/out" | sed 's/^/      输出：/'
      echo "     → 怎么办：照上面这一格点名的条件和输出改本脚本，再单独起 bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --selftest 到它退 0"
      failures=$((failures + 1))
    fi
  }
  local shard_directory="research/results/e10-aging-compaction-2099-01-01"
  cell="$work/fresh"; make_tree "$cell"
  original_sha=$(sha256sum -- "$cell/tree/scripts/experiment-apparatus/target/release/e10-aging-compaction" | cut -d' ' -f1)
  invoke "$cell"; exit_code=$?
  expect "全新：二十一片现跑、合并出产物、每片片旁记着装置 sha256、cargo 编一次" '((exit_code == 0)) && (($(wc -l < "$cell/calls") == shard_count + 1)) && [[ -f "$cell/tree/$product" ]] && (($(ls "$cell/tree/$shard_directory"/*.out.binary-sha256 | wc -l) == shard_count)) && [[ "$(cat "$cell/tree/$shard_directory"/*.out.binary-sha256 | sort -u)" == "$original_sha" ]] && (($(wc -l < "$cell/calls.cargo") == 1))'
  invoke "$cell"; exit_code=$?
  expect "原样重起：二十一片全沿用、装置一次没起、产物已在不重写" '((exit_code == 0)) && (($(grep -c "已做完，复用" "$cell/out") == shard_count)) && (($(wc -l < "$cell/calls") == shard_count + 1)) && grep -qF "产物已在" "$cell/out"'
  cell="$work/old-binary"; make_tree "$cell"
  invoke "$cell"; first_exit=$?
  old_sha=$(sha256sum -- "$cell/tree/scripts/experiment-apparatus/target/release/e10-aging-compaction" | cut -d' ' -f1)
  echo "# a newer build of the device" >> "$cell/tree/scripts/experiment-apparatus/target/release/e10-aging-compaction"
  invoke "$cell"; exit_code=$?
  expect "旧装置打的二十一片全作废：逐片报作废、全现跑、旧片连记录整份留在作废目录、片旁记的都是新装置的" '((first_exit == 0 && exit_code == 0)) && (($(grep -c "^作废：.*（装置 sha256 不同）" "$cell/out") == shard_count)) && grep -q "^作废：main-s4（装置 sha256 不同）" "$cell/out" && (($(find "$cell"/logs/void-* -name "*.out" | wc -l) == shard_count)) && (($(find "$cell"/logs/void-* -name "*.binary-sha256" | wc -l) == shard_count)) && ! grep -q "$old_sha" "$cell/tree/$shard_directory"/*.binary-sha256 && (($(wc -l < "$cell/calls") == 2 * shard_count + 1))'
  cell="$work/stale-two"; make_tree "$cell"
  invoke "$cell"; first_exit=$?
  printf '%s\n' "$(printf 'f%.0s' {1..64})" > "$cell/tree/$shard_directory/main-s8.out.binary-sha256"
  rm -f -- "$cell/tree/$shard_directory/leaf_203-s16.out.binary-sha256"
  invoke "$cell"; exit_code=$?
  expect "片旁记的 sha256 不同的、没有记录的片都作废重跑，另外十九片沿用" '((first_exit == 0 && exit_code == 0)) && (($(grep -c "已做完，复用" "$cell/out") == shard_count - 2)) && grep -q "^作废：main-s8（装置 sha256 不同）：记的是 f" "$cell/out" && grep -q "^作废：leaf_203-s16（装置 sha256 不同）：记的是 没有记录" "$cell/out" && (($(wc -l < "$cell/calls") == shard_count + 1 + 2))'
  cell="$work/swap"; make_tree "$cell"
  swap_target="$cell/tree/scripts/experiment-apparatus/target/release/e10-aging-compaction"
  original_sha=$(sha256sum -- "$swap_target" | cut -d' ' -f1)
  E10_SELFTEST_SWAP_TARGET="$swap_target" invoke "$cell"; exit_code=$?
  expect "装置在一趟里不许换：跑着的时候编译目录里那份被换掉，之后起的二十一片自报的 sha256 与片旁记的都是开跑时那份" '((exit_code == 0)) && [[ -e "$swap_target.swapped" ]] && [[ "$(sha256sum -- "$swap_target" | cut -d" " -f1)" != "$original_sha" ]] && [[ "$(grep -v -- "--merge" "$cell/calls.self-sha" | awk "{print \$NF}" | sort -u)" == "$original_sha" ]] && (($(grep -vc -- "--merge" "$cell/calls.self-sha") == shard_count)) && [[ "$(cat "$cell/tree/$shard_directory"/*.binary-sha256 | sort -u)" == "$original_sha" ]]'
  cell="$work/merge-check"; make_tree "$cell"
  E10_SELFTEST_TAMPER_STAMP="$cell/tree/$shard_directory/main-s4.out.binary-sha256" invoke "$cell"; exit_code=$?
  expect "合并之前核二十一片的装置 sha256：有一片对不上就拒（退 1、点名那一片、装置 --merge 不起、不出产物）" '((exit_code == 1)) && grep -qF "片 main-s4 的装置 sha256 对不上" "$cell/out" && grep -qF "合并产物之前核全部片的装置 sha256" "$cell/out" && (($(wc -l < "$cell/calls") == shard_count)) && [[ ! -e "$cell/tree/$product" ]]'
  cell="$work/shipped"; make_tree "$cell"
  original_sha=$(sha256sum -- "$cell/shipped/e10-aging-compaction" | cut -d' ' -f1)
  E10_BINARY="$cell/shipped/e10-aging-compaction" E10_SHIPPED_BINARY_SHA256="$original_sha" invoke "$cell" --shards main-s4,main-s8; exit_code=$?
  expect "别台送来的装置 sha256 对得上：不 cargo build、放 python3 垫片、点名的两片现跑、片旁记着送来那份的 sha256" '((exit_code == 3)) && [[ ! -e "$cell/calls.cargo" ]] && [[ -x "$(ls "$cell"/logs/python3-path-shim-*/python3)" ]] && (($(wc -l < "$cell/calls") == 2)) && [[ "$(cat "$cell/tree/$shard_directory"/*.binary-sha256 | sort -u)" == "$original_sha" ]]'
  cell="$work/shipped-mismatch"; make_tree "$cell"
  echo "# corrupted in transfer" >> "$cell/shipped/e10-aging-compaction"
  E10_BINARY="$cell/shipped/e10-aging-compaction" E10_SHIPPED_BINARY_SHA256="$original_sha" invoke "$cell" --shards main-s4; exit_code=$?
  expect "别台送来的装置 sha256 对不上：退 1、一片不跑" '((exit_code == 1)) && grep -qF "不是同一份" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  cell="$work/list"; make_tree "$cell"
  invoke "$cell"; first_exit=$?
  echo "# a newer build of the device" >> "$cell/tree/scripts/experiment-apparatus/target/release/e10-aging-compaction"
  invoke "$cell" --list-items; exit_code=$?
  expect "--list-items：装置换了之后，旧装置打的片标出续跑时作废" '((first_exit == 0 && exit_code == 0)) && (($(grep -c "做完（装置 sha256 与现在这份不同，续跑时作废）" "$cell/out") == shard_count))'
  cell="$work/partial"; make_tree "$cell"
  invoke "$cell" --shards main-s4,main-s8; exit_code=$?
  expect "--shards 点名的两片现跑、不合并、退 3" '((exit_code == 3)) && (($(wc -l < "$cell/calls") == 2)) && [[ ! -e "$cell/tree/$product" ]]'
  cell="$work/truncated"; make_tree "$cell"
  E10_SELFTEST_DEVICE=truncated invoke "$cell"; exit_code=$?
  expect "末行不齐的片输出不进片目录：退 1、没有片文件、不合并" '((exit_code == 1)) && (($(ls "$cell/tree/$shard_directory"/*.out 2> /dev/null | wc -l) == 0)) && [[ ! -e "$cell/tree/$product" ]]'
  cell="$work/bad-argument"; make_tree "$cell"
  invoke "$cell" --bogus; exit_code=$?
  expect "认不得的参数退 2" '((exit_code == 2)) && grep -qF "认不出的参数：--bogus" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  invoke "$cell" --shards nonsense; exit_code=$?
  expect "不认得的片名退 2" '((exit_code == 2)) && grep -qF "不认得的片名" "$cell/out" && [[ ! -e "$cell/calls" ]]'
  rm -rf -- "${work:?}"
  if ((failures > 0)); then echo "e10-aging-compaction-run.sh 自证：$failures 格红（共 $cell_count 格）"; return 1; fi
  echo "e10-aging-compaction-run.sh 自证：$cell_count 格全绿"
  return 0
}

selected=()
case "${1:-}" in
  "") mapfile -t selected < <(all_shard_names) ;;
  --list-items)
    # 装置程序文件在就算它的 sha256，片旁记的不是这份的标出续跑时作废；不在就只看片末行
    PINNED_BINARY_SHA256=""
    [[ -x "$BINARY" ]] && PINNED_BINARY_SHA256=$(binary_identity_sha256_of "$BINARY")
    while read -r name; do
      if ! is_complete_output "$SHARD_DIRECTORY/$name.out"; then echo "$name 没做"
      elif [[ -z "$PINNED_BINARY_SHA256" ]]; then echo "$name 做完（装置没编好，判不了是哪一版）"
      elif binary_identity_is_current "$SHARD_DIRECTORY/$name.out"; then echo "$name 做完"
      else echo "$name 做完（装置 sha256 与现在这份不同，续跑时作废）"; fi
    done < <(all_shard_names)
    exit 0 ;;
  --selftest)
    run_selftest
    exit $? ;;
  --shards)
    [[ -n "${2:-}" ]] || { echo "  ✗ --shards 后面缺片名（逗号分隔）" >&2; echo "     → 怎么办：--list-items 列出可点名的片，几片用逗号隔开写在 --shards 后面" >&2; exit 2; }
    IFS=',' read -r -a selected <<< "$2"
    known_shard_names="$(all_shard_names)"
    for name in "${selected[@]}"; do
      grep -qxF -- "$name" <<<"$known_shard_names" || { echo "  ✗ 不认得的片名「$name」" >&2; echo "     → 怎么办：bash scripts/experiment-apparatus/e10-aging-compaction-run.sh --list-items 列出可点名的片，照它写" >&2; exit 2; }
    done ;;
  *) echo "  ✗ 认不出的参数：$1" >&2; echo "     → 怎么办：照用法写：e10-aging-compaction-run.sh [--shards <片名,…> | --list-items | --selftest]" >&2; exit 2 ;;
esac

mkdir -p -- "$SHARD_DIRECTORY" "$LOG_DIRECTORY" || exit 1
if [[ -z "${E10_BINARY:-}" ]]; then
  cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml --bin e10-aging-compaction > /dev/null 2> "$SHARD_DIRECTORY.build.log" \
    || { echo "  ✗ 装置编不过" >&2; echo "     → 怎么办：看构建日志 $SHARD_DIRECTORY.build.log 里第一处 error，修好装置再起同一条命令" >&2; exit 1; }
  rm -f -- "$SHARD_DIRECTORY.build.log"
fi
# 开跑时拷一份私有副本：每片、合并都按这份拷贝起
SOURCE_BINARY="$BINARY"
binary_identity_pin "$SOURCE_BINARY" "$LOG_DIRECTORY/pinned-binary" || exit 1
BINARY="$PINNED_BINARY"
LAUNCH_BINARY="$BINARY"
break_is exec-root-binary && LAUNCH_BINARY="$SOURCE_BINARY"
if [[ -n "${E10_SHIPPED_BINARY_SHA256:-}" ]]; then
  # 别台编好送来的装置：核 sha256，并放 python3 垫片（装置的准入判法按编译机的绝对路径找 preflight.py 与源文件）
  if [[ "$PINNED_BINARY_SHA256" != "$E10_SHIPPED_BINARY_SHA256" ]]; then
    echo "  ✗ 送到的装置程序文件 sha256 是 $PINNED_BINARY_SHA256，本机的是 $E10_SHIPPED_BINARY_SHA256，不是同一份：这一份一片都不跑" >&2
    echo "     → 怎么办：看 multi-host-run.sh 的 --send 有没有送全（盘满、传输中断）；重起本机那条命令" >&2
    exit 1
  fi
  binary_identity_install_python_shim "$DEVICE_SOURCE_PATH" "$LOG_DIRECTORY/python3-path-shim-$$" || exit 1
fi
THREADS=$(thread_count) || { echo "  ✗ 取不到装置的线程数（原因见上一行）" >&2; echo "     → 怎么办：设 E10_THREADS，或照上一行的出路修好 .claude/heavy-thread-default" >&2; exit 2; }
echo "e10 分片跑法：线程 $THREADS、每片内存上限 $MEMORY_CAP、片目录 $SHARD_DIRECTORY、装置 sha256 $PINNED_BINARY_SHA256（私有副本 $PINNED_BINARY，源 $SOURCE_BINARY）"
void_directory="$LOG_DIRECTORY/void-$(TZ=Asia/Tokyo date +%F)-$$"
failed=0
for name in "${selected[@]}"; do
  if is_complete_output "$SHARD_DIRECTORY/$name.out"; then
    # 做完了还要是这一趟的装置打的；记的不同或没有记录的整份移到作废目录，逐片报作废，现跑
    if break_is stale-binary-reused || binary_identity_is_current "$SHARD_DIRECTORY/$name.out"; then
      echo "片 $name：已做完，复用"
      continue
    fi
    binary_identity_void_shard "$name" "$SHARD_DIRECTORY/$name.out" "$void_directory" || exit 1
  fi
  run_shard "$name" || failed=$((failed + 1))
done
[[ "$failed" == 0 ]] || { echo "  ✗ $failed 片跑失败" >&2; echo "     → 怎么办：看上面那几片各自的日志，修好之后同一条命令重起，只补没做完的片" >&2; exit 1; }
[[ -z "${1:-}" ]] || exit 3

shard_files=()
while read -r name; do
  is_complete_output "$SHARD_DIRECTORY/$name.out" || { echo "片还没齐：缺 $name" >&2; exit 3; }
  shard_files+=("$SHARD_DIRECTORY/$name.out")
done < <(all_shard_names)
if is_complete_output "$PRODUCT"; then
  echo "产物已在：$PRODUCT（不重写）"
  exit 0
fi
[[ ! -e "$PRODUCT" ]] || { echo "  ✗ $PRODUCT 在而收尾行不对，不覆盖" >&2; echo "     → 怎么办：交主 agent 看那份产物；要重跑就设 E10_RUN_DATE=<跑的那天> 写一份新产物，不盖旧的" >&2; exit 1; }
# 合并之前再核一遍：二十一片片旁记的装置 sha256 都得是这一趟的，不同或没有记录就拒、不出产物
if ! break_is merge-sha-unchecked; then
  checked_pairs=()
  for file in "${shard_files[@]}"; do checked_pairs+=("$(basename -- "$file" .out)" "$(binary_identity_stamp_read "$file")"); done
  binary_identity_check_same "$PINNED_BINARY_SHA256" "${checked_pairs[@]}" || exit 1
fi
temporary=$(mktemp -d)
bash scripts/hosts/run-with-memory-cap.sh "$MEMORY_CAP" "$BINARY" --merge "${shard_files[@]}" > "$temporary/out" 2> "$temporary/err" \
  && is_complete_output "$temporary/out" \
  || { echo "  ✗ 合并失败" >&2; echo "     → 怎么办：看输出 $temporary 里的报错，修好装置的合并那一步再起同一条命令（做完的片复用）" >&2; exit 1; }
mv -- "$temporary/out" "$PRODUCT"
rm -rf -- "$temporary"
echo "产物：$PRODUCT（$(wc -l < "$PRODUCT") 行，sha256 $(sha256sum "$PRODUCT" | cut -d' ' -f1)）"
