# E201（三oracle合谋今树整表）第 1 次重跑的问题单

不是为岔路建的重跑：门禁 doc-experiments 的 evidence-in-repo 格判 E201 的变异表改过而 `research/results/` 里没有不比它旧的产物。只写问题与够判条件，不写倾向与已有的数。

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 变异表 `scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv` 此刻的每一行，在今天的树上各是抓到、无效还是没红 | 全部抓到；有无效或没红的行 | 任何一行不是抓到 | 变异驱动 `mutation_driver.py` 对整张表出一份新产物，产物头报的行数等于表里此刻的数据行数，三个数（抓到、无效、没红）各自落定 | 开着 |
| 装置的单测在今天的树上是否全过 | 全过；有红 | 任何一条红（此刻任务 19166 红一条：`test_every_repository_root_path_the_crates_name_outside_test_code_is_copied_and_synchronized`，`['.'] != []`） | 六个单测模块在今天的树上全过；红的那一条定出是装置的同步清单要跟 `crates/` 的改动，还是 `crates/` 那一侧新写的路径不该出现 | 开着 |
| 变异驱动此刻起得来吗 | 起得来；起不来 | 任务 19167 那一次退 2、日志里没有输出 | 定出退 2 的原因并修到驱动出产物 | 开着 |

不在这一次重跑里的：整表两段（`full_table_run.py`）要卡，等 GPU 批处理验证服务代跑测试的接口定下来（现行账本 `records/2026-11-17-subagent账本自上而下清欠.md` 第二百三十二节）。
