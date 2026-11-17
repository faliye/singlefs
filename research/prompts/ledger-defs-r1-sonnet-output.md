# ledger-defs-r1 正推腿报告（Sonnet）

日期：2026-11-12。立场：正推。从今天的脚本、钩子与服务推出「这几句话照着做会发生什么」，再逐条与改后的句子比。方案按仓里今天的脚本与钩子谈，`crates/` 产品代码不在这一轮里。

## 开头：核了什么、直接跑了什么

- 六份被判文件的 sha256 与开工快照一致（量的：`sha256sum -c research/prompts/ledger-defs-r1-snapshot/judged-sha256.txt`，六行 OK）。
- 读了：六份文件被点名的句子；`scripts/gate/merge-and-verify.sh` 文件头与 `merge_in_commands`；`scripts/gate/apply-writer-patch.py` 文件头；`scripts/hosts/minimal-copy.py` 文件头；`.claude/hooks/run-scope-guard.sh` 与 `lib_run_scope.py`；`.claude/hooks/completion-cleanup-guard.sh`；`.claude/hooks/bash-command-detector.sh` 的 ㉑；`scripts/crash/crash-point-replay.sh` 与 `crash-amplification.sh` 文件头；`.claude/gate.d/crash-point-replay.sh` 文件头；`.claude/gate.d/code-tooling.sh` 的 long-run-split-exempt 格；`task-server/task_server.py` 与 `task_server_lib.py` 里 long-run-split 那几处。
- 直接跑了一次 `minimal-copy.py`（共用约束「写」一节许直接跑）：照主 agent 定义写段第 2 条那一整条命令（`--with-test-inputs --with-gate-tables` 加三条 `--with`），建在自己的草稿目录。读数（量的，输出在草稿目录 `minimal-copy.out`）：退出码 0；合计 123.7 MiB（11154 个文件），阈值 200.0 MiB；副本根是 git 工作树、索引是空的（`git ls-files` 0 行）；门禁表、`scripts/crash/`、`.claude/scripts/preflight.sh`、`.claude/gate.d/crash-point-replay-items.tsv` 都在，`multi-host.env` 不在（照设计不拷）。建完已删（删之前 `du -sh` 为 187M，量的）。
- 没起别的项目代码。要起的写在末尾「要主 agent 跑的命令」。

## 结论一览（细节在各节；判定一览表在末尾）

| 组 | 条 | 判 |
|---|---|---|
| 一 | 1 走一遍一段实现 | 推不出，其中一处不成立：每条命令今天的脚本都认；验证段 1 指给脚本的「段树那一句」与写段第 2 条的建树命令对不上；合进主工作区那一刻的主工作区没人核 |
| 一 | 2 三棵树什么时候不一样 | 不成立：定义只写了冲突那一种 |
| 一 | 3 还有旧句 | 不成立：验证段 1 与验证段 2 先后矛盾，另有数处还照旧次序写（见 1.3） |
| 一 | 4 代码轮快照 | 成立 |
| 一 | 5 段树与完工清理闸 | 推不出：定义没写跨轮留着怎么办 |
| 二 | 1 残句与两条新写法 | 成立（一处措辞留旧词，不叫人交） |
| 二 | 2 照抄过范围闸 | 两条新模板成立；同一句里的变异模板不成立 |
| 二 | 3 旧点得到新点不到 | 有两类 |
| 三 | 1 与钩子逐字对 | 成立，三处边角没写 |
| 三 | 2 会不会读成不想删就交 | 成立（条件有写，禁止句没写，闸不核） |
| 四 | 1 三条与脚本、服务对 | 成立；差的一半只写了用时一侧 |
| 四 | 2 三处包名 | 成立（规则目录下没有旧名；别处留一处旧名） |

## 组一：写段补丁打进专用的一棵树，验证段先验后合

### 1.1 照改后的句子走一遍：两批有依赖的实现员、三份补丁（A、B 在前一批，C 依赖 A）

记号：M 是主工作区，T 是段树，S 是合入后验证拷的快照。每一步交什么、今天的脚本认不认、落在哪棵树上：

| 步 | 定义里的句子 | 今天的脚本或钩子 | 树 | 判 |
|---|---|---|---|---|
| 建 T | `.claude/main-agent.md:47`：「一段实现开工时，主 agent 在自己认领的草稿目录下用 python3 scripts/hosts/minimal-copy.py <仓根> <段树> --with-test-inputs --with-gate-tables --with scripts=<为什么> --with .claude/scripts=<为什么> --with .claude/singlefs-ai-sop=<为什么> 建这一段专用的一棵树」 | `scripts/hosts/minimal-copy.py:17`：「--with <条目>=<为什么> 逐条声明要多拷的仓根条目」；`scripts/hosts/minimal-copy.py:40`：「阈值 MAXIMUM_COPY_BYTES = 200 MiB，推的」；我跑了一次，退 0、123.7 MiB | T（新） | 认 |
| 实现员建副本 | `.claude/agent-common.md:50`：「派发提示给了段树的，<仓根> 填段树」 | `scripts/hosts/minimal-copy.py:103`：「REQUIRED_ITEMS = ("crates", "Cargo.toml")」，T 有这两样 | T → 副本 | 认 |
| A、B 打进 T | `.claude/main-agent.md:47`：「写段的补丁只打进段树，不打主工作区」；命令是 task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <补丁目录> --root <段树> | `scripts/gate/apply-writer-patch.py:82`：「--root 指的不是这份脚本所在的主工作区时，开头在 <副本根>/.prove-red.lock 上试拿排他锁」；`scripts/gate/apply-writer-patch.py:114`：「副本上不查（副本不是那一轮取快照的工作区）。」 | T | 认；开工快照那一道这一步不查 |
| 派 C，副本从 T 建 | `.claude/agents/implementation-writer.md:15`：「派发提示给了段树路径的（.claude/main-agent.md」；后半句写补丁出的是段树此刻到改后（转述） | 副本索引为空（`scripts/hosts/minimal-copy.py:31`：「副本里没有提交、索引是空的」）；基线怎么取不在这一轮句子里 | T | 认 |
| C 打进 T | 同 A、B | 同上 | T | 认 |
| 验证段 1：A、B、C 合入后验证 | `.claude/main-agent.md:57`：命令 merge-and-verify.sh <补丁目录…>（不带 --skip-apply），补丁目录照打进段树的次序全给 | `scripts/gate/merge-and-verify.sh:25`：「快照（不拿锁）：只有这一种跑法，主工作区一个字节不写。」；`.claude/hooks/run-scope-guard.sh:71`：「不带 --item 的 N = 给的补丁目录份数」，三份写「3 份补丁」对得上 | S = M 此刻 + A、B、C | 认 |
| 验证段 1 绿后合进 M | `.claude/main-agent.md:57`：先一条 copy-originals.py 留底，再逐份 apply-writer-patch.py（前一条退 0 才交下一条） | `scripts/gate/merge-and-verify.sh:2535`：「照这里列的次序逐条交，前一条退 0 才交下一条；留底的拷贝在 $logs/originals，合完要撤回照 python3 scripts/agents/copy-originals.py check $logs 列出改过的那几份：」；`scripts/gate/merge-and-verify.sh:3813` 只在 verify 退 0 且不带 --skip-apply 时打 | M | 认 |
| 验证段 2：带 --item 的几条 | `.claude/main-agent.md:58`：「在合进主工作区之前交，--cwd 与命令里的 <根> 都写段树（写段的补丁都已打在它上面）；这一步绿了才照第 1 步打出的那一段命令合进主工作区」 | 这几条入口与门禁都在 T 里（我建的 T 里逐个 ls 到：`.claude/gate.d/crash-point-replay.sh`、`formal-verification.sh`、`crates-mutation-replay.sh`、`scripts/crash/crash-amplification.sh`、`.claude/scripts/preflight.sh`）；范围闸见组二 | T | 认；在 T 里真起得起来待主 agent 跑 |

判：上表每一行的命令今天的脚本与钩子都认。下面三处不成立或推不出。

**1.1 段树那一句与建树命令对不上（不成立）：验证段 1 指给脚本的「段树那一句」与写段第 2 条的建树命令不一致。**
`.claude/main-agent.md:57`：「打补丁核不过（脚本整批停、退 2）的，照它出路里段树那一句办」。
脚本那一句今天打的是 `scripts/gate/merge-and-verify.sh:2332`：「以主工作区此刻建一棵新段树（python3 scripts/hosts/minimal-copy.py <仓根> <新段树> --with-test-inputs）」，没有 `--with-gate-tables` 与三条 `--with`。
`scripts/hosts/minimal-copy.py:15`：「--with-gate-tables 再声明 .claude/gate.d/（要门禁表的副本）」；不带它，新段树里没有 `.claude/gate.d/`，验证段 2 的带 --item 门禁起不来。
照两条合一份读：新段树要按写段第 2 条那一整条建；但验证段 1 把读者指到脚本那一句，那一句是旧形。按「脚本与定义对不上时报定义那一句」，报验证段 1 的这个指向。
推翻条件：脚本那一句改成带 `--with-gate-tables` 与三条 `--with` 的整条，或验证段 1 改成指回写段第 2 条的命令。

**1.1 合进主工作区那一刻没人核主工作区（推不出）：没有句子核「主工作区还是快照那个样子」。**
S 在开跑那一刻拷（`scripts/gate/merge-and-verify.sh:27`：「带工作区里的全部源码与未提交改动」），之后验证段 1 的剩余部分与整个验证段 2（崩溃放量、形式化、变异行的 --item，可以是小时级）才跑完，合进 M 用的是不带 --root 的 apply-writer-patch.py。
`scripts/gate/apply-writer-patch.py:95`：「别的会话同时改到补丁碰的那几处、核不过的，」——这一句后面是先照三方合并重对，合得上就打。所以 M 在 S 之后被别的会话改过、又与补丁不冲突时，合进去的是一个没人验过的组合，脚本不拦。
定义：`.claude/main-agent.md:57`：「前一条退 0 才交下一条」，句里没有合之前核 M。
另一支：合到一半后一份退 2，M 里就留着没验过的前缀。脚本在 `scripts/gate/merge-and-verify.sh:2535` 打了「合完要撤回照 python3 scripts/agents/copy-originals.py check $logs 列出改过的那几份」，定义里没有。`.claude/main-agent.md:51`：「写段的补丁都在段树里、主工作区里没有打了而没验过的补丁，直接回那个会话」只在写段成立，半截合入后就不成立。
推翻条件：定义里有一句合之前核 M 自 S 取快照以来没改到补丁碰的文件，或半截合入的处置。

**1.1 合入后另交的 mutation-tables 一格，定义没叫主 agent 交（推不出）。**
`scripts/gate/merge-and-verify.sh:2550`：「的 mutation-tables 那一条；写段建的段树用完就删」（前半句是打补丁之后另交，原句含引号，这里引到句内）。`scripts/gate/apply-writer-patch.py:75`：「mutation-tables 那一格（.claude/gate.d/code-source-discipline.sh --check mutation-tables）：真打与 --finish-pending 都不在这个进程里起它」。
定义这边：`.claude/main-agent.md:49`：「apply-writer-patch.py 打完报的 code-source-discipline 红，记进调度记录，留到验证段一起看，不为它单派」——脚本不跑这一格，也就不会「报红」；验证段 1、2 里也没有一条叫主 agent 交脚本打出的那一条命令。这一句不在这一轮改的句子里，但它是写段的新流程里要走的那一步。

### 1.2 三棵树什么时候不一样，不一样时哪一步的绿不作数（不成立：定义没写）

| 树 | 是什么 | 什么时候与别的树不同 |
|---|---|---|
| M | 主工作区此刻 | 别的会话改；主 agent 自己写的探索性改动（`.claude/main-agent.md:119`：「主 agent 自己写、直接和用户对话 → 并进同一批的」合入后验证与三方） |
| T | 建 T 那一刻的 M + 已打进的补丁；另含那一刻拷的 `.claude/gate.d/`、`scripts/` 等 | 建 T 之后 M 动了：补丁碰的文件、T 里拷的门禁表与脚本、`crates/` 别处 |
| S | 验证段 1 开跑那一刻的 M（带未提交改动）+ 全部补丁 | 与 T 不同：M 在建 T 之后、开跑之前动过 |
| Mg | 合进去那一刻的 M + 全部补丁（合进时的主工作区） | 与 S 不同：M 在 S 之后（验证段 1 与 2 的整段耗时）动过 |

哪一步的绿不作数：
- 验证段 2 在 T 上绿，只对 T 作数。M 在建 T 之后动过 `crates/`、门禁表、`scripts/` 或入口的，T 上的绿不覆盖 S 与 Mg。主 agent 自己的探索性改动在 S 里、不在 T 里，验证段 2 看不到它。
- 验证段 1 在 S 上绿，只对 S 作数；合进 Mg 之前 M 动过的，不覆盖。
- 定义里写了的只有一种：`.claude/main-agent.md:57`：「脚本整批停、退 2」。脚本自己写明它只在同一段冲突时停：`scripts/gate/merge-and-verify.sh:148`：「会停的是主工作区在段树建好之后又改了补丁改的同一段」。不冲突的漂移不停，也没有定义句子管。
- 建议（不是采纳）：验证段 2 开始前与合进 M 之前各核一次 M 自 T 建成、自 S 取快照以来有没有改到补丁碰的文件与 T 里拷的门禁表和脚本；改了就以 M 此刻重建 T、整批重交验证段 1、2。
推翻条件：能指出定义里管「M 在 T 建成之后又动了」的一句；或证明主工作区在一段实现期间不可能被改（`.claude/main-agent.md:32`：「本机常有别的会话在同一个仓里干活」，与之矛盾）。

### 1.3 同一份里还有没有照旧做法写、与新句子矛盾的句子（不成立）

打过的与没打过的分两条、补丁打进主工作区再验：在五份文件里 `grep -c '没打过\|分两条'` 零命中（量的，主工作区那五份：main-agent、agent-common、implementation-writer、investigator、implementation-workflow）。这两处旧写法已清。仍矛盾的：

1. **验证段 1 与验证段 2 先后矛盾。** `.claude/main-agent.md:57`：「这一条绿了（末行 TASK_SERVER_RESULT 是 exit=0），照它在汇总行之后打出的」；同一行接着是 `.claude/main-agent.md:57`：「那几条命令，先一条 copy-originals.py 留底，再照同一次序逐份 task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <补丁目录> 打进主工作区（前一条退 0 才交下一条），合完照写段第 2 条删段树」。`.claude/main-agent.md:58`：「在合进主工作区之前交，--cwd 与命令里的 <根> 都写段树（写段的补丁都已打在它上面）；这一步绿了才照第 1 步打出的那一段命令合进主工作区」。照验证段 1 做：绿了就合、合完删段树，验证段 2 没有树可用。验证段 2 那一句是后加的、它说的次序才是对的，但验证段 1 没加「第 2 步也绿了」。脚本那一边 `scripts/gate/merge-and-verify.sh:2550`：「写段建的段树用完就删」，也不提验证段 2。
2. **三处「绿了照它打出的命令逐份合进主工作区」没带验证段 2 的前提。** `.claude/main-agent.md:7`、`.claude/main-agent.md:33`、`.claude/main-agent.md:118` 都有这一句（同一句在三行，量的）。第 33 行接着写了 `.claude/main-agent.md:33`：「再由主 agent 经服务交登记的崩溃枚举用例」，读起来是合之后再交带 --item 的那几条，与验证段 2「合之前、在段树上」相反。第 118 行同样没说树与先后。
3. **`.claude/main-agent.md:65`：「合入后验证（验证段第 1 步）绿了之后，主 agent 经服务交」`refused-test-registry.py list`，`.claude/main-agent.md:65`：「（在合入后的主工作区里跑）」。** 先验后合下，验证段 1 绿了的那一刻补丁还不在 M 里，要等验证段 2 绿、合完才在；照字面「绿了之后就交、在主工作区里跑」会在没有补丁的 M 上跑出空表。
4. **`.claude/agents/implementation-writer.md:34`：「补丁打进主工作区之后用的那一条点名这几行」**（新变异行的 crates-mutation-replay 点名命令，`<仓根>` 填主工作区）。验证段 2 是合进之前在段树上交变异行的 --item；这一条写的是合进之后。两种读法都能做，但同一份流程里写了两个次序。
5. **`.claude/main-agent.md:34`：「哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用」** 与段树设计相反：段树就是要留给下一批实现员做基线的。只由写段第 2 条的特例解开，第 11 条没改。
6. **验证段 2 红了落在哪一支，没写。** `.claude/main-agent.md:60`：「第 1 步红的（这一段的补丁还没合进主工作区），修补以段树为基线写」：只列了第 1 步红；验证段 2 红时补丁同样还没合进主工作区，该走这一支，要读者自己推；脚本打的「只重跑红的」整条命令也只有验证段 1 的。
7. 旧句留存，非这一轮改的：`.claude/main-agent.md:48`：「写段不跑：scripts/gate/merge-and-verify.sh、带 --item 的门禁与崩溃放量入口、代码轮三方、验证包那一步、实验装置与实验跑片」，同一行后面又写主 agent 交实现员报告里带 --item 的命令（「这是写的一部分，不算验证」）。这一轮只改了入口名。
推翻条件：1 与 2 里任何一处能被读成「验证段 2 的前提已隐含」（需要主 agent 定）；3 里「合入后的主工作区」若指全部合完之后，则只缺「验证段 2 绿且合完」的明说。

### 1.4 代码轮快照「取在全部合入之后」在先验后合下还成不成立（成立）

`.claude/main-agent.md:59`：「快照取在全部合入之后，攻整段的合入」；`.claude/main-agent.md:33`：「代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮」。次序：验证段 3 在验证段 2 之后，验证段 2 的末句才合进 M（`.claude/main-agent.md:58`），所以代码轮开在合进 M 之后。`.claude/main-agent.md:60`：「已合进主工作区之后才红的（代码轮判决打中这类），修补以主工作区为基线写」与它衔接。
边角：半截合入（1.1 合进主工作区那一刻没人核主工作区，后一支）时不该开代码轮，定义没写，但「全部合入之后」已隐含。
推翻条件：出现任何一条取快照早于合进 M 的流程（例如拿 T 当代码轮的树）。

### 1.5 段树用完就删与完工清理闸：段树跨轮留着时定义叫主 agent 怎么办（推不出：没写）

事实（今天的闸）：
- T 建在主 agent 自己认领的草稿目录下：`.claude/main-agent.md:47`：「在自己认领的草稿目录下用」。T 有 `.git` 目录，闸按仓副本认（我建的 T 里 `ls -a` 有 `.git`，量的）。
- `.claude/hooks/completion-cleanup-guard.sh:12`：「Stop（主 agent）：手里没有在跑的后台任务与子 agent 时才判；有就放行、什么都不打。」所以写段里有子 agent 在跑、验证段里有 task wait 在后台时不拦；主 agent 结束一轮而手里什么都没跑时（`.claude/main-agent.md:35`：「一批任务停下来就停，等用户指令」），T 被列进单子、拦下收工。
- `.claude/hooks/completion-cleanup-guard.sh:93`：「拦过一次、续跑回来（stop_hook_active 为真）还是同一张单（一个都没删）：放行，把单子作 systemMessage 交给用户处置（免得卡死）；单子变了照拦。」
- 交付物声明：`.claude/hooks/completion-cleanup-guard.sh:85`：「交接：子 agent 交给主 agent 接着用的副本（主 agent 照它的报告在里面跑命令）。子 agent 交回、收工时不列；主 agent 结束一轮时照列，用完就删。」；`.claude/hooks/completion-cleanup-guard.sh:86`：「声明了的那一棵里仓副本、库拷贝、旧文件库与库备份不列，编译目录照列（编译目录不能声明成交付物）」。即：对主 agent，「交接」种类不顶用；「补丁」「备份」「报告」种类声明后整棵里的仓副本不列。

定义这边只有两句：`.claude/main-agent.md:47`：「用完就删：这一段的补丁照验证段第 1 步都合进主工作区之后删」；`.claude/main-agent.md:47`：「打的次序记进调度记录」。没有一句写 T 跨轮（用户叫停、会话压缩、等用户指令）留着时怎么办：删了重建（补丁目录还在、次序在调度记录里，能重建），还是按 `.deliverables` 声明留着、该写哪一种。第 11 条的总则（`.claude/main-agent.md:34`）反着写：「哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用」。
建议（不是采纳）：写一句「T 要跨轮留着时，在认领目录的 .deliverables 声明（种类、理由），或删掉、下一轮以 M 此刻重建并按调度记录里的次序重打补丁」。声明哪个种类要主 agent 定：「补丁」种类能让闸放过，但种类名是补丁而内容是整棵仓副本，闸不核。
推翻条件：主 agent 实测一次 T 在无后台任务的 Stop 上不被列（则我读的 `:12`、`:93` 的条件有漏）。

## 组二：崩溃点重放入口带 `--item` 换成两条

### 2.1 六份里还有没有叫人交崩溃点重放入口带 --item、或与两条新写法对不上（成立）

- 六份里「崩溃点重放入口」一词只剩两处，都是描述闸、不是叫人交（量的，对六份逐个 grep）：`.claude/agent-common.md:106`：「带 --item 的门禁与崩溃点重放入口、广谱变异只点名 gate-verdict」（派发闸 ⑲ 一段）；`.claude/agent-common.md:107`：「起验证的命令（崩溃点重放入口、门禁阶段的一道或几格、」（范围声明闸一段）。
  - 107 的说法与钩子相符：范围声明闸今天还判 `scripts/crash/crash-point-replay.sh`（`.claude/hooks/run-scope-guard.sh:66`：「崩溃点重放  scripts/crash/crash-point-replay.sh：带 --item 是 items」）。
  - 106 的措辞是旧词：钩子 ⑲ 只管 `.claude/gate.d/` 那一层（`.claude/hooks/runner-dispatch-guard.sh:176`：「⑲ 门禁阶段（gate_stage_verdict，拆分提案第六十四节 ①」），「崩溃点重放入口」归 ①（`.claude/hooks/runner-dispatch-guard.sh:976` 的 RUN_OBJECT_HEAVY）。这一句不叫人交 --item，只是把两道闸的名单并在一句里；不判不成立，记一笔。
- 两条新模板在 `.claude/agent-common.md:68` 与 `.claude/agents/implementation-writer.md:34` 两处逐字相同（我用同一段子串在两行都命中，量的）：
  - `.claude/agents/implementation-writer.md:34`：「task-server/task submit --cwd <根> -- bash .claude/gate.d/crash-point-replay.sh --check crash-case-replay --crash-cases-only --item crash-case:<用例名> [--item crash-case:<用例名>…] <根>」
  - `.claude/agents/implementation-writer.md:34`：「task-server/task submit --cwd <根> -- bash scripts/crash/crash-amplification.sh --item <分项> [--item <分项>…]」
- 形状与范围写法：门禁阶段带 --check / --item 是 items，N 取格数加项数（`.claude/hooks/run-scope-guard.sh:65`：「带 --check / --item 是 items，N = --check 点名的格（逗号隔的逐个算）与 --item 点名的项，各自去重后相加」）：模板「范围=<项数加一> 项（1 格加点名的 <N> 项）」对得上；崩溃放量 N 是分项数（`.claude/hooks/run-scope-guard.sh:67`：「崩溃放量    scripts/crash/crash-amplification.sh：带 --item 是 items（N = 分项数）」）：模板「范围=点名的 <N> 项」对得上。`--crash-cases-only` 只与 --item crash-case 同给（`.claude/gate.d/crash-point-replay.sh:42`：「--crash-cases-only：只与点名崩溃枚举用例的 --item 同给，日常只判点名的用例、不起整个验证包」），模板两样都带。
- 别处：investigator 的一处 `.claude/agents/investigator.md:32`、implementation-workflow 的一处 `.claude/rules/implementation-workflow.md:63`、main-agent 的 `:33`、`:58`、`:118` 都写成「登记的崩溃枚举用例（门禁 crash-point-replay 按块那一路）、崩溃放量分项」，没有残留旧形（量的，对这五处逐个读）。
推翻条件：六份里能找到一句叫人写 `scripts/crash/crash-point-replay.sh --item …` 的；或两条模板里的 --check / --item 个数与范围写法对不上闸。

### 2.2 写死的命令照抄去交，过不过得了范围闸当天换上的两判

闸里两判的位置：不指定范围没带标识的拒（`.claude/hooks/lib_run_scope.py:580` 的 commit_gate_problem）；指定范围点到会走崩溃枚举的项、没缩域也没写全域理由的拒（`.claude/hooks/lib_run_scope.py:873` 的 crash_domain_problem）。后者只认四种形状（`.claude/hooks/run-scope-guard.sh:49`、`:50` 起的那一段：崩溃点重放入口点名项、cargo test 点名验证包测试目标、证红脚本、变异门禁点名行）。

| 模板（所在句） | 不指定范围那一判 | 崩溃域那一判 | 判 |
|---|---|---|---|
| 门禁 crash-point-replay 带 --check crash-case-replay --crash-cases-only --item crash-case:… | 带了 --check 与 --item，是 items，不是 whole：不触发 | 形状判的是脚本词落在 scripts/crash 的那份入口（`.claude/hooks/lib_run_scope.py:687`：「if name == "crash-point-replay.sh" and project_code_runs.located_in(word, directory, "scripts/crash"):」），`.claude/gate.d/` 下的那一份不落进去；`.claude/hooks/run-scope-guard.sh:59`：「判不到的：门禁 crash-point-replay 点名 crash-case 用例（--item crash-case:<用例>）、崩溃放量入口点名分项、合入后验证、不点名 --test 的整包 cargo test、验证包 src/ 下的内联测试」 | 过 |
| 崩溃放量 crash-amplification.sh --item … | 带 --item，items：不触发 | 同上，写在「判不到的」里 | 过闸；脚本自己另有一道（见下「放量模板的 79」） |
| 变异门禁 crates-mutation-replay.sh --item <项>（同一句里的第三条） | 不触发 | 命中形状 mutation-items（`.claude/hooks/lib_run_scope.py:684`：「if project_code_runs.gate_stage_name(word, directory) == "crates-mutation-replay.sh":」，`.claude/hooks/lib_run_scope.py:686`：「return "mutation-items" if named_option_values(arguments, "--item") and not listing else None」）：点到的变异行的测试目标会走崩溃枚举时，没缩域、为什么一段也没写「全域理由：」就拒 | 点到这类行时不成立 |

- **放量模板的 79（脚本层，不是闸）**：`scripts/crash/crash-amplification.sh:18`：「或估不出就拒：不编、不跑，退 79，说明里给两条出路——带 --crash-expand-up-to <n>」（同文件 `:196` 列出退出码 79）。定义里的放量模板没带缩域，也没写 79 的出路：`.claude/agents/implementation-writer.md:34` 说「命令原样可抄」。五份定义里 `--crash-expand-up-to` 只在 `.claude/main-agent.md:57` 出现一次（写的是合入后验证那一条）。所以照抄放量模板去交，闸放行，脚本可能因点名分项这一趟要现判的态数超 200000 或估不出而退 79；定义没写怎么办。
- **门禁那一条没有上限（门禁层）**：`.claude/gate.d/crash-point-replay.sh` 里 `grep -c '崩溃域'` 为 0（量的）；闸不判它（上表第一行），门禁自己也不估，所以第一条模板无论点多少用例都不被拦。规则里「指定范围的崩溃状态数压在 20 万量级」对这条路径没有机器在管。合入后验证有自己的估算（`scripts/gate/merge-and-verify.sh` 文件头「崩溃域上限」一段），手交的这一条没有。
- **变异模板**：`.claude/agents/implementation-writer.md:34` 的 crates-mutation-replay 两条模板（带 `--item <项>` 与点名新变异行的 `--item '<行名>'`）都落进 mutation-items 形状。点到的行的测试目标是登记的崩溃枚举目标时，今天的闸拒、说明里写缩域变量与全域理由的写法；定义没写这两样。我在 `crates/mutations.tsv` 里找到一行满足「验证包 + --test 点名登记的崩溃枚举目标」：变异名 `Ignore 那一遍恢复不过 oracle`（目标 crash_enumeration_fixed_script_stream，量的）。
- **动到的测试目标那一句（同一份里的 cargo test）**：`.claude/agents/implementation-writer.md:40`：「动到的测试目标（整个目标，不按名字挑；登记的崩溃枚举用例走第 1d 步）」。动到的目标若是登记的崩溃枚举目标，`cargo test -p singlefs-verification --test <目标>` 是形状 cargo-targets，同样要缩域或全域理由；句子没写怎样把登记的用例排出去（今天的入口自己经 --skip 排，`scripts/crash/crash-point-replay.sh:50`：「测试函数那一段 = 每项的测试目标里除登记成 crash-case: 的用例与放量分项之外的全部测试函数。」）。
判：两条新模板过得了闸，成立；放量模板的 79 与变异模板的崩溃域判，是同一句里邻近模板与今天的钩子、脚本对不上，不成立。读到的结论待主 agent 用末尾第一条命令实测（喂四条 JSON）。
推翻条件：末尾探针里 case-1、case-2 退 2，或 case-3 退 0。

### 2.3 没被登记成崩溃枚举用例、也不是放量分项的东西，旧写法点得到、新写法点不到的有没有（有，两类）

旧写法是入口 `scripts/crash/crash-point-replay.sh --item <合表项>`；它的三段（`scripts/crash/crash-point-replay.sh:50`、`:55`、`:57`）是测试函数那一段、崩溃点重放部分、放量部分，另有整趟才跑的几类（`scripts/crash/crash-point-replay.sh:83`：「各是一项 whole-run:<类名>，--item 点得到、--list-items 列出」）。新写法两条各只走其中一段：
1. **测试函数那一段**：合表项的测试目标里没登记成崩溃枚举用例、也不是放量分项的全部测试函数，按条复用地跑。新写法里的门禁 `--crash-cases-only` 明说不起这一趟（`.claude/gate.d/crash-point-replay.sh:44`：「包里全部测试函数按条复用那一趟不起：不算计划、不起 cargo、不记库」），放量入口只跑分项。
2. **整趟才跑的几类**（`scripts/crash/crash-point-replay.sh:441`、`:442`、`:443` 三行：小输入的 GPU 与 CPU 对拍一组、剪枝自证两条、钉红例一类）：既不是登记的崩溃枚举用例，也不是放量分项，新的两条都点不到。
还能点到它们的路：`cargo test` 点名测试目标（形状 target；若该目标登记过崩溃枚举，要缩域或全域理由，见 2.2 动到的测试目标那一句）；主 agent 流程里合入后验证的测试那一步（`scripts/gate/merge-and-verify.sh` 文件头：补丁改到的测试目标全部按条复用地跑）。所以不是没有路，是定义没写这两类挪到哪里去了，也没说旧「合表项」名字作废（`.claude/gate.d/crash-point-replay-items.tsv:9`：「每一项的测试目标不另登记：就是 ② ③ 点名的部分的测试目标」）。
推翻条件：能指出新两条中任何一条的参数点得到这两类；或定义里有一句交代这两类的新去处。

## 组三：交回闸与检出钩子的两句

### 3.1 与钩子今天的行为逐字对不对得上（成立；三处边角没写）

`.claude/agent-common.md:51`（与 `.claude/agent-common.md:104` 同一条出路）：

| 问 | 定义的写法 | 钩子今天的行为 | 对得上 |
|---|---|---|---|
| 认哪几种目录 | `.claude/agent-common.md:51`：「删不掉的编译目录、工作树与仓副本（rm 被权限拒、被别的闸挡），在交回正文里逐个写一行」 | `.claude/hooks/completion-cleanup-guard.sh:452`：「HAND_TO_MAIN_KINDS = (KIND_BUILD, KIND_WORKTREE, KIND_REPOSITORY)」；库拷贝、旧文件库、库备份、迁库遗留与服务编译目录不认（`.claude/hooks/completion-cleanup-guard.sh:90`：「库拷贝、旧文件库、库备份、迁库遗留与服务编译目录不认这种写法；主 agent 结束一轮时照旧列这些副本，由主 agent 删。」）；`.claude/agent-common.md:104` 写了前三样不认，没写后两样（子 agent 不碰服务编译目录，迁库遗留另算）| 对 |
| 只在哪些位置认 | 没写 | `.claude/hooks/completion-cleanup-guard.sh:87`：「（只子 agent，只编译目录、工作树与仓副本三种，只认认领目录与草稿目录里单子上列出的那几处）」；`handed_leftovers` 只拿 `found_in_roots` 判（`.claude/hooks/completion-cleanup-guard.sh:1755`），崩溃放量运行目录里的副本（`.claude/agent-common.md:104` 开头那句也点名它们）不认这一行 | 边角（只认哪些位置）：没写 |
| 路径要不要一字不差 | `.claude/agent-common.md:104`：「路径要与清单上的一字不差，上层目录不替下层」 | 路径经 `os.path.normpath` 后与清单逐个比（`.claude/hooks/completion-cleanup-guard.sh:1699`）；上层写了不替下层（`.claude/hooks/completion-cleanup-guard.sh:1703` 的 `handed_leftovers`，只在弄坏开关 hand-ancestor-accepted 下才认上层）；行首要有 `/`（`.claude/hooks/completion-cleanup-guard.sh:424` 的 `HAND_TO_MAIN_LINE`）。定义写得比钩子严（钩子容忍末尾斜杠），方向安全 | 对 |
| 原因几个字 | `.claude/agent-common.md:51`：「删不掉的原因，至少 8 个字>」（原句是尖括号占位，这里引到句内） | `.claude/hooks/completion-cleanup-guard.sh:423`：「HAND_TO_MAIN_REASON_MINIMUM = 8」；`.claude/hooks/completion-cleanup-guard.sh:1698`：「if len(reason) >= minimum:」，按字符数、去掉首尾空白，标点与英文字母也算一个 | 对 |
| 主 agent 一侧照不照列 | `.claude/agent-common.md:51`：「（全路径照抄闸列的清单），闸认这一行、不再拦这几份；主 agent 结束一轮时照旧列它们，由主 agent 删」 | `.claude/hooks/completion-cleanup-guard.sh:1691`：「if broken("hand-declaration-ignored") or (role == "main" and not broken("hand-exempt-for-main")):」：主 agent 一律返回空表，照旧列 | 对 |
| 看哪段文字 | `.claude/agent-common.md:104`：「子 agent 交回正文里（收工时看已送达的交回正文与收工前的回复）逐个写了一行」 | `hand_to_main_texts`（`.claude/hooks/completion-cleanup-guard.sh:1674` 起）：交回工具那一次看这一次交回的 message，收工看已送达的交回正文加收工前回复，被交回闸拒掉的那次不算 | 对 |

边角（上游交回闸另一种写法）：上游交回闸管 /tmp 一类临时目录，看的是另一种写法。`.claude/agent-common.md:103`：「交回报告里又没逐个写全路径与为什么不删」，上游判的词表是 `.claude/singlefs-ai-sop/scripts/handback-scratch.py:74` 的 EXPLANATION_KEYWORDS（没删、未删、不删、保留、留着、留给……）；新一行「交主 agent 删 …」不含这些词，除非原因里碰巧有。子 agent 把副本留在临时目录里、只写新一行，上游闸仍拦。定义 `.claude/agent-common.md:51` 没分位置。
边角（看门狗只认没删）：看门狗「交回之后副本还在」那一条只认写了没删加全路径的那一种（正则 KEPT_COPY_MENTION 在 `scripts/agents/agent-watch.py:434`），`.claude/main-agent.md:82`：「交回之后副本还在（交回报告写了「没删 <全路径>」而路径还在，主 agent 删）」同样写旧形。新一行写的副本，主 agent 只在自己结束一轮时由完工清理闸列出，看门狗不叫醒。两处不矛盾，但两种写法并存、各有各的读者。

`.claude/agent-common.md:99`（检出钩子新一道 ㉑）逐句对：

| 句 | 钩子 | 对 |
|---|---|---|
| 「子 agent 的一条 Bash 命令跑完，仓里多出不在它写范围里、未跟踪也没被忽略的文件（挂在 PostToolUse 上，不是执行前拒；只记不报、不拦，主 agent 不判）」 | `.claude/hooks/bash-command-detector.sh:5`：「hook-events: PreToolUse:Bash PostToolUse:Bash」；`.claude/settings.json:164` 注册了 PostToolUse（量的）；`.claude/hooks/bash-command-detector.sh:376`：「退 0，stderr 不写。命令跑完才查，不拦、不撤销任何东西。」 | 对 |
| 临时文件三种形态不算 | `.claude/hooks/bash-command-detector.sh:7351` 与 `:7352` 的两条正则：`\..+\.[a-z0-9_]{8}\.(?:replacing\|swapping)` 与 `.+\.tmp\.[0-9]+\.[0-9a-f]{12}`，覆盖 `.<名>.<八位>.replacing`、`.<名>.<八位>.swapping`、`<名>.tmp.<进程号>.<十二位>` | 对 |
| 判不了的照常放行并记一条检出 | `.claude/hooks/bash-command-detector.sh:378`：「判不了的（照常放行并记一条检出）：跑前或跑完 git 起不来（没有 git、超时、不在仓里）、输入里没有认得出的 session_id 与 tool_use_id、快照读不了、」。定义列了三种，钩子六种；定义那一串是举例，不是穷举 | 对（举例） |
| 放行的几种 | `.claude/hooks/bash-command-detector.sh:377`：「放行：范围里的新文件、改名换上的临时文件名、跑前就在的未跟踪文件、被忽略的文件、跑完时已 git add 的、主 agent（输入里没有 agent_type）、没有快照的」，定义只写了「未跟踪也没被忽略」「主 agent 不判」两样 | 对（子集） |

边角（83 行措辞宽）：`.claude/agent-common.md:83`：「其余前台与 run_in_background 一样拒」，㉑ 是「记不拒」，这一句孤立读会把 ㉑ 也读成拒；㉑ 自己那一句（99 行）写清了不是。不矛盾，措辞宽。
推翻条件：钩子里有一处与上面任一格不符（例如 ㉑ 退 2，或「交主 agent 删」行被主 agent 一侧认下）。

### 3.2 「用完就删」与这条出路放一起，子 agent 会不会读成「不想删就写一行交主 agent 删」（成立，有保留）

- 写了的条件：`.claude/agent-common.md:51`：「交回之前删掉自己认领、自己建的那几份」在前，紧接「删不掉的编译目录、工作树与仓副本（rm 被权限拒、被别的闸挡）」才用这一行，括号里给了两种删不掉的情形。照字面读，条件是写在句子里的。
- 没写的：没有「能删的一律删、这一行不代替删」这样的禁止句；「被别的闸挡」没说是哪道闸。
- 闸不核：`.claude/hooks/completion-cleanup-guard.sh:1698` 只比原因长度，不核 rm 失败过没有。`.claude/agent-common.md:104` 那一句描述闸的行为，没有条件，单看它会读成「写了一行就不列」。
- 判：读者不会无意读成「不想删就交」（条件写了）；但一个想省事的子 agent 照 104 行能做成，闸拦不住。把「能删的不许用这一行」写进 51 行，是定义这边唯一能补的。
推翻条件：找到一个子 agent 的交回正文，里面写了这一行而 rm 没失败过，被定义判为合规。

## 组四：长跑拆分规则的「门禁管哪一半」与三处包名

### 4.1 三条与 code-tooling.sh 第 65 行那一格、task_server_lib.py 第 179 行对不对得上（成立；差的一半只写了用时一侧）

- 对服务的那一条：`.claude/rules/long-run-split.md:34`：「统一任务服务判用时：任务（分片的按片）出结局时，服务记的用时超过 7200 秒、又不在 .claude/long-run-split-exempt 里的，给提交它的会话推一回 long-run-split 提醒」；`task-server/task_server_lib.py:179`：「long-run-split 任务（分片的按片）出结局时服务记的用时超过 7200 秒、又不在 .claude/long-run-split-exempt 里（出路指到 .claude/rules/long-run-split.md；只报一回），」。对。判用时取各次尝试记的 seconds、撞上限的从最后一次之后算（`task-server/task_server.py:5811`：「撞了内存上限、照大一档重来的，从最后一次撞上限之后那一次起算（撞上限的那几次和更早的不算；同 run_limit_accumulated）」），与规则判据一致（`.claude/rules/long-run-split.md:12`：「一项一次运行的用时超过 7200 秒，就要拆」后接同一条撞上限的算法）。推给提交它的会话：`task-server/task_server.py:316`：「出结局的任务由 push_outcomes 发结局之前那一次与 wait 的结局回话带出」。
- 规则说「一项」、服务按片或整条判，这一节有没有把差的写出来：有，一半。`.claude/rules/long-run-split.md:34`：「它按片或整条任务的用时判，不拆到任务里的一项」；服务自己也这么写（`task-server/task_server.py:315`：「依据写这是按片或按整条任务的用时判的」），并且从不去日志里取那一项的起止（`settled_run_seconds` 只加各次尝试的 seconds）。交主 agent 的那一条把「按日志里各项的起止判是哪一项超了」交给主 agent：`.claude/rules/long-run-split.md:36`：「还靠主 agent：收到提醒之后按日志里各项的起止判是哪一项超了 7200 秒；那一项跑完拆没拆，看调度记录里「怎么做」要的那一行；豁免表的项名是不是真有这一项」。用时这一侧写全了。
- **没写的另一半（豁免匹配）**：服务不是按「项」查豁免表，是按「这条任务的命令里有没有一个词等于表里的项名」。`task-server/task_server.py:5820`：「表里每行 <项名>  # <理由> 的项名，等于这条命令的某个词（--item 后面的项、--test 后面的测试名、」，`:5832` 到 `:5834` 把整个词与按 `=`、`,` 切开的各段都加进去。后果：一条任务的命令里没点名那一项（提交时不指定范围的整轮分片、不带 --item 的入口），豁免表里写了它也不压提醒。`.claude/rules/long-run-split.md:34` 的「又不在 .claude/long-run-split-exempt 里的」与 `.claude/rules/long-run-split.md:30`：「只有用户点名的项豁免」都没交代这个匹配方式。
- 小处：`task-server/task_server_lib.py:2426`：「任务（分片的按片、估算的按估算子任务）出结局，服务记的用时超过 LONG_RUN_SPLIT_SECONDS」比 179 行与规则多一种单位（估算子任务）；规则与 179 行相符，与 2426 行不全相符。
- 对格的那一条：`.claude/rules/long-run-split.md:35` 的五样（有 #、理由至少 8 个字、理由里有存在的 records 出处且行号不超行数、项名不重复、不是目录也不是门禁文件名）逐样对得上 `.claude/gate.d/code-tooling.sh:65`：「跑过两小时也不拆的豁免表 .claude/long-run-split-exempt 的每一行写对：有 #、理由至少 8 个字、理由里有存在的 records/<文件>.md:<行>（行号不超过文件行数）、项名不重复、不是目录也不是门禁文件名」；「项名在各入口的 --list-items 里列不列得出，它不判」对 `.claude/gate.d/code-tooling.sh:11887`：「不判：项名在不在该项所属入口的 --list-items 里」。对。另：今天表里零行登记（`.claude/long-run-split-exempt` 两行都是注释，量的），格的这一行退 77（`.claude/gate.d/code-tooling.sh:11885`：「表不在、或一行登记行都没有（只有注释与空行）：退 77，这一格无对象可判，不报绿」），规则没提，但不矛盾。
推翻条件：能给出服务按「项」而不是按命令里的词查豁免表的代码；或有一条登记行的项名不在命令里却压住了提醒。

### 4.2 三处包名换了之后，同一份与 .claude/rules/ 下别处还写旧包名、说同一件事的句子有没有（成立）

- 三处：`.claude/main-agent.md:33`：「补丁改到产品 GPU 包的测试包 singlefs-background-gpu-tests 的真卡对拍时交」；`.claude/main-agent.md:57`：「补丁改到的产品 GPU 包的测试包 singlefs-background-gpu-tests 的真卡对拍逐项交门禁」；`.claude/main-agent.md:122`：「product-gpu-real-card-tests 格：产品 GPU 包的测试包 singlefs-background-gpu-tests 带」。与实现对得上：`.claude/gate.d/crash-point-replay.sh:738`：「PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu-tests"」，`crates/singlefs-background-gpu-tests/Cargo.toml:2` 的 `name` 也是它（量的）。
- 同一份里别处：`.claude/main-agent.md:118` 写「产品 GPU 包的真卡对拍」不带包名，`.claude/main-agent.md:72` 写「产品 GPU 包与别的包里的新测试」指产品包本身；都不说同一件事的旧名。
- `.claude/rules/` 下（量的：`grep -rnE 'singlefs-background-gpu([^-]|$)'` 对 `.claude/rules/*.md` 零命中）：新名出现在 `.claude/rules/verification.md:39`、`.claude/rules/implementation-workflow.md:64`、`:88`，没有旧名。
- 规则目录之外、同一件事还写旧名一处：`.claude/skills/crash-test/commit-stages.md:22`：「跑什么=产品 GPU 包 singlefs-background-gpu 的对拍目标 <目标>」，同一行里命令却是 `cargo test --release -p singlefs-background-gpu-tests …`（量的）。这一行不在这一轮六份里，写给主 agent 看。仓根 CLAUDE.md 第 8 行写的 singlefs-background-gpu 指产品包本身，不是旧名。
推翻条件：`.claude/rules/` 下任何一份里能 grep 到说真卡对拍而写 singlefs-background-gpu（不带 -tests）的句子。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一·1 走一遍一段实现 | 推不出（含一处不成立） | 每条命令今天的脚本都认；验证段 1 指给脚本的「段树那一句」打的是不带门禁表与脚本的旧建树命令；合进主工作区那一刻没人核主工作区；合入后另交的 mutation-tables 格定义没叫交 |
| 一·2 三棵树什么时候不一样 | 不成立 | 建段树之后主工作区动了（不冲突）、主 agent 自己的探索性改动，哪一步的绿不作数，定义只写了冲突退 2 那一种 |
| 一·3 还有旧句 | 不成立 | 验证段 1 绿了就合并删段树，验证段 2 要求合之前在段树上跑，先后矛盾；主 agent 定义第 7、33、118、65 行与实现员定义第 34 行、第 11 条总则仍照旧次序写 |
| 一·4 代码轮快照 | 成立 | 验证段 3 在验证段 2 合进之后，快照取在合入之后 |
| 一·5 段树与完工清理闸 | 推不出 | 定义只写合完删；跨轮留着时闸会列、拦一次、第二次放行，定义没写选哪条路 |
| 二·1 残句与两条新写法 | 成立 | 两条模板两处逐字相同；六份里不再有叫人交崩溃点重放入口带 --item 的句子，只剩描述闸的旧词一处 |
| 二·2 照抄过范围闸 | 两条新模板成立；变异模板不成立 | 新模板闸不判；放量模板可能被脚本退 79、门禁那一条没有上限；同句里的变异模板会被崩溃域判拒 |
| 二·3 旧点得到新点不到 | 有两类 | 测试函数那一段（合表项测试目标里没登记的函数）与整趟才跑的三类 |
| 三·1 与钩子今天的行为 | 成立 | 目录种类、路径、8 个字、主 agent 照列、㉑ 的各句都对得上；另有四处边角（只认哪些位置、上游交回闸另一种写法、看门狗只认没删、83 行措辞宽） |
| 三·2 会不会读成不想删就交 | 成立（有保留） | 条件写在句子里；没有禁止句，闸只比长度不核 rm 失败 |
| 四·1 三条与脚本、服务 | 成立（差一半） | 对服务、对格、交主 agent 三条都对得上；规则写了用时按片或整条判，没写豁免按命令里的词匹配 |
| 四·2 三处包名 | 成立 | 三处与门禁常量、包名一致；规则目录下没有旧名；`.claude/skills/crash-test/commit-stages.md:22` 留一处 |

## 要主 agent 跑的命令

不起任何这一条之外的东西；两条都是读数用的，不改仓。

1. 把四条喂给范围声明闸（只跑钩子，不起被喂的命令；检出记录写进草稿目录里的 `detections.tsv`，不写共用的那一份）。
   - 命令：`task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/ledger-defs-r1-sonnet/probe-scope-guard.sh`
   - 在哪个目录起：仓根。
   - 看哪几行：`TASK_SERVER_RESULT` 末行 `exit=0`；`task-server/task log <号>` 里四行 `case-… rc=…` 与各自的 stderr 头三行。预期：`case-1-gate-crash-case rc=0`、`case-2-crash-amplification rc=0`、`case-3-crates-mutation-replay-on-an-enumerating-target rc=2`（stderr 写「崩溃域没缩」）、`case-4-the-old-entry-with-item rc=2`（同）。
   - 与预期不同的含义：case-1 或 case-2 退 2 推翻 2.2 的「新模板过闸」；case-3 退 0 推翻 2.2 的「变异模板被拒」；case-4 退 0 说明旧写法今天也不被崩溃域判拒，2.3 里对旧入口的描述要重核。
2. 段树里起得起来带 --item 的入口（只列，不编、不跑，两条都不要范围声明）。先直接跑建树：`python3 scripts/hosts/minimal-copy.py <仓根> <主 agent 草稿目录>/segment-tree --with-test-inputs --with-gate-tables --with scripts=段树里起带项的入口 --with .claude/scripts=段树里起带项的入口 --with .claude/singlefs-ai-sop=段树里起带项的入口`（我跑过同一条，退 0、123.7 MiB）。再交：
   - `task-server/task submit --cwd <主 agent 草稿目录>/segment-tree -- bash .claude/gate.d/crash-point-replay.sh --list-items <主 agent 草稿目录>/segment-tree`
   - `task-server/task submit --cwd <主 agent 草稿目录>/segment-tree -- bash scripts/crash/crash-amplification.sh --list-items`
   - 在哪个目录起：如命令里的 `--cwd`。看哪几行：`exit=0`；前一条的输出里有 `crash-case:` 开头的项行，数目与在仓根起同一条命令（`--cwd` 与根都写仓根）逐行相同。不同则 1.1 表末格「认」要改。用完经 `rm -r` 删段树前先 `du -sh` 记大小。
3. 复核不了：段树里没有 `multi-host.env`（`scripts/hosts/minimal-copy.py` 文件头：多机配置决不拷进副本），验证段 2 的带 --item 命令在段树里起时读不读得到主工作区那一份多机配置（`scripts/crash/crash-amplification-lib.sh:200`：「if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then printf '%s\n' "$SINGLEFS_MULTI_HOST_CONFIG"; return 0; fi」，没设时取段树自己的 git 目录上一层，即段树根）。`--list-items` 不读它；要看真跑，得在段树里交一条真正点名一项的入口并读日志里的 GPU 配置行。命令写不出（项要主 agent 按一段实现选），写「复核不了」。同类句子在 `.claude/rules/implementation-workflow.md:51`：「树里起命令要的多机配置照 scripts/hosts/minimal-copy.py:25 那一句交」，写在快照树那一段，段树这条路没写。

## 没做什么

- 不判别的腿的格（攻方腿的「找做错的具体走法」我没单独找，只在读到矛盾时记）；不替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数没复跑（命令在上面）。
- 没跑任何脚本或钩子的自证、样本档、弄坏开关；脚本与钩子当证据读，没判它们写得对不对。
- 没判六份里别的会话的改动，只读了被点名的句子与同一份里与之相邻的句子；`.claude/main-agent.md:48` 与 `:49` 两句不是这一轮改的，写在 1.3 与 1.1 里只因为它们与新流程直接相接。
- 没读禁读清单里的文件（另一条腿的报告、名字相近的另一轮）。
- 没起验证段真跑：没有实测 T 里带 --item 的门禁与入口能不能真跑完，也没量主工作区在一段实现期间被改的频率。
- 引文：`.claude/` 下规则与定义的行号、脚本行号都是这一次 `grep` 与 `frag.py`（逐句在行里找）取的；kb 条款这一轮没引。
- 草稿目录 `~/.local/share/singlefs/work/ledger-defs-r1-sonnet` 里没有仓副本与编译目录（段树副本已删，删前 187M）；留着的是探针脚本与四份钩子输入 JSON，供上面第一条命令用，别的文件是我读材料用的小脚本与输出。
