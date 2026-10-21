# 翻译术语表 / Translation glossary / 翻訳用語集

**仓根 README、`briefs/` 与 `.claude/kb/` 的三语版本照这张对照表译。** 登记在 `.claude/trilingual-documents` 的文件，中文是默认版本（`<名>.md`），英语是 `<名>.en.md`，日语是 `<名>.ja.md`；改一份就要三份一起改。
新术语先加进对照表，再在译文里用。门禁 doc-text 的 trilingual-glossary 格判：中文列的串出现在中文那份里，英语列的串（不分大小写）就要出现在英语那份里，日语列的串就要出现在日语那份里。
说明列可以空；不空就写三段，用 `<br>` 隔开，依次是中文、英语、日语。
编号的简称照登记位的中文原样写在中文列；英语与日语版里编号后面用半角括号写译名，写成 `E<编号> (<译名>)`，不用全角括号：全角括号会被「编号与简称」那一道当成中文简称去比。

> The trilingual versions of the root README, of `briefs/` and of `.claude/kb/` are translated with this table. For every file registered in `.claude/trilingual-documents`, Chinese is the default (`<name>.md`), English is `<name>.en.md` and Japanese is `<name>.ja.md`; a change to one is a change to all three.
> Add a new term here first, then use it in the translations. The trilingual-glossary cell of gate doc-text checks that whenever the Chinese string occurs in the Chinese version, the English string (case-insensitive) occurs in the English version and the Japanese string occurs in the Japanese version.
> In the English and Japanese versions, a numbered item is followed by its translated short name in ASCII parentheses, as in `E<number> (<translated name>)`.

> ルート README、`briefs/` と `.claude/kb/` の三言語版はこの対訳表に従って訳す。`.claude/trilingual-documents` に登録されたファイルは中国語が既定版（`<名>.md`）、英語が `<名>.en.md`、日本語が `<名>.ja.md` で、一つを変えたら三つとも変える。
> 新しい用語はまずここに加え、それから訳文で使う。ゲート doc-text の trilingual-glossary セルは、中国語列の文字列が中国語版に現れたら、英語列の文字列（大文字小文字を区別しない）が英語版に、日本語列の文字列が日本語版に現れることを検査する。
> 英語版と日本語版では、番号の後ろに訳した略称を半角括弧で `E<番号> (<訳名>)` のように書く。

## 术语 / Terms / 用語

<!-- trilingual-glossary -->
| 中文 | English | 日本語 | 说明 / Note / 注 |
|---|---|---|---|
| 写时复制 | copy-on-write | コピーオンライト | |
| 文件系统 | file system | ファイルシステム | |
| 门禁 | gate | ゲート | 自动化准入检查的总称<br>umbrella term for the automated acceptance checks<br>自動受入検査の総称 |
| 准入 | acceptance | 受入 | |
| 证据 | evidence | 根拠 | |
| 口径 | measurement basis | 計測条件 | 一个数是怎么量出来的<br>how a number was measured<br>その数値がどう測られたか |
| 治理 | governance | ガバナンス | |
| 里程碑 | milestone | マイルストーン | |
| 简报 | brief | ブリーフ | `briefs/` 下按日期一份<br>one per date under `briefs/`<br>`briefs/` の下に日付ごとに一つ |
| 决策 | decision | 決定 | |
| 已定项 | settled item | 確定項目 | 决策里定下来的一条分项<br>a settled sub-item of a decision<br>決定のうち確定した小項目 |
| 实验 | experiment | 実験 | |
| 不变量 | invariant | 不変条件 | |
| 欠账 | owed check | 未済の検査 | 知道要拦什么但还拦不了的检查<br>a check we know we need but cannot enforce yet<br>何を止めるべきか分かっているがまだ止められない検査 |
| 新池新建文件 | new-pool file creation | 新規プールでのファイル作成 | 里程碑名，也是一条写流<br>a milestone name and a write stream<br>マイルストーン名であり、書き込みストリームの名前でもある |
| 覆盖写、释放、回退与复用 | overwrite, release, rollback and reuse | 上書き・解放・ロールバック・再利用 | 里程碑名<br>a milestone name<br>マイルストーン名 |
| 删除与多次 COW | deletion and repeated COW | 削除と複数回の COW | 里程碑名<br>a milestone name<br>マイルストーン名 |
| 共享与回收 | sharing and reclamation | 共有と回収 | 里程碑名<br>a milestone name<br>マイルストーン名 |
| 覆盖写 | overwrite | 上書き | |
| 释放 | release | 解放 | |
| 回退 | rollback | ロールバック | |
| 回退下界 | rollback floor | ロールバック下限 | 简称 F<br>called F for short<br>略して F |
| 管理员回退 | administrator rollback | 管理者ロールバック | |
| 复用 | reuse | 再利用 | |
| 续跑 | resume | 再開 | |
| 发布 | publish | パブリッシュ | |
| 多记录发布 | multi-record publish | マルチレコードパブリッシュ | |
| 落点 | placement | 配置先 | 一个单元在盘上落的位置<br>where a unit lands on disk<br>ユニットがディスク上に置かれる位置 |
| 系统配置 | system configuration | システム構成 | 每盘偏移 0 与 4096 两个槽里的那块结构<br>the structure in the two slots at offsets 0 and 4096 of each disk<br>各ディスクのオフセット 0 と 4096 の二つのスロットにある構造 |
| 根环 | root ring | ルートリング | |
| 根记录 | root record | ルートレコード | |
| 树表 | tree table | ツリーテーブル | |
| 索引树 | index tree | インデックスツリー | |
| 分配记录树 | allocation record tree | 割り当てレコードツリー | |
| 数据单元 | data unit | データユニット | |
| 单元 | unit | ユニット | |
| 自描述 | self-describing | 自己記述 | |
| 校验和 | checksum | チェックサム | |
| 补齐 | padding | パディング | |
| 位置条目 | position entry | 位置エントリ | |
| 撕裂写 | torn write | 断裂書き込み | |
| 屏障 | barrier | バリア | |
| 实例号 | instance number | インスタンス番号 | |
| 暖机 | warm-up | ウォームアップ | |
| 可写挂载 | writable mount | 書き込み可能マウント | |
| 冷启动 | cold start | コールドスタート | |
| 恢复重放 | recovery replay | リカバリ再生 | |
| 择根 | root selection | ルート選択 | |
| 扫环 | ring scan | リングスキャン | |
| 前缀 | prefix | プレフィックス | |
| 固定脚本 | fixed script | 固定スクリプト | |
| 随机历史 | random history | ランダム履歴 | |
| 故障注入 | fault injection | 故障注入 | |
| 崩溃注入 | crash injection | クラッシュ注入 | |
| 崩溃点重放 | crash-point replay | クラッシュポイント再生 | |
| 崩溃放量 | crash amplification | クラッシュ増幅 | |
| 崩溃一致性 | crash consistency | クラッシュ整合性 | |
| 崩溃枚举用例 | crash enumeration case | クラッシュ列挙ケース | |
| 展开上限 | expansion limit | 展開上限 | |
| 违例 | violation | 違反 | |
| 池级 checker | pool-level checker | プールレベル checker | 判一个镜像的判决器库<br>the library that judges one image<br>一つのイメージを判定するライブラリ |
| 记录核对器 | record checker | レコード照合器 | |
| 判器 | judge | 判定器 | |
| 判定库 | verdict library | 判定ライブラリ | |
| 判法单元指纹 | judging-unit fingerprint | 判定法ユニット指紋 | |
| 观察者 | observer | オブザーバー | |
| 必红流 | must-red stream | 必赤ストリーム | 改坏一个字段、必须红在指定不变量上的流<br>a stream with one field corrupted that must go red on a named invariant<br>一つのフィールドを壊し、指定の不変条件で赤にならなければならないストリーム |
| 对拍 | differential | 突き合わせ | 两条互不共享代码的路子算同一件事再逐项比<br>two code paths that share no code compute the same thing and are compared item by item<br>コードを共有しない二つの経路で同じものを計算し、項目ごとに比べる |
| 理想模型 | ideal model | 理想モデル | |
| 形式化验证 | formal verification | 形式検証 | |
| 广谱变异 | broad mutation | 広域ミューテーション | |
| 变异表 | mutation table | ミューテーション表 | |
| harness 档 | harness tier | harness 層 | |
| checker 档 | checker tier | checker 層 | |
| 快档 | quick tier | クイック層 | |
| 慢档用例 | slow-tier case | スロー層ケース | |
| 全量 | full run | フル実行 | |
| 剪枝 | pruning | 枝刈り | |
| 轻型 | light tier | ライト層 | |
| 重型 | heavy tier | ヘビー層 | |
| 本次未跑 | not run this time | 今回未実行 | |
| 清场 | quiesce | 静止化 | 起 GPU 之前停掉占卡的服务<br>stop the services holding the GPUs before a GPU run<br>GPU を使う前に GPU を占有しているサービスを止める |
| 显卡 | GPU card | GPU カード | |
| 内核 | kernel | カーネル | |
| 派活 | dispatch | ディスパッチ | |
| 双机 | two-host | 二台構成 | |
| 第二台 | second host | 二台目 | |
| 分项 | item | 項目 | |
| 合表 | merged table | 統合表 | |
| 登记表 | registry | 登録表 | |
| 钉表 | pin table | 固定表 | |
| 产物 | artifact | 成果物 | |
| 弄坏开关 | breakage switch | 破壊スイッチ | |
| 主 agent | main agent | メイン agent | |
| 实现员 | implementation writer | 実装担当 | |
| 合入后验证 | post-merge verification | マージ後検証 | |
| 验证路线 | verification route | 検証ルート | D13（验证路线） 的简称<br>short name of that decision<br>その決定の略称 |
| 小数据打包容器 | small-data packing container | 小データパッキングコンテナ | D27（小数据打包容器） 的简称<br>short name of that decision<br>その決定の略称 |
| 补齐之后小文件怎么落 | how small files land after padding | パディング後に小ファイルがどう配置されるか | E66（补齐之后小文件怎么落） 的简称<br>short name of that experiment<br>その実験の略称 |
| 分配记录的条目数与整理开销 | allocation record entry count and compaction cost | 割り当てレコードのエントリ数と整理コスト | E74（分配记录的条目数与整理开销） 的简称<br>short name of that experiment<br>その実験の略称 |
| 档表取等比时公比怎么定 | choosing the common ratio of a geometric size-class table | 等比のサイズクラス表で公比をどう決めるか | E120（档表取等比时公比怎么定） 的简称<br>short name of that experiment<br>その実験の略称 |
| 按里程碑对比六家文件系统的文件性能 | per-milestone file performance against six file systems | マイルストーン別の六ファイルシステムとのファイル性能比較 | E152（按里程碑对比六家文件系统的文件性能） 的简称<br>short name of that experiment<br>その実験の略称 |
| 恢复重放有界穷举与变异杀伤界 | bounded exhaustive recovery replay and mutation kill bound | リカバリ再生の有界全数探索とミューテーション殺傷限界 | E165（恢复重放有界穷举与变异杀伤界） 的简称<br>short name of that experiment<br>その実験の略称 |
| 吞写之后可写挂载被拒而模型说该成 | writable mount refused after a swallowed write while the model says it should succeed | 書き込み消失後に書き込み可能マウントが拒否されモデルは成功すべきとする | C600（吞写之后可写挂载被拒而模型说该成） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| fsid 前 8 字节相同时重做 mkfs 实例代号不变式红 | instance generation invariant red when mkfs is redone with the same first 8 bytes of fsid | fsid の先頭 8 バイトが同じまま mkfs をやり直すとインスタンスコード不変条件が赤 | C659（fsid 前 8 字节相同时重做 mkfs 实例代号不变式红） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 钉槽的两类坏实现镜像上判不出 | two kinds of bad slot-pin implementations not judgeable on images | スロット固定の二種の不良実装がイメージ上で判定できない | C863（钉槽的两类坏实现镜像上判不出） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 树表条目未知 flags 位让整张树表读不下去 | unknown flags bits in a tree table entry make the whole tree table unreadable | ツリーテーブルエントリの未知 flags ビットでツリーテーブル全体が読めなくなる | C865（树表条目未知 flags 位让整张树表读不下去） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| F2 退回不分字节丢了与内容拒收 | F2 rollback does not tell lost bytes from rejected content | F2 のロールバックがバイト消失と内容拒否を区別しない | C866（F2 退回不分字节丢了与内容拒收） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 别的自证结构的读错位没人查 | misplaced reads of other self-verifying structures go unchecked | 他の自己検証構造の位置ずれ読み出しを誰も検査していない | C864（别的自证结构的读错位没人查） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 另外六处读者只拒窄于字段表的条目宽 | six other readers reject only entry widths narrower than the field table | 他の六箇所の読者がフィールド表より狭いエントリ幅だけを拒否 | C867（另外六处读者只拒窄于字段表的条目宽） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 系统配置管道身份没有 checker 判定 | system configuration pipeline identity has no checker verdict | システム構成のパイプライン同一性に checker 判定がない | C868（系统配置管道身份没有 checker 判定） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 重放前缀严格连续没有会红的判定 | strictly contiguous replay prefix has no must-red verdict | 再生プレフィックスの厳密な連続性に赤になる判定がない | C869（重放前缀严格连续没有会红的判定） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 提交原子没有崩溃点重放 oracle | commit atomicity has no crash-point replay oracle | コミットの原子性にクラッシュポイント再生の oracle がない | C870（提交原子没有崩溃点重放 oracle） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 记录 inode 号等于 key 没有 checker 判定 | record inode number equal to key has no checker verdict | レコードの inode 番号が key と等しいことに checker 判定がない | C871（记录 inode 号等于 key 没有 checker 判定） 的简称<br>short name of that owed check<br>その未済の検査の略称 |
| 空间分配 | space allocation | 領域割り当て | D3（空间分配） 的简称<br>short name of that decision<br>その決定の略称 |
| 核心索引结构 | core index structure | コアインデックス構造 | D8（核心索引结构） 的简称<br>short name of that decision<br>その決定の略称 |
| 加密 | encryption | 暗号化 | D9（加密） 的简称<br>short name of that decision<br>その決定の略称 |
| 格式冻结政策 | format freeze policy | フォーマット凍結ポリシー | D15（格式冻结政策） 的简称<br>short name of that decision<br>その決定の略称 |
| 发布语义 | publish semantics | パブリッシュセマンティクス | D16（发布语义） 的简称<br>short name of that decision<br>その決定の略称 |
| 块里携带什么信息 | information carried in a block | ブロックに含める情報 | D18（块里携带什么信息） 的简称<br>short name of that decision<br>その決定の略称 |
| 后台整理与放置回收 | background compaction and placement reclamation | バックグラウンド整理と配置回収 | D26（后台整理与放置回收） 的简称<br>short name of that decision<br>その決定の略称 |
| 单元键 | unit key | ユニットキー | 判定按它复用：函数单元指纹、工具链、依赖版本<br>the key a verdict is reused by: function-unit fingerprints, toolchain, dependency versions<br>判定の再利用の鍵: 関数ユニットの指紋、ツールチェーン、依存の版 |
| 函数单元 | function unit | 関数ユニット | |
| 块键 | block key | ブロックキー | |
| 录入段 | recording stage | 記録段 | |
| 判段 | judging stage | 判定段 | |
| 读集见证 | read-set witness | 読み出し集合の証人 | |
| 恢复报告分桶 | recovery-report bucketing | リカバリ報告のバケット分け | |
| 判表 | judge tables | 判定表 | |
| 显存账 | GPU memory ledger | GPU メモリ台帳 | |
| 预期红 | expected red | 想定内の赤 | |
