# 样例工具页

跑 `bash .claude/gate.d/doc-text.sh --check prime-marks,term-renames` 只重跑那两格；写法照 `bash .claude/gate.d/<门禁>.sh --check <格名>`。
门禁 harness-test-environment 排在跑测试的门禁之后。
对照表在 `records/2026-09-28-门禁59号提速与双机分片.md`「旧编号与现在的门禁」一节（记录文件名里带的号不是在称呼门禁）。

## 历史版本

- 那时叫门禁 77 号，判的是跑完测试之后机器干不干净（`.claude/gate.d/77-test-environment.sh`）。
