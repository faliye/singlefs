#!/usr/bin/env bash
# D3 探针：两个会话共写一份产品代码，这一批只改注释、用户只豁免这一批的改动；照 main-agent.md 第 9 条「按内容哈希登记的照文件现状的哈希登记」
# 登记之后，门禁 crates-adversarial-review / agent-def-adversarial-review 两格共用的 user_exemption_reason（冻结副本
# .claude/gate.d/doc-process-records.sh 里那个函数，原样抽出来调）对「这一批 + 别的会话没审过的代码改动」那一版交不交豁免理由。
# 用法：bash d3-exempt.sh <冻结副本> <新的工作目录>
set -euo pipefail
FROZEN="$1"; WORK="$2"
ROOT="$WORK/d3-root"
mkdir -p "$ROOT/crates/demo/src" "$ROOT/.claude"
start=$(grep -n '^user_exemption_reason() {' "$FROZEN/.claude/gate.d/doc-process-records.sh" | cut -d: -f1)
awk -v s="$start" 'NR>=s { print } NR>s && /^}$/ { exit }' "$FROZEN/.claude/gate.d/doc-process-records.sh" > "$WORK/user_exemption_reason.sh"
source "$WORK/user_exemption_reason.sh"
f="$ROOT/crates/demo/src/lib.rs"
# 基线（HEAD 那一版）
printf '// 判一个块号在不在池里\npub fn in_pool(block: u64, size: u64) -> bool {\n    block < size\n}\n' > "$f"
# 别的会话：没提交、没审的代码改动（边界从 < 改成 <=）
sed 's/block < size/block <= size/' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
# 这一批：只改注释，用户说「豁免」
sed 's#// 判一个块号在不在池里#// 判一个块号在不在池里（D19（块指针的结构与宽度预算） 已定项 1）#' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
# 照第 9 条：按文件现状的哈希登记
cd "$ROOT"
printf '# 表头\ncrates/demo/src/lib.rs\t%s\t用户 2026-10-12 指示「豁免」：这一批只改注释\n' "$(sha256sum crates/demo/src/lib.rs | cut -d' ' -f1)" > .claude/crates-review-exempt
why="$(user_exemption_reason .claude/crates-review-exempt crates/demo/src/lib.rs)"
echo "name=d3 case=register-current-hash exempted=$([[ -n "$why" ]] && echo yes || echo no) file_has_other_session_code_change=$(grep -c 'block <= size' crates/demo/src/lib.rs)"
# 对照：只算这一批自己那一版（基线 + 注释）的哈希
printf '// 判一个块号在不在池里（D19（块指针的结构与宽度预算） 已定项 1）\npub fn in_pool(block: u64, size: u64) -> bool {\n    block < size\n}\n' > "$WORK/own-only.rs"
printf '# 表头\ncrates/demo/src/lib.rs\t%s\t同上\n' "$(sha256sum "$WORK/own-only.rs" | cut -d' ' -f1)" > .claude/crates-review-exempt
why="$(user_exemption_reason .claude/crates-review-exempt crates/demo/src/lib.rs)"
echo "name=d3 case=register-own-change-only-hash exempted=$([[ -n "$why" ]] && echo yes || echo no)"
# 暂存那一版：主 agent 照 main-agent.md 派发表用 stage-mine.py 只暂存这一批的块，gate.sh --staged 判的树里这份文件是「基线 + 注释」，
# 豁免表登记的却是工作区现状（含别的会话那一块）的哈希
STAGED="$WORK/d3-staged-root"
mkdir -p "$STAGED/crates/demo/src" "$STAGED/.claude"
cp "$WORK/own-only.rs" "$STAGED/crates/demo/src/lib.rs"
printf '# 表头\ncrates/demo/src/lib.rs\t%s\t用户 2026-10-12 指示「豁免」：这一批只改注释\n' "$(sha256sum crates/demo/src/lib.rs | cut -d' ' -f1)" > "$STAGED/.claude/crates-review-exempt"
cd "$STAGED"
why="$(user_exemption_reason .claude/crates-review-exempt crates/demo/src/lib.rs)"
echo "name=d3 case=staged-tree-with-current-hash-registered exempted=$([[ -n "$why" ]] && echo yes || echo no)"
