import sys, os, shutil, re
src = "/tmp/claude-1000/m3-gov-r3-freeze/tree/research/scripts"
name = sys.argv[1]
dst0 = f"/tmp/claude-1000/m3-gov-r3-defense/drvs/{name}"; dst = dst0 + "/research/scripts"
if os.path.exists(dst0): shutil.rmtree(dst0)
shutil.copytree(src, dst)
shutil.copytree('/tmp/claude-1000/m3-gov-r3-freeze/tree/.claude/scripts', dst0 + '/.claude/scripts')
p = dst + "/broad-mutation.py"
s = open(p, encoding="utf-8").read()
def rep(old, new):
    global s
    assert s.count(old) == 1, (name, old[:50], s.count(old))
    s = s.replace(old, new)
forced = {"gkl-no-toolchain": "global-key-no-toolchain-line", "gkl-no-files": "global-key-no-file-lines", "gkl-no-build": "global-key-no-build-lines",
          "gkl-no-tools": "global-key-no-tool-lines", "gkl-no-limits": "global-key-no-limits-line", "fop-path-blind": "fingerprint-path-blind"}
if name in forced:
    rep("def break_is(name):\n    return test_mode() and raw_break(name)", f"def break_is(name):\n    return name == {forced[name]!r}")
elif name == "limits-constant":
    rep("        lines.append(limits_line)\n", '        lines.append("limits constant")\n')
elif name == "gkl-no-escalation":
    rep("    lines.append(escalation_line())\n", "")
elif name == "gkl-no-exclusions":
    rep('    lines.append(f"exclusions {exclusions_digest}")\n', "")
elif name == "gkl-no-rules":
    rep('    lines = [f"rules {rules}"]\n', "    lines = []\n")
elif name == "vk-no-global":
    rep('judge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n"', 'judge {JUDGE_RULES_VERSION}\\n"')
elif name == "vk-no-toolchain":
    rep('f"{source_fingerprint}\\n{toolchain}judge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n"', 'f"{source_fingerprint}\\njudge {JUDGE_RULES_VERSION}\\nglobal {global_key_digest}\\n"')
elif name == "fop-content-blind":
    rep('    text = "".join(f"{manifest[path]}  {path}\\n" for path in sorted(manifest))', '    text = "".join(f"{path}\\n" for path in sorted(manifest))')
elif name == "doc-vars-dropped":
    rep('for name in DOC_TEST_VARIABLES if name in environment]', 'for name in () if name in environment]')
elif name == "build-env-own-copy":   # build_environment_of 不调 admission，自己只认 RUSTFLAGS
    rep("        lines = admission.build_environment_lines(root, environment)", '        lines = [(f"<构建环境：环境变量 RUSTFLAGS>", environment["RUSTFLAGS"].encode())] if "RUSTFLAGS" in environment else []')
elif name == "base":
    pass
else:
    raise SystemExit("unknown " + name)
open(p, "w", encoding="utf-8").write(s)
