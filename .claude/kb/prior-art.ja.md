# 他社方式の調査

[中文](prior-art.md) · [English](prior-art.en.md) · **日本語**

**すべてドキュメント精読またはソースコードの行単位の突合によるものであり、本プロジェクト内では未検証です**。また、いかなるプロジェクトもコンパイル・実行していません。
スター数/コミット数は調査時点の表示値です（2026-08-25）。

**参照元の固定点と再検証メカニズム**: 重大な引用元となるソースコードと文献はローカルマシン上の `~/code/fs-refs/` に固定されています——`linux-6.17/fs/bcachefs/`（kernel.org の `linux-6.17.tar.xz`、260ファイル）、`zfs/`（OpenZFS master、`git log -1` = `58d73c9`、2026-08-27）、`docs/` 配下の7件の文献（RFC 8439、NIST SP 800-38B / 800-38D、DJB *Cryptography in NaCl*、OSTEP 第45章、BetrFS FAST 2017 / 2018、NetApp FAST'20；取得方法と sha256 は `scripts/kb/fetch-refs.sh`、抽出スクリプトは `scripts/kb/pdf-text.py` を参照）。
**89件**の重要な引用は再実行可能な一語一語のアサーション（`scripts/kb/verify-citations.sh`）として仕組まれています（この件数はスクリプト自身が末尾行で報告する `bash scripts/kb/verify-citations.sh` の「結果：N件ヒット」によるものであり、手作業で数えたものではありません。ソース内の `^(ck|ckn|ckdoc|ckdocn)` 行には `--selftest` 内のみで走る11件の合成アサーションも含まれます）。そのうち38件は固定点ツリーではなくローカルのカーネルツリーを検証しており（C44）、ゲート検証段階「外部引用の検証が依然として機能するか」（`.claude/gate.d/doc-registries.sh` の citations スロット）に組み込まれています——**ソースツリーが存在しない場合も赤（失敗）判定**となります。「スキップ」こそが前回の文献群を音もなく蒸発させた原因です。
⚠️ **ZFS On-Disk Specification は除外**: CID 2バイトフォントを使用しているため `pdf-text.py` で文字化けが発生します ⇒ スクリプトの規則に従い**抽出失敗**として扱い、「原文にその記述が存在しない」とは扱いません。これを引用する箇所は固定済みの OpenZFS ソースツリーから検証するように変更しました。
⚠️ どの引用が固定点ツリー以外を検証しているかについては、[checks-owed.md](checks-owed.md) C44 を参照してください。

## 1. 会計（アカウンティング）モデルの4つの選択肢

| モデル | 採用例 | メカニズム | コスト |
|---|---|---|---|
| extent refcount + 逆インデックス走査 | btrfs qgroup | shared/exclusive を正確に計算 | コミット時間 +76%、トランザクション待機 +1347%、書き込みスループット −25% |
| 元の作成者に帰属、グローバル refcount なし | btrfs squota | OWNER_REF インライン、extent ライフサイクルに 1:1 で紐づけ、逆インデックス不要 | クォータ無効時と比較してわずか 3.4% 低下；shared/exclusive は計算不可 |
| birth txg + deadlist | ZFS | ブロック自身が誕生世代番号を保持；削除時に「1つ古いスナップショット」と世代を比較 → 直接解放、またはそのスナップショットの deadlist に追加；スナップショット破棄時に deadlist をカスケードマージ | 会計は増分更新、全走査は一切不要 |
| 専用 btree に accounting key を配置 | bcachefs（2024年再設計） | カウンタ自体が bkey（tagged union で拡張可能）；メモリ側は percpu + eytzinger インデックス；更新は通常のトランザクション + write buffer 経由 | この再設計はディスクフォーマット互換性を破綻させ、6.11 で強制アップグレード |

**数値の計測条件**: qgroup / squota の割合は btrfs 公式が LWN の squota 記事で提示した比較テストによるものです。ワークロードとハードウェアの詳細は完全には開示されていないため、**本プロジェクトの性能予測値として扱うことはできず**、「この路線は行き詰まる」という定性的証拠としてのみ参照します。

**重要な知見: btrfs 自身が白旗を上げたこと。** squota の存在は、厳密な会計が維持困難であることを公式に認めた証拠です。この路線が破綻することは、本プロジェクトが改めて追試する必要はありません。

**3者の勝敗の分かれ目は同一の点にある: 会計がトランザクションの副産物であるか、事後の走査であるか。**
btrfs qgroup の事後走査 → 破綻；ZFS および再設計後 bcachefs のトランザクション時増分更新 → 存続。

参照元:
- [btrfs: simple quotas (LWN)](https://lwn.net/Articles/944371/)
- [bcachefs disk accounting rewrite (LWN)](https://lwn.net/Articles/963570/)
- [Advanced ZFS Dataset Management — Klara](https://klarasystems.com/articles/advanced-zfs-dataset-management/)
- [Disk Space Accounting for ZFS Snapshots — Oracle](https://docs.oracle.com/cd/E19253-01/819-5461/gbcxc/index.html)

## 2. bcachefs で研究に値する論点

参照元: [bcachefs Snapshots 設計ドキュメント](https://bcachefs.org/Snapshots/)、
[Principles of Operation (2026-04-16, PDF)](https://bcachefs.org/bcachefs-principles-of-operation.pdf)。
**Principles of Operation は最新かつ最も権威ある bcachefs 設計ドキュメントですが、完全通読は未了です。**

1. **bucket + generation 番号**。ディスクを bucket に分割し、各 bucket に世代番号を付与；ポインタは「期待される世代番号」を保持し、世代が一致しなければ解放済みとみなす → データの無効化において逆インデックスを更新する必要がない。D1 (データ移動性 / 逆インデックス)の負担を直接軽減。
2. **キー下位ビットへのスナップショット ID 組み込み**。スナップショット ID 自体がツリー構造を成し（ルート `U32_MAX`、下向きに割り当て、親 ID は常に子 ID より大きい）、探索時に祖先関係でフィルタリング。**ツリーのクローンは一切不要**で、O(1) 作成、百万単位のスナップショットに対応可能。
   代償: スナップショット削除時にスナップショット付きの全 btree を走査する必要がある（公式は bloom filter による最適化を検討と記載）；読み取りパスに一定の祖先フィルタリングオーバーヘッドが発生；書き込みパスで whiteout やスナップショット跨ぎの部分上書き分割を処理する必要がある。
3. **ポインタ内インライン複数レプリカ**。1つの extent ポインタ内に N 個のレプリカ位置を直接列挙。ファイル単位でレプリカ数を設定可能——btrfs の per-chunk profile では不可能。
4. **B-tree ノード内部のログ構造化**（大きなノード、小さな更新を追記、読み取り時マージ、バックグラウンドコンパクション）。
5. **copygc / rebalance が常駐バックグラウンドプロセス**であり、管理者が手動実行するコマンドではない。
6. **逆インデックスが後付けであったこと**——これは教訓です: 後から追加することがいかに困難であるかを証明しています。

**エンジニアリングの進行ペースを真似てはならない**: bcachefs のディスクフォーマットは安定するまでに長年を要し、2025年にカーネルメインラインのメンテナンスから外されました。技術は参考にできますが、開発ペースは参考にできません。

## 3. Rust 側の既存実装

| プロジェクト | 言語 | 到達点 | 状態 |
|---|---|---|---|
| [rustutils/btrfsutils](https://github.com/rustutils/btrfsutils) | Rust | disk（RAID5/6 Reed-Solomon含む）/ transaction（書き込みパス）/ fuse（読み取り専用マウント）/ stream / mkfs / tune / cli；btrfs check 7段階、rescue chunk-recover 実装済み | 0.13.0（2026-05-14）。13 stars、464 commits、**0 open issues、20 open PRs**。テストは unit + cargo insta スナップショット、**ページ上に fuzz / xfstests / 差分テストは見当たらない**。transaction には「大切なファイルシステムでは使用しないでください」との警告あり |
| [adam900710/btrfs-fuse](https://github.com/adam900710/btrfs-fuse) | C | 読み取り専用 FUSE、全 RAID(0/1/10/5/6)、全圧縮、4種チェックサム | 作者は btrfs カーネルメンテナー |
| [btrfs-rec](https://www.lukeshu.com/blog/btrfs-rec.html) | Go | 読み取り専用 + 破損ツリー/chunk マッピング再構築 | リカバリ分野で最も強力なオープンソース実装 |
| [btrfscue](https://github.com/cblichmann/btrfscue) | Go | 全 superblock 上書き時のデータサルベージ | 古いプロジェクト |
| [GodTamIt/btrfs-diskformat](https://github.com/GodTamIt/btrfs-diskformat) | Rust | 構造体定義のみ、ディスク読み込み不可 | 22 commits、停滞 |
| [Dicklesworthstone/frankenfs](https://github.com/Dicklesworthstone/frankenfs) | Rust | ext4+btrfs 再実装を主張、22 crates、zero unsafe | 21 stars / **8213 commits** / 0 issues。**この比率はエージェントによる一括生成を強く示唆**——推測であり、コード未読 |

**本プロジェクトへの意義**: btrfsutils は「btrfs フォーマットに沿った Rust 実装」が3ヶ月で雛形を作れることを示していますが、それは**フォーマットが既知であり、`btrfs inspect-internal dump-tree` が無償の参照オラクルとして機能するから**です。
本プロジェクトはゼロからの設計であり、そのいずれもありません——**オラクルが存在しないことがゼロから構築する最大の隠れコスト**であり、予算に組み込む必要があります（`.claude/singlefs-ai-sop/rules/test-discipline.md` の「モデル突き合わせ」参照）。

## 4. カーネル Rust ファイルシステム抽象化レイヤー

2026年時点で、Rust は Linux のドライバ、ファイルシステム、ネットワークコンポーネントに導入されています。fs 抽象化レイヤーは依然として進化途上であり、「汎用的なものか、単純なファイルシステム専用か」についての合意には達していません。

参照元: [Rust for filesystems (LWN)](https://lwn.net/Articles/978738/)。
**本プロジェクトとの関連性は希薄**——D7 (Linux メインラインへのマージ是非)において、初期の数年間はメインラインに入らない方針が確定しています。

## 5. インデックス構造（D8 (コアインデックス構造) 専門調査）

**計測条件**: 「5. インデックス構造」の節は bcachefs *Principles of Operation* Rev 1.39.2（2026-08-25 改訂版）原文、LWN、XFS ドキュメントおよび論文要約に基づきます。**本プロジェクト内での検証は未実施であり、いずれの実装もクローン/コンパイル/実行は行っていません。**

### 5.1 bcachefs: 28本のツリー、単一の btree 実装

> "The entire filesystem is built on two primitives: btrees and buckets.
> All filesystem state is a key-value pair in a btree. ... There are no separate
> inode tables, bitmap allocators, or per-inode extent trees."

「アクセスパターンに応じてデータ構造を切り替える」のではなく、**28個の独立した keyspace に分割し、同一の btree 実装を共有**した上で、扱いづらい2種類のアクセスパターンを2つのフロントエンドで解決しています:

| フロントエンド | 解決する課題 | 適用先 |
|---|---|---|
| **key cache** | 同一キー群の反復的な読み書き、毎回の全ツリー検索は高コスト | alloc btree（extent 更新時に所属 bucket の割り当て情報を修正） |
| **write buffer** | 高頻度の小さな更新、バッチ適用のほうが有利 | backpointers / LRU / accounting |

28本のツリーのうち **10本が write-buffered** です: `lru`、`need_discard`、`backpointers`、`deleted_inodes`、`accounting`、`stripe_backpointers`、4本の `reconcile_*`。
⚠️ 28 / 10 は Principles of Operation Rev 1.39.2 ドキュメント内の数値です。2026-08-29 に linux-6.17 ソースの `BCH_BTREE_IDS()` を確認したところ 21本、write buffer 経由は 5本でした。D8 (コアインデックス構造) 本文の記載を正とします。

**write buffer の代償（原文に明記）**: 更新は**順不同かつ結果整合性**です——btree は次回の flush まで未処理の更新を認識できません；flush はキー位置順にソートされ、**時間順序は破棄**されます；重複排除後に btree ノード順にバッチ挿入されます。
> "this sort-merge-sweep is inherently single-threaded, making its efficiency
> critical for multithreaded workloads."

すなわち: write buffer は書き込み増幅を抑制しますが、**引き換えにシングルスレッドの flush ボトルネックを抱え込みます**。

### 5.2 ログ構造化された btree ノード

ノードサイズは 128K–256K（デフォルト 256K）で、**内部はログ構造化**されています: 新しいキーは追記され、ソート挿入は行われません。
1つのノードは複数の bset を含みます——前回の全量書き換えで残ったものに加え、その後に追記されたいくつかの bset です。
検索時はすべての bset の結果をマージする必要があります；メモリ内で定期的に並べ替えを行い、1つのノードが**最大約3個の bset** に収まるようにします（書き込み中の bset は ≤ 8×16K、残りはほぼ等比数列）。

注目すべき3つの特性:

1. **書き込みがシーケンシャルである**（SSD および HDD の双方に好適）。
2. **ノードを読み取り専用ロック下でディスクに書き出せる**——追記のみであるため。これによりロック保持時間が極めて短くなり、ライトバックがリーダーをブロックしません。**これは単なる IO 面だけでなく、並行性における大きな実益です。**
3. **削除は whiteout（墓石の挿入）に頼るしかない**——すでにディスクに書き込まれた bset 内からは削除できません。これはログ構造化ノードの必然的帰結であり、削除に関わるすべてのコードに波及します。

キーは bset 内で**パック化**されます: 各 bset は全フィールドで共通の値を記録するフォーマット記述子を持ち、圧縮格納することで**メタデータ空間を 30–50% 節約**します。

**他社が大きなノードを採用できない理由**: 他の COW ファイルシステムはメタデータを Linux の page cache に保存するため、4K ノードに制限されます。bcachefs は btree ノードキャッシュを独自管理しているため、128K–256K を採用し、回収を独立して調整できます。
> "At petabyte scale with spinning disks, deep btrees with 4K nodes mean more
> seeks per lookup and level-2 nodes too large to stay resident under page
> cache pressure."

**本プロジェクトへの意義**: D7 (Linux メインラインへのマージ是非)でメインラインに入らないことが確定しているため、**この制約は当方には一切存在しません**——大きなノードは無償で享受でき、「キャッシュの独自管理」以外の代償を支払う必要はありません。

### 5.3 bucket アロケータ: 走査スレッドから btree への移行

- bucket サイズは 512K–2M；bucket とデータポインタの双方が **generation 番号** を保持し、世代番号をインクリメントするだけで古いポインタをすべて無効化できます。**逆参照を探して削除しに行く必要はありません**。
- 原文では世代番号メカニズムの価値はキャッシュ無効化にとどまらないと説明されています: ジャーナルリプレイ時の**割り当て情報の自己ブートストラップ**を可能にし、**本来修復不可能な破損を修復可能にします**。← これは当方の「再構築性」という設計目標に直結します。
- bucket 内は**絶対に上書きされず**、シーケンシャルに満杯になったら凍結されます → copying GC が必須。**copyGC の予約領域は 8%（デフォルト、5%–20% で設定可能）**——これは D3 (領域割り当て) の「過大な予約は不可」に対する格好の参照値です。
- bucket の充填度を記録する **fragmentation LRU btree** が存在し、copygc は全 alloc btree を走査することなく最も断片化した bucket を直接特定できます。
- **アロケータは当初、メモリ上の bucket 配列 + 専用走査スレッド** で freelist/discard/eviction を管理していましたが、後にすべて btree（freespace / discard / LRU の3本）に置き換えられ、**走査スレッドは完全に削除**されました。
  原文の理由: トランザクショナルな btree コードは「far easier to debug and reason about」であり、古いスレッドはファイルシステムが満杯に近づいた際に**膨大な CPU を浪費**していました——btree への移行によりこれらコーナーケースが解消しました。
- bucket は SMR / zoned デバイスの zone に自然にマッピングされます。

### 5.4 会計（D5 (スナップショット / 空間会計メカニズム)に対する独立した裏付け）

> "Accounting counters are maintained by triggers as keys are committed,
> not by walking the btrees."

一方、`check_allocations` は**全走査による会計の再構築**に依存し、「カウンタの陳腐化」と「キーが真に存在するか」を識別します。
これは `crates/singlefs-checker/invariants/05-空间记账.md` の I-3.1 (割り当て済み統計が一致する)と同一の設計です: **実行時は増分維持、チェッカーは走査で検証**。

### 5.5 イレースコーディング: write hole に対する第2の既知の解法

bcachefs はフォアグラウンドの書き込みをストライプに分割するのではなく、**バックグラウンドで bucket 全体を符号化**します:
フォアグラウンドは通常の複数レプリカ書き込み → バックグラウンドで十分な候補 bucket を蓄積 → Reed-Solomon を計算 → パリティを書き込み → 指し示す全 extent をアトミックに更新（余剰レプリカを破棄し、パリティポインタを追加）。

> "This approach avoids the write hole entirely: parity is computed once for
> immutable data, and the extent updates are atomic btree operations."

**これは ZFS RAID-Z の「可変幅ストライプ、毎回全幅書き込み」と並ぶ第2の優れた既知の回答法です**。
アプローチは全く異なります: 一方は部分ストライプ書き込みを存在させない手法、もう一方はパリティを不変データに対して一度だけ計算する手法です。
代償: ストライプ内に生存データが残っている限りその bucket は再利用できず、copygc はストライプ全体を退避させる必要があります。

### 5.6 Bε-tree / BetrFS: フルパスインデックスの名前変更コストは2018年に部分木深度まで削減された

Bε-tree は B-tree の変種であり、**内部ノードに ε 比率のメッセージバッファ領域を確保**し、書き込みをバッチ処理で下位へ償却しながらフラッシュすることで、書き込み I/O を理論上 B+tree より低く抑えます。BetrFS はこれを採用した最初のカーネルファイルシステムです。

BetrFS は**フルパスインデックス**（キーが完全なパスそのもの）を採用し、scan が極めて高速です。
名前変更（rename）は部分木内のすべてのキーのプレフィックスを置換する必要があり、**ナイーブな実装では部分木全体を書き直す必要があります**——FAST 2018 論文原文ではこれをフルパスインデックスの「アキレス腱」と呼んでいます。

**BetrFS 0.4（FAST 2018）は lifted Bε-tree を用いてこのコストを「部分木のサイズ」から「部分木の深度」に引き下げました。**
論文原文: key lifting の採用により、「the number of paths that need to be modified in a range rename also changes from being proportional to the size of the subtree to the depth of the subtree」；計算量の節ではさらに、tree surgery で分割またはマージされるノード数は「at most proportional to the height of the tree」と述べています。

メカニズムは2段階:

1. **tree surgery**——部分木の左右の境界にある「移動対象と非対象のキーが混在する」ノードを分割（通常のノード分割コードを再利用し、中央値ではなく分割キーで切る）、その後にポインタを移動。
2. **batched key update**——部分木内の全キーのプレフィックスを一括置換。このステップは **lifting 不変条件により無償で完了**します: lifted Bε-tree 内のキーの値は「そのノードへ到達するまでに辿ったパス」によって定義されるため、ポインタ移動が確定した時点で、部分木内の全キーのプレフィックスは暗黙的に置換されます。

**lifting がキーエンコーディングに課す要件**: 「同一部分木がプレフィックスを共有する」ことがツリー構造レベルで成立していなければなりません。BetrFS 0.4 はキーの比較に `memcmp` をそのまま使えるようキーフォーマットを調整しています。
また、「内部ノードの子供数を4〜16個に保つ」不変条件を維持し、ツリーの高さを抑える healing メカニズムも備えています。

**論文が自認する代償**: BetrFS 0.4 が 0.3 に対して悪化するケースは、**ノードの分割とマージがクリティカルパス上に位置する場合**であり、lifting による追加の計算オーバーヘッドが利益を上回ります；すべてのアクセスパターンのうち唯一悪影響を受けたのは**シーケンシャル書き込み**でした。
名前変更性能についての主張は「competitive with indirection-based file systems for a range of sizes」——**一定のサイズ範囲内で競争力があるということであり、全サイズで勝っているわけではありません**。

計測条件: FAST 2018 論文 + TOS 2018 拡張版、**本プロジェクト内では未検証**。
本プロジェクトのキーレイアウトへの影響は [decisions.md](decisions.md) D8 (コアインデックス構造) を参照。

参照元:
- [The Full Path to Full-Path Indexing (FAST 2018)](https://www.usenix.org/system/files/conference/fast18/fast18-zhan.pdf)
- [Efficient Directory Mutations in a Full-Path-Indexed File System (TOS 2018)](https://www.cs.unc.edu/~porter/pubs/tos18.pdf)

### 5.7 XFS 逆マッピング: コスト規模ともう一つの正当化根拠

rmapbt は各 AG ごとに1本の b+tree であり、`(物理ブロック, 所有者, オフセット, ブロック数)` を記録します。

**空間コスト（単一の情報源による基準、未再確認）**: 50TB のファイルシステムにおいて、素の `mkfs.xfs` は約 52GB のメタデータ（~1%）；`rmapbt=1` を有効化すると 675GB（~1.35%）に増加；さらに `reflink=1` を有効化すると 981GB（~1.96%）に達します。
⚠️ この数値は個人ブログの mkfs 出力によるものであり、**フォーマット時のメタデータ規模であって実際の使用量ではありません**。規模感の目安に留まります。

**さらに重要な点は、D1 (データ移動性 / 逆インデックス)に第2の正当化根拠を与えていることです**: XFS ドキュメントには、逆マッピングが「is an essential feature for repairing filesystems online because the secondary copy can be used to rebuild damaged primary metadata」と記載されています。

すなわち: **逆インデックスは単に balance/defrag のためだけではなく、オンライン修復を可能にするための前提条件です**——順方向インデックスが破損した際、逆方向インデックスを用いて再構築できます。これは避坑項目リスト第10条（誰も実行を恐れる修復ツール）に直結します。

参照元:
- [bcachefs: Principles of Operation, Rev 1.39.2 (2026-08-25)](https://bcachefs.org/bcachefs-principles-of-operation.pdf)
- [bcachefs Allocator](https://bcachefs.org/Allocator/)
- [xfs: add reverse mapping support (LWN)](https://lwn.net/Articles/695290/)
- [mkfs.xfs(8)](https://www.man7.org/linux/man-pages/man8/mkfs.xfs.8.html)
- [Notes on XFS: metadata size (k1024.org)](https://k1024.org/posts/2019/2019-02-08-notes-on-xfs/)
- [BetrFS: A Right-Optimized Write-Optimized File System (FAST'15)](https://www.usenix.org/system/files/conference/fast15/fast15-paper-jannen_william.pdf)
- [The Full Path to Full-Path Indexing (FAST'18)](https://oscarlab.github.io/papers/fast18-betrfs.pdf)
- [Closing the B-tree vs LSM-tree Write Amplification Gap (FAST'22)](https://arxiv.org/abs/2107.13987)

## 6. 暗号化（D9（暗号化） 専門調査）

**計測条件**: bcachefs Encryption 設計ドキュメント原文（最終編集 2025-11-02 と表記）、
OpenZFS `module/os/linux/zfs/zio_crypt.c` 先頭の設計コメント（master ブランチ、2026-08-26 取得）、
Linux fscrypt カーネルドキュメント、OpenZFS `zfs-load-key(8)` マニュアルページ、および OpenZFS の公開 issue から取得。
**すべてドキュメントおよびソースコードの精読によるものであり、本プロジェクト内では未検証であり、いかなる実装もコンパイル・実行していません。**

### 6.1 3種類の暗号化境界

| 境界 | 暗号化対象 | 改ざん検知 | 鍵なし時の可能操作 | 主な代償 |
|---|---|---|---|---|
| ブロック層（LUKS / dm-crypt） | デバイス全体 | **不可**（MAC 格納場所なし） | ファイルシステムの全機能が通常通り | ファイル/サブボリュームごとの鍵分離不可；MAC と nonce の配置場所なし |
| ファイル単位（fscrypt: ext4 / f2fs / UBIFS） | ファイル内容 + ファイル名 | **不可**（XTS / CBC-CTS / Adiantum 使用） | 全メタデータの読み取り可能 | サイズ、パーミッション、タイムスタンプ、xattr、ホールの位置が全平文 |
| ポインタ内インライン AEAD（ZFS / bcachefs） | 6.2 参照、両者で範囲が異なる | 可能、ポインタ内に MAC を格納 | 6.2 参照、両者で大幅に乖離 | フォーマットが最初から MAC / nonce フィールドを確保必須 |

bcachefs ドキュメントはブロック層暗号化を断罪する理由として構造的な一文を挙げています:
> "at the block level there's nowhere to store MACs or nonces without causing painful
> alignment problems."

fscrypt カーネルドキュメント自身も認証を行わない旨を明記しています:
> "fscrypt is not guaranteed to protect confidentiality or authenticity if an attacker
> is able to manipulate the filesystem offline prior to an authorized user later
> accessing the filesystem."
理由は「ciphertext expansion（暗号文拡張）の取り扱いが困難である」こと——すなわち暗号文が平文より長くなり、ブロックアライメントされたファイルシステムに収まらないためです。

**本プロジェクトへの意義**: チェックサムがポインタに追従する方式（[decisions.md](decisions.md) D4 (チェックサム配置位置)）自体が、MAC と nonce の格納領域を自然に確保しています。AEAD は本プロジェクトにとって実質無償で得られるものであり、ブロックアライメントされたファイルシステムとは前提が異なります。

### 6.2 ZFS と bcachefs が同一の問いで袂を分かつ: 鍵なしで保守が可能か

| | OpenZFS | bcachefs |
|---|---|---|
| 暗号化範囲 | **レベル0ブロックのみ暗号化**；間接ブロック（blkptr ツリー）は非暗号化 | システム設定を除きすべて暗号化 |
| 認証構造 | 間接層は SHA512(下層 MAC 集合) で集約；objset 層に2つの 256 ビット MAC（portable / local） | Poly1305 MAC をブロック単位で付与、システム設定まで連なる chain of trust |
| アルゴリズム | AES-CCM / AES-GCM（128/192/256ビット）；認証補助に SHA512-HMAC | ChaCha20 + Poly1305、RFC 7539 に沿って暗号プリミティブを直接使用 |
| MAC 格納場所 | blk_cksum の後方 128 ビット | メタデータ: 平文ヘッダ内の 128 ビットフィールド；データ: ポインタに追従 |
| Nonce 格納場所 | 96 ビット IV: 64 ビットは DVA[2] の第2ワード、32 ビットは blk_fill 上位 32 ビット | メタデータ: 平文ヘッダのシーケンス番号；データ: 鍵の 96 ビットバージョン番号 + 派生量（6.3 参照） |
| 鍵なし時の可能操作 | **scrub / resilver / 名前変更 / 削除 / raw send** | ほぼ皆無——メタデータ自体を復号しないと読めない |
| 漏洩する情報 | プール構造、データセット・スナップショット名、データセット属性、ファイルサイズ、ファイルホール、dedup テーブル | 「この位置が btree ノード/ログエントリであること」およびその長さのみ |

**ZFS が「鍵なし scrub 可能」のために支払った代償はソースコメントに率直に書かれています**:
`blk_cksum` の先頭 128 ビットは scrub 専用の**暗号文の切り詰めチェックサム**に割り当てられ、後方 128 ビットのみが MAC です。
また、MAC ツリーを Merkle ツリーにしなかった理由は、raw send が間接ブロックを転送せず、受信側がマスターキーを持たないと再計算できないためです:
> "Ideally, the cleanest solution would be to maintain a tree of authentication MACs
> going up the bp tree. However, this presents a problem for raw sends."
ZIL と dnode にもそれぞれ「平文で残さなければならない」部分が存在し（claiming や scrubbing で使用）、
これらの平文部分は AEAD の AAD メカニズムによって認証されます。

**ZFS の鍵階層**: マスターキーはデータを直接暗号化しません；64ビット salt + HKDF でワーキングキーを導出し、
**4億ブロック書き込みごとに salt を更新**します——コメントでは誕生日問題に基づき、96ビット IV 空間下で衝突確率を 1/10^12 未満に抑える上限が 398,065,730 ブロックと算出されています。マスターキーはユーザーのラッピングキーで暗号化されてディスクに保存されるため、ユーザーパスフレーズを変更してもデータの書き直しは不要です。

**bcachefs がディレクトリ単位暗号化を拒否した理由**（原文）: fsck にファイルサイズが必要なため、サイズを平文にする必要があり、さもなければ fsck のその部分を鍵提供後まで延期せざるを得なくなります；filenames、xattrs、inline extents にも同様の面倒が生じます。
> "With whole filesystem encryption, it's much easier to say what is and isn't encrypted."

**bcachefs の MAC 切り詰め論証**（セキュリティを実装上の制約に賭けている点で特筆に値します）:
データ extent の Poly1305 MAC はデフォルトで **80ビット** に切り詰められ、extent あたり 8バイトを節約します（単一レプリカ extent 32バイト vs 40バイト）。その論拠は、ファイルシステム攻撃者の偽造試行回数が有界であるという点にあります——MAC が不一致であればデバイス障害と判定され、直ちに停止されます:
> "attacker gets a very small (on the order of 10) attempts to forge a particular extent"
この前提を受け入れない環境（ネットワークアクセス経由や、データ破損を起こしやすいパスなど）では 128ビット MAC を使用する必要があります。

**アルゴリズムの切り替え可能性**: bcachefs は暗号化を「新しいチェックサム種別の一種」として設計しており、すべてのデータおよびメタデータ書き込みがアルゴリズムフィールドを保持するため、アルゴリズムの切り替えに古いデータの書き直しは不要です。

### 6.3 Nonce: 暗号化が COW フォーマットに突きつける真の要件

**これは bcachefs 設計ドキュメントの中で本プロジェクトにとって最も有益な節であり**、フォーマットへの追加を余儀なくされたフィールドの記録です。

- キーは 96ビットのバージョン番号を持ち、暗号化有効時は強制され、新規 extent ごとに新しい番号を取得します。
- extent が部分上書きまたは分割されて再書き込みされる際: **番号の変更は不可**（バージョン番号には別の意味論がある）、**番号の流用も不可**（書き込むデータが同一ではない）。
- 「元の extent 内の生存データ位置までキーストリームをスキップする」手法は圧縮と併用すると破綻します: 4K 捨ててから圧縮するのと、圧縮してから 4K 捨てる出力は一致しません。
- 最終的な解決策は、**extent に「元の extent 起点からのオフセット」フィールドを追加すること**であり、nonce はバージョン番号 + 当該オフセット + 現在のサイズ + 圧縮アルゴリズムから合成されます:
  > "offset + current size uniquely determines the uncompressed data, so, offset +
  > current size + compression function will uniquely determine the compressed output."

**クラッシュ後の nonce 再利用**: データは特定のバージョン番号で先にディスクに書き込まれ、その後に extent が btree に挿入されます。そのため不完全なシャットダウン後は「使用されたが btree 内で発見できない」番号が存在します。bcachefs ドキュメントによれば、厳密な解決策はジャーナルヘッダ内に水位（それ未満の番号は使用済みの可能性がある）を記録することであり、**現時点では未実装と注記されています**；現在の実装は既存の最大番号を走査し、次の番号をその 64k 上に設定しています。

### 6.4 暗号化で踏まれた落とし穴（公開記録）

| 落とし穴 | 記録 | 計測条件 |
|---|---|---|
| ZFS ネイティブ暗号化 + send/recv の長期破損 | openzfs/zfs#12014 は 2021-05 よりオープン；2024-02-12 Phoronix 報道；openzfs-docs#494 でドキュメント警告追加を提案；同 issue 下で 2025-05-31 に非 raw send の暗号化バグ2件修正と報告、2.2.8 / 2.3.3 へマージ | 公開 issue とニュース報道、**本プロジェクト内では未検証**。「暗号化がレプリケーション/スナップショット経路に浸食する」定性的証拠であり、性能や確率データではない |
| 暗号化前の圧縮は必然的に長さを漏洩 | Kelsey, *Compression and Information Leakage of Plaintext*, FSE 2002 | 順序は逆転不可（暗号文は圧縮できない）のため、圧縮を選択した以上受容すべき漏洩 |
| dedup + 暗号化は同一性を漏洩 | ZFS は**平文の HMAC** を salt+IV として使用（先頭64ビットを salt、後方96ビットを IV）、同一平文から同一暗号文が得られることを保証 | ソースコメントの自認: 攻撃者はどのブロックが同一であるかを検知可能、「だが dedup は元々それを漏洩している」。また暗号化下での dedup は clone family 内でのみ機能 |
| ユーザーキーの変更 != 旧キーの無効化 | `zfs-load-key(8)`: 古いラッピング済みマスターキーは「accessible via forensic analysis for an indeterminate length of time」 | マニュアルページ原文 |

参照元:
- [bcachefs Encryption 設計ドキュメント](https://bcachefs.org/Encryption/)
- [OpenZFS `zio_crypt.c` 設計コメント](https://github.com/openzfs/zfs/blob/master/module/os/linux/zfs/zio_crypt.c)
- [Linux fscrypt カーネルドキュメント](https://www.kernel.org/doc/html/latest/filesystems/fscrypt.html)
- [zfs-load-key(8)](https://openzfs.github.io/openzfs-docs/man/master/8/zfs-load-key.8.html)
- [OpenZFS 暗号化ドキュメント警告提案 openzfs-docs#494](https://github.com/openzfs/openzfs-docs/issues/494)
- [Phoronix: OpenZFS Native Encryption Use Raises Data Corruption Concerns (2024-02-12)](https://www.phoronix.com/news/OpenZFS-Encrypt-Corrupt)
- [Kelsey, Compression and Information Leakage of Plaintext (FSE 2002)](https://link.springer.com/chapter/10.1007/3-540-45661-9_21)

### 6.5 ボリューム同一性はどの層に置くか: ext4 も fscrypt も導出層に置き、ユニット単位入力には置かない

**2026-08-29 にローカルカーネルツリーを行単位で確認、D9（暗号化）既定項目8の確定に使用。**

| 実装 | ボリューム/ファイル同一性が投入される層 | ユニット単位入力に含まれるもの | 出典 |
|---|---|---|---|
| ext4 チェックサム | s_uuid を**マウント時に1回畳み込み** s_csum_seed を生成 | inum、gen、ブロック内容のみ；s_uuid は一度も現れない | fs/ext4/super.c ext4_init_metadata_csum；fs/ext4/ialloc.c __ext4_new_inode；fs/ext4/bitmap.c ext4_inode_bitmap_csum_verify；fs/ext4/super.c ext4_group_desc_csum |
| fscrypt 鍵 | ファイルごとの nonce を**inode 作成時に乱数として1回取得**、ディスク上 context に永続化 | 導出されたファイルごとの鍵 | fs/crypto/keysetup.c fscrypt_prepare_new_inode 内 get_random_bytes(nonce, FSCRYPT_FILE_NONCE_SIZE)；fs/crypto/fscrypt_private.h struct fscrypt_context_v2 |

⚠️ **fscrypt は「マウントごとに新しいソルトを生成する」わけではありません**——本プロジェクトの初期ドラフトでそのように記述していましたが、誤りでした。
nonce は **inode 作成時に** 1回生成されてディスク上の context に書き込まれ、以降のマウントでは同一の値が読み出されます。
「マウントごとに新しいソルト」を実際に適用した場合、前回のマウントで書き込まれたブロックは二度と復号できなくなります。

**本プロジェクトへの意義**: D9（暗号化）において fsid を AAD から KDF へ移動させたことは、これら2つの実装の階層構造と合致しています——ボリューム/ファイル同一性は導出層に入り、オブジェクト同一性はユニット単位層に入ります。[decisions.md](decisions.md) D9（暗号化）既定項目8を参照。

### 6.6 平文システム設定内の水位フィールドはロールバック可能であり、現役で実証された障害モードである

**2026-08-29 にローカルカーネルツリーで確認。** dm-integrity の `recalc_sector` は平文システム設定内に存在し、そのドキュメントは以下のように自認しています:

> "legacy_recalculate — Allow recalculating of volumes with HMAC keys. This is disabled by
> default for security reasons - an attacker could modify the volume, set recalc_sector to
> zero, and the kernel would not detect the modification."
> （`Documentation/admin-guide/device-mapper/dm-integrity.rst`）

アップストリームの対処は「水位を MAC に含める」ことではなく、**鍵が存在する場合はデフォルトでそのパスを無効化し**、明示的に `legacy_recalculate` を指定した場合のみ有効化する設計としました（`drivers/md/dm-integrity.c` の `dm_integrity_disable_recalculate()`）。

**本プロジェクトへの意義**: I-6.5 (nonce 水位単調)の nonce 水位は同一の構造であり、その結果はさらに致命的です——ロールバックは単なる「改ざん検知の失敗」にとどまらず、**ストリーム暗号の nonce 再利用 ⇒ 2つの平文の直接 XOR 漏洩** を招きます。
⇒ D9（暗号化）既定項目8にて「水位はマスターキーから導出された MAC で保護されなければならず、非鍵側による書き込みは禁止される」と定められました。

### 6.7 AEAD の tag 長: カーネルは 32 ビットまでの切り詰めを許容

`crypto_gcm_check_authsize()` は 4 / 8 / 12 / 13 / 14 / 15 / 16 バイトを受け付けます
（`include/crypto/gcm.h` の `crypto_gcm_check_authsize()`、2026-08-29 確認）。
⇒ 「MAC は一律 128 ビットで切り詰めない」という規則は、**本プロジェクト独自の設計制約であり、暗号理論やカーネルから強制されたものではありません**。
それが支払う空間コスト（ユニットあたり 16 バイト）はこの計測条件に基づいて記録されるべきであり、「選択の余地がなかった」と表現してはなりません。

## 7. 直近10年の学術成果スキャン（2012–2026）

**「7. 直近10年の学術成果スキャン（2012–2026）」は「既存の決定事項の前提を覆す可能性のある」成果のみを収録し、一般的な文献レビューは行いません。** 各項目がどの決定事項に影響するかを明記します。
すべて公開論文およびプロジェクトドキュメントに基づくものであり、**本プロジェクト内では未検証**です。

### 7.1 ランダム小書き込み: 書き込み最適化インデックスにより非問題化（D10 に影響）

同一のランダム小書き込みマイクロベンチマークにおける BetrFS 0.4 と主要ファイルシステムの実測値（FAST 2018 論文 Table 2）:

| ファイルシステム | 256K 回の 4バイトランダム書き込み所要時間（秒、低いほど良い） |
|---|---|
| BetrFS 0.4 | 4.9 ± 0.3 |
| BetrFS 0.3 | 5.9 ± 0.1 |
| nilfs2 | 2013.1 ± 19.1 |
| btrfs | 2147.5 ± 7.4 |
| ext4 | 2776.0 ± 40.2 |
| xfs | 2835.7 ± 67.9 |
| zfs | 3288.9 ± 394.7 |

**計測条件**: 256K 回の 4バイトランダム書き込み、合計わずか 1 MiB の書き込み量——これは**極端に振り切ったマイクロベンチマーク**であり、データベースや仮想マシンイメージの現実的な負荷ではありません。論文自認で「up to 600 times faster」。

**⚠️ この数値群の永続化モデルは singlefs と異なるため、外挿前に本節を精読してください（2026-08-26 確認）。**
FAST 2018 論文 2.1 および 4.x 節原文:

> Crash consistency is ensured using **logical logging**, i.e., by logging the inserts,
> deletes, etc, performed on the tree. **Internal operations, such as node splits, flushes,
> etc, are not logged.** Nodes are written to disk using **copy-on-write**. At a periodic
> checkpoint (**every 5 seconds**), all dirty nodes are written to disk and the log can be trimmed.

> BetrFS ensures crash consistency by keeping a **redo log** of pending messages and applying
> messages to nodes copy-on-write. Crash recovery simply replays the redo log since the last checkpoint.

ここから本プロジェクトにとって重要な3つの事実が導かれます:

| 事実 | singlefs にとっての含意 |
|---|---|
| BetrFS は COW **である**——ノードは copy-on-write でディスクに書き出される | 「その数値は非 COW システムのものだ」という反論は成り立ちません；これを理由に Bε を退けることはできません |
| BetrFS の永続性は **redo log + 5秒ごとの checkpoint** に依存している | その5秒ウィンドウが膨大な量の COW ノード書き換えを償却しています。⚠️ **本プロジェクトには「トランザクションごとに新しいルートをパブリッシュする」というコミットメントはありません**——`litmus/commit-publish.litmus` は単一のパブリッシュイベント**内部**の順序（ブロック内容書き込み → smp_wmb → generation 書き込み）を規定しているだけで、パブリッシュ頻度には一切言及していません；`decisions.md` / `CLAUDE.md` / `.claude/rules/fs-design.md` にもそのような記述はありません（2026-08-26 grep 確認）。パブリッシュ頻度は**未決定**事項です |
| 論文全体を通じて **checksum が一切登場しない**。親ノードが子ノードのチェックサムを保持する Merkle 連鎖が存在しない | singlefs の D4 インラインチェックサム + D9 AEAD MAC/nonce は、各ノード書き換えに BetrFS が支払わない固定コストを課します。クロスオーバーポイントは移動しますが、その移動幅は未知数です |

**したがってこの数値が証明できること**: メッセージバッファは「キーが書き直される回数」を O(樹高) から O(樹高/B) に引き下げられること。
**証明できないこと**: Merkle チェックサム連鎖 + AEAD が存在し、パブリッシュ頻度を特定の値に設定した前提下で、どれほどの利益が残存するか。
残存する利益の検証は [experiments.md](experiments.md) E7 (オフラインインデックスharness)で独自に測定する必要があります。

⚠️ **パブリッシュ頻度はここでは自由変数であり、定数ではありません。** 本プロジェクトが checkpoint セマンティクス（メモリ内にダーティノードを蓄積し、定期的に一括書き込みする）を採用する場合、「小さな書き込みが葉から根までの全パスを直ちに書き直す必要がない」という効果は、**checkpoint バッチ処理自体が無償で提供します**。フォーマット内に永続的なメッセージバッファを組み込む必要はありません。
BetrFS の数値のうち、どれだけがメッセージバッファに起因し、どれだけが5秒 checkpoint に起因するかは、**論文内で分離されたことがありません**。
E7 の対照群はしたがって、2つの「メッセージバッファなし」ベースラインを区別しなければなりません: バッチ処理皆無、および checkpoint バッチ処理はあるがフォーマットレベルのメッセージバッファはない構成。両者を混同すると、メッセージバッファの必要性が実際以上に過大評価されます。

メカニズムは、小書き込みをインプレース更新や COW の無効化ではなく、Bε-tree のメッセージバッチフラッシュに変換することです。

D10 (ランダム書き込み / 断片化の真の回答)への意義: btrfs の `nodatacow` は「COW を無効化してランダム書き込み性能を得る代わりに、チェックサムとスナップショット整合性を同時に喪失する」アプローチですが、この路線は「ランダム書き込みをインデックスのメッセージ層に吸収し」、チェックサムとスナップショットの双方を無傷で維持します。
**これは実測数値に裏付けられた D10 の唯一の候補方向です。**

**付随して確認されたもう一つの事実**: BetrFS のすべてのバージョンは**2本の** Bε-tree を使用しています——1本はファイルデータ用、もう1本はファイルシステムメタデータ用（論文 2.2 節原文: "All versions of BetrFS use two Bε-trees: one for file data and one for file system metadata"）。
これは D8 (コアインデックス構造) の「複数の独立 keyspace」と同一の思想であり、ツリーの数がはるかに少ないだけです（bcachefs は 28本）。

### 7.2 老化は宿命ではなく、構造設計レベルで排除可能（D3 (領域割り当て)、D10 (ランダム書き込み / 断片化の真の回答) に影響）

FAST 2017「File Systems Fated for Senescence? Nonsense, Says Science!」は、**Linux カーネルソースの連続 git checkout** によってファイルシステムを劣化させ、btrfs / ext4 / F2FS / XFS / ZFS と BetrFS を測定しました。
結論: **劣化した BetrFS は、他のファイルシステムが劣化していない未老朽化状態（btrfs を除く）をすら上回りました**。
FAST 2018 の動機として引用されたもう1つの数値: git バージョン管理ワークロードは ext4 の scan 性能を最大 **15倍** 退化させます。

本プロジェクトにとって: D3 (領域割り当て) は「分類しない → 断片化圧力が単一のアロケータに集中する」という理由から、常駐の自動デフラグを公約しています。
この成果は第2の道筋を提示しています——**劣化はインデックス構造そのものによって吸収可能であり、事後整理に全面的に頼る必要はない**ということです。
両方式のトレードオフは測定可能であり、判定基準は [experiments.md](experiments.md) E10 を参照してください。

### 7.3 インデックス構造: パラメータ空間は「別のツリーへの換装」より遥かに広大（D8 (コアインデックス構造) に影響）

| 成果 | 結論 | 計測条件 |
|---|---|---|
| SplinterDB（ATC 2020） | STBε-tree = Bε-tree + LSM の size-tiering、「ツリーのツリー」構造: 幹（trunk）ツリーの各ノードが B-tree 分岐の集合を指し、各分岐が quotient filter を保持。RocksDB 比: 挿入 6–10倍高速、ポイント検索 2–2.6倍高速、書き込み増幅 2倍低減 | 論文抄録数値、NVMe SSD 対象 |
| FAST 2022「Closing the B+-tree vs. LSM-tree Write Amplification Gap」 | ハードウェア内蔵の透過的圧縮下で、Bε-tree の書き込み増幅は 28（8KB ページ）/ 36（16KB ページ）；同条件で RocksDB は 38、WiredTiger は 268（8KB）/ 530（16KB） | 500GB データセット、32B レコード、4 クライアントスレッド |
| WiscKey 式の key-value 分離 | 大きな value をインデックス外へ追い出しインデックスにはポインタのみ保持。コンパクションはキーとメタデータのみを移動し、書き込み増幅を最大2桁削減。業界では RocksDB BlobDB、Titan、TerarkDB として実用化 | 論文 + 産業実装 |
| HashFS（FAST 2021「Rethinking File Mapping for Persistent Memory」） | ファイルマッピング自体が IO パスの **70%** を占め得る；ハッシュ式マッピングはページキャッシュ内の extent ツリーに対して LevelDB 上の YCSB スループットを最大 45% 向上 | **永続メモリ環境**；本プロジェクトが PM を対象としない場合は参考扱い |
| TableFS（ATC 2013） | メタデータをすべて LSM（LevelDB）に格納、メタデータ集約型負荷で ext4 / XFS / btrfs より 50%–1000% 高速 | FUSE 実装、著者自認で非効率な実装ながらも優位 |

D8 (コアインデックス構造) への意義: D8 (コアインデックス構造) は「単一の btree 実装 + 複数 keyspace + write buffer / key cache の2つのフロントエンド」を決定済みです。
これらの成果はこの方向性を否定するものではなく、**同一構造内部での調整可能項目** を大幅に拡大します:
Bε バッファ比率、size-tiering、key-value 分離、フィルタ種別——これらはすべて単一の btree 内のパラメータであり、「ツリー種別の追加」ではありません。
これは D8 (コアインデックス構造) の「会計ツリー、中央マッピングツリー、ツリーテーブル、インスタンステーブル、inode ツリーは単一の btree で統一する」と矛盾せず、むしろ補強となります（割り当てレコードツリーと extent ツリーは確定項目 14 により異種可能とされたため除外）。

### 7.4 クラッシュ整合性の検証: 「オラクルが存在しない」という前提の再評価（検証計画に影響）

[CLAUDE.md](../../CLAUDE.md) には、本プロジェクト最大の隠れたコストは「ゼロからの設計には比較できる参照実装がない」ことだと記されています。
過去10年間に少なくとも5種類の既存手法があり、それぞれ得られるものが異なります:

| 手法 | 代表例 | 提供されるもの | コスト / 前提 |
|---|---|---|---|
| プッシュボタン式形式検証 | Yggdrasil（OSDI 2016, Best Paper） | **crash refinement** 正当性定義: 実装が生成し得るディスク状態集合（クラッシュ時含む）が仕様の許容集合の部分集合でなければならない。Z3 による自動検証、**手動の証明注釈不要** | 実装をシンボリック実行可能な形態で記述する必要あり；論理表現と物理表現の分離により検証時間を1分以内に短縮と報告 |
| 有界ブラックボックスクラッシュテスト | CrashMonkey + ACE（OSDI 2018） | 有界パラメータでワークロードを網羅探索し、各クラッシュ点後の復旧を検査。65台で2日間実行: 既知バグ24個再現、新バグ10個発見（7個は2014年からカーネル内に存在）；**形式検証済みの FSCQ においてもクラッシュ整合性バグを発見** | 有界空間のみカバー；境界外については何も保証しない |
| PM 向け同種フレームワーク | Chipmunk（EuroSys 2023 Best Paper 候補） | 同種手法を永続メモリファイルシステムに適用、5つの PM ファイルシステムで23個の新バグを発見（マウント不可、rename 原子性破壊含む） | PM セマンティクス対象 |
| モデル検査 + 差分テスト | Metis（FAST 2024） | モデル検査の状態空間網羅と差分テストを結合、**抽象モデル不要**: 別のファイルシステムを参照とし、抽象状態、システムコール戻り値、errno を逐次比較。バグ再現性向上に特化した軽量参照ファイルシステム **RefFS** を開発 | 信頼できる参照実装が必要 |
| 並行 + クラッシュの機械検証証明 | GoJournal（OSDI 2021）/ DaisyNFS（OSDI 2022） | GoJournal は検証済みの並行クラッシュ安全ログ層（Coq / Perennial 2.0）、**検証過程で大量の単体テストをすり抜けた深刻な並行性バグを摘出**；DaisyNFS は GoTxn 上で Dafny を用いて各 NFS 操作を**逐次的に**検証 | 高い証明コスト |

**本プロジェクトに直結する2つの教訓**:

1. **Metis の RefFS は、「参照実装」が他者の成熟したファイルシステムである必要はなく、差分テスト専用に書かれた小さな実装でよいことを示している。**
   これは `.claude/singlefs-ai-sop/rules/test-discipline.md` のモデル突き合わせと一致しますが、Metis はランダム操作列を流すだけでなく、状態空間の網羅的探索と接続させています。
2. **DaisyNFS の階層構造は `.claude/rules/fs-design.md` の「全構造が共有する単一のトランザクション層」と同型である**:
   トランザクション層が並行性とクラッシュを一度に解決すれば、上位層は逐次的な正当性のみを検証すればよくなります。
   これはプロジェクト外の独立した根拠を与えています。

### 7.5 コミットプロトコル: 順序付けは必ずしも flush で買う必要はない（トランザクション層設計に影響）

OptFS（SOSP 2013）は ext4 のジャーナルコミットを楽観的プロトコルに改修しました:
**トランザクションチェックサム**（データブロックもチェックサムに含め、復旧時に不一致ならトランザクション全体を破棄）によるコミット順序付けの代替；
**非同期永続化通知**（asynchronous durability notification）による同期的 flush の代替；
順序性と永続性を2つの API（`osync()` / `dsync()`）に分離。特定負荷で1桁の性能向上を報告。

NoFS（FAST 2012, "Consistency Without Ordering"）はさらに進んでいます:
各データブロックに**バックポインタ**を付与し、読み取り時にバックポインタで整合性を判定、**書き込みの順序付けを完全に排除**し、形式化モデルを提示。

本プロジェクトにとって: `litmus/commit-publish.litmus` は単一のパブリッシュイベント内部の「ブロック内容を先に書き、システム設定を後に書く」順序を固定しており、パブリッシュ頻度には触れていません。
これらの成果は、順序付けに代替形態が存在し、トレードオフが異なることを示しています。
⚠️ **NoFS のバックポインタと D1 の逆インデックスは別物です**:
NoFS のものは「各ブロックが誰に属するかを自証する」単一フィールドであり、D1 は「物理ブロックが誰から参照されているか」の検索可能インデックスです。
本プロジェクトの「メタデータブロックは自己記述的でなければならない」（`.claude/rules/fs-design.md`）は NoFS のフィールドに近接しています。

### 7.6 ハードウェアインタフェース: ブロックインタフェースはもはや唯一の抽象層ではない（D2 (RAID ストライプ方針)、D3 (領域割り当て)、D8 (コアインデックス構造) のノードサイズに影響）

| インタフェース | 状態 | 本プロジェクトへの意義 |
|---|---|---|
| ZNS（Zoned Namespace） | シーケンシャル書き込み、ホスト管理 GC；コスト効率のため大規模配備で採用 | zone サイズは D8 (コアインデックス構造) の「デバイス特性に応じてノードサイズを選択する」規則への入力値 |
| NVMe FDP（Flexible Data Placement） | Google SmartFTL と Meta Direct Placement Mode を統合、ZNS と標準 SSD の間のコスト/利益の空白を埋める。**シーケンシャル書き込みを強制せず後方互換、未改変アプリも稼働** | ZNS より本プロジェクトに適した層: シーケンシャル書き込みを強制せず、COW の配置自由度と衝突しない |

計測条件: NVM Express および SNIA 公開資料、2025年 arXiv 論文、**本プロジェクト内では未検証**。
両者の共通点はデータ配置と GC の責務をホストに委ねる点にあります——**これは COW ファイルシステム自身が割り当てを管理することと重複します**。
重複部分が利益となるか二重管理の代償となるかは、実測による検証が必要です。

### 7.7 カーネルファイルシステムにおける Rust の現状（工法に影響、フォーマットには影響せず）

| 事実 | 出典および日付 |
|---|---|
| Bento（FAST 2021）: safe Rust でカーネルファイルシステムを記述、エラーをファイルシステム内にサンドボックス化。性能はネイティブ VFS ext4 と同等、FUSE 版より 7倍（git clone）〜 90倍（Filebench）高速；**オンラインアップグレードに対応、サービス中断は約 10ms**、実行中のファイルシステムを無停止置換 | 論文およびプロジェクトページ |
| Greg Kroah-Hartman 2026-07-15 表明: カーネル内 Rust はもはや実験ではなく恒久的構成要素；グラフィックスサブシステムは今後 Rust 製の新規ドライバのみ受領 | 二次報道、**原文メール未確認** |
| Rust binder ドライバが Linux 6.18 にマージ；PuzzleFS が Rust for Linux プロジェクト一覧に掲載 | 二次報道 |

D7 で定めたメインライン非マージ方針に対して: Bento のオンラインアップグレード機能は注目に値します——「ファイルシステムがカーネル内に存在すべきか否か」を二者択一から三者択一（ユーザー空間 / カーネル C モジュール / ホット置換可能なカーネル Rust モジュール）に変容させています。

### 7.8 bcachefs の現状更新（D7 の根拠を補強、結論は変更せず）

| 事実 | 日付 |
|---|---|
| bcachefs は Linux 6.17 で externally maintained とマークされ、**コードは 6.18 でメインラインから削除** | 2025–2026、公開報道 |
| DKMS モジュール配布へ移行、形態は OpenZFS on Linux と同様；開発者は独立メーリングリストへパッチ投稿 | 同時期 |
| bcachefs-tools v1.38.6 で **experimental タグを削除** | 2026-06-17 |

D7 のメインライン非マージ方針に対して: この事実は論拠を「bcachefs のメインライン離脱は技術的失敗ではなく社会的失敗である」から、「**メインライン離脱後もプロジェクトは正常に進化を続けており、その後に実験的タグが外された**」へと強化します——メインライン外の配布は実行可能なパスであり、失敗後の退路に留まりません。

### 7.10 bcachefs 公式ドキュメント追補（2026-08-26 確認、Principles of Operation Rev 1.39.2+2202696）

| 事実 | 原文 / 計測条件 | 本プロジェクトへの意義 |
|---|---|---|
| **bucket gen の適用範囲はキャッシュデータに限定** | §1.5: "we can reuse a bucket with **cached data** in it without finding and deleting all the data pointers by incrementing the generation number" | [decisions.md](decisions.md) D1: 無効化のみを管理し、移動は管理しない |
| **gen ラップアラウンド機構は逆インデックスに置換済み** | 状態一覧 need_gc_gens: "Legacy state, retained for compatibility … **now effectively unused since the invalidate worker uses backpointers instead of generation bumping**" | D1 で選択した「初日からの逆インデックス」方針と同調 |
| **copygc は 8% を予約、5–20% で設定可能** | §1.5 および §9.1.10。また "Normal writes cannot dip into this reserve"、容量約90%超過時に書き込みレイテンシが上昇 | D3 (領域割り当て) の「空間予約はアドミッション制御形態とし、量は小さく抑える」に対する比較基準: **他社は 8% を使用不可として確保**、本プロジェクトは動的アドミッションを採用しており、差異を明記すべき |
| **暗号化は全か無か、mkfs 時のみ有効化可能** | §2.1.2: "Encryption is all-or-nothing at the filesystem level: all data and metadata except the superblock is encrypted, and all data and metadata is authenticated … **Encryption can only be enabled at format time; it cannot be added to an existing filesystem**" | D9 の方向性を裏付け；ただし本プロジェクト D9 は「システム設定の KDF 識別子 + マスターキースロット」を予約し、**既存ファイルシステム上での暗号化有効化** を目指しており、意図的な相違点 |
| ⚠️ **nocow で書き込まれたデータは暗号化ファイルシステム上でも平文保存** | §2.1.2: "Data written with the nocow option is stored **unencrypted**, even on an encrypted filesystem. **This is a hard design incompatibility, not a policy choice**: ChaCha20 requires a unique nonce per (key, plaintext), and bcachefs stores the nonce externally alongside each data pointer" | **D9 と D10 の連動を示す実証**: 本プロジェクトは「COW を回避する高速パスであっても暗号化を回避してはならない；インプレース上書きは直ちに nonce 再利用を招く」と決定済み。bcachefs も同一の制約に直面し、**nocow データを非暗号化とする**選択をした——本プロジェクトはその妥協を受け入れず、D10 のいかなる候補もインプレース上書きを禁止 |
| **マウント前にアンロック必須** | §4.2.3: "the passphrase is requested once the devices have been found and **before any attempt to mount**"；FAQ: 未アンロックマウントは Required key not available を返却 | D9 既定項目5の推論を支える前提の1つ |

**見つからなかったもの（明記し、補完しない）**: 「copygc / rebalance / scrub / fsck は鍵なしでは動作できない」と散文で明記した一次資料は存在しませんでした。
「鍵なし時のバックグラウンド整理停止」は上表の複数の事実から導き出された**推論であり、引用文ではありません**（decisions.md D9 参照）。

参照元: [bcachefs Principles of Operation (PDF)](https://bcachefs.org/bcachefs-principles-of-operation.pdf)、
[bcachefs.org/Encryption](https://bcachefs.org/Encryption/)、
[bcachefs.org/FAQ](https://bcachefs.org/FAQ/)、
[The Programmer's Guide to bcache](https://bcache.evilpiepirate.org/BcacheGuide/)、
[Kent Overstreet: Bcachefs - encryption, fsck, and more (LWN ミラー)](https://lwn.net/Articles/717378/)、
[Bringing bcachefs to the mainline (LWN, LSFMM 2022)](https://lwn.net/Articles/895266/)。

### 7.11 bcachefs の会計検証における自己無効化の欠陥（Linux v6.17 ソースで行単位確認、2026-08-26）

**本節は推測ではなく、現実に発生した障害を記録するものです。**

bcachefs の `bch2_check_allocations()` は全走査によって会計を再構築し、実行時カウンタと `memcmp` で突き合わせます——**形態は本プロジェクトの [invariants/05-空间记账.md](../../crates/singlefs-checker/invariants/05-空间记账.md) における I-3.1 (割り当て済み統計が一致する)と同一です**。しかし、両パスは全く同一の加減算コードを共有しています:

```c
this_cpu_add(e->v[gc][i], a.v->d[i]);        // fs/bcachefs/disk_accounting.h, bch2_accounting_mem_mod_locked()
```

`gc` が唯一の相違点であり、実行時は `v[0]`、GC 再構築時は `v[1]` に書き込みます。
アップストリームの `__trigger_extent()` も単一の `bool gc = flags & BTREE_TRIGGER_gc;` で分岐しているに過ぎません。
`btree_gc.c` のコメントは再構築側について「is not idempotent; we'll calculate the wrong result if we run it multiple times」と自認しており——**再構築側自体が一連の += 処理です**。

| 検知できるもの | 検知できないもの |
|---|---|
| 増分適用過程の乖離: 更新の脱落、並行性の喪失、クラッシュ後の未リプレイ | **加減算の意味論自体の誤り**: ある分岐で符号を取り違えた場合、実行時と再構築時が同一の誤った値に一致し、memcmp が完全にパスしてしまう |

**本プロジェクトへの意義**: `.claude/singlefs-ai-sop/rules/evidence-discipline.md` には「検証の2つのパスは同一のコード、サンプリング、ツールを共有してはならない」と規定されています。本節は、その規則が実際のシステムで**すでに破られ、死角を生み出した**実例です。処置は [checks-owed.md](checks-owed.md) C12 を参照。

**付随して記録された会計形式の事実**（同バッチで確認）: accounting ツリーは `bcachefs_format.h` の `BCH_BTREE_IDS()` において**フォーマットレベル**で `BTREE_IS_write_buffer` に束縛されています；`disk_accounting_format.h` 原文: "Unlike with other key types, **updates are _deltas_**"；マージ関数は `dst->v.d[i] += src.v->d[i]`；write buffer は accounting と通常キーに対し、バッファ投入、flush 重複排除、btree 書き出しの3箇所で明示的に分岐します；Δ は非べき等であるため、特定の Δ が適用済みかを判定する独立した **bversion 全順序** を設ける必要があり、ジャーナルリプレイ完了前は accounting キーの flush が一律禁止されます。
**「バッファに増分を置く」ことから派生するコスト一覧がこれらです。**

出典: Linux mainline v6.17 `fs/bcachefs/`（`disk_accounting.{c,h}`、`disk_accounting_format.h`、`btree_write_buffer.{c,h}`、`btree_trans_commit.c`、`btree_gc.c`、`buckets.c`、`bcachefs_format.h`）、取得の上行単位で突合。

## 8. ルートリングのスロット幅と「長時間消費者がパブリッシュとどう競合するか」の実装状況

**2026-09-08 にローカル固定点 `~/code/fs-refs` で確認・抽出。いずれも本プロジェクト内では未検証。**
D2 既定項目15（ライフサイクルの異なるオブジェクトの一般的基準）および [checks-owed.md](checks-owed.md) C214 に対する回答です。

### 8.1 ZFS: ルートリングのスロット幅はデバイス粒度に応じて拡大するが、上限 8 KiB であり、スロット拡大はスロット数削減であって専有面積の増加ではない

出典: `zfs/include/sys/vdev_impl.h` 内の `VDEV_UBERBLOCK_RING` から `VDEV_UBERBLOCK_SIZE(vd)` のマクロ群および `zfs/include/sys/uberblock_impl.h` の `UBERBLOCK_SHIFT`:

```c
#define	VDEV_UBERBLOCK_RING	(128 << 10)
/* The largest uberblock we support is 8k. */
#define	MAX_UBERBLOCK_SHIFT (13)
#define	VDEV_UBERBLOCK_SHIFT(vd)	\
	MIN(MAX((vd)->vdev_top->vdev_ashift, UBERBLOCK_SHIFT), \
	    MAX_UBERBLOCK_SHIFT)
#define	VDEV_UBERBLOCK_COUNT(vd)	\
	(VDEV_UBERBLOCK_RING >> VDEV_UBERBLOCK_SHIFT(vd))
```

`UBERBLOCK_SHIFT` は `10 /* up to 1K */`；`vdev_ashift` のフィールドコメントは `/* block alignment shift */`。

**3つの読み取り結果**:

| 読み取り事項 | 根拠コード |
|---|---|
| **スロット幅はデバイス粒度に追従**し、固定定数ではない | `MAX((vd)->vdev_top->vdev_ashift, UBERBLOCK_SHIFT)` |
| **ただし上限 8 KiB**——アレイのストライプブロックが大きいからといって過大に膨らまない | `MAX_UBERBLOCK_SHIFT (13)` とその直前のコメント |
| **リング総サイズは 128 KiB 固定 ⇒ スロット幅拡大はスロット数削減を意味**し、ディスク専有領域の増加ではない | `VDEV_UBERBLOCK_COUNT = VDEV_UBERBLOCK_RING >> SHIFT` |

⚠️ **使用されている指標は ashift（アライメント / 最小割り当て単位）であり、Linux の io_min（アレイ上のストライプチャンクサイズ）ではありません。**
両指標は md アレイ上で大幅に乖離します——E34 (ルートリングスロット幾何) は実環境の md/raid5 で 128倍の開きを測定しました。

⚠️ **本プロジェクトとの既知の相違点**（`.claude/singlefs-ai-sop/rules/evidence-discipline.md` の要求事項）:
ZFS のラベルはディスクあたり4箇所、各 256 KiB の**固定領域**であり、uberblock リングはそのうち 128 KiB を恒久的に占有します；
一方本プロジェクトのルートリングは R 個の領域が素数ストライドでディスク上に分散配置され、**領域サイズはフォーマット定数ではありません**——領域あたりスロット数 S はシステム構成に保持され、区間の二つの境界（下限 4、上限 16）だけがフォーマット定数です（D22 (ユニット原子性の合成手法) 確定項目 2）。⇒ 「スロット幅拡大 = スロット数削減」という等式は、本プロジェクトにおいて「領域サイズ固定」の条項が明文化されて初めて成立するものであり、現状リポジトリにその条項はありません。

### 8.2 btrfs: commit root + 中断可能走査に依存するが、本プロジェクトでは2つの独立したゲートにより除外

出典: `linux-6.17/fs/btrfs/`、2026-09-08 確認:

| 箇所 | コード原文 |
|---|---|
| scrub.c scrub_setup_ctx 内の2箇所 | `sctx->extent_path.search_commit_root = 1;` / `sctx->csum_path.search_commit_root = 1;` |
| scrub.c scrub_enumerate_chunks | `path->search_commit_root = 1;` |
| transaction.c switch_commit_roots | `down_write(&fs_info->commit_root_sem);` / `up_write(&fs_info->commit_root_sem);` |
| ctree.c finish_need_commit_sem_search | `lockdep_assert_held_read(&lowest->fs_info->commit_root_sem);` |
| transaction.c btrfs_commit_transaction | `btrfs_scrub_pause(fs_info);` |

⚠️ **本プロジェクトはこの手法を採用できません。2つの除外ゲートが独立して存在します（2026-09-08 確定）**:
① **コミット間隔が 8.3万倍乖離**——btrfs のデフォルトコミット間隔は `#define BTRFS_DEFAULT_COMMIT_INTERVAL (30)`（`linux-6.17/fs/btrfs/fs.h`、30秒）ですが、本プロジェクトの負荷甲におけるパブリッシュ間隔は **0.36ミリ秒**（2785パブリッシュ/秒）です。btrfs の前提は「コミット間隔が消費者の走査スライスより遥かに長い」ことであり、移植した場合 scrub は毎秒2785回中断され、走査が完了しません。
**この暗黙の前提はその手法とともに引用されたことがありません。**
② **チェッカー側が構造的にそのロックを取得できない**——D13 (検証ルート) 確定項目 5 の原文: 「kb から生成された定数モジュールのみを共有；newtype、パース、チェックサム、走査、会計はそれぞれ独立実装」⇒ 独立したチェッカーが被テスト実装の `commit_root_sem` を保持することは不可能です。

3点セットの記録は、何が除外されたかを明確化するためであり、候補案ではありません。

=> **3点セット**: scrub は生ツリーではなく **commit root**（直近コミット済みトランザクションのルート）を読み取る；コミット経路が commit root を切り替える際は `commit_root_sem` の**書き込みロック**を取得し、走査側は読み取りロックを取得して相互排他する；コミット経路は **scrub を明示的に一時停止**（`btrfs_scrub_pause`）させ、失効寸前のルートで処理を続行させない。

**これは「保護ウィンドウを走査期間をカバーするまで引き伸ばす」こととは根本的に異なる道筋です**:
ブロック再利用の世代ウィンドウには手を加えず、消費者を**一時停止可能にし、コミット点と同期させる**方式です。

⚠️ **本プロジェクトとの既知の相違点**: btrfs の commit root は**1世代のみ**（直近コミット済みトランザクション）をメモリ内の読み書きロックで維持するため、**実行時**の半分のみをカバーします；
一方 I-7.4 は「クラッシュ後は defer キューのメモリ状態が消失し、実行時の K はルートリングの深度が引き継ぐ（掃除アドミッション『解放世代 ≤ リング内の最古ルート txg』ゲート）、保護幅はそれに応じて変化する」と規定しており——**クラッシュ後の半分に対して、ロックは何の発言権も持ちません。**

<!-- doc-lint:not-numbers EXT4 NILFS2 -->

## 9. オブジェクトごとの拡張スロット: 現役5実装における「オブジェクトに属するが主要フィールド外」のデータの格納方式（D21 既定項目3 ③ 外部調査、2026-09-13 確認）

D21 既定項目3 ③: 「ユーザー要求の外部調査（他社がオブジェクトごとの拡張スロットをどう実装しているか）が未完了」。5実装すべて原文を確認（URL は表内に記載）、**本プロジェクト内では未検証**；SOP 規則に基づき、仮説の提示やテスト対象パスの指摘に留め、肯定/否定/検証の根拠には組み入れません。各実装について本プロジェクトとの差異を1点記載します。

| 実装 | 拡張スロットの配置場所 | 形態と幅 | 収まらない場合の挙動 | 出典（確認済み） | 本プロジェクトとの差異 |
|---|---|---|---|---|---|
| ext4 | inode レコード内の固定フィールド以降の領域（i_extra_isize に Size of this inode - 128 を記録、デフォルト 256B inode では 160B が固定、残りが xattr 用） | ヘッダ ext4_xattr_ibody_header 4B（magic 0xEA020000）+ エントリ ext4_xattr_entry（e_name_len, e_name_index, e_value_offs, e_value_inum, e_value_size, e_hash）、値は 4B アライメント；インラインデータは system.data という名称の xattr（EXT4_INLINE_DATA_FL） | i_file_acl が指す独立ブロックへ退避（「このブロックが第2の拡張属性ブロックへのポインタを含むことはできない」）；さらに大きい場合は EA inode（e_value_inum, EXT4_EA_INODE_FL）を使用 | https://www.kernel.org/doc/html/latest/filesystems/ext4/inodes.html , https://www.kernel.org/doc/html/latest/filesystems/ext4/attributes.html | スロットは inode レコード内にあり任意のキー値を格納可能；本プロジェクトの拡張点は D1 既定項目3に従い「空であるか単一の参照であるか」であり、データをインライン化しない |
| XFS | inode リテラル領域（XFS_LITINO = inode サイズ - コア）をデータフォークと属性フォークで共有、di_forkoff（attr fork offs, <<3 for 64b align、単位 8B）で属性フォーク起点を設定；di_aformat で形式指定 | 短形式: xfs_attr_sf_hdr { totsize u16, count u8, padding u8 } + xfs_attr_sf_entry { namelen u8, valuelen u8, flags u8, nameval[] }、「リテラル領域に収まるよう極力詰め込む」 | 属性フォークを extents / btree 形式に切り替え、リーフブロック、ノードブロック、リモート値ブロックを指す | https://raw.githubusercontent.com/torvalds/linux/master/fs/xfs/libxfs/xfs_format.h , https://raw.githubusercontent.com/torvalds/linux/master/fs/xfs/libxfs/xfs_da_format.h | フォーク起点は inode ごとに可変オフセット（di_forkoff）；本プロジェクトの拡張点起点はシステム設定の N とユニット構造によって暗黙的に決定され、ユニットごとには保持しない |
| btrfs | fs ツリー内で objectid に紐づくエントリ: XATTR_ITEM（0x18、キーに名前ハッシュ）、DIR_ITEM（0x54）、EXTENT_DATA（0x6c、インライン時は「残りのアイテムバイトがデータバイト」） | リーフ内の可変長エントリ: header \| item 0 \| item 1 \| … \| free space \| data N \| … \| data 0；インラインデータ上限 max_inline は default: min(2048, page size)、4KiB sectorsize 時約 3900B | 同一ハッシュの xattr は同一リーフに収める必要あり；max_inline を超えるデータは外部 extent を利用 | https://btrfs.readthedocs.io/en/latest/dev/On-disk-format.html , https://btrfs.readthedocs.io/en/latest/Administration.html | 拡張データはインデックスツリー内の独立した可変長エントリ；本プロジェクトはユニット自己記述ヘッダ後の固定クォータ内に配置 |
| ZFS | dnode（DNODE_SHIFT 9、512B）コア 64B 直後のボーナスバッファ: DN_BONUS_SIZE(dnsize) = dnsize - DNODE_CORE_SIZE - (1 << SPA_BLKPTRSHIFT)（512B dnode で 320B）、dn_bonustype u8、dn_bonuslen u16 | ボーナスはシステム属性（SA）を格納；種別は dn_bonustype で宣言 | スピルブロック（Spill blocks）を用いてボーナスバッファに収まらないメタデータを格納（DNODE_FLAG_SPILL_BLKPTR）、スピルポインタはボーナス末尾の blkptr を消費 | https://raw.githubusercontent.com/openzfs/zfs/master/include/sys/dnode.h | オブジェクトごとのスロット幅は dnode サイズマイナスコア（1KiB 超の dnode も可能）；本プロジェクトの拡張点クォータ N はシステム設定で宣言され、第1版では 0 |
| APFS | inode レコード値 j_inode_val 末尾の可変長拡張フィールド: apfs_xf_blob { xf_num_exts u16, xf_used_data u16, xf_data[] }、各フィールド apfs_x_field { x_type u8, x_flags u8, x_size u16 }；DSTREAM=8, NAME=4, FINDER_INFO=7 等 | フィールドごとに 4B メタデータ + 値；xattr 値 apfs_xattr_val { flags u16, xdata_len u16, xdata[] } | 埋め込み上限 APFS_XATTR_MAX_EMBEDDED_SIZE = 3804（XATTR_DATA_EMBEDDED=2）、それを超えるものはデータストリーム（XATTR_DATA_STREAM=1） | https://raw.githubusercontent.com/linux-apfs/linux-apfs-rw/master/apfs_raw.h（Apple PDF は CID フォントのため Linux ドライバヘッダで確認） | 型 + 長さの自己記述フィールドリストでデータを格納；本プロジェクトの拡張点は単一の参照のみを格納 |

**5実装に共通する形態**: いずれも「オブジェクトレコード内の上限付き領域 + 収まらない場合の外部参照」であり、上限は 320バイト（ZFS 512B dnode）から約 3.9 KiB（btrfs 1リーフ、APFS 3804）。**「単一の参照のみを許容し、インラインデータを禁じる」設計を採用している実装は存在しない**——これは本プロジェクト D1 既定項目3独自の形態であり、SOP 証拠規則において「現役実装が1つも X の道を選んでいない」ことは最も価値の高い外部シグナルです: 立証責任は当方にあります（D1 既定項目3および D21 ハード制約5に理由を記録）。
**「空」をどう表現するか、各実装の手法**: ext4 は `e_value_size = 0` とエントリ数；XFS は `count = 0`；btrfs はエントリの不在；ZFS は `dn_bonuslen = 0`；APFS は `xf_num_exts = 0`——**いずれも「カウントが 0」であり「オールゼロのポインタ」ではない**。これは D21 既定項目3 ②の「オールゼロは空として直接扱えない」という方針と同調しています。

## 10. スナップショット下でのデッドブロック会計、共有ブロック移動、discard タイミング

取得日 2026-10-12；いずれも本プロジェクト内では未検証であり、手がかりであって証拠ではありません。

**調査バージョンと取得方法**

| 対象 | バージョン / コミット | 取得方法 |
|---|---|---|
| OpenZFS master | 4542a317e2fa524e24c1c28b6fbca9cd3634ba56（2026-10-09） | git clone --depth 1 --filter=blob:none --sparse、module/zfs include/sys man cmd/zpool cmd/zfs をチェックアウト |
| Linux Mainline | 24a19b953e2a3d0ea5a8c4951a0ab8f32720c907（Linux 7.3-rc7、2026-10-11） | --depth 1 --sparse、fs/btrfs fs/f2fs fs/nilfs2 include/uapi/linux Documentation/filesystems をチェックアウト |
| bcachefs | bcachefs-tools efec39c7e8d1dda11fb16bad0e1e75d86ead3105（2026-10-10）の fs/ ディレクトリ | --depth 1。bcachefs はメインライン外: HEAD git ls-tree fs/ \| grep -c bcachefs は 0 |
| nilfs-utils | master 26efb5daff0757365101035145331b0a5a85d9d9 | raw.githubusercontent から lib/gc.c、man/nilfs_cleanerd.conf.5 を取得 |
| APFS | Apple File System Reference、PDF 作成日 2020-06-22、181頁 | developer.apple.com からダウンロード、pypdf でテキスト抽出 |
| btrfs ドキュメント | btrfs.readthedocs.io/en/latest の Trim、btrfs-quota、Qgroups、btrfs-balance、btrfs-man5 | curl で取得 |

ファイル:行番号はすべて「確認した版と取得方法」の表にあるコミットを指す。本機に元からあった `/home/fy5090/code/fs-refs/zfs`（`58d73c90`、2026-08-27）と `/home/fy5090/linux-bug-fix/linux`（v7.3-rc1、浅いクローン）はメインラインより古いため、引用には使わなかった。

本プロジェクト側で差分を書くのに使った事実は二つだけで、どちらもその場で確認したものです：`.claude/kb/decisions/19-块指针的结构与宽度预算.md` の「確定項目 5：位置エントリはヒントに降格し、中央マッピングが唯一の入口」と「確定項目 7：ポインタヘッダに誕生 (tree, txg) を持ち、各 8 バイト」（略して「本プロジェクトは中央マッピング経由」「本プロジェクトのポインタは誕生 txg を持つ」）。

### 10.1 ZFS: deadlist、livelist、BP rewrite 不在の理由、device removal の indirect mapping

| 事実 | 出典（OpenZFS 4542a317） | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| メモリ構造: deadlist は dle_mintxg でソートされた AVL ツリー、各エントリが bpobj を保持；dl_cache が存在；ディスク上ヘッダ dsl_deadlist_phys_t は used/comp/uncomp の3合計のみ保持 | include/sys/dsl_deadlist.h 定義；module/zfs/dsl_deadlist.c:106（dsl_deadlist_compare が dle_mintxg を比較） | ソース精読 | 誕生 txg でバケット分割された deadlist: ZFS のブロックポインタは物理 DVA であり中央マッピングなし；本プロジェクトは中央マッピング経由で誕生 txg を保持 |
| ディスク上のキー: deadlist オブジェクトは ZAP、キーは mintxg（10進数文字列）、値は bpobj オブジェクト番号 | dsl_deadlist.c:534（zap_add_int_key(... mintxg, obj ...)）、:895-897 | ソース精読 | ZFS は物理 DVA；本プロジェクトは中央マッピング経由 |
| バケット投入規則: 解放ブロックは BP_GET_BIRTH(bp) に基づき mintxg が自身より厳密に小さい最大キーを検索（mintxg is not inclusive） | dsl_deadlist.c:477-491；:511-514 コメント | ソース精読 | ZFS は物理 DVA；本プロジェクトは中央マッピング経由 |
| deadlist 投入タイミング: dsl_dataset_block_kill で birth > ds_prev_snap_txg のブロックは直接解放；そうでなければ head の deadlist へ；zio コールバック時は ds_pending_deadlist に先行投入、sync スレッドで追補 | module/zfs/dsl_dataset.c:304-349；:2397-2398 | ソース精読 | ZFS は物理 DVA；本プロジェクトは中央マッピング経由 |
| スナップショット作成: スナップショットが head の旧 deadlist オブジェクトを継承；head はクローン（同一キー、空 bpobj）に置き換わり、新キー ds_prev_snap_txg を追加 | dsl_dataset.c:1787、:1842-1849；dsl_deadlist.c:700-730 | ソース精読 | ZFS は物理 DVA；本プロジェクトは中央マッピング経由 |
| 並行性: deadlist は syncing スレッドのみで変更；insert を除き dp_config_rwlock 書き込みロックが必要 | dsl_deadlist.c:25-31 | コメント精読 | 並行性モデルの差異は手がかりに留める |
| スナップショット削除（新形式）: prev の unique/snapused を調整；next の deadlist 内で mintxg >= 削除スナップショットの ds_prev_snap_txg であるバケットを丸ごとプールの dp_free_bpobj へ移動；削除スナップショット自身の deadlist を next の deadlist へマージ；head の deadlist から creation_txg キーを削除 | module/zfs/dsl_destroy.c:403-439、:478-487；dsl_deadlist.c:920-979、:855-918、:539-567 | ソース精読 | 本プロジェクトは「スナップショット削除時に deadlist をより新しい側へマージ」予定；ZFS も next（より新しい側）へマージ |
| マージと移動は bpobj サブオブジェクトの丸ごと連結（bpobj_enqueue_subobj / dle_enqueue_subobj）であり、個別ブロックの移動ではない；最大128個のサブ deadlist をプリフェッチ | dsl_deadlist.c:822、:954、:882-891、:942-949 | ソース精読 | ZFS は物理 DVA；本プロジェクトは中央マッピング経由 |
| 旧ディスクフォーマット（deadlist が単一の bpobj、dl_oldfmt）は process_old_deadlist: birth と ds_prev_snap_txg を逐次比較 | dsl_destroy.c:127-151、:154-188 | ソース精読 | 互換性の負債: 本プロジェクトには旧フォーマットなし |
| livelist の定義: dsl_deadlist_t を流用、クローンの dsl_dir に紐づき、クローン作成後に出生したブロックのみ記録、各エントリに FREE/ALLOC を付与；クローン削除時に逆スキャンしてペアを相殺、残余 ALLOC を解放 | dsl_deadlist.c:45-63；dsl_dataset.c:291-302 | コメントおよびソース | 本プロジェクトは読み取り専用スナップショットのみ；ZFS livelist は書き込み可能クローン専用 |
| livelist 利用条件: livelist 特性有効時にクローン作成で割り当て；共有比率 <= zfs_livelist_min_percent_shared（デフォルト 75%）で無効化；promote/swap 時に破棄；クローン削除時に livelist があれば dsl_async_clone_destroy を実行 | dsl_dataset.c:1353-1365、:2169-2183、:2400-2404、:4019-4026、:4518-4524；dsl_destroy.c:1139-1146 | ソース精読 | 本プロジェクトは読み取り専用スナップショットのみ；ZFS livelist は書き込み可能クローン専用 |
| livelist サブテーブル: エントリが zfs_livelist_max_entries（デフォルト 50万、約 128M）超過で新規分割；birth txg でグループ化し同一ブロックのペアを同一サブテーブルに収めることで増分削除可能；定期 condense スレッド | dsl_deadlist.c:64-75、:77-86；spa.c:3486、:3559 | コメントおよびソース | 本プロジェクトは読み取り専用スナップショットのみ；ZFS livelist は書き込み可能クローン専用 |
| livelist 特性ページ説明: ランダム/スパース書き込みが多いクローンにおいて旧手法より高速に削除；作成時から有効、全クローン削除まで持続 | man/man7/zpool-features.7:775-782 | ドキュメント精読 | 本プロジェクトは読み取り専用スナップショットのみ；ZFS livelist は書き込み可能クローン専用 |

ZFS 第2群: block pointer rewrite 不在、device removal は indirect mapping を採用。

| 事実 | 出典および日付 | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| ソース自認の理由: 削除デバイスに対応する indirect vdev は「since we cannot rewrite block pointers of snapshots, etc.」のため旧位置から新位置へのマッピングを保持し、DVA アクセスごとに照合 | module/zfs/vdev_indirect.c:36-47（4542a317） | コメント精読 | ポインタを変更せず SPA 層に旧→新マッピングを追加: ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| マッピングは論理ブロック境界に従わず、1つの DVA が複数セグメントに跨がり得る（split block）ため、全 DVA を新 DVA に置換できるわけではなく、セグメント単位コールバック vdev_remap を提供 | vdev_indirect.c:41-58 | コメント精読 | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| 設計者発言（Alex Reece, 2015-01-13, Delphix ブログ）: ブロックポインタごとのマッピングは「quickly requires an unreasonably large data structure that must be stored both on disk and kept in memory」；BP rewrite も巨大な一時テーブルを要するため、SPA 層で連続セグメント単位マッピングを採用 | Delphix ブログ 2020年 web.archive.org 魚拓 104-107行 | 魚拓ドキュメント精読 | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| 削除フロー: 割り当て停止；スレッドが LBA 順に割り当て済みセグメントをコピー、sync task がマッピング更新；コピー中の free は旧位置とコピー先新位置を同時解放；完了後 vdev は indirect に変化 | module/zfs/vdev_removal.c:43-73 | コメント精読 | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| 制約: 通常クラスに raidz/draid トップレベル vdev 不可；全トップレベル vdev の ashift 一致；暗号化データセットのキー読み込み済み；全 vdev オンライン・健全 | vdev_removal.c:75-92；man/man8/zpool-remove.8:50-52 | コメントとドキュメント | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| 既知の未解決事項：「device removal for RAID-Z」issue #9013（ahrens、2019-07-10 起票、2026-10-12 時点でも open）、「RFC: generic device removal」#9129（2019-08-05 起票、引き続き open） | GitHub API `repos/openzfs/zfs/issues/9013`、`/9129`、2026-10-12 取得 | issue を読んだ | ZFS は通常は中央マッピングを持たず、デバイスを除去した後にその vdev だけにマッピングを追加する。本プロジェクトは中央マッピング方式であり、すべてのポインタが最初からマッピングを経由する。 |
| メモリコスト: マッピング全体をメモリに展開（vmem_alloc(map_size) + dmu_read）；zpool remove -n が予想メモリを表示；zpool status が使用メモリを表示 | module/zfs/vdev_indirect_mapping.c:357-361；zpool-remove.8:78-83；cmd/zpool/zpool_main.c:10607 | ソースとドキュメント | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| ディスク上の1マッピング = vimep_src（u64、元オフセット63ビット + 1ビットマーク）+ vimep_dst（dva_t）；asize 24ビット、単一エントリで最大 8GB マッピング | include/sys/vdev_indirect_mapping.h:29-43 | ソース精読；推定 24バイト（8 + 16） | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| あるユーザーが報告した量：6.94T の vdev を除去した後「39.8M memory used for removed device mappings」 | issue #18930 本文（2026-08-15、TrueNAS、zfs-2.3.0） | 他者が報告した出力。負荷と断片化の程度は未確認 | ZFS は通常は中央マッピングを持たず、デバイスを除去した後にその vdev だけにマッピングを追加する。本プロジェクトは中央マッピング方式であり、すべてのポインタが最初からマッピングを経由する。 |
| マッピング縮小: 解放または remap されたブロックは obsolete となりカウント；obsolete 率 >= zfs_condense_indirect_obsolete_pct（デフォルト 25%）または obsolete spacemap > 1GB で condense 実行、有効エントリのみで新マッピング再書き込み；再開可能；obsolete_counts 特性は不可逆 | vdev_indirect.c:60-124、:126-171、:175-197；zpool-features.7:864-878 | コメントとドキュメント | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| remap: 間接ブロック書き込み時に indirect vdev を指す DVA を具体的 DVA に置換（置換可能時のみ）；dedup/gang は対象外；スナップショット参照の旧 DVA は ds_remap_deadlist へ投入 | module/zfs/metaslab.c:5699-5730；vdev_indirect.c:100-123；dsl_destroy.c:280-302 | ソース精読 | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| zfs remap サブコマンドは master で廃止済み | grep '"remap"' cmd/zfs/zfs_main.c で該当なし | grep 実測 | 互換性の負債: 本プロジェクトにはなし |
| split block 読み取り時に破損レプリカがある場合、組み合わせで再構築。組み合わせ数が上限（デフォルト 4096）を超えるとランダムに 100種のみ試行 | vdev_indirect.c:207-215 | コメント精読 | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |
| 最近の indirect 関連の不具合（タイトルと状態だけ）:#18930「Kernel NULL pointer dereference in vdev_indirect_mapping_duplicate_adjacent_entries() … on a pool with an indirect vdev」open、#19120「vdev_indirect: validate split-block copies before repair」closed（2026-09-16）、#12200「VERIFY3(0 == vdev_obsolete_sm_object(...)) failed」open | GitHub search API `q=repo:openzfs/zfs "indirect vdev"`、全 292 件、2026-10-12 取得 | タイトルだけを読んだ。#18930 は本文の冒頭も読んだ | ZFS は通常は中央マッピングを持たず、デバイスを除去した後にその vdev だけにマッピングを追加する。本プロジェクトは中央マッピング方式であり、すべてのポインタが最初からマッピングを経由する。 |
| 近似の現役手法: zfs rewrite はファイルブロックを新位置にそのまま書き直す；スナップショット共有ブロックは新旧両方残り容量増、-S で共有ブロック除外、-C で BRT クローン除外、-P で論理 birth 保持 | man/man8/zfs-rewrite.8:31-57、:84-91；zpool-features.7:880-895 | ドキュメント精読 | ZFS は通常時中央マッピングなし；本プロジェクトは全ポインタが中央マッピング経由 |

調べたが見つからなかったもの：OpenZFS のツリーに「なぜ BP rewrite をしないか」を正面から説明する設計文書は見つからなかった（疎なチェックアウトの `module include man cmd` で `grep -rn -i 'bp rewrite\|bprewrite\|block pointer rewrite\|rewrite block pointers\|bp_rewrite'` を実行し、唯一の一致は `module/zfs/vdev_indirect.c:38` のコメント。チェックアウトしていない `tests`、`lib` などのディレクトリは調べていない）。Delphix の元のブログページは公開終了し、アーカイブだけが残る。indirect mapping の読み取り経路の遅延コストを公開測定した数値は見つからなかった。

### 10.2 btrfs: 共有ブロック参照カウント、スナップショット削除、balance/relocation

| 事実 | 出典（Linux 24a19b95、7.3-rc7） | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| スナップショット作成: btrfs_copy_root はルートノードのみコピー、新ルートに btrfs_inc_ref 呼び出し（子ポインタ参照カウント +1）、元ツリーの last_snapshot に transid 設定；非ルートブロックは複製せず | fs/btrfs/ctree.c:238-296；transaction.c:1798、:1827 | ソース精読 | extent tree 内の参照カウントとバックレフ、COW 時の遅延参照プッシュ: btrfs は親ノードが子の論理アドレスを直接保持；本プロジェクトは中央マッピング経由 |
| 共有判定: 非共有ツリー、ルートノード、generation > last_snapshot かつ relocation 由来でないブロックは extent tree を見ずに非共有と判定 | ctree.c:312-347 | ソース精読 | btrfs は親ノードが子の論理アドレスを直接保持；本プロジェクトは中央マッピング経由 |
| 共有可能性ブロックの COW（update_ref_for_cow）: refs > 1 で自ツリー所属かつ FULL_BACKREF なしの場合、旧ブロックに btrfs_inc_ref(..., full_backref=true) し BTRFS_BLOCK_FLAG_FULL_BACKREF を付与；refs == 1 で FULL_BACKREF の場合は新ブロック inc_ref、旧ブロック dec_ref | ctree.c:349-451 | ソース精読 | btrfs は親ノードが子の論理アドレスを直接保持；本プロジェクトは中央マッピング経由 |
| btrfs_inc_ref / btrfs_dec_ref は __btrfs_mod_ref へ: ブロック内子ポインタを走査し delayed ref 発行；full_backref 時は parent = buf->start、通常は parent = 0 | extent-tree.c:2553-2649 | ソース精読 | btrfs は親ノードが子の論理アドレスを直接保持；本プロジェクトは中央マッピング経由 |
| スナップショット削除（btrfs_drop_snapshot）: ツリーを走査し自ツリー専有ブロックを解放；共有ブロックは参照カウントを -1 して下降しない（do_walk_down コメント） | extent-tree.c:6247-6259、:5902-5913 | コメント精読 | 本プロジェクトは参照カウントツリー走査ではなく deadlist を計画 |
| 中断・再開可能: 各ラウンドで drop_progress と drop_level を root_item に記録；トランザクション終了要求時に永続化して -EAGAIN で脱出、マウント時に再開 | extent-tree.c:6318-6373、:6403-6433、:6514-6519 | ソース精読 | btrfs は親ノードが子の論理アドレスを直接保持；本プロジェクトは中央マッピング経由 |
| 再開時の二重減算防止: wc->restarted 時に check_ref_exists で参照の存在を確認してからフラグをクリア | extent-tree.c:5856-5869 | ソース精読 | btrfs は親ノードが子の論理アドレスを直接保持；本プロジェクトは中央マッピング経由 |
| qgroup 下でのスナップショット削除: refs > 1 の共有部分木で btrfs_qgroup_trace_subtree 呼び出し；部分木ルート階層 >= qgroup_drop_subtree_thres（デフォルト 3）時は計算せず inconsistent とマーク、コミット遅延を防止 | extent-tree.c:5871-5887；qgroup.c:2647-2658；qgroup.h:127；sysfs.c:2447-2448 | ソース精読 | btrfs は親ノードが子の論理アドレスを直接保持；本プロジェクトは中央マッピング経由 |
| 公式ドキュメントの qgroup 記述: スナップショット作成/削除を頻繁に行う環境での日常利用は非推奨；qgroup とスナップショットの中核設計は「初日から非互換」；ハングを引き起こし得る | btrfs-quota.html STABILITY AND PERFORMANCE IMPLICATIONS | ドキュメント精読 | 本プロジェクトに qgroup 会計層なし；手がかり |
| squota ドキュメント: qgroup は参照変更時に参照ツリー数を集計する必要があり、スナップショット増加時に容認できない遅延を生む | Qgroups.html Simple quotas (squota) | ドキュメント精読 | 本プロジェクトに qgroup 会計層なし；手がかり |
| relocation 全体フロー: ブロックグループを読み取り専用化；全 extent 記録；data reloc tree と reloc tree を作成；元ツリーと reloc tree を交換；reloc tree をクリーンアップ | relocation.c:42-87 | コメント精読 | 従来型 relocation: バックレフ追跡、reloc tree、全ポインタ書き換え: btrfs は各呼び出し元のポインタ書き換え必須；本プロジェクトは中央マッピング経由 |
| バックレフ追跡: build_backref_tree がバックレフから親ブロックを再帰検索しツリールートまで遡る | relocation.c:485-498 | コメント精読 | btrfs は各呼び出し元のポインタ書き換え必須；本プロジェクトは中央マッピング経由 |
| データ relocation 2段階: MOVE_DATA_EXTENTS でデータコピー、flush 後 UPDATE_DATA_PTRS で file extent item を新位置へ書き換え | relocation.c:5382-5431、:928-959 | ソース精読 | btrfs は各呼び出し元のポインタ書き換え必須；本プロジェクトは中央マッピング経由 |
| クラッシュ復旧: btrfs_recover_relocation が reloc tree と fs tree のマージを再開、共有を維持；balance 進捗は内部保持されマウント時に再開 | relocation.c:5614-5620；btrfs-balance.html | コメントとドキュメント | btrfs は各呼び出し元のポインタ書き換え必須；本プロジェクトは中央マッピング経由 |
| balance ドキュメント: extent 共有は維持され reflink も破損しない；フィルタなし balance は全ブロックポインタ更新のため多大な時間を要する | btrfs-balance.html 111-123行 | ドキュメント精読 | btrfs は各呼び出し元のポインタ書き換え必須；本プロジェクトは中央マッピング経由 |
| qgroup と relocation: 部分木交換時に大量のブロック所属追跡が必要、qgroup_trace_extent_swap および btrfs_qgroup_add_swapped_blocks で遅延追跡 | qgroup.c:2297-2330、:4751-4765 | コメント精読 | btrfs は各呼び出し元のポインタ書き換え必須；本プロジェクトは中央マッピング経由 |
| バックレフ走査コストの回避策: send は参照数 > SEND_MAX_EXTENT_REFS（1024）の extent を clone から write にフォールバック、「backreference walking code の限界回避」 | send.c:37-43、:1506-1514 | ソース精読 | btrfs は各呼び出し元のポインタ書き換え必須；本プロジェクトは中央マッピング経由 |

btrfs 第2群: メインライン新規の remap tree（relocation 時に参照元ポインタを変更せず、アドレス再マッピング層を挟む方式）。

| 事実 | 出典および日付 | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| 導入: incompat ビット BTRFS_FEATURE_INCOMPAT_REMAP_TREE、ツリー ID BTRFS_REMAP_TREE_OBJECTID 13、キー IDENTITY_REMAP(234)/REMAP(235)/REMAP_BACKREF(236)、フラグ REMAPPED & METADATA_REMAP | include/uapi/linux/btrfs.h:339；btrfs_tree.h:80、:288-290、:1174-1175、:1348-1355 | ソース精読 | ブロックグループ単位の旧→新アドレスマッピング、I/O 時に変換: btrfs は移動対象ブロックグループのみマッピング；本プロジェクトは全ポインタが中央マッピング経由 |
| 初回マージ: v6.19 で未導入、v7.0-rc1 で導入；コミット ef6a31d035a1（Mark Harmstone, 2026-01-07） | git.kernel.org btrfs.h およびコミット | git 検証 | btrfs は移動対象ブロックグループのみマッピング；本プロジェクトは全ポインタが中央マッピング経由 |
| 実験的機能: sysfs remap_tree 属性は CONFIG_BTRFS_EXPERIMENTAL 内に配置、「機能完成後に削除」と注記 | sysfs.c:297-304、:333-337 | ソース精読 | btrfs は移動対象ブロックグループのみマッピング；本プロジェクトは全ポインタが中央マッピング経由 |
| 対象ブロックグループ: REMAP_TREE 有効かつ非 SYSTEM、非 METADATA_REMAP；move_existing_remaps、start_block_group_remapping、do_remap_reloc を実行 | relocation.h:15-24；relocation.c:5542-5559、:3747-3751；volumes.c:3745-3750 | ソース精読 | btrfs は移動対象ブロックグループのみマッピング；本プロジェクトは全ポインタが中央マッピング経由 |
| I/O 変換: REMAPPED ブロックグループアクセス時、btrfs_translate_remap が remap tree から該当エントリを検索；IDENTITY_REMAP は未移動、REMAP は新開始アドレスを返却 | relocation.c:5256-5304；volumes.c:7258-7268 | ソース精読 | btrfs は移動対象ブロックグループのみマッピング；本プロジェクトは全ポインタが中央マッピング経由 |
| 空ブロックグループ判定: REMAPPED は identity_remap_count == 0、通常は used == 0 && remap_bytes == 0 | discard.c:218-235 | ソース精読 | btrfs は移動対象ブロックグループのみマッピング；本プロジェクトは全ポインタが中央マッピング経由 |
| RFC の動機: 「バックレフを追跡してメタデータツリー内のアドレスを書き換える現行方式は低速であり、nocow ファイルを破壊する副作用がある」 | Mark Harmstone, RFC PATCH btrfs: remap tree, 2025-05-15, LWN | メール要約 | btrfs は移動対象ブロックグループのみマッピング；本プロジェクトは全ポインタが中央マッピング経由 |

調べたが見つからなかったもの：btrfs メインラインで「qgroup を有効にすると balance がどれだけ遅くなるか」の公開測定値は見つからず、文書の定性的な記述だけだった。7.x の remap tree に既知の不具合一覧があるかは調べていない（linux-btrfs メーリングリストと 7.0～7.3 の fixes は読んでいない）。remap tree 上でスナップショット共有 extent を移動する際に qgroup をどう計算するかのソースも読んでいない。

### 10.3 bcachefs: スナップショットツリーとキーの可視性、copygc による共有データ移動、backpointers

| 事実 | 出典（bcachefs-tools efec39c7） | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| 4本の btree がスナップショット対応（extents, inodes, dirents, xattrs）；各キーの位置にスナップショット ID（bpos.snapshot）を保持；要求スナップショットの祖先または自身であり、より近い祖先で上書きされていない場合に可視；削除時は whiteout を挿入 | snapshots/snapshot.c:44-54 | ドキュメントコメント | 参照カウントや deadlist ではなくスナップショット ID によるキー可視性で共有: bcachefs はキー空間内で共有；本プロジェクトは誕生 txg 分割 deadlist + 中央マッピング |
| 祖先判定: 各スナップショットノードが 128ビット祖先ビットマップ（差 128 以内 O(1)）+ 3要素ランダムスキップリストを保持 | snapshots/snapshot.c:56-60；snapshots/types.h:14 | コメントとソース | bcachefs はキー空間内で共有；本プロジェクトは誕生 txg 分割 deadlist + 中央マッピング |
| スナップショット ID は下向き割り当て: 子は親より小さく、「祖先は後方、最新が前方」に配置されるため、可視性フィルタは前方スキャン1回 | snapshots/snapshot.c:62-84 | コメント精読 | bcachefs はキー空間内で共有；本プロジェクトは誕生 txg 分割 deadlist + 中央マッピング |
| スナップショット作成: 現在のスナップショットノード下に2つの子ノードを分岐（1つは新スナップショット、1つは元サブボリューム置換）；キーの複製は行わない | snapshots/snapshot.c:86-98 | コメント精読 | bcachefs はキー空間内で共有；本プロジェクトは誕生 txg 分割 deadlist + 中央マッピング |
| extent 不変条件: 異なるスナップショットの2つの extent は完全に同一範囲であるか、一切重複しないかのいずれか；書き込み時に祖先 extent を分割してアライメント | snapshots/snapshot.c:117-121 | コメント精読 | bcachefs はキー空間内で共有；本プロジェクトは誕生 txg 分割 deadlist + 中央マッピング |
| スナップショット削除：まずマークし、続いて `delete_dead_snapshots` リカバリ pass（オンラインでも実行）で一回または複数回に分けて回収する。「Cost is proportional to the data in the deleted snapshot」。生存する子孫がないノードの key は直接捨て、生存する子が一つだけの中間ノードでは key をその子（`live_child`）に下げる。生存する子が二つある中間ノードは残す。 | `snapshots/snapshot.c:128-163` | コメント精読 | bcachefs の共有単位は btree key で、スナップショットツリーは key 空間にある。本プロジェクトは誕生 txg による deadlist と中央マッピングを使う計画。本プロジェクトの計画は「deadlist を新しい側へマージ」で、bcachefs は key を唯一の生存子孫に下げる。 |
| 中断セマンティクス: キーは下向きにのみ移動し、inodes btree は最後に処理されるため、中断時は「内容キー移動済み、inode 未移動」の偏向のみ残る | snapshots/snapshot.c:215-218 | コメント精読 | bcachefs はキー空間内で共有；本プロジェクトは誕生 txg 分割 deadlist + 中央マッピング |
| copygc: bucket は追記のみ；copygc が最も断片化した bucket を選択し、backpointers で生存データを特定、新 bucket へ書き出して extent ポインタをアトミック更新；copygc_reserve（デフォルト 8%、5–20%）を確保 | data/copygc.c:6-28；doc/bcachefs-principles-of-operation.tex:308-321 | ドキュメントコメント | extent は論理位置をキーとし物理ポインタを値に格納；bcachefs は単一の共有キーを書き換え；本プロジェクトは中央マッピングエントリを書き換え |
| bucket 選択: 断片化 LRU は dirty_sectors / bucket_size を 2^31 でスケーリング（生存データ最少のものを貪欲選択）；書き込み中、書き込み不可、backpointer 不一致の bucket はスキップ | alloc/background.h:127-145；data/copygc.c:98-140 | ソース精読 | bcachefs は単一の共有キーを書き換え；本プロジェクトは中央マッピングエントリを書き換え |
| 移動コミット: bch2_data_update が旧キーと同一スナップショット位置に新キーを書き込み（BTREE_UPDATE_internal_snapshot_node 付与）；extent 分割時は whiteout を補完 | data/update.c:431、:473-479；btree/update.c:123-159 | ソース精読 | bcachefs は単一の共有キーを書き換え；本プロジェクトは中央マッピングエントリを書き換え |
| 「論理アドレスでキー付けするのでポインタを変更しなくてよい」のか：ソースによれば、移動時にも extent key 内の物理ポインタを変更する必要がある（同じ key に対して一トランザクション）。変更不要なのは「各スナップショットそれぞれの参照」であり、共有 key が一つしかないため。 | `data/copygc.c:25-27`、`data/update.c:473-479` | ソースを読んだ。「スナップショットごとの変更は不要」は「共有 key が一つだけ」からの推論 | bcachefs の key はスナップショット ID を持つ。複数のスナップショットが共有するデータには祖先スナップショット内の key が一つだけあり、移動時はそれだけを変更する。本プロジェクトは中央マッピングを経由し、移動時はマッピングエントリを変更する。 |
| backpointers：データを持つディスク上の各セクタ範囲に逆ポインタがあり、書き込みバッファ btree に保存する。key = (device, sector offset, discriminator)、値には所有者 key の btree ID、level、位置（スナップショット ID を含む）、データ型、バケット generation を記録する。extent trigger が自動的に追加・削除する。 | `alloc/background.c:246-263`、構造は `bcachefs_format.h:568-577` | コメントとソース | bcachefs の key はスナップショット ID を持つ。複数のスナップショットが共有するデータには祖先スナップショット内の key が一つだけあり、移動時はそれだけを変更する。本プロジェクトは中央マッピングを経由し、移動時はマッピングエントリを変更する。 backpointer が指すのはその唯一の key であり、スナップショットごとに一条あるのではない（key の共有からの推論）。 |
| 整合性：backpointer のセクタ数とバケットのセクタ数が同じなら、そのバケットは整合していると見なし、一致しない場合だけ双方向の全検査を行う。三つのリカバリ pass が相互検査する。 | `alloc/background.c:284-306` | コメント精読 | bcachefs の key はスナップショット ID を持つ。複数のスナップショットが共有するデータには祖先スナップショット内の key が一つだけあり、移動時はそれだけを変更する。本プロジェクトは中央マッピングを経由し、移動時はマッピングエントリを変更する。 |
| backpointers 利用操作: copygc、デバイス撤退、scrub、reconcile | alloc/background.c:271-282 | コメント精読 | bcachefs は単一の共有キーを指す；本プロジェクトは中央マッピングエントリを書き換え |

調べたが見つからなかったもの：bcachefs が「スナップショットが多い場合の copygc または delete_dead_snapshots のコスト」を公開測定した数値は見つからなかった。`doc/bcachefs-principles-of-operation.tex` の 722 行目「Subvolumes and snapshots」節は一行ずつ読んではおらず、ソースにある `DOC_LATEX(snapshots)` コメントだけを読んだ。

### 10.4 F2FS と NILFS2: セグメントクリーニング

F2FS（Linux 24a19b95）:

| 事実 | 出典 | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| 3種のセグメント選択アルゴリズム: GC_CB（費用便益）、GC_GREEDY、GC_AT（年齢閾値）；2種の GC モード: BG_GC（バックグラウンド）、FG_GC（オンデマンド） | fs/f2fs/segment.h:132-154 | ソース精読 | ログ構造セグメントクリーニング: F2FS はスナップショットを持たず共有ブロックを扱わない；本プロジェクトの GC はスナップショット共有ブロックを移動 |
| デフォルト対応: BG_GC は CB（atgc 有効時は AT）、FG_GC は GREEDY；gc_urgent/gc_idle で強制指定可能 | fs/f2fs/gc.c:248-273；Documentation/filesystems/f2fs.rst:822-828 | ソースとドキュメント | F2FS はスナップショットなし；本プロジェクトの GC は共有ブロックを移動 |
| greedy コスト = セグメント内の有効ブロック数；CB コスト = UINT_MAX - (100*(100-u)*age)/(100+u)（u は有効ブロック割合、age は正規化 mtime） | gc.c:390-410、:362-388 | ソース精読 | F2FS はスナップショットなし；本プロジェクトの GC は共有ブロックを移動 |
| バックグラウンド GC が選定した victim は victim_secmap に記録され、フォアグラウンド GC が優先利用；探索範囲は max_victim_search で制限 | gc.c:343-360、:302-310 | ソース精読 | F2FS はスナップショットなし；本プロジェクトの GC は共有ブロックを移動 |
| フォアグラウンドトリガー: 空き section 不足で FG_GC へ移行；prefree が十分あれば checkpoint を先行して prefree を free 化し FG_GC を回避 | gc.c:1986-2004、:2068-2075 | ソース精読 | F2FS はスナップショットなし；本プロジェクトの GC は共有ブロックを移動 |
| 移動の原子性: クリーンアップされたセグメントは prefree に入り、do_checkpoint 成功後に f2fs_clear_prefree_segments がクリアして discard を発行 | fs/f2fs/checkpoint.c:2039-2046；fs/f2fs/segment.c:2209-2282 | ソース精読 | F2FS はスナップショットなし；本プロジェクトの GC は共有ブロックを移動 |
| SSR 割り当ては ckpt_valid_map \| cur_valid_map のゼロビットを選択、前回 checkpoint 時の有効ブロックは次回 checkpoint 前に上書きされない | segment.c:3126-3140；segment.h:182-185 | ソース精読 | F2FS はスナップショットなし；本プロジェクトの GC は共有ブロックを移動 |
| ドキュメント: 「F2FS adopts greedy algorithm for on-demand cleaner, while background cleaner adopts cost-benefit algorithm」 | Documentation/filesystems/f2fs.rst:817-828 | ドキュメント精読 | F2FS はスナップショットなし；本プロジェクトの GC は共有ブロックを移動 |

NILFS2（Linux 24a19b95 カーネル、nilfs-utils 26efb5da ユーザー空間）:

| 事実 | 出典 | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| チェックポイントは数秒ごとまたは同期書き込み時に自動生成；チェックポイントをスナップショットへ変換可能；NILFS_IOCTL_CHANGE_CPMODE で切り替え | Documentation/filesystems/nilfs2.rst:14-18、:100-102 | ドキュメント精読 | DAT によるブロック仮想化、GC は DAT を修正しツリーは不変、生存期間はチェックポイント区間で判定: 双方とも間接層あり；NILFS2 は全巻チェックポイントでユーザー空間デーモン判定 |
| チェックポイント削除時のスナップショットスキップ: 範囲削除でスナップショットに遭遇すると -EBUSY；単一削除も -EBUSY | fs/nilfs2/cpfile.c:507-513、:558-559、:750-777 | ソース精読 | DAT 仮想ブロック番号マッピング vs 本プロジェクト中央マッピング |
| GC はユーザー空間で実行する：`nilfs_cleanerd` がセグメントを選び、生存を判定し、`NILFS_IOCTL_CLEAN_SEGMENTS` で削除する checkpoint 区間、解放する仮想ブロック番号、移動するブロック、解放するセグメントを一括してカーネルに渡す。 | `nilfs2.rst:138-141`、`fs/nilfs2/ioctl.c:850-860`（argv の五区画） | ドキュメントとソース | NILFS2 の DAT は仮想ブロック番号から物理ブロック番号への一層のマッピングで、本プロジェクトは中央マッピングを使うため、どちらにも一層の間接参照がある。違いは、NILFS2 のスナップショットがボリューム全体の checkpoint で、生存判定をユーザー空間のデーモンが行う点。 |
| セグメント選定方針：`selection_policy` は `timestamp`（最古から最新の順で回収）のみ対応。`nsegments_per_clean` の既定値は 2、`protection_period` は 3600 秒で、「NILFS2 GC never deletes checkpoints and their blocks whose duration time is less than the value」。 | nilfs-utils `man/nilfs_cleanerd.conf.5:22-26`、`:52-58` | ドキュメント精読 | NILFS2 の DAT は仮想ブロック番号から物理ブロック番号への一層のマッピングで、本プロジェクトは中央マッピングを使うため、どちらにも一層の間接参照がある。違いは、NILFS2 のスナップショットがボリューム全体の checkpoint で、生存判定をユーザー空間のデーモンが行う点。 F2FS は greedy/CB を使うが NILFS2 は時刻だけによる。 |
| 生存判定（ユーザー空間）: ブロックの生存区間 [p_start, p_end) はチェックポイント番号；p_end が NILFS_CNO_MAX または保護期間内なら生存、区間内にスナップショットがあれば生存 | nilfs-utils lib/gc.c:417-470 | ソース精読 | DAT 仮想ブロック番号マッピング vs 本プロジェクト中央マッピング |
| カーネル側の原子性：`nilfs_clean_segments` はまず DAT を shadow map に保存し、準備に失敗すれば shadow から復元する。その後 `SC_LSEG_SR`（super root 付きの完全なログ）でセグメントを書き、同じセグメント構築の SUFILE 段階でセグメントを解放する（`nilfs_free_segments`）。構築に失敗したら `nilfs_sufile_cancel_freev` で取り消す。成功後、discard が有効なときだけ、解放したセグメントへ discard を発行する。 | `fs/nilfs2/segment.c:2546-2580`、`:1260-1265`、`:1566-1573`、`:1880-1886` | ソース精読 | NILFS2 の DAT は仮想ブロック番号から物理ブロック番号への一層のマッピングで、本プロジェクトは中央マッピングを使うため、どちらにも一層の間接参照がある。違いは、NILFS2 のスナップショットがボリューム全体の checkpoint で、生存判定をユーザー空間のデーモンが行う点。 |
| 準備ステップを安全に中止できる理由はコメントにある。checkpoint の削除は「can be removed independently」、DAT は「is updated atomically using a copy-on-write technique」、ダーティの印付けは「nondestructive」。 | `fs/nilfs2/ioctl.c:797-834` | コメント精読 | NILFS2 の DAT は仮想ブロック番号から物理ブロック番号への一層のマッピングで、本プロジェクトは中央マッピングを使うため、どちらにも一層の間接参照がある。違いは、NILFS2 のスナップショットがボリューム全体の checkpoint で、生存判定をユーザー空間のデーモンが行う点。 |
| ブロック移動は DAT を変更: nilfs_btree_assign_gc が nilfs_dat_move を呼び出し仮想→物理対応を変更；ディスク上 binfo は仮想番号を保持するため参照元ツリーは修正不要 | fs/nilfs2/btree.c:2354-2379；fs/nilfs2/dat.c:317-331 | ソース精読 | NILFS2 の DAT は仮想ブロック番号から物理ブロック番号への一層のマッピングで、本プロジェクトは中央マッピングを使うため、どちらにも一層の間接参照がある。違いは、NILFS2 のスナップショットがボリューム全体の checkpoint で、生存判定をユーザー空間のデーモンが行う点。 |
| GC 読み込みブロックはダミー inode（gcinode）に保持、新ログ書き出し後に解放；現行世代のダーティブロックと GC 対象は重複しない | fs/nilfs2/gcinode.c:11-22 | コメント精読 | NILFS2 の DAT は仮想ブロック番号から物理ブロック番号への一層のマッピングで、本プロジェクトは中央マッピングを使うため、どちらにも一層の間接参照がある。違いは、NILFS2 のスナップショットがボリューム全体の checkpoint で、生存判定をユーザー空間のデーモンが行う点。 |

見つからなかったもの: F2FS、NILFS2 の双方とも GC 中断クラッシュ後の専用再開ロジックは見当たらず（前者は checkpoint 巻き戻り、後者は super root ログの原子性に依存）。NILFS2 カーネル側にはセグメント選定コードなし（全量ユーザー空間）。

### 10.5 APFS: スナップショットと空間回収に関する公開情報

| 事実 | 出典および日付 | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| スナップショット作成時、現在の extent-reference tree 全体をスナップショットへ移管し、ボリューム側は空の新規ツリーへ置換 | Apple File System Reference（2020-06-22版）55頁 apfs_extentref_tree_oid | 公式ドキュメント | スナップショットごとに1本の extent 参照ツリー + 物理 extent 参照カウント: APFS は物理 extent 単位で参照管理、本プロジェクトは誕生 txg 分割 deadlist；APFS ファイルデータは物理アドレス直接指定 |
| スナップショットメタデータのレコード `j_snap_metadata_val_t` は `extentref_tree_oid`、`sblock_oid` などを持つ。文書の原文は「Snapshots are designed to be fast and inexpensive to create; however, deleting a snapshot involves more work」。 | Apple File System Reference（2020-06-22 版）117～118 ページ | 公式ドキュメント | APFS は物理 extent ごとに参照数を記録し、本プロジェクトは誕生 txg ごとに deadlist を記録する計画。APFS には仮想 oid を物理アドレスに写すオブジェクトマップ（omap）があるが、ファイルデータ extent は物理アドレスを直接使う（102 ページの `j_phys_ext_key_t` の key は物理ブロックアドレス）。本プロジェクトの「すべてのポインタが中央マッピング経由」とは異なる。 |
| 物理 extent レコード `j_phys_ext_val_t` には `refcnt` があり、「The extent can be deleted when its reference count reaches zero」、また「For a volume that has no snapshots, the kind is always APFS_KIND_NEW」。 | Apple File System Reference（2020-06-22 版）102～103 ページ | 公式ドキュメント | APFS は物理 extent ごとに参照数を記録し、本プロジェクトは誕生 txg ごとに deadlist を記録する計画。APFS には仮想 oid を物理アドレスに写すオブジェクトマップ（omap）があるが、ファイルデータ extent は物理アドレスを直接使う（102 ページの `j_phys_ext_key_t` の key は物理ブロックアドレス）。本プロジェクトの「すべてのポインタが中央マッピング経由」とは異なる。 |
| オブジェクトマッピングにはスナップショットツリー（xid → `omap_snapshot_t`）があり、アクセス時のキーは xid。 | Apple File System Reference（2020-06-22 版）17 ページ、47 ページ | 公式ドキュメント | APFS は物理 extent ごとに参照数を記録し、本プロジェクトは誕生 txg ごとに deadlist を記録する計画。APFS には仮想 oid を物理アドレスに写すオブジェクトマップ（omap）があるが、ファイルデータ extent は物理アドレスを直接使う（102 ページの `j_phys_ext_key_t` の key は物理ブロックアドレス）。本プロジェクトの「すべてのポインタが中央マッピング経由」とは異なる。 |
| reaper は「deletion of objects that are too large to be deleted in the time between transactions」を担当し、削除の進捗を複数のトランザクションにまたがって保存する。スナップショット削除は `apfs_reap_state_t`（`last_pbn`、`cur_snap_xid`、`phase`）と omap の二つの reap 段階（先に map tree、後に snapshot tree）を使う。 | Apple File System Reference（2020-06-22 版）26 ページ、50 ページ、166～167 ページ | 公式文書を読んだ（`apfs_reap_state_t` の説明は「No overview available」） | APFS は物理 extent ごとに参照数を記録し、本プロジェクトは誕生 txg ごとに deadlist を記録する計画。APFS には仮想 oid を物理アドレスに写すオブジェクトマップ（omap）があるが、ファイルデータ extent は物理アドレスを直接使う（102 ページの `j_phys_ext_key_t` の key は物理ブロックアドレス）。本プロジェクトの「すべてのポインタが中央マッピング経由」とは異なる。 本プロジェクトで予定する deadlist のマージには reaper のようなトランザクションをまたいで削除するオブジェクトはない。 |
| 空間マネージャは (xid, paddr) をキーとする free queue を保持、spaceman_free_queue_t は sfq_oldest_xid を保持；再割り当て可能時期の記述はなし | 同 159-160頁 | 公式ドキュメント | 手がかり |
| パーティション縮小は evict-mapping tree で「where the blocks are being moved to」（ブロックの移動先）を記録する。 | Apple File System Reference（2020-06-22 版）33 ページ、42～43 ページ | 公式ドキュメント | 違いは不明であり、手がかりとしてしか扱えない |
| 二次情報（Apple 以外）：非常に小さいスナップショットの削除は「may take around 0.2 seconds, mainly with cleaning up the object map」で、コンテナ唯一の Reaper が行う。 | Howard Oakley、「APFS: Snapshots」、eclecticlight.co、2024-04-08（WebFetch の要約経由で、原文との逐語照合はしていない） | 二次記事 | 違いは不明であり、手がかりとしてしか扱えない |

調べたが見つからなかったもの：Apple の参考マニュアル全文に `grep -c -i trim` をかけると 7 行一致したが、すべて `INODE_ACTIVE_FILE_TRIMMED`（「overprovisioning file that has been trimmed」）とその索引であり、TRIM をいつ発行するか、ブロックの再割り当てとの前後関係を説明する箇所はない。マニュアルはスナップショット削除でどの extent を解放するかの決め方、新しいスナップショットと古いものを削除する際の解放量の違いも説明していない。Hansen と Toolan のリバースエンジニアリング論文（Digital Investigation、2017）は今回原文を取得できなかった。Apple 開発者サイトにもスナップショット空間回収を説明する別の公式文書は見つからなかった（ウェブ検索は一回だけ）。

### 10.6 discard/TRIM: 発行時期とブロック再割り当て可能時期の順序

| 事実 | 出典 | 計測条件 | 本プロジェクトとの差異 |
|---|---|---|---|
| ZFS 解放ブロックは ms_defer[txg % TXG_DEFER_SIZE]（TXG_DEFER_SIZE = 2）へ投入；満了で ms_allocatable へ戻ると同時に autotrim 有効時は ms_trim へ追加；slop 不足や vdev 撤退時は defer せず直接 ms_trim へ | module/zfs/metaslab.c:4616-4667；include/sys/txg.h:39 | ソース精読 | 再割り当て可能化が先、TRIM は後、割り当て時に TRIM 待機集合から除外: ZFS は metaslab 単位、本プロジェクトは全空セグメント生成後に discard 発行 |
| 割り当て時に該当範囲を ms_trim から削除するため、再割り当て済み範囲が再 TRIM されることはない | metaslab.c:4928-4929、:5945-5946 | ソース精読 | ZFS は metaslab 単位、本プロジェクトは全空セグメント生成後に discard 発行 |
| autotrim スレッド（トップレベル vdev ごとに1本）は metaslab 処理時に割り当てを一時停止し、ms_trim を空ツリーと交換して TRIM を発行；同一 metaslab は zfs_trim_txg_batch（デフォルト 32）txg 以上の間隔；<= 32KiB はスキップ | module/zfs/vdev_trim.c:1257-1305、:54-78；metaslab.c:6304-6311 | ソース精読 | ZFS は metaslab 単位、本プロジェクトは全空セグメント生成後に discard 発行 |
| 手動 zpool trim: リーフ vdev ごとに1スレッド、全 metaslab の未割り当て範囲を走査、trim 中は割り当て不可；進捗永続化、再開可能 | vdev_trim.c:40-52、:71-72 | コメント精読 | ZFS は metaslab 単位、本プロジェクトは全空セグメント生成後に discard 発行 |
| autotrim 属性ドキュメント: 解放直後に即時 TRIM するのではなく、バッチ処理のため遅延；デフォルト off | man/man7/zpoolprops.7:444-458 | ドキュメント精読 | ZFS は metaslab 単位、本プロジェクトは全空セグメント生成後に discard 発行 |
| btrfs discard=sync: トランザクション完了 btrfs_finish_extent_commit で pinned 範囲に対し btrfs_discard_extent 実行後に unpin_extent_range で再割り当て可能化；同期的 discard は再割り当て前 | fs/btrfs/extent-tree.c:2998-3013 | ソース精読 | btrfs はブロックグループのメモリ内 free space cache で管理、本プロジェクトはセグメント単位 |
| btrfs discard=async: discard 状態はメモリ内 free space cache のみ存在し非永続化；空き時間 LRU 順、初期遅延 BTRFS_DISCARD_DELAY 120秒、全空グループ 10秒；最大 IOPS 1000 | fs/btrfs/discard.c:16-61 | コメントとソース | btrfs はブロックグループのメモリ内 free space cache で管理、本プロジェクトはセグメント単位 |
| btrfs async の順序: unpin された範囲が先に再割り当て可能へ復帰し、後に discard worker が処理；処理時に unlink_free_space で排他し discard 後 TRIMMED で復帰；割り当て時に trimmed ビットマップをリセット | extent-tree.c:3052-3054；free-space-cache.c:3768、:3794、:3650-3710；discard.c:44-52 | ソース精読 | btrfs はブロックグループのメモリ内 free space cache で管理、本プロジェクトはセグメント単位 |
| btrfs ドキュメント: sync discard は重大な性能低下を招き非推奨；async はメモリ内で蓄積；カーネル 6.2 より対応デバイスで discard=async が自動有効化 | btrfs.readthedocs.io Trim.html | ドキュメント精読 | btrfs はブロックグループのメモリ内 free space cache で管理、本プロジェクトはセグメント単位 |
| F2FS: prefree セグメントは checkpoint 成功後に free へ変化し discard 発行；ブロック割り当て時に未発行 discard が重なれば除外、発行中なら完了を待機 | fs/f2fs/segment.c:2223-2282、:1874-1917、:3933 | ソース精読 | セグメント回収形態は本プロジェクトと類似；F2FS はスナップショットなし |
| NILFS2: GC 解放セグメントは super root 付きログの書き込み成功後に discard 発行 | fs/nilfs2/segment.c:2560-2580 | ソース精読 | セグメント回収形態は本プロジェクトと類似 |
| bcachefs: bucket が空になった時点で trigger が need_discard を need_discard btree に設定；discard worker が TRIM 完了後にフラグを解除し freespace btree へ投入；ジャーナル番号が flushed_seq_ondisk 以下かつ rewind_seq_ondisk 未満であることを要求 | alloc/background.c:1291-1300、:56-76；alloc/discard.c:343-351 | ソース精読 | bcachefs は「discard が先、再割り当て可能化が後」、ZFS/btrfs async と逆 |
| APFS: 公式マニュアルに TRIM 仕様の記述なし | APFS マニュアルでの TRIM の扱いは 10.5 末尾「調べたが見つからなかったもの」を参照 | ドキュメント精読 | 不明 |

見つからなかったもの: ZFS、btrfs 双方とも discard 未完了時のクラッシュ復旧に関する記述なし；各社の discard 遅延・スループットの実測値なし。

### 10.7 反証リスト

| 反証 | 調査対象 | 調査方法 |
|---|---|---|
| 調査した全実装において、「誕生 txg 分割の deadlist」と「スナップショット共有ブロックを移動可能なセグメントクリーニング」を同一実装で両立させた例は存在しない | ZFS（deadlist あり、汎用 GC なし、device removal は indirect mapping）；btrfs（extent 参照カウント共有、スナップショット削除はツリー走査減算）；bcachefs（スナップショット ID によるキー可視性共有、スナップショット削除はキー移動）；F2FS（スナップショットなし）；NILFS2（DAT にチェックポイント区間記録、生存はスナップショット比較判定）；APFS（スナップショットごとの extent 参照ツリー + 物理 extent refcnt、GC 非公開） | 10.1〜10.5 のソースおよびドキュメント精読；「存在しない」は調査した6実装の指定コミット範囲内 |
| OpenZFS master にはスナップショットから参照されるブロックポインタを書き換えるメカニズムが存在しない: ソース自認で「we cannot rewrite block pointers of snapshots」；zfs rewrite は共有ブロックを別位置へ書き直し旧複製をスナップショットに残す | ZFS のみ | vdev_indirect.c:36-40；zfs-rewrite.8:52-57；bp rewrite 関連 grep で当該コメントのみヒット |
| 調査した全実装において、各参照元のポインタを変更せずにスナップショット共有データを移動する手法はすべて間接層を導入している: NILFS2 は DAT 変更（仮想→物理）、ZFS device removal は indirect mapping、btrfs remap tree はブロックグループ単位再マッピング、bcachefs は元々単一キー共有；全参照元ポインタを直接書き換えるのは btrfs の従来型 relocation のみであり、RFC で「低速になり得る」と自認 | ZFS、btrfs、bcachefs、NILFS2；F2FS は非該当；APFS は不明 | 10.1〜10.4 のソースおよび RFC 原文 |
| ZFS および btrfs async は「ブロックが先に再割り当て可能となり、後に TRIM 発行；割り当て時に待機集合から除外」；bcachefs は逆で「先に discard、後に freespace 投入」 | ZFS、btrfs、bcachefs；F2FS は checkpoint 後発行；NILFS2 はログコミット後発行 | 10.6 のソース |

計測条件補記: LWN 上の remap tree RFC および eclecticlight の APFS 記事の引用は WebFetch 要約によるものであり原文メールとの行単位突合は未実施；Delphix ブログは web.archive.org の 2020年魚拓を使用。

### 7.9 第7節（「直近10年の学術成果スキャン（2012–2026）」）の参照元

- [The Full Path to Full-Path Indexing (FAST 2018)](https://www.usenix.org/conference/fast18/presentation/zhan)
- [Copy-on-Abundant-Write for Nimble File System Clones (TOS 2021) / How to Copy Files (FAST 2020)](https://www.usenix.org/conference/fast20/presentation/zhan)
- [File Systems Fated for Senescence? Nonsense, Says Science! (FAST 2017)](https://www.usenix.org/conference/fast17/technical-sessions/presentation/conway)
- [SplinterDB: Closing the Bandwidth Gap for NVMe Key-Value Stores (ATC 2020)](https://www.usenix.org/conference/atc20/presentation/conway)
- [Closing the B+-tree vs. LSM-tree Write Amplification Gap (FAST 2022)](https://www.usenix.org/conference/fast22/presentation/qiao)
- [Rethinking File Mapping for Persistent Memory (FAST 2021)](https://www.usenix.org/conference/fast21/presentation/neal)
- [TABLEFS: Enhancing Metadata Efficiency in the Local File System (ATC 2013)](https://www.usenix.org/conference/atc13/technical-sessions/presentation/ren)
- [Push-Button Verification of File Systems via Crash Refinement (OSDI 2016)](https://www.usenix.org/conference/osdi16/technical-sessions/presentation/sigurbjarnarson)
- [Finding Crash-Consistency Bugs with Bounded Black-Box Crash Testing (OSDI 2018)](https://www.usenix.org/conference/osdi18/presentation/mohan)
- [Chipmunk: Investigating Crash-Consistency in Persistent-Memory File Systems (EuroSys 2023)](https://dl.acm.org/doi/10.1145/3552326.3567498)
- [Metis: File System Model Checking via Versatile Input and State Exploration (FAST 2024)](https://www.usenix.org/conference/fast24/presentation/liu-yifei)
- [GoJournal: a verified, concurrent, crash-safe journaling system (OSDI 2021)](https://www.usenix.org/conference/osdi21/presentation/chajed)
- [Verifying the DaisyNFS concurrent and crash-safe file system with sequential reasoning (OSDI 2022)](https://www.usenix.org/conference/osdi22/presentation/chajed)
- [Optimistic Crash Consistency (SOSP 2013)](https://research.cs.wisc.edu/adsl/Publications/optfs-sosp13.pdf)
- [Consistency Without Ordering (FAST 2012)](https://www.usenix.org/conference/fast12/consistency-without-ordering)
- [High Velocity Kernel File Systems with Bento (FAST 2021)](https://www.usenix.org/publications/loginonline/high-velocity-kernel-file-systems-bento)
- [NVMe Flexible Data Placement Overview (NVM Express, FMS 2023)](https://nvmexpress.org/wp-content/uploads/FMS-2023-Flexible-Data-Placement-FDP-Overview.pdf)
- [Bcachefs removed from the mainline kernel (LWN)](https://lwn.net/Articles/1040120/)

---
