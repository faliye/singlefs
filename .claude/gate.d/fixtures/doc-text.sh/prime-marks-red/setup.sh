#!/usr/bin/env bash
# 红样本现场，每种形态各几格，文件全由这里现写：
# 角标：一份 md 带「K9」加一撇、一份 rs 带「G5」加两撇（八进制转义），各该点名一处；
# 钟点：一份 md 写「日期 时区词 时:分x」、一份 rs 写「时:分 时区词」、一份 md 三行各是「时区词 时:分」「时:分x」「时:分Z」，
# 「N 点 / N 时」与时段词的四个判据行各一份 md，一行一种写法（取自仓里清掉之前的原写法），每行都该点名，判据行名字也要对上；
# 不挨日期时区的钟点（时:分、时:分:秒、带毫秒、区间、「月-日 时:分」、反引号里两位小时的）一份 md，
# 光秃的时区词一份 md，ISO 与机器时间戳（T 形态、Z 结尾、diff 头与 stat 的带秒带偏移）一份 md；
# 放行标记写在 md 里（不认）、写在 rs 里却没写理由（不认），各一行照判钟点。
# 这份源码本身不出现那几个撇号字符、字面钟点与时区词：冒号由变量拼进去，时区词由 "" 断开拼进变量，
# 数字与「点」「时」、日期与时段词之间也用 "" 断开——否则门禁 doc-text.sh 扫样本现场时会先把这份 setup.sh 判红，多红几行。
set -e
colon=:
jst="J""ST"
utc="U""TC"
mkdir -p .claude/kb crates/demo/src records
printf '## 候选\n\n取 K9\342\200\262（收严版），不取 K9。\n' > .claude/kb/候选.md
printf '// 变体 G5\342\200\263 的开销\nfn variant() {}\n' > crates/demo/src/variant.rs
printf "# 记录\n\n用户 2026-09-26 ${jst} 15${colon}3x 弹窗定甲二。\n" > records/记录.md
printf "// 跑于 03${colon}44 ${utc}（修复之后）\nfn main() {}\n" > crates/demo/src/main.rs
printf '%s\n' "同日 ${utc} 14${colon}00 前后" "主 agent 22${colon}5x 跑前修订" "跑于 03${colon}44Z 的那一轮" > records/time-colon.md
printf '%s\n' "用户 2026-09-28 ${jst} 08 ""点前后要开" "本机 ${utc} 2026-09-26 23 ""点后" "2026-09-16 ${utc} 17 ""时许的快照" \
  "09-24 ${jst} 19 ""点前后问了四件" "2026-09-25 ${jst} 01 ""点后弹窗" > records/date-zone-hour.md
printf '%s\n' "交回于 2026-09-27 ${jst} ""上午" "降为待证（2026-09-13 ""上午）" "2026-09-12 ""凌晨（${jst}）" "（${jst} 2026-09-27 ""早上）" \
  "满载到 2026-09-28 ""中午前后" "### 2026-09-27（""下午）：E162" "2026-09-27 ""上午 08${colon}32 现跑" > records/date-zone-period.md
printf '%s\n' "上午 08${colon}32 现跑" "晚上 ""8 点" > records/period-hour.md
printf '%s\n' "19 ""点前后问了" > records/hour-suffix.md
printf '%s\n' "交回 14${colon}00${colon}10、续做 14${colon}00${colon}15" "用户 23${colon}10 定" "07${colon}58${colon}02.123 起跑" \
  "14${colon}00–15${colon}30 之间跑完" "核查员缺快照（09-24 23${colon}44 那一次）" "模型（改于 12${colon}22，是别的会话那一次）" \
  "跑于 \`09${colon}55\` 的那一轮" "攻方腿 9${colon}05 写一句" > records/bare-clock.md
printf '%s\n' "跑于 2026-09-27（${jst}），本机" "本机时钟是 ${utc}、直接 date 会差一天" "${jst} 与 ${utc} 两个时区" "${utc} 时间" > records/zone-word.md
printf '%s\n' "派腿时刻 2026-09-27T09${colon}55${colon}07+00${colon}00。" "交回于 …T23${colon}15Z" \
  "legs dispatched at 2026-09-28T00${colon}31${colon}09+00${colon}00" > records/iso.md
printf -- "--- a/x.rs\t2026-09-24 16${colon}27${colon}05.398761624 +0000\nModify: 2026-09-24 18${colon}48${colon}58.254848952 +0000\n" > records/diff.md
printf '%s\n' "用户 23${colon}10 定  # clock-times:allow 写在 md 里不认" > records/allow-in-md.md
printf '%s\n' "// 跑于 23${colon}10  // clock-times:allow" "fn main() {}" > crates/demo/src/no_reason.rs
