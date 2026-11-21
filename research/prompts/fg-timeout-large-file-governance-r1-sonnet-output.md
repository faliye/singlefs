# fg-timeout-large-file-governance-r1 正推腿报告（第 1 到 4 问）

日期 2026-11-18。读的是主工作区换上之后的三份，sha256 与开工快照 `research/prompts/fg-timeout-large-file-governance-r1-snapshot/judged-sha256.txt` 一致：
`bash-command-detector.sh` b7627f14…、`large-file-check.py` 7889c968…、`agent-common.md` c93670a7…（我开工时现算，三份前 16 位对上）。
没起任何脚本、没跑模型；没写模型，只写了探针脚本（见末尾「要主 agent 跑的命令」），所以没有 `rerun.sh`。
`bash-command-detector.sh` 里别的会话改的 ⑫、⑭、㉓ 不在这一轮判，我没判。本报告引的文件:行都是现读现查的行号，不取自背景材料。

## 问 1：㉑ 一段对 `foreground_timeout_refusal` 与 `main()` 里的调用

结论：**成立，附三处小偏差（打中）**。

逐句：

| 文件头那句（`bash-command-detector.sh` 418-423） | 代码 | 判定 |
|---|---|---|
| 只看前台：「run_in_background 不是 true」 | 「tool_input.get("run_in_background") is True」（`.claude/hooks/bash-command-detector.sh:3089`）：只有 JSON 布尔 true 放行；字符串 "true"、1 都算前台 | 一致 |
| timeout 是数（整数、小数）而超过 240000 | 常量 `FOREGROUND_TIMEOUT_MILLISECONDS = 240000`（3083）；`value <= 常量` 放行（3094）；整数照整数、小数照小数交回（3096）；等于 240000 放行 | 一致 |
| 「只含数字的字符串」 | 「re.fullmatch(r"\s*\d+(?:\.\d+)?\s*", value)」（3092）：前后空白也认，`\d` 连其他文字的数字也认 | 小偏差，见 H1.1 |
| 没写 timeout、布尔、认不出的字符串、不超过的放行 | `bool` 单独排除、非数非可认字符串走 `not isinstance(value,(int,float))`（3094）交 None；"1e6"、"-1" 的字符串认不出，放行 | 一致 |
| 子 agent 的 run_in_background 归 ⑩ | `subagent_background_refusal` 是「bool(agent_type) and run_in_background is True」（3482）；㉑ 里 run_in_background 为 true 直接交 None（3089），两道互不重叠 | 一致 |
| 出路按调用方分：主 agent 改 `run_in_background: true`、结束本轮；跑项目代码的 submit 再起 `task wait <号>`、不设 timeout | `foreground_timeout_messages`（3098）无 `agent_type` 的一支写了这几样，另多写「末行 TASK_SERVER_RESULT」「确要前台跑的，timeout 不超过 240000」 | 一致（代码比文件头多两句，无冲突） |
| 子 agent 写进报告「要主 agent 跑的命令」一节交回 | 有 `agent_type` 的一支写了这句，并指到共用约束「子 agent 不交任务」；另多写「只读的短命令前台跑、timeout 不超过 240000」 | 一致（多一句） |
| 弄坏开关 `allow-long-foreground-timeout`（一概放行） | 3089 的 `detector_break_is_set(...)` | 一致 |

`main()` 里的调用（9215-9221）：

- `long_timeout = foreground_timeout_refusal(hook_input)` 包在 try 里，异常走 `not_judged(...)` 放行并记检出，交 None（9216-9220）；不为 None 才 `refusal_lines.extend(foreground_timeout_messages(long_timeout, hook_input.get("agent_type")))`（9221）。主 agent 没有 `agent_type`，走主 agent 那一支；有 `agent_type` 的全走子 agent 那一支，文件头「主 agent 与子 agent 一样拒」成立。
- **和别的拒绝一起列的次序**：`refusal_lines` 每类两行、按追加次序一块块打印；㉑ 是第二块，在 ⑩（9210、9212）之后、终止进程（9234）之前。其余次序：终止进程 9234 → 放出追踪 9245 → 起 claude 9262 → 单独 & 9272 → 轮询输出 9287 → 前台等待循环 9292 → … → 手拷仓副本 9486 → ㉒ 9498 → ㉓ 9505 → 看门狗 9507。文件头 418-423 没有写次序，agent-common.md 也没写：**这一格「规则没说」**，只能按代码读。
- 退出：有拒绝块就 `return 2`（9533）。

打中（可照喂）：

- **H1.1** `"timeout": " 300000 "`（前后空白的字符串）：代码拒（3092 的 `\s*`），文件头「只含数字」读不出空白也算。探针 T08。
- **H1.2** 400 位整数 `"timeout": 1000…0`（1 后 400 个 0）：`float(value)`（3096，`int(value) if float(value).is_integer() …`）抛 OverflowError，`main()` 的 except（9217-9220）把它当判不了：**放行并记检出，不拒**，文件头「超过 240000 就拒」没有这个例外。我在 python 里现验：`float(10**400)` 抛 OverflowError。探针 T15。（真实 harness 的 timeout 有上限，现实里不会出现；写出来是因为这一格问的是文件头与代码对不对。）
- **H1.3** `"timeout": NaN`（Python 的 `json.loads` 认 NaN，我现验解出 nan）：`nan <= 240000` 为假、`float(nan).is_integer()` 为假，3096 交回 nan，**拒**，说明里写「nan 毫秒」。文件头说「数而超过」才拒，NaN 不超过任何数。探针 T14。

什么现象会推翻「成立」：T02、T03 放行（超过的没拒）；T01 拒（等于门槛不该拒）；T05 拒（后台被拒）；T07 放行（只含数字的字符串没当数）；T04 的说明里没有「要主 agent 跑的命令」或 T03 的说明里没有 `run_in_background: true`。

## 问 2：㉒ 一段对 `git_commit_form`、`git_push_judged`、`git_add_form`、`large_file_findings`

结论：**不成立（有三处代码与文件头不一致，加一处文件头没写的静默放过）**；其余句子成立。三处都给了可喂的输入。

逐句（文件头 424-443 ↔ 代码）：

| 文件头那句 | 代码 | 判定 |
|---|---|---|
| 只有命中 git commit / git push 才起判法；命令文本里没有 git 一词连切词都不切 | 「if "git" not in command:」（`.claude/hooks/bash-command-detector.sh:3258`）直接 `return []`；起判法只在 commit、push 两支（3314-3341 的 `calls`），`add` 只收参数（3280-3284） | 一致 |
| 命令位置上的 git（`-C`、cd、bash -c、命令替换、nice / env 前缀都认） | 取 `shell_words.commands_at_command_position(command, start_directory).commands`，`os.path.basename(words[0]) != "git"` 跳过（3265），`git_global_split` + `git_location` 认 `-C`（2780、2793）；bash -c、命令替换、前缀、`export` 的变量由 `lib_shell_words.py` 切（`environment` 含前面 export 的，`.claude/hooks/lib_shell_words.py:445`）。我没逐项读 lib 里这几层，这一句的 bash -c、命令替换那一半「推不出」，探针 G07d-G07f、G07h 能直接核 | 一致；lib 内部这一半待探针 |
| 不带 -a、不带路径：暂存区相对 HEAD 新增或改过的 blob；`GIT_INDEX_FILE=<私有索引> git commit` 照那份索引判 | 无路径、无 -a、没开关时只 `calls.append(... "commit-index" ...)`（3327-3328）；`GIT_INDEX_FILE` 取 `environment` 里的，规整成相对 `directory` 的绝对路径、写进起判法的环境（3307-3313），只对 commit 带（`kind == "commit"`） | 一致 |
| -a / --all（-am 合写也认）：暂存区加上工作区里相对 HEAD 改过的跟踪文件 | `git_commit_form` 里 `--all` 与短选项字母 `a`（3143、3155）；`include_all` 且无路径时 `calls` 是 commit-index 加 commit-worktree（3323-3324）；`-m`、`-F`、`-C`、`-c`、`-t` 的值跳过（`GIT_COMMIT_SHORT_OPTIONS_WITH_VALUE = "mFCct"`，3118） | 一致 |
| 带路径（`<路径>`、`-- <路径>`、-o / --only）：只判这几个路径的工作区内容，别的已暂存的不进这一笔；加 -i / --include 时暂存区也判 | `paths` 非空时 `calls` 是 commit-worktree 带 `--pathspec`；`include_staged` 才加 commit-index（3319-3322）；`-o`/`--only` 本身不处理（路径照收） | 一致 |
| 同一条命令里排在它前面的 git add 也判（分号、\|\| 连着同样算）；排在 commit 之后的不算 | 按 `steps` 次序走，`git add` 只在遇到它那一步时进 `adds`（3280-3284），commit 那一步用此刻的 `adds`（3329-3336）；因为是按文本次序扫，分号与 \|\| 与 && 同样算，commit 之后的 add 进 `adds` 时前面的 commit 已判完 | 一致 |
| add 点名的路径，或 -A / --all / `.` 要新加的内容，含没跟踪的；add -f 连被忽略的 | `git_add_form`（3186-3217）：`-A`/`--all`/`--no-ignore-removal` 是整棵树，`-u` 不加没跟踪的（`untracked = not update`，3217），`-f` 是 `ignored_too`；调用 commit-worktree 加 `--untracked`、`--ignored-too`（3334-3336） | 一致 |
| 不判：--help、-h、--dry-run、-p / --patch / --interactive 的 commit；--dry-run、-n、--delete、-d、--help 的 push；只读的 git log、git status | `git_commit_form` 的 `skip`（3142 的长选项、3158-3159 的 `letter in "hp"`）；`git_push_judged`（3169-3178）多认 `-h`；`steps` 只收 commit/push/add（3268-3269，除非弄坏开关 `large-file-every-git-command`） | 一致（push 的 `-h` 比文件头多，无冲突） |
| 不带 -a、不带路径的 commit 不会带进没暂存的文件，不拒 | 3325-3328 | 一致 |
| 判不了（放行，往检出追加一条）：命令所在目录算不出 | `directory is None` → `unjudged(...)`（3299-3301） | 一致 |
| 判不了：--git-dir / --work-tree / GIT_DIR / GIT_WORK_TREE 改了仓的位置 | 3304-3306 | 一致 |
| 判不了：判法起不来、超过 LARGE_FILE_CHECK_SECONDS 秒、退 77、退出码不是 0 与 1、输出读不成 JSON、对象库里有读不到的对象 | `large_file_check_run`（3229-3252）：`TimeoutExpired`、`OSError`、读最后一行 JSON 失败、`completed.returncode not in (0, 1)`（3250）都交原因；`unreadable` 走 3348-3349 的 `unjudged(...)` | 一致 |
| 判不了：git push 的当前分支没有上游 | `result.get("not_run")` 非空 → `unjudged`（3346-3347；`--guard push` 无上游时 `not_run` 写原因、退 0，见问 3） | 一致 |
| 判不了：`--pathspec-from-file` | commit 一支：`git_commit_form` 对它置 `unjudged`（3147-3149），非空 → `unjudged(...)`（3291-3293）。**git add 一支不同，见 H2.2** | commit 一支一致；add 一支不一致 |
| git push 只判当前分支相对上游，推别的分支不判 | push 一支只起 `"push"` 一次（3315-3316），不读 refspec | 一致（见 H2.5：`HEAD:other` 仍判当前分支） |
| 拒绝说明逐个文件写出路（最多列 5 个）：gzip -n -9 -k、git rm --cached、git add .gz；已进没推提交的写推之前重做提交 | `large_file_messages`（3358）：`LARGE_FILE_LISTED = 5`（3115）；`where == "没推的提交"` 一支写「重做提交时压成 `gzip -n -9 -k`、`git add <文件>.gz`」并带重做提交那句（3370-3373、3382）；别的文件写三步，「没跟踪」的少 `git rm --cached`（3375）；路径用 `shlex.quote` | 一致（没跟踪少一步、push 那支少 rm --cached，文件头没写，无冲突） |
| 自证把门限压小：环境变量只在自证标记对得上时认 | `selftest_limit_bytes`（3219-3227）：标记文件内容必须等于 `SELFTEST_MARK_CONTENT`；平常不设标记，变量被忽略 | 一致 |
| 上限不另抄常量 | `large_file_check_run` 只加 `--limit-bytes`（3233）当 `limit_bytes` 非空；钩子文件里没有 104857600 | 一致（grep 全仓只有 `scripts/gate/large-file-check.py:66` 一处，见问 3） |

打中：

- **H2.1（误拒，文件头自相矛盾）** 路径形的 commit，前面排着别的路径的 `git add`：文件头 431 写「别的已暂存的不进这一笔」，432-433 却写排在前面的 `git add` 的内容也判。代码（3329-3336）对所有非 push 的判过的 commit 都把 `adds` 并进去，**包括带路径的**：`git add new.bin && git commit -m x small.txt` 真 git 只提交 small.txt、new.bin 进不了这笔提交，钩子仍拒。多拒，不漏拒。探针 G03e。
- **H2.2（静默放过，文件头说记一条）** `git_add_form` 对 `--pathspec-from-file` 置 `skip`（3197）、`judged` 为假，那一条 add 整个不进 `adds`、**不调 `unjudged`，也就不记检出**；`git add --pathspec-from-file=list.txt && git commit -m x` 放行且无记录。同样，commit 的路径有 `$`、反引号时判不了并记（3294-3296），**add 的路径没有这道检查**：`git add "$BIGFILE" && git commit -m x`、`git add $(ls *.bin) && git commit -m x` 把字面 `$…` 当 pathspec 传给判法，匹配不到任何文件，放行、不记。文件头 436-438 的「判不了的…往检出记录追加一条」读不出把 add 排除在外。探针 G03f、G03g、G03h。
- **H2.3（静默放过）** 目录算得出、但此刻不存在（例：`mkdir -p d && cd d && git init && git add big && git commit -m x`，或 `cd /nonexistent && git commit`）：3302-3303 `if not os.path.isdir(directory): continue`，不记。文件头 436 只写「目录算不出」才记，没有「算得出而不存在」这一格。探针 G07b。
- **H2.4（文件头没写的判不了）** 代码还有三处「判不了并记」文件头没提：commit 路径写成变量或命令替换（3294-3296）、`GIT_INDEX_FILE` 写成变量（3310-3312）、排在前面的 `git add` 目录算不出或改了仓位置（3331-3333）。只是文件头少写，不是行为错。
- **H2.5（说法不精）** 「推别的分支不判」：`git push origin HEAD:other`、`--all`、`--mirror` 不读 refspec（3315-3316），一律判当前分支相对上游的提交；推的是别的分支时，判的内容与真推出去的不一样（可能多拒，也可能漏判被推的那条分支上的大文件）。文件头 438 的话本身成立，但「别的分支」那半句没说判的是什么。探针 G06d-G06f。
- **H2.6（文件头没写）** 别名（`git ci`、`git -c alias.ci=commit ci`）与长选项缩写（`--inc` 当 `--include`）：`git_global_split` 取到的子命令不是 commit/push/add 就跳过（3268-3269），选项名只认全名（3142-3151 一带），**静默放过、不记**。文件头没列这两种，「规则没说」；是不是缺口留给攻方腿。探针 G07g。

什么现象会推翻「不成立」：G03e 放行（则 H2.1 不成立）；G03f、G03g、G03h 的新增检出各为 1 条（则 H2.2 不成立）；G07b 新增检出 1 条（则 H2.3 不成立）。反过来，G01d 拒、G01e 放行、G02 拒、G03d 拒则文件头对应句不成立。

H2.1 到 H2.3 在钩子自证里没有对应格：我读了 8566-8670 一段里「提交前大文件」各样本的标签，只有 `--help`（8651）与 `--dry-run`（8652）的不判格，没有 `-i`、`--pathspec-from-file`、add 点名变量路径、带路径加前置 add、不存在的目录这几种（grep 这几个词在该段 0 命中）。

## 问 3：`--guard` 三种模式、退出码与文件头第 26 到 39 行；上限是不是一份

结论：模式一句一句**成立**；上限只有一份、`exceeds` 是同一个函数，**成立**；退出码那一段**不成立**（文件头的码表与实际可退出的码对不上，三处打中）。

逐句（`scripts/gate/large-file-check.py` 26-39 ↔ 代码）：

| 文件头 | 代码 | 判定 |
|---|---|---|
| 26 `<目录> --guard <模式> [--pathspec <路径>]… [--untracked] [--ignored-too] [--limit-bytes N]` | `main_guard`（395-424）认这五种；`--pathspec` 可重复（`pathspecs.append`）；`--pathspec`、`--untracked`、`--ignored-too` 只在 commit-worktree 下认，别的模式给了退 2（417） | 一致 |
| 28 标准输出只有一行 JSON，键 mode、limit、limit_text、checked、too_big（path、size、where、commit、title）、unreadable、not_run、error；退出码同下 | `print(json.dumps(result))` 只此一处（423）；`guard_result`（320-323）八个键；`too_big` 项五个键（339、373） | 一致 |
| 29 目录是仓里任一目录，`--pathspec` 相对它起算，不必是顶层 | `guard`（377-392）先 `rev-parse --show-toplevel`、`os.path.realpath`；`guard_worktree` 有路径时 `base = directory`（346），git 的输出路径相对仓顶层再接 `top` 取大小（359-362） | 一致 |
| 30 commit-index：暂存区相对 HEAD 新增或改过的 blob（没有 HEAD 就是整个暂存区），where 是「暂存区」 | `guard_staged`（325-340）：`raw_entries(top, "--cached", "HEAD")` 去掉 `D` 与子模块（327-329），无 HEAD 取 `tracked_entries(top)`（331），`where: "暂存区"`（339） | 一致 |
| 31-33 commit-worktree：工作区相对 HEAD 改过的跟踪文件取工作区普通文件的大小；`--pathspec` 限路径、不给整棵树；`--untracked` 另算没跟踪、没被忽略的；`--ignored-too` 连被忽略的 | `guard_worktree`（343-364）：`raw_entries(base, "HEAD", …)`（348）、`S_ISREG` 才算（362，软链接不算）、`--others --exclude-standard` 与去掉 `--exclude-standard`（354-355）；`where` 是「工作区」（352）、「没跟踪」（355） | 一致（`--ignored-too` 单独给、不带 `--untracked` 时无效，353 的 `if untracked:`，文件头没写） |
| 34 push：当前分支相对上游没推的提交里新增或改过的 blob；没有上游 not_run 写原因、退 0 | `guard_unpushed`（367-374）：无上游 `not_run=why_not`（371），`guard` 末行 `1 if too_big else 0`（392）→ 退 0；有上游走 `judge_unpushed`，`where: "没推的提交"`（373） | 一致 |
| 35 暂存区的认法跟着 `GIT_INDEX_FILE`；上限与「超过」同不带 --guard 时一份 | `git()` 起 subprocess 不传 `env`，继承环境，git 自己认 `GIT_INDEX_FILE`（80）；上限见下 | 一致 |
| 38-39 退出码：0 都不超；1 有超的；2 用法错；77 根不是 git 仓的顶层（--guard：目录不在 git 仓里）、或既没有跟踪文件也没有待检查的未推送提交；3 只在 --guard 下：git 没跑成 | 0、1：392；2：`main_guard` 的 416、420；77：`guard` 381；3：`guard` 的 `except GitFailure`（390-391） | 见 H3.1 到 H3.3 |

上限只有一份：

- `SINGLE_FILE_SIZE_LIMIT_BYTES = 104857600` 只在 `scripts/gate/large-file-check.py:66`；我在 `scripts`、`.claude`、`task-server`、`prune-server`、`crates` 下 grep `104857600`、`100 * 1024 * 1024` 这几种写法，除 fixtures 外别处的 100 M 都是别的量（内存上限、测试用的环）。`main` 与 `main_guard` 都从它起算 `limit`，`--limit-bytes` 是覆盖参数（401-402、434），钩子只在 `selftest_limit_bytes()` 非空时传（`.claude/hooks/bash-command-detector.sh:3233`）。
- 「超过」比较只经 `exceeds`（89-90）：调用点 151（甲）、220（乙）、338（commit-index）、362（commit-worktree），四处同一函数；147 的 `status.st_size > size` 是取两份里大的那个、不是比上限。

打中：

- **H3.1** `--limit-bytes ²`（上标 2，`str.isdigit()` 为真）：401 行 `rest[0].isdigit() and int(rest[0]) > 0` 里的 `int("²")` 抛 ValueError，没人接；Python 退 1、标准输出没有 JSON。文件头 38 写「1 有超的」，实际是崩溃也退 1。钩子拿到 1 但读不出 JSON，走「判不了」记检出放行（`.claude/hooks/bash-command-detector.sh:3247`），没出事；只是码表有歧义。探针 Q14。
- **H3.2** 码表缺 78：文件头第 3 行声明 `run-condition: command git`，开头 `preflight(__file__)`（477），git 不在 PATH 时 `preflight` 退 78（`.claude/singlefs-ai-sop/scripts/preflight.py:620` 的 `EXIT_REFUSED`）；`preflight` 的声明写坏退 1（同文件 606）。码表里没有这两个。另 `settle_break_switches`（461）在非样本目录设了 `LARGE_FILE_CHECK_BREAK` 退 2，不是「用法错」，说明在文件头 41 行起的弄坏开关一段。钩子侧 78 也走「退出码不是 0 与 1」放行并记（`.claude/hooks/bash-command-detector.sh:3250`）。探针 Q16、Q17。
- **H3.3** 「77 … 或既没有跟踪文件也没有待检查的未推送提交」：`--guard` 下没有这条出口。`guard`（377-392）只有 `top is None` 一处 77（381）；空仓、没有提交的仓都退 0。这半句只对不带 `--guard` 的 `judge`（`return 77` 在不带 guard 的判法里）成立，写在同一句里读成两种模式都适用，歧义。探针 Q20、Q21（预期 Q20 退 1、Q21 退 0 且 not_run 非空）。
- **H3.4（说法不精）** 77 的触发是 `git rev-parse --show-toplevel` 失败（380），不只是「不在仓里」：裸仓、`.git` 目录里、git 报 dubious ownership 也退 77、not_run 写「不在 git 仓里」。钩子一律当判不了，没后果。

什么现象会推翻「模式成立」：Q01 不退 1、Q02 里出现 untracked.bin、Q03 里没有 untracked.bin、Q04 里没有 x.ign、Q08 里 where 不是「没推的提交」、Q09 退 1、Q18 退 1（等于上限不该超）或 Q19 退 0。推翻「上限一份」：全仓 grep 再出现一份 104857600，或 `exceeds` 之外的比较。

## 问 4：`.claude/agent-common.md` 第 70、75、96、97 行与钩子今天的行为；用户要的两件

结论：70、96 **成立**；75 里 ㉑ 的那半 **成立**、② 的那半**不成立**（不是这一轮改的）；97 **大体成立，一处不精**。

| 行 | 句子 | 钩子今天 | 判定 |
|---|---|---|---|
| 70 | 主 agent 与子 agent 的前台命令的 `timeout` 都不超过 240000 毫秒（超过的执行前拒，「执行前拒绝的写法」㉑） | 3083、3094、9221、9533：超过拒、退 2；子 agent 与主 agent 同拒 | 成立 |
| 70 | 主 agent 要跑得久的活用 `run_in_background` 起、结束本轮等完成通知，子 agent 的交回 | `foreground_timeout_messages` 两支（3098） | 成立 |
| 70 | 「超过的记一条检出」已删（`git diff HEAD` 里这句是 `-` 行） | `subagent_timeout_findings` 全仓 grep 0 命中 | 成立 |
| 75 | 「② ㉑ 只在前台拒」 | ㉑ 只拒前台（3089）；**② 不是只在前台拒**：run_in_background 里轮询本会话后台任务输出的循环也拒（`.claude/hooks/bash-command-detector.sh:9286-9288` 的 `if loops and tool_input.get("run_in_background") is True:` 那一支，agent-common.md 第 77 行 ② 自己也写了「run_in_background 里轮询…都拒」） | ㉑ 成立；② 不成立（`②` 在括号里原来就在这一类，不是这一轮加的，只加了 ㉑） |
| 75 | 「④ ⑩ 只在 run_in_background 里拒」 | ⑩ 是 `run_in_background is True`（3482）；④ 文件头写「前台的 `… &` 不拒」 | 成立 |
| 75 | 「其余两处都拒」 | ㉒（`large_file_findings` 不看 `run_in_background`、不看 `agent_type`）、㉓ 前台后台一样 | 成立 |
| 96 | ㉑ 前台命令的 timeout 超过 240000；run_in_background 不判，子 agent 的归 ⑩；出路主 agent 改 `run_in_background: true`、submit + 后台 `task wait`；子 agent 写进报告交回 | 3085-3096、3098-3112 | 成立 |
| 97 | ㉒ 主 agent 与子 agent 一样拒；commit 判这笔提交要进的暂存区内容（`-a`、点名的路径、同一条命令里排在前面的 `git add` 另判工作区内容），push 判当前分支相对上游没推的提交 | `-a`：暂存区加工作区（3323-3324）；点名的路径：**只判工作区，暂存区不判**，`-i` 才加暂存区（3319-3322）；前置 add：工作区（3329-3336）；push（3315-3316） | 大体成立；路径那一格不精，见 H4.1 |
| 97 | 判不了的放行并记一条检出 | 3274-3277、3299-3349 | 大体成立；H2.2、H2.3 的静默放过没记 |
| 97 | 出路逐个文件：gzip -n -9 -k、git rm --cached、git add .gz；读它的地方改成认 .gz；引用照 path-moves.md 全仓改名；已进没推的提交的，推之前重做提交 | `large_file_messages`（3358-3384） | 成立 |

- **H4.1** 97 行把 `-a`、点名的路径、前置 `git add` 并列在「暂存区内容…另判工作区内容」下，读成这几种形式下暂存区都判；代码里点名路径时暂存区**不判**（3319-3322，除 `-i`），与文件头 431「别的已暂存的不进这一笔」一致，97 行读不出来这个例外。后果是读 97 行的人以为 `git add big && git commit small.txt` 之外的路径提交会判暂存区。
- **用户要的两件各落在哪一行**（用户两句话见 `records/2026-11-18-前台超时与提交时大文件两道闸.md:7` 的课题；第二句里的「压缩」二字下面单说）：
  - 件一，拦截前台 timeout 600000 的写法：钩子 418-423、3083-3112、9215-9221；agent-common.md 第 70 行（规则）、第 96 行（㉑ 列项）、第 75 行括号里的 ㉑。
  - 件二，提交时大文件：钩子 424-443、3114-3384、9492-9498；agent-common.md 第 97 行（㉒ 列项）。
- **多做的**（都是这一轮代码里有、用户两句话字面里没有的）：① 对 `git push` 也判（3297、3315-3316、`guard_unpushed`）；用户第二句说「提交时候」，课题 7 行写的是「两次进了提交、推不上去」且「直接起的 `git commit` / `git push` 不经过它」，push 可算同一件事，但是多出来的一支。② `-i`、`--only`、`GIT_INDEX_FILE`、前置 `git add` 链、`-a` 这一串精细判法（3128-3217、3307-3341）：只为不误拒、不漏拒，不是用户字面要的。③ ㉑ 对子 agent 也拒、并写了子 agent 一支的出路（3102-3105）；课题 24 行的数据里子 agent 有 66 次 600000，要这样做有依据，但用户字面说的是「那条前台命令」。④ 删掉了原先 ③ 里只记子 agent 的 timeout 检出那一支（`subagent_timeout_findings` 0 命中），agent-common.md 第 70 行的「记一条检出」同删。⑤ agent-common.md 本轮 `git diff HEAD` 里还有别的会话的几处改动（第 4 行、第 44 行、第 89 行 ⑭、第 103 行第十一道），不在这一轮，我没判也没算多做。
- **用户第二句的「压缩」推不出（算打中）**：用户原话「大文件压缩的钩子在提交时候没有生效」，仓里没有一个会自动压缩的钩子：`.git/hooks/pre-commit` 只是提交窗口闸，门禁 code-tooling 的 committed-file-size 格（`.claude/gate.d/code-tooling.sh:5716`）只判不压，㉒ 也只拒、把 `gzip -n -9 -k` 写进出路，不替用户压。课题 7 行把这句读成「在执行前拒」；用户要的是「提交时自动压缩」还是「提交时拒并给出路」，从仓里的文字推不出，要用户确认。可照喂：问用户「提交时遇到超限文件，要钩子自动压成 .gz 并改暂存区，还是只拒并给命令」。

什么现象会推翻问 4 的结论：钩子对 `timeout` 超过 240000 的前台调用放行（推翻 70、96）；`git commit <路径>` 在没有 `-i` 时判了暂存区（推翻 H4.1）；`run_in_background: true` 的 timeout 600000 被 ㉑ 拒（推翻 75、96）。

## 要主 agent 跑的命令

探针脚本都在草稿目录 `/home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet/probes/`，我只做了 `bash -n`（五个文件都过），一次没跑。每个脚本的第一个参数是**还不存在的新运行目录**（已存在就退 2、不删任何东西）；检出记录写进 `<运行目录>/detections.jsonl`（`AGENT_HOOK_DETECTIONS`），不碰真实的检出记录。钩子与判法读的是主工作区 `/home/fy5090/code/singlefs` 的换上版（`REPO_ROOT` 默认值）。每格输出 `CASE <号> | 预期：… | rc=… | 新增检出 N 条`，预期写在脚本里，对不上的就是打中。

文件 sha256：

```
e63ebda7f505ace729bd5a51bfeb7cb9a7eb8c6b4b247f41ff676ef4131d3c2e  probes/mkjson.py
b0fc14d642dafba7d703fe8fada8c98bd3477fa1e2cd29994e7ef92874c1925b  probes/probe-lib.sh
f0fc4697a516b68f4ecb89c50f316819f830f1f2f6615caa7df4bbc392cf7256  probes/probe-timeout.sh
995256375a1e8a62ab8b318f85fa159af049eecb93986c785f76ad04f1fb9787  probes/probe-hook-git.sh
8e2360a0fbb5016adf366b62e6b94482766d517ff06f51251e6823a15f20d7f2  probes/probe-guard.sh
```

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet -- bash /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet/probes/probe-timeout.sh /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet/run-timeout-1
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet -- bash /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet/probes/probe-hook-git.sh /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet/run-hook-git-1
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet -- bash /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet/probes/probe-guard.sh /home/fy5090/.local/share/singlefs/work/fg-timeout-large-file-governance-r1-sonnet/run-guard-1
```

资源与时间（推的，没测）：每格起一次钩子（python 解释一份约 9500 行的脚本）与最多几次 `large-file-check.py`；`probe-timeout.sh` 18 格、`probe-hook-git.sh` 约 45 格、`probe-guard.sh` 23 格，预计各在几分钟内跑完，内存几百 MB 以内；磁盘上每个临时仓有一个 100 MiB 的稀疏文件（实占接近 0，`git add` 后对象约百 KB）。读完输出后运行目录由主 agent 删（我没写删除；目录在工作根下、归属文件见草稿目录）。

每个探针要读的格（对应打中项）：

| 打中项 | 探针格 | 预期（文件头与代码给的） |
|---|---|---|
| H1.1 | T08 | 拒（`\s*` 认空白）；若放行则 H1.1 不成立 |
| H1.2 | T15 | rc=0 且新增检出 1 条（OverflowError 被 `main()` 当判不了） |
| H1.3 | T14 | rc=2，说明里写 nan |
| 问 1 次序 | T04、T17 | T04 只一块（㉑ 子 agent 支）；T17 只一块（⑩）；两类同踩的次序要另造一条同时有 `run_in_background` 与前台等待循环的输入才看得到，探针里没造 |
| H2.1 | G03e | rc=2（误拒）；放行则 H2.1 不成立 |
| H2.2 | G03f、G03g、G03h | rc=0、新增检出 0 条；有 1 条则 H2.2 不成立 |
| H2.3 | G07b | rc=0、新增检出 0 条 |
| H2.5 | G06d、G06e、G06f | rc=2（判当前分支相对上游）；看有没有说明提到推的是别的分支 |
| H2.6 | G07g | rc=0、新增检出 0 条 |
| 问 2 其余 | G01 到 G07 其余格 | 预期写在每格的第二个字符串里；G07d-G07f、G07h 核 bash -c、命令替换、`-C`、`command git` 都认 |
| H3.1 | Q14 | 退 1、stdout 行数 0、stderr 有 ValueError |
| H3.2 | Q16、Q17 | Q16 退 78；Q17 退 2 |
| H3.3 | Q20、Q21 | Q20 退 1；Q21 退 0 且 not_run 非空 |
| 问 3 模式 | Q01 到 Q13、Q18、Q19、Q22、Q23 | 预期写在每格里 |
| 问 4 H4.1 | G01d、G01e | G01d 放行、G01e 拒 |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问 1 ㉑ 逐句 | 成立（三处小偏差） | 放行/拒、出路、开关对得上；空白字符串、400 位整数（放行记检出）、NaN（拒）与文件头字面有出入 |
| 问 1 次序 | 规则没说 | 文件头没写次序，代码是 ⑩ → ㉑ → 终止进程 … → ㉒ → ㉓ → 看门狗 |
| 问 2 ㉒ 逐句 | 不成立 | 带路径 + 前置 add 误拒（H2.1）、add 的 `--pathspec-from-file` 与变量路径静默放过（H2.2）、算得出而不存在的目录静默放过（H2.3） |
| 问 2 出路字面 | 成立 | 三步命令、重做提交、最多列 5 个、`shlex.quote` 都对得上 |
| 问 2 别名、长选项缩写 | 规则没说 | 静默放过且不记，文件头没列（H2.6） |
| 问 3 三种模式 | 成立 | 逐句对得上，JSON 键、where 取值、`--ignored-too` 单独无效（文件头没写） |
| 问 3 上限一份 | 成立 | 只在 `scripts/gate/large-file-check.py:66`；比较只经 `exceeds`（89-90） |
| 问 3 退出码 | 不成立 | 码表漏 78、崩溃也退 1、`--guard` 下没有「空仓退 77」（H3.1 到 H3.3） |
| 问 4 第 70、96 行 | 成立 | 与 ㉑ 逐句对上 |
| 问 4 第 75 行 | ㉑ 成立；② 不成立 | ② 在 run_in_background 里轮询输出文件也拒（原有的，不是这一轮加的） |
| 问 4 第 97 行 | 大体成立 | 路径形不判暂存区，97 行读不出（H4.1）；「记一条检出」有静默例外 |
| 用户的两件 | 落点清楚 | 件一：418-423、3083-3112、70/96/75 行；件二：424-443、3114-3384、97 行 |
| 多做的 | 有 | push 一支、`-i`/`GIT_INDEX_FILE`/add 链等精细判法、子 agent 一支、删 ③ 的子 agent 检出支 |
| 「压缩」 | 推不出 | 仓里没有自动压缩的钩子，要不要自动压缩要用户确认 |

## 没做什么

- 不判别的腿的格（第 5 到 11 问：浮点之外的 timeout 变体对别的道的撞法、漏拒的写法穷举、拖慢与时限、判法出错的各路、出路过别的闸、九条开关的重叠、多机）；这几格只在上面顺带提到的地方碰过，没有结论。
- 不替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数没复跑，探针一个没跑（命令在「要主 agent 跑的命令」一节）。
- `lib_shell_words.py` 里 bash -c、命令替换、`nice`/`env` 前缀、cd 目录算法的内部没读，问 2 第二行对应的那一半标「待探针」。
- 钩子自证里「前台超时:」与「提交前大文件:」两组格只读了标签（8566-8670 一段），没逐格核它们的期望与开关。
- 没写模型，所以没有 `rerun.sh`。
