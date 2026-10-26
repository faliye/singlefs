import importlib.util, sys
spec = importlib.util.spec_from_file_location("go", sys.argv[1])
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
root = "/home/fy5090/code/singlefs"
for cmd in [
  "cp /tmp/x/a.sh .claude/hooks/a.sh.new && mv .claude/hooks/a.sh.new .claude/hooks/a.sh",
  "python3 scripts/gate/hook-swap.py spec.json --hook .claude/hooks/a.sh --scratch /tmp/s",
  "mv .claude/hooks/a.sh.new .claude/hooks/a.sh",
]:
    print(repr(cmd[:80]), '->', sorted(m.bash_write_targets(cmd, root)))
