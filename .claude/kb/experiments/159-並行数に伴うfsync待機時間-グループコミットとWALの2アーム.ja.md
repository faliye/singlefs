## E159 並行数に伴うfsync待機時間：グループコミットとWALの2アーム —— 装置は作成済み、計時は未実施（2026-09-24、継続セッション；純粋な計数部分は決定的、55単体テスト / 17件のミューテーションを全て検出；正式計時のメイングリッド、PC1–PC3、AA、G8、G1、G6の5ラウンドは未実施；装置の `RECORD_HEADER_BYTES` は依然311、指名項目の固定は67のままで、ヘッダ496・指名項目64になった後は再実行されていない；未済の検査 C776（E155、E159の装置はヘッダ311と指名項目67で止まっている））

[中文](159-fsync等待时间随并发数组提交与wal两臂.md) · [English](159-fsync-wait-time-vs-concurrency-group-commit-and-wal-arms.en.md) · **日本語**

<!-- doc-lint:registry name-col=2 -->

| 番号 | 略称 |
|---|---|
| J1 | 甲がある k で M を超える |
| J4 | p99 がある k で M を超える |

**問い**：事前登録 `e159-preregistration.md` —— 分岐表 `m2-fsync-wait-forks.md` の1行目「甲（現在の方式：fsync ごとにダーティなリーフと全祖先を書き、D16（パブリッシュ意味論）確定項目10に従ってバッチ化・グループコミットする）の待機時間は許容できるか」。「1回の fsync が発行から返るまでにどれだけ待ったか」を、甲と2本のWALアーム（wal_full-K10、乙-M-K10。両方残し、それぞれ別個に判定する）で、並行数 k に応じてどう推移するかを測る。装置は `scripts/experiment-apparatus/e159_fsync_wait_group_commit.rs` にある（独立した手書きモデルであり、リポジトリ登録済みの装置ではない —— `crates/` にはグループコミットがなく、第3節2.3で今回検索してヒット0）。遅延は実機（実マシン、実 O_DIRECT デバイス、実スレッド）での実走で測り、計数モデルからの換算は使わない。理由は事前登録の「五、5.1」を参照。

**この区間（セッション上限に達して中断した実行者を引き継ぐ）は「装置とスモークテスト」までしか終えていない**：正式計時（本機の計測条件での5ラウンドのメイングリッド、PC1–PC3、AA、G8、G1、G6）は実施していない —— マシン上で並行コンパイルが走っており、計時が汚染されるため（`.claude/kb/tooling.md` の「リポジトリ内の性能数値は本機でのみ成立する」と同種の懸念）、マシンが空いてから別途割り当てる。

### この区間で行ったこと

1. **装置**：`scripts/experiment-apparatus/e155_fourth_run_group_commit_concurrency.rs` の36–2058行から、定数、K1–K4、5.3のグループ式、3アームの単発/バッチ関数をそのままコピーした（事前登録の停止条項 S2 はこの範囲がソースファイルと項目ごとに diff 空であることを要求する。確認済み、「再実行」を参照）；新規に書いたのは、グループコミットキュー（`SharedQueue`、`drain_batch_from_queue`）、WAL チェックポイントスレッド（`checkpoint_loop`、`should_trigger_checkpoint`）、クローズドループの呼び出し側（`caller_loop`）、O_DIRECT I/O（`RealBlockDevice`）、構造的単体テスト用の偽デバイス（`RecordingBlockDevice`、`SleepingBlockDevice`）、統計（分位数、Mann–Whitney A、規則 R、J1（甲がある k で M を超える）/J2a/J4（p99 がある k で M を超える）の判定）、PC1 のビジーウェイト注入、`cell`/`probe-device`/`anchors` の3サブコマンド。
2. **停止条項 S1（アンカーセルを `crates/` と項目ごとに突き合わせる）**：単体テストが甲の F1、P=1、k=1 における単一ディスク総バイト数172544、書き込み呼び出し11回を固定（`jia_single_fsync_batch_matches_crates_test_table`。第3節の `crates/` テスト表の項目ごとの合計と一致）、パブリッシュ順序 ユニット→バリア→レコード→バリア→ルートスロット FUA→システム構成スロット（先に2回読み、次に1回書く）を段階ごとに照合（`jia_single_fsync_batch_step_sequence_matches_crates_order`）、ルートスロットの永続時刻がルートスロット FUA より早くないこと（`jia_durable_instant_is_not_earlier_than_the_root_slot_fua`）。S1 は発動しない。
3. **停止条項 S2（コピーブロックの diff が空）**：`diff <(sed -n '39,2061p' e159_...) <(sed -n '36,2058p' e155_...)` の終了コードは0。
4. **停止条項 S3（ゲスト `logical_block_size`）**：実機 VM プローブのサブコマンド `probe-device` が `logical_block_size=512` を読み戻し、主幾何定数 `physical_block_size_bytes` と一致する。発動しない。
5. **単体テストとミューテーション**：54個の単体テストが全て緑（`cargo test --release -p e7-index-bench --bin e159-fsync-wait-group-commit`）；ミューテーション表 `scripts/mutation/tables/e159_fsync_wait_group_commit.tsv` は14件（M1–M14、第9節の全対象を網羅：甲のパブリッシュから2本目のバリアを除去、1本目のバリア直後に解放、WALアームが計画関数を取り違え、グループコミットがオンでもバッチを1に制限、`t_issue` がキュー待ち区間を落とす、PC1 のビジーウェイトを解放の後ろへ移動、確率的丸めを切り捨てに変更、`T_time` 定数の読み違い、システム構成スロットを書く前に読まない、ルートスロット幅の誤変更、J1（甲がある k で M を超える）の境界比較演算子の誤変更、5ラウンドの統合を過半数ラウンドの判定に変更、オープンループの `issued_at` を実際の発行時刻に変更、`release_to_wake` が起床区間を落とす）、`mutate.sh` を2回（ソース変更の前後で各1回）実行し、いずれも14/14全て検出、締めの「復元済み、ベースラインは依然全て緑」。M3–M6（グループコミットキューと実並行のタイミングに関わる4件）は元々、実並行エンジンを覆う単体テストがなかったため、この区間で6件の単体テストを追加して補った：`plan_batch_for_arm_routes_each_arm_to_its_own_solver_not_a_different_one`（M3）、`drain_batch_from_queue_collects_everything_when_group_commit_is_on` / `_collects_only_one_when_group_commit_is_off`（M4）、`closed_loop_queueing_delay_is_counted_when_group_commit_is_off`（M5、`SleepingBlockDevice` でバリア遅延を固定してキュー待ちを強制し、実ディスクは不要）、`pc1_injected_wait_lands_before_release_not_after`（M6、同じく `SleepingBlockDevice` を使用）、`checkpoint_time_threshold_constant_is_five_seconds`（M8、`T_time` 定数を `CHECKPOINT_TIME_THRESHOLD` という名前付き定数に切り出し、`run_cell_subcommand` と単体テストが同じ配置先を共有し、リテラルをそれぞれ書かないようにした）。
6. **実機スモークテストで本物のバグに遭遇、修正済み**：`RealBlockDevice::read_at` の元の実装は、データを通常の `vec![0u8; length]`（非アライン）に読み込んでから O_DIRECT の `read_exact_at` に渡していた；初回の実走（`--arm jia --k 1024`）で即座に `panicked ... EINVAL`；この経路は2026-09-24の修正前は単体テスト0件でカバーされていなかった（旧単体テストは全て `RecordingBlockDevice`/`SleepingBlockDevice` のみを使用）。修正方法：データを `ALIGNMENT_BYTES` にアラインされた `AlignedBuffer`（新規 `AlignedBuffer::as_mut`）へ直接読み込み、読み終えてから戻り値へコピーする；修正後は同じコマンドを k=4 に替えても panic しない。成果物は「成果物から丸ごと転記した結果行」を参照。
7. **実機スモークテスト：3アームそれぞれ1セルが完走**（`VM_DISK_PREALLOCATE=1`、短い duration。装置が実走してクラッシュしないことを確認するだけで、正式な数値を出すためではない）：甲 k=1、wal_full-K10 k=2（duration 4s、`T_time`=5s にも `T_dirty`=256MiB にも達せず、checkpoint_count=0、想定どおり）、乙-M-K10 k=2（duration 6s、チェックポイントが1回発動、チェックポイント機構は実際に通った）。3アーム、グループコミットエンジン、WAL チェックポイントスレッドがいずれもエンドツーエンドで動くことを確認し、新たな panic には遭遇しなかった。
8. **`print_anchors` のリファクタ**：`Emitter` 経由で `E7RESULT` 行を出力し、`name=done emitted=N` で締めるように変更（2026-09-24のリファクタ前は素の `println!` で、`scripts/experiment-apparatus/replay.sh` の完全性ゲートを満たしていなかった）。また甲の行の単一ディスクバイト数の算出を `outcome.total_bytes() / DEVICE_COUNT`（ルートスロット項目には誤った近似で、単一ディスク専用の512バイトのルートスロットまで半分にしてしまう）から、単体テストと共有する `jia_outcome_single_device_bytes`（モジュール最上位の関数へ移動）に変更した；出力は172288から正しい172544に戻り、B4 アンカーと一致する。
9. **命名規律**：`naming-lint.sh` はこのファイルに対して終始通過（唯一の指摘は `naming-lint:external` で注記済みの外部フラグ名 `O_DIRECT` で、違反に数えない）。

### 成果物から丸ごと転記した結果行（成果物がワークスペース外へ移された行は、ポインタ1行にしてある）

`e159-fsync-wait-group-commit-2026-09-24-anchors.out`（純粋な算術、バイト単位で再実行可能、「再実行」を参照）：

```text
E7RESULT name=anchor label=jia_p1_f1_k1_single_device_bytes value=172544
E7RESULT name=anchor label=write_ahead_log_full_k10_p1_f1_k1_single_device_bytes value=102400
E7RESULT name=anchor label=yi_m_k10_p1_f1_k1_single_device_bytes value=86016
E7RESULT name=anchor label=write_ahead_log_full_k10_f8a_p10000_k1_single_device_expected_bytes value=365346.000
E7RESULT name=anchor label=write_ahead_log_full_k10_f8a_p10000_k16_single_device_expected_bytes value=4313362.000
E7RESULT name=done emitted=6
```

`e159-fsync-wait-group-commit-2026-09-24-smoke.out`（実機スモークテストの存証。`replay.sh` のバイト単位比較には参加しない —— 実遅延は毎回異なる —— 装置が実走できることだけを示す；完全な文脈は当該ファイルを参照）：修正後の3アームそれぞれ1セルの `E159RESULT` 行。例として乙-M-K10 のセル：

```text
产物行见 git show 5d0b7e7c1667:research/results/e159-fsync-wait-group-commit-2026-09-24-smoke.out 第 175 行（E7RESULT E159RESULT arm=yi-M-K10）
```

`containment_violations=0`（V5 包含セルフチェックに違反なし）、`checkpoint_count=1`（WAL チェックポイント機構が実際に1回発動した）。

### 再実行

```
bash scripts/experiment-apparatus/replay.sh E159
```

登録したのは `anchors` サブコマンドの1行のみ（純粋な算術、実デバイスに触れず、バイト単位で再実行可能）：`scripts/experiment-apparatus/replay.sh` 内の E159 (並行数に伴うfsync待機時間：グループコミットとWALの2アーム)の `anchors` 行、成果物 `e159-fsync-wait-group-commit-2026-09-24-anchors.out`、判定方式 `exact`、引き渡し前に実行済みで、判定は「バイト一致」。実機スモークテスト（S3 プローブと3アームそれぞれ1セル）は `replay.sh` に登録しない —— 遅延の数値は毎回異なり、固定できるバイト単位の判定基準がない；再実行コマンドはそのまま `e159-fsync-wait-group-commit-2026-09-24-smoke.out` ファイルのヘッダに保存してあり、再実行が必要なときは手動で実行する。例：

```
cargo build --release --target x86_64-unknown-linux-musl --bin e159-fsync-wait-group-commit -p e7-index-bench
bash scripts/devices/vm-bench.sh <musl 二进制路径> probe-device --device /dev/vda
VM_DISK_PREALLOCATE=1 bash scripts/devices/vm-bench.sh <musl 二进制路径> cell --device /dev/vda \
  --arm yi-m --k 2 --group-commit on --warmup-seconds 0.3 --duration-seconds 6 --seed 5 \
  --file-count 10000 --family f8a
```

単体テスト：`cargo test --release -p e7-index-bench --bin e159-fsync-wait-group-commit` → `55 passed; 0 failed`（2026-09-24のヘッダ 307 → 311 の再実行で1件追加）。ミューテーション：`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e159-fsync-wait-group-commit e159_fsync_wait_group_commit.rs ../mutations/e159_fsync_wait_group_commit.tsv` → 17/17 全て検出（M15/M16/M17 を追加）、締めの「復元済み、ベースラインは依然全て緑」（2回実行、ソース変更の前後で各1回、結論は不変）。

### 答えられないこと

- **分岐表1行目の反転観測にはまだ答えがない**：正式計時（5ラウンドのメイングリッド × 3アーム × 8個の k、PC1–PC3、AA、G8、G1、G6）は1セルも実施しておらず、J1（甲がある k で M を超える）/J2a/J4（p99 がある k で M を超える）の3つの判定基準にはまだ値がない。この区間が示したのは、装置が実走で通ること、バイト計測条件が `crates/` と合っていること、ミューテーション表が全て検出することだけである。
- **S4 の書き込み量予算（事前登録5.7）は精算できていない**：断片的なスモーク読み値があるだけ（例えば甲 k=4、duration 3s で約58.5 MB を書き込み、換算で約17.7 MB/s。ただしこれはスパースディスクで初めてセクタに触れ、「測定前にディスク全体を1回書く」手順を踏んでいない状態で測った値であり、本当にウォームアップした後のスループットは明らかに高い可能性があり、実際の総バイト数はこの値で過小評価されているかもしれない）；第1節の全セル（メイングリッド24セル + PC1/PC2/PC3/AA/アンカー + G8 24セル + G1 12セル + G6 8セル、各セル11sまたは2s、5ラウンド）をセルごとに積み上げて正確な予算総量を算出していないため、1 TiB の停止ラインが発動するかも確認できていない。次の区間で正式計時を走らせる前に、この勘定を先に精算するか、ウォームアップ済みのディスクで代表セルを実測して推算する必要がある。
- **「測定前にディスク全体を1回書く」（事前登録5.3）手順はまだ装置に実装されていない**：この区間のスモークテストは `VM_DISK_PREALLOCATE=1`（`fallocate` による事前割り当てで、全面書き込みではない）のみを使用したため、このパラメータなしの最初のスモークテストは samples=0 に遭遇した（バグではなく、スパースディスクで初めてセクタに触れるときのホスト ext4 メタデータのオーバーヘッド）；read_at の修正後、「ディスク全体を1回書く」手順そのものは検証していない。
- **G8 オープンループ・ポアソン到達、G1 非共有、G6 `T_time=200ms` の3つの逆方向サンプリング点はまだ走らせていない**ため、第8節の「方向が逆のサンプリング点」による判別力の自己検証も行っていない。
- **PC1/PC2/PC3/AA の4つの陽性対照とノイズフロアは正式には一度も走らせていない**（非公式のスモークテストのみ：乙-M-K10 のセルが実チェックポイントを1回発動させ、チェックポイント機構が通ることを示したが、PC系列の正式な判定ではない）。

### 影響を受ける決定

この区間は装置とスモークテストのみを渡し、計時結論はない；「影響を受ける決定」表の各項目の「根拠」の段落はいずれも E159 (並行数に伴うfsync待機時間：グループコミットとWALの2アーム)を引用していない（各決定ファイルを `grep -n` で調べ、ヒット0）ため、フォーマット進化規律に従いすべて「準備」と記録し、正式計時が終わってから、支持または反証に格上げするかを見直す。

| 決定項目 | 関係 | 見直し |
|---|---|---|
| D23 (ジャーナルの役割と形式)確定項目1 | 準備 | 2026-10-15 影響なし：丸ごと転記した成果物行を成果物へのポインタに置き換えた。数値、結論、成果物は不変 |
| D16 (パブリッシュセマンティクス)確定項目2 | 準備 | 2026-10-15 影響なし：丸ごと転記した成果物行を成果物へのポインタに置き換えた。数値、結論、成果物は不変 |
| D16 (パブリッシュセマンティクス)確定項目5 | 準備 | 2026-10-15 影響なし：丸ごと転記した成果物行を成果物へのポインタに置き換えた。数値、結論、成果物は不変 |
| D16 (パブリッシュセマンティクス)確定項目7 | 準備 | 2026-10-15 影響なし：丸ごと転記した成果物行を成果物へのポインタに置き換えた。数値、結論、成果物は不変 |
| D16 (パブリッシュセマンティクス)確定項目10 | 準備 | 2026-10-15 影響なし：丸ごと転記した成果物行を成果物へのポインタに置き換えた。数値、結論、成果物は不変 |
| D16 (パブリッシュセマンティクス)確定項目11 | 準備 | 2026-10-15 影響なし：丸ごと転記した成果物行を成果物へのポインタに置き換えた。数値、結論、成果物は不変 |
| D25 (対象ワークロードの優先順位)確定項目7 | 準備 | 2026-10-15 影響なし：丸ごと転記した成果物行を成果物へのポインタに置き換えた。数値、結論、成果物は不変 |

## 履歴バージョン

### 2026-09-24：レコードヘッダ 307 → 311 の再実行（アンカーとスモークテスト、問い表5行目）

- 変更前：装置定数 `RECORD_HEADER_BYTES` は依然307で、`crates/singlefs-format/src/lib.rs` は既に311だった。`H` が装置に入る経路は1本だけ（`NAMED_ITEMS_PER_RECORD_MAIN`）で、しかもレコード件数の経路にしか影響しない；`anchors` 成果物（書き込み量アンカー）は待機時間の計時結論とは全く無関係である —— 今回は計時区間を走らせない。
- 変更後：`e159-r1-prereg.md` で判定基準を固定した後に定数を311へ変更し、剰余33を固定する単体テストを1件追加した（加算の形で書く。理由は E155 (永続化毎の書き込み量：3種のfsync形態と反実仮想上限)の第5回再実行の登録と同じ）；`anchors` 成果物はバイト単位で同一（既存の `e159-fsync-wait-group-commit-2026-09-24-anchors.out` ともバイト単位で同一で、新ファイルには差し替えていない）。`e155_fourth_run_group_commit_concurrency.rs` と共有する4つの書き込み量関数（`solve_fixed_point_core`、`solve_jia_batch_publish`、`solve_write_ahead_log_batch_fsync`、`solve_write_ahead_log_batch_checkpoint`）のバイト単位 diff は空で、S2 は発動しなかった。実機スモークテストの再実行：S3 プローブは依然 `logical_block_size=512` を報告；甲/wal_full-K10/乙-M-K10 の3アームそれぞれ1セルを実行し、終了コードは全て0、結果行は揃い、`emitted=` も合っている（`samples=0` —— 測定ウィンドウ内に warmup 後へ着地した fsync がなく、計時とは無関係で、この数は判定しない）、新しい存証は `e159-fsync-wait-group-commit-2026-09-24-h311-smoke.out`。55単体テスト全て緑（1件追加）、17件のミューテーション全て検出（M15/M16/M17 を追加、定数変更の前後で各1回実行、無効0、赤にならなかったもの0）。**問い表5行目の今回の範囲（書き込み量の半分とスモークテスト）は「不変」と記録する**；待機時間の半分は依然未測定で、計時区間（マシンが空いたときに別途割り当てる）に残す。
- 根拠：`e159-r1-prereg.md`；`m2-header311-rerun-questions.md` 5行目；D23 (ジャーナルの役割と形式)確定項目4。

### 2026-09-25（ゲート69号のための成果物の補完）

- 2026-09-25、ゲート69号のために成果物を補完した（バイト単位で同一）：前回の再実行の `anchors` 成果物は「既存の成果物とバイト単位で同一」と判定されたが新しい成果物を別途保存しておらず、`e159-fsync-wait-group-commit-2026-09-24-anchors.out` の mtime が装置ソースを311に変更した時刻より前だったため、ゲート69号が赤になった。今日、`replay.sh` 内のこの実験の登録行を再実行し、登録行が当時指していた成果物とバイト単位で一致した；`e159-fsync-wait-group-commit-2026-09-25-h311-replay.out` として別途保存した（旧成果物とバイト単位で同一、旧成果物は削除せず残す）。`replay.sh` の当該行の登録は新ファイルを指すように変更した；装置も結論も変更しない。
