"""crates-mutation-rows.py 拆出的一份。
自证的脚手架：样本仓与样本库的造法、格里读调用日志与改文件的小函数。假 cargo 与假 llvm 工具的正文（FAKE_CARGO、FAKE_LLVM_TOOL）在 base。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import contextlib
import hashlib
import io
import os
import re
import shutil
import subprocess
from crates_mutation_rows_base import (MODULE_SOURCE_PATHS, STAGE, STAGE_REPOSITORY, TABLE, admission_module, reuse_checks)


def selftest_write(path, text, executable=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if executable:
        os.chmod(path, 0o755)


def selftest_script_registration_table():
    """本模块所在那一份仓的登记表（.claude/gate.d/stage-inputs.tsv）的路径。"""
    return os.path.join(STAGE_REPOSITORY, admission_module().REGISTRATION_TABLE)


def selftest_registration_table_path(directory):
    """样本树 directory 里登记表的路径（父目录建好）。"""
    path = os.path.join(directory, admission_module().REGISTRATION_TABLE)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    return path


def selftest_copy_registration_table(directory):
    """样本树 directory 里放一份本模块所在那一份仓的登记表（整份）：static-check 读被判那棵树的登记表（文件头「拷贝范围」），样本树带着与真仓同一份拷贝范围与崩溃枚举用例。"""
    shutil.copyfile(selftest_script_registration_table(), selftest_registration_table_path(directory))


def selftest_write_stage_registration_table(directory, extra_text="", copy_ranges_added=(), copy_ranges_dropped=()):
    """样本树 directory 的登记表只放本模块所在那一份仓的登记表里本阶段（STAGE）那一行，再接 extra_text（自证自己的崩溃枚举用例行）；
    那一行的 copy-range= 照 copy_ranges_dropped 去掉、照 copy_ranges_added 补上（写法同登记表里的，目录带斜杠）。"""
    with open(selftest_script_registration_table(), encoding="utf-8") as handle:
        stage_rows = [line for line in handle.read().split("\n") if line.startswith(STAGE + "\t")]
    assert len(stage_rows) == 1, stage_rows
    columns = stage_rows[0].split("\t")
    conditions = [token for column in columns[2:] if not column.lstrip().startswith("#") for token in column.split()]
    dropped = {"copy-range=" + path for path in copy_ranges_dropped}
    conditions = [token for token in conditions if token not in dropped] + ["copy-range=" + path for path in copy_ranges_added]
    selftest_write(selftest_registration_table_path(directory), "\t".join(columns[:2] + [" ".join(conditions)]) + "\n" + extra_text)


def build_selftest_repository(directory, table_rows, extra_source="", source=None):
    """一棵临时 git 仓：Cargo.toml、crates/tiny、变异表、仓根下拷贝范围之外的 fixture-data/answer.txt，与本模块所在那一份仓的登记表（selftest_copy_registration_table）。
    source 不给取 SELFTEST_SOURCE（缺省值不写成那个名字：def 头在 import 时求值，而夹具只在 --selftest 里载入，文件头「自证夹具」）。"""
    source = SELFTEST_SOURCE if source is None else source
    selftest_write(os.path.join(directory, "Cargo.toml"), '[workspace]\nmembers = ["crates/tiny"]\n')
    selftest_write(os.path.join(directory, "crates", "tiny", "Cargo.toml"), '[package]\nname = "tiny"\nversion = "0.1.0"\n')
    selftest_write(os.path.join(directory, "crates", "tiny", "src", "lib.rs"), source + extra_source)
    selftest_write(os.path.join(directory, TABLE), "# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名\n" + "".join(row + "\n" for row in table_rows))
    selftest_write(os.path.join(directory, "fixture-data", "answer.txt"), "42\n")
    selftest_copy_registration_table(directory)
    subprocess.run(["git", "init", "-q", directory], check=True)


def selftest_new_library_copy(work, repository, name):
    """repository 整份（连同它自己的 .git）拷到 work/<name>：git common-dir 不同，判定库在临时 prune-server 里的命名空间是新的一份（文件头「判定库」），
    那一份里一条记录都没有，跑一趟就是全部现判（基线、覆盖也现跑）。要「同一份输入再现判一趟」的格用它换一棵树，库没有「不读」这一路。"""
    copy = os.path.join(work, name)
    shutil.copytree(repository, copy, symlinks=True)
    return copy


def selftest_one_forced(row_count):
    """复用哨兵取 row_count - 1 条、剩下那一条强制现判（SINGLEFS_REUSE_SENTINEL_COUNT、SINGLEFS_REUSE_SENTINEL_SEED，lib_reuse_checks.py 文件头「二」）：
    库里 row_count 行都是抓到、键相同的一趟里，只让一行经哨兵的强制现判路径不读库、其余照复用。同键现判、记红盖绿的几格用它。"""
    return {reuse_checks.SENTINEL_COUNT_VARIABLE: str(row_count - 1), reuse_checks.SENTINEL_SEED_VARIABLE: "5"}


def selftest_forced_row_name(run):
    """一趟 selftest.run 的结果里，复用哨兵抽中强制现判的那一行的变异名（stderr 里「强制现判 1 条：<行号> <变异名>」）；没抽出交 None。"""
    match = re.search(r"强制现判 1 条：\d+ (.+)$", run[2], re.M)
    return match.group(1) if match else None


def plain_invocations(invocations):
    """假 cargo 的记录里基线与变异那几次（记覆盖的 --no-run 与 --exact 两次不算）。"""
    return [line for line in invocations if "--no-run" not in line and "--exact" not in line]


def coverage_invocations(invocations):
    return [line for line in invocations if "--no-run" in line or "--exact" in line]


def mutation_invocations(invocations, pristine_hash):
    """假 cargo 的记录里改坏了源码的那几次（lib.rs 的哈希不是没改坏的那一份）。"""
    return [line for line in invocations if not line.endswith("\t" + pristine_hash)]


def selftest_verdict_view(stdout):
    """stdout 去掉随记录变的那几处（「复用：」「覆盖：」两行、抓到那一行末尾的「（现判：…）」「（复用）」「（拒算）」、成功句末尾「；这一趟现判 …」），剩下的是判定本身。"""
    lines = []
    for line in stdout.splitlines():
        if line.startswith("  复用：") or line.startswith("  覆盖："):
            continue
        line = re.sub(r"（现判：[^）]*）$|（复用）$|（拒算）$", "", line)
        lines.append(re.sub(r"；这一趟现判 .*$", "", line))
    return "\n".join(lines)


def source_hash(root):
    with open(os.path.join(root, "crates", "tiny", "src", "lib.rs"), encoding="utf-8") as handle:
        return hashlib.sha256(handle.read().encode()).hexdigest()[:12]


def replace_in_file(path, old, new):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    assert text.count(old) == 1, (path, old)
    selftest_write(path, text.replace(old, new))


def replace_in_part(root, old, new):
    """root 这棵拷贝里入口与拆出的各份模块（MODULE_SOURCE_PATHS）中含 old 的那一份（只许一份含），在它里面把 old 换成 new（old 在那一份里恰好一处）。"""
    holders = []
    for path in MODULE_SOURCE_PATHS:
        copied = os.path.join(root, os.path.relpath(path, STAGE_REPOSITORY))
        if os.path.isfile(copied):
            with open(copied, encoding="utf-8") as handle:
                if old in handle.read():
                    holders.append(copied)
    assert len(holders) == 1, (old, holders)
    replace_in_file(holders[0], old, new)


def per_row_how(stdout):
    """stdout 里抓到那几行：{变异名: 「现判：<栏>」「复用」或「拒算」}。"""
    found = {}
    for match in re.finditer(r"^  ✓ (\S+)：\S+ 红了（(现判：[^）]*|复用|拒算)）$", stdout, re.M):
        found[match.group(1)] = match.group(2)
    return found


def run_command_capturing(command, arguments, **keywords):
    stdout = io.StringIO()
    with contextlib.redirect_stdout(stdout):
        exit_code = command(arguments, **keywords)
    return exit_code, stdout.getvalue()
