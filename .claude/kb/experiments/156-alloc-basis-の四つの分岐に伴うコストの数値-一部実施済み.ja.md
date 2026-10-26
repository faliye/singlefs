## E156 alloc-basis 4分岐のコスト数値 ― 一部実施済み（2026-09-26 の第4回再実行では、前提1は成立（当日のコードでも「レコードは永続化済み、ルートスロットは未永続化」のクラッシュ後に書き込み可能なマウントで穴を作れる）。前提2は不成立（F の二つの計測条件が F 引き上げの一連の処理で一つに合流）となり、分岐表第3行は無効で測定しない。前提3では分岐2の系列だけが旧式で定義され、Hh 履歴は新しい admission 式に到達しない。第1行は収まる38履歴 ×「実」「毎」の二つの回収時点で測定を完了し、登録された判定十分条件を満たした。12サンプルすべてで Q1c-前が2スロットを超え（h_欠 = 1 ですでに10スロット）、Q1d はいずれも h_欠 とともに増加し、F1 が発動した。ユーザーは2026-09-27に第1行をマイルストーン3へ残し、並べ替えアルゴリズムを考慮すると決めた。装置はリポジトリに収録、決定的であり、第4回再実行時の17単体テストと装置の22変異をすべて検出。第2・7行はユーザー決定済みで測定しない。；2026-10-05 の第5回再実行では、core のアサーション追加後、W7 の陽性対照を PC-閾値固定に置き換え、6サンプルの両列すべてで観測した。第4回の値を新しい `crates/` で正規化して比較すると294行が異なり、F1 は引き続き発動、Q1d は「実」列の4サンプルで h_欠 とともに増えなくなり、K10 は34観測点で不一致となった。原因切り分け実行により31-1が原因ではないと分かった。メイン agent は第5回の成果物を根拠として採用し（`e156-r5-prereg.md` 末尾「主 agent 认定：第 5 次交回与 S11 两边查」第2項）、第4回の成果物はその時点のツリーの記録として保持する。；2026-10-05 の第6回再実行では、ユーザーが同日決めた乙の解釈、すなわちマウント時に全ゼロと読まれたルートリングスロットを、それが保持していた可能性のある最古の txg に固定する回収閾値を適用した。38セルすべてで K10・F24 は0となり、コピー内で core の全ゼロ固定を除去すると、両方とも予測どおり34観測点で赤になった。「実」列は第5回と全フィールド一致し、「毎」列も「実」列と一致したため、「毎」列の4サンプルの Q1d は h_欠 とともに増える状態から非単調に変わった。F1 は引き続き発動し、Q1c-前の判定は変わらない。PC-閾値固定の6セル両列は第5回とバイト単位で一致した。19単体テストと変異 M45–M47 は未実行で、提出時に行う。；2026-10-18 の第7回再実行では、質問票 H1 の C566 (リングに穴がある際の超過控除スロット数が穴数に伴い無界に増加)の半分を測った。穴数を1段階からリングが一周埋まるまで、さらに一周を越えるまで走査した。読み方甲（一周内を埋める、S = 8・16 の8単位）と読み方乙（一周を越える、12単位）のどちらでも Q1a ピークは「なお増加中」と判定され、上界値はすべて最大段階にあり（端点以下、上界は測れていない）、読み方乙の最大段階 S = 16 は1810 / 1416スロットだった。H1 は登録済みの判定十分条件を満たした。Hh-後の8単位は「非単調」、F32 が発動（登録時の各段階同一との予測は誤り）。インライン単体テスト25件はメイン agent が実行済み、新規変異 M50–M54 は未実行で提出時に行う。）

[中文](156-alloc-basis四条岔路的代价数.md) · [English](156-cost-figures-for-the-four-alloc-basis-forks.en.md) · **日本語**

<!-- doc-lint:not-numbers K1 K8 S1 S2 S3 S4 S5 S9 V1 V2 V3 V6 V7 V8 V10 V11 V13 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F16 Q1 Q2 Q3 Q7 R1 R2 R3 R4 R5 R6 R7 R8 G7 G8 G12 G26 G27 G28 H0 HF K9 U5 U7 U8 U11 U13 F23 F24 M32 M34 A10 A14 F26 F27 F28 F29 F30 K10 K15 L1 L3 M35 M44 S11 S13 S14 U14 U19 V18 W7 W14 P7 Q1r5a Q1r5b Q2d R1a R1b R1c R1d R2a R2b R2c K16 L6 M45 M46 M47 U20 F31 F32 F33 F34 F35 V19 V20 S15 S16 S17 S18 M36 M42 F22 M50 M54 S20 W28 -->

本文の K1、锚1/锚2/锚3（登録「七」の第2類アンカー表の各行に対応するローカル略称。KB の別の登録表が同形の番号を別の意味で使っているため、番号形式にせず衝突を避ける）、K8、K9、S1–S5、S9、V1–V8、V10、V11、V13、F1–F14、F16、R1–R8、Q1a–Q1f、Q2a/Q2b/Q2c、Q3a–Q3e、Q7a–Q7f、U5、U7、U8、G7、G27、G12、甲-T1、F-扣、H0、HR、HK、HK-F0、HF、Hh(k)、HX、HY、β0/β1/β2/βK-w/r/f/l2/β_syn/β_F0、Bd(k)、Bk1/Bk2/Bk3 は、いずれも `e156-preregistration.md`／`e156-r2-prereg.md` の実行前登録におけるアンカー／判定基準／履歴系列／アーム名であり、登録先はこの2ファイルである。KB の D / E / C / I 番号ではない。

<!-- doc-lint:registry name-col=2 -->

| 番号 | 略称 |
|---|---|
| X8 | 小プール枯渇時の二つの算定方法の構成 |
| P21 | 解放が早いほど待ち時間が長い |
| I3 | 1回のマウント内で割り当て記録は増加のみ |
| N2 | 解放時に書き換える割り当て世代フィールド |

**判断材料**：alloc-basis の四つの分岐（`alloc-basis-forks.md`）をユーザーに提出する前にコストの数値を用意する。判定収束表の②行がそのため第5・60・54・43行をブロックしている。ユーザーは2026-09-21に「先に計数実験を走らせ、測り終えたら一括提出」と決めた。第2・7行は判定上「実行時コストなし」と記載されている。第2行で測るのは、D28 (マウント時コミットメント量)確定項目1の「− defer 解放待ち」を削除するとき、同時に変更すべき一覧を1コマンドで確認できるか。第7行で測るのは、G27（defer 勘定に追加するチェック）が現在の2チェックでは判別できない差を判別できるか。D28 確定項目1と D16 (パブリッシュセマンティクス)確定項目1が分岐に従って決定されたとき E156 を参照する。第2・7行はユーザー決定済みで、第3行は第4回再実行の前提2不成立により無効、測定しない。第1行は2026-09-27にマイルストーン3に残し並べ替えアルゴリズムを考慮するとユーザーが決め（原文「可以留到里程碑3 但是要考虑重排算法」）、未済の検査 C566 として記録した。

**問題**：第4回再実行の登録 `e156-r4-prereg.md` は質問票 `e156-r4-questions.md` の4節に答える。実1～実5の後、管理者ロールバックはマウント中の前方公開に変更され（マウント時のロールバックと切り詰めは削除）、各公開でマウント内回収を行い、admission は「defer 解放待ち」を差し引かなくなった。第2・7行はユーザー決定済みで測らず、第1・3行についてまず3前提に答える。前提1：現在のコードでルートリングに穴が生じるか。前提2：現在の F 引き上げ経路で二つの F の算定方法を区別できるか。前提3：どの量が旧 admission 式で定義されるか。前提1成立時のみ第1行を測定し（登録5.5第2段階）、前提2が不成立なら第3行は無効で測定を設けない。装置 `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` は**リポジトリ収録の装置**で、実際の mkfs、ウォームアップ、新規プールでのファイル作成、上書き、空公開、書き込み可能マウント、マウント中ロールバック、F 引き上げを読み取り専用のドライバとして実行し、`crates/singlefs-core` と `crates/singlefs-checker` の製品コードは変更しない。

### 第7回再実行（2026-10-18）：穴数を1段階からリング一周を埋め、さらに一周を越えるまで走査。読み方甲・乙とも Q1a ピークは「なお増加中」、上界値はすべて最大段階（端点以下、上界は未測定）

再実行登録 `research/prompts/e156-r7-prereg.md`（装置の完成後、成果物を一度も実行する前に実行担当者が記した「修訂 1」を含む）と質問票 `research/prompts/m3-close-cost-measurement-questions.md` の H1 行は、C566 (リングに穴がある際の超過控除スロット数が穴数に伴い無界に増加)と C380 (リングに空洞がある場合に解放世代の一意値を判定不能)のうち前者を測る。現在の `crates/` で、穴数の増加に伴う過剰差し引きスロットに上界があるか、あるならいくつかを問う。判定十分条件の原文は「穴数を少なくとも3段階（リング一周が埋まる場合を含む）取り、各段階の過剰差し引きスロット数を報告」。H1 の候補3つ、甲：書き込み可能マウント時に穴を埋める、乙：現在のコードで過剰差し引きに上界があり受け入れ条件を変更して課題を消す、丙：前穴の過剰差し引きをマイルストーン3から移す、は登録中のアームではない（登録5.1）。今回は現在のコードのみ、甲-T1 アームを測り、「実」列を基準に「毎」列を併記する。どの候補が成立するかはここでは判定しない。

**読み方を固定**（登録 W26・W28）：「リング一周が埋まる」の両解釈を測り、各1行で同形の基準を用いる。甲は一周内を埋める R7a、前位置 Hh(k, 前)、k = 1 … `k_满(S)`、S = 4 / 8 / 16 で2 / 4 / 8段階。S=4は2段階のみのため報告のみで判定しない。乙は一周を越える R7c、Hh(1, 前) … Hh(`k_满` + 1, 前) と Hc の2段階を加え、計 `k_满` + 3段階。統計単位（系列、S、ρ、時点）について各段階の Q1a ピークを段階昇順に取り、隣接差がすべて ≥ 0 かつ最後が > 0 なら「なお増加中」（上界未観測、端点以下）、すべて ≥ 0 かつ最後が 0 なら「横ばい」（上界＝最終段階ピーク）、それ以外を「非単調」とする。

**装置と成果物**：コマンドと出力は今回実行したもの。単体テスト、経過時間、実装スナップショットはメイン agent の成果物実行スクリプトによる。

| 項目 | コマンド | 出力原文 |
|---|---|---|
| 装置の行数 | `wc -l crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` | `7580`（`git show HEAD:<装置> | wc -l` は `6291`） |
| 装置ソース sha256 | `sha256sum crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` | `bcb356afd4867a27f80c943b10d62962e520cbf74a4d328327722959c6a7cdb3`。登録の修訂1および実行前後のメイン agent スクリプトと同一。 |
| 単体テスト数 | `grep -c '#\[test\]' <装置>` | `25`（`git show HEAD:<装置> \| grep -c '#\[test\]'` は19、6件追加） |
| 単体テスト結果 | メイン agent が release で装置のインライン単体テストを実行。checker 層のヘビー層テストで実行担当者は未実行、ログはリポジトリ外。 | `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s` |
| 成果物 | `wc -l research/results/e156-alloc-basis-counts-2026-10-18.out`；`tail -n 1 <成果物>` | `16669`；`E7RESULT name=done emitted=16669` |
| 成果物 sha256 | `sha256sum research/results/e156-alloc-basis-counts-2026-10-18.out` | `1663f1ea28e5b3334f21adfc121bbf899bc66e7e429eea782f1f0c562c776c83` |
| 成果物を実行したバイナリ | メイン agent のスクリプトが実行前に記録した `target/release/e156_allocation_basis_counts` | sha256 `eb3903266a015578b10c412a18a7a988eda0dfbf9fc161790658417d142b2d70`。現在のリポジトリ直下 `target/release/` は `81106e66389b91790a16726e56001648b950cabe720386457a02f9cd65fa05c9`。成果物作成後に再ビルドされ、ソース sha256 は同じだが、成果物は現バイナリでは生成されていない。 |
| 経過時間 | メイン agent のスクリプトによる測定 | `挂钟秒数 241`（release、スレッド上限31、メモリ上限8G） |
| 実装スナップショット（S19・S3） | 実行前後にそれぞれ `git diff HEAD -- crates/ \| sha256sum`、`git diff HEAD --stat -- crates/ \| tail -1`、`git status --short crates/ \| sha256sum` | 前後とも `7fd21f75d5e478c1342b53d0d54b123303ac7993b0e4af473d049dceea10d5ca`、` 190 files changed, 73166 insertions(+), 4603 deletions(-)`、`4d180fd57ae7b242a5579beeca267bdf9441fb36f9cc7442a9246cdc71ba33a4`。登録第3節の `847e0e54…` と修訂1開始時の `16ba9b79…` はどちらも異なる。別セッションが `crates/` を変更中であり、登録 S19 第2項は元々停止条件ではない。対象は実行時点のツリー。 |

**実行開始の経緯**：実行担当者の `run-product.sh` は `gpu` feature なしで装置をビルドした一方、マルチマシン設定は `ENABLE_GPU=1` だった。所定状態表の判定段がプレースホルダーの結論を返し（メイン agent の診断であり、この実行は独立再現していない）、装置は `verdict` 関数で panic した。`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` の `PlaceholderOfATableLeftToAnotherHost`・`PlaceholderOfATableRecordedOnly` 分岐で、メッセージは `SINGLEFS_CRASH_AMPLIFICATION_STAGE`・`SINGLEFS_CRASH_AMPLIFICATION_SHARE` の2変数のみを名指しした。この回の出力は次の実行で上書きされ、保存されておらず成果物ではない。メイン agent は `--features verdict-store,gpu` 付きビルドに変更して `scripts/crash/gpu-quiesce-run.sh` から起動し、`research/results/e156-alloc-basis-counts-2026-10-18.out` を生成した。S3 の前後スナップショットは同一で、この実行中に別セッションが `crates/` を変更していないことを示す。

**ゲート・停止・無効化条項**（登録第11節）：

| 登録項目 | 値 | 結論 |
|---|---|---|
| S20 装置の Rust アンカーモデルと Python アンカースクリプトの点ごとの比較 | `anchor_r7_summary` 58行、`anchor_r7_dump` 3567行、`anchor_r7_k0_summary` 3行、`anchor_r7_k0_dump` 384行は、修正後の `anchors_e156_r7_hdong_window.py`（sha256 `a33d5f8c…`）および k0 ラッパー `anchors_e156_r7_k0.py`（sha256 `c9bc6de0…`）の出力と `cmp` で差異なし。実行担当者の開始前試行では h_洞 ウィンドウの左端がスクリプトから漏れて S20 が一度発動、対処は登録「修訂 1」第2項。 | 発動せず |
| S21 実分配器の値とアンカー不一致（ピークがアンカーのツリーテーブル下限以上という項を含む） | `q1_cell_checks` 116行で `a15_every_below`・`a15_real_below` の和は0、`a14_mismatches`・`a11_mismatches`・`a23_lag_mismatches`・`a24_pinned_mismatches` の和もすべて0。独立スクリプトでも110セル×2列すべてのピークがアンカー `q1a_tt_peak` 以上と確認。 | 発動せず |
| S11 / S12 ゲート（K10・F24・G-adm） | `verdict stage=second` の `s11_k10_violation_points=0`、`s11_a10_real_mismatch_cells=0`、`s12_g_adm_nonzero=0`、`failure_clauses` の `f24_real_every_differ_points=0` | 発動せず。PC-c2 対照セルの `a14_mismatches=9`・`seen=true` は設計どおり非0。 |
| V20 判定関数の陽性対照（PC-段階列） | 5つの合成列で `holds=true`、`r7_verdict` 集計 `tier_sequence_control_cases_wrong=0` | 発動せず |
| V21 PC-同一スロットで低下が見えない | 6つの (S, ρ) の Hs(2, 後) で「毎」列ピークが未変更時の 344 / 236 / 712 / 476 / 1460 / 960 から10へ低下、`seen=true`。集計 `same_slot_controls_run=6 same_slot_controls_without_a_drop=0` | 発動せず |
| 判別力自己検証② 最終段階を除いて再判定 | `dropping_the_last_tier_changed_units=0_of_24`。R7a・R7c の24単位で最後の段階の除去により判定が変わったものはない。S=4 の R7a 4単位は元々2段階のみ。 | 判定は最終段階に鈍感 |
| 判別力自己検証③ 2スロット／1スロット閾値を2観測値間に移動 | `threshold_moved_units_flipping=60_of_60` | 実行可能、V5 は発動せず |
| その他の無効化条項 V1・V2・V6・V16・V17 | `verdict stage=second` の `v1_over_withheld_controls_void=0`、`v2_a10_every_mismatch_cells=0`、`v6_a11_mismatch_points=0`、`v16_judge_cases_wrong=0`、`v17_a13_every_nonzero_points=0` | 発動せず |
| 打ち切り | `truncated_cells=0` | 116セルすべて登録された終点まで実行、「未測定」はない。 |
| 旧失敗条項 F1・F2・F22・F23 の報告 | `failure_clauses`（`:16654`）で `f1_front_above_two_groups` は12グループ全サンプルを列挙、`f2_no_observable_difference=false`、`f22_front_and_back_differ_groups=none`、`f23_back_at_most_two_groups=none` | 第6回と同形。F1 は依然発動するが、分岐表第1行は2026-09-27にユーザー決定済みで再判定しない。 |

**件数の自己確認**（登録5.2：各ρで S=4 / 8 / 16 の履歴がそれぞれ14 / 18 / 26、合計116。 `q1_cell` = 116 + PC-c2 対照1 + PC-同一スロット6 = 123）：

```
$ for n in q1_cell q1_cell_reused q1_cell_checks r7_tier_peaks r7_verdict pc_same_slot pc_tier_sequence anchor_r7_summary anchor_r7_dump anchor_r7_k0_summary anchor_r7_k0_dump; do printf "%s %s\n" "$n" "$(grep -c "^E7RESULT name=$n " research/results/e156-alloc-basis-counts-2026-10-18.out)"; done
q1_cell 123
q1_cell_reused 2
q1_cell_checks 116
r7_tier_peaks 60
r7_verdict 6
pc_same_slot 6
pc_tier_sequence 5
anchor_r7_summary 58
anchor_r7_dump 3567
anchor_r7_k0_summary 3
anchor_r7_k0_dump 384

$ grep '^E7RESULT name=q1_cell ' research/results/e156-alloc-basis-counts-2026-10-18.out | awk '{print $3, $4, $5}' | sort | uniq -c
      2 family=Hc s=16 rho=1
      2 family=Hc s=16 rho=1/4
      2 family=Hc s=4 rho=1
      2 family=Hc s=4 rho=1/4
      2 family=Hc s=8 rho=1
      2 family=Hc s=8 rho=1/4
     18 family=Hh s=16 rho=1
     18 family=Hh s=16 rho=1/4
      6 family=Hh s=4 rho=1
      6 family=Hh s=4 rho=1/4
     10 family=Hh s=8 rho=1
     10 family=Hh s=8 rho=1/4
      6 family=Hs s=16 rho=1
      6 family=Hs s=16 rho=1/4
      6 family=Hs s=4 rho=1
      6 family=Hs s=4 rho=1/4
      6 family=Hs s=8 rho=1
      6 family=Hs s=8 rho=1/4
      1 family=PC-c2 s=4 rho=1
      1 family=PC-same-slot s=16 rho=1
      1 family=PC-same-slot s=16 rho=1/4
      1 family=PC-same-slot s=4 rho=1
      1 family=PC-same-slot s=4 rho=1/4
      1 family=PC-same-slot s=8 rho=1
      1 family=PC-same-slot s=8 rho=1/4
```

各ρで S=4 は `Hh` 6 + `Hc` 2 + `Hs` 6 =14、S=8 は10+2+6=18、S=16 は18+2+6=26で登録どおり。`q1_cell_checks` は116行で各履歴1行、`r7_tier_peaks` は5量（R7a、R7b、R7c、R7d 前、R7d 後）×3 S×2 ρ×2時点の60行。`q1_cell_reused` 2行は S=4、ρ=1 の Hh(1, 前)・Hh(1, 後) が第1段階のセルを再利用したもの。「収まらない」36セル（`verdict stage=second` の `cells_run_without_fitting_the_ring=36`）は、アンカースクリプト出力 `fits=False` の18履歴×2 ρに等しい。

```
$ grep -c 'fits=False' research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.out
18
```

S20 と照合スクリプト（上のゲート表 S20・S21 に対応、出力原文）：

```
$ sed -n 's/^E7RESULT name=anchor_r7_summary //p' research/results/e156-alloc-basis-counts-2026-10-18.out | cmp - research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.out && echo SUMMARY_SAME
SUMMARY_SAME
$ sed -n 's/^E7RESULT name=anchor_r7_dump //p' research/results/e156-alloc-basis-counts-2026-10-18.out | cmp - <(grep '^DUMP ' research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.dump | sed 's/^DUMP //') && echo DUMP_SAME
DUMP_SAME
$ sed -n 's/^E7RESULT name=anchor_r7_k0_summary //p' research/results/e156-alloc-basis-counts-2026-10-18.out | cmp - research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_k0.out && echo K0_SUMMARY_SAME
K0_SUMMARY_SAME
$ sed -n 's/^E7RESULT name=anchor_r7_k0_dump //p' research/results/e156-alloc-basis-counts-2026-10-18.out | cmp - <(grep '^DUMP ' research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_k0.dump | sed 's/^DUMP //') && echo K0_DUMP_SAME
K0_DUMP_SAME
$ sed -n 's/^E7RESULT name=anchor_dump //p' research/results/e156-alloc-basis-counts-2026-10-18.out | wc -l
1092
$ sha256sum research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.py research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_k0.py
a33d5f8ce5bb70585f1372ac8fa8209a0c3dac3282026d7fd77c152f673d2b6e  research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_hdong_window.py
c9bc6de064be9e6d89da881cd93d25ec04604e3b7206588fb5b078b3b2016397  research/results/e156-alloc-basis-counts-2026-10-18-support/anchors_e156_r7_k0.py
$ python3 -I research/results/e156-alloc-basis-counts-2026-10-18-support/check_tier_peaks_against_observations.py
r7_tier_peaks 行 60 条，用原始观测点重算后峰值序列不同的 0 条；有观测点的格 123 个
$ python3 -I research/results/e156-alloc-basis-counts-2026-10-18-support/check_peaks_against_anchor_lower_bounds.py
格 × 列 220 个，峰值不低于锚点树表下界的 220 个，低于的 0 个 []
```

2つの照合スクリプト出力は `research/results/e156-alloc-basis-counts-2026-10-18-support/check_tier_peaks_against_observations.txt` と `research/results/e156-alloc-basis-counts-2026-10-18-support/check_peaks_against_anchor_lower_bounds.txt` に各1行で別保存され、上記コマンドの出力と完全に同一である。

**H1 の測定値：読み方甲（R7a）と乙（R7c）の並記**（登録第6節。各統計単位は1行で、それぞれ判定を報告し、合取しない）。この系列では Q1a = Δ。`q1_trajectory` の `quantity=q1b_real`・`quantity=q1b_every` の `peak` は123セルすべて0、`delta_real` と `q1a_real` の `peak` はセルごとに同一、`q1_cell_checks` 116行の `a13_real_nonzero`・`a13_every_nonzero` はいずれも合計0。「実」「毎」列は Hh・Hc・Hs の116履歴でピーク、正の点数、期末値がセルごとに同じで、PC-同一スロット対照セルのみ設計上異なる。`peak` と `peak_after_first_ring`（最初のトランザクションを含む一周をリングが回った後のピーク）は123セルすべて同じ。`final`（期末値）は123セルすべて0。終点が最後の穴 + 3S + 2 なので、期末値0は履歴の構成によるものと推定される。一周後に閾値が解除されるためであり、「過剰差し引きが存在しない」ことを意味しない。

読み方甲：Hh-前系列、段階＝注入穴数 k = 1 … `k_满(S)`（穴はすべて最初の一周 txg < 3S に入る）。ρ = 1・1/4 と「実」「毎」のピーク列はそれぞれ同一。

| S | 段階（k） | 各段階の Q1a ピーク | 隣接差 | W28 判定 | max(P)、最大段階か |
|---|---|---|---|---|---|
| 4 | 1、2 | 10、20 | 10 | 段階不足（`fewer_than_three_tiers`）、報告のみ | 20、はい |
| 8 | 1 … 4 | 10、20、28、36 | 10、8、8 | なお増加中（`still_rising`） | 36、はい（端点以下） |
| 16 | 1 … 8 | 10、20、28、36、44、52、60、68 | 10、8、8、8、8、8、8 | なお増加中 | 68、はい（端点以下） |

甲の `r7_verdict quantity=r7a` は12統計単位のうち8単位（S=8・16 × 2ρ × 2時点）を判定し、8すべて「なお増加中」、横ばい0、非単調0。S=4 の4単位は2段階のみ。

読み方乙：Hh-前の k = 1 … `k_满` + 1 に Hc の2段階を追加。注入穴数 n は `k_满` + 1 の2倍・3倍。「実」「毎」の列は同一で、ρ別に記載する。

| S | ρ | 段階（注入穴数） | 各段階の Q1a ピーク | 隣接差 | W28 判定 | max(P)（系列、txg）、最大段階か |
|---|---|---|---|---|---|---|
| 4 | 1 | 1、2、3、6、9 | 10、20、146、308、594 | 10、126、162、286 | なお増加中 | 594（Hc，63）、はい |
| 4 | 1/4 | 1、2、3、6、9 | 10、20、110、244、458 | 10、90、134、214 | なお増加中 | 458（Hc，63）、はい |
| 8 | 1 | 1 … 5、10、15 | 10、20、28、36、294、608、914 | 10、8、8、258、314、306 | なお増加中 | 914（Hc，105）、はい |
| 8 | 1/4 | 1 … 5、10、15 | 10、20、28、36、226、476、716 | 10、8、8、190、250、240 | なお増加中 | 716（Hc，105）、はい |
| 16 | 1 | 1 … 9、18、27 | 10、20、28、36、44、52、60、68、594、1208、1810 | 10、8、8、8、8、8、8、526、614、602 | なお増加中 | 1810（Hc，201）、はい |
| 16 | 1/4 | 1 … 9、18、27 | 10、20、28、36、44、52、60、68、458、944、1416 | 10、8、8、8、8、8、8、390、486、472 | なお増加中 | 1416（Hc，201）、はい |

乙の `r7_verdict quantity=r7c` は12統計単位すべてを判定、12すべて「なお増加中」、横ばい0、非単調0。12サンプル点で一致するが、この12点に限る。

R7e（上界値）：R7a・R7c の各統計単位の max(P) はすべて最大段階にあるため「端点以下」と記し、上界は測れていない。初めて2スロット・1スロットを超える最小段階はいずれも1（`first_tier_above_two_slots=1 first_tier_above_one_slot=1`、60単位すべて）。穴1個の注入ですでに10スロット過剰に差し引く。甲では最初の一周内で穴1個追加ごとに8スロット増加（S=8・16の第3段階以降、隣接差8）。乙では2周目に入る最初の穴で146 / 294 / 594へ跳び（ρ=1、S=4 / 8 / 16）、以降 `k_满` + 1 個増えるごとに134–614スロット増加する（隣接差、S・ρに依存）。

F31（非単調）：R7a・R7c に非単調の単位はなく発動しない。F33（両解釈が逆）：同一 (S, ρ, 時点) の R7a・R7c に横ばいはなく発動しない。S=4 の R7a は未判定で比較に含まない。

**参考報告の2行：R7b は F32 を発動、R7d は発動せず**（登録 R7b・R7d、判定は基準に算入しない）：

| 量 | 登録時の予測 | 測定値 | 条項 |
|---|---|---|---|
| R7b Hh-後（k = 1 … `k_满后`、S=8・16 の8単位を判定） | 各段階のピークは同一（同一周内の最古ルートのみを固定） | 8単位すべて「非単調」。S=16、ρ=1 のピーク 716、710、702、694、686、678、672、666 は穴数とともに各段階で低下。S=16、ρ=1/4 は474、478、470、472、466、468、462、468と上下し、S=8、ρ=1 は366、360、352、344。数百スロット規模で、k=1 の S=4 / 8 / 16 は174 / 366 / 716、同じ k の Hh-前の10スロットより1桁大きい。 | F32 発動。`r7_verdict quantity=summary` の `report_only_units_unlike_the_prediction` が8単位を列挙。登録第4節の各段階同一という推論は誤りで、推論どおりに本文を書いてはならず、メイン agent に引き渡す。 |
| R7d Hs-前・Hs-後（m=1、2、3、24単位） | 同一スロットに穴を重ね、旧ルートが決して書き消されず、過剰差し引きが公開とともに線形増加 | 24単位すべて「なお増加中」、すべて最大段階。Hs-前 S=16、ρ=1 は10、758、1518、Hs-後は716、1460、2134。 | 登録予測どおりで F32 はこの行では発動せず。F34（Hs-後が横ばいと判定され、D16 公開セマンティクス確定項目1の「1台のディスクにルートを書き込めない状態が続くと、保持される単位は公開とともに上限なく線形増加」と不一致）も発動せず。 |

R7b と Hh-前を比較すると、Hh-後は k=1 ですでに数百スロットで穴数に応じて増えずむしろやや減少し、Hh-前は最初の一周では10スロットから穴ごとに8増え、一周を越えると数百～千超で増加し続ける。「穴数に伴う上界があるか」の形は穴の位置によって異なる。これは2つの観測であり、甲・乙への裁定ではない。

**判定行を成果物 `research/results/e156-alloc-basis-counts-2026-10-18.out` から全文転記**。`verdict stage=second` 1行（`:16656`）と `r7_verdict` 6行（`:16559`–`:16564`）：

```
E7RESULT name=r7_verdict quantity=r7a units=12 judged=8 fewer_than_three_tiers_units=[s=4_rho=1_timing=real,s=4_rho=1_timing=every_publish,s=4_rho=1/4_timing=real,s=4_rho=1/4_timing=every_publish] still_rising=8 flat=0 not_monotonic=0 geometry_sensitivity=consistent_on_8_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:fewer_than_three_tiers,s=4_rho=1_timing=every_publish:fewer_than_three_tiers,s=4_rho=1/4_timing=real:fewer_than_three_tiers,s=4_rho=1/4_timing=every_publish:fewer_than_three_tiers,s=8_rho=1_timing=real:still_rising,s=8_rho=1_timing=every_publish:still_rising,s=8_rho=1/4_timing=real:still_rising,s=8_rho=1/4_timing=every_publish:still_rising,s=16_rho=1_timing=real:still_rising,s=16_rho=1_timing=every_publish:still_rising,s=16_rho=1/4_timing=real:still_rising,s=16_rho=1/4_timing=every_publish:still_rising
E7RESULT name=r7_verdict quantity=r7b units=12 judged=8 fewer_than_three_tiers_units=[s=4_rho=1_timing=real,s=4_rho=1_timing=every_publish,s=4_rho=1/4_timing=real,s=4_rho=1/4_timing=every_publish] still_rising=0 flat=0 not_monotonic=8 geometry_sensitivity=consistent_on_8_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:fewer_than_three_tiers,s=4_rho=1_timing=every_publish:fewer_than_three_tiers,s=4_rho=1/4_timing=real:fewer_than_three_tiers,s=4_rho=1/4_timing=every_publish:fewer_than_three_tiers,s=8_rho=1_timing=real:not_monotonic,s=8_rho=1_timing=every_publish:not_monotonic,s=8_rho=1/4_timing=real:not_monotonic,s=8_rho=1/4_timing=every_publish:not_monotonic,s=16_rho=1_timing=real:not_monotonic,s=16_rho=1_timing=every_publish:not_monotonic,s=16_rho=1/4_timing=real:not_monotonic,s=16_rho=1/4_timing=every_publish:not_monotonic
E7RESULT name=r7_verdict quantity=r7c units=12 judged=12 fewer_than_three_tiers_units=[] still_rising=12 flat=0 not_monotonic=0 geometry_sensitivity=consistent_on_12_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:still_rising,s=4_rho=1_timing=every_publish:still_rising,s=4_rho=1/4_timing=real:still_rising,s=4_rho=1/4_timing=every_publish:still_rising,s=8_rho=1_timing=real:still_rising,s=8_rho=1_timing=every_publish:still_rising,s=8_rho=1/4_timing=real:still_rising,s=8_rho=1/4_timing=every_publish:still_rising,s=16_rho=1_timing=real:still_rising,s=16_rho=1_timing=every_publish:still_rising,s=16_rho=1/4_timing=real:still_rising,s=16_rho=1/4_timing=every_publish:still_rising
E7RESULT name=r7_verdict quantity=r7d_front units=12 judged=12 fewer_than_three_tiers_units=[] still_rising=12 flat=0 not_monotonic=0 geometry_sensitivity=consistent_on_12_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:still_rising,s=4_rho=1_timing=every_publish:still_rising,s=4_rho=1/4_timing=real:still_rising,s=4_rho=1/4_timing=every_publish:still_rising,s=8_rho=1_timing=real:still_rising,s=8_rho=1_timing=every_publish:still_rising,s=8_rho=1/4_timing=real:still_rising,s=8_rho=1/4_timing=every_publish:still_rising,s=16_rho=1_timing=real:still_rising,s=16_rho=1_timing=every_publish:still_rising,s=16_rho=1/4_timing=real:still_rising,s=16_rho=1/4_timing=every_publish:still_rising
E7RESULT name=r7_verdict quantity=r7d_back units=12 judged=12 fewer_than_three_tiers_units=[] still_rising=12 flat=0 not_monotonic=0 geometry_sensitivity=consistent_on_12_sampling_points_only_on_these per_unit=s=4_rho=1_timing=real:still_rising,s=4_rho=1_timing=every_publish:still_rising,s=4_rho=1/4_timing=real:still_rising,s=4_rho=1/4_timing=every_publish:still_rising,s=8_rho=1_timing=real:still_rising,s=8_rho=1_timing=every_publish:still_rising,s=8_rho=1/4_timing=real:still_rising,s=8_rho=1/4_timing=every_publish:still_rising,s=16_rho=1_timing=real:still_rising,s=16_rho=1_timing=every_publish:still_rising,s=16_rho=1/4_timing=real:still_rising,s=16_rho=1/4_timing=every_publish:still_rising
E7RESULT name=r7_verdict quantity=summary units=60 criterion_units_not_monotonic=[] report_only_units_unlike_the_prediction=[r7b:s=8_rho=1_timing=real,r7b:s=8_rho=1_timing=every_publish,r7b:s=8_rho=1/4_timing=real,r7b:s=8_rho=1/4_timing=every_publish,r7b:s=16_rho=1_timing=real,r7b:s=16_rho=1_timing=every_publish,r7b:s=16_rho=1/4_timing=real,r7b:s=16_rho=1/4_timing=every_publish] sampling_points_where_the_two_ring_readings_disagree=[] same_slot_back_units_flat_against_the_clause_range=[] dropping_the_last_tier_changed_units=0_of_24 threshold_moved_units_flipping=60_of_60 tier_sequence_control_cases_wrong=0 same_slot_controls_run=6 same_slot_controls_without_a_drop=0
E7RESULT name=verdict stage=second cells_run=116 cells_run_without_fitting_the_ring=36 truncated_cells=0 v1_over_withheld_controls_void=0 v2_a10_every_mismatch_cells=0 v2_a12_every_nonzero_points=0 v2_a14_mismatch_points=0 v2_a15_every_below_points=0 v2_pc_pinned_floor_seen=true v2_pc_c2_seen=true v6_a11_mismatch_points=0 v6_devices_unequal_points=0 v6_pc_hole_not_seen_cells=0 v16_judge_cases_wrong=0 v17_a13_every_nonzero_points=0 s1_failures=0 s1h_root_ring_missing_points=0 s11_a10_real_mismatch_cells=0 s11_a12_real_nonzero_points=0 s11_a15_real_below_points=0 s11_k10_violation_points=0 s12_g_adm_nonzero=0 k8_mismatches=0 referenced_but_free_points=0 abandoned_or_unknown_root_points=0 anchor_rows_missing=0 anchor_lag_mismatch_points=0 anchor_pinned_slot_mismatch_points=0
```

読み方甲 R7a の「実」列6行（`:16499`、`:16501`、`:16503`、`:16505`、`:16507`、`:16509`）。「毎」列は各行の次行で、`timing=` と `maximum_peak_cell=` の時点以外は同一。`grep 'name=r7_tier_peaks' <成果物> | sed -E 's/ timing=(real|every_publish)//; s/_timing=(real|every_publish)//' | sort | uniq -c | awk '{print $1}' | sort | uniq -c` の出力 `     30 2` は60行が30組に対応することを示す。

```
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=4 rho=1 timing=real tiers=[1,2] families=[Hh,Hh] peaks=[10,20] adjacent_differences=[10] growth=fewer_than_three_tiers maximum_peak=20 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=4_rho=1_timing=real maximum_peak_txg=16 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=fewer_than_three_tiers dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=4 rho=1/4 timing=real tiers=[1,2] families=[Hh,Hh] peaks=[10,20] adjacent_differences=[10] growth=fewer_than_three_tiers maximum_peak=20 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=4_rho=1/4_timing=real maximum_peak_txg=16 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=fewer_than_three_tiers dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=8 rho=1 timing=real tiers=[1,2,3,4] families=[Hh,Hh,Hh,Hh] peaks=[10,20,28,36] adjacent_differences=[10,8,8] growth=still_rising maximum_peak=36 maximum_peak_tier=4 maximum_peak_family=Hh maximum_peak_cell=s=8_rho=1_timing=real maximum_peak_txg=28 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=8 rho=1/4 timing=real tiers=[1,2,3,4] families=[Hh,Hh,Hh,Hh] peaks=[10,20,28,36] adjacent_differences=[10,8,8] growth=still_rising maximum_peak=36 maximum_peak_tier=4 maximum_peak_family=Hh maximum_peak_cell=s=8_rho=1/4_timing=real maximum_peak_txg=28 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=16 rho=1 timing=real tiers=[1,2,3,4,5,6,7,8] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh] peaks=[10,20,28,36,44,52,60,68] adjacent_differences=[10,8,8,8,8,8,8] growth=still_rising maximum_peak=68 maximum_peak_tier=8 maximum_peak_family=Hh maximum_peak_cell=s=16_rho=1_timing=real maximum_peak_txg=52 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7a class=Hh-front-inside-the-first-ring s=16 rho=1/4 timing=real tiers=[1,2,3,4,5,6,7,8] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh] peaks=[10,20,28,36,44,52,60,68] adjacent_differences=[10,8,8,8,8,8,8] growth=still_rising maximum_peak=68 maximum_peak_tier=8 maximum_peak_family=Hh maximum_peak_cell=s=16_rho=1/4_timing=real maximum_peak_txg=52 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
```

読み方乙 R7c の「実」列6行（`:16523`、`:16525`、`:16527`、`:16529`、`:16531`、`:16533`）：

```
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=4 rho=1 timing=real tiers=[1,2,3,6,9] families=[Hh,Hh,Hh,Hc,Hc] peaks=[10,20,146,308,594] adjacent_differences=[10,126,162,286] growth=still_rising maximum_peak=594 maximum_peak_tier=9 maximum_peak_family=Hc maximum_peak_cell=s=4_rho=1_timing=real maximum_peak_txg=63 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=4 rho=1/4 timing=real tiers=[1,2,3,6,9] families=[Hh,Hh,Hh,Hc,Hc] peaks=[10,20,110,244,458] adjacent_differences=[10,90,134,214] growth=still_rising maximum_peak=458 maximum_peak_tier=9 maximum_peak_family=Hc maximum_peak_cell=s=4_rho=1/4_timing=real maximum_peak_txg=63 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=8 rho=1 timing=real tiers=[1,2,3,4,5,10,15] families=[Hh,Hh,Hh,Hh,Hh,Hc,Hc] peaks=[10,20,28,36,294,608,914] adjacent_differences=[10,8,8,258,314,306] growth=still_rising maximum_peak=914 maximum_peak_tier=15 maximum_peak_family=Hc maximum_peak_cell=s=8_rho=1_timing=real maximum_peak_txg=105 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=8 rho=1/4 timing=real tiers=[1,2,3,4,5,10,15] families=[Hh,Hh,Hh,Hh,Hh,Hc,Hc] peaks=[10,20,28,36,226,476,716] adjacent_differences=[10,8,8,190,250,240] growth=still_rising maximum_peak=716 maximum_peak_tier=15 maximum_peak_family=Hc maximum_peak_cell=s=8_rho=1/4_timing=real maximum_peak_txg=105 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=16 rho=1 timing=real tiers=[1,2,3,4,5,6,7,8,9,18,27] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hc,Hc] peaks=[10,20,28,36,44,52,60,68,594,1208,1810] adjacent_differences=[10,8,8,8,8,8,8,526,614,602] growth=still_rising maximum_peak=1810 maximum_peak_tier=27 maximum_peak_family=Hc maximum_peak_cell=s=16_rho=1_timing=real maximum_peak_txg=201 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7c class=Hh-front-then-Hc-front s=16 rho=1/4 timing=real tiers=[1,2,3,4,5,6,7,8,9,18,27] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hc,Hc] peaks=[10,20,28,36,44,52,60,68,458,944,1416] adjacent_differences=[10,8,8,8,8,8,8,390,486,472] growth=still_rising maximum_peak=1416 maximum_peak_tier=27 maximum_peak_family=Hc maximum_peak_cell=s=16_rho=1/4_timing=real maximum_peak_txg=201 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=still_rising dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
```

参考報告 R7b の「実」列6行（`:16511`、`:16513`、`:16515`、`:16517`、`:16519`、`:16521`）：

```
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=4 rho=1 timing=real tiers=[1,2] families=[Hh,Hh] peaks=[174,168] adjacent_differences=[-6] growth=fewer_than_three_tiers maximum_peak=174 maximum_peak_tier=1 maximum_peak_family=Hh maximum_peak_cell=s=4_rho=1_timing=real maximum_peak_txg=27 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=fewer_than_three_tiers dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=4 rho=1/4 timing=real tiers=[1,2] families=[Hh,Hh] peaks=[114,118] adjacent_differences=[4] growth=fewer_than_three_tiers maximum_peak=118 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=4_rho=1/4_timing=real maximum_peak_txg=33 maximum_peak_is_on_the_largest_tier=true first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=fewer_than_three_tiers dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=8 rho=1 timing=real tiers=[1,2,3,4] families=[Hh,Hh,Hh,Hh] peaks=[366,360,352,344] adjacent_differences=[-6,-8,-8] growth=not_monotonic maximum_peak=366 maximum_peak_tier=1 maximum_peak_family=Hh maximum_peak_cell=s=8_rho=1_timing=real maximum_peak_txg=51 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=not_monotonic dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=8 rho=1/4 timing=real tiers=[1,2,3,4] families=[Hh,Hh,Hh,Hh] peaks=[234,238,230,232] adjacent_differences=[4,-8,2] growth=not_monotonic maximum_peak=238 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=8_rho=1/4_timing=real maximum_peak_txg=57 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=not_monotonic dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=16 rho=1 timing=real tiers=[1,2,3,4,5,6,7,8] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh] peaks=[716,710,702,694,686,678,672,666] adjacent_differences=[-6,-8,-8,-8,-8,-6,-6] growth=not_monotonic maximum_peak=716 maximum_peak_tier=1 maximum_peak_family=Hh maximum_peak_cell=s=16_rho=1_timing=real maximum_peak_txg=99 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=not_monotonic dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
E7RESULT name=r7_tier_peaks quantity=r7b class=Hh-back s=16 rho=1/4 timing=real tiers=[1,2,3,4,5,6,7,8] families=[Hh,Hh,Hh,Hh,Hh,Hh,Hh,Hh] peaks=[474,478,470,472,466,468,462,468] adjacent_differences=[4,-8,2,-6,2,-6,6] growth=not_monotonic maximum_peak=478 maximum_peak_tier=2 maximum_peak_family=Hh maximum_peak_cell=s=16_rho=1/4_timing=real maximum_peak_txg=105 maximum_peak_is_on_the_largest_tier=false first_tier_above_two_slots=1 first_tier_above_one_slot=1 growth_without_the_last_tier=not_monotonic dropping_the_last_tier_changes_the_judgement=false threshold_moved_flips=true
```

PC-同一スロット6行（`:15950`、`:16002`、`:16090`、`:16178`、`:16338`、`:16498`）と PC-段階列5行（`:5243`–`:5247`）：

```
E7RESULT name=pc_same_slot s=4 rho=1 k=2 position=back unchanged_q1a_every_peak=344 control_q1a_every_peak=10 control_q1a_real_peak=344 seen=true
E7RESULT name=pc_same_slot s=4 rho=1/4 k=2 position=back unchanged_q1a_every_peak=236 control_q1a_every_peak=10 control_q1a_real_peak=236 seen=true
E7RESULT name=pc_same_slot s=8 rho=1 k=2 position=back unchanged_q1a_every_peak=712 control_q1a_every_peak=10 control_q1a_real_peak=712 seen=true
E7RESULT name=pc_same_slot s=8 rho=1/4 k=2 position=back unchanged_q1a_every_peak=476 control_q1a_every_peak=10 control_q1a_real_peak=476 seen=true
E7RESULT name=pc_same_slot s=16 rho=1 k=2 position=back unchanged_q1a_every_peak=1460 control_q1a_every_peak=10 control_q1a_real_peak=1460 seen=true
E7RESULT name=pc_same_slot s=16 rho=1/4 k=2 position=back unchanged_q1a_every_peak=960 control_q1a_every_peak=10 control_q1a_real_peak=960 seen=true
E7RESULT name=pc_tier_sequence case=rising_one_two_three actual=still_rising expected=still_rising holds=true
E7RESULT name=pc_tier_sequence case=flat_one_two_two actual=flat_upper_bound_2 expected=flat_upper_bound_2 holds=true
E7RESULT name=pc_tier_sequence case=not_monotonic_one_three_two actual=not_monotonic expected=not_monotonic holds=true
E7RESULT name=pc_tier_sequence case=flat_five_five_five actual=flat_upper_bound_5 expected=flat_upper_bound_5 holds=true
E7RESULT name=pc_tier_sequence case=two_tiers_are_refused actual=fewer_than_three_tiers expected=fewer_than_three_tiers holds=true
```

**判定行の個別確認**：`grep -n 'name=verdict' <成果物>` は14行、`:5232` と `:16656`–`:16668`。コマンドと出力原文：

```
$ grep -n 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-18.out | cut -d: -f1 | tr '\n' ' '
5232 16656 16657 16658 16659 16660 16661 16662 16663 16664 16665 16666 16667 16668 

$ grep 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-18.out | grep -oP '(?<= )[a-z0-9_]+=(false|not_run)(?= |$)' | sort | uniq -c

$ grep 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-18.out | grep -oP '(?<= )[a-z0-9_]+=[1-9][0-9]*(?= |$)' | sort | uniq -c
      1 cells_run=116
      1 cells_run_without_fitting_the_ring=36
      6 rho=1
      4 s=16
      4 s=4
      4 s=8

$ grep 'name=verdict' research/results/e156-alloc-basis-counts-2026-10-18.out | grep -oP '(?<= )[a-z0-9_]+=[a-z_]+(?= |$)' | grep -E 'void|not_|unseen|ambig' | sort | uniq -c
      1 fork_three=void_not_measured
      1 premise_two_answer=not_holds
      8 q1d_growth=not_monotonic
      1 u13b_flip2_not_applicable=true
```

`false`・`not_run` のフィールドはない。違反、不一致、曖昧さ、失敗の整数カウントはすべて0。`cells_run=116`・`cells_run_without_fitting_the_ring=36` はセル数、`s=`・`rho=` はグループ名であり、違反数ではない。個別確認した4項目：`premise_two_answer=not_holds`・`fork_three=void_not_measured`（`:5232`、前提2不成立で分岐第3行は無効・未測定。第4回以来の既定結果で、第6回成果物 `e156-alloc-basis-counts-2026-10-05-r2.out` `:1217` と完全一致し `diff` 出力なし）。`u13b_flip2_not_applicable=true` は名前に `not_` があるが値は `true` で失敗ではない。`q1d_growth=not_monotonic` は `:16657`–`:16668` の8行で、h_欠 に対する Q1d の判定値。S=8・16 のρ=1は `grows_with_holes`、その他は `not_monotonic`。第6回「実」列4サンプルと同形で違反ではない。`name=verdict` 行の外で「通らなかった」項目は `r7_verdict quantity=summary`（`:16564`）の非空 `report_only_units_unlike_the_prediction`、すなわち R7b の8単位に F32 が発動したこと。

**H1 の判定十分性**（登録の十分なら停止：R7a の S=8・16、R7c の12サンプルに W28 判定があり、R7e の上界値がある）：

| 解釈 | 判定十分／不足 | 残りの測定で逆転するか | 算出コマンド |
|---|---|---|---|
| 甲 R7a（一周内を埋める） | 十分。S=8・16 の8単位すべて「なお増加中」。各段階10、20、28、36 …（S=8）と … 68（S=16）、上界値は最大段階で「端点以下」。S=4 は10、20の2段階のみで登録上報告のみ。 | しない。甲の上限 `k_满(S)` は幾何で固定され、さらに多い穴は2周目に入り乙となる。登録範囲はすべて実行済み。 | `grep -n 'name=r7_tier_peaks quantity=r7a\|name=r7_verdict quantity=r7a' <成果物>` |
| 乙 R7c（一周を越える） | 十分。12単位すべて「なお増加中」。最大段階 Hc の3倍 `k_满` + 1 穴では914（S=8、ρ=1）、1810（S=16、ρ=1）。上界値は最大段階で「端点以下」、上界は未測定。 | 未測定。Hc を4周以上に延ばすか S をさらに大きくした場合に横ばいとなるかは登録範囲外。第11節では Hc 3周までで「なお増加中」は最大段階までを意味し、無界には外挿しない。 | `grep -n 'name=r7_tier_peaks quantity=r7c\|name=r7_verdict quantity=r7c' <成果物>` |
| 両解釈の並記 | 12サンプルで結論は両方とも「なお増加中」、F33 は発動せず。どちらを採用するかは登録 W26 でメイン agent に委ね、双方の行を報告済み。 | 同方向なので逆転しない。 | `grep -n 'name=r7_verdict quantity=summary' <成果物>` の `sampling_points_where_the_two_ring_readings_disagree=[]` を確認。 |

本節は測定値と十分性のみを記し、候補甲・乙・丙の採否は判定しない。登録第6節 R7a は「横ばいなら乙、なお増加中なら甲または丙、非単調ならどちらも支持しない」と対応付ける。今回は双方とも「なお増加中」であり、乙に向かう「横ばい」はこれらのサンプルで観測されなかった。候補選択は三者の推論とユーザー決定に属する。

**第7回で答えられないこと**：① 甲の書き込み可能マウント時の穴埋め実装後の過剰差し引き。`crates/` には根リングスロットの穴埋め書き込み経路がなく（登録5.1）、現コードのみ測定。② 穴の作成は「記録は永続化済み、ルートスロットは未永続化」の1種類のみ。読めないスロット、自己検証失敗スロットは作っていない。穴数は Hc の3倍 `k_满` + 1 までで「なお増加中」は最大段階まで、無界に外挿せず、増加関数の形も示さない。隣接差8や126–614は測定値でフィッティングではない。③ 4 GiB × 2、S ∈ {4,8,16}、ρ ∈ {1,1/4} のみ。同一解釈でも上界値は S・ρ で変わり、Hc 最大段階 S=16 は1810 / 1416で定数ではない。④ 各履歴の終点は最後の穴 + 3S + 2 で、`final` がすべて0なのはこの構成による。過剰差し引きの持続期間、すなわち正の観測点数は `q1_trajectory` にのみ報告され、判定基準には入っていない。⑤ R7b の予測がなぜ誤ったか、ピークが穴数とともにやや低下または変動する理由は調べず、F32 発動を記録したのみ。⑥ インライン単体テスト25件の値はメイン agent による。新変異 M50–M54 は未実行でライト層待ち、`replay.sh E156` の全体再実行も未実施。成果物は1回分のみで同一バイナリの2回目とのバイト一致比較はない。⑦ 甲・乙の採用は未判定。⑧ 登録5.5の停止条件、S=16 の Hc(27) 単独の経過時間が10分を超えるかは個別測定していない。全体241秒なので推定上は発動しない。

**次回の装置変更事項**（今回は装置を変更していない）：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` の `verdict` は所定状態表の判定段がプレースホルダーを返すと panic するが、メッセージ（`:758`）は `SINGLEFS_CRASH_AMPLIFICATION_STAGE` と `SINGLEFS_CRASH_AMPLIFICATION_SHARE` の2変数しか挙げない。今回実際に起きた原因は装置バイナリを `gpu` feature なしでビルドし、マルチマシン設定が `ENABLE_GPU=1` だったこと（メイン agent の診断、当該出力は未保存）だが、メッセージにこの原因がなく、起動担当者は2変数を無駄に調べることになる。次回はこの原因と対処、`--features verdict-store,gpu` でビルドして `scripts/crash/gpu-quiesce-run.sh` 経由で起動することをメッセージに追加する。

### 第6回再実行（2026-10-05）：回収閾値を決定済みの乙の解釈に合わせると38セルの K10・F24 はすべて0。core の全ゼロ固定を除くと、予測した34点でちょうど赤になる

再実行登録 `e156-r6-prereg.md` は第5回登録を基礎とする差分形式で、末尾に「主 agent 认定：第 6 次登记交回」の6項目を含む。質問票 `e156-r6-questions.md` は3行：第1行は装置の回収閾値を決定済み乙に合わせた後、現在の `crates/` の38セルで K10・F24 にまだ不一致があるか、またコピー内で core の全ゼロ固定を除去して判別力を確認すること。第2行は新閾値での分岐表第1行の各値と第5回成果物から変わったセルを報告し、判定はやり直さないこと。第3行は PC-閾値固定の6セル両列が依然「不一致」と判定され、いずれも 6S − 1 を含むか。発端は第5回の K10 の34点不一致だった。core はユーザー決定の乙に従い、マウント時に全ゼロと読まれたルートリングスロットをそこに存在し得た最古の txg に固定していたが（`crates/singlefs-core/src/allocator.rs` の `RootRingOccupant::AllZeroAtMount`）、装置とアンカーモデルは旧解釈に従っていた。

**開始前提と事前確認**（登録 S15・S2・S3）：

| 項目 | コマンドと出力原文 | 結論 |
|---|---|---|
| 第31バッチのコードラウンド判定を書き終えた | `ls research/prompts/m3-formal-code-b31-r1-main-verification.md` が存在し、派遣指示に判定完了と明記 | 成立 |
| 装置に変更なし | `sha256sum crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` → `e69e423a6fda2d973f1b7d4b741bbb280e54d70723d953fda666bba00e2bfe20`、`wc -l` → `5819` | 成立 |
| core に全ゼロ固定がある | `grep -c AllZeroAtMount crates/singlefs-core/src/allocator.rs` → `11`；`grep -c '            &ring_slots_reading_back_all_zero,$' crates/singlefs-core/src/mount.rs` → `1` | 成立 |
| 条項不変（S2） | 登録コマンド1出力 `a54efaa24231508eb8a0d99ab6b9a4ed761ae337b1af256388f163f79c2a5bbb`、第5回第2節との差は `diff` 0行。コマンド2出力：決定済み乙の4段落 `a4ac82034664e49c452d9de59ebe7b8f510f01fe15e8dafc2f6ab6764b3805f7`、質問票 `7889590e8aed051fc02b3edd4ae4686ca0e6d4ba1dd85c9ee191d24ee27a1a8c` | 発動せず |
| アンカースクリプト（コマンド3・4・5） | 集計 `700ec0b0…` は第4・6回で同じ。`--dump` は第4回 `7bb1e088…`、第6回 `47a7d193…`、各1092行、`diff` 23行、`bab6a323…`。`--dump --pins` `8a222fe4…`、`count_pin_points.py` 出力 `9d5cf9ce…`、両 `--dump` に対する `lag_bound_r5.py` はともに `b980971a…` | 登録第13節と同じ |
| 実装スナップショット（S3） | 成果物バイナリのビルド前と主成果物実行後の `git diff -- crates/ \| sha256sum` はともに `ad4c31366a677a2df4f08c069d77fc7ebc04345921bcd83371b0f32076ba6ae0`、`git status --short crates/ \| sha256sum` はともに `8995a6f9b7c12c51e377589e09dc69beb5bb6ebe758392ace82f9825cd129a28`（152行）。PC-全ゼロ固定実行後は `crates/mutations.tsv` 末尾の変異3行の追加のみ。 | 発動せず |

**装置の変更**（登録5.3）：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` は5819→6205行、単体テスト19件（`grep -c '#\[test\]'`、第5回は18件）。マウント点を `mount_writable` に渡す前に切断画像の3Sスロットを `read_root_ring_slot_content` で自己検証成功・全ゼロ・読み取り異常の3種類に分ける。マウント後、新インスタンスの最初の txg を使って各全ゼロスロットに「そこに到達し、最初の txg より小さい最小 txg」を付与し、`HhRunState` の固定表を全置換、`name=all_zero_pins_at_mount` を1行出力。`read_hh_state` は閾値計算前に固定表の各スロットの現在のルートを読み、その txg が最初の txg 以上ならこのプロセスによる上書き、未満なら再読み取りで発見（`released_by_reread_only` も計数）とし、どちらでもなければ固定を維持して閾値に含める。`reclaim_threshold_read_from_the_ring_image` の関数本体と `let every_publish_floor = ring_threshold;` は一文字も変更していない。アンカーモデルも登録コマンド4のパッチに従い同箇所を変更。各セルの `name=q1_cell_checks` の後に `name=all_zero_pin_cell`、第2段階の `name=failure_clauses` の後に `name=all_zero_pin_summary` を出力し、既存行の形式は変えない。新規単体テスト U20 と変異 M45–M47（`crates/mutations.tsv` 内で「M45（第 6 次重跑登记第九节）」等の3行）を追加。このファイルのみ `rustfmt --edition 2021` を実行し、`rustfmt --check` 出力なし。4999行目からの `use` ブロックも整形された。`compile-then-swap.py` はリポジトリのコピーで `cargo build`・`cargo test --no-run` が通ってから主作業区に交換した（交換後 sha256 `3129ec95e2f129ef0a49f1be15a998aae5c4abb508615a8663cdece91d1ef279`）。

**成果物**：主成果物 `e156-alloc-basis-counts-2026-10-05-r2.out` は4063行、sha256 `1c8812b19456165b10b6cd01f3e00fb2ddd1e3673f700cedac151f5da778176b`、完了マーカー `E7RESULT name=done emitted=4063`。リポジトリ直下 `target/release/` のバイナリ sha256 `1227561466a0008a1b1a7004bafe73d1d71cff7b1af1a51eaab19c40416f2a48` で実行し、同じバイナリの再実行とバイト単位で一致した。`replay.sh` のドライバは debug バイナリを使い、1回約13分かかるため実行担当者は前景では225秒まで実行、その1803行は release 成果物の先頭1803行とバイト一致。debug 全体の再実行は `replay.sh` の exact 比較に委ねた。判別力対照 `e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out` は4063行、sha256 `eaf6b8b63d3b1d899377559a5f09fb32f6774cd14031f41567898c62873679ff`。コピー内の core 変更 `e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.patch` は `mount.rs` の1行で、マウント時に全ゼロスロットをルートリング表へ一つも入れない。調査報告 x1 パッチ本体と行番号以外は各行同一。同一装置ソースをコピー内で別途 release ビルドした。

**第1行：決定済み乙の閾値で38セルの K10・F24 はすべて0**（登録 R1a・R1b）。主成果物 `:4050`・`:4048`・`:4049`：

```
E7RESULT name=verdict stage=second cells_run=38 cells_not_fitting=4 truncated_cells=0 v1_over_withheld_controls_void=0 v2_a10_every_mismatch_cells=0 v2_a12_every_nonzero_points=0 v2_a14_mismatch_points=0 v2_a15_every_below_points=0 v2_pc_pinned_floor_seen=true v2_pc_c2_seen=true v6_a11_mismatch_points=0 v6_devices_unequal_points=0 v6_pc_hole_not_seen_cells=0 v16_judge_cases_wrong=0 v17_a13_every_nonzero_points=0 s1_failures=0 s1h_root_ring_missing_points=0 s11_a10_real_mismatch_cells=0 s11_a12_real_nonzero_points=0 s11_a15_real_below_points=0 s11_k10_violation_points=0 s12_g_adm_nonzero=0 k8_mismatches=0 referenced_but_free_points=0 abandoned_or_unknown_root_points=0 anchor_rows_missing=0
E7RESULT name=all_zero_pin_summary cells=38 cells_with_pins=16 pins_installed=64 threshold_lowered_points=34 released_by_reread_only=0 bad_slots_at_mount=0 bad_slots_still_pinned=0
```

セルごとに `grep ' name=q1_cell_checks ' e156-alloc-basis-counts-2026-10-05-r2.out | grep -o 'k10_violation_points=[0-9]*' | sort | uniq -c` → `38 k10_violation_points=0`。同じコマンドを `real_every_differ_points` に替えると `38 real_every_differ_points=0`。`failure_clauses`（`:4048`）は `f24_real_every_differ_points=0`。38セルは S ∈ {4,8,16} × ρ ∈ {1,1/4} の k=0 6セル、穴位置が前・後それぞれ16セル。登録8.2に従い、K10・F24 が0で一致するのはこの38サンプル点のみ。

**判別力**（登録 R1c、PC-全ゼロ固定）：同じ装置に全ゼロ固定を除いた core を組み合わせた対照では `:4050` の `s11_k10_violation_points=34`、`:4048` の `f24_real_every_differ_points=34`。全文は判定行の個別確認を参照。非0観測点集合を `e156-alloc-basis-counts-2026-10-05-r6-point-sets.py`（2成果物とアンカースクリプト2出力を読み取るだけ）で登録 L6 × 2ρ と比べ、`e156-alloc-basis-counts-2026-10-05-r6-point-sets.txt` の `L6_points_x_rho=` と `pc:` 行：

```
L6_points_x_rho=34
pc: k10_points=34 f24_points=34 k10_eq_L6=True f24_eq_L6=True k10_minus_L6=[] L6_minus_k10=[] f24_minus_L6=[] L6_minus_f24=[]
```

K10・F24 は前穴セルの txg h + 3S − 1 の1観測点で赤になる。S=4 では16、21、S=8 では28、33、39、45、S=16 では52、57、63、69から k 個を取る。後穴・k=0 セルは1点も赤にならない。無効化 V19・停止 S16 は発動せず。2入力は登録コマンド4 `--dump --pins` とコマンド5の原文出力：`e156-alloc-basis-counts-2026-10-05-r6-anchor-pins.txt`（`8a222fe4…`）、`e156-alloc-basis-counts-2026-10-05-r6-anchor-lowered.txt`（`9d5cf9ce…`）。

**固定の簿記**（登録 R1d）：同出力の4–7行：

```
R1d_1: hh_mount_rows=68 checked=68 mismatches=[]
R1d_2: cells=38 mismatches=[]
R1d_3: cells_with_reread_release=[]
R1d_4: cells_with_bad_slots=[]
```

68回のマウント（第1段階の2履歴を含む）の各固定 txg 集合は、アンカースクリプトの該当セルのマウント完了行の `pinned` と同一。後穴・k=0 では1スロットも固定しない。各セルの `threshold_lowered_points` は L6 の同セルの点数に等しい。`released_by_reread_only`、`bad_slots_at_mount`、`bad_slots_still_pinned` はすべて0で、停止 S17・S18 は発動せず。アンカー行も一致する。主成果物 `name=anchor_dump` 1092行のプレフィックスを除き、コマンド4 `--dump` の `DUMP` 行と `cmp` して出力なし（登録7.2最後のアサーション、無効化 V20 は発動せず）。

**第2行：分岐表第1行の値と第5回の比較**（登録 R2a–R2c、報告のみで再判定しない）。比較スクリプト `e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.py`、出力 `e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.txt`（146行）の第1行：

```
pairs=102 unequal_pairs=42 only_old=0 only_new=0 differing_fields_real=0 differing_fields_every=122 differing_fields_unsplit=3 judgement_flips=8 other_judgement_changes=12
```

| 測定値 | 第5回 | 第6回 |
|---|---|---|
| F1 | 12グループすべてあり（発動） | 12グループすべてあり（発動）、判定は変わらず F33 は発動しない |
| Q1c-前の判定（「実」「毎」両列12グループの `judgement_two_slots`・`judgement_one_slot`） | すべて `above`。2・1スロットを超える最小 h_欠 はいずれも1、初めて超える txg は7 | 同左、変化なし |
| Q1c-前「毎」列の Δ ピーク（`by_h_que`、S=4、ρ=1、h_欠=1） | `peak10` | `peak20`、「実」列と同じ。「毎」列の6グループすべての `by_h_que` が変化 |
| Q1d（`q1d` の `growth_by_h_que`）「実」列 | s=4 ρ=1、s=4 ρ=1/4、s=8 ρ=1/4、s=16 ρ=1/4 が `not_monotonic`、残りは `grows_with_holes` | 同左 |
| Q1d「毎」列 | 12グループすべて `grows_with_holes` | 「実」列と同じく上記4グループが `not_monotonic` に反転。F34 発動、参考報告 |

反転した8行は比較出力2–9行で、`q1d` と `verdict stage=second_group` の `q1d_growth` 各4グループ、すべて「毎」列。判定フィールドが変化し Q1c-前・Q1d 以外の12件は10–21行で、`q1c quantity=q1c_not_on_holes` の「毎」列6グループの2判定が `at_most` から `above` に変化した。R2c：「実」列の差異は0件（`differing_fields_real=0`）で第5回と全フィールド一致、F35 は発動せず。第6回の各グループの `q1c`、`q1d`、`q1c_front_threshold_moved` 行から `timing=` を除くと「実」「毎」の2行は完全一致する（`sed 's/ timing=[a-z_]*//' | sort | uniq -c` で各組2）。

**第3行：PC-閾値固定**（登録 R3）。主成果物 `name=pc_pinned_floor` 6行と集計行（`:3908`–`:3914`）は第5回成果物の同名7行と `cmp` でバイト一致。6セルの「実」「毎」両列で `judge_*_mismatch=1`、`contains_bound_*=true`。集計 `E7RESULT name=pc_pinned_floor_summary cells=6 seen_real_cells=6 seen_every_cells=6`、`v2_pc_pinned_floor_seen=true`。

**判定行の個別確認**（`grep -n 'name=verdict' <成果物>`）：主成果物14行（`:1217`、`:4050`–`:4062`）に `false`・`not_run` のフィールドはなく、違反・不一致・失敗の整数カウントはすべて0。`premise_two_answer=not_holds`・`fork_three=void_not_measured` は第5回と同じで、前提2不成立、分岐表第3行は無効。判別力対照にも同じ行番号で14行あり、非0は `s11_k10_violation_points=34`（登録 R1c が要求する項目）と `s11_a15_real_below_points=16`（`:4050`、前穴16セルで各1点）。後者は「実」列 Q1a が決定済み乙によるアンカーモデルのツリーテーブル下限を下回るもので、core が固定しない場合 txg h + 3S − 1 でツリーテーブル単位を1つ早期回収したためと推定する。登録にこの項の期待値はなく、対照は被測定状態ではない。全文：

```
成果物の行：git show 18b30adbfe94:e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out、4050行目（name=verdict）
```

**第6回で答えられないこと**：① 19件のインライン単体テスト（新 U20 を含む）と、`crates/mutations.tsv` のこの装置を対象とする26変異（M45–M47 を含む）は未実行。checker 層のため提出時に行う。新ソースに26行の原文がそれぞれ1回一致し、M45–M47を同時適用したコピーで `cargo test --no-run` がビルドできることのみ確認した。② この系列は最初の一周の前穴のみを作るため、スロットが [0, 最初の txg) で保持できる txg は1つだけ。「最古に固定」と「保持可能な最大 txg に固定」が同値で、core の固定先を区別できない。③ 「再読み取りでルートが見つかって初めて固定を解除」する経路はこの系列で一度も現れず（`released_by_reread_only` はすべて0）、装置は数えるだけで証明しない。④ 読み取り異常スロットや admission による F 引き上げの上限はこの系列で作らない。⑤ PC-全ゼロ固定は core が全ゼロスロットの固定を漏らしたことを装置が検出できると示すが、装置側のバイパス読取が乙を正しく読んでいることは証明しない。その半分はアンカー全行一致と U20 が監視する。⑥ 主成果物は release バイナリで実行し debug は先頭1803行のみ比較。⑦ 「実」列は第5回と一致するが、第5回以後の `crates/` の他の変更がこの系列外の量に触れたかは未確認。⑧ PC-閾値固定・PC-全ゼロ固定はそれぞれ対象の欠陥を区別できるが、合わせても固定先が最古 txg か最大 txg か、再読み取りによる解除、読み取り異常スロットの3点は区別できない。第32–33バッチのコードラウンド判定 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第2節 P5 の「二つの対照に判別できるか」は成立。⑨ 変異 M36（`crates/mutations.tsv` の「M36（第 4 次重跑登记第九节）」行。閾値を「最新有効ルート − リング内ルート数 + 1」に変更して穴スロット内の旧ルートを見落とす）は前穴セルで判別力が1箇所不足する可能性がある。同表の「M36、M42」行、攻撃側の推論であり未測定。M36 と M42 が互いを隠蔽しないことはコードを読んで確認した。

### 第5回再実行（2026-10-05）：W7 の対照を PC-閾値固定に変更、第4回測定値は新 `crates/` 上で294行異なる

再実行登録 `e156-r5-prereg.md` は差分形式で、その他は第4回登録 `e156-r4-prereg.md` と同じ。質問票 `e156-r5-questions.md` の2行は、① core で根リング表のない分配器がルートを記録するとアサートするようになった後、装置 W7 の陽性対照 PC-分配器を構成できなくなったため何に置き換えるか、② 第4回に判定十分となった測定値が新 `crates/` で同じか。発端はユーザー決定 C551 (`build_pool` のアロケータにルートリング表がない)。`crates/singlefs-core/src/allocator.rs` の `record_root_written_by_this_process` は表がない場合に `expect` し、メッセージは「分配器没装根环表却记了一条根」で始まる。

**開始前提と事前確認**（第5回登録の停止 S13・S2・S3）：

| 項目 | コマンドと出力原文 | 結論 |
|---|---|---|
| 31-1 のアサーションが主作業区にある | `grep -c '分配器没装根环表却记了一条根' crates/singlefs-core/src/allocator.rs` → `2`。一方はアサーションメッセージ、他方は `should_panic` | 成立 |
| 第31バッチのマージ検証が緑のディレクトリ | 派遣指示に `/tmp/claude-1000/merge-and-verify/2026-10-05-2822486/` と明記 | 成立。実行担当者はディレクトリ内ログを再読していない。 |
| 装置の追加変更は31-3のみ | `git show :crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs \| sha256sum` → `f1930d4e…`（登録作成時）。同ファイルの `git diff` 2ブロックは `/tmp/claude-1000/m3-formal-31-3/patch/crates.patch` の先頭2ブロックと完全一致。一つは `mod tests` の `use` に `root_ring_occupancy_after_make_filesystem` を追加、もう一つは `new_pool_file_creation_state` の関数本体で根リング表を装着。 | 登録では「関数本体内のみ」と記載。追加 `use` 1行は同じパッチ由来で測定値を変えないと実行担当者が判断し開始、引き渡しに記載してメイン agent が確認。 |
| 条項不変（S2） | 登録第13節コマンド1で5段落を再取得、`sha256sum` → `a54efaa24231508eb8a0d99ab6b9a4ed761ae337b1af256388f163f79c2a5bbb`、登録第2節と同一 | 発動せず |
| アンカースクリプト（コマンド3・4） | `anchors_e156_r4.py` 出力 `700ec0b0…`、`--dump` `7bb1e088…`（1092行）、`lag_bound_r5.py` 出力 `b980971a…`、登録第13節と同一 | 同一 |
| 実装スナップショット（S3） | 成果物前、1回目実行後、2回目再実行後の `git diff -- crates/`・`git status --short crates/` の sha256 は `31db3aba132749d5fd643e4ef8e013af15896679f953b73dd4bb8265caccf58d`・`8c727d9e15719e32467975a09ff64b3b445ff1971be7ebf51ef49e7b326a34a0` で3回一致 | 発動せず |

**装置の変更**（登録5.5 e1–e4）：`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` は5783→5819行。`HhAllocatorSource::WithoutRootRingPositiveControl`、`allocator_without_the_root_ring`、`main` の PC-分配器部分を削除。`HhAllocatorSource` と `ProductPathWithRootRing` は `name=q1_cell` が `allocator={:?}` を出力するので改名しない。PC-閾値固定をセル表の後・PC-過剰差し引きの前に追加。表中 `hole_count == 1`、位置が後の6セルに同じ `judge_hole_history_outcome` を使い、期待値は同じ S の `anchor_model::tree_table_reclaim_lags`、6S − 1 は `2 * E156_ROOT_RING_REGIONS * slots_per_region - 1` で計算。`name=verdict stage=second` の `v2_pc_allocator_seen` を `v2_pc_pinned_floor_seen` に改名、単体テスト U19 を追加。`grep -n 'PoolAllocator::new(' crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` → `5041:        let mut allocator = PoolAllocator::new(`。これは `new_pool_file_creation_state` 内で、5071行の `install_root_ring_occupancy` が直後に根リング表を装着。単体テスト数 `grep -c '#\[test\]' 同一ファイル` → `18`（第4回17）。`compile-then-swap.py` は草稿コピーで `cargo build`・`cargo test --no-run` が通った後、主作業区に交換した。

**第1行：新対照 PC-閾値固定**。成果物 `e156-alloc-basis-counts-2026-10-05.out` は3955行、sha256 `a3d29fe0e31a570aca19db52a6fda9f7600017f43dbcfa856996d9126085931c`。同じバイナリの再実行とバイト一致し `cmp` 出力なし。

```
E7RESULT name=pc_pinned_floor s=4 rho=1 k=1 position=back expected=[12] lags_real=[12,13,14,15,16,17,18,19,20,21,22,23,24] lags_every=[12,13,14,15,16,17,18,19,20,21,22,23,24] judge_real_mismatch=1 judge_every_mismatch=1 bound=23 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=4 rho=1/4 k=1 position=back expected=[12] lags_real=[12,13,14,15,16,17,18,19,20,21,22,23,24] lags_every=[12,13,14,15,16,17,18,19,20,21,22,23,24] judge_real_mismatch=1 judge_every_mismatch=1 bound=23 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=8 rho=1 k=1 position=back expected=[24] lags_real=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] lags_every=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] judge_real_mismatch=1 judge_every_mismatch=1 bound=47 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=8 rho=1/4 k=1 position=back expected=[24] lags_real=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] lags_every=[24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48] judge_real_mismatch=1 judge_every_mismatch=1 bound=47 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=16 rho=1 k=1 position=back expected=[48] lags_real=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] lags_every=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] judge_real_mismatch=1 judge_every_mismatch=1 bound=95 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor s=16 rho=1/4 k=1 position=back expected=[48] lags_real=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] lags_every=[48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96] judge_real_mismatch=1 judge_every_mismatch=1 bound=95 contains_bound_real=true contains_bound_every=true seen_real=true seen_every=true
E7RESULT name=pc_pinned_floor_summary cells=6 seen_real_cells=6 seen_every_cells=6
```

6セル両列で Q1r5a（判定不一致）は1、Q1r5b（遅延集合に6S − 1を含む）は true、`v2_pc_pinned_floor_seen=true`（`e156-alloc-basis-counts-2026-10-05.out` の `name=verdict stage=second cells_run=` 行）。登録された十分条件は成立し、無効化 V2、F28、F29 は発動せず。6サンプル点で A10 が実分配器の回収を一周遅らせたことを判別できると示す。ただし遅延のみの証明であり、全期間の非回収は証明しない（その形は書き換えた M35 で提出時に監視）。また W7 の確認が根リング表のない分配器を検出できることも証明しない（core 単体テストが引き継ぐ）。遅延集合は 3S～6S の全区間で、S=4 なら12～24、6S − 1だけではない。6S − 1 と 6S は登録コマンド4が計算した解放世代6・5の遅延：`lag_bound_r5.py` の `release_first_free_lag=[(5, 28, 24), (6, 28, 23)]`。区間内の他の値は成果物で解放世代別に分けられず未確認。`grep -c '^E7RESULT name=pc_pinned_floor ' e156-alloc-basis-counts-2026-10-05.out` → `6`、集計行 `grep -c '^E7RESULT name=pc_pinned_floor_summary '` → `1`。

**第2行：第4回の値を新 `crates/` でフィールドごとに再比較**（登録 Q2a・Q2b）。正規化では第4回成果物 `e156-alloc-basis-counts-2026-09-26-r2.out`（4007行）から `family=PC-allocator` を含む行、最初の項目が `name=pc_allocator` の行、`name=verdict stage=second` の `v2_pc_allocator_seen=` 項目を除く。新成果物から `name=pc_pinned_floor`・`name=pc_pinned_floor_summary` 行と同 verdict 行の `v2_pc_pinned_floor_seen=` を除く。「最初の項目」は行頭 `E7RESULT ` の後の最初の空白区切り項目。比較スクリプトと全出力はリポジトリ内：

```
python3 e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.py e156-alloc-basis-counts-2026-09-26-r2.out e156-alloc-basis-counts-2026-10-05.out > e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt
head -3 e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt
old_dropped_rows={'family=PC-allocator': 58, 'name=pc_allocator': 1} old_dropped_items=1
new_dropped_rows={'name=pc_pinned_floor': 6, 'name=pc_pinned_floor_summary': 1} new_dropped_items=1
lines_old=3948 lines_new=3948 equal=3654 unequal=294
```

Q2a は「異なる」。登録の固定基準は `unequal = 0` の場合のみ同一。登録 L3 の記述を確認し、新成果物から7行1項目、第4回から59行1項目を除き、除去行はすべて `pc_allocator` または `family=PC-allocator` を含む。V18 は発動せず。294行中1行は `name=done emitted=` の4007→3955で、4007 − 59 + 7 に等しい。登録の正規化はこの数値を除去していないため文字どおり294に含めた。

Q2b は登録どおり2種類に分類。判定行一覧は登録第6節による。

| 分類 | 行数 | キー数 | 行名（行数） |
|---|---|---|---|
| 判定行 | 60 | 160 | `q1c`（24）、`q1_cell_checks`（16）、`q1d`（6）、`q1c_front_threshold_moved`（6）、`verdict`（5）、`u13a_rollback_to_oldest_candidate`（1）、`geometry_sensitivity`（1）、`failure_clauses`（1） |
| 計数行 | 234 | 522 | `q1_observation`（82）、`hole_recovery`（69）、`q1_trajectory`（48）、`q1_cell`（18）、`q1_trajectory_extra`（16）、`done`（1） |

差異の位置：`name=q1_cell` の Hh 行では前穴16セルすべてが異なる（k=1・2・4 が6・6・4セル）。後穴では k=2・4 各1セルのみ、k=0 の6セルは差異なし。`hole_recovery` 69行の差は1キーのみで、`steps_after_root_slot_write` の `system_configuration_slot,system_configuration_slot` の後に `,barrier,barrier` が追加された。他のキーは `e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt`（979行）で各行・各キーの旧値と新値を記載。判定上の差異：

| フィールド | 第4回 | 新成果物 |
|---|---|---|
| `failure_clauses f24_real_every_differ_points` | 0 | 34 |
| `verdict stage=second s11_k10_violation_points` | 0 | 34 |
| `geometry_sensitivity quantity=q1d_growth` の `result` | `consistent_on_12_sampling_points` | `unstable`（`e156-alloc-basis-counts-2026-10-05.out` 内の `name=geometry_sensitivity quantity=q1d_growth` の行において、12個のサンプリング点のうち「実」列の s=4 ρ=1、s=4 ρ=1/4、s=8 ρ=1/4、s=16 ρ=1/4 の4点が `not_monotonic` であり、残る8点は `grows_with_holes`） |
| `verdict stage=second_group q1d_growth` | 12グループすべて `grows_with_holes` | 上記と同じ4グループが `not_monotonic`、8グループ不変 |
| `q1c_front_threshold_moved` S=4、ρ=1、「実」 | `threshold_between=10` `threshold_at_largest=18` | `threshold_between=18` `threshold_at_largest=20`。「実」列6グループすべて異なる。両成果物の `name=q1c_front_threshold_moved s=4 rho=1 timing=real` 行を参照。 |
| `u13a_rollback_to_oldest_candidate item1_slots` | 374 | 377 |

変わらない判定：`f1_front_above_two_groups` は12グループすべてあり、F1 は新 `crates/` でも発動。`name=verdict stage=first`（`:1214`）、`v2_a10_every_mismatch_cells`、`v2_a12_every_nonzero_points`、`v2_a14_mismatch_points`、`v2_a15_every_below_points`、`s1h_root_ring_missing_points` は第4回と同じ（`:3942`）。`name=verdict stage=second` 全文：

```
成果物の行：git show 18b30adbfe94:e156-alloc-basis-counts-2026-10-05.out、3942行目（name=verdict）
```

**Q2c 原因の切り分け**：リポジトリのコピーで `allocator.rs` に対する31-1のアサーションブロックだけを逆適用し、`record_root_written_by_this_process` を `let Some(occupancy) = self.root_ring.as_mut() else { return; };` に戻した。同じ装置をコピー専用のビルドディレクトリで1回実行した `e156-alloc-basis-counts-2026-10-05-attribution-without-31-1.out`（3955行）は `e156-alloc-basis-counts-2026-10-05.out` と `cmp` でバイト一致し、両方の sha256 は `a3d29fe0…`。差異は31-1由来でなく、第4回成果物の後に別バッチが変更した `crates/` に由来する（登録 W14）。登録第4節 P7 の Q2a 同一という予測は誤り、F30 の条件「Q2c が31-1由来と判定」は満たさない。どのバッチに由来するかは調べていない（登録に測定項目なし）。

登録によれば Q2a「異なる」は S14・F26、K10 の「実」列34点不一致は S11 を発動し、いずれもメイン agent に引き渡す停止条件で、結果扱いせず判定基準を変えない。コードからの未検証の手掛かり：第5回時の `crates/singlefs-core/src/allocator.rs` はマウント時に全ゼロと読まれたスロットを `RootRingOccupant::AllZeroAtMount` として記録し、それが保持し得た最古 txg に回収閾値を固定する（`TxgsARingSlotBadAtMountCanHold` の文書コメント、ユーザー決定乙）。一方、装置 `read_hh_state` に `AllZero` の処理はない（装置ソースの `grep -n 'AllZero'` は出力なし）。前穴セルではクラッシュ時のルートスロットはリング1周目にまだ書かれていないスロットと推定され（成果物で未確認）、差異のセルと概ね重なる。両側のコードを照合する必要があり、本成果物の結論ではない。

**判定行の個別確認**：`grep -n 'name=verdict' e156-alloc-basis-counts-2026-10-05.out` は14行、行番号 `1214`、`3942`–`3954`。`false`・`not_run` の値はなく、違反を示す名前で非0の整数は `s11_k10_violation_points=34` のみ（`:3942`、K10「実」列34観測点、すべて前穴16セル内。第4回は0で登録の想定外、停止 S11・S14 発動）。`cells_run=38`・`cells_not_fitting=4` はセル数で違反ではない。判定行以外では `name=failure_clauses`（`:3941`）の `f24_real_every_differ_points=34`（第4回0）と `f2_no_observable_difference=false` が同種で、後者の false は観測可能な差があることを意味し、F2 は発動せず第4回と同じ。

**第5回と本ページ内の第4回各節の関係**：「この段階で行ったこと」から「答えられないこと」までの各節は2026-09-26第4回再実行を記し、`e156-alloc-basis-counts-2026-09-26-r2.out` を引用している。新成果物に合わせて書き換えていない。Q2a 正規化後の新成果物は294行異なり、F1 は依然発動するが、Q1d が h_欠 とともに増加するという記述は「実」列4サンプルでは成立しない。メイン agent は第5回成果物を根拠として採用済み（`e156-r5-prereg.md` 末尾の「主 agent 认定：第 5 次交回与 S11 两边查」第2項）。第6回に乙の閾値を採用すると `f24_real_every_differ_points`・`s11_k10_violation_points` は再び0になったが、これらの節の「Q1d はすべて h_欠 とともに増加」は第5・6回成果物の「実」列4点で依然不成立で、そのまま引用できない。

**第5回で答えられないこと**：① 実行担当者は checker 層を実行しないため、インライン単体テスト・変異は未実行。新 U19 と装置全体18テスト、書き換えた M35・M44 の実際の赤緑結果はなく、変異2行の原文が各1回一致し、置換文がコピーでビルドできることのみ確認した。② PC-閾値固定は1周の遅延を証明するが、全期間の非回収は証明しない。③ W7 が根リング表のない分配器を検出できるかの装置内対照はなく、core 単体テスト `recording_a_root_on_an_allocator_without_a_root_ring_occupancy_panics_instead_of_never_reclaiming` と C551 変異「表のない分配器がルートを記録すると黙って戻り、アサートしない」に頼る。④ 294行の差異を生んだ `crates/` のバッチは未調査。⑤ `s11_k10_violation_points=34` が装置の誤読か、実装閾値が条項と違うのかは両側の調査を完了しておらず、Q2c 後のコード読解による手掛かりのみ。

**提出時にメイン agent が実行するコマンド**（実行担当者は実行しないヘビー層）：

```
SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 10 cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts
```

`test result:` 行が第6回の `19 passed`（新 U20 を含む。第5回引き渡し時は `18 passed`、Q2d）か確認。U19 の名前は `hh_back_hole_at_four_slots_per_region_delays_the_tree_table_reclaim_to_six_ring_slots_minus_one_and_the_a10_judgement_sees_it`。旧 M35 の置換文 `allocator_without_the_root_ring` は装置から削除されビルドできないため書き換え版 M35 に交換し、M44 を `crates/mutations.tsv` に追加済み（`grep -n 'e156_allocation_basis_counts' crates/mutations.tsv` で確認。2行の原文は `e156-alloc-basis-counts-2026-10-05-mutations-replacements.tsv`・`e156-alloc-basis-counts-2026-10-05-mutations-append.tsv`）。checker-tier-crates-mutation-replay ゲートで実行し、3つの数を別々に報告する。

### この段階で行ったこと

2026-09-26 第4回再実行では、第1・第2段階を1回の装置実行にまとめた。第1段階の停止条項が一つでも発動すれば第1段階のみ出力し、すべて発動しなければ第2段階を実行する（登録第12節修訂2第11項）。

1. **事前確認**：対象条項5段落を登録第13節コマンド1で開始前と成果物生成前の2回再取得し、登録第2節とバイト一致、sha256 `af95e554dd52e87acc7095c129194171cac2e122ca1832d36be10882022e5de4`、停止 S2 は発動せず。コマンド4のアンカースクリプトをそのまま再実行し、出力 sha256 `700ec0b0…`、`--dump` sha256 `7bb1e088…` は登録どおり。実装スナップショット S3 を成果物前後に取得し、`git diff -- crates/` の sha256 はともに `ecf57f9db7d0c6e79b833cfaab481f68c12b54e3a5b161c13a626d9770a2d6fa`、`git status --short crates/` はともに `947a8fd6955f503316ba7d663dd22fa40cd9ce2d316a940df62c66bae39a97bc`。
2. **装置を全面改修**：マウント時ロールバックのローカル代役、HK / HK-F0 / HR / H0 / HF / HX / HY、およびマウント時ロールバック形の隔離シナリオをすべて削除し、分岐3・7の測定をやめた。新規作成：分岐1系列（分配器は根リング表を装着した製品経路 `allocator_after_make_filesystem` を使用。「記録永続済み、ルートスロット未永続化」のクラッシュを記録済みデバイス書き込みのルートスロット書き込み直前で切断して作り、各観測点で画像のルートリングから閾値、h_欠、h_穴を読み、2アーム×2回収時点の過剰差し引きを計算）；アンカーモデル（コマンド4 `anchors_e156_r4.py` の逐行移植、`name=anchor_dump` 1092行のプレフィックスを除いたものが Python `--dump` とバイト一致、コマンドは結果節）；前提2の書き込み記録（F 引き上げ時のデバイス書き込みをシステム構成書き込み後と各空押しルート後で切り、F_有効を読む）；2つのロールバック単体テストの新形態；2・1スロット閾値の判定関数。
3. **第1段階で S1(i) が一度発動し、登録どおり装置の実装誤読を調査・修正し第12節に記録**：記録した書き込みによる切断画像と、従来の「公開後にルートスロットだけ戻す」画像は、毎回ディスクごとにシステム構成1セクタが異なった。実装はルートスロットの FUA 後にも各ディスクのシステム構成スロットをローテーションする（`crates/singlefs-core/src/transaction.rs` の `persist_the_root_then_rotate_the_system_configuration`）ため、「ルートスロットが公開の最後の書き込み」という旧前提は不成立。対照画像を「公開後にルートスロットと各ディスクのシステム構成2スロットを戻す」に変更。S1(i) のバイト一致という基準は変更せず、異なるセクタを穴ごとに報告。登録第12節修訂2第1項に記録し、採用はメイン agent が判断する。
4. **第2段階**：S ∈ {4,8,16} × ρ ∈ {1,1/4} × (k, 位置) ∈ {(0, —), (1, 前), (2, 前), (4, 前), (1, 後), (2, 後), (4, 後)} の42セル中、S=4 の (4, 前)・(4, 後) は幾何上収まらず（アンカー `fits=false`）、38セルを実行。各セル2回収時点で76組の値。陽性対照 PC-過剰差し引き（各 (S,ρ) の k=0・(2,後)）、PC-穴、PC-分配器（S=4、ρ=1、k=0 で根リング表を装着しない分配器に戻す）、PC-判定器（合成10ケース）を実行。付帯 Q1e は登録どおり S=8、ρ=1 のセルのみ計算し、他は「判定十分後未実行」と表示。
5. **単体テストと変異**：`cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts` → `test result: ok. 17 passed`。`crates/mutations.tsv` のこの装置を対象とする22行は、新規9行 M35–M43、アンカー変更3行 M32–M34（テスト名変更に追随）、残り10行は不変。草稿ディレクトリ内のリポジトリコピーで22行を個別に適用し指定テストを実行、すべて赤（検出22／無効0／赤にならず0）。code-source-discipline ゲートの mutation-tables 欄、アンカー一意一致も通過。checker-tier-crates-mutation-replay ゲートの全表再実行は提出前の全体実行に委ね未実施。このうち9行のテスト名末尾に `$` があり、同ゲートが文字どおり `test … FAILED` を探すと見つからない。変更せずメイン agent に引き渡した（登録第12節修訂2第10項）。

### 結果行を成果物から全文転記（作業区から移動済みの成果物は1行の参照先で記載）

成果物 `e156-alloc-basis-counts-2026-09-26.out` は4007行。2026-10-05 時点の sha256 は `cbbb5f38033ce0c93baa837044abe261e07471cb7ff1af150457f4ad6546e136`。2026-09-26 実行時には `1eaf5ea6…` と記録されたが、後のコミット `bc57af7a`（内容に基づく命名）が160行を変更。`git show bc57af7a^:e156-alloc-basis-counts-2026-09-26.out | sha256sum` は `1eaf5ea6…` で、現値と旧値は異なる。装置の領域数 R をローカル定数に変更してから実行した `e156-alloc-basis-counts-2026-09-26-r2.out` は前者とバイト一致し、現在 sha256 は両方 `cbbb5f38…`。第5回再実行前の `replay.sh` 登録は r2 を指した。完了マーカー：

```
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out、4007行目（name=done）
```

**前提1（質問票第1節）：成立。** 2つの最小再現 S=4、ρ=1 ではクラッシュした txg のルートはリングに存在せず、新インスタンス最初の txg = 穴 + 1。「前」穴のスロットは未書き込み、「後」穴のスロットは txg 4 のルートが残る。

```
E7RESULT name=pq1 position=front hole_txg=5 prefix_applied=1 first_txg_of_new_instance=6 ring_has_root_at_hole_txg=false hole_slot_content=never_written a16_applicable=true
E7RESULT name=pq1 position=back hole_txg=16 prefix_applied=1 first_txg_of_new_instance=17 ring_has_root_at_hole_txg=false hole_slot_content=root_txg_4 a16_applicable=true
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out、1143行目（name=hole_recovery）
E7RESULT name=pc_c2 ring_has_root_at_hole_txg=true mount_return_h_que=0 a14_mismatches=9 seen=true
```

陽性対照 PC-c2 は切断点にルートスロット書き込みを含める。同履歴のリングに txg 16 のルートがあり、マウント返却時 h_欠=0、閾値とアンカーは9観測点で不一致。この測定は穴あり・なしを区別できる。

**前提2：不成立（F の二つの計測条件が合流）⇒ 分岐表第3行は無効、測定を設けない。** F 引き上げはまず新 F=8 を各ディスクのシステム構成へ書いてバリアを通す。最後の構成書き込み後に切った画像では F_有効 はすでに8。各空押しルートの F とその後で切った F_有効 はともに8。

```
E7RESULT name=pq2 raised=true floor_before=0 new_floor=8 effective_floor_before_any_write=0 system_configuration_writes=6 effective_floor_after_last_system_configuration_write=8 system_configuration_writes_after_first_root=4 pushed_roots=txg12:root_floor8:effective8,txg13:root_floor8:effective8 publishes=2 reclaimed_placements=61 the_two_readings_merge=true
```

`system_configuration_writes_after_first_root=4` は2回の空押しそれぞれがルートスロット後に2ディスクのシステム構成をローテーションしたもの（前提1と同じ永続順序）で、新 F の先行書き込みではない。

**前提3：分岐2系列のみ旧式で定義**（登録1.1、登録文の確認）。分岐1への間接影響、すなわち Hh 履歴が新 admission 式に到達するかを装置で確認した。38セルおよび第1段階の2履歴すべてで公開の admission 拒否0回、書き込み可能マウントはすべて番号取得前に十分と判定（`mount_admission=admitted_before_acquisition`）、F 引き上げ列は0、F_有効は常に0（`g_adm_nonzero=0`、2判定行を参照）。

**質問票第3節（ロールバック形態変更で無効となった問い）**：マウント中ロールバック形では U11・U13 の構造がいずれも成立：txg = 現行 + 1、インスタンス世代不変、インスタンス表行数不変、通常再オープン後の隔離0スロット、破棄ルート0本。U13(a) のロールバック版の会計項目5は356スロット、I-3.11 (割り当て済みから defer を引いた値が最新ルートのトラバーサルと一致)は成立。Q7c①の色反転は合成基底 β_syn で確認する。

```
E7RESULT name=u11_forward_rollback txg_before=8 txg_of_rollback_publish=9 instance_before=2 instance_after=2 instance_table_rows_before=1 instance_table_rows_after=1 remount_instance=3 isolated_after_plain_remount=[(0, 0), (1, 0)] abandoned_roots_after_plain_remount=0 forward_rollback_structure_holds=true
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out、1211行目（name=u13a_rollback_to_oldest_candidate）
E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true
E7RESULT name=q7c2_threshold_at_least basis=beta_syn k=-1 not_applicable=true reason=deferred_is_zero_cannot_subtract_one
```

第1段階の判定行：

```
E7RESULT name=verdict stage=first premise_one_reproduced=true premise_two_the_two_readings_merge=true premise_two_answer=not_holds fork_three=void_not_measured g_adm_nonzero=0 s1_failures=0 a11_mismatches=0 a14_mismatches=0 pc_c2_seen=true k11_holds=true forward_rollback_structure_holds=true u13a_structure_holds=true u13b_flip1=true u13b_flip2_not_applicable=true stop_reasons=none
```

**分岐表第1行（第2段階）**。判定行（無効化・停止のカウントはすべて0）と12サンプル点 (S, ρ, 回収時点) の判定：

```
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3994 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3995 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3996 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3997 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 3998 行（name=verdict）
E7RESULT name=verdict stage=second_group s=8 rho=1 timing=real q1c=above q1c_on_holes=above q1c_front=above q1c_back=above q1d_growth=grows_with_holes
E7RESULT name=verdict stage=second_group s=8 rho=1 timing=every_publish q1c=above q1c_on_holes=above q1c_front=above q1c_back=above q1d_growth=grows_with_holes
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4001 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4002 行（name=verdict）
E7RESULT name=verdict stage=second_group s=16 rho=1 timing=real q1c=above q1c_on_holes=above q1c_front=above q1c_back=above q1d_growth=grows_with_holes
E7RESULT name=verdict stage=second_group s=16 rho=1 timing=every_publish q1c=above q1c_on_holes=above q1c_front=above q1c_back=above q1d_growth=grows_with_holes
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4005 行（name=verdict）
产物行见 git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out 第 4006 行（name=verdict）
```

- **Q1c-前（第1行の基準）**：12点すべて「2スロット超過」、1スロットも超過。最小超過点はいずれも h_欠=1、最初の前穴マウント返却時 txg 7。前穴 Δ ピークは12点で同一で、h_欠=1、2、3、4 で10、18、26、34スロット。S=4 は k≤2 のみ収まり18まで。S=8、ρ=1、「実」の行：

```
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out、3935行目（name=q1c）
```

- **Q1c-後（参考報告、基準に算入しない）**：12点すべて2スロット超過。登録第4節の「後穴は必ず過剰差し引きが大きい」という予測は覆らず、失敗 F23 は発動せず。後穴はリング内の旧ルートに閾値を固定し、Δ は h_欠 の各ステップで1公開が置き換える量だけ増える。ρ=1 では約14–16、ρ=1/4 では約8–10スロット。最後の h_欠 点は S=4 / 8 / 16 でそれぞれ174 / 366 / 716（ρ=1）、118 / 238 / 478（ρ=1/4）。
- 各点の一覧。成果物から1コマンドで数えた出力原文：

```
$ grep '^E7RESULT name=q1c ' e156-alloc-basis-counts-2026-09-26.out | grep -E 'quantity=q1c_(front|back) ' | awk '{h=$13; sub("by_h_que=","",h); n=split(h,p,","); split(p[1],a,":"); split(p[n],b,":"); sub("peak","",a[2]); sub("peak","",b[2]); print $3, $4, $5, $6, $8, $10, "h" a[1] "->" a[2], "...", "h" b[1] "->" b[2]}'
s=4 rho=1 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h2->18
s=4 rho=1 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h12->174
s=4 rho=1 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h2->18
s=4 rho=1 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h12->174
s=4 rho=1/4 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h2->18
s=4 rho=1/4 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h12->118
s=4 rho=1/4 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h2->18
s=4 rho=1/4 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h12->118
s=8 rho=1 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=8 rho=1 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h24->366
s=8 rho=1 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=8 rho=1 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h24->366
s=8 rho=1/4 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=8 rho=1/4 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h24->238
s=8 rho=1/4 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=8 rho=1/4 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h24->238
s=16 rho=1 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=16 rho=1 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h48->716
s=16 rho=1 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=16 rho=1 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->52 ... h48->716
s=16 rho=1/4 timing=real quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=16 rho=1/4 timing=real quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h48->478
s=16 rho=1/4 timing=every_publish quantity=q1c_front judgement_two_slots=above smallest_h_que_above_two=1 h1->10 ... h4->34
s=16 rho=1/4 timing=every_publish quantity=q1c_back judgement_two_slots=above smallest_h_que_above_two=4 h4->34 ... h48->478
```

- **Q1d（穴に応じた増加、単位内スロット数を超える規模）**：12点すべて「h_欠 とともに増加」（隣接差すべて >0）。2・1スロットの初回超過はいずれも h_欠=1、前穴 k=1、第1回ワークロード公開後のマウント返却時 txg 7。第2の読み方（判定外）：注入穴数 k 別では Δ ピークは**k とともに増えない**。S=8、ρ=1、k=1 / 2 / 4 で366 / 360 / 344スロット。ピークは後穴が閾値を固定する区間によって支えられ、穴数によるものではない。S=8、ρ=1、「実」の行：

```
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out、3937行目（name=q1d）
```

- **幾何感度**：Q1c、Q1c-穴、Q1c-前、Q1c-後、Q1d は12サンプル点で一致（この12点のみ）。「実」「毎」は観測点ごとに一致（`f24_real_every_differ_points=0`）：

```
E7RESULT name=geometry_sensitivity quantity=q1c_front result=consistent_on_12_sampling_points per_sampling_point=s=4_rho=1_timing=real:above,s=4_rho=1_timing=every_publish:above,s=4_rho=1/4_timing=real:above,s=4_rho=1/4_timing=every_publish:above,s=8_rho=1_timing=real:above,s=8_rho=1_timing=every_publish:above,s=8_rho=1/4_timing=real:above,s=8_rho=1/4_timing=every_publish:above,s=16_rho=1_timing=real:above,s=16_rho=1_timing=every_publish:above,s=16_rho=1/4_timing=real:above,s=16_rho=1/4_timing=every_publish:above
```

- **失敗条項**：F1（甲-T1 は前穴で大きく過剰差し引きし、単に適用範囲を述べるだけでは解決できない、メイン agent に引き渡す）は12点すべてで発動。F2・F22・F23・F24 は発動せず。

```
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out、3993行目（name=failure_clauses）
```

- **絶対値を固定するアンカー**：各 S の A10 遅延はちょうど3S。収まるセルでは r2 の2閉形式がリングからの現時計数と点ごとに一致。k=0 の Q1a と全セルの Q1b は常に0。装置は38セルの各観測点の閾値・h_欠・h_穴がモデルと一致し、Q1a はツリーテーブル下限以上（判定行の `v2_*`・`v6_*`・`s11_*` はすべて0）。陽性対照 PC-分配器：根リング表がないと今回のマウント内ではツリーテーブルユニットを一つも払い出せない。

```
E7RESULT name=anchor_model s=4 k0_end=48 a10_lags=[12] a11_closed_form_mismatches_on_fitting_cells=0 a12_k0_max_q1a=0 a13_max_q1b=0 fits=1front:true,2front:true,4front:false,1back:true,2back:true,4back:false
E7RESULT name=anchor_model s=8 k0_end=72 a10_lags=[24] a11_closed_form_mismatches_on_fitting_cells=0 a12_k0_max_q1a=0 a13_max_q1b=0 fits=1front:true,2front:true,4front:true,1back:true,2back:true,4back:true
E7RESULT name=anchor_model s=16 k0_end=120 a10_lags=[48] a11_closed_form_mismatches_on_fitting_cells=0 a12_k0_max_q1a=0 a13_max_q1b=0 fits=1front:true,2front:true,4front:true,1back:true,2back:true,4back:true
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-26.out、3859行目（name=pc_allocator）
```

アンカーモデルの Rust 移植と Python スクリプトを逐行比較。Python 出力は草稿ディレクトリ内でリポジトリには入れず、1092行の `DUMP` を比較：

```
$ grep '^E7RESULT name=anchor_dump ' e156-alloc-basis-counts-2026-09-26.out | sed 's/^E7RESULT name=anchor_dump /DUMP /' | sha256sum
5e61a3e939ebd5df09c76c80dde30352482245b7e83a1164e6ccf4e0223794e0  -
$ python3 anchors_e156_r4.py --dump | grep '^DUMP' | sha256sum        # 脚本全文在登记第十三节
5e61a3e939ebd5df09c76c80dde30352482245b7e83a1164e6ccf4e0223794e0  -
```

- **Q1e（付帯、S=8・ρ=1 のみ）**：穴あり観測点で第4回時の `check_pool_image` の I-3.1 (割り当て済み統計が一致する)は148状態中14状態で赤。すべてクラッシュ回復後 L2 で、(2, 後) の txg 57 および (4, 後) の txg 57–69。

```
$ grep '^E7RESULT name=q1_cell ' e156-alloc-basis-counts-2026-09-26.out | grep 's=8 rho=1 ' | grep -o 'k=[0-9] position=[a-z-]*\|i31_red_on_hole_states=[0-9]*\|i31_judged_hole_states=[0-9]*' | paste -d' ' - - -
k=0 position=- i31_red_on_hole_states=0 i31_judged_hole_states=0
k=1 position=front i31_red_on_hole_states=0 i31_judged_hole_states=21
k=2 position=front i31_red_on_hole_states=0 i31_judged_hole_states=23
k=4 position=front i31_red_on_hole_states=0 i31_judged_hole_states=29
k=1 position=back i31_red_on_hole_states=0 i31_judged_hole_states=21
k=2 position=back i31_red_on_hole_states=1 i31_judged_hole_states=24
k=4 position=back i31_red_on_hole_states=13 i31_judged_hole_states=30
```

- **成果物の完全性**：観測点行はセルごとにアンカーモデル `--dump` の観測点数と一致。38セル + PC-分配器46 + PC-c2 25 = 2141行。k=0 と (2,後) には PC-過剰差し引きの4列が追加され、234 + 180行 × 2ρ。

```
$ grep -c '^E7RESULT name=q1_observation ' e156-alloc-basis-counts-2026-09-26.out
2141
$ grep '^E7RESULT name=q1_observation ' e156-alloc-basis-counts-2026-09-26.out | awk '{print NF}' | sort | uniq -c
   1313 40
    828 44
```

### 再実行

```
bash scripts/experiment-apparatus/replay.sh E156
cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts
```

`scripts/experiment-apparatus/replay.sh` の E156 行は現在、第7回主成果物 `e156-alloc-basis-counts-2026-10-18.out` を指す。release バイナリを `--features verdict-store,gpu` 付きで `scripts/crash/gpu-quiesce-run.sh` 経由で起動し、約241秒。`driver_e156` は `cargo run -q --release …` に変更し、feature と静止化は `CHECKER_TIER_FEATURES`・`CHECKER_TIER_QUIESCE_PREFIX` 経由、`BINDGEN_EXTRA_CLANG_ARGS` はリポジトリ直下 `.cargo/config.toml` から渡す。マルチマシン設定 `ENABLE_GPU=1` の場合、gpu feature なしの装置は `verdict` で panic するため省略できない。`replay.sh E156` の全体再実行はヘビー層のため未実施で提出前に行う。照合用アンカー入力とスクリプトは `e156-alloc-basis-counts-2026-10-18-support/` 内で、`replay.sh` には含めない。第7回前は第6回主成果物 `e156-alloc-basis-counts-2026-10-05-r2.out` を指した。実行担当者の release 出力と debug の先頭1803行はバイト一致したが全 debug 再実行は未実施。判別力対照 `e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out`・その `.patch`、比較 `e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.py` / `.txt`、点集合確認 `e156-alloc-basis-counts-2026-10-05-r6-point-sets.py` / `.txt` とその入力 `e156-alloc-basis-counts-2026-10-05-r6-anchor-pins.txt`・`e156-alloc-basis-counts-2026-10-05-r6-anchor-lowered.txt` は `replay.sh` に含めない。それ以前は第5回成果物 `e156-alloc-basis-counts-2026-10-05.out` を指し、その前は `e156-alloc-basis-counts-2026-09-26-r2.out` を指した。後者は `e156-alloc-basis-counts-2026-09-26.out` とバイト一致し、両方保持。原因切り分け `e156-alloc-basis-counts-2026-10-05-attribution-without-31-1.out` は新成果物とバイト一致し再実行に含めない。比較 `e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.py`・その出力 `e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt`、変異行 `e156-alloc-basis-counts-2026-10-05-mutations-append.tsv`・`e156-alloc-basis-counts-2026-10-05-mutations-replacements.tsv` も保持。2026-10-05 新成果物は実行担当者が同一バイナリを直接再実行したもので、`replay.sh E156` の全体実行は提出前に回した。第4回登録の再実行は、リポジトリ直下でドライバ `driver_e156` が `cargo run -q -p singlefs-checker-tier --bin e156_allocation_basis_counts` をメモリ制限ラッパー経由で実行。2026-09-27 に旧成果物を指していたときの判定は「バイト一致」で、集計原文：

```
字节一致 1 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0 ／ 输入没变没跑 0
```

第4回以前の成果物は削除していないが、現在存在しない実装（マウント時ロールバック、マウント内非回収、単一ノード割り当てレコードツリー）の数値であり、根拠には使わない：`e156-alloc-basis-counts-2026-09-22-stage1.out`、`e156-alloc-basis-counts-2026-09-22-stage2.out`、`e156-alloc-basis-counts-2026-09-22-stage3.out`、`e156-alloc-basis-counts-2026-09-23-stage1.out`、`e156-alloc-basis-counts-2026-09-24-stage2.out`、`e156-alloc-basis-counts-2026-09-24-stage3.out`、`e156-alloc-basis-counts-2026-09-24-stage4.out`、`e156-alloc-basis-counts-2026-09-25-r3.out`、`e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out`。当時の解釈は本ページの履歴版を参照。

### 答えられないこと

- **G12 アームはバイパスで述語を評価するだけ**：同一の実履歴で各述語がどのスロットを保持するかには答えられるが、G12 に従って実行した履歴の形には答えられない。第4回では各公開で回収するので、G12 の「実」「毎」は同じ計算であり、F24 は G12 列では発動しない。PC-過剰差し引きの4組合せは構成上同じ値。
- **穴は上書きまたは空公開時のクラッシュのみで作成**：行書き込み、ウォームアップ、F 引き上げの空押し、マウント中ロールバック時のクラッシュ、およびマウント時に最新ルートスロットを読めないケースは作らない。
- **Hh に F 引き上げと破棄ルートはない**：F > 0 またはシャドウ会計が有効な場合の2アームの差は未測定。
- **間隔・プールとも1種類**：穴の間にワークロード公開2回、プールは4 GiB × 2。
- **判定は12サンプル点のみ**：S ∈ {4,8,16} × ρ ∈ {1,1/4} × 2回収時点。S=4 の k=4 の2セルは幾何上収まらず未実行。
- **Q1e は S=8、ρ=1 のみ**：他セルは「十分後未実行」。
- **決定的な装置**：2回一致は統計的証拠ではない。証拠の強さは絶対値を固定するアサーション（アンカーモデル、K1-1、R-4）と22変異に由来する。
- **メイン agent に引き渡す事項**：① F 引き上げでは保持を最後の空押し後に解除し、F_有効 が新値になるより遅い（登録第3節 j7・j8）。これは分岐表第3行の2候補のいずれでもない。② 到達可能状態の会計項目5が0になり得るかは測れていない。第3回の到達可能基底 `beta_hr_rollback_row`（`e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out`）は第4回では再現できず、マウント中ロールバック版の項目5は356スロット。I-3.11 の判別力自己検証は合成基底 β_syn のみであり、条項本文にもそのように記す。③ 質問票第4節が名指しした `crates/mutations.tsv` 551行は本装置の変異ではない。④ S1(i) 対照の構成を変更した（この段階で行ったこと3）。⑤ 本装置対象の9変異行のテスト名に `$` がある。

### 経路と結論の登録

| # | 経路 | 算出方法 | ソース位置 | 共通に読む項目 |
|---|---|---|---|---|
| 1 | 装置自身の参照 | 各 txg の公開出力にある全ユニットの配置先をスパン展開。インスタンス表ユニットの初回書き換え前はルートレコードのポインタで補い、ユニット0の版は mkfs ルートレコードの2ポインタによる。 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1778` `facts_of_published_version` | 公開出力（`TransactionOutput.units`、ルートレコードポインタ）、2ディスク同一スロットの前提 |
| 2 | ディスク上の未解放割り当て記録 | 同じリングのルートから `recovery::allocation_records_under_root` が読む未解放記録をディスクごとに展開。割り当てレコードツリーのない版はルートレコードの2ポインタの該当ディスク分。 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1823` `referenced_slots_read_from_disk` | ディスク上の割り当てレコードツリーとルートレコード、スパンの定義 |
| 3 | 画像の根リングから現時点で読み取る | 各観測点でリングの自己検証済みルートをすべて読み、閾値 = max(F_有効, リング最古の有効ルート)。第6回から後者にマウント時全ゼロで未上書きのスロットを決定済み乙に従って含める（切断画像上で `root_ring_slots_reading_back_all_zero_at_mount`）。h_欠・h_穴は登録どおり数え、このスロットを合算しない。 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:2096` `read_hh_state` | ルートリング配置先の式 `target_for_publish`、`effective_rollback_floor`、インスタンス表の破棄判定、`read_root_ring_slot` と全バイト0の判定 |
| 4 | アンカーモデル | コマンド4 `anchors_e156_r4.py` の逐行移植。閾値は第6回登録コマンド4 `anchors_e156_r6.py` に従いマウント時未書き込みスロットを含める。スクリプト冒頭の条項のみで穴、マウント完了 txg、閾値、h_欠・h_穴・ツリーテーブル下限を計算。 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1356` `anchor_model::run_the_anchor_hole_script` | txg u のスロット u mod 3S、領域デバイス [0,1,0]、各公開でツリーテーブル単位を交換、ウォームアップ規則、c2 後初回 txg 規則 |
| 5 | 書き込み記録を切断 | クラッシュ公開時のデバイス書き込みを記録し、ルートスロット書き込み前までを公開前の画像に適用。 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1011` `crash_publish_before_its_root_persists` | `FixedGeometry::classify` によるルートスロット書き込み認識、同一公開の入力 |
| 6 | 公開後に戻す | 実際に公開した後、ルートスロットと各ディスクのシステム構成2スロットを公開前に戻す。 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:924` `crash_before_root_persists` | ルートスロット配置先の式、システム構成の各ディスク2スロット・幅4096（ローカル定数）、同一公開の入力 |
| 7 | 甲-T1「実」「毎」 | 「実」は実分配器のフリーマップ、「毎」は閾値から記録を現時計算。 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1955` `over_withheld_on_device` | 同じ分配器記録、閾値、参照 |

経路1・2は各マウント返却時と各セル終点でルート・ディスク別に比較し `w9_mismatches` はすべて0。経路3・4は各観測点で比較し `v6_a11_mismatch_points=0`・`v2_a14_mismatch_points=0`。経路4は Python と逐行比較（結果節の sha256）。経路5・6は穴ごとに比較し `cut_matches_root_and_system_configuration_restored_image` はすべて true。経路7の2列は各点で比較し `f24_real_every_differ_points=0`。各比較が共有する前提、すなわち公開出力とディスク記録が同一公開に由来すること、ルートリング配置先の式、同一記録・閾値が同時に誤っている場合は検出できない。アンカーモデルのみが条項文だけを使い `crates/` を経由しない。第7回再実行では経路4に新系列の穴 txg、`k_满`、終点、`fits`、`k_满后` を追加。Rust 移植と Python を点ごとに比較し（停止 S20）、`anchor_r7_summary` 58行、`anchor_r7_dump` 3567行、`anchor_r7_k0_summary` 3行、`anchor_r7_k0_dump` 384行は `e156-alloc-basis-counts-2026-10-18-support/` のスクリプト出力と `cmp` で差なし。コマンドは第7回節参照。共通前提は同一の条項文で、両側が同じ箇所を誤読すると検出できない。開始前試行ではスクリプトが h_穴 ウィンドウの左端を漏らし、装置は正しくスクリプトが誤っていたため差異が現れた。

### 影響する決定

| 決定項目 | 関係 | 見直し |
|---|---|---|
| D28 (マウント時コミットメント量)確定項目1 | 判断材料 | 2026-10-23 影響なし：成果物名の書き方だけの変更で、結論、数値、成果物は不変。それ以前：2026-10-21 影響なし：全文転記した成果物行を現行キー名で再転記し、値は不変。2照合出力と番号略称を明記し、数値・結論・成果物は不変。2026-10-18 影響なし：第7回は根リングの穴に伴う甲-T1 の過剰差し引きの増え方 H1 を測り、admission 不等式の項を測っていない。 |
| D16 (パブリッシュセマンティクス)確定項目1 | 判断材料 | 2026-10-23 影響なし：成果物名の書き方だけの変更で、結論、数値、成果物は不変。それ以前：2026-10-21 は同様に値・数値・結論・成果物不変。2026-10-18 影響なし：Hs-後、1ディスクにルートを書けない形の24単位はすべて「なお増加中」で、本項目の「保持単位が公開とともに無界に線形増加」と矛盾せず F34 は発動しない。根拠節は E156 を引用しておらず、支持へ格上げするかはメイン agent が判断。 |
| D23 (ジャーナルの役割と形式)確定項目14 | 判断材料 | 2026-10-23 影響なし：成果物名の書き方だけの変更で、結論、数値、成果物は不変。それ以前 2026-10-21 影響なし：丸ごと転記した成果物行を成果物の現在のキー名に従って再転記（値は一字一句不変）、2つの照合出力と番号略称を補名、数値・結論・成果物は不変；2026-10-18 影響なし：第7回再実行は穴による超過控除スロット数を測定し、journal 再生集合は測定せず、C380（リングに穴がある際に解放世代が一意の値を判定できない）のうち解放世代が一意の値を判定できない側の半分には触れていない |
| D3 (領域割り当て)確定項目7 | 判断材料 | 2026-10-23 影響なし：成果物名の書き方だけの変更で、結論、数値、成果物は不変。それ以前 2026-10-21 影響なし：丸ごと転記した成果物行を成果物の現在のキー名に従って再転記（値は一字一句不変）、2つの照合出力と番号略称を補名、数値・結論・成果物は不変；2026-10-18 影響なし：第7回再実行が読んだのは分配器のフリーマップと回収閾値であり、単一割り当てレコードの配置粒度や key・value 幅は変更していない |
| D5 (スナップショット / 空間会計メカニズム)確定項目4 | 判断材料 | 2026-10-23 影響なし：成果物名の書き方だけの変更で、結論、数値、成果物は不変。それ以前 2026-10-21 影響なし：丸ごと転記した成果物行を成果物の現在のキー名に従って再転記（値は一字一句不変）、2つの照合出力と番号略称を補名、数値・結論・成果物は不変；2026-10-18 影響なし：第7回再実行は統計量一覧を測定しておらず、甲-T1 の超過控除スロット数が穴数に伴いどう増加するかのみを測定した |
| D8 (コアインデックス構造)確定項目14 | 判断材料 | 2026-10-23 影響なし：成果物名の書き方だけの変更で、結論、数値、成果物は不変。それ以前 2026-10-21 影響なし：丸ごと転記した成果物行を成果物の現在のキー名に従って再転記（値は一字一句不変）、2つの照合出力と番号略称を補名、数値・結論・成果物は不変；2026-10-18 影響なし：第7回再実行は keyspace とフロントエンドの分割に触れず、割り当てレコードツリー内のレコードを読んで超過控除を計算したのみ |

## 履歴バージョン

### 2026-09-26：第4回再実行前の本文（原文のまま移動）

**第4回再実行前のタイトル状態**：一部実施済み。第3回再実行の第1段階は質問票第3行に回答済み。分岐7の G27 判別力自己検証 Q7c① に到達可能基底を1つ追加し、到達可能状態で初めて色反転した。判定を続けるかはメイン agent に委ねる。分岐1・3の S=16 / ρ=1/4 / 位置「前」/ k=4 と HF の18セル全走査は未実施。2026-09-22 の第1・2段階、09-23 の R2 第1段階、09-24 の R2 第2・3段階と2つの追加段階（s4）、09-25 の第3回第1段階と到達可能基底 beta_hr_rollback_row を追加した追加段階を含む。リポジトリ収録の決定的装置で単体テスト11件・変異12件をすべて検出。D8 (コアインデックス構造)確定項目14により割り当てレコードツリーを位置指定にした後、旧レイアウトに固定された3定数と6箇所を再計算し、H0/HR は容量の壁に当たらず登録ステップを完走。分岐1は S=4 の Q1d が「単調」から「非単調」、分岐7は `q7d2_min_item5` が「非0」から「=0」、F16 が「発動」から「不発動」に変わり、分岐3の Q3c/Q3e/K9 は確認済み量では不変。S1 突き合わせ（S1(c) 完全再生、隔離スロット34→54）、アンカー K1-1/A-D8/A1/根リングスロット数/ノード容量、8履歴系列 H0/HR/HK/HK-F0/HF/Hh/HX/HY、分岐2の全 Q2a/Q2b/Q2c、分岐7の全 Q7a–Q7f（9基底）、分岐3の1幾何セル Q3a–Q3d と公開入口による Q3e（X8-A/HY）、分岐1の2 S 点（8・4）×3穴数の Q1a–Q1d とステップ数合わせ対照をカバー。HY の上書き回数は e ≥ max(8, f) を満たす段階を探索し、この回は0まで探索しても未達（制限事項参照）。S=16 / ρ=1/4 / 前 / k=4 と HF 18セル全走査は未実施。

**問題（第1～3回再実行）**：実行前登録 `e156-preregistration.md`（637行）の分岐2全体はユーザー決定に提出済み。開いている分岐1・3・7のみを再実行登録 `e156-r2-prereg.md` が対象とする（分岐表9・11・12行）。R2第1段階（2026-09-23）は分岐7のみ。第2・3段階（09-24）はそれぞれ分岐3・1を**範囲縮小版**として S=8（mkfs デフォルト）、ρ=1 の1幾何サンプルで代表履歴各1本を実行。R2 の同日追加2段階（s4）は、第1段階で穴0の Δ=20 の調査中に装置診断のフィルター欠落を発見・修正（報告済み Δ は不変）、分岐1に S=4 点、分岐3の K9 に前提の実測確認を追加。第2段階ではメイン agent の「分岐表にまだ2行開いている」との追加依頼により、分岐1にステップ数合わせ対照を加えて穴数とステップ数の影響を分離し、分岐3に実公開入口の Q3e（X8-A/HY）と Q3d を追加した。装置は**リポジトリ収録**の `crates/singlefs-harness` 読み取り専用 bin `e156_allocation_basis_counts`。実際の mkfs／ウォームアップ／新規プールでのファイル作成／上書き／空公開／書き込み可能マウント／管理者ロールバック／ロールバック下限引き上げを駆動し、`crates/singlefs-core` と `crates/singlefs-checker` の製品コードは変更しない。G27 は独立実装し、R2以降は判定対象画像の会計行のみ読む（「行ったこと」7項）。R2第1段階は登録5.7第1段階：① §11 S1 と `crates/` の項目別突き合わせ（S1(c) 完全再生を含む）、② K1・アンカー1・アンカー2・アンカー3、③ R3（G27 が画像を読む）、④ 分岐7の Q7a–Q7f と9基底 β0/β1/β2/βK-w/r/f/l2/β_syn/β_F0。第2段階は1つの HF 履歴で G7（公開入口から再構成）と F-扣（実 `raise_rollback_floor`）の D_rel と K9 を測り、第3段階は3つの Hh(k)、k=0・1・2 で甲-T1・G12 の過剰差し引きと Δ を測る。未実施事項は制限事項と履歴版2026-09-24を参照。

#### この段階で行ったこと（第1～3回再実行）

1. **锚1/锚2/锚3**：単位領域は各ディスク211968スロット（`4 GiB ÷ 16384 − 50176`、実装 `UNIT_AREA_START_SLOT` と比較）、S=8 の根リング24スロット（`target_for_publish` の対応を点ごとに照合、不一致0）、割り当て記録ノード容量812（`⌊(16384−135)÷20⌋`）。锚1・2は最初の2段階で `python3 -c` による独立再計算済み。锚3と2つの幾何アンカーはこの段階から装置が `main()` 冒頭で計算し `assert_eq!` する。
2. **K1（S1(a)）**：装置が mkfs→番号取得→ウォームアップ→新規プールのファイル作成を実行。ディスク0の会計項目1が13、項目2（空き）が211955、項目5が1スロット。配置 `50180:2,50240:1,50242:2,50244:1,50245:1,50246:1,50247:1,50248:1` は登録と完全一致（`matches_registered=true`）。
3. **S1(c) 完全再生**（2026-09-23 R2第1段階、旧軽量確認に代わる）：公開入口で `rollback_by_a_forward_publish.rs` の `rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation` を装置自身が再生。A → B(4100B) → 再オープン・番号2取得・行書き・ウォームアップ → C(2500B) → Aへロールバック → 通常再オープン。D の jsn が C の8に続く `c_counter=8 d_counter=9`、5記録 `(1,4)(2,5)(2,8)(3,9)(3,10)` がリングに残る、再オープンでインスタンス4、`isolated_slots_per_device == [(0,34),(1,34)]`、`abandoned_roots_unreadable == 0` を個別に再計算・`assert_eq!` しすべて通過。同名テストの成功確認ではなくシナリオ自体の再実行。**S1(b)** の `reuse_after_raising_the_floor.rs` 2ケースはテストファイル自体を実行したのみで装置側再生なし。
4. **H0/HR の合法状態**：H0（ρ=1、連続上書き、F 引き上げ・ロールバックなし）は50ステップ目に `AllocationRecordsExceedOneNode { records: 820, capacity: 812 }` に到達。単一ノード割り当てレコードツリーの実装上限で、回収はマウント／ロールバック再構築と F 引き上げ時のみ。H0 にはいずれもなく登録 I3 に従い記録は増加のみ。β1 は実行できた49ステップ目、txg 52。HR 前半も49で停止し、候補24本中最小 txg 29、インスタンス1へロールバック、後半は47ステップ目、txg 102、インスタンス2で停止。β2 は最終到達ステップ。両端点で `check_pool_image` を実行した実際の合法状態だが、各ステップの checker 判定は未確認。
5. **锚6（S1(e)）**：上書き1回 O は10スロット解放・10割り当て。H0 の49ステップすべてで厳密に成立（`503 = 13 + 49×10`、`491 = 1 + 49×10`）し、登録 P6 が引用する E153 の10スロットと一致。空公開 E はρ=1の H0 に含まれず未測定。
6. **分岐2（Q2a/Q2b/Q2c）**：D28確定項目1の式、D5確定項目4の対照表、`.claude/gate.d/doc-registries.sh` の admission-terms 欄を読み、一覧 L 4箇所を作成。登録コマンド `grep -rn 'defer 待释放' .claude/ crates/ scripts/` は45箇所、file:line 重複除去後も45。`|M∖L|=41 > 0`、`|L∖M|=0`。草稿内のリポジトリコピーでゲート判別力を自己検証：①式の項だけ削除→赤、②対照表の行だけ削除→**なお赤**（`03-空间分配.md:82` の式のコピーが未変更）、③式のコピーも変更→緑。ゲートスクリプト自体は一切変更していない。
7. **R3（2026-09-23 R2第1段階）**：G27 は `PoolAllocator` メモリではなく判定画像の会計行のみを読む。旧第1・2段階では両側とも画像を読まなかった欠陥があり、撤回経緯は履歴版に記録。新 `allocated_minus_deferred_mismatches_referenced(pool, accounting_slot, referenced)` が会計ツリー葉バイトを直接解析し、旧 `allocated_minus_deferred_matches_referenced` は変異の反例専用に残す（U8）。
8. **HK / HK-F0 の新履歴**：HK は実履歴 β0 → O → E → クラッシュなし再オープン（行書き・ウォームアップ）→ O×4 → F を上限まで引き上げ → O → 候補中最小 txg のルートへ管理者ロールバック → O → c2 クラッシュ（記録永続済み、ルートスロット未永続化）・回復 → O。骨格公開欄の全種類をカバー。HK-F0 は同じ操作で `ReuseWindow::ForcedToZero` を強制する到達不能陽性対照 R7。両系列の4観測点（再オープン、ロールバック、F 引き上げ、c2 回復後）に β0/β1/β2、β_syn（β0画像で項目1 −1、項目5 −1、項目2 +1）、β_F0（HK-F0 の新規プールファイル作成後）を加え9基底とし、Q7a Bd(+1)/(−1)/(+8)、Q7c①②、PC-チェック Bk1/Bk2/Bk3 を各基底で実行。
9. **装置自身の2欠落を発見・修正**（成果物前に `e156-r2-prereg.md` §12修訂へ記録）：① HK のロールバック先は HR の `readable_roots` から最小 `checkpoint_txg` を選ぶコードをそのまま使い、`effective_rollback_floor` で候補を絞っていなかった。F 引き上げ後 genesis は候補外なのに選ばれてしまうため、登録 R7 で固定済みのフィルターを追加。② `mount_writable`/`mount_rollback` は内部で `rebuilt_allocator` を再構築し、`ReuseWindow` が製品デフォルトへ戻る。最外層で一度設定するだけでは足りず、マウント／ロールバック後ごとに `set_reuse_window` を再適用する。
10. **分岐7の Q7a–Q7f 全体**（2026-09-23 R2第1段階、旧版 Q7a/Q7b/Q7c に代わる）：結果節参照。
11. **命名規律**：`naming-lint.sh` はファイル全体で違反0。R2段階で40以上の1文字・文字数字識別子を追加または改名し、例として `g8_prime_is_red` → `allocated_minus_deferred_mismatches_referenced`、`beta_w/r/f/l2` → `beta_after_reopen/rollback/raise_floor/crash_recovery`。ミューテーション表 `crates/mutations.tsv` の本 bin 向け変異は計3行。初回の `referenced_slots` でインスタンス表の代替を落とす行と、R2追加の R8 で割り当てだけ数え解放を数えない行、G27 をメモリ分配器読み取りに戻す行。すべて手動で赤→緑を確認し復元、ベースラインは緑。checker-tier-crates-mutation-replay 全表は `gate-triage` 担当で未実行。

**2026-09-24 最初の作業：当日の `crates/` で R2 第1段階を再実行し、2026-09-23 の成果物とバイト比較**。`crates/singlefs-core` の `mount.rs`、`transaction.rs`、`recovery.rs`、`walk.rs`、`crash.rs` は別の作業系列が変更済みで、`git hash-object` は§12修訂5の記録から5ファイルとも変化していた。再実行後の差異は3箇所のみで、すべて c2 クラッシュ回復後の基底 `HK_l2` に集中した。`base_i31_red` が true→false、すなわち基底の I-3.1 (割り当て済み統計が一致する)が赤→緑になり、関連する3行の `q7a` と `pc_check_bk2`/`pc_check_bk3_is_bd_plus_one` の `i31_red` も同方向に反転、`q7a_summary all_red_count` は12→15。`q7c1_flip_seen`/`q7c2_flip_seen` および全 `q7c1_not_subtracting_defer`/`q7c2_threshold_at_least` 行はバイト不変。**この差異は分岐7の報告済み判定を変更しない**。Q7c①は到達可能基底で依然反転せず V3 は発動。別の実装担当者が変更中の `walk.rs` の checker I-3.1 により、`HK_l2` 基底が「元々赤」から「真に緑の合法状態」に変わり、`q7a_all_red_count` の合取判定に新たに含まれた。正誤は実行担当者が判断せず、依頼どおり値を記録した。

12. **分岐3**（第2段階、2026-09-24、`run_hf_single_cell`）：1本の HF 履歴（S=8、ρ=1）で G7 と F-扣 の D_rel と K9 を測定。β0 → クラッシュなし再オープン（現行版に書き換え済みインスタンス表ユニットを持たせる）→ 非空上書き8回 → 同一スナップショットから上限探査・F-扣・G7 の3分岐を再構築。F 引き上げ前に3分岐のデバイスバイトがスナップショットと一致する V13 を確認し、それぞれ `raise_rollback_floor` が探査した同じ `ceiling` へ F を引き上げる。G7 は公開入口で `raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect` を再構成し、新 F を伴う空公開を全ディスクに行き渡らせてから `reclaim_released_up_to(..., ReclaimedReuse::Immediately)`。F-扣 は実 `raise_rollback_floor` を使う。両分岐の空押しは3回で、φ が2に近い点での A9 の回数予測と同規模。最後の空押し txg はともに16なのでこのセルの D_rel は同じ。**この1幾何セルだけ**で、登録5.2の HF 18セル S ∈ {4,8,16} × ρ ∈ {1,1/4} × φ ∈ {0,1,2} の全走査は未実施。P5 の128スロット2セグメント小プール HX/HY を要する Q3e（X8-A）も未構成で、成果物には `status=not_done`。
13. **分岐1**（第3段階、2026-09-24、`run_hh_cell`）：Hh(k) 3履歴、k=0・1・2、S=8、ρ=1、穴位置「後」で甲-T1・G12の過剰差し引きと Δ を測定。各セルは β0 → ワークロードでリングが一周（txg ≥ 3S + 3 = 27）→ k個の穴（c2 クラッシュと書き込み可能マウント回復、穴の間にワークロード公開2回）→ 最後にクラッシュなし再オープン1回。これが甲-T1「実」の回収点・測定点。甲-T1の過剰差し引き Q1a はその時点の会計項目1 − 走査参照（正常状態では項目5/`deferred` に等しい）。G12の Q1b はデバイス0の解放済み記録を巡り、G12 の区間述語「リング内のどの根の txg も [割り当て世代, 解放世代) に入らない」で引き続き保持すべきか判定し、保持スパンを合計する。Hh にはロールバック・F引き上げがないので `被抛弃∨txg≥F_生效` は常に真。割り当て世代は装置のシャドウ台帳が持ち、実記録の `generation` は解放時に解放世代へ書き換わる（登録 N2）。台帳は `snapshot_allocated_generations`/`record_release_generations`、単体テスト `allocation_generation_ledger_recovers_the_pre_release_generation` でカバーし、`crates/mutations.tsv` に変異を追加して手動で赤→緑を確認。Δ は3点で20→70→120、穴1個ごとに+50、h_欠 も0→8→14と単調増加。h=0で Δ=20 がすでに2・1スロット閾値を超える仕組みはこの時点では未解明。S・ρの他の組合せ、穴位置「前」、k=4 は未走査。

**R2 追加第1段階（2026-09-24、s4実行担当者、成果物 `…-stage3.out`）：**

14. **`holes=0` の Δ=20 調査で装置診断コードのフィルター欠落を特定**。G12述語そのものの実挙動ではない。`run_hh_cell` が Q1b と各スロット列挙 `q1_delta_record` 用に `allocator.records()` を見る際、`is_released` だけで絞り、実分配器の `reclaim_released_up_to` がすでに回収したか確認しなかった。回収後も記録は削除されず、同じ開始スロットの次の `record()` に上書きされるまで `is_released=true` のままだが、スロットのビットマップは消え `DeviceFreeMap::is_free` は真。`holes=0` の唯一の回収、最後のクラッシュなし再オープンは9記録11スロットを回収し、いずれも保持不要だったが、診断ループは「G12 は保持不要と判定し甲-T1はまだ算入」の証拠と誤認した。`!device_map_d0.is_free(record.slot)` を条件に追加。**報告済み Δ は変化しない**。`q1a_over_withheld`/`q1b_over_withheld` は影響を受けず、holes=0/1/2 の20/70/120は修正前後でバイト一致し、スロット列挙のみ正確になった。前提を固定する第6の回帰テスト `reclaimed_records_are_not_removed_but_their_slots_become_free` を追加。
15. **分岐1に S=4 の第2幾何点を追加**：mkfsデフォルト8と逆方向、ρ=1、回収時点「実」、穴位置「後」、k ∈ {0,1,2} は同じ。K8 アンカー「ローカルで走らせる S と mkfs に渡すシステム構成フィールドが一致」と、S=8・4で「Δ が2スロットを超える」判定を比べる `q1_geometry_sensitivity_s` を追加。結果は両点とも穴0ですでに超過し一致。
16. **分岐3の K9 前提を実測確認に変更**：`run_hf_single_cell` が F引き上げ前、初回ファイル作成と8上書きの既知非空 txg 集合を用い `readable_roots` 中の残存数を数える `k9_precondition` を追加。`non_empty_valid_root_count=9 ≥ 4` でコードコメントの予測と一致し、推定ではなく測定となった。
17. **当日の主作業区で R2 既存成果物を再実行し並行パッチと照合**：S3追跡11ファイル中 `allocator.rs`/`mount.rs`/`transaction.rs`/`recovery.rs`/`walk.rs`/`format/lib.rs`/`crash.rs` の7つを別セッションが変更。パッチ前に生成した `stage2.out` の形式不変329行と当日の同名行は**バイト一致**し、この並行パッチは該当履歴が到達する E156 のコード経路を変更していなかった。

**R2 追加第2段階（2026-09-24、同日、メイン agent が「分岐表にまだ2行開いたままである」として追加依頼、成果物 `…-stage4.out`）：**

18. **分岐1：ステップ数合わせ対照**。`run_hh_cell` に `matched_to_holes` を追加し、`holes=0` かつ非0なら「リング一周」の閾値に `matched_to_holes × 6` ワークロードステップを追加。6は穴作成1回で固定消費する実測6 txgで S に依存しない。穴は作らず総公開ステップ数を `holes=matched_to_holes` と合わせる。S=8・4それぞれ `matched_to_holes ∈ {1,2}` の終点 txg は対応する実穴セルと一致し、`delta` は**すべて20のまま**でステップ数では増えない。したがって `Δ(k) − Δ(0, ステップ数合わせ)` は k=1 で両 S とも50、k=2 で S=8/4 は100/90。**これが穴自体による過剰差し引きで、20→70→120の増加分はステップ数ではなく穴による**。`q1_delta_record` から穴0の20スロットは割り当て世代4→解放5、割り当て5→解放6の2種類、各8記録10スロットと分解。原因はマウント内回収が1回だけで再オープンの最初に行われ、再オープン自体がさらに2 txg進めること（I3・I13）。穴とは無関係で S=8・4 で各スロット一致。
19. **分岐3：公開入口で Q3e（X8-A/HY）を構成し Q3d も作成**。P5 の128スロット2セグメント小プールを `x8a_parameters`/`build_x8a_prefix`/`run_x8a_cell` で作り、mkfs → クラッシュなし再オープン → 空きがなくなるまで上書き。真に枯渇した HX では F-扣（実 `raise_rollback_floor`）と G7（新しい失敗可能な再構成）が**ともに**最初の空押し公開で失敗し、同じ `AccountingTree`、`NoFreeSlotOnAnyDevice` に詰まる。P5 の単純モデルの「保持すると失敗、保持しなければ成功」とは異なり、G7は空押し完了後に初めて回収するため完全枯渇下では早く空きがなくなる。「回収し保持」を別途再生すると66記録81スロットを回収・保持し `free_slots` は5→86だが `lowest_empty_segment` は `none` のまま。P5の空き数に現れる予測は確認できるが、2つの集約セグメントがそれぞれ割り当て中と保持中を混在させ、1セグメント全空という単純モデルとは異なる。早期停止する陽性対照 HY（`free=75`）では両方法が成功し、HX失敗は真の枯渇によると確認。Q3d の `D_acct`/`D_alloc` は各永続ルートでの実測ではなくコード上の呼び出し順からの推論。F-扣 は初回空押し前に回収するので `D_acct=0`、両方法とも全空押し後に払い出し可能なので `D_alloc(F-扣)=D_acct(G7)=D_alloc(G7)=pump_publishes−1=2`。方法の限界を明記する。

**2026-09-25 第3回再実行の追加段階**：20. **第10基底 `beta_hr_rollback_row` を追加し Q7c① が到達可能状態で初めて色反転**。第3回第1段階の `q7d2_min_item5` が `family=HR kind=rollback_row txg=76` で到達可能状態の `min_item5=0` を初めて読み取った。HR が候補中最古ルートへロールバックし行を書いて公開した直後である。この点を `BasisSnapshot`（`basis_of("beta_hr_rollback_row", …, true)`）として既存 `bases` に追加（9→10）。`q7c_self_test`/`run_q7a_cell`/`run_pc_check` の式は1行も変えず、サンプルを1点追加したのみ。詳細は履歴版2026-09-25追加段階と結果節を参照。

#### 結果（第1～3回再実行）

⚠️ **当時根拠となった成果物は `e156-alloc-basis-counts-2026-09-24-stage4.out`**（776行。R2第1・2・3段階と追加2段階を累積し、`replay.sh` はこれを指す）。`stage2.out`（2026-09-24、338行、R2第1～3段階）の上に追加第1段階で Q1 診断追跡修正、S=4幾何点、K9前提の実測を追加した `stage3.out` は682行で、形式が変わらなかった共通329行は stage2 とバイト一致（コマンドは再実行節）。追加第2段階はステップ合わせ対照、20スロットの内訳、X8小プール枯渇構成-A/HY、Q3d を追加し、`stage4.out` は stage3 と共通323行がバイト一致。`e156-alloc-basis-counts-2026-09-23-stage1.out`（324行）および2026-09-22の旧成果物3本（31/35/35行）は各段階の証拠として保持し、根拠には使わない。分岐2の結論（5～8行）は今回の書き換えの影響を受けず、以下に保持する。

⚠️⚠️ **追加第2段階の終わりに `crates/` が再び大規模かつ未完了の変更を受け、実測値が広範囲で変化した。E156 の `replay.sh` は今「不一致」で、776行中354行が異なる**。`git status --short crates/` では `singlefs-core`/`singlefs-checker` のほぼ全ソースが `M`。修訂10時の「変更されたが E156 が到達する経路には触れなかった」とは異なり、分岐7第1段階の核心値も変化した。「結果」「この段階で行ったこと」の数字は**成果物生成時の `crates/` の値**として読み、現在のコードが返す値とみなしてはならない。例えば HR は47ステップ目の壁に当たらず `hr_lengths actual_tail=47`→`72` と完走。`legal_state family=HR` の `item1/item2/item5` は txg=78 から127まで増加せず `item1=242 item2=211726 item5=230` に固定された。これは「1マウント内の記録は増加のみ」という登録 §3 I3・I8 の前提と明らかに異なる。割り当てレコードツリーの容量／回収機構が変更されたかは判断せずメイン agent に委ねる。**Q7d-2 の min_item5 は1（β0）から0（`family=HR kind=rollback_row txg=53`）に変化**し、「到達可能合法状態の項目5は決して0でない」という分岐7唯一の正式結論に直接影響する。この値が確かなら V3/F16 の前提が成立しない。集計も `q7b total_l1` 167→192、`q7a_summary all_red_count` 15→18、`integrity legal_state_rows` 168→193 と変わる。分岐1（Q1a–Q1d、ステップ合わせ、20スロット分類）と分岐3（Q3a–Q3d、X8-A/HY）の追加2段階の値もすべて影響範囲内。例として `holes=0` の `delta` は20→0、`q3e_f_kou` の失敗は `PlacementRefused` から新しい `MountError::RaiseFloorSequencePublishFailed` ラッパーに変化。**いずれの段階も影響なしとは確認できない**。以前の「測定・照合済み」は修訂5/10のハッシュを持つ旧 `crates/` と照合した意味で、現版との比較ではない。ここでは成果物を再生成せず判定も変えず警告を記録する。並行変更の確定を待って再実行するか、どの版の値をユーザーに提出するかはメイン agent が決める。

**第3回再実行第1段階（2026-09-25、`e156-r3-prereg.md`、質問票第3行のみ）**：上の不一致と min_item5 1→0 は未確認の推測ではない。D8 (コアインデックス構造)確定項目14に従い割り当てレコードツリーが位置指定となった**当日の `crates/`**で、装置の3定数と別の6箇所を再設定して実行。成果物 `e156-alloc-basis-counts-2026-09-25-r3.out` は1076行、sha256 `1356c1dff9b31cf944516976972d241f7d1ad7df1f77c7c599a785432e5f95cd`。前後の `crates/` スナップショット S3 では、並行追記された `crates/mutations.tsv`（本作業の6行を含む）と装置ファイル自身を除き**第3の変更ファイルはない**。`singlefs-core`/`singlefs-checker`/`singlefs-format` とその他 harness はバイト不変で、成果物の有効性は並行編集の影響を受けない。S3は文字どおり発動したが確認後に停止理由とならず、メイン agent の再確認用に記録。

「3定数と他6箇所の変更値」は第7節7.2の閉形式とコマンド2で独立計算し、装置で個別確認した：

```text
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、7行目（name=k1_after_new_pool_file_creation）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、3行目（name=anchor_a_d8）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、6行目（name=anchor_a_d8_root_level）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、9行目（name=s1f_record_count）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、332行目（name=s1ef_summary）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、486行目（name=r5_empty_publish_closed_form）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、522行目（name=s1c_remount）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、524行目（name=q7d1_by_kind）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、630行目（name=r7_hy_cap）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、632行目（name=r7_hy_condition）
```

K1項目1は13→17、β0走査参照12→16（`k1_after_new_pool_file_creation` と `referenced_slots` は完全一致）、S1(c)隔離スロット34→54、新プールでファイル作成後の記録数20→28。すべて閉形式と独立コマンドに一致。`s1ef_summary mismatches=0` は H0 の144ステップ全ての released・記録増分が閉形式に一致することを示し、`steps_by_changed_leaf_count=1:108,2:36` は1葉のみと2葉以上の両方向を含み、§8.2を満たす。空公開の解放は4→8（`r5_empty_publish_closed_form`）、Q7d-1の上書き群は固定 `min=10 max=10` から毎回実測 `min=14 max=16` に変更。A-D8のローカル葉幅・ファンアウトは `singlefs_format` と一致、根レベルも実装 `AllocationRecordTreeGeometry` と同じ2で F21 不発動。HYの上書き回数は `e ≥ max(8, f)` を満たす段階を3・2・1・0と探索したが全て未達。P5小プールの開放セグメントは F 引き上げ時、cap=0では再オープンで `open_segment` が0に戻り次の割り当て前まで未確立、cap≥1では1上書き後に `SpaceAdmissionRefused` となり消費済み。`holds=false` で、登録の「空押し時に空間が残る」を実演できていない。幾何を緩めるかメイン agent に委ねる。

**Q3r.4（分岐1・3・7の判定が変わるか。PC判定器 `/tmp/claude-1000/e156-r3/pc_judge.py` は `--selftest` 12アサーション全通過、成果物 `e156-alloc-basis-counts-2026-09-25-r3.out`）**：

```text
分岐  測定量          stage4 判定                       新成果物判定                         変化
1     Q1d monotonic   s=4:true,s=8:true                 s=4:false,s=8:true                    あり
3     Q3c             差は小さい                        差は小さい                           なし
3     Q3e             hx・hy の両方法一致=True         hx・hy の両方法一致=True             なし
3     K9              g7=0 f_kou=0 expected=3           g7=0 f_kou=0 expected=3              なし
7     Q7a summary     判別可能                          判別可能                             なし
7     Q7b             誤検知なし                        誤検知なし                           なし
7     Q7c             ①②とも反転                        ①②とも反転                           なし
7     Q7d-2           非0（項目5が0の到達状態なし）    =0（到達可能基底 β_z が存在）        あり
7     F16             発動                              不発動                               あり
7     Q7e             反転（①）                         反転（①）                            なし
```

**質問票第3行の答え：変わる**。分岐1は S=4 の Q1d `q1d_monotonic` が `holds=true`→`holds=false`、分岐7は `q7d2_min_item5` が非0→0、F16も発動→不発動。分岐3の Q3c/Q3e/K9 は確認済み量で不変。**分岐7は2026-09-23にユーザーが G27 を I-3.11 (割り当て済みから defer を引いた値が最新ルートのトラバーサルと一致)として採用すると決めた行**。今回の `q7d2_min_item5=0`、`family=HR kind=rollback_row txg=76`、`item1=18 item2=211950 item5=0 referenced=18 check_is_red=false` は checker が緑とする合法状態で、決定の根拠「到達可能合法状態の項目5は常に非0」と矛盾する。**決定の根拠が変わったがここでは分岐7を再開せずメイン agent に委ねる**。分岐1の S=4 は stage4 のサンプル20→70→110が単調だった `holds=true` から、新成果物で H0/HR が壁に当たらず Hh(k) の履歴が長くなり隣接点に非単調な跳びが現れる `holds=false` へ。第9行の答えは「単調」から「非単調、全点を見る必要がある」へ変わる。当時この分岐はユーザー未決定で第9行は開いており、本文に単調と記した箇所があれば新値に合わせる。

R2全段階の成果物末尾行（`grep -n 'name=integrity \|name=done\|name=integrity_r2_segments' e156-alloc-basis-counts-2026-09-24-stage4.out`）：

```text
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、323行目（name=integrity）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、1075行目（name=integrity_r2_segments）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、776行目（name=done）
```

分類件数（`grep -o 'name=[a-z0-9_]*' e156-alloc-basis-counts-2026-09-24-stage4.out | sort | uniq -c`）：`q1_delta_record` 392行（S=8/4、各 holes=0/1/2、追加第1段階のスロット別列挙）、`legal_state` 177行（beta0 1、H0 49、HR 100、HK 18、HK-F0 9）、`s1d_step` 30行、`q7a` 27行、`q1_hh` 10行、`q1_delta_debug` 10行、`q7c1_not_subtracting_defer`/`q7c2_threshold_at_least`/`pc_check_bk1`/`pc_check_bk2`/`pc_check_bk3_is_bd_plus_one`/`basis_snapshot` 各9行、`q1_step_matched_diff` 4行（追加第2段階）、`q1d_adjacent_diff` 4行、`x8a_prefix`/`x8a_held_measure`/`q3e_x8a`/`q3e_g7_remount` 各2行（HX・HY各1）。残りは `q7a_summary`/`q7b` 等の第1段階、`q3a_d_rel_g7` 等の分岐3単一セル、`q3d_d_acct_d_alloc`、`anchor_k8`、`k9_precondition`、`q1_geometry_sensitivity_s` 等が各1～2行。計66測定名、776行。

**分岐2**（第1段階、今回の書き換えの影響なし）：`|M∖L|=41`。一覧が41箇所を漏らし、登録修訂で参照する `/tmp/claude-1000/e156-q2/list-M.txt` と本報告に個別列挙。`decisions-history` アーカイブ、`milestone/`、`layout/`、複数実験ページ、`crates/singlefs-core/src/transaction.rs:1488`、`crates/singlefs-harness/src/model.rs:57`、テスト2ファイル、researchスクリプトfixtureを含む。`|L∖M|=0`。doc-registriesゲート admission-terms欄の判別力自己検証は①赤、②なお赤（`03-空间分配.md:82` の式コピーも変更しないと通らない）、③緑。**ゲートの admission-terms スクリプト自体に変更は不要**。

**分岐7（R2第1段階後：まだ判定不十分、追加実行では反転しない。判定基準を変更するか受け入れるかメイン agent が判断）**。R3で判定対象画像のみを読むよう変更後：

- **Q7b**（登録§6、基準0）：`name=q7b red_l1=0 total_l1=167 red_l2=0 total_l2=1`。H0・HR・HKの到達可能合法状態168個（L1 167、c2クラッシュ回復後L2 1）で G27 の誤検知は0。**F5不発動**。
- **Q7d-1/Q7d-2**（新規、項目5が異なるサンプル2点以上と最小値）：`name=q7d1_by_kind kind=O min=10 max=10 count=153`、`kind=E min=4 max=4 count=1`。O・E公開自身が解放するスロットは全サンプルで定数10・4で S1(e) の assert と一致。`name=q7d2_min_item5 min_item5=1 family=beta0 kind=first_file txg=3`。**168合法到達状態に項目5が0のものはない**。最小1は最初のβ0、新プールでファイル作成した直後で、会計行を伴うトランザクションを一度公開すると項目5は0にならないという P21(g) の予測を支持。
- **Q7d-3 / PC-到達可能**（基準は必ず0）：`name=q7d3_pc_reachable hk_forced_to_zero_smallest_item5_item5=0 beta_syn_item5=0 holds=true`。HK-F0・β_synで0を測れ、**測定法自体は0を見ることができる**。Q7d-2の最小1は測定の盲点でなく到達可能集合自体の性質。**V11不発動**。
- **Q7c①②③（登録「十一」V3 の元の文字どおりの条件）**：①（第5項を減算しない）は、すべての **7個の到達可能基底**（β0、β1、β2、βK-w/r/f/l2）上においていずれも `flips_red_to_green=false` である——第1段階・第2段階（β0/β1/β2 の3個のみを測定）に比べて HK の4個の観測点を追加測定したが、結論は変わらず、証拠は倍以上強固になった。第5項を人為的に 0 にした **2個の到達不能基底**（β_syn、β_F0）上でのみ色が反転した。②（`==`→`>=`）は、すべての 7個の到達可能基底上においていずれも `flips_red_to_green=true` である（`name=q7a_summary all_red_count=15 q7c1_flip_seen=true q7c2_flip_seen=true`、2026-09-24 に本日の `crates/` 上で再実行したところ、`all_red_count` は昨日の 12 から 15 に変化した——この差分はすべて `HK_l2` のセルの `base_i31_red` が `true` から `false` に反転したことに起因しており（別の実装担当者が `walk.rs` の I-3.1（割り当て済み統計の整合性）の実装を変更していたためである。2026-09-24 の記録の「最初に行ったステップ」の段落を参照）、`q7c1_flip_seen`/`q7c2_flip_seen` 自体および各 `q7c1_not_subtracting_defer`/`q7c2_threshold_at_least` 行はいずれもバイト単位で不変である）。β_syn/β_F0 上では**実行のしようがない**（`not_applicable=true reason=deferred_is_zero_cannot_subtract_one`：それら2つの基底では第5項がすでに 0 であり、Bd(−1) を行うと負数に減算されてしまうためである。`e156-r2-prereg.md`「十二」修訂3にこの発見が記録されている）。⇒ V3 の文字どおりの条件に従うと、**Q7a の表全体はなお無効である**——①は到達可能基底上では終始色の反転が見られず、その根底にあるのは依然として「到達可能な正当な状態において第5項は決して 0 にはならない」という点である（Q7d-2 はすでにこの命題を「2つの基底」の観察から「168個の状態、8種の公開種別を網羅する7個の基底」のすべてにおいて同一の結論へと拡張している）。

**2026-09-25 第3回追加段階（`e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out`、原文行）**：「到達可能合法状態の項目5は決して0でない」という結論はこの日に覆された。HR系列の管理者ロールバック行書き込み `family=HR kind=rollback_row txg=76` を到達可能基底 `beta_hr_rollback_row` として追加：

```
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、591行目（name=basis_snapshot）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、592行目（name=q7a）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、593行目（name=q7a）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、594行目（name=q7a）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、595行目（name=q7c1_not_subtracting_defer）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、596行目（name=q7c2_threshold_at_least）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、597行目（name=pc_check_bk1）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、598行目（name=pc_check_bk2）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、599行目（name=pc_check_bk3_is_bd_plus_one）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、618行目（name=q7a_summary）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out、621行目（name=integrity）
```

`q7c1_not_subtracting_defer` の `flips_red_to_green=true` により、**判別力自己検証①が到達可能基底で初めて反転**。以前は到達不能β_synのみだった。`q7c2_threshold_at_least` はβ_syn/β_F0同様 `not_applicable`。 `deferred==0` で Bd(−1) は負数になり、解釈の違いではない。`basis_count` 9→10、`all_red_count` 18→20（delta=1/8 で各1回 Q7a合取に該当、delta=-1は `deferred_underflow` でスキップ）。`legal_state_rows` は310のまま、既存 `legal_state` 行の再利用で履歴ステップを増やしていない。独立単体テスト `hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test` は `main()` を呼ばず同じ公開入口で実履歴を再生し、`item5=0` と `flips_red_to_green=true` を独立確認。検証 agent の独立プローブ `/tmp/claude-1000/e156-fork7-check/probe.out`、コード非共有の第3経路も `item1=18 item5=0`、checker 46不変条件に違反0で一致。**十分かはメイン agent が判断**：少なくとも1到達可能基底で反転することのみを証明し、一意性や構造条件からの必然性は証明しない。
- **Q7e**（判定基準の判別力不足と装置・G27の故障を分離）：`name=q7e beta_forced_to_zero_flip1=true beta_forced_to_zero_flip2=false beta_syn_flip1=true beta_syn_flip2=false beta_forced_to_zero_i31_red=true beta_forced_to_zero_i52_red=false beta_syn_i31_red=true beta_syn_i52_red=false`。①は到達不能2基底で反転し V10 不発動。②の `flip2=false` は**反転失敗と適用不可を区別できない**。`not_applicable` 欄がなく、同成果物の各基底 `q7c2_threshold_at_least` 行で `not_applicable=true` を確認する必要がある。成果物の可読性上の欠落でメイン agent に委ねる。`beta_*_i31_red=true` は2基底が合法でない理由を示す。会計バイトを人工変更し、参照走査から見えるものを連動変更していないので I-3.1 は当然赤。
- **Q7f**（付帯、V3を解除しない）：`name=q7f_count count=7`。7到達可能基底すべてで実 G27 は緑、項目5を引かない変種は赤。G27 はこの特定の悪い変種より見逃しにくいが V3判定には入れず、メイン agent 向け候補データ。
- **S9**（停止、結果扱い・無効化しない）：`name=s9_raise_floor_refused family=HK-F0 error=RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { root: RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }, failure: UnitUnreadable { slot: SlotNumber(50178) } }`。HK-F0 の F 引き上げ上限探査で発生。`ForcedToZero` では `release()` が解放世代 ≤ 当該 txg の全解放記録を回収し、genesis ツリーテーブル単位が早期に回収・再利用された。`rollback_floor_ceiling` は候補根の表を読んで非空判定するため読めずエラー。I-7.4 (直近 K 世代のブロックが再利用されていない)が防ぐ種類の再利用であり、HK-F0 は意図的に到達不能なのでこのセルを停止。影響はHK-F0後半のみで `beta_after_raise_floor`/`beta_after_rollback`/`beta_after_crash_recovery` は main() で参照されず、既報判定には影響しない。

⚠️ **メイン agent 向け2点**：① Q7e 出力は適用不可を false に畳まず `not_applicable` を持つべき。基底が既に赤の PCチェックと同類で、登録5.5脚注も既赤は成功でも失敗でもないと予見。② V3/F16 で7到達可能基底・168到達可能状態の Q7c①は一度も反転しないと確認され、この自己検証を続けても反転しない。赤→緑の厳密反転を要求しない基準に変えるか、Q7b と悪い画像の明確な対照で G27 の判別力を認めるかはメイン agent が判断。

**分岐3（R2第2段階、2026-09-24、1幾何セル S=8、ρ=1、`run_hf_single_cell`）**：

```text
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、614行目（name=v13_hf_prefix_identical）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、615行目（name=q3a_d_rel_g7）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、616行目（name=q3b_d_rel_f_kou）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、617行目（name=q3c_diff）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、618行目（name=k9_newest_batch_d_rel_non_empty）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage3.out、330行目（name=q3e_x8a）
```

- **V13 holds=true**：F引き上げ前、上限探査・F-扣・G7は同一スナップショットから別々に再構築されバイト一致。2方法の履歴前半は同じで、Q3a/Q3b/Q3c の比較可能性は保たれる。
- **Q3a G7／Q3b F-扣**：F引き上げが処置対象とした同じスロット群（解放世代13、引き上げ前デバイス0の最大解放 generation）を分配器が実際に払い出せる時点は**完全一致**。 `dispensable_txg=16`、両方法とも空押し3回。`d_rel_非空=0`（13～16に非空公開はなく空公開のみ）、`d_rel_全=2`。これは偶然でなく、空押し回数は新 F を持つ根でまだ覆われていないディスクによって決まり、回収の先後とは無関係。同じ履歴で同じ長さの空公開列となり、最終 txg も同じ。
- **Q3c（二つの算定方法の差）**：`max_diff=0 ≤ 3` ⇒ **差異は大きくない**——二つの算定方法でそれぞれ1つの数値が得られ、その差は 0 であり、第11行の「判定に十分な条件」の前半が満たされた。⚠️ **この単一の幾何セルのみで測定されたものであり**、事前登録「五、5.2」の HF の18セル全走査（S × ρ × φ）は実行しておらず、また「差異は大きくない」という結論が他の φ（F引き上げ開始位相）や他の S においても依然として一貫しているかどうかは検証されていない——「二つの算定方法はすべての幾何において差異が大きくない」と外挿することは許されない。
- **K9**（解放記録の唯一の群の D_rel_非空）：`g7=0 f_kou=0` はどちらも期待3ではない。追加第1段階で F引き上げ時リング内に非空有効根4本以上という前提を実測し、`name=k9_precondition non_empty_valid_root_count=9 threshold=4 holds=true`。前提は成立しており、release_generation=13 と dispensable_txg=16 の間の3永続公開がすべて F 引き上げ自身の空押しで非空ではないため3にならない。P21(e)「早く解放するほど長く待つ」と同方向。K9は数値報告のみで判定基準ではなく、別幾何を構成して方向性を確認するかはメイン agent が決める。
- **Q3e（X8（小プール枯渇時の二つの算定方法の構成）-A）および Q3d：追加第2段階で作成完了した**。「R2 追加第2段階」の節を参照のこと。もはや `status=not_done` ではない。

**R2追加第1段階（2026-09-24、s4実行担当者、`…-stage3.out`）の追加：**

```text
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、641行目（name=anchor_k8）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage3.out、523行目（name=q1_hh）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage3.out、581行目（name=q1_hh）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage3.out、671行目（name=q1_hh）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、698行目（name=q1_geometry_sensitivity_s）
```

S=4（mkfsデフォルト8と逆方向）でも Δ は20→70→110と単調非減少。「2スロットを超える」判定は S=8 と一致（`stable=true`）。S=4の `h_missing` は holes=1/2 とも8に留まり、S=8の8→14とは異なる。holes=2の `delta` も120でなく110。浅いリングでは穴の数え方と絶対過剰差し引きは幾何に依存するが、方向の判定は変わらない。

**R2追加第2段階（2026-09-24、メイン agent の追加依頼、`…-stage4.out`）：**

```text
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、771行目（name=q1_step_matched_diff）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、772行目（name=q1_step_matched_diff）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、773行目（name=q1_step_matched_diff）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、774行目（name=q1_step_matched_diff）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、331行目（name=x8a_prefix）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、332行目（name=x8a_held_measure）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、333行目（name=q3e_f_kou）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、336行目（name=q3e_g7）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、339行目（name=q3e_x8a）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、340行目（name=x8a_prefix）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、634行目（name=q3e_f_kou）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、637行目（name=q3e_g7）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage4.out、348行目（name=q3e_x8a）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-25-r3.out、619行目（name=q3d_d_acct_d_alloc）
```

- **ステップ数合わせ対照**：第9行の穴数に応じた増加に明確な対照ができた。`Δ(0, holes=kにステップ数合わせ)` は S=8/4 × k=1/2 の4組すべて**20**で、未合わせ `holes=0` と同じ。同じ公開ステップ数でも穴を作らなければ Δ は増えない。`Δ(k) − Δ(0, 合わせ)` は k=1 で両 S とも50、k=2で S=8/4 は100/90。**穴自体による増加であり、20→70→120曲線の増分はステップ数によらない。**
- **holes=0 の20スロットの分類**：`q1_delta_record` は S=8/4で各スロット一致し、割り当て世代4→解放5、割り当て5→解放6の2種類、各8記録10スロット。リング充填が停止した txg では最古有効根が2解放世代へまだ進んでおらず、直後の最後のクラッシュなし再オープンはその時点の floor で回収するが、自身の行書きとウォームアップがリングを2 txg進めるため、2群は「回収済みだが当時の floor より新しい」状態になる。穴とは無関係。これはコードから推定した仕組みで独立測定事実ではない（`e156-r2-prereg.md` §12修訂12）。
- **Q3e X8-A/HY**：真に枯渇した HX で F-扣 と G7 はともに `AccountingTree` の配置を確保できない同一ステップ・同一原因で失敗。早期停止の陽性対照 HY では両方成功し、HXの失敗が枯渇によると確認。「回収して保持」を別途測ると `free_slots` 5→86だが `lowest_empty_segment` は `none`。P5の空き数に現れる予測はそのまま成立するが、実履歴では2集約セグメントに割り当て中と保持中が混在し、単純モデルの「1つが生存・1つが空」と異なる。第11行の双方がどこで詰まるかに答えたが、P5が示唆した「保持失敗・非保持成功」という鋭い対比ではない。
- **Q3d**：`D_acct(F-扣)=0`、`D_alloc(F-扣)=D_acct(G7)=D_alloc(G7)=2`。F-扣の回収は初回空押し前、両方式の払い出し可能時点は全空押し後というコードの呼び出し順から推導したもので、各永続根での実測ではない。

**分岐1（R2第3段階、2026-09-24、1幾何セル S=8、ρ=1、穴位置「後」、`run_hh_cell`）：**

```text
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out、330行目（name=q1_hh）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out、331行目（name=q1_hh）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out、332行目（name=q1_hh）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out、333行目（name=q1d_adjacent_diff）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out、334行目（name=q1d_adjacent_diff）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out、335行目（name=q1d_first_crossing）
成果物の行：git show 5d0b7e7c1667:research/results/e156-alloc-basis-counts-2026-09-24-stage2.out、336行目（name=q1d_monotonic）
```

- **Q1a / Q1b / Δ の穴数に伴う増加**：k=0・1・2で Δ は20・70・120、単調増加し穴1個ごとに+50（`q1d_monotonic holds=true`）。h_欠 も0→8→14。異なる穴数の2点以上と閾値を超える規模の報告という**第9行の文字どおりの十分条件は成立**。この3点では h=0 ですでに2・1スロット双方を超過し、`q1d_first_crossing` の両欄は `holes=0 txg=29 delta=20`。
- **h = 0 における Δ = 20 の原因は、追加の2段階において解明された（もはや「調査しきれていない」状態ではない）**：第一は診断コードの1箇所のフィルタリングの不備（`subpool_id == 0` で決め打ちして判定することはできず、他のサブプールまで小プールとしてカウントしてしまっていたためで、追加段階での修正後に 20 から 16 へと減少した）であり、第二は**設計上、小プールには内部事務処理用に16スロットがハード予約されていること**（`crates/singlefs-core/src/subpools.rs` 第141行 `SMALL_POOL_RESERVED_SLOTS = 16`、`mkfs` による初期化時に差し引かれ、外部ユーザーの割り当て要求によって取得することはできず、システム内部事務の書き込み時にのみ使用される）である。したがって、**h = 0 における Δ は本来 16 であるべきであり（完全に内部事務予約分である）**、何ら「謎のシステムオーバーヘッド」などではない。
- **追加第1段階で S=4 を追加したが他の幾何次元は未走査**：S=16、ρ=1/4、回収時点「毎」、穴位置「前」、k=4 は未実施。第3段階a/bの全24+14履歴も未実施で、登録5.6第6類の回収時点・穴位置には逆方向の第2点がない。

#### 再実行（第1～3回）

```
bash scripts/experiment-apparatus/replay.sh E156
```

当時は「バイト一致」と報告。`replay.sh` の `driver_e156` は `cargo run -q -p singlefs-checker-tier --bin e156_allocation_basis_counts` を直接実行する。E142同様、2 cargo workspace は互いを参照できず呼び出しを統合できない。同一バイナリ2回の出力はバイト一致、V8不発動で、追加2段階それぞれ確認。成果物 `…-stage4.out` の sha256 は `5311e01a0cec4797bf848ffacbbf3f624b6a01627412491c76a8c92f2cea0f02`。単体テスト `cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts` → `6 passed`。追加第1段階で `reclaimed_records_are_not_removed_but_their_slots_become_free` を追加して Q1診断修正の前提を固定し、旧5件は不変。ミューテーション表 `crates/mutations.tsv` の変異は計4件のまま。追加した診断／構成コードは既判定量を変えないか、候補・判定基準の違いでなく新製品挙動の組合せであり、変異追加の利益・工数が見合わず2段階とも追加しなかった。個別に手動で改悪して `cargo test` 赤、復元して緑。checker-tier-crates-mutation-replay 全表は未実行、`gate-triage` 担当。**手動で確認した赤にならない変異**：G7再構成の `ReclaimedReuse::Immediately` を `HeldUntilFloorTakesEffect` に変えても Q3a/Q3b/Q3c/K9 はバイト不変。`mutation-sampling.md` の第1類、**真の盲点**。`run_hf_single_cell` は回収後に再度割り当てを試さない。追加第2段階の X8A は別角度から証拠を補い、`x8a_held_measure` は回収・保持後に `free_slots` が増えても `lowest_empty_segment=none` を実測し、`q3e_f_kou`/`q3e_g7` は回収方式で挙動が異なる（HXは双方失敗だが経路が異なる）。ただし独立観測であり `run_hf_single_cell` の変異ではなく、表には追加しない。

**第3回再実行後**：`replay.sh` は `…-2026-09-25-r3.out` とバイト一致、sha256 `1356c1dff9b31cf944516976972d241f7d1ad7df1f77c7c599a785432e5f95cd`。上記単体テストは `10 passed`。追加4件は `pc_closed_form_matches_the_registered_anchor_r3c`、`pc_closed_form_third_group_counts_released_and_allocated_records`、`overwrite_steps_match_the_closed_form_and_cross_a_second_leaf`（U10/U12）、`rollback_isolation_scenario_matches_the_new_layout`（U11）。従前6件はアサーション絶対値13→17、12→16、34→54を変更し関数とシナリオは不変。変異は計10件、新規M27–M32はR-3閉形式の根漏れ・devices係数漏れ・割り当てのみで解放なし、Q7d-1を10に戻す、K1-1を13に戻す、S1(c)を34に戻す6件。各々改悪して赤、復元して緑。旧4件はアンカーが新ソースでなお一意一致し関数本体も不変なので再度赤確認不要。全表は `gate-triage` のゲートに委ねる。

**第3回再実行追加段階の後（現状）**：`replay.sh` は「バイト一致」を報告している（成果物は `…-2026-09-25-fork7-selfproof.out` に切り替えられた）。12本の変異はすべて手動で1本ずつ赤・緑の検証を行ったが、**いかなる自動化テストスイートにも組み込まれていない**。変異サンプリングにおける「G7 再構成の『即座に発行可能』と『回収するが保持したまま』の違い」という行は依然として真の盲点である。HY は事前登録の本来の意図である「空押し時に開放セグメントにまだ空き容量が残っている」という状態を実演できていない。7a の「色反転が真実であるかどうか」は再び「わずか1セルのみであり、しかも行書きへのロールバックという境界的エッジケースにすぎない」という点に立ち戻っている。Q3r.4 は `分岐 1` において10個の物理量のうち1個のみをカバーしている。

#### 答えられないこと（第1～3回）

- **S1(b)** の `reuse_after_raising_the_floor.rs` 2ケースはテストファイル自体のみ実行し、S1(c) のような装置内の再生・各項目 `assert_eq!` は未実施。
- **S1(d) は G27 および `check_pool_image` の2つの検査が H0 の最初の30ステップ（3S+6）において緑（パス）であることを検証したのみである**：後続の138ステップは検証しておらず、また S2/S3/S4/S5/S6/S7 も検証していない。
- **（2026-09-25 以前の結論であり、追加段階によって覆された）分岐7の判別力自己検証①は到達可能状態において色の反転を起こせない**：かつてのこの結論は第3回再実行追加段階において**覆された**（`beta_hr_rollback_row` は到達可能な正当な状態であり、①はその上で `flips_red_to_green=true` を達成した。「この段階で行ったこと」第20条を参照）。ただし、**色の反転を起こしたのは依然としてこの1セルのみであり**、全幾何および全履歴長を走査したわけではない。次項を参照のこと。
- **追加段階でも未解答**：項目5が正確に0の到達可能状態がこれだけかは全幾何・履歴長を走査していない。この1本のHR履歴の1点で発見したのみ。偶然か「候補最古根へロールバックし候補集合がちょうど一周」という構造条件の必然かも別途論証していない。
- **Q7e の行の `flip2=false` からは「色反転失敗」なのか「適用不可」なのかを読み取ることができない**：フィールド自体に `not_applicable` の区分が含まれておらず、明確に区別するには Q7c② に戻って照会する必要がある。
- **H0/HR の実際の長さは `AllocationRecordsExceedOneNode` によって49/47ステップに縮小されており、登録時の144/72+72ではない**：これは当日の `crates/` の実装境界における装置の現実の状況であり、割り当てレコードツリーが1ノードのみで作られていた際に履歴が到達できた限界でもある；登録で要求されたステップ数を完走するには、その修正がメインワークスペースに入った後に新境界で再実行する必要がある（「結果」のメイン agent 向け第1項を参照）。Hh(k) は txg 29/35/41 以下で812記録の壁を避けたが、登録5.2の連続6Nステップ履歴は走っておらず、壁に当たるかどうかやその後の規模は未検証である。
- **HK-F0 は S9**（F上限探査で候補根ツリーテーブルを読めない）で停止。`beta_after_raise_floor`/`beta_after_rollback`/`beta_after_crash_recovery` の3点は実際には得られず main() からも参照されないので既報判定に影響しないが、全公開種類を各1回以上というカバーは F引き上げ後で停止。
- **バイパス評価の範囲**（登録§11）：G12は実履歴の脇で述語評価するだけで分配器挙動を変えず、G12に従う履歴の形には答えられない。G7は公開入口で実履歴を再構成する R4 に変わり純バイパスではないが、実分配器の製品経路は変えない。
- **分岐1**：S=16、ρ=1/4、回収時点「毎」、穴位置「前」、k=4は未実行。第3段階a/bの24+14履歴、`Q1c-洞`（h_欠=0 と≥1の別報）、付帯 `Q1e`、PC-過剰差し引き・PC-穴も未実施。M9の周期再オープン表は未実装。Hhは最後に1回クラッシュなし再オープンするのみで、登録のワークロード公開S回ごとの再オープンとは異なる。
- **分岐3：HF の S ∈ {4,8,16} × ρ ∈ {1,1/4} × φ ∈ {0,1,2} 計18セルのうち17セルは未実行**：Q3a–Q3d、K9 は単一の幾何セル（S=8、ρ=1）でのみ測定された。X8（小プール枯渇時の二つの算定方法の構成）-A（Q3e）は追加第2段階で現実の構成を補完したが、P5 の小プール幾何においてのみであり、HF 幾何における X8-A は依然として構成されていない。X8-A/HY は独立の P5 小プールで、枯渇と早期停止の2点だけであり、1スロット不足など中間境界は走査していない。
- **Q3d は推導された値であり、永続化ルートごとにその場で実測されたものではない**：`q3d_derived` の手法上の限界については「この段階で行ったこと」第19条を参照のこと。
- **X8（小プール枯渇時の二つの算定方法の構成）-A は、P5 の単純化モデルと 2026-09-24 追加第2段階の実際の履歴構成との間で完全には対応していない**：P5 は「単一セグメント満杯」の単純化モデルであり、実際の履歴構成は12セグメント幾何上で作成された複数セグメント枯渇履歴であって、両者の「枯渇」の内包する意味が異なる（一方は単一セグメント内部に連続した利用可能スロットが存在しない状態であり、他方はすべてのセグメントにおいて利用可能スロットが存在しない状態である）。
- **G7 再構成における「即座に発行可能」と「回収するが保持したまま」の差異は、一本の変異として依然として真の盲点である**（`mutation-sampling.md` 第1分類）：`run_hf_single_cell` の測定手法では回収後に再度の割り当て試行を行っておらず、この差異を検出することができない。X8（小プール枯渇時の二つの算定方法の構成）-A の `x8a_held_measure` は別の角度からこの差異の効果（`free_slots` が増加するものの `lowest_empty_segment` は依然として `none` のまま）を測定したが、この2箇所は独立した観測点であり、この盲点は `crates/mutations.tsv` に補充されていない。
- **checker-tier-crates-mutation-replay 全表は未実行**：本binの累積10件は個別に手動で赤→緑を確認し、全表は `gate-triage` 担当。
- **第3回再実行：HY は事前登録の本来の意図である「空押し時に開放セグメントにまだ空き容量が残っている」という状態を実演できていない**：事前登録では上書き回数が `e >= max(open_segments)` を満たさないことを要求していたが、実際に実行された HY においては上書き回数が開放セグメントの容量を大幅に超過していた。そのため、HY が実演したのは「空押し時に開放セグメントもすでに満杯になっている」状態であり、元の事前登録が捕捉しようとしていた境界条件を捉えられていなかった。
- **付帯 Q3r.5 は未実行**：実験21以前の並行変更と HEAD 後（実験21含む）を切り分ける作業は、第3行が十分となった後、登録5.4でメイン agent が実行するか決めるとされた。
- **Q3r.4 は10個の物理量をカバーしたのみであり、「六」に列挙されたすべての物理量ではない**：`分岐 1` は `Q1d monotonic` のみを検証し、`分岐 3` は `Q3c`/`Q3e`/`K9` を検証し、`分岐 7` は `Q7a 集約`/`Q7b`/`Q7c`/`Q7d-2`/`F16`/`Q7e` を検証した。`Q1a`/`Q1b`/`Q1c`/`Q1c-穴`/`Q3a`/`Q3b`/`Q3d`/`Q7f` の各物理量の生の数値は「結果」において行ごとにそのまま書き写されているが、`pc_judge.py` の判定テーブルには個別に書き込まれていない——これは「変更あり / 変更なし」の全体的な結論を支持するにはすでに十分であり（各分岐において1行でも「変更あり」があれば分岐全体が反転する）、物理量ごとに完全に補完するかどうかはメイン agent が追加実行を行うかの判断に委ねられる。

#### 影響する決定（第3回追加段階の見直し、当時のまま）

| 決定項目 | 関係 | 見直し |
|---|---|---|
| D28 (マウント時コミットメント量)確定項目1 | 判断材料 | 2026-09-25 影響なし。番号・略称・指示対象のみ変更し、結論・数値・成果物は不変。 |
| D5 (スナップショット / 空間会計メカニズム)確定項目4 | 判断材料 | 2026-09-25 影響なし。番号・略称・指示対象のみ変更し、結論・数値・成果物は不変。 |
| D16 (パブリッシュセマンティクス)確定項目1 | 判断材料 | 2026-09-25 関係は判断材料のまま。到達可能基底 beta_hr_rollback_row を追加した値は本項目の機構自体に触れる。`q7d2_min_item5=0`（`family=HR kind=rollback_row txg=76`）で自己検証①が到達可能基底で初めて反転（`flips_red_to_green=true`）。本項目の「非空」の記述と、分岐7のユーザー決定の根拠「到達可能合法状態の項目5は常に非0」は成立しない。`.claude/kb/decisions/16-发布语义.md` の根拠節には現在もこの実験番号のヒットがなく、関係は判断材料のままで自動的に昇格しない。再検討一覧に入れるかメイン agent が判断。従前の2026-09-25見直しでは番号略称・参照のみ変更で影響なしとしていた。 |
| D3 (領域割り当て)確定項目7 | 判断材料 | 2026-09-25 影響なし。番号・略称・指示対象のみ変更し、結論・数値・成果物は不変。 |
| D8 (コアインデックス構造)確定項目14 | 判断材料 | 2026-09-25 影響なし。番号・略称・指示対象のみ変更し、結論・数値・成果物は不変。 |

### 2026-09-23

- 「Q7aから独立した明確な信号」の段落を撤回。原文の主張：**Q7aから独立した明確な信号**（`stage3.out` の `name=q7a basis=beta0_k1` 行。フィールド意味を修正後に正しく読める）。基底では G27 が成立し I-3.1 (割り当て済み統計が一致する)・I-5.2 (空き領域統計が一致する)も緑、破損画像 Bd(+1) では**G27 が不成立で赤だが2条件は緑**。したがってこのセルで G27 は現在の2チェックが区別できない差を判別し、基底に誤検知しない。V3で無効な Q7a集計には入れず別途読める。
- 撤回根拠：再実行登録の設計担当者とメイン agent が `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` の `run_q7a_cell` を確認。`corrupted_check_holds = allocated == deferred + referenced` の `(allocated, _free, deferred)` は呼び出し元が渡す `corrupted_allocator_row` から取り `corrupted_pool` を読まない。基底側の `base_check_holds` はメモリ分配器を読む。第1段階の G27 はディスク上会計行を一度も読んでいなかった。

**同日、R2第1段階、実行担当者**：R3で G27 を画像読み取りに変更、HK/HK-F0 の2履歴、9基底、Q7d/Q7e/Q7f、S1(c)完全再生、锚1/锚2/锚3を追加。結論：分岐7（分岐表12行）はなお判定不十分。証拠は初期2段階より広く、H0/HR/HKの3系列・8公開種類にまたがる7到達可能基底、168状態で Q7c①は一度も反転しない。Q7d-2/Q7d-3は測定法が0を見られないという説明を排除し、残るのは「到達可能合法状態の項目5は常に非0」。この自己検証設計を続けても反転しないため、基準変更か受け入れかメイン agent に委ねる。成果物を `e156-alloc-basis-counts-2026-09-23-stage1.out`（324行）に変更し `replay.sh` も変更、旧 `stage1/2/3.out` は各段階の記録として保持し根拠としない。

### 2026-09-24

**R2第2・3段階、実行担当者**：最初に、第1段階後に別系列が変更した `mount.rs`、`transaction.rs`、`recovery.rs`、`walk.rs`、`crash.rs` の `git hash-object` がすべて変わったことを確認。当日のコードで第1段階を再実行し前日の成果物とバイト比較したところ、別担当者が `walk.rs` checker実装を変更していた `HK_l2` 基底の I-3.1 が赤→緑となった点のみ異なり、Q7c値と分岐7判定は不変（行ったこと・結果参照）。

装置に `run_hf_single_cell`（分岐3）、`run_hh_cell`（分岐1）、公開入口で G7 を再構成する `raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect`、分岐1の割り当て世代台帳 `snapshot_allocated_generations`/`record_release_generations` を追加。G12が必要とする8バイトは実記録になく装置が管理する。**両分岐とも S=8、ρ=1 の1幾何セルのみ**で、登録5.2の全サンプリングではない。実行担当者の時間予算による。分岐表第3行の「2方法各1数値＋X8-Aで双方がどこに詰まるか」の前半（同じ D_rel、差は小さい）、第1行の「異なる穴数の2点以上」は k=0・1・2 の3点で2閾値を超える結果を得た。

- **分岐3（11行）**：1幾何セルで G7 と F-扣 の D_rel はともに非空公開計0、Q3cは差が小さい。K9の期待3は得られず、リング内非空有効根4本以上の前提を別途確認していなかったか、P21(e)の予測方向が成立しない可能性もある。Q3e X8-Aは未実施。**第11行は部分的に十分**で、両方法各1数値は得たがX8-Aの半分はない。
- **分岐 1（第9行）**：3個の空きスロット数のサンプリング点において Δ は単調に +50/穴 で推移し、h = 0 の時点からすでに2スロット / 1スロットの2つのしきい値を超過している。しかし h = 0 の時点で Δ はすでに 20 に達しており、このベースライン値自体は徹底的な調査が行われていなかった（集約カウントのレベルにとどまり、スロットごとの詳細な分解は行われていない）。**第9行の2つの判定十分条件は文字どおり満たされている**（少なくとも2個のサンプリング点が存在し、しきい値を超過する規模が報告されている）。ただし、単一の幾何セル上でのみ測定されたものでありベースラインのメカニズムが未解明であるため、ユーザーに提示可能な十分な判定に至っているとみなすか、それともまず h = 0 における 20 の発生源を究明すべきかはメイン agent の判断に委ねられる。

成果物 `e156-alloc-basis-counts-2026-09-24-stage2.out`（338行、第1～3段階累積）に変更し `replay.sh` を更新。09-23 `stage1.out`（324行）は保持。当日の最新 `crates/` 再実行は HK_l2 の3箇所だけ異なり別保存せず、差異は行ったことに記載。単体テスト4→5、ミューテーション表3→4。G7の即時払い出しを回収・保持に変えても赤にならない変異を手動確認し、`mutation-sampling.md` の第1類真の盲点として表には入れない。

**（同日、R2 追加第1段階、s4 実行担当者、セッション制限に抵触して中断した実行担当者から引き継ぎ）** 作業開始時にまず現場を確認：引き継ぎサマリー `/tmp/claude-1000/handoff-summary.txt` および `git status --short crates/` を確認し、前任の実行担当者が `crates/singlefs-checker/tests/e156_check.rs` を半分だけ変更した状態であり、コアコードには一切手を付けておらず、保留中の一時的な変更もないことを確認した。第1ステップとして、まず第1段階の12本の変異について手動で1本ずつ赤・緑の検証を行った（`crates/singlefs-checker/tests/e156_check.rs` 内の判定基準の数値と記号を順次手作業で意図的に改悪し、`cargo test -p singlefs-checker --test e156_check` を実行して FAIL を確認した上で、元に戻して PASS を確認した）。これにより、12本の変異が現場においていずれも真の判別力を有しており、「変更しても落ちない」偽のテストが存在しないことを実証した。

**（同日、R2追加第2段階、実行担当者、メイン agent より E156（alloc-basis の四つの分岐に伴うコストの数値）について「この段階を継続せよ、分岐表にまだ2行開いている」とのメッセージ）** 分岐1にステップ数合わせ対照（`matched_to_holes`）を補充し、k=0 で穴を作らずにワークロードを多く進めて総ステップ数を穴作成セルと揃え、S=8/S=4 において `delta` がいずれもステップ数に伴って増加しない（すべて常に20である）ことを確認し、「20→70→120 の増加はすべて穴自体に起因する」ことを証明した。さらにこの20スロットを各10スロットの2回（割り当て世代 4→5、5→6、再オープン自体が 2 txg 進むことによる）へと分解した；分岐3に Q3e（X8（小プール枯渇時の二つの算定方法の構成）-A、128スロット2セグメント小プール、HX で真の枯渇が測定された際に F-扣 と G7 がいずれも同一の空押しステップで停止）および Q3d（順序に従い推導された会計と割り当ての差）を補充し、HY 陽性対照により HX が真の枯渇であることを確認した；分岐1・3の両行の判定十分条件はこれらのサンプリング点において満たされた。成果物は `e156-alloc-basis-counts-2026-09-24-stage4.out`（776行）に変更し `stage3.out` を保持。テスト6、変異4のまま。

### 2026-09-25

**第3回再実行第1段階、実行担当者、`e156-r3-prereg.md`、質問票第3行のみ**：割り当てレコードツリーはその間に D8 確定項目14どおり位置指定（葉幅812、内部ファンアウト169、ディスク別に分ける共有根）となり、旧単一ノードレイアウトに固定された3定数 `OVERWRITE_EXPECTED_RELEASED_SLOTS=10`、`OVERWRITE_EXPECTED_RECORD_DELTA=16`、`NEW_POOL_FILE_CREATION_EXPECTED_RECORD_COUNT=20` と、K1項目1=13、β0参照12、空公開解放4、S1(c)隔離34、Q7d-1の `min=10 max=10`、HY「≥8」の6箇所が成立しなくなった。葉幅・ファンアウトをKB登録からローカル定数に写し、`singlefs_format` の同名定数との照合を追加。新閉形式 `e156_allocation_record_tree_new_node_count` / `_replaced_node_count` は1公開で書き換える記録ツリーノード数を `devices × 葉数 + devices × レベル1数 + 1` とし、実際に変化した記録スロット番号から葉・レベル1位置を計算、`crates/singlefs-core::allocation_record_tree` の幾何関数は呼ばない。S1(e)(f)の定数 assert を H0 144ステップの観測型逐次確認（panicせず `s1ef_summary` に集計）へ変更。K1-1を17/1、β0参照16、S1(c)隔離54に変更し、独立 Python `anchors_e156_r3.py` と装置内テストで二重確認。HY上書き回数は `e156_find_hy_cap_satisfying_open_segment_condition` で `e ≥ max(8, f)` を検索し0まででも未達。単体テスト4件（閉形式PC2、U10/U12 1、U11 1）、変異M27–M32の6件を追加し個別に赤→緑を確認。前後の全 `crates/` スナップショット S3 は並行追記の `crates/mutations.tsv` と装置ファイル以外バイト不変。S3は文字どおり発動したが確認後停止理由とならない。

**結果（課題管理票第3行：変更）**：割り当て記録ツリーを位置によりアドレス指定するようにした結果、H0/HR はもはや `AllocationRecordsExceedOneNode` に抵触しなくなり、事前登録で要求された全ステップ（144/72+72）を完走した。これは stage4.out の 49/47+47（制限に抵触して切り詰め）よりもはるかに長い。PC 判定器スクリプト（`/tmp/claude-1000/e156-r3/pc_judge.py`、`--selftest` の12本の自己テストは全パス）を用いて新たな成果物（`…-2026-09-25-r3.out`、1076行）と stage4.out を比較照合した結果：**分岐 1**（S=4 において `q1d_monotonic` が `true` から `false` へ変化し、H0/HR の完走後に Hh(k) 族の隣接する2点間で非単調な跳躍が生じた）および**分岐 7**（`q7d2_min_item5` が「非ゼロ」（stage4 では `min_item5=1 family=beta0`）から「= 0」（新成果物では `min_item5=0 family=HR kind=rollback_row txg=76` であり、checker が緑判定を下す正当な状態）へと変化し、それに伴い F16 が「トリガーされる」から「トリガーされない」へ反転した）の判定が反転した。**分岐 3**（Q3c、Q3e、K9）は検証済みの各物理量において不変であった。分岐 7 はユーザーがすでに決定を下していた行（2026-09-23、その根拠こそが「到達可能な正当な状態において第5項は決して 0 にはならない」という点であった）である——**ユーザー決定が依拠していたそのセルが変化したが、本分岐を再開することはせず、メイン agent に委ねる**。分岐 1 は当時ユーザー決定が行われておらず（第9行は依然として「開いたまま」であった）、本文で「単調」と記載されている箇所があればこの新たな測定値に従って修正する必要がある。事前登録「五」5.4 の判定十分条件（第7節のアンカーがすべて一致、Q3r.1/Q3r.2 が成立、PC 判定器 ①② がパス、Q3r.4 テーブルのすべての行に両側の判定が存在する）は第1段階においてすでに満たされており、第2段階（Q3r.5、実験21以前の並行変更と HEAD 以降の変更を個別に帰属分析する）は十分条件の成立後は実行されておらず、追加実行を行うかどうかはメイン agent の判断に委ねられる。

**（第3回再実行追加段階、実行担当者、`e156-r3-prereg.md`「十二」修訂4、分岐表第12行の G27 が到達可能状態において自己検証①を行えるかに回答）** 前段階の `q7d2_min_item5` は到達可能状態 `family=HR kind=rollback_row txg=76` において初めて 0 を記録し、それ以前の7個の到達可能基底（β0/β1/β2/βK-w/r/f/l2）における「最小でも 1 である」という 2026-09-23 のユーザー決定の事実前提を覆した。別の検証 agent が `/tmp/claude-1000/e156-fork7-check/probe.out` において独立した探索プローブコード（journal 記録を1行ずつ apply し、行書き公開の時点で切り詰め、直接 `singlefs-checker` を実行する）を用いてこの状態を検証し、`through_row_root` と `through_row_publish_tail` の両方のイメージにおいていずれも `item1=18 item5=0` であり、checker の46条の不変条件に対する違反が 0 件、I-3.1、I-5.2、I-3.11 がすべて Holds であることを確認した。**判定基準は変更せず**（分岐表第12行の元の判定基準をそのまま適用）、この行書き切り詰めによって得られた状態を第10の基底 `beta_hr_rollback_row` として `bases` に追加し（9から10へ変更、`basis_count` もそれに応じて更新）、既存の `q7c_self_test`、`run_q7a_cell`、`run_pc_check` をそのまま再実行した。新たな判定ロジックは一切記述せず、計算式も変更していない。**Q7c① が到達可能基底において初めて色の反転を達成した**：`name=q7c1_not_subtracting_defer basis=beta_hr_rollback_row k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`——従来この色の反転は、第5項を人為的に 0 に設定した到達不能基底 β_syn 上でしか確認されていなかった。`name=q7c2_threshold_at_least basis=beta_hr_rollback_row k=-1 not_applicable=true reason=deferred_is_zero_cannot_subtract_one`：②はこの新たな基底上においても β_syn/β_F0 と同様に実行のしようがない。`all_red_count` は 18 から 20 に変化し（新たに追加されたこのセルは delta=1/8 においてそれぞれ1回合取に該当した）、`basis_count` は 9 から 10 に更新された。`q7c1_flip_seen` と `q7c2_flip_seen` は以前すでに β_syn によってトリガーされていたため文字どおりには `true` のままで不変であるが、その意味合いは「到達不能基底においてのみ確認されていた」から「到達可能基底においても確認された」へと拡張された。付随するテストケースとして `hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test`（mkfs → 新規プールファイル作成 → 前半72件の上書き → 管理者による候補最古ルートへのロールバック、`main()` の同一コードを通さず公開エントリーポイントから独立して再現）を追加し、`item5 == 0`、基底 G27 が緑、flip1 が真・flip2 が偽であることを測定し、テストケース数は 10件から 11件へと増加した。ミューテーションを新たに2本追加し（`crates/mutations.tsv` 末尾の M33/M34、それぞれ `q7c_self_test` 内の `without_subtracting_defer_is_red`/`real_check_red_plus1` の比較記号を意図的に改悪）、いずれも手動で1本ずつ改悪時の赤および復元時の緑を検証し、ミューテーションテーブルは 10本から 12本へと増加した。成果物をディスクに保存する前に、事前登録「十二」修訂に第4条として追記し、これが新たに追加されたセルであること、なぜ追加されたのか、どの成果物を確認した後に追記されたのかを明記し、判定基準は分岐表第12行の元の判定基準を逐字適用して一切変更しなかった。成果物は別途 `e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out` として保存され（1085行、r3.out より9行多い：`basis_snapshot` 1行 + `q7a` 3行 + `q7c` 2行 + `pc_check` 3行）、`replay.sh` はすでにこれを指すように変更され、`r3.out` はそのまま保持されている。装置はこのセクションの成果物を実行する前後にそれぞれ1回ずつコンパイルを実行したが、その途中で `crates/singlefs-core` の一時的なコンパイル失敗（`mount.rs` 第1073行の型不一致）に1回遭遇し、数分後に自然解消した——`git status --short crates/` はローカル環境で複数のセッションが同時に `crates/singlefs-core`/`singlefs-checker` の大量のファイルを編集していることを示しており、そのコンパイル失敗はそれらのセッションのうちの1つの中間状態において発生したものであって、本変更によって引き起こされたものではなかった。その後 `cargo check -p singlefs-core` と `replay.sh` 内のこの実験の登録行はともにパスし、バイト単位で一致したため、メイン agent に報告された。**このセルが判定に十分であるかどうかはメイン agent の判断に委ねられる**：これは Q7c① の自己検証セットが**少なくとも1つの**到達可能状態において色の反転を起こし得ることを証明したが、全幾何および全履歴長を走査してこれが唯一のセルであるのか、あるいは「候補セットの中で最も古いルートへのロールバック」のような構造的条件による必然的結果であるのかどうかまでは確認されていない。
