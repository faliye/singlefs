# judging-keys-input-only-r2 攻方腿（Opus）报告（2026-11-20）

攻击面：第一轮的修补本身（判决「三、改法」六条与用户在腿交回后定的三格）、产生结论的方法、从未被看过的地方（GPU 服务一侧补丁、prune-server 删库路径、变异与广谱变异的键、两台多机下的复用），另攻耗时与复用。方案按主工作区与集成副本 `~/.local/share/singlefs/work/judging-keys-integration/tree` 谈；引集成副本写「集成副本 <路径> 第 N 行」。

没写模型、没有 `rerun.sh`：这一轮判的是键由什么组成、复用走哪条路，不涉及崩溃状态，没有要枚举的历史。用到的数有两类：

- 量的：集成副本一次构建写出的判法单元键表（任务服务给集成副本建的按树编译目录里，验证包构建脚本写进 `OUT_DIR` 的 `judging_units_keys.tsv`；构建脚本输出里 `rerun-if-changed` 指向 `~/.local/share/singlefs/work/judging-keys-integration/tree`）。两份拷进草稿目录（之后 `gzip -n -9` 压成 `.tsv.gz`，下面的 sha256 是解压后的内容）：`~/.local/share/singlefs/work/judging-keys-input-only-r2-opus/keys-table-ed3b64-4e85.tsv`（sha256 d8618b0a1a4726b296a87d198a0c98d1682c322c489a7c7a3163b236cfbd28af，683,078 行）、`keys-table-ed3b64-7869.tsv`（sha256 848b2ccfa606ccf8c76b9d815743107587aec326a0f3f2f51b6fbd5d5f54bcd6）。我只读、用 awk 数，没跑项目代码；这些数主 agent 还没复跑，判决引用前照「要主 agent 跑的命令」复核。
- 推的：基线报告 `~/.local/share/singlefs/work/judging-keys-baseline-costs/result.md` 的分段秒数（那份报告标「量的」，我没复核）乘上的比例与速率。

## 各格判定一览

| 格 | 攻击面 | 判定 | 一句话 |
|---|---|---|---|
| R1 强制现判按比例或轮转 | ① | 打中（新问题、耗时） | 修得了「发现」，修不了第 4 格「换代」：判器改了绿结论内容之后，按比例抽中的每一块都存不进（同键结论不同），服务点名跳过、待判行留着；没改动的趟也稳定多付 p ×（录入段 + 判段） |
| R2 阳性样本集 | ① | 打中 | 字面「每条不变量两边现判一张违例镜像」走不出去：I-1.13 GPU 内核不置位；I-8.4 只在二次崩溃上判、没有单镜像违例；「不入库」与 GPU 服务只认库里待判行不相容；只抓「整条判法失效」，抓不到某一支变弱 |
| R3 人登记、整族删 | ①③ | 打中（耗时、两台） | 删前缀分帧、不原子，另一台在判的片删完照写；块属性不再记判器版本，删不了「某一版判的」那一部分，只能整族删，每次换代全库重判，与「改判器不作废」相抵 |
| R4 不变量名单不同不复用 | ① | 打中（新问题） | 今天已有收尾核对：不变量清单变了，复用块的计数向量对不上就判红，键不变、下一趟照复用照红（走不出去）；R4 把它换成整库重判，又拿「名字」当复用依据，撞 D13（验证路线） 已定项 17 第 436 行 |
| R5 观察者修订号 | ①⑤ | 打中（耗时） | 任何观察者代码改动都逼换号，整段观察者重录（基线录入段约 84,710 秒 / 9 项），把新键省得最多的那一段又花回去 |
| R6 布局号 7 与删库次序 | ①③ | 打中 | 布局行不在时第一个开库的进程写下自己的号；删库连 `00` 一起删，另一台或旧树编的二进制先开库写下 6，新代码全被拒；不删 `00`，新代码开库就拒。次序里没有「谁写下 7」 |
| 用户格：产品代码进键 | ①② | 打中 | GPU 判器不跑产品代码（D13（验证路线） 第 38 行），产品改动让放量里 GPU 判的块全部重判，判出来的结论与产品改动无关；只有固定的对拍流与 CPU 那一路看得到。量的：这一组 1,770 个单元（core 1,673） |
| 用户格：录入按屏障 | ①③ | 假绿方向推不出；耗时方向打中 | 录入键含录下的流的摘要，写流变了键必变；反过来，录入键另带每组平均 3,202 个 core 单元，流不变的产品改动照重录；观察者模块的单元进了录入键 |
| 用户格：工具链与依赖不进键 | ①④ | 对不上 + 打中（假设照做） | 记录第 99–105 行没有这句原话；集成副本三种组的键与录入键都还带工具链行与 `Cargo.lock` 收窄行。照做之后：rustc 或 LLVM 误编译、vstd 擦除改动让编出的产品恢复变了而键不变；变异行在新工具链下该记无效而照复用抓到 |
| 问 2 取宽取窄 | ② | 两向都打中 | 窄：GPU 路线（同上）。宽：六个入口并成一组，三种结论与对拍共用一把，只经核对器够到的产品单元也作废镜像判定；同一棵树两种构建算出两把判器键，差的 5 个单元全是 core 里只给测试计数的 `lookup_work`（量的） |
| 问 5 耗时与复用 | ⑤ | 打中 | 不降反升的四处：R5、R3、R1 的稳态开销、产品代码进键后 GPU 盲判的全库重判；第一轮的每一处都是推的，按段量级见第五节 |
| 方法：第一轮第 2 格 | ② | 打中（缩格） | 「漏判前后逐字相同」只对「判到计数与恢复字都不变」那一类成立：强制现判的比对含计数向量与恢复字 |
| 方法：第一轮第 1 格 | ② | 打中（补数） | 发现率是块数的函数：9 项共 13,791 块、每趟强制 9 块（0.065%）；判器改动均匀落在 1% 的块上，一趟抓到的概率约 8.6%（算的） |
| 变异与广谱变异「驱动不进键」 | ③ | 打中 | 判「抓到」的逻辑改了，旧的抓到照复用，只靠每趟一条强制现判；工具链不进键时，替换文在新工具链下编不过（该记无效）照复用抓到 |
| 两台多机 | ③ | 打中 | GPU 服务每台一个进程、结论不记判器版本，装错的一台判的块事后认不出；仓里没有钉工具链的文件，两台各取各的 cargo，键去掉工具链之后两台共用、误编译类差别不再被分开 |

## 一、先说对不上的（我核过）

1. 「工具链与依赖版本不进键」没有用户原话出处。正文说用户定案在记录第 99–105 行，那七行里「工具链」「依赖版本」命中 0 次（`awk 'NR>=99 && NR<=105' records/2026-11-19-判定键只算产品与测试代码.md | grep -c '工具链\|依赖版本'` 输出 0）；第 105 行只有编译档与线程数（「编译档与线程数原话」（`records/2026-11-19-判定键只算产品与测试代码.md:105`））。第一轮判决自己写的是「工具链与依赖版本也不进键是 mainagent(B) 消息里转述的用户补定，第二轮材料里补原话出处」（`research/prompts/judging-keys-input-only-r1-main-verification.md:34`），第二轮材料没补上。记录第 110 行那一句是主 agent 的概括，不是原话。
2. 集成副本没照这一格做。三种组（录入、观察者、判器引到的产品代码）的键都拼工具链行与收窄的 `Cargo.lock` 行：集成副本 crates/singlefs-unit-graph/src/entry_groups.rs 第 351 行「toolchain_lines: toolchain_lines.to_vec(),」、第 352 行「lock_lines,」；构建脚本取 `$RUSTC -vV`、`$CARGO -V`、`TARGET`、`CARGO_ENCODED_RUSTFLAGS` 与 `Cargo.lock`（集成副本 crates/singlefs-verification/build.rs 第 12 行）；录入键另拼这个进程的工具链（集成副本 crates/singlefs-verification/src/crash_library/item_manifests_and_two_stages.rs 第 50 行「recording_units_key_of_the_test_on_this_thread(toolchain_of_this_process())」）。量的：键表开头 10 行 `toolchain`（rustc 1.98.0、LLVM 22.1.8、cargo 1.98.0、目标三元组、rustflags 空）。编译档那一半照做了（同一份 build.rs 第 12 行「编译档（`PROFILE` 与 `[profile.<名>]`）不读、不进键」）。
3. 第一轮第 2 格的前提比判决写的窄，见第六节。

推翻条件：用户另有原话把工具链、依赖版本也排出键，出处找得到，第 1 条撤；集成副本之后的修补去掉了这几行，第 2 条撤。

## 二、问 1：第一轮「三、改法」六条

### R1 强制现判按比例或按块键哈希轮转（修第 1、4 格）

改法原文：「强制现判从一块改成按比例或按块键哈希轮转取一批」（`research/prompts/judging-keys-input-only-r1-main-verification.md:52`）。

构造（判器改动让绿结论内容变，第 4 格）：

1. 前缀：放量一项判完入库，块键 K 下存着绿结论 C（判定向量、计数向量、恢复字）。
2. 改动：池级 checker 新判出一类「判到」（某条不变量此前在这类状态上答不适用，改后答判到、成立）。结论内容变成 C'，计数向量里那一条的判到数变了；块键不变（判器不进键，D13（验证路线） 第 444 行「改判器不作废已判的块；判法改了对不对，靠对拍与复用哨兵抓」（`.claude/kb/decisions/13-验证路线.md:444`））。
3. 照 R1 抽比例 p：被抽中的每一块都现判出 C'，往同键存时拒（集成副本 crates/singlefs-verification/src/verdict_store/conclusion_blocks.rs 第 819 行 `DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`）。GPU 那一路：服务把这种错当「说明」点名、跳过（集成副本 gpu-verification-server/src/library_pending_pool.rs 第 449 行「别的（库里的行不合布局这类）交回说明，由调用方点名、跳过」，第 457 行列了这个错），待判行留着（推的，没跟到最后一层），等它的片照服务 README 第 307 行那一支等到被统一任务服务打回。
4. 结局：今天一块走不出去，R1 下变成 p × B 块走不出去；R1 只把「换代」更早暴露出来，没给出路，第 4 格修不了。

另两处新问题：

- 稳态开销：库里什么都没改的一趟，R1 也照 p 现判。同一个旋钮罩观察者录入时，这一趟多付约 p × 84,710 秒的录入与 p × 32,629 秒的判段（基线 9 项，推的，按块数线性）。p = 1/16 约 7,300 秒一趟。
- 两台：哨兵与强制现判只从「归这台机器的块」里抽（集成副本 crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs 第 18 行注释 `candidates=` 那一格），按块键哈希轮转时轮到的块在不在这一台由分份定，两台各轮各的，「N 趟罩全」要两台的趟数对得上才成立。推的。

推不出的那一半：R1 对第 1 格（发现）确实加强，发现率从每流 1 块升到 p；我没构造出「R1 下照样发现不了而一块时发现得了」的历史。

### R2 每趟两边现判一张阳性样本集（修第 2 格）

改法原文：「每趟两边现判一张阳性样本集（每条不变量至少一张违例镜像），不入库复用」（`research/prompts/judging-keys-input-only-r1-main-verification.md:53`）。

构造一（走不出去）：I-1.13 由 GPU 内核不置位，对拍比较处两边都去掉再比（集成副本 crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/main.rs 第 6 行「只屏蔽内核不置位、对拍不比的那几条」）。照字面，I-1.13 那张违例镜像 GPU 一侧永远判不红，R2 要么每趟红，要么另立一张豁免表。I-8.4 只在崩溃注入的二次崩溃上判、不进池级 checker 的清单（D13（验证路线） 已定项 1 M3 一格），没有「一张违例镜像」这种形态。

构造二（抓不到变弱的一支）：判器把 I-8.3 在 journal 环回绕时的那一支改坏（只在回绕的记录表上答成立）。阳性样本那张违例镜像不回绕，照旧红，R2 绿；放量里回绕的合法流本来就是绿，改坏前后逐字相同（计数向量里 I-8.3 的判到数不变）。R2 只抓「一条不变量整条失效」，与第 2 格要防的「红变绿」只交在那一小块。

构造三（与架构相撞）：GPU 服务只照库里的待判行判（D13（验证路线） 已定项 17「各个键由什么组成」表 GPU 服务认领待判行那一行），「不入库」的样本到不了 GPU 一侧。写进库又照样受复用：违例镜像判出红，红的下一趟照现判（第 440 行「键相同而库里记着红的，下一趟现判、不复用；只有判绿的可以复用」（`.claude/kb/decisions/13-验证路线.md:440`）），所以「不入库复用」这半句多余，要的是「写进库、按红每趟现判」。

R2 与已有的东西重叠：对拍测试目标已经「另有一条只改坏它管的那个字段的流」（集成副本 crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state/main.rs 第 10 行），每条不变量一条坏流、要 GPU 红在 CPU 红的同一处。R2 的实际增量只有「这几条坏流的对拍结论不复用、每趟两边重判」。它的代价是那一个测试目标每趟整跑，我没有它的挂钟（要量，见第十节）。

### R3 判法换代由人登记、经 prune-server 客户端整族删（修第 4 格）

改法原文：「判法换代由人登记、经 prune-server 客户端整族删那一种结论」（`research/prompts/judging-keys-input-only-r1-main-verification.md:54`）。

构造一（两台、删前缀不原子）：

1. 前缀：本机改了池级 checker，登记换代；第二台上一条放量的片正在判（它的二进制是改前编的）。
2. 本机跑 `prune_server_client.py delete-prefix` 删镜像判定那一族。删法分帧：「一帧至多 DELETE_PREFIX_KEYS_PER_FRAME 把，客户端接着来到前缀下没有更多的键」（`scripts/prune-server/prune_server_client.py:142`），删完 `--dry-run` 数到 0。
3. 第二台那一片在数到 0 之后写下几块结论（旧判法判的），键与新判法下同一块的键逐字相同（判器不进键）。
4. 结局：下一趟新判法照键复用这几块旧判法的结论，换代没换干净；块属性里不再记判器版本（D13（验证路线） 第 464 行「块属性里不再记 GPU 判器是哪一版」（`.claude/kb/decisions/13-验证路线.md:464`）），事后也挑不出来。要挡，得在删之前停住两台所有在判的片，而「停判段」没有一样会红的检查。

构造二（耗时，与目标相抵）：每次判法换代整族删 = 全库重判。基线里这一族一把键下的量级：放量那一趟 9 项判完的 891,757,345 态（现判 605,888,874 加复用 285,868,471，基线报告第 84 行）；库里最大一把键 5,658,275,265 态（第 163 行），按四卡约 6.5 万态/秒是约 24 小时（第 183 行，推的）。用户要「争取比上次 耗时和复用上提升」（`records/2026-11-19-判定键只算产品与测试代码.md:103`），R3 让每次改判器付的正是新键想省掉的那一笔。

另：R3 是「由人登记」的换代号，第 436 行把「手写的版本常量」列为「一律不进键、也不许当复用依据」（「手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时」（`.claude/kb/decisions/13-验证路线.md:436`））。整族删不是键，字面不撞；但「什么时候删」全靠人记得，与第 444 行「靠对拍与复用哨兵抓」是两套，第一轮判决「交用户的」那一行该把它写成岔路，不当修补。

### R4 计数向量里不变量名与当前清单不同的块答不复用（修第 4 格 B5 那一支）

改法原文：「计数向量里不变量名与当前清单不同的块答不复用」（`research/prompts/judging-keys-input-only-r1-main-verification.md:55`）。

现查：今天的代码已经在收尾时核这件事。集成副本 crates/singlefs-verification/src/crash_amplification/conclusions_read_at_the_end.rs 第 130 行「let mut expected_names: Vec<&str> = IMPLEMENTED_INVARIANTS.to_vec();」，第 134 行对不上时记「逐条不变量计数的行与池级 checker 的清单不同」；崩溃点重放的判段把这种分歧当错交回（集成副本 crates/singlefs-verification/src/crash_state_enumeration/judging_stage.rs 第 298 行「计数向量与计划对不上 {} 处：{}」）。

构造：

1. 前缀：库里块键 K 下存着加 I-x 之前判的绿结论。
2. 改动：池级 checker 实现了一条新的 I-x（加进 `IMPLEMENTED_INVARIANTS`）。块键不变。
3. 今天：下一趟复用 K 那一块，收尾核计数向量，名单少一条 → 判红；再下一趟照键复用、照红。只有删库能出去（第 4 格「走不出去」的又一形）。
4. R4 下：名单不同 → 不复用 → 整库现判。修得了「走不出去」，代价是每加一条不变量全库重判一次（量级同 R3 构造二）。

新问题：R4 拿「不变量的名字」当复用依据，撞第 436 行「名字…也不许当复用依据」。项目里不变量改名登记在 `.claude/kb/term-renames.md` 一类，改一次名 = 全库重判，判法一个字没变。要修，复用依据得换成名字之外的东西（例如计数向量里只比条数与各条的判到计数，不比名字）。推的、没量。

### R5 观察者标识带修订号，钉表核观察者指纹逼出换号（修第 3 格）

改法原文：「观察者标识带修订号，钉表核观察者指纹（指纹不进键）逼出换号」（`research/prompts/judging-keys-input-only-r1-main-verification.md:56`）。

构造（耗时）：观察者函数改了一行日志或重命名一个局部变量，指纹变 → 钉表红 → 只能换修订号（换号之外的出路是改钉表，那就退回没有 R5）→ 判定项标识变 → 这个观察者挂的每一块的观察者结论重录。基线：放量 9 项录入段约 84,710 秒，「几乎全是观察者逐态录入」（基线报告第 11 行）；崩溃点重放一条大用例 22,440 秒里 22,437.6 秒是观察者录入（第 118 行）。新键省得最多的就是「观察者代码不进键」这一段（基线第 4 问那张表第 2 行），R5 把它原样花回去，只是多了一步人按钮。

R5 的判别子：钉表只看得到「指纹变了」，看不到「判法意思变没变」，与第一轮第 3 格「换号没有能机械判的判据」是同一个病，R5 没碰到那一格，只是把「不换」改成「总换」。

### R6 布局号升到 7；删库次序「停判段 → 删待判行与崩溃判定各表 → GPU 服务照新代码重起」（修第 6 格）

改法原文：「布局号升到 7」（`research/prompts/judging-keys-input-only-r1-main-verification.md:57`）。

现查：布局行读法在集成副本 crates/singlefs-verification/src/verdict_store/library_opening_and_layout.rs，第 7 行「const LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION: u32 = 6;」，第 20 行注释「开库时布局行不在的已经写下了这一版的」：布局行不在，开库的进程写下自己那一版的号。

构造一（删了 `00`）：

1. 照 R6 删崩溃判定各表，`00` 布局表也在「各表」里被删。
2. 两台之一上排着一条从旧树编的任务（某个开工快照或仓副本里交的、编译目录是旧的），它先开库，写下 6。
3. 新代码（7）开库：布局号不对，一律拒在 `LibraryLayoutIsNotThisVersion`。新代码全红，直到再删一次。

构造二（没删 `00`）：布局行还是 6，新代码一开库就拒。R6 的次序里没有「删完由谁写下 7」这一步，也没有「旧二进制已经编不出来了」的保证。

改法（推的、只在我的推理上、被攻过零轮）：删库那一步由 prune-server 客户端在同一帧里删各表并写下新布局行（或删完立即写），且删前先 `--dry-run` 列两台在跑的、持认领的持有者；GPU 服务那一侧见布局行不是自己的号就停服（今天服务扫待判行碰到一行读不出只是整扫返回空，集成副本 gpu-verification-server/src/library_pending_pool/image_judgement_pending_rows.rs 第 17 行那一行提示）。

## 三、问 2：「判器引到的产品代码」取宽取窄（含用户格「产品代码进键」）

用户定案：「崩溃判定的键 = 块的输入数据 + 判器引到的被检测产品代码的指纹」（`records/2026-11-19-判定键只算产品与测试代码.md:99`）。集成副本的收法：六个入口（集成副本 crates/singlefs-verification/src/judging_units_entries.rs 第 1269–1276 行：`crash::judge_crash_image`、`crash_judge_cpu_blocks::judge_a_piece_of_a_block_on_the_cpu`、`crash_given_states::judge_the_given_states_on_the_cpu`、`images_after_the_observers::judge_the_images_after_the_observers_on_the_cpu`、`crash_facts::verify_states_from_facts` 与它按段展开的那一个）一起收一个闭包，只留五个产品包的单元，「一组的键进判器判出的三种结论的版本与对拍结论的键」（同一份第 1268 行）。

量的（`keys-table-ed3b64-4e85.tsv`，`awk -F'\t' '$1=="unit" && $2=="judged_product_code"'` 按路径第二段数）：这一组 1,770 个单元，singlefs-core 1,673、singlefs-format 64、singlefs-replay-prefix 33，mount-runtime 与 background-gpu 0。全部 231 组里够到过的 core 单元合计 3,917 个（各组单元按限定名去重），这一组占 43%。

### 收窄那一向：GPU 路线判的块看不到产品改动

1. 前缀：放量一项全部在 GPU 上判完入库（基线那一趟判段 32,629 秒里 GPU 判 28,498 秒，基线报告第 99–100 行）。
2. 改动：产品 `recover` 改坏一处（例：回绕的记录表上少施加最后一次发布），只在这一项的某几类状态上露出来，对拍那几条固定流（新池新建文件、它的几种坏法、固定脚本到回退）走不到这几类状态。
3. 判器引到的产品代码那一组的键变 → 这一项每一块的结论版本变 → 全部重判。GPU 判器照 WGSL 判：「GPU 判器用 WGSL 另写一份 `replay_journal`，验不到 core」（`.claude/kb/decisions/13-验证路线.md:38`），集成副本 judging_units_entries.rs 第 1267–1268 行同说「GPU 判器的 WGSL 是照 Rust 另写的一份，不在这几个入口的闭包里」。
4. 结局：重判出来的结论与改坏前逐字相同（WGSL 没变），记在新键下，带着「判过新产品代码」的样子。能看到改坏的只有：对拍流（走不到）、每流一块的强制现判（也在 GPU 上判）、CPU 那一路（这一项没走）。是假绿，且花了一整趟 GPU。

四句：分辨臂——分得开「产品代码进键」与「不进键」：不进键时复用旧绿（假绿、零代价），进键时重判出同一个绿（假绿、付全库 GPU）；病根在两臂共用的「GPU 判器不跑产品代码」，所以按「打中不分辨臂」记，另立一笔账。判别子——系统判这一块时手里没有任何东西反映产品改动（WGSL 与它的输入都没变）。满足的字面——D13（验证路线） 第 466 行「GPU 判器是加速路径，它与定义那一组一不一致、判法改了对不对，由对拍（GPU 与 CPU 对拍用例）与复用哨兵抓」（`.claude/kb/decisions/13-验证路线.md:466`）的「由对拍…抓」那一分句：对拍只罩固定的几条流。跑前改法——R1 加大强制比例仍在 GPU 上判，不中；R2 阳性样本走 GPU 也不中；只有「产品代码变了的那几趟强制一批块走 CPU 路线、与 GPU 结论比」才中（改法 G1，见第九节）。

### 收宽那一向

构造一（一组并六个入口，跨种类作废）：产品里只经第 ② 段核对的 CPU 参照（`crash_facts::verify_states_from_facts`）够到的单元，改它会作废镜像判定、给定状态判定与对拍三种结论，虽然它们的判器闭包碰不到这个单元。用户把第 ② 段核对归测试服务代码（`records/2026-11-19-判定键只算产品与测试代码.md` 第 88 行那一条），它够到的产品代码却进了另外三种结论的键。哪些单元只经核对器够到，键表里分不出来（一组一份清单），要量，命令见第十节第 1–6 条。

构造二（同一棵树、两种构建、两把判器键）。量的：`keys-table-ed3b64-4e85.tsv` 里这一组键 `1466657a…`、1,770 个单元、表里没有 `build-feature` 行；`keys-table-ed3b64-7869.tsv` 里键 `07bb31fb…`、1,775 个单元，表末两行 `build-feature	singlefs-core	lookup-work-counters` 与 `build-feature	singlefs-core	test-only-publish-thresholds`。两份清单 `diff` 只差 5 行，全是 `singlefs_core::transaction::lookup_work` 里的（`LOOKUP_WORK_ON_THIS_THREAD`、`LookupWorkOfEachSite`、`LookupWorkOnThisThread` 与两条 `use`）；其余 230 组两份逐字相同。这几个单元是测试计数（集成副本 crates/singlefs-core/src/transaction.rs 第 272 行「只在特性 `lookup-work-counters` 下记（验证包的 dev 依赖打开它）」），不改判定结局。两份表都编自集成副本（构建脚本输出里 `rerun-if-changed` 指向它），我没查到各自是哪一条命令编的（推的：带 dev 依赖特性的是测试目标那一份）。结局：同一块在两种构建下按两把版本键各判各存，撞第 436 行末句「两份库重复判同一批，同样算违反」的意思。另外，不开特性那一份里照样有 `lookup_work` 的 8 个单元（`mod_head`、`LookupSite`、`count` 等）进键：给七·6 测试加一个计数点（`LookupSite` 加一个变体），全部崩溃判定作废。

构造三（录入键收进观察者模块）：录入组从场景入口收、不沿观察者入口 fn 收，但观察者所在模块里别的单元照进。量的：150 个录入组里落在 `tests/common_crash_point_observers` 的单元合计 715 处、`tests/common_compaction_crash_point_observers` 465 处、`tests/common_snapshot_crash_point_observers` 122 处（按组计，同一单元在几组里各算一次）；例 `POOL_CHECKER_CLEAN_AFTER_THE_MOUNT_COUNTER` 在 54 组里、`MountContinuationInputs` 在 30 组里。改观察者模块里这类常量或输入结构，全部这些用例重录，与「观察者的代码不进录入键」（集成副本 judging_units_entries.rs 第 632 行「观察者的代码不进录入键」）不符。

推不出的：产品单元从这六个入口之外进到判定结局（例如经录入那一侧的打包判表）的构造。录入键从场景入口收闭包、经录制管线够到的产品单元照收（集成副本 crates/singlefs-unit-graph/src/entry_groups.rs 第 168 行那一段），我查过的打包判表、事实抽取两处都在录入键里。

## 四、问 3：录入按屏障（用户格「录入按屏障」）

用户原话：「是产品代码的屏障决定键值」（`records/2026-11-19-判定键只算产品与测试代码.md:102`）。

### 「写流变了而键不变」：推不出

查过的路径：

- 崩溃放量：录入输入摘要先拼录下的流的 SHA-256（集成副本 crates/singlefs-verification/src/crash_amplification/recorded_flow.rs 第 1034 行「message.extend_from_slice(&sha256_digest(recorded_flow_bytes));」），复用清单前再核一遍流摘要相同（集成副本 run_from_the_environment.rs 第 898 行「.filter(|manifest| manifest.recorded_flow_digest == sha256_digest(&recorded_flow_bytes))」）。
- 崩溃点重放按块那一路：同样先拼流的摘要（集成副本 crates/singlefs-verification/src/crash_state_enumeration.rs 第 277 行「message.extend_from_slice(&sha256_digest(recorded_bytes));」），录入键 = 录入组的单元键 + 这份摘要（集成副本 crash_library/item_manifests_and_two_stages.rs 第 57–70 行）。
- 场景每趟都真跑一遍才拿得到流：录流加算键加写流与判表约 2.7 秒（基线报告第 118 行，量的、我没复核），所以「键命中就不跑场景」这条路不存在。

写流里任何一次写、一道屏障、一个字节变了，流摘要变、录入键变、块键（段写表与起步镜像摘要）也变。我构造不出写流变了而录入复用的历史。场景代码住在验证包库里（`singlefs_verification::history` 生成随机历史、`crash_injection` 切段，崩溃注入那条用例从它们取，集成副本 tests/crash_injection_campaign/main.rs 第 2–3 行）而不进录入组的单元（量的：150 个录入组里 `crates/singlefs-verification/src/` 下的单元 0 个），改它们照样被流摘要兜住。

### 「键变了而写流不变」：打中（耗时）

量的：150 个录入组每组都有 core 单元，平均 3,202 个、最少 2,568、最多 3,757（`keys-table-ed3b64-4e85.tsv`）。core 里改一处不改写流的地方（重命名局部变量、改一句报错文字、动一个只在读路径上用的函数），这 150 组的录入键几乎全变，每条用例重录；观察者组（80 组，每组平均 1,903 个 core 单元、最多 3,642）同时变，观察者结论重录。流摘要已经罩住「录的是什么」，产品单元键在录入键里多出来的作用只剩一样：事实抽取与打包判表经录制管线调到的产品函数改了而流不变时重抽（这是第一轮攻方 B1 的产品那一半）。这一笔的代价是每次产品改动整段重录（基线放量 9 项录入段约 84,710 秒）。

### 录入检测要钉的判据与会红的检查

判据（推的、被攻过零轮）：一条用例录下的流里，每块盘的整写数与屏障数，等于底层内存盘自己数到的写数与 flush 数按同一条「同一块盘上连着的屏障并成一道」的规矩并完之后的数（D13（验证路线） 已定项 4 末段那条规矩，录制器在 `crates/singlefs-verification/src/lib.rs` 的 `push` 并）。录得全不全不靠键，靠这一条逐盘对数。

会红的检查的写法：录入段写清单之前，取 `SparseBlockDevice`（`crates/singlefs-verification/src/memory_pool.rs`）这一侧独立数的每盘写数、flush 数、FUA 写数，与 `RecordedFlow` 里同一块盘的计数比，不等就拒写清单、整条用例红；配一行变异：录制器丢掉每块盘最后一次写（或吞掉一道屏障），这一格必须红。不并屏障那一读会在两道连着的屏障上假红，所以内存盘那一侧要照同一条规矩并完再比（第一次跑先在一条有连着屏障的流上确认不红）。推的，没实现、没跑。

推翻条件：内存盘那一侧的计数本身经录制器取（两边同源），这条检查就不独立，要改成从设备日志另数。

## 五、问 4：工具链与依赖版本不进键

先照第一节：集成副本今天三种组与录入键都带工具链与依赖行，这一问只能按「照用户这一格做了之后」推。查过的路径：产品五包的依赖（`singlefs-core` 只依赖 `libc`、`singlefs-format`、`singlefs-replay-prefix` 与 `vstd`，`singlefs-background-gpu` 另有 `wgpu`、`ash`）；仓根 `[profile.release]`；仓里有没有钉工具链的文件（`ls rust-toolchain*` 没有）。

构造一（工具链，判的那一侧）：

1. 前缀：两台的 rustc 由多机配置 `CARGO_BIN_DIRECTORY_BY_HOST` 各取各的（集成副本 gpu-verification-server/README.md 第 37 行那一段），仓里不钉版本。库里存着 rustc 1.98.0 下 CPU 判器判过的块。
2. 改动：一台换了 rustc，新版 LLVM 在 `singlefs_core::recovery` 某处的优化上误编译（或修好了一处旧的误编译）：同一份源码，`recover` 在某类崩溃镜像上落到的版本变了。
3. 键：块的输入数据不变；判器引到的产品代码一组的单元指纹是源码指纹，不变；工具链不进键 → 键不变。
4. 结局：块照复用，新工具链下的 `recover` 一次也没在崩溃域上跑过。写那一侧的变化有流摘要兜（写流变了照重录），判那一侧没有。单测与变异的键照旧（D13（验证路线） 第 435 行「加工具链（含 cargo 解析后的生效构建环境与外部工具版本）」（`.claude/kb/decisions/13-验证路线.md:435`）），core 的单测会重跑，但它们罩不到崩溃放量那几十亿态。

构造二（依赖）：`vstd` 钉死版本：「vstd = "=0.0.0-2026-09-20-0158"」（`crates/singlefs-core/Cargo.toml:21`），「平时的 cargo build 用这份 vstd 的 verus! 把 spec 与 proof 擦掉，编出的产品代码与没有 verus! 时相同」（`crates/singlefs-core/Cargo.toml:20`）。升 vstd 只改 `Cargo.toml` 与 `Cargo.lock`，单元指纹在剥掉规格之后的视图上算，不变。新版擦除宏若多擦或少擦一句可执行代码，编出的 `recover` 变了而键不变。这一条的前提（擦除宏出错）没量过，只说明键看不见它。

构造三（变异键）：变异行「套用后编不过记无效」（`scripts/mutation/crates-mutation-rows.py:58`），复用只认抓到（「上次没判成抓到  有这把键的记录，判的是别的档（没红、无效、基线不绿、本次未跑……只有抓到的复用）」（`scripts/mutation/crates-mutation-rows.py:261`））。新 rustc 加了一条默认拒绝的 lint，某一行替换文在新工具链下编不过：按现判该记无效（判红、叫人改替换文），工具链不进键时照键复用旧的抓到。

编译档那一半（用户已定不进键）：仓根 release 档开着 `overflow-checks = true` 与 `debug-assertions = true`（仓根 Cargo.toml 第 28、30 行），与 dev 档一致，今天两档之间推不出判定差别；有人为提速把 release 的 `overflow-checks` 关掉，产品里溢出从 panic 变成回绕、写流跟着变，写那一侧由流摘要兜住，判那一侧（恢复读旧镜像时溢出）键不变、照复用。推的。

推翻条件：仓里加 rust-toolchain 文件钉住版本、两台同一份，且升版走一次全库作废（或键里留工具链），构造一、三撤；构造二要另量擦除宏。

## 六、产生结论的方法（攻击面 ②）

第一轮两条腿与判决都靠读代码推，没量过一个数。下面几格换一种取样或一次实跑就会动：

1. 第一轮第 2 格「绿状态是空向量，漏判前后逐字相同」：只看了判定向量。强制现判的比对是整份结论的摘要，含「每个状态的判定向量、违例（块内序号与判定向量）、计数向量（`crate::verdict_store::count_vector_bytes`）、恢复字与点名的被抛弃根」（集成副本 crates/singlefs-verification/src/crash_amplification/reuse_sentinels.rs 第 200 行），计数向量逐条不变量记判到、违例、不适用（集成副本 crash_judge_conclusions.rs 第 194 行）。所以「判法变弱让一条不变量从判到变成不适用」「恢复落到的版本变了」这两类，绿结论前后不逐字相同，强制现判抽中就红。第 2 格只对「仍算判到、只是不再报违例」那一类成立。一次实跑能分开：在副本里把某条不变量的判定改成恒成立（判到计数不变）与改成恒不适用两臂，各跑一项、强制现判抽中受影响的块，看哪一臂红。
2. 第一轮第 1 格「强制现判只一块」：发现率是块数与改动落点的函数，第一轮没给数。基线 9 项录入块合计 13,791 块（基线报告第 75–83 行逐项相加：1,062 + 1,218 + 2 + 3 + 1,923 + 5,398 + 272 + 2,585 + 1,328），每趟强制 9 块，占 0.065%。判器改动均匀落在 1% 的块上，一趟抓到的概率 1 − 0.99⁹ ≈ 8.6%；落在 0.1% 上约 0.9%；全落在 unlink-under-a-snapshot 那一项的一块里，一趟 1/5,398（算的）。
3. 「新键下复用率」（正文第二节写明没量）：判器引到的产品代码一组占够到的 core 单元 43%，录入组每组平均 3,202 个 core 单元、观察者组平均 1,903 个（量的，单元数，不是改动频率）。按改动频率算的复用率要拿真库里相邻几把键的组成清单比，基线报告「没做什么」第 1 条说本仓没有现成命令；第十节第 7 条给一个替代量法。
4. 第一轮判决里六条改法都「零轮」，但其中 R4 要防的「走不出去」今天已经以收尾核对的形式存在（第二节 R4 现查），第一轮没读到这一段，把它当成空白。

## 七、从未被看过的地方（攻击面 ③）

### GPU 服务一侧补丁

- 结论不记判器版本：「块属性里不记 GPU 判器是哪一版」（集成副本 gpu-verification-server/src/packed_tables_family.rs 第 18 行）。服务每台一个进程（集成副本 gpu-verification-server/README.md 第 34 行「多机配置里点名了卡的每一台各一个进程」）。构造：拿一个改过内核的二进制（内核变体探针那一类）误装上一台、照常服务主库，它判的块与正常的一台判的块在库里一模一样，换回正确的二进制之后挑不出该重判的那些，只能整族删（R3 的代价）。`control install` 会两台一起换版，挡住了「两台各跑一版」的常态，挡不住「装错了一版」。
- 一行读不出整扫返回空（第一轮第 6 格）在服务一侧照旧：集成副本 image_judgement_pending_rows.rs 第 17 行「扫不了待判行（库里有一行不合验证包的布局）」之后 `return Ok(Vec::new())`。R6 没改这一处。

### prune-server 删库路径

- 删前缀分帧、可在两次之间被写进（第二节 R3 构造一）；`delete-by-item-name` 按值挑键、逐条删，同样不原子（`scripts/prune-server/prune_server_client.py` 第 147–154 行那一段用法）。
- 布局行在删库之后由谁写下（第二节 R6）。
- 用户「旧的数据和判定 如果无法复用的就都删掉好了」（`records/2026-11-19-判定键只算产品与测试代码.md:94`）的「无法复用」：新键下判器引到的产品代码一组随构建特性有两把（第三节构造二），按一把删会把另一把下的当「无法复用」删掉，或反过来漏删。

### 变异与广谱变异的键在新定案下

用户定「变异的键 = 被改坏的那一处产品或测试代码 + 点名的测试闭包」、驱动与算键库不进（记录第 105 行）。构造：驱动里判「抓到」的那一段（`verdict_of_run`，`scripts/mutation/crates_mutation_rows_judging.py`）有毛病，把「点名的测试一条都没跑、进程退非 0」（例如测试二进制在搭镜像时撞内存上限）记成抓到；修好它不改任何一行的键，旧的假抓到照复用。能发现它的只有复用哨兵每趟「再从剩下的里抽 1 条强制现判」（`scripts/gate/lib_reuse_checks.py:22`），与崩溃判定第 1 格同形。广谱变异同一套（`scripts/mutation/broad_mutation_run.py` 文件头「复用哨兵」）。这一格两条腿第一轮都没碰。

### 两台多机下的复用

- 工具链：今天的键带工具链行（第一节），两台 rustc 不同时库按台分开、互不复用（耗时）；照用户格去掉之后两台共用，构造一（误编译）在两台之间同样成立。量的：任务服务编译目录里这几天编出的 23 份键表，工具链行都是 rustc 1.98.0（只看得到这些表，不代表第二台的编译）。
- 运行时取路：产品 CRC 按这台处理器有没有 SSE4.2 走硬件或软件那一路（集成副本 crates/singlefs-core/src/checksum.rs 第 78 行附近），两路都在同一个闭包里，键相同；一台录的、判的块另一台照复用，软件那一路在崩溃域上可能从没跑过。推的、两台都有 SSE4.2 时这一格空转。

## 八、问 5：耗时与复用，按基线分段

基线的数照基线报告（标「量的」，我没复核）；「新键下」一列是推的，量级取基线同一行的数。「作废因」写这一段在什么改动之后要重付。

| 段（基线，9 项放量那一趟） | 基线 | 新键下的作废因（集成副本） | 不降反升的来源 | 量级 |
|---|---|---|---|---|
| 录入段（含编译） | 约 84,710 秒（第 97 行） | 录入组：流摘要 + 每组平均 3,202 个 core 单元 + 场景文件 + 工具链 + 锁文件；观察者组：块 + 判定项标识 + 每组平均 1,903 个 core 单元。观察者、录制管线、判器改动不再作废 | R5：每次观察者代码改动整段重录；R1 同一个旋钮罩观察者：每趟 p × 这一段 | R5 每次约 84,710 秒；R1 每趟 p × 84,710 秒（p = 1/16 约 5,300 秒） |
| 等卡 | 约 24,784 秒（第 98 行） | 不随键变，随要判的态数变 | R1、R3 与「产品改动后 GPU 盲判」加的态都排在这里 | 与下一行同比例 |
| 判段（GPU 28,498 秒，卡上 25,779 秒） | 32,629 秒（第 99–101 行） | 判器引到的产品代码一组（1,770 个单元，三种结论与对拍共用一把）；GPU 判器、池级 checker、观察者改动不再作废 | 产品改动：全部 GPU 判的块重判，结论与产品改动无关（第三节）；R3、R4：判法换代、加一条不变量各一次全库重判；R1：每趟 p × 这一段；同一棵树两种构建两把判器键：同一块判两遍 | 全库一把键最大 5,658,275,265 态（第 163 行），约 6.5 万态/秒是约 24 小时（第 183 行）；R1 每趟 p × 32,629 秒 |
| 写库 | 852 秒（第 103 行） | 随现判的块 | R1、R2 多写的结论与强制现判凭据 | 与判段同比例 |
| 崩溃点重放各片编验证包 | 17–24 分钟一片（第 117 行） | 不随键变；构建脚本每次编都算 231 组闭包 | 改 core 任一份文件就重跑构建脚本（`rerun-if-changed` 1,251 行，量的） | 三份编译目录里键表比 `invoked.timestamp` 晚 18、23、29 秒（量的，文件时刻；当成构建脚本用时是推的），相对 17–24 分钟可略 |
| 崩溃点重放一条大用例的观察者录入 | 22,437.6 秒（第 118 行） | 同录入段 | 同录入段 | 同录入段 |
| crates 变异表 | 一行平均约 90 秒（第 125 行）；判法单元指纹换过 53 把，每换一把整表作废（第 200–202 行） | 用户定驱动与算键库不进键（记录第 105 行）：这 53 次作废新键下都不发生 | 驱动判「抓到」的逻辑改了不再作废（第七节），省时的同时少了发现手段 | 一次整表约 7,990 行 × 90 秒 ≈ 72 万行秒（算的，行秒不是挂钟） |
| 广谱变异 | 两片各约 17,700 秒全花在基线（第 11 行）；算键 713.9 秒单进程（第 131 行） | 同上，判法闭包不进键 | 同上 | 三趟 10,186 条全部现判、0 条判出（第 9 行） |

按改动种类合起来（推的）：

- 只改观察者、录制管线、GPU 判器、核对器、测试服务：新键下录入与判段都不重付，这是新键真正省下的（基线事件 1、3、4、5、6、11、12 那一类，基线第 4 问表）。R5、R3 让其中「观察者」「判器」两类照付。
- 只改产品（不改写流）：录入（3,202 个单元的录入组 + 1,903 个单元的观察者组）与判段（1,770 个单元的判器组）都重付，和上一版一样多；其中 GPU 判的那部分重判出的结论与改动无关。这一类的频率决定新键整体省多少，没量。
- 改写流的产品改动：块键本来就变，与键怎么组成无关。
- 每一趟（什么都没改）：R1 的 p、R2 的阳性样本与对拍坏流整跑是稳态开销，上一版没有。

第一趟不省：新键合进去之后旧库要删（记录第 94 行），用户「我已经准备全部重判了」（`records/2026-11-19-判定键只算产品与测试代码.md:101`），省下的从第二趟起才看得到。

## 九、改法与各格

都只在我的推理上过（没有模型、没有副本上的读数），被攻过零轮。每一格都是「推的」：没实现、没跑；交主 agent 的待跑命令在第十节。

| 改法 | 修哪一格 | 在打中的格上 | 标 |
|---|---|---|---|
| G1：判器引到的产品代码一组的键变了的那一趟，从 GPU 判过的块里按比例抽一批强制走 CPU 路线判、与库里的 GPU 结论比（不入库换上，只比） | 第三节收窄那一向 | 中：CPU 那一路调产品 `recover`，看得到改动；没抽中的块照旧看不到，发现率是比例的函数 | 推的，待主 agent 跑 |
| G2：录入键去掉产品单元，只留流摘要、场景标识与录制管线调到的产品函数（事实抽取、打包判表经它们的那一截） | 第四节「键变了而写流不变」 | 中：流不变的产品改动只在动到那一截时重录 | 推的 |
| G3：判器引到的产品代码一组按结论种类拆回三组（镜像判定、给定状态、对拍各一组），收键时不跟 `cfg(feature = "lookup-work-counters")` 下的单元 | 第三节收宽构造一、二 | 中：只经核对器够到的单元不再作废镜像判定；两种构建同一把键 | 推的 |
| G4：删库由 prune-server 客户端在删完的同一步写下新布局行，删前列两台在持认领的持有者，GPU 服务见布局号不是自己的就停服 | R6 两个构造、R3 构造一的一半 | 中布局行那一格；R3 的「删完之后另一台再写」仍要停判段，G4 只让它报得出来 | 推的 |
| G5：R4 改成比计数向量的条数与各条判到计数，不比名字 | R4 新问题 | 中：改名不再作废；加一条不变量照旧整库重判（这一半与 R4 相同，不降） | 推的 |
| G6：仓里加一份钉工具链的文件，两台同一份；升版本当一次显式的全库作废（或照集成副本今天的做法留工具链在键里） | 第五节构造一、三 | 中；与用户格「工具链不进键」冲突，交用户 | 推的 |

## 十、要主 agent 跑的命令

第 1–6 条量「判器引到的产品代码」六个入口各自够到哪些产品单元（第三节收宽构造一要的数）。都在集成副本里起、只编函数单元图一个包、不跑崩溃枚举，不点验证包测试目标，不用范围声明（不是起验证的命令；被范围声明闸拒了照拒绝说明补）。

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo run -p singlefs-unit-graph --release --bin unit-graph -- closure /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree singlefs_verification::crash::judge_crash_image --target lib --no-default-features --cfg 'feature="default"' --cfg 'feature="gpu"' --toolchain-line 'host: x86_64-unknown-linux-gnu' --judgment-kind product-behavior`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree`。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`（`exit=3` 是闭包里有拒算，照样有组成清单，可用）；`task-server/task log <号> --all` 存成 `/home/fy5090/.local/share/singlefs/work/judging-keys-input-only-r2-opus/entry-judge-crash-image.txt`。
2. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo run -p singlefs-unit-graph --release --bin unit-graph -- closure /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree singlefs_verification::crash_judge_cpu_blocks::judge_a_piece_of_a_block_on_the_cpu --target lib --no-default-features --cfg 'feature="default"' --cfg 'feature="gpu"' --toolchain-line 'host: x86_64-unknown-linux-gnu' --judgment-kind product-behavior`
   - 在哪起：同上。看哪几行：同上，日志存成同一目录的 `entry-cpu-blocks.txt`。
3. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo run -p singlefs-unit-graph --release --bin unit-graph -- closure /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree singlefs_verification::crash_given_states::judge_the_given_states_on_the_cpu --target lib --no-default-features --cfg 'feature="default"' --cfg 'feature="gpu"' --toolchain-line 'host: x86_64-unknown-linux-gnu' --judgment-kind product-behavior`
   - 在哪起：同上。看哪几行：同上，存成 `entry-given-states.txt`。
4. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo run -p singlefs-unit-graph --release --bin unit-graph -- closure /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree singlefs_verification::crash_amplification::images_after_the_observers::judge_the_images_after_the_observers_on_the_cpu --target lib --no-default-features --cfg 'feature="default"' --cfg 'feature="gpu"' --toolchain-line 'host: x86_64-unknown-linux-gnu' --judgment-kind product-behavior`
   - 在哪起：同上。看哪几行：同上，存成 `entry-images-after-the-observers.txt`。
5. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo run -p singlefs-unit-graph --release --bin unit-graph -- closure /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree singlefs_verification::crash_facts::verify_states_from_facts --target lib --no-default-features --cfg 'feature="default"' --cfg 'feature="gpu"' --toolchain-line 'host: x86_64-unknown-linux-gnu' --judgment-kind product-behavior`
   - 在哪起：同上。看哪几行：同上，存成 `entry-verify-states-from-facts.txt`。
6. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree -- cargo run -p singlefs-unit-graph --release --bin unit-graph -- closure /home/fy5090/.local/share/singlefs/work/judging-keys-integration/tree singlefs_verification::crash_facts::verify_states_from_facts_by_the_segment_expansions --target lib --no-default-features --cfg 'feature="default"' --cfg 'feature="gpu"' --toolchain-line 'host: x86_64-unknown-linux-gnu' --judgment-kind product-behavior`
   - 在哪起：同上。看哪几行：同上，存成 `entry-verify-states-by-the-segment-expansions.txt`。

六份日志到齐之后（承接的同类腿做，不交任务）：每份照单元行的写法（集成副本 crates/singlefs-unit-graph/src/output.rs 第 2 行：`<种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹>`）取产品单元，`awk -F'\t' 'NF==4 && $3 ~ /^crates\/singlefs-(core|format|replay-prefix|mount-runtime|background-gpu)\//{print $2}' <文件> | sort -u > <文件>.units`；再 `cat entry-judge-crash-image.txt.units entry-cpu-blocks.txt.units | sort -u > image.units`，`comm -23 entry-verify-states-from-facts.txt.units image.units | wc -l` 就是「只经核对器够到、却作废镜像判定」的单元数。六份的并集条数应当等于键表里这一组的 1,770（或 1,775）条，对不上说明我对收法的读法错了，第三节收宽构造一撤。

7. 没给命令：「产品改动多常改到这几组」要拿几棵历史树各算一遍判器组与录入组的产品单元指纹、逐棵比。树要主 agent 用 `scripts/agents/snapshot-tree.py` 照提交倒推出来，同第 1 条的命令把根换成那棵树；我没有可用的历史树路径，不写占位命令。这一个数决定第八节「只改产品」那一类的频率，也就是新键整体省多少。
8. 没给命令：方法第 1 条的两臂（某条不变量改成恒成立、改成恒不适用）与改法 G1，要先在副本里改验证包与跑一项放量，量大、要卡，交主 agent 定要不要做；R2 的代价要对拍测试目标整跑一趟的挂钟，任务日志里该有上一次整跑的结局行，我读不到任务号，没取。

交回前我自己跑的（文本工具，不经服务）：`python3 scripts/agents/cite-check.py research/prompts/judging-keys-input-only-r2-opus-output.md`，末行贴在第十三节。

## 十一、没打中的形状

- 写流变了而录入键不变：查了崩溃放量与崩溃点重放按块那一路两处录入输入摘要（都拼流的 SHA-256）、放量复用清单前的流摘要复核、场景每趟真跑这三处，推不出（第四节）。
- 场景代码住在验证包库里（`history`、`crash_injection`、`memory_pool`）而不进录入组：改它们改写流时被流摘要兜住，推不出假绿。
- 编译档：仓根 release 与 dev 两档的 `overflow-checks`、`debug-assertions` 一致，今天两档之间推不出判定差别；只在有人改档时成立（第五节末段，推的）。
- 产品代码里不稳定排序在工具链换版后改写流：查了 core 与 background-gpu 的 `sort_unstable*` 15 处，按不全序键排的只有 `rollbacks_on_the_parent_chain` 一处（集成副本 crates/singlefs-core/src/transaction/transaction_output.rs 第 558 行），它的等键情形只在盘上读回同一个 T_rb 两条的谱系时出现（同一份第 548 行注释），合法写侧不产生，崩溃枚举域里取不到。
- R1 下「照样发现不了而一块时发现得了」：没构造出来。
- 产品单元经六个入口之外进到判定结局：打包判表、事实抽取两处都在录入键里，没构造出来。

## 十二、这条腿自己的限度

- 没跑任何项目代码、没有模型：每个构造都是按代码推的一段历史，没有一格在副本上红给人看。量的只有两份键表里的单元数与组差（awk 数的，草稿目录里有中间文件 `jpc-4e85.txt`、`jpc-7869.txt`、`groups-4e85.txt`、`groups-7869.txt`）。
- 两份键表各是哪一条构建编出来的，我只从编译目录的位置与有没有 `build-feature` 行推，没核到命令；「带 dev 依赖特性的是测试目标那一份」是推的。
- GPU 服务对「同键结论不同」点名跳过之后待判行是否一直留着，我读到 `library_failure_or_the_reason` 那一层为止，没跟到主循环；R1 构造的第 3 步那一半是推的。
- 第三节收宽构造一（只经核对器够到的产品单元有多少）没数出来，可能是 0；是 0 的话这一构造只剩「三种结论共用一把」的结构问题，没有实际的跨种类作废。
- 第八节的耗时全是基线报告的数乘比例；基线自己说「若…差的单元大多落在产品代码里，第 4 问『新键下这些不再作废』就不成立」，我同样受这一条约束：只改产品的改动多常见，没量。
- R2、R3、R5 的代价判断依赖「这几样会被频繁触发」，触发频率没量。
- 只攻了分到的三面加耗时；第一轮已攻的八格本身成不成立没碰（照派发提示）。

## 十三、没做什么

- 不判辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数（第十节第 1–6 条）没复跑；键表里的数主 agent 还没复核。
- 没建仓副本、没建编译目录、没起样本进程；没有崩溃状态要枚举，第 3b 步的取样与 10⁶ 态上限不适用（这一轮一个崩溃状态都没跑）。
- 草稿目录里只有拷来的两份键表与几份 awk 中间文件（不是编译目录、不是仓副本），留给承接的腿核对。
- 交回前跑的引文核对 `python3 scripts/agents/cite-check.py research/prompts/judging-keys-input-only-r2-opus-output.md`，退 0，末行原样：`✓ 核了 26 处引文、0 处裸行号，对上 26 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）`。集成副本与基线报告里的行写成「集成副本 <路径> 第 N 行」「基线报告第 N 行」，这个脚本不核，写进来之前逐处 `grep -nF` 过。
