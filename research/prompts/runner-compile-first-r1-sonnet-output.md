# runner-compile-first-r1 云端正推（Sonnet，R2 格）

判什么：定义 `.claude/agents/experiment-runner.md` 第 2 步「只对入库装置生效」第 ④ 条的每一句，在 `.claude/hooks/write-guard.sh`、`.claude/hooks/bash-command-detector.sh`、`research/scripts/compile-then-swap.py` 三份文件里各落在哪一行、缺哪一句；两个钩子的拒绝信息说的是不是同一件事；`.claude/agent-common.md`「写」与「执行前拒绝的写法」要不要跟着改。只判 R2，不判 R1（放过的写法）与 R3（误拒），那两格不归这条腿。

方法：先用 `grep -nF` 把定义原句、两个钩子的拒绝信息、脚本的函数体在各自文件里现查一遍，命中才写进下表；命中 0 次的改成转述并标「转述」。所有行号都是今天工作区里这三份文件自己的行号（`.claude/agents/experiment-runner.md`、`.claude/hooks/write-guard.sh`、`.claude/hooks/bash-command-detector.sh`、`research/scripts/compile-then-swap.py`、`.claude/agent-common.md`），不是背景材料或附录二 diff 里的行号。

## 一、④条逐句核对

定义原文（`.claude/agents/experiment-runner.md:30`，`grep -cF` 命中 1 次，下面五句都出自这一整行）：

> ④ **入库装置先在草稿目录的副本里改，编过再整份换进主工作区**：主工作区那份先 `cp` 进草稿目录（新建的装置直接建在草稿目录里），只在草稿目录那一份上改；改完跑 `bash research/scripts/capped.sh <线程上限> python3 research/scripts/compile-then-swap.py <草稿副本> crates/singlefs-harness/src/bin/e<号>_<英文名>.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进主工作区，编不过一个字节不写，照它报的错误改草稿副本再跑；主工作区里任何时候都只放编得过的版本，不用 Edit、Write 或 Bash 直接写主工作区 `crates/` 下的 `.rs`。交回之前跑 `python3 research/scripts/compile-then-swap.py --clean --scratch <草稿目录>` 删掉它建的仓副本与编译目录

### 句 1：「主工作区那份先 `cp` 进草稿目录（新建的装置直接建在草稿目录里），只在草稿目录那一份上改」——拷出 + 只改副本

这句本身描述的是执行员自己敲的一条 `cp` 命令，不是钩子或脚本要「执行」的动作；三份被判文件对它的落点都是「不拦」而不是「实现」：

- `write-guard.sh:132`（`decide_compile_first` 的拒绝信息里，`grep -cF 'cp {relative} <草稿目录>/ 拷一份'` 命中 1 次）：把这一步写成「怎么办」的第一句，复述给被拒的执行员看，但函数体本身（`write-guard.sh:120-135`）只在目标命中主工作区 `crates/*.rs` 时才返回拒绝码 2；目标是草稿目录时函数在 `write-guard.sh:126`（`absolute.startswith(os.path.join(root, "crates") + os.sep)` 这一判据不成立）直接 `return 0, None` 放行，没有专门的「认出这是 cp 进草稿目录」的分支——它是靠「目标不在主工作区 crates/ 内」这条判据顺带放行的，不是专门为这一步写的代码。
- `bash-command-detector.sh:2713`（`grep -cF '把主工作区那份拷进草稿目录（\`cp crates/… <草稿目录>/\`）不拦'` 命中 1 次）与文件头注释 `bash-command-detector.sh:65`（`# 放行：…把主工作区那份拷进草稿目录…`）：同样只在文字上确认「不拦」，没有代码专门认「这是一次 cp-进草稿目录」的写法；`compile_first_verdict`（`bash-command-detector.sh:510-543`）判的是目标端落不落在主工作区 `crates/*.rs`，源端在哪它不看。
- `compile-then-swap.py`：全文没有一行代码替执行员做这个 `cp`（它从命令行参数接收 `<草稿副本>` 这个既成路径，不负责生成它）。`swap()` 里 `compile-then-swap.py:188-191`（`for label, path in (("草稿副本", draft), ("草稿目录", scratch)): if os.path.realpath(path) == root or is_inside(path, root): raise BadInput(...)`）核的是反方向：草稿副本与草稿目录本身不许落在主工作区里——这是这句「只在草稿目录那一份上改」的一半防线（防脚本被喂进主工作区里的东西当「草稿」），另一半防线（执行员有没有真的只改草稿、没有绕回主工作区直接改）落在下面句 4 的判定代码里。

结论：这句的「拷出」半句不缺——它不需要被任何一份代码「执行」，因为普通 `cp` 本身够用，钩子只需不拒绝；「只改草稿副本」半句真正的强制机制不是这句自己的代码，而是句 4 的钩子判定（写不了主工作区就只能改草稿）。**什么现象会推翻它**：如果哪天两个钩子里出现了专门拦「cp 进草稿目录」这个方向的判据（把这个方向也拒了），执行员就没有出路了，那时这句就真缺了实现。

### 句 2：「改完跑 `bash research/scripts/capped.sh <线程上限> python3 research/scripts/compile-then-swap.py <草稿副本> crates/…/.rs --scratch <草稿目录>`，它在仓副本里编过才整份换进主工作区」——经脚本编 + 编过才换

- 「经脚本编」：`compile-then-swap.py:138-157`（`def build(...)`）依次跑两条命令——`compile-then-swap.py:142`（`["nice", "-n", "19", "cargo", "build", "--offline", "-p", package, "--bin", bin_name, *profile]`）与 `compile-then-swap.py:143`（`["nice", "-n", "19", "cargo", "test", "--offline", "--no-run", "-p", package, "--bin", bin_name, *profile]`），`CARGO_TARGET_DIR` 指到草稿目录（`compile-then-swap.py:140`）。这两行落实了「在仓副本里编」，且第二条 `cargo test --no-run` 落实了 docstring 里「编 bin 里的 `#[cfg(test)]`」那半句（`compile-then-swap.py:18`，`grep -cF '（后一条编 bin 里的 #\[cfg(test)\]）' research/scripts/compile-then-swap.py` 命中 1 次）。
- 「编过才整份换进主工作区」：`swap()` 函数（`compile-then-swap.py:178-252`）里，`compile-then-swap.py:217`（`if not built:`）编不过就在 `compile-then-swap.py:222`（`return EXIT_BUILD_FAILED, lines`）处直接停下，不往下走；编过了才进到 `compile-then-swap.py:226`（竞态核对 `now != before`）与之后的换上逻辑（`compile-then-swap.py:231-244`：目标不存在走 `create_new_file`，已存在走 `replace_file_contents_by_rename`），换上后 `compile-then-swap.py:246-249` 回读比对 `after != draft_bytes`。这条链落实了「编过才换」与「换上之后回读逐字节比」两件事。
- 两个钩子对这句的落点：`write-guard.sh:133-134` 与 `bash-command-detector.sh:2711-2712` 的「怎么办」里都点名了同一条命令形态（`bash research/scripts/capped.sh <线程上限> python3 research/scripts/compile-then-swap.py … --scratch <草稿目录>`），但钩子本身不执行编译——它们只是在写之前把执行员引导到这条脚本，真正「编过才换」的逻辑只存在于 `compile-then-swap.py` 里，钩子里找不到对应代码，这是分工使然，不算缺。

**什么现象会推翻它**：`compile-then-swap.py:217` 那个 `if not built` 判断被删掉、或 `mode == "swap-on-failure"` 这个自证专用的弄坏开关（`compile-then-swap.py:221`）在非自证场景下也生效，就会出现「编不过也换上」，这句就不成立了；`compile-then-swap.py` 自己的自检（`--selftest`）里「编不过的（bin 本身、`#[cfg(test)]`、新 bin）一个字节不写」这一条覆盖了这个场景（背景材料附录已抄自证末行）。

### 句 3：「编不过一个字节不写，照它报的错误改草稿副本再跑」

- 前半句「编不过一个字节不写」：与句 2 同一处代码（`compile-then-swap.py:217-222`），编不过直接 `return`，不进入 `compile-then-swap.py:231` 之后的任何写文件分支。
- 后半句「照它报的错误改草稿副本再跑」：这是给人看的操作指引，它的字面实现就是 `compile-then-swap.py:218-220` 打印的那句提示——`compile-then-swap.py:219`（`f"→ 怎么办：照上面的错误改草稿副本 {draft}，再跑这条脚本；错误落在草稿副本以外的文件（别的会话在改的），"`）与 `compile-then-swap.py:220`（`"把错误行写进报告交主 agent，不自己去改那些文件"`）。这半句本来就不该、也不可能出现在两个钩子里：钩子在「写之前」拦截，执行不到编译这一步；它只能活在脚本自己的输出里。**这不是缺，是分工对的**。

### 句 4：「主工作区里任何时候都只放编得过的版本，不用 Edit、Write 或 Bash 直接写主工作区 `crates/` 下的 `.rs`」——核心禁止句

这是两个钩子真正落实的那句，逐字对应：

- `write-guard.sh:120-135`（`def decide_compile_first(hook_input, project_root):`）：`write-guard.sh:126`（`if not absolute or not absolute.endswith(".rs") or not absolute.startswith(os.path.join(root, "crates") + os.sep): return 0, None`）判目标是不是主工作区 `crates/` 下的 `.rs`，命中就在 `write-guard.sh:130-135` 返回退出码 2 与拒绝信息。落的是「不用 Edit、Write … 直接写」那半句。
- `bash-command-detector.sh:502-552`（`compile_first_verdict` 与 `compile_first_refusal` 两个函数）：`bash-command-detector.sh:504`（`COMPILE_FIRST_AGENTS = ("experiment-runner",)`）与 `bash-command-detector.sh:2704-2716`（入口调用与打印拒绝信息、`return 2`）。落的是「…或 Bash 直接写」那半句。
- `write-guard.sh:195`（`code, message = decide_compile_first(hook_input, project_root)`，在 `decide()` 里排在 `decide_overwrite`（`write-guard.sh:192`）之后、`decide_scope`（`write-guard.sh:198`）之前）与 `bash-command-detector.sh:2704`（`compile_first_refusal` 的调用排在 `script_in_place_refusal`(⑦) 之后、`repository_in_place_edit_refusal`(⑧，`bash-command-detector.sh:2717`) 之前）都对应上了各自注释里写的判定次序（`write-guard.sh:19`「排在写范围那一道之前判」、`bash-command-detector.sh:59`「排在 ⑧ 之前判」）——这两句「次序」的说明与代码一致，不是空话。

这句两个钩子都落实了，是全篇里唯一被两处代码同时实现的一句。**什么现象会推翻它**：如果哪个钩子的判定函数在 `agent_type == "experiment-runner"` 时对 `crates/*.rs` 目标返回放行（`0, None` 或空列表），这句就破了；两份文件各自的 `--selftest` 已经各自钉了一批必须拒绝的用例（`write-guard.sh` 的 `finding_case("先编后换:experiment-runner Edit 主工作区的入库装置", …)`、`bash-command-detector.sh` 的 `compile_first_cases` 里 `want=1` 的十七行），这两套自证连续判红才说明这句真的破了。

### 句 5：「交回之前跑 `python3 research/scripts/compile-then-swap.py --clean --scratch <草稿目录>` 删掉它建的仓副本与编译目录」——交回前 `--clean`

- 脚本自己实现了「能做什么」：`compile-then-swap.py:256-261`（`def clean(scratch): work = os.path.join(...); if not os.path.isdir(work): return […]; shutil.rmtree(work); return […]`）与 `compile-then-swap.py:392-394`（`main()` 里 `if flags["--clean"]: print("\n".join(clean(options["--scratch"]))); return 0`）。
- 但「该不该做、有没有真的在交回前做」这半句——即这条命令是不是执行员**必须**在交回前跑的一步——在两个被判的钩子文件里一行都找不到：`grep -c -- '--clean' .claude/hooks/write-guard.sh` 与 `grep -c -- '--clean' .claude/hooks/bash-command-detector.sh` 都是 0（现查：两条命令原样跑过，输出都是 `0`）。两个钩子完全没有在「交回」这个动作上核过 `--clean` 有没有被跑过。

**这一句缺失**：定义要求「交回之前跑 `--clean`」，这是一句强制动作，但被判的三份文件里，只有脚本自己具备「能执行 `--clean`」的能力，没有任何一处代码强制或核对「执行员真的在交回前跑了它」。

共用约束里另有一道通用的交回闸（`.claude/agent-common.md:73`「交回闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh`…）：临时目录里自己建的编译目录、工作树与仓副本还在，交回报告里又没逐个写全路径与为什么不删」）字面上可能兜得住这个场景，但它是不是认得出 `compile-then-swap.py` 建的 `<草稿目录>/compile-then-swap/tree`、`<草稿目录>/compile-then-swap/target` 这两个特定子目录名——**复核不了**：这道闸不在这一格点名的三份被判文件（`write-guard.sh`、`bash-command-detector.sh`、`compile-then-swap.py`）之列，是上游 `.claude/singlefs-ai-sop/` 下的脚本，这一轮的禁读与写范围都没有给到它，我没有去读它的判据代码，写「复核不了」而不是替它下结论。

**什么现象会推翻它**：哪天这两个钩子里出现了一条专门核「`compile-then-swap` 目录还在、交回报告没提」的判据，句 5 就补上了；或者拿 `handback-scratch-check.sh` 的判据代码现查一遍，发现它确实认得出这个具体目录名——那也能说明这条闸间接补上了这个缺口（但这需要另外一轮去核，不归这一格）。

## 二、定义与两个钩子的拒绝信息说的是不是同一件事

**一致**。两处拒绝信息共享同一句逐字相同的核心表述：

- `write-guard.sh:130-131`：`f"✗ {agent_type} 不直接写主工作区的 {relative}：入库装置先在草稿目录的副本里改，编过再整份换进主工作区，" "主工作区里任何时候都只放编得过的版本。\n"`
- `bash-command-detector.sh:2709-2710`：`f"  ✗ {hook_input.get('agent_type')} 在 Bash 里写主工作区 crates/ 下的 .rs（认出的写法与目标：{'；'.join(crate_writes)}）：" "入库装置先在草稿目录的副本里改，编过再整份换进主工作区，主工作区里任何时候都只放编得过的版本"`

两者「入库装置先在草稿目录的副本里改，编过再整份换进主工作区，主工作区里任何时候都只放编得过的版本」这十九字这句一字不差（`grep -cF` 在两个文件里各命中 1 次）。出路也指向同一条命令形态（`bash research/scripts/capped.sh <线程上限> python3 research/scripts/compile-then-swap.py … --scratch <草稿目录>`）与同一句收尾：`write-guard.sh:135`「目标不是 crates/<crate>/src/bin/ 下的装置（crate 的库、测试）的，不归你改，写进报告交回主 agent。」（`grep -cF` 命中 1 次）；`bash-command-detector.sh` 里同样的话被源码拆成两行字符串拼接——`bash-command-detector.sh:2713`（`grep -cF '目标不是 crates/<crate>/src/bin/ 下的装置的，不归你改，' .claude/hooks/bash-command-detector.sh` 命中 1 次）与 `bash-command-detector.sh:2714`（`grep -cF '写进报告交回主 agent。这一道只拒这种写法，不停你在跑的任何东西' .claude/hooks/bash-command-detector.sh` 命中 1 次），运行时打印出来是同一整句「目标不是 crates/<crate>/src/bin/ 下的装置的，不归你改，写进报告交回主 agent」——两处意思一致。

差异只在呈现顺序与细节，不构成意思冲突：

- `write-guard.sh:132` 把「先 `cp` 拷一份」写成「怎么办」的第一句；`bash-command-detector.sh:2713` 把「把主工作区那份拷进草稿目录不拦」放在段尾当补充说明。两处都确认这一步放行，顺序不同不影响判断。
- `bash-command-detector.sh:2709` 多贴了一串「认出的写法与目标」（`{'；'.join(crate_writes)}`）——这是 Bash 命令解析出的具体命中列表，`write-guard.sh` 的 `decide_compile_first` 只处理单一目标路径（`file_path`），不需要也没有这份列表，这是两个工具输入形态不同带来的必然差异，不是遗漏。
- 「这一道只拒这种写法，不停你在跑的任何东西」这句在 `bash-command-detector.sh:2714`（`grep -cF` 命中 1 次）逐条判定块都会重复一遍，是该文件内部的一贯风格（其余 ⑤⑥⑦⑧ 判定块同样每块都贴一遍）；`write-guard.sh` 不逐条重复，而是在文件头一次性写清楚（`write-guard.sh:25`「它们拒的是一次写，不停任务和脚本」），也是该文件一贯的写法（一、二、三道判定同样不逐条重复这句）。两边各自内部一致，不是这一条新加的规则单独破例。

**什么现象会推翻它**：如果两处核心句的字面出现分歧（例如一处写「先编后换」另一处写「先测后换」），或者一处出路指向 `compile-then-swap.py`、另一处指向别的脚本，才算「说的不是同一件事」；现查下来两处都没有。

## 三、`.claude/agent-common.md`「写」与「执行前拒绝的写法」要不要跟着改

现查 `.claude/agent-common.md` 今天的内容，**零命中**新规则：

```
$ grep -n 'compile-then-swap\|COMPILE_FIRST\|先编后换' .claude/agent-common.md
（无输出，退出码 1）
```

逐句核这一节今天写的是什么、和两个钩子今天的代码对不对得上：

### 「写范围」那一句（`.claude/agent-common.md:34`）——不用跟着改

原句（`grep -cF` 命中 1 次）：「只写派发提示与定义「写范围」一节给的路径。写范围闸（项目 settings 里的 hook，表在 `.claude/hooks/agent-write-scope.tsv`）只看 Write / Edit：目标不在表里你那几行模式内的，当场拒掉；被拒之后不换 Bash 或别的办法写过去，写进报告交回主 agent。Bash 里的写闸看不见，全靠你守。」

这句字面上专指「写范围闸」（即 `write-guard.sh` 里的第二道判定 `decide_scope`，读 `agent-write-scope.tsv` 那一道），说的是**这一道**只挂 Write / Edit、看不见 Bash。现查 `write-guard.sh` 今天仍然只在 `PreToolUse:Write` 与 `PreToolUse:Edit` 上触发（`write-guard.sh` 是 Python 脚本、由项目 settings 的 hook 配置决定触发点，这一轮没有改 `.claude/settings.json`，背景材料「二、实现今天的样子」已核「`.claude/settings.json` … 没改（`git diff --stat` 为空）」），字面没有变假。新加的「先编后换」判定虽然也拦 Bash（`bash-command-detector.sh` 的 ⑨），但那是**另一个钩子**里的**另一道规则**（先编后换，不是写范围），不是这句说的「写范围闸」本身多出了 Bash 触发点。**这句不用跟着改**——它今天仍然为真。

### 「写」一节列举可直接写仓内文件的脚本清单（`.claude/agent-common.md:35`）——要跟着改

原句节选（`grep -cF` 命中 1 次）：「…Bash 里只许定义点名的脚本自己写（`relabel-item.py`、`kb-scribe` 的 `replace-batch.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向），不用 python / sed 就地改仓内文件（执行前被拒，见「执行前拒绝的写法」⑧）。」

`research/scripts/compile-then-swap.py` 正是这一轮新增的、专门被定义④点名要在 Bash 里调用、且会直接改写主工作区 `crates/*.rs`（换上那一步，`compile-then-swap.py:236`／`:241`）的脚本，性质与清单里已经列出的 `mutate.sh`、`replay.sh`（都是「定义点名、允许直接写仓内特定文件」的脚本）一样，但清单里没有它。**这一句要跟着改**：把 `compile-then-swap.py` 加进这份清单，否则字面上「Bash 里只许定义点名的脚本自己写」这句罗列的清单就没跟上今天真正被点名允许写的那一批脚本。

**什么现象会推翻它**：如果这份清单本来就不追求穷举（比如清单末尾原本就写着「等」或「这一类」这种开放式收尾），漏列一个新脚本就不算「要改」；现查清单收尾是「不用 python / sed 就地改仓内文件（执行前被拒…）」，前面的枚举没有省略号或「这一类」这种开放式用语，是一份闭合列举，所以漏列才算数。

### 「执行前拒绝的写法」——Bash 检出 hook 那一段（`.claude/agent-common.md:61`起）——要跟着改

原句（`grep -cF` 命中 1 次）：「Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）：② 只在前台拒（run_in_background 里的等待循环只记检出、不拒），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：」，往下逐条列了 ①②③④⑤⑥⑧⑦，**没有 ⑨**。现查 `bash-command-detector.sh` 今天确实有 ⑨（`bash-command-detector.sh:57-70` 注释、`bash-command-detector.sh:502-552` 代码），是这一轮新加的、会在执行前把命令拒掉（退出码 2，`bash-command-detector.sh:2716`）的一道判定，落在这一节该列举的范围内（这一节列的正是「执行前」会被拒的写法）。**要跟着改**：补一条「⑨ experiment-runner 在 Bash 里写主工作区 `crates/` 下的 `.rs`：出路是 `research/scripts/compile-then-swap.py`」。

### 「执行前拒绝的写法」——写闸那一条（`.claude/agent-common.md:72`）——要跟着改

原句（`grep -cF` 命中 1 次）：「写闸（`.claude/hooks/write-guard.sh`，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（`.claude/hooks/agent-write-scope.tsv`）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）。」这句只列了 `write-guard.sh` 今天四道判定里的三道（`write-guard.sh:6`「整份覆盖未跟踪文件」、`write-guard.sh:10`「项目 subagent 的写范围」、`write-guard.sh:14`「撇号类角标」），漏了这一轮新加的第四道（`write-guard.sh:18`「四、先编后换」）。**要跟着改**：补一句「experiment-runner 写主工作区 `crates/` 下的 `.rs` 一律拒，出路是 `research/scripts/compile-then-swap.py`」。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| ④句1（拷出+只改副本） | 不缺 | 「拷」这半句不需要代码执行、两个钩子都放行；「只改草稿」半句真正靠句4的禁止逻辑撑住 |
| ④句2（经脚本编+编过才换） | 落实 | `compile-then-swap.py:138-157`（`build()`）编、`:217-252`（`swap()`）编过才走换上分支，两个钩子只转述命令、不参与执行 |
| ④句3（编不过一个字节不写+照错误改草稿再跑） | 落实 | 同句2那处代码（`:217-222` 不写）；后半句是脚本打印给人看的提示（`:219-220`），钩子不该也没有出现这句 |
| ④句4（不用 Edit/Write/Bash 直接写） | 落实（双落点） | `write-guard.sh:120-135` 判 Write/Edit，`bash-command-detector.sh:502-552` 判 Bash，逐字对应定义原句，是唯一被两处代码都实现的一句 |
| ④句5（交回前 `--clean`） | **缺** | 脚本有能力执行（`compile-then-swap.py:256-261`、`:392-394`），但两个钩子里 `grep -c -- '--clean'` 均为 0，没有代码核过「有没有真的在交回前跑」；上游通用交回闸能不能兜底——复核不了 |
| 拒绝信息是否同一件事 | 一致 | 核心句「入库装置先在草稿目录的副本里改，编过再整份换进主工作区，主工作区里任何时候都只放编得过的版本」在 `write-guard.sh:130-131` 与 `bash-command-detector.sh:2709-2710` 逐字相同；出路与收尾句意思一致，差异只是呈现顺序与各文件自身的一贯风格 |
| `agent-common.md`「写范围」一句（:34） | 不用改 | 这句字面专指写范围闸（`decide_scope`），今天仍然只挂 Write/Edit，没有被这一轮动摇 |
| `agent-common.md`「写」一节脚本清单（:35） | 要改 | 闭合列举里没有 `compile-then-swap.py`，它与已列的 `mutate.sh`、`replay.sh` 同类、该收进去 |
| `agent-common.md`「执行前拒绝的写法」Bash 检出 hook 条目（:61起） | 要改 | 列了 ①②③④⑤⑥⑦⑧，漏了这一轮新加的 ⑨ |
| `agent-common.md`「执行前拒绝的写法」写闸条目（:72） | 要改 | 只列了 `write-guard.sh` 四道判定里的前三道，漏了「四、先编后换」 |

## 没做什么

- 没判 R1（还有没有别的路把编不过的代码落进主工作区 `crates/`）与 R3（合法操作会不会被误拒）：这两格不归这条腿，分工表写明由 Opus 与本地攻方各判一格。
- 没有去读上游 `.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh` 的判据代码，因此句 5「交回前 `--clean`」有没有被那道通用交回闸间接兜住，写的是「复核不了」而不是结论；这一轮的禁读清单与写范围都没有把那份文件给到这条腿。
- 没有实际去跑 `write-guard.sh --selftest` 或 `bash-command-detector.sh --selftest`（背景材料「二、实现今天的样子」已经给出这一轮的原样自证末行，这条腿只对着源码逐行核对文字与行号，不重复跑自证；这两份自证本身是否真的绿，由主 agent 或核查员核）。
- 没有改任何主工作区文件；这份报告与草稿目录 `/tmp/claude-1000/runner-compile-first-r1-sonnet/` 是这条腿唯一写过的位置（草稿目录这一轮没有用上，是空的）。
