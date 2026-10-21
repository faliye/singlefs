#!/usr/bin/env bash
# 登记 README.md 与 briefs/*.md。README 缺日语；2026-09-14 三份齐全；2026-09-28 缺英语；
# 2026-10-10 只有英语（中文、日语都缺）；2026-10-12 有英语和日语而中文不在：后两组只能靠「登记目录里单有 .en.md 或 .ja.md」那一条抓出来。
set -e
put() { mkdir -p "$(dirname "$1")"; printf '%s\n' "${@:2}" > "$1"; }
put .claude/trilingual-documents \
  'README.md  # 样本里的仓根说明，中文为默认版本' \
  'briefs/*.md  # 样本里的简报，每份 YYYY-MM-DD.md 是中文版本'
put README.md '# 样本'
put README.en.md '# Sample'
put briefs/2026-09-14.md '# 简报'
put briefs/2026-09-14.en.md '# Brief'
put briefs/2026-09-14.ja.md '# 概要'
put briefs/2026-09-28.md '# 简报'
put briefs/2026-09-28.ja.md '# 概要'
put briefs/2026-10-10.en.md '# Brief'
put briefs/2026-10-12.en.md '# Brief'
put briefs/2026-10-12.ja.md '# 概要'
