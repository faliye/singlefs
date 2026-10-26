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

### 小节清单：`crates/singlefs-checker/invariants/02-根环.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-7 根环 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-7 标题之下、第一个下级标题之前的正文：第 2-38 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`.claude/kb/experiments/50-upper-and-lower-bounds-on-root-ring-slot-count.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E50 Upper and Lower Bounds on Root Ring Slot Count — Run (2026-08-30, N=5 Rounds Byte-Identical, 8 Unit Tests / 6 Mutation Rules Caught): Lower Bound S=1 Is Sufficient, Upper Bound Far Beyond 256 Slots; **One Lap Is Millisecond-Scale** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E50 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### What to Measure | 抄 | 默认全抄（脚本生成） |
| ### Criteria (Hardcoded Before Run) | 抄 | 默认全抄（脚本生成） |
| ### What Constitutes Failure (Hardcoded Before Run, Forbidden to Change After Run) | 抄 | 默认全抄（脚本生成） |
| ### The Half It Cannot Answer | 抄 | 默认全抄（脚本生成） |
| ### Prerequisites | 抄 | 默认全抄（脚本生成） |
| ### Measurement Basis and Reproduction | 抄 | 默认全抄（脚本生成） |
| ### Actual Measurement (2026-08-30, N=5 Rounds Byte-Identical) | 抄 | 默认全抄（脚本生成） |
| #### Quantity 1: One Lap Duration Is **Millisecond-Scale**, Not Second-Scale | 抄 | 随父节带到 |
| #### Quantity 2: Upper Bound Is Far Beyond Any Reasonable Slot Count | 抄 | 随父节带到 |
| #### Quantity 3: Lower Bound — **S = 1 Is Already Satisfied As Long As R ≥ 2** | 抄 | 随父节带到 |
| #### Controls | 抄 | 随父节带到 |
| ### Conclusions | 抄 | 默认全抄（脚本生成） |
| ### Measurement Basis and Known Limitations | 抄 | 默认全抄（脚本生成） |
| ### Impacted Decisions | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-08-30 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`.claude/kb/experiments/50-ルートリングスロット数の上下限.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E50 ルートリングスロット数の上下限 —— 実行済み (2026-08-30、N=5 ラウンドバイト一致、単体テスト 8 件 / 変異 6 件全捕捉)：下限は S=1 で十分、上限は 256 スロットの遥か彼方；**1 周はミリ秒オーダー** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E50 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 何を測定するか | 抄 | 默认全抄（脚本生成） |
| ### 判定基準 (実行前ハードコード) | 抄 | 默认全抄（脚本生成） |
| ### どのような場合を失敗とするか (実行前ハードコード、実行後変更不可) | 抄 | 默认全抄（脚本生成） |
| ### 本実験で答えられない半分 | 抄 | 默认全抄（脚本生成） |
| ### 前提条件 | 抄 | 默认全抄（脚本生成） |
| ### 計測条件と再実行 | 抄 | 默认全抄（脚本生成） |
| ### 実測 (2026-08-30、N=5 ラウンドバイト一致) | 抄 | 默认全抄（脚本生成） |
| #### 測定量 1：1 周の所要時間は**ミリ秒オーダー**であり、秒オーダーではない | 抄 | 随父节带到 |
| #### 測定量 2：上限はいかなる合理的なスロット数の遥か彼方にある | 抄 | 随父节带到 |
| #### 測定量 3：下限 —— **R ≥ 2 であれば S = 1 で既に満たされる** | 抄 | 随父节带到 |
| #### 対照 | 抄 | 随父节带到 |
| ### 結論 | 抄 | 默认全抄（脚本生成） |
| ### 計測条件と既知の限界 | 抄 | 默认全抄（脚本生成） |
| ### 影響を受ける決定 | 抄 | 默认全抄（脚本生成） |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-08-30 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`.claude/kb/experiments/50-根环槽数的上下界.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E50 根环槽数的上下界 —— 已跑（2026-08-30，N=5 轮字节一致，8 单测 / 6 条变异全抓）：下界 S=1 就够，上界远在 256 槽之外；**一圈是毫秒级** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E50 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 要测什么 | 抄 | 默认全抄（脚本生成） |
| ### 判据（跑前写死） | 抄 | 默认全抄（脚本生成） |
| ### 怎样才算失败（跑前写死，跑完不许改） | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的那一半 | 抄 | 默认全抄（脚本生成） |
| ### 前置 | 抄 | 默认全抄（脚本生成） |
| ### 口径与复跑 | 抄 | 默认全抄（脚本生成） |
| ### 实测（2026-08-30，N=5 轮字节一致） | 抄 | 默认全抄（脚本生成） |
| #### 量一：一圈时长是**毫秒级**，不是秒级 | 抄 | 随父节带到 |
| #### 量二：上界远在任何合理槽数之外 | 抄 | 随父节带到 |
| #### 量三：下界 —— **S = 1 就已经满足，只要 R ≥ 2** | 抄 | 随父节带到 |
| #### 对照 | 抄 | 随父节带到 |
| ### 结论 | 抄 | 默认全抄（脚本生成） |
| ### 口径与已知局限 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-08-30 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/checks-owed.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 欠的检查清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （欠的检查清单 标题之下、第一个下级标题之前的正文：第 2-573 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已还清 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`.claude/kb/decisions/15-format-freeze-policy.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D15 (Format Freeze Policy) — Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D15 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Target of Item 7 is Unsettled Items, Not a Decision List | 抄 | 随父节带到 |
| #### Settled Item 2: Tree Table Made an Open List from Day-1 | 抄 | 随父节带到 |
| #### Settled Item 3: `spec_hash` Invariably Computed from Complete Projection of kb Field Tables, Never Switched to Rust Structs | 抄 | 随父节带到 |
| #### Settled Item 4: Bit Allocation Registry for Feature Bits | 抄 | 随父节带到 |
| #### Settled Item 5: Freezing is a Commit Arbitrated by Gates, Not a Verbal Statement | 抄 | 随父节带到 |
| #### Settled Item 6: Checklist That Must Be Completed Before Freeze | 抄 | 随父节带到 |
| #### Settled Item 7: Freezing Split into Four Layers, Plus Three Independent Freeze Components Outside Layers | 抄 | 随父节带到 |
| #### Settled Item 8: What Freedoms Are Lost After Freezing | 抄 | 随父节带到 |
| #### Settled Item 9: Post-Freeze Change Judgment Table | 抄 | 随父节带到 |
| #### Settled Item 10: Bit Count, Accounting, and Allocation Rights of Feature Bits | 抄 | 随父节带到 |
| #### Settled Item 11: Two Gates That Can Go Red Paired with Freezing | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/15-フォーマット凍結ポリシー.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D15 (フォーマット凍結ポリシー) — 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D15 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：第 7 項の判定対象は未確定項目であり決定リストではない | 抄 | 随父节带到 |
| #### 確定項目 2：ツリーテーブルを day-1 から開放リストとする | 抄 | 随父节带到 |
| #### 確定項目 3：`spec_hash` は kb フィールド表の完全射影から常に算出し、Rust 構造体へ切り替えない | 抄 | 随父节带到 |
| #### 確定項目 4：feature bit のビット割り当て登録表 | 抄 | 随父节带到 |
| #### 確定項目 5：凍結はゲート仲裁を受けるコミットであり言葉の表明ではない | 抄 | 随父节带到 |
| #### 確定項目 6：凍結前に完了必須のチェックリスト | 抄 | 随父节带到 |
| #### 確定項目 7：凍結は 4 層に分割し、4 層外に独立した 3 つの凍結コンポーネントを設ける | 抄 | 随父节带到 |
| #### 確定項目 8：凍結後に失われる自由 | 抄 | 随父节带到 |
| #### 確定項目 9：凍結後の変更判定表 | 抄 | 随父节带到 |
| #### 確定項目 10：feature bit のビット数・会計・割り当て権 | 抄 | 随父节带到 |
| #### 確定項目 11：凍結に伴い整備すべき赤判定検査 | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/15-格式冻结政策.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D15 格式冻结政策 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D15 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：第 7 项的对象是未定分项，不是决策名单 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：树表 day-1 做成开放列表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：`spec_hash` 恒对 kb 字段表的完整投影求，不切到 Rust 结构体 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：feature bit 的位分配登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：冻结是一次受门禁仲裁的提交，不是一句话 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：冻结前必须完成的清单 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：冻结分四层，四层之外另有三个独立冻结组件 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：冻结之后失去哪些自由 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：冻结后的改动判定表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：feature bit 的位数、记账与分配权 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：冻结要配的两条会红的检查 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/16-publish-semantics.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D16 (Publish Semantics) — Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D16 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Defer Window Takes Tightened Form of Per-Disk Rollback Floor, Counted by State Count | 抄 | 随父节带到 |
| #### Settled Item 2: Checkpoint Trigger Predicate Takes Earlier of Time and Dirty Data Volume | 抄 | 随父节带到 |
| #### Settled Item 3: Journal Record Format is Third Independent Freeze Component Outside Four Layers | 抄 | 随父节带到 |
| #### Settled Item 4: Publish Applied Atomically or Not at All, Second Replay Implementation Stops at Syntactic Layer | 抄 | 随父节带到 |
| #### Settled Item 5: Values for `T_time` and `T_dirty`, Loss Window Measurement Basis | 抄 | 随父节带到 |
| #### Settled Item 6: Publication Counter, Four Equal Signs Written as One | 抄 | 随父节带到 |
| #### Settled Item 7: Persistence Ordering of Publish | 抄 | 随父节带到 |
| #### Settled Item 8: Warm-Up Takes Option E, New Instance Root First Covers Two Disks | 抄 | 随父节带到 |
| #### Settled Item 9: Empty Publishes Also Write Units | 抄 | 随父节带到 |
| #### Settled Item 10: Publish Granularity Batches Writes, Not Once Per Transaction | 抄 | 随父节带到 |
| #### Settled Item 11: No Operation Bypasses Journal to Publish Roots Directly | 抄 | 随父节带到 |
| #### Settled Item 12: Intermediate State Legitimacy Semantics Rewritten to "Root + Arbitrary Journal Prefix" | 抄 | 随父节带到 |
| #### Settled Item 13: Journal Replay Owned by Record Checker, Not O2 (Independent Parser + Checker) | 抄 | 随父节带到 |
| #### Settled Item 14: Snapshot Publish Points Aligned with Checkpoint Boundary, Pinned After All Data Mutations | 抄 | 随父节带到 |
| #### Settled Item 15: Releases in Checkpoint C Forbidden from Entering Allocatable Set Before C Publishes | 抄 | 随父节带到 |
| #### Settled Item 16: Checkpoint and Message Buffers Buy Identical Property, E7 (Offline Index Harness) Control Group Pinned | 抄 | 随父节带到 |
| #### Settled Item 17: What Callers May Assume After fsync Errors | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/16-パブリッシュセマンティクス.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D16 (パブリッシュセマンティクス) — 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D16 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：defer ウィンドウはディスク単位のロールバック下限の厳格化形態を採用し、状態数として読み取る | 抄 | 随父节带到 |
| #### 確定項目 2：checkpoint のトリガー判定条件は時間とダーティデータ量のいずれか早い方とする | 抄 | 随父节带到 |
| #### 確定項目 3：journal 記録形式は 4 層外の第 3 の独立凍結コンポーネントである | 抄 | 随父节带到 |
| #### 確定項目 4：1 回のパブリッシュを一括適用または一括非適用とし、第 2 の replay 実装は統語論層にとどめる | 抄 | 随父节带到 |
| #### 確定項目 5：`T_time` と `T_dirty` の設定値、消失ウィンドウの計測条件 | 抄 | 随父节带到 |
| #### 確定項目 6：パブリッシュカウンタ、4 つの等号を 1 つに統合 | 抄 | 随父节带到 |
| #### 確定項目 7：パブリッシュの永続順序 | 抄 | 随父节带到 |
| #### 確定項目 8：ウォームアップは戊を採用、新インスタンスのルートを 2 台のディスクに先行配置 | 抄 | 随父节带到 |
| #### 確定項目 9：空パブリッシュもユニットを書き出す | 抄 | 随父节带到 |
| #### 確定項目 10：パブリッシュ粒度はバッチ化を採用し、トランザクションごとの発行は行わない | 抄 | 随父节带到 |
| #### 確定項目 11：いかなる操作も journal を迂回して直接ルートを発行することはできない | 抄 | 随父节带到 |
| #### 確定項目 12：中間状態の正当性セマンティクスを「ルート + journal の任意プレフィックス」に改定 | 抄 | 随父节带到 |
| #### 確定項目 13：本 journal replay はレコード照合器に帰属し、O2 (独立パーサ + checker) には帰属しない | 抄 | 随父节带到 |
| #### 確定項目 14：スナップショット発行点は checkpoint 境界に整列し全データ変更の直後に固定 | 抄 | 随父节带到 |
| #### 確定項目 15：checkpoint C で発生した解放は、C が発行されるまで割り当て可能集合に復帰しない | 抄 | 随父节带到 |
| #### 確定項目 16：checkpoint とメッセージバッファは同一の効用を購入しており、E7 (オフラインインデックス harness) の対照群を固定 | 抄 | 随父节带到 |
| #### 確定項目 17：fsync エラー発生後、呼び出し側は何を前提とできるか | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/16-发布语义.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D16 发布语义 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D16 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：checkpoint 的触发判据取时间与脏数据量，两者取先到 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：journal 记录格式是四层之外的第三个独立冻结组件 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：`T_time` 与 `T_dirty` 取多少，丢失窗口的口径 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：发布计数器，四个等号写成一条 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：发布的持久顺序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：暖机取戊，新实例的根先覆盖两块盘 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：一次空发布也写单元 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：发布粒度取攒批，不是每个事务发一次 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：没有任何操作可以绕过 journal 直接发根 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：中间状态的合法性语义改写成「根 + journal 任意前缀」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：这份 journal replay 归记录核对器，不归 O2（独立解析器 + checker） | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：快照发布点与 checkpoint 边界对齐，钉在全部数据变更之后 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：checkpoint C 中产生的释放，C 发布之前不进可分配集合 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：checkpoint 与消息缓冲买同一样东西，E7（离线索引 harness） 的对照组钉死 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：fsync 报错之后，调用方能假设什么 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/28-mount-time-commitments.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D28 Mount-Time Commitments —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D28 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Authoritative form of admission inequality | 抄 | 随父节带到 |
| #### Settled Item 2: In-flight synthesis of admission control | 抄 | 随父节带到 |
| #### Settled Item 3: Space reservation for instance switching | 抄 | 随父节带到 |
| #### Settled Item 4: Checkpoint reserve pool inclusion in admission | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/28-マウント時コミットメント量.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D28 マウント時コミットメント量 —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D28 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1: 受入不等式の権威形式 | 抄 | 随父节带到 |
| #### 確定項目 2: 受入のインフライト合成 | 抄 | 随父节带到 |
| #### 確定項目 3: インスタンス切り替え時の空間予約 | 抄 | 随父节带到 |
| #### 確定項目 4: checkpoint 保持プールの受入算入 | 抄 | 随父节带到 |
| ## 履歴 | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/28-挂载期承诺量.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D28 挂载期承诺量 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D28 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：准入不等式的权威形式 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：准入的在飞合成 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：实例切换的预留 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：checkpoint 保留池进准入 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
