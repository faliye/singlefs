#!/usr/bin/env bash
# L2：G10 改后的 --tool-versions（安装目录顶层全部文件的摘要、VERUS_Z3_PATH 未设与空串分开、裸名照 PATH 找）罩不罩得住 Verus 真读的东西，摘要稳不稳。
# 直接起的 Verus 只验一份三行的玩具文件（lib_heavy_tests.py 文件头：直接起的 verus 不算重型），每次经内存包装 400M 与 capped.sh 4；
# admission.py 只调判定子命令（stage-marker-write、stage-marker-check、gate-reuse）与它经 environment= 起的阶段 --tool-versions；不跑门禁、不跑判法脚本的干活用法。
# git 写只落在仓拷贝自己新建的 .git 里。安装拷贝 A、B 是安装顶层普通文件的硬链接（B 的宏库删链接后另写一份截断的，不碰原 inode）。
# 用法：bash l2-install.sh <仓拷贝（rsync 出来、没有 .git）> <草稿目录>
set -uo pipefail
COPY="${1:?仓拷贝}"; W="${2:?草稿目录}"; mkdir -p "$W/toy" "$W/empty-rustup-home"
STAGE=checker-tier-formal-verification.sh
INSTALL="${XDG_DATA_HOME:-$HOME/.local/share}/$(awk -F'\t' '$1=="release"{print $5}' "$COPY/.claude/gate.d/verus-pins.tsv")"
printf 'use vstd::prelude::*;\nverus! {\nproof fn one_plus_one() ensures 1 + 1 == 2 {}\n}\n' > "$W/toy/toy.rs"
if [[ ! -d "$W/A" ]]; then
  mkdir -p "$W/A" "$W/B"
  find "$INSTALL" -maxdepth 1 -type f -exec ln {} "$W/A/" \;
  find "$INSTALL" -maxdepth 1 -type f -exec ln {} "$W/B/" \;
  rm "$W/B/libverus_builtin_macros.so"; head -c 4096 "$INSTALL/libverus_builtin_macros.so" > "$W/B/libverus_builtin_macros.so"
fi
toy() {  # $1 标签，其余是 env 参数；可在前面加 STRACE=<文件>
  local label="$1"; shift
  local trace=(); [[ -n "${STRACE:-}" ]] && trace=(strace -f -qq -e trace=openat,open,execve -o "$STRACE")
  ( cd "$W/toy" && env "$@" RUN_WITH_MEMORY_CAP_KEY="r3-opus-l2-toy-$label" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="玩具 Verus 在几种环境下换着跑，失败的那几种峰值很小、不代表正常峰值" \
      RUN_WITH_MEMORY_CAP_PARALLEL_REASON="攻方腿的玩具 Verus 探针，与同会话别的腿各跑各的" nice -n 19 bash "$COPY/research/scripts/run-with-memory-cap.sh" 400M \
      bash "$COPY/research/scripts/capped.sh" 4 "${trace[@]}" "$INSTALL/verus" --no-cheating --crate-type=lib --crate-name toy --num-threads 2 --output-json --rlimit 10 toy.rs \
      > "$W/$label.out" 2> "$W/$label.err" ); local rc=$?
  printf '  %s\t退 %s\t%s\tstderr 末行：%s\n' "$label" "$rc" "$(grep -o '"verified": [0-9]*' "$W/$label.out" | head -1)" "$(grep -v '^\s*$' "$W/$label.err" | grep -v run-with-memory-cap | tail -1 | cut -c1-160)"
}
echo "== 1. Verus 读了安装里哪些文件（strace，原样环境）=="
STRACE="$W/strace-plain.txt" toy S0-原样-strace -u VERUS_ROOT -u VERUS_Z3_PATH
opened() { grep -v ENOENT "$1" | grep -E 'openat|execve' | grep -oE '"/[^"]+"' | tr -d '"' | sort -u; }
printf '  安装顶层被打开或执行的文件：%s 个：%s\n' "$(opened "$W/strace-plain.txt" | grep -cE "^$INSTALL/(\./)?[^/]+$")" "$(opened "$W/strace-plain.txt" | grep -E "^$INSTALL/(\./)?[^/]+$" | sed "s#^$INSTALL/##" | tr '\n' ' ')"
printf '  安装子目录里被打开的文件：%s 个\n' "$(opened "$W/strace-plain.txt" | grep -E "^$INSTALL/" | grep -vE "^$INSTALL/(\./)?[^/]+$" | grep -c .)"
printf '  安装之外 ~/.rustup 下被打开的文件：%s 个（含 %s）\n' "$(opened "$W/strace-plain.txt" | grep -c "^$HOME/.rustup/")" "$(opened "$W/strace-plain.txt" | grep "^$HOME/.rustup/" | grep -oE 'librustc_driver[^/]*|libLLVM[^/]*|settings.toml' | sort -u | tr '\n' ' ')"
printf '  往安装目录里写（O_WRONLY / O_RDWR / O_CREAT）：%s 次\n' "$(grep -E "\"$INSTALL[^\"]*\", [^)]*(O_WRONLY|O_RDWR|O_CREAT)" "$W/strace-plain.txt" | grep -c .)"
STRACE="$W/strace-root-A.txt" toy S1-VERUS_ROOT=A-strace -u VERUS_Z3_PATH VERUS_ROOT="$W/A"
printf '  VERUS_ROOT=A 时从 A 打开的文件：%s 个：%s；从原安装打开的：%s 个：%s\n' "$(opened "$W/strace-root-A.txt" | grep -c "^$W/A/")" "$(opened "$W/strace-root-A.txt" | grep "^$W/A/" | sed "s#^$W/A/##" | tr '\n' ' ')" \
  "$(opened "$W/strace-root-A.txt" | grep -c "^$INSTALL/")" "$(opened "$W/strace-root-A.txt" | grep "^$INSTALL/" | sed "s#^$INSTALL/##" | tr '\n' ' ')"
echo "== 2. 环境变量换了，Verus 判不判得出来 =="
toy T1-原样 -u VERUS_ROOT -u VERUS_Z3_PATH
toy T2-VERUS_ROOT=B -u VERUS_Z3_PATH VERUS_ROOT="$W/B"
toy T3-VERUS_ROOT=A -u VERUS_Z3_PATH VERUS_ROOT="$W/A"
toy T4-VERUS_USE_RUSTUP=0 -u VERUS_ROOT -u VERUS_Z3_PATH VERUS_USE_RUSTUP=0
toy T5-RUSTUP_HOME=空目录 -u VERUS_ROOT -u VERUS_Z3_PATH RUSTUP_HOME="$W/empty-rustup-home"
echo "== 3. tool-versions 那一行（摘要前 16 位）稳不稳、几种环境下变不变，单次挂钟 =="
# 经阶段 --tool-versions 打（与 environment= 起的同一条）；直接写 verus-proof-check.py tool-versions 加带 $ 的仓根，闸按「参数带 $」拒（文件头写的接受的误拒）
tv() { bash "$COPY/.claude/gate.d/checker-tier-formal-verification.sh" --tool-versions | sha256sum | cut -c1-16; }
for i in 1 2 3; do s=$(date +%s.%N); h=$(env -u VERUS_ROOT -u VERUS_Z3_PATH bash -c "$(declare -f tv); COPY='$COPY' tv"); e=$(date +%s.%N); printf '  第 %s 次：%s（%.2f 秒）\n' "$i" "$h" "$(echo "$e - $s" | bc)"; done
toy T6-再跑一次玩具 -u VERUS_ROOT -u VERUS_Z3_PATH >/dev/null
printf '  玩具 Verus 跑过之后：%s\n' "$(env -u VERUS_ROOT -u VERUS_Z3_PATH bash -c "$(declare -f tv); COPY='$COPY' tv")"
for spec in "VERUS_ROOT=$W/B" "VERUS_USE_RUSTUP=0" "RUSTUP_HOME=$W/empty-rustup-home" "VERUS_Z3_PATH="; do
  printf '  %s：%s\n' "${spec/$W/<草稿>}" "$(env -u VERUS_ROOT -u VERUS_Z3_PATH "$spec" bash -c "$(declare -f tv); COPY='$COPY' tv")"
done
echo "== 4. 复用判定：判绿时原样，之后换环境（q1 stage-marker-check 退 0 = 复用；q3 gate-reuse 退 10 = 复用）=="
G=(git -C "$COPY" -c user.name=scratch -c user.email=scratch@invalid)
if [[ ! -d "$COPY/.git" ]]; then "${G[@]}" init -q && "${G[@]}" add -A >/dev/null 2>&1 && "${G[@]}" commit -q -m base; fi
COMMON="$("${G[@]}" rev-parse --path-format=absolute --git-common-dir)"; TREE="$("${G[@]}" write-tree)"
reset_state() { rm -f "$COMMON"/singlefs-stage-green."$STAGE".* "$COMMON/singlefs-gate-environment.$STAGE"; "${G[@]}" update-ref -d refs/sop/staged-green 2>/dev/null || true; }
pair() {  # $1 标签 $2 之后问的 env 参数
  reset_state
  env -u SINGLEFS_STAGED_TREE -u VERUS_ROOT -u VERUS_Z3_PATH python3 "$COPY/research/scripts/admission.py" stage-marker-write "$COPY" "$STAGE" >/dev/null
  local rc1 rc3
  env -u SINGLEFS_STAGED_TREE -u VERUS_ROOT -u VERUS_Z3_PATH $2 python3 "$COPY/research/scripts/admission.py" stage-marker-check "$COPY" "$STAGE" >/dev/null; rc1=$?
  env -u VERUS_ROOT -u VERUS_Z3_PATH $2 SINGLEFS_STAGED_TREE="$TREE" python3 "$COPY/research/scripts/admission.py" gate-reuse "$COPY" "$STAGE" >/dev/null; rc3=$?
  printf '  %s\tq1 退 %s\tq3 退 %s\n' "$1" "$rc1" "$rc3"
}
pair "对照 不换" "SINGLEFS_R3_NOOP=1"
pair "对照 Y1 VERUS_Z3_PATH 未设 → 空串（G10 修的那一格）" "VERUS_Z3_PATH="
pair "Z1 VERUS_ROOT 未设 → 拷贝 B（宏库截断，T2 编不过）" "VERUS_ROOT=$W/B"
pair "Z2 VERUS_USE_RUSTUP 未设 → 0（T4 起不来）" "VERUS_USE_RUSTUP=0"
pair "Z3 RUSTUP_HOME 换成空目录（T5）" "RUSTUP_HOME=$W/empty-rustup-home"
reset_state
