# 翻译核对表：m2-step3-code-r1 本地攻方提示（research/prompts/m2-step3-code-r1-local-attack.md）

管的是提示里翻译自本仓中文条款（决策、不变量、上一轮判决）的那些事实条目（F14-F24）。代码事实条目（F1-F13）是主 agent 直接读源码写下的英文观察，不是翻译，不进这张表；其中三处引用了源码里的中文断言/注释字符串，已在定稿里改成纯英文释义（不保留中文原串），核对见表末「代码内中文字符串」一节。

每行：英文项（提示文件里的编号）/ 原文文件:行 / 首稿缺的 / 定稿怎么补。

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| F14（分配记录条目编码与释放不删） | `.claude/kb/decisions/03-空间分配.md:175-180`（已定项 7）与 `:420-426`（已定项 11） | 首稿只译到「entry gets overwritten by the new allocation's data」，漏了原文最后一句「崩溃丢掉的是没发布的那个 checkpoint，它里面的释放一条都不留痕、对应的旧版本仍被所选根引用」——这是 X3 判断重启后一致性时用得上的限定：释放要等到发布落盘才算数，没发布的释放崩溃后连痕迹都不留 | 补一句："what a crash loses is exactly the unpublished checkpoint, so a release that never got published leaves no trace at all, and the placement's older version is still referenced by whichever root ends up selected." |
| F15（D5 已定项 4 统计量表 1/2/5 项与两条 ⚠️） | `.claude/kb/decisions/05-快照-空间记账机制.md:356-372,398-405` | 无遗漏限定词；省略了警告句里指向 `test-discipline.md` 的引用出处（那是文内引用标注，不是限定条件） | 未改；出处引用不是限定词，省略不改变可判定的内容 |
| F16（I-3.1） | `.claude/kb/invariants.md:120` | 无遗漏 | 未改 |
| F17（I-5.2） | `.claude/kb/invariants.md:167,170` | 无遗漏 | 未改 |
| F18（I-3.8） | `.claude/kb/invariants.md:127` 及 `crates/singlefs-checker/src/walk.rs:407-409` 注释 | 无遗漏 | 未改 |
| F19（I-7.7） | `.claude/kb/invariants.md:53` | 首稿漏译原文最后一句「只在 devs ≥ R 时根环才是超级块之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘」——这条限定说明第一版里 I-7.7 的两个校验源其实不独立，对 X7 第 3c 问（I-3.8 与 I-7.7 会不会互相矛盾）是直接相关的背景 | 补一句：说明只有 devs >= R 时根环才独立于超级块，第一版 R=3、devs=2，两个见证源共用同一对盘的失效域 |
| F20（D19 已定项 5，位置条目降级为提示） | `.claude/kb/decisions/19-块指针的结构与宽度预算.md:198-234` | 省略了第三条硬规则里 E109 给出的具体测量数字（千次读 4760-5480 个节点，比指针权威基线 4000 多 19%-37%）——这是支持性测量数据，不是改变判断条件的限定词 | 未改；判断三条硬规则的存在与内容不受这个数字影响 |
| F21（D13 已定项 4，崩溃点重放枚举域） | `.claude/kb/decisions/13-验证路线.md:328-334,362-363` | 省略了「为什么撕裂态可以并进『没持久』」那一段关于校验和机制的论证（父指针里的校验和、自证单元的整单元校验和），只保留了结论；这段是论证不是限定词 | 未改；结论本身（撕裂态并入未持久、旧字节非零、要真生成镜像）完整保留，论证从略不影响可判定性 |
| F22（D16 已定项 8，暖机） | `.claude/kb/decisions/16-发布语义.md:207-208` | 核心句「覆盖两块盘」在定稿里泛化成了「覆盖每一块设备」；核对代码事实 F3（`mount.rs:285-323` 的循环条件用 `all_devices.iter().any(...)`）与主体正文 S6「直到本实例的根落到每块区域盘上」，确认这不是漏译而是决策原文用的是第一版恰好两块盘的具体数，机制本身按全部设备算；已在同一条事实的警告句里保留「第一版实付 2 次」「归属写死 0/1/0」这些第一版专属的具体数字 | 未改；泛化与代码行为及正文 S6 一致，不算丢限定词 |
| F23（D28 已定项 4，保留池现算；D23 已定项 14 注，切换预留） | `.claude/kb/decisions/28-挂载期承诺量.md:98-100`；切换预留式子引自 `research/prompts/_m2-step3-code-r1-appendix.md:447`（转引 D23 已定项 14 注 3） | 省略了切换预留式子里「实例表链每切一次长一行、写行要 COW 重写整条链，第 k 次比第 1 次贵」这句——这是解释代价为何递增的论证，不是改变 N_switch=3 或换算方式本身的限定词；已保留「越过 N_switch=3 也转只读」这条限定 | 未改；核心限定（N_switch=3、越界转只读、切换机制目前未实现）都在 |
| F24(a)（Y2：空闲改法整撤回，全仓零判红） | `research/prompts/m2-code-r2-main-verification.md:34` | 首稿把「四处改回 HEAD 的形态」简化成了泛泛的「with the independent free-slots counter maintenance removed」，丢了「四处」这个范围限定——它说明撤回覆盖的是这一个修复涉及的全部四个改动点，不是只删掉一行减法 | 补一句：「reverting all four code locations that this one fix had touched」 |
| F24(b)（Y3/Z5：内容超长 32634 改报错） | `research/prompts/m2-code-r2-main-verification.md:35` | 无遗漏 | 未改 |
| F24(c)（Y4-①/Z6：oracle 新臂） | `research/prompts/m2-code-r2-main-verification.md:37` | 无遗漏 | 未改 |
| F24(d)（Y4-②/Z7：Ignore 那一遍另计） | `research/prompts/m2-code-r2-main-verification.md:38` | 无遗漏 | 未改 |
| F24(e)（顺带/Z8：同盘槽号唯一） | `research/prompts/m2-code-r2-main-verification.md:39` | 无遗漏 | 未改 |

## 代码内中文字符串（不算翻译条目，单独核）

以下三处英文提示原稿曾把源码里的中文断言/注释原串与英文释义并排列出；按「提示用英文写」改成只留英文释义，中文原串删除，核对内容有没有跟着丢：

| 位置（提示里的事实号） | 源码文件:行 | 中文原串 | 定稿英文释义是否等价 |
|---|---|---|---|
| F4（PoolAllocator::release 的三条断言消息） | `crates/singlefs-core/src/allocator.rs:359,360,364` | 「释放的落点要有分配记录」「同一个落点释放了两次」「释放的跨度与分配记录不符」 | 等价：三条分别译成 "the released placement must have an allocation record" / "the same placement was released twice" / "the released span does not match the allocation record"，与原串逐句对应 |
| F5（rebuild_from_records 的一条 panic 消息与一条函数注释） | `crates/singlefs-core/src/allocator.rs:300`；注释在 `:287-288` | panic 消息「分配记录的盘在池里：走读逐盘核过」；注释「重开后按『没有开放段』起步：下一个提交内生块从最低的全空段开——上一段里没用完的槽先留着」 | 等价：panic 消息译成 "the allocation record's device must be in the pool: checked during the read-walk"；注释译成 "after a restart, it starts as if there were no open segment: the next commit-generated block bump-allocates from the lowest fully-empty segment; the unused-but-not-full slots left over in the segment that was open before the restart are simply left behind (not reused until the segment is fully empty again)"，两句限定词（「起点用最低全空段」「上一段没用完的槽先留着」）都在 |
| F7（FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE 常量注释） | `crates/singlefs-core/src/recovery.rs:797` | 「mkfs 写在单元区里的 2 个落点加第一个事务的 8 个落点（字节表五：20 条记录，每盘 10 条），之后每次发布只多不少」 | 等价：译成 "the 2 placements mkfs writes in the unit area plus the first transaction's 8 placements (byte table five: 20 records total, 10 per device); after that, every publish only adds more, never fewer"，「只多不少」这条单调性限定保留 |
| F12（oracle_violation_for_versions 的两句诊断文本） | `crates/singlefs-harness/src/crash.rs:570,574` | 「没择到根」「根槽已持久而恢复到旧态」 | 等价：分别译成 "did not select a root" / "the root slot was already persisted at a newer state but recovery landed on an older one"，语义未变，只是从「中文原串 + 英文释义」并排改成单独英文释义 |
