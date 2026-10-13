# singlefs

[English](README.md) · [中文](README.zh.md) · [日本語](README.ja.md)

---
大きなディスクは会社のもの、よい眠りは自分のもの。
---
**ゼロから設計したコピーオンライト（COW）ファイルシステム。Rust で実装。**

## なぜもう一つ作るのか

私は多くのファイルシステムのコードを読んできて、どれも優れていると思う。けれど、苦しい時代を過ごしてきたせいで、どれも贅沢が足りず、結合が強すぎて、こんなに豊かな時代になっても楽ができない。

だから私たちは楽をすることにした。singlefs は醜くてもいいし、ストレージを無駄にしてもいいし、肥大した実装の分岐をたくさん並べて持っていてもいい。

何年も後に誰かが singlefs のコードを評するとき、「一山のクソだ」ではなく「何本ものクソだ」と言ってほしい。

実装上、一か所を変えると多くの分岐を一緒に変えなければならないかもしれない。でも構わない。もう一本、違う分岐を気軽に足せばいい。たとえそれが一味違う臭さでも。

うん、まだ検証中だけど、たぶんいけると思う。
waaagh!

既存の COW ファイルシステムの痛点のいくつかは**フォーマットレベル**のもので、パッチでは直せず、設計の時点で避けるしかない:

| 痛点 | 根本原因 | このプロジェクトの選択 |
|---|---|---|
| ばかげた ENOSPC（空きがあるのに書けない、ファイルを消しても容量不足と言われる） | 二段階の割り当てと硬直した chunk の分類 | data/metadata を分けず、全体で一つのアロケータ |
| RAID5/6 の write hole | 固定幅のストライプは読み込み・変更・書き込みが要る | ストライプ幅を可変にし、読み込み・変更・書き込みを一切しない |
| クォータの集計が遅くて間違う | 集計が事後の走査で、しかも逆引きインデックスを辿る | 集計はトランザクションの副産物で、コミット時に増分で維持する |
| 逆引きインデックスが複雑さのブラックホールになった | 後から生えてきたもので、設計されたものではない | 初日からフォーマットに組み込む |
| 誰も修復ツールを走らせる勇気がない | 再構築可能性が設計目標ではなかった | メタデータブロックを自己記述にし、checker をフォーマットと一緒に進化させる |

決定とその根拠は [`.claude/kb/decisions.md`](.claude/kb/decisions.md)、落とし穴の一覧は [`.claude/kb/pitfalls.md`](.claude/kb/pitfalls.md) にある。

### 性能の見積もり

数値はすべて実際に走らせた実験から取ったもので、それぞれ自分の計測条件で読み、互いに換算しない:

- フォーマット後に使える量: 16 GiB のディスクで単一ディスクは 95.2%、二つのディスクのミラーは 47.6% が使える。他の六つのファイルシステムの既定フォーマットも同じ水準にある（E152 (マイルストーン別の六ファイルシステムとのファイル性能比較)、表は [`scripts/experiments/perf-by-milestone.md`](scripts/experiments/perf-by-milestone.md)）。
- 小さなファイル: 32 KiB 未満のデータはすべて一つの 32 KiB ユニットにパディングされる（E66 (パディング後に小ファイルがどう配置されるか)）。小オブジェクトのパッキングは D27 (小データパッキングコンテナ) で決まっているがまだ実装されていない。実装後はパディングの無駄が最悪で 11.1% になる（E120 (等比のサイズクラス表で公比をどう決めるか)）。
- メタデータ自身が占める量: 割り当てレコードツリーは 32 KiB の配置先、充填率 90% で容量の約 0.055% を占める（E74 (割り当てレコードのエントリ数と整理コスト)、算術モデル）。

## 現状

現在のマイルストーンは「削除と複数回の COW」（[`.claude/kb/milestone/03-third-txn.md`](.claude/kb/milestone/03-third-txn.md)）。前の「上書き・解放・ロールバック・再利用」は 2026-09-29 に終わり、やり残しは現在のマイルストーンの項目五に移った。

できること: 二台のディスクで mkfs する。ファイルを書いてコミットし、複数のデータユニットを順に埋め、オフセットで読み戻す。一回のパブリッシュで複数のファイルを作る（ディレクトリはない）。上書き、解放、そしてロールバック下限がその位置を越えた後で配置先を再利用する。閉じて二つ目の書き込み可能インスタンスとして再マウントする。マウント中に管理者ロールバックを行う。コールドスタート後に読み戻す。inode 番号でファイルを削除する（まず削除の意図をパブリッシュし、それからデータユニットを何回かに分けて回収する）。

できないこと: システムにマウントできない（FUSE もカーネルモジュールもない）。ディレクトリも POSIX インターフェースもなく、スナップショット、暗号化、圧縮もない。**どんなデータもこれに保存しないこと。** ディスクフォーマットはまだ柔らかく、最初の外部ユーザーが現れるまではいつ作り直されてもおかしくない（[`.claude/rules/format-evolution.md`](.claude/rules/format-evolution.md)）。

検証がどこまで届いているか:

| 検証 | 何を判定するか | ゲート |
|---|---|---|
| クラッシュポイント再生 | 書き込み要求のストリームを各ディスクのバリアで区切り、区間内のあらゆる永続化状態を列挙し、状態ごとにイメージを作ってリカバリとプールレベル checker を走らせる | checker-tier-layer0-replay。フル実行はユーザーの要求時のみ |
| クラッシュ増幅 | 同じ列挙を数十億状態まで広げる。ストリームをまず判定ライブラリに記録し、ライブラリから読み戻して判定する。判定器は GPU 上で動き、二台構成で分割する | `scripts/crash/crash-amplification.sh`。ユーザーの要求時に走らせる |
| プールレベル checker | 不変条件の一覧（`crates/singlefs-checker/invariants/`）で「実装済み」の 92 条。コード中の一覧と同数 | 上の二行から呼ばれる |
| モデル突き合わせ | ランダムな操作列を、メモリ上にだけ住む理想モデルと一歩ずつ比べる | harness-model-differential-and-scenarios |
| QEMU 実デバイス | 二つの virtio ディスク上で固定の負荷を走らせ、デバイス側で記録した書き込みと FLUSH をプログラムが記録したストリームと項目ごとに比べる。クラッシュ注入はない | checker-tier-qemu-device-streams |
| メモリ順序 | herd7 が `litmus/` 以下の各 Never を判定する。それぞれにバリアを外した対照がある | checker-tier-lkmm |
| 形式検証 | Verus が、リカバリ再生で journal のプレフィックスを取り、ルート選択とリングスキャンを行う純関数の核（`crates/singlefs-replay-prefix`）を検証する。製品にコンパイルされるのもこのコードそのものである | checker-tier-formal-verification |
| ミューテーション | crates のミューテーション表は一行ごとに一か所を壊し、指名したテストが赤にならなければならない。広域ミューテーション（cargo-mutants）はすべての変異に結論があり、生き残りが一つもないことを求める | checker-tier-crates-mutation-replay、checker-tier-broad-mutation |

**ゲートが全部緑でも、それはモデル層でのクラッシュ整合性の根拠にすぎず**、「検証済み」ではない。各項目がどこまで走ったか、どれが今回未実行と記録されたか、まだ何を覆っていないかは、[`briefs/`](briefs/) の最新のブリーフを見ること。

## システム構成

マウントは各ディスクのオフセット 0 と 4096 にある二つの 4096 バイトのスロットから読み始める。そこにあるのが**システム構成**で、489 バイトの構造がディスクごとに一つ、二つのスロットを交互に使う。チェーン全体の起点であり（システム構成 → ルートリング → ルートレコード → ツリーテーブル → 各インデックスツリー → データユニット）、ディスク上で唯一救い出せない小さな部分でもある。他はすべて自己記述のユニットからスキャンし直せるが、journal リングがどこにあるか、ユニットの大きさはどれだけか、といったジオメトリは他のどこにも書かれていない。

中身は四種類に分かれる。mkfs のときにユーザーが設定し、変えるにはファイルシステムを作り直す必要がある不変の構成（389 バイト）。変えてもインデックスを再構築するだけでデータは失わない可変の構成（4 バイト）。マウントのたびに設定し直せる実行時の構成（36 バイト）。ファイルシステム自身が維持し、ユーザーが決して設定できない実行時の量（60 バイト。パブリッシュのたびに動くので、定数としてキャッシュしないこと）。フィールド表は [`.claude/kb/layout/01-first-txn.md`](.claude/kb/layout/01-first-txn.md) にある。

## コントリビューターガバナンス（Contributor Governance）

> **Implementation That Favors the AI, Review That Favors the Human（実装は AI に優しく、レビューは人に優しく）.**
>
> **Make every submitted patch review-worthy（すべての提出をレビューに値するものにする）.**
>
> **Contribution throughput may be unbounded; acceptance throughput is evidence-bound（投稿のスループットは無制限でよいが、受入のスループットは根拠に縛られる）.**

ゲートは誰かを締め出すためにあるのではない。すべての提出を、人が時間を割いて見る価値のある線まで引き上げるためにある。機械的な部分はスクリプトに任せ、人の注意は人にしか判断できないところに使う。

**受入基準は自動化された検証である。** このプロジェクトは提出者を出自で区別しない。区別は一つだけで、根拠を伴う提出と、伴わない提出である。根拠の基準は誰にとっても同じ物差しで、送る前にゲートを一度走らせれば、自分がどこに立っているか分かる。

> 開示: このプロジェクトのコード、ドキュメント、ゲートスクリプトは人と AI の協働で作られており、今後もずっとそうである。これは私たち自身のやり方の説明であって、提出者を分類するものではない。

## 貢献

三つの必須要件:

1. **`crates/*/src/` を変えたら必ずテストを付ける。** 例外はなく、「次のパッチで足す」もない。
2. **新しいテストはまず赤になることを示す**: テスト対象のコードを壊し、テストが失敗することを確かめ、元に戻す。どう確かめたかをコミットメッセージに書く。赤になることを示せないテストは書かなかったのと同じ。
3. **未実装の検証は合格のふりをしない。** ゲートはまだ組み込まれていない段階を明示的に列挙する。緑は実装済みの部分が通ったことしか意味しない。

```bash
bash .claude/scripts/gate.sh                                  # ゲート一巡: 共有段階と .claude/gate.d/ のプロジェクト段階
cargo test -p singlefs-harness                                # harness 層: 単体テストと結合テスト。コードを変えたらいつでも走らせる
bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick   # ライト層のクラッシュポイント再生
```

checker 層（クラッシュ列挙ケース、QEMU、herd7、ミューテーション表、形式検証、広域ミューテーション）は既定ではコミット時にだけ走り、コマンドに `SINGLEFS_HEAVY_TESTS=commit` を付ける。クラッシュポイント再生のフル実行とクラッシュ増幅のフル実行はユーザーの要求時にだけ走る。何がヘビー層に数えられるか、ライト層とヘビー層をどう分けるかは [`.claude/rules/verification.md`](.claude/rules/verification.md) にある。

前提: herd7 には `opam install herdtools7` が要り、`SINGLEFS_KERNEL_TREE=` で `tools/memory-model` を持つ Linux のソースツリーを指す。QEMU の段階には KVM と、blklogwrites のパッチを当てた QEMU が要る（`scripts/devices/build-patched-qemu.sh` でビルドしてプライベートなディレクトリに入れ、[`qemu.env.example`](qemu.env.example) に従ってリポジトリのルートに `qemu.env` を書く）。欠けていれば走るのを拒み、システムの QEMU には戻らない。仮想マシンのテストハーネスの前提は [`.claude/kb/vm-harness.md`](.claude/kb/vm-harness.md) にある。

### 複数マシンと GPU（任意）

リポジトリのルートにあるテンプレート [`multi-host.env.example`](multi-host.env.example) に従って、手元に `multi-host.env`（git では無視される）を書く。二台構成での分割、GPU 判定器、GPU カードの優先順と割り当て量、静止化のコマンドがすべてそこにあり、各キーの説明はテンプレートにある。書き終えたら `bash scripts/crash/layer0-shard-configuration-check.sh` で一度判定する。GPU を使うときは必ず先に静止化し、終わっても元に戻さない（`scripts/crash/gpu-quiesce-run.sh`）。結果がファイルになる作業を二台目に移すには `bash scripts/hosts/multi-host-run.sh --peer -- <コマンド>`。各スクリプトの置き場所は [`scripts/README.md`](scripts/README.md) にある。

ゲートスクリプトとルールは singlefs-ai-sop がまとめて配布しており、参加者は全員同じ一式を走らせる。

## 始め方

ルールとゲートは独立したリポジトリ singlefs-ai-sop にある。同じ仕様に中国語、英語、日本語の三つのリポジトリがあり（[`singlefs-ai-sop-zh`](https://github.com/faliye/singlefs-ai-sop-zh)、[`-en`](https://github.com/faliye/singlefs-ai-sop-en)、`-ja`）、対外的には `-en` が正である。このプロジェクトが使うのは `-zh`。プロジェクト内のものは**コピー**で、上流の `.git` と `CLAUDE.md` は含まない:

```bash
git clone https://github.com/faliye/singlefs.git
git clone https://github.com/faliye/singlefs-ai-sop-zh.git
cd singlefs
rsync -a --exclude '.git' --exclude CLAUDE.md ../singlefs-ai-sop-zh/ .claude/singlefs-ai-sop/
diff -rq --exclude=.git --exclude=CLAUDE.md ../singlefs-ai-sop-zh .claude/singlefs-ai-sop   # 何も出力されなければコピーは完全
bash .claude/singlefs-ai-sop/install.sh     # バージョンスタンプ .singlefs-ai-sop-version を更新する
bash .claude/scripts/gate.sh
```

上流がリリースしたら、同じ `rsync` でもう一度コピーし、`install.sh` をもう一度走らせる。**ルールの変更は上流リポジトリで行う**。`.claude/singlefs-ai-sop/` をその場で書き換えてはならない。

## ディレクトリ

| パス | 内容 |
|---|---|
| [`crates/singlefs-format`](crates/singlefs-format) | フォーマット定数。各幅の値は kb の決定の小項目から来る |
| [`crates/singlefs-core`](crates/singlefs-core) | mkfs、アロケータ、トランザクション層、書き込み可能マウントとセッション、リカバリ、マウント中の読み取り、各インデックスツリー、ブロックデバイスのバックエンド |
| [`crates/singlefs-harness`](crates/singlefs-harness) | harness 層: 単体テストと結合テスト、それにレコーダ、メモリプール、理想モデル、ランダム履歴、故障注入といった足場 |
| [`crates/singlefs-checker-tier`](crates/singlefs-checker-tier) | checker 層: クラッシュ状態の列挙、判定ライブラリ、クラッシュ増幅、GPU 判定器、実デバイスのテストハーネス。既定ではコミット時にだけ走る |
| [`crates/singlefs-checker`](crates/singlefs-checker) | プールレベル checker: 一つのイメージを判定するライブラリ。実装と共有するのはフォーマット定数だけ |
| [`crates/singlefs-replay-prefix`](crates/singlefs-replay-prefix) | リカバリ再生でプレフィックスを取り、ルート選択とリングスキャンを行う純関数の核。Verus が検証するのはこれ |
| [`crates/singlefs-reference-filesystem`](crates/singlefs-reference-filesystem) | RefFS: 意味レベルの参照ファイルシステム。メモリ上だけで POSIX 操作に従って動く |
| [`crates/mutations.tsv`](crates/mutations.tsv) | crates のミューテーション表 |
| [`scripts/`](scripts/) | 繰り返し使うスクリプト。用途別のサブディレクトリに分かれ、何をどこに置くかはその README にある |
| [`litmus/`](litmus/) | herd7 の litmus テスト |
| [`.claude/kb/`](.claude/kb/) | 決定と変更履歴、実験、未済の検査、バイトレイアウト表、マイルストーン計画、先行事例の調査と落とし穴 |
| [`.claude/rules/`](.claude/rules/) | プロジェクトローカルのルール。共有ルールは `.claude/singlefs-ai-sop/rules/` にある |
| [`.claude/gate.d/`](.claude/gate.d/) | プロジェクトローカルのゲート段階とその登録表 |
| [`.claude/agents/`](.claude/agents/)、[`.claude/hooks/`](.claude/hooks/)、[`.claude/skills/`](.claude/skills/) | subagent の定義、Claude Code のフック、skill |
| `.claude/singlefs-ai-sop/` | 共有仕様とゲートスクリプトのコピー。git では無視され、「始め方」の手順で取得する |
| [`research/`](research/) | 三者論証の資料と判決（`prompts/`）、実験で保存した成果物（`results/`）。どちらも丸ごと消して取り直せる |
| [`records/`](records/) | 構築の過程 |
| [`briefs/`](briefs/) | 更新ごとのブリーフ。日付ごとに一つで、その版で何ができるか、どこまで検証したか、まだ何を覆っていないかを書く。各ブリーフに英語（既定）、中国語（`.zh.md`）、日本語（`.ja.md`）の三つの版がある |

## ライセンス

デュアルライセンス: [Apache-2.0](LICENSE-APACHE) または [MIT](LICENSE-MIT) のいずれかを選べる。

このプロジェクトに提出され採用された貢献は、Apache-2.0 の定義に従い、追加条件なしに上記のデュアルライセンスで許諾される。
