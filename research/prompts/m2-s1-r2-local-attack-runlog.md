# m2-s1-r2 本地攻方：运行记录

提示文件：`research/prompts/m2-s1-r2-local-attack.md`（攻击面：I5、I6，判据表见 `research/prompts/_m2-s1-r2-body.md` 第八节）。
核对表：`research/prompts/m2-s1-r2-local-attack-translation-audit.md`。
前台调用，未用 `setsid`/`&`/`disown`，`nice -n 19` 跑。跑前 `ps -o pid,args -u "$(id -u)"` 检查过 `qemu-system`/`vm-bench.sh`/`e152-file-system-benchmark`/`fio`：均无相关进程；仅另一会话在跑 `nice -n 19 bash .claude/scripts/gate.sh --staged`（非性能测量，未影响本轮）。

## 逐次调用

| 序号 | 落盘文件 | 退出码 | 词数（corruption-check.py words=） | oov-check.py 生词 | 判定 |
|---|---|---|---|---|---|
| 1 | `m2-s1-r2-local-attack-output-s1.md` | 0 | 837 | 15（`Falsifier dimensionless recomputation`，去重前含 15 个不同生词，均为真实英文词/派生词，无粘连信号） | 干净 |
| 2 | `m2-s1-r2-local-attack-output-s2.md` | 0 | 569 | 16（`Falsifier dimensionless contradicting superlinearly`，均为真实英文词/派生词，无粘连信号） | 干净 |

两次调用都走了 `ask-local.sh` 内建的两道检测器（`corruption-check.py` + `oov-check.py`），均判绿，退出码 0；未触发字词损坏闸（退出码 5），无作废副本产生。两次之外另单独跑 `python3 research/scripts/oov-check.py <样本>` 复核生词表（步骤 5 要求，即便闸已判绿）：两份样本的生词均为登记词表之外的真实英文词或规则派生词，没有见到缺头的粘连词（例如 `achievesceives` 那一类），未判「带损坏」。另通读两份样本全文，未见缺词、断句、孤立标点等损坏形态（例如列表里整词缺失只剩标点）。

干净样本数：2（s1、s2），达到「至少两份」门槛，两次调用即达标，未连续五次调用都拿不到两份干净。

## 没做什么

- 未解读、未总结、未采纳本地模型的答复内容；两份样本回答的方向是否一致（例如 Section 1 问题 1 里两份样本对「k 个 fsync 如何到达」给出的假设不同：s1 假设固定到达间隔 Δt、s2 假设同时到达）不在本报告判断范围，交主 agent 判。
- 未修改 `research/scripts/corruption-check.py`、`research/scripts/oov-check.py` 或 `research/scripts/ask-local.sh`。
- 未处理禁读清单内的文件（`research/prompts/m2-s1-r2-*` 除本条腿自己的产出外、`research/prompts/_c381-r2-*`、`c381-r2-*`、`research/prompts/e152-*`），未读、未引。
- I1–I4、I7、I8 六格不归本条腿，未碰、未答。
