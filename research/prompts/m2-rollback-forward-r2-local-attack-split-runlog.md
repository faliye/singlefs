# m2-rollback-forward-r2 本地攻方：拆分后运行记录

提示文件：
- `research/prompts/m2-rollback-forward-r2-local-attack-part1.md`（task one：16 格 F_效值 / 回退候选集表）
- `research/prompts/m2-rollback-forward-r2-local-attack-part2.md`（task two：B1 卸载串空发布次数，三种 t mod 3 情形）

拆分核对表：`research/prompts/m2-rollback-forward-r2-local-attack-split-audit.md`。
上一任在同一份未拆分提示上的失败记录（原样不改）：`research/prompts/m2-rollback-forward-r2-local-attack-runlog.md`、`-output-s1.md`（空文件）。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，前台跑，全程未用 `setsid` / `&` / `disown`。跑前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`：零命中，不是性能测量在跑。

## part1（task one，16 格表）

| 号 | 命令 | 退出码 | 词数 | 生词清单 | 判定 |
|---|---|---|---|---|---|
| s1 | `... research/prompts/m2-rollback-forward-r2-local-attack-part1.md > ...-part1-output-s1.md` | 3 | 0（文件 0 字节） | 未跑（网关侧在字词损坏闸之前就报错退出，脚本没走到那一步） | 无判定，本次调用失败 |

`ask-local.sh` 网关侧返回体（JSON error，非字词损坏闸判红）：`{"message": "输出预算不足被截断（已用 0 tok）——该调的是 max_tokens，不是模型或 prompt", "type": "truncated", ..., "diagnoses": ["第1次：正文复读跑飞，已掐断上游（判定时正文 6000 字符）", "第2次：...6005 字符", "第3次：...6003 字符"]}`。`partial_content` 字段（网关截断前的片段，非 `ask-local.sh` 正式输出）显示模型已经在逐行作答 16 格表，但三次重试均在中途复读跑飞被上游掐断，没有一次给出完整正文；退出码落在「响应是 JSON 且带 error 字段」那一支，脚本在写 `$TXT`、跑字词损坏闸之前就 `exit 3`，没有产出作废副本（`ls` 现查 `research/prompts/` 下无 `m2-rollback-forward-r2-local-attack-part1-output-void*.md`），重定向建出的 `...-part1-output-s1.md` 是空文件（`ls -la` 现查 0 字节）。

按派发指令「某一份提示再次撞上网关报错（退出码非 0 非 5），就停下那一份，报『本地腿这一问缺席』，另一份照跑」：这次调用的退出码是 3，已停下 part1，不再重跑。**本地腿这一问（task one，16 格表）缺席**，累计 1 次调用（s1）。

## part2（task two，B1 卸载串空发布次数）

| 号 | 命令 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 判定 |
|---|---|---|---|---|---|---|---|
| s1 | `... -part2.md > ...-part2-output-s1.md` | 0 | 129 | 绿，生词=1（`falsified`，完整词，非粘连/缺头）拼接=0 | 灰，语料太小（cjk=0 words=87），判不了——按脚本内部逻辑该文件不计入 bad，`ask-local.sh` 整体判绿 exit 0 | 三种情形（t mod 3 = 0/1/2）各给出逐次发布的区域与盘归属、总数，闭合问题给出最小/最大值与推翻条件一句；句子完整、无缺词、无断句、无孤立标点、无成对标记落单 | 干净 |
| s2 | `... -part2.md > ...-part2-output-s2.md` | 0 | 122 | 绿，生词=1（`falsified`，完整词）拼接=0 | 灰，语料太小（cjk=0 words=80），判不了，同上按内部逻辑判绿 | 同上，三种情形与闭合问题齐全、完整、无损坏迹象 | 干净 |

两份干净样本达标，part2 到此为止，不再抽样。

## 交回前核查

`ps -o pid,args -u "$(id -u)"` 现查不到任何 `ask-local.sh` 或对网关 `:8200` 的 `curl` 残留进程；交回前后台无遗留。
