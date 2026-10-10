# 9-6 第 1 条（I-7.9 两个一圈窗口）补丁：主 agent 定了再打（2026-10-04）

前提：先打 `/tmp/claude-1000/m3-r1/batch9-checker/patch/`，再打这一目录（`crates.patch` 以那一份打完的树为底）。这一条在主报告里停下交主 agent（`/tmp/claude-1000/m3-r1/batch9-checker/patch/report.md` 第六节 Q1）：条款字面是窗口锚在被判那条抬 F 的根上；这一份**另并一个锚在镜像根环里最新有效根上的窗口**（候选过滤那一个），是条款没写的改法，只有主 agent 定了用它才打。

## 改了什么

- `crates/singlefs-checker/src/walk.rs`：`rollback_floor_ceiling_before_the_raise` 的「之前的根」加两道窗口过滤（锚在被判那条根上、锚在镜像最新有效根上）；`judge_rollback_floor_raises_against_their_ceilings` 多一个参数（`check_pool_image` 里候选过滤用的那个 `window`）；两处文档注释写明理由，删掉「缺了它们重算只会更宽（抓不到，不会误红）」那句；`OneRevolutionWindow` 的字段 `newest_valid_root_txg` 改名 `anchor_txg`（锚由调用方取，原名在 I-7.9 那一处说错了）。带卸载记号那一支没动（条款写明照旧）。
- 新用例 `crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs` 四条：删除与并发那条线的草稿（`a_remount_over_a_root_ring_hole_reddens_an_old_floor_raise`）翻成判绿；inject-6 种子 1 那一形（随机历史「抬 F 之后复用」、4 GiB、20 步、种子 1，按 (实例 5, txg 25) 找那一段扣下）；K1 类那一形（按 (实例 6, txg 26) 找）；坏镜像（种子 1 那一形挂载加一次发布之后，把 txg 9 那条根的 F 从 5 改成 7、重封根槽：照旧红在「从 0 抬到 7」）。崩溃状态各只取一个，不枚举，不归 checker 档。
- `mutations-append.tsv` 4 行：三形各一行「只取锚在被判那条根上的窗口」，坏镜像一行「准入那一支恒判成立」。`mutations-replacements.tsv` 2 行：主表「收口表第 26 行 I-7.9：有效根按抬之后的 F 取」（调用多了一个实参，原文跟着改）与「一圈窗口：checker 的窗口算错一位」（字段改名）。

## 验证

两个补丁都打的那棵树：`run3`（`/tmp/claude-1000/m3-r1/batch9-checker/logs/run3-harness.log`）七个二进制全绿，其中这一份 `test result: ok. 4 passed; 0 failed`；那之后只改了注释与 `cargo fmt`。只取条款字面那个窗口时三形用例红（`logs/run2-harness.log`），即这一份前三行变异各自要抓到的那一形。随机历史两批的数在主报告第六节 Q1 的表里。4 行变异都没经 `prove-red.sh` 证（额度用完，主报告第五节）。clippy 在这棵树上跑过（主报告第四节，只有别人文件的两处 `shadow_unrelated`）。

## 受影响的层 0 流与崩溃枚举用例

（推的，没跑。）`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 里抬过 F 的三条流「要求真被评估过」那几格、`crash_enumeration_floor_raise_pushed_by_admission.rs`、`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`：镜像比抬 F 那条根新出一圈以上时那条根改成不判，「真被评估过」的计数可能变少；原先在环上有洞的状态里误红的转绿。GPU 判器的 I-7.9 要取同样两个窗口，否则 GPU 对拍在这些状态上分叉。

## 没做什么

没走三方对抗；checker 档一样没跑；带卸载记号那一支没取窗口（Q1 ②）；没提交。
