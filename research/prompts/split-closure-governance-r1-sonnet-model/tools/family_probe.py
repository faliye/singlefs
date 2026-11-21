#!/usr/bin/env python3
"""拆分族认法的探针：把仓里 scripts/gate/lib_lag_split_family.py 的 family_growth、keyed_family_growth、family_owner 拿几组输入各喂一次，
每组打一行「case=<名> actual=<实际交出的> expected=<读码推出的>  verdict=<same|DIFFERENT>」。
用法：family_probe.py --repo <仓根>；退出码 0 全与读码推的一致，1 有一组不一致（不一致就是读码读错了，或实现变了）。"""
import argparse
import sys


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    options = parser.parse_args()
    sys.path.insert(0, options.repo + "/scripts/gate")
    import lib_lag_split_family as family

    def growth_paths(base, current):
        return sorted(item.path for item in family.family_growth(base, current))

    cases = [
        ("new-part-old-row-kept-sum-over",
         growth_paths({"x/foo.py": 5}, {"x/foo.py": 5, "x/foo_part.py": 2}), ["x/foo.py", "x/foo_part.py"]),
        ("new-part-sum-equal",
         growth_paths({"x/foo.py": 5}, {"x/foo.py": 3, "x/foo_part.py": 2}), []),
        ("same-prefix-unrelated-module-absorbed",
         growth_paths({"scripts/gate/lib_function.py": 5}, {"scripts/gate/lib_function.py": 2, "scripts/gate/lib_function_units.py": 3}), []),
        ("same-prefix-unrelated-module-own-row-in-base",
         growth_paths({"scripts/gate/lib_function.py": 5, "scripts/gate/lib_function_units.py": 1},
                      {"scripts/gate/lib_function.py": 2, "scripts/gate/lib_function_units.py": 3}), ["scripts/gate/lib_function_units.py"]),
        ("main-rs-entry-absorbs-whole-directory",
         growth_paths({"a/src/main.rs": 3}, {"a/src/main.rs": 1, "a/src/other_module.rs": 2}), []),
        ("hook-entry-absorbs-dash-sibling-script",
         growth_paths({"scripts/gate/gate.sh": 4}, {"scripts/gate/gate.sh": 1, "scripts/gate/gate-staged.sh": 3}), []),
        ("different-directory-is-outside",
         growth_paths({"x/foo.py": 5}, {"y/foo_part.py": 1}), ["y/foo_part.py"]),
        ("same-directory-name-not-matching-the-split-is-outside",
         growth_paths({"x/foo.py": 5}, {"x/foobar.py": 1}), ["x/foobar.py"]),
        ("nested-owner-takes-the-longest-prefix",
         family.family_owner("t/alpha_beta_gamma.py", {"t/alpha.py", "t/alpha_beta.py"}), "t/alpha_beta.py"),
        ("keyed-same-anchor-in-three-part-files-all-pass",
         family.keyed_family_growth({("x/foo.py", "S")}, [("x/foo.py", "S"), ("x/foo_a.py", "S"), ("x/foo_b.py", "S")]), []),
        ("keyed-new-anchor-in-part-file-is-growth",
         family.keyed_family_growth({("x/foo.py", "S")}, [("x/foo_a.py", "T")]), [("x/foo_a.py", "T")]),
        ("keyed-outside-family-is-growth",
         family.keyed_family_growth({("x/foo.py", "S")}, [("y/bar.py", "S")]), [("y/bar.py", "S")]),
    ]
    differences = 0
    for name, actual, expected in cases:
        verdict = "same" if actual == expected else "DIFFERENT"
        differences += verdict != "same"
        print(f"case={name} actual={actual} expected={expected}  verdict={verdict}")
    print(f"name=family_probe cases={len(cases)} different={differences}")
    return 1 if differences else 0


if __name__ == "__main__":
    sys.exit(main())
