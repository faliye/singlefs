# m3-prune-gpu-r2 本地攻方运行记录

提示文件：`research/prompts/m3-prune-gpu-r2-local-attack.md`；核对表：`research/prompts/m3-prune-gpu-r2-local-attack-translation-audit.md`。

## 调用记录

| 号 | 路径 | 退出码 | 词数 | 生词清单（`oov-check.py`，只打前 300 字符） | 判定 |
|---|---|---|---|---|---|
| s1 | `research/prompts/m3-prune-gpu-r2-local-attack-output-s1.md` | 0 | 921 | 绿；生词=2 拼接=0；生词: segments'（所有格，非损坏） | 干净：闸判绿，逐段通读没有缺头粘连词、没有缺词断句 |
| s2 | `research/prompts/m3-prune-gpu-r2-local-attack-output-s2.md` | 0 | 973 | 绿；生词=0 拼接=0 | 干净：闸判绿，逐段通读没有缺头粘连词、没有缺词断句 |

两次调用都是退出码 0，都没有触发损坏闸（没有 `-output-void*.md` 副本产生），两份都不是 0 字节。两份样本都读全了，没发现列表里整词丢失、断句、缺头粘连（如 `achievesceives`）这类损坏。

## 算术核对（5b）

主 agent 没有给这一轮的答案表路径（派发提示第四节分工表只给了攻击面，没有另给算好的答案表路径）：**没有答案表，算术没比**。

## 干净 / 参考样本汇总

干净样本 2 份：`research/prompts/m3-prune-gpu-r2-local-attack-output-s1.md`、`research/prompts/m3-prune-gpu-r2-local-attack-output-s2.md`。参考样本（带损坏或作废）：无，本轮两次调用都一次成功、都判绿。

## 没做什么

- 不解读、不总结、不采纳本地模型的答复；两份样本具体填了什么数、两份彼此是否一致，不在这份运行记录里写，判它打没打中是主 agent 的事。
- 没有第三次及以后的调用：两份干净样本在头两次调用内就拿到了，没有触发「连续五次拿不到两份干净的」那条停线。
