# 治理文档核对报告 D（fs-design / format-evolution / implementation-first / implementation-workflow / path-moves）

核对对象：`/home/user/singlefs` 工作区现状（2026-09-26）。只读，未改任何文件。
跑过的轻量阶段与脚本（原样日志在同目录）：`doc-lint.sh`（rc=0，`D-doclint.log`）、`.claude/gate.d/10-kb-rot.sh`（rc=0，`D-10.log`）、共享「编号与简称」阶段 `number-name-sync.sh`（rc=1，红的 5 处都不在本组文件里，`D-nns.log`）。

## 逐条

| # | 文件:行 | 类别 | 原句（整句抄） | 证据 | 建议改法 |
|---|---|---|---|---|---|
| 1 | .claude/rules/fs-design.md:137 | 腐烂（简称与登记位不符） | `落点：[checks-owed.md](../kb/checks-owed.md) C8（范围判定）与 C19（腰线渗透）。` | `checks-owed.md:22` 登记 `\| C8 \| 门禁范围判不出来 \|`；`checks-owed.md:29` 登记 `\| C19 \| 介质判定渗进事务层 \|`。本组文件全部编号引用的比对脚本输出：`fs-design.md:137 C8 写「范围判定」 登记「门禁范围判不出来」 MISMATCH`、`C19 写「腰线渗透」 登记「介质判定渗进事务层」 MISMATCH`。门禁没抓到的原因：`number-name-sync.sh` 第 30 行只收 `experiments/*.md`、`decisions/*.md` 的 E/D 登记位（`for pat, pre in (('.claude/kb/experiments/*.md', 'E'), ('.claude/kb/decisions/*.md', 'D'))`），C/I 编号在 kb 之外没有任何检查；doc-lint 的 G 段只扫 `kb_files` | 改成 `C8（门禁范围判不出来）与 C19（介质判定渗进事务层）`；另见第 11 条（门禁射程缺口） |
| 2 | .claude/rules/fs-design.md:32 | 腐烂（指向错位） | `权威记录在 \`.claude/kb/decisions/05-快照-空间记账机制.md\` D5（快照 / 空间记账机制） 已定项索引表第 8 行。` | D5 已定项索引表第 8 行是 `05-…md:16: \| 8 \| **第一个事务记账树里写哪几行** \| 有值的统计量写一行、没有对象的不写行；第一个事务写 15 行 **状态：已定。** \|`，与快照用量无关。真正的权威记录在已定项 4 的统计量表第 8 行：`05-…md:123: \| 8 \| **已撤回·此编号不再使用** \| 曾定「每树独占字节」。撤回依据：…第一版不提供每快照用量…将来若要做 per-snapshot 配额，它是运行时准入要读的量，当场回到 \`.claude/rules/fs-design.md\`「记账是事务的副产品」第一格…`（第 9 行 124 同一次撤回） | 改成「D5（快照 / 空间记账机制） 已定项 4 统计量表第 8、9 行」 |
| 3 | .claude/rules/path-moves.md:37 | 不一致（与门禁 90 号、豁免表、仓里实际做法相反） | `**历史类文件保留旧名**（\`.claude/kb/decisions-history/\`、\`experiments-history.md\`、\`records/\`），判据与「说搬迁这件事本身的那一行保留旧路径」那一步同形：把旧名换成新名之后那句话还是不是真话——它们断言「那一天文件字面写了什么」，换了就是假话。` | 门禁 90 号头：`# gate-stage: 改过名的术语，全仓不再出现旧名（排除表见 .claude/term-rename-exempt…）`。`.claude/term-rename-exempt` 里历史类只豁免两份单文件，且理由写明历史文件已经换名：`.claude/kb/decisions-history/2026-09.md  # 两处 F2FS 的原题引文…这份变更史别处的旧名已随本次改名一起换掉，所以只豁免这一份、不豁免整个目录`、`records/2026-09-07-D14项2第一轮.md  # …records/ 别处的旧名已随本次改名一起换掉，不豁免整个目录`。现查 `grep -c '超级块' .claude/kb/decisions-history/2026-09.md .claude/kb/experiments-history.md` → `0` / `0`。照 37 行做（历史文件留旧名、不登记豁免）90 号必红；实际做法与门禁一致，规则句与之相反 | 改成：历史类文件同样换名；只有「换了就成假话」的那一句（别家术语、整行抄自产物）留旧名，并逐文件登记进 `.claude/term-rename-exempt`。同时与 29 行「历史类文件里的引用也要改」统一口径 |
| 4 | .claude/rules/path-moves.md:29 | 腐烂（门禁射程说宽） | `\| **编号的简称** \| doc-lint（登记位与全仓每处引用要逐字一致） \| …` | doc-lint 的引用检查只扫 kb：`doc-lint.sh:682  G 查引用带简称且与登记位一致`，所在分支 `if [[ -n "$kb_files" ]]`；kb 之外由「编号与简称」阶段（`number-name-sync.sh`）管，而它只认 D / E（见第 1 条证据）。第 1 条的 C8 / C19 错简称在 doc-lint rc=0、number-name-sync 不报的情况下留在规则里，就是反证 | 改成：kb 内由 doc-lint、kb 外的 D / E 由共享「编号与简称」阶段判；C / I 编号在 kb 之外没有检查，改名时要手工全仓搜 |
| 5 | .claude/rules/implementation-workflow.md:55 | 不一致（与门禁 57 号实际行为不符） | `崩溃验证员跑 54、55、57、59 号那几道；门禁分诊员跑门禁其余阶段并分诊，54、55、57、59 靠「输入没变就复用上一次全绿判定」不重跑。` | `grep -ln 'stage-must-run' .claude/gate.d/*.sh` → 47、54、55、59、74、87，**没有 57**；`.claude/gate.d/stage-inputs.tsv` 只有 54、55、59、74、87 五行，没有 57。57 号（`.claude/scripts/lkmm.sh`）没有任何复用判定，`gate.sh --staged` 里会整道重跑 herd7。同一说法还在 `.claude/agents/gate-triage.md:26`（`54、55、57、59 这几道重阶段在 \`gate.sh\` 里照各自的复用判定走`）与 `.claude/main-agent.md:59`；32 行也写「herd7 读 \`litmus/\` 与代码里的锚点…每一道读哪些路径，以 \`.claude/gate.d/stage-inputs.tsv\` 里它那一行为准」，而那一行不存在 | 二选一：给 57 号接 `stage-must-run.sh` 并在 `stage-inputs.tsv` 登记 `litmus/`、`.claude/scripts/lkmm.sh`、锚点所在的 `crates/`；或把三处文档改成「54、55、59 复用，57 在 gate.sh 里照跑」 |
| 6 | .claude/rules/implementation-workflow.md:52 | 腐烂（本工作区现状） | `\| 每次提交代码 \| **必须跑**：主 agent 在提交流程里后台起（命令带 \`SINGLEFS_HEAVY_TESTS=commit\`），看门狗盯；git 的 pre-commit hook 照旧跑整轮门禁 \|` | `git config core.hooksPath` 无输出；`ls .git/hooks/` 只有 `*.sample`，`ls -la .git/hooks/pre-commit` → `No such file or directory`；`grep -rln 'pre-commit' research/scripts .claude/hooks .claude/scripts .claude/gate.d CLAUDE.md` 只命中 `heavy-test-guard.sh`（只是提到），仓里没有安装它的脚本。未核实：用户本机是否另装了 | 写明 pre-commit 由谁、用哪条命令装（或放一个安装脚本）；装不上的环境里这一句不成立，要写出「没有时提交前手工跑 `gate.sh --staged`」 |
| 7 | .claude/rules/format-evolution.md:26 | 腐烂 / 不清楚（门禁号与形态说错） | `半定 / 待定的决策例外，要按门禁 20 号写「—— 半定（一项未定）」，只许这一种括注。` | 20 号只要求标题里有状态词（`20-kb-shape.sh:126 if not re.search(r'——\s*(已定\|半定\|待定)', title)`），括注是**可选**的：第 ⑤ 段 `m = re.search(r'([0-9]+\|[一二两三四五六七八九十])\s*[项条]未定', title)`，`if not m: continue`——没写括注就不判。「不带别的括注」是 75 号 ⑦ 判的：`75-…sh:226 if not re.match(r'^## D\d+ \S.*? —— (已定\|半定\|待定)(（[0-9一二三四五六七八九十]+[项条]未定）)?\s*$', …)`，括注同样可选。且实际形态是「（N 项未定）」不是只有「一项」：`head -1 .claude/kb/decisions/26-*.md` → `## D26 后台整理与放置回收 —— 半定（三项未定）` | 改成：半定 / 待定的决策在状态后写「（N 项未定）」，N 与「### 未定项」条数一致（20 号 ⑤ 判条数），除此之外不带括注（75 号 ⑦ 判）；并写明现在两道门禁都不强制必须写 |
| 8 | .claude/rules/format-evolution.md:53 | 不清楚（漏了一道门禁） | `**门禁管哪一半**：\`.claude/gate.d/75-decision-experiment-links.sh\` 判表的形状、双向对不对得上、回看过没过期、这次改动该回看的回看了没有。还没回填的实验页与决策记在 \`.claude/decision-links-pending\`，只减不增。…` | 45–51 行「路径与结论登记」一节由 99 号判，不由 75 号：`99-multipath-registry.sh:6 # 判据与形态在 \`.claude/rules/format-evolution.md\`「实验页的「路径与结论登记」」，这一道判三条`；它还有自己的待补清单 `.claude/gate.d/multipath-registry-lag.tsv`（只缩不涨）。本节只点 75 号，读者会以为路径登记归 75 号或没人判 | 在「门禁管哪一半」补一句：路径与结论登记由 99 号判形状、落点存在、共用项不空，未补的页在 `.claude/gate.d/multipath-registry-lag.tsv` |
| 9 | .claude/rules/format-evolution.md:70 | 不清楚（射程说宽） | `改完把**只可能指旧值**的那几个串登记进它 \`format-const\` 标记的 \`stale=\`（\`\|\` 分隔），27 号从此替你盯着。` | 27 号旧值串只扫 kb 正文与 `.rs`：`27-…sh:105-106 scan = [...kb_files] ; scan += [... for f in srcs]`，`srcs = glob('research/**/*.rs') + glob('crates/**/*.rs')`。63–65 行列的派生形态里有「门禁脚本里抄过去的数」，那一类（`.sh`/`.py`）不在 27 号扫描范围；`research/results/` 也显式不扫（`27-…sh:103`） | 写明 27 号只盯 kb 正文与 `research/`、`crates/` 下的 `.rs`；门禁脚本与产物里的旧值仍要第 1 步手工搜 |
| 10 | .claude/rules/fs-design.md:175–177 | 不一致（示例与位号登记表冲突） | `const INCOMPAT_LAYOUT_ROTATIONAL: u64 = 1 << 0;` / `const INCOMPAT_LAYOUT_SSD:        u64 = 1 << 1;` / `const INCOMPAT_LAYOUT_ZONED:      u64 = 1 << 2;` | 位号唯一登记位 D15（格式冻结政策） 已定项 4：`15-…md:78 \| incompat \| 0 \| 第一条纯 SSD 布局线 \|`，`:79 \| incompat \| 1..255 \| 未分配…`；`feature-bits.md` 表：`\| 0 \| incompat \| INCOMPAT_FIRST_SSD_LINE_BIT \|…`，代码 `crates/singlefs-core/src/system_configuration.rs` 的 `INCOMPAT_FIRST_SSD_LINE_BIT = 0x01`。示例把位 0 给了 ROTATIONAL、SSD 放在位 1，常量名也与代码不同；照抄会与登记表冲突 | 示例前加一句「示意，位号与名字以 D15（格式冻结政策） 已定项 4 与 `.claude/kb/feature-bits.md` 为准」，或把示例改成与登记表一致（位 0 = `INCOMPAT_FIRST_SSD_LINE_BIT`，其余写「未分配」） |
| 11 | .claude/rules/fs-design.md:100、214、215、216；implementation-workflow.md:48；path-moves.md:29、30 | 不一致（裸编号，与 CLAUDE.md / 其余规则的写法不同） | fs-design:100 `见 [decisions.md](../kb/decisions.md) D17 与 D13。`；:214 `\| **D12 的介质分层布局** \|…`；:215 `\| **D9 预留的「每个写的算法类型字段」** \|…`；:216 `…（见 [decisions.md](../kb/decisions.md) D14）…`；implementation-workflow:48 `…全部实验复跑（87 号）、E152 装置。`；path-moves:29 `…（那次 17 个：\`C245\`、\`I-7.7\`、\`E126\`……）…`；:30 `…（那次 C232 从 48 涨到 50）…` | 比对脚本输出：`fs-design.md:100 D17 写「」 登记「实现分层与第三方管道」 BARE`、`D13 … 登记「验证路线」 BARE`、`:214 D12 … 登记「目标介质」`、`:215 D9 … 登记「加密」`、`:216 D14 … 登记「双轨（大小文件 / 持久临时）」`、`implementation-workflow.md:48 E152 … 登记「按里程碑对比六家文件系统的文件性能」`、`path-moves.md:29 C245/I-7.7/E126`、`:30 C232` 均 BARE。同一份 fs-design 的 32、79、81 行与 CLAUDE.md（`D7（是否进 Linux 主线）`、`E152（按里程碑对比六家文件系统的文件性能）`）都带简称。门禁不判规则文件里的裸编号（doc-lint G 只扫 kb；number-name-sync 只判「带了括注而写错」，不判裸编号） | 补上简称。若确认规则文件允许裸编号，至少把第 1 条那种写了错简称的改掉 |
| 12 | .claude/rules/path-moves.md:29–30 | 不一致（规则正文里写经过与实测） | `（那次 17 个：\`C245\`、\`I-7.7\`、\`E126\`……）`；`新名比旧名长时可能撞上限（那次 C232 从 48 涨到 50），要当场缩简称并全仓同步` | `rules-discipline.md` 第 1 条「不写：这条怎么来的：实测、日期、谁定的、踩过的坑」。「那次」指哪一次在本文件里没写（推的：`term-renames.md` 里唯一一次「超级块 → 系统配置」）。C232 现登记简称 `系统配置槽宽：类规则给了答案，字段表说未定`，现算显示宽度 42，「50」只在那一天成立 | 删掉两处括注；要例子就写成不带数的形态，经过留在 `decisions-history` 或 `term-renames.md` |
| 13 | .claude/rules/path-moves.md:25 | 不清楚（指代不明） | `它不归本规则的登记表管，落地时同样要一次做完：正文改完之后还有五处会红。` | path-moves.md 全文没有任何「登记表」（`grep -n '登记表' .claude/rules/path-moves.md` 只命中 25 行本身与 57 行指 `term-renames.md` 的那句）。读者分不清「本规则的登记表」是哪张、术语改名到底要不要登记进 `term-renames.md`（57 行说要） | 改成「术语改名登记在 `.claude/kb/term-renames.md`，由门禁 90 号判；落地时正文之外还有五处会红」 |
| 14 | .claude/rules/path-moves.md:10、54 | 不一致（本文件内自相矛盾） | 10 行 `搬迁是低频操作，核验是一次性的，**没有常驻门禁兜着**：每一步都要自己跑出来看。`；54 行 `**搬迁这一半门禁一道都不管**` | 同一文件 31 行自己写 `\| **文件名与指向它的链接** \| 共享 \`gate.sh\` 的「链接指向」阶段 \|`；`link-targets.py` 头「文档里的相对链接与「第 N 节」指向，指不指得到」；门禁 10 号第 4 段「治理文档里的门禁号、路径与「小节」指得到」（本次跑出 `扫 29 份治理文档：门禁号 87 处、路径 259 处`）。搬迁后的 Markdown 链接与治理文档里的路径其实有门禁兜 | 改成「搬迁没有专门的阶段：Markdown 相对链接由「链接指向」、治理文档里的路径由 10 号第 4 段兜一部分；`Cargo.toml`、`replay.sh` 登记表、代码与脚本里的路径没人兜，要照六步自己核」 |
| 15 | .claude/rules/fs-design.md:153 | 不清楚（指错条） | `5. **分支变量不许在操作中途改变。** 见第 1 条（切换判据必须显式、可计算、写下来）。` | 「操作中途变的分支变量」的判据写在「但有四样成本没有被解除」第 1 条（fs-design.md:120–124 `1. **验证是组合的，不是线性的。** … → **只在操作期间不会变的输入上分支。** 设备类型稳定，可以；「当前填充率」会在操作中途越过阈值，那是危险的分支变量。`），不在「五条硬要求」第 1 条（:144 切换判据要显式）。括注把指向钉到了硬要求第 1 条，而那一条没说中途变不变 | 改成「见「但有四样成本没有被解除」第 1 条（验证是组合的，不是线性的）」 |
| 16 | .claude/rules/fs-design.md:57 | 不清楚（引语没出处） | `⚠️ **这与「不许指向核心层的对象」不冲突**：**「是单元」说的是格式形态，` | 这句引语在本文件里没有出处。全仓现查它的来源是 D21（权威态与派生态的分界） 硬约束 5：`21-权威态与派生态的分界.md:357 5. **扩展点里的指向不许指向核心层的对象。** 只能指向该扩展点自己配额内的空间。…`。不点名来源，读者不知道这是扩展点的约束，容易读成对所有空间的约束 | 改成「这与 D21（权威态与派生态的分界） 硬约束 5「扩展点里的指向不许指向核心层的对象」不冲突」 |
| 17 | .claude/rules/fs-design.md:84 | 不清楚（标题承诺的内容正文没有） | `## feature bit 分三档，且预留空位` | 正文只有 86 行三档定义，「预留空位」预留多少、在哪没写。权威在 `.claude/kb/feature-bits.md:3`「三张位图（incompat / compat_ro / compat）各 256 位，住系统配置」与 D15（格式冻结政策） 已定项 4 登记表（`15-…md:79-80` 「未分配」） | 正文补一句指向：「每档 256 位，分配与登记见 D15（格式冻结政策） 已定项 4 与 `.claude/kb/feature-bits.md`」 |
| 18 | .claude/rules/implementation-workflow.md:11 | 不一致（腿的名字与三方规则不一） | `…正推腿核「代码做的是不是条款说的」、反推腿攻「哪一格会错」、本地腿找反例；打中的写回代码，再攻一轮` | `.claude/rules/three-way-inference.md`「各条腿必须互不重复」：一轮三条腿是「云端攻方（Opus）、云端正推或云端辩方（Sonnet，一轮一条）、本地攻方或本地辩方（一轮一条）」；`.claude/main-agent.md` 派发表用 `three-way-forward`/`three-way-defense`、`three-way-attack`、`three-way-local-attack`/`three-way-local-defense`。「反推腿」在两处都没有这个名字；「本地腿找反例」与「本地攻方或本地辩方，派这一轮缺的那一侧」不同（辩方轮不找反例） | 改成按定义名写：`three-way-forward`（或 defense）核代码与条款、`three-way-attack` 攻、`three-way-local-attack` 或 `-defense` 按 three-way-inference.md 取 |
| 19 | .claude/rules/implementation-workflow.md:55 | 不清楚（与分诊员定义读法相反） | `…谁都不把整轮全量从头跑一遍 \|` | `.claude/agents/gate-triage.md:26`「暂存过的跑 \`… bash .claude/scripts/gate.sh --staged\`…；主 agent 明写全量的跑不带参数（前缀照带）」；`.claude/agent-common.md:46`「\`gate-triage\` 只跑 \`gate.sh\` 整轮与 87 号」。不带 `--staged` 的 `gate.sh` 是不是「整轮全量从头」没有定义；再加上第 5 条（57 号无复用），`gate.sh --staged` 本身也会整道重跑 herd7 | 写明「整轮全量从头」指什么（是否指不带 `--staged`、是否指跳过复用），与 gate-triage.md 第 2 步对齐 |

## 核过没问题的

- fs-design.md:4 `.claude/kb/decisions.md` 存在。
- fs-design.md:32 D5（快照 / 空间记账机制） 简称与登记一致；「那两个统计量因此撤回、第一版不提供每快照用量」与 `05-…md:123-124` 相符（只是行号指错，见第 2 条）。
- fs-design.md:79 D19（块指针的结构与宽度预算） 已定项 3：`19-…md:60 #### 已定项 3：定宽，不加密时留位`，逐字相符。
- fs-design.md:81 D9（加密） 已定项 10：`09-…md:227 #### 已定项 10：加密不进第一个可运行版本，只预留格式字段`，相符。
- fs-design.md:86 三档名 `incompat / compat_ro / compat` 与 `feature-bits.md:3` 一致。
- fs-design.md:90 newtype：`crates/singlefs-core/src/address.rs` 有 `DeviceOffsetInBytes`、`SlotNumber`、`CheckpointTxg`、`InstanceGeneration`、`InodeNumber`、`FileOffsetInBytes`，模块文档带 `compile_fail` 例。代码里没有名为「逻辑地址 / 物理地址」的类型（`grep -rn 'Logical\|Physical' crates/*/src \| grep 'struct\|type '` 只命中 `PhysicalBlockSizeInBytes`），规则是约束不是现状陈述，不算腐烂。
- fs-design.md:98 封闭的提交步骤枚举：`crates/singlefs-core/src/transaction.rs:113 pub enum CommitStep<'publish>`。
- fs-design.md:129 指向 `.claude/rules/format-evolution.md` 存在；:137 两笔欠账仍在欠账表（`checks-owed.md:22`、`:29`），状态与「落点」相符（简称错见第 1 条）。
- fs-design.md:214–216 三个实例的判定与 D12（目标介质）、D9（加密）、D14（双轨（大小文件 / 持久临时）） 索引结论一致（`decisions.md` 索引行：D12「按介质开放分支，各占一个 incompat 位」、D14「不做格式级第二轨」）。
- format-evolution.md:18 49 号 `--write` 存在（`49-history-brief.sh:15`）；「两行快查」与 `decisions-history.md:369` 一致。
- format-evolution.md:28 两把尺的取值与 31 号正则一致（`31-…sh:101-105` 是/否/无对象，动/不动/界不定）；D15（格式冻结政策） 已定项 1 原句「三个取值：**动 / 不动 / 界不定（视同动）**；**没写过视同动**」逐字相符；「状态：已定」由 20 号第 3 段判（`20-…sh:159`）；100 字上限由 75 号 ⑦ 判。
- format-evolution.md:32 「无实验」由 75 号 ⑧ 判并在成功行报数（`75-…sh:238`、`:490`）。
- format-evolution.md:39 75 号按分项查依据：`75-…sh:335 … decisions[decision]['basis'].get(item, set())`。
- format-evolution.md:43 `## 回看决策` 与「不涉及决策：理由」由 75 号判（`75-…sh:472-479`）。
- format-evolution.md:51 C49（全称断言只在抽样点验过） 简称一致、仍在欠账表（`checks-owed.md:51`）。
- format-evolution.md:53 `.claude/decision-links-pending` 存在（现在只有注释行，清单为空）。
- format-evolution.md:63–71 27 号只核 `const 名字 = 值` 与 `stale=` 串、`|` 分隔；E145（码 2 自描述头与映射树 key 宽的代价） 简称一致，extent 扇出 148 在它的产物行里（`145-…md:48 … leaf_fanout=148`）；算术 ⌊16253/148⌋=109、7×148=1036 成立。
- implementation-first.md:6 `crates/` 下四个 crate（checker、core、format、harness）；分配器、发布、恢复、checker 都有源码（`allocator.rs`、`transaction.rs`、`recovery.rs`、`singlefs-checker/src/`）。
- implementation-first.md:19 58 号截止日期 `CUTOFF="2026-09-17"`、只看 `research/prompts/_*-body.md` 里有无 `crates/`，相符。
- implementation-workflow.md:10 共享 gate.sh 有「Show me test」阶段（`gate.sh:340-342`）。
- implementation-workflow.md:11 56 号判「这次改动里每个 crates/*/src/…rs 在新写的 main-verification 里被按路径点名」，相符（实际正则 `^crates/[^/]+/src/.*\.rs$` 连子目录也罩，比规则写的宽，不影响执行）。
- implementation-workflow.md:12 `crates/mutations.tsv` 存在；33 号「每个实验二进制都要有变异表」。
- implementation-workflow.md:18 共享 gate.sh「规则纪律（项目本地）」阶段扫 `.claude/agents/*.md`、`.claude/agent-common.md`、`.claude/main-agent.md`（`gate.sh:324-325`）。
- implementation-workflow.md:20 72 号判形式、形态照 56 号（`72-…sh:4-5`）。
- implementation-workflow.md:22 `show-me-test.md`「新增的测试必须先证明它会红」里有「**这条同样管设计论证**」。
- implementation-workflow.md:26 「核查员」= `three-way-verifier`（`agents/three-way-verifier.md:9 # 核查员（three-way-verifier）`）。
- implementation-workflow.md:39 `research/scripts/stage-must-run.sh` 存在，基准 `REUSE_REF="refs/sop/staged-green"`（:34）。
- implementation-workflow.md:44 引的 fs-design「门禁的范围必须可判定」在 fs-design.md:131；10 号第 4 段判「小节」指得到，本次绿。
- implementation-workflow.md:48、56、58 重型清单、各 agent 放行范围与 `heavy-test-guard.sh` 头注释逐类一致（层 0、QEMU 含 vm-bench.sh、herd7 含 lkmm.sh、59、全量 cargo test 与 check.sh、gate.sh 与 gate-staged.sh、87、E152；crash-verifier 拒 vm-bench.sh）；`research/scripts/gate-staged.sh`、`research/scripts/vm-bench.sh`、`.claude/hooks/runner-dispatch-guard.sh` 存在。main-agent.md:14、:20 现已改成指向这一句，不再另列清单。
- implementation-workflow.md:64 lkmm.sh：每条 Never 要同名前缀的 Sometimes 对照组（`lkmm.sh:29`）、缺 herd7 判红（`:304-305 bad "herd7 缺失"`）。
- implementation-workflow.md:72 `SharedStream` 是 `Rc`：`crates/singlefs-harness/src/lib.rs:124 pub struct SharedStream(Rc<RefCell<StreamState>>);`。
- implementation-workflow.md:77 54 号：线程数经 `SINGLEFS_LAYER0_THREADS`、未显式设 1 却只起 1 线程判红（`54-…sh:39-41`、`:166`）；全绿标记要 LAYER0 与 LAYER0B 都 `exhaustive=true`（`:18`）。
- path-moves.md:15 `replay.sh` 登记表第 4 列是 `results/` 下纯文件名（`replay.sh:607-614`）。
- path-moves.md:21 `.claude/kb/layout/` 与 `.claude/kb/milestone/` 各有 `01-first-txn.md`、`02-second-txn.md`。
- path-moves.md:30 宽度上限 48：`doc-lint.sh:683 NAME_MAX=48`。
- path-moves.md:31 共享 gate.sh 有「链接指向」阶段（`gate.sh:269`）。
- path-moves.md:32 `21-decision-items-sync.sh --write` 存在；生成器按宽度截断（`gen-decision-items.py:32-43`）。
- path-moves.md:33 75 号 ⑤「实验页正文改了…这次改动要在那张表里改过至少一行」，与句意相符。
- path-moves.md:35 `.claude/term-rename-exempt` 存在，88 号判整行抄的 `E7RESULT` 行。
- path-moves.md:43、50 12 号判的五个码位与句中逐字一致，射程全仓含冻结证据（`12-…sh:6`）。
- path-moves.md:57 90 号按 `term-renames.md` 登记表经 `sweep-term.py --check` 查全仓。
- 本组全部带简称的编号引用中，除第 1 条两处外都与登记位一致（D5、D9、D15、D19、C49、E145）。

## 没做什么

- 没跑重型测试（54、55、57、59、87、gate.sh 整轮、全量 cargo test），也没跑 20、21、27、31、56、58、72、75、90、99 号本身，只读了它们的判据代码；判据是否在今天的仓上判绿未核实。
- fs-design.md:38「本工程已经在第二、三格豁免过三次」只核了三项在句中列全、`scrub` 在 D6/D13/D26 里有出现；「豁免过」这件事本身是历史陈述，没去决策变更史里逐次对。
- path-moves.md:29「那次 17 个」没核：那是某次改名当天的计数，今天的仓里算不出。
- fs-design.md:23–40 记账三格的设计判断、「持久 / 临时」分轨不成立等是设计推论，不在核对范围。
- 用户本机的 git pre-commit 是否另装（第 6 条）无从核实，只核了本工作区。
- 顺带看到但不属本组的：门禁 90 号头第 5 行写成「「系统配置 → 系统配置」」（改名时把自己的说明也换了，`90-term-renames.sh:5`）；共享「编号与简称」阶段现在红，5 处在 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:2783,2850` 与 `records/2026-09-24-里程碑二收尾调度.md:62,140`。未深入。
