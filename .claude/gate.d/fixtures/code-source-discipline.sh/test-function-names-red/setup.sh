#!/usr/bin/env bash
# 红样本现场（test-function-names 格）：src/ 里一条 c123_ 起头的单测、tests/ 子目录里一条以 _c789 收尾的用例、豁免表里一行对不上（c999_ 早改了名），三处点名判红；
# 登记了豁免的 c456_ 不报。源文件由这里现写，仓里的样本目录本身不带 .rs（真仓上这一道扫 crates/，本来也扫不到这里）。
set -e
mkdir -p .claude/gate.d crates/demo-core/src crates/singlefs-harness/tests/common
printf '#[cfg(test)]\nmod tests {\n    #[test]\n    fn c123_two_devices_refuse_one_unit() {}\n\n    #[test]\n    fn c456_listed_while_it_is_read() {}\n\n    #[test]\n    fn refuses_one_unit_when_one_slot_short() {}\n}\n' > crates/demo-core/src/admission.rs
printf '#[test]\nfn helper_checked_by_c789() {}\n' > crates/singlefs-harness/tests/common/mod.rs
printf '# 样本豁免表\ncrates/demo-core/src/admission.rs\tc456_listed_while_it_is_read\t# 样本：存量，待逐条读\ncrates/demo-core/src/admission.rs\tc999_renamed_long_ago\t# 样本：早改了名，这一行该删\n' > .claude/gate.d/test-function-names-exempt.tsv
