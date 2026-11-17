# gpu-tests-src-backfill-code-r1：判决（2026-11-17）

正文 `research/prompts/_gpu-tests-src-backfill-code-r1-body.md`，开工快照 `research/prompts/gpu-tests-src-backfill-code-r1-snapshot/`。这是这一题的第一轮，补的是产品 GPU 包的测试包 `src` 四份在途改动没走过的三方；射程是 E202（GPU与CPU两种模式GC对照组） 装置搬迁之外的改动加两张入口登记表与 `lib.rs`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推 | `research/prompts/gpu-tests-src-backfill-code-r1-sonnet-output.md` | 取卡、开卡表、先录后判、两张登记表对得上；「全部卡臂」与跑前登记对不上（登记没随用户定案修订）；接口改名不全是纯改名；搬迁之后变异行与旧路径引用都干净；开跑自检的摘要不含装置自己；新单测 30 条里 3 条没有变异行；检查单「类型与名字」不成立 |
| 攻方 | `research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md` | 打中续跑时卡换了张数没变照样接着跑、没交卡那一趟之后产物续不了、搬迁之后摘要不含装置自己，弱打中 stage-three-cost 一张卡没交时退 0；取卡之前没有臂用卡、开卡表、先录后判、判定行进 `--product` 没打中；123 行变异不罩全；检查单「所有任务多机可用」不成立 |

核查：

- 写判决之前 `sha256sum -c research/prompts/gpu-tests-src-backfill-code-r1-snapshot/crates-sha256.txt research/prompts/gpu-tests-src-backfill-code-r1-snapshot/kb-sha256.txt` 全部 OK，没倒推快照树。
- `python3 scripts/agents/cite-check.py research/prompts/gpu-tests-src-backfill-code-r1-opus-output.md research/prompts/gpu-tests-src-backfill-code-r1-sonnet-output.md --root . --background research/prompts/_gpu-tests-src-backfill-code-r1-body.md` 末行「✓ 核了 25 处引文、69 处裸行号，对上 25 处，没判 69 处（其中裸行号没有引文可比 69 处；逐处列在上面）」。
- 主 agent 现读的行：`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:4133-4137`、`:5933-5937`、`:5954-5968`，`research/prompts/e202-preregistration.md:49`。
- 复跑的读数（任务 18249，攻方副本里三条探针，退 101 在期望内，`test result: FAILED. 0 passed; 3 failed`）：`ATTACK_PROBE other_cards_same_count repeats_done=1 numbers_read_back=1 next_repeat=1` 与 `ATTACK_PROBE other_cards_same_count stopped=None`；`ATTACK_PROBE zero_card_then_resume stopped=Some("cards_named_in_the_product=none named_by_in_the_product=none cards_named_now=2 named_by_now=every_handed_over_card reason=产物里有结果行而没有every_card_arm_cards行，认不出那几次全部卡臂开了几张，换一个--product")`；`ATTACK_PROBE crates_digest files=61 of_the_apparatus_package=0`。探针断的是装置自己注释与登记写下的性质，红就是打中；这是副本上的读数，只用来坐实这三处，不进 kb。
- 没核的：正推腿交的 6 条命令没交（装置的整个测试目标、对拍用例、变异静态核、clippy、并行下一次性库与真卡用例的冲突），留到改完之后那一轮；stage-three-cost 一张卡没交时退 0 那一条要服务把别台的卡交过来才走得到，没量；「交来先后不是 PCI 升序且 `--cards N` 小于交来张数时，点名一张卡臂的卡可能不在全部卡臂的 N 张里」是正推腿推的，没量。

## 二、判决

1. **取卡只用服务交的卡**：两条腿都没打中（卡只来自环境变量或领卡通道，只用 CPU 的活都在取卡之前，没卡的出路退 78、77），记「没打中」。
2. **全部卡臂点名服务交的卡、续跑核对**：
   - 打中：续跑只比张数与开法（主 agent 现读 `:5954-5968`，只取 `cards_named` 与 `named_by` 两个字段），交来的卡换了而张数没变照样接着跑（任务 18249）。两组卡的重复会混进同一份产物一起判；检查单「所有任务多机可用」因此不成立（两台的卡不同，在一台起、另一台续就是这一形）。
   - 打中：注释 `:5934` 写一张都没交的那一趟「以后在交了卡的那一台续跑时不被它挡住」，实际那一趟写下开头几行之后续跑被拒（任务 18249）。注释与代码对不上。
   - 打中：与跑前登记对不上。登记 `research/prompts/e202-preregistration.md:49` 写「全部卡」是「不给 `background_gpu_devices`」，代码点名交来的卡，`--cards` 与 `name=every_card_arm_cards` 行登记里没有（正推腿；主 agent 现读了登记第 49 行）。用户改过这条臂的开法，登记没跟着修订。
3. **开卡表整个进程一份**：两条腿都没打中，记「没打中」。
4. **跟核心层接口改名**：新名在核心层都查得到、旧名零命中；正推腿指它不全是纯改名（枚举两个成员变三个、特征少一个方法多两个带缺省的方法），那是核心层那一侧的改动、装置跟着适配，核心层的改动不在这一轮被判的文件里。
5. **先录后判、经判定库**：两条腿都没打中（录、键、判、存放都走验证包那一份库）。正推腿另报一条并行下的冲突（装置二进制里用一次性库的用例与开真库的真卡用例之间没有守卫），推的、没量，并进改法。
6. **搬迁**：变异行的包名、目标与锚点都对（正推腿逐行数的），旧路径没有活引用。打中：开跑自检的 crates 摘要只走产品 GPU 包与核心层两个 `src/`（主 agent 现读 `:4133-4137`），装置搬进测试包之后它自己与经 `#[path]` 引进来的两份共用模块都不在摘要里（任务 18249：`of_the_apparatus_package=0`），登记的停机条款与同版本核对看不见装置自己的改动。两条腿一致。
7. **两张入口登记表与 `lib.rs`**：39 个入口各恰好命中一处，`singlefs_background_gpu_tests::` 前缀只用来找包（正推腿读了单元图的入口解析），对得上。打中：参照一侧那张登记表没有测试逐组核、没有变异行（两条腿一致），登记错一组入口不会有任何地方红。
8. **判定行追加进 `--product` 文件、续跑读回**：两条腿都判不算另开判定库，依据都是 `.claude/rules/verification.md`「判定库只有一份」里不进库的那一类（运行记录、产物）；记「没打中」。它是实验产物，装置 `admission: always`、每次现量。
9. **新加的单测与变异**：新单测 30 条，3 条没有变异行；取卡链与三条子命令的接线没有单测；123 行变异不罩全第 1 到 5 件的判红分支（攻方报告第八节逐条列）。打中。
10. **检查单八格**：「所有任务多机可用」不成立（第 2 条第一处）；「类型与名字」正推判不成立（新单测里 `LOCKSTEP_ARMS[下标]` 取臂四处，攻方数到这一次新加的三处在 `:12651`、`:12654`、`:12656`），算打中；其余六格两条腿都判成立。
11. **事务员线索**：第 3、6、7、10、11 条攻方判为真，但都在 HEAD 旧路径那一份已有的代码里，不在这一轮射程内，登记欠账；钟速单测前提不成立时返回那一条同样是旧代码。
12. **事实表写错的**：把新加的回调参数当成「多传开卡表」；把枚举换名写成纯改名；漏了新测试里三处位置取臂。

## 三、改法

管 E202 的会话回话说装置它没动过、这几处由这一轮派实现员改，跑前登记的修订同归这一批。

1. 续跑核对把卡的身份记进产物、续跑时逐张比（`name=every_card_arm_cards` 行加卡的标识，点名一张卡那一臂与并发跑同样记、同样核），对不上停、退 2。**零轮**（攻方提的），装置不是产品代码，A/B 由攻方那条探针顶：改前 `stopped=None`、改后要停。
2. 一张都没交的那一趟：照注释改代码（那一趟不往产物里写会挡住续跑的行，或续跑认得出「上一趟一张没交」而放行）。**零轮**，探针同上。
3. crates 摘要把装置自己与它 `#[path]` 引进来的共用模块算进去。**量过**（探针 `of_the_apparatus_package=0`），改后不为 0，并配一行变异。
4. 跑前登记第 49、297 行与第十二节：照用户定案（`records/` 里的原话，正推腿点了行号）补一条修订，写明「全部卡」臂现在的开法、`--cards` 与续跑核对。登记是冻结证据之外的在用文件，修订另起一节、原文不改。
5. 参照一侧登记表补一条逐组核入口的测试与变异行；3 条没有变异行的新单测各挂一行；攻方第八节列的没有变异罩着的判红分支逐处补；`LOCKSTEP_ARMS[下标]` 那几处改成按名字取臂。
6. 并行下一次性库用例与真卡用例的冲突：先写一条能复现的测试，复现得出再加守卫，复现不出写明。
7. 第 11 条那几处旧代码里的线索登记欠账，不在这一批改。

## 交用户的

没有要用户定的岔路：打中的几处要么是装置自己的注释与登记写下的性质没做到，要么是登记没随用户已有的定案修订。

## 这一轮被判过的文件

- crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs：判了搬迁之外的改动（取卡、全部卡臂与续跑核对、开卡表、接口改名、先录后判、新单测）与搬迁带来的摘要射程，打中续跑不核卡的身份、没交卡那一趟之后续不了、摘要不含装置自己、与跑前登记对不上、变异不罩全，照第三节改。
- crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs：判了 GPU 一侧入口登记表的每个入口在源码里找不找得到、前缀怎么解，对得上。
- crates/singlefs-background-gpu-tests/src/reference_units_entries.rs：判了参照一侧入口登记表，入口都找得到；打中它没有逐组核的测试与变异行，照第三节第 5 条补。
- crates/singlefs-background-gpu-tests/src/lib.rs：判了它只有四行、库本身不放入口而登记表靠前缀找包的写法，对得上。

## 回看决策

不涉及决策：这一轮判的是实验装置与它的登记表，没有拿它的结论去推翻或确立哪一条决策分项。
