## E143 1トランザクション1ユニット下でのジャーナル記帳 —— 実行完了 (2026-09-13、純粋算術、6 単体テスト / 10 ミューテーション全件捕捉)

[中文](143-一事务一单元下的journal账.md) · [English](143-一事务一单元下的journal账.en.md) · **日本語**

問うているのは、ユーザー確定 (分割規律は 1 トランザクション 1 ユニット、D16 (パブリッシュセマンティクス) 確定項目 5 末段は従前通り) の下で、C310 (トランザクション分割規律とレコード数計測条件の衝突) の記帳に対してどれだけ支払う必要があるかである：
D25 (目標負荷優先度) の 6 つの負荷点において、fsync ごとに何レコード発生し、ジャーナルがユーザーデータの何割を占め、T_dirty 満杯ウィンドウに対してどれほどの大きさのリングが必要か。
**6 つの基準すべてに合格、装置横断ゲート成立**：アーム丙は E91 (確定フォーマット下でのリング記帳と受入記帳) の計測条件に従い 1847 レコード / 7388 KiB を算出し、逐字一致。
**主要数値**：アーム甲 (条文) は seq バッチ 1 において fsync ごとに 8 レコード、ジャーナルがユーザーデータの 12.5% を占有；T_dirty 満杯ウィンドウは 65 536 レコード、256 MiB であり、想定された 64 MiB リングの 4 倍；
アーム乙 (1 レコードに複数トランザクションを格納、対照) は 1 レコード、1.56%、満杯ウィンドウ 4232 KiB。数値報告のみを行い、条文を改定すべきか否かは判定しない。

**再実行コマンド** (`exact` モード、コミット済み成果物とのバイト単位比較)：

```bash
bash scripts/experiments/replay.sh E143
# 装置を直接実行：
cd scripts/experiments && cargo run --release --bin e143-one-unit-per-txn-journal
```

コード `scripts/experiments/e143_one_unit_per_transaction_journal.rs`、生出力 `e143-one-unit-per-txn-journal-2026-09-13.out` (34 行、最終行 `emitted=34`)、
事前登録 `e143-preregistration.md`、ミューテーション表 `scripts/mutation/tables/e143_one_unit_per_transaction_journal.tsv`。

### これは純粋算術であり、測定ではない

I/O も乱数源もない ⇒ N 回実行すればバイト単位で一致する。根拠の強度は 6 件の単体テストで固定された絶対値 ＋ 10 件のミューテーション全件捕捉に由来する：
レコードヘッダを 78 に戻す、乙の項目幅を 56 に変更、甲が祖先を別レコードに分離、seq バッチ 1 を 12 葉に変更、ディスク数を 1 に変更、丙の満杯ウィンドウをユニット数で計算、F を 2 に変更、リングを 128 MiB に変更、丙で 1 ユニットあたり 1 トランザクション、T_dirty を 1 GiB に変更、の各項目においてテストが赤になる。

### テスト対象とアーム

| アーム | トランザクションをどう分割するか | 1 レコードに何を格納するか | 1 レコードに何項目格納できるか |
|---|---|---|---|
| 甲 今日の条文 | データユニットごとに 1 トランザクション (D16 (パブリッシュセマンティクス) 確定項目 5 末段) | 1 トランザクション (D23 (journal の役割とフォーマット) 確定項目 7：1 レコードに 1 つのトランザクション番号)；祖先はこの fsync の末尾レコードに統合 | (4096 − 95) ÷ 56 = 71 |
| 乙 対照 | データユニットごとに 1 トランザクション | 複数トランザクション：指名項目が各々トランザクション番号 8 を持ち、レコードヘッダからトランザクション番号とコミットマークを除去し、先頭・末尾トランザクション番号 16 を追加 | (4096 − 102) ÷ 64 = 62 |
| 丙 対照 (ユーザー却下済み) | 1 回の fsync で 1 トランザクション (E75 (レコード寸法とリング幾何) の旧解釈) | 1 トランザクション | 71 |

負荷点は D25 (目標負荷優先度) に準拠：rand バッチ 1 (1 葉 ＋ 4 祖先)、multistream バッチ 1 (1 ＋ 4)、seq バッチ 1 (8 ＋ 4)、metaheavy バッチ 1 (2 ＋ 5.937、祖先は項目数で整数化 6)、seq バッチ 10 (80 ＋ 4)、multistream バッチ 10 (10 ＋ 31)；
「葉」は E75 (レコード寸法とリング幾何) の解釈に従いデータユニットとして扱う。

### 判定 (実行前ハードコード基準、成果物からそのまま転記)

```text
E7RESULT name=capacity arm=jia items_per_record=71
E7RESULT name=capacity arm=yi items_per_record=62
E7RESULT name=fsync load=seq batch=1 leaves=8 ancestors=4 arm=jia transactions=8 records=8 journal_bytes_two_devices=65536 data_bytes_two_devices=524288 journal_over_data_bp=1250 records_per_second=22280 journal_kib_per_second=178240
E7RESULT name=fsync load=seq batch=1 leaves=8 ancestors=4 arm=yi transactions=8 records=1 journal_bytes_two_devices=8192 data_bytes_two_devices=524288 journal_over_data_bp=156 records_per_second=2785 journal_kib_per_second=22280
E7RESULT name=fsync load=seq batch=10 leaves=80 ancestors=4 arm=jia transactions=80 records=80 journal_bytes_two_devices=655360 data_bytes_two_devices=5242880 journal_over_data_bp=1250 records_per_second=222800 journal_kib_per_second=1782400
E7RESULT name=fsync load=multistream batch=10 leaves=10 ancestors=31 arm=jia transactions=10 records=10 journal_bytes_two_devices=81920 data_bytes_two_devices=655360 journal_over_data_bp=1250 records_per_second=27850 journal_kib_per_second=222800
E7RESULT name=window arm=jia records=65536 bytes_kib=262144 ring_kib=65536 window_over_ring_milli=4000
E7RESULT name=window arm=yi records=1058 bytes_kib=4232 ring_kib=65536 window_over_ring_milli=64
E7RESULT name=window arm=bing records=1847 bytes_kib=7388 ring_kib=65536 window_over_ring_milli=112
E7RESULT name=ring arm=jia worst_transaction_bytes=4096 lower_bound_any_transaction_kib=12 lower_bound_dirty_window_kib=786432
E7RESULT name=ring_cap arm=jia ring_kib=65536 safety_factor=3 dirty_bytes_allowed=178946048 dirty_kib_allowed=174752
E7RESULT name=keys load=seq batch=1 arm=jia data_units=8 distinct_write_orders=8
E7RESULT name=verdict jia_records_seq_batch1=8 jia_journal_over_data_bp_seq_batch1=1250 bing_window_records=1847 bing_window_kib=7388 e91_anchor_holds=true jia_window_over_ring_milli=4000
```

| # | 基準 | 結果 |
|---|---|---|
| 1 | fsync ごとのレコード数 | 甲 1 / 1 / 8 / 2 / 80 / 10、乙 1 / 1 / 1 / 1 / 2 / 1、丙 1 / 1 / 1 / 1 / 2 / 1、登録表とセルごとに一致 |
| 2 | ジャーナルのユーザーデータ占有率 | 甲 12.5% (6 つの負荷点ですべて同一、各データユニット恒常的に 1 レコード)、乙 seq バッチ 1 は 1.56% |
| 3 | 装置横断ゲート | 丙は E91 (確定フォーマット下でのリング記帳と受入記帳) 計測条件の 1847 レコード、7388 KiB と逐字一致 |
| 4 | T_dirty 満杯ウィンドウ | 甲 65 536 レコード、262 144 KiB、64 MiB リングの 4.0 倍；乙 1058 レコード、4232 KiB |
| 5 | I-8.1 (リング幾何が十分大きい) の 2 つの解釈 | 「任意のトランザクション」の解釈では、甲の最悪 1 トランザクションは 1 レコード、F = 3 下限 12 KiB；「満杯ウィンドウがリングに収まる」解釈では、甲は 768 MiB を要求、または T_dirty を 174 752 KiB (約 170.7 MiB) に圧縮 |
| 6 | マッピング key の一意性 | 甲、乙の次回 fsync の 8 データユニットは 8 つの異なる書き込み順序を持つ；丙は 1 つのみ (これこそ D19 (ブロックポインタの構造と幅の予算) 確定項目 6 が分割規律に賭けていたセルである) |

### これら数値が示すこと

1. **D23 (journal の役割とフォーマット) 確定項目 12 の「目標負荷 12 項目トランザクションはちょうど 1 レコードを占有 ⇒ 5.9 倍のマージン」という記述は丙の解釈に準拠している**。ユーザー確定の分割規律に従えば、seq バッチ 1 は 8 レコード、seq バッチ 10 は 80 レコードとなる；マージンのセルは消失し、
   各レコードはわずか 1 から 5 個の指名項目しか格納しない (4096 バイト中 151 から 375 バイトを使用)。
2. **リング幾何の制約が「任意のトランザクション」から「満杯ウィンドウ」へと移行する**：I-8.1 (リング幾何が十分大きい) の字面は任意のトランザクションの最悪占有を律しており、甲の下では 1 レコード、下限 12 KiB で制約は無いに等しい；
   真にボトルネックとなるのは T_dirty 満杯ウィンドウ 256 MiB 対 64 MiB リングである —— リング ≥ 768 MiB (F = 3) とするか、T_dirty ≤ 170.7 MiB とする、あるいはレコードに複数トランザクションを格納する (乙)。
3. **乙は候補条文ではない**：D23 (journal の役割とフォーマット) 確定項目 7 のレコードヘッダの変更 (トランザクション番号とコミットマークを除去し、指名項目にトランザクション番号を付与) を要し、それは D16 (パブリッシュセマンティクス) 確定項目 3 が定めた独立凍結コンポーネントである；「12.5% と 4 倍」の対照として設けられたに過ぎない。
4. 秒あたり 2785 回の fsync を行う seq バッチ 10 におけるジャーナル書き込み帯域は 1.7 GiB/s (2 ディスク合算) であり、データ書き込み帯域 5 GiB/s に対する比率は依然として 12.5% である。

### 何に答えられないか

1. 純粋算術：リングの周回をシミュレートせず、時間を測定しない；祖先ノードをどのレコードが指名するかは条文に記述がなく、甲は「末尾レコードに統合」を採用。
2. 乙のレコードヘッダ改定手法は装置が対照用に構築したものであり、候補ではない。
3. T_dirty の「ダーティバイト」がデータユニットを数えるかノードを数えるかは D16 (パブリッシュセマンティクス) 確定項目 5 に記述がない；甲、乙はデータユニット単位、丙は E91 (確定フォーマット下でのリング記帳と受入記帳) のノード計測条件に従い、3 者は同一計測条件上で比較されていない。
4. 決定論性：N 回実行しても同一である。

### 影響を受ける決定事項

| 決定項目 | 関係 | 振り返り |
|---|---|---|
| D23 (journal の役割とフォーマット) 確定項目 19 | 支持 | 2026-10-11 影響なし：表現のみ (時間表現、位置の指示、一人称)；結論、数値、成果物は不変 |
| D23 (journal の役割とフォーマット) 確定項目 12 | 準備 | 2026-10-11 影響なし：表現のみ (時間表現、位置の指示、一人称)；結論、数値、成果物は不変 |
| D23 (journal の役割とフォーマット) 確定項目 7 | 影響なし | 2026-10-11 影響なし：表現のみ (時間表現、位置の指示、一人称)；結論、数値、成果物は不変 |
| D16 (パブリッシュセマンティクス) 確定項目 5 | 準備 | 2026-10-11 影響なし：表現のみ (時間表現、位置の指示、一人称)；結論、数値、成果物は不変 |
| D19 (ブロックポインタの構造と幅の予算) | 準備 | 2026-10-11 影響なし：表現のみ (時間表現、位置の指示、一人称)；結論、数値、成果物は不変 |
| D25 (目標負荷優先度) | 影響なし | 2026-10-11 影響なし：表現のみ (時間表現、位置の指示、一人称)；結論、数値、成果物は不変 |

## 歴史的バージョン

### 2026-09-13
- 新規作成。事前登録は装置作成前に執筆；1 回実行、基準と期待値は未変更。
