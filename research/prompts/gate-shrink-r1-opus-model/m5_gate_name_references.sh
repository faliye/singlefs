#!/usr/bin/env bash
# 模型 M5（gate-shrink-r1 云端攻方）：「门禁取消编号、以后按全名记录」之后，治理文档里指向门禁的写法还有没有人判。
# 10 号 governance-refs 那一格调 .claude/gate.d/lib-governance-refs.py；这里在临时目录里摆一份只有一个 CLAUDE.md 的小仓，
# 同一句话写三种指法：两位数加「号」、反引号里的阶段路径、只写全名（方案要的写法），三者指的都是不存在的门禁。
# lib 按它自己所在的目录认「有哪些门禁号」，所以用的是本仓那份门禁目录；小仓里一个门禁脚本都不放。
# 用法：bash m5_gate_name_references.sh <仓根> <输出目录>
set -uo pipefail
repository_root="$(cd "${1:?仓根}" && pwd)"
output_directory="${2:?输出目录}"
mkdir -p "$output_directory"
work_directory="$(mktemp -d "${TMPDIR:-/tmp}/m5-work.XXXXXX")"
git -C "$work_directory" init -q
cat > "$work_directory/CLAUDE.md" <<'MD'
# 样本

- 写完决策跑门禁 98 号。
- 写完决策跑 `.claude/gate.d/doc-decisions.sh`。
- 写完决策跑 doc-decisions 那一道的 decision-items-sync 格。
MD
library_exit=0
( cd "$work_directory" && python3 "$repository_root/.claude/gate.d/lib-governance-refs.py" ) > "$output_directory/m5.log" 2>&1 || library_exit=$?
echo "lib_exit=$library_exit"
for line_number in 3 4 5; do
  if grep -q "CLAUDE.md:$line_number:" "$output_directory/m5.log"; then echo "CLAUDE.md 第 $line_number 行：判红"; else echo "CLAUDE.md 第 $line_number 行：没判红"; fi
done
tail -1 "$output_directory/m5.log"
rm -rf "${work_directory:?}"
