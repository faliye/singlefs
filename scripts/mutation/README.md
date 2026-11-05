# scripts/mutation

变异：实验装置的变异驱动、crates 变异表的判法、实现员证红、广谱变异。变异表在 `tables/`（一个实验一张，`e<编号>_<简称>.tsv`）；crates 的变异表是 `crates/mutations.tsv`。

| 文件 | 做什么 |
|---|---|
| `mutate.sh` | 在 `scripts/experiment-apparatus/` 下逐条注入变异、跑测试、记红了哪几个；变异只改副本，副本根下照层级链回仓里的是 `scripts/hosts/lib_repository_copy.py` 的闭包里副本还没有的那几样（装置编译期 `include_str!` 读的 `.claude/heavy-thread-default` 这类、运行期读的与它们 source、起的；`research/results/` 另链）；每条判定按函数单元键写进 prune-server（mutation-units 族；库名 `<库根的 git common-dir>/singlefs-mutate-library` 只当名字与命名空间，本地不建、不读、不写；取不到 git common-dir 就退 2、不退到别的键前缀），键对得上的复用，`--full` 只写不读 |
| `prove-red.sh` | 实现员在仓副本里逐条证 `crates/mutations.tsv` 的行会红 |
| `crates-mutation-rows.py`、`mutation-shard-run.sh` | 门禁 crates-mutation-replay 的判法、分片驱动（照 task-server 分片协议的计划、各片、合并三步，经 `task-server/task submit` 单独交，门禁不在一件任务里面调它）：每行一条判定，行键是被改坏的函数单元、原文、替换文、点名测试的单元闭包、判法单元、工具链与依赖版本；一份库（prune-server 里 `mutation-units/singlefs-crates-mutation-library/`，库根照被判那棵树的 git common-dir 取、只当名字），两片判的都写进这一份，合并那一步只按此刻的行键核两片判的行在服务里读得到；变异复跑真跑测试的每一次各对自己起的一份临时 prune-server 跑（不读、不写真库），`crates-mutation-rows.py` 文件头「变异复跑的崩溃判定」 |
| `lib_mutation_units.py` | `crates-mutation-rows.py`、`broad-mutation.py` 共用的函数单元键与按条判定库：被改坏的单元、点名测试是哪个单元、闭包键（切分与闭包调 `scripts/gate/lib_function_units.py`）、工具链行、依赖版本行（闭包真点名的外部 crate 与传递依赖，规则与 `crates/singlefs-unit-graph` 的 `--lock-file` 相同；`mutation-prune.py` 也调这一份）、库的存放与多久没被引用就删 |
| `broad-mutation.py`、`mutation-prune.py` | 广谱变异的驱动、剪枝规则（第二段 验证包的测试里，登记的崩溃枚举用例与放量分项经 `--skip` 排掉，排除单读 `scripts/gate/admission.py` 的 `registered_test_functions`）：清单与结论都按单元键存在判定库服务里（`scripts/prune-server/prune_server_client.py` 的族 broad-mutation，没有「一版」与版本目录；库目录 `<数据根>/broad-mutation-units/` 只放运行目录、日志与锁）；每条变异的键是被变异的函数单元与它调到的单元、原文、替换文、抓它的测试的单元闭包、工具链与依赖版本、判法单元指纹（切分与闭包调 `scripts/gate/lib_function_units.py`），不带参数就是剪枝（沿用得了沿用、沿用不了现判）；不按提交、不按 HEAD 选条 |

审核回归：`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- python3 scripts/mutation/tests/test_audit_regressions.py --selftest`（等于在临时 prune-server 底下跑 `python3 -m unittest discover -s scripts/mutation/tests -v`；判定库那几条只连临时服务，裸跑 `--selftest` 退 2；门禁 code-tooling 的 research-script-selftests 格照这一句跑）。覆盖配置取值、缓存键、并发索引合并、过期回收、信号退出、参数取值，以及证红的副本边界、基线和源码恢复；使用临时目录与假命令，不跑真实变异。各驱动原有的 `--selftest` 仍需单独运行。
