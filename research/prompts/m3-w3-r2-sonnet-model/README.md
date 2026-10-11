# m3-w3-r2 辩方腿（Sonnet）的模型

在 m3-w3-r1 攻方腿（Opus）的两套装置上加东西，只答 K1（复核第一轮判决）。副本上的数不是入库装置上的数，我加的形态被攻过零轮。

- `timeline-model/`：抽象时间线模型（不依赖 `crates/`）。比攻方腿的多：墓碑（跟现行那一版走、第一版不回收）、谱系树 (T_rb, T_old)、
  「现有检查」I-9.29（含谱系上的墓碑）与 I-9.6（含墓碑那一半）的并集口径、持久镜像计数、C1b / C2i / C4g 等最强形态、V3 臂（恢复时水位少算 1）、
  `W3_OPS` / `W3_LONGEST` / `W3_RING` 环境变量（把历史拉到超过环长）、`replay` 模式（按给定序列在给定环长上逐步打印）。
- `w3r2_module.rs`、`make_device.py`：E175 装置（`research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs`）上的影子记录模块与生成脚本，
  加 C1b、C2i、C4g、C2ig 与持久镜像计数；其余沿用攻方腿。
- `tools/derivation/`：从攻方腿的三份文件（`research/prompts/m3-w3-r1-opus-model/` 下的 `timeline-model/src/main.rs`、`w3_module.rs`、`make_device.py`）生成上面三份的补丁脚本；`rerun.sh derive` 重新生成并逐字节比。
- `rerun.sh`：分段复跑，每段一条前台命令；`compare.sh`：把输出与 `expected/` 比；`expected/`：模型输出（去掉 W3TRACE 行）与装置输出的摘要。
- 复跑顺序（在空目录 W 里；`SINGLEFS_REPO` 取仓根）：
  `derive build-model build-P build-V1 quick model-main model-defense model-mutation model-ringwin model-replay model-deep dev-g-long-v1-idx24 dev-g-long-v1 dev-f2-v1 dev-g-long-p dev-f2-p-1 dev-f2-p-2 compare`，
  最后一段末行 `compared=37 status=0`。编译 6G、跑 4G，线程 6，经 `scripts/hosts/run-with-memory-cap.sh`；只跑 harness 档。
- 看表：`python3 tools/table2.py <候选,候选…> expected/model-defense-0.out`（模型），`python3 tools/mut_table.py <含 mutation-0..5 的目录>`（变异），`python3 tools/agg.py <装置结果文件…>`（装置合计）。
