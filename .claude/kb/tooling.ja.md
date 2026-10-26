# ツールと環境の事実

[中文](tooling.md) · [English](tooling.en.md) · **日本語**

**tooling.md には「本プロジェクトで使用するツールが現在どのような状態にあるか、使用時にどう作業するか」を記録する**。
ルールは tooling.md には入れず、[CLAUDE.md](../../CLAUDE.md) または `.claude/rules/` に記載する。

## ローカル LLM（三者論証の脚としては派遣しない）

三者論証ではクラウドアシスタント2脚＋メイン agent の判定のみを用い、ローカルモデルの脚は派遣しない。リポジトリ内のゲートウェイ呼び出しスクリプトおよび破損チェッカー（`ask-local.sh`、`ask-local-selftest.sh`、`local-repro.sh`、`local-repro-tuned.sh`、`corruption-check.py`、`corruption-summary.py`、`oov-check.py`、`en-words.txt`）は 2026-10-11 に削除された。再度必要な場合は、`git log --diff-filter=D -- scripts/agents/ask-local.sh` で削除コミットを特定し、その直前のバージョンから復旧すること。

| 項目 | 値 |
|---|---|
| エントリポイント | 本機のローカルモデルゲートウェイ（OpenAI 互換。アドレスと所在ディレクトリはリポジトリに入れない） |
| 認証 | リクエストヘッダー `X-Api-Key`、値は本機の非公開設定にあり、リポジトリに入れない |
| モデル | `local` → `Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`、max_model_len 262144 |
| 呼び出しスクリプト | リポジトリ内から削除済み（復旧手順は本節冒頭の段落を参照） |
| 予算 | "max_tokens" および "thinking_token_budget" を渡さない |

使用時の注意事項：

- プロンプトは必ず英語を用い、Markdown 強調（`**bold**:`）は使用しない。`presence_penalty` や `repetition_penalty` による中国語の重複修正には依存しない。
- 提示された具体的なフィールド名、構造体名、機構の説明はすべて未検証の手がかりとして扱い、検証が通らなければ項目全体を破棄する。
- 最も陥りやすいカテゴリー錯誤をプロンプト内で明示して禁止する：「2つの異なる媒体上にある同一システム」を「2つの異なるシステム」と答えてはならない。「幅は別箇所で定義される」場合は「延期されたのは幅であり、フィールドそのものではない」と明記する。
- 「例示できない」と回答された場合は陽性対照（positive control）を実行する：同一プロンプトの質問対象のみを答えが「存在する」と分かっている対象に差し替え、今回例示できた場合に初めてその「例示できない」を有効な回答として認める。

## リポジトリ内の性能数値は本マシン上でのみ成立する。引用にはマシンの計測条件を付記すること

**適用範囲**：実経過時間（wall-clock）、スループット、パブリッシュあたりミリ秒数、ゲート実行時間、コールドマウント秒数、並列高速化比など、時間および速度に関わるすべて。

**作業手順**：

- 性能数値を引用する際は**マシンの計測条件**（どのマシンか、どのディスクか、どのような負荷か、アイドルかビジーか）を必ず併記する。計測条件を欠く数値を判定基準に入れてはならない。
- ローカルマシンの数値を用いて**閾値、上限値、タイムアウト**を定める場合は、「これはローカルマシンの値であり、他のマシンでは再計測が必要」と明記する。
- マシン横断で成立する数値が必要な場合は、2台以上のマシンで計測するか、または条件付き記述（「X を前提に計算すれば Y となるが、その帯域はローカルで測定できず仮説に留まる」）とする。

ゲート層における全リングゼロクリア所要時間、コールド再起動リカバリ所要時間、クラッシュポイント再生2ストリームの実経過時間、E152 (マイルストーン別の六ファイルシステムとのファイル性能比較) および6種との全比較数値はいずれもローカルマシンの数値である（C486 (全リングゼロクリアの 55 秒とコールド再オープンの 17 秒が未監視)）。

## タイムアウト下限をアイドル状態のマシンで調整すると、「マシンが他タスクで高負荷」を「変異を検出できない」と誤認する

- 並列度に対象ユニット自身が占有するコア数を掛けた値が、コア総数を超えてはならない。本リポジトリで cargo-mutants を実行する際は `--jobs 1` を使用する。
- タイムアウト下限は負荷がかかったマシンで調整する。正確に調整できない場合はタイムアウトを設定しない。
- タイムアウトに達した項目は単独で列挙し、検出・未検出（赤にならない）・無効のいずれの列にも合算しない。報告時は4つの数値をすべて報告する。
- 下限値にはマシンの計測条件を付記する（「リポジトリ内の性能数値は本マシン上でのみ成立する。引用にはマシンの計測条件を付記すること」の節を参照）。

## 変異件数はソースコードに伴って変動するため、KB 内で固定値として書き固めてはならない

本文には件数だけでなく、計測コマンドを記述する：

```
cargo mutants --list --package singlefs-checker | wc -l
cargo mutants --list --package singlefs-core | wc -l
```

具体的な件数を引用する場合は、計測日とともに出来事の文（「<日付> に N 件を計上」）として記述し、現在の恒常状態を表す文としては書かない。

## 参照元を変更した場合は、リポジトリ全体を grep して参照先を更新すること

表現を1箇所修正した後、「すべて修正完了」とする前に、まずリポジトリ全体を `grep -rn` して件数を確認する。範囲には `kb/` の全サブディレクトリ、`research/prompts/`（三者論証に与える背景資料）、`scripts/` および `research/` 配下のソースコードコメント、`records/`、ゲートスクリプトが含まれる。確認しきれない場合は「これらの数箇所を修正したが、全リポジトリの検証は未実施」と記載する。

## 自身が不正と判定する構成から数値を算出してはならない

- 上限または下限を算出する前に、その構成が本プロジェクトの判定基準において正当であることを確認する。
- 数値を撤回する際は、実際の破棄理由（入力欠落、不正な構成、計測条件の誤り）を記述し、残された他の構成にも同様に当てはまる性質を理由として記述しない。

## 未定項目は必ず番号付き項目とし、見出しには必ず件数を明記する

`^\d+\.` で抽出するスクリプトは番号付き項目のみを認識する。`.claude/gate.d/doc-decisions.sh` の kb-shape スロットによる件数チェックは、見出しで「N 項目未定」と宣言された場合にのみ作動する。

## 検証成果物の配置先：4箇所の配置場所がそれぞれの段階を管轄

| 配置場所 | 格納するもの | 読み取り対象 | リポジトリ登録 |
|---|---|---|---|
| `.claude/kb/experiments.md` | 結論、計測条件、判定基準、再現コマンド、**決定的となる数行の数値** | モデル検索 | 登録する |
| `records/` | 各ラウンドの記述：何を決定したか、何を覆したか、その根拠 | 人間による通読 | 登録する |
| `scripts/` | 実験コードおよびスクリプト | 実行担当者 | 登録する（`.gitignore` で除外するのは `target/` および一部の超大容量ファイルのみ） |
| `research/results/<実験>-<日付>.out` | 生の出力全文 | 次回再実行時の行単位比較 | 登録する。最新ラウンドより前のものは次回コミット時に git 履歴へアーカイブ（ゲート doc-experiments の archive-past-rounds スロット） |

- 決定的な数値は KB 本文に転記する。
- 各生出力は、それを出力した実行によるものであることを自証できなければならない：最終行の `E7RESULT name=done emitted=N` における N がファイル内の `E7RESULT` 行数と一致している必要があり、一致しない場合はラウンド全体が無効となる。アーカイブ時に確認する：
  `for f in research/results/*.out; do echo "$(grep -c '^E7RESULT' $f) vs $(grep 'name=done' $f)"; done`

## カーネルソース：ローカルに完全なツリーが存在するため WebFetch は不要

| 項目 | 値 |
|---|---|
| パス | `~/kbuild/linux-om`（1.8 GB） |
| バージョン | **Linux 7.2.0**（`Makefile` の `VERSION = 7` / `PATCHLEVEL = 2` / `SUBLEVEL = 0`） |
| 含まれるもの | 完全な `.c`/`.h` 本文（`fs/jbd2/`、`fs/xfs/`、`fs/btrfs/`、`fs/erofs/`、`fs/ext4/`、`fs/f2fs/`、`drivers/nvme/`、`drivers/md/`、`crypto/`、`Documentation/` を含む） |
| **含まれないもの** | **`fs/bcachefs/` はこのツリーに含まれない**（[decisions.md](decisions.md) D7 (Linux メインラインへのマージ是非)を参照）。`/usr/src/linux-headers-6.17.0-1028-oem/fs/bcachefs/` には `Kconfig` + `Makefile` のみが存在し、**ソースコードは存在しない** |

- ソースコードの調査には `~/kbuild/linux-om` を使用し、`/usr/src/linux-*-headers-*`（`Kconfig` と `Makefile` のみ）は使用しない。
- 引用するツリーのバージョン番号を明記し、2つのバージョンの結論を混同して引用しない。KB 内で「Linux 6.17 を都度調査」と記されている引用は GitHub の `v6.17` タグに由来し、本マシンにはそのバージョンは存在しない。
- OpenZFS のツリーは本マシンに存在しないため、`https://raw.githubusercontent.com/openzfs/zfs/master/...` から都度取得し、引用時には取得日を記録する（行番号は恒久的なものではない）。

## 指定時刻の自動作業開始：remind.sh で登録する

指定時刻にセッション自身で自動的に作業を開始させる場合は、`bash scripts/agents/remind.sh start --every <分> --message <リマインド内容> --once` のみを使用する（Bash の `run_in_background` で起動し、指定時刻に終了してセッションを起床させる。ルールは `.claude/rules/timer-only-remind.md`、記法は `.claude/skills/remind/SKILL.md` を参照）。CronCreate や、タイマー目的のみの Bash `sleep` は使用しない。リマインド内容には作業指示が置かれているパスを記載し、最初の手順で `set -o noclobber` により「作業開始済み」フラグを排他的に作成する。フラグが既に存在する場合は何もしない（`remind.sh` 再起動時に重複通知される可能性があるため）。

- リマインダーは現在のセッション内でのみ有効であり、ユーザーに対してセッションを閉じず、マシンをスリープさせないよう伝えること。

## Rust ツールチェーン

| 項目 | 値 |
|---|---|
| バージョン | rustc 1.98.0 / cargo 1.98.0 |
| インストール方法 | rustup、`--profile minimal --default-toolchain stable`、ホストの `~/.cargo` に導入 |
| 追加ターゲット | `x86_64-unknown-linux-musl`。実験バイナリ（`scripts/experiment-apparatus/` 配下のパッケージ）を静的リンクでコンパイルし、VM の busybox initramfs 上で実行する。checker 層の実デバイスバイナリ `new_pool_file_creation_on_device` はこれを使用せず（checker 層はデフォルトで RocksDB の C++ をビルドするが、本ホストには musl の C/C++ コンパイラがない）、`--target x86_64-unknown-linux-gnu` に `+crt-static` を付与して static-pie としてビルドする（コマンドは `.claude/gate.d/checker-tier-qemu-device-streams.sh` を参照） |
| コンポーネント | `rustfmt` + `clippy`（`--profile minimal` には含まれないため個別導入が必要） |
| **PATH** | rustup の shim シンボリックリンクが PATH 上のユーザー単位の実行ファイルディレクトリに配置されている（cargo / rustc / rustup / rustdoc / rustfmt / cargo-fmt / cargo-clippy）。ゲートおよびスクリプトは `bash -c` で起動され シェルの起動ファイルを読み込まないため、PATH は dotfile 変更に依存しない |
| release オーバーフロー検査 | ルートの `Cargo.toml` および `scripts/experiment-apparatus/Cargo.toml` の `[profile.release]` に `overflow-checks = true` を設定済み。profile はワークスペースルートでのみ認識され、メンバーの manifest に記述しても効果がない |

## RocksDB と snappy のプリビルトライブラリ：ホストごとに一度だけビルドし、値はプロセス環境だけで渡す

librocksdb-sys の C++（`librocksdb.a` と `libsnappy.a`）はホストごとに一度だけビルドし、各ビルドディレクトリは環境変数でこの 1 つを参照する。ビルドディレクトリごとにその場でビルドすることはしない。ツールは `scripts/hosts/rocksdb-prebuilt.py` で、使い方はファイル冒頭に書かれている（`--help` は認識せず、終了コード 2 で終わる）。サブコマンドはすべてリポジトリのルートで実行し、`--root R` を省略するとこのスクリプトが置かれているリポジトリを指す。

| サブコマンド | 内容 |
|---|---|
| `key` | このホストの現時点のキー（各構成要素）とプリビルトライブラリのディレクトリを表示する。ビルドはしない |
| `env` | プリビルトライブラリが存在し一致するとき、4 行の `export` を出力する（`eval` 用）。存在しなければ終了コード 1 で、標準出力は空 |
| `build` | プリビルトライブラリがなければ一度だけビルドする。ロックを取る：複数のプロセスが同時に来ても一度だけビルドし、後から来たものはロックを待ってビルド済みを確認すると終了コード 0 で終わる。本物の cargo を起動する重い作業で、`bash scripts/hosts/capped.sh <N>` 経由で起動する |
| `install` | `build` を行い、続けてこのホストの 3 か所の環境の供給元を書き込み、ユーザー単位の systemd マネージャーの現在の環境を更新し、最後に `check` を実行する。こちらも重い作業 |
| `check` | このホストを判定する：2 つのワークスペース（`crates/singlefs-checker-tier/Cargo.toml` と `scripts/experiment-apparatus/Cargo.toml`）の依存が一致し、プリビルトライブラリが存在して一致し、3 か所の環境の供給元と現在のプロセス環境の値がいずれもプリビルトライブラリと一致し、cargo 設定にこの 4 つの変数が書かれていない |
| `audit-target <ビルドディレクトリ>…` | ビルドディレクトリを判定する：librocksdb-sys のビルドスクリプトがそのとき受け取った値がこのホストのプリビルトライブラリか、`out/` にその場でビルドされた C++ がないか、同じキーの下で何種類の値が見られたか。ビルドスクリプトの記録が 1 つも見つからなければ終了コード 77（今回は未判定） |
| `entry [--cargo C]` | cargo を起動する入口の冒頭で呼ぶ軽い判定（誰が呼ぶか、値が欠けたときの対処は「cargo を起動する入口」の段落に書いてある） |

**プリビルトライブラリの置き場所**：`${SINGLEFS_ROCKSDB_PREBUILT_STORE:-~/.local/share/singlefs/rocksdb-prebuilt}/librocksdb-sys-<バージョン>/<キーの先頭 16 桁>/lib/{librocksdb.a,libsnappy.a}`。同じディレクトリの `manifest.json` にキーの各構成要素と 2 つのライブラリの sha256 を記録し、`complete` を最後に書く（これがあって初めてビルド済みとみなす）。起動時に消去されるディレクトリには置かない。`SINGLEFS_ROCKSDB_PREBUILT_STORE` は自己検証で一時ディレクトリに切り替えるためだけに使う。キーは、2 つのワークスペースの rocksdb 依存の書き方、2 つの `Cargo.lock` にある関連パッケージのバージョンと checksum、`rustc -vV`、c++ / cc / ar の実パスとバージョン、C++ に渡す命令セット、ビルド方法から成り、どれか 1 つでも変わるとキーが変わり、ビルドし直しが必要になる。

**値の出どころ**：4 つの変数 `ROCKSDB_LIB_DIR`、`ROCKSDB_STATIC`、`SNAPPY_LIB_DIR`、`SNAPPY_STATIC` の値は 3 か所の環境の供給元に書かれ、すべて `install` が書く：

| 場所 | 役割 |
|---|---|
| `~/.config/singlefs/rocksdb-prebuilt.env` | 唯一の値の供給元：4 行の `export` と `unset ROCKSDB_COMPILE`、`unset SNAPPY_COMPILE`。手で編集せず、変えたら `install` をやり直す |
| `~/.bashrc` の先頭のひとまとまり（3 行、`# >>> singlefs rocksdb-prebuilt >>>` から `# <<< singlefs rocksdb-prebuilt <<<` まで） | その env ファイルを `source` する。対話判定より前に置かれているので、対話シェル、そこから起動したセッション、ssh で起動した非対話コマンドのいずれも読み込める |
| `~/.config/environment.d/60-singlefs-rocksdb-prebuilt.conf` | ユーザー単位の systemd マネージャーが起動するユニット（`scripts/hosts/detached-run.sh` で起動した長時間ジョブ）が読む。`install` は `systemctl --user set-environment` も実行し、いま動いているマネージャーにその場で値を反映させる |

**4 つの変数はプロセス環境だけで渡す**：`.cargo/config.toml` やほかの層の cargo 設定の `[env]` には書かない（書いてあると `check` が赤と判定する。理由は `.cargo/config.toml` 冒頭のコメントにある）。`ROCKSDB_COMPILE` と `SNAPPY_COMPILE` は一切設定しない。1 にするとビルドディレクトリでその場でビルドされる。

**cargo を起動する入口**：ゲートのステージ、クラッシュとミューテーションのドライバ、`merge-and-verify.sh`、`admission.py` が渡すコマンド、二台目の側は、いずれも冒頭で `entry` を呼ぶ。判定するのは 3 点だけ：このホストに導入済みであること（プリビルトライブラリが存在して一致する。2 つのライブラリの sha256 は計算しない）、現在のプロセス環境の 4 つの値がプリビルトライブラリと一致すること、`ROCKSDB_COMPILE` と `SNAPPY_COMPILE` が設定されていないこと。3 か所の環境の供給元は判定しない（それは `check` の仕事）。値が欠けている、または一致しないときは、入口がその場で拒否し、cargo を一切起動せず、失敗した項目ごとに対処を書き出す。対処：現在のプロセスに値がない場合は、新しいセッションを開く（`install` の後に起動したログインシェルは `~/.bashrc` の先頭のひとまとまりを読む）か、コマンドの前に `eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"` を付ける。このホストにまだ導入していない場合は、先に `bash scripts/hosts/capped.sh <N> python3 scripts/hosts/rocksdb-prebuilt.py install` を実行する。ビルド対象のツリーに rocksdb に依存するワークスペースがない場合（librocksdb-sys をビルドできない。ゲートのサンプルや自己検証の小さなリポジトリはたいていこれ）、`entry` は判定せず終了コード 0 で終わる。

## VM ベンチマークハーネス（E7 (オフラインインデックスharness)で使用）

| 項目 | 値 |
|---|---|
| スクリプト | `bash scripts/devices/vm-bench.sh <静的バイナリ>` / `--selftest` |
| 概要 | プロジェクト独自の VM 装置：VM に virtio ディスクをアタッチし（スパースファイルは `TMPDIR` に配置）、バイナリを initramfs に同梱する |
| 完全性ゲート | テスト対象プログラムの終了行が `name=done emitted=N` を報告し、ホスト側で捕捉した件数が N と完全に一致しなければならず、不一致の場合はラウンド全体を無効とする |
| デフォルト仕様 | メモリ 2048M / 4 vCPU / ディスク 4096M / タイムアウト 900s（すべて環境変数で上書き可能） |

- 結果行の捕捉は行頭に固定しない（コンソールの先頭行の前に BIOS エスケープシーケンスが存在するため）。件数は完全性ゲートで検証する。
- VM 内のベアデバイスにおけるシーケンシャル書き込みスループットは不安定であるため、いかなる判断の入力値としても使用しない。
- バッチスクリプトでラウンドを無効化する際は、当該ラウンドの出力末尾も併せて記録する。

---

## 異なる幾何を持つデバイスの構築：実ハードウェアは不要

本マシンは異種デバイスプールを構築するすべての手段を備えており、[decisions.md](decisions.md) D12 (対象メディア)の確定項目 3「デバイスレベルの幾何をどこに置くか、異種プールをサポートするか」を実測検証できる。

### QEMU ルート（推奨：root 不要、既存ハーネスに直接接続可能）

`zoned.zone_size` および `logical_block_size` はコントローラーではなく **`nvme-ns`（ネームスペース）** に紐付いており、同一コントローラー配下の複数ネームスペースが異なる幾何を持つことができる。

実測検証済み（QEMU 8.2.2、終了コード 0、3つのデバイスすべて正常構築完了）：

```
-device nvme,serial=A,id=nv0
-device nvme-ns,drive=d1,bus=nv0,nsid=1,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d2,bus=nv0,nsid=2,zoned=true,zoned.zone_size=128M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d3,bus=nv0,nsid=3,logical_block_size=512,physical_block_size=512
```

1つのコマンドで以下を同時に構築できる：**異種 zoned プール**（64M vs 128M zone）、**512e/4Kn 混在**、**zoned と通常の混在**。

- `logical_block_size` には必ず `physical_block_size` を併記する必要があり、前者のみを指定すると拒否される：`logical_block_size > physical_block_size not supported`。
- ゲストカーネルから3つの異なる幾何が見えているかは未検証：ゲストに入って `/sys/block/*/queue/{logical_block_size,chunk_sectors}` および `blkzone report` を確認する必要がある。

### ホスト側ルート（root が必要だが、VM を起動せずに済むのが利点）

| 手段 | 構築対象 | 主要パラメータ |
|---|---|---|
| null_blk | zoned デバイス、メモリバックエンド | `zoned=1 zone_size=<MB> zone_capacity zone_nr_conv zone_max_open`。モジュールパラメータはグローバルであるため、**デバイスごとの個別幾何は configfs を経由する**（`/sys/kernel/config` は本マシンでマウント済み） |
| zloop | **ファイルバックエンド**の zoned デバイス | モジュールは `drivers/block/zloop.ko.zst` にあり、本マシンに存在 |
| scsi_debug | 任意のセクタサイズ + ZBC | `sector_size=<bytes> zbc=host-managed zone_cap_mb zone_nr_conv` |

実行中カーネル 6.17.0-1028-oem の関連設定：`CONFIG_BLK_DEV_ZONED=y`、
`CONFIG_BLK_DEV_NULL_BLK=m`、`CONFIG_BLK_DEV_ZONED_LOOP=m`、`CONFIG_SCSI_DEBUG=m`。

⚠️ 本マシン唯一の実ブロックデバイスは `nvme0n1`（3.6T、512/512、非回転型）である——
**本物の回転型ディスク、本物の 4Kn ディスク、本物の zoned ディスクは存在しない**。すべての関連数値はシミュレーションによるものであり、引用時には必ずこの計測条件を明記すること。

## scripts/mutation/mutate.sh の結末分類

- 結末：`✅ [検出]`（赤になった）/ `❌ [未検出]`（テストの盲点）/ `⏭ 変異によるコンパイル失敗（無効な変異。盲点にも命中にも計上しない）`。その他に `⏱ [タイムアウト]`、`🧱 [メモリ上限衝突]`、パッケージング順待ち不能、全体メモリ上限による圧殺、VRAM 不足、およびアンカー破損（`⚓ [アンカー破損]`：原文の完全一致がちょうど1回でなかった行は実行せず判定ライブラリに入れない。他の行は通常通り実行され、ラウンド全体は終了コード 3 を返す）の各分類があり、それぞれ単独で列挙する（全結末は `scripts/mutation/mutate.sh` のファイルヘッダーおよび `--selftest` を参照）。
- 判定ツールを変更した場合は、正解が判明しているデータセット全体でリグレッションテストを行い、新たに追加した数項目のみで済ませてはならない。

## 複数セッションで単一リポジトリを共用する場合のステージングとコミット手順

ゲートが赤になった場合は、誰の変更によって赤になったのかを切り分けてから着手する。ゲートを緑にする目的だけで他者のインフライトのファイルを変更してはならない。ルールは上流の `session-wrapup.md` 第4節にあり、本リポジトリでの手順は以下の通り：

| 項目 | 作業手順 |
|---|---|
| 自身のハンクのみをステージング | `scripts/gate/stage-mine.py --match 自身のマーカー --foreign 他者のマーカー ファイル…` を実行。まず `--dry-run` でどのハンクが選択されたか確認する |
| ステージング領域（インデックス）も共用 | ステージング前に `git diff --cached --name-only` が空であることを確認。ステージング完了後、コミット前に対象一覧とファイルごとの cached diff を再確認し、パス指定なしで `git commit` を実行する |
| `git commit -- <パス>` を使用しない | パスを指定した場合、git はステージング領域ではなく作業ツリー内の内容をコミットしてしまう |
| 同一行を双方が変更した場合 | stage-mine が「ステージング領域に入るハンク内に --foreign に一致する行がある」と報告し、1行を分割できない場合：相手がそのファイル内で行った変更が可逆的な置換（名前変更）のみであれば、作業ツリー版において相手の置換を逆適用し、`git show HEAD:ファイル` との diff で自身の変更のみが残っていることを確認した上で、`git hash-object -w` + `git update-index --cacheinfo 100644,<sha>,<パス>` でステージング領域に格納し、作業ツリーは変更しない |
| worktree 上でのコミット | worktree（HEAD 上）で今回のコミット対象となる全変更をスクリプトで再現 → メイン作業ツリーとファイル単位で比較（差異部分はすべて相手のものであるべき。比較時は `git -c core.quotePath=false` を指定すること。さもないと非 ASCII パスがエスケープされ比較ツールがすべて不一致と判定する）→ worktree 上でゲート全体を実行してコミット → メインリポジトリで `git update-ref refs/heads/master <新コミット> <旧 HEAD>` を実行後に `git read-tree <新コミット>` を行い、作業ツリーは変更せず相手の変更をそのまま残す |
| 今回コミット対象でない変更の識別 | `git -c core.quotepath=false status --short` を使用する。非 ASCII パスはデフォルトで8進数エスケープされるため、キーワード検索で漏れが生じる |
| 同一ファイル内に双方の変更が混在 | すべてを1つのコミットにまとめ、コミットメッセージにその旨を明記する（コミットを分割すると、ゲート doc-registries の experiment-refs スロットおよびゲート doc-experiments の experiment-orphans スロットが赤と判定される） |

## 再実行および変異スクリプトは scripts/experiment-apparatus/ 配下で実行すること

- `scripts/mutation/mutate.sh` は `scripts/experiment-apparatus/` 配下で実行する：`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh <bin 名> <ソースファイル> ../mutation/tables/<ミューテーション表>`。
- `replay.sh` は E 付きの実験番号（`E` に数字が続く形式）のみを認識し、数字単体は終了コード 2 となる。
- `replay.sh` は実行ごとに `${TMPDIR:-/tmp}/singlefs-replay-build-<プロセス番号>-XXXXXX/target/` に自前のビルドディレクトリを作ってそこへビルドし、終了後に削除する（呼び出し側が `CARGO_TARGET_DIR` を設定した場合はそこへビルドし、削除しない）。研究バイナリは `$REPLAY_RELEASE_DIRECTORY`（`$CARGO_TARGET_DIR/release/`）から起動する。`scripts/experiment-apparatus/target/` には再実行でビルドしたバイナリはもう置かれない。

## サブ項目を一括作成／確定した際にゲートが赤（エラー）になるパターン

一連のサブ項目を一括作成または確定した後、コミット前にゲートの以下の各スロットが次のパターンで赤になる。記述時は「正しい書き方」列に従うこと：

| ゲート | 赤になるパターン | 正しい書き方 |
|---|---|---|
| doc-registries の first-txn-hooks スロット（3ファイルの連動） | `\| N \|` 行で未定項目を分割するため、最終行がそれ以降の `#### 未定項目 N` の本文すべてを自身の項目として取り込んでしまう。本文内に「新規プールでのファイル作成のバイト変更：是（」と書かれていると、「否」と判定される最終行も「是」として扱われる | インデックス行は規範形式 `新規プールでのファイル作成のバイト変更：是（日付、根拠：…）` を用い、本文の小見出しは `**新規プールでのファイル作成のバイト変更**：是（…）` とする |
| doc-decisions の item-ref-status スロット（参照の帰属） | 行内の単独の「確定項目 k」が同一行内の直近の `D<n>（略称）` に帰属される | サブ項目を新設した後は doc-decisions の item-ref-status スロットを実行し、逆転した単独参照に決定名を補完する。決定本文内で他の決定のサブ項目を参照する場合は、同一行内で自身の決定名を重複して明記する |
| doc-lint | F に数字が続くようなレビュー番号が未登録の番号として扱われる。略称の後ろに読点注釈を付加すると「名前不一致」と判定される | 「発見事項 1」「提案 2」と記述する。括弧注記は別途括弧を起こし、「番号（略称）（条文策定済み）」と記述し、略称の括弧内に押し込まない |
| doc-registries の table-shape スロット（未済の検査テーブルの形状） | 新たな6列の未済の検査の行がファイル末尾に追加され、そこが「返済済み」の4列表となっている。未返済テーブル内のセル先頭に「返済済み（…）」や「帳消し」と書かれている。または未返済テーブルの途中に空行が挟まっている | 第1テーブルの最終行の後ろに追加する：最初の5本縦線の `\|---\|` の前、`\| C` で始まり縦線が7本以上ある最終行の後ろ。全額返済された場合は4列の返済済みテーブルへ移動し、一部のみ返済されたものは先頭に「半額返済」「決定確定・検査未済」「前提返済済み」と記載する。テーブル途中に空行を設けない |
| doc-registries の segment-registry スロット（セグメントシーケンス登録表） | `--selftest` において、「mkfs 種まきルート」の行のセグメントシーケンス文字列が `layout/01-first-txn.md` 全体で1回のみ出現することが要求され、同一文字列が他箇所で再度記述されると赤になる | 他箇所の引用は空白を入れた「4 + 1 + 1 + 4」の形式で記述する |
| ゲート doc-decisions の変更履歴3スロット（entry-added、shape、history-loss） | 「（其のN）」、クイック参照2行、および節先頭の現状行 | 記法と判定ルールは `.claude/rules/changelog-format.md` を参照。件数や番号の脱落は history-loss スロットで判定されるため、変更履歴整理時はステージング前に `scripts/kb/history-consolidate.py loss-check` を実行する |
| code-source-discipline の format-constants スロット（フォーマット定数） | KB タグの値変更後、`scripts/experiment-apparatus/**/*.rs` および `crates/**/*.rs` 内のすべての同名 `const` が追従しなければならない（`research/` 配下は凍结された証拠のみを保持し、走査されるが `.claude/format-constants-exclude` により個別許可される）。`stale=` 列の旧値文字列が KB 本文内に再出現してはならない | 他箇所で依然として正当に出現している文字列（`base = 95` など）を `stale=` に追加してはならない |
| clippy shadow_unrelated | 同一テスト関数内で2回目の `let reader = …` / `let segments = …` が赤と判定される。rustfmt の再フォーマット後に正規表現置換を行うと後半の修正漏れが発生する | 各 reader / セグメント / レポートに異なる命名を行う（`extent_reader`、`warm_up_segments`、`control_report` など） |
| naming-lint | 「英字 + 数字」が単一英字として扱われる：read_u48 が赤、テスト名が a_ で始まると赤。全リポジトリの .rs を走査し、`research/prompts` 内のクラウドアシスタントが書いたモデルソースコードも走査対象となる | `read_six_byte_unsigned` と記述する。`research/prompts` は `.claude/naming-lint-exclude` に登録済み（証拠ディレクトリのため回収対象外）。アシスタントのモデルコードを他ディレクトリに記述した場合は通常通り検査される |
| checker-tier-crash-point-replay（クラッシュポイント再生フル実行） | テストケースに `#[ignore]` が付与され `crates/singlefs-checker-tier/tests/` に配置されている（D13 (検証ルート) 確定項目 15）。`cargo test -p singlefs-checker-tier` で数件の ignored が報告されるのは正常動作である。harness 層の `cargo test -p singlefs-harness` からは到達できない | ゲートの `--full` は release 下で再利用判定に基づき1件ずつ実行する（クイック層ではフル実行をしない）。フル実行の最中に `CARGO_TARGET_DIR` を変更することで、他のテストを同時にコンパイル可能 |
| 変異証明 | rustfmt 後、複数行にわたる呼び出しの文字列全体一致が「不一致」となる | 「キー文字列を含む行を探し、`);` で終わる行までを削除する」手順で行う。変更後は復元し、ファイルが原文とバイト単位で完全一致することを `assert` する |
| checker の新規判定 | 対象が同一検査を既に通過した後にのみ判定を行う判定ロジックは、決して赤（エラー）にならない | 新規判定には必ず既知の不良イメージ（`checker_known_bad_images.rs` の形式）を用意し、クリーンなイメージがすべて緑になることだけを確認してはならない |
| 一括定点置換 | 1件ずつディスクに書き込むと、1箇所のアンカー不一致で中途半端な変更が残存する | `scripts/kb/replace-batch.py` を使用する：すべての置換がメモリ上で「ちょうど1回一致」することを確認してからディスクに書き込む |

## 本マシンの Claude Code セッションにおける grep と find は GNU 版ではない

`type grep`、`type find` を実行するといずれもシェル関数であることが表示される：`grep` の実体は ugrep であり、`find` の実体は bfs である（エラー出力はそれぞれ `ugrep: error`、`bfs: error` で始まる）。GNU 版と異なる挙動は以下の2点：

| 記法 | 結果 | 修正方法 |
|---|---|---|
| マルチバイト行に対する `.\{0,90\}` などの有界反復を伴う `grep -o` | `exceeds complexity limits`、終了コード 2、1行も出力されない | Python のスライスを使用：`line[max(0, i - 90):i + 90]` |
| `find <ディレクトリ> -newermt '-40 minutes'` | `Invalid timestamp`、相対時間を認識しない | ISO 形式の日付を指定：`-newermt '<YYYY-MM-DD>'` |

いずれの失敗時も「そのコマンドが結果を出力しなかった」という外見になるため、パイプや `||` の後ろに接続されている場合に「一致がなかった」と誤読してはならない。
