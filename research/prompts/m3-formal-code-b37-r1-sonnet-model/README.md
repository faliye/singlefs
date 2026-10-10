# m3-formal-code-b37-r1 云端正推腿的模型

只读脚本，不编译、不改仓：`tools/coverage.py`（B1：三趟对 `Criterion::ALL` 的覆盖与变异推演）、`tools/kinds.py`（B2：三种写的名字与次序）、
`tools/anchors.py`（B4：两行变异的锚点命中数）。`tools/rerun.sh <冻结副本根> [harness]` 一次跑三个；带 `harness` 时再跑一条 harness 档 cargo（经 `run-with-memory-cap.sh`、`capped.sh`）。
`rerun-output.txt` 是对 `~/.local/share/singlefs/work/m3-formal-code-b37-r1/tree` 跑的原样输出。
