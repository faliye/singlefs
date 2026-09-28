#!/bin/bash
# A9「除了行数、备注之外都算改了」：只改一行注释，一条 harness 档测试的结局就翻了（它读 model.rs 的原文，注释里的字也读）。
# 同一份仓副本上先原样跑一次，打上 model-comment.patch 再跑一次，换回原样；两次的词法单元摘要由 rust_tokens.py 另比。
# 用法：bash comment_flip.sh <仓副本> <编译目录> <输出目录>
set -uo pipefail
model="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
copy="${1:?仓副本}"; target="${2:?编译目录}"; out="${3:?输出目录}"
mkdir -p "$out"
test_name=model_comparison::tests::the_model_module_uses_only_the_standard_library_and_the_format_constants
run_test() {
  (cd "$copy" && CARGO_TARGET_DIR="$target" nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 \
    cargo test --offline -p singlefs-harness --lib -- --exact "$test_name") > "$out/comment-flip-$1.log" 2>&1
  echo "$?"
}
cp "$copy/crates/singlefs-harness/src/model.rs" "$out/model.rs.pristine"
before=$(run_test pristine)
patch -s -d "$copy" -p1 -i "$model/model-comment.patch"
python3 "$model/rust_tokens.py" file-digests "$out/model.rs.pristine" "$copy/crates/singlefs-harness/src/model.rs" > "$out/comment-flip-digests.out"
after=$(run_test commented)
cp "$out/model.rs.pristine" "$copy/crates/singlefs-harness/src/model.rs"
token_before=$(awk 'NR==1' "$out/comment-flip-digests.out" | grep -oE 'tokens_doc_as_comment=[0-9a-f]+')
token_after=$(awk 'NR==2' "$out/comment-flip-digests.out" | grep -oE 'tokens_doc_as_comment=[0-9a-f]+')
flipped=0
if [ "$before" = 0 ] && [ "$after" != 0 ] && [ "$token_before" = "$token_after" ]; then flipped=1; fi
echo "E7RESULT name=r3_a9_comment_flip test=$test_name exit_pristine=$before exit_comment_only=$after token_digest_same=$([ "$token_before" = "$token_after" ] && echo true || echo false) must_be_nonzero=$flipped"
