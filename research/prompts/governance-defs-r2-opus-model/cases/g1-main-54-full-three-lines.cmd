layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"
git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }
SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"
