# rot-2026-11-02 阶段同步第一段：事实表

基准 `HEAD`（59ce5a865），结束 `e990b34010e73577c50f09be3bc358d00e02fcea`（批次对象）。变更清单 `changes.md` 里 0 条 H 编号，事实表的出处写提交与手工补的文件。

## 产出

- 事实表 `/home/fy5090/.local/share/singlefs/work/rot-2026-11-02-sync/facts.tsv`，29 行（F1-F29，F28、F29 是「只改措辞」，F11、F14、F17 是「新立：」）。
- 候选表 `/home/fy5090/.local/share/singlefs/work/rot-2026-11-02-sync/candidates.tsv`，611 行。
- `--check-facts` 退出码 0，末行：`✓ 事实表罩全了：0 条变更记录都有出处，29 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）`（输出 `check3.txt`）。
- `--facts` 退出码 0，末行：`✓ 候选表 …/candidates.tsv：611 行（611 个不同的行）`（输出 `facts-run.txt`）。
- 候选行按组：F1 231、F2 20、F3 47、F4 27、F5 17、F6 68、F7 24、F8 4、F9 34、F10 9、F12 56、F13 3、F15 3、F16 8、F18 14、F19 10、F20 3、F22 3、F23 4、F24 11、F25 14、F27 1，其余组（F11、F14、F17、F21、F26、F28、F29）并进别组或没有候选。
- 宽词抽查：只有 F1（`run-with-memory-cap|内存包装`，231 行）超过 100 行，抽 10 行（种子 7）全部在说调用方套内存包装起命令或这个包装自己，相干 10 行；出处列已写。其余检索词最大 90 行（F6）。
- 草稿目录里另有 `make_facts*.py`（生成事实表的脚本，最终版 `make_facts3.py`）、`changes-v1-with-e172.md`（主 agent 留的旧变更清单，我没动）。

## 事实表一览（旧 → 新，检索词）

| 编号 | 一句话 | 检索词 | 结束版命中 |
|---|---|---|---|
| F1 | 调用方套内存包装 → 整条经 task submit，内存额度由服务定 | `run-with-memory-cap\|内存包装` | 231 |
| F2 | 调用方套 capped.sh / 设 SINGLEFS_THREAD_CAP → 线程固定取登记值，服务套 | `capped\.sh\|SINGLEFS_THREAD_CAP` | 74 |
| F3 | 调用方自己清场、挑台、取回 → 机器、卡、清场由服务定（含 C922 的现状） | `gpu-quiesce-run\|multi-host-run\|--peer\|host-dispatch` | 60 |
| F4 | detached-run.sh 脱离会话起长跑 → 经 task submit | `detached-run\|--wall-clock\|within-2h\|over-2h\|long-run-check` | 23 |
| F5 | 变异命令里写内存、并发、超时、nice → 不写，不限时，卡住由服务判（kb 工具页变异节改名） | `MUTATE_MEMORY_MAX\|MUTATE_JOBS\|MUTATE_TIMEOUT\|超时地板` | 17 |
| F6 | 实验页先 cargo build 再跑 ./target/release → 同一条任务里 cargo run | `\./target/release\|target/release/e\d` | 90 |
| F7 | `--threads`、`--local-only` 调用方可写 → 只给自证，交服务的命令里不写 | `--threads\|--local-only` | 33 |
| F8 | 包装脚本为在用核数取样 → 不取样，只记进 task status | `在用核数` | 7 |
| F9 | 子 agent 自己起重活 / 非重型 → 一律不交任务（用户 2026-11-01 定） | `重型测试\|轻型三条` | 43 |
| F10 | 现行账本是 10-23、09-16 两份 → 封存，现行是 2026-10-29 那份 | `2026-09-16-subagent拆分提案\|2026-10-23-subagent拆分提案` | 9 |
| F11 | 新立：kb 工具页 task-server 一节、CLAUDE.md 一行 | `task-server\|统一任务服务`（基准 0） | 180 |
| F12 | D13 已定项 15 定案的起法 → task submit，变更史三语各加一条 | `harness 档\|第二趟` | 67 |
| F13 | 欠账 C631 开着 → 行删掉（理由是推的） | `C631\|内存排不上` | 3 |
| F14 | 新立：欠账 C922 | `C922`（基准 0） | 3 |
| F15 | 欠账表 11 个测试名带 c<编号>_ 前缀 → 换成后继名 | 九个旧测试名前缀 | 3 |
| F16 | governance-refs 只管治理文档 → 路径与小节判法扩到 kb，保留表、滞后表、两种自动跳过、认 .out.gz | `governance-refs\|搬迁这一半\|链接指向` | 8 |
| F17 | 新立：path-successor.py 与三张登记表 | `dead-path-lag\|dead-path-kept\|dead-section-lag\|path-successor`（基准 0） | 2 |
| F18 | stale-candidates 的 records 全扫、每事实一行、对照 4190 行 → 只扫登记的、合并位置、P1/S1 不出候选、对照 2410 行 | `stale-candidates` | 14 |
| F19 | verify-citations 零命中管道当失败、自证 16 格 → 退 1 按 0 计、23 格、两条新开关 | `verify-citations` | 10 |
| F20 | 里程碑三第 24 行：登记表那一行已撤（暂存区）→ 随提交撤、整份随 767668191 删 | `e165-void-red-example-ledgers` | 3 |
| F21 | 里程碑三第 227 行：agent-watch 阈值开着 → 已做 | `三次采样` | 1 |
| F22 | 里程碑四 ⑫：样本形态文件「落在真 kb 里」→「落进过……此刻已不在盘上」，出口不再要删 | `01-甲\.ja\|03-丙\.en` | 3 |
| F23 | E173 页：第二段单测 17 条 / 变异表 23 条 → 30 条 / 33 条（见下面的异常 1） | `单测 17 条\|变异表 23 条` | 4 |
| F24 | E185 第 2 次重跑命令 → task submit，补 `--disk-model` | `e185-r2-segment-run` | 14 |
| F25 | host-resources 预测与峰值表记账 → 服务按上限记账，峰值乘 1.5，没有记录先记 4G | `host-resources\|memory-peaks\|峰值表` | 32 |
| F26 | batch-target.py 的 --patch 与按块挑合用，还没在真活上用过 → 这一批就是真活 | `batch-target` | 1 |
| F27 | path-successor.py 的上游副本一步还没在真活上用过 → 真仓 governance-refs 格任务 2266 绿 | `上游副本` | 1 |
| F28 | 只改措辞：scripts/kb/README.md 清单、path-moves.md 指代（见异常 4） | `path-moves` | — |
| F29 | 只改措辞：E184、E185 等页回看格补日期 | `影响的决策` | — |

## 手工补进出处的文件（比对对象里没进的）

出处列写了文件名，没写 `records/…` 与暂存区新文件的路径（`--check-facts` 判暂存区新文件不能当出处，第一次跑就红在这里，改成「2026-10-29 那份提案账本」）：

- E172 三语页复跑块起法、background-gpu 四份测试的 ignore 理由：F1、F2、F6。
- `gpu_unit_checks.rs`：F1、F3；`gpu-quiesce-run.sh`、`e185-r2-segment-run.sh`、`scripts/hosts/README.md`：F3、F24；`e180-reading-shards.sh`、`e183-sweep-history-shards.sh`：F3、F7；`e10-aging-compaction-run.sh`、`capped.sh`：F2；`mutate.sh`：F1、F5；`stage-inputs.tsv`：F1；`code-tooling.sh`：F8、F19；`doc-experiments.sh`：F16（文件头分工）；`break-switch-replay-files.tsv`：F19；提案账本一百七十四、一百七十六节与 M01 行：F9、F10、F18、F19。

## 异常与要主 agent 知道的（批次对象本身的问题，我没改任何文件）

1. **E173 三语页在批次对象里是坏的**：`git show e990b3401:.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md` 第 25-34 行多出一段「# 第二段单测 30 条与变异表 33 条」的命令、第二段登记行、产物 2026-11-02、任务 4897 的说明，前面没有开代码围栏、后面却有一个 ```` ``` ````；下面「复跑」块里仍写「单测 17 条与变异表 23 条」，两处数对不上。这段是别的会话的 E173 第二段改动被按块挑进了批次对象（HEAD 版没有 `32248184`），不是这一批的活。en、ja 两页同样（各 +23 行）。F23 记了这件事；提交前该把这一块从批次对象里摘掉，或让那个会话带着整页一起提交。
2. **结束版里不少旧起法命中来自「HEAD 版、工作区已改」的文件**：批次对象 = HEAD + 这一批，`.claude/agent-common.md`、`.claude/agents/*.md` 等不在这一批里，所以结束版里它们还是 HEAD 的旧句（例：`agent-common.md:64`「一律用 `bash scripts/hosts/capped.sh N <命令>` 起」，工作区那一份已改）。逐行判时这类行请对工作区那一份再核一遍，别只读结束提交。
3. **detached-run.sh 前后不一**：crates 注释（批次内）写「（该选项已删）」，但批次对象里 `scripts/hosts/detached-run.sh` 还在、`--wall-clock` 还有 70 处，工作区里这份脚本已经没有（`ls` 不到）。删脚本不在这一批。F4 的新事实照此写了。
4. **要求清单第 8 条（path-moves.md 里「门禁 governance-refs 格」改成「门禁 doc-registries 的 governance-refs 格」）在 HEAD..批次对象 的 diff 里没有**：基准那一版第 10、63 行已经是「门禁 doc-registries 的 governance-refs 格」，`git grep '门禁 governance-refs'` 在基准与结束两版都是 0。F28 记成只改措辞并写明这一点。
5. `scripts/kb/README.md` 清单去掉了 `migrate-changelog-format.py`，批次对象里这份脚本还在（`git ls-tree` 有），工作区里没有；也不在这一批。
6. F13 里 C631 删掉的理由，批次对象与 `records/2026-11-01-服务描述与历史腐烂.md` 都没写，我写的「内存额度与排队改由 task-server 定」是推的，已在新事实里标「推的」。
7. 候选表只罩 `.claude/kb`、`.claude/rules`、`.claude/agents`、`.claude/skills`、`agent-common.md`、`main-agent.md`、`CLAUDE.md`、仓根 `README*.md`、不变量清单与登记过的记录，**不罩 `crates/` 的 `.rs` 与 `scripts/`、`.claude/gate.d/`**。结束版里这几类里旧起法的残余（`git grep -c`，行数/份数）：`run-with-memory-cap` crates 4/3、scripts 576/78；`capped.sh` crates 6/4、scripts 256/75；`gpu-quiesce-run` crates 5/5、scripts 127/24；`multi-host-run` crates 1/1、scripts 319/41；`detached-run` crates 2/2、scripts 96/20。scripts 里大多是这些脚本自己的文件头与自证，不都是腐烂；crates 的 `#[ignore]` 理由「经内存包装」在结束版 0 行。这两类要逐行判得另出表。
8. 事实表没有现成的一手依据可指的：F8（在用核数不取样）只据 `records/2026-11-01-服务描述与历史腐烂.md` 工具员丙那一段，我没读 `scripts/hosts/README.md` 与 `code-tooling.sh` 的 diff（不在批次对象里）。

## 交实现员

0 行。本段只写事实表，没有逐行判定表（`grep -c '^| .* | 要改 \|^| .* | 要补 ' /home/fy5090/.local/share/singlefs/work/rot-2026-11-02-sync/result.md` 数不到判定行）。

## 要主 agent 跑的命令

两条都是文本工具，定义允许子 agent 直接跑，我已经各跑了一遍，结果如上；按派发要求仍列出，主 agent 复核时照跑（在仓根起）：

```
python3 scripts/kb/stale-candidates.py --check-facts /home/fy5090/.local/share/singlefs/work/rot-2026-11-02-sync/facts.tsv --base HEAD --target e990b34010e73577c50f09be3bc358d00e02fcea
python3 scripts/kb/stale-candidates.py --facts /home/fy5090/.local/share/singlefs/work/rot-2026-11-02-sync/facts.tsv --base HEAD --target e990b34010e73577c50f09be3bc358d00e02fcea --out /home/fy5090/.local/share/singlefs/work/rot-2026-11-02-sync/candidates.tsv
```

看结局：第一条退 0 且末行「事实表罩全了」；第二条退 0 且末行「候选表 …：611 行（611 个不同的行）」。

## 做完的、没做的

- 做完：事实表 29 行、候选表 611 行；派发列的九件事与两件第一次在真活上用上的事各至少一行（九件：F1-F3、F4 与 F5（起法）、F11（task-server 描述）、F12（D13）、F16-F17（门禁）、F18（候选表）、F19（verify-citations）、F28（path-moves）、F9-F10（两条规矩））。
- 没做：逐行判（下一段，候选表分组由主 agent 分）；没改任何仓内文件。候选表分组建议：F1 的 231 行最重，可单独拆给两到三个回扫员。
- 草稿目录里没有仓副本与编译目录，没有要清理的东西（`du -sh` 1.1M，全是文本）。
