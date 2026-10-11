# 第 10z 批第三块代码轮第一轮：正推腿的模型

都只在草稿目录 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1-sonnet/` 的冻结副本拷贝（`tree/`）里跑；入库的是源码、脚本与输出，编译目录不入库。
冻结副本被判的四份源文件与主工作区逐字节相同（`cmp`）。

- `probes/lost_restart_model.rs`：物理卡模型加「设备丢了、重起、重起被拒」（随机 5 组 + 定向一条）。放 `crates/singlefs-harness/tests/`。
- `probes/rounds_sim4.rs`：第 E 条按轮跑的模拟：照产品代码 `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 的外层循环与
  `judge_on_one_card_and_start_the_next` 的走法，用 harness 档的纯函数搭；随机 3000 格 + 叫停落在第 1、2 轮之间的定向一条。放同一个 tests 目录。
- `probes/killers.rs`：对存活变异补的会红用例草稿（真代码上绿，变异上红）。放同一个 tests 目录。
- `probes/explain_demo.rs`：第 D 条「真 panic 被说成显存不够」两形。放同一个 tests 目录。
- 上一轮两腿的真账探针（`m3-batch10z-a-code-r1-sonnet-model/probes/`、`m3-batch10z-a-code-r1-opus-model/ledger_attack_10z.rs`）原样拷进同一个 tests 目录跑，输出在 `logs/probes-old-legs.log`。
- `mut/mutate_k3b.py`：自造变异驱动（改前串在测试段之前恰好命中一次，改一处、跑、还原）；`_c` 是第二批变异；`_d`/`_e` 是同两份换成不带过滤的 `--lib`；
  `_f`/`_g` 是同两份换成只跑 `--test killers`。脚本里的路径是草稿目录的绝对路径，复跑要改 `TREE`、`TARGET`。
- `logs/`：各条的原样输出（`mut-*.log` 里 M 号对应 `mutate_k3b.py` / `mutate_k3c.py` 里的次序）。

命令（工作目录是副本树根）：
`CARGO_TARGET_DIR=<编译目录> nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 6 cargo test --offline -p singlefs-harness --test <名> -- --nocapture --test-threads 1`
