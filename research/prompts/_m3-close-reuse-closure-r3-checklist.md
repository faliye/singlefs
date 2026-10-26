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
