# E142（第一个事务的干跑） 第十一次跑 F1 诊断的留存

报告原件是 `research/prompts/e142-r11-f1-diagnosis.md`（与诊断员草稿目录里那份 sha256 `209eb5af842779e379dc72b31f52b472cf221280ff11ba17cb6c074d7107cb00` 逐字节一致）。报告附录列的文件，拷进这个目录的是：`cmp.py`、`verify.py`、`verify-device.txt`、`verify-impl.txt`、`snapshot.sha256`，以及五份跑出来的原样输出（`impl-snapshot.out`、`device-run.out`、`impl-snapshot-devcontent.out`、`device-run-vs-devcontent.out`、`device-implcontent-vs-impl.out`）。

没拷的：副本 `repo/`、六个 `dump-*` 目录（21 个区域的整段字节，每个约 344 KiB）、两个构建目录。它们按报告第一节与第六节的做法在副本上换一行文件内容就能重新生成；报告里引的字节值都在上面那几份输出里。
