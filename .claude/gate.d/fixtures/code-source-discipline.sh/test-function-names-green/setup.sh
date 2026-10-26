#!/usr/bin/env bash
# 绿样本现场（test-function-names 格）：一条按测的对象起名的单测，一条登记了豁免的 c456_ 存量。
set -e
mkdir -p .claude/gate.d crates/demo-core/src
printf '#[cfg(test)]\nmod tests {\n    #[test]\n    fn c456_listed_while_it_is_read() {}\n\n    #[test]\n    fn refuses_one_unit_when_one_slot_short() {}\n}\n' > crates/demo-core/src/admission.rs
printf '# 样本豁免表\ncrates/demo-core/src/admission.rs\tc456_listed_while_it_is_read\t# 样本：存量，待逐条读\n' > .claude/gate.d/test-function-names-exempt.tsv
