# 运行记录：m2-supp3-item2-code-r2 本地攻方

提示：`research/prompts/m2-supp3-item2-code-r2-local-attack.md`
核对表：`research/prompts/m2-supp3-item2-code-r2-local-attack-translation-audit.md`
调用：`nice -n 19 bash research/scripts/ask-local.sh <提示文件>`，前台跑，未用 `setsid` / `&` / `disown`。

跑前检查：`ps -o pid,args -u "$(id -u)"` 两次（每次调用前各一次）均未见 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`、别的 `ask-local.sh` 或对 `:8200` 的并发调用。

| 次序 | 输出文件 | 退出码 | 词数（wc -w） | oov-check.py 判定 | oov-check.py 列出的生词（受脚本自身 300 字符截断，非本 agent 截断） | 拼接 | 人工通读（缺词/断句/孤立标点） | 归类 |
|---|---|---|---|---|---|---|---|---|
| s1 | `m2-supp3-item2-code-r2-local-attack-output-s1.md` | 0 | 733 | 绿（`绿 ... 生词=19 拼接=0`） | `overturned AllocationRecordsExceedOneNode countable` | 0 | 16 问全部作答，每问后都有一行「Would be overturned by」；未见缺词、断句或列表里只剩孤立标点的情形；生词表里三个词都是真实存在的英文词或提示自己给出的枚举成员名（`AllocationRecordsExceedOneNode`），不是缺头粘连词 | 干净 |
| s2 | `m2-supp3-item2-code-r2-local-attack-output-s2.md` | 0 | 898 | 绿（`绿 ... 生词=19 拼接=0`） | `overturned AllocationRecordsExceedOneNode version's` | 0 | 16 问全部作答，每问后都有一行「Would be overturned by」；未见缺词、断句或列表里只剩孤立标点的情形；生词表里三个词都是真实存在的英文词（含所有格 `version's`）或提示自己给出的枚举成员名，不是缺头粘连词 | 干净 |

两次调用、两份干净样本，达到「至少两份干净样本」的停止条件，未再继续抽样。无判红、无作废副本（`m2-supp3-item2-code-r2-local-attack-output-void*.md` 不存在，已用 `ls` 现查）。

按定义「没做什么」：不解读、不总结、不采纳两份样本回答的具体数值、映射结果或方向是否一致，那是主 agent 的事；模型答复里如出现行号，一律记「模型自给、未核」——现查两份样本全文，均未出现任何形如文件名加冒号加数字的行号引用。
