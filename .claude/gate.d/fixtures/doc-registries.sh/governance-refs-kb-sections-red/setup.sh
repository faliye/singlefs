#!/usr/bin/env bash
# kb 中文页 .claude/kb/tooling.md 里三处 `文件「小节」`（③ 扩到 kb）：
#   「先写会红的检查」             规则里有这个标题                                  ⇒ 判过、找得到
#   「跑批脚本不许吞掉单轮失败」   规则里没有，最像的是第 5 行那一句                  ⇒ 红，出路写「最像的是那份文件第 5 行的句子…」
#   「从来没有写过的一句话」       规则里没有、也没有够得上的标题或句子               ⇒ 红，出路写「那份文件里没有，改写那一句或删去」
# 历史版本节与英语译文里找不到的小节不判。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb .claude/rules
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的红样本，不派发。\n---\n# 样例定义\n\n这一份不点名小节。\n' > .claude/agents/sample.md
printf '# 样例规则\n\n## 先写会红的检查\n\n**跑批脚本不许把单轮的失败吞掉**（`|| true` 那种）。\n' > .claude/rules/sample-rule.md
printf '# 样例工具页\n\n规则 `.claude/rules/sample-rule.md`「先写会红的检查」是这一条的依据。\n跑批要照 `.claude/rules/sample-rule.md`「跑批脚本不许吞掉单轮失败」。\n另见 `.claude/rules/sample-rule.md`「从来没有写过的一句话」。\n\n## 历史版本\n\n- 早先引的是 `.claude/rules/sample-rule.md`「已经改掉的旧说法」。\n' > .claude/kb/tooling.md
printf '# Sample tooling page\n\nSee `.claude/rules/sample-rule.md`「a section only the translation names」.\n' > .claude/kb/tooling.en.md
git add -A && git commit -qm 基准
