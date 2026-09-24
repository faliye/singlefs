# 转述核对表：m2-checker-supp-code-r1-local-attack（2026-09-21 UTC / 2026-09-22 JST）

逐条核对 `research/prompts/m2-checker-supp-code-r1-local-attack.md`（K4 全面 + K3 里
「实现与判据原文逐字对不对得上」那几格）里每一条 Fact 与中文/源码原文。行号现查工作区版本
（`git status` 显示这些文件都是未提交的工作区改动，不是 HEAD 版本）。
格式：英文项（Fact 编号）/ 原文文件:行 / 首稿缺的或改动的 / 定稿理由。

提示本身不引用任何文件名或行号（按共用约束「英文提示里的每一句转述」一节与主 agent 派发提示
「提示里明令答复不写代码行号与文件行号」），只在这份核对表里写死来源文件与行号。

## Fact 1（早退条件改前改后，K4 事实 1）

| 项 | 内容 |
|---|---|
| 原文文件:行 | 改前条件 `research/prompts/_m2-checker-supp-code-r1-diff.md:580`（`-    if chosen.is_empty() || chosen.len() != system_configurations.len() {`）；改后条件 `crates/singlefs-checker/src/walk.rs:1801`（`if chosen.is_empty() {`）；正文陈述 `research/prompts/_m2-checker-supp-code-r1-body.md:34`（「`walk.rs` 原本『任一块盘择不出系统配置 ⇒ 全部不变量报不适用并 return』；同一天 `recovery.rs` 改成『全部盘都择不出才失败』。两者不一致时，一块盘系统配置全废的镜像恢复挂得上、checker 一条都不判。改成『全部盘都择不出』才早退。」） |
| 首稿缺的/改动的 | 无中文原句可逐字核对，这是本稿综合改前代码、改后代码与正文陈述三处得到的一条陈述，不是某一句的翻译；已核对三处来源在改前条件、改后条件、因果关系三点上互相一致 |
| 定稿理由 | 保留「同一天」（same day）与因果链（两者不一致 ⇒ 挂得上但一条都不判）；这条陈述是 Judgment 1、Judgment 2 的背景铺垫，不直接被判定 |

## Fact 2（早退条件改动的理由注释，K4 事实 1）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-checker/src/walk.rs:1796-1800`（「一块盘的两个系统配置槽都无效时**不早退**：恢复会改用别的盘上那一份、照常挂上（`crates/singlefs-core/src/recovery.rs` 的 `choose_system_configuration`，D22（单元原子性怎么合成） 已定项 8「系统配置每盘放一份」买的就是这份冗余）。早退会让这种镜像上每一条不变量都报「不适用」，而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过（.claude/kb/checks-owed.md 的 C461）。⚠️ 「哪块盘的系统配置全废了」今天没有编号报得出来，仍欠在 C461。」） |
| 首稿缺的/改动的 | 删掉「D22 已定项 8『系统配置每盘放一份』买的就是这份冗余」这一句归因（为什么恢复能 fallback 的设计理由）；C461 的编号本身也隐去，只译成「tracked separately as still owed」 |
| 定稿理由 | Judgment 1、Judgment 2 都不需要知道「为什么」有这份冗余，只需要知道「有」；不点 C461 编号是延续上一轮 K3/K6 核对表的做法（模型不需要也不应该核到一个它验证不了的编号） |

## Fact 3（recovery.rs 的两种失败方式，K4 事实 2）

| 项 | 内容 |
|---|---|
| 原文文件:行 | 文档注释 `crates/singlefs-core/src/recovery.rs:230-231`（「池里每块盘的两个槽都无效（`RecoveryFailure::NoValidSystemConfiguration`）；交得出系统配置的几块盘之间 fsid 或设备数对不上、或者池里一块盘都没有（`RecoveryFailure::SystemConfigurationsDisagree`）。」）；枚举定义 `crates/singlefs-core/src/recovery.rs:118-127`（`NoValidSystemConfiguration { first_device_with_no_valid_system_configuration_slot: DeviceIdentity }` 带一个盘身份字段；`SystemConfigurationsDisagree` 不带字段） |
| 首稿缺的/改动的 | 英文比原文多出「which returns one named failure kind carrying the identity of the first such device found」「carrying no device identity」两句——230-231 行本身没有写「带不带字段」这件事 |
| 定稿理由 | 见下方「多出来的限定词」一节；这两句是从枚举定义（118-127 行）直接现查得到的独立观测，不是 230-231 行那句话的翻译，Judgment 2 需要能区分这两种失败方式各自带不带盘身份，才能回答两个候选镜像各自会撞到哪一种 |

## Fact 4（choose_system_configuration 的控制流，K4 事实 2）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/recovery.rs:238-263`（逐盘循环，两槽都解不出时 `continue` 并只记第一块这样的盘）；`:285-293`（循环结束后 `chosen` 是 `Some` 就直接 `return Ok`，不管其余盘多少块两槽全废；只有循环全程一块有效盘都没找到才返回 `NoValidSystemConfiguration`） |
| 首稿缺的/改动的 | 无中文原句可核对，直接观察源码控制流；已逐行确认：`chosen` 变量只在第一次找到有效盘时赋值一次（`None => chosen = Some(best_on_device)`），之后的迭代只做「有效性对比」不再覆盖 `chosen` |
| 定稿理由 | 按代码原样转述，不是翻译；这是 Judgment 1、Judgment 2 都要用到的关键机制事实 |

## Fact 5（choose_system_configuration 的分歧检查，K4 事实 2）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-core/src/recovery.rs:273-283`（`if previous.immutable.filesystem_identifier != best_on_device.immutable.filesystem_identifier \|\| previous.immutable.device_count != best_on_device.immutable.device_count { return Err(RecoveryFailure::SystemConfigurationsDisagree); }`） |
| 首稿缺的/改动的 | 无中文原句可核对，直接观察源码；已确认这一步在循环内部触发，一旦命中就立即返回，不等循环跑完 |
| 定稿理由 | Judgment 2 的候选镜像 A（fsid 不同）与候选镜像 B（device_count 不同）都要靠这一段代码判断 `choose_system_configuration` 会不会失败 |

## Fact 6（I-1.4 循环只看 chosen 列表、不比 device_count，K4 事实 3）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-checker/src/walk.rs:1810-1822`（`let geometry = chosen[0].2;` 起、`for (device, view, _) in &chosen { root_ring_judgements.judge("I-1.4", view.filesystem_identifier == geometry.filesystem_identifier, ...) }`，注释「各盘择到的系统配置要属于同一个池：fsid 逐盘相同」在 1811 行）；`device_count` 字段全仓引用 `grep -n "device_count" crates/singlefs-checker/src/*.rs crates/singlefs-core/src/recovery.rs` 只命中 `lib.rs:122`（结构体字段）、`lib.rs:182`（解析赋值）、`recovery.rs:278`（分歧检查）、`recovery.rs:1196/1211`（挂载期别处用途），walk.rs 全文件零命中 |
| 首稿缺的/改动的 | 无中文原句可核对，直接观察源码 + grep 结果；「Nothing in this loop, and nothing anywhere else the checker judges, ever reads or compares a device count field」这一句是全仓 grep 之后的独立观测，不是某一句注释的翻译 |
| 定稿理由 | Judgment 2 候选镜像 B（device_count 不同）依赖这一条事实：checker 从头到尾没有任何一处比较 device_count，这是判断「checker 会不会把候选镜像 B 判成正常」的关键证据 |

## Fact 7（I-7.7 的按盘取号与「归 I-1.4 判」的托词，K4 事实 1 + K3 附带）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-checker/src/walk.rs:665-686`（`judge_instance_carriers` 函数，`highest_by_device` 逐盘取「本池 fsid 相同的自证槽里最大的 journal_instance」；`.collect::<Option<Vec<u32>>>()` 一旦有一块盘是 `None` 就整体 `None`，触发 686 行 `not_applicable`）；托词原文 `crates/singlefs-checker/src/walk.rs:685`（「有一块盘没有本池 fsid 的系统配置槽：那块盘归不归这个池由 I-1.4 判」） |
| 首稿缺的/改动的 | 685 行原文直译是「那块盘归不归这个池由 I-1.4 判」，本稿译成「whether that device belongs to this pool at all is left for the filesystem identifier check named in fact 6 to decide」——用「fact 6」代指 I-1.4，不点出 I-1.4 这个编号 |
| 定稿理由 | 按共用约束「答复不写代码行号与文件行号」的精神延伸：编号本身不需要给模型，模型只需要知道「Fact 6 描述的那个检查」与「Fact 7 自己说要靠 Fact 6 判」之间是否真的对得上（Judgment 1 第二问的核心） |

## Fact 8（坏镜像测试的机制与断言，K4 事实 3）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-harness/tests/system_configuration_per_device_redundancy.rs:273-322`（函数 `the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration`；276 行 `let pool = build_pool(...)`；284-290 行只对 `DeviceIdentity(0)` 两个槽翻位；297-301 行 `assert_eq!(after.len(), clean.len(), ...)`；302-306 行 `assert!(after_not_applicable < after.len(), ...)`；307-317 行算 `newly_not_applicable`；318-321 行 `assert!(newly_not_applicable.len() <= 1, "坏掉盘 0 的系统配置最多让一条从判得了变成判不了，实际 {newly_not_applicable:?}（干净镜像上不适用 {clean_not_applicable} 条）")`） |
| 首稿缺的/改动的 | 原文断言消息点名「盘 0」（`DeviceIdentity(0)`），本稿泛化成「exactly one of those devices」，不点具体盘号；269-271 行的说明注释（早退条件为什么改）未重复译入 Fact 8，已在 Fact 1/2 译过 |
| 定稿理由 | 泛化盘号：这条测试选盘 0 只是具体取样，断言的结构性主张（坏一块盘的系统配置最多影响一条不变量）不依赖是哪一块盘，Judgment 1 不需要盘号本身；避免与 Fact 1/2 重复翻译同一句说明 |

## Fact 9（I-7.3 判据原文，K3 事实 4）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/invariants.md:52`（「**环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当『更早代号的记录』，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到」） |
| 首稿缺的/改动的 | 删掉「⚠️ 射程：种子没被覆盖完之前……才会被这一条抓到」这整句（关于「连续两代落同一槽」这另一类 bug 的检测延迟说明）；删掉「(mkfs 把第 0 代种进全部区域，D22 已定项 8)」这个归因括注，只保留「a fresh filesystem itself seeds into every region at creation time」；「更早代号」译成「a strictly earlier generation number」，补了「strictly」 |
| 定稿理由 | 射程说明是另一件事（另一类轮换 bug 的检测延迟），与 Judgment 3 要判的「例外从句抄没抄对」无关，删掉不改变 Judgment 3 的可判性；D22 已定项 8 的归因编号对模型不可核，隐去；「更早」在整数代号语境下等价于「严格更早」，补 strictly 不改变原意，只是消歧 |

## Fact 10（I-7.3 的实现，K3 事实 4）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-checker/src/walk.rs:1653-1657`（文档注释，「判据只读 S 里的代号，不读这个态是怎么来的」在 1656 行）；`:1658-1677`（`judge_root_ring_health` 函数体：`newest_checkpoint_txg`=max，`earlier_generation_exists`=any 严格小于，`every_root_is_the_genesis_generation`=all 等于 0，`judge("I-7.3", earlier_generation_exists \|\| every_root_is_the_genesis_generation, ...)`） |
| 首稿缺的/改动的 | 无中文原句可核对，直接观察源码与其自带注释；已确认代码字面就是「earlier_generation_exists 为真 或者 every_root_is_the_genesis_generation 为真」才判绿，与 Fact 9 的「不存在即判红，除非例外」逐句对应 |
| 定稿理由 | Judgment 3 的判定对象就是这一段代码与 Fact 9 的逐字对应关系 |

## Fact 11（I-8.6 判据原文，K3 事实 4）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/invariants.md:81`（「任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 307 字节头，`header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录反向链恒 0**（D23（journal 的角色与格式） 已定项 10 / 已定项 19，2026-09-13 用户定案，链按实例算与头宽 307 是 2026-09-14 用户定案）。不等的记录不进重放前缀。⚠️ 链按实例算，所以跨实例边界的两条记录之间**不判**这一条——实例边界由实例代号挡着（I-8.3（重放前缀严格连续)）」） |
| 首稿缺的/改动的 | 删掉「(D23 已定项 10/已定项 19，2026-09-13 用户定案……)」整段归因括注；删掉「不等的记录不进重放前缀」这一句（这是「判红之后怎么用」的下游后果，不是判据定义本身）；「实例边界由实例代号挡着（I-8.3）」译成「the instance boundary itself is already guarded by a separate invariant that checks instance numbers directly」，不点 I-8.3 编号 |
| 定稿理由 | 归因括注与「不进重放前缀」这句对 Judgment 4（case ③ 算不算跨边界判定）不改变可判性，删掉不影响判定；I-8.3 编号同 Fact 7 的做法，不点给模型 |

## Fact 12（I-8.6 的实现，K3 事实 4）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-checker/src/walk.rs:1732-1739`（文档注释，三种情形穷举 ①②③ 在 1735-1738 行，「跨实例边界的两条记录之间不比链值——I-8.6 明写不判那一格」在 1738 行）；`:1741-1772`（`judge_journal_back_chain` 函数体，`expected_back_chain` 的 `match`：`*counter == FIRST_JOURNAL_COUNTER => Some(0)`；`Some(previous) if previous.instance == record.instance => Some(previous.chain_value_the_next_record_must_carry)`；`Some(_record_of_another_instance) => Some(0)`；`None => None`） |
| 首稿缺的/改动的 | 无中文原句可核对，直接观察源码与其自带注释；已确认第三支（`Some(_record_of_another_instance) => Some(0)`）确实存在且被 `judgements.judge` 消费（不是被跳过） |
| 定稿理由 | Judgment 4 判的正是这一支：它是「本实例第一条恒 0」的应用，还是「跨实例边界判定」本身，本稿只如实转述代码分支与注释原文，把判断留给模型 |

## Fact 13（I-1.8 判据原文，K3 事实 4）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/invariants.md:31`（「码 1 / 3 的可读已发布单元按『类身份段全部字段含写序』归并成组之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条）」） |
| 首稿缺的/改动的 | 删掉「码 1 的键是写序」这半句归属（码 1 的全序键今天悬而未决，是背景材料已经点名、K3 里不归本地腿攻的那一半，欠账 C464）；删掉「诞生代号打头」的成因括注（「同一实例连着两个 checkpoint……逐字节相同」）与「事务号留在末位」的成因括注（「固定点会迭代……」）；删掉「码 2 不进这条」；「同一组的成员载荷相同」保留为独立小句，用「Separately, and independently of the ordering requirement」明确标出它与归并键定义是两件事 |
| 定稿理由 | 「码 1 的键是写序」是背景材料 C464 已经点名的另一个缺口，不属于这一份提示要攻的面（K3 里只攻「实现与判据原文逐字对不对得上」，且主 agent 已在正文里把码 1 全序键定性为「只落了一半、欠账 C464」，不需要本地腿重新发现同一件事，混进 Judgment 5 会让模型把两个不同的缺口糅成一个答案）；两处成因括注对「归并键定义本身」的逐字核对没有增量；「码 2 不进这条」在 Fact 13 里已经隐含在「两个内容承载类」这个措辞里，不需要显式重复 |

## Fact 14（I-1.8 归并键的实现，K3 事实 4）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `crates/singlefs-checker/src/walk.rs:1335`（「归并键的两段：『类身份段全部字段含写序』**去掉载荷校验和那一段**。」）；`:1441`（「『类身份段全部字段含写序』去掉载荷校验和那一段：同一份内容的两个副本在这上面逐字节相同。」）；`:1550`（函数级文档注释复述同一句定义）；结构体字段 `merge_key_before_payload_checksum` / `merge_key_after_payload_checksum` 在 `:1337-1338`（字段声明）、`:1359-1360`（码 1 取值 `42..101` / `105..105`）、`:1369-1370`（码 3 取值 `42..89` / `93..107`），均跳过各自的载荷校验和/CRC 字段偏移 |
| 首稿缺的/改动的 | 无遗漏；「载荷校验和不进归并键——它正是『同一组的成员载荷相同』要比的那个量，进了键这一条就恒真」（对应完整版注释，diff 里在 1345-1346 行区间）按字面译成「the payload related field is exactly the quantity that the separate identical payload check in fact 13 is supposed to compare... making the identical payload check trivially always true」 |
| 定稿理由 | 这是 Judgment 5 的判定对象本身，按字面全译，不摘句 |

## Fact 15（D18 已定项 7 码 1 类身份段字段表，K3 事实 4）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/decisions/18-块里携带什么信息.md:168`（表头「**类身份段——数据单元（+63 字节 ⇒ 共 105 字节初值，占 32 KiB 单元 0.32%）**：」）；`:171-177`（表格：五元组 33、诞生代号 8、fsid 8、写序 10、载荷 CRC 4 共 5 行；载荷 CRC 一行原文「| **载荷 CRC** | **4** | C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3：副本归并要求『同类身份段的多份可读单元载荷相同』，码 1 没有别的字段能证明这一点 ⇒ ……载荷 CRC 是码 1 副本归并 / 级 1 恢复的前置；只加在码 1 自己的类身份段，不进共同前缀 |」） |
| 首稿缺的/改动的 | 表头「+63 字节 ⇒ 共 105 字节初值」「占 32 KiB 单元 0.32%」两个具体数字未译入 Fact 15（对 Judgment 5 不必要，Judgment 5 只需要知道「载荷 CRC 是这张表列出的 5 个字段之一」这个结构性事实，不需要总宽度或占比）；「不进共同前缀」未译（这是与另一段「共同前缀」字段表的边界声明，与「载荷 CRC 是否属于类身份段」这件事本身无关，`.claude/kb/decisions/18-块里携带什么信息.md` 共同前缀表在 170 行之前另一张表里） |
| 定稿理由 | 只保留「这张表列出恰好 5 个字段，载荷 CRC 是其中之一，且列在最后一行，用途是让两个副本能比对载荷是否相同」这一层，字面对应原文第 177 行的出处列 |

## Fact 16（D18 已定项 11 码 3 类身份段字段表，K3 事实 4）

| 项 | 内容 |
|---|---|
| 原文文件:行 | `.claude/kb/decisions/18-块里携带什么信息.md:271`（表头「**码 3 的类身份段（65 字节，头 107）**：」）；`:274-286`（表格 12 行：单元类型标签 42、出生树 ID 43、打包记录类型 51、容器号 53、容器出生代 61、记录数 69、记录宽 71、诞生代号 73、fsid 81、载荷校验和 89、写序 93、出生序号 103；载荷校验和一行原文「| 89 | 载荷校验和 | 4 | CRC32C，覆盖偏移 107 到单元末尾（nonce / MAC 预留位 + 记录区 + 补齐）。整单元校验和 / MAC 住父指针（D4（校验和位置）），扫描认领时没有父指针，头校验和只是头完整性防线（E76（载荷校验和的判别力）：头落了载荷半落，只有头校验和判不出）。宽度挂 D23（journal 的角色与格式） 已定项 13 |」） |
| 首稿缺的/改动的 | 首稿曾把「出处」列的用途直译成「to let corruption of the packed content be detected before it is claimed as a parent for anything else」，与原文「整单元校验和 / MAC 住父指针……扫描认领时没有父指针，头校验和只是头完整性防线」的实际因果关系（父指针的整单元校验和这时还不存在，头校验和只护得住头部，需要靠这个字段护住载荷）没有对上；核对时改写成「since the whole unit's own authoritative checksum lives in a parent pointer that does not exist yet during scan based reconstruction, and the header checksum by itself only defends the header, this field is what lets corruption specifically inside the packed payload area be detected during that reconstruction」 |
| 定稿理由 | 改写后的说法与原文「出处」列逐句对应（父指针整单元校验和的位置、扫描认领时它还不存在、头校验和只护头部三个从句都保留），首稿那版把因果关系简化丢了，属于「缺一个限定词」的情形，已在这里改正 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| Fact 3："which returns one named failure kind carrying the identity of the first such device found"、"carrying no device identity" | `crates/singlefs-core/src/recovery.rs:118-127`（枚举定义，非 230-231 行那句注释本身） | 230-231 行的文档注释本身只说两种失败方式各自的触发条件，没有说「带不带盘身份」这件事 | Judgment 2 要求模型分别判断候选镜像 A（fsid 不同）与候选镜像 B（device_count 不同）各自会撞上 `choose_system_configuration` 的哪一种失败；两种失败方式带不带盘身份是从枚举定义现查到的独立事实，补上能让模型准确对应 Fact 3 与 Fact 5 之间的关系，不补的话模型分不清两种失败方式是不是同一回事 |
| Fact 6 整条最后一句："Nothing in this loop, and nothing anywhere else the checker judges, ever reads or compares a device count field" | 全仓 grep 结果（见下「命令与输出」），非某一句中文注释 | 这是本稿从「device_count 全仓引用」这条 grep 归纳出的陈述，源码里没有一句注释显式声明「checker 从不比较 device_count」 | Judgment 2 候选镜像 B 的核心判断依据就是「checker 有没有检查 device_count」，这件事必须由一条独立、可核的观测陈述给出，不能只靠 Fact 1（早退条件）反推——Fact 1 的早退条件不涉及 device_count，若不补这一句，模型可能误以为 checker 在別处某个未提及的地方仍会比较 device_count |
| Fact 9："strictly earlier generation number" | `.claude/kb/invariants.md:52`（原文只写「更早代号」） | 补了 strictly | 整数代号语境下「更早」等价于「严格更早」，补 strictly 只是消除「更早或相等」这一读法的歧义，不改变原意 |
| Judgment 1 第二问（"does fact 6 show that the check fact 7's own comment says it is deferring to never actually runs for the device with no valid configuration"） | `crates/singlefs-checker/src/walk.rs:685` + `:1810-1822` 两处对照，非某一句原文的翻译 | 这是本稿从 Fact 6（I-1.4 循环只看 `chosen` 列表）与 Fact 7（685 行「归 I-1.4 判」）两处对照推出的问题，原文任何一句单独都没有提出这个问题 | 主 agent 派发提示要求判「它在这种镜像上判不了是不是对的」；仅凭 Fact 7 自己的措辞无法回答「对不对」，必须把 Fact 6 摆在一起才能看出「归 I-1.4 判」这句话有没有兑现——这正是本地攻方腿要攻的那一格，不是抄来的结论，是留给模型自己核的问题 |
| Judgment 2 整条（两个候选镜像的构造） | 无原文对应，正文 K4 只写「另外核『全部盘都择不出』这个新条件在什么镜像上会与恢复再次分叉」，未给出具体镜像 | 候选镜像 A（fsid 不同）与候选镜像 B（device_count 不同）是本地攻方腿按 Fact 3/5/6 自己构造的两个候选，不是主 agent 或背景材料给出的现成答案 | 主 agent 只给了问题（在什么镜像上会分叉），没有给候选镜像；按「本地腿只问能落成数、能逐格判的题」的要求，必须把问题落成具体的、逐格可判的候选，而不是让模型自己去想象一个镜像——两个候选各自对应 Fact 5 分歧检查的两个字段（fsid、device_count），覆盖该检查的全部触发条件 |

## 命令与输出（本报告依据的现查，非某句原文的翻译，留痕核验）

Fact 6 依据：
```
$ grep -n "device_count" crates/singlefs-checker/src/*.rs crates/singlefs-core/src/recovery.rs
crates/singlefs-core/src/recovery.rs:278:                    || previous.immutable.device_count != best_on_device.immutable.device_count
crates/singlefs-core/src/recovery.rs:1196:    let device_count = device_identities.len();
crates/singlefs-core/src/recovery.rs:1211:    if roots.accounting.entries.len() != 3 + 6 * device_count {
```
（`crates/singlefs-checker/src/*.rs` 部分零命中：`walk.rs`、`image.rs`、`lib.rs` 均未列出，说明 `device_count` 字段在 checker 侧完全没有被引用；`lib.rs:122`、`:182` 两处此前另一条命令核过，是结构体字段声明与解析赋值，不构成「比较」）

Fact 3 附加依据：
```
$ sed -n '118,127p' crates/singlefs-core/src/recovery.rs
    NoValidSystemConfiguration {
        first_device_with_no_valid_system_configuration_slot: DeviceIdentity,
    },
    /// 各盘的系统配置 fsid 或设备数对不上。
    SystemConfigurationsDisagree,
```

Fact 1 附加依据：
```
$ grep -n "chosen.is_empty() || chosen.len()\|if chosen.is_empty() {" research/prompts/_m2-checker-supp-code-r1-diff.md
580:-    if chosen.is_empty() || chosen.len() != system_configurations.len() {
586:+    if chosen.is_empty() {
```

## 历史版本

（暂无历史，这是本轮第一次交这份提示。）
