**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:1-21`（整段抄，未转述）**

````markdown
## E171 3つのオラクルの共謀テスト — 第1回再実行の第1段階と第2段階の表全体（改訂6のツリー）を実行済み（Q1は甲、Q2は乙、Q3は甲で分母 D = 39、Q4はベースラインが甲で行242・269が乙。Q5の警報自己検証は合格し、判定は甲。最終本流の改訂8の実行はツリーの固定待ちで、ストリーム3の区間長は改訂12に従って再固定済み）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Y1 Y3 P1 P2 P3 P4 P5 P8 P9 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 F1 F2 F3 F4 F5 F6 F7 V1 V2 W1 K4 Q5 S15 S17 Z7 U1 U9 U13 U16 V17 V31 V33 V36 R14 R19 R20 R33 F8 RS1 RS2 RS3 RS4 RS5 RS6 RS7 RS8 RA1 RA2 RA3 RX1 S19 F10 Z10 -->

[中文](171-三oracle合谋测试.md) · [English](171-three-oracle-collusion-test.en.md) · **日本語**

```text
このページの局所的な試験群名と記号は小文字（a1、a2、a3、a4 など）で書く。事前登録、装置、成果物、コマンドでは対応する大文字を使い、全体登録の番号と同じ形になる。KB 本文では番号と区別するために小文字を使う。成果物を読む際は対応する大文字に読み替える。
```

`e171-preregistration.md` の事前登録を実行する（第 12 節の改訂 1–5 を含む；Q4 の別の読み方は、主担当エージェントが装置に書き込む前に削除し、全体テーブルが ≤60 秒となるもののみを残す）；問題票 `three-oracle-collusion-questions.md` の Q1（同一バッチの履歴とクラッシュ状態において、一つの不良実装表に対して、三つの oracle がすべて放通する行が存在するか）、Q2（手製の不良イメージを O2（独立解析器 + checker）および O3（独立仕様実行器）に与え、二つがすべて放通する表が存在するか）、Q3（既知で検出された不良実装について、各 oracle を逐次無効化し、残りの oracle が依然として赤判定となるか）、Q4（このテストを一度実行するのに要する時間、およびゲートへの進入方法）；被験条項 D13（検証経路）は項目 6 で確定済み：三つの oracle の独立性は共謀テストによって検証する必要があり、自己主張による免責は認められない。

**現状**: 修正された行ごとに駆動が、同一の凍結したツリー上で 117 行の不良実装表を完走させた（s4、2 つのライン、各ライン 2 スレッド）。これには Q3-コア 3 回とコンパレータが含まれる。s3 の際は、駆動が前行の変異を次行に持ち込み、表全体が無効化されアーカイブされた（「s3 表全体無効化」節）。O3（独立した仕様実行器）は、E170（O3 仕様実行器第 1 版）を用いて凍結した実行器を初めて再実行し、開始と終了の 2 回の凍結コア比較において差異は 0 箇所であった。s4 の分母 D = 19 < 30 により F4 トリガーが登録され、表は 164 行（番号 118–281）に拡張された。この拡張された表は同一の凍結したツリー上で完走し、コンパレータによって両半分を合わせて 281 行とした：分母 D = 37 ≥ 30。2 回の試行でコンパイルに失敗した 6 行（第 33、59、127、163、206、243 行）の当時の駆動は、登録 5.2 に従って代替実行を行っていなかった。駆動をコンパイルに失敗後に代替実行するように変更し、その代替実行試行が同一の凍結したツリー上で完走した（入れ替えられた 6 行はすべて未到達行であった）。コンパレータによって 117 + 164 + 代替実行を合わせて `…-s4-f4-substitutes-comparator.out` とした：分母は依然として D = 37、Q1 および Q3 のリストは代替実行前と完全に一致した（「コンパイルに失敗した 6 行の代替実行（実行済み）」節）。読み取り値が揃った後、3 回の結論判定が行われた（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`、第 3 回が最終回）。この一連の読み取り値が D13（検証ルート）に適合するかは既に決定事項 6：真のブラインドスポット候補は第 15、23、42、242、269、270、271 行で収穫され、残りは全通過および未到達行の中で値が異なる行を他のカテゴリに分類した（「3 回の判定後の三分類」節）。三分類と読み取り値はともに s4 の凍結したツリーのみに関連する。

| 行 | 判定 | 根拠（成果物） | 文言に加えて読むべき事項 |
|---|---|---|---|
| Q1 | はい：甲、乙ともに 12 行で3つのオラクルをすべて通過（s4 の 7 行に拡張表の第 242、269、270、271、277 行を加えたもの）、丙は 13 行（別に第 233 行を追加）；分母 D = 37（代替実行前と後で同一） | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` の `name=q1_alpha`、`name=q1_beta`、`name=q1_gamma`（`verdict=yes_a_row_is_quiet_on_every_oracle`）、`name=q1_denominator`（`counted_D=37 … enough_to_judge_none_side=true`）；代替実行前の `…-s4-f4-comparator.out` において Q1 の三つの読み方の名前行は文字通り同一 | 登録第 6 節の Q1-甲「≥ 1 判『有』」に従い、分母とは無関係；D ≥ 30 は「ない」という側の閾値が十分であることを述べるのみであり、このバッチでは「ある」は既に成立している。s4 の 7 行の読み込み値は F4 産物において変化していない（第 1–117 行の結果行はバイト単位で同一）；拡張表によって新たに生成された 5 行と第 233 行（丙）、バッチ 2 の第 265 行の読み込み値は「F4 で新たに全通過された行」の節に記載されている。三回の判決後の三分類は「三回の判決後の三分類」の節にある：真の盲点候補は 7 行（第 15、23、42、242、269、270、271 行）、その他のカテゴリは 5 行（第 27、112、277 行は記録再構築による回復、第 34 行は割り当てポリシー、第 92 行はバイト表とマイルストーン受入における条項）；一行も「このバッチに等価」と記されていない。第 233 行は丙において全通過され（甲乙において O1（メモリモデルの一致チェック（HashMap の理想的なファイルシステム））-クラッシュ判官が検出）、第 265 行バッチ 2 はサンプリングポイントに不感応である。O3（独立仕様実行器）は 37 行中一行も赤判定を行っていない（`name=q1_column_red_rows … O3（独立仕様実行器）甲=0 O3（独立仕様実行器）乙=0`）。代替実行後の分母は依然として 37 で、「ある」は反転していない |
| Q2 | はい（登録の文言どおり）：甲・乙ともにO2とO3の双方を通過する画像が4件あり、すべてP8 | 同じ成果物の `name=q2_all_quiet`（`table_rows=107 all_quiet_rows=4 verdict=yes_a_bad_image_passes_both`）、`name=f3_p_rows_judged_in_by_o3`、`name=q2_negative_bases`；O2（独立解析器 + checker）が通過し、O3（独立仕様実行器）が含まれないと判定した数が `name=q2_family` の P 族およびブラインドテスト B 族の 2 行にある（`o3_only_strict=12`、`o3_only_strict=17`、行全体は「Q2 において O2（独立解析器 + checker）が通過し、O3（独立仕様実行器）が含まれないと判定した不良イメージ」節に転記）。4 枚すべて E170（O3 仕様実行器第 1 版）の設計者が推論した「含まれない」でありながら O3（独立仕様実行器）が「含まれる」と判定した P8 行で F3 がトリガーし、主担当エージェントは F3 の判定に従って期待値を誤推し、この 4 枚は不良イメージではない（判決は D13（検証ルート）の既定項目 6 の射程に記録）。Q2の数と第2段階 `…-s2-comparator.out` は同一バッチの入力から出ている。s4 比較器で再計算；F4 産物において Q2、F3、および対照の結果行は s4 のその版とバイト単位で一致する。幾何学的な 2 点（`q2_geometry_unstable_kinds=2`）：裏返るのはストリーム1 P2、P5 の 2 つのパッチに対する O2（独立解析器 + checker）の一列であり、基底状態 F は赤判定、Mk は赤判定ではない。O3（独立仕様実行器）は 2 つの基底状態とも「含まれない」と判定するため、この 2 つのパッチの「2 条すべて通過」の 2 点は成立せず、裏返らない。すべて通過される P8 の 2 つの基底状態は `same_on_both_points` である。さらに O2（独立解析器 + checker）が通過し、O3（独立仕様実行器）が含まれないと判定した不良イメージが 29 枚ある（P 族 12 枚にブラインドテスト B 族 17 枚を加えた 12 + 17）。記述は「O2（独立解析器 + checker）が通過し、O3（独立仕様実行器）が含まれないと判定」であり、「O3（独立仕様実行器）固有」とは書かない。Q2 は O2（独立解析器 + checker）と O3（独立仕様実行器）の対比のみを行う。O1（メモリモデルチェック（HashMap 理想ファイルシステム））および RefFS は不良イメージ上で動作していない。この 29 枚は構造的自己整合性を持つ偽造ルート、偽続接イメージであり、パッチと期待値はともに E170（O3 仕様実行器第 1 版）の変更コードから出ている。詳細は「それが答えられない」節にある |
| Q3 | 1つのオラクルだけに依存する行がある：甲・乙ともに23行（O1（メモリモデルチェック（HashMap 理想ファイルシステム））の 32、45、108、115、172、233、237、249、250、281 にのみ依存；O2（独立解析器 + checker）の 8、65、86、87、90、99、102、135、234、238、245、258、265 にのみ依存） | 代替実行後の `…-s4-f4-substitutes-comparator.out` の `name=q3_alpha`、`name=q3_beta`（代替実行前の `…-s4-f4-comparator.out` の同名行は文字通り同一）；Q3-核の 9 行 `name=q3_core_row` はすべて `differences_outside_the_closed_column=0`（Q3-核を第一刀で切り取る行であり、拡張表には第一刀がない） | Q3-核の 9 回走査で 0 の差異があり、読み取り方の 9 が成立し、F5 はトリガーしない。102、233、265 の行においてバッチ 1 とバッチ 2 で赤判定される列が異なるのは、サンプリングポイントに不敏感であるため（バッチ 2 は赤となる区間や状態をサンプリングしなかった）。これを「不安定」と記録する。s4 の 11 行のうち残りの 2 条がなぜ「Q3 にのみ依存する 11 行」節に見えないのか、および拡張表で新たに出現した 12 行が「F4 で新たに出現したみに依存する 12 行」節にあるのかについて説明する |
| Q4 | 全量ゲートにのみ組み込める：117 行の表全体で 2 つの実行レーンを使用し、所要時間は 5,569.6 秒。表を拡張した 164 行では別のラウンドで 7,607.9 秒かかり、閾値は 60 秒です | `e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out` の 2 行 `name=table source=table-driver.out`（`rows=117 table_wall_ns=5569579167726`、`rows=164 table_wall_ns=7607936234181`） | 281 行を 2 ラウンドに分けて実行し、2 つのラウンドの掛時計時間を合計すると約 13,177.5 秒（2 つの数値を足し合わせたもので、単一ラウンドの量ではない）。各行の和である 11,114.4 + 15,191.9 秒は単一実行レーンの推定値であり（単一実行レーンを個別に実行していないため）、コンパイルに失敗した 6 行は別のラウンドで補完され、所要時間は 1,277.9 秒です（`…-s4-f4-substitutes-rows-timing.out` の `name=table source=substitution-driver.out … rows=6 table_wall_ns=1277937171946`）。3 ラウンドを合計すると約 14,455.5 秒となり、これも 3 つの数値の和です。Q3 では核ごとにラウンドあたり 3 行で約 114–130 秒 |

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:22-39`（整段抄，未转述）**

```markdown
### デバイス、研究側と変異表

研究側のバリエーション表を各改訂上で再実行する（`replay.sh` によるバイト単位の比較を行う 3 つの実行）：改訂 8 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-10-07.out`（`E7RESULT name=verdict mutations=26 caught=26 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`）、改訂 10 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-s2-revision-10-tools-research-mutations.out`（`mutations=29 caught=29`）、改訂 11 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-revision-11-tools-research-mutations.out`（`mutations=15 caught=15`）であり、3 つとも `invalid=0 survived=0 timed_out=0` である。

| 件 | 位置 | 説明 |
|---|---|---|
| 登録済み装置 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` | モード `baseline`、`row`、`known-bad-images`；スイッチ `--positive-control`、`--disable-oracle`；読み取り専用・駆動専用・観測専用、core、checker、harness、format、replay-prefix を変更しない；内联単测 31 条（`grep -c '#\[test\]'`） |
| 研究側 | `scripts/experiment-apparatus/e171_three_oracle_collusion/`（Python、ディレクトリに `main.rs` なし、cargo は bin と認識しない） | 選定スクリプト（登録 5.2、sha256 `4032a0e0…`、そのままコピー）、117 行の表 `bad-implementations-117.tsv`（登録 5.9 で名前を照らし出してコピー）、F4 拡張表 `bad-implementations-expansion-164.tsv` とそれを生成する `expansion_table.py`、`row_driver.py`、`comparator.py` とそのいくつかのモジュール、フリーズされた実行器の接続 `frozen_o3.py`、不良イメージを判定する `o3_bad_image_judge.py`、4 つの単一テストファイル、変異駆動 |
| 変異 | 研究側は `scripts/mutation/tables/e171_three_oracle_collusion.tsv` の 67 行；装置側は `crates/mutations.tsv` の末尾 15 行（名前が「E171（三 oracle 合謀テスト）装置」で始まり、`grep -c '^[E]171 装置' crates/mutations.tsv` で確認可能） | 研究側は本ディレクトリの `mutation_driver.py` を実行して登録改訂 1 の第 1 行を処理；装置側はチェッカーファイルであり、3 つの数値が提出時のゲートチェッカー tier-crates-mutation-replay に属する |
| 再実行 | `scripts/experiment-apparatus/replay.sh` 内の E171（三 oracle の共謀テスト）20 行登録（`grep -c '^[E]171.@driver' scripts/experiment-apparatus/replay.sh`）：このページは元々 11 行、第 1 回再実行で 9 行追加（第1段階・改訂 6 その回の 3 行、第2段階 2 行、改訂 8、10、11 の研究側変異 3 行、改訂 12 の 1 行） | 元々 11 行は「再実行」節参照；第1段階の 3 行は「比較器、再実行と登録の実行方法（改訂 6 その回）」節参照；第2段階の 2 行は「第 1 回再実行第2段階・改訂 6 その回」節参照；改訂 12 のその行は「第 1 回再実行・改訂 12：ストリーム3段長重釘（2026-10-13）」節参照；改訂 8、10、11 の 3 行はそのページに節なし |
| 成果物 | `research/results/` 下 `e171-three-oracle-collusion-2026-10-03-` で始まる 46 件（`ls research/results \| grep -c '^e171-three-oracle-collusion-2026-10-03-'`） | 各成果物は「再実行」節の一覧に記載 |

- ストリーム駆動：2 つのストリーム（新しいプールへの新規ファイル作成、固定されたスクリプトを E に書き込みその後正常にアンマウント）は O3 規約実行器第一版（E170）から装置の 2 つのストリーム駆動、サンプリング 3 バッチと中性形式での出力（`#[path]` で参照しないこと）をそのままコピーし、3 カ所を変更する：駆動内の API 呼び出しは `.expect` を使用せず、結果（成功、エラーメンバー、パニック）を `calls.tsv` の `ok` / `err` 列および `call-outcomes.tsv` に記録する。各呼び出しごとに理想モデルを追跡し、O1（メモリモデルのチェック：HashMap による理想ファイルシステム）- チェック（モデルの答えと実装の結果を比較、`judge_and_advance`）を記録する。各サンプリング状態について O1（メモリモデルのチェック：HashMap による理想ファイルシステム）- クラッシュ判官（`crash_recovery_disagreement`）および O2（独立したパーサー + チェッカー）の `check_pool_image` を計算し、`oracle-verdicts.tsv` に書き込む。`baseline` はローカル定数と crates の現在の計算値と比較する（s1、42 行の `name=s1` はすべて `holds=true`；`grep -c 'name=s1 ' …-baseline-run1.out` と `grep -c 'name=s1 .*holds=false'` で現在の数を取得）；`row` は比較せず、バッチ 1 とバッチ 2 のみを使用する。
- S8 の 3 つの呼び出しが装置内の行番号（`grep -n` で現在取得）：`newest_persisted_root` が第 1883 行、`observed_read_back_after_a_crash` が第 1885 行、`crash_recovery_disagreement` が第 1907 行であり、これらはすべて `crash_judge_of_the_state`（第 1875 行）およびその呼び出し先である `crash_judge_verdict` にある。O1（メモリモデルのチェック：HashMap による理想ファイルシステム）- チェックの `judge_and_advance` は第 544 行にある。
- 既知の不具合のあるイメージファミリー：`crates/singlefs-harness/tests/checker_known_bad_images.rs`（sha256 `e8543908…`）からのインポート、定数、ヘルパー関数および `known_bad_images` を装置内の 1 つのモジュールにそのままコピーする（第 7–1294 行に加え、`known_bad_images` に使用され散在している第 1295 行之後の 12 の定数とヘルパー関数を含む）；`known-bad-images` モードの核条数は 41 であり、項目名および元のファイルはそれぞれ一致し、クリーンなイメージのハッシュサマリーは `build_pool` と等しい（S4）。
- 行ごとに駆動：一度実行してまずリポジトリのスナップショットを撮影する（`Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`、237 ファイル、総ハッシュサマリーは駆動の標準出力第一行に記述）、各ラインはスナップショットから自身のツリーとコンパイルディレクトリを同期する。1 カ所を変更し、`cargo build --release` を実行し、装置 `row` を `run-with-memory-cap.sh`（時間制限 1800 秒）経由で実行して復元する。表中の行は名前に基づいて選択し、核アンカーポイントを使用する。失敗した場合は代替実行規則に従って同じグループの次の行に切り替える（最大 3 回）。s4 と F4 の 2 回の駆動では、コンパイル前の失敗に対してのみ代替実行を行う。8 セクション以降でコンパイルが失敗した場合も代替実行を行う（入れ替え後の行でもコンパイルが失敗すれば再度入れ替え、コンパイル前の失敗とコンパイル不能を合わせて最大 3 回）、グループには E171（3 つの oracle の共謀テスト）装置自身のバリアント行は含まれない。`row-result.tsv` は `substituted_from` および試したが使用されなかった各 `failed_attempt_<順序>` を記録する。スイッチ `--keep-compile-failures-as-they-are` は `replay.sh` が s4、F4 の 2 つの製品を再実行する場合のみ有効であり、代替実行された製品にはこのスイッチは含まれない。


```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:40-87`（整段抄，未转述）**

````markdown
### s4 整表の結果（行全体は産物から直接引用）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out`(2119 行，末行 `E7RESULT name=done emitted=2119`)。凍結したツリーペア、陽性対照の 4 つのオラクルそれぞれ 2 回（ベースラインと空変異）、Q1 の分母と 3 つの読み方、7 行すべてを許可（甲）、Q3 の 2 つの読み方、Q3-核 9 回および判定行：

```
E7RESULT name=config segment=s4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=empty_mutation red_units=1 red=true
E7RESULT name=q1_denominator rows_run=117 counted_D=19 untouched=93 only_in_the_ignored_pass=0 compile_failed=2 anchor_not_unique=0 not_found=0 self_crash=3 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=false
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=5 rows_run=117 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=19 O1=5 O1-对拍=5 O1-崩溃判官=1 O2=8 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=0 row_numbers=none
E7RESULT name=q1_alpha counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_quiet_row reading=q1_alpha row=15 row_name=实审_A2c_第_15_条：用户数据落点不从这张图的单元区起点起扫 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=22/22 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=23 row_name=并行线三_C116：分裂出来的右半容器号不是触发分裂那条新记录的_inode_号 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=26/26 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=27 row_name=实二十：记录头读者把末条标志位_0_读反（带标志的读成不是末条） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=34 row_name=步_5：抬_F_生效之后不放开扣住的槽 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/498 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=42 row_name=实二（回退改形态：SysPre_与_B1，D16_已定项_1_/_7、D22_已定项_7_/_9、I-7.9_/_I-7.12、C556_/_C557）：卸载把_F_抬到现行那一版带的_F（原地不动）而不是它的_txg o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/537 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=92 row_name=步_1_验收第_4_条：inode_记录的写入时间留成_A_的（覆盖写照抄上一版的写入时间） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/61 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=112 row_name=步_3：前缀跨实例边界（把别的实例的记录也接上） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_beta counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=19 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_beta caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_core_row closing=O1 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=no_executor row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=0 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=7 q1_beta_all_quiet_rows=7 q1_gamma_all_quiet_rows=7 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=11 q3_beta_rows_held_by_a_single_oracle=11 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
```

分族、分類の計数（1 つのコマンドが結果から数を数え、そのまま出力する）：`grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` は `2 compile_failed`、`19 counted`、`3 self_crash`、`93 untouched`（4 行）を出力する。分母に含まれない 5 行：第 6、7、85 行は自己崩壊を実装し、第 33、59 行はコンパイルに失敗した。登録 5.2 に「1 つの行がコンパイルに失敗……同じファイルグループ内のインデックス j + 1 に変更する」と記す。s3、s4 のバージョンでは、行ごとに駆動はコンパイル前の失敗（見つからない、名前が一意でない、アンカーポイントが一意でない）に対して補完を行うが、第 33、59 行には補完が行われない；駆動が第 8 セグメントでコンパイルに失敗した後にも補完を行い、これらの行および拡張表でコンパイルに失敗した 4 行の補完を行ったラウンドは既に実行済みである（「コンパイルに失敗した 6 行の補完（実行済み）」セクション参照）。

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:88-103`（整段抄，未转述）**

```markdown
### 3 つの完全解放の 7 行

読み取り値は s4 のエクスポートディレクトリ（`scripts/experiment-apparatus/target/e171-dumps/rows-s4/row-<序号>/dump/`）から取得し、`baseline-run1` と操作ごと・状態ごとに比較する。草案スクリプト `quiet_rows_s4_fua.py` および `returned_check.py` はエクスポートディレクトリのみを読み取り、データベースへの登録は行わない。三分類には `.claude/rules/mutation-sampling.md` の判定基準をこのバッチに適用する。**このバッチで同等**とは、2 つのストリームおよびバッチ 1 とバッチ 2 の各状態と、その状態上での実装による復元が、条項が許容する集合に含まれることを意味する（根拠を記述；実行されなかった履歴は横に記す）；**サンプリング点で検出不能**とは、2 つのストリームのいずれかに条項が許容しない状態が存在する場合でも、サンプリング規則によってその状態が取得されていないことを指す；**真の盲点**とは、取得された状態の中に条項が許容しないものが含まれ、かつ 3 つのチェックすべてがそれを通過させた場合を指す。「ミラー像の違い」は `states.tsv` ファイル内のミラー像サマリーとベースラインの状態が異なる状態数を数えるものであり、「新版に選択された」は、そのうち実装がジャーナルから復元した際に見つかったルート txg が「最初の違いが発生した後書き出された最初のルート」以上である状態数を数えるものである。3 つのチェックそれぞれが確認する内容は以下の通り：O1（メモリモデルによる対照（HashMap 理想ファイルシステム））は、各呼び出し後に実装から返却される `ObservedRoot` を比較する（定義は `crates/singlefs-harness/src/model.rs` にあり、ルートキー、ジャーナルカウンター、ルートに付随する F、ファイルコンテンツ、インスタンステーブル、および各ディスクの割り当て記録を含む）；O1（メモリモデルによる対照（HashMap 理想ファイルシステム））-クラッシュ判定官は、実装が復元して選択したルートと読み戻されたコンテンツを比較する；O2（独立解析器 + チェッカー）は単一のミラー像の構造を検証する；O3（独立規約実行器）の許容集合 A(c) は、その行自身が書き出したルートスロットによって構成される（説明は `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/version_table.py` のファイルヘッダーにある）。1 バージョンではルートの 6 つの要素のみを投影する（説明は `root_ring.py` の `VersionProjection` にあり、インスタンス識別子、チェックポイント txg、ツリーテーブルユニットポインタ、中央マッピングツリールートポインタ、ツリー ID レベル、および F を含む）。検証対象は、ミラー像から復元したそのバージョンが A(c) 内にあるかどうかであり（判定ロジックは `executor_core.py` の `judge_state` 内の `landed_is_allowed =` という行に実装されている）。「3 つのチェックすべてを通過した 7 行」の表における「三分類」列は、3 回のラウンドの結論判決後の現在値であり（旧「三分類」列の変更点は「履歴バージョン」セクションに記載）、各行の帰属全体像は「3 回のラウンド判決後の三分類」セクションで詳述されている。

| 行（変異） | ベースラインとどこが異なるか | どの状態に該当するか | O1（メモリモデルの対照（HashMap 理想的なファイルシステム））/O2（独立したパーサー + チェッカー）/O3（独立した仕様実行器）で検知できるかどうか | 三分類 |
|---|---|---|---|---|
| 15（ユーザーデータの配置先がこの図のユニット領域の起点からスキャンされない） | 2 つのストリームの第 22 番目の操作以降：ユーザーデータユニットは各ディスクで 32 KiB の配置先 822149120 → 823132160 にあり、内容はバイト単位で同一；それを指す 16 KiB のノード内のポインタとチェックサムもそれに従って変化する；同一次リリースの 11 つのコミット内生ブロック（t2–t12）も全体として 64 スロット分後方へ移動し、開放区間 [50240, 50304) が全体で [50304, 50368) に移動する：デバイス 0 上では 12 回の書き込みが配置先を変更した（第 22 番目の操作 50180 → 50240、第 24 番目 50240 → 50304、第 26 番目 50242 → 50306、……、第 44 番目 50252 → 50316；2 つのストリームは同一であり、`row15-offsets.out` 内の 2 行 `name=row15_first_publish`）；ストリーム 2 の第 4 回目の呼び出し以降の戻り値の位置が 4 回の操作分後方へ移動（`mount_writable` 92–179 → 92–183）、合計で 555 回の操作が異なる；呼び出しの結果は同一；ジャーナルを参照して状態ごとの復元を確認すると同一 | ミラー像が異なる：ストリーム 1 はバッチ 1 で 1 20 状態、バッチ 2 で 7 状態、ストリーム 2 はバッチ 1 で 1 364 状態、バッチ 2 で 119 状態；新しいバージョンを選択した場合：ストリーム 1 はバッチ 1 で 1 12 状態、ストリーム 2 はバッチ 1 で 1 356 状態、バッチ 2 で 112 状態 | O1（メモリモデルの対照（HashMap 理想的なファイルシステム））：`ObservedRoot` に配置先が含まれておらず、読み戻しは同一であり、検知できない；O2（独立したパーサー + チェッカー）：配置先がユニット領域内にあり、帳簿が自己整合しており、緑判定；O3（独立した仕様実行器）：投影に配置先が含まれておらず、「存在する」 | **真の盲点候補**：根拠は D3（空間割り当て）で既に決定された項目 10 ⑤ の t2–t12 の配置先である：「⑤ は新しいプールでの新規ファイル作成における配置先（開放区間 [50240, 50304)、バンプカーソルは 50240 から始まる）」（`.claude/kb/decisions/03-空間分配.md`「決定事項 10：クラスタ領域の定義、ユーザーデータ配置先の四格、コミット内生ブロックのバンプ順序」の決定）、射程「新規プールの新規ファイル作成のバイトを変更する：はい」「11 つのコミット内生ブロック（デフォルトの幾何）の配置先を全体で再配列」（226 行目）、バイト表「t2 から t12 順に 50240、50242–50243、50244、…」（`.claude/kb/layout/01-first-txn.md`「五、空間帳簿と割り当て」内の「コミット内生ブロック」という行）；変異下ではユーザーデータが 50240 に配置され、11 つのコミット内生ブロック全体が 64 スロット分移動し、⑤ で釘付けされた配置先とは一致しないため、3 つとも許可される。もう一つの根拠（ユーザーが 2026-10-03 に決定した後に追加されたもの）：D3（空間割り当て）の決定事項 8 の射程「例外は**mkfs の水位で初めてツリーを構築したバージョンから**」「新規プールの新規ファイル作成はこのカテゴリーに該当する」、t1@50180「`[layout/01-first-txn.md](../layout/01-first-txn.md)`のバイト表に登録されており、フォーマットレベルであり、そこから書き出されたこの種のミラー像は不正である」（`.claude/kb/decisions/03-空間分配.md`「決定事項 8：ユーザーデータブロックがクラスタ領域に入らず、配置先は選択済みデバイス内での最小の空スロット」の射程）；変異では t1 を 50180 から 50240 に移動させており、それ自体が不正である；3 回の判定で去就を決定する時点ではこの根拠はまだ存在しておらず、⑤ のみ relied している。分岐する歴史（推測されるもの）：ユニット領域がちょうど十分すぎるプールの場合、冒頭の区間をスキップした後、モデルが成立すべき場所に公開された際に空間不足を報告し、O1（メモリモデルの対照（HashMap 理想的なファイルシステム））-対照では赤判定 |
| 23 (分裂した右側コンテナ番号は、分裂をトリガーした新レコードの inode 番号ではない) | 2 つのストリームの第 26 番目の op から：最初のファイルのそのパブリッシュで書かれた inode リーフ（コード 3、ディスク各 32 KiB）オフセット 53 の 1 バイトが 0x01 → 0x00 (コンテナ番号 1 → 0) に変わり、ヘッダーチェックサムもそれに従って変わる；同パブリッシュの第 28 番目の op（16 KiB ノード）ではオフセット 52, 60, 131, 149 などでの 0x01 → 0x00 が起こる；その後、inode ツリーを再書き込む各パブリッシュは (ストリーム 2 の 155 番目の op が異なる) を伴う；呼び出しの結果も同じ；回復の逐次状態も同じ | イメージが異なる状態：ストリーム1はバッチ1で20状態、バッチ2で7状態。ストリーム2はバッチ1で364状態、バッチ2で119状態。最初のファイルのバージョン（txg 3）以降を選択した状態：ストリーム1はバッチ1で12状態、ストリーム2はバッチ1で356状態、バッチ2で112状態 | O1（メモリモデルの対照 (HashMap の理想的なファイルシステム)）：ディスク読み込みなし、`ObservedRoot` にコンテナ番号が含まれていないため、読み戻しは同じ；O2（独立したパーサー + チェッカー）：I-9.4（コンテナ番号が最小キーを超えない）は「inode 番号 ≥ コンテナ番号、リーフ順序に沿って厳密に増加」であれば、1 リーフの場合 1 ≥ 0 が成立し、緑判定；O3（独立した仕様実行器）：投影は inode ツリーに入らない、「存在する」 | **真の盲点候補**：D8（コアインデックス構造）で既に決定された項目 6 は「**新しいプールでの新規ファイル作成**：コード 2 のルートノード 1 つ（ヘッダーにキー範囲 [1, 1] を持つ；エントリ 1 つ：キー 1、タイプ 2、出生ツリー、コンテナ番号 1、出生世代 3、ポインタ）；コード 3 のリーフ 1 つ（タイプ 2、コンテナ番号 1、出生世代 3、レコード数 1、レコード幅 140）」（`.claude/kb/decisions/08-核心索引结构.md`「決定された項目 6：inode ツリーのリーフはコード 3 のパック済みレコードユニットであり、レコードは固定長 140 で、公開済みの (inode 番号, オブジェクトの出生世代) は再利用されない」の決定）を射程として書き、「各コンテナ番号は某レコードが最初に挿入された際の inode 番号である」（第 205 行）と記す；コンテナ番号 0 はどのレコードの inode 番号でもない。新版を選択した状態は、条項から導き出せるミラーには含まれておらず、3 つすべてを許可する。O2（独立したパーサー + チェッカー）で赤判定となるサンプリングポイント（推測であり実行していない）：リーフが満杯になった後の分裂で、右側のコンテナ番号 0 が左側より小さい場合、I-9.4（コンテナ番号が最小キーを超えない）の「リーフ順序に沿って厳密に増加」が赤判定となる |
| 27（レコードヘッダーの読取側が最後のフラグビットを反転して読み取る） | 書き込みストリームとコールの結果は完全に同一である。違いはジャーナルからのみ復元する段階にある：ストリーム1の6状態、ストリーム2の51状態（バッチ20状態）において、ベースラインでは1つのレコードが適用され新しいバージョンが選択されるが、変異体では最後のフラグを持つレコードを「後続がある」として読み取り、0つのレコードを適用し前のバージョンで停止する。戻り値の内容が異なる3/30の状態（例：ストリーム1の41状態ベースライン `1,3 file_read e8ca4bf8…`、変異体 `1,2 no_file`） | 半途中に書き込まれた状態：破棄されたバージョンのコールは返ってこない、そのルートスロットも着地していない（`returned_check.py` の57状態すべてで `call_returned=False its_root_slot_landed=False`） | O1（メモリモデルの対照（HashMap 理想ファイルシステム））-クラッシュ判定官：前のバージョンがモデルにコミットされており、ディスク上に永続化された最新のルートスロットより古くない場合、読み戻した内容がそのバージョンと一致すれば緑判定は正しい；O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照：復元を経ずにコールを実行する；O2（独立解析器 + checker）、O3（独立仕様実行器）：ミラーとベースラインをバイト単位で比較し、それぞれ実装の復元を行わない | **別の種類：実装の復元に依存してレコードから再構築するセグメントは判定されない**。状態自体は合法である（コールが返ってこないため、新旧両方のバージョンが許可集合内にある）が、実装の復元で1つのレコードを適用しなかったことが誤りであり、D13（検証経路）の既定義項1のM4「`Recover` が返すルートは j* であること」と D23（ジャーナルの役割と形式）の既定義項15「レコードからの再構築でそのリリースのルートを決定する」、既定義項14「復元では `(インスタンスID, チェックポイント_TXG)` が選択されたルートより厳密に大きいレコードのみを適用する」に違反する。O1（メモリモデルの対照（HashMap 理想ファイルシステム））-クラッシュ判定官の基準（モデルにコミットされたあるバージョン、ディスク上に永続化された最新のルートスロットより古くないこと）は M4 より弱く、O2（独立解析器 + checker）は復元前のミラーを判定し、O3（独立仕様実行器）は実装の復元を実行しない。M4 の着地点は E165（復元再生の有界な網羅探索と変異殺傷境界）および Verus であり、これら3つではない。分岐するサンプリングポイント（推測されたもの、実行されていないもの）：すでに返り済みで、コミットポイントはジャーナルレコードのみにある（ルートスロットはまだ書き込まれていない）リリース後の状態では、変異体は1回の返り済みのリリースを破棄する；両ストリームのリリースがどちらも返る前にルートスロットに書き込まれるような状態は存在しない。その時、O1（メモリモデルの対照（HashMap 理想ファイルシステム））-クラッシュ判定官の「ディスク上に永続化された最新のルートスロットより古くならない」という条件も防げない（ルートスロットがまだ書き込まれていないため）、3つすべてが通過されるかどうかは計測されていない |
| 34（F を有効にした後、ロックされたスロットを解放しない） | ストリーム1では何の変化もない；ストリーム2の498番目のオペレーションから（F を上げた後のその `publish`）：ユーザーデータユニットの着地点が 822083584 から 822411264 に変更され、内容は同一である；それを指すノードと割り当てレコードも変化する（51 のオペレーションで異なる）；コールの結果は同一；状態ごとの復元も同一 | ミラーが異なる：ストリーム2の68状態、バッチ2の21状態；txg 17 以降のルートを選択した場合：バッチ1の60状態、バッチ2の14状態 | 第15行と同様：O1（メモリモデルの対照（HashMap 理想ファイルシステム））は着地点を比較しない；O2（独立解析器 + checker）は緑判定（帳簿の自己整合性）；O3（独立仕様実行器）は「存在する」 | **別の種類：割り当てポリシー、ミラーが合法**：F を上げた後のリリースで空きスロットを変更し、回収されたスロットを再利用していないため、各状態の割り当てレコードと帳簿の自己整合性が保たれている；D3（空間割り当て）の既定義項8の射程「それ以外の書き込み（上書き書き込み、後のリリース、再マウント後）においてそれを違反する実装が出力したミラーは合法であり、それを阻止するにはランタイムでの読み取りに頼る必要がある」（`.claude/kb/decisions/03-空間分配.md` の既定義項8「ユーザーデータブロックはクラスタセグメントに入らず、着地点は選択されたデバイス内における最小の空スロットである」の射程；新しいプールで新規ファイルを作成するセルはこの例外であり、この行の変更はストリーム2で F を上げた後のリリースに適用されるが、その中には含まれない）、3つとも設計通りに割り当てポリシーを判定しない。分岐する履歴（推測されたもの）：F を上げた後では再利用可能なスロットのみが確保できるプールの場合、リリースで空間不足を報告するがモデルは許可する場合、O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照は赤判定になる；この行は `crates/mutations.tsv` で指定されたテスト `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` が直接再利用を断言している |
| 42（アンマウントで F をその時点のバージョンに持った F に引き上げるのではなく、その txg にする） | ストリーム一は変化なし；ストリーム二では第537操作 から開始（アンマウント呼び出し時の最初の書き込み：2 ディスクシステム構成スロット、オフセット 481 バイト 0x11 → 0x0b、すなわち F 17 → 11、チェックサムを含む）、その後アンマウントが空の公開書き込みをプッシュし、その根とノード内でも同様に 11 となる（24 の op が異なる）；アンマウントは通常通り `ok`；回復状態は同一 | ミラーは異なる：ストリーム二バッチ1 は 48 状態、バッチ2 は 14 状態、全永続状態を含む；アンマウント書き出しの根（txg ≥ 18）を選択すると、バッチ1 は 32 状態、バッチ2 は 7 状態 | O1（メモリモデルのチェック（HashMap 理想ファイルシステム））：アンマウントにはモデルからの回答がない（登録読み取り法 6）。今回の呼び出しはチェックに失敗し、書き込みがアンマウント内の状態クラッシュ判定官記録に含まれていないため見えない（アンマウントからの回答があれば、`ObservedRoot` は根を持つ F で、17 と 11 の比較からプッシュされる）；O2（独立解析器 + checker）：システム構成の F と根に付随する F はどちらも 11 であり、I-7.12（システム構成 F は同ディスク上の根の F よりも低いことがない）が成立するため、単一のミラーではどちらが 17 なのか判断できない；O3（独立仕様実行器）：プロジェクションに F が含まれるが、A(c) はこの行自体が書き出した F = 11 の根であり、「存在する」 | **真の盲点候補**：D16（公開セマンティクス）は既決定項目 1 で確定したフィールドテーブル「正常アンマウント」行に「空の公開をプッシュして F をその時点のバージョンの txg に引き上げる」と記述している（`.claude/kb/decisions/16-发布语义.md`）。射程「正常アンマウントのコスト」条項には「アンマウント後、F = その時点のバージョンの txg となり、ロールバック候補セットはその時点のバージョンのみが残る」とある。変異下でアンマウント返答後の全永続状態 F = 11 となる場合、ロールバック候補セットには txg 11–16 の根が残り、これは条項が許可するミラーに含まれていないため、3 つのケースすべてが通過される。O1（メモリモデルのチェック（HashMap 理想ファイルシステム））の半分は登録読み取り法 6 で明記された既知のギャップである（モデルにはアンマウントからの回答がない）。
| 92（inode 記録の書き込み時刻が A に留保されている） | 流れ一は全く同じ；流れ二第61操作 から起（第二次 `publish`、その回の inode リーフを覆写する、2 つのディスクそれぞれ 32 KiB）：リーフ内のオフセット 200、208、216 の 3 バイトが 0x3c → 0x00 に変わる（第 0 条記録の起点は 136 で、+64 / +72 / +80 は atime、mtime、ctime の秒数：ベースライン 60、変異は A に留保された 0）、ヘッダー内のオフセット 10–13（ヘッダーチェックサム）と 89–92 も異なる；その後このリーフの書き込みを再公開する際もすべて（139 条 op が異なる）を含み、呼び出しの結果は同じで、復旧の状態も同じ | ミラーの違い：流れ二バッチ1 は 344 状態、バッチ2 は 112 状態；覆写されたバージョン（txg 4）以降を選択すると：バッチ1 は 336 状態、バッチ2 は 105 状態 | O1（メモリモデルの対照（HashMap 理想ファイルシステム））：`ObservedFile` は内容のみを持ち、`model.rs` と `model_comparison.rs` の中で `write_time` はゼロヒットし、D13（検証ルート）は既定義項 6 の表 O1（メモリモデルの対照（HashMap 理想ファイルシステム））で「内容、サイズ、ディレクトリ構造」を判定するが、見えない；O2（独立したパーサー + checker）：単一のミラーはこの公開の書き込み時刻パラメータを知らず、緑判定；O3（独立した規約実行器）：投影は inode ツリーに入らず、「存在しない」 | **別の種類：条項はバイトテーブルとマイルストーン受検で決まり、意思決定本文に欠落する一文がある**：D8（コアインデックス構造）は既定義項 6 のフィールド表がオフセット 64「atime / mtime / ctime 秒」の位置のみを定めた；「書き込み時刻は公開パラメータである」という記述は `.claude/kb/milestone/01-first-txn.md`「ステップ 3 inode ツリーとその他のツリー」の現状段落および `02-second-txn.md`「ステップ 1 覆写：同じファイル、同じプール、第二次公開（公開 B）」における inode 記録の行にあり、バイトテーブルは「書き込み時刻（mkfs パラメータ、再現可能）」と記す（`.claude/kb/layout/01-first-txn.md`「四。ファイルメタデータ（inode）」における「inode 記録（オフセット 64）」の行）、受検では「inode 記録の書き込み時刻が A に留保されている ⇒ 赤判定」（`02-second-txn.md` の同じステップにおける「変異：extent リーフ記録のポインタを忘れた」の行）であり、意思決定本文には値を取得する一文が見つからない。新しい状態（書き込み時刻 0、今回の公開パラメータは 60）を選択しても受検に合わず、3 つとも設計通りに見えない（書き込み時刻を返す API が存在しないため）、独立性の失敗とはならない |
| 112 (プレフィックスがインスタンス境界を越える) | 書き込みストリームとコールの帰結は完全に同一である。ジャーナルを参照して行われる復元において、2 つのストリームの各 3 状態（バッチ 1 の第 9–11 状態、セグメント 1：ウォームアップ中の最初の書き込み——新インスタンス 1 のジャーナル記録——が着いたが、そのルートスロットは着いていない）は異なる。ベースラインでは `0,0` を選択し、0 条を適用する。変異では、インスタンス 1 の記録をインスタンス 0 のルートに接続し、1 条を適用して `1,1` を得る。読み戻しにはファイルが存在しない | ウォームアップ中に書き込みが完了した状態（`returned_check.py` の 6 つの状態はすべて `call_returned=False`） | O1 (メモリモデルの対照 (HashMap 理想ファイルシステム))-クラッシュ判定官：`1,1` はモデルにコミットされたバージョンであり、既に永続化された最新のルートスロットより古くなく、読み戻しも一致するため、緑判定が正しい。O2 (独立したパーサー + checker)、O3 (独立した仕様実行器)：ミラーとベースラインはバイト単位で同一である | **別の種類：実装の復元において記録による再構築に依存する 3 つの状態（追加適用）**：ウォームアップが完了せず、`0,0` と `1,1` の両方が許可集合に含まれている。誤りは、実装の復元が他のインスタンスの記録も接続してしまう点であり、D23 (ジャーナルの役割と形式) に違反する。既定義項 14 注 1「プレフィックス規則はインスタンス境界を越えない」（`.claude/kb/decisions/23-journal の役割と形式.md`「既定義項 14：再プレイの下界は選択されたルートによって与えられる；管理者によるロールバック、失敗処理、およびインスタンス切り替え」）に基づいている。この行の変異は、ゲート checker-tier-formal-verification の赤判定固定例リスト（`.claude/gate.d/e165-pinned-red-examples.tsv` で 1 列目が `371a71190dce62e0…` の赤判定例）に含まれており、E165 (復元再プレイの有界総当たりと変異殺傷範囲) による有界総当たりによって判定される。分離するサンプリングポイント（推測され、実行されていないもの）：リング内に他のインスタンスを残し、選択されたルート上の時間線上にない記録（クラッシュ復元でルート以降の新しいインスタンスは破棄される）を接続すると、復元はモデルがコミットしていないバージョンに落ちる。O1 (メモリモデルの対照 (HashMap 理想ファイルシステム))-クラッシュ判定官はこれを赤判定する。2 つのストリームにはこのような状態はない |

第 102 行（F を先に書き込むシステム構成のステップに新しい F を含まない）は、バッチ 2 に余分に出た全行を許可するものである（`name=q1_alpha … rows_that_differ_from_batch_1=102`）：ストリーム 2 の第 440 番目の op（F を先に書き込むシステム構成スロット、オフセット 481 で 1 バイト 0x0b → 0x00）と第 537 番目の op（先に書き込んだシステム構成スロットをアンマウント、0x11 → 0x0b）は異なる。4 つの op；O2（独立したパーサー + チェッカー）はバッチ 1 で 12 の状態を赤判定し、すべて I-7.12（システム構成 F は同じディスクルート上の F よりも低くない）である。第 60、73 セグメントそれぞれで 6 つの状態があり（`name=trajectory row=102 stream=fixed_script_through_e_then_unmount oracle=O2（独立したパーサー + チェッカー） total_red_states=12 peak=6 positive_segments=2 of_segments=79 …`）。バッチ 2 はストリーム 2 で 17 セグメントのみを採取する（7、11、16、20、24、28、32、36、40、44、48、52、57、61、65、70、74）。第 60、73 セグメントの状態は一つも採取されていない（`awk -F'\t' '$3==60\|\|$3==73' states.tsv` はバッチ 1 の 8 + 8 つの状態のみを出力する）：**サンプリングポイントに不感応**、分離されるサンプリングポイントは第 60、73 セグメントのいずれかの状態である。このセルは両バッチで逆転し、「不安定」と記録する；Q1、Q3 の結論文字はバッチ 2 では逆転しない（バッチ 2 で全行を許可しても 8 行が依然として ≥ 1；単一の条件に依存する 10 行も依然として ≥ 1）。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:104-123`（整段抄，未转述）**

```markdown
### Q3 単一の 11 行のみによるもの

到達不能なオブジェクトと判定されていないフィールドのみを記述し、結論は判定しない。読み取り値は s4 のエクスポートディレクトリ（ドラフトスクリプト `caught_rows_s4.py`、データベースへの登録なし）および生成物の `name=trajectory` 行から取得する。セグメントが正の数は「赤判定のセグメント数 / このストリームのセグメント数」である。

| 行（変異） | その条令はどこにあるか | 他の二つの条令が見えない理由 |
|---|---|---|
| 32（書き込み可能マウントでインスタンス表を書かず、旧表をそのまま写す） | O1（メモリモデル照合〔HashMap の理想ファイルシステム〕）：ストリーム2の4回目の `mount_writable` が `disagrees:这一版的实例表` と判定する。モデルは前のインスタンスの行を要求するが、実装は空の表を返す。モデルはこの呼び出しで停止し、以後のストリーム2の324状態（バッチ1）はクラッシュ判定の対象外となる | O2（独立パーサーと checker）：この行で赤判定は0状態（バッチ1のストリーム2の397状態はすべて緑）。実装済み不変条件には、ルートのインスタンス番号をインスタンス表の行と照合するものがないと読み取れる。ただし `crates/singlefs-checker/invariants/` の各条件を逐一確認したわけではない。O3（独立仕様実行器）：投影にインスタンス表がなく、A(c) はこの行自身が書いたルートなので「含まれる」と判定する |
| 45（マウント時点で読めないスロットも再読し、なお不良なら拒否する） | O1（メモリモデル照合）：ストリーム2の12回目の `raise_floor` は `err:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread` を返すが、モデルは成功を期待する | O2とO3：拒否前の書き込みストリームはベースラインと操作ごとに一致する（ストリーム2は第440操作で停止し、異なる操作は0件）。取得したイメージはいずれもベースラインバッチのプレフィックスである。拒否された呼び出しはイメージを残さないため、単一イメージを判定するオラクルには対象がない |
| 108（コード2の読取器が最後のペイロードバイトからパディングバイトを数える） | O1（メモリモデル照合）：両ストリームの2回目の `publish` で `disagrees:这一版的文件内容` と判定する。結合コードは実装が返すユニットを解読できず、「パディングバイトが非ゼロ」とする。ストリーム2の4回目の `mount_writable` は `err:Recovery` を返す | 誤りは実装の読取器側にあり、書き出したバイトはベースラインと同じ（ストリーム1の異なる操作は0件、ストリーム2も第92操作より前に異なる操作は0件）。O2は独自のパーサーで生バイトを読み、core の読取器を呼ばない。O3も Python 製の独自読取器を使うため、両者が見るイメージはベースラインとバイト単位で一致する。クラッシュ点再生用の手書きバージョン表のオラクルとレコード照合器は赤判定となる（`layer0_or_record_checker_red=true`。三つのオラクルには含まれない） |
| 115（プレフィックス選択の核アンカーが txg を見ない） | O1（メモリモデル照合）のクラッシュ判定：バッチ1のストリーム1で32状態、ストリーム2で52状態が「走査失敗：ルートを選べない」（実装の回復は `RootPublishCarriesMoreThanOneLastRecordFlag…` を返す）。赤判定のある区間は8/12、12/16。O1の照合：ストリーム2の4回目の `mount_writable` は `err:Recovery` を返すが、モデルは成功を期待する | 書き出したバイトはベースラインと同じ（ストリーム1の異なる操作は0件）。O2はイメージの構造を判定し、O3は独自のプレフィックス規則でイメージから回復するため、いずれも実装の回復処理を参照しない。見えるイメージはベースラインとバイト単位で一致する。クラッシュ点再生用のオラクルとレコード照合器は赤判定となる |
| 8（割り当てても空き容量を減らさない） | O2（独立パーサーと checker）：I-5.2（空き容量の統計が一致するか）で、バッチ1のストリーム1は9状態、ストリーム2は353状態が赤判定。赤判定のある区間は2/12、69/79。バッチ2のストリーム2は112状態 | O1（メモリモデル照合）：`ObservedRoot` に会計行がなく、呼び出しはすべて成功し、内容も正しい。O3（独立仕様実行器）：投影に会計ツリーがないため「含まれる」と判定する |
| 65（blocks を切り下げる） | O2（独立パーサーと checker）：I-9.15（inode レコードの blocks は ⌈size ÷ 512⌉ に等しい）で、バッチ1のストリーム1は9状態、ストリーム2は353状態が赤判定 | O1（メモリモデル照合）：`model.rs` と `model_comparison.rs` に `blocks` の参照がなく、読み戻した内容も同じ。O3（独立仕様実行器）：投影に inode ツリーがない |
| 86 (システム構成スロット書き込み時に F の 8 バイトを 0 にする) | O2（独立解析器 + checker）: I-7.12（システム構成の F は同ディスク上のルートと同じかそれ以上である）バッチ 1 ストリーム 2 97 状態、セグメントは正 19/79；バッチ 2 28 状態 | O1（メモリモデル対照（HashMap 理想ファイルシステム））: 比較対象はルート帯の F（`ObservedRoot` の `rollback_floor`）であり、システム構成スロット内の F は比較しない；O3（独立仕様実行器）: プロジェクションされた F はルートから取得され、システム構成スロットは幾何学的選択のみに行われる。A(c) はこの行自身が記述したルートであり、「存在する」 |
| 87 (core のその版が上限を飛んで F を 1 つ多算する) | O2（独立解析器 + checker）: I-8.1（リングの幾何学が十分大きい）バッチ 1 ストリーム 1 51 状態、ストリーム 2 385 状態；I-7.13（システム構成プールのフィールドはリーダの受け取り範囲内にある）ストリーム 1 2 状態、ストリーム 2 12 状態；セグメントは正 12/12、79/79 | O1（メモリモデル対照（HashMap 理想ファイルシステム））: システム構成スロットのフィールドと比較しない；O3（独立仕様実行器）: 第 86 行と同様、プロジェクション内には上限を飛ぶものはない |
| 90 (新しいプールで新規作成されたファイルの変更カウント書き込みが 1 に戻る) | O2（独立解析器 + checker）: I-9.26（inode の変更カウントは出生世代とコンテナの誕生世代コードの間にある）バッチ 1 ストリーム 1 9 状態、ストリーム 2 240 状態 | O1（メモリモデル対照（HashMap 理想ファイルシステム））: inode に記録された変更カウントと比較しない；O3（独立仕様実行器）: プロジェクションは inode ツリーには進まない |
| 99 (defer 行に 0 を書き込む) | O2（独立解析器 + checker）: I-3.11（割り当て済みから defer を引いた値が最新ルートの読み取りと一致する）バッチ 1 ストリーム 1 9 状態、ストリーム 2 353 状態 | O1（メモリモデル対照（HashMap 理想ファイルシステム））: 帳簿行と比較しない；O3（独立仕様実行器）: プロジェクションは帳簿ツリーには進まない |
| 102 (F を上げる際、システム構成への書き込みステップに新しい F は含まれない) | O2（独立解析器 + checker）: I-7.12（システム構成の F は同ディスク上のルートと同じかそれ以上である）バッチ 1 ストリーム 2 12 状態、第 60 セグメントと第 73 セグメントそれぞれ 6 状態；バッチ 2 は赤判定にならない（サンプリングポイントに不感であり、「三条全件通過通行の 7 行」節の末尾を参照） | O1（メモリモデル対照（HashMap 理想ファイルシステム））: `raise_floor` の答えは書き出されたルートと比較される（ルート帯の F が正しい）、システム構成スロットとは比較しない；O3（独立仕様実行器）: 第 86 行と同様

O3 (独立仕様実行器) は、s4 の分母に含める 19 行すべてにおいて赤判定を行っていません (`name=q1_column_red_rows … O3（独立仕様実行器）甲=0 O3（独立仕様実行器）乙=0`)。一方、O3 (独立仕様実行器) は、各行の 2 つのストリームにおける 450 のバッチ状態すべてに対して厳密な判定を下しています (`o3_judged_states_strict=450`、`name=q1_o3_coverage … rows_with_no_strict_judgement=0`)。実行器の実装から読み取れる 1 つの事象（読込のみ、別の実行はなし）：A(c) はこの行自身によって書き出されたルートスロットのみで構成されており、プロジェクションにはルートの 6 つの要素しかないため、O3 (独立仕様実行器) は「自己の回復規則と条項が異なる場合」または「一度のリリース後にクラッシュの後半に適用される場合」というカテゴリにおいてのみ赤判定を行います。実装が特定のフィールドを一貫して誤って書き出した場合でも、O3 (独立仕様実行器) は依然として「在（存在）」と判定します。O3 (独立仕様実行器) は、不良イメージの一族（Q2）に対して多くの赤判定を下しています：P 行では 26 のうち 22 が赤判定、ブラインドテストでは 40 のすべてが赤判定、既知の不良イメージでは 41 のうち厳密な赤判定が 28 です (`name=q2_family` の 4 行)。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:124-127`（整段抄，未转述）**

```markdown
### S3 テーブルの無効化

S3 の回において、表全体（出力ルート `scripts/experiment-apparatus/target/e171-dumps/rows-s3/`）の 2 つのラインはそれぞれ跨行してコンパイルディレクトリを再利用した；変更されたファイルを復元するには `copy2` を使用し、同期には `rsync -a` を使用し、どちらも修正時刻をスナップショット時点に保持する。Cargo は修正時刻に基づいて新旧を判定する。ライン 1 は第 112 行（replay-prefix の変異）を実行した後、第 114 行を実行する前の 45 行を実行し、生成された装置はすべて第 112 行の変異を保持している。修正 5 の登録は正確にこの 45 行、Cargo 1.98.0 におけるメカニズムの実測とドライバの変更を記録した；S3 表全体、S3 の Q3-核の各パスおよび S3 との比較器の結果は、Q1、Q3、Q4 の読み取り値として扱われない。アーカイブし削除しない：`research/results/` 下にある `e171-three-oracle-collusion-2026-10-03-s3-comparator.out`、`…-s3-rows-timing.out`、`…-s3-clean-check-comparator.out`（第 5 セクションにおける第 2、27、42、65 行のクリーンチェック）、および `scripts/experiment-apparatus/target/e171-dumps/rows-s3*`、`rows-s5-clean-check`。S3 が読み取った「Q1 で全 40 行が許可され、分母は 52」のうち、38 行は変異を保持した状態で実行された；S4 は修正済みのドライバ上で全 117 行を再実行し、判定行 `rows_compiled_without_the_rebuild_check=0` とした。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:128-136`（整段抄，未转述）**

```markdown
### F4 拡張テーブル

第 10 節 F4 の登録：「触達行数が不足し、Q1 の『なし』では判定に不十分：第 3 段落の表拡張（5.2）に進み、それでも不足すれば『不十分』と記録し、『なし』は記録しない」という規約実行器をトリガーし、「Q1-分母 D < 30」の観測が引き起こされる；s4 の D = 19。表拡張は登録 5.2「表拡張」に従う：ステップ幅 4 の選び方は、ステップ幅 8 よりも 164 行多く（ステップ幅 8 と重複する 56 行は再実行しない）、選び方スクリプトの出力順序に従い、117 の後に接続して 118–281 に番号を割り当て、刀番 3 を付与する。

- 行ファイル `scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv`（sha256 `f1bd082e…`）は、同一ディレクトリの `expansion_table.py` スクリプトを 2 回実行（ステップサイズ 8、ステップサイズ 4）によって生成されたものである。表は `crates/mutations.tsv` から E171（三 oracle 合謀テスト）の装置自身の変異行を取り除いたもの（a6、a7 と同じ除去方法）。s4 の凍結したツリー版と 2026-10-03 リポジトリ内の版ではバイト単位で同一である。ファイル別件数：admission 5、allocation_record_tree 2、allocator 12、block_device 2、code_two_tree 3、extent_tree 2、inode_tree 1、instance_table 2、journal 2、make_filesystem 6、mount 44、mounted_read 3、mounted_session 2、records 2、recovery 28、root_record 1、system_configuration 3、transaction 34、unit 2、replay-prefix 8。登録された sha256 は 5.9 のみで 117 行の行文であり、拡張表各行の行文ハッシュは表生成時の値を使用する。
- 行ごとに駆動に `--other-selected-rows` を追加：代替実行時に 117 行と拡張表内の各名前をスキップ（登録 5.2「拡張と代替実行で既に実行済みの行を重複して選択しない」）。
- コンパレータ `--rows-root` は 2 つのデータセット（117 行版と拡張版、同じ行番号が両方に存在する場合は拒否）を受け取り、`--expansion-rows` は拡張時に追加で 2 つの結果を返す：`name=f4_expansion`（拡張と選択スクリプトが `--mutations-table` で再計算した行ごとに同一、164 行、連続したインデックス、刀番号 3、117 行と重複しない）および `name=q1_denominator_by_table`（分母をテーブルごとに分割して報告）。Q1 の分母は 3 つの読み方、Q3 はすべて統合された 281 行に基づいて計算される。s4 の入力を用いて修正したコンパレータを再実行（rows-s4 のみ提供）、E7RESULT 行と `…-s4-comparator.out` がバイト単位で同一になる（`cmp` で出力なし）。
- 拡張版は s4 の凍結ツリー版上で実行される（スナップショットハッシュ `d1586465…`、s4 の 5 回駆動と同一）、両半分を同一バッチに統合可能。ツリーは再凍結しない。メインエージェントは第 7 段落で提出されたスクリプトに従って 7 ステップを実行（空変異 1 行が `baseline-run1` に到達せず、拡張と選択スクリプトが凍結ツリーの表上で再計算したバイト単位で同一、今回の 2 回駆動と s4 のスナップショットハッシュは 1 つのみ）、拡張版の 2 つのラインは 7 607.9 秒を要する。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:137-169`（整段抄，未转述）**

````markdown
### F4 拡張の結果（全行は生成物から直接コピー）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`(4,698 行、末行 `E7RESULT name=done emitted=4698`; 耗时行は `…-s4-f4-rows-timing.out` にあり、291 行)。並表、分母、3 つの読み方、Q3、判決行、および拡張表で新たに全通過となった行（甲 5 行、丙に追加された第 233 行）:

```
E7RESULT name=config segment=s4-f4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4+rows-s4-f4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=f4_expansion rows=164 expected=164 first_number=118 last_number=281 numbers_are_consecutive=true slice_3_rows=164 names_shared_with_the_117_rows=0 found_once_in_today_table=164 equal_to_the_selection_script_on_today_table=true
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=232 only_in_the_ignored_pass=0 compile_failed=6 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=12 rows_run=281 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=37 O1=12 O1-对拍=11 O1-崩溃判官=4 O2=15 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=2 row_numbers=155,161
E7RESULT name=q1_alpha counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_beta counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=37 all_quiet_rows=13 row_numbers=15,23,27,34,42,92,112,233,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=37 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q3_beta caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q1_denominator_by_table table=step_8_table rows_run=117 counted_D=19 counted_row_numbers=8,11,15,23,27,32,34,42,45,65,86,87,90,92,99,102,108,112,115
E7RESULT name=q1_denominator_by_table table=step_4_expansion rows_run=164 counted_D=18 counted_row_numbers=135,172,233,234,237,238,242,244,245,249,250,258,265,269,270,271,277,281
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=2 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=12 q1_beta_all_quiet_rows=12 q1_gamma_all_quiet_rows=13 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=23 q3_beta_rows_held_by_a_single_oracle=23 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
E7RESULT name=q1_quiet_row reading=q1_alpha row=242 row_name=增补_3_第_4_件（增补_2_收口表第_40_行_/_C381_的前提）：发布的持久顺序把系统配置槽轮换挪到根槽_FUA_写之前，最后一步失败时根就不会已经落盘 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=516 o3_judged_states_provisional=516 first_operation_that_differs_from_the_baseline=8/8 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=269 row_name=C554_乙-配置续：见证值取成槽世代号（mkfs_之后第一次取号写的_tail_不再是_0，新池新建文件的字节变了） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=0/0 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=270 row_name=C577：发布在系统配置轮换之后不过屏障就返回（树表_0_条的一版上返回之后见证值可以落后） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=390 o3_judged_states_provisional=390 first_operation_that_differs_from_the_baseline=20/20 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=271 row_name=D8_已定项_8_头_ID：deleted_inodes_的树表条目头_ID_退回写_0（不写它服务的_inode_树的树_ID） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=44/44 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=277 row_name=取前缀核单测：施加一次发布之后新根段不换（停在所选根的） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_gamma row=233 row_name=由记录重建的根不照抄所选根的文件系统标识（代码轮_m3-formal-code-r1_K3，R3-1「其余字段等于所选根」）：改写全零 o1_comparison_red_calls=0 o1_crash_judge_red_states=45 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=true
```

分類の計数（1 つのコマンドが出力から数を数え、そのまま出力する）：`grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` は `6 compile_failed`、`37 counted`、`6 self_crash`、`232 untouched`（4 行）を出力する。分母に含まれない 12 行は：第 6、7、85、125、235、272 行が自己崩壊し、第 33、59、127、163、206、243 行がコンパイルに失敗する（このデータは代替実行前の読み取りであり、代替実行のラウンドについては「コンパイルに失敗した 6 行の代替実行（実行済み）」セクションを参照のこと）。第 1–117 行の結果行（`name=row`、`row_touch`、`row_matrix`、`trajectory`、`row_o3`、`row_s5`）は F4 出力と s4 出力でバイト単位で同一である（1819 行、`cmp` は出力なし）。7 行すべてが解放された `name=q1_quiet_row` の 3 つの読み取り方法は各行で同一であり、s4 の 7 行については再実行は不要である。Q2、F3、および対照の結果行も 2 つの出力でバイト単位で同一である。8 セグメントで修正された比較器を用いて F4 と s4 の入力を実行し、E7RESULT 行はそれぞれの 2 つの出力とバイト単位で一致する。

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:170-184`（整段抄，未转述）**

```markdown
### F4 新しい全解通過

読み取り値は F4 のエクスポートディレクトリ（`scripts/experiment-apparatus/target/e171-dumps/rows-s4-f4/row-<序号>/dump/`）から取得し、`baseline-run1` と逐条 op および逐態を比較する。草稿スクリプト `quiet_rows_f4.py`（第 7 段落の `quiet_rows_s4_fua.py` を変更してルート出力とする）、`extra_checks_f4.py`、`order_by_segment_f4.py` はすべてエクスポートディレクトリのみを読み取り、データベースへの登録は行わない。3 分の口径は「3 条を全通過する 7 行」の節と同じである。順序に関する 2 行（第 242 行および第 270 行）において、書き込みストリームの条数または順序が変更されているため、逐態とベースラインとの比較は意味をなさない。代わりに、1 回の判定法を用いてその行自身の状態上で「規定に反する状態」の数をカウントし、同様の判定法をベースラインに対して適用してもすべて 0 となる（判定法は区別可能である）。セグメント切り分けには、コンパレータ固有の `baseline_checks.segments_of_the_write_stream` を使用し、FUA の書き込み通過およびバリアの通過を行うディスク領域を対象とする。

| 行（変異） | ベースラインとどこが異なるか | 条項が許さない状態（バッチ 1/バッチ 2） | O1（メモリモデルのチェックサム（HashMap 理想ファイルシステム）） / O2（独立したパーサー + チェッカー） / O3（独立した仕様実行器） が検知するか | 三分類 |
|---|---|---|---|---|
| 242（公開された永続順序が、システム構成スロットをルートスロットの FUA 書き込み前にローテーションして移動させる） | 2 つのストリームの 8 番目の操作から：毎回番号を取得し、ウォームアップ、公開の末尾にある「ルートスロット FUA → 2 つのディスクのシステム構成スロット → バリア」を「2 つのディスクのシステム構成スロット → ルートスロット FUA → バリア」に置き換える。書き込まれる内容は各項目ごとに同じだが、順序が変わる（ストリーム 1 は 9 個の操作、ストリーム 2 は 57 個の操作がインデックス順で異なる）。ベースラインではルートスロット FUA が独立したセグメントを形成しているが、変異はそれをシステム構成書き込みと同じセグメントに切り込む（ストリーム 1 の 8 セグメントのうち 3 セグメント、ストリーム 2 の 59 セグメントのうち 19 セグメント、ベースラインでは 0 セグメント）。呼び出しの結果は同じ | 同じセグメント内でシステム構成書き込みが確定しているが、ルートスロット FUA が完全に確定していない：ストリーム 1 は 21/3 状態、ストリーム 2 は 133/19 状態；ベースラインでは 0 | O1（メモリモデルのチェックサム（HashMap 理想ファイルシステム））-クラッシュ判定官：ルートスロットが確定しておらず、復旧時に前バージョンに選択される。前バージョンはモデルでコミットされたもので、既に永続化された最新のルートスロットより古くないため、緑判定；O1（メモリモデルのチェックサム（HashMap 理想ファイルシステム））-チェックサム：呼び出しの結果は同じ；O2（独立したパーサー + チェッカー）：この行では 0 個の赤判定（単一ミラー内でシステム構成スロットがルートより 1 回早く公開されており、`crates/singlefs-checker/invariants/` にこのケースを管理するものがあるかどうか逐項目チェックしていないため）；O3（独立した仕様実行器）：A(c) はこの行自身が作成したルートで構成され、復旧時に確定されるバージョンはその中にあり、「存在する」 | **真の盲点候補**：D16（公開セマンティクス）は既に決定事項 7 で「1 回の公開の永続順序は常に COW ユニット/ノード → バリア → ジャーナル記録 → バリア → ルートスロット（FUA）→ システム構成スロット → バリア」「システム構成スロットはルートスロットの永続化後に更新される」と定めた（`.claude/kb/decisions/16-公開セマンティクス.md`「決定事項 7：公開の永続順序」）。システム構成がその証言するルートより先にディスクに書き込まれる状態は、条項で定められた書き込み順序では生成できないため、3 つの方法すべてがこれを許可する |
| 269（见证値をスロット世代番号に変更） | ストリーム 1 の最初の 0、1 番目の操作（mkfs 後の最初の番号取得時に書き込まれる 2 つのディスクシステム構成スロット）のオフセット 469 の 1 バイト 0x00 → 0x01 に変更し、チェックサム（155–158）も更新する。ストリーム 2 には第 92、93 番目の操作（書き込み可能なマウント時の番号取得時に書き込む）があり、0x04 → 0x06 に変更する。オフセット 469 はジャーナルテール（`crates/singlefs-checker/src/lib.rs` の `SYSTEM_CONFIGURATION_TAIL_OFFSET`）に対応する；呼び出しの結果は同じであり、ジャーナルを参照して状態を逐次復元する場合も同様である | ミラーとベースラインが異なる状態：ストリーム 1 は 31 / 0、ストリーム 2 は 78 / 14（すべての永続状態は同一である：後のリリースではそのスロットを正しいテールとして書き込む） | O1（メモリモデルの一致チェック（HashMap の理想的なファイルシステム））：`ObservedRoot` にシステム構成が含まれておらず、復元時にテールを直ちに信頼しない（D23（ジャーナルの役割と形式）で決定された項目 3）；読み戻しは同じである。O2（独立したパーサー + チェッカー）：テールを読み込む（`crates/singlefs-checker/src/lib.rs` の `check_system_configuration_slot`）が、その値を検証する不変条件はない。O3（独立した仕様実行器）：実行器はアンカーの断言のみでテールを読む（`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/executor_main.py` の `anchor_assertions` 内）、完全永続状態のみを核査し、判定には使用しない | **真の盲点候補**：D23（ジャーナルの役割と形式）で決定された項目 14 は「番号取得時の書き込み……その瞬間に同じ方法で読み取った証値 c_証値」（すべての自己証明済みのスロットのジャーナルテールは最大値とする）、「mkfs でテールを 0 に書き込む。mkfs 後の最初の番号取得で 0 を読み取り、0 を書き込む：新しいプールでの新規ファイルのバイトは不変」と記している（`.claude/kb/decisions/23-journal の役割と形式.md`「決定済み項目 14：再プレイの下界は選択されたルートによって与えられる；管理者によるロールバック、失敗処理、およびインスタンス切り替え」）。テールを 1 に書き込んだシステム構成スロットは、規定から導かれるミラーには含まれておらず、すべてのケースが許可される |
| 270（システム構成のローテーション後にバリアを越えずに返却） | リリースの末尾にあるバリアがなくなる：ストリーム 1 は 57 → 53 件の操作、ストリーム 2 は 595 → 565 件となり、後の戻りポイント全体が前方に移動する；書き込まれる内容は同一である。システム構成のローテーションと次の呼び出しの最初の書き込みは同じセグメントに入る | 1 回の呼び出しでのシステム構成書き込みが完了していない状態に対して、その後の呼び出しの書き込みが既に完了している：ストリーム 1 は 7 / 6 の状態、ストリーム 2 は 70 / 59 の状態；ベースラインは 0 | O1（メモリモデルの一致チェック（HashMap の理想的なファイルシステム））-クラッシュ判定官：そのリリースのルートスロット FUA が既に永続化されており、復元時にそれを選択し、緑判定を行う。O1（メモリモデルの一致チェック（HashMap の理想的なファイルシステム））-一致チェック：呼び出しの結果は同一である。O2（独立したパーサー + チェッカー）：システム構成がルートの 1 つ前のリリースで単一のミラー内で合法である（I-7.12（システム構成 F は同じディスク上のルート上の F と同等以上）は F のみを比較する）。O3（独立した仕様実行器）：「在」 | **真の盲点候補**：D16（リリースセマンティクス）で決定された項目 7 は「ルートスロット（FUA）→ システム構成スロット → バリア；fsync などのシステム構成ローテーションが永続化された後にのみ返却する」（第 181 行）、射程「バリア口径」はシステム構成ローテーション後のバリアを「リリース返却前に追加するバリア」と明記している（第 186 行）。返却後でもシステム構成がまだ永続化されていない状態に対して、その後の書き込みが先に永続化する状態は規定から生成できず、すべてのケースが許可される |
| 271 (deleted_inodes のツリー表エントリヘッダー ID に 0 を書き込む) | 2 つのストリームにおける第 44、45 番目の操作（ツリー表ユニット、各ディスク 16 KiB）で、オフセット 1647、1649 の 2 バイト 0x01 → 0x00（ヘッダー ID 65537 → 0）を書き込み、ユニットのチェックサムも変更する。これに伴い、それを指すノード（第 48、49 番目の操作）とルートスロット（第 52 番目の操作）へのポインタおよびチェックサムも変化する。ストリーム 2 の後にツリー表の公開を再書き込む際、すべてが（85 個の操作の違いを含む）変更される。呼び出しの結果は同じであり、回復の状態も逐次同じである | 新しいバージョンを選択する（ルート txg が最初の違いが発生した後に書き出された最初のルート以上の場合）：ストリーム 1 は状態 12 / 0、ストリーム 2 は状態 356 / 112 | O1（メモリモデルの対照（HashMap を理想とするファイルシステム））：ツリー表を読まない。O2（独立したパーサー + チェッカー）：ツリー表エントリはツリー ID、種類、誕生 txg（`crates/singlefs-checker/src/walk.rs` の `tree_table_entry_of_the_walk`）のみを解析し、ヘッダー ID は判定しない。O3（独立した仕様実行器）：投影にはツリー表ユニットのポインタのみを含み、A(c) はその行が書き込んだルートであり、「存在する」 | **真の盲点候補**：D8（コアインデックス構造）は既に決定事項 8 で「ヘッダー ID の 8 バイトの値：……`deleted_inodes` と墓標ツリーに inode ツリーのツリー ID（65537）」を記述している（`.claude/kb/decisions/08-核心索引结构.md`「決定事項 8：ツリー表エントリのフィールドテーブル」）。新しいバージョンを選択した状態ではヘッダー ID は 0 であり、すべてのケースが許可される |
| 277 (プレフィックスを適用して一度公開した後、新しいルートセグメントが変わらない) | スクリプトの書き込みと呼び出しの結果は完全に同一である。ジャーナルによる回復を確認する際、ストリーム 1 の状態 1 / 6 とストリーム 2 の状態 1 / 51（バッチ 2 / 0 の状態）で異なる：ベースラインでは 1 つのレコードを適用して新しいバージョンに到達するが、変異では同じく 1 つのレコードを適用しても選択されたルートのあるバージョンには留まる（例：ストリーム 1 の第 41 状態におけるベースライン `1,3 file_read e8ca4bf8…`、変異 `1,2 no_file`） | 0：状態 57 で丢弃されたバージョンの呼び出しはすべて返っておらず、そのルートスロットも到達していない（`extra_checks_f4.py`：すべて `call_returned=False its_root_slot_landed=False`） | O1（メモリモデルの対照（HashMap を理想とするファイルシステム）-クラッシュ判定官）：前バージョンがモデルによってコミットされ、永続化された最新のルートスロットより古くなく、読み戻したものがそのバージョンと一致する場合、緑の判定は正しい。O2（独立したパーサー + チェッカー）、O3（独立した仕様実行器）：ミラーとベースラインがバイト単位で完全に同一である | **もう一类：3 つのケースが実装による回復におけるレコード再構築に依存する部分で判定されないもの**：呼び出しが返っておらず、新旧両バージョンとも「ルート + ジャーナルの任意のプレフィックス」（D16（公開セマンティクス）決定事項 12）内にあるため、状態自体は合法である。誤りは実装による回復がレコードを適用した後でも新しいルートセグメントが変わらない点であり、D13（検証経路）決定事項 1 の M4 と D23（ジャーナルの役割と形式）決定事項 15（同 27 行目）に違反するため、すべてのケースで判定されない。M4 の到達点は E165（回復再プレイの有界な完全探索と変異殺傷限界）および Verus である。分岐する歴史（推測されたが実行されていないもの）：一度返った公開の後、ルートスロットから読み取れず、回復にはジャーナルの再プレイによってそのバージョンに追いつく必要がある（D16（公開セマンティクス）決定事項 7 の射程「同一インスタンスで 1 つ前の世代へ戻り、ジャーナルの再プレイで追いつける」）。変異は前バージョンで停止する。2 つのストリームにはルートスロットを破損して読んだ状態はない |

- **第 233 行（記録から再構築されたルートは、選択されたルートのファイルシステム識別子をそのままコピーせず全ゼロに書き換えられる）は、シナリオ C のみで解放される**：書込みストリームと呼び出しの結果は完全に同一である。ジャーナル復旧時の読み取りは、記録が適用された状態において失敗する（`InvariantViolated`、`invariant` フィールドは I-1.4（ブロックヘッダーの fsid 整合性）であり、`detail` は「ルート fsid の不一致」）。ストリームバッチ 1：3 状態；ストリームバッチ 2：48 状態（そのうち 6 つの状態がアンマウント中にあり、クラッシュ判定記録は適用されない）。シナリオ A および B では、O1（メモリモデルファジング（HashMap 理想ファイルシステム））- クラッシュ判定者が赤旗を掲げる（ストリーム 1：3 状態；ストリーム 2：42 状態；セグメント正解率 2/12、28/79）。これらは O1 に依存している。シナリオ C では、O1（メモリモデルファジング（HashMap 理想ファイルシステム））- ファジングのみを使用し、呼び出しは復旧を経ず、3 つすべてが解放される。シナリオ C は**真のブラインドスポット候補**である：実装は有効なクラッシュイメージからどのバージョンも復旧できない。D13（検証ルート）は、項目 1 の M2 書き込み再プレイ結果ルートの「他のフィールドは選択されたルートと等しい」（`.claude/kb/decisions/13-verification-route.md` の「項目 1 決定」表における M2 の行）を確定した。O2（独立解析器 + チェッカー）は復旧前のイメージ（ベースラインとバイト単位で同一）を判定し、O3（独立規約実行器）は独自の復旧を実行する。クラッシュ点でのオラクルおよび記録チェッカーはこの行で赤旗を掲げる（`layer0_or_record_checker_red=true`、3 つのいずれでもない）。バッチ 2 は赤旗を掲げない：2 つのストリームバッチ 2 のどちらも「記録が適用された状態」を持たない（`extra_checks_f4.py` の出力 `b2_base_records_applied_gt0: 0`）。**サンプリング点で検出不能**であり、記録が適用された状態をサンプリングポイントはすべて分離する可能性がある；このセルはバッチ間で反転するため、「不安定」と記録される。
- **第 265 行（F を上げる前にシステム構成を書き込まない）は、バッチ 2 の追加分はすべて解放される**：ストリーム 2 のみで操作 440 から差異が生じる（F を上げる際にシステム構成を書き込むステップ全体が削除され、149 つの操作がインデックス順に異なる）。O2（独立解析器 + チェッカー）はバッチ 1 で 12 状態を赤旗とするが、すべて I-7.12（ディスクルート上の F と同様のシステム構成 F が低くない）であり、セグメント 59 および 71 にそれぞれ 6 つの状態がある（`name=trajectory row=265 stream=fixed_script_through_e_then_unmount oracle=O2（独立解析器 + checker） total_red_states=12 peak=6 positive_segments=2 of_segments=77 …`）。バッチ 2 はストリーム 2 でセグメント 7、11、16、20、24、28、32、36、40、44、48、52、56、60、64、68、72 を選択しており、セグメント 59 および 71 は含まれていない（`extra_checks_f4.py` の出力 `batch2_states_in_segments_59_71=0`）。**サンプリング点で検出不能**であり、第 102 行と同様；「不安定」と記録される。条項は D16（公開セマンティクス）の項目 7 で確定済み：「F を上げる際……まず新しい F を各ディスクのシステム構成に書き込み、その後バリアを通過する」。Q1 および Q3 の結論文字列はバッチ 2 では反転しない（バッチ 2 は全 15 行を解放；単一ストリームに依存する行も依然として ≥ 1）。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:185-201`（整段抄，未转述）**

```markdown
### 三輪判決後の三分類

三轮結論判定（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`；第三輪は最終輪であり、その第 3 節「三輪後の結論」と第 5 節を基準とする）でこの一連の読み値を再度三分類する。口径：**真ブラインドスポット候補**＝このバッチで取得した状態が既定義項の下では不合法であり、三条すべてが許可される（E171（3 つのオラクル共謀テスト）s4 ツリー）；**その他のカテゴリ**＝三条は設計通りに判定しないか、あるいは誤りがあるとしてもそれは状態に起因しないもの。執行員が原判で「このバッチは等価である」とした五行（第 15、27、34、112、277 行）の一行ももはや等価ではない。

| クラス | 行 | 一言の根拠（各行の条項原文は「F4 の拡張表の結果（行全体を産物からコピー）」と「F4 で新たに全通過された行」のセクション内の行にあります） | サイクルごとの翻弄 |
|---|---|---|---|
| 真の盲点候補 | 第 15 行 | D3（空間割り当て）既定項目 10⑤ の t2–t12 着地点：変異下で 11 つのコミット内生ブロック全体が 64 スロット分移動（`row15-offsets.out`）、⑤ と `layout/01-first-txn.md`内の「コミット内生ブロック」行に釘付けされているのはこれらの着地点です | サイクル 1 では「このバッチの同等性」から移入されました（D3（空間割り当て）既定項目 10③）；サイクル 2 では維持され、根拠は⑤ の着地点表と `layout/01-first-txn.md`内の「ユーザーデータ着地点ポリシー」行の t1@50180 に基づきます；サイクル 3 ではリストに残り、根拠は⑤ の t2–t12 着地点に変更されました（メインエージェントが現在⑤表の t2@50240 を照会し、第 220 行「改新プールで新規ファイルのバイト数：はい」を再実行した `row15-offsets.out` と同様に確認可能） |
| 真の盲点候補 | 第 23、42、271 行 | 第 23 行 D8（コアインデックス構造）既定項目 6 のコンテナ番号；第 42 行 D16（公開セマンティクス）既定項目 1 のアンロード上げ F；第 271 行 D8（コアインデックス構造）既定項目 8 のヘッダー ID | サイクル全体でリストに残っています |
| 真の盲点候補 | 第 242、269、270 行 | 第 242、270 行 D16（公開セマンティクス）既定項目 7 の書き込み順序；第 269 行 D23（ジャーナルの役割と形式）既定項目 14 の証言値 | サイクル 1 でリストから除外されました（既定項目 12 が合法であり、書き込み順序義務を誰も判定しないため）；サイクル 2 でリストに戻り、サイクル 3 では命中されませんでした |
| その他 | 第 27、112、277 行 | 3 つの条項は実装回復における記録再構築に依存する部分（D13（検証経路）既定項目 1 の M4、D23（ジャーナルの役割と形式）既定項目 14 の注釈 1 と既定項目 15）を判定しません；第 112 行は `.claude/gate.d/e165-pinned-red-examples.tsv` にあります | サイクル 1 で正推法によりこのカテゴリに分類され、サイクル 2 では維持されましたが措辞が狭められ、サイクル 3 では変化がありません |
| その他 | 第 34 行 | 割り当てポリシー、ミラー合法：D3（空間割り当て）既定項目 8 の射程 | サイクル 1 では第 15 行と共に真の盲点候補に移動し、サイクル 2 でリストに戻り、サイクル 3 では維持されました |
| その他 | 第 92 行 | D8（コアインデックス構造）既定項目 6 に書き込み時間値（2026-10-04）が追加済み；3 つのオラクルは設計通りに判定できず、釘付けされたテストケースによって判定されます | サイクル 1 では「条項未定義かつ非可視」によりリストに入らず、サイクル 2 で理由を変更し、サイクル 3 では維持されました |
| その他（ゼロサイクル） | 第 110、162、174 行 | ランタイム読み取り値（書き込み帳簿、ブランチ名）、これら 3 つは設計通りに判定されず、第 34 行と同カテゴリです | サイクル 3 で新たに浮上：攻撃側が戻り値を量る際に発見されました（「触達していないのに赤い 2 行と未触達行の中で戻り値が異なる 7 行」セクション参照） |
| その他（ゼロサイクル） | 第 49、252 行 | 戻り値またはメモリ状態の誤り、書き込みストリームは不変であり、より長い履歴が必要で区別できます | サイクル 3 で新たに浮上：攻撃側が戻り値を量る際に発見されました（「触達していないのに赤い 2 行と未触達行の中で戻り値が異なる 7 行」セクション参照） |

第 155 行および第 161 行は既知の 2 つの行（O1（メモリモデル対照（HashMap 理想ファイルシステム））- 対照赤判定）であり、これらのカテゴリには含まれていない。第 233 行（丙で全通過、甲乙で O1（メモリモデル対照（HashMap 理想ファイルシステム））- 崩壊判官が捉えた）、第 265 行（バッチ 2 サンプル点不敏感）の読み方は「F4 新出の全通過行」節にあり、第 102 行（バッチ 2 サンプル点不敏感）は「3 つの全通過の 7 行」節の末尾にある。3 回とも変更されていない。「もう一类（ゼロ輪）」の 2 行の意味は、この類が第 3 回で初めて現れ、さらに輪を開かず、不足分を記録し、登録された触達定義を変更しないことである。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:202-222`（整段抄，未转述）**

```markdown
### F4 新規の単一依存による 12 行

到達不能なオブジェクトと判定されていないフィールドのみを記述し、結論は判定しない。読み取り値は F4 のエクスポートディレクトリ（ドラフトスクリプト `caught_rows_f4.py`、セグメント 7 の `caught_rows_s4.py` を変更してルートから出力し、データベースへの登録を行わない）と生成物の `name=trajectory`、`name=row_matrix` 行を取得する。セグメントは正の数は「赤判定のセグメント数 / このストリームのセグメント数」である。

| 行（変異） | どの行が赤いのか | なぜ他の二つは見えないのか |
|---|---|---|
| 172（提出されたディスク数 w の下限も拒否する） | O1（メモリモデルの対照（HashMap 理想ファイルシステム））- 対照：ストリーム 2 の第 4 回目の `mount_writable` 呼び出しで `err:WritableDeviceCountBelowTheStripeWidthLowerBound` を返却し、モデルはこれを正しく判定するが、ストリーム 2 は第 92 番目の操作で停止している | 拒否される前の書き込みストリームとベースラインは逐条一致しており（0 件の操作の違い）、取得されたミラーもすべてベースラインのバッチの前缀である。拒否された呼び出しにはミラーが残らないため、O2（独立解析器 + checker）、O3（独立仕様実行器） は判定する対象となるオブジェクトを持たない（同第 45 行参照） |
| 233（記録から再構築されたルートは選択されたルートの fsid をそのままコピーしない） | O1（メモリモデルの対照（HashMap 理想ファイルシステム））- クラッシュ判定官：バッチ 1 ストリーム 1 の 3 状態、ストリーム 2 の 42 状態で読み込み失敗（I-1.4（ブロックヘッダーの fsid 一致） ルート fsid が不一致）、セグメントは正 2/12、28/79；バッチ 2 は 0 状態 | 実装側の回復に誤りがある。書き込みストリームとベースラインは逐バイト一致している。O2（独立解析器 + checker） は回復前のミラーを判定し、O3（独立仕様実行器） は独自の回復を実行する。バッチ 1 とバッチ 2 の赤い列が異なる：バッチ 2 では記録が適用された状態が存在せず、サンプリングポイントに感度がない（「F4 で新設された全通過」セクション参照） |
| 237（システム構成スロットを解読する際、ルートリングの起点を読み取る位置が誤っている） | O1（メモリモデルの対照（HashMap 理想ファイルシステム））- クラッシュ判定官：バッチ 1 ストリーム 1 の 53 状態、ストリーム 2 の 73 状態（セグメントは正 12/12、16/16）、バッチ 2 は 7、14 状態で、「ルートを選択しなかった」（`SystemConfigurationValueRefused … RootRingBaseNotTheFirstVersionSlot`）；O1（メモリモデルの対照（HashMap 理想ファイルシステム））- 対照：ストリーム 2 の第 4 回目の `mount_writable` 呼び出しで `err:Recovery` を返却 | 実装側のリーダー側に誤りがある。書き出されたバイトはベースラインと一致しており（0 件の操作の違い）、O2（独立解析器 + checker）、O3（独立仕様実行器） はそれぞれ独自の解析器を用いてシステム構成を読み取り、観察されるミラーはベースラインと逐バイト一致している |
| 249（データユニットの公開も二つの記録に分割される） | O1（メモリモデルの対照（HashMap 理想ファイルシステム））- 対照：二つのストリームの第 2 回目の `publish` 呼び出しで `disagrees:jsn` と判定する（モデルは当該バージョンの jsn が 3 であると判断し、実装が 4 であるため）、モデルは停止する | 二つの記録を合わせると依然として一回の完全な公開となる：O2（独立解析器 + checker） はこの行で 0 状態に赤くなる（`crates/singlefs-checker/invariants/` に「一度の公開が何件の記録を含むか」の制限があるかどうかを逐条確認していないため）；O3（独立仕様実行器） は記録を独自に再プレイし、到達するバージョンは A(c) 内にあり、jsn を含まない投影となる |
| 250（データユニットのペイロードバイトが逆順で書き込まれる） | O1（メモリモデルの対照（HashMap 理想ファイルシステム））- 対照：二つのストリームの第 2 回目の `publish` 呼び出しで `disagrees:このバージョンのファイル内容` と判定する（読み戻し 3000 バイト、内容は逆順） | ユニットのチェックサムは書き込まれたバイトに基づいて計算され、構造は自己整合している：D13（検証ルート） は既に項目 6 の表を決定しており、O2（独立解析器 + checker） の行「検証できない」欄には「ブロックが別の合法な内容に上書きされた」と記載されている；O3（独立仕様実行器） の投影はファイル内容を含まず、書き込まれた内容と比較を行わない |
| 281（ルートの選択で自己証明済みの最初のスロットを選択するとそれ以降は変更しない） | O1（メモリモデルの対照（HashMap 理想ファイルシステム））- クラッシュ判定官：バッチ 1 ストリーム 1 の 41 状態、ストリーム 2 の 61 状態（セグメントは正 9/12、13/16）、バッチ 2 は 7、14 状態で、「インスタンス 0 の第 0 世代のルートに到達」（ディスク上に永続化されている最新のルートスロットよりも古い）；O1（メモリモデルの対照（HashMap 理想ファイルシステム））- 対照：ストリーム 2 の第 4 回目の `mount_writable` 呼び出しで `err:NewerStateStillUnreadableAfterOneReread` を返却 | 実装側のルート選択に誤りがある。書き込みストリームとベースラインは逐バイト一致しており（0 件の操作の違い）、O2（独立解析器 + checker） はミラーの構造を判定し、O3（独立仕様実行器） は独自のルート選択規則を用いて回復を行い、観察されるミラーはベースラインと同じである |
| 135 (defer のスロット数が defer を復活させる際にも減らない) | O2 (独立解析器 + checker): I-3.11 (割り当て済みから defer を引いた値が最新のルート読み取りに等しい) バッチ 1 フロー 2 225 状態、セグメントは正 44/79; バッチ 2 70 状態 | O1 (メモリモデルの対照 (HashMap 理想ファイルシステム)): `ObservedRoot` (`crates/singlefs-harness/src/model.rs` におけるその定義) に記帳行がない; O3 (独立仕様実行器): プロジェクションは記帳木に含まれない |
| 234 (ルートレコードの書き手はアンマウントフラグに従って flags を書かない) | O2 (独立解析器 + checker): I-7.9 (ロールバック下界 F は上げられた F の上限以下である) (アンマウントフラグごとに 2 分岐) バッチ 1 フロー 2 29 状態、第 73 セグメントから正 6/79; バッチ 2 7 状態 | O1 (メモリモデルの対照 (HashMap 理想ファイルシステム)): 書き込みはアンマウント呼び出しに落ちるが、モデルにはアンマウントの答え (登録読み取り法 6) が存在しないため、対照は行われず、クラッシュ判定官記も適用されず、`ObservedRoot` もルートの flags を持たない; O3 (独立仕様実行器): プロジェクションはアンマウントフラグを持たない (`root_ring.py` 内の `VersionProjection` の説明:「アンマウントフラグとルートレコードの他のフィールドはプロジェクションに含まれない」) |
| 238 (変更カウントに 1 を残す) | O2 (独立解析器 + checker): I-9.26 (inode の変更カウントは出生世代とコンテナ誕生世代番号の間にある) バッチ 1 フロー 2 333 状態、セグメントは正 65/79; バッチ 2 105 状態 | O1 (メモリモデルの対照 (HashMap 理想ファイルシステム)): inode レコードの変更カウントを比較しない; O3 (独立仕様実行器): プロジェクションは inode ツリーに含まれない |
| 245 (オーバーライト書き込みの逆リンクは前バージョンレコード自身の逆リンクを取得する) | O2 (独立解析器 + checker): I-8.6 (逆リンクアルゴリズム) (逆リンクが一致しないレコードは再プレイされない) バッチ 1 フロー 2 336 状態、セグメントは正 67/79; バッチ 2 105 状態 | O1 (メモリモデルの対照 (HashMap 理想ファイルシステム)): `ObservedRoot` はジャーナルカウンターのみを持ち、逆リンクを持たない; O3 (独立仕様実行器): その前規則核は逆リンクを使用する (`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/replay_prefix.py` の `replay_prefix` で不整合の場合停止), この行は厳密かつ一時的な置換も「存在」と判定される; 停止後に到達したバージョンが A(c) にあることは推論される (ルートスロットに落ちているため、再プレイに依存しない), ただしどこで停止したかは逐状態チェックしていない |
| 258 (古い配置位置を解放しない) | O2 (独立解析器 + checker): I-3.1 (割り当て済み統計が一致する), I-3.11 (割り当て済みから defer を引いた値が最新のルート読み取りに等しい) バッチ 1 フロー 2 333 状態、セグメントは正 65/79; バッチ 2 105 状態 | O1 (メモリモデルの対照 (HashMap 理想ファイルシステム)): `ObservedUnitAllocationRecords` (`model.rs` 内の `ObservedUnitAllocationRecords` の定義) はこのバージョンの各役割のユニットの割り当て記録のみを持ち、古い配置位置が解放されたかどうかは含まれていない; O3 (独立仕様実行器): プロジェクションは割り当て記録木と記帳木に含まれない |
| 265 (F を上げる一連の操作はシステム構成を先に書かない) | O2 (独立解析器 + checker): I-7.12 (システム構成 F は同じディスク上のルート上の F と等しいかそれ以上である) バッチ 1 フロー 2 12 状態 (第 59 セグメントと第 71 セグメントそれぞれ 6 状態); バッチ 2 0 状態 | O1 (メモリモデルの対照 (HashMap 理想ファイルシステム)): ルートに付随する F を比較し、システム構成スロットを比較しない; O3 (独立仕様実行器): プロジェクションの F はルートから取得され、A(c) はこの行が自分で書いたルートである。バッチ 1 とバッチ 2 の赤判定列は異なる: バッチ 2 は第 59 セグメントと第 71 セグメントを採樣しなかったため、サンプリングポイントに不敏感である (「F4 新しい全通過行」の節参照) |

O3 (独立仕様実行器) は、F4 の分母に含める 37 行すべてにおいて赤判定が一つもありません (`name=q1_column_red_rows … O3（独立仕様実行器）甲=0 O3（独立仕様実行器）乙=0`)。各行の 2 つのストリームのバッチ 1 状態に対して厳密な判定が与えられています (`name=q1_o3_coverage … rows_with_no_strict_judgement=0`)。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:223-237`（整段抄，未转述）**

````markdown
### 触達していないにもかかわらず赤色に判定された 2 行（第 155 行および第 161 行）と、触達されていない行の中で戻り値が異なる 7 行

`name=untouched_but_red rows=2 row_numbers=155,161`: 両行のバリエーションは同一箇所である (`crates/singlefs-core/src/mount.rs` にて F の上限を上げる処理 `newest_on_every_device.min(fourth_newest)` を `.max(…)` に変更し、両行の原文では行頭の空白のみが異なる), 3 つの要素すべてが異なる (`name=row_touch row=155 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false … touched=false`), O1（メモリモデルの対照 (HashMap の理想的なファイルシステム)）- 対照はストリーム 2 の第 12 回呼び出しで `raise_floor` を実行し `disagrees: F の上限を上げる` と判定する (`call-outcomes.tsv`: モデル「上限 Some(11)（D16 (リリースセマンティクス) で確定した項目 1)」, 実装「実装側が上限 13 を報告」）, バッチ 1、バッチ 2 の両方が赤判定; O2（独立したパーサー + チェッカー）、O3（独立した仕様実行器）は赤判定ではない。

- 装置、比較器またはサンプリングの欠陥ではない：比較器の到達は登録セクション1の読み方4の文言計算（「任意の呼び出しの結果（成功／どのようなエラー）」、`collusion_matrix.py` の `outcome_words` は単に呼び出し名と結果列のみ）であり、今回の呼び出しでは両側とも `ok` である；O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照は成功した呼び出しが返す値を比較する（ここでは F の上限を上げる）ため、書き込みフローが変わらないのは、このリクエストにおける F が 11 を超えていないからであり、上限の緩和は効果を発揮しなかった。O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照は呼び出しごとに計算されるためサンプリング点とは無関係であり、両方のバッチとも赤判定である。
- 欠陥は登録読み方4の到達定義にある：それは「成功した呼び出しが返す値」を3つの項目の外に置いているにもかかわらず、「3つの項目すべてが同一な行……このバッチでは正常実装とバイト単位で同様の動作を示し、どのオラクルも赤判定すべきではない」と記している；しかし、これらの2行の戻り値は異なり、その前提条件は成立しない。装置の `call-outcomes.tsv` は対照が不一致を検出した場合にのみ両側の値を記録するため、成功した呼び出しが何をもたらすかは個別に記録されておらず、登録済み装置にはこの列がない。到達定義は変更されていない（生成物を走査しても登録内容は変わらないため）、分母は依然として 37 である。
- 他の未到達行にも同様の差異があるか：攻撃側（第 3 ラウンド）は s4 ツリーの複製上で測定を行った（複製プローブ、s4 ツリー、登録済み装置にはこの列がない）。プローブはフロー 2 の駆動に従ってメモリスパースディスク上で 1 回実行され、各成功した呼び出しが返す値の `{:?}` テキストサマリー、最終的なアロケータおよび現在のバージョンを記録する；ベースラインと E171（3 つのオラクル共謀テスト）から導出された 595 の操作はすべて逐条一致している；次に未到達行を 1 行ずつ複製に書き込み、再編成して再実行し、復元を行った。232 行を実行した（`make_filesystem.rs` の 6 行を除いた 238 行であり、mkfs は単一の呼び出しではないためプローブはその戻り値を記録せず、149–154 行は実行されなかった）、7 行が異なる：第 49、110、155、161、162、174、252 行である。第 155、161 行とは前述の 2 行であり（この基準は同じ第 12 回目の呼び出し `raise_floor` でこれらを捉えたものであり、上限 11 → 13 となり、既存の陽性対照となる）；残りの 5 行 O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照はそれらのフィールドを比較しないため赤判定とならない：第 49 行は書き込みマウントが返す証言値 4 → 3、第 110 行は書き込み帳簿の構造種別、第 162 行はシャドウ帳簿のブランチ名、第 174 行は F を上げるために返される書き込み帳簿、第 252 行はメモリ内のアロケータにルートスロットの登録が欠けている（第 49、252 行では戻り値またはアロケータが異なり、書き込みフローは不変）；要約行は攻撃側の `return-rows.out` の `name=return_summary` という行をそのままコピーしたものである：

```
name=return_summary hidden_difference=7 identical=225 no_output=6
```

主担当エージェント の再実行 (`tools/rerun.sh … all`) は、この 7 行が文字通り同一である；`return-rows.out` では、変異表の第 28–31 行にある 4 つの `name=return_row`（再実行では `status=no_output`、集計は `hidden_difference=7 identical=221 no_output=10`、攻撃側では `identical`）のみが異なる：攻撃側は `make_filesystem.rs` の行を削除する前にこれら 4 行を実行済みであり、`rerun.sh` の `awk` がそれらをフィルタリングするため、再実行では再度実行されない（`research/prompts/e171-conclusion-r3-main-verification.md` の第 4 節参照）。害悪の規模は計測できない：第 49、252 行が推論しているのは、より長い履歴が必要であることによる分離可能性の問題である。
- その他の読み取りへの影響：両行は分母にも、Q1、Q3 のリストにも含まれない。「触達」基準でこれらをカウントした場合、D = 39 となり、両行とも O1（メモリモデルのチェック（HashMap 理想ファイルシステム））が赤判定（赤判定）、Q3 は O1（メモリモデルのチェック（HashMap 理想ファイルシステム））による行が 2 行多い（第 155、161 行）ため、Q1 の 3 つの読み取り方式における全通過行は不変となる。7 行のうち値が異なる行と第 125 行（「回答できない」）をすべて含めると、D は 37 から 45 に変わる（推論、算術、登録済み装置上で実行されていない）。その 5 行の O1（メモリモデルのチェック（HashMap 理想ファイルシステム））は赤判定とならず、新たな全通過行となる。

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:238-265`（整段抄，未转述）**

````markdown
### 修正できない 6 行の代替実行（実行済み）

第 1 節の登録読み方 5「置換後に編成が通らない（`.claude/rules/mutation-sampling.md` の第 8 類、5.2 の代替実行規則に従って改行）」；s4 と F4 のバージョンのドライバは、編成前の失敗のみで代替実行を行っており、33、59、127、163、206、243 行において編成が通らない後には元のまま「編成が通らない」と記録し、代替実行を行っていない。第 8 段落ではドライバを改め、編成が通らない後も代替実行を行うようにした（同一ファイルグループ内のインデックス j + 1 の行に置き換え、両方の表にも存在せず、今回の実行で他の代替実行も使用されていない行；置換された行は編成前に照样にアンカーポイントを核査し、編成が通らない場合は再度置き換え、編成前の失敗と編成が通らないを合わせて一行あたり最大 3 回；グループには E171（三 oracle の共謀テスト）装置自身の変異行が含まれない）、`--rerun-compile-failed-from` を追加（出力の根の一部のみで結果として「編成が通らない」となる行のみを実行）。コンパレーターに `--substitution-rows-root` を追加（代替実行を行った回の行で、元々「編成が通らない」となった同号行を置き換え、他の状況では拒否）、新たに `name=substitution` と `name=substitution_summary` の 2 つの結果行が増加した。

- 代替実行走査は、s4 の凍結ツリー上で実行された（スナップショットのサマリーは s4 の 5 回分と同じであり、空変異行は `baseline-run1` に到達しなかった）。出力ルート内のコンパイル失敗した行のみを実行した（`rerun_compile_failed rows=33,59,127,163,206,243`）。2 つのレーン、各レーン 2 スレッド。すべてのステップで 0 を返した。代替実行は 6 行を完了し、経過時間は 1,277.9 秒であった。コンパレーターは 117 + 164 + 代替実行を `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` にマージした（4,795 行、最終行 `E7RESULT name=done emitted=4795`）。時間情報は `…-s4-f4-substitutes-rows-timing.out` にある（292 行）。
- 代替実行の 6 行（すべて `…-s4-f4-substitutes-comparator.out` の 6 つの `name=substitution` エントリからコピー）、代替実行サマリー（`name=substitution_summary`）および分母（`name=q1_denominator rows_run=`）：

```
E7RESULT name=substitution row=33 original_name=步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 original_status=compile_failed substituted_by=步_4：影子账把被抛弃根账里已释放的落点也隔离 failed_attempts=1 attempts=compile_failed:步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 category=untouched
E7RESULT name=substitution row=59 original_name=C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） original_status=compile_failed substituted_by=C483_②：挂载态打开时_inode_叶容器只按位置提示读（不经映射回退） failed_attempts=1 attempts=compile_failed:C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） category=untouched
E7RESULT name=substitution row=127 original_name=增补_2：回落只看已分配位（发出影子账隔离的槽） original_status=compile_failed substituted_by=增补_2_第_20c_行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放） failed_attempts=2 attempts=compile_failed:增补_2：回落只看已分配位（发出影子账隔离的槽）|compile_failed:增补_2：回落只看已分配位（抬_F_的空发布发出扣住的槽） category=untouched
E7RESULT name=substitution row=163 original_name=C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） original_status=compile_failed substituted_by=D8_已定项_8_②：可写挂载写行那次的树_ID_水位只沿所选那一版带，不取根环与环里记录的_max（原为_C511_第_3_步回退到暖机根那一格；回退改成向前发布之后由崩溃恢复丢掉一次第一个文件版本那一形护） failed_attempts=1 attempts=compile_failed:C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） category=untouched
E7RESULT name=substitution row=206 original_name=步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） original_status=compile_failed substituted_by=增补_3_第_4_件：一块盘两个系统配置槽都无效就整体_return_Err（每盘一份的冗余当场作废，D22_已定项_8_第_1_条） failed_attempts=1 attempts=compile_failed:步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） category=untouched
E7RESULT name=substitution row=243 original_name=并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 original_status=compile_failed substituted_by=普查_R2：释放判定路径把「映射条目切不动」报成「不在映射」（两件要分流的事并成一个成员） failed_attempts=1 attempts=compile_failed:并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 category=untouched
E7RESULT name=substitution_summary rows_that_did_not_compile=6 substituted_rows=6 still_not_compiled=0 row_numbers=33,59,127,163,206,243
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=238 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
```

- 六行が `category=untouched` に置換された（第 127 行前の置換も行われず、二度目の置換となった）；分母 D = 37 は変化しなかった：`compile_failed` 6 → 0、`untouched` 232 → 238、`name=f6_table_failures` の `rows_not_counted_for_the_table` 12 → 6（六行のみが自己崩壊を実現）。代替実行生成物よりも代替実行前の F4 生成物に追加された行は、この六行の `name=row`、`row_touch`、`row_matrix`、`row_s5`、`row_o3` および `trajectory` に加え、`name=substitution` の六行と `name=substitution_summary` であり、変更された行は `name=config`、`name=q1_denominator`、`name=f6_table_failures` および `name=done` である。
- Q1、Q3 のリストおよび代替実行前のものと文字通り同一（2 つの生成物を自分で diff で確認）：

```
cd research/results; A=e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out; B=e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out; P='name=(q1_alpha|q1_beta|q1_gamma|q1_quiet_row|q1_column_red_rows|q3_alpha|q3_beta|untouched_but_red|q1_denominator_by_table|q1_delta|q1_o3_coverage) '; diff <(grep -E "$P" $A) <(grep -E "$P" $B); echo "diff_rc=$?"; grep -cE "$P" $A; grep -cE "$P" $B
```

`diff_rc=0`、`48`、`48`；`name=q2_`、`name=f3_`、`name=verdict`、`name=f4_expansion`、`name=q1_denominator_by_table`、`name=positive_control`、`name=freeze_check`、`name=q3_core_row` で始まる行も 2 つの出力それぞれについて行ごとに同（同じコマンドで `P` を `'name=(q2_|f3_|verdict|f4_expansion|q1_denominator_by_table|positive_control|freeze_check|q3_core_row)'` に置き換えて、`diff_rc=0`、`171`、`171` を出力）。
- 修正した比較器（代替実行根を与えない）で s4 と F4 の入力を再実行し、E7RESULT 行と 2 つの生成物がそれぞれバイト単位で同一であることを確認する；手製の代替実行根（1 行目を第 34 行の出力に置き換え、もう 1 行目は依然として編成不能）を空走させ、分母とリストは変更された行に基づいて計算し、2 つの新しい結果行はそのまま記述する（ドラフトであり、リポジトリにはコミットしない）。

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:266-294`（整段抄，未转述）**

````markdown
### Q2 O2（独立パースャ＋チェッカー）は通過させながら、O3（独立仕様実行器）は悪い画像を赤でフラグする（アサートから完全な行コピー）

`…-s4-f4-substitutes-comparator.out` の第 4759、4760 行（代替実行前の `…-s4-f4-comparator.out` の第 4662、4663 行と同様の 2 行であり、`sed -n '4662,4663p'` と `sed -n '4759,4760p'` をそれぞれ `md5sum` に与えた結果はどちらも `07b56b24014dac5d9eb543ef15384aa9`）：

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o3_strict_red=22 o3_provisional_red=22 o2_only_strict=0 o3_only_strict=12 both_red_strict=10 o2_only_provisional=0 o3_only_provisional=12 both_red_provisional=10 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=B rows=40 o2_red=23 o2_quiet=17 o3_strict_red=40 o3_provisional_red=40 o2_only_strict=0 o3_only_strict=17 both_red_strict=23 o2_only_provisional=0 o3_only_provisional=17 both_red_provisional=23 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:18,new:19,new:20,new:23,new:26,new:28,new:30,new:32,new:33,new:35,fix:21,fix:22,fix:23,fix:24,fix:25,fix:32,fix:35
```

12 + 17 = 29 枚は O2（独立解析器 + checker）が通過し、O3（独立仕様実行器）は厳密に「含まれない」と判定する。`name=q2_row` から行ごとに数同一バッチを、パッチの種類ごとに分ける（コマンドと元の出力；`B` は前述の産物）：

```
grep 'name=q2_row family=[PB] ' B | grep ' o2=green ' | grep -c 'o3_strict=not_in'
29
for f in P B; do echo "== $f"; grep "name=q2_row family=$f " B | grep ' o2=green ' | grep 'o3_strict=not_in' | sed -E 's/.*bad_name=([^ ]*) .*/\1/' | sed -E 's/^B[0-9]+:(P[0-9]+):.*/\1/; s/^(P[0-9]+)-.*/\1/' | sort | uniq -c; done
== P
      1 P2
      4 P3
      4 P4
      1 P5
      2 P9
== B
      4 P2
     12 P3
      1 P5
```

P 族の 12 枚は、5 つのパッチ（P2、P3、P4、P5、P9）のインスタンスであり、ブラインドテスト B 族の 17 枚は、3 型（P2、P3、P5）のランダムなインスタンスである。これらはすべて構造的自己整合性を有する偽造ルートおよび偽続接ミラーである。

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:295-397`（整段抄，未转述）**

````markdown
### 第1段階のベースラインと対照（全行は s1 の生成物からコピー。作業領域から移動された行は、1 行ポインタとして記述）

`research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out`（ベースライン第一回；二回の E7RESULT 行がバイト単位で同一であり、`diff` は出力しない）：

```
E7RESULT name=stream stream=new_pool_file_creation ending=completed exported=true not_exported_why=- operations=57 writes=41 segments=11 tearable_writes=8 calls=3 full_states=16777260 quick_tier_states=46 sampled_states=82 model_stopped_at_call=none
E7RESULT name=batch stream=new_pool_file_creation batch=1 states=53 segments_touched=12 states_with_a_torn_write=20 o1_crash_judge_red=0 o1_crash_judge_green=53 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=new_pool_file_creation batch=2 states=7 segments_touched=1 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=7 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=stream stream=fixed_script_through_e_then_unmount ending=completed exported=true not_exported_why=- operations=595 writes=477 segments=78 tearable_writes=46 calls=15 full_states=1662648564 quick_tier_states=278 sampled_states=890 model_stopped_at_call=14
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=1 states=397 segments_touched=79 states_with_a_torn_write=115 o1_crash_judge_red=0 o1_crash_judge_green=349 o1_crash_judge_not_applicable=48 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=2 states=119 segments_touched=17 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=105 o1_crash_judge_not_applicable=14 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
```

`e171-three-oracle-collusion-2026-10-03-s1-comparator.out`、ベースラインと対照（Z1 2 回パスで 26 ファイルがバイト単位で同一、S2 と E170（O3 仕様実行器第 1 版）のエクスポート装置は 18 項目がバイト単位で同一、S5 の 2 つの数値比較器は各自で計算、a3 ベースラインは赤判定が 0）：

```
E7RESULT name=a4 table_rows=117 expected=117 per_file_matches_the_registration=true found_once_in_today_table=117 line_unchanged_since_the_registration=117
E7RESULT name=a5 first_slice_rows=24 expected=24 distinct_files=24
E7RESULT name=a6 step_8_rows=117 step_4_rows=220 expected_step_4=220 overlap=56 expected_overlap=56 step_4_only=164 expected_step_4_only=164 step_8_names_equal_the_registered_table=true
E7RESULT name=a7 implementation_rows=844 expected=844 files=24 expected_files=24
E7RESULT name=a3_baseline_reds o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
E7RESULT name=s5_batch_1_states stream=new_pool_file_creation recomputed=53 device=53 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=1 recomputed=53 device=53 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=2 recomputed=7 device=7 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=3 recomputed=22 device=22 equal=true
E7RESULT name=s5_batch_1_states stream=fixed_script_through_e_then_unmount recomputed=397 device=397 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=1 recomputed=349 device=349 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=2 recomputed=105 device=105 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=3 recomputed=330 device=330 equal=true
E7RESULT name=z1_summary files_compared=26 files_differing=0
E7RESULT name=s2_summary files_compared=18 files_differing=0
```

陽性対照（登録 5.6、各オラクルはベースラインと空変異それぞれ 1 回ずつ）および 1 つのゲートのオン/オフ自己検査：

```
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=q3_core closed=O1 run=disable-O1 differences_outside_the_closed_column=0 where=none
E7RESULT name=q3_core closed=O2 run=disable-O2 differences_outside_the_closed_column=0 where=none
```

三行試走のカテゴリ、触れ合いの三つと Q1 の分母：

```
E7RESULT name=row row=0 row_name=空变异（C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式）） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=0 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=1 row_name=C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=1 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=6 row_name=K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出） category=self_crash substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=6 write_stream_differs=true call_outcomes_differ=true recovery_report_differs=false ignored_pass_differs=false touched=true
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 102 行（name=q1_denominator）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 103 行（name=f6_table_failures）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 104 行（name=q1_o1_and_o2_both_quiet）
E7RESULT name=q1_o1_and_o2_both_quiet_batch_2 rows=0 rows_that_differ_from_batch_1=none
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 106 行（name=q1_column_red_rows）
E7RESULT name=untouched_but_red rows=0 row_numbers=none
```

Q2 の O2（独立パースャー＋チェッカー）1 列（O3（独立仕様実行器）2 列、ブラインドテストファミリーは実行されていない）：

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 191 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 192 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 193 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 194 行（name=q2_blind_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 195 行（name=q2_negative_bases）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 196 行（name=a9_a10_a8）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 198 行（name=q2_all_quiet）
```

判決行（比較器自身の行；O3（独立仕様実行器）はすべて `not_run`）：

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 199 行（name=verdict）
```

`e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out`（第一刀第 6 行：分配記録ツリーがスロット番号で葉を探す際に葉の幅を 2 倍に計算し、最初の `publish` でパニックを実装する。装置は「自己崩壊」を記録し、このストリームはエクスポートされない）：

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 2 行（name=stream）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 5 行（name=call）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 14 行（name=stream）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 17 行（name=call）
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out`（41 件の既知の不良イメージがストリーム 1 の全永続状態において：パッチを重ね戻すことと直接改悪して赤判定となる不変量集合が等しく、目標不変量はすべて赤であり、エントリ名と元ファイルは逐条一致している）：

```
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_build_pool local="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" crates="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" holds=true
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_the_full_state_of_the_dump local=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") crates=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") holds=true
E7RESULT name=s4 check=known_bad_test_file_sha256_is_the_copied_one local="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" crates="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" holds=true
E7RESULT name=s4 check=known_bad_entry_count_here local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_count_in_the_original_file local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_names_equal_the_original_file_in_order local=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] crates=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] holds=true
E7RESULT name=verdict known_bad_entries_failing_s4=0 e170_bad_rows_failing_s3_or_s4=0
```

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:398-404`（整段抄，未转述）**

```markdown
### 対照と自己点検

- 陰性（登録 5.6）：ベースラインの 2 つのストリーム、3 バッチの各状態および各呼び出しにおいて、O1（メモリモデルのチェック（HashMap 理想ファイルシステム））- チェック、O1（メモリモデルのチェック（HashMap 理想ファイルシステム））- クラッシュ判官、O2（独立解析器 + checker） 判紅はすべて 0 箇所（a3、`name=a3_baseline_reds`）；空変異（表中の最初の行を取得し、置換文を原文そのものに置き換え、完全な行ごとに駆動を実行）は未到達と判定され、4 つの列が赤ではない（`name=row row=0`、`name=row_touch row=0`、`name=row_matrix row=0`）；パッチなしの 8 つの基底状態（2 つのストリームの F、Mk、Mk⁺、Mk⁻）O2（独立解析器 + checker） は赤ではない（`name=q2_negative_bases`）。O3（独立仕様実行器）、第2段階以降：固定したツリーの仕様実行器は、ベースラインの 2 つのストリーム・バッチ 1、バッチ 2 の 576 状態において判紅が 0 箇所であり、s4 産物の元の行 `E7RESULT name=a3_o3_baseline states=576 strict_in=576 strict_not_in=0 strict_spec_gap=0 strict_error=0 provisional_in=576 provisional_not_in=0 digest_mismatch_states=0 dump_problem_lines=0 base_unrecoverable_streams=none complete=true freeze_lines_equal_the_reference=true`；8 つの基底状態 O3（独立仕様実行器） 甲乙ともに赤ではない（`name=q2_negative_bases … o3_strict_red_bases=0 o3_provisional_red_bases=0`）。s4 全体の表を実行する前に、同一の固定したツリー上の空変異一行は `baseline-run1` に未到達であり（メインエージェントが実行する表全体スクリプトのステップ 3：`write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false touched=false`）。
- 陽性（装置側三条、登録 5.6）：O1（メモリモデルのチェック（HashMap 理想ファイルシステム））- チェックは、ストリーム 2 の最初の `publish` の観測において、そのバージョンのファイル内容の末尾バイトを 1 ビット反転させ、チェックで 1 箇所を検出；O1（メモリモデルのチェック（HashMap 理想ファイルシステム））- クラッシュ判官は、各ストリーム・バッチ 1 の最初の「実装がファイルを再読取った」状態において、再読取内容の末尾バイトを 1 ビット反転させ、各ストリームで 1 箇所ずつ、合計 2 箇所を検出（登録文言は「最初」だが、装置は各ストリームごとに 1 回ずつ反転させ、必要な「赤」の有無に変更はない）；O2（独立解析器 + checker） は、ストリーム 1 の全永続状態に既知の不良イメージファミリーの第一条を重ね合わせ、1 箇所を検出。ベースラインの 1 つと空変異の 1 つをそれぞれ実行し、赤はすべて `oracle-verdicts.tsv` と `call-outcomes.tsv` に記録される（比較器の `name=positive_control` の 6 行）。O3（独立仕様実行器） の陽性対照（第2段階以降）：ストリーム 1 の全永続状態に P1 パッチを重ね合わせ、固定した仕様実行器に提出し、ベースラインと空変異の 2 つの実行でそれぞれ判紅が 1 箇所となる（s4 産物 `name=positive_control oracle=O3（独立仕様実行器）` の 2 行）。反転させたチェックはモデルをその呼び出し時点で停止させ、以降の呼び出しは比較せず、それ以降の状態に書き込まれたものについては O1（メモリモデルのチェック（HashMap 理想ファイルシステム））- クラッシュ判官が「該当なし」と記録するため、O1（メモリモデルのチェック（HashMap 理想ファイルシステム））- チェックの実行における `o1_crash_judge_red_states` は 0 となる。
- 停止条件：s1 ローカル定数と crates の比較で 42 条すべてが合格；s2 2 つのストリームの 8 つのファイルと `impl-verdicts.tsv`（実行時間列を除外）が、E170（O3 仕様実行器第 1 版）のエクスポート装置が同一のツリーで `dump` として書き出したものとバイト単位で一致；s3 O2（独立したパーサー + チェッカー）が直接呼び出す `check_pool_image` と `judge_crash_image` 内の赤判定（赤判定）不変量セットが、ベースラインの各サンプリング状態において同一である（`o2_matches_judgement_false=0`）、76 の不良状態行において E170（O3 仕様実行器第 1 版）の `bad-impl-verdicts.tsv` プールレベルチェッカーの列と行単位で一致；s4 既知の不良イメージファミリー 41 条、エントリ名と元ファイルがそれぞれ一致し、クリーンイメージのハッシュは `build_pool` と一致する、パッチを適用して直接不良赤判定セットを変更した場合も等しくなる、目標不変量はすべて赤；s5 コーポレーター自身が計算した 2 つの数（バッチ 1 の状態数と番号集合、O1（メモリモデルの対照（HashMap 理想ファイルシステム））-クラッシュ判官が適用可能な状態数）と装置の計数が一致；s6 未トリガー：E170（O3 仕様実行器第 1 版）のエクスポート装置が 2026-10-03 のツリーで `dump` を行い、0 回終了、36 条 `holds=true`、0 条 `holds=false`；s7 第1段階でトリガー（O3（独立した仕様実行器）列は実行されない）、第2段階目以降 O3（独立した仕様実行器）が登録改訂 2 で凍結行をコピーしたバージョンを使用し、s4 コーポレーター開始時と終了時の 2 回の凍結核対照は 0 の違い；s8 の 3 つの呼び出しは「装置、研究側および変異表」セクションで行番号が与えられている。1 行ごとの S5（登録改訂 3、4）は s4 の 117 行において 0 行不合格：判定行 `rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0`、`name=q1_denominator … device_stop=0`。
- テスト対象は実行時のツリー：s4 の空変異、117 行および Q3-核の 3 回が同一の凍結したツリーからスナップショットを取得（5 回の駆動による `snapshot_digest` は 1 つのみ、`d15864653cd83cedddc4902b6598d5db9cf590a62e6cbb8a54bed6661e4700cb`）；実行開始前のツリー検証は、第 6 セグメントの読み込み時のリストとファイル単位で比較し、異なる bin（`e168_delete_ring_window_borrow.rs`）のみが異なり、ブロックされない。第1段階のベースライン、対照および試行には `crates/` の実行時状態を使用；E170（O3 仕様実行器第 1 版）のエクスポート装置側は `git show :crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` から取得した一時バージョン（sha256 `8c526293…`、登録セクション 3 に固定されたもの）をドラフトディレクトリでコンパイルし、その `dump` と `bad-states` 出力はワークスペース内の E170（O3 仕様実行器第 1 版）の 1 回目の再実行で修正されたバージョンとバイト単位で同一である（`cmp` で出力なし），したがって再実行駆動では直接ワークスペース内のバージョンを使用する。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:405-410`（整段抄，未转述）**

```markdown
### ユニットテストと変異

- Python 側の単一テストは 110 件（研究側のディレクトリ内の `grep -c '    def test_' test_*.py` で 4 つのファイルをカウント：32 + 29 + 24 + 25）、`python3 -m unittest test_collusion_matrix test_collusion_pipeline test_o3_columns test_o3_comparison` を連続して 5 回実行し、各回で `Ran 110 tests` / `OK`（手造りデータ、決定論的であり、5 回の試行は隠れた状態がないことを示す）：マトリックス（触達三要素、O3（独立した仕様実行器）の 2 カラムの読み方、全通過語句、1 つを閉じる、閾値、軌跡）、区切りとサンプリング公式の即時計算、エクスポートディレクトリの読み込み、不良実装表（アンカー、代替実行、空変異）、選択スクリプトの行数、固定したツリーでのペアリング、不良ミラーの O3（独立した仕様実行器）判定法、行ごとに駆動によるコンパイル記録と再編成照合（真 `rsync -a` と `copy2`、偽 cargo）、F4 拡張表（選択スクリプト 2 回の差分と番号付け、比較器の行ごとに照合、2 つの出力ルートのマージと再番号付け拒否、分母をテーブルごとに分離、代替実行でスキップする 2 つのテーブルの名前）、コンパイル失敗後の代替実行（手造りグループ r0–r5、表には r0 と r2 が含まれ、装置行が中間に挿入：コンパイル失敗は順次 r1、r3、r4 に切り替える、4 回目の代替実行で失敗すると記録；切り替えられた行のアンカーチェックとコンパイル失敗を合計 3 回カウント；空変異は代替実行しない；グループ内には装置行がない；ドライバが同じ行ディレクトリ内で再編成して切り替えられた行を使用、`compile-failed-1.log` と `failed_attempt_1` は同号；再実行スイッチは代替実行しない；比較器は代替実行行のみでコンパイル失敗した同号行を置き換える；他の状況は拒否；代替実行行の結果行には切り替えられた名前が記述される）。各変異によって壊れる可能性がある語句には絶対値の断言を付与する（例：拡張表を手造りテーブル上では `(118, a4)`、`(119, a12)`、`(120, b2)` の 3 行であること；代替実行の全行 `name=substitution row=3 … substituted_by=c_換 failed_attempts=1 …` を固定する）。
- Python 側で 67 の変異が検出され、表 `scripts/mutation/tables/e171_three_oracle_collusion.tsv` に記録された。実行方法は `mutation_driver.py`（第 9 節の登録事項「`scripts/mutation/mutate.sh` で実行」とあるが、当該スクリプトは cargo test のみを起動し、Python の単体テストを起動できないため、登録修正 1 の第 1 行でこの点を記録した）：出力 `e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out` 内の結果は `E7RESULT name=verdict mutations=67 caught=67 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true` で、検出数 / 無効数 / 赤判定（flag red）なし = 67 / 0 / 0（各結果記号を個数で確認：✅ 67、⏭ 0、❌ 0、⏱ 0、💥 0、⚠️ 0、🧱 0）。x60–x74 は代替実行を行わないバッチ（編不成かつ代替実行しないため回数をカウントせず、装置行のグループ内には含めるが、交換された行のアンカーが期限切れの場合の結果は記録しない。代替実行バッチで取得された行は「編不成」ではない行であるため記録しない。試した名前を記録しない。「編不成」後に直接装置を実行し、行を変えずに実行する。比較器が元の行をチェックしない場合は「編不成」とみなす。代替実行の自らの名前をチェックしない。結果行には元の名前を書く。合計で補えないものを交換としてカウントする。行分析は代替実行を適用しない。ログと試した名前の番号が一致しない。再実行スイッチが機能しない。代替実行バッチにおいて同一行が 2 つのルートに存在する場合、拒否しない）に該当する。同名の `…-r6.out` は同一バッチの最初の 13 条を追加した際の表全体実行結果（65 条、65 / 0 / 0）であり、その後 x73、x74 を追加してアーカイブされた。以下に検討したが追加しなかった「赤判定なしの 3 つのカテゴリ」の一つ：「編不成」後の代替実行は表中の名前から下へ検索するが、「編不成」した行からは下へ検索しない——代替実行では表中で既使用済みの名前をスキップするため、元の名前と「編不成」した行の間にある各名前はこれら 2 つの条件に該当し、すべての入力に対して両方の検索方法で同値となるため、これは等価変異であり表には追加されていない。この等価性はアーカイブテストとして 1 つの項目に記述されている（`test_walking_from_the_name_in_the_table_or_from_the_last_failed_name_gives_the_same_substitute`、手動作成グループが各起点から個別に検証）。このテストを追加した後、変異表を再実行し、出力は `-r7.out` とバイト単位で完全に一致した。x52–x59 は F4 の表拡張バッチ（表の重複行を削除しない、番号付けを 117 から開始、装置自身の変異行を選別法に含める、2 つの出力ルートの符号が一致しても拒否しない、代替実行で他の表をスキップしない、表拡張の確認では行数のみを比較する、分母は表の順序を入れ替えて計算する、最初の出力ルートのみを読み取る）であり、各項目は `F4ExpansionTests` 内の対応する単体テストによって検出された。同名の `…-r4.out` は同一表における途中での停止実行結果（外部で設定した `timeout 230` が第 8 の変異時に実行を停止させたため、61 行に達したが判定行がない）であり、カウントには含まれないがアーカイブされた。より早期の `…-s1-comparator-mutations.out`、`…-s2-comparator-mutations.out`、`…-s3-comparator-mutations.out` および `…-s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` は、表が 9、26、37、40、44、44 条目に達した際の表全体実行結果であり、それぞれすべて検出された。
- 装置内のインライン単体テストは 31 件（`grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`）：編成済み（`scripts/hosts/compile-then-swap.py` の `cargo build` と `cargo test --no-run` を実行）、実行未実施（checker ファイルはコミットに含める）。装置側の変異 15 行が `crates/mutations.tsv` の末尾に追加された（名称は「E171（三 oracle 合謀テスト） 装置」で始まる）。提出時の 3 つの数値については、ゲート（gate）による checker-tier-crates-mutation-replay を通す必要がある。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:411-855`（整段抄，未转述）**

````markdown
### 第 1 次再実行第1段階・改訂 6 のその旅（実行済み：第1段階の量走査完了、Q5 警告自己証明済み、判甲；Q2 の集合と Q3 の分母は第2段階の節）

`research/prompts/e171-r1-prereg.md` を登録する（改訂 6、7 およびそれ以降の各「主担当エージェント 認定」を含み、最後の項目は「`-resume2` のマシン間整合性照合で S17 停止が発生した際の対処」である）；問題票 `e171-r1-questions.md` の Q1–Q5 を含む。ツリーは改訂 6 のツリー `scripts/experiment-apparatus/target/e171-r1-frozen-tree-revision-6/`（スナップショットサマリー `7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4`、284 ファイル；最初の新しいツリーは装置のみを交換し、第 31 バッチの core 断言は含まない）であり、ツリー内の装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` の sha256 は `67e7a549…` である。ステップ 0–10 の後綴は `-r6-r3`、ステップ 11 および 12（16 行）はマルチマシンの実行であり後綴は `-resume2`、ステップ 13 は起動されていない（タイムアウト 0 行）。O3（独立仕様実行器）は E170（O3 仕様実行器第 1 版）の 2 回目の再実行である v2-487（改訂 1 を登録）に対応する。改訂 6 の一連の判定は改訂 6 ツリーにのみ属し、改訂 8 の一連（最終的なメインライン）はまだ実行されていないため、ここではその上に数値を一切推し進めない。コンパレーターはこの一連の出力の前缀 `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-`、後綴 `-r6-r3.out` およびエクスポートルート `scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/` のみを参照し、異なる前缀が与えられた場合は終了する（改訂 8 第 8.5 節第 1 条を登録）。

| 行 | 判定（修正版 6 ツリー） | 根拠（生成物） | 文文言以外に併せて読むべきもの |
|---|---|---|---|
| q1 | 甲：七行が、このバッチの三条流上で各行あたり少なくとも一条赤（バッチ 1 定判、厳密と暫代という二つの O3（独立仕様実行器）読み取り方とも甲；七行すべて触達、三条流すべて完了、無効行 0） | `…-s1-comparator-resume2-r6-r3.out` の `name=q1 reading=strict streams=three_streams batch=1 … verdict=jia` | 赤がどのセルにあるか：第 15 行 O2（独立解析器 + チェッカー）、三条流、I-5.5（mkfs の水位でツリーを構築するバージョンの t1 の着地点）、第 23 行 O2（独立解析器 + チェッカー）、ストリーム3のみ、I-1.1（ブロックヘッダーが自己申告する論理アドレス）、I-9.4（コンテナ番号は最小キーを超えない）、I-9.12（区切りキーは子区間の外に落ちる）、第 42 行 O1（メモリモデルの対照（HashMap の理想ファイルシステム））-対照（ストリーム2第 14 回目の `unmount` 呼び出し、`disagrees:根が持つ F`）、第 242、269、270 行 O3（独立仕様実行器）、厳密・暫代ともに同数、第 271 行 O2（独立解析器 + チェッカー）、三条流、I-9.27（ツリーテーブルエントリのヘッダー ID の値）。ストリーム1とストリーム2のみを採取（s4 と同じバッチの履歴口径）の場合、第 23 行の三列すべて赤ではない（`streams=two_streams … all_quiet_rows=23 verdict=yi`）：第 23 行はストリーム3のその区間の跨葉の歴史にのみ検出される。バッチ 2 の判定も甲であり、6 セルが二つのバッチの間で反転する（「q1 をセルごと（修正版 6 のその回）」節）。O1（メモリモデルの対照（HashMap の理想ファイルシステム））の二つのクラッシュサブ列は七行すべてにおいて一態とも赤ではない |
| Q2 | 第1段階は七行のみを報告（記述量）；第2段階は表全体を乙と判定（「第 1 回目の再実行第2段階・修正版 6 のその回」節） | `name=q2_named_row` 14 行（全行を「Q2 七行、対照と Z1（修正版 6 のその回）」節にコピー） | 七行はストリーム1・ストリーム2の両方で触達されていない；第 155、161 行それぞれ 1 回の呼び出しで返却値が異なる、グルーがモデルに渡す観測も異なる、O1（メモリモデルの対照（HashMap の理想ファイルシステム））-対照赤 1 回；第 49、110、162、174、252 行は返却値またはメモリオブジェクトが異なるが観測は同じ、O1（メモリモデルの対照（HashMap の理想ファイルシステム））-対照赤ではない |
| q3 | 甲：第 125 行を分母に含める（c = 12、触達） | `name=q3 row=125 call_panic=true judged_calls_before_the_first_panic=12 … touched=true verdict=jia` | パニックはストリーム2の第 14 回目の `unmount` 呼び出しで発生（`attempt to subtract with overflow`）、それ以前の第 2–13 回目の呼び出しでは O1（メモリモデルの対照（HashMap の理想ファイルシステム））-対照すべてが `agrees`；ストリーム1にはパニックがないため、コンパレーターはパニックが発生したそのストリームのみを数える（ストリーム1の 1 回の `agrees` も含めて 13 とする読み方もあるが、二つの読み方とも ≥ 1 であり判定は変わらない）。分母 D = 39 は第2段階の表全体から出ている（「第 1 回目の再実行第2段階・修正版 6 のその回」節）、第 125 行もそこでは同様に含まれる |
| Q4 | ベースライン A（状態 (ii)0）；行 242 B（状態 (ii)12、セグメント全体が赤/無効でフラグ付け済み）；行 269 B（状態 (ii)58、全体が赤/無効でフラグ付け済み） | `name=q4_trip` 3 行（セクション「Segment One の最初の再実行 · 修正版 6」の判断行の後にコードブロックにコピーされた全行） | 3 回の実行 × 3 つのストリーム × 2 バッチ、各 18 セルをプローブ（`cells_without_a_probe=0`）。Q4 の照合：すべての 18 セルの 2 つの数値は 0（`witness_refusals_without_either_branch=0`、`counter_branch_mounted=0`）、S11 はトリガーされず。(i) 別々に報告されたブランチを特定できない：ベースライン 1 状態 340、行 242 状態 2650、行 269 状態 1340（両バッチの合計、`undecidable_branch=`）、候補には入らない。(ii) 状態ごとのリストとセグメントおよび inode 構築によって並べ替えられた軌道は、「Q4-(ii) State-by-State List and Q4, Q5 Trajectories (Revision 6 Run)」セクションにあります |
| Q5 | A、警報の自己明証性が通過した（プライマリーサンプリングポイントとリバースサンプリングポイントの両方で RefFS の赤色状態が L に等しい；モデル崩壊判定 0 状態赤色、両ポイント同様に判断） | `name=q5_judged` 2 行（セクション「Segment One の最初の再実行 · レビジョン 6」内の判定行の後にコードブロックに完全コピーされた全行）；警報の自己明証性はセクション「有効条件と Q5（レビジョン 6 実行）」の最後の項目を参照 | レビジョン 5 セクション 5.6 アイテム 7 とレビジョン 6 セクション 6.6 で要求される警報の自己明証性（U1–U9、U13–U16 および 2 つの元のインラインテストケースが緑色、`crates/mutations.tsv` が V17–V31、V33–V36 をすべてキャプチャ；U10–U12、V32 はこの項目に含まれない）をメインエージェントが `SINGLEFS_HEAVY_TESTS=user-request` で事前に実行し（「Main Agent Determination: Segment One (Revision 6 Tree) Q5 Alarm Self-Evidence」で登録された有効）、プライマリーサンプリングポイントの L は同値クラスによって導出される（`derived_by=equivalence_classes judged_states=144`）、同値関係は k = 6 の 4,608 状態に対して状態ごとにのみ検証され（`mismatching_states=0`）、残りの状態はレビジョン 5 の A（レビジョン 5 セクション 5.2）に対してクラスでチェックされる。末尾に書き込み順で整理された軌跡はセクション「Q4-(ii) State-by-State List and Q4, Q5 Trajectories (Revision 6 Run)」にあります |

コンパレータの判定行（`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out` の `name=verdict` 行；逐状態リストと軌跡を補完した後の `…-s1-comparator-resume2-r6-r3-2026-10-07.out` の `name=verdict` 行はそれとバイト単位で同一であり、この節の抄録された判定行および Q1、Q3、Q4、Q5、陰性、カバレッジ各行は両方のファイルに存在する）：

```
E7RESULT name=verdict q1=jia q3=jia q4_baseline=jia q4_row_242=yi q4_row_269=yi q5_main=jia q5_reverse=jia q5_same_at_both_sampling_points=true a1_baseline_red=0 a8_failing_controls=0 a11_failing_streams=0 z1_differing_files=0 f7=false a1_empty_mutation_red=0 empty_mutation_touched=false
```

同一の産物の Q1、Q3、Q4、Q5 と陰性、各行を被覆する（全行コピー）：

```
E7RESULT name=a1 batch=1 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1 batch=2 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=o3_third_stream_coverage stream=first_file_unlinked_then_inodes_across_one_leaf sampled_states=6459 strict_in_or_not_in_states=6459 f7=false
E7RESULT name=a11 stream=new_pool_file_creation states=82 crash_judge_applicable_device=82 crash_judge_applicable_recomputed=82 reference_filesystem_voting_device=82 reference_filesystem_voting_recomputed=82 holds=true
E7RESULT name=a11 stream=fixed_script_through_e_then_unmount states=890 crash_judge_applicable_device=890 crash_judge_applicable_recomputed=890 reference_filesystem_voting_device=890 reference_filesystem_voting_recomputed=890 holds=true
E7RESULT name=a11 stream=first_file_unlinked_then_inodes_across_one_leaf states=11695 crash_judge_applicable_device=11695 crash_judge_applicable_recomputed=11695 reference_filesystem_voting_device=81 reference_filesystem_voting_recomputed=81 holds=true
E7RESULT name=a1_empty_mutation batch=1 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1_empty_mutation batch=2 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1_empty_mutation_touch row=0 streams_present=3 touched=false
E7RESULT name=q1 reading=strict streams=three_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=strict streams=three_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=strict streams=two_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=strict streams=two_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=provisional streams=three_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=provisional streams=three_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=provisional streams=two_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=provisional streams=two_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q3 row=125 call_panic=true judged_calls_before_the_first_panic=12 states_judged=514 recovery_panic_states=0 touched=true verdict=jia
E7RESULT name=q4_trip trip=baseline counter_branch=0 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=jia
E7RESULT name=q4_trip trip=row_242 counter_branch=12 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=yi
E7RESULT name=q4_trip trip=row_269 counter_branch=58 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=yi
E7RESULT name=q5_judged sampling_point=main lost=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 q5_negative_o1_red=0 a10_states_equal_the_product=true reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 verdict=jia
E7RESULT name=q5_judged sampling_point=reverse lost=9 reference_filesystem_red=9 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 q5_negative_o1_red=0 a10_states_equal_the_product=true reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 verdict=jia
```

#### Q1 逐格（改訂 6 のその回）

バッチ 1:各行の各ストリームで赤い列（O1（メモリモデルの拍子取り（HashMap 理想ファイルシステム））- 呼び出しごとに拍子取りを行い、それ以外は状態ごとに行う；括弧内は軌跡：ピーク値、正の区間数、期末値、単点赤の区間数）。倉庫のルートで実行するコマンド（`name=q1_row … batch=1` の 21 行のうち非ゼロの列を出力するもの）：

```
grep 'name=q1_row .* batch=1 ' e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out | awk '{o=""; for(i=1;i<=NF;i++){split($i,a,"="); f[a[1]]=a[2]} if (f["o1_comparison_red_calls"]!="0") o=o" O1-对拍="f["o1_comparison_red_calls"]; n=split("O1-模型崩溃判官 O1-RefFS O2 O3甲 O3乙",c," "); for(j=1;j<=n;j++){k=c[j]; if (f[k"_red"]!="0") o=o" "k"="f[k"_red"]"(峰"f[k"_peak"]",正段"f[k"_positive_segments"]",末"f[k"_last"]",单点"f[k"_single_points"]")"} print f["row"], substr(f["stream"],1,9) (o==""?" 全不红":o)}'
```



```
15 new_pool_ O2=9(峰8,正段2,末1,单点1)
15 fixed_scr O2=240(峰8,正段47,末0,单点11)
15 first_fil O2=480(峰8,正段96,末0,单点24)
23 new_pool_ 全不红
23 fixed_scr 全不红
23 first_fil O2=29(峰8,正段6,末1,单点2)
42 new_pool_ 全不红
42 fixed_scr O1-对拍=1
42 first_fil 全不红
242 new_pool_ O3甲=4(峰4,正段1,末0,单点0) O3乙=4(峰4,正段1,末0,单点0)
242 fixed_scr O3甲=8(峰4,正段2,末0,单点0) O3乙=8(峰4,正段2,末0,单点0)
242 first_fil O3甲=4(峰4,正段1,末0,单点0) O3乙=4(峰4,正段1,末0,单点0)
269 new_pool_ O3甲=8(峰4,正段3,末0,单点1) O3乙=8(峰4,正段3,末0,单点1)
269 fixed_scr O3甲=41(峰8,正段10,末0,单点3) O3乙=41(峰8,正段10,末0,单点3)
269 first_fil O3甲=8(峰4,正段3,末0,单点1) O3乙=8(峰4,正段3,末0,单点1)
270 new_pool_ O3甲=3(峰3,正段1,末0,单点0) O3乙=3(峰3,正段1,末0,单点0)
270 fixed_scr O3甲=30(峰3,正段10,末0,单点0) O3乙=30(峰3,正段10,末0,单点0)
270 first_fil O3甲=714(峰3,正段238,末0,单点0) O3乙=714(峰3,正段238,末0,单点0)
271 new_pool_ O2=9(峰8,正段2,末1,单点1)
271 fixed_scr O2=353(峰8,正段69,末1,单点17)
271 first_fil O2=4749(峰8,正段950,末1,单点238)
```

- 登録読み取りの解釈 12（C673：三 Oracle の共謀テストにおける補完 Oracle の改変が実施されなかった）の修正箇所：第 15 行 ← ⑥ O2（独立解析器 + チェッカー）新プール新規ファイル作成箇所、O2（独立解析器 + チェッカー）赤（I-5.5（mkfs の水位から木を構築するバージョンの t1 の箇所））；第 23 行 ← ⑦ ストリーム三、O2（独立解析器 + チェッカー）はストリーム三のみで赤（I-1.1（ブロックヘッダーによる論理アドレスの自己申告）、I-9.4（コンテナ番号が最小キーを超えない）、I-9.12（区切りキーが子区間の外に位置する））；第 42 行 ← ④ モデルのアンマウント応答、O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照はストリーム二における `unmount` で赤；第 242、269 行 ← ② O3（独立規約実行器）述語 W、O3（独立規約実行器）赤；第 270 行 ← ③ O3（独立規約実行器）証言値が tail より低くない、O3（独立規約実行器）赤；第 271 行 ← ⑤ O2（独立解析器 + チェッカー）木表エントリヘッダー ID、O2（独立解析器 + チェッカー）赤（I-9.27（木表エントリのヘッダー ID の値））。このバッチでは 7 つの箇所がすべて赤となった。O2（独立解析器 + チェッカー）の赤色を示す不変量の名称は、各行から導出された `oracle-verdicts.tsv` の第 8、9 列（両バッチ合計）より取得した。
- バッチ 1 とバッチ 2（第 8 節：幾何学的感受性）：コマンド `grep 'name=q1_row' <同一の出力> | awk '{for(i=1;i<=NF;i++){split($i,a,"="); f[a[1]]=a[2]} n=split("o1_comparison_red_calls O1（メモリモデルの対照（HashMap 理想ファイルシステム））-モデルクラッシュ判定官_red O1（メモリモデルの対照（HashMap 理想ファイルシステム））-RefFS_red O2_red O3（独立規約実行器）甲_red O3（独立規約実行器）乙_red",c," "); for(j=1;j<=n;j++){k=f["row"]" "substr(f["stream"],1,9)" "c[j]; r[k, f["batch"]]=(f[c[j]]!="0"); keys[k]=1}} END{t=0; m=0; for(k in keys){t++; if (r[k,1]!=r[k,2]) {m++; print k, "batch1_red=" r[k,1], "batch2_red=" r[k,2]}} print "cells=" t, "flipped_between_batch_1_and_2=" m}' | sort`、出力の最終行 `cells=126 flipped_between_batch_1_and_2=6`。反転した 6 のセルは、第 15 行および 271 行のストリーム一の O2（独立解析器 + チェッカー）、第 269 行のストリーム一とストリーム三の O3（独立規約実行器）厳密・暫定である——これら 6 セルを「不安定」と記録する（バッチ 2 では赤色を取得しなかった箇所）。各行はバッチ 2 でも他のセルが赤色を示しており、Q1 のバッチ 2 は依然として甲と判定された。
- 第 23 行および 42 行の赤色は、単一のストリームおよび単一の Oracle 上でのみ発生した；第 42 行では 1 回の呼び出しのみ（O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照は呼び出し数で計算し、トレースはない）。
- 3 つの行ごとに出力において、第 252 行および 271 行では `line_matches_the_registration=false`（本文の sha256 が登録値 5.9 と異なるため、凍結ツリーの該当行を実行し別途記録）、残りの 14 行は `true`；`substituted_from=-`、`failed_attempts=0`であり、補完はない。

#### Q2 7 行、対照と Z1（改訂 6 のその旅）

```
E7RESULT name=q2_named_row row=49 streams=streams_1_2 touched=false calls_with_a_difference=11 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=11 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=49 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=110 streams=streams_1_2 touched=false calls_with_a_difference=14 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=14 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=110 streams=stream_3 touched=false calls_with_a_difference=238 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=238 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=155 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=1
E7RESULT name=q2_named_row row=155 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=161 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=1
E7RESULT name=q2_named_row row=161 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=162 streams=streams_1_2 touched=false calls_with_a_difference=11 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=11 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=162 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=174 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=1 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=174 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=252 streams=streams_1_2 touched=false calls_with_a_difference=14 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=14 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=252 streams=stream_3 touched=false calls_with_a_difference=238 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=238 o1_comparison_red_calls=0
E7RESULT name=a8 control=O1-对拍 products=1 counted=o1_comparison_red_calls expected=1 observed=1 holds=true
E7RESULT name=a8 control=O1-对拍-卸载 products=1 counted=o1_comparison_red_calls expected=1 observed=1 holds=true
E7RESULT name=a8 control=O1-崩溃判官 products=1 counted=o1_crash_judge_red_states expected=3 observed=3 holds=true
E7RESULT name=a8 control=O2 products=1 counted=o2_red_states expected=1 observed=1 holds=true
E7RESULT name=a8 control=O3 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=RefFS products=1 counted=o1_reference_filesystem_red_states expected=3 observed=3 holds=true
E7RESULT name=a8 control=panic products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=交回值 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=内存对象 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=见证 products=1 counted=counter_branch_states expected=1 observed=1 holds=true
E7RESULT name=z1_unstable_memory_objects names=-
```

- 各回の呼び出しにおいて行ごとに処理（`name=q2_named_row_call`、どのトップレベルフィールド、どのメモリオブジェクトが異なるか）は同一の出力物の中で行われ、529 行目はコピーしない。ストリーム三上の第 110 行および 252 行における 238 回の呼び出しではすべて数値書き込み（`writes`）と `current_version` が異なる。
- a8 の 4 つの項目である `checked=false` のもの（O3（独立仕様実行器）、panic、返却値、メモリオブジェクト）は順序実行法の第 3 ステップにおける独自の判定法によって判定済みである（`…-s1-positive-control-o3-p1-r6-r3.out` の `o3_control_verdict strict=not_in base_digest_matches=true red_strict=true`、返却値およびメモリオブジェクトの呼び出し比較と panic 切断はそれぞれ 1 コアずつ）。空変異のそのラウンドにおける 10 項目（`…-s1-empty-control-*-driver-r6-r3.out`）も第 4 ステップで判定され、比較器はそれらを読み取らない。
- Z1：39 行 `name=z1` はすべて `identical=true`（`grep -c 'name=z1 .*identical=true'` と `grep -c 'name=z1 '` の両方が 39）。ベースラインの 2 回の間には不安定なメモリオブジェクトがない（`names=-`）。

#### Q4-(ii) ステートごとのリストと Q4、Q5 の軌跡（改訂 6 のその旅）

第 6 節 Q4-（ii）の逐状態リストと第 8 節 Q4、Q5 の軌跡は、コンパレータによって補完され、`research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out`（811 行）に含まれています。`…-s1-comparator-resume2-r6-r3.out`（679 行）と比較すると、両ファイルの先頭677行はバイト単位で一致し、判定行どうしも一致します（新しいファイルの判定行は810行目）。新しいファイルではQ5の判定後、判定行の前に132行だけ追加されています（`name=q4_counter_branch_state` 70 行、`name=q4_trajectory` 18 行、`name=q4_trajectory_by_inode_creation` 6 行、`name=q5_trajectory_cell` 36 行、`name=q5_trajectory` 2 行）。末尾の行は `E7RESULT name=done emitted=679` から `E7RESULT name=done emitted=811` に変更されています（`diff` は `677a678,809` と `679c811` のみを報告）。軌道の 3 つの指標（ピークとその初出セル、正のセル数、期末値）は「端点は軌道ではない」として報告されます。期末値とはセル番号が最大のセルを指し、Q4 は区間ごとに並べ替えられた場合においてこのバッチで取得された最後の区間、inode 作成順に並べ替えられた場合においては第 235 回の inode 作成に対応するセル、Q5 では最終区間全体が落下したセルとなります。

逐状態リストはグリッドに統合される（列の状態：趟、流、批、段、このセグメントが第何回の呼び出しで発行されたか、カウントするブランチがどの文か、マウント結果、2 番目の区切り判定；各状態には別途番号、状態番号、着地文字列の sha256、イメージサマリー、c_証人、選択されたバージョン、拒否されたメンバー、単子上の理由と満たされなかった条件が含まれる（プロダクト内））：

```
grep 'name=q4_counter_branch_state ' <新那份> | awk '{o=""; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="trip"||a[1]=="stream"||a[1]=="batch"||a[1]=="segment"||a[1]=="call_of_the_segment"||a[1]=="counter_sentence"||a[1]=="mount"||a[1]=="second_crop") o=o" "$i}; print o}' | sort | uniq -c | sort -k2,2 -k3,3 -k4,4
      2  trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 segment=14 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      2  trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 segment=14 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_242 stream=new_pool_file_creation batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=new_pool_file_creation batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=18 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=21 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=15 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      3  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=17 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      3  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=19 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      8  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=16 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      8  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=20 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      7  trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 segment=16 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      7  trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 segment=20 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=new_pool_file_creation batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=new_pool_file_creation batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=new_pool_file_creation batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=new_pool_file_creation batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
```

第 3 つのストリームの第 0、1 回目の呼び出しは番号取得とウォームアップであり、ストリーム 2 の第 4 回目の呼び出しは書き込みマウントです（各行の `calls.tsv` の第 2 列参照）。満たされていない条件は、カウント支分ごとに 1 つずつ対応付けられます：`RecordAtTheWitnessedCounterIsNewer` は `witness_criterion_true_without_read_faults:RecordAtTheWitnessedCounterNewerThanTheSelectedVersion` に、`AboveTheLastRecordOfTheSelectedVersion` は `…:WitnessedCounterAboveTheLastRecordOfTheSelectedVersion` に、`NeitherRecordReadable` は `…:NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable` に対応します。各セルにリストされた状態数は、同じセルの `name=q4` 行の `counter_branch=` と等しくなります（18 セルすべてで一致し、0 セルが不一致）。また、軌跡の各セルの合計も同様に等しくなります（18 セルすべてで一致し、0 セルが不一致）。

セグメント別に示す（行242、269。ベースラインの6行はいずれも0）：

```
E7RESULT name=q4_trajectory trip=row_242 stream=new_pool_file_creation batch=1 cells=segments segments=9 first_segment=0 last_segment=8 counter_branch_by_cell=2:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=new_pool_file_creation batch=2 cells=segments segments=4 first_segment=2 last_segment=7 counter_branch_by_cell=2:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 cells=segments segments=60 first_segment=0 last_segment=59 counter_branch_by_cell=2:2,14:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 cells=segments segments=36 first_segment=2 last_segment=58 counter_branch_by_cell=2:1,14:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 cells=segments segments=720 first_segment=0 last_segment=719 counter_branch_by_cell=2:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 cells=segments segments=478 first_segment=2 last_segment=718 counter_branch_by_cell=2:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=new_pool_file_creation batch=1 cells=segments segments=12 first_segment=0 last_segment=11 counter_branch_by_cell=0:2,1:3,2:1 counter_branch_peak=3 counter_branch_peak_at=1 counter_branch_positive_cells=3 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=new_pool_file_creation batch=2 cells=segments segments=1 first_segment=7 last_segment=7 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 cells=segments segments=79 first_segment=0 last_segment=78 counter_branch_by_cell=0:2,1:3,2:1,15:2,16:8,17:3,18:1,19:3,20:8,21:1 counter_branch_peak=8 counter_branch_peak_at=16 counter_branch_positive_cells=10 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 cells=segments segments=17 first_segment=7 last_segment=74 counter_branch_by_cell=16:7,20:7 counter_branch_peak=7 counter_branch_peak_at=16 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 cells=segments segments=960 first_segment=0 last_segment=959 counter_branch_by_cell=0:2,1:3,2:1 counter_branch_peak=3 counter_branch_peak_at=1 counter_branch_positive_cells=3 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 cells=segments segments=238 first_segment=7 last_segment=955 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
```

ストリーム3按第幾次インオード作成で並べる（235 回目にインオードを作成し、各 1 グリッド；セグメントはインオード作成呼び出しに含まれない（ii）状態を `counter_branch_outside_create_inode_calls` に記録し、全永続状態のその部分を `counter_branch_after_every_call` に記録する）：

```
E7RESULT name=q4_trajectory_by_inode_creation trip=baseline stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=baseline stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=2 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=1 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=6 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
```

Q5 は最終区間に着地した書き込み数ごとに並べる（主サンプリング点は30区画、逆方向のサンプリング点は6区画。装置の `name=q5_trajectory` 行には L 状態が存在する区画だけが出るため、比較器が区画0から最終区間の書き込み数に対応する区画までを補い、欠けた区画は0と記録する。各区画の合計は `name=q5` 行の L と2つの赤判定数と照合済み）：

```
E7RESULT name=q5_trajectory sampling_point=main cells=30 last_segment_writes=29 cells_add_up_to_the_q5_line=true reference_filesystem_red_by_cell=0:1,1:28,2:376,3:3224,4:19826,5:93104,6:347116,7:1054504,8:2658271,9:5633804,10:10133156,11:15571184,12:20533756,13:23297344,14:22762456,15:19137104,16:13810511,17:8518004,18:4460896,19:1965304,20:719026,21:214544,22:50876,23:9224,24:1201,25:100,26:4 reference_filesystem_red_peak=23297344 reference_filesystem_red_peak_at=13 reference_filesystem_red_positive_cells=27 reference_filesystem_red_last=0 model_crash_judge_red_by_cell=- model_crash_judge_red_peak=0 model_crash_judge_red_peak_at=- model_crash_judge_red_positive_cells=0 model_crash_judge_red_last=0
E7RESULT name=q5_trajectory sampling_point=reverse cells=6 last_segment_writes=5 cells_add_up_to_the_q5_line=true reference_filesystem_red_by_cell=0:1,1:4,2:4 reference_filesystem_red_peak=4 reference_filesystem_red_peak_at=1 reference_filesystem_red_positive_cells=3 reference_filesystem_red_last=0 model_crash_judge_red_by_cell=- model_crash_judge_red_peak=0 model_crash_judge_red_peak_at=- model_crash_judge_red_positive_cells=0 model_crash_judge_red_last=0
E7RESULT name=q5_trajectory_cell sampling_point=main writes_landed_in_the_last_segment=13 lost_states=23297344 reference_filesystem_red=23297344 model_crash_judge_red=0
E7RESULT name=q5_trajectory_cell sampling_point=main writes_landed_in_the_last_segment=29 lost_states=0 reference_filesystem_red=0 model_crash_judge_red=0
E7RESULT name=q5_trajectory_cell sampling_point=reverse writes_landed_in_the_last_segment=5 lost_states=0 reference_filesystem_red=0 model_crash_judge_red=0
```

読み方（判定を変更せず記述のみ）：
- Q4 の（ii）状態は、2 行の両方で番号取得およびウォームアップの 2 回の呼び出しのセグメント（セグメント 0–2）と、ストリーム 2 が書き込みマウントされた際の呼び出しのセグメント（第 242 行 セグメント 14、第 269 行 セグメント 15–21）に集中している。3 回とも各ストリームの各バッチの期末値はすべて 0 であり、2 回目の判定が合法な状態はすべてのセルで 0 である。第 269 行 ストリーム 2 バッチ 1 のピーク値 8 はセグメント 16 にあり（セグメント 20 も同様に 8）、正のセグメント数 10 である。
- ストリーム 3 の（ii）状態は、すべてインオードル作成以外の呼び出しに存在する（第 242 行 バッチ 1 セグメント 2 の状態、バッチ 2 セグメント 1 の状態、第 269 行 バッチ 1 セグメント 6 の状態）。235 回のインオードル作成において各回とも 0 状態である：このバッチでは（ii）がリーフコンテナの分裂に伴い増減する様子は見えない。インオードル作成順に並べられたセルの読み取り値は 0 であり、「分裂前後ともに存在」というものではない。
- Q5 の RefFS 赤状態数は、L 状態が存在するすべてのセルにおいて、そのセルの L 状態数と等しい（主要サンプリングポイント 27 セル、逆方向サンプリングポイント 3 セル）。モデルクラッシュ判定官はすべてのセルで 0 である。最終セル（最終セグメントすべてが該当）の期末値は 0 であり、これはそのセルに L 状態が存在しないため（`lost_states=0`）ではなく、RefFS がそのセルを緑と判定していないからである。
- `q5_verdict` は、L の状態において赤状態が一つも存在せず、かつ両方とも緑で報告される結果がない場合の終了値を `not_enough` から `not_enough_f8` に変更する（登録改訂 5 第 5.1 節第 7 条：F8 で不十分と記録）；この回では 2 つのサンプリングポイントとも赤が存在するため、そのブランチには到達せず、`q5_main=jia q5_reverse=jia` は変更されない。

#### 作動的条件と Q5（改訂 6 のその回）

Q1–Q4 の作数要第十一節の原文に S15 ①②③⑤⑥ を加えても、いずれもトリガーしない（改訂版 6 の第 6.6 節参照）；Q5 は別途、主サンプリングポイントで 2 回とも S1 が全通し、`s13=false`、陽性対照「判態路 panic 単列」が正確な数値となり、2 回の逐バイト比較が同一（Z7）、逆方向サンプリングポイントで S1 が全通し、S15 ④ はトリガーせず、さらに警告による自己証明（既に通過、「作数の条件と Q5（改訂版 6 のその回）」節の最終条）を追加する。一行をそのままコピーする（ファイル名のプレフィックス `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-` を省略）：

```
frozen-tree-r6-r3.out:61: E7RESULT name=verdict s15_1_extra_files_against_the_first_segment_tree=0 s15_1_extra_files_against_the_q5_tree=0 s15_2_holds=true s15_3_unassigned_hunks=0 s15_3_hunks=24 s15_3_table_rows=24 a12_failed_checks=0 a12_checks=12 triggered=none
stream-constants-r6-r3.out:82: E7RESULT name=verdict s1_failed_checks=0
s2-r6-r3.out:20: E7RESULT name=verdict s2_differing_files=0 s2_compared_files=18
s15-5-step-1-r6-r3.out:26: E7RESULT name=verdict s15_five_problems=0 triggered=false
s15-5-step-7-r6-r3.out:28: E7RESULT name=verdict s15_five_problems=0 triggered=false
s15-6-step-7-r6-r3.out:3: E7RESULT name=verdict s15_six_problems=0 triggered=false
s15-4-step-6-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
s15-4-step-10-run-1-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
s15-4-step-10-run-2-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
baseline-run1-r6-r3.out:2035: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 o2_red_states=0 recovery_panic_states=0 probed_states=12667 counter_branch_states=0 counter_branch_states_judged_legal=0 undecidable_branch_states=1340 witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0 piece_panic_states=- unattributed_panic_states=0
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:19: E7RESULT name=q5_equivalence_pairing in_flight_writes_kept=6 states=4608 product_of_the_choices=4608 class_states=4608 classes=144 mismatching_states=0 crash_judge_not_applicable_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:23: E7RESULT name=q5_piece_panic_control states=4608 pool_checker=1152 ignored_recovery=1152 record_checker=576 other_pieces=0 consulted_recovery=0 consulted_recovery_of_the_plain_pairing=0 o1_columns_differing_from_the_plain_pairing=0 mismatching_states=0 s13=false holds=true
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:25: E7RESULT name=q5_shape sampling_point=main_before_the_first_barrier_of_call_3 removed_barriers=[50,51,55,56] root_writes_made_plain=[52] truncation=81 operations_kept=77 confirmed_root=1,3 confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:54: E7RESULT name=q5_equivalence sampling_point=main_before_the_first_barrier_of_call_3 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:200: E7RESULT name=q5 sampling_point=main_before_the_first_barrier_of_call_3 states=1207959552 last_segment_writes=29 last_segment_tearable_writes=2 product_of_the_choices=1207959552 a10_states_equal_the_product=true lost_the_confirmed_version=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 not_lost_states=1056964608 not_lost_o1_red=0 states_without_a_root=0 derived_by=equivalence_classes judged_states=144
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:228: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=144 s13=false piece_panic_states=- unattributed_panic_states=0
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:19: E7RESULT name=q5_shape sampling_point=reverse_at_the_return_point_of_call_2 removed_barriers=[50,51,55,56] root_writes_made_plain=[52] truncation=57 operations_kept=53 confirmed_root=1,3 confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:47: E7RESULT name=q5 sampling_point=reverse_at_the_return_point_of_call_2 states=72 last_segment_writes=5 last_segment_tearable_writes=2 product_of_the_choices=72 a10_states_equal_the_product=true lost_the_confirmed_version=9 reference_filesystem_red=9 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 not_lost_states=63 not_lost_o1_red=0 states_without_a_root=0
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:51: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=72 s13=false piece_panic_states=- unattributed_panic_states=0
```

- Z7: 主サンプリング点の 2 つの E7RESULT 行がバイト単位で同一である（`diff <(grep '^E7RESULT' …-run-1-r6-r3.out) <(grep '^E7RESULT' …-run-2-r6-r3.out) | wc -l` が 0 を出力し、経過時間フィールドを考慮しなくてもよい）。
- ステップ 11 および 12 の合成プロダクト（`…-s1-rows-probe-driver-resume2-r6-r3.out`、`…-s1-rows-driver-resume2-r6-r3.out`）では、`freeze_check` の前後とも `differences=0` であり、`snapshot_digest=7726cdd6…`、16 行の `status=ok`、`device_exit=0`、`o3_status=ran` が存在するが、253 は見当たらない（ステップ 13 は実行されていない）。
- S17（ホスト間整合性チェック）：元のプロダクト `…-s1-rows-cross-host-check-resume2-r6-r3.out` は `E7RESULT name=verdict cross_host_identical=false row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=o3` と判定され、無効化され判定には進めなかった。メインエージェントは（「-resume2 フラグ付きのホスト間整合性チェック S17 における停止処理」の注釈末節第 2–4 条を参照）、当該セルの違いが凍結実行器のプロセス ID、プロセス数、マルチプロセス行順に限定され、これはチェッカーの欠陥であり、2 つのホスト間の不一致ではないと認定した。有効なエントリは新たに生成された `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07.out`（`E7RESULT name=verdict cross_host_identical=true row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=none`）である。`…-2026-10-07-r2.out` はリポジトリ内で同じエントリを再生成し、末尾行 `E7RESULT name=done emitted=8` を補完したバージョンであり、最初の 7 行は `…-2026-10-07.out` とバイト単位で同一である。エントリを読み込むディレクトリ内のオーケストレーターを変更して再生成された `…-2026-10-07-r3.out` は再実行登録版であり、`-r2` とは 2 行目の `entry_sha256=` のみ異なる。
- アラートの自己証明は完了しており、Q5 の判定は有効である：メインエージェントは `SINGLEFS_HEAVY_TESTS=user-request` を設定して早期に実行した（ユーザーによる 2026-10-07「可能な限り前倒しで実行」の要請）、ユニット `e171-q5-alert-selfproof`、ログは `~/.local/state/singlefs/detached-runs/e171-q5-alert-selfproof/log` にある。注釈末節「メインエージェント認定：第1段階（改訂 6 ツリー）Q5 アラート自己証明」からそのまま転記した読み取り値では、リポジトリ内のデバイス sha256 `ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df`、デバイス内のインラインテストケース出力 `test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.75s`（U1–U9、U13–U16 および既存の 2 つを含む）、`crates/mutations.tsv` の「E171R1 デバイス」に関する全 49 行（V17–V31、V33–V36 を含む）は以下の通り：カウント：検出 49、赤旗なし 0、無効 0、メモリ上限到達 0、タイムアウト 0、総上限超過で除外 0、スケジューリングにより実行未実施 0、スコープ起動失敗で実行未実施 0（合計 49）。提出時にこれら 2 つの項目はチェッカーファイル内で再実行される。その時点でリポジトリ内のデバイスが `ab35105f…` でない場合は、当該インスタンスを基準として別途記録する。

#### コンパレータ、再実行とリポジトリへの格納の実行方法（改訂 6 のラウンド）

- 比較器 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py` は、第1段階新樹のバージョンに対して 8 か所変更され、判据は一字も動かず（①–⑥ は主担当エージェントが認識；登録末節「主担当エージェント 認定：第1段階（修訂 6 樹）比較器交回」第 1 条参照；⑦ ⑧ は同条第 3 条で点名された補足事項）：① `oracle-verdicts.tsv` の読み込みを 25 列に変更（`run_directory.py`、修訂 5 第 5.1 節第 5 条末尾に `piece_panics` を追加；24 列の読み込みはこの回のエクスポートでは動作しない）；② a11 の現算を「最後の書き込みが着いた呼び出し」に変更（個別書き込みごとに最大値を取るのと同等、単測で対拍）、ストリーム三の状態 11 は推論の 10 分間で 695 状態から全体比較器では 13 秒に短縮；③ `--dumps-root`、`--product-prefix`、`--product-suffix` の指定が必須で、入力がない場合はコード 2 で終了し、産物を書かない（修訂 8 第 8.5 節第 1 条）；④ 「ストリーム三カバレッジ（O3（独立規約実行器）」行と判決行の `f7=` を追加（第六節、f7）；⑤ 空変異行に a1 と触達列を追加（第七節 a1 の後半）；⑥ `q5_judged` 行に `judge_panic` の状態数を 2 列追加（修訂 5 第 5.1 節第 7 条、記述のみ）；⑦ 第六節 Q4-（ii）の状態別リストと第八節 Q4、Q5 の軌跡を追加（新規行のみ、「Q4-（ii）状態別リストと Q4、Q5 軌跡（修訂 6 の回）」節）；⑧ `q5_verdict` が L 個の状態中で赤い状態が一つもなく、両方緑の状態もなければ `not_enough_f8` を報告（元は `not_enough`；RefFS 全棄権で f8 の順序優先は不変）。
- 単測 41 条（`grep -c '    def test_' test_segment_one.py test_device_timing.py` で 33、8）、`python3 -B -m unittest test_segment_one test_device_timing` を連続 5 回実行すると `Ran 41 tests … OK`（決定論的な純関数であり、5 回の試行は隠れた状態がないことを示す）。研究側変異表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv` は 37 条（元 17 条に R14–R19 で①–⑥、R20–R33 で⑦ ⑧ を追加）。表全体 37 条をすべて検出、無効 0、生存 0、タイムアウト 0、復旧済みでベースラインも全緑：`E7RESULT name=verdict mutations=37 caught=37 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`（`…-s1-research-mutations-2026-10-07-r3.out`）。同日先に実行した `…-s1-research-mutations-2026-10-07.out`（22 条、22 / 0 / 0）と `…-2026-10-07-r2.out`（23 条、23 / 0 / 0）は R19 および R20–R33 を追加前のものとしてアーカイブ済み。`…-s1-research-mutations.out` は第1段階新樹の回のもの。既存の R4（Q5 全棄権で F8 を通さない）は⑧以降、「RefFS が全棄権かつモデルクラッシュ判官が赤」の入力のみで元の式と異なり、単測がこのセルを固定（F8 は甲より前に配置し、元の順序を維持）。
- マルチホスト実行法のリポジトリへの登録（ドラフトからコピー、元のドラフトは変更しない）: `rows_orchestrator.py` (`7bfbdb84…`)、`peer_side.py` (`27cea9e7…`)、`check_product.py` (`3f237083…`) の 3 つのファイルとドラフト内の逐バイト比較は一致する。`cross_host_o3_check.py` (`afc9a9b0…`、ドラフト版 `04376c00…`) は同一ディレクトリに登録されたオーケストレーターを読み取るように変更され（実行中は引き続き `7bfbdb84…` を検証）、`run_segment_one_on_the_revision_6_tree_multi_host.sh` (`567a8682…`、ドラフト版 `ec89ec78…`) のオーケストレーターと `check_product.py` は倉庫内のこのディレクトリを指すように変更され（オーケストレーターは自身のディレクトリから `peer_side.py` と `` を取得）。凍結されたツリー、ルートエクスポート、ログ、および再開リストは、登録されていない `s2_compare.py`、`s15_compare.py`、`frozen_tree_lines_revision_6.py`、`resume_check.py` とともにドラフト `e171-r1-segment1-rev6/` に残されている。倉庫内のバージョンの自己証明：オーケストレーター最終行「自己証明 34 項目、赤 0」、クロスホスト比較エントリ最終行「自己証明 38 項目、赤 0」。`peer_side.py` と実行スクリプトには `--selftest` が含まれておらず、実行スクリプトは `bash -n` および `detached-run.sh long-run-check` を通過した（長期間実行宣言の 3 つのキーが揃っている）。リビジョン 6 の実際の実行ではドラフト内のいくつかのファイルが使用された（登録上の認定はドラフトの sha256 で記録される）。
- 再実行：`scripts/experiment-apparatus/replay.sh` に 3 行を追加し、すべてこの回のみ現存するエクスポートとプロダクト上で再出力され、装置や cargo を起動しない。

| ドライバ関数 | 生成物 | 実行内容 |
|---|---|---|
| `driver_e171_r1_s1_revision_6_comparator` | `…-s1-comparator-resume2-r6-r3-2026-10-07.out` | 比較器、ルート `scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/` をエクスポート；約 11 秒 |
| `driver_e171_r1_s1_revision_6_cross_host_check` | `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out` | ホスト間比較のエントリポイント `compare`、末尾行を補完 |
| `driver_e171_r1_s1_research_mutations` | `…-s1-research-mutations-2026-10-07-r3.out` | 片側 37 の変異を研究；約 40 秒 |

三つのドライバ関数が `replay.sh` の基準に従ってドラフトディレクトリで実行され、出力と登録産物はバイト単位で同一であり、`name=done emitted=` と E7RESULT の行数が一致している（811、8、78）。異なるマシン間の比較では `-r3` と `-r2` は 2 行目の `entry_sha256=`（エントリ自身の SHA-256、`04376c00…` → `afc9a9b0…`）のみが異なり、判定行はバイト単位で同一である。元々参照されていた `…-comparator-resume2-r6-r3.out` および `…-2026-10-07-r2.out` の 2 つのファイルは保持される。ルートエクスポートは倉庫に導入されないため、最初の 2 行が欠落すると赤判定となる。ステップ 0–13 そのもの（凍結ツリー上の装置各回、行ごとにドライバ、O3（独立仕様実行器）実行器、2 つのマシンのオーケストレーション）は数十時間の掛かり時計を推定し、第 2 マシンが必要であり、登録されない。

#### それに答えられないもの（改訂 6 のその回）

- 判定は、修正 6 のツリー（第1段階の新しいツリー交換装置、第 16 バッチの合流後、ルートループ 10b 以前のその版実装、第 31 バッチのコア断言を含まない）にのみ属する。修正 8 は最終的なメインラインに変更されたため、両者の対照は別途行われる（修正 8 の第 8.5 節に記載）。ここでは推論しない。
- Q1 の甲は「七行がこのバッチの三条流、バッチ 1 上で各行に少なくとも一条の赤がある」とのみ述べており、「共同の盲点が補われた」とは言っていない：表に登録されていない不良な方法と取得されなかった履歴が、依然として三条すべてを通過させる可能性がある。第 23 行はストリーム3上でのみ O2（独立解析器 + checker）によって検出され、第 42 行は 1 回の `unmount` 呼び出し上でのみ O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照によって検出された。ストリーム1、ストリーム2の観点では、第 23 行は依然として三条すべてが通過している。
- Q2 の第1段階には七行の記述しかない；集合と判定は第2段階全体の表（ストリーム1、ストリーム2、281 行に代替実行を加えたもの）から出ている。この集合はまさにこの七行であり、判乙である。第 49、110、162、174、252 行では戻り値またはメモリオブジェクトが異なるにもかかわらず観測結果が同じである：これはモデルに渡されなかったグルーのフィールドの違いによるものであり、O1（メモリモデルの対照（HashMap 理想ファイルシステム））-対照は設計通りに見えない。
- Q3 の第1段階では第 125 行をカウントするか否かを判定したのみである；分母 D = 39 は第2段階全体の表から出ている。c は panic を発生させた流のみを数える（12）；panic を発生させなかったストリーム1も含めると 13 となり、判定は変わらない。
- Q4 の（ii）状態はすべて第 242、269 行上にあり、第二区間ではすべて無効と判定された。ベースライン（修正 6 のツリー上の実装のその版）では（ii）0 状態である。（i）ではそのブランチが候補に進まなかったため判定できない；各バッチで数千の状態がある。三つのバッチとも、クラッシュポイントの再実行で切り出された状態であり、読み取り故障は注入されていない。
- Q4 の逐状態リストと軌跡は、二つの不良な実装（第 242、269 行）上でのみ（ii）状態を持つ。ストリーム3の（ii）状態はすべて inode の構築の外にある：inode の構築順に並べ替えた 235 グリッドはすべて 0 であり、（ii）と葉コンテナの分裂との関係については言及できない；このバッチ上では分裂前後の inode 構築呼び出しで（ii）状態を取得しなかったことのみが述べられている。
- Q5 の主要サンプリングポイント L = 150 994 944 は、類推によって決定されたものである（逐状態で 144 状態を判定）。同等性は k = 6 の 4 608 状態上で逐状態で核査されたのみであり、逆方向のサンプリングポイント 72 状態は逐状態で判定された。警告の自己証明は既に完了しており、両点とも甲と判定された；警告の自己証明は Q5 の判決経路における警告が赤になることを証明するだけであり、同等性関係で核査された範囲を拡大するものではない。
- O3（独立仕様実行器）の一列は v2-487 である；ストリーム3上では厳密な読み取り法により 6 459 状態すべてが「在 / 含まれない」（F7 がトリガーされない）として出力された。七行のうち、第 242、269、270 行のみ O3（独立仕様実行器）で赤となり、残りの四行は O3（独立仕様実行器）において一つの状態も赤にならなかった。
- 第 252、271 行では、凍結ツリー内のその行の記述（`line_matches_the_registration=false`）が実行されており、登録 5.9 の書き込み時の記述とは異なる；それらは名前によって取得され、アンカーはちょうど 1 回であり、代替実行はなかった。

#### 生成物（第 1 回目の再実行の最初のセグメント）

修正 6 のラウンドの生成物（`ls research/results | grep '^e171-r1-' | grep -- '-r6-r3'`、81 ファイル、逐状態リストと軌跡の比較器生成物 `…-comparator-resume2-r6-r3-2026-10-07.out` およびエントリ経路変更後に再作成されたクロスホスト照合生成物 `…-cross-host-check-resume2-r6-r3-2026-10-07-r3.out` を含む）と、研究側変異表の 5 つの生成物（`grep 'research-mutations'`）。`-timing.out` は E7TIMING 行です；`…-rows-probe-driver-r6-r3.out` および `…-rows-probe-driver-resume1-r6-r3.out` はステップ 11 の前の 2 回目の起動で停止した半分の生成物（それぞれ 1 行、判定には含まれない）です；`…-rows-cross-host-check-resume2-r6-r3.out` で `false` と判定されたものは無効とし（「有効な条件と Q5（修正 6 のラウンド）」セクションの S17 に記載されている通り）、判定には含めません：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-run1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-streams-1-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o3-p1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-row-0-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-row-0-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-10-run-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-10-run-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-6-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-5-step-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-5-step-7-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-6-step-7-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-research-mutations.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations.out
```

修正前の数本のツリーにおける第1段階の生成物（85 件、`ls research/results | grep '^e171-r1-' | grep -v -- '-r6-r3' | grep -v 'research-mutations'`）を再確認する：`2026-10-04` で始まるものは修正 1 のツリーであり、`2026-10-05` にサフィックスを持たないものは第1段階の新しいツリー（`4d071c12…`、ストリーム三における第 7 歩 a1 は成立せず、修正 6 第 6.1 節参照）、`revision-5-q5-` は Q5 ツリーであり、`-r6` および `-r6-r2` は修正 6 のツリーで最初の 2 回の実行が第 0 歩に停止した記録である。これらはいずれも参照として保持し、いかなる判定にも付与しない（修正 6 第 6.1 節末尾の表および「主担当エージェント 認定：修正 6 の起動前倉庫内の装置と記述が仕様から逸脱している」第 3 条・第 5 条参照）：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-comparator-streams-1-2-before-the-rows.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-o3-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-memory-object.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-comparison.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-comparison-unmount.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-crash-judge-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o3-p1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-panic.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-reference-filesystem-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-return-value.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-witness.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-q5-main-before-the-first-barrier-of-call-3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-q5-reverse-at-the-return-point-of-call-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2-device-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2-e170-export-dump.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-stream-constants.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o3-p1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-1-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-reverse.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-reverse-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-on-the-draft-copy-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-timing.out
```

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:856-956`（整段抄，未转述）**

````markdown
### 第 1 回目の再実行第2段階・改訂 6 の回（全テーブル実行済み：Q2 イ、Q3 分母 D = 39）

`research/prompts/e171-r1-prereg.md` の第 5 節 5.5 項の第2段階、第 6 節 Q2-集合と Q3-分母の 2 行、第 1 節の読み方 4 と読み方 5、改訂 5 の第 5.1 節「既存産物の読み方」の最終条（第2段階は O1（メモリモデルの対照（HashMap 理想ファイルシステム））のみを読み、対照と呼び出しについては、2 つのクラッシュサブシーケンスはこの行に含まれない）、改訂 6 の第 6.6 節「第2段階（表全体）を原文のまま、改訂 6 ツリー上で実行する」、最終節「主担当エージェントが認定：第2段階の表全体はマルチマシン実行法で提出」。ツリーは第1段階と同じ（スナップショットサマリー `7726cdd6…`）。281 行目。ストリーム 1、ストリーム 2。バッチ 1、バッチ 2。仕様実行器を実行せず、プローブを付与しない。各行の時間制限は 1800 秒（T = 75.274 秒、3T は 1800 を超えない）。2 つのマシンで完了し、タイムアウトした行は 0 行、パッケージされた結果の行は 0 行。クロスマシンの一貫性照合では第 201 行目で一致が判定された。

Q2 の「ベースラインと異なる」は、改訂 6 の第 1 ステップ S2 新しい装置のベースライン（`--streams 1,2`、プローブなし、`scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/s2-device-baseline-streams-1-2/` にエクスポート）と比較するものである：改訂 6 の第 6.1 節末尾の表は、Q2 の集合を第 1 ステップのこのラウンドと比較すると述べており（古い装置のものはルートリングテーブルを持ってこられないため、比較結果の各行がすべて「メモリオブジェクトが異なる」となる）、改訂 2 の第 2.5 節第2段階の T もこれに由来する。読み方 4「ベースラインを 2 回実行して異なるオブジェクトを除外」：第 1 ステップは 1 回だけ実行されたが、コンパレータは別の同一ツリー、同一装置の第 7、9 ステップの 2 回の三流ベースライン（`baseline-run1/`、`baseline-run2/`）を用いて、ストリーム 1 とストリーム 2 のみを比較し、8 つのファイル（書き込みストリーム 4 ファイル、`states.tsv`、`oracle-verdicts.tsv`、`call-outcomes.tsv`、`call-return-values.tsv`）はすべてバイト単位で同一であり、不安定なオブジェクトは 0 個である。

| 行 | 判定（仕様実行器 6 樹） | 根拠（生成物） | 文言以外に一緒に読む必要があるもの |
|---|---|---|---|
| q2 | 乙：集合 7 行、O1（メモリモデルの対照（HashMap 理想ファイルシステム））- 赤判定の 2 行（第 155 行、161 行）、赤判定でない 5 行（第 49 行、110 行、162 行、174 行、252 行） | `…-2026-10-07-s2-comparator-r6.out` の `name=q2`（`set_rows=7 o1_red_rows=2 … verdict=yi`） | 集合は正確に C672（戻り値またはメモリ状態が誤って書き込みストリームが変わらない一类）で指定された 7 行であり、テーブルの他の 227 行には未到達行が含まれ、その戻り値とメモリオブジェクトはすべてベースラインと同じである。53 回の異なる呼び出しにおいて、グルーコードがモデルに渡す観測も異なるのは第 155 行、161 行で F を上げるその一回（`ceiling`）のみであり、O1（メモリモデルの対照（HashMap 理想ファイルシステム））- 対照はこの 2 回で `disagrees` する；残りの 51 回の観測は同じである：異なるのはモデルが見えないフィールド（`writes`、`output`、`current`、`publishes`、`allocator`、`system_configuration_writes_before_the_first_publish`）とメモリオブジェクト（`mount_output`、`current_version`、`allocator`）であり、O1（メモリモデルの対照（HashMap 理想ファイルシステム））- 対照はこれらについてすべて `agrees` する。7 行の逐次呼び出しと最初のセグメントにおける三流帯プローブでの計算による文字通りの一致 |
| Q3 分母 | D = 39（1 回目の再実行読み取り法 5、バッチ 1 で確定到達） | `name=q3_denominator` 4 行 | 元の登録読み取り法 5（呼び出しの panic と状態上の回復 panic は一律に自己崩壊）D = 38、違いは第 125 行；バッチ 2 で確定到達 D = 35（元の登録読み取り法 5 は 34）、違いは第 27 行、112 行、233 行、277 行（ジャーナルの回復報告においてベースラインと異なるのはバッチ 1 のある状態でのみ）。分母に含まれない：未到達 234 行、自己崩壊 6 行（第 6 行、7 行、85 行、95 行、235 行、272 行）、見つからない 2 行（第 107 行、168 行）；コンパイルに失敗した 5 行はすべて補完され、タイムアウト 0 行。第 125 行は登録アルゴリズムの両バッチで計上されるが、元の登録アルゴリズムの両バッチでは計上されない |

判決行と Q2、Q3 の各行（行全体は `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out` から抄録。第 6、7、349、410、411、414、415、416、421、426、431–441、447–450 行）：

```
E7RESULT name=unstable_memory_objects stability_baselines=2 names=-
E7RESULT name=driver_product row_time_limit_seconds=1800 snapshot_digest=7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4 rows=281 table_rows=281
E7RESULT name=q2_observation set_rows=7 calls_with_a_difference=53 calls_whose_observation_differs=2 calls_whose_observation_is_the_same=51
E7RESULT name=q2 set_rows=7 o1_red_rows=2 rows_o1_red=155,161 rows_o1_not_red=49,110,162,174,252 verdict=yi
E7RESULT name=q3_denominator reading=registration batch=1 rows=281 counted_D=39 counted=39 untouched=234 self_crash=6 not_found=2
E7RESULT name=q3_denominator_class reading=registration batch=1 class=self_crash rows=6 row_numbers=6,7,85,95,235,272
E7RESULT name=q3_denominator_class reading=registration batch=1 class=not_found rows=2 row_numbers=107,168
E7RESULT name=q3_denominator reading=registration batch=2 rows=281 counted_D=35 counted=35 untouched=238 self_crash=6 not_found=2
E7RESULT name=q3_denominator reading=original_reading_5 batch=1 rows=281 counted_D=38 counted=38 untouched=234 self_crash=7 not_found=2
E7RESULT name=q3_denominator reading=original_reading_5 batch=2 rows=281 counted_D=34 counted=34 untouched=238 self_crash=7 not_found=2
E7RESULT name=q3_self_crash_row row=6 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=7 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=85 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=95 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=125 where=fixed_script_through_e_then_unmount:call_14:judged_12 registration=counted original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=235 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=272 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_row_125 row=125 registration_batch_1=counted counted_registration_batch_1=true registration_batch_2=counted counted_registration_batch_2=true original_reading_5_batch_1=self_crash counted_original_reading_5_batch_1=false original_reading_5_batch_2=self_crash counted_original_reading_5_batch_2=false
E7RESULT name=recovery_panic_sum rows=0 states=0 by_row=-
E7RESULT name=not_located_row row=107 status=not_found failed_attempts=1 substituted_from=- name_hits_in_the_frozen_table=0
E7RESULT name=not_located_row row=168 status=not_found failed_attempts=1 substituted_from=- name_hits_in_the_frozen_table=0
E7RESULT name=untouched_row_red row=155 O1-对拍=1 O1-模型崩溃判官=0 O1-RefFS=0 O2=0
E7RESULT name=untouched_row_red row=161 O1-对拍=1 O1-模型崩溃判官=0 O1-RefFS=0 O2=0
E7RESULT name=driver_product_rows rows_in_the_product=281 rows_in_the_root=281 rows_whose_status_or_exit_differs=-
E7RESULT name=verdict q2=yi q2_set_rows=7 q2_o1_red_rows=2 d_registration_batch_1=39 d_registration_batch_2=35 d_original_reading_5_batch_1=38 d_original_reading_5_batch_2=34 row_125_counted_registration_batch_1=true rows_read=281 table_rows=281 rows_not_located=107,168 unstable_memory_objects=0 stability_baseline_files_differing=0 rows_compiled_without_the_rebuild_check=0 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 untouched_rows_with_a_red_column=2 rows_whose_directory_differs_from_the_driver_product=0
```

集合の7行で呼び出しごとの差異がどこにあるか（リポジトリのルートで `grep 'name=q2_set_call' research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out | sed -E 's/.* row=([0-9]+) .* fields=([^ ]*) objects=([^ ]*) observation_differs=([a-z]*) .*/\1 \2 \3 \4/' | sort -k1,1n -k2 | uniq -c`を実行する。列は順に、出現回数、行、返却値が異なるトップレベルフィールド、異なるメモリオブジェクト、観測が同一か否か）そのままの出力：

```
     10 49 - mount_output false
      1 49 output mount_output false
      1 110 #0 current_version,mount_output false
      1 110 current,output current_version,mount_output false
      1 110 - current_version,mount_output false
      1 110 publishes current_version,mount_output false
      3 110 writes current_version false
      7 110 writes current_version,mount_output false
      1 155 ceiling - true
      1 161 ceiling - true
     10 162 - mount_output false
      1 162 output mount_output false
      1 174 system_configuration_writes_before_the_first_publish - false
      1 252 allocator allocator false
     13 252 - allocator false
```

- 二つの読み取り方法における自己崩壊の行：第 6、7、95 行は、2 つのストリームの最初のファイルでのリリース（2 回目の呼び出し）で panic し、それ以前には番号取得とウォームアップの 2 回のみ `not_modelled` であり、0 回と判定できる。第 85、235、272 行は、1 回目の呼び出し以前に panic（システム実行構成、システム不変構成が書き出したバイト数とコードヘッダーのオフセットおよびフィールドテーブルが一致しないというアサーション；mkfs の段階であると推測され、スタックトレースを確認していない）。2 つのストリームともエクスポートされておらず、1 回の呼び出しも発生していない（装置 `name=stream … exported=false not_exported_why=pipeline_panicked:… calls=0`）。読み取り方法 5 で狭められた原文は「最初の panic 以前に判定可能な呼び出しが 0」であり、1 回目の呼び出し以前に panic のこの 3 行についても判定可能な呼び出しは同様に 0 である。コンパレーターは自己崩壊を記録する（元のコンパレーターも `pipeline_panicked` を自己崩壊として記録）。この箇所での読み取り方法は主エージェントによる確認が必要であり、別の読み取り方法に切り替えてもこれらの行は分母に含まれない（エクスポート可能な比較対象がなく、到達しても判定できないため）。
- 見つからない第 107、168 行：2 つの行の変異名は、凍結ツリーの `crates/mutations.tsv` で 0 回ヒットしている（記述後に名前が変更されているため）。ドライバーによる補完は、「この名前がこのファイルグループ内の今日の位置」から下へ探す必要がある。名前がグループ内に存在しない場合、起点が見つからないため、補完できないと記録する（`failed_attempts=1`、`substituted_from=-`）。登録表 5.2「補完不能な場合はそのまま報告」に従い、分母には含めない。登録表中のグループ内インデックスから下へ探すという別の読み取り方法は主エージェントによる確認を要する；その場合でも最大で 2 行程度であり、D は 39–41 の範囲となり、Q2 の乙は反転できない（集合には赤い行も不赤い行も既に含まれているため）。
- コンパイルに失敗して補完した 5 行（第 33、59、127、163、206 行；第 127 行は 2 回変更）はすべて、代替実行規則に従って同グループの次の行に変更される。変更された行はいずれも到達していない（`name=substitution` の 5 行と `…-s2-rows-substitution-coordination-r6.out` の判定行 `substitute_collisions=0`）。
- 検証：7 行は逐次的な呼び出しにおいて異なる（行、ストリーム、呼び出し、フィールド、オブジェクト、観測の一致・不一致）とあり、第1段階のコンパレーターが 3 つのストリームにプローブを付けた回で計算した `name=q2_named_row_call` のストリーム 1 とストリーム 2 の部分は文字通り同一である（各 53 行を取り出し、第2段階に追加されたタグおよび O1（メモリモデルの一致（HashMap 理想ファイルシステム））- 一致の 2 カラムを除去し、ソート後に `cmp` で出力なし）。両者は同じ比較関数を使用しており、独立しているのは 2 つの装置実行と 2 つのベースラインのみである。
- 無効化と停止：Z6 の `rows_compiled_without_the_rebuild_check=0`；S5 のコンパレーター側では両方の数が 0 である。2 バッチともジャーナルの確認による回復で panic 状態は 0（改訂 5 の第 5.1 節の数え方により、第2段階を 2 つの崩壊サブ列として読み取らないため、数値は変わらない）；Z3 の包装結果とタイムアウトは 0 行（合成プロダクト `check_product.py rows_without_executor` で `timed_out=none wrapper_outcomes=none` と記録）；Z5 の合成プロダクト第 2 行のスナップショットサマリー `7726cdd6…`。

コンパレータ、単体テスト、変異と再実行:

- 比較器 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_two_comparator.py` を別途作成し、第1段階の比較器を変更しない（第1段階の判定、単一テスト、および変異表は変更を許さない；第2段階の入力はベースライン、各行の出力ルート、合成生成物であり、第1段階とは異なる）。到達可能か否か、仕様実行器による値差、判定可能な呼び出し回数、自己崩壊、S5 の 2 つの数値については `collusion_matrix.py` と同様にする。追加するのは、2 つの決定論的に到達した状態のバッチ（そのバッチの状態のみを残し、バッチ 1 の記録を修正して同じ到達可能関数に渡す）、2 つの読み方の分類と集計である。この実行回のエクスポートルート（`revision-6-2026-10-05-r3/` と `revision-6-segment-2/`）および生成物プレフィックス `…-2026-10-07-s2-`、サフィックス `-r6.out` のみを読み込む；その他は 2 つロールバックする。全体テーブルの所要時間は約 16 秒である。
- 単一テスト `test_segment_two.py`（17 条、`grep -c '    def test_' test_segment_two.py` で確認）では、手動でフローと一時ディレクトリを作成し、各カテゴリについて少なくとも 1 つの判定を確保する：バッチ 1 とバッチ 2 の決定論的な到達状態、ドライバと装置の結果の各種類、自己崩壊と到達可能性の順序、2 つの読み方、mkfs ステップでのパニック、分母と各行ごとの分類、Q2 セット（到達可能な行に達しない行、バッチ 1 だけで到達しない行、不安定なオブジェクトを除外）、Q2 の甲乙丙と F4、エクスポートルート、さらに 8 行の全体テーブルをディスクに書き込み `main` を実行する追加テストを加える。連続して 5 回実行すると `Ran 17 tests … OK` が得られる（決定論的な純関数であり、5 回の実行は隠れた状態がないことを示す）。
- 片側の変異表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_segment_two.tsv`（28 条、W1–W28）を、同ディレクトリの `mutation_driver.py` に `--unit-test-module test_segment_two` オプション付きで実行する（ドライバはこのオプションを追加した；指定しない場合は従来通り第1段階の 2 つのモジュールを実行：第1段階の変異表を再実行し、`…-s1-research-mutations-2026-10-07-r3.out` とバイト単位で同一であることを確認）：`E7RESULT name=verdict mutations=28 caught=28 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`（出力は `…-2026-10-07-s2-research-mutations-r6.out`）。検出数 / 無効 / 赤色未フラグ = 28 / 0 / 0。連続して 5 回実行すると、出力がバイト単位で同一になる。最初の表の 37 条のアンカーポイントは、新しいファイルを追加した後でもそれぞれ正確に 1 回ずつ命中する。
- リプレイ：`scripts/experiment-apparatus/replay.sh` に 2 行追加し、既存のエクスポートと生成物上でのみ再作成を行い、装置や cargo を起動しないようにする：

| ドライバ関数 | 生成物 | 実行内容 |
|---|---|---|
| `driver_e171_r1_s2_revision_6_comparator` | `…-2026-10-07-s2-comparator-r6.out` | 第2段階比較器；改訂 6 の走査における第 1、7、9 ステップの出力を `revision-6-segment-2/segment2-rows/` と読み込み、ローカルに保持し倉庫には投入せず、存在しない場合は赤判定とする |
| `driver_e171_r1_s2_research_mutations` | `…-2026-10-07-s2-research-mutations-r6.out` | 片側 28 の変異を研究し、約 40 秒で完了する |

二つのドライバ関数が `replay.sh` の口径に従ってドラフトディレクトリで実行され、出力は登録産物とバイト単位で一致する。全体テーブル自体（281 行の行ごとに改悪、編成、装置実行、2 台のマシン）は登録されない。

第2段階の産物（`ls research/results | grep '2026-10-07-s2-'`、8 件）：合成産物 `…-s2-rows-driver-r6.out`（565 行：281 行 `row`、281 行 `merged_shard` および先頭と末尾の数行）とその `-timing.out`；代替実行調整 `…-s2-rows-substitution-coordination-r6.out`；ホスト間整合性照合 `…-s2-rows-cross-host-row-201-driver-r6.out`、その `-timing.out` および判定 `…-s2-rows-cross-host-check-r6.out`（`E7RESULT name=verdict cross_host_identical=true row=201 host_of_rows_step=2 host_of_cross_host_run=1 parts=4 compared_parts=3 differing_parts=none`、o3 の 1 グリッド `not_run`）；比較器 `…-s2-comparator-r6.out`；研究側変異 `…-s2-research-mutations-r6.out`：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-check-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-row-201-driver-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-row-201-driver-r6-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-substitution-coordination-r6.out
```

#### 答えられないもの（2 段落目）

- Q2 の乙は、「ストリーム一およびストリーム二の履歴において、到達せずかつ戻り値またはメモリオブジェクトが異なる 7 行のうち、O1（メモリモデル対照（HashMap 理想ファイルシステム））-対照赤 2 行」のみ述べた：赤ではない 5 行は、グルーがモデルに渡さないフィールドとオブジェクトにおいて差異しており、O1（メモリモデル対照（HashMap 理想ファイルシステム））-対照は設計上それらを見ることができない；乙はこの 5 行の状態が合法かどうかを述べておらず、また他の履歴における集合の大きさを述べていない。
- D = 39 は記述量であり、Q3 の甲乙は第1段階第 125 行で判定される（c = 12）；第2段階は第1段階のいかなる判定も変更しない。D のいくつかの口径の違いは、読み方 5 の自己崩壊の類別と定到達の類別にあり、4 つの数値をすべて記載する。
- 判定は改訂 6 ツリーにのみ属する；改訂 8 のそのラウンド（最終メインライン）の第2段階がどのツリー上で実行されるかは別判定となる（「主エージェント認定：改訂 8 を返却」第 8 条参照）。
- 第 85、235、272 行および第 107、168 行の分類は 2 つの読み方（「第 1 回第2段階再実行・改訂 6 のそのラウンド」節内の「2 つの読み方下での自己崩壊のいくつかの行」と「見つからない第 107、168 行」の 2 項目）であり、主エージェントによる認定を待っている；2 つの読み方のいずれも Q2 を裏返すことを許さない。

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:957-1026`（整段抄，未转述）**

````markdown
### 第 1 回再実行・改訂 12：ストリーム三段長再釘（2026-10-13）

`research/prompts/e171-r1-prereg.md` の改訂 12（設計員による記述）と改訂 13（実行員による記述、読み取りのみ）を登録する；問題票 `e171-r1-s3-segment-lengths-questions.md` の第 1 行および第 3 行（第 2 行の状態は主エージェントが変更）。この節は装置内の定数 `LOCAL_THIRD_STREAM_SEGMENT_LENGTHS` の 2 つの数にのみ属する；Q1–Q5 の判定、数、ならびに改訂 6 ツリーの産物はすべて変更されていない。

1. **なぜ再釘打つのか**：2026-10-09 に退役樹の判決が実装された後、今日の crates がストリーム三で最初のファイルを削除する際に、2 つの退役ユニットを余分に書き込む（`crates/singlefs-core/src/transaction.rs` 内の `retirement_records_of_this_publish` の箇所）。装置内で 2026-10-05 に釘打ったセグメント長テーブルと、今日の crates で現在計算される値が 2 か所で一致しない。再釘打つ前の回りの生成物（生成物甲、`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12-before-the-repin.out`）の 82 行目の判決行において `s1_failed_checks` は 1（`grep -n 'name=verdict'` で取得）。

セグメント長行（第 72 行、`holds=false`）は 4KB を超えるため、一行全体をコピーしない。登録された第 12.14 節の比較命令は、生成物甲のそのまま出力に対して適用される：

   ```
   lengths 959 959 sums 5980 5980 closed 362808742 362808742
   differing [(15, 20, 24), (43, 24, 20)]
   permutation True
   index_11_15 local 26 20 crates 26 24
   local_sha256 43d5bd3959c56382bc4bf220e35f964fda89b9d49af702d869bc31048aa6c5ce
   crates_sha256 fcec3e5e21df15d48891fc12d6dd366c282fff7306e58bc2144aac6bee2ac66c
   holds false
   ```

2. **新旧対照**: 下標 15: 20 → 24, 下標 43: 24 → 20; 書き込み数 5980、全量状態数 362 809 947、高速階層 3127、閉式 362 808 742 は不変（乙里に対応する `…_writes`、`…_full_states`、`…_quick_tier_states`、`…_two_state_closed_form` の四行の `local` と `crates` の二つの欄が同一であり、`holds=true`）。

3. **逐条判定**（乙 = 重釘後最初のパス、丙 = 次のパス）：

| 量 | 読み取り値 | 判定 |
|---|---|---|
| RS1 | 乙で `grep -c 'holds=false'` が 0（`grep -c 'holds=true'` は 77、S1 全体で 77 行）；`grep -n 'name=verdict'` で `82:E7RESULT name=verdict s1_failed_checks=0` を得た；`cargo run` が 0 で終了（主担当エージェント のバックグラウンドチェーンからの出力に基づく、仕様実行器は当該出力ファイルを見ることができず、配布されたヒントが伝達した） | 全通：三項目すべてが閾値に到達 |
| RS2 | 甲と乙で段長行および `name=verdict` 行を除いた後の `diff` の行数が 0；全体の `diff 甲 乙` では第 72 行、82 行の二箇所のみ異なる | 他の項目はすべて変更なし（三つのストリームの割り当て器に mkfs を含むルートリングテーブルの三行は両方同じ） |
| RS3 | 乙の段長行において `local` と `crates` の二カラムが `differing []`、二カラムのサマリーともに `fcec3e5e…`；ドラフト表と甲の `crates=` が項目ごとに一致（前段の仕様実行者がドラフト上で確認済み） | 判別基準ではない、単に写し取りが正しい：写し取り正解 |
| RS4 | 甲・乙の段長行において `crates=` のカラムサマリーともに `fcec3e5e21df15d48891fc12d6dd366c282fff7306e58bc2144aac6bee2ac66c` であり、登録された新しい釘値サマリーと一致する | 書き込み側と新しい釘値が項目ごとに一致 |
| RS5 | 甲：`holds=false` は正確に第 72 行一行；異なるインデックスは正確に 15、43 で、(local, crates) が (20, 24)、(24, 20)；`s1_failed_checks=1`；終了コード 3（前段の仕様実行者の読み取り値、メモリ包装およびスレッド上限から） | 陽性対照は赤判定が呼名箇所にあり |
| RS6 | 乙と丙で `sha256sum` を計算し、両行ともに `aef90046e0b565776288e5051c7267f706db4ee718bfceeb66dfb0272dfdd2a1` | 二回ともバイト単位で同一（決定論的であり、隠れた状態がないことを示すのみ） |
| RS7 | 「退役記録を書かない」側 | 登録済み装置では不可能、計測なし |
| RS8 | 変更前・変更後それぞれ一度 `grep -c 'stream-constants-r6-r[23]' scripts/experiment-apparatus/replay.sh`：0、0 | 両方とも再実行行がないため、第 7 条「2026-10-05 の二つの旧生成物」に従う |

RS2 のコマンドと出力（`<甲>`、`<乙>` は第 4 条の 2 つのファイルです）：

   ```
   $ diff <(grep -v -e 'check=first_file_unlinked_then_inodes_across_one_leaf_segment_lengths' -e 'name=verdict' <甲>) <(grep -v -e 'check=first_file_unlinked_then_inodes_across_one_leaf_segment_lengths' -e 'name=verdict' <乙>) | wc -l
   0
   ```

RA1：甲の `crates=` の下標 11 は 26、下標 15 は 24（旧釘値 20 に 4 を加えたもの）、条項と実装は一致し、F10 はトリガーされていない。RA2：959 個の数、和 5980、1 + Σ(2^n − 1) = 362,808,742（甲 `lengths 959 959 sums 5980 5980 closed 362808742 362808742`）。RA3：新旧の 2 つの表をソートした結果は同一であり、異なる下標は恰も 15、43（甲 `permutation True`、`differing [(15, 20, 24), (43, 24, 20)]`）。Z10、S19 のトリガー観測は 1 つも出現していない（RS5 は閾値に達せず、RS2 は 0、RS4 は甲乙同一、RS6 は同一、乙 `grep -c '^E7RESULT '` が 83 で `name=done emitted=83` に等しい、メモリ包装は 245 または 250–254 に退避していない）。

幾何感度に関する二点：ストリーム三は変化するが、ストリーム一とストリーム二是不変である。乙と 2026-10-05 の `…-s1-stream-constants-r6-r2.out` を行ごとに比較し、`diff … | grep -E '^[0-9]'` のそのまま出力は `33c33`、`49c49`、`65c65`、`72c72` である：異なる行はちょうど第 33 行（三つのルートリング表サマリー、両方の列ともそれぞれのツリーで現在計算されたもの）、第 49 行、第 65 行および第 72 行（ストリーム三段の長さ）である；ストリーム一とストリーム二のセグメント長、書き込み数、ルートスロット書き込み数、全量 / 高速档 / 閉式に関する各行はバイト単位で完全に一致する。

装置内の `crates/mutations.tsv` でその 47 行が点名され、アンカーによる照合で出力は `rows 47`、`count!=1 []`、`in 200-270 []`（コマンドは登録第 12.14 節参照）となる。

4. **生成物**（すべて `research/results/` 下にある）：
   - 甲：`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12-before-the-repin.out`、83 行、sha256 `dcf5b558bed1a379437b91efb716a67fc15e5dbe5ac01ac6d4b10c7277137c9c`、退 3。仕様実行器を交換した後に再実行しても出力が得られないため、再実行の結果は登録せず、ここでは単に名称のみを記録する。
   - 乙：`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12.out`、83 行、sha256 `aef90046e0b565776288e5051c7267f706db4ee718bfceeb66dfb0272dfdd2a1`、退 0。
   - 丙：リポジトリに保存しない、sha256 は乙と同じ。
   - 仕様実行器を交換した後の 3 つの sha256：変更前 `c556d42d…`、ドラフトおよび交換後とも `6aa1bb82…`（`compile-then-swap.py` で退 0、交換した変更とドラフトの diff はファイル名行を除いて一致）；現在、主作業領域に対して一時保存領域には未一時保存のこのブロックが存在しており、これをどのように一時保存して主担当エージェント に帰属させるか。

5. **再実行**：一行は `scripts/experiment-apparatus/replay.sh` に登録されており、`E7RESULT` 行と exact 比のみを残す；全体の `replay.sh` は重量級であり実行されていないため、乙はこのパイプラインが今日の倉における出力である。

| ドライバ関数 | 生成物 | 実行内容 |
|---|---|---|
| `driver_e171_r1_revision_12_stream_constants` | `…-2026-10-13-s1-stream-constants-revision-12.out` | 装置 `stream-constants --streams 1,2,3` を実行し、約 2 分間かかる |

6. **問題票の第2行**：「書く」側は RS4 で判定済みである。現在の crates（もともと退役レコードを書き込む）が算出した区間長表は、新たに固定した値と項目ごとに一致する。「書かない」側はリポジトリに登録された装置では実行できない。3つの方法はそれぞれ次の理由で使えない。装置自身に切り替えを設けても、退役レコードは core が各公開時に無条件で追記し、装置が操作できるのは公開インターフェースからの履歴変更（別のストリームになる）か事後の書き換えだけで、後者では割り当て位置を元に戻せない。f8a9ddaa より前の退役ツリーがないツリーも、その時点の装置を含むコミットがリポジトリになく、現在との差が22ファイルに及ぶため、退役レコードだけを分離できない。コピー上での反事実は手がかりにとどまる。core のソースを変更する必要があり、登録済み装置の範囲外だからである。

7. **2026-10-05 の 2 つの旧産物**：`…-2026-10-05-s1-stream-constants-r6-r2.out`、`-r6-r3.out`（バイト単位で同一）は、改訂 6 ツリー上の読み取り値であり、`crates=` はそのツリーの現在の計算値であるが、今日の実装の釘値および今日の crates の現在の計算値とは異なる。これらは改訂 6 のその回としての記録として保持され、現在値としては扱われない。再実行表には本来これらの行は存在せず（RS8 変更前・後ともに 0 である）。`…-s1-stream-constants-on-the-draft-copy-r6.out` も同様に保持する。このページの前面で第 1 回目に再実行した第1段階においてそれらを引用している文は、改訂 6 のその回当時の事柄を記述しており、変更されていない。

#### 答えられないもの（改訂 12）

「互換が退役記録によって引き起こされた」という記述は、調査員のコピー上でのみ確認されている：core 内の退役記録を空リストに置き換えると、セグメント長テーブルの値は旧固定値と各項で一致する。コピー上の数値は kb に取り込まれず、単なる手がかり（`records/2026-10-11-崩溃放量两段与脚本审计.md` の 2026-10-13 のエントリが調査員報告を指している）として扱われるのみで、登録済み装置上では再現できない。
- 下標 43 がなぜ同様に変化するか（分配記録ツリーのリフの葉から槽を押し出す、または配置位置の後に槽をいくつか押し出すことによる）も、コピー上の出力からしか得られず、推測に基づくものである。RA1 は下標 11 および 15 の条項の推論のみを担当し、43 を担当しない。
- 改訂 6 のツリーにおいて `crates=` が旧固定値と等しいことは、両者が互換性を持つことを示すに過ぎない：そのツリーは今日のものとの違いが退役記録だけではない（改訂 8 の提出準備時に数えられたファイル数は 140 ファイルであり）、また互換の帰因については言及されていない。
- 装置のインライン使用例は、セグメント長テーブルの和および閉式と比較するだけであり、順序には不敏感であるため、下標 15 と 43 の交換を検知することはできない；検出できるのは `stream-constants` が実際にストリームを 3 回実行した点のみであり（したがって `crates/mutations.tsv` に変異行を追加していない）、セグメント長テーブルにおけるこの変異の交換は、産物甲が負担する。
- 反事実的記述（「互換が退役記録によって引き起こされた」というもの）は手がかりであり、測定値ではない：それは登録済み装置上で再現されず、いかなる判定にも取り込まれない。

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:1027-1043`（整段抄，未转述）**

```markdown
### パスと結論の登録

| # | パス | 計算方法 | ソースコードの位置 | 参照する共有項目 |
|---|---|---|---|---|
| 1 | バッチ 1 の状態数（装置） | サンプル抽出の三つのバッチの公式をクラッシュポイントで再帰的に列挙表に与えられた各区間番号に適用し、さらにテアスキャンを加える；列挙点は全量ではない | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1359` `sample_ordinals` | `crash_state_enumeration_tables` の区間番号とどの回書き込みが三状態を取得したか（`crates/singlefs-checker-tier/src/crash.rs` 参照） |
| 2 | バッチ 1 の状態数（コンパレータによる再計算） | `ops.tsv` をセグメント分割ルールに従って分割し、`torn.tsv` の書き込み番号を三状態の書き込みとして扱う；各セグメントの状態数は、各書き込みが取得した状態数の積から 1 を引いた値とし、その後同一のサンプル抽出公式を用いて再計算する | `scripts/experiment-apparatus/e171_three_oracle_collusion/baseline_checks.py:175` `recompute_first_batch` | サンプル抽出公式（E170（O3 仕様実行器第一版）登録 5.4 の文字）；`torn.tsv` と `ops.tsv`（装置が crates の列挙表と書き込み表から出力）：両パスは「どの回書き込みが三状態を取得したか」という点で共有している |
| 3 | O1（メモリモデルの対照（HashMap 理想ファイルシステム））- クラッシュ判官に適用可能な状態数（装置） | 一つの状態で永続化またはテアされた書き込みが属する最後の呼び出しが、モデルが停止した呼び出しより早くない場合、その状態は適用できない | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1937` `crash_judge_is_applicable` | 呼び出し表の戻り点と書き込み表のストリームインデックス |
| 4 | O1（メモリモデルの対照（HashMap 理想ファイルシステム））- クラッシュ判官に適用可能な状態数（コンパレータによる再計算） | `calls.tsv`、`ops.tsv`、`states.tsv` の着地ベクトル数を `unmount` という呼び出しに含まれる状態として書き込む | `scripts/experiment-apparatus/e171_three_oracle_collusion/baseline_checks.py:184` `recompute_crash_judge_applicable` | `calls.tsv` の戻り点（装置が出力）；モデルは `unmount` へのこの読み取りのみで停止する（登録第 1 節の読み取り法 6 参照） |
| 5 | O2（独立解析器 + チェッカー）における不良状態行上の判定（装置） | ニュートラル形式から着地ベクトルを再構築して `MemoryPool` を作成し、E170（O3 仕様実行器第一版）が記述したパッチを重ねて `check_pool_image` を呼び出す | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:4751` `e170_bad_rows_on_both_streams` | プールレベルのチェッカー自体、E170（O3 仕様実行器第一版）が記述した `bad-patches.bin` と着地ベクトル |
| 6 | O2（独立解析器 + チェッカー）における不良状態行上の判定（E170（O3 仕様実行器第一版）エクスポート装置） | `CrashImage` 上でパッチを書き込み表の末尾に接続し、永続的なシステム設定スロットとして書き込み、`judge_crash_image` を呼び出してプールレベルのチェッカー項目を取得する | `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:3014` `observe_bad_row` | プールレベルのチェッカー自体；同一バッチのパッチバイトと着地ベクトル |
| 7 | 1 回目の再実行：O1（メモリモデルの対照（HashMap 理想ファイルシステム））- モデルクラッシュ判官が適用可能な状態数と RefFS の投票する状態数（装置） | 適用条件：1 つの状態に落ちた書き込みが属する最後の呼び出しは、モデルが停止した呼び出しより早期であること；投票条件：RefFS の翻訳が、その状態でのクラッシュ時刻以前に停止していないこと | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3888` `crash_judge_is_applicable`；同ファイルの `:3172` `reference_filesystem_vote_of_a_state`（行番号は倉庫内の `ab35105f…` 版のものであり、改訂 6 のツリー版では第 3475 行以降それぞれ 7 行少ない） | 呼び出し表の戻り点と書き込み表のストリーム順序インデックス；`reference_filesystem_translation.rs` の翻訳停止点 |
| 8 | 1 回目の再実行：O1（メモリモデルの対照（HashMap 理想ファイルシステム））- モデルクラッシュ判官が適用可能な状態数と RefFS の投票する状態数（コンパレータによる現算） | `calls.tsv`、`ops.tsv`、`states.tsv` から現算：各書き込みがどの呼び出しに落ちたか、最後に落ちた書き込みが属する呼び出しがモデル停止の呼び出しより早期であるか；クラッシュ時刻は当該区間を閉じるステップとし、`unlink` の数ステップ前または不成功なロールバック以前に投票を行う；3 つのストリームの全状態を列挙し、列挙点はスキャン全体ではない | `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py:176` `recomputed_applicable_and_voting` | 装置が出力する `calls.tsv`（最初の op、戻り点）、`ops.tsv`、`states.tsv` の着地点列とセグメント番号、`stream-status.tsv` の `model_stopped_at_call`；セグメント分割規則のコンパレータは `SegmentClosingRule` に従って独自に作成する |

パス 7 と 8 は、第 1 回の再実行によるコンパレータの生成物において逐流比（`…-s1-comparator-resume2-r6-r3.out` の `name=a11` の三行すべてが `holds=true`）となり、同様に二つのパスを直接つなぐ `assert` も存在しない；両パスは共有装置によって書き出された呼び出し表と着地列であり、それらは二つとも誤っている。

パス 1 と 2、3 と 4 は出力結果内でセルごとに比較（`name=s5_batch_1_states`、`name=s5_o1_applicable` の 8 行 `equal=true`）、パス 5 と 6 は行ごとに比較（`name=e170_bad_row` の 76 行 `o2_equals_e170=true`）；これらの比較は出力結果に記述されているが、2 つのパスを直接結びつける `assert` が欠落しており、この項目も不足している。共有項の中で最も重要なのはプールレベルのチェッカーそのものである（パス 5、6 は同一の `check_pool_image` を使用）：それが誤った場合、両方のパスが同時に誤りとなるため、このセルは O2（独立したパーサー + チェッカー） が正しいことを示す証拠として機能できない；パス 1、2 は「どの回数の書き取りで三状態になるか」を共有しており、これが誤ると両方のパスのステート数が同時に誤りとなる。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:1044-1163`（整段抄，未转述）**

````markdown
### 再実行

装置と E170（O3 仕様実行器第一版）、導出装置はすべて checker ファイルの登録済み装置であり、メモリ包装によって開始される。研究側は Python である。`scripts/experiment-apparatus/replay.sh` における E171（三 oracle 合謀テスト）の元々の十一行登録のうち、第 1 回再実行の残りの九行（`grep -c '^E171（三 oracle 合謀テスト）|@driver_e171_r1_' scripts/experiment-apparatus/replay.sh`）はこの表に含まれていない：第1段階・改訂 6 のその回の三行は「比較器、再実行と登録の実行方法（改訂 6 のその回）」の節にあり、第2段階の二行は「第 1 回再実行第2段階・改訂 6 のその回」の節にある。改訂 12 の一行は「第 1 回再実行・改訂 12：ストリーム三段長再釘（2026-10-13）」の節にある。改訂 8、10、11 の研究側変異三行（ドライバ関数 `driver_e171_r1_s1_revision_8_tools_research_mutations`、`driver_e171_r1_s2_revision_10_tools_research_mutations`、`driver_e171_r1_revision_11_tools_research_mutations`）のページには対応する節がない。それぞれは独自の一時ディレクトリを持ち、生成物は E7RESULT 行のみを残し、exact 比較を行う：

| ドライバ関数 | 出力物 | 実行内容 |
|---|---|---|
| `driver_e171_s1_baseline` | `…-s1-baseline-run1.out` | 装置 `baseline` を 1 回実行 |
| `driver_e171_s1_known_bad_images` | `…-s1-known-bad-images.out` | ベースライン、E170（O3 仕様実行器第 1 版）から装置 `bad-states`（プレースホルダーシード）、装置 `known-bad-images` をエクスポート |
| `driver_e171_s2_bad_states_export`、`driver_e171_s2_known_bad_images`、`driver_e171_s2_images_o3`、`driver_e171_s2_o3_control_baseline` | `…-s2-bad-states-export.out`、`…-s2-known-bad-images.out`、`…-s2-images-o3.out`、`…-s2-o3-control-baseline.out` | 不良イメージの 3 つのグループを固定したツリー下での生成方法、O2（独立解析器 + checker）および固定実行器による判定、O3（独立仕様実行器）による陽性対照 |
| `driver_e171_s2_comparator` | `…-s2-comparator.out` | 第 2 セグメントのコンパレータ一連（9 行の実行試み）；コンパレータは第 3〜6 セグメントで判定行と config 行を変更しており、この再実行では一致しないため、変更指針なし |
| `driver_e171_s3_comparator_mutations` | `…-s3-comparator-mutations-2026-10-03-r7.out` | 研究側における 67 の変異を調査、`mutation_driver.py`、約 1 分間 |
| `driver_e171_s4_comparator` | `…-s4-comparator.out` | 第 2 セグメントの前置処理を含む、修正された行ごとにドライバを実行して全 117 行（2 つのライン、各ライン 2 スレッド、1 つの固定したツリー）、Q3-核を 3 回実行しコンパレータを実行；推定約 2 時間、このドライバ関数は実際に実行していない（重負荷であり、コミット時の全実験を再実行する）。117 行の読み取りは現在の `crates/` を対象とし、実装、O2（独立解析器 + checker）、O1（メモリモデルのチェックサム（HashMap 理想ファイルシステム））、装置またはその 117 行の記述が変更された場合、出力も変化する。表全体の手順には `--keep-compile-failures-as-they-are` を付与：この出力物はコンパイル失敗を補正せずに実行したドライバ版によるものである |
| `driver_e171_s4_f4_comparator` | `…-s4-f4-comparator.out` | まず `driver_e171_s4_comparator` を一連で実行し、その後その固定したツリーと作業ルートでで 164 行の拡張表を実行（同様に `--keep-compile-failures-as-they-are` を付与）、コンパレータを半分にし、拡張表付き、セグメント名 s4-f4；推定約 4 時間、実際に実行していない（`bash -n` で確認済み；コンパレータの半分は同一パラメータで既存のエクスポート上で再実行し、出力がバイト単位で一致している） |
| `driver_e171_s4_f4_substitutes_comparator` | `…-s4-f4-substitutes-comparator.out` | まず `driver_e171_s4_f4_comparator` を先に実行する（そのドラフトディレクトリには s4、F4 のエクスポートと固定したツリーが残っている）。その後、同一の固定したツリーと作業ルート上で、2 つの出力ルートの結果としてコンパイルに失敗した行のみを再度実行する（`--rerun-compile-failed-from` を使用し、代替実行で 2 つのテーブルの名前をすべてスキップし、2 つのラインごとに各々 2 スレッドを使用し、`--keep-compile-failures-as-they-are` を付与しない）。コンパレーターは並列に 117 + 164 + 代替実行を行い、`--substitution-rows-root` を使用し、セグメント名 s4-f4-substitutes を設定する。推定所要時間は約 4 時間半（s4 と F4 の一連の処理が約 4 時間で、代替実行に 1,277.9 秒を加算）。実際に実行していない（`bash -n` で確認済み；各ステップのパラメータは代替実行の実行コマンドを逐条照合しており、ドライバーやコンパレーターを再実行していない）。現在の `crates/` を読み込み、実装、O2（独立したパーサー + チェッカー）、O1（メモリモデルのチェックサム（HashMap 理想ファイルシステム））、デバイス、またはその 6 行の記述が変更された場合、生成物も変化する |

s4 の入力（既存のエクスポートディレクトリとドライバ出力）を使用して、2026-10-03 のコンパレータを再実行し、E7RESULT 行が `…-s4-comparator.out` とバイト単位で一致することを確認した（`cmp` は出力なし）。この結果は、コンパレータ側の半分については再現可能である。一方、装置とドライバ側の半分については、表全体を再実行してのみ検証できる。最初の数行の単一コマンドは以下の通り：

```
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- baseline --output-directory <新目录>
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- bad-states --dump-directory <基线导出目录的一份拷贝> --seed-hex 0000000000000000
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- known-bad-images --dump-directory <上一步那份拷贝> --output-directory <新目录> --e170-bad-rows
cd scripts/experiment-apparatus/e171_three_oracle_collusion && python3 mutation_driver.py --table ../../../../mutations/e171_three_oracle_collusion.tsv --scratch <草稿目录> --output <新文件>
```

第1段階比較器その版（`…-s1-comparator.out`、第1段階の `driver_e171_s1_comparator` は s2 のものによって置換され、もはや登録されていない）はまずベースラインを 2 回実行し、E170（O3 仕様実行器第 1 版）から装置の `dump` をエクスポートし、1 つのケースを 2 回実行し、陽性対照を 6 回実行する（ベースライン 3 回、空変異 3 回は `row_driver.py --empty-mutation --positive-control <名>` で）、そして行ごとのドライバで 3 行を逐次実行する（`row_driver.py --empty-mutation --rows 1,6`）。その後、`python3 comparator.py …` を実行し、全体として約十数回の装置実行を行う（各回約 1 分）。ドライバ関数はこの一連の処理を書き込む。比較器の config の行にはディレクトリ名のみを記述するため、再実行するディレクトリ名はドライバ内のそれと一致させる必要がある。比較器の a4–a7 は現在の `crates/mutations.tsv` を読み取る：表中の他の場所でコア実装が追加された行またはその 117 行が変更された場合、これらの行は変化する（これは装置の不具合ではない）。プレースホルダーシード `0000000000000000` で生成されるブラインドテスト行 E171（3 つのオラクル共謀テスト）は使用しない。E170（O3 仕様実行器第 1 版）は 1 回目の再実行時に第二套の不良状態表（`bad-states-second-run`）を追加した。`bad-states` は第一套であり、2 つのバージョンのエクスポート装置が第一套上の出力はバイト単位で同一である（「対照と自己点検」節の最後の項目参照）。

ドライバの自己チェックを再実行する：ベースライン、既知の不具合のあるイメージ、変異したドライバの 3 つのドライバ関数は `replay.sh` の基準に従ってドラフトディレクトリで 1 周実行され、出力とリポジトリに格納された生成物はバイト単位で一致する（`cmp` は出力しない）；コンパレータ側のドライバ関数については全行程を実行していない（十数回の装置実行であり、フロントエンドでは完了しないため）、その接着部分（各ステップのコマンドライン、ディレクトリ名、コンパレータに渡すパラメータ）は「装置、E170（O3 仕様実行器第 1 版）から装置と行ごとにドライバを既存のエクスポートディレクトリからコピーして」スタブで実行し、コンパレータは実際に実行され、出力とリポジトリに格納された生成物はバイト単位で一致する；各ステップのコマンド自体も生成物作成時に 1 つずつ実際に実行されている。全行程の実行はコミット時の再実行によって行う。

生成物（すべて `research/results/` 下にあり、名前プレフィックス `e171-three-oracle-collusion-2026-10-03-s1-` の後に続く）：

| ファイル名（プレフィックス以降） | 内容 |
|---|---|
| `baseline-run1.out`、`baseline-run2.out` | ベースライン 2 回の実行の E7RESULT 行（2 つともバイト単位で同一） |
| `baseline-timing.out` | 2 回の実行の E7TIMING 行（各バッチ・セグメントあたりの所要時間、ストリームドライバ、判、写それぞれに要した時間） |
| `known-bad-images.out` | デバイス `known-bad-images` モード：41 の既知不良イメージと O3 規約実行器第 1 版の 76 行の不良状態における O2（独立解析器 + checker） |
| `comparator.out` | 比較器：登録アンカー、ベースラインとの対比、1 つを閉じる、3 行試走、Q1 の分母、Q2 の O2（独立解析器 + checker）の 1 列 |
| `rows-timing.out` | 行ごとにドライバと比較器の E7TIMING 行（Q4 の単行数） |
| `comparator-mutations.out` | Python 側における 9 つの変異 |
| `e170-pinned-dump.out` | E170（O3 規約実行器第 1 版）の一時保存バージョンからエクスポートされたデバイスの `dump`（S2 の対比） |
| `e170-bad-states-placeholder-seed.out` | 同じバージョンの `bad-states`（P、N 行の出所、ブラインドテスト用行は使用しない） |
| `positive-control-o1-comparison.out`、`positive-control-o1-crash-judge.out`、`positive-control-o2.out` | 3 つの陽性対照がベースラインの実行でデバイスから出力されたもの |
| `disable-o1.out`、`disable-o2.out` | 1 つを閉じるスイッチの自己点検（ベースラインの各実行）におけるデバイスの出力 |
| `row-0-device.out`、`row-1-device.out`、`row-6-device.out` | 3 行試走のデバイス出力（row-0 は空変異） |

全名（`ls research/results | grep '^e171-three-oracle-collusion-2026-10-03-s1-'`、17 件）：

```
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out
e171-three-oracle-collusion-2026-10-03-s1-baseline-run2.out
e171-three-oracle-collusion-2026-10-03-s1-baseline-timing.out
e171-three-oracle-collusion-2026-10-03-s1-comparator-mutations.out
e171-three-oracle-collusion-2026-10-03-s1-comparator.out
e171-three-oracle-collusion-2026-10-03-s1-disable-o1.out
e171-three-oracle-collusion-2026-10-03-s1-disable-o2.out
e171-three-oracle-collusion-2026-10-03-s1-e170-bad-states-placeholder-seed.out
e171-three-oracle-collusion-2026-10-03-s1-e170-pinned-dump.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-comparison.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-crash-judge.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o2.out
e171-three-oracle-collusion-2026-10-03-s1-row-0-device.out
e171-three-oracle-collusion-2026-10-03-s1-row-1-device.out
e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out
e171-three-oracle-collusion-2026-10-03-s1-rows-timing.out
```

第2段階以降の生成物（`ls research/results | grep -E '^e171-three-oracle-collusion-2026-10-03-s[234]-'`、29 件、名前プレフィックス `e171-three-oracle-collusion-2026-10-03-` の後に続く）：

| ファイル名（プレフィックス以降） | 内容 | 集計対象か |
|---|---|---|
| `s2-bad-states-export.out`、`s2-known-bad-images.out`、`s2-images-o3.out`、`s2-o3-control-baseline.out`、`s2-o3-baseline.out` | 不良イメージ三族の生成法と O2（独立解析器 + checker）、O3（独立仕様実行器）判定、O3（独立仕様実行器）陽性対照、固定したツリーの実行器がベースライン上にあること（a3 の O3（独立仕様実行器）1 通し） | 集計対象：s4 コンパレータが読み取るのはこれらである |
| `s2-comparator.out`、`s2-rows-timing.out` | 第2段階コンパレータ（Q2 表全体、9 行試運転）とその所要時間 | Q2 の数は s4 と同一；9 行試運転は s4 全体表で代替される |
| `s2-comparator-mutations.out` | 変異表 26 条時の表全体実行 | 歴史的データ |
| `s3-baseline-after-the-fix.out`、`s3-row-0-device.out` | 改訂 3 で修正後番号取得、暖機記赤後の装置ベースラインと空変異一行（修正前とバイト単位で同一） | 集計対象：改訂 3 の実核 |
| `s3-k4c-attack-row-device.out`、`s3-k4c-check-driver.out` | 改訂 3：番号取得恒拒その行を行ごとにドライバで走査（O1（メモリモデル対照（HashMap 理想ファイルシステム））赤） | 集計対象：改訂 3 の実核 |
| `s3-comparator.out`、`s3-rows-timing.out` | s3 全体表 117 行のコンパレータと所要時間 | 無効化してアーカイブ（「s3 全体表無効」節） |
| `s3-clean-check-comparator.out` | 第5段階でクリーン作業根上で第 2、27、42、65 行を再検証 | 無効化の証拠としてアーカイブ |
| `s3-comparator-mutations.out`、`s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` | 変異表 37、40、44、44 条時の表全体実行 | 歴史的データ |
| `s3-comparator-mutations-2026-10-03-r4.out` | 52 条の回で途中で停止（61 行、判定行なし） | 集計対象外、アーカイブ |
| `s3-comparator-mutations-2026-10-03-r5.out` | 変異表 52 条の表全体実行、52 / 0 / 0 | 歴史的データ |
| `s3-comparator-mutations-2026-10-03-r6.out` | 変異表 65 条の表全体実行（x60–x72 を追加後）、65 / 0 / 0 | 歴史的データ |
| `s3-comparator-mutations-2026-10-03-r7.out` | 変異表 67 条の表全体実行、67 / 0 / 0 | 集計対象：再実行登録がこれを指す |
| `s4-comparator.out`、`s4-rows-timing.out` | s4 全体表 117 行、Q3-核三回走査、コンパレータと所要時間 | 集計対象：117 行の半分の読み取り結果（F4 産物内の第 1–117 行の結果行がこれとバイト単位で同一） |
| `s4-f4-comparator.out`、`s4-f4-rows-timing.out` | F4 拡張表 164 行と s4 の 117 行を併せたコンパレータと所要時間 | 集計対象：代替実行前の読み取り結果（再実行登録に `--keep-compile-failures-as-they-are` を付与した通し） |
| `s4-f4-substitutes-comparator.out`、`s4-f4-substitutes-rows-timing.out` | 117 + 164 行を併せ、編成不能な 6 行の代替実行後のコンパレータと所要時間（代替実行通しは 1,277.9 秒） | 集計対象：代替実行後の Q1、Q3、Q4 の読み取り結果（Q1、Q3 名簿が代替実行前と文字列として同一）、三回判定で引き出された産物行番号はこの通しから取得される |

全名:

```
research/results/e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out
e171-three-oracle-collusion-2026-10-03-s2-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out
e171-three-oracle-collusion-2026-10-03-s2-o3-baseline.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out
e171-three-oracle-collusion-2026-10-03-s2-rows-timing.out
e171-three-oracle-collusion-2026-10-03-s3-baseline-after-the-fix.out
e171-three-oracle-collusion-2026-10-03-s3-clean-check-comparator.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r2.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r3.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r4.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r5.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r6.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations.out
e171-three-oracle-collusion-2026-10-03-s3-comparator.out
e171-three-oracle-collusion-2026-10-03-s3-k4c-attack-row-device.out
e171-three-oracle-collusion-2026-10-03-s3-k4c-check-driver.out
e171-three-oracle-collusion-2026-10-03-s3-row-0-device.out
e171-three-oracle-collusion-2026-10-03-s3-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out
e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-rows-timing.out
e171-three-oracle-collusion-2026-10-03-s4-rows-timing.out
```

````

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:1164-1187`（整段抄，未转述）**

```markdown
### 回答できないもの

- Q1 の「有」は文字通り判定された（≥ 1 行）；12 行（丙 13 行）のうちどの行が真の不良とみなされるか、3 回ループによる結論判決は 3 点を与えた（「3 回ループ判決後の 3 点」節参照）：真のブラインドスポット候補は 7 行（第 15、23、42、242、269、270、271 行）、状態が不合法である根拠となる条項文書は「3 条すべて放通行の 7 行」「F4 で新たに全通過された行」の 2 節に記載されている；その他のカテゴリに属する 5 行のうち第 92 行の記述はバイトテーブルとマイルストーン受入には含まれており、意思決定本文には含まれない。3 点は「このバッチで取得した状態」における読み取り結果であり、各行が別々に分類される歴史的大多数は推測に過ぎず、実行されていない。分母 D = 37 は代替実行前と後で同一である（代替実行の 6 行はいずれも到達していない）、Q1 の「有」は翻っていない。
- 281 行のうち 238 行は到達していない（代替実行後；代替実行前は 232 行、残り 6 行は編成不可能）：この 2 つのストリームの歴史は実装の一部のみを辿っている（番号取得、ウォームアップ、リリース、書き込み可能マウント、管理者ロールバック、F の引き上げ、正常アンマウントがそれぞれ 1〜2 回ずつあり、分裂も削除もない、スペース不足のプールもない）、到達していないとはその改動の書き込みストリームがこの 2 つのストリーム上で変化しなかったことを意味するだけで、どの oracle がそれを覆っているかについては言及しない；うち 7 行の改動は実行されたがディスクには書き込まれていない（第 49、110、155、161、162、174、252 行、「到達しなかったにもかかわらず赤色判定された 2 行と、到達しなかった行の中で戻り値が異なる 7 行」節参照）。3 回ループ後、全通過された行のいずれも「このバッチに等価」として記録されていない。
- 到達定義（登録読み取り法 4）は成功した呼び出しの戻り値と同等ではない；この項目は変更されていない（登録内容は不変）、分母は依然として 37：第 155、161 行における F の引き上げ上限の戻り値は 13（モデル 11）、書き込みストリームと結果は同一であり、到達していないとして記録されたが、O1（メモリモデル対照（HashMap 理想ファイルシステム））-対照判定では赤色判定となった；攻撃側は s4 ツリーの副本上で 232 行の到達しなかった行の成功した呼び出しの戻り値を測定し、7 行で差異があった（第 49、110、155、161、162、174、252 行）、集計行は `name=return_summary hidden_difference=7 identical=225 no_output=6`（副本プローブ、s4 ツリー、登録済み装置にはこの列がない；各行の詳細と由来は「到達しなかったにもかかわらず赤色判定された 2 行と、到達しなかった行の中で戻り値が異なる 7 行」節参照）；mkfs の 6 行（第 149–154 行）は測定されていない。
- 編成不可能な 6 行の代替実行ラウンドは既に実行済みである（「編成不可能な 6 行の代替実行（実行済み）」節参照）；s4、F4 の 2 つの製品は代替実行前のドライバで実行されたため、それらの登録行を再実行する際は `--keep-compile-failures-as-they-are` フラグを使用し、依然として代替実行を行わない。
- 順序に関する条項（第 242、270 行）はクラッシュポイントでの再実行スライスモデル上で数えられた：「条項が許可しない状態」はスライス規則（FUA 書き込みはそのディスクのみを許可、バリアはそのディスクのみを許可）によって区別可能であり、実際のマシン上一次の FUA 書き込みとその後に行われる書き込みの順序はこのバッチには含まれていない。
- O3（独立仕様実行器）は 37 行すべてで赤色判定を行っていない：その A(c) は実装自身が生成したルートによって構成され、「回復されたバージョンが [ack, begun] に含まれない」というカテゴリを捉えている；実装が一貫して誤記しているフィールド（コンテナ番号、書き込み時刻、F の値、帳簿行）であっても、O3（独立仕様実行器）は依然として「存在する」と判定する。これは実行器のコードを読み取って得た結果であり、O3（独立仕様実行器）を赤色判定させるために意図的に作成された不良実装の行を追加して検証することは行われていない；不良ミラーの一族では O3（独立仕様実行器）による赤色判定は多い（Q2）。
- 3 回の試行実行（第1段階）はテーブルのサンプリングではなく、s4 の全体テーブルが Q1 の読み取り値としてそれらを代替している；第 1 行の到達していないとは、その改動がこの 2 つのストリームの歴史において進行しなかったことを意味する。
- ベースラインでは O1（メモリモデル対照（HashMap 理想ファイルシステム））-クラッシュ判官の一部の状態は適用できない：ストリーム 2 の最後の `unmount` 呼び出しに対してモデルが応答せず、その呼び出し内で書き込まれた状態は判定されない；ベースラインバッチ 1 ストリーム 2 の 397 状態のうち 48 個、バッチ 2 の 119 状態のうち 14 個が適用できない（`o1_crash_judge_not_applicable`）、これらの状態において O1（メモリモデル対照（HashMap 理想ファイルシステム））の列は空である；ストリーム 1 には適用できない状態はない。
- 陽性対照 O1（メモリモデル対照（HashMap 理想ファイルシステム））-クラッシュ判官は各ストリームで 1 回ずつ反転した（合計 2 カ所）、登録文字列は「バッチ 1 の最初のもの」；O2（独立パーサー + チェッカー）の対照はストリーム 1 の全永続状態に既知の不良ミラーを第一条として重ねるのみであり、他のサンプリングポイントは存在しない。対照はパイプが赤色判定をマトリクスに運ぶことを証明するだけで、各 oracle があらゆる改悪に対して区別できることを証明するものではない。
- Q2 の O2（独立解析器 + checker）の列は、2 つのパス（パス 5、6）からそれぞれ 1 つずつ採取されています。P 行のパッチバイトとランディングベクトルは E170（O3 仕様実行器第一版）が書き出したものであり、期待される値も E170（O3 仕様実行器第一版）の設計者が推定したものです（登録 a2、F3）。Q2 で全通過された 4 つのパッチはすべて P8 であり、その判定は F3 に帰属します。41 の既知の不良イメージは O2（独立解析器 + checker）自身の回帰コーパスであり、O2（独立解析器 + checker）の 41 件がすべて赤判定であることは構築の結果であり、O2（独立解析器 + checker）の独立性の証拠ではありません。
- 行ごとの駆動と装置読み取り実行時の `crates/`：s4 の 5 回の駆動は同一の固定したツリーですが、`crates/` の他所では変化しており、全体テーブルから出力される数値はその 1 つのツリーのみを反映します。
- 不良イメージ上で実装された復元読み取りで何が返ってくるか（登録第 1 節読み取り法 11 を付記）、旧シードブラインドテストファミリーの回帰は実行されていません。Q1-丁（3 つすべて通過されながらレコードチェッカーまたはクラッシュポイント再プレイ手書きバージョンテーブルのオラクルが赤判定となった行）は実行されました：F4 の 12 行すべて通過（甲、乙）のうち 1 行もありません（`name=q1_delta … =none`）。比較器はこのセルを甲と乙のみで計算し、丙に追加された第 233 行ではクラッシュポイント再プレイのオラクルとレコードチェッカーが赤判定となります（`name=q1_quiet_row reading=q1_gamma row=233 … layer0_or_record_checker_red=true`）。クラッシュポイント再プレイのオラクルとレコードチェッカーは、O1（メモリモデル対照（HashMap 理想ファイルシステム））のみによる第 108、115 行でも赤判定となります。
- 装置内联単一テストは実行されていません（checker アーカイブ）。装置側のバリエーションで 15 行の 3 つの数値はありません；Python 側でのバリエーションは比較器と駆動の述語が赤判定になることを証明するのみであり、それらが読み取る装置出力が正しいことを証明するものではありません（装置出力は 2 回のベースライン、S2、S5 および対照によって別途固定されています）。
- E171（3 つのオラクル共謀テスト）の O1（メモリモデル対照（HashMap 理想ファイルシステム））の列は、モデル対照とモデルのクラッシュ判定官であり、RefFS と接続されていません（登録：クラッシュ判定官を O1（メモリモデル対照（HashMap 理想ファイルシステム））に含め、E171（3 つのオラクル共謀テスト）で RefFS を使用しないように別途記述）、D13（検証ルート）で確定された項目 6 のテーブルにおける O1（メモリモデル対照（HashMap 理想ファイルシステム））行では、クラッシュ後に許容される集合を RefFS と異にします。両者の下界も異なります（ディスク上に永続化された最新のルート、最後の成功した返却に対するリリース）。「確認済みだが失われた」というカテゴリーは E171（3 つのオラクル共謀テスト）の 4 つの列のうち O3（独立仕様実行器）のみが判定しており、RefFS（読み込みコードから読み出されたもの）では 37 行で実行されていません。RefFS の数値は 3 回めのラウンドにおける 2 回目の判定結果にのみ存在します（`research/prompts/e171-conclusion-r2-main-verification.md` 第 2 節 R6 のセル、読み込み値は `judge-summary.out`、攻撃側コピープローブ、s4 ツリーコア、RefFS 直接呼び出し crate、クラッシュ落下点手動指定、翻訳層を経由しない）：ウォームアップ形態において O3（独立仕様実行器）が「含まれない」と判定した 27 の状態に対し、RefFS も緑判定したのはこの 27 の状態のみです（27 / 27）。コンテンツを含む同形ストリーム書き込みにおいて O3（独立仕様実行器）が「含まれない」と判定した 27 の状態では RefFS はすべて赤判定となり、RefFS が赤判定となったのは合計 36 の状態で、さらに 9 の状態では RefFS が赤判定だが O3（独立仕様実行器）は「含まれる」と判定します（両者がクラッシュ時刻への帰属規則が異なるため推定されています）。これらの数値は E171（3 つのオラクル共謀テスト）の装置読み取り結果ではありません。
- Q2 で、O2（独立解析器 + checker）が通過したにもかかわらず O3（独立仕様実行器）が含まれないと判定した 29 枚のデータ（P 族 12、ブラインドテスト B 族 17、「Q2 で O2（独立解析器 + checker）は通過し、O3（独立仕様実行器）は含まれないと判定した不正イメージ」節参照）は、すべて構造上整合した偽造ルートおよび偽造続接イメージであり、不良実装ではない。パッチと期待値はともに E170（O3 仕様実行器第1版）の改変コードに由来する（E170（O3 仕様実行器第1版）ページ「答えられないこと」）。ソースは独立しており、Q2 では O2（独立解析器 + checker）と O3（独立仕様実行器）のみをペアで測定したため、不当な独立読み取りを行ってはいない。O1（メモリモデル対照（HashMap 理想ファイルシステム））は Q2 に含まれていない。ストリーム P2 および P5 の 2 つのパッチにおける O2（独立解析器 + checker）のリストは、ベース状態が切り替わるたびに更新される。この 29 枚の再実行読み取りでは現在の `crates/` を参照するが、s4 の読み取りを復元しない：ベースラインストリームと s4 ツリーが異なるため、数が依然として 12 と 17 であるとは保証されない。D13（検証ルート）のセグメントは、この数値について E171（三 oracle 合謀テスト）ページまでしか指していない。
- このページの行レベル読み取り（全通過、単一依存のみ、未到達リストおよび各行の状態）は、スナップショットサマリーにおけるその凍結したツリーにのみ属する：これは現在の `crates/` と同一のものではないため、ページ内のリストを直接現在の実装のリストとして使用することはできない。`replay.sh` の E171（三 oracle 合謀テスト）再実行行読み取りは現在の `crates/` を参照し、再実行一回目はその瞬間のツリーからの読み取り結果となる。s4 バージョンの登録済み装置（`scripts/experiment-apparatus/e171_three_oracle_collusion/`）では、Q2 の 29 枚を除き、これらの数値をそのまま再現することはできない（このバージョンの装置にはマウントモードがなく、RefFS コラムがないため、panic のストリームはエクスポートされない）。第 1 回再実行の装置 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` では RefFS コラムが存在するが、マウントモードおよび panic のストリームのエクスポートの有無については確認されていない。
- 第 125 行は自己崩壊する 6 行のうちの 1 つである：登録読み取り法 5 は panic を自己崩壊として記録し、分母には含めないため、D = 37 となる。panic 前に判定可能な呼び出しが 13 回存在する：レプリカプローブは s4 ツリー上で動作し、第 125 行のバリアントを適用した際、第 4–13 回の呼び出しにおける全永続状態 O2（独立解析器 + checker）は I-3.11（割り当て済みから defer を引いた値が最新ルート読み取りと一致する）を赤判定とした。第 14 回（`unmount`）で panic が発生し（`drive-s4-m125.tsv`）、装置は panic の全ストリームをエクスポートしない。panic とエラーを同様に切断して判定し、分母に含める場合 D = 38 となる（口径変更の予行演習であり、装置読み取りではないため、37 と 38 の両方を記載する）。もし到達した場合、成功呼び出しの返還値も含める（第 155、161 行を含む）と、D = 40 となる（37 に第 125、155、161 行を加算）。返還値が異なる 7 行をすべて含めると、第 125 行も含まれるため D = 45 となる（推論による計算式）。
- 述語 W（正しい実装の書込みマウントはこの状態を「証言された更新版を読み取れない」として解釈する）は、第 242 行および 269 行で取得したクラッシュ状態で真となる場合、それぞれ 15 および 63 のケースがあり、ベースラインでは 0 / 576 である。s4 ツリー core 上での 2 回の書込みマウントにおける証言クラス拒否は合計 61 件（第 242 行 9 件、第 269 行 52 件）である。これらの数値は攻撃側レプリカスクリプト（W）とレプリカプローブ（マウント）が s4 ツリー上で測定したものであり、登録済み装置には W もマウントモードもない。2026-10-03 の core が s4 から書き出したイメージは広範囲に `refused:Undecidable` となっている（ベースラインストリーム一 44 / 60、ストリーム二 500 / 516）：形式が変更されたため（ジャーナルヘッダー 311 → 487、新しいルートセグメント 188 → 364、識別項目 67 → 64）、古い記録は新しい解析器下で読み取れず、これはリグレッションではない。したがって、W の 15 / 63 および 61 の証言拒否は s4 core に対してのみ成立し、現在の core 上では対照的な意味を持たない。
- ユーザーデータが記録される位置の値が、layout に登録されたバイト数に含まれるかどうかは、このページでは判定しない。3 回の判決をユーザーに委ねる（`research/prompts/e171-conclusion-r3-main-verification.md` の第 5 節第 1 行、0 回）。ユーザーは 2026-10-03 に「含まれる」と決定した：D3（空間割り当て）で既に確定された項目 8 の射程を、新しいプールに新規作成されるファイル上の記録位置の値取得に変更する（t1@50180）ことはフォーマットレベルであり、チェックは C673（3 つの oracle の共謀テストにおける補完 oracle の変更方法が実施されていない点）⑥で行う。これは既に返済済みである（未収金表で返済済みの項目を参照）。ただし、このページの 3 分の 1 は変更しない：第 15 行の根拠は⑤の t2–t12 の記録位置であり、第 34 行の記録位置の変更はストリーム 2 で F を引き上げた後のリリースで行われたものであり、新しいプールに新規作成されるファイルではないため、依然として割り当てポリシーに分類される。
- 新しいツリーを再実行する装置（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`、このページではその読み値が記録されていない）によって生成される `name=batch` および `name=verdict` の 2 つの行において、`piece_panic_states=` は件数ごとにカウントし、状態ごとにはカウントしない：`<件数>:<状態数>` をカンマで連結する。0 の件数は記載せず、すべて 0 の場合は `-` と記す。同じパニック発生が `layer0_oracles` および `crash_judge` の 2 つの列にそれぞれ 1 回ずつ記録される（第 27–29 バッチのコード輪判決 `research/prompts/m3-formal-code-b29-r1-main-verification.md` の第 3 節第 3 条、両足で同様に判定し、装置を変更しない）。
- 変異表の旧第 38 行は「ステップ 4：候補集の F に目標ルート自身が持つ F を使用し、F_を適用しない」という名称に変更された（変更後の `crates/mutations.tsv` 内では、旧名は「最新ルート自身が持つ F」として記載されている）。2 つの `bad-implementations-117.tsv` ファイル（`scripts/experiment-apparatus/e171_three_oracle_collusion/` および `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` 下）の第 33 行は旧名を参照しており、現在のツリーからスナップショットを取得して再実行する際、この行が旧名で検索できないため補完処理が行われ、静默的に失敗しない（現在実行中の改訂 6 は凍結されたツリー内の旧表を読み取り、影響を受けない；第 32–33 バッチのコード輪判決 `research/prompts/m3-formal-code-b33-r1-main-verification.md` の第 2 節 P4「旧第 38 行の名前変更」一行および第 3 節第 5 条を参照）。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:1188-1206`（整段抄，未转述）**

```markdown
### 影響する決定

| 意思決定サブタスク | 関係 | 振り返り |
|---|---|---|
| D13（検証ルート）確定済み項目 6 | 支援 | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D13（検証ルート）確定済み項目 1 | 備料 | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D13（検証ルート）確定済み項目 7 | 備料 | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D13（検証ルート）確定済み項目 15 | 備料 | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D8（コアインデックス構造）確定済み項目 6 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D16（公開セマンティクス）確定済み項目 1 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D16（公開セマンティクス）確定済み項目 7 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D16（公開セマンティクス）確定済み項目 12 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D8（コアインデックス構造）確定済み項目 8 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D23（ジャーナルの役割と形式） 確定済み項目 3 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D23（ジャーナルの役割と形式） 確定済み項目 14 | 備料 | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D23（ジャーナルの役割と形式） 確定済み項目 15 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D3（空間割り当て）確定済み項目 8 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |
| D3（空間割り当て）確定済み項目 10 | 影響なし | 2026-10-18 に影響なし：装置の節に、研究側の変異表を3回再実行した成果物を追記、結論・数値は不変 |

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:1207-1218`（整段抄，未转述）**

```markdown
## 歴史バージョン

### 2026-10-03：ページ作成；O3（独立仕様実行器）接続継続；s3 全体テーブル無効化；s4 全体テーブル；F4 拡張テーブル完了；コンパイルに失敗した行の代替実行完了；結論として 3 回の判定書き込み

- ページ作成時に第1段階が既に実行済み：装置、研究側および単一テストは記述済み；ベースラインは二回走査、三条陽性対照（ベースラインと空変異をそれぞれ一回）、一つのゲートのスイッチ自己検査、Q2 の O2（独立解析器 + checker）を一列、行ごとに駆動で三行試走完了；O3（独立仕様実行器）は一列未実行（登録停止 S7）。ページ状態は「第1段階が既に実行済み」。
- 第2段階から第6段階（ページは段ごとに更新されていない）：O3（独立仕様実行器）を E170（O3 仕様実行器第一版）の第 1 回再走査で凍結した実行器に接続し、Q2 を表全体完了（主エージェントが F3 に従って 4 枚の P8 を判定）；装置側は番号取得、暖機エラーを赤判定（赤判定）として記録および行ごとに S5（登録改訂 3、4）を追加；s3 で表全体走査後に実行レーンで前行の変異が次行に持ち込まれたため、表全体無効化（改訂 5）、駆動は各行の修正時刻を設定し再編成を核として変更。
- s4 で表全体 117 行を同一の凍結したツリー（固定したツリー）上で完了し、Q1、Q3、Q4 は s4 に従って判定；D = 19 < 30 であり、F4 で表を拡張する 164 行の行ファイル、駆動および比較器の読み込み記述が完了したが、表全体走査は待機中。ページ状態は「部分実行済み（s4 表全体 117 行実行済み、F4 拡張表 164 行待機）」に変更され、以前は「第1段階が既に実行済み」であった；変更前の状態では表の四行すべてが「開いている」と記述されていた。
- F4 で拡張した表の 164 行走査完了後、比較器を統合して 281 行（`…-s4-f4-comparator.out`）：現状表 Q1 は「甲乙丙それぞれ 7 行、D = 19」から「甲乙それぞれ 12 行、丙 13 行、D = 37（代替実行前）」に変更され、Q3 は「11 行が一条に依存のみ」から「23 行」に変更され、Q4 に拡張表走査の掛時計を追加；拡張表で新たに生成された全通過行、一条に依存する 12 行、触発されていないにもかかわらず赤判定となった二行三節を追加。行ごとに駆動は編成失敗後にも代替実行を行うように変更（s4 および F4 の両方の産物は変更前の駆動で実行され、再走査ではスイッチを維持して代替実行を行わない）；コンパイルに失敗した 6 行の代替実行走査は待機中；ページ状態は「部分実行済み（s4 表全体 117 行と F4 拡張表 164 行実行済み、コンパイルに失敗した 6 行の代替実行待機）」に変更。研究側単一テストは 98 → 110 件、変異は 52 → 67 件となり、再走査登録の変異産物は `-r7.out` に変更され、`driver_e171_s4_f4_comparator` の一行を追加。
- コンパイルに失敗した 6 行の代替実行走査完了（`…-s4-f4-substitutes-comparator.out`、`…-s4-f4-substitutes-rows-timing.out`）：ページ状態は「部分実行済み（s4 表全体 117 行と F4 拡張表 164 行実行済み、コンパイルに失敗した 6 行の代替実行待機）」から「実行済み（…読み取り値が揃った…）」に変更；分母 D = 37 は代替実行前および後で同一であり、Q1 および Q3 のリストは文字通り同一；「コンパイルに失敗した 6 行の代替実行（待機）」セクションを「（実行済み）」に変更し、当初推定された最初の代替実行（第 33 行 → mount.rs グループ内インデックス 9、第 59 行 → mounted_read.rs インデックス 8、第 127 行 → allocator.rs インデックス 16、第 163 行 → mount.rs インデックス 37、第 206 行 → recovery.rs インデックス 4、第 243 行 → transaction.rs インデックス 25、第一行の交換を仮定して編成に成功すると想定）を取り下げ、実際に交換された行が産物に含まれている（第 127 行前の交換行もコンパイルに失敗したため）；再走査登録には `driver_e171_s4_f4_substitutes_comparator` の一行を追加（十行 → 十一行）、産物は 44 件 → 46 件。
- 結論：第 3 回の判決による書き戻し（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`）：「三分類列」を現値に変更。変更前の「三分類列」は以下の通り——第 15 行「このバッチに等価」（理由「条項が未定義」、歴史推論により分かれる）、第 27、112、277 行「このバッチに等価」、第 34 行「このバッチに等価」、第 92 行「真の盲点候補（条項根拠は主担当エージェントが認めること）」、Q1 行には「真の盲点候補 4 行（第 242、269、270、271 行）、このバッチに等価 1 行（第 277 行）」を記載、s4 の 7 行については「三分類は再実行しない」とした。変更後は、第 15、23、42、242、269、270、271 行が真の盲点候補となった（第 15 行は D3（空間割り当て）の既定義項 10⑤の t2–t12 の着地点に基づく）、第 27、112、277 行、第 34 行、第 92 行は他のカテゴリに分類され、「第 3 回の判決後の三分類」という新節を追加した。
- 同じ書き戻しにおけるその他の箇所：「触達していないにもかかわらず赤判定された 2 行」の節と「回答できないもの」内の触達に関する項目について、変更前は「他の未触達行に同様の差異があるか、装置が記録成功した呼び出しの返却値を把握できない」と記載されていたが、変更後は攻側が s4 のツリーコピー上で計量した 232 行のうち 7 行で返却値が異なる（`name=return_summary hidden_difference=7 identical=225 no_output=6`）と記述し、触達定義および分母 37 は変更しない。Q2 行および新節「Q2 における O2（独立解析器 + checker）の許可と O3（独立規約実行器）による含まれない判定の不良ミラー」には 29 枚（12 + 17）を追加する。「回答できないもの」に RefFS と O1（メモリモデルの対照（HashMap 理想ファイルシステム））の一列、29 枚の出所、行レベルの読み取り値はスナップショットツリーのみが属すること、第 125 行 D = 38 の予行演習、W と 61 の証言拒否は s4 core に対してのみ成立し、ユーザーデータの着地点の値取得をレイアウトバイトに含めるかどうかは別途ユーザーに委ねることを追加する。「影響を受けた意思決定」D13（検証ルート）の既定義項 6 は「備料」から「支援」に変更された（変更前の理由：セグメントが番号順に E171（三 oracle の共謀テスト）を引用しておらず、どの行が真の盲点かという推論であり、三方へのアプローチを行っていない）、さらに D3（空間割り当て）の既定義項 8、既定義項 10 および D23（journal の役割と形式）の既定義項 15 の 3 行、D13（検証ルート）の既定義項 1、D8（コアインデックス構造）の既定義項 6、D23（journal の役割と形式）の既定義項 14 の 3 行について、第 3 回の判決で新たに利用した条項への振り返り文を追加する。

```

**出处 `.claude/kb/experiments/171-3オラクル共謀テスト.ja.md:1209-1218`（整段抄，未转述）**

```markdown
### 2026-10-03：ページ作成；O3（独立仕様実行器）接続継続；s3 全体テーブル無効化；s4 全体テーブル；F4 拡張テーブル完了；コンパイルに失敗した行の代替実行完了；結論として 3 回の判定書き込み

- ページ作成時に第1段階が既に実行済み：装置、研究側および単一テストは記述済み；ベースラインは二回走査、三条陽性対照（ベースラインと空変異をそれぞれ一回）、一つのゲートのスイッチ自己検査、Q2 の O2（独立解析器 + checker）を一列、行ごとに駆動で三行試走完了；O3（独立仕様実行器）は一列未実行（登録停止 S7）。ページ状態は「第1段階が既に実行済み」。
- 第2段階から第6段階（ページは段ごとに更新されていない）：O3（独立仕様実行器）を E170（O3 仕様実行器第一版）の第 1 回再走査で凍結した実行器に接続し、Q2 を表全体完了（主エージェントが F3 に従って 4 枚の P8 を判定）；装置側は番号取得、暖機エラーを赤判定（赤判定）として記録および行ごとに S5（登録改訂 3、4）を追加；s3 で表全体走査後に実行レーンで前行の変異が次行に持ち込まれたため、表全体無効化（改訂 5）、駆動は各行の修正時刻を設定し再編成を核として変更。
- s4 で表全体 117 行を同一の凍結したツリー（固定したツリー）上で完了し、Q1、Q3、Q4 は s4 に従って判定；D = 19 < 30 であり、F4 で表を拡張する 164 行の行ファイル、駆動および比較器の読み込み記述が完了したが、表全体走査は待機中。ページ状態は「部分実行済み（s4 表全体 117 行実行済み、F4 拡張表 164 行待機）」に変更され、以前は「第1段階が既に実行済み」であった；変更前の状態では表の四行すべてが「開いている」と記述されていた。
- F4 で拡張した表の 164 行走査完了後、比較器を統合して 281 行（`…-s4-f4-comparator.out`）：現状表 Q1 は「甲乙丙それぞれ 7 行、D = 19」から「甲乙それぞれ 12 行、丙 13 行、D = 37（代替実行前）」に変更され、Q3 は「11 行が一条に依存のみ」から「23 行」に変更され、Q4 に拡張表走査の掛時計を追加；拡張表で新たに生成された全通過行、一条に依存する 12 行、触発されていないにもかかわらず赤判定となった二行三節を追加。行ごとに駆動は編成失敗後にも代替実行を行うように変更（s4 および F4 の両方の産物は変更前の駆動で実行され、再走査ではスイッチを維持して代替実行を行わない）；コンパイルに失敗した 6 行の代替実行走査は待機中；ページ状態は「部分実行済み（s4 表全体 117 行と F4 拡張表 164 行実行済み、コンパイルに失敗した 6 行の代替実行待機）」に変更。研究側単一テストは 98 → 110 件、変異は 52 → 67 件となり、再走査登録の変異産物は `-r7.out` に変更され、`driver_e171_s4_f4_comparator` の一行を追加。
- コンパイルに失敗した 6 行の代替実行走査完了（`…-s4-f4-substitutes-comparator.out`、`…-s4-f4-substitutes-rows-timing.out`）：ページ状態は「部分実行済み（s4 表全体 117 行と F4 拡張表 164 行実行済み、コンパイルに失敗した 6 行の代替実行待機）」から「実行済み（…読み取り値が揃った…）」に変更；分母 D = 37 は代替実行前および後で同一であり、Q1 および Q3 のリストは文字通り同一；「コンパイルに失敗した 6 行の代替実行（待機）」セクションを「（実行済み）」に変更し、当初推定された最初の代替実行（第 33 行 → mount.rs グループ内インデックス 9、第 59 行 → mounted_read.rs インデックス 8、第 127 行 → allocator.rs インデックス 16、第 163 行 → mount.rs インデックス 37、第 206 行 → recovery.rs インデックス 4、第 243 行 → transaction.rs インデックス 25、第一行の交換を仮定して編成に成功すると想定）を取り下げ、実際に交換された行が産物に含まれている（第 127 行前の交換行もコンパイルに失敗したため）；再走査登録には `driver_e171_s4_f4_substitutes_comparator` の一行を追加（十行 → 十一行）、産物は 44 件 → 46 件。
- 結論：第 3 回の判決による書き戻し（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`）：「三分類列」を現値に変更。変更前の「三分類列」は以下の通り——第 15 行「このバッチに等価」（理由「条項が未定義」、歴史推論により分かれる）、第 27、112、277 行「このバッチに等価」、第 34 行「このバッチに等価」、第 92 行「真の盲点候補（条項根拠は主担当エージェントが認めること）」、Q1 行には「真の盲点候補 4 行（第 242、269、270、271 行）、このバッチに等価 1 行（第 277 行）」を記載、s4 の 7 行については「三分類は再実行しない」とした。変更後は、第 15、23、42、242、269、270、271 行が真の盲点候補となった（第 15 行は D3（空間割り当て）の既定義項 10⑤の t2–t12 の着地点に基づく）、第 27、112、277 行、第 34 行、第 92 行は他のカテゴリに分類され、「第 3 回の判決後の三分類」という新節を追加した。
- 同じ書き戻しにおけるその他の箇所：「触達していないにもかかわらず赤判定された 2 行」の節と「回答できないもの」内の触達に関する項目について、変更前は「他の未触達行に同様の差異があるか、装置が記録成功した呼び出しの返却値を把握できない」と記載されていたが、変更後は攻側が s4 のツリーコピー上で計量した 232 行のうち 7 行で返却値が異なる（`name=return_summary hidden_difference=7 identical=225 no_output=6`）と記述し、触達定義および分母 37 は変更しない。Q2 行および新節「Q2 における O2（独立解析器 + checker）の許可と O3（独立規約実行器）による含まれない判定の不良ミラー」には 29 枚（12 + 17）を追加する。「回答できないもの」に RefFS と O1（メモリモデルの対照（HashMap 理想ファイルシステム））の一列、29 枚の出所、行レベルの読み取り値はスナップショットツリーのみが属すること、第 125 行 D = 38 の予行演習、W と 61 の証言拒否は s4 core に対してのみ成立し、ユーザーデータの着地点の値取得をレイアウトバイトに含めるかどうかは別途ユーザーに委ねることを追加する。「影響を受けた意思決定」D13（検証ルート）の既定義項 6 は「備料」から「支援」に変更された（変更前の理由：セグメントが番号順に E171（三 oracle の共謀テスト）を引用しておらず、どの行が真の盲点かという推論であり、三方へのアプローチを行っていない）、さらに D3（空間割り当て）の既定義項 8、既定義項 10 および D23（journal の役割と形式）の既定義項 15 の 3 行、D13（検証ルート）の既定義項 1、D8（コアインデックス構造）の既定義項 6、D23（journal の役割と形式）の既定義項 14 の 3 行について、第 3 回の判決で新たに利用した条項への振り返り文を追加する。

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:1-21`（整段抄，未转述）**

````markdown
## E171 Three-Oracle Collusion Test — First Re-run of Segment 1 and Segment 2 Full Table (Revision 6 Tree) Completed (Q1 Alpha, Q2 Beta, Q3 Alpha with denominator D = 39; Q4 Baseline Alpha and Rows 242, 269 Beta; Q5 Self-verification of alerts passed, judged Alpha; final main-line Revision 8 run pending tree capture, flow segment lengths re-pinned per Revision 12)

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Y1 Y3 P1 P2 P3 P4 P5 P8 P9 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 F1 F2 F3 F4 F5 F6 F7 V1 V2 W1 K4 Q5 S15 S17 Z7 U1 U9 U13 U16 V17 V31 V33 V36 R14 R19 R20 R33 F8 RS1 RS2 RS3 RS4 RS5 RS6 RS7 RS8 RA1 RA2 RA3 RX1 S19 F10 Z10 -->

[中文](171-三oracle合谋测试.md) · **English** · [日本語](171-3オラクル共謀テスト.ja.md)

```text
Local arm names and symbols on this page use lowercase (a1, a2, a3, a4, …). The preregistration, apparatus, artifacts and commands use the corresponding uppercase forms, matching the global registry identifiers. The KB prose uses lowercase to distinguish them from those identifiers; match uppercase when reading artifacts.
```

Run the preregistration for `e171-preregistration.md` (including revisions 1–5 to Section 12; the alternative reading for Q4 is deleted by the main agent before writing on the device, retaining only the whole-table version ≤ 60 seconds); Question 1 in the issue tracker `three-oracle-collusion-questions.md`: does a single bad implementation have any row that lets all three oracles pass when tested against the same batch of history and crash state?; Question 2: feeding a handcrafted bad image to O2 (independent parser + checker) and O3 (independent specification executor), is there an instance where both let it pass?; Question 3: taking known caught bad implementations one by one, turning off each oracle in turn—do the remaining ones still flag red?; Question 4: how long does this test take to run once, and how do you enter the gate. Clause D13 (verification path) is fixed at item 6: independence of the three oracles must be verified via collusion testing; self-assertion is not permitted.

**Current Status**: The faulty implementation table of 117 rows was run once using the revised row-by-row driver on a single frozen tree (s4, two lanes, each with 2 threads), along with Q3-core three passes and the comparator; during s3, the entire table was invalidated and archived because the driver carried mutations from one row into the next ("Section: s3 Entire Table Invalidated"). O3 (independent specification executor) ran the frozen executor for the first time using E170 (first version of the O3 specification executor); both initial and final frozen core pairings showed zero differences. For s4, the denominator D = 19 < 30 triggered registration F4, expanding the table to 164 rows (sequence numbers 118–281), which were run on the same frozen tree; the comparator merged both halves into 281 rows: denominator D = 37 ≥ 30. The six rows that could not be compiled in either pass (rows 33, 59, 127, 163, 206, 243) were not filled according to registration 5.2; after modifying the driver to fill these uncompiled rows, the filling pass was run on the same frozen tree (the six replaced rows were all unreached rows); the comparator merged 117 + 164 + substituted rows into `…-s4-f4-substitutes-comparator.out`: denominator remained D = 37, and Q1, Q3 lists matched exactly word-for-word before substitution ("Section: Substitution of Six Uncompiled Rows (Executed)"). After readings were complete, three rounds of conclusion judgments were conducted (`research/prompts/e171-conclusion-r1-main-verification.md`, `…-r2-main-verification.md`, `…-r3-main-verification.md`, with the third round being final), determining whether these readings support D13 (verification route) per established item 6: true blind spot candidates were harvested from rows 15, 23, 42, 242, 269, 270, 271; all others returned differing values among passed rows and unreached rows, categorized into other groups ("Section: Three-Fold Classification After Three Rounds of Judgment"); the three-fold classification, like the readings, belongs solely to that s4 frozen tree.

| Line | Verdict | Basis (Artifact) | To Read Alongside Beyond Literal Text |
|---|---|---|---|
| Q1 | Yes: alpha and beta each have 12 lines with passed by all three oracles (7 rows from s4 plus expanded table rows 242, 269, 270, 271, 277), gamma has 13 lines (plus row 233); denominator D = 37 (same before and after padding) | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`'s `name=q1_alpha`, `name=q1_beta`, `name=q1_gamma` (`verdict=yes_a_row_is_quiet_on_every_oracle`), `name=q1_denominator` (`counted_D=37 … enough_to_judge_none_side=true`); the Q1 three-reading lists in the pre-padding `…-s4-f4-comparator.out` are identical character-for-character line by line | Refer to Registration Section 6, Q1-alpha "≥ 1 verdict 'Yes'", which is independent of the denominator; D ≥ 30 only states that the threshold for the "No" side was met, but in this batch, "Yes" has already been established. The readings from those 7 rows in s4 remain unchanged in the F4 artifact (results lines 1–117 are identical byte-for-byte); the readings of the 5 newly generated expanded table rows and row 233 (gamma), as well as batch 2's row 265, appear in the "F4 Newly Generated Fully Released Rows" section. The trinity after three rounds of judgment is found in the "Trinity After Three Rounds of Judgment" section: 7 true blind-spot candidates (rows 15, 23, 42, 242, 269, 270, 271), and another 5 categories (rows 27, 112, 277 recovered via record reconstruction; row 34 allocation policy; row 92 clause in byte table and milestone acceptance); no single line is still recorded as "equivalent to this batch"; row 233 is passed by all three in gamma but caught by O1 (memory model oracle (HashMap ideal file system)) - crash judge in alpha and beta. Batch 2's row 265 is a sampling point insensitive case. O3 (Independent Specification Executor) has no red flags (`flag red`) on any of the 37 rows (`name=q1_column_red_rows … O3 (Independent Specification Executor) Jia=0 O3 (Independent Specification Executor) Yi=0`). After padding, the denominator remains 37, and "Yes" does not flip.
| Q2 | Yes (based on literal registration): both A and B have four rows each that fully pass, all P8. The same artifact has `name=q2_all_quiet` (`table_rows=107 all_quiet_rows=4 verdict=yes_a_bad_image_passes_both`), `name=f3_p_rows_judged_in_by_o3`, and `name=q2_negative_bases`; O2 (independent parser + checker) passes while O3 (independent specification executor) flags rows not present in the P family and blind-test B family rows of `name=q2_family` (`o3_only_strict=12`, `o3_only_strict=17`, copied verbatim into the section "Q2 where O2 (independent parser + checker) passes while O3 (independent specification executor) flags absent bad images"). All four are E170 (first version of O3 specification executor) designer-pushed "absent" P8 rows that O3 (independent specification executor) judges as "present"; F3 triggers; the main agent, following F3, judges them as expected pushes incorrectly, and these four are not bad images (judgment recorded in D13 (verification route), item 6 range). Q2 numbers match the second paragraph `…-s2-comparator.out` from the same input batch; s4 comparator recalculates once; F4 artifact rows for Q2, F3, and comparison match byte-for-byte with that s4 copy. Two geometric points (`q2_geometry_unstable_kinds=2`): flipped are stream one P2, P5 patch columns of O2 (independent parser + checker) where base state F is red and Mk is not red; O3 (independent specification executor) flags both base states as "absent," so these two patches' "both fully pass" points do not hold and do not flip; fully passing P8 has both base states as `same_on_both_points`. Additionally, there are 29 bad images where O2 (independent parser + checker) passes while O3 (independent specification executor) flags absent (12 from P family plus 17 from blind-test B family, 12 + 17): write "O2 (independent parser + checker) passes while O3 (independent specification executor) flags absent," do not write "unique to O3 (independent specification executor)," Q2 compares only O2 (independent parser + checker) and O3 (independent specification executor); O1 (memory model replay (HashMap ideal file system)) and RefFS did not run on bad images; these 29 are structurally consistent forged roots, forged continuation images, with patches and expectations from the same E170 (first version of O3 specification executor) modification code; details in "what it cannot answer." |
| Q3 | One row relies solely on one: A and B each have 23 rows (relying only on O1 (memory model replay (HashMap ideal file system)) lines 32, 45, 108, 115, 172, 233, 237, 249, 250, 281; relying only on O2 (independent parser + checker) lines 8, 65, 86, 87, 90, 99, 102, 135, 234, 238, 245, 258, 265). | Post-substitution `…-s4-f4-substitutes-comparator.out` rows `name=q3_alpha`, `name=q3_beta` (pre-substitution `…-s4-f4-comparator.out` same-named rows are identical character-for-character); Q3-core 9 rows `name=q3_core_row` all have `differences_outside_the_closed_column=0` (Q3-core takes the first-cut rows, expansion table has no first cut). | Q3-core 9 runs show zero differences; reading method 9 holds, F5 does not trigger. Lines 102, 233, 265 batches 1 and 2 have different red columns, all due to sampling point insensitivity (batch 2 did not capture the red segments or that class of state), marked "unstable." Why are those other 11 lines from s4 not visible in the "Q3 relies solely on one row's 11 rows" section? The 12 new rows from expansion appear in the "F4 newly emerged rows relying solely on one" section. (`name=q3_alpha`) |
| Q4 | Can only enter full volume: 117 rows, entire table, two lanes, 5 569.6 seconds; expanded table, 164 rows, another run, 7 607.9 seconds; threshold 60 seconds | Two lines in `e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out` with `name=table source=table-driver.out` (`rows=117 table_wall_ns=5569579167726`, `rows=164 table_wall_ns=7607936234181`) | 281 rows run in two runs; sum of wall-clock times for the two runs is approximately 13 177.5 seconds (sum of two numbers, not a single-run measurement); sum per row 11 114.4 + 15 191.9 seconds is an estimate for a single lane (not run separately). The 6 rows that fail to pass fill in the other run with 1 277.9 seconds (`…-s4-f4-substitutes-rows-timing.out`'s `name=table source=substitution-driver.out … rows=6 table_wall_ns=1277937171946`); sum of three runs is approximately 14 455.5 seconds, also a sum of three numbers. Q3—core per run, three rows, approximately 114–130 seconds |

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:22-40`（整段抄，未转述）**

```markdown
### Device, Research Side, and Mutation Table

Research-side re-runs of the mutation table across revisions (three lines from `replay.sh` performing byte-by-byte comparison): Revision 8 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-10-07.out` (`E7RESULT name=verdict mutations=26 caught=26 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`), Revision 10 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-s2-revision-10-tools-research-mutations.out` (`mutations=29 caught=29`), and Revision 11 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-revision-11-tools-research-mutations.out` (`mutations=15 caught=15`); all three have `invalid=0 survived=0 timed_out=0`.

| Item | Location | Description |
|---|---|---|
| Ingestion Device | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` | Modes: `baseline`, `row`, `known-bad-images`; switches: `--positive-control`, `--disable-oracle`; read-only, driver-only, observation-only; does not modify core, checker, harness, format, or replay-prefix; contains 31 inline unit tests (`grep -c '#\[test\]'`) |
| Research Side | `scripts/experiment-apparatus/e171_three_oracle_collusion/` (Python; no `main.rs` in the directory, so Cargo does not recognize it as a binary) | Selection script (registered revision 5.2, sha256 `4032a0e0…`, copied verbatim); table with 117 rows `bad-implementations-117.tsv` (registered revision 5.9, copied by name); F4 expansion table `bad-implementations-expansion-164.tsv` and its generator `expansion_table.py`; `row_driver.py`, `comparator.py`, and several of its modules; wiring for the frozen executor (`frozen_o3.py`); judgment script for bad images (`o3_bad_image_judge.py`); four unit test files; mutation driver |
| Mutation | Research side: 67 rows in `scripts/mutation/tables/e171_three_oracle_collusion.tsv`; Device side: last 15 rows in `crates/mutations.tsv` (names starting with "E171 apparatus", count via `grep -c '^[E]171 装置' crates/mutations.tsv`) | Research side runs via `mutation_driver.py` in this directory (registered revision 1, line 1); Device side is a checker file; three numbers are assigned to the gate checker-tier-crates-mutation-replay at submission time |
| Re-run | E171 (Three Oracle Collusion Test) entries: 20 rows registered in `scripts/experiment-apparatus/replay.sh` (`grep -c '^[E]171.@driver' scripts/experiment-apparatus/replay.sh`): originally 11 rows, plus 9 rows added from the first re-run (first segment·revision 6 run: three rows; second segment: two rows; revisions 8, 10, 11 research side mutations: three rows; revision 12: one row) | Original 11 rows are in the "Re-run" section; the three rows from the first segment are in the "Comparator, Re-run, and Ingestion Run Methods (Revision 6 Run)" section; the two rows from the second segment are in the "First Re-run Second Segment·Revision 6 Run" section; the revision 12 row is in the "First Re-run·Revision 12: Three-Stream Long Re-nailing (2026-10-13)" section; the three rows for revisions 8, 10, and 11 have no corresponding sections on this page |
| Artifacts | 46 files starting with `e171-three-oracle-collusion-2026-10-03-` under `research/results/` (`ls research/results \| grep -c '^e171-three-oracle-collusion-2026-10-03-'`) | Each file is listed individually in the artifact table of the "Re-run" section |

- Stream drivers: Two streams (new pool new file creation, fixed script to E then normal unmount) are copied from E170 (first version of the O3 specification executor) for the export device's two stream drivers, sampling three batches and writing in neutral format without `#[path]` importing it. Three changes are made: API calls within the driver do not `.expect`; outcomes (success, error members, panic) are recorded into the `ok` / `err` columns of `calls.tsv` and `call-outcomes.tsv`. Following each call, an ideal model is reasoned about, recording O1 (memory model check against HashMap ideal filesystem) - check (comparing the model's answer with the implementation's outcome, `judge_and_advance`). For each sampled state, calculate O1 (memory model check against HashMap ideal filesystem) - crash judge (`crash_recovery_disagreement`) and O2 (independent parser + checker) `check_pool_image`, writing to `oracle-verdicts.tsv`. The `baseline` compares locally with current computed values of constants and crates (s1, 42 rows with `name=s1` all have `holds=true`; `grep -c 'name=s1 ' …-baseline-run1.out` and `grep -c 'name=s1 .*holds=false'` for current counts). The `row` does not compare back; it only takes batch 1 and batch 2.
- Line numbers of the three S8 calls within the device (obtained via `grep -n`): `newest_persisted_root` at line 1883, `observed_read_back_after_a_crash` at line 1885, `crash_recovery_disagreement` at line 1907; all are within `crash_judge_of_the_state` (line 1875) and its called function `crash_judge_verdict`; O1 (memory model check against HashMap ideal filesystem) - check's `judge_and_advance` is at line 544.
- Known bad image family: Imports, constants, helper functions, and `known_bad_images` from `crates/singlefs-harness/tests/checker_known_bad_images.rs` (sha256 `e8543908…`) are copied into a module within the device (lines 7–1294, plus 12 constants and helper functions scattered after line 1295 that are used by `known_bad_images`). The `known-bad-images` pattern has 41 core rules; entry names match original files exactly; clean image digests equal those of `build_pool` (S4).
- Per-line driver: Run once to take a snapshot of the workspace (`Cargo.toml`, `Cargo.lock`, `.cargo`, `crates/`, 237 files, total digest written as the first line of the driver's standard output). Each lane synchronizes its own tree and build directory from this snapshot. Make one change, run `cargo build --release`, execute device `row` via `run-with-memory-cap.sh` (time limit 1800 seconds), then restore. Rows in the table are selected by name and core anchor; if unavailable, replace with the next row in the same group according to fill-in rules (at most three times). For s4 and F4 runs, the driver only performs fill-ins for failures occurring before compilation. Starting from the eighth segment, even failures after compilation trigger fill-ins (replaced rows that fail to compile are swapped again; combined count of pre-compilation failures and post-compilation failures is at most three). Groups do not include E171 (three-oracle collusion test) or the device's own variant rows. `row-result.tsv` records `substituted_from` and every attempted but unused `failed_attempt_<sequence>`. The switch `--keep-compile-failures-as-they-are` is used only for re-running s4 and F4 outputs with `replay.sh`; the fill-in output variant does not include this flag.
Source identifiers: `.expect`, `--keep-compile-failures-as-they-are`, `replay.sh`.


```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:41-89`（整段抄，未转述）**

````markdown
### Results of s4 Table Organization (Rows copied verbatim from artifacts)

`research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out` (2119 lines, last line `E7RESULT name=done emitted=2119`). Frozen tree pairwise comparisons, four positive control oracles each run twice (baseline and empty mutation), Q1 denominator with three reading methods, 7 rows passed by all three (alpha), Q3 two reading methods, Q3-core 9 runs and verdict lines:

```
E7RESULT name=config segment=s4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=empty_mutation red_units=1 red=true
E7RESULT name=q1_denominator rows_run=117 counted_D=19 untouched=93 only_in_the_ignored_pass=0 compile_failed=2 anchor_not_unique=0 not_found=0 self_crash=3 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=false
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=5 rows_run=117 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=19 O1=5 O1-对拍=5 O1-崩溃判官=1 O2=8 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=0 row_numbers=none
E7RESULT name=q1_alpha counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_quiet_row reading=q1_alpha row=15 row_name=实审_A2c_第_15_条：用户数据落点不从这张图的单元区起点起扫 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=22/22 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=23 row_name=并行线三_C116：分裂出来的右半容器号不是触发分裂那条新记录的_inode_号 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=26/26 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=27 row_name=实二十：记录头读者把末条标志位_0_读反（带标志的读成不是末条） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=34 row_name=步_5：抬_F_生效之后不放开扣住的槽 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/498 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=42 row_name=实二（回退改形态：SysPre_与_B1，D16_已定项_1_/_7、D22_已定项_7_/_9、I-7.9_/_I-7.12、C556_/_C557）：卸载把_F_抬到现行那一版带的_F（原地不动）而不是它的_txg o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/537 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=92 row_name=步_1_验收第_4_条：inode_记录的写入时间留成_A_的（覆盖写照抄上一版的写入时间） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/61 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=112 row_name=步_3：前缀跨实例边界（把别的实例的记录也接上） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_beta counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=19 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_beta caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_core_row closing=O1 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=no_executor row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=0 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=7 q1_beta_all_quiet_rows=7 q1_gamma_all_quiet_rows=7 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=11 q3_beta_rows_held_by_a_single_oracle=11 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
```

Count of subgroups and categories (counting one command from the output as-is, outputting it verbatim): `grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` outputs `2 compile_failed`, `19 counted`, `3 self_crash`, `93 untouched` (four lines). The five rows not included in the denominator: implementations on lines 6, 7, and 85 self-collapse; compilations fail on lines 33 and 59. Register 5.2 to write "one compilation fails... switch to index j + 1 within the same file group"; versions s3 and s4 of line-by-line driving only fill gaps for failures prior to compilation (not found, non-unique names, non-unique anchors); lines 33 and 59 do not have gap-filling; after changing to "compilation fails" in the eighth segment, the driver also fills gaps; these two rows along with the four rows of expansion-table compilations that fail were run during that gap-filling pass ("six compilation-fail rows filled (already run)" section).
Source identifiers: `2 compile_failed`.

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:90-105`（整段抄，未转述）**

```markdown
### Three All-Pass Lines of 7 Rows

Read the export directory for s4 (`scripts/experiment-apparatus/target/e171-dumps/rows-s4/row-<序号>/dump/`), comparing each operation and state individually against `baseline-run1`. The draft scripts `quiet_rows_s4_fua.py` and `returned_check.py` only read from the export directory without writing to the database. Apply the criteria defined in `.claude/rules/mutation-sampling.md` to this batch: **Equivalent to this batch** means that for both streams, every state along with its recovery on the implementation of that state falls within the set allowed by the clauses (provide justification; historical reasons for separation are noted beside it but not executed); **Sampling point insensitive** means that if a clause forbids a certain state in one of the two streams, the sampling rule did not select it; **True blind spot** means that among the selected states, there is one forbidden by a clause, yet all three checks pass. The count for "Different Mirrors" refers to states where the mirror summary in `states.tsv` differs from the baseline state at that point. The count for "Selected New Version" refers to states where the root txg chosen during recovery by the implementation reading the journal is not less than the first root written after the first difference. Here is what each of the three checks examines: O1 (Memory Model Checksumming with an Ideal File System using HashMap) compares the `ObservedRoot` returned by the implementation after each call (defined in `crates/singlefs-harness/src/model.rs`: key of the root, journal counter, F on the root, file contents, instance table, and allocation records per disk); O1 (Memory Model Checksumming with an Ideal File System using HashMap) - Crash Judge compares the root chosen during recovery by the implementation and the read-back content; O2 (Independent Parser + Checker) validates the structure of individual mirrors; O3 (Independent Specification Executor)'s allowed set A(c) is constructed from the root slots written in this specific row (`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/version_table.py`, see file header), projecting only six items per version (explained in `root_ring.py` under `VersionProjection`: instance identifier, checkpoint_txg, tree table unit pointer, central mapping tree root pointer, tree ID water level, and F). O3 judges whether the specific version recovered from the mirror by itself belongs to A(c) (`landed_is_allowed =` line within `judge_state` in `executor_core.py`). The last column of the table titled "7 Rows Where All Three Passed" shows the current values of the three checks after the conclusion of all three rounds (changes to the old three-check columns are documented in the "History Versions" section); the full attribution for each row is detailed in the section "Three-Check Results After Three Rounds".

| Line (Mutation) | How it differs from the baseline | Which state does it fall into | Can O1 (Memory Model Fuzzing with HashMap Ideal File System), O2 (Independent Parser + Checker), or O3 (Independent Specification Executor) detect it? | Ternary Classification |
|---|---|---|---|---|
| 15 (User data placement does not start scanning from the beginning of the unit zone in this diagram) | Starting from the 22nd operation in both streams: user data units are placed at offsets 822,149,120 → 823,132,160 on each of the two disks (32 KiB per disk), with identical content byte-by-byte; pointers and checksums within the 16 KiB node pointing to them change accordingly; eleven internal blocks generated in the same release (t2–t12) also shift collectively by 64 slots, moving the open segment [50240, 50304) entirely to [50304, 50368): on device 0, 12 write operations change their placement locations (operation 22: 50180 → 50240; operation 24: 50240 → 50304; operation 26: 50242 → 50306; …; operation 44: 50252 → 50316); both streams are identical, with two rows named `name=row15_first_publish` in `row15-offsets.out`; starting from the 4th invocation of stream two, the return point shifts by 4 operations (`mount_writable` 92–179 → 92–183), resulting in a total of 555 differing operations; call outcomes remain identical; journal-based recovery state-by-state is also identical | Mirror states differ: Stream A batch 1 has 1 and 20 states, batch 2 has 7 states; Stream B batch 1 has 1 and 364 states, batch 2 has 119 states; selecting the new version: Stream A batch 1 has 1 and 12 states; Stream B batch 1 has 1 and 356 states, batch 2 has 112 states | O1 (Memory Model Fuzzing with HashMap Ideal File System): `ObservedRoot` lacks placement offsets, reads back identically, undetectable; O2 (Independent Parser + Checker): placements are within the unit zone, accounting is self-consistent, flagged green (flag red as flag red); O3 (Independent Specification Executor): projection lacks placement offsets, "exists" | **True Blind Spot Candidate**: Based on D3 (Space Allocation) established item 10⑤ regarding t2–t12 placements: "⑤ Placement on newly created files in the new pool (open segment [50240, 50304), bump cursor starting from 50240)" (`.claude/kb/decisions/03-空间分配.md` "Established Item 10: Definition of clustered segments, four-grid user data placement points, and bump order for submission internal blocks" final decision); scope "Modify bytes for newly created files in the new pool: Yes"; eleven submission internal blocks (default geometry) placements collectively rearranged" (line 226); byte table "t2 through t12 sequentially at 50240, 50242–50243, 50244, …" (`.claude/kb/layout/01-first-txn.md` "V. Space Accounting and Allocation" row on "Submission Internal Blocks"); under mutation, user data lands at 50240, eleven submission internal blocks collectively shift by 64 slots, not at the placement fixed by ⑤; all three checks pass. Another basis (only available after user decision on 2026-10-03): D3 (Space Allocation) established item 8 scope "Exception is **the first version where the tree is built from mkfs water level**"; "Newly created files in the new pool belong to this category"; t1@50180 "Registered in byte table of [layout/01-first-txn.md](../layout/01-first-txn.md), considered format-level; mirrors deviating from this written type are invalid" (`.claude/kb/decisions/03-空间分配.md` "Established Item 8: User data blocks do not enter clustered segments, placement is the smallest empty slot within the selected device" scope); mutation moves t1 from 50180 to 50240, inherently invalid; during three rounds of decision-making on retention, this criterion was not yet available, relying solely on ⑤. Historical divergence (pushed): unit zone just sufficient pool, skipping initial segment then publishing where model says it should succeed reports insufficient space; O1 (Memory Model Fuzzing with HashMap Ideal File System) - fuzzing would flag red |
| 23 (the right half container number resulting from the split is not the inode number of the new record that triggered the split) | Starting from the 26th operation in the second stream: for the first file, the write to the inode leaf (code 3, 32 KiB on each of two disks) at offset 53 changes one byte from 0x01 to 0x00 (container number 1 → 0), and the header checksum consequently changes; similarly in the same release, the 28th operation (16 KiB node) changes bytes at offsets 52, 60, 131, 149, etc., from 0x01 to 0x00; subsequent rewrites of the inode tree for each release carry different operations in stream two (155 operations); the call outcome is identical; the recovery state-by-state is identical | Mirror differences: stream 1 has 20 states in batch 1 and 7 in batch 2; stream 2 has 364 states in batch 1 and 119 in batch 2. When the first-file version (txg 3) or a later version is selected: stream 1 has 12 states in batch 1; stream 2 has 356 in batch 1 and 112 in batch 2 | O1 (memory model replay, HashMap ideal filesystem): does not read from disk; `ObservedRoot` contains no container number; reads back the same; O2 (independent parser + checker): I-9.4 (container number must not exceed minimum key) passes if "inode number ≥ container number" and strictly increasing along leaf order; when a single leaf, 1 ≥ 0 holds, resulting in a green flag ; O3 (independent specification executor (specification executor)): projection does not enter the inode tree, "exists" | **True Blind Spot Candidate**: D8 (Core Index Structure) has finalized item 6 as writing "**New Pool New File Write**": one code 2 root node (header with key range [1, 1]; one entry: key 1, type 2, birth tree, container number 1, birth generation 3, pointer); one code 3 leaf (type 2, container number 1, birth generation 3, record count 1, record width 140) (``.claude/kb/decisions/08-core index structure.md` "Finalized Item 6: leaves of the inode tree are code 3 packed record units, records have fixed length 140, published (inode number, object birth generation) are not reused" decision); scope writes "each container number is the inode number at first insertion of some record" (line 205); container number 0 is not the inode number of any record. States selected in the new version do not exist in the mirror derived from the clause text; all three pass. Sampling points that would cause O2 (independent parser + checker) to flag red (赤判定) (theoretical, not run): splitting after a leaf fills up, right half container number 0 is less than left half, I-9.4 (container number must not exceed minimum key)'s "strictly increasing along leaf order" flags red |
| 27 (Record header reader inverts the last flag bit to 0) | Write stream and call outcome are identical; they differ only during recovery on reading the journal: Stream A reaches state 16, Stream B reaches states 151 (Batch 2 state 0). The baseline applies one record and selects a new version; the mutation reads records with the last flag bit as "more follows," applies zero records, and stops at the previous version. Read-back content differs in 3/30 states (e.g., Stream A state 41 baseline `1,3 file_read e8ca4bf8…`, mutation `1,2 no_file`). | Partially published state: The call for the discarded version has not returned, and its root slot has not landed (`returned_check.py` states 57 all have `call_returned=False` and `its_root_slot_landed=False`). | O1 (Memory model replay against HashMap ideal filesystem) - Crash judge: If the previous version was committed by the model, is no older than the latest root slot persisted on disk, and read-back matches that version, then green judgment is correct; O1 (Memory model replay against HashMap ideal filesystem) - Replay: Calls do not undergo recovery; O2 (Independent parser + checker), O3 (Independent specification executor): Mirror and baseline are identical byte-for-byte, each ignoring the implementation's recovery. | **Another category: Three checks that do not rely on record-based reconstruction within implementation recovery.** The state itself is valid (calls have not returned, both old and new versions are in the allowed set); the error lies in the implementation performing fewer record applications during recovery, violating D13 (Verification Path) established item 1 M4 "`Recover` returns a root exactly equal to j*" and D23 (Journal role and format) established items 15 "The root of that publication is reconstructed from records" and 14 "Recovery applies only records with `(instance_id, checkpoint_txg)` strictly greater than the selected root." O1's criterion for the crash judge (a version committed by the model, no older than the latest persisted root slot) is weaker than M4. O2 judges the mirror prior to recovery; O3 does not run the implementation's recovery. The landing point of M4 is E165 (Bounded exhaustive replay and mutation killing scope) and Verus, not these three checks. Sampling points that diverge (pushed but not run): A state after a publication where one call has already returned and the commit point exists only in journal records (root slot not yet written); mutations would discard this already-returned publication. States where publications from both streams write the root slot before returning do not exist. At that time, O1's "must not be older than the latest persisted root slot" also fails to block it (since the root slot is unwritten); we have not measured whether all three checks would pass in such cases. |
| 34 (After raising F, slots held by locks are not released) | Stream A shows no difference; Stream B starting from operation 498 (the `publish` after raising F): User data unit landing point changes from 822083584 to 822411264 with identical content; nodes pointing to it and allocation records change accordingly (51 operations differ); call outcomes are the same; recovery is state-by-state identical. | Mirrors differ: Stream B states 168, Batch 2 state 21; selecting roots at txg 17 or later: Batch 1 state 60, Batch 2 state 14. | Same as line 15: O1 does not compare landing points; O2 judges green (accounting self-consistency); O3 is "present". | **Another category: Allocation policy, mirror valid:** The publication after raising F swaps to a free slot without reusing reclaimed slots; allocation records and accounting are self-consistent in every state. D3 (Space Allocation) established item 8 scope "Remaining writes (overwrites, subsequent publications, remounts) violate it; the implementation's written mirror is valid, blocking it relies only on runtime reads" (``.claude/kb/decisions/03-Space-Allocation.md` "Established Item 8: User data blocks do not enter clustered segments; landing point is the smallest empty slot within the selected device" scope; exception for new pools creating new files; this line's change occurs in Stream B's publication after raising F and does not apply there); all three checks, by design, do not judge allocation policy. Historical divergence (pushed): In a pool where space after raising F is sufficient only to reuse the few locked slots, a publication reports insufficient space while the model says it should succeed; O1's replay would flag red. This line directly asserts re-use in the test named `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` listed in `crates/mutations.tsv`. (`call_returned=False its_root_slot_landed=False`) |
| 42 (unmount raises F to the txg of the current version, not its own) | Stream one is identical; stream two starts at op 537 (the first write in that unmount call: system configuration slot on a two-disk setup, offset 481 bytes 0x11 → 0x0b, i.e., F 17 → 11, along with checksum), and after unmount pushes empty published writes, the roots and nodes within are also 11 (24 ops differ); unmount returns `ok` normally; recovery state is identical | Mirror differs: stream two batch 1 has 48 states, batch 2 has 14 states, including all persistent states; selecting the root written by unmount (txg ≥ 18): batch 1 has 32 states, batch 2 has 7 states | O1 (memory model replay with HashMap ideal filesystem): unmount lacks a modeled answer (registration read rule 6); this call does not replay, writes fall into states within unmount that the crash judge record does not apply to and cannot see (if an unmount answer existed, `ObservedRoot` carrying root F would show 11 vs. 17, pushed); O2 (independent parser + checker): system configuration F and root-carrying F are both 11; I-7.12 (system configuration F is not lower than the F on the same-disk root) holds; a single mirror does not know whether it should be 17 | O3 (independent specification executor): projection carries F, but A(c) is the root written by this very line with F = 11, "present" | **True Blind Spot Candidate**: D16 (publish semantics) has determined item 1's field table row "normal unmount" writes "push a string of empty publishes to raise F to the txg of the current version" (`.claude/kb/decisions/16-publish-semantics.md`); scope "costs of normal unmount" that clause writes "after unmount, F = txg of the current version; rollback candidate set retains only the current version". Under mutation where unmount returns with full persistent state F = 11, the rollback candidate set keeps roots from txg 11–16, which are not in clauses-permitted mirrors; all three paths are allowed. O1 (memory model replay with HashMap ideal filesystem) half is a known gap explicitly stated by registration read rule 6 (model lacks unmount answer) |
| 92 (inode record write time retained as A's) | Stream one is identical; stream two starting from op 61 of the second `publish` (overwriting write that inode leaf, 32 KiB on each of the two disks): bytes at offsets 200, 208, and 216 within the leaf change from 0x3c to 0x00 (record start point 136; +64 / +72 / +80 are seconds for atime, mtime, ctime: baseline 60, variation retained as A's 0); differences also exist in header checksum offsets 10–13 and separately at 89–92; subsequent republishes of this leaf carry (139 different ops); call outcomes are the same; recovery states are the same | Mirror difference: stream two batch 1 has 344 states, batch 2 has 112 states; selecting the version with the overwriting write (txg 4) and later: batch 1 has 336 states, batch 2 has 105 states | O1 (memory model alignment (HashMap ideal filesystem)): `ObservedFile` contains only content; `write_time` has zero hits in both `model.rs` and `model_comparison.rs`; D13 (verification path) item 6 table for O1 (memory model alignment (HashMap ideal filesystem)) row judges "content, size, directory structure" but cannot see it; O2 (independent parser + checker): a single mirror does not know the write time parameter of this publish, judged green; O3 (independent specification executor): projection does not enter the inode tree, "present" | **Another category: clauses in byte tables and milestone acceptance, decision body lacks one sentence**: D8 (core index structure) item 6 field table only defined offset 64 for "atime / mtime / ctime seconds"; "write time is a publish parameter" is written in `.claude/kb/milestone/01-first-txn.md` section "Step 3 inode tree and the remaining trees" current paragraph and `02-second-txn.md` section "Step 1 Overwriting write: same file, same pool, second publish (publish B)" inode record line; byte table writes "write moment (mkfs parameter, reproducible)" (`01-first-txn.md` section "IV. File Metadata (inode)" row "inode record (offset 64)"); acceptance writes "inode record write time retained as A's ⇒ flag red" (`02-second-txn.md` same step "variation: extent leaf record pointer forgot to change"), but cannot find a sentence in the decision body for value retrieval. Selecting new version state (write time 0, this publish parameter is 60) does not meet acceptance; three are invisible by design (no API returns write time), not an independence failure (`.claude/kb/layout/01-first-txn.md`) |
| 112 (Prefix Crossing Instance Boundary) | The write stream and call outcome are identical; however, the recovery pass viewing the journal differs across the two streams in three states: Batch 1 states 9–11, Segment 1: the first warm-up write—the new instance 1's journal record—has landed, but its root slot has not. Baseline selects `0,0`, applies 0 mutations; variation attaches instance 1's record to instance 0's root, applies 1 mutation, resulting in `1,1`; read-backs contain no files | State where warm-up is halfway written (all 6 states in `returned_check.py` have `call_returned=False`) | O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Crash Arbiter: `1,1` is a version submitted by the model, not older than the latest persisted root slot, and read-back matches; therefore, green judgment for O2 (Independent Parser + checker) and O3 (Independent Specification Executor) is correct as mirror and baseline are byte-identical | **Another Category: Three cases where implementation recovery relies on record reconstruction without returning (additional mutations applied)**: Warm-up did not return; both `0,0` and `1,1` are in the allowed set. The error lies in the implementation's recovery attaching records from other instances, violating D23 (Role and Format of Journal). This is based solely on Item 14 Note 1 "Prefix Rule Does Not Cross Instance Boundary" (`.claude/kb/decisions/23-journal_role_and_format.md` "Item 14: The lower bound for replay is given by the selected root; administrator rollback, failure handling, and instance switching"), with only this note as basis. This row's variation is pinned in the red example nail table of the gate checker-tier-formal-verification (`.claude/gate.d/e165-pinned-red-examples.tsv`, where column 1 contains the red example `371a71190dce62e0…`), judged by E165 (Bounded Exhaustion for Recovery Replay and Mutation Killing Boundary) via bounded exhaustion. Sampling points that would diverge (inferred, not executed): leaving records from other instances within the loop, specifically those not on the selected root's timeline (new instances after the root are discarded during crash recovery); upon attachment, recovery falls to a version not submitted by the model; O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Crash Arbiter would flag red; no such states exist in either stream |

Line 102 (lifting F first to write the system configuration step without a new F) is the extra line in batch 2 that allows all through (`name=q1_alpha … rows_that_differ_from_batch_1=102`): stream two's 440th op (lifting F first to write the system configuration slot, offset 481 one byte 0x0b → 0x00) differs from its 537th op (unmounting the previously written system configuration slot, 0x11 → 0x0b), totaling 4 ops; O2 (independent parser + checker) flagged red 12 states in batch 1, all I-7.12 (system configuration F not lower than the F on the same disk root), with 6 states each in segments 60 and 73 (`name=trajectory row=102 stream=fixed_script_through_e_then_unmount oracle=O2（独立解析器 + checker） total_red_states=12 peak=6 positive_segments=2 of_segments=79 …`). Batch 2 took only 17 segments in stream two (7, 11, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 57, 61, 65, 70, 74), taking no states in segments 60 and 73 (`awk -F'\t' '$3==60\|\|$3==73' states.tsv` outputs only batch 1's 8 + 8 states): **sampling points are insensitive**; the sampling points that would separate are any state in segments 60 or 73. This cell flips between the two batches, recorded as "unstable"; Q1 and Q3 conclusion words do not flip on batch 2 (batch 2 allows all through 8 rows still ≥ 1; only relying on one row's 10 rows still ≥ 1).

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:106-125`（整段抄，未转述）**

```markdown
### Q3 Relying Solely on a Single Line of 11 Rows

Only write objects that are unreachable and fields that have not been flagged; do not judge the conclusion. Readings are taken from s4's export directory (draft script `caught_rows_s4.py`, not stored in the database) and the line with `name=trajectory` of the product; segments where the number is positive represent "the count of red-state segments / the total segment count for this stream".

| Line (Mutation) | Where is the red flag located? | Why are the other two not visible? |
|---|---|---|
| 32 (Writable mount does not write to instance table; instance table is copied as-is) | O1 (Memory model replay with HashMap ideal file system): Replay: Stream 2's 4th call to `mount_writable` returns `disagrees: this version of the instance table` (the model must have the row from the previous instance; the implementation returns an empty table). The model stops at this call. Later, Stream 2 in state 324 (batch 1) crashes; the crash judge is not applicable. | O2 (Independent parser + checker): This line has a red flag in state 0 (both batch 1 and stream 2 state 397 are green). None of the implemented invariants check an instance identifier from the root against rows in the instance table (derived from reads, without checking each row individually in `crates/singlefs-checker/invariants/`). O3 (Independent specification executor): The projection does not include the instance table; A(c) is the root written by this line itself; "present". |
| 45 (Slots unreadable at mount time are reread; still bad, then reject) | O1 (Memory model replay with HashMap ideal file system): Replay: Stream 2's 12th call to `raise_floor` returns `err:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`; the model says it should succeed. | O2 (Independent parser + checker) and O3 (Independent specification executor): The write stream before rejection matches the baseline line-by-line (Stream 2 stops at op 440, with 0 differing ops). Retrieved mirrors are all prefixes of that batch from the baseline. Rejected calls do not leave a mirror; the single-mirror judge has no object to judge. |
| 108 (Code 2 reader completes bytes starting from the last payload byte) | O1 (Memory model replay with HashMap ideal file system): Replay: Both streams' 2nd call to `publish` returns `disagrees: this version of the file content` (the glue cannot be untangled; the implementation returns a unit: "non-zero completion bytes"). Stream 2's 4th call to `mount_writable` returns `err:Recovery`. | The error is on the reader side in the implementation. Written bytes match the baseline (Stream 1 has 0 differing ops; Stream 2 also has 0 differences before op 92). O2 (Independent parser + checker) reads raw bytes using its own parser without calling the core's reader. O3 (Independent specification executor) uses Python's own reader. Both sides see mirrors identical to the baseline byte-by-byte. The crash point replay shows that version of the oracle and record checker red (`layer0_or_record_checker_red=true`, not one of the three). |
| 115 (Take prefix, core anchor filter; do not look at txg) | O1 (Memory model replay with HashMap ideal file system): Crash judge: Batch 1 Stream 1 state 32 and Stream 2 state 52 "read failure: root not selected" (implementation recovery reports `RootPublishCarriesMoreThanOneLastRecordFlag…`); segments are positive 8/12, 12/16. O1 (Memory model replay with HashMap ideal file system): Replay: Stream 2's 4th call to `mount_writable` returns `err:Recovery`; the model says it should succeed. | Written bytes match the baseline (Stream 1 has 0 differing ops). O2 (Independent parser + checker) judges mirror structure. O3 (Independent specification executor) recovers from the mirror using its own prefix rules, ignoring the implementation's recovery; seen mirrors are identical to the baseline byte-by-byte. Crash point replay shows that version of the oracle and record checker red. |
| 8 (Idle does not decrease with allocation) | O2 (Independent parser + checker): I-5.2 (idle statistics match). Batch 1 Stream 1 state 9, Stream 2 state 353; segments positive 2/12, 69/79. Batch 2 Stream 2 state 112. | O1 (Memory model replay with HashMap ideal file system): No accounting rows in `ObservedRoot`; all calls succeed and content matches. O3 (Independent specification executor): Projection does not enter the accounting tree; "present". |
| 65 (blocks floor down) | O2 (Independent parser + checker): I-9.15 (inode record blocks equal ⌈size ÷ 512⌉). Batch 1 Stream 1 state 9, Stream 2 state 353. | O1 (Memory model replay with HashMap ideal file system): `blocks` zero hits in `model.rs` and `model_comparison.rs`; read-back content matches. O3 (Independent specification executor): Projection does not enter the inode tree. |
| 86 (When writing the system configuration slot, write those 8 bytes as 0) | O2 (Independent parser + checker): I-7.12 (System configuration F is not lower than the F on the same disk root) Batch 1 Stream Two 97 states; segments are positive 19/79; Batch 2 28 states | O1 (Memory model diffing (HashMap ideal file system)): Compares the F of the root band (`ObservedRoot`'s `rollback_floor`), not the F in the system configuration slot; O3 (Independent specification executor): The projected F is taken from the root, and the system configuration slot is only used for geometric selection; A(c) is the root written by this row itself, "in" |
| 87 (The core version overcounts F by 1 at the upper limit of flight) | O2 (Independent parser + checker): I-8.1 (Ring geometry is large enough) Batch 1 Stream One 51 states, Stream Two 385 states; I-7.13 (System configuration pool-level fields are within the reader's collection range) Stream One 2 states, Stream Two 12 states; segments are positive 12/12, 79/79 | O1 (Memory model diffing (HashMap ideal file system)): Does not compare fields in the system configuration slot; O3 (Independent specification executor): Same as line 86, no overcount at the upper limit of flight in the projection |
| 90 (The change count for new files in a new pool is written back as 1) | O2 (Independent parser + checker): I-9.26 (inode change count falls between the birth generation and the container's birth designation code) Batch 1 Stream One 9 states, Stream Two 240 states | O1 (Memory model diffing (HashMap ideal file system)): Does not compare the change count recorded in the inode; O3 (Independent specification executor): The projection does not enter the inode tree |
| 99 (The defer row writes 0) | O2 (Independent parser + checker): I-3.11 (Allocated minus defer equals the latest root walk-read) Batch 1 Stream One 9 states, Stream Two 353 states | O1 (Memory model diffing (HashMap ideal file system)): Does not compare accounting rows; O3 (Independent specification executor): The projection does not enter the accounting tree |
| 102 (The step of raising F first writing to the system configuration does not include new F) | O2 (Independent parser + checker): I-7.12 (System configuration F is not lower than the F on the same disk root) Batch 1 Stream Two 12 states; segments 60 and 73 each have 6 states; Batch 2 is not flag red (sampling points are insensitive, see the end of the section "Three rows all allowed") | O1 (Memory model diffing (HashMap ideal file system)): The answer for `raise_floor` compares the written root (the F pair on the root), not the system configuration slot; O3 (Independent specification executor): Same as line 86 |

O3 (Independent Specification Executor) shows no red flags on the preceding row of 19 rows where s4 is included in the denominator (`name=q1_column_red_rows … O3 (Independent Specification Executor) A=0`, `O3 (Independent Specification Executor) B=0`), while it provides strict judgments on all 450 batch state-1 instances across two streams per row (`o3_judged_states_strict=450`, `name=q1_o3_coverage … rows_with_no_strict_judgement=0`). A single observation derived from the executor's implementation (read code, not run separately): A(c) consists only of root slots written by this specific row itself; projections involve only six items at the root. Therefore, O3 (Independent Specification Executor) flags red only in cases such as "implementing its own recovery rules and clauses differently" or "applying them partially after a crash during a single release"; even if an implementation consistently writes a field incorrectly, O3 (Independent Specification Executor) still judges it as "in". O3 (Independent Specification Executor) flags red more frequently on the bad image family (Q2): P row has 26 reds out of 22, blind test has all 40 red, and known bad images have 28 strict reds out of 41 (`name=q2_family` across four rows).

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:126-129`（整段抄，未转述）**

```markdown
### Invalidate the Entire S3 Table

The S3 run involved the entire table (output root: `scripts/experiment-apparatus/target/e171-dumps/rows-s3/`), where each of the two lanes reused compilation directories across rows; modified files were restored using `copy2`, and synchronization used `rsync -a`, both preserving modification times back to the snapshot moment. Cargo determines newness based on modification time, so Lane 1, after running past line 112 (the mutation in replay-prefix) but before reaching line 114, compiled devices that all carried the mutation from line 112. Revision 5 was recorded accurately for these 45 rows, along with the mechanism measurements and driver modifications on cargo 1.98.0; consequently, the S3 full table, each pass of the S3 Q3-core, and the S3 comparator outputs are no longer treated as readings for Q1, Q3, or Q4. Files are archived without deletion: `e171-three-oracle-collusion-2026-10-03-s3-comparator.out`, `…-s3-rows-timing.out`, and `…-s3-clean-check-comparator.out` (the fifth section covering clean rechecks for lines 2, 27, 42, and 65) under `research/results/`, along with `scripts/experiment-apparatus/target/e171-dumps/rows-s3*` and `rows-s5-clean-check`. Among the "Q1 full pass of 40 rows out of a denominator of 52" read by S3, 38 rows were executed while carrying the mutation. S4 reran all 117 rows on the corrected driver, with the verdict line set to `rows_compiled_without_the_rebuild_check=0`.

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:130-138`（整段抄，未转述）**

```markdown
### F4 Table Expansion

Section F4 Registration Tenth: "Too few touched rows, Q1's 'none' is insufficient for judgment: proceed to the third paragraph expansion table (5.2); even after expansion, if still insufficient, record 'insufficient for judgment', do not record 'none'", triggering observation "Q1-Denominator D < 30"; s4's D = 19. Expansion table follows Section Registration 5.2 "Expansion Table": the selection method with a step size of 4 yields an additional 164 rows compared to the step size of 8 (the 56 overlapping rows with step size 8 are not re-run), and according to the output order of the selection script, they are appended after row 117, numbered 118–281, blade number 3.

- Row file `scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv` (sha256 `f1bd082e…`), generated twice by the row-picking script in the same directory, `expansion_table.py` (step size 8, step size 4); table rows are taken from `crates/mutations.tsv` excluding E171 (three-oracle collusion test) and its own mutation rows (using the same exclusion method as a6, a7). It is byte-for-byte identical on both the frozen tree version in s4 and the version in the 2026-10-03 repository; per file: admission 5, allocation_record_tree 2, allocator 12, block_device 2, code_two_tree 3, extent_tree 2, inode_tree 1, instance_table 2, journal 2, make_filesystem 6, mount 44, mounted_read 3, mounted_session 2, records 2, recovery 28, root_record 1, system_configuration 3, transaction 34, unit 2, replay-prefix 8. Registered sha256 for 5.9 rows with 117 lines; row summaries for expanded table rows are taken from the table at the moment of table generation.
- Row-by-row driving with `--other-selected-rows`: skip 117 rows and every name in both the expansion table and the fill-in table (registered as 5.2 "Expansion and fill-in do not re-pick already-run rows").
- Comparator `--rows-root` can handle two sets (the 117-row set and the expanded set; reject if the same row number exists in both), `--expansion-rows` yields two additional results during expansion: `name=f4_expansion` (byte-for-byte identical per row calculated on the `--mutations-table` by the expansion script, 164 rows, consecutive indices, knife number 3, no overlap with 117 rows) and `name=q1_denominator_by_table` (denominator reported separately by table); Q1 denominator and Q3 are both calculated based on the combined 281 rows. Re-run the modified comparator with s4 input (only providing rows-s4); E7RESULT row is byte-for-byte identical to `…-s4-comparator.out` (`cmp` produces no output).
- The expansion run runs on the frozen tree version in s4 (snapshot summary `d1586465…`, same as the five s4-driven runs); only then can they be merged into a single batch; do not re-freeze the tree. The main agent runs all seven steps according to the script returned in paragraph 7 (empty mutation row does not reach `baseline-run1`; expansion and row-picking scripts are byte-for-byte identical on the table of the frozen tree; this time, only one snapshot summary exists for both runs compared to s4); the expansion run takes two threads: 7 607.9 seconds.

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:139-172`（整段抄，未转述）**

````markdown
### F4 Expansion Results (entire lines copied from artifacts)

`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out` (4,698 lines, last line `E7RESULT name=done emitted=4698`; timing lines are in `…-s4-f4-rows-timing.out`, 291 lines). Merged table, denominator, three reading methods, Q3, judgment rows, and the newly expanded all-pass rows (5 rows from A, row 233 added by C):

```
E7RESULT name=config segment=s4-f4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4+rows-s4-f4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=f4_expansion rows=164 expected=164 first_number=118 last_number=281 numbers_are_consecutive=true slice_3_rows=164 names_shared_with_the_117_rows=0 found_once_in_today_table=164 equal_to_the_selection_script_on_today_table=true
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=232 only_in_the_ignored_pass=0 compile_failed=6 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=12 rows_run=281 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=37 O1=12 O1-对拍=11 O1-崩溃判官=4 O2=15 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=2 row_numbers=155,161
E7RESULT name=q1_alpha counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_beta counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=37 all_quiet_rows=13 row_numbers=15,23,27,34,42,92,112,233,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=37 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q3_beta caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q1_denominator_by_table table=step_8_table rows_run=117 counted_D=19 counted_row_numbers=8,11,15,23,27,32,34,42,45,65,86,87,90,92,99,102,108,112,115
E7RESULT name=q1_denominator_by_table table=step_4_expansion rows_run=164 counted_D=18 counted_row_numbers=135,172,233,234,237,238,242,244,245,249,250,258,265,269,270,271,277,281
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=2 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=12 q1_beta_all_quiet_rows=12 q1_gamma_all_quiet_rows=13 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=23 q3_beta_rows_held_by_a_single_oracle=23 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
E7RESULT name=q1_quiet_row reading=q1_alpha row=242 row_name=增补_3_第_4_件（增补_2_收口表第_40_行_/_C381_的前提）：发布的持久顺序把系统配置槽轮换挪到根槽_FUA_写之前，最后一步失败时根就不会已经落盘 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=516 o3_judged_states_provisional=516 first_operation_that_differs_from_the_baseline=8/8 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=269 row_name=C554_乙-配置续：见证值取成槽世代号（mkfs_之后第一次取号写的_tail_不再是_0，新池新建文件的字节变了） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=0/0 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=270 row_name=C577：发布在系统配置轮换之后不过屏障就返回（树表_0_条的一版上返回之后见证值可以落后） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=390 o3_judged_states_provisional=390 first_operation_that_differs_from_the_baseline=20/20 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=271 row_name=D8_已定项_8_头_ID：deleted_inodes_的树表条目头_ID_退回写_0（不写它服务的_inode_树的树_ID） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=44/44 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=277 row_name=取前缀核单测：施加一次发布之后新根段不换（停在所选根的） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_gamma row=233 row_name=由记录重建的根不照抄所选根的文件系统标识（代码轮_m3-formal-code-r1_K3，R3-1「其余字段等于所选根」）：改写全零 o1_comparison_red_calls=0 o1_crash_judge_red_states=45 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=true
```

Count of categories (counting one command from the output as-is): `grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` outputs `6 compile_failed`, `37 counted`, `6 self_crash`, `232 untouched` (four lines). The 12 rows not included in the denominator: implementations self-crashed on lines 6, 7, 85, 125, 235, and 272; compilations failed on lines 33, 59, 127, 163, 206, and 243 (this reading is from before the patching run; the patching run for "the six compilation-failed rows patched (already run)" is covered in the section "Patching the Six Compilation-Failed Rows (Already Run)"). The result lines for rows 1–117 (`name=row`, `row_touch`, `row_matrix`, `trajectory`, `row_o3`, `row_s5`) are identical byte-for-byte between F4 output and s4 output (1819 rows, `cmp` produces no output); the seven fully-passed `name=q1_quiet_row` reads are also identical line-by-line; the three variants for those seven lines in s4 do not need to be re-run. The result lines for Q2, F3, and the control group are also identical byte-for-byte between the two outputs. Re-running inputs for F4 and s4 with the modified comparator from the eighth segment yields E7RESULT rows that are identical byte-for-byte with each of the two respective outputs.
Source identifiers: `6 compile_failed`.

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:173-188`（整段抄，未转述）**

```markdown
### F4 Newly Released All-Pass Line

Read the export directory for F4 (`scripts/experiment-apparatus/target/e171-dumps/rows-s4-f4/row-<序号>/dump/`, comparing each operation and state against `baseline-run1` line-by-line); draft scripts include `quiet_rows_f4.py` (modify the seventh paragraph of `quiet_rows_s4_fua.py` to output the root), `extra_checks_f4.py`, and `order_by_segment_f4.py` (all read-only from the export directory, not writing to the database). The three-part criteria are identical to those in the section "Three rows fully allowed." For the two order-related lines (lines 242 and 270), where the number or sequence of write streams changed, comparing state-by-state against the baseline is meaningless; instead, adopt a single judgment method that counts "states not permitted by clauses" on this line's own states. Apply the same judgment method to the baseline, yielding zero in both cases (the judgment method can distinguish them): segment cutting uses the comparator's own `baseline_checks.segments_of_the_write_stream` (FUA write allowance for its disk, barrier allowance for its disk).

| Line (Mutation) | Difference from Baseline | Forbidden State per Clause (Batch 1 / Batch 2) | Detectable by O1 (Memory Model Fuzzing with HashMap Ideal File System) / O2 (Independent Parser + Checker) / O3 (Independent Specification Executor)? | Triforce |
|---|---|---|---|---|
| 242 (Published persistence order rotates system configuration slot to root slot before FUA write) | Starting from the 8th operation of two streams: replace the sequence at the end of each take, warmup, and publish—「root slot FUA → system configuration slots on two disks → barrier」with「system configuration slots on two disks → root slot FUA → barrier」; written content remains identical per line but order changes (stream 1 has 9 operations, stream 2 has 57 operations compared by index); in baseline, root slot FUA forms its own segment, while mutation merges it into the same segment as system configuration writes (3 of 8 segments in stream 1, 19 of 59 segments in stream 2; baseline: 0 segments); call outcome is identical | System configuration write committed within the same segment but root slot FUA not fully committed: stream 1 has 21 / 3 states, stream 2 has 133 / 19 states; baseline: 0 | O1 (Memory Model Fuzzing with HashMap Ideal File System) - Crash Judge: Root slot not committed, recovery selects previous version, which was model-submitted and is no older than the latest persisted root slot → Green; O1 (Memory Model Fuzzing with HashMap Ideal File System) - Fuzzing: Call outcome identical; O2 (Independent Parser + Checker): 0 red flags for this line (system configuration slot precedes root by one publish in a single mirror, and `crates/singlefs-checker/invariants/` does not cover this case); O3 (Independent Specification Executor): A(c) is composed of the root written by this line itself; recovery lands on a version contained within it → "in" | **True Blind Spot Candidate**: D16 (Publish Semantics) has finalized Item 7 stating:「The persistence order for a single publish is always COW unit/node → barrier → journal record → barrier → root slot (FUA) → system configuration slot → barrier」;「System configuration slot updates after the root it witnesses persists」(`.claude/kb/decisions/16-Publish Semantics.md`「Item 7: Publish Persistence Order」); The state where system configuration lands before the root it witnesses cannot be constructed according to clause write order, so all three checks pass |
| 269 (Witness value taken as slot generation number) | Stream One: First two ops (0, 1) after mkfs (the first write of the system configuration slot to both disks): Offset 469 byte 0x00 → 0x01, along with checksum (155–158); Stream Two has additional ops 92, 93 (writable mount generation write) 0x04 → 0x06. Offset 469 is the journal tail (`crates/singlefs-checker/src/lib.rs`'s `SYSTEM_CONFIGURATION_TAIL_OFFSET`); Call outcomes are identical; Journal recovery state-by-state is identical | States differing from mirror and baseline: Stream One 31 / 0, Stream Two 78 / 14 (All persistent states identical: subsequent releases write that slot as the correct tail) | O1 (Memory model replay (HashMap ideal filesystem)): `ObservedRoot` does not include system configuration; recovery does not initially trust the tail (D23 (Role and format of journal) Item 3 already determined); Readback is identical; O2 (Independent parser + checker): Reads the tail (`crates/singlefs-checker/src/lib.rs`'s `check_system_configuration_slot`), but no invariant checks its value; O3 (Independent specification executor): The executor reads the tail only in anchor assertions (`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/executor_main.py`'s `anchor_assertions`), verifies only persistent states, and does not use it for judgment | **True Blind Spot Candidate**: D23 (Role and format of journal) Item 14 writes "Witness value c_witness read at the moment of generation write... rewritten" (all self-verified slots take maximum journal_tail); "mkfs writes tail 0, first generation read after mkfs reads 0, writes 0: new pool new file bytes unchanged" (`.claude/kb/decisions/23-journal_role_and_format.md` "Item 14 already determined: Replay lower bound given by selected root; Administrator rollback, failure handling, and instance switching"); System configuration slot written as 1 is not in the mirror derived from the clause text; all three are allowed |
| 270 (Release returns after system configuration rotation but before barrier) | The final barrier at each release end is removed: Stream One 57 → 53 ops, Stream Two 595 → 565 ops; subsequent return points shift earlier overall; Written content remains the same; System configuration rotation and first batch of writes for next call fall into the same segment | States where system configuration write for one call is not fully persisted, but writes for subsequent calls are: Stream One 7 / 6 states, Stream Two 70 / 59 states; Baseline 0 | O1 (Memory model replay (HashMap ideal filesystem)) - Crash Judge: Root slot FUA of that release is already persisted; recovery selects it; Judgment green; O1 (Memory model replay (HashMap ideal filesystem)) - Replay: Call outcomes are identical; O2 (Independent parser + checker): System configuration lags behind root once per release is valid within a single mirror (I-7.12 (System configuration F not lower than F on same disk root) only compares F); O3 (Independent specification executor): "In" | **True Blind Spot Candidate**: D16 (Release semantics) Item 7 finalized "Root slot (FUA) → System configuration slot → Barrier; return after fsync etc. system configuration rotation persistence" (Line 181), scope "Barrier caliber" explicitly states the barrier after system configuration rotation is "barrier added before release return" (Line 186); States where system configuration has not persisted after return but subsequent writes have already persisted cannot be constructed according to clauses; all three are allowed |
| 271 (tree table entry header ID for deleted_inodes written as 0) | Two streams, operations 44 and 45 (tree table units, 16 KiB each on two disks), bytes at offsets 1647 and 1649 changed from 0x01 to 0x00 (header ID 65537 → 0), along with the unit checksum; pointers and checksums for nodes pointing to it (operations 48 and 49) and the root slot (operation 52) also change; all subsequent tree table publications after stream two carry these differences (85 operations differ); call outcomes are identical; recovery states are identical | Selected new version (root txg not less than the first root written after the first difference): Stream one state 12 / 0, Stream two state 356 / 112 | O1 (memory model replay with HashMap ideal filesystem): does not read tree table; O2 (independent parser + checker): tree table entries only parse tree ID, type, and birth txg (`crates/singlefs-checker/src/walk.rs`'s `tree_table_entry_of_the_walk`), header ID is not validated; O3 (independent specification executor): projection carries only tree table unit pointers, A(c) is the root written by this row itself, "is in" | **True blind spot candidate**: D8 (core index structure) finalized item 8 regarding "value of 8 bytes for header ID: ... `deleted_inodes` and tombstone tree write inode tree's tree ID (65537)" (`.claude/kb/decisions/08-核心索引结构.md` "Finalized Item 8: Tree Table Entry Field Table"); selected new version state has header ID as 0, all three paths allow it |
| 277 (new root segment does not change after applying one prefix publication) | Write streams and call outcomes are identical; journal recovery view differs between Stream one batch 1 state 6 and Stream two batch 1 state 51 (batch 2 state 0): baseline applies 1 record, falls to a new version; mutation also applies 1 record but stops at the selected root version (e.g., Stream one state 41 baseline `1,3 file_read e8ca4bf8…`, mutation `1,2 no_file`) | 0: calls in the dropped version within state 57 did not return, and its root slot did not land (`extra_checks_f4.py`: all `call_returned=False` and `its_root_slot_landed=False`) | O1 (memory model replay with HashMap ideal filesystem) - crash judge: previous version was committed by the model, is not older than the latest persisted root slot, readback matches that version, green judgment is correct; O2 (independent parser + checker), O3 (independent specification executor): mirror and baseline are byte-for-byte identical | **Another category: three paths do not judge implementation recovery relying on record reconstruction**: calls did not return, both old and new versions are within "root + any journal prefix" (D16 (publication semantics) finalized item 12), states themselves are valid; error lies in implementation's failure to update the new root segment after applying records during recovery, violating D13 (verification path) finalized item 1 M4 and D23 (journal role and format) finalized item 15 (same as line 27); all three paths do not judge; M4's landing point is E165 (bounded exhaustive replay of recovery with mutation killing boundary) and Verus. Historical divergence that was pushed but not run: after a published version that has returned, root slot cannot be read back, recovery must rely on journal replay to catch up to that version (D16 (publication semantics) finalized item 7 range "step back one generation within same instance, caught up by journal replay"), mutation would stop at the previous version; no states in either stream involve reading a corrupted root slot (`call_returned=False its_root_slot_landed=False`) |

- **Line 233 (root reconstructed from records does not copy the filesystem identifier of the selected root, rewriting it to all zeros), only released in C**: write streams and call outcomes are identical; reading the journal during recovery fails on a state where records have been applied (`InvariantViolated`, `invariant` field is I-1.4 (consistent fsid in block header)), with detail "root fsid mismatch"; stream batch 1 has 3 states, stream batch 2 has 48 states (among which 6 states fall within unmounting, and the crash judge record does not apply). In A and B, O1 (memory model fuzzer (HashMap ideal filesystem)) - crash judge flags red (stream 1: 3 states, stream 2: 42 states; segments positive 2/12, 28/79), relying solely on O1 (memory model fuzzer (HashMap ideal filesystem)); C only takes O1 (memory model fuzzer (HashMap ideal filesystem)) - fuzzing, calls do not go through recovery, all three are released. In C, this is a **true blind spot candidate**: the implementation fails to recover any version from a valid crash mirror; D13 (verification route) has determined item 1's M2 write replay result root "other fields equal selected root" (``.claude/kb/decisions/13-verification-route.md` table row for M2 under "determined items 1"); O2 (independent parser + checker) judges the mirror before recovery (byte-for-byte identical to baseline), while O3 (independent specification executor) runs its own recovery. The oracle and record checker at this crash point flags red on this line (`layer0_or_record_checker_red=true`, not one of the three). Batch 2 does not flag red: neither of the two stream batches 2 has a state where "records have been applied" (`extra_checks_f4.py` outputs `b2_base_records_applied_gt0: 0`); **sampling point insensitive**, any sampling point that is a state with records applied would split; this cell flips between the two batches, marked as "unstable".
- **Line 265 (raising F sequence does not write system configuration first), batch 2 extra ones all released**: only stream 2 differs starting from operation 440 (the step of writing system configuration before raising F is entirely missing; 149 operations differ by index); O2 (independent parser + checker) flags red in batch 1 for 12 states, all I-7.12 (system configuration F not lower than F on the same disk root), with 6 states each in segments 59 and 71 (`name=trajectory row=265 stream=fixed_script_through_e_then_unmount oracle=O2（独立解析器 + checker） total_red_states=12 peak=6 positive_segments=2 of_segments=77 …`). Batch 2 takes segments 7, 11, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 72 in stream 2, excluding segments 59 and 71 (`extra_checks_f4.py` outputs `batch2_states_in_segments_59_71=0`): **sampling point insensitive**, same as line 102; marked "unstable". The clause is D16 (publish semantics) determined item 7 finalized: "raising F sequence... first write new F into system configuration of every disk, pass through a barrier." Conclusions for Q1 and Q3 do not flip in batch 2 (batch 2 fully releases 15 lines; lines relying on only one test case remain ≥ 1).
Source identifiers: `detail`.

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:189-205`（整段抄，未转述）**

```markdown
### Three Parts After the Triple Judgment

Three-round conclusion verdicts (`research/prompts/e171-conclusion-r1-main-verification.md`, `…-r2-main-verification.md`, `…-r3-main-verification.md`; the third round is the final round, with Section 3 "Conclusions After Three Rounds" and Section 5 as the standard) re-tripartite this batch of readings. Caliber: **True Blind Spot Candidates** = these states obtained in this batch are illegal under established clauses; all three allow passage (E171 (three-oracle collusion test) s4 tree); **Other Categories** = all three do not judge according to design, or the error does not lie with the state. The five lines originally judged by the specification executor as "equivalent for this batch" (lines 15, 27, 34, 112, 277) are no longer equivalent in any single line.

| Class | Line | One-sentence basis (the original text of each line's clause appears in the sections "F4 Expansion Results (entire lines copied from artifacts)" and "F4 Newly Issued Fully Released Lines") | Flips across three rounds |
|---|---|---|---|
| True Blind Spot Candidate | Line 15 | D3 (Space Allocation) Determined Item 10⑤ t2–t12 landing positions: under mutation, eleven submitted endogenous blocks are collectively shifted by 64 slots (`row15-offsets.out`); Item ⑤ and the line "Submitted Endogenous Blocks" in `layout/01-first-txn.md` pin these landing positions | Round 1 moved them in from "equivalent to this batch" (D3 (Space Allocation) Determined Item 10③); Round 2 maintained, based on writing Item ⑤ landing position table and the line "User Data Landing Position Policy" in `layout/01-first-txn.md` with t1@50180; Round 3 kept them on the list, basis changed to Item ⑤'s t2–t12 landing positions (main agent currently checks Item ⑤ table t2@50240 and Line 220 "Modified New Pool New File Bytes: Yes", re-running `row15-offsets.out` yields the same result) |
| True Blind Spot Candidate | Lines 23, 42, 271 | Line 23 D8 (Core Index Structure) Determined Item 6 container number; Line 42 D16 (Publish Semantics) Determined Item 1 unload lift F; Line 271 D8 (Core Index Structure) Determined Item 8 head ID | On the list across all three rounds |
| True Blind Spot Candidate | Lines 242, 269, 270 | Lines 242 and 270 D16 (Publish Semantics) Determined Item 7 write order; Line 269 D23 (Journal's Role and Format) Determined Item 14 witness value | Removed from the list in Round 1 (deemed legal per Determined Item 12, no one judged write order obligation); flipped back in Round 2; not hit in Round 3 |
| Another Category | Lines 27, 112, 277 | Three lines do not judge implementation recovery relying on record reconstruction (D13 (Verification Route) Determined Item 1 M4, D23 (Journal's Role and Format) Determined Item 14 Note 1, and Determined Item 15); Line 112 is already in `.claude/gate.d/e165-pinned-red-examples.tsv` | Round 1 adopted forward reasoning to reclassify into this category; Round 2 stood firm with narrowed wording; Round 3 unchanged |
| Another Category | Line 34 | Allocation policy, mirror legal: D3 (Space Allocation) Determined Item 8 range | Moved in as True Blind Spot Candidate together with Line 15 in Round 1; flipped back in Round 2; maintained in Round 3 |
| Another Category | Line 92 | D8 (Core Index Structure) Determined Item 6 has supplemented write time value (2026-10-04); three oracles do not judge per design, judged by pinned test cases | Round 1 classified as "clause undetermined and invisible," not on list; Round 2 changed reason; Round 3 maintained |
| Another Category (Zero Rounds) | Lines 110, 162, 174 | Runtime readings (write ledger, branch name); all three do not judge per design, same category as Line 34 | Newly emerged in Round 3: discovered when attacker quantity returned values ("Two lines red without reach and seven lines with different returned values among unreached lines" section) |
| Another Category (Zero Rounds) | Lines 49, 252 | Returned value or memory state error, write stream unchanged; requires longer history to distinguish | Newly emerged in Round 3: discovered when attacker quantity returned values ("Two lines red without reach and seven lines with different returned values among unreached lines" section) |

Lines 155 and 161 are the two known lines (O1 (memory model pair-checking (HashMap ideal file system)) - pair-check flag red), which do not fall into these categories; Line 233 (Category gamma allows all, Categories alpha and beta O1 (memory model pair-checking (HashMap ideal file system)) - crash judge caught) and the reading of Line 265 (Batch 2 sampling point insensitive) are in the section "F4 New All-Allow Lines"; the reading of Line 102 (Batch 2 sampling point insensitive) is at the end of the section "Three All-Allows Seven Lines", with no changes across three rounds. The meaning of the two lines under "Another Category (Zero Rounds)" is that this category emerged only in the third round, no further rounds were opened, there was an accumulation of debt, and the definition of touched registration was not modified.

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:206-226`（整段抄，未转述）**

```markdown
### F4 Newly Introduced Single-Threaded 12-Line Variant

Only write objects that are unreachable and fields that have not been flagged; do not judge the conclusion. Readings should be taken from F4's export directory (draft script `caught_rows_f4.py`, seventh segment `caught_rows_s4.py` modified to output root, not stored in database) and the product rows with `name=trajectory` and `name=row_matrix`; segments that are positive numbers represent "the number of red-state segments / the total number of segments for this stream".

| Line (Mutation) | Where the red flag is | Why the other two are not visible |
|---|---|---|
| 172 (The number of disks submitted equals w's lower bound and is also rejected) | O1 (Memory Model Replay (HashMap Ideal Filesystem)) - Replay: Stream 2 4th call to `mount_writable` returns `err:WritableDeviceCountBelowTheStripeWidthLowerBound`, model says it should pass; Stream 2 stops at op 92 | Writes before rejection match the baseline byte-by-byte (0 ops different); snapshots taken are prefixes of the same batch as the baseline; rejected calls do not leave snapshots, so O2 (Independent Parser + Checker) and O3 (Independent Specification Executor) have no objects to judge (same as Line 45) |
| 233 (Root rebuilt from records does not copy the fsid of the selected root) | O1 (Memory Model Replay (HashMap Ideal Filesystem)) - Crash Judge: Batch 1 Stream 1 State 3, Stream 2 State 42 read failure (I-1.4 (Block Header fsid Consistent) Tree Root fsid Mismatch), Segments Positive 2/12, 28/79; Batch 2 State 0 | Error is in the implementation's recovery; write stream matches baseline byte-by-byte; O2 (Independent Parser + Checker) judges snapshots before recovery, O3 (Independent Specification Executor) runs its own recovery. Columns red for Batch 1 and Batch 2 differ: Batch 2 has no states with records applied, sampling points are insensitive ("Section F4 New Full Release Lines") |
| 237 (Reading system configuration slot root ring start position is wrong) | O1 (Memory Model Replay (HashMap Ideal Filesystem)) - Crash Judge: Batch 1 Stream 1 State 53, Stream 2 State 73 (Segments Positive 12/12, 16/16), Batch 2 States 7, 14, "Did Not Select Root" (`SystemConfigurationValueRefused … RootRingBaseNotTheFirstVersionSlot`); O1 (Memory Model Replay (HashMap Ideal Filesystem)) - Replay: Stream 2 4th call to `mount_writable` returns `err:Recovery` | Error is on the reader side of the implementation; written bytes match baseline (0 ops different); O2 (Independent Parser + Checker) and O3 (Independent Specification Executor) each read system configuration with their own parser, seeing snapshots identical byte-by-byte to baseline |
| 249 (Publishing a data unit also splits into two records) | O1 (Memory Model Replay (HashMap Ideal Filesystem)) - Replay: Two streams 2nd call to `publish` judges `disagrees:jsn` (Model says that version's jsn is 3, Implementation is 4), model stops | Two records together still constitute one complete publish: O2 (Independent Parser + Checker) shows red at state 0 for this line (did not check if `crates/singlefs-checker/invariants/` limits how many records per publish); O3 (Independent Specification Executor) replays records itself, landing on a version in A(c), projection does not carry jsn |
| 250 (Payload bytes of data unit written in reverse order) | O1 (Memory Model Replay (HashMap Ideal Filesystem)) - Replay: Two streams 2nd call to `publish` judges `disagrees:this version's file content` (read back 3000 bytes, content reversed) | Unit checksum calculated based on written bytes, structure self-consistent; D13 (Verification Route) Item 6 Table O2 (Independent Parser + Checker) row "Cannot Verify" writes "A block overwritten with another valid content"; O3 (Independent Specification Executor)'s projection does not carry file content, does not compare against written content |
| 281 (Selecting root takes first self-attested slot and stops swapping) | O1 (Memory Model Replay (HashMap Ideal Filesystem)) - Crash Judge: Batch 1 Stream 1 State 41, Stream 2 State 61 (Segments Positive 9/12, 13/16), Batch 2 States 7, 14, "Walked to Instance 0 Generation 0 Root" (Older than latest root slot persisted on disk); O1 (Memory Model Replay (HashMap Ideal Filesystem)) - Replay: Stream 2 4th call to `mount_writable` returns `err:NewerStateStillUnreadableAfterOneReread` | Error is in the implementation's root selection; write stream matches baseline byte-by-byte (0 ops different); O2 (Independent Parser + Checker) judges snapshot structure, O3 (Independent Specification Executor) recovers using its own root selection rules, seeing snapshots identical to baseline |
| 135 (when resuming the defer in that one, the defer slot count does not decrease) | O2 (independent parser + checker): I-3.11 (allocated minus defer equals latest root walk) Batch 1 Stream Two 225 states, segment positive 44/79; Batch 2 70 states | O1 (memory model diffing (HashMap ideal file system)): `ObservedRoot` (its definition in `crates/singlefs-harness/src/model.rs`) has no accounting row; O3 (independent specification executor): projection does not enter the accounting tree |
| 234 (root record writer does not write flags according to unload mark) | O2 (independent parser + checker): I-7.9 (rollback lower bound F is not higher than raise F's upper limit) (split into two branches by unload mark) Batch 1 Stream Two 29 states, from segment 73 onwards segment positive 6/79; Batch 2 7 states | O1 (memory model diffing (HashMap ideal file system)): write falls within that unload call, the model has no unload answer (registration read method 6), diffing does not compare, crash judge record is inapplicable, `ObservedRoot` also does not carry root flags; O3 (independent specification executor): projection does not carry unload mark (`root_ring.py`'s `VersionProjection` explanation: "unload mark and remaining fields of root record do not enter projection") |
| 238 (modification count leaves 1) | O2 (independent parser + checker): I-9.26 (inode modification count falls between birth generation and container birth generation code) Batch 1 Stream Two 333 states, segment positive 65/79; Batch 2 105 states | O1 (memory model diffing (HashMap ideal file system)): does not compare inode record's modification count; O3 (independent specification executor): projection does not enter inode tree |
| 245 (reverse chain of overwrite write takes previous version record's own reverse chain) | O2 (independent parser + checker): I-8.6 (reverse chain algorithm) (records with unequal reverse chains do not enter replay) Batch 1 Stream Two 336 states, segment positive 67/79; Batch 2 105 states | O1 (memory model diffing (HashMap ideal file system)): `ObservedRoot` only carries journal counter, does not carry reverse chain; O3 (independent specification executor): its prefix rule core is reverse chain (`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/replay_prefix.py`'s `replay_prefix`, stops if unequal), this row strictly judges both temporary and current as "present"; after stopping, the version landed there remains in A(c) is a push (root slot has landed, does not rely on replay), did not check state by state where it stopped |
| 258 (does not release old landing points) | O2 (independent parser + checker): I-3.1 (allocated statistics match up), I-3.11 (allocated minus defer equals latest root walk) Batch 1 Stream Two 333 states, segment positive 65/79; Batch 2 105 states | O1 (memory model diffing (HashMap ideal file system)): `ObservedUnitAllocationRecords` only carries allocation records for each role's unit in this version (`model.rs`'s definition of `ObservedUnitAllocationRecords`), whether old landing points are released or not is not inside; O3 (independent specification executor): projection does not enter allocation record tree and accounting tree |
| 265 (raise F series does not write system configuration first) | O2 (independent parser + checker): I-7.12 (system configuration F is not lower than F on same disk root) Batch 1 Stream Two 12 states (6 states each in segments 59 and 71); Batch 2 0 states | O1 (memory model diffing (HashMap ideal file system)): compares F carried by root, does not compare system configuration slot; O3 (independent specification executor): projection's F is taken from root, A(c) is the root written by this row itself. Batch 1 and Batch 2 red columns differ: Batch 2 did not take segments 59 and 71, sampling point insensitive ("F4 new full release line" section) |

O3 (Independent Specification Executor) has no red flags on the line above row 37, where the denominator is included in F4 (`name=q1_column_red_rows … O3 (Independent Specification Executor) A=0 O3 (Independent Specification Executor) B=0`), and every row provides strict judgments for the batch state of two streams (`name=q1_o3_coverage … rows_with_no_strict_judgement=0`).

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:227-241`（整段抄，未转述）**

````markdown
### Two Lines That Are Red Despite Not Being Reached (Lines 155, 161) and Seven Lines Where Return Values Differ Within Unreached Lines

`name=untouched_but_red rows=2 row_numbers=155,161`: The mutations in both lines originate from the same location (`crates/singlefs-core/src/mount.rs`, where the F upper bound is raised by changing `newest_on_every_device.min(fourth_newest)` to `.max(…)`, with the only difference between the original two lines being leading whitespace). They trigger three distinct differences (`name=row_touch row=155 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false … touched=false`). O1 (memory model replay against an ideal file system using HashMap) detects a disagreement on the 12th invocation of `raise_floor` in stream two, flagging it as "raising the F upper bound" (`call-outcomes.tsv`: Model reports "upper bound Some(11) (D16 (publish semantics) item already determined)", Implementation reports "upper bound 13"). Both batch 1 and batch 2 are flagged red. O2 (independent parser + checker) and O3 (independent specification executor) do not flag red.

- Not a defect of the device, comparator, or sampling: The comparator's reach is calculated according to Registration Reading 4 literal (the "outcome of any single invocation (success / which error)", where `collusion_matrix.py`'s `outcome_words` compares only the invocation name and outcome column); both sides are `ok` for this invocation; O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - fuzzing compares the values returned by successful invocations (here, the upper limit of F raised), and the write stream remains unchanged because the F for this request does not exceed 11, so relaxing the upper limit had no effect. O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - fuzzing is calculated per invocation and is independent of sampling points; both batches are red.
- The gap lies in the reach definition of Registration Reading 4: it places "the value returned by a successful invocation" outside three items, yet writes "rows where all three are identical... behave byte-for-byte identically to the normal implementation in this batch, so no oracle should be red"; these two rows return different values, making that premise invalid. The device's `call-outcomes.tsv` records values from both sides only when fuzzing detects a difference; it does not separately record what successful invocations return, and the ingestion device lacks this column. The reach definition was not modified (the registration is not altered after running products), so the denominator remains 37.
- Are there similar differences in other unreachable rows: The attacker (Round 3) measured on a copy of the s4 tree (copy probe, s4 tree; the ingestion device lacks this column). The probe runs once on the memory-sparse disk driven by Stream Two, recording the `{:?}` text summary of the value returned by each successful invocation, along with the final allocator and the current version. Each line from the baseline and E171 (Three Oracle Collusion Test) exported 595 operations is checked for exact match; then unreachable rows are entered into the copy one by one, rebuilt and rerun, and restored. 232 lines were run (6 lines of `make_filesystem.rs` removed from the original 238 lines, as mkfs is not a single invocation and the probe does not record its return value; lines 149–154 were not executed). 7 lines differed: lines 49, 110, 155, 161, 162, 174, and 252. Lines 155 and 161 are the two mentioned earlier (this ruler caught them on the same 12th invocation of `raise_floor`; upper limit 11 → 13, serving as a ready positive control); the other 5 lines did not turn red in O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) because fuzzing does not compare those fields: line 49 returned witness value 4 → 3 for writable mount; line 110 concerns the structure type of write accounting; line 162 concerns the shadow account branch name; line 174 concerns write accounting returned by raising F; line 252 has a missing root slot registration in the allocator within memory (lines 49 and 252 differ in return values or allocators, but the write stream remains unchanged). The summary row is copied entirely from the attacker's `return-rows.out` under `name=return_summary`:

```
name=return_summary hidden_difference=7 identical=225 no_output=6
```

The main agent re-run (`tools/rerun.sh … all`) matches these 7 lines verbatim; `return-rows.out` differs only in the four `name=return_row` entries from rows 28–31 of the mutation table (the re-run shows `status=no_output`, while the summary is `hidden_difference=7 identical=221 no_output=10`; the attacker shows `identical`). The attacker had already executed these four lines before deciding to remove the line for `make_filesystem.rs`; the `awk` filter in `rerun.sh` removes them, so they are not re-executed (see section 4 of `research/prompts/e171-conclusion-r3-main-verification.md`). The harm is unquantified: lines 49 and 252 suggest that a longer history would be needed to distinguish the cases.
- Impact on other reads: these two lines do not enter the denominator, nor any Q1 or Q3 lists. Counting them under "reach" yields D = 39; both rows are flagged red  only for O1 (memory model oracle (HashMap ideal filesystem)), and Q3 gains exactly two additional rows relying solely on O1 (memory model oracle (HashMap ideal filesystem))—namely lines 155 and 161. The full set of passing rows remains unchanged for all three read types in Q1. Including the seven lines with differing return values plus line 125 ("it cannot answer") yields D = 45 instead of 37 (theoretical, arithmetic; not executed on the ingestion apparatus). Those five O1 (memory model oracle (HashMap ideal filesystem)) rows that are not flagged red would become new full-passing rows.

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:242-269`（整段抄，未转述）**

````markdown
### The Unpassable 6-Line Patch (Already Executed)

Register Section 1 Reading Rule 5: "Failing to compile after substitution" (`.claude/rules/mutation-sampling.md`, Category 8, line breaks according to the fill-in rules in 5.2); s4 and F4 versions of that driver only perform fill-ins on failures occurring before compilation; lines 33, 59, 127, 163, 206, and 243 remain marked as failing to compile after substitution without any fill-in. The eighth paragraph modifies the driver to also perform fill-ins after failing to compile (replacing with line j + 1 from the same file group, which is absent in both tables and has not been used for filling in during this run; the substituted line undergoes anchor verification before compilation again; if it fails to compile, replace once more; combining failures before compilation and failures after compilation allows at most three replacements per row; the group excludes E171 (the three-oracle collusion test) device's own mutated rows); add `--rerun-compile-failed-from` (run only those lines ending in failure to compile within a few output roots); the comparator adds `--substitution-rows-root` (replace the original failing-to-compile row with the same index during that fill-in pass; reject in other scenarios), resulting in two additional result rows: `name=substitution` and `name=substitution_summary`.

- The fill-in run executed on the frozen tree (frozen tree) from s4 (snapshot summary identical to s4's five runs, with an empty mutation row not reaching `baseline-run1`), running only those lines ending in compilation failure within two output roots (`rerun_compile_failed rows=33,59,127,163,206,243`); two lanes, each with 2 threads; all six steps regressed by 0; the fill-in completed 6 rows with a clock time of 1,277.9 seconds; the comparator merged 117 + 164 + fill-in into `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` (4,795 lines, last line `E7RESULT name=done emitted=4795`), with timing rows in `…-s4-f4-substitutes-rows-timing.out` (292 lines).
- The six fill-in rows (entirely copied from the six `name=substitution` lines within `…-s4-f4-substitutes-comparator.out`), the fill-in summary (`name=substitution_summary`), and the denominator (`name=q1_denominator rows_run=`):

```
E7RESULT name=substitution row=33 original_name=步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 original_status=compile_failed substituted_by=步_4：影子账把被抛弃根账里已释放的落点也隔离 failed_attempts=1 attempts=compile_failed:步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 category=untouched
E7RESULT name=substitution row=59 original_name=C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） original_status=compile_failed substituted_by=C483_②：挂载态打开时_inode_叶容器只按位置提示读（不经映射回退） failed_attempts=1 attempts=compile_failed:C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） category=untouched
E7RESULT name=substitution row=127 original_name=增补_2：回落只看已分配位（发出影子账隔离的槽） original_status=compile_failed substituted_by=增补_2_第_20c_行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放） failed_attempts=2 attempts=compile_failed:增补_2：回落只看已分配位（发出影子账隔离的槽）|compile_failed:增补_2：回落只看已分配位（抬_F_的空发布发出扣住的槽） category=untouched
E7RESULT name=substitution row=163 original_name=C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） original_status=compile_failed substituted_by=D8_已定项_8_②：可写挂载写行那次的树_ID_水位只沿所选那一版带，不取根环与环里记录的_max（原为_C511_第_3_步回退到暖机根那一格；回退改成向前发布之后由崩溃恢复丢掉一次第一个文件版本那一形护） failed_attempts=1 attempts=compile_failed:C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） category=untouched
E7RESULT name=substitution row=206 original_name=步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） original_status=compile_failed substituted_by=增补_3_第_4_件：一块盘两个系统配置槽都无效就整体_return_Err（每盘一份的冗余当场作废，D22_已定项_8_第_1_条） failed_attempts=1 attempts=compile_failed:步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） category=untouched
E7RESULT name=substitution row=243 original_name=并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 original_status=compile_failed substituted_by=普查_R2：释放判定路径把「映射条目切不动」报成「不在映射」（两件要分流的事并成一个成员） failed_attempts=1 attempts=compile_failed:并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 category=untouched
E7RESULT name=substitution_summary rows_that_did_not_compile=6 substituted_rows=6 still_not_compiled=0 row_numbers=33,59,127,163,206,243
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=238 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
```

- All six rows replaced have `category=untouched` (the row immediately preceding line 127 also failed to compile and was replaced a second time); the denominator D = 37 remains unchanged: `compile_failed` 6 → 0, `untouched` 232 → 238, for `name=f6_table_failures`, `rows_not_counted_for_the_table` 12 → 6 (only six rows remain self-collapsing). The additional rows in the patch product compared to the F4 product before patching are only these six rows with `name=row`, `row_touch`, `row_matrix`, `row_s5`, `row_o3`, and `trajectory`, plus six rows with `name=substitution` and one row with `name=substitution_summary`; the modified rows are `name=config`, `name=q1_denominator`, `name=f6_table_failures`, and `name=done`.
- The Q1, Q3 lists are identical character-for-character to those before patching (verify by diffing both product sets):

```
cd research/results; A=e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out; B=e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out; P='name=(q1_alpha|q1_beta|q1_gamma|q1_quiet_row|q1_column_red_rows|q3_alpha|q3_beta|untouched_but_red|q1_denominator_by_table|q1_delta|q1_o3_coverage) '; diff <(grep -E "$P" $A) <(grep -E "$P" $B); echo "diff_rc=$?"; grep -cE "$P" $A; grep -cE "$P" $B
```

Output `diff_rc=0`, `48`, `48`; lines starting with `name=q2_`, `name=f3_`, `name=verdict`, `name=f4_expansion`, `name=q1_denominator_by_table`, `name=positive_control`, `name=freeze_check`, `name=q3_core_row` are also processed line by line identically (the same command replaces `P` with `'name=(q2_|f3_|verdict|f4_expansion|q1_denominator_by_table|positive_control|freeze_check|q3_core_row)'`, outputting `diff_rc=0`, `171`, `171`).
- Re-run inputs for s4 and F4 using the modified comparator (without providing a padding root); the E7RESULT line must be identical byte-for-byte with both outputs; run an artificially created padding root once without execution (one row replaced by the export from line 34, one row still failing compilation), calculate denominators and lists based on the changed rows, and write both new result lines as drafts (not committed to storage).

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:270-299`（整段抄，未转述）**

````markdown
### Q2 allows O2 (independent parser + checker) while O3 (independent specification executor) flags the bad image as not present (the entire line is copied from the artifact)

`…-s4-f4-substitutes-comparator.out` lines 4759, 4760 (the same two lines as `…-s4-f4-comparator.out` before padding at lines 4662, 4663; both `sed -n '4662,4663p'` and `sed -n '4759,4760p'` fed to `md5sum` yield `07b56b24014dac5d9eb543ef15384aa9`):
Source identifiers: `md5sum`, `07b56b24014dac5d9eb543ef15384aa9`.

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o3_strict_red=22 o3_provisional_red=22 o2_only_strict=0 o3_only_strict=12 both_red_strict=10 o2_only_provisional=0 o3_only_provisional=12 both_red_provisional=10 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=B rows=40 o2_red=23 o2_quiet=17 o3_strict_red=40 o3_provisional_red=40 o2_only_strict=0 o3_only_strict=17 both_red_strict=23 o2_only_provisional=0 o3_only_provisional=17 both_red_provisional=23 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:18,new:19,new:20,new:23,new:26,new:28,new:30,new:32,new:33,new:35,fix:21,fix:22,fix:23,fix:24,fix:25,fix:32,fix:35
```

12 + 17 = 29 sheets are O2 (independent parser + checker) and pass, while O3 (independent specification executor) strictly judges them as "not present". From `name=q2_row`, count row by row for the same batch, categorized by patch type (command and original output; `B` is the previous product):

```
grep 'name=q2_row family=[PB] ' B | grep ' o2=green ' | grep -c 'o3_strict=not_in'
29
for f in P B; do echo "== $f"; grep "name=q2_row family=$f " B | grep ' o2=green ' | grep 'o3_strict=not_in' | sed -E 's/.*bad_name=([^ ]*) .*/\1/' | sed -E 's/^B[0-9]+:(P[0-9]+):.*/\1/; s/^(P[0-9]+)-.*/\1/' | sort | uniq -c; done
== P
      1 P2
      4 P3
      4 P4
      1 P5
      2 P9
== B
      4 P2
     12 P3
      1 P5
```

The P family consists of 12 instances, which are examples of five patches (P2, P3, P4, P5, and P9); the blind test B family comprises 17 instances that are random examples of type 3 (P2, P3, and P5); all are structurally consistent forged roots and forged continuation mirrors.

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:300-402`（整段抄，未转述）**

````markdown
### Baseline and Comparison of the First Paragraph (entire lines copied from s1 artifacts; lines removed from the workspace are written as single-line pointers)

`research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out` (baseline first pass; the E7RESULT rows across two passes are identical byte-for-byte, `diff` produces no output):

```
E7RESULT name=stream stream=new_pool_file_creation ending=completed exported=true not_exported_why=- operations=57 writes=41 segments=11 tearable_writes=8 calls=3 full_states=16777260 quick_tier_states=46 sampled_states=82 model_stopped_at_call=none
E7RESULT name=batch stream=new_pool_file_creation batch=1 states=53 segments_touched=12 states_with_a_torn_write=20 o1_crash_judge_red=0 o1_crash_judge_green=53 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=new_pool_file_creation batch=2 states=7 segments_touched=1 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=7 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=stream stream=fixed_script_through_e_then_unmount ending=completed exported=true not_exported_why=- operations=595 writes=477 segments=78 tearable_writes=46 calls=15 full_states=1662648564 quick_tier_states=278 sampled_states=890 model_stopped_at_call=14
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=1 states=397 segments_touched=79 states_with_a_torn_write=115 o1_crash_judge_red=0 o1_crash_judge_green=349 o1_crash_judge_not_applicable=48 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=2 states=119 segments_touched=17 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=105 o1_crash_judge_not_applicable=14 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
```

`e171-three-oracle-collusion-2026-10-03-s1-comparator.out`, baseline and control (Z1 two passes, 26 files byte-for-byte identical; S2 and E170 (first version of O3 specification executor) export device 18 items byte-for-byte identical; S5 two number comparators calculate themselves on the spot; a3 baseline 0 flag red):

```
E7RESULT name=a4 table_rows=117 expected=117 per_file_matches_the_registration=true found_once_in_today_table=117 line_unchanged_since_the_registration=117
E7RESULT name=a5 first_slice_rows=24 expected=24 distinct_files=24
E7RESULT name=a6 step_8_rows=117 step_4_rows=220 expected_step_4=220 overlap=56 expected_overlap=56 step_4_only=164 expected_step_4_only=164 step_8_names_equal_the_registered_table=true
E7RESULT name=a7 implementation_rows=844 expected=844 files=24 expected_files=24
E7RESULT name=a3_baseline_reds o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
E7RESULT name=s5_batch_1_states stream=new_pool_file_creation recomputed=53 device=53 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=1 recomputed=53 device=53 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=2 recomputed=7 device=7 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=3 recomputed=22 device=22 equal=true
E7RESULT name=s5_batch_1_states stream=fixed_script_through_e_then_unmount recomputed=397 device=397 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=1 recomputed=349 device=349 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=2 recomputed=105 device=105 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=3 recomputed=330 device=330 equal=true
E7RESULT name=z1_summary files_compared=26 files_differing=0
E7RESULT name=s2_summary files_compared=18 files_differing=0
```

Positive control (registration 5.6, each oracle runs once on baseline and once with an empty mutation) and self-check of turning off one switch:

```
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=q3_core closed=O1 run=disable-O1 differences_outside_the_closed_column=0 where=none
E7RESULT name=q3_core closed=O2 run=disable-O2 differences_outside_the_closed_column=0 where=none
```

Classification of three-line trial runs, reachability of three items, and the denominator for Q1:

```
E7RESULT name=row row=0 row_name=空变异（C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式）） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=0 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=1 row_name=C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=1 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=6 row_name=K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出） category=self_crash substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=6 write_stream_differs=true call_outcomes_differ=true recovery_report_differs=false ignored_pass_differs=false touched=true
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 102 行（name=q1_denominator）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 103 行（name=f6_table_failures）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 104 行（name=q1_o1_and_o2_both_quiet）
E7RESULT name=q1_o1_and_o2_both_quiet_batch_2 rows=0 rows_that_differ_from_batch_1=none
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 106 行（name=q1_column_red_rows）
E7RESULT name=untouched_but_red rows=0 row_numbers=none
```

Column for Q2's O2 (independent parser + checker) [one column], Column for O3 (independent specification executor) [two columns, blind test family not run]:

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 191 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 192 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 193 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 194 行（name=q2_blind_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 195 行（name=q2_negative_bases）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 196 行（name=a9_a10_a8）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 198 行（name=q2_all_quiet）
```

Judgment row (the comparator's own row; O3 (independent specification executor) always `not_run`):

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 199 行（name=verdict）
```

`e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out` (First cut, line 6: when finding leaves by slot number in the allocation record tree, leaf width is calculated as double, causing a panic during the first `publish`; the device records "self-collapse implementation", and this stream is not exported):

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 2 行（name=stream）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 5 行（name=call）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 14 行（name=stream）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 17 行（name=call）
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out` (41 known bad images on stream one full persistent state: the set of invariants equal to patching back and directly flagging red is identical, all target invariants are red, entry names match original files line by line):

```
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_build_pool local="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" crates="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" holds=true
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_the_full_state_of_the_dump local=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") crates=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") holds=true
E7RESULT name=s4 check=known_bad_test_file_sha256_is_the_copied_one local="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" crates="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" holds=true
E7RESULT name=s4 check=known_bad_entry_count_here local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_count_in_the_original_file local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_names_equal_the_original_file_in_order local=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] crates=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] holds=true
E7RESULT name=verdict known_bad_entries_failing_s4=0 e170_bad_rows_failing_s3_or_s4=0
```

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:403-409`（整段抄，未转述）**

```markdown
### Comparison and Self-Check

- Negative (Registration 5.6): On the baseline runs for batches two and three, across every state and invocation, O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Fuzzing, O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Crash Judge, and O2 (Independent Parser + Checker) Specification Executor all report zero red flags (a3, `name=a3_baseline_reds`); Empty Mutations (taking the first row from the table and replacing the replacement text with the original text itself, running through the complete line-by-line driver) are reported as Untouched, with no columns flagged red (`name=row row=0`, `name=row_touch row=0`, `name=row_matrix row=0`); Eight unpadded base states (two streams: F, Mk, Mk⁺, Mk⁻) under O2 (Independent Parser + Checker) show no red flags (`name=q2_negative_bases`). Under O3 (Independent Specification Executor), starting from the second paragraph: The frozen executor reports zero red flags on 576 states across the baseline two streams and batches 1 and 2; s4 output original line `E7RESULT name=a3_o3_baseline states=576 strict_in=576 strict_not_in=0 strict_spec_gap=0 strict_error=0 provisional_in=576 provisional_not_in=0 digest_mismatch_states=0 dump_problem_lines=0 base_unrecoverable_streams=none complete=true freeze_lines_equal_the_reference=true`; For the eight base states under O3 (Independent Specification Executor), neither party A nor party B reports any red flags (`name=q2_negative_bases … o3_strict_red_bases=0 o3_provisional_red_bases=0`). Before running the entire s4 table, an empty mutation on a single row of the same frozen tree is reported as Untouched for `baseline-run1` (Step 3 of the full-table script run by the main agent: `write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false touched=false`).
- Positive (Device side three, Registration 5.6): O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Fuzzing flips the last byte of that version's file content in the observation during the first `publish` on stream two, resulting in one detected red flag; O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Crash Judge flips the last byte of the read-back content at the state where "implementation reads back a file" occurs for the first time in each stream batch 1, resulting in one per stream and a total of two (the registration text says "first," but the device flips once per stream, so the required "red" status remains unchanged); O2 (Independent Parser + Checker) stacks the first known bad image family on all persistent states of stream one, resulting in one red flag. The baseline run and the empty mutation run are each executed once, with their red flags carried through to `oracle-verdicts.tsv` and `call-outcomes.tsv` (six rows for the comparator's `name=positive_control`). Positive control under O3 (Independent Specification Executor) (starting from the second paragraph): Stacking patch P1 on all persistent states of stream one and submitting to the frozen executor results in one red flag detected per run for both baseline and empty mutation (two rows in s4 output `name=positive_control oracle=O3（独立规约执行器）`). The fuzzing instance that was flipped causes the model to stop at that invocation; subsequent invocations are not compared, and writes falling after it are marked as "Not Applicable" by O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Crash Judge; therefore, `o1_crash_judge_red_states` for that run of O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Fuzzing is 0.
- Stop conditions: s1 local constants and crates pass all 42 comparisons; s2 the eight files of two streams and `impl-verdicts.tsv` (excluding the duration column) are byte-for-byte identical to those written by a `dump` on the same tree produced by E170 (first version of O3 specification executor) export device; s3 the set of invariants that trigger flag red  within one pass inside `check_pool_image` and `judge_crash_image` under O2 (independent parser + checker), which are directly invoked, matches exactly on every sampled state at baseline (`o2_matches_judgement_false=0`), and aligns row-by-row with the column from the pool-level checker in E170's (first version of O3 specification executor) `bad-impl-verdicts.tsv` across 76 bad states; s4 known bad image families: 41 entries match by name and original file, clean image digests equal those from `build_pool`, patch stack restoration yields the same set as direct modification of flag red , and all target invariants are flagged red; s5 two numbers computed independently by the comparator (state count and index set for batch 1, state count applicable to O1 (memory model replay with ideal HashMap filesystem) - crash judge) match the device's counts; s6 no trigger occurred: E170 (first version of O3 specification executor) export device returned 0 on a `dump` from the tree dated 2026-10-03, with 36 entries where `holds=true` and 0 where `holds=false`; s7 triggered in the first segment only (O3 independent specification executor column did not run); starting from the second segment, O3 (independent specification executor) used version 2 of registered revisions to copy frozen rows; both initial and final freeze pairings performed by the comparator in s4 showed zero differences across all lines; s8's three calls are referenced with line numbers in the section "Device, Research Side, and Mutation Table". The single pass of S5 (registered revisions 3, 4) failed on 0 rows at line 117 of s4: judgment row `rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0`, `name=q1_denominator … device_stop=0`.
- The tested tree is the one present at runtime: s4's empty mutations, line 117, and Q3-core three runs take snapshots from the same frozen tree (five driver-induced `snapshot_digest`s yield only one value: `d15864653cd83cedddc4902b6598d5db9cf590a62e6cbb8a54bed6661e4700cb`); pre-run tree verification compares it file-by-file against the manifest read during segment six, differing only in one other binary (`e168_delete_ring_window_borrow.rs`, which does not block). Baseline, control, and trial runs for the first segment use the state of `crates/` at runtime; E170 (first version of O3 specification executor) export device side uses a staged version extracted via `git show :crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` (sha256 `8c526293…`, the one pinned in section three of registration), built in the draft directory. Its `dump` and `bad-states` outputs are byte-for-byte identical to those from E170 (first version of O3 specification executor) in the workspace after the first re-run modification (`cmp` produces no output); therefore, subsequent runs directly use the version from the workspace.

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:410-415`（整段抄，未转述）**

```markdown
### Unit Testing and Mutation

- On the Python side, there are 110 unit tests (calculated via `grep -c '    def test_' test_*.py` in four files within the research directory: 32 + 29 + 24 + 25). Running `python3 -m unittest test_collusion_matrix test_collusion_pipeline test_o3_columns test_o3_comparison` consecutively for 5 rounds yields `Ran 110 tests` / `OK` each round (using handcrafted data and deterministic execution; the 5 rounds demonstrate only that there are no hidden states): matrix reachability (touching three items), O3 (independent specification executor) reading of two columns, full-pass predicates, turning off one rule, thresholds, trajectories; segment cutting and sampling formulas calculated on-the-fly; directory export read-back; bad implementation table (anchors, fillers, empty mutations); script line counts for selection methods; frozen tree pairwise checks; O3 (independent specification executor) judgment method for bad mirrors; compile records driven line-by-line with recompilation verification (true `rsync -a` and `copy2`, false cargo); F4 expansion table (difference set and numbering from two runs of the selection script, row-by-row comparison by comparator, merging of two output roots with rejection of renumbering, denominators separated by table, skipping filler names for two tables); post-failure fillers (handcrafted groups r0–r5, table contains r0 and r2, device rows inserted in between: upon compilation failure, sequentially switch to r1, r3, r4; if the fourth attempt fails to compile, record as failed-to-compile; switched-in row anchors that fail both anchor checks and compilation counts as three failures; empty mutations do not trigger fillers; groups contain no device rows; driver recompiles switched-in rows in the same directory, `compile-failed-1.log` matches `failed_attempt_1` in naming, re-run switch does not trigger filler; comparator only replaces failed-to-compile matching-numbered rows with filler rows, rejecting other cases; result rows for filler rows are written with the name of the switched-in row). Every predicate that could be broken by mutation is paired with an assertion using absolute values (e.g., expansion table on handcrafted data must include three lines: `(118, a4)`, `(119, a12)`, `(120, b2)`; entire filler rows `name=substitution row=3 … substituted_by=c_换 failed_attempts=1 …` are hardcoded).
- On the Python side, 67 mutations were identified in table `scripts/mutation/tables/e171_three_oracle_collusion.tsv`, executed via `mutation_driver.py` (Section 9 of the register notes "run via `scripts/mutation/mutate.sh`"; that script only runs cargo test and cannot execute Python unit tests; Register Revision 1, line 1 records this discrepancy): within `e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out`, the result is `E7RESULT name=verdict mutations=67 caught=67 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`, yielding 67 caught / 0 invalid / 0 not flagged red (outcome symbols counted individually: ✅ 67, ⏭ 0, ❌ 0, ⏱ 0, 💥 0, ⚠️ 0, 🧱 0). Mutations x60–x74 belong to the batch that failed to generate substitutes for fill-in positions (failures without fill-ins are not counted; groups count them as device rows; replaced row anchors do not expire outcomes if they fail generation; fill-in runs receive rows that were not generated by failure attempts; names of attempted tests are not recorded; after failing generation, the device runs directly on the next line without replacing it; the comparator does not verify original lines for generation failures; names of self-generated fill-ins are not verified; result rows retain original names; totals treat unfillable rows as replaced; row analysis does not apply to fill-ins; log entries and attempted test names do not match; re-run switches are ineffective; in a single fill-in run, the same line is not rejected across two roots). The similarly named `…-r6.out` represents a full-table run of the first 13 items added to that batch (65 mutations total, 65 / 0 / 0); subsequently, x73 and x74 were appended and archived. One considered but unadded rule regarding "the three categories not flagged red": after failing generation, search for fill-ins starting from the original name in the table rather than from the failed row—fill-in skips names already used in the table; all names between the original name and the failed row fall into both categories; both search methods yield identical results across all inputs, constituting equivalent mutations; thus, this was not added to the table but recorded as an equivalence test (`test_walking_from_the_name_in_the_table_or_from_the_last_failed_name_gives_the_same_substitute`, manually verified on each starting point for the group); re-running the mutation table after adding this test produced output byte-for-byte identical to `-r7.out`. Mutations x52–x59 belong to the F4 table expansion batch (expansion does not remove overlapping rows; numbering starts at 117; device-generated mutation rows are included in selection criteria; two output roots with matching signatures are not rejected; fill-in does not skip other tables; expansion verification compares only row counts; denominators are swapped by table; only the first output root is read). Each was caught by its corresponding unit test within `F4ExpansionTests`. The similarly named `…-r4.out` represents a mid-run termination of that same table (the outer `timeout 230` halted execution at mutation #8, resulting in 61 rows with no verdict row); this run is not counted but archived. Earlier runs `…-s1-comparator-mutations.out`, `…-s2-comparator-mutations.out`, `…-s3-comparator-mutations.out`, and `…-s3-comparator-mutations-2026-10-03.out`, along with `-r2.out` and `-r3.out`, represent full-table executions at mutation counts 9, 26, 37, 40, 44, and 44 respectively, each capturing all mutations.
- Device-side inline unit tests: 31 cases (counted via `grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`): compiled (`cargo build` and `cargo test --no-run` via `scripts/hosts/compile-then-swap.py`) but not executed (checker artifacts, pending submission). Device-side mutations: 15 rows appended to the end of `crates/mutations.tsv` (names prefixed with "E171 (Three Oracle Collusion Test) Device"); three metrics await gate verification upon checker-tier-crates-mutation-replay submission.

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:416-864`（整段抄，未转述）**

````markdown
### First Re-run of the First Segment · Revision 6 Run (Completed: Quantity of the first segment finished, Q5 alarm self-verification passed; flagged as "alpha"; Q2's set and Q3's denominator are in the second section)

Register `research/prompts/e171-r1-prereg.md` (revisions 6, 7, and all subsequent "main agent determinations"; the final entry is "`-resume2` cross-machine consistency check S17 shutdown handling"); issue tracker `e171-r1-questions.md` Q1–Q5. The tree is revision 6 tree `scripts/experiment-apparatus/target/e171-r1-frozen-tree-revision-6/` (snapshot summary `7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4`, 284 files; the first new tree only swaps devices, without batch 31 core assertions); device `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` in that tree has sha256 `67e7a549…`. Steps 0–10 have suffix `-r6-r3`; steps 11, 12 (16 lines) run on multiple machines with suffix `-resume2`; step 13 was not started (timeout 0 lines). O3 (independent specification executor) is the v2-487 of E170 (first version of O3 specification executor) second rerun (register revision 1). The determinations for that run of revision 6 belong only to the revision 6 tree; the run for revision 8 (final main line) has not been executed yet, so no numbers are pushed onto it. The comparator reads only this run's output prefix `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-`, suffix `-r6-r3.out`, and export root `scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/`; providing any other prefix causes exit (register revision 8 section 8.5, item 1).

| Line | Verdict (Revision 6 Tree) | Basis (Artifact) | What to Read Alongside Beyond the Literal Text |
|---|---|---|---|
| q1 | alpha: Seven rows have at least one red flag on each of these three streams in this batch (Batch 1 verdict is alpha; both strict and temporary O3 (independent specification executors) readings are alpha; all seven rows reached, all three streams completed, invalid rows = 0) | `…-s1-comparator-resume2-r6-r3.out` with `name=q1 reading=strict streams=three_streams batch=1 … verdict=jia` | Where the red flags appear: Row 15 O2 (independent parser + checker), three streams, I-5.5 (t1 landing point from the mkfs water-level tree-building version), Row 23 O2 (independent parser + checker), stream three only, I-1.1 (block header self-declared logical address), I-9.4 (container number does not exceed minimum key), I-9.12 (separator key falls outside child interval), Row 42 O1 (memory model fuzzer (HashMap ideal filesystem)) - fuzzer disagreement (stream two, 14th call to `unmount`, `disagrees: root carries F`), Rows 242, 269, 270 O3 (independent specification executor), strict and temporary counts match, Row 271 O2 (independent parser + checker), three streams, I-9.27 (tree table entry header ID value). When taking only stream one and stream two (consistent with s4's batch history scope), Row 23 has no red flags in all three columns (`streams=two_streams … all_quiet_rows=23 verdict=yi`): Row 23 is caught only by that segment spanning leaves across streams. Batch 2 verdict is also alpha; six cells flip between the two batches (see section "q1 cell-by-cell (Revision 6 run)"). O1 (memory model fuzzer (HashMap ideal filesystem))'s two crash sub-sequences have no red flags in any of the seven rows |
| Q2 | First segment reports only seven rows (description quantity); second segment verdict is beta for the entire table ("First re-run of second segment · Revision 6 run" section) | `name=q2_named_row` 14 rows (entire row copied into "Q2 Seven Rows, Comparison with Z1 (Revision 6 Run)" section) | The seven rows were not reached on stream one or stream two; Rows 155 and 161 each had one call returning different values, glue passed to model observations also differed, O1 (memory model fuzzer (HashMap ideal filesystem)) - fuzzer red flag once; Rows 49, 110, 162, 174, 252 returned different values or memory objects but had identical observations, O1 (memory model fuzzer (HashMap ideal filesystem)) - fuzzer no red flag |
| q3 | alpha: Row 125 counts toward the denominator (c = 12, reached) | `name=q3 row=125 call_panic=true judged_calls_before_the_first_panic=12 … touched=true verdict=jia` | Panic occurred on stream two during the 14th call to `unmount` (`attempt to subtract with overflow`); prior calls 2–13 O1 (memory model fuzzer (HashMap ideal filesystem)) - fuzzer all `agrees`; stream one had no panic, comparator counts only the stream with panic (counting stream one's single `agrees` as well yields 13; both readings ≥ 1, verdict unchanged). Denominator D = 39 comes from the entire second segment ("First re-run of second segment · Revision 6 run" section), where Row 125 is also counted |
| Q4 | Baseline A ((ii) state 0); line 242 B ((ii) state 12, second segment entirely flagged red ); line 269 B ((ii) state 58, entirely flagged red ) | `name=q4_trip` three lines (the entire lines copied into the code block following the judgment row in the section "First re-run of Segment One · Revision 6") | Three runs × three streams × two batches, 18 cells each probed (`cells_without_a_probe=0`). Q4 reconciliation: both numbers for all 18 cells are 0 (`witness_refusals_without_either_branch=0`, `counter_branch_mounted=0`); S11 not triggered. (i) Unable to determine which branch reported separately: Baseline 1 state 340, line 242 state 2650, line 269 state 1340 (sum of both batches, `undecidable_branch=`), not entering candidates. (ii) The per-state list and the trajectories arranged by segment and by constructed inode are in the section "Q4-(ii) Per-State List and Q4, Q5 Trajectories (Revision 6 Run)" |
| Q5 | A, alarm self-verification passed (the number of RefFS red states at both the primary sampling point and the reverse sampling point equals L; Model Collapse Judge state 0 red, both points judged identically) | `name=q5_judged` two lines (the entire lines copied into the code block following the judgment row in the section "First re-run of Segment One · Revision 6"); alarm self-verification see the last item in the section "Valid Conditions and Q5 (Revision 6 Run)" | Alarm self-verification required by Section 5.6, Item 7 of Revision 5 and Section 6.6 of Revision 6 (U1–U9, U13–U16 plus two original inline test cases green, `crates/mutations.tsv` captures all V17–V31, V33–V36; U10–U12, V32 not included in this item) run in advance by the main agent with `SINGLEFS_HEAVY_TESTS=user-request`, valid (registered at the end of "Main Agent Determination: Segment One (Revision 6 Tree) Q5 Alarm Self-Verification"). The L at the primary sampling point is derived via equivalence classes (`derived_by=equivalence_classes judged_states=144`); equivalence relations verified per-state on the 4,608 states where k = 6 (`mismatching_states=0`), remaining states checked by class against A's derivation in Revision 5 (Section 5.2 of Revision 5). Trajectories arranged by write at the end are in the section "Q4-(ii) Per-State List and Q4, Q5 Trajectories (Revision 6 Run)" |

The comparator's verdict line (the `name=verdict` line in `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out`; the `name=verdict` line in `…-s1-comparator-resume2-r6-r3-2026-10-07.out` after adding the state-by-state list and trajectory, which is identical to it byte-for-byte; this section's copied verdict line appears alongside Q1, Q3, Q4, Q5, negative, and coverage lines in both files):

```
E7RESULT name=verdict q1=jia q3=jia q4_baseline=jia q4_row_242=yi q4_row_269=yi q5_main=jia q5_reverse=jia q5_same_at_both_sampling_points=true a1_baseline_red=0 a8_failing_controls=0 a11_failing_streams=0 z1_differing_files=0 f7=false a1_empty_mutation_red=0 empty_mutation_touched=false
```

The same artifact's Q1, Q3, Q4, Q5 and negative coverage for each row (copying the entire row):

```
E7RESULT name=a1 batch=1 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1 batch=2 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=o3_third_stream_coverage stream=first_file_unlinked_then_inodes_across_one_leaf sampled_states=6459 strict_in_or_not_in_states=6459 f7=false
E7RESULT name=a11 stream=new_pool_file_creation states=82 crash_judge_applicable_device=82 crash_judge_applicable_recomputed=82 reference_filesystem_voting_device=82 reference_filesystem_voting_recomputed=82 holds=true
E7RESULT name=a11 stream=fixed_script_through_e_then_unmount states=890 crash_judge_applicable_device=890 crash_judge_applicable_recomputed=890 reference_filesystem_voting_device=890 reference_filesystem_voting_recomputed=890 holds=true
E7RESULT name=a11 stream=first_file_unlinked_then_inodes_across_one_leaf states=11695 crash_judge_applicable_device=11695 crash_judge_applicable_recomputed=11695 reference_filesystem_voting_device=81 reference_filesystem_voting_recomputed=81 holds=true
E7RESULT name=a1_empty_mutation batch=1 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1_empty_mutation batch=2 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1_empty_mutation_touch row=0 streams_present=3 touched=false
E7RESULT name=q1 reading=strict streams=three_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=strict streams=three_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=strict streams=two_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=strict streams=two_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=provisional streams=three_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=provisional streams=three_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=provisional streams=two_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=provisional streams=two_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q3 row=125 call_panic=true judged_calls_before_the_first_panic=12 states_judged=514 recovery_panic_states=0 touched=true verdict=jia
E7RESULT name=q4_trip trip=baseline counter_branch=0 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=jia
E7RESULT name=q4_trip trip=row_242 counter_branch=12 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=yi
E7RESULT name=q4_trip trip=row_269 counter_branch=58 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=yi
E7RESULT name=q5_judged sampling_point=main lost=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 q5_negative_o1_red=0 a10_states_equal_the_product=true reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 verdict=jia
E7RESULT name=q5_judged sampling_point=reverse lost=9 reference_filesystem_red=9 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 q5_negative_o1_red=0 a10_states_equal_the_product=true reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 verdict=jia
```

#### Q1 Step-by-Step Grid (Revision 6 Run)

Batch 1: For each row and each stream, the columns marked red (O1 (memory model fuzzer (HashMap ideal file system)) - fuzzing by call, others by state; parentheses contain traces: peak, number of positive segments, end value, number of single-point-red segments). Commands run at the repository root (only read `name=q1_row … batch=1` for those 21 rows, outputting non-zero columns):

```
grep 'name=q1_row .* batch=1 ' e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out | awk '{o=""; for(i=1;i<=NF;i++){split($i,a,"="); f[a[1]]=a[2]} if (f["o1_comparison_red_calls"]!="0") o=o" O1-对拍="f["o1_comparison_red_calls"]; n=split("O1-模型崩溃判官 O1-RefFS O2 O3甲 O3乙",c," "); for(j=1;j<=n;j++){k=c[j]; if (f[k"_red"]!="0") o=o" "k"="f[k"_red"]"(峰"f[k"_peak"]",正段"f[k"_positive_segments"]",末"f[k"_last"]",单点"f[k"_single_points"]")"} print f["row"], substr(f["stream"],1,9) (o==""?" 全不红":o)}'
```



```
15 new_pool_ O2=9(峰8,正段2,末1,单点1)
15 fixed_scr O2=240(峰8,正段47,末0,单点11)
15 first_fil O2=480(峰8,正段96,末0,单点24)
23 new_pool_ 全不红
23 fixed_scr 全不红
23 first_fil O2=29(峰8,正段6,末1,单点2)
42 new_pool_ 全不红
42 fixed_scr O1-对拍=1
42 first_fil 全不红
242 new_pool_ O3甲=4(峰4,正段1,末0,单点0) O3乙=4(峰4,正段1,末0,单点0)
242 fixed_scr O3甲=8(峰4,正段2,末0,单点0) O3乙=8(峰4,正段2,末0,单点0)
242 first_fil O3甲=4(峰4,正段1,末0,单点0) O3乙=4(峰4,正段1,末0,单点0)
269 new_pool_ O3甲=8(峰4,正段3,末0,单点1) O3乙=8(峰4,正段3,末0,单点1)
269 fixed_scr O3甲=41(峰8,正段10,末0,单点3) O3乙=41(峰8,正段10,末0,单点3)
269 first_fil O3甲=8(峰4,正段3,末0,单点1) O3乙=8(峰4,正段3,末0,单点1)
270 new_pool_ O3甲=3(峰3,正段1,末0,单点0) O3乙=3(峰3,正段1,末0,单点0)
270 fixed_scr O3甲=30(峰3,正段10,末0,单点0) O3乙=30(峰3,正段10,末0,单点0)
270 first_fil O3甲=714(峰3,正段238,末0,单点0) O3乙=714(峰3,正段238,末0,单点0)
271 new_pool_ O2=9(峰8,正段2,末1,单点1)
271 fixed_scr O2=353(峰8,正段69,末1,单点17)
271 first_fil O2=4749(峰8,正段950,末1,单点238)
```

- Regarding registration reading method 12 (C673, where the fix for the three-oracle collusion test's supplementary oracle modification was not implemented; this is the implementation point): Line 15 ← ⑥ O2 (independent parser + checker) new pool file implementation point, O2 (independent parser + checker) flag red (I-5.5 (implementation point from the t1 version of building a tree based on mkfs water level)); Line 23 ← ⑦ Stream Three, O2 (independent parser + checker) flags red only in Stream Three (I-1.1 (block header self-declared logical address), I-9.4 (container number does not exceed minimum key), I-9.12 (separator key falls outside child interval)); Line 42 ← ④ Model unmount answer, O1 (memory model replay (HashMap ideal file system)) - replay flagged red during the `unmount` in Stream Two; Lines 242, 269 ← ② O3 (independent specification executor) predicate W, O3 (independent specification executor) flags red; Line 270 ← ③ O3 (independent specification executor) witness value not lower than tail, O3 (independent specification executor) flags red; Line 271 ← ⑤ O2 (independent parser + checker) tree table entry header ID, O2 (independent parser + checker) flags red (I-9.27 (tree table entry header ID value)). All seven implementation points flagged red in this batch. The names of the O2 (independent parser + checker) invariant that flagged red are derived from columns 8 and 9 of `oracle-verdicts.tsv` exported by each row (totaling two batches).
- Batch 1 vs. Batch 2 (Section 8 Geometric Sensitivity): Command `grep 'name=q1_row' <same artifact> | awk '{for(i=1;i<=NF;i++){split($i,a,"="); f[a[1]]=a[2]} n=split("o1_comparison_red_calls O1（内存模型对拍（HashMap 理想文件系统））-model crash judge_red O1（内存模型对拍（HashMap 理想文件系统））-RefFS_red O2_red O3（独立规约执行器）甲_red O3（独立规约执行器）乙_red",c," "); for(j=1;j<=n;j++){k=f["row"]" "substr(f["stream"],1,9)" "c[j]; r[k, f["batch"]]=(f[c[j]]!="0"); keys[k]=1}} END{t=0; m=0; for(k in keys){t++; if (r[k,1]!=r[k,2]) {m++; print k, "batch1_red=" r[k,1], "batch2_red=" r[k,2]}} print "cells=" t, "flipped_between_batch_1_and_2=" m}' | sort`, output last line `cells=126 flipped_between_batch_1_and_2=6`. The 6 flipped cells are Line 15 Stream One O2 (independent parser + checker), Line 271 Stream One and Stream Three O3 (independent specification executor) strict, temporary substitute — these 6 cells are recorded as "unstable" (the segments that Batch 2 did not flag red); other cells still flagged red on each row in Batch 2; Q1 Batch 2 still judges Alpha.
- The flags red for Lines 23 and 42 occur only on a single stream and one oracle type; Line 42 has only 1 call (O1 (memory model replay (HashMap ideal file system)) - replay calculated by call count, no trajectory).
- In the three per-row artifacts, Lines 252 and 271 have `line_matches_the_registration=false` (the line's sha256 differs from registration 5.9; run according to that row in the frozen tree and record separately); the remaining fourteen rows are `true`; `substituted_from=-`, `failed_attempts=0`, no supplementary positioning.

#### Q2 Seven Lines, Comparison, and Z1 (Revision 6 Run)

```
E7RESULT name=q2_named_row row=49 streams=streams_1_2 touched=false calls_with_a_difference=11 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=11 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=49 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=110 streams=streams_1_2 touched=false calls_with_a_difference=14 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=14 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=110 streams=stream_3 touched=false calls_with_a_difference=238 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=238 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=155 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=1
E7RESULT name=q2_named_row row=155 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=161 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=1
E7RESULT name=q2_named_row row=161 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=162 streams=streams_1_2 touched=false calls_with_a_difference=11 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=11 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=162 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=174 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=1 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=174 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=252 streams=streams_1_2 touched=false calls_with_a_difference=14 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=14 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=252 streams=stream_3 touched=false calls_with_a_difference=238 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=238 o1_comparison_red_calls=0
E7RESULT name=a8 control=O1-对拍 products=1 counted=o1_comparison_red_calls expected=1 observed=1 holds=true
E7RESULT name=a8 control=O1-对拍-卸载 products=1 counted=o1_comparison_red_calls expected=1 observed=1 holds=true
E7RESULT name=a8 control=O1-崩溃判官 products=1 counted=o1_crash_judge_red_states expected=3 observed=3 holds=true
E7RESULT name=a8 control=O2 products=1 counted=o2_red_states expected=1 observed=1 holds=true
E7RESULT name=a8 control=O3 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=RefFS products=1 counted=o1_reference_filesystem_red_states expected=3 observed=3 holds=true
E7RESULT name=a8 control=panic products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=交回值 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=内存对象 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=见证 products=1 counted=counter_branch_states expected=1 observed=1 holds=true
E7RESULT name=z1_unstable_memory_objects names=-
```

- Each invocation of the line-by-line method (`name=q2_named_row_call`, specifying which top-level fields and which memory object differ) within the same artifact occurs at line 529 without copying; in Stream Three, among 238 invocations on lines 110 and 252, all involve writes differing from `current_version`.
- The four cases with `checked=false` for a8 (O3 (independent specification executor), panic, returned value, memory object) are evaluated by the verdict logic of Step 3 of the sequential execution method (`…-s1-positive-control-o3-p1-r6-r3.out` with `o3_control_verdict strict=not_in base_digest_matches=true red_strict=true`, covering one core each for returned value and memory object invocation comparison, and panic truncation); the empty mutation run (ten items in `…-s1-empty-control-*-driver-r6-r3.out`) is also judged by Step 4, with the comparator not reading them.
- Z1: All 39 lines with `name=z1` have `identical=true` (`grep -c 'name=z1 .*identical=true'` and `grep -c 'name=z1 '` both yield 39), indicating no unstable memory objects between two baselines (`names=-`).
Source identifiers: `writes`, `o3_control_verdict strict=not_in base_digest_matches=true red_strict=true`.

#### Q4-(ii) State-by-State Manifests and Q4, Q5 Trajectories (Revision 6 Run)

Section 6, Item Q4-(ii), the state-by-state list and Section 8, Items Q4 and Q5 trajectories are completed by the comparator in `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out` (811 lines). Compared to `…-s1-comparator-resume2-r6-r3.out` (679 lines): the first 677 lines match byte for byte, as do the verdict lines (the new file's verdict is at line 810); the new file adds 132 lines after the Q5 judgment and before its verdict line (`name=q4_counter_branch_state` 70, `name=q4_trajectory` 18, `name=q4_trajectory_by_inode_creation` 6, `name=q5_trajectory_cell` 36, `name=q5_trajectory` 2); the final line changes from `E7RESULT name=done emitted=679` to `E7RESULT name=done emitted=811` (`diff` reports only `677a678,809` and `679c811`). The three trajectory metrics (peak value and the cell where it first appears, count of positive cells, and end-of-period value) are reported as "endpoint is not a trajectory"; the end-of-period value corresponds to the cell with the largest cell number: when Q4 is arranged by segment, this is the last segment obtained in this batch; when arranged by inode creation, this is the 235th inode creation; for Q5, it is the cell where the final segment fully falls.
Source identifiers: `679c811`.

The per-state list is merged by grid (column states: pass, flow, batch, segment; this segment issued on which call invocation, counting which branch corresponds to which line, mount outcome, second-segment judgment; each state has its own serial number, status code, landed string's SHA256, image digest, c_witness, selected version, rejected members, reasons on the monad and unsatisfied conditions, in the artifact):

```
grep 'name=q4_counter_branch_state ' <新那份> | awk '{o=""; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="trip"||a[1]=="stream"||a[1]=="batch"||a[1]=="segment"||a[1]=="call_of_the_segment"||a[1]=="counter_sentence"||a[1]=="mount"||a[1]=="second_crop") o=o" "$i}; print o}' | sort | uniq -c | sort -k2,2 -k3,3 -k4,4
      2  trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 segment=14 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      2  trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 segment=14 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_242 stream=new_pool_file_creation batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=new_pool_file_creation batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=18 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=21 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=15 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      3  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=17 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      3  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=19 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      8  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=16 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      8  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=20 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      7  trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 segment=16 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      7  trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 segment=20 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=new_pool_file_creation batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=new_pool_file_creation batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=new_pool_file_creation batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=new_pool_file_creation batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
```

The 0th and 1st calls of the three streams are for obtaining a ticket number and warming up; the 4th call of stream two is for writable mounting (column 2 in each `calls.tsv`). Conditions not met are matched one-to-one with the counting branch: `RecordAtTheWitnessedCounterIsNewer` corresponds to `witness_criterion_true_without_read_faults:RecordAtTheWitnessedCounterNewerThanTheSelectedVersion`, `AboveTheLastRecordOfTheSelectedVersion` corresponds to `…:WitnessedCounterAboveTheLastRecordOfTheSelectedVersion`, and `NeitherRecordReadable` corresponds to `…:NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable`. The number of states listed in each cell equals the value of `counter_branch=` on the row with `name=q4` for that same cell (18 cells match, 0 cells do not); similarly, the sum of trajectories across all cells also equals this value (18 cells match, 0 cells do not).
Source identifiers: `name=q4`, `counter_branch=`.

Arranged by segment (rows 242 and 269; all six baseline rows are zero):

```
E7RESULT name=q4_trajectory trip=row_242 stream=new_pool_file_creation batch=1 cells=segments segments=9 first_segment=0 last_segment=8 counter_branch_by_cell=2:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=new_pool_file_creation batch=2 cells=segments segments=4 first_segment=2 last_segment=7 counter_branch_by_cell=2:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 cells=segments segments=60 first_segment=0 last_segment=59 counter_branch_by_cell=2:2,14:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 cells=segments segments=36 first_segment=2 last_segment=58 counter_branch_by_cell=2:1,14:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 cells=segments segments=720 first_segment=0 last_segment=719 counter_branch_by_cell=2:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 cells=segments segments=478 first_segment=2 last_segment=718 counter_branch_by_cell=2:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=new_pool_file_creation batch=1 cells=segments segments=12 first_segment=0 last_segment=11 counter_branch_by_cell=0:2,1:3,2:1 counter_branch_peak=3 counter_branch_peak_at=1 counter_branch_positive_cells=3 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=new_pool_file_creation batch=2 cells=segments segments=1 first_segment=7 last_segment=7 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 cells=segments segments=79 first_segment=0 last_segment=78 counter_branch_by_cell=0:2,1:3,2:1,15:2,16:8,17:3,18:1,19:3,20:8,21:1 counter_branch_peak=8 counter_branch_peak_at=16 counter_branch_positive_cells=10 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 cells=segments segments=17 first_segment=7 last_segment=74 counter_branch_by_cell=16:7,20:7 counter_branch_peak=7 counter_branch_peak_at=16 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 cells=segments segments=960 first_segment=0 last_segment=959 counter_branch_by_cell=0:2,1:3,2:1 counter_branch_peak=3 counter_branch_peak_at=1 counter_branch_positive_cells=3 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 cells=segments segments=238 first_segment=7 last_segment=955 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
```

Stream three: Arrange by the number of times an inode is created (235 times, one cell per creation; segments not in the create_inode call are recorded in state (ii) as `counter_branch_outside_create_inode_calls`, and the fully persistent segment is recorded as `counter_branch_after_every_call`):

```
E7RESULT name=q4_trajectory_by_inode_creation trip=baseline stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=baseline stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=2 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=1 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=6 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
```

Q5 How many are written in the final segment arranged by row (primary sampling points: 30 cells, reverse sampling points: 6 cells; for the device's `name=q5_trajectory` line, only cells with L state appear, and the comparator fills from cell 0 to the last cell containing a write value, marking missing ones as 0; the sum of each cell is cross-checked against the L values and two red numbers in the `name=q5` row):

```
E7RESULT name=q5_trajectory sampling_point=main cells=30 last_segment_writes=29 cells_add_up_to_the_q5_line=true reference_filesystem_red_by_cell=0:1,1:28,2:376,3:3224,4:19826,5:93104,6:347116,7:1054504,8:2658271,9:5633804,10:10133156,11:15571184,12:20533756,13:23297344,14:22762456,15:19137104,16:13810511,17:8518004,18:4460896,19:1965304,20:719026,21:214544,22:50876,23:9224,24:1201,25:100,26:4 reference_filesystem_red_peak=23297344 reference_filesystem_red_peak_at=13 reference_filesystem_red_positive_cells=27 reference_filesystem_red_last=0 model_crash_judge_red_by_cell=- model_crash_judge_red_peak=0 model_crash_judge_red_peak_at=- model_crash_judge_red_positive_cells=0 model_crash_judge_red_last=0
E7RESULT name=q5_trajectory sampling_point=reverse cells=6 last_segment_writes=5 cells_add_up_to_the_q5_line=true reference_filesystem_red_by_cell=0:1,1:4,2:4 reference_filesystem_red_peak=4 reference_filesystem_red_peak_at=1 reference_filesystem_red_positive_cells=3 reference_filesystem_red_last=0 model_crash_judge_red_by_cell=- model_crash_judge_red_peak=0 model_crash_judge_red_peak_at=- model_crash_judge_red_positive_cells=0 model_crash_judge_red_last=0
E7RESULT name=q5_trajectory_cell sampling_point=main writes_landed_in_the_last_segment=13 lost_states=23297344 reference_filesystem_red=23297344 model_crash_judge_red=0
E7RESULT name=q5_trajectory_cell sampling_point=main writes_landed_in_the_last_segment=29 lost_states=0 reference_filesystem_red=0 model_crash_judge_red=0
E7RESULT name=q5_trajectory_cell sampling_point=reverse writes_landed_in_the_last_segment=5 lost_states=0 reference_filesystem_red=0 model_crash_judge_red=0
```

- The (ii) state of Q4 is concentrated in the segments of two calls—number retrieval and warm-up (segments 0–2)—and the call for writable mounting of stream two (segment 14 on line 242, segments 15–21 on line 269); the end-of-batch values per stream per batch are all zero across three runs, and the second segment's legality-checking state is zero in every cell. The peak value of 8 for stream two, batch one on line 269 occurs in segment 16 (also 8 in segment 20), corresponding to a positive segment count of 10.
- All (ii) states of stream three fall within calls outside inode creation (states at batch 1 run 2 and batch 2 run 1 on line 242, state at batch 1 run 6 on line 269); every one of the 235 inode creations shows a zero state: this batch does not show (ii) varying with leaf container splitting into more or fewer; the cell read sorted by inode creation is 0, not "present both before and after splitting."
- The number of red states in RefFS for Q5 equals the number of L states in every cell that has an L state (27 cells at main sampling points, 3 cells at reverse sampling points); the model-collapse judge is zero in every cell; the end-of-batch value for the final cell (where all segments fall) is zero because that cell has no L state (`lost_states=0`), not because RefFS judged green in that cell.
- `q5_verdict` changes its outcome from `not_enough` to `not_enough_f8` when there are no red states among the L states and no states where both are green (per Revision 5, Section 5.1, Item 7: record as insufficient judgment according to F8); for this run, both sampling points have red states and cannot reach that branch, so `q5_main=jia q5_reverse=jia` remain unchanged.

#### Valid Conditions and Q5 (Revision 6 Run)

Q1–Q4 do not trigger the specification executor (specification executor) for the eleventh section original text plus S15 items ①, ②, ③, ⑤, and ⑥ (Revision 6, Section 6.6); Q5 additionally requires two main sampling points where S1 passes completely, `s13=false`, the positive control "flag red (flag red/red verdict)" is exactly the panic single column count, two byte-by-byte identical results (Z7), reverse sampling point S1 passes completely, S15 item ④ does not trigger, plus an additional self-verification alarm (already passed, "conditions for validity and Q5 (Revision 6 run)" final clause at end of section). Copy the entire line (filename prefix omitted `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-`):

```
frozen-tree-r6-r3.out:61: E7RESULT name=verdict s15_1_extra_files_against_the_first_segment_tree=0 s15_1_extra_files_against_the_q5_tree=0 s15_2_holds=true s15_3_unassigned_hunks=0 s15_3_hunks=24 s15_3_table_rows=24 a12_failed_checks=0 a12_checks=12 triggered=none
stream-constants-r6-r3.out:82: E7RESULT name=verdict s1_failed_checks=0
s2-r6-r3.out:20: E7RESULT name=verdict s2_differing_files=0 s2_compared_files=18
s15-5-step-1-r6-r3.out:26: E7RESULT name=verdict s15_five_problems=0 triggered=false
s15-5-step-7-r6-r3.out:28: E7RESULT name=verdict s15_five_problems=0 triggered=false
s15-6-step-7-r6-r3.out:3: E7RESULT name=verdict s15_six_problems=0 triggered=false
s15-4-step-6-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
s15-4-step-10-run-1-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
s15-4-step-10-run-2-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
baseline-run1-r6-r3.out:2035: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 o2_red_states=0 recovery_panic_states=0 probed_states=12667 counter_branch_states=0 counter_branch_states_judged_legal=0 undecidable_branch_states=1340 witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0 piece_panic_states=- unattributed_panic_states=0
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:19: E7RESULT name=q5_equivalence_pairing in_flight_writes_kept=6 states=4608 product_of_the_choices=4608 class_states=4608 classes=144 mismatching_states=0 crash_judge_not_applicable_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:23: E7RESULT name=q5_piece_panic_control states=4608 pool_checker=1152 ignored_recovery=1152 record_checker=576 other_pieces=0 consulted_recovery=0 consulted_recovery_of_the_plain_pairing=0 o1_columns_differing_from_the_plain_pairing=0 mismatching_states=0 s13=false holds=true
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:25: E7RESULT name=q5_shape sampling_point=main_before_the_first_barrier_of_call_3 removed_barriers=[50,51,55,56] root_writes_made_plain=[52] truncation=81 operations_kept=77 confirmed_root=1,3 confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:54: E7RESULT name=q5_equivalence sampling_point=main_before_the_first_barrier_of_call_3 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:200: E7RESULT name=q5 sampling_point=main_before_the_first_barrier_of_call_3 states=1207959552 last_segment_writes=29 last_segment_tearable_writes=2 product_of_the_choices=1207959552 a10_states_equal_the_product=true lost_the_confirmed_version=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 not_lost_states=1056964608 not_lost_o1_red=0 states_without_a_root=0 derived_by=equivalence_classes judged_states=144
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:228: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=144 s13=false piece_panic_states=- unattributed_panic_states=0
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:19: E7RESULT name=q5_shape sampling_point=reverse_at_the_return_point_of_call_2 removed_barriers=[50,51,55,56] root_writes_made_plain=[52] truncation=57 operations_kept=53 confirmed_root=1,3 confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:47: E7RESULT name=q5 sampling_point=reverse_at_the_return_point_of_call_2 states=72 last_segment_writes=5 last_segment_tearable_writes=2 product_of_the_choices=72 a10_states_equal_the_product=true lost_the_confirmed_version=9 reference_filesystem_red=9 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 not_lost_states=63 not_lost_o1_red=0 states_without_a_root=0
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:51: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=72 s13=false piece_panic_states=- unattributed_panic_states=0
```

- Z7: The E7RESULT lines at the primary sampling point are identical byte-for-byte across two runs (`diff <(grep '^E7RESULT' …-run-1-r6-r3.out) <(grep '^E7RESULT' …-run-2-r6-r3.out) | wc -l` outputs 0, even without removing the duration field).
- For steps 11 and 12, both synthesized products (`…-s1-rows-probe-driver-resume2-r6-r3.out`, `…-s1-rows-driver-resume2-r6-r3.out`) show `differences=0` before and after the `freeze_check`, with `snapshot_digest=7726cdd6…`, 16 lines of `status=ok`, `device_exit=0`, and `o3_status=ran`; there is no entry for 253 (step 13 did not execute).
- S17 (cross-host consistency check): The original product `…-s1-rows-cross-host-check-resume2-r6-r3.out` was flagged as `E7RESULT name=verdict cross_host_identical=false row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=o3`, discarded, and not entered into the verdict: the primary agent determined (referencing the endnote on "Handling of S17 shutdown during cross-host consistency check with `-resume2`", items 2–4) that the discrepancy in that cell was limited to the frozen executor's process ID, number of processes, and multi-process row ordering; this is a defect of the checker, not an inconsistency between two hosts. The valid result comes from the newly generated `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07.out` (`E7RESULT name=verdict cross_host_identical=true row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=none`). The file `…-2026-10-07-r2.out` is a re-generated version of the same entry from within the repository, supplemented with the trailing line `E7RESULT name=done emitted=8`, and its first 7 lines are byte-for-byte identical to `…-2026-10-07.out`. The file `…-2026-10-07-r3.out`, re-generated after changing the entry to read the orchestrator stored in the same directory, is a re-run registration version differing from `-r2` only in line 2's `entry_sha256=`.
- The self-proof for the alert has been verified; Q5 verdict is accepted: the primary agent ran early with `SINGLEFS_HEAVY_TESTS=user-request` (user request on 2026-10-07 to "perform what can be done as early as possible"), unit `e171-q5-alert-selfproof`, log at `~/.local/state/singlefs/detached-runs/e171-q5-alert-selfproof/log`; the endnote on "Primary agent verdict: first segment (Revision 6 tree) Q5 alert self-proof" records the original readings verbatim: repository device sha256 `ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df`, device inline test case output `test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.75s` (including U1–U9, U13–U16, and the original two tests), and all 49 lines of `crates/mutations.tsv` for "E171R1 device" (including V17–V31, V33–V36) show: Count: caught 49, no red flags 0, invalid 0, memory limit hit 0, timeout 0, dropped by global cap 0, skipped due to scheduling 0, scope failure skip 0 (total 49). These two items are re-run in the checker file upon submission; if the repository device is no longer `ab35105f…` at that time, use that instance and record accordingly.
Source identifiers: `E7RESULT name=verdict cross_host_identical=false row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=o3`.

#### Comparison, Re-run, and Check-in Execution Methods (Revision 6 Run)

- The comparator `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py` was modified in eight places relative to the version for the first new tree, while the criteria remained unchanged (①–⑥ are recognized by the main agent; registration details: "Main agent recognition: Segment One (Revision 6 Tree) Comparator returned" Item 1; ⑦ and ⑧ correspond to Item 3 of that same entry requiring supplementation): ① Reading `oracle-verdicts.tsv` was changed to 25 columns (`run_directory.py`, Revision 5, Section 5.1, Item 5 added `piece_panics` at the end; the previous reading method for 24 columns fails on this export run); ② The current calculation for a11 was changed to "the call where the last write landed" (equivalent to taking the maximum per individual write, verified by unit test regression testing), reducing Stream Three state 11 from 695 states over ten minutes of inference down to 13 seconds for the entire comparator; ③ Arguments `--dumps-root`, `--product-prefix`, and `--product-suffix` are mandatory; if input is missing in this run, it exits with code 2, or if products are not written (Revision 8, Section 8.5, Item 1); ④ Added a row for "Stream Three Coverage (O3 (Independent Specification Executor))" alongside the verdict row's `f7=` (Section Six, f7); ⑤ Added columns a1 and reachability for the empty mutation row (Section Seven, second half after a1); ⑥ The `q5_judged` row now includes two additional columns: state counts for `judge_panic` (Revision 5, Section 5.1, Item 7, description only); ⑦ Supplemented Section Six Q4-(ii) with a per-state list and Section Eight Q4, Q5 trajectories (only new rows added; "Section on Per-State List for Q4-(ii) and Trajectories for Q4, Q5 (Revision 6 Run)"); ⑧ If `q5_verdict` has no red states among L states and no state where both are green, it reports `not_enough_f8` (previously reported `not_enough`; RefFS full waiver order remains unchanged, proceeding to f8 first).
- Unit tests: 41 lines (`grep -c '    def test_' test_segment_one.py test_device_timing.py` yields 33 and 8 respectively); running `python3 -B -m unittest test_segment_one test_device_timing` consecutively for 5 rounds results in `Ran 41 tests … OK` (deterministic pure functions; 5 runs confirm no hidden state). Research-side mutation table `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv` contains 37 entries (originally 17, added R14–R19 to monitor ①–⑥, added R20–R33 to monitor ⑦ and ⑧); the entire table of 37 entries caught all mutations, with 0 invalid, 0 survived, and 0 timed out; cleanup restored, baseline remains fully green: `E7RESULT name=verdict mutations=37 caught=37 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true` (`…-s1-research-mutations-2026-10-07-r3.out`). Two earlier runs on the same day, `…-s1-research-mutations-2026-10-07.out` (22 entries, 22 / 0 / 0) and `…-2026-10-07-r2.out` (23 entries, 23 / 0 / 0), were conducted before adding R19 and R20–R33 and are archived; `…-s1-research-mutations.out` corresponds to the run for the first new tree. Existing R4 (Q5 full waiver without F8) differs from the original formula only in the input case where "RefFS fully waives while the model crash judge has red" after ⑧, and unit tests pinned this cell (F8 placed before alpha, maintaining original order).
- Multi-machine execution method check-in (copied from draft, original draft remains unchanged): `rows_orchestrator.py` (`7bfbdb84…`), `peer_side.py` (`27cea9e7…`), and `check_product.py` (`3f237083…`) are byte-for-byte identical to the files in the draft; `cross_host_o3_check.py` (`afc9a9b0…`, while the draft version is `04376c00…`) has been modified to read the orchestrator checked into the same directory (execution continues as usual, verifying against `7bfbdb84…`); `run_segment_one_on_the_revision_6_tree_multi_host.sh` (`567a8682…`, while the draft version is `ec89ec78…`) has its orchestrator and `check_product.py` modified to point to this directory in the repository (the orchestrator retrieves `peer_side.py` and `` from its own directory); frozen tree, root export, logs, and resume checklist remain unchanged along with the non-checked-in `s2_compare.py`, `s15_compare.py`, `frozen_tree_lines_revision_6.py`, and `resume_check.py`, which stay in the draft `e171-r1-segment1-rev6/`. Self-verification on the repository version: last line of orchestrator shows "Self-verification 34 items, red 0"; last line of cross-machine comparison entry shows "Self-verification 38 items, red 0". Neither `peer_side.py` nor the execution script includes `--selftest`; the execution script passes `bash -n` and `detached-run.sh long-run-check` (long-running declaration requires all three keys). Revision 6 actually ran using those files from the draft (registrations recordestablished sha256 based on the draft).
- Re-execution: Add three lines to `scripts/experiment-apparatus/replay.sh`, both only re-exporting and reproducing artifacts retained locally in this run, without invoking devices or cargo:

| Driver Function | Output | What to Run |
|---|---|---|
| `driver_e171_r1_s1_revision_6_comparator` | `…-s1-comparator-resume2-r6-r3-2026-10-07.out` | Comparator, export root `scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/`; approximately 11 seconds |
| `driver_e171_r1_s1_revision_6_cross_host_check` | `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out` | Cross-machine comparison entry point `compare`, append trailing rows |
| `driver_e171_r1_s1_research_mutations` | `…-s1-research-mutations-2026-10-07-r3.out` | Research 37 mutations on one side, approximately 40 seconds |

Three driver functions were run in the draft directory according to the specifications of `replay.sh`, producing outputs that are byte-for-byte identical to the stored artifacts, with matching counts for `name=done emitted=` and E7RESULT lines (811, 8, 78). Cross-machine comparison shows that `-r3` and `-r2` differ only in the `entry_sha256=` field of line 2 (the entry's own SHA-256 hash: `04376c00…` → `afc9a9b0…`), while all judgment lines are byte-for-byte identical; the two original files referenced, `…-comparator-resume2-r6-r3.out` and `…-2026-10-07-r2.out`, are retained. The root export does not enter the warehouse; it flags red  if the first two lines are missing. Steps 0–13 themselves (devices on the frozen tree (frozen tree), line-by-line drivers, O3 (independent specification executor (specification executor)) executors, orchestration across two machines) accumulated dozens of hours of wall-clock time and required a second machine; this was not logged.

#### What It Cannot Answer (Revision 6 Run)

- The flag red  applies only to Revision 6 tree (the first segment of the new tree replacing the device, the implementation after Batch 16 merge and before Root Loop 10b, without Batch 31 core assertions); for Revision 8, that run was replaced by the final mainline, and a separate comparison is produced (registered in Section 8.5 of Revision 8), which is not inferred here.
- Q1's alpha only states "seven lines on this batch: three flows, at least one red per line on Batch 1," without mentioning that common blind spots were filled: bad methods and historical data not registered in the table may still be collectively passed by all three flows. Line 23 was caught only on Flow 3 by O2 (independent parser + checker), and Line 42 was caught only once during an `unmount` call by O1 (memory model differential testing with HashMap ideal file system); under the scope of Flows 1 and 2, all three lines for Line 23 were still fully passed.
- Q2's first segment contains descriptions for only seven lines; the sets and judgments originate from the second segment's entire table (Flows 1 and 2, 281 lines plus fillers; the set is precisely these seven lines, Judgment beta). Lines 49, 110, 162, 174, and 252 return different values or memory objects but exhibit identical observations: the difference lies in fields not handed over to the model by the glue code, which O1 (memory model differential testing with HashMap ideal file system) -differential testing cannot see by design.
- Q3's first segment judges only whether Line 125 is included; the denominator D = 39 comes from the second segment's entire table. c counts only the flow that panicked (12); including Flow 1, which did not panic, yields 13, but the judgment remains unchanged.
- Q4's (ii) states are entirely on Lines 242 and 269; the second section fully judges them as invalid; the baseline (the run of the implementation on Revision 6 tree) has (ii) state 0. (i) cannot judge that branch if it does not enter candidates, with thousands of states per run. All three runs are states truncated by replaying crash points, without injecting read faults.
- Q4's per-state list and trajectories exist only for the two bad implementations (Lines 242 and 269) regarding (ii) states; Flow 3's (ii) states are entirely outside inode construction: even when arranged by inode construction with 235 slots, all are 0, so it cannot be stated whether (ii) relates to leaf container splitting; it only states that within this batch, no (ii) state was retrieved in the inode construction calls before and after splitting.
- Q5's primary sampling point L = 150 994 944 is derived by equivalence analogy (per-state judgment covered 144 states); equivalence holds only on 4 608 states at k = 6, which were verified per-state; reverse sampling points cover 72 states judged per-state. The alarm self-verification has passed; both points are judged alpha; the alarm self-verification only proves that alarms on Q5's judgment path will turn red, without expanding the scope of equivalence relations verified per-state.
- O3 (independent specification executor) column is v2-487; on Flow 3, strictly reading 6 459 states all yield "present / absent" (F7 not triggered). Among the seven lines, only Lines 242, 269, and 270 show flag red  for O3 (independent specification executor); the remaining four lines never showed flag red in any state for O3 (independent specification executor).
- Lines 252 and 271 run the text of that line within the frozen tree (frozen tree) (`line_matches_the_registration=false`), differing from the text written during registration 5.9; they retrieve by name, with exactly one anchor point, and no fillers were added.

#### Artifacts (First Re-run of the First Segment)

The artifacts from revision 6 (`ls research/results | grep '^e171-r1-' | grep -- '-r6-r3'`, 81 files, including comparator outputs with resumed state lists and trajectory comparisons `…-comparator-resume2-r6-r3-2026-10-07.out` and cross-host comparison results regenerated after changing the entry path `…-cross-host-check-resume2-r6-r3-2026-10-07-r3.out`) along with five artifacts from the research-side mutation table (`grep 'research-mutations'`). The `-timing.out` file contains the E7TIMING line; `…-rows-probe-driver-r6-r3.out` and `…-rows-probe-driver-resume1-r6-r3.out` are half-artifacts from the first two runs that stopped before step 11 (each containing one row, not entering judgment); the artifact for `…-rows-cross-host-check-resume2-r6-r3.out` that evaluated to `false` is discarded (as per condition S17 in section "Valid conditions and Q5 (revision 6 run)"), and does not enter judgment:
Source identifiers: `false`.

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-run1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-streams-1-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o3-p1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-row-0-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-row-0-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-10-run-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-10-run-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-6-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-5-step-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-5-step-7-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-6-step-7-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-research-mutations.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations.out
```

Revise the first segment of products on several trees prior to revision 6 (85 items, `ls research/results | grep '^e171-r1-' | grep -v -- '-r6-r3' | grep -v 'research-mutations'`): those starting with `2026-10-04` correspond to the tree for revision 1; those dated `2026-10-05` without a suffix represent the first segment of the new tree (`4d071c12…`, step 7a1 does not hold on stream three, section 6.1 of revision 6); `revision-5-q5-` denotes the Q5 tree; `-r6` and `-r6-r2` are records from the first two runs of the revision 6 tree that halted at step 0. All are retained as references and excluded from any flag red  assessment (final table in section 6.1 of revision 6, and items 3 and 5 under "Main Agent Determination: Deviations Between Devices and Notations in the Warehouse Prior to the Start of Revision 6"):

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-comparator-streams-1-2-before-the-rows.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-o3-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-memory-object.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-comparison.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-comparison-unmount.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-crash-judge-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o3-p1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-panic.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-reference-filesystem-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-return-value.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-witness.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-q5-main-before-the-first-barrier-of-call-3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-q5-reverse-at-the-return-point-of-call-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2-device-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2-e170-export-dump.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-stream-constants.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o3-p1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-1-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-reverse.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-reverse-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-on-the-draft-copy-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-timing.out
```

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:865-966`（整段抄，未转述）**

````markdown
### First Re-run of the Second Segment · Revision 6 Run (Full Table Executed: Q2 beta, Q3 Denominator D = 39)

Register the following sections of `research/prompts/e171-r1-prereg.md`: Section 5.5, second paragraph; Section 6, rows for Q2-Set and Q3-Denominator; Reading Method 4 and Reading Method 5 from Section 1; Revision 5, end of Section 5.1 "Reading Methods of Existing Products" (last item: only read O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Fuzzing and Invocation; two crash sub-sequences do not enter these two rows); Revision 6, Section 6.6 "Second Paragraph (Entire Table) as per original text, run on the Revision 6 tree"; final section "Main Agent Determination: Second Paragraph Entire Table Multi-Machine Run Method Returned". The tree is identical to the first paragraph (snapshot summary `7726cdd6…`); Line 281, Stream One, Stream Two, Batch 1 and Batch 2; do not run the specification executor (specification executor), without probes; time limit per row is 1800 seconds (T = 75.274 seconds, 3T less than 1800); two machines completed the runs, zero rows timed out, zero wrapped outcomes; cross-machine consistency check at Line 201 determined identical results for flag red .

Q2's "different from baseline" comparison is against the run of Step 1 in Revision 6, which uses a new device baseline (`--streams 1,2`, without probes, exported to `scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/s2-device-baseline-streams-1-2/`): the table at the end of Section 6.1 in Revision 6 states that Q2's set is compared against this run from Step 1 (the old device version cannot provide a root ring table, resulting in every row showing "memory object different"), and the T mentioned in the second paragraph of Section 2.5 in Revision 2 also originates from it. Reading 4 ("remove objects with different summaries even when running the baseline twice"): Step 1 ran only once; the comparator separately took the same tree and the same device's Runs 7 and 9, generating two three-stream baselines (`baseline-run1/`, `baseline-run2/`) that compared only Stream 1 and Stream 2. Eight files (four write streams, `states.tsv`, `oracle-verdicts.tsv`, `call-outcomes.tsv`, `call-return-values.tsv`) were identical byte-for-byte across the three copies, with zero unstable objects.

| Line | Verdict (Revision 6 Tree) | Basis (Artifact) | To Read Alongside Beyond Literal Text |
|---|---|---|---|
| q2 | beta: Set of 7 lines, O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Fuzzed Red 2 Lines (Lines 155, 161), Not Red 5 Lines (Lines 49, 110, 162, 174, 252) | `…-2026-10-07-s2-comparator-r6.out`'s `name=q2` (`set_rows=7 o1_red_rows=2 … verdict=yi`) | The set is exactly C672 (a class where return values or memory states are incorrect while write streams remain unchanged); the seven named lines, and all other 227 rows in the table that were not reached, have return values and memory objects identical to baseline. Among 53 different calls, glue handed over to the model's observations differed only once on Lines 155, 161 raising F (`ceiling`); O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Fuzzing `disagrees` on these two; remaining 51 observations are identical: differences lie in fields invisible to the model (`writes`, `output`, `current`, `publishes`, `allocator`, `system_configuration_writes_before_the_first_publish`) and memory objects (`mount_output`, `current_version`, `allocator`); O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Fuzzing `agrees`. The seven lines' sequential calls match verbatim with the first segment calculated on that run of three streams with probes |
| Q3 Denominator | D = 39 (First Rerun Read Method 5, Batch 1 Determined Reached) | `name=q3_denominator` Four Lines | Original Registration Read Method 5 (calls to panic and state-based recovery panics all self-collapse); D = 38, differing on Line 125; Batch 2 Determined Reached D = 35 (Original Registration Read Method 5 was 34), differing on Lines 27, 112, 233, 277 (differ only in recovery reports viewed once during a specific state in Batch 1 regarding the journal). Not included in denominator: Unreached 234 lines, Self-collapse 6 lines (Lines 6, 7, 85, 95, 235, 272), Not Found 2 lines (Lines 107, 168); All 5 lines that failed to compile were added; Timeout 0 lines. Line 125 is included in both batches of the registration algorithm but excluded from both batches of the original registration algorithm |

Judgment lines and Q2, Q3 rows (entire lines copied from `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out`, lines 6, 7, 349, 410, 411, 414, 415, 416, 421, 426, 431–441, 447–450):

```
E7RESULT name=unstable_memory_objects stability_baselines=2 names=-
E7RESULT name=driver_product row_time_limit_seconds=1800 snapshot_digest=7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4 rows=281 table_rows=281
E7RESULT name=q2_observation set_rows=7 calls_with_a_difference=53 calls_whose_observation_differs=2 calls_whose_observation_is_the_same=51
E7RESULT name=q2 set_rows=7 o1_red_rows=2 rows_o1_red=155,161 rows_o1_not_red=49,110,162,174,252 verdict=yi
E7RESULT name=q3_denominator reading=registration batch=1 rows=281 counted_D=39 counted=39 untouched=234 self_crash=6 not_found=2
E7RESULT name=q3_denominator_class reading=registration batch=1 class=self_crash rows=6 row_numbers=6,7,85,95,235,272
E7RESULT name=q3_denominator_class reading=registration batch=1 class=not_found rows=2 row_numbers=107,168
E7RESULT name=q3_denominator reading=registration batch=2 rows=281 counted_D=35 counted=35 untouched=238 self_crash=6 not_found=2
E7RESULT name=q3_denominator reading=original_reading_5 batch=1 rows=281 counted_D=38 counted=38 untouched=234 self_crash=7 not_found=2
E7RESULT name=q3_denominator reading=original_reading_5 batch=2 rows=281 counted_D=34 counted=34 untouched=238 self_crash=7 not_found=2
E7RESULT name=q3_self_crash_row row=6 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=7 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=85 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=95 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=125 where=fixed_script_through_e_then_unmount:call_14:judged_12 registration=counted original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=235 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=272 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_row_125 row=125 registration_batch_1=counted counted_registration_batch_1=true registration_batch_2=counted counted_registration_batch_2=true original_reading_5_batch_1=self_crash counted_original_reading_5_batch_1=false original_reading_5_batch_2=self_crash counted_original_reading_5_batch_2=false
E7RESULT name=recovery_panic_sum rows=0 states=0 by_row=-
E7RESULT name=not_located_row row=107 status=not_found failed_attempts=1 substituted_from=- name_hits_in_the_frozen_table=0
E7RESULT name=not_located_row row=168 status=not_found failed_attempts=1 substituted_from=- name_hits_in_the_frozen_table=0
E7RESULT name=untouched_row_red row=155 O1-对拍=1 O1-模型崩溃判官=0 O1-RefFS=0 O2=0
E7RESULT name=untouched_row_red row=161 O1-对拍=1 O1-模型崩溃判官=0 O1-RefFS=0 O2=0
E7RESULT name=driver_product_rows rows_in_the_product=281 rows_in_the_root=281 rows_whose_status_or_exit_differs=-
E7RESULT name=verdict q2=yi q2_set_rows=7 q2_o1_red_rows=2 d_registration_batch_1=39 d_registration_batch_2=35 d_original_reading_5_batch_1=38 d_original_reading_5_batch_2=34 row_125_counted_registration_batch_1=true rows_read=281 table_rows=281 rows_not_located=107,168 unstable_memory_objects=0 stability_baseline_files_differing=0 rows_compiled_without_the_rebuild_check=0 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 untouched_rows_with_a_red_column=2 rows_whose_directory_differs_from_the_driver_product=0
```

Where do the seven rows of the set differ on each invocation (run `grep 'name=q2_set_call' research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out | sed -E 's/.* row=([0-9]+) .* fields=([^ ]*) objects=([^ ]*) observation_differs=([a-z]*) .*/\1 \2 \3 \4/' | sort -k1,1n -k2 | uniq -c` at the repository root; columns are count, row, top-level field with differing return values, differing memory objects, and whether observations differ) output as-is:

```
     10 49 - mount_output false
      1 49 output mount_output false
      1 110 #0 current_version,mount_output false
      1 110 current,output current_version,mount_output false
      1 110 - current_version,mount_output false
      1 110 publishes current_version,mount_output false
      3 110 writes current_version false
      7 110 writes current_version,mount_output false
      1 155 ceiling - true
      1 161 ceiling - true
     10 162 - mount_output false
      1 162 output mount_output false
      1 174 system_configuration_writes_before_the_first_publish - false
      1 252 allocator allocator false
     13 252 - allocator false
```

- Under both reading methods, the following lines self-collapse: Lines 6, 7, and 95 panic during the first file release of the two streams (on the second invocation); prior to this, only two `not_modelled` events occurred for numbering and warm-up, which can be counted as 0. Lines 85, 235, and 272 panic before the first invocation (the assertion indicates a mismatch between the byte count written by system runtime configuration and immutable system configuration versus the offset of code 2's header and field table; it is inferred to occur during the `mkfs` step, though the call stack was not verified). Neither stream exported these lines, and no invocations were issued (device `name=stream … exported=false not_exported_why=pipeline_panicked:… calls=0`). The original text narrowing reading method 5 states "the countable invocations before the first panic are 0"; similarly, for these three lines that panicked before the first invocation, the countable invocations are also 0. The comparator records self-collapse (the original comparator also recorded `pipeline_panicked` as self-collapse); this specific reading method awaits confirmation from the main agent; switching to an alternative reading method means these three lines do not enter the denominator either (no export for comparison, reaching them yields no judgment).
- Lines 107 and 168 are missing: The variant names for both lines hit 0 occurrences in `crates/mutations.tsv` within the frozen tree (the text was renamed after registration); the driver must fill in from "the position of this name within this file group today"; if the name is not in the group, the starting point cannot be found, and it is recorded as unfillable (`failed_attempts=1`, `substituted_from=-`). Following section 5.2 of the register stating "report accurately when unfillable," these do not enter the denominator. Another reading method involves searching downward from the intra-group index recorded in the registration table; this awaits confirmation from the main agent; at most, two lines would be affected, D is 39–41, and Q2's beta cannot flip (the set already contains both red rows and non-red rows).
- The five lines that were not compiled but filled in (Lines 33, 59, 127, 163, 206; Line 127 was swapped twice) are all replaced according to the fill-in rule with the next row in the same group. All replacements were not reached (`name=substitution` for five rows and judgment row `…-s2-rows-substitution-coordination-r6.out` showing `substitute_collisions=0`).
- Verification: The seven lines differ sequentially across invocations (row, stream, invocation, field, object, observation same or different) and match the first segment of the comparator's calculation on the three-stream run with probes for `name=q2_named_row_call`, specifically the Stream 1 and Stream 2 parts are identical character-for-character. Each set takes 53 rows; removing extra labels from the second segment and O1 (memory model alignment (HashMap ideal filesystem)) -alignment two columns, then sorting and running `cmp` produces no output. Both sets use the same comparison function; only the two device runs and the two baselines are independent.
- Discard and Stop: Z6 has `rows_compiled_without_the_rebuild_check=0`; both numbers on the comparator side for S5 are 0; both batches panic to a 0 state during the journal recovery pass (Revision 5, Section 5.1 counting method; the second segment does not read two crash sub-columns, so the count remains unchanged); Z3's wrapper outcome and timeout record 0 rows (synthesized product `check_product.py rows_without_executor` records `timed_out=none wrapper_outcomes=none`); Z5 synthesized product snapshot summary line 2 is `7726cdd6…`.
Source identifiers: `substitute_collisions=0`.

Comparator, Unit Test, Mutation, and Re-run:

- Comparator `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_two_comparator.py`: Write a separate comparator without modifying the first segment's comparator (the verdicts, unit tests, and mutation tables for the first segment must remain unchanged; the second segment uses different inputs: one baseline, one row output root, and one synthesized product). Reuse `collusion_matrix.py` for reachability, value differences, callable invocation counts, self-collapse, and the two S5 numbers. Additionally batch 2 determinate reachabilities (retain only that batch's state; modify record batch 1 again and pass it to the same reachability function), classify and summarize the two reading methods. Read only this run's exported roots (`revision-6-2026-10-05-r3/` and `revision-6-segment-2/`) and product prefixes `…-2026-10-07-s2-`, suffixes `-r6.out`; discard the rest. The entire table takes approximately 16 seconds.
- Unit tests `test_segment_two.py`: 17 tests (`grep -c '    def test_' test_segment_two.py`). Manually construct flows and temporary directories, ensuring at least one test case for each category of verdict: batch 1 and batch 2 determinate reachability, various outcomes for drivers and devices, orderings between self-collapse and reachability, the two reading methods, panic during the `mkfs` step, denominators and row-by-row categorization, Q2 set (rows not reached by reachability, rows not reached by only batch 1 reachability, unstable objects removed), Q2 categories A, B, C, and F4, exported roots. Add one additional test: an 8-row full table write to disk running `main`, with verdict lines pinned row-by-row. Running consecutively for 5 rounds yields `Ran 17 tests … OK` (deterministic pure functions; the 5 runs confirm no hidden state).
- Investigate one side of the mutation table `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_segment_two.tsv` with 28 entries (W1–W28), run via the same directory's `mutation_driver.py` with the optional parameter `--unit-test-module test_segment_two` (the driver added this optional parameter; without it, it runs the first segment's two modules as before: re-run the first segment's table and verify byte-for-byte identity with `…-s1-research-mutations-2026-10-07-r3.out`): `E7RESULT name=verdict mutations=28 caught=28 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true` (`…-2026-10-07-s2-research-mutations-r6.out`). Caught / Invalid / Not Red = 28 / 0 / 0. Running consecutively for 5 rounds produces byte-for-byte identical output. The anchor point of the first table with 37 entries still hits each once after adding new files.
- Replay: Add two lines to `scripts/experiment-apparatus/replay.sh`; both only re-export on existing exports and products, without starting devices or cargo:

| Driver Function | Output | Execution Details |
|---|---|---|
| `driver_e171_r1_s2_revision_6_comparator` | `…-2026-10-07-s2-comparator-r6.out` | Run the second comparator; read exports from steps 1, 7, and 9 of revision 6 against `revision-6-segment-2/segment2-rows/`; retain locally without archiving; if missing, flag red  |
| `driver_e171_r1_s2_research_mutations` | `…-2026-10-07-s2-research-mutations-r6.out` | Research 28 mutations on one side; approximately 40 seconds |

Two driver functions were executed in the draft directory according to the specifications of `replay.sh`, producing outputs that are identical byte-for-byte with the stored artifacts. The entire table itself (281 rows, each modified line-by-line, compiled, and run on devices across two machines) is not registered.

The outputs of the second segment (`ls research/results | grep '2026-10-07-s2-'`, 8 files): synthesized output `…-s2-rows-driver-r6.out` (565 lines: 281 lines `row`, 281 lines `merged_shard`, plus a few lines at the beginning and end) and its `-timing.out`; fill-in coordination `…-s2-rows-substitution-coordination-r6.out`; cross-host consistency comparison `…-s2-rows-cross-host-row-201-driver-r6.out`, its `-timing.out`, and verdict `…-s2-rows-cross-host-check-r6.out` (`E7RESULT name=verdict cross_host_identical=true row=201 host_of_rows_step=2 host_of_cross_host_run=1 parts=4 compared_parts=3 differing_parts=none`, o3 one cell `not_run`); comparator `…-s2-comparator-r6.out`; research-side mutations `…-s2-research-mutations-r6.out`:

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-check-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-row-201-driver-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-row-201-driver-r6-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-substitution-coordination-r6.out
```

#### What It Cannot Answer (Second Paragraph)

- In Q2, beta states only that "in the histories of Stream One and Stream Two, among the seven lines where values were not reached but returned or memory objects differed, O1 (Memory Model Fuzzing (HashMap Ideal File System)) - Fuzz Red 2 lines": The five non-red lines differ in glue fields and objects not handed to the model; O1 (Memory Model Fuzzing (HashMap Ideal File System)) - Fuzz cannot see them by design; it does not state whether the states of these five lines are legal or illegal, nor does it specify how large the sets are on other histories.
- D = 39 is a descriptive quantity; Q3's alpha and beta are determined in line 125 of the first paragraph (c = 12); the second paragraph does not alter any determinations from the first paragraph. The several definitions of D differ only in reading type 5's self-collapse category and the batch of fixed-reach items; all four numbers must be written.
- Determinations belong exclusively to Revision 6 tree; for Revision 8 run (the final main line), it is separately determined where the second paragraph runs on which tree ("Main Agent Determination: Revision 8 Returned" item 8).
- The classifications of lines 85, 235, and 272 versus lines 107 and 168 are two readings (found in the section "First Re-run of Second Pass · Revision 8 Run" under "Those Lines That Self-Collapse Under Two Readings" and "The Missing Lines 107 and 168"), pending Main Agent determination; neither reading allows Q2 to flip sides.

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:967-1039`（整段抄，未转述）**

````markdown
### First Re-run Revision 12: Three-Stage Long Re-nailing (2026-10-13)

Register revisions 12 (written by the designer) and 13 (written by the executor, read-only copy) of `research/prompts/e171-r1-prereg.md`; rows 1 and 3 of issue ticket `e171-r1-s3-segment-lengths-questions.md` (the status in row 2 was changed by the main agent). This section pertains only to two numbers belonging to a single constant within the device, `LOCAL_THIRD_STREAM_SEGMENT_LENGTHS`; none of the judgments, counts for Q1–Q5, or the product from revision 6's frozen tree have been altered.

1. **Why Repin**: After the retirement tree verdict was implemented on 2026-10-09, today's crates write two extra retirement units during the first file deletion in stream three (at `retirement_records_of_this_publish` in `crates/singlefs-core/src/transaction.rs`). The segment length table pinned by the device on 2026-10-05 does not match today's crates' current calculation at two points. Before repinning, the product from that run (Product A, `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12-before-the-repin.out`) has `s1_failed_checks` equal to 1 in the verdict line at row 82 (obtained via `grep -n 'name=verdict'`).

The line for the segment length (line 72, `holds=false`) contains more than 4 KB and does not copy in full lines; register the comparison command from Section 12.14 to output Product A exactly as-is:

   ```
   lengths 959 959 sums 5980 5980 closed 362808742 362808742
   differing [(15, 20, 24), (43, 24, 20)]
   permutation True
   index_11_15 local 26 20 crates 26 24
   local_sha256 43d5bd3959c56382bc4bf220e35f964fda89b9d49af702d869bc31048aa6c5ce
   crates_sha256 fcec3e5e21df15d48891fc12d6dd366c282fff7306e58bc2144aac6bee2ac66c
   holds false
   ```

2. **Old vs New Comparison**: Subscript 15: 20 → 24, Subscript 43: 24 → 20; Write count 5980, Total state count 362,809,947, Quick tier 3127, Closed form 362,808,742 remain unchanged (the four rows `…_writes`, `…_full_states`, `…_quick_tier_states`, `…_two_state_closed_form` corresponding to "beta Li" have identical values in the `local` and `crates` columns with `holds=true`).

3. **Sequential Judgment** (beta = first pass after heavy nailing, gamma = second pass):

| Metric | Reading | Verdict |
|---|---|---|
| RS1 | `grep -c 'holds=false'` yields 0 (`grep -c 'holds=true'` yields 77, S1 total 77 lines); `grep -n 'name=verdict'` returns `82:E7RESULT name=verdict s1_failed_checks=0`; `cargo run` exits with code 0 (derived from the background agent's output chain; the executor did not see that output file, relayed via dispatch prompt) | All passed: all three reached thresholds |
| RS2 | After removing the segment length line and the `name=verdict` line, `diff` between alpha and beta shows 0 differing lines; entire `diff Jia Yi` differs only at lines 72 and 82 | No other item changed (the three-stream allocator's mkfs root ring table rows are identical in both copies) |
| RS3 | In beta's segment length line, columns `local` and `crates` show `differing []`, summaries both `fcec3e5e…`; draft table matches alpha item-by-item for `crates=` (verified by executor on draft previously) | Not a criterion; only copying checked: copied correctly |
| RS4 | In segment length line, column `crates=` summary is identical (`fcec3e5e21df15d48891fc12d6dd366c282fff7306e58bc2144aac6bee2ac66c`) in both alpha and beta, matching the newly written nail value summary | Write side matches new nail value item-by-item |
| RS5 | alpha: `holds=false` exactly line 72; differing indices exactly 15, 43, `(local, crates)` as `(20, 24)`, `(24, 20)`; `s1_failed_checks=1`; exits with code 3 (previous executor's reading, wrapped in memory and subject to thread limits) | Positive control flagged red at naming point |
| RS6 | beta vs gamma `sha256sum`: both lines are `aef90046e0b565776288e5051c7267f706db4ee718bfceeb66dfb0272dfdd2a1` | Both runs byte-for-byte identical (deterministic, indicates no hidden state) |
| RS7 | Side "do not write retirement records" | Cannot be done on ingestion device; no metric available |
| RS8 | `grep -c 'stream-constants-r6-r[23]' scripts/experiment-apparatus/replay.sh` before and after change: 0, 0 | Neither copy has replay rows; handle per Item 7 regarding "two old artifacts from 2026-10-05" (`82:E7RESULT name=verdict s1_failed_checks=0`) |

Commands and outputs of RS2 (`<A>`, `<B>` are the two files from clause 4):

   ```
   $ diff <(grep -v -e 'check=first_file_unlinked_then_inodes_across_one_leaf_segment_lengths' -e 'name=verdict' <甲>) <(grep -v -e 'check=first_file_unlinked_then_inodes_across_one_leaf_segment_lengths' -e 'name=verdict' <乙>) | wc -l
   0
   ```

RA1: The subscript 11 under `crates=` for alpha is 26, and the subscript 15 is 24 (old nail value 20 plus 4); the clause matches the implementation, and F10 was not triggered. RA2: 959 numbers, sum 5980, 1 + Σ(2^n − 1) = 362 808 742 (alpha `lengths 959 959 sums 5980 5980 closed 362808742 362808742`). RA3: After sorting the old and new tables, they are identical; the differing subscripts are exactly 15 and 43 (alpha `permutation True`, `differing [(15, 20, 24), (43, 24, 20)]`). No triggering observations for Z10 or S19 appeared at all (RS5 to threshold, RS2 is 0, RS4 alpha and beta are the same, RS6 is the same, beta `grep -c '^E7RESULT '` equals 83 which matches `name=done emitted=83`, memory packaging did not regress by 245 or 250–254).
Source identifiers: `name=done emitted=83`.

Geometric sensitivity points: stream three changes; streams one and two remain unchanged. A line-by-line comparison of beta with `…-s1-stream-constants-r6-r2.out` from 2026-10-05 shows that the raw output of `diff … | grep -E '^[0-9]'` is `33c33`, `49c49`, `65c65`, `72c72`: the differing lines are exactly line 33, 49, and 65 (three root ring table summaries, where both columns represent each tree's current calculation) and line 72 (stream three segment length); stream one and stream two segment lengths, write counts, root slot write counts, and all rows for full / fast / closed are identical byte-for-byte.
Source identifiers: `33c33`.

The device's `crates/mutations.tsv` lists its 47 rows, with anchor verification outputting `rows 47`, `count!=1 []`, and `in 200-270 []` (command registered in Section 12.14).

4. **Artifacts** (all located under `research/results/`):
   - Alpha: `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12-before-the-repin.out`, 83 lines, sha256 `dcf5b558bed1a379437b91efb716a67fc15e5dbe5ac01ac6d4b10c7277137c9c`, retreat 3. After replacing the device, re-running fails to produce output; therefore, it is not registered for re-run but only noted here.
   - Beta: `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12.out`, 83 lines, sha256 `aef90046e0b565776288e5051c7267f706db4ee718bfceeb66dfb0272dfdd2a1`, retreat 0.
   - Gamma: Not stored in the repository; sha256 is identical to Beta.
   - After replacing the device, the three sha256 values are: before modification `c556d42d…`; both the draft and after replacement are `6aa1bb82…` (`compile-then-swap.py` retreats 0; changes in the replaced version compared to the draft differ only by filename lines). This block currently exists as uncommitted content relative to the staging area within the main workspace; how should it be committed back to the main agent?

5. **Re-run**: One line, registered in `scripts/experiment-apparatus/replay.sh`, retaining only the `E7RESULT` row and exact ratios; the entire `replay.sh` is heavy-duty and has not been run, so this is the output of this pipeline on today's warehouse for beta.

| Driver Function | Artifact | Execution Target |
|---|---|---|
| `driver_e171_r1_revision_12_stream_constants` | `…-2026-10-13-s1-stream-constants-revision-12.out` | Device `stream-constants --streams 1,2,3`, approximately 2 minutes |

6. **Issue Ticket Line 2**: The "write" side is completed by the RS4 specification executor: the segment length table calculated from today's crates (which inherently include retirement records) matches the newly pinned values item by item. On the "non-write" side, it cannot be implemented on the ingestion apparatus; three approaches each yield a single sentence: it cannot be done with a switch built into the device itself because retirement records are unconditionally appended by core upon every release, and the device is driven only through public entry points, allowing action only via changing history (which belongs to another stream) or post-facto erasure (which cannot recover allocation footprints); taking the tree prior to f8a9ddaa that lacked a retirement tree cannot be done because the repository does not contain a submission of that device with that version, and that tree differs from today by 22 files, making it impossible to separate retirement records; relying on counterfactuals from replicas serves only as a clue because doing so would require modifying core's source code and thus falls outside the scope of the ingestion apparatus.

7. **Two Old Artifacts from 2026-10-05**: `…-2026-10-05-s1-stream-constants-r6-r2.out` and `-r6-r3.out` (identical byte-for-byte) are readings on the revision 6 tree; the `crates=` value is that tree's current calculated value, which differs from today's device pin value and today's crates' current calculated value; retained as a record for that run of revision 6, no longer representing current values; their rows were originally absent in the re-run table (both before and after RS8 modification remained 0). `…-s1-stream-constants-on-the-draft-copy-r6.out` is similarly retained. The few sentences earlier on this page that quote them within the first paragraph of the first re-run refer to events at that time for revision 6's run and remain unchanged.
Source identifiers: `-r6-r3.out`.

#### What It Cannot Answer (Revision 12)

- "The swap was caused by retirement records" appears only on the investigator's copy: replacing the retirement records in core with an empty list results in the segment length table matching the old pinned values item-by-item. Numbers on the copy do not enter kb; they serve only as clues (referenced in `records/2026-10-11-崩溃放量两段与脚本审计.md`, specifically the entry from 2026-10-13 pointing to the investigator report); this cannot be reproduced on the ingestion apparatus.
- Why index 43 changes accordingly (pushing slots after leaf creation in the allocation record tree, pushing several slots after slot drop) also comes solely from prints on the copy; it is a push operation. RA1 handles clause deduction only for indices 11 and 15, not for 43.
- Revision 6 states that `crates=` equals the old pinned value only to indicate compatibility between the two: that tree differs from today by more than just retirement records (the count recorded when preparing to return revision 8 was 140 files), and it does not attribute the swap.
- The apparatus's inline cases compare only the sum and closure of the segment length table, are insensitive to order, and cannot detect a swap between indices 15 and 43; what can be detected is that `stream-constants` actually ran stream three (hence no mutation rows were added to `crates/mutations.tsv`, and this variant in the swapped segment length table is borne by product A).
- The counterfactual ("The swap was caused by retirement records") is a clue, not a reading: it did not reproduce on the ingestion apparatus and does not enter any judgment.

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:1040-1056`（整段抄，未转述）**

```markdown
### Path and Conclusion Registration

| # | Path | How it is calculated | Source code location | Which shared items were read |
|---|---|---|---|---|
| 1 | Batch 1 state count (device) | The sampling formula for three batches acts on each ordinal interval given by the crash point replay enumeration table, plus tear scanning; enumeration points, not full volume | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1359` `sample_ordinals` | Segment intervals of `crash_state_enumeration_tables` and which writes take three states (from `crates/singlefs-checker-tier/src/crash.rs`) |
| 2 | Batch 1 state count (comparator current calculation) | From `ops.tsv`, split segments according to splitting rules; write ordinals from `torn.tsv` serve as three-state writes; number of states per segment = product of how many states each write takes minus 1, then recalculate ordinal set using the same sampling formula | `scripts/experiment-apparatus/e171_three_oracle_collusion/baseline_checks.py:175` `recompute_first_batch` | Sampling formula (text from Section 5.4 registered by E170 (first version of O3 specification executor)); `torn.tsv` and `ops.tsv` (device outputs these from enumeration tables and write tables in crates): both paths share "which writes take three states" |
| 3 | O1 (memory model fuzzing (HashMap ideal filesystem)) - applicable state count for crash judge (device) | In one state, if a persistent or torn write belongs to the last call that is not earlier than the call where the model stopped, this state does not apply | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1937` `crash_judge_is_applicable` | Return points in the call table and stream order indices in the write table |
| 4 | O1 (memory model fuzzing (HashMap ideal filesystem)) - applicable state count for crash judge (comparator current calculation) | From `calls.tsv`, `ops.tsv`, `states.tsv`, count states written into the vector landing inside the `unmount` call | `scripts/experiment-apparatus/e171_three_oracle_collusion/baseline_checks.py:184` `recompute_crash_judge_applicable` | Return points in `calls.tsv` (device outputs these); model stops only at this one read method (`unmount`) (registered as first read method 6) |
| 5 | O2 (independent parser + checker) judgment on bad state rows (device) | Reconstruct `MemoryPool` from neutral format using landing vectors, stack patches written by E170 (first version of O3 specification executor), call `check_pool_image` | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:4751` `e170_bad_rows_on_both_streams` | Pool-level checker itself, `bad-patches.bin` written by E170 (first version of O3 specification executor), and landing vectors |
| 6 | O2 (independent parser + checker) judgment on bad state rows (E170 (first version of O3 specification executor) export device) | On `CrashImage`, append patches to the end of the write table as persistent system configuration slots, call `judge_crash_image` to retrieve one item from pool-level checker | `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:3014` `observe_bad_row` | Pool-level checker itself; same batch of patch bytes and landing vectors |
| 7 | First re-run: O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Number of states where the crash judge is applicable and number of states voted by RefFS (Device) | Applicable: The last call to which a write that fell in one state belongs occurred before the call at which the model stopped; Vote: RefFS translation did not stop before the moment of crash in this state | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3888` `crash_judge_is_applicable`; same file `:3172` `reference_filesystem_vote_of_a_state` (line numbers are from the version in commit `ab35105f…`; the version in revision 6 tree has 7 fewer lines after line 3475) | Return points of the call table and stream order indices of the write table; translation stop point of `reference_filesystem_translation.rs` |
| 8 | First re-run: O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Number of states where the crash judge is applicable and number of states voted by RefFS (Comparator Computation) | Recomputed from `calls.tsv`, `ops.tsv`, `states.tsv`: for each write, which call it fell into; the last write that fell belongs to a call earlier than the one at which the model stopped; the moment of crash takes the step that closes this segment, voting occurs before steps like `unlink` or unsuccessful rollbacks; all states across three streams are enumerated; enumeration points do not sweep through | `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py:176` `recomputed_applicable_and_voting` | `calls.tsv` written by the device (first op, return point), `ops.tsv`, landing sequences and segment numbers of `states.tsv`, `model_stopped_at_call` in `stream-status.tsv`; the comparator writes its own copy following the `SegmentClosingRule` for segment splitting rules |

Path 7 and Path 8 both exhibit stream-by-stream comparison (per-stream comparison) in the output of the first re-run of the comparator (`…-s1-comparator-resume2-r6-r3.out`), where all three lines with `name=a11` show `holds=true`; similarly, there is no `assert` connecting the two directly; both paths share a device that writes out the call table and landing sequence, and they are incorrect together (both checks fail together).

Paths 1 and 2, as well as Paths 3 and 4, are compared cell-by-cell in the artifacts (`name=s5_batch_1_states`, `name=s5_o1_applicable` with eight rows where `equal=true`); Paths 5 and 6 are compared row-by-row (`name=e170_bad_row` with 76 rows where `o2_equals_e170=true`). These comparisons are recorded in the artifacts, but there is no `assert` that directly connects the two paths; this link is still missing. Among the shared items, the most critical component is the pool-level checker itself (Paths 5 and 6 share the same `check_pool_image`): since it fails both cases together, this cell cannot serve as evidence that O2 (independent parser + checker) is correct; Paths 1 and 2 share "which write-read tri-state occurrences," and since they fail both state counts together, this also does not constitute valid evidence.

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:1057-1176`（整段抄，未转述）**

````markdown
### Re-run

Devices and the E170 (first version of the specification executor) export devices are both entry-point devices for checker archives, wrapped in memory; the research side is implemented in Python. In `scripts/experiment-apparatus/replay.sh`, the original eleven lines registered for E171 (three-oracle collusion test) do not appear in this table: the first run's nine other lines (`grep -c '^E171（三oracle合谋测试）|@driver_e171_r1_' scripts/experiment-apparatus/replay.sh`) are located as follows—the first three rows of Revision 6 are under the section "Comparison, Re-run, and Entry Methods (Revision 6 Run)", the next two rows of the first re-run are under the section "First Re-run: Second Segment - Revision 6 Run", and one row from Revision 12 is under the section "First Re-run · Revision 12: Three-Stream Long Re-nailing (2026-10-13)"; however, the three rows of research-side mutations for Revisions 8, 10, and 11 (driver functions `driver_e171_r1_s1_revision_8_tools_research_mutations`, `driver_e171_r1_s2_revision_10_tools_research_mutations`, `driver_e171_r1_revision_11_tools_research_mutations`) have no corresponding sections in their respective pages. Each carries its own temporary directory, and only the E7RESULT line and exact comparison are retained:

| Driver Function | Output | What It Runs |
|---|---|---|
| `driver_e171_s1_baseline` | `…-s1-baseline-run1.out` | The device `baseline` runs once |
| `driver_e171_s1_known_bad_images` | `…-s1-known-bad-images.out` | Baseline, E170 (first version of the O3 specification executor) exports the device `bad-states` (placeholder seed), and the device `known-bad-images` |
| `driver_e171_s2_bad_states_export`, `driver_e171_s2_known_bad_images`, `driver_e171_s2_images_o3`, `driver_e171_s2_o3_control_baseline` | `…-s2-bad-states-export.out`, `…-s2-known-bad-images.out`, `…-s2-images-o3.out`, `…-s2-o3-control-baseline.out` | The construction of the three families of bad images under a frozen seed; judgment by O2 (independent parser + checker) and the frozen specification executor; positive control for O3 (independent specification executor) |
| `driver_e171_s2_comparator` | `…-s2-comparator.out` | The sequence from the second comparator run (nine trial runs); the comparator was modified in phases three through six regarding judgment lines and config lines; this re-run does not match, so no modification index is recorded |
| `driver_e171_s3_comparator_mutations` | `…-s3-comparator-mutations-2026-10-03-r7.out` | Research on one side: 67 mutations, using `mutation_driver.py`, approximately one minute |
| `driver_e171_s4_comparator` | `…-s4-comparator.out` | The sequence from the second phase; modified line-by-line driver runs all 117 lines (two lanes, two threads per lane, one frozen tree); Q3-core three runs; comparator; estimated duration approximately two hours; this driver function has not been truly run yet (heavyweight, intended to re-run all experiments at submission time). The readings for the 117 lines read from the current `crates/` implementation; if O2 (independent parser + checker), O1 (memory model diffing with HashMap ideal file system), the device, or the text of those 117 lines changes, the output changes. The entire table step includes `--keep-compile-failures-as-they-are`: this output is from a driver run that failed to compile and did not patch |
| `driver_e171_s4_f4_comparator` | `…-s4-f4-comparator.out` | First, run `driver_e171_s4_comparator` entirely; then, on that same frozen tree and working root, run the expanded table of 164 lines (also with `--keep-compile-failures-as-they-are`), split the comparator in half, include expansion, section name s4-f4; estimated duration approximately four hours; not truly run yet (`bash -n` checked); the comparator half was re-run on existing exports using the same parameter set and produced byte-for-byte identical output |
| `driver_e171_s4_f4_substitutes_comparator` | `…-s4-f4-substitutes-comparator.out` | First run the entire test suite for `driver_e171_s4_f4_comparator`, leaving its draft directory with exports and a frozen tree (frozen tree) for s4 and F4, then on that same frozen tree and working root, run only those lines from the two output roots that end up failing compilation (`--rerun-compile-failed-from`), skipping all names in both tables as placeholders, using two threads per line; use a comparator with `--substitution-rows-root`, segment name s4-f4-substitutes, and combine results 117 + 164 + placeholders. The estimated duration is about four and a half hours (approximately four hours for the s4 and F4 sequence plus 1277.9 seconds for placeholders). This has not actually been run yet (`bash -n` checked; each step's parameters were verified line-by-line against the actual command used in that placeholder run, without re-running the driver or comparator). Read the current `crates/`; if the implementation, O2 (independent parser + checker), O1 (memory model diffing with HashMap ideal file system), device, or those six lines of text change, the artifacts will also change. (`--keep-compile-failures-as-they-are`)

Re-run the comparator for 2026-10-03 using s4's input (the existing exported directory and driver output); the E7RESULT line matches `…-s4-comparator.out` byte-for-byte (`cmp` produces no output): this half of the comparator is reproducible; the device and driver half requires re-running the entire table to verify. The first few sample single commands are:

```
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- baseline --output-directory <新目录>
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- bad-states --dump-directory <基线导出目录的一份拷贝> --seed-hex 0000000000000000
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- known-bad-images --dump-directory <上一步那份拷贝> --output-directory <新目录> --e170-bad-rows
cd scripts/experiment-apparatus/e171_three_oracle_collusion && python3 mutation_driver.py --table ../../../../mutations/e171_three_oracle_collusion.tsv --scratch <草稿目录> --output <新文件>
```

The first segment comparator (`…-s1-comparator.out`, where the `driver_e171_s1_comparator` from the first segment has been replaced by the one from s2 and is no longer registered) must first run two baseline passes, export the device's `dump` via E170 (the first version of the O3 specification executor), close a gate twice, run six positive controls (three baselines and three empty mutations using `row_driver.py --empty-mutation --positive-control <name>`), drive three rows line-by-line (`row_driver.py --empty-mutation --rows 1,6`), and then execute `python3 comparator.py …`, totaling approximately a dozen device runs (each taking about one minute); the driver function hardcodes this sequence. The comparator's config lines only specify directory names, so the directory names in re-runs must match those in the driver. The comparator reads the current `crates/mutations.tsv` for rows a4–a7: if additional rows are added elsewhere in the table or any of these 117 rows are modified, these rows will change; this does not indicate device failure. Blind test rows generated by the placeholder seed `0000000000000000` for E171 (the three-oracle conspiracy test) are not used. E170 (the first version of the O3 specification executor) added a second set of bad-state tables (`bad-states-second-run`) during its first re-run; `bad-states` remains the first set, and the outputs from both versions of the export device on the first set are identical byte-for-byte (as stated in the final item under "Comparison and Self-Check").

Re-running the driver's self-check: The baseline, known-bad image, and mutated driver functions are executed in the draft directory according to the `replay.sh` specification, producing outputs that match the committed artifacts byte-for-byte (with no output from `cmp`). For the comparator driver function, a full end-to-end run was not performed (as it involves dozens of devices and cannot be completed on the foreground system); instead, its glue code (command-line steps for each phase, directory names, and parameters passed to the comparator) was tested using stubs that copy from existing export directories rather than regenerating them line-by-line. The comparator itself ran as if in production, producing outputs matching the committed artifacts byte-for-byte. Each individual command was fully executed during artifact generation step-by-step. Full end-to-end verification is performed upon submission via re-running.

Artifacts (all located under `research/results/`, with names prefixed by `e171-three-oracle-collusion-2026-10-03-s1-` followed by):

| Filename (after prefix) | Content |
|---|---|
| `baseline-run1.out`, `baseline-run2.out` | E7RESULT lines for two baseline runs (identical byte-for-byte in both copies) |
| `baseline-timing.out` | E7TIMING lines for two runs (duration per batch and segment, per stream driver, per flag red judgment, and per write operation) |
| `known-bad-images.out` | Device output in the `known-bad-images` mode: 41 known bad images with 76 rows of bad states on O2 (independent parser + checker) from E170 (first version of specification executor) |
| `comparator.out` | Comparator registration anchor, baseline and reference comparison, one line disabled, three lines trial run, Q1 denominator, and a column for O2 (independent parser + checker) |
| `rows-timing.out` | E7TIMING lines for per-row driving and comparator (Q4 single row count) |
| `comparator-mutations.out` | Nine mutations on the Python side |
| `e170-pinned-dump.out` | Dump of the pinned version from E170 (first version of specification executor), serving as reference for S2 |
| `e170-bad-states-placeholder-seed.out` | Bad states from the same version (source of P and N rows; blind test lines are not used) |
| `positive-control-o1-comparison.out`, `positive-control-o1-crash-judge.out`, `positive-control-o2.out` | Device output for three positive controls during the baseline run |
| `disable-o1.out`, `disable-o2.out` | Device output for switch self-checks when one line is disabled (one run each on the baseline) |
| `row-0-device.out`, `row-1-device.out`, `row-6-device.out` | Device output for three lines trial run (row-0 is an empty mutation) (`dump`, `bad-states`) |

Full names (`ls research/results | grep '^e171-three-oracle-collusion-2026-10-03-s1-'`, 17 files):

```
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out
e171-three-oracle-collusion-2026-10-03-s1-baseline-run2.out
e171-three-oracle-collusion-2026-10-03-s1-baseline-timing.out
e171-three-oracle-collusion-2026-10-03-s1-comparator-mutations.out
e171-three-oracle-collusion-2026-10-03-s1-comparator.out
e171-three-oracle-collusion-2026-10-03-s1-disable-o1.out
e171-three-oracle-collusion-2026-10-03-s1-disable-o2.out
e171-three-oracle-collusion-2026-10-03-s1-e170-bad-states-placeholder-seed.out
e171-three-oracle-collusion-2026-10-03-s1-e170-pinned-dump.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-comparison.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-crash-judge.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o2.out
e171-three-oracle-collusion-2026-10-03-s1-row-0-device.out
e171-three-oracle-collusion-2026-10-03-s1-row-1-device.out
e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out
e171-three-oracle-collusion-2026-10-03-s1-rows-timing.out
```

Products from the second paragraph onward (`ls research/results | grep -E '^e171-three-oracle-collusion-2026-10-03-s[234]-'`, 29 files, with the prefix `e171-three-oracle-collusion-2026-10-03-` followed by):

| Filename (after prefix) | Content | Does it Count? |
|---|---|---|
| `s2-bad-states-export.out`, `s2-known-bad-images.out`, `s2-images-o3.out`, `s2-o3-control-baseline.out`, `s2-o3-baseline.out` | Methods for generating bad images of the three families, and judgments by O2 (independent parser + checker), O3 (independent specification executor), positive controls using O3 (independent specification executor), and a frozen tree with the executor at baseline (one copy of O3 (independent specification executor) from a3). | Yes: The s4 comparator reads these files. |
| `s2-comparator.out`, `s2-rows-timing.out` | The second comparator (Q2 full table, nine-row trial run) and its timing. | Q2 numbers match those of s4; the nine-row trial run is replaced by the s4 full table. |
| `s2-comparator-mutations.out` | Full-table run with 26 mutation entries in the mutation table. | Historical data. |
| `s3-baseline-after-the-fix.out`, `s3-row-0-device.out` | Device baseline after Revision 3 fixes, following number retrieval and flagging red  of warm-up records, alongside an empty mutated row that is identical byte-for-byte to the pre-revision state. | Yes: Core implementation of Revision 3. |
| `s3-k4c-attack-row-device.out`, `s3-k4c-check-driver.out` | Revision 3: A row that constantly rejects number retrieval, run line-by-line via driver (O1 (memory model replay with ideal HashMap filesystem) flagged red). | Yes: Core implementation of Revision 3. |
| `s3-comparator.out`, `s3-rows-timing.out` | Comparator and timing for the full s3 table of 117 rows. | Voided but archived (see section "Voiding the Full s3 Table"). |
| `s3-clean-check-comparator.out` | Fifth segment: Reverification on clean working roots for rows 2, 27, 42, and 65. | Supporting evidence for voiding; archived. |
| `s3-comparator-mutations.out`, `s3-comparator-mutations-2026-10-03.out`, `-r2.out`, `-r3.out` | Full-table runs with 37, 40, 44, and 44 entries in the mutation table. | Historical data. |
| `s3-comparator-mutations-2026-10-03-r4.out` | The run with 52 entries that was interrupted mid-process (61 rows, no verdict row). | Does not count; archived. |
| `s3-comparator-mutations-2026-10-03-r5.out` | Full-table run of the mutation table with 52 entries: 52 / 0 / 0. | Historical data. |
| `s3-comparator-mutations-2026-10-03-r6.out` | Full-table run of the mutation table with 65 entries (after adding x60–x72): 65 / 0 / 0. | Historical data. |
| `s3-comparator-mutations-2026-10-03-r7.out` | Full-table run of the mutation table with 67 entries: 67 / 0 / 0. | Yes: Re-run registration points to this file. |
| `s4-comparator.out`, `s4-rows-timing.out` | Comparator and timing for the full s4 table of 117 rows, Q3 core run three times. | Yes: Readings from half the 117 rows (results rows 1–117 in F4 products are byte-for-byte identical to this). |
| `s4-f4-comparator.out`, `s4-f4-rows-timing.out` | Comparator and timing for merging the expanded F4 table of 164 rows with s4's 117 rows. | Yes: Readings prior to filling gaps (the copy in re-run registration with `--keep-compile-failures-as-they-are`). |
| `s4-f4-substitutes-comparator.out`, `s4-f4-substitutes-rows-timing.out` | Comparator and timing for merging 117 + 164 rows plus the gap-filling run that failed to compile (the gap-fill run took 1,277.9 seconds). | Yes: Readings from Q1, Q3, and Q4 after filling gaps (Q1 and Q3 lists are byte-for-byte identical before filling; product row numbers for three verdict rounds come from this file). |

Full Name:

```
research/results/e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out
e171-three-oracle-collusion-2026-10-03-s2-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out
e171-three-oracle-collusion-2026-10-03-s2-o3-baseline.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out
e171-three-oracle-collusion-2026-10-03-s2-rows-timing.out
e171-three-oracle-collusion-2026-10-03-s3-baseline-after-the-fix.out
e171-three-oracle-collusion-2026-10-03-s3-clean-check-comparator.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r2.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r3.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r4.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r5.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r6.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations.out
e171-three-oracle-collusion-2026-10-03-s3-comparator.out
e171-three-oracle-collusion-2026-10-03-s3-k4c-attack-row-device.out
e171-three-oracle-collusion-2026-10-03-s3-k4c-check-driver.out
e171-three-oracle-collusion-2026-10-03-s3-row-0-device.out
e171-three-oracle-collusion-2026-10-03-s3-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out
e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-rows-timing.out
e171-three-oracle-collusion-2026-10-03-s4-rows-timing.out
```

````

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:1177-1201`（整段抄，未转述）**

```markdown
### What It Cannot Answer

- Q1's "has" is judged literally (≥ 1 row); among the 12 rows (37 total in batch 1, 38 in batch 2), which ones count as truly bad: the three-round conclusion verdict assigned a score of three to seven true blind-spot candidates (rows 15, 23, 42, 242, 269, 270, 271); the clauses governing invalid states for the remaining categories are documented in the sections "Seven Rows Where All Three Rounds Passed" and "F4 Newly Added Rows Where All Three Rounds Passed"; the score of three is merely an interpretation of the state obtained in this batch; historically, rows have been separated based on inference rather than execution, as most were not run. The denominator D = 37 remains unchanged before and after the patching (the six patched rows did not reach execution), so Q1's "has" has not flipped.
- In row 281, row 238 was not reached (after patching; before patching it was row 232, with another 6 rows failing to compile): the historical trajectory of these two streams only covered a small portion of implementation (one or two instances each for numbering, warm-up, release, writable mount, admin rollback, raising F, and normal unmount; no splits, no deletions, no space-constrained pools); "not reached" indicates that the write stream modification at that specific location remained unchanged on these two streams, without specifying which oracle covered it; among them, seven modifications were executed but not written to disk (rows 49, 110, 155, 161, 162, 174, 252, detailed in the section "Two Rows That Were Red Despite Not Being Reached and Seven Rows Where Return Values Differed Among Unreached Rows"); after three rounds, no row where all three rounds passed was recorded as "equivalent to this batch".
- The definition of "reach" (Registration Read Method 4) does not equate to the return value of a successful call; this rule remains unchanged (registration is not modified), keeping the denominator at 37: rows 155 and 161, which raised the F limit with a return value of 13 (model 11), share identical write streams and outcomes but are recorded as unreached, whereas O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Crash Judge flagged them red; the attacker measured successful call return values for 232 unreached rows on copies of the s4 tree, finding differences in 7 rows (rows 49, 110, 155, 161, 162, 174, 252); the summary line is `name=return_summary hidden_difference=7 identical=225 no_output=6` (copy probe, s4 tree; the ingestion device lacks this column; full row details and provenance are in the section "Two Rows That Were Red Despite Not Being Reached and Seven Rows Where Return Values Differed Among Unreached Rows"); mkfs's 6 rows (rows 149–154) were not measured.
- The six rows that failed to compile have already been patched once ("Six Rows That Failed to Compile: Patching Run Completed" section); the s4 and F4 artifacts were generated by drivers running before patching; re-running their registration lines with `--keep-compile-failures-as-they-are` preserves the pre-patch state without applying patches.
- Order-dependent clauses (rows 242, 270) are counted on replayed segments of the crash-point model: "states forbidden by clauses" can be distinguished via segment rules (FUA writes only allow their own disk; barriers allow their own disk); however, the actual ordering between a single FUA write and subsequent writes in real hardware is not included in this batch.
- O3 (Independent Specification Executor) did not flag any of the 37 rows red: its A(c) consists of roots written by the implementation itself, targeting cases like "the version restored to falls outside [ack, begun]"; it still judges fields consistently written incorrectly by the implementation (container number, write time, F value, accounting row) as "in". This is derived from reading the executor code; no additional rows were created solely to make O3 (Independent Specification Executor) flag bad implementations red (see Q2).
- The three trial runs (first paragraph) are not samples of the table; the entire s4 table replaced them for Q1 readings; row 1's "unreached" status indicates that this specific modification did not progress in the history of these two streams.
- On the baseline, O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Crash Judge has states that are not applicable: Stream Two's last `unmount` call yielded no model response, and states written during this call were not judged; among 397 states in Batch 1 Stream Two, 48 were not applicable (`o1_crash_judge_not_applicable`), and among 119 states in Batch 2, 14 were not applicable (the O1 column is empty for these states); Stream One had no non-applicable states.
- For the positive control, O1 (Memory Model Fuzzing (HashMap Ideal Filesystem)) - Crash Judge flipped once per stream (total 2 instances), with literal registration stating "first in Batch 1"; O2 (Independent Parser + Checker) control involved stacking only the first known bad image on Stream One's fully persistent states, with no other sampling points; this control demonstrates that the pipeline carries red flags into the matrix but does not prove that each oracle can distinguish every type of corruption.
- The Q2 O2 (independent parser + checker) column is derived from two paths (paths 5 and 6); the patch bytes and landing vectors in row P are written by E170 (first version of the O3 specification executor), while the expectation comes from what designer D pushed (registered a2, F3). All four Q2 items that were passed by all three are P8; one judgment is attributed to F3. The 41 known bad images are O2's own regression corpus for the independent parser + checker; the fact that all 41 of these turn red is an artifact of construction, not evidence of independence for O2 (independent parser + checker).
- Line-by-line driving and device reads at the moment of execution: s4 performed five drives using the same frozen tree, but other parts of `crates/` were changing; the numbers produced by running the entire table only reflect that single frozen tree.
- On bad images, what recovery read back (registered reading method 1 in section 1, with attachment); regression tests for old seed blind test families were not run. Q1-D (three rows passed by all three while the record checker or crash-point re-run on a handwritten version table turned red) was executed: none of the 12 rows in F4 that were passed by all three (categories A and B) showed this (`name=q1_delta … =none`); the comparator calculated only based on categories A and B for this cell, but category C added row 233 where the crash-point re-run caused the oracle and record checker to turn red (`name=q1_quiet_row reading=q1_gamma row=233 … layer0_or_record_checker_red=true`); additionally, the oracle and record checker from the crash-point re-run turned red on rows 108 and 115 when relying solely on O1 (memory model diffing with HashMap ideal file system).
- Device inline unit tests were not run (checker archive); there are no three numbers for the 15 mutated lines on the device side; mutations on the Python side only prove that the comparator and driver predicates can turn red, but do not prove that the device output they read is correct (device output was separately pinned by two baseline runs, S2, S5, and a control).
- In E171 (three-oracle conspiracy test), the O1 column (memory model diffing with HashMap ideal file system) represents model diffing plus the model's crash judge, without connecting to RefFS (registered: include the crash judge in O1 (memory model diffing with HashMap ideal file system); write a separate E171 (three-oracle conspiracy test) that does not use RefFS). This differs from item 6 of D13 (verification route), which defines six rows for O1 (memory model diffing with HashMap ideal file system) where the set visible after a crash is attributed to RefFS differently; the lower bounds also differ (the latest root persisted on disk versus the last successful return release). The category "confirmed but lost" appears only in the O3 column (independent specification executor) among the four columns of E171 (three-oracle conspiracy test) and was not run against 37 rows using RefFS (code read out by reading). RefFS numbers appear only in round two of three rounds, specifically in cell R6 of section 2 (`research/prompts/e171-conclusion-r2-main-verification.md`), with readings in `judge-summary.out`, attacker replica probes, s4 tree core, direct crate invocation for RefFS, manually provided crash landing points, and bypassing the translation layer: in the warm-up configuration, O3 (independent specification executor) judged "not present" for 27 states, and RefFS also judged green for these same 27 states (27 / 27); for content-bearing isomorphic write streams, all 27 states where O3 (independent specification executor) judged "not present" turned red in RefFS; among a total of 36 states that turned red in RefFS, an additional 9 states were red in RefFS but judged "present" by O3 (independent specification executor) (due to differing rules for attributing crash moments, as designed). These numbers are not device readings from E171 (three-oracle conspiracy test).
Source identifiers: `crates/`.
- In Q2, O2 (independent parser + checker) allowed through while O3 (independent specification executor) flagged as absent the 29 images (12 from Family P, 17 from Blind Test Family B; see the section "Q2 where O2 (independent parser + checker) allows through but O3 (independent specification executor) flags absent bad mirrors"). All are structurally self-consistent forged roots and forged continuation mirrors, not bad implementations. The patches and expectations both originate from E170 (first version of O3 specification executor) modification code (E170 (O3 specification executor first version) page "What it cannot answer"), with the source independently measured in Q2 only for the pair O2 (independent parser + checker), O3 (independent specification executor); do not treat as independent readings. O1 (memory model diffing (HashMap ideal file system)) is not in Q2; Stream One P2 and P5 patch variants' O2 (independent parser + checker) column flips with the baseline state. These 29 images re-run against the current `crates/` do not reproduce s4's readings: the baseline stream differs from the s4 tree, so the counts are not guaranteed to remain 12 and 17. D13 (verification route) segment references only this count on E171 (three oracle collusion test) page.
- The row-level readings on this page (full allow list, relying on a single line, unreachability lists, and state per row) belong exclusively to the snapshot summary of that frozen tree: it is not the same as the current `crates/`, so the lists in the page cannot be directly used as the current implementation's lists. Re-running E171 (three oracle collusion test) via `replay.sh` reads against the current `crates/`; a single re-run produces readings from that specific tree at the moment of re-run. On s4's version of the repository ingestion device (`scripts/experiment-apparatus/e171_three_oracle_collusion/`), besides Q2's 29 images, these other counts cannot be reproduced exactly (this version lacks mount mode, lacks RefFS column, and panic streams are not exported; the first re-run device `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` has the RefFS column, but whether mount mode and panic stream export are verified is unclear).
- Row 125 is one of the six self-collapsing rows: registration reading method 5 records panic as self-collapse, excluding it from the denominator, D = 37. Before panic, there were 13 callable invocations that could be judged: the replica probe drove on the s4 tree, applying mutation to row 125; calls 4–13 had full persistent state O2 (independent parser + checker) flagging I-3.11 (allocated minus defer equals latest root walk) as red; call 14 (`unmount`) panicked (`drive-s4-m125.tsv`), and the device does not export the entire panic stream. Treating panic and errors identically by truncation, including in denominator D = 38 (preliminary rehearsal of changed definition, not device reading; write both 37 and 38); if reached, also include return values from successful calls, including rows 155 and 161, D = 40 (37 plus rows 125, 155, 161); include all 7 rows with differing return values, including row 125, D = 45 (inferred, arithmetic).
- Predicate W (correct implementation's writable mount would read this state as "witnessed update version unreadable") is true on the crash states obtained at rows 242 and 269 for 15 and 63 cases respectively; baseline 0 / 576. On s4 tree core, two writable mounts rejected 61 witness-class instances (row 242: 9, row 269: 52). These counts were measured by the attacker replica script (W) and replica probe (mount) on the s4 tree; the ingestion device lacks W and also lacks mount mode. The core read from 2026-10-03 writing out images shows widespread `refused:Undecidable` (baseline stream one 44 / 60, stream two 500 / 516): format changed (journal header 311 → 487, new root segment 188 → 364, naming item 67 → 64); old records are unreadable under the new parser, not a regression; therefore W's 15 / 63 and 61 witness rejections apply only to s4 core and have no comparative significance on the current core.
- Whether the value of the user data landing point counts as a layout-registered byte is not adjudicated on this page; three rounds of judgment are handed to the user (`research/prompts/e171-conclusion-r3-main-verification.md`, Section 5, Line 1, Round 0). The user determined "counts" on 2026-10-03: D3 (Space Allocation) has established Item 8, changing the range to landing point values on new files created in a new pool (t1@50180), which counts at the format level. Verification was performed in C673 (the fix for the missing oracle in the three-oracle collusion test, item ⑥). This debt has been repaid (see the table of repaid debts in the accounts receivable ledger). It does not change this page's three-part structure: the basis on Line 15 is t2–t12 landing points from Item ⑤; the landing point modification on Line 34 occurred during the release after raising F in Stream Two, not on new files created in a new pool, and thus remains classified under allocation policy.
- The re-run device for the new tree (`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`, whose readings are not recorded on this page) outputs two line types: `name=batch` and `name=verdict`. For `piece_panic_states=`, count by piece, not by state: format as `<piece>:<state_count>`, joined by commas; omit pieces with a value of 0, and write `-` if all are 0. The same panic instance is recorded once in each of the `layer0_oracles` and `crash_judge` columns (Round 27–29 code round judgments: `research/prompts/m3-formal-code-b29-r1-main-verification.md`, Section 3, Item 3; both legs judged identically; do not change the device).
- The old Line 38 of the mutation table has been renamed to "Step 4: Rollback candidate set's F uses the F carried by the target root itself rather than F_take effect" (after renaming, it appears in `crates/mutations.tsv`; the old name was written as "F carried by the latest root itself"). Both copies of `bad-implementations-117.tsv` (located under `scripts/experiment-apparatus/e171_three_oracle_collusion/` and `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/`) reference Line 33 using the old name. When taking a snapshot from the current tree to re-run, this line cannot be found by the old name, triggers a fallback mechanism, and does not silently fail (Revision 6 currently running reads the old table from the frozen tree and is unaffected; Round 32–33 code round judgments: `research/prompts/m3-formal-code-b33-r1-main-verification.md`, Section 2, P4 "Old Line 38 Renamed" line, and Section 3, Item 5).

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:1202-1220`（整段抄，未转述）**

```markdown
### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D13 (Verification Path) Fixed Item 6 | Supports | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D13 (Verification Path) Fixed Item 1 | Preparation | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D13 (Verification Path) Fixed Item 7 | Preparation | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D13 (Verification Path) Fixed Item 15 | Preparation | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D8 (Core Index Structure) Fixed Item 6 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D16 (Publish Semantics) Fixed Item 1 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D16 (Publish Semantics) Fixed Item 7 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D16 (Publish Semantics) Fixed Item 12 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D8 (Core Index Structure) Fixed Item 8 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D23 (Role and Format of journal) Fixed Item 3 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D23 (Role and Format of journal) Fixed Item 14 | Preparation | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D23 (Role and Format of journal) Fixed Item 15 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D3 (Space Allocation) Fixed Item 8 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |
| D3 (Space Allocation) Fixed Item 10 | No Impact | 2026-10-18 Unaffected: The product of three re-runs of the mutation table for one side in the device section's supplementary naming study, with unchanged conclusions and numbers. |

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:1221-1232`（整段抄，未转述）**

```markdown
## Historical Versions

### 2026-10-03: Page Construction; O3 (Independent Specification Executor) Continuing from Previous; s3 Entire Table Invalidated; s4 Entire Table; F4 Expanded Table Completed; Rows That Failed Compilation Padded and Completed; Conclusion: Three-Round Verdicts Written Back

- During the initial page build, the first segment had already run: device-side and research-side code along with unit tests were written; two baseline runs and three positive controls (one for each of baseline and empty mutation), a single switch self-check, Q2's O2 (independent parser + checker) column, and row-by-row driving completed three test runs. The O3 (independent specification executor) column had not run (shutdown registered as S7). The page status was set to "First segment run".
- Segments two through six (the page did not change per segment): O3 (independent specification executor) re-ran the frozen executor connected via E170 (first version of the O3 specification executor), completing the full Q2 table (main agent judged 4 P8s according to F3); device-side added number retrieval, warm-up error flagging red, and row-by-row S5 (revisions registered as 3 and 4). After s3 completed the entire table, it was found that a mutation from the previous row had been carried into the next row, invalidating the whole table (revision 5); driving logic was changed to set modification time per row and recompile upon verification.
- The full s3 table of 117 rows ran on the same frozen tree; Q1, Q3, and Q4 were judged according to s4. D = 19 < 30. F4 expanded the table: reading logic for 164 additional row files was written in driver and comparator, but this full-table run remained pending. The page status was changed to "Partially run (s3 full table of 117 rows run, F4 expanded table of 164 rows pending)"; previously it was "First segment run". Before the change, all four rows in the current status table were marked as "open".
- After running the F4 expanded table of 164 rows, the comparator merged into 281 rows (`…-s4-f4-comparator.out`): Current status table Q1 changed from "7 rows each for A/B/C, D = 19" to "12 rows each for A/B, 13 rows for C, D = 37 (before padding)", Q3 changed from "11 rows relying on a single path" to "23 rows", and Q4 added the clock time for that expanded table run. Added new entries: all-pass rows from the expansion, 12 rows relying on a single path, and two sections of three rows that were red without being reached. Row-by-row driving was changed to also perform padding after compilation failure (s3 and F4 outputs were generated by pre-change driving; re-runs with switches remain unchanged regarding no padding). The 6 rows failing compilation are pending for the padding run. Page status changed to "Partially run (s3 full table of 117 rows and F4 expanded table of 164 rows run, 6 rows failing compilation pending padding)". Research-side unit tests increased from 98 to 110 lines; mutations from 52 to 67. Re-runs registered mutation outputs now point to `-r7.out`, adding one line for `driver_e171_s4_f4_comparator`.
- After completing the run for the 6 rows failing compilation (`…-s4-f4-substitutes-comparator.out`, `…-s4-f4-substitutes-rows-timing.out`): Page status changed from "Partially run (s3 full table of 117 rows and F4 expanded table of 164 rows run, 6 rows failing compilation pending padding)" to "Run complete (...readings collected...)". Denominator D = 37 remained the same before and after padding; Q1 and Q3 lists are identical character-for-character. The section "(Pending: 6 rows failing compilation padding)" was changed to "(Completed)". The originally proposed first set of replacements (row 33 → index 9 in mount.rs group, row 59 → index 8 in mounted_read.rs, row 127 → index 16 in allocator.rs, row 163 → index 37 in mount.rs, row 206 → index 4 in recovery.rs, row 243 → index 25 in transaction.rs, assuming the first replacement row compiled successfully) was removed; actual replaced rows are present in the outputs (the previous replacement for row 127 also failed compilation). Re-runs registration added one line `driver_e171_s4_f4_substitutes_comparator` (10 lines → 11 lines); product count increased from 44 to 46.
- Conclusion: Three-round verdict write-backs (`research/prompts/e171-conclusion-r1-main-verification.md`, `…-r2-main-verification.md`, `…-r3-main-verification.md`): The three-column layout was changed to current values; the pre-change three columns were as follows—Line 15 "Equivalent for this batch" (reason: "Clause not defined," historically inferred to be separated), Lines 27, 112, and 277 "Equivalent for this batch", Line 34 "Equivalent for this batch", Line 92 "True blind spot candidate (clause basis must be recognized by the main agent)", Q1 row wrote "True blind spot candidates: 4 rows (Lines 242, 269, 270, 271); Equivalent for this batch: 1 row (Line 277)"; s4's 7 rows "Three-round verdicts not re-executed". After the change, Lines 15, 23, 42, 242, 269, 270, and 271 are true blind spot candidates (Line 15 is based on D3 (Space Allocation) defined item 10⑤'s t2–t12 landing points); Lines 27, 112, 277, Line 34, and Line 92 were reclassified into other categories; a new section "Three-round verdicts subsequent three columns" was added.
- Other changes in the same write-back: In the section "Two rows that are red but not reached" and within "What it cannot answer," the clause regarding reachability was modified. Pre-change, it stated: "Whether other unreached rows have similar differences is unclear; the device did not record successfully returned values." Post-change, it states: The attacker measured 232 rows in the s4 tree copy, finding 7 rows with different return values (`name=return_summary hidden_difference=7 identical=225 no_output=6`); the definition of reachability and the denominator of 37 remain unchanged. Q2 row and the new section "Q2: O2 (Independent Parser + Checker) allows passage while O3 (Independent Specification Executor) judges bad mirrors as non-existent" were supplemented with 29 images (12 + 17). The section "What it cannot answer" was updated to add a column for RefFS and O1 (Memory Model Checksumming: Ideal Filesystem HashMap), the source of the 29 images, row-level readings belonging only to that snapshot tree, a rehearsal for Line 125 D = 38, W and 61 witness rejections holding only for s4 core, and whether user data landing point value calculations count as layout bytes left to the user's discretion. In "Affected Decisions," D13 (Verification Route) defined item 6 was changed from "Preparation" to "Support" (pre-change reason: segments were not cited by number referencing E171 (Three Oracle Collusion Test); which rows constitute true blind spots are inferences; the three-party route was not taken). Added D3 (Space Allocation) defined items 8, 10, and D23 (Journal Role and Format) defined item 15. Review text for D13 (Verification Route) defined items 1, D8 (Core Index Structure) defined item 6, and D23 (Journal Role and Format) defined item 14 was added to the clauses newly utilized in the three-round verdicts.

```

**出处 `.claude/kb/experiments/171-three-oracle-collusion-test.en.md:1223-1232`（整段抄，未转述）**

```markdown
### 2026-10-03: Page Construction; O3 (Independent Specification Executor) Continuing from Previous; s3 Entire Table Invalidated; s4 Entire Table; F4 Expanded Table Completed; Rows That Failed Compilation Padded and Completed; Conclusion: Three-Round Verdicts Written Back

- During the initial page build, the first segment had already run: device-side and research-side code along with unit tests were written; two baseline runs and three positive controls (one for each of baseline and empty mutation), a single switch self-check, Q2's O2 (independent parser + checker) column, and row-by-row driving completed three test runs. The O3 (independent specification executor) column had not run (shutdown registered as S7). The page status was set to "First segment run".
- Segments two through six (the page did not change per segment): O3 (independent specification executor) re-ran the frozen executor connected via E170 (first version of the O3 specification executor), completing the full Q2 table (main agent judged 4 P8s according to F3); device-side added number retrieval, warm-up error flagging red, and row-by-row S5 (revisions registered as 3 and 4). After s3 completed the entire table, it was found that a mutation from the previous row had been carried into the next row, invalidating the whole table (revision 5); driving logic was changed to set modification time per row and recompile upon verification.
- The full s3 table of 117 rows ran on the same frozen tree; Q1, Q3, and Q4 were judged according to s4. D = 19 < 30. F4 expanded the table: reading logic for 164 additional row files was written in driver and comparator, but this full-table run remained pending. The page status was changed to "Partially run (s3 full table of 117 rows run, F4 expanded table of 164 rows pending)"; previously it was "First segment run". Before the change, all four rows in the current status table were marked as "open".
- After running the F4 expanded table of 164 rows, the comparator merged into 281 rows (`…-s4-f4-comparator.out`): Current status table Q1 changed from "7 rows each for A/B/C, D = 19" to "12 rows each for A/B, 13 rows for C, D = 37 (before padding)", Q3 changed from "11 rows relying on a single path" to "23 rows", and Q4 added the clock time for that expanded table run. Added new entries: all-pass rows from the expansion, 12 rows relying on a single path, and two sections of three rows that were red without being reached. Row-by-row driving was changed to also perform padding after compilation failure (s3 and F4 outputs were generated by pre-change driving; re-runs with switches remain unchanged regarding no padding). The 6 rows failing compilation are pending for the padding run. Page status changed to "Partially run (s3 full table of 117 rows and F4 expanded table of 164 rows run, 6 rows failing compilation pending padding)". Research-side unit tests increased from 98 to 110 lines; mutations from 52 to 67. Re-runs registered mutation outputs now point to `-r7.out`, adding one line for `driver_e171_s4_f4_comparator`.
- After completing the run for the 6 rows failing compilation (`…-s4-f4-substitutes-comparator.out`, `…-s4-f4-substitutes-rows-timing.out`): Page status changed from "Partially run (s3 full table of 117 rows and F4 expanded table of 164 rows run, 6 rows failing compilation pending padding)" to "Run complete (...readings collected...)". Denominator D = 37 remained the same before and after padding; Q1 and Q3 lists are identical character-for-character. The section "(Pending: 6 rows failing compilation padding)" was changed to "(Completed)". The originally proposed first set of replacements (row 33 → index 9 in mount.rs group, row 59 → index 8 in mounted_read.rs, row 127 → index 16 in allocator.rs, row 163 → index 37 in mount.rs, row 206 → index 4 in recovery.rs, row 243 → index 25 in transaction.rs, assuming the first replacement row compiled successfully) was removed; actual replaced rows are present in the outputs (the previous replacement for row 127 also failed compilation). Re-runs registration added one line `driver_e171_s4_f4_substitutes_comparator` (10 lines → 11 lines); product count increased from 44 to 46.
- Conclusion: Three-round verdict write-backs (`research/prompts/e171-conclusion-r1-main-verification.md`, `…-r2-main-verification.md`, `…-r3-main-verification.md`): The three-column layout was changed to current values; the pre-change three columns were as follows—Line 15 "Equivalent for this batch" (reason: "Clause not defined," historically inferred to be separated), Lines 27, 112, and 277 "Equivalent for this batch", Line 34 "Equivalent for this batch", Line 92 "True blind spot candidate (clause basis must be recognized by the main agent)", Q1 row wrote "True blind spot candidates: 4 rows (Lines 242, 269, 270, 271); Equivalent for this batch: 1 row (Line 277)"; s4's 7 rows "Three-round verdicts not re-executed". After the change, Lines 15, 23, 42, 242, 269, 270, and 271 are true blind spot candidates (Line 15 is based on D3 (Space Allocation) defined item 10⑤'s t2–t12 landing points); Lines 27, 112, 277, Line 34, and Line 92 were reclassified into other categories; a new section "Three-round verdicts subsequent three columns" was added.
- Other changes in the same write-back: In the section "Two rows that are red but not reached" and within "What it cannot answer," the clause regarding reachability was modified. Pre-change, it stated: "Whether other unreached rows have similar differences is unclear; the device did not record successfully returned values." Post-change, it states: The attacker measured 232 rows in the s4 tree copy, finding 7 rows with different return values (`name=return_summary hidden_difference=7 identical=225 no_output=6`); the definition of reachability and the denominator of 37 remain unchanged. Q2 row and the new section "Q2: O2 (Independent Parser + Checker) allows passage while O3 (Independent Specification Executor) judges bad mirrors as non-existent" were supplemented with 29 images (12 + 17). The section "What it cannot answer" was updated to add a column for RefFS and O1 (Memory Model Checksumming: Ideal Filesystem HashMap), the source of the 29 images, row-level readings belonging only to that snapshot tree, a rehearsal for Line 125 D = 38, W and 61 witness rejections holding only for s4 core, and whether user data landing point value calculations count as layout bytes left to the user's discretion. In "Affected Decisions," D13 (Verification Route) defined item 6 was changed from "Preparation" to "Support" (pre-change reason: segments were not cited by number referencing E171 (Three Oracle Collusion Test); which rows constitute true blind spots are inferences; the three-party route was not taken). Added D3 (Space Allocation) defined items 8, 10, and D23 (Journal Role and Format) defined item 15. Review text for D13 (Verification Route) defined items 1, D8 (Core Index Structure) defined item 6, and D23 (Journal Role and Format) defined item 14 was added to the clauses newly utilized in the three-round verdicts.

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1-21`（整段抄，未转述）**

````markdown
## E171 三oracle合谋测试 —— 第 1 次重跑第一段与第二段整表（修订 6 树）已跑（Q1 甲、Q2 乙、Q3 甲且分母 D = 39、Q4 基线甲与第 242、269 行乙；Q5 告警自证已过，判甲；最终主线的修订 8 那一趟待拍树，流三段长已按修订 12 重钉）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Y1 Y3 P1 P2 P3 P4 P5 P8 P9 S1 S2 S3 S4 S5 S6 S7 S8 Z1 Z2 Z3 Z4 F1 F2 F3 F4 F5 F6 F7 V1 V2 W1 K4 Q5 S15 S17 Z7 U1 U9 U13 U16 V17 V31 V33 V36 R14 R19 R20 R33 F8 RS1 RS2 RS3 RS4 RS5 RS6 RS7 RS8 RA1 RA2 RA3 RX1 S19 F10 Z10 -->

**中文** · [English](171-three-oracle-collusion-test.en.md) · [日本語](171-3オラクル共謀テスト.ja.md)

```text
本页的本地臂名与记号写小写（a1、a2、a3、a4 ……）：跑前登记、装置、产物与命令里写作对应的大写，与全局登记的编号同形；kb 正文里改成小写只为与那些编号分开，读产物时按大写对。
```

跑前登记 `e171-preregistration.md`（含第十二节修订 1–5；Q4 的另一种读法由主 agent 在装置写之前删去，只留整表 ≤ 60 秒那一种）；问题单 `three-oracle-collusion-questions.md` 的 Q1（一张坏实现表在同一批历史与崩溃状态上，有没有一行三条 oracle 全放行）、Q2（手造坏镜像喂 O2（独立解析器 + checker）、O3（独立规约执行器），有没有一张两条全放行）、Q3（拿已知被抓的坏实现逐条关掉一条 oracle，剩下的还红不红）、Q4（这道测试一次跑多久、怎么进门禁）。被测条款 D13（验证路线） 已定项 6：三个 oracle 的独立性要靠合谋测试验证，不许自称。

**现状**：坏实现表 117 行用修过的逐行驱动在同一棵冻住的树上跑完一遍（s4，两条线道、每条 2 线程），连同 Q3-核三趟与比对器；s3 那一次整表因驱动把上一行的变异带进下一行而整表作废、留档（「s3 整表作废」一节）。O3（独立规约执行器） 用 E170（O3规约执行器第一版） 第 1 次重跑冻结的执行器，开头与收尾两次冻结核对都是 0 处不同。s4 的分母 D = 19 < 30，登记 F4 触发，扩表 164 行（序号 118–281）在同一份冻住的树上跑完，比对器把两半并成 281 行：分母 D = 37 ≥ 30。两趟里编不过的 6 行（第 33、59、127、163、206、243 行）当时的驱动没按登记 5.2 补位；驱动改成编不过之后补位，补位那一趟在同一份冻住的树上跑完（六行换上的都是未触达的行），比对器把 117 + 164 + 补位并成 `…-s4-f4-substitutes-comparator.out`：分母仍 D = 37，Q1、Q3 名单与补位之前逐字同（「编不过的 6 行补位（已跑）」一节）。读数到齐之后开了三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`，第三轮是最终轮），判这批读数撑不撑 D13（验证路线） 已定项 6：真盲区候选收成第 15、23、42、242、269、270、271 行，别的全放行行与未触达行里交回值不同的行归另几类（「三轮判决之后的三分」一节）；三分与读数一样只属于 s4 那一棵冻住的树。

| 行 | 判定 | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| Q1 | 有：甲、乙各 12 行三条全放行（s4 的 7 行加扩表的第 242、269、270、271、277 行），丙 13 行（另加第 233 行）；分母 D = 37（补位之前与之后相同） | `e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out` 的 `name=q1_alpha`、`name=q1_beta`、`name=q1_gamma`（`verdict=yes_a_row_is_quiet_on_every_oracle`）、`name=q1_denominator`（`counted_D=37 … enough_to_judge_none_side=true`）；补位之前的 `…-s4-f4-comparator.out` 里 Q1 三种读法的名单行逐字同 | 照登记第六节 Q1-甲「≥ 1 判『有』」，与分母无关；D ≥ 30 只说「没有」那一边的门槛够了，这一批里「有」已成立。s4 那 7 行的读数在 F4 产物里没变（第 1–117 行的结果行逐字节相同）；扩表新出的 5 行与第 233 行（丙）、批 2 的第 265 行的读数在「F4 新出的全放行行」一节。三轮判决之后的三分在「三轮判决之后的三分」一节：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），另几类 5 行（第 27、112、277 行靠记录重建的恢复，第 34 行分配政策，第 92 行条款在字节表与里程碑验收），没有一行还记「对这一批等价」；第 233 行在丙里全放行（甲乙里 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官抓到），第 265 行批 2 是取样点不敏感。O3（独立规约执行器） 在 37 行上一行都没红（`name=q1_column_red_rows … O3（独立规约执行器）甲=0 O3（独立规约执行器）乙=0`）。补位之后分母仍 37，「有」没翻 |
| Q2 | 有（照登记字面）：甲、乙各 4 张两条全放行，全是 P8 | 同一份产物的 `name=q2_all_quiet`（`table_rows=107 all_quiet_rows=4 verdict=yes_a_bad_image_passes_both`）、`name=f3_p_rows_judged_in_by_o3`、`name=q2_negative_bases`；O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的数在 `name=q2_family` 的 P 族与盲测 B 族两行（`o3_only_strict=12`、`o3_only_strict=17`，整行抄在「Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像」一节） | 4 张都是 E170（O3规约执行器第一版） 设计员推的「不在」而 O3（独立规约执行器） 判「在」的 P8 行，F3 触发；主 agent 照 F3 判成期望推错、这四张不是坏镜像（判决记在 D13（验证路线） 已定项 6 的射程）。Q2 的数与第二段 `…-s2-comparator.out` 同出一批输入，s4 比对器重算一遍；F4 产物里 Q2、F3 与对照的结果行与 s4 那一份逐字节相同。几何两点（`q2_geometry_unstable_kinds=2`）：翻面的是流一 P2、P5 两种补丁的 O2（独立解析器 + checker） 一列（底态 F 红、Mk 不红），O3（独立规约执行器） 两个底态都判「不在」，所以这两种补丁的「两条全放行」两点都不成立、不翻；全放行的 P8 两个底态都是 `same_on_both_points`。另有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像 29 张（P 族 12 张加盲测 B 族 17 张，12 + 17）：写「O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在」，不写「O3（独立规约执行器） 独有」，Q2 只比 O2（独立解析器 + checker） 与 O3（独立规约执行器） 一对，O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 没在坏镜像上跑；这 29 张是结构自洽的伪造根、伪造续接镜像，补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，细节在「它答不了的」 |
| Q3 | 有一行只靠一条：甲、乙各 23 行（只靠 O1（内存模型对拍（HashMap 理想文件系统）） 的 32、45、108、115、172、233、237、249、250、281；只靠 O2（独立解析器 + checker） 的 8、65、86、87、90、99、102、135、234、238、245、258、265） | 补位之后的 `…-s4-f4-substitutes-comparator.out` 的 `name=q3_alpha`、`name=q3_beta`（补位之前的 `…-s4-f4-comparator.out` 同名行逐字同）；Q3-核 9 行 `name=q3_core_row` 全是 `differences_outside_the_closed_column=0`（Q3-核取第一刀的行，扩表没有第一刀） | Q3-核 9 趟 0 处不同，读法 9 成立、F5 不触发。第 102、233、265 行批 1 与批 2 红的列不同，都是取样点不敏感（批 2 没取到红的那几段或那一类态），记「不稳定」。s4 那 11 行另外两条为什么看不到在「Q3 只靠一条的 11 行」一节，扩表新出的 12 行在「F4 新出的只靠一条的 12 行」一节 |
| Q4 | 只能进全量：117 行整表两条线道 5 569.6 秒，扩表 164 行另一趟 7 607.9 秒，门槛 60 秒 | `e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out` 的两行 `name=table source=table-driver.out`（`rows=117 table_wall_ns=5569579167726`、`rows=164 table_wall_ns=7607936234181`） | 281 行分两趟跑，两趟挂钟相加约 13 177.5 秒（两个数相加，不是一趟量的）；各行之和 11 114.4 + 15 191.9 秒是单线道的估计（没单独跑单线道）。编不过的 6 行补位另一趟 1 277.9 秒（`…-s4-f4-substitutes-rows-timing.out` 的 `name=table source=substitution-driver.out … rows=6 table_wall_ns=1277937171946`），三趟相加约 14 455.5 秒，也是三个数相加。Q3-核每趟三行约 114–130 秒 |

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:22-39`（整段抄，未转述）**

```markdown
### 装置、研究一侧与变异表

研究一侧变异表在各修订上的复跑（`replay.sh` 逐字节比对的三行）：修订 8 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-10-07.out`（`E7RESULT name=verdict mutations=26 caught=26 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`）、修订 10 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-s2-revision-10-tools-research-mutations.out`（`mutations=29 caught=29`）、修订 11 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-revision-11-tools-research-mutations.out`（`mutations=15 caught=15`），三份都 `invalid=0 survived=0 timed_out=0`。

| 件 | 位置 | 说明 |
|---|---|---|
| 入库装置 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs` | 模式 `baseline`、`row`、`known-bad-images`，开关 `--positive-control`、`--disable-oracle`；只读、只驱动、只观测，不改 core、checker、harness、format、replay-prefix；内联单测 31 条（`grep -c '#\[test\]'`） |
| 研究一侧 | `scripts/experiment-apparatus/e171_three_oracle_collusion/`（Python，目录里没有 `main.rs`，cargo 不认它当 bin） | 挑法脚本（登记 5.2，sha256 `4032a0e0…`，原样抄）、117 行的表 `bad-implementations-117.tsv`（登记 5.9 照名字抄出）、F4 扩表 `bad-implementations-expansion-164.tsv` 与造它的 `expansion_table.py`、`row_driver.py`、`comparator.py` 与它的几个模块、冻结执行器的接线 `frozen_o3.py`、判坏镜像的 `o3_bad_image_judge.py`、四个单测文件、变异驱动 |
| 变异 | 研究一侧 `scripts/mutation/tables/e171_three_oracle_collusion.tsv` 67 行；装置一侧 `crates/mutations.tsv` 末尾 15 行（名字以「E171（三oracle合谋测试） 装置」起头，`grep -c '^[E]171 装置' crates/mutations.tsv`） | 研究一侧经本目录的 `mutation_driver.py` 跑（登记修订 1 第 1 行）；装置一侧是 checker 档，三个数归提交时的门禁 checker-tier-crates-mutation-replay |
| 复跑 | `scripts/experiment-apparatus/replay.sh` 里 E171（三oracle合谋测试） 二十行登记（`grep -c '^[E]171.@driver' scripts/experiment-apparatus/replay.sh`）：这一页原来的十一行，加第 1 次重跑九行（第一段·修订 6 那一趟三行、第二段两行、修订 8、10、11 的研究一侧变异三行、修订 12 一行） | 原来十一行见「复跑」一节；第一段那三行见「比对器、复跑与入库的跑法（修订 6 那一趟）」一节，第二段那两行见「第 1 次重跑第二段·修订 6 那一趟」一节，修订 12 那一行见「第 1 次重跑·修订 12：流三段长重钉（2026-10-13）」一节，修订 8、10、11 那三行页里没有节 |
| 产物 | `research/results/` 下 `e171-three-oracle-collusion-2026-10-03-` 起头的 46 份（`ls research/results \| grep -c '^e171-three-oracle-collusion-2026-10-03-'`） | 逐份在「复跑」一节的产物表里 |

- 流的驱动：两条流（新池新建文件、固定脚本到 E 再正常卸载）照抄 E170（O3规约执行器第一版） 导出装置的两条流驱动、取样三批与中性格式的写出（不 `#[path]` 引它），改三处：驱动里的 API 调用不 `.expect`，结局（成功、错误成员、panic）记进 `calls.tsv` 的 `ok` / `err` 列与 `call-outcomes.tsv`；跟着每次调用推理想模型，记 O1（内存模型对拍（HashMap 理想文件系统））-对拍（模型的答案与实现的结局比，`judge_and_advance`）；每个取样态算 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官（`crash_recovery_disagreement`）与 O2（独立解析器 + checker） 的 `check_pool_image`，写 `oracle-verdicts.tsv`。`baseline` 回比本地常量与 crates 的现算值（s1，42 条 `name=s1` 行都是 `holds=true`；`grep -c 'name=s1 ' …-baseline-run1.out` 与 `grep -c 'name=s1 .*holds=false'` 现数）；`row` 不回比，只取批 1 与批 2。
- S8 的三处调用在装置里的行号（`grep -n` 现取）：`newest_persisted_root` 第 1883 行、`observed_read_back_after_a_crash` 第 1885 行、`crash_recovery_disagreement` 第 1907 行，都在 `crash_judge_of_the_state`（第 1875 行）与它转调的 `crash_judge_verdict` 里；O1（内存模型对拍（HashMap 理想文件系统））-对拍的 `judge_and_advance` 在第 544 行。
- 已知坏镜像族：`crates/singlefs-harness/tests/checker_known_bad_images.rs`（sha256 `e8543908…`）的导入、常量、辅助函数与 `known_bad_images` 照抄进装置里的一个模块（第 7–1294 行，加上 `known_bad_images` 用到而散在第 1295 行之后的 12 个常量与辅助函数）；`known-bad-images` 模式核条数 41、条目名与原文件逐条相同、干净镜像摘要等于 `build_pool` 的（S4）。
- 逐行驱动：一次跑先拍一张仓快照（`Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`，237 个文件，总摘要写在驱动的标准输出第一行），每条线道从快照同步出自己的树与编译目录，改一处、`cargo build --release`、装置 `row` 经 `run-with-memory-cap.sh`（限时 1800 秒）、还原；表里的行照名字取、核锚点，不行按补位规则换同组下一行（至多三次）。s4 与 F4 两趟时驱动只在编之前的失败上补位；第八段起编不过之后也补位（换上的行编不过再换，编之前的失败与编不过合起来至多三次），组里不含 E171（三oracle合谋测试） 装置自己的变异行，`row-result.tsv` 记 `substituted_from` 与每一个试过而没用上的 `failed_attempt_<序>`；开关 `--keep-compile-failures-as-they-are` 只给 `replay.sh` 复跑 s4、F4 两份产物用，补位那一份产物不带它。


```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:40-87`（整段抄，未转述）**

````markdown
### s4 整表的结果（整行抄自产物）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out`（2119 行，末行 `E7RESULT name=done emitted=2119`）。冻结核对、阳性对照四条 oracle 各两趟（基线与空变异）、Q1 的分母与三种读法、7 行全放行（甲）、Q3 两种读法、Q3-核 9 趟与判决行：

```
E7RESULT name=config segment=s4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O3 run=empty_mutation red_units=1 red=true
E7RESULT name=q1_denominator rows_run=117 counted_D=19 untouched=93 only_in_the_ignored_pass=0 compile_failed=2 anchor_not_unique=0 not_found=0 self_crash=3 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=false
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=5 rows_run=117 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=19 O1=5 O1-对拍=5 O1-崩溃判官=1 O2=8 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=0 row_numbers=none
E7RESULT name=q1_alpha counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_quiet_row reading=q1_alpha row=15 row_name=实审_A2c_第_15_条：用户数据落点不从这张图的单元区起点起扫 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=22/22 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=23 row_name=并行线三_C116：分裂出来的右半容器号不是触发分裂那条新记录的_inode_号 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=26/26 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=27 row_name=实二十：记录头读者把末条标志位_0_读反（带标志的读成不是末条） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=34 row_name=步_5：抬_F_生效之后不放开扣住的槽 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/498 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=42 row_name=实二（回退改形态：SysPre_与_B1，D16_已定项_1_/_7、D22_已定项_7_/_9、I-7.9_/_I-7.12、C556_/_C557）：卸载把_F_抬到现行那一版带的_F（原地不动）而不是它的_txg o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/537 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=92 row_name=步_1_验收第_4_条：inode_记录的写入时间留成_A_的（覆盖写照抄上一版的写入时间） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/61 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=112 row_name=步_3：前缀跨实例边界（把别的实例的记录也接上） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_beta counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=19 all_quiet_rows=7 row_numbers=15,23,27,34,42,92,112 batch_2_all_quiet_rows=8 rows_that_differ_from_batch_1=102 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=19 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_beta caught_rows=12 only_O1=32,45,108,115 only_O2=8,65,86,87,90,99,102 rows_whose_red_columns_differ_between_batch_1_and_2=102
E7RESULT name=q3_core_row closing=O1 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O1 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=O2 row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=false where=none
E7RESULT name=q3_core_row closing=no_executor row=108 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=32 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=q3_core_row closing=no_executor row=8 differences_outside_the_closed_column=0 o3_state_lines_differ=not_applicable_no_executor_in_the_closed_run where=none
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=0 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=7 q1_beta_all_quiet_rows=7 q1_gamma_all_quiet_rows=7 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=11 q3_beta_rows_held_by_a_single_oracle=11 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
```

分族、分类的计数（一条命令从产物里数，原样输出）：`grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` 输出 `2 compile_failed`、`19 counted`、`3 self_crash`、`93 untouched`（四行）。不进分母的 5 行：第 6、7、85 行实现自崩，第 33、59 行编不过。登记 5.2 写「一行编不过……换同一文件组内下标 j + 1」，s3、s4 那一版逐行驱动只在编之前的失败（找不到、名字不唯一、锚点不唯一）上补位，第 33、59 行没有补位；驱动在第八段改成编不过之后也补位，这两行连同扩表编不过的 4 行的补位那一趟已跑（「编不过的 6 行补位（已跑）」一节）。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:88-103`（整段抄，未转述）**

```markdown
### 三条全放行的 7 行

读数取 s4 的导出目录（`scripts/experiment-apparatus/target/e171-dumps/rows-s4/row-<序号>/dump/`，与 `baseline-run1` 逐条 op、逐态比；草稿脚本 `quiet_rows_s4_fua.py`、`returned_check.py` 只读导出目录，不入库）。三分照 `.claude/rules/mutation-sampling.md` 的口径落到这一批上：**对这一批等价**＝两条流、批 1 批 2 的每个态连同实现在态上的恢复都在条款允许的集合里（写依据；会分开的历史写在旁边，没跑）；**取样点不敏感**＝两条流里有条款不许的态，取样规则没取到；**真盲区**＝取到的态里有条款不许的，三条都放行。「镜像不同」数的是 `states.tsv` 里镜像摘要与基线那一态不同的态；「择到新版」数的是其中实现看 journal 那一遍恢复择到的根 txg 不小于「第一处不同之后写出的第一条根」的态。三条各自看什么：O1（内存模型对拍（HashMap 理想文件系统））-对拍比每次调用之后实现交回的 `ObservedRoot`（`crates/singlefs-harness/src/model.rs` 里它的定义：根的 key、journal 计数器、根带的 F、文件内容、实例表、每块盘的分配记录），O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官比实现恢复择到的根与读回内容；O2（独立解析器 + checker） 判单个镜像的结构；O3（独立规约执行器） 的允许集合 A(c) 由这一行自己写出的根槽写构成（`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/version_table.py` 文件头说明），一版只投影根的六样（`root_ring.py` 里 `VersionProjection` 的说明：实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F），判的是它自己从镜像恢复出的那一版在不在 A(c) 里（`executor_core.py` 的 `judge_state` 里 `landed_is_allowed =` 那一行）。「三条全放行的 7 行」那张表末列的三分是三轮结论判决之后的现值（旧三分列的改动写在「历史版本」一节），各行归属的全貌在「三轮判决之后的三分」一节。

| 行（变异） | 和基线不同在哪 | 落在哪种状态里 | O1（内存模型对拍（HashMap 理想文件系统）） / O2（独立解析器 + checker） / O3（独立规约执行器） 看不看得到 | 三分 |
|---|---|---|---|---|
| 15（用户数据落点不从这张图的单元区起点起扫） | 两条流第 22 条 op 起：用户数据单元两块盘各 32 KiB 落点 822149120 → 823132160，内容逐字节相同；指向它的 16 KiB 节点里指针与校验和随之变；同一次发布的十一个提交内生块（t2–t12）也整体后移 64 槽，开放段 [50240, 50304) 整段后移到 [50304, 50368)：设备 0 上 12 条写换了落点（第 22 条 op 50180 → 50240，第 24 条 50240 → 50304，第 26 条 50242 → 50306，……，第 44 条 50252 → 50316；两条流相同，`row15-offsets.out` 里两行 `name=row15_first_publish`）；流二第 4 次调用起返回点后移 4 条 op（`mount_writable` 92–179 → 92–183），共 555 条 op 不同；调用结局同；看 journal 那一遍恢复逐态同 | 镜像不同：流一批 1 20 态、批 2 7 态，流二批 1 364 态、批 2 119 态；择到新版：流一批 1 12 态，流二批 1 356 态、批 2 112 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot` 不带落点，读回同，看不到；O2（独立解析器 + checker）：落点在单元区里、记账自洽，判绿；O3（独立规约执行器）：投影不带落点，「在」 | **真盲区候选**：依据是 D3（空间分配） 已定项 10 ⑤ 的 t2–t12 落点：「⑤ 在新池新建文件上的落点（开放段 [50240, 50304)，bump 游标从 50240 起）」（`.claude/kb/decisions/03-空间分配.md`「已定项 10：聚簇段的定义、用户数据落点的四格、提交内生块的 bump 次序」的定案）、射程「改新池新建文件的字节：是」「十一个提交内生块（默认几何）的落点整体重排」（第 226 行）、字节表「t2 到 t12 依次 50240、50242–50243、50244、…」（`.claude/kb/layout/01-first-txn.md`「五、空间记账与分配」里「提交内生块」那一行）；变异下用户数据落到 50240、十一个提交内生块整体挪 64 槽，不在 ⑤ 钉的落点上，三条都放行。另一条依据（用户 2026-10-03 定案之后才有）：D3（空间分配） 已定项 8 射程「例外是**从 mkfs 水位第一次建树的那一版**」「新池新建文件是这一类」、t1@50180「登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 字节表里，算格式级，偏离它写出的这一类镜像不合法」（`.claude/kb/decisions/03-空间分配.md`「已定项 8：用户数据块不进聚簇段，落点是已选设备内起点最小的空槽」的射程），变异把 t1 从 50180 挪到 50240，本身就不合法；三轮判决定去留时还没有这一条，只靠 ⑤。会分开的历史（推的）：单元区刚好够用的池，跳过开头一段之后发布在模型说该成的地方报空间不够，O1（内存模型对拍（HashMap 理想文件系统））-对拍会红 |
| 23（分裂出来的右半容器号不是触发分裂那条新记录的 inode 号） | 两条流第 26 条 op 起：第一个文件那次发布写的 inode 叶（码 3，两块盘各 32 KiB）偏移 53 一字节 0x01 → 0x00（容器号 1 → 0），头校验和随之变；同一次发布第 28 条 op（16 KiB 节点）偏移 52、60、131、149 等处 0x01 → 0x00；之后重写 inode 树的每次发布都带着（流二 155 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流一批 1 20 态、批 2 7 态，流二批 1 364 态、批 2 119 态；择到第一个文件那一版（txg 3）及之后：流一批 1 12 态，流二批 1 356 态、批 2 112 态 | O1（内存模型对拍（HashMap 理想文件系统））：不读盘，`ObservedRoot` 里没有容器号，读回同；O2（独立解析器 + checker）：I-9.4（容器号不超最小 key）只要「inode 号 ≥ 容器号、沿叶序严格递增」，一片叶时 1 ≥ 0 成立，判绿；O3（独立规约执行器）：投影不进 inode 树，「在」 | **真盲区候选**：D8（核心索引结构） 已定项 6 定案写「**新池新建文件写**：一个码 2 根节点（头带 key 区间 [1, 1]；一条条目：key 1、类型 2、出生树、容器号 1、出生代 3、指针）；一个码 3 叶（类型 2、容器号 1、出生代 3、记录数 1、记录宽 140）」（`.claude/kb/decisions/08-核心索引结构.md`「已定项 6：inode 树的叶是码 3 打包记录单元，记录定长 140，已发布的 (inode 号, 对象出生代) 不复用」的定案），射程写「每个容器号都是某条记录首次插入时的 inode 号」（第 205 行）；容器号 0 不是任何一条记录的 inode 号。择到新版的那些态不在条款写得出的镜像里，三条都放行。会让 O2（独立解析器 + checker） 红的取样点（推的，没跑）：一片叶装满之后的分裂，右半容器号 0 小于左半，I-9.4（容器号不超最小 key） 的「沿叶序严格递增」判红 |
| 27（记录头读者把末条标志位 0 读反） | 写流与调用结局全同；只在看 journal 那一遍恢复上不同：流一批 1 6 态、流二批 1 51 态（批 2 0 态）里，基线施加 1 条记录、择到新的一版，变异把带末条标志的记录读成「后面还有」、施加 0 条、停在前一版；读回内容不同的 3 / 30 态（例：流一第 41 态基线 `1,3 file_read e8ca4bf8…`，变异 `1,2 no_file`） | 发布写到一半的态：被丢掉的那一版的调用没返回、它的根槽没落（`returned_check.py` 57 态全是 `call_returned=False its_root_slot_landed=False`） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：前一版是模型提交过的、不比盘上已持久的最新根槽旧、读回与那一版相同，判绿是对的；O1（内存模型对拍（HashMap 理想文件系统））-对拍：调用不经恢复；O2（独立解析器 + checker）、O3（独立规约执行器）：镜像与基线逐字节相同，各自不看实现的恢复 | **另一类：三条不判实现恢复里靠记录重建的那一段**。态本身合法（调用没返回，新旧两版都在允许集合里），错在实现的恢复少施加一条记录，违反 D13（验证路线） 已定项 1 的 M4「`Recover` 返回的根恰是 j\*」与 D23（journal 的角色与格式） 已定项 15「由记录重建那次发布的根」、已定项 14「恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录」；O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官的判据（模型提交过的某一版、不比盘上已持久的最新根旧）比 M4 弱，O2（独立解析器 + checker） 判恢复之前的镜像，O3（独立规约执行器） 不跑实现的恢复；M4 的落点是 E165（恢复重放有界穷举与变异杀伤界） 与 Verus，不是这三条。会分开的取样点（推的，没跑）：一次已经返回、提交点只在 journal 记录里（根槽还没写）的发布之后的态，变异会丢掉一次已返回的发布；两条流的发布都在返回之前写根槽，这样的态不存在。那时 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官的「不许比盘上已持久的最新根槽旧」也挡不住（根槽没写），会不会三条都放行没量过 |
| 34（抬 F 生效之后不放开扣住的槽） | 流一无不同；流二第 498 条 op 起（抬 F 之后那次 `publish`）：用户数据单元落点 822083584 → 822411264，内容相同；指向它的节点与分配记录随之变（51 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流二批 1 68 态、批 2 21 态；择到 txg 17 及之后的根：批 1 60 态、批 2 14 态 | 同第 15 行：O1（内存模型对拍（HashMap 理想文件系统）） 不比落点，O2（独立解析器 + checker） 判绿（记账自洽），O3（独立规约执行器）「在」 | **另一类：分配政策，镜像合法**：抬 F 之后那次发布换了一个空闲槽、没复用回收的槽，每个态的分配记录与记账自洽；D3（空间分配） 已定项 8 射程「其余的写（覆盖写、之后的发布、重挂之后）上违反它的实现写出的镜像合法，拦它只能靠运行时读数」（`.claude/kb/decisions/03-空间分配.md`「已定项 8：用户数据块不进聚簇段，落点是已选设备内起点最小的空槽」的射程；新池新建文件那一格例外，这一行的改动在流二抬 F 之后那次发布上，不在其中），三条按设计都不判分配政策。会分开的历史（推的）：抬 F 之后空间只够复用扣住那几槽的池，发布报空间不够而模型说该成，O1（内存模型对拍（HashMap 理想文件系统））-对拍会红；这一行在 `crates/mutations.tsv` 点名的测试 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 直接断言复用 |
| 42（卸载把 F 抬到现行那一版带的 F 而不是它的 txg） | 流一无不同；流二第 537 条 op 起（卸载那次调用的第一条写：两块盘的系统配置槽，偏移 481 一字节 0x11 → 0x0b，即 F 17 → 11，连同校验和），之后卸载推的空发布写的根与节点里同样是 11（24 条 op 不同）；卸载照常 `ok`；恢复逐态同 | 镜像不同：流二批 1 48 态、批 2 14 态，全持久态在内；择到卸载写出的根（txg ≥ 18）：批 1 32 态、批 2 7 态 | O1（内存模型对拍（HashMap 理想文件系统））：卸载没有模型的答（登记读法 6），这次调用不对拍、写落在卸载里的态崩溃判官记不适用，看不到（若有卸载的答，`ObservedRoot` 带根的 F，会比出 11 对 17，推的）；O2（独立解析器 + checker）：系统配置的 F 与根带的 F 都是 11，I-7.12（系统配置 F 不低于同盘根上的 F）成立，单个镜像不知道该是 17；O3（独立规约执行器）：投影带 F，但 A(c) 就是这一行自己写的 F = 11 的根，「在」 | **真盲区候选**：D16（发布语义） 已定项 1 定案的字段表「正常卸载」一行写「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md`），射程「正常卸载的代价」那一条写「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」。变异下卸载返回之后的全持久态 F = 11，回退候选集里留着 txg 11–16 的根，不在条款允许的镜像里，三条都放行。O1（内存模型对拍（HashMap 理想文件系统）） 那一半是登记读法 6 写明的已知缺口（模型没有卸载的答） |
| 92（inode 记录的写入时间留成 A 的） | 流一无不同；流二第 61 条 op 起（第二次 `publish`，覆盖写那次的 inode 叶，两块盘各 32 KiB）：叶内偏移 200、208、216 三个字节 0x3c → 0x00（第 0 条记录起点 136，+64 / +72 / +80 是 atime、mtime、ctime 秒：基线 60，变异留 A 的 0），头里偏移 10–13（头校验和）与 89–92 另有不同；之后重写这片叶的发布都带着（139 条 op 不同）；调用结局同；恢复逐态同 | 镜像不同：流二批 1 344 态、批 2 112 态；择到覆盖写那一版（txg 4）及之后：批 1 336 态、批 2 105 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedFile` 只有内容，`model.rs` 与 `model_comparison.rs` 里 `write_time` 零命中，D13（验证路线） 已定项 6 表 O1（内存模型对拍（HashMap 理想文件系统）） 行判「内容、大小、目录结构」，看不到；O2（独立解析器 + checker）：单个镜像不知道这次发布的写入时间参数，判绿；O3（独立规约执行器）：投影不进 inode 树，「在」 | **另一类：条款在字节表与里程碑验收，决策正文欠一句**：D8（核心索引结构） 已定项 6 的字段表只定了偏移 64「atime / mtime / ctime 秒」的位置；「写入时间是发布参数」写在 `.claude/kb/milestone/01-first-txn.md`「步 3　inode 树与其余几棵树」的现状一段与 `02-second-txn.md`「步 1　覆盖写：同一个文件、同一个池、第二次发布（发布 B）」里 inode 记录那一条，字节表写「写入时刻（mkfs 参数，可复现）」（`.claude/kb/layout/01-first-txn.md`「四、文件元数据（inode）」里「inode 记录（偏移 64）」那一行），验收写「inode 记录的写入时间留成 A 的 ⇒ 判红」（`02-second-txn.md` 同一步里「变异：extent 叶记录的指针忘了换」那一条），决策正文里没找到取值的一句。择到新版的态（写入时间 0、这次发布的参数是 60）不合验收，三条按设计看不见（没有 API 交回写入时间），不当独立性失败 |
| 112（前缀跨实例边界） | 写流与调用结局全同；看 journal 那一遍恢复在两条流各 3 态（批 1 第 9–11 态，段 1：暖机的第一条写——新实例 1 的 journal 记录——落了、它的根槽没落）不同：基线择 `0,0`、施加 0 条，变异把实例 1 的记录接到实例 0 的根上、施加 1 条、得到 `1,1`；读回都没有文件 | 暖机写到一半的态（`returned_check.py` 6 态全是 `call_returned=False`） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：`1,1` 是模型提交过的一版、不比已持久的最新根槽旧、读回同，判绿是对的；O2（独立解析器 + checker）、O3（独立规约执行器）：镜像与基线逐字节相同 | **另一类：三条不判实现恢复里靠记录重建的那一段（多施加）**：暖机没返回，`0,0` 与 `1,1` 都在允许集合里，错在实现的恢复把别的实例的记录也接上，违反 D23（journal 的角色与格式） 已定项 14 注 1「前缀规则不跨实例边界」（`.claude/kb/decisions/23-journal的角色与格式.md`「已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换」），依据只有这一条注；这一行的变异已在门禁 checker-tier-formal-verification 的红例钉表里（`.claude/gate.d/e165-pinned-red-examples.tsv` 里第 1 列是 `371a71190dce62e0…` 的红例），由 E165（恢复重放有界穷举与变异杀伤界） 的有界穷举判。会分开的取样点（推的，没跑）：环里留着别的实例、不在所选根那条时间线上的记录（崩溃恢复抛弃了根之后的新实例），接进来之后恢复落到模型没提交过的一版，O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官会红；两条流里没有这种态 |

第 102 行（抬 F 先写系统配置那一步不带新 F）是批 2 多出来的那一行全放行（`name=q1_alpha … rows_that_differ_from_batch_1=102`）：流二第 440 条 op（抬 F 先写的系统配置槽，偏移 481 一字节 0x0b → 0x00）与第 537 条 op（卸载先写的系统配置槽，0x11 → 0x0b）不同，4 条 op；O2（独立解析器 + checker） 在批 1 判红 12 态，都是 I-7.12（系统配置 F 不低于同盘根上的 F），在第 60、73 段各 6 态（`name=trajectory row=102 stream=fixed_script_through_e_then_unmount oracle=O2（独立解析器 + checker） total_red_states=12 peak=6 positive_segments=2 of_segments=79 …`）。批 2 在流二只取 17 段（7、11、16、20、24、28、32、36、40、44、48、52、57、61、65、70、74），第 60、73 段一个态都没取（`awk -F'\t' '$3==60\|\|$3==73' states.tsv` 只出批 1 的 8 + 8 态）：**取样点不敏感**，会分开的取样点就是第 60、73 段的任一态。这一格两批翻面，记「不稳定」；Q1、Q3 的结论字在批 2 上不翻（批 2 全放行 8 行仍 ≥ 1；只靠一条的 10 行仍 ≥ 1）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:104-123`（整段抄，未转述）**

```markdown
### Q3 只靠一条的 11 行

只写够不到的对象与没判的字段，不判结论。读数取 s4 的导出目录（草稿脚本 `caught_rows_s4.py`，不入库）与产物的 `name=trajectory` 行；段为正的数是「红态的段数 / 这条流的段数」。

| 行（变异） | 那一条红在哪 | 另外两条为什么看不到 |
|---|---|---|
| 32（可写挂载不写实例表的行，实例表照抄） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 判 `disagrees:这一版的实例表`（模型要有上一个实例那一行，实现交回空表），模型停在这次调用，之后流二 324 态（批 1）崩溃判官记不适用 | O2（独立解析器 + checker）：这一行 0 态红（批 1 流二 397 态都绿），实现了的不变量里没有一条拿根的实例代号去对实例表的行（由读数推的，没逐条查 `crates/singlefs-checker/invariants/`）；O3（独立规约执行器）：投影里没有实例表，A(c) 是这一行自己写的根，「在」 |
| 45（挂载那一刻就读不出的槽也重读、仍坏就拒） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 12 次调用 `raise_floor` 交回 `err:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，模型说该成 | O2（独立解析器 + checker）、O3（独立规约执行器）：被拒之前的写流与基线逐条相同（流二停在第 440 条 op，0 条 op 不同），取到的镜像都是基线那一批的前缀；被拒的调用不留镜像，单镜像的判官没有对象可判 |
| 108（码 2 读者补齐字节从最后一个载荷字节算起） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：两条流第 2 次调用 `publish` 判 `disagrees:这一版的文件内容`（胶水解不开实现交回的单元：「补齐字节非零」）；流二第 4 次调用 `mount_writable` 交回 `err:Recovery` | 错在实现的读者一侧，写出的字节与基线相同（流一 0 条 op 不同，流二停在第 92 条 op 之前也 0 条不同）；O2（独立解析器 + checker） 用自己的解析器读裸字节、不调 core 的读者，O3（独立规约执行器） 用 Python 自己的读者，两边看到的镜像与基线逐字节相同。崩溃点重放那份手写版本表的 oracle 与记录核对器红（`layer0_or_record_checker_red=true`，不是三条之一） |
| 115（取前缀核锚点筛选不看 txg） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：批 1 流一 32 态、流二 52 态「走读失败：没择到根」（实现恢复报 `RootPublishCarriesMoreThanOneLastRecordFlag…`），段为正 8/12、12/16；O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 交回 `err:Recovery`，模型说该成 | 写出的字节与基线相同（流一 0 条 op 不同）；O2（独立解析器 + checker） 判镜像结构、O3（独立规约执行器） 用它自己的前缀规则从镜像恢复，都不看实现的恢复，看到的镜像与基线逐字节相同。崩溃点重放那份 oracle 与记录核对器红 |
| 8（空闲不随分配减） | O2（独立解析器 + checker）：I-5.2（空闲统计对得上）批 1 流一 9 态、流二 353 态，段为正 2/12、69/79；批 2 流二 112 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot` 里没有记账行，调用都成、内容都对；O3（独立规约执行器）：投影不进记账树，「在」 |
| 65（blocks 向下取整） | O2（独立解析器 + checker）：I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）批 1 流一 9 态、流二 353 态 | O1（内存模型对拍（HashMap 理想文件系统））：`model.rs` 与 `model_comparison.rs` 里 `blocks` 零命中，读回内容同；O3（独立规约执行器）：投影不进 inode 树 |
| 86（写系统配置槽时 F 那 8 字节写成 0） | O2（独立解析器 + checker）：I-7.12（系统配置 F 不低于同盘根上的 F）批 1 流二 97 态，段为正 19/79；批 2 28 态 | O1（内存模型对拍（HashMap 理想文件系统））：比的是根带的 F（`ObservedRoot` 的 `rollback_floor`），不比系统配置槽里的 F；O3（独立规约执行器）：投影的 F 取自根，系统配置槽只拿来择几何，A(c) 是这一行自己写的根，「在」 |
| 87（core 那份在飞上限把 F 多算 1） | O2（独立解析器 + checker）：I-8.1（环几何够大）批 1 流一 51 态、流二 385 态，I-7.13（系统配置池级字段在读者收的范围里）流一 2 态、流二 12 态；段为正 12/12、79/79 | O1（内存模型对拍（HashMap 理想文件系统））：不比系统配置槽的字段；O3（独立规约执行器）：同第 86 行，投影里没有在飞上限 |
| 90（新池新建文件的改动计数写回 1） | O2（独立解析器 + checker）：I-9.26（inode 改动计数落在出生代与容器诞生代号之间）批 1 流一 9 态、流二 240 态 | O1（内存模型对拍（HashMap 理想文件系统））：不比 inode 记录的改动计数；O3（独立规约执行器）：投影不进 inode 树 |
| 99（defer 行写 0） | O2（独立解析器 + checker）：I-3.11（已分配减 defer 等于最新根走读）批 1 流一 9 态、流二 353 态 | O1（内存模型对拍（HashMap 理想文件系统））：不比记账行；O3（独立规约执行器）：投影不进记账树 |
| 102（抬 F 先写系统配置那一步不带新 F） | O2（独立解析器 + checker）：I-7.12（系统配置 F 不低于同盘根上的 F）批 1 流二 12 态，第 60、73 段各 6 态；批 2 不红（取样点不敏感，见「三条全放行的 7 行」一节末） | O1（内存模型对拍（HashMap 理想文件系统））：`raise_floor` 的答比的是写出的根（根带的 F 对），不比系统配置槽；O3（独立规约执行器）：同第 86 行 |

O3（独立规约执行器） 在 s4 计入分母的 19 行上一行都没红（`name=q1_column_red_rows … O3（独立规约执行器）甲=0 O3（独立规约执行器）乙=0`），而它在每行两条流的 450 个批 1 态上都给出了严格判定（`o3_judged_states_strict=450`，`name=q1_o3_coverage … rows_with_no_strict_judgement=0`）。由执行器的写法读出来的一条（读码，没另跑）：A(c) 只由这一行自己写出的根槽写构成，投影只有根的六样，所以 O3（独立规约执行器） 只在「实现自己的恢复规则与条款不同、或一次发布在崩溃之后半施加」这一类上红；实现把某个字段一致地写错，O3（独立规约执行器） 照样判「在」。O3（独立规约执行器） 在坏镜像那一族（Q2）上红得多：P 行 26 张红 22、盲测 40 张全红、已知坏镜像 41 张严格红 28（`name=q2_family` 四行）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:124-127`（整段抄，未转述）**

```markdown
### s3 整表作废

s3 那一次整表（输出根 `scripts/experiment-apparatus/target/e171-dumps/rows-s3/`）两条线道各自跨行复用编译目录；还原改过的文件用 `copy2`、同步用 `rsync -a`，都把修改时间带回快照那一刻，cargo 按修改时间判新旧，线道 1 跑过第 112 行（replay-prefix 的变异）之后、跑第 114 行之前的 45 行，编出来的装置都带着第 112 行的变异。登记修订 5 照实记了这 45 行、cargo 1.98.0 上的机理实测与驱动的改法；s3 整表、s3 的 Q3-核各趟与 s3 比对器产物不再当 Q1、Q3、Q4 的读数。留档不删：`research/results/` 下的 `e171-three-oracle-collusion-2026-10-03-s3-comparator.out`、`…-s3-rows-timing.out`、`…-s3-clean-check-comparator.out`（第五段对第 2、27、42、65 行的干净复核），与 `scripts/experiment-apparatus/target/e171-dumps/rows-s3*`、`rows-s5-clean-check`。s3 读到的「Q1 全放行 40 行、分母 52」里 38 行是被带着跑的；s4 在修过的驱动上重跑全部 117 行，判决行 `rows_compiled_without_the_rebuild_check=0`。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:128-136`（整段抄，未转述）**

```markdown
### F4 扩表

登记第十节 F4：「触达行太少，Q1 的『没有』不够判：进第三段扩表（5.2），扩完仍不够记『不够判』，不记『没有』」，触发观测「Q1-分母 D < 30」；s4 的 D = 19。扩表照登记 5.2「扩表」：步长 4 的挑法比步长 8 多出来的 164 行（与步长 8 重叠的 56 行不重跑），按挑法脚本的输出顺序接在 117 后面编号 118–281、刀号 3。

- 行文件 `scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-expansion-164.tsv`（sha256 `f1bd082e…`），由同一目录的 `expansion_table.py` 起挑法脚本两次（步长 8、步长 4）造出；表取 `crates/mutations.tsv` 去掉 E171（三oracle合谋测试） 装置自己的变异行（与 a6、a7 同一个去法）。在 s4 冻住的树的那一份与 2026-10-03 仓里的那一份上造出来逐字节相同；按文件：admission 5、allocation_record_tree 2、allocator 12、block_device 2、code_two_tree 3、extent_tree 2、inode_tree 1、instance_table 2、journal 2、make_filesystem 6、mount 44、mounted_read 3、mounted_session 2、records 2、recovery 28、root_record 1、system_configuration 3、transaction 34、unit 2、replay-prefix 8。登记 5.9 只钉了 117 行的行文 sha256，扩表各行的行文摘要取造表那一刻的表。
- 逐行驱动加 `--other-selected-rows`：补位时跳过 117 行与扩表两张表里的每一个名字（登记 5.2「扩表与补位都不重挑已跑过的行」）。
- 比对器 `--rows-root` 可以给两份（117 行那份与扩表那份，同一个行号在两份里都有就拒），`--expansion-rows` 给扩表时多两行结果：`name=f4_expansion`（扩表与挑法脚本在 `--mutations-table` 那张表上现算的逐行相同、164 行、序号连续、刀号 3、与 117 行不重名）与 `name=q1_denominator_by_table`（分母按表分开再报）；Q1 的分母与三种读法、Q3 都按并起来的 281 行算。拿 s4 的输入重跑改过的比对器（只给 rows-s4），E7RESULT 行与 `…-s4-comparator.out` 逐字节相同（`cmp` 无输出）。
- 扩表那一趟在 s4 那一份冻住的树上跑（快照摘要 `d1586465…`，与 s4 五趟驱动的相同），两半才能并成同一批；树不重新冻。主 agent 照第七段交回的脚本跑完七步（空变异一行对 `baseline-run1` 未触达、扩表与挑法脚本在冻住的树的表上现算的逐字节相同、这一次两趟驱动与 s4 的快照摘要只有一种），扩表那一趟两条线道 7 607.9 秒。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:137-169`（整段抄，未转述）**

````markdown
### F4 扩表的结果（整行抄自产物）

`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`（4698 行，末行 `E7RESULT name=done emitted=4698`；耗时行在 `…-s4-f4-rows-timing.out`，291 行）。并表、分母、三种读法、Q3、判决行，与扩表新出的全放行行（甲 5 行、丙多出的第 233 行）：

```
E7RESULT name=config segment=s4-f4 o3=frozen_executor baseline_run=baseline-run1 rows_root=rows-s4+rows-s4-f4 controls=8 disabled_runs=2 q3_core_comparisons=3
E7RESULT name=freeze_check phase=start reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=f4_expansion rows=164 expected=164 first_number=118 last_number=281 numbers_are_consecutive=true slice_3_rows=164 names_shared_with_the_117_rows=0 found_once_in_today_table=164 equal_to_the_selection_script_on_today_table=true
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=232 only_in_the_ignored_pass=0 compile_failed=6 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
E7RESULT name=f6_table_failures rows_not_counted_for_the_table=12 rows_run=281 more_than_half=false
E7RESULT name=q1_column_red_rows counted_rows=37 O1=12 O1-对拍=11 O1-崩溃判官=4 O2=15 O3甲=0 O3乙=0
E7RESULT name=untouched_but_red rows=2 row_numbers=155,161
E7RESULT name=q1_alpha counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_beta counted_rows=37 all_quiet_rows=12 row_numbers=15,23,27,34,42,92,112,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,233,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_gamma counted_rows=37 all_quiet_rows=13 row_numbers=15,23,27,34,42,92,112,233,242,269,270,271,277 batch_2_all_quiet_rows=15 rows_that_differ_from_batch_1=102,265 verdict=yes_a_row_is_quiet_on_every_oracle
E7RESULT name=q1_o3_coverage counted_rows=37 rows_with_no_strict_judgement=0 strict_reading_unreachable=false
E7RESULT name=q1_delta rows_quiet_on_the_three_oracles_but_red_on_the_record_checker_or_the_layer0_oracle=none
E7RESULT name=q3_alpha caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q3_beta caught_rows=25 only_O1=32,45,108,115,172,233,237,249,250,281 only_O2=8,65,86,87,90,99,102,135,234,238,245,258,265 rows_whose_red_columns_differ_between_batch_1_and_2=102,233,265
E7RESULT name=q1_denominator_by_table table=step_8_table rows_run=117 counted_D=19 counted_row_numbers=8,11,15,23,27,32,34,42,45,65,86,87,90,92,99,102,108,112,115
E7RESULT name=q1_denominator_by_table table=step_4_expansion rows_run=164 counted_D=18 counted_row_numbers=135,172,233,234,237,238,242,244,245,249,250,258,265,269,270,271,277,281
E7RESULT name=freeze_check phase=end reference=e170-r1-o3-spec-executor-2026-10-03-s1-executor.out files=15 seed=85441ae491419411 differences=0 detail=none
E7RESULT name=verdict a3_columns_with_red_on_the_baseline=0 z1_files_differing=0 s2_files_differing=0 s5_numbers_that_differ=0 positive_controls_run=8 positive_controls_not_red=0 disabled_runs_with_differences_outside_the_closed_column=0 q3_core_runs=9 q3_core_runs_with_differences=0 rows_untouched_but_red=2 negative_bases_with_o2_red=0 negative_bases_with_o3_red=0 known_bad_rows_with_o2_quiet=0 o3_freeze_differences=0 o3_digest_mismatch_states=0 o3_dump_problem_lines=0 o3_image_digest_mismatches=0 rows_with_o3_freeze_lines_not_the_reference=0 rows_with_the_o3_executor_failed=0 a2_p_rows_not_expected_not_in=0 f3_p_rows_judged_in_by_o3=4 q2_geometry_unstable_kinds=2 q1_alpha_all_quiet_rows=12 q1_beta_all_quiet_rows=12 q1_gamma_all_quiet_rows=13 q1_o3_strict_reading_unreachable=0 q2_all_quiet_alpha=4 q2_all_quiet_beta=4 q3_alpha_rows_held_by_a_single_oracle=23 q3_beta_rows_held_by_a_single_oracle=23 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 rows_compiled_without_the_rebuild_check=0
E7RESULT name=q1_quiet_row reading=q1_alpha row=242 row_name=增补_3_第_4_件（增补_2_收口表第_40_行_/_C381_的前提）：发布的持久顺序把系统配置槽轮换挪到根槽_FUA_写之前，最后一步失败时根就不会已经落盘 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=516 o3_judged_states_provisional=516 first_operation_that_differs_from_the_baseline=8/8 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=269 row_name=C554_乙-配置续：见证值取成槽世代号（mkfs_之后第一次取号写的_tail_不再是_0，新池新建文件的字节变了） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=0/0 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=270 row_name=C577：发布在系统配置轮换之后不过屏障就返回（树表_0_条的一版上返回之后见证值可以落后） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=390 o3_judged_states_provisional=390 first_operation_that_differs_from_the_baseline=20/20 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=271 row_name=D8_已定项_8_头_ID：deleted_inodes_的树表条目头_ID_退回写_0（不写它服务的_inode_树的树_ID） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=44/44 layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_alpha row=277 row_name=取前缀核单测：施加一次发布之后新根段不换（停在所选根的） o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=false
E7RESULT name=q1_quiet_row reading=q1_gamma row=233 row_name=由记录重建的根不照抄所选根的文件系统标识（代码轮_m3-formal-code-r1_K3，R3-1「其余字段等于所选根」）：改写全零 o1_comparison_red_calls=0 o1_crash_judge_red_states=45 o2_red_states=0 o3_judged_states_strict=450 o3_judged_states_provisional=450 first_operation_that_differs_from_the_baseline=same/same layer0_or_record_checker_red=true
```

分类的计数（一条命令从产物里数，原样输出）：`grep -o 'name=row row=[0-9]* .* category=[a-z_]*' research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out | sed -E 's/.*category=//' | sort | uniq -c` 输出 `6 compile_failed`、`37 counted`、`6 self_crash`、`232 untouched`（四行）。不进分母的 12 行：第 6、7、85、125、235、272 行实现自崩，第 33、59、127、163、206、243 行编不过（这一份是补位之前的读数，补位那一趟在「编不过的 6 行补位（已跑）」一节）。第 1–117 行的结果行（`name=row`、`row_touch`、`row_matrix`、`trajectory`、`row_o3`、`row_s5`）在 F4 产物与 s4 产物里逐字节相同（1819 行，`cmp` 无输出），7 行全放行的 `name=q1_quiet_row` 三种读法逐行相同：s4 那 7 行的三分不重做。Q2、F3 与对照的结果行两份也逐字节相同。拿第八段改过的比对器重跑 F4 与 s4 的输入，E7RESULT 行与两份产物各自逐字节相同。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:170-184`（整段抄，未转述）**

```markdown
### F4 新出的全放行行

读数取 F4 的导出目录（`scripts/experiment-apparatus/target/e171-dumps/rows-s4-f4/row-<序号>/dump/`，与 `baseline-run1` 逐条 op、逐态比）；草稿脚本 `quiet_rows_f4.py`（第七段 `quiet_rows_s4_fua.py` 改输出根）、`extra_checks_f4.py`、`order_by_segment_f4.py`（都只读导出目录，不入库）。三分的口径与「三条全放行的 7 行」一节相同。顺序类的两行（第 242、270 行）写流的条数或次序变了，逐态与基线比没有意义，改用一条判法在这一行自己的态上数「条款不许的态」，同一判法在基线上数一遍都是 0（判法分得出）：切段用比对器自己的 `baseline_checks.segments_of_the_write_stream`（FUA 写放行它那块盘、屏障放行它那块盘）。

| 行（变异） | 和基线不同在哪 | 条款不许的态（批 1 / 批 2） | O1（内存模型对拍（HashMap 理想文件系统）） / O2（独立解析器 + checker） / O3（独立规约执行器） 看不看得到 | 三分 |
|---|---|---|---|---|
| 242（发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前） | 两条流第 8 条 op 起：每次取号、暖机、发布末尾的「根槽 FUA → 两块盘的系统配置槽 → 屏障」换成「两块盘的系统配置槽 → 根槽 FUA → 屏障」，写的内容逐条相同、次序变了（流一 9 条、流二 57 条 op 按下标比不同）；基线里根槽 FUA 自成一段，变异把它与系统配置写切进同一段（流一 8 段里 3 段、流二 59 段里 19 段，基线 0 段）；调用结局同 | 同一段里系统配置写已落、根槽 FUA 没落全：流一 21 / 3 态，流二 133 / 19 态；基线 0 | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：根槽没落，恢复择到前一版，前一版是模型提交过的、不比已持久的最新根槽旧，判绿；O1（内存模型对拍（HashMap 理想文件系统））-对拍：调用结局同；O2（独立解析器 + checker）：这一行 0 态红（单个镜像里系统配置槽比根早一次发布，没逐条查 `crates/singlefs-checker/invariants/` 有没有管这一样的）；O3（独立规约执行器）：A(c) 由这一行自己写的根构成，恢复落到的版本在里面，「在」 | **真盲区候选**：D16（发布语义） 已定项 7 定案「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」「系统配置槽在根槽持久之后再更新」（`.claude/kb/decisions/16-发布语义.md`「已定项 7：发布的持久顺序」）；系统配置先于它见证的根落盘的态，照条款的写序造不出来，三条都放行 |
| 269（见证值取成槽世代号） | 流一第 0、1 条 op（mkfs 之后第一次取号写的两块盘系统配置槽）偏移 469 一字节 0x00 → 0x01，连同校验和（155–158）；流二另有第 92、93 条 op（可写挂载取号那一写）0x04 → 0x06。偏移 469 是 journal tail（`crates/singlefs-checker/src/lib.rs` 的 `SYSTEM_CONFIGURATION_TAIL_OFFSET`）；调用结局同；看 journal 那一遍恢复逐态同 | 镜像与基线不同的态：流一 31 / 0，流二 78 / 14（全持久态相同：之后的发布把那一槽写成正确的 tail） | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot` 不带系统配置，恢复不先信 tail（D23（journal 的角色与格式） 已定项 3），读回同；O2（独立解析器 + checker）：读了 tail（`crates/singlefs-checker/src/lib.rs` 的 `check_system_configuration_slot`），没有一条不变量判它的值；O3（独立规约执行器）：执行器只在锚点断言里读 tail（`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/executor_main.py` 的 `anchor_assertions` 里），只核全持久态，判定不用它 | **真盲区候选**：D23（journal 的角色与格式） 已定项 14 写「取号那一写……改写取号那一刻按同一取法读到的见证值 c_见证」（全部自证过的槽 journal_tail 取最大）、「mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变」（`.claude/kb/decisions/23-journal的角色与格式.md`「已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换」）；tail 写成 1 的系统配置槽不在条款写得出的镜像里，三条都放行 |
| 270（发布在系统配置轮换之后不过屏障就返回） | 每次发布末尾那道屏障没了：流一 57 → 53 条 op、流二 595 → 565 条，之后的返回点整体前移；写的内容同；系统配置轮换与下一次调用的第一批写切进同一段 | 一次调用的系统配置写没落全、之后的调用的写已落：流一 7 / 6 态，流二 70 / 59 态；基线 0 | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：那次发布的根槽 FUA 已落，恢复择到它，判绿；O1（内存模型对拍（HashMap 理想文件系统））-对拍：调用结局同；O2（独立解析器 + checker）：系统配置落后根一次发布在单个镜像里合法（I-7.12（系统配置 F 不低于同盘根上的 F） 只比 F）；O3（独立规约执行器）：「在」 | **真盲区候选**：D16（发布语义） 已定项 7 定案「根槽（FUA）→ 系统配置槽 → 屏障；fsync 等系统配置轮换持久之后才返回」（第 181 行），射程「屏障口径」写明系统配置轮换之后那一道屏障是「发布返回前加屏障」（第 186 行）；返回之后系统配置还没落、后面的写先落的态照条款造不出来，三条都放行 |
| 271（deleted_inodes 的树表条目头 ID 写 0） | 两条流第 44、45 条 op（树表单元，两块盘各 16 KiB）偏移 1647、1649 两字节 0x01 → 0x00（头 ID 65537 → 0），连同单元校验和；指向它的节点（第 48、49 条 op）与根槽（第 52 条 op）的指针与校验和随之变；流二之后重写树表的发布都带着（85 条 op 不同）；调用结局同；恢复逐态同 | 择到新版（根 txg 不小于第一处不同之后写出的第一条根）：流一 12 / 0 态，流二 356 / 112 态 | O1（内存模型对拍（HashMap 理想文件系统））：不读树表；O2（独立解析器 + checker）：树表条目只解树 ID、种类、诞生 txg（`crates/singlefs-checker/src/walk.rs` 的 `tree_table_entry_of_the_walk`），头 ID 不判；O3（独立规约执行器）：投影只带树表单元指针，A(c) 是这一行自己写的根，「在」 | **真盲区候选**：D8（核心索引结构） 已定项 8 定案「头 ID 8 字节的取值：……`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537）」（`.claude/kb/decisions/08-核心索引结构.md`「已定项 8：树表条目的字段表」）；择到新版的态里头 ID 是 0，三条都放行 |
| 277（取前缀施加一次发布之后新根段不换） | 写流与调用结局全同；看 journal 那一遍恢复在流一批 1 6 态、流二批 1 51 态（批 2 0 态）不同：基线施加 1 条记录、落到新的一版，变异同样施加 1 条、却停在所选根那一版（例：流一第 41 态基线 `1,3 file_read e8ca4bf8…`，变异 `1,2 no_file`） | 0：57 态里被丢掉的那一版的调用都没返回、它的根槽都没落（`extra_checks_f4.py`：全是 `call_returned=False its_root_slot_landed=False`） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：前一版是模型提交过的、不比已持久的最新根槽旧、读回与那一版相同，判绿是对的；O2（独立解析器 + checker）、O3（独立规约执行器）：镜像与基线逐字节相同 | **另一类：三条不判实现恢复里靠记录重建的那一段**：调用没返回，新旧两版都在「根 + journal 任意前缀」（D16（发布语义） 已定项 12）里，态本身合法；错在实现的恢复施加记录之后新根段不换，违反 D13（验证路线） 已定项 1 的 M4 与 D23（journal 的角色与格式） 已定项 15（同第 27 行），三条不判；M4 的落点是 E165（恢复重放有界穷举与变异杀伤界） 与 Verus。会分开的历史（推的，没跑）：一次已返回的发布之后根槽读不出、恢复要靠 journal 重放追上那一版（D16（发布语义） 已定项 7 射程「同一实例里退一代，靠 journal 重放追得上」），变异会停在前一版；两条流里没有读坏根槽的态 |

- **第 233 行（由记录重建的根不照抄所选根的文件系统标识，改写全零），只在丙里全放行**：写流与调用结局全同；看 journal 那一遍恢复在施加了记录的态上走读失败（`InvariantViolated`，`invariant` 字段是 I-1.4（块头 fsid 一致），`detail` 是「树根 fsid 不符」），流一批 1 3 态、流二批 1 48 态（其中 6 态写落在卸载里、崩溃判官记不适用）。甲、乙里 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官判红（流一 3 态、流二 42 态，段为正 2/12、28/79），只靠 O1（内存模型对拍（HashMap 理想文件系统））；丙只取 O1（内存模型对拍（HashMap 理想文件系统））-对拍，调用不经恢复，三条全放行。在丙里是**真盲区候选**：实现在合法的崩溃镜像上恢复不出任何一版，D13（验证路线） 已定项 1 的 M2 写重放结果的根「其余字段等于所选根」（`.claude/kb/decisions/13-验证路线.md`「已定项 1」表里 M2 那一行）；O2（独立解析器 + checker） 判恢复之前的镜像（与基线逐字节相同），O3（独立规约执行器） 跑的是它自己的恢复。崩溃点重放那份 oracle 与记录核对器在这一行上红（`layer0_or_record_checker_red=true`，不是三条之一）。批 2 不红：两条流批 2 一个「基线施加了记录」的态都没有（`extra_checks_f4.py` 输出 `b2_base_records_applied_gt0: 0`），**取样点不敏感**，会分开的取样点是任一个施加了记录的态；这一格两批翻面，记「不稳定」。
- **第 265 行（抬 F 那一串不先写系统配置），批 2 多出来的全放行**：只有流二第 440 条 op 起不同（抬 F 先写系统配置那一步整步没了，149 条 op 按下标比不同）；O2（独立解析器 + checker） 在批 1 判红 12 态，都是 I-7.12（系统配置 F 不低于同盘根上的 F），在第 59、71 段各 6 态（`name=trajectory row=265 stream=fixed_script_through_e_then_unmount oracle=O2（独立解析器 + checker） total_red_states=12 peak=6 positive_segments=2 of_segments=77 …`）。批 2 在流二取的段是 7、11、16、20、24、28、32、36、40、44、48、52、56、60、64、68、72，没有第 59、71 段（`extra_checks_f4.py` 输出 `batch2_states_in_segments_59_71=0`）：**取样点不敏感**，同第 102 行；记「不稳定」。条款是 D16（发布语义） 已定项 7 定案「抬 F 那一串……先把新 F 写进每块盘的系统配置、过一道屏障」。Q1、Q3 的结论字在批 2 上不翻（批 2 全放行 15 行，只靠一条的行仍 ≥ 1）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:185-201`（整段抄，未转述）**

```markdown
### 三轮判决之后的三分

三轮结论判决（`research/prompts/e171-conclusion-r1-main-verification.md`、`…-r2-main-verification.md`、`…-r3-main-verification.md`；第三轮是最终轮，以它第三节「三轮之后的结论」与第五节为准）把这批读数重新三分。口径：**真盲区候选**＝这一批取到的态在已定条款下不合法、三条都放行（E171（三oracle合谋测试） s4 树）；**另几类**＝三条按设计不判、或错不在态上。执行员原判「对这一批等价」的五行（第 15、27、34、112、277 行）一行都不再是等价。

| 类 | 行 | 一句话依据（各行条款原文在「F4 扩表的结果（整行抄自产物）」与「F4 新出的全放行行」两节的行里） | 三轮里的翻动 |
|---|---|---|---|
| 真盲区候选 | 第 15 行 | D3（空间分配） 已定项 10 ⑤ 的 t2–t12 落点：变异下十一个提交内生块整体挪 64 槽（`row15-offsets.out`），⑤ 与 `layout/01-first-txn.md` 里「提交内生块」那一行钉的是这些落点 | 第一轮从「对这一批等价」挪进来（D3（空间分配） 已定项 10 ③）；第二轮维持，依据写 ⑤ 落点表与 `layout/01-first-txn.md` 里「用户数据落点政策」那一行的 t1@50180；第三轮留名单、依据换成 ⑤ 的 t2–t12 落点（主 agent 现查 ⑤ 表 t2@50240 与第 220 行「改新池新建文件的字节：是」，复跑 `row15-offsets.out` 同） |
| 真盲区候选 | 第 23、42、271 行 | 第 23 行 D8（核心索引结构） 已定项 6 的容器号；第 42 行 D16（发布语义） 已定项 1 卸载抬 F；第 271 行 D8（核心索引结构） 已定项 8 头 ID | 三轮都在名单 |
| 真盲区候选 | 第 242、269、270 行 | 第 242、270 行 D16（发布语义） 已定项 7 的写序；第 269 行 D23（journal 的角色与格式） 已定项 14 的见证值 | 第一轮被改出名单（按已定项 12 合法、写序义务无人判），第二轮翻回，第三轮没被打中 |
| 另一类 | 第 27、112、277 行 | 三条不判实现恢复里靠记录重建的那一段（D13（验证路线） 已定项 1 的 M4、D23（journal 的角色与格式） 已定项 14 注 1 与已定项 15）；第 112 行已在 `.claude/gate.d/e165-pinned-red-examples.tsv` 里 | 第一轮采正推改归这一类，第二轮站住、措辞收窄，第三轮没变 |
| 另一类 | 第 34 行 | 分配政策，镜像合法：D3（空间分配） 已定项 8 射程 | 第一轮与第 15 行一起挪进真盲区候选，第二轮翻回，第三轮维持 |
| 另一类 | 第 92 行 | D8（核心索引结构） 已定项 6 已补写入时间取值（2026-10-04）；三条 oracle 按设计判不到，由钉写的用例判 | 第一轮归「条款没定且不可见」不进名单，第二轮改理由，第三轮维持 |
| 另一类（零轮） | 第 110、162、174 行 | 运行时读数（写账、分支名），三条按设计都不判，与第 34 行同类 | 第三轮新冒出：攻方量交回值时发现（「没触达却红的两行与未触达行里交回值不同的七行」一节） |
| 另一类（零轮） | 第 49、252 行 | 交回值或内存状态错、写流不变，要更长的历史才分得开 | 第三轮新冒出：攻方量交回值时发现（「没触达却红的两行与未触达行里交回值不同的七行」一节） |

第 155、161 行是已知的那两行（O1（内存模型对拍（HashMap 理想文件系统））-对拍判红），不在这几类里；第 233 行（丙里全放行、甲乙里 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官抓到）、第 265 行（批 2 取样点不敏感）的读法在「F4 新出的全放行行」一节，第 102 行（批 2 取样点不敏感）在「三条全放行的 7 行」一节末，三轮没有改。「另一类（零轮）」两行的意思是这一类是第三轮才冒出来的，没有再开轮，进欠账，不改登记的触达定义。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:202-222`（整段抄，未转述）**

```markdown
### F4 新出的只靠一条的 12 行

只写够不到的对象与没判的字段，不判结论。读数取 F4 的导出目录（草稿脚本 `caught_rows_f4.py`，第七段 `caught_rows_s4.py` 改输出根，不入库）与产物的 `name=trajectory`、`name=row_matrix` 行；段为正的数是「红态的段数 / 这条流的段数」。

| 行（变异） | 那一条红在哪 | 另外两条为什么看不到 |
|---|---|---|
| 172（交进来的盘数等于 w 的下限也拒） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 交回 `err:WritableDeviceCountBelowTheStripeWidthLowerBound`，模型说该成；流二停在第 92 条 op | 被拒之前的写流与基线逐条相同（0 条 op 不同），取到的镜像都是基线那一批的前缀；被拒的调用不留镜像，O2（独立解析器 + checker）、O3（独立规约执行器） 没有对象可判（同第 45 行） |
| 233（由记录重建的根不照抄所选根的 fsid） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：批 1 流一 3 态、流二 42 态走读失败（I-1.4（块头 fsid 一致） 树根 fsid 不符），段为正 2/12、28/79；批 2 0 态 | 错在实现的恢复，写流与基线逐字节相同；O2（独立解析器 + checker） 判恢复之前的镜像，O3（独立规约执行器） 跑它自己的恢复。批 1 与批 2 红的列不同：批 2 没有施加了记录的态，取样点不敏感（「F4 新出的全放行行」一节） |
| 237（解系统配置槽时根环起点读错了位置） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：批 1 流一 53 态、流二 73 态（段为正 12/12、16/16），批 2 7、14 态，「没择到根」（`SystemConfigurationValueRefused … RootRingBaseNotTheFirstVersionSlot`）；O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 交回 `err:Recovery` | 错在实现的读者一侧，写出的字节与基线相同（0 条 op 不同）；O2（独立解析器 + checker）、O3（独立规约执行器） 各用自己的解析器读系统配置，看到的镜像与基线逐字节相同 |
| 249（一个数据单元的发布也切成两条记录） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：两条流第 2 次调用 `publish` 判 `disagrees:jsn`（模型说那一版的 jsn 3，实现 4），模型停下 | 两条记录合起来仍是一次完整的发布：O2（独立解析器 + checker） 在这一行 0 态红（没逐条查 `crates/singlefs-checker/invariants/` 有没有限一次发布几条记录的）；O3（独立规约执行器） 自己重放记录，落到的版本在 A(c) 里，投影不带 jsn |
| 250（数据单元的净荷字节倒序写） | O1（内存模型对拍（HashMap 理想文件系统））-对拍：两条流第 2 次调用 `publish` 判 `disagrees:这一版的文件内容`（读回 3000 字节、内容倒序） | 单元的校验和按写下去的字节算，结构自洽：D13（验证路线） 已定项 6 表 O2（独立解析器 + checker） 行「验不到」一栏写的「一个块被覆盖成另一份合法内容」；O3（独立规约执行器） 的投影不带文件内容，不拿写入的内容去比 |
| 281（择根取第一个自证过的槽就不再换） | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官：批 1 流一 41 态、流二 61 态（段为正 9/12、13/16），批 2 7、14 态，「走到实例 0 第 0 代根」（比盘上已持久的最新根槽旧）；O1（内存模型对拍（HashMap 理想文件系统））-对拍：流二第 4 次调用 `mount_writable` 交回 `err:NewerStateStillUnreadableAfterOneReread` | 错在实现的择根，写流与基线逐字节相同（0 条 op 不同）；O2（独立解析器 + checker） 判镜像结构，O3（独立规约执行器） 用自己的择根规则恢复，看到的镜像与基线相同 |
| 135（复活 defer 里那一条时 defer 槽数不减） | O2（独立解析器 + checker）：I-3.11（已分配减 defer 等于最新根走读）批 1 流二 225 态，段为正 44/79；批 2 70 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot`（`crates/singlefs-harness/src/model.rs` 里它的定义）没有记账行；O3（独立规约执行器）：投影不进记账树 |
| 234（根记录写者不照卸载记号写 flags） | O2（独立解析器 + checker）：I-7.9（回退下界 F 不高于抬 F 的上限）（按卸载记号分两支）批 1 流二 29 态，第 73 段起段为正 6/79；批 2 7 态 | O1（内存模型对拍（HashMap 理想文件系统））：写落在卸载那次调用里，模型没有卸载的答（登记读法 6），对拍不比、崩溃判官记不适用，`ObservedRoot` 也不带根的 flags；O3（独立规约执行器）：投影不带卸载记号（`root_ring.py` 里 `VersionProjection` 的说明：「卸载记号与根记录其余字段不进投影」） |
| 238（改动计数留 1） | O2（独立解析器 + checker）：I-9.26（inode 改动计数落在出生代与容器诞生代号之间）批 1 流二 333 态，段为正 65/79；批 2 105 态 | O1（内存模型对拍（HashMap 理想文件系统））：不比 inode 记录的改动计数；O3（独立规约执行器）：投影不进 inode 树 |
| 245（覆盖写的反向链取上一版记录自己的反向链） | O2（独立解析器 + checker）：I-8.6（反向链算法）（反向链不等的记录不进重放）批 1 流二 336 态，段为正 67/79；批 2 105 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedRoot` 只带 journal 计数器，不带反向链；O3（独立规约执行器）：它的前缀规则核反向链（`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/replay_prefix.py` 的 `replay_prefix`，不等就停），这一行严格、暂代都判「在」；停下之后落到的那一版仍在 A(c) 里是推的（根槽已落、不靠重放），没逐态查它停在哪 |
| 258（不释放旧落点） | O2（独立解析器 + checker）：I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读） 批 1 流二 333 态，段为正 65/79；批 2 105 态 | O1（内存模型对拍（HashMap 理想文件系统））：`ObservedUnitAllocationRecords` 只带这一版每个角色的单元的分配记录（`model.rs` 里 `ObservedUnitAllocationRecords` 的定义），旧落点释没释放不在里面；O3（独立规约执行器）：投影不进分配记录树与记账树 |
| 265（抬 F 那一串不先写系统配置） | O2（独立解析器 + checker）：I-7.12（系统配置 F 不低于同盘根上的 F） 批 1 流二 12 态（第 59、71 段各 6 态）；批 2 0 态 | O1（内存模型对拍（HashMap 理想文件系统））：比的是根带的 F，不比系统配置槽；O3（独立规约执行器）：投影的 F 取自根，A(c) 是这一行自己写的根。批 1 与批 2 红的列不同：批 2 没取第 59、71 段，取样点不敏感（「F4 新出的全放行行」一节） |

O3（独立规约执行器） 在 F4 计入分母的 37 行上一行都没红（`name=q1_column_red_rows … O3（独立规约执行器）甲=0 O3（独立规约执行器）乙=0`），每行两条流的批 1 态都给了严格判定（`name=q1_o3_coverage … rows_with_no_strict_judgement=0`）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:223-237`（整段抄，未转述）**

````markdown
### 没触达却红的两行（第 155、161 行）与未触达行里交回值不同的七行

`name=untouched_but_red rows=2 row_numbers=155,161`：两行的变异是同一处（`crates/singlefs-core/src/mount.rs` 里抬 F 上限 `newest_on_every_device.min(fourth_newest)` 改成 `.max(…)`，两行原文只差行首空白），触达三样都不同（`name=row_touch row=155 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false … touched=false`），O1（内存模型对拍（HashMap 理想文件系统））-对拍在流二第 12 次调用 `raise_floor` 判 `disagrees:抬 F 的上限`（`call-outcomes.tsv`：模型「上限 Some(11)（D16（发布语义） 已定项 1）」，实现「实现报上限 13」），批 1、批 2 都红；O2（独立解析器 + checker）、O3（独立规约执行器） 不红。

- 不是装置、比对器或取样的缺陷：比对器的触达照登记第一节读法 4 字面算（「任一次调用的结局（成功 / 哪一种错误）」，`collusion_matrix.py` 的 `outcome_words` 只比调用名与结局列），这一次调用两边都是 `ok`；O1（内存模型对拍（HashMap 理想文件系统））-对拍比的是成功的调用交回的值（这里是抬 F 的上限），写流不变是因为这一次请求的 F 不超过 11，上限放宽没起作用。O1（内存模型对拍（HashMap 理想文件系统））-对拍按调用算，与取样点无关，两批都红。
- 缺口在登记读法 4 的触达定义：它把「成功的调用交回的值」排在三样之外，却写「三样都相同的行……在这一批上与正常实现逐字节同行为，哪条 oracle 都不该红」；这两行交回值不同，那句前提不成立。装置的 `call-outcomes.tsv` 只在对拍判出不同时记两边的值，成功的调用交回什么没有单独记，入库装置没有这一列。触达定义没改（产物跑过之后不改登记），分母仍 37。
- 别的未触达行有没有同类差异：攻方（第三轮）在 s4 树的副本上量了（副本探针，s4 树，入库装置没有这一列）。探针照流二的驱动在内存稀疏盘上跑一遍，记每次成功调用交回值的 `{:?}` 文本摘要、末了的分配器与现行那一版；基线与 E171（三oracle合谋测试） 导出的 595 条 op 逐条相同；再把未触达的行逐行打进副本、重编重跑、还原。跑了 232 行（238 行去掉 `make_filesystem.rs` 的 6 行，mkfs 不是一次调用，探针不记它的交回值，第 149–154 行没跑），7 行不同：第 49、110、155、161、162、174、252 行。第 155、161 行就是前面说的那两行（这把尺子在同一次第 12 次调用 `raise_floor` 上抓到它们，上限 11 → 13，是现成的阳性对照）；另 5 行 O1（内存模型对拍（HashMap 理想文件系统））-对拍不比那几个字段所以不红：第 49 行可写挂载交回的见证值 4 → 3，第 110 行写账的结构种类，第 162 行影子账分支名，第 174 行抬 F 交回的写账，第 252 行内存里的分配器少了根槽登记（第 49、252 行交回值或分配器不同，写流不变）。汇总行整行抄自攻方 `return-rows.out` 的 `name=return_summary` 那一行：

```
name=return_summary hidden_difference=7 identical=225 no_output=6
```

  主 agent 复跑（`tools/rerun.sh … all`）这 7 行逐字同；`return-rows.out` 只差变异表第 28–31 行那四条 `name=return_row`（复跑是 `status=no_output`、汇总 `hidden_difference=7 identical=221 no_output=10`，攻方是 `identical`）：攻方在决定去掉 `make_filesystem.rs` 的行之前已跑过这四行，`rerun.sh` 的 `awk` 把它们滤掉，复跑就不再跑（`research/prompts/e171-conclusion-r3-main-verification.md` 第四节）。害处没量：第 49、252 行推的是要更长的历史才分得开。
- 对其余读数的影响：两行不进分母，也不进任何 Q1、Q3 名单。照「触达」把它们算进去，D = 39，两行都只有 O1（内存模型对拍（HashMap 理想文件系统）） 红，Q3 只靠 O1（内存模型对拍（HashMap 理想文件系统）） 的行多两行（第 155、161 行），Q1 三种读法的全放行行不变。把 7 行交回值不同的行与第 125 行（「它答不了的」）都算进来，D 由 37 变 45（推的，算术，没在入库装置上跑），那 5 行 O1（内存模型对拍（HashMap 理想文件系统）） 不红，会成为新的全放行行。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:238-265`（整段抄，未转述）**

````markdown
### 编不过的 6 行补位（已跑）

登记第一节读法 5「替换之后编不过（`.claude/rules/mutation-sampling.md` 第八类，按 5.2 的补位规则换行）」；s4 与 F4 那一版驱动只在编之前的失败上补位，第 33、59、127、163、206、243 行编不过之后照原样记编不过、没有补位。第八段把驱动改成编不过之后也补位（换同一文件组里下标 j + 1、两张表里都没有、这一次跑里别的补位也没用过的那一行，换上的行编之前照样核锚点，编不过再换，编之前的失败与编不过合起来一行至多三次；组里不含 E171（三oracle合谋测试） 装置自己的变异行），加 `--rerun-compile-failed-from`（只跑几个输出根里结局是编不过的行）；比对器加 `--substitution-rows-root`（补位那一趟的行换掉原来编不过的同号行，别的情形拒），多出 `name=substitution` 与 `name=substitution_summary` 两种结果行。

- 补位那一趟在 s4 那一份冻住的树上跑（快照摘要与 s4 五趟的相同，空变异一行对 `baseline-run1` 未触达），只跑两份输出根里结局是编不过的行（`rerun_compile_failed rows=33,59,127,163,206,243`），两条线道、每条 2 线程，六步都退 0，补位 6 行挂钟 1 277.9 秒；比对器把 117 + 164 + 补位并成 `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`（4795 行，末行 `E7RESULT name=done emitted=4795`），耗时行在 `…-s4-f4-substitutes-rows-timing.out`（292 行）。
- 补位的六行（整行抄自 `…-s4-f4-substitutes-comparator.out` 里六行 `name=substitution`）、补位汇总（`name=substitution_summary`）与分母（`name=q1_denominator rows_run=`）：

```
E7RESULT name=substitution row=33 original_name=步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 original_status=compile_failed substituted_by=步_4：影子账把被抛弃根账里已释放的落点也隔离 failed_attempts=1 attempts=compile_failed:步_4：回退候选集的_F_用最新根自己带的_F_而不是_F_生效 category=untouched
E7RESULT name=substitution row=59 original_name=C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） original_status=compile_failed substituted_by=C483_②：挂载态打开时_inode_叶容器只按位置提示读（不经映射回退） failed_attempts=1 attempts=compile_failed:C483_②：挂载态打开时_inode_树根只按位置提示读（不经映射回退） category=untouched
E7RESULT name=substitution row=127 original_name=增补_2：回落只看已分配位（发出影子账隔离的槽） original_status=compile_failed substituted_by=增补_2_第_20c_行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放） failed_attempts=2 attempts=compile_failed:增补_2：回落只看已分配位（发出影子账隔离的槽）|compile_failed:增补_2：回落只看已分配位（抬_F_的空发布发出扣住的槽） category=untouched
E7RESULT name=substitution row=163 original_name=C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） original_status=compile_failed substituted_by=D8_已定项_8_②：可写挂载写行那次的树_ID_水位只沿所选那一版带，不取根环与环里记录的_max（原为_C511_第_3_步回退到暖机根那一格；回退改成向前发布之后由崩溃恢复丢掉一次第一个文件版本那一形护） failed_attempts=1 attempts=compile_failed:C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷） category=untouched
E7RESULT name=substitution row=206 original_name=步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） original_status=compile_failed substituted_by=增补_3_第_4_件：一块盘两个系统配置槽都无效就整体_return_Err（每盘一份的冗余当场作废，D22_已定项_8_第_1_条） failed_attempts=1 attempts=compile_failed:步_6：恢复从系统配置_tail_起逐条验证记录的点名单元、失配即中止（E78_的自我中止形态） category=untouched
E7RESULT name=substitution row=243 original_name=并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 original_status=compile_failed substituted_by=普查_R2：释放判定路径把「映射条目切不动」报成「不在映射」（两件要分流的事并成一个成员） failed_attempts=1 attempts=compile_failed:并行线三_C116：叶容器被_COW_重写时身份跟着这次发布换出生代 category=untouched
E7RESULT name=substitution_summary rows_that_did_not_compile=6 substituted_rows=6 still_not_compiled=0 row_numbers=33,59,127,163,206,243
E7RESULT name=q1_denominator rows_run=281 counted_D=37 untouched=238 only_in_the_ignored_pass=0 compile_failed=0 anchor_not_unique=0 not_found=0 self_crash=6 timeout=0 device_stop=0 wrapper_ended=0 enough_to_judge_none_side=true
```

- 六行换上的都是 `category=untouched`（第 127 行前一个换上的行也编不过，换了第二次）；分母 D = 37 没变：`compile_failed` 6 → 0、`untouched` 232 → 238，`name=f6_table_failures` 的 `rows_not_counted_for_the_table` 12 → 6（只剩六行实现自崩）。补位产物比补位之前的 F4 产物多出来的行只有这六行的 `name=row`、`row_touch`、`row_matrix`、`row_s5`、`row_o3` 与 `trajectory`，加 `name=substitution` 六行与 `name=substitution_summary`，改了的行是 `name=config`、`name=q1_denominator`、`name=f6_table_failures` 与 `name=done`。
- Q1、Q3 名单与补位之前逐字同（自己 diff 两份产物核）：

```
cd research/results; A=e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out; B=e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out; P='name=(q1_alpha|q1_beta|q1_gamma|q1_quiet_row|q1_column_red_rows|q3_alpha|q3_beta|untouched_but_red|q1_denominator_by_table|q1_delta|q1_o3_coverage) '; diff <(grep -E "$P" $A) <(grep -E "$P" $B); echo "diff_rc=$?"; grep -cE "$P" $A; grep -cE "$P" $B
```

  输出 `diff_rc=0`、`48`、`48`；`name=q2_`、`name=f3_`、`name=verdict`、`name=f4_expansion`、`name=q1_denominator_by_table`、`name=positive_control`、`name=freeze_check`、`name=q3_core_row` 开头的行两份也逐行同（同一条命令把 `P` 换成 `'name=(q2_|f3_|verdict|f4_expansion|q1_denominator_by_table|positive_control|freeze_check|q3_core_row)'`，输出 `diff_rc=0`、`171`、`171`）。
- 拿改过的比对器（不给补位根）重跑 s4 与 F4 的输入，E7RESULT 行与两份产物各自逐字节相同；拿一个手造的补位根（一行换成第 34 行的导出、一行仍编不过）空跑过一遍，分母与名单照换过的行算、两种新结果行照写（草稿，不入库）。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:266-294`（整段抄，未转述）**

````markdown
### Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像（整行抄自产物）

`…-s4-f4-substitutes-comparator.out` 第 4759、4760 行（补位之前的 `…-s4-f4-comparator.out` 第 4662、4663 行同两行，`sed -n '4662,4663p'` 与 `sed -n '4759,4760p'` 各自喂 `md5sum` 都是 `07b56b24014dac5d9eb543ef15384aa9`）：

```
E7RESULT name=q2_family family=P rows=26 o2_red=10 o2_quiet=16 o3_strict_red=22 o3_provisional_red=22 o2_only_strict=0 o3_only_strict=12 both_red_strict=10 o2_only_provisional=0 o3_only_provisional=12 both_red_provisional=10 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:2,new:3,new:5,new:7,new:8,new:9,new:10,new:11,new:12,fix:2,fix:3,fix:5,fix:8,fix:9,fix:11,fix:12
E7RESULT name=q2_family family=B rows=40 o2_red=23 o2_quiet=17 o3_strict_red=40 o3_provisional_red=40 o2_only_strict=0 o3_only_strict=17 both_red_strict=23 o2_only_provisional=0 o3_only_provisional=17 both_red_provisional=23 o3_strict_spec_gap=0 o3_error=0 o2_quiet_rows=new:18,new:19,new:20,new:23,new:26,new:28,new:30,new:32,new:33,new:35,fix:21,fix:22,fix:23,fix:24,fix:25,fix:32,fix:35
```

12 + 17 = 29 张是 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 严格判「不在」。从 `name=q2_row` 逐行数同一批、按补丁种类分（命令与原样输出；`B` 是前面那份产物）：

```
grep 'name=q2_row family=[PB] ' B | grep ' o2=green ' | grep -c 'o3_strict=not_in'
29
for f in P B; do echo "== $f"; grep "name=q2_row family=$f " B | grep ' o2=green ' | grep 'o3_strict=not_in' | sed -E 's/.*bad_name=([^ ]*) .*/\1/' | sed -E 's/^B[0-9]+:(P[0-9]+):.*/\1/; s/^(P[0-9]+)-.*/\1/' | sort | uniq -c; done
== P
      1 P2
      4 P3
      4 P4
      1 P5
      2 P9
== B
      4 P2
     12 P3
      1 P5
```

P 族 12 张是 5 种补丁（P2、P3、P4、P5、P9）的实例，盲测 B 族 17 张是 3 型（P2、P3、P5）的随机实例；全是结构自洽的伪造根、伪造续接镜像。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:295-397`（整段抄，未转述）**

````markdown
### 第一段的基线与对照（整行抄自 s1 产物（产物已移出工作区的行写成一行指针））

`research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out`（基线第一趟；两趟的 E7RESULT 行逐字节相同，`diff` 无输出）：

```
E7RESULT name=stream stream=new_pool_file_creation ending=completed exported=true not_exported_why=- operations=57 writes=41 segments=11 tearable_writes=8 calls=3 full_states=16777260 quick_tier_states=46 sampled_states=82 model_stopped_at_call=none
E7RESULT name=batch stream=new_pool_file_creation batch=1 states=53 segments_touched=12 states_with_a_torn_write=20 o1_crash_judge_red=0 o1_crash_judge_green=53 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=new_pool_file_creation batch=2 states=7 segments_touched=1 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=7 o1_crash_judge_not_applicable=0 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=stream stream=fixed_script_through_e_then_unmount ending=completed exported=true not_exported_why=- operations=595 writes=477 segments=78 tearable_writes=46 calls=15 full_states=1662648564 quick_tier_states=278 sampled_states=890 model_stopped_at_call=14
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=1 states=397 segments_touched=79 states_with_a_torn_write=115 o1_crash_judge_red=0 o1_crash_judge_green=349 o1_crash_judge_not_applicable=48 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=batch stream=fixed_script_through_e_then_unmount batch=2 states=119 segments_touched=17 states_with_a_torn_write=0 o1_crash_judge_red=0 o1_crash_judge_green=105 o1_crash_judge_not_applicable=14 o1_crash_judge_off=0 o2_red=0 o2_matches_judgement_false=0 judge_crash_image_pool_checker_red=0 record_checker_red_states=0 consulted_oracle_red_states=0 ignored_oracle_red_states=0 consulted_effective_root_none=0 consulted_failed_outcomes=0
E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
```

`e171-three-oracle-collusion-2026-10-03-s1-comparator.out`，基线与对照（Z1 两趟 26 个文件逐字节相同、S2 与 E170（O3规约执行器第一版） 导出装置 18 项逐字节相同、S5 两个数比对器自己现算、a3 基线 0 红）：

```
E7RESULT name=a4 table_rows=117 expected=117 per_file_matches_the_registration=true found_once_in_today_table=117 line_unchanged_since_the_registration=117
E7RESULT name=a5 first_slice_rows=24 expected=24 distinct_files=24
E7RESULT name=a6 step_8_rows=117 step_4_rows=220 expected_step_4=220 overlap=56 expected_overlap=56 step_4_only=164 expected_step_4_only=164 step_8_names_equal_the_registered_table=true
E7RESULT name=a7 implementation_rows=844 expected=844 files=24 expected_files=24
E7RESULT name=a3_baseline_reds o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o2_red_states=0
E7RESULT name=s5_batch_1_states stream=new_pool_file_creation recomputed=53 device=53 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=1 recomputed=53 device=53 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=2 recomputed=7 device=7 equal=true
E7RESULT name=s5_o1_applicable stream=new_pool_file_creation batch=3 recomputed=22 device=22 equal=true
E7RESULT name=s5_batch_1_states stream=fixed_script_through_e_then_unmount recomputed=397 device=397 equal_counts=true equal_ordinals=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=1 recomputed=349 device=349 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=2 recomputed=105 device=105 equal=true
E7RESULT name=s5_o1_applicable stream=fixed_script_through_e_then_unmount batch=3 recomputed=330 device=330 equal=true
E7RESULT name=z1_summary files_compared=26 files_differing=0
E7RESULT name=s2_summary files_compared=18 files_differing=0
```

阳性对照（登记 5.6，每条 oracle 在基线与空变异各一趟）与关一条的开关自检：

```
E7RESULT name=positive_control oracle=O1-对拍 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O1-对拍 run=empty_mutation red_units=1 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=baseline red_units=2 red=true
E7RESULT name=positive_control oracle=O1-崩溃判官 run=empty_mutation red_units=2 red=true
E7RESULT name=positive_control oracle=O2 run=baseline red_units=1 red=true
E7RESULT name=positive_control oracle=O2 run=empty_mutation red_units=1 red=true
E7RESULT name=q3_core closed=O1 run=disable-O1 differences_outside_the_closed_column=0 where=none
E7RESULT name=q3_core closed=O2 run=disable-O2 differences_outside_the_closed_column=0 where=none
```

三行试跑的分类、触达三样与 Q1 的分母：

```
E7RESULT name=row row=0 row_name=空变异（C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式）） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=0 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=1 row_name=C318_实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒_0（影子账隔离的单元又不进不等式） category=untouched substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=1 write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false ignored_pass_differs=false touched=false
E7RESULT name=row row=6 row_name=K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出） category=self_crash substituted_from=- line_matches_the_registration=true
E7RESULT name=row_touch row=6 write_stream_differs=true call_outcomes_differ=true recovery_report_differs=false ignored_pass_differs=false touched=true
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 102 行（name=q1_denominator）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 103 行（name=f6_table_failures）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 104 行（name=q1_o1_and_o2_both_quiet）
E7RESULT name=q1_o1_and_o2_both_quiet_batch_2 rows=0 rows_that_differ_from_batch_1=none
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 106 行（name=q1_column_red_rows）
E7RESULT name=untouched_but_red rows=0 row_numbers=none
```

Q2 的 O2（独立解析器 + checker） 一列（O3（独立规约执行器） 两列、盲测族没跑）：

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 191 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 192 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 193 行（name=q2_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 194 行（name=q2_blind_family）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 195 行（name=q2_negative_bases）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 196 行（name=a9_a10_a8）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 198 行（name=q2_all_quiet）
```

判决行（比对器自己的那一行；O3（独立规约执行器） 一律 `not_run`）：

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-comparator.out 第 199 行（name=verdict）
```

`e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out`（第一刀第 6 行：分配记录树按槽号找叶时叶宽算成两倍，实现在第一次 `publish` 里 panic，装置记「实现自崩」、这条流不导出）：

```
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 2 行（name=stream）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 5 行（name=call）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 14 行（name=stream）
产物行见 git show 5d0b7e7c1667:research/results/e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out 第 17 行（name=call）
```

`research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out`（41 条已知坏镜像在流一全持久态上：补丁叠回与直接改坏判红的不变量集合相等、目标不变量都红、条目名与原文件逐条相同）：

```
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_build_pool local="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" crates="9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034" holds=true
E7RESULT name=s4 check=known_bad_clean_image_digest_equals_the_full_state_of_the_dump local=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") crates=Some("9b4ee0cf7baf2bf175a5727d189d253c8f161695650cdc9023e354d8a4d39034") holds=true
E7RESULT name=s4 check=known_bad_test_file_sha256_is_the_copied_one local="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" crates="e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820" holds=true
E7RESULT name=s4 check=known_bad_entry_count_here local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_count_in_the_original_file local=41 crates=41 holds=true
E7RESULT name=s4 check=known_bad_entry_names_equal_the_original_file_in_order local=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] crates=["I-7.4","I-4.8","I-1.1","I-1.3","I-1.4","I-1.4","I-1.6","I-1.7","I-1.10","I-2.1","I-2.3","I-2.4","I-2.5","I-3.1","I-5.1","I-5.2","I-7.1","I-7.2","I-7.6","I-7.13","I-7.14","I-7.7","I-7.7","I-7.7","I-7.8","I-9.1","I-9.2","I-9.4","I-9.7","I-9.10","I-9.15","I-9.16","I-3.8","I-8.6","I-1.8","I-1.2","I-1.11","I-4.2","I-9.6","I-9.12","I-9.13"] holds=true
E7RESULT name=verdict known_bad_entries_failing_s4=0 e170_bad_rows_failing_s3_or_s4=0
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:398-404`（整段抄，未转述）**

```markdown
### 对照与自检

- 阴性（登记 5.6）：基线两趟、三批的每一态与每一次调用上，O1（内存模型对拍（HashMap 理想文件系统））-对拍、O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官、O2（独立解析器 + checker） 判红都是 0 处（a3，`name=a3_baseline_reds`）；空变异（拿表里第一行、把替换文换成原文本身，走完整条逐行驱动）判未触达、四列不红（`name=row row=0`、`name=row_touch row=0`、`name=row_matrix row=0`）；8 张不打补丁的底态（两条流的 F、Mk、Mk⁺、Mk⁻）O2（独立解析器 + checker） 不红（`name=q2_negative_bases`）。O3（独立规约执行器），第二段起：冻结的执行器在基线两条流批 1、批 2 的 576 态上判红 0 处，s4 产物原行 `E7RESULT name=a3_o3_baseline states=576 strict_in=576 strict_not_in=0 strict_spec_gap=0 strict_error=0 provisional_in=576 provisional_not_in=0 digest_mismatch_states=0 dump_problem_lines=0 base_unrecoverable_streams=none complete=true freeze_lines_equal_the_reference=true`；8 张底态 O3（独立规约执行器） 甲乙都不红（`name=q2_negative_bases … o3_strict_red_bases=0 o3_provisional_red_bases=0`）。s4 整表开跑前，同一棵冻住的树上的空变异一行对 `baseline-run1` 未触达（主 agent 跑的整表脚本第 3 步：`write_stream_differs=false call_outcomes_differ=false recovery_report_differs=false touched=false`）。
- 阳性（装置一侧三条，登记 5.6）：O1（内存模型对拍（HashMap 理想文件系统））-对拍在流二第一次 `publish` 的观测里把那一版文件内容的末字节翻一位，对拍判出一处；O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官在每条流批 1 第一个「实现读回了文件」的态把读回内容末字节翻一位，每条流各一处、合计 2 处（登记字面是「第一个」，装置按每条流各翻一次，对照要的「红」没变）；O2（独立解析器 + checker） 在流一全持久态上叠已知坏镜像族第一条，1 处。基线一趟与空变异一趟各跑一遍，红都一路带到 `oracle-verdicts.tsv` 与 `call-outcomes.tsv`（比对器的 `name=positive_control` 六行）。O3（独立规约执行器） 的阳性对照（第二段起）：流一全持久态叠 P1 补丁交冻结的执行器，基线与空变异两趟各判红 1 处（s4 产物 `name=positive_control oracle=O3（独立规约执行器）` 两行）。翻过的那次对拍让模型停在那次调用，此后的调用不比、写落在它之后的态 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官记「不适用」，所以 O1（内存模型对拍（HashMap 理想文件系统））-对拍那一趟的 `o1_crash_judge_red_states` 是 0。
- 停机条款：s1 本地常量与 crates 回比 42 条全过；s2 两条流的八个文件与 `impl-verdicts.tsv`（去掉耗时列）逐字节等于 E170（O3规约执行器第一版） 导出装置在同一棵树上 `dump` 写出的；s3 O2（独立解析器 + checker） 直接调 `check_pool_image` 与 `judge_crash_image` 内那一遍判红的不变量集合在基线每个取样态上相同（`o2_matches_judgement_false=0`），在 76 行坏状态上与 E170（O3规约执行器第一版） `bad-impl-verdicts.tsv` 池级 checker 那一列逐行相同；s4 已知坏镜像族 41 条、条目名与原文件逐条相同、干净镜像摘要等于 `build_pool` 的、补丁叠回与直接改坏判红集合相等、目标不变量都红；s5 比对器自己现算的两个数（批 1 的态数与序号集合、O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官适用的态数）与装置的计数相等；s6 没触发：E170（O3规约执行器第一版） 导出装置在 2026-10-03 的树上 `dump` 退 0、36 条 `holds=true`、0 条 `holds=false`；s7 在第一段触发（O3（独立规约执行器） 列不跑），第二段起 O3（独立规约执行器） 用登记修订 2 抄下冻结行的那一版，s4 比对器开头与收尾两次冻结核对都是 0 处不同；s8 的三处调用在「装置、研究一侧与变异表」一节给了行号。逐行那一趟的 S5（登记修订 3、4）在 s4 117 行上 0 行不过：判决行 `rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0`，`name=q1_denominator … device_stop=0`。
- 被测的是跑那一刻的树：s4 的空变异、117 行与 Q3-核三趟从同一份冻住的树拍快照（五趟驱动的 `snapshot_digest` 只有一种，`d15864653cd83cedddc4902b6598d5db9cf590a62e6cbb8a54bed6661e4700cb`）；开跑前树核对拿它与第六段读码时的清单逐文件比，只差一个别的 bin（`e168_delete_ring_window_borrow.rs`，不挡）。第一段的基线、对照与试跑用的是 `crates/` 在跑那一刻的状态；E170（O3规约执行器第一版） 导出装置一侧用的是 `git show :crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs` 取出的暂存版本（sha256 `8c526293…`，登记第三节钉的那一份）在草稿目录里编出来的，它的 `dump` 与 `bad-states` 输出与工作区里 E170（O3规约执行器第一版） 第 1 次重跑改过的版本逐字节相同（`cmp` 无输出），所以复跑驱动直接用工作区里的版本。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:405-410`（整段抄，未转述）**

```markdown
### 单测与变异

- Python 一侧单测 110 条（研究一侧目录里 `grep -c '    def test_' test_*.py` 四个文件 32 + 29 + 24 + 25），`python3 -m unittest test_collusion_matrix test_collusion_pipeline test_o3_columns test_o3_comparison` 连跑 5 轮每轮 `Ran 110 tests` / `OK`（手造数据、确定性，5 轮只说明没有隐藏状态）：矩阵（触达三样、O3（独立规约执行器） 两列的读法、全放行谓词、关一条、门槛、轨迹）、切段与取样公式现算、导出目录读回、坏实现表（锚点、补位、空变异）、挑法脚本的行数、冻结核对、坏镜像的 O3（独立规约执行器） 判法、逐行驱动的编译记录与重编核对（真 `rsync -a` 与 `copy2`、假 cargo）、F4 扩表（挑法脚本两次的差集与编号、比对器的逐行核对、两个输出根的合并与重号拒绝、分母按表分开、补位跳过两张表的名字）、编不过之后的补位（手造组 r0–r5、表里有 r0 与 r2、装置行插在中间：编不过依次换到 r1、r3、r4，第四次补不上记编不过；换上的行锚点不过与编不过合算三次；空变异不补位；组里没有装置行；驱动在同一个行目录里重编换上的行、`compile-failed-1.log` 与 `failed_attempt_1` 同号、复跑开关不补位；比对器只拿补位行换掉编不过的同号行、别的情形拒、补位行的结果行写换上的名字）。每个会被变异弄坏的谓词都配了钉绝对值的断言（例：扩表在手造表上必须是 `(118, a4)`、`(119, a12)`、`(120, b2)` 三行；补位的整行 `name=substitution row=3 … substituted_by=c_换 failed_attempts=1 …` 写死）。
- Python 一侧变异 67 条，表 `scripts/mutation/tables/e171_three_oracle_collusion.tsv`，跑法 `mutation_driver.py`（登记第九节写「经 `scripts/mutation/mutate.sh` 跑」，那条脚本只起 cargo test，起不了 Python 的单测，登记修订 1 第 1 行记了这一处）：`e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out` 里 `E7RESULT name=verdict mutations=67 caught=67 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`，抓到 / 无效 / 没红 = 67 / 0 / 0（结局符号逐个数：✅ 67、⏭ 0、❌ 0、⏱ 0、💥 0、⚠️ 0、🧱 0）。x60–x74 是编不过补位那一批（编不过不补位、不计次数、组里算进装置行、换上的行锚点不过时结局不记编不过、补位那一趟领了不是编不过的行、不记试过的名字、编不过之后不换行直接跑装置、比对器不核原行是编不过、不核补位自的名字、结果行写原名、合计把补不上的算换上、行分析不套补位、日志与试过的名字不同号、复跑开关不起作用、补位那一趟同一行在两个根里不拒）。同名的 `…-r6.out` 是同一批的前 13 条加进来时的全表跑（65 条，65 / 0 / 0），之后补了 x73、x74，留档。按「没红的三类」想过而没加的一条：编不过之后的补位从表里原名往下找、不从编不过的那一行往下找——补位跳过表里与已用过的名字，原名与编不过那一行之间的每个名字都在这两样里，两种找法在所有输入上同值，是等价变异，没加进表，等价性写成一条留档测试（`test_walking_from_the_name_in_the_table_or_from_the_last_failed_name_gives_the_same_substitute`，手造组上逐个起点核）；加了这条测试之后变异表重跑一遍，输出与 `-r7.out` 逐字节相同。x52–x59 是 F4 扩表那一批（扩表不去掉重叠行、从 117 编号、把装置自己的变异行算进挑法、两个输出根同号不拒、补位不跳过别的表、扩表核对只比行数、分母按表对调、只读第一个输出根），每条都由 `F4ExpansionTests` 里对应的那条单测抓到。同名的 `…-r4.out` 是同一张表的一次中途被停的跑（外面套的 `timeout 230` 在第 8 条变异时停了它，61 行、没有判决行），不算数、留档。更早的 `…-s1-comparator-mutations.out`、`…-s2-comparator-mutations.out`、`…-s3-comparator-mutations.out` 与 `…-s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` 是表在 9、26、37、40、44、44 条时的全表跑，各自全抓。
- 装置内联单测 31 条（`grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`）：编过（`scripts/hosts/compile-then-swap.py` 的 `cargo build` 与 `cargo test --no-run`），没跑（checker 档，归提交）。装置一侧变异 15 行追加在 `crates/mutations.tsv` 末尾（名字以「E171（三oracle合谋测试） 装置」起头），三个数待提交时门禁 checker-tier-crates-mutation-replay。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:411-855`（整段抄，未转述）**

````markdown
### 第 1 次重跑第一段·修订 6 那一趟（已跑：第一段的量跑完，Q5 告警自证已过、判甲；Q2 的集合与 Q3 的分母在第二段那一节）

登记 `research/prompts/e171-r1-prereg.md`（修订 6、7 与其后各条「主 agent 认定」，最后一条是「`-resume2` 跨机一致对照 S17 停机的处置」）；问题单 `e171-r1-questions.md` 的 Q1–Q5。树是修订 6 树 `scripts/experiment-apparatus/target/e171-r1-frozen-tree-revision-6/`（快照摘要 `7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4`，284 个文件；第一段新树只换装置，不带第 31 批的 core 断言），装置 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 在树里那一份 sha256 `67e7a549…`。第 0–10 步后缀 `-r6-r3`，第 11、12 步（16 行）多机跑、后缀 `-resume2`，第 13 步没起（超时 0 行）。O3（独立规约执行器） 是 E170（O3规约执行器第一版） 第 2 次重跑的 v2-487（登记修订 1）。修订 6 那一趟的判定只属于修订 6 树；修订 8 那一趟（最终主线）还没跑，这里一个数都不推到它上面。比对器只读这一趟的产物前缀 `e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-`、后缀 `-r6-r3.out` 与导出根 `scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/`，给到别的前缀就退出（登记修订 8 第 8.5 节第 1 条）。

| 行 | 判定（修订 6 树） | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| q1 | 甲：七行在这一批三条流上每行至少一条红（批 1 定判，严格与暂代两种 O3（独立规约执行器） 读法都是甲；七行都触达、三条流都跑完，无效行 0） | `…-s1-comparator-resume2-r6-r3.out` 的 `name=q1 reading=strict streams=three_streams batch=1 … verdict=jia` | 红在哪一格：第 15 行 O2（独立解析器 + checker），三条流，I-5.5（从 mkfs 水位建树那一版的 t1 落点）、第 23 行 O2（独立解析器 + checker），只在流三，I-1.1（块头自述逻辑地址）、I-9.4（容器号不超最小 key）、I-9.12（分隔 key 落在孩子区间之外）、第 42 行 O1（内存模型对拍（HashMap 理想文件系统））-对拍（流二第 14 次调用 `unmount`，`disagrees:根带的 F`）、第 242、269、270 行 O3（独立规约执行器），严格、暂代同数、第 271 行 O2（独立解析器 + checker），三条流，I-9.27（树表条目的头 ID 取值）。只取流一、流二（与 s4 同一批历史的口径）时第 23 行三列全不红（`streams=two_streams … all_quiet_rows=23 verdict=yi`）：第 23 行只靠流三那段跨叶的历史才被抓。批 2 的判定同是甲，6 格在两批之间翻面（「q1 逐格（修订 6 那一趟）」一节）。O1（内存模型对拍（HashMap 理想文件系统）） 的两个崩溃子列在七行上一态都没红 |
| Q2 | 第一段只报七行（描述量）；第二段整表判乙（「第 1 次重跑第二段·修订 6 那一趟」一节） | `name=q2_named_row` 14 行（整行抄在「Q2 七行、对照与 Z1（修订 6 那一趟）」一节） | 七行在流一、流二上都未触达；第 155、161 行各 1 次调用交回值不同、胶水交给模型的观测也不同、O1（内存模型对拍（HashMap 理想文件系统））-对拍红 1 次；第 49、110、162、174、252 行交回值或内存对象不同而观测相同、O1（内存模型对拍（HashMap 理想文件系统））-对拍不红 |
| q3 | 甲：第 125 行计入分母（c = 12，触达） | `name=q3 row=125 call_panic=true judged_calls_before_the_first_panic=12 … touched=true verdict=jia` | panic 在流二第 14 次调用 `unmount`（`attempt to subtract with overflow`），之前第 2–13 次调用 O1（内存模型对拍（HashMap 理想文件系统））-对拍都 `agrees`；流一没有 panic，比对器只数有 panic 的那条流（把流一那 1 次 `agrees` 也算上是 13，两种读法都 ≥ 1、判定不变）。分母 D = 39 出自第二段整表（「第 1 次重跑第二段·修订 6 那一趟」一节），第 125 行在那里同样计入 |
| Q4 | 基线甲（（ii）0 态）；第 242 行乙（（ii）12 态，第二截全判不合法）；第 269 行乙（（ii）58 态，全判不合法） | `name=q4_trip` 三行（整行抄在「第 1 次重跑第一段·修订 6 那一趟」一节判决行之后的代码块） | 三趟 × 三条流 × 两批 18 格每格都探过（`cells_without_a_probe=0`）。Q4-对账两数 18 格都是 0（`witness_refusals_without_either_branch=0`、`counter_branch_mounted=0`），S11 不触发。（i）判不出那一支另报：基线 1 340 态、第 242 行 2 650 态、第 269 行 1 340 态（两批合计，`undecidable_branch=` 之和），不进候选。（ii）的逐态清单与按段、按建 inode 排开的轨迹在「Q4-（ii）逐态清单与 Q4、Q5 轨迹（修订 6 那一趟）」一节 |
| Q5 | 甲，告警自证已过（主取样点与反向取样点上 RefFS 红的态数都等于 L，模型崩溃判官 0 态红，两点同判） | `name=q5_judged` 两行（整行抄在「第 1 次重跑第一段·修订 6 那一趟」一节判决行之后的代码块）；告警自证见「作数的条件与 Q5（修订 6 那一趟）」一节末条 | 修订 5 第 5.6 节第 7 条与修订 6 第 6.6 节要的告警自证（U1–U9、U13–U16 与原有的两条内联用例绿，`crates/mutations.tsv` 里 V17–V31、V33–V36 全抓；U10–U12、V32 不进这一条）由主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 提前跑过，作数（登记末节「主 agent 认定：第一段（修订 6 树）Q5 告警自证」）。主取样点的 L 是按等价类推出的（`derived_by=equivalence_classes judged_states=144`），等价关系只在 k = 6 的 4 608 态上逐态核过（`mismatching_states=0`），其余态按类核对 ③ 与修订 5 甲推（修订 5 第 5.2 节）。按末段落了几个写排开的轨迹在「Q4-（ii）逐态清单与 Q4、Q5 轨迹（修订 6 那一趟）」一节 |

比对器的判决行（`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out` 的 `name=verdict` 那一行；补了逐态清单与轨迹之后的 `…-s1-comparator-resume2-r6-r3-2026-10-07.out` 的 `name=verdict` 那一行与它逐字节相同，这一节抄的判决行与 Q1、Q3、Q4、Q5、阴性、覆盖各行在两份里都有）：

```
E7RESULT name=verdict q1=jia q3=jia q4_baseline=jia q4_row_242=yi q4_row_269=yi q5_main=jia q5_reverse=jia q5_same_at_both_sampling_points=true a1_baseline_red=0 a8_failing_controls=0 a11_failing_streams=0 z1_differing_files=0 f7=false a1_empty_mutation_red=0 empty_mutation_touched=false
```

同一份产物的 Q1、Q3、Q4、Q5 与阴性、覆盖各行（整行抄）：

```
E7RESULT name=a1 batch=1 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1 batch=2 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=o3_third_stream_coverage stream=first_file_unlinked_then_inodes_across_one_leaf sampled_states=6459 strict_in_or_not_in_states=6459 f7=false
E7RESULT name=a11 stream=new_pool_file_creation states=82 crash_judge_applicable_device=82 crash_judge_applicable_recomputed=82 reference_filesystem_voting_device=82 reference_filesystem_voting_recomputed=82 holds=true
E7RESULT name=a11 stream=fixed_script_through_e_then_unmount states=890 crash_judge_applicable_device=890 crash_judge_applicable_recomputed=890 reference_filesystem_voting_device=890 reference_filesystem_voting_recomputed=890 holds=true
E7RESULT name=a11 stream=first_file_unlinked_then_inodes_across_one_leaf states=11695 crash_judge_applicable_device=11695 crash_judge_applicable_recomputed=11695 reference_filesystem_voting_device=81 reference_filesystem_voting_recomputed=81 holds=true
E7RESULT name=a1_empty_mutation batch=1 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1_empty_mutation batch=2 o1_comparison_red_calls=0 O1-模型崩溃判官=0 O1-RefFS=0 O2=0 O3甲=0 O3乙=0 o3_ran=true
E7RESULT name=a1_empty_mutation_touch row=0 streams_present=3 touched=false
E7RESULT name=q1 reading=strict streams=three_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=strict streams=three_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=strict streams=two_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=strict streams=two_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=provisional streams=three_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=provisional streams=three_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=- verdict=jia
E7RESULT name=q1 reading=provisional streams=two_streams batch=1 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q1 reading=provisional streams=two_streams batch=2 rows=7 invalid_rows=- all_quiet_rows=23 verdict=yi
E7RESULT name=q3 row=125 call_panic=true judged_calls_before_the_first_panic=12 states_judged=514 recovery_panic_states=0 touched=true verdict=jia
E7RESULT name=q4_trip trip=baseline counter_branch=0 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=jia
E7RESULT name=q4_trip trip=row_242 counter_branch=12 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=yi
E7RESULT name=q4_trip trip=row_269 counter_branch=58 counter_branch_judged_legal=0 cells_without_a_probe=0 verdict=yi
E7RESULT name=q5_judged sampling_point=main lost=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 q5_negative_o1_red=0 a10_states_equal_the_product=true reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 verdict=jia
E7RESULT name=q5_judged sampling_point=reverse lost=9 reference_filesystem_red=9 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 q5_negative_o1_red=0 a10_states_equal_the_product=true reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 verdict=jia
```

#### Q1 逐格（修订 6 那一趟）

批 1 每行每条流红的列（O1（内存模型对拍（HashMap 理想文件系统））-对拍按调用，其余按态；括号里是轨迹：峰值、为正段数、期末值、单点红的段数）。在仓根跑的命令（只读 `name=q1_row … batch=1` 那 21 行，打出非 0 的列）：

```
grep 'name=q1_row .* batch=1 ' e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out | awk '{o=""; for(i=1;i<=NF;i++){split($i,a,"="); f[a[1]]=a[2]} if (f["o1_comparison_red_calls"]!="0") o=o" O1-对拍="f["o1_comparison_red_calls"]; n=split("O1-模型崩溃判官 O1-RefFS O2 O3甲 O3乙",c," "); for(j=1;j<=n;j++){k=c[j]; if (f[k"_red"]!="0") o=o" "k"="f[k"_red"]"(峰"f[k"_peak"]",正段"f[k"_positive_segments"]",末"f[k"_last"]",单点"f[k"_single_points"]")"} print f["row"], substr(f["stream"],1,9) (o==""?" 全不红":o)}'
```

原样输出：

```
15 new_pool_ O2=9(峰8,正段2,末1,单点1)
15 fixed_scr O2=240(峰8,正段47,末0,单点11)
15 first_fil O2=480(峰8,正段96,末0,单点24)
23 new_pool_ 全不红
23 fixed_scr 全不红
23 first_fil O2=29(峰8,正段6,末1,单点2)
42 new_pool_ 全不红
42 fixed_scr O1-对拍=1
42 first_fil 全不红
242 new_pool_ O3甲=4(峰4,正段1,末0,单点0) O3乙=4(峰4,正段1,末0,单点0)
242 fixed_scr O3甲=8(峰4,正段2,末0,单点0) O3乙=8(峰4,正段2,末0,单点0)
242 first_fil O3甲=4(峰4,正段1,末0,单点0) O3乙=4(峰4,正段1,末0,单点0)
269 new_pool_ O3甲=8(峰4,正段3,末0,单点1) O3乙=8(峰4,正段3,末0,单点1)
269 fixed_scr O3甲=41(峰8,正段10,末0,单点3) O3乙=41(峰8,正段10,末0,单点3)
269 first_fil O3甲=8(峰4,正段3,末0,单点1) O3乙=8(峰4,正段3,末0,单点1)
270 new_pool_ O3甲=3(峰3,正段1,末0,单点0) O3乙=3(峰3,正段1,末0,单点0)
270 fixed_scr O3甲=30(峰3,正段10,末0,单点0) O3乙=30(峰3,正段10,末0,单点0)
270 first_fil O3甲=714(峰3,正段238,末0,单点0) O3乙=714(峰3,正段238,末0,单点0)
271 new_pool_ O2=9(峰8,正段2,末1,单点1)
271 fixed_scr O2=353(峰8,正段69,末1,单点17)
271 first_fil O2=4749(峰8,正段950,末1,单点238)
```

- 对着登记读法 12（C673（三 oracle 合谋测试的补 oracle 改法没做） 改法落点）：第 15 行 ← ⑥ O2（独立解析器 + checker） 新池新建文件落点，O2（独立解析器 + checker） 红（I-5.5（从 mkfs 水位建树那一版的 t1 落点））；第 23 行 ← ⑦ 流三，O2（独立解析器 + checker） 只在流三红（I-1.1（块头自述逻辑地址）、I-9.4（容器号不超最小 key）、I-9.12（分隔 key 落在孩子区间之外））；第 42 行 ← ④ 模型的卸载答，O1（内存模型对拍（HashMap 理想文件系统））-对拍在流二那次 `unmount` 上红；第 242、269 行 ← ② O3（独立规约执行器） 谓词 W，O3（独立规约执行器） 红；第 270 行 ← ③ O3（独立规约执行器） 见证值不低于 tail，O3（独立规约执行器） 红；第 271 行 ← ⑤ O2（独立解析器 + checker） 树表条目头 ID，O2（独立解析器 + checker） 红（I-9.27（树表条目的头 ID 取值））。七个落点在这一批上都红了。O2（独立解析器 + checker） 红的不变量名取自各行导出 `oracle-verdicts.tsv` 第 8、9 列（两批合计）。
- 批 1 对批 2（第八节几何敏感性）：命令 `grep 'name=q1_row' <同一份产物> | awk '{for(i=1;i<=NF;i++){split($i,a,"="); f[a[1]]=a[2]} n=split("o1_comparison_red_calls O1（内存模型对拍（HashMap 理想文件系统））-模型崩溃判官_red O1（内存模型对拍（HashMap 理想文件系统））-RefFS_red O2_red O3（独立规约执行器）甲_red O3（独立规约执行器）乙_red",c," "); for(j=1;j<=n;j++){k=f["row"]" "substr(f["stream"],1,9)" "c[j]; r[k, f["batch"]]=(f[c[j]]!="0"); keys[k]=1}} END{t=0; m=0; for(k in keys){t++; if (r[k,1]!=r[k,2]) {m++; print k, "batch1_red=" r[k,1], "batch2_red=" r[k,2]}} print "cells=" t, "flipped_between_batch_1_and_2=" m}' | sort`，输出末行 `cells=126 flipped_between_batch_1_and_2=6`，翻面的 6 格是第 15、271 行流一的 O2（独立解析器 + checker），第 269 行流一与流三的 O3（独立规约执行器） 严格、暂代——这 6 格记「不稳定」（批 2 没取到红的那几段）；各行在批 2 上照样有别的格红，Q1 批 2 仍判甲。
- 第 23、42 行的红只在一条流、一种 oracle 上；第 42 行只有 1 次调用（O1（内存模型对拍（HashMap 理想文件系统））-对拍按调用算，没有轨迹）。
- 三份逐行产物里第 252、271 行的 `line_matches_the_registration=false`（行文的 sha256 与登记 5.9 不同，照冻结树里那一行跑、另记），其余十四行是 `true`；`substituted_from=-`、`failed_attempts=0`，没有补位。

#### Q2 七行、对照与 Z1（修订 6 那一趟）

```
E7RESULT name=q2_named_row row=49 streams=streams_1_2 touched=false calls_with_a_difference=11 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=11 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=49 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=110 streams=streams_1_2 touched=false calls_with_a_difference=14 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=14 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=110 streams=stream_3 touched=false calls_with_a_difference=238 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=238 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=155 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=1
E7RESULT name=q2_named_row row=155 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=161 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=1 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=1
E7RESULT name=q2_named_row row=161 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=162 streams=streams_1_2 touched=false calls_with_a_difference=11 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=11 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=162 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=174 streams=streams_1_2 touched=false calls_with_a_difference=1 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=1 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=174 streams=stream_3 touched=false calls_with_a_difference=0 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=0 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=252 streams=streams_1_2 touched=false calls_with_a_difference=14 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=14 o1_comparison_red_calls=0
E7RESULT name=q2_named_row row=252 streams=stream_3 touched=false calls_with_a_difference=238 calls_whose_observation_differs=0 calls_whose_observation_is_the_same=238 o1_comparison_red_calls=0
E7RESULT name=a8 control=O1-对拍 products=1 counted=o1_comparison_red_calls expected=1 observed=1 holds=true
E7RESULT name=a8 control=O1-对拍-卸载 products=1 counted=o1_comparison_red_calls expected=1 observed=1 holds=true
E7RESULT name=a8 control=O1-崩溃判官 products=1 counted=o1_crash_judge_red_states expected=3 observed=3 holds=true
E7RESULT name=a8 control=O2 products=1 counted=o2_red_states expected=1 observed=1 holds=true
E7RESULT name=a8 control=O3 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=RefFS products=1 counted=o1_reference_filesystem_red_states expected=3 observed=3 holds=true
E7RESULT name=a8 control=panic products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=交回值 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=内存对象 products=1 checked=false why=judged_from_the_call_lines_by_hand
E7RESULT name=a8 control=见证 products=1 counted=counter_branch_states expected=1 observed=1 holds=true
E7RESULT name=z1_unstable_memory_objects names=-
```

- 每次调用逐行（`name=q2_named_row_call`，哪几个顶层字段、哪个内存对象不同）在同一份产物里，529 行，不抄；流三上第 110、252 行 238 次调用里都是写数（`writes`）与 `current_version` 不同。
- a8 那 4 样 `checked=false` 的（O3（独立规约执行器）、panic、交回值、内存对象）由按序跑法第 3 步自己的判法判过（`…-s1-positive-control-o3-p1-r6-r3.out` 的 `o3_control_verdict strict=not_in base_digest_matches=true red_strict=true`，交回值、内存对象的调用比对与 panic 截断各一核）；空变异那一趟十项（`…-s1-empty-control-*-driver-r6-r3.out`）同由第 4 步判，比对器不读它们。
- Z1：39 行 `name=z1` 全是 `identical=true`（`grep -c 'name=z1 .*identical=true'` 与 `grep -c 'name=z1 '` 都是 39），基线两次之间没有不稳的内存对象（`names=-`）。

#### Q4-（ii）逐态清单与 Q4、Q5 轨迹（修订 6 那一趟）

登记第六节 Q4-（ii）要的逐态清单与第八节 Q4、Q5 的轨迹由比对器补出，在 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out`（811 行）里。与 `…-s1-comparator-resume2-r6-r3.out`（679 行）比：前 677 行与判决行（新那份第 810 行）逐字节相同，只在 Q5 判定之后、判决行之前多出 132 行（`name=q4_counter_branch_state` 70、`name=q4_trajectory` 18、`name=q4_trajectory_by_inode_creation` 6、`name=q5_trajectory_cell` 36、`name=q5_trajectory` 2），收尾行由 `E7RESULT name=done emitted=679` 变成 `E7RESULT name=done emitted=811`（`diff` 只报 `677a678,809` 与 `679c811`）。轨迹三样（峰值与它第一次出现的格、为正的格数、期末值）照「端点不是轨迹」报；期末值是格号最大的那一格，Q4 按段排开时是这一批取到的最后一段，按建 inode 排开时是第 235 次建 inode，Q5 是末段全落那一格。

逐态清单按格归并（列的态：趟、流、批、段、这一段由第几次调用发出、计数那一支是哪一句、挂载结局、第二截判定；每一态另有序号、状态号、落地串的 sha256、镜像摘要、c_见证、所选版本、拒的成员、单子上的理由与没满足的条件，在产物里）：

```
grep 'name=q4_counter_branch_state ' <新那份> | awk '{o=""; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="trip"||a[1]=="stream"||a[1]=="batch"||a[1]=="segment"||a[1]=="call_of_the_segment"||a[1]=="counter_sentence"||a[1]=="mount"||a[1]=="second_crop") o=o" "$i}; print o}' | sort | uniq -c | sort -k2,2 -k3,3 -k4,4
      2  trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 segment=14 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      2  trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 segment=14 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_242 stream=new_pool_file_creation batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_242 stream=new_pool_file_creation batch=2 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=18 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=21 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=15 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      2  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      3  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=17 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      3  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=19 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      8  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=16 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      8  trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 segment=20 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      7  trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 segment=16 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      7  trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 segment=20 call_of_the_segment=4 counter_sentence=AboveTheLastRecordOfTheSelectedVersion mount=refused second_crop=illegal
      1  trip=row_269 stream=new_pool_file_creation batch=1 segment=1 call_of_the_segment=1 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      1  trip=row_269 stream=new_pool_file_creation batch=1 segment=2 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
      2  trip=row_269 stream=new_pool_file_creation batch=1 segment=0 call_of_the_segment=0 counter_sentence=NeitherRecordReadable mount=refused second_crop=illegal
      2  trip=row_269 stream=new_pool_file_creation batch=1 segment=1 call_of_the_segment=1 counter_sentence=RecordAtTheWitnessedCounterIsNewer mount=refused second_crop=illegal
```

三条流的第 0、1 次调用是取号、暖机，流二第 4 次调用是可写挂载（各行 `calls.tsv` 第 2 列）。没满足的条件按计数那一支一一对上：`RecordAtTheWitnessedCounterIsNewer` 对 `witness_criterion_true_without_read_faults:RecordAtTheWitnessedCounterNewerThanTheSelectedVersion`，`AboveTheLastRecordOfTheSelectedVersion` 对 `…:WitnessedCounterAboveTheLastRecordOfTheSelectedVersion`，`NeitherRecordReadable` 对 `…:NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable`。每格列出的态数与同一格 `name=q4` 行的 `counter_branch=` 相等（18 格，0 格不等），轨迹各格之和也与它相等（18 格，0 格不等）。

按段排开（第 242、269 行；基线 6 行都是 0）：

```
E7RESULT name=q4_trajectory trip=row_242 stream=new_pool_file_creation batch=1 cells=segments segments=9 first_segment=0 last_segment=8 counter_branch_by_cell=2:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=new_pool_file_creation batch=2 cells=segments segments=4 first_segment=2 last_segment=7 counter_branch_by_cell=2:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=fixed_script_through_e_then_unmount batch=1 cells=segments segments=60 first_segment=0 last_segment=59 counter_branch_by_cell=2:2,14:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=fixed_script_through_e_then_unmount batch=2 cells=segments segments=36 first_segment=2 last_segment=58 counter_branch_by_cell=2:1,14:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 cells=segments segments=720 first_segment=0 last_segment=719 counter_branch_by_cell=2:2 counter_branch_peak=2 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 cells=segments segments=478 first_segment=2 last_segment=718 counter_branch_by_cell=2:1 counter_branch_peak=1 counter_branch_peak_at=2 counter_branch_positive_cells=1 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=new_pool_file_creation batch=1 cells=segments segments=12 first_segment=0 last_segment=11 counter_branch_by_cell=0:2,1:3,2:1 counter_branch_peak=3 counter_branch_peak_at=1 counter_branch_positive_cells=3 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=new_pool_file_creation batch=2 cells=segments segments=1 first_segment=7 last_segment=7 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=fixed_script_through_e_then_unmount batch=1 cells=segments segments=79 first_segment=0 last_segment=78 counter_branch_by_cell=0:2,1:3,2:1,15:2,16:8,17:3,18:1,19:3,20:8,21:1 counter_branch_peak=8 counter_branch_peak_at=16 counter_branch_positive_cells=10 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=fixed_script_through_e_then_unmount batch=2 cells=segments segments=17 first_segment=7 last_segment=74 counter_branch_by_cell=16:7,20:7 counter_branch_peak=7 counter_branch_peak_at=16 counter_branch_positive_cells=2 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 cells=segments segments=960 first_segment=0 last_segment=959 counter_branch_by_cell=0:2,1:3,2:1 counter_branch_peak=3 counter_branch_peak_at=1 counter_branch_positive_cells=3 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 cells=segments segments=238 first_segment=7 last_segment=955 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
```

流三按第几次建 inode 排开（235 次建 inode 各一格；段不在建 inode 调用里的（ii）态记 `counter_branch_outside_create_inode_calls`，全持久态那一段记 `counter_branch_after_every_call`）：

```
E7RESULT name=q4_trajectory_by_inode_creation trip=baseline stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=baseline stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=2 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_242 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=1 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=1 create_inode_calls=235 counter_branch_outside_create_inode_calls=6 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
E7RESULT name=q4_trajectory_by_inode_creation trip=row_269 stream=first_file_unlinked_then_inodes_across_one_leaf batch=2 create_inode_calls=235 counter_branch_outside_create_inode_calls=0 judged_legal_outside_create_inode_calls=0 counter_branch_after_every_call=0 judged_legal_after_every_call=0 counter_branch_by_cell=- counter_branch_peak=0 counter_branch_peak_at=- counter_branch_positive_cells=0 counter_branch_last=0 judged_legal_by_cell=- judged_legal_peak=0 judged_legal_peak_at=- judged_legal_positive_cells=0 judged_legal_last=0
```

Q5 按末段里落了几个写排开（主取样点 30 格、反向取样点 6 格；装置的 `name=q5_trajectory` 行只出有 L 态的格，比对器补成第 0 格到末段写数那一格、没有的记 0，各格之和与 `name=q5` 行的 L、两个红数核过）：

```
E7RESULT name=q5_trajectory sampling_point=main cells=30 last_segment_writes=29 cells_add_up_to_the_q5_line=true reference_filesystem_red_by_cell=0:1,1:28,2:376,3:3224,4:19826,5:93104,6:347116,7:1054504,8:2658271,9:5633804,10:10133156,11:15571184,12:20533756,13:23297344,14:22762456,15:19137104,16:13810511,17:8518004,18:4460896,19:1965304,20:719026,21:214544,22:50876,23:9224,24:1201,25:100,26:4 reference_filesystem_red_peak=23297344 reference_filesystem_red_peak_at=13 reference_filesystem_red_positive_cells=27 reference_filesystem_red_last=0 model_crash_judge_red_by_cell=- model_crash_judge_red_peak=0 model_crash_judge_red_peak_at=- model_crash_judge_red_positive_cells=0 model_crash_judge_red_last=0
E7RESULT name=q5_trajectory sampling_point=reverse cells=6 last_segment_writes=5 cells_add_up_to_the_q5_line=true reference_filesystem_red_by_cell=0:1,1:4,2:4 reference_filesystem_red_peak=4 reference_filesystem_red_peak_at=1 reference_filesystem_red_positive_cells=3 reference_filesystem_red_last=0 model_crash_judge_red_by_cell=- model_crash_judge_red_peak=0 model_crash_judge_red_peak_at=- model_crash_judge_red_positive_cells=0 model_crash_judge_red_last=0
E7RESULT name=q5_trajectory_cell sampling_point=main writes_landed_in_the_last_segment=13 lost_states=23297344 reference_filesystem_red=23297344 model_crash_judge_red=0
E7RESULT name=q5_trajectory_cell sampling_point=main writes_landed_in_the_last_segment=29 lost_states=0 reference_filesystem_red=0 model_crash_judge_red=0
E7RESULT name=q5_trajectory_cell sampling_point=reverse writes_landed_in_the_last_segment=5 lost_states=0 reference_filesystem_red=0 model_crash_judge_red=0
```

读法（只描述，不改任何判定）：
- Q4 的（ii）态在两行上都集中在取号、暖机那两次调用的段（段 0–2）与流二可写挂载那次调用的段（第 242 行段 14，第 269 行段 15–21）；三趟每条流每批的期末值都是 0，第二截判合法的态在每一格都是 0。第 269 行流二批 1 的峰值 8 在段 16（段 20 同为 8），为正段数 10。
- 流三的（ii）态全落在建 inode 之外的调用里（第 242 行批 1 2 态、批 2 1 态，第 269 行批 1 6 态），235 次建 inode 每一次 0 态：这一批上看不出（ii）随叶容器分裂变多变少，按建 inode 排开的那一格读数是 0，不是「分裂前后都有」。
- Q5 的 RefFS 红态数在每一个有 L 态的格上都等于那一格的 L 态数（主取样点 27 格、反向取样点 3 格），模型崩溃判官在每一格都是 0；末格（末段全落）的期末值是 0，因为那一格没有 L 态（`lost_states=0`），不是 RefFS 在那一格判绿。
- `q5_verdict` 在 L 个态上没有一态红、也没有一态两者都绿时报的结局由 `not_enough` 改成 `not_enough_f8`（登记修订 5 第 5.1 节第 7 条：照 F8 记不够判）；这一趟两个取样点都有红、到不了那一支，`q5_main=jia q5_reverse=jia` 不变。

#### 作数的条件与 Q5（修订 6 那一趟）

Q1–Q4 作数要第十一节原文加 S15 ①②③⑤⑥ 都不触发（修订 6 第 6.6 节）；Q5 另要主取样点两次 S1 全过、`s13=false`、阳性对照「判态路 panic 单列」恰为精确数、两次逐字节相同（Z7），反向取样点 S1 全过、S15 ④ 不触发，再加告警自证（已过，「作数的条件与 Q5（修订 6 那一趟）」一节末条）。整行抄（文件名省前缀 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-`）：

```
frozen-tree-r6-r3.out:61: E7RESULT name=verdict s15_1_extra_files_against_the_first_segment_tree=0 s15_1_extra_files_against_the_q5_tree=0 s15_2_holds=true s15_3_unassigned_hunks=0 s15_3_hunks=24 s15_3_table_rows=24 a12_failed_checks=0 a12_checks=12 triggered=none
stream-constants-r6-r3.out:82: E7RESULT name=verdict s1_failed_checks=0
s2-r6-r3.out:20: E7RESULT name=verdict s2_differing_files=0 s2_compared_files=18
s15-5-step-1-r6-r3.out:26: E7RESULT name=verdict s15_five_problems=0 triggered=false
s15-5-step-7-r6-r3.out:28: E7RESULT name=verdict s15_five_problems=0 triggered=false
s15-6-step-7-r6-r3.out:3: E7RESULT name=verdict s15_six_problems=0 triggered=false
s15-4-step-6-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
s15-4-step-10-run-1-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
s15-4-step-10-run-2-r6-r3.out:10: E7RESULT name=verdict s15_four_problems=0 triggered=false
baseline-run1-r6-r3.out:2035: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 o2_red_states=0 recovery_panic_states=0 probed_states=12667 counter_branch_states=0 counter_branch_states_judged_legal=0 undecidable_branch_states=1340 witness_refusals_without_either_branch=0 counter_branch_states_that_mounted=0 piece_panic_states=- unattributed_panic_states=0
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:19: E7RESULT name=q5_equivalence_pairing in_flight_writes_kept=6 states=4608 product_of_the_choices=4608 class_states=4608 classes=144 mismatching_states=0 crash_judge_not_applicable_states=0 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:23: E7RESULT name=q5_piece_panic_control states=4608 pool_checker=1152 ignored_recovery=1152 record_checker=576 other_pieces=0 consulted_recovery=0 consulted_recovery_of_the_plain_pairing=0 o1_columns_differing_from_the_plain_pairing=0 mismatching_states=0 s13=false holds=true
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:25: E7RESULT name=q5_shape sampling_point=main_before_the_first_barrier_of_call_3 removed_barriers=[50,51,55,56] root_writes_made_plain=[52] truncation=81 operations_kept=77 confirmed_root=1,3 confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:54: E7RESULT name=q5_equivalence sampling_point=main_before_the_first_barrier_of_call_3 input_list_mismatches=0 in_flight_writes_of_a_forbidden_kind=0 read_overlaps_left=0 recording_report_mismatches=0 derived_class_mismatches=0 class_landing_mismatches=0 s13=false
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:200: E7RESULT name=q5 sampling_point=main_before_the_first_barrier_of_call_3 states=1207959552 last_segment_writes=29 last_segment_tearable_writes=2 product_of_the_choices=1207959552 a10_states_equal_the_product=true lost_the_confirmed_version=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 not_lost_states=1056964608 not_lost_o1_red=0 states_without_a_root=0 derived_by=equivalence_classes judged_states=144
q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out:228: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=144 s13=false piece_panic_states=- unattributed_panic_states=0
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:19: E7RESULT name=q5_shape sampling_point=reverse_at_the_return_point_of_call_2 removed_barriers=[50,51,55,56] root_writes_made_plain=[52] truncation=57 operations_kept=53 confirmed_root=1,3 confirmed_call_returned_before_the_truncation=true in_flight_call_returned=false
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:47: E7RESULT name=q5 sampling_point=reverse_at_the_return_point_of_call_2 states=72 last_segment_writes=5 last_segment_tearable_writes=2 product_of_the_choices=72 a10_states_equal_the_product=true lost_the_confirmed_version=9 reference_filesystem_red=9 model_crash_judge_red=0 both_green=0 reference_filesystem_abstained=0 reference_filesystem_judge_panic=0 model_crash_judge_judge_panic=0 not_lost_states=63 not_lost_o1_red=0 states_without_a_root=0
q5-reverse-at-the-return-point-of-call-2-r6-r3.out:51: E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 s3_o2_judgement_mismatches=0 states=72 s13=false piece_panic_states=- unattributed_panic_states=0
```

- Z7：主取样点两次的 E7RESULT 行逐字节相同（`diff <(grep '^E7RESULT' …-run-1-r6-r3.out) <(grep '^E7RESULT' …-run-2-r6-r3.out) | wc -l` 输出 0，连耗时字段都不用去）。
- 第 11、12 步两份合成产物（`…-s1-rows-probe-driver-resume2-r6-r3.out`、`…-s1-rows-driver-resume2-r6-r3.out`）的 `freeze_check` 前后都是 `differences=0`，`snapshot_digest=7726cdd6…`，16 行 `status=ok`、`device_exit=0`、`o3_status=ran`，没有 253（第 13 步不起）。
- S17（跨机一致对照）：原产物 `…-s1-rows-cross-host-check-resume2-r6-r3.out` 判 `E7RESULT name=verdict cross_host_identical=false row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=o3`，作废，不进判定：主 agent 认定（登记末节「`-resume2` 跨机一致对照 S17 停机的处置」第 2–4 条）那一格的不同只在冻结执行器的工作进程号、工作进程数与多进程行序，是对照器缺陷，不是两台不一致。作数的是新入口重出的 `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07.out`（`E7RESULT name=verdict cross_host_identical=true row=0 host_of_step_12_row_0=1 host_of_cross_host_row_0=2 parts=4 differing_parts=none`）；`…-2026-10-07-r2.out` 是同一个入口在仓里那一份上重出、补了收尾行 `E7RESULT name=done emitted=8` 的一版，前 7 行与 `…-2026-10-07.out` 逐字节相同；入口改成读同一目录里入库的编排器之后重出的 `…-2026-10-07-r3.out` 是复跑登记版，与 `-r2` 只差第 2 行的 `entry_sha256=`。
- 告警自证已过，Q5 判甲作数：主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 提前跑（用户 2026-10-07「把能做的往前做」），单元 `e171-q5-alert-selfproof`，日志 `~/.local/state/singlefs/detached-runs/e171-q5-alert-selfproof/log`；登记末节「主 agent 认定：第一段（修订 6 树）Q5 告警自证」原样抄的读数：仓里装置 sha256 `ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df`，装置内联用例 `test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.75s`（含 U1–U9、U13–U16 与原有两条），`crates/mutations.tsv` 全部 49 行「E171R1 装置」（含 V17–V31、V33–V36）`计数：抓到 49 条、没红 0 条、无效 0 条、内存撞顶 0 条、超时 0 条、被总上限挤掉 0 条、排不上没跑 0 条、scope 起不来没跑 0 条（共 49 条）`。提交时这两样照旧在 checker 档复跑；那时仓里装置若已不是 `ab35105f…`，以那一次为准另记。

#### 比对器、复跑与入库的跑法（修订 6 那一趟）

- 比对器 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py` 相对第一段新树那一份改了八处，判据一字没动（①–⑥ 由主 agent 认，登记末节「主 agent 认定：第一段（修订 6 树）比对器交回」第 1 条；⑦ ⑧ 是那一条第 3 条点名要补的）：① 读 `oracle-verdicts.tsv` 改成 25 列（`run_directory.py`，修订 5 第 5.1 节第 5 条在末尾加了 `piece_panics`，24 列的读法在这一趟的导出上读不动）；② a11 的现算改成「最后一个落了的写所在的调用」（与逐个写取最大值同值，单测对拍），流三 11 695 态由推的十分钟上下降到整份比对器 13 秒；③ 必须给 `--dumps-root`、`--product-prefix`、`--product-suffix`，输入不在这一趟里就退 2、不写产物（修订 8 第 8.5 节第 1 条）；④ 加「流三覆盖（O3（独立规约执行器））」一行与判决行的 `f7=`（第六节、f7）；⑤ 加空变异那一行的 a1 与触达（第七节 a1 后一半）；⑥ `q5_judged` 行多带两列 `judge_panic` 的态数（修订 5 第 5.1 节第 7 条，只描述）；⑦ 补第六节 Q4-（ii）逐态清单与第八节 Q4、Q5 轨迹（只多出新行，「Q4-（ii）逐态清单与 Q4、Q5 轨迹（修订 6 那一趟）」一节）；⑧ `q5_verdict` 在 L 个态上没有一态红、也没有一态两者都绿时报 `not_enough_f8`（原来报 `not_enough`；RefFS 全弃权先走 f8 的次序不变）。
- 单测 41 条（`grep -c '    def test_' test_segment_one.py test_device_timing.py` 为 33、8），`python3 -B -m unittest test_segment_one test_device_timing` 连跑 5 轮都是 `Ran 41 tests … OK`（确定性的纯函数，5 轮只说明没有隐藏状态）。研究一侧变异表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv` 37 条（原 17 条，加 R14–R19 盯 ①–⑥，加 R20–R33 盯 ⑦ ⑧），整表 37 条全抓、0 无效、0 没红、0 超时，收尾已还原、基线仍全绿：`E7RESULT name=verdict mutations=37 caught=37 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`（`…-s1-research-mutations-2026-10-07-r3.out`）。同一天先跑的两份 `…-s1-research-mutations-2026-10-07.out`（22 条，22 / 0 / 0）、`…-2026-10-07-r2.out`（23 条，23 / 0 / 0）是加 R19、R20–R33 之前的，留档；`…-s1-research-mutations.out` 是第一段新树那一趟的。原有的 R4（Q5 全弃权不走 F8）在 ⑧ 之后只在「RefFS 全弃权而模型崩溃判官有红」那一种输入上与原式不同，单测钉了这一格（F8 排在甲之前，原有次序）。
- 多机跑法入库（拷自草稿，原草稿留着不动）：`rows_orchestrator.py`（`7bfbdb84…`）、`peer_side.py`（`27cea9e7…`）、`check_product.py`（`3f237083…`）与 `` 三个文件与草稿里的逐字节相同；`cross_host_o3_check.py`（`afc9a9b0…`，草稿那份 `04376c00…`）改成读同一目录里入库的编排器（开跑照旧核 `7bfbdb84…`），`run_segment_one_on_the_revision_6_tree_multi_host.sh`（`567a8682…`，草稿那份 `ec89ec78…`）的编排器与 `check_product.py` 改指仓里这一目录（编排器按它自己的目录取 `peer_side.py` 与 ``）；冻结树、导出根、日志与续跑清单，与没入库的 `s2_compare.py`、`s15_compare.py`、`frozen_tree_lines_revision_6.py`、`resume_check.py` 照旧在草稿 `e171-r1-segment1-rev6/` 里。仓里那一份上的自证：编排器末行 `自证 34 项，红 0`，跨机对照入口末行 `自证 38 项，红 0`；`peer_side.py` 与跑法脚本没有 `--selftest`，跑法脚本过了 `bash -n` 与 `detached-run.sh long-run-check`（长活声明三个键齐）。修订 6 那一趟实际跑的是草稿里那几份（登记里的认定按草稿的 sha256 记）。
- 复跑：`scripts/experiment-apparatus/replay.sh` 里加三行，都只在这一趟本机留存的导出与产物上重出，不起装置与 cargo：

| 驱动函数 | 产物 | 跑什么 |
|---|---|---|
| `driver_e171_r1_s1_revision_6_comparator` | `…-s1-comparator-resume2-r6-r3-2026-10-07.out` | 比对器，导出根 `scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/`；约 11 秒 |
| `driver_e171_r1_s1_revision_6_cross_host_check` | `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out` | 跨机对照入口 `compare`，补收尾行 |
| `driver_e171_r1_s1_research_mutations` | `…-s1-research-mutations-2026-10-07-r3.out` | 研究一侧 37 条变异，约 40 秒 |

  三个驱动函数照 `replay.sh` 的口径在草稿目录里跑过一遍，输出与入库产物逐字节相同、`name=done emitted=` 与 E7RESULT 行数相等（811、8、78）。跨机对照那一份 `-r3` 与 `-r2` 只差第 2 行的 `entry_sha256=`（入口自己的 sha256，`04376c00…` → `afc9a9b0…`），判决行逐字节相同；三行原来指的 `…-comparator-resume2-r6-r3.out`、`…-2026-10-07-r2.out` 两份留着。导出根不进仓，它没了前两行就判红。第 0–13 步本身（冻结树上的装置各趟、逐行驱动、O3（独立规约执行器） 执行器、两台机器的编排）推的挂钟几十小时、要第二台，不登记。

#### 它答不了的（修订 6 那一趟）

- 判定只属于修订 6 树（第一段新树换装置，第 16 批合入之后、根环线 10b 之前那一份实现，不带第 31 批 core 断言）；修订 8 那一趟换成最终主线，两趟的对照另出（登记修订 8 第 8.5 节），这里不推。
- Q1 的甲只说「七行在这一批三条流、批 1 上每行至少一条红」，不说共同盲区补上了：表里没登记的坏法与没取到的历史照样可能三条一起放过。第 23 行只在流三上被 O2（独立解析器 + checker） 抓、第 42 行只在 1 次 `unmount` 调用上被 O1（内存模型对拍（HashMap 理想文件系统））-对拍抓；流一、流二的口径下第 23 行仍三条全放行。
- Q2 第一段只有七行的描述；集合与判定出自第二段整表（流一、流二，281 行加补位，集合正是这七行、判乙）。第 49、110、162、174、252 行交回值或内存对象不同而观测相同：差在胶水没交给模型的字段上，O1（内存模型对拍（HashMap 理想文件系统））-对拍按设计看不见。
- Q3 第一段只判了第 125 行计不计入；分母 D = 39 出自第二段整表。c 只数有 panic 的那条流（12）；把没有 panic 的流一也算进来是 13，判定不变。
- Q4 的（ii）态全在第 242、269 行上、第二截全判不合法；基线（修订 6 树上的实现那一趟）（ii）0 态。（i）判不出那一支不进候选，各趟上千态。三趟都是崩溃点重放截断出的态，没注入读故障。
- Q4 的逐态清单与轨迹只在两行坏实现（第 242、269 行）上有（ii）态，流三的（ii）态全在建 inode 之外：按建 inode 排开 235 格都是 0，说不出（ii）与叶容器分裂有没有关系，只说这一批上分裂前后的建 inode 调用里没有取到（ii）态。
- Q5 的主取样点 L = 150 994 944 是按等价类推出来的（逐态判了 144 态），等价只在 k = 6 的 4 608 态上逐态核过；反向取样点 72 态逐态判。告警自证已过，两点都判甲；告警自证只证 Q5 判决路上的告警会红，不扩大等价关系逐态核过的范围。
- O3（独立规约执行器） 一列是 v2-487；在流三上严格读法 6 459 态全给出「在 / 不在」（F7 不触发）。七行里只有第 242、269、270 行 O3（独立规约执行器） 红，其余四行 O3（独立规约执行器） 一态都没红。
- 第 252、271 行跑的是冻结树里那一行的行文（`line_matches_the_registration=false`），与登记 5.9 写下时的行文不同；它们照名字取到、锚点恰 1 次、没补位。

#### 产物（第 1 次重跑第一段）

修订 6 那一趟的产物（`ls research/results | grep '^e171-r1-' | grep -- '-r6-r3'`，81 份，含补了逐态清单与轨迹的比对器产物 `…-comparator-resume2-r6-r3-2026-10-07.out` 与入口改路径之后重出的跨机对照 `…-cross-host-check-resume2-r6-r3-2026-10-07-r3.out`）与研究一侧变异表的五份产物（`grep 'research-mutations'`）。`-timing.out` 是 E7TIMING 行；`…-rows-probe-driver-r6-r3.out`、`…-rows-probe-driver-resume1-r6-r3.out` 是第 11 步前两次起跑停下留的半份（各 1 行，不进判定）；`…-rows-cross-host-check-resume2-r6-r3.out` 判 `false` 的那一份作废（「作数的条件与 Q5（修订 6 那一趟）」一节 S17 那一条），不进判定：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-run1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-streams-1-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o3-p1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-main-before-the-first-barrier-of-call-3-run-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-row-0-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-row-0-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-probe-driver-resume2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-10-run-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-10-run-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-4-step-6-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-5-step-1-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-5-step-7-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s15-6-step-7-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r3-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-research-mutations.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations.out
```

修订 6 树之前几棵树上的第一段产物（85 份，`ls research/results | grep '^e171-r1-' | grep -v -- '-r6-r3' | grep -v 'research-mutations'`）：`2026-10-04` 起头的是修订 1 那棵树，`2026-10-05` 不带后缀的是第一段新树（`4d071c12…`，第 7 步 a1 在流三上不成立，修订 6 第 6.1 节），`revision-5-q5-` 是 Q5 树，`-r6`、`-r6-r2` 是修订 6 树前两趟停在第 0 步的记录。都保留为参照、不进任何判定（修订 6 第 6.1 节末表与「主 agent 认定：修订 6 起跑前仓里装置与写法偏离」第 3、5 条）：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-comparator-streams-1-2-before-the-rows.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-o3-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-memory-object.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-comparison.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-comparison-unmount.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o1-crash-judge-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-o3-p1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-panic.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-reference-filesystem-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-return-value.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-positive-control-witness.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-q5-main-before-the-first-barrier-of-call-3.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-q5-reverse-at-the-return-point-of-call-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2-device-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2-e170-export-dump.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-s2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-04-s1-stream-constants.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-baseline-run1-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-memory-object-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-comparison-unmount-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o1-crash-judge-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o2-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-o3-p1-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-panic-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-reference-filesystem-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-return-value-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-empty-control-witness-driver-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-frozen-tree-r6-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-o3-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-memory-object-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-comparison-unmount-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o1-crash-judge-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-o3-p1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-panic-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-reference-filesystem-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-return-value-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-positive-control-witness-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-q5-reverse-at-the-return-point-of-call-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-frozen-tree.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-1.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-1-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-main-run-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-reverse.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-revision-5-q5-reverse-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-e170-export-dump-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s3-s11-baseline-stream-1-with-probe-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-on-the-draft-copy-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r2.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-r6-r2-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-stream-constants-timing.out
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:856-956`（整段抄，未转述）**

````markdown
### 第 1 次重跑第二段·修订 6 那一趟（整表已跑：Q2 乙，Q3 分母 D = 39）

登记 `research/prompts/e171-r1-prereg.md` 第五节 5.5 第二段、第六节 Q2-集合与 Q3-分母两行、第一节读法 4 与读法 5、修订 5 第 5.1 节「已有产物的读法」末条（第二段只读 O1（内存模型对拍（HashMap 理想文件系统））-对拍与调用，两个崩溃子列不进这两行）、修订 6 第 6.6 节「第二段（整表）照原文，在修订 6 树上跑」，末节「主 agent 认定：第二段整表多机跑法交回」。树同第一段（快照摘要 `7726cdd6…`）；281 行，流一、流二，批 1 批 2，不跑执行器、不带探针，每行限时 1800 秒（T = 75.274 秒，3T 不到 1800）；两台机器跑完，超时 0 行，包装的结局 0 行，跨机一致对照第 201 行判同。

Q2 的「与基线不同」比的是修订 6 那一趟第 1 步 S2 新装置基线（`--streams 1,2`、不带探针，导出 `scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3/s2-device-baseline-streams-1-2/`）：修订 6 第 6.1 节末表说 Q2 的集合拿第 1 步这一趟比（旧装置那一份带不了根环表，比出来每行都会是「内存对象不同」），修订 2 第 2.5 节第二段的 T 也出自它。读法 4「基线连跑两次摘要不同的对象拿掉」：第 1 步只跑了一次，比对器另拿同一棵树、同一份装置的第 7、9 步两次三流基线（`baseline-run1/`、`baseline-run2/`）只比流一、流二，八个文件（写流四件、`states.tsv`、`oracle-verdicts.tsv`、`call-outcomes.tsv`、`call-return-values.tsv`）三份逐字节相同，不稳的对象 0 个。

| 行 | 判定（修订 6 树） | 依据（产物） | 照字面之外要一起读的 |
|---|---|---|---|
| q2 | 乙：集合 7 行，O1（内存模型对拍（HashMap 理想文件系统））-对拍红的 2 行（第 155、161 行），没红的 5 行（第 49、110、162、174、252 行） | `…-2026-10-07-s2-comparator-r6.out` 的 `name=q2`（`set_rows=7 o1_red_rows=2 … verdict=yi`） | 集合正好是 C672（交回值或内存状态错而写流不变的一类） 点名的七行，整表别的 227 行未触达行交回值与内存对象都与基线同。53 次不同的调用里，胶水交给模型的观测也不同的只有第 155、161 行抬 F 那一次（`ceiling`），O1（内存模型对拍（HashMap 理想文件系统））-对拍在这两次上 `disagrees`；其余 51 次观测相同：不同的是模型看不到的字段（`writes`、`output`、`current`、`publishes`、`allocator`、`system_configuration_writes_before_the_first_publish`）与内存对象（`mount_output`、`current_version`、`allocator`），O1（内存模型对拍（HashMap 理想文件系统））-对拍都 `agrees`。七行逐次调用与第一段在三流带探针那一趟上算出的逐字同 |
| Q3 分母 | D = 39（第 1 次重跑读法 5、批 1 定触达） | `name=q3_denominator` 四行 | 原登记读法 5（调用 panic 与态上的恢复 panic 一律自崩）D = 38，差的是第 125 行；批 2 定触达 D = 35（原登记读法 5 是 34），差的是第 27、112、233、277 行（只在批 1 某态看 journal 那一遍的恢复报告上与基线不同）。不进分母：未触达 234 行、自崩 6 行（第 6、7、85、95、235、272 行）、找不到 2 行（第 107、168 行）；编不过的 5 行都补上了，超时 0 行。第 125 行在登记算法两批都计入，在原登记算法两批都不计入 |

判决行与 Q2、Q3 各行（整行抄自 `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out`，第 6、7、349、410、411、414、415、416、421、426、431–441、447–450 行）：

```
E7RESULT name=unstable_memory_objects stability_baselines=2 names=-
E7RESULT name=driver_product row_time_limit_seconds=1800 snapshot_digest=7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4 rows=281 table_rows=281
E7RESULT name=q2_observation set_rows=7 calls_with_a_difference=53 calls_whose_observation_differs=2 calls_whose_observation_is_the_same=51
E7RESULT name=q2 set_rows=7 o1_red_rows=2 rows_o1_red=155,161 rows_o1_not_red=49,110,162,174,252 verdict=yi
E7RESULT name=q3_denominator reading=registration batch=1 rows=281 counted_D=39 counted=39 untouched=234 self_crash=6 not_found=2
E7RESULT name=q3_denominator_class reading=registration batch=1 class=self_crash rows=6 row_numbers=6,7,85,95,235,272
E7RESULT name=q3_denominator_class reading=registration batch=1 class=not_found rows=2 row_numbers=107,168
E7RESULT name=q3_denominator reading=registration batch=2 rows=281 counted_D=35 counted=35 untouched=238 self_crash=6 not_found=2
E7RESULT name=q3_denominator reading=original_reading_5 batch=1 rows=281 counted_D=38 counted=38 untouched=234 self_crash=7 not_found=2
E7RESULT name=q3_denominator reading=original_reading_5 batch=2 rows=281 counted_D=34 counted=34 untouched=238 self_crash=7 not_found=2
E7RESULT name=q3_self_crash_row row=6 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=7 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=85 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=95 where=new_pool_file_creation:call_2:judged_0,fixed_script_through_e_then_unmount:call_2:judged_0 registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=125 where=fixed_script_through_e_then_unmount:call_14:judged_12 registration=counted original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=235 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_self_crash_row row=272 where=before_the_first_call registration=self_crash original_reading_5=self_crash
E7RESULT name=q3_row_125 row=125 registration_batch_1=counted counted_registration_batch_1=true registration_batch_2=counted counted_registration_batch_2=true original_reading_5_batch_1=self_crash counted_original_reading_5_batch_1=false original_reading_5_batch_2=self_crash counted_original_reading_5_batch_2=false
E7RESULT name=recovery_panic_sum rows=0 states=0 by_row=-
E7RESULT name=not_located_row row=107 status=not_found failed_attempts=1 substituted_from=- name_hits_in_the_frozen_table=0
E7RESULT name=not_located_row row=168 status=not_found failed_attempts=1 substituted_from=- name_hits_in_the_frozen_table=0
E7RESULT name=untouched_row_red row=155 O1-对拍=1 O1-模型崩溃判官=0 O1-RefFS=0 O2=0
E7RESULT name=untouched_row_red row=161 O1-对拍=1 O1-模型崩溃判官=0 O1-RefFS=0 O2=0
E7RESULT name=driver_product_rows rows_in_the_product=281 rows_in_the_root=281 rows_whose_status_or_exit_differs=-
E7RESULT name=verdict q2=yi q2_set_rows=7 q2_o1_red_rows=2 d_registration_batch_1=39 d_registration_batch_2=35 d_original_reading_5_batch_1=38 d_original_reading_5_batch_2=34 row_125_counted_registration_batch_1=true rows_read=281 table_rows=281 rows_not_located=107,168 unstable_memory_objects=0 stability_baseline_files_differing=0 rows_compiled_without_the_rebuild_check=0 rows_with_an_errored_call_the_comparison_did_not_judge=0 rows_with_o1_not_applicable_from_another_source=0 untouched_rows_with_a_red_column=2 rows_whose_directory_differs_from_the_driver_product=0
```

集合七行每次调用不同在哪（在仓根跑 `grep 'name=q2_set_call' research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out | sed -E 's/.* row=([0-9]+) .* fields=([^ ]*) objects=([^ ]*) observation_differs=([a-z]*) .*/\1 \2 \3 \4/' | sort -k1,1n -k2 | uniq -c`，列是次数、行、交回值不同的顶层字段、不同的内存对象、观测同不同）原样输出：

```
     10 49 - mount_output false
      1 49 output mount_output false
      1 110 #0 current_version,mount_output false
      1 110 current,output current_version,mount_output false
      1 110 - current_version,mount_output false
      1 110 publishes current_version,mount_output false
      3 110 writes current_version false
      7 110 writes current_version,mount_output false
      1 155 ceiling - true
      1 161 ceiling - true
     10 162 - mount_output false
      1 162 output mount_output false
      1 174 system_configuration_writes_before_the_first_publish - false
      1 252 allocator allocator false
     13 252 - allocator false
```

- 两种读法下自崩那几行：第 6、7、95 行在两条流的第一个文件那次发布（第 2 次调用）panic，之前只有取号、暖机两次 `not_modelled`，可判 0 次；第 85、235、272 行在第一次调用之前就 panic（断言是系统运行配置、系统不可变配置写出的字节数与码 2 头的偏移与字段表对不上，推是 mkfs 那一步，没核到调用栈），两条流都没导出、一次调用没发（装置 `name=stream … exported=false not_exported_why=pipeline_panicked:… calls=0`）。读法 5 收窄的原文是「第一处调用 panic 之前可判的调用为 0」，第一次调用之前就 panic 的这三行可判的调用同样是 0，比对器记自崩（原比对器把 `pipeline_panicked` 也记自崩）；这一处读法待主 agent 认，换成另一种读法这三行也不进分母（没有导出可比，触达判不出）。
- 找不到的第 107、168 行：两行的变异名在冻结树的 `crates/mutations.tsv` 里命中 0 次（行文在登记之后改过名），驱动补位要从「这一名在今天这个文件组里的位置」往下找，名字不在组里就找不到起点，记补不上（`failed_attempts=1`、`substituted_from=-`）；登记 5.2「补不上照实报」照办，不进分母。照登记表里记的组内下标往下找是另一种读法，待主 agent 认；那样至多多两行，D 是 39–41，Q2 的乙翻不了（集合里已经有红的行，也有不红的行）。
- 编不过而补上的 5 行（第 33、59、127、163、206 行，第 127 行换了两次）都照补位规则换成同组下一行，换上的都未触达（`name=substitution` 五行与 `…-s2-rows-substitution-coordination-r6.out` 判决行 `substitute_collisions=0`）。
- 校验：七行逐次调用的不同（行、流、调用、字段、对象、观测同不同）与第一段比对器在三流带探针那一趟上算出的 `name=q2_named_row_call` 流一、流二部分逐字相同（两份各取 53 行、去掉第二段多出的标签与 O1（内存模型对拍（HashMap 理想文件系统））-对拍两列、排序之后 `cmp` 无输出）。两份走的是同一个比较函数，独立的只是两趟装置运行与两份基线。
- 作废与停机：Z6 的 `rows_compiled_without_the_rebuild_check=0`；S5 的比对器一侧两数都是 0；两批的看 journal 那遍恢复 panic 0 态（修订 5 第 5.1 节数法，第二段不读两个崩溃子列，读数不变）；Z3 的包装结局与超时 0 行（合成产物 `check_product.py rows_without_executor` 记 `timed_out=none wrapper_outcomes=none`）；Z5 合成产物第 2 行快照摘要 `7726cdd6…`。

比对器、单测、变异与复跑：

- 比对器 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_two_comparator.py`，另写一份、不改第一段比对器（第一段判决、单测与变异表都不许变；第二段的输入是一份基线、一个行输出根与一份合成产物，与第一段不同）。触达、交回值差、可判调用数、自崩、S5 两数照用 `collusion_matrix.py`；只多批 2 定触达（只留那一批的态、改记批 1 再交给同一个触达函数）、两种读法的分类与汇总。只读这一趟的导出根（`revision-6-2026-10-05-r3/` 与 `revision-6-segment-2/`）与产物前缀 `…-2026-10-07-s2-`、后缀 `-r6.out`，别的退 2。整表约 16 秒。
- 单测 `test_segment_two.py` 17 条（`grep -c '    def test_' test_segment_two.py`），手造流与临时目录，每一类判定至少一条：批 1 与批 2 定触达、驱动与装置结局的各类、自崩与触达的先后、两种读法、mkfs 那一步 panic、分母与各类逐行、Q2 集合（触达的行不进、只批 1 触达的行不进、不稳对象拿掉）、Q2 甲乙丙与 F4、导出根，加一条 8 行整表落盘跑 `main`、判决行整行钉死。连跑 5 轮都是 `Ran 17 tests … OK`（确定性的纯函数，5 轮只说明没有隐藏状态）。
- 研究一侧变异表 `scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_segment_two.tsv` 28 条（W1–W28），经同目录 `mutation_driver.py` 带 `--unit-test-module test_segment_two` 跑（驱动加了这个可选参数，不给时照旧跑第一段那两个模块：第一段那张表重跑一遍与 `…-s1-research-mutations-2026-10-07-r3.out` 逐字节相同）：`E7RESULT name=verdict mutations=28 caught=28 invalid=0 survived=0 timed_out=0 restored_and_baseline_green=true`（`…-2026-10-07-s2-research-mutations-r6.out`），抓到 / 无效 / 没红 = 28 / 0 / 0，连跑 5 轮输出逐字节相同。第一张表 37 条的锚点在加了新文件之后仍各命中 1 次。
- 复跑：`scripts/experiment-apparatus/replay.sh` 加两行，都只在已有的导出与产物上重出、不起装置与 cargo：

| 驱动函数 | 产物 | 跑什么 |
|---|---|---|
| `driver_e171_r1_s2_revision_6_comparator` | `…-2026-10-07-s2-comparator-r6.out` | 第二段比对器；读修订 6 那一趟第 1、7、9 步的导出与 `revision-6-segment-2/segment2-rows/`，本机留存、不进仓，没了就判红 |
| `driver_e171_r1_s2_research_mutations` | `…-2026-10-07-s2-research-mutations-r6.out` | 研究一侧 28 条变异，约 40 秒 |

  两个驱动函数照 `replay.sh` 的口径在草稿目录里跑过，输出与入库产物逐字节相同。整表本身（281 行逐行改坏、编、跑装置，两台机器）不登记。

第二段的产物（`ls research/results | grep '2026-10-07-s2-'`，8 份）：合成产物 `…-s2-rows-driver-r6.out`（565 行：281 行 `row`、281 行 `merged_shard` 与开头结尾几行）与它的 `-timing.out`；补位协调 `…-s2-rows-substitution-coordination-r6.out`；跨机一致对照 `…-s2-rows-cross-host-row-201-driver-r6.out`、它的 `-timing.out` 与判定 `…-s2-rows-cross-host-check-r6.out`（`E7RESULT name=verdict cross_host_identical=true row=201 host_of_rows_step=2 host_of_cross_host_run=1 parts=4 compared_parts=3 differing_parts=none`，o3 一格 `not_run`）；比对器 `…-s2-comparator-r6.out`；研究一侧变异 `…-s2-research-mutations-r6.out`：

```
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-check-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-row-201-driver-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-cross-host-row-201-driver-r6-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6-timing.out
e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-substitution-coordination-r6.out
```

#### 它答不了的（第二段）

- Q2 的乙只说「在流一、流二这两条历史上，未触达而交回值或内存对象不同的 7 行里 O1（内存模型对拍（HashMap 理想文件系统））-对拍红 2 行」：没红的 5 行差在胶水不交给模型的字段与对象上，O1（内存模型对拍（HashMap 理想文件系统））-对拍按设计看不见；它不说这 5 行的状态合不合法，也不说别的历史上集合多大。
- D = 39 是描述量，Q3 的甲乙由第一段第 125 行判（c = 12）；第二段不改第一段任何判定。D 的几个口径只差在读法 5 的自崩那一类与定触达的那一批，四个数都写。
- 判定只属于修订 6 树；修订 8 那一趟（最终主线）的第二段在哪棵树上跑另判（「主 agent 认定：修订 8 交回」第 8 条）。
- 第 85、235、272 行与第 107、168 行的归类是两处读法（「第 1 次重跑第二段·修订 6 那一趟」一节里「两种读法下自崩那几行」与「找不到的第 107、168 行」两条），待主 agent 认；两种读法都不让 Q2 翻面。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:957-1026`（整段抄，未转述）**

````markdown
### 第 1 次重跑·修订 12：流三段长重钉（2026-10-13）

登记 `research/prompts/e171-r1-prereg.md` 修订 12（设计员写）与修订 13（执行员写，只抄读数）；问题单 `e171-r1-s3-segment-lengths-questions.md` 第 1、3 行（第 2 行的状态由主 agent 改）。这一节只属于装置里一个常量 `LOCAL_THIRD_STREAM_SEGMENT_LENGTHS` 的两个数；Q1–Q5 的判定、数与修订 6 树的产物一个没动。

1. **为什么重钉**：2026-10-09 退役树判决落地后，今天的 crates 在流三删第一个文件那次多写两个退役单元（`crates/singlefs-core/src/transaction.rs` 里 `retirement_records_of_this_publish` 那一处），装置里 2026-10-05 钉的段长表与今天 crates 现算的在两处对不上。重钉前那一趟的产物（产物甲，`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12-before-the-repin.out`）第 82 行判决行里 `s1_failed_checks` 是 1（`grep -n 'name=verdict'` 取）。

   段长那一行（第 72 行，`holds=false`）4 KB 多，不整行抄；登记第 12.14 节的比对命令对产物甲的原样输出：

   ```
   lengths 959 959 sums 5980 5980 closed 362808742 362808742
   differing [(15, 20, 24), (43, 24, 20)]
   permutation True
   index_11_15 local 26 20 crates 26 24
   local_sha256 43d5bd3959c56382bc4bf220e35f964fda89b9d49af702d869bc31048aa6c5ce
   crates_sha256 fcec3e5e21df15d48891fc12d6dd366c282fff7306e58bc2144aac6bee2ac66c
   holds false
   ```

2. **新旧对照**：下标 15：20 → 24，下标 43：24 → 20；写数 5980、全量状态数 362 809 947、快档 3127、闭式 362 808 742 不变（乙里对应的 `…_writes`、`…_full_states`、`…_quick_tier_states`、`…_two_state_closed_form` 四行 `local` 与 `crates` 两栏相同、`holds=true`）。

3. **逐条判定**（乙 = 重钉后第一趟，丙 = 第二趟）：

   | 量 | 读数 | 判定 |
   |---|---|---|
   | RS1 | 乙里 `grep -c 'holds=false'` 为 0（`grep -c 'holds=true'` 为 77，S1 共 77 条）；`grep -n 'name=verdict'` 得 `82:E7RESULT name=verdict s1_failed_checks=0`；`cargo run` 退 0（取自主 agent 后台链的输出，执行员没有见到那份输出文件，由派发提示转述） | 全过：三样都到门槛 |
   | RS2 | 甲乙去掉段长那一行与 `name=verdict` 那一行之后 `diff` 的行数为 0；整份 `diff 甲 乙` 只有第 72、82 行两处不同 | 别的一项都没动（三条流的分配器带 mkfs 的根环表那三行两份相同） |
   | RS3 | 乙的段长那一行 `local` 与 `crates` 两栏 `differing []`、两栏摘要都是 `fcec3e5e…`；草稿表与甲 `crates=` 逐项相同（上一段执行员在草稿上核过） | 不是判据，只核抄对了：抄对 |
   | RS4 | 甲、乙段长那一行 `crates=` 一栏摘要都是 `fcec3e5e21df15d48891fc12d6dd366c282fff7306e58bc2144aac6bee2ac66c`，等于登记写的新钉值摘要 | 写一侧与新钉值逐项相同 |
   | RS5 | 甲：`holds=false` 恰第 72 行一行；不同下标恰为 15、43，(local, crates) 为 (20, 24)、(24, 20)；`s1_failed_checks=1`；退 3（上一段执行员的读数，经内存包装与线程上限起） | 阳性对照红在点名处 |
   | RS6 | 乙与丙 `sha256sum`，两行都是 `aef90046e0b565776288e5051c7267f706db4ee718bfceeb66dfb0272dfdd2a1` | 两趟逐字节相同（确定性的，只说明没有隐藏状态） |
   | RS7 | 「不写退役记录」一侧 | 入库装置上做不到，没量 |
   | RS8 | 改前、改后各一次 `grep -c 'stream-constants-r6-r[23]' scripts/experiment-apparatus/replay.sh`：0、0 | 两份没有复跑行，按第 7 条「2026-10-05 的两份旧产物」办 |

   RS2 的命令与输出（`<甲>`、`<乙>` 是第 4 条的两个文件）：

   ```
   $ diff <(grep -v -e 'check=first_file_unlinked_then_inodes_across_one_leaf_segment_lengths' -e 'name=verdict' <甲>) <(grep -v -e 'check=first_file_unlinked_then_inodes_across_one_leaf_segment_lengths' -e 'name=verdict' <乙>) | wc -l
   0
   ```

   RA1：甲的 `crates=` 下标 11 为 26、下标 15 为 24（旧钉值 20 加 4），条款与实现相符，F10 未触发。RA2：959 个数、和 5980、1 + Σ(2^n − 1) = 362 808 742（甲 `lengths 959 959 sums 5980 5980 closed 362808742 362808742`）。RA3：新旧两张表排序后相同，不同下标恰为 15、43（甲 `permutation True`、`differing [(15, 20, 24), (43, 24, 20)]`）。Z10、S19 的触发观测一条都没出现（RS5 到门槛、RS2 为 0、RS4 甲乙相同、RS6 相同、乙 `grep -c '^E7RESULT '` 为 83 等于 `name=done emitted=83`、内存包装没有退 245 或 250–254）。

   几何敏感性两点：流三变、流一流二不变。乙与 2026-10-05 的 `…-s1-stream-constants-r6-r2.out` 逐行比，`diff … | grep -E '^[0-9]'` 的原样输出是 `33c33`、`49c49`、`65c65`、`72c72`：不同的行恰为第 33、49、65 行（三条根环表摘要，两栏都是各自那棵树现算的）与第 72 行（流三段长）；流一、流二的段长、写数、根槽写数、全量 / 快档 / 闭式各行逐字节相同。

   装置里 `crates/mutations.tsv` 点名它的 47 行，锚点核对输出 `rows 47`、`count!=1 []`、`in 200-270 []`（命令在登记第 12.14 节）。

4. **产物**（都在 `research/results/` 下）：
   - 甲：`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12-before-the-repin.out`，83 行，sha256 `dcf5b558bed1a379437b91efb716a67fc15e5dbe5ac01ac6d4b10c7277137c9c`，退 3。重钉前的装置换上之后复跑不出，所以不登记复跑，只在这里点名。
   - 乙：`e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12.out`，83 行，sha256 `aef90046e0b565776288e5051c7267f706db4ee718bfceeb66dfb0272dfdd2a1`，退 0。
   - 丙：不入库，sha256 同乙。
   - 装置换上之后三份 sha256：改前 `c556d42d…`，草稿与换上之后都是 `6aa1bb82…`（`compile-then-swap.py` 退 0，换上的改动与草稿 diff 除文件名行外一致）；装置在主工作区相对暂存区现在有未暂存的这一块，怎么暂存归主 agent。

5. **复跑**：一行，登记在 `scripts/experiment-apparatus/replay.sh`，只留 `E7RESULT` 行、exact 比；整份 `replay.sh` 是重型，没跑，乙就是这条管道在今天的仓上的输出。

   | 驱动函数 | 产物 | 跑什么 |
   |---|---|---|
   | `driver_e171_r1_revision_12_stream_constants` | `…-2026-10-13-s1-stream-constants-revision-12.out` | 装置 `stream-constants --streams 1,2,3`，约 2 分钟 |

6. **问题单第 2 行**：「写」一侧由 RS4 判完：今天的 crates（本来就写退役记录）算出的段长表与新钉值逐项相同。「不写」一侧入库装置上做不到，三个取法各一句：装置自己带开关做不到，因为退役记录由 core 每次发布无条件追加，装置只经公开入口驱动、能动的只有换历史（那是另一条流）或事后抹写（抹不回分配落点）；取 f8a9ddaa 之前没有退役树的那棵树做不到，因为仓里没有带这份装置的那一次提交，且那棵树与今天差 22 个文件，分不开退役记录；取副本上的反事实只作线索，因为那要改 core 的源码，不属于入库装置。

7. **2026-10-05 的两份旧产物**：`…-2026-10-05-s1-stream-constants-r6-r2.out`、`-r6-r3.out`（逐字节相同）是修订 6 树上的读数，`crates=` 是那棵树现算的值、与今天装置的钉值和今天 crates 的现算值都不同；留作修订 6 那一趟的记录，不再是现值；复跑表里本来没有它们的行（RS8 改前改后都是 0）。`…-s1-stream-constants-on-the-draft-copy-r6.out` 同样照留。本页前面第 1 次重跑第一段里引它们的几句说的是修订 6 那一趟当时的事，没改。

#### 它答不了的（修订 12）

- 「互换由退役记录引起」只在调查员的副本上做过：把 core 里的退役记录换成空列表，段长表就与旧钉值逐项相同。副本上的数不进 kb，只当线索（记录 `records/2026-10-11-崩溃放量两段与脚本审计.md` 2026-10-13 那一条指到调查员报告）；入库装置上做不到，没有复现。
- 下标 43 为什么跟着变（分配记录树的叶起槽后推、落点后推几个槽）同样只来自副本上的打印，是推的；RA1 只管下标 11、15 的条款推算，不管 43。
- 修订 6 树上 `crates=` 等于旧钉值只说两者相容：那棵树与今天差的不止退役记录（登记修订 8 准备交回时数的是 140 个文件），不说互换的归因。
- 装置的内联用例只比段长表的和与闭式，对次序不敏感，下标 15、43 互换它看不出来；能看出的只有 `stream-constants` 真跑流三（所以没有往 `crates/mutations.tsv` 加变异行，段长表互换这条变异由产物甲担）。
- 反事实（「互换由退役记录引起」那一条）是线索，不是读数：它没有在入库装置上复现，不进任何判定。

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1027-1043`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 批 1 的态数（装置） | 取样三批的公式作用在崩溃点重放枚举表给的每段序号区间上，再加撕裂扫描；枚举点，不是全量 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1359` `sample_ordinals` | `crash_state_enumeration_tables` 的段区间与哪几次写取三态（来自 `crates/singlefs-checker-tier/src/crash.rs`） |
| 2 | 批 1 的态数（比对器现算） | 从 `ops.tsv` 按切段规则切段、`torn.tsv` 的写序号当三态写，每段状态数 = 各写取几态的乘积减 1，再按同一套取样公式现算序号集合 | `scripts/experiment-apparatus/e171_three_oracle_collusion/baseline_checks.py:175` `recompute_first_batch` | 取样公式（E170（O3规约执行器第一版） 登记 5.4 的文字）；`torn.tsv` 与 `ops.tsv`（装置从 crates 的枚举表与写表写出）：两条路径共用「哪几次写取三态」 |
| 3 | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官适用的态数（装置） | 一态里持久或撕裂的写属于的最后一次调用不早于模型停下的那次调用，这一态不适用 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1937` `crash_judge_is_applicable` | 调用表的返回点与写表的流序下标 |
| 4 | O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官适用的态数（比对器现算） | 从 `calls.tsv`、`ops.tsv`、`states.tsv` 的落地向量数写落在 `unmount` 那次调用里的态 | `scripts/experiment-apparatus/e171_three_oracle_collusion/baseline_checks.py:184` `recompute_crash_judge_applicable` | `calls.tsv` 的返回点（装置写出）；模型只停在 `unmount` 这一条读法（登记第一节读法 6） |
| 5 | O2（独立解析器 + checker） 在坏状态行上的判定（装置） | 从中性格式按落地向量重建 `MemoryPool`、叠 E170（O3规约执行器第一版） 写的补丁、调 `check_pool_image` | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:4751` `e170_bad_rows_on_both_streams` | 池级 checker 本身、E170（O3规约执行器第一版） 写的 `bad-patches.bin` 与落地向量 |
| 6 | O2（独立解析器 + checker） 在坏状态行上的判定（E170（O3规约执行器第一版） 导出装置） | `CrashImage` 上把补丁接在写表末尾当持久的系统配置槽写，调 `judge_crash_image` 取池级 checker 一项 | `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:3014` `observe_bad_row` | 池级 checker 本身；同一批补丁字节与落地向量 |
| 7 | 第 1 次重跑：O1（内存模型对拍（HashMap 理想文件系统））-模型崩溃判官适用的态数与 RefFS 投票的态数（装置） | 适用：一态落了的写属于的最后一次调用早于模型停下的那次调用；投票：RefFS 翻译没停在这一态的崩溃时刻之前 | `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3888` `crash_judge_is_applicable`；同文件 `:3172` `reference_filesystem_vote_of_a_state`（行号是仓里 `ab35105f…` 那一份的；修订 6 树里那一份在第 3475 行之后各少 7 行） | 调用表的返回点与写表的流序下标；`reference_filesystem_translation.rs` 的翻译停点 |
| 8 | 第 1 次重跑：O1（内存模型对拍（HashMap 理想文件系统））-模型崩溃判官适用的态数与 RefFS 投票的态数（比对器现算） | 从 `calls.tsv`、`ops.tsv`、`states.tsv` 现算：每个写落在哪次调用、最后一个落了的写所在的调用早于模型停下的那次；崩溃时刻取关上这一段的那一步，在 `unlink` 那几步或没成功的回退之前才投票；三条流全量态，枚举点不是扫遍 | `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py:176` `recomputed_applicable_and_voting` | 装置写出的 `calls.tsv`（第一条 op、返回点）、`ops.tsv`、`states.tsv` 的落地串与段号、`stream-status.tsv` 的 `model_stopped_at_call`；切段规则比对器自己照 `SegmentClosingRule` 写一份 |

路径 7 与 8 在第 1 次重跑比对器的产物里逐流比（`…-s1-comparator-resume2-r6-r3.out` 的 `name=a11` 三行都是 `holds=true`），同样没有 `assert` 把两条直接连起来；两条共用装置写出的调用表与落地串，它们错了两条一起错。

路径 1 与 2、3 与 4 在产物里逐格比（`name=s5_batch_1_states`、`name=s5_o1_applicable` 八行 `equal=true`），路径 5 与 6 逐行比（`name=e170_bad_row` 76 行 `o2_equals_e170=true`）；这几处比对写在产物里，没有 `assert` 把两条路径直接连起来，还欠这一条。共用项里最要紧的是池级 checker 本身（路径 5、6 同一份 `check_pool_image`）：它错了两条一起错，所以这一格不能当 O2（独立解析器 + checker） 判得对的证据；路径 1、2 共用「哪几次写取三态」，它错了两份态数一起错。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1044-1163`（整段抄，未转述）**

````markdown
### 复跑

装置与 E170（O3规约执行器第一版） 导出装置都是 checker 档的入库装置，经内存包装起；研究一侧是 Python。`scripts/experiment-apparatus/replay.sh` 里 E171（三oracle合谋测试） 原来的十一行登记（第 1 次重跑另九行（`grep -c '^E171（三oracle合谋测试）|@driver_e171_r1_' scripts/experiment-apparatus/replay.sh`）不在这张表里：第一段·修订 6 那一趟三行在「比对器、复跑与入库的跑法（修订 6 那一趟）」一节，第二段两行在「第 1 次重跑第二段·修订 6 那一趟」一节，修订 12 一行在「第 1 次重跑·修订 12：流三段长重钉（2026-10-13）」一节；修订 8、10、11 的研究一侧变异三行（驱动函数 `driver_e171_r1_s1_revision_8_tools_research_mutations`、`driver_e171_r1_s2_revision_10_tools_research_mutations`、`driver_e171_r1_revision_11_tools_research_mutations`）页里没有对应的节），各带自己的临时目录，产物只留 E7RESULT 行、exact 比：

| 驱动函数 | 产物 | 跑什么 |
|---|---|---|
| `driver_e171_s1_baseline` | `…-s1-baseline-run1.out` | 装置 `baseline` 一趟 |
| `driver_e171_s1_known_bad_images` | `…-s1-known-bad-images.out` | 基线、E170（O3规约执行器第一版） 导出装置 `bad-states`（占位种子）、装置 `known-bad-images` |
| `driver_e171_s2_bad_states_export`、`driver_e171_s2_known_bad_images`、`driver_e171_s2_images_o3`、`driver_e171_s2_o3_control_baseline` | `…-s2-bad-states-export.out`、`…-s2-known-bad-images.out`、`…-s2-images-o3.out`、`…-s2-o3-control-baseline.out` | 坏镜像三族在冻结种子下的造法、O2（独立解析器 + checker） 与冻结执行器的判定、O3（独立规约执行器） 阳性对照 |
| `driver_e171_s2_comparator` | `…-s2-comparator.out` | 第二段比对器那一串（九行试跑）；比对器在第三至六段改过判决行与 config 行，这份复跑对不上，没改指 |
| `driver_e171_s3_comparator_mutations` | `…-s3-comparator-mutations-2026-10-03-r7.out` | 研究一侧 67 条变异，`mutation_driver.py`，约一分钟 |
| `driver_e171_s4_comparator` | `…-s4-comparator.out` | 第二段那一串前置、修过的逐行驱动跑全部 117 行（两条线道、每条 2 线程，一份冻住的树）、Q3-核三趟、比对器；推的约两小时，没真跑过这个驱动函数（重型，归提交时的全部实验复跑）。117 行的读数读现行的 `crates/`，实现、O2（独立解析器 + checker）、O1（内存模型对拍（HashMap 理想文件系统））、装置或那 117 行的行文变了产物就变。整表那一步带 `--keep-compile-failures-as-they-are`：这份产物是编不过不补位的那一版驱动跑的 |
| `driver_e171_s4_f4_comparator` | `…-s4-f4-comparator.out` | 先整条跑 `driver_e171_s4_comparator`，再在它那一份冻住的树与工作根上跑扩表 164 行（同样带 `--keep-compile-failures-as-they-are`），比对器并两半、带扩表、段名 s4-f4；推的约四小时，没真跑过（`bash -n` 过；比对器那一半用同一组参数在已有导出上重跑过、与产物逐字节相同） |
| `driver_e171_s4_f4_substitutes_comparator` | `…-s4-f4-substitutes-comparator.out` | 先整条跑 `driver_e171_s4_f4_comparator`（它的草稿目录留着 s4、F4 的导出与冻住的树），再在同一份冻住的树与工作根上只跑两份输出根里结局是编不过的行（`--rerun-compile-failed-from`，补位跳过两张表的全部名字，两条线道、每条 2 线程，不带 `--keep-compile-failures-as-they-are`），比对器并 117 + 164 + 补位、带 `--substitution-rows-root`、段名 s4-f4-substitutes；推的约四个半小时（s4 与 F4 那一串约四小时，加补位 1 277.9 秒），没真跑过（`bash -n` 过；各步参数照补位那一趟实际跑的命令逐条对过，没有重跑驱动或比对器）。读现行的 `crates/`，实现、O2（独立解析器 + checker）、O1（内存模型对拍（HashMap 理想文件系统））、装置或那六行的行文变了产物就变 |

拿 s4 的输入（已有的导出目录与驱动输出）重跑 2026-10-03 的比对器，E7RESULT 行与 `…-s4-comparator.out` 逐字节相同（`cmp` 无输出）：比对器这一半复得出；装置与驱动那一半要整表重跑才核得到。第一段几样的单条命令：

```
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- baseline --output-directory <新目录>
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e170_o3_spec_executor_export -- bad-states --dump-directory <基线导出目录的一份拷贝> --seed-hex 0000000000000000
bash scripts/hosts/run-with-memory-cap.sh 8G cargo run -q --release -p singlefs-checker-tier --bin e171_three_oracle_collusion -- known-bad-images --dump-directory <上一步那份拷贝> --output-directory <新目录> --e170-bad-rows
cd scripts/experiment-apparatus/e171_three_oracle_collusion && python3 mutation_driver.py --table ../../../../mutations/e171_three_oracle_collusion.tsv --scratch <草稿目录> --output <新文件>
```

第一段比对器那份（`…-s1-comparator.out`，第一段的 `driver_e171_s1_comparator` 已由 s2 那一份取代、不再登记）要先跑齐基线两趟、E170（O3规约执行器第一版） 导出装置的 `dump`、关一条两趟、阳性对照六趟（基线三趟，空变异三趟经 `row_driver.py --empty-mutation --positive-control <名>`）、逐行驱动三行（`row_driver.py --empty-mutation --rows 1,6`），再 `python3 comparator.py …`，大约十几趟装置（每趟约一分钟），驱动函数把这一串写死；比对器的 config 行只写目录的名字，所以复跑目录的名字要与驱动里的相同。比对器的 a4–a7 读现行的 `crates/mutations.tsv`：表里别处加了核心实现的行或改了那 117 行，这几行会变，不是装置坏了。占位种子 `0000000000000000` 造出的盲测行 E171（三oracle合谋测试） 不用；E170（O3规约执行器第一版） 在第 1 次重跑里加了第二套坏状态表（`bad-states-second-run`），`bad-states` 仍是第一套，两个版本的导出装置在第一套上的输出逐字节相同（「对照与自检」一节最后一条）。

复跑驱动的自检：基线、已知坏镜像、变异驱动三个驱动函数照 `replay.sh` 的口径在草稿目录里跑过一遍，输出与入库产物逐字节相同（`cmp` 无输出）；比对器那一个驱动函数没有整条真跑（十几趟装置，前台跑不完），它的胶水（各步命令行、目录的名字、传给比对器的参数）用「装置、E170（O3规约执行器第一版） 导出装置与逐行驱动换成从已有导出目录拷」的桩跑过，比对器照真的跑，输出与入库产物逐字节相同；各步命令本身在做产物时逐条真跑过。整条真跑由提交时的复跑做。

产物（都在 `research/results/` 下，名字前缀 `e171-three-oracle-collusion-2026-10-03-s1-` 之后接）：

| 文件名（前缀之后） | 内容 |
|---|---|
| `baseline-run1.out`、`baseline-run2.out` | 基线两趟的 E7RESULT 行（两份逐字节相同） |
| `baseline-timing.out` | 两趟的 E7TIMING 行（每批每段耗时、每条流驱动、判、写各花多久） |
| `known-bad-images.out` | 装置 `known-bad-images` 模式：41 条已知坏镜像与 E170（O3规约执行器第一版） 的 76 行坏状态上的 O2（独立解析器 + checker） |
| `comparator.out` | 比对器：登记锚点、基线与对照、关一条、三行试跑、Q1 分母、Q2 的 O2（独立解析器 + checker） 一列 |
| `rows-timing.out` | 逐行驱动与比对器的 E7TIMING 行（Q4 单行数） |
| `comparator-mutations.out` | Python 一侧 9 条变异 |
| `e170-pinned-dump.out` | E170（O3规约执行器第一版） 暂存版本导出装置的 `dump`（S2 的对照） |
| `e170-bad-states-placeholder-seed.out` | 同一版本的 `bad-states`（P、N 行的出处，盲测行不用） |
| `positive-control-o1-comparison.out`、`positive-control-o1-crash-judge.out`、`positive-control-o2.out` | 三条阳性对照在基线那一趟的装置输出 |
| `disable-o1.out`、`disable-o2.out` | 关一条的开关自检（基线上各一趟）的装置输出 |
| `row-0-device.out`、`row-1-device.out`、`row-6-device.out` | 三行试跑的装置输出（row-0 是空变异） |

全名（`ls research/results | grep '^e171-three-oracle-collusion-2026-10-03-s1-'`，17 份）：

```
research/results/e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out
e171-three-oracle-collusion-2026-10-03-s1-baseline-run2.out
e171-three-oracle-collusion-2026-10-03-s1-baseline-timing.out
e171-three-oracle-collusion-2026-10-03-s1-comparator-mutations.out
e171-three-oracle-collusion-2026-10-03-s1-comparator.out
e171-three-oracle-collusion-2026-10-03-s1-disable-o1.out
e171-three-oracle-collusion-2026-10-03-s1-disable-o2.out
e171-three-oracle-collusion-2026-10-03-s1-e170-bad-states-placeholder-seed.out
e171-three-oracle-collusion-2026-10-03-s1-e170-pinned-dump.out
research/results/e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-comparison.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o1-crash-judge.out
e171-three-oracle-collusion-2026-10-03-s1-positive-control-o2.out
e171-three-oracle-collusion-2026-10-03-s1-row-0-device.out
e171-three-oracle-collusion-2026-10-03-s1-row-1-device.out
e171-three-oracle-collusion-2026-10-03-s1-row-6-device.out
e171-three-oracle-collusion-2026-10-03-s1-rows-timing.out
```

第二段起的产物（`ls research/results | grep -E '^e171-three-oracle-collusion-2026-10-03-s[234]-'`，29 份，名字前缀 `e171-three-oracle-collusion-2026-10-03-` 之后接）：

| 文件名（前缀之后） | 内容 | 算不算数 |
|---|---|---|
| `s2-bad-states-export.out`、`s2-known-bad-images.out`、`s2-images-o3.out`、`s2-o3-control-baseline.out`、`s2-o3-baseline.out` | 坏镜像三族的造法与 O2（独立解析器 + checker）、O3（独立规约执行器） 判定、O3（独立规约执行器） 阳性对照、冻结执行器在基线上（a3 的 O3（独立规约执行器） 一份） | 算：s4 比对器读的就是这几份 |
| `s2-comparator.out`、`s2-rows-timing.out` | 第二段比对器（Q2 全表、九行试跑）与它的耗时 | Q2 的数同 s4；九行试跑由 s4 整表取代 |
| `s2-comparator-mutations.out` | 变异表 26 条时的全表跑 | 历史 |
| `s3-baseline-after-the-fix.out`、`s3-row-0-device.out` | 修订 3 修过取号、暖机记红之后的装置基线与空变异一行（与修之前逐字节相同） | 算：修订 3 的实核 |
| `s3-k4c-attack-row-device.out`、`s3-k4c-check-driver.out` | 修订 3：取号恒拒那一行经逐行驱动跑一行（O1（内存模型对拍（HashMap 理想文件系统）） 红） | 算：修订 3 的实核 |
| `s3-comparator.out`、`s3-rows-timing.out` | s3 整表 117 行的比对器与耗时 | 作废留档（「s3 整表作废」一节） |
| `s3-clean-check-comparator.out` | 第五段在干净工作根上复核第 2、27、42、65 行 | 作废的佐证，留档 |
| `s3-comparator-mutations.out`、`s3-comparator-mutations-2026-10-03.out`、`-r2.out`、`-r3.out` | 变异表 37、40、44、44 条时的全表跑 | 历史 |
| `s3-comparator-mutations-2026-10-03-r4.out` | 52 条那一次中途被停（61 行、没有判决行） | 不算数，留档 |
| `s3-comparator-mutations-2026-10-03-r5.out` | 变异表 52 条的全表跑，52 / 0 / 0 | 历史 |
| `s3-comparator-mutations-2026-10-03-r6.out` | 变异表 65 条的全表跑（x60–x72 加进来之后），65 / 0 / 0 | 历史 |
| `s3-comparator-mutations-2026-10-03-r7.out` | 变异表 67 条的全表跑，67 / 0 / 0 | 算：复跑登记指它 |
| `s4-comparator.out`、`s4-rows-timing.out` | s4 整表 117 行、Q3-核三趟、比对器与耗时 | 算：117 行那一半的读数（F4 产物里第 1–117 行的结果行与它逐字节相同） |
| `s4-f4-comparator.out`、`s4-f4-rows-timing.out` | F4 扩表 164 行与 s4 的 117 行并表的比对器与耗时 | 算：补位之前的读数（复跑登记里带 `--keep-compile-failures-as-they-are` 的那一份） |
| `s4-f4-substitutes-comparator.out`、`s4-f4-substitutes-rows-timing.out` | 117 + 164 行并上编不过 6 行的补位之后的比对器与耗时（补位那一趟 1 277.9 秒） | 算：补位之后的 Q1、Q3、Q4 读数（Q1、Q3 名单与补位之前逐字同），三轮判决引的产物行号取自这一份 |

全名：

```
research/results/e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out
e171-three-oracle-collusion-2026-10-03-s2-comparator-mutations.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-images-o3.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out
e171-three-oracle-collusion-2026-10-03-s2-o3-baseline.out
research/results/e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out
e171-three-oracle-collusion-2026-10-03-s2-rows-timing.out
e171-three-oracle-collusion-2026-10-03-s3-baseline-after-the-fix.out
e171-three-oracle-collusion-2026-10-03-s3-clean-check-comparator.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r2.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r3.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r4.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r5.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r6.out
research/results/e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out
e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations.out
e171-three-oracle-collusion-2026-10-03-s3-comparator.out
e171-three-oracle-collusion-2026-10-03-s3-k4c-attack-row-device.out
e171-three-oracle-collusion-2026-10-03-s3-k4c-check-driver.out
e171-three-oracle-collusion-2026-10-03-s3-row-0-device.out
e171-three-oracle-collusion-2026-10-03-s3-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out
e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out
research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-rows-timing.out
e171-three-oracle-collusion-2026-10-03-s4-rows-timing.out
```

````

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1164-1187`（整段抄，未转述）**

```markdown
### 它答不了的

- Q1 的「有」是照字面判的（≥ 1 行）；12 行（丙 13 行）里哪几行算真坏，三轮结论判决给了三分（「三轮判决之后的三分」一节）：真盲区候选 7 行（第 15、23、42、242、269、270、271 行），状态不合法所依的条款句写在「三条全放行的 7 行」「F4 新出的全放行行」两节；另几类 5 行里第 92 行那一句在字节表与里程碑验收、不在决策正文。三分只是「这一批取到的态」上的读法，各行会分开的历史多数是推的、没跑。分母 D = 37 补位之前与之后相同（补位的六行都未触达），Q1 的「有」没翻。
- 281 行里 238 行未触达（补位之后；补位之前 232 行，另 6 行编不过）：这两条流的历史只走到实现的一小部分（取号、暖机、发布、可写挂载、管理员回退、抬 F、正常卸载各一两次，没有分裂、没有删除、没有空间紧的池），未触达只说那一处改动的写流在这两条流上没变，不说它被哪条 oracle 罩着；其中 7 行改动走到了、只是没写到盘上（第 49、110、155、161、162、174、252 行，「没触达却红的两行与未触达行里交回值不同的七行」一节）。三轮之后没有一行全放行行还记「对这一批等价」。
- 触达的定义（登记读法 4）不比成功的调用交回的值，这一条没改（登记不改），分母仍 37：第 155、161 行抬 F 的上限交回 13（模型 11）、写流与结局都同，记未触达，而 O1（内存模型对拍（HashMap 理想文件系统））-对拍判红；攻方在 s4 树的副本上量了 232 行未触达行的成功调用交回值，7 行不同（第 49、110、155、161、162、174、252 行），汇总行 `name=return_summary hidden_difference=7 identical=225 no_output=6`（副本探针，s4 树，入库装置没有这一列；整行与来历在「没触达却红的两行与未触达行里交回值不同的七行」一节）；mkfs 的 6 行（第 149–154 行）没量。
- 编不过的 6 行的补位那一趟已跑（「编不过的 6 行补位（已跑）」一节）；s4、F4 两份产物是补位之前的驱动跑的，复跑它们的登记行带 `--keep-compile-failures-as-they-are`、照旧不补位。
- 顺序类的条款（第 242、270 行）是在崩溃点重放的切段模型上数的：「条款不许的态」靠切段规则（FUA 写只放行它自己那块盘、屏障放行它那块盘）分得出，真机上一次 FUA 写与随后的写怎么排，不在这一批里。
- O3（独立规约执行器） 在 37 行上一行都没红：它的 A(c) 由实现自己写出的根构成，抓的是「恢复落到的那一版不在 [ack, begun] 里」这一类，实现一致地写错的字段（容器号、写入时间、F 的取值、记账行）它照样判「在」。这是读执行器代码读出来的，没有另造一行只让 O3（独立规约执行器） 红的坏实现去核；坏镜像那一族里 O3（独立规约执行器） 红得多（Q2）。
- 三行试跑（第一段）不是表的抽样，s4 整表取代了它们当 Q1 的读数；第 1 行未触达只说那一处改动在这两条流的历史上没走到。
- 基线上 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官有一部分态不适用：流二最后一次调用 `unmount` 模型没有答，写落在这次调用里的态不判；基线批 1 流二 397 态里 48 个、批 2 的 119 态里 14 个不适用（`o1_crash_judge_not_applicable`），这些态上 O1（内存模型对拍（HashMap 理想文件系统）） 一列是空的。流一没有不适用的态。
- 阳性对照 O1（内存模型对拍（HashMap 理想文件系统））-崩溃判官每条流各翻一次（合计 2 处），登记字面是「批 1 里第一个」；O2（独立解析器 + checker） 的对照只在流一的全持久态上叠第一条已知坏镜像，没有别的取样点。对照证明管子把红带到矩阵，不证明每条 oracle 对每种改坏都分得出。
- Q2 的 O2（独立解析器 + checker） 一列取自两条路径（路径 5、6）；P 行的补丁字节与落地向量是 E170（O3规约执行器第一版） 导出装置写的，期望是 E170（O3规约执行器第一版） 设计员推的（登记 a2、F3）。Q2 全放行的 4 张全是 P8，那一判归 F3。41 条已知坏镜像是 O2（独立解析器 + checker） 自己的回归语料，O2（独立解析器 + checker） 41 条全红是构造的结果，不是 O2（独立解析器 + checker） 独立性的证据。
- 逐行驱动与装置读跑那一刻的 `crates/`：s4 五趟驱动是同一份冻住的树，但 `crates/` 里别处在变，整表跑出来的数只说那一份树。
- 坏镜像上实现的恢复读回什么（登记第一节读法 11，附带）、旧种子盲测族的回归都没跑。Q1-丁（三条全放行而记录核对器或崩溃点重放手写版本表的 oracle 红的行）跑了：F4 的 12 行全放行（甲、乙）里一行都没有（`name=q1_delta … =none`）；比对器只按甲乙算这一格，丙多出的第 233 行上崩溃点重放那份 oracle 与记录核对器红（`name=q1_quiet_row reading=q1_gamma row=233 … layer0_or_record_checker_red=true`）；崩溃点重放那份 oracle 与记录核对器另在只靠 O1（内存模型对拍（HashMap 理想文件系统）） 的第 108、115 行上红。
- 装置内联单测没跑（checker 档），装置一侧变异 15 行的三个数没有；Python 一侧的变异只证明比对器与驱动的谓词会红，不证明它们读的装置输出是对的（装置输出由两趟基线、S2、S5 与对照另行钉）。
- E171（三oracle合谋测试） 的 O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS（登记把崩溃判官算进 O1（内存模型对拍（HashMap 理想文件系统）），另写 E171（三oracle合谋测试） 不用 RefFS），与 D13（验证路线） 已定项 6 表 O1（内存模型对拍（HashMap 理想文件系统）） 行把崩溃之后允许看到的集合归 RefFS 不同；两者的下界也不同（盘上已持久的最新根，对最后一次返回成功的发布）。「已确认而丢了」那一类在 E171（三oracle合谋测试） 的四列里只有 O3（独立规约执行器） 判，没有在 37 行上跑过 RefFS（读码读出来的）。RefFS 的数只在三轮的第二轮判决里（`research/prompts/e171-conclusion-r2-main-verification.md` 第二节 R6 一格，读数在 `judge-summary.out`，攻方副本探针，s4 树 core，RefFS 直调 crate、崩溃落点手给、没经翻译层）：暖机那一形 O3（独立规约执行器） 判「不在」27 态，RefFS 判绿的也是这 27 态（27 / 27）；带内容的同形写流 O3（独立规约执行器） 判「不在」的 27 态 RefFS 全红，RefFS 红的共 36 态，另 9 态 RefFS 红而 O3（独立规约执行器） 判在（两边归属崩溃时刻的规矩不同，推的）。这几个数不是 E171（三oracle合谋测试） 装置读数。
- Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的 29 张（P 族 12、盲测 B 族 17，「Q2 里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的坏镜像」一节）全是结构自洽的伪造根、伪造续接镜像，不是坏实现；补丁与期望同出 E170（O3规约执行器第一版） 的改法代码（E170（O3规约执行器第一版） 页「它答不了的」），源头独立在 Q2 只量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；流一 P2、P5 两种补丁的 O2（独立解析器 + checker） 一列随底态翻面。这 29 张的复跑读现行的 `crates/`，不还原 s4 的读数：基线流与 s4 树不同，数不保证仍是 12 与 17。D13（验证路线） 依据段只指到 E171（三oracle合谋测试） 页的这个数。
- 这一页的行级读数（全放行、只靠一条、未触达的名单与每行的态）只属于快照摘要那一棵冻住的树：它与现行的 `crates/` 不是同一份，页里的名单不能直接当现行实现的名单；`replay.sh` 的 E171（三oracle合谋测试） 复跑行读现行的 `crates/`，复跑一次出的是复跑那一刻那棵树的读数。s4 那一版入库装置（`scripts/experiment-apparatus/e171_three_oracle_collusion/`）上除 Q2 的 29 张之外，这几个数都不能原样重做（这一版装置没有挂载模式、没有 RefFS 列、panic 的流不导出；第 1 次重跑的装置 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` 有 RefFS 那一列，挂载模式与 panic 的流导不导出两项没核）。
- 第 125 行是自崩的 6 行之一：登记读法 5 把 panic 记自崩、不进分母，D = 37。它在 panic 之前有 13 次调用可判：副本探针在 s4 树上驱动、打第 125 行变异，第 4–13 次调用的全持久态 O2（独立解析器 + checker） 判 I-3.11（已分配减 defer 等于最新根走读） 红，第 14 次（`unmount`）panic（`drive-s4-m125.tsv`），装置对 panic 整条流不导出。把 panic 与报错同样截断照判，计入分母 D = 38（改口径的预演，不是装置读数，37 与 38 都写）；若触达也把成功调用的交回值算进去，含第 155、161 行，D = 40（37 加第 125、155、161 行）；7 行交回值不同的行都算进来、含第 125 行，D = 45（推的，算术）。
- 谓词 W（正确实现的可写挂载会把这一态读成「见证过的更新一版读不出」）在第 242、269 行取到的崩溃态上为真的态各 15、63 个，基线 0 / 576；s4 树 core 上两次可写挂载的见证类拒 61 个（第 242 行 9、第 269 行 52）。这些数是攻方副本脚本（W）与副本探针（挂载）在 s4 树上量的，入库装置没有 W、也没有挂载模式。2026-10-03 的 core 读 s4 写出的镜像大面积 `refused:Undecidable`（基线流一 44 / 60、流二 500 / 516）：格式变了（journal 头 311 → 487、新根段 188 → 364、点名项 67 → 64），旧记录在新解析器下读不出，不是回归；所以 W 的 15 / 63 与 61 个见证拒只对 s4 core 成立，在现行的 core 上没有对照意义。
- 用户数据落点的取值算不算 layout 登记的字节，这一页不判；三轮判决交用户（`research/prompts/e171-conclusion-r3-main-verification.md` 第五节第 1 行，零轮），用户 2026-10-03 定「算」：D3（空间分配） 已定项 8 射程改成新池新建文件上的落点取值（t1@50180）算格式级，检查在 C673（三 oracle 合谋测试的补 oracle 改法没做） ⑥，已还清（见欠账表已还清表）。它不改这一页的三分：第 15 行的依据是 ⑤ 的 t2–t12 落点；第 34 行的落点改动在流二抬 F 之后那次发布上，不是新池新建文件，仍归分配政策。
- 新树重跑装置（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`，这一页没记它的读数）打的 `name=batch`、`name=verdict` 两种行里，`piece_panic_states=` 按件计、不按态：`<件>:<态数>` 逗号连起来，0 的件不写，全 0 写 `-`；同一次 panic 在 `layer0_oracles` 与 `crash_judge` 两列各记一次（第 27–29 批代码轮判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第三节第 3 条，两腿同判，不改装置）。
- 变异表旧第 38 行已改名「步 4：回退候选集的 F 用目标根自己带的 F 而不是 F_生效」（改名后在 `crates/mutations.tsv` 里，旧名写「最新根自己带的 F」）；两份 `bad-implementations-117.tsv`（`scripts/experiment-apparatus/e171_three_oracle_collusion/` 与 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` 下）第 33 行按旧名引用，从现行的树取快照重跑时这一行按旧名找不到、走补位、不静默失效（正在跑的修订 6 读冻结树里的旧表，不受影响；第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P4「旧第 38 行改名」一行与第三节第 5 条）。

```

**出处 `.claude/kb/experiments/171-三oracle合谋测试.md:1188-1206`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 6 | 支撑 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D13（验证路线） 已定项 1 | 备料 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D13（验证路线） 已定项 7 | 备料 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D13（验证路线） 已定项 15 | 备料 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D16（发布语义） 已定项 7 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D16（发布语义） 已定项 12 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D23（journal 的角色与格式） 已定项 3 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D23（journal 的角色与格式） 已定项 15 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D3（空间分配） 已定项 8 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |
| D3（空间分配） 已定项 10 | 不影响 | 2026-10-18 不受影响：装置一节补点名研究一侧变异表三次复跑的产物，结论、数不变 |

```

**出处 `.claude/kb/tooling.md:1-7`（整段抄，未转述）**

```markdown
# 工具与环境事实

**中文** · [English](tooling.en.md) · [日本語](tooling.ja.md)

**tooling.md 放「本工程用到的工具现在是什么状态、用的时候怎么做」**。
规则不进 tooling.md，进 [CLAUDE.md](../../CLAUDE.md) 或 `.claude/rules/`。

```

**出处 `.claude/kb/tooling.md:8-26`（整段抄，未转述）**

```markdown
## 本地 LLM（三方论证不派它当腿）

三方论证只派两条云端腿加主 agent 判决，不派本地模型的腿。仓里调网关的脚本与字词损坏检测器（`ask-local.sh`、`ask-local-selftest.sh`、`local-repro.sh`、`local-repro-tuned.sh`、`corruption-check.py`、`corruption-summary.py`、`oov-check.py`、`en-words.txt`）2026-10-11 删掉了；要再用，`git log --diff-filter=D -- scripts/agents/ask-local.sh` 找到删它的那次提交，从它的上一版取回。

| 项 | 值 |
|---|---|
| 入口 | 本机的本地模型网关（OpenAI 兼容，地址与所在目录不进仓） |
| 鉴权 | 请求头 `X-Api-Key`，取值在本机私有配置里，不进仓 |
| 模型 | `local` → `Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`，`max_model_len` 262144 |
| 调用脚本 | 仓里没有了（取回办法见本节开头那一段） |
| 预算 | 不传 `max_tokens` 与 `thinking_token_budget` |

用它的时候：

- 提示一律用英文，不用 markdown 强调（`**粗体**:`）；不靠 `presence_penalty`、`repetition_penalty` 修中文复读。
- 它给出的具体字段名、结构名、机制描述一律当待核线索；核不动整条丢掉。
- 提示里把最容易上钩的类别错误写出来并禁止：「同一系统在两种介质上」不许答成「两个不同系统」；「宽度在别处定」写明「推迟的是宽度，不是字段本身」。
- 它答「举不出来」时跑阳性对照：同一份提示只把被问对象换成已知答案是「有」的对象，这次举得出来，那个「举不出来」才算数。

```

**出处 `.claude/kb/tooling.md:27-38`（整段抄，未转述）**

```markdown
## 仓里的性能数都只在本机成立，引用要带机器口径

**射程**：挂钟、吞吐、每次发布多少毫秒、门禁跑多久、冷挂载多少秒、并行加速比，凡是时间与速率。

**怎么做**：

- 引一个性能数时把**机器口径**一起带（哪台机、什么盘、什么负载、空不空闲），不带口径的数不许进判据。
- 拿本机的数去定**阈值、上界、超时**时，写明「这是本机值，别的机器上要重量」。
- 需要一个跨机器成立的数时，要么量两台以上，要么写成条件式（「按 X 算是 Y，那一档本机测不到，仍是假设」）。

门禁那一档整环清零的耗时、冷重开恢复的耗时、崩溃点重放两条流的挂钟、E152（按里程碑对比六家文件系统的文件性能） 与六家的全部对比数都是本机数（C486（整环清零的 55 秒与冷重开的 17 秒都没人管））。

```

**出处 `.claude/kb/tooling.md:39-45`（整段抄，未转述）**

```markdown
## 超时地板按空闲机器标定，就会把「机器上还有别的活」记成「变异抓不到」

- 并发度乘以一条被测单位自己占的核数，不超过核数。cargo-mutants 跑本仓用 `--jobs 1`。
- 超时地板按带负载的机器标定；标不准就不设超时。
- 撞满超时的条目单列，不并进抓到、没红、无效任何一栏；报数时四个数都报。
- 地板值带机器口径（「仓里的性能数都只在本机成立，引用要带机器口径」一节）。

```

**出处 `.claude/kb/tooling.md:46-56`（整段抄，未转述）**

````markdown
## 变异条数会随源码漂，不许在 kb 里钉死

正文里写数它的命令，不只写数：

```
cargo mutants --list --package singlefs-checker | wc -l
cargo mutants --list --package singlefs-core | wc -l
```

要引一个具体的数，连同量它的日期写成事件句（「<日期> 数出 N 条」），不写成现状句。

````

**出处 `.claude/kb/tooling.md:57-60`（整段抄，未转述）**

```markdown
## 改了引用方，要 grep 全仓改被引用方

改正一处措辞之后，说「已全部改完」之前，先 `grep -rn` 全仓数一遍。范围含 `kb/` 全部子目录、`research/prompts/`（喂给三方论证的背景材料）、`scripts/` 与 `research/` 下的源码注释、`records/`、门禁脚本。数不清就写「改了这几处，没有全仓核过」。

```

**出处 `.claude/kb/tooling.md:61-65`（整段抄，未转述）**

```markdown
## 一个数不许从「它自己判为非法」的配置里算出来

- 算一个上限或下限之前，先确认那个配置在本工程的判定下是合法的。
- 撤回一个数时，作废理由写成真实的那一条（缺输入、配置非法、口径错），不写成它保留下来的别的档同样有的性质。

```

**出处 `.claude/kb/tooling.md:66-69`（整段抄，未转述）**

```markdown
## 未定项一律写成编号条目，标题一律声明条数

按 `^\d+\.` 提取的脚本只认编号条目；`.claude/gate.d/doc-decisions.sh` 的 kb-shape 格的条数检查只在标题声明了「N 项未定」时启动。

```

**出处 `.claude/kb/tooling.md:70-82`（整段抄，未转述）**

```markdown
## 验证产物放哪：四个位置，各管一段

| 位置 | 放什么 | 谁读 | 进仓库 |
|---|---|---|---|
| `.claude/kb/experiments.md` | 结论、口径、判据、复现命令、**判决性的那几行数字** | 模型检索 | 进 |
| `records/` | 每一轮的叙述：定了什么、推翻了什么、依据 | 人通读 | 进 |
| `scripts/` | 实验代码与脚本 | 跑的人 | 进（`.gitignore` 只排除 `target/` 与个别超大文件） |
| `research/results/<实验>-<日期>.out` | 原始输出全文 | 下一轮复跑时逐行比对 | 进；早于最近一轮的在下一次提交时归档进 git 历史（门禁 doc-experiments 的 archive-past-rounds 格） |

- 判决性的数字贴进 kb 正文。
- 每份原始输出要能自证是产出它的那一次运行留下的：收尾行 `E7RESULT name=done emitted=N` 里的 N 等于文件里 `E7RESULT` 的行数，对不上整轮作废。归档时核一遍：
  `for f in research/results/*.out; do echo "$(grep -c '^E7RESULT' $f) vs $(grep 'name=done' $f)"; done`

```

**出处 `.claude/kb/tooling.md:83-95`（整段抄，未转述）**

```markdown
## 内核源码：本机有完整树，不必 WebFetch

| 项 | 值 |
|---|---|
| 路径 | `~/kbuild/linux-om`（1.8 GB） |
| 版本 | **Linux 7.2.0**（`Makefile` 的 `VERSION = 7` / `PATCHLEVEL = 2` / `SUBLEVEL = 0`） |
| 有什么 | 完整 `.c`/`.h` 正文，含 `fs/jbd2/`、`fs/xfs/`、`fs/btrfs/`、`fs/erofs/`、`fs/ext4/`、`fs/f2fs/`、`drivers/nvme/`、`drivers/md/`、`crypto/`、`Documentation/` |
| **没有什么** | **`fs/bcachefs/` 不在这棵树里**（见 [decisions.md](decisions.md) D7（是否进 Linux 主线））。`/usr/src/linux-headers-6.17.0-1028-oem/fs/bcachefs/` 只有 `Kconfig` + `Makefile`，**没有源码** |

- 查源码用 `~/kbuild/linux-om`，不用 `/usr/src/linux-*-headers-*`（那里只有 `Kconfig` 与 `Makefile`）。
- 引哪棵树就标哪个版本号，两个版本的结论不混着引。kb 里标「现查 Linux 6.17」的引用来自 GitHub 的 `v6.17` tag，本机没有那个版本。
- OpenZFS 本机没有树，现取 `https://raw.githubusercontent.com/openzfs/zfs/master/...`，引用时记下取回日期，行号不是永久的。

```

**出处 `.claude/kb/tooling.md:96-101`（整段抄，未转述）**

```markdown
## 到点自己开工：用 remind.sh 登记

用户要会话到点自己开工时，只用 `bash scripts/agents/remind.sh start --every <分钟> --message <提醒内容> --once`（Bash 的 `run_in_background` 起，到点退出叫醒会话；规则 `.claude/rules/timer-only-remind.md`，写法 `.claude/skills/remind/SKILL.md`）；不用 CronCreate，也不在 Bash 里只为定时而写 `sleep`。提醒内容写开工指令所在的路径，第一步用 `set -o noclobber` 排他建「已开工」标记，标记已在就什么都不做（`remind.sh` 重起时可能重复提醒）。

- 提醒只活在当前会话里，告诉用户会话不能关、机器不能休眠。

```

**出处 `.claude/kb/tooling.md:102-112`（整段抄，未转述）**

```markdown
## Rust 工具链

| 项 | 值 |
|---|---|
| 版本 | rustc 1.98.0 / cargo 1.98.0 |
| 安装方式 | rustup，`--profile minimal --default-toolchain stable`，装在宿主 `~/.cargo` |
| 额外目标 | `x86_64-unknown-linux-musl`，实验二进制（`scripts/experiment-apparatus/` 下的包）编成静态的在虚机的 busybox initramfs 里跑；checker 档的真设备二进制 `new_pool_file_creation_on_device` 不走它（checker 档默认编 RocksDB 的 C++，本机没有 musl 的 C/C++ 编译器），改用 `--target x86_64-unknown-linux-gnu` 加 `+crt-static` 编成 static-pie，命令在 `.claude/gate.d/checker-tier-qemu-device-streams.sh` |
| 组件 | `rustfmt` + `clippy`（`--profile minimal` 不含它们，要另装） |
| **PATH** | rustup 的 shim 软链在 PATH 上的用户级可执行文件目录里（cargo / rustc / rustup / rustdoc / rustfmt / cargo-fmt / cargo-clippy）；门禁与脚本用 `bash -c` 起，不读 shell 的启动文件，PATH 不靠改 dotfile |
| release 溢出检查 | 树根 `Cargo.toml` 与 `scripts/experiment-apparatus/Cargo.toml` 的 `[profile.release]` 都设了 `overflow-checks = true`；profile 只认工作区根，写在成员 manifest 里不生效 |

```

**出处 `.claude/kb/tooling.md:113-140`（整段抄，未转述）**

```markdown
## RocksDB 与 snappy 的预编库：每台主机编一次，取值只走进程环境

librocksdb-sys 的 C++（`librocksdb.a` 与 `libsnappy.a`）每台主机只编一次，各编译目录经环境变量链这一份，不在每个编译目录里现编。工具是 `scripts/hosts/rocksdb-prebuilt.py`，用法写在它的文件头（它不认 `--help`，退出码 2）。子命令都在仓根跑，`--root R` 不给就是这份脚本所在的仓。

| 子命令 | 做什么 |
|---|---|
| `key` | 打这台主机此刻的键（各组成）与预编库目录，不编 |
| `env` | 预编库在、对得上时打四行 `export`（给 `eval` 用）；不在退 1、标准输出为空 |
| `build` | 预编库不在就编一次，持锁：几个进程同时来只编一次，后来的等锁之后看到编好了直接退 0；起真 cargo，是重活，经 `bash scripts/hosts/capped.sh <N>` 起 |
| `install` | `build`，再写这台主机的三处环境来源、改用户级 systemd 管理器此刻的环境，最后跑 `check`；同样是重活 |
| `check` | 判这台主机：两个工作区（`crates/singlefs-checker-tier/Cargo.toml` 与 `scripts/experiment-apparatus/Cargo.toml`）的依赖对得上、预编库在且对得上、三处环境来源与此刻进程环境的取值都与预编库一致、cargo 配置里没写这四个变量 |
| `audit-target <编译目录>…` | 判编译目录：librocksdb-sys 构建脚本那一次拿到的取值是不是这台的预编库、`out/` 里有没有现编过的 C++、同一把键下见过几种取值；一份构建脚本记录都没找到退 77（本次未判） |
| `entry [--cargo C]` | 起 cargo 的入口开头调的轻判（谁调、取值缺了怎么办写在「起 cargo 的入口」那一段） |

**预编库放哪**：`${SINGLEFS_ROCKSDB_PREBUILT_STORE:-~/.local/share/singlefs/rocksdb-prebuilt}/librocksdb-sys-<版本>/<键的前 16 位>/lib/{librocksdb.a,libsnappy.a}`；同目录 `manifest.json` 记键的各组成与两份库的 sha256，`complete` 最后写（有它才算编好）；不放开机清空的目录。`SINGLEFS_ROCKSDB_PREBUILT_STORE` 只给自证换临时目录用。键由两个工作区的 rocksdb 依赖写法、两份 `Cargo.lock` 里相关包的版本与 checksum、`rustc -vV`、c++ / cc / ar 的真实路径与版本、交给 C++ 的指令集、编法组成，哪一样变了键就变、要重编。

**取值从哪来**：四个变量 `ROCKSDB_LIB_DIR`、`ROCKSDB_STATIC`、`SNAPPY_LIB_DIR`、`SNAPPY_STATIC` 的取值写在三处环境来源里，都由 `install` 写：

| 位置 | 作用 |
|---|---|
| `~/.config/singlefs/rocksdb-prebuilt.env` | 唯一的取值来源：四行 `export`，外加 `unset ROCKSDB_COMPILE`、`unset SNAPPY_COMPILE`；别手改，改了重跑 `install` |
| `~/.bashrc` 最前面一段（三行，`# >>> singlefs rocksdb-prebuilt >>>` 到 `# <<< singlefs rocksdb-prebuilt <<<`） | `source` 那份 env 文件；放在交互判断之前，所以交互 shell、从它起的会话与 ssh 起的非交互命令都读得到 |
| `~/.config/environment.d/60-singlefs-rocksdb-prebuilt.conf` | 用户级 systemd 管理器起的单元读（`scripts/hosts/detached-run.sh` 起的长活）；`install` 另跑 `systemctl --user set-environment`，让此刻在跑的管理器当场换上 |

**四个变量只走进程环境**：不写进 `.cargo/config.toml` 或别的哪一层 cargo 配置的 `[env]`（写了 `check` 判红；理由在 `.cargo/config.toml` 开头的注释）。`ROCKSDB_COMPILE`、`SNAPPY_COMPILE` 一律不设，设成 1 就在编译目录里现编。

**起 cargo 的入口**：门禁阶段、崩溃与变异驱动、`merge-and-verify.sh`、`admission.py` 交的命令、第二台那一侧，开头都调 `entry`。它只判三样：这台装了（预编库在、对得上，不算两份库的 sha256）、此刻进程环境的四个取值与预编库一致、没设 `ROCKSDB_COMPILE` 与 `SNAPPY_COMPILE`；不判三处环境来源（那是 `check` 的事）。取值缺了或对不上，入口当场拒、一个 cargo 都不起，逐项写出出路。出路：此刻进程没有取值的，开新会话（`install` 之后起的登录 shell 读 `~/.bashrc` 最前面那一段），或在命令前 `eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"`；这台还没装的先 `bash scripts/hosts/capped.sh <N> python3 scripts/hosts/rocksdb-prebuilt.py install`。要编的那棵树里没有依赖 rocksdb 的工作区时（编不到 librocksdb-sys，门禁样本与自证的小仓多是这样），`entry` 不判、退 0。

```

**出处 `.claude/kb/tooling.md:141-155`（整段抄，未转述）**

```markdown
## 虚机 benchmark harness（E7（离线索引 harness） 用）

| 项 | 值 |
|---|---|
| 脚本 | `bash scripts/devices/vm-bench.sh <静态二进制>` / `--selftest` |
| 它是什么 | 项目自己的虚机装置：给虚机挂 virtio 盘（稀疏文件落 `TMPDIR`）并把二进制塞进 initramfs |
| 完整性闸 | 被测程序收尾行报 `name=done emitted=N`，宿主抓到的条数必须等于 N，否则整轮作废 |
| 默认规格 | 2048M 内存 / 4 vCPU / 4096M 盘 / 900s 超时，均可用环境变量覆盖 |

- 抓结果行不锚定行首（控制台首行前面有 BIOS 的转义序列），条数靠完整性闸核。
- 虚机里裸设备的顺序写吞吐不稳定，不作为任何判断的输入。
- 跑批脚本作废一轮时，把该轮输出的尾部一并记下。

---

```

**出处 `.claude/kb/tooling.md:156-192`（整段抄，未转述）**

````markdown
## 造几何量不同的设备：不需要真硬件

本机具备造异构设备池的全部手段，[decisions.md](decisions.md) D12（目标介质） 已定项 3「设备级几何量该放哪、异构池支不支持」可以实测。

### QEMU 路线（推荐：无需 root，直接接现有 harness）

`zoned.zone_size` 与 `logical_block_size` 挂在 **`nvme-ns`（命名空间）** 上，不是挂在控制器上，同一个控制器下的多个命名空间可以有不同的几何量。

已实测通过（QEMU 8.2.2，退出码 0，三个设备全部构造成功）：

```
-device nvme,serial=A,id=nv0
-device nvme-ns,drive=d1,bus=nv0,nsid=1,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d2,bus=nv0,nsid=2,zoned=true,zoned.zone_size=128M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d3,bus=nv0,nsid=3,logical_block_size=512,physical_block_size=512
```

一条命令同时造出：**异构 zoned 池**（64M vs 128M zone）、**512e/4Kn 混用**、**zoned 与常规混用**。

- `logical_block_size` 必须配 `physical_block_size`，只给前者会被拒：`logical_block_size > physical_block_size not supported`。
- 来宾内核看不看得到三份不同的几何没有验过：要进来宾读 `/sys/block/*/queue/{logical_block_size,chunk_sectors}` 与 `blkzone report`。

### 宿主侧路线（要 root，胜在不用启虚机）

| 手段 | 造什么 | 关键参数 |
|---|---|---|
| `null_blk` | zoned 设备，内存后端 | `zoned=1 zone_size=<MB> zone_capacity zone_nr_conv zone_max_open`。模块参数是全局的，**每设备各自的几何要走 configfs**（`/sys/kernel/config` 本机已挂载） |
| `zloop` | **文件后端**的 zoned 设备 | 模块在 `drivers/block/zloop.ko.zst`，本机有 |
| `scsi_debug` | 任意扇区大小 + ZBC | `sector_size=<bytes> zbc=host-managed zone_cap_mb zone_nr_conv` |

运行内核 6.17.0-1028-oem 的相关配置：`CONFIG_BLK_DEV_ZONED=y`、
`CONFIG_BLK_DEV_NULL_BLK=m`、`CONFIG_BLK_DEV_ZONED_LOOP=m`、`CONFIG_SCSI_DEBUG=m`。

⚠️ 本机唯一的真实块设备是 `nvme0n1`（3.6T，512/512，非旋转）——
**没有真的旋转盘、没有真的 4Kn 盘、没有真的 zoned 盘**，所有相关数字都只能是模拟出来的，
引用时必须带这个口径。

````

**出处 `.claude/kb/tooling.md:193-197`（整段抄，未转述）**

```markdown
## `scripts/mutation/mutate.sh` 的结局分类

- 结局：`✅ [抓到]`（红了）/ `❌ [没红]`（测试盲区）/ `⏭ 变异导致编译失败（无效变异，不计入盲区也不算命中）`；另有 `⏱ [超时]`、`🧱 [撞内存]`、包装排不上、被内存总上限挤掉、显存不够与锚点失效（`⚓ [锚点腐化]`，原文命中不是恰好一次的那一条不跑、不入判定库，别的条照跑，整轮退 3）几类，各自单列（全部结局见 `scripts/mutation/mutate.sh` 文件头与 `--selftest`）。
- 改了判定工具，拿已知答案的整套语料回归一遍，不只跑新加的那几条。

```

**出处 `.claude/kb/tooling.md:198-211`（整段抄，未转述）**

```markdown
## 几个会话共用一个仓时怎么暂存与提交

门禁红了先分清红的是谁的改动，再动手；不为了让门禁变绿去改别人在飞的文件。规矩在上游 `session-wrapup.md` 第 4 节，本仓的做法：

| 事 | 怎么做 |
|---|---|
| 只暂存自己的块 | `scripts/gate/stage-mine.py --match 我的标记 --foreign 别人的标记 文件…`，先 `--dry-run` 看它挑了哪几块 |
| 暂存区也是共用的 | 暂存前先确认 `git diff --cached --name-only` 为空；暂存完、提交前再核一遍名单与逐文件 cached diff，然后不带路径地 `git commit` |
| 不要 `git commit -- <路径>` | 带路径时 git 提交的是这些路径在工作区里的内容，不是暂存区 |
| 同一行两边都改了 | stage-mine 报「进暂存区的一块里有命中 --foreign 的行」时一行切不开：对方在那份文件里只做了可逆替换（改名）的，拿工作区版本把对方的替换反做一遍、与 `git show HEAD:文件` 比 diff 确认只剩自己的，再 `git hash-object -w` + `git update-index --cacheinfo 100644,<sha>,<路径>` 放进暂存区，工作区不动 |
| 在 worktree 上提交 | 在 worktree（HEAD 上）用脚本重放这次要提交的全部改动 → 与主工作区逐文件比对（不同的部分应当全是对方的；比对时 `git -c core.quotePath=false`，否则中文路径被转义、`cmp` 全报不同）→ 在 worktree 跑整道门禁并提交 → 主仓里 `git update-ref refs/heads/master <新提交> <旧 HEAD>` 再 `git read-tree <新提交>`，工作区不动、对方的改动原样留着 |
| 看清哪些不是这次要提交的 | `git -c core.quotepath=false status --short`，中文路径默认被转义成八进制，grep 中文关键词会漏 |
| 两边改动交织在同一文件 | 一个提交带全部，消息里写明（拆开提交会让门禁 doc-registries 的 experiment-refs 格与门禁 doc-experiments 的 experiment-orphans 格判红） |

```

**出处 `.claude/kb/tooling.md:212-217`（整段抄，未转述）**

```markdown
## 复跑与变异脚本要在 `scripts/experiment-apparatus/` 下跑

- `scripts/mutation/mutate.sh` 在 `scripts/experiment-apparatus/` 下跑：`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh <bin 名> <源文件> ../mutation/tables/<变异表>`。
- `replay.sh` 只认带 E 的实验号（`E` 加数字），裸数字退出码 2。
- `replay.sh` 每一趟编进它在 `${TMPDIR:-/tmp}/singlefs-replay-build-<进程号>-XXXXXX/target/` 自己建的编译目录、跑完删（调用方设了 `CARGO_TARGET_DIR` 的照它编、不删），研究二进制从 `$REPLAY_RELEASE_DIRECTORY`（`$CARGO_TARGET_DIR/release/`）起；`scripts/experiment-apparatus/target/` 不再有复跑编出来的二进制。

```

**出处 `.claude/kb/tooling.md:218-237`（整段抄，未转述）**

```markdown
## 批量立 / 定分项时门禁会红的形态

一次立或定一批分项之后，提交前门禁里这几格会按下列形态红，写的时候照「写法」一列写：

| 门禁 | 会红的形态 | 写法 |
|---|---|---|
| doc-registries 的 first-txn-hooks 格（三份文件挂钩） | 按 `\| N \|` 行切未定项条目，最后一行把后面所有 `#### 未定项 N` 正文吞进自己那一条；正文里写「改新池新建文件的字节：是（」，判「否」的最后一行也被当成「是」 | 索引行用规范形态 `改新池新建文件的字节：是（日期，依据：…）`，正文小节写成 `**改新池新建文件的字节**：是（…）` |
| doc-decisions 的 item-ref-status 格（引用归属） | 行内裸的「已定项 k」归属给同一行最近的 `D<n>（简称）` | 新立分项之后跑 doc-decisions 的 item-ref-status 格，把翻掉的裸引用补上决策名；决策正文里引别的决策的分项，同一行里要重复写自己的决策名 |
| doc-lint | `F` 加数字这类审核编号被当成没有登记位的编号；简称后面加逗号注判「名字不符」 | 写「发现 1」「提议 2」；括注另起一对括号，写成「编号（简称）（条款已写）」，不塞进简称的括号里 |
| doc-registries 的 table-shape 格（欠账表形状） | 新的六列欠账行接在文件末尾，而那里是「已还清」那张 4 列表；欠着那张里一格开头写「已还（…）」或「销账」；欠着那张中间夹一个空行 | 接在第一张表最后一行后面：第一个五竖线的 `\|---\|` 之前、最后一行以 `\| C` 开头且竖线 ≥ 7 的行；还清了就按四列挪进已还清表，只还一部分的开头写「已还一半」「决策已定、检查仍欠」「前置已还」；表中间不留空行 |
| doc-registries 的 segment-registry 格（段序列登记表） | `--selftest` 要「mkfs 种根」那一行的段序列串在整份 `layout/01-first-txn.md` 里只出现一次，同一串在别处再写一遍就红 | 别处引用写成「4 + 1 + 1 + 1 + 4」带空格的写法 |
| 门禁 doc-decisions 的变更史三格（entry-added、shape、history-loss） | 「（其N）」、快查两行与节顶现状行 | 写法与判法在 `.claude/rules/changelog-format.md`；数与编号有没有丢由 history-loss 格判，整理变更史时暂存前自己先跑 `scripts/kb/history-consolidate.py loss-check` |
| code-source-discipline 的 format-constants 格（格式常量） | kb 标记改值后，`scripts/experiment-apparatus/**/*.rs` 与 `crates/**/*.rs` 里每一份同名 `const` 都要跟（`research/` 下只剩冻结证据，扫它但由 `.claude/format-constants-exclude` 逐份放行）；`stale=` 列的旧值串不许再出现在 kb 正文 | 别把仍在别处合法出现的串（`base = 95`）加进 `stale=` |
| clippy `shadow_unrelated` | 同一个测试函数里第二次 `let reader = …` / `let segments = …` 判红，rustfmt 重排之后用正则替换还会漏改后半段 | 每个读者 / 段 / 报告起不同的名字（`extent_reader`、`warm_up_segments`、`control_report`） |
| naming-lint | 「字母 + 数字」当单字母：`read_u48` 判红，测试名以 `a_` 开头判红；它扫全仓的 .rs，连 `research/prompts` 里云端腿写的模型源码也扫 | 写 `read_six_byte_unsigned`；`research/prompts` 在 `.claude/naming-lint-exclude`（证据目录，不回收），腿的模型写到别的目录照查 |
| checker-tier-crash-point-replay（崩溃点重放全量） | 用例标 `#[ignore]`、住 `crates/singlefs-checker-tier/tests/`（D13（验证路线） 已定项 15），`cargo test -p singlefs-checker-tier` 报几条 ignored 是正常的；harness 档 `cargo test -p singlefs-harness` 跑不到它们 | 门禁的 `--full` 在 release 下逐条照复用判定跑（快档不跑全量）；全量在跑时换 `CARGO_TARGET_DIR` 就能同时编别的测试 |
| 变异证明 | rustfmt 之后多行调用的整串匹配「没命中」 | 按「找到含关键串的行、删到以 `);` 收尾的那一行」做；改完复原并 `assert` 文件逐字节等于原文 |
| checker 新判定 | 只在对象已经过了同一关之后才判的判定永远判不红 | 每条新判定配一份坏镜像（`checker_known_bad_images.rs` 的形态），不许只看干净镜像全绿 |
| 批量定点替换 | 逐条写盘会在一处锚点不中时留下半套改动 | 用 `scripts/kb/replace-batch.py`：全部替换先在内存里核「恰好命中 1 次」，全过了才写盘 |

```

**出处 `.claude/kb/tooling.md:238-248`（整段抄，未转述）**

```markdown
## 本机 Claude Code 会话里的 `grep` 与 `find` 不是 GNU 那两个

`type grep`、`type find` 显示两个都是 shell 函数：`grep` 底下是 ugrep，`find` 底下是 bfs（报错分别以 `ugrep: error`、`bfs: error` 开头）。两处行为与 GNU 不同：

| 写法 | 结果 | 改成 |
|---|---|---|
| `grep -o` 带 `.\{0,90\}` 这种有界重复，作用在中文行上 | `exceeds complexity limits`，退出码 2，一行都不输出 | 用 python 按下标切：`line[max(0, i - 90):i + 90]` |
| `find 目录 -newermt '-40 minutes'` | `Invalid timestamp`，不认相对时间 | 写 ISO 日期：`-newermt '<YYYY-MM-DD>'` |

两种失败的样子都是「这条命令没给出结果」，接在管道或 `||` 后面时不要读成「没有命中」。

```
