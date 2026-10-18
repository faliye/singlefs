# 复用键全工程普查：第一轮对账底稿（主 agent 并的）

口径（用户 2026-10-15 定）：剪枝的单元是函数。一条判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本。整文件原文、文件级 / 包级 / 整表级指纹、按提交或 HEAD 算的改动范围、手写版本常量、路径与名字进键、没有库只有整道标记、库放会丢的位置，一律删掉重做；做不到单元级的那一道重做。不做兼容迁移、不为旧库补签，改完重跑。

五路报告（只读）：crates `survey-pruning-r1-crates/report.md`（C1–C40，sha256 bb34d6f9…）、gates `…-gates/report.md`（G01–G68，8d9a961e…）、scripts `…-scripts/report.md`（B1–B15、C1–C10、M1、K1–K7、L1、E1–E4、R1–R5、X1–X6，ba200fe8…）、docs `…-docs/report.md`（D1–D97，119c2406…）、disk（待交回）。下表「路子编号」列各路自己的号；「核」列是主 agent 现查过的。

## 一、按机制归并（每行一个要重做的机制）

| 号 | 机制 | 落点（文件:行号） | 现状的键 | 违反 | 路子编号 | 核 | 重做方向（定案） |
|---|---|---|---|---|---|---|---|
| L1 | 崩溃判定库的判器闭包按文件收、整份词法进摘要 | `judging_closure.rs:877-990`（闭包）、`:856-875`、`:1073-1085` | 闭包 53 份 .rs 整份词法单元串各一摘要；core 32 份含写路径；`#[cfg(test)]` 照收 | ③ | C1、C5；三路审计 2、6 | 现查 cfg(test) 不剔；crates 路量：core 加空函数 137 行换 134、测试模块加空单测换 121 | 闭包按调用图从判器入口 fn 收函数单元（fn、方法、const、static、类型定义、宏），每单元去注释取指纹；键 = 单元指纹多重集 + 工具链 + 依赖版本。E164 F4 V5/V11 当阳性对照（函数单元必须连引用的常量、类型一起算） |
| L2 | wgsl 与 `include_str!` 文件按原文字节进摘要 | `judging_closure.rs:747-759`、`:903-915`；`crash_verify_gpu.rs:361-367`；`crash_judge_gpu.rs:531-548` | 五份内核原文 sha256；核对着色器原文 | ② | C2、C21、C25；三路审计 4 | crates 路量：wgsl 加注释 GPU 把换 | wgsl 按 `fn` 切单元、去 `//` 注释；按 `@compute` 入口调用图收；Rust 侧生成的常量串当它的生成函数单元 |
| L3 | 摘要每行带路径、`entry/stop` 行带路径；块键节点名带设点文件路径、崩溃点名、项名、表名、种子串；观察者种类带名字 | `judging_closure.rs:1067-1084`；`crash_identity.rs:71-78`、`:572-578`；`crash_amplification.rs:399-406`、`:613-615`、`:822-824`、`:12081-12145`；`verdict_store.rs:152-190`、`:629-643`；`crash_state_enumeration.rs:745-754`；e170/e171 装置 `sampling_item_name_of_this_call` | 路径与名字字面进键 | ⑤ | C3、C10–C15、C19、C27、C29、C34–C36；三路审计 8 | crates 路量：format 目录改名三把全换 | 键只放内容：单元指纹多重集、写表与起步镜像摘要、段展开；路径、名字只进索引表与报告 |
| L4 | `Cargo.lock` 与各 `Cargo.toml` 整份原文进版本 | `judging_closure.rs:993-1005`、`:1086-1092` | 整份原文 sha256 | ②③ | C4；三路审计 5 | crates 路量：lock 加注释 137 行全换 | 从 `Cargo.lock` 解析闭包单元真引到的外部包（名、版本、来源、checksum）及传递依赖；`Cargo.toml` 只取生效 features、edition、profile，不取原文 |
| L5 | CPU 结论与观察者结论版本含整份设点文件 / 观察者文件闭包 | `judging_closure.rs:1137-1145`；`crash_state_enumeration.rs:1000`、`:1009`；`crash_amplification.rs:12962-12976`、`:14085-14097`、`:14382-14394` | 整份测试文件出发的闭包 | ③⑤ | C6、C18、C19；三路审计 7 | — | 入口改成这一项真执行的那个测试 fn 单元 |
| L6 | 块键按整点、整条流切；给定状态表一张表一块 | `crash_amplification.rs:353-366`、`:643-733`；`crash_state_enumeration.rs:745-754`；`:12081-12145` | 计划哈希罩整点；崩溃枚举用例整条流一点；给定状态表恒一块 | ③ | C13、C14、C15；路子二 7 | — | 块按段切，键 = 段写表与起步镜像摘要 + 段展开 + 判器另读输入 + 段内序号；崩溃枚举用例按步切成前缀树节点与放量同一套 |
| L7 | 手写版本号当判法版本（结论版本写法号、给定状态写法号、观察者拼块号、GPU 派活号、判表号、事实抽取号、核对器前缀） | `crash_identity.rs:604`；`judging_closure.rs:783`、`:1176`；`crash_amplification.rs:12095`、`:12136`、`:14060`、`:14387`、`:3934-3941`；`crash_judge_gpu.rs:502-504`；`crash_judge_tables.rs:54`；`crash_facts.rs:35`；`crash_verify_gpu.rs:361` | 手写常量；停止文件里的代码改了版本不动 | ④⑦ | C9、C17、C20–C25 | 现查 `GPU_JUDGE_REVISION = 1`、`OBSERVER_CONCLUSION_COMPOSITION_VERSION = 2` | 全部删手写号；写法版本由算键那几个函数单元的指纹代替；派活里决定结论归位的函数、拼块函数、抽取函数并进相应结论的单元集合；死键（C21、C22）删 |
| L8 | 核对复用缺一格；核对键不含跨 crate 调到的函数 | `crash_amplification.rs:4043-4050`；`:3934-3941` | 只看（块键，核对代码摘要）在不在；`check_unit`、`crc32` 表不进键 | ⑦ | C24、C26 | 现查 C26 行 | 核对键 = `verify_state_from_facts` 调用图单元指纹 + 存着的事实表摘要 |
| L9 | 编译时整张摘要表与盘上现算整表比，任何一行不等整趟不写库 | `crash_identity.rs:771-783`；`crash_amplification.rs:12951-12958`、`:11861-11875`；`judging_closure.rs:1181-1262`（候选按目录列） | 整表级 | ③⑥ | C7、C8 | — | 只比这一趟要用的那几把；表的行按登记的项与它的入口 fn 单元列 |
| L10 | 另一套「判法版本」键（流水线单测用）；流水线单测起跑清库；对拍结论不入库 | `verdict_store.rs:16-24`；`crash_amplification.rs:1134-1188`、`:1644-1647`；`crash_library.rs:186-197`、`:866-938` | 整份词法串连路径；清空；不存 | ②③⑤⑥⑧ | C28、C32、C33 | — | 并进 L1 的单元键；空库起跑的单测只在一次性副本建库；对拍结论按（输入摘要、两边判器单元指纹）入库 |
| L11 | 录入段每趟无条件跑；没有分项级键 | `crash_library.rs:767-786`；`crash_amplification.rs:10988-11080`；`crash-amplification-item.sh:762-765` | 无键 | ⑥ | C30、K1；路子二 9 | 现查 `:764` 无条件 | 录入输出按（录入函数单元指纹 + 流输入摘要）入库，命中就跳过 |
| L12 | 判定库分六类六份、目录名写死带类名与存法名；同一批状态两库各判；旧库留着没人读；分片进度与清单按名字键 | `crash_library.rs:110-161`、`:229-245`、`:703-722`；`verdict_store.rs:240-264`、`:917-920`；`crash-amplification-lib.sh:106-120`；`crash-point-replay.sh:7`、`:27-30`、`:210`；`crash-point-replay-shard-run.sh:85` | 类 → 目录名；节点名不同 | ⑧⑤ | C29、C31、K2、K3；三路审计 10 | — | 一份库、键只由内容定，类当索引；库目录固定一处不随趟换、不带存法名；清单与进度按内容摘要寻址 |
| L13 | GPU 与 CPU 结论互不借；驱动与卡型不进版本；`judge_inputs` 链住前面全部写；`include_str!(concat!…)` 不进闭包 | `verdict_store.rs:629-633`、`:562-570`；`crash_amplification.rs:965-1126`、`:11861`；`judging_closure.rs:498-505` | — | 判不了 | C38、crates 路第 4 节；路子二 9 | — | 定：两种判器单元不同、键自然不同，不互借；驱动与卡型留块属性不进键（判定不依赖它，对拍守着）；`judge_inputs` 是判器真读的数据，进键但按段切（随 L6）；`concat!` 形态由闭包算法认出来、认不出就拒算（不许静默漏） |
| L14 | 门禁整道键：整张 `stage-inputs.tsv`、整份阶段脚本、登记行原文、手写档位、按目录前缀 `git ls-files` 列文件、文件级摘要连路径 | `admission.py:2797-2817`、`:417-423`、`:598-617`、`:840-865`、`:834-837`、`:338`；`lib_source_digest.py:152-162`、`:532-610`；`stage-inputs.tsv:30-34` | 整道级 | ②③④⑤ | G01–G05、G16、G65；三路审计 14 | 现查 `:1592`、`:2808` | 整道标记降为索引：道与格下每条判定各自单元键命中才报复用；登记表只答「从哪些入口起算」，整张表不进任何键；`lib_source_digest` 切到函数单元（Rust fn 项与方法、Python def、shell 函数）；删手写档位 |
| L15 | 门禁第二条复用路：与 `refs/sop/staged-green` 树比；`change-touches-crates.sh` 对 HEAD 取 diff；环境按树记录 | `admission.py:1547-1629`、`:1523-1536`、`:1490-1506`、`:1285-1351`；`stage-must-run.sh:56-61`；`gate-staged.sh:28-48`；`change-touches-crates.sh:57`、`:89-120` | 按树、按 HEAD | ①③⑤⑦ | G11–G14、G26–G28；docs D5、D11–D19 | — | 整条树对比路删（连 `SINGLEFS_STAGED_TREE` 支）；四道门禁里 `change-touches-crates` 那一问删；`refs/sop/staged-green` 不再当复用基准（gate-staged 只管跑整轮） |
| L16 | 构建环境与工具链取法：配置文件原文、runner 脚本整份原文；树对比路不比工具链 | `admission.py:669-717`、`:744-803`、`:643-658`、`:1608-1609` | 原文 | ②③⑦ | G06、G07、G12 | — | 工具链保留（`rustc -Vv`）；构建环境取 cargo 解析后的生效值（RUSTFLAGS、profile、target），runner / wrapper 取版本或函数单元指纹；每条判定键都带工具链 |
| L17 | 崩溃枚举用例全绿标记：整个 `crates/` 减排除、双机驱动脚本整份、判法摘要取函数原文连注释、文件名带用例名 | `admission.py:2158-2204`、`:2716-2793`、`:2213-2218`、`:326-327` | 包级 + 原文 | ②③④⑤⑥⑧ | G18–G21、G40、K6、K7；三路审计 12；docs D2、D3、D9 | 现查判法摘要文档串「每个定义的原文」 | 用例级标记整个删：复用全部落到 crates 侧块级判定库（L1–L12）；判法摘要保留闭包法但单元内容去注释（tokenize 或不带位置的 ast dump）；双机驱动只取真走到的 shell 函数单元 |
| L18 | 快档两趟、模型对拍六格、research-unit-tests、research-script-selftests 没有库，每趟全跑 | `checker-tier-crash-point-replay.sh:351-356`、`:433-441`、feature-gated 格；`harness-model-differential-and-scenarios.sh:149-161`；`checker-tier-research-build-and-replay.sh:187`；`code-tooling.sh:311-428`；`crash-point-replay.sh:63-64`、`:212-216`、`:517-518` | 无 | ⑥ | G37–G39、G52、G54、G59、K4；路子一 10 | — | 每条测试函数 / 每份自证一条判定入库，键 = 那条测试调用闭包单元 + 特性组合 + 工具链 + 依赖版本；只跑没命中的（`--exact` 名单，经 `lib_libtest_result_lines.py` 认结局） |
| L19 | QEMU 六档一个标记；键含整个 `scripts/`、`research/results/`；QEMU 版本、内核镜像不进键；`--item` 不写标记 | `checker-tier-qemu-device-streams.sh:218-233`、`:450-457`；`stage-inputs.tsv:37` | 整道 | ①③⑤⑥⑦ | G41；三路审计 23 | — | 每档一条判定，键 = 宿主与来宾两侧真走到的函数单元 + QEMU 版本 + 内核镜像与 initramfs 内容哈希 + 工具链 |
| L20 | LKMM 八份 litmus 一个标记；整个 `crates/` 进键；`tools/memory-model` 不进 | `checker-tier-lkmm.sh:56-61`、`:78-87`、`:22`；`lkmm.sh:332-346`；`stage-inputs.tsv:38` | 整道 | ①③⑤⑥⑦ | G42；三路审计 24 | — | 每份 litmus 一条判定，键 = litmus 内容（去注释）+ 锚点指到的函数单元指纹 + herd7 版本 + `.cat/.bell` 内容；锚点文法要能细到 fn（第二轮核） |
| L21 | crates 变异：判法版本按整份原文（阶段脚本、本模块、prune、libtest 解析、存储库、内存包装）；只留 3 个版本目录；行键带变异名、路径、测试名、上限、限时；基线按整树快照；覆盖键带用例名与整树快照号；函数外改动按整文件；第二台另一份库 | `crates-mutation-rows.py:601-619`、`:284`、`:711-726`、`:640-642`、`:729-753`、`:2229-2236`、`:649-657`、`:942-1071`；`mutation-shard-run.sh:31-35`、`:383-416` | 原文 + 名字 + 整树 | ②③⑤⑧ | 脚本路 C1–C9、G43、G44；三路审计 15、16 | 现查 `:609-615` read_bytes | 判法版本 = 判「抓没抓到」的函数单元指纹；行键 = 被改坏的函数单元指纹 + 原文 + 替换文 + 执行过的单元集合指纹 + 工具链 + 依赖版本，不带名字、路径、上限、限时；版本目录不按个数删；基线与覆盖按执行过的单元存；一份库，第二台导入即弃 |
| L22 | 广谱变异：版本树由 `git archive <提交>` 导出、整树逐文件原文连路径当版本指纹；不带参数不读结论表；`--changed-since` 一律现判改到的；结论键带路径与限定名、指纹拼整文件函数外部分；切不开退整文件；宏 / include 依赖整文件或整树；抓到沿用只比测试函数自己、认不出退整包；全局键收 Cargo.toml / lock / .cargo / 排除表原文；判法版本手写常量；无效、没红、升档结论不入可沿用库；覆盖与基线按版本目录；`notice_if_head_moved` | `broad-mutation.py:1421-1451`、`:988-1015`、`:1056-1075`、`:2521-2596`、`:3114-3142`、`:3297-3303`、`:2869-2886`、`:469`、`:2412-2431`、`:1992-2010`、`:2254`、`:3406-3415`；`mutation-prune.py:509-532`、`:594-623`、`:604-608`、`:640-658`、`:661-681`、`:700-726` | 按提交、整树、整文件、名字、手写 | ①②③④⑤⑥⑦ | B1–B15、G48、G49；三路审计 19–22 | 现查 `:1442`、`:1014`、`:3302`、`plan_changed_since`、`carry_decision` | 「一版」不再是容器：清单与结论都按单元键存；选条 = 每条过沿用判据，沿用不了才现判，不带参数就是剪枝、`--full` 只写不读；删 `--changed-since`、`--sync-tree` 按提交那一层、`changed-since-tree`、`notice_if_head_moved`；结论键 = 被变异的 fn 单元指纹 + 原文 + 替换文 + 抓它的测试执行过的单元集合指纹 + 依赖版本；函数外的项（const、static、struct、impl 头、宏）各切单元，切不开当解析缺陷修；无效按容器单元沿用，没红按「覆盖到它的测试集合与各自执行过的单元不变」沿用，升档结论进库；覆盖与基线按执行过的单元存；全局键只留工具链与 cargo-mutants / nextest / llvm 版本；判法版本 = 判结局那几个函数单元指纹 |
| L23 | 实验变异表 `mutate.sh` 没有判定库 | `mutate.sh` 文件头 `admission: always` | 无 | ⑥ | M1；路子三 U13 | — | 按条入库：键 = 原文 + 替换文 + 包住原文的 fn 单元与其依赖单元指纹 + cargo 参数与 features + 工具链 + 依赖版本；只编译那一格同一把键 |
| L24 | 形式化：判法版本两份脚本整份原文；按条键收核 crate 整目录原文、`verus-pins.tsv` 原文、限时；proof 整 crate 一项；空真项名带路径:行号；必红世界行带名字与路径；钉表 360 行共用装置依赖闭包原文；钉表行带账路径与变异名；E165 第二段续跑只认变体名 | `verus-proof-check.py:788-813`、`:853-858`、`:1026`、`:1068`、`:1201-1202`；`e165-pin-red-examples.py:1162-1187`、`:1204-1214`；`e165-replay-mutation-kill-bounds.sh:1815-1829`；`lib_verdict_library.py:52-86` | 原文 + 名字 + 整 crate | ②③⑤⑦ | G31–G35、G47、E1–E4、L1（脚本路）；三路审计 18 | 现查 `:793` | 每项键 = 这一项涉及的 Verus 函数单元（被验 fn、它 requires/ensures 用到的 spec fn、proof fn）去注释指纹 + Verus 与 vstd 版本；proof 按函数拆项（能否做到第二轮核）；空真项身份 = 所属函数单元指纹 + 函数内第几处；必红行去名字路径；钉表每行键 = 这一行档与参数真走到的装置函数单元闭包 + 状态输入 + 期望逐态行 + 变异原文替换文；限时移出键；共用库加「函数单元摘要」取法 |
| L25 | 研究复跑：两格不写标记、外层只认 staged-green 树；按行键收装置整份文件 / 整子目录 / 整包、复跑表行原文（含实验号与文件名）、驱动函数原文留行内注释；GPU 开关与特性不进键；全复用也先试编清场；准入按登记路径指纹；按行库从未落盘；mutation-tables-compile 库在 /tmp、整表一项、判法版本整份阶段脚本 | `admission.py:3124-3256`、`:67-80`；`replay.sh:33-38`、`:55`、`:2401-2480`、`:2332-2395`、`:2438-2439`；`checker-tier-research-build-and-replay.sh:327-345`、`:385-432`、`:455-460`、`:504`、`:521`；`gate.sh:176` | 原文 + 整目录 + 名字 | ②③⑤⑥⑦⑧ | G23–G25、G50、G51、R1–R4；三路审计 25–27 | — | 行键 = 装置 `main` 起的函数单元闭包 + 这一行参数 + GPU 开关与特性集 + 期望产物内容 + 工具链 + 依赖版本；实验号、文件名不进；先算全部行的键，全复用就不编不清场；准入那一路删并进行键；库挪到 git common-dir；只编译格按表内每条变异的单元键 |
| L26 | 实验准入（`admission.py experiment`）：登记路径逐文件原文 + 登记行 + kb 决策文档；`manifest` 子命令无调用方 | `admission.py:868-876`、`:1017-1038`、`:1106-1189`、`:2993-3015`；`stage-inputs.tsv:50` | 原文 + 路径 | ②③⑤⑥ | G08、G09、G61；docs D40 | — | 定：在射程里（它挡的是重型实验的重跑）。键改成装置入口函数单元闭包 + 跑前登记里影响结论的参数；kb 决策文档不进键（不是代码）；`manifest` 子命令确无调用方就删（第二轮核） |
| L27 | E171 冻结的旧修订装置（修订 6、8、10、11）的复用库与续跑：键带轮名、文件级闭包带行号、schema 手写串、只按行号与变体名收旧行 | `reuse_library.py:88-89`、`:172-243`、`:499-546`；`segment_one_units_revision_8.py:211-300`；`rows_orchestrator_segment_two_revision_10.py:329-350`；`row_driver.py:494-507` | 名字 + 原文 | ②③⑤⑦⑧ | X1–X6 | — | 定：修订 6、8、10 是冻结证据，不改（`evidence-discipline.md`），它们的结论不被门禁复用；修订 11 的 `reuse_library.py` 是在用的装置库，随 L25 改成单元键、库名去轮名；`resume_check.py` 不在仓里的那一支按「没入库」记 |
| L28 | 文件头、文档与代码不一致 | `crash_identity.rs:6-8`；`crash_amplification.rs:272`；`verdict_store.rs:88`；`crash_judge_gpu.rs:21`；`crash_judge_tables.rs:11`；e170/e171 文件头；crates 路第 3 节 10 处 | — | — | 三路审计 28；crates 路第 3 节 | — | 随对应机制一起改成现状 |
| L29 | 规则、技能、定义、kb、欠账、钩子注释里写死的旧口径（按树 / 按 HEAD / 整文件 / 整道标记 / 判法版本） | docs 路 D1–D97（`verification.md:13,34-38,47,52,111,119,121`；`implementation-workflow.md:29-41,52,53,74,76`；`commit-stages.md` 第 2b 步与 1、2、4、6 步；`staged-gate.md:16,22`；`SKILL.md:21,38`；`main-agent.md:10,94`；`13-验证路线.md:253-254`；`test-script-discipline.md:62`、`command-safety.md:153`（上游）；C8、C489、C528、C564、C635、C645、C800 等；E164 F4 / W5、E142、E171；`stage-inputs.tsv:5-6,11-17,25-34`） | — | 文字 | D1–D97 | 现查 E164 F4、`lib_heavy_tests.py:577` | 算法改完同一批改文字；D13 新立「复用键与剪枝单元」分项、改已定项 9；上游两句交上游会话；钩子只改注释与自证用例（判法不读 `--changed-since`） |

## 一之一、第二轮 gaps 路补的（`survey-pruning-r2-gaps/report.md`，sha256 adc21e89…；6714 份命中文件逐份有去向 `hits-disposition.tsv`）

| 号 | 机制 | 落点 | 现状的键 | 违反 | 核 | 定 |
|---|---|---|---|---|---|---|
| L30 | E178 第四段项目录续跑 | `scripts/experiments/e178_leaf_and_record_death_counts.rs:6191`、`:6202`、`:6208`、`:6213`、`:6243`、`:6848`；登记 `research/prompts/e178-preregistration.md:888`；库在 `~/.local/share/singlefs/work/e178-segment-four/items` | 项名（带序号）+ 装置整份源码 sha256（含注释）+ 几何标签；工具链不进 | ②③⑤⑦⑧ | gaps 路量：加一行注释键变 | 改：键 = 装置入口函数单元闭包 + 几何参数 + 工具链 + 依赖版本，库挪到 `~/.local/share/singlefs/` 固定位置；走跑前登记修订（experiment-designer 写修订、runner 改装置），并进改动批 |
| L31 | E174 父进程按产物续跑 | `e174_lease_drop_wait_memory_return.rs:312`、`:1495`、`:2792`、`:2987` | 只有格标签与轮号 | ⑤⑥⑦ | — | 改：同 L30 的键形态 |
| — | `scripts/gate/rerun-failed-tests.py` 只重跑上一份日志里红的用例 | `verification.md:25`、`implementation-writer.md:34` | 按测试名沿用绿的 | 不算 | — | 不改：它是「红了只重跑红的」的辅助，不是判定库；完整一遍归提交时整轮门禁，由按测试函数入库（L18）罩住 |

另：`admission.py manifest` 子命令在当前树无调用方（量的），删，它两格自证改走 `crash-case-manifest`；钉旧键形态的自证、样本、钩子用例共 78 行 375 处（`survey-pruning-r2-gaps/q7-detail.tsv`），改算法时同一批换；三份 checker 档测试把 `judge_version=` 打进汇总行、原样记进全绿标记，同批改；L3 改块键时要一起改的调用方全集 23 份、L23 实验变异表 54 份，在报告表二。

## 一之三、第二轮 verify 路的核与改写（`survey-pruning-r2-verify/report.md`，sha256 b5ab76cd…；29 条里 27 条在副本上造改动量过或现场读码，没有整条判错；原样输出在它的第 4 节与 `out/`）

照抄进派发提示的改写：
- L14 现状的键补：`.rs` 按整文件去注释词法；有算出来的 include（`include_str!(concat!…)`）的 crate 整个取原文——此刻 `singlefs-harness` 273 份全部原文（`heavy_threads.rs:22`，量的：`memory_pool.rs` 加一行注释 18 把门禁键换 14 把，换成字面路径后 0 把换），`include_str!` 字面读到的 `.rs`（`crash_facts.rs`、`crash_amplification.rs`、`gpu_unit_checks.rs`、`model.rs`）取原文；落点加 `scripts/gate/lib_source_digest.py:504-517`。
- L17 引 K7 的半句改：四份双机驱动脚本按 shell 摘要进 4 条分片用例的键，只改注释不作废，任一份里有函数级改动（哪怕没人调）就作废全部 4 条。
- L22 重做方向改成「结论键 = 被变异 fn 单元与它调到的单元各自的指纹 + 原文 + 替换文 + 抓它的测试执行过的单元集合指纹 + 依赖版本」；量的漏判实例：`index_node_entry_capacity` 调的 `index_node_header_bytes` 改了，`carry_decision` 仍交 carried；这一格当阳性对照。
- L26 现状的键补：逐文件原文 sha256（`input_manifest` 不带 `source_digests`），core 加一行注释 E142 两把键都换。
- L6 现状的键补：给定状态输入串带 `seeds_and_scale=`，`seed_count` 让同一批每张表的键都随种子个数变。
- L12 盘上的数补：`crash-amplification/runs/` 436 个目录 89G。
- L7 落点行号 `crash_verify_gpu.rs:361` 改 `:364`。

量到坐实的：整张 `stage-inputs.tsv` 加一行注释 8 把整道键全换、格键与用例键不换；树对比路换假 rustc 三道仍判可跳过；QEMU 版本串与 `.cat` 改了键不变；复跑 247 行键不含 GPU 开关、`replay_one` 加行尾注释 247 行全换；广谱变异不带参数 `Pruner.carry` 被调 0 次；行键里限时、上限、变异名改哪样都换；三把判法版本加注释都换；核 crate 文件头插一行注释 33 个空真项名全换。另：主工作区此刻 crates 变异判法版本 `c6d72979…` 不在盘上三个目录里，下一趟复用 0。

## 一之二、盘上量到的（disk 路 `survey-pruning-r1-disk/report.md`，sha256 0a976fe7…；库目录 L1–L30、复用掉 0 归因 R1–R20、阻碍清单 X1–X38；用只读解析器直接读 SST 与 WAL，条数与 RocksDB LOG 的 num_entries 逐份对上）

| 机制 | 盘上的数（量的） |
|---|---|
| L3 名字进块键 | 旧库 `library`：块内容五样全等、只差设点文件路径的 4100 块、268 435 542 态各存各判；`layer0-enumeration-library` 与 `library-by-conclusion-kind` 之间只差点名写法（`position_addressed::X::full` 对 `position_addressed_X`）的 9679 块、632 578 246 态；给定状态库节点名带 `seed=…` |
| L6 / L12 切块单元不对齐 | 崩溃点重放整条流一节点：`fixed_script::full` 1 个输入指纹、1 个计划哈希管 25 611 块；放量按点切；两边块键、输入指纹、计划哈希交集都是 0——两库合一之前要先对齐切块单元 |
| L1 / L2 判法版本一换整库重判 | `library-by-conclusion-kind` 累计多判 3 935 242 934 态；7fa0adbf→f0b9512b 重判 30 172 块，那次闭包里改动的 29 份文件有 24 份去注释后逐字相同，3 份 wgsl 只改注释 |
| L21 crates 变异判法版本 | 10-12 到 10-14 换了至少 7 把，每把「复用抓到 0 条」，日志删版本目录 7 次；限时 1800→3600 的 163 行行键全变，行号变而限时不变的 123 行行键不变（反例 0 对） |
| L24 形式化 | proof 10-14 重判一次，components 五样逐字相同、只有判法版本变（改的是线程缺省值）；空真项带相对路径与行号 |
| L22 广谱变异 | 两版 `global-key.txt` 只差手写 `rules` 一行、`manifest.tsv` 逐行相同，304 行结果一行没进跨版本结论表 |
| L12 / L25 丢库 | 孤立：`layer0-enumeration-library` 620M、旧 `library` 271M、`polluted` 库 104M、`.git/singlefs-layer0-progress` 100M；研究侧 /tmp 的库已不在；5 条旧用例名标记成孤儿，14 条登记用例里 7 条没有标记 |
| 不是键的问题 | R5 以前没判过、R9 分片 panic 没判出东西、R12 清场回读没过退 78 |
| 归因缺口（X10） | 新布局崩溃库一条 `09` 清单都没写；旧库 `09` 只记文件名不记摘要；crates 变异与形式化的版本目录不存版本输入；整道标记只存文件个数不存清单——重做时每把键要把组成清单落盘，作废时能报出是哪个单元变了 |
| 射程外撞见 | 轻型 `--quick` 与重型共用一份库；放量快档每趟写一把新 GPU 版本；crates 变异 `--quick` 占掉「只留 3 个目录」名额（3 个里 2 个是轻型建的） |

## 二、主 agent 定的（不再问用户）

1. 全量开关（`--full`、`SINGLEFS_GATE_FULL`、`--start-over`、`--milestone-full`）保留：它是「只写不读」的显式开关，不是复用键，不算 ①–⑧；但全量不许另建一份没人读的库（K5），现判的照键写回长期库。
2. GPU 与 CPU 结论不互借：两种判器的函数单元不同，键自然不同。
3. 驱动版本、卡型留块属性，不进键（判定不依赖它，GPU 与 CPU 对拍守着）。
4. 构建环境（RUSTFLAGS、profile、target、runner）算工具链的一部分，按 cargo 解析后的生效值进键，不取配置文件原文。
5. 实验准入（L26）在改的射程里；preflight 的 `inputs-changed`（轻脚本）与 doc / code 类静态门禁的「只判新加行」改动范围（`changed-paths.sh`）不在：后者是判据语义，不是沿用旧判定。
6. 「没红」「无效」两种结论进可沿用库，键按 L22 写。
7. E171 冻结的旧修订不改；修订 11 在用的库随 L25 改。
8. 门禁侧与 crates 侧的函数单元切分与调用图共用一份算法（Rust 一份、Python 一份、shell 一份、wgsl 一份），避免两套口径再分叉；放哪、谁调谁第二轮定。

9. words 路第八节 11 项的定：① 轻型不另立「没有剪枝」：轻型与重型同一份库、同一套单元键复用，`verification-build.md:288` 改成现状句。② `checker-tier-crash-point-replay` 的 `--full` = 全量档（全部登记用例）、按块复用；`--start-over` = 只写不读、写回同一份库；别的入口的 `--full` / `SINGLEFS_GATE_FULL` = 只写不读（底稿第二节第 1 条照此改写）。③ commit-stages 第 1 步「工作区等于暂存区」改成核整个工作区与暂存区一致（跟踪文件无差），不按路径集合。④ `refs/sop/staged-green` 保留作「上次整轮全绿的树」记录，不再被任何复用判定读；`admission.py` 的树对比整条删。⑤ 第 6 步「几道绿是不是同一份源码」删 `git rev-parse HEAD`，改比各道判定键集合（单元指纹）相同。⑥ 「判法版本」改叫「判法单元指纹」；`crates-mutation-rows.py` 的 `judge-version` 子命令改叫 `judging-units`。⑦ 三条新要求都做：各道 `--list-items` 报每项键对不上的判定条数；判定库每块记判定日期与跑法；作废时把变了的单元清单进红句（补 disk 路的归因缺口 X10）。⑧ 广谱变异 `--import-results` 留，结果并进本机那一份库，第二台不留长期库。⑨ `stage-inputs.tsv` 第二列改写入口（测试函数、bin、litmus、虚机档），`closure=` 删（闭包由算法从入口算），`@<别的键>` 留。⑩ 五路加五路的十份报告拷进 `research/prompts/pruning-survey-2026-10-15/` 作冻结证据，D13 已定项 17 的依据引它与盘上的数。⑪ 改 `main-agent.md`、`agent-common.md`、`experiment-designer.md`、`mutation-triage.md` 并进改动批之后的三轮攻方（用户要求改完强攻三轮），不单开。

10. Verus 三问（主 agent 在副本 `~/.local/share/singlefs/work/verus-probe-2026-10-15/` 跑，`probe.log`、`*.json`、`*.err` 原样留着；量的）：① 整 crate 对照 `verified 56、errors 0、is-verifying-entire-crate true`；② `--verify-root --verify-function <名>` 能逐函数验，但按子串匹配：90 个 `fn` 名里 7 个是 `verus!` 外的测试函数报「could not find function」，`record`、`root_choice_key` 报「more than one match found … consider using wildcard」，名字是别人子串的几条 verified=2，加起来 48 对不上 56；③ 在 `lemma_candidates_lie_below_their_end_and_above_the_chosen_root` 体里插 `assume(false);`、只验另一个函数，`--no-cheating` 照样报「assume/admit not allowed with --no-cheating」退 1，整 crate 同样。定：U7 走「按函数拆项」那一种：每个 `verus!` 里的 proof / exec fn 一项，键 = 那个 fn 单元 + 它 requires / ensures 与函数体用到的 spec fn、proof fn 单元指纹 + Verus 与 vstd 版本；项名用唯一匹配（撞名的四个由实现员改名或改成 `--verify-function` 能唯一命中的写法），自证里加「各项 verified 之和等于整 crate 的 56」一格、不等判红；作弊扫描不另立项（部分验证已拒）。

## 三、第二轮要答的

- Rust 调用图按名字收单元在 trait 动态分派、宏展开、泛型、关联常量、`include_str!(concat!…)` 下漏不漏；现有 `judging_closure.rs`、`mutation-prune.py` 的 FileIndex、`lib_source_digest.py` 三份各能到哪一步，合成一份放哪。
- Verus 能不能按函数拆项（`--verify-function` 在 `--no-cheating` 下的语义与 verified 计数）。
- QEMU 每档的来宾内核、initramfs 内容哈希怎么取；LKMM 锚点文法能不能细到 fn。
- `lib_libtest_result_lines.py` 能不能逐条认出测试名与结局（快档按测试函数入库的前提）。
- 盘上的库：每份谁建谁读、哪些没人读、同一批状态在几份库里（disk 路）。
- `admission.py manifest` 有没有调用方。
- 自证、样本、钩子用例里钉住旧键形态（全绿标记文件名、`--changed-since`、整道键）的有哪些，改算法时一起换。
- 用 E164 F4 的 V5、V11 当新键的阳性对照：列出每道门禁「必须作废 / 必须不作废」的改动样本各至少三个。
