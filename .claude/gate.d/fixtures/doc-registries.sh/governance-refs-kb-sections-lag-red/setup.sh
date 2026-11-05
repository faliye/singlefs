#!/usr/bin/env bash
# 小节滞后表 dead-section-lag.tsv 与保留表里的小节行各种写错：
#   基准里登了三行：「还没改的旧说法」（还找不到，照认）、「先写会红的检查」（此刻找得到了 ⇒ 该删）、「载体不再引的一句」（kb 不再引 ⇒ 该删）；
#   基准之后加了四行：「新写的旧说法」（比基准多 ⇒ 涨）、只有两列、理由不到 4 个字、与第一行重复；
#   保留表两行小节行：「样例规则」（此刻找得到了 ⇒ 该删）、「还没改的旧说法」（小节滞后表也登了 ⇒ 两张表只登一张）。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb .claude/rules .claude/gate.d
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的红样本，不派发。\n---\n# 样例定义\n\n这一份不点名小节。\n' > .claude/agents/sample.md
printf '# 样例规则\n\n## 先写会红的检查\n\n先造一个该红的输入。\n' > .claude/rules/sample-rule.md
printf '# 样例工具页\n\n旧说法 `.claude/rules/sample-rule.md`「还没改的旧说法」留在滞后表。\n新写的 `.claude/rules/sample-rule.md`「新写的旧说法」也登了滞后表。\n`.claude/rules/sample-rule.md`「先写会红的检查」早就找得到了。\n' > .claude/kb/tooling.md
printf '# 样本小节滞后表\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「还没改的旧说法」\t# 样本：存量，下一批改\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「先写会红的检查」\t# 样本：存量，下一批改\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「载体不再引的一句」\t# 样本：存量，下一批改\n' > .claude/gate.d/dead-section-lag.tsv
git add -A && git commit -qm 基准
printf '.claude/kb/tooling.md\t.claude/rules/sample-rule.md「新写的旧说法」\t# 样本：基准之后新加的\n.claude/kb/tooling.md\t只有两列\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「理由短」\t# 短\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「还没改的旧说法」\t# 样本：与第 2 行重复\n' >> .claude/gate.d/dead-section-lag.tsv
printf '# 样本保留表\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「样例规则」\t# 样本：句子本身说那一句已改掉\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「还没改的旧说法」\t# 样本：句子本身说那一句已改掉\n' > .claude/gate.d/dead-path-kept.tsv
