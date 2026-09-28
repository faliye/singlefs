# m2-closeout-code-r3 本地攻方腿 提示翻译核对表

栏位：英文项 / 原文文件:行 / 首稿缺的 / 定稿

来源快照：`refs/sop/m2-closeout-code-r3-snapshot`（`crates/`、`litmus/`），取到 `/tmp/claude-1000/m2-closeout-code-r3-local-attack/snapshot/`。下表「原文文件」除标了 `research/prompts/` 前缀的两份实现员报告外，均指快照里的路径，行号是快照里那份文件自己的行号（现查，非从背景材料数）。

## 背景定义段（FACT SET 2 之前的公共定义）

1. 英文项：「a "two-state closed form"...是 1 + the sum over every segment of (2^(segment length) - 1)；a "three-state closed form"... in-place overwrite 的段用 3^n-1」
   原文文件:行：`crates/singlefs-checker-tier/src/crash.rs` 里 `closed_form_state_count` 与 `layer0_state_count_with_torn_in_place_overwrites` 两个函数各自的语义（本轮没有另引这两个函数的文档注释原文，改引用同一份测试文件里把两种闭式摆在一起对比的注释，见下条）
   首稿缺的：没有单独列出这两个函数各自的原文文档注释（那两条在 r2 已经引过一次，见 `research/prompts/m2-closeout-code-r2-local-attack-translation-audit.md` 第 3 条，本轮不重引，直接沿用同一条已经核过的定义，避免与 r2 重复取样同一处原文）
   定稿：定义段末尾改引 `crash_enumeration_multi_record_publish_stream.rs:391`（见下一条），不再单独抄两个函数各自的文档注释

2. 英文项：「the closed form report the enumeration domain's closed form (in-place overwrite takes three states): six segments that are only system-configuration slot writes of 2 writes each contribute 3^2 - 1, three record segments of 2 writes each contribute 3, five 1-write segments, the unit segments of A/B/C each contribute 2^n - 1, the 4-write segment 15, the 6-write segment 63.」
   原文文件:行：`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:391`「枚举域的闭式：六个只有系统配置槽写的 2 写段各 3² − 1、三个记录 2 写段各 3、五个 1 写段、A / B / C 的单元段各 2^n − 1、4 写段 15、6 写段 63」，框架句「计数行的 closed_form 报枚举域的闭式（原地覆写三态）」取自同文件 :374（两条相邻但不是同一行，合并成一句）
   首稿缺的：「六个只有系统配置槽写的 2 写段」里的「只有」（only）首稿英译写成「are system-configuration-slot segments」，没有字面译出「只有…写」这个限定——判定：这个限定词是在强调这六段除了系统配置槽写、不含任何别的写，但本题的任务只要这六段的「态数」（3）与「个数」（6），「段内只有一种写」这条限定不改变本题要算的任何数字，故首稿的意译不影响算术，仍在这里记一笔缺口
   定稿：不改份提示原文（因为不影响本题算术），在本表里记「只有」这个限定词首稿没有字面译出

## FACT SET 1（第一条流 11 段，两态 / 三态闭式）

3. 英文项：1a 段序描述「acquiring the instance number, two writes | ... | the system-configuration slot, two writes」（完整见提示文件第 12 行）
   原文文件:行：`crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_a_formatted_pool.rs:64-65`「取号两写 | 写行发布的记录两写 | 根 | 系统配置两写 | 暖机的记录两写 | 根 | 系统配置两写（C577：发布末尾那道屏障之后自成一段）| 第一个文件版本的二十四个单元写（十二个单元各两盘：七个角色加分配记录树五个节点，D8（核心索引结构） 已定项 14）| 记录两写 | 根 | 系统配置两写」
   首稿缺的：「二十四个单元写」后面的括注「（十二个单元各两盘：七个角色加分配记录树五个节点，D8（核心索引结构） 已定项 14）」没有译入——判定：这一段是解释「为什么是 24 个单元写」的背景说明，不改变本题要用的「长度 24、kind=unit」这两个事实，本题只需要长度与 kind，不需要 24 这个数字本身的来历
   定稿：不补，理由记于此；C577 那个括注（发布末尾那道屏障之后自成一段）在原文只挂在第三次出现的「系统配置两写」后面，首稿英译同样只把 C577 括注挂在第三次出现处（紧接在单元写之前），逐一核对顺序无误

4. 英文项：1b 数组「[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]」与消息「after C577 the second warm-up publish's...」
   原文文件:行：`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:466`（数组）、:467（消息「C577 之后暖机第二次的系统配置槽轮换过了发布末尾那道屏障，不再与 A 的 24 个单元写同段」）
   首稿缺的：无，逐句照译
   定稿：同首稿

5. 英文项：1c「FULL_STATES_WITH_TWO_STATES_PER_WRITE = 16,777,240 ...; FULL_STATES = 16,777,260 ...」
   原文文件:行：同文件 :442（常量 `16_777_240`）、:446（常量 `16_777_260`），说明句取自 :441、:445 的注释（「1 + (3+3+1+3+3+1+3+3+1+3) + (2^24 − 1) = 16777240」「1 + 4 × 8 + 3 × 3 + 3 × 1 + (2²⁴ − 1) = 16777260」）
   首稿缺的：这两句是我自己写的说明（"the two-state closed form of the 1b array" 等），不是原文逐句译文，是从 :441、:445 的算式反推出常量的角色再用自己的话点出——按核对规则算「多出来的」，多出来的理由：本题需要模型知道这两个常量各自对应哪一种闭式，原文本身没有用一句英文说清「这是哪种闭式」，是从算式结构反推的
   定稿：保留这两句说明，作为「多出来的」记在此条，不改成逐字翻译（逐字翻译只有符号算式，模型看不出哪个是两态哪个是三态）

## FACT SET 2（第二条流 19 段）

6. 英文项：2a 数组「[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2]」
   原文文件:行：`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:185`
   首稿缺的：无
   定稿：同首稿

7. 英文项：2b「the closed form when every write takes two states: 1 + Sigma(2^|segment| - 1), nineteen segments」+ 算式 + 「Here "9*3" means...」
   原文文件:行：注释 :368「每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，十九段」；算式 :369-372；常量 :373（`1_358_954_604`）
   首稿缺的：「Here "9*3" means nine of the array's segments have length 2...」这几句是我自己加的说明，原文这一条注释本身没有把「9*3」拆成「九段长度 2」逐字讲清楚（这个拆法的文字依据在 2c 引的三态断言注释里，两条断言分开写、拆法只在三态那条里点名）
   定稿：保留说明句，记为「多出来的」，理由：本题要模型按段填表，需要知道「9*3」对应哪些段，原文把这个对应关系分写在另一条断言的注释里（本表第 8 条），这里合并引用是为了给模型一份自足的事实，不是凭空编的数字（数字 9、3、5 等全部来自 2b、2c 原文算式本身）

8. 英文项：2c「1 + 6 * (3 * 3 - 1) + 3 * 3 + 5 + ... 」+「Here six of the nine length-2 segments are system-configuration-slot segments...」
   原文文件:行：注释 :374（框架句）与断言消息 :391（「枚举域的闭式：六个只有系统配置槽写的 2 写段各 3² − 1、三个记录 2 写段各 3、五个 1 写段、A / B / C 的单元段各 2^n − 1、4 写段 15、6 写段 63」）；算式 :383-390；常量 :393（`1_358_954_634`）
   首稿缺的：同背景定义段第 2 条记的「只有」限定词，此处重复出现，不再重记；另外原文断言消息里「4 写段 15、6 写段 63」是把「2^4−1=15」「2^6−1=63」直接写成结果数字、没有写中间的「2^4−1」这一步，首稿英译保留了原文这种「直接给结果」的写法（"the 4-write segment 15"），与原文忠实度一致，不算缺口
   定稿：同首稿

9. 英文项：FACT SET 2 的 TASK 段「State the numeric difference between the three-state total and the two-state total...equals 6 * (8 - 3) = 5」
   原文文件:行：无对应原文单句——这是我给模型出的推导题，不是原文的量，用意是让模型自己对齐两态与三态之间只有「6 个系统配置段各多 5」这一处差别
   首稿缺的：此项本身就是「多出来的」（任务设计，不是转述）
   定稿：按「多出来的」记录于此，不在正文另加原文出处

## FACT SET 3（到 C 脚本，27 段 → 32 段）

10. 英文项：3a「32 segments, the two-state closed form 50,724,921」
    原文文件:行：`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:593`（注释「32 段、每次写只取两态时闭式 50724921、甲二 69」）、:602-603（断言 `segments.len()==32`、`closed_form_state_count==50_724_921`）
    首稿缺的：注释里「甲二 69」（quick-tier 快档闭式 69）没有译入——判定：本题只用两态全量闭式，甲二那一档不是本题任务要算的量，不译入不影响本题
    定稿：不补，理由记于此

11. 英文项：3b「before C577, 27 segments, 202,113,066」+「...(26, 26, 26, 18, 18); five places each get split into 2 and n」+「together 9 * 2^24 + 6 * 2^16 - 15 = 151,388,145」
    原文文件:行：同文件 :595-596「C577 之前 27 段、202113066、84、454426691、159：A、B、C 的 24 个单元写与写行之后两次暖机的 16 个单元写各与上一次的轮换同段（26、26、26、18、18），五处各拆成 2 与 n——两态闭式每处少 3 · 2^n − 3（共 9 · 2^24 + 6 · 2^16 − 15 = 151388145）」
    首稿缺的：① 「84、454426691、159」三个数字（甲二快档 84、三态全量 454426691、三态甲二 159）没有译入，本题只用两态全量 202113066，其余三个不是本题要算的量；② 「两态闭式每处少 3 · 2^n − 3」这句公式原文有明写，首稿没有把它当「给定的引文」抄进事实块，而是放进 TASK 段让模型自己推导同一条公式——判定：这不是漏译，是任务设计选择（让模型自己证明这条公式，而不是照抄答案），公式本身的数字结论（151388145）仍原样引在事实块里
    定稿：①不补三个未用数字，理由记于此；②TASK 段保留「模型自己推导」的设计，不改成直接抄公式

## FACT SET 4（取号屏障流，28 段末两段）

12. 英文项：4a 28 段数组 + 「the segment numbers shift two places later」
    原文文件:行：`crates/singlefs-checker-tier/tests/crash_enumeration_acquisition_barrier.rs:229-230`（数组）、:224-226（注释「与第二条流发 C 之前那一截逐段相同...C577 之后每次轮换过了发布末尾那道屏障、自成一段：A、B 的 24 个单元写与暖机两次的 16 个单元写不再并上一次的轮换（改之前 26、26、18、18），段号跟着后移两位」）
    首稿缺的：「与第二条流发 C 之前那一截逐段相同」这句（这 28 段与 fixed_script_stream 到 C 之前那一截逐段相同）没有译入——判定：这是两份文件之间的交叉引用说明，不是本题要算的算术输入，不译入不影响本题
    定稿：不补，理由记于此

13. 英文项：4b「the acquisition segment has only the two system-configuration slot writes of acquiring the instance number...」
    原文文件:行：:191-193「取号那一段只有取号两次系统配置槽写（2 写，原地覆写各取三态，3² − 1 个；B 的两次轮换在它前面、隔着取号写之前那道屏障，代码审阅第 19 条），写行那次发布的单元写一段（18 个单元写，各取两态，2^18 − 1 个），再加全部持久那一个」
    首稿缺的：「B 的两次轮换在它前面、隔着取号写之前那道屏障，代码审阅第 19 条」这半句没有译入——判定：这是解释「取号段为什么单独成段」的背景（B 的两次轮换在它之前、被取号写之前那道屏障隔开），不是本题要算的数字，不译入不影响 [2,18] 这两个长度与三态/两态归类
    定稿：不补，理由记于此

14. 英文项：4c「the two-state closed form of the [2, 18] pair is stated as "1 + 3 + 262,143"; the three-state closed form...is stated as "1 + (3 * 3 - 1) + 262,143"」
    原文文件:行：:204（`1 + 3 + 262_143`，消息「每次写只取两态时的闭式（补第三态之前的数）」）、:223（`1 + (3 * 3 - 1) + 262_143`）
    首稿缺的：无
    定稿：同首稿

## FACT SET 5（σ 段，(16,16) 与 65536）

15. 英文项：5a「sigma: 16 unit writes (after C577...)」+「(16, 16): the first 16 is the sigma segment's own length...the second 16 is...how many are unit writes」
    原文文件:行：`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:328-329`（断言 `(sigma_segment.len(), 过滤出的单元写计数) == (16, 16)`，消息「σ：16 个单元写（C577 之后上一次发布的系统配置槽轮换过了发布末尾那道屏障、在 σ 前一段；改之前 2 个原地写 + 16 个单元写）」）
    首稿缺的：初稿这里把 (16,16) 误写成「the first 16 is the count of segments in a group being asserted」——这是错的：读 :320-328 代码，比较的元组是 `(sigma_segment.len(), 该段里单元写的计数)`，不是「一组段的个数」。这是本轮核对时发现的一处理解错误，不是「少一个限定词」，是转述错了断言比的是什么
    定稿：已用 `research/scripts/replace-once.py` 改成「the first 16 is the sigma segment's own length (how many writes it holds); the second 16 is, of those writes, how many are unit writes」，与 :320-327 的代码逐字对过（`sigma_segment.len()` 与 `.filter(kind==UnitWrite).count()`），改稿已回读确认写入提示文件

16. 英文项：5b「enumerated.states, 65,536」+「sigma has 16 writes (before C577, 18, and 262,144)」
    原文文件:行：:732（`enumerated.states, 65_536,`）、:733（「σ 有 16 个写（C577 之前 18 个、262144）」）
    首稿缺的：无
    定稿：同首稿

## FACT SET 6（sharded_across_processes 打印行 / 进度文件行）

17. 英文项：6a「67 lines = the first pass ... 8 lines total; the second pass ... 59 lines total.」+「55 lines: one header line plus 54 slice lines」
    原文文件:行：`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:822-826`（「67 行 = 第一趟...共 8 行，第二趟...共 59 行」）、:836（常量 `67`）、:839（「55 行：文件头 + 54 行片行」）
    首稿缺的：:828-835 讲这批钎值的历史来历（头一版取自加分片之前的代码、实审 B3a-2 / B3a-3 状态数 29→54 的两次重取、发现日志加了两行）整段没有译入——判定：这段是「这两个常量是怎么走到今天这个数的」历史说明，与本题「用同一套数行规则把 54 换成 46 重算」这个任务无关，不译入不影响算术
    定稿：不补，理由记于此

18. 英文项：6b「predicted: under 46 states, the printed-line count = ... 58 lines total (pinned today at 67); the progress file has 47 lines...」
    原文文件:行：`research/prompts/m2-impl-r2-fixes-b-implementer-report.md:48`「推的：46 个状态下打印行 = 第一趟 1 + ⌈46/16⌉ 3 + 3 = 7 行、第二趟 1 + 1 + 46 + 3 = 51 行，共 58 行（今天钉 67）；进度文件 47 行（文件头 + 46 行片行）；两个 SHA-256 与进度文件名里的计划哈希推不出来，待提交时 crash-verifier 跑出来再钉。」
    首稿缺的：句尾「两个 SHA-256 与进度文件名里的计划哈希推不出来，待提交时 crash-verifier 跑出来再钉」没有译入——判定：这句讲的是「哪些量不能靠算术推出、要等实跑」，不是本题要算的数字，不译入不影响 58、47 这两个数的算法本身；不译入反而避免模型误以为还要去算 SHA-256
    定稿：不补，理由记于此

## FACT SET 7（FLUSH 计数差 1）

19. 英文项：7a 断言结构描述 + 注释「after C577, publish D closes with one pool barrier...but the device still receives two FLUSHes.」
    原文文件:行：`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2779-2783`（注释「C577 之后发布 D 以一道池屏障收尾，抬 F 那一串开的新写入口开头又一道，两道之间没有写：录制器（`SharedStream::push` 的末尾一串屏障去重）把后一道并掉，设备照收两次 FLUSH，所以每块盘设备一层比录制流投出的多 1」、常量 :2783）、:2808-2813（断言）
    首稿缺的：注释里「所以每块盘设备一层比录制流投出的多 1」这句结论首稿是用「the device still receives two FLUSHes」加上后文「plus...(a constant set to 1)」两处拼出来的等价说法，没有逐字译出「设备一层比录制流投出的多 1」这句话本身；另外「盘 0 的 12 / 13 是合入后验证一那一次跑出来的；盘 1 同一个机制推的，没跑」这句（区分盘 0 是实跑、盘 1 是推的，没跑）没有译入
    定稿：「盘 1 没跑」这条限定确实会影响模型怎么看 7b 给的两个数——但 7b 引的「left: 12 right: 13、left: 16 right: 17」本身就是两次不同断言各自第一次不等的输出，不是「盘 0」「盘 1」两块盘的数（现查报告原文 :257 的上下文，这两对数分别是 `bin_first_transaction_on_device` 两条不同用例各自的断言输出，不是同一条用例的两块盘），首稿称它们为「the first observed pair」「the second observed pair」而不称「盘 0」「盘 1」，这一点与原文一致，不需要改

20. 英文项：7b「both differ by 1 (a log file recorded: left: 12 right: 13, left: 16 right: 17, each assertion only prints the first mismatch).」
    原文文件:行：`research/prompts/m2-impl-merge-verify-1-implementer-report.md:257`「现象：两条都在比录制流投出来的 FLUSH 数与设备一层数到的「屏障 + FUA」，差 1（`run1/bin-first_transaction_on_device.log`：`left: 12 right: 13`、`left: 16 right: 17`，每条只打出第一处不等）。」
    首稿缺的：无，逐句照译（日志文件名本身按提示规则不写路径，只留「a log file recorded」，这是遵守「答复不写代码行号与文件行号」之外「提示自足」的写法，不算漏译）
    定稿：同首稿

## FACT SET 8（变异表 16 行）

21. 英文项：8a「whole-line replacements (7 lines, anchors this round changed): ... not one of these 7 lines was proven red.」
    原文文件:行：`research/prompts/m2-impl-merge-verify-1-implementer-report.md:239`「整行替换（7 行，锚点被这一轮改掉的）：711（实二...）、1266（...）、1332、1333（...）、1334（...）、1335（...）、1336（...）。这 7 行一条都没证红。」
    首稿缺的：7 行各自的锚点说明（711 是哪个函数改名、1266 是哪个模型分支等六处具体内容）没有译入，只留了总数 7 与「一条都没证红」这个结论——判定：本题只要「7」这个数与「没证红」这个结论去和总数、去和证红计数做加减，不需要每一行具体改的是什么
    定稿：不补，理由记于此

22. 英文项：8b「appended (9 lines): two P1 lines...three Q7 lines...one Q4 line...none of these were proven red...Three crash-injection lines...all three of these lines WERE proven red, but proven on a probe copy...」
    原文文件:行：`research/prompts/m2-impl-merge-verify-1-implementer-report.md:240-242`「追加 9 行：P1 两行（已知槽判定...）、Q7 三行（回退两处传空集...）、Q4 一行（抬 F 的错改回...）——都没证红，留给下一件或 59 号。崩溃注入三行（C554 乙那一判关掉...），点名 second_transaction_supplement_three_crash_injection 的三条改钉用例——三行都证过，但证在探针上：那份二进制登记了崩溃枚举用例、重型闸不让跑，我把改后的文件去掉登记的快档那一条、拷成 edit/…/tests/zz_probe_crash_injection.rs 跑 prove-red.sh...」
    首稿缺的：P1、Q7、Q4 各行具体改的是什么（if false && / if true || 之类）与探针跑法的细节（拷成 zz_probe_crash_injection.rs、prove-red.sh 的两行日志原样）没有译入——判定：本题只要「2+3+1=6」「3」「16−3=13」这几个数，不需要每行改的内容与探针文件名；「证在探针上、不是登记的那份文件」这一条限定保留了（"but proven on a probe copy...the probe copy is not the checked-in file"），因为它是本题「16 行里有多少条真正对着登记文件证红」这一问的关键限定，不能省
    定稿：细节不补，理由记于此；「证在探针副本上、不是登记文件」这条限定保留在译文里，逐句核对无误
