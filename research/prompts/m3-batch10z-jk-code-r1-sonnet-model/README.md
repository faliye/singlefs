# m3-batch10z-jk-code-r1 正推腿的模型与日志

都建在冻结副本 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1/tree/` 的拷贝上（拷贝与冻结副本 `diff -rq crates` 无差），不进仓内 crates。

- `src/probe_jk.rs`：改自第七八九块实现员的 `probe_10zj.rs`（`~/.local/share/singlefs/work/m3-r1/batch10z-j/keep/`）：加了 `pre`（先做几次发布）、`post`（R 之后再发布）、
  Q 的回退、故障 `qacct`、`rlin`（R 写的谱系打包单元读回全零）、`rold`（R_old 的根槽读回全零）、`roldacct`（R_old 的树表与分配记录树根读回全零）、`sacct`。
  放进拷贝的 `crates/singlefs-harness/tests/`，需要同目录的 `common_unwitnessed_publish`。
- `src/probe-arms-on-mount.diff`：加在拷贝 `mount.rs` 上的环境变量臂（`SINGLEFS_PROBE_ARM`）：`tri`（认回退三态：点名的打包单元有读不出的 ⇒ 当作可能是回退，改并现行账里全部已释放的落点；
  认出回退而 R_old 或它的账读不出同样），`control`、`none`/`today`、`chain-off`、`anylin`（不比 rollback_txg 的谱系记录）。`implemented`（空串）是入库代码原样。
- `src/probe_kept_leases_on_a_lost_device.rs.txt`：账层探针（第十一块 K4-2/K6-2 那一形）。
- `scripts/run.sh`、`runjobs.sh`（并行起一批）、`agg.py`（汇总）；`jobs-*.txt` 是三批的件表；`mutdrv.py` 加 `muts-*.tsv` 与 `extra-*.json` 是变异驱动与变异行；
  `scan-model/` 是第十一块 checker 档两条扫源码用例的逐字搬运，`scanmut.py` 在冻结副本的源文件上套变异后喂它。
- `logs/`：各批汇总；`runs/`：每一件的原始输出（`.log`）与退出码（`.rc`）。
- 重跑要的命令都经 `scripts/run-with-memory-cap.sh` 与 `capped.sh`；release 构建探针：`cargo test --release --offline -p singlefs-harness --test probe_jk --no-run`。
