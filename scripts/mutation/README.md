# scripts/mutation

变异：实验装置的变异驱动、crates 变异表的判法、实现员证红、广谱变异。变异表在 `tables/`（一个实验一张，`e<编号>_<简称>.tsv`）；crates 的变异表是 `crates/mutations.tsv`。

| 文件 | 做什么 |
|---|---|
| `mutate.sh` | 在 `scripts/experiment-apparatus/` 下逐条注入变异、跑测试、记红了哪几个；变异只改副本，装置 `include_str!` 读的仓内别处文件（`research/results/`、`.claude/heavy-thread-default` 这类）在副本根下照层级链回仓里；每条判定按函数单元键入库（git common-dir 底下的 `singlefs-mutate-library/`），键对得上的复用，`--full` 只写不读 |
| `prove-red.sh` | 实现员在仓副本里逐条证 `crates/mutations.tsv` 的行会红 |
| `crates-mutation-rows.py`、`crates-mutation-light-rows.py`、`mutation-shard-run.sh` | 门禁 checker-tier-crates-mutation-replay 的判法、轻型登记表、双机分片驱动：每行一条判定，行键是被改坏的函数单元、原文、替换文、点名测试的单元闭包、判法单元、工具链与依赖版本；一份库（被判那棵树的 git common-dir 底下的 `singlefs-crates-mutation-library/`），第二台判出的当场导入、不留长期库；变异复跑起的 cargo 写自己的崩溃判定库：真跑家目录底下的 `mutation-replay/`，`crates-mutation-rows.py` 文件头「变异复跑的判定库」 |
| `lib_mutation_units.py` | 上面两份共用的函数单元键与按条判定库：被改坏的单元、点名测试是哪个单元、闭包键（切分与闭包调 `scripts/gate/lib_function_units.py`）、工具链行、依赖版本行（闭包真点名的外部 crate 与传递依赖，规则与 `crates/singlefs-unit-graph` 的 `--lock-file` 相同；`mutation-prune.py` 也调这一份）、库的存放与多久没被引用就删 |
| `broad-mutation.py`、`mutation-prune.py`、`broad-mutation-ignored-exclusions.tsv` | 广谱变异的驱动、剪枝规则、第二段慢档用例排掉的大档与工具：清单与结论都按单元键存在一份固定库目录下（`~/.local/share/singlefs/broad-mutation-units/`，没有「一版」与版本目录）；每条变异的键是被变异的函数单元与它调到的单元、原文、替换文、抓它的测试的单元闭包、工具链与依赖版本、判法单元指纹（切分与闭包调 `scripts/gate/lib_function_units.py`），不带参数就是剪枝（沿用得了沿用、沿用不了现判），`--full` 只写不读；不按提交、不按 HEAD 选条 |
