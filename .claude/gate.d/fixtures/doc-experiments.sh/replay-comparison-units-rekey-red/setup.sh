#!/usr/bin/env bash
# 借键红样本的现场：基准提交里的登记表是四行 E991 的 none 行（丙、戊、庚、甲），工作区照原样（见登记表文件头）：
# 前三行的（登记出处，留存产物名）与基准那一版相同，实验号或单元名换了，是新的比对对象，none 行比基准多；甲那一行原样，不报。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
units=.claude/gate.d/replay-comparison-units.tsv
current="$(mktemp)"
cp "$units" "$current"
cat > "$units" <<'TSV'
# 样本登记表（借键红）的基准那一版
E991	research/prompts/e991-registration.md	丙	crates/sample/units.rs	e991-c.out	none:没有复跑行的理由甲
E991	research/prompts/e991-registration.md	戊	crates/sample/units.rs	e991-e.out	none:没有复跑行的理由乙
E991	research/prompts/e991-registration.md	庚	crates/sample/units.rs	e991-g.out	none:没有复跑行的理由丙
E991	research/prompts/e991-registration.md	甲	crates/sample/units.rs	e991-a.out	none:没有复跑行的理由丁
TSV
git add -A
git commit -qm '样本基准：登记表四行 E991 的 none 行'
cp "$current" "$units"
rm -f "${current:?}"
