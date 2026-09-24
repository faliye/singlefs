#!/usr/bin/env bash
# T1：戊（撤掉 10 号 §3、射程交 75 号 ⑤）与丙b*（这一批改成已跑的实验，正文引用它的决策文件要在这一批里）在同一批改动上的判定；
# 再看 10 号剩下的「已跑实验要有决策引用」与 75 号 ⑨ 是不是同一件事。跑的是真仓现在的 10 号、75 号；丙b* 用本目录 t1-judge.py。
# 用法：bash t1-scenes.sh <真仓根> <草稿目录> <模型目录>
set -uo pipefail
REPO="$(cd "$1" && pwd)"; D="$2"; MODEL="$(cd "$3" && pwd)"; mkdir -p "$D"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
unset GATE_BASE GATE_DIFF_BASE GATE_STAGED_FROM
stage() { local out rc=0; out="$(bash "$REPO/.claude/gate.d/$1.sh" "$PWD" 2>&1)" || rc=$?; printf '%s 号退 %s' "${1%%-*}" "$rc"; grep -E '^\s+(\[|✗)' <<<"$out" | grep -vE '^\s+✗ (决策与实验|kb 腐化)' | head -3 | sed 's/^ */；/' | tr -d '\n'; }
judge() { printf '丙b* %s' "$(python3 "$MODEL/t1-judge.py" "$PWD" "$REPO/research/scripts/changed-paths.sh")"; }
report() { echo "  $1"; echo "    $(stage 75-decision-experiment-links)"; echo "    $(stage 10-kb-rot)"; echo "    $(judge)"; }
base() {
  rm -rf "$D/$1"; mkdir -p "$D/$1"; cd "$D/$1" && git init -q -b master .
  mkdir -p .claude/kb/decisions .claude/kb/experiments
  F75="$REPO/.claude/gate.d/fixtures/75-decision-experiment-links.sh/green"; F10="$REPO/.claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/kb"
  cp "$F10/invariants.md" "$F10/checks-owed.md" .claude/kb/
  printf '# 决策索引\n\n## 历史版本\n' > .claude/kb/decisions.md
  sed -n "/^cat > .claude\/kb\/decisions\/03-丙.md <<'EOF'$/,/^EOF$/p" "$F75/setup.sh" | sed '1d;$d' > .claude/kb/decisions/03-丙.md
  cat > .claude/kb/decisions/01-甲.md <<'MD'
## D1 甲 —— 已定

### 已定项

| # | 分项 | 定案 |
|---|---|---|
| 1 | 样本分项 | 一句话定案 |

#### 已定项 1：样本分项

**定案**：样本规则。

**射程**：只管样本。

**依据**：E1（样本实验一） 证明了样本规则成立。

**欠**：E2（样本实验二） 跑完再核射程；E4（样本实验四） 跑完再核定案。

## 历史版本
MD
  page() { printf '## E%s %s —— %s\n\n%s\n\n### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n%s\n\n## 历史版本\n' "$@" > ".claude/kb/experiments/0$1-样本.md"; }
  page 1 样本实验一 已跑 '正文提到 D1（甲）。' '| D1（甲） 已定项 1 | 支撑 | 2026-09-18 不受影响：结论与定案同向 |'
  page 2 样本实验二 待跑 '正文提到 D3（丙）。' '| D3（丙） 已定项 1 | 备料 | 2026-09-18 不受影响：还没有跑完 |'
  page 4 样本实验四 待跑 '正文提到 D1（甲） 与 D3（丙）。' "$(printf '| D1（甲） 已定项 1 | 备料 | 2026-09-18 不受影响：还没有跑完 |\n| D3（丙） 已定项 1 | 不影响 | 2026-09-18 不受影响：只作背景 |')"
  git add -A && git commit -qm base && git branch 上游 && git branch -q -u 上游 master
}
run_to_ran() {  # $1 = 实验号：标题改成已跑、加一行结论，回看 D3 那一行（D1 那份决策不动）
  sed -i "1s/—— 待跑/—— 已跑/; 3s/\$/结论：样本量出来了。/" ".claude/kb/experiments/0$1-样本.md"
  sed -i '/^| D3/ s/| 2026-09-18 不受影响：\(还没有跑完\|只作背景\) |$/| 2026-09-24 不受影响：结论不碰 D3 |/' ".claude/kb/experiments/0$1-样本.md"
}
history_entry() { printf '\n### 2026-09-24\n\n- 跑完。\n' >> ".claude/kb/experiments/0$1-样本.md"; }

echo "== 第一族：这一批把一个待跑实验改成已跑，只回看了影响表里另一行；引用它的 D1 一个字没动"
base 基线; report "基线（未改）"
base A; run_to_ran 4; report "A  E4 已跑；表里有 D1（备料）与 D3 两行，只回看 D3 那一行"
base A2; run_to_ran 4; history_entry 4; report "A' 同 A，另在 E4 页内历史节记一条 2026-09-24"
base B; run_to_ran 2; history_entry 2; report "B  E2 已跑（页内也记了 2026-09-24）；D1 只在「**欠**」块里点名它，E2 的表里没有 D1 这一行"
base B0; run_to_ran 2; sed -i 's/E2（样本实验二） 跑完再核射程/E2（样本实验二） 已跑，射程不变/' .claude/kb/decisions/01-甲.md; report "对照 同 B，D1 的「**欠**」块跟着改了"
base N; sed -i "1s/—— 待跑/—— 已跑/" .claude/kb/experiments/04-样本.md; report "对照 E4 只改标题成已跑，影响表一行都没回看"
cp .git/index "$D/N.index"; printf 'DIRC garbage' > .git/index; report "同上一格，另把 .git/index 写坏（git diff 退 128）"; cp "$D/N.index" .git/index

echo
echo "== 第二族：10 号剩下那一段与 75 号 ⑨ 分得开的三种页（各在基线上加一页）"
extra() { base "$1"; printf '%s\n' "$2" > ".claude/kb/experiments/$3"; [[ -n "${4:-}" ]] && sed -i "s/^## 历史版本\$/## 历史版本\n\n- 2026-09-10 早先参考过 $4。/" .claude/kb/decisions/03-丙.md; git add -A; git commit -qm extra -q; git branch -f 上游 HEAD; }
extra C "$(printf '## E5 样本实验五 —— 已跑，结论作废\n\n装置错了，那条路不通。\n\n### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n| D3（丙） | 不影响 | 2026-09-18 不受影响：结论作废，只记那条路不通 |\n\n## 历史版本')" 05-样本.md
report "C  已跑但结论作废，没有决策提到它（75 号 ⑨ 按标题里的「作废」不判）"
extra D "$(printf '## E6 样本实验六 —— 已跑\n\n结果等政策用。\n\n### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n| D3（丙） 已定项 1 | 备料 | 2026-09-18 不受影响：等政策用 |\n\n## 历史版本')" 06-样本.md
report "D  已跑；影响表里有一行「备料」，正文没写「**备料**：」那一行，也没有决策提到它"
extra E "$(printf '## E7 样本实验七 —— 已跑\n\n背景实验。\n\n### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n| D3（丙） | 不影响 | 2026-09-18 不受影响：只作背景 |\n\n## 历史版本')" 07-样本.md "E7（样本实验七）"
report "E  已跑；影响表只有「不影响」；D3 只在自己的「## 历史版本」里提到它"
