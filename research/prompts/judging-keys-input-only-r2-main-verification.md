# judging-keys-input-only-r2：判决（2026-11-20）

正文 `research/prompts/_judging-keys-input-only-r2-body.md`，开工快照 `research/prompts/judging-keys-input-only-r2-snapshot/`。这是这一题的第二轮（设计推论轮）；第一轮判决 `research/prompts/judging-keys-input-only-r1-main-verification.md`（两腿都打中，七格挂起，六条零轮改法交这一轮攻）。这一轮辩方腿与攻方腿都打中，照 `.claude/rules/three-way-inference.md`「开不开下一轮（设计推论轮）」开第三轮（最终轮）。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/judging-keys-input-only-r2-sonnet-output.md` | 七格大多站得住，格 1「只剩」说窄了、格 2「没有东西看得见」够不着、格 7 拆三支；六条改法里只有布局号那一条修得了 |
| 攻方（Opus） | `research/prompts/judging-keys-input-only-r2-opus-output.md` | 六条改法与用户后定的三格都打中：「GPU 判器不跑产品代码…产品改动让放量里 GPU 判的块全部重判，判出来的结论与产品改动无关」；工具链与依赖不进键「没有用户原话出处」 |

核查：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/judging-keys-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py judging-keys-input-only-r2 --out ~/.local/share/singlefs/work/judging-keys-r1-verdict/judging-keys-input-only-r2-tree --repository /home/fy5090/code/singlefs`：type=succeeded exit=0，「清单点名 1 份，倒推出 1 份，sha256 对上 1 份」（取工作区此刻那一份）。
- `python3 scripts/agents/cite-check.py research/prompts/judging-keys-input-only-r2-opus-output.md research/prompts/judging-keys-input-only-r2-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_judging-keys-input-only-r2-background.md`：「✓ 核了 62 处引文、0 处裸行号，对上 62 处，没判 0 处」。
- 攻方腿的单元数主 agent 复算（拷进它草稿目录的集成副本键表 `keys-table-ed3b64-4e85.tsv.gz`，`zcat … | awk -F'\t'` 按第 5 列路径分）：判器引到的产品代码那一组 1,770 个单元（singlefs-core 1,673、singlefs-format 64、singlefs-replay-prefix 33）；录入组里落在观察者模块的单元按组计 `common_crash_point_observers` 715、`common_compaction_crash_point_observers` 465、`common_snapshot_crash_point_observers` 122；键表带 10 行 `toolchain`（rustc 1.98.0、LLVM 22.1.8、cargo 1.98.0 等）。与腿报的数逐个相同。
- 主 agent 现读坐实：
  - GPU 判器不跑产品恢复：`.claude/kb/decisions/13-验证路线.md` 里「GPU 判器用 WGSL 另写一份」命中 1 处；集成副本 `crates/singlefs-verification/src/judging_units_entries.rs` 第 1267–1268 行「GPU 判器的 WGSL 是照 Rust 另写的一份，不在这几个入口的闭包里」。
  - 布局行不在时开库的进程写下自己那一版：集成副本 `crates/singlefs-verification/src/verdict_store/library_opening_and_layout.rs` 第 20 行「开库时布局行不在的已经写下了这一版的」，第 7 行布局号仍是 6。
  - 删前缀分帧：`scripts/prune-server/prune_server_client.py:142`「一帧至多 DELETE_PREFIX_KEYS_PER_FRAME 把，客户端接着来到前缀下没有更多的键」。
  - 观察者代码已不进键：集成副本观察者那几组只留产品包，补丁报告 `~/.local/share/singlefs/work/judging-keys-input-data-only/patch/result.md` 第 17 行「观察者版本 = 判定项标识 + 这一组的键 + 另读的输入…观察者自己的代码不进」；组名里的 `observer_code` 是旧名。
  - 工具链与依赖版本：用户的三次弹窗只问 crates 变异、广谱变异与实验变异表 `mutate.sh` 三族的键（`records/2026-11-20-拆分带出的门禁与变异红.md` 第 248 行起那一段）；崩溃判定与录入键没有用户定案，第一轮判决「二、判决」第三行那一句不成立，已在 `records/2026-11-19-判定键只算产品与测试代码.md` 第 112、114 行更正。

## 二、判决

两条腿都打中，开第三轮。第一轮挂起的七格经这一轮复核后的样子：

| 第一轮的格 | 这一轮判 |
|---|---|
| 1 判器改了谁发现 | 站得住，范围缩一句：第 ③ 段逐块比对每趟都比，缺的是复用的旧块那一半（辩方格 1）。发现率是块数的函数：9 项共 13,791 块、每趟强制 9 块（攻方「方法」一格，算的） |
| 2 漏判类 | 缩格：「逐字相同」只对判到计数与恢复字都不变的那一类成立（攻方「方法」）；114 条不变量各有一张坏镜像、变异表点名 WGSL 604 行与池级 checker 948 行（辩方格 2，数是腿读的，主 agent 没复数） |
| 3 观察者 | 站得住 |
| 4 判法换代 | 站得住；今天「增删不变量」的结局是每趟收尾都红、走不出去（两腿同） |
| 5 枚举与状态物化 | 缩格：只改一份实现会被互相对拍抓到，共用辅助一起改才逃得掉（辩方格 5） |
| 6 删旧库 | 站得住；「`02`、`03` 删不掉」站不住：拒的只是写批里的删除记号，`delete-prefix` 不看表码（辩方格 6） |
| 7 字面冲突 | 拆三支：判定项标识撞名字与路径站得住；定值版本撞手写版本常量站不住（版本是种类名接产品指纹的摘要）；测试服务代码撞单测键只在 kb 字面上成立（辩方格 7） |

第一轮六条零轮改法，这一轮都打中，不采：

| 改法 | 打中在哪 |
|---|---|
| 强制现判按比例 | 修得了「发现」，修不了「换代」：判器改了绿结论内容之后，抽中的每一块都存不进（同键结论不同）；没改动的趟也稳定多付约 p ×（84,710 + 32,629）秒（攻方 R1，推的） |
| 阳性样本集 | I-1.13 由 GPU 内核不置位、I-8.4 没有单镜像违例，照字面走不出去；「不入库」与 GPU 服务只认库里待判行冲突；与对拍里每条不变量一条坏流重叠（攻方 R2，辩方改法 2） |
| 人登记整族删 | 删前缀分帧、另一台在判的片删完照写；块属性不再记判器版本，只能整族删、全库重判（攻方 R3） |
| 不变量名单不同不复用 | 拿名字当复用依据撞已定项 17「名字…不许当复用依据」；换成整库重判（攻方 R4） |
| 观察者修订号 | 任何观察者代码改动都逼着换号、整段重录，把新键省得最多的那一段花回去（攻方 R5） |
| 布局号升 7 | 布局号要升，次序要补「谁写下 7」：布局行不在时第一个开库的进程写下自己的号，旧二进制先开库就写下 6（攻方 R6，主 agent 现查坐实） |

用户后定的格，这一轮新打中的（挂起，交第三轮）：

1. 产品代码进键：GPU 判器跑 WGSL 另写的恢复，不跑产品代码；产品一改，GPU 判的块全部重判，判出的结论与产品改动无关，只有对拍的固定流与 CPU 那一路看得到（攻方问 2 收窄一向）。
2. 产品代码那一组收宽：六个入口并成一组，三种结论与对拍共用一把键，只经第 ② 段核对器够到的产品单元也作废镜像判定；同一棵树两种构建算出两把键，差的 5 个单元全是 `singlefs_core::transaction::lookup_work` 里只给测试计数的（攻方问 2 收宽一向，量的）。
3. 录入键：写流变了而键不变推不出（录入键拼了录下的流的摘要）；反方向打中耗时：每组平均带 3,202 个 core 单元，流不变的产品改动照样重录，观察者模块的单元进了录入键（攻方问 3，主 agent 复算观察者模块那三个数）。「录入检测」在 `crates/` 与 `scripts/` 里还零命中，没有落成检查（辩方定案乙）。
4. 工具链与依赖版本：崩溃判定与录入键里没有用户定案；集成副本照 D13（验证路线） 已定项 17 第一条留着。攻方构造的假绿（rustc 或 LLVM 误编译、vstd 擦除改动）是在「假设照做」上推的，照今天的集成副本不成立。变异三族去掉工具链之后，替换文在新工具链下该记无效却照复用抓到（攻方「变异与广谱变异」一格），归 mainagent(B) 那一件。
5. 两台：GPU 服务结论不记判器版本，装错的一台判的块事后认不出（攻方「两台」一格）。

## 三、改法

这一轮腿提的改法（攻方 G1–G6、辩方提的 F4）都只在各自的推理上过，标零轮，交第三轮一并攻；都落在验证包、`scripts/` 与 kb，不写进产品代码，不做 A/B。

| 改法 | 修哪一格 | 出处 | 标 |
|---|---|---|---|
| 判器引到的产品代码一组的键变了的那一趟，从 GPU 判过的块里按比例抽一批强制走 CPU 路线判、与库里的 GPU 结论比 | 用户格 1 | 攻方 G1 | 零轮 |
| 录入键去掉产品单元，只留流摘要、场景标识与录制管线调到的产品函数 | 用户格 3 | 攻方 G2 | 零轮 |
| 判器引到的产品代码一组按结论种类拆回三组，收键时不跟只给测试计数的 cfg 单元 | 用户格 2 | 攻方 G3 | 零轮 |
| 删库由 prune-server 客户端在删完的同一步写下新布局行，删前列两台持认领的持有者，GPU 服务见布局号不是自己的就停服 | 第一轮格 6、改法 6 | 攻方 G4 | 零轮 |
| 计数向量只比条数与各条判到计数，不比名字 | 第一轮改法 4 的新问题 | 攻方 G5 | 零轮 |
| 仓里钉一份工具链文件，两台同一份 | 用户格 4、5 | 攻方 G6 | 零轮，要用户定 |
| 录入命中时仍现算事实表与判表、与清单里的摘要比 | 第一轮 B1 | 第一轮攻方 F4，第二轮辩方建议第三轮看 | 零轮 |

## 交用户的

第三轮攻完再交。到时写进 `research/prompts/judging-keys-input-only-r1-forks.md` 的岔路：判器改了之后靠哪一样发现、换代怎么作废；产品代码进键对 GPU 路线怎么办；崩溃判定与录入键里工具链与依赖版本留不留。

## 这一轮被判过的文件

- .claude/kb/decisions/13-验证路线.md：已定项 17 的新定案加用户后定的三格（产品代码进键、录入按屏障、编译档线程不进键），判第一轮六条改法修不修得了挂起的格、新定案在 GPU 路线、录入键、工具链、两台下的假绿与耗时；七格复核后的样子与新打中的五格见「二、判决」。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 备料 | 2026-11-20 不受影响：第二轮打中的先挂起，第三轮之后才改正文 |
