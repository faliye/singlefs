# m2-rollback-forward-r2 本地攻方：运行记录

提示文件：`research/prompts/m2-rollback-forward-r2-local-attack.md`
核对表：`research/prompts/m2-rollback-forward-r2-local-attack-translation-audit.md`

## 调用记录

| 号 | 命令 | 退出码 | 词数 | 生词清单 | 判定 |
|---|---|---|---|---|---|
| s1 | `bash research/scripts/ask-local.sh research/prompts/m2-rollback-forward-r2-local-attack.md > research/prompts/m2-rollback-forward-r2-local-attack-output-s1.md`（前台跑，未用 setsid / & / disown；Bash 工具因超过其 120s 默认时限自动转入后台等待，非本 agent 主动后台化） | 3 | 0（文件 0 字节） | 未跑（`oov-check.py` 对 0 字节文件判绿，生词=0，但这不构成「干净样本」——见下） | 无判定，本地腿此次调用本身失败 |

`ask-local.sh` 内部日志（网关侧返回的 JSON 错误体，非本地判红）：

```
ask-local: 网关报错 {"message": "输出预算不足被截断（已用 0 tok）——该调的是 max_tokens，不是模型或 prompt", "type": "truncated", "code": "truncated", "session_id": "33cab272-aa31-468b-a66d-3de3550f10e8", "attempts": 3, "diagnoses": ["第1次：正文复读跑飞，已掐断上游（判定时正文 6008 字符）", "第2次：正文复读跑飞，已掐断上游（判定时正文 6000 字符）", "第3次：正文复读跑飞，已掐断上游（判定时正文 6003 字符）"], "partial_content": "……"}
EXITCODE=3
```

退出码 3 落在 `ask-local.sh` 里「响应是 JSON 且带 error 字段」那一支，脚本在写 `$TXT`、在跑字词损坏闸（`corruption-check.py` / `oov-check.py`）之前就直接 `exit 3`——因此没有产出作废副本（`-output-void*.md` 不存在，已用 `ls` 现查），重定向建出的 `research/prompts/m2-rollback-forward-r2-local-attack-output-s1.md` 是空文件（0 字节，`ls -la` 已现查）。

`partial_content` 字段里网关自己截断前吐出的内容（非 `ask-local.sh` 的正式输出，只是错误体的附带信息）看得出模型在两道题上都已经开始逐格作答（16 行的 Part 3 表格与 t mod 3 的三格都出现在里面），但网关记录「正文复读跑飞」重试了 3 次、3 次都在跑飞后被掐断，最终没有一次给出可用的完整正文。

## 判定：本地腿缺席

按派发指令第 4 条「退出码非 0 非 5、或网关不通：停下，报『本地腿缺席』」：这次调用的退出码是 3（既非 0 也非 5），已停下，不再重跑 s2。**未拿到任何一份干净或带损坏的样本**，累计 1 次调用（s1），未达到「连续五次调用都拿不到两份干净的才停」这一步就先撞上了非 0 非 5 的退出码，按指令这一条优先于「凑够五次」那一条。

后台进程核查：`ps` 现查不到任何 `ask-local.sh` / 对网关 `:8200` 的 `curl` 残留进程，交回前后台无遗留。
