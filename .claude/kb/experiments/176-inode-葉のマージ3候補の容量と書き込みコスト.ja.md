## E176 inodeツリーリーフ統合3候補の容量と書き込みコスト — 実行済み（2段階）

[中文](176-inode树叶合并三候选的容量与写代价.md) · [English](176-capacity-and-write-costs-of-three-inode-leaf-merge-candidates.en.md) · **日本語**

<!-- doc-lint:not-numbers M1 M2 M3 Q1 Q2 Q3 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 V15 V16 F1 F2 F3 F4 F5 F6 F7 -->

```text
このページのローカルなアーム名と記号は小文字（a1、a2、a3、a4 ……）で記す。事前登録、装置、成果物、コマンドでは対応する大文字で、全体登録の番号と同形。kb 本文で小文字にするのは区別のためだけで、成果物を読むときは大文字に対応させる。
```

事前登録 `e176-preregistration.md`（第2段完了時 sha256 `f7599076bb469dcc483f98cb0f557aca5bc6d79759fe674b4b52f150b8291fa9`）には第12節の修訂3件を含む。修訂1は M3 f=59 の陽性対照と共通窓の読み方、修訂2は突き合わせ bin をオプション機能から独立 manifest に移動、修訂3は第2段の3共通窓・3変種の陽性対照・変異 V14～V16 を第2段成果物前に追加。分岐票 `m3-3-5-forks.md` の M 行、すなわち inode 木の葉コンテナが疎になったときの回収、C116 (リーフコンテナの分割・マージ規律に失敗しうる検査がない)のマージ側、C536 (inode ツリーの「左が右から借用」がコンテナ番号条件を偽にする)に答える。

候補は M1：空になった葉のみ除去（E176 実行時の `crates/`、現在は M3）、M2：左が右を吸収するのみ（隣接2片の合計≤233行ならマージ）、M3：D8 (コアインデックス構造)確定項目6どおり左吸収と左が右から借りる方式（主点の下限 f=117、均等化して借りる）。装置 `scripts/experiment-apparatus/e176_inode_leaf_merge_capacity_and_rewrites.rs` は独立手書きモデルで `crates/` とコードを共有しない。M1 は別途 `scripts/experiment-apparatus/e176_crates_oracle/oracle.rs` で当時の `crates/singlefs-core/src/inode_tree.rs` の2純関数と逐次突き合わせ（現在 `crates/` は M3 となり突き合わせはやり直していない）。

**現状：**

- 第1段完了：22系列（W-R ランダム挿入削除 p=0.55/0.75/0.95 各5種、W-S 順序削除3点、W-A 間引き削除4点）×4アーム変種（M1/M2/M3/M3 f=59）。第6節の各行は3負荷族の主点で判定があり、第8節の第1段ノブには逆方向の点があり、分岐票 M 行の判定十分条件を満たす。
- 主窓の総判定（`research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` で `window=main` とされた七行の `name=verdict scope=row`）：Q1r は「マージで容量が倍単位に増える」（W-R p=0.55 の五つの種子すべてで R12 は 0.35 前後）。位置記法 Q2p の Δ2p/Δ3p は「反転なし」、W-R p=0.75 種2の0.1046/0.1100のみ0.10をわずかに超えるため。内容記法 Q2c 2行と Q3 2行は「無視可能」。f=59 窓の6判定も同じ（770～775行）。
- 感度：Q1r は負荷に対し不安定。W-R p=0.95、W-S 3点、W-A 3/4保持では R12≥0.90（マージなしの容量コスト小）、W-A 1/233保持では0.0711（倍単位）。Q2p 2行は挿入確率0.55対0.75の種2で不安定。
- 第 2 段は実行済みである。発動条件「第 1 段の Q2 または Q3 のいずれかの行が閾値から 0.05 未満」が成立した（Q2p の二行は W-R p = 0.75 の五つの種子で 0.0724 … 0.1100 の間にある）ため、メイン agent が実行を指定した。同じ 22 本の系列に三つの変種を追加した：M3 が下限にちょうど足りるだけ借りる `M3_just_enough`、M2 が左を先に調べる `M2_left_first`、M3 が左を先に調べる `M3_left_first`。それぞれ共通ウィンドウを開く（`just_enough` ウィンドウ = M1、M2、M3_just_enough；`left_first` ウィンドウ = M1、M2_left_first、M3_left_first；修訂 3）。総判定は、成果物 `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` で `window=just_enough` と `window=left_first` に属する十二行の `name=verdict scope=row` にある：
  - 左を先に調べるウィンドウの六行の判定は主ウィンドウと同じである。Q2p の二行は「反転なし」（W-R p = 0.75 の種子 2 では Δ2p が 0.1046、Δ3p が 0.1111）、Q2c と Q3 の四行は「無視可能」となる。
  - just_enough は Q2p 2行が「反転なし」。Q2c Δ3c と Q3 2行は主窓の「無視可能」から「反転なし」に変化。W-S 順序削除2点の Δ3c=0.5018、W-A 1/233保持は0.4568でいずれも1.00未満。3窓とも「顕著」はなく Q2p が「無視可能」になる窓もない。
  - 借用量ノブでは15組の判定が主窓「無視可能」→ just_enough「反転なし」に変わる。p=0.75 種1/4/5 の Δ3p 各1組、W-S 昇順削除2点と W-A 1/233保持で M3 関連4行ずつ。隣接確認順ノブは132組すべて同判定、うち99組は値が完全一致し閾値移動による自己検証不可。
- 停止条項は該当しなかった（両方の段で一回ずつ実行）。突き合わせ bin は 22 本の系列を M1 が拒否されるまで一歩ずつ比較し、差はゼロだった。ローカル定数 233、135 は `crates/` と照合して等しかった。第 2 段の突き合わせは変更後のソースで再ビルドし、成果物は第 1 段のものとバイト単位で同一だった。無効化条項も該当しなかった：第 1 段の乙類アンカー 26 セル、第 2 段の 29 セルはすべて正しい。第 1 段の 44 個、第 2 段の 66 個の共通ウィンドウで、三つのアームの操作ストリームのプレフィックス指紋はすべて同一だった。両段とも二回ずつ（8 スレッドと 1 スレッド）実行し、sha256 が同じだった。丙類アサーションは一つも赤にならなかった（一つでも赤なら装置はその場で panic し、終了行を出さない）。第 2 段の主ウィンドウと M1、M2、M3 の三アームの 396 行は第 1 段と一字一句同じだった。変更後のソースで第 1 段を再実行しても、リポジトリに入れた成果物とバイト単位で同じだった。

決定的モデル：2回のバイト一致は隠れた状態やスレッド数の出力影響がないことを示すのであり、統計的安定性ではない。証拠の強さは絶対値アンカー、陽性対照、突き合わせ、ミューテーション表に依存する。

### 実施内容

- **モデル：** 葉コンテナ表には葉順のコンテナ番号と各葉内の昇順レコード番号を保存。挿入は当時の `crates/` に従い、右端に空きがあれば追加、満杯なら末尾に新葉を分割しコンテナ番号を新レコード番号とする。新規後135片超なら拒否し木は不変。削除は該当葉から削り空なら除去（3アーム共通）。M2 は除去後の新たな隣接対、または残った葉の左右隣接対を調べ、合計≤233なら左が右を吸収し、収まる対がなくなるまで繰り返す。M3 は今回マージなし・葉が空でない・行数<f・右隣ありの場合、右隣から最小の ⌊(|R|−|X|)÷2⌋ 行を借り、その右隣対を M2 規則で調べる。
- **2種類の書き換え数：** 位置記法は内容が変わった葉に加え、最初に除去された位置とその右に残るすべての葉（当時の `inode_tree.rs` `remove_record_from_leaf_containers` のリスト）。内容記法は内容が変わりパブリッシュ後に残る葉のみ。1 unlink は最後のバッチのパブリッシュのみ数える。
- **逐次並走：** 各系列の4変種を各々別生成器と操作列 SHA-256 で同時に進める。共通窓は3アームのいずれかが最初に操作を拒否する直前まで。主ウィンドウの三つのアームは M1、M2、M3、f = 59 ウィンドウの三つのアームは M1、M2、M3 f = 59 である（修訂 1 の第 2 行）。パブリッシュ後の丙類検査は葉内昇順、各葉1～233行、コンテナ番号≤最小番号、葉順に番号が厳密増加、隣接キー範囲非重複。I-9.4 (コンテナ番号が最小 key を超えない) の第三文は M1/M2 でアサート、M3 では回数を数える。M2/M3 の隣接葉の合計≥234、内容記法の1操作書き換えは M1/M2 ≤1片、M3 ≤2片。
- **負荷：** W-R は splitmix64（種が初期値）。53ビット一様値 u を取り、生存集合が空または u×100 < 挿入百分率×2^53 なら挿入（正確な十進値で整数比較）、それ以外は next() mod 長さの要素を swap_remove で削除。W-S は毎回1000行挿入し最小または最大 D 件を削除。W-A は連番 A 件を挿入し登録された割合を番号昇順で削除。各アーム上限2×10⁷操作、第1段88アームはすべて上限前に拒否。
- **突き合わせ：** M1 拒否まで各操作をモデルと `write_records_into_leaf_containers` / `remove_record_from_leaf_containers` に与え、葉数、各葉のコンテナ番号・件数・最小・最大、位置記法リスト、内容記法、除去葉数、拒否操作・マスを比較。同じモデルを `#[path]` で参照し、独立 workspace の `e176_crates_oracle/Cargo.toml` から `singlefs-core`、`singlefs-format` をパス依存で読み取り専用にビルド。`scripts/experiment-apparatus/Cargo.toml` の任意機能にしないのは、任意パス依存でも Cargo が `../../crates` の manifest を読み、`scripts/mutation/mutate.sh` は `research/` のみを一時ディレクトリにコピーするため全ミューテーション表のベースラインがビルドできないから（修訂2）。
- **突き合わせの判別力（第9節）：** 草稿コピーで V1（M1 の空葉除去で右側の葉を位置リストに記録しない）を有効にすると W-S 主点の操作1233で最初の差を報告、登録どおり。両段各1回、出力はバイト一致。 読み取り値は「突き合わせ」の節に記載している。
- **第 2 段**（登録第 5 節の第 2 段の行と修訂 3）：同じモデルに `segment-two` 入口を追加し、22 本の各系列を六つのアーム変種（M1、M2、M3、M3_just_enough、M2_left_first、M3_left_first）へ入力し、三つの共通ウィンドウ（`main`、`just_enough`、`left_first`）を開く。M3_just_enough は f − |X| 行を借りる（ちょうど 117 に達するまで）。M2_left_first と M3_left_first は削除後に左隣と X の対を先に調べ、その後に X と右隣の対を調べる。下限と借りる量は主設定を使う。「借りる量」の設定は主ウィンドウと `just_enough` ウィンドウで M3 に関係する四行を比較し、「どの対を先に調べるか」の設定は主ウィンドウと `left_first` ウィンドウの六行を比較する。Q1r はこの二つの設定では判定しない（登録第 6 節が M2 の主設定に固定している）。M2_left_first が拒否された時点の生存数は読み取り値としてだけ報告する。

### 再実行

```
# リポジトリルート。モデルは crates/ に依存せず、SINGLEFS_THREAD_CAP は時間のみ影響し出力は不変
cd /home/fy5090/code/singlefs/scripts/experiment-apparatus
bash ../hosts/capped.sh 8 cargo build --release --bin e176-inode-leaf-merge-capacity-and-rewrites
bash ../hosts/run-with-memory-cap.sh 4G bash ../hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-capacity-and-rewrites segment-one
bash ../hosts/run-with-memory-cap.sh 4G bash ../hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-capacity-and-rewrites segment-two
# 登録された再実行（segment-one、segment-two）
cd ../.. && bash scripts/experiment-apparatus/replay.sh E176
# 突き合わせ（独立 manifest、crates/ のパス依存は読み取り専用）
cd scripts/experiment-apparatus
bash ../hosts/capped.sh 8 cargo build --release --manifest-path e176_crates_oracle/Cargo.toml --target-dir target
bash ../hosts/run-with-memory-cap.sh 4G bash ../hosts/capped.sh 8 ./target/release/e176-inode-leaf-merge-crates-oracle
# 単体テスト22件、変異16件
bash ../hosts/run-with-memory-cap.sh 4G bash ../hosts/capped.sh 8 cargo test --release --bin e176-inode-leaf-merge-capacity-and-rewrites
MUTATE_JOBS=4 MUTATE_CARGO_JOBS=2 MUTATE_MEMORY_MAX=4G bash ../mutation/mutate.sh e176-inode-leaf-merge-capacity-and-rewrites e176_inode_leaf_merge_capacity_and_rewrites.rs ../mutation/tables/e176_inode_leaf_merge_capacity_and_rewrites.tsv
```

第1段は8スレッド約16秒、1スレッド約23秒。第2段はそれぞれ約19秒・25秒（メモリ制限ラッパー含む）、突き合わせは8スレッド約23秒。

| ファイル | 内容 |
|---|---|
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` | 第1段：config、アンカーと陽性対照、22 本の系列 × 四つのアーム変種、総判定、感度。991行、sha256 `fe3a322978d570c3a6db180ac6e10d00a9e744c9b16b403f4e49105ef5f58dc4`。8 スレッドと 1 スレッドで二回実行した結果は、バイト単位で同一だった。第 2 段で変更したソースでもう一度実行しても、バイト単位で同一だった。 |
| `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` | 第2段：config、アンカーと陽性対照（47 セル、うち登録された答えは 29 セル）、22 本の系列 × 六つのアーム変種 × 三つのウィンドウ、総判定、二つの設定の感度。1370行、sha256 `8906bfd7d87a516abbe61c74c410dfc00d9c967b57fbad916f08f14c21e60737`。8 スレッドと 1 スレッドで二回実行した結果は、バイト単位で同一だった。 |
| `e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out` | 第1段突き合わせ：定数・3つの crates 単体テスト形状・22系列を拒否まで比較。28行、sha256 `c0a6635e1cab890f1b9dc134df1fb43e53adc2ba2c993f59bbaef981f8c69ae3`、manifest 変更前の草稿実行と一致。 |
| `e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out` | 第2段ソースから再ビルドした突き合わせ。第1段とバイト一致、同 sha256。 |
| `e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out` | 第1段突き合わせの判別力：草稿コピーで V1 を有効化し判決を意図的に false とする。 |
| `e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1-2026-10-08.out` | 第 2 段の突き合わせの判別力（第 2 段のソースで V1 を有効にした）。第 1 段の成果物とバイト単位で同一、sha256 `29e6ab1ed77318939f8d377b7c90728c7a4028a24771d0e8aad080c71f909bb3`。 |
| `e176-inode-leaf-merge-capacity-and-rewrites-unit-tests.out` | 第1段単体テスト14件。 |
| `e176-inode-leaf-merge-capacity-and-rewrites-unit-tests-2026-10-08.out` | 第2段単体テスト22件。 |
| `e176-inode-leaf-merge-capacity-and-rewrites-mutations.out` | 第1段のミューテーション表13件全体、stderr 先頭行を先に記載。 |
| `e176-inode-leaf-merge-capacity-and-rewrites-mutations-2026-10-08.out` | 第2段ソース・ミューテーション表16件全体、stderr 先頭行を先に記載。 |

### 判決（成果物の全行）

`grep -n 'name=verdict'` の全判決行：

```
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:33:E7RESULT name=verdict scope=positive_controls anchors_registered=26 anchors_matching=26 all_registered_anchors_match=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:760:E7RESULT name=verdict scope=window_prefixes windows=44 windows_with_equal_prefixes=44 all_window_prefixes_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:761:E7RESULT name=verdict scope=f6_m1_reached points=22 m1_not_reached_points=0 m1_reached_on_every_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:762:E7RESULT name=verdict scope=f5_m3_borrows_on_main_points main_point_m3_borrows=5930 m3_borrowed_on_some_main_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:763:E7RESULT name=verdict scope=row row=Q1r window=main points=22 judged_points=22 verdict=merge_buys_double_capacity
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:764:E7RESULT name=verdict scope=row row=Q2p_delta2 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:765:E7RESULT name=verdict scope=row row=Q2p_delta3 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:766:E7RESULT name=verdict scope=row row=Q2c_delta2 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:767:E7RESULT name=verdict scope=row row=Q2c_delta3 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:768:E7RESULT name=verdict scope=row row=Q3p_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:769:E7RESULT name=verdict scope=row row=Q3c_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:770:E7RESULT name=verdict scope=row row=Q2p_delta2 window=floor59 points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:771:E7RESULT name=verdict scope=row row=Q2p_delta3 window=floor59 points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:772:E7RESULT name=verdict scope=row row=Q2c_delta2 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:773:E7RESULT name=verdict scope=row row=Q2c_delta3 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:774:E7RESULT name=verdict scope=row row=Q3p_delta32 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:775:E7RESULT name=verdict scope=row row=Q3c_delta32 window=floor59 points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out:990:E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=200 unstable_pairs=14 moved_to_unstable=162 equal_values_not_separable=38 every_separable_same_pair_moved_to_unstable=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
```

最初の2成果物には false / not_run がなく、違反・不一致・曖昧・失敗の整数は0。3つ目は判別力の自己検証で V1 を有効化し差異を出すため `all_equal=false` と `crates_unit_test_shapes_equal=false` は想定内。`verdict=no_flip` は失敗ではなく第6節の「その他⇒反転なし」分類。

第 2 段の実行（突き合わせ、判別力と第 2 段の成果物）について、`grep -n 'name=verdict'` が列挙したすべての判決行：

```
research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1-2026-10-08.out:27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:49:E7RESULT name=verdict scope=positive_controls anchors_registered=29 anchors_matching=29 all_registered_anchors_match=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1128:E7RESULT name=verdict scope=window_prefixes windows=66 windows_with_equal_prefixes=66 all_window_prefixes_equal=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1129:E7RESULT name=verdict scope=f6_m1_reached points=22 m1_not_reached_points=0 m1_reached_on_every_point=true
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1130:E7RESULT name=verdict scope=row row=Q1r window=main points=22 judged_points=22 verdict=merge_buys_double_capacity
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1131:E7RESULT name=verdict scope=row row=Q2p_delta2 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1132:E7RESULT name=verdict scope=row row=Q2p_delta3 window=main points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1133:E7RESULT name=verdict scope=row row=Q2c_delta2 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1134:E7RESULT name=verdict scope=row row=Q2c_delta3 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1135:E7RESULT name=verdict scope=row row=Q3p_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1136:E7RESULT name=verdict scope=row row=Q3c_delta32 window=main points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1137:E7RESULT name=verdict scope=row row=Q2p_delta2 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1138:E7RESULT name=verdict scope=row row=Q2p_delta3 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1139:E7RESULT name=verdict scope=row row=Q2c_delta2 window=just_enough points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1140:E7RESULT name=verdict scope=row row=Q2c_delta3 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1141:E7RESULT name=verdict scope=row row=Q3p_delta32 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1142:E7RESULT name=verdict scope=row row=Q3c_delta32 window=just_enough points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1143:E7RESULT name=verdict scope=row row=Q2p_delta2 window=left_first points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1144:E7RESULT name=verdict scope=row row=Q2p_delta3 window=left_first points=22 judged_points=22 verdict=no_flip
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1145:E7RESULT name=verdict scope=row row=Q2c_delta2 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1146:E7RESULT name=verdict scope=row row=Q2c_delta3 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1147:E7RESULT name=verdict scope=row row=Q3p_delta32 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1148:E7RESULT name=verdict scope=row row=Q3c_delta32 window=left_first points=22 judged_points=22 verdict=negligible
research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out:1369:E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=205 unstable_pairs=15 moved_to_unstable=74 equal_values_not_separable=131 every_separable_same_pair_moved_to_unstable=true
```

第2段突き合わせは false なし、判別力の2つの false は第1段と同じ理由で想定内。成果物には false / not_run がなく `m1_not_reached_points=0`。主窓1130～1136行は第1段763～769行と同判定。`equal_values_not_separable=131` は「不合格」を示すものではない。閾値を移す自己検証を実施できない対の数を表す。二つの点の値がビット単位で等しいため、閾値を二点の間に移せない。登録第8節に報告規定がないため他と合算せず別記。

### アンカーと陽性対照（第1段の全31 `name=anchor` 行）

登録第7節甲・乙の全マスと修訂1の f=59 変種（a2～a6 は報告のみ `matches=unregistered`）。135上限なしの M1 で10⁶件を満たし9割削除する空間増幅マスは29行 `ratio=9.98`、D8 項目6末尾の9.98倍と一致し F3 非該当。F1 対照も合格：a2 の M2 生存数31455 > M1 の367、a3 で M3 借用1回、a5 で M2 マージ1回。

```
E7RESULT name=anchor cell=A1 arm=M1 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M2 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3_floor59 refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A2 arm=M1 refused_insert=31688 live=367 containers=135 merges=0 borrows=0 matches=true
E7RESULT name=anchor cell=A2 arm=M2 refused_insert=62776 live=31455 containers=135 merges=134 borrows=0 matches=true
E7RESULT name=anchor cell=A2 arm=M3 refused_insert=62776 live=31455 containers=135 merges=134 borrows=918 matches=true
E7RESULT name=anchor cell=A2 arm=M3_floor59 refused_insert=62776 live=31455 containers=135 merges=134 borrows=58 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M1 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=true
E7RESULT name=anchor cell=A3 arm=M2 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=true
E7RESULT name=anchor cell=A3 arm=M3 content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=174/176 matches=true
E7RESULT name=anchor cell=A3_boundary arm=M3 leaf0_largest=291 leaf1_smallest=292 matches=true
E7RESULT name=anchor cell=A3 arm=M3_floor59 content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M1 content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M2 content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M3 content=240 position=242 containers=3 merges=1 borrows=7 third_sentence_false=0 sizes=233/233/1 matches=true
E7RESULT name=anchor cell=A4 arm=M3_floor59 content=234 position=236 containers=3 merges=1 borrows=1 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M1 content=234 position=234 containers=5 merges=0 borrows=0 third_sentence_false=0 sizes=233/116/116/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M2 content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M3 content=241 position=243 containers=4 merges=1 borrows=7 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=A5 arm=M3_floor59 content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M1 content=233 position=233 containers=2 merges=0 borrows=0 third_sentence_false=0 sizes=116/117 matches=true
E7RESULT name=anchor cell=A6 arm=M2 content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=true
E7RESULT name=anchor cell=A6 arm=M3 content=240 position=240 containers=1 merges=1 borrows=7 third_sentence_false=0 sizes=233 matches=true
E7RESULT name=anchor cell=A6 arm=M3_floor59 content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M3_just_enough content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=117/233 matches=true
E7RESULT name=anchor cell=A5 arm=M2_left_first content=234 position=236 containers=4 merges=1 borrows=0 third_sentence_false=0 sizes=233/232/233/1 matches=true
E7RESULT name=anchor cell=C1 arm=M1_unlimited leaves=4292 live=100000 ideal=430 ratio=9.98 matches=true
E7RESULT name=anchor cell=crates_insert_500 arm=M1 sizes=233/233/34 container_numbers=1/234/467 matches=true
E7RESULT name=anchor cell=crates_insert_31456 arm=M1 refused_inode=31456 refused_containers=136 capacity=135 matches=true
E7RESULT name=anchor cell=crates_drop_middle arm=M1 position_rewritten=[1] dropped=1 matches=true
```

### 測定：Q1（拒否時生存数と R12 = L_M1 ÷ L_M2）

読み方1：あるアームが初めて挿入を拒否したとき、最後に受理された版のレコード数。88アームすべて拒否に到達（761行 `m1_not_reached_points=0`、M2/M3 も `stop=refused`）し、上界のみの点はない。列は系列、L_M1、L_M2、L_M3（Q1b、参考）、L_M3 f=59、R12、判定。リポジトリルートからのコマンドと原出力：

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} live[v["sequence"],v["arm"]]=v["live_at_stop"]} /name=point_verdict/ && / window=main / && / row=Q1r /{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} print v["sequence"], live[v["sequence"],"M1"], live[v["sequence"],"M2"], live[v["sequence"],"M3"], live[v["sequence"],"M3_floor59"], v["value"], v["verdict"]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
```

```
wr_p055_seed1 8070 22452 22116 22452 0.3594 merge_buys_double_capacity
wr_p055_seed2 8177 22422 21909 21995 0.3647 merge_buys_double_capacity
wr_p055_seed3 7978 22190 21867 22190 0.3595 merge_buys_double_capacity
wr_p055_seed4 7933 22823 21820 22771 0.3476 merge_buys_double_capacity
wr_p055_seed5 8083 22396 22260 21912 0.3609 merge_buys_double_capacity
wr_p075_seed1 21360 24167 23551 24167 0.8838 no_flip
wr_p075_seed2 21173 24427 24264 24427 0.8668 no_flip
wr_p075_seed3 21378 24225 24225 24225 0.8825 no_flip
wr_p075_seed4 20994 24299 23980 24299 0.8640 no_flip
wr_p075_seed5 21157 24325 24163 24325 0.8698 no_flip
wr_p095_seed1 29865 29865 29865 29865 1.0000 no_merge_capacity_cost_small
wr_p095_seed2 29810 29811 29811 29811 1.0000 no_merge_capacity_cost_small
wr_p095_seed3 29802 29803 29803 29803 1.0000 no_merge_capacity_cost_small
wr_p095_seed4 29815 29815 29815 29815 1.0000 no_merge_capacity_cost_small
wr_p095_seed5 29821 29821 29821 29821 1.0000 no_merge_capacity_cost_small
ws_a1000_d500_smallest_first 31245 31245 31245 31245 1.0000 no_merge_capacity_cost_small
ws_a1000_d950_smallest_first 31319 31319 31284 31278 1.0000 no_merge_capacity_cost_small
ws_a1000_d500_largest_first 31455 31455 31455 31455 1.0000 no_merge_capacity_cost_small
wa_keep1of2_a1000 17132 27054 25464 27054 0.6333 no_flip
wa_keep1of2_a2330 16773 31455 31455 31455 0.5332 no_flip
wa_keep1of233_a2330 2236 31455 31081 31081 0.0711 merge_buys_double_capacity
wa_keep3of4_a1000 24606 25478 25478 25478 0.9658 no_merge_capacity_cost_small
```

Q1r 総判定は「マージで容量が倍単位に増える」。W-R p=0.55 の5種は0.3476～0.3647で全て0.50未満。一方 M2/M3 に不利な側も表にある：W-S 3点、W-R p=0.95 の5種、W-A 3/4保持は R12≥0.90（マージなしの容量コスト小）。W-S は順序削除で丸ごと空になった葉を M1 も除去するため M1/M2 の拒否時生存数は各点で同じ。W-A 1/2保持の主点は0.6333「反転なし」。M3 の拒否時生存数は22系列中 M2 より少ない12、同じ10、多い0（W-R p=0.55 は21820～22260対22190～22823）で、この負荷群では借用による容量増はない。数え方と出力：

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} live[v["sequence"],v["arm"]]=v["live_at_stop"]; seq[v["sequence"]]=1} END{less=0; equal=0; more=0; for(s in seq){ if (live[s,"M3"]+0 < live[s,"M2"]+0) less++; else if (live[s,"M3"]+0 == live[s,"M2"]+0) equal++; else more++ } print "m3_less_than_m2=" less, "equal=" equal, "m3_more_than_m2=" more}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
m3_less_than_m2=12 equal=10 m3_more_than_m2=0
```

### 測定：Q2/Q3（主窓の unlink 当たり平均書き換え片数差）

Δ =（アーム甲の合計−乙の合計）÷窓内最後バッチ数。44窓の最小は1643回（`name=window` の `unlinks` 最小）で標本不足なし。負値は M1 より書き換えが少ないことを表す。コマンドと原出力：

```
awk '/name=point_verdict/ && / window=main / && !/ row=Q1r /{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} out[v["sequence"]]=out[v["sequence"]] " " v["row"] "=" v["value"]; if(!(v["sequence"] in seen)){seen[v["sequence"]]=1; order[++n]=v["sequence"]}} END{for(i=1;i<=n;i++) print order[i] out[order[i]]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out
```

```
wr_p055_seed1 Q2p_delta2=-0.0260 Q2p_delta3=-0.0157 Q2c_delta2=0.0012 Q2c_delta3=0.0111 Q3p_delta32=0.0103 Q3c_delta32=0.0099
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=-0.0301 Q2c_delta2=0.0013 Q2c_delta3=0.0115 Q3p_delta32=0.0092 Q3c_delta32=0.0102
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=-0.0131 Q2c_delta2=0.0012 Q2c_delta3=0.0111 Q3p_delta32=0.0099 Q3c_delta32=0.0099
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=-0.0076 Q2c_delta2=0.0013 Q2c_delta3=0.0118 Q3p_delta32=0.0097 Q3c_delta32=0.0105
wr_p055_seed5 Q2p_delta2=-0.0219 Q2p_delta3=-0.0128 Q2c_delta2=0.0013 Q2c_delta3=0.0108 Q3p_delta32=0.0091 Q3c_delta32=0.0095
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.0724 Q2c_delta2=0.0000 Q2c_delta3=0.0047 Q3p_delta32=-0.0158 Q3c_delta32=0.0047
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1100 Q2c_delta2=0.0000 Q2c_delta3=0.0038 Q3p_delta32=0.0054 Q3c_delta32=0.0038
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0822 Q2c_delta2=0.0000 Q2c_delta3=0.0040 Q3p_delta32=0.0026 Q3c_delta32=0.0040
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.0786 Q2c_delta2=0.0000 Q2c_delta3=0.0041 Q3p_delta32=-0.0199 Q3c_delta32=0.0041
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.0936 Q2c_delta2=0.0000 Q2c_delta3=0.0046 Q3p_delta32=0.0001 Q3c_delta32=0.0046
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0299 Q2c_delta2=0.0000 Q2c_delta3=0.0342 Q3p_delta32=0.0299 Q3c_delta32=0.0342
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0300 Q2c_delta2=0.0000 Q2c_delta3=0.0343 Q3p_delta32=0.0300 Q3c_delta32=0.0343
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0055 Q2c_delta2=0.0000 Q2c_delta3=0.0016 Q3p_delta32=0.0004 Q3c_delta32=0.0016
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.0457 Q2c_delta2=0.0004 Q2c_delta3=0.0275 Q3p_delta32=0.0265 Q3c_delta32=0.0271
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

2記法の結論は分けて書く：

- **位置記法（実行時の `crates/`）：** Δ2p/Δ3p 総判定は「反転なし」。主点では両行とも≤0.10、p=0.55 の5種は−0.0393～−0.0076。M1 が空葉を除去すると右側の残葉が左へ移動し全て書き換え、1回最大133片。M2 は早くマージし除去時の右側が少ない。「マージは無視可能」に不利な側は p=0.75 の Δ2p=0.0796～0.1046、Δ3p=0.0724～0.1100、種2は両方0.10をわずかに超える。この族で M1 は右端葉しか除去せず `position_largest=1`、M2/M3 のマージは右側を移動し1回最大90～105片。
- **内容記法**：Δ2c と Δ3c の総判定は「無視可能」である。最も大きい値は W-S の二つの点における Δ3c（0.0342、0.0343）と、W-A で 1/233 を残す場合の 0.0275。Δ2c の最大値は 0.0013 である。
- **Q3（M3−M2）：** 両記法とも「無視可能」、最大0.0343（W-S 順序削除950、内容）。主点で借用は実際に発生（F5 `main_point_m3_borrows=5930`）し、unlink 当たり平均は0.035片未満。

主点窓の書き換え分布の原行。ヒストグラム5区分は0、1、2、3～9、10～134片の回数：

```
39:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M1 unlinks=35448 content_sum=35406 position_sum=38633 content_mean=0.9988 position_mean=1.0898 content_largest=1 position_largest=127 content_histogram=42/35406/0/0/0 position_histogram=1/35406/0/0/41
40:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M2 unlinks=35448 content_sum=35447 position_sum=37712 content_mean=1.0000 position_mean=1.0639 content_largest=1 position_largest=46 content_histogram=1/35447/0/0/0 position_histogram=1/35320/1/25/101
41:E7RESULT name=window_rewrites sequence=wr_p055_seed1 window=main arm=M3 unlinks=35448 content_sum=35798 position_sum=38076 content_mean=1.0099 position_mean=1.0741 content_largest=2 position_largest=48 content_histogram=1/35096/351/0/0 position_histogram=1/34971/351/24/101
72:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M1 unlinks=37793 content_sum=37743 position_sum=41605 content_mean=0.9987 position_mean=1.1009 content_largest=1 position_largest=130 content_histogram=50/37743/0/0/0 position_histogram=1/37743/0/2/47
73:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M2 unlinks=37793 content_sum=37792 position_sum=40119 content_mean=1.0000 position_mean=1.0615 content_largest=1 position_largest=45 content_histogram=1/37792/0/0/0 position_histogram=1/37657/3/27/105
74:E7RESULT name=window_rewrites sequence=wr_p055_seed2 window=main arm=M3 unlinks=37793 content_sum=38176 position_sum=40467 content_mean=1.0101 position_mean=1.0708 content_largest=2 position_largest=45 content_histogram=1/37408/384/0/0 position_histogram=1/37278/385/28/101
105:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M1 unlinks=36583 content_sum=36535 position_sum=39784 content_mean=0.9987 position_mean=1.0875 content_largest=1 position_largest=133 content_histogram=48/36535/0/0/0 position_histogram=3/36535/0/0/45
106:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M2 unlinks=36583 content_sum=36580 position_sum=38942 content_mean=0.9999 position_mean=1.0645 content_largest=1 position_largest=48 content_histogram=3/36580/0/0/0 position_histogram=3/36447/2/28/103
107:E7RESULT name=window_rewrites sequence=wr_p055_seed3 window=main arm=M3 unlinks=36583 content_sum=36942 position_sum=39303 content_mean=1.0098 position_mean=1.0744 content_largest=2 position_largest=49 content_histogram=3/36218/362/0/0 position_histogram=3/36088/363/27/102
138:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M1 unlinks=37574 content_sum=37524 position_sum=40519 content_mean=0.9987 position_mean=1.0784 content_largest=1 position_largest=131 content_histogram=50/37524/0/0/0 position_histogram=2/37524/0/1/47
139:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M2 unlinks=37574 content_sum=37572 position_sum=39871 content_mean=0.9999 position_mean=1.0611 content_largest=1 position_largest=46 content_histogram=2/37572/0/0/0 position_histogram=2/37436/3/31/102
140:E7RESULT name=window_rewrites sequence=wr_p055_seed4 window=main arm=M3 unlinks=37574 content_sum=37966 position_sum=40235 content_mean=1.0104 position_mean=1.0708 content_largest=2 position_largest=44 content_histogram=2/37178/394/0/0 position_histogram=2/37045/395/32/100
171:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M1 unlinks=37279 content_sum=37228 position_sum=40474 content_mean=0.9986 position_mean=1.0857 content_largest=1 position_largest=127 content_histogram=51/37228/0/0/0 position_histogram=4/37228/0/1/46
172:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M2 unlinks=37279 content_sum=37275 position_sum=39658 content_mean=0.9999 position_mean=1.0638 content_largest=1 position_largest=47 content_histogram=4/37275/0/0/0 position_histogram=4/37139/3/28/105
173:E7RESULT name=window_rewrites sequence=wr_p055_seed5 window=main arm=M3 unlinks=37279 content_sum=37630 position_sum=39997 content_mean=1.0094 position_mean=1.0729 content_largest=2 position_largest=47 content_histogram=4/36920/355/0/0 position_histogram=4/36788/356/27/104
534:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M1 unlinks=30500 content_sum=30370 position_sum=39229 content_mean=0.9957 position_mean=1.2862 content_largest=1 position_largest=133 content_histogram=130/30370/0/0/0 position_histogram=0/30370/0/6/124
535:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M2 unlinks=30500 content_sum=30370 position_sum=39229 content_mean=0.9957 position_mean=1.2862 content_largest=1 position_largest=133 content_histogram=130/30370/0/0/0 position_histogram=0/30370/0/6/124
536:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=main arm=M3 unlinks=30500 content_sum=31413 position_sum=40142 content_mean=1.0299 position_mean=1.3161 content_largest=2 position_largest=133 content_histogram=0/29587/913/0/0 position_histogram=0/29457/913/6/124
633:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M1 unlinks=16500 content_sum=16500 position_sum=16500 content_mean=1.0000 position_mean=1.0000 content_largest=1 position_largest=1 content_histogram=0/16500/0/0/0 position_histogram=0/16500/0/0/0
634:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M2 unlinks=16500 content_sum=16500 position_sum=16583 content_mean=1.0000 position_mean=1.0050 content_largest=1 position_largest=4 content_histogram=0/16500/0/0/0 position_histogram=0/16467/0/33/0
635:E7RESULT name=window_rewrites sequence=wa_keep1of2_a1000 window=main arm=M3 unlinks=16500 content_sum=16526 position_sum=16590 content_mean=1.0016 position_mean=1.0055 content_largest=2 position_largest=4 content_histogram=0/16474/26/0/0 position_histogram=0/16441/38/21/0
```

### 軌跡（第8節）

判定述語が使うのは葉コンテナ片数（135超で拒否）。各アームはピーク片数、135片の公開回数、終了時片数、最小充填率と拒否前最後の版の充填率、M3 では I-9.4 第3文が偽の隣接対のピーク・正となる公開回数・終了時値を報告する。最小充填率は88アームすべて0.0043（1÷233、最初の inode 作成時）なので初期値しか示さず、疎になった状態は拒否前最後の列を見る。主点の軌跡行：

```
34:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=78967 operations_accepted=78966 live_at_stop=8070 live_over_31455=0.2566 containers_peak=135 publishes_at_135=651 containers_final=135 fill_smallest=0.0043 fill_final=0.2566 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=fe4304e70876af50f0fa91b8f2c75791c72d36710fdc199dddfffe13d732a1f6
35:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=230361 operations_accepted=230360 live_at_stop=22452 live_over_31455=0.7138 containers_peak=135 publishes_at_135=961 containers_final=135 fill_smallest=0.0043 fill_final=0.7138 merges=429 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=d71ba0162292de7d49f990dd26d0a8a0ca1a8264ebf95e4f3c6f7f38339e3711
36:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=226987 operations_accepted=226986 live_at_stop=22116 live_over_31455=0.7031 containers_peak=135 publishes_at_135=774 containers_final=135 fill_smallest=0.0043 fill_final=0.7031 merges=427 borrows=1020 i94_third_false_peak=45 i94_third_false_positive_publishes=224603 i94_third_false_final=41 consumed_prefix_sha256=068dd9dcc4c0fcf20ceeeba91e5e38383cc4e6cb4a34b17db8b8d9e8c6bfb60b
37:E7RESULT name=arm_run sequence=wr_p055_seed1 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=230361 operations_accepted=230360 live_at_stop=22452 live_over_31455=0.7138 containers_peak=135 publishes_at_135=961 containers_final=135 fill_smallest=0.0043 fill_final=0.7138 merges=429 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=d71ba0162292de7d49f990dd26d0a8a0ca1a8264ebf95e4f3c6f7f38339e3711
67:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=83764 operations_accepted=83763 live_at_stop=8177 live_over_31455=0.2600 containers_peak=135 publishes_at_135=414 containers_final=135 fill_smallest=0.0043 fill_final=0.2600 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=5afa3f6043a5318a7b34d32dfb6f6026d7cef5496eb8025df9220a2318f62fb7
68:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=231163 operations_accepted=231162 live_at_stop=22422 live_over_31455=0.7128 containers_peak=135 publishes_at_135=869 containers_final=135 fill_smallest=0.0043 fill_final=0.7128 merges=422 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=583feaff08554eee9716e557803ce97d7cf45dcb040d06b8c32d311c5bbd29c3
69:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=226290 operations_accepted=226289 live_at_stop=21909 live_over_31455=0.6965 containers_peak=135 publishes_at_135=609 containers_final=135 fill_smallest=0.0043 fill_final=0.6965 merges=430 borrows=1029 i94_third_false_peak=44 i94_third_false_positive_publishes=221873 i94_third_false_final=40 consumed_prefix_sha256=f05ae2f3871ea2821bd23aa97983f8503deb14b6ba96a2e97415ba0a6785dc09
70:E7RESULT name=arm_run sequence=wr_p055_seed2 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=227128 operations_accepted=227127 live_at_stop=21995 live_over_31455=0.6993 containers_peak=135 publishes_at_135=418 containers_final=135 fill_smallest=0.0043 fill_final=0.6993 merges=434 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=481 i94_third_false_final=0 consumed_prefix_sha256=828525d844500cf89c5b0d5c924022bad7ae3605871882afed2fd3bff2a23f01
100:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=81145 operations_accepted=81144 live_at_stop=7978 live_over_31455=0.2536 containers_peak=135 publishes_at_135=860 containers_final=135 fill_smallest=0.0043 fill_final=0.2536 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=fa40f6fa0318543033abd9c7fb1815326ad7f6e44f60abb0f39d5b89e6e4c056
101:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=226565 operations_accepted=226564 live_at_stop=22190 live_over_31455=0.7055 containers_peak=135 publishes_at_135=517 containers_final=135 fill_smallest=0.0043 fill_final=0.7055 merges=406 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=761e85f85e52f879b3ddfac416f02a96e4f142cbea8fb99e92ae78fcc6aed648
102:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=223438 operations_accepted=223437 live_at_stop=21867 live_over_31455=0.6952 containers_peak=135 publishes_at_135=473 containers_final=135 fill_smallest=0.0043 fill_final=0.6952 merges=406 borrows=950 i94_third_false_peak=39 i94_third_false_positive_publishes=219845 i94_third_false_final=36 consumed_prefix_sha256=79d49208dbd51d8c5a97528d345a18f1dc75948794454ee27c3ffb98820663bf
103:E7RESULT name=arm_run sequence=wr_p055_seed3 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=226565 operations_accepted=226564 live_at_stop=22190 live_over_31455=0.7055 containers_peak=135 publishes_at_135=517 containers_final=135 fill_smallest=0.0043 fill_final=0.7055 merges=406 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=761e85f85e52f879b3ddfac416f02a96e4f142cbea8fb99e92ae78fcc6aed648
133:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=83082 operations_accepted=83081 live_at_stop=7933 live_over_31455=0.2522 containers_peak=135 publishes_at_135=672 containers_final=135 fill_smallest=0.0043 fill_final=0.2522 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=b1cb5e13c8eb2872837027dd4e6efced911dbd51ee052f0db59e4d2af111a6c3
134:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=229232 operations_accepted=229231 live_at_stop=22823 live_over_31455=0.7256 containers_peak=135 publishes_at_135=936 containers_final=135 fill_smallest=0.0043 fill_final=0.7256 merges=408 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=799af22c99b01dad0a9069d884d81e6fff284b13b3e22814ccb768912e660b65
135:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=220027 operations_accepted=220026 live_at_stop=21820 live_over_31455=0.6937 containers_peak=135 publishes_at_135=453 containers_final=135 fill_smallest=0.0043 fill_final=0.6937 merges=395 borrows=981 i94_third_false_peak=43 i94_third_false_positive_publishes=215205 i94_third_false_final=42 consumed_prefix_sha256=b07328935fa974782873a50ce2936b459454b7acc1da713b58cfa434a8af04ea
136:E7RESULT name=arm_run sequence=wr_p055_seed4 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=228432 operations_accepted=228431 live_at_stop=22771 live_over_31455=0.7239 containers_peak=135 publishes_at_135=402 containers_final=135 fill_smallest=0.0043 fill_final=0.7239 merges=408 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=347 i94_third_false_final=0 consumed_prefix_sha256=d90c6c10aeb892b21485fa66808ae4f65c172cc475802204e88d7e881205c0cb
166:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M1 stop=refused refused_operation=82642 operations_accepted=82641 live_at_stop=8083 live_over_31455=0.2570 containers_peak=135 publishes_at_135=411 containers_final=135 fill_smallest=0.0043 fill_final=0.2570 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0a565b850da4ed7e69f6217e4518839373fad02f0a377d28380a93eed1138447
167:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M2 stop=refused refused_operation=224797 operations_accepted=224796 live_at_stop=22396 live_over_31455=0.7120 containers_peak=135 publishes_at_135=408 containers_final=135 fill_smallest=0.0043 fill_final=0.7120 merges=407 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=16eb9a1342ae8c1dada946306ff2bd89c0746725dbffec13a982ac78576fcf56
168:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M3 stop=refused refused_operation=223717 operations_accepted=223716 live_at_stop=22260 live_over_31455=0.7077 containers_peak=135 publishes_at_135=696 containers_final=135 fill_smallest=0.0043 fill_final=0.7077 merges=399 borrows=1001 i94_third_false_peak=42 i94_third_false_positive_publishes=220737 i94_third_false_final=38 consumed_prefix_sha256=dd7c20e7d9320424e738ef15abe21786782c370a3fee1dce44f36e49d6b01301
169:E7RESULT name=arm_run sequence=wr_p055_seed5 family=W-R main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=220315 operations_accepted=220314 live_at_stop=21912 live_over_31455=0.6966 containers_peak=135 publishes_at_135=411 containers_final=135 fill_smallest=0.0043 fill_final=0.6966 merges=388 borrows=1 i94_third_false_peak=1 i94_third_false_positive_publishes=574 i94_third_false_final=0 consumed_prefix_sha256=beba41c2b4e574d2fb2cb0b69d3bec5bbe75bb30d0a377f771f132484466afe1
529:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M1 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
530:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M2 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
531:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M3 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=130 borrows=913 i94_third_false_peak=1 i94_third_false_positive_publishes=44919 i94_third_false_final=1 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
532:E7RESULT name=arm_run sequence=ws_a1000_d500_smallest_first family=W-S main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=92246 operations_accepted=92245 live_at_stop=31245 live_over_31455=0.9933 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.9933 merges=130 borrows=131 i94_third_false_peak=1 i94_third_false_positive_publishes=22321 i94_third_false_final=1 consumed_prefix_sha256=c5cd7f135997e487211265f76c794ce3c285a3e9061b595aefcda8ca42f2d317
628:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M1 stop=refused refused_operation=50133 operations_accepted=50132 live_at_stop=17132 live_over_31455=0.5447 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.5447 merges=0 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=254c551cd054829735b28c412d02a379d6a0645758e962936459248375911f66
629:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M2 stop=refused refused_operation=80055 operations_accepted=80054 live_at_stop=27054 live_over_31455=0.8601 containers_peak=135 publishes_at_135=528 containers_final=135 fill_smallest=0.0043 fill_final=0.8601 merges=106 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0146ed25b25641ce21fcb425ea93c20cf1918917204687cd3a9548be1fdfaf20
630:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M3 stop=refused refused_operation=74465 operations_accepted=74464 live_at_stop=25464 live_over_31455=0.8095 containers_peak=135 publishes_at_135=233 containers_final=135 fill_smallest=0.0043 fill_final=0.8095 merges=81 borrows=36 i94_third_false_peak=19 i94_third_false_positive_publishes=71807 i94_third_false_final=18 consumed_prefix_sha256=2e2bb7b45ee7a2dd79d0d34d754b9252faa3222ad1addb3a58b0f9d714d352e9
631:E7RESULT name=arm_run sequence=wa_keep1of2_a1000 family=W-A main_sampling_point=true arm=M3_floor59 stop=refused refused_operation=80055 operations_accepted=80054 live_at_stop=27054 live_over_31455=0.8601 containers_peak=135 publishes_at_135=528 containers_final=135 fill_smallest=0.0043 fill_final=0.8601 merges=106 borrows=0 i94_third_false_peak=0 i94_third_false_positive_publishes=0 i94_third_false_final=0 consumed_prefix_sha256=0146ed25b25641ce21fcb425ea93c20cf1918917204687cd3a9548be1fdfaf20
```

`fill_smallest` の集計は `88 fill_smallest=0.0043`。

累積書き換え差 S(t)=Σ(アーム−M1)、t は主窓内の最後バッチ。W-R p=0.55 の位置記法では終了値が負だがピークは正：先に正へ増加し、M1 の葉除去時の大きな書き換え（最大133片）で負になる。終了値だけでは先行区間を見落とす。除去が窓内のどこに現れたかは逐次記録していない。

```
42:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M2-M1 notation=position peak=122 positive_single_differences=127 final=-921
43:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M2-M1 notation=content peak=41 positive_single_differences=41 final=41
44:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M3-M1 notation=position peak=297 positive_single_differences=476 final=-557
45:E7RESULT name=cumulative_difference sequence=wr_p055_seed1 window=main pair=M3-M1 notation=content peak=392 positive_single_differences=392 final=392
75:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M2-M1 notation=position peak=192 positive_single_differences=135 final=-1486
76:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M2-M1 notation=content peak=49 positive_single_differences=49 final=49
77:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M3-M1 notation=position peak=450 positive_single_differences=514 final=-1138
78:E7RESULT name=cumulative_difference sequence=wr_p055_seed2 window=main pair=M3-M1 notation=content peak=433 positive_single_differences=433 final=433
108:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M2-M1 notation=position peak=56 positive_single_differences=133 final=-842
109:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M2-M1 notation=content peak=45 positive_single_differences=45 final=45
110:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M3-M1 notation=position peak=163 positive_single_differences=492 final=-481
111:E7RESULT name=cumulative_difference sequence=wr_p055_seed3 window=main pair=M3-M1 notation=content peak=407 positive_single_differences=407 final=407
141:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M2-M1 notation=position peak=25 positive_single_differences=135 final=-648
142:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M2-M1 notation=content peak=48 positive_single_differences=48 final=48
143:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M3-M1 notation=position peak=168 positive_single_differences=527 final=-284
144:E7RESULT name=cumulative_difference sequence=wr_p055_seed4 window=main pair=M3-M1 notation=content peak=442 positive_single_differences=442 final=442
174:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M2-M1 notation=position peak=39 positive_single_differences=136 final=-816
175:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M2-M1 notation=content peak=47 positive_single_differences=47 final=47
176:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M3-M1 notation=position peak=198 positive_single_differences=486 final=-477
177:E7RESULT name=cumulative_difference sequence=wr_p055_seed5 window=main pair=M3-M1 notation=content peak=402 positive_single_differences=401 final=402
537:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M2-M1 notation=position peak=0 positive_single_differences=0 final=0
538:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M2-M1 notation=content peak=0 positive_single_differences=0 final=0
539:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M3-M1 notation=position peak=913 positive_single_differences=913 final=913
540:E7RESULT name=cumulative_difference sequence=ws_a1000_d500_smallest_first window=main pair=M3-M1 notation=content peak=1043 positive_single_differences=1043 final=1043
636:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M2-M1 notation=position peak=83 positive_single_differences=33 final=83
637:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M2-M1 notation=content peak=0 positive_single_differences=0 final=0
638:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M3-M1 notation=position peak=90 positive_single_differences=59 final=90
639:E7RESULT name=cumulative_difference sequence=wa_keep1of2_a1000 window=main pair=M3-M1 notation=content peak=26 positive_single_differences=26 final=26
```

### 幾何・ノブ感度（第8節）

各ノブの主点または参照点と反対点を7行で比較し、同判定を `two_points_same`、異なる判定を `unstable` とする。負荷ノブは点ごとに対応（W-R は種別、0.75 も比較）。1/233保持は A=2330 のみなので同 A の1/2保持を参照。f ノブは22系列の主窓と f=59 窓で M3 関連4行を比較。記法ノブは7主点の Q2p/Q2c、Q3p/Q3c を比較。判別力の自己検証は同判定の2値の間へ閾値を移して異判定にする。設定ごとの件数を数える（コマンドはリポジトリのルートで実行し、出力をそのまま示す）。第三列は判定の比較、第四列は閾値を移した後の結果であり、`-` は二点の判定がもともと異なるため自己検証を行わないことを示す：

```
awk '/name=sensitivity/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} key=v["knob"] " " v["result"] " " (v["moved_result"]==""?"-":v["moved_result"]); count[key]++} END{for(k in count) print count[k], k}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out | sort -k2,2 -k3,3
```

```
7 batch_size_2330 two_points_same unstable
32 borrow_floor_59 two_points_same equal_values_cannot_be_separated
56 borrow_floor_59 two_points_same unstable
2 deleted_fraction_0.95 two_points_same equal_values_cannot_be_separated
5 deleted_fraction_0.95 two_points_same unstable
3 deletion_order_largest_first two_points_same equal_values_cannot_be_separated
4 deletion_order_largest_first two_points_same unstable
28 insert_probability_0.75 two_points_same unstable
7 insert_probability_0.75 unstable -
30 insert_probability_0.95 two_points_same unstable
5 insert_probability_0.95 unstable -
6 kept_fraction_1of233 two_points_same unstable
1 kept_fraction_1of233 unstable -
6 kept_fraction_3of4 two_points_same unstable
1 kept_fraction_3of4 unstable -
1 rewrite_notation_content two_points_same equal_values_cannot_be_separated
20 rewrite_notation_content two_points_same unstable
```

同判定200組のうち162組は閾値移動で異判定。38組は値が完全一致し閾値を間に移せない（990行 `equal_values_not_separable=38`）。第8節に報告方法がないため別記：f ノブ32組（8系列×4行。p=0.95 の5種、W-S 降順、W-A 1/2保持 A=2330、W-A 3/4保持はいずれも M3 と M3 f = 59 の両方で一度も借りておらず、軌跡行は `borrows=0`）、W-S の2ノブ5組、記法ノブ1組（両値0.0000または1.0000）。この38組で言えるのは値が同じことだけで、判別力は自己検証されていない。

判定が異なる14組の原行：

```
776:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed1 main_value=0.3594 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed1 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
783:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed2 main_value=0.3647 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed2 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
790:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed3 main_value=0.3595 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed3 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
797:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed4 main_value=0.3476 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed4 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
804:E7RESULT name=sensitivity knob=insert_probability_0.95 row=Q1r main=wr_p055_seed5 main_value=0.3609 main_verdict=merge_buys_double_capacity opposite=wr_p095_seed5 opposite_value=1.0000 opposite_verdict=no_merge_capacity_cost_small result=unstable
811:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed1 main_value=0.3594 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed1 opposite_value=0.8838 opposite_verdict=no_flip result=unstable
818:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed2 main_value=0.3647 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed2 opposite_value=0.8668 opposite_verdict=no_flip result=unstable
819:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q2p_delta2 main=wr_p055_seed2 main_value=-0.0393 main_verdict=negligible opposite=wr_p075_seed2 opposite_value=0.1046 opposite_verdict=no_flip result=unstable
820:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q2p_delta3 main=wr_p055_seed2 main_value=-0.0301 main_verdict=negligible opposite=wr_p075_seed2 opposite_value=0.1100 opposite_verdict=no_flip result=unstable
825:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed3 main_value=0.3595 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed3 opposite_value=0.8825 opposite_verdict=no_flip result=unstable
832:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed4 main_value=0.3476 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed4 opposite_value=0.8640 opposite_verdict=no_flip result=unstable
839:E7RESULT name=sensitivity knob=insert_probability_0.75 row=Q1r main=wr_p055_seed5 main_value=0.3609 main_verdict=merge_buys_double_capacity opposite=wr_p075_seed5 opposite_value=0.8698 opposite_verdict=no_flip result=unstable
860:E7RESULT name=sensitivity knob=kept_fraction_3of4 row=Q1r main=wa_keep1of2_a1000 main_value=0.6333 main_verdict=no_flip opposite=wa_keep3of4_a1000 opposite_value=0.9658 opposite_verdict=no_merge_capacity_cost_small result=unstable
867:E7RESULT name=sensitivity knob=kept_fraction_1of233 row=Q1r main=wa_keep1of2_a2330 main_value=0.5332 main_verdict=no_flip opposite=wa_keep1of233_a2330 opposite_value=0.0711 opposite_verdict=merge_buys_double_capacity result=unstable
```

Q1r は、挿入確率（0.55 対 0.95、0.55 対 0.75 の二方向では五つの種子すべてが反転）と、残す割合（1/2 対 3/4、1/2 対 1/233 の二方向）のどちらに対しても不安定である。Q2p の二行は、挿入確率 0.55 対 0.75 の種子 2 でだけ不安定である。Q2c と Q3 の両記法は、どの設定でも二点の判定が同じになる。

### 第2段：アンカーと陽性対照

第2段成果物2～32行は第1段の同じ行と逐語一致（diff 出力なし）。33～48行は3変種の追加16マス（修訂3：a1 は登録答で判定、他は報告のみ）：

```
E7RESULT name=anchor cell=A1 arm=M3_just_enough refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M2_left_first refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A1 arm=M3_left_first refused_insert=31456 live=31455 containers=135 matches=true
E7RESULT name=anchor cell=A2 arm=M3_just_enough refused_insert=62776 live=31455 containers=135 merges=134 borrows=15197 matches=unregistered
E7RESULT name=anchor cell=A2 arm=M2_left_first refused_insert=62776 live=31455 containers=135 merges=134 borrows=0 matches=unregistered
E7RESULT name=anchor cell=A2 arm=M3_left_first refused_insert=62776 live=31455 containers=135 merges=134 borrows=918 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M2_left_first content=117 position=117 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=116/233/1 matches=unregistered
E7RESULT name=anchor cell=A3 arm=M3_left_first content=118 position=118 containers=2 merges=1 borrows=1 third_sentence_false=1 sizes=174/176 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M3_just_enough content=349 position=351 containers=3 merges=1 borrows=116 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M2_left_first content=232 position=235 containers=3 merges=0 borrows=0 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A4 arm=M3_left_first content=240 position=242 containers=3 merges=1 borrows=7 third_sentence_false=0 sizes=233/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M3_just_enough content=350 position=352 containers=4 merges=1 borrows=116 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A5 arm=M3_left_first content=241 position=243 containers=4 merges=1 borrows=7 third_sentence_false=0 sizes=233/232/233/1 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M3_just_enough content=349 position=349 containers=1 merges=1 borrows=116 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M2_left_first content=233 position=233 containers=1 merges=1 borrows=0 third_sentence_false=0 sizes=233 matches=unregistered
E7RESULT name=anchor cell=A6 arm=M3_left_first content=240 position=240 containers=1 merges=1 borrows=7 third_sentence_false=0 sizes=233 matches=unregistered
```

登録された手作り a2～a6 では左先確認と主ノブの M2/M3 各値がマスごとに等しく、このノブを弁別できない。単体テストの117/117/117の3葉は弁別可能（修訂3第5行、変異 V14 で失敗）。just_enough は a4～a6 で116回借り、主ノブは7回。これは逐次観察でない推論：毎回117まで借りると次の削除で再び下限を割りまた借りる。

### 第2段測定：Q2/Q3（3窓）

主ウィンドウの一組の数値は、「読み取り値：Q2、Q3」の節にある第 1 段の表と一字一句同じである（第 2 段の成果物の主ウィンドウ 396 行は、第 1 段の同名行と同一）。残りの二つのウィンドウについて、コマンド（リポジトリのルートで実行し、`W` は `just_enough` または `left_first` とする）と、そのままの出力を示す：

```
awk -v W=just_enough '/name=point_verdict/ && $0 ~ (" window=" W " ") {delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} out[v["sequence"]]=out[v["sequence"]] " " v["row"] "=" v["value"]; if(!(v["sequence"] in seen)){seen[v["sequence"]]=1; order[++n]=v["sequence"]}} END{for(i=1;i<=n;i++) print order[i] out[order[i]]}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out
```

`just_enough`（M1/M2/M3_just_enough）：

```
wr_p055_seed1 Q2p_delta2=-0.0260 Q2p_delta3=0.0177 Q2c_delta2=0.0012 Q2c_delta3=0.0454 Q3p_delta32=0.0437 Q3c_delta32=0.0442
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=0.0255 Q2c_delta2=0.0013 Q2c_delta3=0.0661 Q3p_delta32=0.0648 Q3c_delta32=0.0648
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=0.0381 Q2c_delta2=0.0012 Q2c_delta3=0.0616 Q3p_delta32=0.0611 Q3c_delta32=0.0604
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=0.0377 Q2c_delta2=0.0013 Q2c_delta3=0.0567 Q3p_delta32=0.0550 Q3c_delta32=0.0554
wr_p055_seed5 Q2p_delta2=-0.0219 Q2p_delta3=0.0376 Q2c_delta2=0.0013 Q2c_delta3=0.0637 Q3p_delta32=0.0595 Q3c_delta32=0.0624
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.1106 Q2c_delta2=0.0000 Q2c_delta3=0.0192 Q3p_delta32=0.0225 Q3c_delta32=0.0192
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1282 Q2c_delta2=0.0000 Q2c_delta3=0.0216 Q3p_delta32=0.0236 Q3c_delta32=0.0216
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0957 Q2c_delta2=0.0000 Q2c_delta3=0.0141 Q3p_delta32=0.0161 Q3c_delta32=0.0141
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.1096 Q2c_delta2=0.0000 Q2c_delta3=0.0186 Q3p_delta32=0.0111 Q3c_delta32=0.0186
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.1068 Q2c_delta2=0.0000 Q2c_delta3=0.0266 Q3p_delta32=0.0133 Q3c_delta32=0.0266
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.4975 Q2c_delta2=0.0000 Q2c_delta3=0.5018 Q3p_delta32=0.4975 Q3c_delta32=0.5018
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.4975 Q2c_delta2=0.0000 Q2c_delta3=0.5018 Q3p_delta32=0.4975 Q3c_delta32=0.5018
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0053 Q2c_delta2=0.0000 Q2c_delta3=0.0002 Q3p_delta32=0.0002 Q3c_delta32=0.0002
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.4749 Q2c_delta2=0.0004 Q2c_delta3=0.4568 Q3p_delta32=0.4558 Q3c_delta32=0.4564
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

`left_first`（M1/M2_left_first/M3_left_first）：

```
wr_p055_seed1 Q2p_delta2=-0.0248 Q2p_delta3=-0.0147 Q2c_delta2=0.0012 Q2c_delta3=0.0113 Q3p_delta32=0.0101 Q3c_delta32=0.0101
wr_p055_seed2 Q2p_delta2=-0.0393 Q2p_delta3=-0.0288 Q2c_delta2=0.0013 Q2c_delta3=0.0108 Q3p_delta32=0.0105 Q3c_delta32=0.0096
wr_p055_seed3 Q2p_delta2=-0.0230 Q2p_delta3=-0.0128 Q2c_delta2=0.0012 Q2c_delta3=0.0108 Q3p_delta32=0.0102 Q3c_delta32=0.0096
wr_p055_seed4 Q2p_delta2=-0.0172 Q2p_delta3=-0.0091 Q2c_delta2=0.0013 Q2c_delta3=0.0116 Q3p_delta32=0.0081 Q3c_delta32=0.0103
wr_p055_seed5 Q2p_delta2=-0.0233 Q2p_delta3=-0.0134 Q2c_delta2=0.0013 Q2c_delta3=0.0106 Q3p_delta32=0.0100 Q3c_delta32=0.0093
wr_p075_seed1 Q2p_delta2=0.0881 Q2p_delta3=0.0864 Q2c_delta2=0.0000 Q2c_delta3=0.0045 Q3p_delta32=-0.0017 Q3c_delta32=0.0045
wr_p075_seed2 Q2p_delta2=0.1046 Q2p_delta3=0.1111 Q2c_delta2=0.0000 Q2c_delta3=0.0042 Q3p_delta32=0.0066 Q3c_delta32=0.0042
wr_p075_seed3 Q2p_delta2=0.0796 Q2p_delta3=0.0822 Q2c_delta2=0.0000 Q2c_delta3=0.0040 Q3p_delta32=0.0026 Q3c_delta32=0.0040
wr_p075_seed4 Q2p_delta2=0.0985 Q2p_delta3=0.0875 Q2c_delta2=0.0000 Q2c_delta3=0.0041 Q3p_delta32=-0.0110 Q3c_delta32=0.0041
wr_p075_seed5 Q2p_delta2=0.0935 Q2p_delta3=0.0936 Q2c_delta2=0.0000 Q2c_delta3=0.0046 Q3p_delta32=0.0001 Q3c_delta32=0.0046
wr_p095_seed1 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed2 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed3 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed4 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wr_p095_seed5 Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
ws_a1000_d500_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0299 Q2c_delta2=0.0000 Q2c_delta3=0.0342 Q3p_delta32=0.0299 Q3c_delta32=0.0342
ws_a1000_d950_smallest_first Q2p_delta2=0.0000 Q2p_delta3=0.0300 Q2c_delta2=0.0000 Q2c_delta3=0.0343 Q3p_delta32=0.0300 Q3c_delta32=0.0343
ws_a1000_d500_largest_first Q2p_delta2=0.0000 Q2p_delta3=0.0000 Q2c_delta2=0.0000 Q2c_delta3=0.0000 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of2_a1000 Q2p_delta2=0.0050 Q2p_delta3=0.0055 Q2c_delta2=0.0000 Q2c_delta3=0.0016 Q3p_delta32=0.0004 Q3c_delta32=0.0016
wa_keep1of2_a2330 Q2p_delta2=0.0172 Q2p_delta3=0.0172 Q2c_delta2=0.0001 Q2c_delta3=0.0001 Q3p_delta32=0.0000 Q3c_delta32=0.0000
wa_keep1of233_a2330 Q2p_delta2=0.0191 Q2p_delta3=0.0457 Q2c_delta2=0.0004 Q2c_delta3=0.0275 Q3p_delta32=0.0265 Q3c_delta32=0.0271
wa_keep3of4_a1000 Q2p_delta2=0.0003 Q2p_delta3=0.0003 Q2c_delta2=0.0003 Q2c_delta3=0.0003 Q3p_delta32=0.0000 Q3c_delta32=0.0000
```

記法とノブをそれぞれ分けて結論を書く：

- **左先・位置記法：** Δ2p/Δ3p は「反転なし」のまま。p=0.75 種2の0.1046/0.1111は0.10を超える。p=0.55 主点は−0.0393～−0.0091のまま。最大の変化は p=0.75 種1 Δ3p の0.0724→0.0864、絶対差0.014だが判定不変。 数え方は、各系列の Δ2p と Δ3p の二行について `left_first` ウィンドウから主ウィンドウを引いた値の絶対値を取り、その最大を求める。`awk` で `name=point_verdict` 行を読むと、`max_abs_change 0.014 wr_p075_seed1 Q2p_delta3 0.0724 -> 0.0864` と出力される。
- **左先・内容記法と Q3：** 4行とも「無視可能」のまま。最大0.0343（W-S 昇順950、主窓と同じ）。
- **下限にちょうど足りるだけ借りる、位置記法**：Δ3p は依然として「反転なし」だが、0.10 を超える点は一つから四つに増える（W-R p = 0.75 の種子 1、2、4、5：0.1106、0.1282、0.1096、0.1068）。W-S の二つの昇順削除点は 0.4975 に、W-A の 1/233 を残す点は 0.4749 に達する。主サンプル点 W-R p = 0.55 では 0.0177 … 0.0381 となる。1.00 に達する点は一つもない。
- **下限ぴったり・内容記法と Q3：** Δ3c と Q3 2行は「無視可能」から「反転なし」に変化。借用コスト無視可能に不利な側では、W-S 主点（昇順500）の Δ3c=0.5018、Q3 は0.4975/0.5018、W-A 1/233保持は0.4558～0.4568。1.00に達せず「顕著」には該当しない。
- Δ2p と Δ2c の二行は、`just_enough` ウィンドウでは主ウィンドウと一つずつ同じ値になる（同じ一対のアーム M1、M2 を比較している）。

軌跡と借用回数（各アーム拒否時生存数、3 M3 変種の借用、2 M2 変種のマージ）。コマンドと原出力：

```
awk '/name=arm_run/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} s=v["sequence"]; if(!(s in seen)){seen[s]=1; order[++n]=s} live[s,v["arm"]]=v["live_at_stop"]; br[s,v["arm"]]=v["borrows"]; mg[s,v["arm"]]=v["merges"]} END{print "sequence M1 M2 M3 M3_just_enough M2_left_first M3_left_first | borrows M3 M3_just_enough M3_left_first | merges M2 M2_left_first"; for(i=1;i<=n;i++){s=order[i]; print s, live[s,"M1"], live[s,"M2"], live[s,"M3"], live[s,"M3_just_enough"], live[s,"M2_left_first"], live[s,"M3_left_first"], "|", br[s,"M3"], br[s,"M3_just_enough"], br[s,"M3_left_first"], "|", mg[s,"M2"], mg[s,"M2_left_first"]}}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out
```

```
sequence M1 M2 M3 M3_just_enough M2_left_first M3_left_first | borrows M3 M3_just_enough M3_left_first | merges M2 M2_left_first
wr_p055_seed1 8070 22452 22116 22391 22416 22163 | 1020 6212 1024 | 429 428
wr_p055_seed2 8177 22422 21909 22306 22493 21909 | 1029 6774 989 | 422 424
wr_p055_seed3 7978 22190 21867 22277 22293 21757 | 950 6298 931 | 406 409
wr_p055_seed4 7933 22823 21820 22317 22823 22308 | 981 6273 1012 | 408 408
wr_p055_seed5 8083 22396 22260 22096 22396 21704 | 1001 6615 944 | 407 407
wr_p075_seed1 21360 24167 23551 24167 24167 24011 | 53 222 52 | 20 20
wr_p075_seed2 21173 24427 24264 24427 24427 24264 | 42 245 51 | 22 22
wr_p075_seed3 21378 24225 24225 24388 24225 24225 | 50 172 50 | 22 22
wr_p075_seed4 20994 24299 23980 24299 24299 23980 | 56 249 51 | 23 23
wr_p075_seed5 21157 24325 24163 24325 24325 24163 | 53 322 53 | 23 23
wr_p095_seed1 29865 29865 29865 29865 29865 29865 | 0 0 0 | 0 0
wr_p095_seed2 29810 29811 29811 29811 29811 29811 | 0 0 0 | 1 1
wr_p095_seed3 29802 29803 29803 29803 29803 29803 | 0 0 0 | 1 1
wr_p095_seed4 29815 29815 29815 29815 29815 29815 | 0 0 0 | 0 0
wr_p095_seed5 29821 29821 29821 29821 29821 29821 | 0 0 0 | 0 0
ws_a1000_d500_smallest_first 31245 31245 31245 31245 31245 31245 | 913 15174 913 | 0 0
ws_a1000_d950_smallest_first 31319 31319 31284 31275 31319 31284 | 17277 286396 17277 | 4 4
ws_a1000_d500_largest_first 31455 31455 31455 31455 31455 31455 | 0 0 0 | 0 0
wa_keep1of2_a1000 17132 27054 25464 27054 27054 25464 | 36 4 36 | 106 106
wa_keep1of2_a2330 16773 31455 31455 31455 31455 31455 | 0 0 0 | 130 130
wa_keep1of233_a2330 2236 31455 31081 31081 31455 31081 | 179568 2951927 179568 | 29004 29004
wa_keep3of4_a1000 24606 25478 25478 25478 25478 25478 | 0 0 0 | 33 33
```

132 本のアームはすべて拒否された（`grep name=arm_run research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out | grep -c 'stop=refused'` の出力は 132）。W-S 主点の just_enough は15174回借り、主ノブは913回。同じ窓の内容記法ヒストグラムは0片0回・1片15326回・2片15174回（第 809 行を行全体のまま引用）：

```
809:E7RESULT name=window_rewrites sequence=ws_a1000_d500_smallest_first window=just_enough arm=M3_just_enough unlinks=30500 content_sum=45674 position_sum=54403 content_mean=1.4975 position_mean=1.7837 content_largest=2 position_largest=133 content_histogram=0/15326/15174/0/0 position_histogram=0/15196/15174/6/124
```

M2_left_first が拒否された時点の生存数は、単なる読み取り値であり、この設定について Q1r を判定することはしない（登録第 6 節）。W-R p = 0.55 では 22,293 … 22,823 であり、主設定の M2（22,190 … 22,823）と同じ桁の大きさである。

### 第2段：2ノブの感度

ノブごとの集計（リポジトリルート、そのまま出力；第3列は判定比較、第4列は閾値移動後であり、`-` は2点が元々異なっており自己証明を行わないことを示す）：

```
awk '/name=sensitivity/{delete v; for(i=2;i<=NF;i++){split($i,a,"=");v[a[1]]=a[2]} key=v["knob"] " " v["result"] " " (v["moved_result"]==""?"-":v["moved_result"]); count[key]++} END{for(k in count) print count[k], k}' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out | sort -k2,2 -k3,3
```

```
32 borrow_amount_just_enough two_points_same equal_values_cannot_be_separated
41 borrow_amount_just_enough two_points_same unstable
15 borrow_amount_just_enough unstable -
33 check_order_left_first two_points_same unstable
99 check_order_left_first two_points_same equal_values_cannot_be_separated
```

両点同判定の 205 対のうち 74 対は閾値移動後に異なる判定に転じる；閾値移動の自己証明ができない対の数は 131 対である（両点の値がビット単位で等しい；第 1369 行 `equal_values_not_separable=131`、単独列挙し、他のカウントに合算しない）。系列数によるカウント（`awk` で `moved_result=equal_values_cannot_be_separated` の感度行を読み、ノブと主点系列名でカウント）：「借用量」32 対 = 8 系列 × 4 行であり、まさに M3 と M3_just_enough がともに 1 度も借用しなかった 8 系列である（W-R p = 0.95 の 5 種、W-S 降順、W-A 1/2 保持 A = 2330、W-A 3/4 保持；借用回数は「第 2 段読み取り値：Q2、Q3（3 つの窓）」の節の軌跡と借用回数の表を参照）；「先に調べる対」99 対 = 14 系列で 6 行すべて一致（W-S 3 点、W-A 4 点、W-R p = 0.95 の 5 種、p = 0.75 の種 3 と 5）の計 84 対に、W-R p = 0.55 の 5 種および p = 0.75 の種 1、2、4 で一部の行が一致する計 15 対を加えたもの。この 131 対については「両点の値が同一である」としか記述できず、判別力の自己証明はない。判定が異なる 15 対はすべて「借用量」ノブに存在する（第 1169、1181、1185、1209–1216、1229–1232 行；`grep -n 'name=sensitivity.* result=unstable' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` で列出し、行全体を 4 行抜粋して例示）：

```
1169:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2p_delta3 main=wr_p075_seed1@M3 main_value=0.0724 main_verdict=negligible opposite=wr_p075_seed1@M3_just_enough opposite_value=0.1106 opposite_verdict=no_flip result=unstable
1209:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2p_delta3 main=ws_a1000_d500_smallest_first@M3 main_value=0.0299 main_verdict=negligible opposite=ws_a1000_d500_smallest_first@M3_just_enough opposite_value=0.4975 opposite_verdict=no_flip result=unstable
1210:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2c_delta3 main=ws_a1000_d500_smallest_first@M3 main_value=0.0342 main_verdict=negligible opposite=ws_a1000_d500_smallest_first@M3_just_enough opposite_value=0.5018 opposite_verdict=no_flip result=unstable
1230:E7RESULT name=sensitivity knob=borrow_amount_just_enough row=Q2c_delta3 main=wa_keep1of233_a2330@M3 main_value=0.0275 main_verdict=negligible opposite=wa_keep1of233_a2330@M3_just_enough opposite_value=0.4568 opposite_verdict=no_flip result=unstable
```

Q2p、Q2c、Q3 は「借用量」に対して不安定である（M3 に関わる 4 行であり、W-R p = 0.75、W-S 昇順削除、W-A 1/233 保持において判定が反転する）；「先に調べる対」に対しては2点とも同一判定である。

### 突き合わせ（停止条項、原行）

`e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out` 全文：

```
1:E7RESULT name=oracle_constants model_leaf_record_capacity=233 crates_inode_leaf_records=233 model_root_node_leaf_container_capacity=135 crates_leaf_containers_one_root_node_holds=135 equal=true
2:E7RESULT name=oracle_crates_unit_test_shape model=sizes=233/233/34,container_numbers=1/234/467 crates=sizes=233/233/34,container_numbers=1/234/467 equal=true
3:E7RESULT name=oracle_crates_unit_test_shape model=refused_inode=31456,refused_containers=136,capacity=135 crates=refused_inode=31456,refused_containers=136,capacity=135 equal=true
4:E7RESULT name=oracle_crates_unit_test_shape model=position_rewritten=[1],dropped=1 crates=position_rewritten=[1],dropped=1 equal=true
5:E7RESULT name=oracle sequence=wr_p055_seed1 operations_compared=78967 unlinks_compared=35448 m1_refused_operation=78967 first_difference=none
6:E7RESULT name=oracle sequence=wr_p055_seed2 operations_compared=83764 unlinks_compared=37793 m1_refused_operation=83764 first_difference=none
7:E7RESULT name=oracle sequence=wr_p055_seed3 operations_compared=81145 unlinks_compared=36583 m1_refused_operation=81145 first_difference=none
8:E7RESULT name=oracle sequence=wr_p055_seed4 operations_compared=83082 unlinks_compared=37574 m1_refused_operation=83082 first_difference=none
9:E7RESULT name=oracle sequence=wr_p055_seed5 operations_compared=82642 unlinks_compared=37279 m1_refused_operation=82642 first_difference=none
10:E7RESULT name=oracle sequence=wr_p075_seed1 operations_compared=42169 unlinks_compared=10404 m1_refused_operation=42169 first_difference=none
11:E7RESULT name=oracle sequence=wr_p075_seed2 operations_compared=42502 unlinks_compared=10664 m1_refused_operation=42502 first_difference=none
12:E7RESULT name=oracle sequence=wr_p075_seed3 operations_compared=42241 unlinks_compared=10431 m1_refused_operation=42241 first_difference=none
13:E7RESULT name=oracle sequence=wr_p075_seed4 operations_compared=42701 unlinks_compared=10853 m1_refused_operation=42701 first_difference=none
14:E7RESULT name=oracle sequence=wr_p075_seed5 operations_compared=42524 unlinks_compared=10683 m1_refused_operation=42524 first_difference=none
15:E7RESULT name=oracle sequence=wr_p095_seed1 operations_compared=33152 unlinks_compared=1643 m1_refused_operation=33152 first_difference=none
16:E7RESULT name=oracle sequence=wr_p095_seed2 operations_compared=33173 unlinks_compared=1681 m1_refused_operation=33173 first_difference=none
17:E7RESULT name=oracle sequence=wr_p095_seed3 operations_compared=33191 unlinks_compared=1694 m1_refused_operation=33191 first_difference=none
18:E7RESULT name=oracle sequence=wr_p095_seed4 operations_compared=33180 unlinks_compared=1682 m1_refused_operation=33180 first_difference=none
19:E7RESULT name=oracle sequence=wr_p095_seed5 operations_compared=33190 unlinks_compared=1684 m1_refused_operation=33190 first_difference=none
20:E7RESULT name=oracle sequence=ws_a1000_d500_smallest_first operations_compared=92246 unlinks_compared=30500 m1_refused_operation=92246 first_difference=none
21:E7RESULT name=oracle sequence=ws_a1000_d950_smallest_first operations_compared=1184620 unlinks_compared=576650 m1_refused_operation=1184620 first_difference=none
22:E7RESULT name=oracle sequence=ws_a1000_d500_largest_first operations_compared=92456 unlinks_compared=30500 m1_refused_operation=92456 first_difference=none
23:E7RESULT name=oracle sequence=wa_keep1of2_a1000 operations_compared=50133 unlinks_compared=16500 m1_refused_operation=50133 first_difference=none
24:E7RESULT name=oracle sequence=wa_keep1of2_a2330 operations_compared=47064 unlinks_compared=15145 m1_refused_operation=47064 first_difference=none
25:E7RESULT name=oracle sequence=wa_keep1of233_a2330 operations_compared=67197 unlinks_compared=32480 m1_refused_operation=67197 first_difference=none
26:E7RESULT name=oracle sequence=wa_keep3of4_a1000 operations_compared=40607 unlinks_compared=8000 m1_refused_operation=40607 first_difference=none
27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=22 constants_equal=true crates_unit_test_shapes_equal=true all_equal=true
28:E7RESULT name=done emitted=28
```

判別力の自己検証：モデルで V1 を有効化した `e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1.out` は7系列で最初の差を報告。W-S 主点では操作1233、`crates/` の書き換えリスト長4、モデル0で登録第9節どおり。`crates/` の「空の中央葉を除去」単体テスト形状も不一致。残り15系列（p=0.75/0.95、W-S 降順、W-A 4点）は M1 拒否前に右端以外を除去せず、位置左移動の分岐に到達しない。

```
4:E7RESULT name=oracle_crates_unit_test_shape model=position_rewritten=[],dropped=1 crates=position_rewritten=[1],dropped=1 equal=false
20:E7RESULT name=oracle sequence=ws_a1000_d500_smallest_first operations_compared=1233 unlinks_compared=233 m1_refused_operation=none first_difference=operation=1233 Unlink(233) 位置記法の書き換えリスト不一致：crates [0, 1, 2, 3]（長さ4）、モデル []（長さ0）
27:E7RESULT name=verdict scope=oracle sequences=22 sequences_compared_to_refusal_without_difference=15 constants_equal=true crates_unit_test_shapes_equal=false all_equal=false
```

第 2 段を始める前に、突き合わせと判別力をそれぞれ第 2 段のソースから再ビルドし、一回ずつ再実行した。`e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out` は第 1 段の突き合わせ成果物と `cmp` でバイト単位の一致を確認した（判決行は `all_equal=true`）。`e176-inode-leaf-merge-capacity-and-rewrites-oracle-discrimination-v1-2026-10-08.out` も第 1 段の判別力成果物と `cmp` でバイト単位の一致を確認した（V1 のコピーは終了コード 1 で、同じく W-S の主サンプル点の第 1233 操作で差を報告した）。突き合わせが比較するのは M1 だけであり、第 2 段の三つの変種はその射程に入らない。

### 単体テストと変異

第 2 段のソースには単体テストが 22 個ある（`grep -c '^test .* \.\.\. ok$' e176-inode-leaf-merge-capacity-and-rewrites-unit-tests-2026-10-08.out` は 22 を出力し、同じファイルに `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s` がある）。第 1 段の 14 個（`e176-inode-leaf-merge-capacity-and-rewrites-unit-tests.out`）のうち絶対値を固定するものは、すべてのアンカーセル（26 セルの登録された答えと一字一句比較）、a1 と a2 の M1 を別に加算式で表したもの、SHA-256 の公開テストベクトル三つ、四種の負荷生成器の最初の数ステップ、丙類アサーションが五種の不正形態それぞれで赤になること、閾値の境界値、閾値を移す自己検証、系列ごとのスレッド実行と単一スレッド実行の完全一致、W-R 主サンプル点で三つのアームの操作ストリームのプレフィックスが同じであること。第 2 段で追加した八つは、第 2 段のウィンドウ表と三変種の方針を一つずつ固定するもの、第 2 段アンカー（登録セル 29、全体 47）、三片 117 / 117 / 117 で右を先に調べる場合と左を先に調べる場合の末状態と二つの記法（M2、M3 各二つで計四つ）、第 2 段の六アーム変種のスレッド実行と単一スレッドの完全一致、第 2 段の感度行の設定と行を一つずつ固定して左を先に調べる点が `left_first` ウィンドウを取ることを確認するもの。ランダム長時間実行の丙類アサーションの単体テストには M3_left_first を追加した。

ミューテーション表 `scripts/mutation/tables/e176_inode_leaf_merge_capacity_and_rewrites.tsv` は16件（登録第9節 V1～V13、修訂3 V14～V16）、アンカーは第 2 段の装置ソースから取り、それぞれちょうど一回一致する。検出16、無効0、生存0、💥/⚠️/⏱/🧱 も各0。終了時の集計は「メモリ上限到達 0 件（上限 4G）、タイムアウト 0 件、実行枠に入れないもの 0 件、総上限で押し出されたもの 0 件、GPU メモリ不足 0 件」、そして「復元済み、ベースラインは引き続き全緑」である。第1段の13件も全検出。`e176-inode-leaf-merge-capacity-and-rewrites-mutations-2026-10-08.out` の集計行と復元後全緑の二行を参照。第 1 段の 13 件すべての検出結果は `e176-inode-leaf-merge-capacity-and-rewrites-mutations.out` にある。V4 は系列・アンカー共通の `live_inodes_at_refusal` を変更、V13 は W-R 主点で無効化アサーションが即失敗、V9 は丙類の隣接キー範囲非重複で失敗。V14 は117/117/117の左先テスト、V15 は a3 just_enough 登録答、V16 は第2段窓表テストが検出。

16件全検出はアサートされた箇所が変異検査されたことのみで、モデル全行の検査を意味しない。`point_verdict`、`overall_verdict`、`moved_threshold_verdicts` の算術は単体テストで境界固定するが変異なし。`segment_two_sensitivity_lines` の窓選択も単体テストで1点確認し変異なし。

### 分岐表

| 分岐の行 | 状態 | 根拠（成果物からそれを計算するコマンド） | まだ何が足りないか；残った量によって判定が反転する可能性はあるか |
|---|---|---|---|
| M 行の反転観測の第一文：M1 が 135 片に達した時点の生存数 | 判定に十分 | `grep 'name=arm_run' research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` の `live_at_stop`；Q1r の `awk` コマンド（「読み取り値：Q1」の節）；第 1 段の総判定の第 763 行と第 2 段の主ウィンドウの第 1130 行は同じ判定 | 不足はない。登録された量は両方の段で実行済みで、残った量はない。第 2 段は M1 と主設定の M2 を変更していない（第 2 段の主ウィンドウと M1、M2、M3 の三つのアームの 396 行は第 1 段と同一）ため、Q1r の判定は変わらない。Q1r が負荷に対して不安定であること（感度の節）は、測定された性質であり、不足している量ではない |
| M 行の反転観測の第二文：M2、M3 が unlink 一回ごとに M1 より余分に書き直す葉片数 | 判定に十分。位置記法では Δ2p、Δ3p は三つのウィンドウすべてで「反転なし」。内容記法では Δ2c は三つすべてで「無視可能」。Δ3c と Q3 の二行は主ウィンドウと `left_first` ウィンドウで「無視可能」、`just_enough` ウィンドウで「反転なし」 | 第 1 段の総判定の第 764–775 行、第 2 段の総判定の第 1131–1148 行。「第 2 段の読み取り値：Q2、Q3」の節の `awk`（`W` は `just_enough`、`left_first`） | 不足はない。第 2 段は登録された最後の段で、完了しており、残った量はない。位置記法は二つの設定のどちらでも「無視可能」または「顕著」へ反転していない（`just_enough` ウィンドウの Δ3p 最大値は 0.4975、`left_first` ウィンドウでは 0.1111）。「借りる量」の設定は Δ3c と Q3 を「無視可能」から「反転なし」へ変え、15 対で判定が異なる（「第 2 段：二つの設定の感度」の節）ため、「不安定」と記録する。三つのウィンドウに 1.00 に達する点はない |
| M 行の判定に十分という条件：三つの負荷族で二つの量をどちらも測る | 判定に十分 | 第 1 段成果物の第 761 行 `m1_reached_on_every_point=true`、第 760 行 `all_window_prefixes_equal=true`；第 2 段成果物の第 1129 行 `m1_reached_on_every_point=true`、第 1128 行 `all_window_prefixes_equal=true`。第 1 段の 88 行、第 2 段の 132 行の `name=arm_run` はすべて `stop=refused` | 不足はない |

### 答えられないこと

- 葉のグループ化と書き換え片数のみ。内部節点の区切りキー経路（M3 が借りた番号は区切りキーで右葉へルーティングされ、I-9.4 判定修訂時に併せて答える）、クラッシュポイント再生（C116 の他半分）、GPU 判定変更の作業量は作らない。M の選択はこれら未測定にも依存する。
- 1パブリッシュにつき1操作。inode 一括作成、unlink のインテントのパブリッシュと中間バッチ（1行をその場で変更し全アームで1葉のみ書き換え）は比較外。最後のパブリッシュで3アームとも根を書き換えるので比較外。
- 22 本の系列は列挙した点であり、定義域ではない。W-R は三つの p しかなく、W-A は四つの残し方しかない。この一群の負荷では Q1r は p と残す割合によって反転する。ほかの負荷形態、例えば長時間の定常状態やディレクトリ一つをまとめて削除する場合がどちら側になるかは、測定していない。
- M3 で測ったのは、下限の二段階（117、59）、借りる量の二段階（均等配分、下限にちょうど足りるだけ借りる）、どの対を先に調べるかの二段階（右、左）だけである。この三つの設定は交差させていない。下限にちょうど足りるだけ借りる場合と左を先に調べる場合は、いずれも f = 117 でのみ測定した。ほかの借り方、例えば別の割合まで借りる方法は測っていない。「借りる量」は Δ3c と Q3 を反転させるが、測った段階の間の値がどちら側になるかは測っていない。
- 「どの対を先に調べるか」の感度では、99 対で二点の値がビット単位で等しく、閾値を移す自己検証ができない。これらの系列で、この設定が両方の対とも収まる分岐にそもそも到達していないのかどうかは、逐次の計数をしていないため読み取れない。
- 各アームの上限は 2 × 10⁷ 操作であり、両方の段とも、この上限に到達したアームはない。W-A で 1/233 を残す場合の M2 と M2_left_first は、拒否されるまでに 13,547,775 操作を要した。これは二つの段のうち第 2 段にある、操作数が最大の系列である。
- 最小充填率は第1段88アーム、第2段132アームとも最初の inode 作成時の1÷233で疎の程度を示さない。
- 再実行節の `replay.sh` は記述時点で全表の形状・受入事前検査により終了2、1行も実行しない。第2段執筆時は E175 (製品経路における inode 番号水位の後退と番号再発行) の行の第 4 列にスラッシュ付きパス、第1段時は E142 (新規プールでのファイル作成のドライラン) の受入キー衝突で、どちらも E176 (inode 葉のマージ 3 候補の容量と書き込みコスト) とは無関係。両成果物は同じコマンドを手動で2回実行しバイト比較。

### 経路と結論の登録

| # | 経路 | 算出 | ソース | 共通入力 |
|---|---|---|---|---|
| 1 | 独立手書きモデル | 葉表は番号のみ。末尾分割・空葉除去・M2 マージ・M3 借用、変更コンテナと最初の除去位置から2記法を計算。M1 は22登録系列の拒否前各操作で経路2と比較（列挙点、定義域全体ではない）。アンカーは経路3と比較。 | `scripts/experiment-apparatus/e176_inode_leaf_merge_capacity_and_rewrites.rs:155 insert`、`:278 remove`、`:321 rewrites_of_this_publish` | 233行/片、根ファンアウト135は `crates/` からコピーし経路2で突き合わせ。位置記法は当時の `remove_record_from_leaf_containers` 由来。突き合わせ両側は同じ負荷生成器を使用。 |
| 2 | `crates/` 2純関数 | 当時の M1：`write_records_into_leaf_containers` は末尾追加・分割、135超で拒否。`remove_record_from_leaf_containers` は空葉除去、右側すべてをリストへ。 | `crates/singlefs-core/src/inode_tree.rs:181 write_records_into_leaf_containers`、`:286 remove_record_from_leaf_containers` | format の `INODE_LEAF_RECORDS`、inode_tree の `leaf_containers_one_root_node_holds`、経路1の負荷生成器。 |
| 3 | 設計者の独立 Python 参照 | 同じ3アームを登録第7節の手作り6シナリオと空間増幅マスのみで計算。 | 実装は入庫されず、`e176-preregistration.md` 500行以降の `anchors.py` コードブロックのみ。 | 233、135、同一位置記法。X が空で除去された後に新隣接対を調べない（第13節、6マスに影響なし）。 |

経路間の比較アサーション：経路 1 対経路 2 は、突き合わせ bin `scripts/experiment-apparatus/e176_crates_oracle/oracle.rs` の `compare_one_sequence` に固定されている（一歩ずつ比較し、最初の差で停止する）。経路 1 対経路 3 は単体テスト `every_registered_anchor_matches_the_independent_reference` に固定されている（同じソースの第 2154 行、26 セルの登録された答えを一字一句比較する。第 2 段には別に `segment_two_anchors_judge_a1_for_the_three_variants_and_only_report_the_rest` があり、29 セルを比較する）。共通項についての問い：233 と 135 が誤っていれば三つの経路が一緒に誤り、「kb と実装がともに誤った値を選ぶ」ことは防げない。位置記法の三経路は同じ読み方に由来するため、その読み方自体が `crates/` の意図と違えば三経路とも誤る。負荷生成器は経路 1 に一つだけ存在するため、突き合わせでは生成器が登録どおりに生成したとは証明できない。その半分は単体テストで四種の負荷の最初の数ステップを固定することで確認する。

### 影響する決定

| 決定項目 | 関係 | 再確認 |
|---|---|---|
| D8 (コアインデックス構造)確定項目6 | 支持 | 2026-10-17 は文言・成果物参照・パス・数値の同期のみで結論・数・成果物は不変。2026-10-11 も時間語・位置指示・自称の文言変更のみで不変。 |

## 履歴バージョン

### 2026-10-08：第2段実行済み

- **変更前：** 第1段のみ、条件を満たした第2段は実行指定待ち。3変種未実行、制限に下限ぴったり・左先は第2段で未実行と記載。単体14、変異13、`replay.sh` は segment-one の1行。
- **変更後：** 両段実行済み。第2段22系列×6変種×3窓のアンカー、2窓の Q2/Q3、軌跡・借用、2ノブ感度、判決を追加。第1段の等値38組を閾値移動自己検証不可として明記。単体22、変異16（V14～V16 追加）。第2段ソースで突き合わせ・判別力を再実行し第1段とバイト一致。replay に segment-two を追加。D8 項目6は引き続き参考資料で再確認を書き換え。
- **根拠：** `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out` の第 1137–1148 行（二つの新しいウィンドウの総判定）、1369行 `E7RESULT name=verdict scope=sensitivity_self_proof same_pairs=205 unstable_pairs=15 moved_to_unstable=74 equal_values_not_separable=131 every_separable_same_pair_moved_to_unstable=true`、登録第12節修訂3。

### 2026-10-08：ページ作成、第1段実行済み

事前登録 `e176-preregistration.md` は装置作成前。モデル単体テスト後・成果物前に修訂1（f=59 陽性対照と共通窓）追加。初回変異実行で任意パス依存が `mutate.sh` コピーをビルド不能とする問題に当たり、修訂2で突き合わせ bin を独立 manifest に変更。第1段完了、第2段発動条件成立、メイン agent の指定待ち。
