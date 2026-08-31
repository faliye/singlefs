#!/usr/bin/env bash
# E6 的单元大小 × CPU 特性档：两档各跑一次虚机，合成一份产物，并判三条跨档判据。
#
#   bash research/scripts/e6-units.sh [轮数]     # 默认 5 轮
#
# 为什么要有驱动脚本：判据里有三条是**跨档**的（一档的数字单独看不出问题），
# 而二进制一次只跑一档。把判定留给人去心算，等于没有判据。
#
# ⚠️ **屏蔽 AES-NI 必须同时屏蔽 vaes。** `aes` 0.9.3 用 cpufeatures 分别探
# `aes` 与 `vaes` 两个 CPUID 位（`src/lib.rs` 第 152–154 行）。只写 `-aes` 时
# 来宾仍然看得见 VAES ⇒ **跑得和有 AES-NI 一样快，而算出来的标签是错的**
# （2026-08-31 实测：tag=2112edc9… 而外部实现给 62d27233…）。
# 那一档的吞吐数字一个字都不能用，而它看起来完全正常——正是已知答案测试要拦的形态。
set -uo pipefail
cd "$(dirname "$0")/.."

ROUNDS="${1:-5}"
BIN=target/x86_64-unknown-linux-musl/release/e6-units
OUT="${E6_OUT:-results/e6-units-$(date +%Y-%m-%d).out}"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-e6units.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

command -v cargo >/dev/null || { echo "没有 cargo"; exit 2; }
cargo build --release --target x86_64-unknown-linux-musl --bin e6-units >/dev/null 2>&1 \
  || { echo "musl 构建失败：rustup target add x86_64-unknown-linux-musl"; exit 2; }

run_arm() { # run_arm <档名> <VM_CPU> <落点>
  local tag="$1" cpu="$2" dst="$3"
  echo "── 档 $tag（VM_CPU=$cpu）"
  VM_CPU="$cpu" bash scripts/vm-bench.sh "$BIN" "$ROUNDS" >"$dst" 2>&1
  local rc=$?
  grep -E '^E7RESULT' "$dst" >"$dst.res"
  if [[ $rc -ne 0 ]]; then
    echo "  ✗ 档 $tag 退出码 $rc —— 整轮作废"; sed -n '1,20p' "$dst"; return 1
  fi
  # 完整性闸：程序自报的条数必须等于抓到的条数（command-safety.md「结果抓取要有完整性闸」）
  local claimed got
  claimed="$(sed -n 's/.*name=done emitted=\([0-9]*\).*/\1/p' "$dst.res" | tail -1)"
  got="$(wc -l <"$dst.res")"
  if [[ -z "$claimed" || "$claimed" != "$got" ]]; then
    echo "  ✗ 档 $tag 抓到 $got 条，程序声称 ${claimed:-无} 条 —— 整轮作废"; return 1
  fi
  echo "  ✓ 档 $tag 抓到 $got 条"
}

run_arm ni   "host"             "$TMP/ni"   || exit 1
run_arm soft "host,-aes,-vaes"  "$TMP/soft" || exit 1

fld() { sed -n "s/.*\balg=$2 unit=$3 .*\b$4=\([0-9.]*\).*/\1/p" "$1" | tail -1; }

bad=0
echo "── 跨档判据"

# 判据 3（自带对照）：两档必须跑的是同一份计算，差别只许在快慢上。
# 它排的是「一档悄悄少算了一半」——那只会显得更快，吞吐上看不出来。
for u in 4096 16384 65536 262144; do
  for a in aes256gcm chacha20poly1305; do
    x="$(fld "$TMP/ni.res" "$a" "$u" tagchk)"; y="$(fld "$TMP/soft.res" "$a" "$u" tagchk)"
    if [[ -z "$x" || -z "$y" ]]; then echo "  ✗ $a/$u tagchk 读不到"; bad=1
    elif [[ "$x" != "$y" ]]; then echo "  ✗ $a/$u 两档 tagchk 不同（$x vs $y）—— 算的不是同一份东西"; bad=1; fi
  done
done
[[ $bad -eq 0 ]] && echo "  ✓ 自带对照：八格 tagchk 两档逐位相同"

# 判据 1（失败条款）：屏蔽后 AES 必须有可分辨的下降（≥2×）。
# 测不出 ⇒ CPU flag 没生效，整轮作废。
for u in 4096 16384 65536 262144; do
  x="$(fld "$TMP/ni.res" aes256gcm "$u" mibs)"; y="$(fld "$TMP/soft.res" aes256gcm "$u" mibs)"
  r="$(awk -v a="$x" -v b="$y" 'BEGIN{if(b>0) printf "%.2f", a/b; else print "NA"}')"
  if awk -v r="$r" 'BEGIN{exit !(r>=2)}'; then echo "  ✓ 失败条款 AES/$u 掉 ${r}×（$x → $y）"
  else echo "  ✗ 失败条款 AES/$u 只掉 ${r}×（要求 ≥2）—— CPU flag 没生效，整轮作废"; bad=1; fi
done

# 判据 2（阴性对照）：ChaCha 不使用 AES-NI ⇒ 每一档 |Δ| ≤ 5%。
# 它证明那个 flag 只作用在该作用的地方，不是整机变慢。
for u in 4096 16384 65536 262144; do
  x="$(fld "$TMP/ni.res" chacha20poly1305 "$u" mibs)"; y="$(fld "$TMP/soft.res" chacha20poly1305 "$u" mibs)"
  d="$(awk -v a="$x" -v b="$y" 'BEGIN{printf "%.2f", (b-a)/a*100}')"
  if awk -v d="$d" 'BEGIN{exit !(d<=5 && d>=-5)}'; then echo "  ✓ 阴性对照 ChaCha/$u 变动 ${d}%"
  else echo "  ✗ 阴性对照 ChaCha/$u 变动 ${d}%（要求 ≤5%）—— 屏蔽把整机拖慢了，整轮作废"; bad=1; fi
done

{ echo "# E6 单元大小 × CPU 特性档，两档合成。ROUNDS=$ROUNDS"
  echo "# 档一 VM_CPU=host；档二 VM_CPU=host,-aes,-vaes（必须连 vaes 一起屏蔽，见脚本头注）"
  cat "$TMP/ni.res"; cat "$TMP/soft.res"; } >"$OUT"
echo "── 产物：$OUT"
[[ $bad -eq 0 ]] || { echo "✗ 有判据没过，上面那份产物不许引用"; exit 1; }
echo "✓ 三条跨档判据全过"
