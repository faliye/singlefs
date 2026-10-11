# m3-w3-r3 正推腿（Sonnet）的模型

K1（三个形态并排的代价表）与 K2（Y1 两种取法）用的模型与算术。副本上的数不是入库装置上的数；我并进来的候选与新臂只在这里量过、被攻过零轮。

- `timeline-model/`：抽象时间线模型（不依赖 `crates/`）。底子是 m3-w3-r2 辩方腿的模型（`research/prompts/m3-w3-r2-sonnet-model/timeline-model/`），并进 m3-w3-r2 攻方腿（`research/prompts/m3-w3-r2-opus-model/timeline-model/`）的 C2aL / C3rL 最强形、I-9.30 草案的几种读法与五种新变异，加 Y1 的两个臂（`ProductWithoutAbandonedRoots`、控制臂 `RollbackRingOnly`）、字母 `rollback-blind`、三个「容许跳升」的候选（名字带 `-tol`）。
  老候选与辩方腿的 expected 逐数相等（`rerun.sh compare-r2`），攻方腿的候选与攻方腿的 expected 逐数相等（`rerun.sh compare-r2opus`）。
- `probe/`：两段只接在冻结树副本末尾的测试探针，调真的规划器 `plan_the_tree_after_this_publish` 与 `after_a_container_is_opened_at_the_right_end`，逐发布打重写节点数与树高（`W3COST`、`W3SHAPE` 行）。
- `cost/`：`planner.py` 是规划器的 Python 复刻，`probe_compare.py` 与探针逐行对账（`--red` 先弄坏复刻必须判红），`cost_model.py` 出代价表里的算术，`check_constants.py` 把用到的格式常量逐个在冻结树里 `grep -nF`。
- `tools/`：`table2.py`（单个输出文件排表）、`mut_table3.py`（八种变异排表）、`compare_r2.py`、`compare_r2opus.py`、`report_tables.py`（报告里的汇总表从 `expected/` 现算）。
- `expected/`：模型输出（去掉 W3TRACE 行、排序）与算术、探针、对账的输出。
- 复跑：`SINGLEFS_REPO=<仓根> bash rerun.sh <空目录> <段>`，段的次序与每段做什么见 `rerun.sh` 文件头。冻结树取 `FROZEN_TREE`（默认 `~/.local/share/singlefs/work/m3-w3-r1/tree`）。
  编译 6G、跑 4G，都经 `scripts/hosts/run-with-memory-cap.sh`，只跑 harness 档（探针编的是 `singlefs-core` 的 lib 测试，不碰 checker 档）。
- 依赖的外部东西：`compare-r2`、`compare-r2opus` 读第二轮两条腿的 `expected/`（`research/prompts/m3-w3-r2-{sonnet,opus}-model/expected/`），那批被归档之后这两段要先从 git 历史里取回；`probe-build`、`cost-check` 读冻结树（`FROZEN_TREE`）与 `research/prompts/m3-w3-r1-snapshot/tree-sha256.txt`；别的段不依赖仓里别的东西。
