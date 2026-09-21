# 转述核对表：m2-supp3-item3-code-r2-local-attack（2026-09-21）

逐句核对 `research/prompts/m2-supp3-item3-code-r2-local-attack-k4.md`（K4：种子基按周期重抽之后复现还成不成立）与
`research/prompts/m2-supp3-item3-code-r2-local-attack-k5.md`（K5：已知红的判别力挂在哪）里每一句转述与工作区源码/测试自己的原文。
行号全部现查工作区 2026-09-21 版本的源文件（`crates/singlefs-harness/src/crash_injection.rs`、
`crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs`、
`crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`、
`crates/singlefs-harness/src/history.rs`、`crates/singlefs-checker/src/walk.rs`、
`.claude/gate.d/59-crates-mutation-replay.sh`、`crates/mutations.tsv`），不是 kb 条款，不走 quote-kb.py。
格式：英文项 / 原文文件:行 / 首稿缺的或改动的 / 定稿理由。

## 事实核查先于翻译核查：一处行号分歧

派发提示第三节写「`crates/mutations.tsv` 第 191、192 行钉的是『换成别的数就红』」；本轮现查工作区 `crates/mutations.tsv`
（`awk -F'\t' 'NR>=185 && NR<=192{print NR": "$1}' crates/mutations.tsv`）：**第 191 行今天钉的是「用户 2026-09-20
定案第 3 条的配套口径」（记录核对器不认「被流里更晚的写盖过」，是 K3 的攻击面，不是种子基），换基之后就红的两行今天实际是
第 189 行（`crash_injection.rs` 的 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`）与第 192 行（`second_transaction_supplement_three_random_history.rs`
的 `FAST_TIER_FIRST_SEED`）。** 本核对表与两份提示按现查结果用 189、192（提示正文里不写行号，按「本地腿的提示按格数拆」
一节的规则只用 Row Alpha / Row Beta 这两个标签指），核对表这里留痕；这处分歧写进交回报告，行号是否在这一轮之内发生过位移
（比如派发之后又有改动）不归本子 agent 判断。

## K4（种子基按周期重抽之后，复现还成不成立）

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿 |
|---|---|---|---|
| Fact 1: 开一个新周期要做三样（重抽、删镜像重建、整表在新基上重验） | `crash_injection.rs:61-64`（「**开一个新周期要做三样**：1. 照上面那条命令重抽一次，把下面这个数换掉；2. 旧的数据盘与虚拟盘镜像全删掉、全部重建（上一个周期的盘面是在旧种子基上攒出来的）；3. `crates/mutations.tsv` 整表在新种子基上重验一遍——点名随机历史与崩溃注入那些行的判红，都是在旧基上量的。」） | 无遗漏，三条逐字对应译出，未合并、未跳过任何一条的限定语（「上一个周期的盘面是在旧种子基上攒出来的」「都是在旧基上量的」两处理由从句都译了） | 按字面译，三句对应三个 first/second/third |
| Fact 2: 周期之内写死不动、种子基一动就成了掷骰子 | `crash_injection.rs:66-67`（「周期之内写死不动：`crates/mutations.tsv` 的『这条变异必须红』与里程碑那几条验收说的都是**这一批**历史上的判红，种子基一动它们就成了掷骰子，判别力本身没了。」） | 无遗漏；「这一批」的着重号译成「this specific batch」保留强调；「判别力本身没了」译成「the discriminating power…is no longer established」，语气从「没了」改写成「尚未确立」——核对时认为这一步收窄了原文（原文更强，是「已经没了」，英文读起来像「还没建立」），已在下方「多出来的限定词」一节单列 | 按现有译法留用，但在下方限定词表里标出这处语气收窄，供主 agent 判断要不要更强的措辞 |
| Fact 3: `the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle` 整个测试体 | `second_transaction_supplement_three_crash_injection.rs:74-79`（`assert_eq!` 比对写死字面量 `7_463_871_032_432_355_113`，失败信息「种子基不是这个测试周期 2026-09-20 抽的那个数：周期之内不许换，换了这个周期量过的判红全部作废；下一个周期才重抽」）；后续 `run_crash_injection_campaign` 与两条 `assert_eq!`/`assert!`（约 :80-100，报告 `first_seed` 与重放串包含种子基） | 无遗漏；测试体分两段译（先断言常量、再断言报告与重放串），对应源码里两组独立断言，未合并成一句丢失细节 | 按字面译，保留「先比对字面量、再验证报告/重放串带回同一个值」两步结构 |
| Fact 4: `the_five_sampling_tiers_start_from_the_test_cycle_seed_base`，五档常量都直接赋值同一个种子基 | `second_transaction_supplement_three_random_history.rs:62-78`（循环比对五档、失败信息「…的种子基是 {first_seed}，不是这个测试周期的种子基：这一段跑的是另一批历史，变异表与验收在它上面量过的判红都不算数」）+ 五处声明 `:32,37,42,47,52`（`const ... : u64 = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;`，五档逐个现查全部同一种赋值，非各自独立字面量） | 首稿写成「只有最快档」是这样赋值，与现查的五处声明矛盾（五档全部如此）；核对时用 `grep -n` 现查五处声明发现首稿这句与源码不符，已改 | 改成「every one of these five constants…none of the five has its own independent literal value」，五档一视同仁 |
| Fact 5: 门禁脚本与全仓对「按周期删镜像重建」「seed_base/test_cycle 字样」两次检索零命中 | 本子 agent 自己现跑的检索，不是某句中文的译文：`grep -rniE "seed_base|seed base|test_cycle|测试周期" .claude/gate.d/*.sh` 零命中；`grep -rniE "rebuild.*image|delete.*mirror|delete.*image|全部重建|旧镜像全删|重建镜像|delete_all.*image|wipe.*image" crates/ .claude/gate.d/` 只命中与「单次跑完删自己临时镜像」（`FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes`，`crash_injection.rs:180` 等）相关的行，没有一行是「开新周期删旧镜像重建」 | 不适用（非译文，是本子 agent 现跑的检索结果） | 检索命令与结果原样写进 Fact 5，供主 agent 复核 |
| Fact 6: 门禁 59 号自己的说明（读表里的六段、原样替换、跑点名测试、不认种子基） | `.claude/gate.d/59-crates-mutation-replay.sh:4-9`（「判据：`crates/mutations.tsv` 里每一条变异（文件、原文、替换文、cargo test 参数、必须红的测试名），原文在文件里恰好命中一次；把仓拷到临时目录、改坏那一处、跑点名的测试，那条测试必须判红；跑完还原再下一条。一条锚点腐化，一条没红，整道红。」）+ 脚本正文对种子基零引用（同 Fact 5 检索） | 无遗漏；「一条锚点腐化，一条没红，整道红」没有直接译，因为这句讲的是脚本自己的失败模式（原文命中次数不对/测试没红都会整道门禁判红），与 K4 要判的「认不认种子基」无关，译进去会引入不相关的材料——遗漏是有意的，属于「不抄」范畴 | 保留「不识别种子基」这一半，跳过「锚点腐化」那一半（与本文档的判断无关） |
| Fact 7: mutation 行 Alpha / Beta（种子基改成别的字面量） | `crates/mutations.tsv:189`（崩溃注入二进制：`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE` 从 `7_463_871_032_432_355_113` 改成 `12345`，必须红 `the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle`）与 `:192`（随机历史二进制：`FAST_TIER_FIRST_SEED` 从 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE` 改成字面量 `0`，必须红 `the_five_sampling_tiers_start_from_the_test_cycle_seed_base`） | 行号按上面「事实核查先于翻译核查」一节改用现查的 189、192（不是派发提示写的 191、192）；数值译法：`12345` 译成英文单词「twelve thousand three hundred forty five」而非阿拉伯数字，避免提示里出现的数字被误认成「行号」——这是本地腿提示的通用避坑写法，不是原文要求 | 内容按字面译，行号改用现查值，数值写成英文单词 |

## K5（已知红的判别力挂在哪）

（下一段续写字段、函数体与三条变异行的核对表）

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿 |
|---|---|---|---|
| Fact 1: `AllocationStatisticMechanism` 八个字段 | `history.rs:1241-1262`（结构体注释「I-3.1 判红时 checker 在说明文字里带的机理标识…遍历覆盖了哪些根槽、剩下的按什么理由没覆盖。『已知红』清单按它分辨机理，不按签名」+ 八个字段各自的行内注释：`根环一圈的槽数 R × S`、`环里最新那条根的 txg`、`环里自证过的根槽个数`、`环里最老的那条自证过的根的 txg`、`遍历真走过的候选根槽个数`、`被最新根的实例表判成「被抛弃的时间线」而没走的根槽个数`、`最新根带的回退下界 F`、`低于 F 而没走的根槽个数`） | 「被抛弃的时间线」这个限定词在英文里简化成了「judged them abandoned」，没有译出「时间线」；核对时确认这处简化与格式化字符串本身（`walk.rs:1517` 只写「被实例表判抛弃的根槽」，同样没提「时间线」）一致，不是本稿单独丢的字 | 保留简化译法，「时间线」的省略与源码自己在对外文字里的省略一致，列在下方限定词表里存证，不改 |
| Fact 2: `leading_number_after` 整个函数体 | `history.rs:1230,1231-1239`（注释「`label` 之后紧跟着的那串十进制数字；`label` 不在、或它后面不是数字，都是 None。」，函数体 `text.split_once(label)?.1.chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()`） | 无遗漏；额外补一句「does not know where in the overall string the label occurs, and does not check the label only occurs once」——这是从函数体本身（`split_once` 只找第一处、不检查唯一性）现读出的结构性事实，不是注释逐字，原文注释没有这句 | 保留，作为从函数体直接观察到的推论单列在下方限定词表 |
| Fact 3: `allocation_statistic_mechanism` 整个函数体、全或无 | `history.rs:1279,1281-1291`（注释「从 I-3.1 的说明文字里读机理标识；少一项就 None（说明这条违例不是那一段格式写出来的）。」，函数体八次 `leading_number_after(detail, "标签")?` 逐字段构造 `AllocationStatisticMechanism`） | 无遗漏；「the entire function returns nothing at all, not a partially filled structure」是从 `?` 操作符在 `Some(Struct { .. })` 字面量里逐字段短路这一 Rust 语义读出的结构性事实，源码注释「少一项就 None」已经蕴含这一点，本稿只是把「就 None」具体化成「不是部分填充的结构体」，未引入新事实 | 保留具体化的措辞 |
| Fact 4: 两个布尔方法读哪些字段 | `history.rs:1265-1266,1268-1271`（形态零：「环转过一圈把 F 之上的根挤出了环：环里最老的自证过的根已经高过 F」，方法体 `self.newest_root_txg >= self.root_ring_slot_count && self.oldest_readable_root_txg > self.rollback_floor`）与 `:1273,1275-1277`（形态一：「回退下界把环里读得到的根挡在了遍历之外」，方法体 `self.root_slots_dropped_below_floor >= 1`） | 无遗漏；「reads two…for its first condition and two…for its second condition，four fields in total，one of which is the rollback floor field」是本稿对方法体逐字段计数写出来的，源码注释本身没有这样计数，但每个字段确实是方法体里字面出现的那几个，不是推断新事实 | 保留逐字段计数的写法，帮模型不用自己再数一遍 |
| Fact 5: checker 侧注释 + 格式串原样顺序 | `walk.rs:1503-1504`（「I-3.1 判红时，说明文字里带一段机理标识：遍历覆盖了哪些根、剩下的按什么理由没覆盖。判读的一方（`singlefs-harness` 的 `history::allocation_statistic_mechanism`）按它分辨『记账多算』是哪一种机理造成的…两者的签名…一模一样，不带这一段就只能按签名认」）+ `:1517`（格式串「；机理：根环槽数 {}、最新根 txg {}、环里自证过的根槽 {} 个、最老的自证过的根 txg {}、遍历的候选根槽 {} 个、被实例表判抛弃的根槽 {} 个、回退下界 F {}、低于 F 的根槽 {} 个」） | 无遗漏，八个标签按格式串原样顺序逐个列出，未重排、未省略任何一个 | 按字面译，顺序与源码格式串完全一致 |
| Fact 6: mutation 行 Gamma（`crates/mutations.tsv:185`） | 原文（原文段）：`observation.root_ring_has_turned() && only_allocated_statistic_above_walked(observation) && allocation_statistic_mechanism_of(observation).is_some_and(|mechanism| mechanism.the_ring_turn_dropped_roots_above_the_floor())`；替换文（改坏后）：`observation.root_ring_has_turned() && only_allocated_statistic_above_walked(observation)`；必须红：`history::tests::known_red_forms_are_matched_by_mechanism` 的 `known_red_forms_are_matched_by_mechanism_not_by_signature` | 无遗漏；提示里把「改坏后的样子」当作这一行要描述的效果来写（「after this row's change…」），而不是把「改坏前」当效果——这是故意的顺序选择：mutations.tsv 的「原文」是良性代码、「替换文」才是变异，测试必须在替换文上判红，提示里描述的正是替换文生效之后的行为，与 tsv 语义一致 | 按替换文（变异后）描述效果，原文（良性代码）作为「这行删掉的那句」来源 |
| Fact 7: mutation 行 Delta（`crates/mutations.tsv:186`） | 原文：`&& allocation_statistic_mechanism_of(observation).is_some_and(|mechanism| mechanism.the_floor_dropped_readable_roots())`（前缀 `observation.raised_floor_lands_only_on_abandoned_roots == Some(true) && only_allocated_statistic_above_walked(observation)` 不变）；替换文：删掉这一整段 `&&` 子句；必须红同 Fact 6 | 无遗漏；「same named test as fact 6」现查 tsv 第 186 行必须红字段与第 185 行完全相同（同一个测试函数覆盖两条已知红形态），已核对两行必须红字段逐字相同 | 按字面译，并现查确认两行测试名相同 |
| Fact 8: mutation 行 Epsilon（`crates/mutations.tsv:187`） | 原文：`、回退下界 F {newest_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"`；替换文：`、低于 F 的根槽 {root_slots_dropped_below_floor} 个"`；必须红：`second_transaction_supplement_three_random_history` 的 `turning_the_root_ring_with_overwrites_in_one_mount_ends_in_the_first_known_red_form` | 无遗漏；「immediately follows whatever clause used to precede the removed one」是从原文/替换文两段字面比对读出的结构性事实（删掉的那一节前面是「被实例表判抛弃的根槽」那一节，删掉之后「低于 F 的根槽」直接接上前一节），源码本身没有这句旁白，但完全由给定的原文/替换文可以推出，不引入新事实 | 保留，帮模型不用自己再比对原文/替换文两段字符串 |

## 两份共用：本轮问题框架（不是源码原文）

问题 1–5（K4）与问题 1–5（K5）都是本轮新写的任务说明，不对应源码某一句注释或断言消息，是本地攻方腿自己按派发提示第三节
K4/K5 两段攻击面与「一份提示不超过 16 格」的约束拆出来的任务框架；每个问题引用的事实编号都指向上面已核过的 Fact 1–7（K4）
或 Fact 1–8（K5），未引入未核过的新事实。K5 问题 2–4 里新增的「Row Zeta」（假设性的第四行：只改一个标签的措辞、七个不变）
不对应 `crates/mutations.tsv` 里任何一行，是本子 agent 为了单独测「重命名一个标签」这个场景另写的假设项，在提示第 29 行里
已经明写「is not in the mutation table but is a hypothetical for you to reason about」，不冒充源码事实。

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| K4 Fact 2: "the discriminating power…is no longer established for the new batch" | `crash_injection.rs:67`（「判别力本身没了」） | 原文是断言式的「没了」（已经消失），英文改写成「尚未确立」，语气比原文弱一档 | 翻译时的措辞选择，非有意加限定词，已在上表该行标出、留给主 agent 判断要不要改回更强的「no longer holds」 |
| K4 Fact 4: "none of the five has its own independent literal value anywhere in the source" | `second_transaction_supplement_three_random_history.rs:32,37,42,47,52`（五处声明本身，原文没有这句总括） | 五处声明各自只是一行赋值语句，原文（用户 2026-09-20 定案第 7 条的引文）没有「五个都不是独立字面量」这句总括 | 这是本子 agent 现查五处声明之后加的总括句，直接支撑 Fact 4 的核心论点（五档全部共用同一个符号，不是各自独立抽样），删掉这句模型就要自己现读五处声明才能得出同样的结论；已在 Fact 4 本体交代「from direct observation of the source」，不冒充源码逐字 |
| K4 Fact 7: 数字写成英文单词而非阿拉伯数字 | `crates/mutations.tsv:189,192`（原文是 `12345`、`0` 两个阿拉伯数字） | 把 `12345` 写成「twelve thousand three hundred forty five」 | 避免提示正文出现阿拉伯数字串被误当成行号引用（提示要求「不写文件名与行号」，写成单词从形式上更难被误用成行号） |
| K5 Fact 2: "does not know where in the overall string the label occurs, and does not check the label only occurs once" | `history.rs:1231-1239`（函数体本身，注释 `history.rs:1230` 没有这句） | 原文注释只说「label 不在、或它后面不是数字，都是 None」，没有明写「不检查唯一性」「不知道标签在哪」 | 这是从 `split_once` 的语义（只找第一处、返回切分后的后半段）直接读出的结构性事实，是问题 2/3/4 判断「新插入一个字段会不会撞到已有标签」这一支路时必需的背景，不给这条模型无法判断插入新字段是否安全；已在 Fact 2 本体交代「from direct observation of the source」 |
| K5 Fact 3: "not a partially filled structure" | `history.rs:1279`（注释「少一项就 None」） | 注释只说「就 None」，没有明写「不是部分填充的结构体」这个对照说法 | 这是 `?` 操作符在 `Some(Struct { field: expr? , .. })` 字面量里的 Rust 语义蕎结果（一个字段短路，整个 `Some(..)` 都不构造），源码注释已经蕴含这一点，加这句是把语言语义的后果说明白，不引入新事实 |
| K5 Fact 4: 逐字段计数（「four fields in total, one of which is the rollback floor field」「exactly one of the eight fields, and that one field is not the rollback floor field itself」） | `history.rs:1269-1270,1276`（方法体本身） | 源码注释是自然语言描述（「环转过一圈把 F 之上的根挤出了环」），没有逐字段计数 | 直接读方法体数出来的字段清单，帮模型不用自己重新数一遍就能回答问题 3（哪个字段被哪个方法用到），减少模型自己解析 Rust 表达式出错的机会；已在 Fact 4 本体交代「from direct observation of the source」 |
| K5 Fact 8: "immediately follows whatever clause used to precede the removed one" | `crates/mutations.tsv:187`（原文/替换文两段字符串本身） | tsv 行本身只给两段字符串，没有旁白说明删除位置的前后文关系 | 由给定的原文/替换文两段字符串直接可比对得出（删除的是「回退下界 F …、」这一节，前一节「被实例表判抛弃的根槽 … 个、」不变，后一节「低于 F 的根槽 … 个」原样保留、现在直接接上前一节），不引入原文没有的新事实，只是替模型把这次字符串比对做完 |

## 历史版本

（暂无历史，这是本轮第一次交这两份提示）
