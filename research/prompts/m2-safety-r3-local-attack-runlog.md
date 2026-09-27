# m2-safety-r3 本地攻方腿：运行记录

提示文件：`research/prompts/m2-safety-r3-local-attack.md`；核对表：`research/prompts/m2-safety-r3-local-attack-translation-audit.md`。

## 调用记录

| 调用 | 命令 | 退出码 | 结果文件 | 词数 | oov-check 生词 | 判定 |
|---|---|---|---|---|---|---|
| 1（s1 第一次） | `bash research/scripts/ask-local.sh research/prompts/m2-safety-r3-local-attack.md > research/prompts/m2-safety-r3-local-attack-output-s1.md` | 5 | `m2-safety-r3-local-attack-output-s1.md`（重定向建出，空文件，0 字节）；作废副本 `m2-safety-r3-local-attack-output-void1.md`（285 词，脚本自己写） | 0（s1 空文件）/ 285（void1） | 未跑（`ask-local.sh` 在字词损坏闸判红时提前 exit 5，走不到调用方这一步） | 作废：`corruption-check.py` 判红的是**提示文件**本身（`cjk=12 粘连=7`），不是模型答复；粘连全部是 Rust `::` 命名空间与本轮日志格式 `+:`/`-:` 记法触发的假阳性（见核对表第 8 节），改完提示后未再复现 |
| 2（s1 沿用同一个号重跑） | 同上 | 0 | `m2-safety-r3-local-attack-output-s1.md` | 258 | 1（`CheckpointReservePool`，提示原文里就有的合法标识符，不是缺头粘连词） | 干净 |
| 3（s2） | `bash research/scripts/ask-local.sh research/prompts/m2-safety-r3-local-attack.md > research/prompts/m2-safety-r3-local-attack-output-s2.md` | 0 | `m2-safety-r3-local-attack-output-s2.md` | 282 | 1（`CheckpointReservePool`，同上） | 干净 |

已够两份干净样本（s1、s2），按定义第 6 条在此停止，不再往下调用。

## 逐份判定依据

- s1、s2：退出码 0；`oov-check.py` 判绿，唯一生词 `CheckpointReservePool` 是提示 fact A5 里原文给出的合法标识符（`space_budget_of_role` 的一个枚举值名），不是缺头粘连词（不同于 `achievesceives` 这类），核对时按「一律记带损坏」的口径复核过其构词——`Checkpoint` + `Reserve` + `Pool` 三个完整词直接拼接，无缺字——判定不算损坏。通读全文（各 4 道题的编号列表、Question 4 的三行表格）未见缺词、断句或列表内孤立标点的损坏形态。
- void1：`corruption-check.py` 对提示文件本身判红（`cjk=12 words=2096 粘连=7`），触发 `ask-local.sh` 退出码 5，按定义第 4 条这一份作废、不算一次观测；此后改了提示文件（见核对表第 8 节），未改变任何事实或数字，只改了标点相邻处的空格与三处文件名的中文字符，之后再跑同一份提示未再判红。

## 本地腿是否缺席

未缺席。所有调用要么是 exit 5（判红作废，按定义处理）要么是 exit 0；没有出现非 0 非 5 的退出码，也没有出现网关不通。
