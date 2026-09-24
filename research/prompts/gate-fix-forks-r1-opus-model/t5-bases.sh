#!/usr/bin/env bash
# T5 模型：同一个仓、同一时刻，门禁里五种「改动范围基准」取法各取到哪个提交。
# 取法原样照抄自真仓（行号见报告）；现场建在 $2（草稿目录）下的临时 git 仓里。用法：bash t5-bases.sh <真仓根> <草稿目录>
set -uo pipefail
REPO="$(cd "$1" && pwd)"; DRAFT="$2"
work="$(mktemp -d "$DRAFT/t5.XXXXXX")"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_AUTHOR_NAME=m GIT_AUTHOR_EMAIL=m@x.invalid GIT_COMMITTER_NAME=m GIT_COMMITTER_EMAIL=m@x.invalid
unset GATE_BASE
git init -q --bare "$work/origin.git"
git clone -q "$work/origin.git" "$work/clone" 2>/dev/null
cd "$work/clone" || exit 2
git checkout -q -b master 2>/dev/null
for i in 1 2 3; do echo "$i" > f; git add f; git commit -qm "c$i"; done
git push -q -u origin master 2>/dev/null
bases() {
  local name
  # ① 上游 lib.sh 的 diff_base（gate.sh 算好导出成 GATE_DIFF_BASE；show-me-test 用它）
  ( source "$REPO/.claude/singlefs-ai-sop/scripts/lib.sh" >/dev/null 2>&1; set +e; printf '  ① lib.sh diff_base（GATE_DIFF_BASE）：%s\n' "$(git log -1 --format=%s "$(diff_base .)")" )
  # ② research/scripts/changed-paths.sh 的 gate 取法（11 号用 head，56/68/69/75/92/97 用 gate）
  ( source "$REPO/research/scripts/changed-paths.sh"; printf '  ② changed-paths.sh gate：%s\n' "$(git log -1 --format=%s "$(gate_diff_base gate)")" )
  # ③ 61 号第 34–42 行
  ( BASE="${GATE_BASE:-}"; if [[ -z "$BASE" ]]; then for def in master main; do if git rev-parse --verify -q "$def" >/dev/null; then BASE="$(git merge-base HEAD "$def" 2>/dev/null)" && break; fi; done; fi; [[ -n "$BASE" ]] || BASE=HEAD
    printf '  ③ 61 号自带取法：%s\n' "$(git log -1 --format=%s "$BASE")" )
  # ④ 40 号第 86–89 行、88 号第 28–31 行（同一句）
  ( BASE="${GATE_BASE:-}"; if [[ -z "$BASE" ]]; then BASE="$(git rev-parse --verify --quiet refs/sop/gate-ok || git rev-parse --verify --quiet refs/singlefs/gate-ok || git rev-parse --verify --quiet '@{upstream}' || true)"; fi
    printf '  ④ 40/88 号自带取法：%s\n' "$( [[ -n "$BASE" ]] && git log -1 --format=%s "$BASE" || echo 空（判全部）)" )
}
echo "== 情形 A：全推出去了，工作区有改动"; echo x >> f; bases; git checkout -q f
echo "== 情形 B：本地多一个没推的提交 c4（第一次提交），工作区又改了（第二次）"; echo 4 > f; git add f; git commit -qm c4; echo y >> f; bases
echo "== 情形 C：同 B，另有上一轮整轮全绿记下的 refs/sop/gate-ok = c4"; git update-ref refs/sop/gate-ok HEAD; bases
echo "现场：$work"
