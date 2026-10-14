#!/usr/bin/env bash
# 登记 README.md 与 briefs/*.md。README 缺日语；2026-09-14 三份齐全；2026-09-28 缺中文；
# 2026-10-10 只有中文（英语、日语都缺）；2026-10-12 有中文和日语而英语不在：后两组只能靠「登记目录里单有 .zh.md 或 .ja.md」那一条抓出来。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，英语为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是英语版本'
put README.md '# Sample'
put README.zh.md '# 样本'
put briefs/2026-09-14.md '# Brief'
put briefs/2026-09-14.zh.md '# 简报'
put briefs/2026-09-14.ja.md '# 概要'
put briefs/2026-09-28.md '# Brief'
put briefs/2026-09-28.ja.md '# 概要'
put briefs/2026-10-10.zh.md '# 简报'
put briefs/2026-10-12.zh.md '# 简报'
put briefs/2026-10-12.ja.md '# 概要'
