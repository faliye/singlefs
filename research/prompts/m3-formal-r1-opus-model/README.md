# m3-formal-r1 攻方腿的模型目录

不装任何工具：在冻结副本的 `replay_journal`（`crates/singlefs-core/src/recovery.rs`）上做有界穷举，真 CRC、真 `JournalRecord::to_bytes` / `parse`、真反向链。副本上的数，不是入库装置上的数。

## 文件

- `formal_probe_replay_bounded.rs`：原型（放进副本的 `crates/singlefs-core/examples/` 当 example 编）。模式：`base`（合法历史的任意记录子集）、`units` / `units2`（点名单元一位 / 两份各一位）、`stray`（另一实例的孤记录）、`residue`（同 jsn 不同内容的残留）、`wrap`（6 槽小环，在飞上限 2）、`edge`（txg 顶到 u64 最大值）、`adv`（自证过、字段任取的记录）、`mono`（根槽也进崩溃状态，量单调两种读法）。判据 R / P / M / X / F 的定义在文件头。
- `campaign.py`：原实现 + 23 条入库变异（`crates/mutations.tsv` 里落在 recovery.rs 第 2930–3230 行的）+ 4 条自造变异，逐个改副本、编、扫。
- `summarize.py`：把 `results.tsv` 汇成报告里的表。
- `results.tsv`、`campaign.log`、`summary.txt`：2026-09-30 这一次跑的原样产物。

## 复跑

```
S=/tmp/claude-1000/<草稿目录>
rsync -a --exclude target --exclude .git /tmp/claude-1000/m3-ring-r1/tree/ $S/tree/
mkdir -p $S/tree/crates/singlefs-core/examples
cp research/prompts/m3-formal-r1-opus-model/formal_probe_replay_bounded.rs $S/tree/crates/singlefs-core/examples/
nice -n 19 bash scripts/run-with-memory-cap.sh 6G bash scripts/capped.sh 6 \
  python3 research/prompts/m3-formal-r1-opus-model/campaign.py $S/tree $S/target $S/out 6
python3 research/prompts/m3-formal-r1-opus-model/summarize.py $S/out/results.tsv $S/out/campaign.log
```

这一次：28 个变体共 143 秒（编译在内），`replay_journal` 被判的状态 6 986 364 个（`campaign.log` 末行 `total_states`；F 判据每个状态另调一次，mono 的对数不计在内）。
`results.tsv` 与计数逐字可复现；`campaign.log` 里的 `secs=` 与 `first_*` 举的那一个例子随线程次序变（计数不变）。
