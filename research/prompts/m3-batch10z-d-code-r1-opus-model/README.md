# m3-batch10z-d-code-r1 攻方腿的原型

报告：`research/prompts/m3-batch10z-d-code-r1-opus-output.md`。数都是在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree/` 的拷贝上量的，是副本上的数，不是入库装置上的数。

- `src/opus10zd-arms.patch`：打在冻结副本上（`patch -p0`），改 `crates/singlefs-core/src/mount.rs` 一处、根 `Cargo.toml` 加一个成员。环境变量 `OPUS10ZD_ARM` 不设（或设 `h2`）时行为与冻结副本相同。
  - `today`：抬 F 那一串不借 C 那一圈（撤掉 H2 的 `.or_else`，表里没记着的照读盘）。
  - `diskfirst`（攻方改法甲）：借来的 C 只在这一遍读盘读不出那条根的账时才用。
  - `cplusreleased`（攻方改法乙）：借来的 C 再并上现行账里释放代大于那条根 txg 的全部已释放落点。
  - `cplusmountaccount`（攻方改法丙）：可写挂载并 `C_NEWER_ROOTS` 时，较新可读根的账读不出而记录齐的那一支，C 那一槽并上挂载那一刻现行账里仍分配着的全部落点（抬 F 那一串照 H2 借这一份）。
  三个改法只在这份模型上量过，被攻过零轮。
- `src/opus10zd-probe/`：放进副本 `crates/` 的原型包（不是 checker 档包；依赖 checker 档只为 `enumerate_layer0_versions`）。
  - `tests/opus10zd_h2_borrowed.rs`（K2-2）：照入库 `common_unwitnessed_publish` 的录法录 R（种类放开：o 覆盖写、n 建 inode、s 顺序写三个数据单元），崩在 R 的根槽落盘之后、轮换之前；
    可写挂载那一刻 R 自己写的账那几个单元（`acct`）或账之外那几个（`data`，入库 H2 用例那一形）读回全零，F2 退回 P；挂上之后撤故障（`OPUS_PERSIST=1` 不撤），用户动作放开扫。
    `OPUS_CRASH=1` 时对挂载起录的整条流按 `closed_form_state_count` 估量，不超过 10⁶ 的整条流调 `enumerate_layer0_versions` 全量枚举，超过的写 `crash=skipped(closed_form=…)`。
  - `tests/opus10zd_abandoned_slot.rs`（K2-1、K2-3）：入库 `pool_whose_newest_root_was_abandoned_by_a_crash_recovery` 同一造法的被抛弃根 C；`slot-after(-err)` 挂上之后 C 的根槽读坏，
    `slot-zero-at-mount` / `slot-err-at-mount` 只在挂载那一刻读坏；`OPUS_FAULT_DROPS_ON_WRITE=1` 时这个进程写到那一槽就撤故障（x2 那一组）。
- `scripts/spec-*.tsv`：每行一件（件名、环境变量串）；`agg.py`、`aggx.py` 汇总；`batch2.sh` 是草稿目录里分批跑用的，复跑用 `rerun.sh`。
- `out/summary-*.txt`：每组汇总；`out/raw-lines.txt`：几行原样输出与副本上入库用例在三个臂上的结局。

复跑：`SINGLEFS_FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-d-code-r1/tree SINGLEFS_TOOLS=<仓根> THREADS=6 bash rerun.sh <新工作目录> [a1 a2 …]`，
末尾每组一行「summary-<组>.txt 全同」。全跑本机在负载下约 70 分钟（a1 约 15 分钟、a2 约 18 分钟、a3 约 7 分钟、a4–a6 各约 2–5 分钟、x 约 12 分钟、x2 约 8 分钟）。
