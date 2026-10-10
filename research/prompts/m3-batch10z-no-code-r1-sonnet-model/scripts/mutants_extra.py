# 追加的变异（mutate.py 导入）：名 -> [(文件, 旧串, 新串)]
X = {}
X['z2_accessor_zero'] = [('allocator.rs', '.filter(|(is_allocated, is_isolated)| **is_allocated && **is_isolated)', '.filter(|(_is_allocated, _is_isolated)| false)')]
X['z3_exempt_released_too'] = [('mount.rs', """        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    let mut referenced_by_candidates""", """        .iter()
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    let mut referenced_by_candidates""")]
X['z5_isolated_counter_not_bumped'] = [('allocator.rs', """            self.isolated_per_segment[segment] += 1;
            self.isolated_slots += 1;
        }
    }

    /// 分配位与隔离位同时置着的槽数""", """            self.isolated_per_segment[segment] += 1;
        }
    }

    /// 分配位与隔离位同时置着的槽数""")]
X['z13_skip_held_slots'] = [('allocator.rs', """            if !self.allocated[index] {
                self.isolate_slot_at_index(index);
            }""", """            if !self.allocated[index] && !self.held_until_floor_takes_effect[index] {
                self.isolate_slot_at_index(index);
            }""")]
X['z14_c_only_first_slot'] = [('allocator.rs', """        for index in start..end {
            if !self.allocated[index] {""", """        for index in start..start + 1 {
            if !self.allocated[index] {""")]
X['m0_c_loop_isolate_abandoned'] = [('mount.rs', """        allocator
            .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(
                placement.device,
                placement.slot,
                placement.span,
            );""", """        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);""")]
