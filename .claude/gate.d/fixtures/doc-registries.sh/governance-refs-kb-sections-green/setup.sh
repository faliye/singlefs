#!/usr/bin/env bash
# kb 中文页 .claude/kb/tooling.md 里六处 `文件「小节」`（③ 扩到 kb）都不红：
#   「看它能不能把『用错了』变成『挂不上』」  嵌套引号写成『』，规则原文是「」                         ⇒ 找得到
#   「I-1（块自描述） 块自描述」               编号带简称，原文标题是「I-1 块自描述 —— 类」              ⇒ 去掉（简称）找得到
#   scripts/kb/count-sample.sh 的「结果：N 条命中」  脚本的输出句，N 对上 echo 里的 $pass               ⇒ 占位找得到
#   「还没改的旧说法」                         找不到，登在小节滞后表（基准里就有那一行）              ⇒ 滞后 1
#   「早先那一句」                             找不到，登在保留表（第二列写 <文件>「<小节>」）        ⇒ 保留 1
#   gone-prompt.md「某一节」                   文件在四个解析基准下都找不到                           ⇒ 文件找不到没判 1
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb .claude/rules .claude/gate.d crates/sample scripts/kb
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的绿样本，不派发。\n---\n# 样例定义\n\n这一份不点名小节。\n' > .claude/agents/sample.md
printf '# 样例规则\n\n> **一条格式分支值不值得，看它能不能把「用错了」变成「挂不上」**。\n' > .claude/rules/sample-rule.md
printf '## I-1 块自描述 —— 类\n\n每个单元自带类型与校验和。\n' > crates/sample/classes.md
printf '#!/usr/bin/env bash\npass=0 fail=0\necho "══ 结果：$pass 条命中，$fail 条未命中"\n' > scripts/kb/count-sample.sh
printf '# 样例工具页\n\n判据照 `.claude/rules/sample-rule.md`「看它能不能把『用错了』变成『挂不上』」。\n类目照 `crates/sample/classes.md`「I-1（块自描述） 块自描述」一节。\n数由 `scripts/kb/count-sample.sh` 的「结果：N 条命中」报出来。\n旧说法 `.claude/rules/sample-rule.md`「还没改的旧说法」登在小节滞后表。\n当时记的 `.claude/rules/sample-rule.md`「早先那一句」已改掉。\n已归档的 gone-prompt.md「某一节」不在四个解析基准下。\n' > .claude/kb/tooling.md
printf '# 样本小节滞后表\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「还没改的旧说法」\t# 样本：存量，下一批改\n' > .claude/gate.d/dead-section-lag.tsv
printf '# 样本保留表\n.claude/kb/tooling.md\t.claude/rules/sample-rule.md「早先那一句」\t# 样本：句子本身说那一句已改掉\n' > .claude/gate.d/dead-path-kept.tsv
git add -A && git commit -qm 基准
