# 第 10y 批代码轮第一轮正推腿的模型与原始输出（2026-10-07）

报告：`research/prompts/m3-batch10y-code-r1-sonnet-output.md`。这里的东西都是探针与脚本，不进仓的 `crates/`；跑在冻结副本的拷贝上。

## src/

| 文件 | 干什么 | 对应报告 |
|---|---|---|
| `sonnet_probe_nvidia_smi_report.rs` | 本机真跑 `nvidia-smi -q -d MEMORY,PIDS` 喂给新解析，与 `--query-gpu` 比；再喂几种改坏的报告（里面读 `smi-q-1.txt` 用的绝对路径要改成你机器上放它的地方，原样在 `out/smi-q-1.txt`） | K1 第 1 条 |
| `sonnet_probe_k2_returned.rs` | 真账 `GpuMemoryLedger` + 假读数：长借据在时已销账的短借据让此刻能借的一路变少 | K2-2 P1 |
| `sonnet_probe_k2_property.rs` | 随机 300 万组：`room_now` 有没有超过现读的空闲 | K2-1 |
| `sonnet_probe_k4_sweep.rs` | 5 种读回全零集合 × k ∈ {3,5,7} × M1 之后 0–30 次覆盖写：I-7.4、卸载、抬 F、再挂载 | K4-2 |
| `sonnet_probe_k4_units.rs`、`sonnet_probe_k4_units2.rs` | R 的单个单元在 M1 里读回全零：M1 之后与「写行根落盘之前断电」两个盘面 | K4-1 |
| `sonnet_probe_k4_records.rs` | R（顺序写三个数据单元）的记录、记录加数据单元读回全零 | K4-2 |
| `mutate-harness.py`、`rows-10y-harness.tsv`、`rows-extra.tsv` | 变异行逐条套在草稿副本上跑指定用例，报红还是绿，跑完还原；`rows-10y-harness.tsv` 是 `crates/mutations.tsv` 第 3180–3194 行原样，`rows-extra.tsv` 是我自己的 13 条 | K3 |
| `patch-c-without-candidate-exemption.diff` | 只改 `crates/singlefs-core/src/mount.rs` 一行：C 那一圈循环不看 `referenced_by_candidates`，坐实 K4-1 的机理 | K4-1 |

## 怎么重跑（都在草稿副本里，不在仓里）

```
rsync -a --exclude=/target --exclude=/.git --exclude=/scripts/experiments/target ~/.local/share/singlefs/work/m3-batch10y-code-r1/tree/ <草稿>/repo/
cp src/sonnet_probe_*.rs <草稿>/repo/crates/singlefs-harness/tests/
cd <草稿>/repo && CARGO_TARGET_DIR=<草稿>/target bash scripts/capped.sh 4 bash scripts/run-with-memory-cap.sh 6G \
  cargo test -p singlefs-harness --test sonnet_probe_k4_units2 -- --nocapture --test-threads=1
```

（`cargo test` 一律经 `run-with-memory-cap.sh`；变异脚本里已经套了，用法 `python3 mutate-harness.py <行文件> <从第几条> <到第几条>`，里面写死了草稿目录 `~/.local/share/singlefs/work/m3-batch10y-code-r1-sonnet/repo`。）
K4-1 的补丁对照：`patch -p0` 应用 `src/patch-c-without-candidate-exemption.diff` 到草稿副本的 `crates/singlefs-core/src/mount.rs`（diff 头里的路径是我的草稿路径，按需改），再跑 `sonnet_probe_k4_units2`。

## out/

各探针与变异的原始输出（`probe1.log`、`lib1.log`、`probe-k2.log`、`k4-*.log`、`bc-tests.log`、`mut-*.log`）与本机 `nvidia-smi` 原样（`smi-q-1.txt`、`smi-query-1.txt`）。
`mut-extra-1.log` 里 X1 那一行写「旧串命中 0 次，跳过」是第一次跑时我写错了缩进；改好后重跑的结果在 `mut-extra-0.log`。
`k4-sweep1.log` 是后台任务（自动转后台，没有 `run_in_background`）跑完的，`real 8m53s`。
