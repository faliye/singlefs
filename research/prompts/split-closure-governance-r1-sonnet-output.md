# split-closure-governance-r1 正推腿报告

日期：2026-11-20。判第 1、3、4 格。方案按 `crates/` 今天的实现来谈：这一批没改 `crates/`，本报告读的是 `scripts/gate/`、`.claude/gate.d/` 下的实现，与 `crates/` 里没有测试服务代码登记表的事实（命令见格 3）。

## 开头

- 被判的 43 份先 `sha256sum -c research/prompts/split-closure-governance-r1-snapshot/judged-sha256.txt`：除了 OK 行没有别的输出，43 份都对得上；`kb-sha256.txt` 那一份 `.claude/kb/decisions/13-验证路线.md` 也 OK。
- 此刻仓的 HEAD：`4319aa2b035b2d8238d64aeb68b3ceb7674cc723`；上游与 HEAD 同一个提交，所以「gate 基准」（`scripts/gate/changed-paths.sh` 的 `gate_diff_base gate`，没给 `GATE_BASE` 时取与上游的 merge-base）此刻就是 HEAD。主 agent 若给了 `GATE_BASE`，下面用 HEAD 当基准算的数要重算。
- 我没跑任何项目代码：所有「待主 agent 跑」的格都在「要主 agent 跑的命令」一节。例外是 `family_sums.py` 与 `family_model.py`：它们只读 tsv 与 `git show`、自己重写比法，不 import 仓里任何代码，我直接跑过（`family_sums.py` 在草稿目录里跑的是同一份逻辑，模型目录里那份只把仓根改成参数，输出逐字节相同，`cmp` 无输出）。
- 模型目录 `research/prompts/split-closure-governance-r1-sonnet-model/`：`tools/family_probe.py`（真调仓里 `lib_lag_split_family.py`）、`tools/family_model.py`（独立重写）、`tools/family_sums.py`（滞后表按族与 HEAD 比的独立重算）、`tools/closure_probe.sh` 与 `samples/`（11 个样本根，真起 `lib_function_units.py` 的 `closure` 与 `key`）、`tools/rerun.sh`（拷自 `scripts/agents/three-way-rerun-template.sh`，只改了 `rerun_steps`）、`tools/rerun-from-a-clean-work-directory.sh`。
- 每个文件的 sha256（量的，`sha256sum` 原样）：

```
68e85a5ec0366509ffb36d1f42615915a4b1340238787d156c7c3b578d0730df  tools/closure_probe.sh
0c12deb43fc9f67d97c43299ddb3d81699d5a404f37d8cabd42a376242aedc89  tools/family_model.py
3df282856cd8fa6f94bba66b120759562a492faf443c8ec496512c189b29424e  tools/family_probe.py
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
184430e2a5959504657276a776d7ad23867490f56a7b87f2ec7b3d0287c5bc6c  tools/rerun.sh
48cb6cacbe038806ba631c464e137e1b4074322876908e15a16711b35d36ddb3  tools/family_sums.py
18a8f58c29ee8381dd314098a53163d2892d715f1ffb55b479d732eae8bc33db  samples/p1c-one-printf-two-variables/s/entry.sh
158a9f1018b4027a9abb736fd7504f2a77f6df40ab1f7efe9cb0cabef27135cf  samples/p1c-one-printf-two-variables/s/helper-used.sh
3d7076abd6e7a976865abe2f816b142843b5ccd5ec47c8b219ab4ae77ab5a580  samples/p1-two-printf-statements/s/entry.sh
158a9f1018b4027a9abb736fd7504f2a77f6df40ab1f7efe9cb0cabef27135cf  samples/p1-two-printf-statements/s/helper-used.sh
366648d64f07ad7f465bf2a9275fa93457c3621ac7a4f0d45f40e8892c489bd1  samples/p2-read-from-file/s/entry.sh
7b2224502c8ce3f30ffdb129ad4dfb98ba89841f2b674e9e9080584cfb82f77a  samples/p3-read-from-file-after-literal/s/entry.sh
984e13ae980f07159476f6dbd37c9c9252e68e9b606030ef0dd1d9270c13a368  samples/p4c-module-object-contained/s/entry.sh
18eefa1d38e680659690b7ee8ef3bac8d19d6ffdd0c2973fd910d1656b894bc6  samples/p4c-module-object-contained/s/helper-unused.sh
158a9f1018b4027a9abb736fd7504f2a77f6df40ab1f7efe9cb0cabef27135cf  samples/p4c-module-object-contained/s/helper-used.sh
395820a61ebebdf249b9314d0ef0578d4ed6b8dfc0120c4cc0a2bed8846598b4  samples/p4-module-object-handed-out/s/entry.sh
18eefa1d38e680659690b7ee8ef3bac8d19d6ffdd0c2973fd910d1656b894bc6  samples/p4-module-object-handed-out/s/helper-unused.sh
158a9f1018b4027a9abb736fd7504f2a77f6df40ab1f7efe9cb0cabef27135cf  samples/p4-module-object-handed-out/s/helper-used.sh
8720ebf0e19d52ecf657a4fca9a1edcee3a7a0e445803aab9ef74b6e492bc8d0  samples/p5c-name-collision-unregistered/s/entry.sh
8720ebf0e19d52ecf657a4fca9a1edcee3a7a0e445803aab9ef74b6e492bc8d0  samples/p5-name-collision-registered/.claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh
8e449d296c71307a98a973046ee7475fcc7816bef81d5dba78c154c48e503130  samples/p6-service-entry-a/task-server/probe_entry.py
97231ca4a92375b19e42752255ea4d7d82d0a115cc23d9eff66714e2a365daa4  samples/p6-service-entry-b/task-server/probe_entry.py
45afa12e0eb73ef239c6f96cb7fe81143b3057e01823005544568bfe0aaef5b6  samples/p6-service-entry-helper-changed/task-server/probe_entry.py
```

- 复跑命令（整条在「要主 agent 跑的命令」一节第一项）：`rerun-from-a-clean-work-directory.sh` 收运行目录参数、开头先删那一个以 `-run` 收尾的运行目录，再调同一个模型目录的 `tools/rerun.sh`。探针的内存与时间：推的，几个小样本加十几次纯函数调用，没有编译；没量过。
- 「成立 / 不成立 / 推不出」在每一格里的含义：格 1 成立 = 要拦的事都拦住；格 3 成立 = 登记表与 D13 对得上、键的组成与主机无关；格 4 成立 = 没留旧读法与旧路径。带「推的」的是读码推出、没起样本坐实的。

## 格 1：拦不拦得住它要拦的事

### 1.1 拆分族认法，喂「拆出一份新部分文件、滞后表里旧路径那一行没删」

- 判的那一行：`comment-history-check.py` 把此刻表里登记的行交给 `split_family.family_growth`（`scripts/gate/comment-history-check.py:700`：「for item in split_family.family_growth(base_counts, current, frozenset(」）。
- 族内之和比基准：`scripts/gate/lib_lag_split_family.py:170`：「if (count if SUM_UNCHECKED in defects else total) > allowed:」；之和超了，族里每个路径都交出来判红。
- 旧行没删、旧文件还在，但命中已搬走一部分：逐文件比，登记数大于此刻命中数判「登记的行数比此刻命中多」：`scripts/gate/comment-history-check.py:803`：「slack.append((path, registered[0], count, registered[1]))」。
- 旧行没删、旧文件已不在：`scripts/gate/comment-history-check.py:669`：「problem = f"登记的路径不在判法射程里的文件中：{path}"」。
- 喂的输入与结果（读码推的，真调用在 `family_probe.py` 前两组，独立模型 `family_model.py` 的前两组我已直接跑过）：基准 `x/foo.py` 登 5，此刻 `x/foo.py` 5 加新部分 `x/foo_part.py` 2：之和 7 大于 5，两个路径都红；`x/foo.py` 3 加 `x/foo_part.py` 2：之和 5，不红。独立模型原样输出：

```
model-case=1 red=['x/foo.py', 'x/foo_part.py']
model-case=2 red=[]
```

- 判：**成立**。旧行没删这一类被三处挡住（之和、登记数大于现数、路径不在射程），没有一条只靠旧行删没删。
- 什么会推翻它：真调 `family_probe.py` 的 `new-part-old-row-kept-sum-over` 或 `new-part-sum-equal` 两行 `verdict=DIFFERENT`；或一份旧行没删、命中数与登记数相等、又在族内加了新部分且新部分有命中，而门禁 comment-history 格没红。

### 1.2 拆分族认法，喂「一份不同族、名字前缀相同的文件」

分三种情形，每种读到哪一行、判成什么：

- 前缀按拆法的字面认：Python 拆法的前缀是入口名「-」换「_」再加「_」（`scripts/gate/lib_lag_split_family.py:94`：「stem.replace("-", "_") + "_"」），钩子拆法的前缀是入口名加「-」（同文件 `:96`：「recipes.append((HOOK_SPLIT_UNRECOGNIZED, own_directory, stem + "-", SCRIPT_SUFFIXES))」）；名字以前缀起头且前缀之后还有部分名才算（`:113`：「if name.startswith(prefix) and name[len(prefix):-len(".py")]:」）。所以 `x/foobar.py` 对 `x/foo.py` 是族外（拆法要求 `foo_`），不同目录同名也是族外：`family_probe.py` 的 `same-directory-name-not-matching-the-split-is-outside`、`different-directory-is-outside` 两组，独立模型 case 7、8 原样输出 `red=['y/foo_part.py']`、`red=['x/foobar.py']`，判红。
- **前缀合拍的不同模块被并进同一族**：`lib_function.py` 与另一个本不是它拆出的 `lib_function_units.py`，只要后者不在基准里，就归到 `lib_function.py` 名下（`:154`：「return max(owners, key=lambda owner: (lengths[owner], len(owner), owner)) if owners else None」）。基准 `lib_function.py` 登 5，此刻 `lib_function.py` 清到 2、`lib_function_units.py` 新登 3，之和 5，**不红**（独立模型 case 3 `red=[]`）。也就是说新模块里新写的 3 行历史写法，被同前缀那份文件清掉的 3 行抵掉了。`lib_function_units.py` 若也在基准里（case 4），它自己一行比自己，红。
  - 这是文件头「拆分族」那一节写明的做法，不是漏写：总数不涨，只是「新写的」与「别处清掉的」可以抵。我把它记成打中的一种边界，不是错判。真仓里这种合并已经在发生：基准里 `task-server/task_server.py` 一行名下，此刻表里并着 `task_server_constants.py`、`task_server_machines.py`、`task_server_store.py`、`task_server_scheduler_*.py` 共 9 个路径，之和 95 对基准 102（量的，`family_sums.py` 输出，命令与输出见格 4）。
- **`main.rs`、`mod.rs` 入口与 `.sh` 入口的族更宽**：`main.rs` 入口认整个目录（`:81`：「if is_directory_entry(path, defects):」那一支，族前缀是目录本身），基准里点名 `a/src/main.rs` 的一行，同目录新来的 `a/src/other_module.rs` 照样归它（case 5 `red=[]`）；基准里点名 `scripts/gate/gate.sh`，同目录 `scripts/gate/gate-staged.sh`（本是另一个脚本）因钩子拆法的前缀 `gate-` 归它（case 6 `red=[]`）。同样是「合并之后总数不涨」。
- 判：**成立**（该红的红：族外、之和超了；该放的放：按文件头认的族），带一条边界：同前缀、同目录的无关文件能与入口互相抵账，上界是基准那一行的数。
- 什么会推翻「按文件头认」：`family_probe.py` 里这四组（`same-prefix-unrelated-module-absorbed`、`main-rs-entry-absorbs-whole-directory`、`hook-entry-absorbs-dash-sibling-script`、`nested-owner-takes-the-longest-prefix`）有 `verdict=DIFFERENT`。要推翻「边界不是错判」，得有用户或 D 号写明「同前缀的无关模块不许与入口抵账」，我没去 kb 里全文搜这样一句（没做，推的）。

### 1.3 拆分族认法在排除表上的另一个比法（`keyed_family_growth`）

- `scripts/gate/lib_lag_split_family.py:180`：「return [(path, segment) for path, segment in current_keys if (family_owner(path, base_paths, defects), segment) not in base_keys]」。键是（路径, 锚串），没有 `family_growth` 那种「之和不超过基准那一行」的计数。
- 输入：基准登一行（`x/foo.py`, `S`），此刻是（`x/foo.py`, `S`）、（`x/foo_a.py`, `S`）、（`x/foo_b.py`, `S`）：三行都归到 `x/foo.py` 名下、段 `S` 在基准里有，**一行都不算涨**（`family_probe.py` 的 `keyed-same-anchor-in-three-part-files-all-pass`，读码推的，待跑）。段不同（`T`）、路径落在族外的照红（同一份探针里另两组）。
- 影响：三张排除表的锚串是「那份文件里含锚串的被判行都放行」（`.claude/heavy-thread-exempt:2` 的文件头写法），所以同族新文件照搬一个已登的锚串，就不用登、不用新理由。`available_parallelism` 这类通用锚串（`scripts/experiment-apparatus/e6_multicore.rs` 那行）尤其宽；`.rs` 入口的族是 `<名>/` 下全部 `.rs`。
- 判：**不成立**（对「只缩不涨」这句话的字面）——计数不设上界，一个入口名下可以无限加同锚串的新部分文件。严重度低：锚串本来就按文件放行，拆出的部分文件继承它是拆分的本意，要新写一处真跑法照搬同一锚串才会漏；我没有在真仓里找到这样的行（没做：三张表此刻的行与基准逐行对比由攻方腿的格 5 管）。
- 什么会推翻它：`family_probe.py` 的 `keyed-same-anchor-in-three-part-files-all-pass` 那一行打出 `verdict=DIFFERENT`（即仓里那份实际红了其中一行）。

### 1.4 算键库，喂「库正文里起一个折不出的命令」

- 这一类的路径：库正文（exec 进模块对象的 here-doc 文本）按内联脚本载入，里面起子进程的那一处折不出时，走 `refuse_launch`：先看登记的被起程序（`scripts/gate/lib_function_units_python_launches.py:355`：「launched = launched_programs_registered(self.source_file(), unit.short.rsplit(".", 1)[-1], self.launch_texts(node, item), self.index.root)」），再看不拒算的程序词（同文件 `:361`：「if self.launch_word_registered(node, item, unit):」），都不是才拒算（`caller_launches` 另有出处推得出是函数参数的那一支，收闭包时判）。
- 拒算落在哪份文件、哪一行：正文出自哪份 here-doc 由 `heredoc_texts_located` 找（先起它的那份文件里整份找逐字相同的，再找 source 的、被 source 的）；`scripts/gate/lib_function_units_python_data_reads.py:1224`：「return dict(self.inline.body_files).get(self.text, self.inline.file)」。找不到出处的正文算在起它的那份文件上。
- 不拒算的登记按（文件、函数短名、词）认：`scripts/gate/lib_function_units_registry.py:867`：「return "program-words-refused" not in active_breaks() and any((file, function, word) in UNKEYED_PROGRAM_WORDS for word in words)」，调用处 `scripts/gate/lib_function_units_python_launches.py:348` 取的函数是 `unit.short.rsplit(".", 1)[-1]`，**类名被丢掉，只剩最后一段**。
- 这批新登的两行：`scripts/gate/lib_function_units_registry.py:63`：「(".claude/gate.d/lib/code-tooling-shared-libraries.sh", "admission_run", "command or ['python3', admission, *arguments]"):」与 `:263`：「(".claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh", "run", "command"):」。
- 打中（读码推的，待样本 `p5`、`p5c` 坐实）：第二行的函数短名是 `run`、词是 `command`，两个都是极普通的名字。同一份 `.sh` 里（含找不到出处而算在这份文件上的别的内联 python 正文）任何类的 `run(self, command)` 里 `subprocess.run(command)` 折不出时，都落在这一行登记上，**不拒算也不连边**。输入：样本根里放 `.claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh`，正文是 `class OtherLauncher: def run(self, command): return subprocess.run(command)`，调用处给 `os.environ["PROBE_COMMAND"].split()`；对照把同一段放在没登记的 `s/entry.sh`。读码推的结果：登记路径上 `closure` 退 0、无拒算；对照退 4。
- 判：「该拒算的拒了没有」对这一类 **不成立**（窄：要同文件、同短名、同词）；admission_run 那行的词是一整句表达式，撞名的可能小得多，我不判它。
- 什么会推翻它：`closure_probe.sh` 打出的 `sample=p5-name-collision-registered` 一行 `exit=4`（读错了：登记还看别的）；或 `p5` 与 `p5c` 同为 `exit=0`（那是 `refuse_launch` 之前就被别的路收掉，与登记无关，则这条打中不成立，要改成「推不出」）。
- 另一个没起样本的边：同样的正文在两份文件里逐字相同时，`heredoc_body_line` 取头一份（`scripts/gate/lib_function_units_shell_index.py:1652` 起的 `found` 列表、`:1654` 取 `found[0]`），按文件查的登记表会照那一份认；我没造这样的输入，推不出。

### 1.5 算键库，喂「`$(函数)` 的函数体是两条 printf」

- 先走 `function_output`（只认函数体恰一条命令的）：`scripts/gate/lib_function_units_shell_index.py:1393`：「if len(commands) != 1:」，两条返回 None。
- 再走 `function_printed_word`（一条 `printf '%s\n'` 打几个词的）：`:810`：「if len(body) != 1 or len(body[0]) < 3」，两条也返回 None。
- 两路都 None 之后 `fold_substitution` 落到「只有一条命令、头是 dirname 一类」那几支，函数名不在其中，返回 None；外面 `-c "$(f)"` 或 `python3 - "$(f)"` 折不出，拒算（文件头「折不出拒算 inline-script-not-literal」；正文作库 exec 进模块对象时落成 `compiled-part-exec`，`scripts/gate/lib_function_units_python_launches.py:192`：「self.refuse(facts, "compiled-part-exec", node)」）。
- 对照：同一份正文改成一条 `printf '%s\n' "$A" "$B"`（样本 `p1c`），折得出，库正文连进闭包。批里自带的夹具只造了「一条 printf 打两个变量、其中一个没赋过」那一格（`scripts/gate/lib_function_units_selftest_fixtures_real_repo.py:1332` 的 `s/printed_unfolded.sh`），「两条语句」这一形态没有专门的格，被 `len(body) != 1` 一行挡住。
- 判：**成立**（读码推的，样本 `p1` 预期 `exit=4`、`p1c` 预期 `exit=0` 且输出里有 `helper-used.sh`，待跑）。
- 什么会推翻它：`p1-two-printf-statements` 打出 `exit=0`；或 `p1c` 打出 `exit=4`。
- 观察（不是打中）：函数体那一条 printf 若带重定向（`printf '%s\n' "$x" > "$out"`），`body[0]` 里是否留着重定向词、会不会被当成「打出的行」，我没读 `split_commands`，推不出；对键只会多收（把本不执行的文本当库正文），不会少收。

### 1.6 算键库，喂「`read -d ''` 从 `"$1"` 那份文件读」

- 只认 here-doc 那一种：`scripts/gate/lib_function_units_shell_index.py:441`：「if first == "read" and not looped and "shell-read-heredoc-assignments-ignored" not in self.breaks:」进 `read_heredoc_assignment`；`:506` 起的条件「or len(inputs) != 1 or len(heredocs) != 1 or heredocs[0] is None:」把 `< "$1"` 挡掉：输入是文件，`heredocs` 为空，函数直接返回，不记任何赋值（`:512` 的 `facts.assignments[names[0]].append(...)` 走不到）。
- 变量没有别的赋值时：`assignment_sites` 空，`fold_word` 折不出，库正文拒算。批里夹具 `s/read_unfolded.sh`（`scripts/gate/lib_function_units_selftest_fixtures_real_repo.py:1347`）就是这一形，样本 `p2` 照抄它。**成立**。
- 打中（读码推的，待样本 `p3` 坐实）：变量先被赋过字面值、再被 `read` 从文件改写：

```
FILE_LIBRARY=""
IFS= read -r -d '' FILE_LIBRARY < "$1" || true
FILE_LIBRARY="$FILE_LIBRARY" python3 - <<'PY'
... exec(compile(os.environ["FILE_LIBRARY"], ..., "exec"), file_library.__dict__)
```

  `assignment_sites` 只收 `facts.assignments` 里记过的赋值（`scripts/gate/lib_function_units_shell_index.py:647`：「found += [(word, holder) for word in carrying(other_facts.assignments.get(name))]」），`read` 从文件读进来的那一次不在里面；`FILE_LIBRARY=""` 是记过的，于是 `FILE_LIBRARY` 折成空串，库正文是空的，不拒算、不连库正文里起的任何脚本，而运行时 exec 的是 `"$1"` 那份文件的内容。夹具 `s/read_unfolded.sh` 里变量没有字面赋值（只有 `FILE_LIBRARY="$FILE_LIBRARY"` 这种赋成自己的词，`carrying` 忽略它），所以夹具碰不到这一形。
- 文件头的话说过头了：`scripts/gate/lib_function_units.py:704`：「输入不是这一条命令自己的 here-doc（< 文件、管道、while 循环）的都不认，变量照旧折不出」。「照旧折不出」只在变量没有别的赋值时为真。
- 这个洞不是这一批引入的（普通 `read` 一向不记赋值；批里新加的是 here-doc 那一支），但批里的文字声称覆盖了它。
- 判：**不成立**（窄：先有字面赋值、再从文件 `read` 的写法，现实里少见）。
- 什么会推翻它：`closure_probe.sh` 的 `sample=p3-read-from-file-after-literal` 打出 `exit=4`（那就是别处还有挡它的路，读错了）。
- 出路（只列，不替主 agent 采纳）：`read` 的目标变量一律记成「折不出」的赋值（所有 `read`、`mapfile`、`printf -v` 都记），而不是只在 here-doc 那一支记；或自证加一格带前置字面赋值的夹具。

### 1.7 算键库，喂「exec 进模块对象而模块对象被交出去」

- 源码折不出：`scripts/gate/lib_function_units_python_launches.py:190`：「if kind is None:」之后 `:191`：「if "python-module-object-exec-unfolded-allowed" not in breaks:」拒算 `compiled-part-exec`（落在 exec 那一行，夹具 `s/module_exec_unfolded.sh`，`scripts/gate/lib_function_units_selftest_fixtures_real_repo.py:1307`）。
- 模块对象交出去：`module_object_contained` 里 `scripts/gate/lib_function_units_python_launches.py:278`：「if self.inline is None or self.tree is None:」非内联脚本一律「出得去」；内联脚本里绑这个模块对象的每个名字，每一处读只许是 `<名>.属性`、`getattr/hasattr`、条件、exec 进它的命名空间、`<别名> = <名>`，任何别的用法（`sys.modules[...] = <名>`、当参数、放进容器、`return`）都 `return False`（同文件 `:310`），`exec_facts` 于是把它记成「出得去」（`:201`：「facts.module_object_texts.append((sources, contained))」），按整个闭包见过的属性名连。结果是多收、不少收。
- 样本 `p4`（`sys.modules["lib_handed_out"] = handed` 之后只调 `used()`）预期输出里有 `helper-unused.sh`；对照 `p4c`（出不去）预期只有 `helper-used.sh`。
- 判：**成立**（读码推的，待样本；方向是保守的）。
- 什么会推翻它：`p4` 的输出里没有 `helper-unused.sh`（交出去的没按整个闭包的属性名连）；或 `p4c` 的输出里有它（出不去的也多连了，只是多收，不算打中，但「按属性只连调到的」那句话就不成立）。
- 没覆盖的边：`exec(compile(...), <名>.__dict__)` 里 `<名>` 换成 `vars(<名>)` 或一个普通 dict 命名空间时，`module_object_code` 返回 None，改走 `compiled_part_exec`，后者只认模块级、且 compile 的文件名不是字面串的那一种；文件头写明「文件名是字面串的、函数里的」不算这一种（`scripts/gate/lib_function_units.py:299`：「不算这一种的：exec 进模块对象的（下面「exec 进模块对象」；那一处的 compile 也不算同一个单元里的那一处）、文件名是字面串的、只 compile 不 exec 的（核语法）、函数里的。」）。这是文件头认下的缺口，不是这一批引入的，我没造输入核它是不是真漏。

## 格 3：复用键与多机

### 3.1 登记表与 D13 那一行逐项对

- kb 原句（`.claude/kb/decisions/13-验证路线.md:454`）：「| 测试服务代码 | `task-server/`、`gpu-verification-server/`、`prune-server/`、`singlefs-unit-graph`、`singlefs-device-library`、`singlefs-background-gpu-pairing-keys`（算键与存取判定的），加崩溃三段管线：录制、核对（CPU 参照 `crash_facts.rs` 与 GPU 内核 `crash_verify_gpu.rs` 都算）、GPU 判器（`crash_judge_gpu.rs`、`crash_judge_dispatch.rs`）与 WGSL 内核 | 不进任何键 |」。
- 键的规矩那一条（同一份文件 `:443`）：「- 判定与复用的键里，代码指纹只取产品代码与测试代码；测试服务代码（task-server、gpu-verification-server 连同它跑的 GPU 判器与核对内核、prune-server）不进任何键，连这些代码自己的单测与变异行也不带它们（用户定）。」
- 表 `scripts/gate/test-service-code-paths.tsv`：内容行数 25（量的，`grep -vc '^#'` 打 25）。逐项：三个服务目录、三个 `crates/…/` 前缀各一行，共 6 行；`crash_facts.rs`、`crash_verify_gpu.rs`、`crash_judge_gpu.rs`、`crash_judge_dispatch.rs` 各一行，共 4 行；WGSL 逐份登 15 行。仓里 `crates/singlefs-verification/src/crash_judge*.wgsl` 也是 15 份（量的，`ls … | wc -l` 打 15；表里 `grep -c 'crash_judge.*\.wgsl'` 打 15）。6 加 4 加 15 等于 25，kb 点名的每一项都在表里。
- kb 点名了「录制」而没点文件；表头 `scripts/gate/test-service-code-paths.tsv:6`：「# 那一行的「录制」没点名文件，这里不登。」。这一格照 kb 字面对得上；录制的代码是不是该登、在哪个文件，kb 没写，我推不出，也不替 kb 补。
- `.rs` 登的行连拆分族：`scripts/gate/lib_function_units_registry.py:927`：「elif path == row or (families and row.endswith(TEST_SERVICE_CODE_FAMILY_SUFFIXES) and split_family.in_family(row, path)):」；`crash_judge_gpu.rs` 与 `crash_judge_dispatch.rs` 各有同名目录（`crash_judge_gpu/tests.rs`、`crash_judge_dispatch/` 下 8 项，量的，`ls` 列过），随族算进，「连它们自己的单测都不进」这一句落得对。`.wgsl` 只认登的那一份（逐份登）。
- 表里没有、但 gpu 服务源码在用的验证包模块：`gpu-verification-server/src` 里 `singlefs_verification::verdict_store` 8 处、`crash_library` 4 处、`crash_amplification` 9 处、`crash_identity` 2 处（量的，`grep -rhoE … | sort | uniq -c`）。kb 点名的是三个 crate 名而非这几个模块，表照字面不登；它们是不是该算「存取判定的」代码，kb 没写，推不出。
- 判（表与 kb 的逐项）：**成立**，带「录制」与上面几个模块两处 kb 本身没点名的空档。
- 什么会推翻它：算键库自证里 `TEST_SERVICE_CODE_TABLE_CELL` 那一格红（它按反引号逐项比 kb 那一行）；或 kb 那一行的反引号项里有一项表里没有（我手对过 kb 里的十项，没有缺）。

### 3.2 测试服务代码进不进键：落在 `closure_key` 的哪一行

- 排除落在单元行与数据行：`scripts/gate/lib_function_units_closure_keys.py:363`：「keyed = [unit for unit in closure.order if service_code_keyed or not is_test_service_code(unit.file)]」。闭包照旧经这些单元往下收，所以经它们才连得到的产品代码仍进键（`closure_key` 文档串 `:347`：「闭包照旧经它们往下收，entry 行照出。」）。
- **打中（读码推的，待样本 `p6` 坐实）：入口单元自己在测试服务代码里时，它的指纹经 entry 行进了键。** `scripts/gate/lib_function_units_closure_keys.py:366`：「lines += entry_key_lines(closure)」不看 `is_test_service_code`；`entry_key_lines` 取 `closure.entry_units` 的（种类，指纹）。入口单元在 `task-server/` 下（例：服务自己的自证脚本或单测函数）时，改入口单元自己的正文，键变；改它调到的别的单元（同在 `task-server/`），键不变。这与 D13（验证路线） `:443`「连这些代码自己的单测与变异行也不带它们」的字面不合。
- 样本：`p6-service-entry-a` 与 `p6-service-entry-b` 只差入口模块体里 `print(helper())` 与 `print(helper() + 1)`，`p6-service-entry-helper-changed` 只改 `helper()` 的返回值。读码推的：a 与 b 的键不同，a 与 helper-changed 的键相同。
- 判：**不成立**（对 `:443` 的严格读法；若用户的原意只是「它们的代码不进别人的键」，这一条就不成立打中）。严重度低：只有入口单元本身（模块体、main-guard、被点名的测试函数）的改动漏进去。
- 什么会推翻它：`closure_probe.sh` 打出的 `p6-service-entry-a` 与 `p6-service-entry-b` 两行读到同一个键（`closure_probe` 的输出文件 `out/closure/p6-service-entry-*.key.txt` 里的键行）。
- 不对称（批里已知）：同文件 `:410`：「lines += sorted(f"unit\t{unit.kind}\t{unit.fingerprint}" for unit in closure.order if not unit.data)」是照 Rust 一侧 `reuse_key` 写的对拍口径，不过滤测试服务代码；Rust 一侧 `grep -rln 'test-service-code\|test_service_code' crates --include=*.rs` 零命中（量的，退出码 1）。批里正文自己写明「等 Rust 一侧补丁合入后整组重录」，我记成已知的未合入，不算打中；补丁合入之前两份实现对同一个 Rust 闭包出的键不同。

### 3.3 拆后闭包里随主机、编译目录、仓根绝对路径变的成分

- 登记表本身：不含绝对路径（`grep -c '/home' scripts/gate/test-service-code-paths.tsv` 打 0，量的）。读它的位置从算键库自己所在的仓根取：`scripts/gate/lib_function_units_base.py:404`：「LIBRARY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))」，不读环境变量、不写主机名；算键库拷进样本仓时随 `RUNTIME_DATA_FILES`（`scripts/gate/lib_function_units_registry.py:37`）一起拷。
- 键里的 unit、data、entry 行只有（种类，指纹）：`entry_key_lines` 文档串 `scripts/gate/lib_function_units_closure_keys.py:372` 写明「不带名字、路径与行号」；shell 里折出来的绝对路径（`BASH_SOURCE`、仓根）只用来找文件，不进指纹。
- **在被判的文件里、不在这一批改动里的一条随主机的成分**：`closure_key` 里 `scripts/gate/lib_function_units_closure_keys.py:361`：「lines += sorted(f"external\t{name}\t{sha}\t{version}" for name, (sha, version) in externals.items())」。调用方 `scripts/gate/admission_unit_key.py:682`：「return {program: context.external(program) for program in sorted(getattr(closure, "programs", {}) or {})}」，取值处 `:469`：「digest = sha256_of_file(os.path.realpath(path))」加 `--version` 第一行。也就是闭包里点名了 `cargo`、`herd7`、`busybox`、`qemu-system-*` 的判定，键里带这台机器上那个二进制的内容 sha256 与版本行；两台装的版本或构建不同，同一份判定的键不同、互不复用。D13（验证路线）`:435` 写「所有判定的键都不带工具链（含 cargo 解析后的生效构建环境）、外部工具版本（Verus、herd7、QEMU 这几个）」。HEAD 里这一行同样在（`git show HEAD:scripts/gate/lib_function_units.py` 里 `grep -nF 'external\t{name}'` 打 15427），所以不是这一批引入的；背景材料写的 `key . .claude/gate.d/code-tooling.sh` 退 4、只 5 行 `external-program-unkeyed`（转述，没复核）是同一条机制的另一面。我不判它归谁（别的会话的变异键整包可能正管着它），只记：键的组成与 D13 `:435` 不合，且随主机。
- 判：表与读表位置 **成立**（与主机无关）；键里 entry 行（3.2）与 external 行（本条）两处与 D13 字面不合，**不成立**，其中 external 行不在这一批改动里。
- 什么会推翻它：两台上各 `closure_programs_externals` 打出的（sha256, 版本行）相同时，external 行不让键随主机变（这时只剩与 D13 字面不合）；我没在第二台上量，推不出实际是否相同。

## 格 4：不兼容旧格式与旧开关

- 拆分族认法 `scripts/gate/lib_lag_split_family.py`：文件里没有 `legacy`、`旧版本`、认拆前路径的分支；唯一一处「旧」字是 `:19` 解释为什么不用搬迁登记（「搬迁登记一个旧前缀只能换一个新前缀……所以由比法这里认族」）。比的是基准路径对此刻路径，没有按拆前文件名再认一遍的读法。开关表（`KNOWN_DEFECTS`）里八项都是弄坏开关，只在自证里打开，不当平常读法。
- 算键库：`grep -nE 'legacy|兼容|backward|deprecated|旧路径|旧开关|旧名|旧版本|旧布局|旧口径|旧格式|拆前|拆之前'` 扫 `lib_function_units*.py`、`comment-history-check.py`、`lib_verus_trait_impl_lag.py`、`verus-proof-check.py`、`verus_proof_check_*.py`、`lib_test_target_sources.py`（量的），只命中三处：
  - `scripts/gate/lib_function_units_rust_closure.py:80`：「self.legacy = "wgsl-external-names-by-long-identifiers" in breaks」与 `:19`：「旧口径（只给弄坏开关 wgsl-external-names-by-long-identifiers）」；`scripts/gate/lib_function_units_rust_workspace.py:963`：「弄坏开关 rust-dependency-features-from-every-section 下退回旧口径」。这些是弄坏开关的实现与说明：打开开关时才走「旧口径」那一支，平常读法不认。它们不是为旧数据、旧版本号、旧路径留的读法，不触犯 `.claude/agent-common.md`「里程碑 17 收尾之前不兼容任何旧东西」那一条；但「写旧口径」是文字上的改动经过，门禁 comment-history 的词表里没有「旧口径」，不会红，只记一笔（不在本格范围）。
  - `scripts/gate/lib_function_units_registry.py:549` 起的 `COMPATIBILITY_RECORD`：那是 `list_check.py` 读的记录文件名常量，与算键库的读法无关。
- `comment-history-lag.tsv` 有没有拆前路径残留：`awk` 取第一列再逐个测文件是否存在，136 行都在盘上、零缺失（量的）。拆前那几份大文件（`crates/singlefs-core/src/mount.rs` 一类）的行还在，是因为它们是拆分后的入口、仍有命中，不是残留。
- 但表与基准比有两处此刻会红（读码加自己重写比法算的，推的）：用我不 import 仓里代码、照 `lib_lag_split_family.py` 文件头独立重写的比法（`family_sums.py`）对 HEAD 里那份 tsv 算，输出原样：

```
.claude/gate.d/crash-point-replay.sh	base=43	cur_sum=50	members=4	RED
.claude/gate.d/doc-process-records.sh	base=39	cur_sum=40	members=3	RED
None	base=0	cur_sum=9	members=6	RED
```

  `crash-point-replay.sh` 一族（表里第 9、12、13、14 行：入口加三份 `lib/crash-point-replay-*.sh`）登记 31 加 6 加 10 加 3 等于 50，基准那一行是 43；`doc-process-records.sh` 一族（第 10、15、16 行）33 加 3 加 4 等于 40，基准 39；族外 6 行没有对应的基准入口（第 11 行 `lib/code-tooling-copy-closure-and-selftest-shape-cells.sh`、第 17 至 20 行 `.claude/hooks/` 下四份、第 112 行 `scripts/gate/large-file-git-hook.py`）。这些行不是拆前路径残留，是拆分后表里的数比基准大。基准是 HEAD 是推的（上游与 HEAD 同一提交）；族外那 6 行里 `.claude/hooks/` 与 `large-file-git-hook.py` 这几份是否出自这一批，我不知道，别的会话同仓干活。
- 判：**成立**（没有旧路径、旧开关名的读法）；`comment-history-lag.tsv` 无拆前路径残留，但与基准比两族之和超、六行无基准，真值待主 agent 跑门禁 comment-history 格。
- 什么会推翻它：门禁 comment-history 格在此刻的树上退 0（那就是基准不是 HEAD，或这几份的命中数已不是表里的数）；或在 `scripts/gate/` 下 `grep -n 'legacy'` 命中别的兼容分支。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 拆分族：拆出新部分文件、旧行没删 | 成立 | 之和超、登记数大于现数、路径不在射程三处各挡一类（`lib_lag_split_family.py:170`、`comment-history-check.py:803`、`:669`） |
| 1 拆分族：不同族、前缀相同的文件 | 成立（带边界） | 族外与不合拆法的红；合前缀的无关模块、`main.rs` 入口同目录、`.sh` 入口的 `-` 兄弟被并进同一族，之和不涨就放，按文件头写的做法，不算错判 |
| 1 排除表的 `keyed_family_growth` | 不成立（低） | 只比（路径, 锚串）在基准里有没有，没有之和上界，同族可无限加同锚串的新部分文件（`lib_lag_split_family.py:180`，读码推的） |
| 1 算键库：库正文里起折不出的命令 | 不成立（窄，推的） | 不拒算的登记按（文件、函数短名、词）认、类名被丢，`TripLauncher.run` 那行的 `run` 与 `command` 太普通（`lib_function_units_python_launches.py:348`、`lib_function_units_registry.py:263`），待样本 `p5`/`p5c` |
| 1 算键库：`$(函数)` 函数体两条 printf | 成立（推的） | `scripts/gate/lib_function_units_shell_index.py:810` 与 `:1393` 两路都返回 None，折不出拒算；样本 `p1`/`p1c` 待跑 |
| 1 算键库：`read -d ''` 从 `"$1"` 读 | 成立（简单形）；不成立（先赋字面值再 read，推的） | `scripts/gate/lib_function_units_shell_index.py:506` 不记赋值，变量没有别的赋值时折不出；变量先赋过字面值则只看到字面值，文件头 `lib_function_units.py:704` 说过头，样本 `p2`/`p3` 待跑 |
| 1 算键库：exec 进模块对象而模块对象交出去 | 成立（推的） | 内联脚本里任何别的用法都 `return False`，按整个闭包的属性名连，多收不少收；样本 `p4`/`p4c` 待跑 |
| 1 合计 | 不成立 | 三处打中：两处读码推的、一处低严重度；其余成立 |
| 3 登记表与 D13 逐项 | 成立 | 25 行对 kb 十个反引号项，WGSL 15 份与仓里 15 份对上；「录制」kb 没点文件，表头写明不登，推不出该不该登 |
| 3 键里测试服务代码的落点 | 不成立（推的） | 单元行、数据行排掉了，但 entry 行不看 `is_test_service_code`（`scripts/gate/lib_function_units_closure_keys.py:366`），入口单元在服务代码里时它的指纹进键；样本 `p6` 待跑 |
| 3 Rust 一侧与 Python 对拍口径 | 推不出 | `unit_graph_key_lines` 不过滤（`scripts/gate/lib_function_units_closure_keys.py:410`）、`crates/` 零命中；批里自己写明等 Rust 补丁合入 |
| 3 随主机的成分 | 成立（表与读表）；不成立（`external` 行，不在这批） | 表无绝对路径、读表从 `LIBRARY_ROOT`；`external` 行带本机二进制 sha256 与版本行（`scripts/gate/lib_function_units_closure_keys.py:361`、`scripts/gate/admission_unit_key.py:469`），HEAD 就在，与 D13 `:435` 字面不合 |
| 3 合计 | 不成立 | 一处打中（entry 行，推的）、一处在被判文件里但不在这批改动里 |
| 4 旧格式与旧开关 | 成立 | 拆分族、算键库里没有为拆前路径、旧开关名留的读法；`legacy` 只出现在弄坏开关的实现里；136 行滞后表路径都在盘上 |
| 4 滞后表按族与基准比（顺带） | 推不出，待跑 | 独立重算对 HEAD：`crash-point-replay.sh` 一族 50 对 43、`doc-process-records.sh` 一族 40 对 39、族外 6 行；基准是不是 HEAD 取决于有没有给 `GATE_BASE` |

## 要主 agent 跑的命令

1. 复跑模型（探针样本、滞后表独立重算、纯函数探针一并跑，输出在运行目录 `out/` 下）。
   - 命令：`task-server/task submit -- bash research/prompts/split-closure-governance-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/split-closure-governance-r1-snapshot /home/fy5090/.local/share/singlefs/work/split-closure-governance-r1-forward-run /home/fy5090/code/singlefs/research/prompts/split-closure-governance-r1-sonnet-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=0`；`task-server/task log` 里：`name=family_probe cases=12 different=0`（`different` 不是 0 就是我读码读错了一组，逐组看运行目录 `out/family_probe.txt`）、三行 `RED` 的族（`family_sums.py` 的输出）、11 行 `sample=` 与 `name=closure_probe samples=11`；每行 `sample=` 的 `exit=` 对着它行尾 `expect=` 看，推断对不上的逐个读运行目录 `out/closure/<样本名>.<模式>.txt`。`p6` 三个样本的键行在 `out/closure/p6-service-entry-*.key.txt`，要比的是 a、b、helper-changed 三个键。
2. 模板自证（rerun.sh 拷自模板，自证走模板里的格）。
   - 命令：`task-server/task submit -- bash research/prompts/split-closure-governance-r1-sonnet-model/tools/rerun.sh --selftest`
   - 在哪起：仓根。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；日志里「自检通过」那一行。
3. 门禁 comment-history 格单跑（核格 4 末项：滞后表与 gate 基准比）。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-text.sh 的 comment-history 格; 范围=1 格; 为什么=核滞后表与 gate 基准比时拆分族之和有没有超、族外有没有新行' task-server/task submit -- bash .claude/gate.d/doc-text.sh --check comment-history`
   - 在哪起：仓根。
   - 看哪几行：`exit=0` 是我的重算错了（基准不是 HEAD，或命中数已变）；`exit=1` 且日志里有「拆分族 .claude/gate.d/crash-point-replay.sh 里 4 行登记之和是 50 行，基准里那一行是 43 行」「拆分族 .claude/gate.d/doc-process-records.sh 里 3 行登记之和是 40 行，基准里那一行是 39 行」与若干行「基准里没有这一行」，是对上了。

## 没做什么

- 没有跑任何项目代码、门禁、自证，也没有往任务服务交任务；格 1、3 里标「推的」「待样本」的结论都是读码推的，真值在上面第 1 项的输出里。
- 不判格 2 与格 5（攻方腿的），也没核「已经量的」那一节里每一句能不能从调度记录与日志里核到；背景材料里的数都当转述，没复核。
- 没通读 43 份：读了 `lib_lag_split_family.py` 全文，`comment-history-check.py` 的滞后表读法与比法、`lib_function_units.py` 文件头里与这几格有关的几节，`lib_function_units_shell_index.py`、`lib_function_units_python_launches.py`、`lib_function_units_registry.py`、`lib_function_units_closure_keys.py` 里引到的函数，与自证夹具里 `exec 进模块对象` 那组。没读：`lib_function_units_rust_*`、`lib_function_units_wgsl.py`、`lib_function_units_lexer.py`、`verus-proof-check.py` 与 `verus_proof_check_*` 的主体、`lib_verus_trait_impl_lag.py` 的比法实现（只读了它调拆分族的几行）、`lib_test_target_sources.py`、几份 `.claude/gate.d/lib/code-tooling-*-cells.sh` 与 doc-experiments、doc-text 那两份、三张排除表的逐行（攻方腿的）。这些没读的部分里出的问题，本报告一概没有说。
- 没看第二台，没量两台上 `external` 行的值是否相同。
- 没判「Z、P4、X、Y 八份先后换上」的换上经过，只看了此刻的文件。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/split-closure-governance-r1-forward` 共 32K（`du -sh`，量的），没有仓副本、编译目录；里面只有 `family_sums.py` 与两份输出，交主 agent 看完后删。
