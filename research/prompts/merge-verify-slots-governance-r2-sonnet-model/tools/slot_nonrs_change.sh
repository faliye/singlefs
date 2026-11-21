#!/usr/bin/env bash
# 槽里只改非 .rs 输入（include_str! 读的数据文件、Cargo.toml）时：真 cargo 重不重编、slot-changed-sources 的 awk 收不收、lib_rebuilt_packages.py 的重编核对认不认；
# 再模拟「槽里的修改时刻没跟上」（等价于弄坏开关 slot-copy-keeps-old-times），看 .rs 与非 .rs 各被抓不抓。
# 用法：slot_nonrs_change.sh <运行目录> <仓根>
#   运行目录下建 slot-model/（主树 main/、槽 slot/、编译目录 target/），收尾整个删掉；只用 rsync、cargo --offline（一个无依赖的小包）、仓里的 lib_rebuilt_packages.py check。
# 每一步一行：name=slot_nonrs step=<步> <读数>。退出码：0 全部与预期相同；1 有不同；2 用法错或抄的那一行 awk 在仓里不见了。
set -euo pipefail
(($# == 2)) || { echo "用法：slot_nonrs_change.sh <运行目录> <仓根>" >&2; exit 2; }
work="$1"; repo="$2"
root="$work/slot-model"
finish() { mkdir -p "$work/out" && rm -rf -- "${work:?}/out/slot-model-logs" && cp -R -- "$root/logs" "$work/out/slot-model-logs"; rm -rf -- "${root:?}"; }
trap finish EXIT
rm -rf -- "${root:?}"
mkdir -p "$root/main/src" "$root/slot" "$root/logs"
export CARGO_INCREMENTAL=0 CARGO_TERM_COLOR=never CARGO_TARGET_DIR="$root/target"
lib="$repo/scripts/gate/merge-and-verify-snapshot-lib.sh"
checker="$repo/scripts/gate/lib_rebuilt_packages.py"
# 与 scripts/gate/merge-and-verify-snapshot-lib.sh refresh_slot_snapshot 末尾同一句 awk（逐字抄；下面核它还在仓里）
awk_program='{ code = substr($0, 1, 11); path = substr($0, 13) } (code ~ /^>f/ || code ~ /^\*deleting/) && path ~ /\.rs$/ { print path }'
if [[ "$(grep -cF -- "$awk_program" "$lib")" -lt 1 ]]; then
  echo "name=slot_nonrs step=awk-text-in-repo hits=0 verdict=DIFFERENT（那一句 awk 在 $lib 里找不到：仓里改过，这份模型要对着新写法重抄）"
  exit 2
fi
echo "name=slot_nonrs step=awk-text-in-repo hits=1"
different=0
expect() {   # expect <步> <读到的> <预期>
  local flag=""
  [[ "$2" == "$3" ]] || { flag=" verdict=DIFFERENT"; different=1; }
  echo "name=slot_nonrs step=$1 got=$2 expected=$3$flag"
}
printf '[package]\nname = "slotdemo"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n' > "$root/main/Cargo.toml"
printf 'fn main() { print!("{}", include_str!("data.txt")); }\n' > "$root/main/src/main.rs"
printf 'v1\n' > "$root/main/src/data.txt"
# 与 refresh_slot_snapshot 同一组 rsync 参数：按内容比、不带时间、内容没变的不动
copy_into_slot() {   # copy_into_slot <日志前缀>
  rsync --recursive --links --checksum --no-times --executability --delete --itemize-changes \
    --exclude .git --exclude 'target/' --exclude 'target-*/' --exclude '__pycache__/' -- "$root/main/" "$root/slot/" > "$root/logs/$1.itemized"
  awk "$awk_program" "$root/logs/$1.itemized" > "$root/logs/$1.changed-rs"
}
compile_in_slot() {   # compile_in_slot <日志名> → 打这一趟 Compiling 行数
  (cd "$root/slot" && cargo build --offline > "$root/logs/$1.log" 2>&1)
  grep -c 'Compiling slotdemo' "$root/logs/$1.log" || true
}
check_rows() {   # check_rows <日志名> <相对路径> → 退出码（0 都重编了、1 有包没重编、77 本次未判）
  local rc=0
  python3 -I "$checker" check --tree "$root/slot" --target-dir "$root/target" --log "$root/logs/$1.log" --path "$2" > "$root/logs/$1.check-$(basename "$2").txt" 2>&1 || rc=$?
  echo "$rc"
}
copy_into_slot first
expect first-compile-lines "$(compile_in_slot first)" 1
sleep 1.1
copy_into_slot second
expect no-change-rsync-written "$(grep -c '^>f' "$root/logs/second.itemized" || true)" 0
expect no-change-compile-lines "$(compile_in_slot second)" 0
# 一、只改 include_str! 读的数据文件：rsync 写了它、修改时刻是此刻；awk 不收（不是 .rs）；cargo 照修改时刻重编；
#    就算把它交给重编核对（--path），files_by_package 也只认 .rs，退 77 本次未判
sleep 1.1
printf 'v2\n' > "$root/main/src/data.txt"
copy_into_slot data
expect data-rsync-written "$(grep -c '^>f' "$root/logs/data.itemized" || true)" 1
expect data-slot-changed-sources-lines "$(wc -l < "$root/logs/data.changed-rs" | tr -d ' ')" 0
expect data-compile-lines "$(compile_in_slot data)" 1
expect data-check-as-path-exit "$(check_rows data src/data.txt)" 77
# 二、改 Cargo.toml 的版本号：同样不是 .rs；cargo 的指纹认版本，重编
sleep 1.1
printf '[package]\nname = "slotdemo"\nversion = "0.1.1"\nedition = "2021"\n\n[dependencies]\n' > "$root/main/Cargo.toml"
copy_into_slot manifest
expect manifest-slot-changed-sources-lines "$(wc -l < "$root/logs/manifest.changed-rs" | tr -d ' ')" 0
expect manifest-compile-lines "$(compile_in_slot manifest)" 1
expect manifest-check-as-path-exit "$(check_rows manifest Cargo.toml)" 77
# 三、槽里的修改时刻没跟上（拷完把槽里的文件拨回很久以前，等价于弄坏开关 slot-copy-keeps-old-times）：cargo 当它没变、不重编，产物是旧的；
#    非 .rs（数据文件）：核对不判（77），旧产物判绿；.rs：核对点名「没重编」（1）
sleep 1.1
printf 'v3\n' > "$root/main/src/data.txt"
copy_into_slot stale-data
touch -d @946684800 "$root/slot/src/data.txt"
expect stale-data-compile-lines "$(compile_in_slot stale-data)" 0
expect stale-data-binary-prints-old "$("$root/target/debug/slotdemo" | tr -d '\n')" v2
expect stale-data-check-exit "$(check_rows stale-data src/data.txt)" 77
sleep 1.1
printf 'fn main() { print!("{}", include_str!("data.txt")); }\n// 改了一行注释\n' > "$root/main/src/main.rs"
copy_into_slot stale-rs
expect stale-rs-slot-changed-sources-lines "$(wc -l < "$root/logs/stale-rs.changed-rs" | tr -d ' ')" 1
touch -d @946684800 "$root/slot/src/main.rs"
expect stale-rs-compile-lines "$(compile_in_slot stale-rs)" 0
expect stale-rs-check-exit "$(check_rows stale-rs src/main.rs)" 1
echo "name=slot_nonrs step=end different=$different"
exit "$different"
