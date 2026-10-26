# 恢复、形式化验证与对拍复核报告

复核整理：当日。本报告审查 `crates/singlefs-checker-tier` 当前工作区，包含未提交改动。关联 checker 的两项复核另见 [2026-10-24-singlefs-checker-审计复核.md](2026-10-24-singlefs-checker-审计复核.md)。文件摘要、执行记录和探针保留在 `research/prompts/recovery-audit-checker-tier-2026-10-24/`，报告统一存放于 `records/`。

## 当前结论与初稿更正

**初稿把“可观察到的信息损失”过快归为六项实现缺陷，定性不够严格。本轮不保留这个笼统计数。** 两个 crate 合计保留四项具体问题：checker 两项、本 crate 两项；其余观察明确区分为待证风险、已约定行为或验证范围。

| 初稿编号 | 复核后的处置 | 依据与边界 |
| --- | --- | --- |
| F01 | 保留为 checker C01 / P1 | 完整镜像原版→临时补检查→再次原版，唯一相关行为随修改翻转；合法镜像一直通过 |
| F02 | 保留为 checker C02，P1 改 P2 | 相同真实镜像和几何下确认扫描不同；限于冲突记录输入，不能外推正常发布流 |
| F03 | 保留为本报告 T01，P1 改 P2 | 12 组完整 u64 根身份的高位差异在真实比较函数中消失；未在 GPU 上注入实际错误恢复 |
| F04 | **撤回“已确认 P1 恢复缺陷”评级**，记 R01 待证风险 | 证明了字节枚举集合不同，但未证明违反已选三态抽象的观察等价，也未复现真实文件系统结果分歧 |
| F05 | **撤回笼统的“缓存有损就是缺陷”定性**；以更具体的 T02 / P2 替代 | 有损字段是明示约定；本轮进一步找到了现有调用方错误依赖该字段，实际引擎冷判/热复用能复现覆盖记录 83→0 |
| F06 | **撤回已确认缺陷评级**，记 B02 约定边界 | 失败码明确只编码 enum 大类；不同语义子原因同码符合当前接口约定，未证明现有调用方要求更细粒度 |
| G01 | 保留为 B01 范围说明 | 给定状态字不比内容；独立内容 oracle 的正反对照确实能发现错误内容 |
| G02 | 保留为 B03 证明范围说明 | 本轮重跑核证明通过；证明的前提和外部胶水应准确解释，没有发现证明失败 |
| G03 / G04 | 保留为 B04 / B05 信任边界 | 同源预处理及宽松复用豁免是需要说明的条件，不等于已证实算法出错 |

历史初稿保存在 [2026-10-23-singlefs-checker与checker-tier-审计初稿-已取代.md](2026-10-23-singlefs-checker与checker-tier-审计初稿-已取代.md)，仅供核对更正过程，不能再作为当前问题清单。

## 本轮实际复核

运行：

```bash
python3 research/prompts/recovery-audit-checker-tier-2026-10-24/verify_audit.py --verus
```

脚本创建临时 cargo 工程，调用当前 crate 的公开 API。在三个独立进程中分别运行全部断言，每个进程的缓存对照都使用独立的一次性 RocksDB 库。环境通过既有守卫恢复，不污染用户的长期判定库。`--verus` 另外运行整个形式化核。

- [探针源码](../research/prompts/recovery-audit-checker-tier-2026-10-24/verdict_encoding_and_torn_state_probes.rs)
- [完整日志](../research/prompts/recovery-audit-checker-tier-2026-10-24/evidence/verdict-and-torn-model-recheck.log)
- [实际测试消费者中逐字抽取的覆盖方法](../research/prompts/recovery-audit-checker-tier-2026-10-24/evidence/coverage-method-extracted.rs)
- [本轮 Verus JSON](../research/prompts/recovery-audit-checker-tier-2026-10-24/evidence/formal-kernel-recheck.json)
- [本轮 Verus stderr](../research/prompts/recovery-audit-checker-tier-2026-10-24/evidence/formal-kernel-recheck.stderr)

本轮没有运行完整 GPU 卡上对拍、全部变异战役和所有 ignored 全量历史。因此不会把对结论编码的 CPU 端证明，冒充 GPU 上已产生错误恢复结果的实测。

## T01：完整 txg 身份在对拍表示中发生别名

**优先级 P2。确认的是比较覆盖不完整，不是实际恢复选择了错误版本。**

位置：[crash_judge_conclusions.rs:367](../crates/singlefs-checker-tier/src/crash_judge_conclusions.rs) 的 `LandedRootWords` 和约 381 行的 `landed_root_words_of`；恢复字编码约 68 行，差异计数约 85 行。GPU 输出在 `crash_judge_recovery.wgsl:1538`、`1544` 两处只写 `effective_txg.x`。

恢复版本类型为 u64 txg，但比较/存储仅保留低 32 位。代码确实明写了这个截断；本项不是声称它没有遵照自己的“取低 32 位”实现，而是指出这个实现不能承担“根版本身份相同”的完整验证。

### 重复与正反对照

低位选 `0、1、7、u32::MAX`，高位选 `1、2、0xFFFF_FFFF`，共 12 对：

```text
A = (实例1, txg=low)
B = (实例1, txg=(high<<32)|low)
```

每对均断言：

1. 原始 `effective_root` 不同。
2. 构造出的 `CrossCheckWords` 相同。
3. **实际** `cross_check_words_agree_in_the_cross_check` 返回 true。
4. 恢复字相同，`recovery_passes_differ` 返回 false。

负对照使用 txg7 与 txg8：真实比较函数返回 false。三个独立进程均得到相同结果，排除了“比较函数总为 true”或一次性缓存污染的解释。

### 影响与限制

如果两路恢复错误地落在同实例、高位不同而低位相同的版本上，这个比较器不能发现它，差异计数和持久恢复字也会丢失该区别。当前小 txg 流一般不会触及此域，因此优先级从初稿 P1 降为 P2。

探针直接构造类型正确的恢复报告，没有在磁盘上制造并执行两种错误恢复。尚未实测一个 GPU 高位错误；也没有证明常见短历史会触发它。上述限制不影响“12 对完整身份在真实比较器中别名”的确定结论。

### 建议与验收

显式保存 `has_root`、实例号、txg 高低位；CPU 字、GPU 输出、恢复字、缓存格式一起更新。或者明确给验证域增加经检查的 `txg < 2^32` 限制，并停止将结论解释为对完整 u64 域的保证。

验收应覆盖高位变化、`2^32-1`→`2^32`、无根/有根、存储还原和 GPU 输出。不能只为现有小 txg 流增加样本。

## T02：现有覆盖统计错误读取了缓存未保存的逐状态判到位

**优先级 P2。错误位于缓存契约与使用方之间；本轮没有复现缓存把坏镜像判成好镜像。**

有损还原位置：[crash_judge_conclusions.rs:1066](../crates/singlefs-checker-tier/src/crash_judge_conclusions.rs)，约 1120 行明确将 `pool_checker_judged_bits` 置 0，扫描重建结局置 NOT_REACHED。块级 `count_vector` 另存，并非丢失。

实际调用方位于：

- [随机历史测试:318](../crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs)：`note_the_invariants_held` 用逐状态 `judged_bits & !violated_bits` 统计“成立过”。
- 同文件约 511 行调用该方法。
- 同文件约 1271 行，全量测试要求 I-3.1、I-5.4、I-3.11、I-7.9、I-9.15 至少成立过一次。

### 为什么本轮不能简单撤回 F05

仅证明还原字段变成 0，确实不足以判定现有系统有 bug：接口注释允许它，正常崩溃注入汇总读取的是另存的计数向量。然而继续追到上述真实消费者后，发现它恰好依赖了不允许依赖的字段。

因此，初稿“有损缓存本身就是缺陷”的泛化定性被撤回；保留的是**现有覆盖使用方未适配该缓存约定**这个更窄、更具体的问题。

### 实际引擎的冷判/热复用对照

构造既有真实 mkfs→暖机→文件发布的合法镜像，基线为 mkfs 后，后续写流组成给定状态表；同一个实际 `GivenStatesEngine` 对同一张表判两次：

| 项目 | 首次冷判 | 第二次缓存复用 |
| --- | --- | --- |
| 引擎账 | tables_judged_here=1 | tables_reused=1 |
| red items | 空 | 空 |
| 完整块级 count_vector | 与热复用相等 | 与冷判相等 |
| 使用现有测试方法得到的“成立过”集合 | **83 条** | **0 条** |
| 键拒算 | 0 | 0 |

覆盖计算方法由脚本从当前既有测试源码逐字抽取，不是另写一个近似算法。抽出的原文也随证据保存。每个独立进程重新创建一次性库，三次都得到 83→0。

该小夹具能冷判成立 I-3.1、I-5.4、I-3.11、I-9.15；I-7.9 在这个夹具上没有相应覆盖，故没有冒称冷判覆盖了全量测试要求的全部五项。用例最初过强的“五项都应冷判成立”前提被修正为与实际夹具匹配的四项；不是把产品失败当成实现问题。

### 能断言到哪一步

已确认：从缓存复用出来的表，交给现有 `note_the_invariants_held` 时不再贡献任何成立覆盖，尽管块级计数证明这些检查做过。若相关全量表都命中缓存，当前全量覆盖断言会因此缺失成立记录。

没有本轮运行十分钟级全量历史并宣称它实际失败，也没有声称冷判/热复用使已有块级计数变化。主要崩溃注入路径读取 `count_vector`，这部分做法是正确的。

### 修复建议与验收

覆盖消费者应从 `GivenStatesConclusionsOfATable.count_vector` 中使用 `judged > violated` 记录“至少一次成立”，或者存回真实逐状态判到位。不要把所有状态或所有不变量一律补成成立。

验收：同一输入冷判、热复用的“成立过”集合和覆盖断言必须相同，且保留 red items、完整计数向量一致性。另构造某条不变量在所有状态都违例或不适用的表，确保修复不会伪造成立覆盖。

## R01：撕裂模型——确认字节集合不同，尚不能确认恢复漏检

位置：`crash.rs:2347` 附近 `WritesWithTornImages::of`；约 2371 行将全部前写施加后固定生成撕裂镜像。`torn_image_of_in_place_overwrite` 在约 2309 行。

合成原始字节输入：4096 字节基线 0x11，同段 W0 写 0x22，W1 写 0x33。枚举器得到九个三态组合，但没有 `[0x33×2048 | 0x11×2048]`。独立的逐扇区参考在 W0 不落、W1 八个扇区各取新/旧的 256 个状态中包含该结果；参考也能得到全旧/全新，作为正对照。

探针还明确检查了 W0 原写及其撕裂像都未选择、W1 撕裂的状态：尾半段仍出现来自 W0 的 0x22。相较初稿仅检查 W0 原写未选择，本轮排除了“W0 其实选了撕裂态”的反解释。

**为什么撤回初稿 P1 缺陷评级：**

1. 这些全填同字节的 `SystemConfigurationSlot` 不是一个可解析的真实系统配置流；目前只在枚举器层测试字节。
2. 设计已明写原地覆写用三个抽象状态，discard 的任意扇区组合也不另枚举（`crash.rs:2180` 附近）。不能因为不是物理逐扇区全枚举，就自动认定违背了既定模型。
3. 尚未给出真实文件系统上两个被合并状态导致恢复、记录核对器、checker 判定不同的例子，也未证明登记流会形成这组同段重叠写。

同扇区偏移10、20的字节切分探针仍表明代表像不一定物理可达；但不可达代表像可以是有意抽象。必须进一步证明或否定**判器观察等价**，而不是仅比较镜像字节就给出不健全结论。

后续验收应以真实可解析写流、独立扇区模型及恢复/判定结果差异为证据；当前登记为待证模型风险，不计入四项已确认问题。

2026-10-26 补查（量的，调查员报告 `~/.local/share/singlefs/work/investigate-overlapping-in-place-overwrites/result.md`，主 agent 核过它的产物）：合表登记的 45 条崩溃流里，同段、同盘、区间重叠、两次都被 `WritesWithTornImages::of` 认成可撕裂的原地覆写对数全是 0，所以今天的登记流碰不到 R01 这种缺口。风险本身仍登在 `.claude/kb/checks-owed.md` C898，管以后新登记的流。去向与经过见 [2026-10-24-checker-tier审计复核修复.md](2026-10-24-checker-tier审计复核修复.md)。

## B01：给定状态字不比较内容，但全内容 oracle 正常工作

给定状态构造函数的契约明确不含期望版本表。根相同、都报 FileRead、内容 [1]/[99] 时，字确实相同。

本轮补了相反方向对照：给普通内容 oracle 一张期望内容为 [1] 的版本表，[1] 被接受，[99] 被拒绝。因此“给定状态摘要不能单独保证内容正确”成立；“所有崩溃测试都不检查内容”不成立，也未在报告中保留。

实际调用场景若需要内容正确性，应另接允许版本集合；不能把给定状态摘要当成完整内容证明。此项属于范围约定，不计为独立实现 bug。

## B02：失败码按错误大类编码，撤回 F06 缺陷评级

`gpu_failure_point_code` 的注释明确说 enum 成员各一码，成员携带的细节不进码。因此 I-1.2 与 I-1.3 都属于 `InvariantViolated`，同码符合契约。

本轮正反对照：同属 InvariantViolated 的两个原因同码；InvariantViolated 与 NoValidRoot 不同码，编码确实保留了约定的大类区别。未发现现有调用者要求“必须比较不变量编号”的更强契约或由此造成实际错误结论。

更细原因编码可以提高对拍判别力，但不能把改进建议冒充已证实实现错误。初稿 F06 的缺陷评级撤回。

## B03：形式化证明重新通过；范围不是整个恢复磁盘过程

本轮使用已安装 Verus `0.2026.09.20.aef82ed`，直接运行：

```text
verus crates/singlefs-replay-prefix/src/lib.rs --crate-type=lib
      --no-cheating --output-json --rlimit 10 --num-threads 2
```

结果：**78 verified、0 errors、success=true、is-verifying-entire-crate=true**。该结论来自实际 JSON，不是复用登记行；本轮没有跑完整空真与必红世界集合，不能把两者也写为已通过。

审查确认：

- `replay_core` 确被 `crates/singlefs-core/src/recovery.rs:5199` 调用，核不是纯展示品；core 约 5143 行也断言了键序来源和 counter 后继前提。
- 原始字节解析、读盘回调正确性、checker 自己建表、NewRootSource 到实际根字段的拼接、可写挂载的磁盘副作用不在这段纯函数证明之内。
- 记录级幂等与不动点性质描述 RootSource/applied 集合，并带稳定读者、txg 次序、在飞上限等前提；不等于证明所有恢复写盘续作逐字节幂等。
- 单调推论要求较大状态的选根覆盖关系等写侧前提；核没有独立从扇区持久集合包含关系证明这些前提。
- 整理选段/计划的证明核由测试对拍连接到产品逻辑，产品不直接调用这两个核；不能宣称整理发布和崩溃恢复已端到端证明。

这些是准确解释证明结果所需的范围说明，不是发现 Verus 验错。现有工程也有删除续作幂等测试等独立覆盖，不能因为核未覆盖就说工程全无相关测试。

## B04：GPU 判表存在同源预处理边界

`crash_judge_tables.rs:17` 引入 core 的 CRC/宽校验函数，约 1355 行 `version_checks` 预计算校验结果。GPU 消费了其中一部分事实，不能把 GPU 路径描述为从原始字节起全部独立验证。

但独立 CPU checker 另有校验实现，完整对拍可以帮助发现这种同源错误。没有发现 CRC 实际算错，也没有实测共因错误穿透整个验证链。本项是信任边界，不计为已确认 bug。

## B05：记录核对器的复用豁免明确是宽松判据

`reuse_is_not_proven_illegal_by_the_reclaim_predicate` 按界判“未证明非法”，不读取全部真实释放历史来证明完整合法性。源码已有说明。

本轮没有新增非法复用穿透全部其他判器的反例。保留提醒：记录核对器通过不能单独作为回收合法性的完整证明。以后若增强精确参考，应避免继续复用这一个近似函数。

## 优先处理顺序

1. checker C01：已有完整镜像零违例漏检和因果对照，优先修复。
2. checker C02 与 T01：明确冲突输入策略，完整保存版本身份或限制验证域。
3. T02：修正已有测试覆盖消费者，使冷判与缓存复用语义相同。
4. R01：先补真实流和观察等价证据，再决定是否升级为模型缺陷。

任何修复都应独立验收，不把审计目录 `research/prompts/recovery-audit-checker-tier-2026-10-24/` 里“成功复现问题”的通过结果当作修复成功。本次只审计与整理证据，没有修改产品实现。
