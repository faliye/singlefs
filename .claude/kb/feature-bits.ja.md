# feature bit 記帳表

[中文](feature-bits.md) · [English](feature-bits.en.md) · **日本語**

3つのビットマップ（incompat / compat_ro / compat）は各256ビットで構成され、システム構成内に保持される。1ビット割り当てられるごとにここに1行追記し、その名称、どのフォーマットバージョンのどのコミットで導入されたか、現在の状態、およびその意味（1文での定義）を明確に記録する。

**ビット番号の唯一の規範的登録箇所は本ファイル内にはなく、D15 (フォーマット凍結ポリシー)確定項目 4 の登録表**（`.claude/kb/decisions/15-格式冻结政策.md`）にある。「何番目のビットが何に割り当てられているか」はその表を正とする。両者に齟齬が生じた場合は `.claude/kb/feature-bits.md` 側を修正し、当該登録表は修正しない。ゲート code-source-discipline の feature-bits スロット（`.claude/gate.d/code-source-discipline.sh`）が両者を1ビットずつ照合し、不一致時は赤と判定する。

割り当て権は承認制ではなく「先マージ先占有」である（D15 確定項目 10）。**一度使用されたビットは回収・再利用してはならない**：回収して新機能へ再割り当てすると、古いイメージの該当ビットを新コードが「新機能が有効化されている」と誤認し、ファイルシステムが実際に誤動作を引き起こす（例えば、存在しない暗号化属性を存在するものと誤認して存在しない暗号鍵を探しに行くなど）。退役したビットの状態には「退役」と記載し、その意味は退役直前の最後の定義文言に永久凍結され、書き換えを禁止する。

<!-- feature-bits:table -->
1行に1ビットを記述し、7列構成は D15 確定項目 10 で定められた列順に従う。「1文での定義」は D15 確定項目 4 登録表内の同一ビットの「意味」を逐字的に含んでいる必要があり、ゲート code-source-discipline の feature-bits スロットはこの条項に基づき照合する。「名称」にはコード内の定数名を記述し、コード側に定数がない場合は `—` と記載する。

| ビット番号 | カテゴリ | 名称 | 導入バージョン | 導入コミット | 状態 | 1文での定義 |
|---|---|---|---|---|---|---|
| 0 | incompat | INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT | フォーマットバージョン 1 | 53c9f38 | 退役 | 最初の純 SSD レイアウト系。mkfs 時からセットされ、このビットを認識しないリーダーはマウント不可（D12 (対象メディア)確定項目 1「レイアウト系ごとに1つの incompat ビット」） |
| 1 | incompat | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | フォーマットバージョン 1 | 45d49aaf | 使用中 | 最初の純 SSD レイアウト系（システム構成にロールバック下限 F を保持、ルートレコードにアンマウント記号を保持、ロールバック見撃およびロールバック行なし）。mkfs 時からセットされ、このビットを認識しないリーダーはマウント不可。ビット 0 退役後はビット 0 を検出したリーダーは一律マウントを拒絶 |
| 2 | incompat | INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT | フォーマットバージョン 1 | 0b926bd3f | 使用中 | inode レコードが切り詰めフィールドを持つ（オフセット 120 から T と N_w；ビット 2 を持たないプールではオフセット 120–135 はすべてゼロでなければならない）；mkfs 時からセットされ、書き込み可能マウントはビット 2 を持たない旧プールを番号取得の書き込みからビット 2 付きに引き上げる；リーダーはビット 1 のみ（第 1 バイト 0x02）とビット 1・ビット 2 の両方（0x06）の 2 通りを受け入れる |

**compat_ro および compat の2つのビットマップは1ビットも割り当てられておらず、incompat はビット 0（退役）、ビット 1 とビット 2 を割り当て済みで、他のビットは一切未割り当てである**（出典：D15 (フォーマット凍結ポリシー) 確定項目 4 登録表）。

**コード側で参照されているビット**：`crates/singlefs-core/src/system_configuration.rs` の `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02` がビット 1、`INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT = 0x04` がビット 2。同ファイルの `INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT = 0x01` は退役したビット 0 であり、テストおよびコメントでのみ使用され「使用済みビットは回収しない」規律をコードに定着させるために残されている。`SUPPORTED_INCOMPAT_BITS` はビット 1 とビット 2 から合成されておりビットマスクリテラルではないため、ゲート code-source-discipline の feature-bits スロットはビット番号をパースできず、成功メッセージの「未判定ビット番号」リストに計上される。`crates/singlefs-checker/src/lib.rs` は独自の名前付き定数 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`（276行目）と `INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT = 0x04`（280行目、定数名はコア層のものより S が 1 文字少ない）を用いて incompat を判定する。

**ビットマップのエンディアン**：ビットマップはリトルエンディアンであり、ビット n は第 n div 8 バイト目の下位第 n mod 8 ビットに位置する（D22 (ユニット原子性の合成手法)確定項目 13）。
