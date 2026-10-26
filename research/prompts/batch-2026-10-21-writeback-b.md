# kb-scribe 报告：batch-2026-10-21 里程碑页与 kb 导航页同步（写回员 b）

草稿目录：`/home/fy5090/.local/share/singlefs/work/batch-2026-10-21-writeback-b`
行清单：`/home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-rows/b.tsv`（73 行）

## 1. 规格文件与 kb-spec-check

| 文件 | 内容 | sha256 |
|---|---|---|
| `spec.json` | 中文原文 50 条（kb-spec-check 的规格，JSON） | `f1a322ee755b503b7da51faa6e9984a5c586bcee967177b67f275fe5afe6fa48` |
| `spec.md` | 同内容 markdown 写法 | `f855e71c6805563446c2bfb230d744ba8dff74f02307df541d2419336e1de08b` |
| `spec-translations.json` | 英、日译文 102 条（JSON） | `06754ce787eaa63cbb9acfc910cc5d7d7385b00d00690160c4e69c5a7257c186` |
| `spec-translations.md` | 同内容 markdown 写法 | `83c45c6edf644ce692f6840515f4f242023426002529aab62e58fbba837eb503` |
| `batch.json` | replace-batch 实际用的合并规格（152 处，中、英、日） | `09af64ed3ba2dc627b074f98cad196ac48be335b834771aa586e234a6740aa58` |

`python3 scripts/kb/kb-spec-check.py <草稿目录>/spec.json` 与 `spec.md` 的原样末行（退出码 0）：

```
  ✓ 规格 50 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 50 条）
```

译文那份 `spec-translations.json`（102 条）退出码 1，输出落在 `spec-translations-check.txt`，红 20 处，全部是同一类：
`D26 / E10 / C857 / C820 / C409 / C383 / C301（没带简称）`。原因：脚本只认全角括号的简称（`C383（简称）`），而英、日译文按
`.claude/kb/translation-glossary.md` 规定写半角 `C383 (译名)`，全角括号在译文里会被门禁「编号与简称」那一道当中文简称去比。
我逐条核过，这 20 处的新串里每个编号后面都紧跟半角括号的译名，旧串命中数、位置指代、时间指代、自称三类都是 0 红。
`grep "✗ 第" spec-translations-check.txt | grep -vc "没带简称"` 输出 0。这一类是脚本对译文的已知局限，不是新串写错；要主 agent 知道。

## 2. replace-batch.py 原样输出

`--dry-run`（退出码 0）：

```
✓ 152 处都命中得对（24 个文件），--dry-run 没写
```

实写（退出码 0）：

```
✓ 152 处替换，写了 24 个文件（改名换上新 inode），回读一致
```

写之前 24 个文件的路径逐个喂过写范围闸（`write-guard.sh`，agent_type=kb-scribe），退出码全是 0（`precheck rc=0`）。

## 3. 没跑的

- 没跑 `relabel-item.py`（这一份没有分项翻状态）。
- 没跑 `doc-decisions.sh --write --check decision-items-sync`（派发提示：并行写回，由主 agent 在全部交回后跑一次；也没有碰决策正文、索引表、变更史）。
- 没跑任何门禁阶段、三语四格、lint（交回前不跑，归主 agent 提交前）。

## 4. 改了哪几处（文件:行，改前一句、改后一句；每处只摘改动的核心一段，全文在 spec.json、spec-translations.json）

行号是写完之后的。同一处的中、英、日三份相邻（顺序：中、英、日），共 152 处（中文 50、译文 102）。

| 文件:行 | 改前 | 改后 |
|---|---|---|
| `milestone/08-eighth-txn.md:56` | 检出没有时间线索）、C344（单调统计量的点删遇墓碑没有会红的检查 | 检出没有时间线索）、C383（同一次发布里同 key 的中间态折叠没有检查）（C344（单调统计量的点删遇墓碑没有会红的检查） … |
| `milestone/08-eighth-txn.en.md:56` | me clue), C344 (monotonic metric point deletion encountering tom… | me clue), C383 (Intermediate state folding of same key in same p… |
| `milestone/08-eighth-txn.ja.md:56` | 的手がかりがない)、C344 (単調統計量のポイント削除が墓石に遭遇した際に赤となる検査の欠落) | 的手がかりがない)、C383 (同一パブリッシュ内での同キー中間状態の折りたたみ検査欠落)（C344 (単調統計量のポイント削除… |
| `milestone/02-second-txn.md:366` | ）、I-8.4（重放幂等 | ）、I-8.4（重放幂等，已接进崩溃注入的二次崩溃、没接进池级 checker |
| `milestone/02-second-txn.md:366` | .3（重放前缀严格连续）、I-8.4（重放幂等） 仍未实现 | .3（重放前缀严格连续） 仍未实现，I-8.4（重放幂等） 已实现在崩溃注入的二次崩溃上（不进池级 checker 清单） |
| `milestone/02-second-txn.en.md:352` |  Idempotence |  Idempotence, wired into the second crash of crash injection, no… |
| `milestone/02-second-txn.en.md:352` | play Prefix), I-8.4 (Replay Idempotence) still unimplemented | play Prefix) still unimplemented, I-8.4 (Replay Idempotence) imp… |
| `milestone/02-second-txn.ja.md:352` | -8.4 (再生の冪等性 | -8.4 (再生の冪等性、クラッシュ注入の二次クラッシュに配線済みでプールレベル checker には未配線 |
| `milestone/02-second-txn.ja.md:352` | ィックスの厳密な連続性)、I-8.4 (再生の冪等性) は未実装 | ィックスの厳密な連続性) は未実装、I-8.4 (再生の冪等性) はクラッシュ注入の二次クラッシュ上で実装済み（プールレベル c… |
| `milestone/02-second-txn.md:147` |  根环) + 1 = 2 写进两份系统配置再发一道屏障（`acquire_instance |  根环) + 1 = 2，分两轮写进每块盘的两槽系统配置（每轮逐盘写一槽、之后一道池屏障，`write_the_witnesse… |
| `milestone/02-second-txn.en.md:133` | = 2, writes to both system configurations followed by a barrier … | = 2, writes it to the two slots of the system configuration on e… |
| `milestone/02-second-txn.ja.md:133` | + 1 = 2 を取得し 2 部のシステム構成へ書き込みバリアを発行（`acquire_instance | + 1 = 2 を取得し、各ディスクの 2 スロットのシステム構成へ 2 ラウンドに分けて書き込む（各ラウンドでディスクごとに … |
| `milestone/01-first-txn.md:161` | 取号 = 逐盘一次系统配置槽写（世代号按那块盘自证过的槽里最大的 + 1）+ 一道屏障，新号 = max(每块盘两槽里全部自证过… | 取号 = 两轮系统配置槽写，每轮逐盘写一槽（世代号按那块盘自证过的槽里最大的 + 1）、之后一道池屏障，新号 = max(每块盘… |
| `milestone/01-first-txn.en.md:134` | quisition = one system configuration slot write per disk (genera… | quisition = two rounds of system configuration slot writes, each… |
| `milestone/01-first-txn.ja.md:134` | 番号取得 = ディスクごとにシステム構成スロット書き込み 1 回（世代番号はそのディスクで自己検証済みスロットの最大値 + 1）… | 番号取得 = システム構成スロット書き込み 2 ラウンド、各ラウンドでディスクごとに 1 スロットを書き（世代番号はそのディスク… |
| `milestone/02-second-txn.md:7` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/02-second-txn.md:486` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/02-second-txn.en.md:472` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/02-second-txn.ja.md:472` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.md:55` | ode 号写数据的入口：写数据的入口只收第一个文件（`crates/singlefs-core/src/mounted_sess… | ode 号写数据的入口：入口 `UserChange::OverwriteOfAnInode`、`UserChange::Seq… |
| `milestone/03-third-txn.md:58` | nsaction.rs`，排在里程碑四之后 | nsaction.rs`），入口与三条流已随里程碑四核心层那批代码合入（提交 0b926bd3f |
| `milestone/03-third-txn.en.md:55` | write data entry currently accepts only first file (`UserChange:… | entries `UserChange::OverwriteOfAnInode` and `UserChange::Sequen… |
| `milestone/03-third-txn.en.md:58` | nsaction.rs`, scheduled after Milestone 4 | nsaction.rs`); the entry and the three streams have landed with … |
| `milestone/03-third-txn.ja.md:55` | データ書き込み入口は最初のファイルのみを受理（`crates/singlefs-core/src/mounted_session… | 入口 `UserChange::OverwriteOfAnInode`、`UserChange::SequentialWrite… |
| `milestone/03-third-txn.ja.md:58` | tion.rs` を改修、マイルストーン四の後に配置 | tion.rs` を改修）、入口と三本のストリームはマイルストーン四のコア層のバッチとともにマージ済み（コミット 0b926bd… |
| `milestone/03-third-txn.md:149` | 常规层、大负载层 在这个里程碑立阶段（C409（常规层、大负载层在门禁里没有阶段）（已挪 [04-fourth-txn.md](… | 常规层、大负载层 的立阶段已挪里程碑四 [04-fourth-txn.md](04-fourth-txn.md) 八·3（用户 … |
| `milestone/03-third-txn.md:152` | 常规层、大负载层 立阶段（ | 常规层、大负载层 的立阶段已挪里程碑四 [04-fourth-txn.md](04-fourth-txn.md) 八·3（C40… |
| `milestone/03-third-txn.en.md:149` | normal tier and large workload tier establish stages in this mil… | establishing stages for the normal tier and large workload tier … |
| `milestone/03-third-txn.en.md:152` | normal tier and large workload tier establish stages ( | establishing stages for the normal tier and large workload tier … |
| `milestone/03-third-txn.ja.md:149` | 、大規模負荷層のステージを本マイルストーンで開設（C409 (通常層と大規模負荷層がゲート内にステージを持たない)（[04-fo… | 、大規模負荷層のステージ開設はマイルストーン四 [04-fourth-txn.md](04-fourth-txn.md) 八·3… |
| `milestone/03-third-txn.ja.md:152` | 規模負荷層のステージ開設（ | 規模負荷層のステージ開設はマイルストーン四 [04-fourth-txn.md](04-fourth-txn.md) 八·3 へ… |
| `milestone/03-third-txn.md:72` | 放时凑不齐映射 key） | 放时凑不齐映射 key）（已还清） |
| `milestone/03-third-txn.md:107` |  MiB 的盘建不了卷） |  MiB 的盘建不了卷）（已还清） |
| `milestone/03-third-txn.md:107` | 引用隔离单元不进逐盘验） | 引用隔离单元不进逐盘验）（已还清） |
| `milestone/03-third-txn.md:107` | 入先拒连带红，改法待选） | 入先拒连带红，改法待选）（已还清） |
| `milestone/03-third-txn.md:124` | 漂移、第 12 行不红） | 漂移、第 12 行不红）（已还清：两条用例的结局已钉、变异已按名字登记，见欠账表已还清表） |
| `milestone/03-third-txn.en.md:72` | rnal replay) | rnal replay) (cleared) |
| `milestone/03-third-txn.en.md:107` | e tightened) | e tightened) (cleared) |
| `milestone/03-third-txn.en.md:107` | erification) | erification) (cleared) |
| `milestone/03-third-txn.en.md:107` | ding choice) | ding choice) (cleared) |
| `milestone/03-third-txn.en.md:124` | w 12 un-red) | w 12 un-red) (cleared: the outcomes of the two test cases are pi… |
| `milestone/03-third-txn.ja.md:72` | ングキーを揃えられない) | ングキーを揃えられない)（完済） |
| `milestone/03-third-txn.ja.md:107` | クでボリューム作成不能) | クでボリューム作成不能)（完済） |
| `milestone/03-third-txn.ja.md:107` | ィスク別検証に入らない) | ィスク別検証に入らない)（完済） |
| `milestone/03-third-txn.ja.md:107` | 的に赤、改訂法選定待ち) | 的に赤、改訂法選定待ち)（完済） |
| `milestone/03-third-txn.ja.md:124` | フト、第 12 行不赤） | フト、第 12 行不赤）（完済：2 テストケースの結末は固定済み、ミューテーションは名前で登録済み、未済の検査完済表参照） |
| `milestone/03-third-txn.md:141` | 登记的流 29 | 登记的流 35 |
| `milestone/03-third-txn.en.md:141` | reams count 29 | reams count 35 |
| `milestone/03-third-txn.ja.md:141` | 登録済みストリーム 29 | 登録済みストリーム 35 |
| `milestone/03-third-txn.md:50` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.md:113` | /2026-10-10.zh. | /2026-10-10. |
| `milestone/03-third-txn.md:141` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.md:141` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.en.md:50` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.en.md:113` | /2026-10-10.zh. | /2026-10-10. |
| `milestone/03-third-txn.en.md:141` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.en.md:141` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.ja.md:50` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.ja.md:113` | /2026-10-10.zh. | /2026-10-10. |
| `milestone/03-third-txn.ja.md:141` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/03-third-txn.ja.md:141` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/04-fourth-txn.md:35` | 其余先做 \| 未开 | 其余先做 \| 做了（一·1 `research/prompts/m4-snapshot-r1-main-verificatio… |
| `milestone/04-fourth-txn.md:36` | 先有共享才造得出来 \| 未开 | 先有共享才造得出来 \| 核心层已合入 master（提交 0b926bd3f）：一·2 建只读快照 / 快照下覆盖写与删文件 … |
| `milestone/04-fourth-txn.md:37` | 、7 在这一段满足 \| 未开 | 、7 在这一段满足 \| 流已入库并登记在 `.claude/gate.d/stage-inputs.tsv` 的 `crash… |
| `milestone/04-fourth-txn.en.md:34` | ed first) \| Unopened | ed first) \| Done (I·1 `research/prompts/m4-snapshot-r1-main-ver… |
| `milestone/04-fourth-txn.en.md:35` | nstructed \| Unopened | nstructed \| Core layer merged into master (commit 0b926bd3f): I… |
| `milestone/04-fourth-txn.en.md:36` | s segment \| Unopened | s segment \| Streams are in the repository and registered in the… |
| `milestone/04-fourth-txn.ja.md:34` | 残りは先行可能） \| 未着手 | 残りは先行可能） \| 完了（一·1 `research/prompts/m4-snapshot-r1-main-verific… |
| `milestone/04-fourth-txn.ja.md:35` | して初めて構築可能 \| 未着手 | して初めて構築可能 \| コア層は master にマージ済み（コミット 0b926bd3f）：一·2 読み取り専用スナップショ… |
| `milestone/04-fourth-txn.ja.md:36` | 本セグメントで充足 \| 未着手 | 本セグメントで充足 \| ストリームは取り込み済みで `.claude/gate.d/stage-inputs.tsv` の `… |
| `milestone/04-fourth-txn.md:66` | （GPU 的口子，「优化项：不在出口，进入条件已登记」一节 | （GPU 垃圾回收在出口 8，见「七　GPU 垃圾回收」，拿这份纯函数当 CPU 参照 |
| `milestone/04-fourth-txn.en.md:65` | (GPU interface, section "Optimization items: Not in exit criteri… | (GPU garbage collection is exit 8, see "VII GPU Garbage Collecti… |
| `milestone/04-fourth-txn.ja.md:65` | （GPU への接続口、「最適化項目：出口対象外、参入条件登録済み」節 | （GPU ガベージコレクションは出口 8、「七　GPU ガベージコレクション」を参照、この純粋関数を CPU 参照とする |
| `milestone/04-fourth-txn.md:66` | 项 6）；discard | 项 6）；discard（发布返回之后的 discard 已接上产品路径：`crates/singlefs-core/src/m… |
| `milestone/04-fourth-txn.en.md:65` | 6); discard  | 6); discard (the discard after a publish returns is wired into t… |
| `milestone/04-fourth-txn.ja.md:65` | 目 6）；discard | 目 6）；discard（パブリッシュ復帰後の discard は製品パスに接続済み：`crates/singlefs-core… |
| `milestone/04-fourth-txn.md:45` | 没有任何写它们的操作；`grep -rn "克隆" crates/singlefs-core/src` 只命中 `transac… | deadlist 已有写者（`transaction.rs` 的 `publish_create_snapshot` 建快照，快… |
| `milestone/04-fourth-txn.md:45` | 删除批与覆盖写今天共用 `crates/singlefs-core/src/transaction.rs` 里 `for rel… | 删除批与覆盖写共用的释放按快照分支：被快照引用的块进 deadlist，别的照常释放（`crates/singlefs-core… |
| `milestone/04-fourth-txn.en.md:44` | with zero operations writing them; `grep -rn "克隆" crates/singlef… | the deadlist now has writers (`publish_create_snapshot` in `tran… |
| `milestone/04-fourth-txn.en.md:44` | Today deletion batches and overwrite share a single `allocator.r… | Deletion batches and overwrite share a release that branches on … |
| `milestone/04-fourth-txn.ja.md:44` | これらへの書き込み操作は皆無；`grep -rn "克隆" crates/singlefs-core/src` は `trans… | deadlist には書き込み手がある（`transaction.rs` の `publish_create_snapshot`… |
| `milestone/04-fourth-txn.ja.md:44` | 削除バッチと上書きは現在 `crates/singlefs-core/src/transaction.rs` 内の `for r… | 削除バッチと上書きが共用する解放はスナップショットで分岐する：スナップショットに参照されるブロックは deadlist に入り、… |
| `milestone/04-fourth-txn.md:87` | 扫出来的只供 I-7.8（根记录树 ID 水位不低于全池最大树 ID）、I-1.8（归并后版本全序） 与 journal 那几条… | 扫出来的供 I-7.8（根记录树 ID 水位不低于全池最大树 ID）、I-1.8（归并后版本全序） 与 journal 那几条判… |
| `milestone/04-fourth-txn.md:93` | ·1；deadlist 与 livelist 那一部分等 C301（deadlist 与 livelist 的态别没写）  | ·1；deadlist 条目与被钉住单元的映射条目是权威态（C301（deadlist 与 livelist 的态别没写） 已定… |
| `milestone/04-fourth-txn.md:105` | deadlist / livelist 等快照落地之后、C301（deadlist 与 livelist 的态别没写）  | deadlist 条目与被钉住单元的映射条目是权威态，不由扫描重建；销毁队列（带销毁中记号的码 14 条目）是派生态，第一版不重… |
| `milestone/04-fourth-txn.en.md:86` | esults used only for judgments of I-7.8 (root record tree ID wat… | esults used for judgments of I-7.8 (root record tree ID watermar… |
| `milestone/04-fourth-txn.en.md:92` | 1; deadlist and livelist parts await settlement | 1; deadlist entries and the mapping entries of pinned units are … |
| `milestone/04-fourth-txn.en.md:104` | deadlist / livelist compared after snapshot lands and | deadlist entries and the mapping entries of pinned units are aut… |
| `milestone/04-fourth-txn.ja.md:86` |  journal の判定のみに使用；checker 内にスキャン結果を用いてインデックスを再構築するコードは存在しない（2026… |  journal の判定に使用されるほか、読み取り専用スキャン再構築 `crates/singlefs-checker/src/… |
| `milestone/04-fourth-txn.ja.md:92` | ·1；deadlist および livelist 部分は C301 (deadlist と livelist の状態区別が未記述… | ·1；deadlist エントリと固定されたユニットのマッピングエントリは権威状態（C301 (deadlist と livel… |
| `milestone/04-fourth-txn.ja.md:104` | deadlist / livelist 等はスナップショット着地後、C301 (deadlist と livelist の状態区… | deadlist エントリと固定されたユニットのマッピングエントリは権威状態でありスキャンでは再構築しない；破棄キュー（破棄中の… |
| `milestone/04-fourth-txn.md:60` | 代码：`crates/singlefs-core/src/allocator.rs` 的 `reclaim_released_u… | 代码：整理已入库——意图条目格式在 `crates/singlefs-core/src/logged_operations.rs… |
| `milestone/04-fourth-txn.md:60` | 定案 K-I1：零根注册、字段表等整理落地时再定，原话在变更史，被攻过零轮），今天代码里只有零根与种类码 `TREE_KIND_… | 定案 K-I1：零根注册，原话在变更史，被攻过零轮）；字段表后来由判决 `research/prompts/m4-gc-r1-m… |
| `milestone/04-fourth-txn.en.md:59` | Code: `reclaim_released_up_to` in `crates/singlefs-core/src/allo… | Code: compaction has landed — intent entry format in `crates/sin… |
| `milestone/04-fourth-txn.en.md:59` | gistration, field table to be settled when compaction lands, ori… | gistration, original words in changelog, zero attack rounds); th… |
| `milestone/04-fourth-txn.en.md:59` | E10 (aging: structural anti-aging vs post-hoc compaction) unru | the first run of E10 (aging: structural anti-aging vs post-hoc c… |
| `milestone/04-fourth-txn.en.md:59` | E10 (aging: structural anti-aging vs post-hoc compaction) has no… | The pre-run registration of E10 (aging: structural anti-aging vs… |
| `milestone/04-fourth-txn.ja.md:59` | コード：`crates/singlefs-core/src/allocator.rs` の `reclaim_released_… | コード：整理は取り込み済み——インテントエントリ形式は `crates/singlefs-core/src/logged_ope… |
| `milestone/04-fourth-txn.ja.md:59` | -I1：ゼロルート登録、フィールド表等は整理着地時に決定、原話は変更履歴、攻撃ゼロ回）；現在コード内にはゼロルートと種別コード … | -I1：ゼロルート登録、原話は変更履歴、攻撃ゼロ回）；フィールド表はその後、判定 `research/prompts/m4-gc… |
| `milestone/04-fourth-txn.ja.md:59` | ング vs 事後整理) は未実行 | ング vs 事後整理) の初回実行はまだ結論に至っていない |
| `milestone/04-fourth-txn.ja.md:59` | ング vs 事後整理) は実行前登録がなく（`research/prompts` 配下に `e10-*` はゼロ件）、 | ング vs 事後整理) の実行前登録は `research/prompts/e10-r1-prereg.md`（2026-10-… |
| `milestone/04-fourth-txn.md:65` | 轮：整理意图记录的字段表 | 轮：整理意图记录的字段表（已定：D26（后台整理与放置回收） 已定项 2 的字段表，key 10 + value 34；仍欠 I… |
| `milestone/04-fourth-txn.en.md:64` | tent records | tent records (settled: the field table of settled item 2 of D26 … |
| `milestone/04-fourth-txn.ja.md:64` | トレコードのフィールド表 | トレコードのフィールド表（確定済み：D26 (バックグラウンド整理と配置回収) 確定項目 2 のフィールド表、key 10 + … |
| `milestone/04-fourth-txn.md:188` | 026-10-12 现查 | 026-10-12 现查；第 1 问一栏后来更新，第 2 问一栏「没有按快照分支」已不成立，一·2 已按快照分支 |
| `milestone/04-fourth-txn.md:192` |  K-I1 让它零根注册、字段表等整理落地再定（被攻过零轮） |  K-I1 让它零根注册（被攻过零轮），字段表已由判决 `research/prompts/m4-gc-r1-main-veri… |
| `milestone/04-fourth-txn.md:192` | fs-core/src/records.rs` 的 `TREE_KIND_LOGGED_OPERATIONS` 没有写者 | fs-core/src/transaction.rs` 已有写 logged_ops 条目的代码（`logged_operati… |
| `milestone/04-fourth-txn.md:192` | 二·1 先定 logged_ops 的字段表 | 字段表已定，仍欠 I-4.4（意图只描述目标态） 的判法 |
| `milestone/04-fourth-txn.en.md:184` | n 2026-10-12 | n 2026-10-12; the answer column of question 1 was updated later,… |
| `milestone/04-fourth-txn.en.md:188` | registration, field table to be settled when compaction lands (z… | registration (zero attack rounds), and the field table was settl… |
| `milestone/04-fourth-txn.en.md:188` | `TREE_KIND_LOGGED_OPERATIONS` in `crates/singlefs-core/src/recor… | `crates/singlefs-core/src/transaction.rs` already has code writi… |
| `milestone/04-fourth-txn.en.md:188` | II·1 settles field table of logged_ops first | Field table settled, judging method of I-4.4 (intent only descri… |
| `milestone/04-fourth-txn.ja.md:184` | 026-10-12 調査 | 026-10-12 調査；第 1 問の回答欄はその後更新され、第 2 問の回答欄の「スナップショット分岐なし」は成立しなくなり、… |
| `milestone/04-fourth-txn.ja.md:188` | 1 によりゼロルート登録、フィールド表等は整理着地時に決定（攻撃ゼロ回） | 1 によりゼロルート登録（攻撃ゼロ回）、フィールド表は判定 `research/prompts/m4-gc-r1-main-ve… |
| `milestone/04-fourth-txn.ja.md:188` | fs-core/src/records.rs` の `TREE_KIND_LOGGED_OPERATIONS` に書き込み手なし | fs-core/src/transaction.rs` に logged_ops エントリを書くコードがある（`logged_o… |
| `milestone/04-fourth-txn.ja.md:188` | 二·1 で logged_ops のフィールド表を先行決定 | フィールド表は確定済み、I-4.4 (インテントは目標状態のみを記述する) の判定法が未済 |
| `milestone/04-fourth-txn.md:75` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/04-fourth-txn.en.md:74` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/04-fourth-txn.ja.md:74` | /2026-09-28.zh. | /2026-09-28. |
| `milestone/04-fourth-txn.md:131` | 写段在 m4-predev 工作树里做（ | 写段原在 m4-predev 工作树里做，已合进 master（提交 0b926bd3f；过程见  |
| `milestone/04-fourth-txn.en.md:130` | Implementation segment performed in m4-predev worktree ( | The implementation segment was originally performed in the m4-pr… |
| `milestone/04-fourth-txn.ja.md:130` | 実装セグメントは m4-predev ワークツリーで実施（ | 実装セグメントは元は m4-predev ワークツリーで実施し、master にマージ済み（コミット 0b926bd3f；経過は… |
| `milestone/05-fifth-txn.md:22` |  1、2、3、5、6、7 |  1、2、3、5、6、7、8、9 |
| `milestone/05-fifth-txn.md:22` | 未满足：代码在工作树 `~/.local/share/singlefs/work/m4-predev`（分支 `m4-prede… | 未满足：代码已合进 master（提交 0b926bd3f，m4-predev 工作树已不在 |
| `milestone/05-fifth-txn.md:23` | 程碑四的未提交改动 \| 未满足：m4-predev 工作树 `git status --short` 285 条，全部未提交 | 程碑四的未提交改动 \| 满足（2026-10-21 现查）：`git log master` 里有里程碑四的提交 0b926b… |
| `milestone/05-fifth-txn.md:24` | 未满足：只在 m4-predev 工作树里有 | 满足（2026-10-21 现查，提交 0b926bd3f）：表里八行的落点在 master 里都在 |
| `milestone/05-fifth-txn.md:37` | 整理接口按 m4-predev 工作树谈，材料写明以哪棵树为准，里程碑四合入之后 | 整理接口已合入 master（提交 0b926bd3f），材料写明以哪个提交为准，开工时 |
| `milestone/05-fifth-txn.md:41` | 2026-10-17 在 m4-predev 工作树里现查（未合入 master，不算数） | 2026-10-21 在 master 的提交 0b926bd3f 里现查 |
| `milestone/05-fifth-txn.md:52` | 位 0（退役）、位 1 在用；判决 `research/prompts/m4-trunc-r2-main-verificatio… | 位 0（退役）、位 1、位 2（截断字段，`INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BI… |
| `milestone/05-fifth-txn.md:111` | 与意图机制在里程碑四落地：2026-10-17 只在 m4-predev 工作树里有，落点见「里程碑四交出的整理接口」表；合进 … | 与意图机制在里程碑四落地，已合进 master（提交 0b926bd3f），落点见「里程碑四交出的整理接口」表； |
| `milestone/05-fifth-txn.en.md:22` | , 3, 5, 6, 7 | , 3, 5, 6, 7, 8, 9 |
| `milestone/05-fifth-txn.en.md:22` | sfied: code in worktree `~/.local/share/singlefs/work/m4-predev`… | sfied: code merged into master (commit 0b926bd3f, the m4-predev … |
| `milestone/05-fifth-txn.en.md:23` | 4 changes \| Unsatisfied: m4-predev worktree `git status --short… | 4 changes \| Satisfied (checked 2026-10-21): `git log master` co… |
| `milestone/05-fifth-txn.en.md:24` | Unsatisfied: present only in m4-predev worktree | Satisfied (checked 2026-10-21, commit 0b926bd3f): all eight rows… |
| `milestone/05-fifth-txn.en.md:37` |  interfaces discussed based on m4-predev worktree, materials spe… |  interfaces already merged into master (commit 0b926bd3f), mater… |
| `milestone/05-fifth-txn.en.md:41` | Checked in m4-predev worktree on 2026-10-17 (unmerged to master,… | Checked on 2026-10-21 in commit 0b926bd3f of master |
| `milestone/05-fifth-txn.en.md:52` | etired), bit 1 in use; verdict `research/prompts/m4-trunc-r2-mai… | etired), bits 1 and 2 in use (bit 2 for truncation fields, `INCO… |
| `milestone/05-fifth-txn.en.md:111` |  Milestone 4: as of 2026-10-17 present only in m4-predev worktre… |  Milestone 4, merged into master (commit 0b926bd3f), placements … |
| `milestone/05-fifth-txn.ja.md:22` |  1、2、3、5、6、7 |  1、2、3、5、6、7、8、9 |
| `milestone/05-fifth-txn.ja.md:22` | 未充足：コードは作業ツリー `~/.local/share/singlefs/work/m4-predev`（ブランチ `m4-… | 未充足：コードは master にマージ済み（コミット 0b926bd3f、m4-predev 作業ツリーは既に存在しない |
| `milestone/05-fifth-txn.ja.md:23` | ト変更が存在しない \| 未充足：m4-predev 作業ツリーの `git status --short` が 285 件、す… | ト変更が存在しない \| 充足（2026-10-21 現物確認）：`git log master` にマイルストーン四のコミット… |
| `milestone/05-fifth-txn.ja.md:24` | 未充足：m4-predev 作業ツリー内のみ | 充足（2026-10-21 現物確認、コミット 0b926bd3f）：表の八行すべての配置先が master  |
| `milestone/05-fifth-txn.ja.md:37` | 理インターフェースは m4-predev 作業ツリーを元に協議、資料に依拠ツリーを明記、マイルストーン四マージ後 | 理インターフェースは master（コミット 0b926bd3f）にマージ済み、資料に依拠するコミットを明記、着工時 |
| `milestone/05-fifth-txn.ja.md:41` | 2026-10-17 に m4-predev 作業ツリー内で現物確認（master 未マージのため算入せず） | 2026-10-21 に master のコミット 0b926bd3f で現物確認 |
| `milestone/05-fifth-txn.ja.md:52` | 0（退役）、ビット 1 使用中；判定 `research/prompts/m4-trunc-r2-main-verificati… | 0（退役）、ビット 1 とビット 2 使用中（ビット 2 は切り詰めフィールド、`INCOMPAT_INODE_RECORD_T… |
| `milestone/05-fifth-txn.ja.md:111` | 整理器と意図メカニズムはマイルストーン四で着地：2026-10-17 時点では m4-predev 作業ツリーのみに存在、配置先… | 整理器とインテントメカニズムはマイルストーン四で着地し、master にマージ済み（コミット 0b926bd3f）、配置先は「マ… |
| `milestone/09-ninth-txn.md:119` | 实验索引里状态是「未跑」 | 实验索引里 E10（老化：结构抗老化 vs 事后整理） 的状态格现写「未跑」，实际装置已入库、第一段 21 片跑了 4 片、读数… |
| `milestone/09-ninth-txn.en.md:119` | status is "not run" in experiment index | the status cell of E10 (aging: structural resistance vs post-hoc… |
| `milestone/09-ninth-txn.ja.md:119` | 実験インデックスにおける状態は「未実行」 | 実験インデックスの E10 (老化：構造的抗老化 vs 事後デフラグ) の状態欄は現在「未実行」だが、実際には装置は取り込み済み… |
| `INDEX.md:28` | 仓根 README 与 `briefs | 仓根 README、`briefs/` 与 `.claude/kb |
| `INDEX.en.md:28` |  root README and `briefs |  root README, `briefs/` and `.claude/kb |
| `INDEX.ja.md:28` | ルート README および `briefs | ルート README、`briefs/` および `.claude/kb |

## 5. 剔出的行（依据只在暂存区或工作区、不在 HEAD，没写）

| 行清单 | 载体 | 为什么剔 |
|---|---|---|
| 第 3 段 F17 | `milestone/03-third-txn.md:71` | C656「已照用户定案做完」依据 mkfs 两趟写序、随机 fsid 与用例 `crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs`。`git cat-file -e HEAD:crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs` 不成立（`git status` 里是 `A` 暂存），HEAD 的 stage-inputs.tsv 里也没有 `make-filesystem-over-a-used-pool-stream` 行；`crates/singlefs-core/src/make_filesystem.rs` 是 `M` 暂存。交那个会话自己同步 |
| 第 3 段 F17 | `milestone/03-third-txn.en.md:71` | 同上 |
| 第 3 段 F17 | `milestone/03-third-txn.ja.md:71` | 同上 |

行清单 73 行里写了 70 行，剔 3 行（F17 三语）；候选之外的同类行（第 9 段的入·2、入·3、二·1、整理接口表小标题、`04-fourth-txn.md`「写段在 m4-predev 工作树里做」）已处理，见第 4 节。

一处要说清：第 3 段 F59 里「登记的流 35 条」。`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 现数 35；HEAD 版（`git show HEAD:.claude/gate.d/stage-inputs.tsv | grep -c '^crash-case:'`）是 34，多出来的一条是已暂存的 stage-inputs.tsv 新加的 `make-filesystem-over-a-used-pool-stream`，它指向上面那个只在暂存区的用例。我按判定规则 1 的「HEAD 加这一轮已暂存的门禁与脚本改动」写了 35。如果主 agent 决定那一批 crates 不和 stage-inputs.tsv 一起提交，三语 `03-third-txn` 141 行的 35 要改回 34。

## 6. 写之前现查的事实（命令，基准 HEAD）

- `git merge-base --is-ancestor 0b926bd3f HEAD && echo ancestor` 输出 `ancestor`；`git worktree list` 只有主仓、`m3-close-w12-integration/tree`、`m5-dev`，没有 m4-predev。
- 取号两轮：`git grep -n "write_the_witnessed_acquisition" HEAD -- crates` 命中 `transaction.rs:1225`；`git show HEAD:crates/singlefs-core/src/transaction.rs | sed -n 1195,1260p` 里 `ACQUISITION_WRITE_ROUNDS` 两个成员，每轮逐盘 `write_system_configuration_slot`、之后 `CommitStep::Barrier`。`git ls-tree HEAD --name-only crates/singlefs-checker-tier/tests/ | grep acquisition` 有 `crash_enumeration_acquisition_barrier.rs`。`.claude/kb/layout/01-first-txn.md:411,420` 现文已写取号两轮、「取号写一轮时」。
- 按 inode 号写：`git show HEAD:crates/singlefs-core/src/mounted_session.rs | grep -n "OverwriteOfAnInode\|SequentialWriteOfAnInode"` 命中；`transaction.rs` 有 `publish_overwrite_of_an_inode`（8718）、`publish_sequential_write_of_an_inode`（8769）；三条流用例文件 `git cat-file -e HEAD:…` 都成立，`git show HEAD:.claude/gate.d/stage-inputs.tsv | grep -c <流名>` 各 1。
- 快照：`transaction.rs` 有 `publish_create_snapshot`（8885）、`publish_snapshot_destruction_intent`（8963）、`snapshot_lifetime_verdict`（5944）；`grep -n livelist` 在 HEAD 的 `transaction.rs` 里只命中树表字段与注册（3003–3108、13469），`sparse_side_table` 同；`crates/singlefs-checker/invariants/05-空间记账.md` 状态列：I-3.6、I-3.7「已实现（2026-10-15」，I-3.5「部分实现」且 ③ ④ 已实现。
- 整理：`git cat-file -e HEAD:` 对 `compaction_plan.rs`、`region_scan.rs`、`logged_operations.rs`、`truncate.rs`、`system_configuration_repair.rs`、`crates/singlefs-checker/src/rebuild.rs`、`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs` 都成立；`run_compaction`（`mounted_session.rs:1938`）注释写「批大小条款没写，产品路径在签任何意图之前停下」；`discard_the_segments_reusable_as_a_whole` 在 `mounted_session.rs` 的 1920（用户改动的发布之后）、1994（整理显式入口之后）两处被调；`git grep -c discard HEAD -- crates/singlefs-core/src` 得 allocator 18、block_device 135、mounted_session 31；`COMPACTION_WATERMARK_BUILT_IN_DEFAULT: u64 = 0` 仍在（`system_configuration.rs:103`）；`logged_operations.rs` 有 `to_bytes`、`parse`，`LOGGED_OPERATIONS_KEY_WIDTH_IN_BYTES: usize = 10`，D26 已定项 2 写「合计 44 字节：key 10、value 34」；`INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT: u8 = 0x04`（`system_configuration.rs:35`）在 `SUPPORTED_INCOMPAT_BITS` 里；`unit.rs` 的 `UNIT_CLASS_DATA=1`、`INDEX_NODE=2`、`PACKED=3`；`logged_operations.rs` 种类码 1 整理、2 截断。
- 设计判决：`git cat-file -e HEAD:research/prompts/{m4-snapshot-r1,m4-gc-r1,m4-trunc-r1,m4-trunc-r2}-main-verification.md`、`HEAD:research/prompts/e10-r1-prereg.md` 都成立；`ls research/results/e10-aging-compaction-2026-10-13/` 4 个 `.out`。
- D3 已定项 7 现文：`grep -n "N₀\` 第一版取" .claude/kb/decisions/03-空间分配.md` 得第 148 行「`N₀` 第一版取 1，算式 …等计数模型验过再定」；`grep -n "^| C820 " .claude/kb/checks-owed.md` 得 535 行。
- C409：`grep -n "^| C409 " .claude/kb/checks-owed.md` 得 294 行（去向里程碑四八·3）；`04-fourth-txn.md` 八·3 行在。
- C296、C568、C572、C576：`grep -n "^| C296 \|^| C568 \|^| C572 \|^| C576 " .claude/kb/checks-owed.md` 得 860、858、852、859 行（已还清表）。
- `ls briefs`：只有 `.md`、`.en.md`、`.ja.md`；`.claude/kb/milestone/` 与 INDEX 里 `grep -c "\.zh\.md"` 改完为 0。
- `.claude/kb/layout/` 下仍只有 01–03，没有 `04-` 开头的一份（05-fifth 入·2 的「没有 04- 开头的那份」仍真，保留）。

## 7. 要主 agent 判的点（我没替它选）

1. **`04-fourth-txn` 实现进度那一句只改了一半**（中 131 行、英日 130 行）：我只把「写段在 m4-predev 工作树里做」改成「原在 m4-predev 做，已合进 master（提交 0b926bd3f）」。同一句里「m4-trunc-r2 那一份（核心层、池级 checker、模型、GPU）还没派」「验证段没开」「`every_step_image_of_the_fixed_truncation_history_is_green_under_the_pool_checker` 今天红，等 m4-trunc-r2 的实现」没核：HEAD 里已有 `truncate.rs`、incompat 位 2 常量、`crates/mutations.tsv` 里点名 m4-trunc-r2 的「截庚」变异行，看着多半过时，但那个用例红不红要跑 checker 档才知道。
2. **`05-fifth-txn` 入·2** 里「截断还差一批实现（截断#9）」「验证段没跑完」「崩溃流全域没跑」同样没核，保留；只改了「代码在工作树 m4-predev」一段。出口列表按 `04-fourth-txn.md` 现在的出口表写成 1、2、3、5、6、7、8、9（行清单 F23 只说补 8；主 agent 这一轮加了第 9 件，所以我连 9 一起补）。
3. **`04-fourth-txn` 英、日出口表还没跟上**：英、日写「Seven items」「七件」，只到出口 8，没有出口 9；05 英、日现在写「出口 …、8、9」，出口 9 在 04 英、日里要等主 agent 翻出口表那一行才对得上。
4. **分组表「3 覆盖」状态格**：我写「流已入库并登记在 stage-inputs.tsv 的 crash-case: 行；全域 0 红与二·4 的性质证明还没有记录」。后半句依据是 `research/results/` 里没有这几条流的全域产物（只有 E184 的代价计数产物），没有更强的证据；如果全域跑过，要改。
5. **分组表「0 接手」「GPU GC」两行**状态格仍是「未开」，不在行清单里，没动。
6. **三问答案现状表**：第 1 问一栏按 F41 改了；第 2 问一栏「没有按快照分支」现在不真（`snapshot_lifetime_verdict`），行清单没有这一行，我只在表标题后注明「第 2 问一栏的「没有按快照分支」已不成立」，没改那一栏的正文和「里程碑四要做的」一栏（「按「否」办：一·2 改这处共用释放（推的，没量过）」）。要不要把第 2 问一栏整行重写，主 agent 定。
7. **判决与我现查不一致的两处，按我现查的写了**：(1) 行清单 F34 写「deadlist 与 livelist 已有写者」，`grep livelist` 在 HEAD 的 `transaction.rs` 里只有树表字段与注册，没有写者（livelist 是可写克隆那一半，C301 登记行也写仍欠），我写成「deadlist 已有写者，livelist 与稀疏旁表仍只注册、没有写者」；(2) 同一行「I-3.5 部分实现：出现规则已实现，③ ④ 未实现」里 ③ ④ 的「未实现」不真（05-空间记账.md 状态列写 ③ ④ 连同逆向已实现，2026-10-15），我写成已实现，余下以状态格为准。
8. **英、日 04 的 E10 两句顺手跟上了中文**：59 行「E10 未跑」「E10 没有跑前登记」主 agent 已在中文改了，英、日还是旧句，我同一次改动里译上了（不在行清单里）。
9. **英、日 `03-third-txn` 的 141、149 行有中文没有的 C862 提法**（`C862（长跑分片续跑不核装置是哪一版）`），是本来就有的三语漂移，没碰。
10. **`decisions.md` 在我开工到收工之间被别的写回员改了**（sha256 前后不同），不是我写的，我没碰。
11. **C364 简称**：只改了正文里「字段表未定」「没有写者」的现状句，没改简称（派发提示 4）。
12. 03 的 C409 两处（六·4、开工前要先定的）按判定规则 3 改成指到里程碑四八·3，同一行第三格的「（已挪 …八·3）」原样保留；行清单标「要人看」的两行（第 5 段 F27）已按规则 3 处理，没有别的「要人看」行留下。
13. 行清单 F31 要求的状态格里「各作业点是否逐项做完，以主 agent 对各作业点行的核对为准」我没写进正文，只写了能现查的事实。

## 8. 没做什么

- 没写欠账表、决策正文、不变量清单、变更史；没碰 `layout/`、`checks-owed*.md`、`experiments*.md`。
- 没判定案对不对；没改 C364 简称；没提交，没跑门禁。
- 剔出的 3 行（F17 三语）没写。
- 没碰 `04-fourth-txn.md` 里别的会话暂存的七·6 那一行（`git diff -- 04-fourth-txn.md` 改前改后，七·6 的行没出现在我的 152 处里）。
- 没有起重活，没起后台任务，没建仓副本与编译目录。

## 9. 改过的全部文件与前后 sha256

开工时（写之前）与收尾时各记一次；`.claude/kb/decisions.md` 与 `decisions-history.md` 是派发提示要求一并记的，我没写它们。

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/INDEX.en.md` | `536032724d29667b…` | `ad4aa10a327b46dd…` |
| `.claude/kb/INDEX.ja.md` | `6737f668b06298bd…` | `e219cb92fa35dd85…` |
| `.claude/kb/INDEX.md` | `c735c7d6103dcf76…` | `85fbf53e4fbea2b9…` |
| `.claude/kb/decisions-history.md` | `873e66d2dc1d6500…` | `873e66d2dc1d6500…`（未变） |
| `.claude/kb/decisions.md` | `9abb9da05230e6e0…` | `7102be6513e29932…`（别的写回员改了，不是我） |
| `.claude/kb/milestone/01-first-txn.en.md` | `9c8a23101bbd3edf…` | `5fab507cbd98b75e…` |
| `.claude/kb/milestone/01-first-txn.ja.md` | `9e1e484fdad5f871…` | `89532df50aa505c8…` |
| `.claude/kb/milestone/01-first-txn.md` | `683b2eef23e166a3…` | `12bd951d76b3926d…` |
| `.claude/kb/milestone/02-second-txn.en.md` | `8e24ab7797f6a369…` | `83db520bdafe02cb…` |
| `.claude/kb/milestone/02-second-txn.ja.md` | `3cff0ab200b2367f…` | `6eed644a7938bd85…` |
| `.claude/kb/milestone/02-second-txn.md` | `bfb6a5ce937f87ed…` | `387cda9d37542e26…` |
| `.claude/kb/milestone/03-third-txn.en.md` | `7a84d93b4ab4d45e…` | `b9948809896c8265…` |
| `.claude/kb/milestone/03-third-txn.ja.md` | `41bc7d8e416fa819…` | `61ff1019c2dbd4af…` |
| `.claude/kb/milestone/03-third-txn.md` | `9f523ed55e805ed1…` | `fa99871399a8feec…` |
| `.claude/kb/milestone/04-fourth-txn.en.md` | `1005571fad1a16bd…` | `e25fdded535e659b…` |
| `.claude/kb/milestone/04-fourth-txn.ja.md` | `96d6424acb8d4f7e…` | `bca42c2f4236d32d…` |
| `.claude/kb/milestone/04-fourth-txn.md` | `93928f4f080b8445…` | `11c4fdf084b5d388…` |
| `.claude/kb/milestone/05-fifth-txn.en.md` | `e76c7c47b48f367b…` | `050b38cb3ae658b6…` |
| `.claude/kb/milestone/05-fifth-txn.ja.md` | `5586dd98d87bd9c9…` | `0bcc6d4e621479c8…` |
| `.claude/kb/milestone/05-fifth-txn.md` | `ee461ab4ad566526…` | `97ab694bfeaedc25…` |
| `.claude/kb/milestone/08-eighth-txn.en.md` | `9ee40ff715d3e0fa…` | `4afdfc3a2546f6cd…` |
| `.claude/kb/milestone/08-eighth-txn.ja.md` | `6ed960adb6a947a4…` | `74ccd3be59e312e4…` |
| `.claude/kb/milestone/08-eighth-txn.md` | `28d45ca9bcc6f841…` | `3aac315860abbf3d…` |
| `.claude/kb/milestone/09-ninth-txn.en.md` | `22eca502a375f5cb…` | `afdf2759586d3bc0…` |
| `.claude/kb/milestone/09-ninth-txn.ja.md` | `53ca530466656739…` | `7a513b8fc6fe714d…` |
| `.claude/kb/milestone/09-ninth-txn.md` | `abf7d16952e90066…` | `0cbef4c4ae38b28c…` |
