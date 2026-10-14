# scripts/mutation

变异：实验装置的变异驱动、crates 变异表的判法、实现员证红、广谱变异。变异表在 `tables/`（一个实验一张，`e<编号>_<简称>.tsv`）；crates 的变异表是 `crates/mutations.tsv`。

| 文件 | 做什么 |
|---|---|
| `mutate.sh` | 在 `scripts/experiments/` 下逐条注入变异、跑测试、记红了哪几个 |
| `prove-red.sh` | 实现员在仓副本里逐条证 `crates/mutations.tsv` 的行会红 |
| `crates-mutation-rows.py`、`crates-mutation-light-rows.py`、`mutation-shard-run.sh` | 门禁 checker-tier-crates-mutation-replay 的判法、轻型登记表、双机分片驱动（变异复跑起的 cargo 写自己的崩溃判定库：真跑家目录底下的 `mutation-replay/`，`crates-mutation-rows.py` 文件头「变异复跑的判定库」） |
| `broad-mutation.py`、`mutation-prune.py`、`broad-mutation-ignored-exclusions.tsv` | 广谱变异的驱动、剪枝规则、忽略的排除项 |
