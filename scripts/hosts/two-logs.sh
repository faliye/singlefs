#!/usr/bin/env bash
# admission: always 每一次调都包这一次的命令、写这一次的日志，上一次的结论不替这一次作保
# run-condition: command python3
# 日志出两份：一份全量，一份只收要看的行（错误与警告），一般只看后一份。
#
#   two-logs.sh [--quiet] <名> -- <命令…>   包住命令：标准输出与标准错误各自照原样转打，同时写 <名>.log（全量）与 <名>.problems.log
#                                          （只收要看的行，每行前面带它在全量日志里的行号，末行写收了几行、全量几行）；退出码是被包命令的
#   two-logs.sh --from-log <名>             不跑命令：从已有的 <名>.log 现筛出 <名>.problems.log
#   two-logs.sh --selftest                  自证（拿答案已知的合成输入逐类核筛选，另起真进程核转打、退出码、信号、管道被占着）
# <名> 是日志路径去掉 .log 的前缀（建好它所在的目录）；--quiet 不转打也不打路径行，只写两份日志（入口脚本里本来只落盘的构建日志用）。
# 退出码：被包命令的；125 用法不对或日志建不了（同 env 的约定）；126 命令起不来；127 找不到命令。
# 收尾往标准错误打一行两份日志的路径（--quiet 不打）；这一行不进任何一份日志。
#
# 收哪些行（筛选写成纯函数，在 two_logs.py 的 filter_problems / ProblemFilter）：
#   门禁符号：带 ✗ 的行；✗ 与 ⚠ 之后跟着的 → 出路行，连同 → 之前的明细行、→ 之后缩进的续行（一块至多 40 行，多的写一行说明）；带 ⚠ 的行；
#             带「本次未跑」的行；逐格汇总（「── 汇总」起头的行，与整行是「<格名>：绿 / 红 / 红（退出 N） / 本次未跑」的行）。
#   工具报错：cargo / rustc 的 error、error[E…]、warning: 行，连同紧跟其后指到文件行号的 --> 那一行；libtest 的 FAILED 与 panicked；
#             Python 的 Traceback 起的那一段；报非 0 退出码的行（「退出码 N」「退出 N」「退 N」、exit status / exit code N）。
#   别的行不收。
#
# 被包的命令里的 --force：这一份只对 python3 做准入（不带参数问 preflight），所以命令的参数原样交给命令，不会被摘掉。
# 被包命令退出后，管道要是被它放到后台的孙进程占着不关，再等 2 秒（TWO_LOGS_DRAIN_SECONDS 可改）就收工；收到 SIGTERM、SIGHUP 原样转给被包的命令。
#
# 配套的钩子 .claude/hooks/two-logs-guard.sh 在执行前拒绝把门禁、cargo test、四个入口脚本（replay.sh、crash-point-replay.sh、crash-amplification.sh、
# merge-and-verify.sh）与 gate-staged.sh 的输出用 >、>>、&>、| tee 写到 .log，说明里指到这里。
#
# 弄坏开关 TWO_LOGS_BREAK=<项>（逗号分隔，只给自证证红；认不出的项直接拒，退 125）。打开后自证红的格（失败行「✗ two_logs 自证：<格名>：…」）：
#   筛选一侧：
#   fail-mark-ignored  不认 ✗ 行
#       红：「✗ 行连同明细与出路」「✗ 之后的 → 出路行到下一处非 → 续行为止」「明细超过上限只收到上限并说明」
#   fail-arrow-dropped  ✗ 与 ⚠ 后面的 → 出路行不收
#       红：「✗ 行连同明细与出路」「✗ 之后的 → 出路行到下一处非 → 续行为止」「⚠ 行连同出路」「明细超过上限只收到上限并说明」
#   fail-details-dropped  ✗ 之后的明细与 → 之后的续行不收
#       红：「✗ 行连同明细与出路」「明细超过上限只收到上限并说明」
#   warn-mark-ignored  不认 ⚠ 行
#       红：「⚠ 行连同出路」
#   not-run-ignored  不认「本次未跑」行
#       红：「本次未跑的行」
#   summary-ignored  不收汇总段
#       红：「逐格汇总段」
#   cargo-error-ignored  不认 cargo 的 error 行
#       红：「cargo error 连同 --> 行」
#   cargo-warning-ignored  不认 cargo 的 warning: 行
#       红：「cargo warning 连同 --> 行」
#   cargo-location-dropped  不收 error、warning 下面的 --> 行
#       红：「cargo error 连同 --> 行」「cargo warning 连同 --> 行」
#   libtest-failed-ignored  不认 FAILED
#       红：「libtest FAILED 与 panicked」
#   panicked-ignored  不认 panicked
#       红：「libtest FAILED 与 panicked」
#   traceback-body-dropped  只收 Traceback 起头那一行，栈帧与异常行不收
#       红：「Python Traceback 整段」
#   exit-code-ignored  不认报非 0 退出码的行
#       红：「报非 0 退出码的行」
#   collect-everything  什么行都收
#       红：筛选一侧每一格
#   relay 一侧：
#   line-number-missing  problems 日志的行不带行号
#       红：「problems 日志的行带行号、末行写收了几行全量几行」「没有换行的最后一行」「非 UTF-8 字节原样转打」
#   footer-missing  不写收尾行
#       红：「problems 日志的行带行号、末行写收了几行全量几行」
#   exit-code-lost  退出码恒为 0
#       红：「退出码原样交回」「入口脚本 two-logs.sh 转参数、交退出码」
#   stdout-not-forwarded  不转打标准输出
#       红：「标准输出与标准错误各自照原样转打、全量日志收全」「非 UTF-8 字节原样转打」「入口脚本 two-logs.sh 转参数、交退出码」
#   stderr-not-forwarded  不转打标准错误
#       红：「标准输出与标准错误各自照原样转打、全量日志收全」
#   stderr-merged-into-stdout  标准错误并进标准输出转打
#       红：「标准输出与标准错误各自照原样转打、全量日志收全」
#   full-log-skips-stderr  全量日志不收标准错误
#       红：「标准输出与标准错误各自照原样转打、全量日志收全」「problems 日志的行带行号、末行写收了几行全量几行」
#   partial-line-dropped  没有换行的最后一行丢掉
#       红：「没有换行的最后一行」
#   stale-logs-kept  开跑前不截旧日志
#       红：「旧日志不残留」
#   signal-not-forwarded  SIGTERM 不转给被包的命令
#       红：「SIGTERM 转给被包的命令」
#   drain-unbounded  被包命令退出后一直等管道关
#       红：「后台孙进程占着管道不挂住」
#   quiet-forwards  --quiet 也转打
#       红：「--quiet 只写日志不转打」
#   strict-decoding  按 UTF-8 严格解码，遇到非 UTF-8 字节就崩
#       红：「非 UTF-8 字节原样转打」
#   sigint-ignored-inherited  本进程用 SIG_IGN 忽略 SIGINT（被包的命令 exec 之后继承「不理 Ctrl-C」）
#       红：「被包的命令的 SIGINT 仍是默认处置」
# 设了开关再跑 --selftest：照它判一遍，列出红的格、退 1，开关轮不跑；不设时末了跑开关轮，逐个打开、核红的格与上面写的逐格相等（two_logs.py 的 BREAK_RED_CELLS），
# 每个开关名都要写在这一段里。
# 退役判据：入口脚本与门禁不再各写各的 .log、日志两份的约定改了，或这一份被别的日志工具取代，删掉它与 two_logs.py、two-logs-guard.sh。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}"
exec python3 "$(dirname "${BASH_SOURCE[0]}")/two_logs.py" "$@"
