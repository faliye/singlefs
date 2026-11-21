use super::*;

pub(super) fn cell_of(mounts: usize, user_publishes: usize) -> RootChoiceCell {
    RootChoiceCell {
        mounts,
        user_publishes,
    }
}
