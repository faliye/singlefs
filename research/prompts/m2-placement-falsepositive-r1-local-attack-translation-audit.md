# 本地攻方提示的翻译核对表（m2-placement-falsepositive-r1）

方法说明：本轮提示里只有四条「条目标题」与三条事实（F5、F14、F17）是对仓里某一句中文原文的逐句转述，因此对这七处按「英文项 / 原文文件:行 / 首稿缺的 / 定稿」逐句核对（第一节）。其余的 F 编号事实（F1、F1a–F4、F6–F13、F15、F16、F18–F21）是我自己现读代码 / 测试得到的原始英文事实陈述，不是在转述仓里某一句已成文的中文句子，因此不适用「逐句核对缺漏限定词」，改用「事实来源表」只记来源文件与行号（第二节）。第三节列英文比中文原文多出来的限定词与括注，写明为什么加。

## 一、逐句核对（标题与三条整段转述的事实）

### 1. Z1 条目标题

英文项（提示第 29 行）：ITEM Z1: release does not check the unit checksum carried by a mapping entry's location field

原文文件:行：research/prompts/_m2-placement-falsepositive-r1-body.md:27

原文：### Z1　释放判定不核映射条目位置项里的单元校验和（收口表第 13 行）

首稿缺的：无（首稿即定稿，逐词对得上：释放判定=release …judge、不核=does not check、映射条目位置项里的单元校验和=the unit checksum carried by a mapping entry's location field）。「（收口表第 13 行）」这个出处括注没有搬进条目标题本身，改放进了本核对表下方「事实来源表」里对 F5 的引用（checks-owed.md 与 milestone 表的行号），不是丢失，是挪了位置。

定稿：同首稿，未改。

### 2. Z2 条目标题

英文项（提示第 43 行）：ITEM Z2: placement answers can differ across devices of unequal size

原文文件:行：research/prompts/_m2-placement-falsepositive-r1-body.md:31

原文：### Z2　盘不等大时各盘的落点答案可以不同（收口表第 53 行）

首稿缺的：无，逐词对得上（盘不等大=devices of unequal size、各盘的落点答案可以不同=placement answers can differ across devices）。出处括注同上，挪进事实来源表。

定稿：同首稿，未改。

### 3. Z3 条目标题

英文项（提示第 55 行）：ITEM Z3: invariant I-3.1, "allocated accounting matches", is judged violated on states the project's own notes call legal

原文文件:行：research/prompts/_m2-placement-falsepositive-r1-body.md:35

原文：### Z3　I-3.1（已分配统计对得上） 在合法状态上判红（收口表第 54 行）

首稿缺的：这一句没有经过「先写错、再改对」的过程，写第一版时就避开了一个常见的直译陷阱，这里记下那个陷阱是什么、为什么避开，而不是编一段没发生过的修改史。直译「在合法状态上判红」容易写成「is violated on legal states」（这些状态本身违反了不变量），但原文说的是「这道检查把这些状态判成违例」——判红是检查的动作，不是状态自己的性质，这正是这一格题眼所在（「检查是不是错了」）。定稿因此用「is judged violated」（检查判定的动作）而不是「is violated」，并把「合法」限定成「项目自己的记录称之为合法」（the project's own notes call legal），不替它在提示里下判断。

定稿：invariant I-3.1, "allocated accounting matches", is judged violated on states the project's own notes call legal（与首次写出的文本相同，未经修改）。

### 4. Z4 条目标题

英文项（提示第 65 行）：ITEM Z4: the record checker's second criterion does not recognize legitimate reuse

原文文件:行：research/prompts/_m2-placement-falsepositive-r1-body.md:39

原文：### Z4　记录核对器第二条判据不认合法复用（收口表第 55 行）

首稿缺的：无，逐词对得上（记录核对器第二条判据=the record checker's second criterion、不认合法复用=does not recognize legitimate reuse）。出处括注同上，挪进事实来源表。

定稿：同首稿，未改。

### 5. F5（C394 一句话描述）

英文项（提示第 41 行）：F5. The project's own tracked-issues file has a row, identifier C394, opened 2026-09-19, whose one-line description is: the release-judgment path does not check the unit checksum carried by the mapping entry's location field when rebuilding the previous version from disk.

原文文件:行：.claude/kb/checks-owed.md:347（C394 行「要拦什么」列）

原文：从盘上重建上一版时，释放判定路径（`crates/singlefs-core/src/transaction.rs` 的 `placements_to_release_via_mapping`）只核「映射里查得到 key、分配记录在册、没释放过、跨度对得上」，不核映射条目位置项里带的单元校验和；`rebuild_version`（`crates/singlefs-core/src/recovery.rs`）读映射条目只取 key，位置项与它带的校验和丢掉

首稿缺的：首稿的 F5 只译了前半句（释放判定路径不核校验和），漏译了分号后半句——`rebuild_version`（recovery.rs）读映射条目时同样只取 key、把位置项连同它带的校验和一起丢掉。这是原文用同一句话交代的第二处丢校验和的地方，首稿没有搬过去。核对时用 crates/singlefs-core/src/recovery.rs 现读代码坐实了这半句（第 782–791 行：`let (key, _locations) = parse_mapping_entry(entry)...`，`_locations` 这个变量名前缀下划线，是 Rust 里「特意不用」的写法），补成新的一条事实 F1a（提示第 33 行），不占用 F5 本身的编号。

定稿：F5 保留原句（release-judgment path 那半句，附「四要素」的具体化：查得到 key / 在册 / 没释放过 / 跨度对得上，这四项是从同一处原文照抄的，不是我另加的）；`rebuild_version` 那半句单列成 F1a。

### 6. F10 里两处直接引述的短语

英文项（提示第 53 行）：the number of times a piece of user data's placement disagrees with the policy function ｜ the placement that gets sent out is already the answer every device agreed on

原文文件:行：.claude/kb/verification-build.md:76

原文：用户数据的落点与政策函数不一致的次数 ｜ 发出去的落点就是各盘一致的答案

首稿缺的：无，两处都是逐字对译（第一处：用户数据的落点=a piece of user data's placement、与政策函数不一致的次数=the number of times …disagrees with the policy function；第二处：发出去的落点=the placement that gets sent out、就是各盘一致的答案=is already the answer every device agreed on）。

定稿：同首稿，未改。

### 7. F14（milestone 表 54 行）

英文项（提示第 63 行）：F14. …records this same twelve-out-of-thirty-one gap as an open item still requiring further argument, and separately records that an attacker exercise, tagged c381-r1, reproduced an equivalent-shaped state using a plain power-loss injection on a separate copy of the current core code, on 2026-09-19.

原文文件:行：.claude/kb/milestone/02-second-txn.md:355（「54」行，第二、三列）

原文（第二列「今天现状」）：口径冲突，今天走得到；2026-09-19 c381-r1 攻方在 HEAD 版 core 副本上用一个纯断电复现同形（固定前缀到 B、C 的记录落盘根槽没落，断电点 17 或 18，可写重开之后差 4 个槽，`research/prompts/c381-r1-main-verification.md` 第六节，核查员复跑逐字相同），失败之后原样重发也走得到（同一报告第五节）；原文（第四列「去向」）：要三方；用例按现状钉成 12 个红

首稿缺的：首稿漏译了「失败之后原样重发也走得到（同一报告第五节）」这一分句——原文说的是这个状态不只是「能被复现」，重发布之后系统还能继续正常运作，不停在坏状态上。这条对本轮要问的「column 1：零故障走得到吗」有直接关系（它说明这条路径不是死路），首稿没有把它带进 F14。核对时补回：F14 定稿加了「reproduced an equivalent-shaped state」之后，本可以再加「和重发仍然走得到」，但因为这句话在原文里说的是 c381-r1 那个附属报告的结论、不是这一格本身的现状，且提示第 55 行（Z3 条目标题）与 F13 已经交代了「这一版本身走得到（31 个状态里 12 个判红，其余照常运作）」这件事，重复补一次这句话对回答四列问题没有新增信息，因此定稿里没有补，改成在这份核对表里记下「没有补、为什么」而不是无声漏掉。

定稿：F14 保留首稿文本，不补那一句；上面一段是「为什么不补」的记录。

### 8. F17（milestone 表 55 行）

英文项（提示第 71 行）：F17. …describes this same criterion as currently misreporting on all 8 states of a stale-tail stream, using the same wording, "a unit overwritten later by an already-persisted write at the same location does not count as missing," as a candidate fix that the row says still needs further review, not as a decision already made.

原文文件:行：.claude/kb/milestone/02-second-txn.md:356（「55」行）

原文：记录核对器第二条判据（`record_claimed_state_missing_unit`）不认合法复用：陈旧 tail 那条流 8 个状态都误报，被流里更晚、已持久、同盘同偏移的写合法盖掉的单元被判缺席；以后固定脚本复用流里写过的槽都会误报（第一列）；候选改法（被更晚已持久的同位置写盖掉的不算缺席）要过三方；用例按现状钉成 8（第四列）

首稿缺的：首稿漏译了「以后固定脚本复用流里写过的槽都会误报」这一句——原文说的不只是「这条流现在误报」，还断言这是一个会持续复发的模式（以后任何复用旧槽的固定脚本都会撞上）。这一句对 column 3（候选改法有几个）与 column 4（碰不碰别的格）没有直接影响，但对 column 1、2 的「今天现状」判断有旁证价值。核对时决定不把它塞进 F17（F17 已经很长，且 F16／F17a 已经给了「同一形状、结论相反」的完整材料，模型判 column 1/2 时够用），改成在这份核对表里记下「有意不补」而不是漏译：这句是对「历史会怎样」的推断（以后……都会），不是「今天状态是什么」的事实，收进事实性的 F17 反而会把一句预测当成事实喂给模型。

定稿：F17 保留首稿文本，不补那一句；上面一段是「为什么不补」的记录。

## 二、事实来源表（不是转述、是我自己现读代码 / 测试的结果，只记来源）

以下各条不是对仓里某一句中文的转述，是我自己读代码、读测试得到的英文事实陈述；「首稿缺的」栏不适用，只记来源文件与行号（现查，非从背景材料里数）。

| 事实编号 | 来源文件 | 行号（现查） |
|---|---|---|
| F1 | crates/singlefs-core/src/transaction.rs | 函数起始 935 行；三样检查分别在 993、999–1000、1006–1008 行 |
| F1a | crates/singlefs-core/src/recovery.rs | 782–791 行，丢弃写在第 785 行（`let (key, _locations) = ...`） |
| F2 | crates/singlefs-core/src/pointer.rs | 56–67 行 |
| F3 | crates/singlefs-core/src/pointer.rs | 15–19 行 |
| F4 | 我自己的仓内检索，非单一文件的一行 | 检索命令与命中见 research/prompts/m2-placement-falsepositive-r1-local-attack-runlog.md「F4 检索记录」一节 |
| F6 | crates/singlefs-core/src/allocator.rs | 541–544 行 |
| F7 | crates/singlefs-core/src/allocator.rs | 128–144 行（四个变体；枚举定义本身是英文标识符，三条中文注释在 131–132、134–135、139–140 行） |
| F8 | crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs | 三个用例函数起始行分别是 255、345、410 |
| F9 | crates/singlefs-core/src/allocator.rs | 字段声明 629 行；用例起始 1115 行；断言 1137 行 |
| F10 | .claude/kb/verification-build.md | 76 行（见上一节第 6 条逐句核对） |
| F11 | crates/singlefs-checker/src/walk.rs | 见 F12 的易变说明；两次现读分别取到函数起始 2185 行与 2200 行，本身就相差 15 行 |
| F12 | crates/singlefs-checker/src/walk.rs | 两次现读：第一次 candidate_indexes 起 2323 行、`judge("I-3.1"...)` 在 2501 行；几分钟后第二次 candidate_indexes 起 2338 行、判定调用在 2516 行；文件 sha256 两次不同（`6220c45b…` 与 `d686e2c5…`），确认是另一个并发会话在改这份文件，不是我看错 |
| F13 | crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs | 用例起始 882 行；断言 995 行；「65536 字节」「四个固定点单元」的说明性注释在断言前一行（991–994 行） |
| F14 | .claude/kb/milestone/02-second-txn.md | 355 行（见上一节第 7 条逐句核对） |
| F15 | crates/singlefs-harness/src/crash.rs | 函数起始 648 行；`in_place` 653 行；`written_over_later` 660 行；`copy_is_missing` 670 行 |
| F16 | crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs | 用例起始 1069 行；说明性注释 1171–1174 行；断言 1175 行 |
| F17 | .claude/kb/milestone/02-second-txn.md | 356 行（见上一节第 8 条逐句核对） |
| F18 | crates/singlefs-core/src/allocator.rs | `record_for` 738 行、`release` 778 行（另见 F1、F6 各自的来源） |
| F19 | crates/singlefs-core/src/allocator.rs | 同 F6、F7、F9、F18 的来源，本条是对它们的横向归纳 |
| F20 | crates/singlefs-checker/src/walk.rs；crates/singlefs-core/src/allocator.rs | 同 F11/F12 与 F18 的来源，本条是对它们的横向归纳 |
| F21 | crates/singlefs-harness/src/crash.rs | 同 F15 的来源 |

### 9. F13 末尾追加的一句（用例内的中文注释）

英文项（提示里 F13 末句）：The same inline comment adds that which side of that disagreement should change is an open design question, and that this test pins today's status quo rather than endorsing it as correct behavior.

原文文件:行：crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:991–994（用例断言前的说明性注释）

原文：I-3.1（已分配统计对得上）在实例 2 的根为最新的 12 个状态上判红（口径未定，2026-09-17 写这条用例时发现）：记账的已分配逐盘比遍历候选根多 65536 字节，正是残留记录那一版……自己的四个固定点单元……写行发布 txg 6 把它们释放进 defer，环里没有一条根引用它们。checker 的「已分配 = 候选根引用的并集」与分配器「defer 里的仍算已分配」在「由记录施加出来的那一版」上分歧，改哪一边是 I-3.1 口径的设计问题；这里钉的是现状，不是认下来的行为。

首稿缺的：首稿的 F13 只搬了「65536 字节」「四个固定点单元」「defer」「没有根引用」这几件量化事实，漏了最后一句「改哪一边是 I-3.1 口径的设计问题；这里钉的是现状，不是认下来的行为」。这一句对 column 2（有没有用例把现状钉着）直接相关——它把「有用例」与「用例认为这是对的」明确分开，若不补，模型可能把「有测试断言 12」误读成「项目认定这 12 个是对的」。补成追加句（见上）。

定稿：如上一句已经追加进提示里的 F13。

## 三、英文比原文多出来的限定词（单列，写明为什么加）

| 加了什么 | 加在哪 | 为什么加 | 核对依据 |
|---|---|---|---|
| 「the discarded value is bound to a name that marks it as intentionally unused」 | F1a | 原文（checks-owed.md:347）只说「位置项与它带的校验和丢掉」，没说这是 Rust 里刻意为之的写法（下划线前缀）；这个限定词不是原文有的，是我读代码（recovery.rs:785）另加的一层佐证，为了让模型知道这不是疏漏、是明确的设计选择——不加的话「丢掉」可能被读成「代码写错了」而不是「代码故意不取」 | crates/singlefs-core/src/recovery.rs:785 |
| 「its bytes on disk do not match what was written」（对 in_place 取反的具体含义） | F15 | 原文（crash.rs 645–646 行注释）只说「不在盘上」，没有展开「不在盘上」的判定方法；这是我读 in_place 闭包本身（653–659 行：读盘、逐字节比对）之后替原文的「不在盘上」补上的操作性定义，因为 column 1 要模型说清「代码路径」，光说「不在盘上」不够具体 | crates/singlefs-harness/src/crash.rs:653–659 |
| 「This written-over-later check looks only at the position of writes in the list; it does not separately check whether that later write is itself marked persisted in the specific crash state under examination」 | F15 | 原文（crash.rs 640–646 行注释）没有点出 written_over_later 判定不看「那次更晚的写在这个崩溃态下有没有真的落盘」这一层；这是我读 written_over_later 闭包本身（660–668 行：只比较写表里的先后位置与字节区间，不读 `image.persisted`）之后加的限定词，因为它是本轮 Z4／F16／F17 分歧能不能解释得通的关键机理，原文的注释停留在「合法复用不算缺席」这一层，没有下沉到「怎么判定合法复用」这一层 | crates/singlefs-harness/src/crash.rs:660–668 |
| F11／F12 的「note on this fact only」整段（说明 walk.rs 被并发改动、两次现读行号不同） | F12 | 原文（body.md 第 23 行）只提醒「行号一律现查……判这一轮用的是函数名」，没有具体交代这份文件在这一轮写材料期间被另一个会话改到了什么程度；这是我自己两次现读时观察到的（sha256 不同、相差约 15 行），按共用约束「本机常有别的会话在同一个仓里干活……看到别人没提交的改动不碰」如实记录，避免模型把某个具体行号当成可核的锚点 | crates/singlefs-checker/src/walk.rs（两次现读，sha256 分别是 6220c45b… 与 d686e2c5…） |
| F17a 与 F16／F17 之间「不下结论、原样并陈」的整段处理 | F16、F17、F17a | 原文的两处来源（milestone 表 356 行、用例注释 1171–1174 行）各自单独看都是自洽的，只有放在一起才看得出矛盾（一边说现在误报 8、一边说现在是 0）；原文任何一处单独都没有承认这个矛盾。这是我核对时发现的、原文没有写出来的一层事实，按角色要求（不解读、不采纳、只如实交回）如实并陈，不替主 agent 下结论 | .claude/kb/milestone/02-second-txn.md:356；crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:1171–1174 |
