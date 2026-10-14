#!/usr/bin/env bash
# 绿样本现场：ordinal-names 那一格——词表里的两个词只出现在 .rs、.sh、.py 的注释与字符串里，一个名字是层序号后面紧跟一位数字（另一个号，不算），
# 排除表登记的那一份 .rs 还带着层序号（还没改完，登记了就不判）。
# 源文件与词表由这里现写；词表里的两个词由几段拼进变量，这份 setup.sh 里不出现它们的字面（理由同红样本那一份）。
set -e
layer_number_word="layer""0"
stream_number_word="stream""_0"
mkdir -p .claude crates/demo/src scripts/demo
printf '%s  # 样本：按层序号叫崩溃点重放\n%s  # 样本：按流序号叫一条负载\n' "$layer_number_word" "$stream_number_word" > .claude/naming-ordinal-tokens
printf 'crates/demo/src/pending.rs  # 样本：还没改完名\n' > .claude/naming-ordinal-exclude
printf 'fn enumerate_crash_states() {}\n// 注释里提到旧名 %s 不归这一格判\nfn main() { let note = "%s 写在字符串里"; let %s1_count = 1; enumerate_crash_states(); }\n' \
  "$layer_number_word" "$stream_number_word" "$layer_number_word" > crates/demo/src/main.rs
printf 'fn enumerate_%s_pending() {}\n' "$layer_number_word" > crates/demo/src/pending.rs
printf '#!/usr/bin/env bash\n# 注释里的 %s= 不判\nnew_pool_file_creation_stream=1\necho '"'"'%s= 写在单引号里'"'"' "$new_pool_file_creation_stream"\n' \
  "$stream_number_word" "$stream_number_word" > scripts/demo/run.sh
printf 'import os\n# %s 写在注释里\nlabel = "%s"\nthreads = os.environ.get("SINGLEFS_CRASH_POINT_REPLAY_THREADS", "1")\n' \
  "$layer_number_word" "$stream_number_word" > scripts/demo/judge.py
git init -q . && git add -A
