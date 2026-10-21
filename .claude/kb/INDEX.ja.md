# kb 索引

[中文](INDEX.md) · [English](INDEX.en.md) · **日本語**

**ルールは kb に入れず、[CLAUDE.md](../../CLAUDE.md) に置く。** kb には事実、決定、調査、実測データのみを置く。

| ファイル | 内容 |
|---|---|
| [decisions.md](decisions.md) | 決定インデックス: 番号、略称、状態、進捗、本文へのリンク |
| [decisions/](decisions/) | 決定ごとに一つのファイル（`NN-略称.md`）、本文と論証を収める |
| [decisions-history.md](decisions-history.md) | すべての決定の変更履歴 |
| [../../crates/singlefs-checker/invariants/README.md](../../crates/singlefs-checker/invariants/README.md) | 不変条件一覧、カテゴリ別に同ディレクトリの 9 ファイルに分割。README に説明、状態、件数登録を置く。checker はその実行可能形式 |
| [prior-art.md](prior-art.md) | 他社方式の調査、出典と計測条件を含む |
| [pitfalls.md](pitfalls.md) | 避坑リスト。設計の決定を行うたびに確認する |
| [experiments.md](experiments.md) | 実験インデックス: 番号、略称、状態、本文へのリンク |
| [experiments/](experiments/) | 実験ごとに一つのファイル（`NN-略称.md`）、測定対象、判定基準、計測条件を収める |
| [experiments-history.md](experiments-history.md) | すべての実験の変更履歴 |
| [checks-owed.md](checks-owed.md) | 未済の検査: 何を止めるべきか分かっているがまだ止められない検査、前提を含む |
| [pending-rules.md](pending-rules.md) | 審議中の規則: 赤になる検査にできず共有 SOP から取り下げられた条項、原文をそのまま残し規則にするため何が不足しているかを明記 |
| [tooling.md](tooling.md) | ツールと環境の事実: ローカル LLM、Rust ツールチェーン、QEMU harness の現状と不足 |
| [vm-harness.md](vm-harness.md) | 実験を VM に送り**実ブロックデバイス**上で走らせる方法: 三つの前提、衛生検査、VM 内で追加される検証経路、CPU 機能をマスクするティアの既知解答ゲート、デバイス側の独立記録（blklogwrites）、追加ルートとディスクイメージの事前割り当て、計時とブロック層カウントの計測条件 |
| [verification-build.md](verification-build.md) | 三つの検証手段（checker、トランザクション層、クラッシュポイント再生）の実装方針: それぞれどの確定条項を消費し、どの未確定項目に阻まれているか、research/ から何を引き継げるか、初版の最小範囲、当時のユーザー決定待ち問題（T1（通常層 壁時計予算）のみ未確定）、クラッシュ増幅のフローとユーザー決定の二つの層 |
| [layout/](layout/) | 各マイルストーンで新しく書き出されるバイト列、マイルストーンごとに一つのファイル: `01-first-txn.md` 新規プールでのファイル作成（各セグメント・フィールドが決定項目を指し、幅と値を与える。セグメントシーケンス登録表もここにある）、`02-second-txn.md` 上書き・解放・ロールバック・再利用で新しく書き出される形態、`03-third-txn.md` 削除で新しく書き出される形態 |
| [milestone/](milestone/) | マイルストーン計画、マイルストーンごとに一つのファイル（`NN-略称.md`）: 各ステップで実装を想定する内容、予想される詳細、受入基準、書き出されるバイト列、遭遇する決定ポイント。当時の判断に基づいて記述し、各ステップ着工前に見直す |
| [feature-bits.md](feature-bits.md) | feature bit 台帳: incompat / compat_ro / compat の 3 つのビットマップ各 256 ビット、1 ビット割り当てるごとに 1 行追加。ビット番号の唯一の登録場所は D15 (フォーマット凍結ポリシー) の確定項目 4 |
| [freeze-layer-membership.md](freeze-layer-membership.md) | 凍結層所属登録表: 構造体ごとに 1 行、D15 (フォーマット凍結ポリシー) の確定項目 7 のどの層に属するか、権威状態か派生状態か、凍結を離脱するか否か |
| [term-renames.md](term-renames.md) | 用語改名対応表: 旧名と新名の唯一の権威ある対応表、他の場所ではすべて新名を使用 |
| [translation-glossary.md](translation-glossary.md) | 翻訳用語集: 中国語・英語・日本語の三言語を並記、ルート README および `briefs/` の三言語版はその用語に従う |

## 識別子ではない記号

`doc-lint:not-numbers` の行にある記号は識別子と同じ形式（英大文字 + 数字）をしていますが、
これらはドメイン用語であり、本プロジェクトの識別子ではないため、略称を付ける必要はありません。同種の用語を追加する場合はその行に加え、ゲートを変更しないでください。
`F41` も同様に CRC-32C の Castagnoli 多項式 `0x1EDC6F41` の下位 4 桁を切り出したものです（D18 (ブロックに含める情報) の確定項目 17 がこのパラメータを固定しています）。

<!-- doc-lint:not-numbers L1 L2 L3 AES-256 AES256 SHA256 SHA-256 SHA512 CRC32 CRC32C CRC64 RAID5 RAID6 OCFS2 Z3 T10 M1 M2 M3 M4 M5 M6 M7 M8 M9 M12 M13 M25 M26 F41 -->

`M` に数字が続くもの（`doc-lint:not-numbers` 行の `M1`〜`M9`、`M12`、`M13`、`M25`、`M26`）は**ミューテーション表の行名**であり、本プロジェクトの識別子ではありません。それらの登録場所は
`scripts/mutation/tables/<bin>.tsv` です——各表の第 1 列で、バイナリと 1 対 1 に対応します。
これらを参照するときは所属するバイナリを明記してください（例:「`e39_back_chain` の `M8_退回第一版哈希`」）。
**同じ `M8` であっても表ごとに指す対象が異なる**ためです。
