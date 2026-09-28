#!/usr/bin/env bash
# 绿样本现场，文件全由这里现写：
# 正常写法——变体起了新名字、ASCII 单引号；只写日期；
# 与「N 点」、时段词近似却不是钟点的写法（序号、个数、光秃时段词、用户原话引文里的「东京时间」、标识符里的时区词），都该放行；
# 长得像钟点的冒号对（产物行的 名字=值、Python 切片、字典与逗号列表、时区偏移、引号与反引号里的代码字面、百分比、time -v 的格式说明、
# 「门禁号:行号」表格、门禁文件名、同一行带「文件:行号」的编号对）一行一类，靠上下文规则放行：
# LIB_FORBIDDEN_NOTATIONS_BREAK_CONTEXT=clock-times 时这里判红；
# .sh、.py、.rs 里带 clock-times:allow 放行标记（写了理由）的三行放行：LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=clock-times 时这里判红。
# 每种形态落在它排除前缀下的一格：上游副本里带角标的 md（八进制转义）；门禁样本的 expect、hook 自检、c510 那一轮的提示里带钟点或时区词的行
# （冒号由变量拼、时区词由 "" 断开拼进变量）。这几格只在排除前缀生效时放行：LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS 点名那一形态时这里判红。
set -e
colon=:
jst="J""ST"
utc="U""TC"
mkdir -p .claude/kb records .claude/singlefs-ai-sop/rules .claude/gate.d/fixtures/x/red .claude/hooks .claude/scripts crates/demo/src research/prompts
printf '%s\n' '## 候选' '' "取 K13（K9 的收严版），不取 K9。引号 '甲' 与 don't 这类写法不管。" > .claude/kb/候选.md
printf '# 记录\n\n用户 2026-09-26 弹窗定甲二。\n' > records/记录.md
printf '%s\n' '第 3 点前后两句' '3 点意见' '1000 个崩溃点' '全量由用户要求或夜间跑' '用户 2026-09-28 定' '同一天下午续跑' '用户原话「东京时间7点前」' \
  "\`${jst}_OFFSET\`、\`started_${utc}\` 这类标识符" > records/近似写法.md
{
  printf '%s\n' 'remount_chosen=2:12 device_chosen=1:16 values=1:16.000,3:12.000,5:14.000'
  printf '%s\n' 'adjacent_differences=0->1:10,1->2:8 steps_by_changed_leaf_count=1:108,2:36'
  printf '%s\n' '分布 {0:14, 1:14, 2:16}'
  printf '%s\n' "r[8:12]=i.to_bytes(4,'little'); fields[11:15]; arguments[5:13]"
  printf '%s\n' 'stamp.replace("Z", "+00:00")'
  printf '%s\n' '            Some("2:11"),'
  printf '%s\n' "                      printf '2:11' ;;"
  printf '%s\n' 'B Lost Writes **13:54%**'
  printf '%s\n' 'Elapsed (wall clock) time (h:mm:ss or m:ss): 0:03.21'
  printf '%s\n' '| 共用脚本不在 | 21:38 | 31:68 |'
  printf '%s\n' '| kb 目录不在 | —— | 21:39、24:27、27:39 |'
  printf '%s\n' '门禁 20:54-layer0-replay.sh 那一行'
  printf '%s\n' '触发的那一处（decisions/06-快照实现模型.md:42），06:42 正是没抽到的那一行'
  printf '%s\n' '撞键 `3:10`：冷重开是 `2:11`'
  printf '%s\n' 'taken_jst=2026-09-27T01:20:12+09:00 started_jst=2026-09-15T03:12'
} > records/colon-pairs.md
printf '%s\n' "TZ=${utc}-14 date +%F  # clock-times:allow 环境变量写法：POSIX 的时区串，符号与东京时间相反" > .claude/scripts/tz.sh
printf '%s\n' "RESETS = re.compile(r\"resets (\\d+)(am|pm) \\(${utc}\\)\")  # clock-times:allow 解析外部通知文本，格式由对方定" > .claude/hooks/parse.py
printf '%s\n' "let stamp = \"2026-09-27T09${colon}55${colon}07Z\"; // clock-times:allow 造时间戳格式的自检输入" > crates/demo/src/stamp.rs
printf '取 K9\342\200\262 的读法（上游的样本）\n' > .claude/singlefs-ai-sop/rules/变体.md
printf '%s\n' "改于 2026-09-05 00${colon}00" > .claude/gate.d/fixtures/x/red/expect
printf '%s\n' "# 自检的假日志：2026-09-25 07${colon}58${colon}02 ${utc}" > .claude/hooks/session-start.sh
printf '%s\n' "本机时钟是 ${utc}、人在东京时，东京 00${colon}00–09${colon}00 写下的当天日期比 ${utc} 的今天晚一天" > research/prompts/_c510-date-gate-r1-appendix.md
