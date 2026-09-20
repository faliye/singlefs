# 运行记录：m2-supp3-item2-code-r1 本地攻方

提示：`research/prompts/m2-supp3-item2-code-r1-local-attack.md`
核对表：`research/prompts/m2-supp3-item2-code-r1-local-attack-translation-audit.md`
调用：`nice -n 19 bash research/scripts/ask-local.sh <提示文件>`，前台跑，未用 `setsid` / `&` / `disown`。

跑前检查：`ps -o pid,args -u "$(id -u)"` 两次（每次调用前各一次）均未见 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`、别的 `ask-local.sh` 或对 `:8200` 的并发 `curl`；常驻的 `vllm serve ... --port 8100` 与 ray 相关进程是本地模型的后台服务本身，不是并发调用者。

| 次序 | 输出文件 | 退出码 | 词数（wc -w） | oov-check.py 判定 | oov-check.py 列出的生词 | 拼接 | 人工通读（缺词/断句） | 归类 |
|---|---|---|---|---|---|---|---|---|
| s1 | `m2-supp3-item2-code-r1-local-attack-output-s1.md` | 0（此次调用超过 Bash 工具默认 120s 前台超时被工具自动转后台执行，非本 agent 使用 setsid/&/disown；后台任务完成通知 exit code 0） | 1105 | 绿（`绿 ... 生词=7 拼接=0`） | `overturned`（是词表外但真实存在的英文词，非拼接） | 0 | 未见缺词或断句；列表与句子完整 | 干净 |
| s2 | `m2-supp3-item2-code-r1-local-attack-output-s2.md` | 0 | 638 | 绿（`绿 ... 生词=7 拼接=0`） | `overturned`（同上） | 0 | 未见缺词或断句；列表与句子完整 | 干净 |

两次调用、两份干净样本，达到「至少两份干净样本」的停止条件，未再继续抽样。无判红、无作废副本（`-output-void*.md` 不存在）。

按定义「没做什么」：不解读、不总结、不采纳两份样本回答的具体数值或方向是否一致，那是主 agent 的事。
