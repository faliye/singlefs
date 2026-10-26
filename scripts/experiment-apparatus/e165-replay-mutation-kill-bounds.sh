#!/usr/bin/env bash
# admission: always 每一次调都照此刻的仓、变异表与结果文件判还差哪几个变体，上一次跑完不替这一次作保
# run-condition: command cargo python3 nice git
# E165（恢复重放有界穷举与变异杀伤界）第二段：量变异表里动 replay_journal 那五个函数的每个变体在小界内红不红、首红的 N。
# 登记：research/prompts/e165-preregistration.md 5.5 第二段、5.6 的界、第九节甲表、第十二节修订第 10、11、12 条；
#   第 1 次重跑 research/prompts/e165-r1-prereg.md 5.5 第二段、第九节（判定搬进 crates/singlefs-replay-prefix/src/lib.rs 之后，
#   选行按变异名与它改的判定、不按行号；改的文件取变异行第二列）；
#   第 2 次重跑 research/prompts/e165-r2-prereg.md 5.6 第二段、第九节 9.1–9.5（这份驱动只为 r2 档写）：基线与每个变体调装置都带
#   --profile r2；条款判据栏加 J6Qn 两行与 J7 五行；每个变体逐域报 J6 与 J6Qn 各自的首红（first_red_by_criterion，SQ2）与
#   首个文字不同的 N 上不同的判据名（first_text_red_criteria，SQ5 要的 Z5 那一格）；自造表 Z1–Z5；红例账照落，写完当场钉进钉表、当场核钉全（「钉红例」一段）。
#   r1 档那份第二段要复跑，用 git 里 r1 那份驱动与 r1 冻结的根（r2 登记第十一节「新旧产物怎么并存」）。
#   第 3 次重跑 research/prompts/e165-r3-prereg.md 9.3（第三段，加 --profile r3 模式；不给 --profile 仍是 r2 那一套，一个字不改）：
#   基线与每个变体调装置都是 --profile r3 --bounds segment1 --family root_choice；变体名单是 9.1 那六个名字（1822–1827，按名字选行，
#   改的文件取变异行第二列，原文命中处要落在 choose_root_slot 或 choose_root 里）；每个变体逐 (域, M, U) 比 KJ0–KJ3 的违例数与
#   root_choice_states 行，全跑不早停；产物每个变体一行（first_red_by_criterion、每格违例数 cells、ka_kj2 与 ka_pin）；红例账不落（9.3 第 5 条）。
#   r3 模式要 --root 指第二段冻结根（REGISTERED_R3 钉着它的 recovery.rs、lib.rs、crates/mutations.tsv 的 sha256 与两个函数的区间），
#   内存包装与第二段产物（TV7 的比对对象）取脚本所在的仓、不取 --root（冻结根里只有 Cargo.toml、Cargo.lock、.cargo、crates/）。
#
#   e165-replay-mutation-kill-bounds.sh --profile r3 --root <第二段冻结根> --scratch <草稿目录> --output <产物> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>]
#       r3 模式整轮：开头基线（核 criteria=root_choice 判决行在、与第二段产物 E7RESULT 逐行相同，不同退 6）、每个变体一份副本各编一次、收尾基线；
#       7 个变体都有行、收尾基线与开头逐字相同，才写收尾三行；不给 --ledger（给了退 2）；r3 模式不认 --recheck、--select-rows、--measure（退 2）。
#       选行 4 条（恰 1 行、文件列、原文命中 1 次、命中处落在两个函数里）有一条不满足，那一行记 outcome=not_compared（登记 TH5），别的行照做，整轮退 1。
#       变体在 KA 上 KJ2 的违例数不等于登记钉值（1823 → 18、1824 → 52、1825 → 36、1826 → 18）记 ka_pin=mismatch，整轮跑完退 1（TF8）；
#       1822、1827 的 0、18 是登记里推的，不钉，报 ka_inferred=match|mismatch，不影响退出码。
#   e165-replay-mutation-kill-bounds.sh --profile r3 --root <第二段冻结根> --list-items
#       列 7 行（基线 1 个、变体 6 个），不编不跑。
#
#   第 4 次重跑 research/prompts/e165-r4-prereg.md 9.3（第二段，加 --profile r4 模式；不给 --profile 仍是 r2 那一套，--profile r3 仍是 r3 那一套，一个字不改）：
#   基线与每个变体调装置都是 --profile r4 --bounds segment1 --family root_choice_faults；变体名单是 9.1 那六个名字（登记名 v1822–v1827，r3 的编号当名字用，在
#   根里的 crates/mutations.tsv 按名字选行，改的文件取变异行第二列，原文命中处要落在 choose_root_slot 或 choose_root 里）；每个变体逐 (域, M, U, 被判时故障,
#   历史标签) 比 KJ0、KJ1、KJ1T、KJ2、KJ3 的违例数与 root_choice_fault_states 行，全跑不早停；产物每个变体一行（first_red_by_criterion 先 KJ1、KJ1T、KJ2 再 KJ0、KJ3，
#   每格违例数合计 cells、ka_kj2 与 ka_pin、kg_pin、differs_on）；红例账不落。四个合法域任一子格上 KJ1、KJ1T、KJ2 与基线不同才记 red；只在 KG 或 KA 上与基线不同记
#   not_red_within_bounds 并报 differs_on=KG|KA|KG+KA。变体在 KA 上 KJ2 的违例数不等于登记钉值（v1823 → 18、v1824 → 52、v1825 → 36、v1826 → 18）记 ka_pin=mismatch，
#   在 KG 上每个 (M, U) 格的 KJ1、KJ1T、KJ2 合计不等于登记第七节「本地择根序」表里它那一列（v1827 取 v1823 那一列）记 kg_pin=mismatch，整轮跑完退 1（TF7）；
#   v1822、v1827 的 KA 读数 0、18 是登记里推的，不钉，报 ka_inferred=match|mismatch，KG 那一列的核不管钉没钉都做（kg_pin）。
#   r4 模式要 --root 指第一段根（R4_REGISTERED 钉着它的 recovery.rs、lib.rs、crates/mutations.tsv 的 sha256 与两个函数的区间），内存包装与第一段产物（TV7 的比对对象）取脚本所在的仓、
#   不取 --root；基线的 E7RESULT 行必须与第一段产物逐行相同（不同退 6）、criteria=root_choice_faults 的判决行要在（缺了退 7）；整遍峰值 RSS 约 1.7 GB，
#   E165_KILL_BOUNDS_MEMORY_CAP 默认 4G（r2 512M、r3 1G），同时跑的变体数 --jobs 照机器的内存排队。
#   e165-replay-mutation-kill-bounds.sh --profile r4 --root <第一段根> --scratch <草稿目录> --output <产物> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>]
#       r4 模式整轮：开头基线、每个变体一份副本各编一次、收尾基线；7 个变体都有行、收尾基线与开头逐字相同，才写收尾三行；不给 --ledger（给了退 2）；
#       r4 模式不认 --recheck、--select-rows、--measure（退 2）；选行 4 条有一条不满足，那一行记 outcome=not_compared（登记 TH5），别的行照做，整轮退 1。
#   e165-replay-mutation-kill-bounds.sh --profile r4 --root <第一段根> --list-items
#       列 7 行（基线 1 个、变体 6 个），不编不跑。
#   下面五项是 r4 登记 9.3 第 6 条那张表的弄坏开关，外面打开时自证同样红（自证的 r4 那几趟把外面给的这五项原样传进去，别的趟不带）：
#   E165_KILL_BOUNDS_BREAK=r4itemsmissing  r4 名单少一条
#       「r4：--list-items 应当列 7 行（基线 1、变体 6）」（实际 6 行）
#       连带红的格：「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：变异名选不到的那一行」
#   E165_KILL_BOUNDS_BREAK=r4firstredswap  首红那一栏里 KJ1T 与 KJ2 两列对调
#       「r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同应当报 KJ1T:KF=M1U2/none/inside、KJ2:KF=M1U0/RN/inside」（实际两列互换）
#   E165_KILL_BOUNDS_BREAK=r4kgcountsasred  KG 当合法域：只在 KG 与基线不同的变体也记 red
#       『r4：假变体只在 KG 与基线不同应当记 not_red_within_bounds、differs_on=KG』（实际 red）
#       连带红的格：「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v…、v… 不钉…」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」
#   E165_KILL_BOUNDS_BREAK=r4kgpinunchecked  不核 KG 钉值（读数不等于那一列也不报 mismatch、不退 1）
#       「r4：假变体的 KG 读数不等于它那一列应当报 kg_pin=mismatch 且整轮退 1」（实际退 0）
#       连带红的格：「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：变异名选不到的那一行」
#   E165_KILL_BOUNDS_BREAK=r4noprofile  调装置不带 --profile r4
#       「r4 整轮应当退 0」（实际退非 0：假装置不认没有 --profile r4 的 --family root_choice_faults）、「r4 整轮里假装置每一次被调都应当带 --profile r4」
#       连带红的格：「✗ 基线没跑完或退出码不是 0（2；日志 …）」、「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：假变体的 KG 读数不等于它那一列」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」、「✗ 自检：r4：变异名选不到的那一行」、「✗ 自检：r4：基线与第一段产物不逐行相同」、「✗ 自检：r4：基线缺 criteria=root_choice_faults 判决行」、「✗ 自检：r4：第一段产物不在时 TV7 的比对」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」
#
#   第 5 次重跑 research/prompts/e165-r5-prereg.md 9.3（第二段）与修订 1「修1.九」乙（第五段 5-乙）：
#   --pins r5   三个模式（不给 --profile、--profile r3、--profile r4）都认：开工核对改用 R5_PINS 那一套（第一段根 /tmp/claude-1000/e165-r5-segment1/root 上的
#       recovery.rs、replay-prefix lib.rs、crates/mutations.tsv 的 sha256 与函数区间；名字、标签、KA / KG 钉值、custom_keys 同旧的），V7 / TV7 的比对对象换成第一段的重跑产物；
#       自造表、标签表、内存包装与第一段产物一律取脚本所在的仓（第一段根里只有 Cargo.toml、Cargo.lock、.cargo、crates/）。不给 --pins 时三个模式照旧取原来三套，一个字不改。
#   --gate r2   只在 --profile r4 --pins r5 同给时认（别的组合退 2、一条都不跑）：调装置在 --bounds segment1 --family root_choice_faults 之后加 --gate r2；
#       基线 family=root_choice_faults 的 name=config 行要带 witness_gate=r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots（5-甲 的装置在 --gate r2 时
#       实际写的闸字段与值，见 5-甲 产物第一行），缺了退 7、变体一个不编；TV7 的比对对象换成 5-甲 的产物；
#       驱动自己写的产物：config 行带 gate=r2 与根里装置的 sha256，每个 kill_bound 行与收尾判决行带 gate=r2（这几处是驱动的写法，装置不写）。
#       R5_GATE_R2_PINS 是 5-甲 的根（第一段根盖上 5-甲 的装置与变异表）上的值：变异表 sha256 与 5-甲 产物名都已填真值，两份源文件与两个函数区间同第一段根（5-甲 不改它们）。
#   下面两项是 r5 的弄坏开关（外面打开时自证的每一趟都带着它）：
#   E165_KILL_BOUNDS_BREAK=r5pinsignored  给了 --pins r5 照样用原来那套钉值
#       「r5：给了 --pins r5 应当按 r5 钉值核对、在旧钉值的假仓上停在 RH1」（三个模式各一格；实际退 0）
#       连带红的格：「✗ 自检：r5：--gate r2 整轮」、「✗ 自检：r5：--gate r2 那一套的变异表钉值与仓里的表对不上时」
#   E165_KILL_BOUNDS_BREAK=r4gateignored  给了 --gate r2 而调装置不带 --gate r2
#       「r5：--gate r2 整轮应当退 0、假装置每一次被调都带 --gate r2、产物带 gate=r2」（实际退 7：基线缺 R2 的闸字段、变体一个不编）
#   E165_KILL_BOUNDS_BREAK=defaultone  --jobs、--threads 都不给时取 1 × 1（改前的样子），不取登记的重活默认并行度
#       「--jobs、--threads 的缺省：…」（实际 (1, 1)）
#   下面两项外面打开时自证的每一趟都带着它（drive() 照下面七项那一段的带法）：
#   E165_KILL_BOUNDS_BREAK=pincheckskipped  钉红例时不跑 --check（当场追加之后不核钉全）
#       「账写完而钉表没钉时驱动当场核钉全应当判红、退 8、一次不编」（实际退 0）、「已收尾的一对重跑同一条命令应当只重做钉红例」（实际「核了 没核」）、
#       「整轮写完账与产物要当场钉进钉表、再核钉全」「补跑补齐之后照样当场钉、当场核」（实际成功那句不报核了几行）
#   E165_KILL_BOUNDS_BREAK=retainedmissingred  留存产物不在时照改前不报本次未跑：V7 读不到（退非 0）、TV7 报 TV8 退 6
#       「r2：第一段产物不在时 V7 的比对应当逐项报本次未跑」「r3：第二段产物不在时 TV7 的比对应当逐项报本次未跑」「r4：第一段产物不在时 TV7 的比对应当逐项报本次未跑」
#       连带红的格：「✗ 作废条款 TV8：第一段产物读不到：…」、「✗ 作废条款 TV8：第二段产物读不到：…」
#
#   e165-replay-mutation-kill-bounds.sh --list-items
#       逐行列变体（基线 1 个、crates/mutations.tsv 里 21 种不同的替换、自造 5 条，共 27 行），不编不跑。
#   e165-replay-mutation-kill-bounds.sh --select-rows --output <产物>
#       只做登记 r1 第九节的选行（RQ4）与标签比对（RQ5、RQ6）：照 23 个变异名在今天的 crates/mutations.tsv 里找行，核 4 条（恰 1 行、文件列是 recovery.rs 或 lib.rs、
#       原文命中 1 次、命中处落在 9.2 的函数里），再把 scripts/mutation/tables/e165_replay_bounded_enumeration_tags.tsv 里每行的标签集合与登记时的集合比；
#       写 <产物>（已存在就不写、退 2），不编不跑。选不到（RQ4 / RQ6 有一行不满足）退 1，逐行比对结果照写；核对输入对不上（停机条款 RH1）退 4、不写。
#       r2 登记 9.1 不判标签（附带照报）：Z5 不在标签表里，它那一行的标签几列写 not_judged_in_r2，只按原文命中 1 次判选没选到。
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output <产物> --ledger <红例账> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>]
#       跑还没落行的变体（给了 --item 就只看点名的）；每个变体跑完立刻记下，产物里照变体的次序落行；重跑同一条命令只补没跑的。
#       27 个变体都有行、收尾的基线与开头逐字相同，才写收尾三行（name=restored、name=verdict、name=done）。
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output <产物> --ledger <红例账> --recheck --item <变体>
#       只复跑点名的那一条，拿新的行与产物里已有的行比，不写产物；不同就退 1。
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --measure <变体>
#       量一次：从改 recovery.rs 到编完 bin 的挂钟与内存峰值（冷编译，经 run-with-memory-cap.sh），再量跑 --bounds segment2 的挂钟。不写产物。
#   e165-replay-mutation-kill-bounds.sh --clean --scratch <草稿目录>
#       删掉这条脚本在草稿目录里建的 kill-bounds/（副本、编译目录、每个变体的记录），删前报大小。
#   e165-replay-mutation-kill-bounds.sh --selftest
#       拿假的仓、假的 cargo 走一遍：锚点命中 0 次必须报「锚点腐化」退 3、一条都不跑；红、界内不红、无效各一条判得对；
#       被打断的结果文件（只剩前一条、末行半截）重跑只补没跑的；弄坏开关开着时这两样必须判错。
#       r2 补的格（登记 r2 9.4 第 6 条）：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同 ⇒ 报 (2, 3)；只有 J6Qn 不同 ⇒ J6 报 none；
#       基线里缺 J6Qn_ 行 ⇒ 报「装置没带 --profile r2」退 7、变体一个不编；自造表 Z 五行时 --list-items 列全五条。
#       r3 补的格（登记 r3 9.3 第 6 条，在自己的假仓里走，不碰上面 r2 的假仓）：--list-items 列 7 行；假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线
#       不同 ⇒ 报 KJ1:K=M1U2、KJ2:K=M1U0；假变体在 KA 上的 KJ2 读数不等于钉值 ⇒ ka_pin=mismatch 且整轮退 1；基线缺 criteria=root_choice 判决行
#       ⇒ 报「装置没带 --profile r3」退 7、变体一个不编；另有基线与第二段产物不同退 6、选行不满足记 not_compared、变体跑到装置自己退 1（TF9 的 halt 行）
#       而照常落行三格。
#   每种用法都可以加 --root <仓根>：被测的仓不是这条脚本所在的那一个时用（默认取脚本所在的仓）。
#
# 做法（每个变体一份副本、一个编译目录，仓里的 crates/ 一个字节不碰）：
#   1. 开工核对：仓里 recovery.rs、lib.rs 与 crates/mutations.tsv 的 sha256 等于 REGISTERED 里的值（r2 登记 9.4 第 1 条：第一段收尾冻结的根上的值；
#      crates/mutations.tsv 那一个在第一段收尾之前填不出，写的是占位串，开工核对一律停在 RH1）、9.2 那九个函数的行区间（从 fn 行到其后第一个只有 } 的行）
#      等于 9.2 的表，不等是停机条款 RH1（退 4，一条都不跑）。自造表的 Z 行名字要等于 REGISTERED 的 custom_keys（Z1–Z5），不等同退 4。再按 23 个变异名从 crates/mutations.tsv 选行（r1 第九节 9.1，不按行号）：每个名字恰 1 行、
#      文件列是 recovery.rs 或 lib.rs、原文在那份文件里命中 1 次、命中处的起行或末行落在 9.2 的函数里，有一个选不到就退 4、一条都不跑；
#      自造表每行的文件列也要是这两份之一。今天的表行号只用来给变体起名（tsv-<行号>）。
#   2. 每条变异的原文在它第二列点名的那份文件里恰好命中一次（\n 还原成换行）；有一条不是就报「锚点腐化」，退 3，一条都不跑。
#   3. 基线（不改的副本）先编先跑 --profile r2 --bounds segment2；每个有 J6 行的 (域, N) 都要有 J6Qn 两行，缺了就是装置没带 --profile r2，退 7、
#      变体一个不编（r2 9.4 第 6 条）；它每个 (域, N) 的行必须逐字出现在第一段 r2 档的两份产物里（作废条款 V7，r2 9.4 第 2 条、SV9），不然退 6
#      （产物有不在的，这一项本次未跑，见「留存产物」一段）。
#   4. 每个变体：把 Cargo.toml、Cargo.lock、.cargo、crates/（不带 target）拷成一份副本，原件另存在副本之外、核 sha256；
#      改坏、`cargo build --release --offline -p singlefs-checker-tier --bin e165_replay_bounded_enumeration`（编不过记「无效」）、
#      跑 --bounds segment2、逐行与基线比；红了的域在首红的 N 上找一个反例状态（两边的 --list-states 逐行比出第一个判定不同的状态），
#      再拿两边的二进制各 --replay-state 单条重放一遍，判定必须与逐态行相同；收尾把原件拷回副本、核 sha256，删掉这份副本与编译目录。
#   5. 收尾再建一份不改的副本重编重跑基线，输出必须与开头逐字相同；仓里 recovery.rs 的 sha256 与 `git status --porcelain crates/`
#      开头与结尾相同（作废条款 V6），不然退 5、不写收尾行。
#   6. 红例账照落，写完收尾三行之后当场钉进 .claude/gate.d/e165-pinned-red-examples.tsv、当场核钉全（「钉红例」一段）。
# 钉红例：不给 --profile 的那一套（写红例账的只有它）写完一对账（--ledger）与 kill-bounds 产物（--output）、收尾三行落盘之后，
#   先跑 python3 scripts/experiment-apparatus/e165-pin-red-examples.py --root <脚本所在的仓> --kill-bounds <产物> <账>（追加：已钉的不重复，档取产物 config 行的
#   device_arguments=，即 --profile r2），再跑同一份的 --check --kill-bounds <产物> <账>（显式给这一对）；钉表与变异表取脚本所在的仓，不取 --root。
#   有一步退非 0，原样转出它的输出、整趟退 8（EXIT_PIN_INCOMPLETE）；都过，成功那句报「钉红例：钉进 N 行、核了 M 行」。
#   产物已收尾的，重跑同一条命令不跑变体，只重做这一步（上一趟钉红例没过、改好之后走这里）。--recheck 不钉。
#   只供自证：E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND 设了（且是自证起的那一趟）就不追加，造「账写完了、钉表没钉」。
# 留存产物：V7、TV7 比的对象（第一段、第二段的产物，五-甲的产物）在 research/results/ 下，那个目录可以整个删掉。比对的对象有不在的，
#   那一项本次未跑：逐个点名不在的产物打一行「! 本次未跑：作废条款 <V7|TV7> 的比对：留存产物 <路径> 不在」（环境里有 GATE_NOT_RUN_FILE 的另往那份文件追加一行），
#   不当绿、不判红，照跑完；基线那一行的比对栏（rows_found_verbatim_in_segment1_products、rows_identical_to_segment1_product、rows_identical_to_segment2_product）
#   与收尾判决行的 baseline_equals_segment1_product、baseline_equals_segment2_product 写 not_run，收尾那句接「本次未跑 N 项：…」。在的照旧比，不同照旧退 6；
#   在而一行 E7RESULT 都读不到照旧按 TV8 退 6。
# 界固定为登记 5.6 的上沿（修订第 10 条去掉了放宽规则）：每个变体整份 --bounds segment2 跑一遍，每个域的「首红的 N」取第一个与基线不同的 N，
#   与「从小往大扫、红了就停」得出的是同一个数；到上沿没红记「界内不红」。
# 「红」的口径：这个 (域, N) 上某条判据的违例数与基线不同。条款判据（J0–J4、J5a、J5b、J6、J6Qn、J7a 三格、J7b、J7c、A0、AS 三条、A-结构、A-限）与
#   实现锚点（IA1、IA2、合法历史一族里原实现报错的态数）分两栏；只有别的量（见证计数、施加过的态数、见证文字）不同的另列，不算红。
#   结局：red（有条款判据红）、red_only_by_implementation_anchor、not_red_within_bounds、invalid（编不过）。
#   first_clause_red 照上面的口径（按违例数、红了就不再往上看那个域）。r2 另报两样，口径是行的文字、整份 --bounds segment2 都比、不早停：
#   first_red_by_criterion=J6:<域>=<N>,…;J6Qn:<域>=<N>,…——J6（J6_premise_holds、J6_premise_fails 两行合看）与 J6Qn（两行合看）各自在每个域上
#   最小的 N，使变体那条判据任一行与基线同一 (域, N) 的同一行文字不同（变体缺那一行也算不同）；到上沿都相同写 none（登记里的「无」），
#   基线在那个域上一行那条判据都没有写 no_rows。
#   first_text_red_criteria=<域>:<N>:<判据>+<判据>;…——每个域上任一条判据的 name=violations 行文字与基线不同的最小 N，与那个 N 上文字不同的全部判据名
#   （SQ5 要 Z5 的这一格；每个变体都报）。
#
# 并行：--jobs 个变体同时做，每个变体的编译与装置各用 --threads 条线程；经 scripts/hosts/capped.sh N 起时 jobs × threads 不许超过 N。
#   缺省：--jobs 不给是 1；--threads 不给是 N ÷ --jobs（向下取整，至少 1），N = SINGLEFS_THREAD_CAP，没设取登记的重活默认并行度（.claude/heavy-thread-default，
#   scripts/hosts/heavy_threads.py）；所以什么都不给是 1 个变体、N 条线程（登记 31 时 jobs 1 × threads 31，--jobs 4 时 4 × 7）。
# 内存：跑编译出来的二进制一律经 scripts/hosts/run-with-memory-cap.sh（上限 E165_KILL_BOUNDS_MEMORY_CAP，默认 512M：
#   2026-10-01 量到基线 --bounds segment1 --family legal 的峰值 47 MiB，变体的用量没逐个量过，留十倍）；退出码 245（显存不够：起止之间内核报了
#   NV_ERR_NO_MEMORY，单独报「显存不够」）与 250–254 的那一次不算结果，那个变体不落行，整条退 1（判法 wrapper_own_problem，三套变体共用）。
#   cargo build 不跑编出来的代码，不经包装。
#
# 退出码：0 点名的都落了行（或本来就都有）；1 有变体没得出结果（包装结局、装置没跑完）、--recheck 对不上、或自证判红；2 用法不对；
#   3 锚点腐化；4 停机条款 H1（含自造表的 Z 行名字与登记不等）；5 作废条款 V6（收尾基线与开头不同，或仓里的 crates/ 在跑的时候变了）；6 作废条款 V7；
#   7 基线里缺 J6Qn_ 行（装置没带 --profile r2，r2 9.4 第 6 条）；8 钉红例没过（当场追加或 --check 退非 0，「钉红例」一段）。
# 弄坏开关 E165_KILL_BOUNDS_BREAK=<项>（只给 --selftest 证明它会红用，逗号分隔可给几个；不认识的项当场退出）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（自证的例没有标签，项名直接写那条「✗ 自检：」失败行的句首，括号里写那一行接着报的）：
#   E165_KILL_BOUNDS_BREAK=gpumemoryasresult  wrapper_own_problem 不认 245（显存不够），照装置自己的退出码往下比
#       「经假内存包装跑一趟、包装退 245 时应当判不算结果、报显存不够」（实际退 245，判的 None）
#   E165_KILL_BOUNDS_BREAK=progressnever  进度间隔的判断永不触发、一行进度都不打
#       「假时钟上过了进度间隔（300 秒）应当各打一行、没到不打」（实际：差一秒 None；到了 None；再过一个间隔 None）
#       「整轮里一个变体跑过进度间隔应当打出点名它的进度行」（实际退 0、进度行 []）
#   下面四项是 r2 登记 9.5「驱动」三行加一项，外面打开时自证同样红（自证的 r2 那几趟把外面给的这四项原样传进去）：
#   E165_KILL_BOUNDS_BREAK=j6qnfromj6  J6Qn 的首红取了 J6 那两行（9.5「J6Qn 的首红取了 J6 的列」）
#       「r2：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同应当报 J6:L=2、J6Qn:L=3」（实际 J6Qn:L=2）
#       连带红的格：「✗ 自检：r2：只有 J6Qn 不同的假变体」
#   E165_KILL_BOUNDS_BREAK=noprofile  调装置（--bounds、--list-states、--replay-state）不带 --profile r2（9.5「基线调装置不带 --profile r2」）
#       「r2 整轮应当退 0」（实际退 7）、「r2 整轮里假装置每一次被调都应当带 --profile r2」
#       连带红的格：「✗ 自检：r2：J6 只差见证文字时按违例数的首红看不见它（L:3 才红、红的是 J6Qn），按文字的首红在 L:2、点名 J6_premi…」、「✗ 自检：r2：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同」、「✗ 自检：r2：只有 J6Qn 不同的假变体」、「✗ 装置没带 --profile r2：基线有 J6 行的 3 个 (域, N) 里 3 个缺 J6Qn_ 行，第一个 L:1」
#   E165_KILL_BOUNDS_BREAK=noprofilecheck  不核基线里有没有 J6Qn_ 行（9.5 同一行写的「第三格由退出非 0 转放行」）
#       「r2：基线里缺 J6Qn_ 行应当报「装置没带 --profile r2」退 7、变体一个不编」（实际退 0）
#   E165_KILL_BOUNDS_BREAK=zfirstfour  自造表只读前四行（9.5「Z 行只读前四条」）
#       「✗ 自检：r2：自造表 Z 五行时 --list-items」（应当列 8 行，实际 7 行）
#   下面四项是 r3 登记 9.3 第 6 条那张表的弄坏开关，外面打开时自证同样红（自证的 r3 那几趟把外面给的这四项原样传进去，别的趟不带）：
#   E165_KILL_BOUNDS_BREAK=r3itemsmissing  r3 名单少一条（9.3「r3 --list-items 7 行」）
#       「r3：--list-items 应当列 7 行（基线 1、变体 6）」（实际 6 行）
#       连带红的格：「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：变异名选不到的那一行」
#   E165_KILL_BOUNDS_BREAK=r3firstredswap  首红那一栏里 KJ1 与 KJ2 两列对调
#       「r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同应当报 KJ1:K=M1U2、KJ2:K=M1U0」（实际 KJ1:K=M1U0、KJ2:K=M1U2）
#   E165_KILL_BOUNDS_BREAK=r3kapinunchecked  不核 KA 上的钉值（读数不等于钉值也不报 mismatch、不退 1）
#       「r3：假变体的 KA 读数不等于钉值应当报 ka_pin=mismatch 且整轮退 1」（实际退 0）
#       连带红的格：「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：变异名选不到的那一行」
#   E165_KILL_BOUNDS_BREAK=r3noprofile  调装置不带 --profile r3
#       「r3 整轮应当退 0」（实际退 1：假装置不认没有 --profile r3 的 --family root_choice）、「r3 整轮里假装置每一次被调都应当带 --profile r3」
#       连带红的格：「✗ 基线没跑完或退出码不是 0（2；日志 …）」、「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同」、「✗ 自检：r3：假变体的 KA 读数不等于钉值」、「✗ 自检：r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红」、「✗ 自检：r3：变异名选不到的那一行」、「✗ 自检：r3：基线与第二段产物不逐行相同」、「✗ 自检：r3：基线缺 criteria=root_choice 判决行」、「✗ 自检：r3：第二段产物不在时 TV7 的比对」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」
# 下面七项外面打开时，自证的每一趟都带着它（drive() 把外面给的并进那一趟自己点名的；r2 那四项照旧只由 r2 那几趟带），红在下面点名的项；
# 自证里对照那几例自己打开同一项（证明前一例的红来自那一处检查），外面打开时那几例本来就开着、不另红：
#   E165_KILL_BOUNDS_BREAK=noanchorcheck  不核锚点、命中 0 次的那条照「改坏」（什么都没改）跑
#       「锚点命中 0 次应当报「锚点腐化」退 3」（实际退 0）、「锚点腐化时一次都不许编、不许写产物」（实际编了 7 次）、
#       「弄坏开关 recoveryonly 开着时文件列是 lib.rs 的那一条应当由红转「锚点腐化」退 3（证明整轮那一格的红来自按第二列改文件）」（实际退 0）
#   E165_KILL_BOUNDS_BREAK=rerunall  不看结果文件里已有的行、每个变体都重跑
#       「补跑应当只编 6 次（基线 2、没跑的变体 4）」（实际 7）、「补跑之后每个变体恰好一行完整的」（实际 baseline、tsv-2 各 2 行）、
#       「补跑出来的变体行应当与整轮的逐字相同」、「--recheck 同一条应当逐字相同、退 0」（实际退 1）
#       连带红的格：「✗ 自检：补跑」、「✗ 自检：补跑补齐之后照样当场钉、当场核：同一批红例已钉，钉表」
#   E165_KILL_BOUNDS_BREAK=byrownumber  选行按登记时的行号、不按变异名（r1 9.5）
#       「选行 ①：名字不在表里应当报 no_row_with_this_name、点名第 2 行、退 1、不编」（实际退 0、编 0 次）、
#       「选行 ②：同名两行应当报 more_than_one_row_with_this_name、退 1、不编」（实际退 0、编 0 次）
#       连带红的格：「✗ 有变体选行不满足（TH5）记了 not_compared，交主 agent；别的变体照判」、「✗ 自检：r3 整轮」、「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：--list-items」、「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同」、「✗ 自检：r3：假变体的 KA 读数不等于钉值」、「✗ 自检：r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红」、「✗ 自检：r3：变异名选不到的那一行」、「✗ 自检：r4 整轮」、「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：--list-items」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：假变体的 KG 读数不等于它那一列」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」、「✗ 自检：r4：变异名选不到的那一行」、「✗ 自检：r5：--gate r2 整轮」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」、「✗ 自检：选行 ⑧：登记名那一行不成形（cargo 参数那一段空着）」
#   E165_KILL_BOUNDS_BREAK=nodupcheck  同名多行时不报、取第一行
#       「选行 ②：同名两行应当报 more_than_one_row_with_this_name、退 1、不编」（实际退 0、编 0 次）
#   E165_KILL_BOUNDS_BREAK=owntablesplit  选行回到自己切 crates/mutations.tsv（四段以上就收、不看哪一段空着），不经 scripts/mutation/crates-mutation-rows.py 的 read_table
#       「选行 ⑧：登记名那一行不成形（cargo 参数那一段空着）应当报 no_row_with_this_name、点名第 2 行、退 1、不编」（实际退 0）
#   E165_KILL_BOUNDS_BREAK=nolocationcheck  不核命中处在不在 9.2 的函数里
#       「选行 ③：原文在 9.2 的函数之外应当报 outside_the_registered_functions、点名第 5 行、退 1、不编」（实际退 0、编 0 次）
#   E165_KILL_BOUNDS_BREAK=recoveryonly  改坏时只写 recovery.rs、不看第二列
#       「整轮应当退 0」（实际 3）、「文件列是 lib.rs 的那一条应当改 lib.rs、记 outcome=red」、「自造的那一条（文件列 lib.rs）应当记 red」、
#       「收尾要打「已还原，基线与开头逐字相同」并写 name=restored 行」、「收尾行应当是 name=done emitted=10」（实际是一条 name=kill_bound 行）、
#       「整轮应当编 7 次（基线 2、变体 5）」（实际 4）、「补跑应当退 0」（实际 3）、「补跑应当只编 6 次（基线 2、没跑的变体 4）」（实际 3）、
#       「补跑之后每个变体恰好一行完整的」（实际 tsv-5、Z1 各 0 行）、「补齐之后要写收尾行」、
#       「弄坏开关 rerunall 开着时应当每个变体都重编（证明上一例的 6 次来自只补没跑的）」（实际 4）、「r2 整轮应当退 0」（实际 3）
#       连带红的格：「✗ 自检：r2：第一段产物不在时 V7 的比对」、「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同」、「✗ 自检：r3：假变体的 KA 读数不等于钉值」、「✗ 自检：r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红」、「✗ 自检：r3：变异名选不到的那一行」、「✗ 自检：r3：第二段产物不在时 TV7 的比对」、「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：假变体的 KG 读数不等于它那一列」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」、「✗ 自检：r4：变异名选不到的那一行」、「✗ 自检：r4：第一段产物不在时 TV7 的比对」、「✗ 自检：已收尾的一对重跑同一条命令」、「✗ 自检：弄坏开关 pincheckskipped 开着时」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」、「✗ 自检：整轮写完账与产物要当场钉进钉表、再核钉全：红例账里有 1 行反例，钉表」、「✗ 自检：补跑补齐之后照样当场钉、当场核：同一批红例已钉，钉表」、「✗ 自检：账写完而钉表没钉时驱动当场核钉全」、「✗ 锚点腐化：Z1 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」、「✗ 锚点腐化：tsv-2 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」、『✗ 锚点腐化：tsv-3 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次』、「✗ 锚点腐化：tsv-5 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」
# 进度：跑着的时候每过 PROGRESS_INTERVAL_SECONDS 秒（300，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」定的进度间隔）往 stderr 打一行
#   E165_KILL_BOUNDS_PROGRESS：挂钟、变体做完几个（共几个），在做的每一项在哪一步（建副本与编译、跑 --bounds segment2、在哪个 域:N 上找红例或单条重放）、
#   这一步与这一项各跑了多久。开头与收尾的基线也算在做的项。
#   只供自证：E165_KILL_BOUNDS_PROGRESS_INTERVAL=<秒> 换掉间隔，只在自证起的那一趟（E165_KILL_BOUNDS_SELFTEST_ROOT 也设了）认。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
E165_KILL_BOUNDS_SELF_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
export E165_KILL_BOUNDS_SELF_PATH
exec python3 - "$@" <<'PYTHON'
import concurrent.futures
import hashlib
import importlib.util
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time

sys.dont_write_bytecode = True
SELF_PATH = os.environ["E165_KILL_BOUNDS_SELF_PATH"]
SCRIPT_DIRECTORY = os.path.dirname(SELF_PATH)
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
from heavy_threads import default_heavy_threads  # noqa: E402
DEFAULT_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
BIN_NAME = "e165_replay_bounded_enumeration"
RECOVERY_PATH = "crates/singlefs-core/src/recovery.rs"
LIB_PATH = "crates/singlefs-replay-prefix/src/lib.rs"
SOURCE_FILES = (RECOVERY_PATH, LIB_PATH)     # 变异改的文件取变异行第二列，只认这两份（r1 9.1）
TABLE_PATH = "crates/mutations.tsv"
CUSTOM_TABLE_PATH = "scripts/mutation/tables/e165_replay_bounded_enumeration_custom.tsv"
TAGS_TABLE_PATH = "scripts/mutation/tables/e165_replay_bounded_enumeration_tags.tsv"   # r1 9.3：每个变体在两版源码上的判定标签集合与依据
COPIED_ROOT_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain", "rust-toolchain.toml", "crates")
WORK_DIRECTORY_NAME = "kill-bounds"
# crates/mutations.tsv 的登记值在第一段收尾之前填不出（第一段要往表里加装置的行）：占位串让开工核对一律停在 RH1，填了真值才跑得起来。
TABLE_SHA256_PENDING = "pending-segment1-frozen-root"
# 登记 r2（research/prompts/e165-r2-prereg.md）9.4 第 1 条：两份源文件的 sha256 取第三节记的值（第一段不许动这两份，SH1，冻结的根上应当相同）、
# crates/mutations.tsv 的 sha256 取第一段收尾冻结的根上的值（没填之前是占位串）、9.2 的函数行区间取 2026-10-02 在工作区上现算的那一张；
# 冻结的根上重算与这里不同，就以冻结的根为准改这里并写进登记第十二节修订（行号不是判据，名字与函数才是）。
# 9.1 的 23 个变异名（登记时的表行号与 r1 登记时的标签集合）同 r1；z_tags 是 r1 9.4 给 Z1–Z4 登记的标签集合（r2 不判标签，Z5 没有登记集合）；
# custom_keys 是 r2 9.3 的 Z1–Z5。变异名取登记时 HEAD:crates/mutations.tsv 的第一列逐字；按名字选行，表行号只在产物里标出登记时在第几行。
REGISTERED = {
    "files_sha256": {
        RECOVERY_PATH: "3d79b99ece9bb7a308c8c503f31f445539d0f8020a12f1c6d7ab05f21f5b6acd",
        LIB_PATH: "e53aeb448cf2b6b63fae4bbc1c225656d5aed9094a2db94992faae5284a6b397",
    },
    "table_sha256": "9d5fd9fc20a555a5a98043afcd45293b1c3bc69a523e1c8d35531b69b0d278f0",
    "functions": [
        (LIB_PATH, "record_ends_its_publish", 1055, 1063),
        (LIB_PATH, "readable_record_of_the_same_publish_follows", 1067, 1182),
        (LIB_PATH, "counter_of_the_last_record_the_root_covers", 1186, 1254),
        (LIB_PATH, "is_above_the_chosen_root", 1257, 1266),
        (LIB_PATH, "maximum_transaction_of", 1269, 1315),
        (LIB_PATH, "judge_the_candidate", 1326, 1491),
        (LIB_PATH, "replay_core", 1497, 1876),
        (RECOVERY_PATH, "judge_back_chain_against_the_previous_record_on_disk", 2965, 2997),
        (RECOVERY_PATH, "replay_journal", 3028, 3178),
    ],
    "names": [
        (23, "步 3：前缀跨实例边界（把别的实例的记录也接上）", ["PD1"]),
        (24, "步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）", ["PD7"]),
        (36, "步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条", ["PD6"]),
        (226, "只供测试的开关：关掉点名验证时 verification_passed 照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条）", ["PD18"]),
        (354, "增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）", ["PD17"]),
        (355, "增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上", ["PD1"]),
        (406, "并行线一：C376：每条记录都当成它那次发布的末条（发布边界不认，前缀停在发布中间也施加）", ["PD19"]),
        (414, "并行线一：崩溃点重放：发布边界不认（每条记录都当末条）", ["PD19"]),
        (448, "P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b）", []),
        (512, "实二十：锚点取读法甲——所选根那次发布读得出的记录里 jsn 最大那条，不看末条标志（替换第 438 行那条锚点已腐化的变异）", ["PD3", "PD4"]),
        (513, "实二十：所选根那次发布带末条标志的多于一条时不停下（条款没写那一格照常接链）", ["PD4"]),
        (514, "实二十：一次发布之内跳号不断链", ["PD13"]),
        (515, "实二十：一个事务的提交标记没出现、下一条换了事务号也照接", ["PD14"]),
        (516, "实二十：末条不带提交标记也整次施加", ["PD15"]),
        (517, "实二十：崩溃点重放 L8：恢复在不带提交标记的记录上就停（一事务一条时的旧写法，跨两条记录的事务永远施加不上）", ["PD15"]),
        (518, "实二十：崩溃点重放 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去）", ["PD20"]),
        (613, "实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接", ["PD11"]),
        (614, "实二二三 b（C540，D23 已定项 4 读者规则）：带末条标志的那一条之后同一 (实例, txg) 还有记录照字面按标志认边界", ["PD16"]),
        (920, "实审 A2a 第 25 条：链值不等的记录照样进重放前缀", ["PD8"]),
        (921, "实审 A2a 第 25 条：前一条不在盘上也断前缀", ["PD9"]),
        (922, "实审 A2a 第 25 条：链值比较写反（对得上的记录被挡）", ["PD8"]),
        (1003, "实审 A2b 第 36 条：所选根的 txg 改回裸加一", ["PD5"]),
        (1004, "实审 A2b 第 36 条：所选根的 txg 加二才算溢出（边界）", ["PD5", "PD6"]),
    ],
    "z_tags": {"Z1": ["PD2"], "Z2": ["PD2"], "Z3": ["PD17"], "Z4": ["PD2"]},
    "custom_keys": ["Z1", "Z2", "Z3", "Z4", "Z5"],
    # r2 9.4 第 2 条：开头那份基线的逐行对应对的是第一段 r2 档的两份产物（5.6 的名字；日期是第一段跑的那一天，与这里不同就随冻结值一起改）。
    "segment1_products": [
        "research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-legal-family.out",
        "research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family.out",
    ],
}
# 原文与替换文逐字相同的两对（原登记第九节）：355 与 23、414 与 406，标签表里共用前一行的依据。
SAME_AS = {355: 23, 414: 406}
# 装置的档参数（r2 登记 5.1、9.4 第 2 条）：--bounds、--list-states、--replay-state 每一次都带。
DEVICE_PROFILE_ARGUMENTS = ["--profile", "r2"]
# r2 9.4 第 3 条：条款判据栏加 r2 档的新行，名字与装置的输出行同一组。
J6_ROWS = ("J6_premise_holds", "J6_premise_fails")
J6QN_ROWS = ("J6Qn_premise_holds", "J6Qn_premise_fails")
J7_ROWS = ("J7a_instance_table", "J7a_allocation_record_tree_root", "J7a_filesystem_identifier", "J7b", "J7c")
CLAUSE_CRITERIA = {"J0", "J1", "J2", "J3", "J4", *J6_ROWS, *J6QN_ROWS, *J7_ROWS, "A0", "AS_whole_table",
                   "AS_consecutive_gap_continues", "AS_consecutive_gap_stops", "A_structure", "A_limit"}
ANCHOR_CRITERIA = {"IA1", "IA2"}
# 登记第七节甲类锚点：这几个域里原实现报错的态数是 0。
DOMAINS_WHERE_FAILED_STATES_ARE_PINNED = {"L", "L6", "L9", "U_verify_on", "X", "E1"}
ADVERSARIAL_DOMAINS = {"A", "A6", "AC"}
EXIT_OK, EXIT_INCOMPLETE, EXIT_USAGE, EXIT_ANCHOR, EXIT_HALT_H1, EXIT_VOID_V6, EXIT_VOID_V7, EXIT_NOT_PROFILE_R2 = 0, 1, 2, 3, 4, 5, 6, 7
EXIT_PIN_INCOMPLETE = 8   # 钉红例（文件头「钉红例」）：当场追加或 --check 没过
# 钉红例用的那一份：追加与 --check 的写法在它文件头
PIN_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "e165-pin-red-examples.py")
BREAK_MODES = ("noanchorcheck", "rerunall", "progressnever", "byrownumber", "nodupcheck", "nolocationcheck", "recoveryonly",
               "j6qnfromj6", "noprofile", "noprofilecheck", "zfirstfour",
               "r3itemsmissing", "r3firstredswap", "r3kapinunchecked", "r3noprofile",
               "r4itemsmissing", "r4firstredswap", "r4kgcountsasred", "r4kgpinunchecked", "r4noprofile",
               "r5pinsignored", "r4gateignored", "gpumemoryasresult", "owntablesplit", "defaultone",
               "pincheckskipped", "retainedmissingred")
# 外面打开时自证的 r2 那几趟原样传进去的四项（文件头「弄坏开关」）
R2_BREAK_MODES = ("j6qnfromj6", "noprofile", "noprofilecheck", "zfirstfour")
# 外面打开时自证的 r3 那几趟原样传进去的四项（登记 r3 9.3 第 6 条的表）；别的趟不带
R3_BREAK_MODES = ("r3itemsmissing", "r3firstredswap", "r3kapinunchecked", "r3noprofile")
# 外面打开时自证的 r4 那几趟原样传进去的五项（登记 r4 9.3 第 6 条的表）；别的趟不带
R4_BREAK_MODES = ("r4itemsmissing", "r4firstredswap", "r4kgcountsasred", "r4kgpinunchecked", "r4noprofile")
# ── r3 档（登记 r3 9.3，第三段）。不给 --profile 是 r2 档，上面那一套一个字不动。
R3_DEVICE_PROFILE_ARGUMENTS = ["--profile", "r3"]
R3_BOUNDS_ARGUMENTS = ["--bounds", "segment1", "--family", "root_choice"]
R3_LEGAL_DOMAINS = ("K", "K9", "KW")
R3_ALL_DOMAINS = ("K", "K9", "KW", "KA")
# 首红先报登记要的 KJ1、KJ2（TQ5），再附带 KJ0、KJ3
R3_CRITERIA = ("KJ1", "KJ2", "KJ0", "KJ3")
R3_ROW_CRITERIA = ("KJ0", "KJ1", "KJ2", "KJ3")
# 取第二段冻结根上的值（第二段报告第九节）：recovery.rs、lib.rs 整份哈希与登记第三节相同；crates/mutations.tsv 是根里那一份（第二段盖上装置变异行之后）；
# 两个函数的区间用 r1 9.2 的取法（fn 行到其后第一个只有 } 的行）在根上现算；names 的第三项是 KA 上 KJ2 的违例数：("pinned", n) 是登记 5.7、第七节钉的值
# （1823 → 18、1824 → 52、1825 → 36、1826 → 18，TQ6），("inferred", n) 是登记 9.1 表里推的、没钉（1822 → 0、1827 → 18）。名字取根里 crates/mutations.tsv 第一列逐字。
R3_REGISTERED = {
    "files_sha256": {
        RECOVERY_PATH: "9337dc1b50eb13afdb346ee0a58360709e3a9f2bba6e5da8948fdda5685edd18",
        LIB_PATH: "40cd99b58242a5e522a9e302fa83ceaed17cdd5c2ea8e8811e0e223a4649f08c",
    },
    "table_sha256": "0a52bf84cd1d811619d24ae55f4f5da2866d80ce7e99d0db255501b6e5c96977",
    "functions": [
        (LIB_PATH, "choose_root_slot", 3977, 4016),
        (RECOVERY_PATH, "choose_root", 1250, 1287),
    ],
    "names": [
        (1822, "择根核平局取后者赢（里程碑三七·4 择根核，原型 m1_tie_last_wins）：择新键完全相同的两个槽取了后面那个，不是读根环次序里前面那个", ("inferred", 0)),
        (1823, "择根核比较不看实例代号（D22 已定项 7 平局实例代号高者赢；原型 m2_ignore_instance）：txg 平局时停在读根环次序里前面那条实例代号低的根", ("pinned", 18)),
        (1824, "择根核取第一个有效者（D22 已定项 24 先逐个验证全部候选、再在有效者中择新；原型 m3_first_valid_wins）：择了一个自证过的槽就不再换", ("pinned", 52)),
        (1825, "择根核 txg 平局时实例代号低者赢（C650 改坏择根序：同 checkpoint_txg 平局取低）", ("pinned", 36)),
        (1826, "择根核以实例代号为主、txg 只破平局（C650 改坏择根序：实例代号为主）", ("pinned", 18)),
        (1827, "择根胶水摊择新键时不带实例代号（核看到的实例代号都是 0，txg 平局时停在读根环次序里前面那条）", ("inferred", 18)),
    ],
    # TV7 的比对对象：第二段的产物（取脚本所在的仓里的 research/results/，不取 --root）
    "segment2_product": "research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out",
}
# 登记第七节「本地择根序加理想重放」表 KG 那八行（出自设计员的 anchors_r4.out，sha256 5a36b72237f84c52e6901ae5affcfc5431fd4c8b123f5b9e476907b749ba1fc7）：
# 每个 (M, U) 格上 KJ1 / KJ1T / KJ2 的违例数，按择根序分列；第二段拿它当每条变体在 KG 上每格的钉值（TQ10，合法域那几行只当本地对照、不钉变体）。
R4_KG_RULE_READINGS = {
    "today": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 0), (2, 1): (0, 0, 0), (2, 2): (53154, 53154, 0), (2, 3): (265923, 284607, 0)},
    "v1822": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 0), (2, 1): (0, 0, 0), (2, 2): (53154, 53154, 0), (2, 3): (265923, 284607, 0)},
    "v1823": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 0), (2, 1): (0, 0, 0), (2, 2): (53154, 53154, 0), (2, 3): (265923, 284607, 0)},
    "v1824": {(1, 0): (0, 0, 8), (1, 1): (0, 0, 117), (1, 2): (468, 468, 351), (1, 3): (1404, 1404, 1053), (2, 0): (0, 0, 10824), (2, 1): (0, 0, 88296), (2, 2): (292302, 292302, 423162), (2, 3): (1765179, 1770093, 1756485)},
    "v1825": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 0), (2, 1): (0, 0, 0), (2, 2): (53154, 53154, 0), (2, 3): (265923, 284607, 0)},
    "v1826": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 36), (2, 1): (0, 0, 630), (2, 2): (58824, 58824, 7236), (2, 3): (309609, 346005, 49680)},
}

# ── r4 档（登记 r4 9.3，第二段）。不给 --profile 是 r2 档、--profile r3 是 r3 档，上面两套一个字不动。
R4_DEVICE_PROFILE_ARGUMENTS = ["--profile", "r4"]
R4_BOUNDS_ARGUMENTS = ["--bounds", "segment1", "--family", "root_choice_faults"]
R4_LEGAL_DOMAINS = ("KF", "KF3", "KF9", "KFW")
R4_ALL_DOMAINS = ("KF", "KF3", "KF9", "KFW", "KG", "KA")
# 首红先报登记要的 KJ1、KJ1T、KJ2（TQ9），再附带 KJ0、KJ3
R4_CRITERIA = ("KJ1", "KJ1T", "KJ2", "KJ0", "KJ3")
R4_ROW_CRITERIA = ("KJ0", "KJ1", "KJ1T", "KJ2", "KJ3")
R4_CLAUSE_CRITERIA = ("KJ1", "KJ1T", "KJ2")
# 首红的次序：同一 (M, U) 格里先按被判时故障（无、RN、RI、D0、D1），再按历史标签（罩着、不罩）（登记 TQ9）
R4_JUDGED_FAULTS = ("none", "RN", "RI", "D0", "D1")
R4_HISTORY_LABELS = ("inside_the_promise", "outside_the_promise")
R4_SUBCELLS = tuple((fault, label) for fault in R4_JUDGED_FAULTS for label in R4_HISTORY_LABELS)
# 变体登记名（r3 的编号，今天主工作区里在第 1821–1826 行）对 KG 钉值取哪一列：v1827 对 v1823 那一列，别的对各自那一列（登记 5.6、第七节）
R4_KG_COLUMN_OF = {"v1822": "v1822", "v1823": "v1823", "v1824": "v1824", "v1825": "v1825", "v1826": "v1826", "v1827": "v1823"}
# 取第一段根上的值（登记 r4 修订 2）：recovery.rs、lib.rs 整份哈希与登记第三节相同；crates/mutations.tsv 是根里那一份（盖上装置变异行之后）；
# 两个函数的区间用 r1 9.2 的取法（fn 行到其后第一个只有 } 的行）在根上现算；names 的第一项是今天主工作区里的表行号（只用来给变体起名与在产物里标出来），
# 第三项是 KA 上 KJ2 的违例数：("pinned", n) 是登记第七节钉的值（v1823 → 18、v1824 → 52、v1825 → 36、v1826 → 18，TQ10），("inferred", n) 是登记 9.1 表里推的、没钉
# （v1822 → 0、v1827 → 18）；labels 是各行在登记里的变体名（v1822–v1827，r3 的编号当名字用，同一次序）。名字取根里 crates/mutations.tsv 第一列逐字。
R4_REGISTERED = {
    "files_sha256": {
        RECOVERY_PATH: "264664af7a694fbda1295ac450c3d9c2198336d002b3c288b0f797731350e625",
        LIB_PATH: "40cd99b58242a5e522a9e302fa83ceaed17cdd5c2ea8e8811e0e223a4649f08c",
    },
    "table_sha256": "db2981d0d6fccb1a65bdf701897d4475302ec1ba8cd87501fa1374d191bd9349",
    "functions": [
        (LIB_PATH, "choose_root_slot", 3977, 4016),
        (RECOVERY_PATH, "choose_root", 1255, 1292),
    ],
    "names": [
        (1821, "择根核平局取后者赢（里程碑三七·4 择根核，原型 m1_tie_last_wins）：择新键完全相同的两个槽取了后面那个，不是读根环次序里前面那个", ("inferred", 0)),
        (1822, "择根核比较不看实例代号（D22 已定项 7 平局实例代号高者赢；原型 m2_ignore_instance）：txg 平局时停在读根环次序里前面那条实例代号低的根", ("pinned", 18)),
        (1823, "择根核取第一个有效者（D22 已定项 24 先逐个验证全部候选、再在有效者中择新；原型 m3_first_valid_wins）：择了一个自证过的槽就不再换", ("pinned", 52)),
        (1824, "择根核 txg 平局时实例代号低者赢（C650 改坏择根序：同 checkpoint_txg 平局取低）", ("pinned", 36)),
        (1825, "择根核以实例代号为主、txg 只破平局（C650 改坏择根序：实例代号为主）", ("pinned", 18)),
        (1826, "择根胶水摊择新键时不带实例代号（核看到的实例代号都是 0，txg 平局时停在读根环次序里前面那条）", ("inferred", 18)),
    ],
    "labels": ["v1822", "v1823", "v1824", "v1825", "v1826", "v1827"],
    "kg_columns": None,   # 下面一行填进来（自证里换成假的）
    # TV7 的比对对象：第一段的产物（取脚本所在的仓里的 research/results/，不取 --root）；两遍的 E7RESULT 逐字节相同，取最后一次源码改动之后那一遍
    "segment1_product": "research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out",
}
R4_REGISTERED["kg_columns"] = {rule: {f"{m},{u}": list(reading) for (m, u), reading in cells.items()} for rule, cells in R4_KG_RULE_READINGS.items()}

# ── 第 5 次重跑的钉值（登记 research/prompts/e165-r5-prereg.md 9.3 第 1、2 条，第二段）：--pins r5 选它；不给 --pins 时三个模式照旧取上面三套，一个字不改。
# 都是第一段根（/tmp/claude-1000/e165-r5-segment1/root，清单 root-manifest-at-start.sha256 的 sha256 95237ecf…）上 2026-10-04 现算的：两份源文件与 crates/mutations.tsv 的整份 sha256，
# 函数区间照 r1 9.2 的取法（fn 行到其后第一个只有 } 的行）；names 的第一项换成根里 crates/mutations.tsv 的表行号（只用来起名），名字、标签、KA 钉值、labels、KG 钉值、custom_keys 不动。
R5_FILES_SHA256 = {
    RECOVERY_PATH: "4454f6d3eab170121ccb92778e7d4525cdaa6a10a6c6bf16d7c213d0edc22632",
    LIB_PATH: "9966f42f396d22c2a90bdd35864557edd7ce45285be3d6b796ba9f8d18aa2fa4",
}
R5_TABLE_SHA256 = "bc2627f85100b155c14b2d7720488e2a49c7a87a20a45325343a22870942249d"
R5_R2_TABLE_ROWS = [23, 24, 36, 224, 347, 348, 399, 407, 441, 505, 506, 507, 508, 509, 510, 511, 604, 605, 908, 909, 910, 990, 991]
R5_ROOT_CHOICE_TABLE_ROWS = [1810, 1811, 1812, 1813, 1814, 1815]
R5_ROOT_CHOICE_FUNCTIONS = [(LIB_PATH, "choose_root_slot", 3980, 4019), (RECOVERY_PATH, "choose_root", 1268, 1305)]
R5_PRODUCT_PREFIX = "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-"
R5_PINS = {
    "r2": dict(REGISTERED, files_sha256=R5_FILES_SHA256, table_sha256=R5_TABLE_SHA256,
               functions=[
                   (LIB_PATH, "record_ends_its_publish", 3101, 3109),
                   (LIB_PATH, "readable_record_of_the_same_publish_follows", 3113, 3228),
                   (LIB_PATH, "counter_of_the_last_record_the_root_covers", 3232, 3300),
                   (LIB_PATH, "is_above_the_chosen_root", 3303, 3312),
                   (LIB_PATH, "maximum_transaction_of", 3315, 3361),
                   (LIB_PATH, "judge_the_candidate", 3372, 3537),
                   (LIB_PATH, "replay_core", 3543, 3975),
                   (RECOVERY_PATH, "judge_back_chain_against_the_previous_record_on_disk", 3404, 3436),
                   (RECOVERY_PATH, "replay_journal", 3467, 3614),
               ],
               names=[(row, name, tags) for row, (_registered_row, name, tags) in zip(R5_R2_TABLE_ROWS, REGISTERED["names"])],
               # V7 的比对对象：第一段重跑的 r2 档合法历史、对抗两份（登记第十一节末尾比对对象表第 4、5 行的重跑）
               segment1_products=[R5_PRODUCT_PREFIX + "segment1-r2-legal-family.out", R5_PRODUCT_PREFIX + "segment1-r2-adversarial-family.out"]),
    "r3": dict(R3_REGISTERED, files_sha256=R5_FILES_SHA256, table_sha256=R5_TABLE_SHA256, functions=list(R5_ROOT_CHOICE_FUNCTIONS),
               names=[(row, name, pin) for row, (_registered_row, name, pin) in zip(R5_ROOT_CHOICE_TABLE_ROWS, R3_REGISTERED["names"])],
               # TV7 的比对对象：第一段重跑的 r3 择根那一份（比对对象表第 15 行的重跑）
               segment2_product=R5_PRODUCT_PREFIX + "segment1-r3-root-choice.out"),
    "r4": dict(R4_REGISTERED, files_sha256=R5_FILES_SHA256, table_sha256=R5_TABLE_SHA256, functions=list(R5_ROOT_CHOICE_FUNCTIONS),
               names=[(row, name, pin) for row, (_registered_row, name, pin) in zip(R5_ROOT_CHOICE_TABLE_ROWS, R4_REGISTERED["names"])],
               # TV7 的比对对象：第一段重跑的 r4 带故障那一份（比对对象表第 16 行的重跑）
               segment1_product=R5_PRODUCT_PREFIX + "segment1-r4-root-choice-faults.out"),
}
# 第五段 5-乙（登记修订 1「修1.九」乙第 1–4 条）：--gate r2 只在 --profile r4 --pins r5 同给时认；调装置在 R4_BOUNDS_ARGUMENTS 之后加这两项。
R4_GATE_R2_ARGUMENTS = ["--gate", "r2"]
# 基线 family=root_choice_faults 的 name=config 行要带的 R2 闸字段与值：5-甲 的装置在 --gate r2 时实际写的（产物 R5_GATE_R2_PINS["segment1_product"] 第一行的 witness_gate=…）
R4_GATE_R2_CONFIG_FIELD = ("witness_gate", "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots")
# 5-乙 跑在 5-甲 的根上（/tmp/claude-1000/e165-r5-segment3/root5：第一段根盖上 5-甲 改过的装置与 crates/mutations.tsv，登记修1.十一「根、清单、在哪编」）：
# 变异表 sha256 是那份根上的 crates/mutations.tsv 的；TV7 的比对对象是 5-甲 的产物（取脚本所在的仓的 research/results/，同别的几套是路径名）。
# 两份源文件与两个函数区间仍是第一段根上的（5-甲 不改它们；开工核对时对着根现判，对不上停在 RH1）。
R5_GATE_R2_TABLE_SHA256 = "c9a03eb4af85a8e4c7206dfa3609593390516b80ca52be7090aced5696e84046"
R5_GATE_R2_PINS = dict(R5_PINS["r4"], table_sha256=R5_GATE_R2_TABLE_SHA256, segment1_product=R5_PRODUCT_PREFIX + "segment5-r2-gate-root-choice-faults.out")
BREAK = set(filter(None, os.environ.get("E165_KILL_BOUNDS_BREAK", "").split(",")))
for mode in BREAK:
    if mode not in BREAK_MODES:
        sys.exit(f"  ✗ 不认识的弄坏开关：{mode}\n  → 怎么办：E165_KILL_BOUNDS_BREAK 只认 {'、'.join(BREAK_MODES)}")
    print(f"  ! 弄坏开关开着：{mode}（只给自证用，这一次的结果不算数）", file=sys.stderr)


def default_jobs_and_threads(jobs, threads, environment=os.environ):
    """没给 --jobs 是 1；没给 --threads 是 N ÷ jobs（向下取整，至少 1），N 是 SINGLEFS_THREAD_CAP，没设取登记的重活默认并行度。"""
    if "defaultone" in BREAK:   # 弄坏开关：改前的样子
        return jobs or 1, threads or 1
    jobs = jobs or 1
    return jobs, threads or max(1, default_heavy_threads(environment) // jobs)


class Stop(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


PROGRESS_INTERVAL_SECONDS = 300   # 进度间隔（文件头「进度」）


class Progress:
    """跑着的时候每过一个进度间隔往 stderr 打一行（文件头「进度」）。clock 与 sink 可以换成假的（自证）；
    poll 判过没过间隔：过了打一行并交回它，没过交 None。"""

    def __init__(self, total, interval, clock=time.monotonic, sink=None):
        self.total, self.interval, self.clock = total, interval, clock
        self.sink = sink if sink is not None else sys.stderr
        self.lock = threading.Lock()
        self.in_flight = {}
        self.done = 0
        self.started = self.last_line = clock()
        self.stopped = threading.Event()
        self.thread = None

    def phase(self, name, text):
        with self.lock:
            now = self.clock()
            entry = self.in_flight.setdefault(name, {"since": now})
            entry.update(text=text, phase_since=now)

    def finished(self, name, counted=True):
        with self.lock:
            self.in_flight.pop(name, None)
            self.done += 1 if counted else 0

    def due(self, now):
        return "progressnever" not in BREAK and now - self.last_line >= self.interval

    def poll(self):
        now = self.clock()
        if not self.due(now):
            return None
        with self.lock:
            doing = "、".join(f"{name}（{entry['text']}，这一步 {now - entry['phase_since']:.0f} 秒，这一项共 {now - entry['since']:.0f} 秒）"
                             for name, entry in self.in_flight.items()) or "无"
            line = f"E165_KILL_BOUNDS_PROGRESS 挂钟 {now - self.started:.0f} 秒：变体做完 {self.done}/{self.total} 个；在做 {len(self.in_flight)} 项：{doing}"
        self.last_line = now
        print(line, file=self.sink, flush=True)
        return line

    def start(self):
        def loop():
            while not self.stopped.wait(max(0.01, min(self.interval / 4, 5.0))):
                self.poll()
        self.thread = threading.Thread(target=loop, name="e165-kill-bounds-progress", daemon=True)
        self.thread.start()

    def stop(self):
        self.stopped.set()
        if self.thread is not None:
            self.thread.join()


def progress_interval(setup):
    given = os.environ.get("E165_KILL_BOUNDS_PROGRESS_INTERVAL")
    return float(given) if given and setup.selftest else PROGRESS_INTERVAL_SECONDS


def mark(options, name, text):
    """在做的这一项走到哪一步（进度行里报）；没有进度的用法（--measure）什么都不做。"""
    progress = options.get("progress")
    if progress is not None:
        progress.phase(name, text)


def sha256_of_file(path):
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def unescape(text):
    return text.replace("\\n", "\n")


class Setup:
    """这一次跑的仓根与登记值；自证时换成假的仓与假的登记值。"""

    def __init__(self, root, selftest, profile="r2", pins=None, gate=None):
        self.root = root
        self.selftest = selftest
        self.profile = profile
        # --pins r5（登记 r5 9.3 第 1、2 条）；弄坏开关 r5pinsignored：给了也照样用原来那套
        self.pins = pins
        self.gate = gate
        uses_r5_pins = pins == "r5" and "r5pinsignored" not in BREAK
        if uses_r5_pins:
            self.registered = dict(R5_GATE_R2_PINS if gate == "r2" else R5_PINS[profile])
        else:
            self.registered = dict(REGISTERED if profile == "r2" else R3_REGISTERED if profile == "r3" else R4_REGISTERED)
        if selftest:
            name = "selftest-registered" if profile == "r2" else "selftest-registered-r3" if profile == "r3" else "selftest-registered-r4"
            if uses_r5_pins:
                name += "-r5-gate-r2" if gate == "r2" else "-r5"
            with open(os.path.join(root, name + ".json"), encoding="utf-8") as handle:
                self.registered = json.load(handle)
        self.cargo = [os.path.join(root, "fake-cargo")] if selftest else ["cargo"]
        # r3 模式的 --root 是第二段冻结根（只有 Cargo.toml、Cargo.lock、.cargo、crates/）：内存包装与第二段产物取脚本所在的仓（自证里取假仓）。
        self.script_root = root if selftest else DEFAULT_ROOT
        # r2 模式原来的冻结根自己带 research/（自造表、标签表、内存包装、第一段产物都从根里读）；--pins r5 的根是第一段根，只有 crates/ 那几样，
        # 这几样改取脚本所在的仓（登记 r5 第十一节「根、清单、在哪编」：驱动的自造表与 scripts 取脚本所在的仓）。
        self.research_root = self.script_root if pins == "r5" else root
        self.memory_cap_runner = os.path.join(self.research_root if profile == "r2" else self.script_root, "scripts/hosts/run-with-memory-cap.sh")
        # 这一趟本次未跑的比对（留存产物不在，retained_product_not_run 记），收尾那句逐项列出
        self.not_run = []

    def bounds_arguments(self):
        """每次整份跑装置的界参数：r2 是 --bounds segment2；r3 是 --bounds segment1 --family root_choice（登记 r3 9.3 第 3 条）；
        r4 是 --bounds segment1 --family root_choice_faults，给了 --gate r2 时再加 --gate r2（登记 r5 修1.九乙第 1 条；弄坏开关 r4gateignored 不加）。"""
        if self.profile == "r2":
            return ["--bounds", "segment2"]
        if self.profile == "r3":
            return list(R3_BOUNDS_ARGUMENTS)
        gate = list(R4_GATE_R2_ARGUMENTS) if self.gate == "r2" and "r4gateignored" not in BREAK else []
        return list(R4_BOUNDS_ARGUMENTS) + gate

    def pins_arguments(self):
        """复跑命令里要带的 --pins / --gate（照这一次给的原样写）。"""
        return (["--pins", self.pins] if self.pins else []) + (["--gate", self.gate] if self.gate else [])

    def wrap(self, kind, command):
        """跑编译出来的二进制：经内存包装（自证里的假二进制是 shell 脚本，直接跑）。command 第一项是二进制，档参数接在它后面（r2 9.4 第 2 条；r3 同法带 --profile r3）。"""
        if self.profile == "r2":
            profile = [] if "noprofile" in BREAK else DEVICE_PROFILE_ARGUMENTS
        elif self.profile == "r3":
            profile = [] if "r3noprofile" in BREAK else R3_DEVICE_PROFILE_ARGUMENTS
        else:
            profile = [] if "r4noprofile" in BREAK else R4_DEVICE_PROFILE_ARGUMENTS
        command = command[:1] + profile + command[1:]
        if self.selftest:
            return command, dict(os.environ)
        environment = dict(os.environ)
        environment["RUN_WITH_MEMORY_CAP_KEY"] = f"e165-kill-bounds {kind}"
        environment["RUN_WITH_MEMORY_CAP_PARALLEL_REASON"] = ("E165 第二段变体之间并行，每个变体各跑各的二进制（登记修订第 11 条）" if self.profile == "r2"
                                                              else "E165 第 3 次重跑第三段变体之间并行，每个变体各跑各的二进制" if self.profile == "r3"
                                                              else "E165 第 4 次重跑第二段变体之间并行，每个变体各跑各的二进制")
        environment["RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON"] = "变体的内存用量没逐个量过，照基线峰值留十倍"
        environment.setdefault("RUN_WITH_MEMORY_CAP_WAIT_SECONDS", "1800")
        if kind == "segment2 run":
            environment["RUN_WITH_MEMORY_CAP_TIME_LIMIT"] = "3600"
        cap = os.environ.get("E165_KILL_BOUNDS_MEMORY_CAP", "512M" if self.profile == "r2" else "1G" if self.profile == "r3" else "4G")
        return ["bash", self.memory_cap_runner, cap] + command, environment


DEVICE_SOURCE_PATH = "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"


def pins_config_text(setup):
    """给了 --pins r5 / --gate r2 时产物 config 行多带的字段（登记 r5 修1.九乙第 3 条：config 行带上根里装置的 sha256）；都不给时是空串，原来三套的 config 行一个字不变。"""
    text = ""
    if setup.pins:
        text += f" pins={setup.pins} pins_registration=research/prompts/e165-r5-prereg.md"
    if setup.gate:
        device = os.path.join(setup.root, DEVICE_SOURCE_PATH)
        text += f" gate={setup.gate} device_sha256={sha256_of_file(device) if os.path.exists(device) else 'missing'}"
    return text


def read_source(setup, path):
    with open(os.path.join(setup.root, path), encoding="utf-8") as handle:
        return handle.read()


_CRATES_MUTATION_ROWS = []


def mutation_table_rows(setup):
    """crates/mutations.tsv 里成形的行：[(表行号, 各段原样)]。切表与成形照 scripts/mutation/crates-mutation-rows.py 的 read_table
    （门禁 checker-tier-crates-mutation-replay 与 prove-red.sh 同一份）；弄坏开关 owntablesplit 下照改前自己切（四段以上就收）。"""
    if "owntablesplit" in BREAK:
        rows = []
        for number, line in enumerate(read_source(setup, TABLE_PATH).split("\n"), 1):
            if line.startswith("#") or not line.strip():
                continue
            fields = line.split("\t")
            if len(fields) >= 4:
                rows.append((number, fields))
        return rows
    if not _CRATES_MUTATION_ROWS:
        specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_e165_kill_bounds", os.path.join(SCRIPT_DIRECTORY, "..", "mutation", "crates-mutation-rows.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _CRATES_MUTATION_ROWS.append(module)
    rows, _malformed, _memory_from_table, _timeout_from_table = _CRATES_MUTATION_ROWS[0].read_table(os.path.join(setup.root, TABLE_PATH))
    return [(row.line_number, list(row.fields)) for row in rows]


def function_ranges(setup):
    """r1 9.2：函数的行区间 = 从 `fn 名字`（或 `pub fn 名字`）那一行到其后第一行只有 `}` 的那一行。
    返回 [(文件, 名字, 起, 止)]；找不到、不唯一或没有收尾行时起止记 None。"""
    texts, ranges = {}, []
    for path, name, _first, _last in setup.registered["functions"]:
        lines = texts.setdefault(path, read_source(setup, path).split("\n"))
        starts = [number for number, line in enumerate(lines, 1) if re.match(r"^(pub )?fn " + re.escape(name) + r"\b", line)]
        if len(starts) != 1:
            ranges.append((path, name, None, None))
            continue
        end = next((number for number in range(starts[0], len(lines) + 1) if lines[number - 1] == "}"), None)
        ranges.append((path, name, starts[0], end))
    return ranges


def locate_in_functions(ranges, path, first_line, last_line):
    """命中处的起行落在 9.2 的某个函数里就取它，否则末行落在某个函数里取它；都不在交 None。"""
    for line in (first_line, last_line):
        for range_path, name, start, end in ranges:
            if range_path == path and start is not None and start <= line <= end:
                return name, start, end
    return None


def tag_text(tags):
    return ",".join(sorted(tags, key=lambda tag: int(tag[2:]))) or "empty"


def compare_tag_sets(registered, today):
    """r1 9.3：两个集合相等 ⇔ 对得上同一个判定。返回 (相等, 今天多出来的, 今天少了的)。"""
    registered, today = set(registered), set(today)
    return registered == today, sorted(today - registered), sorted(registered - today)


def load_tag_table(setup):
    """scripts/mutation/tables/e165_replay_bounded_enumeration_tags.tsv：键 → 版本 → [(标签们, 在不在, 依据)]；集合 = 标 yes 的标签。"""
    table = {}
    with open(os.path.join(setup.research_root, TAGS_TABLE_PATH), encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            if line.startswith("#") or not line.strip():
                continue
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 5 or fields[1] not in ("old", "today") or fields[3] not in ("yes", "no", "none"):
                raise Stop(EXIT_HALT_H1, f"  ✗ {TAGS_TABLE_PATH} 第 {line_number} 行不是五段、或版本不是 old / today、或第四段不是 yes / no / none\n"
                                        "  → 怎么办：照表头改；这一次什么都没跑")
            table.setdefault(fields[0], {}).setdefault(fields[1], []).append((fields[2], fields[3], fields[4], line_number))
    return table


def tags_of(table, key, version):
    """集合：第四段为 yes 的行的标签（一行可以写几个，逗号连）；空集的行第四段写 none。键或版本缺了交 None。"""
    entries = table.get(key, {}).get(version)
    if entries is None:
        return None
    tags = set()
    for tag, member, _text, _line in entries:
        if member == "yes":
            tags.update(tag.split(","))
    return tags


def select_by_name(setup, ranges, texts):
    """RQ4（r1 9.1）：登记的每个变异名在今天的 crates/mutations.tsv 里按第一列逐字找；那一行要满足四条：恰 1 行、
    第二列是 recovery.rs 或 lib.rs、原文在那份文件里命中 1 次、命中处的起行或末行落在 9.2 的函数里。不按行号选。"""
    table_lines = read_source(setup, TABLE_PATH).split("\n")   # 只给弄坏开关 byrownumber 按登记时的行号取行
    by_name = {}
    for number, fields in mutation_table_rows(setup):
        by_name.setdefault(fields[0], []).append((number, fields))
    selected = []
    for registered_row, name, registered_tags in setup.registered["names"]:
        entry = {"registered_row": registered_row, "name": name, "registered_tags": registered_tags, "failure": None, "today_row": None,
                 "file": None, "hits": None, "first": None, "last": None, "function": None, "function_lines": None, "old": None, "new": None}
        candidates = by_name.get(name, [])
        if "byrownumber" in BREAK:      # 弄坏开关：按登记时的行号取行，不看名字
            candidates = [(registered_row, table_lines[registered_row - 1].split("\t"))] if registered_row <= len(table_lines) else []
        entry["rows_with_this_name"] = len(candidates)
        if not candidates:
            entry["failure"] = "no_row_with_this_name"
        elif len(candidates) > 1 and "nodupcheck" not in BREAK:
            entry["failure"] = "more_than_one_row_with_this_name"
        if entry["failure"] is None:
            number, fields = candidates[0]
            entry.update(today_row=number, file=fields[1], old=unescape(fields[2]), new=unescape(fields[3]), raw_old=fields[2], raw_new=fields[3])
            if fields[1] not in SOURCE_FILES:
                entry["failure"] = "file_column_is_not_a_registered_source"
            else:
                entry["hits"] = texts[fields[1]].count(entry["old"])
                if entry["hits"] != 1:
                    entry["failure"] = f"anchor_hits_{entry['hits']}"
                else:
                    first = texts[fields[1]].count("\n", 0, texts[fields[1]].index(entry["old"])) + 1
                    last = first + entry["old"].count("\n")
                    entry.update(first=first, last=last)
                    located = locate_in_functions(ranges, fields[1], first, last)
                    if located is None and "nolocationcheck" not in BREAK:
                        entry["failure"] = "outside_the_registered_functions"
                    elif located is not None:
                        entry["function"], entry["function_lines"] = located[0], (located[1], located[2])
        selected.append(entry)
    return selected


def custom_rows(setup, texts):
    """自造表每行：四段、文件列是 recovery.rs 或 lib.rs；原文命中次数与所在函数一并交回。"""
    rows = []
    with open(os.path.join(setup.research_root, CUSTOM_TABLE_PATH), encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            if line.startswith("#") or not line.strip():
                continue
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 4 or fields[1] not in SOURCE_FILES:
                raise Stop(EXIT_USAGE, f"  ✗ {CUSTOM_TABLE_PATH} 第 {line_number} 行不是四段、或文件列不是 {' 或 '.join(SOURCE_FILES)}\n"
                                       "  → 怎么办：自造表每行写 变异名、文件、原文、替换文 四段，制表符分隔")
            old = unescape(fields[2])
            hits = texts[fields[1]].count(old)
            first = texts[fields[1]].count("\n", 0, texts[fields[1]].index(old)) + 1 if hits == 1 else None
            rows.append({"line": line_number, "title": fields[0], "key": fields[0].split()[0], "file": fields[1], "old": old,
                         "new": unescape(fields[3]), "hits": hits, "first": first, "last": first + old.count("\n") if first else None})
            if "zfirstfour" in BREAK and len(rows) == 4:   # 弄坏开关：只读前四行
                break
    return rows


def check_the_custom_keys(setup, items):
    """r2 9.3、9.4 第 1 条：自造表的 Z 行名字（依次）要等于登记的 custom_keys（Z1–Z5）；少一行、多一行、换了名都停在 RH1。"""
    keys = [item["name"] for item in items if item["table"] == CUSTOM_TABLE_PATH]
    expected = list(setup.registered.get("custom_keys", []))
    if keys != expected:
        raise Stop(EXIT_HALT_H1, f"  ✗ 停机条款 RH1：{CUSTOM_TABLE_PATH} 的 Z 行是 {'、'.join(keys) or '（一行都没有）'}，登记的是 {'、'.join(expected)}\n"
                                 "  → 怎么办：自造表照登记 r2 9.3（Z1–Z4 不动，加 Z5）；这一次一条变异都没跑")


def load_items(setup):
    """变体清单：基线、照 r1 9.1 按变异名选出的行（原文、替换文与文件都逐字相同的并成一个变体）、自造表的每一行。
    选不到的名字（RQ4 有一条不满足）是停机：退 4、一条都不跑，细节交 --select-rows 的产物。"""
    texts = {path: read_source(setup, path) for path in SOURCE_FILES}
    ranges = function_ranges(setup)
    selection = select_by_name(setup, ranges, texts)
    failed = [entry for entry in selection if entry["failure"] is not None]
    if failed:
        raise Stop(EXIT_HALT_H1, "\n".join(f"  ✗ 停机（r1 RF2 / RQ4）：「{entry['name'][:40]}」今天选不到：{entry['failure']}" for entry in failed)
                   + "\n  → 怎么办：登记里的判定在今天的表与源码里选不到；交回主 agent，不接着跑（--select-rows 写逐行结果）")
    items = [{"name": "baseline", "table": "none", "rows": [], "titles": ["基线（不改）"], "old": None, "new": None, "file": None}]
    by_replacement = {}
    for entry in sorted(selection, key=lambda candidate: candidate["today_row"]):
        key = (entry["file"], entry["raw_old"], entry["raw_new"])
        if key in by_replacement:
            by_replacement[key]["rows"].append(entry["today_row"])
            by_replacement[key]["titles"].append(entry["name"])
            continue
        item = {"name": f"tsv-{entry['today_row']}", "table": TABLE_PATH, "rows": [entry["today_row"]], "titles": [entry["name"]],
                "file": entry["file"], "old": entry["old"], "new": entry["new"]}
        by_replacement[key] = item
        items.append(item)
    for row in custom_rows(setup, texts):
        items.append({"name": row["key"], "table": CUSTOM_TABLE_PATH, "rows": [row["line"]], "titles": [row["title"]],
                      "file": row["file"], "old": row["old"], "new": row["new"]})
    return items, [entry["today_row"] for entry in selection], texts


def source_hashes(setup):
    return {path: sha256_of_file(os.path.join(setup.root, path)) for path in SOURCE_FILES}


def check_the_registered_inputs(setup):
    """停机条款 RH1：两份源文件与变异表的 sha256、9.2 的函数行区间都要等于登记 r1 的值。"""
    problems = []
    for path, actual in source_hashes(setup).items():
        if actual != setup.registered["files_sha256"][path]:
            problems.append(f"{path} 的 sha256 是 {actual}，登记的是 {setup.registered['files_sha256'][path]}")
    expected_table = setup.registered.get("table_sha256")
    if expected_table == TABLE_SHA256_PENDING:
        problems.append(f"{TABLE_PATH} 的登记值还是占位串（{TABLE_SHA256_PENDING}）：r2 9.4 第 1 条要第一段收尾冻结的根上的值，填进 REGISTERED 再跑")
    elif expected_table and sha256_of_file(os.path.join(setup.root, TABLE_PATH)) != expected_table:
        problems.append(f"{TABLE_PATH} 的 sha256 不等于登记的 {expected_table}")
    registered_ranges = [tuple(entry) for entry in setup.registered["functions"]]
    for registered_entry, actual_entry in zip(registered_ranges, function_ranges(setup)):
        if tuple(registered_entry) != tuple(actual_entry):
            problems.append(f"9.2 的函数 {actual_entry[1]} 在 {actual_entry[0]} 里今天的行区间是 {actual_entry[2]}–{actual_entry[3]}，登记的是 {registered_entry[2]}–{registered_entry[3]}")
    if problems:
        raise Stop(EXIT_HALT_H1, "\n".join(f"  ✗ 停机条款 RH1：{problem}" for problem in problems)
                   + "\n  → 怎么办：源码或变异表变了，9.1 的位置、9.2 的行区间都要重核；交回主 agent，不接着跑")


def check_the_anchors(items, texts):
    rotten = [item for item in items if item["old"] is not None and texts[item["file"]].count(item["old"]) != 1]
    if rotten and "noanchorcheck" not in BREAK:
        lines = [f"  ✗ 锚点腐化：{item['name']}（{item['table']} 第 {item['rows'][0]} 行「{item['titles'][0]}」）的原文在 {item['file']} 里命中 "
                 f"{texts[item['file']].count(item['old'])} 次，不是 1 次" for item in rotten]
        raise Stop(EXIT_ANCHOR, "\n".join(lines) + "\n  → 怎么办：把那一行的原文改到今天源码里逐字存在、只出现一次的那一段；这一次一条变异都没跑")


# ── 装置输出的行

def parse_row(line):
    """`E7RESULT name=… k=v …`：值里带空格的字段（witness、rerun），不带「键=」开头的词并进前一个字段。"""
    fields = {}
    last_key = None
    for token in line.split(" ")[1:]:
        if re.match(r"^[a-z_0-9]+=", token):
            key, value = token.split("=", 1)
            fields[key] = value
            last_key = key
        elif last_key is not None:
            fields[last_key] += " " + token
    return fields


def result_rows(text):
    return [line for line in text.split("\n") if line.startswith("E7RESULT ")]


def rows_by_place(rows):
    """(域, N) → {行的键: 字段}；只收带 domain= 与 n= 的行（每个 (域, N) 的量），判决行与自检、对照行不在内。"""
    places = {}
    order = []
    for line in rows:
        fields = parse_row(line)
        if "domain" not in fields or "n" not in fields or fields.get("name") == "state":
            continue
        place = (fields["domain"], int(fields["n"]))
        if place not in places:
            places[place] = {}
            order.append(place)
        key = (fields["name"], fields.get("criterion") or fields.get("reading") or fields.get("readings") or fields.get("set") or "")
        places[place][key] = fields
    return places, order


def differences_at(domain, baseline_fields, mutant_fields):
    """一个 (域, N) 上变体与基线不同的量，分三栏：条款判据、实现锚点、别的。"""
    clause, anchors, other = [], [], []
    for key, baseline in baseline_fields.items():
        mutant = mutant_fields.get(key)
        name, discriminator = key
        if mutant is None:
            other.append(f"{name}.{discriminator or 'row'}.missing")
            continue
        for field, baseline_value in baseline.items():
            if mutant.get(field) == baseline_value:
                continue
            if name == "violations" and field == "violations":
                if discriminator in CLAUSE_CRITERIA:
                    clause.append(discriminator)
                elif discriminator in ANCHOR_CRITERIA:
                    anchors.append(discriminator)
                else:
                    other.append(f"violations.{discriminator}")
            elif name == "pairs" and field in ("today_j5a", "today_j5b"):
                clause.append(f"{'J5a' if field == 'today_j5a' else 'J5b'}[{discriminator}]")
            elif name == "states" and field == "failed" and domain in DOMAINS_WHERE_FAILED_STATES_ARE_PINNED:
                anchors.append("failed_states")
            elif name == "states" and field == "panicked":
                pass  # 与 J0 / A0 那一行是同一个数
            else:
                other.append(f"{name}.{discriminator + '.' if discriminator else ''}{field}")
    for key in mutant_fields:
        if key not in baseline_fields:
            name, discriminator = key
            if name == "violations" and discriminator in ANCHOR_CRITERIA:
                anchors.append(discriminator)
            else:
                other.append(f"{name}.{discriminator or 'row'}.extra")
    return clause, anchors, other


def compare_with_baseline(baseline_rows, mutant_rows):
    baseline_places, order = rows_by_place(baseline_rows)
    mutant_places, _ = rows_by_place(mutant_rows)
    domains = []
    for domain, _size in order:
        if domain not in domains:
            domains.append(domain)
    per_domain = {}
    for domain in domains:
        sizes = [size for (place_domain, size) in order if place_domain == domain]
        entry = {"upper_bound": max(sizes), "clause": None, "anchor": None, "other": None, "missing": None}
        for size in sizes:
            if (domain, size) not in mutant_places:
                entry["missing"] = size
                break
            clause, anchors, other = differences_at(domain, baseline_places[(domain, size)], mutant_places[(domain, size)])
            if clause and entry["clause"] is None:
                entry["clause"] = (size, clause)
            if anchors and entry["anchor"] is None:
                entry["anchor"] = (size, anchors)
            if other and entry["other"] is None:
                entry["other"] = (size, other)
            if entry["clause"] is not None:
                break  # 红了就不再往上看这个域
        per_domain[domain] = entry
    return domains, per_domain


def violation_lines_by_place(rows):
    """(域, N) → {判据名: 那一行的整行文字}，只收 name=violations 行；域与 N 的次序照行出现的先后。"""
    places, order = {}, []
    for line in rows:
        fields = parse_row(line)
        if fields.get("name") != "violations" or "domain" not in fields or "n" not in fields or "criterion" not in fields:
            continue
        place = (fields["domain"], int(fields["n"]))
        if place not in places:
            places[place] = {}
            order.append(place)
        places[place][fields["criterion"]] = line
    return places, order


def domains_in_order(order):
    domains = []
    for domain, _size in order:
        if domain not in domains:
            domains.append(domain)
    return domains


def first_red_by_criterion(baseline_rows, mutant_rows):
    """r2 9.4 第 4 条（SQ2）：J6 两行合看、J6Qn 两行合看，每个域各求最小的 N——那条判据任一行与基线同一 (域, N) 的同一行文字不同
    （变体缺那一行也算不同）；到上沿都相同记 None，基线在那个域上一行那条判据都没有记 "no_rows"（与「界内不红」分开）。整份比完，不早停。
    返回 {"J6": [(域, N、None 或 "no_rows")…], "J6Qn": […]}，域的次序照基线。弄坏开关 j6qnfromj6：J6Qn 取了 J6 那两行。"""
    baseline_places, order = violation_lines_by_place(baseline_rows)
    mutant_places, _ = violation_lines_by_place(mutant_rows)
    families = {"J6": J6_ROWS, "J6Qn": J6_ROWS if "j6qnfromj6" in BREAK else J6QN_ROWS}
    result = {}
    for family, criteria in families.items():
        per_domain = []
        for domain in domains_in_order(order):
            sizes = sorted(size for (place_domain, size) in order if place_domain == domain)
            first = None if any(criterion in baseline_places[(domain, size)] for size in sizes for criterion in criteria) else "no_rows"
            for size in sizes if first is None else []:
                baseline_lines = baseline_places[(domain, size)]
                mutant_lines = mutant_places.get((domain, size), {})
                if any(criterion in baseline_lines and mutant_lines.get(criterion) != baseline_lines[criterion] for criterion in criteria):
                    first = size
                    break
            per_domain.append((domain, first))
        result[family] = per_domain
    return result


def first_red_by_criterion_text(first_reds):
    return ";".join(f"{family}:" + ",".join(f"{domain}={'none' if size is None else size}" for domain, size in per_domain)
                    for family, per_domain in first_reds.items())


def first_text_red_criteria(baseline_rows, mutant_rows):
    """r2 9.4 第 4 条后半（SQ5）：每个域上任一条判据的 name=violations 行文字与基线不同的最小 N，与那个 N 上文字不同的全部判据名（基线的次序）。
    交 [(域, N, [判据名…])]，到上沿都相同的域不列。"""
    baseline_places, order = violation_lines_by_place(baseline_rows)
    mutant_places, _ = violation_lines_by_place(mutant_rows)
    found = []
    for domain in domains_in_order(order):
        for size in sorted(size for (place_domain, size) in order if place_domain == domain):
            baseline_lines = baseline_places[(domain, size)]
            mutant_lines = mutant_places.get((domain, size), {})
            differing = [criterion for criterion, line in baseline_lines.items() if mutant_lines.get(criterion) != line]
            differing += [criterion for criterion in mutant_lines if criterion not in baseline_lines]
            if differing:
                found.append((domain, size, differing))
                break
    return found


def check_the_profile_rows(baseline):
    """r2 9.4 第 6 条：基线每个有 J6 行的 (域, N) 都要有 J6Qn 两行；一个有 J6 行的 (域, N) 都没有、或有一处缺 J6Qn 行，就是装置没带 --profile r2。
    弄坏开关 noprofilecheck：不核。"""
    if "noprofilecheck" in BREAK:
        return
    places, order = violation_lines_by_place(baseline["rows"])
    with_j6 = [place for place in order if any(criterion in places[place] for criterion in J6_ROWS)]
    lacking = [place for place in with_j6 if any(criterion not in places[place] for criterion in J6QN_ROWS)]
    if lacking or not with_j6:
        first = f"{lacking[0][0]}:{lacking[0][1]}" if lacking else "（基线一行 J6 都没有）"
        raise Stop(EXIT_NOT_PROFILE_R2, f"  ✗ 装置没带 --profile r2：基线有 J6 行的 {len(with_j6)} 个 (域, N) 里 {len(lacking)} 个缺 J6Qn_ 行，第一个 {first}\n"
                                        "  → 怎么办：装置要认 --profile r2（r2 登记 5.1）、驱动调它时要带上；这一次变体一个都没编")


# ── 一个变体

def copy_the_tree(setup, destination):
    os.makedirs(destination)
    for entry in COPIED_ROOT_ENTRIES:
        source = os.path.join(setup.root, entry)
        if os.path.isdir(source):
            shutil.copytree(source, os.path.join(destination, entry), symlinks=True,
                            ignore=shutil.ignore_patterns("target"))
        elif os.path.exists(source):
            shutil.copy2(source, os.path.join(destination, entry))


def run_captured(command, environment, cwd=None, log_path=None):
    completed = subprocess.run(command, env=environment, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if log_path is not None:
        with open(log_path, "w", encoding="utf-8") as handle:
            handle.write(completed.stdout)
            handle.write("\n--- stderr ---\n")
            handle.write(completed.stderr)
    return completed


def prepare_and_build(setup, item, directory, threads, wrap_build_for_measure=False):
    """建副本、另存原件、改坏、编。返回 (二进制路径或 None, 说明, 编译挂钟秒)。"""
    if os.path.exists(directory):
        shutil.rmtree(directory)
    tree = os.path.join(directory, "tree")
    copy_the_tree(setup, tree)
    for path in SOURCE_FILES:
        if sha256_of_file(os.path.join(tree, path)) != setup.registered["files_sha256"][path]:
            raise Stop(EXIT_HALT_H1, f"  ✗ 停机条款 RH1：副本里的 {path} 的 sha256 不等于登记值\n  → 怎么办：仓里的源文件在跑的时候被改了；交回主 agent")
    target = target_of(item)
    original = os.path.join(directory, "source.original")
    if target is not None:
        shutil.copy2(os.path.join(tree, target), original)
    if item["old"] is not None:
        text = open(original, encoding="utf-8").read()
        if text.count(item["old"]) == 1:
            text = text.replace(item["old"], item["new"])
        elif "noanchorcheck" not in BREAK:
            raise Stop(EXIT_ANCHOR, f"  ✗ 锚点腐化：{item['name']} 的原文在副本里的 {target} 命中 {text.count(item['old'])} 次\n  → 怎么办：改那一行的原文")
        with open(os.path.join(tree, target), "w", encoding="utf-8") as handle:
            handle.write(text)
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = os.path.join(directory, "target")
    environment["CARGO_BUILD_JOBS"] = str(threads)
    command = ["nice", "-n", "19"] + setup.cargo + ["build", "--release", "--offline", "-p", "singlefs-checker-tier", "--bin", BIN_NAME]
    if wrap_build_for_measure:
        environment["RUN_WITH_MEMORY_CAP_KEY"] = "e165-kill-bounds measure build"
        environment["RUN_WITH_MEMORY_CAP_PARALLEL_REASON"] = "E165 第二段定并行度之前量一次单变体编译的内存峰值"
        command = ["bash", setup.memory_cap_runner, "8G"] + command
    started = time.monotonic()
    completed = run_captured(command, environment, cwd=tree, log_path=os.path.join(directory, "build.log"))
    elapsed = time.monotonic() - started
    binary = os.path.join(directory, "target", "release", BIN_NAME)
    if completed.returncode != 0 or not os.path.exists(binary):
        error_lines = [line for line in completed.stderr.split("\n") if line.startswith("error")]
        return None, (error_lines[0] if error_lines else f"cargo build 退出码 {completed.returncode}"), elapsed
    return binary, "", elapsed


def target_of(item):
    """这个变体改副本里的哪一份源文件：变异行第二列（基线不改）。弄坏开关 recoveryonly 只改 recovery.rs、不看第二列。"""
    if item.get("old") is None:
        return None
    return RECOVERY_PATH if "recoveryonly" in BREAK else item.get("file")


def restore_and_remove(setup, directory, keep, target=None):
    """被改的那一份原件拷回副本，两份源文件的 sha256 都要等于登记值；不留的删掉整份。"""
    tree = os.path.join(directory, "tree")
    original = os.path.join(directory, "source.original")
    restored = False
    if os.path.isdir(tree):
        if target is not None and os.path.exists(original):
            shutil.copy2(original, os.path.join(tree, target))
        restored = all(os.path.exists(os.path.join(tree, path)) and sha256_of_file(os.path.join(tree, path)) == setup.registered["files_sha256"][path]
                       for path in SOURCE_FILES)
    if not keep:
        shutil.rmtree(directory, ignore_errors=True)
    return restored


GPU_MEMORY_HIT_EXIT = 245   # run-with-memory-cap.sh：显存不够（命令退了非 0、起止之间内核报了 NV_ERR_NO_MEMORY），结果不算数


def wrapper_own_problem(returncode):
    """内存包装自己的结局（run-with-memory-cap.sh 文件头「退出码」）：这一次不算结果，交原因；装置自己的退出码交 None。
    245（显存不够）单独一支；弄坏开关 gpumemoryasresult 下当装置自己的退出码。"""
    if returncode == GPU_MEMORY_HIT_EXIT and "gpumemoryasresult" not in BREAK:
        return f"内存包装退出码 {returncode}：显存不够（起止之间内核报了 NV_ERR_NO_MEMORY），这一次不算结果"
    if 250 <= returncode <= 254:
        return f"内存包装退出码 {returncode}，这一次不算结果"
    return None


def run_segment_two(setup, binary, threads, log_directory):
    command, environment = setup.wrap("segment2 run", [binary] + setup.bounds_arguments())
    environment["E165_THREADS"] = str(threads)
    started = time.monotonic()
    completed = run_captured(["nice", "-n", "19"] + command, environment, log_path=os.path.join(log_directory, "run.log"))
    return completed, time.monotonic() - started


def state_lines(setup, binary, kind, place):
    command, environment = setup.wrap(kind, [binary, "--list-states" if kind == "list-states" else "--replay-state", place])
    return subprocess.Popen(["nice", "-n", "19"] + command, env=environment, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)


def counters_of(line):
    match = re.search(r" counters=(\S+)", line)
    return match.group(1) if match else ""


def first_differing_state(setup, baseline_binary, mutant_binary, domain, size):
    """两边的逐态行照枚举次序逐行比：先找第一个「加了一的量」不同的状态；一个都没有时退到第一个整行不同的状态（只有结局不同）。"""
    place = f"{domain}:{size}"
    baseline_process = state_lines(setup, baseline_binary, "list-states", place)
    mutant_process = state_lines(setup, mutant_binary, "list-states", place)
    found = None
    only_outcome = None
    for baseline_line, mutant_line in zip(baseline_process.stdout, mutant_process.stdout):
        if baseline_line == mutant_line or " name=state " not in baseline_line:
            continue
        ordinal = parse_row(baseline_line.rstrip("\n")).get("ordinal")
        if counters_of(baseline_line) != counters_of(mutant_line):
            found = ("counters", ordinal, baseline_line.rstrip("\n"), mutant_line.rstrip("\n"))
            break
        if only_outcome is None:
            only_outcome = ("outcome_only", ordinal, baseline_line.rstrip("\n"), mutant_line.rstrip("\n"))
    for process in (baseline_process, mutant_process):
        process.stdout.close()  # 还在写的那一边收到管道关闭就退出
        process.wait()
    return found or only_outcome


def replayed_state_line(setup, binary, domain, size, ordinal):
    process = state_lines(setup, binary, "replay-state", f"{domain}:{size}:{ordinal}")
    output = process.stdout.read()
    process.stdout.close()
    process.wait()
    lines = [line for line in output.split("\n") if " name=state " in line]
    return lines[0] if len(lines) == 1 else None


def counter_difference(baseline_line, mutant_line):
    baseline = set(counters_of(baseline_line).split(","))
    mutant = set(counters_of(mutant_line).split(","))
    parts = [f"+{name}" for name in sorted(mutant - baseline)] + [f"-{name}" for name in sorted(baseline - mutant)]
    return ",".join(parts) if parts else "none"


def rerun_command(item, options):
    return (f"bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh{options.get('pins_text', '')} --scratch <草稿目录> --output {options['output_shown']} "
            f"--ledger {options['ledger_shown']} --recheck --item {item['name']}")


def process_item(setup, item, baseline, options):
    """做一个变体，返回 {lines: 产物行, ledger: 红例账行, problem: 没得出结果的原因或 None, timing: …}。"""
    directory = os.path.join(options["work"], item["name"])
    started = time.monotonic()
    mark(options, item["name"], "建副本、改坏、编译")
    binary, build_problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"])

    def head(row):
        return (f"E7RESULT name=kill_bound item={item['name']} table={item['table']} table_row={row} "
                f"rows_sharing_this_replacement={'+'.join(str(shared) for shared in item['rows']) or 'none'}")

    def tail(title):
        return f"rerun=\"{rerun_command(item, options)}\" title={title.replace(' ', '_')} complete=yes"

    if binary is None:
        restored = restore_and_remove(setup, directory, keep=False, target=target_of(item))
        lines = [f"{head(row)} outcome=invalid build_error={build_problem.replace(' ', '_')} copy_restored={str(restored).lower()} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "timing": (build_seconds, 0.0, time.monotonic() - started)}
    mark(options, item["name"], "跑 --bounds segment2")
    completed, run_seconds = run_segment_two(setup, binary, options["threads"], directory)
    wrapper_problem = wrapper_own_problem(completed.returncode)
    if wrapper_problem:
        restore_and_remove(setup, directory, keep=False, target=target_of(item))
        return {"lines": [], "ledger": [], "problem": wrapper_problem,
                "timing": (build_seconds, run_seconds, time.monotonic() - started)}
    mutant_rows = result_rows(completed.stdout)
    domains, per_domain = compare_with_baseline(baseline["rows"], mutant_rows)
    finished = bool(mutant_rows) and mutant_rows[-1].startswith("E7RESULT name=done ")
    panic_lines = [line for line in completed.stderr.split("\n") if "panicked at" in line]
    clause_parts, anchor_parts, other_parts, not_red, missing_parts = [], [], [], [], []
    legal_minimum = adversarial_minimum = None
    ledger = []
    for domain in domains:
        entry = per_domain[domain]
        red_sizes = [found[0] for found in (entry["clause"], entry["anchor"]) if found is not None]
        if entry["clause"] is not None:
            clause_parts.append(f"{domain}:{entry['clause'][0]}:{'+'.join(entry['clause'][1])}")
        if entry["anchor"] is not None:
            anchor_parts.append(f"{domain}:{entry['anchor'][0]}:{'+'.join(entry['anchor'][1])}")
        if entry["other"] is not None and (not red_sizes or entry["other"][0] < min(red_sizes)):
            other_parts.append(f"{domain}:{entry['other'][0]}:{'+'.join(entry['other'][1])}")
        if entry["missing"] is not None:
            missing_parts.append(f"{domain}:{entry['missing']}")
        if not red_sizes:
            if entry["missing"] is None:
                not_red.append(f"{domain}(≤{entry['upper_bound']})")
            continue
        first_red = min(red_sizes)
        if domain in ADVERSARIAL_DOMAINS:
            adversarial_minimum = first_red if adversarial_minimum is None else min(adversarial_minimum, first_red)
        else:
            legal_minimum = first_red if legal_minimum is None else min(legal_minimum, first_red)
        # 红例：这个域首红的 N 上第一个判定不同的状态，两边各单条重放一遍。
        rows_text = "+".join(str(row) for row in item["rows"])
        mark(options, item["name"], f"在 {domain}:{first_red} 上找红例（两边 --list-states 逐行比）")
        example = first_differing_state(setup, baseline["binary"], binary, domain, first_red)
        if example is None:
            ledger.append({"domain": domain, "size": first_red, "found": False, "fields": [
                item["name"], rows_text, domain, str(first_red), "none", "no_single_state_differs", "none", "false", "false", "", "", ""]})
            continue
        kind, ordinal, baseline_line, mutant_line = example
        mark(options, item["name"], f"在 {domain}:{first_red} 上单条重放第 {ordinal} 个状态")
        on_original = replayed_state_line(setup, baseline["binary"], domain, first_red, ordinal)
        on_mutant = replayed_state_line(setup, binary, domain, first_red, ordinal)
        ledger.append({"domain": domain, "size": first_red, "found": True, "fields": [
            item["name"], rows_text, domain, str(first_red), ordinal, kind,
            counter_difference(baseline_line, mutant_line) if kind == "counters" else "outcome",
            str(on_original == baseline_line).lower(), str(on_mutant == mutant_line).lower(),
            f"target/release/{BIN_NAME} --replay-state {domain}:{first_red}:{ordinal}", baseline_line, mutant_line]})
    if clause_parts:
        outcome = "red"
    elif anchor_parts:
        outcome = "red_only_by_implementation_anchor"
    elif missing_parts or not finished:
        outcome = "run_did_not_finish"
    else:
        outcome = "not_red_within_bounds"
    found_entries = [entry for entry in ledger if entry["found"]]
    first_of_variant = min(found_entries, key=lambda entry: (entry["size"], domains.index(entry["domain"]))) if found_entries else None
    for entry in ledger:
        entry["fields"].insert(9, str(entry is first_of_variant).lower())
    confirmed = sum(1 for entry in ledger if entry["fields"][7] == "true" and entry["fields"][8] == "true")
    restored = restore_and_remove(setup, directory, keep=False, target=target_of(item))
    abort = panic_lines[0].strip().replace(" ", "_")[:200] if panic_lines and not finished else "none"
    text_reds = first_text_red_criteria(baseline["rows"], mutant_rows)
    text_red_text = ";".join(f"{domain}:{size}:{'+'.join(criteria)}" for domain, size, criteria in text_reds) or "none"
    body = (f"outcome={outcome} first_clause_red={';'.join(clause_parts) or 'none'} first_anchor_red={';'.join(anchor_parts) or 'none'} "
            f"first_red_by_criterion={first_red_by_criterion_text(first_red_by_criterion(baseline['rows'], mutant_rows))} "
            f"first_text_red_criteria={text_red_text} "
            f"other_difference_before_any_red={';'.join(other_parts) or 'none'} not_red_domains={','.join(not_red) or 'none'} "
            f"legal_family_minimum={legal_minimum if legal_minimum is not None else 'none'} "
            f"adversarial_family_minimum={adversarial_minimum if adversarial_minimum is not None else 'none'} "
            f"run_exit={completed.returncode} run_finished={str(finished).lower()} domains_without_rows={';'.join(missing_parts) or 'none'} "
            f"abort={abort} red_examples={len(ledger)} red_examples_confirmed_by_single_state_replay={confirmed} copy_restored={str(restored).lower()}")
    lines = [f"{head(row)} {body} {tail(title)}" for row, title in zip(item["rows"], item["titles"])]
    problem = None if finished or outcome.startswith("red") else f"装置没跑完（退出码 {completed.returncode}）而没有一个域红"
    return {"lines": lines, "ledger": ["\t".join(entry["fields"]) for entry in ledger], "problem": problem,
            "timing": (build_seconds, run_seconds, time.monotonic() - started)}


def baseline_item_line(item, baseline, options):
    rows = baseline["rows"]
    digest = hashlib.sha256("\n".join(rows).encode("utf-8")).hexdigest()
    return (f"E7RESULT name=kill_bound item=baseline table=none table_row=none rows_sharing_this_replacement=none outcome=baseline "
            f"result_rows={len(rows)} result_rows_sha256={digest} run_exit={baseline['exit']} "
            f"rows_found_verbatim_in_segment1_products={baseline['verbatim_rows']} rerun=\"{rerun_command(item, options)}\" title=基线（不改） complete=yes")


def build_and_run_baseline(setup, options, name):
    item = {"name": name, "old": None, "new": None}
    directory = os.path.join(options["work"], name)
    label = "开头基线" if name == "baseline" else "收尾基线"
    mark(options, label, "建副本、编译")
    binary, problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"] * options["jobs"])
    if binary is None:
        raise Stop(EXIT_INCOMPLETE, f"  ✗ 基线编不过：{problem}（日志 {directory}/build.log）\n  → 怎么办：先让仓里这个 bin 编得过")
    mark(options, label, "跑 " + " ".join(setup.bounds_arguments()))
    completed, run_seconds = run_segment_two(setup, binary, options["threads"] * options["jobs"], directory)
    if options.get("progress") is not None:
        options["progress"].finished(label, counted=False)
    rows = result_rows(completed.stdout)
    if completed.returncode != 0 or not rows or not rows[-1].startswith("E7RESULT name=done "):
        raise Stop(EXIT_INCOMPLETE, f"  ✗ 基线没跑完或退出码不是 0（{completed.returncode}；日志 {directory}/run.log）\n  → 怎么办：退出码 245（显存不够）与 250–254 是内存包装的结局，照它报的办（245 先 nvidia-smi 看谁在用卡、等它跑完）；别的先单跑基线看")
    print(f"  基线 {name}：编译 {build_seconds:.1f} 秒，跑 {run_seconds:.1f} 秒，{len(rows)} 行", file=sys.stderr)
    return {"binary": binary, "rows": rows, "exit": completed.returncode, "directory": directory}


def retained_product_not_run(setup, clause, paths):
    """比对的对象是 research/ 下的留存产物（可以整个删掉）：有不在的，这一项比对本次未跑——逐个点名不在的产物，记进 setup.not_run、打一行「本次未跑」，
    环境里有 GATE_NOT_RUN_FILE 的另往那份文件追加一行；不当绿、不判红，交 True（调用方不比、比对那一栏写 not_run）。都在交 False（照旧比）。
    弄坏开关 retainedmissingred 下一律交 False（照改前：不在就判红或读不到）。"""
    absent = [path for path in paths if not os.path.exists(path)]
    if not absent or "retainedmissingred" in BREAK:
        return False
    for path in absent:
        line = f"作废条款 {clause} 的比对：留存产物 {path} 不在"
        setup.not_run.append(line)
        print(f"  ! 本次未跑：{line}（留存产物可以整个删掉；在时照比，不在不当绿、不判红）")
        not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
        if not_run_file:
            with open(not_run_file, "a", encoding="utf-8") as handle:
                handle.write(f"e165-replay-mutation-kill-bounds：{line}\n")
    return True


def retained_comparison_text(baseline, compared_text):
    """基线那一行、收尾判决行里比对那一栏：留存产物不在、没比的写 not_run，比过的写 compared_text。"""
    return "not_run" if baseline.get("verbatim_rows") == "not_run" else compared_text


def check_against_segment_one(setup, baseline):
    """作废条款 V7：基线每个 (域, N) 的行必须逐字出现在第一段的产物里。第一段产物有不在的，这一项本次未跑（retained_product_not_run）。"""
    if retained_product_not_run(setup, "V7", [os.path.join(setup.research_root, product) for product in setup.registered["segment1_products"]]):
        baseline["verbatim_rows"] = "not_run"
        return
    known = set()
    for product in setup.registered["segment1_products"]:
        with open(os.path.join(setup.research_root, product), encoding="utf-8") as handle:
            known.update(line.rstrip("\n") for line in handle)
    place_rows = [row for row in baseline["rows"] if " domain=" in row and " n=" in row and " name=verdict " not in row]
    absent = [row for row in place_rows if row not in known]
    if absent or not place_rows:
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 V7：基线有 {len(absent)} 行（共 {len(place_rows)} 行）不在第一段的产物里，第一行：{absent[0][:200] if absent else '一行都没有'}\n"
                                 "  → 怎么办：第一段的产物与今天的 bin 对不上，先复跑第一段")
    baseline["verbatim_rows"] = len(place_rows)


# ── r3 档（登记 r3 9.3，第三段）：整份装置输出的读法。判据是违例数，格是 (域, M, U)，KA 的 M、U 都写 0。

def r3_parse(rows):
    """装置 --family root_choice 的 E7RESULT 行 → (违例, 计数行, 格的先后次序)。
    违例：{格: {判据: (违例数, 整行)}}（只收 name=violations）；计数行：{格: (字段, 整行)}（name=root_choice_states）；格 = (域, M, U)。"""
    violations, states, order = {}, {}, []
    for line in rows:
        fields = parse_row(line)
        if fields.get("name") not in ("violations", "root_choice_states") or not {"domain", "mounts", "user_publishes"} <= set(fields):
            continue
        cell = (fields["domain"], int(fields["mounts"]), int(fields["user_publishes"]))
        if cell not in order:
            order.append(cell)
        if fields["name"] == "violations":
            if "criterion" in fields:
                violations.setdefault(cell, {})[fields["criterion"]] = (int(fields["violations"]), line)
        else:
            states[cell] = (fields, line)
    return violations, states, order


def r3_cells_of(order, domain):
    """一个域的格，按 (M 升序, U 升序)。"""
    return sorted((mounts, publishes) for cell_domain, mounts, publishes in order if cell_domain == domain)


def r3_first_reds(baseline, mutant):
    """登记 TQ5：每个变体、每个合法历史域、每条判据，按 (M 升序, U 升序) 第一个违例数与基线不同的格（变体缺那一格或缺那一行也算不同）；
    到上沿都相同记 None；基线在那个域上一行那条判据都没有记 "no_rows"。整份比完，不早停。返回 {判据: [(域, (M, U) 或 None 或 "no_rows")…]}。
    弄坏开关 r3firstredswap：KJ1 与 KJ2 两列对调。"""
    baseline_violations, _states, baseline_order = baseline
    mutant_violations = mutant[0]
    result = {}
    for criterion in R3_CRITERIA:
        per_domain = []
        for domain in R3_LEGAL_DOMAINS:
            cells = r3_cells_of(baseline_order, domain)
            if not any(criterion in baseline_violations[(domain, m, u)] for m, u in cells):
                per_domain.append((domain, "no_rows"))
                continue
            first = None
            for m, u in cells:
                baseline_reading = baseline_violations[(domain, m, u)].get(criterion)
                mutant_reading = mutant_violations.get((domain, m, u), {}).get(criterion)
                if baseline_reading is not None and (mutant_reading is None or mutant_reading[0] != baseline_reading[0]):
                    first = (m, u)
                    break
            per_domain.append((domain, first))
        result[criterion] = per_domain
    if "r3firstredswap" in BREAK:
        result["KJ1"], result["KJ2"] = result["KJ2"], result["KJ1"]
    return result


def r3_first_red_text(first_reds):
    def one(value):
        return "none" if value is None else value if isinstance(value, str) else f"M{value[0]}U{value[1]}"
    return ";".join(f"{criterion}:" + ",".join(f"{domain}={one(value)}" for domain, value in per_domain) for criterion, per_domain in first_reds.items())


def r3_differing_cells_text(baseline, mutant):
    """每条判据、每个合法历史域：违例数与基线不同的格数（格数总数写在 of 后面）。"""
    baseline_violations, _states, baseline_order = baseline
    parts = []
    for criterion in R3_CRITERIA:
        per_domain = []
        for domain in R3_LEGAL_DOMAINS:
            cells = r3_cells_of(baseline_order, domain)
            differing = sum(1 for m, u in cells if criterion in baseline_violations[(domain, m, u)]
                            and mutant[0].get((domain, m, u), {}).get(criterion, (None,))[0] != baseline_violations[(domain, m, u)][criterion][0])
            per_domain.append(f"{domain}={differing}of{len(cells)}")
        parts.append(f"{criterion}:" + ",".join(per_domain))
    return ";".join(parts)


def r3_cells_text(violations, order):
    """每个格的 KJ0/KJ1/KJ2/KJ3 违例数（没有那一行写 -）：<域>.M<m>U<u>=a/b/c/d，格用逗号连（KA 的 M、U 是 0）。"""
    parts = []
    for domain in R3_ALL_DOMAINS:
        for m, u in r3_cells_of(order, domain):
            readings = violations.get((domain, m, u), {})
            parts.append(f"{domain}.M{m}U{u}=" + "/".join(str(readings[criterion][0]) if criterion in readings else "-" for criterion in R3_ROW_CRITERIA))
    return ",".join(parts) or "none"


def r3_ka_kj2(violations):
    """KA 上 KJ2 的违例数；KA 那一格或那一行不在交 None。"""
    for (domain, _m, _u), readings in violations.items():
        if domain == "KA" and "KJ2" in readings:
            return readings["KJ2"][0]
    return None


def r3_states_row_differences(baseline, mutant):
    """每个域：计数行与基线不同的第一个格，与那一格上不同的字段名；全相同的域不列。"""
    _violations, baseline_states, baseline_order = baseline
    mutant_states = mutant[1]
    found = []
    for domain in R3_ALL_DOMAINS:
        for m, u in r3_cells_of(baseline_order, domain):
            cell = (domain, m, u)
            if cell not in mutant_states:
                found.append(f"{domain}=M{m}U{u}:missing")
                break
            differing = sorted(key for key in baseline_states[cell][0] if mutant_states[cell][0].get(key) != baseline_states[cell][0][key])
            if differing:
                found.append(f"{domain}=M{m}U{u}:" + "+".join(differing))
                break
    return ";".join(found) or "none"


def r3_state_totals(states, field, how):
    """每个域上计数行某个字段的合计（how 是 sum 或 max）：K:a,K9:b,KW:c,KA:d。"""
    parts = []
    for domain in R3_ALL_DOMAINS:
        values = [int(fields.get(field, "0")) for (cell_domain, _m, _u), (fields, _line) in states.items() if cell_domain == domain]
        parts.append(f"{domain}:{(sum(values) if how == 'sum' else max(values, default=0))}")
    return ",".join(parts)


def r3_ka_pin(entry, ka_kj2):
    """TQ6：变体在 KA 上 KJ2 的违例数对着登记。entry = ("pinned", n) 或 ("inferred", n)。返回 (ka_expected, ka_pin, ka_inferred)；
    pinned 的等于钉值写 match、不等写 mismatch；inferred 的不钉（ka_pin=not_pinned），只在 ka_inferred 里报 match 或 mismatch。弄坏开关 r3kapinunchecked：不核。"""
    kind, expected = entry
    if "r3kapinunchecked" in BREAK:
        return expected, "unchecked", "none"
    if kind == "pinned":
        return expected, "match" if ka_kj2 == expected else "mismatch", "none"
    return expected, "not_pinned", "match" if ka_kj2 == expected else "mismatch"


def r3_normalised(rows):
    """TV7 比对用：E7RESULT 行去掉耗时字段（登记第六节「elapsed_seconds、E7TIMING 行除外」；E7TIMING 行本来不是 E7RESULT 行）。"""
    return [re.sub(r" elapsed_seconds=\S+", "", row) for row in rows]


def r4_parse(rows):
    """装置 --family root_choice_faults 的 E7RESULT 行 → (违例, 计数行, 格的先后次序)。
    违例：{格: {(被判时故障, 历史标签): {判据: (违例数, 整行)}}}（只收 name=violations；KA 的行是 r3 的写法、没有这两样，记成 ("none", "inside_the_promise")）；
    计数行：{格: (字段, 整行)}（name=root_choice_fault_states，KA 取 name=root_choice_states）；格 = (域, M, U)。"""
    violations, states, order = {}, {}, []
    for line in rows:
        fields = parse_row(line)
        if fields.get("name") not in ("violations", "root_choice_fault_states", "root_choice_states") or not {"domain", "mounts", "user_publishes"} <= set(fields):
            continue
        cell = (fields["domain"], int(fields["mounts"]), int(fields["user_publishes"]))
        if cell not in order:
            order.append(cell)
        if fields["name"] == "violations":
            if "criterion" in fields:
                subcell = (fields.get("judged_fault", "none"), fields.get("history", "inside_the_promise"))
                violations.setdefault(cell, {}).setdefault(subcell, {})[fields["criterion"]] = (int(fields["violations"]), line)
        else:
            states[cell] = (fields, line)
    return violations, states, order


def r4_cells_of(order, domain):
    """一个域的格，按 (M 升序, U 升序)。"""
    return sorted((mounts, publishes) for cell_domain, mounts, publishes in order if cell_domain == domain)


def r4_subcell_text(subcell):
    fault, label = subcell
    return f"{fault}/{'inside' if label == 'inside_the_promise' else 'outside'}"


def r4_first_reds(baseline, mutant):
    """登记 TQ9：每个变体、每个合法历史域、每条判据，按 (M 升序, U 升序, 被判时故障, 历史标签) 第一个违例数与基线不同的格（变体缺那一格或缺那一行也算不同）；
    到上沿都相同记 None；基线在那个域上一行那条判据都没有记 "no_rows"。整份比完，不早停。返回 {判据: [(域, (M, U, 子格) 或 None 或 "no_rows")…]}。
    弄坏开关 r4firstredswap：KJ1T 与 KJ2 两列对调。"""
    baseline_violations, _states, baseline_order = baseline
    mutant_violations = mutant[0]
    result = {}
    for criterion in R4_CRITERIA:
        per_domain = []
        for domain in R4_LEGAL_DOMAINS:
            cells = r4_cells_of(baseline_order, domain)
            if not any(criterion in readings for m, u in cells for readings in baseline_violations.get((domain, m, u), {}).values()):
                per_domain.append((domain, "no_rows"))
                continue
            first = None
            for m, u in cells:
                for subcell in R4_SUBCELLS:
                    baseline_reading = baseline_violations.get((domain, m, u), {}).get(subcell, {}).get(criterion)
                    mutant_reading = mutant_violations.get((domain, m, u), {}).get(subcell, {}).get(criterion)
                    if baseline_reading is not None and (mutant_reading is None or mutant_reading[0] != baseline_reading[0]):
                        first = (m, u, subcell)
                        break
                if first is not None:
                    break
            per_domain.append((domain, first))
        result[criterion] = per_domain
    if "r4firstredswap" in BREAK:
        result["KJ1T"], result["KJ2"] = result["KJ2"], result["KJ1T"]
    return result


def r4_first_red_text(first_reds):
    def one(value):
        if value is None:
            return "none"
        if isinstance(value, str):
            return value
        return f"M{value[0]}U{value[1]}/{r4_subcell_text(value[2])}"
    return ";".join(f"{criterion}:" + ",".join(f"{domain}={one(value)}" for domain, value in per_domain) for criterion, per_domain in first_reds.items())


def r4_differing_text(baseline, mutant, domains):
    """每条判据、给定的每个域：违例数与基线不同的 (格, 被判时故障, 历史标签) 子格数（子格总数写在 of 后面）。"""
    baseline_violations, _states, baseline_order = baseline
    parts = []
    for criterion in R4_CRITERIA:
        per_domain = []
        for domain in domains:
            total = differing = 0
            for m, u in r4_cells_of(baseline_order, domain):
                for subcell, readings in baseline_violations.get((domain, m, u), {}).items():
                    if criterion not in readings:
                        continue
                    total += 1
                    mutant_reading = mutant[0].get((domain, m, u), {}).get(subcell, {}).get(criterion)
                    differing += int(mutant_reading is None or mutant_reading[0] != readings[criterion][0])
            per_domain.append(f"{domain}={differing}of{total}")
        parts.append(f"{criterion}:" + ",".join(per_domain))
    return ";".join(parts)


def r4_cell_sums(violations, cell):
    """一个格上五条判据的违例数合计（各子格相加）：{判据: 合计}，那一格没有的判据不列。"""
    sums = {}
    for readings in violations.get(cell, {}).values():
        for criterion, (count, _line) in readings.items():
            sums[criterion] = sums.get(criterion, 0) + count
    return sums


def r4_cells_text(violations, order):
    """每个格的 KJ0/KJ1/KJ1T/KJ2/KJ3 违例数合计（没有那一行写 -）：<域>.M<m>U<u>=a/b/c/d/e，格用逗号连（KA 的 M、U 是 0）。"""
    parts = []
    for domain in R4_ALL_DOMAINS:
        for m, u in r4_cells_of(order, domain):
            sums = r4_cell_sums(violations, (domain, m, u))
            parts.append(f"{domain}.M{m}U{u}=" + "/".join(str(sums[criterion]) if criterion in sums else "-" for criterion in R4_ROW_CRITERIA))
    return ",".join(parts) or "none"


def r4_ka_kj2(violations):
    """KA 上 KJ2 的违例数；KA 那一格或那一行不在交 None。"""
    for (domain, _m, _u), subcells in violations.items():
        if domain == "KA":
            for readings in subcells.values():
                if "KJ2" in readings:
                    return readings["KJ2"][0]
    return None


def r4_states_row_differences(baseline, mutant):
    """每个域：计数行与基线不同的第一个格，与那一格上不同的字段名（见证文字不比）；全相同的域不列。"""
    _violations, baseline_states, baseline_order = baseline
    mutant_states = mutant[1]
    found = []
    for domain in R4_ALL_DOMAINS:
        for m, u in r4_cells_of(baseline_order, domain):
            cell = (domain, m, u)
            if cell not in mutant_states:
                found.append(f"{domain}=M{m}U{u}:missing")
                break
            differing = sorted(key for key in baseline_states[cell][0]
                               if not key.endswith("witness") and mutant_states[cell][0].get(key) != baseline_states[cell][0][key])
            if differing:
                found.append(f"{domain}=M{m}U{u}:" + "+".join(differing))
                break
    return ";".join(found) or "none"


def r4_state_totals(states, field, how):
    """每个域上计数行某个字段的合计（how 是 sum 或 max）：KF:a,KF3:b,KF9:c,KFW:d,KG:e,KA:f。"""
    parts = []
    for domain in R4_ALL_DOMAINS:
        values = [int(fields.get(field, "0")) for (cell_domain, _m, _u), (fields, _line) in states.items() if cell_domain == domain]
        parts.append(f"{domain}:{(sum(values) if how == 'sum' else max(values, default=0))}")
    return ",".join(parts)

def r4_ka_pin(entry, ka_kj2):
    """TQ10：变体在 KA 上 KJ2 的违例数对着登记。entry = ("pinned", n) 或 ("inferred", n)。返回 (ka_expected, ka_pin, ka_inferred)；
    pinned 的等于钉值写 match、不等写 mismatch；inferred 的不钉（ka_pin=not_pinned），只在 ka_inferred 里报 match 或 mismatch。弄坏开关 r4kapinunchecked 不存在：
    KA 钉值与 r3 同一组、r3 那一格已经证过它会红。"""
    kind, expected = entry
    if kind == "pinned":
        return expected, "match" if ka_kj2 == expected else "mismatch", "none"
    return expected, "not_pinned", "match" if ka_kj2 == expected else "mismatch"


def r4_kg_pin(setup, label, violations):
    """TQ10：变体在 KG 上每个 (M, U) 格的 KJ1、KJ1T、KJ2 违例数（各子格相加）对着登记第七节「本地择根序」表里它那一列（v1827 取 v1823 那一列）。
    返回 (kg_pin, 不符的格)；kg_pin 是 match 或 mismatch；弄坏开关 r4kgpinunchecked：不核（unchecked）。"""
    if "r4kgpinunchecked" in BREAK:
        return "unchecked", "none"
    column = setup.registered["kg_columns"][R4_KG_COLUMN_OF[label]]
    mismatched = []
    for key, expected in column.items():
        m, u = (int(part) for part in key.split(","))
        sums = r4_cell_sums(violations, ("KG", m, u))
        got = [sums.get(criterion) for criterion in R4_CLAUSE_CRITERIA]
        if got != list(expected):
            mismatched.append(f"M{m}U{u}:" + "/".join("-" if value is None else str(value) for value in got) + "!=" + "/".join(str(value) for value in expected))
    return ("mismatch" if mismatched else "match"), (";".join(mismatched) or "none")


def r4_baseline_kg_today_pin(setup, violations):
    """基线在 KG 上每格的读数对「今天的序」那一列（登记 5.6：本地理想恢复与今天的三步逐格同值）；只报，不停机。"""
    column = setup.registered["kg_columns"]["today"]
    for key, expected in column.items():
        m, u = (int(part) for part in key.split(","))
        sums = r4_cell_sums(violations, ("KG", m, u))
        if [sums.get(criterion) for criterion in R4_CLAUSE_CRITERIA] != list(expected):
            return "mismatch"
    return "match"


def r4_differs_on(baseline, mutant):
    """变体在合法域不红时，KG 与 KA 上有没有与基线不同的量：KG（三条条款判据任一子格违例数不同）、KA（KJ2）。"""
    baseline_violations = baseline[0]
    on_kg = any(
        (mutant[0].get(("KG", m, u), {}).get(subcell, {}).get(criterion) is None
         or mutant[0][("KG", m, u)][subcell][criterion][0] != reading[0])
        for (domain, m, u), subcells in baseline_violations.items() if domain == "KG"
        for subcell, readings in subcells.items() for criterion, reading in readings.items() if criterion in R4_CLAUSE_CRITERIA)
    on_ka = r4_ka_kj2(baseline_violations) != r4_ka_kj2(mutant[0])
    return "+".join(name for name, differs in (("KG", on_kg), ("KA", on_ka)) if differs) or "none", on_kg


def check_the_profile_rows_r4(baseline):
    """登记 r4 9.3 第 6 条第五格：基线要有五个合法域与 KG 各一条 criteria=root_choice_faults 的判决行、KA 一条 criteria=root_choice 的判决行；缺了就是装置没带
    --profile r4，退 7、变体一个不编。（不带 --profile r4 时装置自己退 2，走不到这里，这一格防的是装置不认这一族却照旧跑出别的行。）"""
    found = {parse_row(row).get("domain"): parse_row(row).get("criteria") for row in baseline["rows"] if " name=verdict " in row and " domain=" in row}
    lacking = [domain for domain in R4_ALL_DOMAINS if found.get(domain) != ("root_choice" if domain == "KA" else "root_choice_faults")]
    if lacking:
        raise Stop(EXIT_NOT_PROFILE_R2, f"  ✗ 装置没带 --profile r4：基线缺 criteria=root_choice_faults 的判决行，缺 {'、'.join(lacking)} 域\n"
                                        "  → 怎么办：装置要认 --profile r4 --family root_choice_faults（r4 登记 5.1）、驱动调它时要带上；这一次变体一个都没编")


def check_the_gate_rows_r4(setup, baseline):
    """登记 r5 修1.九乙第 4 条：给了 --gate r2 时，基线 family=root_choice_faults 的 name=config 行要带 R2 的闸字段（R4_GATE_R2_CONFIG_FIELD：witness_gate=r2_…）；缺了就是装置没收到
    或不认 --gate r2，退 7、变体一个不编。不给 --gate 时不判。"""
    if setup.gate != "r2":
        return
    field, value = R4_GATE_R2_CONFIG_FIELD
    configs = [parse_row(row) for row in baseline["rows"] if " name=config " in row and " family=root_choice_faults" in row]
    if not configs or any(config.get(field) != value for config in configs):
        raise Stop(EXIT_NOT_PROFILE_R2, f"  ✗ 装置没带 --gate r2：基线 family=root_choice_faults 的 name=config 行 {len(configs)} 行，缺 {field}={value}\n"
                                        f"  → 怎么办：装置要认 --profile r4 --family root_choice_faults --gate r2 并在 config 行写 {field}={value}（5-甲），驱动调它时要带上；这一次变体一个都没编")


def check_against_segment_one_product(setup, baseline):
    """作废条款 TV7（r4 9.3 第 3 条与第六节末段）：开头基线的 E7RESULT 行必须与第一段的产物逐行相同（同一份根、同一份源码编出来的）；
    一行都没有按 TV8 不当结果；产物不在，这一项本次未跑（retained_product_not_run）。"""
    path = os.path.join(setup.script_root, setup.registered["segment1_product"])
    if retained_product_not_run(setup, "TV7", [path]):
        baseline["verbatim_rows"] = "not_run"
        return
    if not os.path.exists(path):
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV8：第一段产物读不到：{path}\n  → 怎么办：产物在脚本所在的仓的 research/results/ 下；这一次不当结果")
    with open(path, encoding="utf-8") as handle:
        product = r3_normalised(result_rows(handle.read()))
    mine = r3_normalised(baseline["rows"])
    if not product or not mine:
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV8：比对读到 0 行（第一段产物 {len(product)} 行、基线 {len(mine)} 行）\n  → 怎么办：先看两份文件各自有没有 E7RESULT 行")
    if product != mine:
        first = next((index for index, (left, right) in enumerate(zip(product, mine)) if left != right), min(len(product), len(mine)))
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV7：开头基线 {len(mine)} 行与第一段产物 {len(product)} 行不逐行相同，第一处不同在第 {first + 1} 行\n"
                                 f"     产物：{(product[first] if first < len(product) else '（没有）')[:240]}\n     基线：{(mine[first] if first < len(mine) else '（没有）')[:240]}\n"
                                 "  → 怎么办：第一段的产物与今天根上编出的装置对不上；这一次不当结果，先查是谁换了装置或根")
    baseline["verbatim_rows"] = len(mine)


def load_items_r4(setup):
    """r4 的变体清单：基线、9.1 那六个名字按名字选出的行（按今天的表行号升序，起名 tsv-<行号>）。四条里有一条不满足的行不停机：
    记成 failure、那一行记 outcome=not_compared（登记 TH5），别的行照做。弄坏开关 r4itemsmissing：名单少最后一条。"""
    texts = {path: read_source(setup, path) for path in SOURCE_FILES}
    ranges = function_ranges(setup)
    selection = select_by_name(setup, ranges, texts)
    labels = dict(zip((row for row, _name, _pin in setup.registered["names"]), setup.registered["labels"]))
    if "r4itemsmissing" in BREAK:
        selection = selection[:-1]
    items = [{"name": "baseline", "table": "none", "rows": [], "titles": ["基线（不改）"], "old": None, "new": None, "file": None, "failure": None}]
    for entry in sorted(selection, key=lambda candidate: (candidate["today_row"] is None, candidate["today_row"] or 0, candidate["registered_row"])):
        name = f"tsv-{entry['today_row']}" if entry["today_row"] is not None else f"tsv-registered-{entry['registered_row']}"
        items.append({"name": name, "table": TABLE_PATH, "rows": [entry["today_row"] if entry["today_row"] is not None else entry["registered_row"]],
                      "titles": [entry["name"]], "file": entry["file"], "old": entry["old"], "new": entry["new"], "failure": entry["failure"],
                      "pin": list(entry["registered_tags"]), "registered_row": entry["registered_row"], "function": entry["function"],
                      "label": labels[entry["registered_row"]]})
    return items, texts


def baseline_item_line_r4(setup, item, baseline, options):
    rows = baseline["rows"]
    digest = hashlib.sha256("\n".join(rows).encode("utf-8")).hexdigest()
    kg_pin = r4_baseline_kg_today_pin(setup, r4_parse(rows)[0]) if rows else "no_rows"
    return (f"E7RESULT name=kill_bound item=baseline table=none table_row=none rows_sharing_this_replacement=none{options.get('gate_field', '')} outcome=baseline "
            f"result_rows={len(rows)} result_rows_sha256={digest} run_exit={baseline['exit']} "
            f"rows_identical_to_segment1_product={retained_comparison_text(baseline, str(baseline.get('verbatim_rows', 0) == len(rows)).lower())} kg_today_pin={kg_pin} "
            f"rerun=\"{rerun_command_r4(item, options)}\" title=基线（不改） complete=yes")


def rerun_command_r4(item, options):
    return (f"bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r4{options.get('pins_text', '')} --root <第一段根> --scratch <草稿目录> "
            f"--output {options['output_shown']} --item {item['name']}")

def process_item_r4(setup, item, baseline, options):
    """做一个变体，返回 {lines, ledger（恒空）, problem, timing}。整份 --bounds segment1 --family root_choice_faults 跑完，逐 (域, M, U, 被判时故障, 历史标签) 与基线比，不早停。
    给了 --gate r2 时每一行在 rows_sharing_this_replacement 之后多一个 gate=r2（登记 r5 修1.九乙第 3 条）。"""
    directory = os.path.join(options["work"], item["name"])
    started = time.monotonic()

    def head(row):
        return (f"E7RESULT name=kill_bound item={item['name']} table={item['table']} table_row={row} "
                f"rows_sharing_this_replacement={'+'.join(str(shared) for shared in item['rows']) or 'none'}{options.get('gate_field', '')}")

    def tail(title):
        return f"rerun=\"{rerun_command_r4(item, options)}\" title={title.replace(' ', '_')} complete=yes"

    if item["failure"] is not None:     # 登记 TH5：四条里有一条不满足，记没比、别的行照做
        lines = [f"{head(row)} outcome=not_compared failed_condition={item['failure']} registered_table_row={item['registered_row']} variant={item['label']} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "timing": (0.0, 0.0, 0.0)}
    mark(options, item["name"], "建副本、改坏、编译")
    binary, build_problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"])
    if binary is None:
        restored = restore_and_remove(setup, directory, keep=False, target=target_of(item))
        lines = [f"{head(row)} outcome=invalid build_error={build_problem.replace(' ', '_')} copy_restored={str(restored).lower()} variant={item['label']} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "timing": (build_seconds, 0.0, time.monotonic() - started)}
    mark(options, item["name"], "跑 " + " ".join(setup.bounds_arguments()))
    completed, run_seconds = run_segment_two(setup, binary, options["threads"], directory)
    wrapper_problem = wrapper_own_problem(completed.returncode)
    if wrapper_problem:
        restore_and_remove(setup, directory, keep=False, target=target_of(item))
        return {"lines": [], "ledger": [], "problem": wrapper_problem,
                "timing": (build_seconds, run_seconds, time.monotonic() - started)}
    mutant_rows = result_rows(completed.stdout)
    finished = bool(mutant_rows) and mutant_rows[-1].startswith("E7RESULT name=done ")
    panic_lines = [line for line in completed.stderr.split("\n") if "panicked at" in line]
    baseline_parsed, mutant_parsed = r4_parse(baseline["rows"]), r4_parse(mutant_rows)
    first_reds = r4_first_reds(baseline_parsed, mutant_parsed)
    red_domains = [domain for domain in R4_LEGAL_DOMAINS
                   if any(isinstance(value, tuple) for criterion in R4_CLAUSE_CRITERIA for found_domain, value in first_reds[criterion] if found_domain == domain)]
    differs_on, kg_differs = r4_differs_on(baseline_parsed, mutant_parsed)
    if "r4kgcountsasred" in BREAK and kg_differs:      # 弄坏开关：KG 当合法域
        red_domains = red_domains + ["KG"]
    other_red = any(isinstance(value, tuple) for criterion in ("KJ0", "KJ3") for _domain, value in first_reds[criterion])
    cells_without_rows = sum(1 for cell in baseline_parsed[2] if cell not in mutant_parsed[2])
    ka_kj2 = r4_ka_kj2(mutant_parsed[0])
    ka_expected, ka_pin, ka_inferred = r4_ka_pin(item["pin"], ka_kj2)
    kg_pin, kg_mismatched = r4_kg_pin(setup, item["label"], mutant_parsed[0])
    if red_domains:
        outcome = "red"
    elif other_red:
        outcome = "red_only_by_kj0_or_kj3"
    elif cells_without_rows or not finished:
        outcome = "run_did_not_finish"
    else:
        outcome = "not_red_within_bounds"
    restored = restore_and_remove(setup, directory, keep=False, target=target_of(item))
    abort = panic_lines[0].strip().replace(" ", "_")[:200] if panic_lines and not finished else "none"
    halts = sum(1 for row in mutant_rows if row.startswith("E7RESULT name=halt "))
    voids = sum(1 for row in mutant_rows if row.startswith("E7RESULT name=void "))
    body = (f"outcome={outcome} variant={item['label']} red_legal_domains={','.join(red_domains) or 'none'} "
            f"differs_on={differs_on if outcome == 'not_red_within_bounds' else 'not_applicable'} "
            f"first_red_by_criterion={r4_first_red_text(first_reds)} "
            f"differing_subcells={r4_differing_text(baseline_parsed, mutant_parsed, R4_LEGAL_DOMAINS)} "
            f"differing_subcells_kg={r4_differing_text(baseline_parsed, mutant_parsed, ('KG',))} cells={r4_cells_text(mutant_parsed[0], mutant_parsed[2])} "
            f"ka_kj2={'none' if ka_kj2 is None else ka_kj2} ka_expected={ka_expected} ka_pin={ka_pin} ka_inferred={ka_inferred} "
            f"kg_pin={kg_pin} kg_mismatched_cells={kg_mismatched} "
            f"states_row_differences={r4_states_row_differences(baseline_parsed, mutant_parsed)} "
            f"choose_root_returned_none_with_a_readable_root={r4_state_totals(mutant_parsed[1], 'choose_root_returned_none_with_a_readable_root', 'sum')} "
            f"replay_failed={r4_state_totals(mutant_parsed[1], 'replay_failed', 'sum')} panicked={r4_state_totals(mutant_parsed[1], 'panicked', 'sum')} "
            f"kj1_and_kj1t_differing_states={r4_state_totals(mutant_parsed[1], 'kj1_and_kj1t_differing_states', 'sum')} "
            f"halt_conditions={halts} void_conditions={voids} "
            f"run_exit={completed.returncode} run_finished={str(finished).lower()} cells_without_rows={cells_without_rows} "
            f"abort={abort} copy_restored={str(restored).lower()}")
    lines = [f"{head(row)} {body} {tail(title)}" for row, title in zip(item["rows"], item["titles"])]
    problem = None if finished or outcome == "red" else f"装置没跑完（退出码 {completed.returncode}）而没有一个域红"
    return {"lines": lines, "ledger": [], "problem": problem, "timing": (build_seconds, run_seconds, time.monotonic() - started)}


# ── 结果文件

def recorded_items(path):
    """结果文件里已经有完整一行的变体 → 它的行；半截的行（没有 complete=yes 收尾）不算。"""
    recorded = {}
    closed = False
    torn = False
    if not os.path.exists(path):
        return recorded, closed, torn
    content = open(path, encoding="utf-8").read()
    torn = bool(content) and not content.endswith("\n")
    for line in content.split("\n"):
        if line.startswith("E7RESULT name=done "):
            closed = True
        if line.startswith("E7RESULT name=kill_bound ") and line.endswith(" complete=yes"):
            recorded.setdefault(parse_row(line)["item"], []).append(line)
    return recorded, closed, torn


def append_lines(path, lines, torn):
    with open(path, "a", encoding="utf-8") as handle:
        if torn:
            handle.write("\n")
        for line in lines:
            handle.write(line + "\n")
        handle.flush()
        os.fsync(handle.fileno())


LEDGER_HEADER = ("# E165 第二段红例账：每个红的变体在每个红了的域首红的 N 上一个反例状态。制表符分隔，十三列：变体、表行、域、N、状态在枚举次序里的序号、"
                 "怎么不同（counters=加了一的量不同 / outcome_only=只有结局不同）、不同的量（+ 是变体多加的、- 是变体少加的）、"
                 "原实现单条重放与逐态行相同、变体单条重放与逐态行相同、是不是这个变体首红的那一个、单条重放的命令、原实现的逐态行、变体的逐态行")


def check_the_profile_rows_r3(baseline):
    """登记 r3 9.3 第 6 条第四格：基线要有每个域一条 criteria=root_choice 的判决行；缺了就是装置没带 --profile r3，退 7、变体一个不编。
    （r3 档才有 --family root_choice；不带 --profile r3 时装置自己退 2，走不到这里，这一格防的是装置不认这一族却照旧跑出别的行。）"""
    found = {parse_row(row).get("domain") for row in baseline["rows"] if " name=verdict " in row and " criteria=root_choice" in row and " domain=" in row}
    lacking = [domain for domain in R3_ALL_DOMAINS if domain not in found]
    if lacking:
        raise Stop(EXIT_NOT_PROFILE_R2, f"  ✗ 装置没带 --profile r3：基线缺 criteria=root_choice 的判决行，缺 {'、'.join(lacking)} 域\n"
                                        "  → 怎么办：装置要认 --profile r3 --family root_choice（r3 登记 5.1）、驱动调它时要带上；这一次变体一个都没编")


def check_against_segment_two_product(setup, baseline):
    """作废条款 TV7（r3 9.3 第 3 条与第六节末段）：开头基线的 E7RESULT 行必须与第二段产物逐行相同（同一份根、同一份源码编出来的，耗时字段除外）；
    一行都没有按 TV8 不当结果；产物不在，这一项本次未跑（retained_product_not_run）。"""
    path = os.path.join(setup.script_root, setup.registered["segment2_product"])
    if retained_product_not_run(setup, "TV7", [path]):
        baseline["verbatim_rows"] = "not_run"
        return
    if not os.path.exists(path):
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV8：第二段产物读不到：{path}\n  → 怎么办：产物在脚本所在的仓的 research/results/ 下；这一次不当结果")
    with open(path, encoding="utf-8") as handle:
        product = r3_normalised(result_rows(handle.read()))
    mine = r3_normalised(baseline["rows"])
    if not product or not mine:
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV8：比对读到 0 行（第二段产物 {len(product)} 行、基线 {len(mine)} 行）\n  → 怎么办：先看两份文件各自有没有 E7RESULT 行")
    if product != mine:
        first = next((index for index, (left, right) in enumerate(zip(product, mine)) if left != right), min(len(product), len(mine)))
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV7：开头基线 {len(mine)} 行与第二段产物 {len(product)} 行不逐行相同，第一处不同在第 {first + 1} 行\n"
                                 f"     产物：{(product[first] if first < len(product) else '（没有）')[:240]}\n     基线：{(mine[first] if first < len(mine) else '（没有）')[:240]}\n"
                                 "  → 怎么办：第二段的产物与今天冻结根上编出的装置对不上；这一次不当结果，先查是谁换了装置或根")
    baseline["verbatim_rows"] = len(mine)


def load_items_r3(setup):
    """r3 的变体清单：基线、9.1 那六个名字按名字选出的行（按 tsv 行号升序，起名 tsv-<行号>）。四条里有一条不满足的行不停机：
    记成 failure、那一行记 outcome=not_compared（登记 TH5），别的行照做。弄坏开关 r3itemsmissing：名单少最后一条。"""
    texts = {path: read_source(setup, path) for path in SOURCE_FILES}
    ranges = function_ranges(setup)
    selection = select_by_name(setup, ranges, texts)
    if "r3itemsmissing" in BREAK:
        selection = selection[:-1]
    items = [{"name": "baseline", "table": "none", "rows": [], "titles": ["基线（不改）"], "old": None, "new": None, "file": None, "failure": None}]
    for entry in sorted(selection, key=lambda candidate: (candidate["today_row"] is None, candidate["today_row"] or 0, candidate["registered_row"])):
        name = f"tsv-{entry['today_row']}" if entry["today_row"] is not None else f"tsv-registered-{entry['registered_row']}"
        items.append({"name": name, "table": TABLE_PATH, "rows": [entry["today_row"] if entry["today_row"] is not None else entry["registered_row"]],
                      "titles": [entry["name"]], "file": entry["file"], "old": entry["old"], "new": entry["new"], "failure": entry["failure"],
                      "pin": list(entry["registered_tags"]), "registered_row": entry["registered_row"], "function": entry["function"]})
    return items, texts


def baseline_item_line_r3(item, baseline, options):
    rows = baseline["rows"]
    digest = hashlib.sha256("\n".join(rows).encode("utf-8")).hexdigest()
    return (f"E7RESULT name=kill_bound item=baseline table=none table_row=none rows_sharing_this_replacement=none outcome=baseline "
            f"result_rows={len(rows)} result_rows_sha256={digest} run_exit={baseline['exit']} "
            f"rows_identical_to_segment2_product={retained_comparison_text(baseline, str(baseline.get('verbatim_rows', 0) == len(rows)).lower())} rerun=\"{rerun_command_r3(item, options)}\" title=基线（不改） complete=yes")


def rerun_command_r3(item, options):
    return (f"bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r3{options.get('pins_text', '')} --root {'<r5 第一段根>' if options.get('pins_text') else '<第二段冻结根>'} --scratch <草稿目录> "
            f"--output {options['output_shown']} --item {item['name']}")


def process_item_r3(setup, item, baseline, options):
    """做一个变体，返回 {lines, ledger（恒空）, problem, timing}。整份 --bounds segment1 --family root_choice 跑完，逐 (域, M, U) 与基线比，不早停。"""
    directory = os.path.join(options["work"], item["name"])
    started = time.monotonic()

    def head(row):
        return (f"E7RESULT name=kill_bound item={item['name']} table={item['table']} table_row={row} "
                f"rows_sharing_this_replacement={'+'.join(str(shared) for shared in item['rows']) or 'none'}")

    def tail(title):
        return f"rerun=\"{rerun_command_r3(item, options)}\" title={title.replace(' ', '_')} complete=yes"

    if item["failure"] is not None:     # 登记 TH5：四条里有一条不满足，记没比、别的行照做
        lines = [f"{head(row)} outcome=not_compared failed_condition={item['failure']} registered_table_row={item['registered_row']} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "timing": (0.0, 0.0, 0.0)}
    mark(options, item["name"], "建副本、改坏、编译")
    binary, build_problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"])
    if binary is None:
        restored = restore_and_remove(setup, directory, keep=False, target=target_of(item))
        lines = [f"{head(row)} outcome=invalid build_error={build_problem.replace(' ', '_')} copy_restored={str(restored).lower()} {tail(title)}"
                 for row, title in zip(item["rows"], item["titles"])]
        return {"lines": lines, "ledger": [], "problem": None, "timing": (build_seconds, 0.0, time.monotonic() - started)}
    mark(options, item["name"], "跑 " + " ".join(setup.bounds_arguments()))
    completed, run_seconds = run_segment_two(setup, binary, options["threads"], directory)
    wrapper_problem = wrapper_own_problem(completed.returncode)
    if wrapper_problem:
        restore_and_remove(setup, directory, keep=False, target=target_of(item))
        return {"lines": [], "ledger": [], "problem": wrapper_problem,
                "timing": (build_seconds, run_seconds, time.monotonic() - started)}
    mutant_rows = result_rows(completed.stdout)
    finished = bool(mutant_rows) and mutant_rows[-1].startswith("E7RESULT name=done ")
    panic_lines = [line for line in completed.stderr.split("\n") if "panicked at" in line]
    baseline_parsed, mutant_parsed = r3_parse(baseline["rows"]), r3_parse(mutant_rows)
    first_reds = r3_first_reds(baseline_parsed, mutant_parsed)
    red_domains = [domain for domain in R3_LEGAL_DOMAINS
                   if any(isinstance(value, tuple) for criterion in ("KJ1", "KJ2") for found_domain, value in first_reds[criterion] if found_domain == domain)]
    other_red = any(isinstance(value, tuple) for criterion in ("KJ0", "KJ3") for _domain, value in first_reds[criterion])
    cells_without_rows = sum(1 for cell in baseline_parsed[2] if cell not in mutant_parsed[2])
    ka_kj2 = r3_ka_kj2(mutant_parsed[0])
    ka_expected, ka_pin, ka_inferred = r3_ka_pin(item["pin"], ka_kj2)
    if red_domains:
        outcome = "red"
    elif other_red:
        outcome = "red_only_by_kj0_or_kj3"
    elif cells_without_rows or not finished:
        outcome = "run_did_not_finish"
    else:
        outcome = "not_red_within_bounds"
    restored = restore_and_remove(setup, directory, keep=False, target=target_of(item))
    abort = panic_lines[0].strip().replace(" ", "_")[:200] if panic_lines and not finished else "none"
    halts = sum(1 for row in mutant_rows if row.startswith("E7RESULT name=halt "))
    voids = sum(1 for row in mutant_rows if row.startswith("E7RESULT name=void "))
    body = (f"outcome={outcome} red_legal_domains={','.join(red_domains) or 'none'} first_red_by_criterion={r3_first_red_text(first_reds)} "
            f"differing_cells={r3_differing_cells_text(baseline_parsed, mutant_parsed)} cells={r3_cells_text(mutant_parsed[0], mutant_parsed[2])} "
            f"ka_kj2={'none' if ka_kj2 is None else ka_kj2} ka_expected={ka_expected} ka_pin={ka_pin} ka_inferred={ka_inferred} "
            f"states_row_differences={r3_states_row_differences(baseline_parsed, mutant_parsed)} "
            f"above_chosen_root_max={r3_state_totals(mutant_parsed[1], 'above_chosen_root_max', 'max')} "
            f"above_chosen_root_over_limit={r3_state_totals(mutant_parsed[1], 'above_chosen_root_over_limit', 'sum')} "
            f"choose_root_returned_none={r3_state_totals(mutant_parsed[1], 'choose_root_returned_none', 'sum')} "
            f"replay_failed={r3_state_totals(mutant_parsed[1], 'replay_failed', 'sum')} halt_conditions={halts} void_conditions={voids} "
            f"run_exit={completed.returncode} run_finished={str(finished).lower()} cells_without_rows={cells_without_rows} "
            f"abort={abort} copy_restored={str(restored).lower()}")
    lines = [f"{head(row)} {body} {tail(title)}" for row, title in zip(item["rows"], item["titles"])]
    problem = None if finished or outcome == "red" else f"装置没跑完（退出码 {completed.returncode}）而没有一个域红"
    return {"lines": lines, "ledger": [], "problem": problem, "timing": (build_seconds, run_seconds, time.monotonic() - started)}


def not_run_suffix(setup):
    """收尾那句后面接的：这一趟本次未跑的比对逐项列出（留存产物不在）；没有交空串。"""
    return f"；本次未跑 {len(setup.not_run)} 项：{'；'.join(setup.not_run)}" if setup.not_run else ""


PIN_LINE_LIMIT = 40   # 钉红例那一步没过时，原样转出它的输出最多几行


def pin_red_examples(setup, options):
    """钉红例（文件头「钉红例」）：这一对账（options["ledger"]）与 kill-bounds 产物（options["output"]）写完之后当场钉进钉表，再核钉全：
    先照 e165-pin-red-examples.py 的追加用法（--kill-bounds <产物> <账>），再 --check 显式给这一对；钉表与变异表取脚本所在的仓（setup.script_root）。
    有一步退非 0 就 raise Stop(EXIT_PIN_INCOMPLETE)，原样转出它的输出；都过交成功那句要报的「钉进 N 行、核了 M 行」。
    只供自证：E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND 设了（且是自证起的那一趟）就不追加，造「账写完了、钉表没钉」。
    弄坏开关 pincheckskipped 下不跑 --check（当场不核钉全）。"""
    ledger, product = options["ledger"], options["output"]
    base = [sys.executable, PIN_SCRIPT, "--root", setup.script_root]
    steps = []
    if not (setup.selftest and os.environ.get("E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND")):
        steps.append(("追加", base + ["--kill-bounds", product, ledger]))
    if "pincheckskipped" not in BREAK:
        steps.append(("核钉全（--check）", base + ["--check", "--kill-bounds", product, ledger]))
    said = {}
    for label, command in steps:
        environment = dict(os.environ)
        environment.pop("E165_PIN_RED_EXAMPLES_BREAK", None)   # 那一份的弄坏开关只给它自己的自证用，不从这里带进去
        completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=environment)
        lines = [line for line in completed.stdout.split("\n") if line.strip()]
        if completed.returncode != 0:
            shown = "\n".join("     " + line for line in lines[-PIN_LINE_LIMIT:])
            raise Stop(EXIT_PIN_INCOMPLETE, f"  ✗ 钉红例：{label}没过（{' '.join(command[1:])} 退 {completed.returncode}），这一趟的红例没钉全：\n{shown}\n"
                                            "  → 怎么办：照上面它打的出路办（同档冲突、按名字找不到变异、配对不上，写法在 scripts/experiment-apparatus/e165-pin-red-examples.py 文件头），"
                                            "改好之后重跑同一条命令：产物已收尾的只重做钉红例这一步，不重跑变体")
        said[label] = next((line for line in reversed(lines) if line.lstrip().startswith("✓")), lines[-1] if lines else "")
    appended = said.get("追加", "")
    pinned_match = re.search(r"钉进 (\d+) 行", appended)
    pinned = pinned_match.group(1) if pinned_match else ("0" if "没有要新钉的" in appended else "不追加（自证）" if not appended else "取不出")
    checked_line = said.get("核钉全（--check）")
    checked_match = re.search(r"份红例账 (\d+) 行都钉在", checked_line or "")
    checked = checked_match.group(1) if checked_match else ("没核（弄坏开关 pincheckskipped）" if checked_line is None else "取不出")
    return f"钉红例：钉进 {pinned} 行、核了 {checked} 行（账 {options.get('ledger_shown', ledger)} 配产物 {options.get('output_shown', product)}）"


def main_run(setup, options, item_names, recheck):
    check_the_registered_inputs(setup)
    items, selected_rows, texts = load_items(setup)
    check_the_anchors(items, texts)
    check_the_custom_keys(setup, items)
    by_name = {item["name"]: item for item in items}
    for name in item_names:
        if name not in by_name:
            raise Stop(EXIT_USAGE, f"  ✗ 不认识的变体：{name}\n  → 怎么办：--list-items 看全部变体的名字")
    wanted = [by_name[name] for name in item_names] if item_names else items
    recorded, closed, torn = recorded_items(options["output"])
    if "rerunall" in BREAK:
        recorded = {}
    records_directory = os.path.join(options["work"], "records")
    os.makedirs(records_directory, exist_ok=True)

    def stored_record(item):
        path = os.path.join(records_directory, item["name"] + ".json")
        if os.path.exists(path) and "rerunall" not in BREAK:
            with open(path, encoding="utf-8") as handle:
                return json.load(handle)
        return None

    if closed and not recheck:
        # 已收尾的一对照样当场钉、当场核（上一趟钉红例没过、改好之后重跑同一条命令走这里；已钉的不重复）
        pin_text = pin_red_examples(setup, options)
        print(f"  {options['output']} 已经收尾（name=done），{len(recorded)} 个变体都有行，这一次不跑变体；{pin_text}")
        return EXIT_OK
    to_run = [item for item in wanted if recheck or (item["name"] not in recorded and stored_record(item) is None)]
    status_at_start = subprocess.run(["git", "-C", setup.root, "status", "--porcelain", "crates/"], stdout=subprocess.PIPE, text=True).stdout
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；已有行 {sum(1 for item in wanted if item['name'] in recorded)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()   # 守护线程：收尾基线跑完就停；中途 Stop 退出的，进程退出时随之结束
    baseline = build_and_run_baseline(setup, options, "baseline")
    check_the_profile_rows(baseline)
    check_against_segment_one(setup, baseline)
    baseline_line = baseline_item_line(by_name["baseline"], baseline, options)
    if "baseline" in recorded and recorded["baseline"][0] != baseline_line:
        raise Stop(EXIT_VOID_V6, f"  ✗ 这一次的基线与结果文件里记的基线不同\n     记的：{recorded['baseline'][0][:240]}\n     这次：{baseline_line[:240]}\n"
                                 "  → 怎么办：bin 或 recovery.rs 变过；已有的行不能与新的行混在一份产物里，换一个 --output 重跑全部")

    results = {}
    problems = []
    lock = threading.Lock()
    flushed = set(recorded)
    if not recheck:
        if not os.path.exists(options["output"]):
            append_lines(options["output"], [
                f"E7RESULT name=config experiment=E165 segment=2 registration=research/prompts/e165-r2-prereg.md "
                f"device_arguments={'+'.join([] if 'noprofile' in BREAK else DEVICE_PROFILE_ARGUMENTS) or 'none'} "
                f"bounds=segment2 items={len(items)} table_rows={len(selected_rows)} "
                f"custom_rows={sum(1 for item in items if item['table'] == CUSTOM_TABLE_PATH)} recovery_sha256={sha_at_start[RECOVERY_PATH]} "
                f"lib_sha256={sha_at_start[LIB_PATH]} widening=none{pins_config_text(setup)}"], False)
        if not os.path.exists(options["ledger"]):
            append_lines(options["ledger"], [LEDGER_HEADER], False)

    def flush_in_order():
        """照变体的次序把已经有结果的落进产物：点名的里面前面还有没做完的就等它。"""
        nonlocal torn
        for item in items:
            name = item["name"]
            if name in flushed:
                continue
            record = results.get(name) or stored_record(item)
            if record is None or record.get("problem"):
                if item in wanted:
                    return
                continue
            append_lines(options["output"], record["lines"], torn)
            torn = False
            if record["ledger"]:
                append_lines(options["ledger"], record["ledger"], False)
            flushed.add(name)

    def work(item):
        if item["name"] == "baseline":
            record = {"lines": [baseline_line], "ledger": [], "problem": None, "timing": (0.0, 0.0, 0.0)}
        else:
            record = process_item(setup, item, baseline, options)
        with lock:
            results[item["name"]] = record
            options["progress"].finished(item["name"])
            build_seconds, run_seconds, total_seconds = record["timing"]
            print(f"  {item['name']}：编译 {build_seconds:.1f} 秒，跑 {run_seconds:.1f} 秒，合计 {total_seconds:.1f} 秒"
                  + (f"；没得出结果：{record['problem']}" if record["problem"] else ""), file=sys.stderr)
            with open(os.path.join(options["work"], "progress.log"), "a", encoding="utf-8") as handle:
                handle.write(f"{item['name']}\tbuild={build_seconds:.1f}\trun={run_seconds:.1f}\ttotal={total_seconds:.1f}\tproblem={record['problem']}\n")
            if record["problem"]:
                problems.append(f"{item['name']}：{record['problem']}")
            elif not recheck:
                with open(os.path.join(records_directory, item["name"] + ".json"), "w", encoding="utf-8") as handle:
                    json.dump(record, handle, ensure_ascii=False)
                flush_in_order()

    started = time.monotonic()
    halted = None
    with concurrent.futures.ThreadPoolExecutor(max_workers=options["jobs"]) as pool:
        futures = [pool.submit(work, item) for item in to_run]
        for future in concurrent.futures.as_completed(futures):
            try:
                future.result()
            except Stop as stop:
                halted = halted or stop
    if halted is not None:
        raise halted
    if not recheck:
        with lock:
            flush_in_order()
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；仓里的 crates/ 没变。
    baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
    options["progress"].stop()
    same_baseline = baseline_at_end["rows"] == baseline["rows"]
    restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
    restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    status_at_end = subprocess.run(["git", "-C", setup.root, "status", "--porcelain", "crates/"], stdout=subprocess.PIPE, text=True).stdout
    sha_at_end = source_hashes(setup)
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if (not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"]
            or status_at_start != status_at_end or not (restored_start and restored_end)):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 V6：收尾基线与开头逐字相同={same_baseline}；仓里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… 结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；"
                                 f"`git status --porcelain crates/` 前后相同={status_at_start == status_at_end}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了 crates/，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同")

    if recheck:
        different = 0
        for item in wanted:
            record = results.get(item["name"])
            fresh = record["lines"] if record else []
            same = fresh == recorded.get(item["name"], [])
            different += 0 if same else 1
            print(f"  {'✓' if same else '✗'} {item['name']}：新跑的 {len(fresh)} 行与产物里的 {len(recorded.get(item['name'], []))} 行{'逐字相同' if same else '不同'}")
            if not same:
                for line in fresh:
                    print(f"     新：{line}")
        if different:
            print("  → 怎么办：bin、recovery.rs 或变异表变过；看上面新跑的行与产物里那一行差在哪")
        return EXIT_INCOMPLETE if different or problems else EXIT_OK

    recorded, closed, _torn = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，再重跑同一条命令")
    if missing:
        print(f"  还有 {len(missing)} 个变体没有行：{' '.join(missing)}；重跑同一条命令（不带 --item）只补这些")
        return EXIT_INCOMPLETE if problems else EXIT_OK
    outcomes = {}
    for item in items:
        if item["name"] != "baseline":
            outcome = parse_row(recorded[item["name"]][0])["outcome"]
            outcomes[outcome] = outcomes.get(outcome, 0) + 1
    closing = [
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={len(baseline['rows'])} "
        f"repository_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} repository_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"repository_lib_sha256_at_start={sha_at_start[LIB_PATH]} repository_lib_sha256_at_end={sha_at_end[LIB_PATH]} crates_git_status_unchanged=true copies_restored=true",
        f"E7RESULT name=verdict scope=segment2 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_implementation_anchor={outcomes.get('red_only_by_implementation_anchor', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} ledger_sha256={sha256_of_file(options['ledger'])}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"], False)
    pin_text = pin_red_examples(setup, options)
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}；{pin_text}{not_run_suffix(setup)}")
    return EXIT_OK


def main_run_r3(setup, options, item_names):
    """r3 模式整轮（登记 r3 9.3）。做法同 main_run：开工核对、开头基线、每个变体一份副本、收尾基线；差在：基线要带 criteria=root_choice 判决行（退 7）、
    要与第二段产物逐行相同（退 6）、不落红例账、选行不满足的记 not_compared、KA 钉值不符整轮退 1（TF8）。"""
    check_the_registered_inputs(setup)
    items, texts = load_items_r3(setup)
    check_the_anchors([item for item in items if item["failure"] is None], texts)
    by_name = {item["name"]: item for item in items}
    for name in item_names:
        if name not in by_name:
            raise Stop(EXIT_USAGE, f"  ✗ 不认识的变体：{name}\n  → 怎么办：--profile r3 --list-items 看全部变体的名字")
    wanted = [by_name[name] for name in item_names] if item_names else items
    recorded, closed, torn = recorded_items(options["output"])
    if "rerunall" in BREAK:
        recorded = {}
    records_directory = os.path.join(options["work"], "records")
    os.makedirs(records_directory, exist_ok=True)

    def stored_record(item):
        path = os.path.join(records_directory, item["name"] + ".json")
        if os.path.exists(path) and "rerunall" not in BREAK:
            with open(path, encoding="utf-8") as handle:
                return json.load(handle)
        return None

    if closed:
        print(f"  {options['output']} 已经收尾（name=done），{len(recorded)} 个变体都有行，这一次什么都不跑")
        return EXIT_OK
    to_run = [item for item in wanted if item["name"] not in recorded and stored_record(item) is None]
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；已有行 {sum(1 for item in wanted if item['name'] in recorded)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()
    baseline = build_and_run_baseline(setup, options, "baseline")
    check_the_profile_rows_r3(baseline)
    check_against_segment_two_product(setup, baseline)
    baseline_line = baseline_item_line_r3(by_name["baseline"], baseline, options)
    if "baseline" in recorded and recorded["baseline"][0] != baseline_line:
        raise Stop(EXIT_VOID_V6, f"  ✗ 这一次的基线与结果文件里记的基线不同\n     记的：{recorded['baseline'][0][:240]}\n     这次：{baseline_line[:240]}\n"
                                 "  → 怎么办：bin 或根变过；已有的行不能与新的行混在一份产物里，换一个 --output 重跑全部")

    results = {}
    problems = []
    lock = threading.Lock()
    flushed = set(recorded)
    if not os.path.exists(options["output"]):
        append_lines(options["output"], [
            f"E7RESULT name=config experiment=E165 segment=3 profile=r3 registration=research/prompts/e165-r3-prereg.md "
            f"device_arguments={'+'.join(([] if 'r3noprofile' in BREAK else R3_DEVICE_PROFILE_ARGUMENTS) + R3_BOUNDS_ARGUMENTS) or 'none'} "
            f"items={len(items)} table_rows={len(items) - 1} recovery_sha256={sha_at_start[RECOVERY_PATH]} lib_sha256={sha_at_start[LIB_PATH]} "
            f"table_sha256={sha256_of_file(os.path.join(setup.root, TABLE_PATH))} widening=none{pins_config_text(setup)}"], False)

    def flush_in_order():
        nonlocal torn
        for item in items:
            name = item["name"]
            if name in flushed:
                continue
            record = results.get(name) or stored_record(item)
            if record is None or record.get("problem"):
                if item in wanted:
                    return
                continue
            append_lines(options["output"], record["lines"], torn)
            torn = False
            flushed.add(name)

    def work(item):
        if item["name"] == "baseline":
            record = {"lines": [baseline_line], "ledger": [], "problem": None, "timing": (0.0, 0.0, 0.0)}
        else:
            record = process_item_r3(setup, item, baseline, options)
        with lock:
            results[item["name"]] = record
            options["progress"].finished(item["name"])
            build_seconds, run_seconds, total_seconds = record["timing"]
            print(f"  {item['name']}：编译 {build_seconds:.1f} 秒，跑 {run_seconds:.1f} 秒，合计 {total_seconds:.1f} 秒"
                  + (f"；没得出结果：{record['problem']}" if record["problem"] else ""), file=sys.stderr)
            with open(os.path.join(options["work"], "progress.log"), "a", encoding="utf-8") as handle:
                handle.write(f"{item['name']}\tbuild={build_seconds:.1f}\trun={run_seconds:.1f}\ttotal={total_seconds:.1f}\tproblem={record['problem']}\n")
            if record["problem"]:
                problems.append(f"{item['name']}：{record['problem']}")
            else:
                with open(os.path.join(records_directory, item["name"] + ".json"), "w", encoding="utf-8") as handle:
                    json.dump(record, handle, ensure_ascii=False)
                flush_in_order()

    started = time.monotonic()
    halted = None
    with concurrent.futures.ThreadPoolExecutor(max_workers=options["jobs"]) as pool:
        futures = [pool.submit(work, item) for item in to_run]
        for future in concurrent.futures.as_completed(futures):
            try:
                future.result()
            except Stop as stop:
                halted = halted or stop
    if halted is not None:
        raise halted
    with lock:
        flush_in_order()
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；两份源文件与变异表的 sha256 开头结尾相同（冻结根不是 git 仓，不看 git status）。
    baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
    options["progress"].stop()
    same_baseline = baseline_at_end["rows"] == baseline["rows"]
    restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
    restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    sha_at_end = source_hashes(setup)
    table_unchanged = sha256_of_file(os.path.join(setup.root, TABLE_PATH)) == setup.registered["table_sha256"] if setup.registered.get("table_sha256") else True
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"] or not table_unchanged or not (restored_start and restored_end):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 TV9：收尾基线与开头逐字相同={same_baseline}；根里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… "
                                 f"结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；变异表没变={table_unchanged}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了根，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同")

    recorded, closed, _torn = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，再重跑同一条命令")
    if missing:
        print(f"  还有 {len(missing)} 个变体没有行：{' '.join(missing)}；重跑同一条命令（不带 --item）只补这些")
        return EXIT_INCOMPLETE if problems else EXIT_OK
    outcomes, pin_mismatches = {}, []
    for item in items:
        if item["name"] != "baseline":
            fields = parse_row(recorded[item["name"]][0])
            outcomes[fields["outcome"]] = outcomes.get(fields["outcome"], 0) + 1
            if fields.get("ka_pin") == "mismatch":
                pin_mismatches.append(item["name"])
    closing = [
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={len(baseline['rows'])} "
        f"root_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} root_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"root_lib_sha256_at_start={sha_at_start[LIB_PATH]} root_lib_sha256_at_end={sha_at_end[LIB_PATH]} copies_restored=true",
        f"E7RESULT name=verdict scope=segment3 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_kj0_or_kj3={outcomes.get('red_only_by_kj0_or_kj3', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} not_compared={outcomes.get('not_compared', 0)} "
        f"ka_pin_mismatches={len(pin_mismatches)} baseline_equals_segment2_product={retained_comparison_text(baseline, 'true')}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"], False)
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}{not_run_suffix(setup)}")
    if pin_mismatches:
        print(f"  ✗ KA 钉值对不上（TF8）：{' '.join(pin_mismatches)}；这几个变体的 K 族结果不能读成「没有取样点」，交主 agent；别的变体照判")
        print("     → 怎么办：把这几个变体的 KA 读数与跑前登记 TF8 钉的值并排交主 agent，由主 agent 判是登记钉错还是装置算错")
    if outcomes.get("not_compared", 0):
        print("  ✗ 有变体选行不满足（TH5）记了 not_compared，交主 agent；别的变体照判")
    return EXIT_INCOMPLETE if pin_mismatches or outcomes.get("not_compared", 0) else EXIT_OK


def main_run_r4(setup, options, item_names):
    """r4 模式整轮（登记 r4 9.3）。做法同 main_run_r3：开工核对、开头基线、每个变体一份副本、收尾基线；差在：基线要带 criteria=root_choice_faults 判决行（退 7）、
    要与第一段产物逐行相同（退 6）、不落红例账、选行不满足的记 not_compared、KA 或 KG 钉值不符整轮退 1（TF7）。"""
    check_the_registered_inputs(setup)
    items, texts = load_items_r4(setup)
    check_the_anchors([item for item in items if item["failure"] is None], texts)
    by_name = {item["name"]: item for item in items}
    for name in item_names:
        if name not in by_name:
            raise Stop(EXIT_USAGE, f"  ✗ 不认识的变体：{name}\n  → 怎么办：--profile r4 --list-items 看全部变体的名字")
    wanted = [by_name[name] for name in item_names] if item_names else items
    recorded, closed, torn = recorded_items(options["output"])
    if "rerunall" in BREAK:
        recorded = {}
    records_directory = os.path.join(options["work"], "records")
    os.makedirs(records_directory, exist_ok=True)

    def stored_record(item):
        path = os.path.join(records_directory, item["name"] + ".json")
        if os.path.exists(path) and "rerunall" not in BREAK:
            with open(path, encoding="utf-8") as handle:
                return json.load(handle)
        return None

    if closed:
        print(f"  {options['output']} 已经收尾（name=done），{len(recorded)} 个变体都有行，这一次什么都不跑")
        return EXIT_OK
    to_run = [item for item in wanted if item["name"] not in recorded and stored_record(item) is None]
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；已有行 {sum(1 for item in wanted if item['name'] in recorded)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()
    baseline = build_and_run_baseline(setup, options, "baseline")
    check_the_profile_rows_r4(baseline)
    check_the_gate_rows_r4(setup, baseline)
    check_against_segment_one_product(setup, baseline)
    baseline_line = baseline_item_line_r4(setup, by_name["baseline"], baseline, options)
    if "baseline" in recorded and recorded["baseline"][0] != baseline_line:
        raise Stop(EXIT_VOID_V6, f"  ✗ 这一次的基线与结果文件里记的基线不同\n     记的：{recorded['baseline'][0][:240]}\n     这次：{baseline_line[:240]}\n"
                                 "  → 怎么办：bin 或根变过；已有的行不能与新的行混在一份产物里，换一个 --output 重跑全部")

    results = {}
    problems = []
    lock = threading.Lock()
    flushed = set(recorded)
    if not os.path.exists(options["output"]):
        append_lines(options["output"], [
            f"E7RESULT name=config experiment=E165 segment=2 profile=r4 registration=research/prompts/e165-r4-prereg.md "
            f"device_arguments={'+'.join(([] if 'r4noprofile' in BREAK else R4_DEVICE_PROFILE_ARGUMENTS) + setup.bounds_arguments()) or 'none'} "
            f"items={len(items)} table_rows={len(items) - 1} recovery_sha256={sha_at_start[RECOVERY_PATH]} lib_sha256={sha_at_start[LIB_PATH]} "
            f"table_sha256={sha256_of_file(os.path.join(setup.root, TABLE_PATH))} widening=none{pins_config_text(setup)}"], False)

    def flush_in_order():
        nonlocal torn
        for item in items:
            name = item["name"]
            if name in flushed:
                continue
            record = results.get(name) or stored_record(item)
            if record is None or record.get("problem"):
                if item in wanted:
                    return
                continue
            append_lines(options["output"], record["lines"], torn)
            torn = False
            flushed.add(name)

    def work(item):
        if item["name"] == "baseline":
            record = {"lines": [baseline_line], "ledger": [], "problem": None, "timing": (0.0, 0.0, 0.0)}
        else:
            record = process_item_r4(setup, item, baseline, options)
        with lock:
            results[item["name"]] = record
            options["progress"].finished(item["name"])
            build_seconds, run_seconds, total_seconds = record["timing"]
            print(f"  {item['name']}：编译 {build_seconds:.1f} 秒，跑 {run_seconds:.1f} 秒，合计 {total_seconds:.1f} 秒"
                  + (f"；没得出结果：{record['problem']}" if record["problem"] else ""), file=sys.stderr)
            with open(os.path.join(options["work"], "progress.log"), "a", encoding="utf-8") as handle:
                handle.write(f"{item['name']}\tbuild={build_seconds:.1f}\trun={run_seconds:.1f}\ttotal={total_seconds:.1f}\tproblem={record['problem']}\n")
            if record["problem"]:
                problems.append(f"{item['name']}：{record['problem']}")
            else:
                with open(os.path.join(records_directory, item["name"] + ".json"), "w", encoding="utf-8") as handle:
                    json.dump(record, handle, ensure_ascii=False)
                flush_in_order()

    started = time.monotonic()
    halted = None
    with concurrent.futures.ThreadPoolExecutor(max_workers=options["jobs"]) as pool:
        futures = [pool.submit(work, item) for item in to_run]
        for future in concurrent.futures.as_completed(futures):
            try:
                future.result()
            except Stop as stop:
                halted = halted or stop
    if halted is not None:
        raise halted
    with lock:
        flush_in_order()
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；两份源文件与变异表的 sha256 开头结尾相同（冻结根不是 git 仓，不看 git status）。
    baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
    options["progress"].stop()
    same_baseline = baseline_at_end["rows"] == baseline["rows"]
    restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
    restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    sha_at_end = source_hashes(setup)
    table_unchanged = sha256_of_file(os.path.join(setup.root, TABLE_PATH)) == setup.registered["table_sha256"] if setup.registered.get("table_sha256") else True
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"] or not table_unchanged or not (restored_start and restored_end):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 TV9：收尾基线与开头逐字相同={same_baseline}；根里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… "
                                 f"结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；变异表没变={table_unchanged}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了根，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同")

    recorded, closed, _torn = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，再重跑同一条命令")
    if missing:
        print(f"  还有 {len(missing)} 个变体没有行：{' '.join(missing)}；重跑同一条命令（不带 --item）只补这些")
        return EXIT_INCOMPLETE if problems else EXIT_OK
    outcomes, pin_mismatches, kg_mismatches = {}, [], []
    for item in items:
        if item["name"] != "baseline":
            fields = parse_row(recorded[item["name"]][0])
            outcomes[fields["outcome"]] = outcomes.get(fields["outcome"], 0) + 1
            if fields.get("ka_pin") == "mismatch":
                pin_mismatches.append(item["name"])
            if fields.get("kg_pin") == "mismatch":
                kg_mismatches.append(item["name"])
    closing = [
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={len(baseline['rows'])} "
        f"root_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} root_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"root_lib_sha256_at_start={sha_at_start[LIB_PATH]} root_lib_sha256_at_end={sha_at_end[LIB_PATH]} copies_restored=true",
        f"E7RESULT name=verdict scope=segment2 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_kj0_or_kj3={outcomes.get('red_only_by_kj0_or_kj3', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} not_compared={outcomes.get('not_compared', 0)} "
        f"ka_pin_mismatches={len(pin_mismatches)} kg_pin_mismatches={len(kg_mismatches)} baseline_equals_segment1_product={retained_comparison_text(baseline, 'true')}{options.get('gate_field', '')}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"], False)
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}{not_run_suffix(setup)}")
    if pin_mismatches:
        print(f"  ✗ KA 钉值对不上（TF7）：{' '.join(pin_mismatches)}；这几个变体的合法域结果不能读成「没有取样点」，交主 agent；别的变体照判")
        print("     → 怎么办：把这几个变体的 KA 读数与跑前登记 TF7 钉的值并排交主 agent，由主 agent 判是登记钉错还是装置算错")
    if kg_mismatches:
        print(f"  ✗ KG 钉值对不上（TF7）：{' '.join(kg_mismatches)}；这几个变体在 KG 上的落点不是登记写的那一种择根序加理想重放，交主 agent；别的变体照判")
        print("     → 怎么办：把这几个变体在 KG 上的落点与跑前登记 TF7 写的择根序并排交主 agent，由主 agent 判是登记写错还是装置算错")
    if outcomes.get("not_compared", 0):
        print("  ✗ 有变体选行不满足（TH5）记了 not_compared，交主 agent；别的变体照判")
    return EXIT_INCOMPLETE if pin_mismatches or kg_mismatches or outcomes.get("not_compared", 0) else EXIT_OK


def measure(setup, options, name):
    check_the_registered_inputs(setup)
    items, selected_rows, texts = load_items(setup)
    check_the_anchors(items, texts)
    item = next((item for item in items if item["name"] == name), None)
    if item is None:
        raise Stop(EXIT_USAGE, f"  ✗ 不认识的变体：{name}\n  → 怎么办：--list-items 看全部变体的名字")
    directory = os.path.join(options["work"], "measure-" + name)
    binary, problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"], wrap_build_for_measure=True)
    peak = "没读到"
    peaks_path = os.environ.get("RUN_WITH_MEMORY_CAP_PEAKS") or os.path.join(os.environ.get("XDG_DATA_HOME") or os.path.expanduser("~/.local/share"), "singlefs", "memory-peaks.tsv")
    if os.path.exists(peaks_path):
        for line in open(peaks_path, encoding="utf-8"):
            fields = line.rstrip("\n").split("\t")
            if len(fields) >= 4 and fields[3] == "e165-kill-bounds measure build":
                peak = f"{int(fields[0]) / (1 << 20):.0f} MiB"
    print(f"量到：变体 {name}，{options['threads']} 线程冷编译（从改 recovery.rs 到编完 bin）挂钟 {build_seconds:.1f} 秒，"
          f"内存峰值（cgroup memory.peak，含页缓存）{peak}；编译{'过了' if binary else '没过：' + problem}")
    if binary:
        completed, run_seconds = run_segment_two(setup, binary, options["threads"], directory)
        print(f"量到：跑 --bounds segment2（{options['threads']} 线程）挂钟 {run_seconds:.1f} 秒，退出码 {completed.returncode}，E7RESULT {len(result_rows(completed.stdout))} 行")
    size = subprocess.run(["du", "-sh", directory], stdout=subprocess.PIPE, text=True).stdout.split("\t")[0]
    print(f"量到：这一份副本加编译目录 {size}")
    restore_and_remove(setup, directory, keep=False, target=target_of(item))
    return EXIT_OK


def clean_value(text):
    return str(text).replace(" ", "_").replace("=", "＝")


def build_row_selection(setup, output_shown):
    """r1 RQ4 到 RQ7：产物行与总判。返回 (行, 23 行都选到且 Z 四行都对得上)。"""
    texts = {path: read_source(setup, path) for path in SOURCE_FILES}
    ranges = function_ranges(setup)
    selection = select_by_name(setup, ranges, texts)
    customs = custom_rows(setup, texts)
    tag_table = load_tag_table(setup)
    hashes = source_hashes(setup)
    lines = [f"E7RESULT name=config experiment=E165 segment=2 part=row_selection registration=research/prompts/e165-r2-prereg.md "
             f"recovery_sha256={hashes[RECOVERY_PATH]} lib_sha256={hashes[LIB_PATH]} table_sha256={sha256_of_file(os.path.join(setup.root, TABLE_PATH))} "
             f"registered_names={len(setup.registered['names'])} custom_rows={len(customs)} function_ranges={len(ranges)} "
             f"rerun=\"bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --select-rows --output {output_shown}\""]
    for (path, name, start, end), registered_entry in zip(ranges, setup.registered["functions"]):
        lines.append(f"E7RESULT name=function_range file={path} function={name} first={start} last={end} registered_first={registered_entry[2]} registered_last={registered_entry[3]}")
    item_of = {}
    for entry in sorted((candidate for candidate in selection if candidate["today_row"] is not None), key=lambda candidate: candidate["today_row"]):
        item_of.setdefault((entry["file"], entry["raw_old"], entry["raw_new"]), f"tsv-{entry['today_row']}")
    totals = {"selected": 0, "same": 0, "differs": [], "extra": [], "missing": [], "rh4": [], "same_method_equal": 0, "not_selected": []}
    for entry in selection:
        key = f"r{SAME_AS.get(entry['registered_row'], entry['registered_row'])}"
        old_tags, today_tags = tags_of(tag_table, key, "old"), tags_of(tag_table, key, "today")
        if old_tags is None or today_tags is None:
            raise Stop(EXIT_HALT_H1, f"  ✗ {TAGS_TABLE_PATH} 缺键 {key} 的 old 或 today 行\n  → 怎么办：9.3 要求每一行逐版写集合；这一次什么都没跑")
        equal, extra, missing = compare_tag_sets(entry["registered_tags"], today_tags)
        independent_equal = old_tags == set(entry["registered_tags"])
        selected = entry["failure"] is None
        totals["selected"] += 1 if selected else 0
        if not selected:
            totals["not_selected"].append(str(entry["registered_row"]))
        if equal:
            totals["same"] += 1
        else:
            totals["differs"].append(str(entry["registered_row"]))
            if extra:
                totals["extra"].append(str(entry["registered_row"]))
            if missing:
                totals["missing"].append(str(entry["registered_row"]))
        if not independent_equal:
            totals["rh4"].append(str(entry["registered_row"]))
        totals["same_method_equal"] += 1 if old_tags == today_tags else 0
        anchor = f"{entry['first']}-{entry['last']}" if entry["first"] is not None else "none"
        function_lines = f"{entry['function_lines'][0]}-{entry['function_lines'][1]}" if entry["function_lines"] else "none"
        lines.append(
            f"E7RESULT name=row_selection row_kind=table item={item_of.get((entry['file'], entry.get('raw_old'), entry.get('raw_new')), 'none') if selected else 'none'} "
            f"mutation_name={clean_value(entry['name'])} registered_table_row={entry['registered_row']} today_table_row={entry['today_row'] if entry['today_row'] else 'none'} "
            f"rows_with_this_name={entry['rows_with_this_name']} file={entry['file'] or 'none'} anchor_hits={entry['hits'] if entry['hits'] is not None else 'none'} "
            f"anchor_lines={anchor} function={entry['function'] or 'none'} function_lines={function_lines} selected={str(selected).lower()} "
            f"failed_condition={entry['failure'] or 'none'} tag_key={key} registered_tags={tag_text(entry['registered_tags'])} "
            f"registered_version_tags_by_the_same_method={tag_text(old_tags)} today_tags={tag_text(today_tags)} today_equals_registered={str(equal).lower()} "
            f"today_extra_tags={tag_text(extra) if extra else 'none'} today_missing_tags={tag_text(missing) if missing else 'none'} "
            f"registered_version_by_the_same_method_equals_registered={str(independent_equal).lower()} rh4={str(not independent_equal).lower()} "
            f"registered_version_equals_today_by_the_same_method={str(old_tags == today_tags).lower()}")
    z_ok = 0
    for row in customs:
        key = row["key"]
        located = locate_in_functions(ranges, row["file"], row["first"], row["last"]) if row["first"] else None
        if key not in setup.registered["z_tags"]:
            # r2 9.1：标签不判；没有登记标签集合的 Z 行（Z5）只按原文命中 1 次判选没选到，标签几列写 not_judged_in_r2。
            failure = "none" if row["hits"] == 1 else f"anchor_hits_{row['hits']}"
            z_ok += 1 if failure == "none" else 0
            lines.append(
                f"E7RESULT name=row_selection row_kind=custom item={key} mutation_name={clean_value(row['title'])} registered_table_row=custom_line_{row['line']} "
                f"today_table_row=custom_line_{row['line']} rows_with_this_name=1 file={row['file']} anchor_hits={row['hits']} "
                f"anchor_lines={str(row['first']) + '-' + str(row['last']) if row['first'] else 'none'} function={located[0] if located else 'none'} "
                f"function_lines={str(located[1]) + '-' + str(located[2]) if located else 'none'} selected={str(failure == 'none').lower()} failed_condition={failure} "
                f"tag_key={key} registered_tags=not_judged_in_r2 today_tags=not_judged_in_r2")
            continue
        registered = setup.registered["z_tags"][key]
        old_tags, today_tags = tags_of(tag_table, key, "old"), tags_of(tag_table, key, "today")
        if old_tags is None or today_tags is None:
            raise Stop(EXIT_HALT_H1, f"  ✗ {TAGS_TABLE_PATH} 缺键 {key} 的 old 或 today 行\n  → 怎么办：9.3 要求每一行逐版写集合；这一次什么都没跑")
        equal, extra, missing = compare_tag_sets(registered, today_tags)
        failure = "none" if row["hits"] == 1 and equal else (f"anchor_hits_{row['hits']}" if row["hits"] != 1 else "today_tags_differ_from_9_4")
        z_ok += 1 if failure == "none" else 0
        independent_equal = old_tags == set(registered)
        lines.append(
            f"E7RESULT name=row_selection row_kind=custom item={key} mutation_name={clean_value(row['title'])} registered_table_row=custom_line_{row['line']} "
            f"today_table_row=custom_line_{row['line']} rows_with_this_name=1 file={row['file']} anchor_hits={row['hits']} "
            f"anchor_lines={str(row['first']) + '-' + str(row['last']) if row['first'] else 'none'} function={located[0] if located else 'none'} "
            f"function_lines={str(located[1]) + '-' + str(located[2]) if located else 'none'} selected={str(failure == 'none').lower()} failed_condition={failure} "
            f"tag_key={key} registered_tags={tag_text(registered)} registered_version_tags_by_the_same_method={tag_text(old_tags)} today_tags={tag_text(today_tags)} "
            f"today_equals_registered={str(equal).lower()} today_extra_tags={tag_text(extra) if extra else 'none'} today_missing_tags={tag_text(missing) if missing else 'none'} "
            f"registered_version_by_the_same_method_equals_registered={str(independent_equal).lower()} rh4={str(not independent_equal).lower()} "
            f"registered_version_equals_today_by_the_same_method={str(old_tags == today_tags).lower()}")
    for key, versions in tag_table.items():
        for version in ("old", "today"):
            for tag, member, text, line_number in versions.get(version, []):
                lines.append(f"E7RESULT name=tag_judgement key={key} version={version} tags={tag} member={member} table_line={line_number} text={clean_value(text)}")
    # RQ7（附带）：9.2 的函数里还有哪些 crates/mutations.tsv 行不在 23 个名字之中；只列，不判标签。
    registered_names = {name for _row, name, _tags in setup.registered["names"]}
    additional = 0
    for number, fields in mutation_table_rows(setup):
        if fields[1] not in SOURCE_FILES or fields[0] in registered_names:
            continue
        old = unescape(fields[2])
        if texts[fields[1]].count(old) != 1:
            continue
        first = texts[fields[1]].count("\n", 0, texts[fields[1]].index(old)) + 1
        located = locate_in_functions(ranges, fields[1], first, first + old.count("\n"))
        if located is None:
            continue
        additional += 1
        lines.append(f"E7RESULT name=additional_row_in_the_function_ranges mutation_name={clean_value(fields[0])} today_table_row={number} file={fields[1]} "
                     f"anchor_lines={first}-{first + old.count(chr(10))} function={located[0]} tags=not_judged_enough_to_judge_without_it")
    all_ok = not totals["not_selected"] and z_ok == len(customs)
    lines.append(
        f"E7RESULT name=verdict scope=row_selection registered_names={len(selection)} selected={totals['selected']} not_selected={len(totals['not_selected'])} "
        f"not_selected_rows={'+'.join(totals['not_selected']) or 'none'} custom_rows={len(customs)} custom_ok={z_ok} "
        f"today_equals_registered={totals['same']} today_differs_from_registered={len(totals['differs'])} differing_rows={'+'.join(totals['differs']) or 'none'} "
        f"differing_rows_with_extra_tags={'+'.join(totals['extra']) or 'none'} differing_rows_with_missing_tags={'+'.join(totals['missing']) or 'none'} "
        f"registered_version_by_the_same_method_differs_from_registered={len(totals['rh4'])} rh4_rows={'+'.join(totals['rh4']) or 'none'} "
        f"registered_version_equals_today_by_the_same_method={totals['same_method_equal']} additional_rows_in_the_function_ranges={additional}")
    lines.append(f"E7RESULT name=done emitted={len(lines) + 1}")
    return lines, all_ok


def select_rows_action(setup, output, output_shown):
    if os.path.exists(output):
        raise Stop(EXIT_USAGE, f"  ✗ {output_shown} 已经存在\n  → 怎么办：research/results 下已有的产物不覆盖；换一个新文件名")
    check_the_registered_inputs(setup)
    lines, all_ok = build_row_selection(setup, output_shown)
    with open(output, "x", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    print(f"  ✓ 写了 {len(lines)} 行：{output_shown}；23 个名字与自造 Z 行{'都选到、有登记集合的 Z 行标签对得上' if all_ok else '里有一行选不到（RQ4 / RQ6），看 name=verdict 与各行 failed_condition'}")
    return EXIT_OK if all_ok else EXIT_INCOMPLETE


# ── 自证

FAKE_CARGO = r'''#!/usr/bin/env bash
# 假的 cargo：只认 build。副本的 recovery.rs 或 lib.rs 里有 BROKEN_SYNTAX 就「编不过」；否则「编」出一个 shell 脚本当二进制，
# 它照两份源文件里有没有 FLAG_RED 决定 L 域 N = 2 的 J4 违例数；有 FLAG_J6_TWO 时 N ≥ 2 的 J6_premise_holds 行见证文字变（违例数不变），
# 有 FLAG_J6QN_THREE 时 N ≥ 3 的 J6Qn_premise_fails 违例数变 1。二进制把每一次被调的参数记进 FAKE_BIN_LOG；带 --profile r2 才打 J6Qn 两行
# （FAKE_BIN_NO_R2_ROWS 设了时带了也不打：认不得 r2 档的装置）。
echo "$PWD" >> "$FAKE_CARGO_LOG"
source_file="crates/singlefs-core/src/recovery.rs crates/singlefs-replay-prefix/src/lib.rs"
if grep -q BROKEN_SYNTAX $source_file; then echo "error: could not compile" >&2; exit 101; fi
red=0; grep -q FLAG_RED $source_file && red=1
j6_two=0; grep -q FLAG_J6_TWO $source_file && j6_two=1
j6qn_three=0; grep -q FLAG_J6QN_THREE $source_file && j6qn_three=1
mkdir -p "$CARGO_TARGET_DIR/release"
cat > "$CARGO_TARGET_DIR/release/e165_replay_bounded_enumeration" <<BIN
#!/usr/bin/env bash
red=$red; j6_two=$j6_two; j6qn_three=$j6qn_three
echo "\$*" >> "\${FAKE_BIN_LOG:-/dev/null}"
profile=r1
if [ "\$1" = --profile ]; then profile="\$2"; shift 2; fi
r2_rows=0; [ "\$profile" = r2 ] && [ -z "\${FAKE_BIN_NO_R2_ROWS:-}" ] && r2_rows=1
state_line() { # n ordinal
  local counters=states
  if [ "\$red" = 1 ] && [ "\$1" = 2 ] && [ "\$2" = 1 ]; then counters=states,J4; fi
  echo "E7RESULT name=state domain=L ring_slots=64 n=\$1 ordinal=\$2 counters=\$counters state=fake\$2;first=returned"
}
case "\$1" in
  --bounds)
    sleep "\${FAKE_BIN_SLEEP:-0}"
    echo "E7RESULT name=config experiment=E165 bounds=segment2"
    for n in 1 2 3; do
      violations=0; [ "\$red" = 1 ] && [ "\$n" = 2 ] && violations=1
      witness=none; [ "\$j6_two" = 1 ] && [ "\$n" -ge 2 ] && witness=changed
      echo "E7RESULT name=states domain=L ring_slots=64 n=\$n states=2 panicked=0 failed=0"
      echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J4 violations=\$violations judged=2 witness=none"
      echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J6_premise_holds violations=0 judged=2 witness=\$witness"
      echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J6_premise_fails violations=0 judged=0 witness=none"
      if [ "\$r2_rows" = 1 ]; then
        qn=0; [ "\$j6qn_three" = 1 ] && [ "\$n" -ge 3 ] && qn=1
        echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J6Qn_premise_holds violations=0 judged=2 witness=none"
        echo "E7RESULT name=violations domain=L ring_slots=64 n=\$n criterion=J6Qn_premise_fails violations=\$qn judged=0 witness=none"
      fi
    done
    echo "E7RESULT name=verdict domain=L ring_slots=64 smallest_n=1 largest_n=3 states=6"
    echo "E7RESULT name=done emitted=21" ;;
  --list-states)
    n="\${2#L:}"; state_line "\$n" 0; state_line "\$n" 1 ;;
  --replay-state)
    rest="\${2#L:}"; state_line "\${rest%%:*}" "\${rest##*:}" ;;
esac
BIN
chmod +x "$CARGO_TARGET_DIR/release/e165_replay_bounded_enumeration"
'''


FAKE_CARGO_R3 = r'''#!/usr/bin/env python3
# 假的 cargo（r3 档自证用）：只认 build。副本的 recovery.rs 或 lib.rs 里有 BROKEN_SYNTAX 就「编不过」；否则「编」出一个 python 脚本当二进制，
# 它照两份源文件里的记号决定 --profile r3 --bounds segment1 --family root_choice 的输出：FLAG_KJ1（K 域 M = 1、U = 2 的 KJ1 违例数 3）、
# FLAG_KJ2（K 域 M = 1、U = 0 的 KJ2 违例数 2）、FLAG_KA_<n>（KA 的 KJ2 违例数 n，没有时 0）、FLAG_HALT（K 域 M = 1、U = 1 的 KJ0 违例 1、
# choose_root 交回空 1，落完行后退 1）、FLAG_OVER（K9 的所选根之上大于上限的态数 4）。二进制把每一次被调的参数记进 FAKE_BIN_LOG；
# 不带 --profile r3 就认不得 --family root_choice（退 2，同真装置）；FAKE_BIN_NO_R3_ROWS 设了时不打 criteria=root_choice 的判决行。
import os
import re
import sys

with open(os.environ["FAKE_CARGO_LOG"], "a") as log:
    log.write(os.getcwd() + "\n")
sources = ""
for path in ("crates/singlefs-core/src/recovery.rs", "crates/singlefs-replay-prefix/src/lib.rs"):
    with open(path, encoding="utf-8") as handle:
        sources += handle.read()
if "BROKEN_SYNTAX" in sources:
    print("error: could not compile", file=sys.stderr)
    sys.exit(101)
ka = re.search(r"FLAG_KA_(\d+)", sources)
flags = {"kj1": "FLAG_KJ1" in sources, "kj2": "FLAG_KJ2" in sources, "halt": "FLAG_HALT" in sources, "over": "FLAG_OVER" in sources,
         "ka": int(ka.group(1)) if ka else 0}
BINARY = """#!/usr/bin/env python3
import os
import sys

FLAGS = __FLAGS__
arguments = sys.argv[1:]
with open(os.environ.get('FAKE_BIN_LOG', os.devnull), 'a') as log:
    log.write(' '.join(arguments) + '\\n')
profile = None
rest = []
index = 0
while index < len(arguments):
    if arguments[index] == '--profile':
        profile = arguments[index + 1]
        index += 2
    else:
        rest.append(arguments[index])
        index += 1
if 'root_choice' in rest and profile != 'r3':
    print('  ✗ --family root_choice 只在 --profile r3 时有', file=sys.stderr)
    print('     → 怎么办：加 --profile r3 重跑，或换一个别的 --family', file=sys.stderr)
    sys.exit(2)
rows = ['E7RESULT name=config experiment=E165 bounds=segment1 family=root_choice profile=r3']
cells = [('K', 1, 0), ('K', 1, 1), ('K', 1, 2), ('K9', 1, 0), ('KW', 1, 0), ('KW', 1, 1)]


def state_row(domain, m, u, none, over):
    return (f'E7RESULT name=root_choice_states domain={domain} ring_slots=64 slots_per_region=8 mounts={m} user_publishes={u} states=10 '
            f'choose_root_returned_none={none} replay_failed=0 above_chosen_root_over_limit={over} above_chosen_root_at_limit=0 '
            f'above_chosen_root_max=1 panicked=0')


def violation_row(domain, m, u, criterion, count, judged, witness='none'):
    return (f'E7RESULT name=violations domain={domain} ring_slots=64 slots_per_region=8 mounts={m} user_publishes={u} '
            f'criterion={criterion} violations={count} judged={judged} witness={witness}')


for domain, m, u in cells:
    kj0 = kj1 = kj2 = kj3 = none = over = 0
    if FLAGS['kj1'] and (domain, m, u) == ('K', 1, 2):
        kj1 = 3
    if FLAGS['kj2'] and (domain, m, u) == ('K', 1, 0):
        kj2 = 2
    if FLAGS['halt'] and (domain, m, u) == ('K', 1, 1):
        kj0, none = 1, 1
    if FLAGS['over'] and domain == 'K9':
        over = 4
    rows.append(state_row(domain, m, u, none, over))
    for criterion, count in (('KJ0', kj0), ('KJ1', kj1), ('KJ2', kj2), ('KJ3', kj3)):
        rows.append(violation_row(domain, m, u, criterion, count, 10))
rows.append(state_row('KA', 0, 0, 1, 0))
rows.append(violation_row('KA', 0, 0, 'KJ0', 1, 125, 'choose_root_returned_none;ka_region0_slots012=[empty,empty,empty]'))
rows.append(violation_row('KA', 0, 0, 'KJ2', FLAGS['ka'], 124))
if not os.environ.get('FAKE_BIN_NO_R3_ROWS'):
    for domain in ('K', 'K9', 'KW', 'KA'):
        rows.append(f'E7RESULT name=verdict domain={domain} ring_slots=64 slots_per_region=8 largest_mounts=1 largest_user_publishes=2 criteria=root_choice states=10')
if FLAGS['halt']:
    rows.append('E7RESULT name=halt condition=TF9_K_域里_choose_root_交回空的态数_1')
rows.append(f"E7RESULT name=verdict scope=run bounds=segment1 family=root_choice states=60 halt_conditions={1 if FLAGS['halt'] else 0}")
rows.append(f'E7RESULT name=done emitted={len(rows) + 1}')
print('\\n'.join(rows))
sys.exit(1 if FLAGS['halt'] else 0)
"""
directory = os.path.join(os.environ["CARGO_TARGET_DIR"], "release")
os.makedirs(directory, exist_ok=True)
binary = os.path.join(directory, "e165_replay_bounded_enumeration")
with open(binary, "w", encoding="utf-8") as handle:
    handle.write(BINARY.replace("__FLAGS__", repr(flags)))
os.chmod(binary, 0o755)
'''


FAKE_CARGO_R4 = r'''#!/usr/bin/env python3
# 假的 cargo（r4 档自证用）：只认 build。副本的 recovery.rs 或 lib.rs 里有 BROKEN_SYNTAX 就「编不过」；否则「编」出一个 python 脚本当二进制，
# 它照两份源文件里的记号决定 --profile r4 --bounds segment1 --family root_choice_faults 的输出：FLAG_KJ1T（KF 域 M = 1、U = 2、被判时无故障、历史罩着的格 KJ1T 违例 3）、
# FLAG_KJ2（KF 域 M = 1、U = 0、被判时 RN、历史罩着的格 KJ2 违例 2）、FLAG_KA_<n>（KA 的 KJ2 违例数 n，没有时 0）、FLAG_KG_<列>（KG 两个格的读数换成登记表里那一列，
# 没有时取「今天的序」那一列）、FLAG_KG_BAD（KG 的读数不等于任何一列）、FLAG_HALT（KF 域 M = 1、U = 1 的 KJ0 违例 1、落完行后退 1）。
# 二进制把每一次被调的参数记进 FAKE_BIN_LOG；不带 --profile r4 就认不得 --family root_choice_faults（退 2，同真装置）；FAKE_BIN_NO_R4_ROWS 设了时不打 criteria=root_choice_faults 的判决行。
import os
import re
import sys

with open(os.environ["FAKE_CARGO_LOG"], "a") as log:
    log.write(os.getcwd() + "\n")
sources = ""
for path in ("crates/singlefs-core/src/recovery.rs", "crates/singlefs-replay-prefix/src/lib.rs"):
    with open(path, encoding="utf-8") as handle:
        sources += handle.read()
if "BROKEN_SYNTAX" in sources:
    print("error: could not compile", file=sys.stderr)
    sys.exit(101)
ka = re.search(r"FLAG_KA_(\d+)", sources)
kg = re.search(r"FLAG_KG_(v\d+)", sources)
flags = {"kj1t": "FLAG_KJ1T" in sources, "kj2": "FLAG_KJ2" in sources, "halt": "FLAG_HALT" in sources, "kg_bad": "FLAG_KG_BAD" in sources,
         "kg": kg.group(1) if kg else "today", "ka": int(ka.group(1)) if ka else 0}
BINARY = """#!/usr/bin/env python3
import os
import sys

FLAGS = __FLAGS__
KG_COLUMNS = {
    'today': {(1, 0): (0, 0, 0), (1, 2): (117, 117, 0)},
    'v1822': {(1, 0): (0, 0, 0), (1, 2): (117, 117, 3)},
    'v1823': {(1, 0): (0, 0, 0), (1, 2): (117, 117, 0)},
    'v1824': {(1, 0): (0, 0, 8), (1, 2): (468, 468, 351)},
    'v1825': {(1, 0): (0, 0, 0), (1, 2): (117, 117, 0)},
    'v1826': {(1, 0): (0, 0, 0), (1, 2): (200, 200, 5)},
}
arguments = sys.argv[1:]
with open(os.environ.get('FAKE_BIN_LOG', os.devnull), 'a') as log:
    log.write(' '.join(arguments) + '\\n')
profile = None
gate = None
rest = []
index = 0
while index < len(arguments):
    if arguments[index] == '--profile':
        profile = arguments[index + 1]
        index += 2
    elif arguments[index] == '--gate':
        gate = arguments[index + 1]
        index += 2
    else:
        rest.append(arguments[index])
        index += 1
if 'root_choice_faults' in rest and profile != 'r4':
    print('  ✗ --family root_choice_faults 只在 --profile r4 时有', file=sys.stderr)
    sys.exit(2)
rows = ['E7RESULT name=config experiment=E165 bounds=segment1 family=root_choice_faults profile=r4' + (' witness_gate=r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots' if gate == 'r2' else '')]
cells = [('KF', 1, 0), ('KF', 1, 1), ('KF', 1, 2), ('KF3', 1, 0), ('KF9', 1, 0), ('KFW', 1, 0), ('KG', 1, 0), ('KG', 1, 2)]
subcells = [('none', 'inside_the_promise'), ('RN', 'inside_the_promise'), ('RI', 'outside_the_promise')]


def state_row(domain, m, u, none):
    return (f'E7RESULT name=root_choice_fault_states domain={domain} ring_slots=64 slots_per_region=8 mounts={m} user_publishes={u} histories=10 '
            f'choose_root_returned_none_with_a_readable_root={none} replay_failed=0 panicked=0 kj1_and_kj1t_differing_states=0')


def violation_row(domain, m, u, fault, label, criterion, count):
    return (f'E7RESULT name=violations domain={domain} ring_slots=64 slots_per_region=8 mounts={m} user_publishes={u} judged_fault={fault} history={label} '
            f'criterion={criterion} states=10 violations={count} judged=10 witness=none')


def count_of(domain, m, u, fault, criterion):
    if FLAGS['kj1t'] and (domain, m, u, fault, criterion) == ('KF', 1, 2, 'none', 'KJ1T'):
        return 3
    if FLAGS['kj2'] and (domain, m, u, fault, criterion) == ('KF', 1, 0, 'RN', 'KJ2'):
        return 2
    if FLAGS['halt'] and (domain, m, u, fault, criterion) == ('KF', 1, 1, 'none', 'KJ0'):
        return 1
    if domain == 'KG' and fault == 'none' and criterion in ('KJ1', 'KJ1T', 'KJ2'):
        reading = KG_COLUMNS[FLAGS['kg']][(m, u)]
        if FLAGS['kg_bad']:
            reading = (reading[0] + 1, reading[1], reading[2])
        return reading[('KJ1', 'KJ1T', 'KJ2').index(criterion)]
    return 0


for domain, m, u in cells:
    rows.append(state_row(domain, m, u, 1 if FLAGS['halt'] and (domain, m, u) == ('KF', 1, 1) else 0))
    for fault, label in subcells:
        for criterion in ('KJ0', 'KJ1', 'KJ1T', 'KJ2', 'KJ3'):
            rows.append(violation_row(domain, m, u, fault, label, criterion, count_of(domain, m, u, fault, criterion)))
rows.append('E7RESULT name=root_choice_states domain=KA ring_slots=64 slots_per_region=8 mounts=0 user_publishes=0 states=125')
rows.append('E7RESULT name=violations domain=KA ring_slots=64 slots_per_region=8 mounts=0 user_publishes=0 criterion=KJ0 violations=1 judged=125 witness=choose_root_returned_none')
rows.append(f"E7RESULT name=violations domain=KA ring_slots=64 slots_per_region=8 mounts=0 user_publishes=0 criterion=KJ2 violations={FLAGS['ka']} judged=124 witness=none")
if not os.environ.get('FAKE_BIN_NO_R4_ROWS'):
    for domain in ('KF', 'KF3', 'KF9', 'KFW', 'KG'):
        rows.append(f'E7RESULT name=verdict domain={domain} ring_slots=64 slots_per_region=8 largest_mounts=1 largest_user_publishes=2 criteria=root_choice_faults states=10')
    rows.append('E7RESULT name=verdict domain=KA ring_slots=64 slots_per_region=8 largest_mounts=0 largest_user_publishes=0 criteria=root_choice states=125')
if FLAGS['halt']:
    rows.append('E7RESULT name=halt condition=TF9_KF_域里_replay_journal_报错的态数_1')
rows.append(f"E7RESULT name=verdict scope=run bounds=segment1 family=root_choice_faults states=60 halt_conditions={1 if FLAGS['halt'] else 0} profile=r4")
rows.append(f'E7RESULT name=done emitted={len(rows) + 1}')
print('\\n'.join(rows))
sys.exit(1 if FLAGS['halt'] else 0)
"""
directory = os.path.join(os.environ["CARGO_TARGET_DIR"], "release")
os.makedirs(directory, exist_ok=True)
binary = os.path.join(directory, "e165_replay_bounded_enumeration")
with open(binary, "w", encoding="utf-8") as handle:
    handle.write(BINARY.replace("__FLAGS__", repr(flags)))
os.chmod(binary, 0o755)
'''


def selftest():
    failures = []
    checked = 0

    def expect(condition, what, fix):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append((what, fix))

    def write(path, text):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)

    def read(path):
        if not os.path.exists(path):
            return ""
        with open(path, encoding="utf-8") as handle:
            return handle.read()

    # --jobs、--threads 的缺省：登记文件第一条非注释行的数是 N；什么都不给 1 × N，只给 --jobs J 是 J × (N ÷ J)，设了上限 7 取 7，两个都给原样
    registered = next((int(line.split()[0]) for line in read(os.path.join(DEFAULT_ROOT, ".claude", "heavy-thread-default")).splitlines()
                       if line.strip() and not line.lstrip().startswith("#")), None)
    defaults = (default_jobs_and_threads(None, None, {}), default_jobs_and_threads(4, None, {}), default_jobs_and_threads(None, None, {"SINGLEFS_THREAD_CAP": "7"}),
                default_jobs_and_threads(2, 3, {}), default_jobs_and_threads(None, 5, {}))
    expect(registered is not None and defaults == ((1, registered), (4, max(1, registered // 4)), (1, 7), (2, 3), (1, 5)),
           f"--jobs、--threads 的缺省：什么都不给要 1 × 登记的重活默认并行度（{registered}）、只给 --jobs 4 要 4 × 登记值 ÷ 4、上限 7 要 1 × 7、给了的原样（实际 {defaults}）",
           "看 default_jobs_and_threads 与 .claude/heavy-thread-default")

    with tempfile.TemporaryDirectory(prefix="e165-kill-bounds-selftest-") as temporary:
        # 内存包装自己的结局：不是自证模式的 Setup（经包装起），包装换成假的（它所在的根下 scripts/hosts/run-with-memory-cap.sh 只退给定的码），
        # 走 run_segment_two 跑一趟，再照三套变体共用的 wrapper_own_problem 判
        wrapper_root = os.path.join(temporary, "wrapper-root")
        os.makedirs(os.path.join(wrapper_root, "scripts", "hosts"))
        wrapper_codes = {}
        for wrapper_exit in (245, 252, 101):
            write(os.path.join(wrapper_root, "scripts/hosts/run-with-memory-cap.sh"),
                  f"#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 {wrapper_exit}\nexit {wrapper_exit}\n")
            wrapper_log = os.path.join(temporary, f"wrapper-{wrapper_exit}")
            os.makedirs(wrapper_log)
            wrapped, _ = run_segment_two(Setup(wrapper_root, False), "/bin/true", 1, wrapper_log)
            wrapper_codes[wrapper_exit] = (wrapped.returncode, wrapper_own_problem(wrapped.returncode))
        expect(wrapper_codes[245][0] == 245 and "显存不够" in (wrapper_codes[245][1] or "") and "不算结果" in (wrapper_codes[245][1] or ""),
               f"经假内存包装跑一趟、包装退 245 时应当判不算结果、报显存不够（实际退 {wrapper_codes[245][0]}，判的 {wrapper_codes[245][1]}）",
               "看 wrapper_own_problem 的 245 那一支")
        expect(wrapper_codes[252][1] is not None and "显存不够" not in wrapper_codes[252][1] and wrapper_codes[101][1] is None,
               f"包装退 252 应当判不算结果（不说显存不够）、装置自己退 101 不算包装的结局（实际 {wrapper_codes[252][1]}、{wrapper_codes[101][1]}）",
               "看 wrapper_own_problem 的 250–254 那一支")
        root = os.path.join(temporary, "root")
        os.makedirs(os.path.join(root, "crates/singlefs-core/src"))
        os.makedirs(os.path.join(root, "crates/singlefs-replay-prefix/src"))
        os.makedirs(os.path.join(root, "scripts/mutation/tables"))
        recovery = "fn replay() {\n    let alpha = 1;\n    let beta = 2;\n    let gamma = 3;\n}\n"
        library = "fn core_one() {\n    let delta = 4;\n    let epsilon = 5;\n}\n\nfn outside_registered() {\n    let zeta = 6;\n}\n"
        write(os.path.join(root, RECOVERY_PATH), recovery)
        write(os.path.join(root, LIB_PATH), library)
        write(os.path.join(root, "Cargo.toml"), "[workspace]\n")
        good_table = ("# 表头\n"
                      "红的\t" + RECOVERY_PATH + "\tlet alpha = 1;\tlet alpha = 9; // FLAG_RED\t-p x\tt\n"
                      "不红的\t" + RECOVERY_PATH + "\tlet beta = 2;\tlet beta = 7;\t-p x\tt\n"
                      "编不过的\t" + RECOVERY_PATH + "\tlet gamma = 3;\tBROKEN_SYNTAX\t-p x\tt\n"
                      "第二份源文件里红的\t" + LIB_PATH + "\tlet delta = 4;\tlet delta = 9; // FLAG_RED\t-p x\tt\n")
        good_custom = "Z1 自造的红\t" + LIB_PATH + "\tlet epsilon = 5;\tlet epsilon = 8; // FLAG_RED\n"
        # 假标签表：r3 今天的集合比登记时多一个（对不上）、r5 登记时那一版自己定的集合比登记的多一个（RH4）
        good_tags = ("# 自检用的假标签表\n"
                     "r2\told\tPD1\tyes\t见证\nr2\ttoday\tPD1\tyes\t见证\n"
                     "r3\told\tPD2\tyes\t见证\nr3\ttoday\tPD2,PD5\tyes\t见证\n"
                     "r4\told\tPD3\tyes\t见证\nr4\ttoday\tPD3\tyes\t见证\n"
                     "r5\told\tPD4,PD6\tyes\t见证\nr5\ttoday\tPD4\tyes\t见证\nr5\ttoday\tPD6\tno\t理由\n"
                     "Z1\told\tPD2\tyes\t见证\nZ1\ttoday\tPD2\tyes\t见证\n")
        registered_names = [[2, "红的", ["PD1"]], [3, "不红的", ["PD2"]], [4, "编不过的", ["PD3"]], [5, "第二份源文件里红的", ["PD4"]]]

        def register(names=None, library_text=None, custom_keys=("Z1",)):
            write(os.path.join(root, "selftest-registered.json"), json.dumps({
                "files_sha256": {RECOVERY_PATH: hashlib.sha256(recovery.encode()).hexdigest(), LIB_PATH: hashlib.sha256((library_text or library).encode()).hexdigest()},
                "functions": [[RECOVERY_PATH, "replay", 1, 5], [LIB_PATH, "core_one", 1, 4]],
                "names": names if names is not None else registered_names,
                "z_tags": {"Z1": ["PD2"]}, "custom_keys": list(custom_keys), "segment1_products": ["seg1.out"]}))

        write(os.path.join(root, TABLE_PATH), good_table)
        write(os.path.join(root, CUSTOM_TABLE_PATH), good_custom)
        write(os.path.join(root, TAGS_TABLE_PATH), good_tags)
        # 第一段 r2 档产物的样子：假二进制不带任何 FLAG、带 --profile r2 时 --bounds 打的每个 (域, N) 行
        write(os.path.join(root, "seg1.out"),
              "".join(f"E7RESULT name=states domain=L ring_slots=64 n={n} states=2 panicked=0 failed=0\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J4 violations=0 judged=2 witness=none\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J6_premise_holds violations=0 judged=2 witness=none\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J6_premise_fails violations=0 judged=0 witness=none\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J6Qn_premise_holds violations=0 judged=2 witness=none\n"
                      f"E7RESULT name=violations domain=L ring_slots=64 n={n} criterion=J6Qn_premise_fails violations=0 judged=0 witness=none\n"
                      for n in (1, 2, 3)))
        register()
        os.makedirs(os.path.join(root, ".claude/gate.d"))
        write(os.path.join(root, ".claude/gate.d/e165-pinned-red-examples.tsv"), "# 自检用的钉住的表\n")
        write(os.path.join(root, "fake-cargo"), FAKE_CARGO)
        os.chmod(os.path.join(root, "fake-cargo"), 0o755)
        subprocess.run(["git", "init", "-q", root], check=True)
        cargo_log = os.path.join(temporary, "cargo.log")
        binary_log = os.path.join(temporary, "binary.log")

        def drive(arguments, broken="", extra=None, fake_root=None):
            # 弄坏开关：外面设了（BREAK，自证这个进程读到的）就每一趟都带上，再并上这一趟自己点名的（对照那几例）；外面没设只带这一趟自己的。
            # r2 那四项（R2_BREAK_MODES）与 r3 那四项（R3_BREAK_MODES）不在这里带：r2、r3 那几趟自己把外面给的原样传进去（r2_broken、r3_broken），别的几趟照常跑
            environment = dict(os.environ)
            environment["E165_KILL_BOUNDS_SELFTEST_ROOT"] = fake_root or root
            environment["FAKE_CARGO_LOG"] = cargo_log
            environment["FAKE_BIN_LOG"] = binary_log
            environment["E165_KILL_BOUNDS_BREAK"] = ",".join(sorted((BREAK - set(R2_BREAK_MODES) - set(R3_BREAK_MODES) - set(R4_BREAK_MODES)) | set(filter(None, broken.split(",")))))
            environment.pop("SINGLEFS_THREAD_CAP", None)
            environment.pop("E165_KILL_BOUNDS_PROGRESS_INTERVAL", None)
            environment.pop("FAKE_BIN_SLEEP", None)
            environment.pop("FAKE_BIN_NO_R2_ROWS", None)
            environment.pop("FAKE_BIN_NO_R3_ROWS", None)
            environment.pop("FAKE_BIN_NO_R4_ROWS", None)
            environment.update(extra or {})
            write(cargo_log, "")
            write(binary_log, "")
            completed = subprocess.run(["bash", SELF_PATH] + arguments, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            builds = [line for line in read(cargo_log).split("\n") if line]
            return completed, builds

        def paths(tag):
            return ["--scratch", os.path.join(temporary, "scratch-" + tag), "--output", os.path.join(temporary, tag + ".out"),
                    "--ledger", os.path.join(temporary, tag + ".ledger"), "--jobs", "2", "--threads", "1"]

        # 一、整轮：红、界内不红、无效各一条，自造一条红；收尾行齐。
        completed, builds = drive(paths("full"))
        output = read(os.path.join(temporary, "full.out"))
        ledger = read(os.path.join(temporary, "full.ledger"))
        last_line = output.rstrip("\n").split("\n")[-1]
        expect(completed.returncode == 0, f"整轮应当退 0，实际 {completed.returncode}：{completed.stdout[-400:]}{completed.stderr[-400:]}", "看 main_run 在假仓上卡在哪一步")
        expect(re.search(r"item=tsv-2 .* outcome=red first_clause_red=L:2:J4 .* legal_family_minimum=2 ", output) is not None, "红的那一条应当记 outcome=red、首红 L:2:J4", "看 compare_with_baseline 与 differences_at")
        expect(re.search(r"item=tsv-3 .* outcome=not_red_within_bounds .* not_red_domains=L\(≤3\) ", output) is not None, "不红的那一条应当记 not_red_within_bounds、上沿 3", "看 process_item 的结局分类")
        expect(re.search(r"item=tsv-4 .* outcome=invalid ", output) is not None, "编不过的那一条应当记 invalid", "看 prepare_and_build 的返回")
        expect(re.search(r"item=tsv-5 .* outcome=red first_clause_red=L:2:J4 ", output) is not None, "文件列是 lib.rs 的那一条应当改 lib.rs、记 outcome=red", "看 prepare_and_build 的 target_of")
        expect(re.search(r"item=Z1 .* outcome=red ", output) is not None, "自造的那一条（文件列 lib.rs）应当记 red", "看 load_items 读自造表")
        expect("已还原，基线与开头逐字相同" in completed.stdout and " name=restored " in output, "收尾要打「已还原，基线与开头逐字相同」并写 name=restored 行", "看 main_run 的收尾")
        expect(last_line == "E7RESULT name=done emitted=10", f"收尾行应当是 name=done emitted=10，实际 {last_line!r}", "emitted 数的是 E7RESULT 行，含 config 与 done")
        expect(re.search(r"tsv-2\t2\tL\t2\t1\tcounters\t\+J4\ttrue\ttrue\ttrue\t", ledger) is not None, f"红例账应当有 tsv-2 在 L、N = 2、序号 1、+J4、两边单条重放都对上的一行，实际：{ledger[-300:]!r}", "看 first_differing_state 与 replayed_state_line")
        expect(len(builds) == 7, f"整轮应当编 7 次（基线 2、变体 5），实际 {len(builds)}", "每个变体一份副本、编一次")
        expect(not os.path.exists(os.path.join(temporary, "scratch-full", WORK_DIRECTORY_NAME, "tsv-2")), "变体的副本与编译目录做完要删掉", "看 restore_and_remove")
        pinned_table = os.path.join(root, ".claude/gate.d/e165-pinned-red-examples.tsv")
        ledger_examples = [line for line in ledger.split("\n") if line and not line.startswith("#") and line.split("\t")[4] != "none"]
        pinned_rows = [line for line in read(pinned_table).split("\n") if line and not line.startswith("#")]
        expect(ledger_examples and len(pinned_rows) == len(ledger_examples) and all(row.endswith("\t--profile r2") for row in pinned_rows)
               and f"钉红例：钉进 {len(ledger_examples)} 行、核了 {len(ledger_examples)} 行" in completed.stdout,
               f"整轮写完账与产物要当场钉进钉表、再核钉全：红例账里有 {len(ledger_examples)} 行反例，钉表应当有同样多行、档都是 --profile r2，"
               f"成功那句报钉进与核了几行；实际表里 {len(pinned_rows)} 行：{completed.stdout[-300:]}", "看 main_run 收尾的 pin_red_examples")

        # 二、锚点命中 0 次：自造表里的报「锚点腐化」退 3；crates 表里的让选出的行对不上登记（停机条款 H1）退 4；两样都一次不编、不写产物。
        write(os.path.join(root, CUSTOM_TABLE_PATH), "Z1 锚点坏的\t" + LIB_PATH + "\tlet omega = 4;\tlet omega = 8;\n")
        completed, builds = drive(paths("rot"))
        expect(completed.returncode == 3 and "锚点腐化" in completed.stdout + completed.stderr, f"锚点命中 0 次应当报「锚点腐化」退 3，实际退 {completed.returncode}", "看 check_the_anchors")
        expect(len(builds) == 0 and not os.path.exists(os.path.join(temporary, "rot.out")), f"锚点腐化时一次都不许编、不许写产物，实际编了 {len(builds)} 次", "check_the_anchors 要在建副本之前")
        completed, builds = drive(paths("rot-broken"), broken="noanchorcheck")
        expect(completed.returncode != 3 and len(builds) > 0, f"弄坏开关 noanchorcheck 开着时应当照跑（证明上一例的红来自锚点检查），实际退 {completed.returncode}、编 {len(builds)} 次", "看 BREAK 的 noanchorcheck")
        write(os.path.join(root, CUSTOM_TABLE_PATH), good_custom)
        write(os.path.join(root, TABLE_PATH), good_table.replace("let beta = 2;\tlet beta = 7;", "let omega = 4;\tlet omega = 7;"))
        completed, builds = drive(paths("rot-table"))
        expect(completed.returncode == 4 and "RF2" in completed.stdout and "anchor_hits_0" in completed.stdout and len(builds) == 0, f"crates 表里锚点命中 0 次应当选不到（RF2）退 4、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 select_by_name 与 load_items")
        write(os.path.join(root, TABLE_PATH), good_table)

        # 三、被打断的结果文件：只剩基线与第一条、末行半截；重跑只补没跑的。
        kept = [line for line in output.split("\n") if " name=config " in line or " item=baseline " in line or " item=tsv-2 " in line]
        torn_line = [line for line in output.split("\n") if " item=tsv-3 " in line][0][:60]
        interrupted = os.path.join(temporary, "resume.out")
        kept_text = "\n".join(kept).replace("full.", "resume.") + "\n" + torn_line
        write(interrupted, kept_text)
        write(os.path.join(temporary, "resume.ledger"), "\n".join(line for line in ledger.split("\n") if line.startswith("#") or line.startswith("tsv-2\t")) + "\n")
        completed, builds = drive(paths("resume"))
        resumed = read(interrupted)
        expect(completed.returncode == 0, f"补跑应当退 0，实际 {completed.returncode}：{completed.stdout[-300:]}{completed.stderr[-300:]}", "看 recorded_items 与 main_run")
        expect(len(builds) == 6, f"补跑应当只编 6 次（基线 2、没跑的变体 4），实际 {len(builds)}", "已有完整一行的变体不许重跑；半截的那一行不算")
        counts = {name: len(re.findall(rf"name=kill_bound item={name} .* complete=yes", resumed)) for name in ("baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "Z1")}
        expect(all(count == 1 for count in counts.values()), f"补跑之后每个变体恰好一行完整的，实际 {counts}", "看 flush_in_order")
        expect(resumed.rstrip("\n").split("\n")[-1].startswith("E7RESULT name=done "), "补齐之后要写收尾行", "看 main_run 的收尾")
        complete_lines = lambda text: sorted(line for line in text.replace("resume.", "full.").split("\n") if line.endswith("complete=yes"))
        expect(complete_lines(resumed) == complete_lines(output), "补跑出来的变体行应当与整轮的逐字相同", "变体行里不许有随跑法变的量")
        pinned_after_resume = [line for line in read(pinned_table).split("\n") if line and not line.startswith("#")]
        expect(pinned_after_resume == pinned_rows and f"钉红例：钉进 0 行、核了 {len(ledger_examples)} 行" in completed.stdout,
               f"补跑补齐之后照样当场钉、当场核：同一批红例已钉，钉表应当不变、报钉进 0 行，实际 {len(pinned_after_resume)} 行：{completed.stdout[-300:]}",
               "看 main_run 收尾的 pin_red_examples：追加已钉的不重复")
        write(interrupted, kept_text)
        shutil.rmtree(os.path.join(temporary, "scratch-resume"), ignore_errors=True)
        completed, builds = drive(paths("resume"), broken="rerunall")
        expect(len(builds) == 7, f"弄坏开关 rerunall 开着时应当每个变体都重编（证明上一例的 6 次来自只补没跑的），实际 {len(builds)}", "看 BREAK 的 rerunall")

        # 三之二、钉红例（文件头「钉红例」）：已收尾的一对重跑同一条命令也照钉照核，不编不跑变体。
        # 钉表清空，再造「账写完了、钉表没钉」（只供自证的 E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND：不追加）：当场 --check 判红、整趟退 8（开关 pincheckskipped 下退 0）
        write(pinned_table, "# 自检用的钉住的表\n")
        completed, builds = drive(paths("full"), extra={"E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND": "1"})
        expect(completed.returncode == EXIT_PIN_INCOMPLETE and "钉红例：核钉全（--check）没过" in completed.stdout and "行没钉进" in completed.stdout and len(builds) == 0,
               f"账写完而钉表没钉时驱动当场核钉全应当判红、退 {EXIT_PIN_INCOMPLETE}、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-300:]}",
               "看 pin_red_examples 的 --check 那一步与它在 main_run 收尾（含已收尾那一支）的位置")
        completed, builds = drive(paths("full"), broken="pincheckskipped", extra={"E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND": "1"})
        expect(completed.returncode == 0 and "核了 没核" in completed.stdout,
               f"弄坏开关 pincheckskipped 开着时应当照样退 0（证明上一例的红来自当场核钉全），实际退 {completed.returncode}：{completed.stdout[-300:]}", "看 BREAK 的 pincheckskipped")
        completed, builds = drive(paths("full"))
        repinned = [line for line in read(pinned_table).split("\n") if line and not line.startswith("#")]
        expect(completed.returncode == 0 and repinned == pinned_rows and len(builds) == 0
               and f"钉进 {len(ledger_examples)} 行、核了 {len(ledger_examples)} 行" in completed.stdout,
               f"已收尾的一对重跑同一条命令应当只重做钉红例：钉回同样的 {len(pinned_rows)} 行、一次不编，实际退 {completed.returncode}、{len(repinned)} 行、编 {len(builds)} 次：{completed.stdout[-300:]}",
               "看 main_run 已收尾那一支")

        # 三之三、留存产物不在（research/ 可以整个删掉）：V7 的比对本次未跑——逐项点名、不当绿也不判红，基线那一行写 not_run（开关 retainedmissingred 下照改前读不到、退非 0）
        os.rename(os.path.join(root, "seg1.out"), os.path.join(temporary, "seg1.out.aside"))
        completed, builds = drive(paths("no-seg1"))
        os.rename(os.path.join(temporary, "seg1.out.aside"), os.path.join(root, "seg1.out"))
        no_seg1 = read(os.path.join(temporary, "no-seg1.out"))
        expect(completed.returncode == 0 and "本次未跑：作废条款 V7 的比对：留存产物" in completed.stdout and "seg1.out 不在" in completed.stdout
               and "本次未跑 1 项" in completed.stdout and " rows_found_verbatim_in_segment1_products=not_run " in no_seg1,
               f"r2：第一段产物不在时 V7 的比对应当逐项报本次未跑、照跑完退 0、基线那一行写 not_run，实际退 {completed.returncode}：{completed.stdout[-400:]}{completed.stderr[-200:]}",
               "看 retained_product_not_run 与 check_against_segment_one")

        # 四、--recheck 单条复跑与产物里的行相同；--list-items 列全、不编。
        completed, builds = drive(["--scratch", os.path.join(temporary, "scratch-recheck"), "--output", os.path.join(temporary, "full.out"),
                                   "--ledger", os.path.join(temporary, "full.ledger"), "--jobs", "1", "--threads", "1", "--recheck", "--item", "tsv-2"])
        expect(completed.returncode == 0 and "✓ tsv-2" in completed.stdout, f"--recheck 同一条应当逐字相同、退 0，实际退 {completed.returncode}：{completed.stdout[-300:]}", "看 main_run 的 recheck 分支")
        completed, builds = drive(["--list-items"])
        listed = [line for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 6 and len(builds) == 0, f"--list-items 应当列 6 个变体、不编，实际：{completed.stdout!r}", "看 main 的 list 分支")

        # 五、进度（假时钟）：差一秒到间隔不打；到了间隔打一行，点名在做的变体、它在哪一步、做完几个；再过一个间隔再打一行。
        clock = [1000.0]
        sink = io.StringIO()
        progress = Progress(3, PROGRESS_INTERVAL_SECONDS, clock=lambda: clock[0], sink=sink)
        progress.phase("tsv-9", "跑 --bounds segment2")
        clock[0] += PROGRESS_INTERVAL_SECONDS - 1
        early = progress.poll()
        clock[0] += 1
        first = progress.poll()
        progress.finished("tsv-9")
        clock[0] += PROGRESS_INTERVAL_SECONDS
        second = progress.poll()
        expect(early is None and first is not None and "tsv-9（跑 --bounds segment2，这一步 300 秒" in first and "做完 0/3 个" in first
               and second is not None and "做完 1/3 个" in second and sink.getvalue().count("E165_KILL_BOUNDS_PROGRESS ") == 2,
               f"假时钟上过了进度间隔（{PROGRESS_INTERVAL_SECONDS} 秒）应当各打一行、没到不打，实际：差一秒 {early!r}；到了 {first!r}；再过一个间隔 {second!r}",
               "看 Progress.due 与 Progress.poll")

        # 六、进度接在整轮上：假二进制每次跑 --bounds 慢 1 秒、间隔换成 0.2 秒，stderr 里要有点名在做的那个变体的进度行。
        progress_scratch = os.path.join(temporary, "scratch-progress")
        completed, builds = drive(["--scratch", progress_scratch, "--output", os.path.join(progress_scratch, "progress.out"),
                                   "--ledger", os.path.join(progress_scratch, "progress.ledger"), "--jobs", "1", "--threads", "1", "--item", "tsv-3"],
                                  broken=",".join(sorted(BREAK & {"progressnever"})), extra={"E165_KILL_BOUNDS_PROGRESS_INTERVAL": "0.2", "FAKE_BIN_SLEEP": "1"})
        progress_lines = [line for line in completed.stderr.split("\n") if line.startswith("E165_KILL_BOUNDS_PROGRESS ")]
        expect(completed.returncode == 0 and any("tsv-3（" in line for line in progress_lines),
               f"整轮里一个变体跑过进度间隔应当打出点名它的进度行，实际退 {completed.returncode}、进度行 {progress_lines[:3]!r}",
               "看 main_run 里 Progress 的 start、process_item 里的 mark")


        # 七、r1 第二段的选行（登记 5.2 R2 三格、文件列一格、标签比对两格、RH1 一格）：都只跑 --select-rows 或停在选行，一次不编；每一格配一个弄坏开关证明它会红。
        def select_run(tag, broken=""):
            target = os.path.join(temporary, tag + ".select.out")
            completed, builds = drive(["--select-rows", "--output", target], broken=broken)
            return completed, builds, read(target)

        verdict_of = lambda product: next((line for line in product.split("\n") if " name=verdict " in line), "")
        write(os.path.join(root, TABLE_PATH), good_table)
        completed, builds, product = select_run("sel-good")
        verdict = verdict_of(product)
        expect(completed.returncode == 0 and len(builds) == 0 and "registered_names=4 selected=4 not_selected=0 " in verdict and "custom_ok=1" in verdict,
               f"选行：假仓里 4 个名字都在、Z1 对得上，应当退 0、不编，实际退 {completed.returncode}、编 {len(builds)} 次：{verdict}", "看 build_row_selection")
        expect("today_equals_registered=3 today_differs_from_registered=1 differing_rows=3 differing_rows_with_extra_tags=3 differing_rows_with_missing_tags=none " in verdict
               and "rh4_rows=5 " in verdict and "registered_version_equals_today_by_the_same_method=2 " in verdict,
               f"选行：假标签表里 r3 今天多一个标签（对不上）、r5 登记时那一版自己定的集合多一个（RH4），总判行应当照实报，实际：{verdict}", "看 compare_tag_sets 与 build_row_selection 的计数")
        expect(re.search(r"name=row_selection row_kind=table item=tsv-5 mutation_name=第二份源文件里红的 .* file=crates/singlefs-replay-prefix/src/lib.rs anchor_hits=1 anchor_lines=2-2 function=core_one function_lines=1-4 selected=true ", product) is not None
               and re.search(r"row_kind=custom item=Z1 .* function=core_one .* selected=true ", product) is not None,
               "选行：文件列是 lib.rs 的那一行应当报它的文件、命中行与所在函数", "看 select_by_name 与 locate_in_functions")
        completed, builds = drive(["--select-rows", "--output", os.path.join(temporary, "sel-good.select.out")])
        expect(completed.returncode == 2 and read(os.path.join(temporary, "sel-good.select.out")) == product, f"选行：产物已存在时应当退 2、不覆盖，实际退 {completed.returncode}", "看 select_rows_action")
        # ① 名字被改掉、行号留着：按名字选应当报「选不到」并点名；弄坏开关 byrownumber 开着时按行号选到
        write(os.path.join(root, TABLE_PATH), good_table.replace("红的\t", "改了名的红的\t", 1))
        completed, builds, product = select_run("sel-renamed")
        expect(completed.returncode == 1 and len(builds) == 0 and "failed_condition=no_row_with_this_name" in product and "not_selected_rows=2 " in verdict_of(product),
               f"选行 ①：名字不在表里应当报 no_row_with_this_name、点名第 2 行、退 1、不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 select_by_name")
        completed, builds, product = select_run("sel-renamed-broken", broken="byrownumber")
        expect(completed.returncode == 0 and "not_selected=0 " in verdict_of(product), f"弄坏开关 byrownumber 开着时应当按行号选到（证明上一例的「选不到」来自按名字选），实际退 {completed.returncode}", "看 BREAK 的 byrownumber")
        # ② 同一个名字多一行
        write(os.path.join(root, TABLE_PATH), good_table + "不红的\t" + RECOVERY_PATH + "\tlet beta = 2;\tlet beta = 6;\t-p x\tt\n")
        completed, builds, product = select_run("sel-duplicate")
        expect(completed.returncode == 1 and len(builds) == 0 and "failed_condition=more_than_one_row_with_this_name" in product and "rows_with_this_name=2" in product,
               f"选行 ②：同名两行应当报 more_than_one_row_with_this_name、退 1、不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 select_by_name")
        completed, builds, product = select_run("sel-duplicate-broken", broken="nodupcheck")
        expect(completed.returncode == 0, f"弄坏开关 nodupcheck 开着时应当取第一行选到（证明上一例的红来自核同名多行），实际退 {completed.returncode}", "看 BREAK 的 nodupcheck")
        # ③ 某一行的原文挪到 9.2 的函数之外
        write(os.path.join(root, TABLE_PATH), good_table.replace("let delta = 4;\tlet delta = 9; // FLAG_RED", "let zeta = 6;\tlet zeta = 9; // FLAG_RED"))
        completed, builds, product = select_run("sel-outside")
        expect(completed.returncode == 1 and len(builds) == 0 and "failed_condition=outside_the_registered_functions" in product and "not_selected_rows=5 " in verdict_of(product),
               f"选行 ③：原文在 9.2 的函数之外应当报 outside_the_registered_functions、点名第 5 行、退 1、不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 locate_in_functions")
        completed, builds, product = select_run("sel-outside-broken", broken="nolocationcheck")
        expect(completed.returncode == 0, f"弄坏开关 nolocationcheck 开着时应当选到（证明上一例的红来自核位置），实际退 {completed.returncode}", "看 BREAK 的 nolocationcheck")
        # ④ 文件列不是 recovery.rs 或 lib.rs
        write(os.path.join(root, TABLE_PATH), good_table.replace("\t" + LIB_PATH + "\tlet delta", "\tcrates/other.rs\tlet delta"))
        completed, builds, product = select_run("sel-other-file")
        expect(completed.returncode == 1 and "failed_condition=file_column_is_not_a_registered_source" in product, f"选行 ④：文件列不是登记的两份应当报 file_column_is_not_a_registered_source，实际退 {completed.returncode}", "看 select_by_name")
        # ⑧ 登记名那一行不成形（cargo 参数那一段空着）：切表照 crates-mutation-rows.py 的 read_table，不收它；弄坏开关 owntablesplit 开着时照收
        write(os.path.join(root, TABLE_PATH), good_table.replace("\tlet alpha = 9; // FLAG_RED\t-p x\tt\n", "\tlet alpha = 9; // FLAG_RED\t\tt\n", 1))
        completed, builds, product = select_run("sel-malformed")
        expect(completed.returncode == 1 and len(builds) == 0 and "failed_condition=no_row_with_this_name" in product and "not_selected_rows=2 " in verdict_of(product),
               f"选行 ⑧：登记名那一行不成形（cargo 参数那一段空着）应当报 no_row_with_this_name、点名第 2 行、退 1、不编，实际退 {completed.returncode}、编 {len(builds)} 次",
               "看 mutation_table_rows（切表照 crates-mutation-rows.py 的 read_table）")
        completed, builds, product = select_run("sel-malformed-broken", broken="owntablesplit")
        expect(completed.returncode == 0, f"弄坏开关 owntablesplit 开着时应当照收那一行、选到（证明上一例的「选不到」来自 read_table 判它不成形），实际退 {completed.returncode}",
               "看 BREAK 的 owntablesplit")
        write(os.path.join(root, TABLE_PATH), good_table)
        # ⑤ 改坏时只写 recovery.rs、不看第二列：文件列是 lib.rs 的那一条由红转「锚点腐化」
        completed, builds = drive(paths("rec-only") + ["--item", "tsv-5"], broken="recoveryonly")
        expect(completed.returncode == 3 and "锚点腐化" in completed.stdout, f"弄坏开关 recoveryonly 开着时文件列是 lib.rs 的那一条应当由红转「锚点腐化」退 3（证明整轮那一格的红来自按第二列改文件），实际退 {completed.returncode}", "看 target_of 与 prepare_and_build")
        # ⑥ RH1：lib.rs 与登记的 sha256 不等：不选行、不编
        write(os.path.join(root, LIB_PATH), "// 多一行\n" + library)
        completed, builds, product = select_run("sel-sha")
        expect(completed.returncode == 4 and "停机条款 RH1" in completed.stdout and len(builds) == 0 and product == "", f"选行 ⑥：lib.rs 的 sha256 不等于登记值应当报停机条款 RH1 退 4、不写产物，实际退 {completed.returncode}", "看 check_the_registered_inputs")
        write(os.path.join(root, LIB_PATH), library)
        # ⑦ 标签比对（登记 5.2 R2 第二行）与登记的绝对数：23 个名字、Z 四行；第 516 行（{PD15}）与第 513 行（{PD4}）互换比判不等，同一行与自己比判相等
        by_row = {row: tags for row, _name, tags in REGISTERED["names"]}
        expect(by_row[516] == ["PD15"] and by_row[513] == ["PD4"] and compare_tag_sets(by_row[516], by_row[513])[0] is False and compare_tag_sets(by_row[516], by_row[516])[0] is True
               and compare_tag_sets(by_row[516], by_row[516] + ["PD1"]) == (False, ["PD1"], []) and compare_tag_sets(by_row[1004], by_row[1004][:1]) == (False, [], ["PD6"]),
               f"标签比对：登记里第 516 行应是 PD15、第 513 行应是 PD4，互换比判不等、自己比自己判相等，多一个报多出来的、少一个报少了的，实际 {by_row.get(516)} {by_row.get(513)}", "看 compare_tag_sets")
        expect(len(REGISTERED["names"]) == 23 and len({row for row, _name, _tags in REGISTERED["names"]}) == 23 and len(REGISTERED["z_tags"]) == 4 and len(REGISTERED["functions"]) == 9
               and [row for row, _name, _tags in REGISTERED["names"]] == [23, 24, 36, 226, 354, 355, 406, 414, 448, 512, 513, 514, 515, 516, 517, 518, 613, 614, 920, 921, 922, 1003, 1004],
               "登记写死的数：23 个名字（登记时的行号 23、24、36、…、1004）、r1 登记标签集合的 Z 四行、9.2 的 9 个函数", "看 REGISTERED")
        expect(REGISTERED["custom_keys"] == ["Z1", "Z2", "Z3", "Z4", "Z5"] and DEVICE_PROFILE_ARGUMENTS == ["--profile", "r2"]
               and J6QN_ROWS == ("J6Qn_premise_holds", "J6Qn_premise_fails")
               and J7_ROWS == ("J7a_instance_table", "J7a_allocation_record_tree_root", "J7a_filesystem_identifier", "J7b", "J7c")
               and set(J6_ROWS) | set(J6QN_ROWS) | set(J7_ROWS) <= CLAUSE_CRITERIA,
               "r2 登记写死的：Z 五行（custom_keys Z1–Z5）、档参数 --profile r2、9.4 第 3 条那七个判据名都在条款判据栏里", "看 REGISTERED、DEVICE_PROFILE_ARGUMENTS 与 CLAUSE_CRITERIA")

        # 八、r2 补的格（登记 r2 9.4 第 6 条）。外面打开的 R2_BREAK_MODES 原样传进这几趟（文件头「弄坏开关」）。
        r2_broken = ",".join(sorted(BREAK & set(R2_BREAK_MODES)))
        r2_table = ("# 表头\n"
                    "J6 二 J6Qn 三\t" + RECOVERY_PATH + "\tlet alpha = 1;\tlet alpha = 9; // FLAG_J6_TWO FLAG_J6QN_THREE\t-p x\tt\n"
                    "只有 J6Qn 三\t" + RECOVERY_PATH + "\tlet beta = 2;\tlet beta = 7; // FLAG_J6QN_THREE\t-p x\tt\n")
        write(os.path.join(root, TABLE_PATH), r2_table)
        write(os.path.join(root, CUSTOM_TABLE_PATH), good_custom)
        register(names=[[2, "J6 二 J6Qn 三", []], [3, "只有 J6Qn 三", []]])
        # ① J6 在 N = 2、J6Qn 在 N = 3 与基线不同 ⇒ (2, 3)；只有 J6Qn 不同 ⇒ J6 报 none；装置每一次被调都带 --profile r2
        completed, builds = drive(paths("r2"), broken=r2_broken)
        output = read(os.path.join(temporary, "r2.out"))
        calls = [line for line in read(binary_log).split("\n") if line]
        expect(completed.returncode == 0, f"r2 整轮应当退 0，实际 {completed.returncode}：{completed.stdout[-400:]}{completed.stderr[-300:]}", "看 main_run 与 check_the_profile_rows")
        expect(re.search(r"item=tsv-2 .* first_red_by_criterion=J6:L=2;J6Qn:L=3 ", output) is not None,
               f"r2：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同应当报 J6:L=2、J6Qn:L=3，实际 {re.findall(r'item=tsv-2 .*?(first_red_by_criterion=\S*)', output)}",
               "看 first_red_by_criterion：两族各自取各自那两行、比整行文字")
        expect(re.search(r"item=tsv-3 .* first_red_by_criterion=J6:L=none;J6Qn:L=3 ", output) is not None,
               f"r2：只有 J6Qn 不同的假变体应当报 J6:L=none、J6Qn:L=3，实际 {re.findall(r'item=tsv-3 .*?(first_red_by_criterion=\S*)', output)}", "看 first_red_by_criterion")
        expect(re.search(r"item=tsv-2 .* first_clause_red=L:3:J6Qn_premise_fails .* first_text_red_criteria=L:2:J6_premise_holds ", output) is not None,
               "r2：J6 只差见证文字时按违例数的首红看不见它（L:3 才红、红的是 J6Qn），按文字的首红在 L:2、点名 J6_premise_holds", "看 CLAUSE_CRITERIA 与 first_text_red_criteria")
        expect(calls and all(line.startswith("--profile r2 ") for line in calls) and any(line.startswith("--profile r2 --list-states ") for line in calls),
               f"r2 整轮里假装置每一次被调都应当带 --profile r2（含 --list-states），实际 {len(calls)} 次、前几次 {calls[:3]!r}", "看 Setup.wrap 与 DEVICE_PROFILE_ARGUMENTS")
        # ② 基线里缺 J6Qn_ 行（装置认不得 r2 档）⇒ 报「装置没带 --profile r2」退 7、变体一个不编、不写产物
        completed, builds = drive(paths("r2-no-rows"), broken=r2_broken, extra={"FAKE_BIN_NO_R2_ROWS": "1"})
        expect(completed.returncode == EXIT_NOT_PROFILE_R2 and "装置没带 --profile r2" in completed.stdout and len(builds) == 1
               and not os.path.exists(os.path.join(temporary, "r2-no-rows.out")),
               f"r2：基线里缺 J6Qn_ 行应当报「装置没带 --profile r2」退 7、变体一个不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_the_profile_rows 与它在 main_run 里的位置")
        # ③ 自造表 Z 五行：--list-items 列全；登记的 Z 行与表对不上时整轮停在 RH1、一次不编
        five_custom = "".join(f"Z{index} 自造第 {index} 条\t{path}\t{old}\t{old[:-1]} // 改\n" for index, (path, old) in enumerate(
            [(LIB_PATH, "let epsilon = 5;"), (RECOVERY_PATH, "let gamma = 3;"), (LIB_PATH, "let delta = 4;"), (LIB_PATH, "let zeta = 6;"), (RECOVERY_PATH, "let alpha = 1;")], 1))
        write(os.path.join(root, CUSTOM_TABLE_PATH), five_custom)
        completed, builds = drive(["--list-items"], broken=r2_broken)
        listed = [line.split("\t")[0] for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 8 and listed[-5:] == ["Z1", "Z2", "Z3", "Z4", "Z5"],
               f"r2：自造表 Z 五行时 --list-items 应当列 8 行、末五行是 Z1–Z5，实际 {len(listed)} 行：{listed}", "看 custom_rows")
        completed, builds = drive(paths("r2-keys"), broken=r2_broken)
        expect(completed.returncode == EXIT_HALT_H1 and "Z 行" in completed.stdout and len(builds) == 0,
               f"r2：登记的 Z 行（假仓里只有 Z1）与自造表的五行对不上应当停在 RH1 退 4、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_the_custom_keys")

        # 九、r3 补的格（登记 r3 9.3 第 6 条的四格，另加几格）。r3 有自己的假仓 root3（假 cargo 是 python，假装置认 --profile r3 --family root_choice）；
        # 外面打开的 R3_BREAK_MODES 原样传进这几趟（文件头「弄坏开关」）。
        r3_broken = ",".join(sorted(BREAK & set(R3_BREAK_MODES)))
        root3 = os.path.join(temporary, "root3")
        os.makedirs(os.path.join(root3, "crates/singlefs-core/src"))
        os.makedirs(os.path.join(root3, "crates/singlefs-replay-prefix/src"))
        recovery3 = "fn choose_root() {\n    let b1 = 1;\n    let b2 = 2;\n}\n"
        library3 = "fn choose_root_slot() {\n    let a1 = 1;\n    let a2 = 2;\n    let a3 = 3;\n    let a4 = 4;\n    let a5 = 5;\n}\n"
        write(os.path.join(root3, RECOVERY_PATH), recovery3)
        write(os.path.join(root3, LIB_PATH), library3)
        write(os.path.join(root3, "Cargo.toml"), "[workspace]\n")
        write(os.path.join(root3, "fake-cargo"), FAKE_CARGO_R3)
        os.chmod(os.path.join(root3, "fake-cargo"), 0o755)
        fake_names = [(1822, "平局取后者赢", ["inferred", 0], LIB_PATH, "let a1 = 1;", "let a1 = 9;"),
                      (1823, "比较不看实例", ["pinned", 18], LIB_PATH, "let a2 = 2;", "let a2 = 9; // FLAG_KJ1 FLAG_KJ2 FLAG_KA_18"),
                      (1824, "取第一个有效者", ["pinned", 52], LIB_PATH, "let a3 = 3;", "let a3 = 9; // FLAG_KA_52"),
                      (1825, "平局取低", ["pinned", 36], LIB_PATH, "let a4 = 4;", "let a4 = 9; // FLAG_KA_36"),
                      (1826, "实例代号为主", ["pinned", 18], LIB_PATH, "let a5 = 5;", "let a5 = 9; // FLAG_KA_18 FLAG_HALT FLAG_OVER"),
                      (1827, "胶水不带实例", ["inferred", 18], RECOVERY_PATH, "let b1 = 1;", "let b1 = 9; // FLAG_KA_18")]

        def table3(name_of=lambda row, name: name, new_of=lambda row, new: new):
            return "# 表头\n" + "".join(f"{name_of(row, name)}\t{path}\t{old}\t{new_of(row, new)}\t-p x\tt\n" for row, name, _pin, path, old, new in fake_names)

        def register3():
            write(os.path.join(root3, "selftest-registered-r3.json"), json.dumps({
                "files_sha256": {RECOVERY_PATH: hashlib.sha256(recovery3.encode()).hexdigest(), LIB_PATH: hashlib.sha256(library3.encode()).hexdigest()},
                "functions": [[LIB_PATH, "choose_root_slot", 1, 7], [RECOVERY_PATH, "choose_root", 1, 4]],
                "names": [[row, name, pin] for row, name, pin, _path, _old, _new in fake_names],
                "segment2_product": "seg2.out"}))

        def paths3(tag):
            return ["--profile", "r3", "--scratch", os.path.join(temporary, "scratch-" + tag), "--output", os.path.join(temporary, tag + ".out"),
                    "--jobs", "2", "--threads", "1"]

        write(os.path.join(root3, TABLE_PATH), table3())
        register3()
        # 第二段产物的样子：不带任何记号的假装置在 --profile r3 --bounds segment1 --family root_choice 上打的 E7RESULT 行
        build_environment = dict(os.environ, FAKE_CARGO_LOG=os.devnull, CARGO_TARGET_DIR=os.path.join(temporary, "target3"))
        subprocess.run([os.path.join(root3, "fake-cargo"), "build"], cwd=root3, env=build_environment, check=True)
        baseline_product = subprocess.run([os.path.join(temporary, "target3/release/e165_replay_bounded_enumeration"), "--profile", "r3", "--bounds", "segment1",
                                           "--family", "root_choice"], env=dict(os.environ, FAKE_BIN_LOG=os.devnull), stdout=subprocess.PIPE, text=True, check=True).stdout
        write(os.path.join(root3, "seg2.out"), baseline_product)
        # ① 名单：--list-items 列 7 行（基线 1、变体 6），变体起名 tsv-<今天的表行号>
        completed, builds = drive(["--profile", "r3", "--list-items"], broken=r3_broken, fake_root=root3)
        listed = [line.split("\t")[0] for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 7 and listed == ["baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "tsv-6", "tsv-7"] and len(builds) == 0,
               f"r3：--list-items 应当列 7 行（基线 1、变体 6）、不编，实际 {len(listed)} 行：{listed}", "看 load_items_r3")
        # ② 整轮：钉值都对得上，退 0；首红按 (M, U) 升序取第一个违例数与基线不同的格，KJ1 在 (1, 2)、KJ2 在 (1, 0)；装置每一次被调都带 --profile r3
        completed, builds = drive(paths3("r3"), broken=r3_broken, fake_root=root3)
        output = read(os.path.join(temporary, "r3.out"))
        calls = [line for line in read(binary_log).split("\n") if line]
        expect(completed.returncode == 0, f"r3 整轮应当退 0，实际 {completed.returncode}：{completed.stdout[-500:]}{completed.stderr[-300:]}", "看 main_run_r3 与 check_the_profile_rows_r3")
        expect(re.search(r"item=tsv-3 .* outcome=red red_legal_domains=K first_red_by_criterion=KJ1:K=M1U2,K9=none,KW=none;KJ2:K=M1U0,K9=none,KW=none;KJ0:K=none,K9=none,KW=none;KJ3:K=none,K9=none,KW=none ", output) is not None,
               f"r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同应当报 KJ1:K=M1U2、KJ2:K=M1U0，实际 {re.findall(r'item=tsv-3 .*?(first_red_by_criterion=\S*)', output)}",
               "看 r3_first_reds：每条判据各取各的格，按 (M, U) 升序")
        expect(re.search(r"item=tsv-4 .* outcome=not_red_within_bounds .* ka_kj2=52 ka_expected=52 ka_pin=match ka_inferred=none ", output) is not None
               and re.search(r"item=tsv-2 .* outcome=not_red_within_bounds .* ka_kj2=0 ka_expected=0 ka_pin=not_pinned ka_inferred=match ", output) is not None,
               "r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1822、1827）不钉，只报 ka_inferred", "看 r3_ka_pin 与 process_item_r3")
        expect(re.search(r"item=tsv-6 .* outcome=red_only_by_kj0_or_kj3 red_legal_domains=none first_red_by_criterion=KJ1:K=none,K9=none,KW=none;KJ2:K=none,K9=none,KW=none;KJ0:K=M1U1,K9=none,KW=none;KJ3:.* "
                         r"states_row_differences=K=M1U1:choose_root_returned_none;K9=M1U0:above_chosen_root_over_limit .* above_chosen_root_over_limit=K:0,K9:4,KW:0,KA:0 "
                         r"choose_root_returned_none=K:1,K9:0,KW:0,KA:1 .* halt_conditions=1 .* run_exit=1 run_finished=true ", output) is not None,
               f"r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红，应当照常落行、记 run_exit=1、run_finished=true、outcome=red_only_by_kj0_or_kj3，实际 {[line[:700] for line in output.split(chr(10)) if 'item=tsv-6 ' in line]}",
               "看 process_item_r3：装置退 1 但最后一行是 name=done 就是跑完了")
        expect(output.rstrip("\n").split("\n")[-1] == "E7RESULT name=done emitted=11" and "已还原，基线与开头逐字相同" in completed.stdout and len(builds) == 8,
               f"r3 整轮：收尾行应当是 name=done emitted=11、打「已还原，基线与开头逐字相同」、编 8 次（基线 2、变体 6），实际 {output.rstrip(chr(10)).split(chr(10))[-1]!r}、编 {len(builds)} 次", "看 main_run_r3 的收尾")
        expect(calls and all(line.startswith("--profile r3 --bounds segment1 --family root_choice") for line in calls),
               f"r3 整轮里假装置每一次被调都应当带 --profile r3（连同 --bounds segment1 --family root_choice），实际 {len(calls)} 次、前几次 {calls[:3]!r}", "看 Setup.wrap 与 bounds_arguments")
        # ③ KA 上的读数不等于钉值（1825 的假读数 35、钉 36）⇒ ka_pin=mismatch、整轮跑完退 1；弄坏开关 r3kapinunchecked 开着时退 0
        write(os.path.join(root3, TABLE_PATH), table3(new_of=lambda row, new: new.replace("FLAG_KA_36", "FLAG_KA_35")))
        completed, builds = drive(paths3("r3-pin"), broken=r3_broken, fake_root=root3)
        output = read(os.path.join(temporary, "r3-pin.out"))
        expect(completed.returncode == EXIT_INCOMPLETE and re.search(r"item=tsv-5 .* ka_kj2=35 ka_expected=36 ka_pin=mismatch ", output) is not None
               and "ka_pin_mismatches=1 " in output and "KA 钉值对不上" in completed.stdout,
               f"r3：假变体的 KA 读数不等于钉值应当报 ka_pin=mismatch 且整轮退 1（落完行与收尾再退），实际退 {completed.returncode}、{re.findall(r'item=tsv-5 .*?(ka_pin=\S*)', output)}",
               "看 r3_ka_pin 与 main_run_r3 的收尾")
        completed, builds = drive(paths3("r3-pin-broken"), broken=",".join(filter(None, [r3_broken, "r3kapinunchecked"])), fake_root=root3)
        expect(completed.returncode == EXIT_OK, f"弄坏开关 r3kapinunchecked 开着时同一趟应当退 0（证明上一例的退 1 来自核钉值），实际退 {completed.returncode}", "看 BREAK 的 r3kapinunchecked")
        # ④ 基线缺 criteria=root_choice 判决行 ⇒ 报「装置没带 --profile r3」退 7、变体一个不编、不写产物
        write(os.path.join(root3, TABLE_PATH), table3())
        completed, builds = drive(paths3("r3-no-rows"), broken=r3_broken, extra={"FAKE_BIN_NO_R3_ROWS": "1"}, fake_root=root3)
        expect(completed.returncode == EXIT_NOT_PROFILE_R2 and "装置没带 --profile r3" in completed.stdout and len(builds) == 1
               and not os.path.exists(os.path.join(temporary, "r3-no-rows.out")),
               f"r3：基线缺 criteria=root_choice 判决行应当报「装置没带 --profile r3」退 7、变体一个不编、不写产物，实际退 {completed.returncode}、编 {len(builds)} 次",
               "看 check_the_profile_rows_r3 与它在 main_run_r3 里的位置")
        # ⑤ 基线与第二段产物不同 ⇒ 作废条款 TV7，退 6、变体一个不编
        write(os.path.join(root3, "seg2.out"), baseline_product.replace("states=10", "states=11", 1))
        completed, builds = drive(paths3("r3-tv7"), broken=r3_broken, fake_root=root3)
        expect(completed.returncode == EXIT_VOID_V7 and "TV7" in completed.stdout and len(builds) == 1 and not os.path.exists(os.path.join(temporary, "r3-tv7.out")),
               f"r3：基线与第二段产物不逐行相同应当报作废条款 TV7 退 6、变体一个不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_against_segment_two_product")
        # ⑤之二 第二段产物不在（留存产物可以整个删掉）⇒ TV7 的比对本次未跑、逐项点名，照跑完；收尾判决行写 baseline_equals_segment2_product=not_run（开关 retainedmissingred 下照改前报 TV8 退 6）
        os.remove(os.path.join(root3, "seg2.out"))
        completed, builds = drive(paths3("r3-no-seg2"), broken=r3_broken, fake_root=root3)
        r3_no_seg2 = read(os.path.join(temporary, "r3-no-seg2.out"))
        expect(completed.returncode != EXIT_VOID_V7 and "本次未跑：作废条款 TV7 的比对：留存产物" in completed.stdout and "seg2.out 不在" in completed.stdout
               and " baseline_equals_segment2_product=not_run" in r3_no_seg2 and " rows_identical_to_segment2_product=not_run " in r3_no_seg2,
               f"r3：第二段产物不在时 TV7 的比对应当逐项报本次未跑、照跑完、产物写 not_run，实际退 {completed.returncode}：{completed.stdout[-400:]}",
               "看 retained_product_not_run 与 check_against_segment_two_product")
        write(os.path.join(root3, "seg2.out"), baseline_product)
        # ⑥ 选行不满足（变异名在表里改了）⇒ 那一行记 outcome=not_compared、别的行照做、整轮退 1（登记 TH5）
        write(os.path.join(root3, TABLE_PATH), table3(name_of=lambda row, name: "改了名的" + name if row == 1823 else name))
        completed, builds = drive(paths3("r3-th5"), broken=r3_broken, fake_root=root3)
        output = read(os.path.join(temporary, "r3-th5.out"))
        expect(completed.returncode == EXIT_INCOMPLETE and re.search(r"item=tsv-registered-1823 .* outcome=not_compared failed_condition=no_row_with_this_name ", output) is not None
               and "ka_pin=match" in output and len(builds) == 7 and "not_compared=1 " in output,
               f"r3：变异名选不到的那一行应当记 not_compared、别的五行照做（编 7 次：基线 2、变体 5）、整轮退 1，实际退 {completed.returncode}、编 {len(builds)} 次", "看 load_items_r3 与 process_item_r3 的 TH5 一支")
        write(os.path.join(root3, TABLE_PATH), table3())
        # ⑦ r3 模式不认 --select-rows、--recheck、--measure 与 --ledger（退 2）；--profile 只认 r2、r3
        completed, builds = drive(["--profile", "r3", "--select-rows", "--output", os.path.join(temporary, "r3-select.out")], broken=r3_broken, fake_root=root3)
        expect(completed.returncode == EXIT_USAGE and len(builds) == 0, f"r3：--select-rows 应当退 2，实际退 {completed.returncode}", "看 main 的 r3 用法判断")
        completed, builds = drive(paths3("r3-ledger") + ["--ledger", os.path.join(temporary, "r3-ledger.ledger")], broken=r3_broken, fake_root=root3)
        expect(completed.returncode == EXIT_USAGE and len(builds) == 0, f"r3：给了 --ledger 应当退 2（红例账不落），实际退 {completed.returncode}", "看 main 的 r3 用法判断")
        completed, builds = drive(["--profile", "r5", "--list-items"], broken=r3_broken, fake_root=root3)
        expect(completed.returncode == EXIT_USAGE, f"--profile r5 应当退 2，实际退 {completed.returncode}", "看 main 的 --profile")
        # ⑧ 登记写死的 r3 数：六个名字（行号 1822–1827）、钉值 18 / 52 / 36 / 18、两个函数、档参数、界参数、TQ5 看的判据次序
        expect([row for row, _name, _pin in R3_REGISTERED["names"]] == [1822, 1823, 1824, 1825, 1826, 1827]
               and [pin for _row, _name, pin in R3_REGISTERED["names"]] == [("inferred", 0), ("pinned", 18), ("pinned", 52), ("pinned", 36), ("pinned", 18), ("inferred", 18)]
               and [name for _path, name, _first, _last in R3_REGISTERED["functions"]] == ["choose_root_slot", "choose_root"]
               and R3_DEVICE_PROFILE_ARGUMENTS == ["--profile", "r3"] and R3_BOUNDS_ARGUMENTS == ["--bounds", "segment1", "--family", "root_choice"]
               and R3_CRITERIA[:2] == ("KJ1", "KJ2") and R3_LEGAL_DOMAINS == ("K", "K9", "KW"),
               "r3 登记写死的：六个变异名（1822–1827）、KA 上的钉值（1823 → 18、1824 → 52、1825 → 36、1826 → 18，1822、1827 是推的 0、18）、两个函数、档与界参数、首红的判据次序",
               "看 R3_REGISTERED、R3_DEVICE_PROFILE_ARGUMENTS、R3_BOUNDS_ARGUMENTS")

        # 十、r4 补的格（登记 r4 9.3 第 6 条的五格，另加几格）。r4 有自己的假仓 root4（假 cargo 是 python，假装置认 --profile r4 --family root_choice_faults）；
        # 外面打开的 R4_BREAK_MODES 原样传进这几趟（文件头「弄坏开关」）。
        r4_broken = ",".join(sorted(BREAK & set(R4_BREAK_MODES)))
        root4 = os.path.join(temporary, "root4")
        os.makedirs(os.path.join(root4, "crates/singlefs-core/src"))
        os.makedirs(os.path.join(root4, "crates/singlefs-replay-prefix/src"))
        recovery4 = "fn choose_root() {\n    let b1 = 1;\n    let b2 = 2;\n}\n"
        library4 = "fn choose_root_slot() {\n    let a1 = 1;\n    let a2 = 2;\n    let a3 = 3;\n    let a4 = 4;\n    let a5 = 5;\n}\n"
        write(os.path.join(root4, RECOVERY_PATH), recovery4)
        write(os.path.join(root4, LIB_PATH), library4)
        write(os.path.join(root4, "Cargo.toml"), "[workspace]\n")
        write(os.path.join(root4, "fake-cargo"), FAKE_CARGO_R4)
        os.chmod(os.path.join(root4, "fake-cargo"), 0o755)
        fake_names4 = [(1821, "平局取后者赢", ["inferred", 0], LIB_PATH, "let a1 = 1;", "let a1 = 9; // FLAG_KG_v1822"),
                       (1822, "比较不看实例", ["pinned", 18], LIB_PATH, "let a2 = 2;", "let a2 = 9; // FLAG_KJ1T FLAG_KJ2 FLAG_KA_18"),
                       (1823, "取第一个有效者", ["pinned", 52], LIB_PATH, "let a3 = 3;", "let a3 = 9; // FLAG_KA_52 FLAG_KG_v1824"),
                       (1824, "平局取低", ["pinned", 36], LIB_PATH, "let a4 = 4;", "let a4 = 9; // FLAG_KA_36"),
                       (1825, "实例代号为主", ["pinned", 18], LIB_PATH, "let a5 = 5;", "let a5 = 9; // FLAG_KA_18 FLAG_HALT FLAG_KG_v1826"),
                       (1826, "胶水不带实例", ["inferred", 18], RECOVERY_PATH, "let b1 = 1;", "let b1 = 9; // FLAG_KA_18")]
        fake_kg_columns = {"today": {"1,0": [0, 0, 0], "1,2": [117, 117, 0]}, "v1822": {"1,0": [0, 0, 0], "1,2": [117, 117, 3]},
                           "v1823": {"1,0": [0, 0, 0], "1,2": [117, 117, 0]}, "v1824": {"1,0": [0, 0, 8], "1,2": [468, 468, 351]},
                           "v1825": {"1,0": [0, 0, 0], "1,2": [117, 117, 0]}, "v1826": {"1,0": [0, 0, 0], "1,2": [200, 200, 5]}}

        def table4(name_of=lambda row, name: name, new_of=lambda row, new: new):
            return "# 表头\n" + "".join(f"{name_of(row, name)}\t{path}\t{old}\t{new_of(row, new)}\t-p x\tt\n" for row, name, _pin, path, old, new in fake_names4)

        def register4():
            write(os.path.join(root4, "selftest-registered-r4.json"), json.dumps({
                "files_sha256": {RECOVERY_PATH: hashlib.sha256(recovery4.encode()).hexdigest(), LIB_PATH: hashlib.sha256(library4.encode()).hexdigest()},
                "functions": [[LIB_PATH, "choose_root_slot", 1, 7], [RECOVERY_PATH, "choose_root", 1, 4]],
                "names": [[row, name, pin] for row, name, pin, _path, _old, _new in fake_names4],
                "labels": ["v1822", "v1823", "v1824", "v1825", "v1826", "v1827"],
                "kg_columns": fake_kg_columns,
                "segment1_product": "seg1.out"}))

        def paths4(tag):
            return ["--profile", "r4", "--scratch", os.path.join(temporary, "scratch-" + tag), "--output", os.path.join(temporary, tag + ".out"),
                    "--jobs", "2", "--threads", "1"]

        write(os.path.join(root4, TABLE_PATH), table4())
        register4()
        # 第一段产物的样子：不带任何记号的假装置在 --profile r4 --bounds segment1 --family root_choice_faults 上打的 E7RESULT 行
        build_environment = dict(os.environ, FAKE_CARGO_LOG=os.devnull, CARGO_TARGET_DIR=os.path.join(temporary, "target4"))
        subprocess.run([os.path.join(root4, "fake-cargo"), "build"], cwd=root4, env=build_environment, check=True)
        baseline_product4 = subprocess.run([os.path.join(temporary, "target4/release/e165_replay_bounded_enumeration"), "--profile", "r4", "--bounds", "segment1",
                                            "--family", "root_choice_faults"], env=dict(os.environ, FAKE_BIN_LOG=os.devnull), stdout=subprocess.PIPE, text=True, check=True).stdout
        write(os.path.join(root4, "seg1.out"), baseline_product4)
        # ① 名单：--list-items 列 7 行（基线 1、变体 6），变体起名 tsv-<今天的表行号>
        completed, builds = drive(["--profile", "r4", "--list-items"], broken=r4_broken, fake_root=root4)
        listed = [line.split("\t")[0] for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 7 and listed == ["baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "tsv-6", "tsv-7"] and len(builds) == 0,
               f"r4：--list-items 应当列 7 行（基线 1、变体 6）、不编，实际 {len(listed)} 行：{listed}", "看 load_items_r4")
        # ② 整轮：钉值都对得上，退 0；首红按 (M, U, 被判时故障, 历史标签) 升序取第一个违例数与基线不同的子格，KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着)；装置每一次被调都带 --profile r4
        completed, builds = drive(paths4("r4"), broken=r4_broken, fake_root=root4)
        output = read(os.path.join(temporary, "r4.out"))
        calls = [line for line in read(binary_log).split("\n") if line]
        expect(completed.returncode == 0, f"r4 整轮应当退 0，实际 {completed.returncode}：{completed.stdout[-500:]}{completed.stderr[-300:]}", "看 main_run_r4 与 check_the_profile_rows_r4")
        none_text = "none,KF3=none,KF9=none,KFW=none"
        expect(re.search(r"item=tsv-3 .* outcome=red variant=v1823 red_legal_domains=KF differs_on=not_applicable first_red_by_criterion=KJ1:KF=" + none_text
                         + r";KJ1T:KF=M1U2/none/inside,KF3=none,KF9=none,KFW=none;KJ2:KF=M1U0/RN/inside,KF3=none,KF9=none,KFW=none;KJ0:KF=" + none_text, output) is not None,
               f"r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同应当报 KJ1T:KF=M1U2/none/inside、KJ2:KF=M1U0/RN/inside，实际 {re.findall(r'item=tsv-3 .*?(first_red_by_criterion=\S*)', output)}",
               "看 r4_first_reds：每条判据各取各的子格，按 (M, U, 被判时故障, 历史标签) 升序")
        expect(re.search(r"item=tsv-2 .* outcome=not_red_within_bounds variant=v1822 red_legal_domains=none differs_on=KG .* kg_pin=match ", output) is not None,
               f"r4：假变体只在 KG 与基线不同（读数换成它自己那一列）应当记 not_red_within_bounds、differs_on=KG、kg_pin=match，实际 {[line[:600] for line in output.split(chr(10)) if 'item=tsv-2 ' in line]}",
               "看 r4_differs_on 与 process_item_r4 的结局分类")
        expect(re.search(r"item=tsv-4 .* outcome=not_red_within_bounds variant=v1824 .* differs_on=KG\+KA .* ka_kj2=52 ka_expected=52 ka_pin=match ka_inferred=none kg_pin=match ", output) is not None
               and re.search(r"item=tsv-5 .* outcome=not_red_within_bounds variant=v1825 .* differs_on=KA .* ka_kj2=36 ka_expected=36 ka_pin=match ka_inferred=none kg_pin=match ", output) is not None
               and re.search(r"item=tsv-7 .* ka_kj2=18 ka_expected=18 ka_pin=not_pinned ka_inferred=match kg_pin=match ", output) is not None,
               "r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1827 不钉，只报 ka_inferred（v1827 的 KG 取 v1823 那一列）", "看 r4_ka_pin、r4_kg_pin 与 process_item_r4")
        expect(re.search(r"item=tsv-6 .* outcome=red_only_by_kj0_or_kj3 variant=v1826 red_legal_domains=none .* first_red_by_criterion=KJ1:KF=none,KF3=none,KF9=none,KFW=none;"
                         r"KJ1T:KF=none,KF3=none,KF9=none,KFW=none;KJ2:KF=none,KF3=none,KF9=none,KFW=none;KJ0:KF=M1U1/none/inside,.* "
                         r"choose_root_returned_none_with_a_readable_root=KF:1,.* halt_conditions=1 .* run_exit=1 run_finished=true ", output) is not None,
               f"r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红，应当照常落行、记 run_exit=1、run_finished=true、outcome=red_only_by_kj0_or_kj3，实际 {[line[:700] for line in output.split(chr(10)) if 'item=tsv-6 ' in line]}",
               "看 process_item_r4：装置退 1 但最后一行是 name=done 就是跑完了")
        expect(output.rstrip("\n").split("\n")[-1] == "E7RESULT name=done emitted=11" and "已还原，基线与开头逐字相同" in completed.stdout and len(builds) == 8,
               f"r4 整轮：收尾行应当是 name=done emitted=11、打「已还原，基线与开头逐字相同」、编 8 次（基线 2、变体 6），实际 {output.rstrip(chr(10)).split(chr(10))[-1]!r}、编 {len(builds)} 次", "看 main_run_r4 的收尾")
        expect(calls and all(line.startswith("--profile r4 --bounds segment1 --family root_choice_faults") for line in calls),
               f"r4 整轮里假装置每一次被调都应当带 --profile r4（连同 --bounds segment1 --family root_choice_faults），实际 {len(calls)} 次、前几次 {calls[:3]!r}", "看 Setup.wrap 与 bounds_arguments")
        # ③ 变体在 KG 上的读数不等于它那一列（FLAG_KG_BAD）⇒ kg_pin=mismatch、整轮跑完退 1；弄坏开关 r4kgpinunchecked 开着时退 0
        write(os.path.join(root4, TABLE_PATH), table4(new_of=lambda row, new: new.replace("FLAG_KA_36", "FLAG_KA_36 FLAG_KG_BAD")))
        completed, builds = drive(paths4("r4-pin"), broken=r4_broken, fake_root=root4)
        output = read(os.path.join(temporary, "r4-pin.out"))
        expect(completed.returncode == EXIT_INCOMPLETE and re.search(r"item=tsv-5 .* kg_pin=mismatch kg_mismatched_cells=M1U0:1/0/0!=0/0/0;M1U2:118/117/0!=117/117/0 ", output) is not None
               and "kg_pin_mismatches=1 " in output and "KG 钉值对不上" in completed.stdout,
               f"r4：假变体的 KG 读数不等于它那一列应当报 kg_pin=mismatch 且整轮退 1（落完行与收尾再退），实际退 {completed.returncode}、{re.findall(r'item=tsv-5 .*?(kg_pin=\S* kg_mismatched_cells=\S*)', output)}",
               "看 r4_kg_pin 与 main_run_r4 的收尾")
        completed, builds = drive(paths4("r4-pin-broken"), broken=",".join(filter(None, [r4_broken, "r4kgpinunchecked"])), fake_root=root4)
        expect(completed.returncode == EXIT_OK, f"弄坏开关 r4kgpinunchecked 开着时同一趟应当退 0（证明上一例的退 1 来自核 KG 钉值），实际退 {completed.returncode}", "看 BREAK 的 r4kgpinunchecked")
        # ④ 基线缺 criteria=root_choice_faults 判决行 ⇒ 报「装置没带 --profile r4」退 7、变体一个不编、不写产物
        write(os.path.join(root4, TABLE_PATH), table4())
        completed, builds = drive(paths4("r4-no-rows"), broken=r4_broken, extra={"FAKE_BIN_NO_R4_ROWS": "1"}, fake_root=root4)
        expect(completed.returncode == EXIT_NOT_PROFILE_R2 and "装置没带 --profile r4" in completed.stdout and len(builds) == 1
               and not os.path.exists(os.path.join(temporary, "r4-no-rows.out")),
               f"r4：基线缺 criteria=root_choice_faults 判决行应当报「装置没带 --profile r4」退 7、变体一个不编、不写产物，实际退 {completed.returncode}、编 {len(builds)} 次",
               "看 check_the_profile_rows_r4 与它在 main_run_r4 里的位置")
        # ⑤ 基线与第一段产物不同 ⇒ 作废条款 TV7，退 6、变体一个不编
        write(os.path.join(root4, "seg1.out"), baseline_product4.replace("states=10", "states=11", 1))
        completed, builds = drive(paths4("r4-tv7"), broken=r4_broken, fake_root=root4)
        expect(completed.returncode == EXIT_VOID_V7 and "TV7" in completed.stdout and len(builds) == 1 and not os.path.exists(os.path.join(temporary, "r4-tv7.out")),
               f"r4：基线与第一段产物不逐行相同应当报作废条款 TV7 退 6、变体一个不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_against_segment_one_product")
        # ⑤之二 第一段产物不在（留存产物可以整个删掉）⇒ TV7 的比对本次未跑、逐项点名，照跑完；收尾判决行写 baseline_equals_segment1_product=not_run（开关 retainedmissingred 下照改前报 TV8 退 6）
        os.remove(os.path.join(root4, "seg1.out"))
        completed, builds = drive(paths4("r4-no-seg1"), broken=r4_broken, fake_root=root4)
        r4_no_seg1 = read(os.path.join(temporary, "r4-no-seg1.out"))
        expect(completed.returncode != EXIT_VOID_V7 and "本次未跑：作废条款 TV7 的比对：留存产物" in completed.stdout and "seg1.out 不在" in completed.stdout
               and " baseline_equals_segment1_product=not_run" in r4_no_seg1 and " rows_identical_to_segment1_product=not_run " in r4_no_seg1,
               f"r4：第一段产物不在时 TV7 的比对应当逐项报本次未跑、照跑完、产物写 not_run，实际退 {completed.returncode}：{completed.stdout[-400:]}",
               "看 retained_product_not_run 与 check_against_segment_one_product")
        write(os.path.join(root4, "seg1.out"), baseline_product4)
        # ⑥ 选行不满足（变异名在表里改了）⇒ 那一行记 outcome=not_compared、别的行照做、整轮退 1（登记 TH5）
        write(os.path.join(root4, TABLE_PATH), table4(name_of=lambda row, name: "改了名的" + name if row == 1822 else name))
        completed, builds = drive(paths4("r4-th5"), broken=r4_broken, fake_root=root4)
        output = read(os.path.join(temporary, "r4-th5.out"))
        expect(completed.returncode == EXIT_INCOMPLETE and re.search(r"item=tsv-registered-1822 .* outcome=not_compared failed_condition=no_row_with_this_name ", output) is not None
               and "kg_pin=match" in output and len(builds) == 7 and "not_compared=1 " in output,
               f"r4：变异名选不到的那一行应当记 not_compared、别的五行照做（编 7 次：基线 2、变体 5）、整轮退 1，实际退 {completed.returncode}、编 {len(builds)} 次", "看 load_items_r4 与 process_item_r4 的 TH5 一支")
        write(os.path.join(root4, TABLE_PATH), table4())
        # ⑦ r4 模式不认 --select-rows、--recheck、--measure 与 --ledger（退 2）；--profile 只认 r2、r3、r4
        completed, builds = drive(["--profile", "r4", "--select-rows", "--output", os.path.join(temporary, "r4-select.out")], broken=r4_broken, fake_root=root4)
        expect(completed.returncode == EXIT_USAGE and len(builds) == 0, f"r4：--select-rows 应当退 2，实际退 {completed.returncode}", "看 main 的 r4 用法判断")
        completed, builds = drive(paths4("r4-ledger") + ["--ledger", os.path.join(temporary, "r4-ledger.ledger")], broken=r4_broken, fake_root=root4)
        expect(completed.returncode == EXIT_USAGE and len(builds) == 0, f"r4：给了 --ledger 应当退 2（红例账不落），实际退 {completed.returncode}", "看 main 的 r4 用法判断")
        completed, builds = drive(["--profile", "r5", "--list-items"], broken=r4_broken, fake_root=root4)
        expect(completed.returncode == EXIT_USAGE, f"--profile r5 应当退 2，实际退 {completed.returncode}", "看 main 的 --profile")
        # ⑧ 登记写死的 r4 数：六个名字（行号 1821–1826）、钉值 18 / 52 / 36 / 18、两个函数、档参数、界参数、TQ9 看的判据次序、KG 钉值表八个格
        expect([row for row, _name, _pin in R4_REGISTERED["names"]] == [1821, 1822, 1823, 1824, 1825, 1826]
               and [pin for _row, _name, pin in R4_REGISTERED["names"]] == [("inferred", 0), ("pinned", 18), ("pinned", 52), ("pinned", 36), ("pinned", 18), ("inferred", 18)]
               and R4_REGISTERED["labels"] == ["v1822", "v1823", "v1824", "v1825", "v1826", "v1827"]
               and [name for _path, name, _first, _last in R4_REGISTERED["functions"]] == ["choose_root_slot", "choose_root"]
               and R4_DEVICE_PROFILE_ARGUMENTS == ["--profile", "r4"] and R4_BOUNDS_ARGUMENTS == ["--bounds", "segment1", "--family", "root_choice_faults"]
               and R4_CRITERIA[:3] == ("KJ1", "KJ1T", "KJ2") and R4_LEGAL_DOMAINS == ("KF", "KF3", "KF9", "KFW")
               and sorted(R4_REGISTERED["kg_columns"]) == ["today", "v1822", "v1823", "v1824", "v1825", "v1826"]
               and all(len(column) == 8 for column in R4_REGISTERED["kg_columns"].values())
               and R4_REGISTERED["kg_columns"]["v1826"]["2,3"] == [309609, 346005, 49680] and R4_REGISTERED["kg_columns"]["today"]["2,3"] == [265923, 284607, 0],
               "r4 登记写死的：六个变异名（今天的第 1821–1826 行，登记名 v1822–v1827）、KA 上的钉值（v1823 → 18、v1824 → 52、v1825 → 36、v1826 → 18，v1822、v1827 是推的 0、18）、"
               "两个函数、档与界参数、首红的判据次序、KG 上六种本地择根序每格的钉值（KG M = 2、U = 3 一格 v1826 是 309 609 / 346 005 / 49 680、今天的序是 265 923 / 284 607 / 0）",
               "看 R4_REGISTERED、R4_DEVICE_PROFILE_ARGUMENTS、R4_BOUNDS_ARGUMENTS、R4_KG_RULE_READINGS")

        # 十一、r5 补的格（登记 r5 9.3 第 3 条、修1.九乙第 4 条）。三个假仓的源文件都等于原来那套假钉值；r5 那一套（selftest-registered*-r5.json）的两份源文件 sha256 写成全 0，
        # 所以「给了 --pins r5」必须停在 RH1、一次不编，「不给」照常跑（上面几节）。弄坏开关 r5pinsignored 外面打开时这三格红；每格后面自己打开它跑一次对照。
        def r5_variant_of(path, **changes):
            with open(path, encoding="utf-8") as handle:
                registration = json.load(handle)
            registration.update(changes)
            return json.dumps(registration)

        zero_hashes = {RECOVERY_PATH: "0" * 64, LIB_PATH: "0" * 64}
        write(os.path.join(root, TABLE_PATH), r2_table)
        write(os.path.join(root, CUSTOM_TABLE_PATH), good_custom)
        register(names=[[2, "J6 二 J6Qn 三", []], [3, "只有 J6Qn 三", []]])
        write(os.path.join(root, "selftest-registered-r5.json"), r5_variant_of(os.path.join(root, "selftest-registered.json"), files_sha256=zero_hashes))
        write(os.path.join(root3, "selftest-registered-r3-r5.json"), r5_variant_of(os.path.join(root3, "selftest-registered-r3.json"), files_sha256=zero_hashes))
        write(os.path.join(root4, "selftest-registered-r4-r5.json"), r5_variant_of(os.path.join(root4, "selftest-registered-r4.json"), files_sha256=zero_hashes))
        for mode, mode_paths, mode_root in (("r2", paths("r5-pins-r2"), root), ("r3", paths3("r5-pins-r3"), root3), ("r4", paths4("r5-pins-r4"), root4)):
            output_path = mode_paths[mode_paths.index("--output") + 1]
            completed, builds = drive(mode_paths + ["--pins", "r5"], fake_root=mode_root)
            expect(completed.returncode == EXIT_HALT_H1 and "停机条款 RH1" in completed.stdout and len(builds) == 0 and not os.path.exists(output_path),
                   f"r5：给了 --pins r5 应当按 r5 钉值核对、在旧钉值的假仓上停在 RH1（{mode} 模式：退 4、一次不编、不写产物），实际退 {completed.returncode}、编 {len(builds)} 次",
                   "看 Setup：--pins r5 要选 R5_PINS（自证里是 selftest-registered*-r5.json）")
            completed, builds = drive([word.replace("r5-pins-", "r5-pins-ignored-") for word in mode_paths] + ["--pins", "r5"], broken="r5pinsignored", fake_root=mode_root)
            expect(completed.returncode != EXIT_HALT_H1 and len(builds) > 0,
                   f"弄坏开关 r5pinsignored 开着时同一趟（{mode} 模式）应当照旧钉值跑过开工核对、编得到（证明上一例的 RH1 来自选了 r5 那一套），实际退 {completed.returncode}、编 {len(builds)} 次",
                   "看 BREAK 的 r5pinsignored")
        completed, builds = drive(["--profile", "r4", "--pins", "r6", "--list-items"], fake_root=root4)
        expect(completed.returncode == EXIT_USAGE, f"r5：--pins 只认 r5，给 r6 应当退 2，实际退 {completed.returncode}", "看 main 的 --pins")
        # --gate r2：只在 --profile r4 --pins r5 同给时认，别的组合退 2、一次不编
        for tag, arguments in (("no-pins", paths4("r5-gate-no-pins") + ["--gate", "r2"]),
                               ("r3", paths3("r5-gate-r3") + ["--pins", "r5", "--gate", "r2"]),
                               ("r2", paths("r5-gate-r2") + ["--pins", "r5", "--gate", "r2"]),
                               ("r1", paths4("r5-gate-r1") + ["--pins", "r5", "--gate", "r1"])):
            completed, builds = drive(arguments, fake_root=root3 if tag == "r3" else root if tag == "r2" else root4)
            expect(completed.returncode == EXIT_USAGE and len(builds) == 0,
                   f"r5：--gate r2 只在 --profile r4 --pins r5 同给时认（{tag} 那一种组合应当退 2、一次不编），实际退 {completed.returncode}、编 {len(builds)} 次", "看 main 的 --gate 判断")
        # --gate r2 那一套的变异表 sha256 与假仓里的表对不上 ⇒ 停在 RH1、一次不编
        write(os.path.join(root4, "selftest-registered-r4-r5-gate-r2.json"),
              r5_variant_of(os.path.join(root4, "selftest-registered-r4.json"), table_sha256="0" * 64, segment1_product="seg5.out"))
        completed, builds = drive(paths4("r5-gate-table-mismatch") + ["--pins", "r5", "--gate", "r2"], fake_root=root4)
        expect(completed.returncode == EXIT_HALT_H1 and f"{TABLE_PATH} 的 sha256 不等于登记的 {'0' * 64}" in completed.stdout and len(builds) == 0,
               f"r5：--gate r2 那一套的变异表钉值与仓里的表对不上时应当停在 RH1、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次", "看 check_the_registered_inputs 的变异表一支")
        # --gate r2 整轮：假装置每一次被调都带 --gate r2、基线 config 行带 gate=r2、TV7 比 5-甲 那份产物（seg5.out）；产物每个 kill_bound 行与收尾判决行带 gate=r2
        write(os.path.join(root4, "selftest-registered-r4-r5-gate-r2.json"),
              r5_variant_of(os.path.join(root4, "selftest-registered-r4.json"), segment1_product="seg5.out"))
        os.makedirs(os.path.join(root4, os.path.dirname(DEVICE_SOURCE_PATH)), exist_ok=True)
        write(os.path.join(root4, DEVICE_SOURCE_PATH), "// 自检用的假装置源码\n")
        device_sha256 = hashlib.sha256("// 自检用的假装置源码\n".encode()).hexdigest()
        gate_product = subprocess.run([os.path.join(temporary, "target4/release/e165_replay_bounded_enumeration"), "--profile", "r4", "--bounds", "segment1",
                                       "--family", "root_choice_faults", "--gate", "r2"], env=dict(os.environ, FAKE_BIN_LOG=os.devnull), stdout=subprocess.PIPE, text=True, check=True).stdout
        write(os.path.join(root4, "seg5.out"), gate_product)
        completed, builds = drive(paths4("r5-gate") + ["--pins", "r5", "--gate", "r2"], fake_root=root4)
        output = read(os.path.join(temporary, "r5-gate.out"))
        calls = [line for line in read(binary_log).split("\n") if line]
        kill_lines = [line for line in output.split("\n") if line.startswith("E7RESULT name=kill_bound ")]
        verdict_lines = [line for line in output.split("\n") if line.startswith("E7RESULT name=verdict ")]
        config_lines = [line for line in output.split("\n") if line.startswith("E7RESULT name=config ")]
        expect(completed.returncode == EXIT_OK and len(builds) == 8 and calls
               and all(line.startswith("--profile r4 --bounds segment1 --family root_choice_faults --gate r2") for line in calls)
               and len(kill_lines) == 7 and all(" gate=r2 " in line for line in kill_lines)
               and len(verdict_lines) == 1 and verdict_lines[0].endswith(" gate=r2")
               and len(config_lines) == 1 and f" pins=r5 pins_registration=research/prompts/e165-r5-prereg.md gate=r2 device_sha256={device_sha256}" in config_lines[0]
               and "device_arguments=--profile+r4+--bounds+segment1+--family+root_choice_faults+--gate+r2 " in config_lines[0],
               f"r5：--gate r2 整轮应当退 0、假装置每一次被调都带 --gate r2、产物带 gate=r2（kill_bound 7 行、判决行、config 行带装置 sha256），实际退 {completed.returncode}、编 {len(builds)} 次、"
               f"前几次调用 {calls[:2]!r}、kill_bound {len(kill_lines)} 行、{completed.stdout[-300:]}",
               "看 Setup.bounds_arguments、check_the_gate_rows_r4、pins_config_text 与 process_item_r4 的 head")
        completed, builds = drive(paths4("r5-gate-ignored") + ["--pins", "r5", "--gate", "r2"], broken="r4gateignored", fake_root=root4)
        expect(completed.returncode == EXIT_NOT_PROFILE_R2 and "装置没带 --gate r2" in completed.stdout and len(builds) == 1
               and not os.path.exists(os.path.join(temporary, "r5-gate-ignored.out")),
               f"弄坏开关 r4gateignored 开着时同一趟应当报「装置没带 --gate r2」退 7、变体一个不编、不写产物（基线缺 R2 的闸字段），实际退 {completed.returncode}、编 {len(builds)} 次",
               "看 BREAK 的 r4gateignored 与 check_the_gate_rows_r4")
        # 登记写死的 r5 数：钉值取第一段根上的（9.3 第 1 条），名字、标签、KA / KG 钉值、custom_keys 同旧的；原来三套一个字不改（9.3 第 2 条）
        expect([row for row, _name, _tags in R5_PINS["r2"]["names"]] == [23, 24, 36, 224, 347, 348, 399, 407, 441, 505, 506, 507, 508, 509, 510, 511, 604, 605, 908, 909, 910, 990, 991]
               and [(name, tags) for _row, name, tags in R5_PINS["r2"]["names"]] == [(name, tags) for _row, name, tags in REGISTERED["names"]]
               and [(path, name) for path, name, _first, _last in R5_PINS["r2"]["functions"]] == [(path, name) for path, name, _first, _last in REGISTERED["functions"]]
               and R5_PINS["r2"]["functions"][-1] == (RECOVERY_PATH, "replay_journal", 3467, 3614) and R5_PINS["r2"]["custom_keys"] == ["Z1", "Z2", "Z3", "Z4", "Z5"]
               and R5_PINS["r2"]["segment1_products"] == ["research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-legal-family.out",
                                                          "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-adversarial-family.out"]
               and all(R5_PINS[mode]["files_sha256"][RECOVERY_PATH].startswith("4454f6d3") and R5_PINS[mode]["files_sha256"][LIB_PATH].startswith("9966f42f")
                       and R5_PINS[mode]["table_sha256"].startswith("bc2627f8") for mode in ("r2", "r3", "r4"))
               and [row for row, _name, _pin in R5_PINS["r3"]["names"]] == [1810, 1811, 1812, 1813, 1814, 1815] == [row for row, _name, _pin in R5_PINS["r4"]["names"]]
               and [(name, pin) for _row, name, pin in R5_PINS["r3"]["names"]] == [(name, pin) for _row, name, pin in R3_REGISTERED["names"]]
               and [(name, pin) for _row, name, pin in R5_PINS["r4"]["names"]] == [(name, pin) for _row, name, pin in R4_REGISTERED["names"]]
               and R5_PINS["r4"]["labels"] == R4_REGISTERED["labels"] and R5_PINS["r4"]["kg_columns"] == R4_REGISTERED["kg_columns"]
               and R5_PINS["r3"]["functions"] == R5_PINS["r4"]["functions"] == [(LIB_PATH, "choose_root_slot", 3980, 4019), (RECOVERY_PATH, "choose_root", 1268, 1305)]
               and R5_PINS["r3"]["segment2_product"] == "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-root-choice.out"
               and R5_PINS["r4"]["segment1_product"] == "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r4-root-choice-faults.out"
               and R5_GATE_R2_PINS["table_sha256"] == "c9a03eb4af85a8e4c7206dfa3609593390516b80ca52be7090aced5696e84046" != R5_TABLE_SHA256
               and R5_GATE_R2_PINS["segment1_product"] == "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults.out"
               and R5_GATE_R2_PINS["files_sha256"] == R5_FILES_SHA256 and R5_GATE_R2_PINS["functions"] == R5_PINS["r4"]["functions"] and R5_GATE_R2_PINS["names"] == R5_PINS["r4"]["names"]
               and R4_GATE_R2_ARGUMENTS == ["--gate", "r2"]
               and R4_GATE_R2_CONFIG_FIELD == ("witness_gate", "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots")
               and REGISTERED["files_sha256"][RECOVERY_PATH].startswith("3d79b99e") and R3_REGISTERED["files_sha256"][RECOVERY_PATH].startswith("9337dc1b")
               and R4_REGISTERED["files_sha256"][RECOVERY_PATH].startswith("264664af") and R4_REGISTERED["table_sha256"].startswith("db2981d0"),
               "r5 登记写死的：三个模式的 r5 钉值（第一段根上的 recovery.rs 4454f6d3…、lib.rs 9966f42f…、表 bc2627f8…，r2 的 23 个名字在根里第 23–991 行、九个函数，"
               "r3 / r4 的六个名字在第 1810–1815 行、两个函数，比对对象是第一段的重跑产物）、名字与钉值同旧的、--gate r2 那一套（5-甲 根上的表 c9a03eb4…、5-甲 产物名、装置 config 行的 witness_gate=r2_… 键值、两个源文件与函数区间同 r4 那一套）、原来三套的哈希没动",
               "看 R5_PINS、R5_GATE_R2_PINS、R4_GATE_R2_ARGUMENTS")

    for what, fix in failures:
        print(f"  ✗ 自检：{what}")
        print(f"  → 怎么办：{fix}")
    print(f"自检：查了 {checked} 项，{len(failures)} 项不过")
    return EXIT_INCOMPLETE if failures else EXIT_OK


def main(arguments):
    options = {"jobs": None, "threads": None}   # 没给的由 default_jobs_and_threads 补
    item_names = []
    scratch = output = ledger = measure_name = None
    root = DEFAULT_ROOT
    profile = "r2"
    pins = gate = None
    action = "run"
    recheck = False
    position = 0
    flags = {"--list-items": "list", "--selftest": "selftest", "--clean": "clean", "--select-rows": "select"}
    while position < len(arguments):
        argument = arguments[position]
        if argument in flags:
            action = flags[argument]
            position += 1
            continue
        if argument == "--recheck":
            recheck = True
            position += 1
            continue
        if position + 1 >= len(arguments) or argument not in ("--scratch", "--output", "--ledger", "--item", "--jobs", "--threads", "--measure", "--root", "--profile",
                                                              "--pins", "--gate"):
            raise Stop(EXIT_USAGE, f"  ✗ 不认识的参数或缺值：{argument}\n  → 怎么办：用法见文件头")
        value = arguments[position + 1]
        position += 2
        if argument == "--pins":
            if value != "r5":
                raise Stop(EXIT_USAGE, f"  ✗ --pins 只认 r5，收到「{value}」\n  → 怎么办：不给 --pins 是原来那三套钉值；第 5 次重跑第二段用 --pins r5")
            pins = value
        elif argument == "--gate":
            if value != "r2":
                raise Stop(EXIT_USAGE, f"  ✗ --gate 只认 r2，收到「{value}」\n  → 怎么办：不给 --gate 是取甲；第 5 次重跑第五段 5-乙 用 --profile r4 --pins r5 --gate r2")
            gate = value
        elif argument == "--profile":
            if value not in ("r2", "r3", "r4"):
                raise Stop(EXIT_USAGE, f"  ✗ --profile 只认 r2、r3 与 r4，收到「{value}」\n  → 怎么办：不给 --profile 是 r2 那一套；第三段用 --profile r3；第 4 次重跑第二段用 --profile r4")
            profile = value
        elif argument == "--scratch":
            scratch = os.path.abspath(value)
        elif argument == "--output":
            output = value
        elif argument == "--ledger":
            ledger = value
        elif argument == "--item":
            item_names.append(value)
        elif argument == "--root":
            root = os.path.abspath(value)
        elif argument == "--measure":
            action, measure_name = "measure", value
        else:
            if not re.match(r"^[1-9][0-9]*$", value):
                raise Stop(EXIT_USAGE, f"  ✗ {argument} 要正整数，收到「{value}」\n  → 怎么办：写成 {argument} 4")
            options["jobs" if argument == "--jobs" else "threads"] = int(value)
    options["jobs"], options["threads"] = default_jobs_and_threads(options["jobs"], options["threads"])
    if action == "selftest":
        return selftest()
    if gate is not None and (profile != "r4" or pins != "r5"):
        raise Stop(EXIT_USAGE, f"  ✗ --gate r2 只在 --profile r4 --pins r5 同给时认，这一次是 --profile {profile}、--pins {pins or '（没给）'}（登记 r5 修1.九乙第 1 条）\n"
                               "  → 怎么办：写成 --profile r4 --pins r5 --gate r2；这一次一条都没跑")
    selftest_root = os.environ.get("E165_KILL_BOUNDS_SELFTEST_ROOT")
    setup = Setup(selftest_root or root, selftest_root is not None, profile, pins, gate)
    options["pins_text"] = "".join(" " + word for word in setup.pins_arguments())
    options["gate_field"] = f" gate={gate}" if gate else ""
    if profile in ("r3", "r4") and (action in ("select", "measure") or recheck or ledger is not None):
        raise Stop(EXIT_USAGE, f"  ✗ {profile} 模式不认 --select-rows、--measure、--recheck 与 --ledger（登记 {profile} 9.3：红例账不落，选行在整轮开头做）\n"
                               f"  → 怎么办：{profile} 模式只有 --list-items、--clean 与整轮（--scratch、--output、--item、--jobs、--threads）")
    if action == "list":
        items = load_items_r4(setup)[0] if profile == "r4" else load_items_r3(setup)[0] if profile == "r3" else load_items(setup)[0]
        for item in items:
            print(f"{item['name']}\t{item['table']}\t{'+'.join(str(row) for row in item['rows']) or '-'}\t{' ‖ '.join(item['titles'])}")
        return EXIT_OK
    if action == "select":
        if output is None:
            raise Stop(EXIT_USAGE, "  ✗ --select-rows 要 --output <产物>\n  → 怎么办：产物进 research/results/，名字照登记 5.4")
        output_path = os.path.abspath(output)
        shown_path = os.path.relpath(output_path, setup.root) if os.path.commonpath([output_path, setup.root]) == setup.root else os.path.basename(output_path)
        return select_rows_action(setup, output_path, shown_path)
    if scratch is None or os.path.commonpath([scratch, setup.root]) == setup.root:
        raise Stop(EXIT_USAGE, "  ✗ 缺 --scratch，或草稿目录在仓里\n  → 怎么办：--scratch 给一个仓外的目录（副本与编译目录建在它下面的 kill-bounds/）")
    options["work"] = os.path.join(scratch, WORK_DIRECTORY_NAME)
    if action == "clean":
        if os.path.exists(options["work"]):
            print(subprocess.run(["du", "-sh", options["work"]], stdout=subprocess.PIPE, text=True).stdout.strip())
            shutil.rmtree(options["work"])
        # gate-lint:nocount 这一句是收尾删工作目录，判了多少写在各段的 E7RESULT 行与汇总里
        print(f"  ✓ 已删 {options['work']}")
        return EXIT_OK
    cap = os.environ.get("SINGLEFS_THREAD_CAP")
    if cap and cap.isdigit() and options["jobs"] * options["threads"] > int(cap):
        raise Stop(EXIT_USAGE, f"  ✗ jobs × threads = {options['jobs'] * options['threads']} 超过线程上限 {cap}\n  → 怎么办：压小 --jobs 或 --threads")
    os.makedirs(options["work"], exist_ok=True)
    if action == "measure":
        return measure(setup, options, measure_name)
    if profile == "r3":
        if output is None:
            raise Stop(EXIT_USAGE, "  ✗ 缺 --output\n  → 怎么办：产物进 research/results/，名字照登记 r3 5.8（前缀 e165-replay-bounded-enumeration-<日期>-r3-segment3-）")
        options["output"] = os.path.abspath(output)
        options["output_shown"] = os.path.relpath(options["output"], setup.root) if os.path.commonpath([options["output"], setup.root]) == setup.root else os.path.basename(options["output"])
        return main_run_r3(setup, options, item_names)
    if profile == "r4":
        if output is None:
            raise Stop(EXIT_USAGE, "  ✗ 缺 --output\n  → 怎么办：产物进 research/results/，名字照登记 r4 5.7（前缀 e165-replay-bounded-enumeration-<日期>-r4-segment2-）")
        options["output"] = os.path.abspath(output)
        options["output_shown"] = os.path.relpath(options["output"], setup.root) if os.path.commonpath([options["output"], setup.root]) == setup.root else os.path.basename(options["output"])
        return main_run_r4(setup, options, item_names)
    if output is None or ledger is None:
        raise Stop(EXIT_USAGE, "  ✗ 缺 --output 或 --ledger\n  → 怎么办：两个都给；产物进 research/results/，名字照登记 5.5")
    if recheck and not item_names:
        raise Stop(EXIT_USAGE, "  ✗ --recheck 要用 --item 点名\n  → 怎么办：--recheck --item <变体>")
    options["output"], options["ledger"] = os.path.abspath(output), os.path.abspath(ledger)

    def shown(path):
        return os.path.relpath(path, setup.root) if os.path.commonpath([path, setup.root]) == setup.root else os.path.basename(path)

    options["output_shown"], options["ledger_shown"] = shown(options["output"]), shown(options["ledger"])
    return main_run(setup, options, item_names, recheck)


try:
    sys.exit(main(sys.argv[1:]))
except Stop as stop:
    print(str(stop))
    sys.exit(stop.code)
PYTHON
