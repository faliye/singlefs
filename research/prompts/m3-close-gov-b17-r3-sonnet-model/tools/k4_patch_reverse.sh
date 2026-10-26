#!/usr/bin/env bash
# K4：把修补二批的补丁（开工快照里的 governance-fix-2.patch）倒着打在主工作区现文的拷贝上，看倒回去的 sha256 是不是第二轮判决时记的
# （第二轮快照 research/prompts/m3-close-gov-b17-r2-snapshot/judged-sha256.txt）；verification.md 另把第 35 行倒回「解析出的依赖版本」再算一次。
# 用法：k4_patch_reverse.sh <仓根> <临时目录（要不存在或为空）>；用完临时目录自己删。
set -euo pipefail
repo="$1"; scratch="$2"
mkdir -p "$scratch/rev/.claude/rules" "$scratch/rev/scripts/mutation"
cp "$repo/.claude/rules/implementation-workflow.md" "$repo/.claude/rules/verification.md" "$scratch/rev/.claude/rules/"
cp "$repo/scripts/mutation/crates-mutation-rows.py" "$scratch/rev/scripts/mutation/"
(cd "$scratch/rev" && patch -R -p1 < "$repo/research/prompts/m3-close-gov-b17-r3-snapshot/governance-fix-2.patch")
(cd "$scratch/rev" && sha256sum .claude/rules/implementation-workflow.md .claude/rules/verification.md scripts/mutation/crates-mutation-rows.py)
echo "第二轮判决时记的 sha256："
grep -E 'implementation-workflow.md|verification.md|crates-mutation-rows.py' "$repo/research/prompts/m3-close-gov-b17-r2-snapshot/judged-sha256.txt"
python3 -I - "$scratch/rev/.claude/rules/verification.md" <<'PY'
import hashlib, sys
lines = open(sys.argv[1], encoding="utf-8").read().split("\n")
print("verification.md 第 35 行含「解析出的闭包真引到的依赖版本」的次数：", lines[34].count("解析出的闭包真引到的依赖版本"))
lines[34] = lines[34].replace("解析出的闭包真引到的依赖版本", "解析出的依赖版本")
digest = hashlib.sha256("\n".join(lines).encode("utf-8")).hexdigest()
print("verification.md 倒回补丁、再把第 35 行倒回「解析出的依赖版本」：", digest)
print("等于第二轮判决时记的：", digest == "e59b30f6723738dd9e49e8c5c350a8853fac1bff43b36a41169a17a97d6b4339")
PY
