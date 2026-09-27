# kb 写回第五批：书记员报告

规格文件：`/tmp/claude-1000/kb-batch5-drafter/spec.json`（`spec.md` 同内容，已存进 `research/prompts/m2-kb-writeback-batch5-spec.md`，7 条）。

## 开工时记的 sha256（写之前）

```
c49d96e56bfb4cdc67856bad8946ba42c68bf45b7ffe72fe548bd72b12724f9c  .claude/kb/decisions/22-单元原子性怎么合成.md
10988a041e8f6b63b0dfc3b43630a7bd27fd37450d30ae422ded6ca12290f1b2  .claude/kb/decisions/23-journal的角色与格式.md
0cb5c1e6820a254e39d6b529269fa70fbae1ac6905a436ca56bc8aa916841ce7  .claude/kb/decisions/19-块指针的结构与宽度预算.md
1ebc1ca00dcf8c658c9105e8272380ef4bbe340a27293a6557199fd89b55e719  .claude/kb/decisions/09-加密.md
2a2a6c58ac6f23883aa09e49bad8a136b1d73d02f766dd701ef0e805aac6fde2  .claude/kb/checks-owed.md
84eca733d8265ef6dacb45cecf18fcb945050cac624a2994066ffc6740f67025  .claude/kb/decisions-history/2026-09.md
0a18bc4144686ef8d696a471a0a89cb60b66a385966c5a897a536026b5b2adce  .claude/kb/decisions-history.md
0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895  .claude/kb/decisions.md
```

（`git status --porcelain` 在我开工时已经显示 `.claude/kb/checks-owed.md`、`.claude/kb/decisions-history.md`、`.claude/kb/decisions-history/2026-09.md` 与
`.claude/kb/decisions/23-journal的角色与格式.md` 带别的会话未提交的改动，上面的哈希是那份状态下的现值，不是 HEAD 的。）

## 写范围闸预检（第 1 步）

对规格点名的 6 个文件路径各喂一次写范围闸，全部 `exit=0`（放行）：
`.claude/kb/decisions/22-单元原子性怎么合成.md`、`.claude/kb/decisions/23-journal的角色与格式.md`、
`.claude/kb/decisions/19-块指针的结构与宽度预算.md`、`.claude/kb/decisions/09-加密.md`、
`.claude/kb/checks-owed.md`、`.claude/kb/decisions-history/2026-09.md`。
检出记录落在 `/tmp/claude-1000/kb-batch5-scribe/precheck.jsonl`。

## replace-batch.py 原样输出

`kb-spec-check.py` 复核（写之前）：

```
  ✓ 规格 7 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 7 条）
```

`--dry-run`：

```
✓ 7 处都命中得对（6 个文件），--dry-run 没写
```

实写：

```
✓ 7 处替换，写了 6 个文件（改名换上新 inode），回读一致
```

## 决策变更史（第 2 步）

- 取号复核（写之前现查，与起草报告一致）：`.claude/kb/checks-owed.md` 当时最大号 C580（未被占用，取 C581/C582）；
  `.claude/kb/decisions-history/2026-09.md` 2026-09-27 当天最大「其N」是「其九」（未被占用，取其十~其十三）。
- 原文四条新条目（其十 D22、其十一 D23、其十二 D19、其十三 D9）随 replace-batch.py 一次写进
  `.claude/kb/decisions-history/2026-09.md`，插在原有「### 2026-09-27（其八）」之上，与规格逐字一致。
- 先跑不带 `--write` 的 `bash .claude/gate.d/49-history-brief.sh`：
  ```
    ✗ 决策变更史的快查与原文有 1 处对不上
       .claude/kb/decisions-history.md 的生成块与按原文重新生成的不一致
  ```
  只有「生成块与重新生成不一致」这一条（意味着四条新条目自己的快查都齐、没有一条落进「快查·… 还没写」清单），
  没有属于别的批次的「还没写」条目，于是往下跑 `--write`。
- `bash .claude/gate.d/49-history-brief.sh --write`：
  ```
    ✓ 按原文重新生成了 .claude/kb/decisions-history.md：572 条条目，这次补了 0 条「（待补）」快查
  ```
- 跑前跑后各 `git diff -- .claude/kb/decisions-history.md` 一次（落盘为 `history-diff-before.txt` / `history-diff-after.txt`），
  对两份 diff 文本再取一次 diff，只看新出现的表格行（`> +|` 开头）：命中 6 行，逐行核对——
  D9（加密） 已定项 10、D19（块指针的结构与宽度预算） 已定项 3、D22（单元原子性怎么合成） 已定项 2、
  D23（journal 的角色与格式） 已定项 18/19 这 4 行是我这一批新加的（与规格四条一一对应）；
  另外 2 行（C554 乙-配置续、管理员回退「释放」一格）内容与写之前的 diff 里已经存在的两行逐字相同，
  只是因为新插入的行把它们顶到了后面 3 个名额之内的不同位置（D23 卡片「只留最近 3 次」挤掉的是较旧的
  「已定项 14、22：管理员回退改成挂着时的一次向前发布」那一行，这是设计内的挤出，不是这一轮的新增内容）——
  新增的行里没有一行是这一轮之外的条目。

## kb 门禁阶段（第 4 步）

`stage-owners.tsv` 登记给 `kb-scribe` 的阶段：
```
awk -F'\t' -v me="kb-scribe" '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv | wc -l
41
```

逐个跑（`nice -n 19 bash .claude/gate.d/<文件>`），原样末行 + 退出码：

```
10-kb-rot.sh 1
16-freeze-layer-membership.sh 0
20-kb-shape.sh 0
21-decision-items-sync.sh 0
22-item-ref-status.sh 0
24-status-redundancy.sh 0
27-format-constants.sh 1
28-cross-decision-status.sh 0
29-settled-item-self-open.sh 0
30-decision-history.sh 0
31-blocking-verdict.sh 0
32-first-txn-fields.sh 0
33-mutation-tables.sh 1
35-user-verdict-owed.sh 77
36-invariant-count-cross-file.sh 77
37-decision-summary-width.sh 0
38-field-table-projection.sh 0
39-field-table-sum.sh 0
42-first-txn-trio.sh 0
43-owed-table-shape.sh 0
44-settled-ref-says-open.sh 0
48-history-month-file.sh 0
49-history-brief.sh 0
51-admission-terms-covered.sh 0
52-segment-registry.sh 1
53-format-const-placeholders.sh 0
60-stale-open-items.sh 0
61-settled-same-file.sh 77
70-citations.sh 0
67-milestone-closeout-owed.sh 0
69-evidence-in-repo.sh 1
75-decision-experiment-links.sh 0
76-second-txn-hooks.sh 0
78-owed-cited-tests.sh 0
92-layout-checker-sync.sh 0
97-invariant-field-anchors.sh 0
98-kb-registry.sh 0
93-feature-bits.sh 0
79-tree-table-reserve.sh 0
81-audit-contradictions.sh 0
82-clause-enum-pairs.sh 0
```

77（本次未跑，不算失败）：35、36、61 三道，无关本批。

红的 5 道（10、27、33、52、69），逐一判归属——点名的文件/行都不在这一批规格里，判「不是这一轮的，不修，照写」：

| 阶段 | 点名的文件 | 是否本批 |
|---|---|---|
| 10-kb-rot.sh | `.claude/agents/tooling-writer.md`、`.claude/rules/format-evolution.md`、`.claude/rules/implementation-workflow.md`、`.claude/rules/path-moves.md`、`.claude/skills/decide/SKILL.md` | 不是；本批只写 `.claude/kb/**` |
| 27-format-constants.sh | `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`、`e157_parallel_line_one_clauses.rs` | 不是 |
| 33-mutation-tables.sh | `crates/mutations.tsv:659`（对应 `e158_root_choice_repair.rs`） | 不是 |
| 52-segment-registry.sh | `.claude/kb/layout/01-first-txn.md`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 不是 |
| 69-evidence-in-repo.sh | `research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs`、`research/prompts/m2-kb-writeback-batch{2,3,4}-scribe-report.md` | 不是 |

这五道在我开工前（第一条命令，`replace-batch.py --help` 打错触发的写入后钩子）就已经报过同样的两道红（27、52），
与 gitStatus 里一开始就列出的一批别的会话未提交改动（`.claude/agents/*.md`、`.claude/gate.d/*.sh` 等）对得上，确认是别的会话在飞的改动，不修。

## doc-lint（第 4 步）

```
✓ 文档铁律检查通过（检查 521，跳过 0；DOC_LINT_VERBOSE=1 看全部）
```
（还有两行 `!` 级别的说明性提示，不影响这一行的判定。）

## 收尾时的 sha256（写之后）

```
3532cb98c1747408e4ab69247dc88b30c77d4d6998905a61f468b71f8048c2a9  .claude/kb/decisions/22-单元原子性怎么合成.md
976f08d4d437822215b531bee5266ef5f52588c97a1bf8f2528ecae83283a6d6  .claude/kb/decisions/23-journal的角色与格式.md
0febebf6f32ad2fe9c99eea0acb05ced1ef908b088d6b15049612620dd6559e2  .claude/kb/decisions/19-块指针的结构与宽度预算.md
4a880955a551193bdfe9ba8bd1e6baa9659bd9fef314efbd0eb7f494643fb010  .claude/kb/decisions/09-加密.md
e5cb983dfff4ef764d7576e6e5f181f1e060d1f72e7a943609134c15a90d83bb  .claude/kb/checks-owed.md
a4bebfdb5cb2fb20e4120a878e14d592e75caa57572cad1ec74d5d7f037c6b4e  .claude/kb/decisions-history/2026-09.md
d48c37aa1d61886dcb89de6c59686aff3fcc0606c340c482a2a34a69f9988eb0  .claude/kb/decisions-history.md
0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895  .claude/kb/decisions.md（未改，哈希与开工时一致）
```

改过的文件共 7 个：上表除 `decisions.md` 之外的每一个。

## 没做什么

- 没跑 `relabel-item.py`：这一批 7 条规格都是「改一条分项的射程」（D22 已定项2、D23 已定项18/19、D19 已定项3、
  D9 已定项10）与欠账表增补/更新、变更史新增，没有一条翻转「已定 / 未定」状态，不适用第 3 步。
- 没有配套的实验页「### 影响的决策」表改动：这一批没有改动任何决策的「**依据**」段、没有增删任何 `E<n>` 引用
  （起草报告已核过，本轮复核一致），门禁 75 号的判据（依据段变了才要求实验页那几行同批改）没有被触发。
- 没修 5 道判红的门禁阶段（10、27、33、52、69）：点名的文件都不在这一批规格里，判定「不是这一轮的」，照实列在上表，不自己收拾。
- 没提交、没跑重型测试、没碰 `.claude/agents/`、`.claude/rules/` 等写范围外的路径。
- 没有自建编译目录或仓副本，无需清理。
