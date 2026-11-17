#!/usr/bin/env bash
# admission: always 每一趟照此刻的根、变异表与判定库现算每个变体的键、现查 prune-server 里的按项判定（long_run_item_verdicts.py）；上一趟的结论只经判定库按键复用，产物与草稿目录不替这一趟作保
# run-condition: command cargo python3 nice git rsync
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
#   --measure 用的内存包装与第二段产物（TV7 的比对对象）取脚本所在的仓、不取 --root（冻结根里只有 Cargo.toml、Cargo.lock、.cargo、crates/）。
#
#   e165-replay-mutation-kill-bounds.sh --profile r3 --root <第二段冻结根> --scratch <草稿目录> --output <产物> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>（只给自证）]
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
#   r4 模式要 --root 指第一段根（R4_REGISTERED 钉着它的 recovery.rs、lib.rs、crates/mutations.tsv 的 sha256 与两个函数的区间），--measure 用的内存包装与第一段产物（TV7 的比对对象）取脚本所在的仓、
#   不取 --root；基线的 E7RESULT 行必须与第一段产物逐行相同（不同退 6）、criteria=root_choice_faults 的判决行要在（缺了退 7）；整遍峰值 RSS 约 1.7 GB，
#   同时跑的变体数 --jobs 与每个变体的线程数 --threads 见「并行」一段；内存额度整条命令交 task-server（见「内存」一段）。
#   e165-replay-mutation-kill-bounds.sh --profile r4 --root <第一段根> --scratch <草稿目录> --output <产物> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>（只给自证）]
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
#       自造表、标签表、--measure 用的内存包装与第一段产物一律取脚本所在的仓（第一段根里只有 Cargo.toml、Cargo.lock、.cargo、crates/）。不给 --pins 时三个模式照旧取原来三套，一个字不改。
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
#   E165_KILL_BOUNDS_BREAK=defaultone  --jobs、--threads 都不给时取 1 × 1，不取登记的重活默认并行度
#       「--jobs、--threads 的缺省：…」（实际 (1, 1)）
#   E165_KILL_BOUNDS_BREAK=fixedcopylist  每个变体的副本照写死的 FIXED_COPY_ENTRIES 拷，不取共用库的闭包（没有 path 依赖的 prune-server/ 与编译期读的 .claude/heavy-thread-default）
#       「副本应当照共用库的闭包拷：…」（实际只有 Cargo.toml、Cargo.lock 与 crates/ 底下的）
#   E165_KILL_BOUNDS_BREAK=copyconfigurationunset  副本里编、跑装置不交多机配置（--root 给的根不在 git 里，照它的位置找不到配置）
#       「副本里起命令的多机配置应当取脚本所在的仓那一份：…」（实际没设得 None）
#   下面六项外面打开时自证的每一趟都带着它（drive() 照下面九项那一段的带法）：
#   E165_KILL_BOUNDS_BREAK=outputoverwritten  三套整轮的 --output 指到已经存在的文件照收（开跑之后整份换掉那一份）
#       「--output 指到已经存在的文件应当退 2、出路写换一个路径或先挪走那一份、一次不编、那一份一个字不动」（实际退 0、编 7 次）
#   E165_KILL_BOUNDS_BREAK=ledgeroverwritten  不给 --profile 那一套的 --ledger 指到已经存在的文件照收（开跑之后整份换掉那一份）
#       「--ledger 指到已经存在的文件应当退 2、出路写换一个路径或先挪走那一份、一次不编、那一份一个字不动」（实际退 0、编 7 次）
#   E165_KILL_BOUNDS_BREAK=judgeoffthetree  自证起的那几趟 judge: 另带 judge-root:..（判法闭包换到假树的上一层去算：那里没有假判法，每一项都拒算）
#       「改了假树里的判法应当每一项都现判、一项不拒算」（实际拒算 6）
#       连带红的格：「✗ 自检：续跑应当只编 5 次」、「✗ 自检：续跑：库里复用的三项」、「✗ 自检：全部复用的一趟应当一次不编」、「✗ 自检：账写完而钉表没钉时驱动当场核钉全」、
#       「✗ 自检：判过的一对重交同一条命令应当全部复用、照钉照核」、「✗ 自检：弄坏开关 reuseignored 开着时」、「✗ 自检：复用那六格」（复用格随 reuse-cells 带进去，六格都红）
#   E165_KILL_BOUNDS_BREAK=judgewholedriver  真跑的按项判定请求里判法写整份驱动（judge:<本脚本>），不写判法模块里判一个变体的那几个函数
#       「真跑的按项判定请求里判法应当是判法模块里判一个变体的函数与写基线那一行的函数（三套各两个）、judge-root 是脚本所在的仓，不是整份驱动」（实际三套都是 judge:scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh）
#   E165_KILL_BOUNDS_BREAK=pincheckskipped  钉红例时不跑 --check（当场追加之后不核钉全）
#       「账写完而钉表没钉时驱动当场核钉全应当判红、退 8、一次不编」（实际退 0）、「判过的一对重交同一条命令应当全部复用、照钉照核」（实际「核了 没核」）、
#       「整轮写完账与产物要当场钉进钉表、再核钉全」「续跑补齐之后照样当场钉、当场核」（实际成功那句不报核了几行）
#   E165_KILL_BOUNDS_BREAK=retainedmissingred  留存产物不在时不报本次未跑：V7 读不到（退非 0）、TV7 报 TV8 退 6
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
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output <产物> --ledger <红例账> [--item <变体>]… [--jobs <并行度>] [--threads <每个变体的线程数>（只给自证）]
#       点名的变体（不给 --item 是全部）每个一条判定：键相同、库里判绿的取库里的行、不编不跑，别的现判、判完当场入库（「按项判定」一段）；
#       产物与红例账是这一趟的，从头写，照变体的次序落行；中途停下：挪走这一趟没收尾的产物与红例账，再重交同一条命令，只补没判的。
#       27 个变体都有行、收尾的基线与开头逐字相同，才写收尾三行（name=restored、name=verdict、name=done）。
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output <产物> --ledger <红例账> --recheck --item <变体>
#       只复跑点名的那一条，拿新的行与产物里已有的行比，不写产物、不读不写判定库；不同就退 1。
#   e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --measure <变体>
#       量一次：从改 recovery.rs 到编完 bin 的挂钟与内存峰值（冷编译，经 run-with-memory-cap.sh），再量跑 --bounds segment2 的挂钟。不写产物。
#   e165-replay-mutation-kill-bounds.sh --clean --scratch <草稿目录>
#       删掉这条脚本在草稿目录里建的 kill-bounds/（副本、编译目录、每个变体的记录），删前报大小。
#   e165-replay-mutation-kill-bounds.sh --selftest
#       判定写进临时 prune-server：只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem；不在就退 2、一格都不跑），起法：
#         python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --selftest
#       每一趟一份新的库命名空间；假仓里没有真装置的源码，装置入口不换的那几趟每一项都拒算、照跑（入口先算、算不出就停，判法的闭包不收）。
#       自证起的每一趟算键都不碰脚本所在的仓：装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假树里的 Python 函数，judge: 换成假树里的
#       SELFTEST_JUDGE_ENTRY（不带 judge-root:，判法闭包在假树上算；「按项判定」一段）。每判完一项打一行 E165_KILL_BOUNDS_SELFTEST_PROGRESS（立刻刷出）。
#       拿假的仓、假的 cargo 走一遍：锚点命中 0 次必须报「锚点腐化」退 3、一条都不跑；红、界内不红、无效各一条判得对；--output、--ledger 已经存在退 2、一次不编；
#       真跑的按项判定请求里判法写判法模块里判一个变体的函数与写基线那一行的函数（三套各两个），不写整份驱动；
#       续跑：判完 tsv-3 就被停下的一趟重交只补没判的、库里复用的三项不编，再交一趟全部复用、一次不编（装置入口换成假树里的 Python 函数）；
#       改了假树里的判法，同一条命令每一项都现判、一项不拒算；
#       复用那六格（long_run_item_verdicts.py reuse-cells，打 REUSE_SELFTEST script=scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh …）；
#       弄坏开关开着时这几样必须判错。
#       r2 补的格（登记 r2 9.4 第 6 条）：假变体 J6 在 N = 2、J6Qn 在 N = 3 与基线不同 ⇒ 报 (2, 3)；只有 J6Qn 不同 ⇒ J6 报 none；
#       基线里缺 J6Qn_ 行 ⇒ 报「装置没带 --profile r2」退 7、变体一个不编；自造表 Z 五行时 --list-items 列全五条。
#       r3 补的格（登记 r3 9.3 第 6 条，在自己的假仓里走，不碰上面 r2 的假仓）：--list-items 列 7 行；假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线
#       不同 ⇒ 报 KJ1:K=M1U2、KJ2:K=M1U0；假变体在 KA 上的 KJ2 读数不等于钉值 ⇒ ka_pin=mismatch 且整轮退 1；基线缺 criteria=root_choice 判决行
#       ⇒ 报「装置没带 --profile r3」退 7、变体一个不编；另有基线与第二段产物不同退 6、选行不满足记 not_compared、变体跑到装置自己退 1（TF9 的 halt 行）
#       而照常落行三格。
#   每种用法都可以加 --root <仓根>：被测的仓不是这条脚本所在的那一个时用（默认取脚本所在的仓）。
#   三套整轮的 --output、不给 --profile 那一套的 --ledger 指到已经存在的文件：退 2、不编不跑、不写产物与账（出路：换一个路径，或先挪走那一份）；
#   --recheck 读已有的产物与账，不受这一条管。
#
# 做法（每个变体一份副本、一个编译目录，仓里的 crates/ 一个字节不碰；第 3–5 步只做这一趟要现判的变体，库里复用的不编不跑，「按项判定」一段）：
#   1. 开工核对：仓里 recovery.rs、lib.rs 与 crates/mutations.tsv 的 sha256 等于 REGISTERED 里的值（r2 登记 9.4 第 1 条：第一段收尾冻结的根上的值；
#      crates/mutations.tsv 那一个在第一段收尾之前填不出，写的是占位串，开工核对一律停在 RH1）、9.2 那九个函数的行区间（从 fn 行到其后第一个只有 } 的行）
#      等于 9.2 的表，不等是停机条款 RH1（退 4，一条都不跑）。自造表的 Z 行名字要等于 REGISTERED 的 custom_keys（Z1–Z5），不等同退 4。再按 23 个变异名从 crates/mutations.tsv 选行（r1 第九节 9.1，不按行号）：每个名字恰 1 行、
#      文件列是 recovery.rs 或 lib.rs、原文在那份文件里命中 1 次、命中处的起行或末行落在 9.2 的函数里，有一个选不到就退 4、一条都不跑；
#      自造表每行的文件列也要是这两份之一。今天的表行号只用来给变体起名（tsv-<行号>）。
#   2. 每条变异的原文在它第二列点名的那份文件里恰好命中一次（\n 还原成换行）；有一条不是就报「锚点腐化」，退 3，一条都不跑。
#   3. 基线（不改的副本）先编先跑 --profile r2 --bounds segment2；每个有 J6 行的 (域, N) 都要有 J6Qn 两行，缺了就是装置没带 --profile r2，退 7、
#      变体一个不编（r2 9.4 第 6 条）；它每个 (域, N) 的行必须逐字出现在第一段 r2 档的两份产物里（作废条款 V7，r2 9.4 第 2 条、SV9），不然退 6
#      （产物有不在的，这一项本次未跑，见「留存产物」一段）。
#   4. 每个变体：照 scripts/hosts/lib_repository_copy.py 在根上现算的共用清单与闭包（path 依赖、编译期读与运行期起的仓根文件）拷成一份副本（不带 target），
#      原件另存在副本之外、核 sha256；副本里编、跑编出来的装置时，多机配置取脚本所在的仓那一份（照副本或冻结根的位置找不到它）；
#      改坏、`cargo build --release --offline -p singlefs-verification --bin e165_replay_bounded_enumeration`（编不过记「无效」）、
#      跑 --bounds segment2、逐行与基线比；红了的域在首红的 N 上找一个反例状态（两边的 --list-states 逐行比出第一个判定不同的状态），
#      再拿两边的二进制各 --replay-state 单条重放一遍，判定必须与逐态行相同；收尾把原件拷回副本、核 sha256，删掉这份副本与编译目录。
#      建副本、改坏、编译在这份驱动里；起装置、逐行比、找红例、出产物行与红例账行在判法模块 e165_replay_mutation_kill_bounds_variant_judgement.py（「按项判定」一段）。
#   5. 收尾再建一份不改的副本重编重跑基线，输出必须与开头逐字相同；仓里 recovery.rs 的 sha256 与 `git status --porcelain crates/`
#      开头与结尾相同（作废条款 V6），不然退 5、不写收尾行。
#   6. 红例账照落，写完收尾三行之后当场钉进 .claude/gate.d/e165-pinned-red-examples.tsv、当场核钉全（「钉红例」一段）。
# 钉红例：不给 --profile 的那一套（写红例账的只有它）写完一对账（--ledger）与 kill-bounds 产物（--output）、收尾三行落盘之后，
#   先跑 python3 scripts/experiment-apparatus/e165-pin-red-examples.py --root <脚本所在的仓> --kill-bounds <产物> <账>（追加：已钉的不重复，档取产物 config 行的
#   device_arguments=，即 --profile r2），再跑同一份的 --check --kill-bounds <产物> <账>（显式给这一对）；钉表与变异表取脚本所在的仓，不取 --root。
#   有一步退非 0，原样转出它的输出、整趟退 8（EXIT_PIN_INCOMPLETE）；都过，成功那句报「钉红例：钉进 N 行、核了 M 行」。
#   上一趟钉红例没过、改好之后挪走那一趟的产物与账再重跑同一条命令：判过的变体全部从判定库复用、不编不跑，产物与账照样从头写，这一步照样做。--recheck 不钉。
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
# 并行：--jobs 个变体同时做，每个变体的编译与装置各用 --threads 条线程；jobs × threads 不许超过线程上限 N（统一任务服务套的 capped.sh <N> 设的 SINGLEFS_THREAD_CAP）。
#   --threads 只给自证（自证拿它把假跑压到 1 条线程）；交给统一任务服务的命令里不写，写了被拒。
#   缺省：--jobs 不给是 1；--threads 不给是 N ÷ --jobs（向下取整，至少 1），N = SINGLEFS_THREAD_CAP，没设取登记的重活默认并行度（.claude/heavy-thread-default，
#   scripts/hosts/heavy_threads.py）；所以什么都不给是 1 个变体、N 条线程（登记 31 时 jobs 1 × threads 31，--jobs 4 时 4 × 7）。
# 内存：装置、基线与每个变体的二进制都直接起，脚本里不套内存包装、不设内层上限：整条命令交 task-server
#   （task-server/task submit -- bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh …），内存额度按记录由服务给，撞上限由服务放大重排整条命令。
#   变体的退出码一律是装置自己的退出码，照装置的退出码比，没有「包装的结局不算结果」那一支。
#   唯一的例外是 --measure：它量的就是内存峰值，量编译那一步仍经 scripts/hosts/run-with-memory-cap.sh 8G 读它写的峰值表。
#
# 退出码：0 点名的都落了行（现判的或从判定库复用的）；1 有变体没得出结果（装置没跑完）、--recheck 对不上、派出与收回的项数对不上（「每项一行与总计」一段）、
#   按项判定收尾判红（复用哨兵红、键在跑的中途变了，「按项判定」一段）、或自证判红；
#   2 用法不对（--item 写了不认识的变体名时列出可点名的全部变体）、三套整轮的 --output 或不给 --profile 那一套的 --ledger 已经存在、--selftest 不在临时 prune-server 底下；
#   3 锚点腐化；4 停机条款 H1（含自造表的 Z 行名字与登记不等）；5 作废条款 V6（收尾基线与开头不同，或仓里的 crates/ 在跑的时候变了；
#   或这一趟的基线与判定库里同一把键的基线不逐字相同）；6 作废条款 V7；
#   7 基线里缺 J6Qn_ 行（装置没带 --profile r2，r2 9.4 第 6 条）；8 钉红例没过（当场追加或 --check 退非 0，「钉红例」一段）；75 prune-server 不可用（「按项判定」一段）。
# 弄坏开关 E165_KILL_BOUNDS_BREAK=<项>（只给 --selftest 证明它会红用，逗号分隔可给几个；不认识的项当场退出）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（自证的例没有标签，项名直接写那条「✗ 自检：」失败行的句首，括号里写那一行接着报的）：
#   E165_KILL_BOUNDS_BREAK=progressnever  进度间隔的判断永不触发、一行进度都不打
#       「假时钟上过了进度间隔（300 秒）应当各打一行、没到不打」（实际：差一秒 None；到了 None；再过一个间隔 None）
#       「整轮里一个变体跑过进度间隔应当打出点名它的进度行」（实际退 0、进度行 []）
#   下面三项是入口脚本格式那几格（「每项一行与总计」一段），外面打开时自证的每一趟都带着它：
#   E165_KILL_BOUNDS_BREAK=itemcountsdropped  每项一行不带 judged=、red=、reused=
#       「整轮每项一行应当带判了、红了、复用了多少与秒，末行总计」、「续跑：库里复用的三项应当各打一行 reused=1」
#   E165_KILL_BOUNDS_BREAK=itemlinedropped  第一个做完的变体（基线除外）不打它那一行、不算收回（造一项收不回）
#       「整轮每项一行应当带判了、红了、复用了多少与秒，末行总计」（实际派出 6 项、收回 5 项，退 1）
#       连带红的格（这几趟都有变体做完，少收回一项就退 1）：「✗ 自检：整轮应当退 0」、「✗ 自检：续跑应当退 0」、「✗ 自检：续跑：库里复用的三项」、「✗ 自检：r2：第一段产物不在时 V7 的比对」、
#       「✗ 自检：--recheck 同一条」、「✗ 自检：整轮里一个变体跑过进度间隔」、「✗ 自检：r2 整轮」、「✗ 自检：r3 整轮」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」、
#       「✗ 自检：r4 整轮」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」、「✗ 自检：r5：--gate r2 整轮」
#   E165_KILL_BOUNDS_BREAK=unknownitemunlisted  --item 写错时不列可点名的变体（只说「--list-items 看」）
#       「--item 写了不认识的变体名应当退 2、列出可点名的 6 项、一次不编」
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
# 下面九项外面打开时，自证的每一趟都带着它（drive() 把外面给的并进那一趟自己点名的；r2 那四项照旧只由 r2 那几趟带），红在下面点名的项；
# 自证里对照那几例自己打开同一项（证明前一例的红来自那一处检查），外面打开时那几例本来就开着、不另红：
#   E165_KILL_BOUNDS_BREAK=noanchorcheck  不核锚点、命中 0 次的那条照「改坏」（什么都没改）跑
#       「锚点命中 0 次应当报「锚点腐化」退 3」（实际退 0）、「锚点腐化时一次都不许编、不许写产物」（实际编了 7 次）、
#       「弄坏开关 recoveryonly 开着时文件列是 lib.rs 的那一条应当由红转「锚点腐化」退 3（证明整轮那一格的红来自按第二列改文件）」（实际退 0）
#   E165_KILL_BOUNDS_BREAK=reuseignored  plan 答 reuse 的项也照现判（不取库里的行，判完照 record：哨兵按「键相同却现判」判红）
#       「续跑应当只编 5 次（基线 2、没判的变体 3）」（实际 7）、「续跑：库里复用的三项应当各打一行 reused=1」、「全部复用的一趟应当一次不编」（实际编 7 次、退 1）
#       连带红的格：「✗ 自检：续跑应当退 0」、「✗ 自检：续跑补齐之后要写收尾行」、「✗ 自检：续跑补齐之后照样当场钉、当场核」、「✗ 自检：账写完而钉表没钉时驱动当场核钉全」、
#       「✗ 自检：弄坏开关 pincheckskipped 开着时」、「✗ 自检：判过的一对重交同一条命令应当全部复用、照钉照核」、「✗ 自检：复用那六格」；
#       复用那六格里红的：「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「forced-still-reused-red」
#   E165_KILL_BOUNDS_BREAK=finishignored  不认 long_run_item_verdicts.py finish 的红（复用哨兵红、键中途变了照写收尾行、照退 0）
#       「弄坏开关 reuseignored 开着时应当每个变体都重编、复用哨兵判红退 1」（实际退 0）、「复用那六格应当全过」；
#       复用那六格里红的：「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
#   E165_KILL_BOUNDS_BREAK=recorddeferred  现判的项判完不当场 record，等全部判完、finish 之前一并写
#       「续跑应当只编 5 次（基线 2、没判的变体 3）」（实际 7：被停下的那一趟一项都没入库）、「续跑：库里复用的三项应当各打一行 reused=1」
#   复用那几格与续跑那几格另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；随环境带进每一趟，打开后复用那六格红在它点名的同名格，
#       library-unread 另红续跑那几格）。
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
#       「整轮应当编 7 次（基线 2、变体 5）」（实际 4）、「续跑应当退 0」（实际 3）、「续跑应当只编 5 次（基线 2、没判的变体 3）」（实际 2）、
#       「续跑之后每个变体恰好一行完整的」（实际 tsv-5、Z1 各 0 行）、「续跑补齐之后要写收尾行」、「全部复用的一趟应当一次不编」（实际退 3）、
#       「弄坏开关 reuseignored 开着时应当每个变体都重编、复用哨兵判红退 1」（实际退 3）、
#       「弄坏开关 recorddeferred 开着时被停下的那一趟一项都没入库，重交应当全部重编」（实际 4）、「r2 整轮应当退 0」（实际 3）
#       连带红的格：「✗ 自检：r2：第一段产物不在时 V7 的比对」、「✗ 自检：r3 整轮：收尾行」、「✗ 自检：r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1…」、「✗ 自检：r3：假变体 KJ1 在 (1, 2)、KJ2 在 (1, 0) 与基线不同」、「✗ 自检：r3：假变体的 KA 读数不等于钉值」、「✗ 自检：r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红」、「✗ 自检：r3：变异名选不到的那一行」、「✗ 自检：r3：第二段产物不在时 TV7 的比对」、「✗ 自检：r4 整轮：收尾行」、「✗ 自检：r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1…」、「✗ 自检：r4：假变体 KJ1T 在 (1, 2, 无, 罩着)、KJ2 在 (1, 0, RN, 罩着) 与基线不同」、「✗ 自检：r4：假变体只在 KG 与基线不同（读数换成它自己那一列）」、「✗ 自检：r4：假变体的 KG 读数不等于它那一列」、「✗ 自检：r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红」、「✗ 自检：r4：变异名选不到的那一行」、「✗ 自检：r4：第一段产物不在时 TV7 的比对」、「✗ 自检：判过的一对重交同一条命令应当全部复用、照钉照核」、「✗ 自检：弄坏开关 pincheckskipped 开着时」、「✗ 自检：弄坏开关 r3kapinunchecked 开着时同一趟」、「✗ 自检：弄坏开关 r4kgpinunchecked 开着时同一趟」、「✗ 自检：整轮写完账与产物要当场钉进钉表、再核钉全：红例账里有 1 行反例，钉表」、「✗ 自检：续跑补齐之后照样当场钉、当场核：同一批红例已钉，钉表」、「✗ 自检：账写完而钉表没钉时驱动当场核钉全」、「✗ 锚点腐化：Z1 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」、「✗ 锚点腐化：tsv-2 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」、『✗ 锚点腐化：tsv-3 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次』、「✗ 锚点腐化：tsv-5 的原文在副本里的 crates/singlefs-core/src/recovery.rs 命中 0 次」
# 每项一行与总计（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」）：三套整轮（不给 --profile、--profile r3、--profile r4）都往 stdout 打，
#   每个点名的变体（含基线）一行 E165_KILL_BOUNDS_ITEM item=<变体> exit=<0 得出结果、1 没得出> judged=<这一趟现判 1，否则 0> red=<那一行的结局是 red 记 1，否则 0>
#   reused=<判定库里键相同、判绿，这一趟取库里的行、没跑记 1，否则 0> seconds=<这一项的挂钟>；这一趟跑的做完时打，库里复用的开跑前打。
#   整轮收尾（中途停机退出的也算）末行 E165_KILL_BOUNDS_TOTAL dispatched=<派出几项> collected=<收回几项> items=<打了几行> judged= red= reused= failed= seconds=；
#   派出与收回对不上（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）先打 ✗ 再打总计，本来退 0 的改退 1。
#   --list-items、--select-rows、--measure、--clean 不打。判定复用经 prune-server 里的按项判定（「按项判定」一段），不设 --full。
# 按项判定（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：三套整轮每个变体（含基线）一条判定，经
#   scripts/experiment-apparatus/long_run_item_verdicts.py 的 plan / record / finish 存进 prune-server 的 apparatus-items 族，种类 e165_replay_bounded_enumeration
#   （判定库只有那一份，本地不建库目录；产物与草稿目录都不是复用来源）。--recheck、--list-items、--select-rows、--measure、--clean 不读不写判定库。
#   键（long_run_item_verdicts.py 文件头「键」，即 admission.py 的 compute_unit_key）：装置入口 crates/singlefs-verification/src/bin/e165_replay_bounded_enumeration.rs::main
#   在 --root 那棵树上的单元闭包（profile:release）、工具链、闭包真引到的依赖版本；judge: 判法模块 scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py
#   里这一套判一个变体的函数与写基线那一行的函数（JUDGE_FUNCTIONS：judge_variant 与 baseline_item_line，--profile r3、r4 各是带 _r3、_r4 的那两个；判法闭包在
#   脚本所在的仓上算，judge-root:）：闭包是起装置、读它的输出、按「红」的口径出产物行与红例账行的那几个函数、它们调到的单元与那份模块 import 时执行的语句；
#   建副本、改坏、编译、选行、钉红例、产物收尾与读写判定库的代码在这份驱动里，不进判法闭包，它们读的表（变异表、自造表、钉表、按项判定的登记表）也就不进判法键
#   （变异行原文经下面的 line:item= 进键）；自证起的那几趟换成假树里的 SELFTEST_JUDGE_ENTRY、不带 judge-root:，判法闭包在假树上算）；
#   line: 这一趟的档（--profile、--pins、--gate 与界参数）、这一套登记值的 sha256、这一项写进产物行与红例账的字段（变异行原文、文件、表行号、名字、标题，ITEM_KEY_FIELDS）。
#   改坏之后的源码不进闭包，变异行原文经 line: 进键；改坏那一处碰到模块集合（MODULE_SET_TOKENS：mod、#[path、include_str!、include_bytes!、include!）的变体拒算：
#   不进请求，照跑、不复用、不入库，收尾逐个报。基线是一项，留存产物（V7 / TV7 的比对对象）经 @input 按内容进它的键（不在也是一种取值）；每个变体 @after 基线。
#   值是这一项的产物行与红例账行（键排序的 JSON）；产物行里复跑命令那一段 rerun="…" 存成 rerun="<复跑命令>"，取回时照这一趟的 --output、--ledger 填回，
#   与现跑的一个字不差。得出结果的判绿（被测变体被抓到、记 outcome=red 的也是得出了结果）；驱动没得出结果的（装置没跑完而没有一个域红）record --red，
#   记红撤绿、下一趟现判。
#   一趟：开工核对与选行 → plan（每项打 LONG_RUN_ITEM 行）→ 有项要现判才编、跑开头基线（库里复用的基线也照跑一遍给变体比，与库里那一行不逐字相同退 5，不入库；
#   没点名的基线库里没有可复用的，照跑照记）→ 现判的项判完当场 record → 收尾基线 → finish（复用哨兵、键中途变了，红了退 1、产物不收尾）→
#   产物从头写：config 行、点名的变体照变体的次序落行（库里复用的取库里的）、全都有行才写收尾三行。一项都不用现判时开头与收尾的基线都不编不跑，
#   收尾行要的基线行数取库里基线那一行的 result_rows。中途停下：挪走没收尾的产物与红例账（--output、--ledger 已经存在退 2），重交同一条命令：判完的已在库里，只补没判的。
#   成功那句报「按项判定 现判 N、复用 M、拒算 R」（R 含模块集合拒算的）。服务连不上退 75。状态目录在 <草稿目录>/kill-bounds/item-verdicts/，每一趟新建、跑完删。
#   只供自证：E165_KILL_BOUNDS_SELFTEST_KILL_AFTER=<项>（自证起的那一趟才认）判完这一项、record 之后整个进程被 SIGKILL，造「中途停下」；
#   E165_KILL_BOUNDS_SELFTEST_SET_OUTPUT_ASIDE=1（自证起的那一趟才认）已经存在的 --output 与 --ledger 先挪成 <名>.previous 再判（同一条命令重交几趟的那几格）。
# 进度：跑着的时候每过 PROGRESS_INTERVAL_SECONDS 秒（300，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」定的进度间隔）往 stderr 打一行
#   E165_KILL_BOUNDS_PROGRESS：挂钟、变体做完几个（共几个），在做的每一项在哪一步（建副本与编译、跑 --bounds segment2、在哪个 域:N 上找红例或单条重放）、
#   这一步与这一项各跑了多久。开头与收尾的基线也算在做的项。
#   只供自证：E165_KILL_BOUNDS_PROGRESS_INTERVAL=<秒> 换掉间隔，只在自证起的那一趟（E165_KILL_BOUNDS_SELFTEST_ROOT 也设了）认。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# 弄坏开关只在 --selftest 里认（文件头「弄坏开关」）：命令行第一个参数不是 --selftest、又不是自证起的子进程而下面的变量有一个非空（不管值是什么），
# 在参数解析之前退 2、点名它只在自证里认，一步都没做
for break_variable in E165_KILL_BOUNDS_BREAK; do
  if [[ -n "${!break_variable:-}" && "${1:-}" != --selftest && -z "${E165_KILL_BOUNDS_SELFTEST_ROOT:-}" ]]; then
    echo "  ✗ $break_variable 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证，一步都没做" >&2
    echo "     → 怎么办：去掉 $break_variable 再跑；要证红就跑 $break_variable=<项> bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --selftest" >&2
    exit 2
  fi
done
E165_KILL_BOUNDS_SELF_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
export E165_KILL_BOUNDS_SELF_PATH
exec python3 - "$@" <<'PYTHON'
import atexit
import concurrent.futures
import hashlib
import importlib.util
import io
import json
import os
import re
import shutil
import signal
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
import lib_repository_copy as repository_copy  # noqa: E402   拷副本的范围与副本里起命令的环境（scripts/hosts/lib_repository_copy.py）
sys.path.insert(0, SCRIPT_DIRECTORY)
# 判一个变体的那几个函数在判法模块里（按项判定的 judge: 指它们，文件头「按项判定」）；BREAK 是两边共用的那一份，下面照环境填
from e165_replay_mutation_kill_bounds_variant_judgement import (  # noqa: E402
    BIN_NAME, BREAK, DEVICE_PROFILE_ARGUMENTS, J6_ROWS, J6QN_ROWS, R3_ALL_DOMAINS, R3_BOUNDS_ARGUMENTS, R3_DEVICE_PROFILE_ARGUMENTS,
    R4_ALL_DOMAINS, R4_DEVICE_PROFILE_ARGUMENTS, R4_GATE_R2_ARGUMENTS,
    CLAUSE_CRITERIA, J7_ROWS, R3_CRITERIA, R3_LEGAL_DOMAINS, R4_BOUNDS_ARGUMENTS, R4_CRITERIA, R4_LEGAL_DOMAINS,
    baseline_item_line, baseline_item_line_r3, baseline_item_line_r4, device_bounds_arguments, judge_variant, judge_variant_r3, judge_variant_r4,
    mark, parse_row, result_rows, retained_comparison_text, rerun_command, rerun_command_r3, rerun_command_r4, run_captured, run_segment_two,
    violation_lines_by_place)
DEFAULT_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
RECOVERY_PATH = "crates/singlefs-core/src/recovery.rs"
LIB_PATH = "crates/singlefs-replay-prefix/src/lib.rs"
SOURCE_FILES = (RECOVERY_PATH, LIB_PATH)     # 变异改的文件取变异行第二列，只认这两份（r1 9.1）
TABLE_PATH = "crates/mutations.tsv"
CUSTOM_TABLE_PATH = "scripts/mutation/tables/e165_replay_bounded_enumeration_custom.tsv"
TAGS_TABLE_PATH = "scripts/mutation/tables/e165_replay_bounded_enumeration_tags.tsv"   # r1 9.3：每个变体在两版源码上的判定标签集合与依据
FIXED_COPY_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain", "rust-toolchain.toml", "crates")   # 弄坏开关 fixedcopylist 下照拷的那几样
WORK_DIRECTORY_NAME = "kill-bounds"
# crates/mutations.tsv 的登记值在第一段收尾之前填不出（第一段要往表里加装置的行）：占位串让开工核对一律停在 RH1，填了真值才跑得起来。
TABLE_SHA256_PENDING = "pending-segment1-frozen-root"
# 登记 r2（research/prompts/e165-r2-prereg.md）9.4 第 1 条：两份源文件的 sha256 取第三节记的值（第一段不许动这两份，SH1，冻结的根上应当相同）、
# crates/mutations.tsv 的 sha256 取第一段收尾冻结的根上的值（没填之前是占位串）、9.2 的函数行区间取在工作区上现算的那一张；
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
EXIT_OK, EXIT_INCOMPLETE, EXIT_USAGE, EXIT_ANCHOR, EXIT_HALT_H1, EXIT_VOID_V6, EXIT_VOID_V7, EXIT_NOT_PROFILE_R2 = 0, 1, 2, 3, 4, 5, 6, 7
EXIT_PIN_INCOMPLETE = 8   # 钉红例（文件头「钉红例」）：当场追加或 --check 没过
# 钉红例用的那一份：追加与 --check 的写法在它文件头
PIN_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "e165-pin-red-examples.py")
BREAK_MODES = ("noanchorcheck", "reuseignored", "finishignored", "recorddeferred", "progressnever", "byrownumber", "nodupcheck", "nolocationcheck", "recoveryonly",
               "j6qnfromj6", "noprofile", "noprofilecheck", "zfirstfour",
               "r3itemsmissing", "r3firstredswap", "r3kapinunchecked", "r3noprofile",
               "r4itemsmissing", "r4firstredswap", "r4kgcountsasred", "r4kgpinunchecked", "r4noprofile",
               "r5pinsignored", "r4gateignored", "owntablesplit", "defaultone",
               "pincheckskipped", "retainedmissingred", "itemcountsdropped", "itemlinedropped", "unknownitemunlisted",
               "fixedcopylist", "copyconfigurationunset", "outputoverwritten", "judgeoffthetree", "ledgeroverwritten", "judgewholedriver")
# 按项判定那一层的几项（文件头「弄坏开关」）：复用那六格只把外面开着的这几项带进去
ITEM_VERDICT_BREAK_MODES = ("reuseignored", "finishignored", "recorddeferred", "judgeoffthetree")
# 外面打开时自证的 r2 那几趟原样传进去的四项（文件头「弄坏开关」）
R2_BREAK_MODES = ("j6qnfromj6", "noprofile", "noprofilecheck", "zfirstfour")
# 外面打开时自证的 r3 那几趟原样传进去的四项（登记 r3 9.3 第 6 条的表）；别的趟不带
R3_BREAK_MODES = ("r3itemsmissing", "r3firstredswap", "r3kapinunchecked", "r3noprofile")
# 外面打开时自证的 r4 那几趟原样传进去的五项（登记 r4 9.3 第 6 条的表）；别的趟不带
R4_BREAK_MODES = ("r4itemsmissing", "r4firstredswap", "r4kgcountsasred", "r4kgpinunchecked", "r4noprofile")
# ── r3 档（登记 r3 9.3，第三段）。不给 --profile 是 r2 档，上面那一套一个字不动。
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
# 都是第一段根（/tmp/claude-1000/e165-r5-segment1/root，清单 root-manifest-at-start.sha256 的 sha256 95237ecf…）上现算的：两份源文件与 crates/mutations.tsv 的整份 sha256，
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
# 基线 family=root_choice_faults 的 name=config 行要带的 R2 闸字段与值：5-甲 的装置在 --gate r2 时实际写的（产物 R5_GATE_R2_PINS["segment1_product"] 第一行的 witness_gate=…）
R4_GATE_R2_CONFIG_FIELD = ("witness_gate", "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots")
# 5-乙 跑在 5-甲 的根上（/tmp/claude-1000/e165-r5-segment3/root5：第一段根盖上 5-甲 改过的装置与 crates/mutations.tsv，登记修1.十一「根、清单、在哪编」）：
# 变异表 sha256 是那份根上的 crates/mutations.tsv 的；TV7 的比对对象是 5-甲 的产物（取脚本所在的仓的 research/results/，同别的几套是路径名）。
# 两份源文件与两个函数区间仍是第一段根上的（5-甲 不改它们；开工核对时对着根现判，对不上停在 RH1）。
R5_GATE_R2_TABLE_SHA256 = "c9a03eb4af85a8e4c7206dfa3609593390516b80ca52be7090aced5696e84046"
R5_GATE_R2_PINS = dict(R5_PINS["r4"], table_sha256=R5_GATE_R2_TABLE_SHA256, segment1_product=R5_PRODUCT_PREFIX + "segment5-r2-gate-root-choice-faults.out")
BREAK.update(filter(None, os.environ.get("E165_KILL_BOUNDS_BREAK", "").split(",")))   # 判法模块里的那一份（两边共用），照环境填
for mode in BREAK:
    if mode not in BREAK_MODES:
        sys.exit(f"  ✗ 不认识的弄坏开关：{mode}\n  → 怎么办：E165_KILL_BOUNDS_BREAK 只认 {'、'.join(BREAK_MODES)}")
    print(f"  ! 弄坏开关开着：{mode}（只给自证用，这一次的结果不算数）", file=sys.stderr)


def default_jobs_and_threads(jobs, threads, environment=os.environ):
    """没给 --jobs 是 1；没给 --threads 是 N ÷ jobs（向下取整，至少 1），N 是 SINGLEFS_THREAD_CAP，没设取登记的重活默认并行度。"""
    if "defaultone" in BREAK:   # 弄坏开关：取 1 × 1
        return jobs or 1, threads or 1
    jobs = jobs or 1
    return jobs, threads or max(1, default_heavy_threads(environment) // jobs)


class Stop(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


def unknown_item(name, names, listing):
    """--item 写了不认识的变体名：列出可点名的变体，退 2（文件头「每项一行与总计」那一条规则）。"""
    named = "" if "unknownitemunlisted" in BREAK else f"；可点名的 {len(names)} 项：{' '.join(names)}"
    return Stop(EXIT_USAGE, f"  ✗ 不认识的变体：{name}{named}\n  → 怎么办：--item 照可点名的名字写；{listing} 逐行列变体与它的表、行号")


ITEM_LINES = {"report": None}   # 这一趟整轮的那一份 ItemLines；文件末尾收尾时打总计


class ItemLines:
    """整轮每项一行与末行总计（文件头「每项一行与总计」）。打的时候由调用方拿着锁（work() 里那把），这里不另加锁。"""

    def __init__(self):
        self.dispatched = self.collected = self.lines = 0
        self.judged = self.red = self.reused = self.failed = 0
        self.dropped = False
        self.started = time.monotonic()
        ITEM_LINES["report"] = self

    def emit(self, name, exit_code, judged, red, reused, seconds):
        counts = "" if "itemcountsdropped" in BREAK else f" judged={judged} red={red} reused={reused}"
        print(f"E165_KILL_BOUNDS_ITEM item={name} exit={exit_code}{counts} seconds={seconds:.1f}", flush=True)
        self.lines += 1
        self.judged, self.red, self.reused, self.failed = self.judged + judged, self.red + red, self.reused + reused, self.failed + (1 if exit_code else 0)

    def reused_item(self, name, line):
        self.emit(name, 0, 0, int(parse_row(line).get("outcome") == "red"), 1, 0.0)

    def dispatch(self, count):
        self.dispatched += count

    def judged_item(self, name, record):
        if "itemlinedropped" in BREAK and not self.dropped and name != "baseline":
            self.dropped = True
            return
        self.collected += 1
        outcome = parse_row(record["lines"][0]).get("outcome") if record["lines"] else None
        self.emit(name, 1 if record["problem"] else 0, 0 if record["problem"] else 1, int(outcome == "red"), 0, record["timing"][2])

    def close(self, code):
        if self.collected != self.dispatched:
            print(f"  ✗ 派出 {self.dispatched} 项、收回 {self.collected} 项：有变体做完了没打出它那一行，这一趟的总计不算数")
            print("  → 怎么办：看上面每项一行少了哪个变体、它的日志在草稿目录 kill-bounds/ 下；修好之后挪走这一趟的产物（有红例账的连同它；--output、--ledger 已经存在的不收），再重跑同一条命令")
            code = code or EXIT_INCOMPLETE
        print(f"E165_KILL_BOUNDS_TOTAL dispatched={self.dispatched} collected={self.collected} items={self.lines} judged={self.judged} red={self.red} "
              f"reused={self.reused} failed={self.failed} seconds={time.monotonic() - self.started:.1f}", flush=True)
        return code


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
        # r3 模式的 --root 是第二段冻结根（只有 Cargo.toml、Cargo.lock、.cargo、crates/）：--measure 用的内存包装与第二段产物取脚本所在的仓（自证里取假仓）。
        self.script_root = root if selftest else DEFAULT_ROOT
        # r2 模式原来的冻结根自己带 research/（自造表、标签表、--measure 用的内存包装、第一段产物都从根里读）；--pins r5 的根是第一段根，只有 crates/ 那几样，
        # 这几样改取脚本所在的仓（登记 r5 第十一节「根、清单、在哪编」：驱动的自造表与 scripts 取脚本所在的仓）。
        self.research_root = self.script_root if pins == "r5" else root
        self.memory_cap_runner = os.path.join(self.research_root if profile == "r2" else self.script_root, "scripts/hosts/run-with-memory-cap.sh")
        # 这一趟本次未跑的比对（留存产物不在，retained_product_not_run 记），收尾那句逐项列出
        self.not_run = []

    def bounds_arguments(self):
        """每次整份跑装置的界参数（判法模块的 device_bounds_arguments）。"""
        return device_bounds_arguments(self)

    def pins_arguments(self):
        """复跑命令里要带的 --pins / --gate（照这一次给的原样写）。"""
        return (["--pins", self.pins] if self.pins else []) + (["--gate", self.gate] if self.gate else [])

    def command_environment(self):
        """副本里跑编出来的装置的环境：多机配置取脚本所在的仓那一份（environment_in_the_copy）。判法模块起装置时经它取，拷副本那一层不进判法闭包。"""
        return environment_in_the_copy(self.script_root)


DEVICE_SOURCE_PATH = "crates/singlefs-verification/src/bin/e165_replay_bounded_enumeration.rs"


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
    （门禁 crates-mutation-replay 与 prove-red.sh 同一份）；弄坏开关 owntablesplit 下自己切（四段以上就收）。"""
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
    rows, _malformed = _CRATES_MUTATION_ROWS[0].read_table(os.path.join(setup.root, TABLE_PATH))
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

COPY_ENTRIES_BY_ROOT = {}            # 每棵根只算一次闭包：一趟里根不变；几个变体并行建副本，用锁串起来
COPY_ENTRIES_LOCK = threading.Lock()


def copy_root_entries(root, destination):
    """把 root 照 scripts/hosts/lib_repository_copy.py 在它上面现算的共用清单与闭包拷进 destination（不在就建；文件头「做法」第 4 步）。
    闭包算不成、拷不成抛 Stop（退 1，点名哪一处）。弄坏开关 fixedcopylist：只拷 FIXED_COPY_ENTRIES。"""
    if "fixedcopylist" in BREAK:
        entries = FIXED_COPY_ENTRIES
    else:
        with COPY_ENTRIES_LOCK:
            if root not in COPY_ENTRIES_BY_ROOT:
                try:
                    COPY_ENTRIES_BY_ROOT[root] = repository_copy.copy_closure(root).entries
                except ValueError as error:   # ClosureRefused 是它的子类
                    raise Stop(EXIT_INCOMPLETE, f"  ✗ 副本的范围在 {root} 上算不成：{error}\n"
                                                "  → 怎么办：照 scripts/hosts/lib_repository_copy.py 文件头「闭包」修好那一处（根里缺编译期读的文件就先补进根）") from error
            entries = COPY_ENTRIES_BY_ROOT[root]
    os.makedirs(destination)
    try:
        repository_copy.copy_entries(root, destination, entries)
    except OSError as error:
        raise Stop(EXIT_INCOMPLETE, f"  ✗ 副本 {destination} 没拷成：{error}\n  → 怎么办：看 rsync 的报错（盘满、权限）") from error


def environment_in_the_copy(script_root, environment=None):
    """副本里起命令（编、跑副本里编出来的装置）的环境：多机配置取脚本所在的仓那一份（scripts/hosts/lib_repository_copy.py 的
    environment_for_commands_in_the_copy；--root 给的根与副本都不在 git 里，照它们的位置找不到配置）。弄坏开关 copyconfigurationunset：照原样交。"""
    if "copyconfigurationunset" in BREAK:
        return dict(os.environ if environment is None else environment)
    return repository_copy.environment_for_commands_in_the_copy(script_root, environment)


def prepare_and_build(setup, item, directory, threads, wrap_build_for_measure=False):
    """建副本、另存原件、改坏、编。返回 (二进制路径或 None, 说明, 编译挂钟秒)。"""
    if os.path.exists(directory):
        shutil.rmtree(directory)
    tree = os.path.join(directory, "tree")
    copy_root_entries(setup.root, tree)
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
    environment = environment_in_the_copy(setup.script_root)
    environment["CARGO_TARGET_DIR"] = os.path.join(directory, "target")
    environment["CARGO_BUILD_JOBS"] = str(threads)
    command = ["nice", "-n", "19"] + setup.cargo + ["build", "--release", "--offline", "-p", "singlefs-verification", "--bin", BIN_NAME]
    if wrap_build_for_measure:
        environment["RUN_WITH_MEMORY_CAP_KEY"] = "e165-kill-bounds measure build"
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


def judge_built_variant(setup, item, baseline, options, judge):
    """做一个变体：在这里建副本、改坏、编（prepare_and_build），起装置、比、出产物行与红例账行交判法模块的 judge（judge_variant、judge_variant_r3、
    judge_variant_r4；判完照交进去的 restore 还原并删掉副本）。r3、r4 选行没满足的（item["failure"]，登记 TH5）不建副本、不编，直接交 judge 记 not_compared。
    返回 {lines: 产物行, ledger: 红例账行, problem: 没得出结果的原因或 None, timing: (编译秒, 跑秒, 这一项的秒)}。"""
    if item.get("failure") is not None:
        judged = judge(setup, item, baseline, options, None, None, None, None)
        timing = (0.0, 0.0, 0.0)
    else:
        directory = os.path.join(options["work"], item["name"])
        started = time.monotonic()
        mark(options, item["name"], "建副本、改坏、编译")
        binary, build_problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"])
        judged = judge(setup, item, baseline, options, binary, build_problem, directory,
                       lambda: restore_and_remove(setup, directory, keep=False, target=target_of(item)))
        timing = (build_seconds, judged["run_seconds"], time.monotonic() - started)
    return {"lines": judged["lines"], "ledger": judged["ledger"], "problem": judged["problem"], "timing": timing}


def process_item(setup, item, baseline, options):
    """不给 --profile 那一套做一个变体（判法模块的 judge_variant）。"""
    return judge_built_variant(setup, item, baseline, options, judge_variant)


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
        raise Stop(EXIT_INCOMPLETE, f"  ✗ 基线没跑完或退出码不是 0（{completed.returncode}；日志 {directory}/run.log）\n  → 怎么办：先单跑基线看（撞内存上限的由 task-server 放大重排整条命令，不在这里判）")
    print(f"  基线 {name}：编译 {build_seconds:.1f} 秒，跑 {run_seconds:.1f} 秒，{len(rows)} 行", file=sys.stderr)
    return {"binary": binary, "rows": rows, "row_count": len(rows), "exit": completed.returncode, "directory": directory}


def retained_product_not_run(setup, clause, paths):
    """比对的对象是 research/ 下的留存产物（可以整个删掉）：有不在的，这一项比对本次未跑——逐个点名不在的产物，记进 setup.not_run、打一行「本次未跑」，
    环境里有 GATE_NOT_RUN_FILE 的另往那份文件追加一行；不当绿、不判红，交 True（调用方不比、比对那一栏写 not_run）。都在交 False（照旧比）。
    弄坏开关 retainedmissingred 下一律交 False（不在就判红或读不到）。"""
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


# ── r3 档（登记 r3 9.3，第三段）：整份装置输出的读法在判法模块里，这里只留 TV7 比对要的那一个。

def r3_normalised(rows):
    """TV7 比对用：E7RESULT 行去掉耗时字段（登记第六节「elapsed_seconds、E7TIMING 行除外」；E7TIMING 行本来不是 E7RESULT 行）。"""
    return [re.sub(r" elapsed_seconds=\S+", "", row) for row in rows]


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


def process_item_r4(setup, item, baseline, options):
    """--profile r4 那一套做一个变体（判法模块的 judge_variant_r4）。"""
    return judge_built_variant(setup, item, baseline, options, judge_variant_r4)


# ── 结果文件

def recorded_items(path):
    """产物里有完整一行（complete=yes 收尾）的变体 → 它的行：--recheck 拿来比的那一份产物，与整轮收尾读回这一趟自己写的产物。"""
    recorded = {}
    if not os.path.exists(path):
        return recorded
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    for line in content.split("\n"):
        if line.startswith("E7RESULT name=kill_bound ") and line.endswith(" complete=yes"):
            recorded.setdefault(parse_row(line)["item"], []).append(line)
    return recorded


def start_file(path, lines):
    """这一趟的产物（或红例账）从头写，先写开头那几行（config 行、账头）：--output 与 --ledger 已经存在的在开跑之前就拒了（refuse_existing_output；
    弄坏开关 outputoverwritten、ledgeroverwritten 下照收，这里整份换掉那一份）；产物与账都不是复用来源（文件头「按项判定」）。"""
    with open(path, "w", encoding="utf-8") as handle:
        for line in lines:
            handle.write(line + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def refuse_existing_output(setup, path, shown, ledger=None, ledger_shown=None):
    """三套整轮的 --output、不给 --profile 那一套的 --ledger 指到已经存在的文件：退 2、不编不跑、不写产物与账，那一份一个字不动
    （--recheck 读已有的产物与账，不走这里）。弄坏开关 outputoverwritten 下 --output 照收、ledgeroverwritten 下 --ledger 照收（start_file 整份换掉那一份）。
    只供自证：SET_OUTPUT_ASIDE_VARIABLE 设成 1（且是自证起的那一趟）时，先把已经存在的产物与账挪成 <名>.previous，再判（同一条命令重交几趟的那几格）。"""
    if setup.selftest and os.environ.get(SET_OUTPUT_ASIDE_VARIABLE) == "1":
        for existing in (path, ledger):
            if existing is not None and os.path.exists(existing):
                os.replace(existing, existing + ".previous")
    for option, existing, existing_shown, broken in (("--output", path, shown, "outputoverwritten"), ("--ledger", ledger, ledger_shown, "ledgeroverwritten")):
        if existing is not None and os.path.exists(existing) and broken not in BREAK:
            raise Stop(EXIT_USAGE, f"  ✗ {option} {existing_shown} 已经存在：这一趟不编不跑、不写产物，那一份没动\n  → 怎么办：换一个路径，或先挪走那一份")


def append_lines(path, lines):
    with open(path, "a", encoding="utf-8") as handle:
        for line in lines:
            handle.write(line + "\n")
        handle.flush()
        os.fsync(handle.fileno())


# ── 按项判定（文件头「按项判定」）：每个变体一项，经 long_run_item_verdicts.py 的 plan / record / finish 存进 prune-server 的 apparatus-items 族

ITEM_VERDICTS_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "long_run_item_verdicts.py")
SCRIPT_KEY = "scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh"   # 按项判定 plan 与复用格认的脚本名（弄坏开关 judgewholedriver 下判法也写它）
LIBRARY_KIND = BIN_NAME                                                            # 判定库的种类：装置的 bin 名，三套整轮共用，档写在 line: 里
APPARATUS_ENTRY = DEVICE_SOURCE_PATH + "::main"
ITEM_STATE_DIRECTORY_NAME = "item-verdicts"   # 这一趟的状态目录（计划、复用项取回的值、交给 record 的值），在 <草稿目录>/kill-bounds/ 下，跑完删
RERUN_PLACEHOLDER = 'rerun="<复跑命令>"'      # 存进库的产物行里复跑命令那一段换成它，取回时照这一趟的 --output、--ledger 填回
# 改坏那一处碰到模块集合的变体拒算：改坏之后的模块集合与没改坏的闭包不同，变异行原文进键罩不住
MODULE_SET_TOKENS = ("mod ", "#[path", "include_str!", "include_bytes!", "include!")
# 产物行与红例账行里写到的这一项的字段：都进这一项的键（line:item=）
ITEM_KEY_FIELDS = ("name", "table", "rows", "titles", "file", "old", "new", "failure", "registered_row", "label", "pin", "function")
EXIT_LIBRARY_UNAVAILABLE = 75
KILL_AFTER_VARIABLE = "E165_KILL_BOUNDS_SELFTEST_KILL_AFTER"   # 只供自证：判完这一项、record 之后整个进程被 SIGKILL（文件头「按项判定」）
SET_OUTPUT_ASIDE_VARIABLE = "E165_KILL_BOUNDS_SELFTEST_SET_OUTPUT_ASIDE"   # 只供自证：已经存在的 --output 与 --ledger 先挪走再判（refuse_existing_output）
# 自证起的那几趟的判法（文件头「按项判定」）：自证把这份文件写进假树，judge: 指它、不带 judge-root:，判法闭包在假树上算
SELFTEST_JUDGE_FILE = "scripts/experiment-apparatus/e165_kill_bounds_selftest_judge.py"
SELFTEST_JUDGE_ENTRY = SELFTEST_JUDGE_FILE + "::judge_variant"
SELFTEST_JUDGE_RESULT = '    return "red when a criterion differs from the baseline"\n'   # 自证「改了假树里的判法」那一格改的就是这一行
SELFTEST_JUDGE_TEXT = ('"""E165 第二段驱动自证的假判法：自证起的那几趟，按项判定的 judge: 指这里（判法闭包在假树上算，不在脚本所在的仓上收整份驱动）。"""\n\n\n'
                       "def judge_variant():\n" + SELFTEST_JUDGE_RESULT)


# 真跑的按项判定请求里的判法（文件头「按项判定」）：判法模块里这一套判一个变体的函数与写基线那一行的函数，judge: 逐个指它们
JUDGE_MODULE_KEY = "scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py"
JUDGE_FUNCTIONS = {"r2": (judge_variant, baseline_item_line), "r3": (judge_variant_r3, baseline_item_line_r3), "r4": (judge_variant_r4, baseline_item_line_r4)}


def judge_tokens(setup):
    """按项判定请求里的判法：真跑是判法模块里这一套的 JUDGE_FUNCTIONS，判法闭包在脚本所在的仓上算（judge-root:）；
    自证起的那几趟换成假树里的 SELFTEST_JUDGE_ENTRY、不带 judge-root:。
    弄坏开关 judgewholedriver：真跑的判法写整份驱动（judge:<本脚本>）；judgeoffthetree：自证的判法另带 judge-root:..，
    闭包换到假树的上一层去算（那里没有假判法，每一项拒算）。"""
    if not setup.selftest:
        if "judgewholedriver" in BREAK:
            return [f"judge:{SCRIPT_KEY}", f"judge-root:{DEFAULT_ROOT}"]
        return [f"judge:{JUDGE_MODULE_KEY}::{function.__name__}" for function in JUDGE_FUNCTIONS[setup.profile]] + [f"judge-root:{DEFAULT_ROOT}"]
    return [f"judge:{SELFTEST_JUDGE_ENTRY}"] + (["judge-root:.."] if "judgeoffthetree" in BREAK else [])


def touched_module_set_tokens(item):
    text = (item.get("old") or "") + (item.get("new") or "")
    return [token.strip() for token in MODULE_SET_TOKENS if token in text]


def item_verdicts_command(arguments):
    """起 long_run_item_verdicts.py 的一条子命令（plan、record、finish、自证用的几条）：交 (退出码, 合起来的输出)。"""
    completed = subprocess.run([sys.executable, ITEM_VERDICTS_SCRIPT, *arguments], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    return completed.returncode, completed.stdout


def item_verdicts_failure(step, code, output):
    """按项判定那一步没成：75（prune-server 不可用）与 2（用法错）照交，别的按 1 交。"""
    return Stop(code if code in (EXIT_LIBRARY_UNAVAILABLE, EXIT_USAGE) else EXIT_INCOMPLETE,
                f"{output.rstrip()}\n  ✗ 按项判定的 {step} 没成（long_run_item_verdicts.py 退 {code}）\n"
                "  → 怎么办：退 75 是 prune-server 不可用：python3 prune-server/control check 看服务，修好之后挪走这一趟写了一半的产物（有红例账的连同它；--output、--ledger 已经存在的不收）、重交同一条命令（报给主 agent）；别的照上面它打的出路改")


def item_requests_text(setup, planned, retained_paths):
    """请求正文（long_run_item_verdicts.py 文件头「请求文件」）：每项一行，基线之外的每项 @after 基线；留存产物经 @input 进基线的键。"""
    registered = hashlib.sha256(json.dumps(setup.registered, ensure_ascii=False, sort_keys=True).encode("utf-8")).hexdigest()
    common = [APPARATUS_ENTRY, "profile:release", *judge_tokens(setup),
              f"line:mode=profile {setup.profile} pins {setup.pins or 'none'} gate {setup.gate or 'none'} bounds {' '.join(setup.bounds_arguments())}",
              f"line:registered={registered}"]
    rows = []
    for item in planned:
        described = json.dumps({field: item.get(field) for field in ITEM_KEY_FIELDS}, ensure_ascii=False, sort_keys=True)
        rows.append("\t".join([item["name"], *common, "line:item=" + described]))
        if item["name"] == "baseline":
            rows += [f"@input\tbaseline\t{os.path.basename(path)}\t{path}" for path in retained_paths]
        else:
            rows.append(f"@after\t{item['name']}\tbaseline")
    return "".join(row + "\n" for row in rows)


class ItemVerdicts:
    """一趟整轮的按项判定（文件头「按项判定」）：构造时 plan（算键、查库），库里复用的项取回它的产物行与红例账行；现判的项判完 record，收尾 finish。
    基线不管点没点名都在计划里（变体 @after 它）。改坏碰到模块集合的变体不进计划：照跑、不复用、不入库。"""

    def __init__(self, setup, options, items, wanted, rerun_of, retained_paths):
        self.setup, self.rerun_of = setup, rerun_of
        self.by_name = {item["name"]: item for item in items}
        self.wanted_names = [item["name"] for item in wanted]
        self.directory = os.path.join(options["work"], ITEM_STATE_DIRECTORY_NAME)
        shutil.rmtree(self.directory, ignore_errors=True)
        os.makedirs(os.path.join(self.directory, "values"))
        atexit.register(shutil.rmtree, self.directory, True)
        self.state = os.path.join(self.directory, "plan")
        self.module_set = {item["name"]: touched_module_set_tokens(item) for item in wanted if touched_module_set_tokens(item)}
        planned = [item for item in items if item["name"] == "baseline" or (item["name"] in self.wanted_names and item["name"] not in self.module_set)]
        requests = os.path.join(self.directory, "requests.tsv")
        with open(requests, "w", encoding="utf-8") as handle:
            handle.write(item_requests_text(setup, planned, retained_paths))
        code, output = item_verdicts_command(["plan", self.state, "--script", SCRIPT_KEY, "--kind", LIBRARY_KIND, "--root", setup.root, "--requests", requests])
        print(output, end="", flush=True)
        if code != 0:
            raise item_verdicts_failure("plan", code, output)
        self.decisions = {}
        with open(os.path.join(self.state, "decisions.tsv"), encoding="utf-8") as handle:
            for line in handle:
                name, _tab, decision = line.rstrip("\n").partition("\t")
                self.decisions[name] = decision
        self.reused = {}
        if "reuseignored" not in BREAK:   # 弄坏开关：plan 答 reuse 的也照现判
            self.reused = {name: self.reused_record(name) for name, decision in self.decisions.items() if decision == "reuse"}
        self.to_run = [item for item in wanted if item["name"] not in self.reused]
        # 没点名的基线库里没有可复用的（要现判、拒算）：照跑、照记，强制现判抽中它时也是这一支
        self.baseline_pending = "baseline" not in self.reused and "baseline" not in self.wanted_names
        self.deferred = []
        self.summary = "取不出"

    def rerun_text(self, name):
        return f'rerun="{self.rerun_of(self.by_name[name])}"'

    def reused_record(self, name):
        """库里复用的一项：取回的值是 {"lines", "ledger"}，复跑命令那一段照这一趟填回。"""
        try:
            with open(os.path.join(self.state, "outputs", name + ".out"), encoding="utf-8") as handle:
                value = json.load(handle)
            lines, ledger = list(value["lines"]), list(value["ledger"])
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise Stop(EXIT_INCOMPLETE, f"  ✗ 判定库里 {name} 那一项的值读不出产物行与红例账行（{error}）\n"
                                        "  → 怎么办：这把键的值不是这份驱动写的那一种；查是谁往 apparatus-items 族这个种类里写了别的东西，删掉那一条再起同一条命令") from error
        return {"lines": [line.replace(RERUN_PLACEHOLDER, self.rerun_text(name)) for line in lines], "ledger": ledger, "problem": None, "timing": (0.0, 0.0, 0.0)}

    def value_text(self, name, record):
        """存进库的值：产物行（复跑命令那一段换成占位）与红例账行，键排序的 JSON，同一份结果写出来逐字节相同。"""
        rerun = self.rerun_text(name)
        lines = []
        for line in record["lines"]:
            if line.count(rerun) != 1:
                raise Stop(EXIT_INCOMPLETE, f"  ✗ {name} 的产物行里复跑命令那一段不是恰好一处，存不进判定库：{line[:200]}\n  → 怎么办：看判法模块里这一套的 rerun_command 与 judge_variant 的 tail")
            lines.append(line.replace(rerun, RERUN_PLACEHOLDER))
        return json.dumps({"ledger": record["ledger"], "lines": lines}, ensure_ascii=False, sort_keys=True) + "\n"

    def report_reused(self, item_lines):
        """库里复用的点名项开跑前各打一行（文件头「每项一行与总计」）。"""
        for name in self.wanted_names:
            if name in self.reused:
                item_lines.reused_item(name, self.reused[name]["lines"][0])

    def record(self, name, record):
        """现判或拒算的项判完当场写进库：得出结果的判绿，驱动没得出结果的（record 的 problem）记红。复用的项不调；模块集合拒算的不入库。"""
        if name in self.module_set or name not in self.decisions:
            return
        if "recorddeferred" in BREAK:   # 弄坏开关：判完不当场写，finish 之前一并写
            self.deferred.append((name, record))
        else:
            self.write(name, record)
        if self.setup.selftest and os.environ.get(KILL_AFTER_VARIABLE) == name:
            os.kill(os.getpid(), signal.SIGKILL)

    def write(self, name, record):
        red = record["problem"] is not None
        value = os.path.join(self.directory, "values", name + ".json")
        with open(value, "w", encoding="utf-8") as handle:
            handle.write("" if red else self.value_text(name, record))
        code, output = item_verdicts_command(["record", self.state, name, "1" if red else "0", value] + (["--red"] if red else []))
        if code != 0:
            raise item_verdicts_failure(f"record（项 {name}）", code, output)

    def baseline_ran(self, baseline_line):
        """开头基线跑完：库里复用的基线要与这一次的逐字相同（键相同而基线变了，退 5）；没点名、库里又没有可复用的基线，当场记。"""
        stored = self.reused.get("baseline")
        if stored is not None:
            if stored["lines"][0] != baseline_line:
                raise Stop(EXIT_VOID_V6, f"  ✗ 这一次的基线与判定库里同一把键的基线不同\n     库里的：{stored['lines'][0][:240]}\n     这次：{baseline_line[:240]}\n"
                                         "  → 怎么办：键相同而基线变了，判法或装置读了键之外的输入；这一趟变体一个都没判，交主 agent 查")
        elif self.baseline_pending:
            self.record("baseline", {"lines": [baseline_line], "ledger": [], "problem": None})

    def finish(self):
        """收尾：写 recorddeferred 压下的那几项、起 finish、报模块集合拒算的项；交 True（没红）或 False（复用哨兵红、键中途变了：产物不收尾）。"""
        for name, record in self.deferred:
            self.write(name, record)
        code, output = item_verdicts_command(["finish", self.state])
        print(output, end="", flush=True)
        if code > 1:
            raise item_verdicts_failure("finish", code, output)
        for name, tokens in self.module_set.items():
            print(f"  · {name}：拒算（改坏那一处碰到模块集合：{'、'.join(tokens)}；照跑、不复用、不入库）")
        found = re.findall(r"^LONG_RUN_ITEMS .* 现判 (\d+)、复用 (\d+)、拒算 (\d+)$", output, re.M)
        if found:
            judged, reused, refused = (int(value) for value in found[-1])
            self.summary = f"现判 {judged}、复用 {reused}、拒算 {refused + len(self.module_set)}"
        if code == 1 and "finishignored" not in BREAK:   # 弄坏开关：finish 的红不认
            print(f"  ✗ 按项判定收尾判红（复用哨兵或键在这一趟中途变了，见上面 ✗ 那几行；{self.summary}）：这一趟的产物不收尾")
            print("  → 怎么办：照上面那几行的出路办，挪走这一趟没收尾的产物（有红例账的连同它；--output、--ledger 已经存在的不收），再起同一条命令")
            return False
        return True


def opening_baseline(setup, options, verdicts, to_run, run_checks, line_of, retained_absent):
    """开头基线（文件头「做法」第 3 步）：这一趟有项要跑才编、才跑、才核（库里复用的项不要基线）；交 (基线, 基线那一行)。
    一项都不跑时交一份替身、不编不跑：收尾行要的基线行数取库里基线那一行的 result_rows，留存产物在不在照现查（不在的照报本次未跑）。"""
    if to_run or (verdicts is not None and verdicts.baseline_pending):
        baseline = build_and_run_baseline(setup, options, "baseline")
        run_checks(baseline)
        baseline_line = line_of(baseline)
        if verdicts is not None:
            verdicts.baseline_ran(baseline_line)
        return baseline, baseline_line
    stored = verdicts.reused.get("baseline")
    row_count = int(parse_row(stored["lines"][0]).get("result_rows", "0")) if stored is not None else None
    return {"binary": None, "rows": None, "row_count": row_count, "verbatim_rows": "not_run" if retained_absent() else 0}, None


def judge_items_in_parallel(setup, options, items, wanted, to_run, verdicts, baseline, baseline_line, process, item_lines, write_product):
    """to_run 里的项按 --jobs 并行做（基线那一项就是开头基线那一行）：每项做完当场打它那一行、记进判定库；write_product 时照变体的次序落进这一趟的产物
    （库里复用的项拿库里的行；没得出结果的那一项与它后面的不落，等下一趟）。交 (results, problems)；有项抛 Stop 的，别的项做完之后抛先抛的那一个。"""
    results, problems = {}, []
    lock = threading.Lock()
    flushed = set()

    def flush_in_order():
        for item in items:
            name = item["name"]
            if name in flushed or item not in wanted:
                continue
            record = results.get(name) or (verdicts.reused.get(name) if verdicts is not None else None)
            if record is None or record.get("problem"):
                return
            append_lines(options["output"], record["lines"])
            if record["ledger"]:
                append_lines(options["ledger"], record["ledger"])
            flushed.add(name)

    def work(item):
        if item["name"] == "baseline":
            record = {"lines": [baseline_line], "ledger": [], "problem": None, "timing": (0.0, 0.0, 0.0)}
        else:
            record = process(setup, item, baseline, options)
        with lock:
            results[item["name"]] = record
            options["progress"].finished(item["name"])
            item_lines.judged_item(item["name"], record)
            build_seconds, run_seconds, total_seconds = record["timing"]
            print(f"  {item['name']}：编译 {build_seconds:.1f} 秒，跑 {run_seconds:.1f} 秒，合计 {total_seconds:.1f} 秒"
                  + (f"；没得出结果：{record['problem']}" if record["problem"] else ""), file=sys.stderr)
            with open(os.path.join(options["work"], "progress.log"), "a", encoding="utf-8") as handle:
                handle.write(f"{item['name']}\tbuild={build_seconds:.1f}\trun={run_seconds:.1f}\ttotal={total_seconds:.1f}\tproblem={record['problem']}\n")
            if record["problem"]:
                problems.append(f"{item['name']}：{record['problem']}")
            if verdicts is not None:
                verdicts.record(item["name"], record)
            if write_product and not record["problem"]:
                flush_in_order()

    halted = None
    with concurrent.futures.ThreadPoolExecutor(max_workers=options["jobs"]) as pool:
        item_lines.dispatch(len(to_run))
        futures = [pool.submit(work, item) for item in to_run]
        for future in concurrent.futures.as_completed(futures):
            try:
                future.result()
            except Stop as stop:
                halted = halted or stop
    if halted is not None:
        raise halted
    if write_product:
        with lock:
            flush_in_order()
    return results, problems


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


def process_item_r3(setup, item, baseline, options):
    """--profile r3 那一套做一个变体（判法模块的 judge_variant_r3）。"""
    return judge_built_variant(setup, item, baseline, options, judge_variant_r3)


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
    steps = []
    if not (setup.selftest and os.environ.get("E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND")):
        steps.append(("追加", ["--kill-bounds", product, ledger]))
    if "pincheckskipped" not in BREAK:
        steps.append(("核钉全（--check）", ["--check", "--kill-bounds", product, ledger]))
    said = {}
    for label, arguments in steps:
        command = [sys.executable, PIN_SCRIPT, "--root", setup.script_root, *arguments]
        environment = dict(os.environ)
        environment.pop("E165_PIN_RED_EXAMPLES_BREAK", None)   # 那一份的弄坏开关只给它自己的自证用，不从这里带进去
        completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=environment)
        lines = [line for line in completed.stdout.split("\n") if line.strip()]
        if completed.returncode != 0:
            shown = "\n".join("     " + line for line in lines[-PIN_LINE_LIMIT:])
            raise Stop(EXIT_PIN_INCOMPLETE, f"  ✗ 钉红例：{label}没过（{' '.join(command[1:])} 退 {completed.returncode}），这一趟的红例没钉全：\n{shown}\n"
                                            "  → 怎么办：照上面它打的出路办（同档冲突、按名字找不到变异、配对不上，写法在 scripts/experiment-apparatus/e165-pin-red-examples.py 文件头），"
                                            "改好之后挪走这一趟的产物与账（--output、--ledger 已经存在的不收）、重跑同一条命令：判过的变体从判定库复用、不重跑，产物与账照样从头写、照样钉")
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
            raise unknown_item(name, list(by_name), "--list-items")
    wanted = [by_name[name] for name in item_names] if item_names else items
    retained = [os.path.join(setup.research_root, product) for product in setup.registered["segment1_products"]]
    # --recheck 只拿新跑的行与给的那一份产物比，不读不写判定库、不写产物（文件头「用法」）
    recorded = recorded_items(options["output"]) if recheck else {}
    verdicts = None if recheck else ItemVerdicts(setup, options, items, wanted, lambda item: rerun_command(item, options), retained)
    to_run = list(wanted) if recheck else verdicts.to_run
    status_at_start = subprocess.run(["git", "-C", setup.root, "status", "--porcelain", "crates/"], stdout=subprocess.PIPE, text=True).stdout
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；判定库里复用 {len(wanted) - len(to_run)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")
    item_lines = ItemLines()   # 文件头「每项一行与总计」：库里复用的那几项开跑前各打一行
    if verdicts is not None:
        verdicts.report_reused(item_lines)

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()   # 守护线程：收尾基线跑完就停；中途 Stop 退出的，进程退出时随之结束

    def run_checks(baseline):
        check_the_profile_rows(baseline)
        check_against_segment_one(setup, baseline)

    baseline, baseline_line = opening_baseline(setup, options, verdicts, to_run, run_checks,
                                               lambda baseline: baseline_item_line(by_name["baseline"], baseline, options),
                                               lambda: retained_product_not_run(setup, "V7", retained))
    if not recheck:
        start_file(options["output"], [
            f"E7RESULT name=config experiment=E165 segment=2 registration=research/prompts/e165-r2-prereg.md "
            f"device_arguments={'+'.join([] if 'noprofile' in BREAK else DEVICE_PROFILE_ARGUMENTS) or 'none'} "
            f"bounds=segment2 items={len(items)} table_rows={len(selected_rows)} "
            f"custom_rows={sum(1 for item in items if item['table'] == CUSTOM_TABLE_PATH)} recovery_sha256={sha_at_start[RECOVERY_PATH]} "
            f"lib_sha256={sha_at_start[LIB_PATH]} widening=none{pins_config_text(setup)}"])
        start_file(options["ledger"], [LEDGER_HEADER])

    started = time.monotonic()
    results, problems = judge_items_in_parallel(setup, options, items, wanted, to_run, verdicts, baseline, baseline_line, process_item, item_lines, not recheck)
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；仓里的 crates/ 没变。开头基线没跑的（一项都不用现判）收尾基线也不跑，只核仓里的 crates/。
    if baseline["binary"] is not None:
        baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
        options["progress"].stop()
        same_baseline = baseline_at_end["rows"] == baseline["rows"]
        restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
        restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    else:
        options["progress"].stop()
        same_baseline = restored_start = restored_end = True
    status_at_end = subprocess.run(["git", "-C", setup.root, "status", "--porcelain", "crates/"], stdout=subprocess.PIPE, text=True).stdout
    sha_at_end = source_hashes(setup)
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if (not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"]
            or status_at_start != status_at_end or not (restored_start and restored_end)):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 V6：收尾基线与开头逐字相同={same_baseline}；仓里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… 结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；"
                                 f"`git status --porcelain crates/` 前后相同={status_at_start == status_at_end}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了 crates/，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同" if baseline["binary"] is not None else "  这一趟没有要现判的项，开头与收尾的基线都没编没跑；仓里 recovery.rs、lib.rs 与 crates/ 前后相同")

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

    if not verdicts.finish():
        return EXIT_INCOMPLETE
    recorded = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，挪走这一趟的产物与红例账（--output、--ledger 已经存在的不收），再重跑同一条命令（判过的从判定库复用，只现判没判绿的）")
    if missing:
        print(f"  这一趟的产物里还有 {len(missing)} 个变体没有行：{' '.join(missing)}；挪走这一趟的产物与红例账（--output、--ledger 已经存在的不收）、不带 --item 重跑同一条命令补齐（判过的从判定库复用）；按项判定 {verdicts.summary}")
        return EXIT_INCOMPLETE if problems else EXIT_OK
    outcomes = {}
    for item in items:
        if item["name"] != "baseline":
            outcome = parse_row(recorded[item["name"]][0])["outcome"]
            outcomes[outcome] = outcomes.get(outcome, 0) + 1
    closing = [
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={baseline['row_count']} "
        f"repository_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} repository_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"repository_lib_sha256_at_start={sha_at_start[LIB_PATH]} repository_lib_sha256_at_end={sha_at_end[LIB_PATH]} crates_git_status_unchanged=true copies_restored=true",
        f"E7RESULT name=verdict scope=segment2 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_implementation_anchor={outcomes.get('red_only_by_implementation_anchor', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} ledger_sha256={sha256_of_file(options['ledger'])}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"])
    pin_text = pin_red_examples(setup, options)
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}；{pin_text}；按项判定 {verdicts.summary}{not_run_suffix(setup)}")
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
            raise unknown_item(name, list(by_name), "--profile r3 --list-items")
    wanted = [by_name[name] for name in item_names] if item_names else items
    retained = os.path.join(setup.script_root, setup.registered["segment2_product"])
    verdicts = ItemVerdicts(setup, options, items, wanted, lambda item: rerun_command_r3(item, options), [retained])
    to_run = verdicts.to_run
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；判定库里复用 {len(wanted) - len(to_run)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")
    item_lines = ItemLines()   # 文件头「每项一行与总计」：库里复用的那几项开跑前各打一行
    verdicts.report_reused(item_lines)

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()

    def run_checks(baseline):
        check_the_profile_rows_r3(baseline)
        check_against_segment_two_product(setup, baseline)

    baseline, baseline_line = opening_baseline(setup, options, verdicts, to_run, run_checks,
                                               lambda baseline: baseline_item_line_r3(by_name["baseline"], baseline, options),
                                               lambda: retained_product_not_run(setup, "TV7", [retained]))
    start_file(options["output"], [
        f"E7RESULT name=config experiment=E165 segment=3 profile=r3 registration=research/prompts/e165-r3-prereg.md "
        f"device_arguments={'+'.join(([] if 'r3noprofile' in BREAK else R3_DEVICE_PROFILE_ARGUMENTS) + R3_BOUNDS_ARGUMENTS) or 'none'} "
        f"items={len(items)} table_rows={len(items) - 1} recovery_sha256={sha_at_start[RECOVERY_PATH]} lib_sha256={sha_at_start[LIB_PATH]} "
        f"table_sha256={sha256_of_file(os.path.join(setup.root, TABLE_PATH))} widening=none{pins_config_text(setup)}"])

    started = time.monotonic()
    _results, problems = judge_items_in_parallel(setup, options, items, wanted, to_run, verdicts, baseline, baseline_line, process_item_r3, item_lines, True)
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；两份源文件与变异表的 sha256 开头结尾相同（冻结根不是 git 仓，不看 git status）。
    # 开头基线没跑的（一项都不用现判）收尾基线也不跑，只核根里的两份源文件与变异表。
    if baseline["binary"] is not None:
        baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
        options["progress"].stop()
        same_baseline = baseline_at_end["rows"] == baseline["rows"]
        restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
        restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    else:
        options["progress"].stop()
        same_baseline = restored_start = restored_end = True
    sha_at_end = source_hashes(setup)
    table_unchanged = sha256_of_file(os.path.join(setup.root, TABLE_PATH)) == setup.registered["table_sha256"] if setup.registered.get("table_sha256") else True
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"] or not table_unchanged or not (restored_start and restored_end):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 TV9：收尾基线与开头逐字相同={same_baseline}；根里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… "
                                 f"结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；变异表没变={table_unchanged}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了根，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同" if baseline["binary"] is not None else "  这一趟没有要现判的项，开头与收尾的基线都没编没跑；根里 recovery.rs、lib.rs 与变异表前后相同")

    if not verdicts.finish():
        return EXIT_INCOMPLETE
    recorded = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，挪走这一趟的产物（--output 已经存在的不收），再重跑同一条命令（判过的从判定库复用，只现判没判绿的）")
    if missing:
        print(f"  这一趟的产物里还有 {len(missing)} 个变体没有行：{' '.join(missing)}；挪走这一趟的产物（--output 已经存在的不收）、不带 --item 重跑同一条命令补齐（判过的从判定库复用）；按项判定 {verdicts.summary}")
        return EXIT_INCOMPLETE if problems else EXIT_OK
    outcomes, pin_mismatches = {}, []
    for item in items:
        if item["name"] != "baseline":
            fields = parse_row(recorded[item["name"]][0])
            outcomes[fields["outcome"]] = outcomes.get(fields["outcome"], 0) + 1
            if fields.get("ka_pin") == "mismatch":
                pin_mismatches.append(item["name"])
    closing = [
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={baseline['row_count']} "
        f"root_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} root_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"root_lib_sha256_at_start={sha_at_start[LIB_PATH]} root_lib_sha256_at_end={sha_at_end[LIB_PATH]} copies_restored=true",
        f"E7RESULT name=verdict scope=segment3 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_kj0_or_kj3={outcomes.get('red_only_by_kj0_or_kj3', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} not_compared={outcomes.get('not_compared', 0)} "
        f"ka_pin_mismatches={len(pin_mismatches)} baseline_equals_segment2_product={retained_comparison_text(baseline, 'true')}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"])
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}；按项判定 {verdicts.summary}{not_run_suffix(setup)}")
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
            raise unknown_item(name, list(by_name), "--profile r4 --list-items")
    wanted = [by_name[name] for name in item_names] if item_names else items
    retained = os.path.join(setup.script_root, setup.registered["segment1_product"])
    verdicts = ItemVerdicts(setup, options, items, wanted, lambda item: rerun_command_r4(item, options), [retained])
    to_run = verdicts.to_run
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；判定库里复用 {len(wanted) - len(to_run)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")
    item_lines = ItemLines()   # 文件头「每项一行与总计」：库里复用的那几项开跑前各打一行
    verdicts.report_reused(item_lines)

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()

    def run_checks(baseline):
        check_the_profile_rows_r4(baseline)
        check_the_gate_rows_r4(setup, baseline)
        check_against_segment_one_product(setup, baseline)

    baseline, baseline_line = opening_baseline(setup, options, verdicts, to_run, run_checks,
                                               lambda baseline: baseline_item_line_r4(setup, by_name["baseline"], baseline, options),
                                               lambda: retained_product_not_run(setup, "TV7", [retained]))
    start_file(options["output"], [
        f"E7RESULT name=config experiment=E165 segment=2 profile=r4 registration=research/prompts/e165-r4-prereg.md "
        f"device_arguments={'+'.join(([] if 'r4noprofile' in BREAK else R4_DEVICE_PROFILE_ARGUMENTS) + setup.bounds_arguments()) or 'none'} "
        f"items={len(items)} table_rows={len(items) - 1} recovery_sha256={sha_at_start[RECOVERY_PATH]} lib_sha256={sha_at_start[LIB_PATH]} "
        f"table_sha256={sha256_of_file(os.path.join(setup.root, TABLE_PATH))} widening=none{pins_config_text(setup)}"])

    started = time.monotonic()
    _results, problems = judge_items_in_parallel(setup, options, items, wanted, to_run, verdicts, baseline, baseline_line, process_item_r4, item_lines, True)
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；两份源文件与变异表的 sha256 开头结尾相同（冻结根不是 git 仓，不看 git status）。
    # 开头基线没跑的（一项都不用现判）收尾基线也不跑，只核根里的两份源文件与变异表。
    if baseline["binary"] is not None:
        baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
        options["progress"].stop()
        same_baseline = baseline_at_end["rows"] == baseline["rows"]
        restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
        restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    else:
        options["progress"].stop()
        same_baseline = restored_start = restored_end = True
    sha_at_end = source_hashes(setup)
    table_unchanged = sha256_of_file(os.path.join(setup.root, TABLE_PATH)) == setup.registered["table_sha256"] if setup.registered.get("table_sha256") else True
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"] or not table_unchanged or not (restored_start and restored_end):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 TV9：收尾基线与开头逐字相同={same_baseline}；根里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… "
                                 f"结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；变异表没变={table_unchanged}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了根，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同" if baseline["binary"] is not None else "  这一趟没有要现判的项，开头与收尾的基线都没编没跑；根里 recovery.rs、lib.rs 与变异表前后相同")

    if not verdicts.finish():
        return EXIT_INCOMPLETE
    recorded = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，挪走这一趟的产物（--output 已经存在的不收），再重跑同一条命令（判过的从判定库复用，只现判没判绿的）")
    if missing:
        print(f"  这一趟的产物里还有 {len(missing)} 个变体没有行：{' '.join(missing)}；挪走这一趟的产物（--output 已经存在的不收）、不带 --item 重跑同一条命令补齐（判过的从判定库复用）；按项判定 {verdicts.summary}")
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
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={baseline['row_count']} "
        f"root_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} root_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"root_lib_sha256_at_start={sha_at_start[LIB_PATH]} root_lib_sha256_at_end={sha_at_end[LIB_PATH]} copies_restored=true",
        f"E7RESULT name=verdict scope=segment2 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_kj0_or_kj3={outcomes.get('red_only_by_kj0_or_kj3', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} not_compared={outcomes.get('not_compared', 0)} "
        f"ka_pin_mismatches={len(pin_mismatches)} kg_pin_mismatches={len(kg_mismatches)} baseline_equals_segment1_product={retained_comparison_text(baseline, 'true')}{options.get('gate_field', '')}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"])
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}；按项判定 {verdicts.summary}{not_run_suffix(setup)}")
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
        raise unknown_item(name, [item["name"] for item in items], "--list-items")
    directory = os.path.join(options["work"], "measure-" + name)
    binary, problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"], wrap_build_for_measure=True)
    peak = "没读到"
    # 峰值表的位置与读法只在 scripts/hosts/memory_peaks_lib.py（共享数据根下一份、只认本台的行；RUN_WITH_MEMORY_CAP_PEAKS 给了照它）
    sys.path.insert(0, os.path.join(DEFAULT_ROOT, "scripts", "hosts"))
    import memory_peaks_lib
    try:
        peaks_path = os.environ.get("RUN_WITH_MEMORY_CAP_PEAKS") or memory_peaks_lib.default_peak_table()
        row = memory_peaks_lib.read_peak_rows(peaks_path).get(memory_peaks_lib.command_key([], "e165-kill-bounds measure build"))
        if row is not None:
            peak = f"{row[0] / (1 << 20):.0f} MiB"
    except (OSError, ValueError) as error:   # PeakTableUnavailable 与 TableRowRefused 都是 ValueError
        peak = f"没读到（{error}）"
    print(f"量到：变体 {name}，{options['threads']} 线程冷编译（从改 recovery.rs 到编完 bin）挂钟 {build_seconds:.1f} 秒，"
          f"内存峰值（回收不掉的用量的峰值，不含页缓存）{peak}；编译{'过了' if binary else '没过：' + problem}")
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
    # 判定写进临时 prune-server：只在临时服务底下跑（long_run_item_verdicts.py selftest-service，即客户端 selftest_service_problem），不在就退 2、一格都不跑
    code, output = item_verdicts_command(["selftest-service"])
    if code != 0:
        print(output.rstrip())
        print("  ✗ 自检没跑：判定写进临时 prune-server，自证只在临时服务底下跑")
        print("  → 怎么办：照文件头「用法」--selftest 那一条的包装写法起（python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash <本脚本> --selftest）")
        return EXIT_USAGE
    failures = []
    checked = 0
    selftest_started = time.monotonic()

    def say_progress(text):
        # 自证要跑一阵：每判完一项（与每段长的开跑前）打一行、立刻刷出，看日志的人知道走到哪了（文件头「用法」--selftest）
        print(f"E165_KILL_BOUNDS_SELFTEST_PROGRESS 挂钟 {time.monotonic() - selftest_started:.0f} 秒，判完 {checked} 项（不过 {len(failures)} 项）；{text}", flush=True)

    def expect(condition, what, fix):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append((what, fix))
        say_progress(f"刚判完的一项{'过了' if condition else '不过'}：{what[:120]}")

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

    # 真跑的按项判定请求里判法那一段（item_requests_text 经 judge_tokens 写）：三套各写判法模块里判一个变体的函数与写基线那一行的函数、judge-root 是脚本所在的仓，
    # 不写整份驱动（整份驱动的判法闭包连到选行、钉红例与自证读的整张表，表改一行每一项的键都变）；那几个函数要真在判法模块里。开关 judgewholedriver 下判法写回整份驱动
    judge_module = "scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py"
    wanted_judges = {"r2": ["judge_variant", "baseline_item_line"], "r3": ["judge_variant_r3", "baseline_item_line_r3"], "r4": ["judge_variant_r4", "baseline_item_line_r4"]}
    written_judges = {}
    for profile in wanted_judges:
        request = item_requests_text(Setup(DEFAULT_ROOT, False, profile), [{"name": "baseline"}], [])
        written_judges[profile] = [token for token in request.rstrip("\n").split("\t") if token.startswith(("judge:", "judge-root:"))]
    in_module = sorted({os.path.realpath(function.__code__.co_filename) for functions in JUDGE_FUNCTIONS.values() for function in functions})
    import e165_replay_mutation_kill_bounds_variant_judgement as judgement_module
    names_of_the_driver = [function.__name__ for profile in ("r2", "r3", "r4") for function in JUDGE_FUNCTIONS[profile]]
    expect(list(judgement_module.JUDGING_ENTRIES) == names_of_the_driver,
           f"判法模块登记的判法入口 JUDGING_ENTRIES 应当与驱动的 JUDGE_FUNCTIONS 按 r2、r3、r4 的次序同名，实际模块 {list(judgement_module.JUDGING_ENTRIES)}、驱动 {names_of_the_driver}",
           "看判法模块的 JUDGING_ENTRIES 与驱动的 JUDGE_FUNCTIONS：加、删、改名一个判法函数时两边一起改（算键库自证照模块那一行逐个收闭包）")
    expect(all(written_judges[profile] == [f"judge:{judge_module}::{name}" for name in names] + [f"judge-root:{DEFAULT_ROOT}"] for profile, names in wanted_judges.items())
           and in_module == [os.path.realpath(os.path.join(DEFAULT_ROOT, judge_module))],
           f"真跑的按项判定请求里判法应当是判法模块里判一个变体的函数与写基线那一行的函数（三套各两个）、judge-root 是脚本所在的仓，不是整份驱动，"
           f"实际 {written_judges}，函数所在的文件 {in_module}",
           "看 judge_tokens 与 JUDGE_FUNCTIONS：判法写 judge:<判法模块>::<函数>，函数要在判法模块里")

    # 副本的范围取共用库在根上现算的闭包（copy_root_entries）：假仓里 crates/demo 经 path 依赖仓根下的 served/、编译期读 .claude/heavy-thread-default
    # ⇒ 两样连同共用清单拷进副本，research/ 不拷；副本里起命令的多机配置取脚本所在的仓那一份（environment_in_the_copy：假仓 git init 过，没设取它的 multi-host.env，设了的照原样）
    with tempfile.TemporaryDirectory(prefix="e165-kill-bounds-copy-") as temporary:
        source = os.path.join(temporary, "root")
        for relative, text in (("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/demo"]\n'), ("Cargo.lock", "# lock\n"),
                               ("crates/demo/Cargo.toml", '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n'
                                                          '[dependencies]\nserved = { path = "../../served" }\n'),
                               ("crates/demo/src/lib.rs", 'pub const THREADS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.claude/heavy-thread-default"));\n'),
                               ("served/Cargo.toml", '[package]\nname = "served"\nversion = "0.1.0"\nedition = "2021"\n'), ("served/src/lib.rs", "pub fn served() {}\n"),
                               (".claude/heavy-thread-default", "7\n"), ("research/results/big.out", "r\n")):
            os.makedirs(os.path.dirname(os.path.join(source, relative)), exist_ok=True)
            write(os.path.join(source, relative), text)
        copy = os.path.join(temporary, "copy")
        try:
            copy_root_entries(source, copy)
            problem = None
        except Stop as stop:
            problem = str(stop)
        copied = sorted(os.path.relpath(os.path.join(directory, name), copy) for directory, _, names in os.walk(copy) for name in names)
        wanted = [".claude/heavy-thread-default", "Cargo.lock", "Cargo.toml", "crates/demo/Cargo.toml", "crates/demo/src/lib.rs", "served/Cargo.toml", "served/src/lib.rs"]
        expect(problem is None and copied == wanted,
               f"副本应当照共用库的闭包拷：应当是 {wanted}（path 依赖的 served/、编译期读的 .claude/heavy-thread-default 都在，research/ 不在），实际 {problem or copied}",
               "看 copy_root_entries 与 scripts/hosts/lib_repository_copy.py 的 copy_closure、copy_entries")
        git_made = subprocess.run(["git", "init", "-q", source], capture_output=True, stdin=subprocess.DEVNULL).returncode == 0
        wanted_configuration = os.path.join(os.path.realpath(source), "multi-host.env")
        unset = environment_in_the_copy(source, {}).get("SINGLEFS_MULTI_HOST_CONFIG")
        given = environment_in_the_copy(source, {"SINGLEFS_MULTI_HOST_CONFIG": "/given/multi-host.env"}).get("SINGLEFS_MULTI_HOST_CONFIG")
        expect(git_made and unset == wanted_configuration and given == "/given/multi-host.env",
               f"副本里起命令的多机配置应当取脚本所在的仓那一份：没设应当是 {wanted_configuration}、设了的照原样，实际 git 建仓 {git_made}、没设得 {unset}、设了得 {given}",
               "看 environment_in_the_copy 与 scripts/hosts/lib_repository_copy.py 的 environment_for_commands_in_the_copy")

    with tempfile.TemporaryDirectory(prefix="e165-kill-bounds-selftest-") as temporary:
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
        # 自证起的那几趟按项判定的判法（judge_tokens）：判法闭包在假树上算，复用那六格的样板树是这棵树的拷贝，同样带着它
        os.makedirs(os.path.dirname(os.path.join(root, SELFTEST_JUDGE_FILE)), exist_ok=True)
        write(os.path.join(root, SELFTEST_JUDGE_FILE), SELFTEST_JUDGE_TEXT)
        subprocess.run(["git", "init", "-q", root], check=True)
        cargo_log = os.path.join(temporary, "cargo.log")
        binary_log = os.path.join(temporary, "binary.log")

        library_count = [0]

        def drive(arguments, broken="", extra=None, fake_root=None, library=None):
            # 弄坏开关：外面设了（BREAK，自证这个进程读到的）就每一趟都带上，再并上这一趟自己点名的（对照那几例）；外面没设只带这一趟自己的。
            # r2 那四项（R2_BREAK_MODES）与 r3 那四项（R3_BREAK_MODES）不在这里带：r2、r3 那几趟自己把外面给的原样传进去（r2_broken、r3_broken），别的几趟照常跑
            # 判定库：临时 prune-server 里每一趟一份新的命名空间（LONG_RUN_ITEMS_LIBRARY_DIRECTORY），续跑那几格给 library 让几趟共用一份；
            # 复用哨兵全当哨兵、不抽强制现判（编几次是定数）；假仓里没有真装置的源码，不给 LONG_RUN_ITEMS_SELFTEST_ENTRIES 的那几趟每一项都拒算、照跑
            library_count[0] += 1
            environment = dict(os.environ)
            environment["E165_KILL_BOUNDS_SELFTEST_ROOT"] = fake_root or root
            environment["FAKE_CARGO_LOG"] = cargo_log
            environment["FAKE_BIN_LOG"] = binary_log
            environment["E165_KILL_BOUNDS_BREAK"] = ",".join(sorted((BREAK - set(R2_BREAK_MODES) - set(R3_BREAK_MODES) - set(R4_BREAK_MODES)) | set(filter(None, broken.split(",")))))
            environment["LONG_RUN_ITEMS_LIBRARY_DIRECTORY"] = library or os.path.join(temporary, f"library-{library_count[0]}")
            environment["SINGLEFS_REUSE_SENTINEL_COUNT"] = "1000"
            for name in ("SINGLEFS_REUSE_SENTINEL_SEED", "LONG_RUN_ITEMS_SELFTEST_ENTRIES", "LONG_RUN_ITEMS_SELFTEST_FAULT", "LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE", KILL_AFTER_VARIABLE):
                environment.pop(name, None)
            environment.pop("SINGLEFS_THREAD_CAP", None)
            environment.pop("E165_KILL_BOUNDS_PROGRESS_INTERVAL", None)
            environment.pop("FAKE_BIN_SLEEP", None)
            environment.pop("FAKE_BIN_NO_R2_ROWS", None)
            environment.pop("FAKE_BIN_NO_R3_ROWS", None)
            environment.pop("FAKE_BIN_NO_R4_ROWS", None)
            environment.update(extra or {})
            # 几趟共用一份库的那几格（续跑、钉红例、改判法）重交同一条命令：已经存在的 --output 不收（文件头「用法」），
            # 驱动先把上一趟的产物与账挪成 .previous（只供自证的 SET_OUTPUT_ASIDE_VARIABLE）；别的格不设，--output、--ledger 已经存在的照拒
            environment.pop(SET_OUTPUT_ASIDE_VARIABLE, None)
            if library is not None:
                environment[SET_OUTPUT_ASIDE_VARIABLE] = "1"
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
        expect(re.search(r"item=tsv-3 .* outcome=not_red_within_bounds .* not_red_domains=L\(≤3\) ", output) is not None, "不红的那一条应当记 not_red_within_bounds、上沿 3", "看判法模块 judge_variant 的结局分类")
        expect(re.search(r"item=tsv-4 .* outcome=invalid ", output) is not None, "编不过的那一条应当记 invalid", "看 prepare_and_build 的返回")
        expect(re.search(r"item=tsv-5 .* outcome=red first_clause_red=L:2:J4 ", output) is not None, "文件列是 lib.rs 的那一条应当改 lib.rs、记 outcome=red", "看 prepare_and_build 的 target_of")
        expect(re.search(r"item=Z1 .* outcome=red ", output) is not None, "自造的那一条（文件列 lib.rs）应当记 red", "看 load_items 读自造表")
        expect("已还原，基线与开头逐字相同" in completed.stdout and " name=restored " in output, "收尾要打「已还原，基线与开头逐字相同」并写 name=restored 行", "看 main_run 的收尾")
        expect(last_line == "E7RESULT name=done emitted=10", f"收尾行应当是 name=done emitted=10，实际 {last_line!r}", "emitted 数的是 E7RESULT 行，含 config 与 done")
        item_rows = [line for line in completed.stdout.split("\n") if line.startswith("E165_KILL_BOUNDS_ITEM ")]
        item_reds = {found.group(1): int(found.group(2)) for found in (re.match(r"E165_KILL_BOUNDS_ITEM item=(\S+) exit=0 judged=1 red=(\d) reused=0 seconds=\d+\.\d$", line) for line in item_rows) if found}
        stdout_last = completed.stdout.rstrip("\n").split("\n")[-1]
        expect(len(item_rows) == 6 and item_reds == {"baseline": 0, "tsv-2": 1, "tsv-3": 0, "tsv-4": 0, "tsv-5": 1, "Z1": 1}
               and re.match(r"E165_KILL_BOUNDS_TOTAL dispatched=6 collected=6 items=6 judged=6 red=3 reused=0 failed=0 seconds=\d+\.\d$", stdout_last) is not None,
               f"整轮每项一行应当带判了、红了、复用了多少与秒，末行总计派出 6 收回 6、红 3，实际 {item_rows}；末行 {stdout_last!r}",
               "看 ItemLines 与三套 main_run 里 work() 的 judged_item")
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
        # 一之二、--output 指到已经存在的文件（上面整轮写的那一份），--ledger 是新的：退 2、出路写全、一次不编，那一份一个字不动、账不写（开关 outputoverwritten 下照收、整份换掉）。
        # 一之三、--ledger 指到已经存在的文件（整轮写的那一份），--output 是新的：同样退 2、同一句出路、一次不编，账一个字不动、产物不写（开关 ledgeroverwritten 下照收）。
        # 开关开着时那一趟照新的一对钉、把整轮那一份换掉：两格判完把产物、账与钉表放回整轮写完时的样子，后面几格照整轮那一份比
        pinned_text = read(pinned_table)
        output_taken = paths("full")
        output_taken[output_taken.index("--ledger") + 1] = os.path.join(temporary, "output-taken.ledger")
        completed, builds = drive(output_taken)
        output_untouched = read(os.path.join(temporary, "full.out")) == output
        expect(completed.returncode == EXIT_USAGE and "--output full.out 已经存在" in completed.stdout and "换一个路径，或先挪走那一份" in completed.stdout
               and len(builds) == 0 and output_untouched and not os.path.exists(os.path.join(temporary, "output-taken.ledger")),
               f"--output 指到已经存在的文件应当退 2、出路写换一个路径或先挪走那一份、一次不编、那一份一个字不动，实际退 {completed.returncode}、编 {len(builds)} 次、"
               f"那一份{'没动' if output_untouched else '变了'}：{completed.stdout[-300:]}",
               "看 main 里三套整轮调 refuse_existing_output 的那几处")
        ledger_taken = paths("ledger-taken")
        ledger_taken[ledger_taken.index("--ledger") + 1] = os.path.join(temporary, "full.ledger")
        completed, builds = drive(ledger_taken)
        ledger_untouched = read(os.path.join(temporary, "full.ledger")) == ledger
        expect(completed.returncode == EXIT_USAGE and "--ledger full.ledger 已经存在" in completed.stdout and "换一个路径，或先挪走那一份" in completed.stdout
               and len(builds) == 0 and ledger_untouched and not os.path.exists(os.path.join(temporary, "ledger-taken.out")),
               f"--ledger 指到已经存在的文件应当退 2、出路写换一个路径或先挪走那一份、一次不编、那一份一个字不动，实际退 {completed.returncode}、编 {len(builds)} 次、"
               f"账{'没动' if ledger_untouched else '变了'}：{completed.stdout[-300:]}",
               "看 main 里不给 --profile 那一套调 refuse_existing_output 时交进去的 --ledger")
        write(os.path.join(temporary, "full.out"), output)
        write(os.path.join(temporary, "full.ledger"), ledger)
        write(pinned_table, pinned_text)

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

        # 三、续跑（文件头「按项判定」）：判完 tsv-3 就被停下的一趟（只供自证的 E165_KILL_BOUNDS_SELFTEST_KILL_AFTER：record 之后整个进程被 SIGKILL），
        # 重交同一条命令只补没判的三项，库里复用的三项不编、各打一行 reused=1，产物行与整轮的逐字相同；再交一趟全部复用、一次不编、产物逐字相同。
        # 三趟共用一份判定库命名空间；假仓里的装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假树里的 Python 函数（不换就全部拒算、一项都不复用）。
        planted_code, planted = item_verdicts_command(["plant-fake-entries", root])
        resume_entries = os.path.join(temporary, "resume-entries.tsv")
        write(resume_entries, "".join(f"{name}\tscripts/experiment-apparatus/long_run_reuse_fake_entries.py::{'measure_beta' if name == 'baseline' else 'measure_alpha'}\n"
                                      for name in ("baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "Z1")))
        resume_extra = {"LONG_RUN_ITEMS_SELFTEST_ENTRIES": resume_entries}
        resume_library = os.path.join(temporary, "resume-library")
        resume_arguments = paths("resume")[:-4] + ["--jobs", "1", "--threads", "1"]
        resumed_path = os.path.join(temporary, "resume.out")
        completed, builds = drive(resume_arguments, library=resume_library, extra=dict(resume_extra, **{KILL_AFTER_VARIABLE: "tsv-3"}))
        expect(planted_code == 0 and completed.returncode == -signal.SIGKILL and len(builds) == 3,
               f"续跑：判完 tsv-3 就被停下的那一趟应当被 SIGKILL、只编了 3 次（开头基线、tsv-2、tsv-3），实际种假入口退 {planted_code}（{planted.strip()[-200:]}）、"
               f"退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-300:]}{completed.stderr[-300:]}",
               "看 ItemVerdicts.record 与只供自证的 E165_KILL_BOUNDS_SELFTEST_KILL_AFTER")
        completed, builds = drive(resume_arguments, library=resume_library, extra=resume_extra)
        resumed = read(resumed_path)
        expect(completed.returncode == 0, f"续跑应当退 0，实际 {completed.returncode}：{completed.stdout[-300:]}{completed.stderr[-300:]}", "看 ItemVerdicts 与 main_run")
        expect(len(builds) == 5, f"续跑应当只编 5 次（基线 2、没判的变体 3），实际 {len(builds)}", "判完的变体当场入库（ItemVerdicts.record），重交时 plan 答 reuse 的不编不跑")
        counts = {name: len(re.findall(rf"name=kill_bound item={name} .* complete=yes", resumed)) for name in ("baseline", "tsv-2", "tsv-3", "tsv-4", "tsv-5", "Z1")}
        expect(all(count == 1 for count in counts.values()), f"续跑之后每个变体恰好一行完整的，实际 {counts}", "看 judge_items_in_parallel 的 flush_in_order")
        expect(resumed.rstrip("\n").split("\n")[-1].startswith("E7RESULT name=done "), "续跑补齐之后要写收尾行", "看 main_run 的收尾")
        complete_lines = lambda text: sorted(line for line in text.replace("resume.", "full.").split("\n") if line.endswith("complete=yes"))
        expect(complete_lines(resumed) == complete_lines(output), "续跑出来的变体行（库里取回的与现判的）应当与整轮的逐字相同",
               "看 ItemVerdicts 的 value_text 与 reused_record：复跑命令那一段存成占位、取回时照这一趟填回；变体行里不许有随跑法变的量")
        resumed_items = [line for line in completed.stdout.split("\n") if line.startswith("E165_KILL_BOUNDS_ITEM ")]
        resumed_last = completed.stdout.rstrip("\n").split("\n")[-1]
        expect(any(line.startswith("E165_KILL_BOUNDS_ITEM item=baseline exit=0 judged=0 red=0 reused=1 ") for line in resumed_items)
               and any(line.startswith("E165_KILL_BOUNDS_ITEM item=tsv-2 exit=0 judged=0 red=1 reused=1 ") for line in resumed_items)
               and any(line.startswith("E165_KILL_BOUNDS_ITEM item=tsv-3 exit=0 judged=0 red=0 reused=1 ") for line in resumed_items) and len(resumed_items) == 6
               and re.match(r"E165_KILL_BOUNDS_TOTAL dispatched=3 collected=3 items=6 judged=3 red=3 reused=3 failed=0 ", resumed_last) is not None
               and "按项判定 现判 3、复用 3、拒算 0" in completed.stdout,
               f"续跑：库里复用的三项应当各打一行 reused=1（tsv-2 照库里那一行记 red=1），末行总计派出 3 收回 3、复用 3，成功句报现判 3、复用 3、拒算 0，"
               f"实际 {resumed_items}；末行 {resumed_last!r}", "看 ItemVerdicts.report_reused 与 finish 的 summary")
        pinned_after_resume = [line for line in read(pinned_table).split("\n") if line and not line.startswith("#")]
        expect(pinned_after_resume == pinned_rows and f"钉红例：钉进 0 行、核了 {len(ledger_examples)} 行" in completed.stdout,
               f"续跑补齐之后照样当场钉、当场核：同一批红例已钉，钉表应当不变、报钉进 0 行，实际 {len(pinned_after_resume)} 行：{completed.stdout[-300:]}",
               "看 main_run 收尾的 pin_red_examples：追加已钉的不重复")
        completed, builds = drive(resume_arguments, library=resume_library, extra=resume_extra)
        expect(completed.returncode == 0 and len(builds) == 0 and read(resumed_path) == resumed and "按项判定 现判 0、复用 6、拒算 0" in completed.stdout
               and "开头与收尾的基线都没编没跑" in completed.stdout,
               f"全部复用的一趟应当一次不编、产物与上一趟逐字相同、报现判 0、复用 6，实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-400:]}",
               "看 opening_baseline 一项都不跑的那一支与收尾行的基线行数")

        # 三之二、钉红例（文件头「钉红例」）：全部从库里复用的一趟（不编不跑变体）照样从头写产物与账、照钉照核。
        # 钉表清空，再造「账写完了、钉表没钉」（只供自证的 E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND：不追加）：当场 --check 判红、整趟退 8（开关 pincheckskipped 下退 0）
        write(pinned_table, "# 自检用的钉住的表\n")
        completed, builds = drive(resume_arguments, library=resume_library, extra=dict(resume_extra, E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND="1"))
        expect(completed.returncode == EXIT_PIN_INCOMPLETE and "钉红例：核钉全（--check）没过" in completed.stdout and "行没钉进" in completed.stdout and len(builds) == 0,
               f"账写完而钉表没钉时驱动当场核钉全应当判红、退 {EXIT_PIN_INCOMPLETE}、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-300:]}",
               "看 pin_red_examples 的 --check 那一步与它在 main_run 收尾的位置")
        completed, builds = drive(resume_arguments, library=resume_library, broken="pincheckskipped", extra=dict(resume_extra, E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND="1"))
        expect(completed.returncode == 0 and "核了 没核" in completed.stdout,
               f"弄坏开关 pincheckskipped 开着时应当照样退 0（证明上一例的红来自当场核钉全），实际退 {completed.returncode}：{completed.stdout[-300:]}", "看 BREAK 的 pincheckskipped")
        completed, builds = drive(resume_arguments, library=resume_library, extra=resume_extra)
        repinned = [line for line in read(pinned_table).split("\n") if line and not line.startswith("#")]
        # 钉表的出处 ⑤ 是「<红例账文件名>:<行号>」（e165-pin-red-examples.py 的 pinned_fields）：整轮钉的写 full.ledger，这一趟钉的写 resume.ledger；
        # 别的 21 段要与整轮钉的逐字相同，⑤ 要指到这一趟的账
        without_source = lambda rows: [row.split("\t")[:4] + row.split("\t")[5:] for row in rows]
        expect(completed.returncode == 0 and without_source(repinned) == without_source(pinned_rows) and len(builds) == 0
               and all(row.split("\t")[4].startswith("resume.ledger:") for row in repinned) and len(repinned) == len(pinned_rows)
               and f"钉进 {len(ledger_examples)} 行、核了 {len(ledger_examples)} 行" in completed.stdout,
               f"判过的一对重交同一条命令应当全部复用、照钉照核：钉回同样的 {len(pinned_rows)} 行（出处 ⑤ 换成这一趟的 resume.ledger、别的段与整轮钉的逐字相同）、一次不编，"
               f"实际退 {completed.returncode}、{len(repinned)} 行、编 {len(builds)} 次、出处 {[row.split(chr(9))[4] for row in repinned]}、"
               f"除出处外与整轮钉的{'相同' if without_source(repinned) == without_source(pinned_rows) else '不同'}：{completed.stdout[-300:]}",
               "看 main_run 收尾的 pin_red_examples 与 opening_baseline 一项都不跑的那一支")
        # 对照：弄坏开关 reuseignored（plan 答 reuse 的也照现判）下同一条命令每个变体都重编、复用哨兵判红；
        # recorddeferred（判完不当场入库）下被停下的那一趟一项都没入库，重交全部重编
        completed, builds = drive(resume_arguments, library=resume_library, broken="reuseignored", extra=resume_extra)
        expect(len(builds) == 7 and completed.returncode == EXIT_INCOMPLETE and "复用失灵" in completed.stdout,
               f"弄坏开关 reuseignored 开着时应当每个变体都重编、复用哨兵判红退 1（证明「续跑应当只编 5 次」「全部复用的一趟应当一次不编」来自库里复用），"
               f"实际退 {completed.returncode}、编 {len(builds)} 次", "看 BREAK 的 reuseignored")
        deferred_library = os.path.join(temporary, "deferred-library")
        drive(resume_arguments, library=deferred_library, broken="recorddeferred", extra=dict(resume_extra, **{KILL_AFTER_VARIABLE: "tsv-3"}))
        completed, builds = drive(resume_arguments, library=deferred_library, extra=resume_extra)
        expect(len(builds) == 7, f"弄坏开关 recorddeferred 开着时被停下的那一趟一项都没入库，重交应当全部重编（证明「续跑应当只编 5 次」来自判完当场入库），实际编 {len(builds)} 次",
               "看 BREAK 的 recorddeferred")
        # 改判法：假树里的判法（SELFTEST_JUDGE_FILE）改掉返回的那一行，库里六项都判过，同一条命令应当每一项都现判、一项不拒算；改回去，后面几格照旧。
        # 证明自证的键里判法的闭包在假树上算（不在脚本所在的仓上收整份驱动），判法也进键（开关 judgeoffthetree 下闭包换到假树的上一层，每一项拒算）
        judge_path = os.path.join(root, SELFTEST_JUDGE_FILE)
        write(judge_path, SELFTEST_JUDGE_TEXT.replace(SELFTEST_JUDGE_RESULT, '    return "red when any criterion row differs"\n'))
        completed, builds = drive(resume_arguments, library=resume_library, extra=resume_extra)
        write(judge_path, SELFTEST_JUDGE_TEXT)
        expect(completed.returncode == 0 and len(builds) == 7 and "按项判定 现判 6、复用 0、拒算 0" in completed.stdout,
               f"改了假树里的判法应当每一项都现判、一项不拒算（报现判 6、复用 0、拒算 0，编 7 次），实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-400:]}",
               "看 judge_tokens：自证起的那几趟 judge: 要指假树里的 SELFTEST_JUDGE_ENTRY、不带 judge-root:")

        # 三之三、留存产物不在（research/ 可以整个删掉）：V7 的比对本次未跑——逐项点名、不当绿也不判红，基线那一行写 not_run（开关 retainedmissingred 下读不到、退非 0）
        os.rename(os.path.join(root, "seg1.out"), os.path.join(temporary, "seg1.out.aside"))
        completed, builds = drive(paths("no-seg1"))
        os.rename(os.path.join(temporary, "seg1.out.aside"), os.path.join(root, "seg1.out"))
        no_seg1 = read(os.path.join(temporary, "no-seg1.out"))
        expect(completed.returncode == 0 and "本次未跑：作废条款 V7 的比对：留存产物" in completed.stdout and "seg1.out 不在" in completed.stdout
               and "本次未跑 1 项" in completed.stdout and " rows_found_verbatim_in_segment1_products=not_run " in no_seg1,
               f"r2：第一段产物不在时 V7 的比对应当逐项报本次未跑、照跑完退 0、基线那一行写 not_run，实际退 {completed.returncode}：{completed.stdout[-400:]}{completed.stderr[-200:]}",
               "看 retained_product_not_run 与 check_against_segment_one")

        # 三之四、复用那六格（long_run_item_verdicts.py 文件头「复用格」）：样板树是这时的假仓（表、自造表、登记值同整轮），项甲 tsv-2、项乙 基线，
        # 只点名 tsv-2（基线照 @after 进计划）；两项的装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假树里的两个 Python 函数。
        # 外面开着的按项判定那几项弄坏开关（ITEM_VERDICT_BREAK_MODES）原样带进去，别的不带；LONG_RUN_ITEMS_BREAK 随环境带进去。
        reuse_template = os.path.join(temporary, "reuse-template")
        shutil.copytree(root, reuse_template, symlinks=True)
        # 复用三问一格里同一条命令起四趟：已经存在的 --output 不收（文件头「用法」），驱动先把上一趟的产物与账挪走（只供自证的 SET_OUTPUT_ASIDE_VARIABLE）
        reuse_command = ["env", "E165_KILL_BOUNDS_SELFTEST_ROOT={tree}", "FAKE_CARGO_LOG=/dev/null", "FAKE_BIN_LOG=/dev/null", SET_OUTPUT_ASIDE_VARIABLE + "=1",
                         "E165_KILL_BOUNDS_BREAK=" + ",".join(sorted(BREAK & set(ITEM_VERDICT_BREAK_MODES))), "bash", SELF_PATH,
                         "--scratch", "{tree}/../scratch", "--output", "{tree}/../reuse-cells-kill-bounds.out", "--ledger", "{tree}/../reuse-cells-red-examples.tsv",
                         "--jobs", "1", "--threads", "1", "--item", "tsv-2"]
        say_progress("开跑复用那六格（long_run_item_verdicts.py reuse-cells，十一趟，跑完一并打出）")
        reuse_code, reuse_output = item_verdicts_command(["reuse-cells", "--script", SCRIPT_KEY, "--template", reuse_template, "--items", "tsv-2,baseline",
                                                          "--work", os.path.join(temporary, "reuse-cells"), "--green-exit", "0", "--", *reuse_command])
        print(reuse_output.rstrip())
        expect(reuse_code == 0 and "：6 格，红 0 格" in reuse_output,
               f"复用那六格（long_run_item_verdicts.py reuse-cells）应当全过，实际退 {reuse_code}：{reuse_output.strip()[-600:]}",
               "看上面 ✗ 那一格点名的条件、ItemVerdicts 的 plan / record / finish 与 item_requests_text 进键的那几样")

        # 四、--recheck 单条复跑与产物里的行相同；--list-items 列全、不编。
        completed, builds = drive(["--scratch", os.path.join(temporary, "scratch-recheck"), "--output", os.path.join(temporary, "full.out"),
                                   "--ledger", os.path.join(temporary, "full.ledger"), "--jobs", "1", "--threads", "1", "--recheck", "--item", "tsv-2"])
        expect(completed.returncode == 0 and "✓ tsv-2" in completed.stdout, f"--recheck 同一条应当逐字相同、退 0，实际退 {completed.returncode}：{completed.stdout[-300:]}", "看 main_run 的 recheck 分支")
        completed, builds = drive(["--list-items"])
        listed = [line for line in completed.stdout.split("\n") if line.strip()]
        expect(completed.returncode == 0 and len(listed) == 6 and len(builds) == 0, f"--list-items 应当列 6 个变体、不编，实际：{completed.stdout!r}", "看 main 的 list 分支")
        completed, builds = drive(paths("unknown-item") + ["--item", "tsv-99"])
        expect(completed.returncode == 2 and "不认识的变体：tsv-99；可点名的 6 项：baseline tsv-2 tsv-3 tsv-4 tsv-5 Z1" in completed.stdout and len(builds) == 0,
               f"--item 写了不认识的变体名应当退 2、列出可点名的 6 项、一次不编，实际退 {completed.returncode}、编 {len(builds)} 次：{completed.stdout[-300:]}", "看 unknown_item")

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
               "看 main_run 里 Progress 的 start、judge_built_variant 与判法模块 judge_variant 里的 mark")


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
               "r3：K 族不红而 KA 读数等于钉值的变体记 not_red_within_bounds、ka_pin=match；推的两个（1822、1827）不钉，只报 ka_inferred", "看 r3_ka_pin 与 judge_variant_r3")
        expect(re.search(r"item=tsv-6 .* outcome=red_only_by_kj0_or_kj3 red_legal_domains=none first_red_by_criterion=KJ1:K=none,K9=none,KW=none;KJ2:K=none,K9=none,KW=none;KJ0:K=M1U1,K9=none,KW=none;KJ3:.* "
                         r"states_row_differences=K=M1U1:choose_root_returned_none;K9=M1U0:above_chosen_root_over_limit .* above_chosen_root_over_limit=K:0,K9:4,KW:0,KA:0 "
                         r"choose_root_returned_none=K:1,K9:0,KW:0,KA:1 .* halt_conditions=1 .* run_exit=1 run_finished=true ", output) is not None,
               f"r3：变体跑到装置自己退 1（halt 行）而 K 域 KJ0 变红，应当照常落行、记 run_exit=1、run_finished=true、outcome=red_only_by_kj0_or_kj3，实际 {[line[:700] for line in output.split(chr(10)) if 'item=tsv-6 ' in line]}",
               "看 judge_variant_r3：装置退 1 但最后一行是 name=done 就是跑完了")
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
        # ⑤之二 第二段产物不在（留存产物可以整个删掉）⇒ TV7 的比对本次未跑、逐项点名，照跑完；收尾判决行写 baseline_equals_segment2_product=not_run（开关 retainedmissingred 下报 TV8 退 6）
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
               f"r3：变异名选不到的那一行应当记 not_compared、别的五行照做（编 7 次：基线 2、变体 5）、整轮退 1，实际退 {completed.returncode}、编 {len(builds)} 次", "看 load_items_r3 与 judge_variant_r3 的 TH5 一支")
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
               "看 r4_differs_on 与 judge_variant_r4 的结局分类")
        expect(re.search(r"item=tsv-4 .* outcome=not_red_within_bounds variant=v1824 .* differs_on=KG\+KA .* ka_kj2=52 ka_expected=52 ka_pin=match ka_inferred=none kg_pin=match ", output) is not None
               and re.search(r"item=tsv-5 .* outcome=not_red_within_bounds variant=v1825 .* differs_on=KA .* ka_kj2=36 ka_expected=36 ka_pin=match ka_inferred=none kg_pin=match ", output) is not None
               and re.search(r"item=tsv-7 .* ka_kj2=18 ka_expected=18 ka_pin=not_pinned ka_inferred=match kg_pin=match ", output) is not None,
               "r4：KA 读数等于钉值、KG 读数等于那一列的变体记 ka_pin=match、kg_pin=match；推的 v1822、v1827 不钉，只报 ka_inferred（v1827 的 KG 取 v1823 那一列）", "看 r4_ka_pin、r4_kg_pin 与 judge_variant_r4")
        expect(re.search(r"item=tsv-6 .* outcome=red_only_by_kj0_or_kj3 variant=v1826 red_legal_domains=none .* first_red_by_criterion=KJ1:KF=none,KF3=none,KF9=none,KFW=none;"
                         r"KJ1T:KF=none,KF3=none,KF9=none,KFW=none;KJ2:KF=none,KF3=none,KF9=none,KFW=none;KJ0:KF=M1U1/none/inside,.* "
                         r"choose_root_returned_none_with_a_readable_root=KF:1,.* halt_conditions=1 .* run_exit=1 run_finished=true ", output) is not None,
               f"r4：变体跑到装置自己退 1（halt 行）而 KF 域 KJ0 变红，应当照常落行、记 run_exit=1、run_finished=true、outcome=red_only_by_kj0_or_kj3，实际 {[line[:700] for line in output.split(chr(10)) if 'item=tsv-6 ' in line]}",
               "看 judge_variant_r4：装置退 1 但最后一行是 name=done 就是跑完了")
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
        # ⑤之二 第一段产物不在（留存产物可以整个删掉）⇒ TV7 的比对本次未跑、逐项点名，照跑完；收尾判决行写 baseline_equals_segment1_product=not_run（开关 retainedmissingred 下报 TV8 退 6）
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
               f"r4：变异名选不到的那一行应当记 not_compared、别的五行照做（编 7 次：基线 2、变体 5）、整轮退 1，实际退 {completed.returncode}、编 {len(builds)} 次", "看 load_items_r4 与 judge_variant_r4 的 TH5 一支")
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
               "看 device_bounds_arguments、check_the_gate_rows_r4、pins_config_text 与 judge_variant_r4 的 head")
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
                               f"  → 怎么办：{profile} 模式只有 --list-items、--clean 与整轮（--scratch、--output、--item、--jobs；每个变体的线程数不写 --threads，取每台默认重活线程数 ÷ --jobs，经 task-server/task submit 交时 --threads 当场被拒）")
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
        raise Stop(EXIT_USAGE, f"  ✗ jobs × threads = {options['jobs'] * options['threads']} 超过线程上限 {cap}\n  → 怎么办：压小 --jobs、去掉 --threads（不写时取 {cap} ÷ --jobs；经 task-server/task submit 交时 --threads 当场被拒）")
    os.makedirs(options["work"], exist_ok=True)
    if action == "measure":
        return measure(setup, options, measure_name)
    if profile == "r3":
        if output is None:
            raise Stop(EXIT_USAGE, "  ✗ 缺 --output\n  → 怎么办：产物进 research/results/，名字照登记 r3 5.8（前缀 e165-replay-bounded-enumeration-<日期>-r3-segment3-）")
        options["output"] = os.path.abspath(output)
        options["output_shown"] = os.path.relpath(options["output"], setup.root) if os.path.commonpath([options["output"], setup.root]) == setup.root else os.path.basename(options["output"])
        refuse_existing_output(setup, options["output"], options["output_shown"])
        return main_run_r3(setup, options, item_names)
    if profile == "r4":
        if output is None:
            raise Stop(EXIT_USAGE, "  ✗ 缺 --output\n  → 怎么办：产物进 research/results/，名字照登记 r4 5.7（前缀 e165-replay-bounded-enumeration-<日期>-r4-segment2-）")
        options["output"] = os.path.abspath(output)
        options["output_shown"] = os.path.relpath(options["output"], setup.root) if os.path.commonpath([options["output"], setup.root]) == setup.root else os.path.basename(options["output"])
        refuse_existing_output(setup, options["output"], options["output_shown"])
        return main_run_r4(setup, options, item_names)
    if output is None or ledger is None:
        raise Stop(EXIT_USAGE, "  ✗ 缺 --output 或 --ledger\n  → 怎么办：两个都给；产物进 research/results/，名字照登记 5.5")
    if recheck and not item_names:
        raise Stop(EXIT_USAGE, "  ✗ --recheck 要用 --item 点名\n  → 怎么办：--recheck --item <变体>")
    options["output"], options["ledger"] = os.path.abspath(output), os.path.abspath(ledger)

    def shown(path):
        return os.path.relpath(path, setup.root) if os.path.commonpath([path, setup.root]) == setup.root else os.path.basename(path)

    options["output_shown"], options["ledger_shown"] = shown(options["output"]), shown(options["ledger"])
    if not recheck:   # --recheck 拿新跑的行与已有的产物比，产物与账本来就该在
        refuse_existing_output(setup, options["output"], options["output_shown"], options["ledger"], options["ledger_shown"])
    return main_run(setup, options, item_names, recheck)


try:
    exit_code = main(sys.argv[1:])
except Stop as stop:
    print(str(stop))
    exit_code = stop.code
if ITEM_LINES["report"] is not None:
    exit_code = ITEM_LINES["report"].close(exit_code)
sys.exit(exit_code)
PYTHON
