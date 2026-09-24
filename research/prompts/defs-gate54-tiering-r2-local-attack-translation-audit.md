# 转述核对表：defs-gate54-tiering-r2-local-attack（2026-09-24 UTC）

逐条核对 `research/prompts/defs-gate54-tiering-r2-local-attack.md`（本地攻方：V3、V4 逐格填表）
里每一条 FACT 与真实脚本行为。这一轮的英文事实不是逐句翻译某一句中文，而是对着
`.claude/gate.d/54-layer0-replay.sh`、`.claude/gate.d/stage-inputs.tsv`、
`research/scripts/stage-must-run.sh` 三份真文件的代码行为写的转述；核对方式是把每条 FACT
与对应代码行为并排读一遍，找缺的限定词或写错的机制。行号全部现查（`grep -n` / `sed -n`
+ `cat -n` 现取，2026-09-24 工作区状态，三份文件与 `git status --short` 一致：
`.claude/gate.d/54-layer0-replay.sh` 397 行，sha256
`232c8c0de11bf6561f9ca1736652745f387d05832ad0a5c99a08bc80c0386bf7`，与第三轮实现报告
`research/prompts/defs-gate54-tiering-implementer/r3-report.md` 第 5 行给的 sha256 相同；
`.claude/gate.d/stage-inputs.tsv` 15 行；`research/scripts/stage-must-run.sh` 87 行）。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节与派发消息
「提示用英文写」），只在这份核对表里写死来源文件与行号；核对表与运行记录里，样本自带的
行号（若模型自己写出行号）一律标「模型自给、未核」。

格式：英文项（FACT/ROW 编号）/ 原文文件:行 / 首稿缺的或写错的（核对时发现并已在定稿里
改正的部分） / 定稿理由。

## FACT 1–4（标记文件格式与全量自己的两处独立检查）

| 英文项 | 原文文件:行 | 首稿缺的/写错的 | 定稿理由 |
|---|---|---|---|
| FACT 1：标记文件的完整格式（各字段的先后顺序、LAYER0/CHECKER/LAYER0B 三行紧邻、input_file 逐行） | `.claude/gate.d/54-layer0-replay.sh:376-388`（`echo "# 层 0 全量全绿标记…不进工作树，别手改。"` 到 `sed 's/^/input_file /' "$manifest_at_finish"` 整段 heredoc 式 `{ … } > "$marker_being_written"`） | 首稿把第 377 行 `input_hash=` 误当成文件字面意义上的「第一行」（"the very first line names the combined hash"），漏了第 376 行那条 `#` 开头的人读注释行本身也是标记文件真正的第一行；核对时发现并改写：定稿改成「每一个字段都按自己的行首标签被找到，不按位置」，并把 376 行那条注释单独列成「文件真正的第一行，脚本自己从不读回它」 | 这处不是措辞好看与否的问题：FACT 2 第三步读取 `input_hash` 用的是 `sed -n 's/^input_hash=//p' … \| head -1`（第 260 行），按内容前缀匹配，不按位置——若提示误导模型以为是「文件的第一行」，V3HASH、V3INPUTFILE 两格在需要判断「往标记最前面插一行会不会打乱读取」这类变体时会依据错误的机制推理；改正后 FACT 1、FACT 2、V3HASH、V3INPUTFILE 四处都统一改成「按标签找」的说法（见下方「改动记录」） |
| FACT 2：快档六步读取流程（先算真实文件哈希、再按哈希找标记、比对 input_hash、三行计数、两个 exhaustive=true、读 finished_utc 展示） | `.claude/gate.d/54-layer0-replay.sh:1013-1014`（`write_layer0_input_manifest` 现算哈希）、`:1015`（`full_green_marker_path` 按哈希拼文件名）、`:1019`（`if [[ ! -f … ]]` 无标记判红，真实行号 253）、`:260-261`（`marker_input_hash` 比对）、`:270-272`（三行计数）、`:279-280`（两行 exhaustive=true 计数）、`:288`、`:290`（展示 `finished_utc`）。注：附录里这几行的行号是附录自己拼接后的偏移，本核对表的行号已全部按真实文件重新用 `grep -n`/`awk` 现查，与附录数字不一定相同，以本表为准 | 首稿三处误写「on that marker's own first line」（对应真实第 260 行 `sed -n 's/^input_hash=//p'`）；核对时发现并改成「on the marker's own line that fact 1 says begins with the word input_hash」 | 同上一行，改正理由一致：第 260 行本身就是按标签前缀 `sed` 取值，不是取「第一行」 |
| FACT 3：LAYER0、LAYER0B 两行在全量运行时各自独立检查，没有任何代码把两行合并成一次「计两行」的检查 | `.claude/gate.d/54-layer0-replay.sh:330`（`if [[ "$line" != *"exhaustive=true"* ]]; then` 判 LAYER0）、`:356`（`if [[ "$line_b" != *"exhaustive=true"* ]]; then` 判 LAYER0B），两处是完全独立的两个 if 块，中间隔着两次发布那条流的整段 cargo 调用 | 无遗漏：核对时现查过这两行确实是两个独立的 if 块（`grep -n 'exhaustive=true' .claude/gate.d/54-layer0-replay.sh` 只在 330、356 两行命中裸判断，279 行那处是标记读回阶段的合并计数，与全量自己写入时的判定是两套完全不同的代码） | 这条恰恰是 V3DUPLICATE、V3SCOPE 两格能成立的前提：全量自己从不会产出「两行 LAYER0、零行 LAYER0B」这种标记，所以两行必须分开陈述，不能笼统说成「全量检查两行都要 exhaustive=true」——否则模型会把「全量自己的两处独立检查」和「快档读回时的合并计数检查」搞混，误以为两者是同一段代码 |
| FACT 4：工作线程数检查是全量运行时读实时进度行算出来的，与标记文件无关，且发生在写标记之前；标记写好之后快档从不读回这一行 | `.claude/gate.d/54-layer0-replay.sh:130-134`（`worker_threads_of_full_run` 读 `LAYER0_PARALLEL_FINISHED` 行）、`:137-…`（`worker_threads_are_acceptable`）、`:335`、`:361`（两次调用，各自 `\|\| exit 1`，都在第 374 行标记写入块之前） | 无遗漏：核对时现查过调用顺序（335、361 两行在前，标记写入块从 376 行才开始），确认线程判定确实完全先于标记写入，且失败会在写标记之前就 `exit 1` | 按字面译；这条支撑 V3THREAD 格的核心事实（一旦标记已经写出来，快档再也读不回 worker_threads 那一行） |

## FACT 5–8（登记表格式、复用助手、两处判据不对齐、两问顺序）

| 英文项 | 原文文件:行 | 首稿缺的/写错的 | 定稿理由 |
|---|---|---|---|
| FACT 5：两份脚本各自一份相同的查表逻辑；表里没有这一行时两边行为不同 | `.claude/gate.d/stage-inputs.tsv:1-15`（表格式与「宁宽勿窄」一段）；`.claude/gate.d/54-layer0-replay.sh:56-64`（54 号自己的查表循环）、`:66-70`（无匹配行判红退出 1，真实行号）；`research/scripts/stage-must-run.sh:58-60`（复用助手同一件事：`inputs=…`、`[[ -n "$inputs" ]] \|\| say_run …`） | 首稿一句「does so before it has even decided yet whether it is being run as a quick tier or with the flag called full」写错了：54 号在到达查表这一步之前，命令行参数早已解析完（`layer0_tier` 变量在脚本最开头的 `for` 循环里就已经确定，早于 `ROOT=` 赋值），并不是「还没决定」；核对时发现并改写成「这一处代码在两种模式下都会原样跑到，且发生在脚本里两种模式开始分岔处理之前，因此这个失败与是否带 --full 无关」 | 这不是文字风格问题：如果模型以为「这时候还不知道是哪种模式」，它可能在 V4DELETE 的 REPLAYRUN 里错误地把这一步的判断依据归到「等模式确定之后才失败」，从而误判失败发生的时间点或误判 --full 是否同样会撞上这条红线（实际上两种模式都会，因为这段代码在两者共用的路径上） |
| FACT 6：复用助手的比较对象（上一次整轮全绿的暂存树 vs 这一次的暂存树）与固定多算的两条路径 | `research/scripts/stage-must-run.sh:34`（`REUSE_REF="refs/sop/staged-green"`）、`:55-56`（`green=…`、找不到判「要跑」）、`:65-70`（`input_paths+=` 汇总 + 固定追加 `"$INPUT_TABLE" ".claude/gate.d/$STAGE"`）、`:83-87`（`git diff --quiet` 相同判 skip，不同判 must run）；文件头 `:16-19`（「断言的只有一句『带 --staged 的整轮在这棵树上绿过』」） | 无遗漏（核对后确认）：五层限定——比较对象是树不是提交本身的内容、两条固定追加路径、相同才 skip、不同或任一状态取不到都判 must run——全部译出 | 有意简化一处，未展开：没有单独区分「`refs/sop/staged-green` 只由带 `--staged` 的整轮更新，不由普通 `gate.sh` 更新」这一层（文件头 `:16-19` 有这层区分），因为这一轮 V4 四格问的都是「表这一行被写坏」，不涉及「哪种跑法会前移这条 ref」；这条区分对 V4NARROW/DELETE/WRONGPATH/SELFCHANGE 四格都不承重，略去不影响任何一格的可判定性 |
| FACT 7：54 号自己算哈希用的路径集合，与复用助手比较用的路径集合，两者不是同一个集合（复用助手多算表文件自己与 54 号脚本自己） | `.claude/gate.d/54-layer0-replay.sh:56-71`（`layer0_registered_input_paths` 只来自表里第二字段，从未追加表文件或脚本自己的路径）与 `research/scripts/stage-must-run.sh:70`（`input_paths+=("$INPUT_TABLE" ".claude/gate.d/$STAGE")`）两处并排现查得出的综合结论，不是某一处单独写明的一句话 | 无遗漏，但这条本身是核对时从两处代码比对合成的，不是转述某一句原文；已在正文里如实写成「不是同一个集合」而不是摘引某一行注释 | 这条是 V4SELFCHANGE 格能成立的唯一依据：54 号自己的哈希从不覆盖它自己的脚本内容或表文件内容，而复用助手总是覆盖这两者；没有这条综合事实，V4SELFCHANGE 无法被模型机械地推出结论，只能靠猜 |
| FACT 8：快档必须先后通过两问，第一问答「可跳过」时第二问与 FACT 2 全部步骤都不会被执行 | `.claude/gate.d/54-layer0-replay.sh:79-86`（第一问：调用 `stage-must-run.sh`，`reuse_rc != 0` 判「跳过」退出 77）、`:87-93`（第二问，不属本腿范围） | 无遗漏：顺序（先复用助手、后 change-touches-crates.sh）、第一问跳过时第二问完全不问、退出状态的含义（既非通过也非失败），三层限定全部译出 | 按字面译；「not covered by this leg's assignment」是主动排除给云端腿 V1/V2 的第二问细节，不是遗漏 |

## FACT 9–10（git 路径过滤器行为、combined 文件清单为空时的独立检查）

| 英文项 | 原文文件:行 | 首稿缺的/写错的 | 定稿理由 |
|---|---|---|---|
| FACT 9：拼错或从未存在过的路径交给底层 git 命令时静默匹配零个文件，不报错，也不影响同一条命令里别的正确路径 | 不出自项目任何一份文件的注释——这是 git 本身的通用行为（`git ls-files`/`git diff` 的 pathspec 语义），项目脚本本身从未显式记录这一层；来源是 `.claude/gate.d/54-layer0-replay.sh:159`（`git … ls-files -z … -- "${layer0_registered_input_paths[@]}"`）与 `research/scripts/stage-must-run.sh:83,86`（`git … diff --quiet/--name-only … -- "${input_paths[@]}"`）两处都是把整个路径数组原样交给 git 做路径过滤器，脚本自己没有任何前置校验路径是否存在 | 这一条整个是外加的背景事实，不是转述项目文件里的哪一句话，故单列在下面「多出来的限定词」表里而不是当成「首稿缺的」处理 | 见下方说明 |
| FACT 10：这一处「合并后文件清单为空则判红」的检查在快档与全量里共用同一段代码，不只在快档过完两问之后才会跑到 | `.claude/gate.d/54-layer0-replay.sh:156-166`（`write_layer0_input_manifest` 函数本体，`:164` 是 `(( ${#existing_files[@]} > 0 )) \|\| return 1`）、`:171`（`fail_without_input_manifest`，专属报错文案，与 FACT 5 缺行那条报错文案不同）；调用处：快档 `:1013-1014`（过完两问之后）、全量 `:298` 附近（`manifest_at_start`，全量从不经过 FACT 8 的两问） | 首稿一句「that only ever runs once it has already gotten past both of fact 8's two questions」写错了范围：这段函数是全量与快档共用的同一份代码，全量运行根本不经过 FACT 8 的两问，却同样会跑到这一检查；核对时发现并改写成「快档下必须先过两问才会跑到这里，全量本来就总会跑到这里，因为两问只对快档生效」 | V4 四格的 REPLAYRUN 部分明确限定「the very next time it is actually run as a quick tier」，所以这条overstate 本身不会直接带偏 V4 四格的答案，但作为一条独立陈述的事实它是错的，按核对纪律必须改正，不能因为「反正这轮问题问的是快档」就放过一句本身写错的事实 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定/括注 | 为什么加 |
|---|---|---|---|
| FACT 9 整条 | 不出自任何一份项目文件 | 「一条被拼错或从未存在过的路径在底层 git 命令里静默匹配零个文件，不报错，也不影响同一条命令里别的路径」 | 这是模型推理 V4WRONGPATH、V4NARROW 两格必需的背景知识（54 号与复用助手的脚本注释都从未显式写出这一层 git 语义），不给出这条事实模型只能凭自己对 git 的了解去猜，猜错了就会把这一条腿的可信度建立在它自己未经核实的 git 知识上而不是提示给的事实上；这不是转述某一句中文，是主动外加的必要背景，因此没有放进上面的「首稿缺的/写错的」表，单独列在此处 |
| FACT 7 整条 | `.claude/gate.d/54-layer0-replay.sh:56-71` 与 `research/scripts/stage-must-run.sh:70` 两处并排 | 「54 号自己的哈希计算与复用助手的比较范围不是同一个路径集合」这一句概括性结论 | 原文任何一份文件里都没有这句话本身；它是核对两处代码之后才能看出的综合事实，不摘引也无从摘引某一行，因此单列说明来源是「比对得出」而不是「摘句」 |
| FACT 1「文件开头那条注释行，脚本自己从不读回它」 | `.claude/gate.d/54-layer0-replay.sh:376` | 「脚本自己从不读回它」这一断言 | 原文第 376 行本身只是一行 `echo` 语句，没有任何代码位置显式说「这一行从不会被读回」；这条是核对全文件里所有 `sed -n 's/^…//p'` 读取模式（只匹配 `input_hash=`、`finished_utc=`、`input_file `、`LAYER0`/`CHECKER`/`LAYER0B` 前缀）之后，用排除法确认没有任何一处会匹配 `#` 开头的行，才能下的结论，不是原文字面写出来的 |

## 没做什么（本核对表）

- 未核对分给云端攻方腿的 V1、V2 与分给云端辩方腿的 V5——那两支材料按分工表不归本地腿，
  这份提示本身也没有引用它们，FACT 5、FACT 8 里凡涉及第二问（`change-touches-crates.sh`）
  与 worktree、暂存区的部分，一律只写「不属于这条腿」而不展开机制。
- 未判两次抽样之间答复方向是否一致——那是运行记录与主 agent 的事，不是这份核对表的事。
- 未核对 `research/scripts/stage-must-run.sh` 里与本轮 13 格无关的部分（`--selftest` 分支、
  `SINGLEFS_REUSE_HOURS` 复用上限与 24 小时强制重跑那一段、`SINGLEFS_STAGED_TREE`/
  `SINGLEFS_GATE_FULL` 的取值来源），因为这些机制不改变本轮任何一格的判断，按「引用要射程
  覆盖用得到的部分」的原则不强行译入，以免提示膨胀到超出模型能完整作答的篇幅；这条与
  FACT 6 里「有意简化」那一行是同一类省略，理由已在那一行写明。
