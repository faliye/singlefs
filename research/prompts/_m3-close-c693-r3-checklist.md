### 小节清单：`.claude/kb/milestone/08-eighth-txn.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 里程碑「优化树结构和缓存」：作业点、验收与次序 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （里程碑「优化树结构和缓存」：作业点、验收与次序 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 出口（未定） | 抄 | 默认全抄（脚本生成） |
| ## 分组与次序 | 抄 | 默认全抄（脚本生成） |
| ## 一　write buffer 前端与节点缓存 | 抄 | 默认全抄（脚本生成） |
| ## 二　节点消息缓冲 | 抄 | 默认全抄（脚本生成） |
| ## 三　报警与节点大小复跑 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`.claude/kb/milestone/14-fourteenth-txn.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 里程碑「克隆与多可写头」：作业点、验收与次序 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （里程碑「克隆与多可写头」：作业点、验收与次序 标题之下、第一个下级标题之前的正文：第 2-10 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 出口（未定） | 抄 | 默认全抄（脚本生成） |
| ## 分组与次序 | 抄 | 默认全抄（脚本生成） |
| ## 一　克隆 | 抄 | 默认全抄（脚本生成） |
| ## 二　销毁可写头 | 抄 | 默认全抄（脚本生成） |
| ## 三　崩溃覆盖 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`.claude/kb/checks-owed.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 欠的检查清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （欠的检查清单 标题之下、第一个下级标题之前的正文：第 2-587 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已还清 | 抄 | 默认全抄（脚本生成） |

### 小节清单：`.claude/kb/decisions/08-核心索引结构.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 Core Index Structure —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Accounting Expressed as Idempotent Full Values, Batch Generational Replacement | 抄 | 随父节带到 |
| #### Settled Item 2: Node Size 16 KiB, Pinned as Constant | 抄 | 随父节带到 |
| #### Settled Item 3: Key Layout —— Decoupling Rename from Indexing | 抄 | 随父节带到 |
| #### Settled Item 4: Node Size —— Upper Bound Determined by Recovery Time, Value Set by Experiment | 抄 | 随父节带到 |
| #### Settled Item 5: Whiteout Deletion —— Accepted, but Batch Deletions Must Follow Intent Log | 抄 | 随父节带到 |
| #### Settled Item 6: Inode Tree Leaves Are Code 3 Packed Record Units, Record Fixed Length 140, Published (Inode Number, Object Birth Generation) Not Reused | 抄 | 随父节带到 |
| #### Settled Item 7: Write Buffer Entry `seq` —— 4 Bytes, Residing in Value | 抄 | 随父节带到 |
| #### Settled Item 8: Field Table of Tree Table Entries | 抄 | 随父节带到 |
| #### Settled Item 9: Registry of Tree Kind Codes | 抄 | 随父节带到 |
| #### Settled Item 10: First-Version Accounting Lands Directly on Leaves, `seq` Assignment Rule | 抄 | 随父节带到 |
| #### Settled Item 11: Code 2 Payload Layout and Key Comparison Order | 抄 | 随父节带到 |
| #### Settled Item 12: Coupling Between Write Buffer Front-End and Node Message Buffer —— One Level Permitted Per Tree | 抄 | 随父节带到 |
| #### Settled Item 13: Index Tree Updates Perform State Folding —— Intermediate States of Same Key Excluded from Leaves in Same Publish | 抄 | 随父节带到 |
| #### Settled Item 14: One Btree Implementation + Multiple Independent Keyspaces + Two Front-Ends, Derived Trees May Be Heterogeneous | 抄 | 随父节带到 |
| #### Settled Item 15: Three Hard Requirements for Write Buffer Entries | 抄 | 随父节带到 |
| #### Settled Item 16: First-Version Unlink: Creates deleted_inodes Only, Batched from Tail, Resumes to Empty During Mount | 抄 | 随父节带到 |
| ## Version History | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/08-核心索引结构.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 コアインデックス構造 —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：会計は冪等完全値を取り、一括世代交代とする | 抄 | 随父节带到 |
| #### 確定項目 2：ノードサイズは 16 KiB とし、定数として固定する | 抄 | 随父节带到 |
| #### 確定項目 3：キー配置 —— リネームとインデックスは分離して処理する | 抄 | 随父节带到 |
| #### 確定項目 4：ノードサイズ —— 上限はリカバリ時間によって定め、実験によって値を決める | 抄 | 随父节带到 |
| #### 確定項目 5：whiteout 削除 —— 許容するが、バッチ削除は必ず intent log を経由する | 抄 | 随父节带到 |
| #### 確定項目 6：inode ツリーのリーフはコード 3 パッキングレコードユニットとし、レコードは固定長 140、パブリッシュ済みの (inode 番号, オブジェクト誕生世代) は再利用しない | 抄 | 随父节带到 |
| #### 確定項目 7：write buffer エントリ付属の `seq` —— 4 バイト、value に格納 | 抄 | 随父节带到 |
| #### 確定項目 8：ツリーテーブルエントリのフィールド表 | 抄 | 随父节带到 |
| #### 確定項目 9：ツリー種別コードの登録表 | 抄 | 随父节带到 |
| #### 確定項目 10：第1版の会計はリーフへ直接書き込み、seq の値生成規則 | 抄 | 随父节带到 |
| #### 確定項目 11：コード 2 のペイロード配置とキー比較順序 | 抄 | 随父节带到 |
| #### 確定項目 12：write buffer フロントエンドとノードメッセージバッファの連携 —— 1 つのツリーには 1 レベルのみ許可 | 抄 | 随父节带到 |
| #### 確定項目 13：インデックスツリー更新で状態縮小を行う —— 同一パブリッシュ内で同一キーの中間状態はリーフへ書き出さない | 抄 | 随父节带到 |
| #### 確定項目 14：1 組の btree 実装 + 複数の独立 keyspace + 2 つのフロントエンド、派生ツリーは異種構造を許容 | 抄 | 随父节带到 |
| #### 確定項目 15：write buffer エントリの3つの厳格要件 | 抄 | 随父节带到 |
| #### 確定項目 16：第1版の unlink：deleted_inodes のみを立て、末尾からバッチ処理し、マウント内で空になるまで継続実行する | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/08-核心索引结构.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D8 核心索引结构 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D8 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：记账取幂等完整值，整批换代 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：节点大小 16 KiB，钉成常量 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：key 布局 —— 改名与索引分开处理 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：节点大小 —— 上限由恢复时间定，靠实验取值 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：whiteout 删除 —— 接受，但批量删除必须走 intent log | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：write buffer 条目的 `seq` —— 4 字节，住 value | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：树表条目的字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：树的种类码的登记表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：第一版记账直落叶，`seq` 的取值规则 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：码 2 的载荷布局与 key 比较序 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：write buffer 前端与节点消息缓冲区的衔接 —— 一棵树只准一级 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：索引树更新做状态折叠 —— 同一次发布里同一个 key 的中间状态不落叶 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：一套 btree 实现 + 多个独立 keyspace + 两个前端，派生树可以异构 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：write buffer 条目的三条硬要求 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：第一版 unlink：只立 deleted_inodes、从尾部分批、挂载里续做到空 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/05-快照-空间记账机制.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D5 Snapshot / Space Accounting Mechanism —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D5 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Which Dimensions Are in Dimension Tuple | 抄 | 随父节带到 |
| #### Settled Item 2: What "Generation" Takes, When Old Generations Can Be Discarded | 抄 | 随父节带到 |
| #### Settled Item 3: Carrier Form of Accounting | 抄 | 随父节带到 |
| #### Settled Item 4: Which Statistics Exist | 抄 | 随父节带到 |
| #### Settled Item 5: Widths of Accounting Key Segments | 抄 | 随父节带到 |
| #### Settled Item 6: Placement and Field Table of Sparse Side Table | 抄 | 随父节带到 |
| #### Settled Item 7: Whether "Allocated Bytes" Includes Fixed Structures Written by mkfs | 抄 | 随父节带到 |
| #### Settled Item 8: Which Rows Written to Accounting Tree on New-Pool File Creation | 抄 | 随父节带到 |
| #### Settled Item 9: Which ID Identifies Writable Head, Who Owns `previous_snapshot_txg` | 抄 | 随父节带到 |
| #### Settled Item 10: Statistics Tag Registry and Two Reserved Values | 抄 | 随父节带到 |
| #### Settled Item 11: Whether to Build Deadlist Carrier Day-1 | 抄 | 随父节带到 |
| #### Settled Item 12: Form of Deadlist Entries | 抄 | 随父节带到 |
| #### Settled Item 13: Which Mechanism to Adopt for Lifecycle Determination | 抄 | 随父节带到 |
| #### Settled Item 14: Necessary and Sufficient Condition of Reference Interval | 抄 | 随父节带到 |
| #### Settled Item 15: Whether Background Relocation Modifies `birth` | 抄 | 随父节带到 |
| #### Settled Item 16: Which Side Deadlist Merges to Upon Snapshot Destruction | 抄 | 随父节带到 |
| #### Settled Item 17: Four Premises; Crossing Any One Invalidates It | 抄 | 随父节带到 |
| #### Settled Item 18: Three Clauses to Add for Clones / Writable Snapshots | 抄 | 随父节带到 |
| #### Settled Item 19: Runtime Form of Liveness Judgment and Cost of Snapshot Destruction | 抄 | 随父节带到 |
| #### Settled Item 20: Whether Pending-Deletion Structures Count as Live or Dead | 抄 | 随父节带到 |
| #### Settled Item 21: Whether Snapshot List Needs an Enumerable Persistent Structure | 抄 | 随父节带到 |
| ## Version History | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/05-快照-空间记账机制.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D5 スナップショット / 空間会計メカニズム —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D5 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1: 次元タプルにどの次元を含めるか | 抄 | 随父节带到 |
| #### 確定項目 2: 「世代」に何を採用し、旧世代をいつ破棄できるか | 抄 | 随父节带到 |
| #### 確定項目 3: 会計のキャリア形態 | 抄 | 随父节带到 |
| #### 確定項目 4: どの統計量が存在するか | 抄 | 随父节带到 |
| #### 確定項目 5: 会計キー各セグメントの幅 | 抄 | 随父节带到 |
| #### 確定項目 6: スパースサイドテーブルの配置先とフィールド表 | 抄 | 随父节带到 |
| #### 確定項目 7: 「割り当て済みバイト」に mkfs が書き込んだ固定構造を含むか | 抄 | 随父节带到 |
| #### 確定項目 8: 新規プールでのファイル作成時に会計ツリーへ書き込む行 | 抄 | 随父节带到 |
| #### 確定項目 9: 書き込み可能ヘッドをどのIDで識別し、`previous_snapshot_txg` は誰に帰属するか | 抄 | 随父节带到 |
| #### 確定項目 10: 統計量タグ登録表と二つの予約値 | 抄 | 随父节带到 |
| #### 確定項目 11: deadlist のキャリアを day-1 に作成するか | 抄 | 随父节带到 |
| #### 確定項目 12: deadlist エントリの形態 | 抄 | 随父节带到 |
| #### 確定項目 13: ライフサイクル判定にどのメカニズムを採用するか | 抄 | 随父节带到 |
| #### 確定項目 14: 参照区間の必要十分条件 | 抄 | 随父节带到 |
| #### 確定項目 15: バックグラウンド再配置で `birth` を変更するか | 抄 | 随父节带到 |
| #### 確定項目 16: スナップショット破棄時に deadlist をどちら側へマージするか | 抄 | 随父节带到 |
| #### 確定項目 17: 4つの前提、いずれかを超過すると無効化する | 抄 | 随父节带到 |
| #### 確定項目 18: クローン / 書き込み可能スナップショットに追加すべき3条 | 抄 | 随父节带到 |
| #### 確定項目 19: 生死判定の実行時形態とスナップショット破棄のコスト | 抄 | 随父节带到 |
| #### 确定項目 20: 削除待ち構造を生存とみなすか死亡とみなすか | 抄 | 随父节带到 |
| #### 確定項目 21: スナップショット一覧に列挙可能な永続構造が必要か | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/05-快照-空间记账机制.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D5 快照 / 空间记账机制 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D5 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：维度元组里有哪几维 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：「代」取什么，旧代什么时候可以丢 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：记账的载体形态 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：统计量有哪几个 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：记账 key 各段的宽度 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：稀疏旁表的落点与字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：「已分配字节」含不含 mkfs 写下的固定结构 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：新池新建文件记账树里写哪几行 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：可写头由哪个 ID 标识，`previous_snapshot_txg` 归谁 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：统计量标签登记表与两个保留值 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：deadlist 的载体 day-1 建不建 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：deadlist 条目的形态 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：生命周期判定取哪套机制 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：引用区间的充要条件 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：后台搬迁改不改 `birth` | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：销毁快照时 deadlist 往哪一侧并 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：四条前提，越过任何一条它就失效 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 18：克隆 / 可写快照要加三条 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 19：生死判定的运行时形态与销毁快照的代价 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 20：待删除结构算活的还是死的 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 21：快照列表要不要一个可枚举的持久结构 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/decisions/23-journal的角色与格式.en.md`（kb-sections.py 全量生成，未经任何过滤）

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

### 小节清单：`.claude/kb/decisions/23-journal的角色与格式.ja.md`（kb-sections.py 全量生成，未经任何过滤）

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

### 小节清单：`.claude/kb/decisions/18-块里携带什么信息.en.md`（kb-sections.py 全量生成，未经任何过滤）

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

### 小节清单：`.claude/kb/decisions/18-块里携带什么信息.ja.md`（kb-sections.py 全量生成，未经任何过滤）

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

### 小节清单：`.claude/kb/decisions/18-块里携带什么信息.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D18 块里携带什么信息 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D18 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：谱系重写序号不做 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：全部码 2 节点无条件带 key 区间 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：数据块带明文自描述头 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：形态取元数据多副本 + 分散放置，链臂否掉，单盘布局线先不开 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：重建是降级路径，按见证者分三级 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：数据块自描述头放在低位，载荷从头之后开始 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：第一版单元头字段表，按单元类分 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：删除要写墓碑单元 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：「块」就是单元，一个单元一份头 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：墓碑取区间记录 + 打包共享单元 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：单元类型标签的值域是一张登记表，打包记录单元是独立的一类 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：码 2 的写序收窄成 4 字节，腾出的 6 字节给载荷 CRC 与预留 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：加密开启时码 1 的身份段与 `声明长度` 都进密文 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：nonce 代号就是完整的 96 位 nonce，住单元明文头 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：已撤回·此编号不再使用 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：nonce / MAC / 算法类型预留位算进头，声明长度的原点是含预留位的头末端 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：32 字节头校验和字段放 CRC32C 4 字节 + 28 字节零；自证结构的校验和覆盖整个槽含补齐 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 18：码 2 的声明长度 = 条目数 × 条目宽，补齐区恒 0，码 1 / 码 2 各补一张带偏移字段表 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 19：字段值不值得带进块里：三条判据 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 20：已定要带的身份字段 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 21：自描述的射程是错向写，不是丢失写 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 22：要支持扫盘重建的信息必须显式写下，不折进校验和种子 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 23：否决清单 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 24：第一版墓碑：墓碑树、40 字节记录、谱系记录 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/experiments/71-记账key的条目数与宽度.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E71 Entry Count and Width of Accounting Keys — Run (2026-09-01, Pure Arithmetic, 12 Unit Tests / All 8 Mutations Caught): **What Stretches to Bursting First Is the Product `Tree Count × Device Count`, and Only Under the "Dimensions Fully Utilized" Interpretation** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E71 标题之下、第一个下级标题之前的正文：第 2-20 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Criteria (Hardcoded Before Run, Unchanged After Run) | 抄 | 默认全抄（脚本生成） |
| ### Failure Clauses (Hardcoded Before Run) | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ Formula Stated in Criteria 1 and 3 Is an Upper Bound, Not Entry Count | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ One Thing Unsettled by D5 (Snapshot / Space Accounting Mechanism) Settled Item 4, E71 (Entry Count and Width of Accounting Keys) Does Not Settle on Its Behalf | 抄 | 默认全抄（脚本生成） |
| ### Measurement | 抄 | 默认全抄（脚本生成） |
| #### Entry Counts Across Three Arms (t=64 Trees, d=8 Disks, K=25) | 抄 | 随父节带到 |
| #### Criterion 3: Doubling Experiment | 抄 | 随父节带到 |
| #### Positive Control: Dropping Device Dimension | 抄 | 随父节带到 |
| #### Criterion 2: Accounting Write Bytes per Checkpoint vs 1% of `T_dirty` | 抄 | 随父节带到 |
| #### Range of K (D5 (Snapshot / Space Accounting Mechanism) Settled Item 2 + D22 (How to Synthesize Unit Atomicity) Settled Item 2) | 抄 | 随父节带到 |
| ### Run 1 Re-run (2026-10-20): Criterion 2 per Each Root Holding Only One Generation, Write-Only Without Deletion | 不抄 | 英文译文；中文原文同一节「### 第 1 次重跑（2026-10-20）：判据 2 按每条根只装一代、只写不删」照抄。英文标题里的半角冒号让 quote-kb.py 把取法切成「文件:行区间」，取不出（工具欠账，`records/2026-09-16-subagent拆分提案.md` 七十四） |
| ### Paths and Conclusion Registration | 抄 | 默认全抄（脚本生成） |
| ### What It Cannot Answer | 抄 | 默认全抄（脚本生成） |
| ### Prerequisites | 抄 | 默认全抄（脚本生成） |
| ### Measurement Basis and Re-run | 抄 | 默认全抄（脚本生成） |
| ### Impacted Decisions | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/experiments/71-记账key的条目数与宽度.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E71 記帳キーのエントリ数と幅 —— 実行済み (2026-09-01、純算術、単体テスト 12 件 / ミューテーション 8 件全捕捉)：**先にパンクするのは `ツリー数 × デバイス数` の積であり、かつ「次元をフルに使い切る」解釈でのみパンクする** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E71 标题之下、第一个下级标题之前的正文：第 2-20 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 判定基準 (実行前ハードコード、実行後未改変) | 抄 | 默认全抄（脚本生成） |
| ### 失敗条項 (実行前ハードコード) | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ 判定基準 1 と 3 が示す算式は上限であり、エントリ数ではない | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ D5 (スナップショット / 領域会計メカニズム) 確定項目 4 が未定とした点について、E71 (記帳キーのエントリ数と幅) は代行決定しない | 抄 | 默认全抄（脚本生成） |
| ### 実測 | 抄 | 默认全抄（脚本生成） |
| #### 3腕のエントリ数 (t=64 ツリー、d=8 ディスク、K=25) | 抄 | 随父节带到 |
| #### 判定基準 3：倍増試験 | 抄 | 随父节带到 |
| #### 陽性対照：デバイス次元の除去 | 抄 | 随父节带到 |
| #### 判定基準 2：1回のチェックポイントでの記帳書き込みバイト数 vs `T_dirty` の 1% | 抄 | 随父节带到 |
| #### K の値域 (D5 (スナップショット / 領域会計メカニズム) 確定項目 2 + D22 (ユニット原子性をどう合成するか) 確定項目 2) | 抄 | 随父节带到 |
| ### 第 1 回再実行 (2026-10-20)：判定基準 2 を各ルートが1世代のみ保持・書き込みのみで削除なしとして再実行 | 抄 | 默认全抄（脚本生成） |
| ### パスと結論登録 | 抄 | 默认全抄（脚本生成） |
| ### 本実験で答えられない半面 | 抄 | 默认全抄（脚本生成） |
| ### 前提条件 | 抄 | 默认全抄（脚本生成） |
| ### 計測条件と再実行 | 抄 | 默认全抄（脚本生成） |
| ### 影響を受ける決定 | 抄 | 默认全抄（脚本生成） |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/experiments/71-记账key的条目数与宽度.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E71 记账 key 的条目数与宽度 —— 已跑（2026-09-01，纯算术，12 单测 / 8 条变异全抓）：**先撑爆的是 `树数 × 设备数` 这个乘积，且只在「维度用满」那个读法下撑爆** | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E71 标题之下、第一个下级标题之前的正文：第 2-19 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 判据（跑前写死，跑完没改） | 抄 | 默认全抄（脚本生成） |
| ### 失败条款（跑前写死） | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ 判据 1 与 3 说的那个公式是上界，不是条目数 | 抄 | 默认全抄（脚本生成） |
| ### ⚠️ 一处 D5（快照 / 空间记账机制） 已定项 4 没定的东西，E71（记账 key 的条目数与宽度） 不替它定 | 抄 | 默认全抄（脚本生成） |
| ### 实测 | 抄 | 默认全抄（脚本生成） |
| #### 三条臂的条目数（t=64 树、d=8 盘、K=25） | 抄 | 随父节带到 |
| #### 判据 3：加倍试验 | 抄 | 随父节带到 |
| #### 阳性对照：去掉设备维 | 抄 | 随父节带到 |
| #### 判据 2：一次 checkpoint 的记账写字节 vs `T_dirty` 的 1% | 抄 | 随父节带到 |
| #### K 的取值范围（D5（快照 / 空间记账机制） 已定项 2 + D22（单元原子性怎么合成） 已定项 2） | 抄 | 随父节带到 |
| ### 第 1 次重跑（2026-10-20）：判据 2 按每条根只装一代、只写不删 | 抄 | 默认全抄（脚本生成） |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的那一半 | 抄 | 默认全抄（脚本生成） |
| ### 前置 | 抄 | 默认全抄（脚本生成） |
| ### 口径与复跑 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

### 小节清单：`.claude/kb/experiments/97-记账与分配记录的条目编码.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E97 Entry Encoding of Accounting and Allocation Records —— Executed (2026-09-03, pure arithmetic, 22 unit tests / 17 mutations caught, rerun three times after three rounds of adversarial testing): **It eliminated candidates and narrowed two unsettled items, but neither was settled** — segment width does not affect tree height; finite modulo generation cycles in 91 ms; placement coordinate system must be 16 KiB slot numbers; "can represent all legal starting points" and "half a unit cannot be represented" are mutually exclusive; segment manifest still lacks settled field `seq` | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E97 标题之下、第一个下级标题之前的正文：第 2-14 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Cited Clauses (Transcribed Verbatim into Source Comments) | 抄 | 默认全抄（脚本生成） |
| ### Criteria (Hardcoded Before Run, Immutable After Run) | 抄 | 默认全抄（脚本生成） |
| ### Failure Clauses (Hardcoded Before Run) | 抄 | 默认全抄（脚本生成） |
| ### Measurements (2026-09-03, pure arithmetic, N=5 rounds byte-identical, 15 unit tests / 10 mutations all caught) | 抄 | 默认全抄（脚本生成） |
| #### Accounting Tree: Width Does Not Shift Tree Height by a Single Cell | 抄 | 随父节带到 |
| #### Allocation Record Tree: Under Same Width, Changing Encoding Gains 32768x Addressing, Illegal Placements Zeroed | 抄 | 随父节带到 |
| #### Criterion 9: Manifest Missing a Segment — D8 (Core Index Structure) Write Buffer Hard Requirement 3 `seq` | 抄 | 随父节带到 |
| #### Criterion 7: Encoding by 32 KiB Unit Numbers Cannot Represent Placements for Half of Index Nodes | 抄 | 随父节带到 |
| #### Placement Granularity Dimension: Halving Granularity Doubles Entries and Capacity, Tree Height Unchanged | 抄 | 随父节带到 |
| #### Execution of Failure Clauses | 抄 | 随父节带到 |
| ### What It Cannot Answer | 抄 | 默认全抄（脚本生成） |
| ### Influenced Decisions | 抄 | 默认全抄（脚本生成） |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-09-03 | 抄 | 默认全抄（脚本生成） |
| #### (Part 1) | 抄 | 随父节带到 |
| #### (Part 2): Two Criteria and Two Dimensions Supplemented After First Adversarial Round, Rerun | 抄 | 随父节带到 |
| #### (Part 3): Two More Criteria Added, One Metric Corrected, Two Arms Added After Second Adversarial Round, Rerun | 抄 | 随父节带到 |
| #### (Part 4): Third Adversarial Round Removed Artifact — "Half Unit Unrepresentable" is Arm Tag, Not Encoding Property | 抄 | 随父节带到 |

### 小节清单：`.claude/kb/experiments/97-记账与分配记录的条目编码.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E97 記帳および割り当て記録のエントリ符号化 —— 実行済み (2026-09-03、純粋算術、22 単体テスト / 17 ミューテーション全捕捉、3 回の対抗後に 3 回再実行)：**候補は除外され、2 つの未確定項目を絞り込んだが、いずれも確定には至らず**——セグメント幅はツリー高に影響せず；有限剰余の世代は 91 ミリ秒で周回；配置先座標系は 16 KiB スロット番号が必須；「全正当開始点を表現可能」と「半ユニット表現不可」は両立せず；セグメント一覧に確定済みフィールド `seq` が不足 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E97 标题之下、第一个下级标题之前的正文：第 2-14 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 引用条項（ソースコードコメントへ逐字転記） | 抄 | 默认全抄（脚本生成） |
| ### 判定基準（実行前固定、実行後変更不可） | 抄 | 默认全抄（脚本生成） |
| ### 失敗条項（実行前固定） | 抄 | 默认全抄（脚本生成） |
| ### 実測（2026-09-03、純粋算術、N=5 ラウンドでバイト完全一致、15 単体テスト / 10 ミューテーション全捕捉） | 抄 | 默认全抄（脚本生成） |
| #### 会計ツリー：幅はツリー高を 1 マスも動かさず | 抄 | 随父节带到 |
| #### 割り当てレコードツリー：同幅で符号化を変更すると 32768 倍のアドレス空間が得られ、不正配置先はゼロ化 | 抄 | 随父节带到 |
| #### 判定基準 9：セグメント一覧に不足あり——D8 (コアインデックス構造) write buffer 硬要件 3 の `seq` | 抄 | 随父节带到 |
| #### 判定基準 7：32 KiB ユニット番号による符号化は半数のインデックスノードの配置先を表現不能 | 抄 | 随父节带到 |
| #### 配置先粒度という次元：粒度半減でエントリ数と容量占有率が倍増、ツリー高は不変 | 抄 | 随父节带到 |
| #### 失敗条項の実施状況 | 抄 | 随父节带到 |
| ### 本実験で回答できないこと | 抄 | 默认全抄（脚本生成） |
| ### 影響を受ける決定 | 抄 | 默认全抄（脚本生成） |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |
| ### 2026-09-03 | 抄 | 默认全抄（脚本生成） |
| #### （その 1） | 抄 | 随父节带到 |
| #### （その 2）：第 1 ラウンド三方論証後に判定基準 2 条、次元 2 つを追加し再実行 | 抄 | 随父节带到 |
| #### （その 3）：第 2 ラウンド三方論証後にさらに判定基準 2 条を追加、計測条件 1 箇所修正、アーム 2 つを追加し再実行 | 抄 | 随父节带到 |
| #### （その 4）：第 3 ラウンド三方論証で偽像を排除——「半ユニット表現不可」はアームのタグであり符号化の性質ではない | 抄 | 随父节带到 |

### 小节清单：`.claude/kb/experiments/97-记账与分配记录的条目编码.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E97 记账与分配记录的条目编码 —— 已跑（2026-09-03，纯算术，22 单测 / 17 条变异全抓，三轮对抗后重跑三次）：**它排除了候选、收窄了两条未定项，但两条都没定成**——段宽不动树高；有限模的代 91 毫秒就绕一圈；落点坐标系必须是 16 KiB 槽号；「表达得了全部合法起点」与「半个单元不可表示」不可兼得；段清单还缺一个已定字段 `seq` | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E97 标题之下、第一个下级标题之前的正文：第 2-14 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 被引用条款（逐字抄进源码注释） | 抄 | 默认全抄（脚本生成） |
| ### 判据（跑前写死，跑完不许改） | 抄 | 默认全抄（脚本生成） |
| ### 失败条款（跑前写死） | 抄 | 默认全抄（脚本生成） |
| ### 实测（2026-09-03，纯算术，N=5 轮逐字节一致，15 单测 / 10 条变异全被抓） | 抄 | 默认全抄（脚本生成） |
| #### 记账树：宽度对树高一格都不动 | 抄 | 随父节带到 |
| #### 分配记录树：同宽换个编码，多 32768 倍寻址，非法落点归零 | 抄 | 随父节带到 |
| #### 判据 9：段清单少一段——D8（核心索引结构） write buffer 硬要求 3 的 `seq` | 抄 | 随父节带到 |
| #### 判据 7：按 32 KiB 单元号编码表达不了半数索引节点的落点 | 抄 | 随父节带到 |
| #### 落点粒度这一维：粒度减半，条目数与占容量翻倍，树高不动 | 抄 | 随父节带到 |
| #### 失败条款的执行情况 | 抄 | 随父节带到 |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 2026-10-20：判据 10 按「每条根只装一代、只写不删」重算（第五次重跑，问题单 `c693-forks.md` Q3 的 E97 那一半，重跑登记 `e97-r5-prereg.md`） | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-09-03 | 不抄 | 历史节的子节 |
| #### （其一） | 不抄 | 历史节的子节 |
| #### （其二）：第一轮三方论证之后补两条判据、补两个维度，重跑 | 不抄 | 历史节的子节 |
| #### （其三）：第二轮三方论证之后再补两条判据、改一处口径、补两条臂，重跑 | 不抄 | 历史节的子节 |
| #### （其四）：第三轮三方论证打掉一处伪影——「半个单元不可表示」是臂的标签，不是编码的性质 | 不抄 | 历史节的子节 |

### 小节清单：`crates/singlefs-checker/invariants/09-inode树结构.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-9 inode 树结构 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-9 标题之下、第一个下级标题之前的正文：第 2-56 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
