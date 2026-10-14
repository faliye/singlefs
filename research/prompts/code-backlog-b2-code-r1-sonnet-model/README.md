# 代码欠账批甲簇代码轮：正推腿（Sonnet）模型目录

报告：`research/prompts/code-backlog-b2-code-r1-sonnet-output.md`。这里是它「量的」数的来源，只读冻结副本、不改它。

- `tools/rerun.sh`：一条命令复跑 S1–S8 八组（见报告第 0 节），输出写进 `--work` 给的新草稿目录的 `out/`，最后与本目录 `out/*.txt`（22 份）逐行比，末行 `name=rerun_compare differing=N`。
- `tools/mutate.py`：把变异表某一行（或我加的两个取样点 `order`、`guard`）改进树里的 `mount.rs`，原文要恰好命中一次。
- `tools/core-probes.rs.txt` 与 `tools/apply-core-probes.py`：探针 P1（豁免先于判重的次序）、P3（分配位置着的槽被两条被抛弃根交叠罩着），接进 core 的 B3 用例模块末尾。
- `src/probe_k2_shared_unit_is_not_verified.rs`：K2 探针（现行版本与目标共用单元、其中一份坏），复制进 harness 的 `tests/` 跑。
- `mutations-b3-c572.tsv`：冻结副本 `crates/mutations.tsv` 里追加的六行；`mutations-replaced.tsv`：被整行替换的八行。
- `tools/anchors2.py` 与 `anchors-output.txt`：十四行变异在冻结副本 `mount.rs` 里的锚点命中次数（都是 1）与同冻结副本变异表逐字相同。
- `out/`：我最后一次复跑的输出（耗时去掉、按行排序）。

复跑（整条经内存包装，仓根、冻结副本、新草稿目录都从参数给）：

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/code-backlog-b2-code-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>
```

`tools/rerun.sh --selftest` 判的是模板自带的 `rerun_steps`，这份换成了自己的步骤，所以它的自证会红，不用它。
