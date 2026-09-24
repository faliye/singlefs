#!/usr/bin/env bash
# T1：75 号 ⑤ 后一半（改成已跑 ⇒ 正文引了它的决策都要回看）与 10 号并进 ⑨，这两处改法自己的放过与误拦。
# 用法：bash t1-scenes.sh <快照树> <真仓> <草稿目录>
#   快照树里的 75 号与共用脚本是被判的那一版；真仓只用来取 HEAD 那一版的 75、10 号作对照（git show，只读）。
#   每一格从同一个基线仓（临时 git 仓，与本机 git 配置隔开）拷一份，做一批改动，跑一遍，贴退出码与点名的那一句。
set -uo pipefail
SNAP="$(cd "$1" && pwd)"; REPO="$(cd "$2" && pwd)"; D="$3"
rm -rf "$D"; mkdir -p "$D/head/.claude/gate.d"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE
S75="$SNAP/.claude/gate.d/75-decision-experiment-links.sh"
git -C "$REPO" show HEAD:.claude/gate.d/75-decision-experiment-links.sh > "$D/head/.claude/gate.d/75.sh"
git -C "$REPO" show HEAD:.claude/gate.d/10-kb-rot.sh > "$D/head/.claude/gate.d/10.sh"

decision() {  # $1 = 欠账块里 E2 的写法
  mkdir -p .claude/kb/decisions .claude/kb/experiments
  cat > .claude/kb/decisions/01-甲.md <<EOF
## D1 甲 —— 已定

### 已定项

| # | 分项 | 定案 |
|---|---|---|
| 1 | 样本分项 | 一句话定案 |

#### 已定项 1：样本分项

**定案**：样本规则。

**射程**：只管样本。

**依据**：E1（样本实验一） 证明了样本规则成立。

**欠**：等 $1 跑完再核射程；E5（样本实验五） 的结论也在这里等着复核。

## 历史版本
EOF
  cat > .claude/kb/decisions/03-丙.md <<'EOF'
## D3 丙 —— 已定

### 已定项

| # | 分项 | 定案 |
|---|---|---|
| 1 | 政策分项 | 不进样本主线 |

#### 已定项 1：政策分项

**定案**：不进样本主线。

**射程**：只管样本。

**依据**：无实验：纯政策定案，没有可量的量。

**欠**：无

## 历史版本
EOF
  : > .claude/kb/decisions.md
}
experiment() {  # 文件 号 简称 状态 表行
  printf '## E%s %s —— %s\n\n正文提到 D%s。\n\n### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n%s\n\n## 历史版本\n' \
    "$2" "$3" "$4" "$([[ "$5" == *D1* ]] && echo '1（甲）' || echo '3（丙）')" "$5" > ".claude/kb/experiments/$1"
}
base_world() {  # $1 = 目录，$2 = 欠账块里 E2 的写法
  mkdir -p "$1"; ( cd "$1" && git init -q -b master . && decision "$2"
    experiment 01-样本1.md 1 样本实验一 '已跑（2026-09-18）' '| D1（甲） 已定项 1 | 支撑 | 2026-09-18 不受影响：结论与定案同向 |'
    experiment 02-样本2.md 2 样本实验二 '未跑' '| D3（丙） | 备料 | 2026-09-18 不受影响：样本理由 |'
    experiment 05-样本5.md 5 样本实验五 '已跑（2026-09-18）' '| D3（丙） | 备料 | 2026-09-18 不受影响：样本理由 |'
    git add -A && git commit -qm base )
}
run75() {  # $1 = 被判的阶段，$2 = 仓；打印「退出码 | 第一条点名」
  local out rc=0
  out="$(nice -n 19 bash "$1" "$2" 2>&1)" || rc=$?
  printf '退 %s | %s' "$rc" "$(grep -m1 -oE '\[没回看引用它的决策\][^，]*，[^，]*|\[没对应决策\][^—]*|\[[^]]*\][^。]{0,60}|✓ 决策与实验双向登记对得上' <<<"$out" | head -1)"
}
flip() {  # E2 改成给定的状态，同时回看表里 D3 那一行（只回看别的决策）
  sed -i -e "s/^## E2 样本实验二 —— 未跑$/## E2 样本实验二 —— $1/" \
         -e 's/^| D3（丙） | 备料 | 2026-09-18 不受影响：样本理由 |$/| D3（丙） | 备料 | 2026-09-24 不受影响：结论不碰丙 |/' .claude/kb/experiments/02-样本2.md
}
fresh() { rm -rf "$D/w"; cp -a "$D/base-$1" "$D/w"; cd "$D/w" || exit 2; }
base_world "$D/base-std" 'E2（样本实验二）'
echo "基线（D1 欠账块等 E2、E5；E2 未跑、表里只有 D3；E5 已跑、表里只有 D3）：快照 75 $(run75 "$S75" "$D/base-std")"
echo
echo "== A：E2 未跑 → 某个「跑完了」的写法，只回看 D3 那一行，D1 没动（快照 75）"
for status in '已跑（2026-09-24）' '已跑两轮（2026-09-24）' '首轮已跑（2026-09-24）' '已测（2026-09-24）' \
              '**已跑**（2026-09-24）' '已跑 (2026-09-24)' '已跑：结论不碰甲' '已跑，结论不碰甲' '已跑。'; do
  fresh std; flip "$status"; printf '  %-26s %s\n' "「$status」" "$(run75 "$S75" "$D/w")"
done
echo
echo "== B：D1 欠账块换一种 doc-lint 认的写法提到 E2（doc-lint.sh 的 G 检查认粗体、反引号、空格、半角括号），E2 改成已跑（快照 75）"
for form in std:'E2（样本实验二）' bold:'**E2**（样本实验二）' code:'`E2`（样本实验二）' space:'E2 （样本实验二）' half:'E2(样本实验二)'; do
  name="${form%%:*}"; [[ "$name" == std ]] || base_world "$D/base-$name" "${form#*:}"
  printf '  %-6s 基线 %s；' "$name" "$(run75 "$S75" "$D/base-$name")"
  fresh "$name"; flip '已跑（2026-09-24）'; printf '改成已跑 %s\n' "$(run75 "$S75" "$D/w")"
done
echo
echo "== C：同一批里 D1 有一处与 E2 无关的改动（全仓术语改名这类），D1 欠账块照旧等 E2"
fresh std; flip '已跑（2026-09-24）'; sed -i 's/^\*\*定案\*\*：样本规则。$/**定案**：样本准则。/' .claude/kb/decisions/01-甲.md
echo "  C1 工作区直接跑：快照 75 $(run75 "$S75" "$D/w")"
fresh std; flip '已跑（2026-09-24）'; git add -A; sed -i 's/^\*\*定案\*\*：样本规则。$/**定案**：样本准则。/' .claude/kb/decisions/01-甲.md
echo "  C2 这一批暂存了翻状态，别的会话在工作区改了 D1 没暂存；工作区直接跑：快照 75 $(run75 "$S75" "$D/w")"
git worktree add -q --detach "$D/wt" HEAD && git diff --cached --binary | git -C "$D/wt" apply --index
echo "     照 gate.sh --staged 的做法在临时 worktree 上跑（GATE_BASE=HEAD）：快照 75 $(GATE_BASE="$(git rev-parse HEAD)" run75 "$S75" "$D/wt")"
git worktree remove --force "$D/wt"
echo
echo "== D：E5 早就已跑，D1 欠账块等它、E5 的表里没有 D1"
fresh std; git mv .claude/kb/experiments/05-样本5.md .claude/kb/experiments/05-样本五.md
echo "  D1 只把 E5 的页 git mv 换个文件名（path-moves 许可的搬家），一个字不改：快照 75 $(run75 "$S75" "$D/w")"
echo "                                                       HEAD 75 $(run75 "$D/head/.claude/gate.d/75.sh" "$D/w")"
fresh std; mv .claude/kb/experiments/05-样本5.md .claude/kb/experiments/05-样本五.md
echo "  D2 同上但用 mv、不暂存：快照 75 $(run75 "$S75" "$D/w")；HEAD 75 $(run75 "$D/head/.claude/gate.d/75.sh" "$D/w")"
sed -i 's/^| D3（丙） | 备料 | 2026-09-18 不受影响：样本理由 |$/&\n| D1（甲） | 不影响 | 2026-09-24 不受影响：只是搬家/' .claude/kb/experiments/05-样本五.md
echo "  D3 在 D2 上给 E5 的表补一行 D1（不影响）：快照 75 $(run75 "$S75" "$D/w")"
fresh std
sed -i -e 's/^正文提到 D3（丙）。$/正文提到 D3（丙），重跑之后结论翻了。/' \
       -e 's/^| D3（丙） | 备料 | 2026-09-18 不受影响：样本理由 |$/| D3（丙） | 备料 | 2026-09-24 不受影响：结论不碰丙 |/' .claude/kb/experiments/05-样本5.md
echo "  D4 E5 重跑、结论翻了（标题照旧是已跑），只回看 D3 那一行，D1 没动：快照 75 $(run75 "$S75" "$D/w")"
echo
echo "== E：10 号并进 ⑨ 之后——名字或结论里带「退役」「作废」两个字、结论照样成立的实验"
for name in 退役盘的重建代价 坏盘的重建代价; do
  fresh std
  printf '## E9 %s —— 已跑（2026-09-24）\n\n正文提到 D3（丙）。\n\n### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n| D3（丙） | 不影响 | 2026-09-24 不受影响：样本理由 |\n\n## 历史版本\n' "$name" \
    > .claude/kb/experiments/09-样本9.md
  ten="$(nice -n 19 bash "$D/head/.claude/gate.d/10.sh" "$D/w" 2>&1 | grep -m1 -E 'E9 已跑|每个已跑实验都被决策引用' | sed 's/^ *//')"
  printf '  E9「%s」没有一条决策引它、表里只有不影响：快照 75 %s\n      HEAD 10 号第 4 段：%s\n' "$name" "$(run75 "$S75" "$D/w")" "$ten"
done
echo
echo "== F（归 T5：75 号自己那份按行号取新增行的函数）：同 A 第一格（该红），只多一条使用者的 git 配置"
printf '#!/bin/sh\necho "external diff: $1"\n' > "$D/fake-external-diff.sh"; chmod +x "$D/fake-external-diff.sh"
for config in color.ui=always "diff.external=$D/fake-external-diff.sh"; do
  fresh std; flip '已跑（2026-09-24）'; git config "${config%%=*}" "${config#*=}"
  printf '  %-40s 快照 75 %s\n' "${config/$D/<草稿>}" "$(run75 "$S75" "$D/w")"
done
