# singlefs-harness 恢复、形式化验证与对拍审计

审计日期：当日。对象：`/home/fy5090/code/singlefs/crates/singlefs-harness` 当前工作区，包含已有未提交修改；不是仅审查 HEAD 或某次提交的差异。本轮只新增审计材料，没有修改产品实现和既有测试，也没有执行提交。

方法参照 [checker 审计目录](../research/prompts/recovery-audit-checker-2026-10-24/README.md) 与 [checker-tier 复核报告](2026-10-24-singlefs-checker-tier-审计复核.md)：保留可执行探针、源码摘要、原始日志，区分实现错误、接口条件、明示范围及尚未证明的影响。没有把上一轮 checker/checker-tier 的问题重复算成 harness 新问题。

## 结论

确认三项具体问题，其中两项是设备包装契约问题，一项是模型判定入口的防漏检问题。**本轮没有复现正常历史中恢复到错误内容，也没有发现 Verus 证明失败。** 三项问题的强度不同，不能笼统称为“三项数据丢失漏洞”。

| 编号 | 优先级 | 位置 | 已证实的行为 | 影响成立的条件 |
| --- | --- | --- | --- | --- |
| H01 | P2 | `read_tally.rs:194` | 加读日志包装后，支持的 discard 变成 Unsupported，内存盘数据保留 | 使用此包装器执行 discard |
| H02 | P3，条件性兼容错误（复核由 P2 下调） | `lib.rs:429`、`fault_injection.rs:1069`、`read_tally.rs:194` | 三种包装器丢失四个专用读/缓存通知接口；专用重读变成普通读 | 内层设备覆盖这些接口；当前常用裸 SparseBlockDevice 内层不产生差异 |
| H03 | P3 | `model.rs:7275` | 冷启动答案可与空 Publishes 配对并计为一次成功，完全不交读回结果 | 调用者/胶水交错 effect；当前冷启动执行器交的是正确类型 |

P2 表示应修复验证装置中的具体行为错误；P3 表示条件性接口兼容及判定入口补强。没有 P0/P1 结论。修复应各自独立验收，不能把本目录“复现成功”的绿测试当作产品已经修好。

## 范围和执行材料

重点跟踪的链路是：

```text
history 生成/执行 → model_comparison 生成观测 → IdealModel 判定
录制流 → MemoryPool / CrashImage → core 恢复 → 恢复内容 oracle
故障注入 → 重开 → 理想模型 / RefFS / 外部 checker 结论合并
崩溃镜像恢复 R1 → 可写挂载/续作 → 恢复 R2 → 挂载透明逐字段比较
core 扫描/重放胶水 → singlefs-replay-prefix 形式化核
```

定向阅读了 `history.rs`、`model.rs`、`model_comparison.rs`、`fault_injection.rs`、`memory_pool.rs`、`mount_transparency.rs`、`reference_filesystem_translation.rs`、`segments.rs`、`lib.rs`、`read_tally.rs` 的相关路径，并追查 core 中的设备接口、恢复调用及形式化核。`gpu_memory_ledger.rs` 的显存调度和所有集成用例没有逐行穷尽审查，因此本报告不提供整个 crate 无缺陷的背书。

审计材料集中于 [recovery-audit-harness-2026-10-24](../research/prompts/recovery-audit-harness-2026-10-24/README.md)：

- [公开 API 探针](../research/prompts/recovery-audit-harness-2026-10-24/probes.rs)：H01–H03 的正反对照。
- [重放脚本](../research/prompts/recovery-audit-harness-2026-10-24/verify_audit.py)：临时 Cargo 工程、两次独立测试进程、结束时核对源码未变。
- [探针日志](../research/prompts/recovery-audit-harness-2026-10-24/evidence/probes.log)、[库测试日志](../research/prompts/recovery-audit-harness-2026-10-24/evidence/library-tests.log)、[恢复集成测试日志](../research/prompts/recovery-audit-harness-2026-10-24/evidence/recovery-integration-tests.log)。
- [源码 SHA-256](../research/prompts/recovery-audit-harness-2026-10-24/evidence/source-sha256.txt)：相关六个 crate 的 Rust 源文件，排除 audits 文件；不是工作区所有文件的摘要。
- [本轮 Verus JSON](../research/prompts/recovery-audit-harness-2026-10-24/evidence/formal-kernel.json) 与 [stderr](../research/prompts/recovery-audit-harness-2026-10-24/evidence/formal-kernel.stderr)。

探针重放：

```bash
python3 research/prompts/recovery-audit-harness-2026-10-24/verify_audit.py
```

脚本需要本地 Rust 和离线依赖缓存，编译产物放在仓库 target，临时工程结束后清理。它不改产品文件、不接入产品自动测试，也不要求 GPU 或 RocksDB 判定库。

本轮实际执行结果：

| 检查 | 结果 |
| --- | --- |
| harness 库测试 | 292 passed，0 failed，1 ignored |
| 8 个恢复相关集成测试目标 | 合计 82 passed，0 failed，0 ignored |
| 本轮三组审计探针 | 两个独立进程各 3 passed；包含 SparseBlockDevice 的 discard 字节对照 |
| Verus 形式化核 | 78 verified，0 errors，success=true |

既有测试的重放命令：

```bash
cargo test --release --offline -p singlefs-harness --lib -- --test-threads=2

cargo test --release --offline -j 2 -p singlefs-harness \
  --test mount_transparency_judged_field_by_field_after_a_writable_mount \
  --test reference_filesystem_translation_of_histories \
  --test mounted_recovery_reads_each_candidate_once_in_order \
  --test writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache \
  --test journal_scan_keeps_the_first_copy_of_a_duplicated_record_key \
  --test journal_replay_applies_no_more_records_than_the_in_flight_limit \
  --test sparse_block_device_journal_hint_matches_the_full_ring_scan \
  --test sparse_devices_refuse_requests_outside_the_device_like_the_file_backend \
  -- --test-threads=2
```

库测试被跳过的是 `model_comparison::tests::every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways`。没有跑全部集成测试、全部 ignored 历史、全量故障战役或硬件 GPU 对拍；上述结果只代表列明的执行范围。

## H01：读日志包装器吞掉底层支持的 discard

位置：[read_tally.rs:194](../crates/singlefs-harness/src/read_tally.rs) 的 `BlockDevice for ReadLoggingBlockDevice<Inner>`。该实现转发了普通读、写、清零、屏障和 journal 提示，却没有覆盖 `discard_at`。

因此 Rust 使用 [block_device.rs:198](../crates/singlefs-core/src/block_device.rs) 的 trait 默认实现：请求合法且长度非零时返回 `Unsupported`，不调用 inner。接口文档已明确规定设备包装应转给内层；默认实现只适用于不支持 discard 的后端。

### 复现与对照

同样的合法请求：偏移 0、长度 512、物理块宽 512、设备大小不小于请求范围。

| 设备 | 返回值 | 底层收到 discard | 读回结果 |
| --- | --- | --- | --- |
| 直接调用支持 discard 的计数设备 | Applied | 1 次 | 不作内容判断 |
| 外加 ReadLoggingBlockDevice | Unsupported | 没有新增调用 | 不作内容判断 |
| 直接调用预先写入 0x5a 的 SparseBlockDevice | Applied | 实际清零 | 512 字节全 0 |
| 相同 SparseBlockDevice 外加读日志包装 | Unsupported | 未执行 | 512 字节仍为 0x5a |

第二组使用项目自己的内存盘，排除了“只是假设备计数定义不合理”的解释。这里不是声称真实设备 discard 必须读出 0；真实设备可以保留旧字节。确认的是**同一个底层设备加观察包装后能力、返回值和实际动作发生变化**。

### 影响及限制

包在该设备上的测试会走“不支持 discard”的路径，无法代表同一内存盘支持 discard 时的行为。观测器改变被测对象，会破坏开日志与不开日志的对照前提。已有使用点包括 `tests/presumed_clause_checks.rs:523`；没有证明该既有用例当前一定执行了非零 discard，更没有把这项包装错误说成真实硬盘的数据损坏。

修复建议：显式转发 `discard_at`，保持返回值和底层错误；若需要日志，再记录实际动作，不借用 Unsupported 作为省略记录的手段。

验收应覆盖 Applied、Unsupported、底层 Err、零长度及非法请求；并比较加包装前后的返回值、底层调用数与数据变化。只测普通读写不会发现本问题。

## H02：三个设备包装器丢失专用读路由与缓存通知

位置：

- [RecordingBlockDevice](../crates/singlefs-harness/src/lib.rs)。
- [FaultInjectingBlockDevice](../crates/singlefs-harness/src/fault_injection.rs)。
- [ReadLoggingBlockDevice](../crates/singlefs-harness/src/read_tally.rs)。

三者均未覆盖以下四个 BlockDevice 方法：

1. `reread_at_after_a_bad_read`：坏读后的专门重读。
2. `read_at_straight_from_the_disk`：绕过挂载缓存的即时读盘。
3. `keep_the_self_verified_reread`：通知缓存保留自证通过的重读字节。
4. `note_the_read_failing_self_verification_and_forget_its_bytes`：通知缓存记坏读并丢弃缓存字节。

前两个默认转为 `self.read_at`，于是包装器最终调用 `inner.read_at`；后两个默认为空操作。内层提供的专用语义被擦掉了。这与“内层没有实现专用接口，合理采用默认值”不同：即使内层确实覆盖了，外层也无法把调用交给它。

### 复现与正反对照

探针使用一个显式实现上述接口的内层设备：普通读返回 0x11、重读返回 0x22、即时读盘返回 0x33；两个通知各自计数。先直接调用内层，验证四个接口都有效，再分别加三种包装。故障包装使用 **unarmed** 计划，排除注入本身改变结果。

| 路径 | 重读返回 | 即时读盘返回 | keep/forget 通知 |
| --- | --- | --- | --- |
| 直接内层 | 0x22 | 0x33 | 各 1 次 |
| 读日志包装 | 0x11 | 0x11 | 各 0 次 |
| 录制包装 | 0x11 | 0x11 | 各 0 次 |
| 未开故障计划的故障包装 | 0x11 | 0x11 | 各 0 次 |

三种包装合计仅增加 6 次普通读，内层四个专用接口调用数全部为零。确认了读路由与通知丢失，不只是在源码中搜索不到方法名。

### 与恢复及形式化验证的关系

core 的 `mounted_session.rs:1015` 确有对上述接口的非默认实现，用于挂载期间的缓存与重读。若测试用的缓存设备或自定义后端被这些通用包装包在外面，坏读后可能再次拿到普通缓存结果，验证通过的重读无法刷新缓存，坏字节通知也不会落到内层。此时恢复对拍的输入语义已经被测试装置改变。

**当前路径限制：** 常用 `HistoryDevice` 是 `FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>`（`history.rs:276`）。裸 SparseBlockDevice 本身采用默认接口，故这条现有栈上“丢了转发”通常不改变结果。core 的挂载缓存是内部包装，本轮没有证明它在现行历史栈中恰好处于这三个包装器之内。因此这是已证实的通用包装兼容错误，不是已复现的现行挂载缓存故障。

修复建议：四个方法逐个显式转发；读日志、故障计数、故障注入应保留原调用种类，再调用内层对应方法，不能直接把专用方法映射为普通读。对四接口通知也要保持调用参数。

验收应使用具有不同普通/重读/即时读行为的内层；同时验证调用计数、返回字节、两个通知，以及“缓存层在包装器内/外”两种栈顺序。只使用默认接口的内存盘，正反实现都会绿。

## H03：模型允许冷启动答案被空发布结果替代

入口：[model.rs:6972](../crates/singlefs-harness/src/model.rs) 的 `judge_and_advance`；缺口在约 7282 行 `judge_effect` 的 `ObservedEffect::Publishes` 分支。

该分支检查上限、回退目标及根列表，却不核对 `answer.operation` 是否允许 Publishes。`answer_cold_start_recover` 给出空 `expected_roots` 和非空 `expected_read_back`；调用者若交入 `Publishes { roots: [], reported_ceiling: None }`，空根列表比较通过，预期读回结果完全未使用，然后 `successes_matched += 1`。

### 复现

在相同 mkfs 后模型及其真实生成的冷启动答案上运行三个独立模型实例：

| 交入观测 | 结果 |
| --- | --- |
| ColdStart，正确根上的 NoFile | Ok |
| ColdStart，Failed | Err |
| Publishes，空 roots、无 ceiling、没有任何 read_back | **Ok，successes_matched=1** |

这不是“模型所有失败都不判”的情况：正确 effect 中的失败能判红，错误 effect 反而绕过检查。

### 影响及限制

模型是验证胶水的判官；胶水丢了冷启动观测或选错 effect 时，入口应拒绝，不能记录成功。本项增加的是防止验证失接的能力。

当前 `history.rs:7666` 的 `apply_cold_start_recover` 确实构造 `ObservedEffect::ColdStart`，并使用 `observed_read_back_after_a_crash(&report)`。本轮没有发现现行调用方实际交入错误 effect；所以评级 P3，没有宣称已有随机历史通过此洞漏掉了一次真实读回失败。

修复建议：先核对 operation 与 effect 的兼容关系，再检查字段。冷启动只接受 ColdStart，挂载只接受 Mount，发布类接受 Publishes；保留现有容量墙等分支，不靠补造读回结果解决。

验收应对错误 effect 组合做拒收对照，同时保证合法的成功、拒绝、部分发布等组合保持原行为。尤其要断言错误组合不增加成功计数、不推进模型状态。

## 形式化验证：本轮实际通过，但不包含三个问题所在的胶水

本轮执行：

```text
/home/fy5090/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus
  crates/singlefs-replay-prefix/src/lib.rs --crate-type=lib
  --no-cheating --output-json --rlimit 10 --num-threads 2
```

JSON 结果：`success=true`、`verified=78`、`errors=0`、`is-verifying-entire-crate=true`；版本 `0.2026.09.20.aef82ed`。这里的 entire-crate 指传给 Verus 的 **singlefs-replay-prefix**，不是整个 harness、core 或工作区。

核确实被真实路径使用：`core/recovery.rs:4983` 调用 `first_copy_of_each_record_key`，约 5199 行调用 `replay_core`。不是只证明一份完全未被调用的演示算法。

但证明接收的是已解析记录与读者回调的结果，不证明包装器转发、缓存通知、读盘字节真实性、RefFS 翻译及模型观测组合。这解释了 H01–H03 为什么能与 78 项证明通过同时存在。本轮没有运行全部必红世界、空真检查、GPU 卡上对拍或变异战役，不能把它们登记为已通过。

与核的范围互补，harness 现有 `mounted_recovery_reads_each_candidate_once_in_order.rs` 和 `writable_mount_reads_each_candidate_in_order_through_the_read_stage_cache.rs` 在块层钉住了部分具体读次数与顺序。它们提供的是列明形状上的执行证据，不等价于对任意设备包装栈的证明。H02 的非默认内层控制正是现有普通内存盘控制不能替代的一格。

## 对拍与故障恢复中必须保留的范围说明

以下是读码确认的验证边界，不另计为 bug：

| 边界 | 位置及观察 | 不能据此声称的结论 |
| --- | --- | --- |
| 故障后的 RefFS 弃权 | `reference_filesystem_translation.rs:1393` 附近：实际注入的六类故障状态都弃权；NeverFired 才可照判，现有测试明确要求这一约定 | 不能说每个注入点均由理想模型和 RefFS 两个独立 oracle 共同认可 |
| 恢复 oracle 聚焦首文件 | `model.rs:8054` 的输入是 root→可选内容，`ObservedReadBack` 不携带所有 inode 和快照内容 | 不能从一次重开首文件比较推出所有文件和快照逐字节正确；发布及快照读有其他独立检查 |
| 故障允许集合有意放宽 | `crates/singlefs-harness/src/fault_injection.rs:3538` 起加入测量跑版本及停下步骤的新根；后者内容可取模型已知内容或本次发布内容 | 这不是对失败操作原子性的完整规范判定；源码已明确留给既有定案，不能仅凭放宽就报缺陷 |
| 说谎设备的恢复失败可豁免 | `fault_injection.rs:3200`，与失败 mkfs 一起单记 | “没有新发现”不等于所有注入后的恢复均成功；应一起阅读 failed、excused、abstention 计数 |
| 挂载透明的 R1 构造复用 core | `mount_transparency.rs:149` 重跑 choose/replay/fallback，并断言与 recover 身份相同 | 这是跨动作一致性验证；相同读法中的共因错误可能同时影响期望与实际，不能作为完全独立的恢复规范实现 |
| 稀疏 journal 提示影响输入范围 | MemoryPool 与 CrashImage 为扫描提供候选槽；既有 hint/full-scan 对照有价值 | 两路都使用同一错误提示时，CPU/GPU 一致不能单独证明全环无漏读 |
| 默认测试不包含 ignored | 库测试本次跳过一项逐版本重型对拍；tests 目录另有大量 ignored 注解 | 不能将一次普通 cargo test 描述为全部历史、全部恢复形状均已验证 |

本轮没有把 checker 的引用缓存漏检、checker-tier 的版本编码和缓存覆盖问题复制为新发现。它们影响上游结论可信度，但具体证据与修复应继续引用各自原报告。

## 修复顺序与验收要求

1. 修复 H01，确保日志包装前后 discard 行为一致，随后检查其使用点是否曾无意走到 Unsupported 分支。
2. 修复 H02，并保留具有非默认读接口的内层测试。修复时一并核对记录及故障计数，避免转发后漏计或双计。
3. 补 H03 的 effect 兼容检查，防止胶水接错时把未执行检查记为成功。
4. 持续将通过、失败、弃权、未覆盖分开报告。证明核通过、单元测试通过、GPU 对拍一致是不同证据，不能相互替代。

本报告未把尚未复现的正常恢复错误升级为缺陷；后续若能提供完整镜像下的错误恢复与因果对照，应新增证据并更新评级。

## 用户要求后的结果复核（2026-10-24）

本次复核重新审查三项结论的实验条件、真实调用点和严重性，而不只重复原来的断言。**H01 保持 P2；H02 从 P2 下调为 P3；H03 保持 P3。** 三项可观察行为没有撤回，但不能把它们合称三项已在现有正常历史触发的恢复缺陷。

H02 下调的理由：当前 `HistoryDevice` 的内层是裸 SparseBlockDevice，并未提供有别于普通读的专用接口；真实 core 缓存层的存在也不能证明它在现有栈中恰好位于 harness 包装器里面。复核没有找到现行历史因此发生错误恢复的证据。因此保留“通用包装器无法保持非默认内层语义”的事实，将修复顺序收紧为 P3。

H01 与此有区别：项目自带 SparseBlockDevice 的非零 discard 已有直接的动作和字节差异；trait 文档也明确要求包装器转发。但复核在现有 `presumed_clause_checks.rs` 使用点仍未找到显式 discard 调用，不能声称该既有用例已经因为 H01 误判。

H03 重新追查到 `history.rs:7666`，现有冷启动执行器传递正确的 ColdStart effect。因此仍只记为判定入口防御缺口，没有扩大为现行冷启动失败漏检。

### 临时源码副本的因果对照

新增 [verify_recheck.py](../research/prompts/recovery-audit-harness-2026-10-24/verify_recheck.py)，复制 harness 的源码及编译期线程登记文件，仅修改临时副本。审计程序通过路径依赖链接这个副本，core 等依赖仍使用工作区源码。对照补丁全文保留于 [temporary-control.patch](../research/prompts/recovery-audit-harness-2026-10-24/evidence/temporary-control.patch)。

临时补丁只做三件事：给读日志包装补 discard 转发；给三个包装补四个专用接口转发；在 Publishes 分支拒绝需要读回结果的答案。**这是定位因果的实验修改，不是正式修复方案**：尤其没有完成转发后的故障注入与观测计数契约，不能将它直接作为产品补丁落地。

| 探针 | 原版第一次 | 临时补检查 | 恢复原版再次运行 |
| --- | --- | --- | --- |
| H01 非零 discard | 包装后 Unsupported，内存盘仍是 0x5a | 包装后 Applied，内存盘为 0 | 再次 Unsupported，内存盘仍是 0x5a |
| H02 三个包装器的两种专用读 | 全部变为普通读 0x11，内层专用通知未到达 | 全部交回内层的 0x22/0x33，四种接口各到达三次 | 再次全部变为 0x11，内层专用通知未到达 |
| H03 冷启动答案＋空 Publishes | Ok，记成功 | Err | 再次 Ok，记成功 |

三轮各 3 项审计测试通过，合计 9 次；每一轮的断言分别要求观察到原缺口、定向翻转、原缺口重现。三轮中直接内层、合法冷启动观测的正对照仍通过；真正的 Failed 读回仍被拒绝。由此排除了“计数假设备所有接口都返回同样内容”“模型所有输入一律接受”“临时补检查把合法输入也一律判红”等解释。

完整执行记录见 [causal-recheck.log](../research/prompts/recovery-audit-harness-2026-10-24/evidence/causal-recheck.log)。复核使用独立 `target/harness-causal-recheck`，避免与其他任务争用构建锁。

复核搭建过程也保留记录：首次临时副本缺少 `include_str!` 所需的线程登记文件，未进入测试，见 [initial-setup-error](../research/prompts/recovery-audit-harness-2026-10-24/evidence/causal-recheck-initial-setup-error.log)；之后一次共享 target 运行在等待编译锁时主动终止，见 [shared-target-interrupted](../research/prompts/recovery-audit-harness-2026-10-24/evidence/causal-recheck-shared-target-interrupted.log)。二者均不计为产品测试失败或有效对照结果。

复核的 [源码摘要](../research/prompts/recovery-audit-harness-2026-10-24/evidence/recheck-source-sha256.txt) 与首轮 434 个相关 Rust 文件一致，三轮完成后脚本再次确认这些产品源码未变。原报告 292 项库测试、82 项集成测试及 Verus 78/0 的记录重新核对无误；此次补做的是上述因果实验，没有把旧日志冒充再次执行完整测试或再次运行 Verus。
