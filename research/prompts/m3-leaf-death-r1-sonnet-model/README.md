# m3-leaf-death-r1 正推腿的副本装置

这是 E177 装置（`scripts/experiment-apparatus/e177_per_tree_resurrection_without_tombstones.rs`，sha256 `9061c07b…2848f5`，与冻结树里那份相同）的**副本**加五层补丁，用来量四个候选在 E177 主取样点上的结局。副本上的数不进 kb；要引，先在入库装置上重做。

- `apply_patch.py`：候选 `retire`（容器身份消失写退役记录）、`zero`（删空的容器照样落零条目版本）、`all_suppressed`（只改读侧）；合并政策 `E177_MERGE=absorb | absorb_borrow`（E176 的 M2 / M3 均分借）。
- `apply_patch2.py`：候选 `shrink_range`（文件变短那次发布写区间墓碑，模型里不占槽）。
- `apply_patch3.py`：数事件（删空摘片、左吸收右、借）。
- `apply_patch4.py`：候选 `retire_shrink` = retire + shrink_range。
- `apply_patch5.py`：`E177_TIGHT=literal`（G-紧按登记 5.1 字面读法，只数活槽）。
- `summary.py`：把 16 份产物压成稳定摘要；`--compare` 与 `expected/summary.txt` 逐行比。
- `rerun.sh`：`SINGLEFS_REPO=<仓根> bash rerun.sh <空工作目录> build | run | compare | all`。`run` 一次前台约跑 3 次，已完成的跳过，可反复调用。
- `expected/summary.txt`、`expected/out-sha256.txt`：期望摘要与 16 份完整产物的 sha256（完整产物每份约 10 MB，没入库，重跑得到）。

基线（不设环境变量）的产物与入库的 E177 第一段产物逐行相同，只多一行 `name=candidate_run`（`rerun.sh run` 末尾自检）。
