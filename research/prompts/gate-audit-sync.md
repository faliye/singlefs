<!-- knowledge-sync -->
# gate-audit 阶段同步

触发文件：.claude/agent-common.md�.claude/gate.d/10-kb-rot.sh�.claude/gate.d/11-batch-scope.sh�.claude/gate.d/12-no-prime-marks.sh�.claude/gate.d/15-research-build.sh�.claude/gate.d/20-kb-shape.sh�.claude/gate.d/21-decision-items-sync.sh�.claude/gate.d/22-item-ref-status.sh�.claude/gate.d/24-status-redundancy.sh�.claude/gate.d/27-format-constants.sh�.claude/gate.d/28-cross-decision-status.sh�.claude/gate.d/29-settled-item-self-open.sh�.claude/gate.d/30-decision-history.sh�.claude/gate.d/31-blocking-verdict.sh�.claude/gate.d/32-first-txn-fields.sh�.claude/gate.d/33-mutation-tables.sh�.claude/gate.d/35-user-verdict-owed.sh�.claude/gate.d/39-field-table-sum.sh�.claude/gate.d/40-results-cited.sh�.claude/gate.d/44-settled-ref-says-open.sh�.claude/gate.d/47-research-script-selftests.sh�.claude/gate.d/48-history-month-file.sh�.claude/gate.d/49-history-brief.sh�.claude/gate.d/50-rules-manifest.sh�.claude/gate.d/51-admission-terms-covered.sh�.claude/gate.d/52-segment-registry.sh�.claude/gate.d/53-format-const-placeholders.sh�.claude/gate.d/56-crates-adversarial-review.sh�.claude/gate.d/60-stale-open-items.sh�.claude/gate.d/61-settled-same-file.sh�.claude/gate.d/62-stage-owners.sh�.claude/gate.d/63-agent-write-scope.sh�.claude/gate.d/64-change-range-single-source.sh�.claude/gate.d/66-abandoned-rounds.sh�.claude/gate.d/67-milestone-closeout-owed.sh�.claude/gate.d/68-knowledge-sync.sh�.claude/gate.d/69-evidence-in-repo.sh�.claude/gate.d/70-citations.sh�.claude/gate.d/72-agent-def-adversarial-review.sh�.claude/gate.d/73-research-gate-lint.sh�.claude/gate.d/75-decision-experiment-links.sh�.claude/gate.d/77-test-environment.sh�.claude/gate.d/80-absolute-assertions.sh�.claude/gate.d/83-closeout-row-numbers.sh�.claude/gate.d/88-quoted-result-lines.sh�.claude/gate.d/90-term-renames.sh�.claude/gate.d/92-layout-checker-sync.sh�.claude/gate.d/93-feature-bits.sh�.claude/gate.d/95-fixture-claims.sh�.claude/gate.d/96-experiment-source-discipline.sh�.claude/gate.d/97-invariant-field-anchors.sh�.claude/gate.d/98-kb-registry.sh�.claude/gate.d/99-multipath-registry.sh�.claude/gate.d/lib-format-const.py�.claude/gate.d/lib-history-brief.py�.claude/gate.d/lib-index-vs-body.py�.claude/gate.d/lib-item-ref-status.py�.claude/gate.d/lib-manifest.py�.claude/gate.d/lib-open-item-review.py�.claude/gate.d/lib-owed.py�.claude/gate.d/multipath-registry-lag.tsv�.claude/gate.d/stage-owners.tsv�.claude/hooks/kb-scribe-followup.sh�.claude/hooks/kb-scribe-followups.tsv�.claude/rules/implementation-workflow.md�.claude/rules/path-moves.md�.claude/settings.json�crates/singlefs-core/src/mount.rs�crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs�crates/singlefs-harness/src/model.rs�research/scripts/changed-paths.sh�research/scripts/fixtures/stale-candidates-benchmark-facts.tsv�research/scripts/replay.sh

这一阶段做成的事：门禁审计与修复（三轮三方 gate-fix-forks r1–r3）；共用改动范围脚本 `research/scripts/changed-paths.sh` 与门禁 64 号；门禁 12 号全仓禁用撇号角标与全仓改名；书记员写入后的核对 hook；欠账 C520–C532、C534、C535；E153、E154、E155 改名之后复跑入库。

## 搜索

基准 `3b60f09`，终点树 `0ee4a865f118451ef316a7ef21404ea4785af2a0`（工作区全量，变更史所在四份文件换成「HEAD + 这一批的改名」，另一个会话新建的 E156、E158 两页拿掉；变更记录 10 条，全是这一批的）。

```
python3 research/scripts/stale-candidates.py --check-facts research/prompts/gate-audit-facts-v2.tsv --base 3b60f09 --target 0ee4a865f118451ef316a7ef21404ea4785af2a0
  → ✓ 事实表罩全了：10 条变更记录都有出处，22 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）
python3 research/scripts/stale-candidates.py --facts research/prompts/gate-audit-facts-v2.tsv --base 3b60f09 --target 0ee4a865f118451ef316a7ef21404ea4785af2a0 --out research/prompts/gate-audit-candidates-v2.tsv
  → ✓ 候选表：25 行（24 个不同的行）
python3 research/scripts/stale-candidates.py --check-report research/prompts/gate-audit-candidates-v2.tsv research/prompts/gate-audit-judge-all.md --groups F2,F6,F7,F8,F9b,F12,F20
  → ✓ 报告判全了：25 行候选都有逐行判定
```

事实表 v1（19 行、候选 2 行）被主 agent 退回：F6–F11 用新说法当检索词、标成新立，一行候选都不出。主 agent 按旧说法在基准现状载体上复搜（`git grep -nE '10-kb-rot\.sh` 第 3 项|10 号阶段第 3 项'` → 1，命中 C114；`(31|52) 号.{0,60}(77|跳过|无对象)` → 0；`80 号.{0,60}(射程|扫|tests|src/bin)` → 0），v2 改用旧说法。逐行判：要改 5 行、不相干 9 行、事件句不改 11 行、反向补记 4 行；主 agent 逐行全看改判 4 行（M1–M4 → 不改）、补 1 行（E155 页第 24 行），写在 `research/prompts/gate-audit-judge-main.md`。漏召回与「新立」压掉候选记成 C534（事实标新立就压掉回扫）。

另：`research/prompts/gate-fix-forks-r3-verifier-output.md` 第 51、69 行两处 /tmp 依据，原样拷进 `research/prompts/gate-fix-forks-r3-verifier-rerun/` 后改了路径，只差路径，内容未改（改前 sha256 在草稿 `/tmp/claude-1000/sync-gate-audit/verifier-output-before.sha`）。

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 事实表 v1（被退回） | research/prompts/gate-audit-facts.tsv | 20 |
| 候选表 v1 | research/prompts/gate-audit-candidates.tsv | 3 |
| 事实表 v2 | research/prompts/gate-audit-facts-v2.tsv | 23 |
| 候选表 v2 | research/prompts/gate-audit-candidates-v2.tsv | 26 |
| 逐行判定报告 | research/prompts/gate-audit-judge-all.md | 79 |
| 主 agent 逐行全看 | research/prompts/gate-audit-judge-main.md | 36 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/checks-owed.md:111 | C114 10 号阶段第 3 项只扫索引页 | 改了：挪进已还清（门禁 10 号撤掉这一项，判据并进 75 号第 ⑤ 条后一半与第 ⑨ 条） |
| .claude/kb/experiments/153-账本形态与环上有洞的代价.md:53 | `replay.sh` 现指向 `e153-ledger-shape-and-ring-holes-2026-09-17-stage5.out` | 改了：现指 `-2026-09-24-rename.out`，旧产物写明已归档 |
| .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:103 | grep 旧产物、按带单引号的 H5 过滤 | 改了：新产物、`history=H8` |
| .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:115 | grep 旧产物 `-stage4.out` | 改了：换成新产物 |
| .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:127 | 一行已改指 `-stage4.out` | 改了：现指 `-2026-09-24-rename.out`，旧产物写明已归档 |
| .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:24 | 那一行已指到 stage4 | 改了：现指 `-2026-09-24-rename.out` |
| .claude/kb/checks-owed.md:475 | （新增） | 补了：C534（事实标新立就压掉回扫） |
| .claude/kb/checks-owed.md:476 | （新增） | 补了：C535（无实验段里的实验编号被当成引用） |
| .claude/kb/checks-owed.md:48 | C46 仍欠 4 个阶段的样本 | 不改：这一批已改成现值 |
| records/2026-09-05-SOP剥离轮.md:27 | 扫到 0 项不是通过，出处 C114 | 不改：说的是 2026-09-05 立规则的出处 |
| .claude/kb/checks-owed.md:462 | C520 75 号只认状态段恰好是已跑 | 不改：另一件事，这一批新立 |
| records/2026-09-19-决策瘦身与双向登记.md:161 | 门禁 75 号把依据段里任何 E 编号都当成一条引用 | 不改：那一天的发现，今天仍真，已记成 C535 |
| .claude/kb/checks-owed.md:390 | C440 40 号第三道够不着判决里的模型引用 | 不改：这一批没动那一道的射程 |
| .claude/kb/checks-owed.md:464 | C522 amend 抹掉产物时 40、88 号放过一次 | 不改：这一批新立的残留 |
| .claude/agent-common.md:4 | 上游「规则纪律（项目本地）」阶段判这一条 | 不改：这一批已改 |
| .claude/rules/implementation-workflow.md:18 | 同上 | 不改：这一批已改 |
| records/2026-09-16-subagent拆分提案.md:641 | 2026-09-18 定的三句与门禁 71、72 号 | 不改：带日期小节里记的那一次 |
| .claude/kb/experiments/153-账本形态与环上有洞的代价.md:37 | 两个新发现，数在 stage5 产物 | 不改：那次跑发现了什么 |
| .claude/kb/experiments/153-账本形态与环上有洞的代价.md:81 | 第五段原始行 | 不改：整行抄已归档产物 |
| .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:18 | 产物扩写为 stage4 | 不改：那次跑做了什么 |
| .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:50 | 第一次跑的产物与 replay.sh 那一行都不动 | 不改：说的是 2026-09-19 那一次 |
| .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:63 | stage2 随门禁 91 号归档 | 不改：2026-09-21 那一次的记录 |
| .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:66 | E155R2 那一行已改指 stage2 | 不改：说的是 2026-09-19 那一次 |
| .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:120 | S5 与第二次产物逐字段对拍 | 不改：2026-09-21 那一次 |
| .claude/kb/milestone/02-second-txn.md:289 | 现状（2026-09-17 立；同日开工） | 不改：带日期的那一天的状态 |
| records/2026-09-17-已分配口径三方与两个实验.md:40 | 最新产物 stage5，replay.sh 字节一致 | 不改：那一次派发的记录 |
| records/2026-09-17-已分配口径三方与两个实验.md:65 | 最新产物 stage4，replay.sh 字节一致 | 不改：那一次派发的记录 |
| records/2026-09-16-subagent拆分提案.md:893 | 用上的第一天两处误认 | 不改：M1 已记在这里 |

## 一句要交上游的

上游样本自检 `scripts/stage-selftest.sh` 跑样本时不清 `GATE_DIFF_BASE`：已填进上游三语仓工作区（`stage-selftest.sh` 两处 `env -u`、`selftest.sh` 一格自证），上游 zh 仓自检 398 个用例全过，等发版会话发版。`diff_base` 全推出去时退到 HEAD~1 那一处见 C528（两条门禁路判的改动窗口不同），跟 T11 一起等用户定。
