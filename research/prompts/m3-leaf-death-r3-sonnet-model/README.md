# m3-leaf-death-r3 正推腿（Sonnet）模型

第二轮攻方腿模型（`research/prompts/m3-leaf-death-r2-opus-model/`）的另写版本，加了候选二 / 三、同一段历史的做法、三个名字空间、按事件记账。不依赖 `crates/`、不依赖 `research/Cargo.toml`。

- `src/model.rs`：主模型。候选 Today / RetA / RetB / Persist / PersistX / Tsz / Fsz，借法 NoBorrow / N1New / N2 / N1Raise；崩溃（在飞单元写逐子集枚举）、丢根、管理员回退、实例表清扫、unlink 分批；每次发布之后扫盘重建，按 inode、data、units 三个名字空间与真值比。
- `src/clone.rs`：克隆头共享叶的有界穷举（叶容量 2，长度 ≤ 6 的全部序列）。
- `src/main.rs`：配置（`SHAPES`、前提族 `prem_`、历史长度族 `len_`、清扫族 `sweep_`、N1 旧形族）与子命令 `sweep <起>:<止>`、`clone`、`determinism`、`configs`、`names`、`trace`。
- `copy/sonnet_r3_leaf_death_images.rs`、`tools/patch_checker.py`：仓副本里的 harness 测试目标与 I-9.4 第三句开关（真 checker 判手改镜像）。
- `tools/tables.py`（汇表）、`tools/views.sh`（报告里贴的那几张小表）。
- `rerun.sh`、`compare.sh`、`expected/`：复跑与期望，用法见 `rerun.sh` 文件头。完整模型输出（`sweep-*.out`，每份约 1–10 MB）不入库，`expected/sorted-output-sha256.txt` 钉着排序后的 sha256。
- 模型里的数全是副本上的，不进 kb；要引由主 agent 在入库装置上重做。
