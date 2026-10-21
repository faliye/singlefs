## E71 記帳キーのエントリ数と幅 —— 実行済み (2026-09-01、純算術、単体テスト 12 件 / ミューテーション 8 件全捕捉)：**先にパンクするのは `ツリー数 × デバイス数` の積であり、かつ「次元をフルに使い切る」解釈でのみパンクする**

[中文](71-记账key的条目数与宽度.md) · [English](71-记账key的条目数与宽度.en.md) · **日本語**

<!-- doc-lint:not-numbers Q3 FC1 FC2 FC3 FC4 FC5 V1 V2 V3 V4 S1 S2 S3 EV1 EV2 EV3 EV4 EV5 EV6 EV7 EV8 EV9 EV10 -->

⚠️ **判定基準 2 は「各ルートは1世代のみ保持、書き込みのみで削除なし」に基づいて再実行済み (2026-10-20)**。結果は「第 1 回再実行 (2026-10-20)：判定基準 2 を各ルートが1世代のみ保持・書き込みのみで削除なしとして再実行」の節に記載；本ページの「判定基準 2：1回のチェックポイントでの記帳書き込みバイト数 vs `T_dirty` の 1%」小節は旧計測条件 (1回書き込み + 1回削除 × 2) の数値である。

D5 (スナップショット / 領域会計メカニズム) 確定項目 1 はキーを `(統計量, ツリー ID, デバイス, 世代)` と決定した。
E54 (会計の世代を何世代保持すべきか) が当時提示した算式 `キー数 = 統計量数 × K` には**次元の組み合わせに関する2つの要素が欠落していた**。
統計量の個数が未確定であったため以前は絶対値を算出できなかったが、D5 (スナップショット / 領域会計メカニズム) 確定項目 4 (ユーザー決定)
が**9つの統計量**を確定したことで算出可能となった。

⚠️ **射程注 (2026-09-03)：E71 (記帳キーのエントリ数と幅) は s = 9 で実行されたが、D5 (スナップショット / 領域会計メカニズム) 確定項目 4
が同月3日に初回再開され、目録は11項目に拡張された。** バイナリと成果物は1文字も変更せず (`replay.sh` はバイト単位の照合を要求)、
s = 11 に基づく再計算は算術であり再実行ではない：上限腕 `s × t × d × K` は t=64 / d=8 / K=25 のセルで **115 200 から 140 800 に変化**；
狭い解釈腕は **4075 から 4300 に変化** (新規追加の2項目はいずれもツリー別分割なし——断片化 runs は次元なし ⇒ +K=25、
完全空クラスタセグメント数はデバイス次元を保持 ⇒ +d×K=200)。**2つの結論はいずれも動かない**：先にパンクするのは依然として `ツリー数 × デバイス数` の積であり、
上限の狭い解釈に対する過大評価倍率は 28.3 倍から 32.7 倍に変化する。

### 判定基準 (実行前ハードコード、実行後未改変)

1. **絶対値アサーション**：エントリ数は正確に `s × t × d × K` に等しくなければならず、腕間の相互比較のみで済ませてはならない。
2. 1回のチェックポイントでの記帳書き込みバイト数 ≤ D16 (パブリッシュセマンティクス) 確定項目 5 の `T_dirty` の 1%、
   さもなければ「記帳自体がダーティデータ予算を消費した」と判定。
3. 4つの独立変数はそれぞれ単調：いずれか1つを倍増させた場合、エントリ数は正確に倍増しなければならない。

### 失敗条項 (実行前ハードコード)

| 条項 | 判定 |
|---|---|
| **陽性対照**：デバイス次元を除去した場合、エントリ数は**d 分の 1 に低下**しなければならない；低下しない場合は次元がキーに反映されていないことを意味し、ラウンド全体を無効化 | **合格** (上限腕 115200 → 14400、正確に 8 分の 1) |
| s の真値が得られない場合、結果に「s = N に基づく計算」の計測条件を付記し、絶対的結論として記述してはならない | **適用除外**：s = 9 確定済み |

### ⚠️ 判定基準 1 と 3 が示す算式は上限であり、エントリ数ではない

`s × t × d × K` は9つの統計量**すべて**にツリー次元とデバイス次元を乗じるが、
D5 (スナップショット / 領域会計メカニズム) 確定項目 4 は「**キーの次元タプルはすべての統計量でフルに使われるわけではない**。
ツリー別としない項目は予約ツリー ID (0 を採用) を『プール全体 / 帰属なし』行として用いる」と明記している。

判定基準は D5 (スナップショット / 領域会計メカニズム) 確定項目 4 の確定**前**に記述されたものであり
(E71 (記帳キーのエントリ数と幅) 本文は当時「絶対値は本日算出不能」と記述)
⇒ **上限腕は原型のまま実行し、判定基準 1 / 3 および陽性対照はすべてそれに適用され、1文字も改変しない**；
項目別に次元を計上する腕を2本追加し、差額を計測した。

### ⚠️ D5 (スナップショット / 領域会計メカニズム) 確定項目 4 が未定とした点について、E71 (記帳キーのエントリ数と幅) は代行決定しない

「どの統計量が**ツリー次元を持つか**」の表には**記載がない**——デバイス次元の有無のみが記載されている。
⇒ 2つの解釈それぞれで1腕ずつ実行し、**1つを選んで正解としない**：

| 解釈 | ツリー別とする項目 | デバイス次元を持つ項目 (D5 (スナップショット / 領域会計メカニズム) 確定項目 4 が明記) |
|---|---|---|
| **狭い解釈** | 第 8 (ツリー別排他)・9 (ツリー別共有) の2項目のみ | 第 1、2、3、5 項目 (計 4 項目) |
| **広い解釈** | 9項目すべて | 第 1、2、3、5 項目 (計 4 項目) |

### 実測

#### 3腕のエントリ数 (t=64 ツリー、d=8 ディスク、K=25)

| 腕 | 1世代のエントリ数 | 総エントリ数 (× K) | 上限比 |
|---|---|---|---|
| **上限 (次元フル利用)** | 4608 | **115 200** | 1.00× |
| 広い解釈 (9項目ツリー別) | 2368 | 59 200 | 0.51× |
| **狭い解釈 (2項目ツリー別)** | **163** | **4 075** | **0.035×** |

⇒ **上限は狭い解釈より 28 倍大きい。** 差額はすべて「次元をフルに使い切る」解釈に起因するが、
D5 (スナップショット / 領域会計メカニズム) 確定項目 4 はその解釈を一字一句否定している
⇒ **`s × t × d × K` を用いて記帳ツリーサイズを見積もると、1〜2桁過大評価となる。**

#### 判定基準 3：倍増試験

| 腕 | 変数 | 基準値 | 倍増後 | 正確に 2 倍 |
|---|---|---|---|---|
| 上限 | t / d / K | 115 200 | 230 400 | **はい (3変数すべて)** |
| **狭い解釈** | t | 4075 | 7275 | **いいえ、1.785 倍** |

⇒ **判定基準 3 は上限腕では成立するが、実際のキーでは不成立**であり、後者こそが本番実装される対象である。
**これは結果であってバグではない**：ツリー別に分割されるのは 2/9 項目のみであり、ツリー数倍増はその2項目のみを拡大するためである。

#### 陽性対照：デバイス次元の除去

| 腕 | デバイス次元あり | 除去後 | 倍率 |
|---|---|---|---|
| **上限** | 115 200 | 14 400 | **正確に 8 倍 (= d) ✅** |
| 広い解釈 | 59 200 | 14 400 | 4.11 倍 |
| 狭い解釈 | 4075 | 3375 | 1.21 倍 |

⇒ 陽性対照は検証対象として設計された腕において**正確に成立**し、デバイス次元が確実にキーに含まれることを証明；
他の2腕で低下幅が小さいのは、デバイス次元を持つのが 4/9 項目にとどまるためである。

#### 判定基準 2：1回のチェックポイントでの記帳書き込みバイト数 vs `T_dirty` の 1%

⚠️ 本小節は 2026-09-01 の旧計測条件 (1回書き込み + 1回削除、書き込み回数係数 2) である。D5 (スナップショット / 領域会計メカニズム) 確定項目 2 が各ルートは1世代のみ保持に改定された後の新計測条件 (係数 1) は「第 1 回再実行 (2026-10-20)：判定基準 2 を各ルートが1世代のみ保持・書き込みのみで削除なしとして再実行」の節に記載し、双方の計測条件はその節の成果物内でセル単位で並記されている。

`T_dirty` = 2 GiB (D16 (パブリッシュセマンティクス) 確定項目 5) ⇒ 予算ライン **21 474 836 バイト**。
1回のチェックポイントは**1世代**の全エントリにアクセスし、それぞれ1回書き込み1回削除する
(D5 (スナップショット / 領域会計メカニズム) 確定項目 2 明記：書き込み回数は `統計量数` から `統計量数 × 2` に変更) ⇒ K はこの式に関与しない。

**90 セル中 9 セルが超過、すべて上限腕において発生；狭い解釈腕では 1 セルも超過せず。**

| セル | 上限腕 | 狭い解釈腕 | 予算 |
|---|---|---|---|
| t=1024, d=64, key+val=30 B | **35 389 440 (1.65 倍超過)** | 138 420 | 21 474 836 |
| t=8192, d=8, key+val=30 B | **35 389 440 (1.65 倍超過)** | — | 21 474 836 |
| t=8192, d=64, key+val=50 B | **471 859 200 (22 倍超過)** | — | 21 474 836 |
| t=1024, d=64, key+val=22 B | **25 952 256 (1.21 倍超過)** | — | 21 474 836 |

⇒ **先にパンクするのは4つの独立変数のいずれでもなく、積 `t × d` である**：
超過した2つのセル (1024×64 および 8192×8) の**積はいずれも 65 536** であり、バイト数は一字一句同一である。
⇒ パンク条件は `t × d ≳ 65536` (30 バイト幅、上限解釈下) である。

⚠️ **K はこの条件に含まれない**——ツリーの全体サイズを拡大するのみで、単一チェックポイントの書き込み量を拡大しないためである。
⇒ **E54 (会計の世代を何世代保持すべきか) が注目する K と判定基準 2 が注目する計測量は別物であり、同一視してはならない。**

#### K の値域 (D5 (スナップショット / 領域会計メカニズム) 確定項目 2 + D22 (ユニット原子性をどう合成するか) 確定項目 2)

`K = ルートリングスロット総数 + 1 = R × S + 1`、R = 3、S は 1..16 の値を取る：

| 領域あたりスロット数 S | スロット総数 | 保守的 K |
|---|---|---|
| 1 | 3 | 4 |
| 8 | 24 | 25 |
| 16 | 48 | 49 |

E53 (ディスク1台まるごと喪失後にルートリングがマウントできるか) の実機計測による最悪ロールバックは 1 ⇒ もう1つの計測条件は **K = 2**。
2つの計測条件には 24.5 倍の開きがあるが、**K は総エントリ数を線形に拡大するのみ**であり、判定基準 2 には影響しない。

### 第 1 回再実行 (2026-10-20)：判定基準 2 を各ルートが1世代のみ保持・書き込みのみで削除なしとして再実行

**状態：実行済み (判定に十分)**、純粋な算術、5ラウンドでバイト単位一致。実行前登録 `research/prompts/e71-r1-prereg.md` (問題票 `research/prompts/c693-forks.md` の Q3、E71 (記帳キーのエントリ数と幅) の半面；未済の検査 C693 (K 世代撤回後に再導出すべき論証))。

**1箇所のみ計測条件を変更**：1回のパブリッシュでの記帳書き込みバイト数 = 書き込み回数係数 × 1世代エントリ数 × エントリ幅、係数は 2 (1回書き込み + 1回削除) から 1 に変更——D5 (スナップショット / 領域会計メカニズム) 確定項目 2 の「各ルートの記帳ツリーはそのルートのその世代の行のみを保持し、複数世代を残さない」、射程「各パブリッシュ時にその世代の行をすべて書き直す」という確定に基づく。元装置から変更なし：2026-09-01 の9統計量表 (デバイス次元あり 4 項目、狭い解釈は 2 項目のみツリー別、広い解釈は 9 項目すべてツリー別)、幅 3 段階 22 / 30 / 50 バイト、t ∈ {1, 8, 64, 1024, 8192}、d ∈ {1, 8, 64}、閾値 21 474 836 バイト (D16 (パブリッシュセマンティクス) 確定項目 5 の `T_dirty` = 2 GiB の 1%、厳格に大で超過判定)。**本節の数値は一律に「2026-09-01 の9項目表、新行数計測条件」に基づく数値である**：現在 D5 (スナップショット / 領域会計メカニズム) 確定項目 4 は12項目を使用中であるが、今回は表を変更していない。

成果物 `research/results/e71-accounting-keys-single-generation-2026-10-20.out` (302 行、末尾行 `E7RESULT name=done emitted=302`)。各セルに1行 `name=publish_budget`、幅4段階 (元3段階 + 34 バイト) × 15 組の (t, d) × 3 腕 = 180 行、旧計測条件バイト数、新計測条件バイト数、2種類の判定、および新計測条件の閾値に対する千分比を並記。

**判定セル (上限腕、30 バイト、t=1024、d=64) は新計測条件で超過せず** (成果物第 288 行)：

```
E7RESULT name=judgement_cell arm=upper_bound width=middle trees=1024 devices=64 old_bytes=35389440 old_over=1 new_bytes=17694720 new_over=0 budget=21474836
```

新計測条件は 17 694 720 バイト、閾値の 823‰ を占有。登録された失敗条項 FC1 (D5 (スナップショット / 領域会計メカニズム) 確定項目 4 根拠内の「`1024 × 64` 時に……`T_dirty` の 1% を超過」が E71 (記帳キーのエントリ数と幅) 側で不成立) が**トリガー**。

**腕ごとの超過セル数** (成果物第 289–294 行；`registered_widths` は元3段階 45 セル、`current_implementation_width` は 34 バイトの 15 セル)：

```
E7RESULT name=over_budget_count arm=upper_bound widths=registered_widths cells=45 old_over=9 new_over=5 new_over_cells=narrow:8192:64,middle:8192:64,wide:1024:64,wide:8192:8,wide:8192:64
E7RESULT name=over_budget_count arm=upper_bound widths=current_implementation_width cells=15 old_over=3 new_over=1 new_over_cells=current_implementation:8192:64
E7RESULT name=over_budget_count arm=per_statistic_narrow widths=registered_widths cells=45 old_over=0 new_over=0 new_over_cells=none
E7RESULT name=over_budget_count arm=per_statistic_narrow widths=current_implementation_width cells=15 old_over=0 new_over=0 new_over_cells=none
E7RESULT name=over_budget_count arm=per_statistic_wide widths=registered_widths cells=45 old_over=5 new_over=3 new_over_cells=narrow:8192:64,middle:8192:64,wide:8192:64
E7RESULT name=over_budget_count arm=per_statistic_wide widths=current_implementation_width cells=15 old_over=1 new_over=1 new_over_cells=current_implementation:8192:64
```

| 腕 | 元3段階 45 セル中旧計測条件で超過 | 新計測条件で超過 | 解釈 |
|---|---|---|---|
| 上限 (次元フル利用) | 9 | 5 | 新計測条件で超過したセルは t × d が大きい側に集中：30 バイト段階では 8192 × 64 のみ残り、1024 × 64 と 8192 × 8 の2セルはいずれも 17 694 720 で超過せず；50 バイト段階では 1024 × 64、8192 × 8、8192 × 64 の3セルが超過。FC2 (超過セル数 0) は不発生。元ページの「パンク条件 `t × d ≳ 65536` (30 バイト)」は新計測条件で不成立：30 バイト段階の標本点で超過したのは t × d = 524 288 のセルのみ (65 536 と 524 288 の間に標本点が存在せず、反転点は未計測) |
| 狭い解釈 | 0 | 0 | 双方の計測条件で1セルも超過せず。新計測条件で最も逼迫したのは 50 バイト、8192 × 64：`new_bytes=832150`、`new_permille_of_budget=38` (成果物第 144 行)。FC3 不発生 |
| 広い解釈 (付帯、原判定基準 2 に本腕なし) | 5 | 3 | 幅3段階それぞれで 8192 × 64 のセルが超過；30 バイト 1024 × 64 セルは新計測条件で 8 017 920 (373‰) となり超過せず。D5 (スナップショット / 領域会計メカニズム) 確定項目 4 根拠文面の「広い解釈の腕」は 1024 × 64 セルで支持を失う；FC5 (広い解釈腕の超過セル数 0) は不発生 |

**幾何学的感度** (成果物第 295–300 行、上限腕判定セルのエントリ幅と t × d 隣接セルの変更)：

```
E7RESULT name=geometry_sensitivity point=narrower_entry_width registered_label=S1 arm=upper_bound width=narrow entry_bytes=22 trees=1024 devices=64 new_bytes=12976128 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=main_entry_width registered_label=main arm=upper_bound width=middle entry_bytes=30 trees=1024 devices=64 new_bytes=17694720 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=current_implementation_entry_width registered_label=S2 arm=upper_bound width=current_implementation entry_bytes=34 trees=1024 devices=64 new_bytes=20054016 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=wider_entry_width registered_label=S3 arm=upper_bound width=wide entry_bytes=50 trees=1024 devices=64 new_bytes=29491200 new_over=1 same_new_verdict_as_main_point=0
E7RESULT name=geometry_sensitivity point=neighbor_fewer_devices registered_label=trees_times_devices_neighbor arm=upper_bound width=middle entry_bytes=30 trees=1024 devices=8 new_bytes=2211840 new_over=0 same_new_verdict_as_main_point=1
E7RESULT name=geometry_sensitivity point=neighbor_more_trees registered_label=trees_times_devices_neighbor arm=upper_bound width=middle entry_bytes=30 trees=8192 devices=64 new_bytes=141557760 new_over=1 same_new_verdict_as_main_point=0
```

判定セルの判定は**エントリ幅に依存する** (FC4 トリガー)：22、30、34 バイトでは超過せず (34 バイトは 20 054 016、933‰ 占有)、50 バイトで超過、反転点は 34 と 50 バイトの間に位置する。判別力自己証明は単体テスト `moving_the_budget_across_the_judgement_cell_flips_its_verdict` に存在：閾値 17 694 720 では判定セルが超過せず、閾値 17 694 719 では超過となる。

**陽性対照と無効化条項** (成果物第 281–287、301 行)：

```
E7RESULT name=positive_control_write_factor arm=upper_bound cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_write_factor arm=per_statistic_narrow cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_write_factor arm=per_statistic_wide cells=60 old_equals_twice_new=60
E7RESULT name=positive_control_drop_device arm=upper_bound trees=64 devices=8 width=middle with_device_new_bytes=138240 without_device_new_bytes=17280 ratio_x1000=8000 exactly_by_device_count=1 holds=1
E7RESULT name=positive_control_drop_device arm=per_statistic_narrow trees=64 devices=8 width=middle with_device_new_bytes=4890 without_device_new_bytes=4050 ratio_x1000=1207 exactly_by_device_count=0 holds=1
E7RESULT name=positive_control_drop_device arm=per_statistic_wide trees=64 devices=8 width=middle with_device_new_bytes=71040 without_device_new_bytes=17280 ratio_x1000=4111 exactly_by_device_count=0 holds=1
E7RESULT name=arm_ordering groups=60 upper_at_least_wide_at_least_narrow=60
E7RESULT name=verdict cells=180 positive_control_write_factor_holds=true positive_control_drop_device_holds=true arm_ordering_holds=true
```

| 無効化条項 | 検証方法 | 結果 |
|---|---|---|
| V1 旧計測条件の元成果物再現 | 新装置が元フォーマットで再出力した 90 行の `name=ckpt_budget` と元成果物 `e71-accounting-keys-2026-09-01.out` (退避済み、`git show 3cff909b0^:research/results/e71-accounting-keys-2026-09-01.out` より抽出、sha256 `9826e85e…`) の同名行を `diff` | 双方 90 行、`diff` は 0 行相違 |
| V2 独立アンカー | 実行前登録著者の `anchors.py` 出力 (sha256 `a3d0bd98…`) 内の 48 行の `e71` アンカーを行ごとに `name=publish_budget` と突合 | `e71_anchor_rows matched=48 differing=0 missing=0` |
| V3 陽性対照 | 成果物第 281–286 行 | 各腕 60 セルで旧 = 2 × 新；上限腕でデバイス次元除去により正確に ÷ 8、狭い解釈・広い解釈は厳格に低下かつ 8 分の 1 未満 |
| V4 決定論 | 同一バイナリの 5 ラウンド実行を `cmp` | 5 ラウンドでバイト単位一致 (sha256 `885c238a…`)。決定論的モデルであり、ラウンド数は隠蔽された状態が存在しないことを示すのみ |

**単体テストとミューテーション**：単体テスト 15 件すべて緑；ミューテーション表 `scripts/mutation/tables/e71_accounting_keys_single_generation.tsv` 10 件 (登録 EV1–EV9 に登録改定補足の EV10「広い解釈腕でデバイス次元喪失」を追加)、捕捉 10、無効 0、未赤 0；`mutate.sh` 末尾行原文 (再利用 3 件) (出力全体は `research/results/e71-accounting-keys-single-generation-mutations-2026-10-20.log`)：

```
计数：内存撞顶 0 条（上限 2G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条、锚点失效 0 条、现判 7 条、复用 3 条、拒算 0 条
已还原，基线仍全绿
```

**登録改定** (成果物生成前に実行前登録「十二、改定」に記録)：広い解釈腕を「判定充足後未実行」から成果物に含めるよう変更；ミューテーション EV10 を追加；陽性対照甲を4段階幅に拡張。**判定充足後未実行**：登録された付帯量「パンク点」 (各幅段階での上限腕新計測条件超過の最小 t × d、D5 (スナップショット / 領域会計メカニズム) 確定項目 1 に対応、Q3 外) および「現在実装の行セット 3 + 6d 行 × 34 バイト」 (オーダー参照のみ提示)。

**本実験で答えられない点**：純算術であり、記帳ツリー、btree、write buffer を持たない；統計量は 2026-09-01 の9項目表であり、現在の12項目でも現在実装が各パブリッシュで書き出す 3 + 6d 行でもない；inode 番号水位の段差行をモデル化していない (ルートと共に残り、1回のパブリッシュで最大1行追加書き込み)；34 バイトは幾何学的感度点としてのみ扱う；エントリバイト数をカウントし、ダーティノード数をカウントしない (それは E97 (会計および割り当てレコードのエントリ符号化) 判定基準 10 に属する)。

### パスと結論登録

| # | パス | 計算方法 | ソースコード配置先 | 読み出した共用項目 |
|---|---|---|---|---|
| 1 | 新装置の2計測条件算術 | 各セルで腕ごとに1世代エントリ数を計上し、書き込み回数係数 (旧 2、新 1) とエントリ幅を乗じ、閾値と比較；登録された 180 セル (幅4段階 × 15 組 (t, d) × 3 腕) を全走査、サンプリングなし | `scripts/experiments/e71_accounting_keys_single_generation.rs:219 publish_write_bytes` | 2026-09-01 の9項目表の次元フラグ、幅3段階 + 34 バイト、閾値 21 474 836、書き込み回数係数2種 (いずれも実行前登録読法第 1–3 項から引用) |
| 2 | 独立アンカースクリプト | 実行前登録読法第 2、3 項の式のみから 48 セル (幅4段階 × 4 組 (t, d) × 3 腕) を都度計算、装置や `crates/` のコードを読まない；48 セルは判定セルと隣接セルを抽出、全走査ではない | 実装なし：リポジトリ内に存在せず、実行前登録著者の下書きディレクトリ `~/.local/share/singlefs/work/m3-close-w8-c693-reruns/anchors.py` (sha256 `1cef0e2b…`) に存在、出力は新装置の単体テスト `independent_anchor_rows_match_on_every_width_arm_and_cell` で 48 行の定数として固定 | 第 1 行の9項目表、幅、閾値および係数と同一 (同一の登録読法) |
| 3 | 原装置旧計測条件 | 原装置 `checkpoint_bytes` (2 × 1世代エントリ数 × エントリ幅)、2026-09-01 に出力された 90 セルの `name=ckpt_budget` | `scripts/experiments/e71_accounting_keys.rs:128 checkpoint_bytes` | 9項目表、幅3段階、閾値；旧係数 2 |

3本のパスは同一の9項目表と同一の閾値を共用している：表自体が判定対象のものでない場合 (現在は12項目)、3本すべてが同時に誤るため、本節の数値は「2026-09-01 の9項目表に基づく」と限定して記述される。パス 1 とパス 2 の比較は単体テストで固定 (48 行のセル単位 `assert_eq!`)；パス 1 とパス 3 の比較は単体テスト `original_format_lines_match_the_original_product_text` で 3 行のみ固定され、90 行フル実行突合は `diff` コマンドであり、ソースコード内でアサーションにより結合されておらず、未済の検査として残る。

### 本実験で答えられない半面

1. 純算術モデル：記帳ツリー実装なし、btree なし、write buffer なし、ファイル操作は 0 箇所。
2. **キー各セグメントのバイト幅はリポジトリ内で一切未定** (D5 (スナップショット / 領域会計メカニズム) 確定項目 3 は形態のみ決定、幅は未定)
   ⇒ 幅3段階 (value 込みで 22 / 30 / 50 バイト) は**仮定**であり、特定の段階のバイト数を結論としてはならない。
   ⚠️ **世代セグメント幅には既に厳格な下限が存在 (2026-09-02)**：D16 (パブリッシュセマンティクス) 確定項目 6 が世代はパブリッシュに伴い進むと決定
   (実機 2785 パブリッシュ/秒下で 32 ビットは 17.8 日でラップアラウンド) ⇒ ライダー条項「世代セグメント ≥ 48 ビット」、
   **E71 (記帳キーのエントリ数と幅) のバイト算術はこれに基づき再実行を要する**——再実行前はこれら3段階の数値は形状の参考にすぎない。
   パンク条件 `t × d ≳ 65536` は 30 バイト段階に基づいて提示されたものであり、幅が変われば再計算を要する。
3. **ツリー数 t の現実的値域に根拠がない**：D6 (スナップショット実装モデル) は多重書き込み可能ヘッドを決定したが、プール内にツリーが何本存在するかは
   一切記載がない。t=8192 等の段階は走査により導出したものであり、実負荷ではない。
4. **「どの統計量がツリー次元を持つか」が未定** (D5 (スナップショット / 領域会計メカニズム) 確定項目 4 の表はデバイス次元の有無のみ記載)、
   そのため狭い解釈と広い解釈の間に 14.5 倍の開きがあるという事実は
   E71 (記帳キーのエントリ数と幅) が計測した**乖離の大きさ**であって、答えではない。

### 前提条件

なし。純粋な算術。**元の登録前提条件「D5 (スナップショット / 領域会計メカニズム) 確定項目 4 (統計量に何があるか)」は 2026-09-01 に確定済み。**

### 計測条件と再実行

- コード `scripts/experiments/e71_accounting_keys.rs`
  (`cargo run --release --bin e71-accounting-keys`)、
  生出力 `e71-accounting-keys-2026-09-01.out` (209 行、末尾行 `emitted=209`)。
- **N=5 ラウンドでバイト単位一致** (同一 md5)。**単体テスト 12 件、ミューテーション 8 件が1件ずつ捕捉、死角 0 件**、
  ミューテーション表 `scripts/mutation/tables/e71_accounting_keys.tsv`
  (`bash scripts/mutation/mutate.sh e71-accounting-keys scripts/experiments/e71_accounting_keys.rs scripts/mutation/tables/e71_accounting_keys.tsv`)。
- 第 1 回再実行 (2026-10-20)：コード `scripts/experiments/e71_accounting_keys_single_generation.rs` (bin `e71-accounting-keys-single-generation`、元装置や `crates/` を `use` せず)、
  生出力 `research/results/e71-accounting-keys-single-generation-2026-10-20.out` (302 行、末尾行 `emitted=302`)、
  再実行登録は `scripts/experiments/replay.sh` の `E71|e71-accounting-keys-single-generation||e71-accounting-keys-single-generation-2026-10-20.out|exact`。
  ミューテーション：`cd scripts/experiments && bash ../mutation/mutate.sh e71-accounting-keys-single-generation e71_accounting_keys_single_generation.rs ../mutation/tables/e71_accounting_keys_single_generation.tsv` (上限 `MUTATE_MEMORY_MAX=2G`)。

### 影響を受ける決定

| 決定項目 | 関係 | 見直し |
|---|---|---|
| D5 (スナップショット / 領域会計メカニズム) 確定項目 4 | 支持 | 2026-10-20 変更：根拠文を第 1 回再実行の数値で書き直した——エントリ数の比は `ツリー数 × デバイス数` に応じて増え、`1024 × 64` では広い解釈の腕は 3 種の幅いずれも新計測条件で超過せず、上限腕は 50 バイトの段だけ超過、狭い解釈の腕は 45 セル中 0 セル超過（`research/prompts/m3-close-c693-r3-main-verification.md` 第 2 節 T3 のセル） |
| D5 (スナップショット / 領域会計メカニズム) 確定項目 1 | 支持 | 2026-10-20 影響なし：根拠内の「先に予算をパンクさせるのは `ツリー数 × デバイス数` の積である」は新計測条件でも従前通り成立——上限腕の超過 5 セルはすべて t × d が大きい側に集中、狭い解釈腕は 0 セル；変化したのは超過閾値点のみ (30 バイト段階 1024 × 64 が超過しなくなった) |
| D5 (スナップショット / 領域会計メカニズム) 確定項目 2 | 影響なし | 2026-10-20 影響なし：第 1 回再実行は本項目の現行確定 (各ルートは1世代のみ保持、各パブリッシュ時に全世代書き直し) を入力計測条件として使用し、計測も判定も行わない |
| D16 (パブリッシュセマンティクス) | 影響なし | 2026-10-20 影響なし：閾値は依然として確定項目 5 の `T_dirty` = 2 GiB の 1% を採用、第 1 回再実行で変更なし |
| D6 (スナップショット実装モデル) | 影響なし | 2026-10-20 影響なし：ツリー数 t は依然として走査により得られた値であり、第 1 回再実行で根拠は追加されていない |
| D22 (ユニット原子性をどう合成するか) | 影響なし | 2026-10-20 影響なし：K の値は総エントリ数にのみ関与し、第 1 回再実行の新計測条件では K ≡ 1 であり使用しない |

## 履歴バージョン

E71 (記帳キーのエントリ数と幅) の履歴エントリは [experiments-history.md](../experiments-history.md) に集約。
