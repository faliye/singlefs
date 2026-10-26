# m4-i315-r1：I-3.15 的隔离判据认不认头读不出的那一份，故障归类器认不认插回去仍红的隔离（推论轮第一轮）正文（2026-10-21）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 -->

## 一、题面

I-3.15（未释放记录罩住的单元由映射指在这一槽）在故障注入的给定状态里挡着 `crates/mutations.tsv` 第 602 行的基线：CPU 池级 checker 就地判故障注入快档（24 段，每段 4 个注入点），归类出新发现 7 条、已知红 3 条、说谎设备 17 条，7 条签名都是 `["I-2.1","I-4.8","I-7.4","I-3.15"]`（调查员报告 `~/.local/share/singlefs/work/m4-validate-investigate-i315-swallowed/report.md`，sha256 `f528f627421f1d4496772638b9c27e31f1b3a551fa99fe5b41c61d9d617e65f6`，「结论」一节第 1 条；数是调查员在副本 harness 档上量的，主 agent 没复跑）。

调查员的定位（线索，腿要现查）：被吞的都是一个码 2 节点两份里的一份；之后一次发布 COW 换下这个节点，释放前的读盘核读到被吞那一份对不上，照 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 把两块盘上的记录都留在已分配。于是：

- **注入之后那张盘**：被吞那一份的头用不了，池级 checker 在 I-3.15 那一段先按份套「头用不了的那一格没有对象」把它跳过，再按剩下的份归组判隔离，组里只剩好的那一份，判不成隔离，I-3.15 红；同一张盘上 I-3.1、I-3.11 按单元归组，成立。
- **插回去那张盘**（被吞的写都落了盘的镜像）：两份都对得上位置项，盘上看不出隔离，记录留在已分配，I-3.15 与 I-3.11 红。故障归类器 `left_by_the_lying_device` 只比不变量名，两张盘上都有 I-3.15，于是判「插回去仍红」，报新发现。

这一轮要的是：两半各该怎么判。① I-3.15 的隔离判据认不认头读不出的那一份（调查员试的乙一、乙二，或腿给的别的读法）；② 故障归类器要不要认「插回去仍红、违例落在被吞的单元上、实现在它上面核出过对不上」这一形（调查员的甲），认的话判据写成什么、要接出哪些东西。两半是否都要，只取一半时会漏什么。要交用户的岔路写成岔路单的形态。

被测条款（材料附录整节抄）：不变量 I-3.15、I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读）三行（`crates/singlefs-checker/invariants/05-空间记账.md`）；D19（块指针的结构与宽度预算） 已定项 5（硬规则 1 与读盘核读不出、核出对不上时怎么办那一段）；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3、G9 两行；D13（验证路线） 里故障注入、说谎设备那几句。

## 二、实现今天的样子（主 agent 的观测，2026-10-21，行号按主工作区此刻的 `crates/`）

- I-3.15 的实现：`crates/singlefs-checker/src/walk.rs` 的 `judge_unreleased_allocation_records_against_the_central_mapping`。第 16378–16385 行读起点槽上的单元头，`.filter(|header| birth_txg_in_a_usable_unit_header(header).is_some())`，用不了就 `continue`（这一份的记录不进归组）；第 16465–16470 行 `for ((slot, span_slots), records_of_the_unit) in records_off_the_mapping_by_unit`，`devices_of_the_unit` 只取组里那几条记录的盘，交 `some_copy_of_the_unit_fails_its_location_entries`（第 16223 行，文档「一个单元（起点槽、跨度，两盘同槽）记着它的那几块盘上，有一份 checker 自己读出来读不出、或与指着它的位置项里的单元校验和对不上……与 I-3.1 那一份同一个判据」）。
- I-3.1 / I-3.11 那一份隔离判据按单元归组：`walk.rs` 第 8388 行一带「按单元而不按份判」那一段（调查员引，腿现查）。
- 实现一侧：`crates/singlefs-core/src/transaction.rs` 第 4592 行 `copies_of_a_released_unit_failing_the_release_checksum_check`（释放前读盘核），第 12580–12588 行把交回的盘上记录留在已分配。
- 故障归类器：`crates/singlefs-harness/src/fault_injection.rs` 第 2566 行 `left_by_the_lying_device`，第 2576 行那一判只比 `invariants_still_violated_if_the_swallowed_writes_had_landed` 里的不变量名；文档写「一条不是注入点造成的违例（插回去仍红）……照报新发现——这是判别力那一半」。同文件已有 `SwallowedWriteByteRange::overlaps`。
- 池级 checker 每条不变量只留第一处违例说明（`crates/singlefs-checker/src/image.rs` 第 494 行一带，调查员引，腿现查）。
- GPU 判器同一写法：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 第 6736–6740 行头用不了 `continue`，第 6779–6781 行组里只核标过的记录；对拍逐位比，`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 第 438 行 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS: &[usize] = &[];`（I-3.15 定义那一行状态栏写「GPU 判器下标 95：内核不置位、对拍不比」，与这张空表对不上）。
- 一条必红用例：`crates/singlefs-harness/tests/checker_known_bad_images.rs` 第 15657 行 `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device`，期望集合里有 I-3.15；那份镜像是一份改坏成 0xa5、另一份是别一版的叶，映射里有一把 key 指着这一槽、对不上头（第 15655 行一带的文档）。
- 调查员在副本里试的三种改法与量到的数在报告「判断材料（不定案）」一节：乙一（判隔离时取这一单元每块盘的全部未释放记录）让那条必红用例的 I-3.15 消失；乙二（只在这一版映射里没有 key 指着这一槽时按单元判隔离）四份点名 I-3.15 的 harness 目标与 `singlefs-checker --lib` 全绿，依据「隔离的单元之后各版没人指着」是推的；甲的粗版只看第一处违例说明。这些是线索，不是结论。
- 方案按 `crates/` 今天的实现（上面这几处）来谈。

## 三、格

### K1　读法的定义
I-3.15 的隔离判据：至少写今天（按份先跳过头用不了的、剩下的归组）、乙一、乙二三种，腿可以加别的；每种写清组里取哪几份、头用不了的那一份算「对不上」还是「没有对象」。归类器：今天（只比名）、甲（违例落在被吞的单元上且实现核出过对不上），腿可以加别的；写清判据要的输入从哪来。

### K2　合法盘与说谎设备的盘上会不会误红
I-3.15 各读法在每一形上判什么：写被吞后隔离（调查员那 7 条的形）、写被吞但没核出对不上、读盘核两次读结果不同类、隔离之后又被整理（搬迁）碰到那一槽、快照或回退候选集里还有一版指着隔离单元、两份都好但有记录没人引用（G9「已死」）。哪种读法在哪一形上误红；有没有一种合法历史让「隔离的单元之后某一版仍有 key 指着这一槽」（乙二的依据），写出那段历史。

### K3　坏盘上会不会放过
I-3.15 各读法在坏盘上判什么：那条必红用例那一形（一份改坏、映射 key 指着、对不上头）；一份头被改坏而映射已去掉 key、记录却没留在已分配之外的别的状态；两份都坏；只有一块盘有记录。哪种读法放过；放过的那一形有没有别的不变量兜住（I-1.11、I-2.1、I-3.1、I-3.11 这类），兜住算不算够。归类器甲在哪一形上会把一个真缺陷误认成说谎设备留下的（判别力那一半）。

### K4　条款文意
I-3.15 定义里「头用不了的那一格没有对象」与「隔离的单元（有一份副本读盘核出与位置项对不上）不判」两句谁先谁后、是否已经蕴含一种读法；「隔离那一格照 I-3.1 那一份隔离判据」这一句今天的实现做到没有。那条必红用例期望里的 I-3.15 是条款要的，还是写用例时按实现写的。归类器的文档「插回去仍红照报新发现」与甲是否冲突。

### K5　改法与代价
按候选列：`walk.rs` 那一段、GPU 内核那一段（对拍）、I-3.15 定义那一行（补哪一句）、那条必红用例、归类器（`fault_injection.rs`，要不要把实现的隔离清单或每一处违例落点接出来，接哪条路）、变异行与必红世界。每处给改法一句与估的行数（推的写明推的）。只取 I-3.15 那一半、只取归类器那一半、两半都取，各会留下什么。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K5 | 逐格从条款原文与上面几处代码推：每种读法的定义、在合法盘、说谎设备的盘与坏盘上各判什么、要动 `crates/` 哪几处与代价；推荐一种具体组合；要交用户的岔路写成候选、翻面观测、够判条件三列，不带倾向 |
| 云端攻方（Opus） | K1–K5 | 找反例：哪一形合法历史让乙一、乙二（或腿提的别的读法）误红，尤其「隔离之后仍有 key 指着」的历史；哪一形坏盘让各读法放过；归类器甲会把哪种真缺陷放成说谎设备。能在主工作区的拷贝上写小模型、造镜像、跑 harness 档量的就量（拷到自己的草稿目录改，不在主工作区里改），调查员的 `probes/instrumentation.patch` 与 `probes/probe_swallowed_write_and_i315.rs` 可以拿来当底 |

### 小节清单：`.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E112 Legacy Writer Encountering Unknown Trees — Run (2026-09-06, pure arithmetic, 11 unit tests / 9 mutants caught): **Among the four mountable cells, `Skip + Carry` has zero silent outcomes, with cost of 2 extra rows written per generation; ⚠️ `Refuse` two cells also have zero silent outcomes, eliminated from balance by `mount_ok` filter — its cost did not enter the silence criterion** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E112 标题之下、第一个下级标题之前的正文：第 2-9 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Cited Items (Copied Verbatim into Source Comments) | 抄 | 默认全抄（脚本生成） |
| ### Model (Locked Before Run) | 抄 | 默认全抄（脚本生成） |
| ### Criteria (Locked Before Run, Unchanged After Run) | 抄 | 默认全抄（脚本生成） |
| ### Failure Clauses (Locked Before Run) | 抄 | 默认全抄（脚本生成） |
| ### Critical Model Premise (Checked on 2026-09-06) | 抄 | 默认全抄（脚本生成） |
| ### Measurements (2026-09-06, pure arithmetic, 11 unit tests / 9 mutants caught) | 抄 | 默认全抄（脚本生成） |
| ### Conclusion (Cross-Checked Against Acceptance Clause Locked Before Run) | 抄 | 默认全抄（脚本生成） |
| ### Measurement Basis | 抄 | 默认全抄（脚本生成） |
| ### Decisions Affected | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-09-06 | 抄 | 默认全抄（脚本生成） |
| #### (Part 1) | 抄 | 随父节带到 |
| #### (Part 2) | 抄 | 随父节带到 |

### 小节清单：`.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E112 旧ライターが未知のツリーに遭遇した場合 —— 実行済み (2026-09-06、純粋算術、11単体テスト / 9変異全検出)：**マウント可能な 4 つのセルにおいて `Skip + Carry` はサイレントな結末がゼロ、代償は世代ごとに 2 行の追加書き込み；⚠️ `Refuse` の 2 セルも同様にサイレントゼロであるが、`mount_ok` フィルタによって天秤から除外されたものである —— その代償はサイレントという判定基準には入っていない** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E112 标题之下、第一个下级标题之前的正文：第 2-9 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 引用された条項 (ソースコードのコメントに逐語転記) | 抄 | 默认全抄（脚本生成） |
| ### モデル (実行前に確定) | 抄 | 默认全抄（脚本生成） |
| ### 判定基準 (実行前に確定、実行後の変更不可) | 抄 | 默认全抄（脚本生成） |
| ### 失敗条項 (実行前に確定) | 抄 | 默认全抄（脚本生成） |
| ### 重要なモデル前提 (2026-09-06 現状確認) | 抄 | 默认全抄（脚本生成） |
| ### 実測 (2026-09-06、純粋算術、11単体テスト / 9変異全検出) | 抄 | 默认全抄（脚本生成） |
| ### 結論 (実行前に確定した受容条項との照合) | 抄 | 默认全抄（脚本生成） |
| ### 計測条件 | 抄 | 默认全抄（脚本生成） |
| ### 影響を受ける意思決定 | 抄 | 默认全抄（脚本生成） |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-09-06 | 抄 | 默认全抄（脚本生成） |
| #### (その一) | 抄 | 随父节带到 |
| #### (その二) | 抄 | 随父节带到 |

### 小节清单：`crates/singlefs-checker/invariants/05-空间记账.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-3 空间记账 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-3 标题之下、第一个下级标题之前的正文：第 2-39 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`crates/singlefs-checker/invariants/04-校验和.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-2 校验和 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-2 标题之下、第一个下级标题之前的正文：第 2-12 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`crates/singlefs-checker/invariants/06-事务与崩溃一致性.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-4 事务与崩溃一致性 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-4 标题之下、第一个下级标题之前的正文：第 2-21 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`crates/singlefs-checker/invariants/02-根环.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-7 根环 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-7 标题之下、第一个下级标题之前的正文：第 2-38 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D19 (Block Pointer Structure & Width Budget) — Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D19 标题之下、第一个下级标题之前的正文：第 2-13 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Position Entries Carry Device Identity | 抄 | 随父节带到 |
| #### Settled Item 2: Ciphertext Checksum Bit-Width Takes 32 Bits | 抄 | 随父节带到 |
| #### Settled Item 3: Fixed-Width, Slots Reserved When Unencrypted | 抄 | 随父节带到 |
| #### Settled Item 4: Location Entry Width Takes 14 Bytes | 抄 | 随父节带到 |
| #### Settled Item 5: Location Authority Is Hybrid, Central Mapping Is Unique Entrance | 抄 | 随父节带到 |
| #### Settled Item 6: Central Mapping Key Reuses Write Sequence Key by Class (Post-Tightening) | 抄 | 随父节带到 |
| #### Settled Item 7: Pointer Header Carries Birth (Tree, TXG), 8 Bytes Each | 抄 | 随父节带到 |
| #### Settled Item 8: All Pointers Enter Mapping, Reference Side Carries Mapping Key Segments by Class | 抄 | 随父节带到 |
| #### Settled Item 9: How Birth Sequence Numbers Are Allocated and in What Order | 抄 | 随父节带到 |
| #### Settled Item 10: Mapping Key Strictly 27 Bytes, Entries Strictly 55 Bytes | 抄 | 随父节带到 |
| #### Settled Item 11: Pointer Offset Table, Position Entry Ordering, Where Mapping Tree Root Resides | 抄 | 随父节带到 |
| #### Settled Item 12: Code 2 / Code 3 Mapping Entries Written Day-1, Fixed-Point Writes Once per Checkpoint | 抄 | 随父节带到 |
| #### Settled Item 13: Block Pointer Split into Two Sections | 抄 | 随父节带到 |
| #### Settled Item 14: Position Entries Listed Individually, Cannot Be Computed from Single Origin via Geometric Formula | 抄 | 随父节带到 |
| #### Settled Item 15: Value Range of Extent Offset from Origin and Version 1 Value | 抄 | 随父节带到 |
| ### Chain Reactions with Other Decisions | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D19 (ブロックポインタ構造と幅予算) — 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D19 标题之下、第一个下级标题之前的正文：第 2-13 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：位置エントリにデバイス識別子を含める | 抄 | 随父节带到 |
| #### 確定項目 2：暗号文チェックサムは 32 ビットを採用 | 抄 | 随父节带到 |
| #### 確定項目 3：固定幅、非暗号化時もスロットを確保 | 抄 | 随父节带到 |
| #### 確定項目 4：位置エントリは 14 バイト | 抄 | 随父节带到 |
| #### 確定項目 5：位置の権威はハイブリッド、中央マッピングが唯一の入口 | 抄 | 随父节带到 |
| #### 確定項目 6：中央マッピング key は種別ごと書き込みシーケンスキー再利用（厳格化後）を採用 | 抄 | 随父节带到 |
| #### 確定項目 7：ポインタヘッダは誕生 (ツリー, txg) を保持、各 8 バイト | 抄 | 随父节带到 |
| #### 確定項目 8：全ポインタがマッピング経由、参照側は種別ごとにマッピング key セグメントを担持 | 抄 | 随父节带到 |
| #### 確定項目 9：誕生シーケンス番号の発行方法と順序 | 抄 | 随父节带到 |
| #### 確定項目 10：マッピング key は一律 27、エントリは一律 55 | 抄 | 随父节带到 |
| #### 確定項目 11：ポインタのオフセット表、位置エントリの順序、マッピングツリー根の配置場所 | 抄 | 随父节带到 |
| #### 確定項目 12：Code 2 / Code 3 のマッピングエントリは day-1 で書き出し、固定点はチェックポイントあたり1回書き出し | 抄 | 随父节带到 |
| #### 確定項目 13：ブロックポインタは2セグメントに分割 | 抄 | 随父节带到 |
| #### 確定項目 14：位置エントリは個別に列挙、単一起点から幾何学的計算で導出しない | 抄 | 随父节带到 |
| #### 確定項目 15：起点からの extent オフセットの値域と第1版の値 | 抄 | 随父节带到 |
| ### 他の決定との連鎖 | 抄 | 默认全抄（脚本生成） |
| ## 過去の版 | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/19-块指针的结构与宽度预算.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D19 块指针的结构与宽度预算 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D19 标题之下、第一个下级标题之前的正文：第 2-13 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：位置条目带设备身份 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：密文校验和取 32 位 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：定宽，不加密时留位 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：位置条目 14 字节 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：位置条目降为提示，中央映射是唯一入口 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：中央映射 key 取按类复用写序键（收严后） | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：指针头部带出生 (树, txg)，各 8 字节 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：全部指针进映射，引用侧按类携带映射 key 的各段 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：出生序号怎么发，按什么次序发 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：映射 key 一律 27，条目一律 55 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：指针的偏移表、位置条目的顺序、映射树的根住哪 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：码 2 / 码 3 的映射条目 day-1 写出，固定点一个 checkpoint 只写一次 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：块指针分两段 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：位置条目逐个列出，不用几何公式从单一起点推算 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：extent 距起点偏移的值域与第一版取值 | 抄 | 默认全抄（脚本生成） |
| ### 与其他决策的连锁 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/13-verification-route.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 Verification Route —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Crash Refinement First Verifies "One Publish + One Crash + One Recovery" | 抄 | 随父节带到 |
| #### Settled Item 2: Serial Commit First | 抄 | 随父节带到 |
| #### Settled Item 3: RefFS Taken to the Extreme, Handling Persistence and Crash Semantics Itself | 抄 | 随父节带到 |
| #### Settled Item 4: Crash-Point Replay Enumerates Full Write Subsets Only | 抄 | 随父节带到 |
| #### Settled Item 5: Sharing Only a Single Constants Module Generated from kb | 抄 | 随父节带到 |
| #### Settled Item 6: Three Mutually Independent Oracles | 抄 | 随父节带到 |
| #### Settled Item 7: O2 (Independent Parser + Checker) Judges a Single Image | 抄 | 随父节带到 |
| #### Settled Item 8: Two Custom-Developed Mechanisms: Merkle Root Chain Diff and Generational Incremental Scrub | 抄 | 随父节带到 |
| #### Settled Item 9: Crash-Point Replay Must Be Tiered to Avoid Wall-Clock Explosion | 抄 | 随父节带到 |
| #### Settled Item 10: Truly Required Exact Coverage N Is "Number of Commit Protocol Equivalence Classes", Not "Layout Schemes" | 抄 | 随父节带到 |
| #### Settled Item 11: When Ignored Nodes Are Enabled: Rejecting Ledger Rotating by Owed Checks, Retaining Three Rules | 抄 | 随父节带到 |
| #### Settled Item 12: Conflict 1: `Checker as Specification` vs `Verification Paths Sharing No Tools` | 抄 | 随父节带到 |
| #### Settled Item 13: Conflict 2: Refinement Spec vs Differential Model | 抄 | 随父节带到 |
| #### Settled Item 14: Interlocks with Other Decisions | 抄 | 随父节带到 |
| #### Settled Item 15: Verification Code in Two Tiers —— Harness Tier Runs Anytime, Checker Tier Defaults to Commit | 抄 | 随父节带到 |
| #### Settled Item 16: Segments Exceeding u64 Judged in Slices, State Count Uses 1280-Bit Fixed-Width Integer | 抄 | 随父节带到 |
| #### Settled Item 17: Unit of Verdict Reuse Is Function, Key Composed Solely of Function Units, Toolchain, Dependency Versions | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/13-検証ルート.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 検証ルート —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：crash refinement はまず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」を検証する | 抄 | 随父节带到 |
| #### 確定項目 2：まず直列コミットとする | 抄 | 随父节带到 |
| #### 確定項目 3：RefFS を極限まで突き詰め、自前で永続化とクラッシュセマンティクスを処理する | 抄 | 随父节带到 |
| #### 確定項目 4：クラッシュポイント再生は完全書き込みの部分集合のみを列挙する | 抄 | 随父节带到 |
| #### 確定項目 5：kb から生成された定数モジュール1つのみを共有する | 抄 | 随父节带到 |
| #### 確定項目 6：互いに独立な3つの oracle | 抄 | 随父节带到 |
| #### 確定項目 7：O2 は何を判定するか：単一イメージとレコード照合器 | 抄 | 随父节带到 |
| #### 確定項目 8：独自開発の2点：Merkle ルートチェーン差分と世代間増分 scrub | 抄 | 随父节带到 |
| #### 確定項目 9：クラッシュポイント再生は3層に分割する。さもなければ実時間が爆発する | 抄 | 随父节带到 |
| #### 確定項目 10：真に正確に網羅すべき N は「レイアウトのセット数」ではなく「コミットプロトコル構造の同値類数」である | 抄 | 随父节带到 |
| #### 確定項目 11：ignore ノードをいつ開放するか：未済の検査によるローテーション台帳を否決し、3条を残す | 抄 | 随父节带到 |
| #### 確定項目 12：衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する の衝突 | 抄 | 随父节带到 |
| #### 確定項目 13：衝突 2：crash refinement の spec とモデル突き合わせの model | 抄 | 随父节带到 |
| #### 確定項目 14：他の決定との連鎖 | 抄 | 随父节带到 |
| #### 確定項目 15：検証コードを2層に分割 —— harness 層は常時実行、checker 層は既定でコミット時のみ実行 | 抄 | 随父节带到 |
| #### 確定項目 16：セグメント単位で登録されたセグメントの全領域が u64 に収まらなければスライス判定し、状態数には 1280 ビット固定幅整数を用いる | 抄 | 随父节带到 |
| #### 確定項目 17：判定再利用のユニットは関数であり、キーは関数ユニット、ツールチェーン、依存バージョンのみで構成する | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/13-验证路线.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 验证路线 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：先串行提交 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：崩溃点重放只枚举整写子集 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：只共享一份从 kb 生成的常量 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：三个互相独立的 oracle | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：崩溃点重放要分三层，否则挂钟必炸 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：与其他决策的连锁 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/18-information-carried-in-a-block.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D18 (Information Carried in a Block) — Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D18 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Lineage Rewrite Sequence Number Not Adopted | 抄 | 随父节带到 |
| #### Settled Item 2: All Code 2 Nodes Unconditionally Carry Key Ranges | 抄 | 随父节带到 |
| #### Settled Item 3: Data Blocks Carry Plaintext Self-Describing Header | 抄 | 随父节带到 |
| #### Settled Item 4: Form Takes Metadata Multi-Copy + Dispersed Placement, Chain Arm Rejected, Single-Disk Layout Line Not Opened Yet | 抄 | 随父节带到 |
| #### Settled Item 5: Rebuild Is a Degraded Path, Graded into Three Levels by Available Witnesses | 抄 | 随父节带到 |
| #### Settled Item 6: Data Block Self-Describing Header Placed at Low Offset, Payload Starts After Header | 抄 | 随父节带到 |
| #### Settled Item 7: V1 Unit Header Field Tables Partitioned by Unit Class | 抄 | 随父节带到 |
| #### Settled Item 8: Deletion Must Write Tombstone Units | 抄 | 随父节带到 |
| #### Settled Item 9: "Block" Means Unit, One Header per Unit | 抄 | 随父节带到 |
| #### Settled Item 10: Tombstones Take Range Records + Packed Shared Units | 抄 | 随父节带到 |
| #### Settled Item 11: Value Range of Unit Type Tag Is a Registry, Packed Record Unit Is an Independent Class | 抄 | 随父节带到 |
| #### Settled Item 12: Code 2 Write Sequence Narrowed to 4 Bytes, Vacated 6 Bytes Allocated to Payload CRC and Reserved | 抄 | 随父节带到 |
| #### Settled Item 13: When Encryption Enabled, Code 1 Identity Segment and Declared Length Both Enter Ciphertext | 抄 | 随父节带到 |
| #### Settled Item 14: Nonce Code Is Full 96-Bit Nonce, Residing in Unit Plaintext Header | 抄 | 随父节带到 |
| #### Settled Item 15: Withdrawn · Number No Longer Used | 抄 | 随父节带到 |
| #### Settled Item 16: Nonce / MAC / Algorithm Type Reserved Bits Counted in Header, Declared Length Origin Is End of Header Including Reserved Bits | 抄 | 随父节带到 |
| #### Settled Item 17: 32-Byte Header Checksum Field Holds CRC32C 4 Bytes + 28 Bytes Zero; Self-Verifying Structure Checksum Covers Entire Slot Including Padding | 抄 | 随父节带到 |
| #### Settled Item 18: Code 2 Declared Length = Entry Count × Entry Width, Padding Area Always 0, Code 1 / Code 2 Each Supplemented with Offset Field Table | 抄 | 随父节带到 |
| #### Settled Item 19: Whether Field Is Worth Carrying into Block: Three Criteria | 抄 | 随父节带到 |
| #### Settled Item 20: Settled Identity Fields to Carry | 抄 | 随父节带到 |
| #### Settled Item 21: Scope of Self-Description Is Misdirected Writes, Not Lost Writes | 抄 | 随父节带到 |
| #### Settled Item 22: Information Supporting Scan Rebuild Must Be Written Explicitly, Not Folded into Checksum Seed | 抄 | 随父节带到 |
| #### Settled Item 23: Rejection List | 抄 | 随父节带到 |
| #### Settled Item 24: V1 Tombstones: Tombstone Tree, 40-Byte Records, Lineage Records | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/18-ブロックに含める情報.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D18 (ブロックに含める情報) —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D18 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：系譜書き換えシーケンス番号は不採用 | 抄 | 随父节带到 |
| #### 確定項目 2：全コード 2 ノードが無条件で key 区間を保持 | 抄 | 随父节带到 |
| #### 確定項目 3：データブロックは平文の自己記述ヘッダを保持 | 抄 | 随父节带到 |
| #### 確定項目 4：形態はメタデータ多重コピー + 分散配置を採用、チェーン案は却下、単一ディスクレイアウト方針は未開設 | 抄 | 随父节带到 |
| #### 確定項目 5：再構築は縮退パスであり、取得可能な目撃証拠により3段階に区分 | 抄 | 随父节带到 |
| #### 確定項目 6：データブロックの自己記述ヘッダを下位に配置、ペイロードはヘッダ直後から開始 | 抄 | 随父节带到 |
| #### 確定項目 7：第1版ユニットヘッダフィールド表、ユニットクラスごとに分割 | 抄 | 随父节带到 |
| #### 確定項目 8：削除時は墓標ユニットを書き込む | 抄 | 随父节带到 |
| #### 確定項目 9：「ブロック」とはユニットであり、1ユニットに1つのヘッダ | 抄 | 随父节带到 |
| #### 確定項目 10：墓標は範囲レコード + パック共有ユニットを採用 | 抄 | 随父节带到 |
| #### 確定項目 11：ユニットタイプタグの値域は登録表であり、パックレコードユニットは独立したクラス | 抄 | 随父节带到 |
| #### 確定項目 12：コード 2 の書き込みシーケンスを 4 バイトに縮小、空いた 6 バイトをペイロード CRC と予約領域に配分 | 抄 | 随父节带到 |
| #### 確定項目 13：暗号化有効時にコード 1 のアイデンティティセグメントと `宣言長` を平文と暗号文のどちらに配置するか | 抄 | 随父节带到 |
| #### 確定項目 14：nonce 代号は完全な 96 ビット nonce であり、ユニット平文ヘッダに常駐 | 抄 | 随父节带到 |
| #### 確定項目 15：撤回済み・本番号は欠番 | 抄 | 随父节带到 |
| #### 確定項目 16：nonce / MAC / アルゴリズムタイプ予約領域はヘッダに算入、宣言長の原点は予約領域を含むヘッダ末尾 | 抄 | 随父节带到 |
| #### 確定項目 17：32 バイトヘッダチェックサムフィールドは CRC32C 4 バイト + 28 バイトのゼロを格納；自己証明構造のチェックサムはパディングを含むスロット全体をカバー | 抄 | 随父节带到 |
| #### 確定項目 18：コード 2 の宣言長 = エントリ数 × エントリ幅、パディング領域は常時 0、コード 1 / コード 2 にオフセット付きフィールド表を追加 | 抄 | 随父节带到 |
| #### 確定項目 19：フィールド値をブロックに含めるべきか：3つの判定基準 | 抄 | 随父节带到 |
| #### 確定項目 20：保持が確定したアイデンティティフィールド | 抄 | 随父节带到 |
| #### 確定項目 21：自己記述の射程は誤方向書き込みであり、消失書き込みではない | 抄 | 随父节带到 |
| #### 確定項目 22：明示的フィールド vs チェックサムシードへの折り込み | 抄 | 随父节带到 |
| #### 確定項目 23：否決リスト | 抄 | 随父节带到 |
| #### 確定項目 24：第1版墓標：墓標ツリー、40 バイトレコード、系譜レコード | 抄 | 随父节带到 |
| ## 過去の版 | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/27-小データパッキングコンテナ.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D27 小データパッキングコンテナ —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D27 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1: 形態はオンラインパッキングかバックグラウンド整理か、およびコンテナ破損救済策 | 抄 | 随父节带到 |
| #### 確定項目 2: 境界線の画定位置 | 抄 | 随父节带到 |
| #### 確定項目 3: コンテナが占有するユニット種別コードと種別同一性フィールド | 抄 | 随父节带到 |
| #### 確定項目 4: コンテナインデックスの配置場所とキーエンコード | 抄 | 随父节带到 |
| #### 確定項目 5: コンテナごとの生存スロット数の配置場所 | 抄 | 随父节带到 |
| #### 確定項目 6: デフラグのトリガー基準とバッチ規模 | 抄 | 随父节带到 |
| #### 確定項目 7: スロットの nonce および MAC 保持媒体 | 抄 | 随父节带到 |
| #### 確定項目 8: データ側で採用するロード規約 | 抄 | 随父节带到 |
| #### 確定項目 9: コンテナ AAD の構成要素とドメインセパレータ | 抄 | 随父节带到 |
| #### 確定項目 10: スロットの COW 堅持原則 | 抄 | 随父节带到 |
| #### 確定項目 11: 射程: 連続小パケットの空間浪費回収に限定 | 抄 | 随父节带到 |
| #### 確定項目 12: サイズクラステーブルのフォーマット組み込み要否 | 抄 | 随父节带到 |
| #### 確定項目 13: 初期版で採用するサイズクラステーブル | 抄 | 随父节带到 |
| ## 履歴 | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`crates/singlefs-checker/invariants/01-块自描述.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-1 块自描述 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-1 标题之下、第一个下级标题之前的正文：第 2-30 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:1-9`（整段抄，未转述）**

```markdown
## E112 Legacy Writer Encountering Unknown Trees — Run (2026-09-06, pure arithmetic, 11 unit tests / 9 mutants caught): **Among the four mountable cells, `Skip + Carry` has zero silent outcomes, with cost of 2 extra rows written per generation; ⚠️ `Refuse` two cells also have zero silent outcomes, eliminated from balance by `mount_ok` filter — its cost did not enter the silence criterion**

[中文](112-旧写者遇到不认识的树.md) · **English** · [日本語](112-旧ライターが未知のツリーに遭遇した場合.ja.md)

**This answers two items in the closure inventory of D8 (core index structure) settled item 8**:
"legacy writer toward unknown accounting dimensions" and "which side remaining 15 bits of flags choose".
The two are **not independent**: when flags choose "skip this entry", v1 **writers** also skip that tree,
leaving its accounting rows unmaintained — falling right into the hole of the first item. Hence one experiment crosses both policy sets.

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:10-21`（整段抄，未转述）**

```markdown
### Cited Items (Copied Verbatim into Source Comments)

| Item | The Exact Sentence Used |
|---|---|
| D8 (core index structure) settled item 1 | Accounting entries take **idempotent full value**, not incremental Δ |
| D5 (snapshot / space accounting mechanism) settled item 2 | Generation takes checkpoint number, **retains only most recent K generations** |
| D5 (snapshot / space accounting mechanism) settled item 4 | Twelve metrics currently in use, of which **one carries tree dimension** (inode number watermark, maintained only for writable heads) |
| D15 (format freeze policy) settled item 2 | Tree table day-1 made as open list (from 2026-09-13 title removes "system configuration", tree table lives in indirection layer of root record); added keyspace for legacy readers **merely means not seeing new tree** ⇒ compat |
| D15 (format freeze policy) decision table | Three tiers: incompat disallows mount if unknown / compat_ro read-only mount if unknown / compat mounts arbitrarily if unknown |
| I-3.1 (allocated statistics match) | Allocated space statistics == sum obtained by actually traversing all references |
| `.claude/rules/fs-design.md` | Whether a format branch is worthwhile depends on whether it can turn "misused" into "unmountable" |

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:22-39`（整段抄，未转述）**

```markdown
### Model (Locked Before Run)

A pool has `T_known` trees known to v1 and `T_new` trees unknown to v1 (new types, or known types with unknown flags bits set for v1).
A **v1 writer** mounts and publishes `G` generations. Each tree has `R` accounting rows per generation, keys carry generation number, **retains only most recent K generations**.

**Two crossed policy sets**:

| Flags Policy `F` | Meaning |
|---|---|
| `Ignore` | Ignore unknown bits, continue interpreting entry per known semantics |
| `Skip` | Skip entry as "unknown tree" (same tier as unknown type) |
| `Refuse` | Refuse mount |

| Accounting Policy `C` | Meaning |
|---|---|
| `Carry` | Writer carries unknown accounting rows **as-is to new generation** (values unchanged, generation updated) |
| `NoCarry` | Do not carry |

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:40-52`（整段抄，未转述）**

```markdown
### Criteria (Locked Before Run, Unchanged After Run)

1. **Each cell reports four absolute values**, forbidding only cross-arm comparisons:
   `misread_records` (records interpreted under wrong encoding), `rows_lost` (accounting rows of unknown trees missing at generation G),
   `rows_written_per_gen` (rows written per generation), `i31_red` (whether I-3.1 (allocated statistics match) turns red, 0/1).
2. **`rows_lost` must have an independently calculated expected value**: under `NoCarry` and `G > K` it should be exactly `T_new × R`,
   given by another closed-form arithmetic rather than simulation counts; the two must match.
3. **Silence must be countable**: an outcome is "silent" if and only if `misread_records > 0 ∧ i31_red = 0`
   or `rows_lost > 0 ∧ i31_red = 0`. **Report silent outcome count per cell**.
4. **Write cost must report absolute values**: how many extra rows `Carry` writes per generation relative to `NoCarry`, reported as `T_new × R`, not percentage.
5. **`Refuse` column must report its cost**: generations where v1 writer cannot mount = `G` (all),
   and whether this is compatible with D15 (format freeze policy) settled item 2 "added keyspace ⇒ compat" (0/1).

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:53-70`（整段抄，未转述）**

```markdown
### Failure Clauses (Locked Before Run)

| Clause | Judgment on Trigger |
|---|---|
| **Positive control 1**: Under `Skip + NoCarry` and `G > K`, `rows_lost` must be > 0 | Not true ⇒ intergenerational discard dimension absent from model, **whole round invalidated** |
| **Positive control 2** (dedicated to `Ignore` arm): Under `Ignore`, `rows_lost` must == 0 **and** `misread_records` > 0 | Not true ⇒ "thinking it knows" dimension absent from model, **whole round invalidated** |
| **Positive control 3** (dedicated to `Refuse` arm): Under `Refuse`, `mount_ok` must == 0, and `rows_lost` and `misread_records` must both == 0 | Not true ⇒ refuse mount dimension absent from model, **whole round invalidated** |
| **Negative control**: Under `Carry` and `G > K`, `rows_lost` must == 0 | Not true ⇒ carry dimension absent from model, **whole round invalidated** |
| **Positive control 4**: Under `Ignore`, `misread_records` must be > 0 | Not true ⇒ misread dimension absent from model, **whole round invalidated** |
| **Reverse acceptance clause**: If silent outcome count of `Skip + Carry` is **not** minimum across whole table, record truthfully and adjust conclusion per minimum cell | — |
| **Reverse acceptance clause 2**: If extra rows written per generation by `Carry` exceeds `T_known × R` (carrying is more expensive than writer's own trees), conclusion changed to "carrying uneconomical, find another route" | Record truthfully |

**Acceptance clause written before run**: Main agent before 2026-09-06 run inclined toward `Skip + Carry`.
**If silent outcome count of `Skip + Carry` is not minimum, or its write cost exceeds reverse acceptance clause 2 line, change conclusion** —
written before run, unchanged after run.

⚠️ **These three positive controls were modified once on 2026-09-06 filing day before run**; previously only one "under `NoCarry` and `G > K` `rows_lost` must be > 0, failing under any `F` invalidates whole round". Reason for change: under `Ignore` arm writer **thinks it knows** that tree and maintains its rows as usual ⇒ `rows_lost` legitimately equals 0, original clause would falsely invalidate correct result. **This is the other side of discipline "positive controls must run for each arm under test"** — controls for each arm are **not the same one**, blind copying harms valid cases. Change happened **before any numbers emerged**, locked thereafter.

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:71-76`（整段抄，未转述）**

```markdown
### Critical Model Premise (Checked on 2026-09-06)

**Today only one metric carries tree dimension**: D5 (snapshot / space accounting mechanism) settled item 4 item 12 "inode number watermark", verbatim "**carries** (maintained only for writable heads)" — items 8 / 9 withdrawn same day.
⇒ **An unknown tree has accounting rows to lose only when it is a writable head**; if not writable head, zero rows exist, `NoCarry` harmless.
⇒ After loss, **who catches it**: I-9.11 (writable clone head has watermark row) verbatim "tree ID of each writable clone head has 'inode number watermark' row in accounting", which is **unimplemented** ⇒ must report two numbers: "caught by specification" vs "caught today".

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:77-102`（整段抄，未转述）**

```markdown
### Measurements (2026-09-06, pure arithmetic, 11 unit tests / 9 mutants caught)

**Measurement basis and rerun**: Code `scripts/experiment-apparatus/e112_old_writer_unknown_tree.rs`
(`cargo run --release --bin e112-old-writer-unknown-tree`),
artifact `e112-old-writer-unknown-tree-2026-09-06.out` (11 lines, ending `emitted=11`),
mutation table `scripts/mutation/tables/e112_old_writer_unknown_tree.tsv`, log `e112-mutate-2026-09-06.log`,
hooked into `scripts/experiment-apparatus/replay.sh` (rerun command `bash scripts/experiment-apparatus/replay.sh` with E112 (legacy writer encountering unknown trees)).
Parameters: `t_known=12 h_known=6 t_new=4 h_new=2 r_tree_dim=1 r_pool=11 k_keep=4 g_gens=10 records_per_new_tree=1000`.

| Cell | Mountable | Misread Records | Lost Rows | Rows Written / Gen | Silent Today | Silent by Spec |
|---|---|---|---|---|---|---|
| `Ignore + Carry` | Yes | **4000** | 0 | 19 | **1** | **1** |
| `Ignore + NoCarry` | Yes | **4000** | 0 | 19 | **1** | **1** |
| **`Skip + Carry`** | Yes | 0 | 0 | 19 | **0** | **0** |
| `Skip + NoCarry` | Yes | 0 | **2** | 17 | **1** | 0 |
| `Refuse + Carry` | **No** | 0 | 0 | 0 | 0 | 0 |
| `Refuse + NoCarry` | **No** | 0 | 0 | 0 | 0 | 0 |

⚠️ **Six cells produce only four distinct outcomes**: `Ignore` two cells and `Refuse` two cells are field-for-field identical — carrying policy has **no point of action** under these two flags policies (under `Ignore` writer thinks it knows, those rows already within its maintenance scope; under `Refuse` it never mounted). ⇒ **Carrying policy is a genuine choice only under `Skip`**, which is quantitative shape of "two policies not independent". Reporting six cells exposes this degeneracy itself, not six independent observations.
- **Criterion 2**: `rows_lost` matches independent closed form across all 6 × 4 sample points (cells × generations).
- **Criterion 4**: Carrying writes **2 extra rows** per generation (difference between `Skip` arms); writer's own rows **17 rows**;
  threshold of reverse acceptance clause 2 `T_known × 12 = 144` ⇒ **not triggered**.
- **Criterion 5**: Under `Refuse` v1 writer **fails to mount across all 10 generations**, and is **incompatible** with D15 (format freeze policy) settled item 2
  "added keyspace for legacy readers merely means not seeing new tree ⇒ compat" (in artifact `compat_with_d15_item2=0`).
- **Generation boundary**: Exactly K generations zero loss, K+1 generation loses 2 rows — off-by-one nailed by unit test.

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:103-120`（整段抄，未转述）**

```markdown
### Conclusion (Cross-Checked Against Acceptance Clause Locked Before Run)

Written before run: "Main agent inclines toward `Skip + Carry`; if its silent outcome count is not minimum, or write cost exceeds threshold, change conclusion."
**Neither triggered**: Silent outcome count of `Skip + Carry` is **0**, write cost 2 rows far below 144. ⚠️ **However "unique across whole table" is false, artifact itself shows**: `Refuse_Carry` / `Refuse_NoCarry` both show `silent_today=0 silent_by_spec=0`. Three of six cells have zero silence; `Skip + Carry` is merely unique **among mountable ones** — what removed `Refuse` from scale was `if o.mount_ok` filter in source code, not its performance. ⇒ **Selecting by "minimum silence" does not price "unmountable"**, `unmountable_gens=10` in Criterion 5 was reported separately, uncounted in comparison.
⇒ **Conclusion maintained**: Unknown bits in flags take "skip this entry", writer carries unknown accounting rows as-is to new generation.

⚠️ **Three findings obtained incidentally from measurements, outside original question**:

1. **`Ignore` is silent even by specification** (`silent_by_spec=1`) — invariant inventory has **no** check catching "records interpreted under wrong encoding".
   This is not `Ignore` losing elsewhere; the cell where it loses is **unwatched by anyone**. Placement recorded in C156 (record encoding misread has no invariant).
2. **`Skip + NoCarry` is silent only under 2026-09-06 implementation** (`silent_today=1` while `silent_by_spec=0`):
   What is lost is writable head's "inode number watermark" row, which I-9.11 (writable clone head has watermark row) directly targets, **yet is unimplemented**.
   ⇒ Implementing I-9.11 (writable clone head has watermark row) downgrades `NoCarry` from "silent error" to "catchable error",
   but **does not** make it correct — `Carry` still loses 0 rows vs 2 rows.
3. **Exposure surface is narrow**: Checked on 2026-09-06 only one metric carries tree dimension, maintained only for writable heads
   ⇒ An unknown tree, **if not a writable head, has zero rows**, `NoCarry` harmless to it.
   ⚠️ **This cell widens as metric inventory expands**: adding another metric carrying tree dimension causes `NoCarry` loss to scale with tree count.

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:121-126`（整段抄，未转述）**

```markdown
### Measurement Basis

Pure arithmetic / discrete simulation, zero I/O. **Running same binary N times is necessarily identical**,
so round count does not constitute evidence strength; evidence strength comes from mutation testing and assertions nailing absolute values
(deterministic exception to "single observation does not count" in `.claude/singlefs-ai-sop/rules/test-discipline.md`).

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:127-134`（整段抄，未转述）**

```markdown
### Decisions Affected

| Decision Item | Relationship | Review |
|---|---|---|
| D8 (core index structure) settled item 8 | Supports | 2026-10-11 unaffected: wording only (temporal words, location references, and self-references), conclusions, numbers, and artifact unchanged |
| D5 (snapshot / space accounting mechanism) | Does not affect | 2026-10-11 unaffected: wording only (temporal words, location references, and self-references), conclusions, numbers, and artifact unchanged |
| D15 (format freeze policy) settled item 2 | Does not affect | 2026-10-11 unaffected: wording only (temporal words, location references, and self-references), conclusions, numbers, and artifact unchanged |

```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:135-146`（整段抄，未转述）**

```markdown
## Historical Versions

### 2026-09-06

#### (Part 1)
- Filed, criteria and failure clauses locked, unrun. Basis: remaining two items in D8 (core index structure) settled item 8 closure inventory,
  after user decided "adopt" on 2026-09-06, requested verify-then-adversarial-then-compare.

#### (Part 2)
- Completed run and wrote back conclusion; all three positive controls and one negative control held, all 9 mutations caught.


```

**出处 `.claude/kb/experiments/112-legacy-writer-encountering-unknown-trees.en.md:137-146`（整段抄，未转述）**

```markdown
### 2026-09-06

#### (Part 1)
- Filed, criteria and failure clauses locked, unrun. Basis: remaining two items in D8 (core index structure) settled item 8 closure inventory,
  after user decided "adopt" on 2026-09-06, requested verify-then-adversarial-then-compare.

#### (Part 2)
- Completed run and wrote back conclusion; all three positive controls and one negative control held, all 9 mutations caught.


```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:1-9`（整段抄，未转述）**

```markdown
## E112 旧ライターが未知のツリーに遭遇した場合 —— 実行済み (2026-09-06、純粋算術、11単体テスト / 9変異全検出)：**マウント可能な 4 つのセルにおいて `Skip + Carry` はサイレントな結末がゼロ、代償は世代ごとに 2 行の追加書き込み；⚠️ `Refuse` の 2 セルも同様にサイレントゼロであるが、`mount_ok` フィルタによって天秤から除外されたものである —— その代償はサイレントという判定基準には入っていない**

[中文](112-旧写者遇到不认识的树.md) · [English](112-legacy-writer-encountering-unknown-trees.en.md) · **日本語**

**これは D8 (コアインデックス構造) 確定項目 8 の収束リストにある 2 項目に答えるものである**：
「旧ライターの未知の記帳次元に対する扱い」と「flags の残り 15 ビットでどちらを選択するか」。
この 2 項目は**独立ではない**：flags が「該当エントリをスキップ」を選択すると、v1 **ライター** もそのツリーをスキップし、
その結果その記帳行を誰も保守しなくなる —— まさに第 1 項目の穴に落ち込む。したがって 1 つの実験で 2 組のポリシーを交差させる。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:10-21`（整段抄，未转述）**

```markdown
### 引用された条項 (ソースコードのコメントに逐語転記)

| 条項 | 使用された一文 |
|---|---|
| D8 (コアインデックス構造) 確定項目 1 | 記帳エントリは**冪等な完全値**を採用し、増分 Δ とはしない |
| D5 (スナップショット / 領域記帳メカニズム) 確定項目 2 | 世代はチェックポイント番号を取得、**直近 K 世代のみを保持** |
| D5 (スナップショット / 領域記帳メカニズム) 確定項目 4 | 統計量は 12 項目を使用中、うち**1 項目がツリー次元を伴う** (inode 番号水位、書き込み可能ヘッドのみに対して保守) |
| D15 (フォーマット凍結ポリシー) 確定項目 2 | ツリーテーブルは day-1 でオープンリストとして作成 (2026-09-13 よりタイトルから「システム構成」を削除、ツリーテーブルはルートレコードの間接層に常駐)；新 keyspace の追加は旧リーダーにとって**単に新ツリーが見えないだけ** ⇒ compat |
| D15 (フォーマット凍結ポリシー) 判定表 | 3 档：incompat は未知ならマウント禁止 / compat_ro は未知なら読み取り専用マウント / compat は未知でも任意にマウント |
| I-3.1 (割り当て済み統計の一致) | 割り当て済み領域統計 == すべての参照を実際に走査して得られた和 |
| `.claude/rules/fs-design.md` | フォーマット分岐に価値があるかは、「誤用」を「マウント不可」に変換できるか否かによる |

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:22-39`（整段抄，未转述）**

```markdown
### モデル (実行前に確定)

あるプールに v1 が認識可能な `T_known` 個のツリーと、v1 が認識不能な `T_new` 個のツリーが存在する（新種、または既知の種別であるが v1 未知の flags ビットが立っている）。
1 つの **v1 ライター** がマウントして `G` 世代パブリッシュする。ツリーごとに各世代 `R` 行の記帳行があり、key に世代番号を保持し、**直近 K 世代のみを保持**。

**2 組のポリシーの交差**：

| flags ポリシー `F` | 意味 |
|---|---|
| `Ignore` | 未知ビットを無視し、既知のセマンティクスに従い該当エントリの解釈を継続 |
| `Skip` | 「該当ツリーを認識不能」としてエントリをスキップ (未知の種別と同档) |
| `Refuse` | マウントを拒否 |

| 記帳ポリシー `C` | 意味 |
|---|---|
| `Carry` | ライターは認識不能な記帳行を**そのまま新世代へと引き継ぐ** (値は不変、世代番号のみ更新) |
| `NoCarry` | 引き継がない |

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:40-52`（整段抄，未转述）**

```markdown
### 判定基準 (実行前に確定、実行後の変更不可)

1. **各セルは 4 つの絶対値を報告**し、腕間比較のみの報告は禁止：
   `misread_records`（誤ったエンコーディングで解釈されたレコード数）、`rows_lost`（第 G 世代時点で既に存在しない v1 認識不能ツリーの記帳行数）、
   `rows_written_per_gen`（世代ごとの書き込み行数）、`i31_red`（I-3.1 (割り当て済み統計の一致) が赤になるか、0/1）。
2. **`rows_lost` は独立して計算された期待値を持つこと**：`NoCarry` かつ `G > K` の時、正確に `T_new × R` であるべきであり、
   この数値はシミュレーションカウントではなく別の閉形式算術によって与えられ、両者は一致しなければならない。
3. **サイレント性をカウント可能とすること**：ある結末が「サイレント」であるのは `misread_records > 0 ∧ i31_red = 0`
   または `rows_lost > 0 ∧ i31_red = 0` の時、かつその時に限る。**セルごとにサイレントな結末数を報告**。
4. **書き込みコストは絶対値を報告**：`Carry` が `NoCarry` に対していかに多くの行を世代ごとに追加書き込みするかを `T_new × R` で報告、パーセンテージでの報告は不可。
5. **`Refuse` の列はその代償を報告**：v1 ライターがマウントできない世代数 = `G`（全世代）、
   およびこれが D15 (フォーマット凍結ポリシー) 確定項目 2「新 keyspace 追加 ⇒ compat」と整合するか (0/1)。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:53-70`（整段抄，未转述）**

```markdown
### 失敗条項 (実行前に確定)

| 条項 | トリガー時の判定 |
|---|---|
| **陽性対照甲**：`Skip + NoCarry` かつ `G > K` の時、`rows_lost` は必ず > 0 | 不成立 ⇒ 世代間破棄の次元がモデルに入っていない、**ラウンド全体を無効化** |
| **陽性対照乙** (`Ignore` 腕専用)：`Ignore` 下で `rows_lost` は必ず == 0 **かつ** `misread_records` > 0 | 不成立 ⇒「自身で認識していると思い込む」次元がモデルに入っていない、**ラウンド全体を無効化** |
| **陽性対照丙** (`Refuse` 腕専用)：`Refuse` 下で `mount_ok` は必ず == 0、かつ `rows_lost` と `misread_records` はともに == 0 | 不成立 ⇒ マウント拒否の次元がモデルに入っていない、**ラウンド全体を無効化** |
| **陰性対照**：`Carry` かつ `G > K` の時、`rows_lost` は必ず == 0 | 不成立 ⇒ 引き継ぎの次元がモデルに入っていない、**ラウンド全体を無効化** |
| **陽性対照二**：`Ignore` 下で `misread_records` は必ず > 0 | 不成立 ⇒ 誤読の次元がモデルに入っていない、**ラウンド全体を無効化** |
| **逆向き受容条項**：もし `Skip + Carry` のサイレント結末数が表全体で最小**でない**場合、実態通りに記述し、最小のセルに従い結論を変更 | —— |
| **逆向き受容条項二**：もし `Carry` の世代あたり追加書き込み行数が `T_known × R` を超過する場合（引き継ぎがライター自身のツリー記帳より高価）、結論を「引き継ぎは割に合わず、別の活路を模索すべき」に変更 | 実態通りに記述 |

**実行前に記載した受容条項**：メイン agent は 2026-09-06 実行前に `Skip + Carry` を支持。
**もし `Skip + Carry` のサイレント結末数が最小でない場合、またはその書き込みコストが逆向き受容条項二の基準を超える場合、結論を変更する** ——
この一文は実行前に記述し、実行後の変更は不可。

⚠️ **これら 3 つの陽性対照は 2026-09-06 ファイル作成当日、実行前に 1 回変更された**。変更前は「`NoCarry` かつ `G > K` の時 `rows_lost` は必ず > 0、いずれかの `F` で不成立ならラウンド全体を無効化」の 1 条のみであった。変更の理由：`Ignore` 腕下ではライターがそのツリーを**認識していると思い込み**、同様に行を保守する ⇒ `rows_lost` は正当に 0 に等しくなり、元の条項では正しい結果をラウンド全体の無効化と誤判定してしまう。**これは「陽性対照は被測定の各腕に対して実行されなければならない」という規律の別側面である** —— 各腕が実行すべき対照は**同一のものではない**ため、盲目的な単一のコピーは正当なケースを巻き添えにする。変更はいかなる数値が現れるより**前**に行われ、変更後の修正は禁止。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:71-76`（整段抄，未转述）**

```markdown
### 重要なモデル前提 (2026-09-06 現状確認)

**本日ツリー次元を伴う統計量は 1 つのみ**：D5 (スナップショット / 領域記帳メカニズム) 確定項目 4 第 12 項「inode 番号水位」、逐語「**伴う** (書き込み可能ヘッドのみに対して保守)」—— 第 8 / 9 項は同日撤回済み。
⇒ **v1 未知のツリーは、それが書き込み可能ヘッドである場合のみ失う記帳行が存在する**；書き込み可能ヘッドでなければ 1 行も存在せず、`NoCarry` は無害。
⇒ 喪失後に**誰が検出するか**：I-9.11 (書き込み可能クローンヘッドは水位行を保持) 逐語「各書き込み可能クローンヘッドのツリー ID は記帳内に『inode 番号水位』行を持つ」、そしてこれは**未実装** ⇒「仕様上検出されるか」と「本日検出されるか」の 2 つの数値を分けて報告する必要がある。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:77-102`（整段抄，未转述）**

```markdown
### 実測 (2026-09-06、純粋算術、11単体テスト / 9変異全検出)

**計測条件と再実行**：コード `scripts/experiment-apparatus/e112_old_writer_unknown_tree.rs`
(`cargo run --release --bin e112-old-writer-unknown-tree`)、
成果物 `e112-old-writer-unknown-tree-2026-09-06.out` (11 行、末尾 `emitted=11`)、
ミューテーション表 `scripts/mutation/tables/e112_old_writer_unknown_tree.tsv`、ログ `e112-mutate-2026-09-06.log`、
`scripts/experiment-apparatus/replay.sh` に接続済み (再実行コマンド `bash scripts/experiment-apparatus/replay.sh` に E112 (旧ライターが未知のツリーに遭遇した場合) の番号を付与)。
パラメータ：`t_known=12 h_known=6 t_new=4 h_new=2 r_tree_dim=1 r_pool=11 k_keep=4 g_gens=10 records_per_new_tree=1000`。

| セル | マウント可 | 誤読レコード | 喪失行 | 世代あたり書込行 | 本日サイレント | 仕様上サイレント |
|---|---|---|---|---|---|---|
| `Ignore + Carry` | 是 | **4000** | 0 | 19 | **1** | **1** |
| `Ignore + NoCarry` | 是 | **4000** | 0 | 19 | **1** | **1** |
| **`Skip + Carry`** | 是 | 0 | 0 | 19 | **0** | **0** |
| `Skip + NoCarry` | 是 | 0 | **2** | 17 | **1** | 0 |
| `Refuse + Carry` | **否** | 0 | 0 | 0 | 0 | 0 |
| `Refuse + NoCarry` | **否** | 0 | 0 | 0 | 0 | 0 |

⚠️ **6 つのセルには 4 つの異なる結末しか存在しない**：`Ignore` の 2 セルと `Refuse` の 2 セルはそれぞれフィールドごとに完全一致 —— 引き継ぎポリシーはこれら 2 つの flags ポリシー下で**作用点を持たない** (`Ignore` 下ではライターが自身で認識していると思い込み、それらの行は元々その保守範囲内にある；`Refuse` 下ではそもそもマウントされていない)。⇒ **引き継ぎポリシーは `Skip` 下においてのみ真の選択肢となる**。そしてこれこそが「2 つのポリシーは独立ではない」という文の定量的形態である。6 セルを報告するのはこの縮退自体を可視化するためであり、6 つの独立観測ではない。
- **判定基準 2**：`rows_lost` は全 6 × 4 個 (セル × 世代数) のサンプリング点において独立した閉形式とセルごとに一致。
- **判定基準 4**：引き継ぎは世代あたり **2 行** 余分に書き込み (`Skip` 2 腕の差)、ライター自身の記帳は **17 行**、
  逆向き受容条項二の閾値 `T_known × 12 = 144` ⇒ **トリガーされず**。
- **判定基準 5**：`Refuse` 下で v1 ライターは **10 世代すべてマウント不可**、かつ D15 (フォーマット凍結ポリシー) 確定項目 2
  「新 keyspace 追加は旧リーダーにとって単に新ツリーが見えないだけ ⇒ compat」と**非互換** (成果物内 `compat_with_d15_item2=0`)。
- **世代境界**：ちょうど K 世代は喪失せず、K+1 世代で 2 行喪失 —— off-by-one は単体テストで固定。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:103-120`（整段抄，未转述）**

```markdown
### 結論 (実行前に確定した受容条項との照合)

実行前の記載：「メイン agent は `Skip + Carry` を支持；もしそのサイレント結末数が最小でない場合、または書き込みコストが閾値を超える場合は結論を変更する」。
**いずれもトリガーされず**：`Skip + Carry` のサイレント結末数は **0** であり、書き込みコスト 2 行は 144 を大幅に下回る。⚠️ **しかし「全表で唯一」という文は誤りであり、成果物自身が示している**：`Refuse_Carry` / `Refuse_NoCarry` の 2 行も同様に `silent_today=0 silent_by_spec=0` である。6 セル中 3 セルがサイレントゼロであり、`Skip + Carry` は単に**マウント可能なものの中で**唯一に過ぎない —— `Refuse` を天秤から除外したのはソースコード内の `if o.mount_ok` フィルタであり、その性能ではない。⇒ **「サイレント最小で選択」という選択方法自体は「マウント不可」を価格付けしない**。判定基準 5 の `unmountable_gens=10` は単独で報告されたものであり、その比較には算入されていない。
⇒ **結論維持**：flags 未知ビットは「該当エントリをスキップ」を選択、ライターは認識不能な記帳行をそのまま新世代へと引き継ぐ。

⚠️ **実測により付随的に得られた、元の問いには含まれていなかった 3 つの結果**：

1. **`Ignore` は仕様上もサイレントである** (`silent_by_spec=1`) —— 不変条件リスト内に「レコードが誤ったエンコーディングで解釈された」を検出する条項が**存在しない**。
   これは `Ignore` が他所で負けたのではなく、それが負けたセルを**誰も監視していない**ためである。配置先 C156 (レコードエンコーディング誤読に不変条件がない)。
2. **`Skip + NoCarry` は 2026-09-06 の実装下でのみサイレントである** (`silent_today=1` かつ `silent_by_spec=0`)：
   失われるのは書き込み可能ヘッドの「inode 番号水位」行であり、I-9.11 (書き込み可能クローンヘッドは水位行を保持) が真正面からこれに対峙しているが、**未実装**。
   ⇒ I-9.11 (書き込み可能クローンヘッドは水位行を保持) の実装は `NoCarry` を「サイレントエラー」から「検出可能なエラー」へと格下げするが、
   それを正しくするわけでは**ない** —— `Carry` は依然として 0 行喪失対 2 行喪失である。
3. **暴露面は極めて狭い**：2026-09-06 現状確認時点でツリー次元を伴う統計量は 1 つのみであり、書き込み可能ヘッドに対してのみ保守
   ⇒ v1 未知のツリーは、**書き込み可能ヘッドでなければ 1 行も存在せず**、`NoCarry` はそれに対して無害。
   ⚠️ **このセルは統計量リストの拡張に伴い拡大する**：ツリー次元を伴う統計量がさらに 1 つ追加されれば、`NoCarry` の喪失量はツリー数に伴い増加する。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:121-126`（整段抄，未转述）**

```markdown
### 計測条件

純粋算術 / 離散シミュレーション、I/O なし。**同一バイナリを N 回実行すれば必然的に完全に同一**となるため、
ラウンド数は根拠の強度を構成せず、根拠の強度は変異テストと絶対値を固定するアサーションに由来する
(`.claude/singlefs-ai-sop/rules/test-discipline.md`「単一観測はカウントされない」の決定論的例外)。

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:127-134`（整段抄，未转述）**

```markdown
### 影響を受ける意思決定

| 意思決定項目 | 関係 | 振り返り |
|---|---|---|
| D8 (コアインデックス構造) 確定項目 8 | 準拠 | 2026-10-11 影響なし：表現のみ修正 (時間表現、位置の指示、一人称)、結論、数値、成果物は不変 |
| D5 (スナップショット / 領域記帳メカニズム) | 影響なし | 2026-10-11 影響なし：表現のみ修正 (時間表現、位置の指示、一人称)、結論、数値、成果物は不変 |
| D15 (フォーマット凍結ポリシー) 確定項目 2 | 影響なし | 2026-10-11 影響なし：表現のみ修正 (時間表現、位置の指示、一人称)、結論、数値、成果物は不変 |

```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:135-146`（整段抄，未转述）**

```markdown
## 履歴バージョン

### 2026-09-06

#### (その一)
- ファイル作成、判定基準および失敗条項を確定、未実行。根拠：D8 (コアインデックス構造) 確定項目 8 収束リストの残り 2 項目、
  2026-09-06 ユーザーが「採用」決定後、検証・対抗・比較を順次実施することを要求。

#### (その二)
- 実行完了し結論を書き戻し；3 つの陽性対照と 1 つの陰性対照がすべて成立、9 件の変異がすべて検出。


```

**出处 `.claude/kb/experiments/112-旧ライターが未知のツリーに遭遇した場合.ja.md:137-146`（整段抄，未转述）**

```markdown
### 2026-09-06

#### (その一)
- ファイル作成、判定基準および失敗条項を確定、未実行。根拠：D8 (コアインデックス構造) 確定項目 8 収束リストの残り 2 項目、
  2026-09-06 ユーザーが「採用」決定後、検証・対抗・比較を順次実施することを要求。

#### (その二)
- 実行完了し結論を書き戻し；3 つの陽性対照と 1 つの陰性対照がすべて成立、9 件の変異がすべて検出。


```

**出处 `crates/singlefs-checker/invariants/05-空间记账.md:1-39`（整段抄，未转述）**

```markdown
## I-3 空间记账 —— 类

**依赖 D5（快照 / 空间记账机制）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） ⚠️ **引用按映射位置记**（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 与第七节用户定案「重构」，D19（块指针的结构与宽度预算） 已定项 5 / 8）：豁免三类（映射树自己的节点、树表单元、实例表单元）之外的码 1 / 2 / 3，走读按那一版的中央映射认位置，指针里的位置条目只当提示——引用集合记映射位置，提示过期（单元被搬走、源槽已释放）不进引用；提示与映射条目在同一序号上带的整单元校验和不同的那条指针不当过期的提示，照它自己的位置条目记（`walk::CentralMappingLocationsOfAVersion::locate`，实现员 2026-10-15 推的读法，交主 agent 认）；合法搬迁过的镜像在 `crates/singlefs-harness/tests/checker_relocated_images_judged_by_the_mapping_position.rs`。⚠️ 「源记录还在册、目的槽没有记录」那一形（映射换位、分配记录一条不动）按槽数加总对得上，这一条看不见，由 G3 的新判 N2 判红：I-3.15（未释放记录罩住的单元由映射指在这一槽），2026-10-15 实现。 ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）。遍历的是最新根与不被抛弃、不低于 F 的有效根（RGall：候选集不再滤一圈窗口，环上有洞时洞那一槽里留着的上一圈的根只要不被抛弃、不低于 F 也走；代码 `crates/singlefs-checker/src/walk.rs` 的 `check_pool_image_given_the_container_capacity_knobs` 里 `candidate_indexes` 那一段）。F_生效 由 checker 自己算：取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与崩溃点重放不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于回收门槛的（还在 defer 里，分配器还不许回收；回收门槛 = max(F_生效, 环里最旧有效根)，D16（发布语义） 已定项 1「可再分配」那一行），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15，走那条记录新根段里的实例表指针与分配记录树根（`walk::walk_version_applied_only_by_records`；用户 2026-10-02 定新根段回到三字段，实例表指针与分配记录树根随新根段换成那次发布的）。⚠️ **没量的前提**：「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`m2-rev-b2-implementer-report.md` 第七节第 2 条）；量它是 C699（取号跳过改认出生代号） 的第一步。 ⚠️ **钉最旧 txg 的代价（用户认的）**：读坏槽与读回全零的槽把回收门槛钉在它可能住过的最旧 txg（D16（发布语义） 已定项 1「根槽这一次读坏」），这样的合法盘面上 I-3.1（已分配统计对得上） 红在记账多的一边，到钉解开为止（这个进程写掉那一槽，或进程内记根时重读读出有效根、读出被抛弃的根，`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant` 文档注释）；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 三条慢档用例在第 10h 批改成只许红在记账多的一边（用户 2026-10-04 定钉最旧 txg、2026-10-05 定全零槽钉门槛而不并进上限，`records/2026-09-30-里程碑三开工第一轮.md` 第 10b、10c 批）。 ⚠️ **钉落盘之后的并法（C773（读坏与全零根槽的回收门槛钉只在内存里） 定案，用户 2026-10-20 认；checker 未实现，随 C773 实现批，GPU 判器那一侧没有现成件）**：根记录带回收门槛 t 与钉槽 p 之后（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）），这一格只把「钉带出来的那一截」不看引用并进遍历得到的和，被抛弃根引用着的槽不并（C684（双故障形下已分配统计该不该并被抛弃根的槽） 保持「不并」，那一形照旧红在记账多的一边）；最新根下面没有记账树时照旧报不适用（`walk.rs` 今天已是）。「钉带出来的那一截」两腿的算例都取释放代落在 (t, 不带钉现算的门槛] 的已释放槽（推的）；checker 要走被抛弃根才分得出「被抛弃根引用着」，底子是 `walk::judge_blocks_referenced_by_abandoned_roots`。出处：根环三方第二轮判决 `research/prompts/m3-close-rootring-r2-main-verification.md` 第二节 Y4 那一行，第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C773 那一行之后的格式件补漏段。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](../../../.claude/kb/decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中；③ 的逆向（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）：deadlist 中每一条 b 都不被活头引用，且 `Reach(b)` 非空（某个活快照引用它）或 b 在未完成销毁意图的待放集里（意图 (头, lo = S.txg, hi, P = prev(S).txg) 下纪元落在 [lo, hi) 里、birth > P 的条目，D5（快照 / 空间记账机制） 已定项 12）。③ ④ 与逆向里的「快照」只算活快照：带销毁中记号（码 14 条目 flags 位 0，同一判决 S4）的快照从写意图那一刻起不再钉住块（D5（快照 / 空间记账机制） 已定项 20 的例外句），不进 `Reach`；「归属区间」按纪元认：快照 X 那一份 = 纪元落在 [prev(X).txg, X.txg)、活头那一份 = [prev(头).txg, ∞)，b 该在的那一份是引用它的最晚那个活快照之后的那一侧（纪元落在 [那个快照的 S.txg, 链上下一个活快照的 S.txg 或 ∞)）。b 按单元认、写成中央映射 key（活快照与活头两棵用户可见树下面每个单元的 key 从指针现算，deadlist 条目里就是 key）。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格（今天的代码走不到，随失败处置在后面的里程碑做）、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与崩溃点重放其余判定都不红）上红。 | 部分实现：出现规则（TF、β、gap、gap-row、n3）已实现（落点折成单元先按中央映射 key 折：搬迁前后两个落点是同一个单元，不算后一版新出现的，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3；`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`），池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`（`IMPLEMENTED_INVARIANTS` 下标 65；坏镜像在 `crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs`；变异在 `crates/mutations.tsv` 里「第8批 I-3.5（引用区间的精确性）」那几行）；③ ④（deadlist 那两条）连同逆向已实现（2026-10-15，里程碑四只读快照 checker 侧，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）：并在下标 65 这一条下，池级 checker 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 在回退候选集每一版（加由记录施加出来的那几版，树表单元相同的只判一次）上判：④ 每一条钉的单元活头不引用，逆向每一条钉的单元有活快照引用或在待放集里，③ 活快照引用而活头不引用的每个单元恰好一条、头是活头、纪元落在它该在的那一侧；一版里一个快照都没有、deadlist 一条都没有时这三格没有对象、不判；树表、活头两棵树、deadlist 树或活快照的两棵树读不全时报射程里判不了。纯判法的单测与必红世界在 `crates/singlefs-checker/src/walk.rs` 的测试模块（④、逆向、③ 缺一条与挂错一侧）与 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`（写路径建两个快照之后往 deadlist 里放一个活头还引用的块、放一个谁都不钉的块）；变异在 `crates/mutations.tsv` 里「只读快照 checker 侧」那几行。GPU 判器内核那一位判出现规则，也判 ③ ④ 与逆向：扫描内核的引用与快照那一截（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_snapshots_and_deadlists_of_the_versions`）照 `singlefs_checker::walk` 的同名函数判、置位，对拍逐位比，下标 65 不在对拍不比的表里（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表）。出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 n3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 n3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向）。S 是活快照或活头，一条条目归哪一侧按纪元认（快照 X 那一份 = [prev(X).txg, X.txg)、活头那一份 = [prev(头).txg, ∞)，D5（快照 / 空间记账机制） 已定项 12）；`birth` 取条目里映射 key 的出生 txg 段。**豁免**（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ②，攻方 H4-2）：落在未完成销毁意图 [lo, hi) 里、birth > P 的条目（写意图之后、它被放掉之前，前驱已传给下一侧）不判；「续做点之前不许再有这样的条目」那一半没有输入（续做点不存、条目被放掉就是进度，判决第七节岔路 2 的用户定案），不判。 | 已实现（2026-10-15，`crates/singlefs-checker` 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 里每一版判；`IMPLEMENTED_INVARIANTS` 下标 93；写路径写出的 deadlist 上的必红世界「生在快照之后、该直接释放的单元挂进 deadlist」与 I-3.5 ③「只被快照引用的单元在 deadlist 里少一条」在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`）：一版里 deadlist 一条条目都没有时判一次、成立（同 I-9.24 空队列报成立）；条目的头不是这一版的活头、纪元落不进恰好一侧（前驱链断了，归 I-9.25）的那几条报射程里判不了；树表、deadlist 树读不全报射程里判不了。必红世界与豁免去掉要红的合法态在 `crates/singlefs-checker/src/walk.rs` 的测试模块（birth 晚于活头前驱的块进了 deadlist；写销毁意图与最后一批之间的待放条目）；变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行。盘上的必红世界要等 deadlist 写者落地（里程碑四丙 2）之后补。GPU 判器内核不置位、对拍不比。 |
| I-3.4 | 可用空间扣待删占用 | 待删中 inode 没释放的单元计在记账「已分配」里（释放只进 defer），记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用；准入式子另扣环窗口常驻项（D28（挂载期承诺量） 已定项 1）。来源 `research/prompts/m3-delete-r3-main-verification.md` 第三节判标签 4 那一格，被攻过零轮 | 已实现（池级 checker `walk::check_pool_image`，函数 `judge_pending_deletion_occupancy_of_a_version`；坏镜像两份：标签 4 那一行的值非 0、整行缺失，登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`；GPU 判器下标 70，扫描内核判）。池级 checker 只判「标签 4 恒 0」：判的每一版里，走全了的记账树都要有标签 4 那一行且值为 0，缺行判红；另两句不在这一行判：计在「已分配」里归 I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读），可用空间已扣待删占用归 I-5.2（空闲统计对得上），即空闲加已分配等于容量；判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法；出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节） |
| I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少）。这个结构是树表里不带销毁中记号的码 14 条目集合（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1；带销毁中记号的照同一判决 S4 算销毁中、不算活快照，最后一批同一次发布删掉它的码 14、15）。可判定形式：① 每一版里每一个活快照都是这一版之前的一次捕获：诞生 txg（S.txg）不大于这一版的 txg，两条根指针的出生 txg 不大于 S.txg；② 回退候选集里 txg 相接的两版之间，活快照只经建快照那次发布多出来（诞生 txg 就是后一版的 txg）、只经写销毁意图（带上记号）与最后一批（带着记号的那一条没了）少下去，同一个快照的诞生 txg 不变；txg 不相接的两版之间不判。第二来源前驱链（码 14 的前驱 = 排在它前面那条的诞生 txg，活头 inode 条目的前驱 = 最后一条的）由 I-9.25 判，这一条不重判。 | 已实现（2026-10-15，`crates/singlefs-checker` 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 每一版判 ①、走完判 ②；`IMPLEMENTED_INVARIANTS` 下标 94；② 依赖写意图与最后一批各是一次独立发布：core 的 `transaction::publish_snapshot_destruction_intent` 只写记号、`publish_snapshot_destruction_step` 能放的条目不多于一批时（零条也是）才是最后一批、同一次删那一对，两者从不并成一次——2026-10-15 核过）：一个快照都没有的一版 ① 判一次、成立；树表读不全报射程里判不了。必红世界在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`（写路径建三个快照之后最新那一版的树表去掉中间那一对：少一条；加一对诞生 txg 是旧 txg 的：多一条）与 `crates/singlefs-checker/src/walk.rs` 的测试模块；变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行。GPU 判器内核不置位、对拍不比。 |
| I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例（记录形态归 I-3.14（实例表每片记录形态合法））；最新根的实例表第 0 片解不开时报不适用，理由点名这一片解不开、行没收全；回收那一半没有输入：删行由整轮清扫做（`crates/singlefs-core/src/instance_table_sweep.rs`，可写挂载末尾一支与后台一支），checker 不判删行的条件） |
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条崩溃点重放流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；一条根引用的落点按它自己的中央映射认（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3：搬走之后源槽在搬迁那一版起不再被引用，释放代落在搬迁那一格；`walk::RootReferences::note_located_by_the_central_mapping`）；从 deadlist 放掉的块的必红世界（释放代改成写意图那次的 txg）在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；崩溃点重放每个崩溃状态都判）。**判法收窄在实现途中**：今天任一候选根走不全就整条报不适用；收窄成只在比最后引用它的那条根晚的候选根都走得全时才判这一条已释放记录，攻方造出的对照格镜像（抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏）钉进用例（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行）。**从 deadlist 放掉的块**（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ③，攻方 H5-3）：释放代落在（写意图 txg, 最后一批 txg]。落法（2026-10-15）：「一条根引用的落点」算进它活快照的两棵树（码 14 照 inode 树、码 15 照 extent 树往下数）与它中央映射树叶里每条映射条目指的单元（被钉单元的映射条目留在映射里，放掉它的那一批同一次发布删条目，同一判决 S3、S4），带销毁中记号的快照的两棵树不算（D5（快照 / 空间记账机制） 已定项 20 的例外句）；于是一个从 deadlist 放掉的块，最后还引用它的是放它那一批之前的那条根、最早不再引用它的是那一批的根，取到的区间落在（写意图 txg, 最后一批 txg] 里，比判决那一句更紧。照数销毁中快照的树，那几个块在最后一批之前的每条根上都还算引用，合法态红；不数映射条目，写意图那条根之后就不再算引用、释放代晚于区间，合法态红。变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行（单测钉「销毁中快照的两棵树不记引用」）；盘上的必红世界（从 deadlist 放掉的块释放代写成写意图那一次的 txg）要等 deadlist 写者与删快照落地（里程碑四丙 2）之后补。 |
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：下一次挂载的恢复照 D23（journal 的角色与格式） 已定项 14 的重放规则施加的**第一次**发布（与 I-4.1（重放后树可遍历） 同一份重放，所选根取最新根），取它末条那一份新根段里树表指着的分配记录树；一次都不施加、锚点认不定、所选根的 txg 没有后继时没有这一版，前缀停在后面某次发布的点名单元上时仍取第一次（`walk::version_the_next_mount_applies_first` 与 `walk::replay_the_next_mount_would_do`；认法依据 `research/prompts/m3-formal-code-b36-r1-main-verification.md` 第二节 P1 与第三节第 1 条，取末条那一份依据 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1；主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定接这一步，被攻过零轮；只接这一步——恢复连着施加几次发布时第二次及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
| I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |
| I-3.12 | 实例表行 T 低于引用它的根的 txg | 根环里任一自证过的根 R，它指着的实例表的每一行 (i, T, W) 满足 T < R 的 `checkpoint_txg`；链上某一片读不出的根不判。依据 D18（块里携带什么信息） 已定项 11（恢复写的行，所选根那个实例的 T 是重放之后那个根的 txg，中间实例写 0）与 D23（journal 的角色与格式） 已定项 14（新实例第一次发布的 `checkpoint_txg` 大于环里全部根与记录的号）。射程是每条根各自指着的那张表，不是只有挂载根那一张（那一张归 I-3.8（实例表行唯一且低于挂载根））；同一实例的行在两张表里是否逐字节相同不判，条款没有这一句。 | 已实现（池级 checker `walk::judge_instance_table_rows_against_the_roots_referencing_them`：每条自证过的根各沿自己的实例表链判，链上有一片读不出的根跳过，一行都没判到时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 6 行） |
| I-3.13 | 记账行代段等于这一版的 txg | 走读到的任一版（候选根，以及由记录施加出来的那几版）的记账树里，每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`，只判走全了的树；inode 号水位那一统计量的台阶行不判（D5（快照 / 空间记账机制） 已定项 2 射程）。依据 D5（快照 / 空间记账机制） 与 D16（发布语义） 已定项 6 的「记账代 = checkpoint 号」（每条根的记账树只装这条根那一代的行）。 | 已实现（池级 checker 走读里的 `walk_tree_table_entry` 对每一版的记账树判，版本含由记录施加出来的，只判走全了的树；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-3.14 | 实例表每片记录形态合法 | 走读到的实例表每一片里：每条记录的 `kind` 只有 0（行）或 1（链指针记录）；链指针记录恰好是这一片的最后一条、片中间不出现，且自身合法（有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零，且位置指针头部 MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移都是第一版的取值：有一段不是第一版的取值，这一片也解不开）；`kind` 0 行的 flags 字节为 0。依据 D18（块里携带什么信息） 已定项 11（`kind` 不是 0 或 1 ⇒ 这一片解不开；flags「第一版恒 0、非 0 拒收」；链指针记录「恒为一片的最后一条」，最后一条不是合法的链指针记录同样解不开，位置指针头部的第一版取值见 D19（块指针的结构与宽度预算） 已定项 3），用户 2026-10-06 定另立；头部那一形用户 2026-10-06 定算第五种形态（岔路单 `research/prompts/m3-batch10o-code-r1-forks.md` K2-1 那一行） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_page_record_forms` 判前三件、`walk_instance_table_chain` 里链指针记录那一格判最后一件，走到的每一片都判；不成立的一片解不开：行不收、链不往下走、候选集滤不全（I-9.29（同号只许一个出生代） 的门关）；只读不判的读法与引用集合那一遍同一个读法；GPU 走读内核同判，扫描内核的引用集合那一段与只读读法、恢复内核读实例表同判 flags；有下一片而位置指针头部有一段不是第一版取值的那一形：CPU 三处读法（主走读、只读不判的读法、引用集合那一遍）经 `crates/singlefs-checker/src/walk.rs` 的 `chain_record_view` 判，GPU 走读与扫描经 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 的 `w_chain_record_kind` 判，恢复内核读实例表同判；坏镜像两张（位置指针 MAC 不全零、extent 偏移非 0），在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |
| I-3.15 | 未释放记录罩住的单元由映射指在这一槽 | 回退候选集里每一版（连同由记录施加出来的那几版）的分配记录里每条未释放的记录：罩住的那一槽是根记录或父指针直接指着的（豁免三类——映射树各节点、树表、实例表各片——与根记录持有的那几样，含树表 0 条那一版由根记录持有、树 ID 0、不进映射的那棵分配记录树的各节点），成立；否则读那一槽上的单元头，头里的出生身份（类标签、诞生代号、写序实例、尾段；码 1 / 码 3 再加偏移 43 的出生树，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不比，同 I-1.11（映射 key 与单元头相符））与同一版中央映射里某把 key 相符、且那把 key 的位置条目指着这一槽（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 的 N2 反查、G9 的「已死」「不一致」两类判红）。头用不了的那一格没有对象（读不出本身归 I-1.1、I-2.1）；隔离的单元（有一份副本读盘核出与位置项对不上，D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，G9 单列的「隔离」类）不判；一版的树表、分配记录树、中央映射树或实例表链读不全时那一版不判（射程里有一部分判不了） | 已实现（2026-10-15，里程碑四己 3：池级 checker `walk::judge_unreleased_allocation_records_against_the_central_mapping`；三形必红世界——只改映射不写副本、只写副本不改映射（只供测试的发布次序开关）、有记录没人引用（G9「已死」）——与合法搬迁之后成立的世界在 `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`；「映射条目改指一份没人分配的副本、分配记录不动」那一形（按槽数加总 I-3.1 / I-3.11 看不见）在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`，主 agent 2026-10-15 定加 N2（`records/2026-10-13-里程碑四预开发.md`「己 2 交回」那一条）。映射 key 与单元头不符的几份坏镜像上与 I-1.11 一起红。隔离那一格照 I-3.1 那一份隔离判据（`walk::some_copy_of_the_unit_fails_its_location_entries`），实现员 2026-10-15 推的读法，交主 agent 认。GPU 判器下标 95：内核不置位、对拍不比，主 agent 合入时补登） |
| I-3.16 | 碎片度与全空段统计对得上 | 最新根记账里每块盘的 runs（D5（快照 / 空间记账机制） 已定项 4 第 10 项：单元区空闲槽的连续段数）与全空聚簇段数（第 11 项；D3（空间分配） 已定项 10 ①：段内 64 槽都没有未释放分配记录的整段数，C372（全空聚簇段数两份定义相反） 按记录口径收口）等于全扫重算的值，全扫不与分配器的增量维护共享代码（C413（碎片度与全空段统计没有不变量））。占着的槽取 I-3.1（已分配统计对得上） 那一份并集（全部走过的版本的引用、隔离豁免、F 之下仍在 defer 的）；那块盘上它的槽数乘 16 KiB 不等于记账的已分配（I-3.1 在这块盘上不成立）、最新根带的 F 低于 F 生效值、或最新根下面没有记账树时不判 | 已实现（2026-10-15，里程碑四己 3：池级 checker `walk::judge_fragmentation_and_empty_segment_statistics`，runs 由 `walk::free_runs_between_the_occupied_slots` 全扫重算；判别力自证「增量少加一次」（runs、全空段数各少一，只红这一条）与合法搬迁之后成立的世界在 `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`）。全空段那一半在两种口径分得开的盘上照判（2026-10-15 起，里程碑四写段收尾）：分配器 `allocator::DeviceFreeMap::empty_segments` 改成按记录口径增量维护（defer 槽不算占用、只数整段），C372 代码那一半收口；崩溃恢复抛弃一条根之后那份干净镜像上两种口径分得开，`checker_known_bad_images.rs` 钉住它在那里判成立。GPU 判器下标 97：内核不置位、对拍不比，主 agent 合入时补登 |

> ⚠️ **I-3.5（引用区间的精确性）四条子判据双侧夹死 与 I-3.1（已分配统计对得上） 正交，两条都必须有。** I-3.1（已分配统计对得上） 是标量总账（已分配 == 遍历和），
> 而**归属错误对它不可见**——块被挂到错的 deadlist 上时，它仍然被恰好一处引用，总账依然平。
> 要检出归属错，必须有**按快照分箱的向量等式**。
> 这与 [decisions.md](../../../.claude/kb/decisions.md) D14（双轨（大小文件 / 持久临时）） 已定项 6 推导 D-3（多轨兼容可判据） 里「标量等式 → 按轨道分箱的向量等式，
> 判别力净增」是同一个形状。
>
> ⚠️ **数据丢失方向只在空间被重新分配之前可检出。** 该进 deadlist 却被释放的块，
> 一旦其空间被复用，症状就退化成 I-2.1（校验和与内容匹配） 校验和不匹配，即「损坏，原因不明」。
> **所以这类检查必须在崩溃点重放 harness 里紧跟着删除动作跑，不能只在全量里跑。**
>
> ⚠️ I-3.1（已分配统计对得上） 是**唯一允许全盘遍历的记账检查**——它是 checker，不是运行时路径。
> 运行时记账必须增量维护（D5（快照 / 空间记账机制）），checker 才用遍历来验证增量维护有没有出错。
> 这条对照关系是本工程验证记账正确性的核心手段。


```

**出处 `crates/singlefs-checker/invariants/04-校验和.md:1-12`（整段抄，未转述）**

```markdown
## I-2 校验和 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配。按多长去比（父条目的类型段说的那一类与孩子头里的类标签不一致时）：走读往下走孩子的那一支，按父条目类型段说的那一类的长度读回比；不往下走孩子的几支（墓碑、谱系、inode 三棵树的类型段不认得、孩子自证成另一类，墓碑、谱系两棵树层级 ≤ 1 节点里类型段 0 的），每条位置条目按那一份头里类标签说的那一类的长度比，类标签不在登记表里或头读不出的，16 KiB 与 32 KiB 两种长度都对不上才算不匹配。同一块、同一对校验和，两支可以判得不同（用户 2026-10-06 定保持，`research/prompts/m3-batch29-code-r1-main-verification.md` 第四节） | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判；「被引用的块」在中央映射认的位置上（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3、G6，D19（块指针的结构与宽度预算） 已定项 5 / 8）：豁免三类（映射树自己的节点、树表单元、实例表单元）之外，指针里的位置条目只当提示——走读先按提示读、只读不判，两份都对不上（校验和或头里的出生身份）再按映射位置读、逐份判这一条；提示过期、提示那一槽已释放或已复用都不判红，映射条目指的每一份由映射那一遍判（合法搬迁过的镜像与「映射指的那一份内容不对」的必红世界在 `crates/singlefs-harness/tests/checker_relocated_images_judged_by_the_mapping_position.rs`）；「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-2.2 | 校验和算法全盘一致 | 校验和算法标识在整个文件系统内一致。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：系统配置自举头段「校验和算法标识 1」，登记表 0 = 无效、1 = CRC-32C，第一版写 1；读到 0 或未登记的码一律拒绝挂载。算法的四个参数写死在 D18（块里携带什么信息） 已定项 17 | 未实现（条款已写、检查仍欠），前置：C309（32 字节校验和的算法没定） |
| I-2.3 | 补齐字节为零且参与校验和 | 任一数据单元（码 1）或打包记录单元（码 3，D18（块里携带什么信息） 已定项 11），其声明长度（净荷字节数；补齐起点 = 头 + 声明长度）之后到单元末尾的补齐字节**恒为 0**，且校验和 / MAC 覆盖**整个单元含补齐**（[decisions.md](../../../.claude/kb/decisions.md) D4（校验和位置） 已定项 2：不足一个单元的 extent 补齐到 32 KiB；码 3 由头校验和（到头末尾）与载荷校验和（头之后到单元末尾）两段合起来覆盖整个单元）。补齐字节非零、或校验和只覆盖声明长度，一律判红。索引节点（码 2）同样在射程内（D18（块里携带什么信息） 已定项 18 用户定案）：声明长度 = 条目区字节数，之后到节点末尾的补齐字节恒为 0，载荷 CRC 覆盖 [86 + 2 × key 宽, 16384)——起点是明文头末尾，含 29 字节预留位、条目区与其后的补齐区，口径与码 1 的 105、码 3 的 107 同（预留位 2026-09-14 从 28 加到 29，含预留位的头是 115 + 2 × key 宽）。⚠️ **加密开启时判的是解密后的明文，不是盘上字节**（2026-09-07 用户定案，还清 C189（I-2.3 在加密开启时判明文还是盘上字节没定））：补齐在盘上是 `E(0)` 而不是 0，按盘上字节判必然全红；checker 本来就要解整单元，取值同一步就有。**与 D18（块里携带什么信息） 已定项 7 的载荷 CRC 方向相反是对的**——载荷 CRC 要给无密钥一侧用，所以算盘上字节；这一条只有 checker 用，所以判明文 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节、指针头部的 MAC 16 + nonce 12 与算法类型 1、压缩算法码 1、压后长度 2（D19（块指针的结构与宽度预算） 已定项 3；第一版不加密、压缩算法码登记表只有码 0、码 0 时压后长度恒 0，D9（加密） 已定项 10 / 已定项 14）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。根槽偏移 428..457 的算法类型 1 + nonce 12 + MAC 16（D22（单元原子性怎么合成） 已定项 7 的字段表）是同一条规则的第三处：加密关着时恒 0，池级 checker 对根环里每一条读得出、自证过的根判这 29 字节全 0（不按回退候选集滤），core 的根记录读者不读它们、挂载透明也不比它们（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1「算法、nonce、MAC」那一行：单镜像性质，归池级 checker）。根槽 [457, 槽宽)（槽宽 = `physical_block_size`，D22（单元原子性怎么合成） 已定项 7「512 槽内余量 55」那一段）与系统配置槽 [489, 4096)（字段表之后到槽末，D22（单元原子性怎么合成） 已定项 9「槽内余 3607」那一段）第一版写 0，core 的读者不读这两段；池级 checker 对根环里每一条读得出、自证过的根判 [457, 槽宽) 全 0，对每块盘每个可择的系统配置槽判 [489, 4096) 全 0（只判可择的槽），非 0 判红这一条（用户 2026-10-20 定 C728（根槽与系统配置槽尾部字节没有条款）「恒 0，由 checker 判」）。往这两段余量里加字段时，同一次改动改 checker 的端点（457 随 C773（读坏与全零根槽的回收门槛钉只在内存里） 改 466 时这一句同改）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里四条指针（树表、实例表、中央映射树、分配记录树）头部有一段不是第一版的取值（MAC / nonce 非 0、算法类型非 0、压缩算法码非 0、压后长度非 0、extent 偏移非 0，五样任一）按这条根自证不过处置：core 的根记录读者（`RootRecord::parse_slot`，与 journal 新根段那四条指针同一判法）把这条根算不可择、退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs`）另判单元头偏移 4 的格式版本 = 1（不认得就判该结构损坏、不按版本 1 的字段表往下解）与 29 字节预留位全 0；走读跟随的每条指针头部按三个判法各判一遍（`crates/singlefs-checker/src/image.rs`：`judge_pointer_mac_and_nonce_are_zero` 判 MAC 16 + nonce 12 全 0，`judge_pointer_encryption_and_compression_fields_take_the_first_version_values` 判算法类型、压缩算法码、压后长度三段各一格全 0，`judge_pointer_extent_offset_is_zero` 判 extent 偏移 0），五个入口是码 2 节点指针（`Walk::read_index_node`：树表单元、树表条目的树根、中央映射树、inode 树、extent 树、分配记录树的节点都经它，根记录里树表、中央映射树、分配记录树那三条根指针也经它）、墓碑树与谱系树内部条目的子指针（`walk_the_entries_of_a_container_tree_node`）、inode 内部条目子指针（`walk_inode_internal_node`）、实例表链指针（`walk_instance_table_chain`，根记录里的实例表指针是链的第 0 片；链指针记录有下一片而位置指针头部有一段不是第一版的取值时，这一片按 I-3.14（实例表每片记录形态合法） 第五形解不开、链不往下走，那条指针不跟随、这一处 I-2.4（头校验和覆盖范围） 不判）、extent 数据指针（`walk_extent_data_pointer`），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；根槽偏移 428..457 那一处也已实现（`image::judge_root_record_algorithm_nonce_and_mac_are_zero`，`walk::check_pool_image` 经 `check_pool_image_given_the_container_capacity_knobs` 在 `valid_roots` 的结果上调它）；根槽尾 [457, 槽宽) 与系统配置槽尾 [489, 4096) 两段也已实现（C728（根槽与系统配置槽尾部字节没有条款）：`image::judge_root_slot_bytes_past_the_record_are_zero` 挂在 `judge_root_record_algorithm_nonce_and_mac_are_zero` 那一处调用里，`image::judge_system_configuration_slot_bytes_past_the_field_table_are_zero` 挂在 I-7.16（系统配置槽世代号与槽号的关系） 的 `judge_system_configuration_slot_generations_follow_the_rotation` 那一处调用里，两处调用点等 `walk.rs` 能改时挪进 `check_pool_image_given_the_container_capacity_knobs`；两段里第一个非 0 字节由 `crates/singlefs-checker/src/lib.rs` 读槽时取（`first_non_zero_byte_past_the_record`、`first_non_zero_byte_past_the_field_table`，起点是 `ROOT_RECORD_BYTES` 与 `SYSTEM_CONFIGURATION_BYTES`）；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的同名两函数与常量 `SYSTEM_CONFIGURATION_FIELD_TABLE_BYTES`；坏镜像是 `crates/singlefs-checker/src/image.rs` 的库单测 `a_non_zero_byte_right_past_the_root_record_reddens_header_checksum_coverage`、`a_non_zero_byte_right_past_the_system_configuration_field_table_reddens_header_checksum_coverage`，GPU 对拍 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 的 `the_gpu_judge_agrees_with_the_cpu_when_a_root_slot_carries_a_non_zero_byte_past_the_root_record`、`the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_carries_a_non_zero_byte_past_its_field_table`）；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半与算法类型、压缩算法码、压后长度那几段已实现**（checker 走读跟随的每条指针都判这三格；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_pointer_head` 与 `w_judge_pointer_extent_offset` 在同样那几个入口判，根槽那一处是同文件的 `judge_root_record_algorithm_nonce_and_mac_are_zero`；core 每个读盘上指针头部的读者都判这五段（MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移）、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from`，`PointerHead`、`NodePointer`、`DataPointer` 各一个；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）经 `NodePointer::read_with_head_fields_judgement_from` 的 `NotJudgedByTheShadowLedger` 不判（影子账要多隔离、不漏隔离）；走读不跟随的指针 I-2.4（头校验和覆盖范围） 不判：根环里走读没走到的旧根，它们的树表单元只被 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 读条目取诞生 txg、不跟随条目里的树根指针，这些指针的头部没人判（测过的镜像上 12 条：三个旧树表单元各四条树根指针，`research/prompts/m3-formal-code-b30-r1-sonnet-output.md`；core 把那样一条根当恢复目标时经 `TreeTableEntry::parse` 判，推的、没造盘面验证）；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`，加密、压缩那几段 core 每类读者各一条在 `crates/singlefs-harness/tests/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs`，根记录四条指针各改一类字段、两种挂载都退到上一条根的在 `crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs`；池级 checker 的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：extent 偏移落在墓碑树、谱系树的容器子指针上（`an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant` 与谱系树那一份），算法类型、压缩算法码、压后长度各一份（`an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant`、`a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant`、`an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant`），根槽算法类型、nonce、MAC 各一份（`a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant` 等三条）；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行、`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行、`research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 的容器子指针那一行、`research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2 的算法类型等三段与旧树表根指针两行、`research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 的根记录四条指针那一行） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |


```

**出处 `crates/singlefs-checker/invariants/06-事务与崩溃一致性.md:1-21`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放之后那一版的树表指针全零时红：core 的根记录读者不把全零当特例，照它那两条位置条目读盘 0 槽 0、读不出（两种挂载都拒），另起的走读跟过去、记一次走读失败（`walk.rs` 的 `Walk::walk_version_applied_only_by_records` 里的 `follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`；用户 2026-10-06 弹窗定，选项原文「甲 两类都判损坏」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）；重放施加了几次发布时判的是最后那一版，前面某次发布全零而最后一版完好时不红，core 也照常挂上。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录新根段的七项：树表指针、中央映射树根指针、树 ID 水位、F、实例表指针、分配记录树根、卸载记号，只有 fsid 照所选根；D23（journal 的角色与格式） 已定项 15）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，已做（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；崩溃点重放里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；崩溃点重放每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归崩溃点重放） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现，前置：C870（提交原子没有崩溃点重放 oracle）（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 已实现（2026-10-15 起判 logged_ops 里的条目（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1「I-4.4 的判法」，正推 `research/prompts/m4-gc-r1-sonnet-output.md` 的 N4）：每一版 logged_ops 树里每条条目——种类码是登记过的（1 = 整理、2 = 截断）、整理条目的 value 只装字段表那几段（白名单：R 的设备、起始槽、槽数、`c0`、`segs0`）、末尾补零 6 字节全 0（「再搬 N 个」这样的增量只能落在白名单之外）、R 对齐（起始槽 64 对齐、槽数 = 64）且在单元区内（设备是池里的盘、[起始槽, 起始槽 + 64) 落在 [单元区起始槽号, 盘末) 里）、`c0` 与签发 txg 都不晚于这一版的 txg。池级 checker `walk::judge_compaction_intents_of_the_versions`；logged_ops 树本身（种类 12，key 10 = 种类 2 + 签发 txg 8，条目 44）在主走读里照多层码 2 树走、判 I-1.1 与 I-1.10、记进引用。一条条目都没有时报不适用，理由逐字是 `crates/singlefs-checker/src/image.rs` 的 `NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION`（别的意图载体——nlink = 0 的记录、`deleted_inodes` 的 key、墓碑——没有可装增量的字段，用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`）。必红世界（补零里塞「再搬 5 个」、R 不对齐）与合法世界在 `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`，整理意图按字节造进 logged_ops（整理意图上盘归核心层）。2026-10-16 起收截断条目（种类码 2，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K7 T-1）：value 只装 inode 号、对象出生代、目标 size，末尾补零 10 字节全 0（「再截 N 个」这样的增量只能落在补零里），签发 txg 不晚于这一版，点名的 inode 在同一版的 inode 树里有记录、nlink ≥ 1、对象出生代相等、目标 = 记录的 size；有截断条目的那一版至多一条（`walk::judge_truncation_intent_against_its_record`，那一版的 inode 树不是恰好一棵或读不全的报射程里判不了）。合法世界（截断的意图与各批，截断条目由核心层真写）与必红世界（补零塞增量、种类码 3、两条截断条目、目标 ≠ size、出生代不符，各只红这一条）在 `crates/singlefs-harness/tests/checker_truncation_holes_and_intent_must_red_worlds.rs`。GPU 判器下标 73，内核不置位；它从恒不适用变成判得到，要登记进对拍不比那张表，主 agent 合入时在 checker 档补登；截断条目那一形 GPU 判器还没判，截断那一批的 GPU 一侧（判决第四节「截戊」）补） |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](../../../.claude/kb/experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态）。「其余全部不变量仍然成立」由各条在每个被判的版本上各自判，中间态的版本也在内；I-4.6（中间态其余不变量成立） 自己判的是单元数对 size 那一格（截断那一批合并 C711（单元数对 size 只判了 nlink = 0 的记录），判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-3 与第六节 S1）：每条记录（nlink = 0 与 ≥ 1 都判），存在的数据单元从 0 起连号；上段叶条目标签 1、2 的单元数等于 ⌈size ÷ 数据单元净荷容量⌉.max(1)（实现 `crates/singlefs-core/src/write_request_split.rs` 的 `data_unit_count_of_a_sequential_write` 里 `div_ceil(...).max(1)`）、不许有洞，有一条对得上的截断条目（inode 号、对象出生代相同、目标 = size）时 size 之外可以多；标签 3、4 至少一个洞单元（存在的单元至少一个、少于它）；标签 0 与没有上段叶条目的号一个单元都没有。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（池级 checker `walk::check_pool_image`，函数 `judge_data_unit_counts_of_the_inodes_against_their_size`；净荷容量 checker 自己算一份（`data_unit_payload_capacity_in_bytes`）；每条记录判两句（连号、单元数对标签），一条记录都没有时报不适用并带理由；截断那一批的合法世界（截断各版、变长留洞、有洞文件的 unlink 意图那一版）与必红世界（中间缺一个单元、标签 1 却少尾单元、标签 3 而没有洞、nlink ≥ 1 多出单元且没条目）在 `crates/singlefs-harness/tests/checker_truncation_holes_and_intent_must_red_worlds.rs`；坏镜像：第一个文件的 size 加一个净荷（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_size_of_the_first_file_by_one_data_unit_payload`）；GPU 判器下标 72，扫描内核判的还是只判 nlink = 0 记录、不看标签的那一形：有 nlink ≥ 1 记录的状态上 CPU 判到位、内核不判到位，对拍对不上，截断那一批的 GPU 一侧（判决第四节「截戊」）补；用户 2026-10-05 弹窗定「只判单元数对 size」，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第四节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`；C711（单元数对 size 只判了 nlink = 0 的记录） 由这一批并进来、射程改成每条记录，欠账表那一行归阶段同步改） |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在。「处于删除中的 inode」指两样：inode 树里 nlink = 0 的记录；extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（池级 checker `walk::check_pool_image`，函数 `judge_inodes_being_deleted_against_the_queue_and_their_data_units`；每个处于删除中的号判一次，一个都没有时报不适用并带理由；坏镜像：extent 树里有数据单元、inode 树里没有它的记录、号也不在队列里（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `drop_the_record_of_the_first_file_and_keep_its_data_units`）；GPU 判器下标 71，扫描内核判；对象取这两样，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节逐格判决表里 I-4.7（删除中 inode 有账） 对象那一格） |
| I-4.9 | 非最新候选根引用的单元不晚于它 | 回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。最新那条根归 I-4.2（无被引用未提交块），不在这一条里。依据 D16（发布语义） 已定项 7（单元先于根落盘）。 | 已实现（池级 checker `walk::judge_unit_birth_against_an_older_candidate_root` 判走读里每个被引用的单元，共享子树里别的版本先走过的单元由 `merge_placements_below_a_unit_walked_by_an_earlier_version` 补判；候选集只剩一条根时报不适用，同 I-3.9（释放代落在停止引用它的那一格区间里）；坏镜像沿 I-4.2（无被引用未提交块） 那一份的造法，映射 key 也要跟着改，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行） |
| I-4.10 | 截断中间态有账 | 每条 inode 记录，存在的数据单元多于 ⌈size ÷ 数据单元净荷容量⌉.max(1)（size 之外还挂着单元）时，同一版 logged_ops 里有一条对得上的截断条目（种类码 2；inode 号、对象出生代相同，目标 = 记录的 size）。判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-2（截断「先落目标」：意图那次写 size = N、各批只从尾部放、最后一批与删条目同一次发布，K2）。只判这一个方向：截在最后一个单元之内的那次意图之后 size 之外一个单元都没有、条目照挂着（下一步是放 0 个单元的最后一批，K11），那一版合法；条目对不对得上记录由 I-4.4（意图只描述目标态） 判。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法，logged_ops 读不全的那一版报射程里判不了 | 已实现（2026-10-16，池级 checker `walk::check_pool_image` 的删除态扫描，函数 `judge_data_units_beyond_the_size_against_the_truncation_intents`；每条记录判一次，一条记录都没有时报不适用并带理由；合法世界（截断各版、截在唯一单元之内的意图那一版）与必红世界（截断中间态上删掉截断条目、留着 size 之外的单元；nlink ≥ 1 的记录 size 改小一个净荷、没有条目）在 `crates/singlefs-harness/tests/checker_truncation_holes_and_intent_must_red_worlds.rs`；GPU 判器下标 99，这一批不进覆盖表、内核不判，截断那一批的 GPU 一侧（判决第四节「截戊」）补） |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。


```

**出处 `crates/singlefs-checker/invariants/02-根环.md:1-38`（整段抄，未转述）**

```markdown
## I-7 根环 —— 类

**依赖 D22（单元原子性怎么合成），形态已定，已实现（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根） 除外：形态 2026-10-20 定，随 C773（读坏与全零根槽的回收门槛钉只在内存里） 实现批）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-7.1 | 根槽有效集非空 | 全部根槽位中，**自证校验和通过**的记录集合 S 非空 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-7.2 | 最新根可完整遍历 | S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态（[decisions.md](../../../.claude/kb/decisions.md) D21（权威态与派生态的分界） 定义：单元 + 记账），无一处失配 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-7.3 | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到 | 已实现（池级 checker `walk::check_pool_image`，例外（自证过的根全部是第 0 代）照判据原样实现；坏镜像建在 mkfs 刚写完那份上（三条第 0 代根的 checkpoint_txg 一起改成 1），例外那一支另有一份阳性对照；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](../../../.claude/kb/decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；被抛弃时间线只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14；留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出）。⚠️ F_生效 按 D16（发布语义） 已定项 1「生效」取根上与系统配置里读得出的最大值；checker 与崩溃点重放也按它取回退候选集下界（`crates/singlefs-checker/src/walk.rs` 的 `check_pool_image`，C556（checker 与崩溃点重放不读系统配置里的 F） 已还清），带 F 的根全坏时不在合法状态上误红这一条与别的按回退候选集判的几条。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 已实现（池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格：走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头；坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） 被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1（校验和与内容匹配） 违例或走读失败就按这条根判红，判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（`walk::judge_blocks_referenced_by_abandoned_roots`，用例在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`）；根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在这一半上照实红，等 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 修好转绿。 |
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是判定宽度的整数倍，判定宽度 = max(声明的 `physical_block_size`, 声明的 `io_min`)（`io_min` 为 0 视为没声明）；mkfs 在任何写之前先拒不按这个宽度对齐的几何，非常规几何由 mkfs 拒、不再是合法镜像。**逐槽判，不是只判第 0 个**——偏移半个槽的槽照样跨 2 个原子单元（E41（根环槽几何 2×4×256） 实测：对齐时风险槽数 1，偏移 256 时是 2），撕裂会连带打掉相邻的槽而没有任何东西报警 | 已实现（池级 checker `judge_root_slots_aligned_to_the_decision_width`（`crates/singlefs-checker/src/image.rs`）逐槽判：判定宽度取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的（异构池取最大，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 1 条），是声明值、注明「声明值，未探测」，探测比对仍欠 C388（checker 的判定宽度不自己探测）；mkfs 在任何写之前拒不按这个宽度对齐的几何（`crates/singlefs-core/src/make_filesystem.rs` 的 `first_slot_not_aligned_to_the_decision_width`）；异构池一份与逐槽判的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 5 行） ⚠️ **射程注（D2（RAID 条带策略） 已定项 15）**：`io_min` 大于 `physical_block_size` 时按 `io_min` 判——已定项 17 收窄后根槽与系统配置槽在它的射程内。 |
| I-7.6 | 根环区域落在互不相同的盘上 | R 个区域**在 mkfs 时逐区域写下的设备身份**（D2（RAID 条带策略） 已定项 7 用户定案逐字「归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来」）取满 min(R, devs) 个不同值，且没有一块盘背超过 ⌈R / devs⌉ 个区域（鸽笼下界，D22（单元原子性怎么合成） 已定项 14，2026-09-13 用户定案）：devs ≥ R 时就是两两不同；devs < R 时不拒绝挂载，容错承诺按盘数降级（E87（固定结构的放置） 量过的 2 盘 R = 3：掉盘 1 最坏 1 代、掉盘 0 最坏 2 代）。取值不满或某盘超过下界即**拒绝挂载**——名义容错 F 个失败域而实际区域挤在同一块盘上，是 E47（根环失败域的损失） 实测过的形态（`even_span` 在 18 个几何格里只有 3/18 真的落到 4 块不同盘） | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-7.7 | 系统配置实例代号不低于根环 | 可写挂载独占打开成功的那个集合里（它至少是过半，过半是准入门槛不是写入范围），每块盘的系统配置实例代号按它两槽里全部自证过（校验和过且 fsid 与本池相同）的槽读，取号用同一个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者，任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」；② 各盘的实例代号（每盘取它全部自证过的槽里最大的那个）不等时，较大者不出现在任何根记录、journal 记录、单元写序里，镜像记不下这次挂载独占打开的集合时报「不适用」，第一版两盘就是全部盘。取号没做完、全或无失败回卷期间的代号不等由 ② 判成立，不另开例外（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的取号规则）。只在 devs ≥ R 时根环才是系统配置之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） ⚠️ **2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 用户定案改成 ① ②**：checker 读每盘两槽里全部自证过的系统配置，扫描方向加读 journal 环与三类单元头的写序；崩溃点重放全量 262165 个状态 I-7.7（系统配置实例代号不低于根环） 违例 0（旧定义「各盘择到的相等且 ≥ 根环」下取号只落一块盘的 2 个状态判红）；坏镜像三份：没人引用的实例 2 孤儿（①）、journal 环里实例 2 的记录（①）、盘 0 最大号 2 而孤儿带 2（②） |
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） 崩溃孤儿两条流（第一个文件版本那次发布的每个前缀 → 可写挂载 → 挂载之后六种动作）在 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`；中间实例行（T = 0）不排除的坏镜像是 `checker_known_bad_images.rs` 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`（实例 2 的根槽全清零之后实例 3 写中间实例行 (2, 0, 0)，根环水位压低之后只红 I-7.8（根记录树 ID 水位不低于全池最大树 ID）；回退行那一道随回退行删掉，D18（块里携带什么信息） 已定项 11）；实现在 `walk::written_after_its_instance_was_last_applied` ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲；2026-09-24 主 agent 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 1 条补第二道，被攻过零轮）**：崩在发布之前的那个事务写出的孤儿节点不算「出现过」。扫描方向不数两类码 2 节点：① 诞生代号超过根环里最大 checkpoint_txg 的；② 写序实例 i 在最新根指着的实例表里有行 (i, Ti, ·)、Ti > 0、而诞生代号 > Ti 的——行说实例 i 只被施加到 Ti，之后写的没发布过；下一个实例第一次发布取的 txg 可以等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 取等号），只靠 ① 挡不住（第一个文件版本崩在记录落盘之前、再可写挂载的两条流上 130 格在合法状态上判红）。Ti = 0 的行（恢复写的中间实例行）不排除：中间实例发布过的号仍要算（D8（核心索引结构） 已定项 8 ②）。节点头 fsid 不是本池的照旧数。按字面数孤儿，崩溃点重放全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 ⚠️ **判法补一句（已实现：`walk::check_pool_image` 末尾在取 max 那一格之后再判最新那条根自己的水位）**：另判最新那条根的水位不低于全池最大树 ID。只取根环里全部根的 max 判不到把最新那条根的水位压低的镜像：攻方量到 checker 49 条 0 违例，而 core 的 `recover` 报 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 失败；core 重发树 11..18 的状态 checker 也 0 违例。用攻方那两份坏镜像钉住（只压低最新那条根的水位；建一次树之后经会话入口覆盖写 24 次把发树那条根轮出去、再把环里每条根的水位压到 11，只红它），被攻过零轮（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 9、10 行） |
| I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；新实例的第一条根没有同实例的前一条，不算抬——它带的是恢复算出的 F_生效；管理员回退是现行实例里的一次向前发布，带的 F 与前一条相同，不算抬；一次抬 F 推的几次空发布只有第一条算）根记录里那个回退下界 F（D22（单元原子性怎么合成） 已定项 7 的字段），按这条根 flags 里的**卸载记号**（D22（单元原子性怎么合成） 已定项 7 flags 位 0）分两支判：**带卸载记号的**，F 不高于根环里 txg 比它小、又落在锚在镜像最新根上的那一个一圈窗口里的最新那条根的 txg（只取这一个窗口，`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_floor_raises_against_their_ceilings`；窗口里一条都没有时不判，同不带记号那一支的「之前的根」；盲带：F 越过洞里一条有效根时这一支看不见，欠账见 C717（回退下界上限的单窗口盲带）；正常卸载把 F 抬到现行那一版的 txg、不另判上限，D16（发布语义） 已定项 1「抬 F 的上限」）；**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」（两支都取）取锚在镜像最新有效根的 txg 上的那一个一圈窗口里 txg 比它小的那些（窗口 = txg > 锚 − 3S，3S 是环长；用户 2026-10-04 定「乙，两支都取两个窗口」，第 10 批起锚在被判那条根上的那个窗口被这一个罩住、不再另取，主 agent 认的读法、被攻过零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 10 批 core 簇交回那一条），窗口里没有比它小的根时那条根不判；只取锚在被判那条根上的窗口时，洞里留着的上一圈的旧根会顶上来、在合法镜像上判红（随机历史 230 段红 21、150 段红 9，并上锚在镜像最新有效根上的窗口之后各剩 1 段，红在带卸载记号那一支，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批 9-6 交回那一条）；取全环时攻方五批随机历史里每批 27–46 段红（判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第五节）；上限照 D16（发布语义） 已定项 1「抬 F 的上限」那一行，窗口里挂载那一刻读不出或自证不过、之后这个进程没写过的槽当作可能是一条非空有效根、F 不越过它，这些槽 checker 看不见，checker 只按镜像上读得出的根判（推的：读故障不留在盘上）。只判抬 F 的那一条，不在后来每张镜像上重算：F 抬过之后再回退，后来的根会把当初撑着 F 的非空根判成无效，现算的上限会低于 F，而 F 不回落（用户定案 2026-09-24，理由见实二报告：按「有效根不要求 txg ≥ F」的读法，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与两个随机种子在合法状态上判红）；非空 = 它的树表里 inode 树、extent 树的根指针与它前一条有效根（按 txg 排）树表里的不同——比的是树表里这两棵树的条目，不比树表单元自己的落点（D16（发布语义） 已定项 1 表格后那段「「非空」从盘上怎么认」）；数「第 4 新的非空」时去重：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入（同一段）。⚠️ 它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬（那是准入的事，D28（挂载期承诺量） 已定项 1）。⚠️ **卸载记号盘上没有别的东西核它**：准入入口误打记号、F 又抬过头，按带记号那一支放过（第三轮判决第二节 K6：(b) 丢的正是「准入抬过头却带了卸载记号」那一格）——这一格归崩溃点重放录制流另核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查）），不归这一条。⚠️ F 写进了系统配置而带新 F 的根没落盘或坏了、恢复后生效值怎么取的那一类，归崩溃点重放与 C556（checker 与崩溃点重放不读系统配置里的 F）（已还清：checker 与崩溃点重放的记录核对器已按系统配置里的 F 取下界），不归这一条。算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判 | 已实现，两支（两支的「之前的根」只取锚在镜像最新有效根上的那一个一圈窗口）：池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格，按根记录读出的卸载记号分支（checker 读成 `UnmountMarkerView`）。**不带记号那一支**：坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；崩溃点重放 `crash_enumeration_fixed_script_stream.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 7 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认、「4 个不同状态」不去重）；用户 2026-09-24 定义，里程碑「覆盖写、释放、回退与复用」收口表第 26 行。**带卸载记号那一支**：坏镜像同一文件的 `marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant`（正常卸载之后的镜像按带记号那一支判 6 ≤ 6 成立、按准入那一支会红；把那条带记号的根的 F 改成 7 只红它）；变异「checker 把每条根都读成不带卸载记号」「带记号那一支把不高于前一条根的 txg 判成低于」；记号的写读在 `crates/singlefs-core/src/root_record.rs`（写者照 `unmount_marker` 写 flags 位 0、读者拒记号以外的位），写者、读者各有一条变异（实二）。**一圈窗口**（用户 2026-10-04 定两个窗口，被攻过零轮；第 10 批起只留锚在镜像最新有效根上的那一个）：harness 用例在 `crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs`（带卸载记号的根在洞上再挂之后不许红，用例 `an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole`；两个窗口里仍照红，用例 `an_unmount_sequence_root_raising_the_floor_above_the_previous_root_inside_both_windows_still_reddens_the_rollback_floor_invariant`；准入那一支的几形同文件），`crates/mutations.tsv` 里以「第9批改钉」打头、点名带卸载记号那一支的 2 行，与以「第9批」打头、点名「一圈窗口」的 4 行；改钉之后随机历史 230 段、150 段两批 0 段红（改钉簇实现员量的，输出在 `research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/`）；GPU 判器的扫描内核两支同取锚在镜像最新有效根上的那一个窗口（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `in_the_one_revolution_window`），对拍用例与 8 行变异在 checker 档、没跑 |
| I-7.10 | 已停用·回退见证表删了 | **此编号不再使用**——2026-09-26 管理员回退改成挂着时的一次向前发布，系统配置里的回退见证表删掉（D23（journal 的角色与格式） 已定项 14、D22（单元原子性怎么合成） 已定项 9）⇒ 这一格**无对象**。曾要求：每块盘两槽里自证过的系统配置槽，槽内偏移 481 起的回退见证表解得开（条数不超过 R × S − 1、条目严格升序、目标实例代号小于新实例代号、条数之后全 0），同一个新实例代号在各盘各槽里记的回退目标相同 | 不适用 |
| I-7.11 | 已停用·见证择根删了 | **此编号不再使用**——2026-09-26 回退见证表删掉（D23（journal 的角色与格式） 已定项 14），择根不再跳过被见证抛弃的根 ⇒ 这一格**无对象**。曾要求：所选根是根环里自证过、不被回退见证表抛弃的根里 (txg, 实例代号) 最大的那一条；根环里至少一条自证过的根不被见证表抛弃；见证表里新实例代号不大于所选根实例代号的每一条，所选根指着的实例表罩得住它 | 不适用 |
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；崩溃点重放每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，崩溃点重放没跑） |
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。**判的字段与 core 收同一张表**：格式版本、加密类型、槽距（上界用同一槽自述的根环起点）、根槽宽、环长、单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上、journal 环起点（偏移 325）与根环起点（偏移 371）等于第一版常量，共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）。R（区域数，偏移 361）与第一版常量 3（`ROOT_RING_REGIONS`）不等、S（每区槽数，偏移 362）不在 [4, 16] 里同样归这一条、整池不作保：S 照 D22（单元原子性怎么合成） 已定项 2「越界整池拒绝挂载，与每区槽数 S 越界同一个处置」（core 报 `RecoveryFailure::RootRingSlotsPerRegionOutOfRange`），R 用户 2026-10-20 定 C663「读并整池拒」；两样此前归「这一槽不可择」，2026-10-20 起不再是。格式版本不认得与 incompat 位不认得同在一槽时先判格式版本（用户 2026-10-20 定 C663「格式版本先」）：这一槽归这一条、整池不作保，不归 incompat 不认得那一支的「这一槽不可择」 | 已实现（池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_system_configuration_values_the_reader_accepts` 在 `walk::check_pool_image` 里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回。**八项 checker 与 core 两边都判**：checker 解槽时判格式版本与加密类型（`crates/singlefs-checker/src/lib.rs`），其余六项在 `geometry_of` 判——先判 journal 环起点（偏移 325）= `JOURNAL_RING_START_SLOT`（1024）、根环起点（偏移 371）= `ROOT_RING_BASE_SLOT`（64）（两个起点先于槽距判：槽距上界按同一槽自述的根环起点算），再判槽距、根槽宽、环长，最后判单元区起始槽号（偏移 417）= 同一槽自述的环起点 + ⌈环长 ÷ 16384⌉（`unit_area_start_slot_is_the_slot_after_the_journal_ring`）且是 `CLUSTER_SEGMENT_SLOTS`（64）的整数倍；新加的四判各报一个 `Verdict` 成员（`JournalRingStartSlotNotTheFirstVersionConstant`、`RootRingBaseSlotNotTheFirstVersionConstant`、`UnitAreaStartNotTheSlotAfterTheJournalRing`、`UnitAreaStartOffTheClusterSegmentBoundary`），都归 `value_refused_by_this_reader`（整池不作保）；常量取自 `singlefs-format`，算式 checker 自己写。core 一侧 `crates/singlefs-core/src/system_configuration.rs` 的 `parse_slot` 读出偏移 325、371 两个字段，`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 按格式版本、加密类型、槽距（上界用同一槽自述的根环起点，`largest_fixed_structure_slot_spacing_under_the_root_ring_base`）、根槽宽、根环起点、journal 环起点、环长、单元区起点的次序判，不收的值报 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 的一个成员（两个起点是 `RootRingBaseNotTheFirstVersionSlot`、`JournalRingStartNotTheFirstVersionSlot`），经 `RecoveryFailure::SystemConfigurationValueRefused` 整池拒。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`（盘 1 槽 0 加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、其余不变量全报不适用），`each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`（几何字段各一格，各只红 I-7.13（系统配置池级字段在读者收的范围里）、违例说明点名被拒的那个成员）。两边收同一张表由 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 钉住：新池新建文件之后的镜像上两块盘四个系统配置槽改同一个几何字段、整槽校验和重封，五格各一条用例，每格 core 的只读与可写挂载都拒成同一个 `SystemConfigurationValueRefused` 值、两块盘逐字节不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、其余不变量报不适用；根环起点写 0 那一格两边点名的成员不同（core 先判槽距上界、报槽距，checker 先判根环起点、报根环起点），照两边各自的判定次序钉（用户 2026-09-27 定「读字段，不等于常量就整池拒」，`records/2026-09-24-里程碑二收尾调度.md`「用户定第二轮改法：四入口再核两样、core 读 325/371（2026-09-27）」那一行；代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「core 与 checker 对系统配置几何字段的读法」那一格）。R、S 与次序（C663）：checker 在 `geometry_of` 判 R、S，R 是 0、1 或 2、大于 3 各报一个成员（`RegionCountZero`、`RegionCountBelowTheFirstVersionConstant`、`RegionCountPastTheRegionDeviceFields`），S 报 `RootRingSlotsPerRegionOutsideTheFormatInterval`，四个成员都归 `value_refused_by_this_reader`；次序在 `crates/singlefs-checker/src/lib.rs` 的 `check_system_configuration_slot`（先格式版本、后 incompat 位），GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `checker_read_system_configuration_slot` 同序同判。坏镜像在 `crates/singlefs-checker/src/image.rs` 的库单测：`a_slot_declaring_seventeen_slots_per_region_refuses_the_whole_pool`、`a_slot_declaring_two_regions_refuses_the_whole_pool`、`a_slot_declaring_zero_regions_refuses_the_whole_pool`、`a_slot_declaring_four_regions_refuses_the_whole_pool`、`a_slot_with_an_unrecognized_format_version_and_an_unknown_incompat_bit_refuses_the_whole_pool_by_its_format_version`（盘 1 槽 0 改一处、重封，只红 I-7.13、其余报不适用），阳性对照 `a_pool_holding_only_first_version_values_reads_every_slot_as_usable_and_keeps_header_checksum_coverage`；GPU 一侧对拍 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 的 `the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_seventeen_slots_per_region` 与 `the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_declares_two_regions`|
| I-7.14 | 系统配置本盘设备号等于盘在池里的身份 | 每块盘两个系统配置槽里，每个自证过（magic 与整槽校验和对）、fsid 与本池相同的槽，自举头里的本盘设备号等于这块盘在池里的身份。字段住 D22（单元原子性怎么合成） 已定项 9 的系统配置字段表（自举头「本盘设备号 4」）；实现一侧是 D18（块里携带什么信息） 已定项 11 里挂载核盘表那一句：可写挂载比盘表（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`，`OwnDeviceNumberDiffersFromTheIdentityHandedIn`）。别的池的槽归 I-1.4（块头 fsid 一致） 判，不在这一条里 | 已实现（池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_own_device_number_is_the_device_identity` 在 `walk::check_pool_image` 里调用，每块盘每个自证过、fsid 属于本池的系统配置槽判一次，本盘设备号与 `ImageReader::devices` 给的这块盘的身份比。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：两块盘四个系统配置槽的本盘设备号互换，`devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` 钉只红 I-7.14（系统配置本盘设备号等于盘在池里的身份），这份坏镜像也在 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 的清单里；变异表 `crates/mutations.tsv` 里对着这一判的三行（判定恒真、池级走读不调这一判、坏镜像写回原值）改法 A 的 `prove-red.sh` 各抓到（`m2-impl-r2-fixes-a-implementer-report.md`「每条新测试的证红」那张表变异 6、7、8 那两行）。合法历史上不判违例：门禁 harness-model-differential-and-scenarios 随机历史那一趟 I-7.14（系统配置本盘设备号等于盘在池里的身份） 判绿 4155 次、不适用 0 次（同一报告「受影响的层 0 流与崩溃枚举用例」一节「I-7.14（系统配置本盘设备号等于盘在池里的身份）：判绿 4155 次、不适用 0 次」那一句）；崩溃点重放快档与全量没跑）|
| I-7.15 | 各盘所选系统配置槽设备数相同 | 各盘所选的系统配置槽（自证过、世代号最大者）里的设备数相同（D22（单元原子性怎么合成） 已定项 9 的字段）。不要求等于盘表：降级镜像（掉了一块盘）里 checker 手里的盘数小于设备数是合法的。形态抄 I-1.4（块头 fsid 一致） 的逐盘循环；core 的 `choose_system_configuration` 在几块盘的设备数对不上时报 `SystemConfigurationsDisagree`、整池挂不上，两边同表。 | 已实现（池级 checker `walk::check_pool_image` 逐盘把各盘所选槽的设备数同第一块盘所选槽的比；坏镜像是盘 1 两个槽的设备数改成 3 并重封，只红它，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C466（几块盘的系统配置设备数对不上时 checker 判不出）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7 行） |
| I-7.16 | 系统配置槽世代号与槽号的关系 | 每块盘上，任一可择的（自证过、池级字段在读者收的范围里）系统配置槽 i：世代号 ≥ 2 时世代号 mod 2 == i；两个槽都可择时世代号相差不超过 1，相等时只许都是 1（mkfs 写下的两槽世代号都是 1，D22（单元原子性怎么合成） 已定项 16）。带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里） 整池拒，不进这一条。 | 已实现（池级 checker `judge_system_configuration_slot_generations_follow_the_rotation`（`crates/singlefs-checker/src/image.rs`）：奇偶一格、差值一格，差值那一格取「两槽都是 mkfs 写的 1，或相差恰为 1」；差值与奇偶各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 7 行与第四节用户定案第 1 条） |
| I-7.17 | 根槽位置等于 txg 算出的位置 | 根环里任一自证过的根所在的槽位 (区域 r, 槽 s)：`checkpoint_txg` ≥ 1 时 r == txg mod R 且 s == (txg div R) mod S；`checkpoint_txg` == 0（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成） 已定项 8）时只判 s == 0，不判 r。公式是 D22（单元原子性怎么合成） 已定项 16 的定案。根环补洞判决要的「槽位与 txg 对得上」就是这一条（`research/prompts/m3-ring-r1-main-verification.md` 第二节第 6 行）。⚠️ **这一条红的镜像上，checker 其余不变量不作保**：合法写者（发布路径与 mkfs 种第 0 代根）造不出错位的根；checker 照旧把错位那条根当有效根，按有效根算的判定（I-3.1（已分配统计对得上） 的候选集、回收门槛与环里最旧有效根）会把它算进去，结论不作数（根环三方第一轮正推 X5 取甲、第二轮 Y5.2，判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第四节）。core 一侧：C743（重建分配器记新类时不核根住不住这一槽） 定案（用户 2026-10-20「所有读根环处都收」），读根环处照同一道落点公式核、读回不住这一槽的根按自证不过处置，第 0 代根照 mkfs 种根的槽另认，与这一条同一例外（根环三方第一轮判决 `research/prompts/m3-close-rootring-r1-main-verification.md` 第二节 X1）；core 那一半随根环实现批，今天没写。 | 已实现（池级 checker `judge_root_records_live_in_the_slot_their_txg_names`（`crates/singlefs-checker/src/image.rs`）：根环里每条自证过的根逐条判，第 0 代只判槽 0；公式同 core 的 `target_for_publish`（`crates/singlefs-core/src/root_ring.rs`）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 8 行） |
| I-7.18 | 根记录的回收门槛等于现算的门槛 | 根记录带回收门槛 t（8 字节）与钉槽 p（1 字节，取建账时还钉着的槽里钉值最小的那个）；两个字段在根记录里的偏移随 C773（读坏与全零根槽的回收门槛钉只在内存里） 实现批定，根记录宽 457 → 466（D22（单元原子性怎么合成） 已定项 7）。写者取 t = max(这条根自己带的 F, 分配器记这条根时的回收门槛)（回收门槛见 D16（发布语义） 已定项 1「可再分配」那一行）；checker 取这条根自己带的 F 现算回收门槛，要求等于 t。**只判建账的发布**（那次发布建了这条根自己的记账树）：零单元发布与无文件版本上的复制根，t、p 抄自上一版，环转满之后求等会误红，不判；最新根下面没有记账树时报不适用。坏实现里「t 恒取 F」与「p 写成没有钉、其实还钉着槽」这一条判得出（后者现算的门槛不带钉、比 t 高，求等红；第三轮辩方 Z4.4 推的，没造坏镜像），判不出的两类见 I-7.19（钉着的槽不住本实例写的根）。出处：根环三方第二轮判决 `research/prompts/m3-close-rootring-r2-main-verification.md` 第二节 Y4 那一行、第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C773 那一行与第三节用户定案（用户 2026-10-20 认 t 的取法与只判建账的发布） | 未实现，前置：C773（读坏与全零根槽的回收门槛钉只在内存里） |
| I-7.19 | 钉着的槽不住本实例写的根 | 根记录的钉槽 p 指着的那一槽（D16（发布语义） 已定项 1「根槽这一次读坏」那一行：读坏或读回全零的根槽把回收门槛钉在它可能住过的最旧 txg），镜像上不得住着这条根同一实例写的根——钉在这个进程写掉那一槽时必解，那一槽已住着本实例的根而 p 还指着它，就是写掉了没解钉。只判建账的发布，最新根下面没有记账树时报不适用，口径同 I-7.18（根记录的回收门槛等于现算的门槛）。两样故障叠在一起（读坏钉着的槽，这个进程对它的写报失败而实际落了盘）时这一条会误红，超出单故障，记在 C773（读坏与全零根槽的回收门槛钉只在内存里） 的射程里。**判不出的两类坏实现**：钉着的槽重读读出有效根而写者没解钉、凭空钉一个读得出的槽——读故障不留在盘上，镜像上与合法的钉逐字节相同；用户 2026-10-20 定不记成已认代价、要做成判得出，账在 C863（钉槽的两类坏实现镜像上判不出）。出处同 I-7.18（根记录的回收门槛等于现算的门槛） | 未实现，前置：C773（读坏与全零根槽的回收门槛钉只在内存里） |

> ⚠️ **I-7.3（环健康性） 的价值在于它不需要真的崩溃一次才能发现 bug。**
> 轮换逻辑写错（例如连续两代落到同一槽位）在**下一次正常提交后的 checker 扫描**里就会被抓到，
> 而不是等到某次撕裂真的发生、发现无处可退时才暴露。
> 这与「校验和会不会红」是两类判别力：前者查机制活着没有，后者查数据对不对。

> ⚠️ **I-7（根环） 类验不了「失败域独立」这个假设。** 根环的多区域布局依赖
> 「盘首与盘尾的物理故障 / 掉电撕裂事件统计独立」，而对着镜像判不出这一条——
> 它只能验证「机制按设计跑了」，不能验证「设计的假设是真的」。
> 该假设**本机无证据**，见 [decisions.md](../../../.claude/kb/decisions.md) D22（单元原子性怎么合成） 已定项 2。


```

**出处 `.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md:1-13`（整段抄，未转述）**

```markdown
## D19 (Block Pointer Structure & Width Budget) — Settled

[中文](19-块指针的结构与宽度预算.md) · **English** · [日本語](19-ブロックポインタ構造と幅予算.ja.md)

D19 (Block Pointer Structure & Width Budget) governs block pointers: how many sections a pointer is divided into, what fields are contained in each section, their widths, and their ordering; who holds the authority between position entries and the central mapping for dereferencing and release judgments; how the central mapping entry key and value are encoded; which segments of the mapping key travel with the pointer, and how birth sequence numbers are allocated. Out of scope: unit header field tables and unit type codes in D18 (information carried in a block); node size and intra-tree entry layout in D8 (core index structure); placement coordinate system and allocation record keys in D3 (space allocation) settled items 3 / 7, commit endogenous block bump sequence in D3 (space allocation) settled item 10; checksum coverage and the relationship between logical extents and units in D4 (Checksum Location); encryption algorithms, nonce derivation, and origin of expected AAD values in D9 (encryption); root record and system configuration field tables in D22 (Synthesizing Unit Atomicity) settled items 7 / 9; stripe width granularity in D2 (RAID Stripe Policy) settled item 1.

<!-- doc-lint:registry name-col=2 -->

| Identifier | Short Name |
|---|---|
| N1 | Reread once upon read failure |
| N3 | Mismatched entries remain in allocated |

```

**出处 `.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md:14-310`（整段抄，未转述）**

```markdown
### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Whether position entries carry device identity** | Yes: each physical position carries a dev id, without introducing a layer of chunk mapping table **Status: Settled.** **Progress: Implemented.** |
| 2 | **Bit-width of ciphertext checksum** | 32 bits; takes complete CRC32C, truncation from a wider CRC is forbidden **Status: Settled.** **Progress: Implemented.** |
| 3 | **Fixed-width or variable-length** | Fixed-width, reserving slots when unencrypted: MAC and nonce constantly occupy slots, kept empty and not repurposed **Status: Settled.** **Progress: Implemented.** |
| 4 | **Width of each section in position entry** | Device ID 4 bytes, physical offset 6 bytes (16 KiB slot number), ciphertext checksum 4 bytes => position entry 14 bytes **Status: Settled.** **Progress: Implemented.** |
| 5 | **Where location authority resides** | Position entry demoted to hint, central mapping is unique entrance for dereference and release verdicts **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Field table of central mapping entries** | Key reuses write sequence key by class (post-tightening): 27 bytes for Code 1, 25 bytes for Code 2 / Code 3; value follows settled items 4 / 5 **Status: Settled.** **Progress: Implemented.** |
| 7 | **Which segments pointer header carries** | Header 50: MAC 16, nonce 12, algorithm type 1, compression algorithm code 1, compressed length 2, extent offset 2, birth tree 8, birth txg 8 **Status: Settled.** **Progress: Implemented.** |
| 8 | **How much birth identity reference side carries, what mapping contains** | All pointers enter mapping, pointers to Code 1 are 88, pointers to Code 2 / Code 3 are 86; mapping tree nodes, tree table units, instance table units bootstrap exempt **Status: Settled.** **Progress: Implemented.** |
| 9 | **How birth sequence numbers are allocated and in what order** | Starts from 0 within tree, Code 2 and Code 3 share one counter, clears upon checkpoint switch, changes number upon rewrite; order matches bump order of commit endogenous blocks **Status: Settled.** **Progress: Implemented.** |
| 10 | **Whether mapping key is fixed-width or variable by class** | Strictly 27 bytes across the board, Code 2 / Code 3 padded with zeroes at tail; central mapping entries strictly 55 bytes across the board **Status: Settled.** **Progress: Implemented.** |
| 11 | **Pointer offset table, position entry order, where mapping tree root resides** | Segments arranged per offset table, position entries in ascending order of device identity; mapping tree root lives in root record, root record totals 457 **Status: Settled.** **Progress: Implemented.** |
| 12 | **Whether Code 2 / Code 3 mapping entries are written day-1, how many times fixed point writes** | Written day-1; an instance writes out each Code 2 node and Code 3 container only once within a checkpoint **Status: Settled.** **Progress: Implemented.** |
| 13 | **How many sections a block pointer is split into** | Split into two sections: logical properties in header, one per logical extent; physical properties in position entries, one per physical placement **Status: Settled.** **Progress: Implemented.** |
| 14 | **Whether position entries can be deduced geometrically from single origin** | Cannot, enumerated individually **Status: Settled.** **Progress: Implemented.** |
| 15 | **Value range of extent offset from origin and version 1 value** | Range [0, 32768), 15 bits sufficient; version 1 constantly 0, bits reserved as specified **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Position Entries Carry Device Identity

**Settlement**: Position entries carry device identity (dev id), one copy per physical position; no indirection layer of "logical address + chunk mapping table" is introduced.

**Scope**: Governs "whether to carry", not width (settled item 4) or ordering (settled item 11). The "per-occurrence variable" property of D2 (RAID Stripe Policy) settled item 1 is the premise for the forward-proving side of this cell, and carrying dev holds under both granularities (it is the more capable side) => granularity settlement only affects whether position entries can be compressed in expression, not whether they are carried. The viability of wide pointers is coupled with node size in D8 (core index structure): for the same 40-byte pointer header, a 4 KiB node pays +98% whereas a 16 KiB node pays only +14%; D8 (core index structure) settled item 2 selects 16 KiB and pins it as a constant. E20 (CPU cache cost of pointer widening) measured a 40-byte header, while the settlement is 50 (settled items 7 / 11); the extra bytes were measured in E128 (point lookup cost after pointer carries birth (tree, txg)) showing accounting tree point lookups slow down by 1.8%–4.5%, leaving the premise intact. The largest cost of carrying dev (removing a device requires a full run scan and rewrite of every tree with pointers) is reduced to O(references of that device) by the reverse index settled in D1 (Data Mobility / Reverse Index), a path this project was already committed to taking.

**Basis**:

- E20 (CPU cache cost of pointer widening): the cost of pointer widening appears only when the working set exceeds L3 cache, which 16 KiB nodes absorb—the direct premise for wide pointers being viable in this project, and the basis side of D8 (core index structure) settled item 2.
- Production implementations (inspected live 2026-08-28, not verified in this project): bcachefs with dev requires a full run scan and rewrite of every tree carrying pointers to remove a device (`bch2_dev_usrdata_drop()`), relying on `BTREE_ID_backpointers` to reduce it to O(references of that device); ZFS welds dev into blkptr, so device removal can only add an immortal layer of `vdev_indirect_mapping` (can only condense, cannot delete); btrfs without dev must carve out a fixed, non-expandable `sys_chunk_array[BTRFS_SYSTEM_CHUNK_ARRAY_SIZE]` (2048 bytes, definition of `BTRFS_SYSTEM_CHUNK_ARRAY_SIZE` in `include/uapi/linux/btrfs_tree.h` and this field in system configuration structure) in system configuration for bootstrap, and balance still rewrites all references.
- Three-party debate (2026-08-28): forward-proving leg and counter-proving leg reached opposite conclusions, and the fact distinguishing them was the variable granularity of D2 (RAID Stripe Policy); local leg invalidated for the round, leg circumstances and process in changelog.

**Owed**: None.

#### Settled Item 2: Ciphertext Checksum Bit-Width Takes 32 Bits

**Settlement**: Ciphertext checksum in position entries takes 32 bits, exactly the width of full CRC32C, involving no truncation; implementations are forbidden from taking the path of "truncating a wider CRC down to 32 bits".

**Scope**: Conclusion is conditional, and conditions must be cited together when referenced—it holds only under this set of inputs: 10 TB pool, 4 KiB blocks, weekly scrub, 10 years, corruption rate 1e-9 per block read, expected undetected upper bound 1e-6. Reopening condition is decidable: if field corruption rates are obtained exceeding 3.37e-9, this cell reopens on the spot. None of the candidate algorithms were eliminated (deviations were all within one order of magnitude). Media corruption is not an attacker in the threat model; this cell does not judge attack resistance.

**Basis**:

- E40 (ciphertext checksum bit-width): minimum bit-width under this input set is 31 (matching rounding of original derivation "n > 30.3"), margin for 32 bits is 3.37x—each of the five inputs (pool capacity, corruption rate, service life, scrub interval, block size) must be off by this factor for 33 bits to be needed; a truncated CRC is not a shorter CRC—full-width CRC32C having 0 undetected single-bit flips is a structural property, whereas truncation turns it into probabilistic detection.
- Collateral evidence (inspected live, not verified in this project): default checksum of btrfs is 32-bit CRC32C (`BTRFS_CSUM_TYPE_CRC32 = 0`), its 256-bit slot is reserved for optional SHA256 / BLAKE2.
- User settlement, verbatim words and that day's derivation in changelog.

**Owed**: None.

#### Settled Item 3: Fixed-Width, Slots Reserved When Unencrypted

**Settlement**: Pointers take fixed width per worst-case scenario—MAC (128 bits) and nonce (96 bits) in the header constantly occupy slots; unencrypted volumes leave them as empty slots, avoiding omission via type flags; these bits remain occupied across all volumes and are not repurposed for other fields.

**Scope**: What is optional is the AAD computation step (not computed when encryption is off), not the format (`.claude/rules/fs-design.md` "encryption is an optional computation, not an optional format"). The mechanism of opening layout branches per medium cannot govern this cell: incompat bits govern whether readers recognize this layout, while this cell governs how a pointer is encoded within the same layout; adopting fixed width first and adding a variable-length layout later is not expansion, but an extra parser plus checker plus crash-point replay that must be permanently maintained. Leaving MAC 16 bytes + nonce 12 bytes = 28 bytes / logical extent unoccupied when unencrypted represents `28 / 32768 = 0.0854%` under D4 (Checksum Location) settled item 7 measurement basis (logical extent occupies exactly 32768 bytes on disk). Variable-length saves those 28 bytes rather than saving a field (algorithm type field D9 (encryption) settled item 12 item 4 must exist regardless), while paying the cost that parsing must proceed from the beginning: fixed-width pointers can directly locate the k-th item by offset, whereas variable-length cannot; "whether scanning a partially corrupted node during recovery can skip a bad item and continue reading" is a deduction, not empirically measured. When encryption is off, MAC 16 and nonce 12 are constantly 0, readers encountering non-zero judge the structure containing the pointer corrupted, following I-2.4 (header checksum coverage) reading for unit header 29 bytes (audit A3a landed in two reader sites—multi-layer Code 2 tree internal entries, journal new root segment pointer: `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero` in `crates/singlefs-core/src/pointer.rs`); extent offset 2 bytes is constantly 0 in version 1 (version 1 pointers point to whole units), readers encountering non-zero judge the structure containing the pointer corrupted, regardless of encryption status; checker checks on every traversed pointer, core checks in every reader of on-disk pointer headers, landing in [checks-owed.md](../checks-owed.md) repaid table C581 (extent offset non-zero has no criterion) row.

**Basis**:

- No experiments: switching from fixed to variable length saves space at the expense of "whether an isolated pointer can be resolved directly by offset", which are different dimensions with no quantifiable metric. It is underpinned by two rules in `.claude/rules/fs-design.md`—"never sacrifice self-containment to save space", and the criteria for format branches (testing whether it turns "misuse" into "unmountable", without permitting "this is faster / leaner" in criteria).
- User settlement, verbatim words and reaffirmation on 2026-09-12 in changelog.
- Verdict 2026-10-01 adopted: extent offset non-zero in version 1 judges the structure containing the pointer corrupted (`research/prompts/m3-inv-a-r1-main-verification.md` section 2 line 11).

**Owed**: C210 (which category three "second sets" belong to has not been judged).

#### Settled Item 4: Location Entry Width Takes 14 Bytes

<!-- format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61 -->

**Settlement**: Device ID 4 bytes, physical offset 6 bytes (16 KiB slot number), ciphertext checksum 4 bytes => position entry 14 bytes.

| Segment | Width | Why this width |
|---|---|---|
| Device ID | **4** | Upper bound points to no settled clause (D12 (Target Media) does not cap media tiers, repository has no dev id recycling clause => narrow width caps total device identities across pool lifetime), takes wide tier per `.claude/rules/fs-design.md` "never sacrifice self-containment to save space"; precedent is settled item 3 |
| Physical offset | **6**, encoded as **16 KiB slot number** | Index nodes aligned to 16384, size is exactly 16 KiB (D18 (information carried in a block) settled item 9) => encoding as 32 KiB unit number cannot express placements of half the index nodes, while byte offset makes non-16 KiB-aligned origins representable. 6-byte slot number addresses 4 EiB / device, whereas same-width byte offset addresses only 256 TiB |
| Ciphertext checksum | **4** | Settled item 2 |

**Scope**: Physical offset shares identical coordinate system with D3 (space allocation) settled item 7 (D3 (space allocation) settled item 3 basis 2 verbatim requires identical coordinate system) => settled together, independent divergent definitions forbidden. Tree table unit pointer and root record total downstream counts adjust accordingly, and root record total is not registered here; the sole authoritative location is the field table of D22 (Synthesizing Unit Atomicity) settled item 7. Relative to legacy 11 bytes, each position entry adds 3 bytes, each pointer adds 6 bytes.

**Basis**:

- E97 (entry encoding of accounting and allocation records): criterion 7 judges placement encoding by representability—when encoded as 16 KiB slot number, all legitimate origins are representable, whereas half cannot be written under coarser unit numbers.
- User settlement, verbatim words and review of "this figure was consumed without passing through decision process" in changelog.

**Owed**: C238 (derived number "53" after position entry changed from 11 to 14 un-back-scanned).

#### Settled Item 5: Location Authority Is Hybrid, Central Mapping Is Unique Entrance

**Settlement**: Takes hybrid—

| Component | Settlement |
|---|---|
| Position entry (14 bytes in settled item 4) | **Demoted to hint**. Width and segment division unchanged; what changes is its status: reading it allows directly trying that placement, but it may be stale |
| Central mapping | **Unique entrance for dereference and release verdicts**. Relocation modifies only one entry here; "who references me" need not be known |
| State classification | Neither is item 4 on the authoritative state list: mapping can be reconstructed via unit self-describing scan, load-bearing surface remains units per D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) |

Three hard rules taking effect with settlement:

1. **Releases strictly proceed via mapping, never via hints**; after verifying through mapping to that entry, a read-disk verification against the unit checksum in the position entry is required before release. **When mismatch is verified, isolate that slot, publish succeeds normally**: logically released as usual (mapping entry removed), physical slot not returned to free pool, logged into isolation and reported via counter.
2. **Unit header or hint must carry block birth**—already satisfied: data unit class identity segment in D18 (information carried in a block) settled item 7 already contains birth txg 8 bytes.
3. **Runtime observation point required for hint staleness multi-hop rate** (taking wrong branch is a performance cliff, and cliffs do not error, `.claude/rules/fs-design.md` hard requirement 4).

**How mount state reads mapping**: Version 1 mount state reads the central mapping entirely into memory when opening the pool (`central_mapping_entries` in `crates/singlefs-core/src/mounted_read.rs`); subsequent dereferences query this in-memory copy without issuing mapping reads; when central mapping grows to multiple levels, opening the pool reads the entire mapping tree into mount state, dereferences still only query memory without issuing device reads for mapping; write side checks mapping tree capacity acceptance on each publish based on current entry count plus maximum new entries from this publish, without relying on coincidental capacity numbers from other trees. Multi-level mapping tree has no entry count ceiling; structural ceiling is only the 1-byte level in header; calculates shape in planning step before any disk writes, rejecting if level cannot hold (`HeightBeyondTheLevelField`), without establishing separate entry ceiling (implemented in sprint 19, main agent settled). The reading "one dereference with stale hint issues 3 device reads" rides on this rule, measured when root doubled as leaf, remeasured by test cases when multi-level landed (milestone "overwrite, release, rollback and reuse" closeout table line 28). Allocation record tree is likewise read entirely into mount state upon mount; extent tree is not read entirely, traversing by position when opening files (D8 (core index structure) settled item 14, user settled 2026-09-24).

**What to do when read-disk check in Hard Rule 1 fails to read or mismatches**: Before release, verify checksum by reading disk per position entry; if read fails or mismatches, reread once; only if both reads fail or both mismatch is it treated as mismatch (reread mismatch half settled by main agent 2026-09-25: transient bad read must not permanently isolate a unit; implemented in sprint 25). Different classes across the two reads (first fails to read, reread mismatches, or vice versa) treated as both mismatched, reporting the reread outcome (sprint 25 adopted, zero attack rounds). If any replica mismatches, this unit's allocation records across every disk remain in "allocated", not flipped to released (even if the replica on the other disk matched, maintaining symmetry across disks)—persists across mounts, counted as allocated in acceptance without extra formulas. Position entries pointing to a disk outside the pool, or two entries pointing to the same disk, treated as corrupted mapping entry: rejected before any write, disk unmodified. Rebuilding previous version from disk (`rebuild_version` in `crates/singlefs-core/src/recovery.rs`) copies position entries of unreadable data units without reading contents; mount proceeds normally, erroring only when reading that file, except when unwitnessed (when the selected version is not witnessed by system configuration and its data units are unreadable, writable mount falls back to the witnessed version per D23 (Role and Format of Journal) settled item 14 "when the selected root fails its walk and system configuration has not witnessed it, fall back to the witnessed version"; the fallback is judged by the union of C755 (recovery and two mount triggers fail differently on rollback), zero-round wording, implemented in the implementation batch).

**Scope**: Simultaneously resolves the list half of "where multiple referrers live under sharing" in D21 (Boundary Between Authoritative and Derived States) and the three-way position authority choice in D26 (background compaction and placement reclamation) settled item 5, all other sites linking to settled item 5; originally also closed C34 (three-way contradiction in plaintext mapping layer state), which became void on 2026-09-07 when D9 (encryption) settled item 5 switched to "scrub only", leaving the other two sites unaffected. Mapping entry field table lives in settled item 6. Three known corners lack numbers: mapping tree key layout must satisfy rider in D8 (core index structure) settled item 7 (deduplication window bounded by `T_dirty`); hint refresh policy only modeled as "refresh upon rewrite"; cache hit dimension has zero measurements, E109 (runtime cost of three position authority arms) explicitly disclaims answering. Three bootstrap exemptions in settled item 8. Read-disk verification handling details in settlement paragraph (three questions of C394 (release judgment does not verify unit checksum in mapping entry position item)); "reread once" rereads only once, without backoff or multiple retries.

**Basis**:

- E109 (runtime cost of three position authority arms): three arms measured across dimensions—unit migration cost is linear in referencing trees under pointer authority, independent under central mapping and hybrid; publish pays a fixed overhead independent of leaf count; read pays only 10%–40% extra. Absolute figures must be cited with sampling points: on primary sampling point referencing tree and mapping tree happen to share height, a degenerate point.
- E96 (consistency of hybrid architecture): resolving placement via hint causes misplacement, forcing Hard Rule 1; its `hdr_key` arm references D18 (information carried in a block) settled item 3 rather than settled item 7 field table, so "requires extra fields" format cost is non-existent.
- User settlement 2026-09-23: read disk to verify unit checksum in position entry before release; isolate slot upon mismatch, publish succeeds normally (`research/prompts/m2-placement-falsepositive-r1-main-verification.md` section 3 judging C394 (release judgment does not verify unit checksum in mapping entry position item): none of the three hard rules answered whether to verify), verbatim words in changelog.
- User settlement 2026-09-24: reading mapping entirely into mount state codified into clause per that day's implementation; multi-level mapping changed to whole-tree read by another user settlement the same day (splitting four trees in milestone 2, including mount reading multi-level mapping), landed in sprint 19 (C483 (reading mapping in mount state and tree node rollback lack clauses) ①, settled during parallel line 2 without clause), verbatim words in changelog.
- User settlement 2026-09-24: C394 (release judgment does not verify unit checksum in mapping entry position item) three questions—read disk failure rereads once before treating as mismatch, tolerate unreadable data units during rebuild, allocation records for isolated replica remain allocated (verdict `research/prompts/m2-newq-r1-main-verification.md` section 3 N1 (Reread once upon read failure)–N3 (Mismatched entries remain in allocated), verbatim words in changelog); no experiments: candidate readings came from counter-proving leg prototype on replica (`research/prompts/m2-newq-r1-opus-model/`), not registered as experiment.
- User settlement 2026-09-24: multi-level mapping read entirely into mount state upon mount, write side adds mapping tree capacity admission (verdict `research/prompts/m2-treesplit-r1-main-verification.md` T3).
- User settlement 2026-09-25: when only one replica mismatches both disks retain allocated, position entries pointing outside pool codified as reject per that day's code (sprint 16 follow-up report `m2-writepath-implementer-report.md` section 6 Q2, Q3), verbatim words in changelog; no experiments: neither alters disk format, only settles policy.
- Main agent settlement 2026-09-25: `records/2026-09-24-里程碑二收尾调度.md` section 3 row on "transient bad read causes permanent unit isolation".
- Except when unwitnessed: root ring three-party round three verdict `research/prompts/m3-close-rootring-r3-main-verification.md` section 4, the row in the table "where the ones that entered no cell went" that revisits D19 (Block Pointer Structure & Width Budget) settled item 5 (defense Z5.2: data units on an unwitnessed R are unreadable, and writable mount changes from mounting R as usual to falling back to the previous version); user settlement 2026-10-20 (C755 (recovery and two mount triggers fail differently on rollback) "add all three sentences"), verbatim wording in history. No experiment: numbers measured on the three-party legs' replicas, no experiment number assigned.

**Owed**: C248 (bootstrap exemption of tree table units unwritten), C253 (contradictory kb claims on 66 048 fixed overhead), C133 (migration forwarding information lacks carrier), C279 (scan rebuild sees both old unit and slot after packing); C417 (hint refresh policy only modeled in one variant), C418 (cache hit dimension of three position authority arms zero measurements)—cells that respective basis experiments acknowledged inability to answer; C543 (rebuilding previous version carries only sheet 0 of instance table).

#### Settled Item 6: Central Mapping Key Reuses Write Sequence Key by Class (Post-Tightening)

**Settlement**: Key is fixed-width, arranged segment by segment, definition and width matching `class_reuse_tagged` arm of E134 (mapping key format cost: slot baseline and two new arms) verbatim—

| Class | Key (fixed-width, segment by segment) | Width |
|---|---|---|
| Code 1 data unit | Class tag 1 + birth tree 8 + birth txg 8 + write sequence 10 (instance number 4 + transaction number 6) | 27 |
| Code 2 / Code 3 | Class tag 1 + birth tree 8 + birth txg 8 + instance number 4 + birth sequence number 4 | 25 |

Value follows settled items 4 / 5: position entry × w replicas (28 bytes when w = 2). Padding and total entry width belong to settled item 10.

**Four tightenings carried with it** (added during three rounds of debate per "record one loss / tighten only / declare where tightened", must still be respected after settlement):

- Class tag takes code in D18 (information carried in a block) settled item 11 registry table for the named object; objects in D27 (small-data packing container) slots are constantly Code 1; upon packing and whole-container rewriting, segments of slot key take verbatim values from pre-packing unit header.
- Birth tree takes birth tree in class identity segment (Code 3 offset 43), birth txg takes birth txg (Code 3 offset 73; Code 1 and Code 2 likewise take birth txg), instance number takes first 4 bytes of write sequence.
- D16 (publish semantics) settled item 5 closing sentence "one transaction writes at most one unit of user data" is read as covering all Code 1 units (preallocations, truncated tail units, punch hole ends, reflink copies on both sides count), preferably incorporated into that statement.
- Birth sequence number serves only as one segment of key, not for multi-version selection at fixed points—using it for selection falls into "sequence number for distinguishing stale replicas" rejected in D18 (information carried in a block) settled item 1; D18 (information carried in a block) settled item 1 owes a scope note following write sequence precedent.

**Scope**: Governs which segments comprise key and value and their widths, not padding or total entry width (settled item 10), classes contained (settled item 8), or birth sequence allocation (settled item 9). Lands in D15 (format freeze policy) Layer 2. Uniqueness of Code 1 half relies on deduction that "Code 1 write sequence is already unique version identifier", built on transaction subdivision discipline in D16 (publish semantics) settled item 5 tail (paired with D23 (Role and Format of Journal) settled item 7 one transaction number per record); that section closed review on 2026-09-13 with C123 (user settlement modified without review), subdivision discipline unchanged, deduction holds; remaining contradictions logged in C310 (transaction subdivision discipline contradicts record count basis). Recorded provisionally: chosen based on existing cost and performance numbers, reopenable upon new counter-evidence, not a proven unique solution.

**Basis**:

- E137 (performance gap of mapping key forms): gap across three arms under fsync, background load, and random reads—key reusing write sequence by class leads in most fsync cells and all random read cells, losing in global random overwrites, file deletion, and single-file snapshot deletion; user selected it after reviewing table. ⚠️ In model, physical placement equals write order, the most favorable scenario for migration and snapshot deletion; shrinkage after aging unmeasured.
- E133 (format cost of mapping key candidates): bytes paid in format by each key variant across two rounds of debate, giving pointer width, entry width, and fanout across seven candidates.
- E134 (mapping key format cost: slot baseline and two new arms): arm chosen here is its `class_reuse_tagged`, reporting 1-byte class tag and two baselines of D27 (small-data packing container) slots line by line.
- E136 (cost rows of mapping key forks): four missing cost numbers before user settlement—adding 4 bytes to Code 2 header drops zero fanout under three real header widths of D18 (information carried in a block) settled item 14, mapping tree height differs by one layer only in a narrow pool scale range.
- Verdicts of three debate rounds and process in changelog; user settlement, verbatim words in changelog.

**Owed**: C288 (consequential edits of mapping key settlement not propagated to other decisions), C289 (unspecified which field Code 2 birth tree takes), C163 (identical write sequence across fixed point rounds defeats tie-breaking), C286 (whether remount after read-only takes new instance number), C287 (switching adopts open checkpoint transactions then crashes), C310 (transaction subdivision discipline contradicts record count basis).

#### Settled Item 7: Pointer Header Carries Birth (Tree, TXG), 8 Bytes Each

**Settlement**: In addition to MAC 16 + nonce 12 + algorithm type 1 + compression algorithm code 1 + compressed length 2 + extent offset 2, pointer header carries birth tree 8 and birth txg 8, header totaling **50**; pointer remains fixed-width (settled item 3). Compression 3 bytes includes compression algorithm code 1 byte (registry table in D9 (encryption), 0 = no compression, version 1 writes 0) plus compressed length 2 bytes (version 1 writes 0, sufficient for [0, 32768) intra-unit length).

**Scope**: Settles which 16 bytes reside in pointer header, not total pointer width (settled by combining settled item 8 and settled item 11 to 88 / 86). Compression 3 bytes provides the format carrier for hard requirement in D9 (encryption) settled item 13 item 2—requiring compression/encryption order to be explicitly defined in format, where previously no field carried "this ciphertext is compressed, original length X". ⚠️ Main agent recommended omitting (compression is optional computation, and "reserve bits across all volumes" principle governs only encryption); user chose retention via popup settlement, without three-party debate, overturnable. Four consumers read these two segments from pointer: D5 (Snapshot / Space Accounting Mechanism) O(1) judgment reads birth txg; deadlist entries recorded by version identity, birth tree taken from pointer; central mapping key; AAD format requires pointer to reserve slots for these fields, values independently verified (D9 (encryption) settled item 16 explicitly forbids reading back from pointer). Alters bytes of new-pool file creation: yes—adds 3 bytes to every pointer.

**Basis**:

- E128 (point lookup cost after pointer carries birth (tree, txg)): accounting tree internal entries widened slower across four rounds, inode tree pair unstable across four rounds and unusable; measures only these 16 bytes, a lower bound on this cell's cost.
- One round of three-party debate, verdict in `research/prompts/d19-item7-r1-main-verification.md`: counter-proving leg noted closing sentence of settled item 8 left ambiguity on "whether living in pointer or referencing entry", both interpretations defensible.
- User settlement ("16 bytes live in pointer"), verbatim words and disqualification of candidates B and C in changelog.

**Owed**: C295 (whether key segments in pointer count as AAD lookup path), C296 (mapping key incomplete during journal replay), C297 (unspecified handling of two birth trees in inode internal entry), C237 (no slots for birth (tree, txg) in Code 1 / Code 2 pointers).

#### Settled Item 8: All Pointers Enter Mapping, Reference Side Carries Mapping Key Segments by Class

**Settlement**: Central mapping holds Code 1, Code 2, and Code 3 units; pointers to a unit carry mapping key segments for that class excluding class tag, class tag supplied by lookup path.

| Target Class | Pointer Carries (key segments from settled item 6) | Pointer Width |
|---|---|---|
| Code 1 | Birth tree 8 + birth txg 8 + write sequence 10 | Data pointer **88** |
| Code 2 / Code 3 | Birth tree 8 + birth txg 8 + instance number 4 + birth sequence number 4 | Node pointer **86** |

**Three classes excluded from mapping (bootstrap exemption)**: Mapping tree's own nodes, tree table units, instance table units. Position entries in their parent pointers (mapping tree root, tree table pointer and instance table pointer in root record) are authoritative, not hints; settled item 5 "unique entrance" and Hard Rule 1 do not apply to these three; all three are pool-level singletons with exactly one referrer. Parent pointers of these three share 86 width with node pointers to Code 2 / Code 3: position entries 14 × 2 are authoritative, instance number 4 + birth sequence number 4 populated from target unit header without querying mapping; parser determines pointer width by target unit class, not by whether target is an exempt class. For non-exempt units (Code 1 data units and Code 2 tree nodes—extent tree root, inode tree root, inode leaf container), unreadable position hints trigger fallback through mapping; exempt three classes (mapping tree root, tree table, root record chain) do not fall back.

**Scope**: Decides which classes mapping holds and which key segments pointers carry, invariant to pointer width (where birth (tree, txg) 16 bytes living in pointer header was settled by settled item 7). Holding Code 1 only loses on two correctness points: Code 2 / Code 3 blocks likewise enter deadlist / livelist, and D5 (Snapshot / Space Accounting Mechanism) settles deadlist entries recorded by version identity and released via mapping; without mapping they must be recorded by placement, causing release at stale placements after compaction or drive-loss reconstruction; migrating a Code 2 / Code 3 referenced by read-only snapshot requires rewriting snapshot tree ancestors, taking new birth violates reference interval while keeping old birth violates `birth` definition. All pointers entering mapping avoids both losses: deadlist / livelist recorded by mapping key, migration alters only one mapping entry, migrated unit remains byte-for-byte unchanged with birth unchanged. Costs: mapping blocks per unit during fsync and background exceed Code 1-only by 10%–20%+, random read misses also rise; column undercounts two items (insert/delete of derived tree Code 2 in mapping, extent nodes with stale hints reading via mapping). Parent node rewrites populate position hints with mapping placements (C417 (hint refresh policy only modeled in one variant) version 1 takes "refresh upon rewrite"). Checker and GPU judge treat position entries of non-exempt Code 1 / 2 / 3 as hints: traversal attempts hint first, reads mapping placement if both mismatch, records mapping placement in reference set.

**Basis**:

- E137 (performance gap of mapping key forms): measured cost gaps between two configurations (Code 1 only / all pointers enter mapping) cell by cell, cost row of this cell taken directly from it.
- E134 (mapping key format cost: slot baseline and two new arms): pointer width, entry width, and fanout across configurations provided by it; 88 / 86 derived from its two rows plus 3 compression bytes from settled item 7.
- E128 (point lookup cost after pointer carries birth (tree, txg)): point lookup cost omitted on settlement day and supplemented next day, recorded in cost row of this cell.
- Two rounds of three-party debate, verdicts in `research/prompts/d19-item8-r1-main-verification.md` and `research/prompts/d19-item8-r2-main-verification.md` (first round cloud legs reached opposite conclusions, user requested rerun, stances reversed in second round).
- User settlement, verbatim words and reasons why both remedy paths required modifying settled clauses in changelog.
- Verdict in `m2-lines234-code-r1-main-verification.md` 2026-09-23 on whether tree nodes fall back via mapping: counter-proving leg tested same corruption where data unit read back correct bytes via extra hop, whereas extent tree root made `open_pool_for_read` unmountable for entire pool.
- Verdict `research/prompts/m4-gc-r1-main-verification.md` section 3 G3: write path, reconstruction, and checker refactored per "mapping is unique entrance" (Option B), parent rewrites write mapping placement, checker and GPU judge treat position entries as hints only. Section 7 user settlement "if inappropriate, illogical, or unnatural, refactor", verbatim words in changelog.

**Owed**: C293 (two items undercounted by E137 after all pointers enter mapping), C292 (rationale for zero consumers of Code 2 identity segment no longer holds), C248 (bootstrap exemption of tree table units unwritten) (merged from C280 (which classes central mapping holds lacks clauses)).

#### Settled Item 9: How Birth Sequence Numbers Are Allocated and in What Order

**Settlement**: Birth sequence numbers start from 0; increments by 1 for each Code 2 or Code 3 unit written within same tree (both classes share one counter); clears when scope shifts to next checkpoint; same node rewritten second time by fixed point within same checkpoint takes new number. Allocation sequence matches bump sequence of commit endogenous blocks from open cluster segment—ascending by tree ID, within tree leaves before root, key-ascending within same level, central mapping tree penultimate, tree table unit last.

**Scope**: Sole registration site for sequence itself is D3 (space allocation) settled item 10, no second copy stored here. Birth sequence number is a segment of central mapping key (settled item 6); key must be determined before unit is written, binding allocation to placement allocation sequence: single cursor in implementation, single sequence checked by checker. Serves only as segment of key, not for multi-version selection at fixed points (fourth tightening of settled item 6). User popup settlement, without three-party debate, overturnable; main agent recommended this tier. Alters bytes of new-pool file creation: yes—placements and birth sequence numbers of eleven commit endogenous blocks (default geometry; 13 blocks when ring is 1024 MiB) in new-pool file creation determined by it, per-block values in D3 (space allocation) settled item 10 ⑤ table.

**Basis**:

- No experiments: sequence and initial values have no quantifiable metric, cost of sharing rule between numbering and placement lives on D3 (space allocation) settled item 10 side; cell supported by deduction "key must be determined before unit is written" plus tightening arguments (same container receiving birth sequence 0 in two checkpoints buys collision-free runtime mapping) holding only under rewrite number change.
- User settlement, verbatim words in changelog.

**Owed**: C291 (allocation rules for birth sequence numbers exist only in experiment source).

#### Settled Item 10: Mapping Key Strictly 27 Bytes, Entries Strictly 55 Bytes

**Settlement**: Central mapping key is **strictly 27 bytes across the board**—Code 1 (class tag 1, birth tree 8, birth txg 8, write sequence 10) is naturally 27; Code 2 / Code 3 (class tag 1, birth tree 8, birth txg 8, instance number 4, birth sequence number 4) is 25 bytes, **zero-padded to 27 at tail**. Central mapping entries are thus **strictly 55 bytes across the board** (key 27 + value 28, value = position entry 14 × 2). Key range in mapping tree node headers is likewise stored at 27.

**Scope**: Two related sites return to fixed-width with it—sparse side table entries in D5 (Snapshot / Space Accounting Mechanism) settled item 6 fixed to 31 (mapping key 27 + count 4), livelist entries in D6 (snapshot implementation model) settled item 2 fixed to 36 (head tree ID 8 + mapping key 27 + event type 1). Varying length by class would violate two universal rules in D8 (core index structure) settled item 11: "fixed-width entries arranged contiguously" and "declared length = entry count × entry width"—entry width in Code 2 header has only one value, so two widths require a sentinel, eliminating intra-node fixed-width binary search and forcing sequential scan from start to resolve a mapping node during scan. Cost is 2 extra bytes per Code 2 / Code 3 entry, 10 extra bytes in new-pool file creation. User popup settlement, without three-party debate, overturnable; main agent recommended this tier. Alters bytes of new-pool file creation: yes.

**Basis**:

- E145 (cost of Code 2 self-describing header and mapping tree key width): measured single-width arm; "two widths 55 / 53, self-delimited by first key byte" was an unmeasured fourth form in that round, taking it as settlement would cite a number with no artifact.
- User settlement, verbatim words in changelog.

**Owed**: None (C321 (Code 2 header entry width is single value while mapping tree entries vary by class) repaid 2026-09-14 with user settlement returning mapping key to fixed-width 27, see owed check repaid table).

#### Settled Item 11: Pointer Offset Table, Position Entry Ordering, Where Mapping Tree Root Resides

**Settlement**: Pointer segments arranged from offset 0—

| Segment | Offset | Width |
|---|---|---|
| MAC | 0 | 16 |
| nonce | 16 | 12 |
| Algorithm type | 28 | 1 |
| Compression algorithm code | 29 | 1 |
| Compressed length | 30 | 2 |
| Extent offset from origin | 32 | 2 |
| Birth tree | 34 | 8 |
| Birth txg | 42 | 8 |
| Position entries × 2, ascending by device identity | 50 | 14 × 2 |
| Key tail: Code 1 write sequence 10; Code 2 / Code 3 instance number 4 + birth sequence number 4 | 78 | 10 / 8 |

=> Pointers to Code 1 are **88**, pointers to Code 2 / Code 3 are **86**. Position entries sorted ascending by device identity, established as I-2.5 (position entries ascending by device identity). **Central mapping tree root lives in root record**: root record carries an 86-byte central mapping tree root pointer, not entering tree table; root record totals **457** bytes. Tree table version 1 has **12** entries.

**Scope**: Settles only ordering—set of segments and widths settled in settled items 3 / 4 / 7 / 8. Compression 3 bytes immediately follows algorithm type, belonging to the group "how this ciphertext was produced"; placed before extent offset to keep four encryption and compression fields contiguous. Sole authoritative location for root record total is field table of D22 (Synthesizing Unit Atomicity) settled item 7; tree table version 1 count follows D5 (Snapshot / Space Accounting Mechanism) settled item 11 adding deadlist. User popup settlement, without three-party debate, overturnable. Alters bytes of new-pool file creation: yes—segment offsets and widths of every pointer, and root record total.

**Basis**:

- No experiments: offset table has no second choice, segments and widths already settled, cell only orders them with no quantifiable metric. Ascending device identity establishes deterministic order for two position entries, enabling invariant.
- User settlement, verbatim words in changelog.

**Owed**: None (C304 (instance table chain pointer record cannot fit 83-wide pointer) repaid 2026-10-11, see owed check repaid table).

#### Settled Item 12: Code 2 / Code 3 Mapping Entries Written Day-1, Fixed-Point Writes Once per Checkpoint

**Settlement**: Maintain day-1 write-out (6 entries in new-pool file creation). Fixed point codified: each Code 2 node and Code 3 container written only once per checkpoint by an instance—one-shot COW at end of publish; version 1 forbids concurrent insertion with COW (implementation permitted by D3 (space allocation) settled item 5 not adopted in version 1).

**Scope**: ⚠️ When concurrent insertion with COW is permitted later, D8 (core index structure) settled item 13 must still be respected: same key lands in leaf only once per publish based on final state. Failing checks counted per unit, not per fixed-point round. Cell settles whether to write and write frequency, not key form (settled item 6) or entry width (settled item 10). When settled, repository lacked release and reuse experiments, leaving other path as "revisit once release experiments exist", an uncodified trigger condition.

**Basis**:

- E81 (commit fixed point): origin of reading "dirty nodes COW only once per publish", and basis for fixed point convergence under this reading; applying it to Code 2 and Code 3 eliminates multi-version same-key loss within same checkpoint.
- User settlement, verbatim words in changelog.

**Owed**: C352 (settlement of writing mapping entries day-1 lacks awakeners for reopening conditions), C163 (identical write sequence across fixed point rounds defeats tie-breaking).

#### Settled Item 13: Block Pointer Split into Two Sections

**Settlement**: **Block pointer is split into two sections: one "header" following the logical extent, and N "position entries" following physical placements.**
Logical properties (MAC, nonce, algorithm type, extent offset from origin) in header, one copy per logical extent;
Physical properties (placement, and ciphertext checksum for unkeyed corruption detection) in position entries, one copy per physical position.

**Scope**: Splitting demotes "total pointer width" from a hard constraint to a budget, rewriting D4 (Checksum Location) into a two-section form. Settles only which class of properties lives in which section, not specific fields (settled items 4 / 7) or order (settled item 11); birth tree and birth txg later added to header (settled item 7) remain logical properties. "Moving out of pointer" does not equal an extra I/O: D4 (Checksum Location) requires parent node to store child checksum, constraining which node it lives in, not which struct.

**Basis**:

- D2 (RAID Stripe Policy) settled "multi-replica writes identical ciphertext N times" => all replicas must share same nonce; storing per pointer duplicates same value N times, and writing one replica wrong yields two incompatible decryption results. Testable form is I-6.7 (replicas share nonce).
- Production implementations exhibit same shape (inspected live 2026-08-28, not verified in this project): bcachefs `struct bch_extent_ptr` is exactly 64 bits (`offset:44, dev:8, gen:8` plus four flags), checksum / nonce / compression info moved to independent variable-length entries (`bch_extent_crc32/64/128`) within same bkey, one crc entry applying contiguously to all following ptr entries; nonce itself unpersisted, `extent_nonce()` in `fs/bcachefs/checksum.h` computed live from `bversion` (96 bits, in bkey header) plus small per-crc offset (`crc64.nonce:10` / `crc128.nonce:13`).
- No experiments: two-section split is a format architectural trade-off, underpinned by settled rule (replicas share nonce) and production shapes, with no quantifiable metric.

**Owed**: None.

#### Settled Item 14: Position Entries Listed Individually, Cannot Be Computed from Single Origin via Geometric Formula

**Settlement**: N position entries listed individually, each carrying device identity; not deduced from single origin plus size for remaining columns.

**Scope**: Governs expression of position entries, not constituent segments (settled item 4). Prerequisite is D2 (RAID Stripe Policy) settled item 1 defining "variable width" as **per-occurrence variable**—listing physical placements individually with device identities and avoiding chunk mapping tables is the exact premise required by this deduction. Isomorphic to D2 (RAID Stripe Policy) settled item 1: buys "no pointer to wrong disk", not "failure domain independence".

**Basis**:

- ZFS can deduce all RAID-Z column positions from "one DVA + asize" (`BP_GET_ASIZE` summing asize of 3 DVAs), predicated on top-level vdev child count being **fixed at pool creation** (inspected live, not verified in this project); D2 (RAID Stripe Policy) settled item 1 negated this premise.
- No experiments: deducibility is directly determined by stripe width granularity, a deduction with no quantifiable metric.

**Owed**: None.

#### Settled Item 15: Value Range of Extent Offset from Origin and Version 1 Value

**Settlement**: Value range of extent offset from origin is capped by **maximum extent size**, not file size; range in this project is [0, 32768), **15 bits sufficient**. Version 1 field is constantly 0—splitting is a reserved feature, disabled in version 1; bits reserved per settled item 3 "fixed-width reserved slots", justified by policy rather than usage.

**Scope**: Addresses value range; width of 2 bytes lives in settled item 11 offset table. D9 (encryption) settled item 12 item 2 estimated 32–64 bits based on file size, an overestimate. Basis for narrower range in this project is D4 (Checksum Location) settled item 7: extent on disk strictly = unit = 32768.

**Basis**:

- Production inspection (inspected live 2026-08-28 in `fs/bcachefs/extents_format.h`, not verified in this project): `crc32.offset:7` / `crc64.offset:9` / `crc128.offset:13`, matching `CRC128_SIZE_MAX (1U << 13)` => value range capped by maximum extent size.
- No experiments: value range is arithmetic dividing two settled clauses, with no quantifiable metric; version 1 disablement settled by D4 (Checksum Location) settled item 7.

**Owed**: None.

```

**出处 `.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md:311-321`（整段抄，未转述）**

```markdown
### Chain Reactions with Other Decisions

| Decision | Chain point |
|---|---|
| D4 (Checksum Location) | Settled item 13 demotes "total pointer width" from constraint to budget, rewriting D4 (Checksum Location) into two-section form; logical extent strictly occupying 32768 given by D4 (Checksum Location) settled item 7, relied upon by settled items 3 and 15 |
| D2 (RAID Stripe Policy) | Prerequisite for settled items 1 and 14 is variable stripe width, settled as per-occurrence variable by D2 (RAID Stripe Policy) settled item 1 |
| D9 (encryption) | Nonce carrier settled half by settled item 13 (in header, one per logical extent); registry tables for algorithm type and compression live in D9 (encryption), expected AAD cannot be read back from pointer (D9 (encryption) settled item 16) |
| D12 (Target Media) | Mount comparison table in D12 (Target Media) settled item 4 rejects mount on device identity mismatch, based on position entries carrying dev in settled item 1. ⚠️ Criterion requires "whether **address arithmetic** consuming it carries device identity", btrfs address arithmetic includes device identity in chunk tree; criterion holds, leap "carried => must be in pointer" falls |
| D15 (format freeze policy) | Pointer field layout is Layer 2 of D15 (format freeze policy) settled item 7; central mapping entry field table in settled item 6 also lands in Layer 2 (explicitly cited in basis of D15 (format freeze policy) settled item 1) |
| D8 (core index structure) | Pointer width enters internal entries and tree table entries, several widths in D8 (core index structure) settled items 6 / 8 adjust with settled items 7 / 8; viability of wide pointers couples with 16 KiB nodes in D8 (core index structure) settled item 2 |

```

**出处 `.claude/kb/decisions/19-block-pointer-structure-width-budget.en.md:322-325`（整段抄，未转述）**

```markdown
## Historical Versions

Historical entries for D19 (Block Pointer Structure & Width Budget) are centralized in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md:1-13`（整段抄，未转述）**

```markdown
## D19 (ブロックポインタ構造と幅予算) — 確定

[中文](19-块指针的结构与宽度预算.md) · [English](19-block-pointer-structure-width-budget.en.md) · **日本語**

D19 (ブロックポインタ構造と幅予算) はブロックポインタのセルを管轄する：ポインタがいくつのセグメントに分かれるか、各セグメントにどのようなフィールドが含まれ、それぞれ幅はいくらか、どのような順序で並ぶか；位置エントリと中央マッピングのどちらがデリファレンスおよび解放判定の権威であるか；中央マッピングエントリの key と value をどうエンコードするか；マッピング key のどのセグメントがポインタに追従し、誕生シーケンス番号をどう発行するか。対象外とする事項：ユニットヘッダのフィールド表とユニット種別コードは D18 (ブロックに含める情報)；ノードサイズとツリー内エントリレイアウトは D8 (コアインデックス構造)；配置先の座標系と割り当てレコードの key は D3 (領域割り当て) 確定項目 3 / 7、コミット内生ブロックのバンプ順序は D3 (領域割り当て) 確定項目 10；チェックサムのカバー範囲、論理 extent とユニットの関係は D4 (チェックサム位置)；暗号化アルゴリズム、nonce の生成方法、AAD 期待値の出所は D9 (暗号化)；ルートレコードとシステム構成のフィールド表は D22 (ユニット原子性の合成手法) 確定項目 7 / 9；ストライプ幅の粒度は D2 (RAID ストライプポリシー) 確定項目 1 に属する。

<!-- doc-lint:registry name-col=2 -->

| 番号 | 略称 |
|---|---|
| N1 | 読み取れない場合はまず1回再読み取り |
| N3 | 一致しないものは割り当て済みに維持 |

```

**出处 `.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md:14-310`（整段抄，未转述）**

```markdown
### 確定項目

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **位置エントリにデバイス識別子を含めるか** | 含める：各物理位置にそれぞれ dev id を保持、chunk マッピングテーブルの階層を挟まない **状態：確定。** **進捗：実装済み。** |
| 2 | **暗号文チェックサムのビット幅** | 32 ビット；完全な CRC32C を採用、より広い CRC からの切り捨ては禁止 **状態：確定。** **進捗：実装済み。** |
| 3 | **固定幅か可変長か** | 固定幅、非暗号化時もスロットを確保：MAC と nonce は恒常的にスロットを占有し、空きスロットとして維持し他用途への転用を禁止 **状態：確定。** **進捗：実装済み。** |
| 4 | **位置エントリ各セグメントの幅** | デバイス ID 4 バイト、物理オフセット 6 バイト（16 KiB スロット番号）、暗号文チェックサム 4 バイト => 位置エントリ 14 バイト **状態：確定。** **進捗：実装済み。** |
| 5 | **位置の権威はどこにあるか** | 位置エントリはヒントに降格、中央マッピングがデリファレンスと解放判定の唯一の入口 **状態：確定。** **進捗：未実装。** |
| 6 | **中央マッピングエントリのフィールド表** | key は種別ごと書き込みシーケンスキー再利用（厳格化後）：Code 1 は 27 バイト、Code 2 / Code 3 は 25 バイト；value は確定項目 4 / 5 に準拠 **状態：確定。** **進捗：実装済み。** |
| 7 | **ポインタヘッダにどのセグメントを含めるか** | ヘッダ 50：MAC 16、nonce 12、アルゴリズム種別 1、圧縮アルゴリズムコード 1、圧縮後長 2、extent オフセット 2、誕生ツリー 8、誕生 txg 8 **状態：確定。** **進捗：実装済み。** |
| 8 | **参照側が担持する誕生識別子の量、マッピングが収容する種別** | 全ポインタがマッピング経由、Code 1 へのポインタは 88、Code 2 / Code 3 へのポインタは 86；マッピングツリーノード、ツリーテーブルユニット、インスタンス表ユニットはブートストラップ免除 **状態：確定。** **進捗：実装済み。** |
| 9 | **誕生シーケンス番号の発行方法と順序** | ツリー内で 0 から開始、Code 2 と Code 3 は単一カウンタを共有、チェックポイント切り替え時にゼロクリア、再書き込み時に番号変更；順序はコミット内生ブロックのバンプ順序と同一 **状態：確定。** **進捗：実装済み。** |
| 10 | **マッピング key は固定幅か種別ごと可変長か** | 一律 27 バイト、Code 2 / Code 3 は末尾をゼロパディング；中央マッピングエントリは一律 55 バイト **状態：確定。** **進捗：実装済み。** |
| 11 | **ポインタのオフセット表、位置エントリの順序、マッピングツリー根の配置場所** | セグメントはオフセット表順に配置、位置エントリはデバイス識別子昇順；マッピングツリー根はルートレコードに配置、ルートレコード合計 457 **状態：確定。** **進捗：実装済み。** |
| 12 | **Code 2 / Code 3 のマッピングエントリは day-1 で書き込むか、固定点は何回書き込むか** | day-1 で書き出し；1つのインスタンスは1つのチェックポイント内で各 Code 2 ノードおよび Code 3 コンテナを1回のみ書き出し **状態：確定。** **進捗：実装済み。** |
| 13 | **ブロックポインタは何セグメントに分割されるか** | 2セグメントに分割：論理属性はヘッダに配置し論理 extent ごとに1つ、物理属性は位置エントリに配置し物理配置先ごとに1つ **状態：確定。** **進捗：実装済み。** |
| 14 | **位置エントリは単一起点から幾何学的計算で導出可能か** | 導出不能、個別に列挙 **状態：確定。** **進捗：実装済み。** |
| 15 | **起点からの extent オフセットの値域と第1版の値** | 値域は [0, 32768)、15 ビットで十分；第1版は恒常的に 0、ビットは規定通り確保 **状態：確定。** **進捗：実装済み。** |

#### 確定項目 1：位置エントリにデバイス識別子を含める

**確定内容**：位置エントリにデバイス識別子（dev id）を含め、各物理位置にそれぞれ1つ保持；「論理アドレス + chunk マッピングテーブル」という間接層は導入しない。

**射程**：管轄するのは「含めるか否か」であり、幅（確定項目 4）と順序（確定項目 11）は対象外。D2 (RAID ストライプポリシー) 確定項目 1 の「都度変更可能」が本セルの順方向推論側の前提であり、dev を含める設計は両方の粒度で成立する（能力がより高い側である）=> 粒度の決定は位置エントリを圧縮表現できるか否かにのみ影響し、含めるか否かには影響しない。幅広ポインタが成立するか否かは D8 (コアインデックス構造) のノードサイズと結合している：同一の 40 バイトポインタヘッダに対し、4 KiB ノードは +98% のコストを払うが、16 KiB ノードは +14% のみであり、D8 (コアインデックス構造) 確定項目 2 は 16 KiB を選択し定数として固定している。E20 (ポインタ肥大化の CPU キャッシュコスト) は 40 バイトのヘッダを測定したが、確定内容は 50（確定項目 7 / 11）である；余分なバイト数について E128 (ポインタへの誕生 (ツリー, txg) 付加後のポイントルックアップコスト) では会計ツリーのポイントルックアップが 1.8%–4.5% 低速化することを測定したが、前提は覆っていない。dev を含める最大のコスト（デバイス削除時にポインタを持つ全ツリーをフル実行走査 (フルスキャン)・再書き込みする必要がある）は D1 (データ移動性 / 逆引きインデックス) で確定済みの逆引きインデックスにより O(当該デバイスの参照数) に削減され、その経路は本プロジェクトがそもそも進むべき道である。

**根拠**：

- E20 (ポインタ肥大化の CPU キャッシュコスト)：ポインタ肥大化のコストはワーキングセットが L3 キャッシュを超過した場合にのみ現れ、16 KiB ノードはこれを吸収する——幅広ポインタが本プロジェクトで成立する直接の前提であり、D8 (コアインデックス構造) 確定項目 2 の根拠側でもある。
- 現役実装（2026-08-28 実地調査、本プロジェクトでの検証は未実施）：dev を持つ bcachefs はデバイス削除時にポインタを持つ全ツリーをフル実行走査 (フルスキャン)・再書き込みする必要があり（`bch2_dev_usrdata_drop()`）、`BTREE_ID_backpointers` により初めて O(当該デバイスの参照数) に低減する。ZFS は dev を blkptr に固定化しており、デバイス削除は消えることのない `vdev_indirect_mapping` 層を追加するしかない（condense のみ可能で削除不能）。dev を持たない btrfs はマッピングテーブルのブートストラップのためにシステム構成内に固定長・拡張不能な `sys_chunk_array[BTRFS_SYSTEM_CHUNK_ARRAY_SIZE]`（2048 バイト、`include/uapi/linux/btrfs_tree.h` 内の `BTRFS_SYSTEM_CHUNK_ARRAY_SIZE` 定義とシステム構成構造体内の同フィールド）を確保せねばならず、balance は同様に全参照を再書き込みする。
- 三者論証（2026-08-28）：順推論側と反駁側で正反対の結論に至ったが、両者を識別する事実は D2 (RAID ストライプポリシー) の可変粒度であった；ローカル側担当はそのラウンド全体が無効、担当状況と経緯は変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 2：暗号文チェックサムは 32 ビットを採用

**確定内容**：位置エントリ内の暗号文チェックサムは 32 ビットを採用し、完全な CRC32C の幅そのものであり、切り捨ては伴わない；実装において「より広い CRC を 32 ビットに切り詰める」手法を採ることは禁止される。

**射程**：結論は条件付きであり、参照時は条件も同時に引用しなければならない——本件は以下の入力セットにおいてのみ成立する：10 TB プール、4 KiB ブロック、週次 scrub、10 年、読み取り 1 ブロックあたりの破損率 1e-9、検出漏れの期待値上限 1e-6。再開条件は判定可能：現場の破損率を取得しそれが 3.37e-9 を超えた場合、本セルは即座に再開される。候補アルゴリズムは1つも脱落していない（偏りはすべて1桁以内）。脅威モデルにおいてメディア破損は攻撃者ではなく、本セルは耐攻撃性を判定しない。

**根拠**：

- E40 (暗号文チェックサムビット幅)：この入力セット下での最小ビット幅は 31（元の演繹文「n > 30.3」の丸めと一致）、32 ビットのマージンは 3.37 倍——5つの入力（プール容量、破損率、稼働年数、scrub 周期、ブロックサイズ）のそれぞれがこれだけの倍率狂って初めて 33 ビットが必要となる；切り捨てられた CRC は短い CRC ではない——フル幅の CRC32C が単一ビット反転に対して検出漏れ 0 であることは構造的性質であり、切り捨て後は確率的検出に変質する。
- 傍証（実地調査、本プロジェクトでの検証は未実施）：btrfs のデフォルトチェックサムはまさに 32 ビット CRC32C（`BTRFS_CSUM_TYPE_CRC32 = 0`）であり、その 256 ビットスロットは任意の SHA256 / BLAKE2 のために残されている。
- ユーザー確定、原文および当日の導出は変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 3：固定幅、非暗号化時もスロットを確保

**確定内容**：ポインタは最悪ケースに基づき固定幅とする——ヘッダ内の MAC（128 ビット）と nonce（96 ビット）は恒常的にスロットを占有し、暗号化を行わないボリュームではこれらを空きスロットとして維持し、種別フラグによる省略は行わない；これらのビットはすべてのボリュームで占有され続け、他のフィールドに転用されることもない。

**射程**：選択可能（オプショナル）なのは AAD の演算処理（暗号化オフなら計算しない）であり、フォーマットではない（`.claude/rules/fs-design.md`「暗号化は選択可能な演算であり、選択可能なフォーマットではない」）。メディアごとのレイアウト分岐開放メカニズムはこのセルを管轄できない：incompat ビットが管轄するのはリーダーがそのレイアウトを認識できるか否かであり、本セルが管轄するのは同一レイアウト内で単一のポインタ内部をどうエンコードするかである；先行して固定幅とし、後から可変長レイアウトを追加することは拡張ではなく、永久保守すべきパーサー、checker、クラッシュポイント再生をもう1組増やすことに他ならない。非暗号化時に MAC 16 バイト + nonce 12 バイト = 28 バイト / 論理 extent を空費することは、D4 (チェックサム位置) 確定項目 7 の計測条件（論理 extent のディスク占有は恒常的に 32768 バイト）に基づき `28 / 32768 = 0.0854%` である。可変長側が削減するのはその 28 バイトであり、1フィールドを削減することではない（アルゴリズム種別フィールド D9 (暗号化) 確定項目 12 第4項はもともと必要）；払うべき代償はパースが先頭から順次実行されねばならないことである：固定幅ポインタはオフセットにより直接 k 番目を特定できるが、可変長では不可能；「リカバリ走査時に部分破損したノードで破損した1件をスキップして読み取りを続行できるか否か」は推論であり、実測は未実施。暗号化オフ時は MAC 16、nonce 12 は恒常的に 0、リーダーが非ゼロに遭遇した場合は当該ポインタが存在する構造を破損と判定、I-2.4 (ヘッダチェックサムカバー範囲) がユニットヘッダ 29 バイトに与えた解釈と同一（実地審査 A3a は2箇所のリーダーに着地——多層 Code 2 ツリー内部エントリ、journal 新ルートセグメントポインタ：`crates/singlefs-core/src/pointer.rs` の `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent オフセットの 2 バイトは第1版で恒常的に 0（第1版ポインタはユニット全体を指す）、リーダーが非ゼロに遭遇した場合は当該ポインタが存在する構造を破損と判定、暗号化のオンオフとは無関係；checker は辿る全ポインタ上で判定し、core はディスク上のポインタヘッダを読む全リーダーで判定、着地点は [checks-owed.md](../checks-owed.md) 返済済み表の C581 (extent オフセット非ゼロに判定基準なし) 行にある。

**根拠**：

- 実験なし：固定幅を可変長に変更して節約できるのは空間であり、失われるのは「単一のポインタを単独で取り出してオフセットから直接解決できるか否か」であり、両者は異なる次元であり定量可能な数値はない。これを支えるのは `.claude/rules/fs-design.md` の2つの原則——「省スペースのために自己完結性を犠牲にしない」、およびフォーマット分岐の判定基準（「誤用」を「マウント不可」に変換できるかを問い、判定基準に「高速・省スペース」を含めることを禁じる）である。
- ユーザー確定、原文および 2026-09-12 の再確認は変更履歴に記録。
- 判定 2026-10-01 採択：第1版で extent オフセットが非ゼロの場合、当該ポインタを含む構造を破損と判定（`research/prompts/m3-inv-a-r1-main-verification.md` 第2節第11行）。

**未済の検査**：C210 (3つの「第2セット」がどの拡張分類に属するか未判定)。

#### 確定項目 4：位置エントリは 14 バイト

<!-- format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61 -->

**確定内容**：デバイス ID 4 バイト、物理オフセット 6 バイト（16 KiB スロット番号）、暗号文チェックサム 4 バイト => 位置エントリ 14 バイト。

| セグメント | 幅 | なぜこの幅か |
|---|---|---|
| デバイス ID | **4** | 上限は確定済み条項のいずれにも依存しない（D12 (対象メディア) はメディア層に上限を設けず、全リポジトリに dev id 回収条項なし => 狭い幅はプール生涯に出現したデバイス識別子の総数を制限する）、`.claude/rules/fs-design.md` の「省スペースのために自己完結性を犠牲にしない」に基づき広い方を選択；先例は確定項目 3 |
| 物理オフセット | **6**、**16 KiB スロット番号**としてエンコード | インデックスノードは 16384 アライメント、サイズは 16 KiB（D18 (ブロックに含める情報) 確定項目 9）=> 32 KiB ユニット番号エンコードでは半数のインデックスノードの配置先を表現不能、バイトオフセットでは 16 KiB 非アライメントの起点が表現可能になってしまう。6 バイトスロット番号は 4 EiB / デバイスをアドレス指定可能、同幅のバイトオフセットは 256 TiB のみ |
| 暗号文チェックサム | **4** | 確定項目 2 |

**射程**：物理オフセットと D3 (領域割り当て) 確定項目 7 は同一の座標系である（D3 (領域割り当て) 確定項目 3 根拠第2条が同一座標系を逐字要求）=> 両者は同時に確定し、個別の決定を禁じる。ツリーテーブルユニットポインタとルートレコード合計の2つの下流数値はこれに伴い変更されるが、ルートレコード合計はここには登録されず、唯一の権威ある箇所は D22 (ユニット原子性の合成手法) 確定項目 7 のフィールド表である。旧来の 11 バイトに対し、各位置エントリは 3 バイト増加、各ポインタは 6 バイト増加。

**根拠**：

- E97 (会計および割り当てレコードのエントリエンコード)：判定基準 7 は表現可能性により配置先エンコードを判定——16 KiB スロット番号エンコード時は全正当起点が表現可能、粗いユニット番号では半分が書き込み不能。
- ユーザー確定、原文および「この数値はこれまで決定プロセスを経ずに消費されていた」再確認は変更履歴に記録。

**未済の検査**：C238 (位置エントリが 11 から 14 へ変更後の派生値「53」の未バック走査)。

#### 確定項目 5：位置の権威はハイブリッド、中央マッピングが唯一の入口

**確定内容**：ハイブリッドを採用——

| コンポーネント | 確定内容 |
|---|---|
| 位置エントリ（確定項目 4 の 14 バイト） | **ヒントに降格**。幅とセグメント区分は不変、変わるのはその地位：読み取った場合は直接その配置先を試行可能だが、陳腐化している可能性がある |
| 中央マッピング | **デリファレンスと解放判定の唯一の入口**。移動時はここの1エントリのみを変更、「誰が私を参照しているか」を知る必要は一切ない |
| 状態分類 | いずれも権威態リストの第4項目ではない：マッピングはユニット自己記述走査から再構築可能であり、荷重受け面は依然として D20 (荷重受け面: ユニットの原子性と自己完結性) のユニットである |

確定に伴い有効化された3つの厳格な規則：

1. **解放は一律マッピングを経由し、ヒントを経由しない**；マッピング経由で当該マッピングエントリを検証した後、解放前に位置エントリ内のユニットチェックサムに基づきディスク読み取り検証を行う。**不一致を検出した場合はそのスロットを隔離し、パブリッシュは正常に成立**：論理的には通常通り解放（マッピングエントリを削除）、物理スロットは空きプールに返却せず隔離として記録しカウンタで報告。
2. **ユニットヘッダまたはヒントはブロック birth を保持しなければならない**——既に充足：D18 (ブロックに含める情報) 確定項目 7 のデータユニット種別識別セグメントに誕生代号 8 バイトがもともと含まれる。
3. **ヒント陳腐化によるマルチホップ率の実行時観測点が必要**（誤った分岐への進入は性能の崖であり、崖はエラーを出さない、`.claude/rules/fs-design.md` 厳格要件 4）。

**マウント状態におけるマッピングの読み取り方法**：第1版のマウント状態ではプールオープン時に中央マッピング全体をマウント状態へ読み込み（`crates/singlefs-core/src/mounted_read.rs` の `central_mapping_entries`）、以降のデリファレンスはこのメモリ内エントリを参照しマッピング読み取りは発行しない；中央マッピングが多層に成長した場合、プールオープン時にマッピングツリー全体をマウント状態へ読み込み、デリファレンスは従前通りメモリのみを参照しマッピングのためのデバイス読み取りは発行しない；書き込み側はパブリッシュごとに現在のマッピングツリーエントリ数と今回のパブリッシュで最大追加されるエントリ数に基づきマッピングツリー容量受入を行い、他ツリーの容量数値の偶然には依存しない。多層化後マッピングツリーにエントリ数上限はなく、構造的上限はヘッダの階層 1 バイトのみである；計画段階・ディスク書き込み前に今回実際に書き込むエントリに基づき形状を算出し、階層に収まらない場合は拒否（`HeightBeyondTheLevelField`）、別途エントリ数上限は設けない（スプリント 19 採用、メイン agent 確定）。「ヒント陳腐化による1回のデリファレンスが3回のデバイス読み取りを発行する」という数値はこの規則に準拠しており、根が葉を兼ねていた際に測定されたものであり、多層化着地時にテストケースで再測定された（マイルストーン「上書き・解放・ロールバック・再利用」クローズアウト表第28行）。割り当てレコードツリーもマウント時に同様にツリー全体をマウント状態へ読み込む；extent ツリーは全体読み込みを行わず、ファイルオープン時に位置に従い辿る（D8 (コアインデックス構造) 確定項目 14、ユーザー 2026-09-24 確定）。

**厳格規則 1 のディスク読み取り検証が読み取れない・不一致の場合の処置**：解放前に位置エントリに基づきディスク読み取りチェックサム検証を行い、読み取り自体の失敗または不一致の場合、まず1回再読み取りを行い、2回とも読み取れないまたは2回とも不一致の場合にのみ不一致として処置（再読み取り不一致の側はメイン agent 2026-09-25 確定：一時的な読み取り不良でユニットを永久隔離してはならない；スプリント 25 で実装済み）。2回の読み取り結果の種別が異なる場合（先が読み取り不可で再読み取りが不一致、またはその逆）は2回とも不一致として処置し、再読み取り時の様態を報告（スプリント 25 採用、攻撃ラウンドゼロ）。いずれかのレプリカで不一致が検出された場合、各ディスク上の当該ユニットの割り当てレコードは「割り当て済み」のまま維持され解放済みに変更されない（もう一方のディスク上のレプリカが一致していても維持され、各ディスクの台帳の対称性を保つ）——ディスク書き込みは再マウントを跨ぎ、受入において割り当て済みとして計上され別式には入らない。位置エントリがプール外のディスクを指している、または2つの位置エントリが同一ディスクを指している場合はマッピングエントリ破損として処置：いかなる書き込み前にも拒否し、ディスク上は不変。ディスクから直前版を再構築する際（`crates/singlefs-core/src/recovery.rs` の `rebuild_version`）、読み取れないデータユニットはその位置エントリをそのままコピーし内容は読まない、マウントは通常通り進行し、当該ファイルを読んだ時点でエラーとなる。ただし目撃がない場合は除く（選択されたバージョンがシステム構成に目撃されておらず、そのデータユニットが読み取れない場合、書き込み可能マウントは D23 (ジャーナルの役割と形式) 確定項目 14「選択されたルートの走査読み出しが失敗し、システム構成がそれを目撃していない場合は目撃されたバージョンへ戻る」に従い目撃されたバージョンへ戻る；C755 (リカバリと二種のマウントでロールバックを引き起こす失敗が異なる) の和集合でロールバックを判定、攻撃ゼロ回の文言、実装は実装バッチで行う）。

**射程**：D21 (権威態と派生態の境界) の「共有下で複数参照者がどこに存在するか」のリスト半分と、D26 (バックグラウンド整理と配置回収) 確定項目 5 の位置権威の3択を同時に収束させ、他箇所はすべて確定項目 5 にリンクする；元々は C34 (平文マッピング層の状態区分の3箇所での衝突) も収束させていたが、C34 (平文マッピング層の状態区分の3箇所での衝突) は 2026-09-07 に D9 (暗号化) 確定項目 5 が「scrub のみ」に方針変更したことで対象外となり、その収束は失われたが他の2箇所には影響しない。マッピングエントリのフィールド表は本セルにはなく確定項目 6 に属する。既知の3つのコーナーケースに番号はない：マッピングツリーの key レイアウトは D8 (コアインデックス構造) 確定項目 7 の付帯条件（重複排除ウィンドウが `T_dirty` で境界づけられる）を満たさねばならない；ヒントのリフレッシュ方針は「再書き込み時リフレッシュ」のみモデル化済み；キャッシュヒットの次元は測定値ゼロであり、E109 (位置権威三系統の実行時コスト) は回答不能と明記している。ブートストラップ免除の3種は確定項目 8 参照。厳格規則 1 のディスク読み取り検証が読み取れない・不一致時の処置は確定内容落着段落参照（C394 (解放判定がマッピングエントリ位置項内のユニットチェックサムを検証しない) の3問）；「まず1回再読み取り」は1回のみ再読み取りを行い、バックオフや多重リトライは行わない。

**根拠**：

- E109 (位置権威三系統の実行時コスト)：3系統を次元ごとに実測——ユニット移動コストはポインタ権威下では参照ツリー数に線形、中央マッピングおよびハイブリッド下では無関係；パブリッシュは葉の数に無関係な一定額を支払う；読み取りは1割から4割の増加にとどまる。絶対値はサンプリング点とともに引用が必要：主サンプリング点では参照ツリーとマッピングツリーが同一高であり、縮退点である。
- E96 (ハイブリッドアーキテクチャの整合性)：ヒントに基づき配置先を解決すると誤解放を引き起こすため、厳格規則 1 が必然的に導出された；その `hdr_key` 側は D18 (ブロックに含める情報) 確定項目 7 ではなく確定項目 3 のフィールド表を参照しており、「フィールド追加を要する」フォーマットコストは存在しない。
- ユーザー確定 2026-09-23：解放前にディスクを読み取り位置項内のユニットチェックサムを検証、不一致時はスロットを隔離しパブリッシュは成立（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` 第3節 C394 (解放判定がマッピングエントリ位置項内のユニットチェックサムを検証しない) 判定セル：3つの厳格規則はいずれも検証要否に答えていなかった）、原文は変更履歴に記録。
- ユーザー確定 2026-09-24：マウント状態でのマッピング全体読み取りは当日の実装に準拠して条項化、多層マッピングはユーザーの同日別確定（マイルストーン2で多層マッピング読み取りを含む4ツリーの分割を実施）により全体読み取りに変更、スプリント 19 着地（C483 (マウント状態のマッピング読み取りとツリーノードロールバックに条項がない) ①、並行ライン2着地時に確定・条項なし）、原文は変更履歴に記録。
- ユーザー確定 2026-09-24：C394 (解放判定がマッピングエントリ位置項内のユニットチェックサムを検証しない) 3問——ディスク読み取り失敗時はまず1回再読み取り後に不一致処置、再構築時は読み取れないデータユニットを許容、隔離された側の割り当てレコードは割り当て済みに維持（判定 `research/prompts/m2-newq-r1-main-verification.md` 第3節 N1 (読み取れない場合はまず1回再読み取り)–N3 (一致しないものは割り当て済みに維持)、原文は変更履歴に記録）；実験なし：候補の読み取り数値は三者攻撃側担当のレプリカ上プロトタイプ（`research/prompts/m2-newq-r1-opus-model/`）由来であり、実験未登録。
- ユーザー確定 2026-09-24：多層マッピングはマウント時に全体をマウント状態へ読み込み、書き込み側にマッピングツリー容量受入を追加（三者 `research/prompts/m2-treesplit-r1-main-verification.md` T3）。
- ユーザー確定 2026-09-25：1レプリカのみ不一致時は両ディスクとも維持、位置項がプール外ディスクを指す場合は当日の拒否挙動を条項化（スプリント 16 継続レポート `m2-writepath-implementer-report.md` 第6節 Q2、Q3）、原文は変更履歴に記録；実験なし：いずれもディスク上フォーマットを変更せず処置のみを規定。
- メイン agent 確定 2026-09-25：`records/2026-09-24-里程碑二收尾调度.md` 第3節「一時的読み取り不良によるユニット永久隔離」行。
- 目撃がない場合は除く：ルートリング三方第 3 ラウンド判定 `research/prompts/m3-close-rootring-r3-main-verification.md` 第 4 節「どのセルにも入らなかったものの行き先」表で D19 (ブロックポインタ構造と幅予算) 確定項目 5 を見直したその行（防御側 Z5.2：目撃のない R 上のデータユニットが読み取れず、書き込み可能マウントは R を通常通りマウントする方式から前の版へ戻る方式に変わる）；ユーザー確定 2026-10-20（C755 (リカバリと二種のマウントでロールバックを引き起こす失敗が異なる)「三文とも補う」）、原文は変更履歴にある。実験なし：数値は三方レグのレプリカで計測、実験番号なし。

**未済の検査**：C248 (ツリーテーブルユニットのブートストラップ免除未記載)、C253 (66 048 の固定オーバーヘッドに関する2つの kb の矛盾言及)、C133 (マイグレーション転送情報にキャリアなし)、C279 (パッキング後にスキャン再構築が旧ユニットとスロットを同時に視認)；C417 (位置ヒントのリフレッシュ方針は1種のみモデル化)、C418 (位置権威三系統のキャッシュヒット次元測定値ゼロ)——根拠の2実験が回答不能と自認したセル；C543 (直前版再構築がインスタンス表第0シートのみ保持)。

#### 確定項目 6：中央マッピング key は種別ごと書き込みシーケンスキー再利用（厳格化後）を採用

**確定内容**：key は固定幅、セグメントごとに配置、定義と幅は E134 (マッピング key フォーマットコスト・スロットベースラインと2つの新系統) の `class_reuse_tagged` 系統と逐字同一——

| 種別 | key（固定幅、セグメントごと） | 幅 |
|---|---|---|
| Code 1 データユニット | 種別タグ 1 + 誕生ツリー 8 + 誕生 txg 8 + 書き込みシーケンス 10（インスタンス番号 4 + トランザクション番号 6） | 27 |
| Code 2 / Code 3 | 種別タグ 1 + 誕生ツリー 8 + 誕生 txg 8 + インスタンス番号 4 + 誕生シーケンス番号 4 | 25 |

value は確定項目 4 / 確定項目 5 に準拠：位置エントリ × w レプリカ（w = 2 時は 28 バイト）。パディングとエントリ総幅は確定項目 10 に属する。

**伴う4つの厳格化**（3ラウンドの論争で「1敗を記録 / 厳格化のみ許可 / 厳格化箇所を明記」に基づき追加、確定後も同様に遵守）：

- 種別タグは命名対象オブジェクトの D18 (ブロックに含める情報) 確定項目 11 登録表におけるコードを採用、D27 (小データパッキングコンテナ) スロット内のオブジェクトは恒常的に Code 1；パッキングおよびコンテナ全体再書き込み時、スロット内 key の各セグメントはパッキング前の版のユニットヘッダの値をそのまま採用。
- 誕生ツリーは種別識別セグメントの誕生ツリーを採用（Code 3 オフセット 43）、誕生 txg は誕生代号を採用（Code 3 オフセット 73；Code 1、Code 2 も同様に誕生代号を採用）、インスタンス番号は書き込みシーケンスの先頭 4 バイトを採用。
- D16 (パブリッシュセマンティクス) 確定項目 5 末尾の「1トランザクションは最大1ユニットのユーザーデータを書き込む」は全 Code 1 ユニットをカバーするものとして解釈（事前割り当て、切り捨てられた末尾ユニット、ホールパンチ両端、reflink 両端のコピーも含む）、該当文への明記が望ましい。
- 誕生シーケンス番号は key の1セグメントとしてのみ機能し、固定点多版の新旧選択には用いない——新旧選択に用いると D18 (ブロックに含める情報) 確定項目 1 で否決された「陳腐化レプリカ識別のためのシーケンス番号」に該当してしまう；D18 (ブロックに含める情報) 確定項目 1 は書き込みシーケンスの先例に倣い射程注記を追記する必要がある。

**射程**：管轄するのは key と value の構成セグメントと各幅であり、パディングとエントリ総幅（確定項目 10）、収容種別（確定項目 8）、誕生シーケンス番号の発行方法（確定項目 9）は対象外。D15 (フォーマット凍結ポリシー) 第2層に着地。Code 1 側の唯一性は「Code 1 の書き込みシーケンスは既に唯一のバージョン識別子である」という推論に依存しており、これは D16 (パブリッシュセマンティクス) 確定項目 5 末尾のトランザクション分割規律に基づく（D23 (ジャーナルの役割と形式) 確定項目 7 の1レコード1トランザクション番号と協調）；該当箇所は 2026-09-13 に C123 (ユーザー確定変更後の未レビュー) の再確認により収束、分割規律は不変、推論は成立、残余の基準衝突は C310 (トランザクション分割規律とレコード数基準の衝突) に記録。「暫定」記録：本セルは既存のコストと性能数値に基づき選定されたものであり、新たな反証により覆された場合は再開され、論証された唯一解ではない。

**根拠**：

- E137 (マッピング key 形態の性能ギャップ)：fsync ごと、バックグラウンド負荷、ランダム読み取りの3種負荷における3系統の格差——種別ごと書き込みシーケンスキー再利用は fsync ごとの多数のセルとランダム読み取りの全セルでリード、全域ランダム上書き、ファイル削除、単一ファイルスナップショット削除で劣後；ユーザーは本表を精査しこれを採択。⚠️ モデル内の物理配置先は書き込み順序そのものであり、マイグレーションやスナップショット削除に最も有利な状況、老朽化後の縮退は未測定。
- E133 (マッピング key 各候補のフォーマットコスト)：2ラウンドの三者論証に出現した各 key がフォーマット上で支払うバイト数、7候補のポインタ幅、エントリ幅、ファンアウトを提示。
- E134 (マッピング key フォーマットコスト・スロットベースラインと2つの新系統)：本セルが採択した系統はその `class_reuse_tagged` であり、厳格化で追加された 1 バイト種別タグと D27 (小データパッキングコンテナ) スロットの2種のベースラインを行単位で報告。
- E136 (マッピング key 分岐のコスト行)：ユーザー確定前に欠落していた4つのコスト数値——Code 2 ヘッダへの 4 バイト追加は D18 (ブロックに含める情報) 確定項目 14 の3段階の現実のヘッダ幅下でファンアウトを1段階も低下させず、マッピングツリー層数は狭いプール規模区間でのみ1層異なるにとどまる。
- 3ラウンドの論争判定と3ラウンド論証の経緯は変更履歴に記録；ユーザー確定、原文は変更履歴に記録。

**未済の検査**：C288 (マッピング key 確定の連随変更が他決定に未反映)、C289 (Code 2 の誕生ツリーがどのフィールドを採用するか未記載)、C163 (固定点各巡で書き込みシーケンス同一によるタイブレーク失敗)、C286 (読み取り専用後 remount で新インスタンス番号を取得するか否か)、C287 (切り替えでオープンなチェックポイントのトランザクション養子縁組後にクラッシュ)、C310 (トランザクション分割規律とレコード数基準の衝突)。

#### 確定項目 7：ポインタヘッダは誕生 (ツリー, txg) を保持、各 8 バイト

**確定内容**：ポインタヘッダは MAC 16 + nonce 12 + アルゴリズム種別 1 + 圧縮アルゴリズムコード 1 + 圧縮後長 2 + extent オフセット 2 の後に誕生ツリー 8 と誕生 txg 8 を保持、ヘッダ合計 **50**；ポインタは従前通り固定幅（確定項目 3）。圧縮の 3 バイトは圧縮アルゴリズムコード 1 バイト（登録表は D9 (暗号化) に配置、0 = 非圧縮、第1版は 0 を記述）と圧縮後長 2 バイト（第1版は 0 を記述、[0, 32768) のユニット内長さを表現可能）である。

**射程**：定めるのはどの 16 バイトがポインタヘッダに存在するかであり、ポインタ総幅（確定項目 8 と確定項目 11 の合成による 88 / 86）は規定しない。圧縮の 3 バイトは D9 (暗号化) 確定項目 13 第2条の厳格要件のフォーマットキャリアを補填する——圧縮と暗号化の順序がフォーマット上明確に定義されることを要求するが、それまでリポジトリには「この暗号文は圧縮済みか、圧縮前長はいくらか」を担持するフィールドが皆無であった。⚠️ メイン agent は非保持を推奨したが（圧縮はオプショナルな演算であり、「暗号化のために確保されたスロットは全ボリュームで保持」の原則は暗号化のみを対象とする）、ユーザーはポップアップ確定で保持を選択、三者論証は経ておらず、覆すことは可能。ポインタからこの2セグメントを取得するコンシューマは4つ：D5 (スナップショット / 領域会計メカニズム) の O(1) 判定が誕生 txg を読み取る；deadlist エントリはバージョン識別子で記録され誕生ツリーをポインタから取得；中央マッピング key；AAD 形態はポインタがこれら2フィールドのスロットを持つことを要求し値は独立検証を経る（D9 (暗号化) 確定項目 16 はポインタからの読み戻しを逐字禁止）。新規プールでのファイル作成のバイト列を変更：はい——全ポインタに 3 バイト追加。

**根拠**：

- E128 (ポインタへの誕生 (ツリー, txg) 付加後のポイントルックアップコスト)：会計ツリー内部エントリ拡幅後は4ラウンドとも低速化、inode ツリーのペアは4ラウンド不安定で引用不可；測定対象はこの 16 バイトのみであり、本セルのコストの下限である。
- 1ラウンドの三者論証、判定は `research/prompts/d19-item7-r1-main-verification.md`：反駁側担当が確定項目 8 当時の末文が「ポインタ内に保持するか参照エントリ内に保持するか」に余地を残しており双方の解釈が可能と指摘。
- ユーザー確定（「16 バイトはポインタ内に保持」）、原文および候補乙・丙の脱落経緯は変更履歴に記録。

**未済の検査**：C295 (ポインタ内の key セグメントが AAD のルックアップパスに含まれるか否か)、C296 (journal リプレイ時にマッピング key が揃わない)、C297 (inode 内部エントリ内の誕生ツリー重複の処置未定)、C237 (Code 1 / Code 2 のポインタ内に誕生 (ツリー, txg) のスロットがない)。

#### 確定項目 8：全ポインタがマッピング経由、参照側は種別ごとにマッピング key セグメントを担持

**確定内容**：中央マッピングは Code 1、Code 2、Code 3 の3種別ユニットを収容；ユニットを指すポインタは当該種別マッピング key の種別タグを除く各セグメントを担持、種別タグはルックアップパスから与えられる。

| 指される種別 | ポインタが担持（確定項目 6 の key 各セグメント） | ポインタ幅 |
|---|---|---|
| Code 1 | 誕生ツリー 8 + 誕生 txg 8 + 書き込みシーケンス 10 | データポインタ **88** |
| Code 2 / Code 3 | 誕生ツリー 8 + 誕生 txg 8 + インスタンス番号 4 + 誕生シーケンス番号 4 | ノードポインタ **86** |

**マッピング対象外の3種別（ブートストラップ免除）**：マッピングツリー自身のノード、ツリーテーブルユニット、インスタンス表ユニット。それらの親ポインタ（マッピングツリー根、ルートレコード内のツリーテーブルポインタおよびインスタンス表ポインタ）内の位置エントリは権威でありヒントではない、確定項目 5 の「唯一の入口」および厳格規則 1 はこれら3種別には適用されない；3種別ともプール単位の単一インスタンスであり、参照者は1つのみ。これら3種別の親ポインタは Code 2 / Code 3 を指すノードポインタと同幅の 86：位置エントリ 14 × 2 は権威、インスタンス番号 4 + 誕生シーケンス番号 4 は被参照ユニットヘッダの値をそのまま設定しマッピング検索には用いない；パーサーは被参照ユニットの種別によりポインタ幅を決定し、相手が免除種別か否かでは区別しない。免除3種別以外のユニット（Code 1 データユニットおよび Code 2 ツリーノード——extent ツリー根、inode ツリー根、inode 葉コンテナ）は位置ヒントが読めない場合は一律マッピング経由でフォールバック；免除3種別（マッピングツリー根、ツリーテーブル、ルートレコードの連鎖）はフォールバックしない。

**射程**：判定するのはマッピングがどの種別を収容しポインタが key のどのセグメントを担持するかであり、ポインタ幅には依存しない（うち誕生 (ツリー, txg) 16 バイトがポインタヘッダに存在することは確定項目 7 で確定）。Code 1 のみ収容する系統は2つの正確性において劣後：Code 2 / Code 3 のブロックも同様に deadlist / livelist に入り、D5 (スナップショット / 領域会計メカニズム) は deadlist エントリがバージョン識別子で記録されマッピング経由で解放されると規定、マッピングに入らなければ配置先で記録するしかなく、整理やドライブ脱落再構築で移動された後に旧配置先で解放されてしまう；読み取り専用スナップショットから参照される Code 2 / Code 3 を移動する場合、スナップショットツリーの祖先を書き換える必要があり、新 birth を取れば参照区間に違反、旧 birth を維持すれば `birth` の定義文に違反する。全ポインタがマッピング経由であればこれら双方で劣後しない：deadlist / livelist はマッピング key で記録、移動時はマッピングの1エントリのみ変更、移動されたユニットは1バイトも変化せず birth も不変。代償：fsync ごとおよびバックグラウンドのユニットあたりマッピングブロック数は Code 1 のみ収容時より1〜2割以上増加、ランダム読み取りミスも増加；本列はさらに2項目を過小評価（派生ツリーの Code 2 のマッピング内削除・挿入、extent ノード自身のヒント陳腐化によるマッピング読み取り）。親ノード再書き込み時は位置ヒントにマッピング位置を書き込む（C417 (位置ヒントのリフレッシュ方針は1種のみモデル化) 第1版は「再書き込み時リフレッシュ」を採用）。checker と GPU 判定器は免除3種別以外の Code 1 / 2 / 3 位置エントリをヒントとしてのみ扱う：走査時はまずヒントに従い読み取り、両方不一致の場合はマッピング位置に従い読み取り、参照集合にはマッピング位置を記録。

**根拠**：

- E137 (マッピング key 形態の性能ギャップ)：2つの構成（Code 1 のみ / 全ポインタがマッピング経由）のコスト差をセルごとに実測、本セルのコスト行全体はここから引用。
- E134 (マッピング key フォーマットコスト・スロットベースラインと2つの新系統)：両構成下のポインタ幅、エントリ幅、ファンアウトを提示、88 / 86 はその2行に確定項目 7 の圧縮 3 バイトを加えたもの。
- E128 (ポインタへの誕生 (ツリー, txg) 付加後のポイントルックアップコスト)：確定当日に記録漏れし翌日補填されたポイントルックアップコスト、本セルのコスト行に記録。
- 2ラウンドの三者論証、判定は `research/prompts/d19-item8-r1-main-verification.md` および `research/prompts/d19-item8-r2-main-verification.md`（第1ラウンドは2つのクラウド側担当が正反対の結論、ユーザーが再推論を要求、第2ラウンドで立場逆転）。
- ユーザー確定、原文および両救済策がなぜ確定済み条項群の変更を要したかは変更履歴に記録。
- 2026-09-23 判定 `m2-lines234-code-r1-main-verification.md` ツリーノードがマッピング経由でフォールバックすべきか否かの判定セル——攻撃側担当が同一の損傷下でデータユニットが1ホップ多く費やして正しいバイト列を読み戻せたのに対し、extent ツリー根が `open_pool_for_read` でプール全体をマウント不能にしたことを実測。
- 判定 `research/prompts/m4-gc-r1-main-verification.md` 第3節 G3：書き込みパス、再構築、checker を「マッピングが唯一の入口」に従いリファクタリング（乙を採用）、親ノード再書き込み時はマッピング位置を記述、checker と GPU 判定器は位置エントリをヒントとしてのみ扱う。第7節ユーザー確定「不適切、論理が通らない、または不自然であればリファクタリングせよ」、原文は変更履歴に記録。

**未済の検査**：C293 (全ポインタがマッピング経由化後に E137 が過小評価した2項目)、C292 (Code 2 識別セグメントのコンシューマゼロの根拠喪失)、C248 (ツリーテーブルユニットのブートストラップ免除未記載)（C280 (中央マッピングがどの種別ユニットを収容するか条項なし) より統合）。

#### 確定項目 9：誕生シーケンス番号の発行方法と順序

**確定内容**：誕生シーケンス番号は 0 から開始；同一ツリー内で Code 2 または Code 3 ユニットを1つ書き出すごとに 1 加算（両種別で単一カウンタを共有）；スコープが次のチェックポイントに移った際にゼロクリア；同一チェックポイント内で同一ノードが固定点により2回目に再書き込みされた場合は新番号に変更。発番順序はコミット内生ブロックがオープンなクラスタセグメントからバンプされる順序と同一——ツリー ID 昇順、ツリー内は葉が先で根が後、同一層内は key 昇順、中央マッピングツリーが最後から2番目、ツリーテーブルユニットが最後。

**射程**：順序自体の唯一の登録箇所は D3 (領域割り当て) 確定項目 10 であり、ここには重複して保持しない。誕生シーケンス番号は中央マッピング key の1セグメントであり（確定項目 6）、key はユニットが書き出される前に確定していなければならないため、発番は配置先の割り当て順序に束縛される：実装におけるカーソルは1つのみ、checker が検証する順序も1組のみ。key の1セグメントとしてのみ機能し、固定点多版の新旧選択には用いない（確定項目 6 第4の厳格化）。ユーザーポップアップ確定、三者論証は経ておらず、覆すことは可能；メイン agent の推奨も本層である。新規プールでのファイル作成のバイト列を変更：はい——新規プールでのファイル作成で書き出される11個のコミット内生ブロック（デフォルト幾何；幾何により個数は変動、リング 1024 MiB 時は 13 個）の配置先と誕生シーケンス番号はこれに準拠して決定、各ブロックの値は D3 (領域割り当て) 確定項目 10 ⑤ の表を参照。

**根拠**：

- 実験なし：順序と初期値自体に定量可能な数値はなく、発番と配置先が単一ルールを共有することのコストは D3 (領域割り当て) 確定項目 10 側にある；本セルを支えるのは「key はユニットが書き出される前に確定していなければならない」推論に加え、確定項目 6 厳格化論証における文言（同一コンテナが2つのチェックポイントでそれぞれ誕生シーケンス番号 0 を取得し、余分に得られるのは実行時マッピングの非衝突のみ）が再書き込み時改番下でのみ成立することである。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：C291 (誕生シーケンス番号の割り当て規則が実験ソースにのみ存在)。

#### 確定項目 10：マッピング key は一律 27、エントリは一律 55

**確定内容**：中央マッピングの key は**一律 27 バイト**——Code 1 の (種別タグ 1, 誕生ツリー 8, 誕生 txg 8, 書き込みシーケンス 10) は自然に 27；Code 2 / Code 3 の (種別タグ 1, 誕生ツリー 8, 誕生 txg 8, インスタンス番号 4, 誕生シーケンス番号 4) は 25 バイトであり、**末尾を 27 にゼロパディング**。したがって中央マッピングエントリは**一律 55 バイト**（key 27 + value 28、value = 位置エントリ 14 × 2）。マッピングツリーノードヘッダの key 区間も同様に 27 で保持。

**射程**：連動する2箇所がこれに伴い固定幅に復帰——D5 (スナップショット / 領域会計メカニズム) 確定項目 6 の疎なサイドテーブルエントリは固定幅 31（マッピング key 27 + カウント 4）、D6 (スナップショット実現モデル) 確定項目 2 の livelist エントリは固定幅 36（ヘッドツリー ID 8 + マッピング key 27 + イベント種別 1）。種別ごとに可変長にすると D8 (コアインデックス構造) 確定項目 11 の2つの汎用ルールが同時に崩壊する：「固定幅エントリの連続配置」と「宣言長 = エントリ数 × エントリ幅」——Code 2 のエントリ幅フィールドは単一の値しか持てず、2幅にするには番兵が必要となり、番兵を設けるとノード内の固定幅二分探索が失われ、スキャン走査時にマッピングノードを先頭から順次解決せねばならなくなる。代償は Code 2 / Code 3 エントリごとに 2 バイト増加、新規プールでのファイル作成で 10 バイト増加。ユーザーポップアップ確定、三者論証は経ておらず、覆すことは可能；メイン agent の推奨も本層である。新規プールでのファイル作成のバイト列を変更：はい。

**根拠**：

- E145 (Code 2 自己記述ヘッダとマッピングツリー key 幅のコスト)：単一幅の系統を測定済み；「2幅 55 / 53、key 先頭バイトによる自己境界画定」はそのラウンドで未測定の第4の形態であり、それを確定内容とすることは成果物のない数値を引用することに等しい。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：なし（C321 (Code 2 ヘッダのエントリ幅が単一値であるのに対しマッピングツリーエントリが種別ごと2幅) は 2026-09-14 にユーザー確定でマッピング key が固定幅 27 に復帰したことで返済済み、返済済み表参照）。

#### 確定項目 11：ポインタのオフセット表、位置エントリの順序、マッピングツリー根の配置場所

**確定内容**：ポインタはオフセット 0 からセグメント順に配置——

| セグメント | オフセット | 幅 |
|---|---|---|
| MAC | 0 | 16 |
| nonce | 16 | 12 |
| アルゴリズム種別 | 28 | 1 |
| 圧縮アルゴリズムコード | 29 | 1 |
| 圧縮後長 | 30 | 2 |
| 起点からの extent オフセット | 32 | 2 |
| 誕生ツリー | 34 | 8 |
| 誕生 txg | 42 | 8 |
| 位置エントリ × 2、デバイス識別子昇順 | 50 | 14 × 2 |
| key 末尾セグメント：Code 1 書き込みシーケンス 10；Code 2 / Code 3 インスタンス番号 4 + 誕生シーケンス番号 4 | 78 | 10 / 8 |

=> Code 1 へのポインタは **88**、Code 2 / Code 3 へのポインタは **86**。位置エントリはデバイス識別子昇順でソート、I-2.5 (位置エントリのデバイス識別子昇順) として確立。**中央マッピングツリーの根はルートレコードに配置**：ルートレコードは 86 バイトの中央マッピングツリー根ポインタを保持し、ツリーテーブルには入らない；ルートレコード合計 **457** バイト。ツリーテーブル第1版は **12** エントリ。

**射程**：定めるのは順序のみ——セグメントの集合と幅は確定項目 3 / 4 / 7 / 8 で確定済み。圧縮の 3 バイトはアルゴリズム種別の直後に続き、「この暗号文がどう生成されたか」のグループに属する；extent オフセットの前に配置するのは暗号化と圧縮の4フィールドを一連のセグメントとするためである。ルートレコード合計の唯一の権威ある箇所は D22 (ユニット原子性の合成手法) 確定項目 7 のフィールド表、ツリーテーブル第1版の該当エントリは D5 (スナップショット / 領域会計メカニズム) 確定項目 11 の deadlist 追加に伴う。ユーザーポップアップ確定、三者論証は経ておらず、覆すことは可能。新規プールでのファイル作成のバイト列を変更：はい——全ポインタのセグメントオフセットと幅、ならびにルートレコードの合計。

**根拠**：

- 実験なし：オフセット表自体に第2の選択肢はなく、セグメントの集合と幅は確定済みであり、本セルは順序を整えるのみで定量可能な数値はない。位置エントリのデバイス識別子昇順は2つの位置エントリに確定的順序を与え不変条件として確立するためである。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：なし（C304 (インスタンス表チェーンポインタレコードが幅 83 のポインタを収容できない) は 2026-10-11 に返済済み、返済済み表参照）。

#### 確定項目 12：Code 2 / Code 3 のマッピングエントリは day-1 で書き出し、固定点はチェックポイントあたり1回書き出し

**確定内容**：day-1 での書き出しを維持（新規プールでのファイル作成で 6 エントリ）。固定点を固定化：各 Code 2 ノードおよび Code 3 コンテナは、1つのインスタンスが1つのチェックポイント内で1回のみ書き出し——パブリッシュ末尾に一括 COW；第1版は挿入中並行 COW を禁止（D3 (領域割り当て) 確定項目 5 が許容した実装を第1版では不採用）。

**射程**：⚠️ 将来挿入中並行 COW が開放された際も、D8 (コアインデックス構造) 確定項目 13 は従前通り遵守されねばならない：同一パブリッシュ内で同一 key は最終状態に基づき葉に1回のみ着地。失敗検査はユニット単位で計上し、固定点ラウンド数では計上しない。本セルが決定するのは書き込み要否と書き込み頻度であり、key 形態（確定項目 6）やエントリ幅（確定項目 10）は規定しない。確定時にリポジトリには解放と再利用の実験が存在せず、もう一方の経路は「解放の実験が整ってから再考」として残されたが、そのトリガー条件は本セルに記述されていない。

**根拠**：

- E81 (コミットの固定点)：「ダーティノードは今回のパブリッシュで1回のみ COW」という解釈の出所であり、この解釈下で固定点が収束する根拠；これに従い Code 2 と Code 3 を保護することで、同一チェックポイント内での同 key 2版による劣後は再発しなくなる。
- ユーザー確定、原文は変更履歴に記録。

**未済の検査**：C352 (day-1 マッピングエントリ書き込み確定の再開条件にリマインダーなし)、C163 (固定点各巡で書き込みシーケンス同一によるタイブレーク失敗)。

#### 確定項目 13：ブロックポインタは2セグメントに分割

**確定内容**：**ブロックポインタは2セグメントに分割：1つの「ヘッダ」が論理 extent に追従し、N 個の「位置エントリ」が物理配置先に追従する。**
論理属性（MAC、nonce、アルゴリズム種別、起点からの extent オフセット）はヘッダにあり、論理 extent ごとに1つ；
物理属性（配置先、および鍵を持たない側が破損検出するための暗号文チェックサム）は位置エントリにあり、物理位置ごとに1つ。

**射程**：分割により「ポインタ総幅」が制約から予算へと降格され、D4 (チェックサム位置) のセルは2セグメント形態へと書き改められた。定めるのはどの属性がどのセグメントに属するかのみであり、各セグメントに含まれるフィールド（確定項目 4 / 7）や順序（確定項目 11）は規定しない；ヘッダに後から追加された誕生ツリーと誕生 txg（確定項目 7）も論理属性である。「ポインタ外への移動」は余分な I/O を意味しない：D4 (チェックサム位置) が要求するのは親ノードが子ノードチェックサムを保持することであり、どの構造体に住むかではなく、どのノードに住むかを制約している。

**根拠**：

- D2 (RAID ストライプポリシー) が「複数レプリカは同一暗号文を N 回書き込む」と確定 => 全レプリカは同一 nonce を共有せねばならない；ポインタごとに各自保持することは同一数値を N 回複製することに等しく、あるレプリカが誤記された場合に同一暗号文から相互に相容れない2つの復号結果が生じる。判定可能形式は I-6.7 (レプリカ間での nonce 共有)。
- 現役実装が同一の形状を示す（2026-08-28 実地調査により逐字確認、本プロジェクトでの検証は未実施）：bcachefs の `struct bch_extent_ptr` はちょうど 64 ビット（`offset:44, dev:8, gen:8` プラス4つのフラグ）、チェックサム / nonce / 圧縮情報は同一 bkey 内の独立した可変長エントリ（`bch_extent_crc32/64/128`）に移動され、1つの crc エントリがそれ以降の全 ptr エントリに連続して適用；nonce 自体は永続化されず、`fs/bcachefs/checksum.h` の `extent_nonce()` は `bversion`（96 ビット、もともと bkey ヘッダに存在）に crc ごとの小さなオフセット（`crc64.nonce:10` / `crc128.nonce:13`）を加えて即時計算。
- 実験なし：2セグメントへの分割はフォーマット構造のトレードオフであり、確定済みルール（レプリカ間での nonce 共有）と現役実装の形状に支えられており、定量可能な数値はない。

**未済の検査**：なし。

#### 確定項目 14：位置エントリは個別に列挙、単一起点から幾何学的計算で導出しない

**確定内容**：N 個の位置エントリを個別に列挙し、それぞれデバイス識別子を保持；単一起点にサイズを加算して残りの各列位置を導出することはしない。

**射程**：管轄するのは位置エントリの表現方法であり、含まれるセグメント（確定項目 4）は対象外。前提となる基準は D2 (RAID ストライプポリシー) 確定項目 1 が「可変幅」の粒度を**都度変更可能**と定めたことである——フォーマット側で物理位置を個別に列挙し、それぞれデバイス識別子を保持し、chunk マッピングテーブルを挟まないことは、まさにこの演繹が必要とする前提である。D2 (RAID ストライプポリシー) 確定項目 1 と同型：得られるのは「誤ったディスクを指さないこと」であり、「障害ドメインの独立性」ではない。

**根拠**：

- ZFS は「1つの DVA + asize」から RAID-Z の全列位置を導出可能（`BP_GET_ASIZE` が3つの DVA の asize を合算）であるが、当該 top-level vdev の子デバイス数が**プール作成時に固定**されていることが前提である（実地調査、本プロジェクトでの検証は未実施）；D2 (RAID ストライプポリシー) 確定項目 1 はこの前提を否定した。
- 実験なし：導出可能か否かはストライプ幅の粒度により直接決定される推論であり、定量可能な数値はない。

**未済の検査**：なし。

#### 確定項目 15：起点からの extent オフセットの値域と第1版の値

**確定内容**：起点からの extent オフセットの値域は**最大 extent サイズ**により上限が画定され、ファイルサイズによってではない；本プロジェクトの値域は [0, 32768) であり、**15 ビットで十分**である。第1版では本フィールドは恒常的に 0——分割は予約機能であり第1版では無効化；ビットは確定項目 3 の「固定幅スロット確保」に従い維持、理由はポリシーであり使用量ではない。

**射程**：言及しているのは値域であり、幅 2 バイトは確定項目 11 のオフセット表にある。D9 (暗号化) 確定項目 12 第2項はファイルサイズに基づき 32–64 ビットと見積もったが、過大評価であった。本プロジェクトで値域がより狭い根拠は D4 (チェックサム位置) 確定項目 7 である：ディスク上の extent 占有は恒常的に = ユニット = 32768。

**根拠**：

- 現役実測（2026-08-28 `fs/bcachefs/extents_format.h` を実地調査、本プロジェクトでの検証は未実施）：`crc32.offset:7` / `crc64.offset:9` / `crc128.offset:13`、付随する `CRC128_SIZE_MAX (1U << 13)` => 本フィールドの値域は最大 extent サイズにより上限が画定される。
- 実験なし：値域は2つの確定済み条項を除算して得られる算術であり、定量可能な数値はない；分割を第1版で無効化するか否かは D4 (チェックサム位置) 確定項目 7 が決定。

**未済の検査**：なし。

```

**出处 `.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md:311-321`（整段抄，未转述）**

```markdown
### 他の決定との連鎖

| 決定 | 連鎖点 |
|---|---|
| D4 (チェックサム位置) | 確定項目 13 は「ポインタ総幅」を制約から予算へと降格させ、D4 (チェックサム位置) の該当セルを2セグメント形態へ書き改めた；論理 extent のディスク占有恒常 32768 は D4 (チェックサム位置) 確定項目 7 から与えられ、確定項目 3 と確定項目 15 はその上に成立している |
| D2 (RAID ストライプポリシー) | 確定項目 1 と確定項目 14 の前提はストライプ幅可変の基準であり、D2 (RAID ストライプポリシー) 確定項目 1 により都度変更可能と定められている |
| D9 (暗号化) | nonce キャリアは確定項目 13 により半分確定（ヘッダに属し、論理 extent ごとに1つ）；アルゴリズム種別と圧縮の各セグメントの登録表は D9 (暗号化) にあり、AAD 期待値をポインタから読み戻すことは禁止（D9 (暗号化) 確定項目 16） |
| D12 (対象メディア) | D12 (対象メディア) 確定項目 4 のマウント照合表において、デバイス識別子の不一致はマウントを拒絶せねばならず、根拠は確定項目 1 の位置エントリが dev を担持することである。⚠️ 当該判定基準が述べているのは「それを消費する**アドレス算術**がデバイス識別子を担持しているか否か」であり、btrfs のアドレス算術にはデバイス識別子が含まれており chunk ツリーにあるだけである；判定基準は成立するが、「担持 => ポインタ内になければならない」という飛躍は崩れる |
| D15 (フォーマット凍結ポリシー) | ポインタフィールドレイアウトは D15 (フォーマット凍结ポリシー) 確定項目 7 の第2層である；確定項目 6 の中央マッピングエントリフィールド表も第2層に位置する（D15 (フォーマット凍結ポリシー) 確定項目 1 の根拠が逐字指定） |
| D8 (コアインデックス構造) | ポインタ幅は内部エントリおよびツリーテーブルエントリに入り、D8 (コアインデックス構造) 確定項目 6 / 8 の複数の幅は確定項目 7 / 8 に伴い変更される；幅広ポインタが成立するか否かは D8 (コアインデックス構造) 確定項目 2 の 16 KiB ノードと結合している |

```

**出处 `.claude/kb/decisions/19-ブロックポインタ構造と幅予算.ja.md:322-325`（整段抄，未转述）**

```markdown
## 過去の版

D19 (ブロックポインタ構造と幅予算) の履歴項目は [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:1-13`（整段抄，未转述）**

```markdown
## D19 块指针的结构与宽度预算 —— 已定

**中文** · [English](19-block-pointer-structure-width-budget.en.md) · [日本語](19-ブロックポインタ構造と幅予算.ja.md)

D19（块指针的结构与宽度预算） 管块指针这一格：指针分几段、每段有哪些字段、各多宽、按什么次序排；位置条目与中央映射谁是解引用与释放判定的权威；中央映射条目的 key 与 value 怎么编码；映射 key 的哪几段跟着指针走，出生序号怎么发。不管的：单元头的字段表与单元类型码在 D18（块里携带什么信息）；节点大小与树内条目布局在 D8（核心索引结构）；落点坐标系与分配记录的 key 在 D3（空间分配） 已定项 3 / 7，提交内生块的 bump 次序在 D3（空间分配） 已定项 10；校验和覆盖多大、逻辑 extent 与单元的关系在 D4（校验和位置）；加密算法、nonce 怎么产出、AAD 期望值从哪来在 D9（加密）；根记录与系统配置的字段表在 D22（单元原子性怎么合成） 已定项 7 / 9；条带宽度的粒度在 D2（RAID 条带策略） 已定项 1。

<!-- doc-lint:registry name-col=2 -->

| 编号 | 简称 |
|---|---|
| N1 | 读不出先重读一次 |
| N3 | 对不上的留在已分配 |

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:34-47`（整段抄，未转述）**

```markdown
#### 已定项 1：位置条目带设备身份

**定案**：位置条目里带设备身份（dev id），每个物理位置各带一份；不引入「逻辑地址 + chunk 映射表」这层间接。

**射程**：管「带不带」，不管宽度（已定项 4）与次序（已定项 11）。D2（RAID 条带策略） 已定项 1 的「逐次可变」是这一格正推那一侧的前提，而带 dev 在两种粒度下都成立（它是能力更强的那一侧）⇒ 粒度定案只影响位置条目能不能被压缩表达，不影响带不带。宽指针成立与否与 D8（核心索引结构） 的节点大小耦合：同样 40 字节的指针头部，4 KiB 节点付 +98%、16 KiB 节点只付 +14%，D8（核心索引结构） 已定项 2 取 16 KiB 并钉成常量。E20（指针变宽的 CPU 缓存代价） 量的是 40 字节的头部，定案是 50（已定项 7 / 11），多出来的那几字节 E128（指针带出生 (树, txg) 之后的点查代价） 量到记账树点查慢 1.8%–4.5%，前提没翻。带 dev 最大的那笔代价（移除设备要全量扫描重写每一棵带指针的树）由 D1（数据可移动性 / 反向索引） 已定的反向索引降到 O(该设备的引用)，那条路本工程本来就要走。

**依据**：

- E20（指针变宽的 CPU 缓存代价）：指针变宽的代价只在工作集超出 L3 时出现，16 KiB 节点把它吸收掉——宽指针在本工程成立的直接前提，也是 D8（核心索引结构） 已定项 2 的依据侧。
- 现役实现（2026-08-28 现查，未在本项目验证）：带 dev 的 bcachefs 移除设备要全量扫描重写每一棵带指针的树（`bch2_dev_usrdata_drop()`），靠 `BTREE_ID_backpointers` 才降到 O(该设备的引用)，ZFS 把 dev 焊死在 blkptr 里、移除设备只能加一层永不消失的 `vdev_indirect_mapping`（只能 condense，不能删）；不带 dev 的 btrfs 要为映射表的自举在系统配置里挖一块定长不可扩的 `sys_chunk_array[BTRFS_SYSTEM_CHUNK_ARRAY_SIZE]`（2048 字节，`include/uapi/linux/btrfs_tree.h` 里 `BTRFS_SYSTEM_CHUNK_ARRAY_SIZE` 的定义与系统配置结构里这个字段），且 balance 照样重写全部引用。
- 三方论证（2026-08-28）：正推腿与反推腿结论相反，分辨它们的事实是 D2（RAID 条带策略） 的可变粒度；本地腿整轮作废，腿况与经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:48-61`（整段抄，未转述）**

```markdown
#### 已定项 2：密文校验和取 32 位

**定案**：位置条目里的密文校验和取 32 位，正好是完整 CRC32C 的宽度，不涉及截断；实现不许走「更宽的 CRC 截断到 32 位」这条路。

**射程**：结论是条件式的，引用时条件要一起引——它只在这一组输入下成立：10 TB 池、4 KiB 块、每周 scrub、10 年、损坏率 1e-9 每块读、漏检期望上限 1e-6。重开条件可判定：拿到现场损坏率且它超过 3.37e-9，这一格当场重开。候选算法一个都没被淘汰（偏离都在一个数量级以内）。威胁模型里介质损坏不是攻击者，这一格不判抗攻击性。

**依据**：

- E40（密文校验和位宽）：这组输入下最小位宽 31（与原推导那句「n > 30.3」取整一致），32 位的余量 3.37 倍——五个输入（池容量、损坏率、服役年限、scrub 周期、块大小）每一个各自错这么多倍才要 33 位；截断的 CRC 不是短一点的 CRC——CRC32C 全宽对单比特翻转 0 漏检是结构性质，截断之后变成概率检出。
- 旁证（现查，未在本项目验证）：btrfs 的默认校验和就是 32 位 CRC32C（`BTRFS_CSUM_TYPE_CRC32 = 0`），它那 256 位槽位是为可选的 SHA256 / BLAKE2 留的。
- 用户定案，原话与那天的推导在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:62-75`（整段抄，未转述）**

```markdown
#### 已定项 3：定宽，不加密时留位

**定案**：指针按最坏情形定宽——头部里的 MAC（128 位）与 nonce（96 位）恒定占位，不加密的卷把它们留成空位，不用类型标记省掉；这些位在所有卷上都占着，也不挪去装别的字段。

**射程**：可选的是 AAD 那道运算（加密关着就不算），不是格式（`.claude/rules/fs-design.md`「加密是可选的运算，不是可选的格式」）。按介质开放布局分支那个机制管不到这一格：incompat 位管的是读者认不认识这套布局，这一格管的是在同一套布局里一个指针内部怎么编码；先定宽、以后再加一套变长布局不是扩张，是多一套要永远维护的解析器加 checker 加崩溃点重放。不加密时白占 MAC 16 字节 + nonce 12 字节 = 28 字节 / 逻辑 extent，按 D4（校验和位置） 已定项 7 的口径（逻辑 extent 占盘恒 32768 字节）是 `28 / 32768 = 0.0854%`。变长那一侧省的是那 28 字节、不是省一个字段（算法类型字段 D9（加密） 已定项 12 第 4 项本来就要有），要付的是解析必须从头走：定宽指针可以按偏移直接定位第 k 个，变长不行；「恢复期扫一个部分损坏的节点时能不能跳过坏掉的那一条继续读」是推论，未实测。 加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏，同 I-2.4（头校验和覆盖范围） 给单元头 29 字节的读法（实审 A3a 落地在两处读者——多层码 2 树内部条目、journal 新根段指针：`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent 偏移那 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与加密开不开无关；checker 在走读跟随的每条指针上判，core 每个读盘上指针头部的读者都判，落点在 [checks-owed.md](../checks-owed.md) 已还清表 C581（extent 偏移非 0 没有判据） 那一行。

**依据**：

- 无实验：定宽换变长省下来的是空间，省掉的是「一个指针单独捡起来能不能直接按偏移解出来」，两边不是同一个量纲，没有可量的量。撑它的是 `.claude/rules/fs-design.md` 两条——「不为省空间牺牲自包含」，以及格式分支的判据（看它能不能把「用错了」变成「挂不上」，且判据里不许出现「这样更快 / 更省」）。
- 用户定案，原话与 2026-09-12 的重申在变更史。
- 判决 2026-10-01 采：extent 偏移第一版非 0 判该指针所在的结构损坏（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。

**欠**：C210（三个「第二套」属于哪一类扩展没人判过）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:76-96`（整段抄，未转述）**

```markdown
#### 已定项 4：位置条目 14 字节

<!-- format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61 -->

**定案**：设备 ID 4 字节、物理偏移 6 字节（16 KiB 槽号）、密文校验和 4 字节 ⇒ 位置条目 14 字节。

| 段 | 宽 | 为什么是这个宽 |
|---|---|---|
| 设备 ID | **4** | 上界指不到任何已定条款（D12（目标介质） 不封顶介质套数、全仓没有 dev id 回收条款 ⇒ 窄宽度定的是池一生里出现过的设备身份总数），按 `.claude/rules/fs-design.md`「不为省空间牺牲自包含」取宽的一档；先例是已定项 3 |
| 物理偏移 | **6**，编码为 **16 KiB 槽号** | 索引节点按 16384 对齐、大小就是 16 KiB（D18（块里携带什么信息） 已定项 9）⇒ 按 32 KiB 单元号编码表达不了半数索引节点的落点，按字节偏移则让非 16 KiB 对齐的起点变得可表示。6 字节槽号寻址 4 EiB / 设备，同宽的字节偏移只有 256 TiB |
| 密文校验和 | **4** | 已定项 2 |

**射程**：物理偏移与 D3（空间分配） 已定项 7 是同一个坐标系（D3（空间分配） 已定项 3 依据第 2 条逐字要求同一个坐标系）⇒ 两处一起定，不许各定各的。树表单元指针与根记录合计这两个下游数随它改，而根记录的合计不在这里登记，唯一权威处是 D22（单元原子性怎么合成） 已定项 7 的字段表。相对旧的 11 字节，每个位置条目多 3 字节、每个指针多 6 字节。

**依据**：

- E97（记账与分配记录的条目编码）：判据 7 按可表示性判落点编码——按 16 KiB 槽号编码时全部合法起点都表达得了，按更粗的单元号有一半写不出来。
- 用户定案，原话与「这个数此前没走过决策流程却已经被消费掉」那次复核在变更史。

**欠**：C238（位置条目从 11 变 14 之后「53」这个派生数没人回扫）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:97-132`（整段抄，未转述）**

```markdown
#### 已定项 5：位置条目降为提示，中央映射是唯一入口

**定案**：取混合——

| 件 | 定案 |
|---|---|
| 位置条目（已定项 4 的 14 字节） | **降为提示**。宽度与段划分不变，变的是它的地位：读到它可以直接去那个落点试，但它可能过期 |
| 中央映射 | **解引用与释放判定的唯一入口**。搬迁只改这里一条条目，「谁引用我」根本不用知道 |
| 态别 | 两者都不是权威态清单的第四项：映射可从单元自描述扫描重建，承重面仍是 D20（承重面：单元的原子性与自包含） 的单元 |

随定案一起生效的三条硬规则：

1. **释放一律经映射，不经提示**；经映射核到那条映射条目之后，释放之前还要按它位置项里带的单元校验和读盘核一次。**核出对不上时隔离那个槽，发布照成**：逻辑上照样释放（映射条目去掉），物理槽不还回空闲池，记进隔离并计数报出。
2. **单元头或提示必须带块 birth**——已满足：D18（块里携带什么信息） 已定项 7 的数据单元类身份段本来就含诞生代号 8 字节。
3. **提示过期的多跳率要有运行时观测点**（走错分支是性能悬崖，而悬崖不报错，`.claude/rules/fs-design.md` 硬要求 4）。

**挂载态怎么读映射**：第一版挂载态打开池时把中央映射整片读进挂载态（`crates/singlefs-core/src/mounted_read.rs` 的 `central_mapping_entries`），之后的解引用查这份内存里的条目、不再发映射读；中央映射长成多层时，挂载态打开池时把整棵映射树读进挂载态，解引用照旧只查内存、不为映射发设备读；写侧每次发布按映射树现在的条目数与这次发布最多新增的条目数做映射树容量准入，不靠别的树容量上的数字巧合。多层之后映射树没有条目数上限，结构上限只有头里层级那 1 字节；在规划那一步、任何落盘之前按这次真要写的条目算出形状，层级写不下就拒（`HeightBeyondTheLevelField`），不另立条目数上界（实十九取，主 agent 定）。「提示过期的一次解引用发 3 次设备读」这个读数就骑在这一条上，这个读数是根兼叶时量的，多层落地时由用例重量（里程碑「覆盖写、释放、回退与复用」收口表第 28 行）。分配记录树挂载时同样整棵读进挂载态；extent 树不整棵读，打开文件时按位置走下去（D8（核心索引结构） 已定项 14，用户 2026-09-24 定）。

**硬规则 1 的读盘核读不出、核出对不上时怎么办**：释放之前按位置项读盘核校验和，读盘本身失败或核出对不上，都先重读一次，两次都读不出或都对不上才按对不上处置（重读对不上那一半主 agent 2026-09-25 定：一次瞬时坏读不许让单元永久隔离；实二五已实现）。两次读结果不同类（先读不出、重读对不上，或反过来）按两次都没对上处置，报重读那一次的样子（实二五取，被攻过零轮）。任一份核出对不上，这个单元在每块盘上的分配记录都留在「已分配」、不改成已释放（另一块盘上那一份对得上也一起留，各盘的账保持对称）——落盘即跨重挂，准入里照已分配算、不另进式子。位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏：在任何写之前拒绝，盘上不变。从盘上重建上一版（`crates/singlefs-core/src/recovery.rs` 的 `rebuild_version`）时读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错，没见证时除外（所选那一版系统配置没见证、它的数据单元读不出时，可写挂载照 D23（journal 的角色与格式） 已定项 14「所选根走读失败、系统配置没见证它时退回见证到的那一版」退回见证到的那一版；C755（恢复与两种挂载触发退回的失败不同） 的并集判退，零轮措辞，实现在实现批）。

**射程**：同时收口 D21（权威态与派生态的分界） 那条「共享之下多引用者住哪」的清单那一半与 D26（后台整理与放置回收） 已定项 5 的位置权威三选一，别处一律链到已定项 5；它原本还收口 C34（明文映射层的态别三处打架），而 C34（明文映射层的态别三处打架） 2026-09-07 随 D9（加密） 已定项 5 改取「只做 scrub」而无对象，那一份收口落空、另两处不受影响。映射条目的字段表不在这一格里，归已定项 6。三处已知边角没有编号：映射树的 key 布局要兑现 D8（核心索引结构） 已定项 7 那条骑手（去重窗口要被 `T_dirty` 界住）；提示的刷新政策只有「随重写刷新」一种被建模过；缓存命中这一维零测量，E109（位置权威三臂的运行时代价） 明说不答。自举豁免的三类见已定项 8。硬规则 1 的读盘核读不出、核出对不上时怎么办见定案那一段（C394（释放判定不核映射条目位置项里的单元校验和） 的三问）；「先重读一次」只重读一次，不退避、不多次重试。

**依据**：

- E109（位置权威三臂的运行时代价）：三条臂逐维量出来——搬一个单元的代价在指针权威下随引用树数线性、在中央映射与混合下与它无关；发布多付一笔与叶数无关的定额；读只多一成到四成。绝对值要连取样点一起引：它的主取样点上引用树与映射树恰好同高，那是一个退化点。
- E96（混合架构的一致性）：按提示解落点会误放，硬规则 1 由它逼出来；它的 `hdr_key` 臂引的是 D18（块里携带什么信息） 已定项 3 而不是已定项 7 的字段表，那条「要加字段」的格式代价不存在。
- 用户定案 2026-09-23：释放之前读盘核位置项里的单元校验和，核出对不上时隔离那个槽、发布照成（`research/prompts/m2-placement-falsepositive-r1-main-verification.md` 第三节判 C394（释放判定不核映射条目位置项里的单元校验和） 那一格：三条硬规则都不回答要不要核），原话在变更史。
- 用户定案 2026-09-24：挂载态整片读映射照当天的实现写成条款，多层映射由用户同日另一次定案（四棵树的分裂都在里程碑二做，含挂载态读多层映射）改成整棵读，实十九落地（C483（挂载态读映射与树节点回退都没有条款） ①，并行线二落地时定下、没有条款），原话在变更史。
- 用户定案 2026-09-24：C394（释放判定不核映射条目位置项里的单元校验和） 三问——读盘失败先重读一次再按对不上处置、重建时容下读不出的数据单元、隔离那一份的分配记录留在已分配（判决 `research/prompts/m2-newq-r1-main-verification.md` 第三节 N1（读不出先重读一次）–N3（对不上的留在已分配），原话在变更史）；无实验：候选的读数来自三方攻方腿在副本上的原型（`research/prompts/m2-newq-r1-opus-model/`），没有登记成实验。
- 用户定案 2026-09-24：多层映射挂载时整棵读进挂载态、写侧补映射树容量准入（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T3）。
- 用户定案 2026-09-25：只一份核出对不上时两块盘一起留、位置项指池外的盘照当天拒绝写成条款（实十六接续报告 `m2-writepath-implementer-report.md` 第六节 Q2、Q3），原话在变更史；无实验：两条都不改盘上格式，只定处置。
- 主 agent 定案 2026-09-25：`records/2026-09-24-里程碑二收尾调度.md` 第三节「瞬时坏读让单元永久隔离」那一行。
- 没见证时除外：根环三方第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第四节「没进任何一格的，去向」表里 D19（块指针的结构与宽度预算） 已定项 5 回看那一行（辩方 Z5.2：没见证的 R 上数据单元读不出，可写挂载由照常挂 R 改成退回上一版）；用户定案 2026-10-20（C755（恢复与两种挂载触发退回的失败不同）「三句都补」），原话在变更史。无实验：数在三方腿的副本上量，没立实验号。

**欠**：C248（树表单元的自举豁免没写过）、C253（66 048 那笔定额两份 kb 说法相反）、C133（搬迁的转发信息没有载体）、C279（打包之后扫描重建同时看到旧单元与槽）；C417（位置提示的刷新政策只建模过一种）、C418（位置权威三臂的缓存命中维零测量）——依据那两条实验各自自陈答不了的格；C543（重建上一版只带实例表第 0 片）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:133-162`（整段抄，未转述）**

```markdown
#### 已定项 6：中央映射 key 取按类复用写序键（收严后）

**定案**：key 定宽、逐段排布，定义与宽度逐字同 E134（映射 key 格式代价·槽基线与两条新臂） 的 `class_reuse_tagged` 臂——

| 类 | key（定宽，逐段） | 宽 |
|---|---|---|
| 码 1 数据单元 | 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10（实例代号 4 + 事务号 6） | 27 |
| 码 2 / 码 3 | 类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 | 25 |

value 照已定项 4 / 已定项 5：位置条目 × w 副本（w = 2 时 28 字节）。补齐与条目总宽归已定项 10。

**它带着的四条收严**（三轮对抗里按「记一次输 / 只许收严 / 写明收严在哪」加上的，定案之后照样要守）：

- 类标签取被命名对象在 D18（块里携带什么信息） 已定项 11 登记表里的码，D27（小数据打包容器） 槽里的对象恒为码 1；打包与整容器重写时，槽里 key 的各段逐字节取打包前那一版单元头里的值。
- 出生树取类身份段里的出生树（码 3 偏移 43），出生 txg 取诞生代号（码 3 偏移 73；码 1、码 2 同取诞生代号），实例代号取写序的前 4 字节。
- D16（发布语义） 已定项 5 末段那句「一个事务最多写一个单元的用户数据」要按罩全部码 1 单元读（预分配、截断的尾单元、打洞两端、reflink 两端的拷贝都算），最好写进那一句。
- 出生序号只做 key 的一段，不拿来给固定点多版择新——拿来择新就落进 D18（块里携带什么信息） 已定项 1 否掉的「为分辨陈旧副本而设的序号」；D18（块里携带什么信息） 已定项 1 照写序那条先例欠一条射程注。

**射程**：管 key 与 value 各是哪几段、各多宽，不管补齐与条目总宽（已定项 10）、不管装哪几类（已定项 8）、不管出生序号怎么发（已定项 9）。它落 D15（格式冻结政策） 第 2 层。码 1 那一半的唯一性靠「码 1 的写序已经是唯一的版本判别子」这条推论，而它建在 D16（发布语义） 已定项 5 末段那条事务切分纪律上（配 D23（journal 的角色与格式） 已定项 7 一条记录一个事务号）；那一段 2026-09-13 随 C123（用户定案被改后无人复核） 的复核收口、切分纪律没改，推论成立，剩下的口径打架记在 C310（事务切分纪律与记录数口径打架）。「先」照录：这一格是按已有的代价与性能数取的，新证据推翻它时重开，不是论证出来的唯一解。

**依据**：

- E137（映射 key 形态的性能差距）：三条臂在每次 fsync、后台负载、随机读三类负载上的差距——按类复用写序键在每次 fsync 的多数格与随机读全格领先，在全域随机覆写、删文件、单文件快照删除上输；用户看了这张表取它。⚠️ 模型里物理落点就是写入顺序，这是它在搬迁与快照删除上最有利的情形，老化之后缩多少没量。
- E133（映射 key 各候选的格式代价）：两轮三方论证里出现过的每一种 key 落到格式上各付多少字节，七条候选的指针宽、条目宽与扇出由它给。
- E134（映射 key 格式代价·槽基线与两条新臂）：这一格取的那条臂就是它的 `class_reuse_tagged`，收严加的 1 字节类标签与 D27（小数据打包容器） 槽的两种基线由它按行报出。
- E136（映射 key 岔路的代价行）：交用户定之前四样缺的代价数——码 2 头加 4 字节在 D18（块里携带什么信息） 已定项 14 的三档真实头宽下一格扇出都不掉，映射树层数只在一段池规模区间里差一层。
- 三轮对抗的判决与三轮论证的经过在变更史；用户定案，原话在变更史。

**欠**：C288（映射 key 定案的连带改动没落到别的决策）、C289（码 2 的出生树取哪个字段没写）、C163（固定点各轮写序相同使平局破除落空）、C286（只读后 remount 取不取新实例代号）、C287（切换收养开放 checkpoint 的事务后再崩）、C310（事务切分纪律与记录数口径打架）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:163-176`（整段抄，未转述）**

```markdown
#### 已定项 7：指针头部带出生 (树, txg)，各 8 字节

**定案**：指针头部在 MAC 16 + nonce 12 + 算法类型 1 + 压缩算法码 1 + 压后长度 2 + extent 偏移 2 之后再带出生树 8 与出生 txg 8，头部合计 **50**；指针照旧定宽（已定项 3）。压缩那 3 字节是压缩算法码 1 字节（登记表住 D9（加密），0 = 不压缩，第一版写 0）加压后长度 2 字节（第一版写 0，够表达 [0, 32768) 的单元内长度）。

**射程**：定的是哪 16 字节住指针头部，不定指针总宽（那是已定项 8 与已定项 11 合出来的 88 / 86）。压缩那 3 字节补的是 D9（加密） 已定项 13 第 2 条那条硬要求的格式载体——它要求压缩与加密的次序在格式上说得清，而在此之前仓里没有任何字段承载「这份密文是压过的、压前多长」。⚠️ 主 agent 推荐的是不留（压缩是可选运算，而「为加密留的位所有卷都留」那条原则只管加密），用户弹窗定案选了留，没有走三方论证，可推翻。从指针拿这两段的消费者有四个：D5（快照 / 空间记账机制） 的 O(1) 判定读出生 txg；deadlist 条目按版本身份记，出生树要从指针取；中央映射 key；AAD 形态要指针有这两个字段的位置，而值要经独立验收（D9（加密） 已定项 16 逐字不许从指针里读回来）。改新池新建文件的字节：是——每一条指针加 3 字节。

**依据**：

- E128（指针带出生 (树, txg) 之后的点查代价）：记账树内部条目加宽之后四轮都更慢，inode 树那一对四轮不稳定、不能引；它量的只是这 16 字节，是这一格代价的下界。
- 一轮三方论证，判决在 `research/prompts/d19-item7-r1-main-verification.md`：反推腿指出已定项 8 当时的末句把「住在指针里还是住在引用条目里」留了口子，两种读法都说得通。
- 用户定案（「16 字节住在指针里」），原话与候选乙、丙的出局经过在变更史。

**欠**：C295（指针里的 key 段算不算 AAD 的查找路径）、C296（journal 重放时凑不齐映射 key）、C297（inode 内部条目里出生树两份没有定处理）、C237（码 1 / 码 2 的指针里没有出生 (树, txg) 的位置）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:177-201`（整段抄，未转述）**

```markdown
#### 已定项 8：全部指针进映射，引用侧按类携带映射 key 的各段

**定案**：中央映射装码 1、码 2、码 3 三类单元；指向一个单元的指针携带该类映射 key 除类标签外的各段，类标签由查找路径给。

| 被指的类 | 指针携带（已定项 6 的 key 各段） | 指针宽 |
|---|---|---|
| 码 1 | 出生树 8 + 出生 txg 8 + 写序 10 | 数据指针 **88** |
| 码 2 / 码 3 | 出生树 8 + 出生 txg 8 + 实例代号 4 + 出生序号 4 | 节点指针 **86** |

**不进映射的三类（自举豁免）**：映射树自己的节点、树表单元、实例表单元。它们的父指针（映射树的根、根记录里的树表指针与实例表指针）里的位置条目是权威，不是提示，已定项 5 的「唯一入口」与硬规则 1 对这三类不适用；三类都是池级单份、引用者只有一个。这三类的父指针与指向码 2 / 码 3 的节点指针同宽 86：位置条目 14 × 2 是权威，实例代号 4 + 出生序号 4 照填被指单元头里的值、不拿去查映射；解析器按被指单元的类定指针宽，不按对面是不是豁免类区分。豁免三类之外的单元（码 1 数据单元与码 2 树节点——extent 树根、inode 树根、inode 叶容器）位置提示读不出时一律经映射回退；豁免三类（映射树根、树表、根记录那条链）不回退。

**射程**：判的是映射装哪几类、指针带 key 的哪几段，不随指针宽度变（其中出生 (树, txg) 那 16 字节住指针头部由已定项 7 定）。只装码 1 那一支在两处正确性上输：码 2 / 码 3 的块同样进 deadlist / livelist，而 D5（快照 / 空间记账机制） 定 deadlist 条目按版本身份记、经映射释放，不进映射就只能按落点记，整理或掉盘重建把它挪走之后会按旧落点释放；搬一个被只读快照引用的码 2 / 码 3 要改写快照树的祖先，取新 birth 违反引用区间、保留旧 birth 违反 `birth` 的定义句。全部指针进映射在这两处都不输：deadlist / livelist 按映射 key 记，搬迁只改映射一条条目，被搬单元逐字节不变、birth 不变。代价：每次 fsync 与后台每单元的映射块都比只装码 1 多一到两成以上，随机读未命中也涨；这一列还少算两块（派生树的码 2 在映射里的删插、extent 节点自己提示过期走映射的读）。父节点重写时位置提示写映射位置（C417（位置提示的刷新政策只建模过一种） 第一版取「随重写刷新」）。checker 与 GPU 判器把豁免三类之外的码 1 / 2 / 3 位置条目只当提示：走读先按提示读，两份都对不上再按映射位置读，引用集合记映射位置。

**依据**：

- E137（映射 key 形态的性能差距）：两种配置（只装码 1 / 全部指针进映射）逐格量出代价差，这一格的代价行整段取自它。
- E134（映射 key 格式代价·槽基线与两条新臂）：两种配置下的指针宽、条目宽与扇出由它给，88 / 86 是它那两行加上已定项 7 的压缩 3 字节。
- E128（指针带出生 (树, txg) 之后的点查代价）：定案那天漏录、次日补上的那一笔点查代价，记在这一格的代价行里。
- 两轮三方论证，判决在 `research/prompts/d19-item8-r1-main-verification.md` 与 `research/prompts/d19-item8-r2-main-verification.md`（第一轮两条云端腿结论相反，用户要求重推，第二轮立场对调）。
- 用户定案，原话与两条补法为什么都要改一批已定条款在变更史。
- 2026-09-23 判决 `m2-lines234-code-r1-main-verification.md` 里判树节点要不要经映射回退那一格——攻方腿实测同一种损伤下数据单元多跳一次读回正确字节，而 extent 树根让 `open_pool_for_read` 整池挂不上。
- 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3：写路径、重建与 checker 照「映射是唯一入口」重构（取乙），父节点重写时写映射位置，checker 与 GPU 判器把位置条目只当提示。第七节用户定案「如果不合适，逻辑不通顺或者不自然，就重构」，原话在变更史。

**欠**：C293（全部指针进映射之后 E137 少算的两块）、C292（码 2 身份段零消费者的理由已不成立）、C248（树表单元的自举豁免没写过）（并入自 C280（中央映射装哪几类单元没有条款））。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:202-214`（整段抄，未转述）**

```markdown
#### 已定项 9：出生序号怎么发，按什么次序发

**定案**：出生序号从 0 起；同一棵树内每写出一个码 2 或码 3 单元加 1（两类共用一个计数）；作用域换到下一个 checkpoint 时清零；同一个 checkpoint 里同一个节点被固定点重写第二次换新号。发号次序与提交内生块从开放聚簇段 bump 的次序是同一个次序——按树 ID 升序，树内先叶后根、同层按 key 升序，中央映射树倒数第二，树表单元最末。

**射程**：次序本身的唯一登记位是 D3（空间分配） 已定项 10，这里不存第二份。出生序号是中央映射 key 的一段（已定项 6），key 必须在单元写出去之前就定得下来，所以发号绑在落点的分配次序上：实现里只有一个游标、checker 只核一份次序。它只做 key 的一段，不拿来给固定点多版择新（已定项 6 的第四条收严）。用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这一档。改新池新建文件的字节：是——新池新建文件写出的十一个提交内生块（默认几何；个数随几何变，环 1024 MiB 时 13 个）的落点与出生序号都随它定，逐块的值见 D3（空间分配） 已定项 10 ⑤ 的表。

**依据**：

- 无实验：次序与初值本身没有可量的量，发号与落点共用一条规则这件事的代价在 D3（空间分配） 已定项 10 那一侧；撑这一格的是「key 要在单元写出去之前定得下来」这条推论加已定项 6 收严论证里那两句（同一容器在两个 checkpoint 各拿到出生序号 0、多买到的只是运行期映射不撞）只在重写换号下成立。
- 用户定案，原话在变更史。

**欠**：C291（出生序号的分配规则只在实验源码里）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:215-227`（整段抄，未转述）**

```markdown
#### 已定项 10：映射 key 一律 27，条目一律 55

**定案**：中央映射的 key **一律 27 字节**——码 1 的 (类标签 1, 出生树 8, 出生 txg 8, 写序 10) 天然就是 27；码 2 / 码 3 的 (类标签 1, 出生树 8, 出生 txg 8, 实例代号 4, 出生序号 4) 是 25 字节，**末尾补零到 27**。中央映射条目因此**一律 55 字节**（key 27 + value 28，value = 位置条目 14 × 2）。映射树节点头的 key 区间同样按 27 存。

**射程**：连带两处随它回到定宽——D5（快照 / 空间记账机制） 已定项 6 的稀疏旁表条目定宽 31（映射 key 27 + 计数 4），D6（快照实现模型） 已定项 2 的 livelist 条目定宽 36（头树 ID 8 + 映射 key 27 + 事件类型 1）。按类变长会把 D8（核心索引结构） 已定项 11 的两条通用规则一起破掉：「定宽条目连续排列」与「声明长度 = 条目数 × 条目宽」——码 2 的条目宽字段只有一个值，两宽就得给它一个哨兵，而哨兵一立，节点内的定宽二分就没了，扫描期解一个映射节点要从头顺序走。代价是每条码 2 / 码 3 条目多 2 字节，新池新建文件多 10 字节。用户弹窗定案，没有走三方论证，可推翻；主 agent 推荐的也是这一档。改新池新建文件的字节：是。

**依据**：

- E145（码 2 自描述头与映射树 key 宽的代价）：它量过的是一宽那条臂；「两宽 55 / 53、按 key 首字节自定界」是那一轮没有量过的第四种形态，拿它当定案等于引一个没有产物的数。
- 用户定案，原话在变更史。

**欠**：无（C321（码 2 头的条目宽是单值而映射树条目按类两宽） 2026-09-14 随用户定案映射 key 回到定宽 27 还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:228-255`（整段抄，未转述）**

```markdown
#### 已定项 11：指针的偏移表、位置条目的顺序、映射树的根住哪

**定案**：指针从偏移 0 起逐段排布——

| 段 | 偏移 | 宽 |
|---|---|---|
| MAC | 0 | 16 |
| nonce | 16 | 12 |
| 算法类型 | 28 | 1 |
| 压缩算法码 | 29 | 1 |
| 压后长度 | 30 | 2 |
| extent 距起点偏移 | 32 | 2 |
| 出生树 | 34 | 8 |
| 出生 txg | 42 | 8 |
| 位置条目 × 2，按设备身份升序 | 50 | 14 × 2 |
| key 尾段：码 1 写序 10；码 2 / 码 3 实例代号 4 + 出生序号 4 | 78 | 10 / 8 |

⇒ 指向码 1 的指针 **88**，指向码 2 / 码 3 的指针 **86**。位置条目按设备身份升序排列，立成 I-2.5（位置条目按设备身份升序）。**中央映射树的根住根记录**：根记录带一条 86 字节的中央映射树根指针，不进树表；根记录合计 **457** 字节。树表第 1 版 **12** 条。

**射程**：定的只是次序——段的集合与宽度是已定项 3 / 4 / 7 / 8 定过的。压缩那 3 字节紧跟算法类型，与它同属「这份密文是怎么产出来的」那一组，排在 extent 偏移之前是为了让加密与压缩四个字段连成一段。根记录合计的唯一权威处是 D22（单元原子性怎么合成） 已定项 7 的字段表，树表第 1 版那一条随 D5（快照 / 空间记账机制） 已定项 11 加 deadlist 而来。用户弹窗定案，没有走三方论证，可推翻。改新池新建文件的字节：是——每一条指针的段偏移与宽度，以及根记录的合计。

**依据**：

- 无实验：偏移表本身没有第二种选法，段的集合与宽度都已经定过，这一格只排次序，没有可量的量。位置条目按设备身份升序是为了让两条位置条目有一个确定的次序，好立成不变量。
- 用户定案，原话在变更史。

**欠**：无（C304（实例表链指针记录装不下 83 宽的指针） 2026-10-11 已还清，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:256-268`（整段抄，未转述）**

```markdown
#### 已定项 12：码 2 / 码 3 的映射条目 day-1 写出，固定点一个 checkpoint 只写一次

**定案**：维持 day-1 写出（新池新建文件 6 条）。固定点写死：每个码 2 节点与码 3 容器，一个实例在一个 checkpoint 里只写出一次——发布末尾一次性 COW；第一版禁边插边 COW（D3（空间分配） 已定项 5 允许的那条实现第一版不取）。

**射程**：⚠️ 以后放开边插边 COW，D8（核心索引结构） 已定项 13 照旧要守：同一次发布里同一个 key 只按最终态落叶一次。会红的检查按单元计，不按固定点轮数计。这一格定的是写不写与写几次，不定 key 形态（已定项 6）与条目宽（已定项 10）。定案时仓里还没有任何释放与重用实验，另一条路留成「等有释放的实验之后再看」，而那句触发条件没有写进这一格。

**依据**：

- E81（提交的固定点）：「已脏节点本次发布只 COW 一次」这条读法的出处，也是固定点在这条读法下收敛的依据；照它罩码 2 与码 3，同一 checkpoint 里两版同 key 那一输才不再出现。
- 用户定案，原话在变更史。

**欠**：C352（day-1 写映射条目的定案，重开条件没人叫醒）、C163（固定点各轮写序相同使平局破除落空）。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:269-284`（整段抄，未转述）**

```markdown
#### 已定项 13：块指针分两段

**定案**：**块指针分两段：一份「头部」跟着逻辑 extent，N 个「位置条目」跟着物理落点。**
逻辑属性（MAC、nonce、算法类型、extent 距起点偏移）在头部，每个逻辑 extent 一份；
物理属性（落点、以及供无密钥侧检损坏的密文校验和）在位置条目，每个物理位置一份。

**射程**：分段把「指针总宽度」从一个约束降级为一个预算，D4（校验和位置） 那一格因此改写成分两段的形态。它只定哪一类属性住哪一段，不定各段有哪些字段（已定项 4 / 7）与次序（已定项 11）；头部后来又加了出生树与出生 txg（已定项 7），加的仍是逻辑属性。「挪出指针」不等于多一次 I/O：D4（校验和位置） 要求的是父节点存子节点校验和，它约束的是住在哪个节点，不是住在哪个结构体。

**依据**：

- D2（RAID 条带策略） 已定「多副本是同一份密文写 N 遍」⇒ 所有副本必须共用同一个 nonce；逐指针各存一份等于把同一个数抄 N 遍，且某副本那份被写错时同一份密文会得到两个互不相容的解密结果。可判定形式是 I-6.7（副本共用 nonce）。
- 现役实现给了同一个形状（2026-08-28 现查逐字核实，未在本项目验证）：bcachefs 的 `struct bch_extent_ptr` 恰好 64 位（`offset:44, dev:8, gen:8` 加四个标志位），校验和 / nonce / 压缩信息全部挪到同一个 bkey 里的独立变长条目（`bch_extent_crc32/64/128`），一条 crc 条目对其后所有 ptr 条目连续生效；nonce 本身不存，`fs/bcachefs/checksum.h` 的 `extent_nonce()` 由 `bversion`（96 位，本来就在 bkey 头部）加 per-crc 的一个小偏移（`crc64.nonce:10` / `crc128.nonce:13`）现算。
- 无实验：分两段是格式结构的取舍，撑它的是一条已定规则（副本共用 nonce）与现役实现的形状，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:285-297`（整段抄，未转述）**

```markdown
#### 已定项 14：位置条目逐个列出，不用几何公式从单一起点推算

**定案**：N 个位置条目逐个列出，各带设备身份；不从单一起点加一个尺寸推算出其余各列的位置。

**射程**：管位置条目怎么表达，不管它带哪几段（已定项 4）。前置口径是 D2（RAID 条带策略） 已定项 1 把「宽度可变」的粒度定成**逐次可变**——格式侧物理位置逐个列出、各带设备身份、不引 chunk 映射表，正是这条推导要的那个前提。它与 D2（RAID 条带策略） 已定项 1 同型：买到的是「不指错盘」，不是「失败域独立」。

**依据**：

- ZFS 能用「一个 DVA + asize」推出 RAID-Z 全部列位置（`BP_GET_ASIZE` 把 3 个 DVA 的 asize 相加），前提是该 top-level vdev 的子设备数**在池创建时固定**（现查，未在本项目验证）；D2（RAID 条带策略） 已定项 1 否定了这个前提。
- 无实验：能不能推算由条带宽度的粒度直接决定，是一条推论，没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:298-310`（整段抄，未转述）**

```markdown
#### 已定项 15：extent 距起点偏移的值域与第一版取值

**定案**：extent 距起点偏移的值域由**最大 extent 尺寸**封顶，不由文件尺寸封顶；本工程的值域是 [0, 32768)，**15 位够**。第一版该字段恒 0——切分是预留特性、第一版关闭；位仍按已定项 3 的「定宽留位」留着，理由是政策不是用量。

**射程**：说的是值域，宽度取 2 字节在已定项 11 的偏移表里。D9（加密） 已定项 12 第 2 项按 32–64 位估这个字段，那是按文件尺寸估的，估高了。本工程值域更窄的依据是 D4（校验和位置） 已定项 7：extent 占盘恒 = 单元 = 32768。

**依据**：

- 现役实测（2026-08-28 现查 `fs/bcachefs/extents_format.h`，未在本项目验证）：`crc32.offset:7` / `crc64.offset:9` / `crc128.offset:13`，配套 `CRC128_SIZE_MAX (1U << 13)` ⇒ 这个字段的值域由最大 extent 尺寸封顶。
- 无实验：值域是两条已定条款相除得出的算术，没有可量的量；切分第一版关不关由 D4（校验和位置） 已定项 7 定。

**欠**：无。

```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:311-321`（整段抄，未转述）**

```markdown
### 与其他决策的连锁

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 已定项 13 把「指针总宽度」从一个约束降级为一个预算，D4（校验和位置） 那一格因此改写成分两段的形态；逻辑 extent 占盘恒 32768 由 D4（校验和位置） 已定项 7 给，已定项 3 与已定项 15 都踩在它上面 |
| D2（RAID 条带策略） | 已定项 1 与已定项 14 的前置是条带宽度可变的口径，由 D2（RAID 条带策略） 已定项 1 定为逐次可变 |
| D9（加密） | nonce 载体由已定项 13 定了一半（属头部、每逻辑 extent 一份）；算法类型与压缩那几段的登记表住 D9（加密），AAD 的期望值不许从指针里读回来（D9（加密） 已定项 16） |
| D12（目标介质） | D12（目标介质） 已定项 4 的挂载比对表里，设备身份错配要拒绝挂载，依据是已定项 1 的位置条目带 dev。⚠️ 那条判据说的是「用到它的**地址算术**是否携带设备身份」，btrfs 的地址算术里有设备身份、只是住在 chunk 树；判据成立，倒下的是「携带 ⇒ 必须在指针里」这个跳步 |
| D15（格式冻结政策） | 指针字段布局是 D15（格式冻结政策） 已定项 7 的第 2 层；已定项 6 的中央映射条目字段表也落在第 2 层（D15（格式冻结政策） 已定项 1 的依据逐字点名） |
| D8（核心索引结构） | 指针宽进内部条目与树表条目，D8（核心索引结构） 已定项 6 / 8 的几个宽度随已定项 7 / 8 改；宽指针成立与否与 D8（核心索引结构） 已定项 2 的 16 KiB 节点耦合 |

```

**出处 `.claude/kb/decisions/13-verification-route.en.md:1-6`（整段抄，未转述）**

```markdown
## D13 Verification Route —— Settled

[中文](13-验证路线.md) · **English** · [日本語](13-検証ルート.ja.md)

D13 (Verification Route) governs how this project verifies correctness: employing all three off-the-shelf approaches—writing a custom reference implementation for differential testing, bounded exhaustive crash testing, and crash refinement formal verification (survey in [prior-art.md](../prior-art.md) Section 7.4)—without slavishly copying them, designing bespoke verification methods tailored to four characteristics unique to this project (COW issuing a new root per transaction, Merkle self-verification, nonce uniqueness for full-volume AEAD, and lack of reference implementation). It settles how oracles are divided, which two items are custom-developed, how crash-point replay is tiered and sampled, how the crash state set is defined, and what is shared between checker and implementation. Out of scope: what each invariant specifies in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md); how the three verification approaches land and what blocks what in [verification-build.md](../verification-build.md); where commit steps are registered, how structural equivalence classes are partitioned, and how many exist belong to D17 (Implementation Layering and Third-Party Pipelines) settled item 2.

```

**出处 `.claude/kb/decisions/13-verification-route.en.md:7-443`（整段抄，未转述）**

```markdown
### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Which part crash refinement verifies first** | Verifies the closed unit of "one publish + one crash + one recovery" first; first milestone is that any `jsn` continuous prefix does not split a publish; mount transparency is an independent fourth property; former candidate I-6.5 (Monotonic Nonce Watermark) withdrawn **Status: Settled.** **Progress: Implemented.** |
| 2 | **Whether transaction layer commits to serial commit** | Serial first, opening concurrency only past the intersection; future concurrent commits append records in transaction number order **Status: Settled.** **Progress: Implemented.** |
| 3 | **To what extent custom reference implementation (RefFS form) is pursued** | Taken to the extreme at the semantic level: operates in memory strictly via POSIX operations, defining persisted state and permissible post-crash states without on-disk format; written from scratch per kb, isolated crate, accepts whitelisted inputs only **Status: Settled.** **Progress: Implemented.** |
| 4 | **How crash state set of crash-point replay is defined** | Write stream segmented by device; segment closes only when every disk with writes in segment is released by its own barrier or FUA; each full write in segment takes states combined arbitrarily; in-place overwrites take third state "neither new nor old readable"; non-persisted takes old bytes generating real images; mirrored writes counted per device **Status: Settled.** **Progress: Implemented.** |
| 5 | **What is permitted to be shared between checker and tested implementation** | Shares only a constants module generated from kb, all else implemented separately; decision width probed by checker itself then compared against system configuration, reporting "declared value, unprobed" if unprobed **Status: Settled.** **Progress: Implemented.** |
| 6 | **Three mutually independent oracles** | In-memory model differential testing, independent parser + checker, and independent spec executor each verify distinct aspects; independence verified by "three-oracle collusion test", never self-proclaimed **Status: Settled.** **Progress: Implemented.** |
| 7 | **What O2 (Independent Parser + Checker) verifies** | Domain of O2 (Independent Parser + Checker) is a single image, judging set membership only; checks requiring a second input belong to record checker (inputs include persisted set from enumerator), unnumbered by design **Status: Settled.** **Progress: Implemented.** |
| 8 | **Two custom-developed mechanisms** | Merkle root chain diff (recursive comparison of adjacent generations of roots, skipping unchanged subtree pointers) and generational incremental scrub (traversing only blocks with birth generation above watermark) **Status: Settled.** **Progress: Unimplemented.** |
| 9 | **How crash-point replay is tiered** | Three tiers all exact full run; incomplete enumeration handled by exact pruning, temporarily unrunnable nodes marked ignore and opened gradually; milestone closeout and judge unit changes require main agent popup asking user to run full run, commits allowed on failure or skip with owed check; failure reproducibility prioritized **Status: Settled.** **Progress: Implemented.** |
| 10 | **What N truly must be covered exactly** | Number of equivalence classes of commit protocol structure, not number of layout schemes; implemented via closed enum plus exhaustive match **Status: Settled.** **Progress: Implemented.** |
| 11 | **When ignored nodes are enabled** | Ledger rotating by owed checks rejected; three rules retained for enabling: deterministic coverage records, `T1` (Regular Tier Wall-Clock Budget) as input, and separating discriminative power from coverage checks **Status: Settled.** **Progress: No code object.** |
| 12 | **Conflict 1: Checker-as-specification and verification paths sharing no tools** | Checker implements two directions: traversal and scan, independently judging the same invariant, sharing only format parsing **Status: Settled.** **Progress: Implemented.** |
| 13 | **Conflict 2: Refinement spec and differential model** | Spec and model written differently at representation level; common-origin observed: ≥ 5 inconsistencies with 0 spec errors triggers merge and budget transfer to exhaustive testing **Status: Settled.** **Progress: No code object.** |
| 14 | **Interlocks with other decisions** | Hash and AEAD treated as uninterpreted functions in formal verification; log structure three tiers burden symbolic execution; checker does not parse message semantics **Status: Settled.** **Progress: No code object.** |
| 15 | **Two tiers of verification code** | Harness tier (`singlefs-harness`) for unit and integration tests run anytime; checker tier (`singlefs-checker-tier`) runs quick tier by default at commit, full run upon user request **Status: Settled.** **Progress: Implemented.** |
| 16 | **Slice sizing and state count width for registered segments** | Segments whose domain exceeds u64 judged in slices, defaulting to 2^32 states per slice starting from library resume point; state count uniformly uses 1280-bit fixed-width integer with checked arithmetic; registry read only via environment variable **Status: Settled.** **Progress: Unimplemented.** |
| 17 | **Unit of verdict reuse is function** | Pruning unit is function: verdict reuse depends solely on de-commented fingerprints of dependent function units, toolchain, and resolved dependency versions; commits, tree diffs, full file text, file/crate fingerprints, manual versions, paths, and whole-stage marks forbidden **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Crash Refinement First Verifies "One Publish + One Crash + One Recovery"

**Settlement**: The first verification target of crash refinement is the relational property of recovery function `Recover`, whose minimal closed unit is "one publish + one crash + one recovery". The subject `Recover` across the four milestones refers to three call sites invoking `replay_journal` with `verify_named_units = true` in `crates/singlefs-core` (`Consult` strategy of `recover`, writable mount read phase, read-only mount), all three taking `records` from the output of `scan_journal`; `Ignore` and `ConsultWithoutNamedVerification` are arms for testing and control, placed in trusted computing base; `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` is not a production caller. The four milestones in sequence are:

| Milestone | Obligation | Placement |
|---|---|---|
| **M1 Totality** | `Recover` is defined, terminates, and outputs an image satisfying I-7.1 (Non-Empty Valid Root Slot Set) and I-8.3 (Strictly Continuous Replay Prefix) on **every** crash state enumerated in D13 (Verification Route) settled item 4 | Scaffold. **Completion criterion is not "proven", but "M2 can be stated on top of it"**—crash-point replay already enumerates full run ([verification-build.md](../verification-build.md) "crash-point replay does not sample, full run enumeration"), recovery hangs are immediately exposed, proving totality alone yields no marginal value. Prefix extraction, root selection, and ring scan are in Verus in `crates/singlefs-replay-prefix/src/lib.rs`: prefix core `replay_core` (termination and non-overflow proven inside core, loops have `decreases`; two `requires`—record table sorted ascending by (instance, counter) and counter has successor—guarded by assertions in glue `replay_journal`), root selection core `choose_root_slot` (takes max (`checkpoint_txg`, instance) among self-verified slots, tie-broken by earliest slot order), ring scan core `first_copy_of_each_record_key` (first come first served for same (instance, counter)), joint verdict written as corollary `corollary_recovery_replays_from_the_newest_self_verified_root`; gate checker-tier-formal-verification checks proof, pinned at 56 verified (`.claude/gate.d/verus-pins.tsv`). E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) serves as lightweight tool, its rerun 3 part 2 root-selection joint-verdict family calling real `choose_root`, `scan_journal`, `replay_journal` on ramdisk. Four buggy root-selection variants on legal history K family show only "take first valid" red, other three (ignoring instance, taking lower txg on tie, instance-primary) covered by Verus must-red worlds (`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` root selection section) and adversarial root ring KA (rerun 3 part 3). Outside core: system configuration selection (`choose_system_configuration`) and two glue sections (root selection slot reading, self-verification checksum, fsid, region-to-disk mapping, slot traversal order; ring scan offset enumeration, hint, parsing and CRC, disk traversal order) are outside Verus, covered by crash-point replay and crash amplification CPU judges invoking `recover` on every crash state (GPU judge rewrites `replay_journal` in WGSL, cannot verify core) |
| **M2 Prefix Does Not Split Publish** | **For any `jsn` continuous prefix, its replay result equals the result of an integer number of published checkpoints—no publish is split by prefix; when prefix stops mid-publish, that publish is wholly unapplied** (D16 (Publish Semantics) settled item 4 "a publish is wholly applied or wholly unapplied"). Two layers. **Syntactic layer** (for arbitrary self-verified record table, unconditional): applied records are integral publishes, in-flight bound and commit markers verified entry by entry, applied counters strictly consecutive from chain head (starting from anchor + 1 if readable; if unreadable, first entry `checkpoint_txg` = chosen root + 1, seq 1); back-pointer chain and named units are results of disk-reading callbacks, phrased as "for every applied record, callback returned true". **Semantic layer** (premise: every on-disk record originates from legal write side of this version, i.e. legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound); same (instance, counter) corresponds to unique content across all timelines): "result" is projection of root plus disk, not whole root—returned root's (instance, `checkpoint_txg`) and four new-root fields equal version j on chosen root's timeline (j not earlier than chosen root), remaining fields equal chosen root except unmount marker (reconstructed root marked "not written by unmount sequence"); on disk every version in (chosen root, j] has readable records, named units verified on both copies, sum of records not exceeding in-flight bound | Placement is I-8.3 (Strictly Continuous Replay Prefix)—its supplementary verdict clause is verbatim this rule ("prefix must not stop mid-publish—a publish is wholly applied or wholly unapplied"), do not create duplicate in [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md). Syntactic layer in Verus core `replay_core` `ensures` (`replay_follows_the_prefix_rules`), must-red worlds in `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` where "modify verdict" lines alter executable verdict only without touching spec; semantic layer judged within bound by legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound), Verus does not prove. Outside core: glue (flattening tables, wrapping callbacks, assembling reports and new root) covered by E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) calling real `replay_journal`, harness tests, and mutation lines in `crates/mutations.tsv`; non-projected fields in root assembly (unmount marker, instance table pointer, allocation record tree root) lack mutation lines, generator of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) writing fixed values; two disk-reading callbacks (back-pointer chain, named unit closure) covered by real callbacks in E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound), ramdisk read counts, mutation lines in `crates/mutations.tsv`, and read-only mount read-count cases |
| **M3 Idempotence and Monotonicity** | **Idempotence**: `Recover(Recover(c)) = Recover(c)`, proven in two readings—record-level (applying same record twice vs once yields identical root, verbatim I-8.4 (Replay Idempotence)) and fixed-point (replaying with reconstructed root as chosen root, same record table, same reader, same switches returns identical root; premise that records above chosen root do not exceed in-flight bound, and record table satisfies I-8.12 (Non-Decreasing Journal Record TXG for Same Instance): journal records for same instance have non-decreasing txg with counter); second run returning `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` with 0 records applied when chosen root `checkpoint_txg` saturates counts as equal. "Is just I-8.4 (Replay Idempotence)" does not hold: I-8.4 (Replay Idempotence) is record-level, fixed-point is distinct property. **Monotonicity** (corollary of M2, not proven separately): under serial commit (settled item 2), persistence order of D16 (Publish Semantics) settled item 7, and chosen root being newest valid root in root ring, for two fault-free crash states `c ⊆ c'` in same run, `Recover(c)` lands on version not later than `Recover(c')`, compared by (`checkpoint_txg`, instance), identical order to D22 (How Unit Atomicity Is Synthesized) settled item 7; reopened if concurrent commit enabled or domain widened to include faults. "Applied transaction set is subset of" reading withdrawn: false even for ideal recovery | Verus core `replay_core` `ensures` proves both readings: record-level and fixed-point (fixed-point premise includes I-8.12 (Non-Decreasing Journal Record TXG for Same Instance)); monotonicity written as corollary `corollary_the_landed_version_does_not_go_back`, root selection and write-side premises in `requires`. Adversarial family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) judges fixed-point within bound, violations strictly on tables violating I-8.12 (Non-Decreasing Journal Record TXG for Same Instance) (rerun 3 part 1). Checker side (I-8.4 (Replay Idempotence)) implemented, judged on secondary crash in crash injection outside pool-level checker list: `REPLAY_IDEMPOTENCE_INVARIANT` and `judge_the_remount_after_a_second_crash` in `crates/singlefs-checker-tier/src/crash_injection.rs`. O2 (Independent Parser + Checker) cannot judge idempotence: I-4.5 (Deprecated: Requires Two Images) retirement reason verbatim "idempotence requires comparing two images, cannot be judged on single image" |
| **M4 Not Under-Applying** | Premise and target identical to M2 semantic layer. Root returned by `Recover` is precisely j\*: farthest version from chosen root where every version is fully on disk (record has at least one self-verified copy, named units verified on both copies), and sum of records does not exceed in-flight bound. Without this, proving M1–M3 cannot exclude "confirmed complete publish discarded by recovery": a recovery applying nothing passes all literal clauses of M1–M3. "Not under-applying" read as "spurious violations": variant must yield spurious violation to be blocked | Farthest-version criterion of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) judged within bound; Verus must-red world line "stopping at first record above watermark and applying zero" goes red; M4 not proven separately |

**Mount Transparency** (independent fourth property, not third reading of M3): recovering read-only once on crash image yields R1; executing an action on same image—mount only (acquire number, write row, warm-up, mount continuation, and mount-initiated floor raising), or mount plus one publish—then recovering read-only again yields R2, comparing R1 and R2 field by field. Version compared: mount only compares to version recovered to, mount plus one publish compares to version immediately before post-mount publish. Also judged on secondary crash images img2 across three phases during mount: C(recover(img2)) = C(v1) (v1 is version landed by R1, C takes POSIX content and existence), judging both content and fields; in field half, falling back to v1 has fields identical to R1, landing on a version written by this mount judged per field-by-field table, scan report unjudged, floor raising F unjudged from first write onward, F prior to that (including warm-up) judged as "mount did not raise floor"; position of first floor raising write determined by location of floor-raising sequences in mount write table, unlocated (`NotLocatedInTheMountWrites`) marks whole-segment F unjudged. Field-by-field table (R2 against R1):

| Field | Judgment |
|---|---|
| POSIX content and existence | Must be identical |
| Pool identity and immutable config in system configuration (except local device number) | Must be identical; 37 immutable rows compared byte-for-byte except local device number |
| Node size in system configuration | Must be identical |
| Runtime config 36 bytes in system configuration (T_time, T_dirty, three watermarks) | R2 equals built-in default (`to_slot` writes defaults, lacking mount option channel) |
| Unmount marker | R2 constantly "not written by unmount", R1 marker excluded from comparison |
| F and effective F | Mount did not raise floor: R2 effective F and root F equal R1 effective F; raised floor: R2 effective F equals last raised floor from mount |
| Instance code | max(self-verified system config instances on both slots across disks, readable root instances in root ring) + 1, read from crash image |
| Instance table row | Written per D18 (Information Carried in a Block) settled item 11 recovery rows |
| `checkpoint_txg` | Lower bound only: ≥ max(root ring roots txg, ring self-verified records `checkpoint_txg`) + 1 (D23 (Role and Format of Journal) settled item 14 lower bound, allowing skipped numbers) |
| Root slot position | Not judged separately, determined by `checkpoint_txg` |
| Tree ID watermark | Identical when mount does not create first file |
| Root pointers of inode tree, extent tree, tombstones, lineage, deleted_inodes in tree table (latter three are tree table entries, kinds 9, 10, 11, not root record fields) | Identical; if mount resumed, judged by post-resumption version: equals closest step root pointers where txg ≤ R2; tree table unit landing point not compared |
| Root pointers, birth txg, head ID, tree ID, kind for tree table kinds 6, 7, 8, 12 | Must be identical (roots for kinds 6, 7, 8, 12 constantly all-zero); birth txg across roots judged by I-9.14 (Tree Table Entry Birth TXG Invariant Across Roots), head ID by I-9.27 (Tree Table Entry Head ID Value) on single image |
| Instance table and tree table unit pointers | Byte comparison omitted, same as tree table unit landing point |
| Scan report | 0 applied, 0 above watermark |
| Root slot offsets 428..457 (algorithm, nonce, MAC) | Must be 0; single-image property, judged by pool-level checker I-2.4 (Header Checksum Coverage), mount transparency does not compare |
| magic, flags remaining bits, self-verification checksum, whole-slot checksum | Not compared (structural, judged at slot read) |
| System configuration slot generation | Not compared, judged by I-7.16 (System Configuration Slot Generation and Slot Number Relation) on R2 image |
| System configuration journal_tail | R1 ≤ R2 ≤ max counter of self-verified records in R2 ring, equality holds when mount finishes |
| System configuration journal instance code | Equals R2 root instance code when mount finishes; secondary crash takes either R1 instance or new instance number |
| Allocation records (including root record `mapping_root`, `allocation_record_tree_root`, and tree table kinds 3, 4 roots) | Whole table comparison omitted: POSIX content identical and pool-level checker green on R2 image; `mapping_root`: R2 all-zero ⇔ R1 all-zero; `allocation_record_tree_root`: R1 has tree ⇒ R2 all-zero, R1 tree table 0 entries ⇒ (R2 non-zero ⇔ new instance number ≥ 2) |

System configuration runtime variables (slot generation, journal_tail, journal instance code) falling back to v1 are judged by their own row rules, not identical to v1.

**Scope**: Marginal value of refinement exists only in dimensions unattainable by crash-point replay enumeration—payload bytes, record counts / segment counts / txg counts (including ring wraparound), remnants of previous timeline, slot counts and device counts; crash-point dimension is already fully enumerated by crash-point replay. "Checksum passed" in model is boolean flag, **must not be read as refinement verifying integrity**—isomorphic to D4 (Checksum Placement) line in settled item 14 "must not read as verifying Merkle", and same informed acceptance regarding CRC32C collision probability as D13 (Verification Route) settled item 4. D13 (Verification Route) settled item 4 is `Crash` on left of refinement (enumeration domain of model layer), not `Spec` on right; treating it as spec makes refinement trivially true. Formalization of I-6.5 (Monotonic Nonce Watermark) withdrawn; reopened once D18 (Information Carried in a Block) settled item 14 settles and nonce allocation rules (monotonic counter + pre-allocated watermark, persisted before use) are established as settled item—when reopened, its sole true advantage is that nonces are invisible to POSIX, its spec structurally incapable of sharing origin with model of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)), immune to Conflict 2 (D13 (Verification Route) settled item 13).

Premises and domains for M1–M4: semantic layer hinges solely on "same (instance, counter) corresponds to unique content across all timelines", guaranteed by verbatim re-issue in D23 (Role and Format of Journal) settled item 14 "failure handling of this version"; re-checked when root slot write failure advancing `checkpoint_txg` lands; premise domain takes legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) (arbitrary landed subset of history written by legal writer, newer root slots may be unreadable). In M2 "remaining fields equal chosen root", root reconstructed from records has unmount marker `NotWrittenByTheUnmountSequence` (D22 (How Unit Atomicity Is Synthesized) settled item 7 "other roots write 0"), which may differ from chosen root marker, hence "except unmount marker". M3 monotonicity corollary holds only under serial commit, fault-free crash states. M4 together with root selection eliminates confirmed complete publishes discarded by recovery under complete premises: fault-free crash states (enumeration domain of settled item 4), chosen root per D22 (How Unit Atomicity Is Synthesized) settled item 7, chain not straddling instance boundaries (D23 (Role and Format of Journal) settled item 14 note 1), new instance roots written across two disks before any fsync returns (warm-up of D16 (Publish Semantics) settled item 8); not eliminated in faulty domain—in-flight bound truncates complete publishes, missing copy on dead disk prevents application (C519 (Losing Whole Disk Discards Just-Confirmed Version)).

Scope of mount transparency: applies only to fault-free crash states resulting in successful writable mount. Rejected writable mounts judged not for transparency, but legality of refusal: refusal legal ⇔ reason in D23 (Role and Format of Journal) settled item 14 list, recalculation holds on crash image, on-disk bytes identical, and read-only read in allowed set; unlisted reasons reported red per crash injection phase 2. Faulty states out of scope: unreadable slot at acquire time and N-config reported as same reason, separated before read-fault tier (C664 (Mount Refusal Legal Recalculation Conflates Two Refusal Reasons)). Tree ID watermark, two tree root pointers, `checkpoint_txg`, and scan report in field-by-field table are inferred; floor raising "raised" half unmeasured; `mapping_root`, `allocation_record_tree_root`, tombstones, lineage, deleted_inodes pointers, algorithm, nonce, MAC, slot generation, and journal_tail classified per C655 (Mount Transparency Field Table Has Unclassified Fields) and C687 (Placement Unjudged for Initial Tree Creation Version) verdicts Y1, Y2 in table.

**Basis**:
- E42 (How Many Records per Transaction): Payload pinned to make "split ⇒ illegal" an identity, structurally blind to "split yet coincidentally legal"—M2 symbolization of payload covers this gap.
- Three-party review: background `_d13-item1-background.md`, outputs `d13-item1-opus-forward-output.md`, `d13-item1-sonnet-reverse-output.md`, `d13-item1-local-attack-v2-output.md`; all three legs ranked former candidate last, cloud legs converging on "prefix judgment and commit atomicity", main agent combining root selection and publish atomicity into single unit (verdict and threshold in history).
- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound): Judges M2 semantic layer, M4, and M3 readings on real `replay_journal` within bound, measuring mutation kill bound; user ordered it as lightweight tool on 2026-10-01; C-domain crash state count divided into two cuts (cut A is barrier position of D16 (Publish Semantics) settled item 7); fixed-point violations in adversarial family strictly on record tables violating I-8.12 (Non-Decreasing Journal Record TXG for Same Instance), 0 violations when predicate holds (rerun 3 part 1); root-selection joint-verdict family calling real `choose_root`, `scan_journal`, `replay_journal` retains confirmed complete publishes within bound, legally and fault-free (rerun 3 part 2); four buggy root-selection variants on legal histories K, K9, KW show only "take first valid" red, other three red on adversarial root ring KA (rerun 3 part 3).
- Three-party verdicts (M1–M4 phrasing and placement): `research/prompts/m3-formal-r1-main-verification.md`, `research/prompts/m3-formal-r2-main-verification.md` Section 2, `research/prompts/m3-formal-r3-main-verification.md` Sections 2, 3; M3 monotonicity 4-step derivation in `m3-formal-r2-opus-output.md`.
- Three-party verdicts (M3 fixed-point premise adding I-8.12 (Non-Decreasing Journal Record TXG for Same Instance)): `research/prompts/m3-formal-code-r1-main-verification.md` Section 2 fixed-point precondition and Section 3 item 5; harness premise wider than Verus, reconciliation in scheduling record `records/2026-10-01-里程碑三形式化接入.md` batch 10.
- User decision (2026-10-01 popup, all per recommendation): unmount marker excluded, M4 scope premises stated fully, monotonic version order takes (`checkpoint_txg`, instance), txg saturation takes "report no_successor with zero applied counts as equal"; write-back deferred until K10 verdict, original wording in history.
- Mount transparency: no experiment: judged in crash injection (`crates/singlefs-harness/src/mount_transparency.rs`, called by checker tier crash injection), not experiment; attack numbers measured on replica harness across three verdict rounds, round 2 measured 147,806 initial crash states and 234,773 secondary crash states with 0 red on merged code without main agent rerun.
- Three-party verdicts (mount transparency): `research/prompts/m3-formal-r4-main-verification.md` Section 2 and Section 4 items 2, 3; `research/prompts/m3-formal-r5-main-verification.md` Section 2 U5, U6 and Section 4 items 5, 6; `research/prompts/m3-formal-r6-main-verification.md` Section 2 U5, U6 and untouched fields line; `research/prompts/m3-formal-code-r2-main-verification.md` Section 2 K2 lines.
- User decision (mount transparency plus secondary crash, 2026-10-02 popup, original wording in history): `research/prompts/m3-formal-r4-main-verification.md` user decision line.
- Root selection variant sentence: scheduling record `records/2026-10-01-里程碑三形式化接入.md` batch 12 E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) rerun 3 part 3 Q2 handback (main agent added this sentence).
- Mount resumption counted as part of mount, F bound located at end of write table: user decision 2026-10-03 "resumption is part of mount" (original wording in history); code round 3 verdict `research/prompts/m3-formal-code-r3-main-verification.md` Section 3 item 3; implementation `crates/singlefs-harness/src/mount_transparency.rs`.

**Owed**: C77 (Replay start undefined): at seam of root selection and prefix judgment; C649 (Two callback results unverified by Verus): paid off (M2 semantic layer back-pointers and named units remain postconditions unjudged by Verus, covered by mutation lines in `crates/mutations.tsv`, see settled owed table); C650 (Root selection and replay lack joint verdict): M4 scope sentence requires joint verdict to hold; C653 (Ring scan glue memory scales with disks); C654 (Must-red world expected lines drift with solver environment); C655 (Mount transparency field table has unclassified fields): paid off (table completed with padding per verdicts Y1, Y2, see settled owed table).

#### Settled Item 2: Serial Commit First

**Settlement**: Transaction layer commits to serial commit first—implementing `.claude/rules/fs-design.md` rule "when criteria cannot be written, implement only one path, and **implement the slow but correct path first**"; serialization is the explicit default of this project, not an arbitrary pick. Concurrency opens only past the intersection, never promised before it. Future concurrent commit constraint: records appended in transaction number order, fsync for transaction N must wait before return until transactions < N are all appended ∨ invalidated ∨ confirmed instance switch or read-only transition (invalidation is in-memory state of commit pipeline, visible to concurrent successors); this rule idles under serial commit.

**Scope**: Serial ceiling is ~11,000 entries per checkpoint (intersection where single-thread merge time catches up to one device sync, formula in C58 (Serial Commit Hitting Ceiling Lacks Alarm)): below intersection device sync is bottleneck and serialization suffers no penalty; above intersection single-thread merge becomes ceiling with zero safety margin. Settled item 2 does not decide record header field list or alter bytes written by new-pool file creation; it dictates performance ceiling and scope of crash refinement. Concurrent commit will affect crash semantics: "prefix" is total order concept; under concurrency, a prefix may contain complete transaction B alongside truncated transaction A; record header already carries transaction boundary fields (D23 (Role and Format of Journal) settled item 7: transaction number 8 + commit marker 1).

**Basis**:
- E17 (Write Buffer Merge Upper Bound): Single-thread sort-merge ceiling is half the intersection; lock-free parallelism per leaf partition is escape hatch past intersection, but assuming it sufficient is forbidden.
- E44 (Local Measurement and Cost of Sequence Width): Local fsync rate is other half of intersection.
- E42 (How Many Records per Transaction): Under serialization, multi-record transactions can still be split by prefix—record header field list dictated by whether transaction produces exactly one journal record, not serial vs concurrent.
- User decision, recorded in history; future concurrent constraint derived from C113 (Current Version Input Missing for Rebuild Multi-Version Units) settlement P1. Unreviewed by three-party: confirms explicit default; review required when switching to concurrency.

**Owed**: C58 (Serial commit hitting ceiling lacks alarm).

#### Settled Item 3: RefFS Taken to the Extreme, Handling Persistence and Crash Semantics Itself

**Settlement**: Refuses functional-only memory model, building a reference implementation (RefFS) handling persistence and crash semantics itself, elevating RefFS from "reserve candidate" to **active commitment**. "Taken to the extreme" means semantic level: RefFS operates in memory strictly via POSIX operations, defining what counts as persisted after each fsync and which POSIX states are permissible after crash, having no on-disk format; disk-writing and disk-recovering tier (effort approximating building another file system) is rejected. RefFS and implementation forbidden from sharing crash-point enumeration code: "three-oracle collusion test" (settled item 6) guards against shared misunderstanding, most critically "whether RefFS and implementation share identical flawed understanding of crash semantics".

Provenance and boundaries of RefFS:

| Item | Settlement |
|---|---|
| Provenance | Written from scratch per kb text only: does not copy code of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)), does not read recording stream, does not invoke segmentation, state planner, or crash enumeration of implementation |
| Placement | Isolated crate `crates/singlefs-reference-filesystem`: dependency closure (including dev-dependencies) ⊆ {itself, `singlefs-format`}, preferring std where possible; source code does not reference `singlefs_core`, `singlefs_checker`, `singlefs_harness`, `singlefs_checker_tier` (including renamed aliases); neither `singlefs-core` nor `singlefs-checker` include it in dependency closure. Judged by gate checker-independence-and-sync cell checker-implementation-disjoint |
| Input whitelist | Provided by caller, accepts five items only: outcome of each operation (success, rejected before write, errored after write, re-issued as-is); operation index where crash occurs (hi) and its attribution rule; rollback target m; content read on each recovery; explicit publish points |
| First-version operations | First file, overwrite, publish call, fsync, rollback |
| Call sites | At call sites of crash injection and fault injection, RefFS verdict conjoins with all four criteria of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge; any failure flags red |

Crash semantics (serial commit, fault-free crash states):

1. **fsync**: Upon successful return of fsync, all operations carried by the publish awaited by this fsync must be present (under serialization, all operations with acquired numbers before call began, counted in units); read by state index, raising lower bound lo of allowed set to end of that publish. Errored fsync does not raise lo, operations of errored publish optionally present; subsequent operations judged against state read on next recovery.
2. **Drop rule: prefix by publish boundary**. When crash occurs during operation hi, recovery permits reading {S_j : j = lo, or j is end of an explicit publish; lo ≤ j ≤ hi}, where S_j is state resulting from applying first j operations after previous recovery, and S_0 is content read on previous recovery. Dropping arbitrary subsets is excluded under serial commit (incompatible with D16 (Publish Semantics) settled item 12), reopened under concurrency.
3. **Raising lo**: Successful publish calls, fsync, and rollback all raise lo (D16 (Publish Semantics) settled item 7: rollback and fsync share return condition); rollback counts as confirmed publish, its index marking publish end.
4. **Rollback**: Rollback is an operation in RefFS; post-rollback content equals content following m-th confirmed publish across entire history; m passed in by caller from root reported by implementation, RefFS does not judge whether target selection is correct. m covers every root-writing entry point (publish call, rollback, write row and warm-up, raise F, unmount), mapping to active version when written, active version including that read on recovery and restarted.
5. **Attribution of crash moment**: Operation index hi where crash occurs is attributed by "which step issued the barrier cutting current segment", not by first intercepted write; used only by RefFS, O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge untouched.
6. **Post-recovery**: Following each recovery, RefFS first verifies that read state lies within allowed set, then advances from that state as starting point.

**Scope**: Settles ambition level and form of RefFS, not scheduling; must not be read as "building RefFS before implementation"—RefFS is verification vehicle, not milestone (`.claude/rules/format-evolution.md` "start from writing, not reading"). Explicit publish points take only publish calls and fsync in history: until automated batch accumulation lands, RefFS interfaces only with explicit publish points; automated publishes outside RefFS scope, belonging to settled item 1 M2 and record checker; recoveries discarding newest root cause RefFS to restart from read content without judging that step. Faulty crash states (lying devices, whole disk loss C519 (Losing Whole Disk Discards Just-Confirmed Version)) inapplicable. Independence: RefFS reads no recording streams, calls no segmentation or state planning, but its (lo, hi) shares origin with inputs to O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge disk-level lower bound criterion, logged under C191 (Common-Origin Counter Covers Only Single Pair). D17 (Implementation Layering and Third-Party Pipelines) settled item 4 delegates executable definition of "entry" to RefFS operation surface, fsync persistence defined by RefFS crash semantics; O3 (Independent Spec Executor) remains independent executor of crash sets with unchanged role. First version of RefFS in `crates/singlefs-reference-filesystem`, connected to crash injection phases and fault injection: harness tier depends on it via translation layer `crates/singlefs-harness/src/reference_filesystem_translation.rs`, call sites taking votes from both RefFS and O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge, flagging red if either fails.

**Basis**:
- No experiment: Ambition level, provenance, and inputs of RefFS are trade-offs between verification investment and boundaries; attack numbers from three review rounds measured on replica harness, unrepeated on main repo (`research/prompts/m3-formal-r5-main-verification.md` Section 5 item 3), uncited here.
- User decision, original wording in history: "take to the extreme" (original settlement); semantic level (2026-10-02, fork ticket `m3-formal-7-forks.md` RefFS level line, reinterpreting original settlement); prefix by publish boundary, call site conjunction, scratch rewrite, isolated crate (2026-10-02, `research/prompts/m3-formal-r4-main-verification.md`); conjunction tightened to all four criteria, rollback target as input (2026-10-02, `research/prompts/m3-formal-r5-main-verification.md`); rollback raises lo, crash moment attributed by barrier, m covers all root entries, whitelist adds re-issue (2026-10-02, `research/prompts/m3-formal-r6-main-verification.md`).
- Three-party verdicts: `research/prompts/m3-formal-r4-main-verification.md` (Round 1: fsync, drop rules, independence, six forks), `research/prompts/m3-formal-r5-main-verification.md` (Round 2: fsync synthesis sentence, publish boundary supplements, conjunction tightening, isolated crate, arbitrary subset rejected under serial), `research/prompts/m3-formal-r6-main-verification.md` (Round 3 final: rollback and lo, scope sentences, whitelist, fsync read by state index).
- D17 (Implementation Layering and Third-Party Pipelines) settled item 4: O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) explicitly states "ignores crash", incapable of defining fsync persistence promise, whereas first runnable target is "correctly committing one transaction"—extreme RefFS fills this exact half.

**Owed**: C447 (Spec executor not connected to crash-point replay to compare against RefFS); C191 (Common-origin counter covers only single pair); C658 (Crash injection third phase permits resurrection of dropped step).

#### Settled Item 4: Crash-Point Replay Enumerates Full Write Subsets Only

**Settlement**: The crash state set to be enumerated by crash-point replay is defined as: recorded write request stream is segmented **by device**—a write is ordered before subsequent writes only by subsequent barriers on its own disk (or FUA writes on its disk); current segment closes only when every disk with writes in the segment is released by its own barrier or FUA; if any disk has unreleased writes, that barrier (or FUA) does not close segment, and writes before and after belong to same segment. A crash state is "preceding segments all persisted, each write in current segment takes one of its states in arbitrary combination, subsequent segments all non-persisted"; enumeration unit is **one full write**. FUA writes release only their own disk: grouped in same segment with preceding unseparated writes on that disk, writes on other disks not persisted thereby. A write takes two states (persisted / non-persisted); **in-place overwrites** (non-unit writes longer than a sector covering pre-existing content, such as system configuration slot writes and journal writes overwriting old records) take a third state "neither new nor old readable", enumerated in both full run and quick tier; third-state images cover entire write range, differing bytes split with first half new bytes and second half old bytes (torn by byte, not sector); torn states of other writes collapsed into "write non-persisted". In "non-persisted" positions, images retain **pre-crash old bytes at that position**, not zeroes, and these images (including third-state) are concretely generated and fed to checker and record checker, never treated as mere counters. Mirrored writes (w ≥ 2 of D2 (RAID Striping Strategy) settled item 6, journal mirroring of D22 (How Unit Atomicity Is Synthesized) settled item 8) counted as distinct writes per **device**, harness state counts computed via closed form per write stream. **Adjacent barriers on same disk collapsed into one**: when recording stream contains consecutive barriers on a disk without intervening writes (barriers on other disks do not count as writes), second barrier omitted (`push` in `crates/singlefs-harness/src/lib.rs`), independent model recording identically; lacking intervening writes, segment produces zero crash states, merging affecting only operation count and kind string without altering crash state set.

**Scope**: Defines enumeration domain at model layer. Whether FLUSH / FUA take effect on real devices as advertised belongs to C6 (Block Layer Semantic Assumptions Flawed), verified in QEMU with real devices. Segmenting by device yields states unreachable in practice (writes released on other disks remain combinable in segment when one disk unreleased), without missing reachable states. Tearing falls into two categories: writes outside in-place overwrites (unit writes, sub-sector writes, writes over empty ranges) differ from "non-persisted" solely by CRC32C collision probability (32-bit, D19 (Block Pointer Structure and Width Budget) settled item 2, D23 (Role and Format of Journal) settled item 11), an **informed acceptance**; tear granularity excluded from model, consistent with D20 (Bearing Surface: Unit Atomicity and Self-Containment) "units with parent pointers depend on no width"; in-place overwrite tears lose old content as well, whereas "non-persisted" retains complete old content, representing distinct states, hence enumerated separately. Crash-point replay sectors take 512 bytes; system configuration slot writes are sole source of third-state on both crash replay streams. Known corner case: third-state old bytes take version where "writes prior to this write fully landed", combinations where earlier overlapping writes in same segment failed to land contain extra bytes (such combination occurs zero times across both streams); multi-sector root slot writes over old content do not support third state in first version (harness assertion; pools built with 512-byte physical blocks, unreachable). Device tearing writes to match checksum collisions excluded from model. D13 (Verification Route) settled item 5 orthogonal: one defines enumeration domain, other defines crate boundaries.

**Basis**:
- E77 (Publish Persistence Order): State count of segment model across four barrier arrangements matches closed form per arm—enumeration domain counted correctly.
- E142 (File Creation Dry Run on New Pool): Rerun 17 part 1 geometry sampling point G7 (positive control on same stream treating only FUA as boundary) measured 4097 states, 2046 violations—lacking barriers, units, journal records, and root slots merge into single segment `[12]`; harness segmentation aligns with `crates/singlefs-harness/src/segments.rs:91` "FUA write closes segment containing it (FUA does not persist preceding writes, grouped together in arbitrary subset)", predicting 2050 / 0 contradicts "FUA forms own segment"—FUA text written per implementation.
- Three-party adversarial review and verdict: [verification-build.md](../verification-build.md) "Three-Party Adversarial Review (2026-09-03)" verdict item 3—every unit class has whole-unit validation (parent-pointed rely on parent checksum, D4 (Checksum Placement) settled item 1, D16 (Publish Semantics) settled item 7, D22 (How Unit Atomicity Is Synthesized) settled item 19; self-verified rely on whole-unit checksum plus checked generation, D22 (How Unit Atomicity Is Synthesized) settled item 21; journal records rely on header checksum and payload checksum, D23 (Role and Format of Journal) settled item 13); torn and non-persisted are identical to checksum.
- User decision (settled per E77 (Publish Persistence Order) and three-party review; FUA segment boundary sentence settled at C313 (Whether FUA Write Counts as Crash Segment Boundary) closeout, updated 2026-09-26 to match implementation "closes containing segment, grouped with preceding unseparated writes"), recorded in history.
- Device segmentation (unreleased disk keeps segment open), FUA releasing local disk only, in-place overwrites taking third state "neither new nor old readable": user decision (code review items 1, 3, 4; 2026-09-27 popup selecting "A merge, track barriers per device", "backfill, run full run", and "in-place overwrite adds third state"); third-state byte-tearing settled by main agent same day, 0 rounds attacked, recorded in history; implementation and discriminative power in `m2-rev-b3a2-implementer-report.md` Sections 1, 3.
- Collapsing adjacent barriers on same disk: user settled 2026-09-28 "collapse, follow today's recorder" (E142 (File Creation Dry Run on New Pool) rerun 19 hit registered stop S19-crates: independent model recorded both, recorder collapsed second, warm-up and full stream differing in op count and kind string but identical in state count, report `e142-r19-runner3-report.md`, prereg `e142-r19-prereg.md` R40), original wording in history.

**Owed**: C6 (Block layer semantic assumptions flawed): reconstructing crash states from VM device logs, where crash tests must turn red.

#### Settled Item 5: Sharing Only a Single Constants Module Generated from kb

**Settlement**: Checker and implementation share exactly one artifact: a constants module generated from kb field tables, where generator reads kb and both sides consume without manual modification (`.claude/singlefs-ai-sop/rules/machine-first.md`: generate repetition, do not copy manually). **All else strictly unshared**: address space newtypes declared separately, format parsing implemented independently, checksums use independent implementations, traversal and accounting code share zero intersection (criterion for C12 (Incremental Semantics Shared) defined by gate checker-independence-and-sync cell checker-implementation-disjoint, enforced as crate-level dependency closure disjointness rather than symbol level; exception "except format parsing and constants" exercised by this item).

- **Decision width must not be obtained elsewhere**: Criteria for I-7.5 (Root Slot Aligned to Decision Width) and I-8.2 (Record Header Does Not Straddle Atomic Unit) depend on `physical_block_size` probed at mount, which is not in image. Checker probes independently, comparing against value recorded in system configuration at mkfs (checker-side form of D22 (How Unit Atomicity Is Synthesized) settled item 2 "mkfs bounds by pool maximum, mount recomputes and compares per device"). **When unprobed, report "declared value, unprobed", never treating as probed**—images backed by files fall into this class, comprising majority of crash-point replay images, hence non-trivial. In heterogeneous pools decision width is a set rather than scalar (C17 (Device Geometry as Global Parameter)), checker recomputing mkfs maximum aggregation to avoid false reds on healthy heterogeneous pools.
- **Generator refuses to emit constants lacking kb placement**, emitting scalar values only: if it emits "offset functions computed from field tables", it becomes format parsing explicitly forbidden from sharing by D13 (Verification Route); criterion is presence of branches or arithmetic in emitted code.
- **Independent CRC32C implementations require pinned parameters** (polynomial, initial value, input/output reflection, XOR output): unpinned parameters yield divergent results on healthy images, producing false reds rather than independence.

**Scope**: Governs crate boundary between checker and implementation; cannot catch erroneous values written in kb itself—shared generated constants and separate manual copying both miss that class, requiring orthogonal gate (C94 (Registered Format Constants Conflict with Later Settlement)). Implementation gap: `crates/singlefs-format` is handwritten rather than generated (values bound by gate code-source-discipline format-constants cell per `format-const` markers in kb, binding marked ones only), generator absent; `crates/singlefs-checker` depends solely on `singlefs-format`, rewriting bitwise CRC-32C; checker reads `physical_block_size` and `io_min` declared in system configuration (`geometry_of` in `crates/singlefs-checker/src/image.rs`), "probed / declared" types defined as `DecisionWidth` in `crates/singlefs-checker/src/lib.rs`: declared form (`DeclaredOnly`) connected to I-7.5 (Root Slot Aligned to Decision Width) judgment (`decision_width_declared_by_the_chosen_system_configurations` taking max(`physical_block_size`, `io_min`) across chosen configs), probed form (`Probed`) exists only as type definition without constructors or connected judgments; I-8.2 (Record Header Does Not Straddle Atomic Unit) unimplemented. `physical_block_size` field at mkfs resides in geometry segment of D22 (How Unit Atomicity Is Synthesized) settled item 9 (4 bytes).

**Basis**:
- No experiment: Sharing boundary is verification independence policy, lacking measurable quantities; discriminative power demonstrated by separate implementations flagging red individually.
- Three-party review: background `_checker-sharing-vehicle-background.md`, outputs `research/prompts/checker-sharing-vehicle-*`, detailed in [verification-build.md](../verification-build.md)—forward leg derived "constants within exception, rest forbidden" from D13 (Verification Route) and C12 (Incremental Semantics Shared); alternate reverse leg and local leg independently constructed finding: errors written in kb missed by both sharing and separate copying.
- User decision, recorded in history.

**Owed**: C17 (Device geometry as global parameter); C387 (Constants module handwritten, not generated from kb); C388 (Checker decision width not self-probed).

#### Settled Item 6: Three Mutually Independent Oracles

**Settlement**: Three oracles each judge distinct aspects, their independence deriving from different sources:

<!-- doc-lint:registry name-col=2 -->

| # | Oracle | What It Judges | Source of Independence | Blind Spots |
|---|---|---|---|---|
| O1 | In-memory model differential testing (HashMap ideal file system) | POSIX visible semantics: content, size, directory structure; content expectation without crash belongs here, permissible post-crash states belong to RefFS (settled item 3) | Touches no disk, parses no disk format | On-disk structural self-consistency; crash semantics; **shared misunderstanding of same POSIX semantics between two implementations** |
| O2 | Independent parser + checker | On-disk bytes satisfy I-1 (Block Self-Description) ~ I-6 (Encryption Invariants) | **Checker calls no deserialization/traversal code of tested crate**, implementing independent parsing directly reading raw device | Pure structural self-consistency only: overwriting block with alternate **valid** content leaves Merkle chain and accounting intact |
| O3 | Independent spec executor | Truncated disk state at each crash point falls within specification-allowed set | **Written in another language** (non-Rust), derived solely from decisions and invariants text; first version in Python (E170 (First Version of O3 Spec Executor)) | Spec itself may be flawed—**meaningful only with positive control**; projected values and unprojected root fields, implementation's own recovery/reader code, system config witness values (journal_tail), write order within publish |

**Independence must be verified, never self-proclaimed**: Design a "three-oracle collusion test"—feeding an intentionally flawed implementation (e.g. constant zero nonce) through all three; if all pass, a shared blind spot is revealed, requiring oracle enhancement. Functional correctness does not rely solely on these three: O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) delegates permissible post-crash states to RefFS (settled item 3), recovery function M2–M4 rests on Verus and E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) (settled item 1), and checks requiring second inputs belong to record checker (settled item 7); shared blind spots identified via "three-oracle collusion test".

**Scope**: Blind spots of each oracle tabulated in final column; domain of O2 (Independent Parser + Checker) belongs to settled item 7, dual-input checks to record checker. **Current implementation status**: O2 (Independent Parser + Checker) in `crates/singlefs-checker` (dependent only on format constants crate); O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) implemented as ideal model in `crates/singlefs-harness/src/model.rs` (memory-only, uses `singlefs_format` constants, models no crashes), gate harness-model-differential-and-scenarios running differential testing on random history; O3 (Independent Spec Executor) is Python executor written per kb text, active version being `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/` used in E171 (Three-Oracle Collusion Test) rerun 1 (E170 (First Version of O3 Spec Executor) rerun 2 copied `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` to `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` adding predicates W, T, then updated 487-byte journal header; rerun 1 archive and original `scripts/experiment-apparatus/e170_o3_spec_executor/` retained): strict execution judges successfully (0 states hit "spec missing clause" on batches 1, 2), state-for-state consistent with implementation (all pass, not evidence of correctness; "selected root not max txg among self-verified roots" and "③ received ≥ 2 slots" paths have 0 states, C661 (Spec Executor Lacks States on Two Paths)), bad state table has discriminative power under corrected expectations (positive control and blind test correct), not yet connected to crash-point replay; judging a crash state: independently recovers version from image per spec recovery rules (system config selection, root selection, prefix per D23 (Role and Format of Journal) settled item 14 five rules), judging not in set if unrecoverable (pool rejection or no self-verified root); projection of recovered version (instance, checkpoint_txg, tree table pointer, central mapping root pointer, tree ID watermark, F) must match projection of a version in version table, with index in [confirmed, started]. Version table consists of active version recovered on base image plus each root slot write in recording stream, projections matching bytes written by implementation; confirmed takes latest version returned before latest landed write; started takes latest version of landed write. System config slots used only to choose slot, determine geometries, and reject out-of-bounds pool fields. Recovery rules written per kb text, allowed set drawn from implementation's own written roots; RefFS (settled item 3) in `crates/singlefs-reference-filesystem`, connected to crash injection and fault injection; "three-oracle collusion test" is E171 (Three-Oracle Collusion Test), read from frozen tree (snapshot summary on experiment page, differs from active `crates/`): O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) column uses model differential plus model crash judge without RefFS (post-crash set half not using RefFS in E171 (Three-Oracle Collusion Test)); O3 (Independent Spec Executor) column uses E170 (First Version of O3 Spec Executor) rerun 1 frozen version; handcrafted bad images fed to O2 (Independent Parser + Checker) and O3 (Independent Spec Executor) only, expectations from E170 (First Version of O3 Spec Executor) designer, Q2 judging "present" per registration (all 4 are P8), judged per F3 as flawed expectation in E170 (First Version of O3 Spec Executor) rather than bad images, zero genuine bad images passed by both O2 (Independent Parser + Checker) and O3 (Independent Spec Executor); independence of origins (table col 4) measured only in Q2 between O2 (Independent Parser + Checker) and O3 (Independent Spec Executor), where patch and expectation derived from E170 (First Version of O3 Spec Executor) code, not treated as independent; O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) excluded from Q2; unmeasured aspects not claimed verified.

**Two aspects unjudged by all three oracles** (first aspect now has judge in crash injection): first, on fault-free crash states, D23 (Role and Format of Journal) settled item 14 writable mount witness criterion holds (witness in system config slot precedes readable records on disk): crash injection phase 2 verifies refusal legality by recalculating criterion on crash image, judging legal if true (mount transparency applies to fault-free states, settled item 1); judge required to ask "crash state lacks injected read fault, why does criterion hold": crash injection phases 2, 3 recalculate criterion on fault-free crash states, judging illegal refusal if true (`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults` in `crates/singlefs-checker-tier/src/crash_injection.rs`, C670 (No Judge for Writable Mount Witness Criterion True on Fault-Free Crash State) paid off, see settled table). Property "every disk has at least one self-verified system config on fault-free crash state" judged by G4 (crash injection phases 2, 3, connected alongside C670 (No Judge for Writable Mount Witness Criterion True on Fault-Free Crash State)): crash states before mkfs returns outside G4 scope, mkfs writing two slots per disk in single loop before disk barrier, crash injection candidate segments and E171 (Three-Oracle Collusion Test) base images chosen post-mkfs; G4 differs: when no system config selectable, recalculation yields `None`, G4 treats as true, read phase having rejected before reaching cell. Second, write order within publish: I-7.12 (System Configuration Floor Not Lower Than Root on Same Disk) of O2 (Independent Parser + Checker) judges step of writing system config first when raising floor; mutation cases test line 270 of E171 (Three-Oracle Collusion Test) (witness value at crash points post-publish return), line 242 case testing failure path freeze; `layout/01-first-txn.md` Section 8 registers root slot FUA and system config rotation as two segments; other write sequences unjudged by the three.

**Basis**:
- E170 (First Version of O3 Spec Executor): O3 (Independent Spec Executor) successfully implemented per "different language, derived solely from kb text, paired with positive control", judging single crash states fast enough for quick tier; rerun 1 (spec executor updated per 2026-10-03 kb) judged strictly, matching implementation on sample states with discriminative positive controls (unrunnable states and weak discriminative power of run 1 detailed on experiment page).
- G4 (every disk has at least one self-verified system config on fault-free crash state): Code round verdict `research/prompts/m3-formal-code-b26-r1-main-verification.md` Section 2 j2-b established, batch 29 implemented, scope and mkfs segment in `research/prompts/m3-formal-code-b29-r1-main-verification.md` Section 2 L3 and Section 3 item 3.
- Division of labor and independence sources of three oracles are architectural design of verification system; independence observed via "three-oracle collusion test" (E171 (Three-Oracle Collusion Test)).
- E171 (Three-Oracle Collusion Test): On frozen tree, flawed implementations table contains rows passed by all three oracles and rows caught by one oracle only, O3 (Independent Spec Executor) catching zero rows in flawed table; handcrafted forged roots and continuation images include cases passed by O2 (Independent Parser + Checker) but rejected by O3 (Independent Spec Executor) (readings on experiment page Q1, Q2, Q3).
- Three-party argumentation: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Division of labor between O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) and RefFS: three-party verdict `research/prompts/m3-formal-r5-main-verification.md` Section 2 isolated crate cell (both legs aligned).

**Owed**: C445 (Model differential sampling points do not scale with write paths); C446 (Unimplemented invariants lack precondition registry and expiry); C447 (Spec executor not connected to crash-point replay to compare against RefFS); C407 (Three-oracle collusion test judge not in gate): E171 (Three-Oracle Collusion Test) measured once, zero gate cells read output; C672 (Erroneous return value or memory state with unchanged write stream); C679 (Unmount marker excluded from model differential).

#### Settled Item 7: O2 (Independent Parser + Checker) Judges a Single Image

**Settlement**:

> **Settlement: The domain of O2 (Independent Parser + Checker) is a single image**—a unary predicate `P(image) → satisfied/unsatisfied`,
> judging set membership only, never identity. **Any check requiring a second input (another image or a recording stream) does not belong to it.**

**Checks requiring a second input belong to record checker**, with arguments `(pre-crash image, recording stream, post-crash image, persisted set)`—persisted set denotes writes landed in this crash state provided by enumerator (segment full write subset in crash-point replay); on-disk bytes alone cannot distinguish "write not landed, in-place content coincidentally identical" from "write landed", judging reuse and absence requiring persisted set; belongs to binary checks alongside mechanism 1 "Merkle root chain diff" (settled item 8). ⚠️ **Unnumbered by design**—confusion surrounding O2 (Independent Parser + Checker) was caused by using numbers as names; adding O4 merely provides another anchor for future drift. ⚠️ **"Post-crash image" consists of two copies, not one**: harness sequence executes implementation's recovery first, then record checker, recovery itself modifying disk (instance switch writing rows, re-issuing in-flight checkpoints, D23 (Role and Format of Journal) settled item 14) ⇒ root selection and prefix judgment inspect **crash-state image** ("recording stream" being journal records in its ring), compared against **implementation post-recovery image**.

**Scope**: In pointer layer target state, checking a record requires only format parsing + checksum + root ring selection; logical intent records name no on-disk objects, requiring checker to implement tree semantics + accounting + allocation validity—D23 (Role and Format of Journal) rejection of logical intent logs rewritten as guard on first reason in D23 (Role and Format of Journal) settled item 5. Record checker code in `crates/singlefs-checker-tier/src/crash.rs`, connected to each crash state in crash-point replay (gate checker-tier-crash-point-replay). Record checker judgment is thus not pure function of image: two states with identical post-crash bytes but different persisted sets may be judged differently, invalidating image-deduplication speedups (O2 (Independent Parser + Checker) and oracle inspect image only). Crash-point replay excludes writable mount: runs read-only recovery only (with and without journal) on each crash state; acquire number, write row, warm-up never run in crash-point replay. Writable mount covered by crash injection (`crates/singlefs-checker-tier/src/crash_injection.rs`): on each sampled crash state, following read-only recovery and judgment, mounts writable on same post-crash image, publishes once, runs pool-level checker; crashes again during mount, placing secondary crash state in acquire number, write row, warm-up phases, executing read-only recovery, querying model, pool-level checker, record checker: phase 3 write table passed to record checker switches from "prefix from mount segment to current segment" to full history stream + full mount stream + post-mount publish, persisted set corresponding segment by segment (history takes phase 1 persisted set, mount takes secondary crash subset, post-mount publish unlanded). "Compared against implementation post-recovery image": record checker accepts two images, crash-state image judging root presence and record presence (root selection and prefix), post-recovery image judging unit presence and reading landed instance table; phase 2 post-recovery image is pool after mount and publish, claimed version taking newest root written by mount and publish, falling back to version landed in phase 1 if mount wrote zero roots.

**Basis**:
- No experiment: **Basis is type, not cost**: "byte-by-byte comparison" is binary relation `R(image₁, image₂)`; **no amount of investment turns unary predicate into binary relation**; I-4.5 (Deprecated: Requires Two Images) retired on identical threshold.
- "Post-crash image" is two copies: settled by user during D16 (Publish Semantics) settled item 4 scope narrowing review (background `_d16-items134-background.md`), recorded in history.
- Persisted set as input: user decision, original wording in history; discriminator unobservability evidence in `research/prompts/m2-layer0-scale-r3-main-verification.md` Section 2 sector exemption line.
- Crash-point replay excludes writable mount, covered by crash injection: user decision (code review item 2, 2026-09-27 popup selecting "crash injection layer covers writable mount"), recorded in history; crash injection implementation in `m2-rev-b3b-implementer-report.md`.

**Owed**: None.

#### Settled Item 8: Two Custom-Developed Mechanisms: Merkle Root Chain Diff and Generational Incremental Scrub

**Settlement**: Two mechanisms unavailable off-the-shelf and custom-developed for this project:

**1. Merkle root chain diff —— Leverage unique to COW + new root per transaction.**
Historical roots physically exist prior to reclamation, forming replayable chain. Recursively comparing "subtree pointer + checksum/MAC" across adjacent root generations,
**directly skipping identical pointers without descending**, with complexity proportional strictly to actual modifications between generations.
Detects error class undetectable by traversal checker: **in-place writes self-consistent after synchronous tampering**—
a write bypassing COW modifying block in-place while synchronously updating checksum appears self-consistent in isolation (I-2.1 (Checksum Matches Content) all green),
whereas root chain diff exposes contradiction: **lacking COW, no new allocation should occur; parent pointer should remain unchanged, yet checksum altered**.
⚠️ **Must pair with negative control**: running same check post-normal COW commit must be all green, avoiding flagging normal updates as tampering.

**2. Generational incremental scrub.** Leveraging I-1.2 (Block Header Write Order Published) and D5 (Snapshot / Space Accounting Mechanism) birth txg
to maintain "generation watermark scanned in previous scrub", traversing only blocks with `birth generation > watermark`—
checksum chains of older subtrees remain self-consistent if unaltered. **Enables "verifying current change completely on every commit" within wall-clock constraints without sampling.**

**Scope**: Re-located replicas retain original `birth`, permanently falling below watermark, invisible to generational incremental scrub. Neither root chain diff (`grep -rn 根链 crates` yields 0 hits) nor generational scrub (`grep -rn scrub crates` yields 0 hits) exists in `crates/`.

**Basis**:
- No experiment: Both rely on structural properties of COW new root per transaction and birth txg; neither implemented, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: C389 (Merkle root chain diff has zero code); C194 (Generational incremental scrub misses relocated replicas); C408 (Generational incremental scrub has zero code).

#### Settled Item 9: Crash-Point Replay Must Be Tiered to Avoid Wall-Clock Explosion

**Settlement**:

| Tier | Scope | Trigger |
|---|---|---|
| Smoke tier | Fixed seed, fixed script workloads and registered crash enumeration cases, unconstrained by write request count (scope below), exact full run | Checker tier full run (settled item 15): run upon user request; commit runs quick tier by default |
| Regular tier | Moderate workloads (thousands of write requests), exact full run, incomplete enumeration handled by exact pruning, unrunnable nodes marked ignore, coverage report and gate listing "not run this time" entry by entry | Every commit |
| Heavy tier | Regular tier workloads + large workloads + multi-device cross matrix, N≥5 rounds, exact full run | User request / pre-release |

**What can be claimed after pruning plus ignore**: can claim "enabled nodes passed across all crash states";
**cannot claim** "ignored nodes passed".

**Full run is not run by default at commit, unconstrained by time, run upon user request; pre-commit = quick tier all green (necessary condition).**
**At milestone closeouts or batches modifying crash-point replay or judges, main agent displays popup asking user to run full run; commits permitted on failure or skip, logging owed check for red or "milestone closeout skipped full run" until resolved.**
"Full run" refers to two items: checker-tier-crash-point-replay `--full` (gate checker-tier-crash-point-replay, settled item 15 full run) and crash amplification full run (`scripts/crash/crash-amplification.sh` without arguments).
"Modifying crash-point replay or judges" calculated dynamically from crash verdict library: triggered if changes cause mismatch in library verdict block unit keys, with mismatched keys containing judge units (function units in closure of judge entry fn); gate reports invalidated block count and changed judge unit list, main agent prompting popup accordingly, avoiding path-based lists.
"Previous full run" recorded by main agent in milestone log: date, conclusions of both runs.
**Full run tier must prioritize "failure reproducibility" over "maximal coverage"**—fixing a full run bug takes wall-clock ≈ `log2(N) × single full run duration`; an irreproducible full run failure has near-zero or negative value, training developers to ignore red flags.

**Scope**: Only smoke tier exists (gate checker-tier-crash-point-replay); smoke tier divided into two modes: regular quick tier takes arbitrary subset for in-place writes and all-or-nothing for COW writes (settled 2026-09-26, designated "variant 2", used after fixing C561 (Record Checker Reuse Exemption False Red on Partial COW)), unable to detect bugs manifesting only under partial COW landing, incapable of replacing full run; full run with resumable checkpoints (gate checker-tier-crash-point-replay `--full`), registered crash enumeration cases in `.claude/gate.d/stage-inputs.tsv` growing to 12,230,590,578 states on parallel line 1 stream, far exceeding smoke tier scale of "two-digit write requests", hence commit runs quick tier by default, full run executed upon user request per settled item 15; regular tier and heavy tier lack gate stages. Defining crash states belongs to settled item 4; exact pruning and enabling ignored nodes belong to settled items 10, 11.

**Basis**:
- No experiment: Tiering is design constraint of gate wall-clock; workloads and runtimes for regular tier and heavy tier absent, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Regular quick tier uses variant 2: user decision, original wording in history; crash replay scale review `research/prompts/m2-layer0-scale-r3-main-verification.md`.
- User decision: "we do not sample, we prune plus initial ignore" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); "enumeration can set ignore flag and open slowly" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); "1. design complete enumeration, 2. incomplete enumeration is purpose of pruning" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); Round 1 verdict `research/prompts/m3-prune-gpu-r1-main-verification.md`: settled item 9 overturned.
- User decision (2026-10-01): Full run unconstrained by time, runnable anytime upon user request, removing "nightly", original wording in history.
- User decision (2026-10-01): Timing for full run all-green is milestone closeout and pre-commit of batches modifying crash replay or judges, prompted via main agent popup, original wording in history; delegated to user per three-party verdict `research/prompts/test-script-align-r1-main-verification.md` Section 2 t1-3.
- Three-party verdict `research/prompts/test-script-align-r2-main-verification.md` Section 2 U3: two meanings of "full run", path judgment for "modifying crash replay or judges", logging "previous full run", synthesizing "admission condition" and "async" into single rule.

**Owed**: C409 (Regular tier and heavy tier lack stages in gate); C635 (Previous full run lacks machine-readable marker).

#### Settled Item 10: Truly Required Exact Coverage N Is "Number of Commit Protocol Equivalence Classes", Not "Layout Schemes"

**Settlement**: Regular tier exact coverage reduces the problem before addressing pruning. Domain of crash points is the **transaction layer state machine**, governed by two top-level rules in `.claude/rules/fs-design.md`:
"One transaction layer, shared by all structures" and "The only layer forbidden from diversifying is the transaction layer". Thus layout can alter only two categories:

| Difference | What It Changes | Judgment |
|---|---|---|
| Node size, stripe width | Bytes written per step, repetition of step | **Parameter**, introduces no new step kinds |
| Rotational vs SSD | Seek cost, IO scheduling | Resides at device layer, **outside transaction layer protocol** |
| **Zoned (ZNS)** | **Ability to overwrite root pointer in-place** | **Structural**: append-only, drifting root, introduces zone finish/reset |

**Implemented via closed enum + exhaustive match**: `CommitStep` in `crates/singlefs-core/src/transaction.rs`, step kinds registered in segment sequence registry of [layout/01-first-txn.md](../layout/01-first-txn.md) Section 8.
Any layout introducing new structural steps **must modify this shared definition ⇒ compiler enforces branches at all call sites ⇒
diff is grepped by gate**, automatically tagging "this commit introduced new structural equivalence class, requiring independent heavy tier replay",
**without relying on human memory** (diff-based gate scope owed under C8 (Gate Scope Indeterminable)). Adding newtype isolation (`RotationalPtr` / `ZonedPtr`),
turning "forbidden sharing" bucket in D12 (Target Media) settled item 2 (accounting fact judgment paths) into compile-time constraint.
⚠️ **Second duty (added by D17 (Implementation Layering and Third-Party Pipelines)): serves as integration point for third-party pipelines.**
Protocol owned by this project; external pipeline introducing new structural commit steps must submit diff to this repository.

**Scope**: Rotational and SSD are isomorphic at commit protocol level, established by D12 (Target Media) three-party review; zoned line indicates it introduces at least one additional equivalence class. Equivalence relation isomorphic to recording stream segment sequence, count belonging to D17 (Implementation Layering and Third-Party Pipelines) settled item 2. Whether kind string identifies nodes adequately hit in Round 1 verdict (suspended, awaiting Rounds 2, 3): `research/prompts/m3-prune-gpu-r1-main-verification.md`.
⚠️ **Pins "which steps exist", but cannot pin "failure atomicity width of each step"**—
persistent memory and object storage alter failure atomicity width of each step, unhandled by enum (gap and remediation in D17 (Implementation Layering and Third-Party Pipelines)).
`RotationalPtr` / `ZonedPtr` newtype isolation absent in `crates/` (`grep -rn "RotationalPtr\|ZonedPtr" crates` yields 0 hits).

**Basis**:
- No experiment: Identity of N derived from `.claude/rules/fs-design.md` top-level rules, lacking measurable quantities; criteria for equivalence relation and counts in D17 (Implementation Layering and Third-Party Pipelines) settled item 2.
- Three-party review: Regular tier sampling round ("three-party review + empirical verification"), background in history.

**Owed**: C8 (Gate scope indeterminable).

#### Settled Item 11: When Ignored Nodes Are Enabled: Rejecting Ledger Rotating by Owed Checks, Retaining Three Rules

**Settlement**:

**"Ledger rotating by owed checks" scheduling: Rejected.** Scheduling based on cells = `(layout, substage_bucket)`, taking head of queue in descending order of owed checks, claiming any cell covered within `K_max = ⌈N×M/C⌉` commits is rejected: fails to guarantee precisely the slow, rare branches it aims to protect (empirical calculations and four rejection reasons in history).

**Three retained rules**:

1. **Correct direction**: Requires **deterministic coverage records capable of turning gates red**, rather than probabilistic arguments—
   probabilistic guarantees equal no guarantee in long-term maintenance.

<!-- doc-lint:registry name-col=2 -->

| Symbol | Short Name | Meaning |
|---|---|---|
| T1 | Regular tier wall-clock budget | Local wall-clock budget for regular tier, an input rather than output |

2. **`T1` (Regular Tier Wall-Clock Budget) is input, not output** (`.claude/rules/fs-design.md`: gate runtime is itself a design constraint).
   Correct sequence is "declare budget first → constrain how many ignored nodes single commit can enable", not "measure runtime → infer how many can open".
   **This rule requires no measurement, settled immediately.**
3. **Discriminative power must be checked separately from coverage**: Commit enabling an ignored node must run its checker against a "known corrupted image",
   flagging red if no error reported—direct reuse of existing positive control discipline in D13 (Verification Route). See C13 (Checker Judgment Ineffective).

Gate lacks "regular tier crash-point replay" stage, unable to measure `T1` (Regular Tier Wall-Clock Budget) / `C` / per-cell runtime.
⚠️ **Therefore `C`, `K_max`, `T1` (Regular Tier Wall-Clock Budget) must not be written into kb at this time.**

**Scope**: Governs when ignored nodes are opened; smoke tier full run enumeration does not require `T1` (Regular Tier Wall-Clock Budget). Rule 2 "settled immediately" conflicts with ending sentence "must not be written into kb", recorded in [verification-build.md](../verification-build.md) "Pending Decisions" question 6, unadjudicated. Node registry carries status field (open / ignore / awaiting implementation), plus three rules—each ignore row names mutating object, turning red if object turns green while node remains ignored, coverage report lists ignored state count and ratio: attacked 0 rounds (`research/prompts/m3-prune-gpu-r1-main-verification.md`), unwritten as settlement pending Rounds 2, 3.

**Basis**:
- No experiment: Rejecting ledger based on simulation of scheduling rules (200 rounds, steady-state maximum of second half, strict prefix greedy, numbers in history), unnumbered as experiment; retained three rules are gate design principles, lacking measurable quantities.
- Three-party review: Regular tier sampling round ("three-party review + empirical verification"), background in history.
- User decision: "enumeration can set ignore flag and open slowly" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); Round 1 verdict `research/prompts/m3-prune-gpu-r1-main-verification.md` overturned settled item 11, providing status field form and three supplementary conditions.

**Owed**: C8 (Gate scope indeterminable); C13 (Checker judgment ineffective).

#### Settled Item 12: Conflict 1: `Checker as Specification` vs `Verification Paths Sharing No Tools`

**Settlement**: **Conflict 1: Direct contradiction between `checker as specification` and `verification paths sharing no tools`.**
If active invariants (authoritative count in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) header) serve as both checker source and formal verification spec, they become single point of failure.
**Viable compromise, paid for in this project**: Checker implements two directions—**traversal** (descending from system configuration)
and **scan** (full disk scan, reconstructing from block headers under "metadata blocks must be self-describing"), both judging the same invariant independently,
sharing only format parsing.

**Scope**: Governs sharing between the two internal directions of checker; sharing between checker and implementation governed by settled item 5. Pool-level checker implements both directions (`crates/singlefs-checker/src/walk.rs` header).

**Basis**:
- No experiment: Trade-off between "checker as spec" and verification path independence, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: C410 (Checker two directions sharing lacks check): rule that two directions "share only format parsing" lacks checks turning red (no stage in `.claude/gate.d/`).

#### Settled Item 13: Conflict 2: Refinement Spec vs Differential Model

**Settlement**: **Conflict 2: Refinement spec and differential model are two expressions of the same entity.**
Merging into one → halves effort, but **directly violates independence requirements**; writing separately → nominal independence,
yet typically authored by same person in same week. Mitigation (reduces correlation without eliminating): ensure two versions **differ at representation level**—
spec phrased as "state predicates over images", model phrased as "op → state transition", facilitating mutual localization upon divergence.
**Common-origin must be observed**: Track counts of "spec and model diverge" events and which side was judged in error;
accumulating ≥ 5 divergences with 0 "spec error" verdicts indicates effective common origin, requiring merging paths and transferring budget to exhaustive testing.
Verdict has three categories: spec error / model error / clause missing sentence (each following valid interpretation where clause left ambiguity). Inconsistency re-evaluated within valid premise domain, recorded only if persisting; "clause missing sentence" recorded separately, excluded from "accumulating ≥ 5" and "spec error", re-evaluated and assigned to first two categories after clause updated.
Model here refers to O1 (In-Memory Model Differential Testing (HashMap Ideal File System)). Until O3 (Independent Spec Executor) delivers verdicts for same (history, crash point) (first version Python executor E170 (First Version of O3 Spec Executor) exists, unconnected to crash replay, C447 (Spec Executor Not Connected to Crash-Point Replay to Compare Against RefFS)), pair O3 (Independent Spec Executor) × RefFS registered as blank, logging zero events; O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge does not substitute; unestablished pairs report "unestablished", never reporting green or entering counts.

**Scope**: Mitigation reduces correlation without eliminating it. Former candidate withdrawn in settled item 1 unaffected, reasons in settled item 1 scope. Crash refinement code consists of Verus core and spec in `crates/singlefs-replay-prefix/src/lib.rs` (prefix extraction, root selection, ring scan, and joint corollary, covering settled item 1 M1, M2 syntactic layer, M3 readings and monotonicity; M2 semantic layer and M4 judged within bound by E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound)); spec cannot evaluate concrete states, provenance of divergence events undefined, counter lacks code (C191 (Common-Origin Counter Covers Only Single Pair)).

**Basis**:
- No experiment: Common origin observed via future divergence counts; spec and counter absent, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Third category "clause missing sentence": three-party verdicts `research/prompts/m3-formal-r1-main-verification.md` Section 2 "F2 common origin", `research/prompts/m3-formal-r2-main-verification.md` Section 2 line 9, `research/prompts/m3-formal-r3-main-verification.md` Section 2 common-origin counter line; no experiment: counter lacks event source.
- "Model refers to O1 (In-Memory Model Differential Testing (HashMap Ideal File System))" and unestablished pairs registered as blank: three-party verdict `research/prompts/m3-formal-r5-main-verification.md` Section 2 common-origin cell; `research/prompts/m3-formal-r4-main-verification.md` user decision.

**Owed**: C191 (Common-origin counter covers only single pair).

#### Settled Item 14: Interlocks with Other Decisions

**Settlement**:

| Decision | Interlock Point |
|---|---|
| D4 (Checksum Placement) | Root chain diff flips Merkle from "necessary overhead" to verification lever; in formal verification hashes are uninterpreted functions, **collision resistance completely outside verification scope, must not be read as "verifying Merkle"** |
| D8 (Core Index Structure) | Log-structured nodes, write buffer, key cache burden symbolic execution (loop bounds, lag windows, cache coherence), rendering "Yggdrasil within one minute" measurement basis invalid in this project |
| D9 (Encryption) | AEAD intractable in SMT, modeled as uninterpreted function; **consequence "nonce reuse leaks plaintext" unprovable**, leaving pure combinatorial property "nonces never repeat"—which is precisely the most valuable output of this leg |
| D11 (Whether to Reserve Message Buffer in Index Nodes) | Settled item 1 selects "reserve"; once D8 (Core Index Structure) settled item 1 adopted idempotent full values, checker I-3.1 (Allocated Statistics Match) need not parse message semantics (D11 (Whether to Reserve Message Buffer in Index Nodes) judging Cost 1 invalid in this project), preventing Conflict 1 from becoming intractable |

**Scope**: Interlocks define boundaries of formal verification and checker when touching these decisions, leaving clauses of those decisions unaltered; root chain diff and Conflict 1 belong to settled items 8, 12.

**Basis**:
- No experiment: Interlocks are boundaries deduced from settled clauses of those decisions, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: None.

#### Settled Item 15: Verification Code in Two Tiers —— Harness Tier Runs Anytime, Checker Tier Defaults to Commit

**Settlement**: Verification code under `crates/` divided into two tiers by "when it runs", each inhabiting a package, comprising three names alongside pool-level checker: **Harness tier** is `crates/singlefs-harness`, housing unit and integration tests plus scaffolding libraries (recorder, ramdisk, ideal model, random history, fault injection device, scenarios), run anytime code changes; internally split into light and heavy: debug runs taking ≥ 60s marked heavy with `#[ignore]`, measured during routine runs rather than isolated benchmarks, runnable anytime, uniformly wrapped in memory; one case per scenario, test files named by target rather than milestone. **Checker tier** is `crates/singlefs-checker-tier`, housing crash state enumeration engine, record checker, crash enumeration cases via verdict library across two phases and counter, discovery logs and toolchain identity, long-term verdict library (block keys determined by content, test types indexing needed blocks) and two-phase protocol, crash injection and bad-disk campaigns, real device log comparisons, experiment and real-device binaries, all corresponding cases, along with QEMU (gate checker-tier-qemu-device-streams), herd7 (checker-tier-lkmm), crates mutation table (checker-tier-crates-mutation-replay), and full experiment replays (checker-tier-research-build-and-replay experiment-replay cell): runs by default at commit only, runnable standalone (command carrying `SINGLEFS_HEAVY_TESTS=user-request`). Crash-point replay within checker tier (gate checker-tier-crash-point-replay) divided into two modes (others undivided): **Quick tier** consists of two passes: `cargo test --release -p singlefs-checker-tier --lib --tests` without `--ignored` (`--tests` including all `test = true` targets, including harness inline unit tests), plus pass with features touched by `cfg` in `tests/` (`--features <features> --lib --test <feature-gated-test>…`, features with `gpu` run via `scripts/crash/gpu-quiesce-run.sh`) (gate checker-tier-crash-point-replay runs both passes during full gate and commit, second pass being feature-gated-tests cell); **Full run** executes registered cases in `.claude/gate.d/stage-inputs.tsv` under key `crash-case:` entry by entry via `--include-ignored --exact` (checker-tier-crash-point-replay `--full`), skipped by default at commit, run upon user request per settled item 9; crash amplification judges migrated entirely to GPU, covering all pool-level checker invariants (`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`). **Pool-level checker** is `crates/singlefs-checker`, the verdict library called by both tiers. Dependencies permitted in single direction only: checker tier depends on harness tier and pool-level checker, harness tier never depends on checker tier (gate checker-independence-and-sync cell checker-implementation-disjoint item ④, dev-dependencies included); shared snippets duplicated separately. Crash enumeration cases written strictly in `crates/singlefs-checker-tier/tests/` (judged by `scripts/crash/crash-case-check.py`); heavy test gate identified by package: tests reaching `singlefs-checker-tier` belong to checker tier.

**Scope**: Governs package placement, execution timing, gate recognition, and naming of code and tests under `crates/` (`.claude/rules/verification.md` "Definitions and Names"); does not govern what each invariant judges ([invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)), enumeration domain definition (settled item 4), or sharing between pool-level checker and implementation (settled item 5: pool-level checker depends solely on `singlefs-format`). Upstream SOP no longer documents project verification methods: crash-point replay, model differential, checker-as-spec, and filesystem-specific deduction gaps consolidated into repository (`.claude/rules/verification.md`, `.claude/gate-not-implemented.tsv`), synced to replica without release. Tier splitting recorded in `records/2026-09-27-验证两档拆分.md`.

**Basis**:
- No experiment: Execution timing and package placement are process policies, lacking measurable quantities; discriminative power demonstrated by test cases and mutation tables.
- User decision (2026-09-27), original wording in history; three-party review `verification-split-r1` (`research/prompts/verification-split-r1-main-verification.md`) reviewed version prior to extracting `singlefs-checker-tier`, subsequent changes unattacked.
- User decision (2026-10-01): Full run unconstrained by time, runnable anytime upon user request, removing "nightly", original wording in history.

**Owed**: None.

#### Settled Item 16: Segments Exceeding u64 Judged in Slices, State Count Uses 1280-Bit Fixed-Width Integer

**Settlement**:
- Slicing: Segments registered by segment in crash amplification (`.claude/gate.d/crash-point-replay-segments.tsv`, five columns per row: item name, flow name, segment index, segment write count, segment write table digest) whose full domain state count exceeds u64 are judged sequentially in fixed-size slices; slice size determined by environment variable `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES`, defaulting to 2^32 states; slice start is resumption point in verdict library (indexed by write table digest and judge unit key; item name, flow name, segment index for display only). Segments fitting in u64 judged unsliced in single pass. Code locations: `crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`, `crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`.
- Bit width: Total state count, slice start, slice end, and progress for registered segments uniformly use 1280-bit (10 × u128) fixed-width unsigned integer `StateCount1280Bits` (`crates/singlefs-checker-tier/src/crash.rs:1640`, limbs defined by `STATE_COUNT_LIMBS` in same file), arithmetic fully checked, non-saturating, non-truncating; driver lines `registered_segment_progress=` report judged states and total states in exact decimal, never powers.
- Passing registry to engine: Engine reads segment registry strictly via environment variable `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` (`crates/singlefs-checker-tier/src/crash_amplification.rs:14764` in `fn registered_segments_of_the_flow_from_the_environment`), treating unconfigured as zero segments registered without fallback. Stream extending 144-unit file to 145 units growing lower extent segment to two levels (`position_addressed_ExtentLowerSegmentGrowsToTwoLevels`) registers both unit write segment (322 unit writes) and record segment (290 in-place journal writes).

**Scope**: Governs slicing, integer representations, and registry reading in crash amplification engine; does not govern which segments should be registered (approved by user, recorded in registry header), how segment domain expands (verdict J10 of `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md`), or how individual crash states are defined (settled item 4). Unregistered segments and whole-flow state sums must still fit in u64, rejecting execution before library write if exceeding; registered segments exceeding 1280 bits likewise rejected before library write. Slice size restricted to range 1 to 2^32: two GPU kernels add intra-slice offset as u32 to slice start (`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`); light tier callers (`scripts/crash/crash-point-replay.sh --quick`) default slice size to `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`. Segments exceeding expansion limit without registration fail amplification entry `scripts/crash/crash-amplification.sh`; crash-point replay, amplification, and item drivers verify table before passing to engine.

**Basis**:
- E161 (Deduplication and Segmentation Elapsed Time in Crash Amplification) "Rerun 2: Sampling Domain Reset to 11-Segment Stream (2026-10-05)": Full state counts match closed form, harness values, and crates plan segment by segment, growing exponentially with write counts, supporting premise that "long segment full domain exceeds u64"; slice size, bit width, and registry reading are engineering parameters settled by user, unsupported by E161.
- User decision, original wording in history: slicing and 2^32 states per slice (`records/2026-10-11-崩溃放量两段与脚本审计.md` line 96), 1280-bit width (lines 98, 99), registering both segments (line 92).
- Slicing and width supplement verdict J10 of `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md`, unattacked by three-party review.

**Owed**: None.

#### Settled Item 17: Unit of Verdict Reuse Is Function, Key Composed Solely of Function Units, Toolchain, Dependency Versions

**Settlement**:
- Verdict reuse across heavy gates, crash verdict library, and experiment admission (crash state blocks, quick tier and model differential test functions, crates mutation lines, broad mutations, formal verification, QEMU tiers, litmus suites, research reruns, experiment mutation tables, experiment admission) depends strictly on its unit key: de-commented fingerprints of dependent function units, plus `#[cfg]`, `#[cfg_attr]`, and `cfg!` atoms genuinely referenced within closures (including mod headers) paired with boolean values under current target (unreferenced atoms excluded: builds with/without `gpu` feature share identical key and library if `gpu` unreferenced in closure), toolchain identity (including cargo build environment and external tool versions), dependency versions resolved from `Cargo.lock` genuinely referenced by closure, and content inputs of the verdict itself (segment write tables, base image digests, litmus content, mutation originals and replacements). Function units collected from entry point along call and reference paths, including referenced constants, types, statics, and macro expansions; WGSL segmented by fn.
- Strictly excluded from keys and reuse criteria: diff ranges computed by commit, HEAD, or tree; entire file text; file-level, crate-level, or table-level fingerprints/closures; manual version constants; paths (including repository root), names, seed strings, line numbers, resource caps, timeouts; whole-stage or whole-cell flags lacking per-verdict libraries. Keys too narrow (missing referenced constants, types, macros), placing libraries in ephemeral paths, losing libraries on rename, or duplicate judging across two libraries constitute violations.
- Incomplete collections (dynamic dispatch, macros, unparsed `include_str!(concat!…)`) reject key computation and run normally, never silently omitting dependencies; each key persists alongside its component unit list, reporting which units changed upon invalidation.
- One library per verdict class, fixed in persistent directory, never changing directory across runs or evaluation methods; repository root is not input: crash verdict library directory accepts absolute paths only via `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`, rejecting relative paths; full run switch writes without reading, updating newly judged results into same library; second host results merged into local copy.
- Invalidating old libraries upon key changes incurs no migration cost: no compatibility migration or backfilled signatures, rerunning after changes.

**Scope**: Governs reuse keys for heavy gates, crash verdict library, and experiment admission; does not govern diff ranges for static gates checking added lines only (`scripts/gate/changed-paths.sh`) or preflight `admission: inputs-changed`; does not govern enumeration domain (settled item 4), tiering, or user prompts for full runs (settled item 9). Image verdicts and given-state verdicts keyed by verdict definition group (closures of CPU reference judges `crash::judge_crash_image`, `crash_given_states::judge_the_given_states_on_the_cpu`), regardless of CPU or GPU evaluation: GPU judge is accelerated path, admitted via differential testing against definition group (quick tier GPU vs CPU differential cases); its units enter block attributes as provenance rather than keys, undivided by kind; GPU card drivers, card models, and multi-card partitioning remain block attributes or run parameters, excluded from keys.

**Basis**:
- User decision (2026-10-15), original wording in history and `records/2026-10-13-复用键与全仓改名.md` Section "2026-10-15 User Settles: Commits Unrelated to Reruns, Reruns Follow Code Changes Only", "2026-10-15 Batch 3".
- User decision (2026-10-17): Verdicts unbound to hardware, CPU vs GPU; reuse and pruning ignore devices; original wording in history and `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 halting full run.
- Main agent decision (2026-10-15): Repository root excluded from keys and libraries, library path accepts absolute only, cfg components include genuinely referenced atoms only, recorded in same document decisions 1, 2.
- Counter-evidence: E164 (Small-Scope Functional Testing of New Crash Amplification Design) F4 demonstrated fingerprinting functions alone without referenced constants missed V5 and V11 (V11 modifying constant `DATA_POINTER_BYTES` only): function units must include referenced constants, types, statics, and macros.
- On-disk measurements: `research/prompts/pruning-survey-2026-10-15/ledger.md` Section 1.2—blocks differing only by name stored and judged separately across 4100 and 9679 blocks; altering single whole-library method digest rejudged 3,935,242,934 states, where 24 of 29 changed files in closure were verbatim identical after de-commenting: keys with names or file-level fingerprints trigger spurious re-judgments.
- Unattacked by three-party review; user settled opening three review rounds post-code changes, attacking strictly "whether pruning is atomic, whether keys comprise function units only".

**Owed**: C8 (Gate scope indeterminable), C489 (Broad mutation of singlefs-core lacks conclusion for current code), C528 (Two gate paths judge different change windows), C564 (Remaining 6 crash-point replay full runs unrun), C605 (Crash amplification reuse and pruning lack disabled control), C630 (Commit-time formal verification step 1 halt frequency unmeasured), C635 (Previous full run lacks machine-readable marker), C645 (Crash amplification full run lacks executor), C668 (Random history harness importer whitelist bypassable via alias), C777 (Five experiment keys lack registered lines in stage-inputs.tsv), C836 (Tree attached to staged-green not necessarily judged tree).

```

**出处 `.claude/kb/decisions/13-verification-route.en.md:444-447`（整段抄，未转述）**

```markdown
## Historical Versions

Historical items for D13 (Verification Route) are consolidated in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:1-6`（整段抄，未转述）**

```markdown
## D13 検証ルート —— 確定

[中文](13-验证路线.md) · [English](13-verification-route.en.md) · **日本語**

D13 (検証ルート) は本プロジェクトが正当性を何によって検証するかを司る：3種の既存手法をすべて実施し —— 独自参照実装の作成による差分突き合わせ、有界全数探索クラッシュテスト、crash refinement 形式検証（調査は [prior-art.md](../prior-art.md) 7.4 参照） —— その上で単なる模倣に留まらず、本プロジェクト固有の4点（トランザクションごとの新ルート発行を伴う COW、Merkle 自己検証、ボリューム全体の AEAD における nonce の一意性、参照実装の欠如）に応じた独自の検証手段を設計する。oracle をどう分割するか、独自開発する2点は何か、クラッシュポイント再生をどのように階層化およびサンプリングするか、クラッシュ状態集合をどう定義するか、checker と実装が何を共有するかを決定する。管轄外：各不変条件の規定内容は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) に委ねる；3種の検証手段の具現化手順と依存関係は [verification-build.md](../verification-build.md) に委ねる；コミット手順の登録場所、構造同値類の分割方針とクラス数は D17 (実装階層とサードパーティパイプライン) 確定項目 2 に帰属する。

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:7-444`（整段抄，未转述）**

```markdown
### 確定項目

| # | 項目 | 確定事項 |
|---|---|---|
| 1 | **crash refinement はどの部分を先に検証するか** | まず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」という閉じたユニットを検証する。第1マイルストーンは任意の `jsn` 連続プレフィックスがパブリッシュを分断しないこと；マウント透明性は第4の独立した性質として確立；旧候補 I-6.5 (nonce 水位の単調性) は撤回 **状態：確定。** **進捗：実装済み。** |
| 2 | **トランザクション層が直列コミットを公約するか** | まず直列とし、交差点を越えた後に並行を開く；将来の並行コミット時はレコードをトランザクション番号順に追加する **状態：確定。** **進捗：実装済み。** |
| 3 | **独自参照実装（RefFS 形態）をどの程度まで突き詰めるか** | セマンティクスレベルを極限まで突き詰める：メモリ内でのみ POSIX 操作に従って動作し、永続化およびクラッシュ後に許容される状態を独自に定義し、ディスク上フォーマットは持たない；kb に基づいてゼロから独立パッケージとして作成し、ホワイトリストの入力のみを受理する **状態：確定。** **進捗：実装済み。** |
| 4 | **クラッシュポイント再生のクラッシュ状態集合をどう定義するか** | 書き込みストリームをデバイス単位でセグメントに分割し、セグメント内で書き込みのあった各ディスクが自らのバリアまたは FUA で解放されて初めてセグメントを閉じる；セグメント内の各完全書き込みは状態を任意に組み合わせ、インプレース上書きには「新旧いずれも読み出せない」という第3の状態を追加；未永続化部分には旧バイトを配置し実際にイメージを生成する；ミラー書き込みはデバイスごとに個別に計算する **状態：確定。** **進捗：実装済み。** |
| 5 | **checker とテスト対象実装の間で共有を許容するもの** | kb から生成された定数モジュール1つのみを共有し、それ以外は各々独自に実装する；判定幅は checker が独自に検出しシステム構成と照合し、検出不能時は「宣言値、未検出」と報告する **状態：確定。** **進捗：実装済み。** |
| 6 | **互いに独立な3つの oracle** | インメモリモデル突き合わせ、独立パーサ + checker、独立規約実行器の3つの oracle が各々異なる側面を判定する；独立性は「3 oracle 結託テスト」によって検証し、自己申告を禁ずる **状態：確定。** **進捗：実装済み。** |
| 7 | **O2 (独立パーサ + checker) は何を判定するか** | O2 (独立パーサ + checker) の定義域は単一イメージであり、集合の要素性のみを判定する；第2の入力を必要とする検査はレコード照合器（入力パラメータに列挙器が提供する永続化集合を含む）に帰属させ、意図的に番号を付与しない **状態：確定。** **進捗：実装済み。** |
| 8 | **独自開発の2点** | Merkle ルートチェーン差分（隣接する2世代のルートを再帰的に比較し、ポインタ不変箇所には掘り下げない）および世代間増分 scrub（birth generation が水位より高いブロックのみ走査） **状態：確定。** **進捗：未実装。** |
| 9 | **クラッシュポイント再生をどう階層化するか** | 3層とも正確なフル実行とし、列挙しきれない場合は正確な枝刈りに依拠し、直ちに実行できないノードには ignore を付与して順次開放する；マイルストーン完了および判定器ユニット変更を伴うコミット前にはポップアップでユーザーにフル実行層の実行を促し、失敗または未実行でもそのままコミット可能とし未済として記録；まず失敗再現性を確保する **状態：確定。** **進捗：実装済み。** |
| 10 | **真に正確に網羅すべき N は何か** | コミットプロトコル構造の同値類数であり、レイアウトのセット数ではない；具現化手段は閉じたコミット手順列挙と網羅的な match である **状態：確定。** **進捗：実装済み。** |
| 11 | **ignore ノードをいつ開放するか** | 未済の検査のローテーション台帳を否決；開放の契機を規定する3条を残す：決定論的なカバレッジ記録、`T1` (通常層 実時間予算) を入力とする、判別力とカバレッジを分離して検査する **状態：確定。** **進捗：対象なし。** |
| 12 | **衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する** | checker はトラバーサルとスキャンの2方向を実装し、同一の不変条件に対して各々独立に判定し、共有するのはフォーマット解析のみとする **状態：確定。** **進捗：実装済み。** |
| 13 | **衝突 2：refinement の spec と突き合わせの model** | spec と model は表現形式のレベルから異なって記述する；同源性を観測し、不一致が5回以上かつ spec の誤りが0回の場合は統合して予算を全数探索へ移管する **状態：確定。** **進捗：対象なし。** |
| 14 | **他の決定との連鎖** | ハッシュと AEAD は形式検証において uninterpreted function としてのみ扱える；ログ構造の3層がシンボリック実行を拘束する；checker はメッセージセマンティクスを解析する必要はない **状態：確定。** **進捗：対象なし。** |
| 15 | **検証コードを2層に分割** | harness 層（`singlefs-harness`）は単体および結合テストであり常時実行；checker 層（`singlefs-checker-tier`）は既定でコミット時にクイック層のみ実行し、フル実行はユーザー要求時に実行する **状態：確定。** **進捗：実装済み。** |
| 16 | **セグメント単位で登録されたセグメントの分割方法と状態数のビット幅** | 登録セグメントの全領域が u64 に収まらない場合はスライスごとに順次判定し、スライスサイズは既定で 2^32 状態、スライス起点はライブラリ内の再開点とする；状態数は一律に 1280 ビット固定幅整数を用い、演算はすべて checked とする；登録表は環境変数経由でのみ読み込む **状態：確定。** **進捗：未実装。** |
| 17 | **判定再利用のユニットは何か** | 枝刈りのユニットは関数である：ある判定を踏襲するか否かは、それが依存する関数ユニットの各コメント除去後指紋、ツールチェーン、解決された依存バージョンのみを参照する；コミットやツリーによる変更計算、ファイル全体の原文、ファイル単位やパッケージ単位の指紋、手書きバージョン番号、パス名のキーへの導入、工程全体のマークは一律に禁止する **状態：確定。** **進捗：未実装。** |

#### 確定項目 1：crash refinement はまず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」を検証する

**確定事項**：crash refinement の最初の検証対象はリカバリ関数 `Recover` の関係的性質であり、最小の閉じたユニットは「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」である。4つのマイルストーンの主語 `Recover` は、`crates/singlefs-core` 内で `verify_named_units = true` を指定して `replay_journal` を呼び出している3箇所（`recover` の `Consult` 戦略、書き込み可能マウントの読み出しフェーズ、読み取り専用マウント）を指し、3箇所の `records` はすべて `scan_journal` の出力を取得する；`Ignore` および `ConsultWithoutNamedVerification` はテストおよび対照用のアームであり、トラステッドコンピューティングベースに配置する；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` はプロダクションの呼び出し元ではない。4つのマイルストーンは順に以下の通りである：

| マイルストーン | 義務 | 配置先 |
|---|---|---|
| **M1 全域性** | `Recover` が D13 (検証ルート) 確定項目 4 で列挙された**すべての**クラッシュ状態において定義され、停止し、I-7.1 (ルートスロット有効集合の非空性) および I-8.3 (再生プレフィックスの厳密な連続性) を満たすイメージを出力すること | スキャフォールド。**完了の判定基準は「証明されたこと」ではなく「M2 をその上に記述できること」である** —— クラッシュポイント再生は元よりフル実行で列挙しており（[verification-build.md](../verification-build.md)「クラッシュポイント再生はサンプリングせず、フル実行で列挙」）、リカバリのハングはその場で露見するため、全域性単独を証明することに限界価値はない。プレフィックス取得、ルート選択、リングスキャンの3区間はすでに Verus に導入され、すべて `crates/singlefs-replay-prefix/src/lib.rs` にある：プレフィックス取得コア `replay_core`（停止性と非オーバーフローはコア内で証明済み、ループはすべて `decreases` を伴う；2つの `requires` —— レコード表が (インスタンス番号, カウンタ) 昇順、カウンタに後続が存在 —— はグルーコード `replay_journal` 内のアサーションで担保）、ルート選択コア `choose_root_slot`（自己検証済みのスロットのうち (`checkpoint_txg`, インスタンス番号) 最大のものを取得、同値時はスロット順最前を取得）、リングスキャンコア `first_copy_of_each_record_key`（同一 (インスタンス番号, カウンタ) は先着順）、ルート選択とプレフィックス取得の結合判定は系 `corollary_recovery_replays_from_the_newest_self_verified_root` として記述；ゲート checker-tier-formal-verification が証明を判定し、固定数は 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) を軽量ツールとして用い、その第3回再実行第2区間のルート選択結合判定ファミリが ramdisk 上で本物の `choose_root`、`scan_journal`、`replay_journal` を呼び出す。ルート選択順序の4つの改悪変種のうち、正当な履歴 K ファミリで赤となるのは「最初の有効なものを選択」のみであり、他の3種（比較時にインスタンス番号を無視、txg 同値時に低位を選択、インスタンス番号主導）は Verus 必赤世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` のルート選択の節）および敵対的ルートリング KA でカバーされる（第3回再実行第3区間）。コア外：システム構成選択（`choose_system_configuration`）および2つのグルーコード（ルート選択のスロット読み出し、自己検証チェックサム、fsid、領域からディスクへのマッピング、スロットの走査順序；リングスキャンのオフセット列挙、ヒント、パースと CRC、ディスクの走査順序）は Verus の対象外であり、クラッシュポイント再生およびクラッシュ増幅の CPU 判定器が各クラッシュ状態上で `recover` を呼び出してこれらをカバーする（GPU 判定器は WGSL で `replay_journal` を別途記述しており、core は検証できない） |
| **M2 プレフィックスがパブリッシュを分断しない** | **任意の `jsn` 連続プレフィックスについて、その再生結果はある整数個のパブリッシュ済み checkpoint の結果と等しくなる —— プレフィックスによって分断されるパブリッシュは存在しない；プレフィックスがあるパブリッシュの途中で停止した場合、そのパブリッシュは全体として適用されない**（D16 (パブリッシュセマンティクス) 確定項目 4「1回のパブリッシュは全体として適用されるか全体として適用されないかのいずれかである」）。2層に分かれる。**構文層**（任意の自己検証済みレコード表に対して、前提なし）：適用されるレコードはいくつかの完全なパブリッシュであり、インフライト上限とコミットマーカーが1条ずつ判定され、適用されるカウンタはチェーン先頭から順に連続する（アンカーが読み出せる場合はアンカー + 1 から開始；読み出せない場合、先頭の `checkpoint_txg` = 選択されたルート + 1、シーケンス番号 1）；逆方向チェーンおよび指名ユニットはディスク読み出しコールバックの結果であり、「適用された各レコードについて、コールバックが真を返した」と記述される。**セマンティクス層**（前提：ディスク上の各レコードはこのバージョンの正当な書き込み側に由来する、すなわち E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリに属する；同一の (インスタンス番号, カウンタ) はすべてのタイムラインを通じて唯一の内容に対応する）：「結果」はルートの射影 + ディスクであり、ルート全体ではない —— 返却されるルートの (インスタンス番号, `checkpoint_txg`) および新ルートセグメントの4フィールドは、選択されたルートのタイムライン上の第 j 版（j は選択されたルートより前ではない）と等しく、その他のフィールドは選択されたルートと等しい（アンマウント記号を除く；レコードから再構築されたルートは「アンマウントシーケンスで書き込まれたものではない」とする）；ディスク上の (選択されたルート, j] の各バージョンのレコードが読み出せ、指名ユニットの両方のコピーが検証済みであり、レコード件数の合計がインフライト上限を超えない | 配置先は I-8.3 (再生プレフィックスの厳密な連続性) —— その判定補足文は逐語的に本ルールそのもの（「プレフィックスがパブリッシュの途中で停止してはならない —— 1回のパブリッシュは全体として適用されるか全体として適用されないかのいずれかである」）であり、[invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 内に別途新設してはならない。構文層は Verus コア `replay_core` の `ensures`（`replay_follows_the_prefix_rules`）内にあり、必赤世界は `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` に登録され、そのうち「判定変更」行は実行可能判定のみを変更し spec には触れない；セマンティクス層は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリによって有界内で判定され、Verus では証明しない。コア外：グルーコード（テーブル展開、コールバックのラッピング、レポートと新ルートの組み立て）は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) が直接本物の `replay_journal` を呼び出すこと、harness テストケース、および `crates/mutations.tsv` のミューテーション行でカバーする。新ルート組み立て時の射影外フィールド（アンマウント記号、インスタンス表ポインタ、割り当てレコードツリーのルート）にはミューテーション行がなく、E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) のジェネレータがこれらを固定値として記述する；2つのディスク読み出しコールバック（逆方向チェーン、指名ユニットクロージャ）は、E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) 内の実コールバック、ramdisk の読み出し回数、`crates/mutations.tsv` のミューテーション行、および読み取り専用マウントの幾つかのパターンの読み出し回数テストケースでカバーする |
| **M3 冪等性と単調性** | **冪等性**：`Recover(Recover(c)) = Recover(c)`、両方の解釈を証明する —— レコードレベル（同一レコードを2回適用した結果と1回適用した結果で得られるルートが同一であること、I-8.4 (再生の冪等性) の文面）および不動点（再構築されたルートを選択されたルートとし、同一のレコード表、同一のリーダー、同一のスイッチで再度再生した際、返却されるルートが不変であること；前提として選択されたルート以上のレコードがインフライト上限を超えず、かつレコード表が I-8.12 (同一インスタンスレコードの txg 非減少) を満たす：同一インスタンス内の journal レコードの txg はカウンタに伴って減少しない）；2回目で選択されたルートの `checkpoint_txg` が上限に達して `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` を報告し、1件も適用されなかった場合は同等とみなす。「すなわち I-8.4 (再生の冪等性) である」は成立しない：I-8.4 (再生の冪等性) はレコードレベルの1解釈であり、不動点は別の独立した性質である。**単調性**（M2 の系、個別には証明しない）：直列コミット（確定項目 2）、D16 (パブリッシュセマンティクス) 確定項目 7 の永続化順序、選択されたルートがルートリング内で最新の有効なルートであるという条件下で、同一実行内の D13 (検証ルート) 確定項目 4 の故障を含まない2つのクラッシュ状態 `c ⊆ c'` について、`Recover(c)` が到達するバージョンは `Recover(c')` が到達するバージョンより遅くならず、バージョンは (`checkpoint_txg`, インスタンス番号) の順で比較され、D22 (ユニットの原子性をどう合成するか) 確定項目 7 の最新選択順序と同一順序である；並行コミットを導入する場合、またはドメインを故障を含む状態に緩和する場合は再審議する。「適用されたトランザクション集合が包含される」という解釈は撤回：理想的なリカバリに対しても偽となるためである | Verus コア `replay_core` の ensures で両方の解釈を証明：レコードレベルおよび不動点（不動点の前提条件には I-8.12 (同一インスタンスレコードの txg 非減少) を含む）；単調性は系 `corollary_the_landed_version_does_not_go_back` として記述し、ルート選択と書き込み側の2つの前提はその requires に記述する。E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の敵対的ファミリが有界内で不動点を判定し、違反はすべて I-8.12 (同一インスタンスレコードの txg 非減少) が成立しないレコード表においてのみ発生する（第3回再実行第1区間）。checker 側（I-8.4 (再生の冪等性)）は実装済みであり、クラッシュ注入の二次クラッシュ上で判定され、プールレベル checker のリストには含めない：`crates/singlefs-checker-tier/src/crash_injection.rs` の `REPLAY_IDEMPOTENCE_INVARIANT` および `judge_the_remount_after_a_second_crash`。O2 (独立パーサ + checker) は冪等性を判定できない：I-4.5 (廃止済み・2イメージ必要) の廃止理由は逐語的に「冪等性は2つのイメージを比較する必要があり、単一イメージでは判定できない」である |
| **M4 不足適用の防止** | 前提と対象は M2 のセマンティクス層と同一。`Recover` が返却するルートはまさに j\* である：選択されたルート以降の各バージョンがディスク上に完全に存在し（レコードのいずれかのコピーが自己検証済み、指名ユニットの両コピーが検証済み）、かつレコード件数の合計がインフライト上限を超えない最も遠いバージョンである。この条項を付加しない場合、M1–M3 をすべて証明したとしても「確認済みの完全なパブリッシュがリカバリによって破棄される」ことを排除できない：1件も適用しないリカバリであっても M1–M3 の文面をすべて通過してしまう。「不足適用の防止」は「追加の違反の発生」として解釈する：変種が追加の違反を生じさせて初めて、本条項によって阻止されたとみなす | E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の最遠バージョン判定基準によって有界内で判定する；Verus の必赤世界において「水位を超える最初のレコードで停止し、1件も適用しない」行が赤となる；M4 自体は単独では証明しない |

**マウント透明性**（M3 の第3の解釈ではなく、独立した第4の性質）：クラッシュイメージ上で読み取り専用リカバリを1回実行して R1 を得る；同一のイメージ上でアクションを1つ実行 —— マウントのみ（番号取得、行書き込み、ウォームアップ、マウント継続実行、およびマウント自体の押し上げによる F 引き上げ）、またはマウント + 1回のパブリッシュ —— その後、再度読み取り専用リカバリを実行して R2 を得て、R1 と R2 をフィールドごとに比較する。比較対象のバージョン：マウントのみの場合はリカバリが到達したバージョンと比較し、マウント + 1回のパブリッシュの場合はマウント後のパブリッシュの直前のバージョンと比較する。マウント途中の3フェーズにおける二次クラッシュイメージ img2 上でも判定する：C(recover(img2)) = C(v1)（v1 は R1 が到達したバージョン、C は POSIX の内容および存在性を取得）、内容とフィールドの両半面を判定する；フィールド半面において、v1 のバージョンにフォールバックした場合は各フィールドが R1 と同一であり、今回のマウントによって書き込まれたいずれかのバージョンに到達した場合はフィールド別表に従って判定し、スキャンレポートは判定せず、F 引き上げの最初の書き込み以降は F を判定せず、それ以前（ウォームアップを含む）は「マウントが F 引き上げを押し上げなかった」セルに従って判定する；F 引き上げの最初の書き込み位置は、マウントが返却した F 引き上げシーケンス（受入で押し上げられたものおよび継続実行で空間のために押し上げられたものの双方）のマウント書き込み表末尾における位置に基づいて取得し、取得不能時（`NotLocatedInTheMountWrites`）は今回のマウントの二次クラッシュ区間全体の F を未判定とする。フィールド別表（R2 対 R1）：

| フィールド | 判定法 |
|---|---|
| POSIX 内容および存在性 | 一致必須 |
| プール識別情報およびシステム構成の不変構成（自ディスクデバイス番号を除く） | 一致必須；不変階層の37行は自ディスクデバイス番号を除き一律に生バイトで比較 |
| システム構成のノードサイズ | 一致必須 |
| システム構成の実行時構成 36 バイト（T_time、T_dirty、3水位） | R2 は組み込み既定値に等しい（マウントオプションチャネルがなく、`to_slot` は常に既定値を書き込む） |
| アンマウント記号 | R2 は常に「アンマウントによる書き込みではない」であり、R1 の記号は比較対象外 |
| F および F 有効値 | マウントが F 引き上げを押し上げなかった場合：R2 の F 有効値および R2 ルート内の F はいずれも R1 の F 有効値に等しい；押し上げた場合（受入によるものと継続実行で空間のために押し上げたものの双方を含む）：R2 の F 有効値はマウントが返却した最後のシーケンスが引き上げた F に等しい |
| インスタンス番号 | max(各ディスクの2スロットの自己検証済みシステム構成のインスタンス番号, ルートリングの全可読ルートのインスタンス番号) + 1、双方の量をクラッシュイメージから読み出す |
| インスタンス表の行 | D18 (ブロックに含める情報) 確定項目 11 に従いリカバリが書き込んだ行 |
| `checkpoint_txg` | 下限のみ判定：≥ max(ルートリングの全ルートの txg, リング内の全自己検証済みレコードの `checkpoint_txg`) + 1（D23 (journal の役割とフォーマット) 確定項目 14 の下限、番号飛びを許容） |
| ルートスロット位置 | 個別には判定せず、`checkpoint_txg` によって決定 |
| ツリー ID 水位 | マウントが最初のファイルを発行しない場合は同一 |
| ツリーテーブル内の inode ツリー、extent ツリー、墓碑、系譜、deleted_inodes のルートポインタ（後者3つはツリーテーブルエントリであり種別 9, 10, 11、ルートレコードのフィールドではない） | 同一；マウントが継続実行された場合は、継続実行後のバージョンに従って判定：マウントが返却した継続実行の各ステップのうち txg が R2 のバージョン以下の最新ステップの各ルートポインタに等しい；ツリーテーブルユニット自体の配置先は比較しない |
| ツリーテーブル種別 6, 7, 8, 12 のルート、誕生 txg、ヘッド ID、ツリー ID、種別 | 一致必須（種別 6, 7, 8, 12 のルートは常にオールゼロ）；誕生 txg のルート間比較は別途 I-9.14 (ツリーテーブルエントリの誕生 txg のルート間不変)、ヘッド ID は別途 I-9.27 (ツリーテーブルエントリのヘッド ID の値) が単一イメージ上で判定 |
| インスタンス表およびツリーテーブルのユニットポインタ | バイト比較は行わず、「ツリーテーブルユニット自体の配置先を比較しない」と同様とする |
| スキャンレポート | 適用 0 件、水位超過 0 件 |
| ルートスロットオフセット 428..457（アルゴリズム、nonce、MAC） | 0 であることが必須；単一イメージの性質であり、プールレベル checker の I-2.4 (ヘッダチェックサム対象範囲) が判定し、マウント透明性では比較しない |
| magic、flags の残りビット、自己検証チェックサム、スロット全体チェックサム | 比較しない（構造的性質であり、スロット読み出し時に判定） |
| システム構成スロット世代番号 | 比較せず、I-7.16 (システム構成スロット世代番号とスロット番号の関係) に委ねて R2 のイメージ上で判定 |
| システム構成の journal_tail | R1 ≤ R2 ≤ R2 リング内の自己検証済みレコードの最大カウンタ、マウント完了時は等号成立 |
| システム構成の journal インスタンス番号 | マウント完了時は R2 ルートのインスタンス番号と等しい；二次クラッシュ時は R1 のインスタンス番号または新インスタンス番号のいずれかを取る |
| 割り当てレコード（ルートレコードの `mapping_root`、`allocation_record_tree_root` およびツリーテーブル種別 3, 4 のルートを含む） | テーブル全体は比較しない：POSIX 内容が同一であり、かつプールレベル checker が R2 のイメージに対して緑と判定すること；`mapping_root`：R2 オールゼロ ⇔ R1 オールゼロ；`allocation_record_tree_root`：R1 がツリーを保持 ⇒ R2 オールゼロ、R1 のツリーテーブルが 0 件 ⇒（R2 非ゼロ ⇔ 新インスタンス番号 ≥ 2） |

システム構成の実行時変数（スロット世代番号、journal_tail、journal インスタンス番号）が v1 にフォールバックした場合、それら自身の行の規則に従って判定し、v1 と同一であることとしては判定しない。

**射程**：refinement の限界価値は、クラッシュポイント再生の列挙では網羅できない幾つかの次元 —— ペイロードバイト、レコード数 / セグメント数 / txg 数（リングラップアラウンドを含む）、直前のタイムラインの残留物、スロット数とデバイス数 —— にのみ存在する；クラッシュポイントの次元自体はクラッシュポイント再生によりすでにフル実行で列挙されている。「チェックサムを通過した」ことはモデル内ではブールフラグに過ぎず、**refinement が完全性を検証したと解釈してはならない** —— 確定項目 14 における D4 (チェックサムの位置) の「『Merkle を検証した』と解釈してはならない」と同形であり、D13 (検証ルート) 確定項目 4 の CRC32C 衝突確率に対する「認識した上での受容」と同一の判断である。D13 (検証ルート) 確定項目 4 は refinement の左側の `Crash`（モデル層の列挙ドメイン）であり、右側の `Spec` ではない；これを spec とみなすと refinement は自明に真となってしまう。I-6.5 (nonce 水位の単調性) の形式化は撤回された；D18 (ブロックに含める情報) 確定項目 14 が確定し、かつ nonce 割り当て規則（単調カウンタ + 前借り水位、水位の使用前永続化）が独立項目として確立された後にこのセルを再開する —— 再開時、その唯一の真の利点は nonce が POSIX から不可視である点であり、その spec は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の model と構造的に同源になり得ず、衝突 2 (D13 (検証ルート) 確定項目 13) の影響を受けない。

M1–M4 の前提とドメイン：セマンティクス層は「同一の (インスタンス番号, カウンタ) はすべてのタイムラインを通じて唯一の内容に対応する」ことのみに依存し、D23 (journal の役割とフォーマット) 確定項目 14「このバージョンの失敗処理」の同一再送によって担保される。ルートスロットの書き込み失敗により `checkpoint_txg` を進めて再送するパスが実装された際に再検証する；前提ドメインは E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリ（正当な書き込み側が書き出した履歴の任意のディスク永続化部分集合であり、新しいルートスロットは読み出せない場合がある）を取る。M2 の「その他のフィールドは選択されたルートと等しい」において、レコードから再構築されたルートのアンマウント記号は `NotWrittenByTheUnmountSequence`（D22 (ユニットの原子性をどう合成するか) 確定項目 7「他のルートは 0 を書き込む」）であり、選択されたルートの記号と異なる可能性があるため、「アンマウント記号を除く」と明記している。M3 の単調性の系は、直列コミットかつ故障を含まないクラッシュ状態においてのみ成立する。M4 はルート選択と連動して初めて「確認済みの」完全なパブリッシュがリカバリによって破棄されることを排除する。前提条件が揃っている必要がある：故障を含まないクラッシュ状態（確定項目 4 の列挙ドメイン）、D22 (ユニットの原子性をどう合成するか) 確定項目 7 のルート選択に従って選択されたルート、インスタンス境界を跨がないチェーン（D23 (journal の役割とフォーマット) 確定項目 14 注 1）、新インスタンスにおいて自インスタンスで書き込まれたルートが2枚のディスクをカバーするまでいかなる fsync も復帰させないこと（D16 (パブリッシュセマンティクス) 確定項目 8 のウォームアップ）；故障を含むドメインにおいては排除されない —— インフライト上限によりレコードが完全なパブリッシュが切り捨てられる場合があり、ディスク1台が丸ごと喪失した際に指名ユニットが1部欠損したバージョンは適用されない（C519 (ディスク丸ごと喪失時に直前に確認されたバージョンがリカバリで破棄される)）。

マウント透明性の射程：故障を含まず、書き込み可能マウントが成功したクラッシュ状態のみを管轄する。書き込み可能マウントが拒否された状態については透明性を判定せず、拒絶自体が正当であるか否かを判定する：拒絶が正当 ⇔ 拒絶理由が D23 (journal の役割とフォーマット) 確定項目 14 のリストに含まれ、かつその条項の判定基準に基づいてクラッシュイメージ上で自前で再計算して成立し、かつディスク上の生バイトが不変であり、かつ読み取り専用読み出しが許容集合内にあること；リストにない拒絶理由についてはクラッシュ注入第2区間の記述に従って赤を報告する。故障を含む状態は射程外である：番号取得の瞬間に1スロットが読み出せないことと N-構成が同一の拒絶理由として報告されるため、読み出し故障を含む階層に接続する前に分離する必要がある（C664 (マウント拒否の正当性再計算が2種類の拒絶理由を1つに統合して報告する)）。フィールド別表におけるツリー ID 水位、2つのツリーのルートポインタ、`checkpoint_txg`、スキャンレポートの4セルは推論されたものである（三者判定は弁護側の計算式を採用し、未測定）、F のセルの「押し上げた」半面は未測定である；ルートレコードの `mapping_root`、`allocation_record_tree_root`、墓碑、系譜、deleted_inodes の3つのルートポインタ、アルゴリズム、nonce、MAC、システム構成のスロット世代番号および journal_tail が各々どのカテゴリに帰属するかは、C655 (マウント透明性のフィールド別表に未分類フィールドが存在する)、C687 (mkfs 水位からツリー構築したバージョンの配置先で判定不能な数セル) の判定 Y1, Y2 に従ってフィールド別表内にパディングで補完して記述している。

**根拠**：
- E42 (1トランザクションあたりのレコード数)：「分断された ⇒ 不正」を恒等式とするためにペイロードを固定した結果、「分断されたが偶然に正当」であるクラスが構造的に不可視となった —— M2 がペイロードを記号化して補完したのはまさにこの点である。
- 三者レビュー：資料 `_d13-item1-background.md`、3つの経路の出力 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；3経路一致して旧候補を最下位とし、2つのクラウド経路はいずれも「プレフィックス判定とコミット原子性」の接合部に収束し、メイン agent は閉鎖性に基づいて最新選択とパブリッシュ原子性を同一ユニットに統合した（判定および旧候補撤回の3つの閾値の原文は変更履歴にある）。
- E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界)：有界内において本物の `replay_journal` に対し M2 のセマンティクス層、M4 および M3 の両解釈を判定し、ミューテーション殺傷限界を測定；ユーザーは 2026-10-01 にこれを軽量ツールと決定；C ドメインの正当なクラッシュ状態数は2種の分割法で記述（甲分割法は D16 (パブリッシュセマンティクス) 確定項目 7 のバリア位置）；敵対的ファミリにおける不動点の違反はすべて I-8.12 (同一インスタンスレコードの txg 非減少) が成立しないレコード表上にあり、述語が成立する側では違反 0（第3回再実行第1区間）；ルート選択結合判定ファミリが本物の `choose_root`、`scan_journal`、`replay_journal` を呼び出し、有界内・正当・故障なしの条件下で確認済みの完全なパブリッシュが失われない（第3回再実行第2区間）；ルート選択順序の4つの改悪変種のうち、正当な履歴 K, K9, KW ファミリの有界内において最初の有効なものを選択のみが赤となり、他の3種は敵対的ルートリング KA 上で赤となる（第3回再実行第3区間）。
- 三者判定（M1–M4 の文面と配置先）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第2節、`research/prompts/m3-formal-r3-main-verification.md` 第2, 3節；M3 単調性の系の4段階の導出は `m3-formal-r2-opus-output.md`「3.2 推論：直列コミット下で、j1 と j2 から到達バージョンが単調であることが導かれる」の節にある。
- 三者判定（M3 不動点の前提条件への I-8.12 (同一インスタンスレコードの txg 非減少) の追加と配置先の現状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第2節不動点前提条件セル（条項に命中、コードには命中せず）および第3節第5項；ハーネス側の前提が Verus の前提条件より広く、違反がすべて当該述語不成立側に位置することの照合は、スケジューリング記録 `records/2026-10-01-里程碑三形式化接入.md` 第10バッチ「10-5 返却」にある。
- ユーザー決定（2026-10-01 ポップアップ、すべて推奨通り）：アンマウント記号の除外、M4 射程文の前提条件の完全記述、単調性のバージョン順序に (`checkpoint_txg`, インスタンス番号) を採用、txg 上限到達時は「no_successor を報告し1件も適用されなかった場合は同等とみなす」を採用；書き戻しは K10 判定完了後にまとめて実施、原文は変更履歴にある。
- マウント透明性：実験なし：判定手法はクラッシュ注入内（`crates/singlefs-harness/src/mount_transparency.rs`、checker 層のクラッシュ注入がこれを呼び出す）にあり、実験ではない；3回の判定において攻撃側の数値はレプリカハーネス上で測定され、コードラウンド第2回で攻撃側がマージ済みコード上で測定した第1次クラッシュ 147 806 状態、二次クラッシュ 234 773 状態 0 赤について、メイン agent は再実行していない。
- 三者判定（マウント透明性）：`research/prompts/m3-formal-r4-main-verification.md` 第2節「マウント後に再リカバリ：何を法とするか」「比較対象と射程」の2行および第4節第2, 3項（独立した第4条、比較対象のバージョン、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第2節 U5, U6 の2行および第4節第5, 6項（フィールド別表、二次クラッシュの v1 比較、正当な拒絶の条件）；`research/prompts/m3-formal-r6-main-verification.md` 第2節 U5, U6 の2行（txg は下限のみ判定、拒絶の判定基準はクラッシュイメージ上で自前で再計算）および「フィールド別表で触れていないフィールド」の行；`research/prompts/m3-formal-code-r2-main-verification.md` 第2節 K2 の各行（実装は判定通り、成立；ウォームアップとマウントによる F 引き上げのセグメント分割）。
- ユーザー決定（マウント透明性に二次クラッシュを追加、2026-10-02 ポップアップ、原文は変更履歴にある）：`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定（2026-10-02）」二次クラッシュの行。
- ルート選択順序変種の一文：スケジューリング記録 `records/2026-10-01-里程碑三形式化接入.md` 第12バッチ E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) 第3回再実行第3区間判定 Q2 返却の項目（メイン agent がこの文を補完すると判定）。
- マウント継続実行をマウントの一部とみなす、F の境界はマウントが返却した F 引き上げの書き込み表末尾における位置とする：ユーザー決定 2026-10-03「継続実行はマウントの一部とみなす」（原文は変更履歴にある）；コードラウンド第3回判定 `research/prompts/m3-formal-code-r3-main-verification.md` 第3節第3項（境界は書き込みストリームの位置に基づき取得、見つからない場合も panic しない）；実装 `crates/singlefs-harness/src/mount_transparency.rs`。

**未済の検査**：C77 (再生起点の未定義)：最新選択とプレフィックス判定の接合部にある；C649 (Verus で判定できない2種のコールバック結果)：弁済済み（M2 セマンティクス層の逆方向チェーンと指名ユニットは依然として後置条件としてのみ記述でき、Verus 側では判定せず、2つのディスク読み出しコールバックは `crates/mutations.tsv` のミューテーション行でカバーする方針に変更、未済の検査弁済済み一覧を参照）；C650 (ルート選択と再生の結合判定がない)：M4 の射程文はルート選択と再生が結合して初めて成立する；C653 (リングスキャンのグルーコードのメモリがディスク数に応じて増加する)；C654 (必赤世界の期待行数がソルバ環境によって変動する)；C655 (マウント透明性のフィールド別表に未分類フィールドが存在する)：弁済済み（フィールド別表は判定 Y1, Y2 に従ってパディングで補完、未済の検査弁済済み一覧を参照）。

#### 確定項目 2：まず直列コミットとする

**確定事項**：トランザクション層はまず直列コミットを公約する —— `.claude/rules/fs-design.md` の「判定基準が書けない場合は、まず1つの経路のみを作成し、かつ**低速であっても正しい経路を先に作成する**」を具現化するものであり、直列化は本プロジェクトが明示する既定値であって、2つの選択肢から選んだ結果ではない。交差点を越えた後に並行を開くのであり、交差点の前に並行を公約するのではない。将来の並行コミットに対する事前の制約事項：レコードはトランザクション番号順に追加され、トランザクション番号 N の fsync は復帰前に < N のトランザクションがすべて追加済み ∨ 無効化済み ∨ インスタンス切り替え確定または読み取り専用遷移済みであることを待機しなければならない（無効化はコミットパイプラインのメモリ状態であり、並行する後続から可視）；直列コミット下ではこの規則は空転する。

**射程**：直列化の上限は1回の checkpoint あたり約 1.1 万エントリである（シングルスレッドのマージ所要時間が1回のデバイス同期に追いつく交差点、計算式は C58 (直列コミットの上限到達時の警報不在) にある）：交差点以下ではデバイス同期がボトルネックであり直列化に不利はないが、交差点を超えるとシングルスレッドのマージが上限となり安全マージンが消滅する。確定項目 2 はレコードヘッダのフィールド一覧を決定せず、新規プールでのファイル作成時に書き出されるバイト列を変更しない。性能上限と crash refinement の適用範囲を決定する。並行コミットは依然としてクラッシュセマンティクスに影響を与える：「プレフィックス」は全順序上の概念であり、並行下ではプレフィックスが完全なトランザクション B と不完全なトランザクション A を同時に含む可能性がある；レコードヘッダにはすでにトランザクション境界フィールドが含まれている（D23 (journal の役割とフォーマット) 確定項目 7：トランザクション番号 8 + コミットマーカー 1）。

**根拠**：
- E17 (write buffer マージ上限)：シングルスレッドの sort-merge の上限は交差点の半分である；リーフ分割によるロックフリー並行処理は交差点を越えた後の退避口であるが、それが十分であると既定で仮定することは許されない。
- E44 (シーケンス番号ビット幅の実測とコスト)：ローカル fsync レートは交差点のもう半分である。
- E42 (1トランザクションあたりのレコード数)：直列下であっても1つのトランザクションが複数レコードに跨がる場合は同様にプレフィックスによって分断され得る —— レコードヘッダのフィールド一覧を決定するのは「1つのトランザクションがちょうど1件の journal レコードを生成するか否か」であり、直列 / 並行ではない。
- ユーザー決定、変更履歴に記録；将来の並行コミットに関する事前制約は C113 (スキャン再構築時に複数バージョンユニットの現行バージョン判定に入力がない) 確定事項 P1 に由来する。三者検証は経ていない：これはすでに明記されている既定値を確認するものであり、真に三者検証を経るべきはいつ並行に変更するかという点であり、そのステップは実施されていない。

**未済の検査**：C58 (直列コミットの上限到達時の警報不在)。

#### 確定項目 3：RefFS を極限まで突き詰め、自前で永続化とクラッシュセマンティクスを処理する

**確定事項**：機能のみを扱うインメモリモデルにとどまらず、自前で永続化とクラッシュセマンティクスを処理する参照実装（RefFS）を作成し、RefFS を「予備候補」から**作成すべき対象**へと引き上げる。「極限まで突き詰める」はセマンティクスレベルを意味する：RefFS はメモリ内でのみ POSIX 操作に従って動作し、各 fsync 後に何が永続化されたとみなされるか、クラッシュ後にどのような POSIX 状態の観測が許容されるかを自前で定義し、ディスク上フォーマットは持たない；独自のディスク上フォーマットを持ち、自前でディスク書き込みとリカバリを行う層（工数はもう1つのファイルシステムを開発することに匹敵）は作成しない。RefFS と実装の間でクラッシュポイント列挙のコードを共用することを禁ずる：極限まで突き詰めた後、「3 oracle 結託テスト」（確定項目 6）が防ぐ共通の誤解の最も危険な落とし穴は、「RefFS と実装が同一のクラッシュセマンティクスに関する誤った理解を共有していないか」である。

RefFS の由来と境界：

| 項目 | 確定事項 |
|---|---|
| 由来 | kb の条文のみを参照して独自に作成：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のコードをコピーせず、記録ストリームを読み込まず、セグメント分割・状態計画・実装のクラッシュポイント列挙を呼び出さない |
| 配置先 | 独立したパッケージ `crates/singlefs-reference-filesystem`：依存関係クロージャ（dev-dependencies を含む）⊆ {それ自身, `singlefs-format`}、std のみで済む場合は std のみを使用；ソースコードは `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier` を参照しない（リネームされたエイリアスも同様）；`singlefs-core` と `singlefs-checker` の依存関係クロージャにこれは含まれない。3条はゲート checker-independence-and-sync の checker-implementation-disjoint セルが判定する |
| 入力ホワイトリスト | すべて呼び出し側から渡され、5点のみを受理：各操作の結末（成功、いかなる書き込みの前での拒絶、書き込み後のエラー、同一再送）；クラッシュが何番目の操作で発生したか（hi）およびその帰属ルール；ロールバック先 m；各リカバリで読み出された内容；明示的なパブリッシュポイント |
| 第1版の操作 | 最初のファイル、上書き、パブリッシュ呼び出し、fsync、ロールバック |
| 呼び出し箇所 | クラッシュ注入および故障注入の呼び出し箇所において、RefFS の判定と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官の全4条の判定基準の論理積を取り、いずれか1つでも赤であれば赤とする |

クラッシュセマンティクス（直列コミット、故障を含まないクラッシュ状態）：

1. **fsync**：fsync が成功復帰した後、今回の fsync が待機したパブリッシュが引き連れたすべての操作が存在しなければならない（直列下では呼び出し開始前に番号取得に成功した全操作であり、ユニット単位で計算）；状態インデックスとして読み替えると、許容集合の下限 lo がそのパブリッシュの末尾まで引き上げられる。fsync がエラーを返した場合は lo を引き上げず、エラーとなったパブリッシュが引き連れた操作は存在しても存在しなくてもよい；以降の操作は次回のリカバリで読み出された状態に接続して判定する。
2. **欠落方法：パブリッシュ境界によるプレフィックス**。クラッシュが第 hi 番目の操作内で発生した際、リカバリで読み出しが許容されるのは {S_j : j = lo、または j がある明示的パブリッシュの末尾；lo ≤ j ≤ hi} であり、S_j は前回のリカバリ以降の先頭 j 個の操作を適用して得られる状態、S_0 は前回のリカバリで読み出された内容である。任意の部分集合を欠落させるような欠落方法は直列コミット下では除外され（D16 (パブリッシュセマンティクス) 確定項目 12 と非互換）、並行コミットを導入する際に再審議する。
3. **lo の引き上げ**：成功復帰したパブリッシュ呼び出し、fsync、ロールバックはいずれも lo を引き上げる（D16 (パブリッシュセマンティクス) 確定項目 7：ロールバックと fsync は同一の復帰条件）；ロールバックは1回の確認済みパブリッシュとみなされ、そのインデックスはパブリッシュ末尾として計算される。
4. **ロールバック**：ロールバックは RefFS の1つの操作であり、ロールバック後の内容は履歴全体における第 m 回の確認済みパブリッシュの後の内容と等しい；m は呼び出し側が実装の報告したロールバックルートから逆算して渡し、RefFS は対象の選定が正しいか否かを判定しない。m の番号付けはルート書き込みのすべての入口（パブリッシュ呼び出し、ロールバック、行書き込みとウォームアップ、F 引き上げ、アンマウント）をカバーし、それを書き込んだ時点の現行バージョンにマッピングされ、現行バージョンには各リカバリで読み出され新たに開始されたバージョンを含む。
5. **クラッシュ時点の帰属**：クラッシュが何番目の操作で発生したかは、「現在のセグメントを区切ったバリアがどのステップから発行されたか」に基づいて帰属させ、最初に差し押さえられた書き込みには基づかない；このルールは RefFS のみに適用され、O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官は変更しない。
6. **リカバリ後**：各リカバリの後、RefFS はまず読み出された状態が許容集合内にあるかを判定し、その状態を起点として処理を続行する。

**射程**：どの程度まで突き詰めるかと RefFS の形態のみを決定し、スケジュールは決定しない；「実装の前にまず RefFS を作成する」と解釈してはならない —— RefFS は検証手段でありマイルストーンではない（`.claude/rules/format-evolution.md`「書き込みから開始し、読み出しから開始しない」）。明示的なパブリッシュポイントは履歴内のパブリッシュ呼び出しと fsync のみを取得する：自動トリガーによるバッチ処理が具現化するまで、RefFS は明示的パブリッシュポイントのみを受け付け、自動トリガーのパブリッシュは RefFS の射程外であり、確定項目 1 の M2 およびレコード照合器に帰属する；クラッシュリカバリで最新ルートを破棄した際、RefFS は読み出された内容から新たに開始し、そのステップ自体は判定しない。故障を含むクラッシュ状態（虚偽のデバイス、ディスク丸ごと喪失 C519 (ディスク丸ごと喪失時に直前に確認されたバージョンがリカバリで破棄される)）には適用されない。独立性：RefFS は記録ストリームを読み込まず、セグメント分割や状態計画を呼び出さないが、その (lo, hi) と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官のディスクレベル下限の入力パラメータは同源であり、C191 (同源性カウンタが1ペアのみ対象) の同源会計として記録される。D17 (実装階層とサードパーティパイプライン) 確定項目 4 はこれに基づき「入口」の実行可能定義を RefFS の操作面に委ね、fsync の永続性公約は RefFS のクラッシュセマンティクスによって定義される；O3 (独立規約実行器) は依然としてクラッシュポイント集合の独立実行器であり、責務は不変である。RefFS の第1版は `crates/singlefs-reference-filesystem` にあり、クラッシュ注入の2区間および故障注入再開の呼び出し箇所に接続済みである：harness 層は変換層 `crates/singlefs-harness/src/reference_filesystem_translation.rs` を介してこれに依存し、呼び出し箇所では RefFS と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官の2票を判定し、いずれかが赤であれば赤とする。

**根拠**：
- 実験なし：どの程度まで突き詰めるか、RefFS の由来と入力の選定は検証投資と境界のトレードオフである；3回の判定における攻撃側の数値はレプリカ装置上で測定されたものであり、マージ先コード上では未再実行であるため（`research/prompts/m3-formal-r5-main-verification.md` 第5節第3項）、ここでは引用しない。
- ユーザー決定、原文は変更履歴にある：「極限まで突き詰める」（当初確定）；セマンティクスレベルとする（2026-10-02、分岐表 `m3-formal-7-forks.md` 内の RefFS をどの階層まで作成するかの行、当初確定事項の解釈変更）；パブリッシュ境界によるプレフィックス、呼び出し箇所の論理積、新規作成、独立パッケージ（2026-10-02、`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定」）；論理積を全4条へ厳格化、ロールバック先を入力とする（2026-10-02、`research/prompts/m3-formal-r5-main-verification.md`「ユーザー決定」）；ロールバックによる lo 引き上げ、クラッシュ時点のバリア帰属、m が全ルート書き込み入口をカバー、ホワイトリストに同一再送を追加（2026-10-02、`research/prompts/m3-formal-r6-main-verification.md`「ユーザー決定」）。
- 三者判定：`research/prompts/m3-formal-r4-main-verification.md`（第1ラウンド：fsync、欠落方法、独立性の3セルと6つの分岐）、`research/prompts/m3-formal-r5-main-verification.md`（第2ラウンド：fsync 合成文、パブリッシュ境界による補足3文、論理積厳格化、新規作成と独立パッケージ、任意部分集合除外の直列限定）、`research/prompts/m3-formal-r6-main-verification.md`（第3ラウンド最終ラウンド：ロールバックと lo、射程2文、ホワイトリスト、状態インデックスによる fsync 読み替え）。
- D17 (実装階層とサードパーティパイプライン) 確定項目 4：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) は「クラッシュは関知しない」と明記しており、fsync の永続性公約を定義できないが、最初の実行可能目標はまさに「1つのトランザクションを正しくコミットすること」である —— 極限まで突き詰めた RefFS が補完するのはまさにこの半分である。

**未済の検査**：C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)；C191 (同源性カウンタが1ペアのみ対象)；C658 (クラッシュ注入第3段階が失われたステップの復活を見逃す)。

#### 確定項目 4：クラッシュポイント再生は完全書き込みの部分集合のみを列挙する

**確定事項**：クラッシュポイント再生が列挙すべきクラッシュ状態集合は以下のように定義される：記録された書き込みリクエストストリームを**デバイス単位で**セグメントに分割する —— ある書き込みは、同一ディスク上の後続のバリア（またはそのディスク上の FUA 書き込み）によってのみ後続の書き込みより前に順序付けられる；現在のセグメント内で書き込みのあった各ディスクが自らのバリアまたは FUA で解放されたときに初めてそのセグメントが閉じられ、1枚でも解放されていないディスクがある場合、そのバリア（または FUA）はセグメントを閉じず、前後の書き込みは同一セグメントとなる。1つのクラッシュ状態は「先行するいくつかのセグメントがすべて永続化され、現在のセグメント内の各書き込みが各自の状態を任意に組み合わせて取得し、後続の各セグメントはすべて未永続化である」状態である；列挙の単位は**1回の完全書き込み**である。FUA を伴う書き込みは自ディスクのみを解放する：それはそのディスク上で先行するバリアで隔てられていない書き込みと同一セグメントとなり、他ディスク上の書き込みがそれによって永続化されることはない。1回の書き込みは2つの状態（永続化 / 未永続化）を取る；**インプレース上書き**（ユニット書き込みではなく、1セクタより長く、対象範囲に元々データが存在していた書き込みであり、システム構成スロットの書き込みや旧レコードを上書きする journal 書き込みがこれに該当）は第3の状態「新旧いずれも読み出せない」を追加で取り、フル実行およびクイック層の双方が列挙する；第3の状態のイメージはその書き込みの全範囲を覆い、新旧で異なる区間の前半は新しいバイト、後半は古いバイトとする（セクタ単位ではなくバイト単位で断裂）；その他の書き込みの断裂状態は一律に「今回の書き込みが未永続化」に統合する。「未永続化」の位置には、イメージ内に**クラッシュ前のその位置の古いバイト**を配置し、ゼロ埋めは行わない。また、これらのイメージ（第3の状態を含む）は実際に生成して checker およびレコード照合器に渡す必要があり、単なるカウンタとして扱うことは許されない。イメージのミラー書き込み（D2 (RAID ストライピング戦略) 確定項目 6 の w ≥ 2、D22 (ユニットの原子性をどう合成するか) 確定項目 8 の journal ミラー）は**デバイス単位で**各1回の書き込みとして計算し、harness の状態数は自らの書き込みストリームに従って別途閉じた式を計算する。**同一ディスク上で連続するバリアは1つに統合する**：記録ストリーム内で、あるディスク上のバリアと同一ディスク上の次のバリアとの間に書き込みが一切存在しない場合（他ディスクのバリアは書き込みとみなさない）、このバリアは記録せず（`crates/singlefs-harness/src/lib.rs` の `push`）、独立モデルも同一規則に従って記録する；中間に書き込みがなければそのセグメントはクラッシュ状態を生成せず、統合の有無はクラッシュ状態集合を変更せず、操作数と種類文字列のみを変更する。

**射程**：モデル層の列挙ドメインを定義する。実デバイスの FLUSH / FUA が公約通りに効力を発揮するか否かは C6 (ブロック層セマンティクス仮定の記述誤り) に帰属し、QEMU 内で実デバイスを用いて検証する必要がある。デバイス単位のセグメント分割により、実際には到達し得ない状態が余分に生成される（1台のディスクが解放されていないとき、他ディスクの解放済み書き込みもセグメント内に残って任意の組み合わせを取る）が、到達可能な状態の取りこぼしはない。断裂は2種類に分かれる：インプレース上書き以外の書き込み（ユニット書き込み、1セクタ以下の書き込み、対象範囲に元々データが存在しなかった書き込み）の断裂状態と「未永続化」の差異は CRC32C の衝突確率（32 ビット、D19 (ブロックポインタの構造と幅の予算) 確定項目 2、D23 (journal の役割とフォーマット) 確定項目 11）のみであり、**認識した上で受容**し、断裂粒度はモデルに含めない。これは D20 (荷重面：ユニットの原子性と自己完結性) の「親ポインタを持つユニットはいかなる幅にも依存しない」と整合する；インプレース上書きが断裂した場合、その箇所の旧データも失われるが、「未永続化」では完全な旧データが保持されるため、両者は同一の状態ではなく、個別に列挙する。クラッシュポイント再生のセクタは 512 バイトとする；2本のクラッシュポイント再生ストリームにおいて第3の状態を取るのはシステム構成スロットの書き込みのみである。既知のエッジケース：第3の状態の旧バイトは「今回の書き込み以前の書き込みがすべて永続化した」バージョンを採用し、同一セグメント内のより早期の重複する書き込みが永続化しなかった組み合わせでは、古い半分にその書き込みのバイトが多く含まれる（2本のクラッシュポイント再生ストリーム上ではこのような組み合わせは一度も発生していない）；ルートスロットの書き込みが1セクタを超えかつ旧データを上書きする場合、第1版では第3の状態をサポートしない（harness アサーション；クラッシュポイント再生のプールはすべて 512 バイトの物理ブロックで構築され、到達しない）。デバイスが1回の書き込みをチェックサム衝突にちょうど重なる形状に断裂させるケースはモデル外である。D13 (検証ルート) 確定項目 5 はこれと直交する：一方は列挙ドメインを決定し、他方は crate の境界を決定する。

**根拠**：
- E77 (パブリッシュの永続化順序)：セグメントモデルの4種のバリア配置における状態数は、アームごとに閉じた式と厳密に等しい —— この列挙ドメインの数え上げは正しい。
- E142 (新規プールでのファイル作成のドライラン)：第17回実行第1区間の幾何学的感度サンプリングポイント G7（同一書き込みストリームに対する陽性対照、FUA のみを境界とする）の実測で 4097 状態、2046 違反 —— バリアが存在しない場合、ユニット、journal レコード、ルートスロットが1つのセグメント `[12]` に統合される；装置の分割法は `crates/singlefs-harness/src/segments.rs:91` の「FUA 書き込みは自身が属するセグメントを閉じる（FUA は前方の通常書き込みを永続化しないため、それらは同一セグメントとなり任意の部分集合を取る）」と一致し、「FUA が単独でセグメントを形成する」という解釈が予言した 2050 / 0 と一致しない —— FUA の一文の字面は実装の分割法に従って記述されている。
- 三者対抗および判定：[verification-build.md](../verification-build.md)「三者対抗（2026-09-03）」判定第3項 —— すべての種類のユニットがユニット全体の検証を持つ（親ポインタを持つものは親ポインタ内のチェックサムに依拠、D4 (チェックサムの位置) 確定項目 1、D16 (パブリッシュセマンティクス) 確定項目 7、D22 (ユニットの原子性をどう合成するか) 確定項目 19；自己検証ユニットはユニット全体のチェックサムと実際に検査されるコードに依拠、D22 (ユニットの原子性をどう合成するか) 確定項目 21；journal レコードはヘッダチェックサムとヘッダ内のペイロードチェックサムに依拠、D23 (journal の役割とフォーマット) 確定項目 13）、断裂状態と「未永続化」はチェックサムの観点からは同一の事象である。
- ユーザー決定（E77 (パブリッシュの永続化順序) および三者対抗に従って決定；FUA 書き込みをセグメント境界とみなす一文は C313 (FUA 書き込みをクラッシュセグメント境界とみなすか否か) 完了時のユーザー決定であり、2026-09-26 にユーザーが実装に合わせて「自身が属するセグメントを閉じ、前方のバリアで隔てられていない書き込みと同一セグメントとする」に変更）、変更履歴に記録。
- デバイス単位のセグメント分割（1台でも解放されていないディスクがあればセグメントを閉じない）、FUA は自ディスクのみを解放、インプレース上書きに「新旧いずれも読み出せない」を追加：ユーザー決定（コードレビュー第1, 3, 4項；2026-09-27 ポップアップで「A 統合、デバイス単位でバリアを記録」「同様に補完、フル実行も走らせる」「インプレース上書きに第3の状態を追加」を選択）、第3の状態をバイト単位で断裂させることはメイン agent が同日に決定し、ゼロラウンド攻撃された、変更履歴に記録；実装と判別力（ディスク1のバリア1つのみを吸収、セグメント末尾 `26,2,1,2` → `26,5`、クラッシュポイント再生小ストリーム内でレコード照合器が報告）は `m2-rev-b3a2-implementer-report.md` 第1, 3節にある。
- 同一ディスク上で連続するバリアを1つに統合：ユーザーが 2026-09-28 に「統合し、本日のレコーダーに追従する」と決定（E142 (新規プールでのファイル作成のドライラン) 第19回実行で登録された停止 S19-crates に遭遇：独立モデルは双方を記録し、レコーダーは2つ目を統合、ウォームアップと全ストリームの4セルで操作数と種類文字列が異なるがクラッシュ状態数は同一、報告 `e142-r19-runner3-report.md` 結論段、登録 `e142-r19-prereg.md` の R40）、原文は変更履歴にある。

**未済の検査**：C6 (ブロック層セマンティクス仮定の記述誤り)：仮想マシンのデバイス側ログを用いてクラッシュ状態を再構築し、クラッシュテストが必ず赤になる側の半分。

#### 確定項目 5：kb から生成された定数モジュール1つのみを共有する

**確定事項**：checker と実装の間で共有するものは唯一つ：kb のフィールド表から生成された定数モジュールであり、ジェネレータが kb から読み取り、双方が消費するのみで、何人も手作業で改変してはならない（`.claude/singlefs-ai-sop/rules/machine-first.md`：重複は生成すべきであり、手作業で書き写してはならない）。**それ以外は一律に共有しない**：アドレス空間の newtype は各自で宣言し、フォーマット解析は各々独自に実装し、チェックサムは各々独立した実装を用い、トラバーサルおよび会計コードの積集合は空とする（C12 (増分セマンティクスの共用) はゲート checker-independence-and-sync の checker-implementation-disjoint セルが定めた判定基準であり、形態は crate 粒度の依存関係クロージャの積集合でありシンボルレベルではない、「フォーマット解析と定数を除く」という例外規定はまさに本項目が利用している）。

- **判定幅を他所から取得することを禁ずる**：I-7.5 (ルートスロットの判定幅アライメント) および I-8.2 (レコードヘッダが原子ユニットを跨がない) の判定基準はマウント時に検出された `physical_block_size` であり、イメージ内には存在しない。checker は自ら1回検出し、システム構成に記録された mkfs 時の値と照合する（D22 (ユニットの原子性をどう合成するか) 確定項目 2「mkfs 時にプール内最大値に基づいて区画を割り当て、マウント時にデバイスごとに再計算して照合する」の checker 側の形態）。**検出不能時は「宣言値、未検出」と報告し、検出されたものとみなすことを禁ずる** —— ファイルをデバイスとするイメージがこれに該当し、クラッシュポイント再生の大多数のイメージはファイルであるため、この規定はエッジケースではない。異種混在プールにおいて判定幅は単一の数値ではなく集合であり（C17 (デバイスジオメトリをグローバルパラメータとみなす誤り)）、checker は mkfs 時の最大値取得の集約を再計算しなければならない。さもなければ正常な異種混在プール上で誤って赤を報告することになる。
- **ジェネレータは kb に配置先のない定数の出力を拒否する**、またスカラー値のみを出力する：仮に「フィールド表に基づいて計算されたオフセット関数」を出力し始めた場合、それは D13 (検証ルート) が明示的に共有を禁止したフォーマット解析となってしまい、もはや定数ではない；判定基準は出力物に分岐と演算が含まれているか否かである。
- **CRC32C を各自で実装する前提はパラメータの固定である**（多項式、初期値、入力/出力の反転、排他的論理和出力）：パラメータを固定しなければ、2つの独立した実装が健全なイメージ上で異なる結果をもたらし、それは独立性ではなく誤判定の赤となる。

**射程**：checker と実装の crate 境界を管轄する；kb 内でそもそも誤って記述された値は管轄できない —— 生成された定数を1つ共有することと双方で個別に書き写すことのいずれもそのクラスの誤りを捕捉できず、直交するゲートを必要とする（C94 (登録されたフォーマット定数と後続の確定事項の不一致)）。実装と確定事項の乖離：`crates/singlefs-format` は手書きであり生成されたものではない（値はゲート code-source-discipline の format-constants セルが kb 内の `format-const` マークに従って固定しており、マークされたもののみを固定）、ジェネレータは存在しない；`crates/singlefs-checker` は `singlefs-format` のみに依存し、CRC-32C はビット単位のものを別途実装している；checker はシステム構成が宣言した `physical_block_size` と `io_min`（`crates/singlefs-checker/src/image.rs` の `geometry_of`）を取得し、「検出 / 宣言」の2種類の出所の型は `crates/singlefs-checker/src/lib.rs` の `DecisionWidth` である：宣言値の形式（`DeclaredOnly`）はすでに I-7.5 (ルートスロットの判定幅アライメント) の判定に接続されており（実装済み、`decision_width_declared_by_the_chosen_system_configurations` は各ディスクで選択されたシステム構成が宣言する max(`physical_block_size`, `io_min`) の最大値を取る）、検出値の形式（`Probed`）は型定義のみが存在し（`crates/singlefs-checker/src/lib.rs`）、リポジトリ全体にそれを構築するコードはなくいかなる判定にも接続されていない；I-8.2 (レコードヘッダが原子ユニットを跨がない) は未実装である。システム構成における mkfs 時の `physical_block_size` フィールドは D22 (ユニットの原子性をどう合成するか) 確定項目 9 のジオメトリセグメント（4バイト）にある。

**根拠**：
- 実験なし：共有の境界は検証の独立性を担保する規律であり、測定可能な量はない；その判別力は双方が個別に実装した検査が1条ずつ赤を報告して示すことに依拠する。
- 三者対抗：資料 `_checker-sharing-vehicle-background.md`、各経路の出力 `research/prompts/checker-sharing-vehicle-*`、状況は [verification-build.md](../verification-build.md) にある —— 順方向経路は D13 (検証ルート)、C12 (増分セマンティクスの共用) から「定数は例外内、他は禁止」を導出；代替逆方向経路とローカル経路は各々同一の結論を構築：kb 内で誤って記述された値は共有と個別記述のいずれでも捕捉できない。
- ユーザー決定、変更履歴に記録。

**未済の検査**：C17 (デバイスジオメトリをグローバルパラメータとみなす誤り)；C387 (定数モジュールが手書きで kb から生成されていない)；C388 (checker の判定幅を自前で検出しない)。

#### 確定項目 6：互いに独立な3つの oracle

**確定事項**：3つの oracle が各々異なる対象を判定し、独立性の根拠は各々異なる：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判定対象 | 独立性の根拠 | 検証できないもの |
|---|---|---|---|---|
| O1 | インメモリモデル突き合わせ (HashMap 理想ファイルシステム) | POSIX 可視セマンティクス：内容、サイズ、ディレクトリ構造；クラッシュのない箇所での内容の期待値はこれに帰属し、クラッシュ後に許容される観測集合は RefFS（確定項目 3）に帰属する | ディスクに触れず、ディスクフォーマットを解析しない | ディスク構造が自己整合しているか；クラッシュセマンティクス；**同一の POSIX セマンティクスに対する2つの実装の共通の誤解** |
| O2 | 独立パーサ + checker | ディスクバイト列が I-1 (ブロックの自己記述) ～ I-6 (暗号化不変条件) を満たすか | **checker はテスト対象 crate のいかなるデシリアライズ / トラバーサルコードも呼び出してはならず**、解析処理を独自に再実装し、raw デバイスを直接読み出す必要がある | 構造の自己整合性のみを判定：あるブロックが別の**正当な**内容で上書きされた場合、Merkle チェーンは依然として自己整合し、会計も一致してしまう |
| O3 | 独立規約実行器 | クラッシュポイントで切断された後の各ディスク状態が規約の許容集合に含まれるか | **別の言語で実装**（Rust を使用しない）、decisions と invariants の記述のみから独自に再導出；第1版は Python で単独実装（E170 (O3仕様実行器第1版)） | 規約自体が誤って記述されている可能性 —— **陽性対照と組み合わせることで初めて意味を持つ**；射影される6種の要素の値および6種以外のルートレコードフィールド、実装自身のリカバリおよびリーダーのコード、システム構成スロットの証拠値（journal_tail）、1回のパブリッシュ内での各書き込みの前後関係 |

**独立性は検証されるべきであり、自己申告を禁ずる**：「3 oracle 結託テスト」を設計する —— 精密に構築された誤った実装（例えば nonce が常に 0）を用いて3つの関門を通し、3者すべてがパスさせた場合は共通の盲点が存在することを示しており、oracle の補完に立ち戻る。機能の正当性はこれら3つの oracle のみに依存するわけではない：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の行はすでにクラッシュ後に許容される観測集合を RefFS（確定項目 3）に委ねており、リカバリ関数の M2–M4 は Verus および E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界)（確定項目 1）に配置され、第2の入力を必要とする照合はレコード照合器（確定項目 7）に帰属する；3つの oracle 間の共通の盲点は「3 oracle 結託テスト」によって発見する。

**射程**：3つの oracle が各々何を検証できないかは表の最終列に記載されている；O2 (独立パーサ + checker) の定義域は確定項目 7 に帰属し、第2の入力を必要とする照合はレコード照合器に帰属する。**実装の現状**：O2 (独立パーサ + checker) は `crates/singlefs-checker`（フォーマット定数 crate にのみ依存）にある；O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の形態は `crates/singlefs-harness/src/model.rs` の理想モデル（メモリ上にのみ存在し、`singlefs_format` の定数のみを使用し、クラッシュは構築しない）であり、ゲート harness-model-differential-and-scenarios がランダム履歴を用いて段階的に突き合わせを行う；O3 (独立規約実行器) は kb の記述のみに基づいて書かれた Python 実行器であり、現行のものは E171 (3 oracle 結託テスト) 第1回再実行で使用された `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170 (O3仕様実行器第1版) 第2回再実行が第1回再実行で凍結された `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` から `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` をコピーし述語 W, T を追加、さらに 487 バイトの journal レコードヘッダに追従；第1回再実行で凍結された版と初回の版 `scripts/experiment-apparatus/e170_o3_spec_executor/` はアーカイブ済み）である：厳密な実行方法により判定可能であり（初回にエクスポートされたバッチ1、バッチ2において「規約の記述不足」に遭遇した状態は 0）、実装と状態ごとに一致し（両者とも完全にパス、これを正しく判定した証拠とはみなさない；「選択されたルートが自己検証済みルートの中で txg 最大のものではない」「③ 2スロット以上を受信」の2つのパスはバッチ内で 0 状態、C661 (規約実行器バッチ内で2つのパスに状態が存在しない)）、不正状態表を修正後の期待値に従って再判定すると判別力を有し（陽性対照およびブラインドテストの双方が正しく判定）、クラッシュポイント再生にはまだ接続されていない；あるクラッシュ状態を判定する手順：規約のリカバリ規則（システム構成選択、ルート選択、プレフィックスは D23 (journal の役割とフォーマット) 確定項目 14 の5条の計測条件に基づく）に従ってイメージから自前で1バージョンをリカバリし、リカバリ不能（プール全体の拒否または自己検証済みルート不在）の場合は集合外と判定；リカバリされたこのバージョンの射影（インスタンス番号、checkpoint_txg、ツリーテーブルユニットポインタ、中央マッピングツリールートポインタ、ツリー ID 水位、F）がバージョン表内のいずれかのバージョンの射影と等しく、かつそのバージョンのインデックスが [確認済み, 開始済み] の間にあること。バージョン表はベースイメージ上で自前でリカバリされた現行バージョンに加え、記録ストリーム内の各ルートスロット書き込みをストリーム順に各1バージョンとし、各バージョンの射影は実装がルートスロットに書き込んだバイト列を取得する；確認済みはクラッシュ状態で永続化した最新の書き込みより前に復帰済みの最新バージョンを取り（永続化した書き込みからのみ推論）、開始済みは永続化した書き込みが属する最新バージョンを取る。システム構成スロットはスロット選択、ルートリングおよび journal リングのジオメトリ取得、プールレベルフィールドの範囲外によるプール全体拒否の判定にのみ用いられる。したがってリカバリ規則の半分は kb の記述に基づいて別途実装されたものであり、許容集合の半分は実装自身が書き出したルートから取得される；RefFS（確定項目 3）は `crates/singlefs-reference-filesystem` にあり、クラッシュ注入および故障注入の呼び出し箇所に接続済みである；「3 oracle 結託テスト」は E171 (3 oracle 結託テスト) であり、測定値は凍結されたツリーから取得される（スナップショットダイジェストは実験ページ参照、現行の `crates/` とは同一ではない）：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の列はモデル突き合わせ + モデルのクラッシュ判定官であり、RefFS には接続されておらず、表の O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の行とは異なる（クラッシュ後の集合の半分において E171 (3 oracle 結託テスト) は RefFS を使用していない）；O3 (独立規約実行器) の列は E170 (O3仕様実行器第1版) 第1回再実行で凍結された版を使用；手動構築された不正イメージファミリは O2 (独立パーサ + checker)、O3 (独立規約実行器) のみに供給され、期待値は E170 (O3仕様実行器第1版) の設計者によるものであり、Q2 は登録字面通りに「存在する」（4枚とも P8）と判定し、登録 F3 に基づいて E170 (O3仕様実行器第1版) の期待値推論の誤りでありこれら4枚は不正イメージではないと判定され、真の不正イメージの中で O2 (独立パーサ + checker)、O3 (独立規約実行器) の両者が完全にパスしたものは 0 枚である；3者の出所が独立であるか（表第4列）について E171 (3 oracle 結託テスト) は Q2 において O2 (独立パーサ + checker)、O3 (独立規約実行器) のペアのみを測定しており、そのペアのパッチと期待値は E170 (O3仕様実行器第1版) の改修コードに由来するため、独立した測定値とはみなさず、O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) は Q2 に含まれていない；測定されていない項目を「互いに独立」が検証されたと記述してはならない。

**3つの oracle がいずれも判定しない2点**（第1点は現在クラッシュ注入側に判定器が存在する）：第1に、読み出し故障のないクラッシュ状態において、D23 (journal の役割とフォーマット) 確定項目 14 の書き込み可能マウントの証拠判定基準が真であること（システム構成スロット内の証拠値がディスク上から読み出せるレコードより先行している）：クラッシュ注入第2段階は拒絶が正当か否かを判定する際にクラッシュイメージ上でこの判定基準を再計算するのみであり、真であれば正当と判定する（マウント透明性の射程は故障を含まない状態のみを管轄する、確定項目 1）；「このクラッシュ状態には読み出し故障が注入されていないのに、判定基準が真である根拠は何か」を問う判定器が必要である：クラッシュ注入第2, 3段階は読み出し故障が注入されていないクラッシュ状態上でこの判定基準を再計算し、真であれば拒絶が不正であると判定する（`crates/singlefs-checker-tier/src/crash_injection.rs` の `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`、C670 (読み出し故障のないクラッシュ状態での書き込み可能マウント証拠判定が真であることの判定器不在) は弁済済み、未済の検査弁済済み一覧を参照）。「読み出し故障のないクラッシュ状態において各ディスクに少なくとも1部の自己検証済みシステム構成が存在する」という性質は G4 が判定する（クラッシュ注入第2, 3段階、C670 (読み出し故障のないクラッシュ状態での書き込み可能マウント証拠判定が真であることの判定器不在) と同一の接続方法）：mkfs 復帰前のクラッシュ状態は G4 の射程外であり、mkfs はディスクごとに2スロットを同一ループ内で書き込みその後に初めてディスクごとにバリアを発行するため、クラッシュ注入の候補セグメントと E171 (3 oracle 結託テスト) のベースイメージはいずれも mkfs の後に取得される；G4 とその条項にはもう1点形状の相違がある：システム構成を選択できない場合、その条項の再計算は `None` を返すが G4 は真とする。この時読み出しフェーズ自体がすでに拒絶しており、このセルには到達しない。第2に、1回のパブリッシュ内での各書き込みの前後関係：O2 (独立パーサ + checker) の I-7.12 (システム構成の F が同一ディスクルート上の F 以上) は、F 引き上げシーケンスにおいてまずシステム構成を書き込むステップを判定する；ミューテーションで指名されたテストケースは E171 (3 oracle 結託テスト) 270行目の種類（パブリッシュ復帰後の各クラッシュポイントにおける証拠値）のみを判定し、242行目で指名されたテストケースは失敗パスの凍結を判定する；`layout/01-first-txn.md` 第8節はルートスロットの FUA とシステム構成スロットのローテーションを2セグメントとして登録しているが、登録簿上に存在するに過ぎない；その他の書き込み順序は3者には含まれない。

**根拠**：
- E170 (O3仕様実行器第1版)：O3 (独立規約実行器) は「別の言語、kb の記述のみから再導出、陽性対照を配備」に従って第1版を記述可能であり、単一のクラッシュ状態を判定でき、1状態あたりの所要時間はクイック層に組み込み可能である；第1回再実行（2026-10-03 の kb に従って1回改修された実行器）は厳密な実行方法で判定可能であり、サンプリングされたバッチ状態上で実装と一致し、陽性対照は判別力を有する（初回実行で判定不能であった点および判別力不足であった2点は実験ページに記録）。
- G4（読み出し故障のないクラッシュ状態において各ディスクに少なくとも1部の自己検証済みシステム構成が存在する）：コードラウンド判定 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第2節 j2-b 成立、第29バッチで実装、射程と mkfs に関する部分は `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第2節 L3 および第3節第3項を参照。
- 3つの oracle の分担とそれぞれの独立性の由来は検証体系の構造設計である；独立性は「3 oracle 結託テスト」によって観測される（E171 (3 oracle 結託テスト)）。
- E171 (3 oracle 結託テスト)：凍結されたツリー上において、不正実装表の中に3つの oracle すべてがパスさせた行があり、1つのみによって捉えられた行もあるが、O3 (独立規約実行器) は不正実装表上で1行も赤を出さなかった；手動作成の偽造ルート、偽造継続イメージの中に O2 (独立パーサ + checker) がパスさせ O3 (独立規約実行器) が集合外と判定したものがある（測定値と限定条件は実験ページ Q1, Q2, Q3 の3行にある）。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) と RefFS の分担文：三者判定 `research/prompts/m3-formal-r5-main-verification.md` 第2節の新規作成および独立パッケージのセル（両経路が同一方向）。

**未済の検査**：C445 (モデル突き合わせのサンプリング点が書き込みパスに応じて増加しない)；C446 (未実装の不変条件に事前登録がなく期日もない)；C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)；C407 (3 oracle 結託テストの判定器がゲート未導入)：E171 (3 oracle 結託テスト) は1回測定されたが、その成果物を読み込むゲートセルは存在しない；C672 (戻り値やメモリ状態が誤っているが書き込みストリームが不変であるクラス)；C679 (アンマウント記号がモデル突き合わせに含まれていない)。

#### 確定項目 7：O2 は何を判定するか：単一イメージとレコード照合器

**確定事項**：

> **確定事項：O2 (独立パーサ + checker) の定義域は単一イメージである** —— 単項述語 `P(イメージ) → 成立/不成立` であり、
> 集合の要素性のみを判定し、決して identity を判定しない。**第2の入力（別のイメージ、または一連のレコードストリーム）を必要とする検査はいずれもこれに属さない。**

**第2の入力を必要とする照合はレコード照合器に帰属する**。入力パラメータは `(クラッシュ前イメージ, レコードストリーム, クラッシュ後イメージ, 永続化集合)` である —— 永続化集合は列挙器が提供する、このクラッシュ状態においてどの書き込みがディスクに永続化したかを示す情報である（クラッシュポイント再生のセグメント内完全書き込み部分集合）；ディスク上の生バイトを参照するだけでは「書き込みが未永続化であり、元の位置の内容が偶然同一であった」ことと「書き込みが永続化した」ことを区別できないため、再利用と欠損の判定には永続化集合を用いて判定しなければならない；これは独自開発第1項の「Merkle ルートチェーン差分」（確定項目 8）と同様に二項検査に属する。⚠️ **意図的にこれに番号を付与しない** —— O2 (独立パーサ + checker) をめぐる混乱は番号を呼称として用いたことに起因しており、さらに O4 を追加することは次なるドリフトの温床を提供するに過ぎない。⚠️ **「クラッシュ後イメージ」は1つではなく2つである**：harness の実行順序はまず実装自身のリカバリを実行し、その後にレコード照合器を実行するが、リカバリ自体がディスクを変更する（インスタンス切り替えによる行書き込み、インフライトな checkpoint の再送、D23 (journal の役割とフォーマット) 確定項目 14）⇒ ルート選択とプレフィックス判定は**クラッシュ状態イメージ**を参照し（「レコードストリーム」はそのリング内の journal レコードである）、比較対象は**実装がリカバリした後のイメージ**となる。

**射程**：ポインタ層の目標状態下において、1件のレコードを照合するにはフォーマット解析 + チェックサム + ルートリングの最新選択のみで足りる；論理意図レコードはいかなるディスク上オブジェクトも指名しないため、照合側はツリーセマンティクス + 会計 + 割り当て正当性を自前で実装しなければならない —— D23 (journal の役割とフォーマット) が論理意図ログを否決した第2の理由はこれに基づき第1の理由の1つの封じ込めとして書き直され、D23 (journal の役割とフォーマット) 確定項目 5 に記述されている。レコード照合器のコードは `crates/singlefs-checker-tier/src/crash.rs` にあり、クラッシュポイント再生の各クラッシュ状態に接続されている（ゲート checker-tier-crash-point-replay）。したがってレコード照合器の判定は単なるイメージの関数ではない：クラッシュ後イメージがバイト単位で同一であっても永続化集合が異なる2つの状態に対して、異なる判定を下すことが可能であり、イメージに基づく重複排除による高速化はこれには適用されない（O2 (独立パーサ + checker) および oracle は従来通りイメージのみを参照する）。クラッシュポイント再生は書き込み可能マウントをカバーしない：クラッシュポイント再生は各クラッシュ状態において読み取り専用リカバリのみを実行し（journal を参照する場合と参照しない場合を各1回）、番号取得、行書き込み、ウォームアップはクラッシュポイント再生のいかなるクラッシュ状態上でも実行しない。書き込み可能マウントはクラッシュ注入によってカバーされる（`crates/singlefs-checker-tier/src/crash_injection.rs`）：抽出された各クラッシュ状態において、読み取り専用リカバリと判定の後に同一のクラッシュ後イメージ上で書き込み可能マウントを1回起動し、再度パブリッシュを行い、プールレベル checker を実行する；マウント途中で再度クラッシュさせ、番号取得、行書き込み、ウォームアップの3区間に各1つの二次クラッシュ状態を配置し、各々において読み取り専用リカバリ、モデルへの問い合わせ、プールレベル checker、レコード照合器を実行する：第3段階においてレコード照合器に渡される書き込み表は「マウント区間から現在のセグメントまでのプレフィックス」から履歴全記録ストリーム + 全マウントストリーム + マウント後のパブリッシュへと置き換えられ、永続化集合はセグメントごとに対応する（履歴区間は第1次クラッシュの永続化集合を取り、マウント区間は二次クラッシュに従って部分集合を取り、その後のパブリッシュはすべて未永続化とする）。「比較対象は実装がリカバリした後のイメージである」の半分：レコード照合器は2つのイメージを受理するように改修され、クラッシュ状態イメージはルートが存在しレコードが存在するか否かを判定し（ルート選択とプレフィックス）、リカバリ後イメージはユニットが存在するかを判定し到達バージョンのインスタンス表を読み出す；第2段階で接続されるリカバリ後イメージはマウントとそのパブリッシュ後のプールであり、リカバリが自称するバージョンはマウントとそのパブリッシュが書き出した最新のルートを取得し、マウントが1つのルートも書き出さなかった場合は第1次クラッシュが到達したバージョンにフォールバックする。

**根拠**：
- 実験なし：**根拠は型であり、コストではない**：「バイト単位の比較」は二項関係 `R(イメージ₁, イメージ₂)` であり、**いかなる投資を行っても単項述語を二項関係に変換することはできない**；I-4.5 (廃止済み・2イメージ必要) は同一の基準によって廃止された。
- 「クラッシュ後イメージ」は2つである：D16 (パブリッシュセマンティクス) 確定項目 4 に伴い質問の焦点を絞り込んだラウンドの三者論証（資料 `_d16-items134-background.md`）においてユーザーが決定、変更履歴に記録。
- 永続化集合という入力パラメータ：ユーザー決定、原文は変更履歴にある；判定子が観測できない実証根拠は `research/prompts/m2-layer0-scale-r3-main-verification.md` 第2節のセクタ単位免除の行にある。
- クラッシュポイント再生は書き込み可能マウントをカバーせず、書き込み可能マウントと二次クラッシュはクラッシュ注入がカバーする：ユーザー決定（コードレビュー第2項、2026-09-27 ポップアップで「クラッシュ注入層で書き込み可能マウントを補完」を選択）、変更履歴に記録；クラッシュ注入側の実装は `m2-rev-b3b-implementer-report.md` にある。

**未済の検査**：なし。

#### 確定項目 8：独自開発の2点：Merkle ルートチェーン差分と世代間増分 scrub

**確定事項**：既存のツールには存在せず、本プロジェクトが独自に開発する2点：

**1. Merkle ルートチェーン差分 —— COW + トランザクションごとの新ルート発行に特有のてこ。**
履歴ルートは回収されるまで物理的に存在し、再生可能なチェーンを構成する。隣接する2世代のルートに対して再帰的に「部分ツリーポインタ + チェックサム / MAC」を比較し、
**ポインタが不変の箇所は直接スキップして掘り下げない**。計算量は2世代間で実際に変更された部分にのみ比例する。
これは従来の走査型 checker では検知できない種類の誤りを検出できる：**同期改ざん後に自己整合したインプレース書き込み** ——
ある書き込みが COW を迂回してインプレースでブロック内容を書き換え、同時にチェックサムを更新した場合、単一点で見ればそれは自己整合している（I-2.1 (チェックサムと内容の一致) は全緑）が、
ルートチェーン差分は矛盾を発見する：**COW を経ていない以上は新たな割り当てが存在するはずがなく、親ポインタは不変であるべきなのに、チェックサムが変化している**。
⚠️ **必ず陰性対照を設けること**：通常の COW コミット後に同一の検査を実行した際に完全に緑でなければならず、さもなければ通常更新のすべてを改ざんとみなしてしまう。

**2. 世代間増分 scrub。** I-1.2 (ブロックヘッダ書き込み順序パブリッシュ済み) および D5 (スナップショット / 空間会計メカニズム) の birth txg に依拠し、
「前回の scrub が走査した generation 水位」を維持し、`birth generation > 水位` のブロックのみを走査する ——
古い部分ツリーのチェックサムチェーンが変化していなければ、必然的に依然として自己整合している。**これにより、「毎コミット時に今回の変更をフル実行で検証する」ことが実時間上で実現可能となり、サンプリングが不要となる。**

**射程**：移動されたレプリカは `birth` が変化せず、永遠に水位の下に位置するため、世代間増分 scrub はそれらを検知できない。`crates/` 内にルートチェーン差分は存在せず（`grep -rn 根链 crates` でヒット 0）、世代間増分 scrub も存在しない（`grep -rn scrub crates` でヒット 0）。

**根拠**：
- 実験なし：2点とも COW のトランザクションごとの新ルート発行と birth txg の構造的性質に基づいて成立しており、いずれもまだ実装されておらず、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C389 (Merkle ルートチェーン差分のコード不在)；C194 (世代間増分 scrub が再配置されたレプリカを検知できない)；C408 (世代間増分 scrub のコード不在)。

#### 確定項目 9：クラッシュポイント再生は3層に分割する。さもなければ実時間が爆発する

**確定事項**：

| 層 | 範囲 | トリガー |
|---|---|---|
| スモーク層 | 固定シード、固定スクリプトの負荷と登録されたクラッシュ列挙ケース、書き込みリクエスト数で規模を制限しない（規模は射程参照）、正確なフル実行 | checker 層のフル実行（確定項目 15）：ユーザー要求時に実行；コミット時は既定でクイック層のみ実行 |
| 通常層 | 中規模負荷（書き込みリクエスト数千件以上）、正確なフル実行、列挙しきれない場合は正確な枝刈りに依拠し、直ちに実行できないノードには ignore を付与、カバレッジレポートおよびゲートで個別に「今回未実行」と列挙 | 毎コミット |
| 大負荷層 | 通常層の負荷 + 大規模負荷 + 複数デバイス階層の交差、N≥5 ラウンド、正確なフル実行 | ユーザー要求時 / リリース前 |

**枝刈りと ignore を導入した後に断言できること**：有効になっているノードがすべてのクラッシュ状態において通過したと断言できる；
**断言できないこと**：ignore されたノードが通過したとは断言できない。

**フル実行層はコミット時には既定で実行せず、時刻を制限せず、ユーザーが要求した際に実行する；コミット前 = クイック層が全緑（必要条件）。**
**マイルストーン完了時、クラッシュポイント再生または判定器を変更した一連のコミットの前には、メイン agent がポップアップでユーザーにフル実行層の実行を促す；失敗した場合やユーザーが実行しない場合でもコミット自体は受け入れ可能とし、修正または追試が完了するまで未結の赤または「この段階完了時にフル実行未実行」として記録する。**
「フル実行」は2点を指す：checker-tier-crash-point-replay `--full`（ゲート checker-tier-crash-point-replay、確定項目 15 のフル実行）およびクラッシュ増幅のフル実行（`scripts/crash/crash-amplification.sh` 引数なし）。
「クラッシュポイント再生または判定器を変更した」の判定はクラッシュ判定ライブラリに基づいて動的に計算する：今回の変更によりライブラリ内に判定ブロックを持つユニットキーが一致しなくなり、一致しないキーの中に判定器ユニット（判定器エントリ fn のユニットクロージャ内の関数ユニット）の変更が含まれている場合に該当とみなす；ステージは無効化されたブロック数と変更された判定器ユニット一覧を出力し、メイン agent はそれに基づいてポップアップを表示し、パス一覧に基づく判定は行わない。
「前回のフル実行」はメイン agent が現在のマイルストーン記録に1行で記述する：日付、2点それぞれの結論。
**フル実行層は「カバレッジの最大化」ではなく「失敗の再現性」を最優先で保証しなければならない** —— フル実行層のバグを1つ修正するための実時間 ≈ `log2(N) × フル実行1回の実時間` であり、再現不可能なフル実行層の失敗の価値はほぼゼロ、
あるいはマイナスであり、赤信号を無視する習慣を助長してしまう。

**射程**：スモーク層（ゲート checker-tier-crash-point-replay）のみが存在する；スモーク層は2層に分かれる：通常のクイック層ではセグメントごとのインプレース書き込みは任意の部分集合を取り、COW 書き込みは完全永続化または完全未永続化のみを取る（ユーザーが 2026-09-26 に決定、「甲二」と呼称、C561 (レコード照合器の再利用免除が再利用の半分のみ永続化された際に誤判定で赤となる) の修正後に使用）、これは COW 部分のみがディスクに永続化した際に初めて現れるクラスの誤りを検知できず、フル実行の代替にはならない；フル実行は中断からの再開をサポートし（ゲート checker-tier-crash-point-replay `--full`）、`.claude/gate.d/stage-inputs.tsv` に登録されたクラッシュ列挙ケースは並行線1のストリームで 12230590578 状態に達しており、もはや「書き込みリクエスト数2桁」のスモーク規模ではないため、コミット時には既定でクイック層のみを実行し、フル実行は確定項目 15 に従いユーザー要求時に実行する；通常層、大負荷層はゲート内にステージを持たない。各クラッシュ状態の定義方法は確定項目 4 に帰属する；ノードの正確な枝刈り方法、ignore ノードをいつ開放するかは確定項目 10、11 に帰属する。

**根拠**：
- 実験なし：階層化はゲートの実時間に関する設計制約であり、通常層、大負荷層の負荷と実時間はまだ存在せず、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- 通常のクイック層に甲二を使用：ユーザー決定、原文は変更履歴にある；クラッシュポイント再生の規模に関する3ラウンドの判定 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- ユーザー決定：「我々はサンプリングではなく、枝刈り + 初期 ignore である」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：サンプリングせず、枝刈り＋初期 ignore」の行）、「列挙には ignore マークを設定し、徐々に開放できる」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：ignore マークを徐々に開放」の行）、「1 列挙すべきものは設計し尽くす、全網羅で設計する」「2 列挙しきれないことこそが枝刈りの出番である」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節で資料担当がゲート doc-process-records の implementation-premise セルに阻止され、ユーザーが『全網羅で設計、列挙しきれない分は枝刈り、適切な粒度』と定めた行）；第1ラウンド判定 `research/prompts/m3-prune-gpu-r1-main-verification.md`「決定の振り返り」の節：確定項目 9 を破棄。
- ユーザー決定（2026-10-01）：フル実行層は時刻を制限せず、ユーザー要求時に随時実行、「夜間」を削除、原文は変更履歴にある。
- ユーザー決定（2026-10-01）：フル実行層が全緑であるべきタイミングは、マイルストーン完了時およびクラッシュポイント再生や判定器を変更した一連のコミットの前であり、メイン agent がポップアップでユーザーに実行を促す、原文は変更履歴にある；ユーザーに委ねるセルは三者判定 `research/prompts/test-script-align-r1-main-verification.md` 第2節 t1-3。
- 三者判定 `research/prompts/test-script-align-r2-main-verification.md` 第2節 U3（「U3 t1-3 の書き戻しは字面通りには実現不能」の行）：「フル実行」が指す2つの対象、「クラッシュポイント再生または判定器を変更した」のパス判定方法、「前回のフル実行」の記録方法、「受入条件」と「非同期」を1文に統合。

**未済の検査**：C409 (通常層・大負荷層がゲート内にステージを持たない)；C635 (前回のフル実行に機械可読なマーカーがない)。

#### 確定項目 10：真に正確に網羅すべき N は「レイアウトのセット数」ではなく「コミットプロトコル構造の同値類数」である

**確定事項**：通常層の正確な網羅は、まず問題を縮小してから枝刈りを議論する。クラッシュポイントの定義域は**トランザクション層ステートマシン**であり、`.claude/rules/fs-design.md` には2つのトップレベルルールがある：
「1つのトランザクション層をすべての構造が共用する」および「唯一多様化を許さないのがトランザクション層である」。したがってレイアウトが変更し得るのは2種類のみである：

| 差異 | 変更されるもの | 判定 |
|---|---|---|
| ノードサイズ、ストライプ幅 | 1ステップで書き込むバイト数、特定ステップの繰り返し回数 | **パラメータ**、ステップ種類の新規追加なし |
| 回転 vs SSD | seek コスト、IO スケジューリング | デバイス層に位置し、**トランザクション層プロトコルには位置しない** |
| **zoned（ZNS）** | **ルートポインタをインプレース上書きできるか否か** | **構造的**：追記のみ、ルート位置のドリフト、zone finish/reset の追加 |

**具現化手段は閉じた列挙 + 網羅的な match である**：`crates/singlefs-core/src/transaction.rs` の `CommitStep`、ステップ種類の登録場所は [layout/01-first-txn.md](../layout/01-first-txn.md) 8のセグメントシーケンス登録表である。
いかなるレイアウトであっても新たな構造的ステップを導入する場合、**この共有定義を変更しなければならない ⇒ コンパイラがすべての参照箇所で分岐の補完を強制する ⇒
この diff はゲートで grep 可能である**。これにより「今回新たな構造同値類が導入されたため、これに対して独立した大負荷層の再生を新たに開始する必要がある」と自動的にマークでき、
**人間が申告を覚えていることに依存しない**（diff によるゲート範囲の計算は C8 (ゲート適用範囲の判定不能) で未済）。さらに newtype による隔離（`RotationalPtr` / `ZonedPtr`）を追加し、
D12 (対象メディア) 確定項目 2 における「共用を禁ずる」バケット（会計事実の判定パス）をコンパイル時の制約とする。
⚠️ **これには第2の責務がある（D17 (実装階層とサードパーティパイプライン) で追加）：サードパーティパイプラインの接続ポイントである。**
プロトコルは本プロジェクトに帰属し、あるパイプラインが新たな構造的コミットステップを導入する場合、本リポジトリにこの diff を送信しなければならない。

**射程**：回転ディスクと SSD はコミットプロトコルのレベルで同型であり、D12 (対象メディア) の三者論証ラウンドがこれを担保する；zoned の行はそれが少なくとも1つの同値類を追加することを示すのみである。同値関係を記録ストリームのセグメントシーケンスの同型性に従って判定すること、および同値類が幾つ存在するかは D17 (実装階層とサードパーティパイプライン) 確定項目 2 に帰属する。種類文字列によってノードを識別することで十分か否かは、第1ラウンド判定で指摘された（保留、第2, 3ラウンドの攻撃待ち）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「セルごとの判定」の節。
⚠️ **これが固定するのは「どのようなステップが存在するか」のみであり、「各ステップの失敗原子性の幅」を固定することはできない** ——
永続性メモリやオブジェクトストレージは各ステップの失敗原子性の幅を変更するが、列挙では対応できない（ギャップと補完策は D17 (実装階層とサードパーティパイプライン) 参照）。
`RotationalPtr` / `ZonedPtr` という newtype による隔離は `crates/` 内に存在しない（`grep -rn "RotationalPtr\|ZonedPtr" crates` でヒット 0）。

**根拠**：
- 実験なし：N が何であるかは `.claude/rules/fs-design.md` の2つのトップレベルルールから導出され、測定可能な量はない；同値関係をどう判定するか、幾つ存在するかに関する根拠は D17 (実装階層とサードパーティパイプライン) 確定項目 2 にある。
- 三者論証：通常層サンプリングのラウンド（「三者論証 + 実測検証」）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C8 (ゲート適用範囲の判定不能)。

#### 確定項目 11：ignore ノードをいつ開放するか：未済の検査によるローテーション台帳を否決し、3条を残す

**確定事項**：

**「未済の検査によるローテーション台帳」スケジューリング：否決。** スロット = `(レイアウト, サブフェーズバケット)` とし、未済の検査の降順でキューの先頭を取得し、任意のスロットが高々 `K_max = ⌈N×M/C⌉` 回のコミットで必ずカバーされると主張したスケジューリングは否決された：それが保証できないものこそがまさに防ぐべき対象 —— 低速で稀な分岐 —— であったためである（実測検証および他の4つの否決理由は変更履歴にある）。

**残された3条**：

1. **方向性の正しさ**：確率的な論証ではなく、**決定論的であり、ゲートで赤と判定可能なカバレッジ記録**を求める ——
   長期的な保守において、確率的な保証は保証が存在しないことに等しい。

<!-- doc-lint:registry name-col=2 -->

| 記号 | 略称 | 意味 |
|---|---|---|
| T1 | 通常層 実時間予算 | 通常層のローカル実時間予算、出力ではなく入力 |

2. **`T1` (通常層 実時間予算) は入力であり、出力ではない**（`.claude/rules/fs-design.md`：ゲートの実行時間自体が設計制約である）。
   正しい順序は「まず予算を宣言する → それを用いて1回のコミットで開放可能な ignore ノード数を制約する」であり、「所要時間を測定する → 開放可能な数を導出する」ではない。
   **この条項はいかなる測定も必要とせず、今すぐ確定すべきである。**
3. **判別力はカバレッジと分離して検査しなければならない**：各 ignore ノードが開放されたコミットにおいて、「既知の壊れたイメージ」に対してそのノード自身の checker を追加で実行し、
   エラーを報告しなければ赤と判定する —— これは D13 (検証ルート) に既存の陽性対照規律の直接の再利用である。C13 (checker 判定の無効化) 参照。

ゲート内には「通常層 クラッシュポイント再生」というステージが存在せず、`T1` (通常層 実時間予算) / `C` / スロットごとの所要時間は測定できない。
⚠️ **したがって `C`、`K_max`、`T1` (通常層 実時間予算) の3つの数値はいずれも現時点で kb に書き込むことを禁ずる。**

**射程**：ignore ノードをいつ開放するかのみを管轄する；スモーク層の全数列挙には `T1` (通常層 実時間予算) は不要である。第2条の「今すぐ確定すべきである」と末尾の「現時点で kb に書き込むことを禁ずる」は矛盾しており、[verification-build.md](../verification-build.md)「未決確定事項」第6問に記録されているが未判定である。ノード登録表には状態フィールド（開放 / ignore / 実装待ち）を持たせ、さらに3条を追加する —— 各 ignore 行は変更対象となるオブジェクトを1つ指名し、そのオブジェクトが緑になったにもかかわらずノードが依然として ignore である場合に赤と判定、カバレッジレポートに ignore の状態数と割合を列挙：これら3条はゼロラウンド攻撃されており（`research/prompts/m3-prune-gpu-r1-main-verification.md`「セルごとの判定」の節）、確定事項とはせず第2, 3ラウンドの攻撃を待つ。

**根拠**：
- 実験なし：台帳の否決はスケジューリング規則のシミュレーション検証（200ラウンド、後半の定常状態最大値を取得、厳密なプレフィックス貪欲法、3行の数値は変更履歴にある）に基づき、実験としては登録されていない；残された3条はゲートの設計原則であり、測定可能な量はない。
- 三者論証：通常層サンプリングのラウンド（「三者論証 + 実測検証」）、資料は保存されていない、経過は変更履歴にある。
- ユーザー決定：「列挙には ignore マークを設定し、徐々に開放できる」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：ignore マークを徐々に開放」の行）；第1ラウンド判定 `research/prompts/m3-prune-gpu-r1-main-verification.md`「決定の振り返り」の節：確定項目 11 を破棄、「セルごとの判定」の節でノード登録表に状態フィールドを持たせる形態と3条の攻撃待ち補足条件を提示。

**未済の検査**：C8 (ゲート適用範囲の判定不能)；C13 (checker 判定の無効化)。

#### 確定項目 12：衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する の衝突

**確定事項**：**衝突 1：`checker 即仕様` と `検証の2つの経路間でツール共有を禁止する` が直接衝突する。**
使用中の不変条件（件数の公式な登録場所は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) のファイル先頭にある）が checker のソースコードであり形式検証の spec でもある場合、単一障害点となってしまう。
**実行可能な妥協点であり、本プロジェクトがすでに投資済みの解決策**：checker は2つの方向を実装する —— **トラバーサル型**（システム構成から順に辿る）
と**スキャン型**（ディスク全体を走査し、「メタデータブロックは自己記述的でなければならない」というブロックヘッダに基づいて再構築する）。双方は同一の不変条件に対して独立に判定を行い、
共有するのはフォーマット解析のみとする。

**射程**：checker 内部の2つの方向の間で何を共有するかを管轄する；checker と実装の間で何を共有するかは確定項目 5 に帰属する。プールレベル checker は双方の方向を備えている（`crates/singlefs-checker/src/walk.rs` のファイル先頭）。

**根拠**：
- 実験なし：これは「checker 即仕様」と検証経路の独立性の間のトレードオフであり、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C410 (checker の2方向での共有事項に関する検査不在)：2つの方向が「共有するのはフォーマット解析のみ」という条項に対して、赤になり得る検査が存在しない（`.claude/gate.d/` 内にこれを判定するステージがない）。

#### 確定項目 13：衝突 2：crash refinement の spec とモデル突き合わせの model

**確定事項**：**衝突 2：crash refinement の spec とモデル突き合わせの model は同一のものの2通りの記述である。**
1つに統合する → 工数を半減できるが、**独立性の要件に直接違反する**；各自で記述する → 名目上は独立だが、
通常は同一人物が同一週に書くことになる。緩和策（相関を低減するのみで排除はしない）：両者を**表現形式のレベルから異ならせる** ——
spec は「イメージに対する状態述語」として記述し、model は「op → 状態遷移」として記述する。不一致が生じた際に少なくとも相互の位置特定が可能となる。
**同源性は観測されなければならない**：「spec と model が不一致」となったイベント数および各判定でどちらが誤っていたかを統計する；
不一致が累計 ≥ 5 回で「spec の誤り」と判定された回数が 0 回であった場合、両者が実質的に同源であることを示しており、2本の経路を統合して予算を全数探索へ移管すべきである。
判定は3つの枠に分かれる：spec の誤り / model の誤り / 条項の記述不足（双方が条項のそれぞれ異なる解釈に従っており、条項にどちらが正しいかが書かれていない）。不一致が発生した場合、まず前提が成立するドメインで再判定し、依然として存在する不一致のみを記録する；「条項の記述不足」イベントはまず未済として記録し、「累計 ≥ 5 回」にも「spec の誤り」にも算入しない。条項に該当する記述が補完された後、補完された文に従って再度判定し、先の2つの枠のいずれかに帰属させる。
ここでの model は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) を指す。O3 (独立規約実行器) が同一の (履歴, クラッシュポイント) に対して判定行を出力できるようになるまで（Python 製実行器の第1版 E170 (O3仕様実行器第1版) は存在するがクラッシュポイント再生に未接続、C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)）、O3 (独立規約実行器) × RefFS のペアは空欄として登録し、イベントとしてカウントしない；O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官を身代わりとしない；未確立のペアは「未確立」と報告し、緑とは報告せず、いかなる枠にも算入しない。

**射程**：緩和策は相関を低減するのみで排除はしない。確定項目 1 で撤回された旧候補はこの条項の影響を受けない、理由は確定項目 1 の射程にある。crash refinement のコードは `crates/singlefs-replay-prefix/src/lib.rs` 内の Verus コアとその spec（プレフィックス取得、ルート選択、リングスキャンの3区間と、ルート選択およびプレフィックス取得の結合判定の系であり、確定項目 1 の M1、M2 構文層および M3 の両解釈と単調性の系をカバーする；M2 セマンティクス層および M4 は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) に依拠して有界内で判定する）；spec は具体的な状態に対して評価を行うことができず、「spec と model の不一致」イベントがどこから生じるかは未定であり、カウンタにはコードが存在しない（C191 (同源性カウンタが1ペアのみ対象)）。

**根拠**：
- 実験なし：同源性は将来の spec と model の不一致イベントのカウントによって観測されるべきであるが、spec もカウンタもまだ存在せず、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- 第3の枠「条項の記述不足」：三者判定 `research/prompts/m3-formal-r1-main-verification.md` 第2節「F2 同源性」の行（攻撃側が指摘、推論）、`research/prompts/m3-formal-r2-main-verification.md` 第2節第9行、`research/prompts/m3-formal-r3-main-verification.md` 第2節同源性カウンタの行、3ラウンド耐えた；実験なし：カウンタには依然としてイベントソースがない。
- 「model は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) を指す」および未確立のペアを空欄として登録：三者判定 `research/prompts/m3-formal-r5-main-verification.md` 第2節同源性カウンタのセル（両経路が同一方向を向いた部分）；`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定」内の身代わりペアの行（身代わりペアとはみなさない）。

**未済の検査**：C191 (同源性カウンタが1ペアのみ対象)。

#### 確定項目 14：他の決定との連鎖

**確定事項**：

| 決定 | 連鎖点 |
|---|---|
| D4 (チェックサムの位置) | ルートチェーン差分が Merkle を「支払うべき代償」から検証のてこへと反転させる；ただし形式検証においてハッシュは uninterpreted function であり、**衝突耐性は検証範囲の完全に外側にあり、「Merkle を検証した」と解釈してはならない** |
| D8 (コアインデックス構造) | ログ構造ノード、write buffer、key cache の3層はいずれもシンボリック実行を拘束し（ループ上限、遅延ウィンドウ、キャッシュ一貫性）、「Yggdrasil は1分以内」という計測条件を本プロジェクトにおいて成立させない |
| D9 (暗号化) | AEAD は SMT で解決不能であり、uninterpreted function としてのみモデル化できる；**「nonce の再利用による平文漏洩」という結果は検証できず**、残るのは「nonce が重複しない」という純粋な組合せ的性質のみである —— そしてこの純粋な組合せ的性質こそが、この検証経路の最も価値ある成果物である |
| D11 (インデックスノードにメッセージバッファを残すか) | 確定項目 1 で「残す」を選択；D8 (コアインデックス構造) 確定項目 1 が冪等完全値を選択した後は、checker の I-3.1 (割り当て済み統計の整合) はメッセージセマンティクスを解析する必要がなくなり（D11 (インデックスノードにメッセージバッファを残すか) はこれに基づきコスト 1 が本プロジェクトで成立しないと判定）、衝突 1 がこれによって「解決不能」になることはない |

**射程**：連鎖するのは形式検証と checker がこれら幾つかの決定に直面した際の境界であり、それらの決定の条項自体を変更するものではない；ルートチェーン差分および衝突 1 自体は確定項目 8、12 に帰属する。

**根拠**：
- 実験なし：連鎖はそれら幾つかの決定の確定条項から演繹された境界であり、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 15：検証コードを2層に分割 —— harness 層は常時実行、checker 層は既定でコミット時のみ実行

**確定事項**：`crates/` 以下の検証コードは「いつ実行するか」に基づいて2層に分割し、各々1つのパッケージに配置する。プールレベル checker と合わせて計3つの名称となる：**harness 層**は `crates/singlefs-harness` であり、単体テストおよび結合テストに加えてそれらが使用するスキャフォールドライブラリ（レコーダー、インメモリプール、理想モデル、ランダム履歴、故障注入デバイス、シナリオ）を含み、コード変更時に随時実行する；パッケージ内はさらに軽量と重量に分かれる：debug ビルドで単一テストの実行が 60 秒以上のものは重量とし、`#[ignore]` を付与する。単一テストの所要時間は通常の実行時に測定されたものを参照し、別途個別に測定はしない。実行したいときは随時実行可能であり、一律にメモリラッパー経由とする；1テストケース1シナリオとし、テストファイル名はテスト対象に基づいて命名しマイルストーン名には基づかない。**checker 層**は `crates/singlefs-checker-tier` であり、クラッシュ状態列挙エンジンとレコード照合器、クラッシュ列挙ケースの判定ライブラリ経由の2段階とそのカウント、検出ログとツールチェーン識別情報、1つの長期判定ライブラリ（ブロックキーは内容のみによって決定され、各種テストが各々どのブロックを必要とするかはインデックスとしてのみ機能）と2段階プロトコル、クラッシュ注入と破損ディスク入力の2つの戦役、実デバイス側でのデバイスログ比較、実験および実デバイスのハーネスバイナリ、ならびにそれらの全テストケースを含み、QEMU（ゲート checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates ミューテーション全表（checker-tier-crates-mutation-replay）、全実験再実行（checker-tier-research-build-and-replay の experiment-replay セル）を伴う：既定ではコミット時のみ実行し、単独での実行も可能（コマンドに `SINGLEFS_HEAVY_TESTS=user-request` を付与）。checker 層内のクラッシュポイント再生（ゲート checker-tier-crash-point-replay）は2層に分かれ、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay は分割しない：**クイック層**は2回実行する：`cargo test --release -p singlefs-checker-tier --lib --tests`（`--ignored` なし、`--tests` はすべての `test = true` のターゲットを含み、ハーネスバイナリのインライン単体テストもこれに含まれる）、およびもう1回は `tests/` 内の `cfg` で指定されたフィーチャを有効化して実行（`--features <フィーチャ> --lib --test <フィーチャ付きテストファイル>…`、フィーチャに `gpu` を含む実行は `scripts/crash/gpu-quiesce-run.sh` 経由で起動）（ゲート checker-tier-crash-point-replay はゲート全体の実行時およびコミット時の双方でこの2回を実行し、2回目はその feature-gated-tests セルである）、**フル実行**は `.claude/gate.d/stage-inputs.tsv` にキー `crash-case:` として登録されたテストケースを1件ずつ `--include-ignored --exact` で実行（checker-tier-crash-point-replay `--full`）、コミット時には既定で実行せずユーザー要求時に実行し、実行方法は確定項目 9 の階層化に従う；クラッシュ増幅判定器はシステム全体が GPU に移行しており、すべてのプールレベル checker 不変条件をカバーする（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**プールレベル checker** は `crates/singlefs-checker` であり、判定器ライブラリとして機能し、上記2層から呼び出される。依存関係は一方向のみ許容される：checker 層は harness 層とプールレベル checker に依存し、harness 層は checker 層に依存しない（ゲート checker-independence-and-sync の checker-implementation-disjoint セル第 ④ 項が判定、dev-dependencies も含む）；双方の層が必要とする短いコードは各々個別に保持することを厭わない。クラッシュ列挙ケースは一律に `crates/singlefs-checker-tier/tests/` 内に記述する（`scripts/crash/crash-case-check.py` が判定）。ヘビー層テストの関門はパッケージ単位で判定する：`singlefs-checker-tier` に到達するテストが checker 層である。

**射程**：`crates/` 以下のコードとテストがどのパッケージに属するか、いつ実行するか、ゲートが何に基づいて識別するか、3つのパッケージをどう呼称するか（`.claude/rules/verification.md`「定義と名前」）を管轄する；各不変条件が何を判定するか（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、列挙ドメインをどう定義するか（確定項目 4）、プールレベル checker と実装の間で何を共有するか（確定項目 5：プールレベル checker は依然として `singlefs-format` にのみ依存する）は関知しない。アップストリームの SOP には本プロジェクトの検証手段をこれ以上記載しない：クラッシュポイント再生、モデル突き合わせ、checker 即仕様、ファイルシステム特有の逆推論ギャップの4節および `gate.sh` の未実装リスト内のプロジェクトキーはすべて本リポジトリに集約された（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`）。アップストリーム側の変更はレプリカに同期済みであり、未リリースである。分割ラウンドの記録は `records/2026-09-27-验证两档拆分.md` にある。

**根拠**：
- 実験なし：いつ実行するか、どのパッケージに属するかはプロセスポリシーであり、測定可能な量はない；2層それぞれの判別力はそれぞれのテストケースおよび変異表によって証明される。
- ユーザー決定（2026-09-27）、原文は変更履歴にある；三者 `verification-split-r1`（判定 `research/prompts/verification-split-r1-main-verification.md`）は `singlefs-checker-tier` 分割前のバージョンを攻撃したものであり、パッケージ分割後の変更はゼロラウンド攻撃されている。
- ユーザー決定（2026-10-01）：フル実行は時刻を制限せず、ユーザー要求時に随時実行、「夜間」を削除、原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 16：セグメント単位で登録されたセグメントの全領域が u64 に収まらなければスライス判定し、状態数には 1280 ビット固定幅整数を用いる

**確定事項**：
- スライス分割：クラッシュ増幅においてセグメント単位で登録されたセグメント（`.claude/gate.d/crash-point-replay-segments.tsv`、1行1セグメント5列：項目名、ストリーム名、セグメント番号、セグメント書き込み数、セグメント書き込み表ダイジェスト）について、全領域の状態数が u64 に収まらない場合は固定サイズのスライスごとに順次判定する；スライスサイズは環境変数 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` で定義され、未設定時は 2^32 状態とする；スライスの起点は判定ライブラリ内の再開点とする（再開点はセグメント書き込み表ダイジェストと判定器ユニットキーによってアドレス指定され、項目名、ストリーム名、セグメント番号は表示用に過ぎない）。全領域が u64 に収まる登録セグメントはスライス分割せず、全領域を一度に判定する。コード配置先：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- ビット幅：登録セグメントの総状態数、スライス起点、スライス末尾、および進捗には一律に 1280 ビット（10 個の u128）固定幅符号なし整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`、手足の数は同ファイルの `STATE_COUNT_LIMBS`）を用い、演算はすべて checked とし、飽和や切り捨ては行わない；ドライバ行 `registered_segment_progress=` の判定済み状態数および総状態数はすべて正確な10進数で記述し、指数の形式では記述しない。
- 登録表をエンジンに渡す方法：エンジンは環境変数 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 経由でのみセグメント登録表を読み込み（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`、`fn registered_segments_of_the_flow_from_the_environment` 内）、未設定時は1セグメントも登録されていないものとして扱い、他の逃げ道は設けない。144 ユニットのファイル順次書き込みが 145 個となり extent 下段が2層に成長したストリーム（エンジンストリーム名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）のユニット書き込みセグメント（322 回のユニット書き込み）およびレコードセグメント（290 回の journal レコードインプレース書き込み）の双方が登録されている。

**射程**：クラッシュ増幅エンジンがセグメント登録されたセグメントをどうスライス分割するか、状態数をどの整数で記録するか、登録表をどこから読み込むかを管轄する；どのセグメントを登録すべきか（ユーザーが承認し登録表のファイル先頭に記述）、登録セグメントのドメイン自体をどう展開するか（判定 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` の J10）、各クラッシュ状態をどう定義するか（確定項目 4）は関知しない。未登録のセグメントの状態数、およびストリーム全体を合算した状態数は依然として u64 に収まる必要があり、収まらない場合はライブラリ書き込み前に実行を拒絶する；登録セグメントの全領域が 1280 ビットを超えた場合も同様にライブラリ書き込み前に実行を拒絶する。スライスサイズは 1 から 2^32 までのみ許容される：2つの GPU カーネルがスライス内インデックスを u32 としてスライス起点に加算するためである（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；ライト層（`scripts/crash/crash-point-replay.sh --quick`）の呼び出し元がスライスサイズを設定していない場合は `LIGHT_REGISTERED_SEGMENT_SLICE_STATES` に設定する。展開上限でカバーできず未登録のセグメントがある場合、増幅エントリ `scripts/crash/crash-amplification.sh` は赤と判定する；クラッシュポイント再生エントリ、増幅エントリ、および項目ドライバは実行開始前に本テーブルを照合した上でエンジンに渡す。

**根拠**：
- E161 (クラッシュ増幅の重複排除とセグメント分割所要時間)「第2回再実行：サンプリングドメインを11セグメントのストリームに従って再定義（2026-10-05）」の節：セグメントのフル実行状態数と閉じた式、ハーネスによる個別の数え上げ、crates の計画がセグメントごとに等しく、状態数はセグメント書き込み数に伴い指数関数的に増加するが、これは「長いセグメントの全領域は u64 に収まらない」という前提を支持するに過ぎない；スライスサイズ、ビット幅の値、登録表の読み出し方法はエンジニアリングパラメータでありユーザーが決定したものであって、本実験が支持するものではない。
- ユーザー決定、原文は変更履歴にある：スライス分割と1スライスあたり 2^32 状態（`records/2026-10-11-崩溃放量两段与脚本审计.md` 96行目）、ビット幅 1280 ビット（同記録 98, 99行目）、両セグメントの登録（同記録 92行目）。
- スライス分割とビット幅は判定 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` の J10（セグメントごとの登録）に対する補足であり、三者による攻撃は経ていない。

**未済の検査**：なし。

#### 確定項目 17：判定再利用のユニットは関数であり、キーは関数ユニット、ツールチェーン、依存バージョンのみで構成する

**確定事項**：
- 重型ゲート、クラッシュ判定ライブラリ、および実験受入における各判定（クラッシュ状態ブロック、クイック層およびモデル突き合わせの各テスト関数、crates ミューテーションの各行、広域ミューテーションの各行、形式検証の各条、QEMU の各階層、litmus の各項目、研究再実行の各行、実験ミューテーション表の各条、実験受入）を踏襲するか否かは、そのユニットキーのみを参照する：それが依存する関数ユニットの各コメント除去後指紋、クロージャ内の各ユニット（mod ヘッダを含む）の `#[cfg]`、`#[cfg_attr]`、`cfg!` が実際に参照する cfg アトムに今回のターゲット下での真理値を付加したもの（クロージャが参照しないアトムはキーに含まれない：`gpu` フィーチャの有無でコンパイルされた2回の実行において、クロージャが `gpu` に言及しない判定キーは同一となり、ライブラリを共有する）、ツールチェーン（cargo 解決後の有効ビルド環境および外部ツールバージョンを含む）、`Cargo.lock` から解決されたクロージャが実際に参照する依存バージョン、ならびにその判定自体の内容入力（セグメント書き込み表と開始イメージのダイジェスト、litmus の内容、ミューテーション原文と置換文など）を加える。関数ユニットはその判定のエントリから呼び出しと参照を辿って収集し、参照される定数、型、静的項目、マクロ展開を合わせて計算する；wgsl は fn 単位で区切る。
- 一律にキーに含めず、再利用の根拠とすることも禁ずるもの：コミット、HEAD、ツリーに基づく変更範囲；ファイル全体の原文；ファイル単位、パッケージ単位、テーブル全体の指紋またはクロージャ；手書きのバージョン定数；パス（リポジトリルートも含む）、名前、シード文字列、行番号、リソース上限、タイムアウト；全工程またはセル全体のマークのみで判定ごとのライブラリを持たないもの。キーが狭すぎる（参照される定数、型、マクロの収集漏れ）、ライブラリが失われやすい場所に置かれている、改名によりライブラリが消失する、2つのライブラリで同一バッチを重複判定することも同様に違反とみなす。
- 完全に収集できないもの（動的ディスパッチ、マクロ、`include_str!(concat!…)` などで認識不能なもの）はその判定のキー計算を拒否してそのまま実行し、暗黙のうちに収集を省略することはしない；各キーはそれを構成するユニット一覧とともにディスクに保存され、無効化された際にはどのユニットが変更されたかを報告できるようにする。
- 判定の種類ごとに1つのライブラリを永続ディレクトリに固定し、1回の実行ごとや判定手法ごとにディレクトリを変更しない；リポジトリルートもライブラリの入力ではない：クラッシュ判定ライブラリのディレクトリが環境変数 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` で指定される際は絶対パスのみを受理し、相対パスは拒絶する；フル実行スイッチは書き込みのみで読み出さず、オンデマンドで判定されたものはキーに従って同一のライブラリに書き戻す；二台目で判定された結果はローカルのものにマージされる。
- キーの変更により古いライブラリが無効化されることはコストとはみなさない：互換マイグレーションは行わず、古い行への追記署名も行わず、変更後は再実行する。

**射程**：重型ゲート、クラッシュ判定ライブラリ、および実験受入の再利用キーを管轄する；静的ゲートの「新規追加行のみ判定」の変更範囲（`scripts/gate/changed-paths.sh`、これは判定基準のセマンティクスである）や preflight の `admission: inputs-changed` は関知しない；列挙ドメイン（確定項目 4）、階層化、およびいつユーザーにフル実行を促すか（確定項目 9）は関知しない。イメージ判定および特定状態判定のバージョンは、判定定義セット（CPU 参照判定器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` のクロージャ）のキーであり、そのブロックが CPU 上で判定されたか GPU 上で判定されたかを問わない：GPU 判定器は加速経路であり、その受入は定義セットとの突き合わせ（クイック層の GPU と CPU の突き合わせテストケース）に依拠し、そのユニットはブロック属性に出典としてのみ入り、キーには入らず、種別も分けない；GPU カードのドライバ、カード型、複数枚のカードの分担もブロック属性や実行時パラメータとして残し、キーには含めない。

**根拠**：
- ユーザー決定（2026-10-15）、原文は変更履歴および `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 ユーザー決定：コミットは再実行と無関係、再実行はコード変更のみに従う」の節、「2026-10-15 バッチ3」の各段にある。
- ユーザー決定（2026-10-17）：判定はハードウェアや CPU / GPU とバインドせず、再利用と枝刈りはデバイスを参照しない；原文は変更履歴および `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 のフル実行停止に関する各段にある。
- メイン agent 決定（2026-10-15）：リポジトリルートはキーやライブラリに含めず、ライブラリディレクトリは絶対パスのみを受理、cfg 成分はクロージャが実際に参照するアトムのみを取得、出所は同記録「2026-10-15 検証フェーズ第1ステップ赤判定、2条の決定、4-3 4バッチ派遣」の決定第1, 2項。
- 反証：E164 (クラッシュ増幅新設計の小規模機能テスト) F4 において、関数のみに基づいて指紋を取得し参照される定数を収集しなかった結果、V5 と V11（V11 は定数 `DATA_POINTER_BYTES` のみを変更）の判定を漏洩したことが測定された：関数ユニットは参照される定数、型、静的項目、マクロとともに計算しなければならない。
- ディスク上の数値：`research/prompts/pruning-survey-2026-10-15/ledger.md`「1の2、ディスク上で測定されたもの」の節 —— 名前のみが異なるブロックが個別に保存・判定されたものが 4100 ブロックおよび 9679 ブロック；ライブラリ全体で一括の判定法ダイジェストを変更した際に累積で 3 935 242 934 状態の追加判定が発生したが、そのクロージャ内で変更された 29 ファイルのうち 24 ファイルはコメント除去後に逐語的に同一であった：キーに名前を含めたり、ファイル単位やバージョン全体で指紋を取得すると、同一の判定が重複して再実行されてしまう。
- 経路状況記録なし：三者による攻撃は経ていない；ユーザーはコード改修後に3ラウンドの攻撃側を開始し、「枝刈りがアトミックであるか、キーが関数ユニットのみで構成されているか」のみを攻撃対象と決定した。

**未済の検査**：C8 (ゲート適用範囲の判定不能)、C489 (singlefs-core の広域ミューテーションが今日のコードに対して結論を持たない)、C528 (2つのゲート経路で判定される変更ウィンドウが異なる)、C564 (残る6本のクラッシュポイント再生フル実行が誰も走らせていない)、C605 (クラッシュ増幅の再利用と枝刈りで無効化して再判定する対照がない)、C630 (コミット時の形式検証ステップ1で停止する頻度が未計測)、C635 (前回のフル実行に機械可読なマーカーがない)、C645 (クラッシュ増幅フル実行を誰もディスパッチできない)、C668 (ランダム履歴スキャフォールドの導入元ホワイトリストがエイリアス記法で迂回可能)、C777 (5つの実験キーが stage-inputs.tsv に登録行を持たない)、C836 (staged-green に紐づくツリーが判定されたツリーとは限らない)。

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:445-448`（整段抄，未转述）**

```markdown
## 履歴バージョン

D13 (検証ルート) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/13-验证路线.md:1-6`（整段抄，未转述）**

```markdown
## D13 验证路线 —— 已定

**中文** · [English](13-verification-route.en.md) · [日本語](13-検証ルート.ja.md)

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、有几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:29-88`（整段抄，未转述）**

```markdown
#### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」

**定案**：crash refinement 的第一个验证对象是恢复函数 `Recover` 的关系性质，最小封闭单元是「一次发布 + 一次崩溃 + 一次恢复」。四个里程碑的主语 `Recover` 指 `crates/singlefs-core` 里以 `verify_named_units = true` 调 `replay_journal` 的三处（`recover` 的 `Consult` 策略、可写挂载读阶段、只读挂载），三处的 `records` 都取 `scan_journal` 的输出；`Ignore` 与 `ConsultWithoutNamedVerification` 是测试与对照用的臂，写进可信部分；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 不是产品调用方。四个里程碑依次是：

| 里程碑 | 义务 | 落点 |
|---|---|---|
| **M1 全性** | `Recover` 在 D13（验证路线） 已定项 4 枚举的**每一个**崩溃状态上有定义、终止、输出一个能过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像 | 脚手架。**完成判据不是「证出来了」，是「M2 能在它上面写出来」**——崩溃点重放本就全量枚举（[verification-build.md](../verification-build.md)「崩溃点重放不抽样、全量枚举」），恢复卡住当场暴露，单独证它没有边际价值。取前缀、择根、扫环三截已进 Verus，都在 `crates/singlefs-replay-prefix/src/lib.rs`：取前缀核 `replay_core`（终止与不溢出在核内证到，循环都带 `decreases`；`requires` 两条——记录表按 (实例代号, 计数器) 升序、计数器有后继——由胶水 `replay_journal` 里的断言兜），择根核 `choose_root_slot`（自证过的槽里取 (`checkpoint_txg`, 实例代号) 最大、键相同取槽序最前），扫环核 `first_copy_of_each_record_key`（同一 (实例代号, 计数器) 先到先得），择根与取前缀的合判写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`；门禁 checker-tier-formal-verification 判证明，登记数 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165（恢复重放有界穷举与变异杀伤界） 当轻量工具，它第 3 次重跑第二段的择根合判族在内存盘上调真 `choose_root`、`scan_journal`、`replay_journal`。择根序的四个改坏变体在合法历史 K 族里只有「取第一个有效者」红，另三种（比较不看实例代号、txg 平局取低、实例代号为主）由 Verus 必红世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 择根那一节）与对抗根环 KA 罩（第 3 次重跑第三段）。核之外：择系统配置（`choose_system_configuration`）与两截胶水（择根的读槽、自证校验和、fsid、区域到盘的映射、槽的遍历次序；扫环的偏移枚举、提示、解析与 CRC、盘的遍历次序）不在 Verus 里，崩溃点重放与崩溃放量的 CPU 判器在每个崩溃状态上调 `recover` 罩它们（GPU 判器用 WGSL 另写一份 `replay_journal`，验不到 core） |
| **M2 前缀不切发布** | **任一 `jsn` 连续前缀，其重放结果等于某个整数个已发布的 checkpoint 的结果——不存在被前缀切开的发布；前缀停在一次发布中间时，那次发布整体不施加**（D16（发布语义） 已定项 4「一次发布整体施加或整体不施加」）。分两层。**句法层**（对任意自证过的记录表，无前提）：施加的记录是若干整次发布，在飞上限与提交标记逐条判过，施加的计数器从链首起逐一连续（锚点读得出时从锚点加一起；读不出时首条 `checkpoint_txg` = 所选根 + 1、序号 1）；反向链与点名单元是读盘回调的结果，写成「被施加的每条记录，回调交回了真」。**语义层**（前提：盘上每条记录出自这一版的合法写侧，即 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族；同一 (实例代号, 计数器) 在全部时间线上只对应一份内容）：「结果」是根的投影加盘，不是整条根——返回的根的 (实例代号, `checkpoint_txg`) 与新根段四个字段等于所选根那条时间线上的第 j 版（j 不早于所选根），其余字段等于所选根，卸载记号除外（由记录重建的根取「不是卸载那一串写下的」）；盘上 (所选根, j] 每一版的记录读得出、点名单元两份都验过、记录条数之和不超在飞上限 | 落点是 I-8.3（重放前缀严格连续）——它的判据补句逐字就是这一条（「前缀不得停在一次发布的中间——一次发布整体施加或整体不施加」），不要在 [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 里另立一条。句法层在 Verus 核 `replay_core` 的 `ensures`（`replay_follows_the_prefix_rules`）里，必红世界登记在 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`，其中「改判定」行只改可执行判定、不碰 spec；语义层靠 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族在界内判，Verus 不证。核之外：胶水（摊表、包回调、拼报告与新根）由 E165（恢复重放有界穷举与变异杀伤界） 直接调真 `replay_journal`、harness 用例与 `crates/mutations.tsv` 的变异行罩，拼根里投影之外的字段（卸载记号、实例表指针、分配记录树根）没有变异行、E165（恢复重放有界穷举与变异杀伤界） 的生成器把它们写成定值；两个读盘回调（反向链、点名单元闭包）由 E165（恢复重放有界穷举与变异杀伤界） 里真回调、内存盘的读数、`crates/mutations.tsv` 的变异行与只读挂载几种形状的读盘次数用例罩 |
| **M3 幂等与单调** | **幂等**：`Recover(Recover(c)) = Recover(c)`，两读都证——记录级（同一条记录施加两次与施加一次，得到的根相同，I-8.4（重放幂等） 的字面）与不动点（把重建出的根当所选根、同一张记录表、同一个读者、同一个开关再重放一次，返回的根不变；前提是所选根之上的记录不超在飞上限，且记录表满足 I-8.12（同实例记录的 txg 不减）：同一实例里 journal 记录的 txg 随计数器不减）；第二次因所选根 `checkpoint_txg` 顶格报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` 且一条没施加的，算相等。「就是 I-8.4（重放幂等）」不成立：I-8.4（重放幂等） 是记录级一读，不动点是另一条性质。**单调**（M2 的推论，不单独证）：串行提交（已定项 2）、D16（发布语义） 已定项 7 的持久顺序、所选根取根环里最新的有效根之下，同一次运行里 D13（验证路线） 已定项 4 的两个不带故障的崩溃状态 `c ⊆ c'`，`Recover(c)` 落到的版本不晚于 `Recover(c')` 落到的版本，版本按 (`checkpoint_txg`, 实例代号) 比，与 D22（单元原子性怎么合成） 已定项 7 的择新序同一个序；开并行提交或把域放宽到带故障时重开。「施加的事务集包含于」那一读法撤：它对理想恢复也为假 | Verus 核 `replay_core` 的 ensures 证两读：记录级与不动点（不动点的前件含 I-8.12（同实例记录的 txg 不减））；单调写成推论 `corollary_the_landed_version_does_not_go_back`，择根与写侧那两条前提写在它的 requires 里。E165（恢复重放有界穷举与变异杀伤界） 的对抗一族在界内判不动点，违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上（第 3 次重跑第一段）。checker 那一半（I-8.4（重放幂等））已实现，取在崩溃注入的二次崩溃上判、不进池级 checker 的清单：`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `REPLAY_IDEMPOTENCE_INVARIANT` 与 `judge_the_remount_after_a_second_crash`。O2（独立解析器 + checker） 判不了幂等：I-4.5（已停用·需两镜像） 退役理由逐字「幂等要比对两个镜像，单镜像判不了」 |
| **M4 不少施加** | 前提与对象同 M2 的语义层。`Recover` 返回的根恰是 j\*：从所选根起每一版都完整在盘上（记录任一份自证过、点名单元两份都验过）、记录条数之和不超在飞上限的最远一版。不加这一条，M1–M3 全部证出来也排除不了「已确认的完整发布被恢复丢掉」：一条都不施加的恢复过 M1–M3 的全部字面。「不少施加」按「多出违例」读：变体多出违例才算被它挡住 | E165（恢复重放有界穷举与变异杀伤界） 的最远一版判据在界内判；Verus 的必红世界里「第一条水位之上的记录就停、一条都不施加」那一行红；M4 自己不单独证 |

**挂载透明**（独立的第四条性质，不是 M3 的第三读）：在崩溃镜像上只读恢复一次得 R1；在同一份镜像上做一个动作——只挂载（取号、写行、暖机、挂载续做与挂载自己推的抬 F），或挂载加一次发布——之后再只读恢复一次得 R2，逐字段比 R1 与 R2。比的那一版：只挂载比恢复落到的那一版，挂载加一次发布比挂载后那次发布之前的那一版。挂载途中三个相位的二次崩溃镜像 img2 上也判：C(recover(img2)) = C(v1)（v1 是 R1 落到的那一版，C 取 POSIX 内容与存在性），内容与字段两半都判；字段一半里，落回 v1 那一版自己的各字段与 R1 相同，落到这次挂载写的某一版的照逐字段表判，扫描报告不判，抬 F 第一个写起 F 不判、在它之前（含暖机）F 照「挂载没推抬 F」那一格判；抬 F 第一个写的位置按挂载交回的抬 F 那几串（准入推的与续做里推的）在挂载写表末尾的位置取，取不出（`NotLocatedInTheMountWrites`）时这一次挂载的二次崩溃整段 F 记不判。逐字段表（R2 对 R1）：

| 字段 | 判法 |
|---|---|
| POSIX 内容与存在性 | 必须相同 |
| 池身份与系统配置的不可变配置（本盘设备号除外） | 必须相同；不可变档 37 行除本盘设备号一律按原字节比 |
| 系统配置的节点大小 | 必须相同 |
| 系统配置的运行配置 36 字节（T_time、T_dirty、三水位） | R2 等于内置默认值（没有挂载选项通道，`to_slot` 总写默认） |
| 卸载记号 | R2 恒为「不是卸载写的」，R1 的记号不进比较 |
| F 与 F 生效值 | 挂载没推抬 F：R2 的 F 生效值与 R2 根里的 F 都等于 R1 的 F 生效值；推了（准入推的与续做里为空间推的都算）：R2 的 F 生效值等于挂载交回的最后一串抬到的 F |
| 实例代号 | max(各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1，两个量都从崩溃镜像读 |
| 实例表行 | 照 D18（块里携带什么信息） 已定项 11 恢复写的行 |
| `checkpoint_txg` | 只判下界：≥ max(根环全部根的 txg, 环里全部自证记录的 `checkpoint_txg`) + 1（D23（journal 的角色与格式） 已定项 14 的下界，允许跳号） |
| 根槽位置 | 不另判，由 `checkpoint_txg` 定 |
| 树 ID 水位 | 挂载不发第一个文件时相同 |
| 树表里 inode 树、extent 树、墓碑、谱系、deleted_inodes 的根指针（后三样是树表条目，种类 9、10、11，不是根记录字段） | 相同；挂载续做过时，按续做之后那一版判：等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的各根指针；树表单元自己的落点不比 |
| 树表种类 6、7、8、12 的根，诞生 txg、头 ID、树 ID、种类 | 必须相同（种类 6、7、8、12 的根恒全零）；诞生 txg 跨根另有 I-9.14（树表条目的诞生 txg 跨根不变），头 ID 另有 I-9.27（树表条目的头 ID 取值），单镜像判 |
| 实例表与树表的单元指针 | 不比字节，与「树表单元自己的落点不比」同 |
| 扫描报告 | 施加 0 条、水位之上 0 条 |
| 根槽偏移 428..457（算法、nonce、MAC） | 必须为 0；单镜像性质，归池级 checker 的 I-2.4（头校验和覆盖范围）判，挂载透明不比 |
| magic、flags 其余位、自证校验和、整槽校验和 | 不比（结构性，读槽时判） |
| 系统配置槽世代号 | 不比，交 I-7.16（系统配置槽世代号与槽号的关系） 在 R2 的镜像上判 |
| 系统配置的 journal_tail | R1 ≤ R2 ≤ R2 环里自证记录的最大计数器，挂载做完取等 |
| 系统配置的 journal 实例代号 | 挂载做完等于 R2 根的实例代号；二次崩溃时取 R1 的实例代号或新实例号两者之一 |
| 分配记录（含根记录的 `mapping_root`、`allocation_record_tree_root` 与树表种类 3、4 的根） | 不比整表：POSIX 内容相同，且池级 checker 对 R2 的镜像判绿；`mapping_root`：R2 全零 ⇔ R1 全零；`allocation_record_tree_root`：R1 带树 ⇒ R2 全零，R1 树表 0 条 ⇒ （R2 非零 ⇔ 新实例号 ≥ 2） |

系统配置的运行量（槽世代号、journal_tail、journal 实例代号）落回 v1 时照它们自己那一行判，不按 v1 相同判。

**射程**：refinement 的边际价值只在崩溃点重放枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数；崩溃点那一维崩溃点重放已全量枚举。「校验和过了」在模型里是一个布尔标志，**不许读成 refinement 验了完整性**——与已定项 14 里 D4（校验和位置） 那一行「不许读成『验了 Merkle』」同形，也与 D13（验证路线） 已定项 4 对 CRC32C 碰撞概率的「知情接受」是同一笔账。D13（验证路线） 已定项 4 是 refinement 左边的 `Crash`（模型层的枚举域），不是右边的 `Spec`，拿它当 spec 会让 refinement 恒真。I-6.5（nonce 水位单调） 的形式化撤下；D18（块里携带什么信息） 已定项 14 定了、且 nonce 分配规则（单调计数器 + 预支水位、水位在使用前持久）被立成一条分项之后，这一格重开——重开时它唯一的真优势是 nonce 对 POSIX 不可见，它的 spec 与 O1（内存模型对拍（HashMap 理想文件系统）） 的 model 结构上不可能同源，不受冲突 2（D13（验证路线） 已定项 13） 影响。

M1–M4 的前提与域：语义层只挂「同一 (实例代号, 计数器) 在全部时间线上只对应一份内容」，由 D23（journal 的角色与格式） 已定项 14「这一版的失败处置」的原样重发担保，根槽写失败推进 `checkpoint_txg` 再发那条路落地时重核；前提域取 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族（合法写侧写出的历史的任一落盘子集，较新的根槽可以读不出）。M2 的「其余字段等于所选根」里，由记录重建的根的卸载记号是 `NotWrittenByTheUnmountSequence`（D22（单元原子性怎么合成） 已定项 7「其余根写 0」），与所选根的记号可以不同，所以写「卸载记号除外」。M3 的单调推论只在串行提交、不带故障的崩溃状态里成立。M4 连同择根才排除「已确认的」完整发布被恢复丢掉，前提要全：不带故障的崩溃状态（已定项 4 的枚举域）、所选根按 D22（单元原子性怎么合成） 已定项 7 的择根取、链不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1）、新实例本实例写成的根覆盖两块盘之前不让任何 fsync 返回（D16（发布语义） 已定项 8 的暖机）；带故障的域里不排除——在飞上限会截掉记录完整的发布，一块盘整块掉了时点名单元少一份的那一版不施加（C519（丢一整块盘时恢复丢掉刚确认的那一版））。

挂载透明的射程：只管不带故障、可写挂载做成的崩溃状态。可写挂载被拒的状态不判透明，判拒绝本身合不合法：拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立，且盘上逐字节不变，且只读读出在允许集合里；拒因不在单子上的照崩溃注入第二截的写法报红。带故障的状态不在射程：取号那一刻有一槽读不出与 N-配置报成同一个拒因，接进带读故障的档之前要分开（C664（挂载拒绝合法复算把两种拒因报成一个））。逐字段表里树 ID 水位、两棵树的根指针、`checkpoint_txg`、扫描报告四格是推的（三方判决取辩方的式子，没量），F 那一格「推了」的一半没量；根记录的 `mapping_root`、`allocation_record_tree_root`，墓碑、谱系、deleted_inodes 三根指针，算法、nonce、MAC，系统配置的槽世代号与 journal_tail 各归哪一类，照 C655（挂载透明逐字段表有字段没归类）、C687（从 mkfs 水位建树那一版落点判不了的几格） 判决 Y1、Y2 写在逐字段表里。

**依据**：
- E42（一事务几条记录）：为了让「被切开 ⇒ 非法」成为恒等式而钉死了载荷，「被切开却碰巧合法」这一类它结构上看不见——M2 把载荷符号化补的正是这一格。
- 三方：材料 `_d13-item1-background.md`，三条腿的产出 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；三腿一致把原候选排最后，两条云端腿各自收敛到「前缀判定与提交原子性」这道接缝，主 agent 按封闭性把择新与发布原子性收进同一个单元（判决与撤下原候选的三条门槛原文在变更史）。
- E165（恢复重放有界穷举与变异杀伤界）：在界内对真 `replay_journal` 判 M2 的语义层、M4 与 M3 两读，并量变异杀伤界；用户 2026-10-01 定它当轻量工具；C 域合法崩溃状态数按两种切法分写（甲切法是 D16（发布语义） 已定项 7 的屏障位置）；对抗一族里不动点的违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上，谓词成立那一边 0 违例（第 3 次重跑第一段）；择根合判族调真 `choose_root`、`scan_journal`、`replay_journal`，界内、合法、不带故障时已确认的完整发布不丢（第 3 次重跑第二段）；择根序的四个改坏变体在合法历史 K、K9、KW 三族界内只有取第一个有效者红，另三种在对抗根环 KA 上红（第 3 次重跑第三段）。
- 三方判决（M1–M4 的措辞与落点）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第二节、`research/prompts/m3-formal-r3-main-verification.md` 第二、三节；M3 单调推论的四步推导在 `m3-formal-r2-opus-output.md`「3.2 推的：串行提交下，j1 与 j2 推得出落到的版本单调」一节。
- 三方判决（M3 不动点前提补 I-8.12（同实例记录的 txg 不减） 与落点现状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第二节不动点前件那一格（打中条款、不打代码）与第三节第 5 条；装置的前提比 Verus 的前件宽、违例全落在该谓词不成立一边的对账在调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 10 批「10-5 交回」那一条。
- 用户定案（2026-10-01 弹窗，都照推荐）：卸载记号除外、M4 射程句的前提写全、单调的版本序取 (`checkpoint_txg`, 实例代号)、txg 顶格取「报 no_successor 且一条没施加算相等」；写回等 K10 判完一起做，原话在变更史。
- 挂载透明：无实验：判法在崩溃注入里（`crates/singlefs-harness/src/mount_transparency.rs`，checker 档的崩溃注入调它），不是实验；三轮判决里攻方的数在副本装置上量，代码轮第二轮攻方在入库代码上量的第一次崩溃 147 806 态、二次崩溃 234 773 态 0 红主 agent 没复跑。
- 三方判决（挂载透明）：`research/prompts/m3-formal-r4-main-verification.md` 第二节「挂载后再恢复：模掉什么」「比的对象与射程」两行与第四节第 2、3 条（独立的第四条、比的那一版、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条（逐字段表、二次崩溃比 v1、拒绝合法的条件）；`research/prompts/m3-formal-r6-main-verification.md` 第二节 U5、U6 两行（txg 只判下界、拒的判据在崩溃镜像上自己复算）与「逐字段表没碰的字段」一行；`research/prompts/m3-formal-code-r2-main-verification.md` 第二节 K2 各行（实现照判、站住；暖机与挂载推的抬 F 拆段）。
- 用户定案（挂载透明加二次崩溃，2026-10-02 弹窗，原话在变更史）：`research/prompts/m3-formal-r4-main-verification.md`「用户定案（2026-10-02）」二次崩溃那一行。
- 择根序变体那一句：调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条（主 agent 判补这一句）。
- 挂载续做算挂载的一部分、F 的界按挂载交回的抬 F 在写表末尾的位置取：用户定案 2026-10-03「续做算挂载的一部分」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条（界按写流位置取，找不到不 panic）；实现 `crates/singlefs-harness/src/mount_transparency.rs`。

**欠**：C77（重放起点未定义）：在择新与前缀判定那道接缝上；C649（Verus 判不到的两样回调结果）：已还清（M2 语义层的反向链与点名单元仍只能写成后置条件，Verus 一侧不判，两个读盘回调改由 `crates/mutations.tsv` 的变异行罩着，见欠账表已还清表）；C650（择根与重放没有合判）：M4 的射程句要择根与重放合起来才成立；C653（扫环胶水内存随盘数涨）；C654（必红世界期望条数随求解环境漂）；C655（挂载透明逐字段表有字段没归类）：已还清（逐字段表照判决 Y1、Y2 补齐，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:89-102`（整段抄，未转述）**

```markdown
#### 已定项 2：先串行提交

**定案**：事务层先承诺串行提交——`.claude/rules/fs-design.md`「判据写不出来时，先只做一条，而且**先做慢而正确的那条**」落到实处，串行是本工程明令的默认值，不是在两条路里挑了一条。越过交点再开并行，而不是在交点之前就承诺并行。对将来并行提交的一条前置约束：记录按事务号顺序追加，事务号 N 的 fsync 在返回前必须等 < N 的事务全部已追加 ∨ 已作废 ∨ 已确定实例切换或转只读（作废是提交管道的内存状态，对并发后继可见）；串行提交下这条规则空转。

**射程**：串行的天花板在一次 checkpoint 约 1.1 万条条目（单线程合并耗时追上一次设备同步的交点，算式在 C58（串行提交撞天花板没人报警））：低于交点设备同步是瓶颈、串行不吃亏，高于交点单线程合并变成天花板，没有安全余量。已定项 2 不决定记录头的字段清单、不改变新池新建文件写出的字节，它决定性能天花板与 crash refinement 的范围。并行提交仍然会动崩溃语义：「前缀」是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A；记录头已带事务边界字段（D23（journal 的角色与格式） 已定项 7：事务号 8 + 提交标记 1）。

**依据**：
- E17（write buffer 合并上限）：单线程 sort-merge 的上限是交点的一半；按叶分区的无锁并行是越过交点之后的逃生口，但不许默认它够。
- E44（序号位宽的本机实测与代价）：本机 fsync 率是交点的另一半。
- E42（一事务几条记录）：串行下一个事务跨多条记录同样会被前缀切开——决定记录头字段清单的是「一个事务恰不恰好产生一条 journal 记录」，不是串行 / 并行。
- 用户定案，记在变更史；将来并行提交那条前置约束来自 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1。没走三方论证：这是确认一条已经写明的默认值，真正要走三方的是何时改成并行，那一步没做。

**欠**：C58（串行提交撞天花板没人报警）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:103-135`（整段抄，未转述）**

```markdown
#### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义

**定案**：不做只管功能的内存模型，做一个自己处理持久化与崩溃语义的参照实现（RefFS），把 RefFS 从「储备候选」升为**要做的东西**。「做到极致」取语义级：RefFS 只在内存里按 POSIX 操作走，自己定义每次 fsync 之后什么算持久、崩溃之后允许看到哪几种 POSIX 状态，没有盘上格式；带自己盘上格式、自己写盘与恢复的那一档（工程量接近再写一个文件系统）不做。RefFS 与实现不许共用崩溃点枚举的那段代码：做到极致之后「三 oracle 合谋测试」（已定项 6）防的那个共同误解，最危险的落点是「RefFS 与实现是否共享同一个崩溃语义的错误理解」。

RefFS 的来处与边界：

| 项 | 定案 |
|---|---|
| 来处 | 只照 kb 条文另写：不拷 O1（内存模型对拍（HashMap 理想文件系统）） 的代码，不读录制流，不调切段、状态计划与实现的崩溃点枚举 |
| 放哪 | 单独一个包 `crates/singlefs-reference-filesystem`：依赖闭包（dev-dependencies 也算）⊆ {它自己, `singlefs-format`}，能只用 std 就只用 std；源码不引 `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier`（经改名的别名同样算）；`singlefs-core` 与 `singlefs-checker` 的依赖闭包里没有它。三条由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判 |
| 输入白名单 | 都由调用方交入，只收五样：每个操作的结局（成、在任何写之前拒、写了之后报错、原样重发）；崩溃落在第几个操作里（hi）与它的归属规则；回退目标 m；每次恢复读出的内容；显式发布点 |
| 第一版的操作 | 第一个文件、覆盖写、发布调用、fsync、回退 |
| 调用点 | 崩溃注入与故障注入的调用点上，RefFS 的判定与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官的全部四条判据合取，任一红即红 |

崩溃语义（串行提交、不带故障的崩溃状态）：

1. **fsync**：fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在（串行下就是调用开始之前已取号成功的全部操作，按单元计）；按状态下标读，即允许集合的下界 lo 抬到那次发布的末尾。fsync 报错时不抬 lo，报错那次发布带的操作可在可不在；之后的操作接在下一次恢复读出的状态上判。
2. **丢法：按发布边界的前缀**。崩溃落在第 hi 个操作里时，恢复允许读出的是 {S_j : j = lo，或 j 恰是某次显式发布的末尾；lo ≤ j ≤ hi}，S_j 是上一次恢复之后前 j 个操作施加出来的状态，S_0 是上一次恢复读出的内容。丢掉任意子集的那种丢法在串行提交下出局（与 D16（发布语义） 已定项 12 不相容），开并行提交时重开。
3. **抬 lo**：返回成功的发布调用、fsync、回退都抬 lo（D16（发布语义） 已定项 7：回退与 fsync 同一个返回条件）；回退算一次已确认发布，它的下标算发布末尾。
4. **回退**：回退是 RefFS 的一个操作，回退之后的内容等于整段历史里第 m 次已确认发布之后的内容；m 由调用方从实现报的回退根翻出来交入，RefFS 不判目标选得对不对。m 的编号覆盖每一个写根的入口（发布调用、回退、写行与暖机、抬 F、卸载），映到写它时的现行版本，现行版本含每次恢复读出、重新起头的那一份。
5. **崩溃时刻的归属**：崩溃落在第几个操作里，按「切开当前段的那道屏障由哪一步发出」归属，不按第一个被扣下的写；这条只给 RefFS 用，O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不动。
6. **恢复之后**：每次恢复之后，RefFS 先判读出的状态在允许集合里，再以它为起点接着走。

**射程**：只定做到什么程度与 RefFS 的形态，不定排期；不许读成「先做 RefFS 再做实现」——RefFS 是验证手段不是里程碑（`.claude/rules/format-evolution.md`「从写开始，不从读开始」）。显式发布点只取历史里的发布调用与 fsync：自动触发的攒批落地之前 RefFS 只接显式发布点，自动触发的发布不在 RefFS 射程，归已定项 1 的 M2 与记录核对器；崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步。带故障的崩溃状态（说谎的设备、丢整块盘 C519（丢一整块盘时恢复丢掉刚确认的那一版））不适用。独立性：RefFS 不读录制流、不调切段与状态计划，但它的 (lo, hi) 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官盘级下界那一条判据的入参输入同源，记在 C191（同源性计数器只盖了一对） 那一类同源账上。D17（实现分层与第三方管道） 已定项 4 据此把「入口」的可执行定义交给 RefFS 的操作面，fsync 的持久性承诺由 RefFS 的崩溃语义定义；O3（独立规约执行器） 仍是崩溃点集合的独立执行器，职责不变。RefFS 的第一版在 `crates/singlefs-reference-filesystem`，崩溃注入两截与故障注入重开的调用点已接：harness 档经翻译层 `crates/singlefs-harness/src/reference_filesystem_translation.rs` 依赖它，调用点判 RefFS 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官两票、任一红即红。

**依据**：
- 无实验：做到什么程度、RefFS 怎么来与收什么是验证投入与边界的取舍；三轮判决里攻方的数都是副本装置上量的，入库装置上还没重做（`research/prompts/m3-formal-r5-main-verification.md` 第五节第 3 条），这里不引。
- 用户定案，原话在变更史：「做到极致」（原定案）；做到语义级（2026-10-02，岔路单 `m3-formal-7-forks.md` 里 RefFS 做到哪一档那一行，是对原定案换了读法）；按发布边界的前缀、调用点合取、另起、单独一个包（2026-10-02，`research/prompts/m3-formal-r4-main-verification.md`「用户定案」）；合取收严成全四条、回退目标作输入（2026-10-02，`research/prompts/m3-formal-r5-main-verification.md`「用户定案」）；回退抬 lo、崩溃时刻按屏障归属、m 覆盖每一个写根的入口、白名单加原样重发（2026-10-02，`research/prompts/m3-formal-r6-main-verification.md`「用户定案」）。
- 三方判决：`research/prompts/m3-formal-r4-main-verification.md`（第一轮：fsync、丢法、独立性三格与六条岔路）、`research/prompts/m3-formal-r5-main-verification.md`（第二轮：fsync 合成句、按发布边界补三句、合取收严、另起与单独一个包、任意子集出局限串行）、`research/prompts/m3-formal-r6-main-verification.md`（第三轮最终轮：回退与 lo、射程两句、白名单、fsync 按状态下标读）。
- D17（实现分层与第三方管道） 已定项 4：O1（内存模型对拍（HashMap 理想文件系统））明写「不管崩溃」，定义不了 fsync 的持久性承诺，而第一个可运行目标恰恰是「正确提交一个事务」——做到极致的 RefFS 补的正是这一半。

**欠**：C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C191（同源性计数器只盖了一对）；C658（崩溃注入第三截放过丢掉的一步复活）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:136-151`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-harness/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。崩溃点重放的扇区取 512 字节；两条崩溃点重放流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条崩溃点重放流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；崩溃点重放的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，崩溃点重放小流里记录核对器报出来）在 `m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `e142-r19-runner3-report.md` 结论段，登记 `e142-r19-prereg.md` 的 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:152-168`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/singlefs-ai-sop/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 code-source-discipline 的 format-constants 格按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size` 与 `io_min`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源的类型是 `crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`：声明值那一种（`DeclaredOnly`）已接到 I-7.5（根槽按判定宽度对齐） 的判定上（已实现，`decision_width_declared_by_the_chosen_system_configurations` 取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的），探到那一种（`Probed`）只有类型定义（`crates/singlefs-checker/src/lib.rs`），全仓没有构造它的代码、没有接到任何判定上；I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:169-198`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义**；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后 |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
功能正确性不只押在这三个 oracle 上：O1（内存模型对拍（HashMap 理想文件系统）） 那一行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠「三 oracle 合谋测试」找。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 是只照 kb 文字写的 Python 执行器，现行那一份是 E171（三oracle合谋测试） 第 1 次重跑用的 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170（O3规约执行器第一版） 第 2 次重跑从第 1 次重跑冻结的 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` 拷出 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` 加谓词 W、T，再跟上 487 字节 journal 记录头；第 1 次重跑冻结的那一份与第一次那一份 `scripts/experiment-apparatus/e170_o3_spec_executor/` 留档）：严格跑法判得出（第一次导出的批 1、批 2 上 0 态撞「规约欠一句」），与实现逐态一致（两边全放行，不当它判得对的证据；「择到的根不是自证根里 txg 最大的那条」「③ 收到不少于 2 槽」两条路径批里 0 态，C661（规约执行器批里两条路径没有态）），坏状态表照改正的期望重判有判别力（阳性对照与盲测都判对），还没接进崩溃点重放；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23（journal 的角色与格式） 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根；RefFS（已定项 3）在 `crates/singlefs-reference-filesystem`，崩溃注入与故障注入的调用点已接；「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与现行 `crates/` 不是同一份）：O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS，与表里 O1（内存模型对拍（HashMap 理想文件系统）） 那一行不同（崩溃之后的集合那一半 E171（三oracle合谋测试） 没用 RefFS）；O3（独立规约执行器） 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2（独立解析器 + checker）、O3（独立规约执行器），期望出自 E170（O3规约执行器第一版） 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171（三oracle合谋测试） 只在 Q2 量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；没有量到的不写成「互相独立」被验证过。

**三条 oracle 都不判的两样**（第一样现在有崩溃注入这一侧的判官）：一是无读故障的崩溃态上，D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1）；要有判官问「这个崩溃态没注入读故障，判据凭什么为真」：崩溃注入第二、三截在没注入读故障的崩溃态上复算这条判据，为真即判拒得不合法（`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`，C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 已还清，见欠账表已还清表）。「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由 G4 判（崩溃注入第二、三截，与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条同接法）：mkfs 返回之前的崩溃态不在 G4 的射程，mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后；G4 与那条还有一处不同形：择不到系统配置时那条的复算交 `None`、G4 按真，这时读阶段本身已拒、到不了这一格。二是一次发布内各写的先后：O2（独立解析器 + checker） 的 I-7.12（系统配置 F 不低于同盘根上的 F） 判抬 F 那一串先写系统配置这一步；变异点名的用例只判 E171（三oracle合谋测试） 第 270 行那一种（发布返回之后每个崩溃点的见证值），第 242 行点名的用例判的是失败路径冻结；`layout/01-first-txn.md` 第八节把根槽 FUA 与系统配置槽轮换登记成两段，只是登记在册；其余写序不在三条之内。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够进快档；第 1 次重跑（照 2026-10-03 的 kb 改过一次的执行器）严格跑法判得出、与实现在取样的批态上一致、阳性对照有判别力（第一次跑判不出与判别力不足那两笔留在实验页）。
- G4（无读故障的崩溃态上每块盘至少一份自证过的系统配置）：代码轮判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 j2-b 立、第 29 批实现，射程与 mkfs 那一段见 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L3 与第三节第 3 条。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171（三oracle合谋测试））。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3（独立规约执行器） 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C672（交回值或内存状态错而写流不变的一类）；C679（卸载记号没进模型对拍）

```

**出处 `.claude/kb/decisions/13-验证路线.md:199-217`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（崩溃点重放的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-checker-tier/src/crash.rs`，接在崩溃点重放的每个崩溃状态上（门禁 checker-tier-crash-point-replay）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 崩溃点重放不覆盖可写挂载：崩溃点重放在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在崩溃点重放的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-checker-tier/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 崩溃点重放不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:218-241`（整段抄，未转述）**

```markdown
#### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub

**定案**：现成工具没有、本工程自研的两样：

**1. Merkle 根链差分 —— COW + 每事务发新根独有的杠杆。**
历史根在被回收前物理存在过，构成一条可回放的链。对相邻两代根递归比较「子树指针 + 校验和/MAC」，
**指针没变的地方直接跳过不下钻**，复杂度只正比于两代之间实际改动的部分。
它能检出常规遍历式 checker 检不出的一类错误：**同步篡改后自洽的原地写**——
某次写绕过 COW、原地改了块内容并同步更新了校验和，单点看它是自洽的（I-2.1（校验和与内容匹配） 全绿），
而根链差分会发现矛盾：**没走 COW 就不该有新分配，父指针理应没变，校验和却变了**。
⚠️ **必须配阴性对照**：正常 COW 提交后跑同一检查必须全绿，否则它会把每次正常更新都当成篡改。

**2. 代际增量 scrub。** 靠 I-1.2（块头写序已发布） 与 D5（快照 / 空间记账机制） 的 birth txg
维护一个「上次 scrub 扫到的 generation 水位」，只遍历 `birth generation > 水位` 的块——
老子树的校验和链没变就必然还自洽。**它让「每次提交都全量验证本次改动」在挂钟上可行，不必抽样。**

**射程**：被搬过的副本 `birth` 不变、会永远落在水位之下，代际增量 scrub 看不见它们。`crates/` 里没有根链差分（`grep -rn 根链 crates` 零命中），也没有代际增量 scrub（`grep -rn scrub crates` 零命中）。

**依据**：
- 无实验：两样都靠 COW 每事务发新根与 birth txg 的结构性质成立，都还没有实现，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C389（Merkle 根链差分零代码）；C194（代际增量 scrub 看不见被搬过的副本）；C408（代际增量 scrub 零代码）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:242-275`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 冒烟层 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程），精确全量 | checker 档的全量（已定项 15）：用户要求时跑；提交时默认只跑快档 |
| 常规层 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 大负载层 | 常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 用户要求时 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快档全绿（必要条件）。**
**里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑。**
「全量」指两样：checker-tier-crash-point-replay `--full`（门禁 checker-tier-crash-point-replay，已定项 15 的全量）与崩溃放量全量（`scripts/crash/crash-amplification.sh` 不带参数）。
「动了崩溃点重放或判器」按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上、对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算；阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判。
「上一次全量」由主 agent 写进当前里程碑记录一行：日期、两样各自的结论。
**全量档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个全量档 bug 的挂钟 ≈ `log2(N) × 一趟全量的挂钟`，一个不可复现的全量档失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：只有冒烟层（门禁 checker-tier-crash-point-replay）；冒烟层分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 checker-tier-crash-point-replay `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求时跑；常规层、大负载层在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束，常规层、大负载层 的负载与挂钟都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；崩溃点重放规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。
- 用户定案（2026-10-01）：全量档不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。
- 用户定案（2026-10-01）：全量档全绿的时点是里程碑收尾与动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑，原话在变更史；交用户那一格是三方判决 `research/prompts/test-script-align-r1-main-verification.md` 第二节 t1-3。
- 三方判决 `research/prompts/test-script-align-r2-main-verification.md` 第二节 U3（「U3 t1-3 写回照字面做不了」那一行）：「全量」的两个所指、「动了崩溃点重放或判器」的路径判法、「上一次全量」的记法，「准入条件」与「异步」合成一句。

**欠**：C409（常规层、大负载层在门禁里没有阶段）；C635（上一次全量没有机器可读的标记）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:276-305`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：常规层 精确覆盖先缩问题，再谈剪枝。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立大负载层 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、有几个，归 D17（实现分层与第三方管道） 已定项 2。按种类串认节点够不够，第一轮判决打中（挂起，待第二、三轮攻）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是每一步的失败原子性宽度，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、有几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:306-340`（整段抄，未转述）**

```markdown
#### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条

**定案**：

**「按欠账轮转的账本」调度：否决。** 格子 = `(布局, 子阶段桶)`、按欠账降序取队首、声称任一格子最多 `K_max = ⌈N×M/C⌉` 次提交必被覆盖的那个调度被否决：它保证不了的恰恰是它要防的那一类——慢的、罕见的分支（实测验算与另外四条否决理由在变更史）。

**留下的三条**：

1. **方向对**：要一个**确定性的、能被门禁判红的覆盖记录**，而不是概率论证——
   概率保证在长期维护里等于没有保证。

<!-- doc-lint:registry name-col=2 -->

| 符号 | 简称 | 含义 |
|---|---|---|
| T1 | 常规层 挂钟预算 | 常规层 本地挂钟预算，是输入不是输出 |

2. **`T1`（常规层 挂钟预算） 是输入，不是输出**（`.claude/rules/fs-design.md`：门禁运行时长本身是设计约束）。
   正确顺序是「先声明预算 → 用它约束一次提交能开几个 ignore 节点」，不是「量出耗时 → 推出能开多少」。
   **这一条不需要任何测量，现在就该定。**
3. **判别力必须与覆盖分开检查**：每个 ignore 节点被打开的那次提交里，额外对一份「已知坏的镜像」跑它自己的 checker，
   不报错即判红——这是 D13（验证路线）已有的阳性对照纪律的直接复用。见 C13（checker 判定失效）。

门禁里没有「常规层 崩溃点重放」这个阶段，量不出 `T1`（常规层 挂钟预算） / `C` / 每格耗时。
⚠️ **因此 `C`、`K_max`、`T1`（常规层 挂钟预算） 这三个数一个都不许现在写进 kb。**

**射程**：只管 ignore 的节点什么时候开；冒烟层全量枚举不需要 `T1`（常规层 挂钟预算）。第 2 条的「现在就该定」与末句「一个都不许现在写进 kb」说反话，记在 [verification-build.md](../verification-build.md)「待定案」第 6 问，没判。节点登记表带状态字段（开着 / ignore / 等实现），另加三条——每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比：这三条被攻过零轮（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节），不写成定案，等第二、三轮再攻。

**依据**：
- 无实验：否决账本靠的是一次调度规则的模拟验算（200 轮、取后半程稳态最大值、严格前缀贪心，三行数在变更史），没有登记成实验；留下的三条是门禁设计原则，没有可量的量。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。
- 用户定案：「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 11 推翻，「逐格判决」一节给节点登记表带状态字段的形态与三条待攻的补充条件。

**欠**：C8（门禁范围判不出来）；C13（checker 判定失效）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:341-356`（整段抄，未转述）**

```markdown
#### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具`

**定案**：**冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` 直接打架。**
在用的不变量（条数的权威登记位在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 文件头）若既是 checker 源码又是形式验证的 spec，就成了单点。
**可行的折中，而且本工程已经付过这笔钱**：checker 做两个方向——**遍历式**（从系统配置走下去）
与**扫描式**（全盘扫，靠「元数据块必须自描述」的块头重建），两者对同一条不变量独立判断，
共享的只有格式解析。

**射程**：管 checker 内部两个方向之间共享什么；checker 与实现之间共享什么归已定项 5。池级 checker 两个方向都有（`crates/singlefs-checker/src/walk.rs` 文件头）。

**依据**：
- 无实验：这是「checker 即规范」与校验路径独立之间的取舍，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C410（checker 两个方向共享什么没有检查）：两个方向「共享的只有格式解析」这一条没有会红的检查（`.claude/gate.d/` 里没有判它的阶段）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:357-377`（整段抄，未转述）**

```markdown
#### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model

**定案**：**冲突 2：crash refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法。**
合并成一份 → 省一半工作量，但**直接违反独立性要求**；各写一份 → 名义独立，
但通常是同一个人同一周写的。缓解（只降低相关性、不消除）：让两份**在表述层面就不同**——
spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移」，不一致时至少能互相定位。
**同源性要被观测**：统计「spec 与 model 不一致」的事件数及每次判定哪一方错；
若累计 ≥ 5 次不一致而判定「spec 错」的次数为 0，说明两者实际同源，应把两条腿合并、预算移给穷举。
判定分三格：spec 错 / model 错 / 条款欠一句（两边各照条款的一种读法，条款没写谁对）。一次不一致先在前提成立的域里重判，还在的才记；「条款欠一句」的事件先记账，不计入「累计 ≥ 5 次」，也不计入「spec 错」，条款补了那一句之后按补的句子再判一次、归到前两格。
这里的 model 指 O1（内存模型对拍（HashMap 理想文件系统））。O3（独立规约执行器） 能对同一（历史、崩溃点）交出判定行之前（第一版 Python 执行器 E170（O3规约执行器第一版） 已有，没接进崩溃点重放，C447（规约执行器没接进崩溃点重放与 RefFS 比对）），O3（独立规约执行器） × RefFS 这一对登记为空格、不计事件；O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不当替身；没立的对报「未立」，不报绿、不计入任何一格。

**射程**：缓解只降低相关性、不消除。已定项 1 撤下的那条原候选不受这条影响，理由在已定项 1 的射程。crash refinement 的代码是 `crates/singlefs-replay-prefix/src/lib.rs` 里的 Verus 核与它的 spec（取前缀、择根、扫环三截与择根和取前缀的合判推论，覆盖已定项 1 的 M1、M2 句法层与 M3 的两读和单调推论；M2 语义层与 M4 靠 E165（恢复重放有界穷举与变异杀伤界） 在界内判）；spec 不能对具体状态求值，「spec 与 model 不一致」的事件从哪来没定，计数器没有代码（C191（同源性计数器只盖了一对））。

**依据**：
- 无实验：同源性要靠将来 spec 与 model 的不一致事件计数观测，spec 与计数器都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 第三格「条款欠一句」：三方判决 `research/prompts/m3-formal-r1-main-verification.md` 第二节「F2 同源性」一行（攻方打中，推的）、`research/prompts/m3-formal-r2-main-verification.md` 第二节第 9 行、`research/prompts/m3-formal-r3-main-verification.md` 第二节同源性计数器一行，三轮站住；无实验：计数器仍没有事件来源。
- 「model 指 O1（内存模型对拍（HashMap 理想文件系统））」与没立的对登记为空格：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节同源性计数器那一格（两腿同向的部分）；`research/prompts/m3-formal-r4-main-verification.md`「用户定案」里替身对那一行（不算替身对）。

**欠**：C191（同源性计数器只盖了一对）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:378-396`（整段抄，未转述）**

```markdown
#### 已定项 14：与其他决策的连锁

**定案**：

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 根链差分把 Merkle 从「要付的代价」翻转成验证杠杆；但形式验证里哈希是 uninterpreted function，**碰撞抗性完全在验证范围之外，不许读成「验了 Merkle」** |
| D8（核心索引结构） | 日志结构节点、write buffer、key cache 三层都会绑架符号执行（循环上界、滞后窗口、缓存一致性），使「Yggdrasil 一分钟内」的口径在本工程不成立 |
| D9（加密） | AEAD 在 SMT 里不可解，只能建模成 uninterpreted function，**「nonce 重用泄漏明文」这个后果验不出来**，只剩「nonce 不重复」这条纯组合性质——而这条纯组合性质恰是这条腿最值钱的产出 |
| D11（索引节点要不要留消息缓冲区） | 已定项 1 取「留」；D8（核心索引结构） 已定项 1 取幂等完整值之后 checker 的 I-3.1（已分配统计对得上） 不必解析消息语义（D11（索引节点要不要留消息缓冲区） 据此判它的代价 1 在本工程不成立），冲突 1 不因此变成「无解」 |

**射程**：连锁的是形式验证与 checker 碰到这几条决策时的边界，不改那几条决策的条款；根链差分与冲突 1 本身归已定项 8、12。

**依据**：
- 无实验：连锁是从那几条决策的已定条款推出来的边界，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:397-409`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、崩溃枚举用例经判定库的两段与它的计数、发现日志与工具链身份、一份长期判定库（块键只由内容定，几类测试各自要哪些块只当索引）与两段协议、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 checker-tier-crash-point-replay）分两档，checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 不分：**快档**是两趟：`cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（`--tests` 带上全部 `test = true` 的目标，装置二进制的内联单测也在里面），再一趟带 `tests/` 里 `cfg` 点到的特性（`--features <特性> --lib --test <挂特性的测试文件>…`，特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起）（门禁 checker-tier-crash-point-replay 在整轮门禁与提交时都跑这两趟，第二趟是它的 feature-gated-tests 格），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（checker-tier-crash-point-replay `--full`），提交时不默认跑，由用户要求时跑，跑法按已定项 9 分层；崩溃放量判器整套上了 GPU，罩全部池级 checker 不变量（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`scripts/crash/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。
- 用户定案（2026-10-01）：全量不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:410-425`（整段抄，未转述）**

```markdown
#### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数

**定案**：
- 分片：崩溃放量里按段登记的段（`.claude/gate.d/crash-point-replay-segments.tsv`，一行一段五列：项名、流名、段号、段写数、段写表摘要），整域态数装不进 u64 时按固定大小的片依次判；片大小由环境变量 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 定，没设取 2^32 态；片起点是判定库里的续点（续点按段写表摘要与判器单元键寻址，项名、流名、段号只作显示）。整域装得进 u64 的登记段不分片，整域一次判。代码落点：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- 位宽：登记段的总态数、片起点、片尾与进度一律用 1280 位（10 个 u128）定宽无符号整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`，肢数是同一文件的 `STATE_COUNT_LIMBS`），算术全部 checked，不饱和、不截断；驱动行 `registered_segment_progress=` 的已判态数与总态数都写精确十进制，不写幂的形态。
- 登记表怎么交给引擎：引擎只经环境变量 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 读按段登记表（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`，在 `fn registered_segments_of_the_flow_from_the_environment` 里），没设就当一段都没登，没有别的退路。144 个单元的文件顺序写成 145 个、extent 下段长成两层的那条流（引擎流名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）的单元写段（322 次单元写）与记录段（290 次 journal 记录原地写）两段都登。

**射程**：管崩溃放量引擎对按段登记的段怎么分片、态数用什么整数记、登记表从哪里读；不管哪几段该登（由用户点头，写在登记表文件头）、不管登记段的域本身怎么展开（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10）、不管每个崩溃状态怎么定义（已定项 4）。没登记的段的态数、与整条流合起来的态数仍要装得进 u64，装不下在写库之前拒跑；登记段整域越过 1280 位同样在写库之前拒跑。片大小只许 1 到 2^32：两个 GPU 内核把片内序号当 u32 加到片起点上（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；轻型（`scripts/crash/crash-point-replay.sh --quick`）调用方没设片大小时设成 `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`。展开上限盖不住又没登记的段，放量入口 `scripts/crash/crash-amplification.sh` 判红；崩溃点重放入口、放量入口与分项驱动开跑前核这份表再送给引擎。

**依据**：
- E161（崩溃放量的去重与分段耗时）「第 2 次重跑：取样域按 11 段的流重定（2026-10-05）」一节：段的全量态数与闭式、装置逐个数、crates 计划逐段相等，态数随段写数按幂长，撑的只是「长段整域装不进 u64」这个前提；片大小、位宽取值与登记表的读法是工程参数，用户定，它不撑。
- 用户定案，原话在变更史：分片与每片 2^32 态（`records/2026-10-11-崩溃放量两段与脚本审计.md` 第 96 行）、位宽 1280 位（同一份记录第 98、99 行）、两段都登（同一份记录第 92 行）。
- 分片与位宽是对判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10（按段登记）的补充，没经三方攻过。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:426-446`（整段抄，未转述）**

```markdown
#### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成

**定案**：
- 重型门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、快档与模型对拍的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加闭包里各单元（含 mod 头）的 `#[cfg]`、`#[cfg_attr]`、`cfg!` 真引到的 cfg 原子、各带这一次目标下的真值（闭包没引到的原子不进键：带不带 `gpu` 特性编的两趟，闭包不提 `gpu` 的判定键相同、共用一份库），加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。
- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。

**射程**：管重型门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与什么时候请用户跑全量（已定项 9）。镜像判定与给定状态判定的版本是判定定义那一组（CPU 参照判器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` 的闭包）的键，不论这一块在 CPU 还是 GPU 上判的：GPU 判器是加速路径，它的准入靠与定义那一组的对拍（快档的 GPU 与 CPU 对拍用例），它的单元只进块属性当出处，不进键、不分种类；显卡驱动、卡型、几张卡怎么分活同样留作块的属性或跑的参数，不进键。

**依据**：
- 用户定案（2026-10-15），原话在变更史与 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 用户定：提交与重跑无关，重跑只跟代码变化走」一节、「2026-10-15 批3」那几段。
- 用户定案（2026-10-17）：判定不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备；原话在变更史与 `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 叫停全量那几段。
- 主 agent 定案（2026-10-15）：仓根不进键与库、库目录只收绝对路径、cfg 成分只取闭包真引到的原子，出处同一份记录「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案第 1、2 条。
- 反证：E164（崩溃放量新设计的小范围功能测试） F4 量到只按函数取指纹、不连引用的常量一起收，漏判 V5 与 V11（V11 只改常量 `DATA_POINTER_BYTES`）：函数单元要连同引用的常量、类型、静态项与宏一起算。
- 盘上的数：`research/prompts/pruning-survey-2026-10-15/ledger.md`「一之二、盘上量到的」一节——只差名字的块各存各判 4100 块与 9679 块；整库一把的判法摘要一换累计多判 3 935 242 934 态，那次闭包里改动的 29 份文件有 24 份去注释后逐字相同：键带名字、按文件或整版取指纹，同一份判定会重判。
- 没有腿况记录：未经三方攻过；用户定改完代码开三轮攻方，只攻「剪枝是不是原子化的、键是不是只由函数单元组成」。

**欠**：C8（门禁范围判不出来）、C489（singlefs-core 的广谱变异对今天的代码没有结论）、C528（两条门禁路判的改动窗口不同）、C564（另外 6 条崩溃点重放全量没人跑）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C630（提交时形式化那一道第 1 步停下的频率没量）、C635（上一次全量没有机器可读的标记）、C645（崩溃放量全量没有人能派）、C668（随机历史脚手架的引入者白名单能被别名写法绕过）、C777（五个实验键在 stage-inputs.tsv 没有登记行）、C836（staged-green 挂的树不一定是被判的树）。

```

**出处 `.claude/kb/decisions/18-information-carried-in-a-block.en.md:1-6`（整段抄，未转述）**

```markdown
## D18 (Information Carried in a Block) — Settled

[中文](18-块里携带什么信息.md) · **English** · [日本語](18-ブロックに含める情報.ja.md)

D18 (Information Carried in a Block) governs what information is carried inside a unit (block): criteria for whether a field is worth carrying into a block (settled item 19), fields settled to be carried and rejected fields (settled item 20, settled item 23), field tables and offsets for each class of unit headers, the value range of unit type tags and the registry for packed record types (including the instance table), the header checksum algorithm and reserved bits for nonce / MAC / algorithm type, and what level of scan rebuild can achieve relying solely on block headers. Out of scope: "Is this block still alive?" is a **question about sets**, which a block can never answer—this is not a defect in format design, but rather D5 (Snapshots / Space Accounting Mechanism) has settled that it is a global attribute, and that is precisely the origin of O(1); the field composition of AAD belongs to D9 (Encryption) settled item 6. Interlocks with other decisions: data unit self-describing is directly related to the leakage surface trade-off in D9 (Encryption) settled item 5 (settled item 3, settled item 13); intra-node parity partially overlaps with the variable-width full stripes of D2 (RAID Stripe Policy) across multiple disks, but **a single disk is where D2 (RAID Stripe Policy) cannot reach** (settled item 4); if carried information falls on **each record** rather than on block headers, the cost multiplies by record count (D8 (Core Index Structure), settled item 19); inlining "last verification time" would turn **scrub from a read-only into a write operation**, creating new blocks on each sweep, condemning old blocks to death and feeding them into deadlists, **working in direct opposition to the goal of resident compaction in D3 (Space Allocation)** (settled item 19 criterion 1).

```

**出处 `.claude/kb/decisions/18-information-carried-in-a-block.en.md:7-609`（整段抄，未转述）**

```markdown
### Settled Items

**D18 (Information Carried in a Block) has only one set of item numbers**, consecutively numbered with the "Unsettled Items" section, without reordering.

| # | Item | Settlement |
|---|---|---|
| 1 | **Lineage rewrite sequence number** | Not implemented: what it aims to purchase ("distinguishing stale replicas from active replicas") has already been purchased by the per-placement predicate of D3 (Space Allocation) settled item 1 **Status: Settled.** **Progress: Implemented.** |
| 2 | **Nodes self-carrying key ranges** | All code 2 nodes unconditionally carry key ranges, not split by tree, without toggle **Status: Settled.** **Progress: Implemented.** |
| 3 | **Data block self-description** | Plaintext, and preferably sized generously: each data unit carries a plaintext self-describing header, five-tuple totaling 33 bytes; no `locality_id` added **Status: Settled.** **Progress: Implemented.** |
| 4 | **Form of single-disk self-recovery** | Takes metadata multi-copy + dispersed placement (ditto), chain arm rejected; single-device layout line not opened yet; placement offset forbidden to take powers of 2 **Status: Settled.** **Progress: Unimplemented.** |
| 5 | **How rebuild path is compatible with A6 (AAD expected values do not come from plaintext side)** | Rebuild is a degraded path, graded into three levels by available witnesses; system configuration `map_provenance` reserved + read-only mount + in-place repair forbidden **Status: Settled.** **Progress: Unimplemented.** |
| 6 | **Position of data block self-describing header** | Header placed at low offset, payload begins after header; header attached at block level, not unit level **Status: Settled.** **Progress: Implemented.** |
| 7 | **Whether block headers need a sixth plaintext field** | Unit header field tables partitioned by unit class: common plaintext prefix 42 + class identity segment, reserved nonce / MAC bits; self-verifying units do not use this table **Status: Settled.** **Progress: Implemented.** |
| 8 | **Whether deletion writes tombstone units** | Yes: deletions leave tombstone records on disk, container takes code 3, records take packed record type 1, deceased identified by record's own fields **Status: Settled.** **Progress: Implemented.** |
| 9 | **What "block" means in "block-level"** | Block = unit, one header per unit; scanner steps by minimum unit size in pool (v1 is 16384); unit origins aligned by class **Status: Settled.** **Progress: Implemented.** |
| 10 | **Granularity and carrier of tombstones** | Range records + packed shared units; loaded by generation, no inter-generation mixing; v1 40 bytes, unreclaimed (settled item 24) **Status: Settled.** **Progress: Implemented.** |
| 11 | **Value range of unit type tags, and which class packed record units belong to** | Value range is a registry; packed record units are an independent class (code 3, header 107); packed record types form secondary registry: 1 tombstone, 2 inode, 3 retirement, 4 instance table, 5 lineage **Status: Settled.** **Progress: Unimplemented.** |
| 12 | **Whether code 2 write sequence should be narrowed to 4 bytes** | Narrowed: code 2 write sequence retains only instance code 4, vacated 6 bytes allocated to payload CRC 4 + reserved 2; code 3 retains those 6 bytes unchanged **Status: Settled.** **Progress: Implemented.** |
| 13 | **Whether code 1 identity segment and `declared length` go to plaintext or ciphertext when encryption enabled** | Takes arm G: code 1 identity segment and declared length become ciphertext in-place, position and width unchanged; code 1 plaintext allowlist narrowed to {fsid, payload CRC} **Status: Settled.** **Progress: Unimplemented.** |
| 14 | **How rebuild path acquires complete 96-bit nonce** | Takes arm A: nonce code is the full 96-bit nonce (12 bytes), residing in unit plaintext header, pointer copy retained **Status: Settled.** **Progress: Implemented.** |
| 15 | **Withdrawn · Number no longer used** | Withdrawn (original topic: version selection rules for code 2): three rounds across three parties found no wording that withstood challenge; code 2 as before excluded from version selection rules **Status: Settled.** **Progress: No code object.** |
| 16 | **Switch pass-through for nonce / MAC / algorithm type reserved bits: code 1 placement, code 2 / code 3 header width with reserved bits, declared length origin** | 29-byte reserved bits immediately following class identity segment counted into header: code 1 header 134, code 3 header 136, code 2 header 115 + 2 × key width; declared length origin is end of header including reserved bits **Status: Settled.** **Progress: Implemented.** |
| 17 | **Algorithm for 32-byte header checksum; coverage of self-verifying structures** | 32-byte field stores CRC32C 4 bytes + 28 bytes zero; checksum of self-verifying structures covers entire slot including padding, padding always 0 **Status: Settled.** **Progress: Implemented.** |
| 18 | **Code 2 declared length semantics, padding area always 0, offset tables for code 1 / code 2** | Code 2 declared length = entry count × entry width; padding area following entry area always 0; code 1 and code 2 each supplemented with field table with offsets **Status: Settled.** **Progress: Implemented.** |
| 19 | **Whether a field is worth carrying into a block: three criteria** | Three superimposed criteria, none dispensable: update cycle bound to content; absence turns judgment from decidable to undecidable; net gain relative to existing mechanisms **Status: Settled.** **Progress: No code object.** |
| 20 | **Settled identity fields to carry** | Own logical address + fsid, generation (birth generation), owning tree ID (birth tree)—all three must be carried **Status: Settled.** **Progress: Implemented.** |
| 21 | **Scope of self-description: misdirected writes, not lost writes** | Lost writes cannot be caught by anything inside the block, requiring witnesses outside the block: expected generation and checksum in parent pointer, recurring to root **Status: Settled.** **Progress: No code object.** |
| 22 | **Explicit fields vs folding into checksum seeds** | Information required to support scan rebuild must be written explicitly; only what needs misapplication detection without rebuild may be folded into seeds **Status: Settled.** **Progress: No code object.** |
| 23 | **Rejection list** | Not carried: child blocks recording current parent node address, checkpoint inlined block count, COW predecessor address, intra-block error correction code, per-block wall-clock time **Status: Settled.** **Progress: Implemented.** |
| 24 | **V1 tombstones: tombstone tree, 40-byte records, lineage records** | V1 tombstones: dedicated tombstone tree, 40-byte whole-object records with death txg, rollback writes lineage records **Status: Settled.** **Progress: Implemented.** |

#### Settled Item 1: Lineage Rewrite Sequence Number Not Adopted

**Settlement**: Lineage rewrite sequence numbers (monotonically increasing along lineages, orthogonal to transaction numbers) are not implemented. **What it aims to purchase has already been purchased by something else**: the purpose of a lineage rewrite sequence number is "distinguishing stale replicas from active replicas", whereas the per-placement predicate of D3 (Space Allocation) settled item 1 is precisely the authoritative recording point for that capability ⇒ adding another sequence number purchases nothing new, only adding a permanent format field.

**Scope**: What is rejected is "a sequence number established to distinguish stale replicas". If another use arises in the future (for instance, if the question of how to coexist with the checkpoint sequence numbers settled in D16 (Publish Semantics) genuinely stands independently), it can be reopened when needed, **but whoever reopens it must first demonstrate that it does not buy what has already been bought**. The unit header's write sequence (instance code, transaction number) (settled item 7) is not within the scope of rejection: it purchases "whether this unit has been published, and which timeline it belongs to"; ordering of published versions of the same key relies sufficiently on birth generation + one net effect per transaction per key; write sequence only incidentally provides ordering when modifying the same key across transactions in the same txg. Settled item 11 once planned to use birth generation to select the newest among multi-version packed containers of the same identity, which is precisely the use rejected here; having shifted to judgment via the predicate of D3 (Space Allocation) settled item 1, settled item 1 will not reopen.

**Basis**:

- E59 (Can buffered messages be recomputed solely from units): When scanning only units, multiple readable replicas of the same key cannot distinguish the active value; after adding the per-placement predicate, divergence across all four categories of messages falls to zero—a sequence number only purchases the overwrite slot; adding another purchases nothing new.
- User settlement, recorded in change history; the scope note on write sequence was added following the settlement of C113 (no inputs for active version judgment of multi-version units during scan rebuild).

**Owed checks**: C288 (collateral changes from mapped key settlement did not land in other decisions) item 6: add note in scope that "birth sequence number only serves as one segment of key, not used to select newest".

#### Settled Item 2: All Code 2 Nodes Unconditionally Carry Key Ranges

**Settlement**: **Every code 2 index node carries `[min_key, max_key]`, not split by tree, without toggle.** The code 2 header width formula `86 + 2 × key width` (settled item 18) therefore holds unconditionally for all trees; decoding a code 2 node during scanning does not require first checking "whether this tree carries it". What it purchases is: a node picked up in isolation can self-prove which range of keys it ought to cover, aligning with `.claude/rules/fs-design.md` "do not sacrifice self-containment to save space". **Range takes the subtree coverage range**: `[min_key, max_key]` is the minimum and maximum key reachable within this node's subtree—on leaf-level code 2 nodes it is the keys of the first and last entries; on internal nodes whose children are code 3 containers, max_key is the key of the last record in that final container, not the separator key of the last entry. Modifies bytes of new-pool file creation: No—the ranges were already present in the bytes written out by the six code 2 trees; both interpretations yield the same value for new-pool file creation (inode tree root [1, 1]).

**Scope**: What is extrapolated is the judgment "whether to carry", not the cost numbers of E73 (Fanout cost of node key range)—how many slots fanout drops for each tree still requires individual measurement. **The dirent tree slot is closed accordingly**: when a dirent tree exists it still carries key ranges; what needs remeasuring then is fanout cost, not this judgment. Key ranges reside in block headers (D15 (Format Freeze Policy) tier 1), sharing the same slot as accounting tree node headers (C102 (no reserved area in index node headers; adding fields is not compat)). Checker's `check_index_node_keys` (`crates/singlefs-checker/src/lib.rs`) checks the keys of the first and last entries; the root of parallel line 3 wrote out [2, 2] while subtree coverage was [2, 234]; both places must be modified per subtree coverage range (C478 (whether key range in code 2 header is entry key range or subtree coverage range)). For position-addressed trees (allocation record tree, extent tree, D8 (Core Index Structure) settled item 14), the key range in the node header writes the span governed by position regulations for that node, which is identical to the "subtree coverage range" on such trees; checker independently computes the span each node ought to cover based on position and verifies node by node.

**Basis**:

- E142 (Dry-run of new-pool file creation): All six code 2 trees written out in run 6 carry key ranges; without them, no tree header width can be calculated; the per-tree carrying approach would require adding a "carry or not" flag bit in the code 2 header, whereas the scan rebuild path has no tree table, creating a cycle where one must know header length before reading the flag.
- E73 (Fanout cost of node key range): Carrying ranges drops fanout by at most 1 slot without increasing tree height; 16 KiB in D8 (Core Index Structure) settled item 2 does not need reopening; isolated node coverage range judgment cannot be resolved by the arm without ranges.
- User settlement (modal prompt, did not go through three-party defense, overturnable; main agent recommended this tier too), verbatim text in change history.
- User settlement 2026-09-23: Range takes subtree coverage range, verbatim text in change history.

**Owed checks**: C478 (whether key range in code 2 header is entry key range or subtree coverage range): checker and write path updated per subtree coverage range, accompanied by a bad image where range is written as entry key range.

#### Settled Item 3: Data Blocks Carry Plaintext Self-Describing Header

**Settlement**: Each data unit carries a plaintext self-describing header, taking the fields settled in D9 (Encryption) settled item 6: `(unit type tag, tree ID, object ID, object birth generation, anchor offset)`. **Preferably sized generously, not trimming fields to save space.** Width of the five fields: **unit type tag 1 + tree ID 8 + object ID 8 + object birth generation 8 + anchor offset 8 = 33** (question 8 of [verification-build.md](../verification-build.md) closed hereby; basis traced back in settled item 11).

- **Boundary with D1 (Data Mobility / Reverse Index)**: **Criteria reside in D1 (Data Mobility / Reverse Index)** (what enters self-contained segments, what enters extension points; see its settled items 1 / 3 / 4); **field tables reside here** (which fields actually entered, how wide each is, how partitioned by unit class). If criteria change, this table changes accordingly; table formatting does not reverse criteria.
- **Do not add sixth field `locality_id`**: `locality_id` is a hint (D8 (Core Index Structure) settled item 3); during rebuild, objects are identified by (tree ID, inode), inspecting only the extent tree of the current header, assigning the same initial segment to all extent keys (surviving and recovered) for this object, taking value inode record > surviving extent key > 0; incorrect assignment merely loses locality; rebuild artifacts are inherently read-only (settled item 5). Rebuild rules text resides in D8 (Core Index Structure) settled item 6 "Rebuild" paragraph.
- **Two lines that must be held**:

| Forbidden to Place | Why |
|---|---|
| **Physical placement / device ID** | D9 (Encryption) settled item 6 settled that "physical locations never enter AAD" ⇒ placing it in block headers creates an **unauthenticated, silently tamperable** field. Authoritative recording points for physical addresses remain in encrypted structures per D9 (Encryption) settled item 5 (scrub only); the non-key-holding side cannot read it anyway |
| **File name / separator key / any path info** | D9 (Encryption) settled item 5 settlement states verbatim that its **entire** difference from the ZFS path is scope—"not exposing separator keys of btree internal nodes", whereas the separator key of the dirent tree is the file name. **Failing to hold this equals taking the ZFS path** |

- **Why physical pointers in extension points do not hit these two lines: reason is "cannot create silent misreads", not "it is authenticated"**: These two lines guard against **silent misreads**—a forged physical placement in a block header would make a reader read the wrong block while believing it read correctly. But pointers in extension points **cannot enter any decision path** (D1 (Data Mobility / Reverse Index) settled item 5 verbatim: it does not constitute a reference); every dereference must pass target birth generation comparison against target unit's own header checksum ⇒ forging it at most makes a clue point into empty space, **incapable of creating "reading wrong while believing read right"**. This belongs to the same category already knowingly accepted in D9 (Encryption) settled item 5 (plaintext side may be reordered, generation may be forged), introducing no new category. **Must not cite "extension point is within checksum / authentication coverage of whole unit" as reason**; reason remains "cannot create silent misreads", which does not depend on whether MAC covers extension point; switching back to "extension point is authenticated" requires renewed defense, which has not been done.

**Scope**: What it purchases is "rebuildable when index is lost", **not "tamper-evident when attacker modifies"**—the latter relies on AAD mismatch and the Merkle tree of D4 (Checksum Position). **Plaintext readability of block headers does not constitute authentication**: during normal reads (indices present), plaintext headers do not participate; AAD expected values come from lookup path per A6 (AAD expected values do not come from plaintext side); rebuild (indices lost) must participate; individually modifying fields causes MAC mismatch, but whole-unit relocation (header + ciphertext moved together) passes verification completely, which is precisely the "block-level reordering" A6 (AAD expected values do not come from plaintext side) guards against—this slot is handled per degraded semantics of settled item 5. The two lines govern self-contained immutable data, not extension points: **physical placement is mutable** (blocks get moved, and the raison d'être of D1 (Data Mobility / Reverse Index) is precisely mobility); by "changes across versions or not", it inherently cannot enter self-contained segments; extension points are for mutable data (D1 (Data Mobility / Reverse Index) settled item 3: either empty or a pointer). When encryption is enabled, this tuple in the block header is its **sole** plaintext expression on disk (D9 (Encryption) settled item 5 establishes no plaintext mapping layer and no plaintext reverse index); anchor offset directly reveals file size and file holes; this leakage is not accepted, handled under settled item 13; settled item 3 itself remains unchanged—data units still carry self-describing headers, field set and widths untouched; settled item 13 merely changes "whether these fields are plaintext or ciphertext on disk when encryption enabled"; primary basis (existing for fast retrieval, not sacrificing self-containment for space) does not depend on the leakage surface slot. This group is simultaneously the AAD field set of D9 (Encryption) settled item 6, and the C72 (relationship between extent and unit unanswered) round established that "AAD field composition is a day-1 permanent contract" ⇒ modifying it is a permanent contract alteration. Three slots derived from settled item 3 form formal items: settled item 5, settled item 6, settled item 7.

**Basis**:

- E28 (Can plaintext mapping layer be rebuilt solely from unit self-description): When removing plaintext mapping layer and scanning only unit headers for rebuild, metadata side has zero missing and zero incorrect entries; data block side deficit exactly equals total data block count; positive control adding self-description to data blocks drops deficit to zero—this is the hole it purchases.
- User settlement: These fields exist for **fast retrieval**, the faster the better; general stance is **do not sacrifice self-containment to save space** (`.claude/rules/fs-design.md`), verbatim text in change history.
- Scope of the two lines: User settlement, verbatim text in change history.
- "Plaintext readability of block headers does not constitute authentication": Reverse-inference leg of three-party defense pointed this out; main agent confirmed verbatim check against D9 (Encryption) settled item 6 criterion A6 (AAD expected values do not come from plaintext side), history recorded in change history.
- No `locality_id` added: User settlement following three rounds across three parties (`research/prompts/d18-locality-r1-main-verification.md`, `d18-locality-r2-main-verification.md`, `d18-locality-r3-main-verification.md`); defense of rejected forms in the three verdicts.

**Owed checks**: C346 (background reclustering undefined); C68 (invariant allowlist conflicts with settled clauses): gate comparison tooling.

#### Settled Item 4: Form Takes Metadata Multi-Copy + Dispersed Placement, Chain Arm Rejected, Single-Disk Layout Line Not Opened Yet

**Settlement**: Form of single-disk self-recovery takes "metadata multi-copy + dispersed placement" (ditto); chain arm formally rejected; but **single-device layout line is not opened yet**. ditto takes ZFS's real form: user data 1 copy, dataset metadata 2 copies, whole-pool metadata 3 copies, copies placed dispersedly. **Mandatory clause when adopting ditto**: placement offset **must not take powers of 2**—resonating with failures of any power-of-2 periodicity causes all three copies to share congruences and perish together. Modifies bytes of new-pool file creation: **No** (ditto is copy count and placement policy, modifying no field tables; single-disk layout line unopened ⇒ bytes written by new-pool file creation remain unchanged).

**Scope**: **Settled item 3 does not diminish settled item 4**: data block self-description guards against "**index lost**"; settled item 4 guards against "**block itself lost**"—self-description cannot read a block no longer on disk; the two have different scopes. Reason for not opening single-disk line: single-disk line is an **entirely different layout**, not "a 2-disk line with one disk pulled"—D2 (RAID Stripe Policy) settled item 6 `w ≥ 2` is mutually exclusive with single device (D2 (RAID Stripe Policy) settled item 9); D12 (Target Media) also requires an incompat bit for it, making it a permanent format branch; per branch criteria in `.claude/rules/fs-design.md`, it must be justified independently; opening the line requires standalone crash-point replay verification, and requires test-only switches to force entry. Multi-disk and single-disk are not equivalent: `w ≥ 2` and zero-redundancy stripes forbidden to issue ⇒ every stripe in a ≥2 disk pool naturally carries parity, losing 1 column can be rebuilt from remaining columns (the claim that "one unit landing on one column in its entirety (D2 (RAID Stripe Policy) settled item 10) ⇒ losing one disk equals losing the entire unit ⇒ multi-disk equals single-disk" **does not hold**); **single disk lacks this mechanism**. If chain takes version chain (prev/next = COW predecessor and successor of this unit), it hits two rejection list items (settled item 23: "child block recording current parent node physical address" judged "temporally impossible", "COW predecessor address (version chain)" judged "negative net value"), leaving only same-batch chain for comparison.

**Basis**:

- E118 (Three arms of single-disk self-recovery): Pre-run hardcoded failure clauses triggered literally—recovered count for same-batch chain on **all** failure points equaled zero-redundancy arm; what it purchases is **detection**, and detection can already be answered by allocation records of D3 (Space Allocation) settled item 3 ⇒ net gain ≈ 0; ditto is the only arm among the three that purchases recovery capability; the offset of "at least 1/8 disk away" in the model was a power of 2, resonating with power-of-2 periodic faults. The six evidentiary burdens prior to line opening are discharged at once by this.
- Six line-opening evidentiary burdens, red-verdict conditions, and user settlements resulting from three-party defense are all in change history.

**Owed checks**: C201 (ditto placement offset forbidden to take powers of 2).

#### Settled Item 5: Rebuild Is a Degraded Path, Graded into Three Levels by Available Witnesses

**Settlement**: Acknowledging rebuild as a degraded path, but the form is not a mere statement, but graded by available witnesses:

| Level | What Witness Obtained | Disposition |
|---|---|---|
| 2 | Allocation record (D3 (Space Allocation) settled item 3) + checksum / MAC dual-witness | Writable, normal promotion; ⚠️ Unreachable—allocation records are derived state (D21 (Boundary Between Authoritative and Derived States) settled item 5); derived state cannot serve as a witness; level 2 witness does not exist |
| 1 | Only checksum / MAC self-consistent, no allocation record | **Read-only mount** |
| 0 | Cannot even verify checksum | Report only, do not enter mapping |

**Grading criteria are computable, and inputs remain constant during operation**, satisfying `.claude/rules/fs-design.md` hard requirements 1 and 5 for branching. **All three must be implemented; lacking any one leaves it as a mere reminder** (`.claude/singlefs-ai-sop/rules/show-me-test.md` verbatim prohibits writing past pitfalls as reminder sentences):

| # | Realized as What | Why It Must Take This Form |
|---|---|---|
| 1 | A `map_provenance` (rebuild epoch + authoritative state digest relied on during rebuild) in system configuration, **reserved day-1** | If not reserved day-1, "can be added later" would quietly expire the day D15 (Format Freeze Policy) freezes. It must fall on authenticated fields (D9 (Encryption) settled item 8 settled that system configuration must be authenticated due to nonce watermark) |
| 2 | `map_provenance = rebuilt` and unpromoted ⇒ **read-only mount** | The `compat_ro` form of `.claude/rules/fs-design.md` "turning misuse into unmountable": mountable, unwritable, explicit failure mode |
| 3 | One discipline: under this state, **no authoritative state structure may be repaired in-place by rebuild results** | It extends I-6.10 (decommissioned: plaintext mapping layer no longer exists) verbatim "any discrepancy is judged red, **in-place repair forbidden**" to rebuild artifacts, not a new mechanism |

Three format-level requirements for `map_provenance`:

| # | Requirement | Why |
|---|---|---|
| 1 | Resides in system configuration, **and must not reside in a position where it can be rewritten without trace** | A gate that can be bypassed by modifying a single byte is equivalent to having no gate at all. Its protection strength varies with encryption toggle (Scope) |
| 2 | **Promotion from Level 1 → Level 2 must leave traces**, in-place modification of flag bit back to normal forbidden | Promotion is "legitimately clearing rebuilt"—**it is itself the shape of a bypass path**. Trace content: based on which batch of dual-witnesses, when promoted; that record itself must be recheckable by checker |
| 3 | Subsequent normal write paths **must not overwrite it** | Amnesia shape: declare once, never inspected again, subsequent mounts all display normal. **Good-faith operation will also lead there** |

**The half-sentence "requiring human confirmation" is not adopted**: administrators hold no original index or reference data to compare against, **unable to judge whether files have been moved** ⇒ that requirement in practice forces humans to trust rebuild processes without verifiable data, **which directly violates the purpose of authentication**. Promotion criteria can only be computable dual-witnesses like Level 2, not a human nod.

**Scope**: **Write truthfully what v1 can and cannot prevent**: D9 (Encryption) settled item 8 requires system configuration to be authenticated via **master-key-derived MAC**, while D9 (Encryption) settled item 10 settles that encryption does not enter the first runnable release ⇒ **v1 lacks a master key, so that authentication does not exist** ⇒ `map_provenance` in v1 is protected only by non-keyed checksums, **guarding against accidental corruption, not deliberate tampering**; this matches D9 (Encryption) settled item 5's stance, but **must be written down**, forbidding interpreting "has this field" as "this degraded state cannot be erased". **A6 (AAD expected values do not come from plaintext side) does not govern a single byte in v1**: what actually changes bytes in v1 is **only the reservation of `map_provenance`** (system configuration "mapping provenance (rebuild epoch + authoritative state digest)" 24 bytes, all zeros, [layout/01-first-txn.md](../layout/01-first-txn.md)); the rest is how rebuild code is written, whereas `crates/` contains zero scan rebuild (`grep -rn "扫描重建" crates --include=*.rs` yields zero hits), and v1 checker (`crates/singlefs-checker`) does not enable encryption and does not check A6 (AAD expected values do not come from plaintext side). **Level 2 witness is gone** (allocation records are derived state; C73 (state classification of allocation records unsettled) verbatim: "derived state must itself be rebuilt, cannot serve as witness"): Level 2 must either find another dual-witness residing in authoritative state, or this slot reopens; **Level 1 and Level 0 remain usable as normal**, with only the promotion step affected.

**Basis**:

- No experiments: Grading defines mount states for rebuild artifacts based on obtainable witnesses, definitional, no measurable quantities.
- Three-party defense: Two legs independently confirmed candidate 1 (relying on parent pointers) is cyclical under the premise "indices must be lost to warrant rebuild"—the parent pointers it needs reside in indices, which are precisely the trees being rebuilt; authoritative side per D21 (Boundary Between Authoritative and Derived States) = units + accounting + root, excluding indices; local leg pointed out "requiring human confirmation" cannot be judged. User settlement chose candidate 2, verbatim text in change history.
- Level 2 witness non-existent: User settlement in D21 (Boundary Between Authoritative and Derived States) settled item 5 (allocation records are derived state).

**Owed checks**: C75 (degraded mount gate itself never turns red).

#### Settled Item 6: Data Block Self-Describing Header Placed at Low Offset, Payload Starts After Header

**Settlement**: Each block's **byte 0 onwards is the self-describing header**, immediately followed by payload. **The half-question "whether header attaches at block level or unit (extent) level" is settled as block-level**: placing at low offset means the scanner hits it by stepping per block, whereas unit-level requires scanner to know unit boundaries beforehand—**which is precisely what it needs to rebuild** (cyclical).

**Scope**: Once block = unit (settled item 9) and extents do not span multiple units (D4 (Checksum Position) settled item 7), "block-level" and "unit-level" coincide under this value; the amortized cost N for "a large extent spanning N blocks paying N copies of headers" originally owed by block-level is constantly 1, removing the target.

**Basis**:

- No experiments: Criterion is "whole-disk scanning during recovery must find these headers **without indices**" ⇒ headers must fall at positions the scanner can **predict**, and block start is the sole position computable without any prior knowledge.
- User settlement, recorded in change history.

**Owed checks**: None.

#### Settled Item 7: V1 Unit Header Field Tables Partitioned by Unit Class

**Settlement**: V1 unit header field tables are partitioned by unit class—common plaintext prefix + class identity segment; **fsid and birth generation placed into header, header checksum takes 32 bytes**; v1 reserves field bits for nonce code and MAC; self-verifying units (journal records / root slots / system configuration slots) do not use this table.

**Common plaintext prefix (data units and index nodes, 42 bytes initial value; width follows E85 (Field table for unit headers) requirement matrix measurement basis, adjusted when writing code)**:

| Field | Width | Origin |
|---|---|---|
| magic | 4 | Scan probe (E86 (Scan step and two unit sizes): probes stepping 16384 falling inside data payloads rejected by this + header checksum) |
| format version | 2 | D21 (Boundary Between Authoritative and Derived States): scan rebuild must decode all historical unit formats |
| flags | 2 | **Offset 6 = unit type tag (code from settled item 11 registry), offset 7 = 8 flag bits, v1 always 0, any non-zero rejected (forward compatibility policy)**—after encryption is enabled metadata identity enters ciphertext; scanner must distinguish class before decryption to know which allowlist applies |
| declared length | 2 | D4 (Checksum Position) settled item 6: I-2.3 (padding bytes zero and participate in checksum) must be decidable during scan without indices. ⚠️ **Ciphertext when encryption enabled** (settled item 13, arm G)—position and width unchanged, merely no longer in plaintext allowlist |
| header checksum | 32 | Sole line of defense for header integrity during scan rebuild without parent pointers (after encryption enabled, covers only plaintext portion for metadata classes; class identity segment integrity shifted to MAC); **coverage settled by settled item 11: offset 0 to end of plaintext header for that class** (I-2.4 (header checksum coverage)); width shares measurement basis with E23 (journal geometry) `header_csum` and D22 (How Unit Atomicity is Synthesized) settled item 7 root record self-verifying checksum. **Algorithm settled by settled item 17**: CRC32C 4 bytes + 28 bytes zero |

**Class identity segment — Data unit (+63 bytes ⇒ 105 bytes initial value, occupying 0.32% of 32 KiB unit)**:
<!-- format-const: DATA_UNIT_HEADER_BYTES = 105 -->

| Field | Width | Origin |
|---|---|---|
| five-tuple | 33 | Settled in D18 (Information Carried in a Block) settled item 3 (plaintext, preferably sized generously); unit type tag is redundant with flags class tag bit, accepted (`.claude/rules/fs-design.md` "do not sacrifice self-containment to save space") |
| birth generation (birth txg) | 8 | Three independent reasons, see Basis |
| fsid | 8 | Two independent reasons, see Basis |
| **write sequence (instance code 4 + transaction number low 48 bits)** | **10** | C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P1: (instance code, transaction number) of the transaction that wrote it; three purposes share one field—ordering published versions of same key, distinguishing pre- and post-crash timelines, separating applied from abandoned during partial replay of in-flight checkpoints; transaction number is low 48 bits of the 8-byte field settled in D23 (Journal Role and Format) settled item 7, introducing no new field |
| **payload CRC** | **4** | C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P3: replica merging requires "multiple readable units of same class identity segment have identical payload", code 1 has no other field proving this ⇒ payload CRC of C104 (no self-contained evidence of payload integrity for code 1 / 2 on scan path) is prerequisite for code 1 replica merge / level 1 recovery; added only to code 1's own class identity segment, not entering common prefix |

- **When encryption is enabled, only fsid and payload CRC remain in plaintext in this class identity segment table** (settled item 13): five-tuple / birth generation / write sequence become ciphertext in-place; position and width untouched; plaintext header total width remains 105. When encryption is disabled, the entire table is plaintext, and bytes written by new-pool file creation remain byte-for-byte unchanged.
- **Payload CRC covers on-disk bytes, not plaintext payload prior to encryption** (settled item 13-c; code 1 / 2 / 3 share identical measurement basis, linked here from elsewhere). When encryption is disabled, both are synonymous; when enabled, it computes over **ciphertext**. Two independent reasons: ① Covering plaintext would prevent the non-key-holding side from reading it, instantly breaking D9 (Encryption) settled item 5 commitment to "detect corruption, report bad blocks"; ② Covering plaintext while CRC value resides in plaintext header provides non-keyed attackers a 32-bit **content confirmation oracle**—guess plaintext, compute CRC, compare, confirming correctness. Knowingly accepted cost: replicas of the same extent share identical ciphertext (D2 (RAID Stripe Policy) settled "N replicas write the same ciphertext N times, sharing one nonce") ⇒ CRC equality identifies replicas ⇒ reveals `w`; v1 `w` is constantly 2 (D2 (RAID Stripe Policy) settled item 9) ⇒ no increment in v1. **It verifies "ciphertext is uncorrupted", not "user data is uncorrupted"**: errors introduced by decryption paths are caught by MAC, not this.
- **Class identity segment — Index node**: tree ID + level + **key range** (settled item 2: carried unconditionally by all code 2 nodes) + birth generation + fsid + **write sequence 4 (instance code only) + birth sequence number 4 + payload CRC 4 + reserved 2 + self-describing three segments (key width 1 + entry count 2 + entry width 2) 5** (write sequence, payload CRC, reserved three segments see settled item 12; birth sequence number see C288 (collateral changes from mapped key settlement did not land in other decisions) item 1; self-describing three segments see settled item 18 and D8 (Core Index Structure) settled item 11; five segments sum to 19), followed by 29-byte nonce / MAC / algorithm type reserved bits (settled item 16). Semantics of each: **write sequence 4** is the sole segment among C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P1's 10 bytes having a consumer on code 2—I-1.2 (block header write sequence published) verbatim "code 2 / 3 requires b ≤ T_pub", predicate reading only instance code; it purchases allowing nodes from unpublished checkpoints to be judged garbage and swept per published predicate. **Payload CRC 4** discharges code 2's half of C104 (no self-contained evidence of payload integrity for code 1 / 2 on scan path), following code 1 and code 3 measurement basis: CRC32C, covering from header end to unit end (records + padding), falling within header checksum coverage. **Reserved 2** follows D8 (Core Index Structure) "identical policy for flags / padding / reserved": constantly 0, encountering non-zero yields EIO for that record (object-level), assigning meaning requires incompat. **Birth sequence number 4**: the 4 bytes added to code 2 / code 3 by D19 (Block Pointer Structure and Width Budget) settled item 6, **immediately following write sequence**—code 2 between write sequence 4 and payload CRC 4, code 3 after write sequence (offset 103, settled item 11); functions solely as a segment of central mapping key, not for newest selection (D19 (Block Pointer Structure and Width Budget) settled item 6). **What was removed is the 6-byte transaction number** (settled item 12). Full offset table for code 2 resides in settled item 18, not duplicated here: plaintext header 86 + 2 × key width, 115 + 2 × key width including 29-byte reserved bits (D8 (Core Index Structure) settled item 11).
- **v1 reserves field bits for nonce code and MAC** (day-1 reservation discipline of D9 (Encryption) settled item 10 "reserve format fields only"): without reservation, enabling encryption becomes a layout change, whereas D21 (Boundary Between Authoritative and Derived States) requires scan rebuild to decode all historical formats.
- **Self-verifying units (journal records / root slots / system configuration slots) do not use this table**: respective headers settled by D23 (Journal Role and Format) and D22 (How Unit Atomicity is Synthesized) settled item 7; they possess no parent references and no tree identity; I-1.1 (block header self-describes logical address) / I-1.3 (block header tree ID consistent) **explicitly exempt** them; self-verification disciplines reside in I-7 (root ring) / I-8 (journal) and D22 (How Unit Atomicity is Synthesized) settled item 21. They occupy one code each in settled item 11 registry—root record (root slot) 16, journal record 17, system configuration slot 18 (order follows settled item 11 registry), functioning solely as AAD domain separators.
- **Claiming rules**: self-backup mirrors of the same volume cannot be blocked by fsid (same fsid); that slot is blocked by **claiming rules**: after the scanner decodes and claims a unit header, it steps by whole unit (code 1 and code 3 two slots, code 2 one slot); probes inside payload no longer participate in claiming. This rule must be formalized into a test that can turn red in checker.
- **Plaintext allowlists after encryption enabled rewritten as two tables** (authoritative record in [invariants/08-加密不变量.md](../../../crates/singlefs-checker/invariants/08-加密不变量.md) I-6.2 (plaintext header field allowlist)): both classes share common prefix + nonce code + MAC; metadata identity fields (five-tuple / key range) enter ciphertext unconditionally (rebuild without key impossible anyway, settled in D21 (Boundary Between Authoritative and Derived States)); data unit class additionally permits {fsid, payload CRC} (following settled item 13 narrowing).

**Scope**: Index node header's (tree ID + level + key range) only answers **scan rebuild** identity, **not AAD identity**—AAD expected values do not come from plaintext side (D9 (Encryption) settled item 6); which identity fields index nodes use in AAD has no landing place warehouse-wide (C107 (no landing place for index node AAD identity); D9 (Encryption) settled item 9 withdrawn). Settled item 7 governs the block header half; the other half (plaintext mapping layer field table) was rescinded along with D9 (Encryption) settled item 5 adopting "scrub only" ⇒ **no longer constrained by "both halves must match field measurement basis"**; block headers are the sole plaintext expression of this tuple on disk. The load-bearing consumer of birth generation is the published predicate (C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P2); the reason "torn orphans rely on it to promote to Level 2" cannot distinguish carrying it while Level 2 witnesses do not exist (settled item 5); the other two reasons remain unaffected. The two fsid reasons describe the state when Level 2 is reachable: imposter units stop at Level 1, unable to reach promotion. After encryption is enabled, fsid and KDF are redundant; **this redundancy is accepted**, costing 8 bytes. Ownership of nonce's 12 bytes settled by settled item 14 (full 96-bit nonce resides in unit plaintext header, pointer copy retained); D9 (Encryption) settled item 4, [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) "nonce code must reside in plaintext header", and unit self-containment hold true simultaneously. Claiming rules step by whole unit, consistent with stepping measured in E88 (Distinguishing imposter units and orphan units) (step to unit origin + unit size).

**Basis**:

- E85 (Field table for unit headers): Requirement matrix and byte accounting—settled minimum set cannot decide I-1.2 (block header write sequence published) / I-1.4 (block header fsid consistent) / header self-verification; patched set fully sufficient; conflicts with I-6.2 (plaintext header field allowlist) named item by item; common prefix width adopts its measurement basis.
- E88 (Distinguishing imposter units and orphan units): Settled form produces zero misclassifications; headers stripped of fsid and birth generation misclassify imposter and torn orphan units even with claiming rules enabled ⇒ both riders hold; self-mirror slot blocked only by claiming rules.
- E104 (Current version judgment in scan rebuild): All three purposes of write sequence depend on it—the arm with only instance code and no transaction number in header failed precisely at "in-flight txg partially replayed" ⇒ code 1 write sequence must carry transaction number.
- Three-party defense (C68 (invariant allowlist conflicts with settled clauses) closeout): Verbatim output from three legs in `research/prompts/c68-unit-header-{sonnet,opus,local}-output.md`, background in `_c68-unit-header-background.md`; reverse-inference leg's three reachable vulnerabilities restored fsid and birth generation into header, header checksum taking 32 bytes. Three reasons for birth generation (kill rule input lacks landing place after rebuild, torn orphans rely on "birth greater than root checkpoint_txg" for affirmative identification, scan direction must decide I-1.2 (block header write sequence published)) and two reasons for fsid (misdirected writes and nested foreign volume mirrors, C74 (no defense against cross-volume misdirected writes during non-encrypted phase) scrub half) verbatim in change history. User approved original proposal; verdict landed per rider amendments.
- Write sequence and payload CRC: C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P1 / P3.
- User settlement (2026-10-03): Step by whole unit after claiming (code 1 and code 3 two slots, code 2 one slot), verdict `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` section 6 S2-2, verbatim in change history.

**Owed checks**: C89 (residual recalculation of unit header closeout): E98 (inode record and inode tree geometry) 91 requires re-run (claiming rules in checker exist: pool-level checker I-7.8 (root record tree ID watermark not lower than max tree ID in pool) scan direction whole-unit claiming pinned by test); C104 (no self-contained evidence of payload integrity for code 1 / 2 on scan path): check still owed; C107 (no landing place for index node AAD identity); C289 (unwritten which field code 2 birth tree takes); C291 (birth sequence number allocation rule exists only in experiment source): check still owed; C68 (invariant allowlist conflicts with settled clauses): gate comparison tooling.

#### Settled Item 8: Deletion Must Write Tombstone Units

**Settlement**: **Deletion is no longer "leaving nothing on disk"—it leaves tombstone records on disk.** V1 writes at intent publish; carrier and fields see settled item 24. Three sentences written separately: container unit type tag takes **code 3 (packed record unit)** from settled item 11 registry; records inside container take **packed record type 1 (tombstone range record)**; deceased identified by tombstone range record's own fields (object ID + object birth generation + range, settled item 10), **not substituting container identity for record identity**. Why it is necessary: if deletions leave no trace on disk, scan rebuild has **zero input** to know that a key died, and lineage rewrite sequence numbers cannot rescue this slot (it only purchases overwrite, and settled item 1 settled not to adopt it—when no trace of a message exists on disk, sequence numbers cannot compare).

**Scope**: Deletion operations themselves must write to disk, while D3 (Space Allocation) settled item 17 states verbatim "COW systems must ensure 'space release' operations themselves require no space allocation"; that account is settled by D3 (Space Allocation) settled item 2: tombstones count into admission control "how much needed to unwind oneself", paid at write time. **Not accounted for second time here.** Settled item 8 settles "leaving a tombstone unit on disk", **not when it may be reclaimed**—reclamation rules, granularity, and carrier belong to settled item 10 (reclamation rules have only initial values, criteria lack experiments). D11 (Whether to Reserve Message Buffer in Index Nodes) settled item 4 slot 3 rests on this clause.

**Basis**:

- E59 (Can buffered messages be recomputed solely from units): When scanning only units, deleted keys resurrect entirely, resurrected count exactly matching deletion count; arm with sequence number + explicit tombstone units yields all four categories 0—deletions must write tombstone units.
- E102 (Unit class registry and packed record units): Five-tuple can only identify 1 entry in a full container; packed class identity segment + parsing identifies all—deceased identified by record's own fields, not container identity.
- V1 writes at intent: User settlement K-U2 "t1: intent writes whole-object tombstone", verbatim in change history.
- User settlement, recorded in change history.

**Owed checks**: None (C634 (tombstone bytes for single unlink lacks trace check) discharged 2026-10-11, see discharged owed checks table; record field table and single deletion tombstone bytes in settled item 24).

#### Settled Item 9: "Block" Means Unit, One Header per Unit

**Settlement**: **One unit pays one copy of self-describing header; scanner steps by minimum unit size in pool.** V1 has three coexisting units—index nodes (code 2) 16 KiB, data units (code 1) and packed record units (code 3) each 32 KiB (D4 (Checksum Position) settled item 1; unit size / alignment of the three classes follows settled item 11 registry), so stepping takes 16384. **It imposes a hard requirement on the allocator**: unit origins aligned by class—**data units and packed record units align to 32768, index nodes align to 16384** (three tiers taken from settled item 11 registry "unit size / alignment" column), otherwise scanner stepping misaligns. Landing place is D3 (Space Allocation) allocator implementation, **must be made into a check forcibly violable by tests** (`.claude/rules/fs-design.md` hard requirement 2): construct an unaligned unit origin, scan rebuild must judge red.

**Scope**: Does not step by 32768—that would miss all 16 KiB nodes located on odd slots. **It is unlocked by D4 (Checksum Position) settled item 7, not by overturning settled item 6**: after D4 (Checksum Position) settled item 7 settles disk extent occupancy **constant 32768**, **unit boundaries become predictable** (stepping by minimum unit size in pool hits all, provided allocator allocates aligned to units) ⇒ "block-level" and "unit-level" coincide under this value, settled item 6 conclusion holds as-is. Allocator side code 3 placement follows D3 (Space Allocation) settled item 10 ⑤ (data unit tier, origin aligned to 32768); `crates/singlefs-core/src/allocator.rs` allocates aligned by class; checker side lacks alignment red-check (`grep -rn "对齐\|align" crates/singlefs-checker/src` yields zero hits).

**Basis**:

- E86 (Scan step and two unit sizes): Stepping by 16384 collects all, at the cost of 1 extra probe inside payload per data unit, rejected via magic plus header checksum; stepping by 32768 misses all nodes on odd slots; having nodes monopolize 32K slot pairs collects all, but doubles metadata space footprint.
- Rejecting "block = device physical block" (arithmetic): If block = 4096-byte physical block, a 32 KiB unit pays **8 copies of byte-identical headers**; on 512-byte disks, **64 copies**, headers occupying 15.6%; five-tuple values in identical headers across the same unit are pure redundancy.
- User settlement (block = unit; step measurement basis corrected to 16384), recorded in change history.

**Owed checks**: C86 (two unit sizes and scan step misaligned): red-green contrast on golden image and red-judgment on unaligned units.

#### Settled Item 10: Tombstones Take Range Records + Packed Shared Units

**Settlement**:

- **Record granularity**: Whole-object deletion writes **one range record** (object ID + object birth generation + full range); partial deletions such as truncate / punch **do not write range tombstones** (range tombstones not implemented); file shortening writes no new records; during scan rebuild, data units are judged alive/dead based on object birth generation and size from reconstructed inode records (Interpretation Z, D8 (Core Index Structure) settled item 6 "Rebuild" paragraph). Object birth generation shares measurement basis with five-tuple, compared field by field during scanning; D8 (Core Index Structure) settled item 6 only guarantees published (inode number, object birth generation) are not reused; numbers reissued after crash recovery separate tombstones of two published objects sharing same number.
- **Carrier**: Tombstone records **packed into shared tombstone units** (container is code 3 packed record unit from settled item 11 registry, record is packed record type 1), not monopolizing a unit per record.
- **Loading**: **Tombstone units loaded by generation—closing current open unit when generation ends, forbidding inter-generation mixing in same unit.** Same discipline as D2 (RAID Stripe Policy) "not allowing two objects with different lifecycles to share same physical mapping unit", applied to tombstones.
- **Reclamation rules (initial value, user measurement basis: write this first, then experiment)**: **Tombstone records cleared along snapshot deadlist cadence**—a tombstone is reclaimable after "the death it identifies has been seen off by all roots (including snapshots and writable heads) still referencing it"; quantifier takes "root", not "snapshot": clone heads are heads not snapshots; any object still alive in it but subsequently deleted in origin can have its kill record reclaimed per "snapshot" quantifier, while the corpse referenced by clone remains readable. Shares identical decision point with D5 (Snapshots / Space Accounting Mechanism) deadlist filtering upon snapshot destruction, without opening separate scans. V1 does not reclaim tombstones (settled item 24).

**Scope**: Packing couples reclamation into E80 (stripe partial release) pin_stripe pinned shape (reclamation granularity = unit). Read per container count after settled item 11: concurrently open containers = tree × packed record type × live generations × live heads (C106 (upper bound on open container count)), and open containers rewritten only on publishes having new records, un-deleted publishes copied verbatim (settled item 24; write amplification accounting in C105 (write amplification of packed containers rewritten per publish frequency)). Two interlocks: ① D3 (Space Allocation) settled item 2 tombstone reserve recalculated per settled item 10 measurement basis—reserving **records** (amortizing 1/815 unit, v1 tombstone record 40 bytes, settled item 24) not whole units; E82 (in-flight synthesis of admission) "tombstone slot" was an overestimate prior to settling granularity; ② tombstone record width fixed 40 bytes (settled item 24). Range granularity (partial deletion one per contiguous extent range) unused in v1: range tombstones out; Interpretation Z only handles file shortening; how partial deletions like punch without size changes are recorded was untouched by verdicts and unsettled.

**Basis**:

- E83 (Granularity of tombstones): Full table of four combinations × three workloads—record monopolizing unit is unusable; range records + packed shared units cheapest, gap widening on large file truncations.
- E84 (Pinned volume of packed tombstones): Mixed generation loading makes sparse deletions worst case, live tombstone units mostly holding long-reclaimable records; **loading partitioned by generation (closing open units at generation boundaries) keeps pinning constantly 0**, costing only one unfilled open unit per live generation.
- E104 (Current version judgment in scan rebuild): Tombstones unreclaimed per gate resurrect old versions—reclamation gate bears load; quantifier "root" relaxed per C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement.
- User settlement (granularity and carrier, loading discipline, initial reclamation values), recorded in change history.
- 40-byte fixed value, v1 unreclaimed, number reissue half-sentence: tombstone verdict `research/prompts/m3-tomb-r1-main-verification.md` section 3 W2 number reissue, W3 reclamation rows; `research/prompts/m3-delete-r3-main-verification.md` section 3 "tombstone round 1 rows".
- Partial deletion not writing range tombstones, file shortening taking Interpretation Z: leaf death verdict `research/prompts/m3-leaf-death-r3-main-verification.md` section 2 "Interpretation Z replaces three sets of new histories", "minimum fix for range tombstones" rows; user settlement 2026-10-08 X2 "Interpretation Z (recommended)", verbatim in change history.

**Owed checks**: Reclamation criteria owed experiment (v1 unreclaimed, record field table in settled item 24; reclamation gate dimension "data unit overwritten or swept" awaits compaction landing, C621 (deletion clauses to modify upon compaction landing lack out-of-list checks)); C105 (write amplification of packed containers rewritten per publish frequency); C106 (upper bound on open container count).

#### Settled Item 11: Value Range of Unit Type Tag Is a Registry, Packed Record Unit Is an Independent Class

**Settlement**:

**The value range has only one registration location—this registry table in D18 (Information Carried in a Block) settled item 11.** The value range row of D9 (Encryption) settled item 6, the field tables of D18 (Information Carried in a Block) settled item 7, D21 (Boundary Between Authoritative and Derived States) settled item 4, I-6.2 (plaintext header field allowlist), the two-class table of D20 (Load-Bearing Surface: Atomicity and Self-Containment of Units), and the exemption row of I-1 (block self-description) each retain judgments in their own dimensions; **class names and codes are universally taken from this table**. **What is frozen is the "code → class" column, frozen at two moments**: codes 1 / 2 / 3 are bytes genuinely existing in the common prefix, frozen the moment the first image is written; codes 16–18 do not exist on disk, appearing only in the first byte of AAD, frozen the moment encryption is powered on (modifying prior has zero cost; modifying after is an AAD composition change, a format fracture per D9 (Encryption) settled item 6). **Once a code is used, recycling is forbidden; retired code meanings are locked permanently**—sharing the identical shape and rationale as D15 (Format Freeze Policy) discipline for feature bits (bytes that genuinely existed on disk).

| Code | Class | Unit Size / Alignment | Header Follows Which Table | Class Identity Segment | AAD Variant | I-6.2 (plaintext header field allowlist) | Extension Point | How Content Is Enumerated After Claiming |
|---|---|---|---|---|---|---|---|---|
| 0 | **Invalid** (never allocated) | — | — | — | — | — | — | Judged corrupt upon sighting |
| 1 | **Data unit** | 32768 / 32768 | D18 (Information Carried in a Block) settled item 7 Data Unit Class | Five-tuple + birth generation + fsid + write sequence + payload CRC (plaintext header 105 / 134 with reserved bits) | Five items of D9 (Encryption) settled item 6—citations must include index row note from D9 (Encryption) settled item 6: the five items are blind and silent to snapshot dimensions, field table owes a reopening; tree ID takes reader tree | Data unit class | Carried (D21 (Boundary Between Authoritative and Derived States) settled item 4) | Payload is user data, not enumerated |
| 2 | **Index node** | 16384 / 16384 | D18 (Information Carried in a Block) settled item 7 Index Node Class | key width 1 + tree ID + level + key range + birth generation + fsid + write sequence 4 + birth sequence number 4 + payload CRC 4 + reserved 2 + entry count 2 + entry width 2 (86 + 2k, 115 + 2k with reserved bits; offset table in settled item 18) | **Unsettled, no landing place** (D9 (Encryption) day-1 item ③)—C107 (no landing place for index node AAD identity) | Metadata class | Conditionally carried (D21 (Boundary Between Authoritative and Derived States) settled item 4) | Decode entries per node format |
| 3 | **Packed record unit** | 32768 / 32768 | Common prefix + D18 (Information Carried in a Block) settled item 11 Class Identity Segment Table | 65 bytes ⇒ header **107** (excluding nonce / MAC reserved bits, same measurement basis as 105; 136 with reserved bits) | (tag, birth tree ID, packed record type, container number, container birth generation)—tree dimension takes **birth tree**, the semantics code 1 must also adopt following D9 (Encryption) reopening; format meanings of E90 (Tree ID entering AAD and sharing across headers) (pointers carry birth (tree, txg), clone ancestor table in authenticated structure) belong to that reopening; prior to their landing, code 3 AAD does not constitute complete contract | Metadata class | Carried (D21 (Boundary Between Authoritative and Derived States) settled item 8: flush against unit tail, v1 N = 0); red-turning check owed in C108 (extension point attribution has no judgment for packed record units) | Fixed-width sliced by record count × record width |
| 4–15 | Reserved for future content classes | | | | | | | Unregistered ⇒ rejected; adding a class occupies one incompat bit (D15 (Format Freeze Policy) judgment table) |
| 16 | Root record (root slot) | Slot width | D22 (How Unit Atomicity is Synthesized) settled item 7 | Own | Belongs to D22 (How Unit Atomicity is Synthesized) / D9 (Encryption) | Not applicable (self-verifying) | Not carried | Functions only as AAD domain separator |
| 17 | Journal record | Record | D23 (Journal Role and Format) settled item 1 item A + settled item 4 487-byte header (including settled item 15 new root segment 364, fsid 8, and MAC 16) | Own | Belongs to D23 (Journal Role and Format) / D9 (Encryption) | Not applicable | Not carried | Functions only as AAD domain separator |
| 18 | System configuration slot | Slot width | D22 (How Unit Atomicity is Synthesized) settled item 9 | Own | Placeholder, unsettled | Not applicable | Not carried | Functions only as AAD domain separator |
| 19–31 | Reserved for future self-verifying structures | | | | | | | Same freeze moment as 16–18 |
| 32–255 | Reserved; allocation rules not settled by settled item 11, any future allocation passes D15 (Format Freeze Policy) judgment table | | | | | | | |

- **How the four names from D9 (Encryption) settled item 6 land**: data extent → 1; btree node → 2; root record → 16; **accounting unit → 2**—D9 (Encryption) settled item 6 provides tags because "different unit classes have different field sets... serializable into identical byte strings", whereas accounting tree nodes and other index nodes share identical field sets in class identity segments ⇒ domain separation purchases nothing here; "accounting" is a role indicated by tree ID. The two differ in state classification (accounting is authoritative state, noted in C102 (no reserved area in index node headers; adding fields is not compat)); state classification is not a criterion for I-6.2 (plaintext header field allowlist) or extension points; if accounting is separated into its own class later by state, D15 (Format Freeze Policy) "altering meaning of existing fields" rules it incompat and code 2 is not reused. Scope of D9 (Encryption) day-1 item ① narrows with value range: only "distinction between accounting / non-accounting forbidden to infer from tree ID" is narrowed away—code 2 itself is an explicit tag value; ① holds as-is for remaining codes. "Accounting unit" in D22 (How Unit Atomicity is Synthesized) settled item 3 basis is henceforth a role name (accounting tree node), not a class name. **"Tombstone" in D18 (Information Carried in a Block) settled item 8**: container = code 3, record = packed record type 1. Self-verifying units also occupy codes because an injective encoding is required, and a unified namespace is the cheapest way to achieve it (1 byte vs 4-byte magic).
- **Tag width and position**: Unit type tag = single byte at common prefix **offset 6** (flags two bytes defined independently, byte-order agnostic); **offset 7 is 8 flag bits, v1 constantly 0, readers encountering non-zero unconditionally reject**—this is forward compatibility policy, not corruption detection (header checksum already covers it): providing no "ignore unrecognized bits" path ⇒ assigning meaning to any bit in future is judged incompat per D15 (Format Freeze Policy) "altering meaning of existing fields"; it is not a reserved area in the sense of row 1 of judgment table (cannot fit fields), which C102 (no reserved area in index node headers; adding fields is not compat) must specify. Take this side only for unit header flags; inode attribute presence bits in D8 (Core Index Structure) settled item 6 belong to another bit space, a dilemma settled item 11 does not resolve. Common prefix 42 untouched by a single character. Data unit five-tuple first byte must equal offset 6; mismatch judged corrupt. In AAD canonical encoding, tag code is the first byte across all classes. Out of 256 values, 7 registered, 249 remaining.
- **Breakdown of five-tuple 33: tag 1 + tree ID 8 + object ID 8 + object birth generation 8 + anchor offset 8** (question 8 of [verification-build.md](../verification-build.md) closed). **Alternatives were not debated; user settled directly based on total 33 and existing origins of the four 8s**: tree ID 4, birth generation 6 (48 bits sufficient for 3,000 years at 2,785 publishes/sec per E44 (Sequence bit-width on-machine measurement and cost)), anchor offset 6 at unit granularity—any would alter 33; no longer options after settlement. **Two candidates for six-tuple are not in this settlement**: `locality_id` (settled item 3 proposal, rejected by user settlement) and birth snapshot dimension (owed reopening of D9 (Encryption) settled item 6 index row), both permanent changes to AAD composition, belonging to their respective items.
- **Packed record unit (code 3) is an independent class**: D21 (Boundary Between Authoritative and Derived States) hard constraint 6 (uncomputable authoritative values forbidden to reside in rebuildable containers) excludes code 2; five-tuple object ID can accommodate only one ⇒ excludes code 1—tombstones identify the deceased, whereas **no** index holds it anymore (premise of settled item 3 that "same information already lies in plaintext reverse index" vanished with removal of plaintext side, conclusion not relying on it).

**Code 3 class identity segment (65 bytes, header 107)**:

| Offset | Field | Width | Why |
|---|---|---|---|
| 42 | unit type tag | 1 | Ciphertext-side replica of offset 6 (I-6.2 (plaintext header field allowlist) supplementary note requires allowlisted items have authoritative replicas in ciphertext). ⚠️ Code 2 lacks this replica, C111 (class tag lacks replica on ciphertext side) |
| 43 | birth tree ID | 8 | Object IDs inevitably collide across keyspaces (D9 (Encryption) settled item 6); a container holds records for only one tree |
| 51 | packed record type | 2 | Code of secondary registry; a container holds only one type |
| 53 | container number | 8 | Allocation rules split by packed record type: type 1 (tombstones) allocated monotonically within tree (C103 (container index lacks landing place) scheme, rollback does not break identity); type 2 (inode records) = inode number of new record triggering split, needing no allocator (D8 (Core Index Structure) settled item 6). Both publish-independent, invariant when container rewritten via COW; 4 bytes wraps in 17 days at 2,785 publishes/sec per E44 (Sequence bit-width on-machine measurement and cost) |
| 61 | container birth generation | 8 | checkpoint_txg of the publish allocating container number; combined with container number forms stable identity, sharing division of labor with data unit's (object ID, object birth generation) |
| 69 | record count | 2 | ≤ 1359 (header 136 with reserved bits, settled item 16; narrowest record is type 5 lineage record, width 24, settled item 24) |
| 71 | record width | 2 | Provided by registry, duplicated in header: allows readers unfamiliar with record type to verify header legality conditions (analogous to D23 (Journal Role and Format) settled item 1 item B) |
| 73 | birth generation | 8 | Records only this write (three reasons for returning to header in D18 (Information Carried in a Block) settled item 7) |
| 81 | fsid | 8 | Same rationale as D18 (Information Carried in a Block) settled item 7 |
| 89 | payload checksum | 4 | CRC32C, covering offset 107 to unit end (nonce / MAC reserved bits + records + padding). Whole-unit checksum / MAC resides in parent pointer (D4 (Checksum Position)); scanning lacks parent pointers, header checksum serves only as header integrity line of defense (E76 (Discriminative power of payload checksum): header lands while payload half-lands, undetectable by header checksum alone). Width tied to D23 (Journal Role and Format) settled item 13 |
| 93 | write sequence (instance code 4 + transaction number low 48 bits) | 10 | C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P1, identical field as code 1; containers written only at checkpoint fixed points, transaction number takes maximum allocated transaction number in that checkpoint; **on code 3 it serves solely as tiebreaker within same checkpoint** (I-1.8 (total version order after merge) total order key for code 3 is (birth generation, instance code, transaction number)); code 2 write sequence narrowed to 4 bytes per settled item 12 (instance code only), not entering P3 or I-1.8 (total version order after merge); containers do not independently add instance code, the 4 bytes in write sequence serve as it (D8 (Core Index Structure) settled item 6 reopened thereby) |
| 103 | birth sequence number | 4 | D19 (Block Pointer Structure and Width Budget) settled item 6: code 3 central mapping key = class tag + birth tree (offset 43) + birth txg (birth generation, offset 73) + instance code (write sequence first 4 bytes, offset 93) + birth sequence number; functions only as segment of key, not for newest selection; position immediately follows write sequence (C288 (collateral changes from mapped key settlement did not land in other decisions) item ① takes option A) |

- Nonce / MAC / algorithm type 29 bytes start at offset 107; code 3 header with reserved bits is 136 (settled item 14 / settled item 16).
- **Header checksum coverage (authoritative entry I-2.4 (header checksum coverage))**: covers offset 0 to final byte of that class's plaintext header, endpoints following field tables of D18 (Information Carried in a Block) settled item 7 / settled item 11 / settled item 18 (current initial values: code 1 to offset 104, code 2 to offset 85 + 2 × key width, code 3 to offset 106), header checksum field itself participates as zero; the 29-byte nonce / MAC / algorithm type reserved bits are outside header checksum coverage (I-2.4 (header checksum coverage)); after encryption is enabled (system configuration level, all-or-nothing, I-6.4 (no unencrypted blocks)), metadata class identity segments enter ciphertext protected by MAC; header checksum coverage = all plaintext header fields in I-6.2 (plaintext header field allowlist) for that class, not listed separately—**under same format version it splits into two forms by system configuration encryption declaration, distinguished by encryption bit of D9 (Encryption) settled item 10 slot**. ⇒ In v1 payload checksum falls within header checksum coverage (landing requirement of D23 (Journal Role and Format) settled item 13).
- **Declared length measurement basis identical to code 1**: declared length = net payload bytes following header, padding origin = header + declared length, declared length ≤ 32768 − 136 (header with 29-byte reserved bits, settled item 16).
- **No fields carried at record level**: uncommitted torn writes judged garbage by I-1.2 (block header write sequence published); post-commit corrupted payloads judged by payload checksum (scan path) and parent pointer checksum / MAC (normal path); reclamation granularity = unit, live bit has no object; one container holds one fixed-width record type, type and length once in header suffice. **I-2.3 (padding bytes zero and participate in checksum) scope extended to code 3**. Judgment order: header checksum → class identity segment → I-1.2 (block header write sequence published) / I-1.3 (block header tree ID consistent) / I-1.4 (block header fsid consistent) → record count × record width ≤ declared length → payload checksum → record by record, specified in I-1.7 (packed container validity and judgment order).
- **Rationale for I-6.2 (plaintext header field allowlist) classifying under metadata class**: allowlist separates what non-keyed side can read, not state classification—accounting tree nodes are authoritative state, categorized under metadata class; non-keyed side per D9 (Encryption) settled item 5 performs only detection and relocation; container identity is useless to it.
- **COW rewriting and indexing of containers**: open containers rewritten only on publishes having new records within their owning generation (settled item 24), new unit, new birth generation, identity invariant. **Container index holds packed unit pointer headers (MAC / nonce / algorithm type) and position hints, authoritative position in central mapping (type 4 instance table exception: physical pointer held directly by root record, D22 (How Unit Atomicity is Synthesized) settled item 7); other trees reference containers by identity**—cost of updating a container splits by packed record type: type 1 (tombstone) = COW it + modify one code 2 node referencing it in tombstone tree (settled item 24; C103 (container index lacks landing place) leaves only stripe member table half); type 2 (inode record) = COW it + modify all ancestors (container index is inode tree internal node, D8 (Core Index Structure) settled item 6), which is the distinction between inode packing (D8 (Core Index Structure) settled item 6) and cross-file tail packing rejected in D14 (Dual-Track (Large/Small Files / Persistent/Temporary)). Old container versions follow ordinary COW release path, writing no tombstones. Physical pointers and MAC monopolized by container index are derived values (MAC = f(key, ciphertext, AAD), recomputable; recomputed authentication strength = trust in scanned payload, precisely the degraded semantics of D18 (Information Carried in a Block) settled item 5, treating container indices and data pointers in extent trees equally) ⇒ does not violate D21 (Boundary Between Authoritative and Derived States) hard constraint 6. **Code 2 does not enter scan rebuild version selection rules**: inode tree internal nodes and root are derived state, rebuild reconstructs from leaves (leaves are code 3, self-carrying payload checksum, D8 (Core Index Structure) settled item 6); accounting tree nodes are authoritative state, but entries take idempotent complete values (D8 (Core Index Structure) settled item 1), multi-versions **merged at entry level** (D5 (Snapshots / Space Accounting Mechanism) settled item 2: monotonic statistics take maximum, non-monotonic statistics take "latter" determined by entry's own seq), **needing no container-level version selection**; unit class code is not a criterion for state classification (D21 (Boundary Between Authoritative and Derived States)). **When scan rebuild finds multi-version containers of same identity simultaneously, judged per C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement rules**: filter unpublished versions via instance table published predicate (settlement P2), then among visible candidates for that root take maximum of **that class's newest-selection key** (settlement P3: code 1 takes write sequence, code 3 takes (birth generation, instance code, transaction number) triplet—birth generation leads because across two consecutive checkpoints of same instance without allocating new transaction numbers, two version write sequences are byte-identical; transaction number remains last as tiebreaker within same checkpoint; fixed points iterate, same container not guaranteed to write only once in a checkpoint); two versions can belong to different heads simultaneously; normal path still judged by D3 (Space Allocation) settled item 1 predicate. Birth generation not used as newest-selection key: write sequence purchases publishedness and timeline (settled item 1 scope), incidentally ordering same-txg cross-transaction modifications of same key. Containers rewritten each publish, weighing far more than data extents (1,000 files / 100 fsyncs ⇒ 100 physical versions for one identity).
- **Four loading disciplines**: ① One packed record type per container; ② One generation per container—generalizing discipline from settled item 10 for tombstones to generation-reclaimed packed record types (type 1 only; type 2 records die individually, containers reclaimed via B-tree split/merge, leaf lives many generations, D8 (Core Index Structure) settled item 6) (D2 (RAID Stripe Policy) write granularity item 2); generation not stored as field, = segment in live snapshot list (I-3.7 (live snapshots enumerable)) containing container birth generation, **used only in reclamation judgment, unused in scan rebuild**; before snapshot publish, currently open containers must be closed and written out (D16 (Publish Semantics) settled item 7 same ordering; likewise type 1 only—type 2 leaves are not open containers in generational sense, snapshots merely share them); ③ One tree per container; ④ Records fixed-width, record count × record width ≤ declared length, verified upon claiming.
- **Packed record type is secondary registry**: 0 invalid (0 in container header judged corrupt; when appearing as identity reference in inode tree internal node entries, 0 indicates "child is code 2 index node", registered in D8 (Core Index Structure) settled item 6 and I-9.2 (entry identity matches child header), sharing no judgment path with container headers); **1 tombstone range record** (width 40, field table in settled item 24; 815 entries per 32 KiB container (record area origin 136); E83 (Granularity of tombstones) and E102 (Unit class registry and packed record units) re-run second time per 40 bytes; E84 (Pinned volume of packed tombstones) still computed per 56 bytes, re-run owed in C691 (packed tombstone harness hardcodes capacity to 583)); **2 inode record** (enabled, D8 (Core Index Structure) settled item 6: fixed length 140, 233 entries per container, containers are leaves of inode tree); **3 retirement record** (C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P5 / P6: retired when leaf container identity disappears—deleted empty removed containers, right halves absorbed during merges, old identities upon rebirth; record = retired (birth tree, type, container number, birth generation) four-tuple plus death txg (checkpoint_txg of publish where identity disappeared), lacking write sequence; resides in own container, container enters dedicated retirement tree, container numbers allocated monotonically per type 1; read side filters per death txg and lineage; retirement tree, tombstone tree, and lineage tree follow active trees selecting versions solely per C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P2; administrator rollback adopts active version of retirement tree, not taking R_old (D23 (Journal Role and Format) settled item 14 unmodified); retirement tree kind code 13 (D8 (Core Index Structure) settled item 9), tree ID 65548 (D8 (Core Index Structure) settled item 11), head ID writes inode tree tree ID (D8 (Core Index Structure) settled item 8), placed last in tree table; record width 34, 32 KiB container holds ⌊(32768 − 136) ÷ 34⌋ = 959 entries; field table (little-endian, no padding, no flags, no reserved): retired identity birth tree 8 | packed record type 2 | container number 8 | container birth generation 8 | death txg 8, first four segments same order and width as identity reference); **4 instance table** (C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P2, field table see "packed record type 4 (instance table) is independent record format"); **5 lineage record** (width 24, field table in settled item 24); 6–65535 reserved. Types 3 / 4 default to incompat per D15 (Format Freeze Policy), zero cost prior to freeze.
- **Type 1 containers monopolized by writer tree** (C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P6): prior to snapshot publish current open container closed and written (settled item 10 loading discipline); clones do not incorporate origin type 1 containers into own container indices; reclamation (rewriting) performed only by writer tree; other heads read them via ancestor tables during scan rebuild; clone head type 1 container numbers allocated monotonically within tree from 1, identities separated by birth tree ⇒ type 1 never COWs across heads; cross-head first COW rebirth applies only to type 2. **Other heads reading them via ancestor table requires version referenced by fork-point snapshot never released** (D6 (Snapshot Implementation Model) settlement collateral trigger item 2: fork-point snapshot forbidden to destroy)—while D6 (Snapshot Implementation Model) nine adoption conditions item 5 explicitly states that was evaded by functional omission, not satisfied; thus "fork-point snapshot destroyable / writable head destroyable" is prerequisite for C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement, expiring simultaneously. Cost: any object alive at fork point and subsequently deleted in origin has kill record that can never be reclaimed (gate never opens), tombstone containers growing monotonically without shrinking—third account beyond C105 (write amplification of packed containers rewritten per publish frequency) and C106 (upper bound on open container count).
- **Packed record type 4 (instance table) is an independent record format, not following "one fixed-width record type per container" model**: record width constant 88 (chain pointer records same width as row records), partitioned into two kinds by first byte `kind`—
  - `kind` = 0 row record: `kind 1 | instance code 4 | selected root checkpoint_txg 8 | maximum applied transaction number W belonging to this instance 8 | flags 1 (v1 constantly 0, non-zero rejected; bit0 retired, unreclaimed—formerly marked rollback rows, no future reassignments, sharing discipline with unit type code "never recycle once used") | reserved 66`;
  <!-- format-const: INSTANCE_ROW_BYTES = 88 -->
  <!-- format-const: INSTANCE_ROW_RESERVED = 66 -->
  <!-- format-const: INSTANCE_TABLE_PAGE_RECORDS = 370 -->
  - `kind` = 1 chain pointer record (constantly last entry of slice): `kind 1 | has next slice 1 | position pointer 86 (zeroed placeholder when no next slice; same format as instance table unit pointer in root record, D19 (Block Pointer Structure and Width Budget) settled items 7 / 8) | reserved 0`. Packed records fixed length, kind 1 and kind 0 must share identical width (`INSTANCE_ROW_BYTES`); with reserved zeroed, this record has no headroom: widening pointer by 1 byte increases `INSTANCE_ROW_BYTES`, altering slice entry count.
  - Records with `kind` neither 0 nor 1: slice undecodable (table unreadable), analogous to row record flags non-zero rejected and unit type tag unregistered; chain pointer record not at slice tail, or tail not legal chain pointer record (including claiming next slice but position pointer header segment not v1 value: MAC / nonce, algorithm type, compression algorithm code and compressed length, extent offset; v1 values see D19 (Block Pointer Structure and Width Budget) settled item 3), likewise undecodable. Check is I-3.14 (instance table per-slice record form legal).
  - **Identity and capacity**: Four-tuple (birth tree 0, type 4, container number = slice index from 0, birth generation 0); birth tree 0 shares convention with D5 (Snapshots / Space Accounting Mechanism) "pool-wide / unattributed" rows; belongs to no tree, does not enter container index, physical pointer held directly by root record (D22 (How Unit Atomicity is Synthesized) settled item 7). One slice holds ⌊(32768 − 136) / 88⌋ = 370 records (including chain pointer, data rows 369; when rows exceed 369, slice fills 369 rows before opening next, final slice holding remainder; 136 is code 3 header with nonce / MAC / algorithm type reserved bits, settled item 14 / settled item 16); mkfs writes one empty table slice, record count 1 (sole entry is "no next slice" chain pointer record).
  - **How rows are written** (C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P2): unique per instance code, later writes overwrite; written only during recovery and instance switch (administrator rollback is forward publish while mounted, writing no rows, D23 (Journal Role and Format) settled item 14), same publish as first new root—**every writable mount writes a row** (instance 0 does not write; row-writing publish is first publish of new instance; every root referenced by new instance includes rows for [max(selected root instance, 1), new instance) (excluding rows deleted by sweep publish); no empty publishes raising F pushed before row write, row-writing metadata uses switch reserve, D28 (Mount-Time Committed Volume) settled item 3); each row write COW-rewrites entire chain.
  - **Rows written by recovery**: recovery writes one row for each instance in [max(selected root instance, 1), new instance) on table version pointed to by selected root—selected root instance writes (i, **checkpoint_txg of root following replay**, **maximum transaction number belonging to instance i applied by this replay**); instances strictly between selected root instance and new instance write (i, 0, 0)—jsn = instance code 32 bits + counter 48 bits (D23 (Journal Role and Format) settled item 9); replay strictly sequential per jsn, stopping on gaps ⇒ no replay crosses instance boundaries ⇒ **among rows written by the same recovery, only selected root instance can have non-zero W; among rows written by the same switch, only write sequence owner of as-is transactions can have non-zero W; remainder constantly 0** (restricted to "batch written by same invocation": subsequent recoveries write rows only for [new selected root instance, new instance), not modifying old rows, allowing multiple rows with W ≠ 0 from different recoveries or switches). Recovery constantly selects latest readable self-verifying root, which may not be latest published root (bad slot retreats one slot, subsequent un-linkable records unapplied, W = 0); applying zero records writes (i, selected root txg, 0).
  - **Rows written by switch**: instance switch (D23 (Journal Role and Format) settled item 14 note) identical to crash rows, except **selected root taken per D23 (Journal Role and Format) settled item 14 instance switch clause**—rereading disk for root selection, not taking in-memory root from caller (C458 (instance switch takes in-memory root without rereading disk)); switch does not replay, rows fall only on [this root instance, new instance), one row per instance, taking first applicable among ①–④: ① rows written by re-issued publish re-issued verbatim (recovery row and intermediate instance rows of row-writing publish); ② write sequence instance k of as-is transactions writes (k, k's last published root txg (0 if never published root), maximum transaction number belonging to k among as-is); ③ selected root instance writes (i, this root txg, 0); ④ remainder write (i, 0, 0)—none of ② applies if re-issued checkpoint contains no records with non-zero transaction numbers.
  - **W is an exact prefix**: transaction numbers monotonic within instance, records appended in transaction number order (holding constantly under v1 serial commits); transaction failing after issuing unit writes triggers instance switch, appending zero higher-numbered records (D23 (Journal Role and Format) settled item 7 note).
  - **Published predicate (global)**: i_now = mounted root instance, unit write sequence (i, n), birth generation b—i < i_now and no row ⇒ published; with row (i, T_pub, W): code 1 ⇒ b ≤ T_pub ∨ n ≤ W (in selected root, or applied by replay), code 2 / 3 ⇒ b ≤ T_pub (fixed-point units following root after replay are unpublished, rewritten by recovery instance; fixed-point units of publishes applied by replay referenced by new root segments in records (D23 (Journal Role and Format) settled item 15), judged published per T_pub = root txg after replay; code 3 6-byte transaction number does not enter published predicate—functions only as tiebreaker within same checkpoint (I-1.8 (total version order after merge) total order key for code 3 is (birth generation, instance code, transaction number) triplet, C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement version 9); code 2 write sequence narrowed to instance code only (settled item 12)); i == i_now ⇒ b ≤ mounted root txg; i > i_now ⇒ judged corrupt. "No row ⇒ published" holds only for same timeline.
  - **Reclamation**: row deletable if and only if whole-round sweep yields determination for **every placement in pool** (read successful and judged per row, header wiped, or overwritten) with no unpublished units of that instance, zero read failures during that sweep (unreadability ≠ non-existence, sector dimension; reread prior to wiping header counts: reread reporting device error or reading back different header aborts deletion this sweep, user settled 2026-10-04, zero rounds), all devices online (device dimension), and root ring contains no roots published by that instance—every root ring slot must read successfully to count as "absent"; unreadable slots count as "potentially containing root of that instance" (temporarily unreadable root slot causing row deletion would return abandoned root to rollback candidate set once restored, while referenced units were already wiped by sweep; cost monitored by C597 (instance table rows monotonic without deletion clauses)); all-zero root slots read successfully treated as unreadable (count as "potentially containing root"), aborting deletion this round; remaining slots must pass self-verification, otherwise reread once, still failing aborts deletion. **Row deletion falls on dedicated sweep publish**: header wiping fully FUA returned, passing barrier before issuing sweep publish; read failure or incomplete devices aborts deletion. **Order is judge first, wipe second**: verify every root ring slot self-verified and ring contains no root of that instance; wipe only after passing, wiping before judging forbidden. **Shape of sweep publish**: transaction number 0, not entering W, instance table full chain rewritten, mount admission formula un-evaluated, deleting all matching rows at once; no dedicated reserve; when placement for sweep publish rejected, sweep does not delete rows, mount returns writable as normal. **Who runs and when**: evaluated at writable mount conclusion (after writing rows, warm-up, raising F, continuing deletions, before handing back). When toggle off, instance table row count exceeding threshold triggers immediate judgment, wipe, and sweep publish during mount; below threshold does not sweep; when toggle on, handed to background sweep while mounted; background rereads slot header in same serial segment before wiping, wiping only if unpublished unit of candidate row. Threshold is config item, default settled separately (C690 (default sweep trigger threshold unsettled)). **How every placement in pool is read**: claim units per claiming rules (settled item 7), skip slots still allocated in accounting of any readable root in root ring (roots below active F (D16 (Publish Semantics) settled item 1) skip accounting, user settled 2026-10-04, zero rounds); sweep identifies "is unit" solely via header checksum, no stricter than any disk-reading judgment. Quantifier takes "every in pool" not "every written by that instance": wiped or overwritten placements cannot identify original owner; writing per latter requires enumerating the very information being destroyed (abandoned roots retained in ring have rows serving as inputs to rollback candidate criteria); zoned v1 does not reclaim, chain length unbounded (C121 (unbounded instance table chain length on zoned)).
  - **When table unreadable**: read-only mount; scan rebuild stops at Level 1—losing table leaves zero rows, all units of all old instances fall into "no row ⇒ published", including timelines abandoned by crash recovery, so ambiguity is not recorded. Instance table units themselves bypass I-1.2 (block header write sequence published) predicate: active version uniquely determined by physical pointer in root record; pointer un-targetable or failing checksum means table unreadable.
  - **Invalidating checkpoint must change instance code**: current instance has no row, predicate can only ask "birth generation ≤ mounted root txg"; re-issuing same code published causes orphans with matching birth generation to be judged published—basis for closing failure tables (D23 (Journal Role and Format) settled item 14 note). **Instance code** resides in system configuration (E100 (system configuration three-segment geometry) segment 4 4 bytes).
  - **Writable mount ordering**: verify if writable mount permitted (writable devices meet w lower bound ∧ **exclusively open majority of devices in pool**—any two majorities intersect, per-device exclusive open becomes pool-level mutual exclusion; this and its cost (4-disk pool with 2 writable can only read-only mount) settled by D2 (RAID Stripe Policy) settled item 13, not settled item 11 ∧ instance table readable ∧ instance switch reserve obtainable ∧ dry-run on allocator replica succeeds for all placements of row-writing and warm-up for this mount—dry-run release verification error likewise rejects writable mount; dry-run omits pre-release disk read check, difference see C542 (dry-run before allocating code omits disk read check, bad disk plus overwritten root burns code)), then allocate new code = max(**system configuration codes across set exclusively opened successfully this mount** (all self-verified slots across two slots per disk—checksum valid and fsid matching pool), instance codes of all root records in root ring) + 1 and write it in two rounds into both system configuration slots of every disk in **that set** (each round writes one slot per disk followed by one pool barrier, and the new code is handed out only after both rounds complete; `crates/singlefs-core/src/transaction.rs` `write_the_witnessed_acquisition`), **all-or-nothing** (system configuration writes follow data units: single I/O error retries until T_retry exhausted before failing; upon exhaustion failing all-or-nothing **first rolls back the written disks to old code, writing the rollback only once per disk** (old code = max instance code among all self-verified slots in set prior to allocation; rollback write occurs before any unit), falling back to read-only mount only if rollback fails—"unequal codes within same set" appearing on disk during rollback judged valid per I-7.7 (system configuration instance code not lower than root ring) clause 2 (larger code carried by no root, record, or unit), opening no exception; subsequent mount repairs via max + 1; barrier following each allocation round (D23 (Journal Role and Format) settled item 16) erroring on any disk likewise fails all-or-nothing, continuing merely by reissuing barrier forbidden; allocation write set identical to max valuation set—writing "all visible" while exclusively opening subset leaves disks outside set subject to external factors, mutually forcing read-only mounts. **Majority is admission threshold, not write scope**: write set takes all devices successfully opened exclusively; writing only majority strands healthy disks outside set with stale codes, indistinguishable from truly missing allocation. **"Visible" = exclusive open successful and system configuration readable**; I-7.7 (system configuration instance code not lower than root ring) clause 2 restricted to same set), after which units may be touched; read-only mounts allocate no codes; starts from 1, 0 invalid.
  - **Check selected version presence per disk before allocating code** (main agent settled 2026-09-25, survived zero rounds of attack; implemented in `crates/singlefs-core/src/mount.rs` `devices_without_the_selected_version`, impl 27): writable mount checks version after applying prefix; any disk missing it rejects writable prior to allocation and any write (`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`); read-only mount skips this, trying each position entry sequentially to read selected version. "Missing" takes two forms: neither system configuration slot self-verified (valid checksum, matching fsid), meaning disk not "visible"; highest-generation system configuration's (instance code, journal tail) strictly less than jsn of last record in selected version publish, **and** selected version has units on this disk with unreadable placements or differing bytes while this disk's own journal ring slots lack the last record of that version's publish, or selected version is selected root whose slot belongs to this disk and reading twice fails to match (unreadable, self-verification failing, or self-verified with differing (instance, txg); rejected as `SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndItsRootNotInItsRingSlot`); versions derived from journal records crashing before root slot FUA exist in no disk's root ring, skipping root slot check. Three permitted forms: lagging behind, units present, root slot intact (crashed after root landed before rotation; allocation failure rolls back to witness value at allocation (C554 (shadow accounting uncomputable when crash-recovery abandoned root temporarily unreadable) B-config-cont)), normal crash state; system configuration current, missing few units (single-copy corruption, isolated by disk read check in D19 (Block Pointer Structure and Width Budget) settled item 5 hard rule 1); lagging while this disk's journal ring slots hold the last record of that version, missing a few units—units are written before the barrier of the last record, and on this disk the last record is the only on-disk evidence that separates "one copy went bad after landing" from "never landed" (the form of C593 (Lagging disk missing one replica of unit permanently refused writable); `crates/singlefs-core/src/mount.rs` `units_of_the_version_missing_on_a_device_behind_it` first checks `last_record_of_the_version_in_its_ring_slot_on_the_device`). After mounting, each entry ingesting disk tables (normal unmount, administrator rollback, session publish, raising F) checks three things before writes: "visible" branch (rejected if a disk has zero self-verified system configuration slots for pool fsid); after "visible", own device number in self-verified slot equals identity from disk table; lagging branch—latest self-verified system configuration's (instance code, journal tail) strictly less than active version's last record jsn, **and** (active version has unit on disk with unreadable placement or bytes differing from verified copy in this process while this disk's own journal ring slots lack that version's last record, or active version root slot belongs to disk and reading twice fails to match), then reject (Approach C, sharing judgment with writable mount lagging branch, substituting "selected version" with caller's active version; missing root reread follows D16 (Publish Semantics) settled item 1 "root slot reads bad this time" reading twice; missing unit directly recorded without reread; lagging behind with units present and matching root not rejected, avoiding false rejections for disks with one self-verified slot stopped at prior rotation). Latter two user settled, origins and basis in code three-party round 2 verdict `research/prompts/m2-closeout-code-r2-main-verification.md` "same-pool old snapshot disk bleached via entry" and "swapped disk table" slots; lagging branch takes "lagging and missing units", not "lagging rejected", following attacker prototype `fix_prototype_swapped_or_behind.patch` (`m2-impl-r2-fixes-b-implementer-report.md` section 4 item 6). Session "each publish" reads as "each user change" (called once before loop); each raise-F sequence pushed for space in loop rechecks all three immediately before reissue, ensuring every actual write checked—inferred, no tests swap disks mid-loop. All three implemented: "visible" branch see `m2-impl-z3a-yi-implementer-report.md`; own device number and lagging branch in `crates/singlefs-core/src/mount.rs` `caller_inputs_agreeing_with_the_disk` (normal unmount, raise F, administrator rollback; device number check is `own_device_numbers_on_disk_differing_from_the_identities_handed_in`, lagging branch is `devices_behind_the_current_version`, sharing `what_a_device_behind_the_version_lacks` with writable mount: first `units_of_the_version_missing_on_a_device_behind_it`, then `root_of_the_version_not_in_its_ring_slot_on_the_device`) and `crates/singlefs-core/src/mounted_session.rs` `publish_user_change` (session), tests in `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`. Error variants upon rejection: "visible"—normal unmount, raise F report `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration` (`disagreeing_device_table` carries `NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`), administrator rollback reports `RollbackError::CallerInputsDisagreeWithTheDisk`, session reports `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`; device number—normal unmount, raise F report same `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration` (`disagreeing_device_table` carries `OwnDeviceNumberDiffersFromTheIdentityHandedIn`), administrator rollback reports `RollbackError::CallerInputsDisagreeWithTheDisk`, session reports `UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`; lagging branch—normal unmount, raise F report `MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`, administrator rollback reports `RollbackError::CallerInputsDisagreeWithTheDisk` (carrying `CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`), session reports `UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`.
  - **Premise**: exactly one writable mount at any moment—guaranteed by majority exclusive open after C833 (writable mount lacks exclusive open) lands (today two processes can still mount writable simultaneously); on multi-host shared storage two writable mounts compute identical codes, predicate unable to distinguish them, declared unprevented along with D9 (Encryption) settled item 8. Incomplete visibility of two-way witnesses at allocation not intercepted extra, sharing slot with whole-volume rollback; I-7.6 (root ring zones land on distinct disks) makes root ring independent witness only when devs ≥ R, failing in v1 (R = 3, devs = 2). Disks missing allocation split into two: **opened and subsequently dropped, with publishes occurring during absence**, entire disk invalidated upon return, read-only until resync completes; **never exclusively opened by this mount**, or zero publishes occurring during absence, not invalidated—physical possibility of branching absent; v1 2 disks, dropping one allows only read-only mount ⇒ split-brain disk return unreachable; D2 (RAID Stripe Policy) settled items 3 / 4 permit online disk addition ⇒ reachable once third disk added; C120 (judgment and resynchronization of split-brain disk return) expires before permitting third disk, alongside majority rule.
  - Distinct namespaces from D23 (Journal Role and Format) record header "type", with different policies.
- **Authoritative recording point for both policies is D15 (Format Freeze Policy) judgment table**: adding unit classes ⇒ incompat; **adding packed record types defaults to incompat**—D15 (Format Freeze Policy) row "adding record type (new key type) ⇒ compat_ro" refers to btree item types, not packed record types: skipping a tombstone record resurrects deleted keys (E59 (Can buffered messages be recomputed solely from units)), skipping inode records leaves read-only mounts idling; neither currently registered type satisfies "skipping changes no read-only answers"; demoted to compat_ro only if proven upon registration, requiring fixed width, key-then-MAC-then-skip when encrypted, never recycled. Type code is AAD field value, not composition. compat_ro readers forbidden to run scan rebuild. New readers encountering unregistered types handle per D15 (Format Freeze Policy) judgment table row 2 (incompat ⇒ unmountable).
- **Settlement across three downstream paths**: tombstones—settled item 8 two sentences split into three (container tag takes code 3; record takes packed record type 1; deceased identified by record's own fields); inode path ①—packed record type 2 reserved, update cost estimates documented in D8 (Core Index Structure) settled item 6; anti-aging—offset 6 is class tag, offset 7 settled as constantly 0 + non-zero rejected; if D26 (Background Compaction and Placement Reclamation) settled item 4 later requires node header bits for tags, offset 7 is unavailable, either opening new field per D15 (Format Freeze Policy) or taking incompat, settled item 11 not settling for D26 (Background Compaction and Placement Reclamation).

<!-- format-const: PACKED_UNIT_HEADER_BYTES = 107 -->
**Numbers** (E102 (Unit class registry and packed record units)): cap(107, 56) = 583 (interval [65, 120]), cap(107, 140) = 233 (interval [9, 148]; header 107 unchanged under settled item 16, record area origin 136, capacity recalculation owed in C89 (residual recalculation of unit header closeout) ⑤).

**Scope**: Value range registry governs "code → class" and judgment landing points for each class; field composition of AAD remains governed by D9 (Encryption) settled item 6 (code 1 five items blind to snapshot dimension, reopening un-executed; code 2 AAD identity lacks landing place, C107 (no landing place for index node AAD identity); code 3 AAD tree dimension takes birth tree, complete contract formed after D9 (Encryption) reopening); relationship between code 3 containers and central mapping follows D19 (Block Pointer Structure and Width Budget) settled items 7 / 8; container index update costs follow respective record types. Registry table code 16–18 freeze moment identical to D9 (Encryption) encryption power-on; codes 1 / 2 / 3 frozen upon first image writeout.

**Basis**:

- E102 (Unit class registry and packed record units): Four lists and tombstones in repo map cleanly under registry; unmapped and multiple-codes-per-name are 0; taking only D9 (Encryption) settled item 6 row as registry leaves tombstone names on unregistered codes (C98 (unit type tag value range has no registration location)).
- E104 (Current version judgment in scan rebuild): Instance table and published predicate—`recovery_own_txns_global_w` world (writing global quantity to every instance in range resurrects recovery instance's own orphans) pins "only selected root instance non-zero W".
- E85 (Field table for unit headers): Five-tuple 33 total originates from its width assumption ("identity fields take 8 bytes"), breakdown never independently argued; four 8s each have independent origins but none established for five-tuple (individual origins in change history).
- E59 (Can buffered messages be recomputed solely from units): Skipping tombstone records resurrects deleted keys—adding packed record types defaults to incompat.
- Three-party defense: Original output of three rounds across nine legs in `research/prompts/d18-item11-r{1,2,3}-*-output.md` and `research/results/d18-item11-r{1,2,3}-local.out`, three proposal versions in `research/prompts/_d18-item11-{unit-class-registry,r2-amended,r3-amended}.md` (round 1 local leg invalidated by word corruption check, round 1 measurement basis was two cloud legs); instance table rules derived from C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement and subsequent user settlements: row-writing and warm-up (C329 (no check for empty publish raising F before row-writing publish) three rounds three parties), intermediate instance rows and switch selected root (C330 (intermediate instance row T_pub taking selected root txg) three rounds three parties), code allocation reading, rollback, and barrier error reporting (C322 (no clause for how to place barrier in code allocation step) three rounds three parties), row width 88 (C304 (instance table chain pointer record cannot fit width-83 pointer)); two categories of disks missing code allocation derived from C113 (no inputs for active version judgment of multi-version units during scan rebuild) round 8 reverse-inference leg (`c113-r8 leg report (artifacts in git)`) 3.1 / 7.2. History and verbatim text in change history.
- User instructions closeout (value range registry, five-tuple breakdown), verbatim text in change history.
- "Sole holder" altered to position hint holder, 1019 / 815 entries, type 1 width 40, and type 5 width 16: tombstone verdict `research/prompts/m3-tomb-r1-main-verification.md` section 3 W1 verbatim sentences, W2 field table rows; `research/prompts/m3-delete-r3-main-verification.md` section 3 G4-lineage residence row (type 5 survived zero rounds of attack).
- User settlement 2026-09-23: Selected root for switch rereads disk for root selection (`research/prompts/c381-r3-main-verification.md` section 4 question 3), verbatim text in change history.
- User settlement 2026-09-24: Instance table rows fill one slice before opening next (verbatim text in change history); no experiments: matches slice count formula max(1, ⌈rows ÷ 369⌉) directly, no measurable cost difference.
- User settlement 2026-09-24: Dry-run on allocator replica checks placements before allocating code; row-writing release verification moved before code allocation; dry-run omitting disk read check noted as owed check (impl 18 report `m2-followups2-implementer-report.md` section 3 1–3), verbatim text in change history; no experiments: written as clauses per 2026-09-24 implementation, dry-run cost unmeasured.
- User settlement 2026-09-26: Administrator rollback changed to forward publish while mounted (D23 (Journal Role and Format) settled item 14); rollback rows (kind 0 record flags bit0) and "rows written by rollback" deleted; old images blocked outside mount by incompat layout identity bits (D15 (Format Freeze Policy) settled item 4), verbatim text in change history; no experiments: three-party verdict `research/prompts/m2-rollback-forward-r3-main-verification.md` section 2 K7 and defense recheck (rollback instance never writes rollback row to self; truncation unreachable is structural fact).
- User settlement 2026-09-27: Post-mount entries ingesting disk tables check own device number and lagging branch after "visible" (code three-party round 2 verdict `research/prompts/m2-closeout-code-r2-main-verification.md` "same-pool old snapshot disk bleached via entry", "swapped disk table" slots), verbatim text in change history; no experiments: implementation and tests in `m2-impl-r2-fixes-b-implementer-report.md` section 3 row 1—`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 10 entries: four entries one each pinning old snapshots, one each pinning swapped disks, one where single self-verified slot causes no false rejections across four entries, one where attacker scans 16 slots; lagging branch trade-off "lagging and missing units" in section 4 item 6 of same document.
- User settlement 2026-09-28: C585 (session publish admitting old snapshot detached during publish) adopts Approach C; user settlement 2026-10-01: missing root half on writable mount retained, clauses supplemented. Verbatim texts in change history. No experiments: Approach C judgment and tests vetted by code three-party rounds m3-code-r1 through m3-code-r3 (verdicts `research/prompts/m3-code-r1-main-verification.md`, `research/prompts/m3-code-r2-main-verification.md`, `research/prompts/m3-code-r3-main-verification.md`).
- Instance table row deletion and whole-round sweeper: three-party verdict `research/prompts/m3-ring-r1-main-verification.md` section 2 row 10 and section 4 item 4 (I-3.8 (instance table rows unique and lower than mounted root) draft condition requires whole-round sweep, repo lacked sweeper; user settled 2026-10-01 to implement immediately); `research/prompts/m3-ring-r2-main-verification.md` section 2 row 10 (row deletion falls on dedicated sweep publish, wiping precedes it, read failure aborts deletion; undeletable half of (j, 0, 0) rows holds); `research/prompts/m3-ring-r3-main-verification.md` section 2 row 9 (successful all-zero root slots not treated as absent roots; each slot self-verified or all-zero otherwise reread once, failing aborts deletion; both tightened in zero rounds). No experiments: numbers on three-party replicas; sweeper was clauses at verdict time with zero code in repo, now implemented (`crates/singlefs-core/src/instance_table_sweep.rs`, triggered at writable mount end).
- Sweeper execution timing, how every placement read, sweep publish reserves and space exhaustion, sweep ordering, row-writing clause exception: verdict `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` section 2 S1 sweep five sentences and section 3 F1, F3, F6 rows; user settlement (2026-10-03, same verdict section 6 Q1, Q3, Q4 rows), verbatim text in change history; toggle-on branch and two halves combination survived zero rounds of attack, ordering and unit recognition measurement basis only tightened; no experiments: numbers on three-party attacker replicas; repo had zero code at verdict, now written (`crates/singlefs-core/src/instance_table_sweep.rs`).
- Sweep accounting skipping roots below active F, reread read failure before wipe aborts row deletion: code round 3 verdict `research/prompts/m3-root-ring-code-r3-main-verification.md` section 2 K1-a, K1-b rows (K1-a: normal unmount raising F on fault-free legal history leaves accounting for old roots below F unreadable, stalling every sweep without deleting rows; K1-b: reread failing before wipe leaves header unwiped while row deleted, stranding orphan after row deletion; measured on replica, main agent re-run matches) and section 6 Q1, Q2 rows; user settlement 2026-10-04, verbatim text in change history. No experiments: numbers measured on attacker replica, no experiment ID established.
- Chain pointer record claiming next slice but position pointer header segment not v1 value also counts as undecodable: code round verdict `research/prompts/m3-batch10o-code-r1-main-verification.md` section 2 K2-1 slot (implementation and checker differed in measurement basis, main agent code inspection confirmed) and section 5 K2-1 row; forks document `m3-batch10o-code-r1-forks.md` K2-1 row; user settlement 2026-10-06, verbatim text in change history. No experiments: measurement basis gap judged from code inspection, no experiment ID established.
- E178 (Leaf container and record death counts): Type 3 retirement records carry death txg, reside in dedicated retirement tree, filtered by lineage (34 bytes per record in model, 959 entries per container); container resurrection and false death judgments are 0 across history of borrowing, absorption, and administrator rollback.
- Type 3 carrier, death txg, and active-following trees selecting versions per P2 only: verdict `research/prompts/m3-leaf-death-r1-main-verification.md` section 2 K1 (form lacking death txg falsely judges death after administrator rollback; carrier undefined); `research/prompts/m3-leaf-death-r2-main-verification.md` section 2 "complete form of candidate 1 (retirement records)" (Approach A); `research/prompts/m3-leaf-death-r3-main-verification.md` section 2; user settlement 2026-10-08 "retirement records · A (recommended)", verbatim text in change history.
- Type 3 retirement record field table (34 bytes), 959 entries per container, residing in retirement tree (kind code 13): verdict `research/prompts/m3-retire-r1-main-verification.md` section 2 K2 "record 34 bytes = identity reference 26 + death txg 8" row (survived two attacker samplings) and section 3 item 2; user settlement "construct upon pool creation (recommended)", verbatim text in change history.
- The missing-units half of the lagging branch checks whether this disk's journal ring slots hold that version's last record: three-party verdicts `research/prompts/m3-close-sysconf-r2-main-verification.md` section 4, `research/prompts/m3-close-sysconf-r3-main-verification.md` section 2 U3 (attacker missed in rounds two and three; the full set of 42 lagging-branch test cases passes under both readings, and a snapshot disk taken while publish units were half-written is still refused, measured on replicas) and section 4 "landing". No experiment: numbers measured on three-party replicas, no experiment number assigned.
- Acquisition written in two rounds, one pool barrier after each round, rollback written only once per disk: system configuration three-party round three verdict `research/prompts/m3-close-sysconf-r3-main-verification.md` section 2 cell A' (acquisition written in two rounds, rollback only once per disk) (attacker missed the two-round write in both rounds two and three; once-per-disk rollback attacked for one round in round three without a hit, measured on replicas) and section 4 "landing"; user settlement 2026-10-19 "cover it, write acquisition in two rounds", 2026-10-20 "keep the two-round acquisition write", verbatim text in changelog. No experiment: numbers measured on three-party replicas, no experiment number assigned.

**Owed checks**: C98 (unit type tag value range has no registration location): gate comparison tooling; C107 (no landing place for index node AAD identity); C108 (extension point attribution has no judgment for packed record units): check turning red; C111 (class tag lacks replica on ciphertext side); C103 (container index lacks landing place); C105 (write amplification of packed containers rewritten per publish frequency); C106 (upper bound on open container count); C109 (which trees' key deaths write tombstones); C113 (no inputs for active version judgment of multi-version units during scan rebuild): check still owed; C597 (instance table rows monotonic without deletion clauses); C690 (default sweep trigger threshold unsettled); C121 (unbounded instance table chain length on zoned); C120 (judgment and resynchronization of split-brain disk return); C329 (no check for empty publish raising F before row-writing publish), C330 (intermediate instance row T_pub taking selected root txg): checks still owed; C89 (residual recalculation of unit header closeout) ⑤: E98 (inode record and inode tree geometry) 91 still owes re-run, E84 (Pinned volume of packed tombstones) harness hardcoded capacity 583 noted in C691 (packed tombstone harness hardcodes capacity to 583) (E83 (Granularity of tombstones) re-run per record area origin 136 and tombstone record 40 bytes, second run 2026-10-04); C833 (writable mount lacks exclusive open): premise of settled item 11 "exclusively open majority of devices in pool" not yet implemented.

#### Settled Item 12: Code 2 Write Sequence Narrowed to 4 Bytes, Vacated 6 Bytes Allocated to Payload CRC and Reserved

**Settlement**: The 6-byte transaction number in the write sequence within code 2 class identity segment is removed, altered to "write sequence 4 (instance code only) + payload CRC 4 + reserved 2"; these three segments still total 10, so settled item 12 itself does not modify header width (code 2 header see settled item 18).

| Who uses which segment of write sequence | Code 1 | Code 2 | Code 3 |
|---|---|---|---|
| Published predicate (settlement P2) | Instance code + transaction number | **Instance code only** | Instance code only |
| Newest-selection key (settlement P3) | Full write sequence | **Does not enter rules** | (birth generation, instance code, transaction number) |
| I-1.8 (total version order after merge) | Covered | **Not covered** | Covered |

**Scope**: The identical rationale holds verbatim for code 3, but changing it would modify C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P3 and I-1.8 (total version order after merge) code 3 total order key; altering key (reverting code 3 newest-selection key to two-tuple) was defeated in round 2 (fixed points iterate, two consecutive checkpoints without allocating new transaction numbers produce identical two-tuple write sequences across two versions, breaking tiebreaker); therefore code 3 keeps 10 bytes unchanged. Code 2 write sequence does not enter P3 because code 2 does not participate in container-level newest selection at all (settled item 11); it does not enter I-1.8 (total version order after merge) because code 2 multi-versions are merged at entry level in accounting trees (D5 (Snapshots / Space Accounting Mechanism) settled item 2), while internal nodes of other trees are derived state rebuilt from leaves (D8 (Core Index Structure) settled item 6). Narrowing code 2 write sequence to 4 bytes vacates 6 bytes precisely discharging code 2's half of C104 (no self-contained evidence of payload integrity for code 1 / 2 on scan path) payload CRC 4, leaving 2 bytes reserved.

**Reserved is only 2 bytes, C102 (no reserved area in index node headers; adding fields is not compat) discharged only half**: C102 (no reserved area in index node headers; adding fields is not compat) self-states flag bits "cannot fit fields" not counted as reserved area; 2 bytes likewise cannot fit most fields—what it purchases is "having a reserved area" itself, not "whatever added later is compat". **Space is not basis**: chopping 6 bytes buys no space on either code 2 or code 3 (code 3 per-container record count zero change; code 2 fanout at most 1 slot extra, tree height unchanged); settlement adopts **exchanging bytes that purchase nothing for bytes where registered owed checks are waiting**, not saving space. Two overturned criteria forbidden to reuse: "zero consumers today ⇒ burden of proof on retaining side" (sweep-back would chop settled item 7 nonce and MAC reserved bits, D19 (Block Pointer Structure and Width Budget) settled item 3 MAC and nonce constant placeholders, and D8 (Core Index Structure) settled item 8 reserved 32 bytes; what holds is the other rule: reserved bits proof is "future use", transaction number proof is "purchases nothing today", two differing in form); "accounting tree contains quantities uncomputable by traversal ⇒ code 2 requires version selection" (multi-version accounting rows merged at **entry level**, not version selection at container level—D5 (Snapshots / Space Accounting Mechanism) settled item 2 verbatim "non-monotonic statistics 'latter wins' (latter determined by seq), monotonic statistics take maximum"). E73 (Fanout cost of node key range) / E97 (Entry encoding for accounting and allocation records) / E98 (inode record and inode tree geometry) / E103 (Update cost of inode packing path 1) numbers computed under old three-tier header widths belong to C89 (residual recalculation of unit header closeout) item 4 and C288 (collateral changes from mapped key settlement did not land in other decisions) item 7 recalculation.

**Basis**:

- E104 (Current version judgment in scan rebuild): Its sole purchased purpose cannot occur under the rules' premises—source code self-states (`e104_current_version.rs` `rebuild_root`): "transaction number of write sequence is maximum transaction number allocated in that checkpoint; code 2 is written only at checkpoint fixed points, so within the same checkpoint code 2 has at most one version per node id" ⇒ transaction number on code 2 never acts as tiebreaker.
- Three-party defense: Verbatim output in `research/prompts/d18-item12-r1-{sonnet,opus}-output.md`, `d18-item12-r2-{opus-defense,sonnet-rebuttal}-output.md`; reverse-inference leg verified payload CRC 4 + reserved 2 consumes precisely 6 bytes without altering 10-byte total width.

**Owed checks**: C163 (fixed-point iterations sharing write sequence nullifies tiebreaker): check turning red; C102 (no reserved area in index node headers; adding fields is not compat): remaining half (whether to widen before freeze).

#### Settled Item 13: When Encryption Enabled, Code 1 Identity Segment and Declared Length Both Enter Ciphertext

**Settlement**: Takes arm G—when encryption is enabled, code 1 five-tuple / birth generation / write sequence **become ciphertext in-place**, and `declared length` is removed from common plaintext prefix shared across three classes, entering ciphertext together. Field positions and widths are untouched; plaintext header total width remains 105; code 1 plaintext allowlist narrows to `{fsid, payload CRC}`.

**Scope**: **Leakage cannot be eliminated to zero; what arms differ in is order of magnitude, not presence vs absence.** Arm G eliminates the heaviest item (`declared length` ⇒ exact small file size), incidentally eliminating code 3 `declared length` ÷ public record width (I-1.7 (packed container validity and judgment order) ⇒ exact record count of small packed container). Items remaining in plaintext:

| Still in Plaintext | What Leaks | Verbatim Evidence |
|---|---|---|
| **Unit type tag** (flags offset 6) | Count of data units vs metadata units | D18 (Information Carried in a Block) settled item 7 verbatim: "scanner must distinguish class before decryption to know which allowlist applies" ⇒ structurally cannot enter ciphertext (A2) |
| **Payload CRC 4** | Equality ⟺ same ciphertext ⇒ replica count `w` (v1 `w` constantly 2, D2 (RAID Stripe Policy) settled item 9 ⇒ no increment in v1) | I-6.2 (plaintext header field allowlist) allowlist note: "equal payload CRC identifying replicas is knowingly accepted" |

Tree table length is not in plaintext: tree table resides in tree table units in root record indirect layer, and tree table units are ciphertext (D22 (How Unit Atomicity is Synthesized) settled items 7 / 12). Cost of G: it provides unit type tag **an additional non-keyed consumer**, turning "hiding class tags too" into an incompat format change; accepted.

**Basis**:

- E88 (Distinguishing imposter units and orphan units): In its source code `declared length` / `decl` appears 0 times; steps by whole unit after claiming ⇒ replacing claiming rule has zero cost, re-run unnecessary.
- Code 3 judgment order zero increment: Among seven steps of I-1.7 (packed container validity and judgment order), `class identity segment` ranks 2nd, `boundary` (including declared length) ranks 5th; code 3 class identity segment was already metadata class ciphertext in its entirety ⇒ step 5 already requires decryption key; moving declared length into ciphertext introduces no new key-holding requirement.
- Zero loss of non-keyed scrub capability: D9 (Encryption) settled item 5 commitment to "detect corruption, report bad blocks" is fulfilled by header checksum + payload CRC + nonce code; none consumes `declared length`; corruption in padding area is caught equally—payload CRC covers from header end to unit end (including padding).
- E28 (Can plaintext mapping layer be rebuilt solely from unit self-description): Arm D (omitting identity segment when encryption enabled) deficit = total data block count, without smaller leakage surface—eliminated.
- User settlement "block header leakage is unacceptable" (basis for opening this item) and selecting Arm G, verbatim text in change history; defense when D18 (Information Carried in a Block) settled item 3 kept data unit identity in plaintext was "same information already lies in plaintext reverse index", while D9 (Encryption) settled item 5 subsequently canceled plaintext reverse index, causing defense foundation to collapse.
- Two rounds of three-party defense: history in [records/2026-09-07-D18项3重开第一轮.md](../../../records/2026-09-07-D18项3重开第一轮.md) and [records/2026-09-07-D18项13第二轮.md](../../../records/2026-09-07-D18项13第二轮.md).
- 13-c Payload CRC covers on-disk bytes: forward-inference and reverse-inference legs independently found repo blank prior to settlement; two independent reasons documented in settled item 7; user settlement, recorded in change history.

**Owed checks**: None.

#### Settled Item 14: Nonce Code Is Full 96-Bit Nonce, Residing in Unit Plaintext Header

**Settlement**: **Takes arm A**: nonce code is the full 96-bit nonce (12 bytes), residing in unit plaintext header, pointer copy retained. Code 2 / code 3 reserved bits (29 bytes, settled item 16) **immediately follow entire class identity segment**; code 1 reserved bits reside at offset 105 (settled item 18).

**Scope**: Rebuild paths have no pointers, while after settled item 13 identity resides in ciphertext ⇒ without full nonce on unit itself decryption is impossible; this fills that gap. Two blanks remain in this slot: I-6.1 (nonce not reused) note "nonce code allocation rule unsettled" and I-6.5 (rebuild does not reuse nonce) "nonce watermark after rebuild"; neither settled here; settled item 14 settles only width (12 bytes) and residence (unit plaintext header). C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P1 write sequence occupies 10 bytes; together with C288 (collateral changes from mapped key settlement did not land in other decisions) item 1 birth sequence number 4 bytes, header width increases by 4 bytes over E117 (header width and geometry with reserved bits included) (C298 (header width plus 4 modified numbers in user settlement)).

**Basis**:

- E117 (Header width and geometry with reserved bits included): Sizing reserved bits into headers drops fanout by 0–1 slots each, code 3 records per container drops 0 slots; four downstream experiments re-run same day, zero conclusions overturned.
- One round three-party defense: history in [records/2026-09-07-D18项14第一轮.md](../../../records/2026-09-07-D18项14第一轮.md); other three arms eliminated—whitening (arm C) purchases nothing (same image already has five structural order/grouping leakages, whitening touches none of them) and kills I-6.5 (nonce watermark monotonic); deleting pointer copy (arm B) perturbs at least 7 registered numbers and turns I-6.7 (replicas share nonce) from structurally automatic into a new bit-by-bit comparison check while buying only space; rebuild without decryption (arm D) hits C113 (no inputs for active version judgment of multi-version units during scan rebuild) settlement P3 where newest-selection keys are entirely in ciphertext.
- User settlement ("take arm A, but calculate header width clearly before settling"), verbatim text in change history; after adding 4 bytes to header width original settlement holds under new numbers (C298 (header width plus 4 modified numbers in user settlement)).

**Owed checks**: C184 (rebuild path cannot acquire full nonce): gate comparison.

#### Settled Item 15: Withdrawn · Number No Longer Used

**Settlement**: Withdrawn, this number is no longer used. Original topic "version selection rules for code 2": whether code 2 needs, and with what inputs (existing birth generation, instance code, key range, or newly added fields), to determine "which on-disk version of code 2 node root R (txg T) references when multiple versions of code 2 exist".

**Scope**: Conclusion across three rounds: two literal candidates (key range taking fence semantics, node number plus retirement records) both silently select stale versions when "active version of parent node and child items are lost together while stale version remains on disk"; the sole form avoiding false answers (fences plus acceptance gate) is undecidable when referenced by snapshots; and D11 (Whether to Reserve Message Buffer in Index Nodes) and D21 (Boundary Between Authoritative and Derived States) already established that code 2 does not require rebuild (reconstructed from leaves or merged at entry level). Therefore code 2 needs no container-level version selection rules, withdrawn.

**Basis**:

- No experiments: Withdrawn item is an unadopted rule; verdicts from three rounds of defense represent evaluations of clauses and models, with no numbers adopted.
- Three rounds across three parties: verification and verdicts in `research/prompts/d18-item15-r1-main-verification.md`, `research/prompts/d18-item15-r2-main-verification.md`, `research/prompts/d18-item15-r3-main-verification.md`.

**Owed checks**: C292 (rationale for zero consumers of code 2 identity segment no longer holds): reconciliation gate.

#### Settled Item 16: Nonce / MAC / Algorithm Type Reserved Bits Counted in Header, Declared Length Origin Is End of Header Including Reserved Bits

**Settlement**: The **29-byte** reserved bits (nonce 12 + MAC 16 + algorithm type 1) for all three unit classes are counted into headers immediately following class identity segments: code 1 header **134**, code 3 header **136**, code 2 header **115 + 2 × key width**; declared length origin is the **end of header including reserved bits**.

**Scope**: What is placed inside reserved bits and why nonce is a full 96 bits belongs to settled item 14; full offsets for code 2 belong to settled item 18; the 29 bytes are outside header checksum coverage (I-2.4 (header checksum coverage)).

**Basis**:

- E117 (Header width and geometry with reserved bits included): Header width and fanout costs including reserved bits have been measured.
- E102 (Unit class registry and packed record units): Second run reproduced cap(136, 40) = ⌊32632 / 40⌋ = 815.
- Settled item 14 settled nonce residing in unit plaintext header; C288 (collateral changes from mapped key settlement did not land in other decisions) item ① settled it immediately following birth sequence number; what was missing was merely "whether counted into header and where declared length originates".
- User settled item by item per options in [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) section 9 (modal prompt, main agent recorded); algorithm type 1 byte settled in separate modal prompt; neither went through three-party defense; main agent recommendation and user choice both overturnable. Verbatim text in change history.

**Owed checks**: None.

#### Settled Item 17: 32-Byte Header Checksum Field Holds CRC32C 4 Bytes + 28 Bytes Zero; Self-Verifying Structure Checksum Covers Entire Slot Including Padding

**Settlement**: 32-byte header checksum field stores CRC32C 4 bytes + 28 bytes zero; root record self-verifying checksum, system configuration whole-slot checksum, and journal `header_csum` share identical measurement basis. Checksum of self-verifying structures (root record, system configuration slot, journal record header) covers entire slot including padding, checksum field itself participating as zero; padding is constantly 0. **Hardcoded parameters**: CRC32C = CRC-32C, Castagnoli polynomial 0x1EDC6F41, reflected implementation (input and output reflected), initial value 0xFFFFFFFF, output inverted; result written in little-endian into **first 4 bytes** of 32-byte field, remaining 28 bytes constantly 0; readers encountering non-zero unconditionally judge structure corrupt. Every CRC32C warehouse-wide (header checksum, payload CRC, 4 bytes in each position entry, self-verifying structure checksums) shares identical parameters, linked here from elsewhere. **Checksum algorithm identifier enters system configuration**: bootstrap header segment adds 1 byte "checksum algorithm identifier", registry 0 = invalid, 1 = CRC-32C, v1 writes 1 (D22 (How Unit Atomicity is Synthesized) settled item 9); mounting selects algorithm per this byte, sharing registration location with 4-byte checksums in unit headers. Modifies bytes of new-pool file creation: **Yes**—value of every checksum field and position of the 4 bytes within field.

**Scope**: The 28 bytes participate as 0 in own checksum (I-2.4 (header checksum coverage)); what protects it is "constantly 0, non-zero judged corrupt", not another checksum. Algorithm adopts standard used by iSCSI / SCTP / btrfs / ext4 (directly computed by `crc32c` hardware instruction), lacking alternative prevailing choice.

**Basis**:

- E144 (Cost and discriminative power of header checksum algorithms): Six candidates exhibited zero leakage across single-bit, double-bit, and random byte flips on headers; SHA-256 far more expensive than CRC32C, though absolute magnitude is a fraction of single fsync; CRC32C is already settled in repository. User guidance "cheaper if unrelated to encryption", verbatim text in change history.
- D13 (Verification Route) settled item 5 requires checker and implementation to write independent implementations, while D13 (Verification Route) itself states verbatim "premise for writing separate CRC32C implementations is pinning parameters (polynomial, initial value, reflection, XOR output): without pinned parameters, two independent implementations yield differing results on healthy images, producing false red rather than independence"—leaving four parameters blank fails to purchase that independence.
- User settled item by item per options in [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) section 9 (modal prompt); parameters completed in general audit round 2; algorithm identifier entering system configuration settled in separate modal prompt; none went through three-party defense, overturnable. Verbatim text in change history.

**Owed checks**: C309 (32-byte checksum algorithm unsettled): independent implementation on checker side and comparison against "implementation and checker using same function".

#### Settled Item 18: Code 2 Declared Length = Entry Count × Entry Width, Padding Area Always 0, Code 1 / Code 2 Each Supplemented with Offset Field Table

**Settlement**: Code 2 declared length = entry count × entry width; padding area following entry area is constantly 0; I-2.3 (padding bytes zero and participate in checksum) scope extended to code 2. Code 1 and code 2 each supplemented with field tables with offsets sharing shape with code 3—

| Class | Field Table with Offsets |
|---|---|
| Code 1 | 42 tag replica / 43 tree ID / 51 object ID / 59 object birth generation / 67 anchor offset / 75 birth generation / 83 fsid / 91 write sequence / 101 payload CRC / 105 reserved bits 29 ⇒ plaintext header 105, 134 with reserved bits |
| Code 2 | 42 tree ID / 50 level / **51 key width 1** / 52 key range 2k <!-- format-const: INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT = 2 --> / 52 + 2k birth generation / 60 + 2k fsid / 68 + 2k write sequence 4 / 72 + 2k birth sequence number / 76 + 2k payload CRC / 80 + 2k reserved 2 / 82 + 2k entry count 2 / 84 + 2k entry width 2 / 86 + 2k reserved bits 29 ⇒ plaintext header 86 + 2k <!-- format-const: INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86 -->, 115 + 2k with reserved bits |
| Code 3 | Plaintext header 107, 136 with reserved bits (reserved bits 29 same measurement basis as other two classes; record area starts at 136, declared length upper bound 32768 − 136) |

- **Key width field at fixed offset 51**, preceding key range: scan rebuild paths have neither parent pointers nor tree tables; decoding a code 2 node requires knowing `k` before knowing how many bytes key range occupies; readers decode `k` from `[0, 51)` fixed-length prefix, then determine entire header via `86 + 2k`, decoding in one pass.
- **Code 2 payload CRC covers `[86 + 2 × key width, 16384)`**, originating at **end of plaintext header**, covering 29-byte reserved bits, entry area, and subsequent padding; shares measurement basis with code 1's 105 and code 3's 107 (all three originate at end of plaintext header, not end of header with reserved bits). Entry area origin is a different number—end of header with reserved bits `115 + 2 × key width` (D8 (Core Index Structure) settled item 11), difference being precisely those 29 bytes.
- Settled item 11 registry table code 2 row supplemented with birth sequence number / payload CRC / reserved / self-describing three segments. Full offsets for code 2 registered solely here; settled item 7 index node section writes field set only, not duplicating numbers.
- Modifies bytes of new-pool file creation: **Yes**—entry area origins, declared lengths, payload CRCs, key width field landing places, and header widths for six code 2 units (`115 + 2k`: inode tree root 131, extent 163, allocation record 135, accounting 159, mapping 169, tree table 131).

**Scope**: Padding area constantly 0 and I-2.3 (padding bytes zero and participate in checksum) scope extrapolated from existing measurement basis of code 1 / code 3, with no alternative choice. Code 2 header width formula `86 + 2k` holding for all trees rests on settled item 2 (all code 2 nodes carry key ranges).

**Basis**:

- E145 (Cost of code 2 self-describing header and mapping tree key width): Self-describing three segments add 5 bytes per node without dropping fanout or tree height by a single slot; non-self-describing headers probing widths sequentially decode registered trees but cannot decode unregistered trees. User guidance "code 2 can be self-describing", verbatim text in change history.
- Moving key width to offset 51: Account opened by C325 (locating code 2 key width field requires knowing k beforehand) (placing key width at `81 + 2k` makes its own offset dependent on `k` ⇒ readers forced to brute-force trial decode `k`).
- User settled item by item per options in [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) section 9 (modal prompt, main agent recorded); two offset table items (key width relocation, reserved bits expanded to 29) settled in separate modal prompt; origin of code 2 payload CRC completed in general audit round 2; none went through three-party defense; main agent recommendation and user choice both overturnable. Verbatim text in change history.

**Owed checks**: None.

#### Settled Item 19: Whether Field Is Worth Carrying into Block: Three Criteria

**Settlement**: **Criteria are settled; three superimposed, none dispensable.** Whether a field is worth carrying into a block:

| # | Question | Consequence of Failure |
|---|---|---|
| **1 (Necessary)** | **Is its update cycle bound to content changes?** Bound = changes only when content changes | Decoupled fields independently trigger full H-level chain COW on every change (under COW address changes anyway, rewriting to root; this is a free ride; update decoupled from content pays full cost for zero payload change) |
| **2 (Necessary)** | **Lacking it, which specific recovery/judgment turns from "decidable" to "undecidable"?** | Inability to answer constitutes permanent format debt, sample see D18 (Information Carried in a Block) settled item 19 "Negative Example" |
| **3 (Necessary)** | **How much net gain relative to existing mechanisms? Decidable to which side?** | Fields overlapping with existing mechanisms approach zero marginal benefit; "recoverable with key" and "recoverable without key" are distinct propositions |

- **Cost falls on each record, not on block headers**: overhead at block header level viewed as ratio of "header / node" is free, whereas adding 1 byte to record formats may force every record to pay an entire extra slot (1 extra byte in bcachefs bkey header ⇒ extent records jump from 40 to 48 bytes, dropping node capacity by 16.7%).
- **Negative example: a field purchasing zero failure mode improvement**—btrfs `chunk_tree_uuid`, 16 bytes/block: written in one place, verified during scrub in one place, **self-referentially** reads expected value during mount; **never read on normal read paths, never checked during mount**. Lacking it, not a single failure mode shifts from detectable to undetectable.

**Scope**: ⚠️ **Criterion 2 is not `.claude/rules/fs-design.md` "turning misuse into unmountable"**—that rule is a mount-time binary determination designed for **format branches**; carried information is runtime evidence serving data recovery and corruption localization. Two candidates that passed the three criteria:

| Candidate | Bound to Which Failure | Status |
|---|---|---|
| **Commit point identity = (counter, commit point hash)**, not just counter | **Monotonic counters repeat after rollback / fsck**; after repetition, leftover blocks from previous timeline impersonate valid blocks (f2fs embeds checkpoint pack hash into NAT / SIT headers for this reason) | Covered by instance table (settled item 11) |
| **"I am a migrated replica" marker + replica-exclusive data CRC** | UBI first-party admission: when two PEBs correspond to same LEB, **counter alone is insufficient**; the newer one might have lost power mid-migration—"this is just an uncommitted copy" | Not needed in v1 (D2 (RAID Stripe Policy) settled item 1 full-stripe writes) |

**Basis**:

- No experiments: Three criteria are field admission rules, respective bases are facts from production implementations—block header level overhead (btrfs `btrfs_header`, bcachefs `btree_node`) is free by ratio; bcachefs bkey demonstrates record-level inflation; btrfs `chunk_tree_uuid` verified via `fs/btrfs/disk-io.c` and `fs/btrfs/volumes.c`.

**Owed checks**: C18 (decoupled fields polluting read-only dimensions): red-turning form of criterion 1.

#### Settled Item 20: Settled Identity Fields to Carry

**Settlement**: Three items passing all three criteria and settled to be carried:

| Field | Which Judgment Undecidable Lacking It | Placement |
|---|---|---|
| **Own logical address + fsid** | Misdirected writes (ghost write) **undetectable**; block number alone insufficient: misdirected write may land on **correct block number of another file system**, so location information **must contain file system identifier** | Data unit five-tuple (settled item 3); I-1.1 (block header self-describes logical address) / I-1.4 (block header fsid consistent) |
| **generation (birth generation)** | **Stale blocks left by lost writes are self-consistent**: comparing block generation with expected generation in parent pointer is the sole way to distinguish "never written" from "written to wrong place" | I-1.2 (block header write sequence published) |
| **Owning tree ID (birth tree)** | Orphan blocks **cannot even determine corruption scope** | I-1.3 (block header tree ID consistent) |

**Scope**: "Own logical address" in data unit headers is expressed by five-tuple (anchor offset is its logical address, I-1.1 (block header self-describes logical address)), containing no physical placement (two lines of settled item 3); landing places across unit headers see settled items 7 / 11 / 18.

**Basis**:

- E88 (Distinguishing imposter units and orphan units): Headers lacking fsid and birth generation allow imposter units into Level 2, failing to clean any torn orphans.
- Production implementation first-party statements: btrfs "the measures have to be done ahead of time" and `fs/btrfs/disk-io.c` "This is how we detect silent data corruption and misdirected writes".

**Owed checks**: None.

#### Settled Item 21: Scope of Self-Description Is Misdirected Writes, Not Lost Writes

**Settlement**: **Lost writes (lost write / phantom write)**—device reports write success but actually did not write—leaves stale blocks that are **completely self-consistent at the intra-block level**: old content, old checksum, correct own logical address, correct fsid, correct owning tree ID. **Self-describing information carried in blocks cannot catch lost writes under any configuration**; catching them fundamentally requires **witnesses outside the block**: expected generation and checksum in parent pointer, recurring to root (D4 (Checksum Position) Merkle tree). Settled item 20's "generation" guards against lost writes precisely because parent pointers record expected values; **the block's own header alone cannot detect it**.

**Scope**: Distinguishes the boundary between self-description and parent pointers: self-description catches misdirected writes; parent pointers catch lost writes; neither replaces the other. D16 (Publish Semantics) checkpoint commit order ensures parent pointers land after child blocks, closing the loop.

**Basis**:

- No experiments: Definitional—valid old versions and valid new versions are **isomorphic at the intra-block level**; three counterexample paths all fail: wall-clock timestamps (old blocks **legitimately** carry old timestamps), incrementing local counters, and version history hash chains ("expected count", "expected chain length", and "expected hash" all reside outside the block).
- OSTEP §45.6 verbatim (local PDF extraction recheck): "Unfortunately, the answer is no: the old block likely has a matching checksum... Thus, checksums alone are not enough to detect lost writes; the file system needs additional machinery...".
- XFS `xfs-online-fsck-design.rst:945-946` claims ondisk addresses carried in blocks "enables the filesystem to detect lost writes"—**factually incorrect, confused with misdirected writes**; confirmed via `fs/xfs/libxfs/xfs_types.h`.

**Owed checks**: C7 (stale block lacks check).

#### Settled Item 22: Information Supporting Scan Rebuild Must Be Written Explicitly, Not Folded into Checksum Seed

**Settlement**: **Any information required to support "scan rebuild" must be written down explicitly; only information that "needs misapplication detection without rebuild" may be folded into seeds.** Folding into seed (ext4 folds `(fs uuid, i_ino, i_generation)` into checksum seed) creates a **one-way check**: readers holding expected identity can verify block correctness, but scanners picking up an isolated block without indices **cannot reverse-solve identity from checksum**.

**Scope**: Settles whether identity information is written as explicit fields or folded into seeds; whether an explicit field is worth carrying still passes the three criteria of settled item 19.

**Basis**:

- No experiments: Whether folding into seeds can reverse-solve identity is definitional (checksums can only verify a given guess), no measurable quantities; ext4 (`fs/ext4/ialloc.c` containing `__le32 inum = cpu_to_le32(ino); csum = ext4_chksum(sbi, csum, (__u8 *)&inum, sizeof(inum));`) verifies it serves purely for validation, unable to support offline inode identification.

**Owed checks**: None.

#### Settled Item 23: Rejection List

**Settlement**: The following candidates are not carried into blocks:

| Candidate | Rejection Reason |
|---|---|
| **Child block recording "current" parent node physical address** | **Temporally impossible**: COW writes child first, parent second—the moment child block writes, what it wants to record does not exist yet. Changing to backfilling violates "write once without writeback". This is logical contradiction, not engineering hassle |
| **Checkpoint inlining "N blocks in this checkpoint"** | **Same temporal deadlock**: N is known only when checkpoint concludes, whereas the field must embed into every block written during the period |
| **COW predecessor address (version chain)** | **Negative net value**: ① Functionally overlaps with D5 (Snapshots / Space Accounting Mechanism) birth txg + deadlist; ② After predecessor block is physically reused, that pointer becomes a **misleading pointer**—following it sees **another object's current content**; ③ It requires adding a new invariant to guard against the pitfall it creates itself |
| **Intra-block error correction code (ECC)** | **Rejected, reason being "neither end economical, no sweet spot in between"** (not "too expensive"). ① Cheap tier: Hamming 16-bit corrects 1 bit, overhead on 4 KiB block is **0.05%**, **but it only corrects the class already fixed by device ECC**; expensive tier: RS correcting an entire 512-byte sector takes 12.5%, correcting a half-block lost to torn write takes 50%; whereas in the two classes device ECC cannot cover, torn write takes 50% parity, and **misdirected write means the entire block is someone else's data with zero redundancy available**. ② **The sole instance across all examined systems is OCFS2 `metaecc`** (`struct ocfs2_block_check` in `fs/ocfs2/ocfs2_fs.h`: 32-bit CRC32 + 16-bit Hamming, `ocfs2_hamming_fix_block` in `blockcheck.c` correcting 1 bit before reverifying CRC), **correcting only single-bit errors and disabled by default**, with its design doc explaining the motivation as "Modifying it to this level of safety isn't really possible **without creating an entirely new filesystem**"—**this motivation does not hold for this project designed from scratch**. ③ Checking 13 other systems (including IRON, CrookFS, NOVA-Fortis, EnvyFS), **intra-block ECC has zero hits**; NOVA-Fortis explicitly evaluated ECC and then rejected it |
| **Per-block wall-clock time** | Checked across four block header implementations, **none carry it**, all using `generation` / `txg`—monotonic, comparable, naturally defined commit points under COW. Wall-clock appears once in ZFS uberblock, **one copy per pool** |

**Scope**: Rejection targets these five field forms. Same-batch chains (prev/next = neighbors in allocation order among batch written in same write) do not hit rejection list (allocation precedes writing, no temporal inversion); whether it is worth carrying judged separately by settled item 4 (eliminated).

**Basis**:

- No experiments: First two are temporal contradictions (child writes before parent; N known only at conclusion); COW predecessor address overlaps with D5 (Snapshots / Space Accounting Mechanism) mechanisms and becomes misleading pointer upon reuse; intra-block ECC and wall-clock times originate from production surveys; citations in rejection reasons and change history, unverified in this project.

**Owed checks**: None.

#### Settled Item 24: V1 Tombstones: Tombstone Tree, 40-Byte Records, Lineage Records

**Settlement**:

1. **Tombstone tree**: a dedicated tree, day-1 zero-root registered into tree table; root constantly code 2, entries follow inode tree internal node 120-byte form (separator key container number 8 + identity reference 26 + child pointer 86), children are code 3 containers (packed record type 1 tombstones); containers and root enter central mapping, pointers in entries are position hints; root grows in height when full per D8 (Core Index Structure) settled item 6, implementation must perform this step (user settlement K-R3-3); container numbers start from 1, next number = max number in tree + 1, 1 at zero-root (sharing rule with settled item 11 clone head type 1 container numbers allocating monotonically from 1 within tree).
2. **Tombstone record**: object ID 8 + object birth generation 8 + range start 8 + range end 8 + death txg 8 = 40 bytes; whole object writes [0, 2⁶⁴ − 1]; one 32 KiB container holds ⌊(32768 − 136) ÷ 40⌋ = 815 entries; death txg = checkpoint_txg of intent publish.
3. **Open container**: rewritten only on publish with new records, copied verbatim on un-deleted publishes; closed only when full; closing path adds test-only capacity knob, knob unpersisted to disk.
4. **Reclamation**: v1 does not reclaim tombstones; statistics 13, 14 (D5 (Snapshots / Space Accounting Mechanism) settled item 4) unregistered in v1; 10⁶ unlinks produce ~76.7 MiB tombstones (two copies).
5. **Lineage record**: a dedicated lineage tree, day-1 zero-root registered, follows active version, append-only; children are code 3 containers (packed record type 5, not recycling type 3 registered in settled item 11); record = T_rb 8 + T_old 8 + H_cur 8 = 24 bytes (T_rb is txg of rollback publish, T_old is txg of R_old, read as "parent of T_rb is T_old"; H_cur is cur (active version prior to rollback) inode number watermark, read from cur version, not dynamically computed from "previous version" handed by publish path; byte order of three segments is T_rb 8 | T_old 8 | H_cur 8, H_cur appended after existing two segments, old segment offsets untouched); one 32 KiB container holds ⌊(32768 − 136) ÷ 24⌋ = 1359 entries; administrator rollback publish appends one entry; lineage tree root grows in height when full, container numbers start from 1, next number = max in tree + 1, 1 at zero-root. H_cur cross-checks with step rows of inode number watermark in accounting (D5 (Snapshots / Space Accounting Mechanism) settled item 2 scope): on each lineage record, maximum watermark among step rows with generation segment ≤ T_rb − 1 **equals** H_cur; H_cur of 0 also judged red (W3 C2aL form; established as I-9.30 (step rows of inode number watermark match watermark, records, and lineage), implemented). When clone heads exist, this check and step rows partition by head (carrying tree ID): step rows partition by (statistic, tree), allocating row for new head when creating clone; byte format of lineage records by head unsettled (inferred, v1 single head).
6. **Registration**: tree table 7 → 12 entries (`deleted_inodes`, tombstone tree, lineage tree, logged_ops, retirement tree zero-root; logged_ops see D8 (Core Index Structure) settled item 5, retirement tree see settled item 11 packed record type 3), tree ID watermark 65544 → 65549 (inferred: five trees occupy one tree ID each; tree IDs allocated starting from 65536, D8 (Core Index Structure) settled item 11); adding no new incompat bits, merging into first layout line.

**Scope**: V1 single head, no snapshots, no sweep. How scan rebuild reads tombstone and lineage records unsettled here, noted as owed check (K-W5b, grouped with C113 (no inputs for active version judgment of multi-version units during scan rebuild)). Lineage tree carrier and packed record type 5 survived zero rounds of attack; record 24-byte form with H_cur survived one round of attack (W3 round 3 two samplings).

**Basis**:

- E83 (Granularity of tombstones): Second run (tombstone record 40 bytes, record area origin 136) reproduced item 2 815 entries per container and item 4 10⁶ unlinks ~76.7 MiB (two copies).
- E102 (Unit class registry and packed record units): Second run cap(136, 40) = 815, matching item 2 815.
- Remaining numbers for tombstone tree and lineage tree: measured on three-party leg models and replica prototypes (tombstone verdict, m3-delete-r3 verdict), unmeasured on repo harness.
- Tombstone tree: tombstone verdict `research/prompts/m3-tomb-r1-main-verification.md` section 3 W1 carrier row (two rounds; `research/prompts/m3-delete-r3-main-verification.md` section 3 "tombstone round 1 rows" deleted type 3 retirement from children); root full growth: user settlement K-R3-3.
- Tombstone record: tombstone verdict section 3 W2 field table, W2 death txg rows; death txg followed user settlement K-W5, m3-delete-r3 verdict section 3 "tombstone round 1 rows".
- Open container: tombstone verdict section 3 W4 open container row (two rounds); knob unpersisted: m3-delete-r3 verdict section 3 G6-capacity knob row (zero rounds).
- Reclamation: tombstone verdict section 3 W3 reclamation row (two rounds); 76.7 MiB is m3-delete-r3 verdict section 3 G8-remaining numbers row (40 bytes, 815 entries per container).
- Lineage record: m3-delete-r3 verdict section 3 G4-lineage residence row (zero rounds).
- Lineage record carrying H_cur, judged by "equality", read from cur, by-head with clones: W3 verdict `research/prompts/m3-w3-r2-main-verification.md` section 2 attacker K3 (comparing with "covered" missed two lineage miswrite mutations; changing to "equality" turned all red; dynamically computing H_cur from publish-supplied "previous version" allowed single-point mutation to leak all 26,410 times across two strongest forms) and section 3 C2aL row; `research/prompts/m3-w3-r3-main-verification.md` section 2 clone head rows (unpartitioned reading falsely reported across two samplings; all three forms must partition by head); user settlement 2026-10-08 Y2 "C2aL: accounting reserves step rows (recommended)", verbatim in change history. No experiments: numbers measured on three-party leg models and repo replica prototypes, no experiment ID established.
- Registration: tombstone verdict section 3 registration row (zero rounds; watermark 19 → 21 when two trees, tree ID numbering at that verdict; tree IDs now allocated starting from 65536).
- Registration adding retirement tree (tree table 12 entries, watermark 65549): verdict `research/prompts/m3-retire-r1-main-verification.md` section 3 item 1 and section 5; user settlement "construct upon pool creation (recommended)", verbatim in change history.
- Lineage record three-segment byte order T_rb | T_old | H_cur: chosen by batch 49 W3 implementation writer (implementation writer report section 2 "lineage record three-segment order" row, `m3-batch49-w3-implementer-report.md`), adopted by main agent, zero rounds. No experiments: order is byte arrangement choice, no measurable quantities.
- User settlements K-U2 "t1: intent writes whole-object tombstone", K-W5 "lineage record", K-W5b "write side lands with lineage, read side records owed check", K-R3-3 "write root grows in height (recommended)", verbatim in change history.
- Container numbers starting from 1, tombstone tree and lineage tree roots growing in height when full: `research/prompts/m3-blanks-r1-main-verification.md` section 2 container number, tombstone tree root growth rows (one round).
- Tree IDs allocated starting from 65536: user settlement 2026-10-01 "Option C: shift tree IDs as a whole", verbatim in change history.

**Owed checks**: C615 (scan rebuild has no input for rollback while mounted), C616 (lineage parent chain relying on txg sequence lacks assertion), C617 (tombstones left by delete-then-rollback lack prepaid coverage), C614 (rollback selecting wrong tombstone tree and post-deletion single image without tombstones undecidable); C84 (tombstone unit granularity unassigned) discharged, C103 (container index lacks landing place) type 1 half discharged, C109 (which trees' key deaths write tombstones) narrowed.

```

**出处 `.claude/kb/decisions/18-information-carried-in-a-block.en.md:610-613`（整段抄，未转述）**

```markdown
## Historical Versions

D18 (Information Carried in a Block) historical entries are consolidated in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/18-ブロックに含める情報.ja.md:1-6`（整段抄，未转述）**

```markdown
## D18 (ブロックに含める情報) —— 確定

[中文](18-块里携带什么信息.md) · [English](18-information-carried-in-a-block.en.md) · **日本語**

D18 (ブロックに含める情報) はユニット (ブロック) 内にどの情報を含めるかを管轄する：あるフィールドをブロックに含める価値があるかの判定基準 (確定項目 19)、含めることが確定したフィールドおよび否決されたフィールド (確定項目 20、確定項目 23)、各ユニットヘッダのフィールド表とオフセット、ユニットタイプタグの値域およびパックレコードタイプの登録表 (インスタンス表を含む)、ヘッダチェックサムのアルゴリズムと nonce / MAC / アルゴリズムタイプ予約領域、ならびにブロックヘッダに基づいて走査再構築がどの段階まで達成可能か。管轄外：「このブロックはまだ生存しているか」は**集合に関する問い**であり、ブロック自身が答えることは永久に不可能である——これはフォーマットの不備ではなく、D5 (スナップショット / 領域会計メカニズム) がそれを大域的属性と確定したためであり、まさにそれが O(1) の源泉である；AAD のフィールド構成は D9 (暗号化) 確定項目 6 に帰属する。他決定との連動：データユニット自己記述は D9 (暗号化) 確定項目 5 の漏洩面トレードオフと直接連動する (確定項目 3、確定項目 13)；ノード内パリティはマルチディスクにおける D2 (RAID ストライプポリシー) の可変全幅ストライプと一部重複するが、**単一ディスクは D2 (RAID ストライプポリシー) の手が届かない領域である** (確定項目 4)；含める情報がブロックヘッダではなく**各レコード**に載る場合、コストはレコード数倍に膨らむ (D8 (コアインデックス構造)、確定項目 19)；「最終検証時刻」のインライン化は **scrub を読み取り専用から書き込み操作へ変質させ**、1回走査するごとに新ブロックを生成し、旧ブロックを無効判定して deadlist に流し込むことになり、**D3 (領域割り当て) の常駐整理の目標と完全に相反する** (確定項目 19 基準 1)。

```

**出处 `.claude/kb/decisions/18-ブロックに含める情報.ja.md:7-609`（整段抄，未转述）**

```markdown
### 確定項目

**D18 (ブロックに含める情報) の項目番号は1体系のみであり**、「未確定項目」節と連番を保ち、番号の再割り振りは行わない。

| # | 項目 | 確定内容 |
|---|---|---|
| 1 | **系譜書き換えシーケンス番号** | 不採用：それが獲得しようとする「古い複製と現行複製の識別」は D3 (領域割り当て) 確定項目 1 の配置先ごと述語ですでに獲得済み **状態：確定。** **進捗：実装済み。** |
| 2 | **ノード自身による key 区間の保持** | 全コード 2 ノードが無条件で key 区間を保持、ツリー別に分割せず、スイッチも設けない **状態：確定。** **進捗：実装済み。** |
| 3 | **データブロック自己記述** | 平文とし、余裕を持ったサイズを確保：各データユニットは平文の自己記述ヘッダを保持、5つ組で計 33 バイト；`locality_id` は追加しない **状態：確定。** **進捗：実装済み。** |
| 4 | **単一ディスク自己回復の形態** | メタデータ多重コピー + 分散配置 (ditto) を採用、チェーン案は却下；単一デバイスレイアウト方針は未開設；配置先オフセットは2のべき乗を禁止 **状態：確定。** **進捗：未実装。** |
| 5 | **再構築パスがいかに A6 (AAD 期待値は平文側から得ない) と両立するか** | 再構築は縮退パスであり、取得可能な目撃証拠により3段階に区分；システム構成 `map_provenance` 予約領域確保 + 読み取り専用マウント + その場修復の禁止 **状態：確定。** **進捗：未実装。** |
| 6 | **データブロック自己記述ヘッダの位置** | ヘッダを下位に配置、ペイロードはヘッダ直後から開始；ヘッダはブロックレベルに配置しユニットレベルには配置しない **状態：確定。** **進捗：実装済み。** |
| 7 | **ブロックヘッダに第6の平文フィールドが必要か** | ユニットヘッダフィールド表はユニットクラスごとに分割：共通平文プレフィックス 42 + クラスアイデンティティセグメント、nonce / MAC 予約領域を確保；自己証明ユニットはこの表を使用しない **状態：確定。** **進捗：実装済み。** |
| 8 | **削除時に墓標ユニットを書き込むか** | 必要：削除はディスク上に墓標レコードを残す、コンテナはコード 3、レコードはパックレコードタイプ 1、対象の死亡物はレコード自身のフィールドにより特定 **状態：確定。** **進捗：実装済み。** |
| 9 | **「ブロックレベル」の「ブロック」とは何か** | ブロック = ユニット、1ユニットに1つのヘッダ；スキャナはプール内の最小ユニットサイズ単位でステップ走査 (第1版は 16384)；ユニット起点はクラスごとにアライメント **状態：確定。** **進捗：実装済み。** |
| 10 | **墓標の粒度とキャリア** | 範囲レコード + パック共有ユニット；世代ごとに格納、異世代の混在禁止；第1版は 40 バイト、回収不能 (確定項目 24) **状态：確定。** **進捗：実装済み。** |
| 11 | **ユニットタイプタグの値域、およびパックレコードユニットの分類** | 値域は1枚の登録表；パックレコードユニットは独立した1クラス (コード 3、ヘッダ 107)；パックレコードタイプは第2段階の登録表：1 墓標、2 inode、3 退役、4 インスタンス表、5 系譜 **状態：確定。** **進捗：未実装。** |
| 12 | **コード 2 の書き込みシーケンスを 4 バイトに縮小すべきか** | 縮小：コード 2 書き込みシーケンスはインスタンス代号 4 のみを残し、空いた 6 バイトをペイロード CRC 4 + 予約 2 に配分；コード 3 の 6 バイトは変更なし **状態：確定。** **進捗：実装済み。** |
| 13 | **暗号化有効時にコード 1 のアイデンティティセグメントと `宣言長` を平文と暗号文のどちらに配置するか** | アーム庚を採用：コード 1 アイデンティティセグメントと宣言長はその場で暗号文へ移行、位置と幅は維持；コード 1 平文ホワイトリストは {fsid, ペイロード CRC} に縮小 **状態：確定。** **進捗：未実装。** |
| 14 | **再構築パスはいかに完全な 96 ビット nonce を取得するか** | アーム甲を採用：nonce 代号は完全な 96 ビット nonce (12 バイト) であり、ユニット平文ヘッダに常駐、ポインタ側のコピーも保持 **状態：確定。** **進捗：実装済み。** |
| 15 | **撤回済み・本番号は欠番** | 撤回 (旧題：コード 2 のバージョン選択規則)：3者論証3巡でも反駁されない記述が見出せず；コード 2 は従来通りバージョン選択規則の対象外 **状態：確定。** **進捗：対象なし。** |
| 16 | **nonce / MAC / アルゴリズムタイプ予約領域のスイッチ貫通：コード 1 の配置先、予約領域を含むコード 2 / コード 3 のヘッダ幅、宣言長の原点** | 29 バイトの予約領域をクラスアイデンティティセグメント直後に配置しヘッダに算入：コード 1 ヘッダ 134、コード 3 ヘッダ 136、コード 2 ヘッダ 115 + 2 × key 幅；宣言長原点は予約領域を含むヘッダ末尾 **状態：確定。** **進捗：実装済み。** |
| 17 | **32 バイトヘッダチェックサムのアルゴリズム；自己証明構造の対象範囲** | 32 バイトフィールドに CRC32C 4 バイト + 28 バイトのゼロを格納；自己証明構造のチェックサムはパディングを含むスロット全体をカバー、パディングは常時 0 **状態：確定。** **進捗：実装済み。** |
| 18 | **コード 2 の宣言長セマンティクス、パディング領域常時 0、コード 1 / コード 2 のオフセット表** | コード 2 宣言長 = エントリ数 × エントリ幅；エントリ領域直後のパディング領域は常時 0；コード 1 とコード 2 にそれぞれオフセット付きフィールド表を補完 **状態：確定。** **進捗：実装済み。** |
| 19 | **フィールド値をブロックに含めるべきか：3つの判定基準** | 3条件の積集合、不可欠：更新周期がコンテンツと連動；欠落により判定が判定可能から判定不能へ転落；既存メカニズムに対する純増分が存在 **状態：確定。** **進捗：対象なし。** |
| 20 | **保持が確定したアイデンティティフィールド** | 自身の論理アドレス + fsid、generation (誕生世代)、所属ツリー ID (誕生ツリー) の3点を必須保持 **状態：確定。** **進捗：実装済み。** |
| 21 | **自己記述の射程：誤方向書き込みであり消失書き込みではない** | 消失書き込みはブロック内のいかなる情報でも検出不能、ブロック外の目撃証拠が必要：親ポインタ内の期待世代とチェックサム、ルートまで再帰 **状態：確定。** **進捗：対象なし。** |
| 22 | **明示的フィールド vs チェックサムシードへの折り込み** | ディスク走査再構築を支援すべき情報は明示的に書き込む必要あり；誤用の検出のみで足り再構築不要な場合のみシードへの折り込みを許容 **状態：確定。** **進捗：対象なし。** |
| 23 | **否決リスト** | 含めない：子ブロックによる「現行」親ノードアドレス記録、チェックポイントインラインブロック数、COW 先行アドレス、ブロック内誤り訂正符号、ブロックごとの実時間 **状態：確定。** **進捗：実装済み。** |
| 24 | **第1版墓標：墓標ツリー、40 バイトレコード、系譜レコード** | 第1版墓標：専用墓標ツリー、死亡 txg を保持する 40 バイト全オブジェクトレコード、ロールバック時は系譜レコードを書き込み **状態：確定。** **進捗：実装済み。** |

#### 確定項目 1：系譜書き換えシーケンス番号は不採用

**確定内容**：系譜書き換えシーケンス番号 (系譜に沿って単調増加し、トランザクション番号と直交) は不採用とする。**それが購入しようとしていた機能は、すでに別の手段により獲得済みである**：系譜書き換えシーケンス番号の用途は「古い複製と現行複製の識別」であるが、D3 (領域割り当て) 確定項目 1 の配置先ごと述語こそがその能力の権威ある記録点である ⇒ さらにシーケンス番号を追加しても新しい能力は得られず、恒久的なフォーマットフィールドを増やすだけに終わる。

**射程**：確定により却下されたのは「古い複製を識別するために設けるシーケンス番号」である。将来別の用途が生じた場合 (例えば D16 (パブリッシュセマンティクス) で確定したチェックポイントシーケンス番号との共存に関する問いが真に独立して成立する場合など)、再開が必要であれば再開するが、**再開を提起する者はすでに獲得済みの機能を重複して購入するものでないことをまず論証しなければならない**。ユニットヘッダの書き込みシーケンス (インスタンス代号, トランザクション番号) (確定項目 7) は却下範囲外である：これは「当該ユニットがパブリッシュされたか否か、どのタイムラインに属するか」を獲得するものである；同一 key のパブリッシュ済みバージョンの順序付けは誕生世代 + 1トランザクション・1 key ごとの1種類の正味効果で十分であり、書き込みシーケンスは同一 txg 内でトランザクションをまたいで同一 key を変更する際の順序付けを副次的に担うにすぎない。確定項目 11 はかつて同一アイデンティティの複数版パックコンテナの最新選択に誕生世代を用いようとしたが、これこそがここで却下された用途である；D3 (領域割り当て) 確定項目 1 の述語による判定へ改められたため、確定項目 1 は再開しない。

**根拠**：

- E59 (バッファメッセージはユニットのみから再計算可能か)：ユニットのみを走査する場合、同一 key の複数の読み取り可能複製の間で現行値を識別できないが、配置先ごと述語を加えた後は4種類のメッセージの不一致がすべてゼロとなった——シーケンス番号が購入できるのは上書きのマスのみであり、さらにもう1つ追加しても新たなものは得られない。
- ユーザー確定、変更履歴に記録；書き込みシーケンスに関する射程注は C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) の確定に伴い追記。

**未済の検査**：C288 (マッピング key 確定に伴う連動変更が他決定へ未反映) 第 ⑥ 条：射程注に「誕生シーケンス番号は key の一区間としてのみ機能し、最新選択には用いない」旨を追記。

#### 確定項目 2：全コード 2 ノードが無条件で key 区間を保持

**確定内容**：**すべてのコード 2 インデックスノードは `[min_key, max_key]` を保持し、ツリー別に分割せず、スイッチも設けない。** これによりコード 2 のヘッダ幅計算式 `86 + 2 × key 幅` (確定項目 18) は全ツリーで無条件に成立し、走査時にコード 2 ノードをデコードする際「このツリーが保持しているか否か」を事前照会する必要がなくなる。これが獲得する能力は：ノードを単独で拾い上げた際に、それがカバーすべき key 範囲を自己証明できることであり、`.claude/rules/fs-design.md` の「領域節約のために自己完結性を犠牲にしない」方針と合致する。**区間はサブツリーカバー区間を採用する**：`[min_key, max_key]` は当該ノードのサブツリー内で到達可能な最小 key と最大 key である——葉層のコード 2 ノードでは先頭と末尾の2エントリの key となる；子がコード 3 コンテナである内部ノードでは、max_key は最後のコンテナ内の最後のレコードの key であり、最後のエントリのセパレータ key ではない。新規プールでのファイル作成のバイト列変更：なし——6本のコード 2 ツリーが書き出すバイト列にはもともと区間が含まれており、新規プールでのファイル作成段階では両方の解釈で同値となる (inode ツリールート [1, 1])。

**射程**：外挿されるのは「保持するか否か」という判定であり、E73 (ノード key 区間のファンアウトコスト) のコスト数値ではない——どのツリーでファンアウトが何スロット低下するかは個別に測定が必要。**dirent ツリーの検討マスはこれに伴い閉鎖**：dirent ツリーが存在する場合も同様に key 区間を保持する；その際再測定すべきはファンアウトコストであり、本判定ではない。key 区間はブロックヘッダ (D15 (フォーマット凍結ポリシー) 第1層) に常駐し、会計ツリーノードヘッダと同一のマスに位置する (C102 (インデックスノードヘッダに予約領域がなくフィールド追加は compat に非ず))。checker の `check_index_node_keys` (`crates/singlefs-checker/src/lib.rs`) は先頭と末尾の2エントリの key を判定しており、並行線3のルートは [2, 2] を書き出すのに対しサブツリーカバーは [2, 234] であったため、両箇所ともサブツリーカバー区間に従って修正が必要 (C478 (コード 2 ヘッダの key 区间はエントリキー区間かサブツリーカバー区間か))。位置アドレス指定ツリー (割り当てレコードツリー、extent ツリー、D8 (コアインデックス構造) 確定項目 14) では、ノードヘッダの key 区間には位置規定に基づき当該ノードがカバーすべき区間を書き込み、これは当該ツリー群において「サブツリーカバー区間」と一致する；checker は位置に基づいて各ノードがカバーすべき区間を独立に算出し、ノードごとに照合する。

**根拠**：

- E142 (新規プールでのファイル作成のドライラン)：第6回実行で書き出された6本のコード 2 ツリーはすべて key 区間を保持しており、保持しなければいずれのツリーもヘッダ幅を計算できない；ツリーごとに保持有無を分ける手法ではコード 2 ヘッダ内に「保持有無」フラグビットを追加せねばならず、走査再構築パスにはツリーテーブルが存在しないため、フラグを読む前にヘッダ長を知る必要があるという循環に陥る。
- E73 (ノード key 区間のファンアウトコスト)：区間を保持してもファンアウトの低下は最大1スロットにとどまりツリー高は増加せず、D8 (コアインデックス構造) 確定項目 2 の 16 KiB は再開不要；孤立ノードのカバー区間判定において、区間を保持しないアームでは判定不能。
- ユーザー確定 (ポップアップ、3者論証は経ておらず覆滅可能；メイン agent の推奨も本選択肢)、原文は変更履歴に記録。
- ユーザー確定 2026-09-23：区間はサブツリーカバー区間を採用、原文は変更履歴に記録。

**未済の検査**：C478 (コード 2 ヘッダの key 区間はエントリキー区間かサブツリーカバー区間か)：checker と書き込みパスをサブツリーカバー区間に合わせて修正し、区間がエントリキー区間として書き込まれた不良イメージを配備。

#### 確定項目 3：データブロックは平文の自己記述ヘッダを保持

**確定内容**：各データユニットは平文の自己記述ヘッダを保持し、フィールドは D9 (暗号化) 確定項目 6 で確定済みの組 `(ユニットタイプタグ, ツリー ID, オブジェクト ID, オブジェクト誕生世代, アンカーオフセット)` を採用する。**領域節約のためにフィールドを削ることはせず、余裕を持ったサイズを確保する。** 5つのフィールド幅：**ユニットタイプタグ 1 + ツリー ID 8 + オブジェクト ID 8 + オブジェクト誕生世代 8 + アンカーオフセット 8 = 33** ([verification-build.md](../verification-build.md) 第8問はこれにより閉鎖；根拠の遡及は確定項目 11)。

- **D1 (データ移動性 / 逆引きインデックス) との境界**：**判定基準は D1 (データ移動性 / 逆引きインデックス) にあり** (何が自己完結セグメントに入り、何が拡張ポイントに入るかは同決定の確定項目 1 / 3 / 4 を参照)、**フィールド表は本決定にある** (実際にどのフィールドが入り、各フィールドの幅、ユニットクラスごとの配分)。判定基準が変われば本表も連動して改定される；表の配置都合で判定基準を逆向きに変更することはない。
- **第6のフィールド `locality_id` は追加しない**：`locality_id` はヒントにすぎず (D8 (コアインデックス構造) 確定項目 3)、再構築時のオブジェクトは (ツリー ID, inode) により識別され、本ヘッダの extent ツリーのみを参照し、当該オブジェクトの全 extent key (残存および回収分) に同一の先頭セグメントを付与し、inode レコード > 残存 extent key > 0 の優先順で値を採る；誤った値を採っても局所性を失うだけであり、再構築の成果物は本来読み取り専用である (確定項目 5)。再構築規則の本文は D8 (コアインデックス構造) 確定項目 6「再構築」の段落にある。
- **厳守すべき2つの防衛線**：

| 配置禁止項目 | 理由 |
|---|---|
| **物理配置先 / デバイス ID** | D9 (暗号化) 確定項目 6 は「物理位置は一律 AAD に含めない」と確定済み ⇒ ブロックヘッダに配置すると**認証されず静かに改ざん可能な**フィールドとなる。物理アドレスの権威ある記録点は D9 (暗号化) 確定項目 5 (scrub のみ実施) により暗号化構造内に留まり、鍵を持たない側は本来読み取ることができない |
| **ファイル名 / セパレータ key / あらゆるパス情報** | D9 (暗号化) 確定項目 5 の確定文言に明記されている通り、ZFS 方式との**すべての**相違点は範囲にある——「btree 内部ノードのセパレータ key を露出しない」、そして dirent ツリーのセパレータ key こそがファイル名である。**これを防守できなければ ZFS と同一の道を歩むことになる** |

- **拡張ポイントへの物理ポインタ配置がこの2線に抵触しない理由：「静かな誤読を引き起こさない」ためであり、「認証されている」からではない**：この2線が防ぐべきは**静かな誤読**である——ブロックヘッダ内の偽造された物理配置先は、読者に誤ったブロックを読ませながら正常に読めたと誤認させる。一方、拡張ポイント内のポインタは**いかなる判定パスにも入らない** (D1 (データ移動性 / 逆引きインデックス) 確定項目 5 文言：参照を構成しない)；参照解決ごとに**ターゲットの誕生世代**の照合とターゲットユニット自身のヘッダチェックサムを通過する必要がある ⇒ 偽造されても精々手がかりが空を指すだけであり、**「誤読したのに正読と錯覚する」事態は生じ得ない**。これは D9 (暗号化) 確定項目 5 がすでに認識のうえ受容しているリスク (平文側は並べ替え可能、世代は偽造可能) と同類であり、新カテゴリを追加するものではない。**「拡張ポイントはユニット全体の検証 / 認証範囲に含まれる」ことを理由にしてはならない**；理由は「静かな誤読を引き起こさない」ことを維持し、MAC が拡張ポイントをカバーしているか否かに依存しない；「拡張ポイントが認証されている」に戻すには再論証が必要であり、未実施である。

**射程**：本項目が購入するのは「インデックス消失時にも再構築可能」という能力であり、**「攻撃者が改ざんした場合に検知可能」ではない**——後者は AAD 不一致および D4 (チェックサム位置) の Merkle ツリーが担う。**ブロックヘッダが平文可読であっても認証を構成しない**：通常の読み取り (インデックス健在時) では平文ヘッダは関与せず、AAD 期待値は A6 (AAD 期待値は平文側から得ない) に従い検索パスから供給される；再構築 (インデックス消失時) では関与が不可避であり、フィールド単体の改ざんは MAC 不一致を招くが、ユニット全体の持ち運び (ヘッダ + 暗号文の同時移動) は検証を完全に通過し、まさにそれこそが A6 (AAD 期待値は平文側から得ない) が阻止すべき「ブロックレベル並べ替え」である——このマスは確定項目 5 の縮退セマンティクスに基づき処置する。2つの防衛線が管轄するのは自己完結不変データのセグメントであり、拡張ポイントは管轄外である：**物理配置先は可変である** (ブロックは移動され得、D1 (データ移動性 / 逆引きインデックス) の存在理由こそが移動可能性にある)；「バージョンに伴い変化するか否か」の基準により本来自己完結セグメントには入れない；拡張ポイントは可変データのためのものである (D1 (データ移動性 / 逆引きインデックス) 確定項目 3：空であるか、1つのポインタである)。暗号化有効時、ブロックヘッダのこの組はディスク上における**唯一の**平文表現であり (D9 (暗号化) 確定項目 5 は平文マッピング層および平文逆引きインデックスを設けない)、アンカーオフセットはファイルサイズとファイルホールを直接露呈する；この漏洩は受容不能であり、処置は確定項目 13 に帰属し、確定項目 3 自体は変更されない——データユニットは引き続き自己記述ヘッダを保持し、フィールド集合と幅は維持され、確定項目 13 が変更するのは「暗号化有効時にこれらフィールドがディスク上で平文か暗号文か」のみである；主たる根拠 (高速取得のために存在し、領域節約のために自己完結性を犠牲にしない) は漏洩面のマスに依存しない。この組は同時に D9 (暗号化) 確定項目 6 の AAD フィールド集合でもあり、C72 (extent とユニットの関係が未回答) の議論で「AAD のフィールド構成は day-1 恒久契約である」と確立された ⇒ これを変更することは恒久契約の改定を意味する。確定項目 3 から派生した3つのマスはそれぞれ正式な項目となっている：確定項目 5、確定項目 6、確定項目 7。

**根拠**：

- E28 (平文マッピング層はユニット自己記述のみから再構築可能か)：平文マッピング層を削除し、ユニットヘッダの走査のみで再構築した場合、メタデータ側は1件の欠落も誤りもなく、データブロック側の不足は正確にデータブロック総数に一致した；陽性対照としてデータブロックに自己記述を加えると不足はゼロとなった——これが購入した欠損埋め合わせである。
- ユーザー確定：これらのフィールドは**高速取得**のために存在し、早ければ早いほどよい；一般的な方針は**領域節約のために自己完結性を犠牲にしない** (`.claude/rules/fs-design.md`)、原文は変更履歴に記録。
- 2つの防衛線の射程：ユーザー確定、原文は変更履歴に記録。
- 「ブロックヘッダ平文可読は認証を構成せず」：3者論証の逆推論側が指摘、メイン agent が D9 (暗号化) 確定項目 6 の判定基準 A6 (AAD 期待値は平文側から得ない) を逐字照合して確定、経緯は変更履歴に記録。
- `locality_id` の不採用：3者論証3巡 (`research/prompts/d18-locality-r1-main-verification.md`、`d18-locality-r2-main-verification.md`、`d18-locality-r3-main-verification.md`) を経たユーザー確定、却下形態の論証は3通の判決文に記載。

**未済の検査**：C346 (バックグラウンド再クラスタリングが未定義)；C68 (不変条件ホワイトリストと確定条項の衝突)：ゲート照合治具。

#### 確定項目 4：形態はメタデータ多重コピー + 分散配置を採用、チェーン案は却下、単一ディスクレイアウト方針は未開設

**確定内容**：単一ディスク自己回復の形態は「メタデータ多重コピー + 分散配置」 (ditto) を採用し、チェーン案は正式に却下する；ただし**単一デバイスレイアウト方針は未開設とする**。ditto は ZFS の実形態を採用：ユーザーデータ 1 コピー、データセットメタデータ 2 コピー、全プールメタデータ 3 コピーとし、複製は分散配置する。**ditto 採用時に必須となる1条**：配置先オフセットは**2のべき乗を禁止する**——2のべき乗周期のあらゆる障害と共振し、3つの複製が同一の合同関係となって全滅するため。新規プールでのファイル作成のバイト列変更：**なし** (ditto は複製のコピー数と配置ポリシーであり、いかなるフィールド表も変更しない；単一ディスクレイアウト方針は未開設 ⇒ 新規プールでのファイル作成が書き出すバイト列は不変)。

**射程**：**確定項目 3 の確定は確定項目 4 を軽減しない**：データブロック自己記述が防ぐのは「**インデックスの消失**」であり、確定項目 4 が防ぐのは「**ブロック自身の消失**」である——自己記述はディスク上に存在しないブロックを読み出すことはできず、両者の射程は異なる。単一ディスク方針を開設しない理由：単一ディスク方針は**別の独立したレイアウト**であり、「2台ディスク構成から1台抜いたもの」ではない——D2 (RAID ストライプポリシー) 確定項目 6 の `w ≥ 2` は単一デバイスと排他関係にあり (D2 (RAID ストライプポリシー) 確定項目 9)、D12 (対象メディア) でも incompat ビットを割り当てる必要があり、恒久的なフォーマット分岐となるため、`.claude/rules/fs-design.md` の分岐判定基準に従い個別の立証を要する；方針開設には個別のクラッシュポイント再生を通過せねばならず、テスト専用の強制移行スイッチも必要となる。マルチディスクと単一ディスクは等価ではない：`w ≥ 2` かつ冗長ゼロのストライプ発行禁止 ⇒ ≥2 台ディスクプール内の全ストライプは生来パリティを保持し、1列喪失しても残存列から再構築可能である (「1ユニットが丸ごと1列に配置される (D2 (RAID ストライプポリシー) 確定項目 10) ⇒ ディスク1台喪失はユニット丸ごとの喪失 ⇒ マルチディスクと単一ディスクは同等」という説は**成立しない**)；**単一ディスクにはこの機構が存在しない**。チェーンがバージョンチェーン (前後 = 本ユニットの COW 先行および後続) を採る場合、否決リストの2項目に抵触する (確定項目 23：「子ブロックによる現行親ノード物理アドレス記録」は「時系列上不可能」、「COW 先行アドレス (バージョンチェーン)」は「正味価値がマイナス」と判定)、比較対象は同一バッチチェーンのみ残る。

**根拠**：

- E118 (単一ディスク自己回復の3アーム)：実行前に固定した失敗条項が文字通り発動——同一バッチチェーンの回復数は**すべての**障害点において非冗長アームと同等であり、それが獲得するのは**検出**であるが、検出は D3 (領域割り当て) 確定項目 3 の割り当てレコードで本来回答可能 ⇒ 純増分 ≈ 0；ditto は3アームの中で唯一回復能力を獲得した；モデル内の「ディスクの少なくとも 1/8 離す」オフセットは2のべき乗であり、2のべき乗周期の障害と共振した。方針開設前の6項目の挙証責任はこれにより一括して解消された。
- 3者論証で課された6項目の開設挙証責任と赤判定条件、ユーザー確定は変更履歴に記録。

**未済の検査**：C201 (ditto の配置先オフセットは2のべき乗を禁止)。

#### 確定項目 5：再構築は縮退パスであり、取得可能な目撃証拠により3段階に区分

**確定内容**：再構築が縮退パスであることを認めるが、その形態は単なる宣言文ではなく、取得可能な目撃証拠による段階区分とする：

| 段階 | 取得された目撃証拠 | 処置 |
|---|---|---|
| 2 | 割り当てレコード (D3 (領域割り当て) 確定項目 3) + チェックサム / MAC の二重証明 | 書き込み可能、正常昇格；⚠️ 到達不能——割り当てレコードは派生状態であり (D21 (権威状態と派生状態の境界) 確定項目 5)、派生状態は目撃証拠になり得ず、段階 2 の目撃証拠は存在しない |
| 1 | チェックサム / MAC の自己整合性のみ、割り当てレコードなし | **読み取り専用マウント** |
| 0 | チェックサムの検証すら不能 | 報告のみ、マッピングには含めない |

**段階区分の判定基準は計算可能であり、入力は操作期間中不変である**ため、`.claude/rules/fs-design.md` 分岐のハード要件 1 および 5 を満たす。**3つの要件すべてが具現化されねばならず、1つでも欠ければ単なる注意喚起にとどまる** (`.claude/singlefs-ai-sop/rules/show-me-test.md` は踏み抜いた落とし穴を注意喚起文として記述することを文字通り禁止している)：

| # | 具現化形態 | なぜこの形態でなければならないか |
|---|---|---|
| 1 | システム構成内に `map_provenance` (再構築エポック + 再構築時に依拠した権威状態要約) を**初日から予約** | 初日に予約しなければ、「後から追加可能」という目論見は D15 (フォーマット凍結ポリシー) の凍結当日に静かに失効する。認証対象フィールドに配置せねばならない (D9 (暗号化) 確定項目 8 は nonce 水位のためシステム構成が認証必須と確定) |
| 2 | `map_provenance = rebuilt` かつ未昇格 ⇒ **読み取り専用マウント** | `.claude/rules/fs-design.md`「誤用をマウント不能に変える」`compat_ro` 形態：マウント可能だが書き込み不能、明示的な失敗モード |
| 3 | 規律の確立：当該状態下において**いかなる権威状態構造も再構築結果によりその場で修復してはならない** | I-6.10 (廃止済み・平文マッピング層は存在しない) の「差異は一律赤判定、**その場修復の禁止**」を再構築成果物へ拡張したものであり、新機構ではない |

`map_provenance` のフォーマットレベルの3要件：

| # | 要件 | 理由 |
|---|---|---|
| 1 | システム構成に常駐し、**痕跡なく書き換え可能な位置に配置してはならない** | 1バイトの改ざんで回避可能な関門は、関門が存在しないことと等価である。保護強度は暗号化スイッチに連動 (射程) |
| 2 | **段階 1 → 段階 2 への昇格は痕跡を残さねばならず**、フラグビットをその場で正常に戻してはならない | 昇格とは「合法的に rebuilt を解除すること」であり、**それ自体が回避パスの形状を呈している**。記録内容：依拠した二重証明のバッチ、昇格時刻；当該記録自体も checker で再検査可能であること |
| 3 | 以後の正常な書き込みパスが**これを上書きしてはならない** | 健忘形態：一度宣言された後二度と参照されず、以後の各マウントで正常表示されること。**善意の振る舞いであってもそこに到達する** |

**「人間の確認を要求する」後半部は不採用**：管理者には比較照合すべき元のインデックスや参照データが存在せず、**ファイルが移動されたか否かを判定できない** ⇒ その要求は実際上、検証可能なデータが存在しない前提で再構築プロセスを信用するよう人間に強要することに等しく、**認証の目的に真っ向から反する**。昇格の判定基準は段階 2 のような計算可能な二重証明に限られ、人間の承認であってはならない。

**射程**：**第1版で防げること・防げないことを事実通りに記述する**：D9 (暗号化) 確定項目 8 によりシステム構成は認証必須とされたが、その手段は**マスターキーから派生した MAC** であり、D9 (暗号化) 確定項目 10 は最初の実行可能バージョンに暗号化を含めないと確定した ⇒ **第1版にはマスターキーが存在せず、その認証も存在しない** ⇒ `map_provenance` は第1版において非鍵チェックサムでのみ保護され、**偶発的な破損を防ぐのみで意図的な改ざんは防げない**；これは D9 (暗号化) 確定項目 5 の方針と合致するが、**明文化せねばならず**、「このフィールドがある」ことを「この縮退状態が消去不能である」と解釈してはならない。**A6 (AAD 期待値は平文側から得ない) は第1版において1バイトも管轄しない**：第1版で実際にバイト列を変更するのは**唯一 `map_provenance` の予約領域のみである** (システム構成「マッピング由来 (再構築エポック + 権威状態要約)」24 バイト、全ゼロ、[layout/01-first-txn.md](../layout/01-first-txn.md))。残りは再構築コードの実装形態であり、`crates/` 内に走査再構築は存在せず (`grep -rn "扫描重建" crates --include=*.rs` はゼロヒット)、第1版 checker (`crates/singlefs-checker`) は暗号化を有効化せず A6 (AAD 期待値は平文側から得ない) を判定しない。**段階 2 の目撃証拠は喪失した** (割り当てレコードは派生状態、C73 (割り当てレコードの状態分類が未確定) 文言「派生状態のものは自身が再構築対象であり、目撃証拠にはなり得ない」)：段階 2 は権威状態に常駐する別の二重証明を見出すか、本マスを再開せねばならない；**段階 1 と段階 0 は通常通り利用可能**であり、影響を受けるのは昇格ステップのみである。

**根拠**：

- 実験なし：段階区分は取得可能な目撃証拠に基づき再構築成果物のマウント状態を定義するものであり、定義的であって計測可能な量はない。
- 3者論証：2つのアームが独立して候補 ① (親ポインタ頼み) は「インデックスが失われて初めて再構築が必要になる」という前提下で循環論法に陥ることを確認——それが利用しようとする親ポインタはインデックスに常駐し、インデックスこそが再構築対象のツリーである；権威側は D21 (権威状態と派生状態の境界) に従い = ユニット + 会計 + ルートであり、インデックスを含まない；ローカル側アームは「人間の確認」は判定不能と指摘。ユーザー確定は候補 ② を採用、原文は変更履歴に記録。
- 段階 2 目撃証拠の非存在：D21 (権威状態と派生状態の境界) 確定項目 5 のユーザー確定 (割り当てレコードは派生状態)。

**未済の検査**：C75 (縮退マウントの関門自体が赤判定にならない)。

#### 確定項目 6：データブロックの自己記述ヘッダを下位に配置、ペイロードはヘッダ直後から開始

**確定内容**：各ブロックの **0 バイト目からが自己記述ヘッダ**であり、ペイロードが直後に続く。**「ヘッダをブロックレベルとユニット (extent) レベルのどちらに設けるか」の論点はブロックレベルと確定**：下位に配置することは、スキャナがブロック単位でステップ走査するだけでヒット可能であることを意味し、ユニットレベルではスキャナが事前にユニット境界を把握している必要が生じる——**それこそが再構築すべき対象である** (循環)。

**射程**：ブロック = ユニット (確定項目 9) かつ extent が複数ユニットにまたがらない (D4 (チェックサム位置) 確定項目 7) とされた後、「ブロックレベル」と「ユニットレベル」はこの値において合致する；ブロックレベルが当初負うべきであった「N ブロックにまたがる大 extent が N 個のヘッダコストを支払う」という償却コスト N は常に 1 となり、問題の対象は解消した。

**根拠**：

- 実験なし：判定基準は「リカバリ期間の全ディスク走査が**インデックスなし**の前提でこれらヘッダを見出せること」にある ⇒ ヘッダはスキャナが**予測可能な**位置に配置されねばならず、ブロック先頭はいかなる事前知識も要さずに算出可能な唯一の位置である。
- ユーザー確定、変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 7：第1版ユニットヘッダフィールド表、ユニットクラスごとに分割

**確定内容**：第1版ユニットヘッダフィールド表はユニットクラスごとに分割する——共通平文プレフィックス + クラスアイデンティティセグメント；**fsid と誕生世代をヘッダ内に配置し、ヘッダチェックサムは 32 バイトを採用**；v1 では nonce 代号と MAC のフィールド領域を予約；自己証明ユニット (journal レコード / ルートスロット / システム構成スロット) は本表を使用しない。

**共通平文プレフィックス (データユニットおよびインデックスノード、初期値 42 バイト；幅は E85 (ユニットヘッダのフィールド表) の要求マトリクス計測条件を継承し、コード実装時に微調整)**：

| フィールド | 幅 | 出典 |
|---|---|---|
| magic | 4 | 走査プローブ (E86 (走査ステップと2つのユニットサイズ)：16384 ステップでデータペイロード内部に落ちるプローブをこれとヘッダチェックサムで拒否) |
| フォーマットバージョン | 2 | D21 (権威状態と派生状態の境界)：走査再構築は過去の全ユニットフォーマットを解釈可能でなければならない |
| flags | 2 | **オフセット 6 = ユニットタイプタグ (確定項目 11 登録表のコード)、オフセット 7 = 8 個の flag ビット、第1版は常時 0、非 0 は一律拒否 (前方互換性ポリシー)**——暗号化有効後、メタデータのアイデンティティは暗号文へ入るため、スキャナは復号前にクラスを判別してどのホワイトリストを適用すべきか知る必要がある |
| 宣言長 | 2 | D4 (チェックサム位置) 確定項目 6：I-2.3 (パディングバイトはゼロかつチェックサムに関与) を走査時にインデックスなしで判定可能とするため。⚠️ **暗号化有効時は暗号文となる** (確定項目 13、アーム庚)——位置と幅は維持され、平文ホワイトリストから外れるのみ |
| ヘッダチェックサム | 32 | 親ポインタが存在しない走査再構築時におけるヘッダ完全性の唯一の防衛線 (暗号化有効後はメタデータクラスに対して平文部分のみをカバーし、クラスアイデンティティセグメントの完全性は MAC が担う)；**カバー範囲は確定項目 11 で確定：オフセット 0 から当該クラスの平文ヘッダ末尾まで** (I-2.4 (ヘッダチェックサムカバー範囲))；幅は E23 (journal 幾何) `header_csum`、D22 (ユニット原子性の合成方法) 確定項目 7 ルートレコード自己証明チェックサムと同計測条件。**アルゴリズムは確定項目 17 で確定**：CRC32C 4 バイト + 28 バイトのゼロ |

**クラスアイデンティティセグメント——データユニット (+63 バイト ⇒ 計 105 バイト初期値、32 KiB ユニットの 0.32% を占有)**：
<!-- format-const: DATA_UNIT_HEADER_BYTES = 105 -->

| フィールド | 幅 | 出典 |
|---|---|---|
| 5つ組 | 33 | D18 (ブロックに含める情報) 確定項目 3 で確定済み (平文、余裕を持った設計)；うちユニットタイプタグは flags クラスラベルビットと冗長であるが受容 (`.claude/rules/fs-design.md`「領域節約のために自己完結性を犠牲にしない」) |
| 誕生世代 (birth txg) | 8 | 3つの独立した理由、根拠を参照 |
| fsid | 8 | 2つの独立した理由、根拠を参照 |
| **書き込みシーケンス (インスタンス代号 4 + トランザクション番号下位 48 ビット)** | **10** | C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P1：それを書き出したトランザクションの (インスタンス代号, トランザクション番号)、3つの目的で1フィールドを共用——同一 key のパブリッシュ済みバージョン順序付け、クラッシュ前後のタイムライン識別、インフライトなチェックポイントの部分再生時に適用済みと放棄分を分離；トランザクション番号は D23 (journal の役割とフォーマット) 確定項目 7 で確定済みの 8 バイトフィールドの下位 48 ビットであり、新規フィールドは開設しない |
| **ペイロード CRC** | **4** | C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P3：複製マージは「同クラスのアイデンティティセグメントを持つ複数の読み取り可能ユニットのペイロードが同一であること」を要求し、コード 1 にはこれを証明する他のフィールドが存在しない ⇒ C104 (走査パス上でコード 1 / 2 のペイロード完全性に自己完結的な根拠がない) のペイロード CRC はコード 1 の複製マージ / 段階 1 リカバリの前提；コード 1 自身のクラスアイデンティティセグメントにのみ追加し、共通プレフィックスには含めない |

- **暗号化有効時、本クラスアイデンティティセグメント表内で平文として残るのは fsid とペイロード CRC のみである** (確定項目 13)：5つ組 / 誕生世代 / 書き込みシーケンスはその場で暗号文となり、位置と幅はいずれも変更されず、平文ヘッダ総幅は 105 のまま維持される。暗号化無効時は表全体が平文であり、新規プールでのファイル作成が書き出すバイト列はバイト単位で不変である。
- **ペイロード CRC がカバーするのはディスク上のバイト列であり、暗号化前の平文ペイロードではない** (確定項目 13-c；コード 1 / 2 / 3 の3クラスで同一の計測条件とし、他箇所からは一律ここにリンクする)。暗号化無効時は両者は同義であるが、有効時は**暗号文**に対して計算される。2つの独立した理由：① 平文をカバーすると鍵を持たない側が平文を読めなくなり、D9 (暗号化) 確定項目 5 が約束した「破損検知、不良ブロック報告」が即座に破綻する；② 平文をカバーしつつ CRC 値が平文ヘッダ内にあると、鍵を持たない攻撃者に 32 ビットの**内容確認オラクル**を与えることになる——平文を推測し、CRC を計算して照合すれば、推測の正否を確認できてしまう。認識のうえ受容したコスト：同一 extent の各複製は同一の暗号文である (D2 (RAID ストライプポリシー) 確定「N 個の複製は同一暗号文を N 回書き込み、1つの nonce を共用する」) ⇒ CRC の一致により複製を識別可能 ⇒ `w` が逆算される；第1版の `w` は常に 2 であるため (D2 (RAID ストライプポリシー) 確定項目 9) ⇒ 本マスにおいて第1版の増分はない。**これが検証するのは「暗号文が破損していないこと」であり、「ユーザーデータが破損していないこと」ではない**：復号パス自体が持ち込む誤りは MAC が捕捉し、本フィールドは捕捉しない。
- **クラスアイデンティティセグメント——インデックスノード**：ツリー ID + 階層 + **key 区間** (確定項目 2：全コード 2 ノードが無条件で保持) + 誕生世代 + fsid + **書き込みシーケンス 4 (インスタンス代号のみ) + 誕生シーケンス番号 4 + ペイロード CRC 4 + 予約 2 + 自己記述3セグメント (key 幅 1 + エントリ数 2 + エントリ幅 2) 5** (書き込みシーケンス、ペイロード CRC、予約の3セグメントは確定項目 12 を参照；誕生シーケンス番号は C288 (マッピング key 確定に伴う連動変更が他決定へ未反映) 第 ① 条を参照；自己記述3セグメントは確定項目 18 および D8 (コアインデックス構造) 確定項目 11 を参照；5セグメント合計 19)、クラスアイデンティティセグメントの直後に 29 バイトの nonce / MAC / アルゴリズムタイプ予約領域が続く (確定項目 16)。3セグメントそれぞれのセマンティクス：**書き込みシーケンス 4** は C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P1 の 10 バイトの中でコード 2 にとって唯一消費者を持つセグメントである——I-1.2 (ブロックヘッダ書き込みシーケンスパブリッシュ済み) 文言「コード 2 / 3 は b ≤ T_pub を要する」、述語はインスタンス代号のみを読み取る；未パブリッシュのチェックポイントのノードをパブリッシュ済み述語によりゴミと判定させ、清掃可能にする能力を購入する。**ペイロード CRC 4** は C104 (走査パス上でコード 1 / 2 のペイロード完全性に自己完結的な根拠がない) のコード 2 側の半分を解消し、計測条件はコード 1 およびコード 3 に準じる：CRC32C、ヘッダ末尾からユニット末尾 (レコード領域 + パディング) をカバーし、ヘッダチェックサムのカバー範囲内に収まる。**予約 2** は D8 (コアインデックス構造)「flags / パディング / 予約の3セグメントは同一ポリシー」に準拠：常時 0、読者が非 0 に遭遇した場合 ⇒ 当該レコードは EIO (オブジェクトレベル)、意味付与には incompat が必要。**誕生シーケンス番号 4**：D19 (ブロックポインタ構造と幅予算) 確定項目 6 がコード 2 / コード 3 にそれぞれ追加した 4 バイト、**書き込みシーケンスの直後に配置**——コード 2 では書き込みシーケンス 4 とペイロード CRC 4 の間、コード 3 では書き込みシーケンスの後 (オフセット 103、確定項目 11)；中央マッピング key の一区間としてのみ機能し、最新選択には用いない (D19 (ブロックポインタ構造と幅予算) 確定項目 6)。**削除されたのは 6 バイトのトランザクション番号である** (確定項目 12)。コード 2 の完全なオフセット表は確定項目 18 を参照し、ここには重複して記載しない：平文ヘッダ 86 + 2 × key 幅、29 バイトの予約領域算入後は 115 + 2 × key 幅 (D8 (コアインデックス構造) 確定項目 11)。
- **v1 初日から nonce 代号と MAC のフィールド領域を予約** (D9 (暗号化) 確定項目 10「フォーマットフィールドのみ予約」の初日予約規律)：予約しなければ暗号化有効化がレイアウト変更となってしまい、D21 (権威状態と派生状態の境界) が要求する「走査再構築が全過去フォーマットを解釈可能であること」に反する。
- **自己証明ユニット (journal レコード / ルートスロット / システム構成スロット) は本表を使用しない**：それぞれのヘッダは D23 (journal の役割とフォーマット) および D22 (ユニット原子性の合成方法) 確定項目 7 で確定済み；これらには親の参照がなく、ツリーアイデンティティもないため、I-1.1 (ブロックヘッダ自己記述論理アドレス) / I-1.3 (ブロックヘッダツリー ID 一致) から**明示的に免除**される；自己証明規律は I-7 (ルートリング) / I-8 (journal) の2クラスおよび D22 (ユニット原子性の合成方法) 確定項目 21 にある。これらは確定項目 11 の登録表でそれぞれ1つのコードを占める——ルートレコード (ルートスロット) 16、journal レコード 17、システム構成スロット 18 (順序は確定項目 11 の登録表に準拠)、AAD ドメインセパレータとしてのみ機能する。
- **取得 (Claiming) 規則**：同一ボリュームの自己バックアップミラーは fsid で防ぐことができない (fsid が同一)；そのマスは**取得規則**により防ぐ：スキャナが1つのユニットヘッダをデコードしてそれを取得した後は、ユニット全体単位でスキップし (コード 1、コード 3 は2スロット、コード 2 は1スロット)、ペイロード内部のプローブはもはや取得に関与しない。この規則は checker 内で赤判定可能な形式として具現化されねばならない。
- **暗号化有効後の平文ホワイトリストは2枚に改訂** (権威ある記録は [invariants/08-加密不变量.md](../../../crates/singlefs-checker/invariants/08-加密不变量.md) I-6.2 (平文ヘッダフィールドホワイトリスト) に存在)：両クラスとも共通プレフィックス + nonce 代号 + MAC を共有；メタデータクラスのアイデンティティフィールド (5つ組 / key 区間) は一律暗号文へ入る (鍵なしの再構築は本来不可能、D21 (権威状態と派生状態の境界) で確定済み)；データユニットクラスは {fsid, ペイロード CRC} を追加で許容 (確定項目 13 の縮小後)。

**射程**：インデックスノードヘッダの (ツリー ID + 階層 + key 区間) は**走査再構築**のアイデンティティにのみ回答し、**AAD アイデンティティには回答しない**——AAD 期待値は平文側から得ない (D9 (暗号化) 確定項目 6)；インデックスノードが AAD に入る際にどのアイデンティティフィールドを用いるかはリポジトリ全体で未着地である (C107 (インデックスノードの AAD アイデンティティが未着地)；D9 (暗号化) 確定項目 9 は撤回済み)。確定項目 7 はブロックヘッダの半分を管轄する；もう半分 (平文マッピング層のフィールド表) は D9 (暗号化) 確定項目 5 が「scrub のみ実施」へ改めたことに伴い一括撤回された ⇒ **「両半分のフィールド計測条件が一致せねばならない」という制約はもはや存在しない**；ブロックヘッダが本組のディスク上における唯一の平文表現である。誕生世代の荷重を受け持つ消費者はパブリッシュ済み述語である (C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P2)；「断裂孤立ユニットはこれによって初めて段階 2 へ昇格できる」という理由は、段階 2 の目撃証拠が存在しない期間中は保持有無を判別できない (確定項目 5)；他の2つの理由は影響を受けない。fsid の2つの理由は段階 2 へ進入可能な際の状態を記述している：なりすましユニットは昇格ステップに達せず段階 1 で止まる。暗号化有効後、fsid と KDF は冗長となるが、**この冗長性は受容され**、コストは 8 バイトである。nonce の 12 バイトの帰属は確定項目 14 により集約される (完全な 96 ビット nonce がユニット平文ヘッダに常駐し、ポインタ側のコピーも保持)、D9 (暗号化) 確定項目 4、[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)「nonce 代号は平文ヘッダに置かねばならない」、およびユニットの自己完結性の3箇所が同時に真となる。取得規則はユニット全体単位でスキップし、E88 (なりすましユニットと孤立ユニットの識別) で測定されたスキップ手法 (本ユニット起点 + ユニットサイズへスキップ) と一致する。

**根拠**：

- E85 (ユニットヘッダのフィールド表)：要求マトリクスとバイト計算——確定最小セットでは I-1.2 (ブロックヘッダ書き込みシーケンスパブリッシュ済み) / I-1.4 (ブロックヘッダ fsid 一致) / ヘッダ自己証明を判定不能、修正セットはすべて充足、I-6.2 (平文ヘッダフィールドホワイトリスト) との衝突を個別指摘；共通プレフィックスの幅はその計測条件を継承。
- E88 (なりすましユニットと孤立ユニットの識別)：確定形態では誤分類ゼロ；fsid と誕生世代を除去したヘッダは、取得規則を有効にしてもなりすましと断裂孤立ユニットを誤分類 ⇒ 2つの条件が成立；自己ミラーのマスは取得規則のみが防護可能。
- E104 (走査再構築における現行バージョン判定)：書き込みシーケンスの3つの用途それぞれに不可欠なマスが存在——ヘッダ内にインスタンス代号のみでトランザクション番号がないアームは、まさに「インフライト txg の部分再生」のマスで破綻 ⇒ コード 1 の書き込みシーケンスはトランザクション番号を保持せねばならない。
- 3者論証 (C68 (不変条件ホワイトリストと確定条項の衝突) 決着)：3アームの生出力は `research/prompts/c68-unit-header-{sonnet,opus,local}-output.md`、背景資料は `_c68-unit-header-background.md`；逆推論側の3つの到達可能脆弱性が fsid と誕生世代をヘッダへ差し戻し、ヘッダチェックサムは 32 バイトを採用。誕生世代の3つの理由 (kill 規則の入力が再構築後に未着地、断裂孤立ユニットは「birth がルートの checkpoint_txg より大」に基づき正面認識を要する、走査方向で I-1.2 (ブロックヘッダ書き込みシーケンスパブリッシュ済み) を判定可能とすべき) および fsid の2つの理由 (誤方向書き込みとネストされた別ボリュームミラー、C74 (暗号化未有効化期間におけるボリューム間誤方向書き込みの防衛線欠如) の scrub 半分) の原文は変更履歴に記録。ユーザーは原提案を承認、判決は付帯条件付き修正を経て着地。
- 書き込みシーケンスとペイロード CRC：C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P1 / P3。
- ユーザー確定 (2026-10-03)：取得後はユニット全体単位でスキップ (コード 1、コード 3 は2スロット、コード 2 は1スロット)、判決 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第6節 S2-2 行、原文は変更履歴に記録。

**未済の検査**：C89 (ユニットヘッダ決着の残存再計算)：E98 (inode レコードと inode ツリーの幾何) の 91 は再実行が必要 (取得規則の checker 組み込みは完了済み：プールレベル checker の I-7.8 (ルートレコードツリー ID 水位は全プール最大ツリー ID 以上) 走査方向ユニット全体取得、テストケース固定)；C104 (走査パス上でコード 1 / 2 のペイロード完全性に自己完結的な根拠がない)：検査は依然未済；C107 (インデックスノードの AAD アイデンティティが未着地)；C289 (コード 2 の誕生ツリーがどのフィールドを採るか未記載)；C291 (誕生シーケンス番号の割り当て規則が実験ソース内にのみ存在)：検査は依然未済；C68 (不変条件ホワイトリストと確定条項の衝突)：ゲート照合治具。

#### 確定項目 8：削除時は墓標ユニットを書き込む

**確定内容**：**削除はもはや「ディスク上に何も残さない」ことではなく——ディスク上に墓標レコードを残す。** 第1版ではインテント発行のパブリッシュ時に書き込み、キャリアとフィールドは確定項目 24 を参照。3文に分解して規定：コンテナのユニットタイプタグは確定項目 11 登録表の**コード 3 (パックレコードユニット)** を採用；コンテナ内のレコードは**パックレコードタイプ 1 (墓標範囲レコード)** を採用；対象の死亡物は墓標範囲レコード自身のフィールド (オブジェクト ID + オブジェクト誕生世代 + 範囲、確定項目 10) により特定され、**コンテナのアイデンティティでレコードのアイデンティティを代用しない**。なぜ必須か：削除がディスク上に痕跡を残さなければ、ディスク走査再構築は当該 key が死亡したことを知る**入力が一切存在せず**、系譜書き換えシーケンス番号もこのマスを救えないため (それは上書きのマスのみを購入し、かつ確定項目 1 で不採用と確定——ディスク上に当該メッセージの痕跡が皆無であるとき、シーケンス番号は比較のしようがない)。

**射程**：削除という操作自体がディスク書き込みを伴い、D3 (領域割り当て) 確定項目 17 には「COW システムは『領域解放』操作自体が新たな領域割り当てを要求しないことを保証せねばならない」と明記されている；その計算は D3 (領域割り当て) 確定項目 2 により決着済みである：墓標は受入制御の「自己を巻き戻すために必要な量」に計上され、書き込みの瞬間に支払われる。**本決定において二重計上は行わない。** 確定項目 8 が定めるのは「ディスク上に墓標ユニットを残す」ことであり、**それがいつ回収可能となるかは定めていない**——回収規則、粒度、キャリアは確定項目 10 に帰属する (回収規則は初期値のみであり、判定基準は実験未済)。D11 (インデックスノードにメッセージバッファを残すか否か) 確定項目 4 の第3マスは本条に依拠している。

**根拠**：

- E59 (バッファメッセージはユニットのみから再計算可能か)：ユニットのみを走査する場合、削除済み key が全数復活し、復活数は正確に削除回数と一致した；シーケンス番号 + 明示的墓標ユニットのアームでは4種類すべてゼロとなった——削除時には墓標ユニットを書き込む必要がある。
- E102 (ユニットクラス登録表とパックレコードユニット)：5つ組では満杯のコンテナに対して1エントリしか特定できないが、パッククラスアイデンティティセグメント + パース処理により全エントリを特定可能——対象の死亡物はレコード自身のフィールドにより特定され、コンテナのアイデンティティで代用しない。
- 第1版はインテント発行時に書き込み：ユーザー確定 K-U2「t1：インテント発行時に全オブジェクト墓標を書き込む」、原文は変更履歴に記録。
- ユーザー確定、変更履歴に記録。

**未済の検査**：なし (C634 (単一 unlink の墓標バイト数に trace 検査がない) は 2026-10-11 に解消済み、未済検査解消済み表を参照；レコードフィールド表と単一削除の墓標バイト数は確定項目 24 に記載)。

#### 確定項目 9：「ブロック」とはユニットであり、1ユニットに1つのヘッダ

**確定内容**：**1つのユニットが1つの自己記述ヘッダを負担する；スキャナはプール内の最小ユニットサイズ単位でステップ走査する。** 第1版では3種類のユニットが共存する——インデックスノード (コード 2) 16 KiB、データユニット (コード 1) およびパックレコードユニット (コード 3) 各 32 KiB (D4 (チェックサム位置) 確定項目 1；3クラスのユニットサイズ / アライメントは確定項目 11 の登録表に準拠)、したがって走査ステップは 16384 を採る。**これはアロケータにハード要件を課す**：ユニット起点はクラスごとにアライメントせねばならない——**データユニットとパックレコードユニットは 32768 アライメント、インデックスノードは 16384 アライメント** (3段階は確定項目 11 登録表の「ユニットサイズ / アライメント」列から取得)、さもなくばスキャナのステップ走査がズレを起こす。着地点は D3 (領域割り当て) のアロケータ実装であり、**テストにより強制的に違反可能な検査として作成せねばならない** (`.claude/rules/fs-design.md` ハード要件 2)：起点のアライメントが不正なユニットを作成した場合、走査再構築は必ず赤判定を出さねばならない。

**射程**：32768 単位ではステップ走査しない——奇数スロットに配置された 16 KiB ノードをすべて見落とすため。**これは D4 (チェックサム位置) 確定項目 7 によって実現されたものであり、確定項目 6 を覆すものではない**：D4 (チェックサム位置) 確定項目 7 が extent のディスク占有を**常時 32768** と定めた後、**ユニット境界は予測可能となった** (アロケータがユニットアライメントに従い割り当てる限り、プール内最小ユニットサイズ単位でステップ走査すれば全数ヒットする) ⇒ 「ブロックレベル」と「ユニットレベル」はこの値において合致し、確定項目 6 の結論はそのまま成立する。アロケータ側でコード 3 がどの区分で配置先を取得するかは D3 (領域割り当て) 確定項目 10 ⑤ により確定 (データユニットの区分に従い、起点 32768 アライメント)、`crates/singlefs-core/src/allocator.rs` はクラスごとにアライメント割り当てを行う；checker 側にはアライメント不正の赤判定が存在しない (`grep -rn "对齐\|align" crates/singlefs-checker/src` はゼロヒット)。

**根拠**：

- E86 (走査ステップと2つのユニットサイズ)：16384 ステップ走査ですべて回収可能、コストはデータユニットごとにペイロード内部に落ちるプローブが1回余計に生じ、magic とヘッダチェックサムにより拒絶されること；32768 ステップ走査では奇数スロットの全ノードを見落とす；ノードに 32K スロットのペアを専有させれば全数回収可能であるが、メタデータの占有空間が倍増する。
- 「ブロック = デバイス物理ブロック」を採用しない理由 (算術)：仮にブロック = 4096 バイト物理ブロックとした場合、32 KiB ユニットは**内容がバイト単位で同一のヘッダを 8 個**負担せねばならない；512 バイトディスクでは **64 個**となり、ヘッダ占有率は 15.6% に達し、同一ユニット内の各ヘッダで 5つ組の値は完全に同一となり純粋な冗長である。
- ユーザー確定 (ブロック = ユニット；ステップ走査の計測条件を 16384 に修正)、変更履歴に記録。

**未済の検査**：C86 (2つのユニットサイズと走査ステップの不整合)：ゴールデンイメージ上の赤緑対照およびアライメント不正ユニットの赤判定。

#### 確定項目 10：墓標は範囲レコード + パック共有ユニットを採用

**確定内容**：

- **レコード粒度**：オブジェクト全体の削除は**1つの範囲レコード** (オブジェクト ID + オブジェクト誕生世代 + 全範囲) を書き込む；truncate / punch などの部分削除は**範囲墓標を書き込まない** (範囲墓標は不採用)；ファイル短縮時は新規レコードを書き込まず、走査再構築時にデータユニットは再構築された inode レコードのオブジェクト誕生世代と size に基づき生死を判定する (解釈 Z、D8 (コアインデックス構造) 確定項目 6「再構築」の段落)。オブジェクト誕生世代は 5つ組と同一の計測条件とし、走査時にフィールドごとに照合；D8 (コアインデックス構造) 確定項目 6 はパブリッシュ済みの (inode 番号, オブジェクト誕生世代) が再利用されないことのみを保証し、クラッシュリカバリ後に番号は再発行されるため、同一番号を持つ2つのパブリッシュ済みオブジェクトの墓標はこれによって区別される。
- **キャリア**：墓標レコードは**共有の墓標ユニットにパック格納する** (コンテナは確定項目 11 登録表のコード 3 パックレコードユニット、レコードはパックレコードタイプ 1)、レコードごとに1ユニットを専有させない。
- **格納規則**：**墓標ユニットは世代ごとに格納する——1つの世代が終了した時点でその時点で開いていたユニットをクローズし、異なる世代のレコードを同一ユニット内に混在させない。** これは D2 (RAID ストライプポリシー)「ライフサイクルの異なる2つのオブジェクトに同一の物理マッピングユニットを共有させない」と同一の規律であり、墓標に適用したものである。
- **回収規則 (初期値、ユーザー計測条件：一旦このように記述し、追って実験)**：**墓標レコードはスナップショット deadlist と同一のペースで消去する**——1つの墓標は「それが指認する死亡が、それをなお参照しているすべてのルート (スナップショットおよび書き込み可能ヘッドを含む) によって見送られた」後に回収可能となる；量化子は「スナップショット」ではなく「ルート」を採る：クローンヘッドはヘッドでありスナップショットではないため、その内部で生存しており origin 側で後に削除されたオブジェクトについて、「スナップショット」量化子ではその kill レコードが回収可能となってしまい、クローンが参照する死体が恒常的に可読となってしまう。D5 (スナップショット / 領域会計メカニズム) のスナップショット破棄時における deadlist フィルタリングと同一の判定点であり、別途走査を立ち上げない。第1版では墓標を回収しない (確定項目 24)。

**射程**：パック化により回収は E80 (ストライプの部分解放) pin_stripe のピン留め形状 (回収粒度 = ユニット) と結合する。確定項目 11 以後はコンテナ数に基づいて解釈：同時に開いているコンテナ数 = ツリー × パックレコードタイプ × 生存世代 × 生存ヘッド (C106 (オープンコンテナ数の上限))、かつオープンコンテナは新規レコードが存在するパブリッシュ時のみ書き換えられ、削除のないパブリッシュ時はそのままコピーされる (確定項目 24；書き込み増幅の計算は C105 (パブリッシュ頻度に応じたパックコンテナ書き換えの書き込み増幅))。2つの連動：① D3 (領域割り当て) 確定項目 2 の墓標予約額は確定項目 10 の計測条件に従い再計算——予約されるのは**レコード**であり (1/815 ユニットの按分、第1版墓標レコードは 40 バイト、確定項目 24)、ユニット全体ではない、E82 (受入のインフライト合成) の「墓標マス」は粒度確定前の過大見積もりであった；② 墓標レコードの幅は固定値 40 バイトである (確定項目 24)。範囲粒度 (部分削除を連続する extent 範囲ごとに1レコードとする) は第1版では不採用：範囲墓標は脱落；解釈 Z はファイル短縮のみを扱い、punch 等の size を変更しない部分削除をどう記録するかは判決で触れられておらず未確定。

**根拠**：

- E83 (墓標の粒度)：4つの組み合わせ × 3つのワークロードの全容表——レコードごとのユニット専有は使用に耐えない形態であり、範囲レコード + パック共有ユニットが最も安価、大ファイルの truncate において差はさらに拡大。
- E84 (パック墓標のピン留め量)：世代混在格納時、スパースな削除が最悪ケースとなり、生存している墓標ユニットの大半がとうに回収可能なレコードで占められる；**世代ごとにユニットを分けて格納する (世代境界でオープンユニットをクローズする) ことでピン留め量は常にゼロとなり**、コストは生存世代ごとに1つの未充填オープンユニットを抱えるのみである。
- E104 (走査再構築における現行バージョン判定)：墓標がゲートに従い回収されない場合、旧バージョンが復活する——回収ゲートが荷重を受け持つ；量化子「ルート」は C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定に伴い緩和。
- ユーザー確定 (粒度とキャリア、格納規律、回収初期値)、変更履歴に記録。
- 40 バイト固定値、第1版未回収、番号再発行の記述：墓標判決 `research/prompts/m3-tomb-r1-main-verification.md` 第3節 W2 番号再発行、W3 回収の2行、`research/prompts/m3-delete-r3-main-verification.md` 第3節「墓標第1巡各行」の行。
- 部分削除での範囲墓標不採用、ファイル短縮での解釈 Z 採用：葉死亡判決 `research/prompts/m3-leaf-death-r3-main-verification.md` 第2節「解釈 Z による3組の新履歴への置換」「範囲墓標の最小改修法」の2行；ユーザー確定 2026-10-08 X2「解釈 Z (推奨)」、原文は変更履歴に記録。

**未済の検査**：回収判定基準は実験未済 (第1版は回収せず、レコードフィールド表は確定項目 24 に記載；回収ゲートの「データユニットが上書きまたは清掃された」次元は整理の実装待ち、C621 (整理実装時に改定すべき削除条項にリスト外の検査がない))；C105 (パブリッシュ頻度に応じたパックコンテナ書き換えの書き込み増幅)；C106 (オープンコンテナ数の上限)。

#### 確定項目 11：ユニットタイプタグの値域は登録表であり、パックレコードユニットは独立したクラス

**確定内容**：

**値域の登録位置は1箇所のみ——D18 (ブロックに含める情報) 確定項目 11 の本登録表である。** D9 (暗号化) 確定項目 6 の値域行、D18 (ブロックに含める情報) 確定項目 7 のフィールド表、D21 (権威状態と派生状態の境界) 確定項目 4、I-6.2 (平文ヘッダフィールドホワイトリスト)、D20 (荷重受け面: ユニットの原子性と自己完結性) の2クラス表、I-1 (ブロック自己記述) の免除行の6箇所はそれぞれ自身の次元の判定を保持し、**クラス名とコードは一律本表から取得する**。**凍結されるのは「コード → クラス」の列であり、2つの時点で凍結される**：コード 1 / 2 / 3 は共通プレフィックス内に実在する1バイトであり、最初のイメージが書き出された瞬間に凍結；コード 16–18 はディスク上には存在せず AAD の先頭バイトにのみ現れるため、暗号化が稼働開始した瞬間に凍結される (それ以前の変更はゼロコスト、以後の変更は AAD 構成変更となり、D9 (暗号化) 確定項目 6 に従いフォーマット断裂となる)。**一度使用したコードの再利用は禁止され、退役したコードの意味は永久に固定される**——D15 (フォーマット凍結ポリシー) の feature bit に対する規律と同型であり、理由も同一である (ディスク上に実在したバイト列であるため)。

| コード | クラス | ユニットサイズ / アライメント | ヘッダ参照表 | クラスアイデンティティセグメント | AAD 変型 | I-6.2 (平文ヘッダフィールドホワイトリスト) | 拡張ポイント | 取得後の内容列挙方法 |
|---|---|---|---|---|---|---|---|---|
| 0 | **無効** (割当禁止) | — | — | — | — | — | — | 検出次第破損と判定 |
| 1 | **データユニット** | 32768 / 32768 | D18 (ブロックに含める情報) 確定項目 7 データユニットクラス | 5つ組 + 誕生世代 + fsid + 書き込みシーケンス + ペイロード CRC (平文ヘッダ 105 / 予約領域込み 134) | D9 (暗号化) 確定項目 6 の5項目——引用時は D9 (暗号化) 確定項目 6 インデックス行の注釈も同時に引用要：5項目はスナップショット次元に対して盲目かつ沈黙であり、フィールド表は再開が未済；ツリー ID は読み取り側ツリーを採用 | データユニットクラス | 保持 (D21 (権威状態と派生状態の境界) 確定項目 4) | ペイロードはユーザーデータであり列挙しない |
| 2 | **インデックスノード** | 16384 / 16384 | D18 (ブロックに含める情報) 確定項目 7 インデックスノードクラス | key 幅 1 + ツリー ID + 階層 + key 区間 + 誕生世代 + fsid + 書き込みシーケンス 4 + 誕生シーケンス番号 4 + ペイロード CRC 4 + 予約 2 + エントリ数 2 + エントリ幅 2 (86 + 2k、予約領域込み 115 + 2k；オフセット表は確定項目 18 を参照) | **未確定、着地点なし** (D9 (暗号化) day-1 第 ③ 項)——C107 (インデックスノードの AAD アイデンティティが未着地) | メタデータクラス | 条件付き保持 (D21 (権威状態と派生状態の境界) 確定項目 4) | ノードフォーマットに従いエントリを解釈 |
| 3 | **パックレコードユニット** | 32768 / 32768 | 共通プレフィックス + D18 (ブロックに含める情報) 確定項目 11 クラスアイデンティティセグメント表 | 65 バイト ⇒ ヘッダ **107** (nonce / MAC 予約領域含まず、105 と同計測条件；予約領域込み 136) | (タグ, 誕生ツリー ID, パックレコードタイプ, コンテナ番号, コンテナ誕生世代)——ツリー次元は**誕生ツリー**を採用、すなわち D9 (暗号化) の再開後にコード 1 も採用すべきセマンティクス；E90 (ツリー ID の AAD 参入とヘッダ間共有) の2つのフォーマット上の意味 (ポインタが誕生 (ツリー, txg) を保持、クローン祖先表が認証済み構造に常駐) は当該再開に帰属し、それらの着地前はコード 3 の AAD は完全な契約を構成しない | メタデータクラス | 保持 (D21 (権威状態と派生状態の境界) 確定項目 8：ユニット末尾に密着、第1版 N = 0)；赤判定となる検査は C108 (拡張ポイント帰属がパックレコードユニットに対して判定なし) に未済 | レコード数 × レコード幅で固定幅スライス |
| 4–15 | 将来のコンテンツクラス用予約 | | | | | | | 未登録 ⇒ 拒絶；新規クラス追加は incompat ビットを1つ占有 (D15 (フォーマット凍結ポリシー) 判定表) |
| 16 | ルートレコード (ルートスロット) | スロット幅 | D22 (ユニット原子性の合成方法) 確定項目 7 | 独自 | D22 (ユニット原子性の合成方法) / D9 (暗号化) に帰属 | 適用外 (自己証明) | 不保持 | AAD ドメインセパレータとしてのみ機能 |
| 17 | journal レコード | レコード | D23 (journal の役割とフォーマット) 確定項目 1 の A 条 + 確定項目 4 の 487 バイトヘッダ (確定項目 15 の新ルートセグメント 364、fsid 8、MAC 16 を含む) | 独自 | D23 (journal の役割とフォーマット) / D9 (暗号化) に帰属 | 適用外 | 不保持 | AAD ドメインセパレータとしてのみ機能 |
| 18 | システム構成スロット | スロット幅 | D22 (ユニット原子性の合成方法) 確定項目 9 | 独自 | プレースホルダ、未確定 | 適用外 | 不保持 | AAD ドメインセパレータとしてのみ機能 |
| 19–31 | 将来の自己証明構造用予約 | | | | | | | 16–18 と同一の凍結時点 |
| 32–255 | 予約；割り当て規則は確定項目 11 では定めず、将来のいかなる割り当ても D15 (フォーマット凍結ポリシー) 判定表を通過する | | | | | | | |

- **D9 (暗号化) 確定項目 6 の4つの名称の着地点**：データ extent → 1；btree ノード → 2；ルートレコード → 16；**会計ユニット → 2**——D9 (暗号化) 確定項目 6 がタグを設けた理由は「異なるユニットクラスはフィールド集合が異なり……同一のバイト列にシリアライズされ得る」ためであるが、会計ツリーノードと他のインデックスノードはクラスアイデンティティセグメントのフィールド集合が同一である ⇒ このペアにおいてドメイン分離は何も購入しない、「会計」はツリー ID が与えるロールである。両者は状態分類が異なる (会計は権威状態、C102 (インデックスノードヘッダに予約領域がなくフィールド追加は compat に非ず) に記録)、状態分類は I-6.2 (平文ヘッダフィールドホワイトリスト) と拡張ポイントの判定基準ではない；将来状態分類に基づき会計を別クラスとして独立させる場合、D15 (フォーマット凍結ポリシー)「既存フィールドの意味変更」に従い incompat かつコード 2 は再利用されない。D9 (暗号化) day-1 第 ① 項の射程は値域とともに縮小：縮小されたのは「会計 / 非会計の区別をツリー ID から推論してはならない」という一文のみであり——コード 2 自体は明示的なタグ値であり、① は他の各コードに対してそのまま成立する。D22 (ユニット原子性の合成方法) 確定項目 3 の根拠にある「会計ユニット」は以後ロール名 (会計ツリーノード) であり、クラス名ではない。**D18 (ブロックに含める情報) 確定項目 8 の「墓標」**：コンテナ = コード 3、レコード = パックレコードタイプ 1。自己証明ユニットもコードを占有するのは単射エンコーディングが必要なためであり、同一の名前空間がそれを達成する最も安価な手段である (4バイト magic に対して1バイト)。
- **タグの幅と位置**：ユニットタイプタグ = 共通プレフィックス**オフセット 6** の1バイト (flags 2バイトはそれぞれ独立に定義され、エンディアンに依存しない)；**オフセット 7 は 8 個の flag ビットであり、第1版は常時 0、読者が非 0 ビットに遭遇した場合は一律拒絶する**——これは前方互換性ポリシーであり、破損検出ではない (ヘッダチェックサムがすでにカバーしている)：これら 8 ビットに「未知のビットを無視する」経路を残さない ⇒ 将来いずれかのビットに意味を付与する際は D15 (フォーマット凍結ポリシー)「既存フィールドの意味変更」に従い incompat と判定；判定表の1行目の意味における予約領域 (フィールドを収容できない) ではなく、C102 (インデックスノードヘッダに予約領域がなくフィールド追加は compat に非ず) にこの旨を明記せねばならない。ユニットヘッダの flags に対してのみこの側を採る；D8 (コアインデックス構造) 確定項目 6 の inode 属性存在ビットは別のビット空間であり、そのジレンマは確定項目 11 では解決しない。共通プレフィックス 42 は一文字も変更しない。データユニット 5つ組の先頭バイトはオフセット 6 と等しくなければならず、不等は破損と判定。AAD のカノニカルエンコーディングにおいてタグコードは最初のバイトであり、全クラス共通である。256個の値のうち7個を登録、残り249個。
- **5つ組 33 の内訳：タグ 1 + ツリー ID 8 + オブジェクト ID 8 + オブジェクト誕生世代 8 + アンカーオフセット 8** ([verification-build.md](../verification-build.md) 第8問は閉鎖)。**代替案は論証されておらず、ユーザーが合計 33 と4つの 8 の既存の出典に基づき直接確定**：ツリー ID 4、誕生世代 6 (48 ビットは E44 (シーケンス番号ビット幅の実機測定とコスト) の 2785 パブリッシュ/秒で3000年使用可能)、アンカーオフセットのユニット粒度 6 など、いずれも 33 を変更することになり、確定後はもはや選択肢ではない。**6つ組の2つの候補は今回の確定には含まれない**：`locality_id` (確定項目 3 の提案、ユーザー確定で却下) と誕生スナップショット次元 (D9 (暗号化) 確定項目 6 インデックス行の未済再開) は、いずれも AAD 構成の恒久的な変更であり、それぞれの項目に帰属する。
- **パックレコードユニット (コード 3) は独立したクラス**：D21 (権威状態と派生状態の境界) ハード制約 6 (計算不能な権威値は再構築可能なコンテナに置いてはならない) によりコード 2 を除外；5つ組のオブジェクト ID は1つしか書き込めない ⇒ コード 1 を除外——墓標が指認するのは死者であり、**いかなる**インデックスももはやそれを保持していない (確定項目 3 当時の「同一情報がすでに平文逆引きインデックスに横たわっている」という前提は平文側の廃止に伴い消失したが、結論はそれに依存しない)。

**コード 3 のクラスアイデンティティセグメント (65 バイト、ヘッダ 107)**：

| オフセット | フィールド | 幅 | 理由 |
|---|---|---|---|
| 42 | ユニットタイプタグ | 1 | オフセット 6 の暗号文側コピー (I-6.2 (平文ヘッダフィールドホワイトリスト) 追記はホワイトリスト項目が暗号文側に権威あるコピーを持つことを要求)。⚠️ コード 2 にはこのコピーが存在しない、C111 (クラスタグの暗号文側コピー欠落) |
| 43 | 誕生ツリー ID | 8 | オブジェクト番号は keyspace をまたぐと必然的に衝突する (D9 (暗号化) 確定項目 6)；1つのコンテナは1本のツリーのレコードのみを格納 |
| 51 | パックレコードタイプ | 2 | 第2段階登録表のコード；1つのコンテナは1種類のみ格納 |
| 53 | コンテナ番号 | 8 | 割り当て規則はパックレコードタイプごとに分割：タイプ 1 (墓標) はツリー内単調割り当て (C103 (コンテナインデックスの着地点なし) の方式、ロールバックでもアイデンティティを破壊しない)；タイプ 2 (inode レコード) = 分割をトリガーした新レコードの inode 番号、アロケータ不要 (D8 (コアインデックス構造) 確定項目 6)。両クラスともパブリッシュ非依存であり、コンテナが COW で書き換えられても不変；4 バイトは E44 (シーケンス番号ビット幅の実機測定とコスト) の 2785 パブリッシュ/秒で 17 日で周回 |
| 61 | コンテナ誕生世代 | 8 | コンテナ番号を割り当てたパブリッシュ時の checkpoint_txg；コンテナ番号と合わせて安定したアイデンティティを形成し、データユニットの (オブジェクト ID, オブジェクト誕生世代) と同一の役割分担 |
| 69 | レコード数 | 2 | ≤ 1359 (予約領域込みヘッダ 136、確定項目 16；最も狭いレコードはタイプ 5 系譜レコードの幅 24、確定項目 24) |
| 71 | レコード幅 | 2 | 登録表で規定し、ヘッダ内にも複製：レコードタイプを解釈できない読者であってもヘッダ正当性条件を判定可能とするため (D23 (journal の役割とフォーマット) 確定項目 1 の B 条と同型) |
| 73 | 誕生世代 | 8 | 今回の書き込みのみを記録 (D18 (ブロックに含める情報) 確定項目 7 でヘッダに差し戻された3つの理由) |
| 81 | fsid | 8 | D18 (ブロックに含める情報) 確定項目 7 と同一の理由 |
| 89 | ペイロードチェックサム | 4 | CRC32C、オフセット 107 からユニット末尾 (nonce / MAC 予約領域 + レコード領域 + パディング) をカバー。ユニット全体のチェックサム / MAC は親ポインタに常駐するが (D4 (チェックサム位置))、走査取得時は親ポインタが存在せず、ヘッダチェックサムはヘッダ完全性の防衛線にすぎない (E76 (ペイロードチェックサムの識別力)：ヘッダが着地しペイロードが半分着地した場合、ヘッダチェックサム単体では判定不能)。幅は D23 (journal の役割とフォーマット) 確定項目 13 に準拠 |
| 93 | 書き込みシーケンス (インスタンス代号 4 + トランザクション番号下位 48 ビット) | 10 | C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P1、コード 1 と同一フィールド；コンテナはチェックポイントの固定点でのみ書き出され、トランザクション番号はそのチェックポイント内で割り当て済みの最大トランザクション番号を採用；**コード 3 においてこれは同一チェックポイント内のタイブレークとしてのみ機能する** (I-1.8 (マージ後バージョン全順序) のコード 3 に対する全順序キーは (誕生世代, インスタンス代号, トランザクション番号) の3つ組)；コード 2 の書き込みシーケンスは確定項目 12 に従い 4 バイト (インスタンス代号のみ) に縮小済みであり、P3 および I-1.8 (マージ後バージョン全順序) には入らない；コンテナには個別にインスタンス代号を追加せず、書き込みシーケンス内の 4 バイトがそれを兼ねる (D8 (コアインデックス構造) 確定項目 6 の記述はこれにより再開) |
| 103 | 誕生シーケンス番号 | 4 | D19 (ブロックポインタ構造と幅予算) 確定項目 6：コード 3 の中央マッピング key = クラスタグ + 誕生ツリー (オフセット 43) + 誕生 txg (誕生世代、オフセット 73) + インスタンス代号 (書き込みシーケンス先頭 4 バイト、オフセット 93) + 誕生シーケンス番号；key の一区間としてのみ機能し、最新選択には用いない；位置は書き込みシーケンスの直後 (C288 (マッピング key 確定に伴う連動変更が他決定へ未反映) 第 ① 条で甲を採用) |

- nonce / MAC / アルゴリズムタイプの 29 バイトはオフセット 107 から始まり、予約領域を含むコード 3 ヘッダは 136 である (確定項目 14 / 確定項目 16)。
- **ヘッダチェックサムのカバー範囲 (権威ある条目 I-2.4 (ヘッダチェックサムカバー範囲))**：オフセット 0 から当該クラスの平文ヘッダの最後のバイトまでをカバーし、終端点は D18 (ブロックに含める情報) 確定項目 7 / 確定項目 11 / 確定項目 18 のフィールド表に従う (現行初期値：コード 1 はオフセット 104 まで、コード 2 はオフセット 85 + 2 × key 幅まで、コード 3 はオフセット 106 まで)、ヘッダチェックサムフィールド自身は 0 として関与；29 バイトの nonce / MAC / アルゴリズムタイプ予約領域はヘッダチェックサムカバー範囲外である (I-2.4 (ヘッダチェックサムカバー範囲))；暗号化有効後 (システム構成レベル、全か無か、I-6.4 (未暗号化ブロックの禁止)) メタデータクラスのアイデンティティセグメントは暗号文へ入り MAC で保護されるため、ヘッダチェックサムのカバー範囲 = I-6.2 (平文ヘッダフィールドホワイトリスト) の当該クラスホワイトリスト内の全平文ヘッダフィールドとなり、別途列挙しない——**同一フォーマットバージョン下においてシステム構成の暗号化宣言に従い2種類に分かれ、判別子は D9 (暗号化) 確定項目 10 の暗号化ビットである**。⇒ 第1版においてペイロードチェックサムはヘッダチェックサムのカバー範囲内に収まる (D23 (journal の役割とフォーマット) 確定項目 13 の着地要件)。
- **宣言長の計測条件はコード 1 と同一**：宣言長 = ヘッダ以降の純ペイロードバイト数、パディング起点 = ヘッダ + 宣言長、宣言長 ≤ 32768 − 136 (29 バイトの予約領域を含むヘッダ、確定項目 16)。
- **レコードレベルではフィールドを保持しない**：未コミット時の断裂書き込みは I-1.2 (ブロックヘッダ書き込みシーケンスパブリッシュ済み) によりゴミと判定；パブリッシュ後のペイロード破損はペイロードチェックサム (走査パス) および親ポインタチェックサム / MAC (通常パス) により判定；回収粒度 = ユニットであり、生存ビットに対象は存在しない；1つのコンテナに1種類の固定幅レコードを格納するため、タイプと長さはヘッダに1つあれば十分である。**I-2.3 (パディングバイトはゼロかつチェックサムに関与) の射程をコード 3 へ拡張**。判定順序：ヘッダチェックサム → クラスアイデンティティセグメント → I-1.2 (ブロックヘッダ書き込みシーケンスパブリッシュ済み) / I-1.3 (ブロックヘッダツリー ID 一致) / I-1.4 (ブロックヘッダ fsid 一致) → レコード数 × レコード幅 ≤ 宣言長 → ペイロードチェックサム → 各レコード順次、I-1.7 (パックコンテナ正当性と判定順序) に記載。
- **I-6.2 (平文ヘッダフィールドホワイトリスト) がメタデータクラスに分類される理由**：ホワイトリストが区分するのは鍵を持たない側が何を読み取れるかであり、状態分類ではない——会計ツリーノードは権威状態であるがメタデータクラスに属する；鍵を持たない側は D9 (暗号化) 確定項目 5 に従い検出と運搬のみを行うため、コンテナのアイデンティティは無用である。
- **コンテナの COW 書き換えとインデックス**：オープンコンテナはそれが属する世代内で新規レコードが存在するパブリッシュ時のみ書き換えられ (確定項目 24)、新規ユニット、新規誕生世代となり、アイデンティティは不変である。**コンテナインデックスはパックユニットポインタヘッダ (MAC / nonce / アルゴリズムタイプ) および位置ヒントの保持者であり、権威ある位置は中央マッピングにある (タイプ 4 インスタンス表は例外：その物理ポインタはルートレコードが直接保持、D22 (ユニット原子性の合成方法) 確定項目 7)；他のツリーはアイデンティティによりコンテナを参照する**——コンテナ更新のコストはパックレコードタイプごとに分割：タイプ 1 (墓標) = それを COW + 墓標ツリー内でそれを指すコード 2 ノードを1エントリ変更 (確定項目 24；C103 (コンテナインデックスの着地点なし) はストライプメンバ表の半分のみ残る)；タイプ 2 (inode レコード) = それを COW + 全祖先を変更 (コンテナインデックスは inode ツリーの内部ノードそのものである、D8 (コアインデックス構造) 確定項目 6)、これは inode パック (D8 (コアインデックス構造) 確定項目 6) と D14 (二重トラック (大小ファイル / 永続一時)) で否決されたファイル間末尾パックとの相違点である。旧版コンテナは通常の COW 解放パスを通り、墓標は書き込まない。コンテナインデックスが専有する物理ポインタと MAC はいずれも派生値である (MAC = f(鍵, 暗号文, AAD) であり再計算可能；再計算後の認証力 = 走査されたペイロードへの信頼であり、まさに D18 (ブロックに含める情報) 確定項目 5 の縮退セマンティクスであり、コンテナインデックスと extent ツリー内のデータポインタを一等親として同等に扱う) ⇒ D21 (権威状態と派生状態の境界) ハード制約 6 に抵触しない。**コード 2 は走査再構築のバージョン選択規則に入らない**：inode ツリーの内部ノードとルートは派生状態であり、再構築は葉から再生成する (葉はコード 3 であり、自前でペイロードチェックサムを保持、D8 (コアインデックス構造) 確定項目 6)；会計ツリーのノードは権威状態であるが、そのエントリは冪等な完全値を採用しており (D8 (コアインデックス構造) 確定項目 1)、複数版は**エントリレベルでマージされる** (D5 (スナップショット / 領域会計メカニズム) 確定項目 2：単調統計量は最大値を採用、非単調統計量はエントリ自身の seq により「後者」を決定)、**コンテナレベルでのバージョン選択は不要である**；ユニットクラスコードは状態分類の判定基準ではない (D21 (権威状態と派生状態の境界))。**走査再構築において同一アイデンティティの複数版コンテナが同時に見出された場合、C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定の規則に従い判定する**：まずインスタンス表のパブリッシュ済み述語により未パブリッシュのバージョンを除外し (確定 P2)、次いで当該ルートから可視な候補の中で**当該クラスの最新選択キー**が最大であるものを選択する (確定 P3：コード 1 は書き込みシーケンスを採用、コード 3 は (誕生世代, インスタンス代号, トランザクション番号) の3つ組を採用——誕生世代を先頭とするのは、同一インスタンスで2回連続してチェックポイントで新規トランザクション番号が割り当てられない場合、2つのバージョンの書き込みシーケンスがバイト単位で同一となるため；トランザクション番号は同一チェックポイント内のタイブレークとして末尾に残る；固定点は反復し、同一コンテナが1つのチェックポイント内で1度しか書き込まれない保証はないため)；2つのバージョンが同時に異なるヘッドに属することも許容される；通常のパスは依然として D3 (領域割り当て) 確定項目 1 の述語により判定される。誕生世代は最新選択キーとしない：書き込みシーケンスが獲得するのはパブリッシュ済み性とタイムラインであり (確定項目 1 の射程)、同一 txg 内でトランザクションをまたいで同一 key を変更する際の順序付けを副次的に担う。コンテナはパブリッシュごとに1回書き換えられ、本マスの重みはデータ extent より遥かに大きい (1000 ファイル / 100 回の fsync ⇒ 1つのアイデンティティに対して 100 個の物理バージョン)。
- **4つの格納規律**：① 1コンテナに1種類のパックレコードタイプ；② 1コンテナに1世代——確定項目 10 が墓標に対して定めた規律を世代ごとに回収されるパックレコードタイプへ拡張 (タイプ 1 のみ；タイプ 2 のレコードは1件ずつ死亡し、コンテナは B ツリーの分割 / 統合により回収され、1枚の葉が多数の世代を生きる、D8 (コアインデックス構造) 確定項目 6) (D2 (RAID ストライプポリシー) 書き込み粒度第 2 条)；世代はフィールドとして保持せず、= 生存スナップショットリスト (I-3.7 (生存スナップショット列挙可能)) 内でコンテナ誕生世代を含む区間であり、**回収判定時のみ使用し、走査再構築では使用しない**；スナップショットパブリッシュ前に現在オープンなコンテナをクローズして書き出さねばならない (D16 (パブリッシュセマンティクス) 確定項目 7 と同一の順序；同様にタイプ 1 のみ——タイプ 2 の葉は世代の意味でのオープンコンテナではなく、スナップショットはそれを共有するのみ)；③ 1コンテナに1本のツリー；④ レコードは固定幅、レコード数 × レコード幅 ≤ 宣言長、取得時に判定。
- **パックレコードタイプは第2段階の登録表である**：0 無効 (コンテナヘッダ内の 0 は破損と判定；同一バイト列がアイデンティティ参照形態として inode ツリー内部ノードエントリ内に現れる場合、0 は「子ノードがコード 2 インデックスノードである」ことを示し、D8 (コアインデックス構造) 確定項目 6 および I-9.2 (エントリのアイデンティティと子のヘッダの一致) に登録、コンテナヘッダの文脈とは判定パスを共有しない)；**1 墓標範囲レコード** (幅 40、フィールド表は確定項目 24 を参照；32 KiB コンテナに 815 件 (レコード領域起点 136)；E83 (墓標の粒度) および E102 (ユニットクラス登録表とパックレコードユニット) は第2回で 40 バイトにて再実行済み、E84 (パック墓標のピン留め量) は依然 56 バイトで計算されており、再実行は C691 (パック墓標ハーネスが容量を 583 に固定) に未済)；**2 inode レコード** (有効化済み、D8 (コアインデックス構造) 確定項目 6：固定長 140、1コンテナに 233 件、コンテナは inode ツリーの葉そのもの)；**3 退役レコード** (C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P5 / P6：葉コンテナのアイデンティティが消失した際に退役——削除により空となり除去されたコンテナ、統合時に吸収された右半分、再生時の旧アイデンティティ；レコード = 退役対象の (誕生ツリー, タイプ, コンテナ番号, 誕生世代) 4つ組 + 死亡 txg (アイデンティティが消失したパブリッシュ時の checkpoint_txg)、書き込みシーケンスは含めない；専用のコンテナに常駐し、コンテナは専用の退役ツリーに入り、コンテナ番号はタイプ 1 と同一方式で単調割り当て；読み取り側は死亡 txg と系譜によりフィルタ、退役ツリー、墓標ツリー、系譜ツリー等の現行追従ツリーは C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P2 のみに従いバージョン選択、管理者ロールバック時の退役ツリーは現行版に追従し R_old は取得しない (D23 (journal の役割とフォーマット) 確定項目 14 は変更しない)；退役ツリーの種別コード 13 (D8 (コアインデックス構造) 確定項目 9)、ツリー ID 65548 (D8 (コアインデックス構造) 確定項目 11)、ヘッド ID は inode ツリーのツリー ID を書き込み (D8 (コアインデックス構造) 確定項目 8)、ツリーテーブルの末尾に配置；レコード幅 34、32 KiB コンテナに ⌊(32768 − 136) ÷ 34⌋ = 959 件格納；フィールド表 (リトルエンディアン、パディングなし、flags なし、予約なし)：退役対象アイデンティティの誕生ツリー 8 | パックレコードタイプ 2 | コンテナ番号 8 | コンテナ誕生世代 8 | 死亡 txg 8、先頭4セグメントはアイデンティティ参照と同一順序・同一幅)；**4 インスタンス表** (C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P2、フィールド表は「パックレコードタイプ 4 (インスタンス表) は独立したレコードフォーマット」の条項を参照)；**5 系譜レコード** (幅 24、フィールド表は確定項目 24 を参照)；6–65535 予約。タイプ 3 / 4 は D15 (フォーマット凍結ポリシー) に従いデフォルトで incompat、凍結前はゼロコスト。
- **タイプ 1 コンテナは書き込み側ツリーが専有** (C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P6)：スナップショットパブリッシュ前に現在オープンなコンテナはまずクローズして書き出され (確定項目 10 の格納規律)、クローンは origin のタイプ 1 コンテナを自身のコンテナインデックスに取り込まず、回収 (書き換え) は書き込み側ツリーのみが実行し、他のヘッドは走査再構築時に祖先表を介してそれらを読み出す；クローンヘッドのタイプ 1 コンテナ番号は 1 からツリー内で単調に割り当てられ、アイデンティティは誕生ツリーにより分離される ⇒ タイプ 1 はヘッド間をまたいで COW されることはなく、ヘッドをまたいだ初回 COW による再生はタイプ 2 にのみ適用される。**他のヘッドが祖先表によりそれらを読み出せる前提として、分岐点スナップショットが参照するバージョンが永久に解放されないことが必要である** (D6 (スナップショット実装モデル) 確定に伴い連動発動した第2箇所：分岐点スナップショットの破棄禁止)——そして D6 (スナップショット実装モデル) の9つの採用条件の第5条は、それが機能的欠落により回避されたものであり満足されたものではないと明記しているため、「分岐点スナップショット破棄可能 / 書き込み可能ヘッド破棄可能」は C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定の前提条件であり、それと同時に失効する。コスト：分岐点時点で生存しており、その後に origin 側で削除されたオブジェクトについて、その kill レコードは永久に回収不能 (ゲートが永久に開かない) となり、墓標コンテナは単調増加して縮小不能となる——C105 (パブリッシュ頻度に応じたパックコンテナ書き換えの書き込み増幅) および C106 (オープンコンテナ数の上限) 以外の第3の負債である。
- **パックレコードタイプ 4 (インスタンス表) は独立したレコードフォーマットであり、「1コンテナに1種類の固定幅レコード」のモデルを適用しない**：1レコードの幅は常に 88 (チェーンポインタレコードと行レコードは同一幅)、先頭バイト `kind` により2種類に分かれる——
  - `kind` = 0 行レコード：`kind 1 | インスタンス代号 4 | 選択されたルートの checkpoint_txg 8 | 当該インスタンスに属する最大適用済みトランザクション番号 W 8 | flags 1 (第1版は常時 0、非 0 は拒絶；bit0 退役、未回収——かつてロールバック行を示した、以後は別の意味を付与せず、ユニットタイプコード「一度使用したら再利用禁止」と同一の規律) | 予約 66`；
  <!-- format-const: INSTANCE_ROW_BYTES = 88 -->
  <!-- format-const: INSTANCE_ROW_RESERVED = 66 -->
  <!-- format-const: INSTANCE_TABLE_PAGE_RECORDS = 370 -->
  - `kind` = 1 チェーンポインタレコード (常にスライスの最後のエントリ)：`kind 1 | 次スライスの有無 1 | 位置ポインタ 86 (次スライスなしの場合はゼロクリアのプレースホルダ；ルートレコード内のインスタンス表ユニットポインタと同型、D19 (ブロックポインタ構造と幅予算) 確定項目 7 / 8) | 予約 0`。パックレコードは固定長であり、kind 1 と kind 0 は同一幅でなければならない (`INSTANCE_ROW_BYTES`)；予約がゼロクリアされた後、このレコードに余白は存在しない：ポインタがさらに1バイト広くなれば、`INSTANCE_ROW_BYTES` も連動して増加し、1スライスのエントリ数も連動して変動する。
  - `kind` が 0 でも 1 でもないレコード：当該スライスは解釈不能 (テーブル可読不能)、行レコード flags の非 0 拒絶やユニットタイプタグの未登録と同型；チェーンポインタレコードがスライスの最後のエントリでない場合、または最後のエントリが正当なチェーンポインタレコードでない場合 (次スライスありとしつつ位置ポインタヘッダセグメントの一部が第1版の値でない場合を含む：MAC / nonce、アルゴリズムタイプ、圧縮アルゴリズムコードと圧縮後長さ、extent オフセット；第1版の値は D19 (ブロックポインタ構造と幅予算) 確定項目 3 を参照)、同様に解釈不能。検査は I-3.14 (インスタンス表の各スライスレコード形態正当)。
  - **アイデンティティと容量**：4つ組 (誕生ツリー 0, タイプ 4, コンテナ番号 = スライス番号 (0から), 誕生世代 0)；誕生ツリー 0 は D5 (スナップショット / 領域会計メカニズム) が「全プール / 帰属なし」行に与えた規約と同一の数値；いかなるツリーにも属さず、コンテナインデックスにも入らず、物理ポインタはルートレコードが直接保持する (D22 (ユニット原子性の合成方法) 確定項目 7)。1スライスは ⌊(32768 − 136) / 88⌋ = 370 レコード (チェーンポインタを含み、データ行 369；行数が 369 を超える場合は1スライスに 369 行を満杯にしてから次スライスを開き、最終スライスに残りを格納；136 は nonce / MAC / アルゴリズムタイプ予約領域を含むコード 3 ヘッダ、確定項目 14 / 確定項目 16)；mkfs は1スライスの空テーブルを書き出し、レコード数は 1 (唯一のエントリは「次スライスなし」のチェーンポインタレコード)。
  - **行の書き込み方法** (C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P2)：インスタンス代号ごとに一意、後勝ち上書き、リカバリ時およびインスタンス切り替え時のみ書き込み (管理者ロールバックはマウント中の前方パブリッシュであり、行を書き込まない、D23 (journal の役割とフォーマット) 確定項目 14)、最初の新ルートと同一のパブリッシュ——**書き込み可能マウントごとに必ず行を書き込む** (インスタンス 0 は書き込まない；行を書き込むパブリッシュは新インスタンスの初回パブリッシュであり、新インスタンスの各ルートが参照するインスタンス表はすでに [max(選択ルートのインスタンス, 1), 新インスタンス) の行を含む (清掃パブリッシュで削除された行を除く)；行書き込み前に F 引き上げの空パブリッシュは行わず、行書き込み時のメタデータは切り替え予約領域を使用、D28 (マウント時コミット量) 確定項目 3)；行書き込みごとにチェーン全体を COW 書き換え。
  - **リカバリ時に書き込む行**：リカバリは選択されたルートが指すバージョンのテーブル上で [max(選択ルートのインスタンス, 1), 新インスタンス) の各インスタンスに対して1行書き込む——選択ルートのインスタンスには (i, **再生後ルートの checkpoint_txg**, **インスタンス i に属し今回の再生により適用された最大トランザクション番号**) を書き込み、選択ルートのインスタンスと新インスタンスの間に厳密に位置するインスタンスには (i, 0, 0) を書き込む——jsn = インスタンス代号 32 ビット + カウンタ 48 ビット (D23 (journal の役割とフォーマット) 確定項目 9) であり、再生は jsn に従い厳密に連続し欠番で停止 ⇒ いかなる再生もインスタンス境界をまたぐことはない ⇒ **同一のリカバリで書き出された一連の行において、W が非ゼロとなり得るのは選択ルートのインスタンスのみである；同一の切り替えで書き出された一連の行において、W が非ゼロとなり得るのは現状維持トランザクションの書き込みシーケンス所有者のみである；残りは常時 0** (「同一の書き込みで書き出された一連」に限定：その後のリカバリは [新選択ルートのインスタンス, 新インスタンス) にのみ行を書き込み、旧行は遡及変更しないため、テーブル上では異なるリカバリまたは切り替えに由来する W ≠ 0 の複数行の存在が許容される)。リカバリは常に最新で可読かつ自己証明を通過したルートを選択し、それは最新パブリッシュ済みルートでない場合がある (スロット破損時は1つ後退し、以後のレコードが繋がらなければ1件も適用されず W = 0)；1件も適用されなかった場合は (i, 選択ルート txg, 0) を書き込む。
  - **切り替え時に書き込む行**：インスタンス切り替え (D23 (journal の役割とフォーマット) 確定項目 14 の注) はクラッシュ行と同様であるが、**選択ルートは D23 (journal の役割とフォーマット) 確定項目 14 インスタンス切り替えの一文に従い取得する**——ディスクを再読み込みしてルート選択を行い、呼び出し側メモリ内のルートは採用しない (C458 (インスタンス切り替えでディスク再読込せずメモリ内ルートを採用))；切り替えは再生を行わず、行は [当該ルートのインスタンス, 新インスタンス) にのみ配置され、各インスタンス1行、①–④ のうち最初に適用可能なものを採用：① 再発行されたパブリッシュ自身が書き込んだ行をそのまま再発行 (行書き込み時のリカバリ行および中間インスタンス行)；② 現状維持トランザクションの書き込みシーケンスインスタンス k には (k, k が最後にパブリッシュしたルートの txg (未パブリッシュ時 0), 現状維持された k に属する最大トランザクション番号) を書き込み；③ 選択ルートのインスタンスには (i, 当該ルートの txg, 0) を書き込み；④ 残りは (i, 0, 0) を書き込み——再発行されたチェックポイント内にトランザクション番号非 0 のレコードが存在しない場合、② は1件も適用されない。
  - **W は正確なプレフィックスである**：トランザクション番号はインスタンス内で単調であり、レコードはトランザクション番号順に追加される (第1版の直列コミット下で常時成立)；トランザクションがユニット書き込みを発行した後に失敗するとインスタンス切り替えとなり、より大きな番号のレコードは1件も追加されない (D23 (journal の役割とフォーマット) 確定項目 7 の注)。
  - **パブリッシュ済み述語 (大域的)**：i_now = マウントされたルートのインスタンス、ユニット書き込みシーケンス (i, n)、誕生世代 b——i < i_now かつ行なし ⇒ パブリッシュ済み；行 (i, T_pub, W) が存在する場合：コード 1 ⇒ b ≤ T_pub ∨ n ≤ W (選択ルート内、または再生により適用)、コード 2 / 3 ⇒ b ≤ T_pub (再生後ルート以降の固定点ユニットは一律未パブリッシュであり、リカバリインスタンスがそれらを書き直す；再生により適用されたパブリッシュの固定点ユニットはレコード内の新ルートセグメントにより参照され (D23 (journal の役割とフォーマット) 確定項目 15)、T_pub = 再生後ルートの txg によりパブリッシュ済みと判定；コード 3 の 6 バイトトランザクション番号はパブリッシュ済み述語に入らない——同一チェックポイント内のタイブレークとしてのみ機能する (I-1.8 (マージ後バージョン全順序) のコード 3 に対する全順序キーは (誕生世代, インスタンス代号, トランザクション番号) の3つ組、C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定第9版)；コード 2 の書き込みシーケンスはインスタンス代号のみに縮小済み (確定項目 12))；i == i_now ⇒ b ≤ マウントされたルートの txg；i > i_now ⇒ 破損と判定。「行なし ⇒ パブリッシュ済み」は同一タイムラインに対してのみ成立。
  - **回収**：行が削除可能となるのは、1巡の清掃により**プール内のすべての配置先**に対して判定が得られ (読み取り成功かつ行に従い判定、ヘッダ消去済み、上書き済み)、かつその中に当該インスタンスの未パブリッシュユニットが存在せず、当該清掃において読み取り失敗が1度も生じず (未読 ≠ 非存在、セクタ次元；ヘッダ消去前の再読み込みも含む：再読み込みでデバイスエラー報告または読み戻しが当該ヘッダと異なる場合は今回の清掃では削除しない、ユーザー 2026-10-04 確定、ゼロ巡)、全デバイスがオンラインであり (デバイス次元)、かつルートリング内に当該インスタンスがパブリッシュしたルートが存在しない場合である——ルートリングの全スロットが読み取り成功して初めて「存在しない」と判定され、読み取れないスロットは「当該インスタンスのルートが存在する可能性あり」として扱われる (一時的に読み取れないルートスロットにより行が削除されると、スロット復旧後に放棄されたルートがロールバック候補集合に戻った際、それが参照するユニットが清掃によりヘッダ消去されてしまうため；コストは C597 (インスタンス表行が単調増加で削除条項なし) が監視)。読み取り成功した全ゼロルートスロットも読み取り不能と同様に扱い (「当該インスタンスのルートが存在する可能性あり」)、今回は削除しない；残りのスロットは自己証明を通過せねばならず、不合格なら1度再読み込みし、なお不合格なら今回は削除しない。**行削除は専用の清掃パブリッシュに配置する**：ヘッダ消去がすべて FUA 復帰し、バリアを1度通過した後に今回の清掃パブリッシュを発行；読み取り失敗やデバイス不揃いの場合は削除しない。**順序は判定が先、消去が後**：まずルートリングの各スロットが自己証明を通過し、リング内に当該インスタンスのルートが存在しないことを判定；合格して初めて消去し、消去してから判定することは禁止。**清掃パブリッシュの形態**：トランザクション番号 0、W には入らず、インスタンス表チェーン全体を書き換え、マウント受入計算式は判定せず、条件を満たす全行を一括削除；専用予約は設けず、清掃パブリッシュの配置先が拒否された場合、今回の清掃では行を削除せず、マウントは通常通り書き込み可能として復帰する。**誰がいつ実行するか**：書き込み可能マウントの末尾 (行書き込み、ウォームアップ、F 引き上げ、削除継続処理の後、制御返却前) に1度判定。スイッチがオフの場合、インスタンス表行数が閾値を超えていれば当該マウント内で直ちに判定・消去・清掃パブリッシュ発行を行い、超えていなければ走査しない；スイッチがオンの場合、マウント中のバックグラウンド清掃に委ね、バックグラウンド消去前に同一シリアルセグメント内で当該スロットのヘッダを再読み込みし、依然として候補行の未パブリッシュユニットである場合のみ消去する。閾値は設定項目であり、デフォルト値は別途決定 (C690 (清掃トリガー閾値のデフォルト値が未定))。**プール内のすべての配置先をいかに読み取るか**：まず取得規則 (確定項目 7) に従いユニットを取得し、次いでルートリング内の任意の可読ルートの会計においてなお割り当てられているスロットをスキップ (現行 F (D16 (パブリッシュセマンティクス) 確定項目 1) 未満のルートはその会計をスキップ、ユーザー 2026-10-04 確定、ゼロ巡)；清掃が「ユニットである」と判定するのはヘッダチェックサムのみを参照し、いかなるディスク読み取り判定よりも厳しくしない。量化子は「当該インスタンスが書き込んだすべて」ではなく「プール内のすべて」を採用：消去済みまたは上書き済みの配置先は元の所属を語ることができず、後者で記述すると列挙すべき集合と破壊すべき情報が同一となってしまうため (放棄されたルートがリングに残る際、その行はロールバック候補判定基準の入力となる)；zoned は第1版で回収せず、チェーン長に上限はない (C121 (zoned 上でインスタンス表チェーン長に上限なし))。
  - **テーブルが可読不能な場合**：読み取り専用マウント；走査再構築は段階 1 で停止——テーブルが消失すれば行が1行も存在せず、全旧インスタンスの全ユニットが「行なし ⇒ パブリッシュ済み」に該当してしまい、クラッシュリカバリにより放棄された時間帯全体も含まれるため、曖昧性の記録とはならない。インスタンス表ユニット自身は I-1.2 (ブロックヘッダ書き込みシーケンスパブリッシュ済み) の述語を通らない：その現行バージョンはルートレコードが保持する物理ポインタにより一意に定まり、ポインタが指せないまたはチェックサム不合格であればテーブル可読不能となる。
  - **チェックポイントの無効化には必ずインスタンス代号を変更せねばならない**：現行インスタンスには行が存在せず、述語はそれに対して「誕生世代 ≤ マウントされたルートの txg」としか問えないため、同一番号を再発行してパブリッシュすると、誕生世代がそれに等しい孤立ユニットがすべてパブリッシュ済みと判定されてしまう——これが失敗表を閉じる根拠である (D23 (journal の役割とフォーマット) 確定項目 14 の注)。**インスタンス代号**はシステム構成に常駐する (E100 (システム構成の3セグメント幾何) セグメント4の 4 バイト)。
  - **書き込み可能マウントの順序**：まず今回書き込み可能マウントが可能かを判定 (書き込み可能デバイス数が w の下限を満たす ∧ **プール内の過半数デバイスを排他オープン**——任意の2つの過半数集合は交差するため、デバイスごとの排他オープンがプールレベルの排他制御となる；本要件とそのコスト (4台ディスクプールで書き込み可能が2台のみの場合は読み取り専用マウントしかできない) は D2 (RAID ストライプポリシー) 確定項目 13 が定めたものであり、確定項目 11 ではない ∧ インスタンス表が可読 ∧ インスタンス切り替え予約領域が取得可能 ∧ 今回のマウントで発行する行書き込みとウォームアップがアロケータの複製上でドライランを行い全配置先を取得可能——ドライラン内での行書き込み時の解放検証エラーも同様に今回書き込み不可と判定；ドライランは解放前ディスク読み取り検査を行わない、差異は C542 (番号取得前ドライランがディスク読込検査せず不良ディスクと旧ルート上書きで番号焼失) を参照)、次いで新代号 = max(**今回のマウントで排他オープンに成功した集合**内の各システム構成の代号 (各ディスクの2スロットのうち自己証明を通過した全スロット——チェックサム合格かつ fsid が本プールと一致), ルートリング内の全ルートレコードのインスタンス代号) + 1 を取得し、**当該集合**内の各ディスクの 2 スロットのシステム構成に 2 ラウンドに分けて書き込み（各ラウンドでディスクごとに 1 スロットずつ書いた後にプールバリアを 1 回通し、2 ラウンドとも完了して初めて新代号を渡す；`crates/singlefs-core/src/transaction.rs` の `write_the_witnessed_acquisition`）、**全か無か** (システム構成書き込みはデータユニット書き込みと同一方針：1回の I/O エラーは T_retry 枯渇まで再試行して初めて失敗と判定；枯渇後に全か無かで失敗と判定された場合は**まず書き出し済みのディスクを旧代号へ巻き戻し書き込みし、巻き戻しは各ディスク 1 回のみ書く** (旧代号 = 番号取得前に当該集合内の全自己証明スロットのうち最大のインスタンス代号；巻き戻し書き込みはいかなるユニットより前に行われる)、巻き戻し不能となって初めて読み取り専用マウントへ移行——巻き戻し期間中にディスク上に現れる「同一集合内での代号不一致」は I-7.7 (システム構成インスタンス代号はルートリング以上) の第 ② 文により合格と判定 (より大きな代号はいかなるルート、レコード、ユニットにも付随していないため)、別途例外を設けない；次回マウント時に max + 1 により修復；番号取得の各ラウンド直後のバリア (D23 (journal の役割とフォーマット) 確定項目 16) がいずれかのディスクでエラーとなった場合も全か無かの失敗と判定し、バリアのみ再発行して継続することは禁止；番号取得の書き込み集合と max の取得集合は同一集合を採用——「全可視」に書き込みつつ排他オープンが一部のみである場合、集合外のディスクに書き込めるか否かはこのマウントでは制御できず、2つのマウントが互いを読み取り専用に追い込み合うことになる。**過半数は受入の敷居値であり、書き込み範囲ではない**：書き込み集合は排他オープンに成功した全デバイスを採用し、過半数のみに書き込むと集合外の健全ディスクが取り残され、それらの代号は以後旧代号となってしまい、真に番号取得を取りこぼしたディスクと区別できなくなる。**「可視」= 排他オープン成功かつシステム構成可読**；I-7.7 (システム構成インスタンス代号はルートリング以上) の第 ② 文は同一集合に限定)、その後に初めてユニットを変更可能；読み取り専用マウントは番号取得を行わない；1から開始、0は無効。
  - **番号取得前に各ディスクが選択されたバージョンを保持しているかを検査** (メイン agent 2026-09-25 確定、攻撃ゼロ巡；`crates/singlefs-core/src/mount.rs` の `devices_without_the_selected_version` で実装、実二七)：書き込み可能マウントはプレフィックス適用後のバージョンを検査；渡されたディスクに1台でも保持していないものがあれば、番号取得前・いかなる書き込み前にも書き込み可能を拒絶 (`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`)；読み取り専用マウントはこの検査を行わず、各位置エントリを順次試行して選択バージョンを読み取る。「保持していない」には2種類：2つのシステム構成スロットのいずれも自己証明を通過していない (チェックサム合格、本プール fsid と一致)、すなわち当該ディスクが「可視」でない；世代番号が最大のシステム構成の (インスタンス代号, journal tail) が選択バージョンのパブリッシュ末尾レコードの jsn より小さく、**かつ**選択バージョンのユニットが当該ディスク上に存在しその配置先が読み取れないかバイト列が異なり、かつ当該ディスク自身の journal リングスロットにそのバージョンのパブリッシュの末尾レコードもない、または選択バージョン自身が選択されたルートでありそのルートスロットが当該ディスクに属しているのに1回読み込み・1回再読み込みのいずれも不一致 (読み取り不能、自己証明不合格、または自己証明合格だが (インスタンス, txg) 不一致；`SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndItsRootNotInItsRingSlot` として拒絶)；journal レコードを適用して得られたバージョンがルートスロット FUA 前にクラッシュし、どのディスクのルートリングにも存在しない場合、ルートスロット側の検査は行わない。許可される3種類：遅延のみでユニットは揃っておりルートスロット側も欠落なし (ルートがディスクに着地後・ローテーション前にクラッシュ；番号取得失敗時は番号取得の瞬間の目撃値へ巻き戻し書き込み (C554 (クラッシュリカバリ放棄ルート一時不読時シャドウ会計不計算) 乙-構成続))、これは正常なクラッシュ状態；システム構成は追従しており数個のユニットのみ欠落 (単一複製の破損、D19 (ブロックポインタ構造と幅予算) 確定項目 5 ハードルール 1 のディスク読込検査で隔離)；遅延しているが当該ディスクの journal リングスロットにそのバージョンの末尾レコードがあり、数個のユニットのみ欠落——ユニットは末尾レコードのバリアより前に書かれるため、末尾レコードは本ディスク上で「着地後に 1 部壊れた」と「一度も着地していない」を区別できる唯一のディスク上の証拠である (C593 (遅延かつ1複製欠落のディスクに対する書き込み可能マウントの恒久拒否) の形態；`crates/singlefs-core/src/mount.rs` の `units_of_the_version_missing_on_a_device_behind_it` はまず `last_record_of_the_version_in_its_ring_slot_on_the_device` を判定)。マウント後、ディスクテーブルを受け入れる各入口 (正常アンマウント、管理者ロールバック、セッション各パブリッシュ、F 引き上げ) は書き込み前に3点を検査：「可視」の分岐 (ディスクが本プール fsid の自己証明システム構成スロットを1つも持たなければ拒絶)；「可視」後、各ディスクの自己証明スロット内の自ディスクデバイス番号がディスクテーブルのアイデンティティと等しいこと；遅延の分岐——当該ディスクの最新自己証明システム構成の (インスタンス代号, journal tail) が現行バージョンの末尾レコードの jsn より小さく、**かつ** (現行バージョンのユニットが当該ディスク上に存在しその配置先が読み取れないか本プロセスで検証合格したコピーとバイト列が異なり、かつ当該ディスク自身の journal リングスロットにそのバージョンの末尾レコードもない、または現行バージョンのルートスロットが当該ディスクに属しているのに1回読み込み・1回再読み込みのいずれも不一致) である場合に初めて拒絶 (改訂法 C、書き込み可能マウントの遅延分岐と同一判定、「選択バージョン」を呼び出し側が保持する現行バージョンに置換；ルート欠落側の再読み込みは D16 (パブリッシュセマンティクス) 確定項目 1「ルートスロットが今回読み取り不良」に従い1回読み込み・1回再読み込み、ユニット側は読み取れなければ直ちに欠落と記録し再読み込みしない；遅延のみでユニットが揃いルートスロット内が合致する場合は拒絶せず、自己証明スロットが1つしか残っておらず前回ローテーションで停止したディスクの誤拒絶を防止)。後者2点はユーザー確定、起因と根拠はコード3者論証第2巡判決 `research/prompts/m2-closeout-code-r2-main-verification.md`「同一プール旧スナップショットディスクの入口経由ロンダリング」および「ディスク入れ替えのディスクテーブル」の2マス；遅延分岐は「遅延かつユニット欠落」を採用し「遅延で直ちに拒絶」は採らず、攻撃側プロトタイプ `fix_prototype_swapped_or_behind.patch` (`m2-impl-r2-fixes-b-implementer-report.md` 第4節第6条) に準拠。セッション「各パブリッシュ」は「各ユーザー変更」と解釈 (1回の呼び出しでループ前に1度のみ検査)；ループ内で領域不足のために発行される一連の F 引き上げごとに、再発行直前に各自3点を再検査するため、実際に発生する各書き込み前に検査済みとなる——これは推論であり、ループ途中でディスクを交換するテストケースは存在しない。3点とも実装済み：「可視」分岐は `m2-impl-z3a-yi-implementer-report.md` を参照；自ディスクデバイス番号と遅延分岐は `crates/singlefs-core/src/mount.rs` の `caller_inputs_agreeing_with_the_disk` (正常アンマウント、F 引き上げ、管理者ロールバック；自ディスク番号検査は `own_device_numbers_on_disk_differing_from_the_identities_handed_in`、遅延分岐は `devices_behind_the_current_version` であり、書き込み可能マウントと `what_a_device_behind_the_version_lacks` を共用：まず `units_of_the_version_missing_on_a_device_behind_it`、ユニットが揃っていれば `root_of_the_version_not_in_its_ring_slot_on_the_device`) および `crates/singlefs-core/src/mounted_session.rs` の `publish_user_change` (セッション)、テストケースは `crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`。拒絶時の報告列挙子：「可視」——正常アンマウント、F 引き上げは `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration` (`disagreeing_device_table` に `NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn` を含む)、管理者ロールバックは `RollbackError::CallerInputsDisagreeWithTheDisk`、セッションは `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration`；自ディスクデバイス番号——正常アンマウント、F 引き上げは同一の `MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration` (`disagreeing_device_table` に `OwnDeviceNumberDiffersFromTheIdentityHandedIn` を含む)、管理者ロールバックは `RollbackError::CallerInputsDisagreeWithTheDisk`、セッションは `UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable`；遅延分岐——正常アンマウント、F 引き上げは `MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`、管理者ロールバックは `RollbackError::CallerInputsDisagreeWithTheDisk` (`CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot` を含む)、セッションは `UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot`。
  - **前提条件**：同一時刻に書き込み可能マウントは1つのみ——C833 (書き込み可能マウントの排他オープン欠落) 着地後に過半数排他オープンにより保証される (現状は2プロセスが同時に書き込み可能マウント可能)；マルチホスト共有ストレージ上では2つの書き込み可能マウントが同一代号を算出してしまい、述語で区別不能となるが、D9 (暗号化) 確定項目 8 と同様に防護対象外と宣言。番号取得の瞬間に2系統の目撃証拠が不完全可視であっても追加の遮断は行わず、ボリューム全体のロールバックと同等の扱い；I-7.6 (ルートリング領域が互いに異なるディスクに配置) は devs ≥ R の場合にのみルートリングを独立した目撃証拠とするが、第1版 (R = 3, devs = 2) では成立しない。番号取得を取りこぼしたディスクは2種類に分かれる：**一度オープンされた後に脱落し、かつ欠番期間中にプール内で実際にパブリッシュが発生した場合**、復帰時にディスク全体が無効化され、再同期完了まで読み取り専用となる；**今回のマウントで一度も排他オープンされていない場合**、または欠番期間中にプール内で1度もパブリッシュがなかった場合、無効化されない——物理的に分岐の可能性が存在しないため；第1版は2台ディスク構成であり、1台脱落時は読み取り専用マウントしかできない ⇒ 分岐ディスクの復帰は到達不能；D2 (RAID ストライプポリシー) 確定項目 3 / 4 はオンラインディスク追加を許容 ⇒ 3台目のディスクが追加されれば到達可能となり、C120 (分岐ディスク復帰の判定と再同期) は3台目のディスク追加許可前に失効、過半数ルールと同時に適用。
  - D23 (journal の役割とフォーマット) レコードヘッダの「タイプ」とは別の名前空間であり、ポリシーも異なる。
- **2つのポリシーの権威ある記録点は D15 (フォーマット凍結ポリシー) 判定表である**：新規ユニットクラス追加 ⇒ incompat；**新規パックレコードタイプ追加もデフォルトで incompat である**——D15 (フォーマット凍結ポリシー) 判定表「新規レコードタイプ (新規 key タイプ) 追加 ⇒ compat_ro」の行は btree item タイプを指しており、パックレコードタイプはこれに従わない：墓標レコードのスキップは削除済み key を復活させ (E59 (バッファメッセージはユニットのみから再計算可能か))、inode レコードのスキップは読み取り専用マウントを空転させるため、現状登録されている2タイプはいずれも「スキップしても読み取り結果を変更しない」を満たさない；ある新タイプを登録する際にこの点が証明された場合のみ compat_ro へ降格され、かつ固定幅、暗号化後は鍵先行・MAC 先行のうえ全体スキップ、再利用禁止を要する。タイプコードは AAD の1フィールドの値であり構成ではない。compat_ro の読者は走査再構築を実行してはならない。新読者が未登録タイプに遭遇した場合は D15 (フォーマット凍結ポリシー) 判定表第2行に従い処置 (incompat ⇒ マウント不可)。
- **3つの下流パスに対する確定事項**：墓標——確定項目 8 の2文を3文に分解 (コンテナタグはコード 3；レコードはパックレコードタイプ 1；死者はレコード自身のフィールドにより特定)；inode パス ①——パックレコードタイプ 2 を予約、更新コスト見積もりは D8 (コアインデックス構造) 確定項目 6 に記載；アンチエイジング——オフセット 6 はクラスタグ、オフセット 7 は常時 0 + 非 0 拒絶と確定；D26 (バックグラウンド整理と配置回収) 確定項目 4 が将来ノードヘッダのビットをタグに使用する場合、オフセット 7 はもはや利用できず、別途フィールドを開設して D15 (フォーマット凍結ポリシー) に従い判定するか、incompat を占有せねばならず、確定項目 11 は D26 (バックグラウンド整理と配置回収) に代わって判定しない。

<!-- format-const: PACKED_UNIT_HEADER_BYTES = 107 -->
**数値** (E102 (ユニットクラス登録表とパックレコードユニット))：cap(107, 56) = 583 (区間 [65, 120])、cap(107, 140) = 233 (区間 [9, 148]；確定項目 16 においてヘッダ 107 は不変、レコード領域起点 136、容量再計算は C89 (ユニットヘッダ決着の残存再計算) ⑤ に未済)。

**射程**：値域登録表が管轄するのは「コード → クラス」および各クラスの判定着地点であり、AAD のフィールド構成は依然として D9 (暗号化) 確定項目 6 に帰属する (コード 1 の5項目はスナップショット次元に対して盲目であり、当該再開は未実施；コード 2 の AAD アイデンティティは着地点なし、C107 (インデックスノードの AAD アイデンティティが未着地)；コード 3 の AAD ツリー次元は誕生ツリーを採り、完全な契約は D9 (暗号化) 再開後に形成される)；コード 3 コンテナと中央マッピングの関係は D19 (ブロックポインタ構造と幅予算) 確定項目 7 / 8 に従う；コンテナインデックス更新コストは各レコードタイプに従う。登録表のコード 16–18 の凍結時点は D9 (暗号化) 暗号化稼働開始と同一；コード 1 / 2 / 3 は初回復製書き出し時に凍結。

**根拠**：

- E102 (ユニットクラス登録表とパックレコードユニット)：リポジトリ内の4箇所のリストと墓標は登録表の下ですべてマッピング可能、マッピング不能および1名称複数コードはいずれもゼロ、D9 (暗号化) 確定項目 6 の行のみを登録表とみなすと墓標等の名称が未登録コードに落ちる (C98 (ユニットタイプタグ値域の登録位置欠落))。
- E104 (走査再構築における現行バージョン判定)：インスタンス表とパブリッシュ済み述語——`recovery_own_txns_global_w` の世界 (大域的変数を範囲内の全インスタンスに書き込むと、リカバリインスタンス自身の孤立ユニットが復活) が「選択ルートのインスタンスのみ W が非ゼロ」を固定。
- E85 (ユニットヘッダのフィールド表)：5つ組 33 の合計値はその幅の仮定 (「アイデンティティフィールドは 8 バイトを採用」と自認) に由来し、内訳が個別に論証されたことはない；4つの 8 はそれぞれ独立した出典を持つが、いずれも 5つ組のために定められたものではない (各出典は変更履歴に記載)。
- E59 (バッファメッセージはユニットのみから再計算可能か)：墓標レコードのスキップは削除済み key を復活させる——新規パックレコードタイプ追加はデフォルトで incompat。
- 3者論証：3巡9アームの生出力は `research/prompts/d18-item11-r{1,2,3}-*-output.md` および `research/results/d18-item11-r{1,2,3}-local.out`、提案3版は `research/prompts/_d18-item11-{unit-class-registry,r2-amended,r3-amended}.md` (第1巡ローカルアームは文字化け検査で赤判定となり無効化、第1巡の計測条件はクラウド2アーム)；インスタンス表の各規則は C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定および以後のユーザー確定に由来：行書き込みとウォームアップ (C329 (行書き込みパブリッシュ前の F 引き上げ空パブリッシュ検査欠落) 3巡3者)、中間インスタンス行と切り替え選択ルート (C330 (中間インスタンス行の T_pub が選択ルート txg を採用) 3巡3者)、番号取得の読み出し・巻き戻し・バリアエラー報告 (C322 (番号取得ステップのバリア配置条項欠落) 3巡3者)、行幅 88 (C304 (インスタンス表チェーンポインタレコードが幅 83 ポインタを収容不能))；番号取得を取りこぼしたディスクの2分類は C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 第8巡逆推論アーム (`c113-r8 巡のアーム報告 (成果物は git 内)`) 3.1 / 7.2 に由来。経緯と原文は変更履歴に記録。
- ユーザー指示による決着 (値域登録表、5つ組内訳)、原文は変更履歴に記録。
- 「唯一の保持者」から位置ヒント保持者への改訂、1019 / 815 件、タイプ 1 幅 40 とタイプ 5 幅 16：墓標判決 `research/prompts/m3-tomb-r1-main-verification.md` 第3節 W1 の2文言、W2 フィールド表の2行、`research/prompts/m3-delete-r3-main-verification.md` 第3節 G4-系譜の配置先の行 (タイプ 5 は攻撃ゼロ巡)。
- ユーザー確定 2026-09-23：切り替えの選択ルートはディスク再読込でルート選択 (`research/prompts/c381-r3-main-verification.md` 第4節第3問)、原文は変更履歴に記録。
- ユーザー確定 2026-09-24：インスタンス表の行は1スライスを満杯にしてから次スライスを開く (原文は変更履歴に記録)；実験なし：スライス数計算式 max(1, ⌈行数 ÷ 369⌉) と直接合致し、計測可能なコスト差は存在しない。
- ユーザー確定 2026-09-24：番号取得前にアロケータ複製上のドライランで配置先を判定、行書き込み時の解放検証を番号取得前へ移動、ドライランがディスク読込検査を行わないマスは未済の検査として記録 (実十八報告 `m2-followups2-implementer-report.md` 第3節 1–3)、原文は変更履歴に記録；実験なし：2026-09-24 の実装に従い条項化、ドライランのコストは未測定。
- ユーザー確定 2026-09-26：管理者ロールバックはマウント中の前方パブリッシュへ変更 (D23 (journal の役割とフォーマット) 確定項目 14)、ロールバック行 (kind 0 レコード flags bit0) と「ロールバック時に書き込む行」は削除、旧イメージは incompat レイアウトアイデンティティビットの変更によりマウント外で遮断 (D15 (フォーマット凍結ポリシー) 確定項目 4)、原文は変更履歴に記録；実験なし：3者判決 `research/prompts/m2-rollback-forward-r3-main-verification.md` 第2節 K7 および弁護側再検査 (ロールバックインスタンスは自身に対してロールバック行を決して書き込まず、切り捨てに到達しないことは構造的事実)。
- ユーザー確定 2026-09-27：マウント後にディスクテーブルを受け入れる入口は「可視」の後に自ディスク番号と遅延分岐を再検査 (コード3者論証第2巡判決 `research/prompts/m2-closeout-code-r2-main-verification.md`「同一プール旧スナップショットディスクの入口経由ロンダリング」「ディスク入れ替えのディスクテーブル」の2マス)、原文は変更履歴に記録；実験なし：実装とテストケースは `m2-impl-r2-fixes-b-implementer-report.md` 第3節第1行——`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs` 10件：4つの入口で各1件旧スナップショットを固定、各1件ディスク入れ替えを固定、自己証明スロット1つのみで4つの入口がいずれも誤拒絶しないことを1件、攻撃側が全開走査する16マスを1件；遅延分岐で「遅延かつユニット欠落」を採用する取捨選択は同文書第4節第6条。
- ユーザー確定 2026-09-28：C585 (セッションパブリッシュでパブリッシュ途中に外された旧スナップショットの混入) は改訂法 C を採用；ユーザー確定 2026-10-01：書き込み可能マウントにおけるルート欠落の半分は残し、条項に文章を補完。原文はいずれも変更履歴に記録。実験なし：改訂法 C の判定法とテストケースはコード3者論証 m3-code-r1 から m3-code-r3 の3巡で判定済み (判決 `research/prompts/m3-code-r1-main-verification.md`、`research/prompts/m3-code-r2-main-verification.md`、`research/prompts/m3-code-r3-main-verification.md`)。
- インスタンス表の行削除と全巡清掃器：3者判決 `research/prompts/m3-ring-r1-main-verification.md` 第2節第10行および第4節第4条 (I-3.8 (インスタンス表行の一意性とマウントルート未満) 草案の条件は全巡清掃を要求、リポジトリに清掃器が存在せず；ユーザー 2026-10-01 に即時実装を決定)、`research/prompts/m3-ring-r2-main-verification.md` 第2節第10行 (行削除は専用の清掃パブリッシュに配置、ヘッダ消去はそれに先行、読み取り失敗時は削除せず；(j, 0, 0) 行が削除不能となる半分が成立)、`research/prompts/m3-ring-r3-main-verification.md` 第2節第9行 (読み取り成功した全ゼロルートスロットはルート不在とみなさない、各スロット自己証明合格または全ゼロでなければ1度再読込しなお不合格なら削除せず、2箇所の厳格化はゼロ巡)。実験なし：数値はすべて3者複製上；清掃器は判決当時は条項でありリポジトリ内コードはゼロであったが、現在は実装済み (`crates/singlefs-core/src/instance_table_sweep.rs`、書き込み可能マウント末尾でトリガー)。
- 清掃の実行主体と実行タイミング、プール内の全配置先の読み取り方法、清掃パブリッシュの予約領域と領域不足、清掃の順序、行書き込み条項の例外追加：判決 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第2節 S1 清掃5文の各行および第3節 F1、F3、F6 行、ユーザー確定 (2026-10-03、同判決第6節 Q1、Q3、Q4 行)、原文は変更履歴に記録；スイッチオン分岐と両半分の併用はいずれも攻撃ゼロ巡、順序とユニット認識計測条件は厳格化のみ；実験なし：数値は3者攻撃側複製上；判決当時清掃器コードはリポジトリ内ゼロ、現在は実装済み (`crates/singlefs-core/src/instance_table_sweep.rs`)。
- 清掃の計算が現行 F 未満のルートをスキップ、ヘッダ消去前の再読込失敗で今回の清掃は行削除せず：コード巡第3巡判決 `research/prompts/m3-root-ring-code-r3-main-verification.md` 第2節 K1-a、K1-b の2行 (K1-a：障害のない正当な履歴で正常アンマウントにより F 引き上げ後、F 未満の旧ルートの会計が読めず、清掃が毎回停止し行が1行も削除不能；K1-b：消去前の再読込失敗時、ヘッダを消去しないのに行を削除してしまい、行削除後に孤立ユニットが1つ残存；複製で測定済み、メイン agent の再実行で合致) および第6節 Q1、Q2 の2行；ユーザー確定 2026-10-04、原文は変更履歴に記録。実験なし：数値は攻撃側複製で測定、実験番号は未開設。
- チェーンポインタレコードが次スライスありとしつつ位置ポインタヘッダセグメントの一部が第1版の値でない場合も解釈不能とみなす：コード巡判決 `research/prompts/m3-batch10o-code-r1-main-verification.md` 第2節 K2-1 のマス (実装と checker で本形態に対する計測条件が異なり、メイン agent のコード読解で確認) および第5節 K2-1 の行、分岐リスト `m3-batch10o-code-r1-forks.md` K2-1 の行；ユーザー確定 2026-10-06、原文は変更履歴に記録。実験なし：計測条件の差異はコード読解により判定、実験番号は未開設。
- E178 (葉コンテナとレコード死亡の計数)：タイプ 3 退役レコードは死亡 txg を保持し、専用の退役ツリーに常駐、系譜によりフィルタ (モデル内では1行 34 バイト、1コンテナ 959 件)、借用と吸収、管理者ロールバックの履歴においてコンテナ復活と死亡誤判定はいずれもゼロ。
- タイプ 3 のキャリア、死亡 txg、および現行追従ツリーの P2 のみに従うバージョン選択：判決 `research/prompts/m3-leaf-death-r1-main-verification.md` 第2節 K1 (死亡 txg を保持しない形態は管理者ロールバック後に死亡を誤判定；キャリア未定義)、`research/prompts/m3-leaf-death-r2-main-verification.md` 第2節「候補1 (退役レコード) の完全形態」 (改訂法甲)、`research/prompts/m3-leaf-death-r3-main-verification.md` 第2節；ユーザー確定 2026-10-08「退役レコード・甲 (推奨)」、原文は変更履歴に記録。
- タイプ 3 退役レコードのフィールド表 (34 バイト)、1コンテナ 959 件、退役ツリー (種別コード 13) に常駐：判決 `research/prompts/m3-retire-r1-main-verification.md` 第2節 K2「レコード 34 バイト = アイデンティティ参照 26 + 死亡 txg 8」の行 (攻撃側サンプリング2回とも回避) および第3節第2条；ユーザー確定「プール開設時に構築 (推奨)」、原文は変更履歴に記録。
- 遅延分岐のユニット欠落側は、本ディスクの journal リングスロットにそのバージョンの末尾レコードがあるかを見る：三方判決 `research/prompts/m3-close-sysconf-r2-main-verification.md` 第4節、`research/prompts/m3-close-sysconf-r3-main-verification.md` 第2節 U3 (第2・第3ラウンドとも攻撃側はヒットせず；遅延分岐のテストケース一式 42 件は 2 通りの読み方のいずれでも通過、パブリッシュのユニット書き込み途中で取り外したスナップショットディスクは引き続き拒絶、レプリカで計測) および第4節「着地」。実験なし：数値は三方レプリカで計測、実験番号なし。
- 番号取得を 2 ラウンドに分けて書き、各ラウンド後にプールバリアを 1 回通し、巻き戻しは各ディスク 1 回のみ書く：システム構成三方第3ラウンド判決 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第2節 甲' (番号取得を 2 ラウンドで書く、巻き戻しは各ディスク 1 回のみ) のセル (2 ラウンド書き込みは第2・第3ラウンドとも攻撃側がヒットせず、各ディスク 1 回の巻き戻しは第3ラウンドで 1 回攻撃されヒットせず、レプリカで計測) および第4節「着地」；ユーザー確定 2026-10-19「カバーする、番号取得は 2 ラウンドで書く」、2026-10-20「番号取得の 2 ラウンド書き込みを維持」、原文は変更履歴にある。実験なし：数値は三方レプリカで計測、実験番号なし。

**未済の検査**：C98 (ユニットタイプタグ値域の登録位置欠落)：ゲート照合治具；C107 (インデックスノードの AAD アイデンティティが未着地)；C108 (拡張ポイント帰属がパックレコードユニットに対して判定なし)：赤判定となる検査；C111 (クラスタグの暗号文側コピー欠落)；C103 (コンテナインデックスの着地点なし)；C105 (パブリッシュ頻度に応じたパックコンテナ書き換えの書き込み増幅)；C106 (オープンコンテナ数の上限)；C109 (どのツリーの key 死亡で墓標を書き込むか)；C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない)：検査は依然未済；C597 (インスタンス表行が単調増加で削除条項なし)；C690 (清掃トリガー閾値のデフォルト値が未定)；C121 (zoned 上でインスタンス表チェーン長に上限なし)；C120 (分岐ディスク復帰の判定と再同期)；C329 (行書き込みパブリッシュ前の F 引き上げ空パブリッシュ検査欠落)、C330 (中間インスタンス行の T_pub が選択ルート txg を採用)：検査は依然未済；C89 (ユニットヘッダ決着の残存再計算) ⑤：E98 (inode レコードと inode ツリーの幾何) の 91 は依然再実行未済、E84 (パック墓標のピン留め量) ハーネスが固定した容量 583 は別途 C691 (パック墓標ハーネスが容量を 583 に固定) に記録 (E83 (墓標の粒度) はレコード領域起点 136、墓標レコード 40 バイトで再実行済み、2026-10-04 第2回)；C833 (書き込み可能マウントの排他オープン欠落)：確定項目 11 の前提条件「プール内過半数デバイスの排他オープン」が未実装。

#### 確定項目 12：コード 2 の書き込みシーケンスを 4 バイトに縮小、空いた 6 バイトをペイロード CRC と予約領域に配分

**確定内容**：コード 2 クラスアイデンティティセグメント内の書き込みシーケンスから 6 バイトのトランザクション番号を削除し、「書き込みシーケンス 4 (インスタンス代号のみ) + ペイロード CRC 4 + 予約 2」へ改定する；これら3セグメントの合計は依然として 10 であり、確定項目 12 自体はヘッダ幅を変更しない (コード 2 のヘッダは確定項目 18 を参照)。

| 書き込みシーケンスのどのセグメントを使用するか | コード 1 | コード 2 | コード 3 |
|---|---|---|---|
| パブリッシュ済み述語 (確定 P2) | インスタンス代号 + トランザクション番号 | **インスタンス代号のみ** | インスタンス代号のみ |
| 最新選択キー (確定 P3) | 書き込みシーケンス全段 | **規則に入らない** | (誕生世代, インスタンス代号, トランザクション番号) |
| I-1.8 (マージ後バージョン全順序) | カバー | **カバーしない** | カバー |

**射程**：同一の根拠はコード 3 に対しても文字通り成立するが、それを改定するには C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P3 および I-1.8 (マージ後バージョン全順序) のコード 3 全順序キーを変更せねばならない；キー改定 (コード 3 の最新選択キーを2つ組へ後退) は第2巡で却下された (固定点が反復し、2回連続してチェックポイントで新トランザクション番号が割り当てられない場合、2つのバージョンの書き込みシーケンスがバイト単位で同一となりタイブレークが失効する)；したがってコード 3 は 10 バイトを変更せず維持する。コード 2 の書き込みシーケンスが P3 に入らないのは、コード 2 がコンテナレベルの最新選択に一切関与しないためである (確定項目 11)；I-1.8 (マージ後バージョン全順序) に入らないのは、コード 2 の複数版が会計ツリーにおいてエントリレベルでマージされ (D5 (スナップショット / 領域会計メカニズム) 確定項目 2)、他ツリーの内部ノードは葉から再生成される派生状態であるためである (D8 (コアインデックス構造) 確定項目 6)。コード 2 の書き込みシーケンスを 4 バイトへ縮小することで空いた 6 バイトは、C104 (走査パス上でコード 1 / 2 のペイロード完全性に自己完結的な根拠がない) のコード 2 側のペイロード CRC 4 バイトをまさに充足し、2 バイトが予約領域として残る。

**予約は 2 バイトのみであり、C102 (インデックスノードヘッダに予約領域がなくフィールド追加は compat に非ず) は半分しか返済できない**：C102 (インデックスノードヘッダに予約領域がなくフィールド追加は compat に非ず) が自認する通り flag ビットは「フィールドを収容できない」ため予約領域とはみなされず、2 バイトも同様に大半のフィールドを収容できない——それが購入するのは「予約領域が存在する」という事実そのものであり、「将来何を追加しても compat になる」ことではない。**領域は根拠ではない**：6 バイト削ってもコード 2、コード 3 のいずれにおいても領域は購入できない (コード 3 の1コンテナあたりレコード数は変化ゼロ、コード 2 のファンアウトは最大でも1スロット増でツリー高は全く不変)；確定が採用したのは**何も購入できないバイト列を、登録された未済の検査が待機しているバイト列へ交換すること**であり、領域節約ではない。却下された2つの判定基準を再利用することは禁止：「現時点で消費者ゼロ ⇒ 挙証責任は残す側にある」 (これに沿えば確定項目 7 の nonce と MAC 予約領域、D19 (ブロックポインタ構造と幅予算) 確定項目 3 の MAC と nonce の恒常プレースホルダ、D8 (コアインデックス構造) 確定項目 8 の予約 32 バイトも一括して削られることになる；成立するのは別の基準：予約領域の挙証は「将来使用する」、トランザクション番号の挙証は「現在何も購入できない」であり両者は形状が異なる)；「会計ツリー内にトラバースで再計算不能な量が存在する ⇒ コード 2 はバージョン選択を要する」 (複数版の会計行は**エントリレベル**でマージされ、コンテナレベルでバージョン選択しない——D5 (スナップショット / 領域会計メカニズム) 確定項目 2 文言「非単調統計量は「後者勝ち」 (後者は seq により決定)、単調統計量は最大値を採用」)。E73 (ノード key 区間のファンアウトコスト) / E97 (会計と割り当てレコードのエントリエンコーディング) / E98 (inode レコードと inode ツリーの幾何) / E103 (inode パックパス 1 の更新コスト) が旧3段階ヘッダ幅で計算した数値は C89 (ユニットヘッダ決着の残存再計算) 第 ④ 筆および C288 (マッピング key 確定に伴う連動変更が他決定へ未反映) 第 ⑦ 条に帰属して再計算。

**根拠**：

- E104 (走査再構築における現行バージョン判定)：それが購入する唯一の用途は規則の前提下では発生し得ない——ソースコード自認 (`e104_current_version.rs` の `rebuild_root`)「書き込みシーケンスのトランザクション番号はそのチェックポイント内で割り当てられた最大トランザクション番号；コード 2 はチェックポイント固定点でのみ書き出されるため、同一チェックポイント内においてコード 2 はノード id ごとに最大1つのバージョンしか存在しない」 ⇒ トランザクション番号がコード 2 においてタイブレークとして機能することは決してない。
- 3者論証：生出力は `research/prompts/d18-item12-r1-{sonnet,opus}-output.md`、`d18-item12-r2-{opus-defense,sonnet-rebuttal}-output.md`；逆推論側アームはペイロード CRC 4 + 予約 2 が正確に 6 バイトを消費し、10 バイトの総幅を変更しないことを検証。

**未済の検査**：C163 (固定点各巡で書き込みシーケンスが同一となりタイブレークが失効)：赤判定となる検査；C102 (インデックスノードヘッダに予約領域がなくフィールド追加は compat に非ず)：残りの半分 (凍結前に拡張すべきか)。

#### 確定項目 13：暗号化有効時にコード 1 のアイデンティティセグメントと `宣言長` を平文と暗号文のどちらに配置するか

**確定内容**：アーム庚を採用——暗号化有効時、コード 1 の 5つ組 / 誕生世代 / 書き込みシーケンスは**その場で暗号文となり**、`宣言長` は3クラス共通の共通平文プレフィックスから除去され、一括して暗号文へ移行する。フィールドの位置と幅はいずれも変更されず、平文ヘッダ総幅は 105 のまま維持され、コード 1 の平文ホワイトリストは `{fsid, ペイロード CRC}` に縮小される。

**射程**：**漏洩をゼロにすることはできず、各アームの差異は有無ではなくオーダーである。** アーム庚は最も重い項目 (`宣言長` ⇒ 小ファイルの正確なサイズ) を解消し、付随してコード 3 の `宣言長` ÷ 公開レコード幅 (I-1.7 (パックコンテナ正当性と判定順序) ⇒ 小規模パックコンテナの正確なレコード数) も解消する。平文に残留する項目：

| 平文残留項目 | 漏洩内容 | 文字通りの根拠 |
|---|---|---|
| **ユニットタイプタグ** (flags オフセット 6) | データユニットとメタデータユニットの各数量 | D18 (ブロックに含める情報) 確定項目 7 文言「スキャナは復号前にクラスを判別してどのホワイトリストを適用すべきか知る必要がある」 ⇒ 構造的に暗号文へ入れない (A2) |
| **ペイロード CRC 4** | 一致 ⟺ 同一暗号文 ⇒ 複製数 `w` (第1版 `w` は常時 2、D2 (RAID ストライプポリシー) 確定項目 9 ⇒ 第1版で本マスに増分なし) | I-6.2 (平文ヘッダフィールドホワイトリスト) 注釈「ペイロード CRC 一致による複製の識別は認識のうえ受容」 |

ツリーテーブル長は平文内には存在しない：ツリーテーブルはルートレコード間接層のツリーテーブルユニット内に常駐し、ツリーテーブルユニットは暗号文である (D22 (ユニット原子性の合成方法) 確定項目 7 / 12)。庚のコスト：ユニットタイプタグに**鍵を持たない側の消費者をもう1つ追加させ**、「クラスタグも隠蔽する」ことを将来の incompat フォーマット変更に変質させる；受容。

**根拠**：

- E88 (なりすましユニットと孤立ユニットの識別)：そのソースコード内で `宣言長` / `decl` の出現は 0 回；取得後はユニット全体単位でスキップ ⇒ 取得規則の変更はゼロコスト、再実行は不要。
- コード 3 の判定順序の増分はゼロ：I-1.7 (パックコンテナ正当性と判定順序) の7ステップにおいて `クラスアイデンティティセグメント` は第2位、`境界` (宣言長を含む) は第5位であり、コード 3 のクラスアイデンティティセグメント全体はもともとメタデータクラスの暗号文である ⇒ 第5ステップではすでに復号鍵を要している；宣言長を暗号文へ移動しても新規の鍵要求は持ち込まれない。
- 鍵なし scrub 能力の喪失はゼロ：D9 (暗号化) 確定項目 5 が約束した「破損検知、不良ブロック報告」はヘッダチェックサム + ペイロード CRC + nonce 代号により履行され、いずれも `宣言長` を消費しない；パディング領域の破損も同様に捕捉される——ペイロード CRC はヘッダ末尾からユニット末尾 (パディングを含む) までをカバーするため。
- E28 (平文マッピング層はユニット自己記述のみから再構築可能か)：アーム丁 (暗号化有効時にアイデンティティセグメントを持たない) の欠落 = データブロック総数であり、漏洩面も縮小されない——却下。
- ユーザー確定「ブロックヘッダの漏洩は受容不能」 (本項目開設の根拠) およびアーム庚の採用、原文は変更履歴に記録；D18 (ブロックに含める情報) 確定項目 3 がデータユニットのアイデンティティを平文に残した際の弁護は「同一情報がすでに平文逆引きインデックスに横たわっている」ことであったが、D9 (暗号化) 確定項目 5 が後に平文逆引きインデックスを取消したことで弁護の前提が崩壊した。
- 2巡の3者論証：経緯は [records/2026-09-07-D18项3重开第一轮.md](../../../records/2026-09-07-D18项3重开第一轮.md) および [records/2026-09-07-D18项13第二轮.md](../../../records/2026-09-07-D18项13第二轮.md) に記録。
- 13-c ペイロード CRC はディスク上のバイト列をカバー：順推論側と逆推論側がそれぞれ独立して確定前はリポジトリ全体が空白であることを確認、2つの独立した理由は確定項目 7 に記述；ユーザー確定、変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 14：nonce 代号は完全な 96 ビット nonce であり、ユニット平文ヘッダに常駐

**確定内容**：**アーム甲を採用**：nonce 代号は完全な 96 ビット nonce (12 バイト) であり、ユニット平文ヘッダに常駐、ポインタ側のコピーも保持する。コード 2 / コード 3 の予約領域 (29 バイト、確定項目 16) は**クラスアイデンティティセグメント全体の直後に配置**；コード 1 の予約領域はオフセット 105 に常駐する (確定項目 18)。

**射程**：再構築パスにはポインタが存在せず、確定項目 13 以後アイデンティティは暗号文内に存在する ⇒ ユニット自身の上に完全な nonce を配置しなければ復号不能となる；本項目はその欠損を補完する。本マスには依然として2箇所の空白が存在する：I-6.1 (nonce 再利用禁止) の注記「nonce 代号の割り当て規則未定」および I-6.5 (再構築時の nonce 再利用禁止)「再構築後の nonce 水位」；いずれも本項目では定めない；確定項目 14 は幅 (12 バイト) と常駐位置 (ユニット平文ヘッダ) のみを確定する。C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P1 の書き込みシーケンスは 10 バイトを占有；C288 (マッピング key 確定に伴う連動変更が他決定へ未反映) 第 ① 条の誕生シーケンス番号 4 バイトと合わせ、ヘッダ幅は E117 (予約領域算入後のヘッダ幅と幾何) より 4 バイト増加した (C298 (ヘッダ幅プラス 4 がユーザー確定の数値を改変))。

**根拠**：

- E117 (予約領域算入後のヘッダ幅と幾何)：予約領域をヘッダに算入した後、ファンアウトの低下は各 0–1 スロット、コード 3 の1コンテナあたりレコード数は1スロットも低下せず；下流の4実験を同日再実行し、結論は1つも覆滅されず。
- 1巡の3者論証：経緯は [records/2026-09-07-D18项14第一轮.md](../../../records/2026-09-07-D18项14第一轮.md) に記録；他の3アームは各自却下——白化 (アーム丙) は何も購入できず (同一イメージ上にすでに5箇所の構造的な順序 / グループ化漏洩が存在し、白化はいずれにも触れられない) かつ I-6.5 (nonce 水位単調) を破壊；ポインタ側コピーの削除 (アーム乙) は少なくとも7箇所の登録数値を動揺させ、I-6.7 (複製間で nonce 共有) を構造的自動成立からビットごとの照合を要する新検査に変質させ、空間しか購入できない；再構築時に復号しない (アーム丁) は C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) 確定 P3 の最新選択キーがすべて暗号文内にあることと衝突。各アームの判決文は変更履歴に記録。
- ユーザー確定 (「甲を採用、ただしヘッダ幅を正確に計算してから確定すること」)、原文は変更履歴に記録；ヘッダ幅が 4 バイト増加した後も新数値に基づき原確定が成立 (C298 (ヘッダ幅プラス 4 がユーザー確定の数値を改変))。

**未済の検査**：C184 (再構築パスで完全な nonce を取得不能)：ゲート照合。

#### 確定項目 15：撤回済み・本番号は欠番

**確定内容**：撤回、本番号は欠番とする。旧題「コード 2 のバージョン選択規則」：複数版のコード 2 が存在する場合に、コード 2 が必要とするか否か、およびどのような入力 (既存の誕生世代、インスタンス代号、key 区間、または新規追加フィールド) を用いて「ルート R (txg T) が参照しているのはディスク上のどの版のコード 2 ノードであるか」を判定すべきかという問い。

**射程**：3巡の結論：文面上の2つの候補 (key 区間をフェンスセマンティクスとする、ノード番号 + 退役レコード) は、「親ノードと子エントリの現行版が同時に喪失し旧版がディスク上に残存している」マスにおいていずれも旧版を静かに選択してしまう；誤った回答を出さない唯一の形態 (フェンス + 受入ゲート) はスナップショットから参照された際に判定不能となる；そして D11 (インデックスノードにメッセージバッファを残すか否か) および D21 (権威状態と派生状態の境界) はコード 2 が再構築を要しない (葉から再生成、またはエントリレベルでマージ) とすでに定めている。したがってコード 2 にはコンテナレベルのバージョン選択規則は不要であり、撤回。

**根拠**：

- 実験なし：撤回されたのは不採用となった規則であり、3巡の論証判決は条項とモデルに対する判断であって、採用された数値はない。
- 3巡3者：検証と判決は `research/prompts/d18-item15-r1-main-verification.md`、`research/prompts/d18-item15-r2-main-verification.md`、`research/prompts/d18-item15-r3-main-verification.md` に記録。

**未済の検査**：C292 (コード 2 アイデンティティセグメントの消費者ゼロの理由が破綻)：照合ゲート。

#### 確定項目 16：nonce / MAC / アルゴリズムタイプ予約領域はヘッダに算入、宣言長の原点は予約領域を含むヘッダ末尾

**確定内容**：3クラスのユニットの **29 バイト**の予約領域 (nonce 12 + MAC 16 + アルゴリズムタイプ 1) は、いずれもクラスアイデンティティセグメントの直後に配置してヘッダに算入する：コード 1 ヘッダ **134**、コード 3 ヘッダ **136**、コード 2 ヘッダ **115 + 2 × key 幅**；宣言長の原点は**予約領域を含むヘッダ末尾**とする。

**射程**：予約領域に何を格納し、nonce がなぜ完全な 96 ビットであるかは確定項目 14 に帰属；コード 2 の完全なオフセットは確定項目 18 に帰属；当該 29 バイトはヘッダチェックサムカバー範囲外である (I-2.4 (ヘッダチェックサムカバー範囲))。

**根拠**：

- E117 (予約領域算入後のヘッダ幅と幾何)：予約領域を含むヘッダ幅とファンアウトコストは測定済み。
- E102 (ユニットクラス登録表とパックレコードユニット)：第2回再現で cap(136, 40) = ⌊32632 / 40⌋ = 815。
- 確定項目 14 が nonce はユニット平文ヘッダに常駐すると定め、C288 (マッピング key 確定に伴う連動変更が他決定へ未反映) 第 ① 条が誕生シーケンス番号の直後と定めており、欠落していたのは「ヘッダに算入するか否か、宣言長はどこから始まるか」の一文のみであった。
- ユーザーは [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第9節の選択肢に従い項目ごとに確定 (ポップアップ問答、メイン agent 記録)；アルゴリズムタイプの 1 バイトは別回のポップアップで確定；いずれも3者論証は経ておらず、メイン agent の推奨とユーザーの選択はいずれも覆滅可能。原文は変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 17：32 バイトヘッダチェックサムフィールドは CRC32C 4 バイト + 28 バイトのゼロを格納；自己証明構造のチェックサムはパディングを含むスロット全体をカバー

**確定内容**：32 バイトのヘッダチェックサムフィールド内に CRC32C 4 バイト + 28 バイトのゼロを格納；ルートレコード自己証明チェックサム、システム構成全スロットチェックサム、journal `header_csum` も同計測条件とする。自己証明構造 (ルートレコード、システム構成スロット、journal レコードヘッダ) のチェックサムはパディングを含むスロット全体をカバーし、チェックサムフィールド自身はゼロとして関与；パディングは常時 0。**ハードコードされたパラメータ**：CRC32C = CRC-32C、Castagnoli 多項式 0x1EDC6F41、リフレクション実装 (入力・出力ともにビット反転)、初期値 0xFFFFFFFF、出力反転；結果はリトルエンディアンで 32 バイトフィールドの**先頭 4 バイト**に書き込み、残り 28 バイトは常時 0、読者が非 0 に遭遇した場合は一律当該構造破損と判定。リポジトリ内のすべての CRC32C (ヘッダチェックサム、ペイロード CRC、各位置エントリ内の 4 バイト、自己証明構造のチェックサム) は同一パラメータセットを用い、他箇所からは一律ここにリンクする。**チェックサムアルゴリズム識別子をシステム構成へ参入**：ブートストラップヘッダセグメントに 1 バイトの「チェックサムアルゴリズム識別子」を追加、登録表 0 = 無効、1 = CRC-32C、第1版は 1 を書き込み (D22 (ユニット原子性の合成方法) 確定項目 9)；マウント時は本バイトに従いアルゴリズムを選択し、ユニットヘッダ内の 4 バイトチェックサムと同一の計測条件として登録。新規プールでのファイル作成のバイト列変更：**あり**——各チェックサムフィールドの値およびフィールド内における 4 バイトの位置。

**射程**：当該 28 バイトは 0 として自身のチェックサムに関与し (I-2.4 (ヘッダチェックサムカバー範囲))、それを防護するのは「常時 0、非 0 は破損判定」という規則であり、別のチェックサムではない。アルゴリズムは iSCSI / SCTP / btrfs / ext4 が共通で使用しているものを採用 (`crc32c` ハードウェア命令が直接算出するものと同一)、広く通用する代替選択肢は存在しない。

**根拠**：

- E144 (ヘッダチェックサムアルゴリズムのコストと識別力)：6つの候補アームはヘッダに対する単一ビット反転、二重ビット反転、ランダムバイト改ざんのいずれにおいても漏れゼロ；SHA-256 は CRC32C より遥かに高コストであるが、絶対量はいずれも1回の fsync の端数にすぎない；CRC32C はリポジトリ内で確定済みの関数。ユーザー指針「暗号化と無関係なら安価なものを」、原文は変更履歴に記録。
- D13 (検証ルート) 確定項目 5 は checker と実装側でそれぞれ独立した実装を作成することを要求し、D13 (検証ルート) 自身に「CRC32C を各自作成する前提はパラメータの固定 (多項式、初期値、入出力反転、排他的論理和出力)：パラメータを固定しなければ、2つの独立実装が健全イメージ上で異なる結果を出力し、独立性ではなく偽陽性の赤判定となる」と明記——4パラメータを空白のままにしては独立性を獲得できない。
- ユーザーは [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第9節の選択肢に従い項目ごとに確定 (ポップアップ問答)、パラメータは総点検第2巡で補完、アルゴリズム識別子のシステム構成参入は別回のポップアップで確定；いずれも3者論証は経ておらず、覆滅可能。原文は変更履歴に記録。

**未済の検査**：C309 (32 バイトチェックサムのアルゴリズム未定)：checker 側の独立実装および「実装と checker が同一関数を使用していること」の照合。

#### 確定項目 18：コード 2 の宣言長 = エントリ数 × エントリ幅、パディング領域は常時 0、コード 1 / コード 2 にオフセット付きフィールド表を追加

**確定内容**：コード 2 宣言長 = エントリ数 × エントリ幅；エントリ領域直後のパディング領域は常時 0；I-2.3 (パディングバイトはゼロかつチェックサムに関与) の射程をコード 2 へ拡張。コード 1 とコード 2 にコード 3 と同型のオフセット付きフィールド表を追加——

| クラス | オフセット付きフィールド表 |
|---|---|
| コード 1 | 42 タグコピー / 43 ツリー ID / 51 オブジェクト ID / 59 オブジェクト誕生世代 / 67 アンカーオフセット / 75 誕生世代 / 83 fsid / 91 書き込みシーケンス / 101 ペイロード CRC / 105 予約領域 29 ⇒ 平文ヘッダ 105、予約領域込み 134 |
| コード 2 | 42 ツリー ID / 50 階層 / **51 key 幅 1** / 52 key 区間 2k <!-- format-const: INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT = 2 --> / 52 + 2k 誕生世代 / 60 + 2k fsid / 68 + 2k 書き込みシーケンス 4 / 72 + 2k 誕生シーケンス番号 / 76 + 2k ペイロード CRC / 80 + 2k 予約 2 / 82 + 2k エントリ数 2 / 84 + 2k エントリ幅 2 / 86 + 2k 予約領域 29 ⇒ 平文ヘッダ 86 + 2k <!-- format-const: INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86 -->、予約領域込み 115 + 2k |
| コード 3 | 平文ヘッダ 107、予約領域込み 136 (予約領域 29 は他2クラスと同計測条件；レコード領域は 136 から開始、宣言長上限 32768 − 136) |

- **key 幅フィールドは固定オフセット 51 に配置**、key 区間の直前に位置：走査再構築パスには親ポインタもツリーテーブルも存在せず、コード 2 ノードをデコードするには key 区間が何バイトを占めるか知るために事前に `k` を知る必要がある；読者は `[0, 51)` の固定長プレフィックスから `k` を読み取り、次いで `86 + 2k` によりヘッダ全体を確定し、一括してデコードする。
- **コード 2 のペイロード CRC は `[86 + 2 × key 幅, 16384)` をカバー**、起点は**平文ヘッダ末尾**であり、29 バイトの予約領域、エントリ領域、およびその後のパディング領域を含む；計測条件はコード 1 の 105、コード 3 の 107 と同一 (3クラスとも平文ヘッダ末尾から起算し、予約領域を含むヘッダ末尾からは起算しない)。エントリ領域の起点は別の数値——予約領域を含むヘッダ末尾 `115 + 2 × key 幅` (D8 (コアインデックス構造) 確定項目 11) であり、両者の差こそが当該 29 バイトである。
- 確定項目 11 登録表のコード 2 行に誕生シーケンス番号 / ペイロード CRC / 予約 / 自己記述3セグメントを補完。コード 2 の完全なオフセットは本箇所にのみ登録し、確定項目 7 のインデックスノード段落にはフィールド集合のみを記載し、数値を重複保持しない。
- 新規プールでのファイル作成のバイト列変更：**あり**——6つのコード 2 ユニットのエントリ領域起点、宣言長、ペイロード CRC、key 幅フィールドの配置先とヘッダ幅 (`115 + 2k`：inode ツリールート 131、extent 163、割り当てレコード 135、会計 159、マッピング 169、ツリーテーブル 131)。

**射程**：パディング領域常時 0 および I-2.3 (パディングバイトはゼロかつチェックサムに関与) の射程はコード 1 / コード 3 の既存計測条件から推論されたものであり、別の選択肢は存在しない。コード 2 ヘッダ幅計算式 `86 + 2k` が全ツリーで成立することは確定項目 2 (全コード 2 ノードが無条件で key 区間を保持) に依存している。

**根拠**：

- E145 (コード 2 自己記述ヘッダとマッピングツリー key 幅のコスト)：自己記述3セグメントはノードあたり 5 バイト増加するのみでファンアウトとツリー高は1スロットも低下せず；非自己記述ヘッダは幅を順次試行することで登録済みツリーを解釈可能であるが、未登録ツリーは解釈不能。ユーザー指針「コード 2 は自己記述可能」、原文は変更履歴に記録。
- key 幅をオフセット 51 へ移動：C325 (コード 2 の key 幅フィールド位置特定に事前の k 把握が必要) の未済検査 (key 幅を `81 + 2k` に置くと自身のオフセットが `k` に依存 ⇒ 読者は `k` を総当たり試行せねばならなくなる)。
- ユーザーは [records/2026-09-13-总审核.md](../../../records/2026-09-13-总审核.md) 第9節の選択肢に従い項目ごとに確定 (ポップアップ問答、メイン agent 記録)、オフセット表の2箇所 (key 幅移動、予約領域 29 への拡張) は別回のポップアップで確定、コード 2 ペイロード CRC の起点は総点検第2巡で補完；いずれも3者論証は経ておらず、メイン agent の推奨とユーザーの選択はいずれも覆滅可能。原文は変更履歴に記録。

**未済の検査**：なし。

#### 確定項目 19：フィールド値をブロックに含めるべきか：3つの判定基準

**確定内容**：**判定基準は確定済みであり、3つの条件の積集合とし、いずれも不可欠である。** あるフィールド値をブロックに含める価値があるか否か：

| # | 問い | 満たさない場合の結果 |
|---|---|---|
| **1 (必須)** | **その更新周期はコンテンツの変更周期と連動しているか？** 連動 = コンテンツが変化して初めてそれも変化する | 連動しないフィールドは変更のたびに H 段の全チェーン COW を独立して誘発する (COW 下ではいずれにせよアドレス変更とルートへの書き換えが必要であり、これはただ乗りできる；更新がコンテンツと連動しなければ、純ペイロード変更ゼロに対して全コストを支払うことになる) |
| **2 (必須)** | **それが欠落した場合、具体的にどのリカバリ/判定が「判定可能」から「判定不能」へ転落するか？** | 答えられない場合は恒久的なフォーマット負債となり、見本は D18 (ブロックに含める情報) 確定項目 19「反面見本」を参照 |
| **3 (必須)** | **既存のメカニズムに対してどれだけの純増分が存在するか？どちら側に対して回復可能か？** | 既存メカニズムと重複するフィールドの限界便益はゼロに漸近する；「鍵があって回復可能」と「鍵がなくても回復可能」は別物である |

- **コストは各レコードに載り、ブロックヘッダには載らない**：ブロックヘッダ層のオーバーヘッドは「ヘッダ / ノード」の比率で見れば実質ゼロであるが、レコードフォーマットに 1 バイト増えるだけで各レコードが丸ごと1スロット余計に支払わされる可能性がある (bcachefs の bkey ヘッダが 1 バイト増える ⇒ extent レコードが 40 から 48 バイトへ跳ね上がり、ノード容量が 16.7% 低下)。
- **反面見本：障害モードの改善を何も購入しなかったフィールド**——btrfs の `chunk_tree_uuid`、16 バイト/ブロック：1箇所で書き込み、1箇所で scrub 検査、1箇所でマウント時に**自己参照的**に期待値を読み出すが、**通常の読み取りパスでは二度と読まれず、マウント時にも検査されない**。これが欠落しても、検知可能から検知不能へ転落する障害モードは1つも存在しない。

**射程**：⚠️ **判定基準 2 は `.claude/rules/fs-design.md` の「誤用をマウント不能に変えられるか」ではない**——あちらは**フォーマット分岐**のために設計されたマウント時の二値判断である；情報保持は実行時の証拠であり、データリカバリと破損箇所の局所化に資する。3条件を通過した2つの候補：

| 候補 | どの障害と連動しているか | 状態 |
|---|---|---|
| **コミットポイントアイデンティティ = (カウンタ, コミットポイント hash)**、単なるカウンタではない | **単調カウンタはロールバック / fsck 後に重複する**；重複後、前タイムラインの残留ブロックが正当なブロックに偽装する (f2fs が checkpoint パックの hash を NAT / SIT ヘッダに埋め込むのはこのため) | インスタンス表によりカバー (確定項目 11) |
| **「私は移転された複製である」フラグ + 複製専用データ CRC** | UBI 第一者自認：2つの PEB が同一の LEB に対応する場合、**カウンタのみでは不十分**であり、新しい側は移転途中に給電停止した可能性がある——「this is just an uncommitted copy」 | 第1版では不要 (D2 (RAID ストライプポリシー) 確定項目 1 全幅ストライプ書き込み) |

**根拠**：

- 実験なし：3つの判定基準はフィールド受入の規則であり、各根拠は実稼働実装の事実である——ブロックヘッダ層のオーバーヘッド (btrfs `btrfs_header`、bcachefs `btree_node`) は比率からして実質ゼロ、bcachefs bkey はレコードレベルの膨張を例証、btrfs `chunk_tree_uuid` は `fs/btrfs/disk-io.c` および `fs/btrfs/volumes.c` により確認。

**未済の検査**：C18 (連動しないフィールドによる読み取り専用次元の汚染)：判定基準 1 の赤判定となる形態。

#### 確定項目 20：保持が確定したアイデンティティフィールド

**確定内容**：判定基準3条をすべて通過し、保持が確定した3項目：

| フィールド | 欠落時にどの判定が判定不能となるか | 配置先 |
|---|---|---|
| **自身の論理アドレス + fsid** | 誤方向書き込み (ghost write) が**検知不能**；ブロック番号のみでは不十分であり、誤方向書き込みは**別ファイルシステムの正しいブロック番号**に着地する可能性があるため、位置情報は**ファイルシステム識別子を含まねばならない** | データユニット 5つ組 (確定項目 3)；I-1.1 (ブロックヘッダ自己記述論理アドレス) / I-1.4 (ブロックヘッダ fsid 一致) |
| **generation (誕生世代)** | **消失書き込みが残した旧ブロックは自己整合的である**：ブロックの世代と親ノードポインタに記録された期待世代を照合して初めて、「そもそも書き込まれなかった」ブロックと「誤った場所に書き込まれた」ブロックを識別可能 | I-1.2 (ブロックヘッダ書き込みシーケンスパブリッシュ済み) |
| **所属ツリー ID (誕生ツリー)** | 孤立ブロックの**破損範囲すら判定不能** | I-1.3 (ブロックヘッダツリー ID 一致) |

**射程**：データユニットヘッダ内の「自身の論理アドレス」は 5つ組により表現され (アンカーオフセットがその論理アドレス、I-1.1 (ブロックヘッダ自己記述論理アドレス))、物理配置先は含めない (確定項目 3 の2つの防衛線)；各ユニットヘッダ内の配置先は確定項目 7 / 11 / 18 を参照。

**根拠**：

- E88 (なりすましユニットと孤立ユニットの識別)：fsid と誕生世代を持たないヘッダは、なりすましユニットが段階 2 へ混入し、断裂孤立ユニットが1つも清掃不能。
- 実稼働実装の第一者言明：btrfs「the measures have to be done ahead of time」および `fs/btrfs/disk-io.c`「This is how we detect silent data corruption and misdirected writes」。

**未済の検査**：なし。

#### 確定項目 21：自己記述の射程は誤方向書き込みであり、消失書き込みではない

**確定内容**：**消失書き込み (lost write / phantom write)**——デバイスが書き込み成功を応答しながら実際には書き込んでいない——が残した旧ブロックは、**ブロック内レベルにおいて完全に自己整合的である**：旧内容、旧チェックサム、正しい自身の論理アドレス、正しい fsid、正しい所属ツリー ID。**ブロック内に保持される自己記述情報は、いかなる設定下でも消失書き込みを捕捉できない**；それを捕捉するには根本的に**ブロック外の目撃証拠**が必要である：親ノードポインタ内の期待世代とチェックサムであり、ルートまで再帰する (D4 (チェックサム位置) Merkle ツリー)。確定項目 20 の「generation」が消失書き込みを防護できるのは、親ポインタが期待値を記録しているためであり、**ブロック自身のヘッダ単体では検知不能である**。

**射程**：自己記述と親ポインタの境界を明確化：自己記述は誤方向書き込みを捕捉し、親ポインタは消失書き込みを捕捉する；両者は互いを代替しない。D16 (パブリッシュセマンティクス) チェックポイントコミット順序は親ポインタが子ブロックの後に着地することを保証し、ループを閉じる。

**根拠**：

- 実験なし：定義的——正当な旧バージョンと正当な新バージョンは**ブロック内レベルにおいて同型である**；3つの反例パスはいずれも失敗：実時間タイムスタンプの保持 (旧ブロックは**正当に**旧タイムスタンプを保持)、局所カウンタのインクリメント、およびバージョン履歴ハッシュチェーン (「期待カウント」「期待チェーン長」「期待ハッシュ」はいずれもブロック外に常駐)。
- OSTEP §45.6 文言 (ローカル PDF 抽出再確認)：「Unfortunately, the answer is no: the old block likely has a matching checksum... Thus, checksums alone are not enough to detect lost writes; the file system needs additional machinery...」。
- XFS の `xfs-online-fsck-design.rst:945-946` はブロックに記録された ondisk address が「enables the filesystem to detect lost writes」と主張しているが——**事実誤認であり、誤方向書き込みと混同している**；`fs/xfs/libxfs/xfs_types.h` により確認。

**未済の検査**：C7 (期限切れブロックの検査欠落)。

#### 確定項目 22：明示的フィールド vs チェックサムシードへの折り込み

**確定内容**：**「ディスク走査再構築」を支援すべき情報はすべて明示的に書き込まねばならない；「誤用の検知のみを要し再構築を要しない」情報のみシードへの折り込みを許容する。** シードへの折り込み (ext4 は `(fs uuid, i_ino, i_generation)` をチェックサムシードへ折り込む) は**単方向の検査**を形成する：期待されるアイデンティティを保持する読者はブロックの正当性を検証できるが、インデックスなしで孤立ブロックを拾い上げたスキャナは**チェックサムからアイデンティティを逆算できない**。

**射程**：アイデンティティ情報を明示的フィールドとして書き出すかシードへ折り込むかを定める；ある明示的フィールドを含める価値があるか否かは、依然として確定項目 19 の3つの判定基準を通過する。

**根拠**：

- 実験なし：シードへの折り込みからアイデンティティを逆算可能か否かは定義的であり (チェックサムは与えられた推測を検証することしかできない)、計測可能な量はない；ext4 (`fs/ext4/ialloc.c` 内の `__le32 inum = cpu_to_le32(ino); csum = ext4_chksum(sbi, csum, (__u8 *)&inum, sizeof(inum));`) はそれが純粋な検証用であり、オフラインでの inode 特定を支援不能であることを裏付ける。

**未済の検査**：なし。

#### 確定項目 23：否決リスト

**確定内容**：以下の候補はブロック内に含めない：

| 候補 | 否決理由 |
|---|---|
| **子ブロックによる「現行」親ノード物理アドレス記録** | **時系列上不可能**：COW は子が先、親が後に書き込む——子ブロックが書き出す瞬間、それが記録したい対象はまだ存在しない。後からのバックフィルへ変更することは「1度書き切ったら書き戻さない」に反する。これは論理的矛盾であり工学的困難ではない |
| **チェックポイントインライン「本回計 N ブロック」** | **同類の時系列デッドロック**：N はチェックポイント終了時に初めて判明するが、フィールドはその間に書き込まれるすべてのブロックへ埋め込まねばならない |
| **COW 先行アドレス (バージョンチェーン)** | **正味価値がマイナス**：① D5 (スナップショット / 領域会計メカニズム) の birth txg + deadlist と機能重複；② 先行ブロックが物理的に再利用された後、そのポインタは**誤導ポインタ**となる——それをたどると**別オブジェクトの現在の内容**が見える；③ 自らが作り出した落とし穴を防ぐために新たな不変条件の追加を要求する |
| **ブロック内誤り訂正符号 (ECC)** | **否決、理由は「両端とも引き合わず、中間にスイートスポットが存在しない」** (「高価すぎる」からではない)。① 安価な側：Hamming 16 ビットで 1 ビット訂正、4 KiB ブロックでのオーバーヘッドは **0.05%** であるが、**デバイス ECC がすでに修復した種類のみを修復する**；高価な側：RS で 512 バイトセクタ全体を修復するには 12.5%、断裂書き込みで失われた半ブロックを修復するには 50% を要する；そしてデバイス ECC がカバーできない2種類のうち、断裂書き込みには 50% のパリティが必要であり、**誤方向書き込みではブロック全体が他人のデータであって利用可能な冗長性は一切存在しない**。② **全調査対象で唯一の実例は OCFS2 `metaecc`** (`fs/ocfs2/ocfs2_fs.h` の `struct ocfs2_block_check`：32 ビット CRC32 + 16 ビット Hamming、`blockcheck.c` の `ocfs2_hamming_fix_block` で 1 ビット訂正後に CRC を再検証)、**単一ビット訂正のみでデフォルト無効**、その設計文書の動機は「Modifying it to this level of safety isn't really possible **without creating an entirely new filesystem**」と述べており——**この動機はゼロから設計された本プロジェクトには成立しない**。③ 他に 13 システム (IRON、CrookFS、NOVA-Fortis、EnvyFS を含む) を調査したが、**ブロック内 ECC はゼロヒット**；NOVA-Fortis は明示的に ECC を評価した上で拒否 |
| **ブロックごとの実時間記録** | 4つの実装のブロックヘッダをフィールドごとに調査したが**1つも存在せず**、すべて `generation` / `txg` を使用——単調、比較可能、COW コミットに自然な定義点が存在。実時間は ZFS の uberblock に1度現れるのみであり、**プール全体で1コピー**である |

**射程**：否決されたのはこれら5種類のフィールド形態である。同一バッチチェーン (前後 = 同一書き込みで書き出されたユニット群の割り当て順における隣接) は否決リストに抵触しない (割り当ては書き込みに先行し時系列逆転なし)；それを含める価値があるか否かは確定項目 4 で別途判定 (脱落)。

**根拠**：

- 実験なし：前2条は時系列矛盾 (子は親に先行して書き込み；N は終了時に初めて判明)、COW 先行アドレスは D5 (スナップショット / 領域会計メカニズム) のメカニズムと重複し再利用後に誤導ポインタ化、ブロック内 ECC と実時間は実稼働調査に由来；出典は否決理由および変更履歴に記載、本プロジェクトでは未検証。

**未済の検査**：なし。

#### 確定項目 24：第1版墓標：墓標ツリー、40 バイトレコード、系譜レコード

**確定内容**：

1. **墓標ツリー**：専用ツリー1本、day-1 にゼロルートとしてツリーテーブルに登録；ルートは常時コード 2、エントリは inode ツリー内部ノードの 120 バイト形態 (セパレータ key コンテナ番号 8 + アイデンティティ参照 26 + 子ポインタ 86)、子はコード 3 コンテナ (パックレコードタイプ 1 墓標)；コンテナとルートは中央マッピングに入り、エントリ内のポインタは位置ヒント；ルート満杯時は背が高くなり、D8 (コアインデックス構造) 確定項目 6 に従い実装はこの手順を実施要 (ユーザー確定 K-R3-3)；コンテナ番号は 1 から開始、次番号 = ツリー内最大番号 + 1、ゼロルート時は 1 (確定項目 11 においてクローンヘッドのタイプ 1 コンテナ番号が 1 からツリー内単調割り当てとなるのと同一規則)。
2. **墓標レコード**：オブジェクト ID 8 + オブジェクト誕生世代 8 + 範囲起点 8 + 範囲終点 8 + 死亡 txg 8 = 40 バイト；全オブジェクトは [0, 2⁶⁴ − 1] を書き込み；32 KiB コンテナに ⌊(32768 − 136) ÷ 40⌋ = 815 件格納；死亡 txg = インテント発行パブリッシュ時の checkpoint_txg。
3. **オープンコンテナ**：新規レコードが存在するパブリッシュ時のみ書き換え、削除のないパブリッシュ時はそのままコピー；満杯時に初めてクローズ；クローズパスにはテスト専用の容量ノブを追加、ノブはディスクに永続化しない。
4. **回収**：第1版では墓標を回収しない、統計量 13、14 (D5 (スナップショット / 領域会計メカニズム) 確定項目 4) は第1版では未登録；10⁶ 回の unlink による墓標は約 76.7 MiB (2コピー)。
5. **系譜レコード**：専用系譜ツリー1本、day-1 にゼロルート登録、現行版に追従、追加のみで削除なし；子はコード 3 コンテナ (パックレコードタイプ 5、確定項目 11 で登録済みのタイプ 3 は再利用しない)；レコード = T_rb 8 + T_old 8 + H_cur 8 = 24 バイト (T_rb はロールバックパブリッシュ時の txg、T_old は R_old の txg、読み方は「T_rb の親は T_old」；H_cur は cur (ロールバック前の現行版) の inode 番号水位であり、cur 版から読み取り、パブリッシュパスから渡された「前回版」から動的計算しない；3セグメントのバイト順序は T_rb 8 | T_old 8 | H_cur 8、H_cur は既存2セグメントの後に追加され、旧2セグメントのオフセットは不変)、32 KiB コンテナに ⌊(32768 − 136) ÷ 24⌋ = 1359 件格納；管理者ロールバックパブリッシュ時に1件追加；系譜ツリールート満杯時も同様に背が高くなり、コンテナ番号は 1 から開始、次番号 = ツリー内最大番号 + 1、ゼロルート時 1。H_cur は会計内の inode 番号水位のステップ行 (D5 (スナップショット / 領域会計メカニズム) 確定項目 2 射程) と相互照合：各系譜レコードにおいて、ステップ行内で世代セグメント ≤ T_rb − 1 である行の最大水位が H_cur と**等しい**こと、H_cur が 0 の場合も赤判定 (W3 の C2aL 形態；I-9.30 (inode 番号水位のステップ行と水位、レコード、系譜の整合) として確立、実装済み)。クローンヘッドが存在する場合、本判定とステップ行はいずれもヘッド別 (ツリー ID 保持)：ステップ行は (統計量, ツリー) ごとに分割、クローン作成時に新ヘッド用に行を確保；ヘッド別とした後の系譜レコードのバイト形態は未確定 (推論、第1版は単一ヘッド)。
6. **登録**：ツリーテーブル 7 → 12 エントリ (`deleted_inodes`、墓標ツリー、系譜ツリー、logged_ops、退役ツリーゼロルート；logged_ops は D8 (コアインデックス構造) 確定項目 5 を参照、退役ツリーは確定項目 11 パックレコードタイプ 3 を参照)、ツリー ID 水位 65544 → 65549 (推論：5ツリーが各1つのツリー ID を占有；ツリー ID は 65536 から採番、D8 (コアインデックス構造) 確定項目 11)；新規 incompat ビットは追加せず、第1レイアウトラインに統合。

**射程**：第1版は単一ヘッド、スナップショットなし、清掃なし。ディスク走査再構築が墓標と系譜レコードをいかに読み取るかは本決定では定めず、未済の検査として記録 (K-W5b、C113 (走査再構築時に複数版ユニットの現行バージョン判定に入力がない) と同一グループ)。系譜ツリーのキャリア、パックレコードタイプ 5 は攻撃ゼロ巡；H_cur を追加したレコードの 24 バイト形態は攻撃1巡を通過 (W3 第3巡2回サンプリング)。

**根拠**：

- E83 (墓標の粒度)：第2回 (墓標レコード 40 バイト、レコード領域起点 136) 再現で第2条の1コンテナあたり 815 件および第4条の 10⁶ 回 unlink 約 76.7 MiB (2コピー)。
- E102 (ユニットクラス登録表とパックレコードユニット)：第2回 cap(136, 40) = 815、第2条の 815 と一致。
- 墓標ツリー、系譜ツリーの残りの数値：3者アームのモデルおよび複製プロトタイプ上で測定 (墓標判決、m3-delete-r3 判決)、リポジトリ内ハーネス上では未測定。
- 墓標ツリー：墓標判決 `research/prompts/m3-tomb-r1-main-verification.md` 第3節 W1 キャリアの行 (2巡；`research/prompts/m3-delete-r3-main-verification.md` 第3節「墓標第1巡各行」の行で子からタイプ 3 退役を削除)；ルート満杯時の成長：ユーザー確定 K-R3-3。
- 墓標レコード：墓標判決第3節 W2 フィールド表、W2 死亡 txg の2行；死亡 txg はユーザー確定 K-W5 に準拠、m3-delete-r3 判決第3節「墓標第1巡各行」の行。
- オープンコンテナ：墓標判決第3節 W4 オープンコンテナの行 (2巡)；ノブの非永続化：m3-delete-r3 判決第3節 G6-容量ノブの行 (ゼロ巡)。
- 回収：墓標判決第3節 W3 回収の行 (2巡)；76.7 MiB は m3-delete-r3 判決第3節 G8-残余数値の行 (40 バイト、1コンテナ 815 件)。
- 系譜レコード：m3-delete-r3 判決第3節 G4-系譜の配置先の行 (ゼロ巡)。
- 系譜レコードの H_cur 付加、「等しい」判定、cur からの読み出し、クローンヘッド存在時のヘッド別：W3 判決 `research/prompts/m3-w3-r2-main-verification.md` 第2節攻撃側 K3 (「被覆」比較により系譜誤書き込みの2変異を見逃し、「等しい」への変更で全赤；H_cur をパブリッシュパス供給の「前回版」から動的計算すると単一箇所の変異で2つの最強形態が 26 410 回全漏れ) および第3節 C2aL の行、`research/prompts/m3-w3-r3-main-verification.md` 第2節クローンヘッド各行 (ヘッド非分離の読み出しは2回サンプリングとも誤報、3形態ともヘッド別が必要)；ユーザー確定 2026-10-08 Y2「C2aL：会計にステップ行を保持 (推奨)」、原文は変更履歴に記録。実験なし：数値は3者アームのモデルおよびリポジトリ複製プロトタイプ上で測定、実験番号は未開設。
- 登録：墓標判決第3節登録の行 (ゼロ巡；2ツリー時の水位 19 → 21、当該判決当時のツリー ID 採番；ツリー ID は現在 65536 から採番)。
- 登録への退役ツリー追加 (ツリーテーブル 12 エントリ、水位 65549)：判決 `research/prompts/m3-retire-r1-main-verification.md` 第3節第1条および第5節；ユーザー確定「プール開設時に構築 (推奨)」、原文は変更履歴に記録。
- 系譜レコード3セグメントのバイト順序 T_rb | T_old | H_cur：第49バッチ W3 実装担当が選択 (実装担当報告第2節「系譜レコード3セグメント順序」行、`m3-batch49-w3-implementer-report.md`)、メイン agent 採用、ゼロ巡。実験なし：順序はバイト配置の選択であり、計測可能な量はない。
- ユーザー確定 K-U2「t1：インテント発行時に全オブジェクト墓標を書き込む」、K-W5「系譜レコード」、K-W5b「書き込み側は系譜と共に着地、読み取り側は未済の検査として記録」、K-R3-3「書き込みルートは背が高くなる (推奨)」、原文は変更履歴に記録。
- コンテナ番号は 1 から開始、墓標ツリーと系譜ツリールート満杯時の成長：`research/prompts/m3-blanks-r1-main-verification.md` 第2節コンテナ番号、墓標ツリールート成長の2行 (1巡)。
- ツリー ID は 65536 から採番：ユーザー確定 2026-10-01「丙 ツリー ID 全体シフト」、原文は変更履歴に記録。

**未済の検査**：C615 (走査再構築にマウント中のロールバックに対する入力がない)、C616 (系譜の親チェーンが txg 連番に依拠しアサーションがない)、C617 (削除後ロールバックが残した墓標に事前支払いのカバーがない)、C614 (ロールバック時に墓標ツリーを取り違えオブジェクト削除後に墓標のない単一イメージが判定不能)；C84 (墓標ユニットの粒度未定) は解消、C103 (コンテナインデックスの着地点なし) のタイプ 1 側は解消、C109 (どのツリーの key 死亡で墓標を書き込むか) は縮小。

```

**出处 `.claude/kb/decisions/18-ブロックに含める情報.ja.md:610-613`（整段抄，未转述）**

```markdown
## 過去の版

D18 (ブロックに含める情報) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/27-小データパッキングコンテナ.ja.md:1-6`（整段抄，未转述）**

```markdown
## D27 小データパッキングコンテナ —— 確定

[中文](27-小数据打包容器.md) · [English](27-small-data-packing-containers.en.md) · **日本語**

D27 (小データパッキングコンテナ) は小オブジェクトを全ユニットへパディングすることによる領域浪費（D4 (チェックサム位置) 確定項目 2: 512 B は 64 倍、1 KiB は 32 倍、4 KiB は 8 倍を占有）をどのように回収するかを管轄する：バックグラウンド整理が複数の小オブジェクトを1つの共有コンテナへ移動させ、書き込みパスは1バイトも変更しない。前提条件は D25 (対象ワークロードの優先順位) の smallfile 行であり、これがあって初めて本経路に分母が生じる。本決定が新設するのはユニット種別であり、軌道ではない：第2の検証セマンティクス、第2の会計台帳、第2のリカバリセマンティクスを新設せず、コンテナは依然として同一の全ユニットチェックサム検証、同一の会計台帳、同一のリカバリパスを辿る単一のユニットである。したがって D14 (双軌（大小ファイル / 永続一時）) 確定項目 6 が述べる「フォーマット級の第2軌」ではない——スロットごとの個別の nonce と MAC こそが第2の検証セマンティクスであり、確定項目 7 と確定項目 9 がその道を遮断したことで、この言明は条項に裏付けられている。また D14 (双軌（大小ファイル / 永続一時）) 確定項目 3 を覆すものでもない：インライン閾値は 0 のままであり、ファイルデータは1バイトもインデックスノードへインライン化されない；コンテナはデータ側のキャリアであり、インデックスノードではない。管轄外：書き込みパス上で2つのファイルの末尾を同一ユニットに詰め込むこと（D14 (双軌（大小ファイル / 永続一時）) 確定項目 3 が否と判定、その判定は不変であり、確定項目 10 にその理由がなぜ「移動」形態に及ばないかが記述されている）。

```

**出处 `.claude/kb/decisions/27-小データパッキングコンテナ.ja.md:7-281`（整段抄，未转述）**

````markdown
### 確定項目

| # | 項目 | 決定事項 |
|---|---|---|
| 1 | **形態はオンラインパッキングかバックグラウンド整理か、およびコンテナ破損救済策** | バックグラウンド整理によりコンテナへ移動、書き込みパスは不変；コンテナへ別途救済策を設けず D2 (RAID ストライプ方針) 確定項目 6 の2レプリカに依拠、双方同時の破損時のみ損失 **状態: 確定。** **進捗: 未実装。** |
| 2 | **境界線の画定位置** | ≤ 4096 バイトのオブジェクトのみコンテナへ収容（4 KiB 層 cap 7）、それ以上は現行規則通り単独ユニット化 **状態: 確定。** **進捗: 未実装。** |
| 3 | **コンテナが占有するユニット種別コードと種別同一性フィールド** | ユニット種別登録表の 4–15 のうち新しいコード（内容クラス）を占有しコード 3 は再利用しない；新種別独自に正当性不変条件を確立 **状態: 確定。** **進捗: 未実装。** |
| 4 | **コンテナインデックスの配置場所とキーエンコード** | コンテナインデックスは開設しない：解放判定は中央マッピングのスロットごとのポイントチェックを辿り、全死コンテナはバックグラウンド整理の領域走査で発見 **状態: 確定。** **進捗: 未実装。** |
| 5 | **コンテナごとの生存スロット数の配置場所** | 生存スロット数は保持しない **状態: 確定。** **進捗: 未実装。** |
| 6 | **デフラグのトリガー基準とバッチ規模** | パッキングは中央マッピングのキー順に進行、1トランザクションで複数マッピング葉を処理、各サイズ層ごとにコンテナを並行保持；ヒステリシスは誕生世代番号による；連続巡回とレート制限、水位は設けない **状態: 確定。** **進捗: 未実装。** |
| 7 | **スロットの nonce および MAC 保持媒体** | コンテナ全体で単一の MAC、スロット個別の nonce と MAC は作成しない **状態: 確定。** **進捗: 未実装。** |
| 8 | **データ側で採用するロード規約** | ①③ を採用、② は不採用、④ 固定長レコードは保持——スロット幅固定、オブジェクトはスロット幅へパディング **状態: 確定。** **進捗: 未実装。** |
| 9 | **コンテナ AAD の構成要素とドメインセパレータ** | AAD はコンテナ同一性にバインド、コード 3 の手法を踏襲し `(誕生ツリー, パッキングレコード種別, コンテナ番号, コンテナ誕生世代)` を採用 **状態: 確定。** **進捗: 未実装。** |
| 10 | **スロットの COW 堅持原則** | 書き出し済みコンテナ内のスロットに対するいかなるインプレース変更も許容しない；インプレース変更に頼らざるを得ない項目があれば D27 (小データパッキングコンテナ) 全体を放棄する **状態: 確定。** **進捗: 未実装。** |
| 11 | **射程: 連続小パケットの空間浪費回収に限定** | 連続小パケットの空間浪費回収のみを対象とし、パイプライン変更や複雑なループ導入を避ける；「追加ステップ」は I/O 回数と間接層数で計測 **状態: 確定。** **進捗: 未実装。** |
| 12 | **サイズクラステーブルのフォーマット組み込み要否** | フォーマットに導入しない：フォーマット層はコンテナヘッダがスロット幅 `W` とスロット数を宣言することのみを公約；利用可能な `W` の選定はデフラグポリシー **状態: 確定。** **進捗: 未実装。** |
| 13 | **初期版で採用するサイズクラステーブル** | スロット幅の等比数列を採用：`W_min` = 64、公比 1.125、末尾層を 4096 に固定、全 36 層；ポリシーでありフォーマットではない **状態: 確定。** **進捗: 未実装。** |

#### 確定項目 1: 形態はオンラインパッキングかバックグラウンド整理か、およびコンテナ破損救済策

**決定事項**: 書き込み時は一律に現行規則通り単独ユニット化する（D4 (チェックサム位置) 確定項目 2 のパディング挙動は不変）；オブジェクト書き込み完了後、D26 (バックグラウンド整理と配置回収) の常駐整理が共有コンテナへ移動させる。コンテナ数 = ⌈N / コンテナ容量⌉ でありパブリッシュリズムとは無関係；判定基準は「**既に書き込みが完了した**当該オブジェクトのサイズ」であり、操作実行中に分岐変数は存在せず、`.claude/rules/fs-design.md` ハード要件 5 を自動充足する。コンテナ破損時の損失規模と救済手段：別個の救済策は設けず、D2 (RAID ストライプ方針) 確定項目 6（w ≥ 2、冗長度0のストライプ発行禁止）の2レプリカに依拠する——利益倍率と恒久データ損失半径倍率は同一数値（いずれもコンテナ容量 `cap` に等しい）であり、両レプリカが同時に破損した場合にのみ失われる。承知の上で受け入れるコスト：オブジェクトはディスクへ2回書き込まれる（書き込みバイト数 = 現行 × (1 + 1/コンテナ容量)）；整理完了までは全く領域を節約できない（ピーク占有量 = 現行のパディング済み占有量）；1つのコンテナが読み出し不能になると当該コンテナ内の全オブジェクトが同時に失われ、C100 (inode コンテナ損失が恒久損失であり冗長条項がカバーしない問題) と同型だがデータ側のため一層重い——インデックスはユニットから再構築可能だがユーザーデータには上流が存在しない；整理自身も書き込みバイト数を消費する。

**適用範囲**: 小オブジェクトの書き込み後の配置場所およびコンテナ破損時の救済手段を管轄；書き込みパスは1バイトも変更しない。利益倍率は確定項目 6 の中央マッピングキー順の整理においてのみ成立する（コンテナ充填順に整理した場合3層すべて正味利益が負となる）。

**根拠**:

- E114 (小データパッキングコンテナの総勘定元帳): オンラインパッキングの利益は1回のパブリッシュに何個の小オブジェクトが含まれるかに依存し、1書き込みごとにパブリッシュすると総勘定元帳は負となる；バックグラウンド整理形態の利益はパブリッシュリズムから独立しているが、オブジェクトが2回書き込まれ整理完了まで領域が節約されないコストを伴う。
- E149 (パッキングコンテナ破損時の3つの救済策とそれぞれの代償): 多レプリカは損失半径を変えず、cap 引き下げは利益と半径を同時に縮小し、パッキング放棄は双方を1とする；追加救済策を設けない場合利益と半径は共に cap となり、両レプリカ同時破損時のみ失われる。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C186 (実行途中追加のアームが実行前失敗条項の射程に入らない問題): 本項目を産出した `packbg` アームが実行前に固定された失敗条項のいずれの射程にも含まれていない；C174 (smallfile 行の3列が未測定): カバレッジ分母；C202 (パッキング巡回の空転オーバーヘッドとヒステリシス K の数値欠如)；C391 (小データパッキングコンテナの D25 に対する挙証責任未済): D25 (対象ワークロードの優先順位) に「恒久的なフォーマット分岐が副次的ワークロードのみに供される場合、`.claude/rules/fs-design.md` の分岐判定基準に従い個別論証を要する」と規定され、`.claude/rules/fs-design.md` の5つのハード要件も挙証リストに含まれる。

#### 確定項目 2: 境界線の画定位置

**決定事項**: ≤ 4096 バイトのオブジェクトのみコンテナへ収容し、それ以上は現行規則通り単独ユニット化する。4 KiB 層 cap 7: 空間利益 6.91×、書き込みバイト比 1.1429、1コンテナ破損で7個喪失。16 KiB 枠はハードな境界であり取捨選択ではない：コンテナに2つ目のオブジェクトが入らない ⇒ 空間は全く節約されず書き込みバイト数は倍増する。境界線を 4 KiB とする決定は「カバレッジ」と「利益」の間で選択された点であり、8 KiB 層（2.98×）は正を保つものの顕著に減衰する。

**適用範囲**: 本数値群は E114 (小データパッキングコンテナの総勘定元帳) の幾何形状（コンテナヘッダ 107、スロット全自己記述 47、w = 2）に基づく；新種別自体のヘッダ幅とスロット幅のフィールド表は未定；回収損益分岐点はレプリカ数 w に伴い移動する。2 KiB 境界へ変更しても 2 KiB オブジェクトの格納方法は変わらず、2–4 KiB の区間が半径1となりそれぞれ 32 KiB ユニットを占有する形へ移行するのみである。

**根拠**:

- E114 (小データパッキングコンテナの総勘定元帳): バックグラウンド整理形態の利益はオブジェクトサイズに伴い減衰し、8 KiB 層では正を保つが、16 KiB 層では2回無駄書きとなり全く節約されない。
- E116 (パッキングコンテナの帳・補メタデータ書き込みと整理戦略): 損益分岐点に達するのは `cap > w + 1` のときかつそのときに限られ、w = 2 のとき臨界点は厳密に 8 KiB に一致し、4 KiB 境界は閉形式により支持され、利益曲線の減衰のみに依拠しない。
- E149 (パッキングコンテナ破損時の3つの救済策とそれぞれの代償): 2 KiB 境界は 2–4 KiB 区間を半径1へ置き換えるのみ；cap 上限は 512 B と 1 KiB の2層のみを抑制し 4 KiB 境界には作用しない。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C361 (パッキングコンテナ新種別のヘッダ幅とスロット幅のフィールド表欠落): フィールド表確定後に cap と利益を再計算；C174 (smallfile 行の3列が未測定)。

#### 確定項目 3: コンテナが占有するユニット種別コードと種別同一性フィールド

**決定事項**: ユニット種別登録表の 4–15 のうち新しいコード（内容クラス）を占有し、コード 3 は再利用しない。登録表 4–15 は逐字で「将来の内容クラスのために予約」（D18 (ブロックに含める情報) 確定項目 11）とあり、コンテナが収容するのはファイルデータである ⇒ 内容クラスに該当する。新種別は独自に正当性不変条件を確立し、I-1.7 (パッキングコンテナ正当性と判定順序) を準用しない。

**適用範囲**: 確定したのは「どのコードを占有するか」および「新種別独自に正当性不変条件を確立し I-1.7 (パッキングコンテナ正当性と判定順序) を準用しない」点のみ；種別同一性セグメントに含めるフィールドは未確定——確定項目 7 と確定項目 9 はコンテナ全体で単一の MAC と AAD 構成を定めたのみで、フィールド表は未提示；4–15 のどのコードを占有するか未固定。コード 3 再利用は C154 (ストライプメンバー表にパッキングレコード種別が未登録) とも衝突する：ストライプメンバー表が非公式に未登録種別コードを占有中。

**根拠**:

- 実験なし: どのコードを占有するかは登録表の分類であり、3つの条項推論を根拠とする——コード 3 再利用はコンテナヘッダ内既存フィールドの再解釈か I-1.7 (パッキングコンテナ正当性と判定順序) の判定チェーン変更のいずれかを要し、双方が確定項目 11 の射程外表に含まれる；コード 3 の第2級登録表でサイズ層ごとに固定幅種別を登録する場合、オブジェクトを層幅へパディングする必要があり（600 B を 1024 へパディングすると 41% の浪費）、D27 (小データパッキングコンテナ) が解消すべき浪費を再導入する；incompat コストは両経路で同一（D18 (ブロックに含める情報) 確定項目 11 逐字「新ユニット種別追加 ⇒ incompat；新パッキングレコード種別追加も既定で incompat」）であり節約にならず、再利用の根拠を構成しない。
- 三方論証: コードおよびフィールド表のラウンド（`_d27-format-crypto-background.md`、クラウド脚 `d27-format-crypto-forward-sonnet.md`、`d27-format-crypto-reverse-opus.md`）および第5ラウンド（`_d27-r5-boundary.md`、`d27-r5-forward-sonnet.md`、`d27-r5-attack-sonnet.md`、`d27-r5-local.out`）。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C361 (パッキングコンテナ新種別のヘッダ幅とスロット幅のフィールド表欠落)；C362 (パッキングコンテナ新種別の登録表内コード未固定)。

#### 確定項目 4: コンテナインデックスの配置場所とキーエンコード

**決定事項**: コンテナインデックスは開設しない。確定項目 11 が課す新構造に対する挙証責任は「それがなければ D27 (小データパッキングコンテナ) が成立しないことの証明」であるが、ここでは証明できない：実行時パスにおいて「このコンテナに生存スロットが何個残っているか」を問う必要はない；全死コンテナはバックグラウンド整理がその領域を走査した際に発見される——D26 (バックグラウンド整理と配置回収) 確定項目 2 の整理意図目標状態は逐字で「**`R` 内に**割り当て世代 ≤ `c0` かつ未解放の配置先が存在しない」とあり ⇒ デフラグ担当は元より領域ごとに配置先を列挙している；解放判定は中央マッピングのスロットごとのポイントチェックを辿る（D19 (ブロックポインタの構造と幅の予算) 確定項目 5: 中央マッピングは逆参照および解放判定の唯一の入口）；会計規律の第3枠（バックグラウンド、再開可能、非決定パス、戒律なし）に着地——`.claude/rules/fs-design.md` の3枠規律は逐字で「整理」を指名、D26 (バックグラウンド整理と配置回収) 本文も同様に逐字で「整理は第3枠に属し、**戒律なし**——走査可能、低速可能」と規定。

逆参照には新ポインタを要しない：スロット自身が論理同一性5つ組を保持（D1 (データ移動性 / 逆インデックス) 確定項目 5 は自己完結不変データに単一のポインタを要求）、中央マッピングのキーは別グループのフィールド（D19 (ブロックポインタの構造と幅の予算) 確定項目 6: コード 1 のキー = 種別タグ 1 + 誕生ツリー 8 + 誕生 txg 8 + 書き込みシーケンス 10、計 27 バイト） ⇒ 位置エントリは従前通りコンテナ所在の配置先を指し、全ユニット読み出し後に中央マッピングキーによりコンテナ内でスロットを照合照会——スロット内キーの各セグメントはパッキング前ユニットヘッダの値をバイト単位で踏襲、種別タグは恒常的にコード 1（D19 (ブロックポインタの構造と幅の予算) 確定項目 6 厳格化 ⑩㈠）；「これがコンテナか単独ユニットか」はユニット自身の種別タグが回答（I-1.6 (クラスラベルが登録され3箇所で一致すること)）。D19 (ブロックポインタの構造と幅の予算) 確定項目 3 / 4 / 5 は変更ゼロ、マッピング値に判別ビットは追加しない。⇒ D27 (小データパッキングコンテナ) が新設する要素は2点のみ：単一のユニット種別とそのフィールド表、およびバックグラウンド整理走査内の追加分岐1つ；中央マッピング、割り当てレコード、会計キーは1バイトも変更しない。

**適用範囲**:

- 回収時期に上限はない：全死コンテナはデフラグ担当がその所在領域を走査するまで待機し、整理の起動・停止時期は D26 (バックグラウンド整理と配置回収) 確定項目 1 が決定（D26 (バックグラウンド整理と配置回収) 確定項目 4 はポリシーの半分を D3 (領域割り当て) 確定項目 8 および D26 (バックグラウンド整理と配置回収) 確定項目 1 へ着地）；この間その 32768 バイトは割り当て済みだが参照者不在となり、D26 (バックグラウンド整理と配置回収) 確定項目 1 の「回収不能バイト」水位と同種であり、D27 (小データパッキングコンテナ) が新設した形態ではない。
- コストは正確に記録：位置エントリ内の 4 バイト暗号文チェックサムは 32 KiB 全ユニットをカバー（D4 (チェックサム位置) 確定項目 1） ⇒ 同一コンテナの N 本のマッピングエントリがそれぞれ同一値を格納；コンテナが再書き込みされた場合、N 本すべてを更新する必要がある。
- スロット内キーの各セグメントは決定事項段に明記済み（C288 (マッピングキー決定の波及修正が他決定へ未反映) ⑤）；スロットが伴う他フィールド（スロット表、オブジェクト実長格納位置、バイト配置、スロット幅）は確定項目 3 の未確定種別同一性表と共に決定（C361 (パッキングコンテナ新種別のヘッダ幅とスロット幅のフィールド表欠落)）。

**根拠**:

- 実験なし: 依拠するのは機械的全数調査と確認——3本の脚が割り当て、ENOSPC 受入、defer ウィンドウ、スナップショット生死判定、`df` の各判定パスで「コンテナ」のヒット0を確認、D5 (スナップショット / 空間記帳メカニズム) 確定項目 4 の会計統計量にコンテナ次元は存在しない；攻撃側脚が構築した「恒久リーク」は「発見にはインデックスが必須」という前提に立脚しており、D26 (バックグラウンド整理と配置回収) 確定項目 2 の領域列挙により反証された。
- 三方論証: 第5ラウンド（`_d27-r5-boundary.md`、`d27-r5-forward-sonnet.md`、`d27-r5-attack-sonnet.md`、`d27-r5-local.out`）。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C361 (パッキングコンテナ新種別のヘッダ幅とスロット幅のフィールド表欠落): スロット側。

#### 確定項目 5: コンテナごとの生存スロット数の配置場所

**決定事項**: 生存スロット数は保持しない。確定項目 10 によりコンテナ内保持は排除されている（コンテナ内保持はオブジェクト搬出後にコンテナ内の死スロットフラグを更新しに戻ることを要する）；確定項目 11 が課す新構造の挙証責任も果たせない：実行時パスにおいて「このコンテナに生存スロットが何個残っているか」を問うものは存在せず、全死コンテナはバックグラウンド整理の領域走査で発見される（確定項目 4）。

**適用範囲**: 死スロットフラグ / 生存ビット / 参照カウントをコンテナ内に保持することを厳禁（確定項目 10）；回収時期に上限がない代償は確定項目 4 の適用範囲に記載。

**根拠**:

- 実験なし: 確定項目 4 と同一の機械的全数調査——割り当て、ENOSPC 受入、defer ウィンドウ、スナップショット生死判定、`df` の各判定パスで「コンテナ」のヒット0、D5 (スナップショット / 空間記帳メカニズム) 確定項目 4 の会計統計量にコンテナ次元なし。
- 三方論証: 第5ラウンド（`_d27-r5-boundary.md`、`d27-r5-forward-sonnet.md`、`d27-r5-attack-sonnet.md`、`d27-r5-local.out`）。
- ユーザー決定: 原文は変更履歴にある。

**未済**: なし。

#### 確定項目 6: デフラグのトリガー基準とバッチ規模

**決定事項**: パッキングと回収は別個の処理であり動作が異なる——パッキング（小オブジェクトのコンテナ搬入）は**中央マッピングのキー順**に従い、コストはマッピング書き込みが主導；回収（全死コンテナの解放）は D26 (バックグラウンド整理と配置回収) の**領域走査**に便乗し、確定項目 4 と確定項目 5 はこれによって新構造の新設を回避。確定された5原則：

1. **順序**: パッキングは中央マッピングキー順に走査し、**コンテナ充填順によらない**。
2. **バッチ規模**: 1回の整理トランザクションは連続キー区間をカバーし、長さは**マッピング葉全体の整数倍**とする ⇒ **各葉の COW は1回のみ**。
3. **複数層並行**: キー順に沿って確定項目 13 の36層それぞれ1つのコンテナを並行して開き、オブジェクトを層別に投入。**マッピングエントリはキー順に更新** ⇒ 層別投入は原則2を損なわない。
4. **ヒステリシス**: `現 checkpoint 番号 − オブジェクト誕生世代 ≥ K` を満たすオブジェクトのみ移動。**新フィールドゼロ、新追跡ゼロ**——COW 下でオブジェクトを変更すると新たな誕生世代が生成されるため、「最終変更からの経過時間」と「誕生からの経過時間」は同値であり、誕生世代は元よりオブジェクトヘッダ内に存在する。
5. **開始条件**: **連続巡回とレート制限**、水位は設けない；新状態ゼロ、新水位ゼロ、D26 (バックグラウンド整理と配置回収) 確定項目 1 は変更せず；確定項目 11「複雑なループの非導入」に合致。

**適用範囲**: パッキング専用の水位は開設しない——D26 (バックグラウンド整理と配置回収) 確定項目 1 の3水位（完全空クラスタ化セグメント数 / 断片化度 / 回収不能バイト）はディスクの満杯度、断片化度、回収不能量を測定するものであり、「パッキングでどれだけ空くか」を測定するものではない。`K` の**形態**は「往復コスト ÷ 単回移行利益」として固定するが、**数値は churn 確率を要し**リポジトリ全体で未測定；アイドル時のマッピング読み取り空転オーバーヘッドは未測定。

**根拠**:

- E116 (パッキングコンテナの帳・補メタデータ書き込みと整理戦略): 損益分岐比率の正負は整理戦略により決定——キー順走査時は各マッピング葉の COW は1回で損益分岐達成、コンテナ充填順かつバッチ1のときはマッピング葉がランダム分散し損益分岐不能。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C202 (パッキング巡回の空転オーバーヘッドとヒステリシス K の数値欠如)。

#### 確定項目 7: スロットの nonce および MAC 保持媒体

**決定事項**: コンテナ全体で単一の MAC を採用し、スロット個別の nonce と MAC は作成しない；D19 (ブロックポインタの構造と幅の予算) 確定項目 3 および D9 (暗号化) 確定項目 4 は1文字も変更しない。スロット個別の MAC は第2の検証セマンティクスに該当し、D14 (双軌（大小ファイル / 永続一時）) 確定項目 6 はフォーマット級の第2軌の新設を明示的に禁止している——第2の検証セマンティクス、第2の会計台帳、第2のリカバリセマンティクスは新設しない。

**適用範囲**: C68 (不変条件ホワイトリストと確定条項の衝突) に懸案の nonce 帰属問題を「2経路それぞれの帰結」から「単一経路の帰結」へと収束させた——その枠は確定項目 7 により「同一の穴」と逐字で指名されている；1つへの収束は未済解消と同義ではない。暗号化は初期実行可能版へ導入されず（D9 (暗号化) 確定項目 10）、本条項は暗号化有効時にのみ実行パスを持つ。

**根拠**:

- 実験なし: セマンティクスの取捨選択であり測定可能な量はない——スロット個別 MAC は第2の検証セマンティクスとなり D14 (双軌（大小ファイル / 永続一時）) 確定項目 6 と衝突。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C68 (不変条件ホワイトリストと確定条項の衝突): それ自体は不変。

#### 確定項目 8: データ側で採用するロード規約

**決定事項**:

| 規約 | データ側での採否 | 根拠 |
|---|---|---|
| ① 1コンテナ1内容種別 | **採用** | I-1.7 (パッキングコンテナ正当性と判定順序) 逐字「コンテナヘッダは1つのパッキングレコード種別、1つの誕生ツリーを宣言」 |
| ② 1コンテナ1世代 | **不採用** | D18 (ブロックに含める情報) 確定項目 11 の規約②は「世代ごとに回収されるパッキングレコード種別へ一般化」との射程を持ち個別死亡種別を免除；データ側オブジェクトは個別に死亡 |
| ③ 1コンテナ1ツリー | **採用** | ① と同一不変条件 |
| ④ 記録固定幅 | **採用（保持）** | ユーザー決定 |

**④ の固定形態**: コンテナは1つのスロット幅 `W` を宣言し、コンテナ内全スロットは同幅、オブジェクトは `W` へパディング；スロット `i` の開始点 = ヘッダ + `i × W`、**スロットディレクトリは不要**；正当性判定は I-1.7 (パッキングコンテナ正当性と判定順序) と同型（`スロット数 × W ≤ 宣言長`）、ただし新種別独自の不変条件が担持。

**適用範囲**: ④ は2つの代償を支払う：パディング浪費の再発——オブジェクトサイズが層間に落ちる際層幅へパディング、層区分は確定項目 13 が規定；デフラグ担当はサイズ別に分類する必要があるが「中央マッピングキー順走査」とは矛盾しない：デフラグ担当はキー順に走査しつつ各層1つのコンテナを並行保持、オブジェクトを層別に投入、マッピングエントリはキー順に更新、コンテナは層別に満杯化。代償として同時オープンコンテナ数 = ツリー数 × 層数となる。

**根拠**:

- E119 (固定幅スロットのパディング代償): 固定幅自体は高価ではなく層区分が粗いことが高価；「各固定幅アームが全分布で30%超損失」という反向条項は誘発されず、④ は再開不要。
- ユーザー決定: 原文は変更履歴にある（メイン agent がスロットディレクトリへの変更を提案したがユーザーが固定幅保持を選択）。

**未済**: C106 (オープンコンテナ数の上限)。

#### 確定項目 9: コンテナ AAD の構成要素とドメインセパレータ

**決定事項**: AAD はコンテナ同一性にバインド、フィールドはコード 3 の手法を踏襲し `(誕生ツリー, パッキングレコード種別, コンテナ番号, コンテナ誕生世代)` を採用——D18 (ブロックに含める情報) 確定項目 11 が既にコード 3 用に規定済みであり、**新設された体系ではない**。コンテナ全体で単一の MAC を採用した結果、本枠の選択肢は1つのみとなる：

| 選択肢 | 成立 / 不成立の理由 |
|---|---|
| AAD は**オブジェクト**同一性 + コンテナ全体単一 MAC | **機構的に排除**: N 個の AAD に対し 1 個の MAC ⇒ N−1 個のオブジェクトが検証失敗。D9 (暗号化) 確定項目 6 がエクステント分割を排除したのと同一の機構であり逐字「不整合が正常操作の必然的結果となる ⇒ 排除」 |
| スロット個別 nonce・MAC + AAD オブジェクト同一性 | 確定項目 7 で閉鎖された経路を前提とする |
| **AAD コンテナ同一性バインド**（本案採用） | 1 AAD 対 1 MAC、自己無撞着 |

**適用範囲**: 数値化されていないコストを承知で受容——確定項目 6 のバックグラウンド整理はオブジェクトを移動させ、AAD がコンテナ同一性にバインドされていることは**オブジェクトがスロットごと別コンテナへ移動した際 AAD が変化し再暗号化を要する**ことを意味する；D27 (小データパッキングコンテナ) 自身が提案時に逐字で「その代償は E114 (小データパッキングコンテナの総勘定元帳) で未計算」と記述しており、決定事項もこれを変更していない。暗号化有効時にのみ実行パスを持つ。

**根拠**:

- 実験なし: 1 AAD 対 1 MAC のみが自己無撞着であることは、確定項目 7 および D9 (暗号化) 確定項目 6 の不整合機構から演繹されたものであり、新たな取捨選択ではない。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C240 (搬送時の再暗号化コストが未計算)。

#### 確定項目 10: スロットの COW 堅持原則

**決定事項**: スロットは依然として COW であり、本原則に違反してはならない；違反する場合は放棄する。⇒ **書き出し済みコンテナ内のいかなるスロットに対するインプレース変更も許容しない。** 本条項は D27 (小データパッキングコンテナ) の**拒否権枠**である：後続のいずれかの分項が「コンテナ内の当該バイトを事後更新する」ことでのみ達成可能な場合、D27 (小データパッキングコンテナ) 全体を放棄し、**本条項を緩和してそれに妥協することを禁ずる**。その場で固定された5原則：

| 原則 | 固定内容 |
|---|---|
| コンテナの変更可能性 | **書き出し後不変**。インプレーススロット変更、部分コンテナ COW、スロット断裂は全面的に存在しない |
| パッキング済みオブジェクトの変更 | **無変更の**単独ユニット書き込みパスに従い新ユニットを書き出し（「外部搬出」）、コンテナは1バイトも変更しない |
| 確定項目 5（生存スロット数保持場所） | **コンテナ内に保持不能**——コンテナ内保持はオブジェクト搬出後にコンテナ内死スロットフラグ更新を要するため |
| 死スロットフラグ / 生存ビット / 参照カウント | **コンテナ内保持を厳禁**、「確定項目 5（生存スロット数保持場所）」と同一の理由 |
| デフラグによる死スロット回収手法 | **完全再書き込み**（新コンテナ書き出し、旧コンテナ全体解放）のみとし、**その場圧縮は禁止** |

「外部搬出」と確定項目 1 は同一機構の2回適用である；もう1つの自己無撞着形態「コンテナ全体 COW」は最悪 58倍の書き込み増幅を招くため不採用。

**適用範囲**:

- 確定項目 11 と対をなす：確定項目 10 は「インプレース変更の禁止」を、確定項目 11 は「外延拡大の禁止」を管轄し、双方が拒否権枠であり、いずれかの分項のために緩和することは許されない。
- D14 (双軌（大小ファイル / 永続一時）) 確定項目 3 の関門に回答。「ファイル間末尾パッキング」を否と判定した D14 (双軌（大小ファイル / 永続一時）) の理由は「2ファイルが1ユニットを共有 ⇒ 一方を書くとユニット全体を COW ⇒ **他方ファイルのポインタも変更必須**」であった；しかし「外部搬出」形態では一方のオブジェクト書き込み時にコンテナを全く COW しない——コンテナは1バイトも動かず、他方オブジェクトのポインタは1本も変わらない ⇒ その理由は本形態には成立しない。3つの限定、いずれも省略不能：① D14 (双軌（大小ファイル / 永続一時）) の「否」判定自体は不変——**書き込みパス上で**2ファイルの末尾を同一ユニットに詰めることを管轄しており、「書き込み完了後バックグラウンドでコンテナへ移動」とは別事象であり、その**理由**が後者に及ばないことを述べるのみ；② 「他方オブジェクトのポインタも変更必須」は**デフラグ時**には成立する：コンテナを圧縮または再書き込みする際、コンテナ内 N 個のオブジェクトの中央マッピングエントリは全更新を要する（E116 (パッキングコンテナの帳・補メタデータ書き込みと整理戦略) はこれを移行書き込みに算入）；デフラグはバックグラウンド再開可能パスであり `.claude/rules/fs-design.md` 会計規律第3枠（戒律なし）に属し、書き込みパスの即時判定とは別枠；③ 本条項は D14 (双軌（大小ファイル / 永続一時）) 確定項目 3 が**逐字で問うた事項**（「複数ファイルによる単一原子単位の共有」）に回答しており、**性能数値で回答したのではない**——D14 (双軌（大小ファイル / 永続一時）) 確定項目 3 は逐字で「性能数値はこれに回答不能」と規定している。

**根拠**:

- 実験なし: ユーザーが定めた拒否権原則であり測定可能な量はない；D27 (小データパッキングコンテナ) に明確で観測可能な放棄条件を与える。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C190 (コンテナ内にインプレース可変フィールドが出現した際に赤となる検査の欠落): その検査を得るまで本条項は記述された原則にとどまる。

#### 確定項目 11: 射程: 連続小パケットの空間浪費回収に限定

**決定事項**: D27 (小データパッキングコンテナ) は連続小パケットの空間浪費問題のために存在し、**パイプラインを変更したりより複雑なループを導入するような再構築物ではない**。候補形態が提示された場合この2問を問い、双方が「否」の場合にのみ射程内と判定：

1. 「パッキング済みオブジェクトの読み出し」パイプラインに**追加ステップ**をもたらすか？
2. **既存条項のフィールドの意味を変更**するか？

判定基準①の「追加ステップ」は**I/O 回数と間接層数で計算し、操作数では計算しない**：

| | 未パッキング | パッキング済み | 射程外の形態（読み出しパス上のコンテナインデックス） |
|---|---|---|---|
| I/O 回数 | 1 | 1 | 2 |
| 間接層数 | 1 | 1 | 2 |
| 読み出し時照合構造数 | 1 | 1 | 2 |

⇒ コンテナ読み出し後に**既に取得済みのバッファ**内で中央マッピングキーによりスロットを照合照会すること（確定項目 4）は**追加ステップとみなさない**——I/O を発行せず、間接層を増やさず、他の構造を照合しないため。判定基準①が阻止するのは「中央マッピング → コンテナインデックス → コンテナ」の経路である。

**射程内**: ユニット種別を1つ追加すること（登録表コード 4–15 は逐字で「将来の内容クラスのために予約」、1つの incompat ビットを占有）——それは**内容種別の追加**でありパイプライン変更ではない；および新種別自身のフィールド表の配置。**射程外**:

| 射程外の動作 | 理由 |
|---|---|
| コード 3（パッキングレコードユニット）コンテナヘッダ内既存フィールドの意味の再解釈 | 既存条項のフィールド意味を変更 |
| 割り当てレコードの値への次元追加（D3 (領域割り当て) 確定項目 7 は `割り当て世代 8` と規定） | 既存条項のフィールド意味を変更 |
| 会計キーへのコンテナ次元追加（D5 (スナップショット / 空間記帳メカニズム) 確定項目 1 は `(統計量タグ, ツリーID, デバイス, 世代)` と規定） | 既存条項のフィールド意味を変更 |
| 中央マッピング値の形態変更（判別ビット追加、同一性参照への変更） | 逆参照パイプラインに追加ステップを導入 |
| I-1.7 (パッキングコンテナ正当性と判定順序) の「レコード幅 == 登録表登録幅」ステップの変更 | 既存条項を変更；新種別が独自に判定を新設する場合は免除 |
| 第2の検証セマンティクス / 第2の会計台帳 / 第2のリカバリセマンティクスの新設 | D27 (小データパッキングコンテナ) 冒頭文で排除済み |

**新構造に対する挙証責任**: D27 (小データパッキングコンテナ) が追加しようとするいかなるツリー / リング / インデックスも、**「それがなければ D27 (小データパッキングコンテナ) が成立しないこと」を事前に証明しなければならない**。証明できないものは追加しない。本原則は確定項目 4（コンテナインデックス）および確定項目 5（生存スロット数）に直結する。

**適用範囲**: 確定項目 10 と対をなす拒否権枠（確定項目 10 の適用範囲）。

**根拠**:

- 実験なし: 射程はユーザーが画定した境界；「追加ステップ」を I/O 計測条件で測る理由は、判定基準①と本項の決定事項が「パイプライン」と「より複雑なループ」を記述し、構造と間接性を測定対象とし命令数ではないため。
- 三方論証: 第5ラウンド（`_d27-r5-boundary.md`、`d27-r5-forward-sonnet.md`、`d27-r5-attack-sonnet.md`、`d27-r5-local.out`）、3本中2本が操作数で判定したが、メイン agent 判定とユーザー決定は I/O 計測条件を選択。
- ユーザー決定: 原文は変更履歴にある。

**未済**: なし。

#### 確定項目 12: サイズクラステーブルのフォーマット組み込み要否

**決定事項**: フォーマットに導入しない。フォーマット層は2点のみを公約する：コンテナヘッダ内に**スロット幅 `W`** フィールド（境界 ≤ 4096 ⇒ 2バイトに収まる）と**スロット数**フィールドを有すること。スロットの読み出しは `ヘッダ + i × W` であり——**読み出し側はサイズクラステーブルの形状を知る必要が全くない**。⇒ **どの `W` が利用可能かはデフラグ担当のポリシーでありフォーマット定数ではない。** テーブルを変更してもフォーマットは不変であり、旧コンテナは自身が宣言した `W` を保持して通常通り読み出せる。

**適用範囲**: 「将来任意に変更可能」を意味しない：旧コンテナの `W` は書き込み済みであり、テーブル変更は**新しく書き出される**コンテナにのみ影響する；旧コンテナを新テーブルに従って再配置するにはデフラグ担当による完全な再書き込みを要する（確定項目 10 はインプレース変更を禁止）。`W` フィールド自体の幅と配置は本項の射程外であり、確定項目 3 のフィールド表と共に決定。

**根拠**:

- E119 (固定幅スロットのパディング代償): 層の切り方はオブジェクトサイズ分布に依存し、同一テーブル群でも3分布間で優劣順位が変動——サイズクラステーブルは分布に応じて選択するほかなく、分布はリポジトリ全体で未測定であり、フォーマットへ凍結することは未知量を恒久公約として凍結することと同義。
- E121 (cap による層分割と層数の最適点): 最適層数はワークロード規模にも依存し、層数が多ければ良いとは限らない。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C174 (smallfile 行の3列が未測定)。

#### 確定項目 13: 初期版で採用するサイズクラステーブル

**決定事項**: スロット幅の等比数列を採用、`W_min` = 64、公比 1.125、末尾層を 4096 に固定、全 36 層（等比のサイズクラス表で公比をどう決めるか）。**層幅テーブル（括弧内は当該層の `cap`、すなわち1コンテナあたりのオブジェクト数）**:

```
  64(305)   72(284)   81(263)   92(241)  104(222)  117(204)
 132(186)  149(170)  168(154)  189(140)  213(127)  240(115)
 270(104)   304(94)   342(84)   385(76)   434(68)   489(61)
  551(54)   620(49)   698(44)   786(39)   885(35)   996(31)
 1121(28)  1262(25)  1420(22)  1598(19)  1798(17)  2023(15)
 2276(14)  2561(12)  2882(11)   3243(9)   3649(8)  4096(7)
```

ポリシーでありフォーマットではない（確定項目 12）: C174 (smallfile 行の3列が未測定) が解消されワークロード測定後に変更可能でフォーマット不変；ただし**新しく書き出されるコンテナにのみ影響**し、旧コンテナの新テーブル準拠再配置はデフラグ担当の完全再書き込みを要する（確定項目 10 はインプレース変更を禁止）。

**適用範囲**: 分布と規模が共に未知のときは「特定の1枠が最適」なテーブルではなく「どの枠でも著しい不利益を被らない」テーブルを選択すべきである。36個の並行オープンコンテナ = 1.15 MiB であり、**デフラグ担当が同時に処理するツリー数を乗算**する必要があり、その乗数は D19 (ブロックポインタの構造と幅の予算) 確定項目 6 に連動：キーはクラスラベル後に誕生ツリーで始まるが、それに応じた再計算は未実施。

**根拠**:

- E120 (等比のサイズクラス表で公比をどう決めるか): 等比テーブルの `W_min` 以上における最悪相対浪費は分布に依存しない；下限領域は絶対バイト数で評価する必要がある。
- E121 (cap による層分割と層数の最適点): 層数が多ければ良いとは限らない；`cap` 等比は全体的に劣る；`W_min` を 64 未満に圧縮しても利益はなく、128 に引き上げると顕著な代償が生じる。
- E119 (固定幅スロットのパディング代償): 4層等間隔は実行前に固定された反向条項を誘発、層は小サイズ側に集中させるべきであり等間隔分割は不適。
- E116 (パッキングコンテナの帳・補メタデータ書き込みと整理戦略): 損益分岐には `cap ≥ 4` が必要；本表の全層で `cap` ≥ 7。
- ユーザー決定: 原文は変更履歴にある。

**未済**: C174 (smallfile 行の3列が未測定)；C106 (オープンコンテナ数の上限): 同時オープンコンテナ数にツリー数を乗算。

````

**出处 `.claude/kb/decisions/27-小データパッキングコンテナ.ja.md:282-285`（整段抄，未转述）**

```markdown
## 履歴

D27 (小データパッキングコンテナ) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `crates/singlefs-checker/invariants/01-块自描述.md:1-30`（整段抄，未转述）**

```markdown
## I-1 块自描述 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-1.1 | 块头自述逻辑地址 | 任一单元，其头记录的**类内身份**与实际引用一致——数据单元的身份是五元组（锚点偏移即它的逻辑地址）；索引节点是树 ID + 层级（+ 该树已定携带的 key 区间，D18（块里携带什么信息）已定项 2 口径）；打包记录单元（码 3）是（出生树 ID, 打包记录类型, 容器号, 容器出生代）四元组，「父级」按打包记录类型分——类型 1 走墓碑树码 2 节点的身份引用（D18（块里携带什么信息）已定项 24；C103（容器索引无落点）只剩条带成员表那一半），类型 2 走 inode 树内部节点条目的身份引用（I-9.2（条目身份与子头相符）），**类型 4（实例表）走根记录的物理指针，不属于任何树、不进容器索引**。**自证单元（journal 记录 / 根槽 / 系统配置槽 / 类型 4 实例表单元）显式豁免**：无父引用无树身份，自证纪律在 I-7（根环）/ I-8（journal）两类与 D22（单元原子性怎么合成）已定项 21；多层码 2 树（记账树、中央映射树）的索引节点，身份里的层级与 key 区间按引用它的父条目核：层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间是子树覆盖区间；按位置寻址的两棵树（分配记录树、extent 树），节点头里的层级与 key 区间等于它的位置规定的那一层、那一段；内部条目的 key 是某个孩子那一段的起点，孩子按位置严格递增；分配记录叶里每条记录的起点与末槽都落在叶罩的那一段里；extent 上段叶条目的 inode 号、下段叶记录的单元号落在叶罩的那一段里，上段叶条目的标签只能是 0 到 4（标签 0 载荷全零、标签 1 与 3 载荷末 2 字节为零；标签 3、4 是判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5 的「最后一个存在的单元之后到 size 是洞」，用户 2026-10-16 定「加标签声明洞」，洞与单元数对不对得上 size 归 I-4.6（中间态其余不变量成立））；码 2 节点里的条目按 key 严格递增，同 key 两条判红（`deleted_inodes`（D8（核心索引结构） 已定项 16）里同一个号两条就归这一条，不归 I-9.24（删除队列至多一条 key））；根之下没有空节点（这一半 checker 由实二五补）。 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判；`walk::Walk::walk_code_two_subtree`；坏镜像用例 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`；`crates/singlefs-checker/src/position_addressed.rs`；码 2 节点 key 严格递增由 `crates/singlefs-checker/src/lib.rs` 的 `check_index_node_keys` 判，`deleted_inodes` 同号两条的坏镜像 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` 的 `a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant`） |
| I-1.2 | 块头写序已发布 | 任一单元，其头记录的写序 (i, n)（D18（块里携带什么信息）已定项 7 / 11 的字段）与诞生代号 b 按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的全局已发布谓词判成已发布：i < 挂载根实例且无该实例的行 ⇒ 已发布，有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub（所选根那个实例的 T_pub = 重放之后那个根的 txg，严格介于所选根的实例与新实例之间、又不是切换照旧事务写序主人的实例写 (i, 0, 0)，切换的所选根按 D23（journal 的角色与格式） 已定项 14 取被重发的在飞 checkpoint 所基于的根；W = **属于实例 i 的**、被这次重放施加的最大事务号，切换时是被照旧的、写序属于 i 的最大事务号；同一次恢复写出的那批行里只有所选根那个实例的 W 可能非 0，同一次切换写出的那批里只有被照旧事务的写序主人的 W 可能非 0，表上允许有多行非 0，它们来自不同次的恢复或切换；2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；i == 挂载根实例 ⇒ b ≤ 挂载根的 checkpoint_txg；i > 挂载根实例 ⇒ 判损坏。**例外**：打包记录类型 4 的实例表单元不走这条谓词，它的现行版本由根记录持有的物理指针唯一确定。**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**——这是扫描方向独立于遍历方向判它的依据（D13（验证路线） 已定项 12 双方向要求）；2026-09-05 前它只比诞生代号与当前 txg，崩溃后 txg 重发让那条判据对同 txg 两条时间线失效（E104（扫描重建的现行版本判定）：无实例表时崩溃世界复活 6） | 已实现（池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`，**只判遍历方向那一半**：走读到的每个被引用单元，头里的诞生代号 b、写序的实例代号 i 与尾段（码 1 是写序的事务号 n、码 2 / 码 3 是出生序号）要与**引用它的那条指针**记的相同——不同则「头记录的写序」不是这个单元的真实身份，谓词判的就是假输入；口径同恢复路径的前置检查（`crates/singlefs-core/src/recovery.rs` 五处报 I-1.2（块头写序已发布） 的 `InvariantViolated` 里「出生身份与指针不符」那三处；「诞生于根之后」那两处是谓词那一半，归 I-4.2（无被引用未提交块）），checker 按字段表自己解一份。⚠️ **扫描方向那一半（「任一单元」都按谓词判、i > 挂载根实例 ⇒ 判损坏）对着一份崩溃镜像判不了，没写**：① 判不成已发布本身不是违例——本行判据自己写着那是未发布的撕裂事务垃圾或被抛弃时间线的残留；② 「i > 挂载根实例 ⇒ 判损坏」只在恢复刚发布完新根的那一刻成立，而上一次挂载取了号、写出单元、一个根都没发布就崩时，盘上合法地留着 i > 最新根实例的在飞单元，那一支由 I-7.7（系统配置实例代号不低于根环） ① 按各盘系统配置的实例代号兜着。谓词本身照旧在用：`PublishedPredicate` 过滤 I-1.8（归并后版本全序） 的射程、判 I-4.2（无被引用未提交块）。打包记录类型 4 的实例表单元按本行的**例外**只免谓词那一半：根记录偏移 170 的实例表指针与链上每一片，指针里的出生 txg、实例代号、尾段与被指单元头里的值相等，仍挂这一条的遍历方向（已实现：`walk::Walk` 的 `judge_birth_identity_against_the_pointer`，被引用单元与实例表链上每一片都判；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）。坏镜像是 extent 树根头里的出生序号 +1、引用它的指针一个字节不动（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；崩溃点重放每个崩溃状态都跑它，走读一个被引用单元都读不到的状态报「不适用」） |
| I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与实际引用它的树一致。中央映射树不进树表，「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 65540：回退到无文件那一版之后再发第一个文件版本，十三棵树从水位起重新发号（D8（核心索引结构） 已定项 8 ②），映射树的号随之变（主 agent 2026-09-23 定，C511（回退到无文件那一版之后诞生代怎么接） 第 3 步实现员查出写死 15 的读法在合法的回退历史上误红）。自证单元（含类型 4 实例表单元）与 I-1.1（块头自述逻辑地址）同口径显式豁免。⚠️ 打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树，跨头共享时与引用树不同，那一情形的判法随 D9（加密） 那次「绑出生快照」的重开（C110（跨头共享与加密后元数据类的块头一致性读法））。这一射程覆盖码 3 全部打包记录类型，含类型 2（inode 树的叶，D8（核心索引结构） 已定项 6）：克隆头共享 origin 的叶时，叶头的出生树是 origin | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-1.4 | 块头 fsid 一致 | 任一单元，其头记录的 fsid == 系统配置 fsid（D18（块里携带什么信息）已定项 7 的字段；加密开启后与 KDF 绑定冗余，接受）。scrub 半边「本卷全部单元归属指向本卷」靠它才写得出（C74（无加密期的跨卷错向写没防线）补救的另一半） journal 记录头也带 fsid 8（系统配置 fsid 低 8 字节，D23（journal 的角色与格式） 已定项 4）：fsid 不符的记录整条不进重放前缀、按断链处理（E142（新池新建文件的干跑） 第七次跑记的 G25） | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-1.5 | 系统配置记管道身份 | 系统配置记录写出该镜像的管道身份（实现标识 + 版本），且非空。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：自举头段「写入者身份 20」= 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32，第一版写 `singlefs-rs` / 1；实现标识全零判红 | 未实现（条款已写、检查仍欠），前置：C868（系统配置管道身份没有 checker 判定） |
| I-1.6 | 类标签登记且三处相等 | 任一单元，共同前缀偏移 6 的单元类型标签 ∈ D18（块里携带什么信息） 已定项 11 的登记表（码 0 与未登记码即判损坏）；偏移 7 的 8 个 flag 位第一版恒 0，非 0 一律拒收（前向兼容政策，D15（格式冻结政策） 判定表）；数据单元（码 1）五元组首字节 == 偏移 6；打包记录单元（码 3）类身份段首字节 == 偏移 6 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-1.7 | 打包容器合法与判定顺序 | 任一打包记录单元（码 3）：记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136（含 29 字节预留位的头，D18（块里携带什么信息） 已定项 16；2026-09-14 预留位从 28 加到 29，`cap(136, 40) = ⌊32632 / 40⌋ = 815` 与按 135 算同值，40 是第一版墓碑记录宽，D18（块里携带什么信息） 已定项 24），**先于**载荷校验和判（D23（journal 的角色与格式） 已定项 1 的 B 条同一纪律）；判定顺序恒为头校验和 → 类身份段 → 记录宽 == 登记表为该打包记录类型登记的宽（类型 2 是 140，D8（核心索引结构） 已定项 6；类型 1（墓碑）是 40、类型 5（谱系）是 16，D18（块里携带什么信息） 已定项 24；不等判损坏，只需头字段）→ I-1.2（块头写序已发布） / I-1.3（块头树 ID 一致） / I-1.4（块头 fsid 一致） → 边界 → 载荷校验和 → 逐条记录，任一步失败即中止；打包记录类型 4 的两种 kind 记录另按 D18（块里携带什么信息） 已定项 11 的实例表字段表判，不套用「记录宽 == 登记表的宽」那一步（一片记录宽恒 88，条数上限由片容量给——D18（块里携带什么信息） 已定项 11 2026-09-13 从 64 改宽）（对象级的判定——I-9.7（记录三段恒零）、I-9.8（记录 inode 等于 key）——在这条中止链之外，只让那条记录 EIO）；容器头声明一种打包记录类型、一棵出生树；打包记录类型 ∈ 第二级登记表，未登记类型的容器其物理指针不得出现在任何回收 / 复用操作的输入集合 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判；类型 1、类型 5 两种记录宽 2026-10-05 删除线第 22 批补坏镜像） |
| I-1.8 | 归并后版本全序 | 码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组（字段表在 D18（块里携带什么信息） 已定项 7（码 1）与已定项 11（码 3）；归并键取这些字段但去掉载荷校验和那一段——它正是 ①「同一组的成员载荷相同」要比的那个量，进了键 ① 就恒真）之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条） | 已实现（池级 checker `walk::check_pool_image` 判 ① 同一组的成员载荷相同；② 同 key 的各组两两全序键不等**只判码 3**——码 1 的键停着等 C464（I-1.8 对码 1 的全序键写窄了） 定案，checker 里显式写成 `TotalOrderKey::UndecidedForDataUnit`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |
| I-1.9 | 清扫水位内无可读垃圾 | 清扫水位（D5（快照 / 空间记账机制） 已定项 4 第 13 项）以内不存在可读垃圾；其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住；数据单元另有两类算被压住（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K6 读法 Z 加写序，用户 2026-10-16 定）：落在同一对象出生代的 inode 记录 size 之外的（覆盖写变短与截断放掉而还在盘上的、截断中间态里还挂着的），与被写序判死的（同一 (inode, 对象出生代) 的当前版本里有偏移更小、出生 txg 更大的单元：变长留洞之后洞里放掉的旧单元），与被读法 Z 第四条判死的（记录的截断字段 T ≠ 0、单元号不小于 ⌈N_w ÷ 净荷⌉.max(1) 而单元头出生 txg 早于 T：截在单元边界再变长之后洞里放掉的旧单元，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R3）——I-1.13（扫描重建与遍历方向的索引相同） 的扫描重建不认领这两类（管理员回退之后的镜像不在此列，见 C615（扫盘重建对挂着时的回退没有输入），出处 `research/prompts/m3-tomb-r1-main-verification.md` 第二节；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4；扫描重建的产物未晋升期间清扫禁止，这条在重建之后失效） | 未实现，前置：C113（扫描重建时多版单元的现行版本判定无输入）。清扫水位是 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4 / P5 的垃圾清扫准入，第一版没有清扫、这一项不登记（D5（快照 / 空间记账机制） 已定项 4 第 13 项的注） |
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树还不判**：D19（块指针的结构与宽度预算） 已定项 10 已定映射树 key 一律 27 字节（码 2 / 码 3 末尾补零）、条目一律 55 字节，但读者只拒窄于 55 的条目，`field_table_width_of_tree_kind` 不认种类 5，宽于 55 的映射节点不判损坏，账在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
| I-1.11 | 映射 key 与单元头相符 | 任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段（码 1 是写序的事务号 6 字节；码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0）分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树（码 1 / 码 3 取类身份段偏移 43，码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定）。不等判损坏。 | 已实现（池级 checker `walk::check_pool_image`，判定在 `crates/singlefs-checker/src/walk.rs` 的 `walk_central_mapping_entries`；key 各段的偏移按 D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的字段表另写一份，不用实现的解析；判到的射程：类标签在登记表里且等于被指单元头的，出生 txg / 实例代号 / 尾段等于单元头的，码 2 / 码 3 尾段后 2 字节补零恒 0，码 1 / 码 3 的出生树等于单元头偏移 43，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判；映射根全零（无文件的一版）时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（映射里数据单元那条 key 的出生 txg 3 → 4）与 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`。编号已落到代码：`crates/singlefs-checker/src/image.rs` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 写的值就是这个编号本身，I-1.11（映射 key 与单元头相符）） |
| I-1.12 | 指针出生树等于被指单元头 | 任一被走读的指针，偏移 34 的出生树等于被指单元头里的出生树：码 1、码 3 取头偏移 43；码 2 的出生树取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定，不判（只写进状态列）。指针各段落点依据 D19（块指针的结构与宽度预算） 已定项 8（指针携带被指单元映射 key 除类标签外的各段）。inode 内部条目到叶容器那一类由 I-9.2（条目身份与子头相符） 罩住，不重判。 | 已实现（池级 checker `walk::check_pool_image` 走读里的 `judge_pointer_birth_tree_against_the_unit_header`，只在两处调用：实例表链上每一片、extent 数据单元；码 1、码 3 取头偏移 43，码 2 不判，码 2 那一支不登记成「判了一部分」，那会让每份干净镜像整条报不适用；没有走到这两处指针的镜像报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C294（指针里的 key 各段与被指单元头相等没有不变量）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行） |
| I-1.13 | 扫描重建与遍历方向的索引相同 | 只从扫描方向扫出的码 1 / 码 3 已发布单元头（加当前版本那几片码 3 容器的记录）重建的三张派生表——中央映射（每个当前版本、没被墓碑压住的码 1 单元与每片当前版本的码 3 容器，实例表的片除外（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12 豁免它）→ 扫到它的副本的落点）、每个 inode 的数据单元集合（(inode, 偏移) → 那个数据单元当前版本的映射 key）、inode 记录集合（inode 号 → 当前版本 inode 叶容器里那条记录原样）——与遍历方向从最新根读出的三张表（中央映射只取码 1 / 码 3 的条目）逐项相同（`.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」；D21（权威态与派生态的分界） 已定项 13）。已发布按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的谓词判（挂载根取最新根，实例表行取它指着的那一版：这是重建方读根的唯一一处，与 I-1.8（归并后版本全序） 那一遍同一来源）；当前版本按定案 P3 择（码 1 写序最大者，码 3 (诞生代号, 实例代号, 事务号) 最大者，与 I-1.8（归并后版本全序） 码 3 的全序键同一套）；墓碑判 key 死亡（D11（索引节点要不要留消息缓冲区） 已定项 4，第一版只有整对象墓碑）。数据单元按读法 Z 加写序判死活（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K6，用户 2026-10-16 定；D18（块里携带什么信息） 已定项 10）：有同一对象出生代的 inode 记录时三条都满足才认领——对象出生代与记录相同、偏移在 ⌈size ÷ 净荷⌉.max(1) 个单元之内、同一 (inode, 对象出生代) 的当前版本里没有偏移更小而出生 txg 更大的单元；再合取第四条（判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R3、R4）——记录偏移 120 的截断字段 T ≠ 0 时，单元号不小于 ⌈N_w ÷ 净荷⌉.max(1)（N_w 在偏移 128）且单元头出生 txg 早于 T 的不认领；没有同一对象出生代的记录的照认领（主 agent 2026-10-13 认的读法不变，判决 `research/prompts/m4-trunc-r1-main-verification.md` 第六节 S3）。写序那一条的前提是活单元的出生 txg 随偏移不降，出现「写到偏移 o 而不重写 o 之后的单元」的入口（pwrite、punch）时要重开。两侧都按 size 滤（判决 K2、K7 T-5）：遍历方向那两张表不收 size 之外的单元（inode 树里有它那个号的记录、偏移 ÷ 净荷不小于 ⌈size ÷ 净荷⌉.max(1)）与它们的映射 key，扫描重建照读法 Z 不认领，两侧各写各的。回退过、删过的镜像上两条读法（用户 2026-10-14 定，`research/prompts/m4-batch2-code-r1-forks.md` 第 1 行「甲」）：谱系记录按「不是任何现行版本的成员」认领——码 1 与 inode 叶容器只在诞生代号落在最新那一版谱系上的版本里择当前版本（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义），回退抛弃的那一段里诞生的版本不进任何一张表，墓碑只认死亡 txg 在谱系上的，墓碑树、退役树、谱系树这三棵随现行那一版走的树的容器不按谱系滤、只按已发布谓词择（C615（扫盘重建对挂着时的回退没有输入） 的读侧约束）；退役记录按退役树认领——死亡 txg 在谱系上、被退役身份是 inode 叶容器的退役记录点名的那片叶容器整个身份不认领（C768（退役记录的扫描重建读侧没有消费者））。落点：遍历方向的落点集合是扫描重建那一份的子集即对得上——搬迁把单元原样写到目的落点、映射条目换位置，源落点回收之前还留着同一版逐字节相同的副本，只从单元头扫不出哪一对是现行落点（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3；实现员 2026-10-15 推的读法，交主 agent 认）；不是子集时两边各只留读得出的那几份再比（遍历方向按位置条目里的整单元校验和读回，扫描重建按头里的载荷校验和读回）。遍历方向那一张中央映射不收最新根 deadlist 点名的那几条：被活快照钉住的单元映射条目照上一版留在中央映射树里（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S3 的 α），它们不是当前版本（实现员 2026-10-15 推的读法，交主 agent 认）。扫描方向与遍历方向共享的只有格式解析（D13（验证路线） 已定项 12） | 已实现（池级 checker `walk::check_pool_image` 末尾的 `judge_scan_rebuild_agrees_with_the_walked_index`，与别的不变量同一份判定；扫描重建本身在 `crates/singlefs-checker/src/rebuild.rs`，纯函数，`use` 清单与非测试代码里指名 crate 内一项的路径由那份文件末尾的白名单单测钉住（C410（checker 两个方向共享什么没有检查） 的会红形态）；违例写每张对不上的表差几处与第一处（表名、key、两边的值）；报不适用的：根环里没有自证过的根、最新根的实例表沿链读不全、遍历方向的索引已被别的不变量判损坏（比对之前查 I-1.11（映射 key 与单元头相符）、I-1.12（指针出生树等于被指单元头）、I-4.2（无被引用未提交块）、I-5.1（物理范围不重叠） 有没有违例，表在 `walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`，理由写明是哪一条红，主 agent 2026-10-13 定；I-9.34（谱系上的退役记录不指本版活叶） 红时同样报不适用、理由点名它——退役树与这一版 inode 树点名同一片叶容器、扫描方向看不见的那一形，用户 2026-10-14 选「乙」；那一形的镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_retirement_record_naming_the_live_leaf_the_scan_cannot_see_leaves_the_scan_rebuild_invariant_not_applicable_naming_the_no_live_leaf_invariant`；I-9.39（截断字段之外的活单元不早于它） 红时照同一先例报不适用、理由点名它——截断字段之外还有出生早于 T 的活单元、扫描重建按读法 Z 第四条不认领的那一形，实现员 2026-10-18 推的、交主 agent 认，镜像在 `crates/singlefs-harness/tests/checker_truncation_field_must_red_worlds.rs` 的 `a_live_data_unit_born_before_the_truncation_field_reddens_only_the_birth_txg_invariant`）、最新根下面的树表 / extent 树 / inode 树 / 中央映射树 / deadlist 树读不全，以及扫描重建停在条款没写之处（`rebuild::ScanRebuildNotDecided`：码 1 同一对象身份两版写序相同（C464（I-1.8 对码 1 的全序键写窄了））、码 3 两组全序键相同、同一版副本载荷校验和不同、当前版本的容器解不开、谱系记录接不成一条父链（有一条 T_old ≥ T_rb 或同一个 T_rb 两条）、谱系上的退役记录点名的 inode 叶容器有一版诞生代号晚于死亡 txg、墓碑区间不是整对象、同一 (inode, 偏移) 上两个对象、同一 inode 号在两片当前容器里）；删除中不再整张报不适用：只对那一个 (号, 对象出生代) 两侧都不比——两侧都去掉它的映射、数据单元与记录，别的对象照比（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5）；删除中由两侧一起认（判决 `research/prompts/m4-c860-r1-main-verification.md` 第三节 C2，用户 2026-10-20 选丙）：扫描一侧见到那一对 (号, 对象出生代) 的记录被谱系上的墓碑压着，遍历方向那一侧也有同号、同对象出生代的记录，且两侧那条记录 nlink 都是 0（D8（核心索引结构） 已定项 16 第 1、3 条）才算；不算的两侧照比，扫描一侧那条记录照收、它的数据单元按墓碑不认领（扫描一侧放一边的记录在 `rebuild::RebuiltIndexTables::records_of_the_tombstoned_objects`，两侧一起认在 `walk::inodes_being_deleted_on_both_sides`）；用例与必红世界（改掉映射叶一条条目的落点、清零码 1 单元头的对象 key）在 `crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs`，读法 Z 加写序与两侧按 size 滤的用例也在那份文件（覆盖写变短、先变短再变长留洞、截断中间态截到单元中间与截在单元边界上，都成立；此前覆盖写变短报不适用，C827（扫描重建对删除中与覆盖写变短的镜像报不适用） 覆盖写变短那一格由此还清），扫描重建本身三条的单测在 `crates/singlefs-checker/src/rebuild.rs`（`a_data_unit_beyond_the_size_of_its_record_is_not_claimed` 与写序两条），第四条的四条单测也在那里（题面那一形、截到 0 留单元 0、出生 txg 恰为 T、单元号恰在边界下），端到端的「截在单元边界再变长，截断字段清零时红、带字段时成立」在 `crates/singlefs-harness/tests/checker_truncation_field_must_red_worlds.rs`；谱系与退役两条读法的用例（挂着回退过、叶删空退役、左片吸收右片之后成立）、必红世界（抹掉那条谱系记录、抹掉那条退役记录）与随机历史上的计数（慢档用例）在 `crates/singlefs-harness/tests/scan_rebuild_reads_lineage_and_retirement_records.rs`；合法搬迁过的镜像上成立的用例在 `crates/singlefs-harness/tests/checker_relocated_images_judged_by_the_mapping_position.rs`，快照下覆盖写之后（映射条目留存）成立的用例在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`；GPU 判器的覆盖表收了这一条（下标 92），扫描内核的第三个入口（`crash_judge_checker_scan_rebuild.wgsl` 的 `scan_rebuild_main`）照判、置位，结局码在 GPU 结论第 24 字，对拍逐位比（用户 2026-10-18 定「补 GPU 扫描重建内核」，两路结论逐位相同；`crash_judge_gpu` 的 `KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE` 编译期守着它不进内核不置位的表，`crash_judge_conclusions` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 今天是空表）） |
| I-1.14 | 同一映射 key 在各版里的单元逐字节相同 | 回退候选集里任两版（连同由记录施加出来的那几版）的中央映射里同一把 key，各自的位置条目指着的那一份单元（按位置条目带的整单元校验和对得上的第一份）逐字节相同（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 的 N6、G7「执行层靠 N6」；D19（块指针的结构与宽度预算） 已定项 8「被搬单元逐字节不变」：搬迁原样拷、头不盖戳）。两版的位置条目相同的不读盘（同一个落点上就是同一份字节）；一组的两份都读不出的不比（读不出归 I-2.1（校验和与内容匹配））；一对都没比上时报不适用并带理由 | 已实现（2026-10-15，里程碑四己 3：池级 checker `walk::judge_unit_bytes_of_each_mapping_key_across_the_versions`；合法搬迁之后成立、搬迁时改一个字节的必红世界在 `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`；GPU 判器下标 98：进覆盖表、内核不置位、对拍不比，照下标 92 的先例，主 agent 合入时在 checker 档补登） |

> I-1（块自描述） 类的适用范围是**任一单元**（数据与元数据都在内，
> D20（承重面：单元的原子性与自包含） 口径），判定字段的落点与宽度在
> [decisions.md](../../../.claude/kb/decisions.md) D18（块里携带什么信息） 已定项 7 的字段表（码 1 / 2）与已定项 11 的类身份段表（码 3）；
> 自证单元（journal 记录 / 根槽 / 系统配置槽）对 I-1.1（块头自述逻辑地址） 与
> I-1.3（块头树 ID 一致） 显式豁免，豁免的依据是它们的自证纪律另有落点
> （I-7（根环） / I-8（journal） 两类与 D22（单元原子性怎么合成） 已定项 21）。
> ⚠️ **豁免行是穷举的，按 D18（块里携带什么信息） 已定项 11 的登记表**：码 1 / 2 / 3 在字段表里有类身份段，码 16 / 17 / 18 在豁免行里点名，
> 其余码即判红（I-1.6（类标签登记且三处相等））——这是「穷举的显式分支」作用在不变量射程上。


```

**出处 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:15-33`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **位置条目带不带设备身份** | 带：每个物理位置各带一份 dev id，不引一层 chunk 映射表 **状态：已定。** **进度：已实现。** |
| 2 | **密文校验和的位宽** | 32 位；取完整 CRC32C，不许由更宽的 CRC 截断而来 **状态：已定。** **进度：已实现。** |
| 3 | **定宽还是变长** | 定宽，不加密时留位：MAC 与 nonce 恒定占位，留成空位、不挪作他用 **状态：已定。** **进度：已实现。** |
| 4 | **位置条目各段的宽度** | 设备 ID 4 字节、物理偏移 6 字节（16 KiB 槽号）、密文校验和 4 字节 ⇒ 位置条目 14 字节 **状态：已定。** **进度：已实现。** |
| 5 | **位置的权威住哪** | 位置条目降为提示，中央映射是解引用与释放判定的唯一入口 **状态：已定。** **进度：未实现。** |
| 6 | **中央映射条目的字段表** | key 取按类复用写序键（收严后）：码 1 为 27 字节、码 2 / 码 3 为 25 字节；value 照已定项 4 / 5 **状态：已定。** **进度：已实现。** |
| 7 | **指针头部带哪几段** | 头部 50：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2、出生树 8、出生 txg 8 **状态：已定。** **进度：已实现。** |
| 8 | **引用侧携带多少出生身份，映射装哪几类** | 全部指针进映射，指向码 1 的指针 88、指向码 2 / 码 3 的 86；映射树节点、树表单元、实例表单元自举豁免 **状态：已定。** **进度：已实现。** |
| 9 | **出生序号怎么发、按什么次序发** | 树内从 0 起、码 2 与码 3 共用一个计数、换 checkpoint 清零、重写换号；次序同提交内生块的 bump 次序 **状态：已定。** **进度：已实现。** |
| 10 | **映射 key 定宽还是按类变长** | 一律 27 字节，码 2 / 码 3 末尾补零；中央映射条目一律 55 字节 **状态：已定。** **进度：已实现。** |
| 11 | **指针的偏移表、位置条目的顺序、映射树的根住哪** | 段按偏移表排，位置条目按设备身份升序；映射树根住根记录，根记录合计 457 **状态：已定。** **进度：已实现。** |
| 12 | **码 2 / 码 3 的映射条目 day-1 写不写，固定点写几次** | day-1 写出；一个实例在一个 checkpoint 里每个码 2 节点与码 3 容器只写出一次 **状态：已定。** **进度：已实现。** |
| 13 | **块指针分几段** | 分两段：逻辑属性住头部、每逻辑 extent 一份，物理属性住位置条目、每物理位置一份 **状态：已定。** **进度：已实现。** |
| 14 | **位置条目能不能用几何公式从单一起点推算** | 不能，逐个列出 **状态：已定。** **进度：已实现。** |
| 15 | **extent 距起点偏移的值域与第一版取值** | 值域 [0, 32768)、15 位够；第一版恒 0，位照留 **状态：已定。** **进度：已实现。** |

```

**出处 `.claude/kb/decisions/13-验证路线.md:8-28`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **crash refinement 先验哪一块** | 先验「一次发布 + 一次崩溃 + 一次恢复」这个封闭单元，第一个里程碑是任一 `jsn` 连续前缀不切开发布；挂载透明另立第四条性质；原候选 I-6.5（nonce 水位单调） 撤下 **状态：已定。** **进度：已实现。** |
| 2 | **事务层承不承诺串行提交** | 先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加 **状态：已定。** **进度：已实现。** |
| 3 | **自写参照实现（RefFS 形态）做到什么程度** | 做到极致取语义级：只在内存里按 POSIX 操作走，自己定义持久与崩溃后允许的状态，没有盘上格式；照 kb 另起、单独一个包、只收白名单输入 **状态：已定。** **进度：已实现。** |
| 4 | **崩溃点重放的崩溃状态集合怎么定义** | 写流按设备切段，段里每块有写的盘都被自己的屏障或 FUA 放行才关段；段内每次整写各取几态、任意组合，原地覆写多一态「新旧都读不出」；没持久放旧字节且真生成镜像；双写按设备各算 **状态：已定。** **进度：已实现。** |
| 5 | **checker 与被测实现允许共享什么** | 只共享一份从 kb 生成的常量模块，其余各写一份；判定宽度由 checker 自己探测再与系统配置比对，探不到报「声明值，未探测」 **状态：已定。** **进度：已实现。** |
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** **进度：已实现。** |
| 7 | **O2（独立解析器 + checker） 判什么** | O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员；需要第二个输入的检查归记录核对器（入参含枚举器给的持久集合），故意不给编号 **状态：已定。** **进度：已实现。** |
| 8 | **自研的两样** | Merkle 根链差分（相邻两代根递归比较，指针没变处不下钻）与代际增量 scrub（只遍历 birth generation 高于水位的块） **状态：已定。** **进度：未实现。** |
| 9 | **崩溃点重放怎么分层** | 三层都精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开；里程碑收尾与判器单元有变的那批提交前弹窗请用户跑全量档，红或不跑照样提交、记未结；先保失败可复现 **状态：已定。** **进度：已实现。** |
| 10 | **真正必须精确覆盖的 N 是什么** | 提交协议结构的等价类数，不是布局套数；落地是一个封闭的提交步骤枚举加穷尽 match **状态：已定。** **进度：已实现。** |
| 11 | **ignore 的节点什么时候开** | 否决按欠账轮转的账本；留下三条改管开的时机：确定性的覆盖记录、`T1`（常规层 挂钟预算） 是输入、判别力与覆盖分开检查 **状态：已定。** **进度：无对象。** |
| 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** **进度：已实现。** |
| 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** **进度：无对象。** |
| 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** **进度：无对象。** |
| 15 | **验证代码分两档** | harness 档（`singlefs-harness`）是单元与集成测试、随时跑；checker 档（`singlefs-checker-tier`）默认只在提交时跑快档，全量用户要求时跑 **状态：已定。** **进度：已实现。** |
| 16 | **按段登记的段怎么分片、态数用多宽** | 登记段整域装不进 u64 就按片依次判，片大小默认 2^32 态、片起点是库里的续点；态数一律用 1280 位定宽整数、算术全 checked；登记表只经环境变量读 **状态：已定。** **进度：未实现。** |
| 17 | **判定复用的单元是什么** | 剪枝的单元是函数：一份判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本；按提交或树算改动、整文件原文、文件级与包级指纹、手写版本号、路径名字进键、整道标记一律不许 **状态：已定。** **进度：未实现。** |

```
