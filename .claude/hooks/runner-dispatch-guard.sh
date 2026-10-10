#!/usr/bin/env bash
# PreToolUse hook（Agent）：派发那一刻查派发提示齐不齐、写得对不对，缺一样当场拒（退出 2，stderr 写原因与出路）。
# hook-events: PreToolUse:Agent PreToolUse:Task
# admission: always Claude Code 在主 agent 每次派发之前调它，判的是这一次的派发提示与此刻会话里在跑的子 agent
# run-condition: command python3
#
# 判法（judge_dispatch）：各条都判完，不成立的按下面的次序一次列全、每条带自己的出路，有一条不成立就拒；哪些情形拒、哪些放照各条写的：
#   ① 重型测试一行：派任何类型，提示里要有一行「重型测试：不跑」（重型测试只由主 agent 跑）（行首可带 - * > 与粗体）。
#      有那一行也照判提示正文里许可子 agent 跑 checker 档 / 重型测试的句子（heavy_permission_sentences），有一句就拒、引出那几句：
#      许可词（可以、允许、准许、能、可……）后面八个字以内接跑 / 执行，同一句里又有 checker 档、--include-ignored（前面最近点名的包是别的包的不算）、重型测试
#      或起测试而点名 checker 档包的 cargo（`cargo test -p singlefs-checker-tier …`，带 --no-run 的不算）、或两档里有轻型的那三条的重型写法
#      （crash-point-replay.sh、checker-tier-formal-verification.sh、checker-tier-crates-mutation-replay.sh 后面同一段命令里——反引号里的到反引号为止、不在反引号里的到子句末——
#      没有 --quick、--list、--list-items、--selftest、--tool-versions、--write 的；heavy_permission_light_capable_runs）；「只跑」与 checker 档同一个子句；
#      「跑 harness 档或 … checker 档」；跑 / 执行之后同一子句里有起测试而点名 checker 档包的 cargo 或那三条的重型写法。前面紧挨否定的（不跑、不带、非、拦……）、
#      轻型那三条带 --quick（可再带 --item）的写法许可子 agent 跑照放（.claude/rules/verification.md「两档：轻型与重型」）；
#      同一子句后面写了不跑 / 归提交 / 重型的、后面接 clippy / build / 编译的不算；「」『』“”里的转述、「重型测试：不跑」那一截、提到主 agent 的子句不判。
#      判的只是这几种字面形，别的说法（「checker 档的用例你自己验一下」）认不出；执行时 heavy-test-guard.sh 与看门狗的进程一层照旧兜底。
#   ⑫ 内置通用类型（general-purpose、claude，或不给 subagent_type）一律拒：一次性的活派项目定义 clerk（起步不带 CLAUDE.md 与规则），只读搜索派 Explore；
#      确要派内置类型，提示里写一行「通用已判：<为什么 clerk 做不了>」放行。
#   ⑬ 骨架的占位没填完：提示里还有 `<填 …>`（research/scripts/dispatch-prompt.py 打出来的占位）就拒，逐个列出那几行。
#   ② 不读共用约束的类型（Explore、写了放行行的通用类型，或没有定义文件的类型）：提示里要有「开工先读」与 `.claude/agent-common.md`。
#   ③ 输入齐不齐：`.claude/agents/<类型>.md` 的 frontmatter 有 `required-inputs:` 的，逐组查——组之间逗号分隔，组内 `|` 分隔的同义词在提示里出现一个就算；
#      定义不在、没写这一行的不查。
#   ④ 写范围：类型在 `.claude/hooks/agent-write-scope.tsv` 里的，提示里「报告路径 / 报告写进 / 写进 / 写到 / 更新」后面紧跟的路径
#      （工作根与过渡期旧根下的、research/、.claude/、crates/、litmus/、records/ 起头）逐个按它那几行模式判，不在里面就拒；前面六个字以内有否定词的不判。
#      工作根（默认 ~/.local/share/singlefs/work/）认展开的绝对路径与 ~/…、$HOME/…、$SINGLEFS_WORK_ROOT/… 几种写法，过渡期旧根认 /tmp/claude-<uid>/，
#      认法与表里 $SINGLEFS_WORK_ROOT/ 起头的模式怎么展开都只在同目录的 lib_work_root.py；旧根在跑的子 agent 与别的会话的子 agent 都交回之后由主 agent 撤（撤法在那份库的文件头）。
#   ⑤ 实现员：要有一行「要动的 crates 文件：…」（份数不设上限，用户 2026-09-27 定；`crates/mutations.tsv` 只追加、不算撞）；
#      与本会话还在跑的实现员登记的文件有交集就拒。登记在 <DISPATCH_GUARD_STATE_ROOT>/<会话 id>/implementation-writers.tsv，一行一次派发
#      （时刻、提示的 sha256、文件）；每次判之前先删掉已结束的：主会话记录里那次派发（toolUseResult 的 prompt 同 sha256）的 agent
#      交回了（`<agent-message from="<id>">` 后跟 `[Subagent hand-back]`）、任务通知是 failed / killed / stopped、或被 TaskStop 停了；
#      派发之后 10 分钟主会话记录里还没有它的，按没派出去删。
#   ⑥ 快照冲突：派 kb-scribe、implementation-writer 时要改的文件在还没判完的三方轮开工快照里，拒（判法见 snapshot_conflict_verdict 上面那段）。
#   ⑦ 并发与模型：本会话还在跑的子 agent（不分模型；主会话记录里派出去、没结束、会话记录 3 小时内动过或 3 小时内派的）
#      已有 SUBAGENT_CONCURRENCY_LIMIT 个（5，用户 2026-10-02 定）就拒，没有放行行。同一条消息里并行派的几个，后几个判的时候前几个可能还没写进主会话记录，数不到。另外，主会话记录里 8 天内有 failed 任务通知写着「resets <时刻> (<时区缩写>)」、它的
#      「model sent to the API」与新派的同一族（opus / sonnet / haiku），而现在还没到那个时刻，拒；那次失败之后同一族又派出去、没再撞限额的
#      算窗口已解开（换了账号或提前恢复），不拒；放行行「限额窗口已判：<理由>」。
#   ⑧ 书记员：提示里点名的工作根或过渡期旧根下的规格文件逐份交给 research/scripts/kb-spec-check.py，红了拒；一份认得出的规格都没有也拒
#      （规格写在提示正文里的，把提示正文当规格判）；放行行「规格检查已判：<理由>」。
#   ⑨ 设计员写重跑登记（提示里有 research/prompts/e<号>-r<n>-prereg.md）：`research/scripts/admission.py experiment <根> E<号>` 退 77
#      （输入自上次产物以来没变）就拒；没登记的实验（退 2）放行；放行行「准入已判：<理由>」。
#   ⑩ 变异分诊员：提示里要有 `research/mutations/…tsv` 或 `crates/mutations.tsv`——不接没有表的广谱活动。
#   ⑪ 执行员：提示里写着「只修锚点」或「只补落产物」的放行这一条；否则要有「这一段回答的岔路：<非空>」；登记对应的实验已经有实验页的算续做，
#      还要有「上一段岔路表里还差：<非空>」，写「无 / 没有」的拒（一行都不差就该交岔路表，不续派）。
#   ⑭ 设计员、执行员的预计挂钟与长活声明（判法在同目录 lib_long_run.py，判据 .claude/singlefs-ai-sop/rules/test-script-discipline.md「用上现有的硬件加速」
#      「判定可复用：按输入存，按前缀剪」）：要有一行「预计挂钟：2 小时内」或「预计挂钟：超过 2 小时」（后面可接逗号与推的依据），没有或认不出就拒；
#      超过的，同一份提示里要有「分片：」「续跑：」「多机：」三行（多机一行点名 --emit-hosts 与 host-resources.py、不写死台数线程内存），
#      缺的由一行「分不了：」点名（至少 8 个字），不齐就拒。在 ⑦ 之后、⑧ 之前判；lib_long_run.py 读不到时不判，stderr 写一句。
# 放行行都写在行首（可带 - * > 与粗体），冒号后面是理由，理由不能空、不能是占位（<理由>）。
# 这一道自己出错（读不了表、主会话记录、规格，JSON 解析不了、脚本跑不起来）不拒：放行，错误写进 stderr。
#
#   runner-dispatch-guard.sh             # 从 stdin 读 hook 的 JSON
#   runner-dispatch-guard.sh --check-prompt <提示文件> --type <类型> [--model <模型>] [--root <仓根>]
#                                        # 派之前拿同一份判法核一份提示（research/scripts/dispatch-prompt.py --check 调它）：不成立的一次列全退 2，都成立退 0；
#                                        # 不读不写实现员登记、不读主会话记录，⑤ 撞文件与 ⑦ 并发、限额窗口写成「派的那一刻另判」
#   runner-dispatch-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝；下面每个弄坏开关单独打开时自检必须判红
# 弄坏开关（一次设一个；自检里经 bash 起钩子的 stdin 那几项也带着同一个值）。逐个写开关名、弄坏哪一处、打开后自检哪几项红：
# 项名是 selftest() 里 cases 的标签，失败行是「✗ 自检：<项名> 应当返回 <应得>，实际 <实得>」，项名就是它的句首；
# 项后面括号里写那一项的应得与实得，项之间用「；」隔开；开关说明末尾括号里的圈号是上面判法的条号：
#   RUNNER_DISPATCH_GUARD_BREAK=disable-check  decide() 一进来就放行，后面哪一条都不判
#       写范围:没登记工作根的类型报告写进工作根拒（应当返回 2，实际 0）；写范围:没登记工作根的类型报告写进过渡期旧根拒（应当返回 2，实际 0）；快照:规格文件写成 ~ 起头也认（应当返回 2，实际 0）；快照:过渡期旧根下的规格文件里点名的照拒（应当返回 2，实际 0）；
#       第一段没点名岔路（应当返回 2，实际 0）；第一段岔路写无（应当返回 2，实际 0）；续做没写还差（应当返回 2，实际 0）；续做还差写无（应当返回 2，实际 0）；重跑登记也算续做（应当返回 2，实际 0）；
#       重型:没写那一行（实现员）（应当返回 2，实际 0）；重型:没写那一行（通用 agent）（应当返回 2，实际 0）；重型:写在句子中间不算那一行（应当返回 2，实际 0）；
#       共用约束:通用 agent 没写开工先读（应当返回 2，实际 0）；通用:general-purpose 没写放行行的拒（应当返回 2，实际 0）；通用:claude 类型同样拒（应当返回 2，实际 0）；
#       骨架:占位没填的拒（应当返回 2，实际 0）；输入:本地攻方缺禁读清单（09-23 那一次）（应当返回 2，实际 0）；输入:核查员缺草稿目录（09-24 那一次）（应当返回 2，实际 0）；
#       输入:核查员缺快照（09-25 那一次）（应当返回 2，实际 0）；写范围:执行员报告写进 research/prompts（09-26 那一次）（应当返回 2，实际 0）；
#       写范围:执行员去更新问题单（应当返回 2，实际 0）；写范围:实现员写进 invariants.md（应当返回 2，实际 0）；变异分诊:没给表（应当返回 2，实际 0）；
#       实现员:第二件撞了在跑的那件（应当返回 2，实际 0）；实现员:没写要动的文件（应当返回 2，实际 0）；并发:已有 5 个在跑再派（应当返回 2，实际 0）；
#       并发:派 sonnet 一样算（应当返回 2，实际 0）；并发:没有定义的类型一样算（应当返回 2，实际 0）；窗口:还没到 resets 时刻不派同一族（应当返回 2，实际 0）；规格:点名的规格旧串不在（应当返回 2，实际 0）；
#       规格:一份认得出的规格都没有（应当返回 2，实际 0）；准入:输入没变不写重跑登记（应当返回 2，实际 0）；快照:书记员撞开着的轮（清单写 ./.claude/）（应当返回 2，实际 0）；
#       快照:实现员撞开着的轮（清单写 tree/crates/）（应当返回 2，实际 0）；快照:提示写项目根起的绝对路径、中文文件名（应当返回 2，实际 0）；
#       快照:清单写 defs/.claude/ 也归一（应当返回 2，实际 0）；快照:规格文件里点名的照拒（路径后面紧跟中文）（应当返回 2，实际 0）；快照:放行行写的是别的轮，照拒（应当返回 2，实际 0）；
#       快照:放行行没写理由，照拒（应当返回 2，实际 0）；快照:放行行没写理由、下一行不算它的理由，照拒（应当返回 2，实际 0）；快照:放行行理由是占位，照拒（应当返回 2，实际 0）；
#       快照:拒绝信息原样转贴，句中的放行模板不算（应当返回 2，实际 0）；快照:一份清单读不了，读得了的那份撞了照拒（应当返回 2，实际 0）；stdin:快照撞了拒绝（退出码 2）（应当返回 2，实际 0）；
#       stdin:快照拒绝的 stderr 写了轮名、文件与出路（应当返回 1，实际 0）；stdin:快照清单读不了时 stderr 写了是哪一份（应当返回 1，实际 0）；stdin:续做没写还差（应当返回 2，实际 0）；
#       stdin:派实现员没写重型测试一行拒绝（退出码 2）（应当返回 2，实际 0）；stdin:拒绝时 stderr 写了出路（应当返回 1，实际 0）；
#       stdin:拒绝时 stderr 的「→ 规矩」那一串列出真编 QEMU（应当返回 1，实际 0）；
#       长活:设计员没写预计挂钟拒（应当返回 2，实际 0）；长活:执行员没写预计挂钟拒（应当返回 2，实际 0）；长活:预计挂钟写的认不出拒（应当返回 2，实际 0）；长活:超过 2 小时一样都没写拒（不分片的长跑法）（应当返回 2，实际 0）；
#       长活:超过 2 小时只写了分片拒（半截声明）（应当返回 2，实际 0）；长活:多机一行写死额度拒（写死额度的派发）（应当返回 2，实际 0）；长活:分不了一行没点名缺的多机拒（应当返回 2，实际 0）；长活:分不了一行不到 8 个字拒（应当返回 2，实际 0）；
#       stdin:超过 2 小时没写长活三样拒绝（退出码 2）（应当返回 2，实际 0）；stdin:长活拒绝的 stderr 写了三行的写法与判据出处（应当返回 1，实际 0）；
#       开关 heavy-permission-unchecked 下红的那十格同样红
#   RUNNER_DISPATCH_GUARD_BREAK=disable-heavy-test  不查「重型测试：不跑」一行（①）
#       重型:没写那一行（实现员）（应当返回 2，实际 0）；重型:写在句子中间不算那一行（应当返回 2，实际 0）；stdin:拒绝时 stderr 写了出路（应当返回 1，实际 0）；
#       stdin:拒绝时 stderr 的「→ 规矩」那一串列出真编 QEMU（应当返回 1，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=heavy-permission-unchecked  不判提示正文里许可跑 checker 档 / 重型测试的句子（①）
#       重型:正文许可跑不带 --include-ignored 的 checker 档快用例拒（第 936 行正推腿的原句）（应当返回 2，实际 0）；重型:正文把 checker 档单个用例列成可跑的拒（第 936 行攻方腿的原句）（应当返回 2，实际 0）；
#       重型:正文要跑 checker 档就只跑快用例拒（应当返回 2，实际 0）；重型:正文许可跑 cargo test -p singlefs-checker-tier 拒（应当返回 2，实际 0）；
#       重型:正文直接写跑 cargo test -p singlefs-checker-tier 拒（应当返回 2，实际 0）；stdin:正文许可跑 checker 档拒绝（退出码 2）（应当返回 2，实际 0）；
#       stdin:许可句拒绝的 stderr 引了那一句、写了出路（应当返回 1，实际 0）；
#       重型:正文许可跑崩溃点重放入口不带 --quick 拒（应当返回 2，实际 0）；重型:正文直接写跑形式化那一道（不带 --quick）拒（应当返回 2，实际 0）；
#       重型:正文许可跑 crates 变异那一道 --full 拒（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=light-tier-commands-unchecked  不认轻型那三条的重型写法（只认 checker 档、--include-ignored、重型测试与 cargo test 点名 checker 档包）（①）
#       重型:正文许可跑崩溃点重放入口不带 --quick 拒（应当返回 2，实际 0）；重型:正文直接写跑形式化那一道（不带 --quick）拒（应当返回 2，实际 0）；
#       重型:正文许可跑 crates 变异那一道 --full 拒（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=light-tier-quick-counted  那三条带 --quick 的也当重型写法（①）
#       重型:正文许可子 agent 跑轻型三条放行（应当返回 0，实际 2）；重型:否定句不跑崩溃点重放的重型跑法、只跑 --quick 放行（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=disable-snapshot  snapshot_conflict_verdict 一进来就放行，快照那一条不判（⑥）
#       快照:规格文件写成 ~ 起头也认（应当返回 2，实际 0）；快照:过渡期旧根下的规格文件里点名的照拒（应当返回 2，实际 0）；
#       快照:书记员撞开着的轮（清单写 ./.claude/）（应当返回 2，实际 0）；快照:实现员撞开着的轮（清单写 tree/crates/）（应当返回 2，实际 0）；
#       快照:提示写项目根起的绝对路径、中文文件名（应当返回 2，实际 0）；快照:清单写 defs/.claude/ 也归一（应当返回 2，实际 0）；快照:规格文件里点名的照拒（路径后面紧跟中文）（应当返回 2，实际 0）；
#       快照:放行行写的是别的轮，照拒（应当返回 2，实际 0）；快照:放行行没写理由，照拒（应当返回 2，实际 0）；快照:放行行没写理由、下一行不算它的理由，照拒（应当返回 2，实际 0）；
#       快照:放行行理由是占位，照拒（应当返回 2，实际 0）；快照:拒绝信息原样转贴，句中的放行模板不算（应当返回 2，实际 0）；快照:一份清单读不了，读得了的那份撞了照拒（应当返回 2，实际 0）；
#       stdin:快照撞了拒绝（退出码 2）（应当返回 2，实际 0）；stdin:快照拒绝的 stderr 写了轮名、文件与出路（应当返回 1，实际 0）；
#       stdin:快照清单读不了时 stderr 写了是哪一份（应当返回 1，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=admission-ignored  designer_admission_verdict 不判重跑登记的准入：admission.py 退 77 也放行（⑨）
#       准入:输入没变不写重跑登记（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=general-purpose-allowed  内置通用类型没写「通用已判」放行行也放行（⑫）
#       通用:general-purpose 没写放行行的拒（应当返回 2，实际 0）；通用:claude 类型同样拒（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=general-purpose-free  没有定义文件的类型没写「开工先读 `.claude/agent-common.md`」也放行（②）
#       共用约束:通用 agent 没写开工先读（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=inputs-ignored  missing_required_inputs 不查定义的 required-inputs（③）
#       输入:本地攻方缺禁读清单（09-23 那一次）（应当返回 2，实际 0）；输入:核查员缺草稿目录（09-24 那一次）（应当返回 2，实际 0）；输入:核查员缺快照（09-25 那一次）（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=json-error-denies  hook 的 JSON 解析不了时拒（退 2），不放行
#       stdin:JSON 解析不了放行（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=json-error-silent  hook 的 JSON 解析不了时 stderr 不写错误
#       stdin:JSON 解析不了时 stderr 写了错误（应当返回 1，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=mutation-table-free  派 mutation-triage 没给变异表也放行（⑩）
#       变异分诊:没给表（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=concurrency-no-limit  model_verdict 不数在跑的子 agent，满上限也放行（⑦）
#       并发:已有 5 个在跑再派（应当返回 2，实际 0）；并发:派 sonnet 一样算（应当返回 2，实际 0）；并发:没有定义的类型一样算（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=placeholders-ignored  骨架占位 `<填 …>` 没填完也放行（⑬）
#       骨架:占位没填的拒（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=registry-ignored  implementation_writer_verdict 不比在跑的实现员登记的文件，撞了也放行（⑤）
#       实现员:第二件撞了在跑的那件（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=registry-no-prune  live_registry_rows 不删已结束的、派发之后没出现的登记行（⑤）
#       实现员:第一件交回之后不再撞（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=runner-backfill-refused  执行员提示写「只补落产物」不放行，照查岔路一行（⑪）
#       执行员:只补落产物放行（应当返回 0，实际 2）；并发:4 个在跑放行（应当返回 0，实际 2）；
#       窗口:换一族放行（应当返回 0，实际 2）；窗口:用户定了照派（应当返回 0，实际 2）；窗口:resets 时刻已过（应当返回 0，实际 2）；窗口:撞限额之后同一族又派出去没再撞，已解开（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=scope-ignored  out_of_scope_mentions 不判写范围（④）
#       写范围:没登记工作根的类型报告写进工作根拒（应当返回 2，实际 0）；写范围:没登记工作根的类型报告写进过渡期旧根拒（应当返回 2，实际 0）；
#       写范围:执行员报告写进 research/prompts（09-26 那一次）（应当返回 2，实际 0）；写范围:执行员去更新问题单（应当返回 2，实际 0）；
#       写范围:实现员写进 invariants.md（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=scope-negation-ignored  路径前面六个字以内的否定词不看，否定句里的路径也判（④）
#       写范围:否定句里的路径不判（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=scope-token-unexpanded  写范围表里 $SINGLEFS_WORK_ROOT/ 起头的模式不展开，照字面比（④）
#       写范围:报告写进草稿目录放行（应当返回 0，实际 2）；写范围:报告写进工作根放行（~ 写法）（应当返回 0，实际 2）；写范围:报告写进工作根放行（展开的绝对路径）（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-any-agent  快照那一条对每种类型都判，不只 kb-scribe 与 implementation-writer（⑥）
#       快照:派别的 agent 放行（设计员）（应当返回 0，实际 2）；快照:派别的 agent 放行（实验设计员）（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-body-optional  open_three_way_rounds 把没有 _<轮>-body.md 正文的快照也算开着的三方轮（⑥）
#       快照:有快照没正文的不是三方轮，放行（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-closed-round-open  open_three_way_rounds 把已有 <轮>-main-verification.md 判决的轮也算开着（⑥）
#       快照:撞的是已判完的轮，放行（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-loose-lines  快照清单每行取最后一个词当路径，不认「<哈希>  <路径>」的形状、不归一（⑥）
#       快照:规格文件写成 ~ 起头也认（应当返回 2，实际 0）；快照:过渡期旧根下的规格文件里点名的照拒（应当返回 2，实际 0）；
#       快照:书记员撞开着的轮（清单写 ./.claude/）（应当返回 2，实际 0）；快照:实现员撞开着的轮（清单写 tree/crates/）（应当返回 2，实际 0）；
#       快照:提示写项目根起的绝对路径、中文文件名（应当返回 2，实际 0）；快照:清单写 defs/.claude/ 也归一（应当返回 2，实际 0）；快照:规格文件里点名的照拒（路径后面紧跟中文）（应当返回 2，实际 0）；
#       快照:放行行写的是别的轮，照拒（应当返回 2，实际 0）；快照:放行行没写理由，照拒（应当返回 2，实际 0）；快照:放行行没写理由、下一行不算它的理由，照拒（应当返回 2，实际 0）；
#       快照:放行行理由是占位，照拒（应当返回 2，实际 0）；快照:拒绝信息原样转贴，句中的放行模板不算（应当返回 2，实际 0）；快照:清单里认不出的行不拒（应当返回 0，实际 2）；
#       stdin:快照撞了拒绝（退出码 2）（应当返回 2，实际 0）；stdin:快照拒绝的 stderr 写了轮名、文件与出路（应当返回 1，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-missing-spec-denies  提示点名而不在的规格文件当成撞了快照（⑥）
#       快照:规格文件不存在不拒（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-no-edges  mentioned_repository_paths 按子串判路径出现，不看前后连着的路径字符（⑥）
#       快照:路径后面连着别的路径字符不算（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-no-normalize  快照清单里的路径不从 .claude/ 或 crates/ 那一段起归一，./、tree/、defs/ 前缀留着（⑥）
#       快照:规格文件写成 ~ 起头也认（应当返回 2，实际 0）；快照:过渡期旧根下的规格文件里点名的照拒（应当返回 2，实际 0）；
#       快照:书记员撞开着的轮（清单写 ./.claude/）（应当返回 2，实际 0）；快照:实现员撞开着的轮（清单写 tree/crates/）（应当返回 2，实际 0）；
#       快照:提示写项目根起的绝对路径、中文文件名（应当返回 2，实际 0）；快照:清单写 defs/.claude/ 也归一（应当返回 2，实际 0）；快照:规格文件里点名的照拒（路径后面紧跟中文）（应当返回 2，实际 0）；
#       快照:放行行写的是别的轮，照拒（应当返回 2，实际 0）；快照:放行行没写理由，照拒（应当返回 2，实际 0）；快照:放行行没写理由、下一行不算它的理由，照拒（应当返回 2，实际 0）；
#       快照:放行行理由是占位，照拒（应当返回 2，实际 0）；快照:拒绝信息原样转贴，句中的放行模板不算（应当返回 2，实际 0）；快照:清单里认不出的行不拒（应当返回 0，实际 2）；
#       stdin:快照撞了拒绝（退出码 2）（应当返回 2，实际 0）；stdin:快照拒绝的 stderr 写了轮名、文件与出路（应当返回 1，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-spec-unread  不读提示点名的工作根或旧根下的规格文件，只判提示正文（⑥）
#       快照:规格文件写成 ~ 起头也认（应当返回 2，实际 0）；快照:过渡期旧根下的规格文件里点名的照拒（应当返回 2，实际 0）；
#       快照:规格文件里点名的照拒（路径后面紧跟中文）（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-unreadable-aborts  读不了的快照清单直接抛异常，整条快照判法走「自己出错放行」（⑥）
#       快照:一份清单读不了，读得了的那份撞了照拒（应当返回 2，实际 0）；stdin:快照清单读不了时 stderr 写了是哪一份（应当返回 1，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-unreadable-denies  读不了的快照清单当成撞了快照（⑥）
#       快照:一份清单读不了，不拒（应当返回 0，实际 2）；stdin:快照清单读不了放行（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-valve-ignored  ruled_snapshot_conflicts 不认「快照冲突已判」放行行（⑥）
#       快照:有放行行，放行（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=snapshot-valve-loose  「快照冲突已判：」出现在任何地方就放行全部轮，不看行首、轮名与理由（⑥）
#       快照:放行行写的是别的轮，照拒（应当返回 2，实际 0）；快照:放行行没写理由，照拒（应当返回 2，实际 0）；快照:放行行没写理由、下一行不算它的理由，照拒（应当返回 2，实际 0）；
#       快照:放行行理由是占位，照拒（应当返回 2，实际 0）；快照:拒绝信息原样转贴，句中的放行模板不算（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=spec-check-skipped  kb_scribe_spec_verdict 不把规格交给 kb-spec-check.py（⑧）
#       规格:点名的规格旧串不在（应当返回 2，实际 0）；规格:一份认得出的规格都没有（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=valve-reason-ignored  valve_reason 一律当没写放行行：「通用已判」「限额窗口已判」「规格检查已判」「准入已判」都不认
#       共用约束:通用 agent 写了开工先读（应当返回 0，实际 2）；窗口:用户定了照派（应当返回 0，实际 2）；规格:写了放行行（应当返回 0，实际 2）；
#       准入:写了放行行（应当返回 0，实际 2）；快照:撞的是已判完的轮，放行（应当返回 0，实际 2）；快照:有放行行，放行（应当返回 0，实际 2）；快照:清单里认不出的行不拒（应当返回 0，实际 2）；
#       快照:规格文件不存在不拒（应当返回 0，实际 2）；快照:一份清单读不了，不拒（应当返回 0，实际 2）；stdin:快照清单读不了放行（应当返回 0，实际 2）；
#       stdin:快照清单读不了时 stderr 写了是哪一份（应当返回 1，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=verdict-not-spec  提示给判决不给规格时照样当规格判，不让写回员自己起草（⑧）
#       书记员:给判决不给规格放行（写回员自己起草）（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=window-ignored  model_verdict 不判限额窗口（⑦）
#       窗口:还没到 resets 时刻不派同一族（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=window-never-lifted  撞限额之后同一族又派出去、没再撞的，也不算窗口解开（⑦）
#       窗口:撞限额之后同一族又派出去没再撞，已解开（应当返回 0，实际 2）
#   RUNNER_DISPATCH_GUARD_BREAK=long-run-unchecked  long_run_verdict 一进来就放行，预计挂钟与长活声明都不判（⑭）
#       长活:设计员没写预计挂钟拒（应当返回 2，实际 0）；长活:执行员没写预计挂钟拒（应当返回 2，实际 0）；长活:预计挂钟写的认不出拒（应当返回 2，实际 0）；长活:超过 2 小时一样都没写拒（不分片的长跑法）（应当返回 2，实际 0）；
#       长活:超过 2 小时只写了分片拒（半截声明）（应当返回 2，实际 0）；长活:多机一行写死额度拒（写死额度的派发）（应当返回 2，实际 0）；长活:分不了一行没点名缺的多机拒（应当返回 2，实际 0）；长活:分不了一行不到 8 个字拒（应当返回 2，实际 0）；
#       stdin:超过 2 小时没写长活三样拒绝（退出码 2）（应当返回 2，实际 0）；stdin:长活拒绝的 stderr 写了三行的写法与判据出处（应当返回 1，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=wall-clock-optional  没写「预计挂钟：」一行当 2 小时内放行（⑭）
#       长活:设计员没写预计挂钟拒（应当返回 2，实际 0）；长活:执行员没写预计挂钟拒（应当返回 2，实际 0）
#   RUNNER_DISPATCH_GUARD_BREAK=stop-at-first  judge_dispatch 只交第一条不成立的（退回逐条试、一次只报一样的旧样子）
#       一次列全:缺三样的提示一次报出三样（重型测试一行、骨架占位、禁读清单）（应当返回 (2, True)，实际 (2, False)）；
#       输入:缺的输入写明按字面子串认、给照抄的一行（应当返回 True，实际 False）
#   RUNNER_DISPATCH_GUARD_BREAK=inputs-hint-dropped  ③ 的拒绝说明退回旧写法：不说按字面子串认哪几个字，不给照抄的一行
#       输入:缺的输入写明按字面子串认、给照抄的一行（应当返回 True，实际 False）
#   RUNNER_DISPATCH_GUARD_BREAK=check-mode-registers  --check-prompt 照派发那一刻判：读主会话记录、读写实现员登记、放行就登记
#       检查模式:--check-prompt 放行时不登记实现员、写明派的那一刻另判的两条（应当返回 (0, True, True)，实际 (0, False, False)）
# 改法加的三格在下面这几个开关下也红：disable-check 下三格都红（一次列全那一格实得 (0, False)，检查模式那一格实得 (0, True, False)）；
# disable-heavy-test、placeholders-ignored 下「一次列全」那一格红（实得 (2, False)）；inputs-ignored 下「一次列全」与「输入:缺的输入…」两格红。
# 长活声明判法的弄坏开关在同目录 lib_long_run.py（LIB_LONG_RUN_BREAK，一次设一个；经 bash 起钩子的 stdin 那几项也带着；
# script-unresolved-accepted 与 wrappers-not-skipped 只 research/scripts/detached-run.sh 用，这里不红）：
#   LIB_LONG_RUN_BREAK=partial-accepted  三样写了一样就当齐了（⑭）
#       长活:超过 2 小时只写了分片拒（半截声明）（应当返回 2，实际 0）；长活:分不了一行没点名缺的多机拒（应当返回 2，实际 0）；长活:分不了一行不到 8 个字拒（应当返回 2，实际 0）
#   LIB_LONG_RUN_BREAK=hosts-unchecked  多机一行不看内容（⑭）
#       长活:多机一行写死额度拒（写死额度的派发）（应当返回 2，实际 0）
#   LIB_LONG_RUN_BREAK=unshardable-unchecked  分不了一行不看字数、不看点没点名（⑭）
#       长活:分不了一行没点名缺的多机拒（应当返回 2，实际 0）；长活:分不了一行不到 8 个字拒（应当返回 2，实际 0）
# 工作根的两个弄坏开关在同目录 lib_work_root.py（一次设一个；经 bash 起钩子的 stdin 那几项也带着）：
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown  工作根退回旧根，提示里工作根下的路径认不出（④⑥⑧）
#       写范围:没登记工作根的类型报告写进工作根拒（应当返回 2，实际 0）；规格:点名的规格检查过了（应当返回 0，实际 2）；快照:规格文件里点名的照拒（路径后面紧跟中文）（应当返回 2，实际 0）；快照:规格文件写成 ~ 起头也认（应当返回 2，实际 0）
#   SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown  不认过渡期旧根，提示里旧根下的路径认不出（④⑥）
#       写范围:没登记工作根的类型报告写进过渡期旧根拒（应当返回 2，实际 0）；快照:过渡期旧根下的规格文件里点名的照拒（应当返回 2，实际 0）
# gate-similar: heavy-test-guard.sh 管同一条「子 agent 不跑重型测试」，但挂 Bash、在执行那一刻判真要跑的命令与它执行的脚本；这里只在派发那一刻查一行结构化声明，判法没有能共用的
# gate-similar: continuation-guard.sh 同样读会话记录认子 agent 交回没有，但它挂 SendMessage、按子 agent 自己的会话记录判收件人；这里挂 Agent|Task、按主会话记录里的派发回执、交回消息、任务通知与 TaskStop 判在跑的实现员与 opus 数，读的记录种类不同
# gate-similar: ask-user-claim-guard.sh 也判一段给人看的中文，但挂 AskUserQuestion、判断言词加出处；这里判的是固定标签行与路径，不逐句理解
# gate-similar: write-guard.sh 读同一张写范围表，但在写的那一刻判目标路径；这里在派发那一刻判提示里点名的路径，模式匹配两边各一份写法（它逐字符拼正则，这里按 ** 与 * 切段）
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$HOOK_DIR/../singlefs-ai-sop/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import glob, hashlib, json, os, re, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timedelta, timezone

try:   # 工作根与过渡期旧根的唯一定义：同目录的 lib_work_root.py（读不到时提示里那两处下的路径都认不出、不判，stderr 写一句）
    sys.dont_write_bytecode = True
    sys.path.insert(0, sys.argv[1]); import lib_work_root as work_roots
    SCRATCH_MENTION_PREFIX, work_roots_error = work_roots.mention_prefix_pattern(), None
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, SCRATCH_MENTION_PREFIX, work_roots_error = None, "(?!)", error
try:   # 长活声明的判法：同目录的 lib_long_run.py（读不到时⑭不判，stderr 写一句）
    sys.path.insert(0, sys.argv[1]); import lib_long_run as long_run
    long_run_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    long_run, long_run_error = None, error

NONE_WORDS = {"", "无", "没有", "无。", "-", "—", "空", "none"}

def labelled_value(prompt, label):
    match = re.search(label + r"[：:]\s*(.*)", prompt)
    return None if match is None else match.group(1).strip()


HEAVY_TEST_OWNERS = set()   # 子 agent 一个都不跑重型测试：提交时的重阶段与整轮门禁由主 agent 跑
GENERAL_PURPOSE_TYPES = {"general-purpose", "claude"}
SKELETON_PLACEHOLDER = re.compile(r"<填[^>\n]*>")   # research/scripts/dispatch-prompt.py 骨架里没填的格   # 内置通用类型：继承项目 CLAUDE.md 与全部规则，起步约 13 万 token
# 写回员（kb-scribe）给判决自己起草规格时，提示里点名的判决文件
VERDICT_MENTION = re.compile(r"research/prompts/[A-Za-z0-9][A-Za-z0-9_.\-]*-main-verification\.md")
LINE_START = r"^[ \t>*_\-]*"
HEAVY_TEST_LINE = re.compile(LINE_START + r"重型测试[* \t]*[：:][* \t]*不跑", re.M)
# 提示正文里许可子 agent 跑 checker 档 / 重型测试的句子（①；判法在 heavy_permission_sentences）：先去掉「」『』“”里的转述与「重型测试：不跑」那一截，
# 按句（。；！？与换行）再按子句（逗号与冒号，反引号里的不算）切；提到主 agent 的子句不算（那是主 agent 跑的）
HEAVY_PERMISSION_QUOTED = re.compile(r"「[^「」\n]*」|『[^『』\n]*』|“[^“”\n]*”")
HEAVY_PERMISSION_LINE_FRAGMENT = re.compile(r"重型测试[*\s]*[：:][*\s]*不跑")
HEAVY_PERMISSION_SENTENCE_SPLIT = re.compile(r"[。；;！？!?\n]")
HEAVY_PERMISSION_CLAUSE_SPLIT = re.compile(r"[，,：:]")
# 起测试的 cargo 点名 checker 档包（只编不跑的带 --no-run，不算）
HEAVY_PERMISSION_CARGO_TEST = re.compile(r"cargo\s+(?:\+\S+\s+)?(?:test|t|nextest\s+run|bench|miri\s+test|llvm-cov|hack\s+test)\b[^，。；;`）)\n]*?"
                                         r"(?:-p|--package)[\s=]*singlefs-checker-tier\b[^，。；;`）)\n]*")
HEAVY_PERMISSION_OBJECT = re.compile(r"checker\s*档|--include-ignored|重型测试|重测试")
HEAVY_PERMISSION_CHECKER_TIER = re.compile(r"checker\s*档")
# 许可词后面八个字以内接跑 / 复跑 / 执行 / 运行（「跑过」「跑完」「跑的」「执行员」「跑不了」这类不是许可跑）
HEAVY_PERMISSION_VERB_TAIL = r"(?:复跑|重跑|跑|执行|运行)(?![过完出的了掉通绿红前员者]|不[了动起完过成])"
HEAVY_PERMISSION_WORD = re.compile(r"(?:可以|允许|准许|不妨|尽管|放心|(?<![许认宁不尽])可|(?<![功才可智性效职])能)[^，,。；;：:]{0,8}?" + HEAVY_PERMISSION_VERB_TAIL)
HEAVY_PERMISSION_RUN_VERB = re.compile(HEAVY_PERMISSION_VERB_TAIL)
HEAVY_PERMISSION_ONLY_RUN = re.compile(r"只(?:复跑|跑|执行|运行)")
HEAVY_PERMISSION_HARNESS_OR_CHECKER = re.compile(r"(?:复跑|重跑|跑|执行|运行)[^，,。；;]{0,6}?harness\s*档[^，,。；;]{0,24}?(?:或|与|和|、|及|加)[^，,。；;]{0,24}?(checker\s*档)")
HEAVY_PERMISSION_NEGATION_BEFORE = re.compile(r"(?:不|没|别|勿|禁止|不许|不准|不得|严禁|无须|无需|不用|不要|切勿|莫|非)[\s*]*$")
HEAVY_PERMISSION_OBJECT_NEGATION_BEFORE = re.compile(r"(?:非|不带|不含|不是|除了|除|不跑|别跑|不许跑|不准跑|禁跑|不碰|不加|拦)[\s*`]*$")
HEAVY_PERMISSION_OBJECT_NEGATION_AFTER = re.compile(r"^[^，,]*?(?:不跑|别跑|不许|不准|禁止|禁跑|一律不|不归|不要|不能|不得|严禁|不碰|归提交|归主|提交时|重型)")
HEAVY_PERMISSION_LINT_AFTER = re.compile(r"^[\s`]*的?[\s`]*(?:clippy|build|fmt|check|编译|--all-targets)")
HEAVY_PERMISSION_MENTION_AFTER = re.compile(r"^\s*(?:闸|钩子|的钩子|的闸|一行|那一行)")
HEAVY_PERMISSION_MAIN_AGENT = re.compile(r"主\s*agent")
# 两档里有轻型的那三条（.claude/rules/verification.md「两档：轻型与重型」）：点名脚本文件（前面可带 bash 与路径）；同一段命令里有 --quick 或只查清单的参数的不是重型写法
HEAVY_PERMISSION_LIGHT_CAPABLE = re.compile(r"(?:\bbash\s+)?(?:[\w.~/-]*/)?(?:crash-point-replay|checker-tier-formal-verification|checker-tier-crates-mutation-replay)\.sh\b")
HEAVY_PERMISSION_LIGHT_OR_QUERY = re.compile(r"--(?:quick|list|list-items|selftest|tool-versions|write)(?![\w-])")
COMMON_CONSTRAINTS_READ = re.compile(r"开工先读[^\n]*\.claude/agent-common\.md")
SUBAGENT_CONCURRENCY_LIMIT = 5              # 用户 2026-10-02 定（原话「一个主agent 最多同时开5个subagent」）
INHERITED_MODEL_FAMILY = "opus"             # inherit 与没有定义的类型按主 agent 的模型算；这个项目的主 agent 跑 opus（推的）
RUNNING_RECENT_SECONDS = 3 * 3600
UNLAUNCHED_GRACE_SECONDS = 600
LIMIT_WINDOW_LOOKBACK_SECONDS = 8 * 86400
CRATES_FILES_LINE = re.compile(LINE_START + r"要动的\s*crates\s*文件[* \t]*[：:](.*)$", re.M)
CRATES_PATH = re.compile(r"crates/[^\s`'\"，。；：、（）()「」,;]+")
SCOPE_MENTION = re.compile(r"(?:报告路径|报告写进|报告写到|写进|写到|更新)[*\s]*[：:]?[*\s]*`?(?P<path>(?:" + SCRATCH_MENTION_PREFIX + r"|research/|\.claude/|crates/|litmus/|records/)[^\s`'\"，。；：、（）()「」,;]*)")
SCOPE_NEGATION = re.compile(r"不|别|禁止|勿")
MUTATION_TABLE_MENTION = re.compile(r"research/mutations/[^\s`'\"]+\.tsv|crates/mutations\.tsv")
RERUN_REGISTRATION = re.compile(r"research/prompts/e(\d+)-r\d+-prereg\.md")
RESETS = re.compile(r"resets (?:(?P<month>[A-Z][a-z]{2}) (?P<day>\d{1,2}),? )?(?P<hour>\d{1,2})(?::(?P<minute>\d{2}))?(?P<half>am|pm) \(UTC\)")  # clock-times:allow 解析外部限额通知的原文，格式由对方定
MODEL_SENT = re.compile(r"model sent to the API: ([\w.\-\[\]]+)")
HANDBACK_FROM = re.compile(r'agent-message from=\\?"(a[0-9a-f]{6,})\\?">(?:\\n|\s)*\[Subagent hand-back\]')
NOTIFICATION_BLOCK = re.compile(r"<task-notification>(.*?)</task-notification>", re.S)
MONTHS = {name: index for index, name in enumerate(["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"], 1)}


def valve_reason(prompt, label):
    """行首「<label>：<理由>」的理由；没有、空、占位都返回 None。"""
    match = re.search(LINE_START + re.escape(label) + r"[* \t]*[：:][* \t]*(.*?)[ \t]*$", prompt, re.M)
    if match is None or break_switch_is("valve-reason-ignored"):
        return None
    reason = match.group(1).strip()
    return None if not reason or reason.startswith("<") or reason in NONE_WORDS else reason


def agent_definition_fields(project_root, subagent_type):
    """定义的 frontmatter → {键: 值}（值去掉行尾 # 注释）；定义不在返回 None。"""
    path = os.path.join(project_root, ".claude", "agents", f"{subagent_type}.md")
    try:
        text = open(path, encoding="utf-8").read()
    except OSError:
        return None
    fields = {}
    if text.startswith("---"):
        for line in text.split("\n---", 1)[0].splitlines()[1:]:
            if ":" in line:
                key, value = line.split(":", 1)
                fields[key.strip()] = value.split("#", 1)[0].strip()
    return fields


def missing_required_inputs(fields, prompt):
    if break_switch_is("inputs-ignored"):
        return []
    groups = [group.strip() for group in (fields.get("required-inputs") or "").split(",") if group.strip()]
    return [group for group in groups if not any(alternative.strip() and alternative.strip() in prompt for alternative in group.split("|"))]


def expanded_mention(text):
    """文字里写的 ~/…、$HOME/…、$SINGLEFS_WORK_ROOT/… 换成绝对路径（lib_work_root.py 的 expand_written_path）；读不到那份库原样交回。"""
    return work_roots.expand_written_path(text) if work_roots else text


def expanded_scope_patterns(pattern):
    """写范围表的一条模式：$SINGLEFS_WORK_ROOT/ 起头的展开成工作根与过渡期旧根各一条；读不到 lib_work_root.py 时丢掉（那两处下的路径本来也认不出）。"""
    if not pattern.startswith("$") or break_switch_is("scope-token-unexpanded"):
        return [pattern]
    return work_roots.expand_scope_pattern(pattern) if work_roots else []


def scope_pattern_regex(pattern):
    pieces = re.split(r"(\*\*|\*)", pattern)
    return re.compile("".join(".*" if piece == "**" else "[^/]*" if piece == "*" else re.escape(piece) for piece in pieces) + r"\Z")


def scope_patterns(project_root, subagent_type):
    patterns = []
    table_path = os.path.join(project_root, ".claude", "hooks", "agent-write-scope.tsv")
    if not os.path.isfile(table_path):
        return patterns
    try:
        for line in open(table_path, encoding="utf-8"):
            fields = line.rstrip("\n").split("\t")
            if not line.startswith("#") and len(fields) >= 2 and fields[0].strip() == subagent_type:
                patterns += expanded_scope_patterns(fields[1].strip())
    except OSError as error:
        print(f"! 派发闸读不了写范围表，写范围这一条没判：{error}", file=sys.stderr)
    return patterns


def out_of_scope_mentions(project_root, subagent_type, prompt):
    patterns = scope_patterns(project_root, subagent_type)
    if not patterns or break_switch_is("scope-ignored"):
        return []
    outside = []
    for match in SCOPE_MENTION.finditer(prompt):
        before = prompt[max(0, match.start() - 6):match.start()]
        if SCOPE_NEGATION.search(before) and not break_switch_is("scope-negation-ignored"):
            continue
        path = expanded_mention(match.group("path").rstrip("/."))
        root_prefix = os.path.normpath(project_root) + "/"
        relative = path[len(root_prefix):] if path.startswith(root_prefix) else path
        if not any(scope_pattern_regex(pattern).match(relative) for pattern in patterns) and relative not in outside:
            outside.append(relative)
    return outside


def model_family(model_text):
    lowered = (model_text or "").lower()
    for family in ("opus", "sonnet", "haiku", "fable"):
        if family in lowered:
            return family
    return INHERITED_MODEL_FAMILY if lowered in ("", "inherit") else lowered


def parse_moment(text):
    return datetime.fromisoformat(text.replace("Z", "+00:00")).timestamp()


def session_dispatch_events(transcript_path):
    """主会话记录 → (launches{agent 号: (派发时刻, 模型, 提示 sha256)}, ended{agent 号: 最晚结束时刻}, windows[(到期时刻, 模型族, 原句)])。
    先用 grep 挑出带这几种记号的行，免得逐行解析上百兆的记录；读不了返回三个空的。"""
    launches, ended, windows = {}, {}, []
    if not transcript_path or not os.path.isfile(transcript_path):
        return launches, ended, windows
    needles = ['"async_launched"', "[Subagent hand-back]", "<task-notification>", "Successfully stopped task"]
    try:
        grep = subprocess.run(["grep", "-F", *sum((["-e", needle] for needle in needles), []), transcript_path],
                              capture_output=True, text=True, timeout=50)
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"! 派发闸读不了主会话记录，按会话判的几条没判：{error}", file=sys.stderr)
        return launches, ended, windows
    now = time.time()
    for line in grep.stdout.splitlines():
        try:
            record = json.loads(line)
            moment = parse_moment(record.get("timestamp") or "")
        except (ValueError, TypeError):
            continue
        result = record.get("toolUseResult")
        if isinstance(result, dict) and result.get("status") == "async_launched" and result.get("agentId"):
            digest = hashlib.sha256(str(result.get("prompt") or "").encode("utf-8")).hexdigest()
            launches[result["agentId"]] = (moment, result.get("resolvedModel") or "", digest)
        if isinstance(result, dict) and "Successfully stopped task" in str(result.get("message", "")) and result.get("task_id"):
            ended[result["task_id"]] = max(moment, ended.get(result["task_id"], 0))
        for agent_id in HANDBACK_FROM.findall(line):
            ended[agent_id] = max(moment, ended.get(agent_id, 0))
        for block in NOTIFICATION_BLOCK.findall(line.replace("\\n", "\n").replace('\\"', '"')):
            status = re.search(r"<status>(\w+)</status>", block)
            for agent_id in re.findall(r"<task-id>(\w+)</task-id>", block):
                if status and status.group(1) in ("failed", "killed", "stopped"):
                    ended[agent_id] = max(moment, ended.get(agent_id, 0))
            resets, sent = RESETS.search(block), MODEL_SENT.search(block)
            if status and status.group(1) == "failed" and resets and now - moment <= LIMIT_WINDOW_LOOKBACK_SECONDS:
                windows.append((reset_moment(resets, moment), model_family(sent.group(1) if sent else ""), resets.group(0), moment,
                                re.findall(r"<task-id>(\w+)</task-id>", block)))
    return launches, ended, windows


def reset_moment(match, notified_at):
    """「resets <时>am (<时区缩写>)」→ 通知之后第一次到那个时刻；写了日期的取那一天（早于通知就算明年）。"""
    hour = int(match.group("hour")) % 12 + (12 if match.group("half") == "pm" else 0)
    minute = int(match.group("minute") or 0)
    base = datetime.fromtimestamp(notified_at, timezone.utc)
    if match.group("month"):
        candidate = base.replace(month=MONTHS.get(match.group("month"), base.month), day=int(match.group("day")), hour=hour, minute=minute, second=0, microsecond=0)
        if candidate.timestamp() < notified_at:
            candidate = candidate.replace(year=candidate.year + 1)
        return candidate.timestamp()
    candidate = base.replace(hour=hour, minute=minute, second=0, microsecond=0)
    if candidate.timestamp() <= notified_at:
        candidate = datetime.fromtimestamp(candidate.timestamp() + 86400, timezone.utc)
    return candidate.timestamp()


def is_running(agent_id, launches, ended, transcript_path):
    launched_at = launches[agent_id][0]
    if ended.get(agent_id, 0) >= launched_at:
        return False
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl") if transcript_path.endswith(".jsonl") else ""
    last_seen = max(launched_at, os.path.getmtime(own) if own and os.path.isfile(own) else 0)
    return time.time() - last_seen <= RUNNING_RECENT_SECONDS


def registry_path(hook_input):
    root = os.environ.get("DISPATCH_GUARD_STATE_ROOT") or (work_roots.state_directory("dispatch-guard-state") if work_roots
                                                           else os.path.join(tempfile.gettempdir(), "dispatch-guard-state"))
    session = re.sub(r"[^\w.-]", "_", str(hook_input.get("session_id") or "unknown-session"))
    return os.path.join(root, session, "implementation-writers.tsv")


def live_registry_rows(hook_input, launches, ended):
    """登记里还在跑的那几行（顺手把结束了的、派发之后没出现的删掉）。→ [(时刻, 提示 sha256, [文件])]。"""
    path = registry_path(hook_input)
    try:
        rows = [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8") if line.strip()]
    except OSError:
        return []
    by_digest = {digest: agent_id for agent_id, (_, _, digest) in launches.items()}
    transcript_path = hook_input.get("transcript_path") or ""
    live = []
    for row in rows:
        if len(row) != 3:
            continue
        registered_at, digest, files = float(row[0]), row[1], row[2].split()
        agent_id = by_digest.get(digest)
        if not break_switch_is("registry-no-prune"):
            if agent_id is None and time.time() - registered_at > UNLAUNCHED_GRACE_SECONDS:
                continue
            if agent_id is not None and not is_running(agent_id, launches, ended, transcript_path):
                continue
        live.append((registered_at, digest, files))
    rewrite_registry(path, live)
    return live


def rewrite_registry(path, rows):
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        temporary = f"{path}.{os.getpid()}.tmp"
        with open(temporary, "w", encoding="utf-8") as handle:
            for registered_at, digest, files in rows:
                handle.write(f"{registered_at}\t{digest}\t{' '.join(files)}\n")
        os.replace(temporary, path)
    except OSError as error:
        print(f"! 派发闸写不了实现员登记 {path}：{error}", file=sys.stderr)


def register_implementation_writer(hook_input, prompt, files, live_rows):
    digest = hashlib.sha256(prompt.encode("utf-8")).hexdigest()
    rewrite_registry(registry_path(hook_input), live_rows + [(time.time(), digest, files)])


def implementation_writer_verdict(hook_input, prompt, launches, ended, registry=True):
    """→ (拒绝说明或 None, 要登记的文件, 还在跑的登记行)。registry 为假（--check-prompt）时只判那一行，不读不写登记。"""
    line = CRATES_FILES_LINE.search(prompt)
    if line is None:
        return ("✗ 派 implementation-writer 的提示里没有「要动的 crates 文件：…」一行。\n"
                "→ 怎么办：把这一件要动的 crates/ 文件逐个写进这一行（路径从仓根起）；说不全要动哪些文件，这件活还没切小，先切。"), [], []
    # crates/mutations.tsv 只追加、不算撞文件（主 agent 合补丁时按 mutations-append.tsv 并）；份数不设上限（用户 2026-09-27 定）。
    files = sorted(set(CRATES_PATH.findall(line.group(1))) - {"crates/mutations.tsv"})
    if not registry:
        return None, files, []
    live = live_registry_rows(hook_input, launches, ended)
    if break_switch_is("registry-ignored"):
        return None, files, live
    overlapping = sorted({path for _, _, registered in live for path in registered} & set(files))
    if overlapping:
        return (f"✗ 这一件要动的文件与还在跑的实现员撞了：{'、'.join(overlapping)}。\n"
                "→ 怎么办：排在那个实现员交回之后再派，或把两件并给同一个实现员（.claude/main-agent.md「一轮怎么开、怎么收」第 4 条）。"), files, live
    return None, files, live


def model_verdict(hook_input, prompt, subagent_type, fields, launches, ended, windows):
    requested = (hook_input.get("tool_input") or {}).get("model") or (fields or {}).get("model") or ""
    family = model_family(requested)
    transcript_path = hook_input.get("transcript_path") or ""
    now = time.time()
    limit_failed = {agent_id for window in windows for agent_id in window[4]}
    for until, window_family, original, notified_at, _ in windows:
        # 撞限额之后同一族又派出去、而且没再撞限额的：窗口已经解开了（换了账号或限额提前恢复）
        lifted = any(model_family(model) == window_family and launched_at > notified_at and agent_id not in limit_failed
                     for agent_id, (launched_at, model, _) in launches.items()) and not break_switch_is("window-never-lifted")
        if (window_family == family and now < until and not lifted and not break_switch_is("window-ignored")
                and valve_reason(prompt, "限额窗口已判") is None):
            wait_minutes = max(1, int((until - now + 59) // 60))
            wait_text = f"{wait_minutes // 60} 小时 {wait_minutes % 60} 分钟" if wait_minutes >= 60 else f"{wait_minutes} 分钟"
            return (f"✗ {family} 这一族撞过限额（「{original}」），限额窗口还要约 {wait_text} 才过去，这之前不派同一族的 agent。\n"
                    f"→ 怎么办：等约 {wait_text} 窗口过去，或换模型（派发参数 model 写别的一族）——换不换交用户定；用户定了照派，提示里写一行「限额窗口已判：<用户怎么定的>」。")
    if break_switch_is("concurrency-no-limit"):
        return None
    running = [agent_id for agent_id in launches if is_running(agent_id, launches, ended, transcript_path)]
    if len(running) >= SUBAGENT_CONCURRENCY_LIMIT:
        return (f"✗ 本会话已有 {len(running)} 个子 agent 在跑（{'、'.join(running[:10])}），一个主 agent 同时最多 {SUBAGENT_CONCURRENCY_LIMIT} 个（用户 2026-10-02 定）。\n"
                "→ 怎么办：等其中一个交回再派；派不下的排进下一波（.claude/main-agent.md「一轮怎么开、怎么收」第 1 条）。")
    return None


def kb_scribe_spec_verdict(prompt, project_root, hook_dir):
    if valve_reason(prompt, "规格检查已判") is not None or break_switch_is("spec-check-skipped"):
        return None
    checker = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "research", "scripts", "kb-spec-check.py")
    candidates = []
    for mention in sorted(set(SCRATCH_FILE_MENTION.findall(prompt))):
        path = expanded_mention(mention.rstrip("。，；：、."))
        if os.path.isfile(path) and path.endswith((".json", ".md", ".txt")):
            candidates.append(path)
    inline = None
    if not candidates and VERDICT_MENTION.search(prompt) and not break_switch_is("verdict-not-spec"):
        return None   # 写回员自己起草：给的是判决，规格由它写、它跑 kb-spec-check.py
    if not candidates:
        inline = tempfile.NamedTemporaryFile("w", suffix=".md", delete=False, encoding="utf-8")
        inline.write(prompt)
        inline.close()
        candidates = [inline.name]
    try:
        recognized, problems = 0, []
        for path in candidates:
            try:
                result = subprocess.run([sys.executable, checker, path, "--root", project_root], capture_output=True, text=True, timeout=50)
            except (OSError, subprocess.TimeoutExpired) as error:
                print(f"! 派发闸跑不了 kb-spec-check.py，规格这一条没判：{error}", file=sys.stderr)
                return None
            if result.returncode in (0, 1):
                recognized += 1
            if result.returncode == 1:
                problems += [line.strip() for line in result.stdout.splitlines() if line.strip().startswith("✗")]
            elif result.returncode not in (0, 2):
                print(f"! kb-spec-check.py 退出码 {result.returncode}，规格这一条没判：{result.stdout[-200:]}{result.stderr[-200:]}", file=sys.stderr)
                return None
        if problems:
            return ("✗ 给书记员的规格会让门禁红（research/scripts/kb-spec-check.py）：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                    "\n→ 怎么办：改规格原文再派；判断不了的交用户；要写回员自己起草就给它判决与条目、不给规格。确要照这份派，提示里写一行「规格检查已判：<理由>」。")
        if recognized == 0:
            return ("✗ 派 kb-scribe 的提示里认不出一份规格（提示正文与点名的工作根或过渡期旧根下的文件都不是 kb-spec-check.py 认的写法）。\n"
                    "→ 怎么办：照 research/scripts/kb-spec-check.py 文件头「规格的两种写法」写，或给判决让写回员自己起草；确要照原样派，提示里写一行「规格检查已判：<理由>」。")
        return None
    finally:
        if inline is not None:
            os.unlink(inline.name)


def designer_admission_verdict(prompt, project_root, hook_dir):
    match = RERUN_REGISTRATION.search(prompt)
    if match is None or break_switch_is("admission-ignored") or valve_reason(prompt, "准入已判") is not None:
        return None
    admission = (os.environ.get("RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT")
                 or os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "research", "scripts", "admission.py"))
    try:
        result = subprocess.run([sys.executable, admission, "experiment", project_root, f"E{int(match.group(1))}"], capture_output=True, text=True, timeout=50)
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"! 派发闸跑不了 admission.py，准入这一条没判：{error}", file=sys.stderr)
        return None
    if result.returncode == 77:
        return (f"✗ E{int(match.group(1))} 的输入自上次产物以来没变（research/scripts/admission.py experiment 退 77），不写它的重跑登记。\n"
                "→ 怎么办：沿用上次产物与结论；要为别的原因重跑（换臂、换判据），在提示里写一行「准入已判：<哪一样变了>」。")
    return None

SNAPSHOT_GUARDED_AGENTS = {"kb-scribe", "implementation-writer"}
SNAPSHOT_DIRECTORY_SUFFIX = "-snapshot"
SNAPSHOT_LIST_NAME_MARK = "sha256"
# 快照清单的一行：64 位十六进制哈希、空白、路径（sha256sum 二进制模式在路径前加 *）
SNAPSHOT_LIST_LINE = re.compile(r"^\s*[0-9a-fA-F]{64}\s+\*?(\S.*?)\s*$")
# 清单里的路径从第一段 .claude/ 或 crates/ 起取（./.claude/…、tree/crates/…、defs/.claude/… 都归一成仓库根相对路径）
REPOSITORY_PATH_START = re.compile(r"(?:^|/)((?:\.claude|crates)/)")
# 一条路径在文字里「出现」：前后不连着路径字符（后面的 . 只在它后面还跟着路径字符时才算连着，句末的点不算）
PATH_LEFT_EDGE = r"(?<![A-Za-z0-9_\-./~])"
PATH_RIGHT_EDGE = r"(?![A-Za-z0-9_\-/]|\.[A-Za-z0-9_])"
DOT_SLASH_BEFORE_REPOSITORY_PATH = re.compile(PATH_LEFT_EDGE + r"\./(?=\.claude/|crates/)")
# 提示里点名的规格文件：工作根或过渡期旧根下的路径（写法见 lib_work_root.py 的 mention_prefix_pattern），到空白、引号、括号或中文标点为止
SCRATCH_FILE_MENTION = re.compile(SCRATCH_MENTION_PREFIX + r"[^\s`'\"，。；：、（）()「」『』“”‘’《》<>\[\]{}|,;*]+")
SPECIFICATION_READ_LIMIT_BYTES = 4 * 1024 * 1024
# 放行口：一行以「快照冲突已判：<轮名> <理由>」开头（行首可以有列表记号与粗体星号）；只在这一行里找，不跨到下一行
SNAPSHOT_CONFLICT_RULED_LINE = re.compile(r"^[ \t>*_\-]*快照冲突已判[* \t]*[：:][* \t]*([^\s，,：:；;。]+)[ \t，,：:；;。]+(.*?)[ \t]*$", re.M)
SNAPSHOT_CONFLICT_RULED_ANYWHERE = re.compile(r"快照冲突已判[*\s]*[：:][*\s]*([^\s，,：:；;。]*)")
SNAPSHOT_CONFLICTS_SHOWN_LIMIT = 20

def break_switch_is(name):
    """自检用的弄坏开关：RUNNER_DISPATCH_GUARD_BREAK 等于 name 时，那一处判法故意走错，自检必须判红。"""
    return os.environ.get("RUNNER_DISPATCH_GUARD_BREAK") == name

def open_three_way_rounds(project_root):
    """还没判完的三方轮：[(轮名, 快照目录)]。research/prompts/<轮>-snapshot/ 在、_<轮>-body.md 在、<轮>-main-verification.md 不在。"""
    prompts_directory = os.path.join(project_root, "research", "prompts")
    if not os.path.isdir(prompts_directory):
        return []
    rounds = []
    for entry_name in sorted(os.listdir(prompts_directory)):
        snapshot_directory = os.path.join(prompts_directory, entry_name)
        if not entry_name.endswith(SNAPSHOT_DIRECTORY_SUFFIX) or not os.path.isdir(snapshot_directory):
            continue
        round_name = entry_name[:-len(SNAPSHOT_DIRECTORY_SUFFIX)]
        has_body = os.path.isfile(os.path.join(prompts_directory, f"_{round_name}-body.md"))
        has_verdict = os.path.exists(os.path.join(prompts_directory, f"{round_name}-main-verification.md"))
        if round_name and (has_body or break_switch_is("snapshot-body-optional")) and (not has_verdict or break_switch_is("snapshot-closed-round-open")):
            rounds.append((round_name, snapshot_directory))
    return rounds

def repository_path_of_snapshot_line(line):
    """快照清单的一行归一成的仓库根相对路径；不是「<哈希>  <路径>」的形状、或路径里没有 .claude/ 与 crates/ 起头的一段，返回 None。"""
    if break_switch_is("snapshot-loose-lines"):
        fields = line.split()
        return fields[-1] if fields else None
    match = SNAPSHOT_LIST_LINE.match(line)
    if match is None:
        return None
    listed_path = match.group(1)
    if break_switch_is("snapshot-no-normalize"):
        return listed_path
    start = REPOSITORY_PATH_START.search(listed_path)
    if start is None:
        return None
    normalized = os.path.normpath(listed_path[start.start(1):])
    return None if normalized.startswith("..") else normalized

def snapshot_listed_paths(snapshot_directory, notes):
    """快照目录里文件名含 sha256 的每份清单读出来的路径：{仓库根相对路径: 清单文件}。认不出的行跳过；读不了的清单记进 notes、跳过。"""
    listed = {}
    for list_name in sorted(os.listdir(snapshot_directory)):
        if SNAPSHOT_LIST_NAME_MARK not in list_name:
            continue
        list_path = os.path.join(snapshot_directory, list_name)
        try:
            with open(list_path, encoding="utf-8", errors="replace") as list_file:
                lines = list_file.read().splitlines()
        except OSError as error:
            if break_switch_is("snapshot-unreadable-aborts"):
                raise
            if break_switch_is("snapshot-unreadable-denies"):
                listed[f"<读不了的清单 {list_name}>"] = list_path
            notes.append(f"! 派发闸（快照那一条）读不了快照清单 {list_path}：{error}；这一份跳过，没当成撞快照")
            continue
        for line in lines:
            repository_path = repository_path_of_snapshot_line(line)
            if repository_path is not None:
                listed.setdefault(repository_path, list_path)
    return listed

def specification_texts(prompt, notes):
    """提示里点名的工作根或过渡期旧根下的规格文件：[(路径, 内容)]。不在的、是目录的跳过；在而读不了的记进 notes、跳过。"""
    texts = []
    seen = set()
    for mention in SCRATCH_FILE_MENTION.finditer(prompt):
        candidate = expanded_mention(mention.group()).rstrip(".:")
        if not os.path.isfile(candidate):
            # 路径后面紧跟中文（「spec.md里」）：去掉尾巴上的非 ASCII 字再看一次
            candidate = re.sub(r"[^\x00-\x7f]+$", "", candidate).rstrip(".:")
        if candidate in seen:
            continue
        seen.add(candidate)
        if not os.path.isfile(candidate):
            if break_switch_is("snapshot-missing-spec-denies") and not os.path.isdir(candidate):
                texts.append((candidate, "<不在的规格>"))
            continue
        try:
            with open(candidate, "rb") as specification_file:
                texts.append((candidate, specification_file.read(SPECIFICATION_READ_LIMIT_BYTES).decode("utf-8", errors="replace")))
        except OSError as error:
            notes.append(f"! 派发闸（快照那一条）读不了派发提示点名的规格 {candidate}：{error}；这一份没收进要改的文件")
    return texts

def mentioned_repository_paths(text, candidate_paths, project_root):
    """candidate_paths 里哪几条在 text 里出现：前后不连着路径字符；项目根起的绝对路径与 ./ 前缀先去掉。"""
    root_prefix = project_root.rstrip("/")
    if root_prefix:
        text = text.replace(root_prefix + "/", "")
    text = DOT_SLASH_BEFORE_REPOSITORY_PATH.sub("", text)
    if break_switch_is("snapshot-no-edges"):
        return {path for path in candidate_paths if path in text}
    return {path for path in candidate_paths if re.search(PATH_LEFT_EDGE + re.escape(path) + PATH_RIGHT_EDGE, text)}

def ruled_snapshot_conflicts(prompt):
    """派发提示里写了「快照冲突已判：<轮名> <理由>」的轮名（理由非空、不是 <理由> 这种占位）。"""
    if break_switch_is("snapshot-valve-ignored"):
        return set()
    if break_switch_is("snapshot-valve-loose"):
        return {"*"} if SNAPSHOT_CONFLICT_RULED_ANYWHERE.search(prompt) else set()
    ruled = set()
    for match in SNAPSHOT_CONFLICT_RULED_LINE.finditer(prompt):
        reason = match.group(2).strip()
        if reason not in NONE_WORDS and not (reason.startswith("<") and reason.endswith(">")):
            ruled.add(match.group(1))
    return ruled

def snapshot_conflict_verdict(subagent_type, prompt, project_root):
    """第三条。返回 (0, 给 stderr 的提醒或 None) 放行；(2, 说明) 拒绝。这一条自己出错一律放行，错误写进说明。"""
    if break_switch_is("disable-snapshot"):
        return 0, None
    if subagent_type not in SNAPSHOT_GUARDED_AGENTS and not break_switch_is("snapshot-any-agent"):
        return 0, None
    notes = []
    try:
        rounds = open_three_way_rounds(project_root)
        if not rounds:
            return 0, None
        sources = [("派发提示", prompt)]
        if not break_switch_is("snapshot-spec-unread"):
            sources += [(f"规格 {path}", text) for path, text in specification_texts(prompt, notes)]
        ruled = ruled_snapshot_conflicts(prompt)
        conflicts = []  # [(轮名, 快照目录, [(路径, 出现在哪, 清单文件)])]
        for round_name, snapshot_directory in rounds:
            listed = snapshot_listed_paths(snapshot_directory, notes)
            hits = {}
            for source_name, text in sources:
                for path in sorted(mentioned_repository_paths(text, listed, project_root)):
                    hits.setdefault(path, (source_name, listed[path]))
            if break_switch_is("snapshot-unreadable-denies"):
                hits.update({path: ("快照清单", list_path) for path, list_path in listed.items() if path.startswith("<读不了的清单")})
            if break_switch_is("snapshot-missing-spec-denies"):
                hits.update({f"<不在的规格 {path}>": ("派发提示", "-") for path, text in sources[1:] if text == "<不在的规格>"})
            if hits and round_name not in ruled and "*" not in ruled:
                conflicts.append((round_name, snapshot_directory, [(path, *hits[path]) for path in sorted(hits)]))
    except Exception as error:  # 挂在每一次派发上的闸，坏了不能把所有派发都挡住
        notes.append(f"! 派发闸（快照那一条）自己出错，这一次放行：{type(error).__name__}: {error}")
        return 0, "\n".join(notes)
    if not conflicts:
        return 0, ("\n".join(notes) or None)
    listing = []
    for round_name, snapshot_directory, hits in conflicts:
        listing.append(f"  三方轮 {round_name}（开工快照 {os.path.relpath(snapshot_directory, project_root)}/，"
                       f"research/prompts/{round_name}-main-verification.md 还不在）：")
        for path, source_name, list_path in hits[:SNAPSHOT_CONFLICTS_SHOWN_LIMIT]:
            listing.append(f"    {path}（出现在{source_name}；快照清单 {os.path.basename(list_path)}）")
        if len(hits) > SNAPSHOT_CONFLICTS_SHOWN_LIMIT:
            listing.append(f"    （另有 {len(hits) - SNAPSHOT_CONFLICTS_SHOWN_LIMIT} 个）")
    round_names = "、".join(round_name for round_name, _, _ in conflicts)
    valve_lines = "；".join(f"「快照冲突已判：{round_name} <理由>」" for round_name, _, _ in conflicts)
    return 2, (f"✗ 派 {subagent_type} 要改的文件落在还没判完的三方轮 {round_names} 的开工快照里：\n" + "\n".join(listing) + "\n"
               "→ 规矩：.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」：腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。\n"
               f"→ 怎么办：等 {round_names} 的判决写完再派，或把这几处改动并进判决；判过这次派发不碰那一轮的证据（例如只读不改），"
               f"在派发提示里单起一行写{valve_lines}（理由不许空）。"
               + ("\n" + "\n".join(notes) if notes else ""))

LONG_RUN_AGENTS = {"experiment-designer", "experiment-runner"}
LONG_RUN_RULE = ("→ 规矩：预计挂钟超过 2 小时的活默认分片、断点续跑、按多机配置自适应"
                 "（.claude/singlefs-ai-sop/rules/test-script-discipline.md「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」）。\n")
LONG_RUN_LINES = ("超过 2 小时的，同一份提示里写三行：「分片：<分片单元>」「续跑：<同一条命令重起只补没做完的，怎么认做完的>」"
                  "「多机：<台数从 layer0-shard-configuration-check.sh --emit-hosts 读、每台的线程与内存从 host-resources.py --json 现算，不写常量>」；"
                  "分不了的写「分不了：<点名缺的分片 / 续跑 / 多机与为什么，至少 8 个字>」。骨架 python3 research/scripts/dispatch-prompt.py <类型> 带这几行。")

def long_run_verdict(subagent_type, prompt):
    """⑭ 派设计员、执行员：预计挂钟一行，超过 2 小时时的分片、续跑、多机三行（判法在 lib_long_run.py）。交回拒绝说明，放行交 None。"""
    if subagent_type not in LONG_RUN_AGENTS or break_switch_is("long-run-unchecked"):
        return None
    if long_run is None:
        print(f"! 派发闸读不到同目录的 lib_long_run.py（{long_run_error!r}）：预计挂钟与长活声明这一次没判", file=sys.stderr)
        return None
    try:
        wall_clock, declared = long_run.prompt_declaration(prompt)
        if wall_clock is None and break_switch_is("wall-clock-optional"):
            wall_clock = long_run.WALL_CLOCK_WITHIN
        if wall_clock is None:
            return (f"✗ 派 {subagent_type} 的提示里没有「预计挂钟：…」一行（这件活的预计挂钟落在 2 小时内还是外）。\n" + LONG_RUN_RULE
                    + "→ 怎么办：单起一行「预计挂钟：2 小时内」或「预计挂钟：超过 2 小时」（后面可接逗号与推的依据）；" + LONG_RUN_LINES)
        if wall_clock == long_run.WALL_CLOCK_WITHIN:
            return None
        if wall_clock != long_run.WALL_CLOCK_OVER:
            return (f"✗ 派 {subagent_type} 的「预计挂钟：{wall_clock}」认不出：只认以「2 小时内」或「超过 2 小时」开头的写法。\n"
                    "→ 怎么办：改成「预计挂钟：2 小时内」或「预计挂钟：超过 2 小时」，后面可接逗号与推的依据；" + LONG_RUN_LINES)
        problems = long_run.judge(declared, long_run.PROMPT_SPELLING)
    except Exception as error:   # 判法自己出错（LIB_LONG_RUN_BREAK 写错这类）不拒
        print(f"! 派发闸判长活声明出错，这一次没判：{error!r}", file=sys.stderr)
        return None
    if not problems:
        return None
    shown = "\n".join(f"  {problem}" for problem in problems)
    return (f"✗ 派 {subagent_type} 的提示写了预计挂钟超过 2 小时，长活声明不齐：\n{shown}\n" + LONG_RUN_RULE + "→ 怎么办：" + LONG_RUN_LINES)

INPUT_EXAMPLE_FALLBACK = ("禁读清单", "禁读：")   # ③ 拒绝说明里「近义的不算」的例子：缺的名字太短或是 ASCII 时用这一对

def missing_inputs_message(subagent_type, missing):
    """③ 缺定义 required-inputs 的拒绝说明：每组按字面子串找的是哪几个字，再给能照抄进提示的几行（与 dispatch-prompt.py 骨架同一个写法）。"""
    definition = f".claude/agents/{subagent_type}.md"
    if break_switch_is("inputs-hint-dropped"):
        return (f"✗ 派 {subagent_type} 的提示缺它定义要的输入：{'、'.join(missing)}（{definition} 的 required-inputs）。\n"
                f"→ 怎么办：照那份定义「输入（主 agent 必须给）」一节补齐，每一样写出它的名字（同义词用 | 分隔，写一个就算）。")
    groups = [[alternative.strip() for alternative in group.split("|") if alternative.strip()] for group in missing]
    first = groups[0][0]
    example = (first, first[:2] + "：") if len(first) >= 3 and not first.isascii() else INPUT_EXAMPLE_FALLBACK
    found_by = [f"  {alternatives[0]}：找「{'」「'.join(alternatives)}」{'这几个字' if len(alternatives) == 1 else '之一'}" for alternatives in groups]
    copy_lines = [f"{alternatives[0]}：<填 {group}>" for alternatives, group in zip(groups, missing)]
    return (f"✗ 派 {subagent_type} 的提示缺它定义要的输入：{'、'.join(alternatives[0] for alternatives in groups)}（{definition} 的 required-inputs）。\n"
            f"→ 认法：每一组按字面子串在整份提示里找，写在哪一行都算，近义的说法不算（例：要的是「{example[0]}」，写「{example[1]}」不算）：\n"
            + "\n".join(found_by) + "\n"
            f"→ 照抄这几行进提示，把 <填 …> 换成实际内容（骨架 python3 research/scripts/dispatch-prompt.py {subagent_type} 打的就是这几行）：\n"
            + "\n".join(copy_lines) + "\n"
            f"→ 怎么办：每一样该写什么，照 {definition}「输入（主 agent 必须给）」一节。")

def runner_fork_verdict(prompt, project_root):
    """⑪ 执行员：这一段回答的岔路；续做时上一段岔路表里还差的行。交回拒绝说明，放行交 None。"""
    if "只修锚点" in prompt or ("只补落产物" in prompt and not break_switch_is("runner-backfill-refused")):
        return None
    answered = labelled_value(prompt, "这一段回答的岔路")
    if answered is None or answered in NONE_WORDS:
        return ("✗ 派 experiment-runner 的提示里没写「这一段回答的岔路：…」。\n"
                "→ 怎么办：对着岔路单（research/prompts/<轮>-forks.md）写明这一段的量让哪几行够判；"
                "说不出是哪一行，这一段就不该派（.claude/rules/three-way-inference.md「交岔路时写岔路单，派实验时带上它」）；"
                "只补落产物、没有岔路可答的，写「只补落产物」。")
    number_match = re.search(r"research/prompts/e(\d+)-(?:preregistration|r\d+-prereg)\.md", prompt)
    if number_match is None:
        return None
    pages = glob.glob(os.path.join(project_root, ".claude", "kb", "experiments", f"{int(number_match.group(1))}-*.md"))
    if not pages:
        return None
    remaining = labelled_value(prompt, "上一段岔路表里还差")
    if remaining is None:
        return (f"✗ E{number_match.group(1)} 已经有实验页，这是续做，派发提示里却没写「上一段岔路表里还差：…」。\n"
                "→ 怎么办：先读上一段交回的岔路表，把还开着的行点名写进提示；一行都不差就停下交岔路表，不续派。")
    if remaining in NONE_WORDS:
        return (f"✗ E{number_match.group(1)} 上一段岔路表里一行都不差了，不续派。\n"
                "→ 怎么办：按岔路单交岔路表给用户；剩下的量在实验页标「够判后未跑」，状态写「已跑（够判）」。")
    return None

def decide(hook_input, project_root, hook_dir=None):
    """返回 (0, None 或给 stderr 的提醒) 放行；(2, 说明) 拒绝。各条都判完（judge_dispatch），不成立的几条一次列全，每条带自己的出路。"""
    refusals, notes, _ = judge_dispatch(hook_input, project_root, hook_dir)
    if not refusals:
        return 0, notes
    subagent_type = (hook_input.get("tool_input") or {}).get("subagent_type") or "general-purpose"
    return 2, combined_refusal(subagent_type, refusals) + (f"\n{notes}" if notes else "")

def combined_refusal(subagent_type, refusals):
    """几条拒绝合成一份说明：一条的原样交回；几条的先写一共几处，再逐处带序号。"""
    if len(refusals) == 1:
        return refusals[0][1]
    blocks = [f"── 第 {number} 处（共 {len(refusals)} 处）──\n{message}" for number, (_, message) in enumerate(refusals, 1)]
    return (f"✗ 派 {subagent_type} 的提示有 {len(refusals)} 处不成立，各条都判完了，下面一次列全，逐处照各自的「→ 怎么办」改；"
            f"改完可以先跑 python3 research/scripts/dispatch-prompt.py --check <提示文件> --type {subagent_type} 核一遍再派。\n" + "\n".join(blocks))

def heavy_permission_object_counts(clause, match):
    """子句里提到的 checker 档 / --include-ignored / 重型测试算不算要跑的对象：前面紧挨否定（非、不带、不跑、拦……）、后面同一子句里有不跑、归提交、重型这类、
    后面接的是 clippy / build / 编译、「重型测试」后面接闸、钩子、一行的、--include-ignored 前面最近点名的是别的包的，都不算。"""
    word, before, after = match.group(0), clause[:match.start()], clause[match.end():]
    if (HEAVY_PERMISSION_OBJECT_NEGATION_BEFORE.search(before[-6:]) or HEAVY_PERMISSION_OBJECT_NEGATION_AFTER.search(after)
            or HEAVY_PERMISSION_LINT_AFTER.search(after)):
        return False
    if word in ("重型测试", "重测试") and HEAVY_PERMISSION_MENTION_AFTER.search(after):
        return False
    if word == "--include-ignored":
        packages = re.findall(r"(?:-p|--package)[\s=]*(singlefs-[a-z-]+)", before)
        if packages and packages[-1] != "singlefs-checker-tier":
            return False
    return True


def heavy_permission_light_capable_runs(clause):
    """子句里点名轻型那三条而写成重型跑法的地方（HEAVY_PERMISSION_LIGHT_CAPABLE）：同一段命令里（在反引号里的到反引号为止，不在的到子句末）没有 --quick 与
    只查清单的参数（HEAVY_PERMISSION_LIGHT_OR_QUERY）、前面没紧挨否定（不跑、不带、非……）、后面同一子句里没有不跑 / 归提交 / 重型这类的，交那几处的 match。
    弄坏开关 light-tier-commands-unchecked 下一处都不交；light-tier-quick-counted 下带 --quick 的也交。"""
    if break_switch_is("light-tier-commands-unchecked"):
        return []
    runs = []
    for found in HEAVY_PERMISSION_LIGHT_CAPABLE.finditer(clause):
        tail = clause[found.end():]
        if clause.count("`", 0, found.start()) % 2 == 1:
            tail = tail.split("`", 1)[0]
        if HEAVY_PERMISSION_LIGHT_OR_QUERY.search(tail) and not break_switch_is("light-tier-quick-counted"):
            continue
        if HEAVY_PERMISSION_OBJECT_NEGATION_BEFORE.search(clause[:found.start()][-6:]) or HEAVY_PERMISSION_OBJECT_NEGATION_AFTER.search(clause[found.end():]):
            continue
        runs.append(found)
    return runs


def heavy_permission_sentences(prompt):
    """提示正文里许可子 agent 跑 checker 档 / 重型测试的句子（①）：逐句判，交命中的句子。一句里有这几种之一就算：
    许可词（可以、允许、准许、能、可……，前面没紧挨否定）后面八个字以内接跑 / 执行，同一句里又有算数的对象（checker 档、--include-ignored、重型测试、
    起测试而点名 checker 档包的 cargo）；「只跑」与算数的 checker 档同一个子句；「跑 harness 档或 … checker 档」；跑 / 执行（前面没紧挨否定）之后同一子句里
    有起测试而点名 checker 档包的 cargo（`cargo test -p singlefs-checker-tier …`，带 --no-run 的只编不跑、不算）或轻型那三条的重型写法（heavy_permission_light_capable_runs，
    与 cargo 同一种待遇）。
    「」『』“”里的转述、「重型测试：不跑」那一截不判；提到主 agent 的子句不判，句子前面提到主 agent 的那句里的 cargo 不判。弄坏开关 heavy-permission-unchecked 下一句都不交。"""
    if break_switch_is("heavy-permission-unchecked"):
        return []
    hits = []
    for line in prompt.split("\n"):
        text = HEAVY_PERMISSION_LINE_FRAGMENT.sub(" ", HEAVY_PERMISSION_QUOTED.sub(" ", line))
        for sentence in HEAVY_PERMISSION_SENTENCE_SPLIT.split(text):
            permitted = has_object = ordered = False
            start = 0
            # 反引号里的逗号、冒号是命令的一部分（`--features a,b`、`device_log::`），不切
            separators = [found for found in HEAVY_PERMISSION_CLAUSE_SPLIT.finditer(sentence) if sentence.count("`", 0, found.start()) % 2 == 0]
            for separator in [*separators, None]:
                end = separator.start() if separator else len(sentence)
                clause, lead = sentence[start:end], sentence[:start]
                start = separator.end() if separator else start
                cargo_tests = [found for found in HEAVY_PERMISSION_CARGO_TEST.finditer(clause)
                               if "--no-run" not in found.group(0) and not HEAVY_PERMISSION_OBJECT_NEGATION_BEFORE.search(clause[:found.start()][-6:])]
                cargo_tests += heavy_permission_light_capable_runs(clause)
                if HEAVY_PERMISSION_MAIN_AGENT.search(clause) or (cargo_tests and HEAVY_PERMISSION_MAIN_AGENT.search(lead)):
                    continue
                def live(found):
                    return not HEAVY_PERMISSION_NEGATION_BEFORE.search(clause[:found.start()][-4:])
                permitted = permitted or any(live(found) for found in HEAVY_PERMISSION_WORD.finditer(clause))
                has_object = has_object or bool(cargo_tests) or any(heavy_permission_object_counts(clause, found) for found in HEAVY_PERMISSION_OBJECT.finditer(clause))
                tiers = [found.start() for found in HEAVY_PERMISSION_CHECKER_TIER.finditer(clause) if heavy_permission_object_counts(clause, found)]
                if tiers and any(live(found) for found in HEAVY_PERMISSION_ONLY_RUN.finditer(clause)):
                    ordered = True
                if any(live(found) and found.start(1) in tiers for found in HEAVY_PERMISSION_HARNESS_OR_CHECKER.finditer(clause)):
                    ordered = True
                if any(any(live(verb) for verb in HEAVY_PERMISSION_RUN_VERB.finditer(clause[:test.start()])) for test in cargo_tests):
                    ordered = True
            if (permitted and has_object) or ordered:
                hits.append(sentence.strip())
    return hits


def judge_dispatch(hook_input, project_root, hook_dir=None, check_only=False):
    """派发闸的全部判法，各条都判完：→ ([(条名, 拒绝说明)], 给 stderr 的提醒或 None, [没判的几条])。
    check_only（dispatch-prompt.py --check 经 --check-prompt 调）：不读不写实现员登记、不读主会话记录，⑤ 撞文件与 ⑦ 并发、限额窗口交「没判」；
    不是 check_only 时实现员全过才登记。弄坏开关 stop-at-first 只交第一条（退回逐条试的旧样子）。"""
    unjudged = []
    if break_switch_is("disable-check"):
        return [], None, unjudged
    hook_dir = hook_dir or os.path.join(project_root, ".claude", "hooks")
    tool_input = hook_input.get("tool_input") or {}
    subagent_type = tool_input.get("subagent_type") or "general-purpose"
    prompt = tool_input.get("prompt") or ""
    refusals = []
    if (subagent_type not in HEAVY_TEST_OWNERS and not break_switch_is("disable-heavy-test")
            and not HEAVY_TEST_LINE.search(prompt)):
        refusals.append(("重型测试一行", f"✗ 派 {subagent_type} 的提示里没有「重型测试：不跑」一行。\n"
                   "→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、形式化验证（Verus）、全量测试、整轮门禁、全部实验复跑、E152 装置、真编 QEMU）只在提交代码时、或用户要求时跑，"
                   "只由主 agent 跑（.claude/skills/crash-test/commit-stages.md 与 .claude/skills/gate/staged-gate.md）；执行时 .claude/hooks/heavy-test-guard.sh 也拒。\n"
                   "→ 怎么办：提示里单独写一行「重型测试：不跑」；提交时的重阶段与整轮门禁由主 agent 自己跑；提交之外确实要跑，先弹窗问用户，"
                   "用户同意了由主 agent 带 SINGLEFS_HEAVY_TESTS=user-request 跑。"))
    permitting = heavy_permission_sentences(prompt) if subagent_type not in HEAVY_TEST_OWNERS else []
    if permitting:
        shown = "\n".join(f"  「{sentence[:160]}」" for sentence in permitting[:6])
        refusals.append(("许可跑重型测试的句子", f"✗ 派 {subagent_type} 的提示正文里有 {len(permitting)} 句许可它跑 checker 档 / 重型测试（有「重型测试：不跑」一行也照判）：\n{shown}\n"
                   "→ 规矩：子 agent 一律不跑重型测试，checker 档包 singlefs-checker-tier 的测试按包判、快用例与不带 --include-ignored 的也算（.claude/rules/verification.md、"
                   ".claude/hooks/lib_heavy_tests.py 的 classify）；执行时 .claude/hooks/heavy-test-guard.sh 也拒。"
                   "轻型三条（crash-point-replay.sh、checker-tier-formal-verification.sh、checker-tier-crates-mutation-replay.sh "
                   "带 --quick，可再带 --item <项>）不算重型，许可子 agent 跑它们的句子照放（.claude/rules/verification.md「两档：轻型与重型」）。\n"
                   "→ 怎么办：删掉那几句，或改成否定句（「不跑 checker 档，只跑 harness 档」）；要它核的写成「要主 agent 跑的命令」交回，由主 agent 带 "
                   "SINGLEFS_HEAVY_TESTS=commit（用户要求时 =user-request）跑；引用别人写过的原句放进「」里。"))
    if (subagent_type in GENERAL_PURPOSE_TYPES and valve_reason(prompt, "通用已判") is None
            and not break_switch_is("general-purpose-allowed")):
        refusals.append(("内置通用类型", f"✗ 派 {subagent_type}：内置通用类型起步就带项目 CLAUDE.md 与全部规则（约 13 万 token），一次性的活不派它。\n"
                   "→ 怎么办：subagent_type 写 clerk（.claude/agents/clerk.md，输入只要草稿目录与报告；难的活派发参数 model 写 opus）；"
                   "只读的搜索派 Explore；确要派内置类型，提示里单起一行「通用已判：<为什么 clerk 做不了>」。"))
    unfilled = [(number, line.strip()) for number, line in enumerate(prompt.split("\n"), 1) if SKELETON_PLACEHOLDER.search(line)]
    if unfilled and not break_switch_is("placeholders-ignored"):
        shown = "\n".join(f"  第 {number} 行：{line[:100]}" for number, line in unfilled[:8])
        refusals.append(("骨架占位", f"✗ 派 {subagent_type} 的提示骨架还有 {len(unfilled)} 处占位没填：\n{shown}\n"
                   "→ 怎么办：把每处 <填 …> 换成实际内容，不适用的整行删掉；python3 research/scripts/dispatch-prompt.py --check <提示文件> 核到没有占位再派。"))
    fields = agent_definition_fields(project_root, subagent_type)
    if fields is None and not COMMON_CONSTRAINTS_READ.search(prompt) and not break_switch_is("general-purpose-free"):
        refusals.append(("开工先读共用约束", f"✗ {subagent_type} 不是项目定义的 agent，不读共用约束，提示里却没写「开工先读 `.claude/agent-common.md`」。\n"
                   "→ 怎么办：先看 .claude/main-agent.md「什么时候派哪个 agent」那张表有没有对应的定义（改门禁、钩子、脚本、定义派 tooling-writer，"
                   "查真假与机理派 investigator）；确要派它，提示里写一行「开工先读 `.claude/agent-common.md`」，长活照那份的「长活交回主 agent」写。"))
    missing = missing_required_inputs(fields or {}, prompt)
    if missing:
        refusals.append(("定义要的输入", missing_inputs_message(subagent_type, missing)))
    outside = out_of_scope_mentions(project_root, subagent_type, prompt)
    if outside:
        refusals.append(("写范围", f"✗ 提示要 {subagent_type} 写进它写范围之外的路径：{'、'.join(outside)}（.claude/hooks/agent-write-scope.tsv）。\n"
                   "→ 怎么办：改成它写范围里的路径（报告与草稿放工作根 ~/.local/share/singlefs/work/ 下）；范围外的改动交该写的 agent 或主 agent 自己改。"))
    snapshot_code, snapshot_message = snapshot_conflict_verdict(subagent_type, prompt, project_root)
    notes = None
    if snapshot_code:
        refusals.append(("快照冲突", snapshot_message))
    else:
        notes = snapshot_message
    session_judged = not check_only or break_switch_is("check-mode-registers")
    launches, ended, windows = session_dispatch_events(hook_input.get("transcript_path") or "") if session_judged else ({}, {}, [])
    if session_judged:
        model_message = model_verdict(hook_input, prompt, subagent_type, fields, launches, ended, windows)
        if model_message:
            refusals.append(("并发与限额窗口", model_message))
    else:
        unjudged.append("⑦ 并发与限额窗口（派的那一刻按主会话记录判）")
    long_run_message = long_run_verdict(subagent_type, prompt)
    if long_run_message:
        refusals.append(("预计挂钟与长活声明", long_run_message))
    if subagent_type == "kb-scribe":
        spec_message = kb_scribe_spec_verdict(prompt, project_root, hook_dir)
        if spec_message:
            refusals.append(("书记员规格", spec_message))
    if subagent_type == "experiment-designer":
        admission_message = designer_admission_verdict(prompt, project_root, hook_dir)
        if admission_message:
            refusals.append(("重跑登记准入", admission_message))
    if subagent_type == "mutation-triage" and not MUTATION_TABLE_MENTION.search(prompt) and not break_switch_is("mutation-table-free"):
        refusals.append(("变异表", "✗ 派 mutation-triage 的提示里没有变异表（research/mutations/<名>.tsv 或 crates/mutations.tsv 的条目名）。\n"
                   "→ 怎么办：给它表名与条目；没有表的广谱变异是实验，照 .claude/main-agent.md 那一行走 experiment-designer 登记。"))
    writer_files, writer_live_rows = None, []
    if subagent_type == "implementation-writer":
        if not session_judged:
            unjudged.append("⑤ 与本会话在跑的实现员撞文件（派的那一刻按登记判）")
        writer_message, writer_files, writer_live_rows = implementation_writer_verdict(hook_input, prompt, launches, ended, registry=session_judged)
        if writer_message:
            refusals.append(("实现员要动的文件", writer_message))
    if subagent_type == "experiment-runner":
        runner_message = runner_fork_verdict(prompt, project_root)
        if runner_message:
            refusals.append(("执行员岔路", runner_message))
    if break_switch_is("stop-at-first"):
        refusals = refusals[:1]
    if not refusals and writer_files is not None and session_judged:
        register_implementation_writer(hook_input, prompt, writer_files, writer_live_rows)
    return refusals, notes, unjudged

# 2026-09-24 至 09-26 派发闸按旧的逐句判法拒掉的 14 句原句（主会话记录里拒绝消息引的那一句，逐字；截断处照原样）：有了「重型测试：不跑」一行都该放行
OLD_HEAVY_TEST_REFUSALS = [
    "**抓到的实例**：实现员把「库单测 → 动到的测试二进制 → 新层 0 七条流全量 → 变异」写进 `/tmp/claude-1000/impl-m2-treesplit/run-chain.sh`，再用后台 Bash 起它",
    "理由照实写：59 号是重型阶段，抽成共用库之后没法在提交之外跑它来证明没改坏",
    "理由照实写：59 号属于只在提交时才执行的重型阶段，改它的实现在提交之前没有办法证明没改坏，所以两边各留一份",
    "还有一处旧判法要一起看：实二四的报告 `research/prompts/m2-vm-raise-floor-implementer-report.md`「要你定的」第 3 条说，55 号还按「宿主只重跑新池新建文件」判 `second-tr",
    "1. 「理由照实写：59 号属于只在提交时才执行的重型阶段，改它的实现在提交之前没有办法证明没改坏，所以两边各留一份」——这是在写一行说明的理由，没有要谁去跑",
    "- `diff <(bash .claude/scripts/check.sh) x`：括号里真在执行重型测试，对 implementation-writer 照拒",
    "跑门禁 59 号样本、73（gate-lint / shell-lint）、doc-lint、62、63，逐道报原样判定行",
    "前任有一次想在副本上跑全量测试被重型测试闸拒了——你只跑自己写的那几个测试二进制",
    "- **不许跑重型测试**：54 的 `--full`、55（QEMU）、57（herd7 真跑）、59（crates 变异表）、`gate.sh`、`cargo test --workspace` 都不许跑，钩子会拒",
    "- 59 号的红绿样本：重型测试闸按文件名拒绝子 agent 跑 59 号",
    "- **禁跑**：名字带 `layer0` 的测试目标、checker-tier-layer0-replay",
    "- 撤回 F9：门禁分诊员定义里□那几处删掉，恢复直接起 `gate.sh`",
    "第二轮你这一侧的探针与复跑在 `research/prompts/m2-layer0-scale-r2-opus-model/`，可以接着用",
    "只在临时拷贝上跑 `admission.py`、checker-tier-layer0-replay 与 hook 的自证",
]

WALL_CLOCK_WITHIN_LINE = "预计挂钟：2 小时内\n"   # 派设计员、执行员要的那一行（⑭）；判它本身的样本用 FOOTER_WITHOUT_WALL_CLOCK
FOOTER_WITHOUT_HEAVY_LINE = WALL_CLOCK_WITHIN_LINE + "开工先读 `.claude/agent-common.md`\n规格检查已判：自检样本不带规格\n"
FOOTER = "重型测试：不跑\n" + FOOTER_WITHOUT_HEAVY_LINE
FOOTER_WITHOUT_WALL_CLOCK = "重型测试：不跑\n开工先读 `.claude/agent-common.md`\n规格检查已判：自检样本不带规格\n"

def selftest(hook_dir):
    if work_roots is None:
        print(f"  ✗ 自检读不到工作根的定义 {os.path.join(hook_dir, 'lib_work_root.py')}：{work_roots_error!r}")
        print("    → 怎么办：恢复那份库（工作根与过渡期旧根只在那里定义），SINGLEFS_WORK_ROOT_BREAK 只写它认的值，再跑 --selftest")
        return 1
    if long_run is None:
        print(f"  ✗ 自检读不到长活声明的判法 {os.path.join(hook_dir, 'lib_long_run.py')}：{long_run_error!r}")
        print("    → 怎么办：恢复那份库（预计挂钟与长活声明的判法只在那里），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp()
    other_directories = []  # 快照那一条另起的临时目录，finally 里一起删
    state_root = tempfile.mkdtemp()  # 实现员登记放自检自己的状态目录，不碰真会话的
    other_directories.append(state_root)
    os.environ["DISPATCH_GUARD_STATE_ROOT"] = state_root
    try:
        os.makedirs(os.path.join(work, ".claude", "kb", "experiments"))
        open(os.path.join(work, ".claude", "kb", "experiments", "153-样本.md"), "w").write("## E153 样本 —— 部分已跑\n")
        first = "跑前登记：research/prompts/e160-preregistration.md\n"
        again = "跑前登记：research/prompts/e153-preregistration.md\n"
        # 默认给每个样本提示补上「重型测试：不跑」、开工先读共用约束与规格检查放行行，实现员再补一行各不相同的要动文件：
        # 这样各条样本只判它自己要判的那一条；要判那几条本身的样本用 footer 换掉或关掉
        writer_case_counter = [0]
        def case_in(project_root, label, subagent_type, prompt, want, footer=True):
            footer_text = FOOTER if footer is True else (footer or "")
            text = prompt + ("\n" + footer_text if footer_text else "")
            if subagent_type == "implementation-writer" and footer_text and "要动的 crates 文件" not in text:
                writer_case_counter[0] += 1
                text += f"要动的 crates 文件：crates/selftest/src/case{writer_case_counter[0]}.rs\n"
            hook_input = {"tool_name": "Agent", "tool_input": {"subagent_type": subagent_type, "prompt": text}}
            return (label, want, decide(hook_input, project_root, hook_dir)[0])
        def case(label, subagent_type, prompt, want, footer=True):
            return case_in(work, label, subagent_type, prompt, want, footer)
        # 快照那一条的样本仓：r-open 开着（有正文、没判决）；r-closed 已判完；r-nobody 有快照没正文（普查一类）；r-odd 开着，清单里全是认不出的行
        def write_sample(project_root, relative_path, content):
            path = os.path.join(project_root, relative_path)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as sample_file:
                sample_file.write(content)
        digest = "0123456789abcdef" * 4
        write_sample(work, "research/prompts/_r-open-body.md", "# r-open 正文\n")
        write_sample(work, "research/prompts/r-open-snapshot/kb-sha256.txt",
                     f"{digest}  ./.claude/kb/checks-owed.md\n{digest}  ./.claude/kb/decisions/18-块里携带什么信息.md\n")
        write_sample(work, "research/prompts/r-open-snapshot/crates-sha256.txt",
                     f"{digest}  tree/crates/singlefs-core/src/mount.rs\n{digest}  defs/.claude/agents/crash-verifier.md\n")
        write_sample(work, "research/prompts/_r-closed-body.md", "# r-closed 正文\n")
        write_sample(work, "research/prompts/r-closed-snapshot/sha256sums.txt", f"{digest}  .claude/kb/decisions/03-空间分配.md\n")
        write_sample(work, "research/prompts/r-closed-main-verification.md", "# r-closed 判决\n")
        write_sample(work, "research/prompts/r-nobody-snapshot/sources.sha256", f"{digest}  crates/singlefs-core/src/allocator.rs\n")
        write_sample(work, "research/prompts/_r-odd-body.md", "# r-odd 正文\n")
        write_sample(work, "research/prompts/r-odd-snapshot/odd-sha256.txt",
                     f"快照时刻\n# 注释\n{digest}  README.md\n{digest}  research/prompts/r-odd-model.md\nnot-a-hash  .claude/kb/pitfalls.md\n")
        # 另一个样本仓：一份清单读不了（悬空的符号链接），另一份读得了
        unreadable_work = tempfile.mkdtemp()
        other_directories.append(unreadable_work)
        write_sample(unreadable_work, "research/prompts/_r-broken-body.md", "# r-broken 正文\n")
        write_sample(unreadable_work, "research/prompts/r-broken-snapshot/kb-sha256.txt", f"{digest}  .claude/kb/checks-owed.md\n")
        os.symlink(os.path.join(unreadable_work, "不在的文件"), os.path.join(unreadable_work, "research/prompts/r-broken-snapshot/gone-sha256.txt"))
        # 规格文件放在工作根下（不看弄坏开关，configured_work_root），另一份放过渡期旧根下；派发闸只认这两处，自检的 TMPDIR 之外，finally 里删
        new_root, old_root = work_roots.configured_work_root(), work_roots.legacy_root()
        home = os.path.expanduser("~")
        home_new_root = "~" + new_root[len(home):] if new_root.startswith(home + "/") else new_root
        for scratch_root in (new_root, old_root):
            os.makedirs(scratch_root, exist_ok=True)
        specification_directory = tempfile.mkdtemp(prefix="runner-dispatch-guard-selftest-", dir=new_root)
        other_directories.append(specification_directory)
        legacy_specification_directory = tempfile.mkdtemp(prefix="runner-dispatch-guard-selftest-", dir=old_root)
        other_directories.append(legacy_specification_directory)
        legacy_specification_path = os.path.join(legacy_specification_directory, "spec.md")
        write_sample(legacy_specification_directory, "spec.md", "1. 文件：.claude/kb/checks-owed.md\n   旧串：| C120 | …\n   新串：| C120 | …\n")
        specification_path = os.path.join(specification_directory, "spec.md")
        write_sample(specification_directory, "spec.md", "1. 文件：.claude/kb/checks-owed.md\n   旧串：| C120 | …\n   新串：| C120 | …\n")
        missing_specification_path = os.path.join(specification_directory, "missing-spec.md")
        collided = "规格：改 `.claude/kb/checks-owed.md` 里 C120 那一行。\n" + FOOTER
        forwarded_refusal = decide({"tool_input": {"subagent_type": "kb-scribe", "prompt": collided}}, work, hook_dir)[1] or ""
        cases = [
            case("不是执行员", "experiment-designer", "随便", 0),
            case("只修锚点", "experiment-runner", "只修锚点：表 e153", 0),
            case("第一段点名了岔路", "experiment-runner", first + "这一段回答的岔路：岔路 1、岔路 4\n", 0),
            case("第一段没点名岔路", "experiment-runner", first, 2),
            case("第一段岔路写无", "experiment-runner", first + "这一段回答的岔路：无\n", 2),
            case("续做点名了还差的行", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：岔路 1 的崩溃支线\n", 0),
            case("续做没写还差", "experiment-runner", again + "这一段回答的岔路：岔路 1\n", 2),
            case("续做还差写无", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：无\n", 2),
            case("重跑登记也算续做", "experiment-runner", "research/prompts/e153-r2-prereg.md\n这一段回答的岔路：岔路 3\n", 2),
        ]
        # ── 派发提示的结构：重型测试一行、共用约束、定义要的输入、写范围、实现员文件与登记、模型并发与限额窗口、书记员规格、设计员准入、变异表 ──
        defs_work = tempfile.mkdtemp()
        other_directories.append(defs_work)
        write_sample(defs_work, ".claude/agents/three-way-local-attack.md", "---\nname: three-way-local-attack\nmodel: sonnet\nrequired-inputs: 禁读清单, 草稿目录, 提示文件\n---\n")
        write_sample(defs_work, ".claude/agents/three-way-verifier.md", "---\nname: three-way-verifier\nmodel: sonnet  # 注释\nrequired-inputs: 快照, 草稿目录, 报告路径|-verifier-output.md\n---\n")
        write_sample(defs_work, ".claude/agents/experiment-runner.md", "---\nname: experiment-runner\nmodel: opus\nrequired-inputs: 草稿目录\n---\n")
        write_sample(defs_work, ".claude/agents/implementation-writer.md", "---\nname: implementation-writer\nmodel: opus\n---\n")
        write_sample(defs_work, ".claude/agents/mutation-triage.md", "---\nname: mutation-triage\nmodel: sonnet\n---\n")
        write_sample(defs_work, ".claude/agents/clerk.md", "---\nname: clerk\nmodel: sonnet\nrequired-inputs: 草稿目录, 报告\n---\n")
        write_sample(defs_work, ".claude/agents/scope-probe.md", "---\nname: scope-probe\nmodel: sonnet\n---\n")
        write_sample(defs_work, ".claude/hooks/agent-write-scope.tsv",
                     "# 样本\nexperiment-runner\tresearch/results/**\t样本\nexperiment-runner\tresearch/prompts/e*-r*-prereg.md\t样本\nexperiment-runner\t$SINGLEFS_WORK_ROOT/**\t样本\n"
                     "implementation-writer\tcrates/**\t样本\nimplementation-writer\t$SINGLEFS_WORK_ROOT/**\t样本\nscope-probe\tcrates/**\t样本：写范围里没有工作根\n")
        heavy_line = "重型测试：不跑\n" + WALL_CLOCK_WITHIN_LINE
        defs_case = lambda label, subagent_type, prompt, want: case_in(defs_work, label, subagent_type, heavy_line + prompt, want, footer=False)
        for index, sentence in enumerate(OLD_HEAVY_TEST_REFUSALS, 1):
            cases.append(case(f"重型:旧判法误拦的原句 {index} 带上那一行就放行", "experiment-designer", sentence, 0))
        cases += [
            case("重型:没写那一行（实现员）", "implementation-writer", "改完交回。", 2, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:没写那一行（通用 agent）", "general-purpose", "查一个文件。", 2, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:派原来跑重型测试的类型名也要那一行", "crash-verifier", "提交流程里跑 checker-tier-layer0-replay --full。", 2, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:那一行带粗体与列表记号也认", "implementation-writer", "改完交回。\n- **重型测试**：不跑\n", 0, footer=FOOTER_WITHOUT_HEAVY_LINE),
            case("重型:写在句子中间不算那一行", "implementation-writer", "这一件的重型测试：不跑也行吧", 2, footer=FOOTER_WITHOUT_HEAVY_LINE),
            # 提示正文里许可子 agent 跑 checker 档的句子：带着「重型测试：不跑」一行照拒（调度记录 records/2026-09-30-里程碑三开工第一轮.md 第 936 行那两腿的原句；
            # 开关 heavy-permission-unchecked 下前五格红）
            case("重型:正文许可跑不带 --include-ignored 的 checker 档快用例拒（第 936 行正推腿的原句）", "clerk",
                 "可以拷仓副本跑 harness 档或不带 `--include-ignored` 的 checker 档快用例来核（经 research/scripts/run-with-memory-cap.sh，上限 ≤ 12G）；不跑门禁阶段、全量。", 2),
            case("重型:正文把 checker 档单个用例列成可跑的拒（第 936 行攻方腿的原句）", "clerk",
                 "能造反例的造反例：拷仓副本，在副本里改装置、跑 harness 档或小规模的 checker 档单个用例（经 research/scripts/run-with-memory-cap.sh，上限 ≤ 12G）。", 2),
            case("重型:正文要跑 checker 档就只跑快用例拒", "clerk", "要跑 checker 档用例就只跑不带 `--include-ignored` 的快用例，跑不了的写「核不动」。", 2),
            case("重型:正文许可跑 cargo test -p singlefs-checker-tier 拒", "clerk",
                 "装置单测可以跑：`cargo test --release -p singlefs-checker-tier --bin e168_delete_ring_window_borrow` 不带 `--include-ignored`，经内存包装。", 2),
            case("重型:正文直接写跑 cargo test -p singlefs-checker-tier 拒", "clerk", "改完跑 `cargo test -p singlefs-checker-tier --lib verdict_store` 看绿。", 2),
            case("重型:否定句不跑 checker 档放行", "clerk", "不跑 checker 档，只跑 harness 档；checker 档一律不跑。", 0),
            case("重型:checker 档归主 agent 的句子放行", "clerk", "子 agent 不跑 checker 档，checker 档由主 agent 提交时带前缀跑。", 0),
            case("重型:「」里转述的原句放行", "clerk", "两腿提示写过「可以跑不带 --include-ignored 的 checker 档快用例」，这一句不许再写。", 0),
            case("重型:说 checker 档是重型、只跑 harness 档放行", "clerk", "checker 档的测试是重型：你只跑 harness 档与能单独跑的 lib 单测。", 0),
            case("重型:只编不跑与 clippy、build 点名 checker 档包放行", "clerk",
                 "checker 档用例是重型，你跑不了：只编译（`cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --no-run` 一类），"
                 "`cargo build -p singlefs-checker-tier` 与 `cargo clippy -p singlefs-checker-tier --all-targets` 不是测试，可以跑。", 0),
            # 两档里的轻型（.claude/rules/verification.md「两档：轻型与重型」）：三条带 --quick 的许可子 agent 跑放行，重型写法拒
            # （开关 light-tier-commands-unchecked 下拒的三格红，light-tier-quick-counted 下放行的两格红）
            case("重型:正文许可子 agent 跑轻型三条放行", "clerk",
                 "交回前跑轻型：`bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/crash-point-replay.sh --quick --item fixed-script`，"
                 "可以跑 `bash .claude/gate.d/checker-tier-formal-verification.sh --quick .` 与 `bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick .`。", 0),
            case("重型:否定句不跑崩溃点重放的重型跑法、只跑 --quick 放行", "clerk",
                 "不跑 `bash research/scripts/crash-point-replay.sh --full`，只跑 `bash research/scripts/crash-point-replay.sh --quick`。", 0),
            case("重型:正文许可跑崩溃点重放入口不带 --quick 拒", "clerk", "改完可以跑 `bash research/scripts/crash-point-replay.sh --item fixed-script` 看绿。", 2),
            case("重型:正文直接写跑形式化那一道（不带 --quick）拒", "clerk", "改完跑 `bash .claude/gate.d/checker-tier-formal-verification.sh .` 核一遍。", 2),
            case("重型:正文许可跑 crates 变异那一道 --full 拒", "clerk", "允许跑 `bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --full .`。", 2),
            case("重型:harness 档带 --include-ignored 可以跑放行", "clerk",
                 "验收：`cargo test -p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored` 全过（经内存包装，harness 慢档你可以跑）。", 0),
            case_in(defs_work, "共用约束:通用 agent 没写开工先读", "general-purpose", heavy_line + "通用已判：要 WebSearch 查外部文档\n查一个文件。", 2, footer=False),
            case_in(defs_work, "共用约束:通用 agent 写了开工先读", "general-purpose", heavy_line + "通用已判：要 WebSearch 查外部文档\n开工先读 `.claude/agent-common.md`。查一个文件。", 0, footer=False),
            case_in(defs_work, "通用:general-purpose 没写放行行的拒", "general-purpose", heavy_line + "开工先读 `.claude/agent-common.md`。查一个文件。", 2, footer=False),
            case_in(defs_work, "通用:claude 类型同样拒", "claude", heavy_line + "开工先读 `.claude/agent-common.md`。查一个文件。", 2, footer=False),
            case_in(defs_work, "通用:Explore 放行", "Explore", heavy_line + "开工先读 `.claude/agent-common.md`。找文件。", 0, footer=False),
            case_in(defs_work, "骨架:占位没填的拒", "Explore", heavy_line + "开工先读 `.claude/agent-common.md`。\n草稿目录：<填 草稿目录>\n找文件。", 2, footer=False),
            case_in(defs_work, "骨架:占位填完放行", "Explore", heavy_line + "开工先读 `.claude/agent-common.md`。\n草稿目录：~/.local/share/singlefs/work/x/\n找文件。", 0, footer=False),
            defs_case("通用:clerk 有定义放行", "clerk", "草稿目录 ~/.local/share/singlefs/work/c/，报告 ~/.local/share/singlefs/work/c/report.md", 0),
            case("书记员:给判决不给规格放行（写回员自己起草）", "kb-scribe", "判决 research/prompts/r-closed-main-verification.md，条目 C120。", 0,
                 footer="重型测试：不跑\n开工先读 `.claude/agent-common.md`\n"),
            defs_case("输入:本地攻方缺禁读清单（09-23 那一次）", "three-way-local-attack", "提示文件 research/prompts/r-local-attack.md，草稿目录 ~/.local/share/singlefs/work/r/", 2),
            defs_case("输入:本地攻方齐了", "three-way-local-attack", "提示文件 research/prompts/r-local-attack.md，草稿目录 ~/.local/share/singlefs/work/r/，禁读清单：别的腿的产出", 0),
            defs_case("输入:核查员缺草稿目录（09-24 那一次）", "three-way-verifier", "快照 research/prompts/r-snapshot/，报告路径 research/prompts/r-verifier-output.md", 2),
            defs_case("输入:核查员缺快照（09-25 那一次）", "three-way-verifier", "草稿目录 ~/.local/share/singlefs/work/v/，报告 research/prompts/r-verifier-output.md", 2),
            defs_case("输入:核查员同义词认得", "three-way-verifier", "快照 x，草稿目录 ~/.local/share/singlefs/work/v/，写 research/prompts/r-verifier-output.md", 0),
            defs_case("写范围:执行员报告写进 research/prompts（09-26 那一次）", "experiment-runner",
                      "草稿目录 ~/.local/share/singlefs/work/e/\n报告路径：research/prompts/e142-r15-runner-report.md\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:执行员去更新问题单", "experiment-runner",
                      "草稿目录 ~/.local/share/singlefs/work/e/\n更新 research/prompts/m2-rbf1-e142-questions.md 的状态\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:实现员写进 invariants.md", "implementation-writer",
                      "要动的 crates 文件：crates/a/src/one.rs\n顺手写进 .claude/kb/invariants.md 的状态列\n", 2),
            defs_case("写范围:报告写进草稿目录放行", "experiment-runner",
                      "草稿目录 /tmp/claude-1000/e/\n报告写进 /tmp/claude-1000/e/report.md\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:否定句里的路径不判", "experiment-runner",
                      "草稿目录 ~/.local/share/singlefs/work/e/\n不要写进 research/prompts/x-report.md\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:报告写进工作根放行（~ 写法）", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n报告写进 {home_new_root}/e/report.md\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:报告写进工作根放行（展开的绝对路径）", "experiment-runner",
                      f"草稿目录 {new_root}/e/\n报告写进 {new_root}/e/report.md\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:没登记工作根的类型报告写进工作根拒", "scope-probe", f"草稿目录 {home_new_root}/p/\n报告写进 {home_new_root}/p/report.md\n", 2),
            defs_case("写范围:没登记工作根的类型报告写进过渡期旧根拒", "scope-probe", f"草稿目录 {old_root}/p/\n报告写进 {old_root}/p/report.md\n", 2),
            defs_case("变异分诊:没给表", "mutation-triage", "跑 2697 条广谱变异。", 2),
            defs_case("变异分诊:给了表", "mutation-triage", "跑 crates/mutations.tsv 里 m2-rollback 那几条。", 0),
            defs_case("执行员:只补落产物放行", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物：E142 那份。", 0),
        ]
        # 实现员：要动的 crates 文件一行、件数上限、与在跑的实现员撞文件（登记在自检自己的状态目录里）
        state_root = tempfile.mkdtemp()
        other_directories.append(state_root)
        os.environ["DISPATCH_GUARD_STATE_ROOT"] = state_root
        session_work = tempfile.mkdtemp()
        other_directories.append(session_work)
        def session_case(label, subagent_type, prompt, want, transcript_name, model=None, project_root=defs_work):
            tool_input = {"subagent_type": subagent_type, "prompt": heavy_line + prompt}
            if model:
                tool_input["model"] = model
            hook_input = {"tool_name": "Agent", "tool_input": tool_input, "session_id": transcript_name,
                          "transcript_path": os.path.join(session_work, transcript_name + ".jsonl")}
            return (label, want, decide(hook_input, project_root)[0])
        def write_session(transcript_name, records):
            with open(os.path.join(session_work, transcript_name + ".jsonl"), "a", encoding="utf-8") as handle:
                for record in records:
                    handle.write(json.dumps(record, ensure_ascii=False) + "\n")
        def stamp(seconds_ago):
            return datetime.fromtimestamp(time.time() - seconds_ago, timezone.utc).isoformat().replace("+00:00", "Z")
        def launch(agent_id, model, prompt, seconds_ago=600):
            return {"timestamp": stamp(seconds_ago), "toolUseResult": {"status": "async_launched", "agentId": agent_id, "resolvedModel": model, "prompt": prompt}}
        def handed_back(agent_id, seconds_ago=60):
            return {"timestamp": stamp(seconds_ago), "content": f'<agent-message from="{agent_id}">\n[Subagent hand-back] 报告'}
        def notification(agent_id, status, summary="", seconds_ago=60):
            return {"timestamp": stamp(seconds_ago), "content": f"<task-notification>\n<task-id>{agent_id}</task-id>\n<status>{status}</status>\n<summary>{summary}</summary>\n</task-notification>"}
        writer_one = "要动的 crates 文件：crates/a/src/one.rs crates/a/src/two.rs\n"
        write_session("registry", [])
        cases.append(session_case("实现员:第一件登记下来", "implementation-writer", writer_one, 0, "registry"))
        cases.append(session_case("实现员:第二件撞了在跑的那件", "implementation-writer", "要动的 crates 文件：crates/a/src/two.rs、crates/a/src/three.rs\n", 2, "registry"))
        cases.append(session_case("实现员:第三件不撞", "implementation-writer", "要动的 crates 文件：crates/a/src/three.rs\n", 0, "registry"))
        write_session("registry", [launch("a00000000000000a1", "claude-opus-5", heavy_line + writer_one), handed_back("a00000000000000a1")])
        cases.append(session_case("实现员:第一件交回之后不再撞", "implementation-writer", "要动的 crates 文件：crates/a/src/two.rs\n", 0, "registry"))
        cases.append(session_case("实现员:没写要动的文件", "implementation-writer", "改挂载准入。", 2, "registry"))
        cases.append(session_case("实现员:文件多也不拒（份数不设上限）", "implementation-writer", "要动的 crates 文件：" + " ".join(f"crates/b/src/f{n}.rs" for n in range(12)) + "\n", 0, "registry"))
        cases.append(session_case("实现员:带 mutations.tsv 的一件登记下来", "implementation-writer", "要动的 crates 文件：crates/mutations.tsv crates/a/src/five.rs\n", 0, "registry"))
        cases.append(session_case("实现员:mutations.tsv 与在跑的重叠不算撞", "implementation-writer", "要动的 crates 文件：crates/mutations.tsv crates/a/src/six.rs\n", 0, "registry"))
        # 并发：上限加 2 个派出去、1 个交回、1 个失败，在跑的正好是上限（不分模型）；限额窗口：失败通知写着还没到的 resets 时刻
        write_session("busy", [launch(f"a000000000000b{n:02d}", "claude-opus-5-5[1m]" if n % 2 else "claude-sonnet-5", f"活 {n}") for n in range(SUBAGENT_CONCURRENCY_LIMIT + 2)]
                      + [handed_back("a000000000000b00"), notification("a000000000000b01", "failed")])
        write_session("light", [launch(f"a000000000000c{n:02d}", "claude-opus-5", f"活 {n}") for n in range(SUBAGENT_CONCURRENCY_LIMIT - 2)]
                      + [launch("a000000000000c99", "claude-sonnet-5", "别的")])
        future = datetime.fromtimestamp(time.time() + 7200, timezone.utc)
        future_text = f"{future.hour % 12 or 12}{'pm' if future.hour >= 12 else 'am'}"
        write_session("window", [launch("a0000000000000d1", "claude-opus-5", "撞限额的", 1200),
                                 notification("a0000000000000d1", "failed", f"You've hit your session limit · resets {future_text} (UTC) (error type rate_limit, model sent to the API: claude-opus-5)", 600)])  # clock-times:allow 自检造的外部限额通知原文
        write_session("windowlifted", [launch("a0000000000000f1", "claude-opus-5", "撞限额的", 1200),
                                       notification("a0000000000000f1", "failed", f"resets {future_text} (UTC) (model sent to the API: claude-opus-5)", 900),  # clock-times:allow 自检造的外部限额通知原文
                                       launch("a0000000000000f2", "claude-opus-5-5[1m]", "换号之后派的", 300)])
        past = datetime.fromtimestamp(time.time() - 30 * 3600, timezone.utc)
        past_reset = datetime.fromtimestamp(time.time() - 29 * 3600, timezone.utc)
        write_session("windowpast", [notification("a0000000000000e1", "failed", f"resets {past_reset.hour % 12 or 12}{'pm' if past_reset.hour >= 12 else 'am'} (UTC) (model sent to the API: claude-opus-5)", 30 * 3600)])  # clock-times:allow 自检造的外部限额通知原文
        cases += [
            session_case(f"并发:已有 {SUBAGENT_CONCURRENCY_LIMIT} 个在跑再派", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物", 2, "busy"),
            session_case("并发:派 sonnet 一样算", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物", 2, "busy", model="sonnet"),
            session_case(f"并发:{SUBAGENT_CONCURRENCY_LIMIT - 1} 个在跑放行", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物", 0, "light"),
            session_case("并发:没有定义的类型一样算", "undefined-adhoc-type", "开工先读 `.claude/agent-common.md`。", 2, "busy"),
            session_case("窗口:还没到 resets 时刻不派同一族", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物", 2, "window"),
            session_case("窗口:换一族放行", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物", 0, "window", model="sonnet"),
            session_case("窗口:用户定了照派", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物\n限额窗口已判：用户说换号了", 0, "window"),
            session_case("窗口:resets 时刻已过", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物", 0, "windowpast"),
            session_case("窗口:撞限额之后同一族又派出去没再撞，已解开", "experiment-runner", "草稿目录 ~/.local/share/singlefs/work/e/\n只补落产物", 0, "windowlifted"),
        ]
        # 书记员的规格：点名的规格文件交给 kb-spec-check.py（仓根是样本仓 work）
        write_sample(work, ".claude/kb/tooling.md", "# 工具\n\n| 甲 | 旧值 |\n\n## 历史版本\n")
        good_spec = os.path.join(specification_directory, "good-spec.json")
        write_sample(specification_directory, "good-spec.json", json.dumps([{"file": ".claude/kb/tooling.md", "old": "| 甲 | 旧值 |", "new": "| 甲 | 新值 |", "basis": "样本"}], ensure_ascii=False))
        bad_spec = os.path.join(specification_directory, "bad-spec.json")
        write_sample(specification_directory, "bad-spec.json", json.dumps([{"file": ".claude/kb/tooling.md", "old": "| 乙 | 不在的行 |", "new": "| 乙 | 新 |", "basis": "样本"}], ensure_ascii=False))
        no_valve = heavy_line + "开工先读 `.claude/agent-common.md`\n"
        cases += [
            case("规格:点名的规格检查过了", "kb-scribe", f"逐条规格在 {good_spec}，照写。", 0, footer=no_valve),
            case("规格:点名的规格旧串不在", "kb-scribe", f"逐条规格在 {bad_spec}，照写。", 2, footer=no_valve),
            case("规格:一份认得出的规格都没有", "kb-scribe", "照判决写回 C120。", 2, footer=no_valve),
            case("规格:写了放行行", "kb-scribe", "照判决写回 C120。\n规格检查已判：用户口述的定案，没有规格文件\n", 0, footer=no_valve),
        ]
        # 设计员的重跑登记：准入判输入没变（假的 admission.py 退 77）就拒
        for name, code in (("admission-unchanged.py", 77), ("admission-changed.py", 0)):
            write_sample(work, name, f"import sys\nsys.exit({code})\n")
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-unchanged.py")
        cases += [
            case("准入:输入没变不写重跑登记", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 2),
            case("准入:写了放行行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。\n准入已判：换了一条臂\n", 0),
            case("准入:不是重跑登记不判", "experiment-designer", "写跑前登记 research/prompts/e170-preregistration.md。", 0),
        ]
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-changed.py")
        cases.append(case("准入:输入变了放行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 0))
        cases += [
            # 快照：派书记员 / 实现员时要改的文件落在还没判完的三方轮的开工快照里，拒绝
            case("快照:书记员撞开着的轮（清单写 ./.claude/）", "kb-scribe", collided, 2),
            case("快照:实现员撞开着的轮（清单写 tree/crates/）", "implementation-writer", "改 crates/singlefs-core/src/mount.rs 的挂载准入，带测试。", 2),
            case("快照:提示写项目根起的绝对路径、中文文件名", "kb-scribe", f"改 {work}/.claude/kb/decisions/18-块里携带什么信息.md 已定项 11 的射程。", 2),
            case("快照:清单写 defs/.claude/ 也归一", "kb-scribe", "规格：改 `.claude/agents/crash-verifier.md` 的写范围一节。", 2),
            case("快照:规格文件里点名的照拒（路径后面紧跟中文）", "kb-scribe", f"逐条规格在 {specification_path}里，照做。报告写到同一目录。", 2),
            case("快照:规格文件写成 ~ 起头也认", "kb-scribe", f"逐条规格在 {home_new_root}{specification_path[len(new_root):]}，照做。", 2),
            case("快照:过渡期旧根下的规格文件里点名的照拒", "kb-scribe", f"逐条规格在 {legacy_specification_path}，照做。", 2),
            case("快照:路径后面连着别的路径字符不算", "implementation-writer", "对照 crates/singlefs-core/src/mount.rs.orig 那份旧拷贝。", 0),
            case("快照:撞的是已判完的轮，放行", "kb-scribe", "规格：改 `.claude/kb/decisions/03-空间分配.md` 已定项 2。", 0),
            case("快照:有快照没正文的不是三方轮，放行", "implementation-writer", "改 crates/singlefs-core/src/allocator.rs 的分配顺序。", 0),
            case("快照:有放行行，放行", "kb-scribe", collided + "快照冲突已判：r-open 只读它核 C120 的行号，不改那一行\n", 0),
            case("快照:放行行写的是别的轮，照拒", "kb-scribe", collided + "快照冲突已判：r-closed 只读\n", 2),
            case("快照:放行行没写理由，照拒", "kb-scribe", collided + "快照冲突已判：r-open\n", 2),
            case("快照:放行行没写理由、下一行不算它的理由，照拒", "kb-scribe", collided + "快照冲突已判：r-open\n报告写到草稿目录。\n", 2),
            case("快照:放行行理由是占位，照拒", "kb-scribe", collided + "- 快照冲突已判：r-open <理由>\n", 2),
            case("快照:拒绝信息原样转贴，句中的放行模板不算", "kb-scribe", forwarded_refusal, 2),
            case("快照:派别的 agent 放行（设计员）", "experiment-designer", collided, 0),
            case("快照:派别的 agent 放行（实验设计员）", "experiment-designer", collided, 0),
            case("快照:清单里认不出的行不拒", "kb-scribe", "规格：改 README.md、research/prompts/r-odd-model.md 与 `.claude/kb/pitfalls.md`。", 0),
            case("快照:规格文件不存在不拒", "kb-scribe", f"逐条规格在 {missing_specification_path}，草稿目录 {specification_directory}/，报告写 {specification_directory}/report.md。", 0),
            case_in(unreadable_work, "快照:一份清单读不了，不拒", "kb-scribe", "规格：改 `.claude/kb/pitfalls.md`。", 0),
            case_in(unreadable_work, "快照:一份清单读不了，读得了的那份撞了照拒", "kb-scribe", collided, 2),
        ]
        # ⑭ 预计挂钟与长活声明：派设计员、执行员要那一行；超过 2 小时要分片、续跑、多机三行，缺的由「分不了：」点名
        runner_base = first + "这一段回答的岔路：岔路 1\n"
        over = "预计挂钟：超过 2 小时，推的：一千行 × 每行 2 分钟\n"
        two_lines = "分片：按行号 mod 片数切\n续跑：结果表里已有的行跳过，重起同一条命令只补没做完的\n"
        cases += [
            case("长活:设计员没写预计挂钟拒", "experiment-designer", "随便", 2, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:执行员没写预计挂钟拒", "experiment-runner", runner_base, 2, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:预计挂钟写的认不出拒", "experiment-runner", runner_base + "预计挂钟：三个钟头\n", 2, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:超过 2 小时一样都没写拒（不分片的长跑法）", "experiment-runner", runner_base + over, 2, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:超过 2 小时只写了分片拒（半截声明）", "experiment-runner", runner_base + over + "分片：按行号 mod 片数切\n", 2, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:多机一行写死额度拒（写死额度的派发）", "experiment-runner", runner_base + over + two_lines
                 + "多机：本机 31 线程 30G、第二台 31 线程 50G（台数照 --emit-hosts，额度参考 host-resources.py）\n", 2, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:分不了一行没点名缺的多机拒", "experiment-runner", runner_base + over + two_lines + "分不了：装置只认本机那块盘，第二台上没有这块盘\n", 2,
                 footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:分不了一行不到 8 个字拒", "experiment-runner", runner_base + over + two_lines + "分不了：多机不行\n", 2, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:三样齐了放行", "experiment-runner", runner_base + over + two_lines + "多机：台数取 layer0-shard-configuration-check.sh --emit-hosts，每台的线程与内存取 host-resources.py --json 现算\n", 0, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:分不了点名了缺的多机放行", "experiment-runner", runner_base + over + two_lines + "分不了：多机这一段缺：装置只认本机那块盘\n", 0,
                 footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:2 小时内放行（设计员）", "experiment-designer", "预计挂钟：2 小时内，推的：12 条 × 5 分钟\n写跑前登记。", 0, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:那几行带粗体与列表记号也认", "experiment-runner", runner_base + "- **预计挂钟**：超过 2 小时\n- **分片**：按行号 mod 片数切\n"
                 "- **续跑**：已有的行跳过\n- **多机**：台数取 layer0-shard-configuration-check.sh --emit-hosts，每台的线程与内存取 host-resources.py --json 现算\n", 0, footer=FOOTER_WITHOUT_WALL_CLOCK),
            case("长活:派别的类型不判", "implementation-writer", "改完交回。", 0, footer=FOOTER_WITHOUT_WALL_CLOCK),
        ]
        script = os.path.join(hook_dir, "runner-dispatch-guard.sh")
        def run_hook(project_root, stdin_text):
            return subprocess.run(["bash", script], input=stdin_text, capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=project_root))
        # 各条都判完一次列全：缺三样的提示一次报出三样（RUNNER_DISPATCH_GUARD_BREAK=stop-at-first 下红）；
        # 缺输入的说明写明按字面子串认、给照抄的一行（inputs-hint-dropped 下红）；红在别的开关下的写在文件头
        three_missing = decide({"tool_input": {"subagent_type": "three-way-local-attack",
                                               "prompt": "草稿目录 ~/.local/share/singlefs/work/r/\n提示文件：<填 提示文件>\n" + WALL_CLOCK_WITHIN_LINE}}, defs_work, hook_dir)
        cases.append(("一次列全:缺三样的提示一次报出三样（重型测试一行、骨架占位、禁读清单）", (2, True),
                      (three_missing[0], all(fragment in (three_missing[1] or "") for fragment in
                                             ("有 3 处不成立", "没有「重型测试：不跑」一行", "占位没填", "缺它定义要的输入：禁读清单")))))
        cases.append(("输入:缺的输入写明按字面子串认、给照抄的一行", True,
                      all(fragment in (three_missing[1] or "") for fragment in ("按字面子串", "写「禁读：」不算", "\n禁读清单：<填 禁读清单>\n"))))
        # --check-prompt：同一份判法；放行时不登记实现员（登记文件一个字节不变），写明 ⑤ ⑦ 派的那一刻另判（check-mode-registers 下红）
        registry_file = registry_path({})
        def registry_bytes():
            return open(registry_file, "rb").read() if os.path.exists(registry_file) else None
        writer_prompt_path = os.path.join(defs_work, "writer-prompt.md")
        write_sample(defs_work, "writer-prompt.md", "重型测试：不跑\n要动的 crates 文件：crates/a/src/check-mode.rs\n")
        registry_before = registry_bytes()
        checked_prompt = subprocess.run(["bash", script, "--check-prompt", writer_prompt_path, "--type", "implementation-writer", "--root", defs_work],
                                        capture_output=True, text=True, env=dict(os.environ))
        cases.append(("检查模式:--check-prompt 放行时不登记实现员、写明派的那一刻另判的两条", (0, True, True),
                      (checked_prompt.returncode, registry_bytes() == registry_before, "⑤" in checked_prompt.stdout and "⑦" in checked_prompt.stdout)))
        snapshot_refusal = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": collided}}))
        cases.append(("stdin:快照撞了拒绝（退出码 2）", 2, snapshot_refusal.returncode))
        cases.append(("stdin:快照拒绝的 stderr 写了轮名、文件与出路", 1,
                      int(all(needle in snapshot_refusal.stderr for needle in ("r-open", ".claude/kb/checks-owed.md", "等 r-open 的判决写完再派", "快照冲突已判：r-open")))))
        broken_json = run_hook(work, "{不是 JSON")
        cases.append(("stdin:JSON 解析不了放行", 0, broken_json.returncode))
        cases.append(("stdin:JSON 解析不了时 stderr 写了错误", 1, int("JSON" in broken_json.stderr)))
        unreadable_list = run_hook(unreadable_work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": "规格：改 `.claude/kb/pitfalls.md`。\n" + FOOTER}}))
        cases.append(("stdin:快照清单读不了放行", 0, unreadable_list.returncode))
        cases.append(("stdin:快照清单读不了时 stderr 写了是哪一份", 1, int("读不了快照清单" in unreadable_list.stderr and "gone-sha256.txt" in unreadable_list.stderr)))
        for label, prompt, want in (("stdin:续做没写还差", again + "这一段回答的岔路：岔路 1\n", 2),
                                    ("stdin:第一段点名了岔路", first + "这一段回答的岔路：岔路 1\n", 0)):
            completed = subprocess.run(["bash", script],
                                       input=json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "experiment-runner", "prompt": prompt + FOOTER}}),
                                       capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=work))
            cases.append((label, want, completed.returncode))
        heavy = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "implementation-writer",
                                                                            "prompt": "改完交回。\n" + FOOTER_WITHOUT_HEAVY_LINE}}))
        cases.append(("stdin:派实现员没写重型测试一行拒绝（退出码 2）", 2, heavy.returncode))
        cases.append(("stdin:拒绝时 stderr 写了出路", 1, int("→ 怎么办" in heavy.stderr and "弹窗" in heavy.stderr)))
        permission = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "clerk",
            "prompt": "可以跑不带 --include-ignored 的 checker 档快用例。\n" + FOOTER}}))
        cases.append(("stdin:正文许可跑 checker 档拒绝（退出码 2）", 2, permission.returncode))
        cases.append(("stdin:许可句拒绝的 stderr 引了那一句、写了出路", 1,
                      int("可以跑不带 --include-ignored 的 checker 档快用例" in permission.stderr and "→ 怎么办：删掉那几句" in permission.stderr)))
        # 「→ 规矩」那一串列的重型测试带真编 QEMU（拒绝说明里删掉这一项，这一格红）
        cases.append(("stdin:拒绝时 stderr 的「→ 规矩」那一串列出真编 QEMU", 1,
                      int(any(line.startswith("→ 规矩：重型测试") and "真编 QEMU" in line for line in heavy.stderr.splitlines()))))
        long_run_refusal = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "experiment-runner",
                                                                                 "prompt": runner_base + over + FOOTER_WITHOUT_WALL_CLOCK}}))
        cases.append(("stdin:超过 2 小时没写长活三样拒绝（退出码 2）", 2, long_run_refusal.returncode))
        cases.append(("stdin:长活拒绝的 stderr 写了三行的写法与判据出处", 1,
                      int(all(needle in long_run_refusal.stderr for needle in ("分片：", "续跑：", "多机：", "--emit-hosts", "host-resources.py", "→ 怎么办", "用上现有的硬件加速")))))
    finally:
        os.environ.pop("DISPATCH_GUARD_STATE_ROOT", None)
        os.environ.pop("RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT", None)
        shutil.rmtree(work)
        for directory in other_directories:
            shutil.rmtree(directory, ignore_errors=True)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide() 与它调的各条判法（重型测试一行、agent_definition_fields、out_of_scope_mentions、implementation_writer_verdict、model_verdict、"
              "kb_scribe_spec_verdict、designer_admission_verdict、snapshot_conflict_verdict）；RUNNER_DISPATCH_GUARD_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种）：没写「重型测试：不跑」一行的拒（哪种类型都要，旧判法误拦的 {len(OLD_HEAVY_TEST_REFUSALS)} 句原句带上那一行放行），正文许可跑 checker 档的句子拒、否定句与转述放行；"
          "内置通用类型不带「通用已判」放行行的拒，Explore 与 clerk 放行；骨架占位没填完的拒；不读共用约束的类型没写开工先读的拒；定义 required-inputs 缺一组的拒、同义词认得；提示点名写范围外的路径拒、否定句里的不判，"
          "工作根（~ 写法与绝对路径）与过渡期旧根下的报告按写范围表里展开的 $SINGLEFS_WORK_ROOT 判，规格文件在工作根（含 ~ 写法）与旧根下都读；"
          "写回员给判决不给规格的放行；"
          "实现员没写要动的 crates 文件、超上限、与在跑的实现员撞文件的拒，交回之后不再撞；opus 在跑满上限的拒、sonnet 与用户放行行放行，"
          "限额窗口没到的同一族拒、换一族或时刻已过放行；书记员规格检查红或一份规格都认不出的拒；设计员重跑登记准入判输入没变的拒；"
          "变异分诊员没给表的拒；执行员没点名岔路、续做没写还差或还差写无的拒，只修锚点与只补落产物放行；"
          "派书记员或实现员时要改的文件在没判完的三方快照里就拒，已判完的轮、没正文的快照、写了放行行的、派别的 agent 的、认不出的清单行、不在的规格与读不了的清单放行；"
          "派设计员、执行员没写预计挂钟或写得认不出的拒，超过 2 小时而分片、续跑、多机三行不齐（没写、半截、多机写死额度、分不了没点名或太短）的拒，齐了、点名了、2 小时内的放行；"
          "各条都判完、不成立的一次列全，缺输入的写明按字面子串认哪几个字并给照抄的一行；--check-prompt 拿同一份判法核提示、不碰实现员登记")
    return 0

CHECK_PROMPT_USAGE = ("  ✗ 用法：runner-dispatch-guard.sh --check-prompt <提示文件> --type <类型> [--model <模型>] [--root <仓根>]\n"
                      "  → 怎么办：平时由 python3 research/scripts/dispatch-prompt.py --check <提示文件> --type <类型> 替你这样调")

def check_prompt_main(arguments, hook_dir):
    """--check-prompt：派之前拿派发闸同一份判法（judge_dispatch，check_only）核一份提示，不成立的一次列全、退 2，都成立退 0；用法错退 2。"""
    rest, options = list(arguments), {}
    path = rest.pop(0) if rest and not rest[0].startswith("--") else None
    while rest:
        flag = rest.pop(0)
        if flag not in ("--type", "--model", "--root") or not rest:
            path = None
            break
        options[flag] = rest.pop(0)
    if path is None or "--type" not in options:
        print(CHECK_PROMPT_USAGE)
        return 2
    try:
        prompt = open(path, encoding="utf-8").read()
    except OSError as error:
        print(f"  ✗ 读不了提示文件 {path}：{error}\n  → 怎么办：给一个存在的提示文件路径")
        return 2
    project_root = options.get("--root") or os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    tool_input = {"subagent_type": options["--type"], "prompt": prompt}
    if "--model" in options:
        tool_input["model"] = options["--model"]
    refusals, notes, unjudged = judge_dispatch({"tool_name": "Agent", "tool_input": tool_input}, project_root, hook_dir, check_only=True)
    later = f"；派的那一刻另判：{'、'.join(unjudged)}" if unjudged else ""
    if refusals:
        print(combined_refusal(options["--type"], refusals))
    if notes:
        print(notes)
    if refusals:
        print(f"→ 派发闸照这份提示会拒 {len(refusals)} 处（{'、'.join(name for name, _ in refusals)}），照上面逐处改完再核{later}")
        return 2
    print(f"  ✓ 派发闸照这份提示放行（{options['--type']}，查了 {len(prompt.splitlines())} 行提示）{later}")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    if len(sys.argv) > 2 and sys.argv[2] == "--check-prompt":
        return check_prompt_main(sys.argv[3:], hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError as error:
        if not break_switch_is("json-error-silent"):
            print(f"! 派发闸读不懂 hook 的 JSON，这一次放行：{error}", file=sys.stderr)
        return 2 if break_switch_is("json-error-denies") else 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    if work_roots is None:
        print(f"! 派发闸读不到 {os.path.join(hook_dir, 'lib_work_root.py')}（{work_roots_error!r}）：提示里工作根与旧根下的路径这一次没判", file=sys.stderr)
    code, message = decide(hook_input, project_root)
    if message:
        print(message, file=sys.stderr)
    return code

sys.exit(main())
PY
