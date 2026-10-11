# 正推腿的模型与脚本（m3-reclaim-window-r1）

纯 Python，无第三方包，不与 `crates/` 共用代码。核心 `e139_port.py` 是 E139 装置
`scripts/experiments/e139_tightened_floor.rs` 里活化一族世界、近满盘两个世界（drain_failure、broken_slot）的端口，
逐行复现产物 `research/results/e139-tightened-floor-2026-10-04.out`（`run_all.sh` 第一、四节的 `cmp_*` 输出是对账）。
对账过了才改一处开关再跑：开关在 `Cfg`（`gate`、`failure`、`recover`、`p2`、`p3`、`p3c`、`p3s`、`p4`）。

- `run_all.sh`：大部分数，实测约 2 分 15 秒（16 进程）；输出落进 `out/`。`nice -n 19 bash run_all.sh`。
- `k1_world.py`：实现形的 K1 世界（崩在记录之后、根槽之前，挂载由记录施加，再逐个发布点崩溃加丢盘）。
- `probe_*.py`：单项探针，输出各一份在 `out/probe_*.txt`。其中 `probe_stale_root_exists.py`、`probe_lag_max.py`、
  `probe_b2_ring_all_six.py` 不在 `run_all.sh` 里，单独 `python3 <文件>` 跑（各几秒）。
- 端口只是装置的另一份手写，不是 `crates/` 的实现；它说的是「装置的机制在这些开关下怎么走」，不替实现作证。
