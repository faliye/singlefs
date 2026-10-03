# m3-formal-r5 辩方腿报告（Sonnet，2026-10-02）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 U1 U2 U3 U4 U5 U6 U7 F1 F2 F3 F4 F5 F6 O1 O2 O3 P Q S Sd R1 R2 Y2 Y7 M2 M3 M4 C191 C447 C519 C331 C381 E143 -->

轮名 m3-formal-r5（第二轮），立场云端辩方：复核第一轮判决 `research/prompts/m3-formal-r4-main-verification.md`（含「用户定案」一节）够不够得着、是不是同样打中所有候选，替被判出局的候选找站得住的理由。用户定下的 F1–F6 不改，只判它们写成原句时哪里站不住。方案按 `crates/` 今天的实现谈。

读了：背景正文与分工（`research/prompts/_m3-formal-r5-body.md`）、第一轮判决、两份腿报告、岔路单 `research/prompts/m3-formal-r4-forks.md`；kb 的 D13、D16、D17、D18 已定项 11、D23 已定项 14–16、C191 与 C447、`verification-build.md` 第七节、`invariants.md` I-8.4；代码读冻结副本 `/tmp/claude-1000/m3-formal-r5/tree/`（开工先核：`sha256sum -c research/prompts/m3-formal-r5-snapshot/crates-sha256.txt | grep -vc ': OK$'` 输出 `0`；`grep -rn 'RefFs\|ref_fs\|reffs' crates --include=*.rs | wc -l` 输出 `0`）。没读禁读清单里的文件；没读 `/tmp/claude-1000/m3-formal-r4-forks/report.md`（岔路单第二节代价的数法，本轮不重数代价）。

记号：**[读]** 读 kb 或代码；**[推]** 推的，没量；**[量]** 是别的腿量过、我引来的数。本轮我自己一个数都没量（不编译、不跑重型测试）；引来的数都标出处与「副本上量的」。引文行号都是这一次在被引文件里 `grep -nF` 现取的；kb 条款行号是 kb 文件自己的行号。

一句话：
- 第一轮三处「打中」里，T3 够不着 D13 已定项 3 那一半（模型文件自己不读录制流），T5 的收严写窄了两处（卸载记号、F），T2 的子点里 T2-1、T2-3 同样打中用户定下的 Q，T2-2 只打中 Q 的边界来源。
- T1 站住但那句话漏了射程；T4、T6 站住但各漏两处（T4：故障注入这一截、二次崩溃的期望值取自实现；T6：二次崩溃镜像上的比对对象、拒绝本身 O1 已有条款）。
- 用户定的 F1–F6 写成原句，F1（Q）、F2（合取）、F6（加二次）各有一处站不住，F2 其实被 F3 推出来了。
- 任意子集、O1 剥原地、F6-只第一次三个出局的候选，各有一格其实不输（第五节）。

## 一、判定一览

判定三档：站得住 / 够不着 / 同样打中别的臂。「同样打中」指同一个打击也打在用户定下的那一个或别的候选上。

| 判决条目 | 辩方判定 | 证据（细节在第二至五节） |
|---|---|---|
| T1 fsync 条款句站住 | 站得住，但句子漏射程与「已接受」的口径 | 句子对着「不带故障、串行」才成立；故障注入里说谎的设备、D23 已定项 14 的 C519 都让它为假（第三节） |
| T2-1 回退 9 / 4 108 态：语义级放过盘级下界 | 同样打中 P、Q、任意子集；但「合取下判红」的前提不实 | 今天的实现在这 9 个态上读回回退后的内容，O1 判官 ③ 现状就在判；放过的是「一类坏实现」不是今天的态（第二节） |
| T2-2 写请求覆盖两单元：2 / 3 / 4 | 够不着 forks 里的 P、Q（已按单元定义吸收）；够得着 Q 的边界来源 | Q 在攒批之后要 RefFS 知道发布切在请求中间的哪里，今天没有任何条款给它这个量（U2） |
| T2-3 今天三种丢法造不出不同集合 | 同样打中所有候选，含 Q | 今天一次 `publish_*` 调用一次发布，Q 与 P 在可达态上同集合（第二节） |
| T2 任意子集出局 | 站得住（串行前提下）；并行提交那一域够不着 | D13 已定项 2 自己写「前缀」是全序上的概念；出局的理由要带「串行」二字（第五节） |
| T3 O1 扩成 RefFS 同时违反 D13 已定项 3 与 Y2 | D13 已定项 3 那一半够不着；Y2 那一半只打中「原样扩」 | 模型文件不读录制流，读流的是调用点；剥原地同样躲得过；(lo, hi) 的翻译层对所有候选一视同仁地同源（第二节） |
| T4 站住 | 站得住，漏攻两处 | 故障注入这一截（F2 已把它算进来）与二次崩溃一截的期望值取自实现自己的读回（第三节） |
| T5-1 卸载记号、F、F 生效值「没有卸载的动作下必须相同」 | 打中了模型原句，但判决改写的这一句仍站不住 | 攻方的改法带「含准入推的抬 F」，判决把括注丢了；卸载记号在 R1 带卸载记号时也不相同（U5） |
| T5-2、T5-3 比的对象 | 站得住；有更便宜的取法 | 取「只挂载、不发布」的 v2a，不必去读环里非最新的那条根（U5、U6） |
| T5-4 实例表行有确定关系 | 站得住，规则能由 D18 已定项 11 逐字写全 | U5 |
| T6 站住 | 站得住，漏攻两处 | 二次崩溃镜像上没有「挂载后那次发布」；拒绝本身 O1 已有条款对着（U6） |
| U1–U7 | 逐格见第四节；每格给候选原句与会推翻它的现象 | — |
| F1–F6 原句 | 各处站不住的地方见第六节 | — |
| 任意子集 / O1 剥原地 / F6-只第一次 | 各有一格其实不输 | 第五节 |

## 二、复核 T2、T3、T5：够不够得着，是不是同样打中所有候选

### 2.1　按候选对一遍

表里「中」= 这个打击成立在这个候选上；「不中」= 够不着；候选按岔路单第一节的名字，加判决里提到的两个。

| 打击 | P（单元级前缀） | Q（发布边界前缀，用户已定） | 任意子集 | O1 剥原地 | 另起（用户已定） |
|---|---|---|---|---|---|
| T2-1 语义级放过回退下界 | 中 | 中 | 中 | 不中（③ 随 O1 留着或删，看怎么剥） | 不中（O1 原样留着，③ 现成） |
| T2-2 请求跨两个单元 | 不中（P 已取到单元） | 中（边界来源） | 不中（任意子集本来就含它） | — | — |
| T2-3 今天分不开 | 中 | 中 | 中 | — | — |
| T3 与 D13 已定项 3 | — | — | — | 不中 | 中（(lo, hi) 翻译层同源） |
| T3 与 Y2 | — | — | — | 不中（删掉盘级那一半） | 不中 |

### 2.2　T2

**T2-1 够不着「合取下判红」那一句。** 背景正文第三节 U3 格写「回退那段 9 / 4 108 个状态在合取下判红、单独判放过」。攻方自己写的是另一回事：「今天的实现在这 9 个上读回的是回退后」（`research/prompts/m3-formal-r4-opus-output.md:213`）。

那 9 个态在今天的实现上读回的是回退后的内容，O1 判官 ③ 与合取都给绿，没有「谁在判红」可问；T2-1 放过的是一类假想的坏实现（读回回退前的内容）。攻方自己写明：「这半句我没证到」（`research/prompts/m3-formal-r4-opus-output.md:218`），判决表里标了「量过」的，量到的是分类，不是一个会出红的态。③ 今天就在崩溃注入里跑：`crash_recovery_disagreement` 里「if chosen < persisted {」（`crates/singlefs-harness/src/model.rs:2263`）；用户定 F3 另起、O1 原样留着，合取就是「现状加一票」：岔路单自己写了「③ 留在 O1，与 F3-剥原地不能同时取」（`research/prompts/m3-formal-r4-forks.md:55`）。所以 F2 不是独立于 F3 的一问，F3 取另起就把 F2 推出来了（第六节 F2）。③ 自己有没有判别力是另一格：岔路单现数 O1 判据 ③ 的变异行 0 行（`research/prompts/m3-formal-r4-forks.md:39`，数法没重做）。

**T2-2 够不着 forks 里的 P、Q，够得着 Q 的边界来源。** 攻方表里的「按 POSIX 操作前缀」取到整个写请求，所以得 2；岔路单把 P、Q 都定义成取到单元：「操作取到一个数据单元的写（一次写请求按单元净荷切成 ⌈长度 ÷ 净荷⌉ 个子操作，偏移升序）」（`research/prompts/m3-formal-r4-forks.md:24`），所以请求级的 P 不在候选里，T2-2 对 forks 的 P 够不着。它对 Q 够得着：Q 的集合取决于发布切在请求中间的哪里，攻方表里写的是「取决于切在哪：切点不是 POSIX 面上的量」（`research/prompts/m3-formal-r4-opus-output.md:232`），2 或 3。D23 已定项 14 允许单元级的中间态：「崩溃原子性的单位是一个单元、不是一次写请求」（`.claude/kb/decisions/23-journal的角色与格式.md:389`），D16 已定项 5 让切分落在请求中间成为可能：「取号 = 事务被分配进某个 checkpoint 那一刻，checkpoint 的切分只关闭成员资格——触发之后不再分配事务进它」（`.claude/kb/decisions/16-发布语义.md:133`），同一请求切出的事务「同一次请求切出的若干事务按其单元的 key 升序取号」（`.claude/kb/decisions/16-发布语义.md:133`）。所以「u1 新、u2 旧」按条款合法，Q 要放行它就得知道那次触发落在 u1 与 u2 之间（U2 展开）。

**T2-3 同样打中所有候选。** 今天一次 `publish_*` 调用一次发布；多单元的写也只是一次发布里的 N 个事务：「从文件偏移 0 写整份内容，按切分纪律切成 N 个一单元事务」（`crates/singlefs-core/src/transaction.rs:4784`），整份换内容、变短时「上一版比这一版多出来的数据单元」同一次发布里释放（`crates/singlefs-core/src/transaction.rs:4792`）。可达态上 P 与 Q 同集合，Q 与 P 一样今天没有会红的态。冻结副本里没有攒批的代码：`grep -rln '攒批' crates` 无输出；`T_time` 与 T_dirty 的触发路径也没有：「也没有触发路径——T_time 与 T_dirty 的发布触发没实现」（`crates/singlefs-core/src/system_configuration.rs:278`）。

### 2.3　T3

判决 T3 一行写：「O1 今天既读录制流、又带盘上的根环落点」（`research/prompts/m3-formal-r4-main-verification.md:23`）。拆开看：

1. **「读录制流」够不着模型文件。** 岔路单现数：「model.rs 自己不读录制流。」（`research/prompts/m3-formal-r4-forks.md:38`）。③ 的入参是函数参数；从流里数它的是 `newest_persisted_root`（`crates/singlefs-harness/src/memory_pool.rs:812`），调用点在「let newest_persisted = newest_persisted_root(&writes, &persisted)」（`crates/singlefs-checker-tier/src/crash_injection.rs:881`）。D13 已定项 3 禁的是「RefFS 与实现不许共用崩溃点枚举的那段代码」（`.claude/kb/decisions/13-验证路线.md:65`）；把模型文件原样扩一扩，不会让它多出一行枚举代码。正推腿写的「这一条没有门禁盯」（`research/prompts/m3-formal-r4-sonnet-output.md:112`）也只对一半：没有门禁阶段，但有一条单测「fn the_model_module_uses_only_the_standard_library_and_the_format_constants() {」（`crates/singlefs-harness/src/model_comparison.rs:1282`）和一行变异「模型模块 use 了 singlefs_core」（`crates/mutations.tsv:136`）。
2. **「带盘上的根环落点」打中「原样扩」，打不中剥原地。** 模型文件头自己写它记的是「记每一版提交的内容、每条根属于哪个实例、实例表的行、回退下界 F」（`crates/singlefs-harness/src/model.rs:1`），与 Y2「没有盘上格式」相冲；剥原地把这一半删掉就不冲了。「原样扩」从来不是候选，所以这一打击不分辨剥原地与另起。
3. **真正对所有候选一视同仁的是翻译层。** RefFS 要的 (lo, hi) 由 harness 的步号翻译层给：「fn step_containing(&self, stream_index: usize) -> Option<StepPosition> {」（`crates/singlefs-checker-tier/src/crash_injection.rs:751`），它与崩溃点枚举读同一份录制流。无论 RefFS 是剥原地、剥复制还是另起，它的两个整数都从这里来。攻方在第五节写过：条款没写翻译这一步属不属于那段代码（`research/prompts/m3-formal-r4-opus-output.md:255`），判决没收。候选条款句（零轮）：RefFS 的源码不引录制、切段、翻译与恢复的任何入口，它与崩溃点枚举之间只有 (lo, hi) 两个整数；这两个整数由翻译层从同一份录制流算出，翻译层把某次发布归错步时 RefFS 与枚举一起歪，这一残余同源由「每一类 RefFS 时刻至少被一个崩溃点命中」的覆盖检查与记录核对器罩，不算 RefFS 的独立性。推翻它的现象：找到一个只有录制流才有、而 RefFS 的允许集合非要不可的量。

### 2.4　T5

T5 的四处打击打在第一轮正推的 14 行表上，不分候选；它们对 F6-只第一次与 F6-加二次一样要用。我复核的是判决把它们收成的那一句，结果在 U5：卸载记号与 F 写窄了。先说数据的射程：攻方 65 634 个态（副本上量的）来自六段手写历史，攻方自己写明「挂载之后的回退、抬 F、连发多次没扫」（`research/prompts/m3-formal-r4-opus-output.md:294`）；原型里没有一段历史在崩溃前做过正常卸载（冻结副本里的历史操作集合也没有卸载：`crates/singlefs-harness/src/history.rs:383` 的关闭操作只是「关掉这个进程的可写会话，mount::mount_writable。」）。所以「没有卸载的动作下相同」是在没有带卸载记号的 R1 上量的，量不到 R1 带记号那一格。

## 三、T1、T4、T6「站住」有没有被漏攻

### 3.1　T1：站住，漏了射程、「已接受」口径与报错的正面半句

候选句是「fsync 返回成功之后，调用开始时刻已经接受的全部操作必须在；fsync 也可以报错，报错时不承诺持久」。它与 D16 的字面相容：「fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号」（`.claude/kb/decisions/16-发布语义.md:156`）、「fsync 的效果是」「提前触发一次发布」（`.claude/kb/decisions/16-发布语义.md:249`）。站得住。被漏攻的四处：

1. **没有射程。** 同一句话在 D13 里带着前提：M4 要「不带故障的崩溃状态（已定项 4 的枚举域）」（`.claude/kb/decisions/13-验证路线.md:38`），还要新实例暖机之前不让 fsync 返回：「新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回」（`.claude/kb/decisions/16-发布语义.md:196`）。带故障时条款自己认下例外：「已确认（fsync 已返回）的最后一版丢掉」（`.claude/kb/decisions/23-journal的角色与格式.md:364`，C519，用户知情认下）。故障注入里说谎的设备（写被吞、屏障被吞）报成功而没落盘：`InjectedFault::WriteIsSwallowed | InjectedFault::BarrierIsSwallowed => true,`（`crates/singlefs-harness/src/fault_injection.rs:180`）；它们「说谎的设备与报错的设备判定档不同。」（`crates/singlefs-harness/src/fault_injection.rs:170`）。按原句，这两种注入下 RefFS 会判红实现没有过错的盘面（故障注入把那一格记成设备丢的，不记成新发现）。
2. **「已接受」要写成口径。** 取号是事务进某个 checkpoint 的那一刻（`.claude/kb/decisions/16-发布语义.md:133`）。被拒绝的操作不算接受：前台分配失败「向调用者返回 ENOSPC」（`.claude/kb/decisions/23-journal的角色与格式.md:397`）。RefFS 的操作序列因此只含已接受的操作，哪些被拒是实现的结局，要由执行器交入（模型侧已有「允许拒绝的区间」那一类：`NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,` 在 `crates/singlefs-harness/src/model.rs:314`）。[推]
3. **报错要带正面半句。** D16 已定项 17 写的是「fsync 报错之后，这次发布的数据可以出现也可以不出现」（`.claude/kb/decisions/16-发布语义.md:336`）；「报错时不承诺持久」若被读成「不在允许集合里」，会把「根已落盘、系统配置轮换失败」那一格判红。故障注入已经放行它：「再放行一样：不注入时失败那一步会写出的那几版（测量跑记的那一份）。」（`crates/singlefs-harness/src/fault_injection.rs:2291`）。
4. **报错之后后续操作的承诺。** D16 已定项 17 只在两格上让调用方收到错误：「那几格上生效」（`.claude/kb/decisions/16-发布语义.md:343`）；转只读那一支「走转只读到下次挂载」（`.claude/kb/decisions/23-journal的角色与格式.md:393`），本次挂载不再接受写，所以同一次挂载里没有「后续操作」要承诺。重挂之后从恢复出的状态起一条新时间线，下界是它（M3 的单调只在不带故障时成立：`.claude/kb/decisions/13-验证路线.md:38`）。下界不会「退回到上一次成功的 fsync」：报错的 fsync 不抬 lo，也不降它。「重做又失败」那一格的收场条款没有明写成只读，我没把它算进去。[推]

### 3.2　T4：站住，漏两处，再补一条不是漏攻的提醒

1. **故障注入一截。** F2 把故障注入算进了合取，T4 原文只列了崩溃注入三截。故障注入的「崩溃」不是一个段内子集：「let all_persisted = vec![true; writes.len()];」（`crates/singlefs-harness/src/fault_injection.rs:2271`），录制流本来就只含落到盘上的写：「录制流因此恒等于真正落到内存盘上的那一串」（`crates/singlefs-harness/src/fault_injection.rs:7`）。所以这一截的 RefFS 只有 (lo = 最后一个成功的步, hi = 失败的那一步)，且说谎的注入要放宽 lo（第 1 点）。判官在这里的调用是「crash_recovery_disagreement(&allowed, newest_persisted, &read_back)」（`crates/singlefs-harness/src/fault_injection.rs:2353`）。
2. **二次崩溃一截的期望值取自实现。** 第二次崩溃判官的允许表里，挂载写出的每条根的内容取第一次崩溃之后实现自己的读回：「versions_allowed.insert(model_root_key(instance, checkpoint_txg), content.clone());」（`crates/singlefs-checker-tier/src/crash_injection.rs:1784`）。RefFS 在这一截要从「实现读回的内容」反找它落在 S_j 的哪个 j；内容相同的 S_j 不止一个时，取最小的 j 才不会误红（取大了会把合法恢复判红）。[推]
3. **提醒：RefFS 的判别力在变异那一边。** 攻方的 T4「没打中」是在正确实现可达的态上找的（一次抽样）。可达态上 RefFS 的下界 lo 被 ③ 罩住：fsync 返回意味着那次发布的根槽已持久，③ 的下界不低于它；上界在今天的流上没有判别力（正推：「上界今天没有」，`research/prompts/m3-formal-r4-sonnet-output.md:102`）。RefFS 独有的是另一类：fsync「返回得太早」的变异。把某次发布的最后一道屏障或 FUA 挪到下一步之后，崩溃枚举里会有「根槽没持久、而这一步已算返回」的态，恢复读回旧的一版：旧版是模型提交过的、内容对，③ 的下界取自持久的写、不高于那个旧版——O1 判官全绿；RefFS 的 lo 取自步边界，红。这条是我从两边的判据读出来的 [推]，没造变异跑。它给 8-5 一条必红自检：O1 加 ③ 绿而 RefFS 红的变异必须有；没有就说明 lo 没接对。推翻它的现象：造出这条变异、O1 判官也红（那说明 ③ 的下界取到了步边界）。

### 3.3　T6：站住，漏两处

1. **二次崩溃镜像上没有「挂载后那次发布」。** T5-3 的收严（比挂载后发布之前那一版）在这里没有对象，见 U6。
2. **「拒绝本身」O1 已有条款对着。** 模型已有拒绝理由 `NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,`（`crates/singlefs-harness/src/model.rs:314`）和 `pub fn answer_mount_writable_with_the_newest_root_unreadable(`（`crates/singlefs-harness/src/model.rs:1276`），条款是「有一槽读不出或本池自证不过 ⇒ 判不出、按真」（`.claude/kb/decisions/23-journal的角色与格式.md:391`）与「可写挂载每次都拒」（`.claude/kb/decisions/23-journal的角色与格式.md:407`）。T6 的「另判拒绝本身」不用 RefFS 判，也不用新写条款，指给 O1 的这几处即可。
3. 另一处要对射程说清：T6-1 那 85 个被拒的态在层 0 的枚举域里（取第三态）；崩溃注入的域是「枚举域与层 0（crate::crash）同一个：更早的段整段持久、当前段任意真子集、更晚的段一个都没持久，撕裂并进「没持久」」（`crates/singlefs-checker-tier/src/crash_injection.rs:5`），今天不取第三态，所以 Y7 落在崩溃注入时「拒绝」只会在单元区墙那一格出现：`if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall) {`（`crates/singlefs-checker-tier/src/crash_injection.rs:1553`），别的拒绝按「Some(format!("崩溃后镜像上的可写挂载拒了：{member}"))」判红（`crates/singlefs-checker-tier/src/crash_injection.rs:1558`）。射程那一句是对将来给崩溃注入加第三态而写的，今天没有样本。

## 四、U1–U7 逐格

每格先答被问的几件事，再给候选原句（零轮，没被攻过）与会推翻它的现象。

### U1　fsync 的条款句

- **一次发布里夹着多次 fsync，这句还成不成立。** 串行提交下一次发布里至多有一次 fsync 返回：fsync 本身就是触发点（「fsync 密集负载下窗口由 fsync 自身触发」，`.claude/kb/decisions/16-发布语义.md:137`），触发之后成员资格关闭（`.claude/kb/decisions/16-发布语义.md:133`），所以 lo 本身是一次发布的末尾，Q 的集合里「j = lo」与「j 是发布末尾」不冲突。多次 fsync 返回共用一次发布只发生在并发 fsync 下：「并发 fsync 的返回时刻被同 checkpoint 的其他事务拖住是组提交固有的代价」（`.claude/kb/decisions/16-发布语义.md:133`），那是并行提交的域（「是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A」，`.claude/kb/decisions/13-验证路线.md:53`）。那里每个 fsync 按自己的调用开始下标抬 lo，比实现实际给的承诺弱（公共那次发布至少盖到较晚那个下标），不会误红，只少一点判别力。[推]
- **「已经接受」是哪个下标。** 取号成功、没被拒的操作数，按单元计（一个写请求切成 N 个事务各自取号：`.claude/kb/decisions/16-发布语义.md:133`）。串行下就是 fsync 调用之前的程序序里，已返回成功的那些子操作的个数。
- **报错之后。** 见 3.1 第 3、4 点：报错的 fsync 不抬 lo、也不降它；那批操作可以在也可以不在。

候选原句（U1）：「在不带故障的崩溃状态里（D13 已定项 4 的枚举域，串行提交），fsync 返回成功之后，调用开始时刻已被接受的全部操作（取号成功、没被拒绝的，按单元计）崩溃之后必须在。fsync 报错时，那次发布带的操作可以出现也可以不出现，下界不变；报错之后本次挂载转只读，重挂之后从恢复出的状态起算。带故障的状态（说谎的设备、丢整块盘 C519）不适用这句。」
推翻它的现象：(a) 找到不带故障、串行、fsync 已返回，而合法实现读回更旧一版的态；(b) 找到两个并发 fsync 共用一次发布、其中一个返回之后崩溃，恢复缺了它调用开始之前已接受的操作。

### U2　按发布边界的前缀（F1 = Q）

**攒批落地之后边界从哪来，与 F3、D13 已定项 3 怎么同时成立。** 三条路：

| 路 | 做法 | 与 F3、D13 已定项 3 | 今天能不能写 |
|---|---|---|---|
| (a) 吃实现的发布日志 | 边界取 `publishes_in` 的结果：`pub fn publishes_in(`（`crates/singlefs-harness/src/memory_pool.rs:961`），③ 的入参就是从它数的（`crates/singlefs-harness/src/memory_pool.rs:816`） | 与枚举读同一份流，等于共用；实现把发布切错地方时 RefFS 把错的当真的，是洗白 | 能写，但违反 F3 |
| (b) 照条文写触发 | RefFS 自己按「(脏字节数 ≥ T_dirty)」与时间那一支算（`.claude/kb/decisions/16-发布语义.md:77`） | 不共用枚举代码，成立 | 写不了：脏字节的口径条款没写：「算数据单元还是节点，D16（发布语义） 已定项 5 没写」（`.claude/kb/experiments/143-一事务一单元下的journal账.md:76`）；实现也没有触发路径（`crates/singlefs-core/src/system_configuration.rs:278`）；RefFS 没有时钟，时间要历史脚本以「过了多久」这类操作交入 |
| (c) 显式发布点 | 边界只取历史里的发布调用与 fsync；harness 起挂载时把两个阈值设成触发不了 | 成立：边界来自历史（所有判官共有的输入），不来自枚举代码 | 能写；自动触发的攒批不在 RefFS 射程内，归 M2 与记录核对器（D16 已定项 4：「一次发布整体施加或整体不施加」，`.claude/kb/decisions/16-发布语义.md:111`） |

所以 Q 与 F3 能同时成立，条件是取 (c)，并把 (b) 留到 D16 补上脏字节口径、实现有触发路径之后。Q 原句如果不写边界来源，攒批一落地就会不稳：切点落在请求中间时，「u1 新、u2 旧」按条款合法，Q 的集合里没有它，会把合法状态判红；要避免这个，边界不全知道时得把请求内每个单元边界都放进边界集合，Q 就退成 P。

**一次写请求覆盖两个单元中途崩溃，Q 给什么集合，与 D23 已定项 14 冲不冲突。** 今天（一次发布）：{S_lo, S_hi} 两个态，不含「u1 新、u2 旧」。D23 已定项 14 说的是单元级的中间态**可以**有：「崩溃原子性的单位是一个单元、不是一次写请求」（`.claude/kb/decisions/23-journal的角色与格式.md:389`），是许可不是要求，所以今天不冲突。攒批之后切点在请求中间时给 3 个，前提是切点在边界集合里；不在就是上一段说的误红。第五条是恢复侧的前提：「施加的单位是一次发布」（`.claude/kb/decisions/23-journal的角色与格式.md:362`），Q 就是它在 POSIX 面上的投影，这一点 Q 比 P 站得稳。

**forks 里 P、Q 的子操作切法有两处要补。** 「⌈长度 ÷ 净荷⌉ 个子操作」只对从单元边界起的写成立，今天的入口恒从文件偏移 0 写整份（`crates/singlefs-core/src/transaction.rs:4784`）所以碰巧对；一般的写触及几个单元取决于起点落在哪，kb 里我没读到写这件事的条款，要用先去查 [推]。整份变短时尾部单元在同一次发布里释放（`crates/singlefs-core/src/transaction.rs:4792`），那一步是不是一个子操作、排在哪，P、Q 的句子没说 [推]。
候选原句（U2）：「RefFS 的允许集合 = { S_j : j = lo，或 j 是某次发布的末尾，lo ≤ j ≤ hi }；操作取到一个单元事务；发布的末尾只取历史里的显式发布点（发布调用与 fsync），自动触发的发布不在射程内。」推翻它的现象：在入库实现上找到一个由显式发布点之外的原因产生的发布边界，使合法恢复落在集合之外。

### U3　合取（F2）

- **独立性有没有被打折，要不要写进条款。** 打折，要写，而且折的不止 ③ 那一侧：③ 的入参取自「let newest_persisted = newest_persisted_root(&writes, &persisted)」（`crates/singlefs-checker-tier/src/crash_injection.rs:881`），RefFS 的 (lo, hi) 取自 `step_containing`（`crates/singlefs-checker-tier/src/crash_injection.rs:751`），两边都从同一份录制流与同一批崩溃点来。D13 已定项 3 禁的是共用枚举**代码**，RefFS 的源码不含它，所以不违反；残余是**输入同源**，条款句见 2.3 第 3 点。
- **三处调用点各自怎么合取。**

| 调用点 | 位置 | RefFS 一票的输入 | 要注意 |
|---|---|---|---|
| 崩溃镜像 | `crates/singlefs-checker-tier/src/crash_injection.rs:881`（O1 判官在 902） | lo、hi 取自翻译层 | 上界在今天的流上没有判别力 |
| 二次崩溃 | `crates/singlefs-checker-tier/src/crash_injection.rs:1826`；允许表在 1784 | 下界 = 第一次恢复读回内容在 S_j 里最小的那个 j | 期望值取自实现自己的读回，取最小 j 才不误红 |
| 故障注入 | `crates/singlefs-harness/src/fault_injection.rs:2353` | lo = 最后一个成功的步，hi = 失败的那一步 | 盘面是「全落」：`vec![true; writes.len()]`；说谎的注入要放宽 lo 或这一票弃权，否则 T1 句子在这里为假 |

- **回退那段 9 / 4 108 个态，谁在判。** 今天没有红：实现读回回退后（2.2）。假想一个读回回退前的实现，红的是 O1 判官 ③；RefFS 一票不红。③ 没有变异行，8-5 要给它补一行，否则「合取罩住这一类」只是推的。
- **F2 是被 F3 推出来的**（见 2.2、第六节）。

### U4　RefFS 另起、单独一个包（F3、F4）

- **D17 已定项 4 的原句。** 现句是「O1（内存模型对拍（HashMap 理想文件系统）） 扩成的 RefFS 定义 fsync 的持久性承诺」（`.claude/kb/decisions/17-实现分层与第三方管道.md:69`）。候选改句（零轮）：「「入口」的可执行定义由 RefFS 给——按 D13 已定项 3「做到极致、自己处理持久化与崩溃语义」，由 RefFS 定义 fsync 的持久性承诺；RefFS 是另起的语义级实现（只照 kb 条文写，不由 O1 扩出，不与实现共用崩溃点枚举的代码）；不另起 O3。」这样改之后 D13 已定项 6 的 O1 一行也要跟：O1 的职责写的是「POSIX 可见语义：内容、大小、目录结构」（`.claude/kb/decisions/13-验证路线.md:117`），RefFS 另起之后两个 oracle 都在判 POSIX 内容，要写一句分工：无崩溃处的内容期望归 O1，崩溃之后的允许集合归 RefFS。D13 已定项 3 的射程那句「fsync 的持久性承诺由 RefFS 的崩溃语义定义」（`.claude/kb/decisions/13-验证路线.md:67`）不用动。
- **依赖：用不用 `singlefs-format`。** 操作取到单元时要用单元净荷：`pub fn data_unit_payload_capacity_in_bytes() -> u64 {`（`crates/singlefs-harness/src/model.rs:280`，值是格式常量之差）。可以由调用方传进来（RefFS 只依赖 std），也可以引常量（依赖 std 与 `singlefs-format`）。门禁条件写成「闭包 ⊆ {自己, singlefs-format}」，不写成「恰好等于」。
- **门禁的 checker-implementation-disjoint 现在判四条**：依赖闭包求交、checker 零引 core、format 无分支、harness 不依赖 checker 档：「singlefs-harness 的传递闭包（dev-dependencies 也算）里没有 singlefs-checker-tier」（`.claude/gate.d/checker-independence-and-sync.sh:24`）。RefFS 要加的三条（候选，零轮）：⑤ RefFS 的传递闭包 ⊆ {自己, `singlefs-format`}；⑥ RefFS 源码零处引 `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier`，经 `package =` 改名的别名同样算；⑦ `singlefs-core` 与 `singlefs-checker` 的闭包里没有 RefFS（核心共用 RefFS 等于实现与参照共用代码；池级 checker 依赖它等于 O2 与 RefFS 不独立）。⑤⑥ 的形状与 ④ 同类，有先例。还要改「五个 crate 路径写死，改名或搬家时判红、不跳过。」（`.claude/gate.d/checker-independence-and-sync.sh:30`）。包边界管不到调用方交进去的 (lo, hi) 与历史，那一层归 2.3 第 3 点的覆盖检查。
- **操作集合照什么取，取今天那一组算不算「拷 O1」。** 今天历史的操作是八种（`crates/singlefs-harness/src/history.rs:376` 起的枚举）。有 POSIX 效果的只有第一个文件与覆盖写；空发布、关闭重挂、冷启动没有 POSIX 效果（空发布是一次不改状态的发布边界，另两个是换会话）；回退、抬 F、崩溃抛弃根不是 POSIX 操作，回退的目标还是「RingRoot { index_from_newest: u64 },」（`crates/singlefs-harness/src/history.rs:345`）这种取根环第几条的盘级量，语义级的 RefFS 解不出它指哪一版。取「历史的操作面」不算拷 O1：那是所有判官共有的输入，D17 已定项 5 的操作面也是「它对外是一组操作（mkfs、打开、写 extent、发布 / fsync、恢复）」（`.claude/kb/decisions/17-实现分层与第三方管道.md:82`）。算拷的是回退那一项：由 O1 的版本表交一个「还原成某一版内容」给 RefFS，等于把 O1 的读法借过来。所以第一版 RefFS 的操作集合取第一个文件、覆盖写，加空发布这个边界标记；含回退的历史上 RefFS 那一票弃权，回退归 O1。[推]

### U5　挂载后再恢复的逐字段表

设 img1 是崩溃镜像；R1 是对 img1 只读恢复的结果，带所选根的实例代号 i1、txg t1（重放之后那个根的）、卸载记号 m1、F、F1（F 生效值）、树 ID 水位 w1，以及 W1 =「这次重放施加的、属于 i1 的最大事务号」、rows1 =「所选根指着的实例表行」；Tm = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg)；N 是挂载取的号。R2 是对「img1 上只挂载、不发布」之后的盘只读恢复的结果（v2a，取法见 U6）。

先说判决那一句有两处写窄了。判决写：「必须相同（POSIX 状态、池身份；没有卸载的动作下卸载记号、F、F 生效值）」（`research/prompts/m3-formal-r4-main-verification.md:44`）。攻方的改法是「抬 F（含准入推的抬 F）时必须相等」（`research/prompts/m3-formal-r4-opus-output.md:131`），括注被丢了。

| # | 字段 | 能直接断言的式子 | 依据 |
|---|---|---|---|
| 1 | POSIX 内容与存在性 | C(R2) = C(R1) | 必须相同。攻方量到挂载下「读回内容（posix.content）」全同（`research/prompts/m3-formal-r4-opus-output.md:109`，副本上量的） |
| 2 | 池身份、区域归属、尺寸 | 相同 | [推] 挂载不改它们 |
| 3 | 实例代号 | instance(R2) = N，N = max(各盘系统配置里自证槽的实例代号, 根环里全部根的实例代号) + 1，两个量都从 img1 读 | 「根环里全部根记录的实例代号) + 1」（`.claude/kb/decisions/18-块里携带什么信息.md:316`） |
| 4 | checkpoint_txg | txg(R2) = Tm + n，n = 挂载录制流里的根槽写条数（写行、暖机、抬 F 各一次发布，每次加一） | 「第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`.claude/kb/decisions/23-journal的角色与格式.md:370`）、「fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号」（`.claude/kb/decisions/16-发布语义.md:156`）[推]；不精确时退成 txg(R2) ≥ Tm + 2（两盘池） |
| 5 | 根槽位置 | 不另判，是 txg 的函数 | D16:156 同一句 |
| 6 | 卸载记号 | marker(R2) = 0 恒成立；与 R1 相同 ⟺ m1 = 0 | 「flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退，以及由记录施加出来的那一版。」（`crates/singlefs-core/src/root_record.rs:26`）；D13 的 M2 已把它排除在相等之外：「其余字段等于所选根，卸载记号除外」（`.claude/kb/decisions/13-验证路线.md:32`）；正推原表写的是「v1 ∈ {卸载写的, 不是}」（`research/prompts/m3-formal-r4-sonnet-output.md:181`） |
| 7 | F（根上带的）与 F 生效值 | F2 ≥ F1；F2 = F1 ⟺ 这次挂载没推抬 F，推没推看挂载录制流里有没有「先写系统配置里的新 F、再发带新 F 的根」那一串 | 「可写挂载的准入不够时，写行那次发布照走切换预留，写完行之后推抬 F 的空发布、再判」（`.claude/kb/decisions/16-发布语义.md:42`）；代码：`fn push_floor_raises_after_the_row_publish<Device: BlockDevice>(`（`crates/singlefs-core/src/mount.rs:4277`） |
| 8 | 树 ID 水位 | w2 ≥ w1；挂载不发第一个文件时 w2 = w1 | 攻方量到「只在发第一个文件时不同」（`research/prompts/m3-formal-r4-opus-output.md:114`）；判决的三类里没有这一行 |
| 9 | 树表里 inode 树、extent 树的根指针 | R2 树表里这两个指针 = R1 树表里的（挂载写的全是空发布） | 「比的是树表里这两棵树的条目、不是树表单元自己的落点」（`.claude/kb/decisions/16-发布语义.md:46`）；「只数改过用户可见状态的根，推空与抬 F 产生的空发布根不算」（`.claude/kb/decisions/16-发布语义.md:29`）[推，没量] |
| 10 | 实例表行 | rows2 = rows1 加上 { row(i) : i ∈ [max(i1, 1), N) }；row(i1) = (i1, t1, W1)，其余 (i, 0, 0)；实例 0 不写行；一条都没施加时 W1 = 0 | 「所选根那个实例写 (i, 重放之后那个根的 checkpoint_txg, 属于实例 i 的、被这次重放施加的最大事务号)」与「严格介于所选根的实例与新实例之间的实例写 (i, 0, 0)」（`.claude/kb/decisions/18-块里携带什么信息.md:309`）；代码「let first_row_instance = previous_row.instance.0.max(1);」（`crates/singlefs-core/src/mount.rs:3141`）、「selected_root_txg: effective_root.checkpoint_txg,」（`crates/singlefs-core/src/mount.rs:4708`）与「applied_transaction_high_water: journal.maximum_applied_transaction,」（`crates/singlefs-core/src/mount.rs:4709`） |
| 11 | 扫描报告 | R2 的 prefix_applied、above_water 为 0 | [推] R2 的根比环里全部记录新；攻方量到施加条数「只在 R1 由记录重建的状态上不同」 |
| 12 | 分配记录 | 不比整表；断言 POSIX 相等（行 1）与池级 checker 对 R2 镜像判绿两句 | 攻方只比了整表（全不同），自己写「后者才是不该模掉的那一半」；没有这两句，「模掉」就没人罩 |
| 13 | 系统配置槽 | 不比 | 取号与每个 checkpoint 各轮换一次（`.claude/kb/decisions/16-发布语义.md:179`）；归池级 checker |

三条回答：
- **「模掉并改判关系」那七样各判什么**：实例代号、txg、根槽位置、扫描报告是行 3、4、5、11 的式子；树指针要拆开，表单元自己的指针模掉、两棵用户可见树的指针按行 9 判相同；分配记录按行 12；系统配置槽不比。
- **实例表行写全**：行 10 就是写全的形态；正推没读透的那一点（W 取「覆盖的」还是「施加的」）条款有答案：施加的（D18:309）。
- **第一轮之外有没有该相等却被模掉、不该相等却写成相同的**：该相等被模掉的有行 9、行 8；不该相等却写成相同的有行 6、行 7。攻方的「必须相同」那类里取的量在没有带卸载记号的 R1 和不缺空间的池上成立，写成条款要带这两个前提。

推翻这张表的现象：(1) 行 6：造一段历史，在正常卸载之后崩溃，R1 带记号而 R2 不带，这一行就是「恒 0」不是「相同」；(2) 行 7：在快满的池上挂载，F2 > F1 而挂载没有卸载；(3) 行 9：挂载之后两棵树的指针变了而 POSIX 内容没变；(4) 行 10：在无文件历史上 rows2 与式子不符。判别入库装置前这些都只是读码，攻方的数是副本上的，进 kb 之前要在入库装置上重做。

### U6　挂载透明的射程与二次崩溃（T6、F6）

- **二次崩溃镜像上比的对象。** 取「只挂载、不发布」的 v2a，不用「挂载后那次发布之前那一版」：后者要读环里非最新的那条根的内容，`recover` 的返回里我没看到它，攻方原型里是自己取的（字段名 `posix.content_of_the_version_before_the_post_mount_publish`）[推]。v2a 现成：「fn content_read_after_the_uninterrupted_mount(」（`crates/singlefs-checker-tier/src/crash_injection.rs:1246`）把挂载写落到崩溃镜像上再只读恢复。对二次崩溃镜像 img2，比的两端是 recover(img2)（v1 的类比）与对 img2 再挂载（不发布）之后的恢复。
- **重叠与独有的判别力。** 三个判官各管一件：二次崩溃判官判 recover(img2) 落在允许表里（`crates/singlefs-checker-tier/src/crash_injection.rs:1826`）；I-8.4 的 R-换判 remount(img2) 的内容等于没崩过的那次挂载之后的内容（`crates/singlefs-checker-tier/src/crash_injection.rs:1881` 起），条款是「任一 journal 记录被重放两次与重放一次，产生的读出内容相同」（`.claude/kb/invariants.md:93`）；第一次崩溃镜像上的挂载透明判 v2a 等于 v1。R-换加第一次的挂载透明蕴含 C(remount(img2)) = C(v1)。F6-加二次比的是 C(remount(img2)) = C(recover(img2))，在 R-换成立时等价于 C(recover(img2)) = C(v2a)：「挂载做到一半崩溃，不改变恢复读回的内容」。它严于二次崩溃判官（允许表里「根 ≥ 下界」放行更旧的已提交版），所以有独有的判别力，但只在 v1 由记录重建、根槽比它旧的态上能与「允许表」分出来：攻方量到这类态每段 3 个（s5 是 6 个）（`research/prompts/m3-formal-r4-opus-output.md:119`，副本上量的）。内容一半的代价是一次已有两个值的比较；字段一半要从两次恢复报告取字段，是另一笔，`content_read_by_recovery` 今天只返内容。[推]
- **拒绝本身按哪条判。** 按 D23 已定项 14 的「有一槽读不出或本池自证不过 ⇒ 判不出、按真」（`.claude/kb/decisions/23-journal的角色与格式.md:391`）与 C331 取甲的代价「可写挂载每次都拒」（`.claude/kb/decisions/23-journal的角色与格式.md:407`），实现在 O1 的 `answer_mount_writable_with_the_newest_root_unreadable`（`crates/singlefs-harness/src/model.rs:1276`）；RefFS 没有系统配置，不判它。

### U7　C191 那一句（F5）

- **「接上」指哪一层。** 不需要「把根翻成 POSIX 状态」的一层：verification-build 写的是 O3「对同一批崩溃状态独立判一遍；同步开始统计它与 RefFS 的不一致事件」（`.claude/kb/verification-build.md:234`），D13 已定项 6 里 O3 判的是「崩溃点截断后的每个磁盘状态是否落在规约允许的集合里」（`.claude/kb/decisions/13-验证路线.md:119`），两边都是对同一个（历史、崩溃点）给判定行，比的是判定。翻成 POSIX 内容的那一步若要有，它在 O3 自己里面（另一种语言的独立解析，D13 已定项 6 已要求），不是第三份实现。攻方说 O3 交的是根或版本、RefFS 交的是 POSIX 内容的允许集合（`research/prompts/m3-formal-r4-opus-output.md:261`），这只在把 O3 读成版本执行器时才成立。[推]
- **与「任一对累计 ≥ 5 次」怎么并存。** 判据照旧，作用在已有的对上（C191 的原句「任一对累计 ≥ 5 次不一致而判」，`.claude/kb/checks-owed.md:168`）。候选句要避开两处矛盾：C191 这一行现写「今天只对 O3（独立规约执行器） × RefFS 一对立了格」（`.claude/kb/checks-owed.md:168`），新句说「这一对不立」，要把「立了格」改成「登记为空格、等 O3」；C447 写的 O3 事件对手是 checker：「与 checker 的判定逐个比对，不一致的落进」C191（`.claude/kb/checks-owed.md:366`），与 C191、verification-build 的 RefFS 不是同一个，新句要选定一个。
- **O1 判官不当替身后的缺口。** RefFS 另起之后，D13 已定项 13 与 C191 第二对里的「model」是 O1 还是 RefFS 没人写明；今天只有 O1 存在，不钉住就悄悄绑到 O1。候选句（零轮）：「O3 有代码且接上 RefFS 之前，O3 × RefFS 这一对登记为空格、不计事件；O1 判官不当替身。D13 已定项 13 与 C191 里的 model 指 O1，RefFS 与 O1 在合取处的不一致照常报红，不记入同源性计数。」推翻它的现象：O3 落地之后发现它的判定行与 RefFS 对不上同一个（历史、崩溃点）键。

**同类先例一览。** 有先例的：任意子集出局限串行（D13 M3，5.1）；门禁⑤⑥的形状（门禁④，U4）；故障注入按说谎与否分档（`the_device_lies`，U1、U3）；卸载记号排除在相等之外（D13 M2，U5 行 6）；实例表行的规则（D18 已定项 11 与 `instance_rows_to_write`，U5 行 10）。没找到先例的：U2 里「边界只取显式发布点」（kb 与代码里我没找到别处拿显式发布点当崩溃边界的做法）；U6 把挂载透明拆成内容一半与字段一半；U7 把「model」钉给 O1。这三处只是我的推，不是先例支撑的。

## 五、替被判出局的候选辩护

### 5.1　任意子集丢法（S、Sd）

出局的理由有两条：与 D16 已定项 12 不相容（「根 + journal 任意前缀，replay 之后合法」，`.claude/kb/decisions/16-发布语义.md:262`），和会放过「前面的没落、后面的落了」（正推的坏2）。辩护：

- **它在并行提交那一域里是正确形态。** D13 已定项 2 自己写「前缀」只在全序上成立：「是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A」（`.claude/kb/decisions/13-验证路线.md:53`）。两个并发的、碰不同文件的操作恢复后任意子集都合法，Sd（再加同路径的前一个操作也得在）就是它的 POSIX 面。第一轮 T2 的「不相容」成立在串行前提上，判决写成条款时要带「串行」二字，否则开并行那天条款是错的。
- **它是 P、Q 的超集，不会把合法状态判红。** Q 在边界不全知道时本来就得退成 P（U2）；P 再放宽到 S 只是多放过一类，不会多误红。
- **它输在哪**：串行下放过坏2（rename 落了数据没落）。这是判别力的损失，不是正确性的损失。
- 同类先例：D13 写「M3 的单调推论只在串行提交、不带故障的崩溃状态里成立」（`.claude/kb/decisions/13-验证路线.md:38`），并写「开并行提交或把域放宽到带故障时重开」（`.claude/kb/decisions/13-验证路线.md:33`）。任意子集出局照这个写法写：限串行、开并行重开。
- 推翻我的辩护：找到串行、不带故障下合法实现落在 S 里而不在前缀集里的态——那就是 P、Q 太紧，不是 S 该留。找不到这种态之前，S 只在并行域有位置。

### 5.2　O1 剥原地（F3-剥原地）

出局的理由是「O1 的盘级对拍整块没了」，丢多少「整表变异是重型，推」（`research/prompts/m3-formal-r4-forks.md:66`），没量。辩护：

- **它是 D13 的字面读法。** O1 一行写的是「POSIX 可见语义：内容、大小、目录结构」（`.claude/kb/decisions/13-验证路线.md:117`），形态是「不建崩溃」（`.claude/kb/decisions/13-验证路线.md:125`）；今天的 O1 比这个定义多带了盘级一半。剥原地等于把 O1 还原成 D13 的定义、再加崩溃语义，D17 已定项 4 的字面不用改（岔路单对剥原地记的是「的字面不用改」，`research/prompts/m3-formal-r4-forks.md:66`）。另起之后 O1 与 RefFS 同时占「判 POSIX 内容」这一格，要新写一句分工（U4）。
- **只有一份 POSIX 读法。** 另起加 F5-不算，是两份各自写出来的 POSIX 读法并存，它们之间的同源没有任何计数；合取处的不一致只以「红」的形式出现，一致不留痕。剥原地没有这一对。D13 已定项 13 与 C191 第二对里的「model」也不用另外钉住（U7）。
- **D13 已定项 3 与 Y2 两头它都躲得过**（2.3 第 1、2 点）。
- **它输在哪**：盘级对拍要么搬走要么丢；F2 的合取要求 ③ 留在 O1（`research/prompts/m3-formal-r4-forks.md:55`），剥原地后 ③ 只能改走 F2-下界入参，那要给 RefFS 多一个整数，与 (lo, hi) 只两个整数相冲。岔路单两边的行数都是推的，没有谁更省的数。
- 一个没被攻过的变体（零轮，起名 F3-拆文件）：把 `model.rs` 按「内容与判官」「盘级」拆成两个文件，RefFS 从前一个长出，盘级那个文件留在 O1 名下，不删任何变异行靠的代码。它与剥复制同源，代价是 RefFS 与 O1 的内容半是同一份读法。没量。
- 推翻我的辩护：量到「摘掉盘级那一半，123 行变异里仍红几行」很少（岔路单要的那个数）、或另起之后的两份 POSIX 读法在合取处出现过一次「读法不一致」的红——后者说明两份真的独立，另起的好处成立。

### 5.3　F6-只第一次

出局的理由是加二次在被第二次崩溃撕过的镜像上也判挂载透明。辩护：

- **内容一半不输。** 第一次镜像上的透明（v2a 等于 v1）加 R-换（remount(img2) 等于没崩过的挂载之后）已经给出 C(remount(img2)) = C(v1)；加二次多出的内容判别力，只在 v1 由记录重建、根槽比它旧的态上（U6，攻方量到每段 3 个，副本上量的）。
- **字段一半在二次镜像上没有现成的证据。** U5 行 4、7 要挂载录制流（写了几次根槽、有没有抬 F 那一串）；二次崩溃镜像上再挂载用的是「pub fn writable_mount_alone_on(」（`crates/singlefs-checker-tier/src/crash_injection.rs:1209`），它用的是 `SparseBlockDevice`、不接录制器，只交回挂载后的池。要加二次的字段一半，这个函数要改成交回录制流。第一次镜像那一边有：`WritableMountOnTheCrashImage` 带 `mount_operations`（`crates/singlefs-checker-tier/src/crash_injection.rs:1282` 起）。
- **相位也有半截状态。** 取号相位的镜像里系统配置已写、行与根没写（烧号），N 取自镜像的系统配置而不是挂载后的根环；行 3、10 要按 img2 自己的恢复报告现算。不是做不了，是多一处可以写错的地方，误红风险在这里。[推]
- 用户已定的是加二次，我不改；把它拆成两半写进原句更稳（零轮，起名 F6-二次内容、F6-二次字段）：内容一半几乎白送，字段一半等 `writable_mount_alone_on` 交回录制流之后再做。

## 六、用户定下的 F1–F6，写成原句哪里站不住

| # | 站不住的地方 | 补什么 |
|---|---|---|
| F1 Q | 没写发布边界来源；攒批之后边界在请求中间时误红合法的「u1 新、u2 旧」；P、Q 那句里的切法只对从单元边界起的写成立 | 边界只取显式发布点；自动触发的发布不在射程内（U2 候选原句） |
| F2 合取 | 被 F3 推出来，不是独立的一问；故障注入一截说谎的注入下 T1 句子为假；二次崩溃一截期望值取自实现的读回；「9 / 4 108 态在合取下判红」的前提不实（今天没有红） | 三处调用点各写一行（U3 表）；说谎注入放宽 lo；③ 补一行变异 |
| F3 另起 | D17 已定项 4 与 D13 已定项 6 的 O1 一行要一起改，分工句要写；操作集合含回退时 RefFS 解不出目标；(lo, hi) 翻译层的残余同源没写；与 F5-不算并起来，两份 POSIX 读法的同源无人计数 | U4 的改句与操作集合；2.3 第 3 点的候选句 |
| F4 包 | 「依赖只有 std 与 `singlefs-format`」要写成闭包 ⊆（语义级的 RefFS 可能只用 std）；门禁今天的四条没有一条罩它；写死的五个路径要加一个；包边界管不到调用方交进去的 (lo, hi) 与历史 | 闭包 ⊆；加 ⑤⑥⑦；覆盖检查（U4） |
| F5 不算 | 与 C191 现行「今天只对 O3 × RefFS 一对立了格」自相矛盾；C447 里 O3 的事件对手写的是 checker；「model」指谁没钉 | U7 的候选句，三处一起改 |
| F6 加二次 | 「判同一条」搬不过去：二次镜像没有「挂载后那次发布」；字段一半缺挂载录制流；半截挂载的相位要另算 | 比的对象取 v2a 的类比；拆成内容一半与字段一半（5.3） |

## 七、没做什么

- 没改用户定案，没替主 agent 撤销或维持判决；只给复核结果、候选原句与会推翻它的现象。
- 没量任何数，没编译、没跑重型测试与门禁。引的数（9 / 4 108、65 634、每段 3 个态、123 行变异等）都是攻方在副本上量的或岔路单现数，没在入库装置上重做；进 kb 之前要重做。
- 没读 `/tmp/claude-1000/m3-formal-r4-forks/report.md`，没重数 F1–F6 的代价。
- 没造变异去验 3.2 第 3 点、U3 的必红自检；它们是读判据推的。
- 没查 kb 里有没有写 extent 起点与单元边界对齐的条款（U2 末，已标 [推]）；「重做又失败」那一格的收场条款没有明写，没算。
- 没写模型，`research/prompts/m3-formal-r5-sonnet-model/` 没建。草稿目录 `/tmp/claude-1000/m3-formal-r5-defense/` 里只有两份核引文行号的小脚本（`q.py`、`q2.py`），没有编译目录与仓副本，没有要删的东西。

## 八、cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/m3-formal-r5-sonnet-output.md --root /tmp/claude-1000/m3-formal-r5/tree --root . --background research/prompts/_m3-formal-r5-background.md` 末行（交回前最后一次跑）：

```
  ✓ 核了 80 处引文，对上 80 处，没判 0 处（逐处列在上面）
```
