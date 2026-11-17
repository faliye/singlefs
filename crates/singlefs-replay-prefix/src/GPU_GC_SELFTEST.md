# GPU GC 20 项新增形式化验证的严格自测

本轮自测发现并修复了两处**契约漏检**。它们不是本轮发现的产品故障：实验改坏的是模型，原契约未拒绝错误模型。覆盖边界仍以 [GPU_GC_PROOFS.md](GPU_GC_PROOFS.md) 为准。

## 先构造错误，再补强契约

任务 `12857` 在修复前分别应用下面两个变异，两次 Verus 都返回 `verified 186、errors 0`，因此反例门禁正确判为失败：

| 错误模型 | 为什么是错误 | 原来漏在哪里 | 修复 |
| --- | --- | --- | --- |
| `record_contribution` 将每条记录的贡献截顶为 2 | 同盘未释放记录 `[0,3)` 对段 `[0,3)` 应贡献 3，截顶后只有 2 | G11 只约束零值和正负；G09/G10 只约束同一贡献函数的分片与重数；见证贡献恰好为 2 | G11 增加同盘活记录的贡献恰等于交集长度 |
| 成功 `round_state` 直接保留旧 `cpu_calls` | 条目已有一次分歧，再次分歧且本轮成功时应恰增加一次 CPU 调用 | G04 只约束失败轮次；G02 检查调用条件，没有约束累计状态 | G04 增加成功轮次的计数表与 CPU 调用精确更新；G05 进一步比较交换次序后的完整状态和调用总数 |

这两个实验均已加入 `verus-must-red-worlds.tsv`，名称为 `gpu-gc-audit-score-capped-at-two` 和 `gpu-gc-audit-successful-cpu-call-lost`。原 G04 变异的锚点也已收窄到失败分支的完整后置条件，避免补强契约后匹配歧义。

## 实测

任务编号由本机 `task-server/task` 分配，可用 `task-server/task log <编号>` 查对应原始输出。以下不是只检查证明数量：反例要求真实证明错误及指定函数落点，编译失败、求解资源耗尽或进程被杀均不算抓到。

| 检查 | 结果 | 任务 |
| --- | --- | --- |
| 修复后 GPU GC 模块，`--no-cheating --rlimit 10` | 26 条义务通过，0 错误 | `12858` 的完成尝试 |
| `singlefs-replay-prefix` 全包证明及信任边界检查 | 170 个验证项重新执行，义务合计 186，0 错误，复用 0；禁止标记、axiom、未登记可信声明均为 0 | `12871` |
| 产品编译与 `verus --compile` 的 MIR 对比 | 产品 109 个条目全部存在，规整后的整段内容逐字相同 | `12871` |
| 24 个必红反例：G01–G20、两个见证、两个审计反例 | 全部重新执行并捕获；编译有效，证明错误落点匹配，未用资源耗尽冒充证明错误；复用 0 | `12872`–`12877`，末批取第 2 次完成尝试 |
| 8 处入口 `requires` 空真筛查 | 全部在入口插入的 `assert(false)` 处报错，未检出空真；复用 0 | `12878`、`12879` |
| 固定工具链与依赖来源 | Verus 版本、commit、归档 SHA256、6 个包的版本/出处/源码指纹及两种 rustc 版本均与登记一致 | `12882` |
| 全宽算术及仲裁对拍 | 4 通过，0 失败、忽略；用 u128 独立参照核对预算、溢出、片界、准入、双核累计和回读 | `12880` |
| 真实主机仲裁适配层 | 2 通过，0 失败、忽略 | `12860` |
| 既有 CPU 测试替身端到端对拍 | 11 通过，0 失败、忽略 | `12860` |
| 四个主机模块回归，含 WGSL 验证器检查 | 56 通过，0 失败、忽略；其他模块的 112 项未纳入本次范围 | `12881` 第 2 次完成尝试 |
| 真 GPU 活槽统计测试 | 6 通过，0 失败、忽略、过滤；含双核对拍、跨 32 位槽界、小显存分段/分片及预算过低时排除设备 | `12861` |
| 真 GPU 区域记录分类测试 | 6 通过，0 失败、忽略、过滤；含双核与 CPU 对拍、小显存切批、映射分片及单元分片 | `12861` |

修复后的 `12877` 明确捕获了修复前能通过的两个错误模型：打分截顶的证明错误落在 G11，漏记 CPU 调用落在 G04。任务的首次尝试因内存上限中断，未计为抓到；这里只采用任务服务自动重试后的完整成功结果。

普通测试合计 85 项通过（4 + 2 + 11 + 56 + 6 + 6）。`12881` 的首次编译也因内存上限中断，只计自动重试后的结果。真 GPU 任务此前因设备占用排队，最终在第二台的卡 1（`0000:05:00.0`）完成；每个用例均有 `GPUS_HANDED_OVER_BY_THE_TASK_SERVER` 记录，日志没有 `NOT_RUN_NO_GPU_ADAPTER` 或 `GPU_DEVICE_LOST`，不是无设备返回成功。预算过低的用例按设计验证排除设备的路径，不声称该用例派发了着色器。

证明、反例和入口空真检查通过项目提供的 `prune_server_client_selftest.py --with-temporary-service` 使用临时判定库，仍使用仓根计算的标准库名及真实固定版本 Verus。临时服务初始没有判定记录，日志逐条记下“现判、复用 0”。没有修改判器、放宽求解限制或启用破坏判器的测试开关。

例如，不复用既有判定地重跑本次发现的两个反例：

```sh
task-server/task submit --manual -- python3 scripts/prune-server/prune_server_client_selftest.py \
  --with-temporary-service -- python3 scripts/gate/verus-proof-check.py must-red-worlds . \
  --item must-red:gpu-gc-audit-score-capped-at-two \
  --item must-red:gpu-gc-audit-successful-cpu-call-lost
```

按任务服务确认单核对项数与范围后确认；工具链与内存、线程额度均由现有入口管理。

真 GPU 两组测试的复跑入口：

```sh
task-server/task submit --manual -- cargo test -p singlefs-background-gpu-tests --offline \
  --features tests-on-a-real-gpu \
  --test live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference \
  --test region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference -- --nocapture
```

新增主机测试在 [gpu_gc_formal_contracts_through_the_host.rs](../../singlefs-background-gpu/tests/gpu_gc_formal_contracts_through_the_host.rs)：穷举 `7^6 = 117649` 条六步事件历史，涵盖两条目的相符/分歧、失败批、空批和重新创建挂载状态，逐步检查按键计数、决策、CPU 回调次数及批统计。另检查 4 种初始状态 × 4 种相符组合 × 2 种顺序，共 32 个批次，核对回调位置和按键结果。参照使用每条目一个布尔未决状态，不调用被验证的仲裁函数来计算预期值。

## 结果的边界

上述有限历史枚举验证实际 Rust 主机接口，不能替代任意长度历史的精化证明。MIR 一致说明产品编译包含被验证的可执行核，不代表外围容器、WGSL、驱动或硬件已形式化验证。八个直接调用的产品核与十二个抽象模型的区分保持不变。
