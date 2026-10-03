# U1 正推腿的模型与原样输出（2026-10-01）

草稿目录 `/tmp/claude-1000/tooling-detached-run-r3-sonnet/` 里按下面的配方搭仓副本（交回前已删，要复跑就重搭）：

- `work/`：`.claude/hooks`、`.claude/scripts`、`.claude/singlefs-ai-sop` 取自主仓，`research/scripts/detached-run.sh` 取自主仓；5 份被判文件与 `/tmp/claude-1000/tooling-detached-run-r3/tree/` 里的逐字节相同（`cmp`）。
- `work-r1/`：`work/` 的 `.claude` 为底，再用 `/tmp/claude-1000/tooling-detached-run-r1/tree/` 里的 3 份钩子与 `detached-run.sh` 盖上。
- `work-r2/`：同上，用 `/tmp/claude-1000/tooling-detached-run-r2/tree/` 里的 4 份钩子与 `detached-run.sh` 盖上。

| 文件 | 干什么 | 原样输出 |
|---|---|---|
| `feed_u1.py` | 把命令写成钩子输入喂给 `work/` 的两道钩子（不执行命令）；带 `名=值` 参数时设成环境变量 | `feed_u1-plain.out`、`feed_u1-break.out`（`LIB_SHELL_WORDS_BREAK=detached-run-force-kept`） |
| `feed_g4.py` | h4、h5、h7 三类样本；四个开关各跑一遍 | `feed_g4.out` |
| `feed_r1.py`、`feed_r2.py` | 第一、二轮判决引的钩子退出码，在各自的冻结钩子上重喂 | `feed_r1.out`、`feed_r2.out` |
| `alarm_model.sh` | 替身 systemctl 下的 `watch`（只读，不起单元），ActiveState 查询按计划逐次给 F（失败）或 A | `alarm_model.out`（计划 `F F A F F A F F F F A A`） |
| `cite-r1.out`、`cite-r2.out` | `cite-check.py` 在第一、二轮冻结副本上重跑两份报告 | 同名 |

feed 脚本里的路径写死在草稿目录；`alarm_model.sh` 的第一个参数要用 `work/research/scripts/detached-run.sh`（冻结副本 `tree/` 里没有 `.claude/scripts/preflight.sh`，脚本开头的 `source` 落空会把参数清掉，退 2）。
