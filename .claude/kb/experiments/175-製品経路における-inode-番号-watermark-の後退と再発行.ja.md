## E175 製品パス上のinode番号水位ロールバックと再発番 — 実行済み（判断に十分）

[中文](175-产品路径上inode号水位退回与重发号.md) · [English](175-inode-number-watermark-regression-and-reissuance-on-the-product-path.en.md) · **日本語**

<!-- doc-lint:not-numbers W1 W2 W3 P1 P2 Q1 Q2 Q3 Q3b Q4 Q5 Q6 Q7 Q9 Q10 Q11 S1 S2 S3 V1 V2 V3 V4 V5 V717 F1 F2 F3 F4 F5 F6 F7 F8 F9 F3a F3b F4 -->

```text
本ページのローカル方式名と記号は小文字（a1、a2、a3、a4 ……）で記す。事前登録、装置、成果物、コマンドでは対応する大文字で記し、全体登録の番号と同形である。kb 本文で小文字にするのは番号と区別するためだけであり、成果物を読む際は大文字に対応させる。
```

事前登録 `e175-preregistration.md`（2区間を実行したときの sha256 `d68c9d2229bcf3800c5efd36c9194e6af9684b193a172258b58846474afae805`）。問いは読み B に固定し、「再発行」は同一時間線上のみを数える。第12節「修訂」は2回：修訂1は実行員による8つの読みの固定と装置の書き方、修訂2は設計員による「2区間を同一凍結コピーで実行」と4ジオメトリ点。判定基準、閾値、方式、測定量の取捨は変えていない。`m3-3-5-forks.md` の分岐 W は C479 (既発行inode番号の再利用が現状再現不能)に残る「活記録がその場で置換される」形態を問う。反転観測は「製品経路上で、テスト専用 watermark スイッチを付けずに inode 番号 watermark を後退させて再発行を作る履歴があるか」。判断十分条件はランダム履歴とロールバックを伴う履歴を登録数だけ実行し、再発行と I-9.29 (同番号に単一の出生世代のみ許可)の不合格回数を数えること。1回あれば判断に十分。

**本ページの数値はすべてリポジトリコピー装置を1つの凍結コピーで実行した結果**：装置は登録第5節の配置に従い、凍結コピー `~/.local/share/singlefs/work/e175-frozen-crates-2/repo/` の `crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs` に配置。主作業領域の `crates/` は変更しない（形式化ラインの凍結期間）。装置ソースは `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs` にそのまま保存（成果物ディレクトリ外）。凍結コピー `code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb` は2026-10-08、ルートリング ライン第10zバッチの第10・第11ブロック統合と統合後検証の全成功後にコピー。HEAD `31e47115471ba5502a4531d25dffcc921be36899`（`baseline-frozen-crates.out` 先頭行）。kb の他箇所でこの数値を引用する際は「リポジトリコピー装置、凍結コピー」と明記する。装置を `crates/` に入れるか、入庫装置で再実行するかはメイン agent が決める。

**現状**：両区間をこの凍結コピーで完了。第1区間は D 族の値固定、F3a の対象を絞った網羅、F2 のロールバック付きランダム履歴。第2区間は既存 F1 ランダム履歴4群と第8節ジオメトリ4点 G-long/G-small/G-draw/G-long-F1。P/V1/V2 を登録どおりそれぞれの規模で実行。第3区間 F3b/F4 は付随項目で W の判断が十分となった後は未実行。

- **旧コード上の数値**：成果物ディレクトリの2サブディレクトリは旧コードの成果物で、W 判定にも測定量の合計にも含めない。`superseded-b000407a/` は前の凍結コピー（`code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`、ルートリング ライン第10zバッチ第9ブロック統合後、一覧は内部の `baseline-frozen-crates.sha`）上の2区間37ファイル。登録修訂2第4項の形でそのまま移し、移動前後の37件の sha256 は行ごとに一致。`superseded-old-crates/` はさらに古い `crates/`（一覧 `superseded-old-crates/old-crates.sha`、336行）上の第1区間19ファイル。2凍結コピーの一覧は10ファイルが異なり追加・削除0、その中に `crates/singlefs-core/src/mount.rs` がある（`placements_in_the_accounts_of_newer_readable_roots`、`rebuilt_allocator`、モジュール内単体テストの変更。装置に `singlefs-core` がコンパイルされる）。新しい26結果ファイルの E7RESULT 行は `elapsed_ns` 除去・ソート後、`superseded-b000407a/` の同名ファイルと個別に一致。2集計も `code_baseline` 以外は行ごとに一致（比較コマンドは「測定値」）。
- **製品方式 P**：登録済み17105履歴（D 8、F3a 16105、F2 256、F1 4群各128、ジオメトリ4点64 + 64 + 64 + 32）で同一時間線上の再発行 Q1 は0回、I-9.29 不合格0履歴、判定不能0ステップ（F1 4群と G-long-F1 に適用外ステップあり、F3a/F2/G-long/G-small/G-draw は全ステップ成立）、watermark 後退 Q3 は0ステップ、破棄された時間線を跨ぐ再発行 Q3b は0回。どの不変条件もどのステップでも不合格にならない。
- **経路に到達**：発行済み番号を越えるロールバック後に同じ時間線で inode を再作成した窓 Q5 は F3a 7870、F2 1062、合計8932で登録50を超える。第2区間には F1 4群572、ジオメトリ4点1196が追加。F1 4群で `PublishFirstFile`（番号1の再発行への唯一の経路）を抽出：F1-broad 197回、F1-reuse 68、F1-deleting 12、F1-newest 197、G-long-F1 203。
- **陽性対照**：V1（ロールバック時に対象版 watermark を使用）と V2（パブリッシュが新規作成数を加算しない）は実行した全族で再発行と I-9.29 不合格を生成。R-V1 は F3a/F2/F1 4群/G-long で合格、R-V2 は F3a/F2/F1 4群で合格（ジオメトリ点は登録どおり G-long の V1 のみ実行）。登録第10節 F1/F2 は非発火。
- **停止条項**：S3 の2経路の破棄判定不一致0回、S1 と S2 の不一致（F3 の2形態）0回、Q11（発行番号が前版 watermark と異なる）0回、条項アンカー a1–a3 と独立アンカー a4–a7 はすべて一致。いずれも非発火。
- **W**：登録第11節「第2区間完了後、F1/F2/F7/F8 が非発火なら W は判断に十分」に従う。登録済み17105件の製品方式履歴で同一時間線上の再発行は生成されず、I-9.29 不合格の履歴もない。これは「登録履歴で生成されなかった」であり、「生成できない」ではない。

### 実施内容

- **3つの読み**（登録第6節）：S1 台帳：各ステップ後に `singlefs_core::recovery::readable_roots` で読める root を列挙し、同じ root key は `rebuild_version` で1回のみ再構築。その版の会計 watermark と inode ツリーの全（番号、誕生世代）を読み、番号ごとに「見た誕生世代 → 保持した root」を記録。S2：各ステップでイメージに `singlefs_checker::walk::check_pool_image` を呼び、不合格でも停止せず、I-9.29 (同番号に単一の出生世代のみ許可)、I-9.6 (水位が二か所の最大番号より大きい)を成立／適用外／一部未判定／違反の4欄で数える。S3：クラッシュ復旧により破棄された root の2経路。経路1は最新 root のインスタンス表を `publish_is_abandoned_by_the_instance_table` で判定。経路2はステップ台帳（各マウント時に作成。新インスタンスは直前のモデル ルートリング 内の最新 root に接続し、同じ旧インスタンスで txg がより大きい root を破棄と記録）。
- **方式**：P は凍結コピー `crates/` をそのまま使用し、コピー時の主作業領域とバイト一致（`baseline-frozen-crates.out` の `main_before_copy_equal=true main_after_copy_equal=true`）。V1 は `crates/singlefs-core/src/mount.rs` のロールバック時 watermark を対象版のものに変更。V2 は `crates/singlefs-core/src/transaction.rs` の `} + new_inodes_of_this_publish;` を `} + 0;` に変更。V717（`crates/mutations.tsv` の「実装3（D23 確定項目14『watermark』）：ロールバック時の inode 番号 watermark をリング内で読める root のみから取り、メモリ上の現行版と max を取らない」）は登録上 Q6 > 0 のときのみ実行。両区間の全族で Q6 = 0 のため「P と同出力（Q6 = 0 から推論）、未実行」と記録。3アンカーの行番号と命中数は `baseline-frozen-crates.out` の `name=mutation_anchor` 3行。パッチは `patch-V1.diff`、`patch-V2.diff` に原文保存。各方式の適用前と復元後にコピーのルートで `baseline-frozen-crates.sha` を確認（`name=arm_check` 9行、`listed_mismatch=0 extra_files=0`）。各方式のバイナリは1回ずつビルドし2区間で共有（`name=arm_binary` 3行）。
- **実行**：`HistoryExecution { per_step_checker: Skipped, device_width: FourGibibytes（G-small 换 UnitAreaOf384Slots）, space_admission: JudgedByTheFormula }`。`set_inode_number_watermark_source` を呼ばず `SharedFaultPlan` を追加しない。5ワーカースレッド（派遣プロンプトの上限）が番号昇順で仕事を取得し、番号順にマージ。単一スレッドと逐語一致（クイック層 `one_thread_and_three_threads_give_the_same_lines`）。各履歴の実時間は装置プロセス内で計測（結果行 `elapsed_ns`）。中断後は同じコマンドで再開し、既存行の番号を飛ばす。各結果ファイル先頭は `# code_baseline=…` で、再開時にヘッダーが異なれば拒否。集計は全入力が同一 `code_baseline` であることを要求し、各集計行にも付加する（修訂2第2項）。
- **試行**（修訂2第5項）：F2 P の最初の8シード488ステップは平均281.86ミリ秒／ステップ／スレッド、第1区間約54分と外挿。G-long P の最初の4シード604ステップは272.74ミリ秒／ステップ、第2区間約45分と外挿。両方2時間未満なので分割しない（`trial-f2-P-first-8-seeds.out`、`trial-g-long-P-first-4-seeds.out`。結果行の `elapsed_ns` 合計を `steps_observed` 合計で除算）。

### 再実行

```
# 冻结副本（拷 Cargo.toml、Cargo.lock、.cargo、crates 四样；主工作区 crates/ 不动），核清单再放装置
F=~/.local/share/singlefs/work/e175-frozen-crates-2/repo; R=$PWD/research/results/e175-product-path-inode-watermark-regression-2026-10-08
(cd $F && sha256sum -c --quiet $R/baseline-frozen-crates.sha)
cp $PWD/scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs $F/crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs
# 每一臂：施加补丁（P 不施加；V1、V2 照 $R/patch-V1.diff、$R/patch-V2.diff）、编、拷出二进制、还原，再核一次清单
ROOT=$PWD; (cd $F && CARGO_TARGET_DIR=<编译目录> bash $ROOT/scripts/hosts/capped.sh 5 cargo test --release -p singlefs-harness --test e175_product_path_inode_watermark_regression --no-run)
E175_ARM=P bash scripts/hosts/capped.sh 5 bash scripts/hosts/run-with-memory-cap.sh 8G <测试二进制> --test-threads 5
E175_ARM=P E175_FAMILY=F2 E175_RESULT_FILE=$R/f2-P.out E175_CODE_BASELINE=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb E175_WORKER_THREADS=5 E175_TIME_BUDGET_SECONDS=170 \
  bash scripts/hosts/capped.sh 5 bash scripts/hosts/run-with-memory-cap.sh 8G <测试二进制> --ignored --exact run_one_family_on_one_arm_into_the_result_file
# 族名：D、F3a、F2、F1-broad、F1-reuse-after-raising-the-floor、F1-deleting-files-of-several-lower-leaves、
#       F1-newest-system-configuration-slot-reading-zeros、G-long、G-small、G-draw、G-long-F1
# 汇总（两段各一份）
E175_SUMMARY_INPUTS=<逗号分隔的结果文件> E175_SUMMARY_FILE=<新文件> E175_CODE_BASELINE=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb <测试二进制> --ignored --exact summarize_the_result_files
# 登记的复跑（scripts/experiment-apparatus/replay.sh 一行 E175|@driver_e175：拷今天的 crates/ 进副本、放进 device-source.rs、跑 D 族 P 臂）
bash scripts/experiment-apparatus/replay.sh E175
```

### 成果物一覧

`research/results/e175-product-path-inode-watermark-regression-2026-10-08/` のトップレベルは37ファイル（`ls -1 | grep -v superseded | wc -l` は37）。すべて凍結コピー `code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`（2026-10-08、ルートリング 第10zバッチ第10・11ブロック統合と検証成功後にコピー）上のもの。別に `superseded-b000407a/` 37ファイル、`superseded-old-crates/` 20ファイルがある（旧コードの数値で判定には含めない）：

| ファイル | 内容 |
|---|---|
| `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs` | 装置ソース。成果物ディレクトリ外の `scripts/experiment-apparatus/` に移動済み。両区間実行時の sha256 は `310c60a51a7a0c5cc015b070006d7d48ae0eb09d8a581ec8acba9d6bfc185fca` で同一。コミット `6aa74a765`、`f778192d3` と 2026-10-23 のディレクトリ名変更がスクリプトパスを変更し、現在の sha256 は `99f4433c0bfb1d009081bf78c0607df31fb91912edeb5a622a7c2d1e02923735`。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/baseline-frozen-crates.sha` | 装置配置前の凍結コピー一覧。340行、パスはコピーのルート相対。sha256 `48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb` が `code_baseline`。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/baseline-frozen-crates.out` | `name=frozen_copy`（コピー日、バッチ、HEAD、diff/status 2行の sha256、ファイル数、コピー前後の主作業領域一覧とコピーの一致）、`name=mutation_anchor` 3行、`name=arm_check` 9行、`name=arm_binary` 3行、`name=main_drift_after_copy` 1行。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/patch-V1.diff`、`e175-product-path-inode-watermark-regression-2026-10-08/patch-V2.diff` | V1、V2 方式に適用するパッチ。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/quick-tests.out` | 3方式のクイックテストと3つの破壊スイッチの出力。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/trial-f2-P-first-8-seeds.out`、`e175-product-path-inode-watermark-regression-2026-10-08/trial-g-long-P-first-4-seeds.out` | 実時間を外挿する2回の試行。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/d-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/d-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/d-V2.out` | D 族の値固定、3方式。 |
| `e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` | `driver_e175` 形式の出力（凍結コピーの D 族 P を4スレッドで再実行、E7RESULT と末尾行）。`replay.sh` が比較し、`elapsed_ns` を除くと `d-P.out` の E7RESULT 行と行ごとに一致。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f3a-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f3a-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f3a-V2.out` | F3a、3方式。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f2-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f2-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f2-V2.out` | F2、3方式。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/summary-segment-one.out` | 第1区間集計：9つの族 × 方式結果ファイルの合計、重複と衝突、イベント数。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-V2.out` | F1 `BROAD`、3方式。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-V2.out` | F1 `REUSE_AFTER_RAISING_THE_FLOOR`、3方式。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-V2.out` | F1 `DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`、3方式。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-V2.out` | F1 `NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`、3方式。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/g-long-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-long-V1.out` | ジオメトリ点 G-long（F2 の重み、150ステップ）、P と V1。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/g-small-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-draw-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-long-f1-P.out` | ジオメトリ点 G-small（`UnitAreaOf384Slots`）、G-draw（`RecentUniformOrBeyond`）、G-long-F1（`BROAD`、150ステップ）、P。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/summary-segment-two.out` | 第2区間集計：17件の族 × 方式結果ファイル。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/superseded-b000407a/` | 前の凍結コピー（`code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`、ルートリング 第10zバッチ第9ブロック統合後）上の2区間37ファイル。元の名前・内容を保持しトップレベル37件と1対1で同名。コピーの一覧は `superseded-b000407a/baseline-frozen-crates.sha`。移動前後の37件の sha256 は行ごとに一致。 |
| `e175-product-path-inode-watermark-regression-2026-10-08/superseded-old-crates/` | 旧コード上の第1区間19ファイル（元の名前・内容を保持。トップレベルと同名の16件のほか `superseded-old-crates/baseline.out`、`superseded-old-crates/baseline-segment-two.out`、`superseded-old-crates/trial-f3a-P-8-long-sequences.out`）と、その `crates/` 一覧 `superseded-old-crates/old-crates.sha`。移動前後の19件の sha256 は行ごとに一致。 |

### 測定値（コマンドと元の出力）

成果物ディレクトリで実行し、2集計から族 × 方式ごとの測定量を取得（`sum_` は履歴間の合計、`segments_q2_one_birth_violated` は I-9.29 (同番号に単一の出生世代のみ許可)が不合格となった履歴数、`sum_i9_29_holds`、`sum_i9_29_not_applicable`、`sum_i9_29_partially_judged` はそれぞれ成立・適用外・判定不能のステップ数）：

```
for s in one two; do echo "== $s"; head -1 summary-segment-$s.out; awk '/name=summary/{o=""; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]~/^(family|arm|histories|complete|duplicates|conflicts|ending_completed|sum_q1|sum_q3b|segments_q2_one_birth_violated|sum_i9_29_holds|sum_i9_29_not_applicable|sum_i9_29_partially_judged|sum_q9_still|sum_q9_gone|sum_q3_steps|max_q3_peak|sum_q4|sum_q5|sum_q6|sum_q7|q10_segments_issuing_at_least_two|sum_q11|sum_s1_unreadable|sum_s3_disagreements|sum_f3_still_but_holds|sum_f3_violated_without_two_births)$/) o=o" "$i}; print substr(o,2)}' summary-segment-$s.out; done
```

```
== one
# code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb
family=D arm=P histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=45 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=4 sum_q5=3 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=D arm=V1 histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=42 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=3 sum_q11=0 segments_q2_one_birth_violated=3 sum_q3_steps=7 sum_q3b=0 sum_q4=4 sum_q5=3 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=3 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=2
family=D arm=V2 histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=23 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=13 sum_q11=0 segments_q2_one_birth_violated=7 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=13 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F2 arm=P histories=256 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=231 ending_completed=256 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=15616 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=1748 sum_q5=1062 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F2 arm=V1 histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1769 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=336 sum_q11=0 segments_q2_one_birth_violated=53 sum_q3_steps=1476 sum_q3b=0 sum_q4=311 sum_q5=195 sum_q6=0 sum_q7=280 sum_q9_gone=5 sum_q9_still=331 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=7
family=F2 arm=V2 histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1265 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=696 sum_q11=0 segments_q2_one_birth_violated=53 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=696 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F3a arm=P histories=16105 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=16105 ending_completed=16105 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=159440 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=11641 sum_q5=7870 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F3a arm=V1 histories=1464 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=1464 ending_completed=1464 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=11788 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=975 sum_q11=0 segments_q2_one_birth_violated=567 sum_q3_steps=2243 sum_q3b=0 sum_q4=798 sum_q5=577 sum_q6=0 sum_q7=168 sum_q9_gone=0 sum_q9_still=975 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=4
family=F3a arm=V2 histories=1464 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=1464 ending_completed=1464 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=4392 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=3990 sum_q11=0 segments_q2_one_birth_violated=1464 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=3990 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
== two
# code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb
family=F1-broad arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=49 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3638 sum_i9_29_not_applicable=330 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=39 sum_q5=13 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-broad arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=821 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=12 sum_q11=0 segments_q2_one_birth_violated=4 sum_q3_steps=43 sum_q3b=0 sum_q4=11 sum_q5=4 sum_q6=0 sum_q7=9 sum_q9_gone=0 sum_q9_still=12 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=5
family=F1-broad arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=637 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=90 sum_q11=0 segments_q2_one_birth_violated=15 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=90 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-deleting-files-of-several-lower-leaves arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=110 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3910 sum_i9_29_not_applicable=58 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=615 sum_q5=526 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-deleting-files-of-several-lower-leaves arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=27 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=556 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=103 sum_q11=0 segments_q2_one_birth_violated=26 sum_q3_steps=327 sum_q3b=0 sum_q4=97 sum_q5=86 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=103 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=12
family=F1-deleting-files-of-several-lower-leaves arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=27 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=433 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=244 sum_q11=0 segments_q2_one_birth_violated=27 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=244 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-newest-system-configuration-slot-reading-zeros arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=57 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3638 sum_i9_29_not_applicable=330 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=52 sum_q5=18 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-newest-system-configuration-slot-reading-zeros arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=17 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=806 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=16 sum_q11=0 segments_q2_one_birth_violated=6 sum_q3_steps=52 sum_q3b=0 sum_q4=14 sum_q5=6 sum_q6=0 sum_q7=14 sum_q9_gone=0 sum_q9_still=16 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=5
family=F1-newest-system-configuration-slot-reading-zeros arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=17 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=613 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=110 sum_q11=0 segments_q2_one_birth_violated=17 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=110 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-reuse-after-raising-the-floor arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=67 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3917 sum_i9_29_not_applicable=51 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=32 sum_q5=15 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-reuse-after-raising-the-floor arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=896 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=7 sum_q11=0 segments_q2_one_birth_violated=6 sum_q3_steps=18 sum_q3b=0 sum_q4=7 sum_q5=6 sum_q6=0 sum_q7=3 sum_q9_gone=0 sum_q9_still=7 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=7
family=F1-reuse-after-raising-the-floor arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=806 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=82 sum_q11=0 segments_q2_one_birth_violated=15 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=82 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-draw arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=56 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3904 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=334 sum_q5=191 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-long arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=9664 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=1045 sum_q5=661 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-long arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=29 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1411 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=476 sum_q11=0 segments_q2_one_birth_violated=29 sum_q3_steps=1943 sum_q3b=0 sum_q4=420 sum_q5=275 sum_q6=0 sum_q7=345 sum_q9_gone=6 sum_q9_still=470 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=9
family=G-long-F1 arm=P histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=25 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=4708 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=188 sum_q5=109 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-small arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3904 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=397 sum_q5=235 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
```

前の凍結コピー `superseded-b000407a/` との族ごとの比較（新旧26結果ファイルの E7RESULT 行から `elapsed_ns` を除きソートして sha256 の先頭16桁を取得。2集計は `code_baseline` を除いて比較）：

```
for f in $(ls -1 d-P.out d-V1.out d-V2.out f1-*.out f2-*.out f3a-*.out g-*.out | sed 's/\.out$//'); do a=$(grep '^E7RESULT' superseded-b000407a/$f.out | sed 's/ elapsed_ns=[0-9]*//' | sort | sha256sum | cut -c1-16); b=$(grep '^E7RESULT' $f.out | sed 's/ elapsed_ns=[0-9]*//' | sort | sha256sum | cut -c1-16); echo "$f old=$a new=$b $([ $a = $b ] && echo same || echo DIFFERENT)"; done | awk '{print $NF}' | sort | uniq -c
for s in one two; do diff <(grep -v '^#' superseded-b000407a/summary-segment-$s.out | sed 's/ code_baseline=[0-9a-f]*//') <(grep -v '^#' summary-segment-$s.out | sed 's/ code_baseline=[0-9a-f]*//') && echo summary-$s-same-apart-from-code-baseline; done
```

出力は `     26 same`（例 `f3a-P old=40278af11b04d80e new=40278af11b04d80e same`）、`summary-one-same-apart-from-code-baseline`、`summary-two-same-apart-from-code-baseline`。P/V1/V2 の各族・各方式の各測定量は前の凍結コピーと同じで、変わった族はない。前の凍結コピーの第1区間9ファイルもさらに古い `crates/`（`superseded-old-crates/`）と個別に一致（例 `f3a-P` は3箇所すべて `40278af11b04d80e`）。

成果物の完全性：各結果ファイルの結果行、`line_end=ok` の行、方式フィールド付き行、イベント行、`first_violation` が `none` 以外の履歴、完了マーカーを数える（各ファイルに `awk '/^E7RESULT name=history /{n++; if($0 ~ / line_end=ok$/)ok++; if($0 ~ / arm=/)arm++; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="first_violation" && a[2]!="none") fv++}} /^E7RESULT name=event /{ev++} /name=family_complete/{c++} NR==1{head=$0} END{print head, n+0, ok+0, arm+0, ev+0, fv+0, c+0}' <文件>`）。26結果ファイルの先頭はすべて `# code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`。各結果行数は登録履歴数と一致（D 8、F3a P 16105/V 1464、F2 P 256/V 64、F1 P 各128/V 各32、G-long P 64/V1 32、G-small P 64、G-draw P 64、G-long-F1 P 32）。各行に `arm=` と `line_end=ok` があり、すべて `name=family_complete` を持つ。P の12ファイルはイベント行0、`first_violation` が `none` 以外の履歴0。2集計は `unfinished_lines count=0`。F3a P の操作ステップは `awk '/^E7RESULT name=history /{n++; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="steps_observed")s+=a[2]}} END{print "f3a-P histories=" n " operation_steps=" s-n}' f3a-P.out` → `f3a-P histories=16105 operation_steps=143335`。

`PublishFirstFile`（番号1再発行への唯一の経路）の抽出回数：`for f in f1-broad-P f1-reuse-after-raising-the-floor-P f1-deleting-files-of-several-lower-leaves-P f1-newest-system-configuration-slot-reading-zeros-P g-long-f1-P f2-P; do echo "$f first=$(grep -h '^E7RESULT name=history' $f.out | grep -o '[[,]first(' | wc -l)"; done` → `f1-broad-P first=197`、`f1-reuse-after-raising-the-floor-P first=68`、`f1-deleting-files-of-several-lower-leaves-P first=12`、`f1-newest-system-configuration-slot-reading-zeros-P first=197`、`g-long-f1-P first=203`、`f2-P first=0`（F2 の重みは `AfterFirstFile` から始まり、unlink 後の表はこれを抽出しない）。

### 判定基準の個別確認（登録第6節、測定量ごとに報告）

F1 4群は `BROAD` / `REUSE_AFTER_RAISING_THE_FLOOR` / `DELETING_FILES_OF_SEVERAL_LOWER_LEAVES` / `NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS` の順、ジオメトリ4点は G-long / G-small / G-draw / G-long-F1 の順。数値は上記2集計による。

| 測定量 | F3a | F2 | F1 4群 | ジオメトリ4点 | 判定 |
|---|---|---|---|---|---|
| Q1 同一時間線の再発行（P） | 0 | 0 | 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 | なし |
| Q2 I-9.29 (同番号に単一の出生世代のみ許可)不合格（P、履歴数） | 0（成立159440ステップ、適用外・未判定0） | 0（成立15616） | 0（成立3638 / 3917 / 3910 / 3638、適用外330 / 51 / 58 / 330、未判定0） | 0（成立9664 / 3904 / 3904 / 4708、G-long-F1 適用外124、未判定0） | なし |
| Q9 | イベントなし | イベントなし | イベントなし | イベントなし | 適用外 |
| Q3 watermark 後退（P、ステップ数、ピークと終点） | 0、ピーク0、終点0 | 0、ピーク0、終点0 | 0、ピーク0、終点0 | 0、ピーク0、終点0 | なし |
| Q3b 破棄時間線を跨ぐ再発行（P） | 0 | 0 | 0 | 0 | なし（付随） |
| Q4 発行番号を越えたロールバック（P） | 11641 | 1748 | 39 / 32 / 615 / 52 | 1045 / 397 / 334 / 188 | 数を報告 |
| Q5 発行番号を越えるロールバック後に inode を作成した窓（P） | 7870 | 1062 | 13 / 15 / 526 / 18 | 661 / 235 / 191 / 109 | F2 + F3a 合計8932 ≥ 50、十分 |
| Q6 ロールバック時にリングの可読 root の最大 watermark < 現行（P） | 0（17954回判定） | 0（2951回） | 0（185 / 74 / 733 / 224回） | 0（1984 / 714 / 612 / 407回） | V717 未実行 |
| Q7 マウント／復旧で watermark の低い root に着地（P） | 0 | 0 | 0 | 0 | 数を報告 |
| Q10 1時間線で2番号以上発行した履歴（各方式） | P 16105、V1 1464、V2 1464 | P 231、V1 53、V2 53 | P 49 / 67 / 110 / 57、V1/V2 各15 / 15 / 27 / 17 | P 53 / 53 / 56 / 25、G-long V1 29 | V2 の実行した全族が不合格である必要 |
| Q11 発行番号が前版 watermark と異なる（全方式） | 0 | 0 | 0 | 0 | 一致 |
| R-V1（V1：Q5 ≥ 1 なら Q1 ≥ 1 かつ Q2 ≥ 1履歴） | Q5 577、Q1 975、Q2 567履歴 | Q5 195、Q1 336、Q2 53履歴 | Q5 4 / 6 / 86 / 6、Q1 12 / 7 / 103 / 16、Q2 4 / 6 / 26 / 6履歴 | G-long：Q5 275、Q1 476、Q2 29履歴 | 実行した全族で合格 |
| R-V2（V2：Q10 ≥ 1 なら Q1 ≥ 1） | Q1 3990 | Q1 696 | Q1 90 / 82 / 244 / 110 | 登録上未実行 | 実行した全族で合格 |
| R-V717 | 未実行 | 未実行 | 未実行 | 未実行 | P と同出力（Q6 = 0 から推論）、未実行 |

ジオメトリ感度（登録第8節）：G-long/G-small/G-draw/G-long-F1 の各点を g0（第1区間 F2）と比較し、P の Q1/Q2/Q9 判定はいずれも「なし」で反転しない。両点0で閾値を間に移動できず、判別力は R-V1/R-V2 が担う（G-long V1 の Q1 476、Q2 29履歴）。

失敗・停止条項（登録第10・11節）：F1/F2 は非発火（R-V1/R-V2）。F3 の2形態は0（`sum_f3_still_but_holds`、`sum_f3_violated_without_two_births` は26ファイルすべて0）。F4/F5 非発火（値固定節）。F6/F9 は P に Q1 イベントがある場合のみ実施し、イベントがないため未実施。F7 非発火（F2 + F3a の Q5 合計8932）。F8 非発火（S1 再構築エラー0。P が root を観測した回数は F3a 1431682、F2 304853、F1 4群 54585 / 67690 / 57613 / 60489、ジオメトリ4点 213378 / 75155 / 75554 / 104381。2集計の `sum_s1_root_observations`）。S3 の2経路不一致0、インスタンス表を読めないステップ0。マージ不確定も非発火（26ファイルとも `conflicts=0`、`duplicates=0`。クイック層で複数スレッドと単一スレッドを突き合わせ）。

V1/V2 の全 Q1 イベントを Q9 分類とそのステップの I-9.29 判定でクロス集計：`grep -h '^E7RESULT name=event ' d-*.out f3a-*.out f2-*.out f1-*.out g-*.out | grep -o 'arm=[A-Z0-9]* .*kind=[a-z0-9_]* .*q9=[A-Za-z]* i9_29_at_step=[a-z_]*' | sed -E 's/index=[0-9]+ //; s/position=[^ ]* inode=[^ ]* new_birth=[^ ]* older_births=[^ ]* //' | awk '{print $1, $2, $3, $4}' | sort | uniq -c` の出力は `2 arm=V1 kind=q1_same_timeline q9=Gone i9_29_at_step=holds`、`9 arm=V1 kind=q1_same_timeline q9=Gone i9_29_at_step=violated`、`1917 arm=V1 kind=q1_same_timeline q9=Still i9_29_at_step=violated`、`5225 arm=V2 kind=q1_same_timeline q9=Still i9_29_at_step=violated` の4行。「まだある」のイベントはそのステップで I-9.29 がすべて違反（F3 第1形態0）。

陽性対照の「活記録がその場で置換される」形態（Q9「移出済み」）は11回：`grep -n 'q9=Gone' f2-V1.out g-long-V1.out` は `f2-V1.out` の67、283、325、326、385行と `g-long-V1.out` の40、71、155、358、389、390行に命中。うち `g-long-V1.out` の `index=24 kind=q1_same_timeline`、`position=op92` と `position=op95` の2行（G-long V1 番号24、92・95ステップで番号7・8を発行）はその2ステップで I-9.29 が成立。旧誕生世代は候補集合から移出済みで、同ステップの候補集合に他の2誕生世代を持つ番号もないため、I-9.29 は再発行を見られない。C479 の「置換前の版が候補集合から移出後は見えない」が V1 に出現。残り9回は別の番号に2誕生世代があり I-9.29 は違反。P ではこの形態0回。

### 値固定（登録第7節、クイック層のケースを3方式で各1回）

- a1/a2（d1、起点）：`first_file_alone_reads_inode_one_born_at_three_and_watermark_two` は3方式すべて合格。現行 inode ツリー {(1, 3)}、watermark 2 で D8 確定項目6「新規プールでのファイル作成の書込」と一致。
- a3/a4（d2(k)、d3、P）：`rollback_one_step_then_create_matches_the_arm` 等6テストと `creations_across_mounts_and_a_cold_start_match_the_arm` が P で合格（`quick-tests.out` の `===== q-P exit=0` 節、`test result: ok. 26 passed; 0 failed; 2 ignored`）。d2(1)–d2(3) が順に2、3、4を発行し、d3 も2、3、4を発行するとアサート。`d-P.out` も一致：d2(1)、d2(2)、d2(3) は各 `issuances=4 watermark_final=5 rollbacks_applied=1`（起点の1と作成3回）。d2(4) は `issuances=3 watermark_final=4 rollbacks_applied=1`（最初のファイルが生存する版へロールバックし、その後 inode 作成を呼ばない）。d2(5)、d2(6) は `rollbacks_applied=0` で a3 の該当3セルにサンプルなし（修訂1第5項）。d3 は `issuances=4 watermark_final=5`。今回のクイックテストは `--nocapture` なしで、テスト内の出力行は `quick-tests.out` に入っていない。
- a5（d2(1)、V1）：3を発行、Q1 はちょうど1、Q9「まだある」、そのステップの I-9.29 は違反で合格。k = 2、3では発行番号 = S1 が読んだ対象版 watermark 2 で合格。
- a6（d2(1)、V2）：最初の2回はともに2を発行し、最初の作成後のステップで I-9.6 (水位が二か所の最大番号より大きい)が違反、2回目の作成ステップで Q1 がちょうど1、合格。
- a7：`f3a_scale_matches_the_registered_counts` 合格（16105、1464、143335）で、成果物の履歴・ステップ数も同じ。第2区間規模 `second_segment_scales_match_the_registered_counts` 合格（F1 P 15360ステップ、F1 V 3840、ジオメトリ4点 P 22080、G-long V1 4800）。各族の成果物履歴数も登録と一致（測定値の `histories=`）。

### 判定行の特定

装置は `name=verdict` 行を出力しない。成果物ディレクトリのトップレベルで `grep -n 'name=verdict' $(ls -1 | grep -v superseded) | wc -l` は0。各測定量はそれぞれ報告（測定値節）し、登録第6節の判定は「判定基準の個別確認」に記載。非ゼロの不合格系カウンタは陽性対照方式のみで登録の想定内：V1/V2 の Q1/Q2 は非ゼロ（陽性対照は不合格になるべき）、V1 の Q3/Q7 は非ゼロ（対象版 watermark をロールバックで取るため後退）。P の Q1/Q2/Q3/Q3b/Q7/Q11、S3 不一致、F3 2形態、S1 再構築エラーはすべて0。

### 単体テストと mutation

単体テスト28件：`grep -c '^#\[test\]' device-source.rs` は28、うち2件は `#[ignore]`（大規模入口と集計）。`quick-tests.out` の3方式それぞれ `test result: ok. 26 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out`。第1区間版より11件追加：第2区間規模・重み2件、新族の各番号0を実行する8件、2コード基線混在またはヘッダーなしの結果ファイルを集計で拒否する1件（`result_files_from_two_code_baselines_or_without_a_header_are_refused_by_the_summary`）。

登録第9節の mutation は6件：V1/V2 は陽性対照方式で実行した全族を区別できる（R-V1/R-V2 合格）。V3/V4/V5 は装置内の破壊スイッチで、`quick-tests.out` では各指定テストが失敗（3回とも `--skip first_history_of` で各族番号0の8サンプルを除外し `17 passed; 1 failed; 2 ignored; 0 measured; 8 filtered out`）。V3（台帳を番号で重複除去）と V4（発行イベント分類時に全 root を破棄と判定）は V1 の `rollback_one_step_then_create_matches_the_arm` の「a5：Q1 はちょうど1」で失敗（`left: 0`、`right: 1`。V4 は `q1=0 q3b=1` も出力）。V5（Q9 を常に「まだある」と判定）は P の `q9_classification_says_gone_when_no_holder_is_readable` で失敗。V717 は登録上条件付きで Q6 = 0 のため未実行。検出5、無効0、失敗せず0、未実行1。装置は `research/` 下でも `crates/mutations.tsv` 内でもなく、ミューテーション表がないため `mutate.sh` は適用外。装置ヘッダーの破壊スイッチ表にある V3 テスト名 `rollback_one_step_then_create_on_v1_reissues_three_on_the_same_timeline` は実際に失敗する `rollback_one_step_then_create_matches_the_arm` と異なる（テスト改名時にヘッダーを更新していない）。後者が失敗したテスト。

### 分岐表

| 分岐行 | 状態 | 根拠 | 残りと、その量が結論を覆し得るか |
|---|---|---|---|
| W：C479 (既発行inode番号の再利用が現状再現不能)の残る「生きたレコードのその場での置換」 | 判定に十分（凍結コピー `code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`；前の `b000407a…` でも判定に十分で各族の読数は同じだが、判定には入れない） | 「読数」の2集計：P 方式の rollback 付き履歴（F3a、F2）とランダム履歴（F1 の4群）、幾何4点の Q1 は全て0、`segments_q2_one_birth_violated` も全て0；F2 + F3a の Q5 合計 8 932 ≥ 50；R-V1、R-V2 は実行した全族で合格；F1、F2、F7、F8 は発火なし | 不足なし。登録された17 105区間の製品方式履歴では同一タイムラインの再発行を作れず、I-9.29 (同番号に単一の出生世代のみ許可)の赤は0区間、ウォーターマーク後退は0ステップ。残りは第3区間付属の F3b（クラッシュ復旧で root を破棄した後に再作成）と F4（書込み／フラッシュ故障注入後に再作成）で、判定十分後は未実行。W 行の条件で指定された2種類の履歴には含まれないが、「ウォーターマーク後退」の半分を覆し得る：クラッシュ復旧で古い root を選んだ後にその root とともに後退するのは、条項が許す破棄済みタイムラインをまたぐ形（Q3b）で、読み方 B では W 行を覆さない。その2族で同一タイムラインの Q1 ≥ 1 を測れば W 行は覆る |

### この実験が答えられないこと

- 「登録された履歴で作れなかった」は「作れない」ではない。P 方式の17 105区間は登録されたサンプル（F3a は中間区間長4まで全探索、F2 / F1 / 幾何点はシードによるランダム）であり、さらに長い履歴や選ばれなかった rollback 先・操作順序の数はない。
- クラッシュ復旧による root 破棄（二重故障形、F3b）と書込み／フラッシュ故障注入後の inode 作成（F4）は登録第3区間の付属で、判定十分後は未実行。履歴中に選ばれた `CrashRecoveryAbandoningTheNewestRoot` は C554 (クラッシュリカバリで放棄されたルートが一時的に読めない時にシャドウ会計が計算不能)の乙以降、マウントが拒否され破棄 root を作らないため、Q3b の0は破棄タイムラインをまたぐ再発行が存在しないことを示さない。装置は「破棄と判定された root」の回数を数えておらず、2区間で S3 が破棄 root を判定したかは結論しない。
- Q6 の読み方には制約がある：現行版はそのステップの可読 root のうち最新から取り、rollback 前にはそれ自体が環内で読める。Q6 > 0 は現行版自体を再構築できない場合にしか起きない。読み故障のない履歴で Q6 = 0 はこの読み方から導かれ、独立した観測ではない。そのため V717 は全族で未実行であり、「rollback で max を取る」ガードがこれらの履歴で通ったかは答えられない。
- 「生きたレコードのその場での置換」は I-9.29 (同番号に単一の出生世代のみ許可)では見えない。陽性対照 V1 方式 G-long 番号24の2回の再発行（`g-long-V1.out` の `index=24 kind=q1_same_timeline` の `position=op92`、`position=op95`）ではその2ステップで I-9.29 は成立した。製品方式ではこの形を作れなかったため、「製品経路で起きたら I-9.29 が捕捉できるか」には答えず、P 方式で0回だったとだけ言う。
- 「リポジトリコピー装置、凍結コピー」：数値は2026-10-08にコピーした2つ目の凍結コピー（`baseline-frozen-crates.out`）から得ており、リポジトリに収めた装置では再実行していない。コピー後に主作業領域の `crates/` はさらに6ファイル変化（`baseline-frozen-crates.out` の `name=main_drift_after_copy`）、うち `crates/singlefs-core/src/mount.rs` と `crates/singlefs-harness/src/gpu_memory_ledger.rs` は装置に組み込まれる。F3a、F2、F1、幾何点は変化後のコードで再実行せず、`scripts/experiment-apparatus/replay.sh` の `driver_e175` だけを変化後の主作業領域でビルドし D 族 P 方式を1回実行、`elapsed_ns` を除いた出力は `e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` と一致した。
- このページの読数を覆す現象：同じ凍結コピー・同じ装置で再実行し、いずれかの P 方式区間の結果行が成果物と異なる（再開実行とマルチスレッド統合は quick で突合済み）；または第3区間 F3b / F4 か登録外の履歴で同一タイムラインの Q1 ≥ 1、I-9.29 が赤になる。

### 経路と結論の登録

| # | 経路 | 算出方法 | ソース位置 | 共用する項目 |
|---|---|---|---|---|
| 1 | S3 経路1（インスタンステーブル） | 最新の可読 root のインスタンステーブルで (インスタンス, txg) ごとに各 root を破棄するか判定。2区間の各ステップの全可読 root を走査 | `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs:713 abandoned_by_the_table`、判定 `crates/singlefs-core/src/recovery.rs:1553 publish_is_abandoned_by_the_instance_table` | `readable_roots` の root、ディスク上のインスタンステーブル、`crates/` の破棄判定 |
| 2 | S3 経路2（ステップ台帳） | マウント成功ごとにモデル ルートリングのそのステップ直前の最新 root を選択 root とし、同じ旧インスタンスで txg が大きい root を破棄と判定 | `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs:723 abandoned_by_the_step_ledger`、選択 root は `crates/singlefs-harness/src/model.rs:2235 answer_mount_writable` の選択法 | `readable_roots` の root、理想モデルの ルートリング、実行器が返すマウント結果 |
| 3 | S1 台帳の再発行（Q1） | 番号ごとに出生世代と保持 root を記録し、発行時に同番号の別出生世代に衝突するとイベントを記録 | `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs:1028 HistoryAccount::observe`、再構築 `crates/singlefs-core/src/recovery.rs:5405 rebuild_version` | `readable_roots`、`rebuild_version`、経路1の破棄判定 |
| 4 | S2 checker の I-9.29 (同番号に単一の出生世代のみ許可) | プールレベル checker が各ステップのイメージを判定 | `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs:1158 HistoryAccount::judge_with_the_checker`、`crates/singlefs-checker/src/walk.rs:10947 check_pool_image` | イメージ本体のみ。1–3とパースコードを共用しない |

経路1と2は同じ量を判定し root ごとに比較して不一致を計数する（`device-source.rs` の `s3_path_disagreements`）。2区間の26結果ファイルは全て0回。ただし両経路は `readable_roots` の root 集合を共用し、列挙が誤れば両方とも誤る。経路3と4は同じ量ではなく、登録第10節 F3 で照合し（1188、1199行）、両区間とも0回。経路間比較はカウントであり `assert_eq!` ではなく、両経路をつなぐ assertion が1つ不足する。

### 影響を受ける意思決定

| 意思決定の項目 | 関係 | 再確認 |
|---|---|---|
| D8 (コアインデックス構造)決定済み項目6 | 参考資料 | 2026-10-21 影響なし：冒頭の装置ソース位置を scripts/experiment-apparatus/ 移転後の現状に変更。結論・数値・成果物は不変。以前の2026-10-18も影響なし：前回成果物をリポジトリへアーカイブ（git 履歴から取得可）、本文の参照はファイル名のみとし、装置ソースを成果物ディレクトリから scripts/experiment-apparatus/ へ移動したが内容・結論・数値は不変 |
| D23 (ジャーナルの役割と形式)決定済み項目14 | 参考資料 | 2026-10-21 影響なし：冒頭の装置ソース位置を scripts/experiment-apparatus/ 移転後の現状に変更。結論・数値・成果物は不変。以前の2026-10-18も影響なし：前回成果物をリポジトリへアーカイブ（git 履歴から取得可）、本文の参照はファイル名のみとし、装置ソースを成果物ディレクトリから scripts/experiment-apparatus/ へ移動したが内容・結論・数値は不変 |

## 履歴バージョン

### 2026-10-08：2区間を2つ目の凍結コピーで再実行し、各族の読数は不変

ルートリング系列の第10zバッチ第10・11ブロックのマージ後、`crates/singlefs-core/src/mount.rs` が再び変化（F の一連を繰り上げ）。メイン agent が2つ目の凍結コピー（`code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`、前のコピーと10ファイル異なる）を作り、2区間（D、F3a、F2、F1 の4群、幾何4点）の P、V1、V2 を登録どおり全て再実行した。装置ソースは不変（`310c60a5…`）。前の凍結コピーの37ファイルは登録改訂2第4条に従い `superseded-b000407a/` へ移し、判定に入れない。新しい26結果ファイルの E7RESULT 行は `elapsed_ns` を除けば前回とそれぞれ一致し、2集計も `code_baseline` 以外は行ごとに一致。状態は「実行済み（判定に十分）」のまま、ページの数値では `code_baseline`、コピーのバッチ、V1 アンカー行番号（7758 → 7917）、スレッド数（7 → 5）、試走の壁時計時間、`f2-V1.out` の「繰り出し済み」行番号だけを変更した。

### 2026-10-08：凍結コピーで2区間を再実行し W 行の判定に十分

ユーザー指定の「新コードで両区間を再実行」と登録改訂2に従い、メイン agent が ルートリング系列第10zバッチ第9ブロックのマージと検証後に凍結コピー（`code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`）を作り、第1区間全体を再実行し、第2区間（F1 の4群、幾何4点）を初めて実行した。いずれも同じコピー上。旧コードでの第1区間の19ファイルは `superseded-old-crates/` に移し判定に入れない。新第1区間の9結果ファイルの E7RESULT 行は `elapsed_ns` を除けば旧版とそれぞれ一致。装置に `code_baseline` のヘッダーと集計列を追加（単体テスト28件）、`e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` を凍結コピーのものに置換。両区間の P 方式は Q1、Q2 とも0、F1 F2 F7 F8 は発火せず、状態を「一部実行済み（第1区間；第2区間停止：crates/ が第1区間と異なる）」から「実行済み（判定に十分）」へ変更。第3区間は判定十分後未実行。

### 2026-10-08：第2区間を割り当て、crates/ が第1区間と異なるため停止

第2区間（F1 の4群、第8節の幾何点）開始前に主作業領域の `crates/` を確認すると6ファイルが第1区間の基線と異なり（`baseline-segment-two.out`）、割当時の指示に従い停止し、成果物は1つも実行しなかった。状態を「一部実行済み（第1区間；第2区間は指定待ち）」から「一部実行済み（第1区間；第2区間停止：crates/ が第1区間と異なる）」へ変更。F1 族と幾何ノブを追加した装置は実行担当者の草稿ディレクトリにのみあり、成果物ディレクトリには入っていない。

### 2026-10-08：ページ作成、第1区間実行済み

リポジトリコピー装置を作成（単体テスト17件、quick の3方式各15件合格・大規模入口2件は ignore）、事前登録第12節に改訂1–8を記入。第1区間（D、F3a、F2 の3族を P、V1、V2 の3方式で）を実行・集計し、状態は「一部実行済み（第1区間；第2区間は指定待ち）」。`scripts/experiment-apparatus/replay.sh` に E175 (製品パス上のinode番号水位ロールバックと再発番)を `|@driver_e175` として1行登録し、抽出したドライバーを1回試走。出力は `d-P.out` の E7RESULT 行と行ごとに一致し、`e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` として保存した。
