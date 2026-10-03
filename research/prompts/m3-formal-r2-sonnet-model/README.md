# m3-formal-r2 辩方腿模型目录

译本与分析脚本，都是 Python 3.12，不编译 Rust、不装工具。**译本是读冻结副本 `recovery.rs`（sha256 `f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb`）与 `research/prompts/e165-preregistration.md` 5.2、5.3 写的，不是 Rust 本身**；它对上了 E165 对抗域 A 的各计数，只说明翻译与 E165 在这个域上逐数相等。

| 文件 | 作用 |
|---|---|
| `reader_rule_model.py` | 对抗域 A 的实现译本（`impl`）与 S 的三种读法（`spec`），`python3 reader_rule_model.py <n> [monotone]` |
| `run_parallel.py` | 同上，按第一个槽切成 73 份、至多 4 个进程，`python3 run_parallel.py <n> [monotone]` |
| `agreement_where_readings_differ.py` | 读法两两不同的态上实现等于哪一种，`python3 agreement_where_readings_differ.py <n>` |
| `mutant448_equivalence.py` | `crates/mutations.tsv` 第 448 行（`recovery.rs:3115` 换成 `|| false`）在 A 域上与原实现逐态比 |
| `ledger_by_criterion.py` | E165 第二段红例账按变异、按族汇总首红例里多出的条款判据，在仓根下跑 |
| `model_outputs.txt`、`ledger_by_criterion.txt` | 上面各脚本这一次真跑的原样输出（命令在输出里逐条写了） |

复跑（在本目录下，n = 4 的 `run_parallel.py` 约 52 秒、单态扫描 4 个进程）：

```
nice -n 19 python3 run_parallel.py 4
nice -n 19 python3 run_parallel.py 4 monotone
nice -n 19 python3 agreement_where_readings_differ.py 4
nice -n 19 python3 mutant448_equivalence.py 4
```

与 E165 对账：n = 1…4 各分项相加等于 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out` 第 84 行（A 域总数）：态数 57 585 320、实现报错 12 435 792、施加至少一条 1 473 356、三条 AS 330 240 / 154 624 / 488 224、A-互三个数 175 616 / 509 216 / 333 600、J6 前提真违例 85 400、序号 0 被拒收 45 815 520、A-结构 0。
