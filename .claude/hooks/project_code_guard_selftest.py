# 项目代码闸 project-code-guard.sh 的 python 部分之一：自证的样本脚本、selftest（拒绝与放行的样本、走真实入口的几格、开关轮）、弄坏开关只在自证里认的拒绝与 main。
# 入口 project-code-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 PROJECT_CODE_GUARD_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import importlib.util
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from typing import NamedTuple
from __main__ import PROJECT_CODE_GUARD_PART_NAMES, SELFTEST_MARK_CONTENT, SELFTEST_MARK_VARIABLE
from project_code_guard_decisions import (
    DEFAULT_DETECTIONS, GUARD_BREAK_VARIABLE, HOOK_DIRECTORY, KNOWN_GUARD_BREAK_ITEMS, MAXIMUM_SCRIPT_BYTES, UNREADABLE_HOOK_INPUT, cargo_lock,
    clippy_lint_table_tail, code_discipline_lints, guard_break_is_set, parsed_hook_input, project_code_runs, record_detection, scan_decision,
    shared_library_problem, shell_words, task_server_decision, work_roots,
)
import project_code_guard_decisions

def build_sample_scripts(work):
    """自检用的脚本：被拒的那一条（herd7 x.litmus）写在 s.sh 第 4 行；另有嵌套、只含轻命令、读不到、看不全与不读的几种。"""
    def write(relative, content, mode="w"):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, mode) as handle:
            handle.write(content)
    write("s.sh", "#!/usr/bin/env bash\n# 验证链：先编，再跑\nset -euo pipefail\nherd7 x.litmus\n")
    write("scripts/mutation/prove-red.sh", "#!/usr/bin/env bash\n# 样本（证红脚本的替身，正文只有一行轻命令）\necho sample\n")
    write("outer.sh", "#!/usr/bin/env bash\necho 起里一层\nbash s.sh\n")
    write("product-gpu.sh", "#!/usr/bin/env bash\n# 样本：产品 GPU 包的真卡对拍\n"
                            "cd product-gpu-sample && cargo test -p singlefs-background-gpu --features tests-on-a-real-gpu -- --test-threads 1\n")
    write("light.sh", "#!/usr/bin/env bash\nbash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract\necho \"herd7 x.litmus\"\n"
                      "cat > note.md <<'EOF'\nherd7 x.litmus\nEOF\nbash scripts/hosts/run-with-memory-cap.sh 4G cargo clippy --all-targets -- -D warnings\n")
    write("loop.sh", "#!/usr/bin/env bash\n[ -n \"${AGAIN:-}\" ] || AGAIN=1 bash loop.sh\n")
    for level in range(1, 8):
        write(f"deep{level}.sh", f"bash deep{level + 1}.sh\n" if level < 7 else "herd7 x.litmus\n")
    write("blob.bin", b"\x00\x01herd7 x.litmus\n", "wb")
    write("big.sh", "# 填充\n" * (MAXIMUM_SCRIPT_BYTES // 8 + 1) + "herd7 x.litmus\n")
    write("tool", "#!/usr/bin/env python3\nimport subprocess\nsubprocess.run(['cargo', 'test', '--all'])\nherd7 x.litmus\n")
    write("prog", b"\x7fELF\x02\x01\x01\x00\x00\nherd7 x.litmus\n", "wb")
    # 超过读的上限的编译产物与 python 程序：先认出不是 shell 脚本、不读也不记，轮不到大小上限（例：e49_chain_width）
    write("bigprog", b"\x7fELF\x02\x01\x01\x00" + b"\x00" * (MAXIMUM_SCRIPT_BYTES + 64) + b"\nherd7 x.litmus\n", "wb")
    write("bigtool", "#!/usr/bin/env python3\n" + "# 填充\n" * (MAXIMUM_SCRIPT_BYTES // 8 + 1) + "herd7 x.litmus\n")
    write("elsewhere/mutate.sh", "#!/usr/bin/env bash\nherd7 x.litmus\n")
    # 路径尾部与仓内位置相同、却不是本仓那一份的拷贝：正文起外部程序，照读正文、照拒
    for copied_entry in ("scripts/experiment-apparatus/replay.sh", "scripts/crash/crash-point-replay.sh", "scripts/crash/crash-amplification.sh"):
        write(f"copy-tree/{copied_entry}", "#!/usr/bin/env bash\nherd7 x.litmus\n")
    # 经符号链接起本仓入口：入口按自己所在的路径（不解符号链接）找旁边的 .claude/scripts/preflight.sh，链接旁边放一份改过的（起 验证包 cargo）
    write("link-tree/.claude/scripts/preflight.sh", "#!/usr/bin/env bash\ncargo test --release -p singlefs-verification --lib --tests\n")
    os.makedirs(os.path.join(work, "link-tree", "scripts", "experiment-apparatus"))
    os.symlink(os.path.join(project_code_runs.HOOK_REPOSITORY, project_code_runs.KNOWN_SCRIPT_LOCATIONS["replay.sh"]),
               os.path.join(work, "link-tree", "scripts", "experiment-apparatus", "replay.sh"))
    write("scripts/gate/gate-staged.sh", "#!/usr/bin/env bash\nbash .claude/scripts/gate.sh --staged\n")
    write(".claude/singlefs-ai-sop/scripts/naming-lint.sh", "#!/usr/bin/env bash\ntrue\n")   # 交回前自查命名的那条命令起的上游 lint：脚本在，读得到
    # 查名字与数组赋值：照 .claude/scripts/fetch-deps.sh 第 44 行、scripts/gate/change-touches-crates.sh 第 61 行原样
    write("command-lookup.sh", "#!/usr/bin/env bash\n  if command -v herd7 >/dev/null 2>&1; then ok \"herd7 已在\"; return 0; fi\n")
    write("array-assignment.sh", "#!/usr/bin/env bash\n  ((${#prefixes[@]})) || prefixes=(\"crates/\")\nchains=(./s.sh ./outer.sh)\n")
    # 命令替换的两种写法，照 .claude/singlefs-ai-sop/scripts/doc-lint.sh 第 43 行、第 70 行原样：切碎之后的片段不许当成脚本路径去读
    write("doc-lint-line-43.sh", "#!/usr/bin/env bash\n  set -- \"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n")
    write("doc-lint-line-70.sh", "#!/usr/bin/env bash\n"
          "DOC_LINT_FAMILY=\"$(sed -n 's/^family=//p' \"$(dirname \"${BASH_SOURCE[0]}\")/../I18N\" 2>/dev/null || true)\"\n")
    write("substitution-fragments.sh", "#!/usr/bin/env bash\ncd $(dirname $0)/.. && echo $((total/2)) `dirname $0`/x.sh\n"
          "$(dirname \"$0\")/helper.sh --flag\n`dirname $0`/helper.sh\n"
          "out=\"$(bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract 2>&1)\"\n")
    # 不经内存包装跑一个测试目标：子 agent 起它要拒（经包装起它放行）
    write("unwrapped.sh", "#!/usr/bin/env bash\ncargo test -p singlefs-core --test core_contract\n")
    write("substitution-heavy.sh", "#!/usr/bin/env bash\n  set -- \"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n"
          "summary=\"$(herd7 x.litmus 2>&1 | tail -3)\"\n")
    # 直接执行时不当脚本读的文本：Rust 源码的文档注释里反引号括着彼此的路径与 `herd7 x.litmus`，当 shell 读就一路互相读下去（一条命令 100 秒判不完）
    checker_sources = ["walk.rs", "image.rs", "lib.rs"]
    for name in checker_sources:
        others = "` 与 `".join(f"crates/singlefs-checker/src/{other}" for other in checker_sources if other != name)
        write(f"crates/singlefs-checker/src/{name}", f"//! 池级 checker：见 `{others}`\n/// 跑法：`herd7 x.litmus`\npub fn check_pool_image() {{}}\n")
    write("notes.md", "# 验证链\nherd7 x.litmus\n")
    write("plain-text", "herd7 x.litmus\n")
    write("2", "echo from-source-two\n")   # 引号数字源测试（cat s.sh "2"> …）用的在的源："2" 是文件名、不是 fd，正常下接起来判（不是 H7 的源不在）
    write(".claude/singlefs-ai-sop/scripts/doc-lint.sh", "#!/usr/bin/env bash\necho doc-lint\n")
    # 直接起照拒的本仓脚本（project_code_use 凭路径尾部在本仓里有这份文件认，盘上不在的不记直接起）：门禁阶段、上游的 lint、门禁结构检查，样本里放一份同名的文本
    # doc-text.sh、doc-process-records.sh、doc-experiments.sh 是「python3 - 读几份门禁」喂的参数：盘上有，LIB_SHELL_WORDS_BREAK=arguments-as-executed 下
    #   走完整判法（task_server_decision）的那一格才把 doc-text.sh 认成 python3 起的本仓门禁阶段而拒
    for sample_script in (".claude/gate.d/code-source-discipline.sh", ".claude/gate.d/code-tooling.sh", ".claude/gate.d/lkmm.sh",
                          ".claude/gate.d/doc-text.sh", ".claude/gate.d/doc-process-records.sh", ".claude/gate.d/doc-experiments.sh",
                          ".claude/singlefs-ai-sop/scripts/gate-lint.sh", ".claude/singlefs-ai-sop/scripts/hooks-registered.sh", "scripts/gate/gate-structure-check.py"):
        write(sample_script, "#!/usr/bin/env python3\nprint('样本')\n" if sample_script.endswith(".py") else "#!/usr/bin/env bash\necho 样本\n")
    # 直接执行时当脚本读的：没扩展名而 `#!` 指到 shell 的，扩展名不对而 `#!` 指到 shell 的
    write("runner", "#!/usr/bin/env bash\nherd7 x.litmus\n")
    write("mislabeled.py", "#!/bin/sh\nherd7 x.litmus\n")
    # 一条命令里同一份脚本起几遍：twice.sh 三种起法各起一遍 s.sh；cyc-a.sh 与 cyc-b.sh 互相调；rel.sh 按相对路径起的脚本随当前目录变
    write("twice.sh", "#!/usr/bin/env bash\nbash s.sh\n./s.sh\nsource s.sh\n")
    write("cyc-a.sh", "bash cyc-b.sh\nherd7 x.litmus\n")
    write("cyc-b.sh", "bash cyc-a.sh\n")
    write("rel.sh", "bash ./inner-rel.sh\n")
    write("inner-rel.sh", "echo 轻\n")
    write("sub/inner-rel.sh", "herd7 x.litmus\n")
    write("has-missing.sh", "bash missing.sh\n")
    write("outer-missing.sh", "echo 起里一层\nbash has-missing.sh\n")
    # BASH_ENV / ENV 指的启动文件：正文只有一行重活的、只有轻活的
    write("startup-heavy.sh", "bash scripts/hosts/run-with-memory-cap.sh 16G herd7 x.litmus\n")
    write("startup-light.sh", "echo hello\n")
    # 直接执行的脚本先读的启动文件：#! 指到 bash、env bash、sh、python、认不出的解释器、zsh 的，和没有 #! 的
    for name, text in (("exec-bash.sh", "#!/bin/bash\necho hello\n"), ("exec-env-bash.sh", "#!/usr/bin/env bash\necho hello\n"), ("exec-sh.sh", "#!/bin/sh\necho hello\n"),
                       ("exec-python.py", "#!/usr/bin/env python3\nprint(1)\n"), ("exec-unknown.sh", "#!/opt/odd/interp\nhello\n"),
                       ("exec-zsh.sh", "#!/bin/zsh\necho hello\n"), ("exec-no-shebang.sh", "echo hello\n")):
        write(name, text)
    # 交互的 bash 读 HOME 下的 .bashrc，zsh 读 ZDOTDIR（没设取 HOME）下的 .zshenv 与 .zshrc
    write("home-heavy/.bashrc", "herd7 x.litmus\n")
    write("home-light/.bashrc", "echo hello\n")
    write("zdot-env/.zshenv", "bash scripts/hosts/run-with-memory-cap.sh 16G herd7 x.litmus\n")
    write("zdot-env/.zshrc", "echo hello\n")
    write("zdot-rc/.zshenv", "echo hello\n")
    write("zdot-rc/.zshrc", "herd7 x.litmus\n")
    write("zdot-light/.zshenv", "echo hello\n")
    write("zdot-light/.zshrc", "echo hello\n")

# 一条命令 100 秒判不完的写法：双引号里没转义的反引号把 `crates/singlefs-checker/src/walk.rs` 当命令替换执行（原样，cwd 换成自检的临时工作区）
SLOW_REPRODUCTION_COMMAND = 'python3 scripts/kb/replace-once.py .claude/kb/verification-build.md "池级 checker（\\`crates/singlefs-checker\\` 的 \\`walk::check_pool_image\\`，24 条）" "池级 checker（\\`crates/singlefs-checker\\` 的 \\`walk::check_pool_image\\`，26 条：第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽））" && python3 scripts/kb/replace-once.py .claude/kb/milestone/02-second-txn.md "checker 新接的只有 I-3.8（实例表行唯一且低于挂载根），I-3.1（已分配统计对得上） 与 I-2.1（校验和与内容匹配） 改按回退候选集判，预想清单里其余各条（I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽） 那一族起）还没有 checker；" "checker 新接了 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）（后两条按候选集里每条根各判一格：走读它引用的单元时校验和对不上或头用不了就红，坏镜像是 mkfs 树表单元——只有第 0 代根还引用——改内容重封 / 抹成零；`crates/singlefs-checker/src/walk.rs`、`image.rs` 清单 26 条），I-3.1（已分配统计对得上） 与 I-2.1（校验和与内容匹配） 改按回退候选集判；预想清单里其余各条（I-3.4（可用空间扣待删占用） 起）还没有 checker；" && bash .claude/singlefs-ai-sop/scripts/doc-lint.sh . 2>&1 | tail -1'
SLOW_REPRODUCTION_SECONDS = 1.0  # 复现输入那一条要在这么多秒内判完

class LayerRun(NamedTuple):
    returncode: int    # 同钩子入口的退出码：0 放行，2 拒绝
    stderr: str        # 拒绝说明（放行是空串）

def category_layer_run(hook_input, project_root, environment_changes, environment_removed=()):
    """自证里在进程中照一格的环境判认不出的写法、外部程序、编译目录那一层（scan_decision）：环境变量照 environment_changes 设、environment_removed 去掉，判完原样还回去。"""
    saved = dict(os.environ)
    try:
        for name in environment_removed:
            os.environ.pop(name, None)
        os.environ.update(environment_changes)
        decision = scan_decision(hook_input, project_root)
    finally:
        os.environ.clear()
        os.environ.update(saved)
    return LayerRun(decision.code, decision.message or "")

def selftest(hook_dir):
    os.environ[SELFTEST_MARK_VARIABLE] = SELFTEST_MARK_CONTENT   # 自证起的钩子子进程（环境原样继承）靠它认：弄坏开关只在自证里认
    if project_code_runs is None:
        print(f"  ✗ 自检：{shared_library_problem(hook_dir)}")
        print("    → 怎么办：恢复 .claude/hooks/lib_project_code_runs.py 与 lib_shell_words.py（判定与切词只有那两份，别在 hook 里再抄一份），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp(prefix="project-code-guard-")
    results = []
    # 登记表那几格起的是本仓里那几份脚本（键按解析后的真实路径认）；喂 stage-selftest.sh 的样本拿临时样本仓的暂存区比（lib_project_code_runs 的 SAMPLE_INDEX_REPOSITORY）
    repo, real_sample_index = project_code_runs.HOOK_REPOSITORY, project_code_runs.lib_project_code_runs_base.SAMPLE_INDEX_REPOSITORY
    try:
        project_code_runs.build_sample_workspace(work)
        project_code_runs.lib_project_code_runs_launchers.FAMILY_OWN_REPOSITORY = work   # 样本工作区里的门禁阶段、同族脚本照「本仓那一份」认（lib_project_code_runs.py 的 family_own_copy）
        shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-empty")   # 系统级启动文件不读本机的 /etc（换成样本目录）
        project_code_runs.lib_project_code_runs_base.SAMPLE_INDEX_REPOSITORY = os.path.join(work, project_code_runs.SAMPLE_INDEX_REFERENCE_NAME)
        build_sample_scripts(work)
        writer, crash, triage, scribe = "implementation-writer", "crash-verifier", "gate-triage", "kb-scribe"
        tier_binary = "./target/release/deps/crash_enumeration_new_pool_file_creation_stream-0123456789abcdef"
        # (说明, agent_type（None 是主 agent）, 命令, 该拒 2 / 该放 0[, 该记几条检出, 该读进去几份脚本]；后两列不写是 0、0)
        cases = [
            ("实现员跑 herd7 x.litmus", writer, "herd7 x.litmus", 2),
            # env 换目录（-C 目录、--chdir=目录、--chdir 目录）照 cd 判（lib_shell_words.py 的 LIB_SHELL_WORDS_BREAK=env-chdir-ignored 下前五格红：应当是 2，实际 0；放行那一格红：应当是 0，实际 2；
            # 第二到四格先 cd 进 验证包目录，所以不跟 env 换的目录就放过去）
            # env 的短选项合写（-iC 目录）与 -S 拆串（lib_shell_words.py 的 env_option / env_split_string；LIB_SHELL_WORDS_BREAK=env-option-clusters-unparsed 下这三格红，
            ("实现员 cd 到变量目录、经内存包装起 验证包 -p：放行", writer,
             f"D={work}; cd $D && bash scripts/hosts/run-with-memory-cap.sh 12G cargo test -p singlefs-verification --lib", 0),
            # 目录写在变量里、不带 -p 的裸 cargo test（lib_shell_words.py 的 expand_known_variables；LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded 下照拒的五格红：
            ("实现员 D=… 之后 cd $D/验证包目录经内存包装裸跑：放行", writer,
             f"D={work}; cd $D/crates/singlefs-verification && bash scripts/hosts/run-with-memory-cap.sh 12G cargo test --lib", 0),
            ("实现员交回前自查命名：naming-lint.sh：放行", writer, "bash .claude/singlefs-ai-sop/scripts/naming-lint.sh .", 0, 0, 1),   # 末两列：不记检出、读进去 naming-lint.sh 那一份
            # 装置的本体与 cargo run --bin 同判（lib_project_code_runs.py 的 device_main_binary；device-main-as-test 下头一格红：应当是 0，实际 2）
            ("实现员在 bash -c 里跑 --all", writer, "bash -c 'herd7 x.litmus'", 2),
            ("实现员 cd 进 验证包裸跑、经内存包装放行（验证包随时跑）", writer, "cd crates/singlefs-verification && bash ../../scripts/hosts/run-with-memory-cap.sh 4G cargo test --release", 0),
            ("实现员 -p 验证包不挑目标、经内存包装放行", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification", 0),
            ("实现员起 qemu-system", writer, "qemu-system-x86_64 -enable-kvm -m 512", 2),
            ("实现员跑 herd7", writer, "herd7 -conf linux-kernel.cfg litmus/a.litmus", 2),
            ("实现员直接跑 herd7 -version 只取版本号：放行", writer, "herd7 -version", 0),
            ("实现员直接跑 herd7 -version 再带 litmus 文件：照拒", writer, "herd7 -version litmus/a.litmus", 2),
            # 只打工具版本的谁都能跑（tool-versions-counted 下末两格红）
            ("实现员经内存包装起循环里 ls 出来的 验证包测试二进制：命令词认不出照拒", writer,
             "b=$(ls -t ./target/release/deps/overwrite_in_one_instance-* | head -1); bash scripts/hosts/run-with-memory-cap.sh 4G $b", 2),
            ("实现员喂给 bash 的 heredoc 里跑 --all", writer, "bash <<'EOF'\nherd7 x.litmus\nEOF", 2),
            # here-string 与管道喂给 shell 的字面文本当命令判，喂进去的是变量、cat 文件这类的放行并记检出（lib_shell_words.py 的 shell_standard_input；
            ("实现员 here-string 喂给 bash 的是变量：认不出照拒、照记检出", writer, 'bash <<< "$CMD"', 2, 1, 0),
            ("实现员 cat 文件经管道喂给 bash：放行并记检出", writer, "cat steps.sh | bash", 0, 1, 0),
            ("实现员经内存包装 here-string 喂给 bash 跑 验证包的目标：放行", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 4G bash <<< 'cargo test -p singlefs-verification --test overwrite_in_one_instance'", 0),
            # 包在外面照样起那条命令的（/usr/bin/time、flock、systemd-run……）剥掉之后照判；里面经内存包装的算经包装
            ("实现员 /usr/bin/time -v 包一层跑 --all", writer, "/usr/bin/time -v herd7 x.litmus", 2),
            ("实现员 perf stat 包一层跑 --all", writer, "perf stat -e cycles herd7 x.litmus", 2),
            ("实现员内存包装里 /usr/bin/time 包一层跑自己的测试目标", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 4G /usr/bin/time -v cargo test -p singlefs-core --test core_contract", 0),
            ("实现员 /usr/bin/time 包在内存包装外面跑自己的测试目标", writer,
             "/usr/bin/time -v bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract", 0),
            ("主 agent time -p 起全量照拒", None, "time -p herd7 x.litmus", 2),
            ("主 agent time -- 起全量照拒", None, "time -- herd7 x.litmus", 2),
            ("主 agent builtin source 读进全量脚本照拒", None, "builtin source s.sh", 2, 0, 1),
            ("主 agent bash --rcfile 起全量脚本照拒", None, "bash --rcfile /dev/null s.sh", 2, 0, 1),
            ("主 agent bash -- 起全量脚本照拒", None, "bash -- s.sh", 2, 0, 1),
            ("主 agent bash - 起全量脚本照拒", None, "bash - s.sh", 2, 0, 1),
            ("主 agent function 关键字定义的函数里起全量照拒", None, "function f { herd7 x.litmus; }; f", 2),
            ("主 agent cp X{,.bak} 拷出再起全量脚本照拒", None, "cp s.sh{,.bak} && bash s.sh.bak", 2, 0, 1),
            ("主 agent cp 之后按 // 开头的路径起全量脚本照拒", None, f"cp s.sh s-slash.sh && bash /{work}/s-slash.sh", 2, 0, 1),
            ("主 agent env -- 带不合变量名的赋值起全量照拒", None, "env -- a-b=1 herd7 x.litmus", 2),
            ("主 agent env =1 开头就是 = 的赋值起全量照拒", None, "env =1 herd7 x.litmus", 2),
            ("主 agent bash -c -- 的代码里起全量照拒", None, "bash -c -- 'herd7 x.litmus'", 2),
            ("builtin 后面不是内建：bash 报错不跑，放行", None, "builtin herd7 x.litmus", 0),
            ("function 只定义不调用：函数体不跑，放行", None, "function f { herd7 x.litmus; }", 0),
            # 不带 function 关键字的只定义函数同样不判函数体（LIB_SHELL_WORDS_BREAK=posix-uncalled-functions-run 与 uncalled-functions-run 下这一格红）
            ("名字() { 函数体 } 只定义不调用：函数体不跑，放行", None, "f() { herd7 x.litmus; }", 0),
            ("bash -s -- 后面的词是位置参数、不是脚本：放行", None, "bash -s -- s.sh", 0, 0, 0),
            ("主 agent function 定义之后经命令替换调用照拒", None, "function f { herd7 x.litmus; }; $(echo f)", 2),
            ("主 agent function 定义之后引号拆开拼写调用照拒", None, 'function foo { herd7 x.litmus; }; "fo"o', 2),
            ("主 agent function 定义之后花括号拆开拼写调用照拒", None, "function foo { herd7 x.litmus; }; fo{o,}", 2),
            ("主 agent function 定义之后反斜杠拆开拼写调用照拒", None, "function foo { herd7 x.litmus; }; fo\\o", 2),
            #   与 PROJECT_CODE_GUARD_BREAK=cat-concatenation-unseen 下第二格红，redirect-fd-duplication-unseen 下第三格红，各连着「：记几条检出」「：读进去几份脚本」）
            ("主 agent cat 01> 拷出再起全量脚本照拒", None, "cat s.sh 01> s-fd01.sh && bash s-fd01.sh", 2, 0, 1),
            ('主 agent cat 源 "2"> 拷出（引号里的 2 是源）再起全量脚本照拒', None, 'cat s.sh "2"> s-fdq.sh ; bash s-fdq.sh', 2, 0, 1),
            ("主 agent cat 3> 再 >&3 拷出再起全量脚本照拒", None, "cat s.sh 3>s-fd3.sh >&3; bash s-fd3.sh", 2, 0, 1),
            #   （LIB_SHELL_WORDS_BREAK=redirect-target-braces-expanded 下第二格与它的「：记几条检出」「：读进去几份脚本」红；起的那一份不在，记一条检出）
            ("别的命令带几个文件参数写到 > 文件不当 cat 接起来：认不出写法写出再起，照拒", None, "wc -l s.sh light.sh > s-wc.sh; bash s-wc.sh", 2, 0, 0),
            ("重定向目标花括号展开成几个词：bash 报 ambiguous redirect 不写，放行", None, "cat s.sh > s-amb{,}.sh; bash s-amb.sh", 0, 1, 0),
            #   redirect-fd-duplication-unseen 下第二格红，各连着「：记几条检出」「：读进去几份脚本」；放行的那一格起的那一份不在，记一条检出）
            ("heredoc 写文件：cat <<EOF 2> 写的那一份拿不到正文：认不出写法写出再起，照拒", None, "cat <<'EOS' 2>s-hd2.sh\nherd7 x.litmus\nEOS\nbash s-hd2.sh", 2, 0, 0),
            ("主 agent heredoc 写文件：cat <<EOF 3> 再 >&3 写出再起全量脚本照拒", None, "cat <<'EOS' 3>s-hd3.sh >&3\nherd7 x.litmus\nEOS\nbash s-hd3.sh", 2, 0, 1),
            # bash -c 之后照 bash 只认短选项、-n 只查语法；env 认到赋值之后不再认选项（shell-long-options-after-short 下第一格红，
            #   shell-syntax-only-code-run 下第二格红，env-options-after-assignment 下第四格红；第三、五格钉住长选项写在前面、选项写在赋值前面的照判）
            ("bash -c 之后的长选项：bash 报错不跑，放行", None, "bash -c --posix 'herd7 x.litmus'", 0),
            ("bash -c -n 只查语法：放行", None, "bash -c -n 'herd7 x.litmus'", 0),
            ("主 agent bash 长选项写在 -c 前面照拒", None, "bash --posix -c 'herd7 x.litmus'", 2),
            ("env =1 之后的 -u 是命令词：env 报错不跑，放行", None, "env =1 -u X herd7 x.litmus", 0),
            ("主 agent env -u 写在赋值前面照拒", None, "env -u X =1 herd7 x.litmus", 2),
            # 带步长、两端宽度不一的补零序列照 bash 补到长的那一边（brace-sequence-width-padded-ends-only 下这一格与它的「：读进去几份脚本」红）
            ("主 agent cp 带步长、两端宽度不一的补零序列拷出再起全量脚本照拒", None, "cp s.sh s{01..100..100}.sh && bash s001.sh", 2, 0, 1),
            # ANSI-C 引号 $'…' 里的 \' 不收尾（ansi-c-quotes-unseen 下这一格红）
            ("主 agent $'…' 里带 \\' 之后 bash <<EOF 里起全量照拒", None, "echo $'a\\'b'; bash <<'EOF'\nherd7 x.litmus\nEOF", 2),
            # ANSI-C 引号解开之后，定义里的 \t 隔开了测试目标名：数组元素赋值与 += 追加照 bash 是赋值、不是命令词（LIB_SHELL_WORDS_BREAK=extended-assignments-as-command 下这一格红）
            ("数组元素赋值与 += 追加是赋值、不是命令词：放行", writer,
             "row=$'x\\tcrash_enumeration_new_pool_file_creation_stream'; seen[$row]=1; rows+=($row)", 0),
            #   第三、四格钉住一项的命令与模式里的命令替换照判）
            ("case 里 ;; 之后的模式词不当命令：放行", None, 'case "$x" in a) echo a;; herd7) echo h;; esac', 0),
            ("主 agent case 一项的命令里起全量照拒", None, 'case "$x" in a) herd7 x.litmus;; esac', 2),
            ("主 agent case 的模式里的命令替换起全量照拒", None, 'case "$x" in $(herd7 x.litmus)) :;; esac', 2),
            ("[[ ]] 里 && 之后的词不当命令：放行", writer, "[[ -f README.md && herd7 == x ]] && echo y", 0),
            ("主 agent time -p 普通命令放行", None, "time -p ls", 0),
            ("echo 花括号展开出来的词不当命令：放行", None, "echo cargo{,-x} test --all", 0),
            ("赋值词里的花括号不展开：放行", None, "a={x,y}; echo $a", 0),
            # eval 的字面代码照一条命令判（LIB_SHELL_WORDS_BREAK=eval-unread 下这一格红）
            ("主 agent eval 字面代码里起全量照拒", None, "eval 'herd7 x.litmus'", 2),
            # 同一条命令里先 cp 再 mv 拷出再起（PROJECT_CODE_GUARD_BREAK=copy-by-cp-only 下它的「：读进去几份脚本」红；第四十三节第 2 行）
            ("主 agent 同一条命令里 cp 再 mv 改名再起全量脚本照拒", None, "cp s.sh s-copy.sh && mv s-copy.sh s-moved.sh && bash s-moved.sh", 2, 0, 1),
            # cat 源 > 目的 后面另有 2> 的、ln -s 的相对源（报告 W3 一节；LIB_SHELL_WORDS_BREAK=redirect-fd-dropped 下前两格红，link-source-from-current-directory 下后两格红，
            #   各连着的「：记几条检出」「：读进去几份脚本」也红；放行的两格起的那一份不在，记一条检出）
            ("主 agent cat > 拷出、另把 stderr 丢进 /dev/null 再起全量脚本照拒", None, "cat s.sh > s-cat.sh 2>/dev/null && bash s-cat.sh", 2, 0, 1),
            ("cat > 拷出、起的是 2> 写的那一份：认不出写法写出再起，照拒", None, "cat s.sh > s-cat2.sh 2> s-err.sh && bash s-err.sh", 2, 0, 0),
            ("主 agent ln -s 的相对源从链接所在的目录算、经它起全量脚本照拒", None, "mkdir -p sub && ln -s ../s.sh sub/s-link.sh && bash sub/s-link.sh", 2, 0, 1),
            ("ln -s 的相对源在链接所在的目录里不在：认不出写法写出再起，照拒", None, "mkdir -p sub && ln -s s.sh sub/s-link2.sh && bash sub/s-link2.sh", 2, 0, 0),
            # heredoc 挂在没收尾的命令替换、进程替换里的 bash 上（第四十节第 18 行；LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen 下前两格红）
            ("主 agent $(bash <<EOF) 里起全量照拒", None, "x=$(bash <<'EOF'\nherd7 x.litmus\nEOF\n)", 2),
            ("主 agent <(bash <<EOF) 里起全量照拒", None, "cat <(bash <<'EOF'\nherd7 x.litmus\nEOF\n)", 2),
            ("$(cat <<EOF) 里的正文是数据：放行", None, "x=$(cat <<'EOF'\nherd7 x.litmus\nEOF\n)", 0),
            # 花括号展开出来的空串照 bash 去掉、整数序列照 bash 补零（LIB_SHELL_WORDS_BREAK=brace-empties-kept 下前四格红，
            #   brace-sequence-unpadded 下第五格与它的「：读进去几份脚本」红）
            ("主 agent {,} 写在命令词前面起全量照拒", None, "{,} herd7 x.litmus", 2),
            ("主 agent cp 补零的序列拷出再起全量脚本照拒", None, "cp s.sh s{01..01}.sh && bash s01.sh", 2, 0, 1),
            ("引号写出的空串花括号展开之后留着、当命令词：放行", None, "''{,} herd7 x.litmus", 0),
            # 同一行里先有双引号包着、已收尾的 $(…)，再接顶层的 bash <<EOF（报告 W2 一节；LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed 下这两格红）
            ('主 agent cd "$(git rev-parse --show-toplevel)" 之后 bash <<EOF 里起全量照拒', None,
             "cd \"$(git rev-parse --show-toplevel)\" && bash <<'EOF'\nherd7 x.litmus\nEOF", 2),
            ('主 agent FOO="$(pwd)" bash <<EOF 里起全量照拒', None, "FOO=\"$(pwd)\" bash <<'EOF'\nherd7 x.litmus\nEOF", 2),
            # 起测试的外部子命令、别名与 runner、只列不跑、没标 #[ignore] 的登记用例
            ("实现员 cargo mutants 整个 验证包（随时跑）", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G cargo mutants -p singlefs-verification", 0),
            # 喂的样本与暂存区比（样本的 setup.sh 起 验证包；LIB_PROJECT_CODE_RUNS_BREAK=stage-samples-fixtures-unchecked 下这一格红）
            # 登记表的键按真实路径认（LIB_PROJECT_CODE_RUNS_BREAK=selftest-registry-by-path-tail 下第一格红）
            ("工具实现员起带 case 模式行 .claude/gate.d/lib/*.py) 的脚本：放行", "tooling-writer", "bash case-pattern.sh", 0, 0, 1),
            # 只查清单、只生成、只读的放行（这一组只过 scan_decision，直接起本仓的门禁阶段不在这一层拒：LIB_SHELL_WORDS_BREAK=arguments-as-executed 下
            #   python3 - 读几份门禁那一格照样放行，红的是 judge_task_cases 之后走完整判法的同一条）
            ("实现员 gate-overlap.py --list 查清单", writer, "python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list", 0),
            ("实现员看门禁阶段的格名表", writer, "bash .claude/gate.d/code-tooling.sh --list", 0),
            ("书记员跑阶段的 --write（生成）", scribe, "bash .claude/gate.d/doc-decisions.sh --write", 0),
            ("工具实现员在门禁目录里 python3 - 读几份门禁（2026-09-28 误拦的那一条原样，目录换成自检的临时工作区）", "tooling-writer",
             f"cd {work}/.claude/gate.d && python3 - doc-text.sh doc-process-records.sh doc-experiments.sh <<'EOF'\nimport re, sys\n"
             "for path in sys.argv[1:]:\n    text = open(path, encoding=\"utf-8\").read()\nEOF", 0),
            # 双机分片的驱动按树根里键那一行判（lib_project_code_runs.py 的 shard-driver-by-name 打开时第一格红）
            ("实现员跑自己动到的测试目标", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --test overwrite_in_one_instance", 0),
            ("实现员 clippy --all-targets", writer, "bash scripts/hosts/run-with-memory-cap.sh 8G cargo clippy --all-targets -- -D warnings", 0),
            ("实现员 build --all-targets", writer, "bash scripts/hosts/run-with-memory-cap.sh 8G cargo build --offline --all-targets", 0),
            ("实现员 echo 引号里的命令", writer, 'echo "herd7 x.litmus"', 0),
            ("实现员写进笔记的 heredoc 正文", writer, "cat > note.md <<'EOF'\nherd7 x.litmus\nEOF", 0),
            ("实现员 -p harness --lib", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --lib", 0),
            ("执行员在 research 里跑自己的 bin", "experiment-runner",
             "cd scripts/experiment-apparatus && bash ../hosts/run-with-memory-cap.sh 16G cargo test --release --bin e160-random-small-read-share", 0),
            # 跑编译出来的代码经没经内存包装（与它认的哪一类无关）：子 agent 不经它的拒，经它包着的（连同 bash -c、它起的脚本）放行，主 agent 不判
            ("实现员经内存包装再经 capped.sh", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 4 cargo test -p singlefs-core --lib", 0),
            ("实现员 capped.sh 在外、内存包装在里", writer, "capped.sh 4 run-with-memory-cap.sh 4G cargo test -p singlefs-core --lib", 0),
            ("实现员 timeout 套在内存包装外面", writer, "timeout 900 bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --lib", 0),
            ("实现员内存包装里 bash -c 跑", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G bash -c 'cd crates/singlefs-core && cargo test --lib'", 0),
            ("实现员经内存包装直接执行测试二进制", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G ./target/debug/deps/core_contract-0123456789abcdef", 0),
            ("实现员只编不跑 cargo test --no-run", writer, "bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --no-run -p singlefs-core", 0),
            ("实现员经内存包装起那份脚本", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G bash unwrapped.sh", 0, 0, 1),
            ("实现员起的脚本里有 cargo test、这一层放行：读正文", writer, "bash unwrapped.sh", 0, 0, 1),
            ("实现员 heredoc 写出 cargo run 再起它、这一层放行：读正文", writer,
             "cat > gen-unwrapped.sh <<'EOF'\ncargo run --release --bin e160-random-small-read-share\nEOF\nbash gen-unwrapped.sh", 0, 0, 1),
            ("起编译：主 agent 裸起 cargo fmt 与 cargo metadata：放行", None, "cargo fmt --all && cargo metadata --format-version 1 --no-deps", 0),
            ("起编译：实现员裸起 cargo fmt 与 cargo metadata：放行", writer, "cargo fmt --all && cargo metadata --format-version 1 --no-deps", 0),
            ("实现员 bash 起含它的脚本", writer, "bash s.sh", 2, 0, 1),
            ("实现员 ./ 直接执行含它的脚本", writer, "./s.sh", 2, 0, 1),
            ("实现员经 capped.sh 4 bash 起含它的脚本", writer, "capped.sh 4 bash s.sh", 2, 0, 1),
            ("实现员 source 含它的脚本", writer, "source s.sh", 2, 0, 1),
            ("实现员 . 读进含它的脚本、nice 与 timeout 包一层", writer, "nice -n 19 timeout 600 . s.sh", 2, 0, 1),
            ("实现员 cd 之后按绝对路径起含它的脚本、输出落文件", writer, f"cd {work} && bash {work}/s.sh > {work}/chain.log 2>&1", 2, 0, 1),
            ("实现员起的脚本里再调含它的脚本", writer, "bash outer.sh", 2, 0, 2),
            ("主 agent 执行含它的脚本", None, "bash s.sh", 2, 0, 1),
            ("实现员起调回自己的脚本（只读一遍）", writer, "bash loop.sh", 0, 0, 1),
            ("实现员 capped.sh 后面不带斜杠的命令词从 PATH 找，不读当前目录的同名脚本", writer, "bash scripts/hosts/capped.sh 4 s.sh", 0, 0, 0),
            ("实现员 ./ 执行 python 程序不读（射程之外）", writer, "./tool", 0, 0, 0),
            ("实现员 ./ 执行编译出来的程序不读", writer, "./prog", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的编译产物：不读也不记", writer, "./bigprog", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的 python 程序：不读也不记", writer, "./bigtool", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的 shell 脚本：放行并记检出", writer, "./big.sh", 0, 1, 0),
            ("实现员起 doc-lint.sh 第 43 行那种命令替换：片段不当路径", writer, "bash doc-lint-line-43.sh", 0, 0, 1),
            ("实现员起 doc-lint.sh 第 70 行那种命令替换：片段不当路径", writer, "bash doc-lint-line-70.sh", 0, 0, 1),
            ("实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径", writer, "bash substitution-fragments.sh", 0, 1, 1),   # `$(dirname "$0")/helper.sh` 换成脚本所在目录之后照路径认，helper.sh 不在记一条
            ("实现员起的脚本里命令替换里跑 --all", writer, "bash substitution-heavy.sh", 2, 0, 1),
            ("实现员起的脚本里 command -v herd7：查名字不是跑", writer, "bash command-lookup.sh", 0, 0, 1),
            ("实现员起的脚本里数组赋值的元素：不当命令、不当路径", writer, "bash array-assignment.sh", 0, 0, 1),
            ("实现员直接跑 command -v herd7", writer, "command -v herd7 >/dev/null && echo 在", 0),
            ("实现员 command herd7（不带 -v）照拒", writer, "command herd7 -conf linux-kernel.cfg litmus/a.litmus", 2),
            ("实现员 bash 起一个目录：不读也不记", writer, "bash crates", 0),
            # 进程替换 `<(…)`：括号里的命令另按一条完整的命令判，`)` 之后的词是外层命令的参数（第一例是现场误拒的原句）
            ("实现员同一行里起 .sh 之后把 python 源码喂给 python3 -：正文不当命令", writer,
             "sed -n 1p light.sh > body.py && bash light.sh && python3 - <<'PY'\nsource = open(\"body.py\", encoding=\"utf-8\").read()\n"
             "body = open(\"/tmp/claude-1000/nowhere/hook_body.py\", encoding=\"utf-8\")\nusage = \"\"\"\nherd7 x.litmus\n\"\"\"\nPY", 0, 0, 1),
            # 喂给 bash 的 heredoc 里出现 python 那种写法：`source =` 不算读脚本，带逗号的词不当路径
            ("实现员喂给 bash 的正文里有 source = 与带逗号的路径：不当脚本", writer,
             "bash light.sh; bash <<'EOF'\nsource = open(\"body.py\", encoding=\"utf-8\").read()\n"
             "body = open(\"/tmp/claude-1000/nowhere/hook_body.py\", encoding=\"utf-8\")\nEOF", 0, 0, 1),
            ("实现员 bash 起通配写出来的脚本路径：不当路径、不记检出", writer, "bash ./chain-*.sh", 0, 0, 0),
            # pattern-expansions-bare 下「写成通配的 capped.sh」那一格红）
            ("实现员写成通配的 capped.sh 包全量：照拒", writer, f"cd {repo} && bash scripts/hosts/capped.s? 4 herd7 x.litmus", 2),
            ("实现员合法的通配 ls scripts/*.sh：放行", writer, f"cd {repo} && ls scripts/*.sh", 0, 0, 0),
            # 方括号里的 POSIX 字符类、等价类、整理符号交给 bash 展开（LIB_PROJECT_CODE_RUNS_BREAK=posix-brackets-by-python-glob 下前四格红，
            # posix-brackets-unresolved-literal 下第四格红）
            ("实现员合法的 POSIX 字符类 ls scripts/[[:alpha:]]*.sh：放行", writer, f"cd {repo} && ls scripts/[[:alpha:]]*.sh", 0, 0, 0),
            # two-logs.sh 包一层不让里面那条脱离闸（LIB_PROJECT_CODE_RUNS_BREAK=two-logs-opaque 下「实现员经 two-logs.sh 起全量 cargo test：照拒」红）
            ("实现员经 two-logs.sh 起全量 cargo test：照拒", writer,
             f"bash {repo}/scripts/hosts/two-logs.sh /tmp/claude-1000/somewhere/log -- herd7 x.litmus", 2),
            # 前两格与「花括号写在包装上」红，brace-command-words-bare 下「花括号写在包装上」红）
            ("实现员合法的花括号 ls scripts/{a,b}.sh：放行", writer, f"cd {repo} && ls scripts/{{a,b}}.sh", 0, 0, 0),
            ("实现员合法的参数通配 cat crates/*.toml：放行", writer, f"cd {repo} && cat crates/*.toml", 0, 0, 0),
            ("实现员起不存在的脚本：放行并记检出", writer, "bash missing.sh", 0, 1, 0),
            ("实现员 ./ 执行不存在的脚本：放行并记检出", writer, "./missing.sh", 0, 1, 0),
            ("实现员 bash 起不是文本的文件：放行并记检出", writer, "bash blob.bin", 0, 1, 0),
            ("实现员起超过上限的脚本：放行并记检出", writer, "bash big.sh", 0, 1, 0),
            ("实现员起的脚本嵌套 7 层：读到第 6 层、看不全，放行并记检出", writer, "bash deep1.sh", 0, 1, 6),
            ("实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它", writer,
             "cat > gen.sh <<'EOF'\n#!/usr/bin/env bash\nherd7 x.litmus\nEOF\nbash gen.sh", 2, 0, 1),
            ("实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行", writer,
             "cat <<'EOF' > gen2.sh\nherd7 x.litmus\nEOF\nchmod +x gen2.sh && ./gen2.sh", 2, 0, 1),
            ("实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source", writer,
             "tee gen3.sh <<'EOF' >/dev/null\nherd7 x.litmus\nEOF\nsource gen3.sh", 2, 0, 1),
            ("实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它", writer,
             "cat > gen4.sh <<-EOF\n\therd7 x.litmus\n\tEOF\nbash gen4.sh", 2, 0, 1),
            ("实现员 cd 进子目录写出、回来按相对路径起", writer,
             "mkdir -p gen-dir && cd gen-dir && cat > here.sh <<'EOF'\nherd7 x.litmus\nEOF\ncd .. && bash gen-dir/here.sh", 2, 0, 1),
            ("实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本", writer,
             "cat > gen-outer.sh <<'EOF'\ncat > gen-inner.sh <<'IN'\nherd7 x.litmus\nIN\nbash gen-inner.sh\nEOF\nbash gen-outer.sh", 2, 0, 2),
            ("实现员 bash -c 里写出含 --all 的脚本再起它", writer,
             "bash -c 'cat > gen-c.sh <<EOF\nherd7 x.litmus\nEOF\nbash gen-c.sh'", 2, 0, 1),
            ("实现员命令替换里写出含 --all 的脚本再起它", writer,
             "out=\"$(cat > gen-sub.sh <<'EOF'\nherd7 x.litmus\nEOF\nbash gen-sub.sh)\"", 2, 0, 1),
            ("实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份", writer,
             "cat > gen-maker.sh <<'EOF'\ncat > gen-made.sh <<'IN'\nherd7 x.litmus\nIN\nEOF\nbash gen-maker.sh && bash gen-made.sh", 2, 0, 2),
            ("实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在", writer,
             "cat > gen8.sh <<'EOF'\nherd7 x.litmus\nEOF\ncat >> gen8.sh <<'EOF'\necho 尾\nEOF\nbash gen8.sh", 2, 0, 1),
            ("实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在", writer,
             "tee -a s.sh <<'EOF'\necho 尾\nEOF\nbash s.sh", 2, 0, 1),
            ("实现员 cat -n 带选项写出：认不出，照拒", writer,
             "cat -n > gen-numbered.sh <<'EOF'\nherd7 x.litmus\nEOF\nbash gen-numbered.sh", 2, 0, 0),
            ("实现员 heredoc 写出 python 程序再 ./ 执行：不读也不记", writer,
             "cat > gen.py <<'EOF'\n#!/usr/bin/env python3\nprint('herd7 x.litmus')\nEOF\n./gen.py", 0, 0, 0),
            ("实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件", writer, "cp s.sh copied.sh && bash copied.sh", 2, 0, 1),
            ("实现员 cp -f 到目录里再起：读源文件", writer, "cp -f s.sh elsewhere/ && bash elsewhere/s.sh", 2, 0, 1),
            ("实现员 cp 同一条命令里 heredoc 写出的脚本再起", writer,
             "cat > gen6.sh <<'EOF'\nherd7 x.litmus\nEOF\ncp gen6.sh gen7.sh && bash gen7.sh", 2, 0, 1),
            ("实现员 cp 的源不在：认不出，照拒", writer, "cp nowhere.sh copied-missing.sh && bash copied-missing.sh", 2, 0, 0),
            ("实现员 printf 写出再起：认不出，照拒", writer, "printf 'herd7 x.litmus\\n' > printed.sh && bash printed.sh", 2, 0, 0),
            ("实现员 ./ 直接执行 .md 文件：不当脚本读、不记", writer, "./notes.md", 0, 0, 0),
            ("实现员 ./ 直接执行没扩展名、没 #! 的文本：不当脚本读、不记", writer, "./plain-text", 0, 0, 0),
            ("实现员 ./ 直接执行还不在、没扩展名的：不记「不存在」", writer, "bash scripts/hosts/run-with-memory-cap.sh 4G ./target/release/not-built-yet", 0, 0, 0),
            ("实现员 ./ 直接执行没扩展名、#! 指到 bash 的脚本：读进去照拒", writer, "./runner", 2, 0, 1),
            ("实现员 ./ 直接执行 .py 名字、#! 指到 sh 的脚本：读进去照拒", writer, "./mislabeled.py", 2, 0, 1),
            ("实现员同一条命令里起含 --all 的 s.sh 两遍：只判一遍", writer, "bash s.sh; ./s.sh", 2, 0, 1),
            ("实现员起的脚本里三种起法各起一遍 s.sh：s.sh 只判一遍", writer, "bash twice.sh", 2, 0, 2),
            ("实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判", writer,
             "bash inner-rel.sh; cat > inner-rel.sh <<'EOF'\nherd7 x.litmus\nEOF\nbash inner-rel.sh", 2, 0, 2),
            ("实现员先起里面起了不存在脚本的 has-missing.sh、再经 outer-missing.sh 起它：检出照复用的那一遍记两条", writer,
             "bash has-missing.sh; bash outer-missing.sh", 0, 2, 2),
            ("实现员同一份 rel.sh 换个当前目录再起：按相对路径起的脚本变了，重判", writer, "bash rel.sh; cd sub && bash ../rel.sh", 2, 0, 4),
            ("实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判", writer, "bash deep1.sh; bash deep6.sh", 2, 1, 8),
            ("实现员先起 cyc-a.sh（里面绕回自己）、再起 cyc-b.sh：cyc-b.sh 那一遍绕回外层，重判", writer, "bash cyc-a.sh; bash cyc-b.sh", 2, 0, 3),
        ]
        def judge_cases(case_list):
            for case in case_list:
                label, agent_type, command, want = case[:4]
                want_notices, want_read = case[4:6] if len(case) > 4 else (0, 0)
                hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
                if agent_type:
                    hook_input["agent_type"] = agent_type
                decision = scan_decision(hook_input, work)
                results.append((label, want, decision.code))
                results.append((f"{label}：记几条检出", want_notices, len(decision.notices)))
                results.append((f"{label}：读进去几份脚本", want_read, len(decision.scripts_read)))
        judge_cases(cases)
        # 本仓的代码（门禁阶段与格、上游的 lint、门禁结构检查）、会编的 cargo、编出来的二进制直接起，由 task_server_decision 拒（scan_decision 只判外部程序、认不出的写法与编译目录）：
        # 这几格走完整的判法，只看退出码（读进去几份脚本、记几条检出由上面 scan_decision 的格钉住）；本仓的脚本凭路径尾部在本仓里有这份文件认，样本工作区放了同名的文本（build_sample_scripts）。
        # PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed 下这几格全红（应当是 2，实际 0）
        def judge_task_cases(case_list):
            for label, agent_type, command in case_list:
                hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
                if agent_type:
                    hook_input["agent_type"] = agent_type
                results.append((label, 2, task_server_decision(hook_input, work).code))
        judge_task_cases([
            ("实现员经内存包装起 cargo test -p 当前目录探针自己的包名：照拒", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 8G bash -c 'cd probes/linked && cargo test -p attack-probe'"),
            ("实现员经内存包装与 bash -c 起 cargo run --package= 点名探针：照拒", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 8G bash -c 'cd probes/linked && cargo run --release --package=attack-probe'"),
            ("实现员经内存包装 env -C 换进探针再 cargo test -p 探针自己的包名：照拒", writer,
             "bash scripts/hosts/run-with-memory-cap.sh 8G env -C probes/linked cargo test -p attack-probe"),
            ("实现员 cd ${D} 工作区根、经 timeout、nice、capped.sh、内存包装四层全量：照拒", writer,
             f"D={work}; cd ${{D}} && timeout 235 nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 12G cargo test"),
            ("主 agent D=… 之后 cd $D 全量：照拒", None, f"D={work}; cd /tmp && cd $D && bash {repo}/scripts/hosts/run-with-memory-cap.sh 16G cargo test"),
            ("实现员交回前自查命名：code-source-discipline.sh 点名那四格也是直接起本仓的门禁阶段，照拒（写进报告交主 agent 跑）", writer,
             "bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names ."),
            ("实现员交回前自查命名：经范围声明与 nice 起、等号写法点名一格，照拒", writer,
             "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=命名四格; 范围=1 格; 为什么=交回前自查命名' nice -n 19 bash .claude/gate.d/code-source-discipline.sh --check=data-names ."),
            ("实现员交回前自查命名：点名里夹着别的格：照拒", writer, "bash .claude/gate.d/code-source-discipline.sh --check vague-names,format-constants ."),
            ("实现员交回前自查命名：不带 --check 整道：照拒", writer, "bash .claude/gate.d/code-source-discipline.sh ."),
            ("实现员在仓根裸跑 cargo test", writer, "cargo test"),
            ("主 agent 裸 cargo run E152", None, "cd scripts/experiment-apparatus && cargo run --release --bin e152-file-system-benchmark"),
            ("主 agent {cargo,} 空的一份起全量照拒", None, "bash scripts/hosts/run-with-memory-cap.sh 16G {cargo,} test --all"),
            ("主 agent cargo {,test} 空的一份起全量照拒", None, "bash scripts/hosts/run-with-memory-cap.sh 16G cargo {,test} --all"),
            ("书记员跑 doc-decisions（直接起本仓的门禁阶段）", scribe, "nice -n 19 bash .claude/gate.d/doc-decisions.sh"),
            ("通用 agent 跑 code-tooling.sh 的 agent-write-scope 格（直接起本仓的门禁阶段）", "general-purpose", "bash .claude/gate.d/code-tooling.sh --check agent-write-scope"),
            ("主 agent 跑 code-tooling.sh（直接起本仓的门禁阶段）", None, "bash .claude/gate.d/code-tooling.sh"),
            ("主 agent 跑 gate-lint.sh", None, "GATE_LINT_DIR=scripts bash .claude/singlefs-ai-sop/scripts/gate-lint.sh"),
            ("实现员跑 gate-structure-check.py", writer, "python3 scripts/gate/gate-structure-check.py"),
            ("主 agent 跑 hooks-registered.sh", None, "bash .claude/singlefs-ai-sop/scripts/hooks-registered.sh ."),
            ("实现员复现输入那一条（反引号里的 walk.rs）：末尾的 doc-lint.sh 是直接起的本仓脚本，照拒", writer, SLOW_REPRODUCTION_COMMAND),
            ("一百三十七#8：工具实现员经 multi-host-run.sh --peer 起门禁格，照拒", "tooling-writer",
             f"bash {repo}/scripts/hosts/multi-host-run.sh --peer --memory-cap 8G -- bash .claude/gate.d/code-tooling.sh --check agent-write-scope ."),
        ])
        # 「python3 - 读几份门禁」走完整的判法：python3 - 后面的词是程序的参数、不是起的脚本，工具实现员放行。上面 cases 里同一条只过 scan_decision，
        # 直接起本仓门禁阶段只在这一层拒；LIB_SHELL_WORDS_BREAK=arguments-as-executed 下 doc-text.sh 被认成 python3 起的本仓门禁阶段（样本工作区里那份）而拒，这一格红
        gate_reading = task_server_decision({"tool_name": "Bash", "cwd": work, "agent_type": "tooling-writer", "tool_input": {"command": (
            f"cd {work}/.claude/gate.d && python3 - doc-text.sh doc-process-records.sh doc-experiments.sh <<'EOF'\nimport re, sys\n"
            "for path in sys.argv[1:]:\n    text = open(path, encoding=\"utf-8\").read()\nEOF")}}, work)
        results.append(("工具实现员在门禁目录里 python3 - 读几份门禁：走完整的判法，放行", 0, gate_reading.code))
        # 认不出的一律拒（偏严）的各格。每格后面的注释写弄坏它的开关
        for relative, text in (("srclib/entry.sh", '#!/usr/bin/env bash\nsource "$(dirname "${BASH_SOURCE[0]}")/lib.sh"\n'),
                               ("srclib/lib.sh", "herd7 x.litmus\n"),
                               ("launcher-function.sh", '#!/usr/bin/env bash\nrun_logged() { local log=$1; shift; "$@" > "$log" 2>&1; }\nrun_logged out.log herd7 x.litmus\n'),
                               ("family-copy/scripts/hosts/run-with-memory-cap.sh", '#!/usr/bin/env bash\nshift; exec "$@"\n'),
                               ("path-dir/runs-all.sh", "#!/usr/bin/env bash\nherd7 x.litmus\n"),
                               ("foreign-dispatch/scripts/hosts/multi-host-run.sh", '#!/usr/bin/env bash\nwhile [[ $# -gt 0 && $1 != -- ]]; do shift; done; shift; exec "$@"\n')):
            os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
            os.chmod(os.path.join(work, relative), 0o755)
        batch137_cases = [
            # 三种静默漏拒
            ("一百三十七：命令替换里不加括号的 case 模式不提前收尾，里面的 --all 照拒", None, 'echo "$(case a in a) herd7 x.litmus;; esac)"', 2),   # LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen
            ("一百三十七：命令替换里注释的 ) 不收尾，后面的 --all 照拒", None, 'echo "$(echo hi # )\nherd7 x.litmus)"', 2),                        # 同上
            ("一百三十七：coproc 后面那条照判", None, "coproc herd7 x.litmus", 2),                                                            # LIB_SHELL_WORDS_BREAK=coproc-as-command
            ("一百三十七：cargo 参数写成前面赋过值的变量，展开之后照判（--lib 放行）", None, "F=--lib; bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-core $F", 0),          # LIB_PROJECT_CODE_RUNS_BREAK=cargo-arguments-unexpanded
            ("一百三十七：bash -c \"$(cat 文件)\" 照拒", writer, 'bash -c "$(cat light.sh)"', 2),                                               # 同上
            ("一百三十七：bash <(…) 照拒", writer, "bash <(echo 'cargo test -p singlefs-core --lib')", 2),                                      # 同上
            ("一百三十七：eval 变量里拼的命令照拒", writer, "c='cargo build'; eval \"$c\"", 2),                                                  # 同上
            ("一百三十七：source <(…) 照拒", writer, "source <(echo true)", 2),                                                                 # 同上
            ("一百三十七：命令词是 $(which cargo) 照拒", writer, "$(which cargo) build", 2),                                                    # 同上
            ("一百三十七：set -- 之后 \"$@\" 照拒", writer, 'set -- cargo build; "$@"', 2),                                                      # 同上
            ("一百三十七：for 循环变量当脚本路径照拒", writer, "for f in light.sh; do bash $f; done", 2),                                         # 同上
            ("一百三十七：PATH 指到放脚本的目录、命令词不带斜杠的 .sh 照读正文", writer, f"PATH={work}/path-dir:$PATH runs-all.sh", 2, 0, 1),                  # PROJECT_CODE_GUARD_BREAK=path-scripts-unread
            ("一百三十七：bash 起通配写出的脚本路径照 bash 展开读正文", writer, "bash ./s.s?", 2, 0, 1),                                       # PROJECT_CODE_GUARD_BREAK=glob-scripts-unread
            ("一百三十七：echo 写出再起照拒", writer, "echo 'cargo build' > gen-echo.sh; bash gen-echo.sh", 2),                               # PROJECT_CODE_GUARD_BREAK=unrecognized-writes-allowed
            ("一百三十七：python 写出再起照拒", writer, "python3 -c \"open('gen-py.sh','w').write('true')\"; bash gen-py.sh", 2),                # 同上
            # 拷出写法 >>、&>、tee：认成拷贝，照拷来的内容判
            ("一百三十七：cat 源 >> 拷出再起，照写出的内容判", writer, "cat s.sh >> gen-append.sh; bash gen-append.sh", 2, 0, 1),               # PROJECT_CODE_GUARD_BREAK=copy-redirections-unseen
            ("一百三十七：cat 轻脚本 &> 拷出再起，照内容判、放行", writer, "cat light.sh &> gen-both.sh; bash gen-both.sh", 0, 0, 1),           # 同上
            ("一百三十七：cat 源 | tee 拷出再起，照写出的内容判", writer, "cat s.sh | tee gen-tee.sh >/dev/null; bash gen-tee.sh", 2, 0, 1),   # PROJECT_CODE_GUARD_BREAK=tee-copy-unseen
            ("一百三十七：tee 目的 < 轻脚本 拷出再起，照内容判、放行", writer, "tee gen-tee2.sh < light.sh >/dev/null; bash gen-tee2.sh", 0, 0, 1),  # 同上
            # 标准输入从文件来的 shell：读那份文件判
            ("一百三十七：bash -s < 文件 读那份文件判", writer, "bash -s < s.sh", 2, 0, 1),                                                      # LIB_SHELL_WORDS_BREAK=shell-standard-input-file-unread
            ("一百三十七：cat 文件 | bash 读那份文件判", writer, "cat s.sh | bash", 2, 0, 1),                                                    # 同上
            ("一百三十七：sh -s < 轻脚本放行", writer, "sh -s < light.sh", 0, 0, 1),                                                             # 同上
            # 名单外启动器
            ("一百三十七：xargs bash 起读进来的脚本照拒", writer, "echo s.sh | xargs bash", 2),                                                  # 同上
            # 派发起法剥开照判（LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen 下红的格逐格写在文件头）
            # 函数当启动器（脚本里经自己的 run_logged 起）
            ("一百三十七：脚本里经自己的 run_logged 起 --all 照拒", writer, "bash launcher-function.sh", 2, 0, 1),                               # PROJECT_CODE_GUARD_BREAK=launcher-functions-unjudged
            # 读正文时展开入口按自己所在目录 source 的库（第一百一十八节跟进）
            ("一百三十七：脚本 source 自己目录下的库，读正文时库也读", writer, "bash srclib/entry.sh", 2, 0, 2),                                # LIB_PROJECT_CODE_RUNS_BREAK=sourced-libraries-unread
            # 别处树里的内存包装拷贝：读正文、不算经内存包装（第一百一十八节「包装脚本同一口径」）
            # 喂给 shell 的是别的命令的输出（认不出喂进去的是什么）：照拒、照记检出（PROJECT_CODE_GUARD_BREAK=unjudged-shell-input-allowed 下红）
            ("一百三十七：grep 的输出经管道喂给 bash：认不出照拒", writer, "grep -h echo light.sh | bash", 2, 1, 0),
            ("一百三十七：别处树里的 run-with-memory-cap.sh 拷贝照读正文", writer,
             f"bash {work}/family-copy/scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract", 0, 0, 1),     # PROJECT_CODE_GUARD_BREAK=wrapper-copies-trusted
            ("一百三十七：函数体外引号里的花括号不当认不准的调用，只定义的函数体不判：放行", None,
             "function f() { herd7 x.litmus; }\nprintf '%s\\n' 'echo a{,}' > ./t3-137.sh", 0),                                            # LIB_SHELL_WORDS_BREAK=quoted-braces-as-unknown
            # BASH_ENV / ENV 指的启动文件当被起的脚本判（件 8f）；下面每一格都在 LIB_SHELL_WORDS_BREAK=startup-files-unread 下红，不红的几格另点名
            ("一百三十七：BASH_ENV 写在命令前、指到字面路径的脚本，正文起全量，照拒", None, f"BASH_ENV={work}/startup-heavy.sh bash -c true", 2, 0, 1),
            ("一百三十七：BASH_ENV 写相对路径，按这条命令的当前目录解，照拒", None, "BASH_ENV=startup-heavy.sh bash -c true", 2, 0, 1),
            ("一百三十七：env BASH_ENV=… 起 bash，照拒", None, f"env BASH_ENV={work}/startup-heavy.sh bash -c true", 2, 0, 1),
            ("一百三十七：前面 export BASH_ENV=… 再起 bash，照拒", None, f"export BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：BASH_ENV 指到的脚本正文起全量，bash 起的又是一份轻脚本，两份都读，照拒", None, f"BASH_ENV={work}/startup-heavy.sh bash startup-light.sh", 2, 0, 2),
            ("一百三十七：BASH_ENV 指到的脚本 source 自己目录下的库，库也读，照拒", None, f"BASH_ENV={work}/srclib/entry.sh bash -c true", 2, 0, 2),
            ("一百三十七：BASH_ENV 指到的脚本正文起全量，实现员照拒", writer, f"BASH_ENV={work}/startup-heavy.sh bash -c true", 2, 0, 1),
            ("一百三十七：BASH_ENV 指到的脚本正文只有轻活，放行", None, f"BASH_ENV={work}/startup-light.sh bash -c true", 0, 0, 1),
            ("一百三十七：BASH_ENV 写成前面赋过字面值的变量，展开之后读正文，照拒", None, "X=startup-heavy.sh; BASH_ENV=$X bash -c true", 2, 0, 1),
            ("一百三十七：ENV 指到的脚本正文起全量，sh 起的，照拒", None, f"ENV={work}/startup-heavy.sh sh -c true", 2, 0, 1),
            ("一百三十七：ENV 指到的脚本正文起全量，bash --posix 起的，照拒", None, f"ENV={work}/startup-heavy.sh bash --posix -c true", 2, 0, 1),
            ("一百三十七：ENV 指到的脚本正文起全量，bash -o posix 起的，照拒", None, f"ENV={work}/startup-heavy.sh bash -o posix -c true", 2, 0, 1),
            ("一百三十七：ENV 指到的脚本，bash 不带 --posix 不读：放行", None, f"ENV={work}/startup-heavy.sh bash -c true", 0),                  # 不读它的格，开关下也不红
            ("一百三十七：BASH_ENV 指到的脚本，sh 不读：放行", None, f"BASH_ENV={work}/startup-heavy.sh sh -c true", 0),                       # 同上
            ("一百三十七：bash -n 只查语法，不读 BASH_ENV：放行", None, f"BASH_ENV={work}/startup-heavy.sh bash -n -c true", 0),               # 同上
            # 下一格另在 LIB_SHELL_WORDS_BREAK=env-ignore-environment-kept 下红（推的：命令前的 BASH_ENV 留到 env -i 后面）
            ("一百三十七：env -i 清掉 BASH_ENV 再起 bash：放行", None, f"BASH_ENV={work}/startup-heavy.sh env -i bash -c true", 0),             # 同上
            ("一百三十七：unset BASH_ENV 之后起 bash：放行", None, f"export BASH_ENV={work}/startup-heavy.sh; unset BASH_ENV; bash -c true", 0), # 同上
            ("一百三十七：BASH_ENV 指到的文件不在，bash 什么都不读：放行、不记检出", None, "BASH_ENV=no-such-startup.sh bash -c true", 0),            # 同上
            ("一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒", writer, "BASH_ENV=$(echo startup-heavy.sh) bash -c true", 2),     # 另在 LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed 下红
            ("一百三十七：BASH_ENV 写成命令替换拼出的路径，认不出，主 agent 照拒", None, "BASH_ENV=$(echo startup-heavy.sh) bash -c true", 2),
            ("一百三十七：BASH_ENV 写成没赋过值的变量，认不出，实现员照拒", writer, "BASH_ENV=$UNASSIGNED bash -c true", 2),
            ("一百三十七：env 的 BASH_ENV 写成命令替换，认不出，实现员照拒", writer, "env BASH_ENV=$(echo startup-heavy.sh) bash -c true", 2),
            ("一百三十七：ENV 写成命令替换，sh 起的，认不出，实现员照拒", writer, "ENV=$(echo startup-heavy.sh) sh -c true", 2),
            # shell 启动时先读的其余几种（件 8g）；括号里是弄坏它的开关（都是 LIB_SHELL_WORDS_BREAK=<项>），startup-files-unread 下读文件的格也红
            # 直接执行的脚本（executed-script-startup-unread）
            ("一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件照读，照拒", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-bash.sh", 2, 0, 2),
            ("一百三十七：直接执行 #!/usr/bin/env bash 的脚本，BASH_ENV 指的文件照读，照拒", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-env-bash.sh", 2, 0, 2),
            ("一百三十七：直接执行没有 #! 的 .sh，从 bash 里起是 bash 另起自己，BASH_ENV 指的文件照读，照拒", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-no-shebang.sh", 2, 0, 2),
            ("一百三十七：env 写的 BASH_ENV，直接执行 #!/bin/bash 的脚本，照拒", None, f"env BASH_ENV={work}/startup-heavy.sh ./exec-bash.sh", 2, 0, 2),
            ("一百三十七：直接执行 #!/bin/sh 的脚本，ENV 指的文件照读，照拒", None, f"ENV={work}/startup-heavy.sh ./exec-sh.sh", 2, 0, 2),
            ("一百三十七：直接执行 #!/bin/zsh 的脚本，ZDOTDIR 下的 .zshenv 照读，照拒", None, f"ZDOTDIR={work}/zdot-env ./exec-zsh.sh", 2, 0, 3),
            ("一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 指的文件只有轻活，放行", None, f"BASH_ENV={work}/startup-light.sh ./exec-bash.sh", 0, 0, 2),
            ("一百三十七：直接执行 #!/bin/sh 的脚本不读 BASH_ENV：放行", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-sh.sh", 0, 0, 1),               # 不读的格，开关下也不红
            ("一百三十七：直接执行 #!/bin/bash 的脚本不读 ENV：放行", None, f"ENV={work}/startup-heavy.sh ./exec-bash.sh", 0, 0, 1),                     # 同上
            ("一百三十七：直接执行 python 脚本不读 BASH_ENV：放行", None, f"BASH_ENV={work}/startup-heavy.sh ./exec-python.py", 0, 0, 0),                 # 同上
            ("一百三十七：直接执行 #! 指到认不出的解释器、环境里设了 BASH_ENV，实现员照拒", writer, f"BASH_ENV={work}/startup-light.sh ./exec-unknown.sh", 2),   # 另在 LIB_PROJECT_CODE_RUNS_BREAK=unrecognized-launches-allowed 下不红（这一句不经 unrecognized_launch）
            ("一百三十七：直接执行 #! 指到认不出的解释器、环境里没设 BASH_ENV / ENV：放行", writer, "./exec-unknown.sh", 0),                                 # 同上
            ("一百三十七：直接执行 #!/bin/bash 的脚本，BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒", writer, "BASH_ENV=$(echo startup-heavy.sh) ./exec-bash.sh", 2, 0, 1),
            # 先赋值后 export、declare -x、typeset -x、export -n、set -a（declared-exports-unseen）
            ("一百三十七：declare -x BASH_ENV=…，再起 bash，照拒", None, f"declare -x BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：typeset -x BASH_ENV=…，再起 bash，照拒", None, f"typeset -x BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：先 BASH_ENV=… 赋值、后 export BASH_ENV，再起 bash，照拒", None, f"BASH_ENV={work}/startup-heavy.sh; export BASH_ENV; bash -c true", 2, 0, 1),
            ("一百三十七：先 export BASH_ENV、后赋值，再起 bash，照拒", None, f"export BASH_ENV; BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：export 过之后换成另一份脚本，起 bash 读的是后一份，照拒", None, f"export BASH_ENV={work}/startup-light.sh; BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：set -a 之后赋值的 BASH_ENV 进环境，照拒", None, f"set -a; BASH_ENV={work}/startup-heavy.sh; bash -c true", 2, 0, 1),
            ("一百三十七：local -x BASH_ENV=… 的函数里起 bash，照拒", None, f"f() {{ local -x BASH_ENV={work}/startup-heavy.sh; bash -c true; }}; f", 2, 0, 1),
            ("一百三十七：export -n 撤掉 export 之后起 bash：放行", None, f"export BASH_ENV={work}/startup-heavy.sh; export -n BASH_ENV; bash -c true", 0, 0, 0),
            ("一百三十七：export -n 带赋值，不进环境：放行", None, f"export -n BASH_ENV={work}/startup-heavy.sh; bash -c true", 0, 0, 0),
            ("一百三十七：declare +x 撤掉 export 之后起 bash：放行", None, f"export BASH_ENV={work}/startup-heavy.sh; declare +x BASH_ENV; bash -c true", 0, 0, 0),
            ("一百三十七：只赋值不 export，bash 看不到：放行", None, f"BASH_ENV={work}/startup-heavy.sh; bash -c true", 0, 0, 0),                    # 不红的格（开关下也一样）
            ("一百三十七：set -a 又 set +a 之后赋值：放行", None, f"set -a; set +a; BASH_ENV={work}/startup-heavy.sh; bash -c true", 0, 0, 0),            # 同上
            ("一百三十七：declare -x BASH_ENV 写成命令替换拼出的路径，认不出，实现员照拒", writer, "declare -x BASH_ENV=$(echo startup-heavy.sh); bash -c true", 2),
            ("一百三十七：先赋值后 export 的 BASH_ENV 是命令替换拼出的路径，认不出，实现员照拒", writer, "BASH_ENV=$(echo startup-heavy.sh); export BASH_ENV; bash -c true", 2),
            # 交互的 bash：不读 BASH_ENV，读 --rcfile / --init-file 指的文件，没给读 HOME 下的 .bashrc；sh -i 与 posix 模式读 ENV（interactive-startup-files-unread）
            ("一百三十七：bash -i 读 HOME 下的 .bashrc，里面起全量，照拒", None, f"HOME={work}/home-heavy bash -i -c true", 2, 0, 1),
            ("一百三十七：bash -ic 合写的 -i 同样读 HOME 下的 .bashrc，照拒", None, f"HOME={work}/home-heavy bash -ic true", 2, 0, 1),
            ("一百三十七：bash --rcfile 文件 -i，文件里起全量，照拒", None, f"bash --rcfile {work}/startup-heavy.sh -i -c true", 2, 0, 1),
            ("一百三十七：bash --init-file 文件 -i，文件里起全量，照拒", None, f"bash --init-file {work}/startup-heavy.sh -i -c true", 2, 0, 1),
            ("一百三十七：bash -i 读 HOME 下只有轻活的 .bashrc：放行", None, f"HOME={work}/home-light bash -i -c true", 0, 0, 1),
            ("一百三十七：bash -i 不读 BASH_ENV，.bashrc 只有轻活：放行", None, f"BASH_ENV={work}/startup-heavy.sh HOME={work}/home-light bash -i -c true", 0, 0, 1),
            ("一百三十七：bash --norc -i 什么都不读：放行", None, f"HOME={work}/home-heavy bash --norc -i -c true", 0, 0, 0),                          # 不红的格（开关下也一样）
            ("一百三十七：bash --rcfile 不带 -i 是非交互，不读：放行", None, f"bash --rcfile {work}/startup-heavy.sh -c true", 0, 0, 0),                    # 同上
            ("一百三十七：bash -i 不读 ENV：放行", None, f"ENV={work}/startup-heavy.sh HOME={work}/home-light bash -i -c true", 0, 0, 1),                    # 同上
            ("一百三十七：sh -i 读 ENV 指的文件，照拒", None, f"ENV={work}/startup-heavy.sh sh -i -c true", 2, 0, 1),
            ("一百三十七：sh -i 读 ENV 指的文件，只有轻活：放行", None, f"ENV={work}/startup-light.sh sh -i -c true", 0, 0, 1),
            ("一百三十七：bash --posix -i 读 ENV 指的文件，照拒", None, f"ENV={work}/startup-heavy.sh bash --posix -i -c true", 2, 0, 1),
            ("一百三十七：bash --rcfile 写成命令替换拼出的路径，认不出，实现员照拒", writer, "bash --rcfile $(echo startup-heavy.sh) -i -c true", 2),
            ("一百三十七：bash -i 的 HOME 写成命令替换，.bashrc 路径认不出，实现员照拒", writer, "HOME=$(echo home-light) bash -i -c true", 2),
            ("一百三十七：bash --rcfile 指到的文件不在，什么都不读：放行", writer, f"bash --rcfile {work}/no-such-rc.sh -i -c true", 0),                              # 同上
            # zsh：读 ZDOTDIR（没设取 HOME）下的 .zshenv、.zshrc，-f、--no-rcs 不读（zsh-startup-files-unread）
            ("一百三十七：zsh 读 ZDOTDIR 下的 .zshenv，里面起全量，照拒", None, f"ZDOTDIR={work}/zdot-env zsh -c true", 2, 0, 2),
            ("一百三十七：zsh 读 ZDOTDIR 下的 .zshrc，里面起全量，照拒", None, f"ZDOTDIR={work}/zdot-rc zsh -c true", 2, 0, 2),
            ("一百三十七：zsh 没设 ZDOTDIR 取 HOME 下的 .zshenv，照拒", None, f"HOME={work}/zdot-env zsh -c true", 2, 0, 2),
            ("一百三十七：前面 export 的 ZDOTDIR，zsh 照读，照拒", None, f"export ZDOTDIR={work}/zdot-env; zsh -c true", 2, 0, 2),
            ("一百三十七：zsh 的启动文件只有轻活：放行", None, f"ZDOTDIR={work}/zdot-light zsh -c true", 0, 0, 2),
            ("一百三十七：zsh -f 不读启动文件：放行", None, f"ZDOTDIR={work}/zdot-env zsh -f -c true", 0, 0, 0),                                              # 不红的格（开关下也一样）
            ("一百三十七：zsh --no-rcs 不读启动文件：放行", None, f"ZDOTDIR={work}/zdot-env zsh --no-rcs -c true", 0, 0, 0),                                 # 同上
            ("一百三十七：bash 不读 ZDOTDIR：放行", None, f"ZDOTDIR={work}/zdot-env bash -c true", 0, 0, 0),                                                 # 同上
            ("一百三十七：zsh 的 ZDOTDIR 写成命令替换，认不出，实现员照拒", writer, "ZDOTDIR=$(echo zdot-env) zsh -c true", 2),
        ]
        judge_cases(batch137_cases)
        # trap、标准输入设备、cd 失败 / 算不出、~+ / ~-、rbash / busybox / dash / ksh。
        # 每格后面注释写弄坏它的开关；startup-heavy.sh（herd7 x.litmus）、startup-light.sh（echo hello）在 work 根
        d137_cases = [
            # H2：trap 'action' 的字符串当 shell 代码判（LIB_SHELL_WORDS_BREAK=trap-action-unread）
            ("d137 H2：trap 'herd7 x.litmus' EXIT 照拒", None, "trap 'herd7 x.litmus' EXIT; true", 2),
            ("d137 H2：trap 'herd7 x.litmus' ERR 照拒", writer, "trap 'herd7 x.litmus' ERR; true", 2),
            ("d137 H2：trap 'herd7 x.litmus' 0 照拒", writer, "trap 'herd7 x.litmus' 0; true", 2),
            ("d137 H2：trap 'bash startup-heavy.sh' DEBUG 读正文照拒", writer, "trap 'bash startup-heavy.sh' DEBUG; true", 2, 0, 1),
            ("d137 H2：trap -- 'herd7 x.litmus' EXIT（-- 之后是 action）照拒", writer, "trap -- 'herd7 x.litmus' EXIT; true", 2),
            ("d137 H2：trap 'rm -f x' EXIT 无害放行", writer, "trap 'rm -f x' EXIT; true", 0),
            ("d137 H2：trap - EXIT 重置、无 action 放行", writer, "trap - EXIT; echo hi", 0),
            # H4：标准输入设备 /dev/stdin、/dev/fd/0（LIB_SHELL_WORDS_BREAK=standard-input-device-script-unread）
            ("d137 H4：bash /dev/stdin < 文件 读那份文件判，照拒", writer, "bash /dev/stdin < startup-heavy.sh", 2, 0, 1),
            ("d137 H4：echo 代码 | bash /dev/stdin 照拒", None, "echo 'herd7 x.litmus' | bash /dev/stdin", 2),
            ("d137 H4：bash /dev/fd/0 <<< 代码 照拒", writer, "bash /dev/fd/0 <<< 'herd7 x.litmus'", 2),
            ("d137 H4：source /dev/stdin <<< 代码 照拒", writer, "source /dev/stdin <<< 'herd7 x.litmus'", 2),
            ("d137 H4：bash /dev/stdin < 轻脚本 放行", writer, "bash /dev/stdin < startup-light.sh", 0, 0, 1),
            # H5：cd 失败 / 目标算不出之后的相对路径脚本（PROJECT_CODE_GUARD_BREAK=relative-script-undeterminable-directory-allowed）
            ("d137 H5：cd 到不在的目录、cd 失败，相对脚本认不出在哪，照拒", None, "cd no-such-d137 2>/dev/null; bash startup-heavy.sh", 2),
            ("d137 H5：cd 目标写成命令替换、算不出，相对脚本认不出，照拒", writer, 'cd "$(echo no-such-d137)" 2>/dev/null; bash startup-heavy.sh', 2),
            ("d137 H5：cd 到在的目录（.）起轻脚本，放行", writer, "cd . ; bash startup-light.sh", 0, 0, 1),
            # H6：~+（$PWD）、~-（$OLDPWD 算不出）（LIB_SHELL_WORDS_BREAK=tilde-plus-minus-unexpanded）
            ("d137 H6：bash ~+/脚本 照 $PWD 解、读正文照拒", writer, "bash ~+/startup-heavy.sh", 2, 0, 1),
            ("d137 H6：BASH_ENV=~+/脚本 起 bash，照 $PWD 解、读正文照拒", None, "BASH_ENV=~+/startup-heavy.sh bash -c true", 2, 0, 1),
            ("d137 H6：bash ~-/脚本 的 OLDPWD 算不出，认不出起哪一份，照拒", writer, "bash ~-/startup-heavy.sh", 2),
            # H8：rbash / busybox applet / dash / ksh（LIB_SHELL_WORDS_BREAK=renamed-shells-unseen；dash、ksh 本来就在 SHELL_NAMES）
            ("d137 H8：rbash -c 代码 照 shell 判、照拒", None, "rbash -c 'herd7 x.litmus'", 2),
            ("d137 H8：rbash 起脚本 读正文照拒", writer, "rbash startup-heavy.sh", 2, 0, 1),
            ("d137 H8：busybox sh -c 代码 照拒", writer, "busybox sh -c 'herd7 x.litmus'", 2),
            ("d137 H8：busybox ash -c 代码 照拒", writer, "busybox ash -c 'herd7 x.litmus'", 2),
            ("d137 H8：dash -c 代码 照拒", writer, "dash -c 'herd7 x.litmus'", 2),
            ("d137 H8：ksh -c 代码 照拒", writer, "ksh -c 'herd7 x.litmus'", 2),
            ("d137 H8：busybox ls 不是 shell applet，放行", writer, "busybox ls", 0),
            ("d137 H8：rbash -c 'echo hi' 无害放行", writer, "rbash -c 'echo hi'", 0),
            # H3：shopt -s expand_aliases 之后别名展开成要拒的命令（LIB_SHELL_WORDS_BREAK=aliases-unexpanded）
            ("d137 H3：别名展开成 herd7 x.litmus 照拒", None, "shopt -s expand_aliases\nalias t='herd7 x.litmus'\nt", 2),
            ("d137 H3：别名 cargo 自展开（换行、先定义后 shopt）照拒", writer, "alias cargo='herd7 x.litmus'\nshopt -s expand_aliases\ncargo", 2),
            ("d137 H3：别名展开成起脚本（同一条里 alias 先提到脚本、认不出写法）照拒", writer, "shopt -s expand_aliases\nalias ll='bash startup-heavy.sh'\nll", 2),
            ("d137 H3：别名展开成 ls -l 无害放行", writer, "shopt -s expand_aliases\nalias ll='ls -l'\nll", 0),
            ("d137 H3：没 shopt -s expand_aliases（非交互 bash 不展开），别名不展开、放行", writer, "alias t='herd7 x.litmus'; t", 0),
        ]
        judge_cases(d137_cases)
        # BASH_ENV 包进起 shell 的转发函数、exec / 复合命令上的输入重定向、拷来 / 链来的 shell 二进制、多源 cat / >> / tee 的源盘上不在。
        # 每格后面注释写弄坏它的开关；startup-heavy.sh / s.sh（herd7 x.litmus）、startup-light.sh（echo hello）在 work 根，/bin/bash 是真的
        d137r3_cases = [
            # A2：BASH_ENV / ENV 包进起 shell 的转发函数（PROJECT_CODE_GUARD_BREAK=bash-env-forwarding-untracked）
            ("d137 A2：BASH_ENV=脚本 包进起 bash 的转发函数、跟进函数体读正文照拒", None, "f(){ bash -c true; }; BASH_ENV=startup-heavy.sh f", 2, 0, 1),
            ("d137 A2：env BASH_ENV=脚本 起转发函数照拒", writer, "g(){ bash -c true; }; env BASH_ENV=startup-heavy.sh g", 2, 0, 1),
            ("d137 A2：function 关键字定义的函数同样跟进、照拒", writer, "function f() { bash -c true; }; BASH_ENV=startup-heavy.sh f", 2, 0, 1),
            ("d137 A2：BASH_ENV 指到轻脚本，放行", writer, "f(){ bash -c true; }; BASH_ENV=startup-light.sh f", 0, 0, 1),
            ("d137 A2：函数体不起 shell（只有 echo）、BASH_ENV 不跟进，放行", writer, "f(){ echo hi; }; BASH_ENV=startup-heavy.sh f", 0, 0, 0),
            ("d137 A2：函数体起 shell 但这一条没设 BASH_ENV，放行", writer, "f(){ bash -c true; }; f", 0, 0, 0),
            # H4：exec / 复合命令上的输入重定向下发给里层读标准输入的 shell（PROJECT_CODE_GUARD_BREAK=compound-stdin-untracked）
            ("d137 H4：exec < 文件; bash 读那份文件判、照拒", writer, "exec < startup-heavy.sh; bash", 2, 0, 1),
            ("d137 H4：exec 0< 文件; bash -s 照拒", writer, "exec 0< startup-heavy.sh; bash -s", 2, 0, 1),
            ("d137 H4：{ bash; } < 文件 照拒", writer, "{ bash; } < startup-heavy.sh", 2, 0, 1),
            ("d137 H4：( bash ) < 文件 照拒", writer, "( bash ) < startup-heavy.sh", 2, 0, 1),
            ("d137 H4：while … done < 文件 照拒", writer, "while true; do bash; break; done < startup-heavy.sh", 2, 0, 1),
            ("d137 H4：exec < 轻文件; bash 放行", writer, "exec < startup-light.sh; bash", 0, 0, 1),
            ("d137 H4：{ bash; } < 轻脚本 放行", writer, "{ bash; } < startup-light.sh", 0, 0, 1),
            ("d137 H4：while read … done < 文件 里没有裸 shell，放行", writer, 'while read l; do echo "$l"; done < startup-heavy.sh', 0, 0, 0),
            ("d137 H4：exec < 变量; bash 来源算不出，照拒", writer, "exec < $SOMEVAR137; bash", 2),
            # H8：同一条命令里 cp / ln / install 拷来、链来的 shell 二进制（PROJECT_CODE_GUARD_BREAK=copied-shell-binary-untracked）
            ("d137 H8：cp /bin/bash ./x; ./x -c 代码 照 bash 判、照拒", None, "cp /bin/bash ./mysh137a && ./mysh137a -c 'herd7 x.litmus'", 2, 0, 0),
            ("d137 H8：ln -sf /bin/bash ./x; ./x -c 代码 照拒", writer, "ln -sf /bin/bash ./mysh137b; ./mysh137b -c 'herd7 x.litmus'", 2, 0, 0),
            ("d137 H8：install /bin/bash ./x; ./x 脚本 读正文照拒", writer, "install /bin/bash ./mysh137c; ./mysh137c startup-heavy.sh", 2, 0, 1),
            ("d137 H8：cp /bin/bash ./x; ./x 轻脚本 放行", writer, "cp /bin/bash ./mysh137d; ./mysh137d startup-light.sh", 0, 0, 1),
            ("d137 H8：cp /bin/ls ./x; ./x 不是 shell、放行", writer, "cp /bin/ls ./myls137; ./myls137", 0, 0, 0),
            ("d137 H8：cp /bin/bash ./x; ./x -c 'echo hi' 无害放行", writer, "cp /bin/bash ./mysh137e; ./mysh137e -c 'echo hi'", 0, 0, 0),
            # H7 连 S3：多源 cat / >> / tee 的源盘上不在，照认不出拒（PROJECT_CODE_GUARD_BREAK=cat-missing-source-allowed）
            ("d137 H7：多源 cat 有盘上不在的源（在的源起全量）、写出再起，照拒", writer, "cat s.sh nocat137a.sh > dd137a.sh && bash dd137a.sh", 2, 0, 0),
            ("d137 H7：多源 cat 有盘上不在的源（在的源无害），照拒", writer, "cat startup-light.sh nocat137b.sh > dd137b.sh && bash dd137b.sh", 2, 0, 0),
            ("d137 H7：源是 echo 写出的（认不出写法、盘上不在），照拒", writer, "echo x > m137a.txt; cat startup-light.sh m137a.txt > dd137c.sh; bash dd137c.sh", 2, 0, 0),
            ("d137 H7：cat 源 >> 追加、源盘上不在，照拒", writer, "echo x > m137b.txt; cat m137b.txt >> dd137d.sh; bash dd137d.sh", 2, 0, 0),
            ("d137 H7：cat 源 | tee 拷出、源盘上不在，照拒", writer, "echo x > m137c.txt; cat m137c.txt | tee dd137e.sh > /dev/null; bash dd137e.sh", 2, 0, 0),
            ("d137 H7：多源 cat 两个源都在、都无害，放行、读写出的内容", writer, "cat startup-light.sh startup-light.sh > dd137f.sh && bash dd137f.sh", 0, 0, 1),
            ("d137 H7：单源 cat 源在、起全量，照拒（控制组）", writer, "cat s.sh > dd137g.sh && bash dd137g.sh", 2, 0, 1),
        ]
        judge_cases(d137r3_cases)
        # 登录 shell 的 profile、系统级启动文件、别的程序里起的命令；脚本正文里起 claude 归 bash-command-detector.sh。
        # 系统级启动文件的目录（lib_shell_words.py 的 SYSTEM_STARTUP_DIRECTORY）平常换成空的 system-startup-empty（不读本机的 /etc），系统级那一组换成 system-startup-heavy
        for relative, text in (
                               ('home-login-heavy/.bash_profile', 'herd7 x.litmus\n'),
                               ('home-login-light/.bash_profile', 'echo hello\n'),
                               ('home-profile-heavy/.profile', 'herd7 x.litmus\n'),
                               ('system-startup-heavy/profile', 'herd7 x.litmus\n'),
                               ('system-startup-heavy/bash.bashrc', 'herd7 x.litmus\n'),
                               ('system-startup-empty/.keep', ''),
                               ('prog-heavy137.awk', 'BEGIN { system("herd7 x.litmus") }\n'),
                               ('prog-light137.awk', '{ print $1 }\n'),
                               ('mk-heavy137/Makefile', 'all:\n\therd7 x.litmus\n'),
                               ('mk-light137/Makefile', 'all:\n\techo hello\n'),
                               ('rcfile-light137.sh', 'echo hello\n'),
                               ('body-embedded137.sh', '#!/usr/bin/env bash\nperl -e \'system("herd7 x.litmus")\'\n'),
):
            os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        # 登录 shell 读 /etc/profile 与 HOME 下的 profile（LIB_SHELL_WORDS_BREAK=login-profiles-unread 下照拒的格红，逐格见文件头开关清单）
        judge_cases([
            ('d137 已知没认：登录 bash -l 读 HOME 下的 .bash_profile，里面起全量，照拒', writer, f'HOME={work}/home-login-heavy bash -l -c true', 2, 0, 1),
            ('d137 已知没认：登录 bash --login 同样读 .bash_profile，照拒', writer, f'HOME={work}/home-login-heavy bash --login -c true', 2, 0, 1),
            ('d137 已知没认：登录 sh -l 读 HOME 下的 .profile，照拒', writer, f'HOME={work}/home-profile-heavy sh -l -c true', 2, 0, 1),
            ('d137 已知没认：登录 exec -l bash 照登录 shell 读 .bash_profile，照拒', writer, f'HOME={work}/home-login-heavy exec -l bash -c true', 2, 0, 1),
            ('d137 已知没认：登录 exec -a -bash bash 照登录 shell 读 .bash_profile，照拒', writer, f'HOME={work}/home-login-heavy exec -a -bash bash -c true', 2, 0, 1),
            ('d137 已知没认：登录 bash -l 读的 profile 只有轻活，放行', writer, f'HOME={work}/home-login-light bash -l -c true', 0, 0, 1),
            ('d137 已知没认：登录 bash --noprofile -l 不读 profile，放行', writer, f'HOME={work}/home-login-heavy bash --noprofile -l -c true', 0, 0, 0),
            ('d137 已知没认：登录 exec -a 名字不以 - 起头，不是登录 shell，放行', writer, f'HOME={work}/home-login-heavy exec -a mybash bash -c true', 0, 0, 0),
        ])
        # 系统级启动文件：/etc/profile、/etc/bash.bashrc（样本目录；LIB_SHELL_WORDS_BREAK=system-startup-files-unread 下照拒的格红）
        shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-heavy")
        try:
            judge_cases([
                ('d137 已知没认：系统级 bash -l 读 /etc/profile（样本目录），里面起全量，照拒', writer, f'HOME={work}/home-login-light bash -l -c true', 2, 0, 2),
                ('d137 已知没认：系统级 bash -i 读 /etc/bash.bashrc（样本目录），照拒', writer, f'HOME={work}/home-light bash -i -c true', 2, 0, 2),
                ('d137 已知没认：系统级 bash --rcfile 轻文件 -i 照读 /etc/bash.bashrc，照拒', writer, f'bash --rcfile {work}/rcfile-light137.sh -i -c true', 2, 0, 2),
                ('d137 已知没认：系统级 bash --norc -i 不读 /etc/bash.bashrc，放行', writer, 'bash --norc -i -c true', 0, 0, 0),
                ('d137 已知没认：系统级 不登录不交互的 bash -c 不读 /etc 下的，放行', writer, 'bash -c true', 0, 0, 0),
            ])
        finally:
            shell_words.SYSTEM_STARTUP_DIRECTORY = os.path.join(work, "system-startup-empty")
        # 别的程序里起的命令（LIB_SHELL_WORDS_BREAK=embedded-launches-unseen 下照拒的格红；PROJECT_CODE_GUARD_BREAK=program-files-unread 下读程序文件的格红，
        # embedded-unjudged-allowed 下认不出的格红；逐格见文件头开关清单）
        judge_cases([
            ('d137 H9：python -c 里 os.system 起全量，照拒', writer, 'python3 -c \'import os; os.system("herd7 x.litmus")\'', 2, 0, 0),
            ('d137 H9：python 读 heredoc 里 subprocess.run 起全量，照拒', writer, "python3 - <<'PY'\nimport subprocess\nsubprocess.run(['herd7', 'x.litmus'])\nPY", 2, 0, 0),
            ('d137 H9：python -c 不起命令，放行', writer, "python3 -c 'print(1)'", 0, 0, 0),
            ('d137 H9：python -c 里 os.system 的命令是变量，认不出照拒', writer, "python3 -c 'import os, sys; os.system(sys.argv[1])' x", 2, 0, 0),
            ('d137 H9：perl -e 里 system 起全量，照拒', writer, 'perl -e \'system("herd7 x.litmus")\'', 2, 0, 0),
            ("d137 H9：perl -ne 'print' 不起命令，放行", writer, "perl -ne 'print' f", 0, 0, 0),
            ('perl 单独写的 -i 不吃下一个词：perl -i -e 里 system 起全量，照拒', writer, 'perl -i -e \'system("herd7 x.litmus")\'', 2, 0, 0),
            ('perl 单独写的 -l 不吃下一个词：perl -l -e 里 system 起全量，照拒', writer, 'perl -l -e \'system("herd7 x.litmus")\'', 2, 0, 0),
            ('perl 单独写的 -0 不吃下一个词：perl -0 -e 里 system 起全量，照拒', writer, 'perl -0 -e \'system("herd7 x.litmus")\'', 2, 0, 0),
            ('d137 H9：ruby -e 里 system 起全量，照拒', writer, 'ruby -e \'system("herd7 x.litmus")\'', 2, 0, 0),
            ('d137 H9：node -e 里 child_process 起全量，照拒', writer, 'node -e \'require("child_process").execSync("herd7 x.litmus")\'', 2, 0, 0),
            ('d137 H9：tclsh 读 here-string 里 exec 起全量，照拒', writer, "tclsh <<< 'exec herd7 x.litmus'", 2, 0, 0),
            ('d137 H9：gawk 的 system() 起全量，照拒', writer, 'gawk \'BEGIN{system("herd7 x.litmus")}\'', 2, 0, 0),
            ('d137 H9：awk 的 print 管道到全量，照拒', writer, 'awk \'BEGIN { print "x" | "herd7 x.litmus" }\'', 2, 0, 0),
            ("d137 H9：awk '{print $1}' 不起命令，放行", writer, "awk '{print $1}' f", 0, 0, 0),
            ('d137 H9：awk -f 程序文件里 system() 起全量，读正文照拒', writer, f'awk -f {work}/prog-heavy137.awk f', 2, 0, 1),
            ('d137 H9：awk -f 程序文件不起命令，放行', writer, f'awk -f {work}/prog-light137.awk f', 0, 0, 1),
            ('d137 H9：sed 的 e 命令起全量，照拒', writer, "sed -n '1e herd7 x.litmus' f", 2, 0, 0),
            ('d137 H9：sed 普通替换不起命令，放行', writer, "sed -n 's/a/b/p' f", 0, 0, 0),
            ('d137 H9：sed 的 s///e 执行模式空间，认不出照拒', writer, "sed -e 's/x/y/e' f", 2, 0, 0),
            ('d137 H9：git -c 别名 ! 起全量，照拒', writer, "git -c alias.x='!herd7 x.litmus' x", 2, 0, 0),
            ('d137 H9：git -c core.pager 起全量，照拒', writer, "git -c core.pager='herd7 x.litmus' log", 2, 0, 0),
            ('d137 H9：GIT_EXTERNAL_DIFF 起全量，照拒', writer, "GIT_EXTERNAL_DIFF='herd7 x.litmus' git diff", 2, 0, 0),
            ('d137 H9：git -c 别名指到 git 子命令，放行', writer, 'git -c alias.st=status st', 0, 0, 0),
            ('d137 H9：make 读 Makefile 配方起全量，照拒', writer, f'make -C {work}/mk-heavy137', 2, 0, 1),
            ('d137 H9：make 读 Makefile 配方只有轻活，放行', writer, f'make -C {work}/mk-light137', 0, 0, 1),
            ('d137 H9：make -f /dev/stdin 读 here-string 的配方起全量，照拒', writer, "make -f /dev/stdin <<< $'all:\\n\\therd7 x.litmus'", 2, 0, 0),
            ('d137 H9：tar --to-command 起全量，照拒', writer, "tar --to-command='herd7 x.litmus' -xf a.tar", 2, 0, 0),
            ('d137 H9：ssh -o ProxyCommand 起全量，照拒', writer, "ssh -o ProxyCommand='herd7 x.litmus' host true", 2, 0, 0),
            ('d137 H9：脚本正文里 perl -e 起全量，读正文照拒', writer, f'bash {work}/body-embedded137.sh', 2, 0, 1),
        ])
        # 认得出起的是什么、不起重活与 claude 的照放，同形起重活的照拒。逐格见文件头开关清单：LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed、
        # sed-text-first-line-only、python-argument-variables-unresolved、awk-command-variables-unresolved、cd-known-targets-unexpanded、
        # cd-failure-guards-unseen、non-interactive-return-unseen 下「放行」的格（cd-known-targets-unexpanded、cd-failure-guards-unseen 另有照拒格的「读进去几份脚本」）红。{work} 换成样本目录
        for relative, text in (
                               ('r2-light.sh', 'echo hello\n'),
                               ('r2-heavy.sh', 'herd7 x.litmus\n'),
                               ('gitrepo137/.git/HEAD', 'ref: refs/heads/master\n'),
                               ('gitrepo137/.git/objects/.keep', ''),
                               ('gitrepo137/.git/refs/heads/.keep', ''),
                               ('gitrepo137/inner/.keep', ''),
                               ('gitrepo137/r2-light.sh', 'echo hello\n'),
                               ('gitrepo137/r2-heavy.sh', 'herd7 x.litmus\n'),
                               ('home-guarded137/.profile', '. {work}/home-guarded137/.bashrc\n'),
                               ('home-guarded137/.bashrc', '# 非交互就 return\ncase $- in\n    *i*) ;;\n      *) return;;\nesac\nherd7 x.litmus\n'),
                               ('home-unguarded137/.profile', '. {work}/home-unguarded137/.bashrc\n'),
                               ('home-unguarded137/.bashrc', 'if false; then\n  case $- in *i*) ;; *) return;; esac\nfi\nherd7 x.litmus\n'),
):
            os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text.replace("{work}", work))
        judge_cases([(label, writer, command.replace("{work}", work), want, want_notices, want_read) for label, command, want, want_notices, want_read in [
            ('d137-r2 A：sed 地址里的算术展开照 sed 判，放行', 'n=40; sed -n "$((n-3)),$((n))p" f', 0, 0, 0),
            ('d137-r2 A：sed 地址里的 $[ ] 算术照 sed 判，放行', 'sed -n "$[n-3],$[n+3]p" f', 0, 0, 0),
            ('d137-r2 A：sed 行号地址是命令替换，照 sed 判，放行', 'sed -n "$(grep -n x f | cut -d: -f1),+45p" f', 0, 0, 0),
            ('d137-r2 A：sed 算术地址后接 e 起全量，照拒', 'sed -n "$((n))e herd7 x.litmus" f', 2, 0, 0),
            ('d137-r2 A：sed 命令替换落在命令位上，照拒', 'sed -n "1$(echo p)" f', 2, 0, 0),
            ('d137-r2 A：sed 地址里的命令替换起全量，照拒', 'sed -n "$(herd7 x.litmus),+45p" f', 2, 0, 0),
            ('d137-r2 B：sed 的 c\\ 多行文本，放行', "sed '/x/c\\\nfoo' f", 0, 0, 0),
            ('d137-r2 B：sed 的 a\\ 续行文本，放行', "sed '/x/a\\\nfoo\\\nbar' f", 0, 0, 0),
            ('d137-r2 B：sed 文本之后另起一行 e 起全量，照拒', "sed '/x/a\\\nfoo\n1e herd7 x.litmus' f", 2, 0, 0),
            ('d137-r2 C：python argv 头一项字面、参数是变量，照命令词判，放行', 'python3 -c \'import subprocess,sys; subprocess.run(["ls", sys.argv[1]])\' x', 0, 0, 0),
            ('d137-r2 C：python argv 起 cargo test 而参数是变量，照拒', 'python3 -c \'import subprocess,sys; subprocess.run(["cargo", "test", sys.argv[1]])\' x', 2, 0, 0),
            ('d137-r2 C：python argv 头一项是变量，照拒', 'python3 -c \'import subprocess,sys; subprocess.run([sys.argv[1], "test"])\' x', 2, 0, 0),
            ('d137-r2 D：python 字面列表先赋变量再起，放行', "python3 - <<'PY'\nimport subprocess\nargs = ['git', 'status']\nsubprocess.run(args)\nPY", 0, 0, 0),
            ('d137-r2 D：python 字面串 .split() 作命令，放行', 'python3 -c \'import subprocess; subprocess.run("ls -l".split())\'', 0, 0, 0),
            ('d137-r2 D：python 先赋变量的字面列表起全量，照拒', "python3 - <<'PY'\nimport subprocess\nargs = ['herd7', 'x.litmus']\nsubprocess.run(args)\nPY", 2, 0, 0),
            ('d137-r2 D：python 列表赋过之后被改，认不出照拒', "python3 - <<'PY'\nimport subprocess\nargs = ['ls']\nargs.append('x')\nsubprocess.run(args)\nPY", 2, 0, 0),
            ('d137-r2 E：awk 里 cmd 赋字面再 | getline，放行', 'awk \'BEGIN { cmd = "date"; cmd | getline d; close(cmd) }\'', 0, 0, 0),
            ('d137-r2 E：awk 命令行 cmd=sort 再 print |，放行', "awk '{ print $1 | cmd }' cmd=sort f", 0, 0, 0),
            ('d137-r2 E：awk 里 cmd 赋全量再 | getline，照拒', 'awk \'BEGIN { cmd = "herd7 x.litmus"; cmd | getline d }\'', 2, 0, 0),
            ('d137-r2 E：awk 里 cmd 是拼接的，认不出照拒', 'awk \'BEGIN { cmd = "ls " $1; cmd | getline d }\'', 2, 0, 0),
            ('d137-r2 F：cd "$PWD" 之后 bash 轻脚本，放行', 'cd "$PWD" && bash r2-light.sh', 0, 0, 1),
            ('d137-r2 F：cd "$PWD" 之后 bash 重脚本，照拒', 'cd "$PWD" && bash r2-heavy.sh', 2, 0, 1),
            ('d137-r2 F：cd 到 git 顶层之后 bash 轻脚本，放行', 'cd gitrepo137/inner && cd "$(git rev-parse --show-toplevel)" && bash r2-light.sh', 0, 0, 1),
            ('d137-r2 F：cd 到 git 顶层之后 bash 重脚本，照拒', 'cd gitrepo137/inner && cd "$(git rev-parse --show-toplevel)" && bash r2-heavy.sh', 2, 0, 1),
            ('d137-r2 F：cd "$OLDPWD" 回来之后 bash 轻脚本，放行', 'cd gitrepo137 && cd "$OLDPWD" && bash r2-light.sh', 0, 0, 1),
            ('d137-r2 F：cd "$OLDPWD" 回来之后 bash 重脚本，照拒', 'cd gitrepo137 && cd "$OLDPWD" && bash r2-heavy.sh', 2, 0, 1),
            ('d137-r2 H5：cd 盘上不在的目录 && 相对脚本，cd 失败就不跑，放行', 'cd nonexist137 && bash r2-heavy.sh', 0, 1, 0),
            ('d137-r2 H5：cd 盘上不在的目录 || exit 之后的相对脚本，放行', 'cd nonexist137 || exit 1; bash r2-heavy.sh', 0, 1, 0),
            ('d137-r2 H5：cd 盘上不在的目录 ; 之后的相对脚本照跑，照拒', 'cd nonexist137; bash r2-heavy.sh', 2, 0, 0),
            ('d137-r2 H5：(cd 不在的目录 || exit) 之后的相对脚本照跑，照拒', '(cd nonexist137 || exit); bash r2-heavy.sh', 2, 0, 0),
            ('d137-r2 H5：cd 不在的目录 && ls || 相对脚本照跑，照拒', 'cd nonexist137 && ls || bash r2-heavy.sh', 2, 0, 0),
            ('d137-r2 H5：mkdir 再 cd 进去跑同一条里写出的重脚本，照拒', "mkdir -p made137 && cat > made137/w.sh <<'EOF'\nherd7 x.litmus\nEOF\ncd made137 && bash w.sh", 2, 0, 1),
            ('d137-r2 G：登录 bash -l 读 .profile 再 source 带非交互守卫的 .bashrc，守卫之后不读，放行', 'HOME={work}/home-guarded137 bash -l -c true', 0, 0, 2),
            ('d137-r2 G：交互的 bash -i 读带守卫的 .bashrc 不截，守卫之后的全量照拒', 'HOME={work}/home-guarded137 bash -i -c true', 2, 0, 1),
            ('d137-r2 G：守卫不在顶层（套在 if 里）不截，照拒', 'HOME={work}/home-unguarded137 bash -l -c true', 2, 0, 2),
        ]])
        # 认得出起的是什么的照放，同形起重活或认不出的照拒。
        # E2 格 PROJECT_CODE_GUARD_BREAK=launcher-recorded-as-writer、E3 格 heredoc-targets-unexpanded、E5 格 program-file-variables-unexpanded、
        # E4 格 LIB_SHELL_WORDS_BREAK=aliases-same-line-expanded、口径格 LIB_PROJECT_CODE_RUNS_BREAK=claude-executable-path-unrecognized 下「放行」的格红（逐格见文件头开关清单）
        judge_cases([(label, agent_type, command.replace("{work}", work), want, want_notices, want_read)
                     for label, agent_type, command, want, want_notices, want_read in [
            ("d137 闸误拒 E2：systemd-run --user --wait 包着 bash 轻脚本，放行", writer, "systemd-run --user --wait bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：chrt -i 0 包着 bash 轻脚本，放行", writer, "chrt -i 0 bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：ssh localhost 包着 bash 轻脚本，放行", writer, "ssh localhost bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：/usr/bin/time 包着 bash 轻脚本，放行", writer, "/usr/bin/time bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：strace -o /dev/null 包着 bash 轻脚本，放行", writer, "strace -o /dev/null bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：prlimit --as 包着 bash 轻脚本，放行", writer, "prlimit --as=4000000000 bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：numactl -N0 包着 bash 轻脚本，放行", writer, "numactl -N0 bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：unshare --user 包着 bash 轻脚本，放行", writer, "unshare --user bash startup-light.sh", 0, 0, 1),
            ("d137 闸误拒 E2：systemd-run --user --wait 包着 bash 重脚本，读正文照拒", writer, "systemd-run --user --wait bash startup-heavy.sh", 2, 0, 1),
            ("d137 闸误拒 E3：t 赋过字面路径，cat > $t 读 heredoc 写出再 bash $t，放行", writer, "t={work}/e3-written137.sh; cat > $t <<'EOF'\necho hi\nEOF\nbash $t", 0, 0, 1),
            ("d137 闸误拒 E3：$t 没赋过，cat > $t 写出再 bash $t，照拒", writer, "cat > $t <<'EOF'\necho hi\nEOF\nbash $t", 2, 0, 0),
            ("d137 闸误拒 E4：同一行先 alias 再用，这一行不展开别名、放行", writer, "shopt -s expand_aliases; alias t='herd7 x.litmus'; t", 0, 0, 0),
            ("d137 闸误拒 E5：W 赋过字面目录，awk -f $W/程序文件，读正文放行", writer, "W={work}; awk -f $W/prog-light137.awk f", 0, 0, 1),
            ("d137 闸误拒 E5：W 没赋过，awk -f $W/程序文件认不出，照拒", writer, "awk -f $W/prog-light137.awk f", 2, 0, 0),
            ("d137 闸口径：主 agent 经 \"$CLAUDE_CODE_EXECPATH\" 起 claude，交检出钩子判，放行", None, '"$CLAUDE_CODE_EXECPATH" -p hi', 0, 0, 0),
        ]])
        # 认不出而放行的五种写法（账本一百六十六①）：照拒的格各配贴着它的放行格。弄坏它的开关（都是 LIB_SHELL_WORDS_BREAK=<项>，逐格见文件头开关清单）：
        # 别名 alias-start-options-unseen、alias-nested-texts-unseen；复合命令转交标准输入 compound-stdin-closers-unseen（PROJECT_CODE_GUARD_BREAK=compound-stdin-untracked 同样红）；
        # make -C / git -c 的变量 embedded-option-variables-unexpanded；变量作用域 variable-scopes-unseen。startup-heavy.sh（herd7 x.litmus）、startup-light.sh（echo hello）、
        # mk-heavy137 / mk-light137（Makefile）在 work 根
        judge_cases([(label, writer, command.replace("{work}", work), want, want_notices, want_read) for label, command, want, want_notices, want_read in [
            ("一百六十六①：bash -O expand_aliases -c 里换行之后用别名展开成全量，照拒", "bash -O expand_aliases -c 'alias t=\"herd7 x.litmus\"\nt'", 2, 0, 0),
            ("一百六十六①：sh -c 里换行之后用别名展开成全量，照拒", "sh -c 'alias t=\"herd7 x.litmus\"\nt'", 2, 0, 0),
            ("一百六十六①：bash -O expand_aliases -c 里的别名展开成 ls -l，放行", "bash -O expand_aliases -c 'alias ll=\"ls -l\"\nll'", 0, 0, 0),
            ("一百六十六①：bash -c 不开别名展开，别名不展开、放行", "bash -c 'alias t=\"herd7 x.litmus\"\nt'", 0, 0, 0),
            ("一百六十六①：shopt 开着时 eval 里用前面定义的别名，照拒", "shopt -s expand_aliases\nalias t='herd7 x.litmus'\neval t", 2, 0, 0),
            ("一百六十六①：同一行 shopt、alias 之后 eval 里用别名（eval 跑的时候才切），照拒", "shopt -s expand_aliases; alias t='herd7 x.litmus'; eval t", 2, 0, 0),
            ("一百六十六①：eval 里定义的别名，下一行照展开，照拒", "shopt -s expand_aliases\neval \"alias t='herd7 x.litmus'\"\nt", 2, 0, 0),
            ("一百六十六①：eval 里用的别名是 ls -l，放行", "shopt -s expand_aliases\nalias ll='ls -l'\neval ll", 0, 0, 0),
            ("一百六十六①：没开别名展开时 eval 里用别名不展开，放行", "alias t='herd7 x.litmus'\neval t", 0, 0, 0),
            ("一百六十六①：if … fi < 文件 里的裸 bash 读那份文件判，照拒", "if true; then bash; fi < startup-heavy.sh", 2, 0, 1),
            ("一百六十六①：case … esac < 文件 里的裸 bash 读那份文件判，照拒", "case x in x) bash;; esac < startup-heavy.sh", 2, 0, 1),
            ("一百六十六①：函数体里的裸 bash、调用时 < 文件，读那份文件判，照拒", "f() { bash; }; f < startup-heavy.sh", 2, 0, 1),
            ("一百六十六①：{ bash; } <&3 从别的描述符转交，认不出照拒", "exec 3< startup-light.sh; { bash; } <&3", 2, 0, 0),
            ("一百六十六①：printf 代码 | ( bash ) 管道接进子 shell 里的裸 bash，认不出照拒", "printf 'echo hi\\n' | ( bash )", 2, 0, 0),
            ("一百六十六①：{ bash; } <<< 代码 here-string 转交，认不出照拒", "{ bash; } <<< 'echo hi'", 2, 0, 0),
            ("一百六十六①：if … fi < 轻脚本 里的裸 bash，读正文放行", "if true; then bash; fi < startup-light.sh", 0, 0, 1),
            ("一百六十六①：if … fi < 文件 里没有裸 shell，放行", "if true; then cat; fi < startup-heavy.sh", 0, 0, 0),
            ("一百六十六①：make -C 的目录写成没赋过的变量，认不出照拒", 'make -C "$MAKEDIR166"', 2, 0, 0),
            ("一百六十六①：make -C 的目录写成前面赋过字面值的变量，代入之后读 Makefile 照拒", 'M={work}/mk-heavy137; make -C "$M"', 2, 0, 1),
            ("一百六十六①：make -C 的目录写成前面赋过字面值的变量、Makefile 只有轻活，放行", 'M={work}/mk-light137; make -C "$M"', 0, 0, 1),
            ("一百六十六①：git -c 的配置整个写成没赋过的变量，认不出照拒", 'git -c "$GITCONFIG166" st', 2, 0, 0),
            ("一百六十六①：git -c 别名的值写成没赋过的变量，认不出照拒", 'git -c "alias.x=$GITALIAS166" x', 2, 0, 0),
            ("一百六十六①：git -c 的配置写成前面赋过字面值的变量（! 别名起全量），代入照拒", "C='alias.x=!herd7 x.litmus'; git -c \"$C\" x", 2, 0, 0),
            ("一百六十六①：git -c 的配置写成前面赋过字面值的变量（user.name），放行", "N='user.name=x'; git -c \"$N\" status", 0, 0, 0),
            ("一百六十六①：git -c 的配置名字面、普通配置项的值写成变量，放行", 'git -c "user.name=$NAME166" status', 0, 0, 0),
            ("一百六十六①：子 shell 里赋的值不带出来，外面的命令词认不出照拒", "C=herd7; (C=ls); $C x.litmus", 2, 0, 0),
            ("一百六十六①：管道段里赋的值不带出来，照拒", "C=herd7; C=ls | true; $C x.litmus", 2, 0, 0),
            ("一百六十六①：放后台的赋值不带出来，照拒", "C=herd7; C=ls & wait; $C x.litmus", 2, 0, 0),
            ("一百六十六①：|| 之后的条件赋值不一定生效，照拒", "C=herd7; true || C=ls; $C x.litmus", 2, 0, 0),
            ("一百六十六①：if 里赋的值不一定生效，照拒", "C=herd7; if false; then C=ls; fi; $C x.litmus", 2, 0, 0),
            ("一百六十六①：子 shell 里赋值、在子 shell 里用，照里面的值判，放行", "C=herd7; (C=ls; $C x.litmus)", 0, 0, 0),
            ("一百六十六①：一串 && 里赋值、同一串里接着用，放行", "true && C=ls && $C x.litmus", 0, 0, 0),
            ("一百六十六①：子 shell 里只读不改，外面的值照用，放行", "C=ls; (true); $C x.litmus", 0, 0, 0),
        ]])
        # 读进来的脚本正文里命令词或脚本路径由变量拼出、解不出，而定义或起它的参数里写着自证、门禁阶段、入口脚本或 cargo 的，照认不出办（例：
        # 草稿里一份脚本 `R=$1; cmds=("bash $R/.claude/hooks/… --selftest" …); … ${cmds[$i]}` 起改过的钩子自证；
        # LIB_PROJECT_CODE_RUNS_BREAK=script-body-launches-unjudged 下「照拒」的三格红，应当是 2，实际 0；cargo 参数解不出那一种由 lib_project_code_runs.py 的格钉住）。直接写 `bash "$R/…/x.sh" --selftest` 按登记表照拒，不另立格；
        # 本仓里的脚本不判这一条（照旧放行那一格）
        for relative, text in (("body-array-selftest.sh", '#!/usr/bin/env bash\nR=$1\ncmds=("bash $R/.claude/hooks/project-code-guard.sh --selftest" "python3 $R/.claude/hooks/lib_project_code_runs.py --selftest")\n'
                                                          'for i in "${!cmds[@]}"; do env "$@" timeout 900 nice -n 19 ${cmds[$i]} > "out-$i" 2>&1; done\n'),
                               ("body-forward.sh", '#!/usr/bin/env bash\n"$@"\n'),
):
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        # 样本工作区平常照「本仓」认（FAMILY_OWN_REPOSITORY），这几格要它当草稿目录：判的时候摘掉，判完放回
        project_code_runs.lib_project_code_runs_launchers.FAMILY_OWN_REPOSITORY = None
        for label, agent_type, command, want in (
                ("一百三十七：脚本正文里经数组变量起拼出路径的钩子自证（件 1 那三趟的形态），照拒", writer, f"bash body-array-selftest.sh {work}/tree", 2),
                ("一百三十七：同一份脚本主 agent 照拒", None, f"bash body-array-selftest.sh {work}/tree", 2),
                ):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            results.append((label, want, scan_decision(hook_input, work).code))
        project_code_runs.lib_project_code_runs_launchers.FAMILY_OWN_REPOSITORY = work
        # 本仓里的脚本不判这一条：还是上面「照拒」那一格的 body-array-selftest.sh 与同一条命令，这时样本工作区照「本仓」认，经变量起的自证放行，两格只差这一点。
        # 不拿仓里真实的脚本当样本：它们的正文随改动会出现字面的要拒的命令（multi-host-run.sh 自证段里有 cargo test），正文照判就拒，这一格就钉不住「经变量起的不判」了
        own_repository_scene = scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": f"bash body-array-selftest.sh {work}/tree"}}, work)
        results.append(("一百三十七：本仓里的脚本正文经变量起包装与命令，不判这一条，照旧放行", 0, own_repository_scene.code))
        # cat 的源里有不在的（修补报告 debt-tw-shell-words-fix2 第 93 行那一类）：照拒，拒绝说明点名不在的源与改法（PROJECT_CODE_GUARD_BREAK=cat-missing-source-unexplained 下红）
        missing_source = scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": 'cat s.sh "5"> d2.sh && bash d2.sh'}}, work)
        results.append(("一百三十七：cat 源里有不在的文件、&& 接起，照拒并点名不在的源", (2, True),
                        (missing_source.code, "盘上不在" in (missing_source.message or "") and "`5`" in (missing_source.message or ""))))
        # cat 追加到盘上读不了的文件（不是 UTF-8 文本的、目录）：那个目的不认，判照常走完；pieces 是 None 还接着 append 的话，判的那一刻抛 AttributeError、整条命令照常执行
        # （例：子 agent 的 `grep -c '    "X' 文件 && … cat 源 >> 超过读的上限的 mutations.tsv`；抛错那一遍引号只是同一条命令里的别的词）。
        # PROJECT_CODE_GUARD_BREAK=cat-append-unreadable-destination-raises 下抛错，这几格拿到的不是 want
        with open(os.path.join(work, "unreadable-binary.tsv"), "wb") as handle:
            handle.write(b"\xff\xfe\x00")
        with open(os.path.join(work, "append-source.txt"), "w", encoding="utf-8") as handle:
            handle.write("追加的一行\n")
        os.makedirs(os.path.join(work, "append-directory"), exist_ok=True)
        for label, agent_type, command, want in (
                ("一百三十七：cat 追加到不是文本的文件之后接 herd7 x.litmus，照拒", writer, "cat append-source.txt >> unreadable-binary.tsv && herd7 x.litmus", 2),
                ("一百三十七：同一条主 agent 起，照拒", None, "cat append-source.txt >> unreadable-binary.tsv && herd7 x.litmus", 2)):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            try:
                code = scan_decision(hook_input, work).code
            except Exception as error:
                code = f"判的时候抛了 {error!r}"
            results.append((label, want, code))
        # 同一份文件一次判定里只从盘上读一遍；直接执行的 .rs 要读开头判是不是脚本，判完不当脚本读，它里面括着的文件不碰
        def disk_reads(command, relative, agent_type=writer):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            return scan_decision(hook_input, work).files_read.count(os.path.realpath(os.path.join(work, relative)))
        for label, command, relative, agent_type, want in [
                ("同一条命令里起 s.sh 两遍：s.sh 从盘上读一遍", "bash s.sh; ./s.sh", "s.sh", writer, 1),
                ("起的脚本里三种起法各起一遍 s.sh：s.sh 从盘上读一遍", "bash twice.sh", "s.sh", writer, 1),
                ]:
            results.append((f"{label}（读盘次数）", want, disk_reads(command, relative, agent_type)))
        # 复现输入那一条要在 SLOW_REPRODUCTION_SECONDS 秒内判完
        started = time.monotonic()
        scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": SLOW_REPRODUCTION_COMMAND}}, work)
        elapsed = time.monotonic() - started
        results.append((f"复现输入那一条在 {SLOW_REPRODUCTION_SECONDS} 秒内判完（这一遍 {elapsed:.3f} 秒）", 1, int(elapsed < SLOW_REPRODUCTION_SECONDS)))
        # 拒绝说明写出脚本路径与行号：直接起的写到 s.sh 第 4 行，嵌套的连外层起它的那一行一起写
        for label, command, location in [("拒绝说明写出脚本路径与行号", "bash s.sh", f"脚本 {work}/s.sh:4 里的 herd7"),
                                         ("嵌套的拒绝说明写出每一层的路径与行号", "bash outer.sh", f"脚本 {work}/outer.sh:3 → {work}/s.sh:4 里的 herd7"),
                                         ("命令替换里的拒绝说明写出它所在那一行", "bash substitution-heavy.sh", f"脚本 {work}/substitution-heavy.sh:3 里的 herd7"),
                                         ("heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号",
                                          "cat > gen.sh <<'EOF'\n#!/usr/bin/env bash\nherd7 x.litmus\nEOF\nbash gen.sh", f"脚本 {work}/gen.sh:2 里的 herd7"),
                                         ("heredoc 写出的脚本里再写出的，逐层写出路径与行号",
                                          "cat > gen-outer.sh <<'EOF'\ncat > gen-inner.sh <<'IN'\nherd7 x.litmus\nIN\nbash gen-inner.sh\nEOF\nbash gen-outer.sh",
                                          f"脚本 {work}/gen-outer.sh:4 → {work}/gen-inner.sh:1 里的 herd7"),
                                         ("复用上一遍的结果，拒绝说明照这一处的路径与行号写",
                                          "bash s.sh; bash outer.sh", f"脚本 {work}/outer.sh:3 → {work}/s.sh:4 里的 herd7"),
                                         ("绕回外层的那一遍不复用，从别处起时照读回来", "bash cyc-a.sh; bash cyc-b.sh",
                                          f"脚本 {work}/cyc-b.sh:1 → {work}/cyc-a.sh:2 里的 herd7")]:
            message = scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": command}}, work).message or ""
            results.append((label, 1, int(location in message and "这一步写在脚本里" in message)))
        # 复用上一遍的检出，照这一处的路径与行号写
        reused_notices = scan_decision({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                 "tool_input": {"command": "bash has-missing.sh; bash outer-missing.sh"}}, work).notices
        results.append(("复用上一遍的检出，照这一处的路径与行号写", 1,
                        int(any(f"脚本 {work}/outer-missing.sh:2 → {work}/has-missing.sh:1 里" in notice for notice in reused_notices))))
        # 切词：source / . 后面跟的像一个路径才算读脚本（python 源码里 `source = …` 那种不算）
        for label, command, want in [("切词：source = … 不算读脚本", "source = open", [("source", None)]),
                                     ("切词：. = … 不算读脚本", ". = x", [(".", None)]),
                                     ("切词：. ./env.sh 算读脚本", ". ./env.sh", [("./env.sh", "source")])]:
            got = [(found.words[0], found.launcher) for found in shell_words.commands_at_command_position(command, work).commands]
            results.append((label, want, got))
        # 切词：进程替换的另一种写法 `>(…)` 同 `<(…)`：里面的命令另判，`)` 之后的词是外层命令的参数
        got = [(found.words[0], found.launcher) for found in shell_words.commands_at_command_position("tee >(bash run.sh) out.log", work).commands]
        results.append(("切词：tee >(bash run.sh) out.log 里 run.sh 另判、out.log 是 tee 的参数", [("run.sh", "shell"), ("tee", None)], got))
        # 切词：heredoc 正文留不留，看它挂在哪条命令上（从标准输入读命令的 bash / sh 才留），不看同一行别处有没有 shell 的名字
        for label, command, kept in [("切词：前一条起了 x.sh、heredoc 喂给 python3 -：正文剥掉", "bash light.sh && python3 - <<'PY'\nBODY\nPY", False),
                                     ("切词：heredoc 喂给 bash：正文留着", "bash <<'EOF'\nBODY\nEOF", True),
                                     ("切词：heredoc 喂给 capped.sh 4 bash -s：正文留着", "capped.sh 4 bash -s -- a <<'EOF'\nBODY\nEOF", True),
                                     ("切词：heredoc 喂给 bash -c：正文剥掉", "bash -c 'cat' <<'EOF'\nBODY\nEOF", False),
                                     ("切词：heredoc 喂给 bash 起的脚本：正文剥掉", "bash light.sh <<'EOF'\nBODY\nEOF", False)]:
            results.append((label, kept, "BODY" in shell_words.strip_data_heredocs(command)))
        # 共用判定模块自己的自检（cargo 命令行与直接执行的测试二进制两种输入）
        results.append(("lib_project_code_runs.py 的自检", 0, project_code_runs.selftest()))
        results.append(("检出记录默认放在工作根下（lib_work_root.py）",
                        os.path.join(work_roots.configured_work_root(), "agent-hook-detections.jsonl") if work_roots else "读得到 lib_work_root.py", DEFAULT_DETECTIONS))
        # 文件头「经 task-server 起」：一切跑项目代码的命令只认经 task-server/task 起（①），子 agent 起客户端只许 status、log（②），绕开服务的写法照拒（③）；
        # 弄坏开关：PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed 下「：照拒」的直接跑那几格红，subagent-task-client-allowed 下子 agent 起客户端「照拒」那几格红，
        # task-server-bypass-allowed 下「绕开：」开头照拒的那几格红；LIB_PROJECT_CODE_RUNS_BREAK=project-scripts-unjudged 下「本仓脚本」那几格红、text-tools-refused 下「名单：」放行那几格红、
        # main-agent-tools-refused 下「名单：主 agent」放行那几格红、task-server-submission-unparsed 下「提交」照拒的那几格红（逐格写在文件头开关表）
        client_source = os.path.join(repo, project_code_runs.TASK_SERVER_CLIENT)
        results.append(("经 task-server：本仓有提交客户端 task-server/task", True, os.path.isfile(client_source)))
        if os.path.isfile(client_source):
            os.makedirs(os.path.join(work, "task-server"), exist_ok=True)
            shutil.copy2(client_source, os.path.join(work, project_code_runs.TASK_SERVER_CLIENT))
        for relative, text in (("own-helper-light.sh", "#!/usr/bin/env bash\n# 草稿里自己的辅助脚本：只读文本\ngrep -c x note.md\n"),
                               ("own-helper-build.sh", "#!/usr/bin/env bash\n# 草稿里自己的辅助脚本：里面直接编\ncargo build --offline -p singlefs-verification\n"),
                               ("own-helper-submits.sh", "#!/usr/bin/env bash\n# 草稿里自己的辅助脚本：里面经 task-server 提交\ntask-server/task submit -- cargo build --offline -p singlefs-verification\n"),
                               ("renamed-project-script.py", "#!/usr/bin/env python3\n# admission: always 样本：改名拷走的本仓脚本\n# run-condition: none 样本\nprint(1)\n")):
            with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        runner = "experiment-runner"
        swap = f"python3 {repo}/scripts/hosts/compile-then-swap.py {work}/draft.rs crates/singlefs-verification/src/bin/e161_sample_device.rs --scratch {work}/swap"
        # 文件头「经 task-server 起」④ 的说明：点名登记表、给整条包装写法（主 agent 与子 agent 各一格）；子 agent 的出路是写进报告「要主 agent 跑的命令」一节
        # （PROJECT_CODE_GUARD_BREAK=temporary-service-remedy-dropped 下两格红，LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-bare 下两格红）
        bare_command = ("task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service "
                        "-- python3 scripts/mutation/crates-mutation-rows.py --selftest")
        bare_submitted = task_server_decision({"tool_name": "Bash", "cwd": repo,
                                               "tool_input": {"command": "task-server/task submit -- python3 scripts/mutation/crates-mutation-rows.py --selftest"}}, repo)
        results.append(("经 task-server：要临时服务的自证裸写被拒时，说明点名登记表、给整条包装写法", True,
                        bare_submitted.code == 2 and all(piece in (bare_submitted.message or "") for piece in (
                            "✗ 要临时 prune-server 的自证写成了裸的 --selftest 被拒", "selftest-temporary-service.tsv", bare_command, "→ 怎么办："))))
        bare_direct = task_server_decision({"tool_name": "Bash", "cwd": repo, "agent_type": writer,
                                            "tool_input": {"command": "python3 scripts/mutation/crates-mutation-rows.py --selftest"}}, repo)
        results.append(("经 task-server：子 agent 直接裸写要临时服务的自证被拒时，说明写进报告「要主 agent 跑的命令」一节、给整条包装写法", True,
                        bare_direct.code == 2 and all(piece in (bare_direct.message or "") for piece in (
                            "✗ 要临时 prune-server 的自证写成了裸的 --selftest 被拒", bare_command, "→ 怎么办：照上面「整条写成」的那一条", "写进报告「要主 agent 跑的命令」一节"))))
        # 文件头「经 task-server 起」⑤：cargo clippy 的 `--` 之后缺 -D warnings 或 check.sh 那张表的某一条：被拒，说明点名缺的、给整段现读的写法；带全的放行；
        # 只问版本、带 --fix 的不判（PROJECT_CODE_GUARD_BREAK=clippy-lint-table-unchecked、clippy-lint-table-remedy-dropped 下第一格红）
        table_lints = code_discipline_lints() or []
        table_tail = clippy_lint_table_tail()
        clippy_short = task_server_decision({"tool_name": "Bash", "cwd": repo,
                                             "tool_input": {"command": "task-server/task submit -- cargo clippy -p singlefs-core --all-targets -- -D warnings"}}, repo)
        results.append(("经 task-server：提交没带全 check.sh lint 表的 clippy 被拒，说明点名缺的 lint、给整段现读的写法", True,
                        bool(table_lints) and clippy_short.code == 2 and all(piece in (clippy_short.message or "") for piece in (
                            "✗ cargo clippy 没带全 check.sh 的 lint 表、放行了表里的 lint、或 check.sh 的表读不成，被拒", table_lints[0], table_tail, "出路：clippy 命令的"))))
        clippy_full = task_server_decision({"tool_name": "Bash", "cwd": repo,
                                            "tool_input": {"command": f"task-server/task submit -- cargo clippy -p singlefs-core --all-targets {table_tail}"}}, repo)
        results.append(("经 task-server：提交带全 check.sh lint 表的 clippy：放行", 0, clippy_full.code))
        clippy_not_judged = [command for command in ("task-server/task submit -- cargo clippy --version", "task-server/task submit -- cargo clippy -p singlefs-core --fix -- -D warnings")
                             if task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {"command": command}}, repo).code != 0]
        results.append(("经 task-server：提交只问 clippy 版本、带 --fix 的 clippy：不判，放行", [], clippy_not_judged))
        # ⑤ 的另外三种：check.sh 的表读不成（PROJECT_CODE_GUARD_BREAK=clippy-lint-unreadable-table-passed 下第一格红）、带全表之后又放行表里的 lint 或带 --cap-lints
        # （clippy-lint-weakening-unchecked 下第二、三、四格红）；这几格与上面那一格一样经 submit 交，clippy-lint-table-unchecked、temporary-service-submit-inner-unread 下前四格红，
        # clippy-lint-table-remedy-dropped 下第一格红（出路那一行点名上游）；先放行、后 deny 的写法 deny 照起作用，放行
        unreadable_table_path = os.path.join(work, "check-lint-table-unreadable.sh")
        with open(unreadable_table_path, "w", encoding="utf-8") as table_handle:
            table_handle.write("CODE_DISCIPLINE_LINTS=(\n  -D clippy::alpha_lint\n)\nCODE_DISCIPLINE_LINTS+=( -D clippy::beta_lint )\n")
        real_table_source = project_code_guard_decisions.CLIPPY_LINT_TABLE_SOURCE
        project_code_guard_decisions.CLIPPY_LINT_TABLE_SOURCE = unreadable_table_path
        try:
            clippy_unreadable = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
                "command": "task-server/task submit -- cargo clippy -p singlefs-core --all-targets -- -D warnings -D clippy::alpha_lint"}}, repo)
        finally:
            project_code_guard_decisions.CLIPPY_LINT_TABLE_SOURCE = real_table_source
        results.append(("经 task-server：check.sh 的 lint 表读不成（另有 += 追加）时 clippy 被拒，说明点名读不成的行、出路交上游改", True,
                        clippy_unreadable.code == 2 and all(piece in (clippy_unreadable.message or "") for piece in (
                            "读不成", "第 4 行", "上游", ".claude/singlefs-ai-sop/scripts/check.sh"))))
        first_lint = table_lints[0] if table_lints else ""
        clippy_weakened = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
            "command": f"task-server/task submit -- cargo clippy -p singlefs-core --all-targets {table_tail} -A {first_lint}"}}, repo)
        results.append(("经 task-server：clippy 带全表、deny 之后又 -A 表里的 lint 被拒，说明点名被放行的那一条", True,
                        bool(table_lints) and clippy_weakened.code == 2 and all(piece in (clippy_weakened.message or "") for piece in ("放行或降级了", table_lints[0]))))
        clippy_group_weakened = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
            "command": f"task-server/task submit -- cargo clippy -p singlefs-core --all-targets {table_tail} -A clippy::pedantic"}}, repo)
        results.append(("经 task-server：clippy 带全表、deny 之后又 -A clippy::pedantic（一整组）被拒", True,
                        bool(table_lints) and clippy_group_weakened.code == 2 and "放行或降级了 clippy::pedantic" in (clippy_group_weakened.message or "")))
        clippy_capped = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
            "command": f"task-server/task submit -- cargo clippy -p singlefs-core --all-targets {table_tail} --cap-lints allow"}}, repo)
        results.append(("经 task-server：clippy 带全表又带 --cap-lints allow 被拒，说明点名 --cap-lints", True,
                        bool(table_lints) and clippy_capped.code == 2 and "--cap-lints" in (clippy_capped.message or "")))
        allowed_first_tail = table_tail.replace("-- ", f"-- -A {first_lint} ", 1)
        clippy_allowed_first = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
            "command": f"task-server/task submit -- cargo clippy -p singlefs-core --all-targets {allowed_first_tail}"}}, repo)
        results.append(("经 task-server：clippy 先 -A、后 -D 同一条 lint：deny 照起作用，放行", (True, 0), (bool(table_lints), clippy_allowed_first.code)))
        # 提交带 --manual、--full-gate、--whole-package-confirmed 时 -- 后面那条照过闸：裸写要临时服务的自证三种都拒、带 --cwd 混写的同样拒
        # （LIB_PROJECT_CODE_RUNS_BREAK=task-server-submit-flags-unparsed、task-server-submission-unparsed 与 temporary-service 那几条开关下红）
        flagged_refusals = []
        for submit_options in ("--manual", "--full-gate", "--whole-package-confirmed /sample/confirmed", f"--cwd {repo} --full-gate"):
            flagged_submit = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
                "command": f"task-server/task submit {submit_options} -- python3 scripts/mutation/crates-mutation-rows.py --selftest"}}, repo)
            flagged_refusals.append((submit_options, flagged_submit.code == 2 and "✗ 要临时 prune-server 的自证写成了裸的 --selftest 被拒" in (flagged_submit.message or "")))
        results.append(("经 task-server：提交带 --manual、--full-gate、--whole-package-confirmed 时 -- 后面那条照过闸，裸写要临时服务的自证照拒", [], [options for options, refused in flagged_refusals if not refused]))
        # 子 agent 交任务被拒，拒绝说明写明出路：写进报告「要主 agent 跑的命令」一节交回、不另起 run_in_background（subagent-task-client-allowed 下这一格红）
        submitted_item = task_server_decision({"tool_name": "Bash", "cwd": repo, "agent_type": writer,
                                               "tool_input": {"command": "task-server/task submit -- bash scripts/crash/crash-point-replay.sh --item fixed-script"}}, repo)
        results.append(("经 task-server：子 agent 交任务的拒绝说明写明写进报告「要主 agent 跑的命令」一节交回", True,
                        submitted_item.code == 2 and all(piece in (submitted_item.message or "") for piece in (
                            f"✗ {writer} 起统一任务服务的客户端被拒", "写进报告「要主 agent 跑的命令」一节交回", "status、log"))))
        # 绕开服务的写法（③）：读签名密钥拒，说明点名绕开写法（task-server-bypass-allowed 下这一格红）
        bypass_read = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {"command": "cat ~/.config/singlefs/task-server-signing.key"}}, repo)
        results.append(("经 task-server：绕开：读签名密钥照拒，说明点名绕开写法", True,
                        bypass_read.code == 2 and "✗ 绕开 task-server 的写法被拒" in (bypass_read.message or "")
                        and "解开符号链接后往上第一层有 .git 的那一层" in (bypass_read.message or "")
                        and "--cwd 解开符号链接后往上哪一层都没有 .git 的，删解开后的那个 --cwd 目录" in (bypass_read.message or "")))
        # 要临时服务的自证套了临时服务的包装（给了 --server-binary）提交：包装剥开重判里面那条时补上标记，放行；自己起服务的自证（own-server）裸写提交也放行
        # （temporary-service-wrapper-not-marked 与 LIB_PROJECT_CODE_RUNS_BREAK 的 temporary-service-marker-ignored、temporary-service-environment-dropped 下第一格红，
        # LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-own-server-as-required 下第二格红）
        wrapped_submit = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {"command": (
            "task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary /sample/prune-server "
            "-- python3 scripts/mutation/crates-mutation-rows.py --selftest")}}, repo)
        results.append(("经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的要临时服务的自证：放行", 0, wrapped_submit.code))
        own_server_submit = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
            "command": "task-server/task submit -- python3 scripts/prune-server/prune-server-load-simulation.py --selftest"}}, repo)
        results.append(("经 task-server：主 agent 提交自己起服务的自证（own-server）裸写：放行", 0, own_server_submit.code))
        # 文件头「经 task-server 起」⑥：门禁样本档（stage-selftest.sh --item）点名的阶段源码里调了临时服务核对、外面没套包装：被拒，说明点名那一道、给整条包装写法；
        # 套了包装（给了 --server-binary）的放行（PROJECT_CODE_GUARD_BREAK=temporary-service-stage-samples-allowed、temporary-service-submit-inner-unread 下第一格红，
        # LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-stage-samples-unjudged、temporary-service-command-bare、temporary-service-command-binary-placeholder 下第一格红；
        # temporary-service-wrapper-not-marked 与 LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-marker-ignored、temporary-service-environment-dropped 下第二格红）
        # 阶段源码原样拷进自证自己造的门禁目录、给它建样本档目录：判法只看阶段源码里有没有临时服务核对，不看仓里的样本档目录在不在
        # （拷的源照字面路径读，算键库把这一道收进键；仓里样本档目录挪走、改名不改这两格的判定）
        stage_sample_gate_directory = os.path.join(work, "guard-repository-stages", ".claude", "gate.d")
        os.makedirs(os.path.join(stage_sample_gate_directory, "fixtures", "crates-mutation-replay.sh", "red"), exist_ok=True)
        shutil.copyfile(os.path.join(repo, ".claude", "gate.d", "crates-mutation-replay.sh"), os.path.join(stage_sample_gate_directory, "crates-mutation-replay.sh"))
        stage_sample_command = f"bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh {stage_sample_gate_directory} --item crates-mutation-replay.sh"
        stage_sample_submitted = task_server_decision({"tool_name": "Bash", "cwd": repo,
                                                       "tool_input": {"command": f"task-server/task submit -- {stage_sample_command}"}}, repo)
        stage_sample_wrapper = "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service"
        results.append(("经 task-server：样本档点名要临时服务的一道、外面没套被拒时，说明点名那一道、给整条包装写法", True,
                        stage_sample_submitted.code == 2 and all(piece in (stage_sample_submitted.message or "") for piece in (
                            "✗ 门禁样本档要临时 prune-server、外面没套包装被拒", "crates-mutation-replay.sh", f"task-server/task submit -- {stage_sample_wrapper} -- {stage_sample_command}", "→ 怎么办："))))
        stage_sample_wrapped = task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {
            "command": f"task-server/task submit -- {stage_sample_wrapper} --server-binary /sample/prune-server -- {stage_sample_command}"}}, repo)
        results.append(("经 task-server：主 agent 提交包着临时服务（给了 --server-binary）的样本档：放行", 0, stage_sample_wrapped.code))
        # 只给主 agent 直接跑的会话工具（lib_project_code_runs.py 的 MAIN_AGENT_TOOLS_RUN_DIRECTLY）：主 agent 直接起查活动会话的 active-sessions.py 放行，子 agent 的同一条照拒
        # （LIB_PROJECT_CODE_RUNS_BREAK=active-sessions-unlisted、main-agent-tools-refused 下第一格红，应当是 0、实际 2；
        # PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed、LIB_PROJECT_CODE_RUNS_BREAK=project-scripts-unjudged 下第二格红，应当是 2、实际 0）
        for label, session_agent, want in (("经 task-server：主 agent 直接起查活动会话的 active-sessions.py：放行", None, 0),
                                           ("经 task-server：子 agent 直接起查活动会话的 active-sessions.py：照拒", writer, 2)):
            session_input = {"tool_name": "Bash", "cwd": repo, "tool_input": {"command": "python3 scripts/agents/active-sessions.py --within-minutes 60"}}
            if session_agent:
                session_input["agent_type"] = session_agent
            results.append((label, want, task_server_decision(session_input, repo).code))
        # python3 -c 在管道里读 JSON、往标准输出打字，代码里的 'task' 字样不是写了后面执行的 task-server/task；
        # 真往 task-server/task 写了再执行的照拒（PROJECT_CODE_GUARD_BREAK=python-filter-code-judged 下前两格红）
        for label, command, want in (
                ("经 task-server：管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了后面执行的 task-server/task：主 agent 放行",
                 "task-server/task wait 7191 >/dev/null 2>&1; task-server/task status 7191 --json | python3 -c \"import json,sys; d=json.load(sys.stdin); print(d['task'].get('compiles'), d['attempts'][0].get('host'))\"; "
                 "task-server/task log 7191 --all 2>&1 | grep -E '✗|PRUNE_SERVER_TEMPORARY|error' | head -6",
                 0),
                ("经 task-server：循环里管道读 JSON 的 python3 -c 代码里写着 'task'，不当写了 task-server/task：主 agent 放行",
                 "for n in 6778 6780; do task-server/task status $n --json | python3 -c \"\nimport sys,json\nd=json.load(sys.stdin)\na=d['attempts'][0]\n"
                 "print('$n', {k:a.get(k) for k in ('host','build_directory','build_directory_removed','build_peak','memory_peak','state')}, 'task.env CARGO_TARGET_DIR=', d['task']['environment'].get('CARGO_TARGET_DIR'))\n\"; done; "
                 "task-server/task status 6778 --json | python3 -c \"import sys,json;d=json.load(sys.stdin);print(sorted(d['attempts'][0].keys()))\"",
                 0),
                ("经 task-server：python3 -c 往 task-server/task 写了再执行它，照拒",
                 "python3 -c \"open('task-server/task','w').write('true')\"; task-server/task status 7191",
                 2)):
            results.append((label, want, task_server_decision({"tool_name": "Bash", "cwd": repo, "tool_input": {"command": command}}, repo).code))
        # 照抄的那一条：直接跑的 harness 测试给出提交的写法，照主 agent 交它再判放行；范围声明挪到 submit 前面、内存包装去掉。
        # 子 agent 那一份的头一行是「→ 写进报告「要主 agent 跑的命令」一节的那一条」（它自己不交，②），照抄的那条照样由主 agent 交、照主 agent 判。
        # 挑目标的 cargo test 是要声明范围的验证命令，经 submit 交的照样要声明、声明写在 submit 前面（run-scope-guard.sh 文件头「经 task-server 提交」，
        # lib_run_scope.py 的 scope_runs_in_text 剥开 submit 照同一张表判）：照抄的那条带四段声明（为什么一段是占位，run_scope_declaration），
        # 照抄的那条另拿范围声明闸的判法（lib_run_scope.decide_scope）判一遍也要放行（PROJECT_CODE_GUARD_BREAK=copy-without-run-scope 下头一格与第三格红）
        def run_scope_code(hook_input):
            try:
                run_scope_spec = importlib.util.spec_from_file_location("lib_run_scope_for_copy", os.path.join(HOOK_DIRECTORY, "lib_run_scope.py"))
                run_scope = importlib.util.module_from_spec(run_scope_spec)
                run_scope_spec.loader.exec_module(run_scope)
                run_scope.load_judging_libraries()
                return run_scope.decide_scope(hook_input, work).code
            except Exception as error:
                return f"范围声明闸的判法载不进或判的时候抛了 {error!r}"
        # 起测试的两条照 .claude/agent-common.md「日常开发的测试不跑整个包」写成点名测试全名、带 --exact 的形态（只点测试目标的照抄，服务提交时照 unnamed_run_problem 拒）
        placeholder_reason = "为什么=<为什么这么跑、这个范围为什么合适>'"
        harness_test_name = "overwrite_publishes_the_second_version_through_the_same_commit_shape"
        library_test_name = "model::ring_positions_and_tree_node_bounds::tests::the_model_places_roots_by_the_slots_per_region_it_was_built_with"
        printed_copies = []   # 下面「照抄命令点名」那一格逐条判的：照抄几格与主 agent 直接跑被拒那一格打出的照抄命令
        for label, agent_type, command, want_copy in (
                ("经 task-server：照抄：实现员直接跑 harness 测试，照抄的那条经 task-server 提交", writer,
                 f"cargo test -p singlefs-verification --test overwrite_in_one_instance -- {harness_test_name} --exact",
                 "SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-verification --test overwrite_in_one_instance 里按全名点名的测试 "
                 f"{harness_test_name}; 范围=1 个测试; {placeholder_reason} task-server/task submit -- cargo test -p singlefs-verification --test overwrite_in_one_instance "
                 f"-- {harness_test_name} --exact"),
                ("经 task-server：照抄：经内存包装直接起的去掉包装", None,
                 "bash scripts/hosts/run-with-memory-cap.sh 8G cargo build -p singlefs-verification", "task-server/task submit -- cargo build -p singlefs-verification"),
                ("经 task-server：照抄：带 && 的整条包进 bash -c", writer,
                 f"cd crates/singlefs-verification && cargo test --lib -- {library_test_name} --exact",
                 f"SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test --lib 里按全名点名的测试 {library_test_name}; 范围=1 个测试; {placeholder_reason} "
                 f"task-server/task submit -- bash -c 'cd crates/singlefs-verification && cargo test --lib -- {library_test_name} --exact'")):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            lines = (task_server_decision(hook_input, work).message or "").splitlines()
            copy_heading = "→ 写进报告「要主 agent 跑的命令」一节的那一条" if agent_type else "→ 照抄"
            got_copy = lines[1] if len(lines) > 1 and lines[0].startswith(copy_heading) else ""
            printed_copies.append(got_copy)
            refed = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": got_copy}}
            results.append((label, (want_copy, 0, 0), (got_copy, task_server_decision(refed, work).code if got_copy else None,
                                                       run_scope_code(refed) if got_copy else None)))
        # 走真实入口：从标准输入喂 JSON，拒绝退出码 2、stderr 带出路，放行退出码 0；拒绝落进检出记录
        script = os.path.join(hook_dir, "project-code-guard.sh")
        detections = os.path.join(work, "detections.jsonl")
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=work, AGENT_HOOK_DETECTIONS=detections)
        def count_detections():
            return sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
        refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}}))
        results.append(("stdin:实现员跑 --all 拒绝（退出码 2）", 2, refused.returncode))
        # 出路写规矩（一切跑项目代码只认经 task-server/task 起）与子 agent 的去处（写进报告「要主 agent 跑的命令」一节交回）
        results.append(("stdin:拒绝时 stderr 写了出路：规矩一句、子 agent 写进报告交回", 1,
                        int("→" in refused.stderr and "一切跑项目代码的命令只认经 task-server/task 起" in refused.stderr and "写进报告「要主 agent 跑的命令」一节" in refused.stderr)))
        # 走真实入口：主 agent 直接跑被拒，stderr 最前面是照抄的说明与经 task-server 提交的那条（照它再喂一遍放行），不贴「谁能跑哪一类」那一段；
        # 检出记录写结局「被拒」（PROJECT_CODE_GUARD_BREAK=task-server-copy-dropped 下第一格红，detection-outcome-dropped 下第三格红）
        def last_detection():
            """检出记录的最后一条；还没有记录时交 {}（那几格照样判红，不让自检抛异常）。"""
            if not os.path.exists(detections):
                return {}
            with open(detections, encoding="utf-8") as handle:
                lines = handle.read().splitlines()
            return json.loads(lines[-1]) if lines else {}
        main_command = "bash .claude/singlefs-ai-sop/scripts/doc-lint.sh ."
        main_refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": main_command}}))
        main_lines = main_refused.stderr.splitlines()
        main_outcome = last_detection().get("outcome")
        copied = main_lines[1] if len(main_lines) > 1 and main_lines[0].startswith("→ 照抄") else ""
        copied_judged = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": copied}}))
        results.append(("stdin:主 agent 直接跑被拒时最前面是能照抄的经 task-server 提交的那条（照抄的那条再判放行）", (2, "task-server/task submit -- " + main_command, 0),
                        (main_refused.returncode, copied, copied_judged.returncode)))
        results.append(("stdin:主 agent 被拒时不贴「谁能跑哪一类」那一段，规矩指到提交写法", True,
                        "task-server/README.md「提交写法」" in main_refused.stderr
                        and "→ 交回前不跑它们" not in main_refused.stderr and "→ 规矩只贴出处" not in main_refused.stderr))
        results.append(("stdin:被拒的检出记录写了结局「被拒」", "被拒", main_outcome))
        # 照抄命令点名：照抄那几格与上面主 agent 直接跑被拒那一格打出的每条照抄命令（经 task-server/task submit 交的那一条；没打出的不算），
        #   去掉起头的赋值（范围声明），拆出 submit 与 -- 之间的选项与 -- 后面的命令，照服务提交时那一判（task-server/task_server_lib.py 的 unnamed_run_problem，
        #   经 lib_project_code_runs.task_server_library 载入，不另写一份）判：-- 后面那一条与 bash -c 里命令位置上的每一条（shell_inner_commands，同服务 ⑦）都要判成点了名，至少判到一条。
        #   对着 scripts/gate/merge-and-verify.sh 自证「重跑命令点名」那一格写。不指定范围的整道门禁那一格（lkmm.sh 整道）打出的那一条也在这里喂：它带 --full-gate（submit_loosening_flag），
        #   照服务的判法带了的不判点名；没带的当没点名拒，另归范围声明闸的 commit_gate_problem（提交时才许跑）。
        #   PROJECT_CODE_GUARD_BREAK=copy-test-names-dropped 下照抄的两条 cargo test 丢了测试全名与 --exact，这一格红；task-server-copy-dropped、task-server-direct-allowed 下一条照抄都没打出，判到 0 条，这一格红；
        #   copy-without-full-gate-flag 下整道门禁那一条没带 --full-gate，这一格红
        printed_copies.append(copied)
        # 整道门禁（lkmm.sh 不带 --check / --item）直接跑被拒：照抄的那一条是提交时那一趟的写法，带 --full-gate、四段范围声明写在 submit 前面（为什么是占位），照它再喂一遍放行
        stage_command = "bash .claude/gate.d/lkmm.sh"
        stage_refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": stage_command}}))
        stage_lines = stage_refused.stderr.splitlines()
        stage_copied = stage_lines[1] if len(stage_lines) > 1 and stage_lines[0].startswith("→ 照抄") else ""
        printed_copies.append(stage_copied)

        def copy_naming_problems(copies):
            """copies 里打出了的每条照抄命令 → (判到的条数, [没点名或拆不开的，逐条一句])。"""
            library, problem = project_code_runs.task_server_library()
            if library is None:
                return 0, [f"判据载不进：{problem}"]
            judged, problems = 0, []
            for copy_command in copies:
                if not copy_command:
                    continue
                try:
                    words = shlex.split(copy_command)
                except ValueError as error:
                    problems.append(f"拆不开（{error}）：{copy_command[:200]}")
                    continue
                while words and project_code_runs.ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
                    words = words[1:]
                if len(words) < 3 or words[:2] != ["task-server/task", "submit"] or "--" not in words:
                    problems.append(f"不是经 task-server/task submit 交的：{copy_command[:200]}")
                    continue
                options, argv = words[2:words.index("--")], words[words.index("--") + 1:]
                directory = options[options.index("--cwd") + 1] if "--cwd" in options and options.index("--cwd") + 1 < len(options) else work
                judged += 1
                for one in [argv] + library.shell_inner_commands(argv):
                    verdict = library.unnamed_run_problem(one, directory, "--full-gate" in options, "--whole-package-confirmed" in options)
                    if verdict:
                        problems.append(f"{shlex.join(one)[:200]}：{verdict['problem'][:200]}")
            return judged, problems

        copies_judged, copy_problems = copy_naming_problems(printed_copies)
        results.append(("经 task-server：照抄：打出的每条照抄命令照 unnamed_run_problem 判都点了名", (True, []),
                        (copies_judged > 0, copy_problems)))
        # 走真实入口：判不了的照常放行、stderr 报一句、记一条检出（结局「只记」）——hook 输入不是 JSON 对象（坏 JSON、解出来是数组；会话号写读不到，
        # 诊断带钩子名与开头的字节）、判的时候抛异常（tool_input.command 不是字符串，decide 里切词抛 TypeError）、here-string 喂给 bash 的是变量。
        # PROJECT_CODE_GUARD_BREAK=failure-unrecorded 下前三格红，disable-check 下第三格红，LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下第四格红
        for label, raw in (("stdin:坏 JSON 放行（退出码 0）、stderr 写诊断、记一条检出", b'{"tool_name": "Bash", "tool_input": '),
                           ("stdin:不是 JSON 对象放行（退出码 0）、stderr 写诊断、记一条检出", b'["Bash"]')):
            recorded = count_detections()
            unreadable = subprocess.run(["bash", script], capture_output=True, env=environment, input=raw)
            stderr_text = unreadable.stderr.decode("utf-8", "replace")
            results.append((label, (0, True, 1, "读不到", "只记"),
                            (unreadable.returncode, "project-code-guard.sh 收到的 hook 输入不是 JSON 对象" in stderr_text and repr(raw[:10])[2:-1] in stderr_text,
                             count_detections() - recorded, last_detection().get("session_id"), last_detection().get("outcome"))))
        recorded = count_detections()
        raising = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-raise", "agent_type": writer,
                                                   "tool_input": {"command": ["cargo", "test", "--all"]}}))
        results.append(("stdin:判的时候抛异常放行（退出码 0）、stderr 写诊断、记一条检出", (0, True, 1, "s-raise", "只记"),
                        (raising.returncode, "project-code-guard.sh 没判成" in raising.stderr, count_detections() - recorded,
                         last_detection().get("session_id"), last_detection().get("outcome"))))
        recorded = count_detections()
        fed_variable = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-fed", "agent_type": writer,
                                                        "tool_input": {"command": 'bash <<< "$CMD"'}}))
        results.append(("stdin:here-string 喂给 bash 的是变量认不出照拒（退出码 2）、stderr 点名判不了、记一条检出", (2, True, 1, "s-fed"),
                        (fed_variable.returncode, "here-string `$CMD`" in fed_variable.stderr, count_detections() - recorded,
                         last_detection().get("session_id"))))
        # 钩子 bash 这一层起不来时也记一条检出（PROJECT_CODE_GUARD_BREAK=bash-layer-unrecorded 下红）：PATH 前面放一份只对 /dev/fd/3 退 127 的假 python3
        shim = os.path.join(work, "bash-layer-shim")
        os.makedirs(shim, exist_ok=True)
        with open(os.path.join(shim, "python3"), "w", encoding="utf-8") as handle:
            handle.write(f'#!/bin/bash\nif [ "$1" = /dev/fd/3 ]; then exit 127; fi\nexec {sys.executable} "$@"\n')
        os.chmod(os.path.join(shim, "python3"), 0o755)
        recorded = count_detections()
        bash_layer = subprocess.run(["bash", script], capture_output=True, text=True, env=dict(environment, PATH=shim + os.pathsep + os.environ.get("PATH", "")),
                                    input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-bash-layer", "agent_type": writer, "tool_input": {"command": "ls"}}))
        results.append(("一百三十七：钩子 bash 这一层起不来（假 python3 退 127）：放行（退出码 0）、记一条检出", (0, 1, "s-bash-layer", "只记"),
                        (bash_layer.returncode, count_detections() - recorded, last_detection().get("session_id"), last_detection().get("outcome"))))
        # PATH 上没有 python3（PROJECT_CODE_GUARD_BREAK=python-missing-unrecorded 下红）：PATH 只放记检出要用的 cat、date、mkdir、dirname
        no_python = os.path.join(work, "no-python-path")
        os.makedirs(no_python, exist_ok=True)
        for tool in ("cat", "date", "mkdir", "dirname"):
            if shutil.which(tool) and not os.path.lexists(os.path.join(no_python, tool)):
                os.symlink(shutil.which(tool), os.path.join(no_python, tool))
        recorded = count_detections()
        no_python_run = subprocess.run([shutil.which("bash"), script], capture_output=True, text=True, env=dict(environment, PATH=no_python),
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-no-python", "agent_type": writer,
                                                         "tool_input": {"command": "cargo test --all"}}))
        results.append(("d137 H13：PATH 上没有 python3：放行（退出码 0）、记一条检出", (0, 1, "s-no-python", "只记"),
                        (no_python_run.returncode, count_detections() - recorded, last_detection().get("session_id"), last_detection().get("outcome"))))
        # 检出记录的位置照 lib_work_root.py 取、不退回家目录：AGENT_HOOK_DETECTIONS、SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 都不设，PATH 上没有 python3，HOME 换成临时目录
        unrooted_home = os.path.join(work, "unrooted-home")
        os.makedirs(unrooted_home, exist_ok=True)
        unrooted_environment = {name: value for name, value in environment.items() if name not in ("AGENT_HOOK_DETECTIONS", "SINGLEFS_WORK_ROOT", "SINGLEFS_DATA_ROOT")}
        unrooted_run = subprocess.run([shutil.which("bash"), script], capture_output=True, text=True, env=dict(unrooted_environment, HOME=unrooted_home, PATH=no_python),
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-unrooted", "agent_type": writer,
                                                        "tool_input": {"command": "cargo test --all"}}))
        results.append(("d137 H13：PATH 上没有 python3、工作根取不到：放行（退出码 0）、不往家目录下写、stderr 说检出没记成", (0, False, True),
                        (unrooted_run.returncode, os.path.exists(os.path.join(unrooted_home, ".local")), "检出没记成" in unrooted_run.stderr)))
        # 走真实入口：照抄的那条是门禁这类验证命令时带四段范围声明（.claude/hooks/run-scope-guard.sh 的写法，为什么一段是占位），整道门禁的还带 --full-gate，
        # 照它再喂一遍放行：项目代码闸（钩子入口）与范围声明闸（lib_run_scope.decide_scope，没带 --full-gate 的整道门禁经 submit 交当场拒）各判一遍
        # （PROJECT_CODE_GUARD_BREAK=copy-without-run-scope、task-server-copy-dropped、disable-check 下红；copy-without-full-gate-flag 下照抄的那条没带 --full-gate，红）
        stage_copied_judged = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                             input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": stage_copied}}))
        stage_declared = ("SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=门禁 lkmm.sh 的全部格（不带 --check / --item）; 范围=整轮; "
                          "为什么=<为什么这么跑、这个范围为什么合适>' task-server/task submit --full-gate -- " + stage_command)
        results.append(("stdin:主 agent 直接跑整道门禁被拒时照抄的那条经 task-server 提交、四段范围声明写在 submit 前面（为什么是占位），照它再判放行",
                        (2, stage_declared, 0, 0),
                        (stage_refused.returncode, stage_copied, stage_copied_judged.returncode,
                         run_scope_code({"tool_name": "Bash", "cwd": work, "tool_input": {"command": stage_copied}}) if stage_copied else None)))
        # 文件头「经 task-server 起」④ 走真实入口：主 agent 提交要临时服务的自证裸写 --selftest，退出码 2、stderr 给包装写法的整条
        # （PROJECT_CODE_GUARD_BREAK=temporary-service-bare-allowed、temporary-service-submit-inner-unread 下红，LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-command-bare 下红）
        bare_submitted_entry = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                              input=json.dumps({"tool_name": "Bash", "cwd": work,
                                                                "tool_input": {"command": f"task-server/task submit -- python3 {repo}/scripts/mutation/crates-mutation-rows.py --selftest"}}))
        results.append(("stdin:主 agent 经 task-server 提交要临时服务的自证裸写 --selftest 拒绝（退出码 2），stderr 给包装写法的整条", (2, 1),
                        (bare_submitted_entry.returncode,
                         int("--with-temporary-service -- python3 scripts/mutation/crates-mutation-rows.py --selftest" in bare_submitted_entry.stderr))))
        # 文件头「经 task-server 起」⑥ 走真实入口：主 agent 提交样本档 stage-selftest.sh --item 点名要临时服务的一道，退出码 2、stderr 给包装写法的整条
        # （PROJECT_CODE_GUARD_BREAK=temporary-service-stage-samples-allowed、temporary-service-submit-inner-unread 下红，LIB_PROJECT_CODE_RUNS_BREAK=temporary-service-stage-samples-unjudged 下红）；
        # 喂的门禁目录是 stage_sample_gate_directory（阶段源码原样拷进自证自己的门禁目录，不看仓里的样本档目录在不在）
        stage_sample_entry = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                            input=json.dumps({"tool_name": "Bash", "cwd": repo, "tool_input": {
                                                "command": f"task-server/task submit -- {stage_sample_command}"}}))
        results.append(("stdin:主 agent 经 task-server 提交样本档点名要临时服务的一道、外面没套 拒绝（退出码 2），stderr 给包装写法的整条", (2, 1),
                        (stage_sample_entry.returncode,
                         int(f"--with-temporary-service -- {stage_sample_command}" in stage_sample_entry.stderr))))
        # 共用读表模块的弄坏开关变量平常设了同样退 2、点名只在自证里认（SELFTEST_ONLY_VARIABLES 里没有它时这一格红；没有对应的弄坏开关：开关读的变量自己也在被拒的那一组里）
        library_break_environment = {key: value for key, value in environment.items() if key not in (SELFTEST_MARK_VARIABLE, GUARD_BREAK_VARIABLE)}
        library_break_run = subprocess.run(["bash", script, "--library-break-probe"], capture_output=True, text=True,
                                           env=dict(library_break_environment, LIB_CODE_DISCIPLINE_LINTS_BREAK="entry-form-unchecked"), input="")
        results.append(("平常跑设了 LIB_CODE_DISCIPLINE_LINTS_BREAK：退 2 并点名只在自证里认", (2, True),
                        (library_break_run.returncode, "LIB_CODE_DISCIPLINE_LINTS_BREAK" in library_break_run.stderr and "自证" in library_break_run.stderr)))
        unknown_breaks = [item for item in os.environ.get(GUARD_BREAK_VARIABLE, "").split(",") if item and item not in KNOWN_GUARD_BREAK_ITEMS]
        results.append((f"{GUARD_BREAK_VARIABLE} 里的项都认得（认得的：{'、'.join(KNOWN_GUARD_BREAK_ITEMS)}）", [], unknown_breaks))
        # 走真实入口：实现员起含被拒命令的脚本，退出码 2、stderr 写出脚本路径与行号
        in_script = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                   input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "bash s.sh"}}))
        results.append(("stdin:实现员起含被拒命令的脚本拒绝（退出码 2）", 2, in_script.returncode))
        results.append(("stdin:脚本里的拒绝 stderr 写出脚本路径与行号", 1, int(f"{work}/s.sh:4" in in_script.stderr)))
        # 走真实入口：实现员直接跑测试目标，退出码 2、stderr 写出经 task-server 提交的那条与写进报告交回的出路、不教子 agent 后台等
        # （PROJECT_CODE_GUARD_BREAK=task-server-direct-allowed 下两格红，task-server-copy-dropped 下第二格红）
        direct_run = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                    input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                                      "tool_input": {"command": "cargo test -p singlefs-core --lib"}}))
        results.append(("stdin:实现员直接跑测试目标拒绝（退出码 2）", 2, direct_run.returncode))
        results.append(("stdin:子 agent 不经 task-server 的拒绝 stderr 写了照抄的提交、写进报告「要主 agent 跑的命令」一节交回，不教后台等", 1,
                        int("✗ 不经 task-server 跑项目代码被拒" in direct_run.stderr and "task-server/task submit -- cargo test -p singlefs-core --lib" in direct_run.stderr
                            and "写进报告「要主 agent 跑的命令」一节交回" in direct_run.stderr and "task-server/task wait <号>" not in direct_run.stderr)))
        recorded = count_detections()
        # 走真实入口：脚本文件不存在时照常放行（退出码 0），stderr 点名那个文件，并记一条检出
        missing = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "bash missing.sh"}}))
        results.append(("stdin:脚本文件不存在照常放行（退出码 0）", 0, missing.returncode))
        results.append(("stdin:脚本文件不存在时 stderr 点名它、记一条检出", 1,
                        int(f"{work}/missing.sh" in missing.stderr and count_detections() == recorded + 1)))
        results.append(("stdin:放行时的检出记录写了结局「只记」", "只记", last_detection().get("outcome")))
        recorded = count_detections()
        # 走真实入口：同一条命令里 heredoc 写出不含要拒的命令的脚本再起它，照常放行（退出码 0），stderr 不报、不记检出
        written_light = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                           "command": "cat > gen-light.sh <<'EOF'\necho 轻\nEOF\nbash gen-light.sh"}}))
        results.append(("stdin:heredoc 写出的轻脚本照常放行（退出码 0）", 0, written_light.returncode))
        results.append(("stdin:heredoc 写出的轻脚本 stderr 不报「不存在」、不记检出", 1,
                        int("不存在" not in written_light.stderr and count_detections() == recorded)))
        written_heavy = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                           "command": "cat > gen.sh <<'EOF'\ncargo test --all\nEOF\nbash gen.sh"}}))
        results.append(("stdin:heredoc 写出的含 --all 的脚本拒绝（退出码 2）", 2, written_heavy.returncode))
        results.append(("stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号", 1, int(f"{work}/gen.sh:1" in written_heavy.stderr)))
        recorded = count_detections()
        # 走真实入口：hook 旁边没有共用模块时照常放行（退出码 0），stderr 点名两份模块，并记一条检出
        lonely_hook_directory = os.path.join(work, "hook-without-shared-modules")
        os.makedirs(lonely_hook_directory)
        for lonely_name in ("project-code-guard.sh",) + PROJECT_CODE_GUARD_PART_NAMES:   # 没有共用模块的钩子目录：入口与它的 python 各部分在，lib_*.py 不在
            shutil.copy(os.path.join(hook_dir, lonely_name), lonely_hook_directory)
        without_library = subprocess.run(["bash", os.path.join(lonely_hook_directory, "project-code-guard.sh")], capture_output=True, text=True, env=environment,
                                         input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}}))
        results.append(("stdin:读不到共用模块照常放行（退出码 0）", 0, without_library.returncode))
        results.append(("stdin:读不到共用模块时 stderr 点名两份模块、记一条检出", 1,
                        int("lib_project_code_runs.py" in without_library.stderr and "lib_shell_words.py" in without_library.stderr
                            and count_detections() == recorded + 1)))
        # 走真实入口：钩子目录里只有入口、它的 python 各部分、lib_project_code_runs.py 的各部分、lib_shell_words.py 的各部分与 lib_workspace_cargo_lock.py（没有别的可选模块），
        # 仓里 scripts/gate/admission.py 是个一调就抛 NameError 的假文件：钩子不读它，点名登记目标的 cargo test 照旧不经 task-server 被拒（退出码 2），不落到入口的「读不到共用模块」「没判成、照常执行」
        # （这一份拷贝漏了哪个部分文件，钩子就读不到共用模块、退 0，这一格红）
        raising_repository = os.path.join(work, "repository-with-raising-admission")
        os.makedirs(os.path.join(raising_repository, ".claude", "hooks"))
        os.makedirs(os.path.join(raising_repository, "scripts"))
        os.makedirs(os.path.join(raising_repository, "scripts", "gate"), exist_ok=True)
        for name in (("project-code-guard.sh",) + PROJECT_CODE_GUARD_PART_NAMES + ("lib_project_code_runs.py",) + project_code_runs.PROJECT_CODE_RUNS_PART_NAMES
                     + (project_code_runs.PROJECT_CODE_RUNS_BOOTSTRAP_FILE_NAME,)
                     + ("lib_shell_words.py", "lib_shell_words_core.py", "lib_shell_words_embedded.py", "lib_workspace_cargo_lock.py")):
            shutil.copy(os.path.join(hook_dir, name), os.path.join(raising_repository, ".claude", "hooks", name))
        for name in ("scripts", "singlefs-ai-sop"):
            os.symlink(os.path.join(os.path.dirname(hook_dir), name), os.path.join(raising_repository, ".claude", name))
        with open(os.path.join(raising_repository, "scripts", "gate", "admission.py"), "w", encoding="utf-8") as handle:
            handle.write("def test_function_is_marked_ignored(*arguments):\n    return definitions_marked_ignored(*arguments)\n")
        raising = subprocess.run(["bash", os.path.join(raising_repository, ".claude", "hooks", "project-code-guard.sh")], capture_output=True, text=True,
                                 env=environment, input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                     "command": "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --test crash_enumeration_new_pool_file_creation_stream"}}))
        results.append(("stdin:admission.py 判的那一刻抛异常时，点名标了 #[ignore] 的登记目标、不带 --ignored 的 cargo test 照拒（退出码 2）", 2, raising.returncode))
        results.append(("stdin:admission.py 判的那一刻抛异常时 stderr 不是「没判成、照常执行」", 1, int("没判成" not in raising.stderr)))
        # 走真实入口：引导读不到、编不过任何一份 python 部分文件：按缺依赖的处置拒（退出码 2、stderr 点名那一份），不抛错退 1（退 1 时 bash 那一层当「没判成」放行）。
        # 截成空文件也一样拒（第一份 decisions 截空、带 main 的那一份 selftest 截空各一格）；
        # 开关 PROJECT_CODE_GUARD_BREAK=part-unreadable-passes 下引导照原样抛错、退 1、bash 那一层放行，下面六格红；检出写自己的一份，不搅别处数检出的格
        bootstrap_repository = os.path.join(work, "repository-with-damaged-part")
        bootstrap_hooks = os.path.join(bootstrap_repository, ".claude", "hooks")
        os.makedirs(bootstrap_hooks)
        for name in ("scripts", "singlefs-ai-sop"):
            os.symlink(os.path.join(os.path.dirname(hook_dir), name), os.path.join(bootstrap_repository, ".claude", name))
        bootstrap_environment = dict(environment, AGENT_HOOK_DETECTIONS=os.path.join(work, "bootstrap-detections.jsonl"))
        bootstrap_input = json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}})
        for part_name in PROJECT_CODE_GUARD_PART_NAMES:
            for damage, damage_label in (("missing", "读不到"), ("syntax-error", "编不过"), ("emptied", "截空")):
                for copied_name in ("project-code-guard.sh",) + PROJECT_CODE_GUARD_PART_NAMES:
                    shutil.copy(os.path.join(hook_dir, copied_name), bootstrap_hooks)
                if damage == "missing":
                    os.remove(os.path.join(bootstrap_hooks, part_name))
                else:
                    with open(os.path.join(bootstrap_hooks, part_name), "w", encoding="utf-8") as handle:
                        handle.write("" if damage == "emptied" else "def broken(:\n")
                bootstrap_run = subprocess.run(["bash", os.path.join(bootstrap_hooks, "project-code-guard.sh")], capture_output=True, text=True,
                                               env=bootstrap_environment, input=bootstrap_input)
                results.append((f"引导:{damage_label}部分文件 {part_name} 按拒判（退 2、点名那一份）", (2, True),
                                (bootstrap_run.returncode, f"部分文件 {os.path.join(bootstrap_hooks, part_name)}" in bootstrap_run.stderr and "按拒判" in bootstrap_run.stderr)))
        # 走真实入口：共用模块 lib_project_code_runs.py 的部分文件读不到：库抛的异常点名那一份，入口照缺共用模块的处置照常放行（退出码 0）、stderr 点名那一份、记一条检出。
        # 开关 LIB_PROJECT_CODE_RUNS_BREAK=part-unreadable-passes 下库抛的是原来的读文件错，说明里没有「部分文件」，这一格红
        library_part_repository = os.path.join(work, "repository-with-damaged-library-part")
        library_part_hooks = os.path.join(library_part_repository, ".claude", "hooks")
        os.makedirs(library_part_hooks)
        for name in ("scripts", "singlefs-ai-sop"):
            os.symlink(os.path.join(os.path.dirname(hook_dir), name), os.path.join(library_part_repository, ".claude", name))
        for name in (("project-code-guard.sh",) + PROJECT_CODE_GUARD_PART_NAMES + ("lib_project_code_runs.py",) + project_code_runs.PROJECT_CODE_RUNS_PART_NAMES
                     + (project_code_runs.PROJECT_CODE_RUNS_BOOTSTRAP_FILE_NAME,)
                     + ("lib_shell_words.py", "lib_shell_words_core.py", "lib_shell_words_embedded.py")):
            shutil.copy(os.path.join(hook_dir, name), library_part_hooks)
        library_part_path = os.path.join(library_part_hooks, project_code_runs.PROJECT_CODE_RUNS_PART_NAMES[0])
        os.remove(library_part_path)
        library_part_environment = dict(environment, AGENT_HOOK_DETECTIONS=os.path.join(work, "library-part-detections.jsonl"))
        library_part_run = subprocess.run(["bash", os.path.join(library_part_hooks, "project-code-guard.sh")], capture_output=True, text=True,
                                          env=library_part_environment, input=bootstrap_input)
        library_part_records = 0
        if os.path.exists(library_part_environment["AGENT_HOOK_DETECTIONS"]):
            with open(library_part_environment["AGENT_HOOK_DETECTIONS"], encoding="utf-8") as handle:
                library_part_records = sum(1 for _ in handle)
        results.append(("stdin:共用模块的部分文件读不到照常放行（退出码 0）、stderr 点名那一份、记一条检出", (0, True, 1),
                        (library_part_run.returncode, f"部分文件 {library_part_path}" in library_part_run.stderr, library_part_records)))
        # 走真实入口：主工作区里直接起、编译目录会落进主仓的 cargo（现场是临时 git 仓当主工作区）。WORKSPACE_CARGO_LOCK_BREAK 打开时这几格红（项与格见文件头）
        if cargo_lock is None:
            results.append(("编译目录：读得到 lib_workspace_cargo_lock.py", 0, 1))
        else:
            target_work = os.path.join(work, "cargo-target")
            os.makedirs(target_work)
            target_main = cargo_lock.build_main_workspace(target_work)
            # 编译目录（文件头「再一道」编译目录那一段）：CLAUDE_PROJECT_DIR 指到自检现场的主工作区，在它里面起；会话环境里的 CARGO_TARGET_DIR 去掉
            # 判在「谁能跑哪一类」那一层（decide）：在进程里照这一格的环境判它（经真入口直接起的 cargo 在 task_server_decision 那一层另拒，文件头「经 task-server 起」）
            def hook_in_target_workspace(command, agent_type=None):
                hook_input = {"tool_name": "Bash", "cwd": target_main, "tool_input": {"command": command}}
                if agent_type:
                    hook_input["agent_type"] = agent_type
                return category_layer_run(hook_input, target_main, {"CARGO_HOME": os.path.join(target_work, "cargo-home")},
                                          ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR"))
            bare_main = hook_in_target_workspace("bash scripts/hosts/run-with-memory-cap.sh 4G cargo build")
            results.append(("编译目录：主 agent 在主工作区裸起 cargo build：拒（退出码 2）", 2, bare_main.returncode))
            capped_writer = hook_in_target_workspace("bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --lib", writer)
            results.append(("编译目录：子 agent 经内存包装在主工作区起 cargo test：拒（退出码 2）", 2,
                            capped_writer.returncode if "编进主仓" in capped_writer.stderr else 0))
            results.append(("编译目录：拒绝说明给出经 task-server 提交的出路", 1,
                            int("编进主仓" in bare_main.stderr and "→ 怎么办" in bare_main.stderr
                                and "task-server/task submit -- " in bare_main.stderr and "经 CARGO_TARGET_DIR 交给命令" in bare_main.stderr
                                and "往上第一层有 .git 的那一层" in bare_main.stderr and "解开后的 --cwd 本身" in bare_main.stderr)))
            results.append(("编译目录：经 run-build-directory.sh 包一层起：放行（退出码 0）", 0,
                            hook_in_target_workspace("bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/run-build-directory.sh cargo build").returncode))
            # 派发起法里面那条照判（LIB_PROJECT_CODE_RUNS_BREAK=dispatch-launchers-unseen 下第一格红）
            dispatched_bare = hook_in_target_workspace("bash scripts/hosts/multi-host-run.sh --peer --label x -- cargo test -p singlefs-verification --lib", "tooling-writer")
            results.append(("编译目录：子 agent 经 multi-host-run.sh --peer 在主工作区起 cargo test、没给编译目录：拒（退出码 2）", 2,
                            dispatched_bare.returncode if "编进主仓" in dispatched_bare.stderr else 0))
            results.append(("编译目录：子 agent 经 multi-host-run.sh --peer 与 run-build-directory.sh 起：放行（退出码 0）", 0,
                            hook_in_target_workspace("bash scripts/hosts/multi-host-run.sh --peer --label x -- bash scripts/hosts/run-build-directory.sh cargo test -p singlefs-verification --lib",
                                                     "tooling-writer").returncode))
            results.append(("编译目录：带仓外 CARGO_TARGET_DIR：放行（退出码 0）", 0,
                            hook_in_target_workspace(f"CARGO_TARGET_DIR={os.path.join(target_work, 'outside-target')} bash scripts/hosts/run-with-memory-cap.sh 4G cargo build").returncode))
            results.append(("编译目录：cargo fmt：放行（退出码 0）", 0, hook_in_target_workspace("cargo fmt --all").returncode))
            # 只定义、调不到的 function 与判项目代码那一处同一个口径（WORKSPACE_CARGO_LOCK_BREAK=uncalled-functions-judged 下第一格红，
            # LIB_SHELL_WORDS_BREAK=function-calls-literal-only 下第二格红）
            results.append(("编译目录：只定义、调不到的 function 函数体：放行（退出码 0）", 0, hook_in_target_workspace("function noop { cargo build; }").returncode))
            results.append(("编译目录：function 定义之后拆开拼写调用：拒（退出码 2）", 2, hook_in_target_workspace('function noop { bash scripts/hosts/run-with-memory-cap.sh 4G cargo build; }; "no"op').returncode))
            results.append(("lib_workspace_cargo_lock.py 的自检", 0, cargo_lock.selftest()))
    finally:
        project_code_runs.lib_project_code_runs_base.SAMPLE_INDEX_REPOSITORY = real_sample_index
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 scan_uses()、scan_decision()、task_server_decision()、script_path_of()、executed_file_is_shell_script()、file_on_disk()、script_text()、command_lines() "
              "与共用的 lib_project_code_runs.py、lib_shell_words.py；"
              "PROJECT_CODE_GUARD_BREAK 设着的话（项与红的格见文件头）这里本来就该红")
        return 1
    refused_count = sum(1 for item in results if item[1] == 2)
    print(f"  ✓ 自检通过（查了 {len(results)} 种，其中该拒 {refused_count} 种）：外部程序（qemu-system-*、herd7、e152-file-system-benchmark，只问版本的 herd7 放行）与认不出起什么的写法拒绝，"
          "不经 task-server 跑项目代码（会编的 cargo、编出来的二进制、本仓的脚本）主 agent 与子 agent 一样拒、只经 task-server 提交；"
          "子 agent 起 task-server/task 的 submit、wait、cancel、clean 拒（带 --item 的、命名检查、自证、样本档、弄坏开关都算，读进来的脚本正文里的照判），status、log 放行，主 agent 交同一条放行；"
          "写进脚本再执行的（bash / ./ / capped.sh / source / 嵌套）读进去照判、拒绝写出路径与行号，脚本读不到或看不全的放行并记检出；"
          "同一条命令里 heredoc（cat / tee）写出、cp 拷出、cat >> / &> 与 tee 拷出再执行的拿写出的内容判、不记「不存在」，认不出的写法写出再起的照认不出拒；"
          "直接执行的只读 `#!` 指到 shell 的与没 `#!` 的 .sh（.rs、.md、没扩展名的文本不读不记），同一份脚本一次判定里只读一遍、只判一遍；"
          "命令词带通配的照 bash 展开、按展开出来的那几份判，带 POSIX 字符类、等价类、整理符号的交给 bash 自己展开、判不出的命令词照拒；"
          "命令词与参数里的花括号照 bash 展开再判；"
          "认不出起什么的写法（命令词、脚本路径、cargo 参数由变量、命令替换、位置参数拼出，名单外启动器读进来的词，喂给 shell 的不是字面）照拒，脚本里自己的转发函数按参数判；"
          "直接跑被拒时最前面给经 task-server 提交的那条命令（要声明范围的验证命令另带四段范围声明）；检出记录写结局（被拒 / 只记）")
    return 0

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（PROJECT_CODE_GUARD_SELFTEST_MARK 对得上）而下面这些变量有一个非空
# （本钩子的，加上它导入的 lib_project_code_runs.py、lib_shell_words.py、lib_workspace_cargo_lock.py、lib_code_discipline_lints.py 的：它们的开关一样能放宽这道闸），读 hook 的 JSON 之前就退 2、点名变量只在自证里认，不管值是什么。
# 自证起的子进程的标记（SELFTEST_MARK_VARIABLE、SELFTEST_MARK_CONTENT）定义在入口的引导里：引导在这几份载入之前就要认它。
SELFTEST_ONLY_VARIABLES = (GUARD_BREAK_VARIABLE, "LIB_PROJECT_CODE_RUNS_BREAK", "LIB_SHELL_WORDS_BREAK", "WORKSPACE_CARGO_LOCK_BREAK", "LIB_CODE_DISCIPLINE_LINTS_BREAK")

def refuse_break_outside_selftest(arguments):
    set_names = [name for name in SELFTEST_ONLY_VARIABLES if os.environ.get(name)]
    if not set_names or arguments == ["--selftest"] or os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT:
        return None
    for name in set_names:
        print(f"  ✗ project-code-guard.sh：{name}={os.environ[name]} 是弄坏开关，只在自证（project-code-guard.sh --selftest）里认，平常跑设了就拒，这一条命令没判", file=sys.stderr)
    print(f"  → 怎么办：钩子的环境是 Claude Code 进程的环境，Bash 命令里 unset 改不到它：退出这个会话，在起 claude 的 shell 里 unset {'、'.join(set_names)} 再起；"
          f"要证红就跑 {set_names[0]}=<项> bash .claude/hooks/project-code-guard.sh --selftest", file=sys.stderr)
    return 2

def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest(sys.argv[2:])
    if refused is not None:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    raw_input = sys.stdin.buffer.read()
    detections_path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
    hook_input, unreadable = parsed_hook_input(raw_input)
    if unreadable:   # 不是 JSON 对象：照常执行，stderr 打一行可定位的诊断、记一条检出（会话号读不到，照实写读不到）
        print(f"  ! {unreadable}", file=sys.stderr)
        if not guard_break_is_set("failure-unrecorded"):
            record_detection(UNREADABLE_HOOK_INPUT, [unreadable], detections_path, "只记")
        return 0
    if project_code_runs is None:
        warning = f"project-code-guard.sh {shared_library_problem(hook_dir)}，这条命令没判、照常执行"
        print(f"  ! {warning}", file=sys.stderr)
        record_detection(hook_input, [warning], detections_path, "只记")
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    try:
        decision = task_server_decision(hook_input, project_root)
    except Exception as error:   # 判的时候抛了：照常执行，stderr 报一句、记一条检出（与读不到共用模块那一支同样记「只记」）
        warning = f"project-code-guard.sh 没判成（{error!r}），这条命令照常执行"
        print(f"  ! {warning}", file=sys.stderr)
        if not guard_break_is_set("failure-unrecorded"):
            record_detection(hook_input, [warning], detections_path, "只记")
        return 0
    findings = [f"项目代码闸没判全：{notice}" for notice in decision.notices]
    for notice in decision.notices:
        print(f"  ! project-code-guard.sh：{notice}；{'' if decision.code else '这条命令照常执行，'}记了一条检出", file=sys.stderr)
    if decision.code:
        print(decision.message, file=sys.stderr)
        message_lines = decision.message.splitlines()
        findings.append(next((line for line in message_lines if line.startswith("✗ ")), message_lines[0]).removeprefix("✗ "))
    if findings:
        record_detection(hook_input, findings, detections_path, "被拒" if decision.code else "只记")
    return decision.code
