
    /// 把紧跟在后面的那一片的计数并进来：计数逐项相加；「第一处」只在前面各片都没有时取这一片的。各片按状态序号从小到大并，
    /// 「第一处」就是序号最小的那一处，与单线程逐个跑逐项相同。按字段拆开写全：新加一个字段而这里没并，编译不过。
    pub fn absorb_following_slice(&mut self, following_slice: Layer0Tally) {
        let Layer0Tally {
            states,
            violations,
            root_persisted_states,
            no_file_states,
            file_read_states,
            failed_states,
            journal_differing_states,
            verification_ran_states,
            verification_failed_states,
            first_violation,
            ignored_violations,
            first_ignored_violation,
            record_root_without_record,
            record_claimed_state_missing_unit,
            checker_evaluated_states,
            checker_violated_states,
            checker_first_violation,
            checker_not_applicable_states,
        } = following_slice;
        self.states += states;
        self.violations += violations;
        self.root_persisted_states += root_persisted_states;
        self.no_file_states += no_file_states;
        self.file_read_states += file_read_states;
        self.failed_states += failed_states;
        self.journal_differing_states += journal_differing_states;
        self.verification_ran_states += verification_ran_states;
        self.verification_failed_states += verification_failed_states;
        if self.first_violation.is_none() {
            self.first_violation = first_violation;
        }
        self.ignored_violations += ignored_violations;
        if self.first_ignored_violation.is_none() {
            self.first_ignored_violation = first_ignored_violation;
        }
        self.record_root_without_record += record_root_without_record;
        self.record_claimed_state_missing_unit += record_claimed_state_missing_unit;
        for (invariant, evaluated_states) in checker_evaluated_states {
            *self.checker_evaluated_states.entry(invariant).or_insert(0) += evaluated_states;
        }
        for (invariant, violated_states) in checker_violated_states {
            *self.checker_violated_states.entry(invariant).or_insert(0) += violated_states;
        }
        for (invariant, detail) in checker_first_violation {
            self.checker_first_violation
                .entry(invariant)
                .or_insert(detail);
        }
        for (invariant, not_applicable_states) in checker_not_applicable_states {
            *self
                .checker_not_applicable_states
                .entry(invariant)
                .or_insert(0) += not_applicable_states;
        }
    }
