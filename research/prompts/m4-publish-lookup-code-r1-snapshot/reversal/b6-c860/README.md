# 主工作区 crates/mutations.tsv 的改动清单（供 m4-publish-lookup-code-r1 倒推用）

- mutations-before.tsv：打 C860 丙补丁之前拷的主工作区原样，sha256 99dc56e6387ada5b7c96ba36f0cd5d0f734eb0fc7e2b528f719e1241c5b0c63a，与那一轮开工快照的 crates/mutations.tsv 相同。
- 丙补丁对它只做三件事（原文就是同目录的 mutations-append.tsv、mutations-replacements.tsv、mutations-delete.txt，整行）：
  - 末尾追加 6 行；
  - 按名字整行替换 2 行（原第 4854、4855 行，两行「扫描重建 M5：…」）；
  - 删掉名字是 mutations-delete.txt 那一行的那一条（原第 4958 行）。
- mutations-after-this-patch-only.tsv：只施加这三件事之后的样子；this-patch-only.diff 是它与改前的 diff（三个 hunk）。
- mutations-after-at-1140-with-eb-sysconf.tsv：11:40 拷的主工作区现样，里面还有 eb 同时打进来的系统配置落地那份（主工作区 transaction.rs、mount.rs 的 mtime 是 11:39:53）；before-to-1140-with-eb-sysconf.diff 是它与改前的 diff。eb 那几份的行要找 eb 要原文。
- sha256sums.txt：本目录各文件的 sha256。
