# 转述核对表：m2-refusals-presumed-r1-local-attack（2026-09-22 UTC / 2026-09-23 JST）

逐条核对 `research/prompts/m2-refusals-presumed-r1-local-attack.md`（R5–R8，20 条 Fact、4 道
Judgment）里每一句英文转述与中文原文/源码原文。行号现查工作区版本（`git status` 显示这些文件都是
未提交的工作区改动）。格式：英文项（Fact/Judgment 编号）/ 原文文件:行 / 首稿缺的 / 定稿理由。

提示本身不引用任何代码行号（按共用约束「答复不写代码行号与文件行号」一节与本轮定义「提示里明令
答复不写代码行号与文件行号」），只在这份核对表里写死来源文件与行号；本表出现的行号一律是本地
攻方腿这次现查工作区所得，标「模型自给、未核」的是样本自己写出来的行号（本轮样本被明令不写行号，
预期不会出现，若出现则在运行记录里如实标注）。

## Fact 1（拒绝的三关：拒得早/拒得准/拒得明）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `research/prompts/_m2-refusals-presumed-r1-background.md:22-26`（「一条拒绝要站得住，得同时过三关……1. 拒得早：在动任何状态之前拒，盘上逐字节不变；2. 拒得准：被拒的那一族历史里，没有一条是零故障可达、而且用户有正当理由要做的；3. 拒得明：错误成员的名字说得出拒的是什么，不与别的形态混用。」） |
| 首稿缺的 | 「⚠️ 『拒绝优先于猜』不是万能出路」这句引子与「缺一关就不算……而是一个误拒」的因果框架未逐字译入 Fact 1，只在派发提示背景段与 Judgment 用语里体现「wrongful refusal」一词 |
| 定稿理由 | 三关本身（early / accurate / clear）逐字保留，是本轮四列判据里①②③三列的直接依据；「误拒」这个后果词已经在 Fact 1 首句用「rather than being a wrongful refusal」带出，不需要重复整段引子 |

## Fact 2（该写条款的判据）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `research/prompts/_m2-refusals-presumed-r1-background.md:92`（「一格算『该写条款』，当且仅当说得出两个都说得通的不同选择，而且它们在盘上字节或可达历史上分得开。分不开的（两种选择写出同样的字节、走到同样的历史），不立条款，登记成实现细节。」） |
| 首稿缺的 | 无遗漏；「for at least some input」是补的限定词，原文「分得开」本身没有显式量词 |
| 定稿理由 | 见下方「多出来的限定词」一节 |

## Fact 3（R5 题面）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `research/prompts/_m2-refusals-presumed-r1-background.md:53` 第二列（「第一个文件版本写之前要不要核上一版记录」） |
| 首稿缺的 | 无中文原句可逐字对应更多细节，本稿按背景材料第二节的观测（暖机两次、`publish_first_file`）把题面具体化为「right after the second of two warm up publishes that occur during the first ever writable mount」，这几处限定词来自 `crates/singlefs-core/src/transaction.rs:1465`（「第一个事务（字节表七 t1..t8 + 六 + 七）」与 1469 行「两次暖机之后」），不是题面原句自带 |
| 定稿理由 | 题面原句「上一版记录」在没有上下文时指代不明，补齐「两次暖机之后」这个限定词是为了让模型准确定位是哪一次发布，不补的话模型可能误判成任意一次发布之前的核对 |

## Fact 4（publish_first_file 的控制流）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/transaction.rs:1481-1493`（`JournalRecord::parse` 解析 `previous_record_bytes` 取 `(checkpoint_txg, counter)`；`follows_directly` 判 `previous_txg.0 + 1 == FIRST_TRANSACTION_TXG && previous_counter + 1 == FIRST_TRANSACTION_TXG`；`if !follows_directly { return Err(...) }`；1494 行起才调用 `publish_version`） |
| 首稿缺的 | `JournalRecord::parse` 的第二个参数 `unit_filesystem_identifier(&pool.parameters.filesystem_identifier)`（解析时顺带核 fsid）未译入——这是「解析」这一步内部还核了 fsid 这一个事实，与「有没有核上一版记录」这个是非题本身无关，不影响四列判定 |
| 定稿理由 | fsid 核对是 `parse` 函数自己的通用契约，不是这条拒绝专属的逻辑，补进来会让模型误以为 fsid 不符是这条拒绝的独立触发条件之一；四列判据只需要知道「解析失败或两个数值对不上就整体拒绝、不调用写函数」这一层 |

## Fact 5（R5 无条款 + 历史成因）

| 项 | 内容 |
|---|---|
| 原文文件:行 | 无条款：`crates/singlefs-core/src/transaction.rs:1468-1469`（「# Errors / txg 与 jsn 写死 3……只接得上 txg 2、jsn 2 的那一版」的上下文里没有引用任何 kb 条款编号，是本地攻方腿现查 `.claude/kb/decisions/` 得到的独立观测，非某一句注释的直接翻译）；历史成因 `research/prompts/_m2-refusals-presumed-r1-background.md:53` 第四列（「无条款。它是三方前一轮攻方腿打中『两个系统配置槽坏加一次崩溃之后写死 txg 3 的文件版本返回成功却读不回』之后加的」） |
| 首稿缺的 | 「三方前一轮攻方腿」未点出具体轮次名（`m2-emptypool-nonempty-r1`，攻击编号 Z3-A，出自 `transaction.rs:1469` 注释「m2-emptypool-nonependent-r1 云端攻方腿 Z3-A」），泛化成「a different independent reviewer, in an earlier round of the same kind of review」 |
| 定稿理由 | 具体轮次名与攻击编号对模型不可核（它读不到那一轮的产物），保留下来不增加可判性；「两个系统配置槽坏加一次崩溃」「返回成功却读不回」这两个关键限定词逐字保留，它们是判断这条拒绝历史成因是否合理的核心事实 |

## Fact 6（R6 题面）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `research/prompts/_m2-refusals-presumed-r1-background.md:54` 第二列（「取号算一次还是两次」） |
| 首稿缺的 | 无中文原句可逐字对应更多细节，本稿按 Fact 7 的机制把题面展开成「计算一次」vs「为准入决策算一次、写之前再算一次、不等就拒」两个分支 |
| 定稿理由 | 题面原句本身就是二选一问句，展开写法没有增删限定词，只是把「一次 / 两次」具体成两种机制描述，方便模型对号入座 |

## Fact 7（establish_instance 的控制流）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/mount.rs:1035-1076`（`establish_instance` 函数：1051 行 `instance_generation_to_acquire(&pool)` 算一次；1053-1062 行 `rows_written`、`refuse_instance_rows_on_version_without_file`、`refuse_formatted_pool_mount_not_shaped_like_the_first_transaction`、`refuse_publishes_before_acquisition_that_do_not_pass_admission` 用这同一个值判定并可提前拒绝；1063 行 `acquire_expected_instance(&mut pool, instance_to_acquire)` 把这个值当 `expected` 传入；该函数内部 `instance_generation_to_acquire(pool)` 独立重算一次，见 `transaction.rs:411`） |
| 首稿缺的 | 无遗漏；「before any refusal check runs and before anything is written」是对 1051-1063 行代码顺序的直接观察，不是某一句注释的翻译 |
| 定稿理由 | 按代码控制流原样转述，Judgment 2 判定「写之前重算一次」这个机制是否存在，正是要看这段控制流本身 |

## Fact 8（不匹配的字段与瞬时读错机制）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/transaction.rs:392-393`（「按调用方判定时算出的号取号没做成……写之前重算出的号与调用方判定时算出的号不同（两次读系统配置之间一次瞬时读错就够：读错的槽当作没有）：一个字节都没写。」） |
| 首稿缺的 | 「(m2-emptypool-nonempty-r1 云端攻方腿 Z2：两块盘第 2 次读系统配置槽 0 各报一次瞬时读错，判定看到号 1 放行、取号读到号 2 写进两盘)」这条具体历史案例（`transaction.rs:402-403`）未译入 Fact 8，只译了机制本身（「一次瞬时读错就够」） |
| 定稿理由 | 具体历史案例是另一条腿（云端攻方）上一轮已经打过的攻击，与本轮 Judgment 2 要判的「这个机制本身算不算零故障可达」是同一件事的另一种问法；不重复列出具体案例，避免把模型的判断锚定成「抄已有案例的结论」而不是自己独立判断这个机制的性质 |

## Fact 9（错误命名的映射关系）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/mount.rs:1063-1076`（`match failure { ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite { expected, recomputed } => MountError::InstanceGenerationChangedBeforeAcquisition { expected, recomputed } }`，字段原样传递） |
| 首稿缺的 | 无中文注释可对应，直接观察源码的一对一映射 |
| 定稿理由 | Judgment 2 第三列（命名对不对得上）需要知道 mount 层的错误名是不是凭空另起的，还是原样承接 transaction 层的失败原因；这段映射是可从源码直接核验的机械事实 |

## Fact 10（C322 在欠账表里的位置与还清依据）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/checks-owed.md:445`（`### 已还清` 分节标题）；`:492`（C322 那一行，「2026-09-14 三轮三方后用户收尾弹窗定案：D23（journal 的角色与格式） 已定项 16 加取号之后那道屏障与两条实现义务，D18（块里携带什么信息） 已定项 11 写死取号读法、回卷号与屏障报错，D22（单元原子性怎么合成） 已定项 16 注逐盘 +1」） |
| 首稿缺的 | D22（单元原子性怎么合成） 已定项 16 那一条（「注逐盘 +1」）未展开说明具体内容，只译成「noting that the number is incremented by one separately on each device」；`crates/singlefs-core/src/transaction.rs` 的 `acquire_instance` 实现引用、I-7.7（系统配置实例代号不低于根环） 改动、判别力自证（层 0 262165 状态、`instance_acquisition.rs` 四条测试）三处均未译入 |
| 定稿理由 | 「D22 已定项16 注逐盘+1」与本轮判定的「写之前重算一次、不等就拒」机制关系最远，只需要知道它是三条清偿条款之一即可，不需要展开细节；后三处（代码落点、checker 改动、判别力自证）是 C322 这笔账「已经被验证过」的旁证，与 Judgment 2 要判的「这个具体机制有没有被这三条清偿条款覆盖」无直接关系，属于可以不抄的辅助信息（这一条本身在核对表里单独占一行，不属于「摘句」——它是决策文本旁边独立的欠账登记表条目，不是决策正文本身） |

## Fact 11（D18 已定项 11「可写挂载的顺序」bullet）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/decisions/18-块里携带什么信息.md:313`（整条 bullet，见下方全文） |
| 首稿缺的 | 「独占打开池中过半的设备——任意两个过半集合相交，每设备的独占打开才成为池级互斥；这条与它的代价（4 盘池只剩 2 块可写时只能只读）是 D2（RAID 条带策略） 已定项 13 定的，不是已定项 11」这个关于「过半」这条准入子项本身出处是哪条决策的说明未译入，只译了「过半」这个准入条件本身；「较大的号没有任何根、记录、单元带着」（I-7.7 第②句判据的具体内容）未译入，只译了「I-7.7 的第②句限定到同一个集合」这个引用关系本身（连引用编号也去掉了）；「写『全部可见』而只独占一部分时……两个挂载能互相逼成只读」「过半是准入门槛，不是写入范围」这一整段关于「可见」与「写入集合」区别的论证未译入 Fact 11，只保留了「写入这个集合、全或无」这个结论 |
| 定稿理由 | 「过半」准入条件本身对本轮判定是必要的（它是 Judgment 2 权衡的准入检查之一），但它出自哪条决策编号（D2 已定项 13 还是已定项 11）对模型不可核、也不影响判定；「可见 vs 写入集合」的区分论证是这条决策对另一个问题（分叉盘回归、过半规则的代价）的展开，与本轮要判的「写之前有没有二次重算」机制无关，删掉不改变 Judgment 2 的可判性——这条决策原文本身极长（一整段无分号的复合句），保留其余部分已经完整传达了「先判准入、再算一次新代号、写、可能回卷」这个顺序，删掉的是对「为什么过半」「可见与写入的区别」这两件旁支问题的详细论证 |

## Fact 12（观察：该段未提及二次重算）

| 项 | 内容 |
|---|---|
| 原文文件:行 | 无对应中文原句，这是本地攻方腿对 Fact 11 译文本身的复读式核对，不是任何一句的翻译 |
| 首稿缺的 | 不适用 |
| 定稿理由 | 与 m2-checker-supp-code-r1 那一轮 Fact 6 最后一句同类做法：把「某段文字里没有提到某件事」列成一条独立、可核的观测陈述，交给模型自己去核这句观测是否属实，不预先替模型下结论说「因此这是空白」 |

## Fact 13（R7 题面）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `research/prompts/_m2-refusals-presumed-r1-background.md:55` 第二列（「mkfs 两块盘时区域归属写死」） |
| 首稿缺的 | 无遗漏，题面展开为完整的是非问句 |
| 定稿理由 | 保持原题面的范围（恰好两块盘时的归属分配） |

## Fact 14（check_geometry 的控制流 + region_devices 是普通参数）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/make_filesystem.rs:175-179`（`if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES { return Err(...) }`）；`:197`（`make_filesystem` 函数体第一步 `check_geometry(parameters, &sizes)?`，其后才有任何设备写入）；`:54-58`（`MakeFilesystemParameters` 结构体 `region_devices` 字段声明） |
| 首稿缺的 | 「nothing in the filesystem creation code derives this field automatically from the number of devices or from the devices' own identities」这句是本地攻方腿全仓 `grep -n "region_devices:"` 之后的独立观测（命中的赋值点全部在测试代码、`checker` 侧或本函数自己，`make_filesystem.rs` 内部没有任何地方由设备数或设备身份反推 `region_devices` 的值），不是某一句注释的翻译 |
| 定稿理由 | Judgment 3 第二列（零故障可达）需要知道这个字段是不是「调用方随手能填错」的普通参数，还是有别的机制兜底；全仓 grep 的结果直接支撑「普通参数、无自动推导」这个陈述 |

## Fact 15（D2 已定项 7 定案与射程）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/decisions/02-RAID条带策略.md:122`（定案：「归属不再按 (r × P) mod devs 算，mkfs 时逐区域把设备身份写下来。第一版三个区域的归属写死 0 / 1 / 0（区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0），mkfs 逐区域写进系统配置那 12 字节（每区域 4 字节设备身份）；它不是 mkfs 参数。」）；`:129`（依据 E87：「区域编号一定之后 0 / 1 / 0 是唯一的规范写法；做成 mkfs 参数只买到『哪块盘背两个』这一格，而两盘同构下它没有可观测差别。」）；`:130`（「用户定案两次（归属改成存身份、第一版三个区域写死 0 / 1 / 0），后一次没有走三方论证、可推翻」） |
| 首稿缺的 | 「(r × P) mod devs」这个具体公式简化译成「computed by a formula from the device count」，未逐字还原 r、P 两个符号的含义；`:124` 射程段「字段落在系统配置哪一段、区域起点怎么算在 D22（单元原子性怎么合成） 已定项 16；暖机空发布的次数由这个归属唯一确定……与第一个事务的 txg 3 一起是格式常量」「改第一个事务的字节：否」这几句关于字段落点与格式常量的说明未译入 Fact 15 |
| 定稿理由 | 公式的具体符号（r、P）对判定「归属是不是写死」这件事没有增量，只需要知道「以前是公式算的，现在是显式写死的」这个对比；「改第一个事务的字节：否」这句与 Judgment 3 要判的「这个归属选择在盘上字节 / 可达历史上分不分得开」有一定相关性但方向相反（它说的是「这句话本身没有新改字节，只是把已经在写的 0/1/0 补成条款」），为避免与 Fact 2 判据里「分得开」的问法混淆，改用 Fact 15 已经译入的 E87 那句（「两盘同构下没有可观测差别」）来承载这一层意思，两句在「两种选择字节上是否分得开」这一点上结论一致，不重复译入 |

## Fact 16（历史成因 Z3-B + 无其他闸门的观察）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/make_filesystem.rs:83-85`（doc 注释：「第一版两块盘时根环三个区域的归属写死盘 0 / 盘 1 / 盘 0（D2（RAID 条带策略） 已定项 7，2026-09-14 用户定案：不是 mkfs 参数），参数给的不是它：暖机次数与第一个事务的 txg 3 都压在这个归属上（m2-emptypool-nonempty-r1 云端攻方腿 Z3-B：[0, 1, 1] 下暖机要推两次、第一个文件版本与暖机撞在同一个 (实例, txg) 上）。」）；「无其他闸门」是本地攻方腿基于 Fact 14 的 grep 结果得出的独立观察，非翻译 |
| 首稿缺的 | 「[0, 1, 1]」这个具体的错误取值未逐字译入，泛化成「one particular different assignment」；轮次名 m2-emptypool-nonempty-r1 与攻击编号 Z3-B 同 Fact 5 一样隐去 |
| 定稿理由 | 具体错误取值对模型不可独立验证（它没有能力去跑一遍 mkfs 代码验证 [0,1,1] 确实会导致两次撞车），保留「会导致暖机次数变化并与第一个文件版本撞在同一个 (实例, txg) 上」这个因果结构就足够支撑 Judgment 3 对「这条拒绝拒得准不准」的判断；具体数值反而可能诱导模型去做它做不到的算术验证 |

## Fact 17（R8 题面）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `research/prompts/_m2-refusals-presumed-r1-background.md:56` 第二列（「根记录指着的实例表或树表不是 mkfs 写的那一版（诞生 txg 不是 0），而这一版树表 0 条」） |
| 首稿缺的 | 无遗漏，逐句展开成完整问句 |
| 定稿理由 | 「诞生 txg 不是 0」与「树表 0 条」两个限定词都保留 |

## Fact 18（format_time_allocator 的控制流 + 两处 doc 注释）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/mount.rs:480-495`（函数体：484-487 行 `if root.instance_table.head.birth_txg != CheckpointTxg(0) \|\| root.tree_table.head.birth_txg != CheckpointTxg(0) { return Err(...) }` 是函数第一条语句，早于任何分配器构造）；`:476-478`（doc 注释：「这一版的落点记在哪没有条款（D16（发布语义） 已定项 9 只定了树表 0 条时空发布写零个单元）」）；`:70-71`（枚举定义上的 doc 注释：「树表 0 条、而根记录指着的实例表或树表不是 mkfs 写的那一版（指针的诞生 txg 不是 0）：这样一版的分配记录在哪没有条款，这个实现自己写不出这样的根（坏盘或别的写者才有），拒绝挂载。」） |
| 首稿缺的 | `:479`「两样都在动分配器之前返回」这句原本是同时说 `VersionWithoutFileNotWrittenByMakeFilesystem` 与 `FormatTimeUnitLocationsOnDifferentSlots` 两个错误共享的性质，本稿只把它落到前者身上（后者不在本轮四格范围内）；「这个实现自己写不出这样的根（坏盘或别的写者才有）」译成「this implementation itself has no way to ever produce a root that points to such a version」「only a corrupted device, or some other piece of software entirely that also writes to this same on disk format, could ever produce one」，把「坏盘」「别的写者」两个并列的可能原因都展开写全 |
| 定稿理由 | 「两样都在动分配器之前返回」的另一半（`FormatTimeUnitLocationsOnDifferentSlots`）不属于 R8 这一格，混进来会让模型误以为这两个错误是同一条判断的两个分支；「坏盘」「别的写者」两种可能原因是 Judgment 4 第二列判断零故障可达性的关键限定词，逐字展开、不合并 |

## Fact 19（D16 已定项 9 定案与射程）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/decisions/16-发布语义.md:207`（定案：「记账树存在时就写——空发布也是发布，按 D5（快照 / 空间记账机制） 已定项 2『每行每发布重写』重写记账行，连带记账树节点、分配记录、映射条目与树表单元。第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块。」）；`:209`（射程：「定的是空发布写不写单元、写哪几样，不定 c_max 怎么算（D28（挂载期承诺量） 已定项 4 按当时结构现算）、也不定空发布什么时候推（推抬 F 在已定项 1，暖机在已定项 8）。」） |
| 首稿缺的 | 「按 D5（快照 / 空间记账机制） 已定项 2『每行每发布重写』」「按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块」「推抬 F 在已定项 1，暖机在已定项 8」三处的具体条款编号未译入，泛化成「a separately defined, freshly recalculated capacity value」「does not decide when an empty publish is triggered in the first place」 |
| 定稿理由 | 三处编号对模型不可核，且都是「这条决策不定的东西记在哪」这类指路信息，与 Judgment 4 要判的「这条决策定不定 R8 这个具体场景」无关；决策本身的两句话（写不写单元、写哪几样；不定 c_max 怎么算、不定何时触发）逐字保留 |

## Fact 20（观察：射程不覆盖 R8 场景）

| 项 | 内容 |
|---|---|
| 原文文件:行 | 无对应中文原句，这是本地攻方腿对照 Fact 17（诞生 txg 不是 0）与 Fact 19（暖机时树表 0 条）两个条件之后的独立观察 |
| 首稿缺的 | 不适用 |
| 定稿理由 | 与 Fact 12 同类做法：把「决策 A 覆盖的场景与决策 B 描述的场景是不是同一个」这件事列成独立观测，不预先替模型下结论 |

## Judgment 1–4（四格判定指令本身）

| 项 | 内容 |
|---|---|
| 原文文件:行 | 无逐句中文原文对应；四列结构（①有没有写过盘 ②零故障可达 ③名字对不对得上 ④写条款还是登记不支持）译自 `research/prompts/_m2-refusals-presumed-r1-background.md:82`（本地攻方腿分工说明原文：「逐格填表，每格四列：① 这个拒绝之前有没有写过盘（yes / no，给代码行号）；② 被拒的那一族里有没有零故障可达的历史（yes / no / unknown）；③ 错误成员的名字与它实际拒的东西对不对得上（yes / no）；④ 这一格该写条款还是登记成第一版不支持。」） |
| 首稿缺的 | ①③ 两列原文只给 yes/no 二值，本稿在指令段（Part 一、Part 三）保留二值但要求附判据事实号与一句理由，未改变原定义域；④ 列原文只给「写条款」「登记不支持」两个选项，本稿新增第三个选项「已被现有条款覆盖」 |
| 定稿理由 | ①③ 附加理由要求来自共用约束「不许只答 yes/no」，不改变列本身要判的是非；④ 新增「已被现有条款覆盖」选项：R6 那一格背后有 C322 这条已还清的欠账（Fact 10），若不给这个选项，模型被迫在「这是个新空白」与「登记成不支持」之间二选一，而这两个选项都不能表达「其实已经有条款、只是没有覆盖到这个具体机制」这第三种可能——这个新增选项本身也需要模型用 Fact 2 的判据去检验，不是凭空放行 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| Fact 2 结尾："for at least some input" | `research/prompts/_m2-refusals-presumed-r1-background.md:92`（原文「分得开」没有显式量词） | 补了「至少存在一个输入使两种选择分得开」这个量词 | 「分得开」在中文里天然读作存在量词（不要求对所有输入都分得开），补上英文的存在量词只是消歧，不改变判据本身；不补的话英文读者可能读成全称量词，把判据读严了 |
| Fact 3："right after the second of two warm up publishes that occur during the first ever writable mount" | `crates/singlefs-core/src/transaction.rs:1465`、`:1469` | 题面原句「上一版记录」没有点名是哪一次发布之后 | 见上方 Fact 3 那一行的说明；不补的话模型无法确定「上一版」指的是哪个具体节点 |
| Fact 9 整条："through a one to one mapping with no other logic in between" | `crates/singlefs-core/src/mount.rs:1063-1076`，非某一句注释 | 这是本地攻方腿从 `match` 表达式直接观察得到的独立陈述，源码本身没有注释显式声明「这是一对一映射、中间没有别的逻辑」 | Judgment 2 第三列（命名对不对得上）需要知道 mount 层的错误名有没有在转换过程中悄悄改变含义；直接读 `match` 分支能确认字段原样传递、没有额外逻辑，这个观察补上能让模型准确判断「MountError 那个名字与 transaction 层的失败原因是不是同一件事」 |
| Fact 10："found in the closed section of that ledger rather than its open section" | `.claude/kb/checks-owed.md:445`（分节标题）与 `:492`（C322 所在行的相对位置） | C322 那一行的文本本身不包含「我在已还清区」这句话，这是本地攻方腿核对文件结构（`:445` 之后、下一个 `## ` 标题之前）得到的独立结构性观察 | Judgment 2 第四列要判断这个机制「已经被现有条款覆盖」还是「仍是空白」，C322 处于已还清区还是开口表区是关键的结构性事实——处于已还清区意味着项目自己认为这个方向的问题已经有条款兜底，处于开口表则相反 |
| Fact 14 结尾："so a caller is free to pass any array of three device choices it likes when calling filesystem creation with exactly two devices" | 全仓 `grep -n "region_devices:"` 结果，非某一句原文 | 这是本地攻方腿从「`region_devices` 在 `make_filesystem.rs` 内部没有被任何逻辑反推」这一观察归纳出的陈述，源码里没有一句注释显式声明「调用方可以随意填」 | Judgment 3 第二列（零故障可达）的核心判断依据就是「这个字段有没有别的东西替调用方把关」；不补这一句，模型可能误以为某个未提及的地方（例如设备探测逻辑）会自动校正这个字段 |

## 命令与输出（本报告依据的现查，留痕核验）

Fact 14 / Fact 16 依据：
```
$ grep -rn "region_devices:" crates/ --include="*.rs" | grep -v "make_filesystem.rs"
crates/singlefs-checker/src/image.rs:115:    pub region_devices: [u32; 3],
crates/singlefs-checker/src/image.rs:151:        region_devices: [
crates/singlefs-harness/tests/second_transaction_step_zero_test_only_switches.rs:215:        region_devices: system_configuration.immutable.region_devices,
crates/singlefs-core/src/system_configuration.rs:106:    pub region_devices: [DeviceIdentity; 3],
crates/singlefs-core/src/system_configuration.rs:471:                region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
（其余命中均为测试装置里的字面量赋值，逐一核过，全部是调用方直接写死的值，没有一处由设备数或设备身份反推）
```

Fact 4 / Fact 5 / Fact 7 / Fact 8 / Fact 9 / Fact 18 依据：本报告正文（本次派发任务过程中）对
`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mount.rs`、
`crates/singlefs-core/src/make_filesystem.rs` 三份文件的直接 `grep -n` 与 `Read` 现查，
行号见上方各 Fact 行的「原文文件:行」列，均为本轮现查所得，不沿用背景材料里 2026-09-22 上午的旧行号。

## 历史版本

（暂无历史，这是本轮第一次交这份提示。）
