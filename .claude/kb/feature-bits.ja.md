# feature bit 記帳表

[中文](feature-bits.md) · [English](feature-bits.en.md) · **日本語**

3つのビットマップ（incompat / compat_ro / compat）は各256ビットで構成され、システム構成内に保持される。1ビット割り当てられるごとにここに1行追記し、その名称、どのフォーマットバージョンのどのコミットで導入されたか、現在の状態、およびその意味（1文での定義）を明確に記録する。

**ビット番号の唯一の規範的登録箇所は本ファイル内にはなく、D15 (フォーマット凍結ポリシー)確定項目 4 の登録表**（`.claude/kb/decisions/15-格式冻结政策.md`）にある。「何番目のビットが何に割り当てられているか」はその表を正とする。両者に齟齬が生じた場合は `.claude/kb/feature-bits.md` 側を修正し、当該登録表は修正しない。ゲート code-source-discipline の feature-bits スロット（`.claude/gate.d/code-source-discipline.sh`）が両者を1ビットずつ照合し、不一致時は赤と判定する。

割り当て権は承認制ではなく「先マージ先占有」である（D15 (フォーマット凍結ポリシー) 確定項目 10）。**マイルストーン 17 の完了まではフォーマット凍結は存在しない；フォーマット凍結までは旧ディスクが存在しないため、ビット番号は回収・並べ替えしてよい**。ビット番号を変えるときは `crates/` と checker を同じ変更で追従させる；凍結後は一度使用されたビットを回収してはならず、理由は D15 (フォーマット凍結ポリシー) 確定項目 10 にある。

<!-- feature-bits:table -->
1行に1ビットを記述し、7列構成は D15 確定項目 10 で定められた列順に従う。「1文での定義」は D15 確定項目 4 登録表内の同一ビットの「意味」を逐字的に含んでいる必要があり、ゲート code-source-discipline の feature-bits スロットはこの条項に基づき照合する。「名称」にはコード内の定数名を記述し、コード側に定数がない場合は `—` と記載する。

| ビット番号 | カテゴリ | 名称 | 導入バージョン | 導入コミット | 状態 | 1文での定義 |
|---|---|---|---|---|---|---|
| 0 | incompat | — | フォーマットバージョン 1 | 53c9f38 | 空き | 空き、再割り当て可；どのレイアウトも使っておらず、リーダーはこれを見ると未認識ビットとして拒絶する |
| 1 | incompat | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | フォーマットバージョン 1 | 45d49aaf | 使用中 | 最初の純 SSD レイアウト系（システム構成にロールバック下限 F を保持、ルートレコードにアンマウント記号を保持、ロールバック見撃およびロールバック行なし）。mkfs 時からセットされ、このビットを認識しないリーダーはマウント不可 |
| 2 | incompat | INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT | フォーマットバージョン 1 | 0b926bd3f | 使用中 | inode レコードが切り詰めフィールドを持つ（オフセット 120 から T と N_w）、切り詰めフィールドは無条件に持つ；ビット 2 はレイアウト識別の一部であり、mkfs 時からセットされ、リーダーはビット 1 とビット 2 を両方持つものだけを受け入れる（第 1 バイト 0x06） |

**compat_ro および compat の2つのビットマップは1ビットも割り当てられておらず、incompat はビット 1 とビット 2 を割り当て済みで、ビット 0 は空き、他のビットは一切未割り当てである**（出典：D15 (フォーマット凍結ポリシー) 確定項目 4 登録表）。

**コード側で参照されているビット**：`crates/singlefs-core/src/system_configuration.rs` の `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02` がビット 1、`INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT = 0x04` がビット 2。空きのビット 0 にはコード側の定数がない。同ファイルの `INCOMPAT_FIRST_BYTE_OF_THIS_LAYOUT` はビット 1 とビット 2 から合成されており（第 1 バイト 0x06）、ビットマスクリテラルではなく宣言が複数行にまたがるため、ゲート code-source-discipline の feature-bits スロットはビット番号をパースできず、成功メッセージの「未判定ビット番号」リストに計上される。`crates/singlefs-checker/src/lib.rs` は独自の名前付き定数 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`（292行目）と `INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT = 0x04`（295行目、定数名はコア層のものより S が 1 文字少ない）を用いて incompat を判定する。

**ビットマップのエンディアン**：ビットマップはリトルエンディアンであり、ビット n は第 n div 8 バイト目の下位第 n mod 8 ビットに位置する（D22 (ユニット原子性の合成手法)確定項目 13）。
