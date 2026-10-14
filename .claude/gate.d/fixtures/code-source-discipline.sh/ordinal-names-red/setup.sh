#!/usr/bin/env bash
# 红样本现场：ordinal-names 那一格——一份 .rs 里一个 enumerate_ 接层序号的函数（注释里再提一次，注释那一处不算），一份 .sh 给带流序号的变量赋值再展开，
# 排除表一行指向已经改好名的文件、一行指向不存在的文件。
# 源文件与词表由这里现写，仓里的样本目录本身不带 .rs / .sh 源文件；词表里的两个词由几段拼进变量，这份 setup.sh 里不出现它们的字面
# （真仓上这一格也扫 .claude/gate.d 下的 .sh，字面写进来这份 setup.sh 自己就成了违规）。
set -e
layer_number_word="layer""0"
stream_number_word="stream""_0"
mkdir -p .claude crates/demo/src scripts/demo
printf '%s  # 样本：按层序号叫崩溃点重放\n%s  # 样本：按流序号叫一条负载\n' "$layer_number_word" "$stream_number_word" > .claude/naming-ordinal-tokens
printf 'crates/demo/src/renamed.rs  # 样本：已经改好名还登记着\ncrates/demo/src/gone.rs  # 样本：指向不存在的文件\n' > .claude/naming-ordinal-exclude
printf 'fn enumerate_%s() {}\n// 注释里的 %s 不归这一格判\nfn main() { enumerate_%s(); }\n' "$layer_number_word" "$layer_number_word" "$layer_number_word" > crates/demo/src/main.rs
printf 'fn enumerate_crash_states() {}\n' > crates/demo/src/renamed.rs
printf '#!/usr/bin/env bash\n%s=1\necho "$%s"\n' "$stream_number_word" "$stream_number_word" > scripts/demo/run.sh
git init -q . && git add -A
