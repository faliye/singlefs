# 用語改名対応表

[中文](term-renames.md) · [English](term-renames.en.md) · **日本語**

リポジトリ全体で改名された概念の、旧名と新名の唯一の権威ある対応表。**他の場所では例外なく新名を使用する**——`scripts/kb/sweep-term.py --check` がリポジトリ全体を走査し、`.claude/term-rename-exempt` に登録された箇所以外で旧名が出現した場合は不合格とする。

本文書自体がその免除表に登録されている：「X を Y に改名した」という文を記述するには X を記述できなければならず、**旧名が検索可能な場所が1箇所存在することで、過去の成果物や古いコミットのフィールドと照合可能になる**。特定の旧名の現在の名称を調べる場合は本文書のみを参照すること；改名当日にどのファイルがなぜ変更されたかを調べる場合は、決定変更履歴の該当日のエントリを参照すること。

## スーパーブロック → システム構成（2026-09-20 以降、英語識別子および凍結ディレクトリは 2026-09-21）

ユーザー確定：ディスク上の 481 バイトの構造は「スーパーブロック」とは呼ばない——本プロジェクトのインデックス木は通常の木であり、喪失しても再構築可能であって、「スーパー」なものは存在しない；その構造は**システム構成**である。2026-09-21 確定：英語識別子、ファイル名、および凍結ディレクトリを一括して変更、「プロジェクト全体から検索されなくなるまで」。

## マイルストーンに基づく名称 → 内容に基づく名称（2026-09-27 以降）

ユーザー確定：テスト、モジュール、テストハーネス、ゲート、および kb 文書をマイルストーンに基づいて命名してはならない（「マイルストーンでのリネームはひどい決定だと思う、これは修正すべきだ」「すべて変更した」）、内容に基づいて命名する：最初の書き込みストリームは mkfs 直後の空プール上に新規ファイルを1つ作成するもので、「新プール新規ファイル作成」（`new_pool_file_creation`）と呼ぶ；2番目の書き込みストリームは同一ファイルへの上書き（`file_overwrite`）である；マイルストーン2が実装したのは上書き、解放、ロールバック、および再利用である；マイルストーン3は当日クラッシュ検証やり直しと命名され、現在の名称は「削除と複数回 COW」（対応表の該当行に改名の経緯を記載）。マイルストーン自体の中国語名も一括変更（ユーザーが同日「一緒に変更」を選択）。`.claude/kb/milestone/` および `.claude/kb/layout/` 下のマイルストーン番号順の文書名（`01-first-txn.md` など）は変更しない（ユーザーが同日「命名は正しい」と確定）、そのためハイフン付き略称は具体的な用途ごとにのみ登録する；付随して変更されたファイル名、ゲート名、テストファイル名は `records/2026-09-27-验证两档拆分.md` の「第2陣」の節に対応表がある。

## 層 0 → クラッシュポイントリプレイ（2026-10-12 以降）

ユーザーが 2026-10-11 に「層 0」から「クラッシュポイントリプレイ」への改名が一部しか行われていないことを指摘し、リポジトリ全体で統一して置き換えることを決定：「層 0 クラッシュポイントリプレイ」の重複使用は「層 0」を削除；「層 0 全量 / 高速層」は「クラッシュポイントリプレイ全量 / 高速層」と記述；「層 0 ケース」は「クラッシュ列挙ケース」と記述；英語識別子（`layer0_*`、ゲート名 `checker-tier-layer0-replay`）は当時は変更せず、2026-10-13 のバッチで変更され、末尾の「第1ストリーム / 第2ストリーム、2段パイプライン、3層テーブルの層 → 内容に基づく名称（2026-10-13 以降）」節に登録された。変更しない2つの分類は `.claude/term-rename-exempt` に従って登録：実験装置、保持成果物、変異名、ゲートフィクスチャ（ファイル全体）；コード内の文字列および原文引用の古いテキスト（元の第3分類、D13（検証ロードマップ）確定項 9 の階層表で層 1、層 2 と並んでいた「層 0」は、階層表の3段階がスモーク層、通常層、高負荷層に変更されたことに伴い対象が存在しなくなった）。

## 第1ストリーム / 第2ストリーム、2段パイプライン、3層テーブルの層 → 内容に基づく名称（2026-10-13 以降）

ユーザー確定：レポートや文書内で序数に基づく名称（「第1ストリーム」「第2ストリーム」「第1段」「第2段」「層 0」「層 1」「層 2」）は何を指しているか分からず、リポジトリ全体で内容を説明する名称に置き換える。一括置換は第三者コードを免除；ダイアログで「提案された名前に変更」を選択し、「例えば、以前実行していたのは最初のトランザクションの新規 COW であり、現在は2番目のトランザクションの複数回 COW を実行している。このような記述のほうが明確でシンプルだと思う」「もちろんこの命名は実際の状況に応じて調整してよい」と述べた。
クラッシュポイントリプレイの2つのワークロード：第1ストリーム → 新プール新規ファイル作成ストリーム、第2ストリーム → 上書き固定スクリプトストリーム（そのスクリプトは上書き、解放、再オープン書き込み行、ウォームアップ、ロールバック、F引き上げ、再利用を各1回実行し、Eまで至る）。
クラッシュ増幅の2段パイプライン：第1段 → 登録段（ストリームを先にストアに保存）、第2段 → 判定段（ストアから読み戻して判定、GPU または CPU 上で実行）。このペアはテーブル行に登録せずゲートも判定しない：同一の単語が実験登録の区分、実行者による区分ごとの引き渡し、セグメントシーケンス、KV 実装の区分において別の量を指しており、クラークが kb と記録で314行を行単位で判定したところ、真にパイプラインを指す行は 0 行であり、文脈に応じて書かれた2つの正規表現がヒットした19箇所はすべて誤検知であった；対象はスクリプト内のコメントと出力テキストのみにあり（scripts/crash の3つは置換済み）、今後は新しい名称で記述する。
D13（検証ロードマップ）確定項 9 の3層テーブルの3段階：層 0 スモーク → スモーク層、層 1 通常 → 通常層、層 2 全量 → 高負荷層；散在する「層 0」は 2026-10-12 の節に従ってクラッシュポイントリプレイに変更。英語識別子（layer0_enumeration、checker-tier-layer0-replay、record など）はこのラウンドでは変更しない。
英語識別子改名のバッチ（`records/2026-10-13-复用键与全仓改名.md#3`、依存テーブル第8行）により、前文で変更しなかった英語識別子も変更された。ユーザーの言葉「関数名に layer0 や類似の名前を付けてはならない、機能と役割が識別できなければならない」：crates、スクリプトとゲート、文書の3経路ですべて変更され、旧名は下記の表に1つずつ登録された（単体の単語 `layer0`、`LAYER0`、`Layer0` は登録しない；実験装置、保持成果物、および原文引用内のものはそのまま維持し、`.claude/term-rename-exempt` に登録）。このバッチの行はすべて正規表現で記述：名前はリテラルで開始し、前後に ASCII 文字、数字、アンダースコアが隣接しない（ハイフン付きの名前はハイフンも単語内の文字とみなす）、単語境界と同様に判定し、隣接する漢字や仮名にもマッチする。`\b` で開始しない理由は、`\b` で始まる行がリポジトリ全体の走査を大幅に遅延させるためである：E178（葉コンテナとレコード死亡カウント）成果物 `e178-leaf-and-record-death-counts-segment-two.out` の最初の 2,000,000 文字において、従来の56行は0.19秒、この234行のリテラル開始を追加した場合は0.71秒、単語境界で記述した場合は2.96秒を要した。

<!-- term-renames:table -->
1行に1つのペア、`scripts/kb/sweep-term.py` はこの表に従って置換・検証を行う；**1行追加すると自動的にゲート管理の対象となる**。
「マッチ」は `完全一致`、`単語境界`、または `正規表現` を記述：`単語境界` は略語に使用し、他の単語内の文字への誤マッチを防止する；同一の文字列が他の場所で別の量を表す場合は `正規表現` を記述し、「旧」列にその概念の表現のみにマッチする Python 正規表現を記述（例：`(?<![每第内外里叶二])(?<![0-9] )层 1(?![0-9])` は「層 1 通常」にマッチし、「各層 134」「第 1 層 2 個」「外層 16.33 ms」にはマッチしない）、パイプは `\|` と記述し、キャプチャグループは使用禁止、「新」列はリテラルで置換する。制限事項の詳細は `scripts/kb/sweep-term.py` のファイル頭部を参照。長いパターンを前に記述し、短いパターンが長いパターンを途中で消費することを防止する。

| 旧 | 新 | マッチ | 対象 |
|---|---|---|---|
| 超级块 | 系统配置 | 完全一致 | 中国語用語 |
| SUPER_BLOCK | SYSTEM_CONFIGURATION | 完全一致 | アンダースコア付き定数名 |
| SUPERBLOCK | SYSTEM_CONFIGURATION | 完全一致 | 定数名 |
| Super_Block | SystemConfiguration | 完全一致 | アンダースコア付きアッパーキャメルケース |
| SuperBlock | SystemConfiguration | 完全一致 | アッパーキャメルケース、型名 |
| Superblock | SystemConfiguration | 完全一致 | b が小文字のアッパーキャメルケース |
| superBlock | systemConfiguration | 完全一致 | ローワーキャメルケース |
| super_block | system_configuration | 完全一致 | アンダースコア付きスネークケース |
| super-block | system-configuration | 完全一致 | ケバブケース（バイナリ名、成果物ファイル名） |
| superblock | system_configuration | 完全一致 | スネークケース識別子、モジュール名、ファイル名 |
| sb_ | system_configuration_ | 単語境界 | 接頭辞略語 (sb_mac) |
| _sb | _system_configuration | 単語境界 | 接尾辞略語 (tail_sb) |
| sb | system_configuration | 単語境界 | 単体略語。sb は元々 superblock のために設けられたものであり、概念削除に伴い略語も削除、同日 SOP 略語表から行削除 |
| harness 耗时用例 | harness 慢档用例 | 完全一致 | ハーネス層でデバッグ下で60秒以上実行され、`#[ignore]` が付与されたケース；ユーザーが 2026-10-01 に「高速層 / 低速層」と定め、.claude/rules/test-script-discipline.md と同名 |
| 耗时用例 | 慢档用例 | 完全一致 | 「harness 耗时用例」の harness 接頭辞なしの短縮名 |
| 轻用例 | 快档用例 | 完全一致 | ハーネス層で `#[ignore]` が付与されず、変更ごとに実行されるケース；低速層用例と同日に改名 |
| harness 重档 | harness 慢档用例 | 完全一致 | ハーネス層でシングルスレッドで60秒フル稼働するケース；重型テストと区別するため改名 |
| 里程碑三（名字待定） | 里程碑「删除与多次 COW」 | 完全一致 | マイルストーン名。当初は「クラッシュ検証やり直し」と命名され、コミット 4e8b0370（2026-09-29）以降、クラッシュ検証部分はマイルストーン「上書き、解放、ロールバック、および再利用」追補4に統合され、マイルストーン3は「削除と複数回 COW」に改名 |
| 第二个事务 | 覆盖写、释放、回退与复用 | 完全一致 | マイルストーン名、およびその書き込みストリームの呼称 |
| 第一个事务 | 新池新建文件 | 完全一致 | マイルストーン名、およびその書き込みストリームの呼称 |
| FIRST_TRANSACTION | NEW_POOL_FILE_CREATION | 完全一致 | 定数名 |
| FirstTransaction | NewPoolFileCreation | 完全一致 | アッパーキャメルケース、型名および列挙バリアント |
| first_transaction | new_pool_file_creation | 完全一致 | スネークケース識別子、モジュール名、ファイル名 |
| first-transaction | new-pool-file-creation | 完全一致 | ケバブケース（実行モード名、ステージファイル名） |
| SECOND_TRANSACTION | FILE_OVERWRITE | 完全一致 | 定数名 |
| SecondTransaction | FileOverwrite | 完全一致 | アッパーキャメルケース、列挙バリアント |
| second_transaction | file_overwrite | 完全一致 | スネークケース識別子、メトリクス名 |
| second-transaction | file-overwrite | 完全一致 | ケバブケース（実行モード名） |
| first_txn | new_pool_file_creation | 完全一致 | 短縮スネークケース |
| e142-first-txn-dry-run | e142-new-pool-file-creation-dry-run | 完全一致 | 成果物ファイル名接頭辞 |
| 32-first-txn-fields | 32-new-pool-file-creation-fields | 完全一致 | ステージファイル名 |
| 42-first-txn-trio | 42-new-pool-file-creation-trio | 完全一致 | ステージファイル名 |
| 76-second-txn-hooks | 76-overwrite-release-rollback-reuse-hooks | 完全一致 | ステージファイル名 |
| first-txn-journal-implies-units | new-pool-file-creation-journal-implies-units | 完全一致 | リトマスファイル名 |
| first-txn-root-implies-units | new-pool-file-creation-root-implies-units | 完全一致 | リトマスファイル名 |
| formatted-layer0-same-as-first-txn | formatted-layer0-same-as-new-pool-file-creation | 完全一致 | ラベル |
| formatted-layer0-first-txn-reference | formatted-layer0-new-pool-file-creation-reference | 完全一致 | ラベル |
| second_txn | overwrite_release_rollback_reuse | 完全一致 | 短縮スネークケース |
| third_txn | crash_verification_rebuild | 完全一致 | 短縮スネークケース |
| 按条记录 | 判定库记录 | 完全一致 | crates 変異表の判定ライブラリ内レコード |
| 判法指纹 | 判法单元指纹 | 完全一致 | crates 変異表判定ライブラリディレクトリのダイジェスト（一時的に判定バージョンと呼ばれた） |
| 底座指纹 | 判法单元指纹 | 完全一致 | 判定指紋のより初期の呼称 |
| 判法版本 | 判法单元指纹 | 完全一致 | 2026-10-15 以降、ライブラリ全体の判定バージョンは廃止；関数ユニットごとの指紋が各判定のユニットキーに入る |
| judging-fingerprint | judge-version | 完全一致 | `crates-mutation-rows.py` のサブコマンド |
| judging_fingerprint | judge_version | 完全一致 | 関数名およびレコードフィールド |
| lib_item_verdicts | lib_verdict_library | 完全一致 | ファイル改名（`.claude/rules/path-moves.md` のパス移転）、用語として一括照合 |
| GATE_FORMAL_VERDICT_STORE | GATE_FORMAL_VERDICT_LIBRARY | 完全一致 | 形式検証ゲートの環境変数 |
| store-in-tmpdir | library-in-tmpdir | 完全一致 | `FORMAL_VERIFICATION_BREAK` の項目 |
| 快速档 | 快档 | 完全一致 | E165 の区分；ハーネス低速層と並ぶ高速・低速の呼称 |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?0(?![0-9.])(?! ?冒烟)` | 崩溃点重放 | 正規表現 | 中国語用語；「第 1 層 0」「各層 0」などの木の階層や D13「層 0 スモーク」にはマッチしない |
| 第一条流 | 新池新建文件流 | 完全一致 | 中国語用語：クラッシュポイントリプレイの第1ワークロード |
| 第二条流 | 覆盖写固定脚本流 | 完全一致 | 中国語用語：クラッシュポイントリプレイの第2ワークロード（Eまでの固定スクリプト） |
| 层 0 冒烟 | 冒烟层 | 完全一致 | D13 確定項 9 の3層テーブル第1段階のラベル |
| 层 1 常规 | 常规层 | 完全一致 | 同テーブル第2段階のラベル |
| 层 2 全量 | 大负载层 | 完全一致 | 同テーブル第3段階のラベル |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?1(?![0-9.])(?! ?常规)` | 常规层 | 正規表現 | 中国語用語：3層テーブル第2段階の単独表記時；木の階層にはマッチしない |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?2(?![0-9.])(?! ?全量)` | 大负载层 | 正規表現 | 中国語用語：3層テーブル第3段階の単独表記時；木の階層にはマッチしない |
| `checker-tier-layer0-replay(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]checker-tier-layer0-replay)` | checker-tier-crash-point-replay | 正規表現 | ゲート名、ステージファイル名、フィクスチャディレクトリ名 |
| `layer0-shard-configuration-check(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shard-configuration-check)` | crash-point-replay-shard-configuration-check | 正規表現 | 2台シャーディング構成判定スクリプト名 |
| `layer0-shard-run(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shard-run)` | crash-point-replay-shard-run | 正規表現 | クラッシュポイントリプレイ2台シャーディングラッパースクリプト名 |
| `layer0-sample-tools(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-sample-tools)` | crash-point-replay-sample-tools | 正規表現 | ゲートフィクスチャルート内のダミーツールディレクトリ名 (`.layer0-sample-tools`) |
| `singlefs-layer0-logs(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]singlefs-layer0-logs)` | singlefs-crash-point-replay-logs | 正規表現 | ゲートの common-dir 内ログディレクトリ名 |
| `singlefs-layer0-shard(?![A-Za-z0-9_])(?<![A-Za-z0-9_]singlefs-layer0-shard)` | singlefs-crash-point-replay-shard | 正規表現 | 2台シャーディングラッパーの一時ディレクトリ接頭辞 |
| src/layer0_library.rs | src/crash_library.rs | 完全一致 | `Cargo.toml` コメント内で参照される判定ライブラリソースファイル |
| `layer0_enumeration(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration)` | crash_state_enumeration | 正規表現 | モジュール名、`.rs` ファイル名、判定ライブラリクラス名、マニフェスト種別 |
| `layer0_progress(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_progress)` | crash_point_replay_progress | 正規表現 | モジュール名および `.rs` ファイル名 |
| `layer0-enumeration-library(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-enumeration-library)` | crash-state-verdicts | 正規表現 | 判定ライブラリ内のクラッシュ列挙ケース用サブディレクトリ名 |
| `layer0-replay(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-replay)` | crash-case-replay | 正規表現 | クラッシュポイントリプレイゲートのセル名 |
| `LAYER0_CELL_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_CELL_NAME)` | CRASH_CASE_REPLAY_CELL_NAME | 正規表現 | ステージ内のセル名定数 |
| `LAYER0_CELL_TITLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_CELL_TITLE)` | CRASH_CASE_REPLAY_CELL_TITLE | 正規表現 | ステージ内のセルタイトル定数 |
| `LAYER0_FEATURE_CELL_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FEATURE_CELL_NAME)` | FEATURE_GATED_TESTS_CELL_NAME | 正規表現 | ステージ内の feature-gated-tests セル名定数 |
| `LAYER0_FEATURE_CELL_TITLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FEATURE_CELL_TITLE)` | FEATURE_GATED_TESTS_CELL_TITLE | 正規表現 | ステージ内の feature-gated-tests セルタイトル |
| `layer0-stage(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-stage)` | crash-point-replay-stage | 正規表現 | 重型テストガード内のステージ種別名 (`lib_heavy_tests.py` STAGE_KIND) |
| `stage_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]stage_layer0)` | stage_crash_point_replay | 正規表現 | 重型テストガードセルフテスト内のステージ名 |
| `layer0_admission_module(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_admission_module)` | crash_point_replay_admission_module | 正規表現 | ステージ変数 |
| `layer0_break(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_break)` | crash_point_replay_break | 正規表現 | ステージ変数 |
| `layer0_cell_state(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_cell_state)` | crash_point_replay_cell_state | 正規表現 | ステージ変数 |
| `layer0_check_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_check_cells)` | crash_point_replay_check_cells | 正規表現 | ステージ変数 |
| `layer0_found(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_found)` | crash_point_replay_found | 正規表现 | ステージ変数 |
| `layer0_full_base(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_full_base)` | crash_point_replay_full_base | 正規表現 | ステージ変数 |
| `layer0_full_rc(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_full_rc)` | crash_point_replay_full_rc | 正規表現 | ステージ変数 |
| `layer0_input_paths_text(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_input_paths_text)` | crash_point_replay_input_paths_text | 正規表現 | ステージ変数 |
| `layer0_input_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_input_table)` | crash_point_replay_input_table | 正規表現 | ステージ変数 |
| `layer0_judge_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_judge_paths)` | crash_point_replay_judge_paths | 正規表現 | ステージ変数 |
| `layer0_kept_rows(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_kept_rows)` | crash_point_replay_kept_rows | 正規表現 | ステージ変数 |
| `layer0_list_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_cells)` | crash_point_replay_list_cells | 正規表現 | ステージ変数 |
| `layer0_list_items(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_items)` | crash_point_replay_list_items | 正規表現 | ステージ変数 |
| `layer0_list_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_table)` | crash_point_replay_list_table | 正規表現 | ステージ変数 |
| `layer0_log_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_log_directory)` | crash_point_replay_log_directory | 正規表現 | ステージ変数 |
| `layer0_named_cases(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_cases)` | crash_point_replay_named_cases | 正規表現 | ステージ変数 |
| `layer0_named_case(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_case)` | crash_point_replay_named_case | 正規表現 | ステージ変数 |
| `layer0_named_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_value)` | crash_point_replay_named_value | 正規表現 | ステージ変数 |
| `layer0_registered_input_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_registered_input_paths)` | crash_point_replay_registered_input_paths | 正規表現 | ステージ変数 |
| `layer0_registered_paths_text(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_registered_paths_text)` | crash_point_replay_registered_paths_text | 正規表現 | ステージ変数 |
| `layer0_report_not_run(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_report_not_run)` | crash_point_replay_report_not_run | 正規表現 | ステージ変数 |
| `layer0_resolved_cargo(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_resolved_cargo)` | crash_point_replay_resolved_cargo | 正規表現 | ステージ変数 |
| `layer0_runs_cell(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_runs_cell)` | crash_point_replay_runs_cell | 正規表現 | ステージ変数 |
| `layer0_sample_mode(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_mode)` | crash_point_replay_sample_mode | 正規表現 | ステージ変数 |
| `layer0_sample_tool_mode(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_mode)` | crash_point_replay_sample_tool_mode | 正規表現 | ステージ変数 |
| `layer0_sample_tool_path(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_path)` | crash_point_replay_sample_tool_path | 正規表現 | ステージ変数 |
| `layer0_sample_tool_problems(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_problems)` | crash_point_replay_sample_tool_problems | 正規表現 | ステージ変数 |
| `layer0_sample_tools_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tools_directory)` | crash_point_replay_sample_tools_directory | 正規表現 | ステージ変数 |
| `layer0_sample_tool(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool)` | crash_point_replay_sample_tool | 正規表現 | ステージ変数 |
| `layer0_scratch_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_scratch_directory)` | crash_point_replay_scratch_directory | 正規表現 | ステージ変数 |
| `layer0_selected_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_selected_cells)` | crash_point_replay_selected_cells | 正規表現 | ステージ変数 |
| `layer0_shard_check_script_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_check_script_directory)` | crash_point_replay_shard_check_script_directory | 正規表現 | ステージ変数 |
| `layer0_shard_configuration_note(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_configuration_note)` | crash_point_replay_shard_configuration_note | 正規表現 | ステージ変数 |
| `layer0_shard_driver(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_driver)` | crash_point_replay_shard_driver | 正規表現 | ステージ変数 |
| `layer0_sharded(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sharded)` | crash_point_replay_sharded | 正規表現 | ステージ変数 |
| `layer0_stage_file_name(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_file_name)` | crash_point_replay_stage_file_name | 正規表現 | ステージ変数 |
| `layer0_stage_repository(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_repository)` | crash_point_replay_stage_repository | 正規表現 | ステージ変数 |
| `layer0_stage_script_path(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_script_path)` | crash_point_replay_stage_script_path | 正規表現 | ステージ変数 |
| `layer0_start_over(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_start_over)` | crash_point_replay_start_over | 正規表現 | ステージ変数 |
| `layer0_static_exit(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_static_exit)` | crash_point_replay_static_exit | 正規表現 | ステージ変数 |
| `layer0_static_judge(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_static_judge)` | crash_point_replay_static_judge | 正規表現 | ステージ変数 |
| `layer0_tier(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tier)` | crash_point_replay_tier | 正規表現 | ステージ変数 |
| `layer0_tree_ready(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tree_ready)` | crash_point_replay_tree_ready | 正規表現 | ステージ変数 |
| `layer0_unknown_cases(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_unknown_cases)` | crash_point_replay_unknown_cases | 正規表現 | ステージ変数 |
| `layer0_unlisted_input_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_unlisted_input_paths)` | crash_point_replay_unlisted_input_paths | 正規表現 | ステージ変数 |
| `Layer0DigitOfASegment(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0DigitOfASegment)` | CrashPointReplayDigitOfASegment | 正規表現 | 型名 |
| `Layer0EnumerationTables(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0EnumerationTables)` | CrashStateEnumerationTables | 正規表現 | 型名 |
| `Layer0Enumeration(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Enumeration)` | CrashStateEnumeration | 正規表現 | 型名 |
| `Layer0FindingsLogBegin(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLogBegin)` | CrashPointReplayFindingsLogBegin | 正規表現 | 型名 |
| `Layer0FindingsLogSection(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLogSection)` | CrashPointReplayFindingsLogSection | 正規表現 | 型名 |
| `Layer0FindingsLog(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLog)` | CrashPointReplayFindingsLog | 正規表現 | 型名 |
| `Layer0FindingSample(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingSample)` | CrashPointReplayFindingSample | 正規表現 | 型名 |
| `Layer0FindingSignature(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingSignature)` | CrashPointReplayFindingSignature | 正規表現 | 型名 |
| `Layer0Findings(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Findings)` | CrashPointReplayFindings | 正規表現 | 型名 |
| `Layer0Finding(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Finding)` | CrashPointReplayFinding | 正規表現 | 型名 |
| `Layer0Item(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Item)` | CrashPointReplayItem | 正規表現 | 型名 |
| `Layer0JudgeRoute(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0JudgeRoute)` | CrashPointReplayJudgeRoute | 正規表現 | 型名 |
| `Layer0ObserverCounts(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0ObserverCounts)` | CrashPointReplayObserverCounts | 正規表現 | 型名 |
| `Layer0Observer(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Observer)` | CrashPointReplayObserver | 正規表現 | 型名 |
| `Layer0OraclesOfThePieces(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OraclesOfThePieces)` | CrashPointReplayOraclesOfThePieces | 正規表現 | 型名 |
| `Layer0Oracles(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Oracles)` | CrashPointReplayOracles | 正規表現 | 型名および列挙バリアント（E171 JudgingPiece） |
| `Layer0OracleViolationKind(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OracleViolationKind)` | CrashPointReplayOracleViolationKind | 正規表現 | 列挙型名 |
| `Layer0OracleViolation(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OracleViolation)` | CrashPointReplayOracleViolation | 正規表現 | 型名 |
| `Layer0PublishOfState(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0PublishOfState)` | CrashPointReplayPublishOfState | 正規表現 | 型名 |
| `Layer0RedPass(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0RedPass)` | CrashPointReplayRedPass | 正規表現 | 列挙型名 |
| `Layer0SegmentExpansion(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentExpansion)` | CrashPointReplaySegmentExpansion | 正規表現 | 型名 |
| `Layer0SegmentOfState(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentOfState)` | CrashPointReplaySegmentOfState | 正規表現 | 型名 |
| `Layer0SegmentSlice(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentSlice)` | CrashPointReplaySegmentSlice | 正規表現 | 型名 |
| `Layer0StateJudgement(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StateJudgement)` | CrashStateJudgement | 正規表現 | 型名 |
| `Layer0StateObserving(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StateObserving)` | CrashStateObserving | 正規表現 | 型名 |
| `Layer0StatePlan(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StatePlan)` | CrashStatePlan | 正規表現 | 型名 |
| `Layer0StatePosition(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StatePosition)` | CrashStatePosition | 正規表現 | 型名 |
| `Layer0Tally(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Tally)` | CrashPointReplayTally | 正規表現 | 型名 |
| `Layer0ToolchainIdentity(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0ToolchainIdentity)` | CrashPointReplayToolchainIdentity | 正規表現 | 型名 |
| `Layer0VersionAtTheRoot(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0VersionAtTheRoot)` | CrashPointReplayVersionAtTheRoot | 正規表現 | 型名 |
| `Layer0VersionTable(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0VersionTable)` | CrashPointReplayVersionTable | 正規表現 | 型名 |
| `enumerate_layer0_quick_tier_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_quick_tier_versions)` | enumerate_crash_states_quick_tier_versions | 正規表現 | 関数名 |
| `enumerate_layer0_selecting_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_selecting_versions)` | enumerate_crash_states_selecting_versions | 正規表現 | 関数名 |
| `enumerate_layer0_selecting(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_selecting)` | enumerate_crash_states_selecting | 正規表現 | 関数名 |
| `enumerate_layer0_through_the_library(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_through_the_library)` | enumerate_crash_states_through_the_library | 正規表現 | 関数名 |
| `enumerate_layer0_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_versions)` | enumerate_crash_states_versions | 正規表現 | 関数名 |
| `enumerate_layer0_with_expansion(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_with_expansion)` | enumerate_crash_states_with_expansion | 正規表現 | 関数名 |
| `enumerate_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0)` | enumerate_crash_states | 正規表現 | 関数名；`crash-case-check.py` がケースを認識する呼び出しでもある |
| `judge_layer0_state_range_of_the_read_back_flow_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_of_the_read_back_flow_with_slices)` | judge_crash_state_range_of_the_read_back_flow_with_slices | 正規表現 | 関数名 |
| `judge_layer0_state_range_of_the_read_back_flow(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_of_the_read_back_flow)` | judge_crash_state_range_of_the_read_back_flow | 正規表現 | 関数名 |
| `judge_layer0_state_range_against_the_version_table_naming_the_publish(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_against_the_version_table_naming_the_publish)` | judge_crash_state_range_against_the_version_table_naming_the_publish | 正規表現 | 関数名（ローカルクロージャ） |
| `judge_layer0_state_range(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range)` | judge_crash_state_range | 正規表現 | 関数名 |
| `layer0_enumeration_tables_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration_tables_with_slices)` | crash_state_enumeration_tables_with_slices | 正規表現 | 関数名 |
| `layer0_enumeration_tables(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration_tables)` | crash_state_enumeration_tables | 正規表現 | 関数名 |
| `layer0_judge_route(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_judge_route)` | crash_point_replay_judge_route | 正規表現 | 関数名 |
| `layer0_ordinal_with_the_same_landing(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_ordinal_with_the_same_landing)` | crash_state_ordinal_with_the_same_landing | 正規表現 | 関数名 |
| `layer0_plan_state_count(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_plan_state_count)` | crash_point_replay_plan_state_count | 正規表現 | 関数名 |
| `layer0_state_count_with_the_torn_third_state(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count_with_the_torn_third_state)` | crash_state_count_with_the_torn_third_state | 正規表現 | 関数名 |
| `layer0_state_count_with_torn_in_place_overwrites(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count_with_torn_in_place_overwrites)` | crash_state_count_with_torn_in_place_overwrites | 正規表現 | 関数名 |
| `layer0_state_count(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count)` | crash_state_count | 正規表現 | 関数名 |
| `layer0_state_ranges_by_segment_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_ranges_by_segment_with_slices)` | crash_state_ranges_by_segment_with_slices | 正規表現 | 関数名 |
| `layer0_state_ranges_by_segment(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_ranges_by_segment)` | crash_state_ranges_by_segment | 正規表現 | 関数名 |
| `layer0_tally_from_the_library(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tally_from_the_library)` | crash_point_replay_tally_from_the_library | 正規表現 | 関数名 |
| `layer0_tally_under_the_expansion_of_the_plan(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tally_under_the_expansion_of_the_plan)` | crash_point_replay_tally_under_the_expansion_of_the_plan | 正規表現 | 関数名 |
| `layer0_worker_threads(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_worker_threads)` | crash_point_replay_worker_threads | 正規表現 | 関数名 |
| `sample_layer0_states_on_the_cpu_with_threads_against_the_version_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu_with_threads_against_the_version_table)` | sample_crash_states_on_the_cpu_with_threads_against_the_version_table | 正規表現 | 関数名 |
| `sample_layer0_states_on_the_cpu_against_the_version_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu_against_the_version_table)` | sample_crash_states_on_the_cpu_against_the_version_table | 正規表現 | 関数名 |
| `sample_layer0_states_on_the_cpu(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu)` | sample_crash_states_on_the_cpu | 正規表現 | 関数名 |
| `visit_layer0_state_range_of_the_read_back_flow_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]visit_layer0_state_range_of_the_read_back_flow_with_slices)` | visit_crash_state_range_of_the_read_back_flow_with_slices | 正規表現 | 関数名 |
| `visit_layer0_state_range(?![A-Za-z0-9_])(?<![A-Za-z0-9_]visit_layer0_state_range)` | visit_crash_state_range | 正規表現 | 関数名（ローカルクロージャ） |
| `synthetic_layer0_log(?![A-Za-z0-9_])(?<![A-Za-z0-9_]synthetic_layer0_log)` | synthetic_crash_point_replay_log | 正規表現 | `admission.py` セルフテストでログを合成する関数 |
| `run_layer0_stage_shard_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]run_layer0_stage_shard_cells)` | run_crash_point_replay_stage_shard_cells | 正規表現 | `admission.py` の関数 |
| `run_layer0_stage_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]run_layer0_stage_cells)` | run_crash_point_replay_stage_cells | 正規表現 | `admission.py` の関数 |
| `layer0_lines(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_lines)` | crash_point_replay_lines | 正規表現 | `admission.py` のローカル名 |
| `LAYER0_FINDING_SAMPLES_KEPT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDING_SAMPLES_KEPT)` | CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT | 正規表現 | 定数名 |
| `LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE)` | CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE | 正規表現 | 定数名 |
| `LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE)` | CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE | 正規表現 | 定数名 |
| `LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE)` | CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE | 正規表現 | 定数名 |
| `LAYER0_RED_PASS_POOL_CHECKER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_POOL_CHECKER)` | CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER | 正規表現 | 定数名 |
| `LAYER0_RED_PASS_RECORD_CHECKER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_RECORD_CHECKER)` | CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER | 正規表現 | 定数名 |
| `LAYER0_SAMPLING_ITEM(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SAMPLING_ITEM)` | CRASH_POINT_REPLAY_SAMPLING_ITEM | 正規表現 | 定数名 |
| `LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE)` | CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE | 正規表現 | 定数名 |
| `READ_FAULTS_OF_A_LAYER0_STATE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]READ_FAULTS_OF_A_LAYER0_STATE)` | READ_FAULTS_OF_A_CRASH_STATE | 正規表現 | 定数名 |
| `LAYER0_STAGE_CASES(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_STAGE_CASES)` | CRASH_POINT_REPLAY_STAGE_CASES | 正規表現 | `admission.py` の定数 |
| `LAYER0_PARALLEL_FINISHED_PREFIX(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_FINISHED_PREFIX)` | CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX | 正規表現 | `admission.py` の定数 |
| `LAYER0_THREADS_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_THREADS_VARIABLE)` | CRASH_POINT_REPLAY_THREADS_VARIABLE | 正規表現 | `admission.py` の定数 |
| `LAYER0_START_OVER_LIBRARY_DIRECTORY_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_START_OVER_LIBRARY_DIRECTORY_NAME)` | CRASH_POINT_REPLAY_START_OVER_LIBRARY_DIRECTORY_NAME | 正規表現 | `admission.py` の定数 |
| `layer0-start-over(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-start-over)` | crash-point-replay-start-over | 正規表現 | 前の定数の値（再開ライブラリ種別のディレクトリ名） |
| `LAYER0_SHARD_CONFIGURATION_KEYS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_CONFIGURATION_KEYS)` | CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS | 正規表現 | 2台シャーディング構成判定の定数 |
| `SINGLEFS_LAYER0_THREADS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_THREADS)` | SINGLEFS_CRASH_POINT_REPLAY_THREADS | 正規表現 | 環境変数 |
| `SINGLEFS_LAYER0_FINDINGS_FILE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_FINDINGS_FILE)` | SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE | 正規表現 | 環境変数 |
| `SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE)` | SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE | 正規表現 | 環境変数 |
| `SINGLEFS_LAYER0_SHARD_PEER_EXIT_PAUSE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_SHARD_PEER_EXIT_PAUSE)` | SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_EXIT_PAUSE | 正規表現 | 環境変数（2台ラッパーセルフテスト） |
| `LAYER0_SHARD_CONFIGURATION_FILE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_CONFIGURATION_FILE)` | CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE | 正規表現 | 構成判定 `--emit-assignments` が出力する変数 |
| `GATE_LAYER0_BREAK(?![A-Za-z0-9_])(?<![A-Za-z0-9_]GATE_LAYER0_BREAK)` | GATE_CRASH_POINT_REPLAY_BREAK | 正規表現 | ゲート破損スイッチ |
| `LAYER0_SHARD_RUN_BREAK(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_RUN_BREAK)` | CRASH_POINT_REPLAY_SHARD_RUN_BREAK | 正規表現 | 2台ラッパー破損スイッチ |
| `LAYER0_SHARD_RUN_TWO_HOSTS_DRIVER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_RUN_TWO_HOSTS_DRIVER)` | CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER | 正規表現 | 2台ラッパーのドライバ変数 |
| `STUB_LAYER0_REUSED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_LAYER0_REUSED)` | STUB_CRASH_POINT_REPLAY_REUSED | 正規表現 | ゲートフィクスチャダミー cargo のスタブ変数 |
| `STUB_LAYER0_EXIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_LAYER0_EXIT)` | STUB_CRASH_POINT_REPLAY_EXIT | 正規表現 | ゲートフィクスチャダミー cargo のスタブ変数 |
| `LAYER0_ACQUISITION_BARRIER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_ACQUISITION_BARRIER)` | CRASH_POINT_REPLAY_ACQUISITION_BARRIER | 正規表現 | カウント行接頭辞 |
| `LAYER0B_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0B_FAST)` | CRASH_POINT_REPLAY_FIXED_SCRIPT_FAST | 正規表現 | 上書き固定スクリプトストリーム高速層のカウント行接頭辞（B は序数代称、接頭辞ごと置換） |
| `LAYER0B(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0B)` | CRASH_POINT_REPLAY_FIXED_SCRIPT | 正規表現 | 上書き固定スクリプトストリームのカウント行接頭辞 |
| `LAYER0_FINDINGS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDINGS)` | CRASH_POINT_REPLAY_FINDINGS | 正規表現 | 発見ログの集約マーカー |
| `LAYER0_FINDING(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDING)` | CRASH_POINT_REPLAY_FINDING | 正規表現 | 発見ログの項目別マーカー |
| `layer0_finding_threshold(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding_threshold)` | crash_point_replay_finding_threshold | 正規表現 | 発見ログの行種別 |
| `layer0_finding_new(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding_new)` | crash_point_replay_finding_new | 正規表現 | 発見ログの行種別 |
| `layer0_findings_begin(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_findings_begin)` | crash_point_replay_findings_begin | 正規表現 | 発見ログの行種別 |
| `layer0_findings_summary(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_findings_summary)` | crash_point_replay_findings_summary | 正規表現 | 発見ログの行種別 |
| `layer0_finding(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding)` | crash_point_replay_finding | 正規表現 | 発見ログの行種別 |
| `LAYER0_FLOW_RECORDED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FLOW_RECORDED)` | CRASH_POINT_REPLAY_FLOW_RECORDED | 正規表現 | ログマーカー |
| `LAYER0_IO_MIN_8192_MOUNT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_IO_MIN_8192_MOUNT)` | CRASH_POINT_REPLAY_IO_MIN_8192_MOUNT | 正規表現 | カウント行接頭辞 |
| `LAYER0_JUDGE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_JUDGE)` | CRASH_POINT_REPLAY_JUDGE | 正規表現 | ログマーカー |
| `LAYER0_PARALLEL_FINISHED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_FINISHED)` | CRASH_POINT_REPLAY_PARALLEL_FINISHED | 正規表現 | ログマーカー |
| `LAYER0_PARALLEL_LINE_ONE_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_LINE_ONE_FAST)` | CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH_FAST | 正規表現 | 複数レコード公開ストリーム高速層のカウント行接頭辞 |
| `LAYER0_PARALLEL_LINE_ONE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_LINE_ONE)` | CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH | 正規表現 | 複数レコード公開ストリームのカウント行接頭辞 |
| `LAYER0_PARALLEL_START(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_START)` | CRASH_POINT_REPLAY_PARALLEL_START | 正規表現 | ログマーカー |
| `LAYER0_POSITION_ADDRESSED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_POSITION_ADDRESSED)` | CRASH_POINT_REPLAY_POSITION_ADDRESSED | 正規表現 | カウント行接頭辞 |
| `LAYER0_PROGRESS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PROGRESS)` | CRASH_POINT_REPLAY_PROGRESS | 正規表現 | 進捗行マーカー |
| `LAYER0_SPILL_OVER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SPILL_OVER)` | CRASH_POINT_REPLAY_SPILL_OVER | 正規表現 | カウント行接頭辞 |
| `LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT | 正規表現 | カウント行接頭辞 |
| `LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT | 正規表現 | カウント行接頭辞 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED | 正規表現 | カウント行接頭辞 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT | 正規表現 | カウント行接頭辞 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED | 正規表現 | カウント行接頭辞 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT | 正規表現 | カウント行接頭辞 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW | 正規表現 | カウント行接頭辞 |
| `LAYER0_TREE_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT | 正規表現 | カウント行接头辞 |
| `LAYER0_UNLINK_AFTER_MULTI_RECORD_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_UNLINK_AFTER_MULTI_RECORD_FAST)` | CRASH_POINT_REPLAY_UNLINK_AFTER_MULTI_RECORD_FAST | 正規表現 | カウント行接頭辞 |
| `LAYER0_UNLINK_AFTER_MULTI_RECORD(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_UNLINK_AFTER_MULTI_RECORD)` | CRASH_POINT_REPLAY_UNLINK_AFTER_MULTI_RECORD | 正規表現 | カウント行接頭辞 |
| `layer0-unlink-after-the-multi-record-publish-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-unlink-after-the-multi-record-publish-stream)` | unlink-after-the-multi-record-publish-stream | 正規表現 | crash-case キー (`crash-case:` の後ろの部分) |
| `layer0-multi-record-publish-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-multi-record-publish-stream)` | multi-record-publish-stream | 正規表現 | crash-case キー |
| `layer0-position-addressed-tree-streams(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-position-addressed-tree-streams)` | position-addressed-tree-streams | 正規表現 | crash-case キー |
| `layer0-tree-split-streams(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-tree-split-streams)` | tree-split-streams | 正規表現 | crash-case キー |
| `layer0-first-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-first-stream)` | new-pool-file-creation-stream | 正規表現 | crash-case キー：新プール新規ファイル作成ストリーム |
| `layer0-second-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-second-stream)` | overwrite-fixed-script-stream | 正規表現 | crash-case キー：上書き固定スクリプトストリーム |
| `a_recording_recovery_that_differs_from_the_layer0_report_is_counted(?![A-Za-z0-9_])(?<![A-Za-z0-9_]a_recording_recovery_that_differs_from_the_layer0_report_is_counted)` | a_recording_recovery_that_differs_from_the_crash_point_replay_report_is_counted | 正規表現 | テスト関数名 |
| `the_state_ordinal_decodes_to_the_same_persisted_set_as_layer0_mixed_radix(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_state_ordinal_decodes_to_the_same_persisted_set_as_layer0_mixed_radix)` | the_state_ordinal_decodes_to_the_same_persisted_set_as_crash_point_replay_mixed_radix | 正規表現 | テスト関数名 |
| `worker_threads_come_from_the_layer0_variable_then_the_checking_threads_variable_then_the_machine(?![A-Za-z0-9_])(?<![A-Za-z0-9_]worker_threads_come_from_the_layer0_variable_then_the_checking_threads_variable_then_the_machine)` | worker_threads_come_from_the_crash_point_replay_variable_then_the_checking_threads_variable_then_the_machine | 正規表現 | テスト関数名 |
| `a_layer0_item_name_cannot_be_a_path_segment_with_a_slash(?![A-Za-z0-9_])(?<![A-Za-z0-9_]a_layer0_item_name_cannot_be_a_path_segment_with_a_slash)` | a_crash_point_replay_item_name_cannot_be_a_path_segment_with_a_slash | 正規表現 | テスト関数名 |
| `layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations)` | crash_point_replay_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations | 正規表現 | テスト関数名（`stage-inputs.tsv` の `test=` 列でもある） |
| `layer0_quick_tier_matches_the_full_tally_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_quick_tier_matches_the_full_tally_shape)` | crash_point_replay_quick_tier_matches_the_full_tally_shape | 正規表現 | テスト関数名 |
| `the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_layer0_tally(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_layer0_tally)` | the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_crash_point_replay_tally | 正規表現 | テスト関数名 |
| `the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape)` | the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_crash_point_replay_shape | 正規表現 | テスト関数名 |
| `acquisition-barrier-layer0(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]acquisition-barrier-layer0)` | acquisition-barrier-crash-point-replay | 正規表現 | プール名 |
| `formatted-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-fast)` | formatted-crash-point-replay-fast | 正規表現 | プール名 |
| `formatted-layer0-new-pool-file-creation-reference(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-new-pool-file-creation-reference)` | formatted-crash-point-replay-new-pool-file-creation-reference | 正規表現 | ラベル |
| `formatted-layer0-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-registered)` | formatted-crash-point-replay-registered | 正規表現 | プール名 |
| `formatted-layer0-same-as-new-pool-file-creation(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-same-as-new-pool-file-creation)` | formatted-crash-point-replay-same-as-new-pool-file-creation | 正規表現 | ラベル |
| `parallel-line-one-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]parallel-line-one-layer0-fast)` | multi-record-publish-crash-point-replay-fast | 正規表現 | プール名 |
| `parallel-line-one-layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]parallel-line-one-layer0-full)` | multi-record-publish-crash-point-replay-full | 正規表現 | プール名 |
| `unlink-after-multi-record-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-fast)` | unlink-after-multi-record-crash-point-replay-fast | 正規表現 | プール名 |
| `unlink-after-multi-record-layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-full)` | unlink-after-multi-record-crash-point-replay-full | 正規表現 | プール名 |
| `unlink-after-multi-record-layer0-rerun(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-rerun)` | unlink-after-multi-record-crash-point-replay-rerun | 正規表現 | プール名 |
| `spill-over-layer0(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]spill-over-layer0)` | spill-over-crash-point-replay | 正規表現 | プール名 |
| `layer0-b-controls(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-b-controls)` | crash-point-replay-b-controls | 正規表現 | プール名 |
| `layer0-c-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-c-registered)` | crash-point-replay-c-registered | 正規表現 | プール名 |
| `layer0-d-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-d-registered)` | crash-point-replay-d-registered | 正規表現 | プール名 |
| `layer0-e-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-e-fast)` | crash-point-replay-e-fast | 正規表現 | プール名 |
| `layer0-e-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-e-full)` | crash-point-replay-e-full | 正規表現 | プール名 |
| `layer0-reuse-that-never-landed(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-reuse-that-never-landed)` | crash-point-replay-reuse-that-never-landed | 正規表現 | プール名 |
| `layer0-residual-record(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-residual-record)` | crash-point-replay-residual-record | 正規表現 | プール名 |
| `layer0-stale-tail(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-stale-tail)` | crash-point-replay-stale-tail | 正規表現 | プール名 |
| `layer0-threaded-sampling(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-threaded-sampling)` | crash-point-replay-threaded-sampling | 正規表現 | プール名 |
| `layer0-partial(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-partial)` | crash-point-replay-partial | 正規表現 | プール名 |
| `layer0-control(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-control)` | crash-point-replay-control | 正規表現 | プール名 |
| `layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-full)` | crash-point-replay-full | 正規表現 | プール名；ハイフンも単語文字とみなし、単独でマッチ |
| `caught_with_the_layer0_threads_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]caught_with_the_layer0_threads_value)` | caught_with_the_crash_point_replay_threads_value | 正規表現 | ローカル変数 |
| `layer0_threads_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_threads_value)` | crash_point_replay_threads_value | 正規表現 | ローカル変数 |
| `layer0_quick_tier_matches(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_quick_tier_matches)` | crash_point_replay_quick_tier_matches | 正規表現 | ローカル変数 |
| `sample-layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample-layer0)` | sample-crash-point-replay | 正規表現 | ゲートフィクスチャ名 |
| `sample_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0)` | sample_crash_point_replay | 正規表現 | 重型テストガードセルフテスト内のフィクスチャ名 |
| `layer0-by-name(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-by-name)` | crash-point-replay-by-name | 正規表現 | `prove-red.sh` の破損スイッチ |
| `named_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]named_layer0)` | named_crash_point_replay | 正規表現 | `prove-red.sh` 内の名前 |
| `demo_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]demo_layer0)` | demo_crash_point_replay | 正規表現 | `prove-red.sh` のフィクスチャターゲット |
| `some_layer0_stream(?![A-Za-z0-9_])(?<![A-Za-z0-9_]some_layer0_stream)` | some_crash_point_replay_stream | 正規表現 | 重型テストガードセルフテストのフィクスチャバイナリ名 |
| `layer0-sample(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0-sample)` | crash-point-replay-sample | 正規表現 | フィクスチャダミーツールのバージョン文字列 (`0.0.0-layer0-sample`) |
| `layer0-demo(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0-demo)` | crash-point-replay-demo | 正規表現 | 2台ラッパーセルフテストのタグ接頭辞 |
| `layer0-shaped-case-records-and-imports(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shaped-case-records-and-imports)` | crash-case-shaped-records-and-imports | 正規表現 | 2つのクラッシュ増幅スクリプトのセルフテストセル名 |
| `library-layer0-shape(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]library-layer0-shape)` | library-crash-case-shape | 正規表現 | 2つのクラッシュ増幅スクリプトセルフテスト内のライブラリディレクトリ名 |
| `layer0-shape(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shape)` | crash-case-shape | 正規表現 | 2つのクラッシュ増幅スクリプトセルフテスト内のフィクスチャファイル名 |
| `layer0_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shape)` | crash_case_shape | 正規表現 | 2つのクラッシュ増幅スクリプトセルフテスト内の変数名 |
| `STUB_CASE_SHAPE=layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_CASE_SHAPE=layer0)` | STUB_CASE_SHAPE=crash-case | 正規表現 | 2つのクラッシュ増幅スクリプトセルフテスト内のダミーケース形態 |

**付随して変更されたファイル名**：`crates/singlefs-core/src/superblock.rs` → `system_configuration.rs`；5つの実験装置、5つの変異表、5つの保持成果物（E100（システム構成の3段ジオメトリ）、E115（システム構成フィールド表の完全性とバイト予算）、E124（D18 確定項 14 に基づくシステム構成フィールド表の再計算）、E126（システム構成スロット幅4候補のコスト）、E147（新プール新規ファイル作成バイトテーブルに基づくシステム構成フィールド表の再計算））。移行手順は [.claude/rules/path-moves.md](../rules/path-moves.md) の「手順」節を参照。

**変更されていないもの**：`SUPERBLOCK_MAGIC` の**値** `*b"SFSB"` は変更しない——これはディスク上に書き込まれるマジックナンバーであり、変更することはディスクフォーマットを変更することと同義である。定数名自体は付随して変更された。

**改名前の原本**：git 内に存在する。中国語バッチの前の状態はコミット `495bede` 以前を参照；英語識別子および凍結ディレクトリバッチは `4105e70` 以降のワークスペース変更を参照。保持成果物とソースコードは同一回で一括変更されたため、`replay.sh` のバイト完全一致比較は改名前後でともに成立する——これは変更後の全量再実行によって証明されており、単なる主張ではない。
