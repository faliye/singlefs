#!/usr/bin/env bash
# 五组各错一项，其余项三份相同：README 的编号集合（中文少 C7）、2026-09-14 的链接目标集合（中文的第二个链接指向别处）、
# 2026-09-28 的二级标题个数（日语少一个）、2026-10-10 的表格行个数（日语少数据行）、2026-10-12 的围栏代码块个数（日语少一个）。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，英语为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是英语版本'
put README.md '# Sample' 'See D5 and E152 (I-10.1) plus C7.'
put README.zh.md '# 样本' '见 D5 与 E152（I-10.1）。'
put README.ja.md '# サンプル' 'D5 と E152（I-10.1）と C7 を参照。'
put briefs/2026-09-14.md '# Brief' 'See [a](docs/a.md) and [b](docs/b.md).'
put briefs/2026-09-14.zh.md '# 简报' '见 [a](docs/a.md) 与 [b](docs/c.md)。'
put briefs/2026-09-14.ja.md '# 概要' '[a](docs/a.md) と [b](docs/b.md) を参照。'
put briefs/2026-09-28.md '# Brief' '## Part A' '## Part B'
put briefs/2026-09-28.zh.md '# 简报' '## 甲部' '## 乙部'
put briefs/2026-09-28.ja.md '# 概要' '## 甲部'
put briefs/2026-10-10.md '# Brief' '| a | b |' '|---|---|' '| 1 | 2 |'
put briefs/2026-10-10.zh.md '# 简报' '| a | b |' '|---|---|' '| 1 | 2 |'
put briefs/2026-10-10.ja.md '# 概要' '| a | b |' '|---|---|'
put briefs/2026-10-12.md '# Brief' '```' 'one' '```' '```' 'two' '```'
put briefs/2026-10-12.zh.md '# 简报' '```' '一' '```' '```' '二' '```'
put briefs/2026-10-12.ja.md '# 概要' '```' '一' '```'
