# kb 写回规格报告（第二批）

草稿目录：`/tmp/claude-1000/kb-writeback-batch2/`
规格文件：
- `/tmp/claude-1000/kb-writeback-batch2/spec.md`（sha256: 见下）
- `/tmp/claude-1000/kb-writeback-batch2/spec.json`（sha256: 见下）

```
$ sha256sum spec.md spec.json
7c82a661fa6bb634d2e31d4c86985c316e5eaea00d415e0cda2ad24cbb024d1c  spec.md
0f757eb1cfc54f1b962d74779ffbaf9d94c89c0ce8786b12365173a193fb3405  spec.json
```

## kb-spec-check.py 原样末行

对 `spec.md`：
```
  ✓ 规格 24 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 24 条）
```

对 `spec.json`（同内容，由 `parse_markdown_spec` 转出）：
```
  ✓ 规格 24 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 24 条）
```

## 规格 24 条覆盖哪几件（对应用户给的 7 项）

- 条 1–2：第 1 项，D13（验证路线） 已定项 7 射程两处（记录核对器只核挂载自己写出的发布 / 恢复后镜像取崩溃态那份）改成 B3c-2 的新交法；条 2 同时写决策变更史两条新条目（其三、其四，其四放在其三之上，因为 D28 那条更靠后写）。
- 条 3–5：第 2 项，D28（挂载期承诺量） 已定项 4 分配记录树那一项的式子从 K0 换成 K1；条 3 改形态、条 4 补依据指针、条 5 补欠账（`admission.rs` 还没改，实现随 A4d）。
- 条 6–9：第 3 项（C554）与第 6 项（A4c 两笔新欠账）合并处理：条 6 改 C554 行「前置」列成现状，条 7 在「怎么拦」列补「验收」三条用例，条 8 补 checks-owed.md 自己的「## 历史版本」2026-09-27 段（一条记 C554 修法定案、一条记新立 C570/C571），条 9 插入 C570、C571 两行新表格行。
- 条 10：第 5 项里 `TEST_IMAGE_DEFAULT_BYTES` 不登记那半句，补进 C323 行「前置」列。
- 条 11–12：第 4 项，invariants.md 第 34、297 行两处「代码暂用占位名」过时句改成现状。
- 条 13–22：第 5 项，10 个 format-const 标记落位（`NONCE_MAC_ALGORITHM_RESERVED_BYTES`、`DATA_UNIT_PAYLOAD_OFFSET`、`PACKED_UNIT_RECORDS_OFFSET`、`INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT`、`INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE`、`JOURNAL_NEW_ROOT_SEGMENT_BYTES`、`JOURNAL_NAMED_ENTRY_BYTES`、`JOURNAL_NAMED_ENTRIES_PER_RECORD`、`JOURNAL_RING_DEFAULT_BYTES`、`ROOT_RING_CHUNK_BYTES`）。
- 条 23–24：第 5 项，E142、E157 两页里被删的 `index_node_header_bytes` 共享函数改成「三方各算一份」的现状，并各补一句登记 `JOURNAL_NEW_ROOT_SEGMENT_BYTES` / `JOURNAL_NAMED_ENTRIES_PER_RECORD` 之后门禁 27 号会红在这份实验源码哪一行（已知、非新 bug）。
- 第 7 项（层 0 钉值与登记句）：spec.md 末尾只留一段占位说明，没有生成改动条——按用户交代等 E142 重出。

## 要主 agent 判的点

1. **D28 已定项 4 的『欠』落点**：分配记录树按 K1 还没写进 `admission.rs` 这句，我放进了「**欠**」段（条 5）；也可以放进「**射程**」当已知边角，两种都成立，我按「这是实现缺口不是射程边界」选了「**欠**」，请确认。
2. **JOURNAL_NEW_ROOT_SEGMENT_BYTES 落点选了 `decisions/23` 已定项 15 定案句（`= 188 字节`那一处）**，C11b 报告给了两个候选（同文件已定项 15，或 `layout/01-first-txn.md:319`），我没有同时落两处；如果主 agent 想两处都落，要再加一条。
3. **门禁 39 号「一条已定项里恰好一个 format-const 标记」在 D18（块里携带什么信息） 已定项 18 那一行会同时出现两个标记**（`INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT` 与 `INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE`，条 16、17），C11b 报告原话「那一格按 39 号的判据怎么走我没验」——这份规格照样落两个标记，39 号红不红要 kb-scribe 写完实跑才知道，不是我能在规格阶段验的。
4. **登记 `JOURNAL_NEW_ROOT_SEGMENT_BYTES`、`JOURNAL_NAMED_ENTRIES_PER_RECORD` 之后，门禁 27 号会红在 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:106` 与 `e157_parallel_line_one_clauses.rs:74`**（两处把值写成算式）——这是用户交代要写进规格的已知后果，我已经在条 23、24 里各补了一句说明；但这两处红本身没有随规格一并修（改成字面量要动 `research/`，不在我写范围，也不在这份规格改动范围内），kb-scribe 写完之后这两处会照常红，交主 agent 判要不要另派人改 `research/` 源码或先撤掉这两个标记。
5. **C554 的「验收」三条用例文字（条 7）**是我按 `/tmp/claude-1000/impl-c554-yi/spec.md`「验收」一节第 1 条的三个子项转述的，不是判决或用户定案逐字给的句子；C554 乙那边的实现员如果改了这三条用例的名字或期望，这句会跟着过时，不是这份规格能钉住的。
6. **C570、C571 的「要拦什么」「怎么拦」描述是我照 A4c 报告第三节第 4、5 条的内容组织的**，不是判决逐字给的验收句——报告本身写「推的，没实现」「推的，没量 df」，我在两行里都保留了这两句限定语，但「怎么拦」那一栏的具体造状态手法（造一次挂载抬 F 回收…／量一次 df…）是我给的建议写法，供主 agent 核对是否要改。
7. **D13/D28 决策变更史「其三」「其四」的编号**是我在起草这一刻（今天 2026-09-27，当月文件已有其一、其二）现查得出的；如果在主 agent 派 kb-scribe 之前，`.claude/kb/decisions-history/2026-09.md` 又被别的改动加了新的 2026-09-27 条目，这两个编号要 kb-scribe 按它自己的规则（照当月文件现取）核一遍再定，不一定还是三、四。

## 没做什么

- 没写 kb、没跑 kb 门禁阶段（书记员写完之后跑）。
- 第 7 项（层 0 钉值与登记句）没有生成任何改动条，只在 `spec.md` 末尾留一段说明性文字，等 E142 重出。
- 规格里的判断以判决 / 报告原文为准；「要主 agent 判的点」列出的几处是我为了把规格写完整、必须做一点组织或选择的地方，没有替主 agent 在判决之外另做别的推论。
- 没有验证登记 `JOURNAL_NEW_ROOT_SEGMENT_BYTES`、`JOURNAL_NAMED_ENTRIES_PER_RECORD` 之后门禁 27、39 号实际红不红——按定义这是 kb-scribe 写完之后跑门禁阶段的活，不归我；我只是把 C11b 报告里已经预告的红点写进了规格对应位置。
