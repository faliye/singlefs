#!/usr/bin/env bash
# E2：F1 + F2 落地之后，环境变了而 tool-versions 那一行不变的几种，复用判定判什么；环境输出稳不稳。
# 只调草稿拷贝里 admission.py 的判定子命令（stage-fingerprint、stage-marker-write、stage-marker-check、gate-reuse）与它经
# environment= 起的阶段 --tool-versions；不跑门禁、不起 Verus、不编。git 写只落在拷贝自己的 .git 里。
# 用法：bash e2-reuse.sh <拷贝根（rsync 出来的、没有 .git）> <e2-verus-toy.sh 用过的草稿目录（里面有 verus-A、verus-B、path-z3）>
set -uo pipefail
COPY="${1:?拷贝根}"; TOY="${2:?玩具草稿目录}"
STAGE=checker-tier-formal-verification.sh
G=(git -C "$COPY" -c user.name=scratch -c user.email=scratch@invalid)
if [[ ! -d "$COPY/.git" ]]; then "${G[@]}" init -q && "${G[@]}" add -A >/dev/null 2>&1 && "${G[@]}" commit -q -m base; fi
COMMON="$("${G[@]}" rev-parse --path-format=absolute --git-common-dir)"
TREE="$("${G[@]}" write-tree)"
INSTALL="${XDG_DATA_HOME:-$HOME/.local/share}/$(awk -F'\t' '$1=="release"{print $5}' "$COPY/.claude/gate.d/verus-pins.tsv")"
mkdir -p "$TOY/path-z3-other"
if [[ ! -d "$TOY/verus-C" ]]; then cp -al "$TOY/verus-A" "$TOY/verus-C"; rm "$TOY/verus-C/version.json"; sed 's/aef82eda71838deef4a8cd0260fda6c9504be9b9/1111111111111111111111111111111111111111/' "$TOY/verus-A/version.json" > "$TOY/verus-C/version.json"; fi
printf '#!/bin/sh\n# 另一份 z3（内容不同）\nexec "%s/z3" "$@"\n' "$INSTALL" > "$TOY/path-z3-other/z3"; chmod +x "$TOY/path-z3-other/z3"
adm() { python3 "$COPY/research/scripts/admission.py" "$@"; }
line() {  # 打这一环境下 environment= 那一条的输出（与 admission 现跑的同一份）
  python3 - "$COPY" "$STAGE" <<'PY'
import sys; sys.path.insert(0, sys.argv[1] + "/research/scripts"); sys.dont_write_bytecode = True
import admission
rows = admission.rows_of_key(admission.read_registration_rows(sys.argv[1]), sys.argv[2])
lines, digest = admission.current_gate_environment(sys.argv[1], rows)
print(digest[:16], lines[0][1].replace(sys.argv[1], "<拷贝>"))
PY
}
reset_state() { rm -f "$COMMON"/singlefs-stage-green."$STAGE".* "$COMMON/singlefs-gate-environment.$STAGE"; "${G[@]}" update-ref -d refs/sop/staged-green 2>/dev/null || true; }
echo "== 稳不稳：同一环境连算三次指纹，再在一棵 git worktree 里算一次 =="
for i in 1 2 3; do echo "  第 $i 次：$(env -u VERUS_Z3_PATH python3 "$COPY/research/scripts/admission.py" stage-fingerprint "$COPY" "$STAGE")"; done
WT="$TOY/worktree"; rm -rf "$WT"; "${G[@]}" worktree prune; "${G[@]}" worktree add -q --detach "$WT" HEAD
echo "  worktree：$(env -u VERUS_Z3_PATH python3 "$WT/research/scripts/admission.py" stage-fingerprint "$WT" "$STAGE")"
"${G[@]}" worktree remove --force "$WT"
echo "== environment= 那一行（摘要前 16 位、第一行）在各环境下 =="
for spec in "U:未设 VERUS_Z3_PATH" "E:VERUS_Z3_PATH 空串" "P1:VERUS_Z3_PATH=z3、PATH 最前是记日志的 z3" "P2:VERUS_Z3_PATH=z3、PATH 最前是另一份 z3" "A:安装拷贝 A" "B:安装拷贝 B（宏库截断）"; do
  key="${spec%%:*}"
  case "$key" in
    U) out="$(env -u VERUS_Z3_PATH bash -c "$(declare -f line); COPY='$COPY' STAGE='$STAGE' line")" ;;
    E) out="$(env VERUS_Z3_PATH= bash -c "$(declare -f line); COPY='$COPY' STAGE='$STAGE' line")" ;;
    P1) out="$(env VERUS_Z3_PATH=z3 PATH="$TOY/path-z3:$PATH" bash -c "$(declare -f line); COPY='$COPY' STAGE='$STAGE' line")" ;;
    P2) out="$(env VERUS_Z3_PATH=z3 PATH="$TOY/path-z3-other:$PATH" bash -c "$(declare -f line); COPY='$COPY' STAGE='$STAGE' line")" ;;
    A) out="$(env -u VERUS_Z3_PATH SINGLEFS_VERUS_ROOT="$TOY/verus-A" bash -c "$(declare -f line); COPY='$COPY' STAGE='$STAGE' line")" ;;
    B) out="$(env -u VERUS_Z3_PATH SINGLEFS_VERUS_ROOT="$TOY/verus-B" bash -c "$(declare -f line); COPY='$COPY' STAGE='$STAGE' line")" ;;
  esac
  printf '  %s（%s）：%s\n' "$key" "${spec#*:}" "$out"
done
# 一段历史：崩溃验证员在工作区判绿（没有暂存树）写标记，环境 W；之后环境换成 Q，三条问法各问一次（q1 stage-marker-check 退 0 = 复用；q3 gate-reuse 退 10 = 复用）
pair() {  # $1 标签 $2 判绿时的 env 参数 $3 之后问的 env 参数
  reset_state
  env -u SINGLEFS_STAGED_TREE $2 python3 "$COPY/research/scripts/admission.py" stage-marker-write "$COPY" "$STAGE" >/dev/null
  local rc1 rc3
  env -u SINGLEFS_STAGED_TREE $3 python3 "$COPY/research/scripts/admission.py" stage-marker-check "$COPY" "$STAGE" >/dev/null; rc1=$?
  env $3 SINGLEFS_STAGED_TREE="$TREE" python3 "$COPY/research/scripts/admission.py" gate-reuse "$COPY" "$STAGE" >/dev/null; rc3=$?
  printf '%s\tq1 stage-marker-check 退 %s（0 = 复用）\tq3 gate.sh --staged 里 gate-reuse 退 %s（10 = 复用）\n' "$1" "$rc1" "$rc3"
}
echo "== 判绿时的环境 → 之后的环境 =="
pair "对照 不换（未设 → 未设）" "-u VERUS_Z3_PATH" "-u VERUS_Z3_PATH"
pair "对照 换安装拷贝 A → 原安装（文件逐字节相同）" "SINGLEFS_VERUS_ROOT=$TOY/verus-A" "-u VERUS_Z3_PATH"
pair "对照 版本变（拷贝 A → 拷贝 C：version.json 的 commit 改）" "SINGLEFS_VERUS_ROOT=$TOY/verus-A" "SINGLEFS_VERUS_ROOT=$TOY/verus-C"
pair "Y1 未设 → 空串（Verus 起不了 z3）" "-u VERUS_Z3_PATH" "VERUS_Z3_PATH="
pair "Y2 安装拷贝 A → 拷贝 B（宏库截断，Verus 编不过）" "SINGLEFS_VERUS_ROOT=$TOY/verus-A" "SINGLEFS_VERUS_ROOT=$TOY/verus-B"
pair "Y3 裸名 z3：PATH 里换一份 z3" "VERUS_Z3_PATH=z3 PATH=$TOY/path-z3:$PATH" "VERUS_Z3_PATH=z3 PATH=$TOY/path-z3-other:$PATH"
reset_state
