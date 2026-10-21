### 小节清单：`.claude/kb/experiments/186-不认识的树-记账维重推.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E186 不认识的树·记账维重推 —— 已跑（第一次跑 Q1、Q2 够判，变异表 17 条全抓；第二次跑 Q1 够判，变异表 26 条全抓） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E186 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 现状 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 读数：Q1 的记账胜者（判的读法 B1、W2，Q2 读「计」；Q2 两种读法下 Q1 逐点相同，`research/results/e186-unknown-tree-accounting-rederive.out.gz` 第 186048 行） | 抄 | 默认全抄（脚本生成） |
| ### 读数：Q2（`RejectEntry` 对 `Skip`，flags 胜者两种读法） | 抄 | 默认全抄（脚本生成） |
| ### 几何敏感性（登记第八节） | 抄 | 默认全抄（脚本生成） |
| ### 产物齐不齐、判决行与没过的字段 | 抄 | 默认全抄（脚本生成） |
| ### 作废、失败、停机条款逐条 | 抄 | 默认全抄（脚本生成） |
| ### 单测与变异 | 抄 | 默认全抄（脚本生成） |
| ### 岔路表（问题单 `research/prompts/c693-forks.md`） | 抄 | 默认全抄（脚本生成） |
| ### 口径 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 第二次跑（2026-10-20，改判据的重跑） | 抄 | 默认全抄（脚本生成） |
| #### 读数：Q1 反向接受条款的判（判的读法 B1、W2，Q2 读「计」；被判点集是合法点里 `h_new > 0`） | 抄 | 随父节带到 |
| #### 对照、自证与停机条款（第二次跑） | 抄 | 随父节带到 |
| #### 单测与变异（第二次跑） | 抄 | 随父节带到 |
| #### 岔路表（问题单 `research/prompts/c693-forks.md` 的 Q1） | 抄 | 随父节带到 |
| #### 它答不了的（第二次跑补） | 抄 | 随父节带到 |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

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

### 小节清单：`crates/singlefs-checker/invariants/05-空间记账.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-3 空间记账 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-3 标题之下、第一个下级标题之前的正文：第 2-39 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

### 小节清单：`crates/singlefs-checker/invariants/09-inode树结构.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## I-9 inode 树结构 —— 类 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （I-9 标题之下、第一个下级标题之前的正文：第 2-56 行） | 抄 | 开篇正文，与顶上标题合成行区间 |

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
| ### Run 1 Re-run (2026-10-20): Criterion 2 per Each Root Holding Only One Generation, Write-Only Without Deletion | 不抄 | 英文译文；中文原文同一节「### 第 1 次重跑（2026-10-20）：判据 2 按每条根只装一代、只写不删」照抄。英文标题里的半角冒号让 quote-kb.py 把取法切成「文件:行区间」，取不出（工具欠账，记进 subagent 拆分提案） |
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
