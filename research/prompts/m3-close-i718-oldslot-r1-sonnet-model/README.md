# m3-close-i718-oldslot-r1 云端正推腿的模型目录

量什么：满环时根槽写失败、恢复之前的镜像上 I-7.18 该不该红；甲、乙、丙三个候选的判别力。

- `patches/measurement-switch.patch`：量法开关（按环境变量 `I718_VARIANT` 在基线 / 甲 / 甲含种子边 / 乙 / 乙按大于之间切），只给量具用，不是候选。
- `patches/candidate-*-cpu.patch`、`candidate-*-gpu.patch`：候选自己的改法，CPU 一半对 `crates/singlefs-checker/src/walk.rs`，GPU 一半对 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`。
  `candidate-a-*` 是调查员投稿的甲（条件 `>`，GPU 一半是照同一写法补的）；`candidate-a-seed-edge-*` 是改成 `>=` 的甲含种子边；`candidate-b-*` 是乙（条件 `>=`）。
- `probe/probe_i718_oldslot.rs`：七个分组的量具，拷进 `crates/singlefs-harness/tests/` 编；`probe/probe_i718_baseline.rs`：调查员的场景探针（拷自他的草稿目录）。
- `tools/rerun.sh`：整条复跑，`RERUN_ONLY` 选段；`tools/summarize.py`：把日志汇成 `out/summary.txt` 那几张表；`tools/closure.py` 与 `tools/naga-check/`：对 GPU 补丁做 naga 解析与校验（前言常量是假值，只校验判 I-7.18 那个函数的闭包）。
- `out/`：这一趟的汇总（`summary.txt`）、候选自己补丁与开关版的交叉核（`cross-check.txt`）、naga 校验（`naga.txt`）、补丁 sha256、逐行原始输出（`rows/`）。

不跑 checker 档；GPU 走读内核的补丁没上 GPU 跑过。
