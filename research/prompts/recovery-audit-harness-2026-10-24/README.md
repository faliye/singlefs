# harness 恢复验证审计

- [详细报告](../../../records/2026-10-24-singlefs-harness-恢复验证审计.md)
- [公开 API 探针](probes.rs)：设备包装 discard、四个重读/缓存接口、模型 effect 兼容性。
- [原始日志](evidence/probes.log)、[库测试](evidence/library-tests.log)、[恢复集成测试](evidence/recovery-integration-tests.log)。
- [本轮形式化核结果](evidence/formal-kernel.json)、[源码摘要](evidence/source-sha256.txt)。

运行：`python3 research/prompts/recovery-audit-harness-2026-10-24/verify_audit.py`。

因果复核：`python3 research/prompts/recovery-audit-harness-2026-10-24/verify_recheck.py`。仅在临时源码副本做定向补检查，运行原版、临时对照、再次原版；使用独立的 `target/harness-causal-recheck` 构建目录。见[复核日志](evidence/causal-recheck.log)、[临时对照补丁](evidence/temporary-control.patch)和[复核源码摘要](evidence/recheck-source-sha256.txt)。临时补丁不作为正式修复方案。

脚本在临时工程调用公开 API，两次运行断言；产品源码及既有测试不改。探针通过表示报告中的缺口成功复现，不能解释为已修复。证明重跑及既有测试命令见报告和对应日志。
