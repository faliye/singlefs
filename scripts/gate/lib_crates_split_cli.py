"""crates-split-pure-move.py 的命令行：三个子命令的参数解析与分发。命令一律返回（退出码, 输出行），打印交给调用方（自证并行跑各格时不抢标准输出）。"""
import argparse
import os
import posixpath

import lib_crates_split_common as common
import lib_crates_split_renames as test_renames
import lib_crates_split_tables as tables
import lib_crates_split_units as units
import lib_function_units as function_units

HOWTO = "用法见 python3 scripts/gate/crates-split-pure-move.py --help"
MUTATION_TABLE = "crates/mutations.tsv"
TEST_RENAMES_HELP = ("测试改名表（每行「旧全名<TAB>新全名」，可给几次取并集），写 <表> 或 <包>:<目标>=<表>（目标 lib、test:<名>、bin:<名>，只管那个包那个目标里的测试）："
                     "remap-mutations 照它改第 5、6 段里的测试名，registries 列出点名旧全名的行")


class ArgumentParser(argparse.ArgumentParser):
    def error(self, message):
        raise common.UsageProblem(message, HOWTO)


def build_parser():
    parser = ArgumentParser(prog="crates-split-pure-move.py", description="crates 大文件拆分用的纯挪动检查、变异行重归与登记表扫描；文件头写了判法与退出码")
    commands = parser.add_subparsers(dest="command", required=True)
    pure = commands.add_parser("pure-move", help="拆前、拆后两棵树里每个包的函数单元多重集对比")
    pure.add_argument("--before", required=True, help="拆前的树根")
    pure.add_argument("--after", required=True, help="拆后的树根")
    pure.add_argument("--crate", action="append", default=[], help="只比这个包目录（相对树根，可给几次）")
    pure.add_argument("--strict-use", action="store_true", help="拆前有、拆后没有的 use 也按差异判")
    pure.add_argument("--list-visibility", action="store_true", help="逐个列出只因可见性变了指纹的单元")
    remap = commands.add_parser("remap-mutations", help="变异表里被拆文件的行，文件列改到锚点现在所在的文件")
    remap.add_argument("--after", required=True, help="拆后的树根")
    remap.add_argument("--before", help="拆前的树根（给了才能从两棵树认被拆的文件，也只收拆前没有的文件进拆后那一族）")
    remap.add_argument("--split", action="append", default=[], help="被拆文件的旧路径（相对树根，可给几次）")
    remap.add_argument("--table", help=f"变异表，缺省取拆前的树（没给取拆后的树）里的 {MUTATION_TABLE}")
    remap.add_argument("--out", required=True, help="改后的整份表写到这里（不就地写）")
    remap.add_argument("--test-renames", action="append", default=[], help=TEST_RENAMES_HELP)
    registries = commands.add_parser("registries", help=".claude/gate.d/ 顶层 *.tsv 里点名被拆文件旧路径的行")
    registries.add_argument("--root", help="扫哪棵树的登记表，缺省取 --before，再缺省取 --after")
    registries.add_argument("--before", help="拆前的树根")
    registries.add_argument("--after", help="拆后的树根")
    registries.add_argument("--split", action="append", default=[], help="被拆文件的旧路径（相对树根，可给几次）")
    registries.add_argument("--test-renames", action="append", default=[], help=TEST_RENAMES_HELP)
    return parser


def existing_tree(path, label):
    if path is None:
        return None
    real = os.path.realpath(path)
    if not os.path.isdir(real):
        raise common.UsageProblem(f"{label} 不是目录：{path}", f"给树根的路径；{HOWTO}")
    return real


def dispatch(options, workers):
    if options.command == "pure-move":
        crate_filter = sorted({posixpath.normpath(path.rstrip("/")) for path in options.crate})
        status, lines, _totals = units.judge_pure_move(existing_tree(options.before, "--before"), existing_tree(options.after, "--after"),
                                                       crate_filter, options.strict_use, options.list_visibility, workers)
        return status, lines
    if options.command == "remap-mutations":
        after_root, before_root = existing_tree(options.after, "--after"), existing_tree(options.before, "--before")
        table = options.table or os.path.join(before_root or after_root, MUTATION_TABLE)
        status, lines, _result = tables.remap_command(after_root, before_root, options.split, table, options.out, test_renames.load_renames(options.test_renames))
        return status, lines
    before_root, after_root = existing_tree(options.before, "--before"), existing_tree(options.after, "--after")
    root = existing_tree(options.root, "--root") or before_root or after_root
    if root is None:
        raise common.UsageProblem("没给 --root，也没给 --before 与 --after：不知道扫哪棵树的登记表", f"--root 给一棵树；{HOWTO}")
    status, lines, _hits = tables.registries_command(root, before_root, after_root, options.split, test_renames.load_renames(options.test_renames))
    return status, lines


def run_command(arguments, workers):
    """→ (退出码, 输出行)。0 过，1 有差异或有行没改，2 用法错，3 算不出，77 本次未跑。"""
    try:
        return dispatch(build_parser().parse_args(arguments), workers)
    except common.UsageProblem as problem:
        return 2, [f"  ✗ {problem}", f"     → 怎么办：{problem.howto}"]
    except (function_units.FunctionUnitsError, OSError, UnicodeError) as error:
        return 3, [f"  ✗ 算不出：{error}", "     → 怎么办：看上面点名的树或文件读得了、Cargo.toml 写得对；两棵树都是完整的仓副本（要有根 Cargo.toml 与各包目录）"]
