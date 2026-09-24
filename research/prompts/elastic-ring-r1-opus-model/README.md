# elastic-ring-r1 云端攻方腿（Opus）的模型与探针

四份东西，两类：

| 文件 | 类 | 怎么跑 |
|---|---|---|
| `geometry_probe.rs`、`field_offset_probe.rs` | Rust 探针，**只在仓副本上跑过** | 见下 |
| `geometry_probe-output.txt` | 上面两份这一次的原样输出 | — |
| `shrink_replay_model.py` + `-output.txt` | 纯算术模型（不读盘、不编译） | `python3 shrink_replay_model.py` |
| `k5_capacity_scan.py` + `-output.txt` | 纯算术模型（不读盘、不编译） | `python3 k5_capacity_scan.py` |

## Rust 探针怎么复跑

主工作区一个字都没动。副本在 `/tmp/claude-1000/elastic-ring-r1-opus/repo`，是
`rsync -a --exclude target --exclude .git` 出来的。复跑：

```
rsync -a --exclude target --exclude .git <仓>/ /tmp/<你的草稿>/repo/
cp geometry_probe.rs   /tmp/<你的草稿>/repo/crates/singlefs-harness/tests/zz_elastic_ring_probe.rs
cp field_offset_probe.rs /tmp/<你的草稿>/repo/crates/singlefs-harness/tests/zz_ring_field_offset2.rs
cd /tmp/<你的草稿>/repo && nice -n 19 cargo test -p singlefs-harness \
    --test zz_elastic_ring_probe --test zz_ring_field_offset2 -- --nocapture
```

两份探针都不改被测代码，只调用 `make_filesystem`、`DeviceFreeMap`、
`check_pool_image`、`SystemConfiguration::to_slot` 这些现成的公开入口；
盘是 `singlefs_harness::crash::SparseBlockDevice`（内存稀疏盘），不落文件。

⚠️ 副本上量出来的数不算入库装置上的数（`.claude/rules/three-way-inference.md`
「判决由主 agent 做，不由投票做」一节）。要引，主 agent 在入库装置上重做一次。
