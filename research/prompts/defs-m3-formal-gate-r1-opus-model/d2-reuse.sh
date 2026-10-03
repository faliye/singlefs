#!/usr/bin/env bash
# D2：换了 Verus、输入没变时，admission.py 的复用判定判什么。只调 admission.py 的判定子命令（gate-reuse、stage-marker-check、
# stage-marker-write、gate-record-environment）与 verus-proof-check.py tool-versions；不跑门禁、不起 Verus、不编。
# 用法：bash d2-reuse.sh <仓根> <草稿目录>
#   在草稿目录里拷一份仓（不带 target、.git），自己 git init 一份，所有 git 写只落在这份拷贝里。
#   假的 Verus 安装只有 version.json、verus、z3 三个文件：tool-versions 只读前两样（verus-proof-check.py 的 tool_versions）。
# 前缀（固定）：一次判绿把标记与环境记下（两种历史：H1 崩溃验证员在工作区判绿，H2 gate.sh --staged 里真跑判绿），之后输入一字不改。
# 放开扫的（用户与流程决定的）：换 Verus 的方式 × 下一次由哪条路问复用 × 标记在不在。
set -uo pipefail
REPO="${1:?仓根}"; WORK="${2:?草稿目录}"
STAGE=checker-tier-formal-verification.sh
COPY="$WORK/repo"
G=(git -C "$COPY" -c user.name=scratch -c user.email=scratch@invalid)
if [[ ! -d "$COPY/.git" ]]; then
  mkdir -p "$COPY"
  rsync -a --exclude target --exclude .git "$REPO/" "$COPY/"
  "${G[@]}" init -q && "${G[@]}" add -A >/dev/null 2>&1 && "${G[@]}" commit -q -m base
fi
echo "拷贝里的 admission.py、stage-inputs.tsv 与仓里的逐字节比："
for f in research/scripts/admission.py .claude/gate.d/stage-inputs.tsv research/scripts/verus-proof-check.py .claude/gate.d/$STAGE; do
  cmp -s "$REPO/$f" "$COPY/$f" && echo "  相同 $f" || echo "  不同 $f"
done
COMMON="$("${G[@]}" rev-parse --path-format=absolute --git-common-dir)"
TREE="$("${G[@]}" write-tree)"

make_install() {  # 目录 version commit verus 内容 z3 内容
  mkdir -p "$1"
  printf '{"verus":{"version":"%s","commit":"%s","toolchain":"1.98.1-x86_64-unknown-linux-gnu"}}\n' "$2" "$3" > "$1/version.json"
  printf '#!/bin/sh\n# %s\n' "$4" > "$1/verus"; chmod +x "$1/verus"
  printf '#!/bin/sh\n# %s\n' "$5" > "$1/z3"; chmod +x "$1/z3"
}
make_install "$WORK/verus-A" 0.2026.09.20.aef82ed aef82eda71838deef4a8cd0260fda6c9504be9b9 launcher-0920 z3-4.12.5
make_install "$WORK/verus-B" 0.2026.10.15.0000001 1111111111111111111111111111111111111111 launcher-1015 z3-4.15.0
make_install "$WORK/verus-C" 0.2026.09.20.aef82ed aef82eda71838deef4a8cd0260fda6c9504be9b9 launcher-0920 z3-4.15.0
echo "tool-versions（environment= 那一条打的东西）："
for v in A B C; do
  printf '  %s: ' "$v"; SINGLEFS_VERUS_ROOT="$WORK/verus-$v" python3 "$COPY/research/scripts/verus-proof-check.py" tool-versions "$COPY"
done

adm() { python3 "$COPY/research/scripts/admission.py" "$@"; }
reset_state() {
  rm -f "$COMMON"/singlefs-stage-green."$STAGE".* "$COMMON/singlefs-gate-environment.$STAGE"
  "${G[@]}" update-ref -d refs/sop/staged-green 2>/dev/null || true
}
advance_green() {  # 照 gate-staged.sh 第 46、47 行：整轮全绿之后前移
  local c; c="$("${G[@]}" commit-tree "$TREE" -m "gate --staged 整轮全绿")"; "${G[@]}" update-ref refs/sop/staged-green "$c"
}
green_run() {  # $1 = H1 | H2：照阶段第 144、147 行，判绿之后先记环境、再写全绿标记
  reset_state
  if [[ "$1" == H1 ]]; then
    ( unset SINGLEFS_STAGED_TREE; export SINGLEFS_VERUS_ROOT="$WORK/verus-A"
      echo "  [$1 判绿] gate-record-environment：$(adm gate-record-environment "$COPY" "$STAGE")（空 = 一个字都没打）"
      echo "  [$1 判绿] stage-marker-write → $(adm stage-marker-write "$COPY" "$STAGE" | sed "s#$COMMON#<common-dir>#")" )
    # 之后 gate-triage 的 gate.sh --staged：阶段先问复用，标记命中就整道退 77；整轮全绿前移 ref
    ( export SINGLEFS_STAGED_TREE="$TREE" SINGLEFS_VERUS_ROOT="$WORK/verus-A"
      out="$(adm gate-reuse "$COPY" "$STAGE")"; echo "  [$1 整轮] gate-reuse 退 $? ：${out}" )
  else
    ( export SINGLEFS_STAGED_TREE="$TREE" SINGLEFS_VERUS_ROOT="$WORK/verus-A"
      echo "  [$1 判绿] gate-record-environment：$(adm gate-record-environment "$COPY" "$STAGE" | sed "s#$COMMON#<common-dir>#")"
      echo "  [$1 判绿] stage-marker-write → $(adm stage-marker-write "$COPY" "$STAGE" | sed "s#$COMMON#<common-dir>#")" )
  fi
  advance_green
}
query() {  # $1 历史 $2 换法 $3 SINGLEFS_VERUS_ROOT $4 额外环境（VERUS_Z3_PATH=… 或空）
  local out rc
  out="$(env -u SINGLEFS_STAGED_TREE SINGLEFS_VERUS_ROOT="$3" $4 python3 "$COPY/research/scripts/admission.py" stage-marker-check "$COPY" "$STAGE")"; rc=$?
  printf '%s\t%s\tq1 崩溃验证员第 1 步 stage-marker-check\t退 %s（0 = 复用）\t%s\n' "$1" "$2" "$rc" "$(echo "$out" | head -1 | sed "s#$COMMON#<common-dir>#")"
  out="$(env -u SINGLEFS_STAGED_TREE SINGLEFS_VERUS_ROOT="$3" $4 python3 "$COPY/research/scripts/admission.py" gate-reuse "$COPY" "$STAGE")"; rc=$?
  printf '%s\t%s\tq2 阶段自己问（工作区，没有暂存树）\t退 %s（10 = 复用）\t%s\n' "$1" "$2" "$rc" "$(echo "$out" | head -1 | sed "s#$COMMON#<common-dir>#")"
  out="$(env SINGLEFS_STAGED_TREE="$TREE" SINGLEFS_VERUS_ROOT="$3" $4 python3 "$COPY/research/scripts/admission.py" gate-reuse "$COPY" "$STAGE")"; rc=$?
  printf '%s\t%s\tq3 gate.sh --staged 里阶段问\t退 %s（10 = 复用）\t%s\n' "$1" "$2" "$rc" "$(echo "$out" | head -1 | sed "s#$COMMON#<common-dir>#")"
}
sweep() {  # $1 历史标签
  query "$1" "不换（A）" "$WORK/verus-A" ""
  query "$1" "换版本（B：version、commit、verus 都变）" "$WORK/verus-B" ""
  query "$1" "只换 z3（C：version.json、verus 不变）" "$WORK/verus-C" ""
  query "$1" "A 加 VERUS_Z3_PATH 指别的 z3" "$WORK/verus-A" "VERUS_Z3_PATH=$WORK/verus-B/z3"
}
echo "== H1：崩溃验证员在工作区判绿（没有 SINGLEFS_STAGED_TREE），之后 gate-staged.sh 整轮全绿 =="
green_run H1
sweep H1
echo "== H2：gate.sh --staged 里这一道真跑、判绿（有 SINGLEFS_STAGED_TREE），整轮全绿 =="
green_run H2
sweep H2
echo "== H3：同 H2，之后全绿标记不在（过期、被清、或指纹没对上），staged-green 仍在 24 小时内 =="
green_run H2 >/dev/null
rm -f "$COMMON"/singlefs-stage-green."$STAGE".*
sweep H3
echo "== H4：同 H1，之后全绿标记不在，staged-green 仍在 24 小时内 =="
green_run H1 >/dev/null
rm -f "$COMMON"/singlefs-stage-green."$STAGE".*
sweep H4
reset_state
