#!/usr/bin/env bash
# 用法：setup-differential.sh <仓根> <冻结副本的拷贝根>
# 把 fcc4d340 的 readable_record_of_the_same_publish_follows 与 replay_journal（改名加 _before）抽出来，
# 与同目录 tail.rs 拼成 crates/singlefs-core/src/opus_differential.rs，在 recovery.rs 末尾挂成 cfg(test) 子模块。只改拷贝，不碰仓。
set -euo pipefail
repo=$1
tree=$2
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
old=$(mktemp)
git -C "$repo" show fcc4d340:crates/singlefs-core/src/recovery.rs > "$old"
# 两段的起止行在 fcc4d340 上现查：readable_… 2941–2956，replay_journal 3057–3229（起行是 fn 那一行，止行是收尾的 }）。
test "$(awk 'NR==2941' "$old")" = 'fn readable_record_of_the_same_publish_follows('
test "$(awk 'NR==3057' "$old")" = 'pub fn replay_journal('
test "$(awk 'NR==3229' "$old")" = '}'
out="$tree/crates/singlefs-core/src/opus_differential.rs"
{
    echo '// 生成的文件：fcc4d340 的两个函数（改名加 _before）+ tail.rs'
    awk 'NR>=2941 && NR<=2956' "$old" | sed 's/^fn readable_record_of_the_same_publish_follows(/fn readable_record_of_the_same_publish_follows_before(/'
    awk 'NR>=3057 && NR<=3229' "$old" \
        | sed -e 's/^pub fn replay_journal(/fn replay_journal_before(/' \
              -e 's/readable_record_of_the_same_publish_follows(record, records)/readable_record_of_the_same_publish_follows_before(record, records)/'
    cat "$here/tail.rs"
} > "$out"
rm -f "$old"
grep -q 'mod opus_differential;' "$tree/crates/singlefs-core/src/recovery.rs" || \
    printf '\n#[cfg(test)]\n#[path = "opus_differential.rs"]\nmod opus_differential;\n' >> "$tree/crates/singlefs-core/src/recovery.rs"
echo "生成 $out（$(wc -l < "$out") 行）"
