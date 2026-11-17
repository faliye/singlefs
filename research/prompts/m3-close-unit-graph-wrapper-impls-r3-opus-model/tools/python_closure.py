#!/usr/bin/env python3
"""三方攻方腿（m3-close-unit-graph-wrapper-impls-r3）：用某一份 scripts/gate/lib_function_units.py 在夹具根上收一个 Rust 入口的闭包，
打出被改的那个方法在不在闭包里、照 unit-graph 的写法算的键（lib_function_units.py 的 unit_graph_key）。

用法：python_closure.py --library <那棵树的 scripts/gate> --root <夹具根> --file crates/<crate>/src/lib.rs --path <entry 或 Trait::method> --edit-line <行>
  --path 是 crate 名之后的路径：一段是模块顶层的 fn；两段是挂在 trait 头或 impl 头下的方法（trait 名，或 impl 自身类型里的一个名字）。
输出两行：holding=<闭包里行区间盖住 --edit-line 的 fn 单元个数>；key=<十六进制键，有拒算写 refused>。
退出码：0 交了两行；2 用法错、入口找不到或不唯一、库载不进来；3 闭包里有拒算（照打两行，另打拒算行）。
设 FUNCTION_UNITS_LEX_CACHE_DIRECTORY=off（不写词法缓存）、去掉 FUNCTION_UNITS_BREAK（规则全开）。"""
import argparse
import os
import sys

TOOLCHAIN_LINE = "host: x86_64-unknown-linux-gnu"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    parser.add_argument("--root", required=True)
    parser.add_argument("--file", required=True)
    parser.add_argument("--path", required=True)
    parser.add_argument("--edit-line", type=int, required=True)
    arguments = parser.parse_args()
    os.environ["FUNCTION_UNITS_LEX_CACHE_DIRECTORY"] = "off"
    os.environ.pop("FUNCTION_UNITS_BREAK", None)
    sys.path.insert(0, os.path.abspath(arguments.library))
    try:
        import lib_function_units as units
    except Exception as error:  # 库载不进来：照用法错退
        print(f"  ✗ 载不进 {arguments.library}/lib_function_units.py：{error!r}", file=sys.stderr)
        return 2
    workspace = units.RustWorkspace(arguments.root)
    segments = arguments.path.split("::")
    name = segments[-1]
    owner = segments[-2] if len(segments) > 1 else None
    rust_file = workspace.file(arguments.file)
    hits = []
    for unit in rust_file.units:
        if unit.kind != "fn" or unit.short != name:
            continue
        container = unit.container
        if owner is None and container is None:
            hits.append(unit)
        elif owner is not None and container is not None and (
                container.short == owner or owner in (container.self_types or ())):
            hits.append(unit)
    if len(hits) != 1:
        print(f"  ✗ {arguments.file} 里 {arguments.path} 命中 {len(hits)} 个 fn 单元（要恰好一个）", file=sys.stderr)
        return 2
    entry, target_id = workspace.find_entry(arguments.file, hits[0].name, None)
    closure = units.rust_closure_of(workspace, entry, target_id)
    wanted = os.path.realpath(os.path.join(arguments.root, arguments.file))
    holding = sum(1 for member in closure.order
                  if member.kind == "fn" and not member.data
                  and os.path.realpath(os.path.join(arguments.root, member.file)) == wanted
                  and member.first_line <= arguments.edit_line <= member.last_line)
    key, refusals = units.unit_graph_key(closure, toolchain=(TOOLCHAIN_LINE,))
    print(f"holding={holding}")
    print(f"key={key if key is not None else 'refused'}")
    for refusal in refusals:
        print(f"refused\t{refusal}")
    return 3 if refusals else 0


if __name__ == "__main__":
    sys.exit(main())
