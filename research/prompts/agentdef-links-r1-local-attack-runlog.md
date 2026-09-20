# agentdef-links 第一轮 本地攻方：运行记录

提示文件：`research/prompts/agentdef-links-r1-local-attack.md`（攻击面：门禁 71 号那条线、kb-discipline 第 4 条）。
核对表：`research/prompts/agentdef-links-r1-local-attack-translation-audit.md`。
前台调用，未用 `setsid`/`&`/`disown`。跑前 `ps` 检查过 `qemu-system`/`vm-bench.sh`/`e152-file-system-benchmark`/`fio`/`cargo`/`gate.sh`，均无相关进程在跑。

## 逐次调用

| 序号 | 落盘文件 | 退出码 | 词数 | 生词/拼接（oov-check.py） | 判定 |
|---|---|---|---|---|---|
| 1 | `agentdef-links-r1-local-attack-output-void1.md` | 5 | 未数（作废） | 未跑到 oov-check：被 `corruption-check.py` 先判红——ACRO_DUP 规则把提示与答复里的日期占位符 `YYYY-MM-DD` 当「缩写自粘」（`YY`+`YY`），提示文件自身先判红（cjk=0 words=2779 fffd=0 缩写自粘=5：YYYY×5），答复也判红（words=2512 缩写自粘=6：YYYY×6） | 作废（工具误判，非模型损坏；改小写 `yyyy-mm-dd` 后同一提示复核判绿，见核对表「工具兼容性改写」一节） |
| 2 | `agentdef-links-r1-local-attack-output-void2.md` | 5 | 未数（作废） | 未跑到 oov-check：被 `corruption-check.py` 判红——实词自复读 1 处：`prohibiting`（提示本身这次判绿） | 作废 |
| 3 | `agentdef-links-r1-local-attack-output-s1.md` | 0 | 2489 | oov-check：绿，生词=0 拼接=0；通读一遍未见缺词、断句、孤立标点等损坏 | 干净 |
| 4 | `agentdef-links-r1-local-attack-output-void3.md` | 5 | 未数（作废） | 未跑到本轮记录用的 oov-check：`ask-local.sh` 内建的 oov-check.py 判红——拼接 1 处 `traceability(=trace+ability)`，生词 4 个：traceability / citations / Citations / substituting | 作废 |
| 5 | `agentdef-links-r1-local-attack-output-s2.md` | 0 | 3095 | oov-check：绿，生词=3（去重后 citations、discrepancies）拼接=0；通读一遍未见缺词、断句、孤立标点等损坏 | 干净 |

干净样本数：2（s1、s2），达到「至少两份」的门槛，未连续五次调用都拿不到两份干净（第 3 次与第 5 次即分别产出两份干净样本），停止抽样。

## 没做什么

- 未解读、未总结、未采纳本地模型的答复内容；样本方向是否一致不在本报告范围。
- 未修改 `research/scripts/corruption-check.py`、`research/scripts/oov-check.py` 或 `research/scripts/ask-local.sh`：void1 的假阳性（`YYYY` 占位符触发 ACRO_DUP）只在自己写的提示文件里改用小写占位符 `yyyy-mm-dd` 规避，未碰检测脚本。
