    fn note_the_invariants_held(&mut self, conclusion: &GivenStateConclusion) {
        let held = conclusion.words.pool_checker_judged_bits
            & !conclusion.words.pool_checker_violated_bits;
        for (position, invariant) in singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .enumerate()
        {
            if held & (1u128 << position) != 0 {
                self.invariants_held_at_least_once.insert(invariant);
            }
        }
    }
