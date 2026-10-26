#!/usr/bin/env bash
# 新增的两句断言各带一条 crates/demo/tests/reject.rs 里 #[test] 标的用例名：一句「走不到」写在正文里，一句「不可达」写在表格行的同一格里 ⇒ 判绿。
# 另一行「触发不了」是存量清单登记的那一行，从文件尾挪到了表前（diff 里看是新增），与清单逐字相同，按清单放过；
# 译文 .claude/kb/decisions/1-样本.en.md 里新增的同类中文句不判。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/kb/decisions .claude/gate.d crates/demo/tests
printf '#[cfg(test)]\nmod tests {}\n\n#[test]\nfn remount_after_drop_is_rejected() {\n    assert!(true);\n}\n\n#[test]\n#[ignore]\npub fn stale_disk_resync_is_rejected() {\n    assert!(true);\n}\n' > crates/demo/tests/reject.rs
printf '# 决策一\n\n正文。\n\n- 旧说法：第二台掉线那一格触发不了。\n' > .claude/kb/decisions/1-样本.md
printf '# Decision one\n\nBody.\n' > .claude/kb/decisions/1-样本.en.md
printf '# 这一格立起来时 kb 里已有的断言行\n.claude/kb/decisions/1-样本.md\t- 旧说法：第二台掉线那一格触发不了。\n' > .claude/gate.d/unreachable-claims-stock.tsv
git add -A && git commit -qm base
{
  printf '# 决策一\n\n- 旧说法：第二台掉线那一格触发不了。\n\n正文。\n\n'
  printf '掉盘之后挂成可写这一格走不到，`remount_after_drop_is_rejected` 构造它并断言被拒。\n\n'
  printf '| 编号 | 断言 |\n|---|---|\n| 1 | 两盘都旧时重同步不可达（stale_disk_resync_is_rejected） |\n'
} > .claude/kb/decisions/1-样本.md
printf '# Decision one\n\nBody.\n\n这一格不可达。\n' > .claude/kb/decisions/1-样本.en.md
