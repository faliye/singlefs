git diff --quiet -- litmus crates .claude/scripts/lkmm.sh .claude/gate.d/57-lkmm.sh; echo "diff=$?"; git ls-files --others --exclude-standard -- crates litmus .lkmm-static-only
