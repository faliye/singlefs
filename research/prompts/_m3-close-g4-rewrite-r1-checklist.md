### 小节清单：`.claude/kb/decisions/22-synthesizing-unit-atomicity.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D22 (Synthesizing Unit Atomicity) — Semi-settled (One Unsettled Item) | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D22 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Journal Takes Fixed-Length Ring | 抄 | 随父节带到 |
| #### Settled Item 2: Root Ring Parameters | 抄 | 随父节带到 |
| #### Settled Item 3: Accounting Structures Adopt COW | 抄 | 随父节带到 |
| #### Settled Item 4: Rule "Blocks Freed in This Transaction Not Reused" Exists | 抄 | 随父节带到 |
| #### Settled Item 5: Scope of D2 (RAID Stripe Policy) Expanded, Completed | 抄 | 随父节带到 |
| #### Settled Item 7: Field Table of Root Records | 抄 | 随父节带到 |
| #### Settled Item 8: Placement of Fixed Structures | 抄 | 随父节带到 |
| #### Settled Item 9: Field Table of System Configuration | 抄 | 随父节带到 |
| #### Settled Item 26: Three-Tier Mutability Classification for System Configuration | 抄 | 随父节带到 |
| #### Settled Item 10: Zone Write Pointer Cannot Serve as Authoritative Recovery Ceiling | 抄 | 随父节带到 |
| #### Settled Item 11: Zone Write Pointer Does Not Enter O2 (Independent Parser + Checker) Judgment Inputs | 抄 | 随父节带到 |
| #### Settled Item 12: System Configuration Does Not Carry Tree Table Unit Pointer | 抄 | 随父节带到 |
| #### Settled Item 13: On-Disk Fixed-Length Fields Little-Endian, Registry Locations Assigned, Bitmap Bit Order Unified | 抄 | 随父节带到 |
| #### Settled Item 14: Scope of I-7.6 (Root Ring Regions on Mutually Distinct Disks) When devs < R | 抄 | 随父节带到 |
| #### Settled Item 15: Quad-Linkages of System Configuration Field Table | 抄 | 随父节带到 |
| #### Settled Item 16: System Config Slot Rotation Rules; Root Ring Formula Address Space and Intra-Region Slots | 抄 | 随父节带到 |
| #### Settled Item 17: Completion of Registry Locations for Item 13 | 抄 | 随父节带到 |
| #### Settled Item 18: Demarcation Line | 抄 | 随父节带到 |
| #### Settled Item 19: Ordinary Units with Parent Pointers: COW + D4 (Checksum Placement) Suffices Without Device Atomicity Assumptions | 抄 | 随父节带到 |
| #### Settled Item 20: Larger Tear Granularity Degrades Issue into Lost Writes Rather Than Worsening It | 抄 | 随父节带到 |
| #### Settled Item 21: In-Place Overwrite Structures Require Both Whole-Unit Checksum and Actively Verified Generation Number | 抄 | 随父节带到 |
| #### Settled Item 22: Demarcation Line Cannot Move Upward: Root Ring Rollback Candidates Become Invalid | 抄 | 随父节带到 |
| #### Settled Item 23: Journal Cannot Be Covered by COW: Holds for Fixed Rings, Belongs to Self-Authenticating Category | 抄 | 随父节带到 |
| #### Settled Item 24: Root Belongs to Same Category, and Both Layout Classes Require Identical Mechanisms | 抄 | 随父节带到 |
| #### Settled Item 25: Root Ring Depth Is Upper Bound on Block Reuse Latency, Not Identical Values | 抄 | 随父节带到 |
| ### Unsettled Items | 抄 | 默认全抄（脚本生成） |
| #### Unsettled Item 6: Zoned Write Pointers as Witnesses Outside Filesystem Checksum Jurisdiction | 抄 | 随父节带到 |
| ### Interactions with Other Decisions | 抄 | 默认全抄（脚本生成） |
| ## Version History | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/22-ユニット原子性の合成手法.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D22 (ユニット原子性の合成手法) — 半確定 (1項目未確定) | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D22 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：journal は固定長リングを採用 | 抄 | 随父节带到 |
| #### 確定項目 2：ルートリングのパラメータ | 抄 | 随父节带到 |
| #### 確定項目 3：会計構造は COW を採用 | 抄 | 随父节带到 |
| #### 確定項目 4：「本トランザクション内で解放されたブロックは再利用禁止」が存在 | 抄 | 随父节带到 |
| #### 確定項目 5：D2 (RAID ストライプ方針) の計測条件拡張、完了済み | 抄 | 随父节带到 |
| #### 確定項目 7：ルートレコードのフィールド表 | 抄 | 随父节带到 |
| #### 確定項目 8：固定構造の配置先 | 抄 | 随父节带到 |
| #### 確定項目 9：システム構成のフィールド表 | 抄 | 随父节带到 |
| #### 確定項目 26：システム構成の3段階可変性 | 抄 | 随父节带到 |
| #### 確定項目 10：ゾーン書き込みポインタをリカバリ判定の権威上限とできない | 抄 | 随父节带到 |
| #### 確定項目 11：ゾーン書き込みポインタは O2 (独立パーサー + checker) の判定入力に入らない | 抄 | 随父节带到 |
| #### 確定項目 12：システム構成は独自のツリーテーブルユニットポインタを持たない | 抄 | 随父节带到 |
| #### 確定項目 13：ディスク上の固定長フィールドは一律リトルエンディアン、列挙値は各登録表に配置、ビットマップビット順序を統一 | 抄 | 随父节带到 |
| #### 確定項目 14：devs < R のとき I-7.6 (ルートリング領域が互いに異なるディスクに配置) は何を判定するか | 抄 | 随父节带到 |
| #### 確定項目 15：システム構成フィールド表の4項目連動 | 抄 | 随父节带到 |
| #### 確定項目 16：システム構成スロットローテーション規則；ルートリング領域計算式のアドレス空間と領域内スロット位置 | 抄 | 随父节带到 |
| #### 確定項目 17：確定項目 13 の登録表一覧の補完 | 抄 | 随父节带到 |
| #### 確定項目 18：境界線 | 抄 | 随父节带到 |
| #### 確定項目 19：親ポインタを持つ通常ユニットは COW + D4 (チェックサム位置) で十分であり、いかなるデバイス原子性仮定にも依存しない | 抄 | 随父节带到 |
| #### 確定項目 20：断裂粒度の拡大は問題を悪化させるのではなく、消失書き込みへと退化させる | 抄 | 随父节带到 |
| #### 確定項目 21：インプレース上書き構造には「ユニット全体のチェックサム + 実際に検証される世代番号」が必須であり、両者不可欠 | 抄 | 随父节带到 |
| #### 確定項目 22：境界線を上位層に引き上げることは不可：ルートリングのロールバック候補が無効化される | 抄 | 随父节带到 |
| #### 確定項目 23：journal を COW でカバーすることは不可能：固定長リングで成立、自己証明クラスに属する | 抄 | 随父节带到 |
| #### 確定項目 24：ルートも同一クラスに属し、2種類のレイアウトは同一のメカニズムを必要とする | 抄 | 随父节带到 |
| #### 確定項目 25：ルートリング深度はブロック再利用遅延の上限であり、両者は同一の値ではない | 抄 | 随父节带到 |
| ### 未確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 未確定項目 6：zoned のゾーン書き込みポインタは本プロジェクトのチェックサム管轄外の目撃者である | 抄 | 随父节带到 |
| ### 他の決定との連鎖 | 抄 | 默认全抄（脚本生成） |
| ## 過去の版 | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/22-单元原子性怎么合成.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D22 单元原子性怎么合成 —— 半定（一项未定） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D22 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：journal 取定长环 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：根环的参数 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：记账结构取 COW | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：「本事务内释放的块不得重用」存在 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：D2（RAID 条带策略） 的口径要扩，已完成 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：根记录的字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：固定结构的放置 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：系统配置的字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 26：系统配置的三档可改性 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：区写指针不得作恢复判定的权威上界 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：区写指针不进 O2（独立解析器 + checker） 的判定输入 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：系统配置不带自己的树表单元指针 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：盘上定长字段一律小端，枚举取值各有登记位，位图位序统一 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：devs < R 时 I-7.6（根环区域落在互不相同的盘上） 判什么 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：系统配置字段表的四格连带 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：系统配置槽轮换规则；根环区域公式的地址空间与区域内槽位 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：已定项 13 的登记位清单补全 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 18：分界线 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 19：有父指针的普通单元，COW + D4（校验和位置） 足够，且不依赖任何设备原子性假设 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 20：撕裂粒度变大不是让问题变严重，是让它退化成丢失写 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 21：原地覆写的结构必须「整单元校验和 + 被实际检查的世代号」，两样缺一不可 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 22：分界线不能往上挪：根环的回退候选会变假 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 23：journal 不可能被 COW 覆盖：对定长环成立，落在自证那一类 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 24：根落在同一类，且两类布局需要的是同一种机制 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 25：根环深度是块重用延迟的上界，两者不是同一个数 | 抄 | 默认全抄（脚本生成） |
| ### 未定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 未定项 6：zoned 的区写指针是一个不受本工程校验和管辖的见证者 | 抄 | 默认全抄（脚本生成） |
| ### 与其他决策的连锁 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/experiments/115-completeness-and-byte-budget-of-system-configuration-field-table.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E115 Completeness and Byte Budget of System Configuration Field Table —— Run (2026-09-07, pure arithmetic, 12 unit tests / 10 mutations caught): **Of 31 items required by settled clauses, E100 (Three-Segment Geometry of System Configuration) candidate table is missing 9 items; after padding, 3 of 4 cells overflow on 512-byte slot; synthesizing indirect pointers into one directory unit fits all 4 cells** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E115 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Referenced Clauses (Copied Verbatim into Source Comments) | 抄 | 默认全抄（脚本生成） |
| ### Criteria (Hardcoded Before Run, Never Modified After) | 抄 | 默认全抄（脚本生成） |
| ### Failure Clauses (Hardcoded Before Run) | 抄 | 默认全抄（脚本生成） |
| ### Actual Run (2026-09-07, Pure Arithmetic, 12 Unit Tests / 10 Mutations Caught) | 抄 | 默认全抄（脚本生成） |
| #### Criterion 1: Completeness Gate | 抄 | 随父节带到 |
| #### Criteria 2 / 3 / 4: Does It Fit in 512-Byte Slot After Padding | 抄 | 随父节带到 |
| #### Cross-Apparatus Reconciliation | 抄 | 随父节带到 |
| #### A Measurement Basis Inconsistency Measured Along the Way | 抄 | 随父节带到 |
| #### Widths Are All Assumed Values | 抄 | 随父节带到 |
| ### What It Cannot Answer | 抄 | 默认全抄（脚本生成） |
| ### Influenced Decisions | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-09-07 | 抄 | 默认全抄（脚本生成） |
| #### (Part 1) | 抄 | 随父节带到 |
| #### (Part 2) | 抄 | 随父节带到 |

### 小节清单：`.claude/kb/experiments/45-ring-space-occupancy-for-multi-record-transactions.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E45 Ring Space Occupancy for Multi-Record Transactions — ⚠️ Run but **Conclusions Invalidated** (2026-08-30 Attack Round): Alignment Rule Stricter than I-8.2 (Record Header Does Not Cross Atomic Unit), Multiple Was Artifact of That Rule | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E45 标题之下、第一个下级标题之前的正文：第 2-11 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Verified Premises (Investigated 2026-08-30) | 抄 | 默认全抄（脚本生成） |
| ### What to Test | 抄 | 默认全抄（脚本生成） |
| ### Path and Conclusion Registry | 抄 | 默认全抄（脚本生成） |
| ### Criteria | 抄 | 默认全抄（脚本生成） |
| ### Positive Controls (Two, Neither May Be Omitted) | 抄 | 默认全抄（脚本生成） |
| ### Failure Clauses | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ Conclusions Invalidated: Entire Multiple Produced by an Undefined Alignment Rule | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ The Claim of "Three Paths Sharing No Code" Also Does Not Hold | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ Four Other Points Identified in Attack Round | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ A Physical Fact Untouched by E45 (Ring Space Occupancy for Multi-Record Transactions) Yet Decisive in Direction | 抄 | 默认全抄（脚本生成） |
| ### Original Empirical Measurements (Calibration Invalidated, Retained for Archive) | 抄 | 默认全抄（脚本生成） |
| ### Original Measurements (2026-08-30, N=5 Rounds Byte-for-Byte Identical; All 7 Mutations Caught, 0 Blind Spots) | 抄 | 默认全抄（脚本生成） |
| #### Claim Stood: Spanning Multiple Records Inflates Ring, Worst-Case 12x | 抄 | 随父节带到 |
| #### Size Scan: Conclusion is "Never Worse", Not "Always 12x" | 抄 | 随父节带到 |
| #### ⚠️ Reverse Cost **Sought But Not Found** | 抄 | 随父节带到 |
| #### Positive Controls | 抄 | 随父节带到 |
| #### ⚠️ A Blind Spot: Alignment Conceals Calibration Discrepancies | 抄 | 随父节带到 |
| ### Prerequisites | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ Round 2 Attack Returned Only One Leg, Leaving Two Unmodeled Costs | 抄 | 默认全抄（脚本生成） |
| ### Affected Decisions | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-08-30 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`.claude/kb/experiments/45-トランザクションの複数レコード跨ぎによるリング占有.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E45 トランザクションの複数レコード跨ぎによるリング占有 —— ⚠️ 実行済みだが**結論無効**（2026-08-30 攻撃ラウンド）：アライメント規則が I-8.2 (レコードヘッダはアトミックユニットを跨がない) より厳格であり、倍率はその規則の産物 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E45 标题之下、第一个下级标题之前的正文：第 2-11 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確認済みの前提（2026-08-30 時点で現物確認） | 抄 | 默认全抄（脚本生成） |
| ### 何を検証するか | 抄 | 默认全抄（脚本生成） |
| ### 経路と結論の登録 | 抄 | 默认全抄（脚本生成） |
| ### 判定基準 | 抄 | 默认全抄（脚本生成） |
| ### 陽性対照（2 つ、双方必須） | 抄 | 默认全抄（脚本生成） |
| ### 失敗条項 | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ 結論無効：倍率全体が未定義のアライメント規則によって作られた産物であった | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ 「コードを共有しない 3 つの独立経路」という主張も成立しない | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ その他 4 点、攻撃ラウンドで逐条指摘 | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ E45 (複数レコードにまたがるトランザクションのリング占有) が触れられないが、方向を決定づける物理的事実 | 抄 | 默认全抄（脚本生成） |
| ### 当初の実測（口径は無効化、アーカイブ保存） | 抄 | 默认全抄（脚本生成） |
| ### 当初実測（2026-08-30、N=5 ラウンド逐次バイト完全一致；ミューテーション 7 件すべて検出、盲点 0） | 抄 | 默认全抄（脚本生成） |
| #### 主張成立：複数跨ぎがリングを過大化、最悪 12 倍 | 抄 | 随父节带到 |
| #### 規模走査：結論は「決して悪化しない」であり、「常に 12 倍」ではない | 抄 | 随父节带到 |
| #### ⚠️ 逆方向のコストを**探索したが、見当たらず** | 抄 | 随父节带到 |
| #### 陽性対照 | 抄 | 随父节带到 |
| #### ⚠️ 1 つの盲点：アライメントが口径差を隠蔽する | 抄 | 随父节带到 |
| ### 前提 | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ 第 2 ラウンド攻撃は 1 レグのみ返還、しかしそのレグが未モデリングのコストを 2 点残した | 抄 | 默认全抄（脚本生成） |
| ### 影響を受ける決定 | 抄 | 默认全抄（脚本生成） |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-08-30 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`crates/singlefs-checker/invariants/03-journal.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-8 journal —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-8 标题之下、第一个下级标题之前的正文：第 2-43 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`.claude/kb/decisions/23-journal-role-format.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D23 (Journal Role & Format) — Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D23 标题之下、第一个下级标题之前的正文：第 2-15 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Role of Journal Itself | 抄 | 随父节带到 |
| #### Settled Item 2: Whether Journal Is Fixed-Length Ring or Chained | 抄 | 随父节带到 |
| #### Settled Item 3: Tail Resides in System Configuration Slot, Modifying JBD2 in Two Places | 抄 | 随父节带到 |
| #### Settled Item 4: Record Header Bounded Within Single Atomic Unit, Header 487 Bytes | 抄 | 随父节带到 |
| #### Settled Item 5: O2 (Independent Parser + Checker) Is Unary Predicate, Byte-by-Byte Comparison Belongs to Record Checker | 抄 | 随父节带到 |
| #### Settled Item 6: Target Workload Priority Moved Out of Journal Track | 抄 | 随父节带到 |
| #### Settled Item 7: Transaction Can Span Multiple Records, Record Header Carries Transaction Boundary Fields | 抄 | 随父节带到 |
| #### Settled Item 8: Record Header Carries 32-Bit Reverse Chain | 抄 | 随父节带到 |
| #### Settled Item 9: jsn = Instance Number 32 Bits + Counter 48 Bits | 抄 | 随父节带到 |
| #### Settled Item 10: previous_hash Covers Entire Record Header of Previous Record | 抄 | 随父节带到 |
| #### Settled Item 11: Reverse Chain Hash Takes Full 32-Bit CRC32C | 抄 | 随父节带到 |
| #### Settled Item 12: Record Size Fixed at 4 KiB | 抄 | 随父节带到 |
| #### Settled Item 13: Payload Integrity, header_csum Covers Entire Record | 抄 | 随父节带到 |
| #### Settled Item 14: Replay Lower Bound Given by Selected Root; Administrator Rollback, Failure Handling, and Instance Switchover | 抄 | 随父节带到 |
| #### Settled Item 15: Crash After Record Persisted but Before Root Slot Persisted: Recovery Reconstructs Root of That Publish from Record | 抄 | 随父节带到 |
| #### Settled Item 16: mkfs Writes Instance Number 0, First Writable Mount Takes 1 | 抄 | 随父节带到 |
| #### Settled Item 17: Field Table for 56-Byte Named Entry | 抄 | 随父节带到 |
| #### Settled Item 18: In-Flight Record Count Upper Bound Value and Semantics, Tail Field Encoding | 抄 | 随父节带到 |
| #### Settled Item 19: Transaction Number and Commit Marker for Empty Publish Records, previous_hash, Ring Length | 抄 | 随父节带到 |
| #### Settled Item 20: Records Carry Pointer Layer Target State, Allocation Precedes Journal | 抄 | 随父节带到 |
| #### Settled Item 21: Generation Number: Record-Granularity Global Monotonic Number, Not D5 (Snapshot / Space Accounting Mechanism) txg | 抄 | 随父节带到 |
| #### Settled Item 22: Prefix Judgment: Strictly Continuous, Discontinuity Halts Immediately | 抄 | 随父节带到 |
| #### Settled Item 23: How Each of the Three Deadlocks Is Broken | 抄 | 随父节带到 |
| #### Settled Item 24: Deadlock 2 and Spurious ENOSPC Are Distinct Faults Requiring Separate Solutions | 抄 | 随父节带到 |
| #### Settled Item 25: D16 (Publish Semantics) "Not Bypassing Journal" Takes Reading Option A | 抄 | 随父节带到 |
| #### Settled Item 26: Two Parsing Disciplines for Record Header | 抄 | 随父节带到 |
| ## History | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/23-ジャーナルの役割と形式.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D23 (ジャーナルの役割と形式) — 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D23 标题之下、第一个下级标题之前的正文：第 2-15 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：ジャーナルの役割自体 | 抄 | 随父节带到 |
| #### 確定項目 2：ジャーナルは固定長リングかチェーン式か | 抄 | 随父节带到 |
| #### 確定項目 3：tail はシステム構成スロットに常駐、jbd2 を2箇所改修 | 抄 | 随父节带到 |
| #### 確定項目 4：レコードヘッダを単一アトミックユニット内に制約、ヘッダ487バイト | 抄 | 随父节带到 |
| #### 確定項目 5：O2 (独立パーサー + checker) は単項述語、バイト単位照合はレコード照合器の所掌 | 抄 | 随父节带到 |
| #### 確定項目 6：対象ワークロード優先順位をジャーナルラインから除外 | 抄 | 随父节带到 |
| #### 確定項目 7：1トランザクションは複数レコードに跨走可能、レコードヘッダにトランザクション境界フィールドを付与 | 抄 | 随父节带到 |
| #### 確定項目 8：レコードヘッダに32ビット逆方向チェーンを付与 | 抄 | 随父节带到 |
| #### 確定項目 9：jsn = インスタンス番号32ビット + カウンタ48ビット | 抄 | 随父节带到 |
| #### 確定項目 10：previous_hash は直前レコードのヘッダ全体を網羅 | 抄 | 随父节带到 |
| #### 確定項目 11：逆方向チェーンのハッシュは完全な32ビットCRC32Cを採用 | 抄 | 随父节带到 |
| #### 確定項目 12：レコードサイズは4 KiBを採用 | 抄 | 随父节带到 |
| #### 確定項目 13：ペイロード完全性、header_csum がレコード全体をカバー | 抄 | 随父节带到 |
| #### 確定項目 14：再生の下限は選択ルートにより提示；管理者ロールバック、障害処理およびインスタンス切り替え | 抄 | 随父节带到 |
| #### 確定項目 15：レコード永続化後・ルートスロット永続化前クラッシュ時、リカバリはレコードから当該パブリッシュのルートを再構築 | 抄 | 随父节带到 |
| #### 確定項目 16：mkfs はインスタンス番号0を書き込み、初回書き込み可能マウントは1を取得 | 抄 | 随父节带到 |
| #### 確定項目 17：指名項目56バイトのフィールド表 | 抄 | 随父节带到 |
| #### 確定項目 18：インフライトレコード数上限の値と意味論、tail フィールドのエンコーディング | 抄 | 随父节带到 |
| #### 確定項目 19：空パブリッシュレコードのトランザクション番号とコミットマーカー、previous_hash、リング長 | 抄 | 随父节带到 |
| #### 確定項目 20：レコードはポインタ層の目標状態を担持、割り当てはジャーナルに先行 | 抄 | 随父节带到 |
| #### 確定項目 21：世代番号：レコード粒度のグローバル単調番号であり、D5 (スナップショット / 空間会計メカニズム) の txg ではない | 抄 | 随父节带到 |
| #### 確定項目 22：プレフィックス判定：厳格連続、番号途絶で即時停止 | 抄 | 随父节带到 |
| #### 確定項目 23：3つのデッドロックの個別打破法 | 抄 | 随父节带到 |
| #### 確定項目 24：デッドロック2と疑似ENOSPCは別個の障害であり個別対処を要する | 抄 | 随父节带到 |
| #### 確定項目 25：D16 (パブリッシュセマンティクス)「ジャーナルを迂回しない」は解釈案Aを採用 | 抄 | 随父节带到 |
| #### 確定項目 26：レコードヘッダの2つのパース規律 | 抄 | 随父节带到 |
| ## 履歴 | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/23-journal的角色与格式.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D23 journal 的角色与格式 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D23 标题之下、第一个下级标题之前的正文：第 2-15 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：journal 的角色本身 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：journal 是定长环还是链式 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：tail 住系统配置槽，改 jbd2 两处 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：记录头约束在单个原子单元内，头 487 字节 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：O2（独立解析器 + checker） 是一元谓词，逐字节比对归记录核对器 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：目标负载优先级移出 journal 这条线 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：一个事务可以跨多条记录，记录头带事务边界字段 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：记录头带一条 32 位反向链 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：jsn = 实例代号 32 位 + 计数器 48 位 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：`previous_hash` 覆盖前一条记录的整个记录头 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：反向链的 hash 取完整 32 位 CRC32C | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：记录尺寸取 4 KiB | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：载荷完整性，`header_csum` 罩整条记录 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：mkfs 写实例代号 0，第一次可写挂载取 1 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：点名项 56 字节的字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 18：在飞记录数上限的取值与语义，tail 字段的编码 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 19：空发布记录的事务号与提交标记、`previous_hash`、环长 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 20：记录承载指针层的目标态，分配先于 journal | 抄 | 默认全抄（脚本生成） |
| #### 已定项 21：世代号：记录粒度的全局单调号，不是 D5（快照 / 空间记账机制） 的 txg | 抄 | 默认全抄（脚本生成） |
| #### 已定项 22：前缀判定：严格连续、断号即止 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 23：三个死锁各自怎么破 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 24：死锁 2 与假性 ENOSPC 是两种故障，要分治 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 25：D16（发布语义）「不绕过 journal」取读法甲 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 26：记录头的两条解析纪律 | 抄 | 默认全抄（脚本生成） |
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

### 小节清单：`.claude/kb/checks-owed.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 欠的检查清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （欠的检查清单 标题之下、第一个下级标题之前的正文：第 2-590 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已还清 | 抄 | 默认全抄（脚本生成） |
