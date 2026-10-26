### 小节清单：`crates/singlefs-checker/invariants/02-根环.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-7 根环 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-7 标题之下、第一个下级标题之前的正文：第 2-38 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

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

### 小节清单：`.claude/kb/checks-owed.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 欠的检查清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （欠的检查清单 标题之下、第一个下级标题之前的正文：第 2-583 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已还清 | 抄 | 默认全抄（脚本生成） |

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
| （I-3 标题之下、第一个下级标题之前的正文：第 2-41 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
