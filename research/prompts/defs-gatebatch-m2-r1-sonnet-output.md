# 门禁批（m2）第一轮三方 — 云端正推（Sonnet）报告：J5（兼核 J1 字面）

## 一、J5：崩溃验证员定义与相关文字

### 1.1 六份被判文字各自的字面（逐句 grep -nF 核过，行号为各文件自己的）

- `.claude/agents/crash-verifier.md:3`（description）：「崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放与登记的崩溃枚举用例（逐条按输入复用）、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。」
- `crash-verifier.md:22`（新增输入行）：「54 号 `--full` 在哪棵 worktree 里跑：HEAD + 暂存区的 worktree 路径，或写明由你照 54 号快档出路里那三行建。只核标记、不跑全量的，写明「54 号不带 --full」；要丢掉续跑的进度文件从头跑的，写明「--start-over」。」
- `crash-verifier.md:29`（做什么第 2 步，改后）：「……54 号默认带 `--full`、在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑……它逐条跑 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；派发提示写明「54 号不带 --full」时才只跑快档（逐条核全绿标记），写明「--start-over」时加它……」
- `crash-verifier.md:31`（第 4 步）：「……54 号 `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。……」
- `crash-verifier.md:37`（写范围）：「报告文件、草稿目录，与为 54 号 `--full` 建的临时 worktree（跑完删）。阶段自己用的临时目录、编译产物……与 54 号写进 git common-dir 的全绿标记、续跑的进度文件是阶段本身的行为；你不改仓里任何文件。」
- `crash-verifier.md:46`（没做什么）：「全绿只说明这几道的证据要求满足；层 0 覆盖到哪几条流、跑了哪几条崩溃枚举用例、哪些没进来，看 54 号头部与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，不由你外推。」
- `.claude/rules/implementation-workflow.md:58`（新增重型测试判据）：「崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标，libtest 参数带 `--ignored` 或 `--include-ignored` 的……不带这两个参数、只跑那个测试目标里快用例的不算。」
- `implementation-workflow.md:89`（门禁管哪一半，改后）：「崩溃点重放由门禁 54 号判：整轮门禁跑快档，再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`）；全量……逐条按输入复用、只重跑输入变了的；……」
- `.claude/gate.d/stage-inputs.tsv:13-16`（新增注释段，逐字）：「崩溃枚举用例（键是 crash-case:<名>）：门禁 54 号逐条按复用判定跑、逐条记全绿标记（在 git common-dir，按用例与它的输入指纹分格），输入没变复用、变了才重跑。路径用排除法写：只写整个 crates/ 与 Cargo 清单、锁，不按用例手列它读哪些文件——admission.py 算输入时自动减去别的测试目标独占的测试文件……」
- `stage-inputs.tsv:32-35`：四条 `crash-case:` 行（`layer0-first-stream`、`layer0-second-stream`、`floor-raise-pushed-by-the-session`、`c561-sigma-full`），各自 `test=` 恰一条、`count-line=` / `exhaustive=` / `threads=` 齐。
- `.claude/gate.d/stage-owners.tsv:37`（改后）：「54-layer0-replay.sh	crash-verifier	层 0 崩溃点重放与 stage-inputs.tsv 里登记的崩溃枚举用例（crash-case: 行，逐条按输入复用、只重跑输入变了的）：提交时在 HEAD + 暂存区的 worktree 里跑 --full」
- `.claude/main-agent.md:61`（本批未改，原样）：「……就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（整条经内存包装，照 `crash-verifier` 定义第 1b 步；层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀……」
- `.claude/agents/gate-triage.md:29`（本批未改，原样节选）：「……红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动……碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」……」

**这六份／六处彼此的字面判定**：`crash-verifier.md` 的 description、输入项、做什么第 2/4 步、写范围、没做什么六处一致地改成了「逐条 crash-case、逐条标记、逐条复用」；`implementation-workflow.md` 的重型测试判据与「门禁管哪一半」两处同样改成逐条；`stage-inputs.tsv`/`stage-owners.tsv` 的新增内容与这五处对得上。`main-agent.md:61` 与 `gate-triage.md:29` 未改，但两处原文用的是「判绿按输入哈希写全绿标记」「那一格的哈希」这类不点明「一格」是全批还是逐用例的措辞，与改后的逐条方案不冲突，只是没有显式跟着更新（详见 1.3）。**这六份被判文字之间没有互相矛盾**，也没有任何一处仍写着「整批一格标记」的字样（`grep -rn "整批.*一格\|按整批输入哈希"` 六份文件全部 0 命中，命令与结果见下）。

```
$ grep -rn "整批.*一格\|按整批输入哈希" .claude/agents/crash-verifier.md .claude/rules/implementation-workflow.md .claude/gate.d/stage-inputs.tsv .claude/gate.d/stage-owners.tsv .claude/main-agent.md .claude/agents/gate-triage.md
（无输出，0 命中）
```

### 1.2 核心发现：六份被判文字与 `m2-layer0-scale-r3-main-verification.md` 的判决字面矛盾

背景材料第三节「条款」点名要抄 `research/prompts/m2-layer0-scale-r2-main-verification.md` 第二、三节与 `-r3-main-verification.md` 第二至四节，这两份文件因此在我的射程内，不是我自己引进来的材料。

- `research/prompts/m2-layer0-scale-r3-main-verification.md:25`（整行抄）：「**乙（按流复用）**：第二轮打中，这一轮没人替它辩，按两轮里一轮打穿、另一轮没攻计，维持挂起、不采纳；54 号照旧按整批输入哈希记一格全绿标记。」
- 对应的「改前」文字（本轮 diff 附录二 `_defs-gatebatch-m2-r1-diff.md:2551`，`-` 起首、即门禁批改动之前 `implementation-workflow.md` 的原文）：「**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再按这批输入的内容哈希核那一格层 0 全量的全绿标记（两条流都是 `exhaustive=true` 才算）；……」——这句「按这批输入的内容哈希核那一格……全绿标记」正是 r3 判决要求维持的「整批输入哈希记一格全绿标记」那个方案的字面。
- 门禁批把这一句改成了现在的 `implementation-workflow.md:89`（1.1 已引）：「……再逐条崩溃枚举用例……按它这批输入的指纹核那一格全绿标记……逐条按输入复用、只重跑输入变了的……」——这正是 r3 判决明确写「维持挂起、不采纳」的那个「乙（按流复用）」方案（`crash-case:` 逐条登记输入、逐条记全绿标记，`stage-inputs.tsv:13-16` 的排除法写法，与 r3 之前那一轮 `m2-layer0-scale-r2-main-verification.md:29`「今天 54 号按整批输入哈希记一格全绿标记的做法照留……另把……补进 54 号的输入指纹……(推的，被攻过零轮)」里描述的「按流复用」是同一件事）。

**时间序**（文件 mtime）：
- `m2-layer0-scale-r3-main-verification.md` mtime 2026-09-26；r3 判决正文自称「用户 2026-09-26 弹窗定」。
- `records/2026-09-24-里程碑二收尾调度.md:190`「崩溃枚举的跑法」条目自称「用户 2026-09-26」，晚于 r3 约 3 小时。该条目定案第 ③ 点：「以后崩溃枚举一律按用例 / 按流各自登记输入、各自复用，改了哪块只重跑读它的那几条——就是层 0 规模第二轮挂起的乙；第二轮打中……门禁批照当时的修法做：清单用排除法写……」——**只点名「第二轮」，字面没有一处提到第三轮，也没有写「推翻」「覆盖」「不采纳那一条不算」这类话**。
- 门禁批的交回与本轮材料 mtime 在 2026-09-26，晚于上面两者，落地了「乙（按流复用）」。

**判定**：这是一处字面矛盾，不是我的读法问题——两句话说的是同一个对象（54 号的全绿标记方案）、同一天，却互相否定。`m2-layer0-scale-r3-main-verification.md:25` 今天仍原样立在仓里，字面上断言「54 号照旧按整批输入哈希记一格全绿标记」；而 `crash-verifier.md`、`implementation-workflow.md`、`stage-inputs.tsv`、`stage-owners.tsv`（六份被判文字之四）现在写的、门禁批已经落地的，是相反的方案。**这正是 J5 问句「有没有哪一处仍按『整批一格标记』写」的答案：六份被判文字本身没有一处这样写（1.1 已核），但 r3 判决书面上仍这样写，而没有任何文字回去改它或标注它已被后来的用户决定推翻。**

不是说门禁批一定错——`records/2026-09-24-里程碑二收尾调度.md:180` 第十步素材第 ⑭ 条另一处提到「用户定：接进崩溃验证员、按用例各自复用、改了只跑改了的部分」，可见这条方案确实被主 agent 记成是用户直接定的，用户有权推翻一个三方轮的谨慎结论（`.claude/singlefs-ai-sop/rules/pushback-discipline.md`「对方坚持，就照做，但要留下记录」）。**但记录本身要留痕**：`kb-discipline.md`「矛盾比空白更糟：同一个事实只许有一处权威记录，别处一律链过去」——今天的状态是两处权威记录互相矛盾，没有一处指向另一处，读 r3 判决的人会得到一个已经不成立的现状描述。

**什么现象会推翻这一条**：在 `records/2026-09-24-里程碑二收尾调度.md:190` 或 `m2-layer0-scale-r3-main-verification.md` 里找到一句明写「乙的挂起被那条用户决定推翻／覆盖，不再维持」；找到了就说明这条矛盾已被显式承认，只是没写进我读的这几份文字，我没找到：`grep -c "推翻\|覆盖\|不再维持\|不再作数" research/prompts/m2-layer0-scale-r3-main-verification.md` 为 0（r3 判决整份文件都没有这几个字）；`records/2026-09-24-里程碑二收尾调度.md` 全文有 14 处命中，但逐一核过（`grep -n` 同一模式），没有一处落在第 190 行「崩溃枚举的跑法」那一条本身——那一条自己（`awk NR==190` 单独核）0 命中，14 处命中的都是别的日期条目（例如第 155、165 行明写「推翻」的是别的决策，与乙（按流复用）无关），可复核。

### 1.3 main-agent.md:61 / gate-triage.md:29 是否与新方案矛盾（次要，与 1.2 分开判）

两处都是「门禁批没改的」原文（body 第 15 行明写），字面上没有改成逐条口径，但也没有写死「一格」指整批：`main-agent.md:61`「判绿按输入哈希写全绿标记」与 `gate-triage.md:29`「那一格的哈希不同」都只说「一格」「哈希」，没有说这一格是四条用例共享一个，还是逐条各自一个。**这两处的字面在「按用例逐条」与「整批合一」两种读法下都读得通，不是矛盾，是没有随着这一批改动显式更新（`.claude/singlefs-ai-sop/rules/kb-discipline.md`「撤回一个数或一条结论，同样要当场回扫谁在引它」这条纪律没有被完全兑现，但字面本身不构成互相打脸）。** 什么现象会推翻「不矛盾」这个判断：如果 `main-agent.md:61` 或 `gate-triage.md:29` 未来被人依这句话的字面写出「只有一格、一个哈希」的代码或判断（例如分诊时把四条用例的红/绿合并成一个结论），那就是矛盾显形了；目前没有这种代码依赖这两句话的字面去分辨「一格」的粒度，我读了 `.claude/scripts/gate.sh` 与 `stage-must-run.sh` 都是按 `stage-inputs.tsv` 逐行处理，不依赖这两句英文提示的字面粒度。

### 1.4 「改了只跑改了的部分」字面兑现没有

`records/2026-09-24-里程碑二收尾调度.md:190` 用户原话：「这次跑可以 下次肯定要接入提交时崩溃验证员， 并且以后跑也不能全量这么跑，改了只跑改了的部分。」

字面对照：`crash-verifier.md:29`「它逐条跑 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑」——四条用例（`layer0-first-stream`、`layer0-second-stream`、`floor-raise-pushed-by-the-session`、`c561-sigma-full`）各自一个指纹、各自一个标记（`stage-inputs.tsv:32-35`），没改的那条复用、改了的那条才跑。**这句话的字面在六份被判文字里兑现了**：「只跑改了的部分」被翻译成「逐条按用例复用判定跑」，粒度是「用例／流」，不是「整批」也不是「单个文件」。但兑现到哪个粒度、`admission.py` 的排除法算得对不对（某条用例的输入变了却被算成无关而漏跑，或反过来把无关改动也算进去多跑），是 J2 的射程，不是我这一格判的字面层。

**什么现象会推翻这句「字面兑现了」**：找到一条用户原话要求的更细粒度（比如「同一条用例内部按段」），而六份文字里没有一处支持这么细；目前用户原话只说「改了只跑改了的部分」，没有比「用例／流」更细的字面要求，六份文字的粒度与它对得上。

### 1.5 「照改后的字面，崩溃验证员在提交时做不做得对」

按 `.claude/main-agent.md:61` 与 `crash-verifier.md:17-29` 的字面走一遍提交时的调用链：主 agent 暂存 → 派 `crash-verifier`（带 `SINGLEFS_HEAVY_TESTS=commit`）→ 崩溃验证员在 HEAD + 暂存区 worktree 里跑 `54-layer0-replay.sh --full`（默认带，`crash-verifier.md:29`）→ 54 号逐条核 `stage-inputs.tsv` 的四条 `crash-case:` 用例的指纹与标记 → 没变的复用、变了的在 release 下重跑并写新标记 → 崩溃验证员抄「✓/✗」与计数行、末句判绿/复用/判红数（`crash-verifier.md:31`）→ 建的 worktree 用完删（`crash-verifier.md:37`）。**这条链每一步都在六份被判文字里有对应的字面指令，链条本身不断**——即字面上照做能跑通、能得到「逐条判定」的结果，不会漏掉一步或指向不存在的命令。

**但「做得对」还要看两处它自己控制不了的前提**：① `admission.py` 的排除法算出的每条用例输入是否真的只包含改了才该重跑的文件（J2 的射程，我不判）；② 54 号 `--full` 内部的续跑、`--start-over`、进度目录键法是否真的不会假复用或串用（J1 的语义射程，Opus 的攻方腿判，我只核了字面，见下 2.2）。字面层面我没找到「跟着做会走不通」的一步。

## 二、兼核 J1 的字面（不做语义攻击，那是本轮 Opus 的射程）

### 2.1 J1 问句字面对照

`54-layer0-replay.sh` 头部注释（现查行号）：
- 逐条跑、逐条标记：`54-layer0-replay.sh:2`「……逐条核 stage-inputs.tsv 里 crash-case: 那几条用例各自那一格全绿标记与它这批输入的指纹相等……那一格全绿标记在就复用，不在才在 release 下跑它、判绿写那一格」——字面是逐条，不是整批合一。
- 续跑目录跨输入串用：脚本内 `layer0_progress_root/<输入指纹>/`（现查行 314：`续跑的进度文件在 ${layer0_progress_root}/<输入指纹>/`）——目录名带指纹，字面上不同输入落在不同目录，不是同一个目录被跨输入复用。
- 跑了一部分被打断留下作数的标记：头部注释第 16 行「一条判红删它这批输入那一格（先绿后红，前一趟那一格不再作数），接着跑下一条；『跑的过程中输入变了』那一支判红不删」——字面是「标记只在判绿时写」（`crash-verifier.md:31`「判绿写那一格」同一句式），被打断（进程被杀、没跑到判定那一步）不会走到「写标记」这一句，字面上不会留下作数的假标记；但**进度文件（续跑用，不是全绿标记）在被打断时会不会残留、残留的是否安全，是运行语义，我没有跑变异或构造真实中断去核，这属于 J1 攻方腿的射程**。
- `--start-over`：`54-layer0-replay.sh:50-61` 逐行看，`--start-over` 只与 `--full` 同用（第 60-61 行显式拒绝单独用），字面与 `crash-verifier.md:22`「要丢掉续跑的进度文件从头跑的，写明『--start-over』」对得上；`SINGLEFS_LAYER0_START_OVER` 只在 `--start-over` 设时导出为 1、否则从环境里 `unset`（第 201-202 行 `start_over_setting=(-u SINGLEFS_LAYER0_START_OVER)`），字面上没有漏清的迹象。

### 2.2 我没做、留给 Opus 的部分

以上只是「字面自洽」核对：注释怎么写、变量怎么传、条件分支的字面顺序。**J1 真正要判的是「有没有一条造得出来的路径让这些字面失效」**（假复用、被打断后仍作数、进度目录串用、`--start-over` 没传到），这需要构造真实的改动、真实跑一次或至少在临时拷贝上模拟中断/续跑，那是攻方腿的射程，我没有跑任何变异或中断实验去验证 `layer0_progress.rs` 内部的续跑判法（`54-layer0-replay.sh:19`「续跑的判法（片方案、校验和、观察者计数、判红删进度文件）在 crates/singlefs-harness/src/layer0_progress.rs」），我也没有读那份 Rust 源码。

## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| J5·六份被判文字互相一致吗 | 一致 | `crash-verifier.md`、`implementation-workflow.md`、`stage-inputs.tsv`、`stage-owners.tsv` 全部改成逐条 `crash-case:` 口径，六处未见「整批一格标记」字样 |
| J5·有没有哪一处仍按「整批一格标记」写 | **冲突（外部）**：六份被判文字里没有，但 `research/prompts/m2-layer0-scale-r3-main-verification.md:25` 仍原样写着「54 号照旧按整批输入哈希记一格全绿标记」，是这一批已改动之后仍成立的判决书面表述与今天的代码互相矛盾 | 门禁批落地的方案正是 r3 判决「维持挂起、不采纳」的那个「乙（按流复用）」；两处文字都在，没有一处互相指认 |
| J5·main-agent.md:61 / gate-triage.md:29 与新方案 | 不矛盾但未更新 | 两处「一格」「哈希」措辞在两种粒度读法下都通，没有代码依赖它们的字面粒度分辨对错 |
| J5·「改了只跑改了的部分」字面兑现没有 | 兑现了（在「用例／流」粒度上） | 四条 `crash-case:` 用例各自复用判定，粒度与用户原话对得上；粒度算得准不准是 J2 的射程 |
| J5·崩溃验证员在提交时做不做得对（字面） | 字面链条走得通 | 六份文字给的每一步指令互相衔接，没有断链或指向不存在的命令；正确性还依赖 J1/J2 的语义层，未在此判 |
| J1（字面兼核）·逐条跑/逐条标记/续跑/`--start-over` 的字面 | 字面自洽，语义未攻 | `54-layer0-replay.sh` 头部注释与代码分支的字面与 `crash-verifier.md` 的指令对得上；假复用等语义漏洞需要构造实验，留给 Opus |

## 四、没做什么

- 没有判 J1 的语义（假复用、被打断后的进度文件残留、续跑目录跨输入串用的真实攻击构造）：只核了字面自洽，深挖代码语义、造反例是本轮 Opus 攻方腿的射程，我没有跑变异、没有构造中断实验、没有读 `crates/singlefs-harness/src/layer0_progress.rs` 源码。
- 没有判 J2、J3、J4：不归我这条腿。
- 没有判 r3 判决「维持挂起、不采纳」被用户决定推翻是否合法、该不该推翻——那是主 agent 的判决权（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判决由主 agent 做，不由投票做」），我只报告字面上两处互相矛盾、没有互相指认这一事实。
- 没有跑任何重型测试、没有跑 `admission.py`、`54-layer0-replay.sh` 的自证或任何编译；本轮只做文字与字面比对，全部靠 `grep -nF` / `awk`/ Read 核对，未使用探针目录（`research/prompts/defs-gatebatch-m2-r1-sonnet-model/` 留空，本格不需要可执行探针；已建目录但没有内容，无需 `SHA256SUMS` / `rerun.sh`）。
- 没有核附录二里 `check-segment-registry.py` 那两处修补（52 号），它不在 J5/J1 的射程内。
