# lib_run_scope.py 的部分模块：自证：样本树、自证格（判法内存里判与另起入口两路）、成功句与开关轮（selftest）。
# 入口 lib_run_scope.py 按 RUN_SCOPE_PART_NAMES 的次序 import 这一份（它 import 引导那一份与判法本体 lib_run_scope_core.py，用到的名字写显式 import），不单独执行；
# 弄坏开关的名单与红的格登记在 lib_run_scope_core.py 里，开关表写在 lib_run_scope.py 的文件头。
import json
import lib_run_scope_target_sources as target_sources
import os
import re
import shutil
import subprocess
import sys
import tempfile
from lib_run_scope_bootstrap import (
    BREAK_VARIABLE_NAME, GUARD_SCRIPT_NAME, HOOK_DIRECTORY, PART_UNREADABLE_STATUS, RUN_SCOPE_BOOTSTRAP_FILE_NAME, RUN_SCOPE_PART_NAMES,
    SELFTEST_MARK_CONTENT, SELFTEST_MARK_VARIABLE,
)
from lib_run_scope_core import (
    BREAK_SWITCH_CELLS, COMMIT_GATE_STAGE_NAMES, CRASH_AMPLIFICATION_ENTRY, CRASH_SHRINK_OPTION, CRASH_SHRINK_VARIABLES, DECLARATION_VARIABLE,
    DETECTIONS_VARIABLE, LOOSENING_SUBMIT_FLAGS, MUTATION_TABLE, REPLAY_ITEMS_TABLE, STAGE_INPUTS_TABLE, SUBMIT_FLAG_USER_RULE, VERIFICATION_PACKAGE,
    VERIFICATION_TESTS_DIRECTORY, decide_scope, load_judging_libraries, run_shape_of,
)
import lib_run_scope_core


# ── 自证 ──────────────────────────────────────────────

def scope_text(kind, what, range_text, reason="改了这一道的判法，只跑改到的这几格看它转绿"):
    return f"{DECLARATION_VARIABLE}='种类={kind}; 跑什么={what}; 范围={range_text}; 为什么={reason}'"


def build_selftest_tree(work):
    """临时目录里的驱动脚本与一份仓的样子（带 .claude/hooks/ 的目录下的 scripts/ 是仓里的工具脚本）。"""
    def write(relative, content, executable=False):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(content)
        if executable:
            os.chmod(path, 0o755)
        return path
    declared_stage = scope_text("whole", "门禁 doc-text 全部格", "整轮", "提交之前补跑这一道，看全仓文字的格是不是都绿")
    write("drivers/missing.sh", "#!/usr/bin/env bash\necho 开跑\nbash .claude/gate.d/doc-text.sh .\n"
                                f"{scope_text('target', 'cargo test 一个目标', '1 个测试目标')} cargo test -p singlefs-verification --test model_contract\n")
    write("drivers/declared.sh", f"#!/usr/bin/env bash\n{declared_stage} bash .claude/gate.d/doc-text.sh .\n"
                                 f"{scope_text('target', 'cargo test 一个目标', '1 个测试目标')} cargo test -p singlefs-verification --test model_contract\n")
    write("drivers/direct.sh", "#!/usr/bin/env bash\ncargo test --workspace\n", executable=True)
    write("drivers/outer.sh", "#!/usr/bin/env bash\nbash drivers/missing.sh\n")
    write("drivers/flagged.sh", "#!/usr/bin/env bash\ntask-server/task submit --full-gate -- ls\n")
    # 交给服务的那一条里套着分片入口：一道文件头写了「# task-server-shards: *」的门禁阶段（服务认它分片），驱动脚本里那一行是它（sharded-wrapped.sh）或是没有分片标记的门禁（unsharded.sh）
    write(".claude/gate.d/sharded-sample.sh", "#!/usr/bin/env bash\n# task-server-shards: *\n")
    write("drivers/sharded-wrapped.sh", f"#!/usr/bin/env bash\n{scope_text('items', '带分片标记的门禁点名一项', '1 项')} bash .claude/gate.d/sharded-sample.sh --item row-a .\n")
    write("drivers/unsharded.sh", f"#!/usr/bin/env bash\n{scope_text('items', '门禁 doc-text 点名一格', '1 格')} bash .claude/gate.d/doc-text.sh --check prime-marks .\n")
    write("repo/.claude/hooks/placeholder", "仓的样子：这一层带 .claude/hooks/\n")
    write("repo/scripts/tool.sh", "#!/usr/bin/env bash\ncargo test --workspace\n")
    # 崩溃域闸读的登记表与测试源码：登记过的崩溃枚举目标、登记过而没有源码（tests/<目标>.rs 与 tests/<目标>/main.rs 都不在）的崩溃枚举目标、登记过的放量分项目标、
    #   没登记的目录式目标（tests/<目标>/main.rs 连同子模块，两个：子模块里代码读判器缩域变量的、哪个文件都不读缩域变量的）、
    #   代码里读判器缩域变量而注释里提到放量那个的目标、不走崩溃枚举的目标
    write(STAGE_INPUTS_TABLE, "# 样本登记表\n"
          f"crash-case:sample-stream\t-\ttest={VERIFICATION_PACKAGE}:crash_enumeration_sample_stream:sample_stream_enumerates count-line=SAMPLE\n"
          f"crash-case:sample-without-source\t-\ttest={VERIFICATION_PACKAGE}:crash_enumeration_sample_without_source:sample_without_source_enumerates count-line=SAMPLE\n")
    write(CRASH_AMPLIFICATION_ENTRY, "#!/usr/bin/env bash\nITEMS=(\n  $'sample-amplification\\tcrash_amplification_sample\\tsample_amplification_runs'\n)\n")
    write(REPLAY_ITEMS_TABLE, "# 样本合表：项名、crash-case 键、放量分项\n"
          "sample-stream-item\tcrash-case:sample-stream\t-\t# 走崩溃点重放那一套\n"
          "sample-amplified-item\t-\tsample-amplification\t# 走放量那一套\n"
          "sample-plain-item\t-\t-\t# 两套都不走\n")
    write(f"{VERIFICATION_TESTS_DIRECTORY}/crash_enumeration_sample_stream.rs", "// 样本：登记过的崩溃枚举目标\n")
    write(f"{VERIFICATION_TESTS_DIRECTORY}/crash_amplification_sample.rs", "// 样本：登记过的放量分项目标\n")
    target_sources.write_directory_target_samples(write, VERIFICATION_TESTS_DIRECTORY)
    write(f"{VERIFICATION_TESTS_DIRECTORY}/crash_judge_sample_every_state.rs",
          f"// 样本：没登记、代码里读判器缩域变量；与 {CRASH_SHRINK_VARIABLES[0]} 同一判法（注释里提到的不算读它）\n"
          f"const VARIABLE: &str = \"{CRASH_SHRINK_VARIABLES[1]}\";\n")
    write(f"{VERIFICATION_TESTS_DIRECTORY}/plain_sample.rs", "// 样本：不走崩溃枚举\n")
    write(MUTATION_TABLE, "# 样本变异表\n"
          f"sample-red-crash\tcrates/sample.rs\told\tnew\t-p {VERIFICATION_PACKAGE} --test crash_enumeration_sample_stream -- sample_stream_enumerates --exact\tsample_stream_enumerates\n"
          f"sample-red-plain\tcrates/sample.rs\told-plain\tnew-plain\t-p {VERIFICATION_PACKAGE} --test plain_sample -- plain_check --exact\tplain_check\n")
    # 命令点明的副本 copy-tree/（证红 --copy、变异门禁的根参数、submit 的 --cwd 指到它）：副本里新登记了崩溃枚举目标 crash_enumeration_copy_only 与点名它的变异行，
    #   这个目标的登记、源码与变异行主工作区这一棵（work 本身）里都没有；副本里没有主工作区的变异行 sample-red-crash
    write(f"copy-tree/{STAGE_INPUTS_TABLE}", "# 样本副本登记表\n"
          f"crash-case:copy-only\t-\ttest={VERIFICATION_PACKAGE}:crash_enumeration_copy_only:copy_only_enumerates count-line=SAMPLE\n")
    write(f"copy-tree/{VERIFICATION_TESTS_DIRECTORY}/crash_enumeration_copy_only.rs", "// 样本：副本里新登记的崩溃枚举目标\n")
    write(f"copy-tree/{MUTATION_TABLE}", "# 样本副本变异表\n"
          f"copy-only-crash\tcrates/sample.rs\told\tnew\t-p {VERIFICATION_PACKAGE} --test crash_enumeration_copy_only -- copy_only_enumerates --exact\tcopy_only_enumerates\n")
    # 提交客户端：与本仓那一份逐字节相同（lib_project_code_runs.task_server_submission 只认这样的）
    client = os.path.join(os.path.dirname(os.path.dirname(HOOK_DIRECTORY)), "task-server", "task")
    if os.path.isfile(client):
        os.makedirs(os.path.join(work, "task-server"), exist_ok=True)
        shutil.copyfile(client, os.path.join(work, "task-server", "task"))


def selftest_cells(work):
    """(说明, agent_type（None 是主 agent）, 命令, 该退 0 / 2, 当前目录)。"""
    sub = "implementation-writer"
    whole_stage = scope_text("whole", "门禁 doc-text 全部格", "整轮")
    one_cell = scope_text("items", "门禁 doc-text 一格", "1 格")
    two_patches = scope_text("items", "合入后验证两份补丁", "2 份补丁", "这一批两份补丁只改到 core 的库与验证包的一个目标，脚本只挑这几个目标")
    two_cells = "bash .claude/gate.d/code-tooling.sh --check agent-write-scope,research-script-selftests ."
    replay_declaration = scope_text("items", "崩溃点重放点名一项", "1 项")
    target_declaration = scope_text("target", "验证包一个测试目标", "1 个测试目标")
    mutation_declaration = scope_text("items", "变异门禁点名一行", "1 项")
    sharded_item = scope_text("items", "带分片标记的门禁点名一项", "1 项")
    amplification_variable, judge_variable = CRASH_SHRINK_VARIABLES
    replay_stream = "bash scripts/crash/crash-point-replay.sh --item sample-stream-item"
    judge_target = f"cargo test -p {VERIFICATION_PACKAGE} --test crash_judge_sample_every_state"
    cells = [
        # 门禁阶段
        ("门禁整道：没带声明拒", None, "bash .claude/gate.d/doc-text.sh .", 2),
        ("门禁整道：带对的声明放", None, f"{whole_stage} bash .claude/gate.d/doc-text.sh .", 0),
        ("门禁 --check 两格写成 2 格：放", None, f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} {two_cells}", 0),
        ("门禁 --check 两格写成 1 格：范围数对不上拒", None, f"{scope_text('items', '门禁 code-tooling 两格', '1 格')} {two_cells}", 2),
        ("门禁 --item 声明成 items：放", None, f"{scope_text('items', '形式化点名 proof', '1 项')} bash .claude/gate.d/formal-verification.sh --item proof .", 0),
        ("门禁 --item 声明成 whole：种类对不上拒", None, f"{scope_text('whole', '形式化', '整轮')} bash .claude/gate.d/formal-verification.sh --item proof .", 2),
        ("门禁 --item 一项声明成 items：放", sub,
         f"{scope_text('items', 'crates 变异点名一项', '1 项')} bash .claude/gate.d/crates-mutation-replay.sh --item row-a .", 0),
        ("门禁 --list 不要声明", None, "bash .claude/gate.d/code-tooling.sh --list", 0),
        ("门禁 --selftest 不要声明", sub, "bash .claude/gate.d/code-tooling.sh --selftest", 0),
        ("whole 范围没写整轮或整包拒", None, f"{scope_text('whole', '门禁 doc-text', '全部')} bash .claude/gate.d/doc-text.sh .", 2),
        # 崩溃入口
        ("崩溃点重放不指定范围跑：没带声明拒", None, "bash scripts/crash/crash-point-replay.sh", 2),
        ("崩溃点重放两项：带对的声明放", None, f"{scope_text('items', '崩溃点重放两项', '2 项')} bash scripts/crash/crash-point-replay.sh --item a --item b", 0),
        ("崩溃点重放 --list-items 不要声明", sub, "bash scripts/crash/crash-point-replay.sh --list-items", 0),
        ("子 agent 起崩溃点重放点名一项、带对的声明：放", sub,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/crash-point-replay.sh --item fixed-script", 0),
        ("崩溃放量一个分项：带对的声明放", None, f"{scope_text('items', '崩溃放量一个分项', '1 项')} bash scripts/crash/crash-amplification.sh --item tree-split-streams", 0),
        # 指定范围的参数也能缩崩溃域：带 --item 时另加展开深度参数（--crash-expand-up-to <N>）照 items 声明放，深度的值不当项名
        ("崩溃点重放点名一项另加展开深度：带对的声明放", None,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/crash-point-replay.sh --item fixed-script --crash-expand-up-to 3", 0),
        ("崩溃放量点名一个分项另加展开深度：带对的声明放", None,
         f"{scope_text('items', '崩溃放量一个分项', '1 项')} bash scripts/crash/crash-amplification.sh --crash-expand-up-to 3 --item tree-split-streams", 0),
        ("崩溃点重放点名一项另加展开深度：写成两项拒（深度的值不是项）", None,
         f"{scope_text('items', '崩溃点重放点名两项', '2 项')} bash scripts/crash/crash-point-replay.sh --item fixed-script --crash-expand-up-to 3", 2),
        ("崩溃放量全部分项：没带声明拒", None, "bash scripts/crash/crash-amplification.sh", 2),
        ("崩溃放量分项驱动：带对的声明放", None,
         f"{scope_text('target', '分项驱动一个目标', '1 个测试目标')} bash scripts/crash/crash-amplification-item.sh crash_amplification_fixed_script --threads 8", 0),
        ("崩溃放量双机驱动：没带声明拒", None, "bash scripts/crash/crash-amplification-two-hosts.sh crash_amplification_fixed_script", 2),
        # 实验复跑、合入后验证、整轮门禁
        ("replay.sh 两个实验：带对的声明放", None, f"{scope_text('items', '复跑 E142 与 E165', '2 个实验')} bash scripts/experiment-apparatus/replay.sh E142 E165", 0),
        ("replay.sh 全部实验：没带声明拒", None, "bash scripts/experiment-apparatus/replay.sh", 2),
        ("replay.sh --admission-only 不要声明", None, "bash scripts/experiment-apparatus/replay.sh --admission-only E142", 0),
        ("replay.sh --list-items 不要声明", None, "bash scripts/experiment-apparatus/replay.sh --list-items", 0),
        # 合入后验证恒是 items：不带 --item 范围数是补丁目录的份数（--crash-expand-up-to 的值不算），声明成 whole 拒（弄坏开关 kind-unchecked、merge-as-whole 下那一格红）
        ("merge-and-verify 声明成 whole：拒", None,
         f"{scope_text('whole', '合入后验证', '整轮', '这一批三份补丁碰了 harness 与 core 两个包，照整轮声明')} bash scripts/gate/merge-and-verify.sh p1 p2", 2),
        ("merge-and-verify 两份补丁：声明成 items 放", None,
         f"{scope_text('items', '合入后验证两份补丁', '2 份补丁', '这一批两份补丁只改到 core 的库与验证包的一个目标，脚本只挑这几个目标')} bash scripts/gate/merge-and-verify.sh p1 p2", 0),
        ("merge-and-verify 两份补丁写成 1 份：范围数对不上拒", None,
         f"{scope_text('items', '合入后验证两份补丁', '1 份补丁', '这一批两份补丁只改到 core 的库与验证包的一个目标，脚本只挑这几个目标')} bash scripts/gate/merge-and-verify.sh p1 p2", 2),
        ("merge-and-verify 两份补丁另加展开深度：写成 2 份补丁放（深度的值不是补丁）", None,
         f"{scope_text('items', '合入后验证两份补丁', '2 份补丁', '这一批两份补丁只改到 core 的库与验证包的一个目标，缩崩溃域看一眼')} "
         "bash scripts/gate/merge-and-verify.sh --skip-apply --crash-expand-up-to 6 p1 p2", 0),
        # 带 --item（也认 --item=X）是 items，范围数是点名的项数；带 --list-items 是问询，不要声明（弄坏开关 merge-as-whole、merge-list-items-declared 下这两格红）
        ("merge-and-verify 点名两项：声明成 items 放", None,
         f"{scope_text('items', '合入后验证点名两项', '2 项', '上次红了这两个测试目标，只重跑它们看改后是不是转绿')} "
         "bash scripts/gate/merge-and-verify.sh --skip-apply --item alpha:src/lib.rs --item=alpha:tests/beta.rs", 0),
        ("merge-and-verify 点名两项写成 1 项：范围数对不上拒", None,
         f"{scope_text('items', '合入后验证点名两项', '1 项', '上次红了这两个测试目标，只重跑它们看改后是不是转绿')} "
         "bash scripts/gate/merge-and-verify.sh --skip-apply --item alpha:src/lib.rs --item=alpha:tests/beta.rs", 2),
        ("merge-and-verify --list-items 不要声明", sub, "bash scripts/gate/merge-and-verify.sh --list-items --with-keys p1", 0),
        ("gate-staged.sh 为什么写提交前整轮：放", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=提交前门禁; 范围=整轮; 为什么=提交前整轮' bash scripts/gate/gate-staged.sh", 0),
        ("gate-staged.sh 没带声明拒", None, "bash scripts/gate/gate-staged.sh", 2),
        ("gate.sh 为什么写提交前整轮：放", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=整轮门禁; 范围=整轮; 为什么=提交前整轮' bash .claude/singlefs-ai-sop/scripts/gate.sh --staged", 0),
        ("gate.sh 为什么只写验证拒", None, f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=整轮门禁; 范围=整轮; 为什么=验证' bash .claude/singlefs-ai-sop/scripts/gate.sh --staged", 2),
        # cargo test
        ("cargo test 整工作区：没带声明拒", None, "cargo test --workspace", 2),
        ("cargo test 整工作区：带对的声明放", None, f"{scope_text('whole', 'cargo test 整工作区', '整轮', '提交之前整工作区跑一遍，确认全部测试二进制都绿')} cargo test --workspace", 0),
        ("cargo test 两个目标、带对的声明：放", None,
         f"{scope_text('target', 'harness 两个目标', '2 个测试目标')} cargo test -p singlefs-verification --test model_contract --test scenarios", 0),
        ("cargo test 两个目标写成 3 个：范围数对不上拒", None,
         f"{scope_text('target', 'harness 两个目标', '3 个测试目标')} cargo test -p singlefs-verification --test model_contract --test scenarios", 2),
        ("cargo test --lib --exact 两个测试、带对的声明：放", None,
         f"{scope_text('target', 'unit-graph 库单测两条', '2 个测试')} cargo test -p singlefs-unit-graph --lib -- keys::first keys::second --exact", 0),
        ("cargo test --exact 两个测试写成 1 个：范围数对不上拒", None,
         f"{scope_text('target', 'unit-graph 库单测两条', '1 个测试')} cargo test -p singlefs-unit-graph --lib -- keys::first keys::second --exact", 2),
        ("cargo test 整包声明成 target：种类对不上拒", None, f"{scope_text('target', 'harness', '1 个测试目标')} cargo test -p singlefs-verification", 2),
        ("子 agent cargo test 点名目标、带对的声明：放", sub,
         f"{scope_text('target', 'harness 一个目标', '1 个测试目标')} bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification --test model_contract", 0),
        ("cargo test --no-run 不要声明", sub, "cargo test -p singlefs-verification --no-run", 0),
        ("cargo test -- --list 不要声明", sub, "cargo test -p singlefs-verification --test model_contract -- --list", 0),
        ("cargo build 不要声明", sub, "cargo build --offline --all-targets", 0),
        # 包装
        ("bash -c 里的门禁没带声明拒", None, "bash -c 'bash .claude/gate.d/doc-text.sh .'", 2),
        ("bash -c 外面带声明放", None, f"{whole_stage} bash -c 'bash .claude/gate.d/doc-text.sh .'", 0),
        # here-string 与管道喂给从标准输入读命令的 shell 的字面文本当命令判（lib_shell_words.py 的 shell_standard_input；LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下这三格红）
        ("here-string 喂给 bash 的 cargo test 整工作区：没带声明拒", None, "bash <<< 'cargo test --workspace'", 2),
        ("printf 经管道喂给 bash 的门禁：没带声明拒", None, "printf 'bash .claude/gate.d/doc-text.sh .\\n' | bash", 2),
        ("echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒", None, "echo 'cargo test --workspace' | sh", 2),
        ("here-string 外面带声明放", None, f"{whole_stage} bash <<< 'bash .claude/gate.d/doc-text.sh .'", 0),
        ("capped.sh 包着的 cargo test 没带声明拒", sub, "bash scripts/hosts/capped.sh 31 cargo test -p singlefs-verification --test model_contract", 2),
        ("capped.sh 外面带声明放", sub,
         f"{scope_text('target', 'harness 一个目标', '1 个测试目标')} bash scripts/hosts/capped.sh 31 cargo test -p singlefs-verification --test model_contract", 0),
        ("env 带别的变量、没带声明拒", None, "env FOO=1 bash .claude/gate.d/doc-text.sh .", 2),
        ("env 的参数里带声明放", None, f"env {whole_stage} bash .claude/gate.d/doc-text.sh .", 0),
        ("nice 与 timeout 包着、没带声明拒", None, "nice -n 19 timeout 600 bash .claude/gate.d/doc-text.sh .", 2),
        ("nice 与 timeout 外面带声明放", None, f"{whole_stage} nice -n 19 timeout 600 bash .claude/gate.d/doc-text.sh .", 0),
        ("同一段命令前面 export 声明放", None, f"export {whole_stage}; bash .claude/gate.d/doc-text.sh .", 0),
        ("flock 包着的门禁没带声明拒", None, "flock /tmp/run-scope-sample.lock bash .claude/gate.d/doc-text.sh .", 2),
        ("flock 外面带声明放", None, f"{whole_stage} flock /tmp/run-scope-sample.lock bash .claude/gate.d/doc-text.sh .", 0),
        ("gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", None, "bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/crash/crash-point-replay.sh --item fixed-script", 2),
        ("gpu-quiesce-run.sh 外面带声明放", None,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/crash/crash-point-replay.sh --item fixed-script", 0),
        ("/usr/bin/time 包着的 cargo test 没带声明拒", None, "/usr/bin/time -v cargo test --workspace", 2),
        # 驱动脚本
        ("驱动脚本里一行缺声明拒", None, "bash drivers/missing.sh", 2),
        ("驱动脚本里每一行都带声明：放", None, "bash drivers/declared.sh", 0),
        ("外面带声明、驱动脚本里一行缺声明拒", None, f"{whole_stage} bash drivers/missing.sh", 2),
        ("驱动脚本再起驱动脚本、里面缺声明拒", None, "bash drivers/outer.sh", 2),
        ("直接执行的驱动脚本缺声明拒", None, "./drivers/direct.sh", 2),
        ("同一条命令里 heredoc 写出的驱动脚本缺声明拒", None, "cat > drivers/written.sh <<'EOF'\ncargo test --workspace\nEOF\nbash drivers/written.sh", 2),
        ("同一条命令里 heredoc 写出的驱动脚本带声明：放", None,
         f"cat > drivers/written-ok.sh <<'EOF'\n{whole_stage} bash .claude/gate.d/doc-text.sh .\nEOF\nbash drivers/written-ok.sh", 0),
        ("仓里的工具脚本不读：放行", None, "bash repo/scripts/tool.sh", 0),
        # 声明的写法
        ("声明缺段：没写为什么拒", None, f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text; 范围=整轮' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明值空：范围一段是空的拒", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text; 范围=; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明写两次：种类写两次拒", None,
         f"{DECLARATION_VARIABLE}='种类=whole; 种类=whole; 跑什么=门禁 doc-text; 范围=整轮; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("声明种类认不出拒", None,
         f"{DECLARATION_VARIABLE}='种类=all; 跑什么=门禁 doc-text; 范围=整轮; 为什么=提交之前补跑这一道，看全仓文字的格是不是都绿' bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么太短拒", None, f"{scope_text('whole', '门禁 doc-text', '整轮', '改了文档')} bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么只写要跑验证拒", None, f"{scope_text('whole', '门禁 doc-text', '整轮', '要跑验证要跑验证要跑验证要跑验证')} bash .claude/gate.d/doc-text.sh .", 2),
        ("为什么里自带分号接到上一段：放", None,
         f"{scope_text('whole', '门禁 doc-text', '整轮', '提交之前补跑; 看全仓文字的格是不是都绿')} bash .claude/gate.d/doc-text.sh .", 0),
        # 子 agent 与别的命令
        # 子 agent 直接起的验证命令这里只判声明（直接跑项目代码由 project-code-guard.sh 的 ① 拒）；经 submit 交的照 lib_project_code_runs.subagent_task_server_refusal 拒
        ("子 agent 直接起整道门禁、带对的声明：放（直接跑由 ① 拒）", sub, f"{whole_stage} bash .claude/gate.d/doc-text.sh .", 0),
        ("子 agent 起门禁两格、带对的声明：放", sub, f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} {two_cells}", 0),
        ("不是验证脚本的命令：放", sub, "ls -la && bash .claude/hooks/run-scope-guard.sh --selftest", 0),
        # 经 task-server/task submit 交给服务的那一条照判，声明写在 submit 前面或 -- 之后 env 里（样本树里放一份与本仓逐字节相同的客户端；开关 task-server-submissions-unread 下「拒」的三格红）
        ("task-server 提交的门禁整道：没带声明拒", None, "task-server/task submit -- bash .claude/gate.d/doc-text.sh .", 2),
        ("task-server 提交的门禁点名一格、submit 前面带对的声明：放", None, f"{one_cell} task-server/task submit -- bash .claude/gate.d/doc-text.sh --check prime-marks .", 0),
        ("task-server 提交的门禁点名一格、-- 之后 env 里带对的声明：放", None, f"task-server/task submit -- env {one_cell} bash .claude/gate.d/doc-text.sh --check prime-marks .", 0),
        ("task-server 提交的 cargo test 整工作区：没带声明拒", None, "task-server/task submit -- cargo test --workspace", 2),
        ("task-server 提交带 --full-gate 的门禁整道：没带声明拒", None, "task-server/task submit --full-gate -- bash .claude/gate.d/doc-text.sh .", 2),
        ("task-server 提交带 --manual 的门禁整道：没带声明拒", None, "task-server/task submit --manual -- bash .claude/gate.d/doc-text.sh .", 2),
        ("task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒", None,
         "task-server/task submit --whole-package-confirmed /tmp/run-scope-sample-confirmed -- cargo test --workspace", 2),
        ("task-server 提交带 --full-gate 的门禁整道、submit 前面带对的声明：放", None, f"{whole_stage} task-server/task submit --full-gate -- bash .claude/gate.d/doc-text.sh .", 0),
        ("子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）", sub,
         f"{scope_text('whole', '崩溃点重放', '整轮')} task-server/task submit -- bash scripts/crash/crash-point-replay.sh", 2),
        ("子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）", sub,
         f"{scope_text('target', 'harness 一个目标', '1 个测试目标')} task-server/task submit -- cargo test -p singlefs-verification --test model_contract", 2),
        ("task-server/task wait 不要声明", sub, "task-server/task wait 7", 0),
        # 提交时才许不指定范围地跑的六道门禁与两份入口：经 submit 交而 `--` 之前一个放宽标识都没带的拒，带 --full-gate 或 --whole-package-confirmed 的放（弹窗另判）；
        # 点名格的、直接起的不判这一条（六道逐道各一格见 selftest_judge_cells）
        ("task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）", None,
         f"{whole_stage} task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh", 2),
        ("task-server 提交不指定范围的崩溃点重放门禁、带 --full-gate：放", None,
         f"{whole_stage} task-server/task submit --full-gate -- bash .claude/gate.d/crash-point-replay.sh", 0),
        ("task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒", None, f"{whole_stage} task-server/task submit -- bash scripts/crash/crash-point-replay.sh", 2),
        ("task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）", None,
         f"{whole_stage} task-server/task submit -- bash scripts/crash/crash-amplification.sh", 2),
        ("task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）", None,
         f"{whole_stage} task-server/task submit --whole-package-confirmed /tmp/run-scope-sample-confirmed -- bash scripts/crash/crash-amplification.sh", 2),
        ("task-server 提交不指定范围的崩溃放量入口、带 --full-gate：放", None,
         f"{whole_stage} task-server/task submit --full-gate -- bash scripts/crash/crash-amplification.sh", 0),
        ("task-server 提交点名格的提交时门禁：放（不是不指定范围）", None,
         f"{scope_text('items', '门禁 lkmm 一格', '1 格')} task-server/task submit -- bash .claude/gate.d/lkmm.sh --check sample-cell", 0),
        ("直接起的不指定范围的提交时门禁：这一条不判（直接跑由 project-code-guard.sh 拒）", None, f"{whole_stage} bash .claude/gate.d/lkmm.sh", 0),
        # 指定范围的崩溃域：按登记表与测试源码判点到的目标或项会不会走崩溃枚举、读哪一个缩域变量；设了读的那一个（多设一个也行）的放，
        #   只设了它不读的那一个的拒（样本树里的登记见 build_selftest_tree）
        ("崩溃点重放点名会走崩溃枚举的项、没缩域：拒", None, f"{replay_declaration} {replay_stream}", 2),
        ("崩溃点重放点名放量分项的项、没缩域：拒", None, f"{replay_declaration} bash scripts/crash/crash-point-replay.sh --item sample-amplified-item", 2),
        ("崩溃点重放点名不走崩溃枚举的项：放", None, f"{replay_declaration} bash scripts/crash/crash-point-replay.sh --item sample-plain-item", 0),
        ("崩溃点重放点名会走崩溃枚举的项、带 --crash-expand-up-to：放", None, f"{replay_declaration} {replay_stream} {CRASH_SHRINK_OPTION} 3", 0),
        ("崩溃点重放点名会走崩溃枚举的项、命令前带放量缩域变量：放", None, f"{replay_declaration} {amplification_variable}=4 {replay_stream}", 0),
        ("崩溃点重放点名会走崩溃枚举的项、命令前只带判器缩域变量：拒（入口读的是放量那个）", None, f"{replay_declaration} {judge_variable}=4 {replay_stream}", 2),
        ("task-server 提交崩溃点重放点名项、-- 之后 env 带缩域变量：放", None,
         f"{replay_declaration} task-server/task submit -- env {CRASH_SHRINK_VARIABLES[0]}=4 {replay_stream}", 0),
        ("崩溃点重放点名会走崩溃枚举的项、为什么里写了全域理由：放", None,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项', '全域理由：这一项刚登记，库里还没有判定，要全域判一遍才有基线')} {replay_stream}", 0),
        ("崩溃点重放点名会走崩溃枚举的项、全域理由太短：拒", None,
         f"{scope_text('items', '崩溃点重放点名一项', '1 项', '这一项刚登记看一遍是不是转绿，全域理由：略')} {replay_stream}", 2),
        ("task-server 提交带 --full-gate、崩溃点重放点名项没缩域：这一条不判，放", None,
         f"{replay_declaration} task-server/task submit --full-gate -- {replay_stream}", 0),
        ("cargo test 点名登记过的崩溃枚举目标、没缩域：拒", None,
         f"{target_declaration} cargo test -p {VERIFICATION_PACKAGE} --test crash_enumeration_sample_stream", 2),
        ("cargo test 点名源码里读缩域变量的目标、没缩域：拒", None, f"{target_declaration} {judge_target}", 2),
        ("cargo test 点名不走崩溃枚举的目标：放", None, f"{target_declaration} cargo test -p {VERIFICATION_PACKAGE} --test plain_sample", 0),
        ("task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放", None,
         f"{target_declaration} task-server/task submit -- env {judge_variable}=4 {judge_target} -- sample_judge_case --exact", 0),
        ("cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）", None,
         f"{target_declaration} {amplification_variable}=4 {judge_target}", 2),
        ("cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放", None,
         f"{target_declaration} {amplification_variable}=4 {judge_variable}=4 {judge_target}", 0),
        ("cargo test 点名登记过的放量分项目标、没缩域：拒", None,
         f"{target_declaration} cargo test -p {VERIFICATION_PACKAGE} --test crash_amplification_sample", 2),
        ("cargo test 点名别的包的目标：这一条不判，放", None,
         f"{target_declaration} cargo test -p singlefs-core --test crash_enumeration_sample_stream", 0),
        ("证红点名会走崩溃枚举的变异行、没缩域：拒", None, f"bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-red-crash", 2),
        ("证红点名不走崩溃枚举的变异行：放", None, f"bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-red-plain", 0),
        ("证红点名会走崩溃枚举的变异行、命令前带放量缩域变量：放", None,
         f"{amplification_variable}=4 bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-red-crash", 0),
        ("变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒", None,
         f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash .", 2),
        ("变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒", None, f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item 2 .", 2),
        ("变异门禁点名不走崩溃枚举的行：放", None, f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-plain .", 0),
        ("变异门禁点名会走崩溃枚举的行、命令前带放量缩域变量：放", None,
         f"{mutation_declaration} {amplification_variable}=4 bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash .", 0),
        # 命令点明的副本（样本树里的 copy-tree/，见 build_selftest_tree）：变异表、登记表与测试源码先读那棵树的，那棵树里没有的才退回当前目录那棵与项目根；
        #   `$名` 写的树（--copy、submit 的 --cwd）用赋过的字面值展开，包在 submit 里的也认
        ("证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒", None,
         f"bash scripts/mutation/prove-red.sh --copy copy-tree {VERIFICATION_PACKAGE} copy-only-crash", 2),
        ("task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", None,
         f"C=copy-tree; task-server/task submit -- bash scripts/mutation/prove-red.sh --copy $C {VERIFICATION_PACKAGE} copy-only-crash", 2),
        ("task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒", None,
         f"S=copy-tree; {target_declaration} task-server/task submit --cwd $S -- cargo test -p {VERIFICATION_PACKAGE} --test crash_enumeration_copy_only -- copy_only_enumerates --exact", 2),
        ("变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒", None,
         f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item copy-only-crash copy-tree", 2),
        ("变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒", None,
         f"{mutation_declaration} bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash copy-tree", 2),
        # 驱动脚本的路径写成赋过字面值的变量（`D=<目录>; bash $D/x.sh`）：展开之后读进去判（script_path_word），包在 submit 里的同样
        ("驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", None, f"D={work}/drivers; bash $D/missing.sh", 2),
        ("驱动脚本路径写成赋过字面值的变量、里面每一行都带声明：放", None, f"D={work}/drivers; bash $D/declared.sh", 0),
        ("task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", None, f"D={work}/drivers; task-server/task submit -- bash $D/missing.sh", 2),
        # 经 submit 交的那一条在命令位置上不是服务认得的分片入口，而它的驱动脚本或 bash -c 里有（sharded_wrapped_entry，判法是 task_server_lib.py 的 sharded_entry）：
        #   服务只看命令位置上的词认分片，套在里面整件不分片。声明都写对，拒的只因为分片入口被套住（样本树里的 sharded-sample.sh 文件头写了分片标记，见 build_selftest_tree）
        ("task-server 提交驱动脚本、里面那一行是带分片标记的入口：拒（套在驱动脚本里不分片）", None, "task-server/task submit -- bash drivers/sharded-wrapped.sh", 2),
        ("task-server 提交 bash -c、里面是带分片标记的入口：拒（套在 bash -c 里不分片）", None,
         f"{sharded_item} task-server/task submit -- bash -c 'bash .claude/gate.d/sharded-sample.sh --item row-a .'", 2),
        ("task-server 提交驱动脚本、里面那一行不是带分片标记的入口：放", None, "task-server/task submit -- bash drivers/unsharded.sh", 0),
        ("task-server 提交命令位置上直接是带分片标记的入口：放", None, f"{sharded_item} task-server/task submit -- bash .claude/gate.d/sharded-sample.sh --item row-a .", 0),
        # 经 submit 交的启动测试与验证要点名（判据是 task_server_lib.py 的 unnamed_run_problem，服务提交时拒 ⑦ 同一个函数）：声明都写对，拒的只因为没点名
        ("task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）", None, f"{two_patches} task-server/task submit -- bash scripts/gate/merge-and-verify.sh p1 p2", 2),
        ("task-server 提交合入后验证带 --item：放", None,
         f"{scope_text('items', '合入后验证点名一项', '1 项', '上次红了这一个测试目标，只重跑它看改后是不是转绿')} "
         "task-server/task submit -- bash scripts/gate/merge-and-verify.sh --skip-apply --item alpha:src/lib.rs p1", 0),
        ("task-server 提交 cargo test 点名目标、不点名测试函数：拒", None, f"{target_declaration} task-server/task submit -- cargo test -p {VERIFICATION_PACKAGE} --test plain_sample", 2),
        ("task-server 提交 cargo test 点名 --exact 的测试函数：放", None,
         f"{scope_text('target', '验证包一条测试函数', '1 个测试')} task-server/task submit -- cargo test -p {VERIFICATION_PACKAGE} --test plain_sample -- plain_check --exact", 0),
        ("task-server 提交证红不点名变异：拒", None, f"task-server/task submit -- bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE}", 2),
        ("task-server 提交证红点名变异：放", None, f"task-server/task submit -- bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-red-plain", 0),
        ("task-server 提交门禁点起测试的格、不带 --item：拒", None,
         f"{scope_text('items', '门禁 crash-point-replay 一格', '1 格')} task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh --check feature-gated-tests .", 2),
        ("task-server 提交门禁只点不起测试的格：放", None,
         f"{scope_text('items', '门禁 crash-point-replay 一格', '1 格')} task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh --check crash-knobs .", 0),
        ("task-server 提交带 --full-gate、证红不点名变异：放（提交时那一趟不判点名）", None,
         f"task-server/task submit --full-gate -- bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE}", 0),
        ("task-server 提交带 --whole-package-confirmed、证红不点名变异：放（用户确认过的整包不判点名）", None,
         f"task-server/task submit --whole-package-confirmed /tmp/run-scope-sample-confirmed -- bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE}", 0),
    ]
    return cells + target_sources.directory_target_selftest_cells(scope_text, VERIFICATION_PACKAGE)


def submit_flag_cells():
    """放宽检查的提交标识：(说明, agent_type（None 是主 agent）, 命令, 该认出的)。该认出的是 submit_flag_summary 的写法：点名的标识，认不出的写「变量」，不该弹窗的是空串。"""
    full, whole = "--full-gate", "--whole-package-confirmed"
    submit = "task-server/task submit"
    return [
        ("提交带 --full-gate：弹窗", None, f"{submit} {full} -- ls", full),
        ("提交带 --whole-package-confirmed：弹窗", None, f"{scope_text('whole', 'cargo test 整包', '整包')} {submit} {whole} /tmp/run-scope-sample-confirmed -- cargo test -p singlefs-verification", whole),
        ("范围声明写在前面：弹窗", None, f"{scope_text('whole', '提交前的整轮门禁', '整轮')} {submit} {full} -- bash scripts/gate/gate-staged.sh", full),
        ("bash -c 里面：弹窗", None, f"bash -c '{submit} {full} -- ls'", full),
        ("&& 连着的后一段：弹窗", None, f"ls && {submit} {full} -- ls", full),
        ("--cwd 在标识前面：弹窗", None, f"{submit} --cwd /tmp/run-scope-sample {full} -- ls", full),
        ("两个标识都带：都点名", None, f"{submit} {full} {whole} /tmp/run-scope-sample-confirmed -- ls", f"{full} {whole}"),
        ("python3 起客户端：弹窗", None, f"python3 {submit} {full} -- ls", full),
        ("cd 进 task-server 再起：弹窗", None, f"cd task-server && ./task submit {full} -- ls", full),
        ("cd 到算不出的目录再起：弹窗", None, f'cd "$SAMPLE_UNKNOWN" && {submit} {full} -- ls', full),
        ("cd 到算不出的目录再起 ./task：弹窗", None, f'cd "$SAMPLE_UNKNOWN" && ./task submit {full} -- ls', full),
        ("eval 字面代码：弹窗", None, f"eval '{submit} {full} -- ls'", full),
        ("赋过字面值的变量写在参数上：认出标识", None, f"sample_flag={full}; {submit} $sample_flag -- ls", full),
        ("数组写在参数上：认出标识", None, f"sample_options=({full}); {submit} $sample_options -- ls", full),
        ("标识写成等号带值：弹窗", None, f"{submit} {whole}=/tmp/run-scope-sample-confirmed -- ls", whole),
        ("参数是变量：弹窗认不出", None, f"{submit} $sample_options -- ls", "变量"),
        ("数组全部元素写在参数上：弹窗认不出", None, f'sample_options=(--manual); {submit} "${{sample_options[@]}}" -- ls', "变量"),
        ("eval 给出的参数是变量：弹窗认不出", None, f'eval "{submit} $sample_options -- ls"', "变量"),
        ("子命令位是变量：弹窗认不出", None, "task-server/task $sample_subcommand --cwd /tmp/run-scope-sample -- ls", "变量"),
        ("参数是命令替换：弹窗认不出", None, f"{submit} $(cat sample-options) -- ls", "变量"),
        ("xargs 读进来的参数：弹窗认不出", None, f"printf x | xargs {submit}", "变量"),
        ("flock 包着：弹窗", None, f"flock /tmp/run-scope-sample.lock {submit} {full} -- ls", full),
        ("python3 -c 里字面写着：弹窗", None, f"""python3 -c "import subprocess; subprocess.run(['task-server/task', 'submit', '{full}', '--', 'ls'])\"""", full),
        ("驱动脚本里：弹窗", None, "bash drivers/flagged.sh", full),
        ("交给服务的那一条里又提交：弹窗", None, f"{submit} --manual -- bash -c '{submit} {full} -- ls'", full),
        ("脚本里的转发函数：弹窗", None, f'sample_forward() {{ "$@"; }}; sample_forward {submit} {full} -- ls', full),
        ("命令词是赋过字面值的变量：弹窗", None, f"sample_command='{submit} {full} -- ls'; $sample_command", full),
        ("eval 赋过字面值的变量：弹窗", None, f"""sample_command='{submit} {full} -- ls'; eval "$sample_command\"""", full),
        ("不带标识的提交：不弹窗", None, f"{submit} -- ls", ""),
        ("--manual 提交：不弹窗", None, f"{submit} --manual -- ls", ""),
        ("grep 这个词：不弹窗", None, f"grep -- {full} sample-file", ""),
        ("commit message 带这个词：不弹窗", None, f"git commit -m '提到 {full} 这个词'", ""),
        ("--why 里提到这个词：不弹窗", None, f"task-server/task confirm 7 --why '为什么提到 {full} 与 {whole}'", ""),
        ("交给服务的那一条里提到这个词：不弹窗", None, f"{submit} -- grep -- {full} sample-file", ""),
        ("交给服务的那一条参数是变量：不弹窗", None, f'{submit} -- ls "$sample_path"', ""),
        ("xargs 读进来的落在交给服务的那一条里：不弹窗", None, f"printf x | xargs {submit} -- ls", ""),
        ("echo 这条命令的文字：不弹窗", None, f"echo '{submit} {full} -- ls'", ""),
        ("范围声明的值里提到这个词：不弹窗", None, f"SINGLEFS_RUN_SCOPE='提到 {full}' {submit} -- ls", ""),
        ("子 agent 带标识：不弹窗（由项目代码闸拒）", "implementation-writer", f"{submit} {full} -- ls", ""),
    ]


def submit_flag_summary(asks):
    """要确认的提交写成一个串给自证格比：点名的标识（照 LOOSENING_SUBMIT_FLAGS 的次序）接认不出的「变量」，空格连，没有要确认的是空串。"""
    named = [flag for flag in LOOSENING_SUBMIT_FLAGS if any(flag in ask.flags for ask in asks)]
    return " ".join(named + (["变量"] if any(ask.unread_words for ask in asks) else []))


def selftest_judge_cells(work):
    """在内存里逐格判；交回 [(说明, 期望, 实际)]。"""
    results = []
    for label, agent_type, command, want in selftest_cells(work):
        hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        try:
            got = decide_scope(hook_input, work).code
        except Exception as error:
            got = f"抛了 {error!r}"
        results.append((label, want, got))
    for label, agent_type, command, want in submit_flag_cells():
        hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        try:
            got = submit_flag_summary(decide_scope(hook_input, work).flag_asks)
        except Exception as error:
            got = f"抛了 {error!r}"
        results.append((label, want, got))
    # 判放宽标识这一段抛了不牵连范围声明：把判法换成一抛就停的，这条命令的 decide_scope 照常交回
    def raising_walker(*_arguments, **_options):
        raise RuntimeError("自证注入的异常")
    real_walker = lib_run_scope_core.submit_flag_asks_in_text
    lib_run_scope_core.submit_flag_asks_in_text = raising_walker
    try:
        kept = decide_scope({"tool_name": "Bash", "cwd": work, "tool_input": {"command": "task-server/task submit --full-gate -- ls"}}, work)
        got = (kept.code, submit_flag_summary(kept.flag_asks), bool(kept.unjudged))
    except Exception as error:
        got = f"抛了 {error!r}"
    finally:
        lib_run_scope_core.submit_flag_asks_in_text = real_walker
    results.append(("判放宽标识时抛异常：不牵连范围声明、提到标识的照弹窗、记一处判不了", (0, "--full-gate", True), got))
    # 带一个值的选项照两份驱动的用法行（CRASH_DRIVER_OPTIONS_WITH_VALUE）：--skip 的值不当测试目标
    skipped = run_shape_of(["scripts/crash/crash-amplification-two-hosts.sh", "--skip", "crash_amplification_slow_case", "crash_amplification_fixed_script"], work)
    results.append(("崩溃放量双机驱动：--skip 的值不当测试目标", "崩溃放量双机驱动起测试目标 crash_amplification_fixed_script", skipped.what if skipped else None))

    def decided(command):
        try:
            return decide_scope({"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}, work)
        except Exception as error:
            return f"抛了 {error!r}"
    # 提交时才许不指定范围地跑的六道门禁逐道各判一遍（登记在 COMMIT_GATE_STAGE_NAMES）
    whole_declaration = scope_text("whole", "提交时的某一道门禁全部格", "整轮")
    six_codes = [getattr(decided(f"{whole_declaration} task-server/task submit -- bash .claude/gate.d/{name}"), "code", "抛了异常") for name in COMMIT_GATE_STAGE_NAMES]
    results.append(("task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒", [2] * len(COMMIT_GATE_STAGE_NAMES), six_codes))
    # 崩溃域闸点到的项或变异名不在登记里：放行，判不出的那一处记进判不了的清单（进检出记录的「范围声明闸判不了」）
    for label, command, name in (
            ("崩溃点重放点名的项不在合表里：放行、记一处判不了", f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/crash-point-replay.sh --item sample-missing-item", "sample-missing-item"),
            ("证红点名的变异名不在变异表里：放行、记一处判不了", f"bash scripts/mutation/prove-red.sh --copy copy {VERIFICATION_PACKAGE} sample-missing-row", "sample-missing-row")):
        decision = decided(command)
        results.append((label, (0, True), (decision.code, any(name in entry for entry in decision.unjudged)) if not isinstance(decision, str) else decision))
    # 拒绝说明的出路点对缩域变量（target_shrink_variables）：登记过的崩溃枚举目标的变异行点放量那个，源码里只写了判器那个变量名的目标点判器那个。
    #   (退出码, 出路里有没有「放量那个=<n>」, 有没有「判器那个=<n>」)
    def way_out_names(decision):
        if isinstance(decision, str):
            return decision
        message = decision.message or ""
        return (decision.code, f"{CRASH_SHRINK_VARIABLES[0]}=<n>" in message, f"{CRASH_SHRINK_VARIABLES[1]}=<n>" in message)
    mutation_way_out = decided(f"{scope_text('items', '变异门禁点名一行', '1 项')} bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash .")
    results.append(("变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量", (2, True, False), way_out_names(mutation_way_out)))
    cargo_way_out = decided(f"{scope_text('target', '验证包一个测试目标', '1 个测试目标')} cargo test -p {VERIFICATION_PACKAGE} --test crash_judge_sample_every_state")
    results.append(("cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量", (2, False, True), way_out_names(cargo_way_out)))
    # 设的缩域变量不是点到的目标读的那一个：拒，说明点名该设的那一个（出路里只有「放量那个=<n>」）
    wrong_variable = decided(f"{scope_text('items', '变异门禁点名一行', '1 项')} {CRASH_SHRINK_VARIABLES[1]}=4 bash .claude/gate.d/crates-mutation-replay.sh --item sample-red-crash .")
    results.append(("变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个", (2, True, False), way_out_names(wrong_variable)))
    # 判不出点到的目标读哪一个缩域变量（登记过、tests/<目标>.rs 与 tests/<目标>/main.rs 都不在）：设了两个之一照放行，那一处记进判不了的清单
    undetermined = decided(f"{scope_text('target', '验证包一个测试目标', '1 个测试目标')} {CRASH_SHRINK_VARIABLES[1]}=4 "
                           f"cargo test -p {VERIFICATION_PACKAGE} --test crash_enumeration_sample_without_source")
    results.append(("cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了", (0, True),
                     (undetermined.code, any("crash_enumeration_sample_without_source" in entry and "读哪一个缩域变量" in entry for entry in undetermined.unjudged))
                     if not isinstance(undetermined, str) else undetermined))
    # 命令点明的副本里与项目根都找不到的变异名：放行，记的判不了里写读过哪几棵树；--copy 写成展开不出值的变量：记的判不了点名那个变量
    def unjudged_mentions(decision, *texts):
        if isinstance(decision, str):
            return decision
        return (decision.code, any(all(text in entry for text in texts) for entry in decision.unjudged))
    results.append(("证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树", (0, True),
                    unjudged_mentions(decided(f"bash scripts/mutation/prove-red.sh --copy copy-tree {VERIFICATION_PACKAGE} sample-missing-row"), "sample-missing-row", "copy-tree")))
    results.append(("证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量", (0, True),
                    unjudged_mentions(decided(f"bash scripts/mutation/prove-red.sh --copy $UNSET_TREE {VERIFICATION_PACKAGE} sample-missing-row"), "$UNSET_TREE", "展开不出")))
    # 判分片的 task_server_lib.py 载不进（把载入结果换成载不进、判完放回）：放行，判不了的那一处记进清单（弄坏开关 sharded-wrapped-unjudged 下没有这一处记录）
    saved_library = list(lib_run_scope_core.project_code_runs.TASK_SERVER_LIBRARY_LOADED)
    lib_run_scope_core.project_code_runs.TASK_SERVER_LIBRARY_LOADED[:] = [(None, "自证注入的载不进")]
    try:
        unloaded = decided(f"{scope_text('items', '带分片标记的门禁点名一项', '1 项')} task-server/task submit -- bash -c 'bash .claude/gate.d/sharded-sample.sh --item row-a .'")
    finally:
        lib_run_scope_core.project_code_runs.TASK_SERVER_LIBRARY_LOADED[:] = saved_library
    results.append(("task-server 提交 bash -c 里带分片标记的入口、task_server_lib 载不进：放行、记一处判不了", (0, True),
                    (unloaded.code, any("分片入口套没套在驱动脚本或 bash -c 里判不了" in entry and "自证注入的载不进" in entry for entry in unloaded.unjudged))
                    if not isinstance(unloaded, str) else unloaded))
    # 范围对不上的说明写这条命令该写的整段「范围=<数> <单位>」（数与单位都是闸自己数的那一个）与数法；命令里有变量拼的词时另写「把 --item 写成字面」。
    #   断言找带引号的「范围=…」：拒绝文本末尾的声明模板行也打不带引号的 范围=…，那一处不经 declaration_to_write，不能当说明里写了这一句的凭据
    def refusal_mentions(decision, *texts):
        if isinstance(decision, str):
            return decision
        return (decision.code, all(text in (decision.message or "") for text in texts))
    results.append(("范围数对不上：一格加两项数成 3 项，说明写整段 范围=3 项 与格项相加的数法", (2, True),
                    refusal_mentions(decided(f"{scope_text('items', '门禁 code-tooling 一格加两项', '2 项')} "
                                             "bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item first-item --item second-item ."), "「范围=3 项」", "各自去重后相加")))
    results.append(("范围数对不上：实验复跑点一个实验写成 2 行，说明写整段 范围=1 个实验", (2, True),
                    refusal_mentions(decided(f"{scope_text('items', '复跑 E207', '2 行')} bash scripts/experiment-apparatus/replay.sh E207"), "「范围=1 个实验」")))
    results.append(("声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面", (2, True),
                    refusal_mentions(decided(f"items='--item a --item b'; {scope_text('items', '门禁 code-tooling 两项', '2 项')} bash .claude/gate.d/code-tooling.sh $items ."),
                                     "「范围=整轮」", "把 --item 写成字面", "$items")))
    return results


def selftest_entry_cells(work, guard_script):
    """另起入口（run-scope-guard.sh）喂 hook 的 JSON：退出码、stderr 的说明、检出记录与上次红的清单。交回 [(说明, 期望, 实际)]。"""
    results = []
    detections = os.path.join(work, "detections.jsonl")
    last_runs = os.path.join(work, "last-runs")
    os.makedirs(last_runs, exist_ok=True)
    with open(os.path.join(last_runs, ".claude__gate.d__doc-text.sh.tsv"), "w", encoding="utf-8") as handle:
        handle.write("# 上次这一道红的\nclock-times\nprime-marks\n")
    environment = dict(os.environ, AGENT_HOOK_DETECTIONS=detections, SINGLEFS_LAST_RUNS_DIRECTORY=last_runs, CLAUDE_PROJECT_DIR=work)

    def feed(command, agent_type=None, session="s-run-scope"):
        hook_input = {"tool_name": "Bash", "cwd": work, "session_id": session, "tool_input": {"command": command}}
        if agent_type:
            hook_input["agent_type"] = agent_type
        return subprocess.run(["bash", guard_script], input=json.dumps(hook_input), capture_output=True, text=True, env=environment, timeout=120)

    refused = feed("bash .claude/gate.d/doc-text.sh .")
    results.append(("入口：没带声明拒（退出码 2）", 2, refused.returncode))
    template = f"{DECLARATION_VARIABLE}='种类=whole; 跑什么=门禁 doc-text.sh 的全部格（不带 --check / --item）; 范围=整轮; 为什么=<"
    results.append(("入口：拒绝说明写跑的是什么、照抄模板与只跑红的写法", True,
                    all(text in refused.stderr for text in ("跑的是：门禁 doc-text.sh 的全部格", template, "--check <格>", "--exact", "rerun-failed-tests.py",
                                                             "挑测试目标在脚本里做"))))
    results.append(("入口：拒绝说明附上次红的清单", True, "clock-times、prime-marks" in refused.stderr))
    no_record = feed("bash .claude/gate.d/doc-experiments.sh .")
    results.append(("入口：没有上次红的记录时照写", True, "没有上次红的记录" in no_record.stderr))
    sub_submitted = feed(f"{scope_text('whole', '门禁 doc-text 全部格', '整轮')} task-server/task submit -- bash .claude/gate.d/doc-text.sh .", "implementation-writer")
    results.append(("入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回", (2, True),
                    (sub_submitted.returncode, "写进报告「要主 agent 跑的命令」一节交回" in sub_submitted.stderr)))
    template_lines = [line for line in sub_submitted.stderr.splitlines() if line.startswith(f"  {DECLARATION_VARIABLE}=")]
    results.append(("入口：命令开头已有声明时模板只留一份", [1], [line.count(f"{DECLARATION_VARIABLE}=") for line in template_lines]))
    if os.path.exists(detections):
        os.remove(detections)
    allowed_command = f"{scope_text('items', '门禁 code-tooling 两格', '2 格')} bash .claude/gate.d/code-tooling.sh --check agent-write-scope,research-script-selftests ."
    allowed = feed(allowed_command)
    entries = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            entries = [json.loads(line) for line in handle if line.strip()]
    recorded = [entry for entry in entries if entry.get("outcome") == "放行"]
    good_record = (len(recorded) == 1 and "session_id" not in recorded[0] and recorded[0].get("scope_session_id") == "s-run-scope"
                   and recorded[0]["run_scope"]["segments"].get("范围") == "2 格" and recorded[0]["run_scope"]["kind_of_command"] == "items"
                   and recorded[0]["command"] == allowed_command)
    results.append(("入口：放行的记进检出记录（四段与命令）", (0, True), (allowed.returncode, good_record)))
    feed("bash .claude/gate.d/doc-text.sh .")
    refused_entries = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            refused_entries = [json.loads(line) for line in handle if line.strip() and json.loads(line).get("outcome") == "被拒"]
    results.append(("入口：被拒的记一行（session_id、范围声明被拒、outcome 被拒）", True,
                    len(refused_entries) == 1 and refused_entries[0].get("session_id") == "s-run-scope"
                    and refused_entries[0]["findings"][0].startswith("范围声明被拒")))
    plain = feed("ls -la")
    results.append(("入口：别的命令放行、不写 stderr", (0, ""), (plain.returncode, plain.stderr)))
    # 放宽检查的提交标识：标准输出整份是一个 JSON 对象（Claude Code 才认得），退 0、permissionDecision 是 ask，理由给用户看
    def asked_reason(run):
        try:
            output = json.loads(run.stdout)["hookSpecificOutput"]
        except (ValueError, KeyError, TypeError):
            return None
        return output.get("permissionDecisionReason") if output.get("permissionDecision") == "ask" and output.get("hookEventName") == "PreToolUse" else None
    flagged_command = f"{scope_text('whole', '提交前的整轮门禁', '整轮')} task-server/task submit --full-gate -- bash scripts/gate/gate-staged.sh"
    flagged = feed(flagged_command)
    flagged_reason = asked_reason(flagged) or ""
    results.append(("入口：提交带 --full-gate 弹窗（退 0、ask、理由写标识、作用、命令全文与不许自己用）", (0, True),
                    (flagged.returncode, all(text in flagged_reason for text in ("--full-gate", LOOSENING_SUBMIT_FLAGS["--full-gate"], f"命令全文：{flagged_command}", SUBMIT_FLAG_USER_RULE)))))
    unread = feed("task-server/task submit $sample_options -- ls")
    results.append(("入口：参数是变量弹窗、理由写认不出带没带标识", (0, True), (unread.returncode, "参数里有变量，认不出带没带标识" in (asked_reason(unread) or ""))))
    unflagged = feed("task-server/task submit -- ls")
    results.append(("入口：不带标识的提交不弹窗（标准输出是空的）", (0, ""), (unflagged.returncode, unflagged.stdout)))
    commit_unflagged = feed(f"{scope_text('whole', '门禁 lkmm 全部格', '整轮')} task-server/task submit -- bash .claude/gate.d/lkmm.sh")
    results.append(("入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）", (2, True, True),
                    (commit_unflagged.returncode, all(text in commit_unflagged.stderr for text in ("commit-stages.md", "--full-gate")),
                     "照抄声明模板" not in commit_unflagged.stderr)))
    # 经 submit 交的 cargo test 只点目标：拒，说明写没点名与改成点名的样子（`-- <测试全名> --exact`），声明写对了不附声明模板；被拒的记一行
    unnamed_command = f"{scope_text('target', '验证包一个测试目标', '1 个测试目标')} task-server/task submit -- cargo test -p {VERIFICATION_PACKAGE} --test plain_sample"
    unnamed = feed(unnamed_command, session="s-unnamed")
    unnamed_entries = []
    if os.path.exists(detections):
        with open(detections, encoding="utf-8") as handle:
            unnamed_entries = [entry for entry in (json.loads(line) for line in handle if line.strip())
                               if entry.get("session_id") == "s-unnamed" and entry.get("outcome") == "被拒"]
    results.append(("入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）", (2, True, True, 1),
                    (unnamed.returncode, all(text in unnamed.stderr for text in ("启动测试与验证没点名", "-- <测试全名> --exact", "→ 怎么办：")),
                     "照抄声明模板" not in unnamed.stderr, len(unnamed_entries))))
    # 经 submit 交的驱动脚本里套着带分片标记的入口：拒，说明点名那份入口与它在驱动脚本里的 脚本:行、写出路原文「把那一行直接交给服务，--item 照字面写进命令」与拼好的那一条提交，
    #   声明都写对了不附声明模板
    wrapped = feed("task-server/task submit -- bash drivers/sharded-wrapped.sh", session="s-wrapped")
    results.append(("入口：交给服务的驱动脚本里套着带分片标记的入口拒（退出码 2、说明点名入口与驱动脚本:行、出路与拼好的提交）", (2, True, True),
                    (wrapped.returncode,
                     all(text in wrapped.stderr for text in ("分片入口 ", "sharded-sample.sh", "sharded-wrapped.sh:2", "把那一行直接交给服务，--item 照字面写进命令：",
                                                              "task-server/task submit --cwd ", "-- bash .claude/gate.d/sharded-sample.sh --item row-a .")),
                     "照抄声明模板" not in wrapped.stderr)))
    domain_unshrunk = feed(f"{scope_text('items', '崩溃点重放点名一项', '1 项')} bash scripts/crash/crash-point-replay.sh --item sample-stream-item")
    results.append(("入口：崩溃域没缩拒（退出码 2、说明写缩域变量与全域理由的写法）", (2, True),
                    (domain_unshrunk.returncode, all(text in domain_unshrunk.stderr for text in (CRASH_SHRINK_OPTION, CRASH_SHRINK_VARIABLES[1], "全域理由：")))))
    # 判不了的照常放行、记一条检出（结局「只记」）：hook 输入不是 JSON 对象（stderr 打带钩子名与开头字节的诊断，会话号写读不到）、
    # 判的时候抛异常（RUN_SCOPE_GUARD_BREAK 里加一个认不出的项，guard_switch_on 抛 ValueError）、here-string 喂给 bash 的是变量。
    # RUN_SCOPE_GUARD_BREAK=failure-unrecorded 下这三格红，LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下第三格红
    def recorded_entries():
        if not os.path.exists(detections):
            return []
        with open(detections, encoding="utf-8") as handle:
            return [json.loads(line) for line in handle if line.strip()]
    before = len(recorded_entries())
    unreadable = [subprocess.run(["bash", guard_script], input=raw, capture_output=True, env=environment, timeout=120)
                  for raw in (b'{"tool_name": "Bash", "tool_input": ', b'["Bash"]')]
    added = recorded_entries()[before:]
    results.append(("入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出", ([0, 0], True, 2, ["读不到", "读不到"]),
                    ([run.returncode for run in unreadable],
                     all(f"{GUARD_SCRIPT_NAME} 收到的 hook 输入不是 JSON 对象" in run.stderr.decode("utf-8", "replace") for run in unreadable)
                     and "b'[\"Bash\"]'" in unreadable[1].stderr.decode("utf-8", "replace"),
                     len(added), [entry.get("session_id") for entry in added])))
    before = len(recorded_entries())
    # 关守卫项 honor-outside-selftest 与别的项同写会被入口的守卫先拒（第一百二十二节 ②），注入认不出的项时摘掉它
    injected = ",".join([item for item in os.environ.get(BREAK_VARIABLE_NAME, "").split(",") if item and item != "honor-outside-selftest"] + ["not-a-switch"])
    raising = subprocess.run(["bash", guard_script], input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-raise",
                             "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}}), capture_output=True, text=True,
                             env=dict(environment, **{BREAK_VARIABLE_NAME: injected}), timeout=120)
    added = recorded_entries()[before:]
    results.append(("入口：判的时候抛异常放行、stderr 报一句、记一条检出", (0, True, 1, "s-raise", "只记"),
                    (raising.returncode, "没判成" in raising.stderr, len(added), added[0].get("session_id") if added else None,
                     added[0].get("outcome") if added else None)))
    raising_flagged = subprocess.run(["bash", guard_script], input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-raise-flagged",
                                     "tool_input": {"command": flagged_command}}), capture_output=True, text=True,
                                     env=dict(environment, **{BREAK_VARIABLE_NAME: injected}), timeout=120)
    results.append(("入口：判的时候抛异常、命令文字里提到标识：弹窗", (0, True), (raising_flagged.returncode, "--full-gate" in (asked_reason(raising_flagged) or ""))))
    before = len(recorded_entries())
    fed_variable = feed('bash <<< "$CMD"', session="s-fed")
    added = [entry for entry in recorded_entries()[before:] if entry.get("session_id") == "s-fed"]
    results.append(("入口：here-string 喂给 bash 的是变量放行、记一条检出", (0, 1, True, "只记"),
                    (fed_variable.returncode, len(added), bool(added) and any("here-string `$CMD`" in finding for finding in added[0].get("findings", [])),
                     added[0].get("outcome") if added else None)))
    # 判法文件读不到：照常放行、报一句、记一条检出（钩子挂在每一条 Bash 上，坏了不许把所有命令都拒掉）
    # 只拷入口与 lib_work_root.py（入口记检出时取默认位置用）；它 source 的 .claude/scripts/preflight.sh 与那一份找的规范副本 .claude/singlefs-ai-sop/ 链接到钩子所在的仓。
    #   入口自己记：AGENT_HOOK_DETECTIONS 设了记那里，没设记进 lib_work_root.py 的默认位置（这里把 SINGLEFS_WORK_ROOT 指进临时目录）。failure-unrecorded 下记检出的两格红
    lonely = os.path.join(work, "lonely", ".claude", "hooks")
    os.makedirs(lonely, exist_ok=True)
    shutil.copy(guard_script, os.path.join(lonely, GUARD_SCRIPT_NAME))
    shutil.copy(os.path.join(os.path.dirname(guard_script), "lib_work_root.py"), os.path.join(lonely, "lib_work_root.py"))   # 入口记检出时取默认位置
    claude_directory = os.path.dirname(os.path.dirname(os.path.abspath(guard_script)))
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(work, "lonely", ".claude", linked)
        if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
            os.symlink(os.path.join(claude_directory, linked), link)
    orphan_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-orphan", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}})
    before = len(recorded_entries())
    orphan = subprocess.run(["bash", os.path.join(lonely, GUARD_SCRIPT_NAME)], input=orphan_input, capture_output=True, text=True, env=environment, timeout=60)
    added = recorded_entries()[before:]
    results.append(("入口：判法文件读不到时放行并报一句", (0, True), (orphan.returncode, "照常执行" in orphan.stderr)))
    results.append(("入口：判法文件读不到时记一条检出（结局只记、会话号照输入）", (1, "s-orphan", "只记", GUARD_SCRIPT_NAME, True),
                    (len(added), added[0].get("session_id") if added else None, added[0].get("outcome") if added else None,
                     added[0].get("hook") if added else None, bool(added) and "读不到判法" in " ".join(added[0].get("findings") or []))))
    default_detections = os.path.join(work, "orphan-work-root", "agent-hook-detections.jsonl")

    def default_entries():
        try:
            with open(default_detections, encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except OSError:
            return []
    before = len(default_entries())
    without_variable = {key: value for key, value in environment.items() if key not in (DETECTIONS_VARIABLE, "SINGLEFS_WORK_ROOT_BREAK")}
    orphan_default = subprocess.run(["bash", os.path.join(lonely, GUARD_SCRIPT_NAME)], input=orphan_input, capture_output=True, text=True, timeout=60,
                                    env=dict(without_variable, SINGLEFS_WORK_ROOT=os.path.dirname(default_detections)))
    added = default_entries()[before:]
    results.append(("入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置", (0, 1, "s-orphan", "只记"),
                    (orphan_default.returncode, len(added), added[0].get("session_id") if added else None, added[0].get("outcome") if added else None)))
    # 判法文件在而没跑完（这里放一份导入就抛异常的 lib_run_scope.py）：入口照常放行、stderr 报一句、自己记一条检出（会话号照输入）；
    #   judge-crash-unrecorded 下这一格红（只打 stderr、不记）
    crashing = os.path.join(work, "crashing", ".claude", "hooks")
    os.makedirs(crashing, exist_ok=True)
    shutil.copy(guard_script, os.path.join(crashing, GUARD_SCRIPT_NAME))
    shutil.copy(os.path.join(os.path.dirname(guard_script), "lib_work_root.py"), os.path.join(crashing, "lib_work_root.py"))
    with open(os.path.join(crashing, "lib_run_scope.py"), "w", encoding="utf-8") as handle:
        handle.write("raise RuntimeError('自证造的导入错')\n")
    for linked in ("scripts", "singlefs-ai-sop"):
        link = os.path.join(work, "crashing", ".claude", linked)
        if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
            os.symlink(os.path.join(claude_directory, linked), link)
    before = len(recorded_entries())
    crashed = subprocess.run(["bash", os.path.join(crashing, GUARD_SCRIPT_NAME)],
                             input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-crash", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}}),
                             capture_output=True, text=True, env=environment, timeout=60)
    added = recorded_entries()[before:]
    results.append(("入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）", (0, True, 1, "s-crash", "只记", True),
                    (crashed.returncode, "没跑完" in crashed.stderr, len(added), added[0].get("session_id") if added else None,
                     added[0].get("outcome") if added else None, bool(added) and "没跑完" in " ".join(added[0].get("findings") or []))))
    # 判法的部分文件读不到、编不过（拷一份判法目录，删掉或写坏部分文件）：入口照常放行、stderr 点名那一份、判法退 70、记一条检出；
    #   按文件路径导入 lib_run_scope.py（调用方的导入法，起子进程）抛 PartFileUnreadable、说明点名那一份；part-unreadable-passes 下判法照原样抛读文件错与语法错，读不到与编不过的两格红；
    #   编得过而被截空的（入口 import 的名字缺了）同样处置，part-incomplete-passes 下照原样抛 ImportError，截空那一格红
    part_probe_code = ("import importlib.util, sys\n"
                       "spec = importlib.util.spec_from_file_location('lib_run_scope_probe', sys.argv[1])\n"
                       "module = importlib.util.module_from_spec(spec)\n"
                       "try:\n"
                       "    spec.loader.exec_module(module)\n"
                       "except Exception as error:\n"
                       "    print(type(error).__name__ + '\\t' + str(error))\n"
                       "else:\n"
                       "    print('loaded\\t')\n")
    for damage, damage_label in (("missing", "读不到"), ("syntax-error", "编不过"), ("empty", "截空")):
        damaged = os.path.join(work, f"damaged-part-{damage}", ".claude", "hooks")
        os.makedirs(damaged, exist_ok=True)
        for copied_name in (GUARD_SCRIPT_NAME, "lib_work_root.py", "lib_run_scope.py", "lib_run_scope_target_sources.py", RUN_SCOPE_BOOTSTRAP_FILE_NAME) + RUN_SCOPE_PART_NAMES:
            shutil.copy(os.path.join(os.path.dirname(guard_script), copied_name), damaged)
        damaged_part = os.path.join(damaged, RUN_SCOPE_PART_NAMES[0])
        if damage == "missing":
            os.remove(damaged_part)
        else:
            with open(damaged_part, "w", encoding="utf-8") as handle:
                handle.write("def broken(:\n" if damage == "syntax-error" else "")
        for linked in ("scripts", "singlefs-ai-sop"):
            link = os.path.join(work, f"damaged-part-{damage}", ".claude", linked)
            if os.path.exists(os.path.join(claude_directory, linked)) and not os.path.lexists(link):
                os.symlink(os.path.join(claude_directory, linked), link)
        before = len(recorded_entries())
        damaged_run = subprocess.run(["bash", os.path.join(damaged, GUARD_SCRIPT_NAME)],
                                     input=json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-damaged-part", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}}),
                                     capture_output=True, text=True, env=environment, timeout=60)
        added = recorded_entries()[before:]
        probe = subprocess.run([sys.executable, "-B", "-c", part_probe_code, os.path.join(damaged, "lib_run_scope.py")], capture_output=True, text=True, timeout=60,
                               env=environment)
        probe_lines = probe.stdout.strip().splitlines()
        probe_name, _separator, probe_text = (probe_lines[-1] if probe_lines else "").partition("\t")
        results.append((f"入口：判法的部分文件{damage_label}：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable",
                        (0, True, True, 1, "PartFileUnreadable", True),
                        (damaged_run.returncode, f"部分文件 {damaged_part}" in damaged_run.stderr, f"退 {PART_UNREADABLE_STATUS}）" in damaged_run.stderr, len(added),
                         probe_name, f"部分文件 {damaged_part}" in probe_text)))
    # 驱动脚本在而读不了（chmod 成 000）：照常放行、这一条命令记一行检出（「范围声明闸判不了」起头）；driver-unread-unrecorded 下这一格红
    locked_driver = os.path.join(work, "locked-driver.sh")
    with open(locked_driver, "w", encoding="utf-8") as handle:
        handle.write("#!/usr/bin/env bash\nbash .claude/gate.d/doc-text.sh .\n")
    os.chmod(locked_driver, 0)
    before = len(recorded_entries())
    try:
        locked_run = feed("bash ./locked-driver.sh", session="s-locked")
    finally:
        os.chmod(locked_driver, 0o600)
    added = [entry for entry in recorded_entries()[before:] if entry.get("session_id") == "s-locked"]
    results.append(("入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）", (0, 1, "只记", True),
                    (locked_run.returncode, len(added), added[0].get("outcome") if added else None,
                     bool(added) and any(str(finding).startswith("范围声明闸判不了") and "读不了" in str(finding) for finding in added[0].get("findings") or []))))
    # 弄坏开关只在自证里认：不带标记、或标记文件内容对不上的平常跑设了开关，入口在读 hook 的 JSON 之前退 2、stderr 点名它只在自证里认（探法同门禁
    #   code-tooling 的 break-switch-outside-selftest 格）；带标记的自证子进程照认开关。honor-outside-selftest 设着时探的开关里也带上它，前两格红。检出写另一份，不搅前面几格的数
    honored = "honor-outside-selftest" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")
    probe_item = "honor-outside-selftest" if honored else "declaration-optional"
    unmarked = {key: value for key, value in environment.items() if key != SELFTEST_MARK_VARIABLE}
    unjudged = os.path.join(work, "outside-selftest-detections.jsonl")
    plain = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                           env=dict(unmarked, **{BREAK_VARIABLE_NAME: probe_item, DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：平常跑设了弄坏开关退 2、点名它只在自证里认", (2, True),
                    (plain.returncode, any(BREAK_VARIABLE_NAME in line and "自证" in line for line in plain.stderr.splitlines()))))
    forged_mark = os.path.join(work, "forged-selftest-mark")
    with open(forged_mark, "w", encoding="utf-8") as handle:
        handle.write("别处写的标记\n")
    plain_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-outside", "tool_input": {"command": "ls -la"}})
    forged = subprocess.run(["bash", guard_script], input=plain_input, capture_output=True, text=True, timeout=120,
                            env=dict(unmarked, **{BREAK_VARIABLE_NAME: probe_item, DETECTIONS_VARIABLE: unjudged, SELFTEST_MARK_VARIABLE: forged_mark}))
    results.append(("入口：标记文件内容对不上照拒", 2, forged.returncode))
    marked_input = json.dumps({"tool_name": "Bash", "cwd": work, "session_id": "s-outside", "tool_input": {"command": "bash .claude/gate.d/doc-text.sh ."}})
    marked = subprocess.run(["bash", guard_script], input=marked_input, capture_output=True, text=True, timeout=120,
                            env=dict(environment, **{BREAK_VARIABLE_NAME: "declaration-optional", DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：自证起的子进程（带标记）照认弄坏开关", 0, marked.returncode))
    # 关守卫项 honor-outside-selftest 只在单独一项时认（records/2026-10-23-subagent拆分提案.md 第一百二十二节 ②）：与别的项同写，平常跑与带标记的子进程都退 2、点名它；
    #   honor-outside-selftest-with-others 设着时带标记那一格的开关里也带上它（同写照认），那一格红
    combined_item = "honor-outside-selftest,declaration-optional"
    combined_plain = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                                    env=dict(unmarked, **{BREAK_VARIABLE_NAME: combined_item, DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：关守卫项与别的项同写：平常跑退 2、点名它只在单独一项时认", (2, True),
                    (combined_plain.returncode, "honor-outside-selftest" in combined_plain.stderr and "唯一一项" in combined_plain.stderr)))
    lenient = "honor-outside-selftest-with-others" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")
    combined_marked = subprocess.run(["bash", guard_script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True, timeout=120,
                                     env=dict(environment, **{BREAK_VARIABLE_NAME: combined_item + (",honor-outside-selftest-with-others" if lenient else ""),
                                                              DETECTIONS_VARIABLE: unjudged}))
    results.append(("入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它", (2, True),
                    (combined_marked.returncode, "honor-outside-selftest" in combined_marked.stderr and "唯一一项" in combined_marked.stderr)))
    return results


def failed_labels(results):
    return [label for label, want, got in results if want != got]


def selftest(hook_directory):
    load_judging_libraries()
    guard_script = os.path.join(hook_directory, GUARD_SCRIPT_NAME)
    work = tempfile.mkdtemp(prefix="run-scope-guard-")
    # 外面设了弄坏开关：照它判一遍、点名的格应当红，不跑开关轮（开关轮自己逐个设）
    outside_break = os.environ.get(BREAK_VARIABLE_NAME) or None
    try:
        build_selftest_tree(work)
        # 自证起的入口带这个标记才认弄坏开关（入口的守卫；不带标记的平常跑设了开关退 2）
        selftest_mark = os.path.join(work, "selftest-mark")
        with open(selftest_mark, "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_MARK_CONTENT + "\n")
        os.environ[SELFTEST_MARK_VARIABLE] = selftest_mark
        results = selftest_judge_cells(work) + selftest_entry_cells(work, guard_script)
        failures = failed_labels(results)
        for label, want, got in results:
            if want != got:
                print(f"  ✗ 自检：{label}：应当是 {want!r}，实际 {got!r}")
                print(f"  → 怎么办：照 {label} 那一格的判法改 lib_run_scope.py，再跑 bash .claude/hooks/run-scope-guard.sh --selftest")
        if outside_break:
            print(f"  （{BREAK_VARIABLE_NAME}={outside_break} 下判了 {len(results)} 格，红 {len(failures)} 格；开关轮不跑）")
            return 1 if failures else 0
        labels = [label for label, _want, _got in results]
        try:
            with open(guard_script, encoding="utf-8") as handle:
                header = handle.read()
        except OSError:
            header = ""
        switch_problems = []
        for switch, cells in BREAK_SWITCH_CELLS.items():
            missing = [cell for cell in cells if cell not in labels]
            if missing:
                switch_problems.append(f"开关 {switch} 点名的格不在自证里：{'、'.join(missing)}")
                continue
            if not re.search(r"^#\s+" + re.escape(f"{BREAK_VARIABLE_NAME}={switch}") + r"\s", header, re.M):
                switch_problems.append(f"开关 {switch} 没写进 {GUARD_SCRIPT_NAME} 的文件头（一行「{BREAK_VARIABLE_NAME}={switch}  <弄坏哪一处>」，下一行起写红的格）")
            os.environ[BREAK_VARIABLE_NAME] = switch
            try:
                broken = selftest_judge_cells(work)
                if any(cell.startswith("入口：") for cell in cells):   # 另起入口的几格慢，只在开关点名了它们时再跑
                    broken += selftest_entry_cells(work, guard_script)
            finally:
                os.environ.pop(BREAK_VARIABLE_NAME, None)
            reds = failed_labels(broken)
            still_green = [cell for cell in cells if cell not in reds]
            if still_green:
                switch_problems.append(f"开关 {switch} 打开后这几格没红：{'、'.join(still_green)}")
            unexpected = [cell for cell in reds if cell not in cells]
            if unexpected:
                switch_problems.append(f"开关 {switch} 打开后另有这几格红、表里没写：{'、'.join(unexpected)}（写进 BREAK_SWITCH_CELLS 与文件头）")
        for problem in switch_problems:
            print(f"  ✗ 自检：{problem}")
        if failures or switch_problems:
            print(f"  → 怎么办：照上面点名的格改 {os.path.join(hook_directory, 'lib_run_scope.py')} 的判法（或 {GUARD_SCRIPT_NAME} 的文件头），再单独跑 "
                  f"bash .claude/hooks/{GUARD_SCRIPT_NAME} --selftest 看它转绿")
            return 1
        memory_count = len([label for label in labels if not label.startswith("入口：")])
        print(f"  ✓ {GUARD_SCRIPT_NAME} 自检通过：{len(results)} 格（内存里判 {memory_count} 格、另起入口 {len(results) - memory_count} 格）；"
              f"开关轮 {len(BREAK_SWITCH_CELLS)} 个开关逐个打开，红的格与表里点名的逐格相等（共 {sum(len(cells) for cells in BREAK_SWITCH_CELLS.values())} 格次）")
        return 0
    finally:
        if outside_break:
            os.environ[BREAK_VARIABLE_NAME] = outside_break
        os.environ.pop(SELFTEST_MARK_VARIABLE, None)
        shutil.rmtree(work, ignore_errors=True)
