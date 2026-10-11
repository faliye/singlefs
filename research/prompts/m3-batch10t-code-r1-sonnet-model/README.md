# m3-batch10t-code-r1 正推腿模型（Sonnet）

全部是 harness 档：只用 `singlefs_harness::gpu_memory_ledger` 的公开接口、假的空闲读数、假进程，或纯文本；不要显卡，不跑 checker 档包的测试二进制。

## 怎么重建草稿副本

```
S=~/.local/share/singlefs/work/m3-batch10t-code-r1-sonnet
rsync -a --exclude=/target --exclude=/.git --exclude=/research --exclude=/records --exclude=/briefs \
  ~/.local/share/singlefs/work/m3-batch10t-code-r1/tree/ $S/repo/
# 探针放进 harness 的 tests/
cp k_forward_ledger_probe.rs k3_chain_record_differential_probe.rs k4_scan_evasions_probe.rs $S/repo/crates/singlefs-harness/tests/
# scan_copy.rs：冻结副本里扫源码的那份测试文件，只把顶格的 fn / struct 改成 pub、三个字段改成 pub，别的一字不动
sed -E 's/^(fn|struct) /pub \1 /; s/^    (file_relative_to_the_package: String|line_number: usize|code: String),/    pub \1,/' \
  ~/.local/share/singlefs/work/m3-batch10t-code-r1/tree/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs \
  > $S/repo/crates/singlefs-harness/tests/scan_copy.rs
# walk.rs 末尾加垫片（只在草稿副本里，不入库）
cat walk-shim.rs.txt >> $S/repo/crates/singlefs-checker/src/walk.rs
```

## 文件

- `k_forward_ledger_probe.rs`：K1 / K2 / K4-3 的账本探针 A–G（A 开账之后不再读空闲；B 外人走了而同一个进程不知道；C 按状态数借按整张卡的容量压、不按剩下的空压；D 两张卡里有空的那张得 0 个状态；E 三个假进程六条线程并发借，合计不越过估计；F 容量不越过任何一个上界（格子穷举）；G 卡上空闲不足一份驱动开销时登记驱动那一份照样登记得上）。
- `k_forward_ledger_probe.run-output.txt`：A–F 那一次真跑的输出（6 条，199.91 秒，主要是 E）。G 在 `run-g.sh` / `run-g.output.txt`。
- `k3_chain_record_differential_probe.rs`、`walk-shim.rs.txt`：K3-1 CPU 两侧差分（实现的 `InstanceTableChainRecord::parse` 对 checker 的 `chain_record_view`）。
- `k4_scan_evasions_probe.rs`：K4-2 扫源码绕法 1–5。
- `rerun-quick.sh`、`rerun-quick.output.txt`：基线 / 变异（容量那一项）/ K3 差分及其变异 / K4-2 探针的一次复跑；经 `bash scripts/run-with-memory-cap.sh 4G bash rerun-quick.sh` 跑。
- `crstr.rs`：K4-2 绕法 1 的真实 Rust 证明（`cr#"x" "#` 这一行合法，`rustc --edition 2021` 编得过，输出见报告 K4-2 末）。
