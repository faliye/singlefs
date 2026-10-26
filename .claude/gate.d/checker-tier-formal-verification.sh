#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓、此刻的 Verus 安装与 cargo 缓存，上一次的结论不替这一次作保
# run-condition: none 要的 Verus、cargo 与装置由各格的判法自己判，缺了判红；核还没进仓时 Verus 那四格报本次未跑
# gate-stage: 形式化验证（Verus 证明恢复重放取前缀的核，版本钉住、空真与必红世界按落点判；E165 全量的红例钉进快档；两条本该独立的腿的同源性计数器）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: checker-tier-lkmm.sh 同是起外部验证器的一道，接法相同（判法放脚本、每条判定按单元键进判定库 scripts/crash/lib_verdict_library.py）；它起 herd7 判 litmus，这里起 Verus 判 crates/ 的证明，并进去会让两件不相干的事共用一份判定库的种类与出路
# gate-similar: checker-tier-crates-mutation-replay.sh 它逐条改坏 crates/ 要求点名的测试红；这里的必红世界要求 Verus 报证明错，写不进它的 crates/mutations.tsv
# gate-similar: code-source-discipline.sh 它的 mutation-tables 格静态判实验二进制的变异表成形、锚点唯一；这里判 E165 钉住红例的表本身、并逐行在今天的实现上重放
# gate-similar: checker-tier-research-build-and-replay.sh 它的 experiment-replay 格整份复跑入库实验比产物；这里只逐行重放钉住的红例（快档）
# gate-similar: checker-tier-crash-point-replay.sh 它判崩溃点重放与登记的崩溃枚举用例；这里判证明与有界穷举的红例，不起崩溃枚举
# gate-similar: checker-independence-and-sync.sh 它判 checker 与实现、RefFS 与实现的依赖闭包不相交（结构上的独立）；same-origin-counter 格数的是两条腿运行时的不一致事件（统计上的独立），读 divergence-events.tsv 一张表、不看依赖，并进它会让一道结构门禁去读事件账
#
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就七格按下面的次序全跑，格名写错退 2）：
# gate-cell: verus-toolchain-pinned Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹、Verus 与产品的 rustc 版本都与登记相同
# gate-cell: verus-proof Verus 逐项验登记的各 crate 被验区域里有证明义务的单元（按全路径：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone，impl 与 trait 里的方法也算），每项限时内出结局、errors 0、verified 至少 1，各 crate 各项 verified 之和等于它登记的数，整格再完整验证 crate、产品构建与 verus --compile 构建的 MIR 逐条目相同；起 Verus 之前判被验区域：可信声明与 cfg 原子照信任清单逐条登记，axiom、作弊标记与 verus! 外面被验类型的 trait impl 判红
# gate-cell: verus-vacuity 核里每一处 requires 的函数体头插 assert(false) 都验不过（未检出入口空真；可满足性另由构造见证证明）
# gate-cell: verus-must-red-worlds 先查函数与不变式负例覆盖；必红世界表每行改坏核之后 Verus 都报错、证明错落在第六段 landing= 登记的函数里（条数只报不判），原文恰好命中一次
# gate-cell: pinned-red-examples-table 钉表 e165-pinned-red-examples.tsv 自己：每行 22 段整行抄带档、第 1 段与第 2、3、7 段重算的指纹相同、抄来的账行与自己的段对得上、第 22 段是规范写法的档，点名的变异还在它的表里；不读红例账、kill-bounds 产物与 research/ 下任何文件
# gate-cell: pinned-red-examples-replay 钉住的每一行是 22 段整行抄带档、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置按档在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同
# gate-cell: same-origin-counter 同源性计数器：不一致事件表 divergence-events.tsv 里每个已立的对累计 ≥ 5 次不一致而判「spec 错」0 次即判红；未立的对报未立、不计入，条款欠一句不计入累计
# 参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh。
#
# 判法都在两份脚本里（判据、默认值、弄坏开关写在各自文件头），这一道只接进门禁、给样本开关：
#   Verus 四格：scripts/gate/verus-proof-check.py <toolchain|proof|vacuity|must-red-worlds> <根>，登记表 .claude/gate.d/verus-pins.tsv（crate 行可以多条：
#     replay-prefix 直接起 verus，singlefs-core 这一种第四段写 cargo、经 cargo verus focus 验），信任清单 .claude/gate.d/verus-trust-list.tsv（verus-proof 格判）。
#     核没进仓（登记的核不在、crate 不依赖 vstd、src 里没有 verus!）时四格都退 77；核不在而 crate 已经接了 Verus 判红。
#     读的环境变量：GATE_VERUS_TIMEOUT_SECONDS（限时，默认 600 秒，超时判红：C5（Z3 超时当通过））、GATE_VERUS_MEMORY_MAX（默认 4G）、
#     GATE_VERUS_JOBS（几处同时验）、SINGLEFS_THREAD_CAP（线程，没设取登记的重活默认并行度 .claude/heavy-thread-default）、SINGLEFS_VERUS_ROOT（Verus 安装目录，盖过登记表）。
#   排不上只等一次：这一道给各格一份空文件（VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE）。Verus 那几格里任何一处第一次拿到内存包装的 252（排不上），
#     判法往文件里写一行「哪一处<TAB>出路」，这一格与后面几格还没起的 Verus 都不再排队（已经起了的照常收），pinned-red-examples-replay 格读到它就不编装置、不起重放，
#     各报「没跑成：内存包装排不上，没再排队」，各格的汇总之后另打一行「这一道没跑成：内存包装排不上」；出路照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段。
#     所以整道最坏只等一次等待上限（RUN_WITH_MEMORY_CAP_WAIT_SECONDS，带 SINGLEFS_HEAVY_TESTS 时默认 3600 秒），不是每处、每格各等一次。
#   红例两格：scripts/experiment-apparatus/e165-pin-red-examples.py --check-table / --replay，表的格式写在 .claude/gate.d/e165-pinned-red-examples.tsv 表头；
#     table 格只读钉表与变异表，不读红例账、kill-bounds 产物与 research/ 下任何文件（钉全由 E165 第二段驱动 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh
#     写完一对账与产物时当场判，写法在它文件头「钉红例」一段）；每行的 ㉒ 是出那份账的那一趟的档，replay 按档分组起装置、每组带 ㉒ 的参数（例 --profile r2）。
#     --replay 编装置的内存上限读 GATE_E165_PINNED_MEMORY_MAX（默认 2G，由那份脚本读）。
#     装置 e165_replay_bounded_enumeration 还不认 --pinned-cases 时，replay 格判完「变异还在」那一半就退 77。
#
# 复用：每条（证明按函数、空真每处 requires、必红世界每行、钉表每行）一条判定，键是这一条涉及的 Verus 函数单元（被验 fn、它 requires / ensures 用到的
#   spec fn 与 proof fn）去注释后的指纹、Verus 与 vstd 版本；钉表行的键是那一行档与参数重放真走到的装置函数单元闭包、状态输入、期望逐态行、变异原文与替换文；
#   限时不进键；空真项身份是所属函数单元指纹加函数内第几处，不带路径与行号。格不复用，不问 stage-must-run.sh，不写全绿标记：一格判绿就是它的每一条判定都判绿
#   （现判的或复用的），verus-toolchain-pinned、pinned-red-examples-table 与 same-origin-counter 每趟现判（只读几份文件，几秒）。必红世界每行的键里是改坏的原文、替换文与第六段落点（landing=），第五段条数只报、不进键。
#   每条判定的键、判法单元指纹（判结局的那几个函数的单元闭包键，判定库与 lib_function_units.py 不进）写在两份判法脚本文件头「逐项点名与按条复用」，
#   存放、组成清单与作废时报出变了的单元写在判定库 scripts/crash/lib_verdict_library.py 文件头。
#   判定库：真仓与 --staged 下在被判那棵树的 git common-dir 的 singlefs-formal-unit-verdicts/（GATE_FORMAL_VERDICT_LIBRARY 盖过它；2026-10-15 前的
#     singlefs-formal-verdicts/ 整个弃用、不读）；一项判完当场原子落盘，被打断之后同一条命令重跑只补没判的。
#     整道、--check、--item、--quick 都读（键相同就复用）；SINGLEFS_GATE_FULL=1 只写不读（--no-reuse），现判的照键写回同一份库。
#   每项一行「· <项>：绿|红，现判（<原因>：<变了的单元>）|复用」，每格一行「· 按条：现判 N、复用 M（现判的原因：…）」，整道末了一行「── 按条（…）：现判 N、复用 M」。
# 逐项点名（.claude/rules/verification.md「门禁的结构」里重型门禁的 --list-items / --item）：
#   --list-items 逐行打「项<TAB>格[<TAB>说明]」：toolchain、proof（整格）、table 各一项，证明每个有证明义务的单元一项 proof:<fn 名或 <类型>::clone>，空真每处 requires 一项
#     vacuity:<所属 fn>#<函数内第几处>，必红世界每行一项 must-red:<变异名>，红例钉表每行一项 pinned:<钉表行号>；只读登记表、核与两张表，不编、不起 Verus 与装置。
#     判定库目录（下面「判定库」一段，真仓与 --staged 下 git common-dir 里那一份）在列项之前定：列项时交给 verus-proof-check.py list-items --verdict-library，
#     它逐项照现在的键比库（只读库、不写库、不起 Verus），每格一行「<格>：键对不上 N 条、无记录 M 条、键相同 K 条、拒算 R 条」与末了「键对不上共 N 条」
#     原样接在项的后面、走 stderr（stdout 照旧只有项的那几行）；样本根不定判定库，末了说一句「列项没比键」。--item 照列出的项找格，不比键。
#   --item <项>（可给多次，取并集）只跑点了项的那几格，证明、空真、必红世界、红例重放四格只判点名的项（proof 一项是整格）；项名写错、缺项名退 2 并列出可点名的项；
#     与 --check、--list-items 不一起用（退 2）。
#   --quick（轻型，.claude/rules/verification.md「两档：轻型与重型」）：核心部分是 QUICK_CORE_ITEMS 那三项（toolchain、proof、table：工具链、证明、钉表），
#     再连同 --item 点名的项（可变部分，取并集）照 --item 那一路跑：与重型同一份判定库、同一套单元键，键相同就复用。
#     与 --check、--list-items 不一起用（退 2）。样本根放 .formal-sample/quick 当成给了 --quick（样本没法带参数）。
# 样本（fixtures/checker-tier-formal-verification.sh/<格名>-<情形>/，各放 .gate-cells 点名一格；共享门禁的 stage-selftest.sh 跑）。名字读不出判法的几份：
#   verus-toolchain-pinned-*：假的 Verus 安装与假的 cargo 缓存放在样本的 .formal-sample/ 下
#   verus-must-red-worlds-list-items：只列项（没给判定库，末了一句「列项没比键」）；-list-items-key-counts：另放 .formal-sample/list-items-library，
#     列项照空的判定库比键，每格「无记录」几条、末了「键对不上共 0 条」（读 Verus 的安装算工具链那一样，不起 Verus）
#   verus-proof-timeout-red：rlimit 设成无穷、样本限时 20 秒；verus-proof-killed-red：替身 Verus 起来就 kill -9 自己，判红，✗ 写「Verus 被信号停了」；
#   pinned-red-examples-replay-killed-red：桩装置打一行就 kill -9 自己，判红，✗ 写「装置被信号停了」；这两份另核汇总出路里「被信号停了」那一句（不改证明或实现）
#   verus-proof-kernel-absent（核没进仓）、pinned-red-examples-replay-device-absent（没有桩、装置源码不认 --pinned-cases）：退 77
#   pinned-red-examples-replay-rocksdb-entry-refused：同 device-absent（没有桩），另放依赖 rocksdb 的 crates/singlefs-checker-tier/Cargo.toml：
#     起装置的 cargo build 之前 rocksdb-prebuilt.py entry 判红（样本根没有 Cargo.lock，进程环境有没有取值都红），这一格退 1、装置没编
#   verus-vacuity-closure-red：外层函数的前置条件可满足、里面闭包的不可满足
#   verus-must-red-worlds-：-rlimit-red 改坏之后只剩 rlimit 用尽；-rlimit-with-proof-error-green 另有一条证明错（proof_errors=1，落点是那条证明错所在的函数）；
#     -legacy-expectation-red 第五段旧写法「verified, 1 errors」；-proof-error-count-drift-green 一条证明错、第五段写 proof_errors=2：条数对不上而落点没变，只报不判，判绿；
#     -landing-moved-red 条数对得上而第六段落点登记成别的函数，判红；-landing-missing-red 五段的旧行，判红并指到补落点的办法（C654）
#   same-origin-counter-*：只放 .claude/gate.d/divergence-events.tsv（不起 Verus）；文件头「弄坏开关」一段点名各让哪份判错
#   pinned-red-examples-table-green：根里没有 research/，钉表三行好，判绿；-table-archived-ledger-green：research/results/ 下有一份账、同一趟的 kill-bounds 产物不在（归档了），
#     钉表好，照判绿（这一格不读账与产物）；-table-red：一行的 ① 与 ②③⑦ 重算的不同；-table-variant-red：⑥ 点名的变异不在 crates/mutations.tsv 里了；
#     -table-profile-red：一行的 ㉒ 写成「--profile r1 --table direct」（默认值也写了，不是规范写法）
#   pinned-red-examples-replay-copied-columns-red：⑳ 与 ⑦ 不同；-replay-old-row-red：8 段的旧行
#   pinned-red-examples-replay-profiles-green：r1、r2 两档各三行，桩照命令行的 --profile 打（r2 档在 counters 后面多一个 j6qn），判绿；
#     -replay-profile-missing-red：有一行没写档（21 段），判红、出路给 --backfill
#   Verus 那几份样本起的是真的 Verus（SINGLEFS_VERUS_ROOT 或登记表 release 行的安装目录），没装就判红，与真仓一样；verus-proof-killed-red 起替身
#   （设了 SINGLEFS_VERUS_ROOT 时它盖过登记表、起的是真 Verus，这一份判错）。
#   被验区域与多 crate 的那几份（出口 5 第三轮判决第三节「门禁」；要拦的写法取自 research/prompts/m4-exit5-relocation-proof-r2-opus-model/verus/ 与
#   research/prompts/m4-exit5-relocation-proof-r3-sonnet-model/verus/ 里的例子，样本核的文件头写着取自哪一份）：
#     verus-proof-trust-unregistered-red（binary_search 的 assume_specification 没登记）、-trust-registered-green（登记齐了：规格原文 sha256、差分测试名，
#     不带 --no-cheating 照验、MIR 比对相同）、-trust-digest-red（登记的 sha256 不对）、-trust-test-missing-red（差分测试不在）、-trust-vstd-red（vstd 行版本不对）、
#     -axiom-red（axiom fn）、-axiom-broadcast-red（broadcast axiom）、-cfg-feature-red（#[cfg(feature)] 分叉的两份 pick）、-cfg-debug-assertions-red（cfg!(debug_assertions)）、
#     -cfg-macro-red（宏在 verus! 外面定义、展开里带 cfg!）、-mir-divergence-red（同一个宏的 cfg 登记了，静态那一关过了，红在 MIR 比对上：产品构建 start + 1、
#     verus --compile 编出 start + span）、-trait-impl-outside-red（被验 trait 在 verus! 外面的 impl；cfg(test) 下的不判）、-external-trait-spec-red
#     （external_trait_specification 没登记）、-verifier-external-red（#[verifier::external]）、-attribute-kernel-red（#[verus_verify] 标的文件没登记成核）、
#     -core-function-green（登记一个 core 小函数：crate 行第四段 cargo、带路径依赖 singlefs-format，impl 里的方法照全路径成项，cargo verus build 的 MIR 相同）；
#     verus-must-red-worlds-list-items-two-crates（两个 crate 逐个列项、全路径不重名）；verus-toolchain-pinned-rustc-red（rustc 行的产品 rustc 版本与 .formal-sample/rustc-version 不同）
# 只供样本的开关（被判的根不是这道阶段所在的仓、也没设 GATE_STAGED_FROM 时才认；真仓与 --staged 下放了也不认）：
#   .formal-sample/verus-install/         版本一致格改读这里的 version.json（不起它）
#   .formal-sample/cargo-home/            版本一致格改读这里的 registry/src/*/
#   .formal-sample/rustc-version          版本一致格把这份文件当产品 rustc -vV 的输出（不起 rustc；样本的 rustc 行照它写）
#   .formal-sample/verus-timeout-seconds  Verus 那几格的限时改成文件里的秒数
#   .formal-sample/e165-stub.sh           replay 格改用这个桩当装置（经 bash 起，不经内存包装）
#   .formal-sample/list-items             当成给了 --list-items（样本没法带参数）
#   .formal-sample/list-items-library     列项时判定库放这一趟的空临时目录（列完删掉）：Verus 那几格逐项比它，样本 verus-must-red-worlds-list-items-key-counts 看「无记录」与「键对不上共 0 条」
#   .formal-sample/items                  每一行当成一个 --item（命令行给了 --item 就不看它）
#   .formal-sample/quick                  当成给了 --quick（样本 quick-green、quick-red：核没进仓，toolchain 与 proof 两项本次未跑，table 照钉表判绿 / 判红）
#   .formal-sample/run-twice              判定库放这一趟的临时目录、读，整道接连跑两趟：第二趟按条全复用（按条复用的样本）
# 弄坏开关：格的判法那一份的开关 VERUS_PROOF_CHECK_BREAK=<项> 与 E165_PIN_RED_EXAMPLES_BREAK=<项>（逐项让哪份样本判错写在两份脚本的文件头）；
#   汇总与 --check 的归共用库（STAGE_CELLS_BREAK=red-swallowed 让每份红样本整道退 0；marker-ignored 让每份样本七格都跑，汇总那条「跑了 1 格」找不到）。
#   逐项点名与按条复用：判法那一半的开关在两份脚本文件头「逐项点名与按条复用」；这一道自己接的那一半 FORMAL_VERIFICATION_BREAK=<项>[,<项>…]，
#   自证是 scripts/gate/admission.py --selftest 的 run_formal_item_cells（临时仓里跑这份脚本、两份判法换成记参数的假驱动）：
#     items-unchecked       --item 写错的项名悄悄丢掉、不退 2        自证「--item 写错退 2」那一条红；样本 pinned-red-examples-replay-item-unknown 判错
#     reuse-on-full-run     SINGLEFS_GATE_FULL=1 照样读判定库（不转 --no-reuse）   自证「SINGLEFS_GATE_FULL=1 只记不读」那一条红
#     library-in-tmpdir     判定库放 $TMPDIR（门禁每轮新给、跑完删）而不是 git common-dir   自证「判定库在 git common-dir」那一条红
#     quick-core-unselected  --quick 不把核心部分那三项放进点名，照整道跑                    样本 quick-green 判错（输出里找不到「--quick：核心部分」）
#     list-items-without-library  --list-items 不把判定库交给 verus-proof-check.py list-items（键对不上几条不报）   样本 verus-must-red-worlds-list-items-key-counts 判错
#     rocksdb-entry-unchecked  pinned-red-examples-replay 不带 --binary 时起装置的 cargo build 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
#                              样本 pinned-red-examples-replay-rocksdb-entry-refused 判错（输出里找不到「rocksdb-prebuilt entry：」，退 77：装置源码不认 --pinned-cases）
#   被验区域与多 crate（判法在 scripts/gate/verus-proof-check.py，那份文件头「被验区域与多 crate 的弄坏开关」一段另列打开之后自证哪几格判错）的开关，
#   开关一行、下一行起写打开之后判错的样本：
#   FORMAL_VERIFICATION_BREAK=single-crate-registry  登记表只认第一个 crate
#       verus-must-red-worlds-list-items-two-crates（core 那几项列不出来）
#   FORMAL_VERIFICATION_BREAK=cargo-mode-ignored  crate 行第四段 cargo 不认、照直接起 verus 判
#       verus-proof-core-function-green（判红：除 vstd 之外还依赖 singlefs-format）
#   FORMAL_VERIFICATION_BREAK=methods-refused  impl 与 trait 里的方法判红、要挪成自由 fn
#       verus-proof-core-function-green、verus-must-red-worlds-list-items-two-crates
#   FORMAL_VERIFICATION_BREAK=attribute-mode-unrecognized  #[verus_verify] 标的文件不当核
#       verus-proof-attribute-kernel-red（判绿）
#   FORMAL_VERIFICATION_BREAK=no-cheating-kept  照旧带 --no-cheating
#       verus-proof-trust-registered-green（判红：Verus 拒 assume_specification）
#   FORMAL_VERIFICATION_BREAK=trust-list-unchecked  信任清单不判
#       verus-proof-trust-unregistered-red、verus-proof-external-trait-spec-red、verus-proof-trust-test-missing-red、verus-proof-trust-digest-red、verus-proof-trust-vstd-red
#   FORMAL_VERIFICATION_BREAK=trust-digest-unchecked  不比登记的规格原文 sha256
#       verus-proof-trust-digest-red（判绿）
#   FORMAL_VERIFICATION_BREAK=trust-test-unchecked  不核差分测试名
#       verus-proof-trust-test-missing-red（判绿）
#   FORMAL_VERIFICATION_BREAK=trust-vstd-unchecked  不核信任清单的 vstd 行
#       verus-proof-trust-vstd-red（判绿）
#   FORMAL_VERIFICATION_BREAK=axiom-accepted  核里的 axiom 不判
#       verus-proof-axiom-red、verus-proof-axiom-broadcast-red（找不到「核里有 axiom」）
#   FORMAL_VERIFICATION_BREAK=cfg-atom-accepted  被验区域里的 cfg 原子不认
#       verus-proof-cfg-feature-red、verus-proof-cfg-debug-assertions-red、verus-proof-cfg-macro-red（找不到「没登记进信任清单」）
#   FORMAL_VERIFICATION_BREAK=cfg-macro-unchecked  定义里有 cfg 的宏在被验区域里的调用不认
#       verus-proof-cfg-macro-red（只剩 MIR 比对那一句红，找不到「#plus_span! 没登记进信任清单」）
#   FORMAL_VERIFICATION_BREAK=trait-impl-outside-accepted  被验 trait 在 verus! 外面的 impl 不判
#       verus-proof-trait-impl-outside-red（判绿）
#   FORMAL_VERIFICATION_BREAK=mir-compare-skipped  不比产品构建与 Verus 构建的 MIR
#       verus-proof-mir-divergence-red（判绿）、verus-proof-trust-registered-green、verus-proof-core-function-green、verus-proof-green（找不到 MIR 那一句）
#   FORMAL_VERIFICATION_BREAK=rustc-pin-unchecked  不核登记表 rustc 行
#       verus-toolchain-pinned-rustc-red（判绿）
#   同源性计数器格（same-origin-counter；判法在本文件 cell_same_origin_counter，表是 .claude/gate.d/divergence-events.tsv）的开关，开关一行、下一行起写打开之后判错的样本：
#   FORMAL_VERIFICATION_BREAK=same-origin-zero-spec-wrong-unchecked  不判「累计 ≥ 5 次而 spec 错 0 次」
#       same-origin-counter-red（判绿）
#   FORMAL_VERIFICATION_BREAK=same-origin-clause-gap-counted  条款欠一句也计入累计
#       same-origin-counter-clause-gap-green（判红）
#   FORMAL_VERIFICATION_BREAK=same-origin-unbuilt-reported-green  未立的对当已立的对数、报绿
#       same-origin-counter-green（找不到「已立 2 对、未立 1 对」）、same-origin-counter-red（汇总里「未立 1 对」找不到）、same-origin-counter-unbuilt-event-red（判绿）
#   FORMAL_VERIFICATION_BREAK=same-origin-unbuilt-events-accepted  未立的对收了事件也不判红
#       same-origin-counter-unbuilt-event-red（判绿）
#   红例两格的开关，开关一行、下一行起写打开之后判错的样本（自证格写在 e165-pin-red-examples.py 文件头）：
#   E165_PIN_RED_EXAMPLES_BREAK=table-check-globs-research  table 格照改前的判法：取 research/results/e165-*-red-examples.tsv、按文件名配产物判钉全
#       pinned-red-examples-table-archived-ledger-green（判红：产物不在）、pinned-red-examples-table-green（退 77：没有账）
#   E165_PIN_RED_EXAMPLES_BREAK=fingerprint-unchecked  不拿 ②③⑦ 重算 ①
#       pinned-red-examples-table-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=variant-unchecked  table 格与 replay 格不判变异还在不在
#       pinned-red-examples-table-variant-red、pinned-red-examples-replay-variant-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=profile-form-unchecked  不核 ㉒ 是不是规范写法的档
#       pinned-red-examples-table-profile-red（判绿）
#   E165_PIN_RED_EXAMPLES_BREAK=old-rows-accepted  表里 8 段的旧行照收、不报没整行抄
#       pinned-red-examples-replay-old-row-red（判绿：桩照 ⑦ 打）
#   E165_PIN_RED_EXAMPLES_BREAK=copied-columns-unchecked  不核 ⑪⑫⑬ 与 ②③④、⑳㉑ 与 ⑦⑧
#       pinned-red-examples-replay-copied-columns-red（判绿：桩照 ⑦ 打）
#   E165_PIN_RED_EXAMPLES_BREAK=replay-without-profile  replay 格起装置不带 ㉒ 的档参数
#       pinned-red-examples-replay-profiles-green（判红：桩一律按 r1 打，r2 那三行与 ⑦ 不同）
#   E165_PIN_RED_EXAMPLES_BREAK=profile-column-unchecked  21 段的行（没写档）照收、当 --profile r1
#       pinned-red-examples-replay-profile-missing-red（判绿：桩照 ⑦ 打）
#
#   bash .claude/gate.d/checker-tier-formal-verification.sh [项目根]
#   bash .claude/gate.d/checker-tier-formal-verification.sh --list        逐行打格名与判什么，不起 Verus、不编
#   bash .claude/gate.d/checker-tier-formal-verification.sh --list-items  逐行打可点名的项与它属于哪一格，不起 Verus、不编
#   bash .claude/gate.d/checker-tier-formal-verification.sh --item <项> [--item <项>…] [项目根]   只跑点名的项
#   bash .claude/gate.d/checker-tier-formal-verification.sh --quick [--item <项>…] [项目根]      轻型：核心部分三项，再加点名的项
#   bash .claude/gate.d/checker-tier-formal-verification.sh --tool-versions   打 Verus 的版本与安装文件的 sha256、安装目录顶层的整体摘要，VERUS_Z3_PATH、VERUS_ROOT、
#     VERUS_USE_RUSTUP 三个变量的取值，以及 Verus 起 rust_verify 用的那条 rustup 工具链：RUSTUP_HOME 与它解析到的路径、工具链 lib/ 与 lib/rustlib/*/lib/ 下全部文件的
#     整体摘要、rustup 程序的 sha256（Verus 每条判定的键里「工具链」那一样就是这一行的摘要：工具链里换了一个文件，Verus 的每一条都现判；
#     逐样的写法在 scripts/gate/verus-proof-check.py 文件头 tool-versions 一段；不起 Verus，不算重型；本机多哈希约 500 MB，页缓存热时多约 0.25 秒）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
STAGE_REPOSITORY="$(cd "$(dirname "$0")/../.." && pwd)"
VERUS_CHECK="$STAGE_REPOSITORY/scripts/gate/verus-proof-check.py"
PIN_CHECK="$STAGE_REPOSITORY/scripts/experiment-apparatus/e165-pin-red-examples.py"
source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"
stage_cell verus-toolchain-pinned cell_verus_toolchain_pinned "Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹、Verus 与产品的 rustc 版本都与登记相同" \
  "装登记的 Verus 发布或用 SINGLEFS_VERUS_ROOT 指到它；缓存不对就删掉那个包目录再 cargo fetch --locked；要换版本先照 .claude/gate.d/verus-pins.tsv 的注释复核同源再改登记"
stage_cell verus-proof cell_verus_proof "Verus 逐项验登记的各 crate 被验区域里有证明义务的单元（按全路径：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 的 clone，impl 与 trait 里的方法也算），每项限时内出结局、errors 0、verified 至少 1，各 crate 各项 verified 之和等于它登记的数，整格再完整验证 crate、产品构建与 verus --compile 构建的 MIR 逐条目相同；起 Verus 之前判被验区域：可信声明与 cfg 原子照信任清单逐条登记，axiom、作弊标记与 verus! 外面被验类型的 trait impl 判红" \
  "照 Verus 报错的位置修证明或实现；verified 数变了而确是加减了证明，就改 .claude/gate.d/verus-pins.tsv 的 crate 行并写明加减了哪几个；超时先查哪条查询慢，不许靠放宽限时了事；✗ 行写「没跑成：内存包装」的照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办，不改证明或实现；✗ 行写「被信号停了」的照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办，这一次的输出不算判定，不改证明或实现"
stage_cell verus-vacuity cell_verus_vacuity "核里每一处 requires 的函数体头插 assert(false) 都验不过（未检出入口空真；可满足性另由构造见证证明）" \
  "验得过的那一处前置条件不可满足：改前置条件让它可满足；插入点认错了就按 scripts/gate/verus-proof-check.py 的 requires_insertion_points 改写法；✗ 行写「没跑成：内存包装」的照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办，不改前置条件"
stage_cell verus-must-red-worlds cell_verus_must_red_worlds "先查函数与不变式负例覆盖；必红世界表每行改坏核之后 Verus 都报错、证明错落在第六段 landing= 登记的函数里（条数只报不判），原文恰好命中一次" \
  "锚点腐化就照今天的核改那一行的原文；改坏了还验得过就补证明或收紧 spec；落点换了先确认红的还是这条变异要盯的那一处（落到别的函数多半是证明或 spec 改过，改证明或换改法），确是本意再照红句里的实测改第六段 landing=<函数名>[,<函数名>…]；条数跳了不用管，第五段 proof_errors=<证明错条数> 只报不判；五段的旧行跑 python3 scripts/gate/verus-proof-check.py must-red-landings <根> --write 补第六段（重型，由主 agent 起）；表在 .claude/gate.d/verus-pins.tsv 的 must-red-worlds 行点名的路径；✗ 行写「没跑成：内存包装」的照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办，不改表或证明"
stage_cell pinned-red-examples-table cell_pinned_red_examples_table "钉表 e165-pinned-red-examples.tsv 自己：每行 22 段整行抄带档、第 1 段与第 2、3、7 段重算的指纹相同、抄来的账行与自己的段对得上、第 22 段是规范写法的档，点名的变异还在它的表里；不读红例账、kill-bounds 产物与 research/ 下任何文件" \
  "表只由 scripts/experiment-apparatus/e165-pin-red-examples.py 写：8 段的旧行与 21 段没写档的行跑 python3 scripts/experiment-apparatus/e165-pin-red-examples.py --backfill；指纹、抄来的段或档的写法对不上是手改坏了，从 git 找回那一行；变异行不在了就找回那一行，确是改名就照新名改这张表（要写明为什么）；新出的红例由 E165 第二段驱动写完账当场钉，这一格不补钉"
stage_cell pinned-red-examples-replay cell_pinned_red_examples_replay "钉住的每一行是 22 段整行抄带档、抄来的账行与自己的段对得上，点名的变异还在它的表里，装置按档在今天的实现上逐行打的结果与原实现期望的逐态行逐字节相同" \
  "8 段的旧行与 21 段没写档的行跑 python3 scripts/experiment-apparatus/e165-pin-red-examples.py --backfill 补成 22 段；抄来的账行与自己的段对不上是手改坏了，从 git 找回再钉；变异行不在了就找回那一行；装置逐行对不上是今天的实现在钉住的输入上与原实现不同，按回归修实现；装置报指纹漂了先查它的枚举次序与状态编码；✗ 行写「没跑成：内存包装」的照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办，不改变异行或实现；✗ 行写「被信号停了」的照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办，这一次的输出不算判定，不改变异行或实现"
stage_cell same-origin-counter cell_same_origin_counter "同源性计数器：不一致事件表 divergence-events.tsv 里每个已立的对累计 ≥ 5 次不一致而判「spec 错」0 次即判红；未立的对报未立、不计入，条款欠一句不计入累计" \
  "红的那一对累计 ≥ 5 次不一致而 spec 错 0 次，说明两条腿实际同源：D13 已定项 13 定的处置是把它们合并、预算移给穷举，这是设计决定，交主 agent 问用户，不靠改表销账；表里确有一次判错了（该是 spec 错）就改那一行的判谁错、出处写重判的记录；表写坏了（日期、对名、判谁错、出处，或未立的对收了事件）照红行点名的那一行改，格式在 .claude/gate.d/divergence-events.tsv 文件头"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}
root_argument=""
tool_versions_requested=0
list_items_requested=0
quick_requested=0
requested_items=()
QUICK_CORE_ITEMS=(toolchain proof table)   # 轻型的核心部分：工具链、证明、钉表（.claude/rules/verification.md「两档：轻型与重型」）
while (($#)); do
  case "$1" in
    --tool-versions) tool_versions_requested=1; shift ;;
    --list-items) list_items_requested=1; shift ;;
    --quick) quick_requested=1; shift ;;
    --item|--item=*)
      if [[ "$1" == --item ]]; then
        if (($# < 2)) || [[ -z "$2" ]]; then
          echo "  ✗ --item 后面缺项名"
          echo "     → 怎么办：写成 --item <项>（可给多次）；可点名的项用 bash .claude/gate.d/$(basename "$0") --list-items 列。"
          exit 2
        fi
        requested_items+=("$2"); shift 2
      else
        requested_items+=("${1#--item=}"); shift
      fi ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--list-items、--item <项>、--quick、--tool-versions 与一个项目根。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不了项目根 $ROOT"; echo "     → 怎么办：第一个参数给被判的仓根（不给就判这道阶段所在的仓）。"; exit 2; }
ROOT="$(pwd)"
if ((tool_versions_requested)); then
  exec python3 "$VERUS_CHECK" tool-versions "$ROOT"
fi

sample_mode=0
if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then sample_mode=1; fi
# 只供样本的点名开关：.formal-sample/list-items 在就当成给了 --list-items，.formal-sample/items 的每一行当成一个 --item（命令行给了的不看它）
if ((sample_mode)) && [[ -f .formal-sample/list-items ]]; then list_items_requested=1; fi
if ((sample_mode)) && ((${#requested_items[@]} == 0)) && [[ -f .formal-sample/items ]]; then
  while IFS= read -r sample_item || [[ -n "$sample_item" ]]; do
    [[ -n "$sample_item" && "$sample_item" != \#* ]] && requested_items+=("$sample_item")
  done < .formal-sample/items
fi
if ((sample_mode)) && [[ -f .formal-sample/quick ]]; then quick_requested=1; fi
# --quick（文件头「逐项点名」一段的 --quick）：核心部分三项并进点名，照 --item 那一路跑（与重型同一份判定库，键相同就复用）
if ((quick_requested)); then
  if ((list_items_requested)) || ((STAGE_CELLS_CHECK_GIVEN)); then
    echo "  ✗ --quick 不与 --list-items、--check 一起用：--quick 已经定了跑哪几项"
    echo "     → 怎么办：只写 --quick（可再带 --item <项>）；要列项单独跑 --list-items，要整格跑只写 --check <格名>。"
    exit 2
  fi
  quick_named_count=${#requested_items[@]}
  if [[ ",${FORMAL_VERIFICATION_BREAK:-}," != *,quick-core-unselected,* ]]; then
    requested_items=("${QUICK_CORE_ITEMS[@]}" ${requested_items[@]+"${requested_items[@]}"})
    echo "  · --quick：核心部分 ${QUICK_CORE_ITEMS[*]} 三项$( ((quick_named_count)) && echo "，另加点名的 $quick_named_count 项" )；与重型同一份判定库，键相同就复用"
  fi
fi

# 判定库（scripts/crash/lib_verdict_library.py）：真仓与 --staged 下放被判那棵树的 git common-dir 里（跨轮留着，几棵 worktree 共用），
#   GATE_FORMAL_VERDICT_LIBRARY 盖过它；没设 SINGLEFS_GATE_FULL=1 就读（键相同就复用），设了只记不读（--no-reuse）。
#   样本不读不写，只有放了 .formal-sample/run-twice 的那份：判定库放这一趟的临时目录、读，整道接连跑两趟（第二趟应当全复用）。
#   --list-items 也认这一份目录（列项时 Verus 那几格逐项照现在的键比它，报键对不上几条），所以在列项之前定；列项不建临时的那一份。
verdict_library=""
verdict_rounds=1
verdict_library_is_temporary=0
if ((sample_mode == 0)); then
  if [[ -n "${GATE_FORMAL_VERDICT_LIBRARY:-}" ]]; then
    verdict_library="$GATE_FORMAL_VERDICT_LIBRARY"
  elif common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]]; then
    verdict_library="$common_directory/singlefs-formal-unit-verdicts"
    [[ ",${FORMAL_VERIFICATION_BREAK:-}," == *,library-in-tmpdir,* ]] && verdict_library="${TMPDIR:-/tmp}/singlefs-formal-unit-verdicts"
  elif ((list_items_requested)); then
    echo "  ! 取不到 $ROOT 的 git common-dir，判定库没处放：列项不报键对不上几条" >&2
  else
    echo "  ! 取不到 $ROOT 的 git common-dir，判定库没处放：这一趟每一项都现判、不记"
  fi
elif { [[ -f .formal-sample/run-twice ]] && ((list_items_requested == 0)); } || { [[ -f .formal-sample/list-items-library ]] && ((list_items_requested)); }; then
  verdict_library="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  verdict_library_is_temporary=1
  ((list_items_requested)) || verdict_rounds=2
  # 下面点名写错退 2、列完项退 0 这类提前退出的路上也删掉它（末了照旧删一次；格在子 shell 里跑，不继承这个陷阱）
  trap 'rm -rf "${verdict_library:?}"' EXIT
fi

# 可点名的项（--list-items 打它、--item 照它核）：两份判法各列自己那几格的，「项<TAB>格[<TAB>说明]」；只读登记表、核、必红世界表与钉表，不编、不起 Verus 与装置。
#   带参数 with-library（--list-items 那一路）且判定库目录定了：Verus 那份另带 --verdict-library <判定库>，它往 stderr 报的每格「键对不上 N 条…」
#   与末了「键对不上共 N 条」收进 formal_items_report_file，列完项之后原样接在后面（stderr，项的那几行 stdout 照旧只有「项<TAB>格[<TAB>说明]」）
formal_items_file=""
formal_items_report_file=""
list_formal_items() {
  formal_items_file="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  formal_items_report_file="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${formal_items_file:?}"; exit 1; }
  local listing_exit
  local library_arguments=()
  if [[ "${1:-}" == with-library && -n "$verdict_library" && ",${FORMAL_VERIFICATION_BREAK:-}," != *,list-items-without-library,* ]]; then
    library_arguments=(--verdict-library "$verdict_library")
  fi
  if python3 "$VERUS_CHECK" list-items "$ROOT" ${library_arguments[@]+"${library_arguments[@]}"} > "$formal_items_file" 2> "$formal_items_report_file"; then listing_exit=0; else listing_exit=$?; fi
  if ((listing_exit != 0)); then
    cat "$formal_items_report_file" >&2
    echo "  ✗ 列不出 Verus 那几格的项：scripts/gate/verus-proof-check.py list-items 退 $listing_exit"
    echo "     → 怎么办：照它上面打的红行修登记表 .claude/gate.d/verus-pins.tsv 或必红世界表；列项只读文件，不起 Verus。"
    rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"; exit 1
  fi
  if python3 "$PIN_CHECK" --list-items --root "$ROOT" >> "$formal_items_file"; then listing_exit=0; else listing_exit=$?; fi
  if ((listing_exit != 0)); then
    cat "$formal_items_report_file" >&2
    echo "  ✗ 列不出红例两格的项：scripts/experiment-apparatus/e165-pin-red-examples.py --list-items 退 $listing_exit"
    echo "     → 怎么办：照它上面打的红行修钉表 .claude/gate.d/e165-pinned-red-examples.tsv；列项只读文件，不编装置。"
    rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"; exit 1
  fi
}
if ((list_items_requested)); then
  if ((${#requested_items[@]})) || ((STAGE_CELLS_CHECK_GIVEN)); then
    echo "  ✗ --list-items 只列项，不与 --item、--check 一起用"
    echo "     → 怎么办：先单独跑 --list-items 看项名，再用 --item <项> 点名跑。"
    exit 2
  fi
  list_formal_items with-library
  cat "$formal_items_file"
  # 键对不上的条数（Verus 那份给了判定库时才有）接在项后面、走 stderr；没给判定库时这份是空的，另说一句没比
  if [[ -s "$formal_items_report_file" ]]; then
    cat "$formal_items_report_file" >&2
  elif [[ -z "$verdict_library" ]]; then
    echo "  · 判定库没定（样本根，或取不到 git common-dir）：列项没比键" >&2
  fi
  rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"
  exit 0
fi
# --item：点名的项各属哪一格照列出的表找；格只跑点了项的那几格（次序照登记），vacuity、must-red、pinned 那几格只判点名的项（转给判法的 --item）
declare -A item_options_of_cell=()
if ((${#requested_items[@]})); then
  if ((STAGE_CELLS_CHECK_GIVEN)); then
    echo "  ✗ --item 与 --check 不一起用：--item 已经定了跑哪几格"
    echo "     → 怎么办：只写 --item <项>（可给多次）；要整格跑就只写 --check <格名>。"
    exit 2
  fi
  list_formal_items
  declare -A cell_of_item=()
  while IFS=$'\t' read -r listed_item listed_cell _listed_note; do
    [[ -n "$listed_item" ]] && cell_of_item[$listed_item]="$listed_cell"
  done < "$formal_items_file"
  unknown_items=()
  declare -A cells_wanted=()
  for wanted_item in "${requested_items[@]}"; do
    if [[ -z "${cell_of_item[$wanted_item]+set}" ]]; then
      [[ ",${FORMAL_VERIFICATION_BREAK:-}," == *,items-unchecked,* ]] || unknown_items+=("$wanted_item")
      continue
    fi
    cells_wanted[${cell_of_item[$wanted_item]}]=1
    case "${cell_of_item[$wanted_item]}" in
      verus-proof|verus-vacuity|verus-must-red-worlds|pinned-red-examples-replay)
        item_options_of_cell[${cell_of_item[$wanted_item]}]+="$wanted_item"$'\n' ;;
    esac
  done
  if ((${#unknown_items[@]})); then
    echo "  ✗ 没有这些项：${unknown_items[*]}"
    echo "     → 怎么办：项名照下面这张表写（第一列是项、第二列是格），给几次 --item 取并集："
    cut -f1,2 "$formal_items_file" | sed 's/^/       /'   # gate-lint:detail
    rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"
    exit 2
  fi
  rm -f "${formal_items_file:?}" "${formal_items_report_file:?}"
  STAGE_CELLS_SELECTED=()
  for cell_name in "${STAGE_CELLS_NAMES[@]}"; do
    [[ -n "${cells_wanted[$cell_name]+set}" ]] && STAGE_CELLS_SELECTED+=("$cell_name")
  done
  STAGE_CELLS_CHECK_GIVEN=1
  STAGE_CELLS_SELECTION_SOURCE="--item 点名 ${#requested_items[@]} 项"
fi
# 把这一格点名的项交成判法的 --item 参数（一项一对），交到 item_arguments 数组里
item_arguments_of() {
  item_arguments=()
  local item_line
  while IFS= read -r item_line; do
    [[ -n "$item_line" ]] && item_arguments+=(--item "$item_line")
  done <<<"${item_options_of_cell[$1]:-}"
}
toolchain_options=()
verus_options=()
replay_options=()
if ((sample_mode)); then
  [[ -d .formal-sample/verus-install ]] && toolchain_options+=(--verus-root "$ROOT/.formal-sample/verus-install")
  [[ -d .formal-sample/cargo-home ]] && toolchain_options+=(--cargo-home "$ROOT/.formal-sample/cargo-home")
  [[ -f .formal-sample/rustc-version ]] && toolchain_options+=(--rustc-version-file "$ROOT/.formal-sample/rustc-version")
  [[ -f .formal-sample/verus-timeout-seconds ]] && verus_options+=(--timeout "$(tr -dc '0-9' < .formal-sample/verus-timeout-seconds)")
  [[ -f .formal-sample/e165-stub.sh ]] && replay_options+=(--binary "$ROOT/.formal-sample/e165-stub.sh" --memory-cap none)
fi
if [[ -n "$verdict_library" ]]; then
  library_options=(--verdict-library "$verdict_library")
  if [[ "${SINGLEFS_GATE_FULL:-}" == 1 && ",${FORMAL_VERIFICATION_BREAK:-}," != *,reuse-on-full-run,* ]]; then library_options+=(--no-reuse); fi
  verus_options+=("${library_options[@]}")
  replay_options+=("${library_options[@]}")
fi

cell_verus_toolchain_pinned() { python3 "$VERUS_CHECK" toolchain "$ROOT" ${toolchain_options[@]+"${toolchain_options[@]}"}; }
cell_verus_proof() { item_arguments_of verus-proof; python3 "$VERUS_CHECK" proof "$ROOT" ${verus_options[@]+"${verus_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}; }
cell_verus_vacuity() { item_arguments_of verus-vacuity; python3 "$VERUS_CHECK" vacuity "$ROOT" ${verus_options[@]+"${verus_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}; }
cell_verus_must_red_worlds() { python3 "$VERUS_CHECK" coverage "$ROOT" || return $?; item_arguments_of verus-must-red-worlds; python3 "$VERUS_CHECK" must-red-worlds "$ROOT" ${verus_options[@]+"${verus_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}; }
cell_pinned_red_examples_table() { python3 "$PIN_CHECK" --check-table --root "$ROOT"; }
cell_pinned_red_examples_replay() {
  # 前面 Verus 那几格已经排不上（判法写了 $queue_refused_record）：这一格不编装置、不起重放，不再排一次队
  if [[ -s "$queue_refused_record" ]]; then
    local refused_where="" refused_howto=""
    IFS=$'\t' read -r refused_where refused_howto < "$queue_refused_record"
    echo "  ✗ 没跑成：内存包装排不上，没再排队（同一道里 $refused_where）：这一格没编装置、没起重放"
    echo "     → 怎么办：${refused_howto:-照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办}"
    return 1
  fi
  item_arguments_of pinned-red-examples-replay
  # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：不带 --binary（样本的桩）时 e165-pin-red-examples.py --replay 起
  #   cargo build -p singlefs-checker-tier（verdict-store 默认开、编到 librocksdb-sys）：起它之前判一次这台装了、此刻进程环境的取值对得上，判红这一格退 1、装置不编。
  #   树里没有依赖 rocksdb 的清单（别的样本）、PATH 上是带标记的假 cargo，entry 报不判。弄坏开关 FORMAL_VERIFICATION_BREAK=rocksdb-entry-unchecked 下不判
  #   （样本 pinned-red-examples-replay-rocksdb-entry-refused 判错）。这一份被拷到不带 scripts/hosts/ 的树里时报一行没判、往下走
  if [[ " ${replay_options[*]-} " != *" --binary "* && ",${FORMAL_VERIFICATION_BREAK:-}," != *,rocksdb-entry-unchecked,* ]] \
     && [[ ! -f "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" ]]; then
    echo "  ! RocksDB 预编库的取值没判：$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py 不在（这一份阶段被拷到了不带 scripts/hosts/ 的树里）"
  elif [[ " ${replay_options[*]-} " != *" --binary "* && ",${FORMAL_VERIFICATION_BREAK:-}," != *,rocksdb-entry-unchecked,* ]] \
     && ! python3 "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" entry --root "$ROOT"; then
    echo "  ✗ 起 cargo 编装置之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），这一格没编装置、没起重放"
    echo "     → 怎么办：照上面的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再只跑这一格：bash .claude/gate.d/checker-tier-formal-verification.sh --check pinned-red-examples-replay"
    return 1
  fi
  python3 "$PIN_CHECK" --replay --root "$ROOT" ${replay_options[@]+"${replay_options[@]}"} ${item_arguments[@]+"${item_arguments[@]}"}
}

# ── 格 same-origin-counter：还 checks-owed.md C191（同源性计数器只盖了一对）。D13 已定项 13「冲突 2」的判据逐字：同源性要被观测，统计 spec 与 model 不一致的事件数
# 及每次判定哪一方错，累计 ≥ 5 次不一致而判「spec 错」的次数为 0，说明两者实际同源，处置是把两条腿合并、预算移给穷举。
# 读被判的根里 .claude/gate.d/divergence-events.tsv（格式在那张表自己的文件头）：「对」行登记哪一对、哪边算 spec、已立还是未立，「事件」行是一次不一致（日期、对名、判谁错、出处、说明）。
#   已立的对：累计 = spec 错 + model 错的行数；累计 ≥ 5 而 spec 错 0 次判红；条款欠一句的行先记账，不计入累计、不计入 spec 错（D13 第三格）；
#             每一对都把累计次数报出来，0 次只说「还没有事件进表」，不当作验过独立。
#   未立的对：报「未立」，不报绿、不计入任何一格；表里有点名它的事件行判红（未立的对不收事件）。
#   表写坏了判红：不是「对」「事件」行、段数不对、对名重复或不合写法、状态不是已立 / 未立、日期不是 YYYY-MM-DD、事件点名没登记的对、判谁错不是 spec错 / model错 / 条款欠一句、出处不是仓内路径起头。
#   一对已立的都没有：退 77（本次未跑），不当通过。
# 样本：same-origin-counter-red（一对已立的对 5 次不一致全判 model 错，判红）、same-origin-counter-green（一对 5 次里有 1 次 spec 错、一对 2 次 model 错加 2 次条款欠一句，
#   一对未立；判绿，报「已立 2 对、未立 1 对」）、same-origin-counter-clause-gap-green（4 次 model 错加 3 次条款欠一句：条款欠一句不计入累计，判绿）、
#   same-origin-counter-unbuilt-event-red（未立的对收了事件，判红）、same-origin-counter-verdict-red（判谁错写成表外的词，判红）。
# 弄坏开关 FORMAL_VERIFICATION_BREAK=same-origin-zero-spec-wrong-unchecked / same-origin-clause-gap-counted / same-origin-unbuilt-reported-green /
#   same-origin-unbuilt-events-accepted，各让哪份样本判错写在本文件头「弄坏开关」一段。
cell_same_origin_counter() {
  python3 - "$ROOT" "${FORMAL_VERIFICATION_BREAK:-}" <<'PY'
import datetime, os, re, sys
root, broken = sys.argv[1], {item for item in sys.argv[2].split(",") if item}
TABLE = ".claude/gate.d/divergence-events.tsv"
THRESHOLD = 5
VERDICTS = ("spec错", "model错", "条款欠一句")
STATES = ("已立", "未立")
path = os.path.join(root, TABLE)
if not os.path.isfile(path):
    print(f"  ✗ 同源性计数器的不一致事件表 {TABLE} 不在")
    print("     → 怎么办：它随仓走（checks-owed.md C191）；被删了从 git 找回，格式写在表自己的文件头")
    sys.exit(1)
pairs, events, problems = {}, [], []
for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
    if not raw.strip() or raw.startswith("#"):
        continue
    fields = raw.split("\t")
    where = f"{TABLE} 第 {number} 行"
    if fields[0] == "对":
        if len(fields) != 6 or not all(fields[1:]):
            problems.append(f"{where}：「对」行要六段（对、对名、spec 一方、model 一方、已立|未立、说明），都不许空，读到 {len(fields)} 段")
        elif not re.fullmatch(r"[a-z0-9][a-z0-9-]*", fields[1]):
            problems.append(f"{where}：对名「{fields[1]}」只许小写字母、数字、连字符")
        elif fields[1] in pairs:
            problems.append(f"{where}：对名「{fields[1]}」登记了两次（第 {pairs[fields[1]]['number']} 行）")
        elif fields[4] not in STATES:
            problems.append(f"{where}：状态「{fields[4]}」要写已立或未立")
        else:
            pairs[fields[1]] = {"number": number, "spec": fields[2], "model": fields[3], "state": fields[4], "note": fields[5]}
    elif fields[0] == "事件":
        if len(fields) != 6 or not all(fields[1:]):
            problems.append(f"{where}：「事件」行要六段（事件、日期、对名、判谁错、出处、说明），都不许空，读到 {len(fields)} 段")
            continue
        try:
            if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", fields[1]):
                raise ValueError
            datetime.date.fromisoformat(fields[1])
        except ValueError:
            problems.append(f"{where}：日期「{fields[1]}」不是 YYYY-MM-DD")
            continue
        if fields[3] not in VERDICTS:
            problems.append(f"{where}：判谁错「{fields[3]}」要写 {' | '.join(VERDICTS)} 之一")
            continue
        if not re.match(r"[A-Za-z0-9_.-]+/\S+", fields[4]):
            problems.append(f"{where}：出处「{fields[4]}」要以仓内相对路径起头（records/…、research/…、.claude/kb/… 这类，可后接行号）")
            continue
        events.append({"number": number, "date": fields[1], "pair": fields[2], "verdict": fields[3]})
    else:
        problems.append(f"{where}：第一段「{fields[0]}」不是「对」或「事件」")
if not pairs and not problems:
    problems.append(f"{TABLE} 里一个「对」都没有：没有要观测的对")
for event in events:
    if event["pair"] not in pairs:
        problems.append(f"{TABLE} 第 {event['number']} 行：事件点名的对「{event['pair']}」没有「对」行登记")
built, unbuilt, lines = [], [], []
for name, pair in pairs.items():
    mine = [event for event in events if event["pair"] == name]
    if pair["state"] == "未立" and "same-origin-unbuilt-reported-green" not in broken:
        unbuilt.append(name)
        lines.append(f"  · {name}：未立（{pair['note']}）——不报绿、不计入任何一格")
        if mine and "same-origin-unbuilt-events-accepted" not in broken:
            problems.append(f"{TABLE} 第 {mine[0]['number']} 行：对「{name}」还是未立，不收事件（共 {len(mine)} 行点名了它）")
        continue
    built.append(name)
    spec_wrong = sum(1 for event in mine if event["verdict"] == "spec错")
    model_wrong = sum(1 for event in mine if event["verdict"] == "model错")
    clause_gap = sum(1 for event in mine if event["verdict"] == "条款欠一句")
    counted = spec_wrong + model_wrong
    judged_count = counted + (clause_gap if "same-origin-clause-gap-counted" in broken else 0)
    state = ("还没有事件进表，累计 0 次只说明没观测到，不说明两边独立" if not mine
             else f"未满 {THRESHOLD} 次，不判同源" if judged_count < THRESHOLD
             else f"已满 {THRESHOLD} 次而 spec 错 0 次，实际同源" if spec_wrong == 0
             else f"已满 {THRESHOLD} 次，spec 错 {spec_wrong} 次，不是同源的样子")
    lines.append(f"  · {name}（{pair['spec']} × {pair['model']}）：已立，累计不一致 {counted} 次（spec 错 {spec_wrong}、model 错 {model_wrong}），"
                 f"条款欠一句 {clause_gap} 次先记账、不计入；{state}")
    if judged_count >= THRESHOLD and spec_wrong == 0 and "same-origin-zero-spec-wrong-unchecked" not in broken:
        problems.append(f"对「{name}」（{pair['spec']} × {pair['model']}）：累计 {judged_count} 次不一致而判「spec 错」0 次（model 错 {model_wrong} 次）：两条腿实际同源")
for line in lines:
    print(line)  # gate-lint:detail
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 同源性计数器：{len(problems)} 处不对（已立 {len(built)} 对、未立 {len(unbuilt)} 对、事件 {len(events)} 行）")  # gate-lint:summary
    print("     → 怎么办：累计 ≥ 5 次而 spec 错 0 次的，是两条腿实际同源：D13 已定项 13 定的处置是把它们合并、预算移给穷举，这是设计决定，交主 agent 问用户，不靠改表销账；"
          "表里确有一次判错了（该是 spec 错）就改那一行的判谁错、出处写重判的记录；表写坏了照上面点名的行改，格式在 .claude/gate.d/divergence-events.tsv 文件头")
    sys.exit(1)
if not built:
    print(f"  ! {TABLE} 里一对已立的都没有（未立 {len(unbuilt)} 对），这一格无对象可判")
    sys.exit(77)
print(f"  ✓ 同源性计数器：已立 {len(built)} 对、未立 {len(unbuilt)} 对（未立的不报绿、不计入）；事件 {len(events)} 行，已立的每一对都没有累计 ≥ {THRESHOLD} 次而 spec 错 0 次")
PY
}

# 格不复用（文件头「复用」一段）：每一格照跑，格里每一条按自己的单元键查判定库。

# 本次未跑的格先收进自己的文件，再原样转给 $GATE_NOT_RUN_FILE
not_run_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
# 一道里只排一次：Verus 那几格第一次拿到内存包装的 252 就往这份文件写一行（verus-proof-check.py 的 QueueRefusal），之后的格读到就不再排队
queue_refused_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${not_run_record:?}"; exit 1; }
# 按条的现判、复用数：判法每格往这份文件追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」（FORMAL_ITEM_TALLY_FILE），跑完整道相加
item_tally_record="$(mktemp)" || { echo "  ✗ 建不了临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; rm -f "${not_run_record:?}" "${queue_refused_record:?}"; exit 1; }
for ((verdict_round = 1; verdict_round <= verdict_rounds; verdict_round++)); do
  ((verdict_rounds > 1)) && echo "── 第 $verdict_round 趟（样本 .formal-sample/run-twice：判定库 $verdict_library，第二趟应当全复用）"
  : > "$item_tally_record"
  if ( export GATE_NOT_RUN_FILE="$not_run_record" VERUS_PROOF_CHECK_QUEUE_REFUSED_FILE="$queue_refused_record" FORMAL_ITEM_TALLY_FILE="$item_tally_record"; stage_cells_run "$ROOT" ); then stage_exit=0; else stage_exit=$?; fi
done
if [[ -s "$item_tally_record" ]]; then
  judged_total=0; reused_total=0; tally_cells=0
  while IFS=$'\t' read -r tally_cell tally_judged tally_reused tally_reasons; do
    tally_cells=$((tally_cells + 1))
    judged_total=$((judged_total + tally_judged)); reused_total=$((reused_total + tally_reused))
    echo "  $tally_cell：现判 $tally_judged、复用 $tally_reused${tally_reasons:+（现判的原因：$tally_reasons）}"
  done < "$item_tally_record"
  echo "── 按条（$tally_cells 格有逐项判定；verus-toolchain-pinned 与 pinned-red-examples-table 每趟现判、不复用）：现判 $judged_total、复用 $reused_total"
fi
rm -f "${item_tally_record:?}"
((verdict_library_is_temporary)) && rm -rf "${verdict_library:?}"
if [[ -n "${GATE_NOT_RUN_FILE:-}" && -s "$not_run_record" ]]; then cat "$not_run_record" >> "$GATE_NOT_RUN_FILE"; fi
if [[ -s "$queue_refused_record" ]]; then
  IFS=$'\t' read -r refused_where refused_howto < "$queue_refused_record"
  echo "  ✗ 这一道没跑成：内存包装排不上（$refused_where），之后的格没再排队，整道只等了一次"
  echo "     → 怎么办：${refused_howto:-照 scripts/hosts/run-with-memory-cap.sh 文件头「退出码」一段办}；空出来之后整道重跑，不改证明、核、必红世界表或实现"
fi
rm -f "${queue_refused_record:?}"
rm -f "${not_run_record:?}"
exit "$stage_exit"
