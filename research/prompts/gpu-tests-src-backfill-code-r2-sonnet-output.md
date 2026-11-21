# gpu-tests-src-backfill-code-r2：辩方腿（复核第一轮判决，替被判打中的一方辩护）

日期：2026-11-18。腿：three-way-defense。草稿目录：`/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r2-defense`（只放名单文本，没有仓副本、没有编译目录）。
方案按 `crates/` 今天的实现来谈。没读禁读清单里的攻方腿报告；没起项目代码，没写模型，没有 `rerun.sh`。
「装置」指 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs`（13881 行，下文行号都是此刻主工作区的，`grep -nF` 现取）。
「补丁」指 `research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch`（1384 行；引它的行号是补丁文件自己的行）。

## 0 开工核对

`sha256sum -c research/prompts/gpu-tests-src-backfill-code-r2-snapshot/crates-sha256.txt research/prompts/gpu-tests-src-backfill-code-r2-snapshot/kb-sha256.txt` 原样输出：

```
crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs: OK
crates/singlefs-background-gpu-tests/tests/reference_units_entries_name_functions_that_exist.rs: OK
crates/singlefs-background-gpu-tests/src/reference_units_entries.rs: OK
crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs: OK
crates/singlefs-background-gpu-tests/src/lib.rs: OK
crates/singlefs-background-gpu-tests/build.rs: OK
crates/mutations.tsv: OK
research/prompts/e202-preregistration.md: OK
research/prompts/gpu-tests-src-backfill-code-r1-main-verification.md: OK
```

主工作区此刻就是快照那一版，下文读到的行都是合入之后的样子。

## 1 判定一览

辩方判定三种：站得住（判决够得着、替代改法也不更小）、够不着（判决写宽了或引错了）、同样打中别的臂（同一形状在别处也成立、改法没罩到）。

| 判决条目 | 辩方判定 | 一句话证据 |
|---|---|---|
| 第二节第 2 条第一处：交来的卡换了而张数没变照样续跑 | 站得住 | 改前函数只比两个字段；任务 18249 读数 `stopped=None`；续跑能落到另一台（`task-server/README.md:25`）。同样打中别的臂：CPU 臂的结果随台变，改法只认卡，不认台 |
| 第 2 条第二处：没交卡那一趟之后续不了 | 站得住 | 注释与代码互相打架（2.2 细说）；任务 18249 读数 `stopped=Some(...)`。改法已落，但新注释的一般句子仍比代码宽一格 |
| 第 2 条第三处：与跑前登记对不上 | 站得住，分量轻 | 登记第 49、297 行原文仍是「不给 `background_gpu_devices`」；一条腿判打中、另一条腿当已知处置没报；一条修订即闭，已写 |
| 第 6 条：摘要不含装置自己 | 站得住，打在括注而非定义字面 | 任务 18249 读数 `of_the_apparatus_package=0`；登记第 168 行摘要定义的字面没违反，打的是停机条款括注「`crates/` 中途被改」 |
| 第 7 条：参照一侧登记表无测试无变异 | 站得住，但判决措辞写宽了 | 「不会有任何地方红」不对：入口认不出时 `build.rs` 会停；开着的只剩「登记成另一个编得过、又没被别组登记的 fn」 |
| 第 9 条：新单测与变异不罩全 | 部分站得住 | 节号写错（八应为九）、「30 条里 3 条」把一条改过的老用例算成新的；其余的缺口多数要真卡或整条命令 |
| 第 10 条：检查单两格不成立 | 两处依据都已关到可判 | `LOCKSTEP_ARMS[下标]` 新加三处已无；多机一格关了卡的部分，换台认不出还开着 |
| 第 1 条：取卡（没打中） | 依据站得住，漏记一个边角 | 两条腿都读了取卡；两条腿都报的 stage-three-cost 一张没交退 0（`:9078`）判决没带进条文，今天还在 |
| 第 3 条：开卡表（没打中） | 站得住 | 攻方推不出的一处由正推腿补上（`background_gpu.rs:1315`、`:1371`） |
| 第 5 条：先录后判（没打中） | 站得住的一半 | 并行冲突只有正推腿看见；攻方的弱打中（`:12422` 静默返回）判决没记、补丁没改 |
| 第 8 条：判定行进 `--product`（没打中） | 站得住 | 两条腿同判；一条保留意见（登记未定项 4 没认）判决没接 |
| 第三节七条改法逐条 | 见第 3 节 | 1、2、3、4、5、6 做了；7 没有落点（找不到欠账行） |
| 检查单八格 | 见第 4 节 | 八格都判成立；其中「所有任务多机可用」「类型与名字」带弱点，交主 agent 定是否按「推不出」算（算则打中） |

## 2 第二节「打中」逐处（够不够得着、替代改法、判重）

下面「探针」都指第一轮攻方在副本上的函数级探针（造一份产物行喂核对函数），读数是判决写的、我没复跑；判决自己写了「这是副本上的读数，只用来坐实这三处」（`research/prompts/gpu-tests-src-backfill-code-r1-main-verification.md:17`）。

### 2.1 第 2 条第一处：交来的卡换了而张数没变照样续

- 够得着：改前核对只比两个字段，补丁的删除行「.map(|(_name, fields)| (fields.get("cards_named"), fields.get("named_by")))」（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:317`）；`pci_bus_addresses` 写进行里却不比，删除行「"tier={} cards_named={} named_by={} pci_bus_addresses={}",」（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:137`）。读数 `ATTACK_PROBE other_cards_same_count stopped=None`（判决第 17 行引的）。
- 可达：续跑能落到另一台。「撞上限重排不钉台」（`task-server/README.md:25`）；同一行写了第二台与本机在同一棵共享的树里跑，产物路径两台读同一份。
- 判重：不算判重，但这一处靠的是检查单「所有任务多机可用」与攻方点名的另外两条路径，不是登记的某一句。攻方自己写「不算违反了某一句登记判据」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:72`）。
- 更小的改法有没有：
  - 只在 `every_card_arm_cards` 行上比 `pci_bus_addresses`：改两处字段，关掉探针测的那一形；罩不到攻方同处报的点名一张卡臂与并发跑：「点名一张卡臂用的是交来的卡里 PCI 地址最小的那一张，这张卡没有任何续跑核对」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:55`）。
  - 拿每趟都写的 `name=gpus` 行比：不用新行种类，但把没用到的卡也算进来；交来的集合本来随空卡变，会误停同一台上的合法续跑（推的）。
  - 所以判决选的「按臂记、按臂核」是同时罩住三条路径的最小一种，没有更小而罩全的。
- 同样打中别的臂：换台时 CPU 臂读数也变：「换台时 CPU 臂的读数也跟着变」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:70`）。改法只认卡：两台同槽同型号、CPU 不同，照样混。补丁自认这一形（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:179`），登记第 12 条末句也认：「产物合并时另看产物所在的机器」（`research/prompts/e202-preregistration.md:485`）。把主机名写进那一行不行：主机名不进仓（`.claude/agent-common.md:44`），产物落在 `research/results/`；可行的是记不可逆摘要或卡自己的编号，装置拿不到编号（补丁同一处）。
- 推翻：续跑发生的两台之间 PCI 地址加型号恒不同，或服务把续跑钉在同一台，这一形就不会漏；反过来，两台有同槽同型号的卡而 CPU 不同，改法认不出（推的，没量）。

### 2.2 第 2 条第二处：没交卡那一趟之后续不了

- 够得着：同一函数里两句互相打架。改前注释「这一台一张都没交的不打不核：哪条 GPU 臂都不跑（退 77），产物里不留一行零张，以后在交了卡的那一台续跑时不被它挡住。」（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:220`），代码「if lines_already_in_the_product.is_empty() {」（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:260`）只放行一行结果都没有的产物，而每一趟在取卡之前已写开头几行。读数 `stopped=Some(...)`（判决第 17 行引的）。不判重。
- 更小的改法：删掉那句注释是零行代码，但产物仍被永久挡住，行为不变，不算关掉。攻方给的「产物里没有 `repeat_done` 与 `run` 行」不分臂，比补丁的按臂版少读一个 `arm=`；补丁的按臂版放行集合更大（见第 3 节第 2 条「做多」）。两种都在判决「照注释改代码」那一支之内。
- 同样打中别的臂（新注释比代码宽一格）：装置头注释写「一条结果行都没跑出来的那一趟」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:49`），括号里只列了三种。记卡行在跑臂之前就写：调用在 `:6552`，`let mut arms_run: Vec<LockstepArm> = Vec::new();` 在 `:6563`。所以写了卡行、没跑出结果行的一趟（被杀在头一条结果行之前；只交一张卡而只点了全部卡臂，`:2819` 判本次未跑）之后换卡续跑照样停：核对函数里「if lines_recorded_in_the_product.peek().is_none()」那一支为假，走 `find` 比字段（`:6114`）。推的，没量。
  - 为什么不能简单放行：放行之后那一趟会再写自己的卡行，产物里两种卡行并存，核对对每一行都比，第三趟起必停；所以这是「每行都核」的代价，不是漏。头注释的一般句子应收窄到那三种，或写明「没写过卡行的那一趟」。
- 推翻：被杀在头一条结果行之前的一趟，接着在另一组卡上续跑不停，这条就不成立。

### 2.3 第 2 条第三处：与跑前登记对不上

- 够得着：登记第 49、297 行原文现读仍是「全部卡」：不给 `background_gpu_devices`（`research/prompts/e202-preregistration.md:49`）；装置头注释已是 `--cards <张数>` 设（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:38`）。
- 判重度：一条腿判打中，另一条腿看见同样的事实，当已知处置没报：「当已知的处置，没当打中来报」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:275`）。两腿读到的事实相同，只差归类。不是代码错，是登记欠一条记录；登记是冻结证据，跑产物前修订合法（`.claude/rules/evidence-discipline.md:114`）。
- 更小的改法：没有，一条修订即闭；已写（登记第 12 条，`research/prompts/e202-preregistration.md:480`），第 49、297 行原文没动（本节开头那行现读还在）。
- 推翻：登记第 12 条与装置行为有一句对不上（见第 3 节第 4 条）。

### 2.4 第 6 条：搬迁之后摘要不含装置自己

- 够得着：改前起走路径只有两处，删除行「PathBuf::from("crates/singlefs-background-gpu/src"),」（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:43`）。读数 `of_the_apparatus_package=0`（判决第 17 行引的）。现在 `find` 数两个 `src` 共 61 个文件（量的：产品 GPU 包 20、核心层 41），与第一轮探针 `files=61` 一致。
- 判重度：打的是登记里两句之间的缝，不是摘要定义的字面：定义「带产品 GPU 包与核心层 `src/` 下每个文件的 sha256」（`research/prompts/e202-preregistration.md:168`）没违反，停机条款括注「`crates/` 中途被改」（`research/prompts/e202-preregistration.md:436`）才是被打中的那一句。攻方同判：「按摘要定义的字面，代码没违反」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:116`）。站得住。
- 更小的改法：只加装置一条路径：关掉「装置自己改了看不见」，但装置经 `#[path]` 编进来的两份共用模块（`:156`、`:159`）是同一个二进制的一部分，改了照样看不见。判决点名三样，补丁的五处是这三样的最小集（`:4135`）。
- 验证包要不要进（判决没点）：我的判断不该进。装置用验证包的是领卡与判定库；领卡的结果由卡行逐张核；判定库只在先录后判的真卡用例里用，不进交替跑与并发跑的量 W、C、S、P；摘要本来就不罩核心层以外的依赖包。推的；推翻：验证包里的改动能改变那几个量。
- A/B 缺口：改后 `files` 数没量，预期 64（61 加三个单文件，推的）。补丁的测试用内存小树核常量（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:1114`），没有一条测 `source_files_and_digests()`（`:4150`）把常量与读盘函数接上；把调用改成只传常量前两项的变异，补丁里的变异行罩不住（那一行换的是常量里的路径，推的）。

### 2.5 第 7 条：参照一侧登记表

- 够得着，但判决措辞写宽了：判决写「登记错一组入口不会有任何地方红」（`research/prompts/gpu-tests-src-backfill-code-r1-main-verification.md:31`），而攻方「构建脚本只在入口路径认不出时 panic」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:231`）、正推「登记错的入口会让 `build.rs` panic」（`research/prompts/gpu-tests-src-backfill-code-r1-sonnet-output.md:214`）都写了入口认不出会停。开着的只是窄的一半：登记成另一个编得过的 fn。
- 补丁之后：新测试 `every_reference_group_is_keyed_and_closes_over_each_of_its_own_entries_once`（`crates/singlefs-background-gpu-tests/tests/reference_units_entries_name_functions_that_exist.rs:80`）核组名不重、入口不重、每组 keyed、入口各恰一个 fn、键不重；两行变异（同组名、入口指到另一组的）都靠它红。
- 窄的一半今天仍开：`is_the_function_of_the_entry`（同文件 `:68`）只核限定名以入口结尾，入口本身就是闭包根，必然命中一个 fn；唯一性检查只防「与别组重复」，不防「指到没被登记的 fn」。实现员自己报了这一条（转述：`~/.local/share/singlefs/work/e202-apparatus-backfill-fixes/result.md` 第 4 条「管不到的」）。
- 更小的改法：不新增测试目标，往已有的 GPU 一侧测试里加参照表循环（那个文件已用 `#[path]` 编进参照表，`crates/singlefs-background-gpu-tests/tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs:12`）；覆盖相同，只省一个测试目标的编译。关窄的一半要把「组」与「用例真交的参照函数」绑起来（改用例签名），两条路都不更小。
- 推翻：把一组入口改成没被登记的编得过的 fn，新测试变红，这一条「仍开」就不成立（推的，没量）。

## 3 第三节七条改法：补丁做的是不是判决写的那一条

补丁两个文件：装置、新测试目标（`changes.patch` 1384 行）。变异表：追加 25 行、换 3 行、删 2 行。我核了 28 个项名各在 `crates/mutations.tsv` 的第一列恰好出现一次，删掉的两个名字 0 次（量的，`cut -f1 | grep -cxF`；表 sha256 与快照一致）；实现员命令里 `--item` 28 个与这 28 个一一对上（`comm` 两个方向都空）。

### 3.1 续跑核卡的身份（零轮）

- 做了：全部卡臂那一行加 `models`、四个字段逐个比（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:271-283`；装置 `:5997` 起的 `CardsRecordedForAnArm`，字段「fields_compared_on_resume: Vec<(&'static str, String)>,」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/run_resumed_by_hardware_group.rs:379`））；点名一张卡臂新行 `name=one_card_arm_card`（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:225-243`）；并发跑同记同核（装置 `:7839` 调用，补丁 `research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:514-534`）；对不上打停下行、退 2（`:6206`，`const EXIT_USAGE: i32 = 2;` 在 `:4125`）。
- 做多（判决没要）：① `models` 字段，判决只说「卡的标识」；② 停机分句改名 `cards_differ_from_the_product_being_resumed`（`:6206`），停下那一行改成逐字段（旧名在 `crates/` 里 0 命中，只剩 `records/` 与第一轮材料里）；③ 每一行记卡行都核，不只头一行；④ 全部对上才打记卡行，一条不对一行都不打；⑤ 按臂认结果行的规则（属下一条）。①②③ 判决的意思里都有，不算越界；④ 是实现员的设计。
- 做少：换台认不出同槽同型号（2.1）；点名一张卡臂记的卡取「let lowest = cards.discrete_gpus.first()?;」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/command_opening_and_construction_lines.rs:14`），真正开的卡取另一处「.take(1)」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/arms_and_background_compute_settings.rs:167`），两处各自从排好序的卡表取头一张，靠注释保持一致，没有测试把「记的」与「开的」绑在一起；变异行只改记的那一处（`.last()`）。推的：两处以后分叉，记的与开的会不同而不报。
- 零轮 A/B 的顶法：成立，在函数级。全部卡臂：改前读数 `stopped=None`（第一轮），改后同一形状写成单测 `resuming_a_product_handed_other_cards_of_the_same_count_stops`（装置 `:13117`），变异行「E202 续跑核卡：全部卡臂只比张数与开法、不比每张卡」把改前行为做回去，任务 19641 全抓到（主 agent 给的读数，我没复跑）。点名一张卡臂与并发跑：攻方没探，没有改前读数，只有读改前代码（只认全部卡臂的函数）与变异行，弱一格。接线层（`lockstep_command` 里 `:6552`、`concurrent_command` 里 `:7839` 两处调用与随后 `product.finish()` 退出）没有测试，也没有变异行（实现员自己写了没补，转述：第二份报告「没做什么」）。

### 3.2 一张都没交的那一趟不挡续跑（零轮）

- 做了：判决给的两支里取了「续跑认得出而放行」：产物里没有这种卡行时，只在有这条臂跑出的结果行才停（「if !the_product_has_runs_mounted_with(」，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/command_opening_and_construction_lines.rs:106`）；按 `arm=` 认臂，多判臂算它那一种，并发跑的 `name=concurrent_run` 照 `ConcurrentArm` 认（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:288-303`）。单测 `a_session_that_ran_no_arm_does_not_block_a_later_resume_on_a_host_handed_cards`（装置 `:13040`）喂「一趟一张没交、一趟 `--cards` 超出」的行；变异行换回改前条件 `lines_already_in_the_product.is_empty()`（`mutations-append.tsv` 里「一条结果行都没跑出来的产物也当成对不上」那一行）。
- 做多：按臂认比判决要的宽：只有 CPU 臂跑出过结果的产物，之后点上全部卡臂也接着跑（单测 `only_the_runs_mounted_like_the_arm_make_its_missing_cards_line_stop_the_resume`，装置 `:13243`）。登记第 12 条的字面支持：「产物里有这条臂跑出的结果行（`name=run`、`name=concurrent_run`）而没有这种行」（`research/prompts/e202-preregistration.md:483`）。没有判决字面，但不与它冲突；注意它放行的产物里 CPU 臂的重复是在别的条件下量的，续上的 GPU 臂在不同重复上没有 CPU 臂可对（推的，没量，且改前也是被挡的那一类）。
- 做少：2.2 的「写了卡行、没跑出结果行」那一形，改前改后都挡；头注释的一般句子比代码宽一格（`:41`）。
- A/B：函数级成立（探针 `stopped=Some(...)` 与新单测、变异行换回改前条件）；新单测用的是手写的产物行，写法照 `lockstep_command` 在没交卡那一支写下的行（`name=gpus enumerated=false count=0`，来自 `emit_the_gpus` 的格式），我读码核了格式相同，没拿真产物比。

### 3.3 crates 摘要算上装置自己（量过）

- 做了：起走路径五处（`const STARTING_PATHS_OF_THE_CRATES_DIGEST: [&str; 5] = [`，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:4135`），目录逐层走、文件取自己；走树改成注入读盘（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:91-125`，`what_the_path_is_on_the_disk` 在装置里）。单测 `the_crates_digest_covers_the_apparatus_and_the_two_modules_it_compiles_in_by_path`（装置 `:13754`）+ 一行变异。
- 做多：把走树抽成可注入的函数、加了 `PathWalkedForTheCratesDigest` 枚举（为了测试，不是判决要的）；错误话「读 {path}（装置要从仓根起）」保留。
- 做少：改后读数没量（2.4）；验证包两块没进（2.4 我的判断是不该进）。`name=crates_version` 的 `files=` 与 `combined_sha256` 随之变：补丁前后两趟同一份产物会照停机条款第 2 条停（实现员第一份报告第 3 条自认，不兼容旧产物，合共用约束）。

### 3.4 跑前登记修订第 12 条（登记 `research/prompts/e202-preregistration.md:480-485`）

逐句对装置今天的样子：

| 第 12 条的句子 | 装置今天 | 对得上吗 |
|---|---|---|
| 卡只用服务交的：环境变量有卡串只开那几张，没有而有领卡通道时造完起步镜像、开设备之前领，两样都没有报错退出，不自己枚举 | 头注释「卡只用服务交的（[`cards_handed_to_the_apparatus`]）：环境变量 `GPU_CARDS_BY_PRIORITY` 有卡串就只开那几张；没有而有领卡通道时，」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:35`）、`:5746` 常量 | 对得上 |
| `--cards N` 开前 N 张；N 小于 2、不是数、多于交来的退 2；一张没交退 77；只交一张报本次未跑 | `:2819` 判只交一张本次未跑；`return EXIT_NO_ARM_RAN;`（`:6597`）；退 2 在 `:6525` 一带 | 对得上 |
| 点名一张卡臂：交来的卡里的一张，哪一张记进产物 | `:6044` | 对得上 |
| 两种行；交替跑与并发跑都打 `one_card_arm_card` | `:6552`、`:7839` 两处调用 | 对得上 |
| 续跑核卡「并进第十一节停机条款」，打 `clause=cards_differ_…` 退 2 | 装置退 2（命令行不对）；停机条款统一「停机时产物照打、退 3，交主 agent。」（`research/prompts/e202-preregistration.md:442`）；装置头注释把退 3 留给停机条款（`:28`） | 差一个词：「并进停机条款」与「退 2」同句并存。建议改「并进」为「按停机的精神处理（不当结果、交主 agent），退出码 2」 |
| 「什么观测会让它触发：……换了卡、换了张数或换了一台」 | 同槽同型号的另一台认不出 | 句内不严：末段已认（`:485`），触发句应写「换了一台且那一台的卡地址或型号不同」 |
| 一条结果行都没跑出来的那一趟不挡以后的续跑 | 2.2 的一形仍挡 | 收窄为「没写过卡行的那一趟」，或同装置头注释一起改 |

「服务交卡」而任务服务已不分卡：事实是「统一任务服务只调配 CPU」（`.claude/rules/verification.md:50`），提交带 `# task-server-gpu-cards:` 声明的脚本会被拒：「卡归 GPU 批处理验证服务，任务不领卡」（`task-server/task_server_lib.py:1781`）。第 12 条那一句说的是装置的接口（卡串从哪来），没说谁来交；今天没有谁给任务设这个变量。我的建议：第 12 条不改成别的交法（接口没定型，写一个还没有的接口就是编造）；在管 E202 的会话改装置头第 4 行的那一次，另起一条修订写「交卡方是 GPU 服务」，仍在产物之前。推的；推翻：GPU 服务代跑的接口把卡串的来路换成别的形状，第 12 条那句字面就要改。

### 3.5 参照登记表的测试与变异、三条单测的变异行、攻方列的判红分支、按名字取臂

- 参照登记表：新测试与两行变异做了（2.5）。三条没有变异行的单测：`a_subcommand_without_the_every_card_arm_refuses_the_cards_option` 挂「E202 concurrent 也认 --cards」；两条真卡用例各挂一行（「内联真卡对拍：持卡判拿每段多一槽的参照比…」「小池 greedy 调用：丙段那一截的次数报成 0」，带 `tests-on-a-real-gpu`，没卡，用户定记本次未跑）。这两行没有跑出过「抓到」，只在静态上锚点对上（任务 19655 mutation-tables 格绿，主 agent 读数），所以这两条「补了行」不等于「有判别力」。
- 判决写「攻方第八节列的」，攻方那张表在第九节：标题「## 九、123 行变异罩不罩得住第 1 到 5 件的每个判红分支（不罩全）」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:198`）。实现员第一份报告也写了「派发写第八节，按内容是第九节那张表」。判决另一处写「新单测 30 条，3 条没有变异行」（`research/prompts/gpu-tests-src-backfill-code-r1-main-verification.md:33`），正推报告写「另有改过的老用例 1 条（10008）和那条真卡对拍」（`research/prompts/gpu-tests-src-backfill-code-r1-sonnet-output.md:404`）：真卡对拍用例是改过的老用例，不在那 30 条里；新的没变异行的只有两条（子命令拒 `--cards` 一条、greedy 真卡一条）。
- 攻方那张表逐处对补丁（「补」是补丁里有测试加变异行；「没补」实现员写了理由：要真环境或要先拆成纯函数，转述 `~/.local/share/singlefs/work/e202-apparatus-backfill-fixes/result.md` 第 5 条表末行）：

| 攻方列的分支 | 补丁 |
|---|---|
| 领卡没成那一支（`map_err`） | 补：`a_failed_lease_through_the_channel_is_reported_as_the_lease_failing`（装置 `:13816`）加一行 |
| concurrent 拒 `--cards` | 补：一行 |
| 续跑核对只比头一行 | 补：`resuming_a_product_whose_later_cards_line_differs_stops` 加一行 |
| lockstep 里接续跑核对 | 半补：核与打拆成纯函数 `:6185` 起，单测加一行；两处调用点（`:6552`、`:7839`）与随后退出没补 |
| 释放标记、盘号、跨槽数的编码 | 补：三行，挂在 `the_record_stage_…` 上 |
| `whether_of_the_number_written` 读到 0、1 以外 | 补：`:13830` 加一行 |
| 持卡判里「GPU 与装置自数不同就 panic」 | 补一行，真卡，没跑 |
| `opening_of_the_command` 没来路退 78、`cards_taken_for_this_process`、`cards_seen_by_the_command` 退 78、先造池后取卡的次序、`lowest_card` 与 `name=run` 型号串的调用点、持卡判没来路或空卡交回没跑成 | 没补 |

  做少：判决写「逐处补」，补了七类，没补的六类都写了理由（要真环境或先拆函数）。做多：无，新测试都对得上攻方那张表或判决点名的单测。没补的算不算打中，要看项目对「单测够不着的取卡链」的口径：攻方自己写那几处「要真卡或 nvidia-smi，单测走不到」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:210`）。
- 按名字取臂：新加的三处（改前 `:12651`、`:12654`、`:12656`）已无，现查 `grep -nF 'LOCKSTEP_ARMS[' …` 剩七处：`:5378`、`:7986`（两处同一行）、`:8038`、`:10665`、`:10942`、`:10943`。七处都在第一轮之前就有：`git show 74baab8e1^:crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs` 里同样的行在 `5239`、`7243`（两处）、`7299`、`9660`、`9936`、`9937`（量的）。其中 `:5378` 是非测试代码「let cpu_arm = LOCKSTEP_ARMS[0];」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/lockstep_tier_judgement_and_repeats.rs:140`），次序一改悄悄换臂；判决只要求新加的，旧的归欠账（3.7）。

### 3.6 并行下一次性库用例与真卡用例的冲突

- 做了：守卫包装 `with_the_one_library_kept_for_this_process`（装置 `:13842`），真卡对拍整段包在里面（「with_the_one_library_kept_for_this_process(|| {」，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/pools_cards_and_sessions_for_tests.rs:205`）；复现测试 `a_disposable_library_is_set_only_after_the_pairing_through_the_one_library_returns`（`:13854`）；变异行把守卫换成 `()`，证红里这一行抓到（任务 19641，主 agent 读数）。
- 守卫罩不罩全开真库的每条用例：这个二进制里开全仓那份库的只有真卡对拍一条。证据：装置里 `pairing_concluded_on_the_cards_through_the_pairing_library` 只命中两处（`grep -nF`）：`:8740` 是注释，`:12383` 是调用；greedy 真卡用例读 `RunStage::of_this_process().judges()`（`:12409`），不开库。包内设一次性库的文件只有装置与 `pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs` 两个（`grep -rln make_it_the_library_of_this_process`），后者是另一个测试二进制、另一个进程，同进程冲突碰不到。
- 守卫有没有把本可并行的用例串起来：没有。它拿读写锁的读侧（`pub fn keep_disposable_libraries_out() -> Self {`，`crates/singlefs-verification/src/crash_library.rs:911`），写侧只有设一次性库那一处（同文件 `:854`）；被串的只有设一次性库的两条用例（先录后判那条、复现那条）与真卡对拍，它们本来就不能同进程并行（写侧本来互斥）。读侧可重入问题：`CrashLibrary::open` 不再拿这把锁（`DISPOSABLE_LIBRARY_SETTING` 全文件除定义（`:891`）外只在 `:853`、`:913` 出现），没有同线程再拿读侧的路。
- 做少：① 守卫放在装置这一侧，共用模块那个函数本身没拿（共用模块在实现员的「不碰」清单，转述第一份报告第 6 条）；以后包内别的目标要在同进程设一次性库，同样会冲突。更小的办法没有：放进共用函数是改共用模块，更大。② 复现测试证的是锁序（设一次性库的线程等对拍段返回），不是原症状（开库开到临时库、退 75）；判决允许「复现不出写明」，实现员写了「复现得出就是变异行让它红」。③ 真卡用例里包装的调用点（`:12382`）带特性、要卡，没有变异罩着：删掉那层包装，没有哪条测试红（推的）。
- 做多：无。
- 一格我没验：复现测试里等 200 毫秒（`:13836` 的常量 `TIME_THE_TEST_STAYS_INSIDE_THE_PAIRING`）在重载下可能让「没守卫」的变异误过（主线程被挂起超过 200 毫秒）；有守卫时恒绿（锁保证次序，不靠计时）。所以它只会漏抓、不会误红。推的。

### 3.7 旧代码里的线索登记欠账

- 判决：「第 11 条那几处旧代码里的线索登记欠账，不在这一批改。」（`research/prompts/gpu-tests-src-backfill-code-r1-main-verification.md:48`）。
- 现查没有落点：`grep -n 'E202\|e202' .claude/kb/checks-owed.md` 0 命中；现行账本 `records/2026-11-17-subagent账本自上而下清欠.md` 里搜 `E202`、`judge-trend`、`GPU 测试包` 0 命中；全仓 `.md`、`.tsv`、`.json` 里搜 `gc_wall_clock_of_the_run`、`full_segments_of_a_device`、`rejudged_live_slots` 只命中 `records/2026-11-13-singlefs-background-gpu与测试包严格审核.md`（那里有 T02，对应线索 6 一条）与第一轮材料。线索 3、7、10、11 没有落点。
- 建议并进同一节的几处（都是现状，在今天的树里）：stage-three-cost 无卡退 0（`:9078`）；greedy 真卡用例拿不到实例静默返回（`:12422`）；产品段按下标取臂（`:5378`）；登记未定项 4「缺：主 agent 认「这一族没有判定要进库」」（`research/prompts/e202-preregistration.md:79`）。加节要走 `scripts/kb/ledger-append-section.py`（只主 agent）。

## 4 检查单八格

命令都是我在补丁文件上现跑的（只读 `grep`）：`sed -n 1,537p changes.patch`（补丁里非测试代码的部分；`mod tests` 的块从 `:538` 起）取 `^\+` 行再数。

| 格 | 判 | 证据 | 什么会推翻 |
|---|---|---|---|
| 走不到的理由二次查实 | 成立 | 实现员原文是「无」，转述自 `~/.local/share/singlefs/work/e202-apparatus-backfill-fixes-second/patch/result.md` 「走不到的理由」一节。我自己搜：非测试新加行里 `todo!`、`unreachable!`、`assert!`、`assert_eq!`、`panic!`、`unwrap(`、`expect(` 命中 0 行（量的：`grep -cE` 输出 0）。新加的兜底「_other_kind => None,」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/command_opening_and_construction_lines.rs:85`）是对 `&str` 的兜底，不是走不到的分支。新加的 `.expect` 全在测试里，消息写的是前提（「点了全部卡臂、这一台交了卡」「五处都在小树里」「登记表里的入口都写成 singlefs_background_gpu_tests::<模块路径>::<fn>」） | 非测试段里有一处我的正则没抓到的 panic 来源（例如索引越界），要读全 `:6018-6210` 才能排；我读了函数体，没有下标运算 |
| 类型与名字 | 成立（按格的字面），写两处弱点 | 不缩写、不写序数名；`LOCKSTEP_ARMS[下标]` 新加三处已无（3.5）；新加的 `expect` 都写前提。弱点：① 兜底分支两处：`_other_kind => None,`（`:6094`，对字符串）与新测试目标里的 `_ => {}`（`crates/singlefs-background-gpu-tests/tests/reference_units_entries_name_functions_that_exist.rs:55`，对切片）；同形的先例是 GPU 一侧测试里的「_ => {}」（`crates/singlefs-background-gpu-tests/tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs:302`），字符串与切片模式没法穷举，clippy 的 `wildcard_enum_match_arm` 只管枚举（合入后验证的 clippy 过，任务 19622，主 agent 读数）。② 卡的身份在 `CardsRecordedForAnArm` 里是「fields_compared_on_resume: Vec<(&'static str, String)>,」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/run_resumed_by_hardware_group.rs:379`）：键是字面、值全是 `String`，地址与型号靠键名区分不靠类型；`pci_bus_addresses_and_models_of`（`:6018`）返回两个同型 `String` 的元组，互换不会编不过，靠单测里整行比对挡。③ `FirstHandedOverCards` 里的 `First`（`:5916`）第一轮正推就记了「推不出」，今天没动 | 主 agent 把「同型不同义的字符串」按「裸整数」那一类算（格的字面只写裸整数），则不成立，要换成地址、型号各一个 newtype；或把 `First` 算位置代称 |
| 不兼容旧代码与旧格式 | 成立 | 新加行里 `legacy`、`旧版`、`旧产物`、`旧库`、`兼容`、`迁移` 命中 0（量的）；旧停机分句名在 `crates/` 里 0 命中，没留别名。「没有卡行」走的是同一条停下的路，条件是有这条臂的结果行（`:6115`），不认版本。唯一放行的一支是「这条臂没跑出结果」，新产物也会走到（`--arms cpu` 的一趟），不是只为旧产物 | 出现按版本分支的新加行 |
| 不新建判定库 | 成立 | 记卡行追加进 `--product`（实验产物，同第一轮判决第 8 条）。补丁里新开存放处的写法搜 `sqlite|rocksdb|create_dir|File::create|OpenOptions|fs::write|DirBuilder`：只命中 `include_str!(concat!(env!("OUT_DIR"), "/reference_units_keys.tsv"))`（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:1266`，构建脚本编译期派生的键表，GPU 一侧同做法）；`DisposableLibrary::new`（`research/prompts/gpu-tests-src-backfill-code-r2-snapshot/changes.patch:1215`）是测试里对临时服务起的自证库，共用约束「不算」 | 有一处写进 prune-server 之外的判定存放处 |
| 所有任务多机可用 | 成立（按格的字面），写一处我不替主 agent 定的弱点 | 新代码读的本机状态是服务交的卡（各台各得各的），那是被判的量本身，没有写死主机名与地址。新测试：参照登记表读编译期 `OUT_DIR`；摘要测试是内存小树；守卫测试起 `DisposableLibrary`（临时目录在 `std::env::temp_dir()`，`crates/singlefs-verification/src/crash_library.rs:746`），与已有的先录后判那条同做法，不改变判定结局。弱点：卡的身份取「PCI 地址加型号」，同槽同型号的另一台认不出（2.1），CPU 臂随台变而改法不认。把机器记进那一行：主机名不能写（`.claude/agent-common.md:44`），可行的是不可逆摘要或卡自己的编号，今天装置拿不到编号。这一格我判「成立（字面）」；若主 agent 按精神判，这个缺口算「推不出」、算打中 | 两台之间存在同槽同型号的卡，并且续跑落到另一台，产物里两台的重复混在一起而不停 |
| 测试优先多线程 | 成立 | 新加行里 `test-threads`、`RUST_TEST_THREADS`、`set_var`、`set_current_dir`、`available_parallelism` 命中 0（量的）。守卫是读写锁的读侧，只让设一次性库的两条用例等（3.6），别的单测照并行；新单测都是内联小用例，不走崩溃枚举。复现测试睡 200 毫秒，只占一条线程 | 守卫的读侧在持有期间挡住别的并行用例 |
| 带 --item 的命令与项 | 成立（静态部分） | 实现员命令里 `--item` 28 个与追加 25 加换 3 的项名一一对上（`comm` 两个方向都空），28 个名字在 `crates/mutations.tsv` 第一列各恰好一次，删掉的两个名字 0 次（均为量的）。主 agent 交了 26 行全抓到（任务 19641），两行带真卡特性没交。没核：28 项能否被 `--list-items` 列出，命令写在第 6 节 | `--list-items` 列不出其中一项，或 26 行之外的两行在有卡的机器上没红 |
| 新崩溃流多罩了什么 | 成立（本轮没有这一类） | 补丁新加行里 `crash-case:`、`enumerate_crash_states` 命中 0（量的）；用到的 `CrashLibrary`、`RunStage` 是判定库的存取与选段，不枚举崩溃状态 | 补丁里出现调 `enumerate_crash_states` 一族的新测试 |

## 5 第二节记「没打中」的几条：两条腿是不是真都覆盖到了

| 判决条目 | 覆盖情况 | 依据站不站得住 |
|---|---|---|
| 第 1 条 取卡只用服务交的卡 | 两条腿都读了：攻方「取卡之前有没有哪条臂用了卡」一节（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:152`）与各子命令出路表；正推「件 1」整节（`research/prompts/gpu-tests-src-backfill-code-r1-sonnet-output.md:28`） | 站得住，但判决条文漏带一个两条腿都报了的边角 |
| 第 3 条 开卡表整个进程一份 | 两条腿都读了；攻方留一处推不出：「产品自己的进程开卡器另有一张表，装置不经它开卡（推不出有没有第二张表在用）」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:19`）；正推读了产品一侧（`research/prompts/gpu-tests-src-backfill-code-r1-sonnet-output.md:98`） | 站得住，正推补上了攻方推不出的一处 |
| 第 5 条 先录后判经判定库 | 主体两条腿都读了（攻方第六节、正推第 6 节）；并行冲突只有正推看见；攻方的弱打中只有攻方看见 | 站得住的一半，另一半见下 |
| 第 8 条 判定行追加进 `--product` | 两条腿同判，依据都是 `.claude/rules/verification.md` 同一节 | 站得住；正推的保留意见判决没接 |
| （第 4 条 接口改名，没点名问） | 只有正推读了（核心层新名与旧名零命中）；攻方没覆盖 | 单腿依据；我现查 `AnswerOfTheCallerBeforeTheNextBatch` 在 `crates/singlefs-core/src/compaction_plan.rs:1541`，对得上 |

### 5.1 第 1 条漏带的边角：stage-three-cost 一张都没交时退 0

- 判决条文写「没卡的出路退 78、77」（`research/prompts/gpu-tests-src-backfill-code-r1-main-verification.md:22`）。两条腿都另报了 stage-three-cost：攻方「stage-three-cost 在这一台一张卡都没交时只跑 CPU 臂、退 0」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:17`），正推「备注 D（推不出，不是这一次改的）」（`research/prompts/gpu-tests-src-backfill-code-r1-sonnet-output.md:54`）。判决在「没核的」一条里记了没量，没进条文、没进改法。
- 今天的样子：「if does_every_arm_agree {」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:695`）；`stage_three_cost_command` 里没有 lockstep、concurrent 那样的空卡早退（`return EXIT_NO_ARM_RAN;` 只在 `:6597` 与 `:7876`）。能走到要「服务交了卡、却没有这一台的」，取卡模块交回 `None`；可达性推的，两条腿都没量。
- 辩方意见：这是第一轮之前就有的行为，不是这一批打出来的，不在这一批该改的射程；但两条腿都报了，判决条文不该写成「退 78、77」一句话带过，应登记（3.7）。推翻：在 `stage_three_cost_command` 里看到空卡早退（我读 `:9003-9080` 没有）。

### 5.2 第 5 条：攻方弱打中没进判决，今天还在

- 攻方：后取卡的那条用例拿到缓存里的空卡，`a_greedy_call…` 在拿不到实例时静默 `return`，门禁按条记库认记号归谁，「后跑的那一条开始时，前一条打的那行已经过去了」（`research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md:160`），可能被记成绿、之后按单元键复用；攻方自己判可达性推不出（服务按需领卡会摘掉卡串）。
- 判决第 5 条只记正推的并行冲突，没记这一条；补丁没改。今天：「if cards.vulkan_instance.is_none() {」后一行 `return;`，没有 `NOT_RUN_NO_GPU_ADAPTER`（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/cards_of_the_command/tests.rs:197`）；对拍那一支同一判断交回 `NotRunOnTheseCards`，不入库（`:8930`）。
- 辩方意见：弱打中不是打中，攻方也没当打中；可达性条件（服务把别台的卡交过来）与 5.1 同一类，建议同并入 3.7 登记，不要求这一批改。推翻：证出这一形在经服务交的任务里走得到。

### 5.3 第 8 条：保留意见没接

- 正推：登记未定项 4 写「缺：主 agent 认「这一族没有判定要进库」」（`research/prompts/e202-preregistration.md:79`），没找到认的记录。判决第 8 条的依据是「它是实验产物，装置 `admission: always`、每次现量」，没有回答这一句。攻方的前提（卡与台不核、装置版本不核）已被改法 1、3 补了一半。
- 辩方意见：判决结论站得住，只缺主 agent 补一句「认」。

## 6 要主 agent 跑的命令

我没起任何项目代码，没写模型，没有 `rerun.sh`，没建仓副本与编译目录。读数里标「推的」的都没量。只有一条要主 agent 交：

1. 28 个新变异项名都列得出（静态，不起 cargo）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的 --list-items; 范围=只列项; 为什么=核这一批 28 个新变异项名都列得出、键对得上、锚点没失效' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs`
   在哪个目录起：仓根 `/home/fy5090/code/singlefs`。
   跑完看：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 28 个项名（`comm` 对 `crates/mutations.tsv` 第一列里追加与替换的 28 个）都在；末尾「键对不上」与「锚点失效」两个条数都是 0。

## 7 什么现象会推翻这些结论

- 2.1：续跑发生的两台之间卡的 PCI 地址加型号恒不同，或续跑被钉在同一台。
- 2.2：被杀在头一条结果行之前的一趟，接着在另一组卡上续跑不停。
- 2.4：`files=` 在仓根起装置读出的不是 64，或验证包里的改动能改变量 W、C、S、P。
- 2.5：把一组入口登记成没被登记的编得过的 fn，新测试变红。
- 3.1：改成「记的」与「开的」不同的卡，有测试红。
- 3.6：删掉真卡用例里的守卫包装，有测试红。
- 4 节：见各行最后一格。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑，命令在第 6 节。
- 没跑第一轮攻方的探针，也没复现任何一处「推的」；2.2 的「写了卡行、没跑出结果行」一形、2.4 的 `files=64`、3.1 的「记的与开的两处分叉」只读码，没量。
- 没读禁读清单里的同一轮攻方腿报告，没读别的轮的腿报告；读了判决点名的第一轮两份腿报告、第一轮判决，与实现员两份报告（工作根下，不是腿报告）。
- 没碰真卡、没验那两行带真卡特性的变异（用户定记本次未跑）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/gpu-tests-src-backfill-code-r2-defense` 里只有两份名单文本（`rows-names.txt`、`items-in-command.txt`），没有副本与编译目录；`du -sh` 量得 28K（交回前量的，含 `cite.txt`）。
