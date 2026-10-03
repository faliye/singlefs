# m3-ring-r3 辩方腿模型目录（Sonnet，2026-10-02）

副本上的数，不是入库装置上的数，不进 kb。路径写死 `/tmp/claude-1000/m3-ring-r3-sonnet/`（草稿目录），要复跑先把脚本里的路径换成自己的。

| 文件 | 作用 |
|---|---|
| `erase_release_direct.rs` | 第二轮攻方 `window_release_direct` 的拷贝加「抹」一步（`ERASE=1` 时把 slot(h) 写成全零）；放进 `crates/ring-window-proto/src/bin/` 编译 |
| `scripts/build.sh`、`run.sh`、`direct.sh`、`erase.sh`、`sweep-sample.sh` | 第二轮攻方原型的重编、单次运行、放槽量 12 份、抹臂 4 份、sweep1 的 S = 4 取样 |
| `scripts/build-r1.sh`、`run-r1-A.sh` | 第一轮攻方原型 A 矩阵 48 份 |
| `scripts/mkcand.py`、`mkmixed.py`、`mkcand-extra.py`、`build-g3.sh`、`rungrid.sh`、`stats.py` | C331 网格移植：造甲-读错、甲-读错-每盘、混合读回开关、R2 与「今天」基线；编译、跑、汇总 |
| `scripts/diffsteps.py` | 比同一步今天与窗口、抹的放出量 |
| `results/direct/` | 第二轮攻方放槽量 12 份重跑，与原 `m3-ring-r2-opus-model/results/direct/` 逐字节相同 |
| `results/erase/` | 抹臂 4 份 |
| `results/h1/` | 第一轮攻方 A 矩阵 48 份重跑，与原 `m3-ring-r1-opus-model/results/h1/A-*.out` 逐字节相同 |
| `results/sweep/` | 自证会红一份（与原逐字节相同）与 sweep1 S = 4、洞 txg 13..24 的 72 行（与原同键行逐行相同） |
| `results/c331-grid/*.log.gz` | C331 网格 1959 格的原始日志（压缩） |
