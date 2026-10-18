# 翻译术语表 / Translation glossary / 翻訳用語集

**仓根 README 与 `briefs/` 的三语版本照这张对照表译。** 登记在 `.claude/trilingual-documents` 的文件，英语是默认版本（`<名>.md`），中文是 `<名>.zh.md`，日语是 `<名>.ja.md`；改一份就要三份一起改。
新术语先加进对照表，再在译文里用。门禁 doc-text 的 trilingual-glossary 格判：中文列的串出现在中文那份里，英语列的串（不分大小写）就要出现在英语那份里，日语列的串就要出现在日语那份里。
说明列可以空；不空就写三段，用 `<br>` 隔开，依次是中文、英语、日语。
编号的简称照登记位的中文原样写在中文列；英语与日语版里编号后面用半角括号写译名，写成 `E<编号> (<译名>)`，不用全角括号：全角括号会被「编号与简称」那一道当成中文简称去比。

> The trilingual versions of the root README and of `briefs/` are translated with this table. For every file registered in `.claude/trilingual-documents`, English is the default (`<name>.md`), Chinese is `<name>.zh.md` and Japanese is `<name>.ja.md`; a change to one is a change to all three.
> Add a new term here first, then use it in the translations. The trilingual-glossary cell of gate doc-text checks that whenever the Chinese string occurs in the Chinese version, the English string (case-insensitive) occurs in the English version and the Japanese string occurs in the Japanese version.
> In the English and Japanese versions, a numbered item is followed by its translated short name in ASCII parentheses, as in `E<number> (<translated name>)`.

> ルート README と `briefs/` の三言語版はこの対訳表に従って訳す。`.claude/trilingual-documents` に登録されたファイルは英語が既定版（`<名>.md`）、中国語が `<名>.zh.md`、日本語が `<名>.ja.md` で、一つを変えたら三つとも変える。
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
