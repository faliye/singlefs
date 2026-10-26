# 恢复验证与对拍审计

- [当前复核报告及初稿更正表](../../../records/2026-10-24-singlefs-checker-tier-审计复核.md)
- [三个独立进程的对照日志](evidence/verdict-and-torn-model-recheck.log)
- [实际公开 API 探针](verdict_encoding_and_torn_state_probes.rs)
- [形式化核实测 JSON](evidence/formal-kernel-recheck.json)

运行：`python3 research/prompts/recovery-audit-checker-tier-2026-10-24/verify_audit.py --verus`。

脚本建立临时工程和独立的一次性判定库。探针中的夹具路径及覆盖消费者方法由脚本从仓库当前位置注入，不依赖固定用户名路径，不修改产品实现。

checker 的完整镜像漏检和 journal 对照见[对应目录](../recovery-audit-checker-2026-10-24/README.md)。历史初稿也已移至 `records/`，已由当前复核报告取代。
