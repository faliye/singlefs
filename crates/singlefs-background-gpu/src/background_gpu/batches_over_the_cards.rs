//! 一批在几张卡上的分活与交活：按卡的顺序填满、切段、重交、超时与显存不足重切，三段（校验和核对、区域扫描、每段活槽数）各一个入口。

use super::*;

/// 区域扫描那一段一种切法要交几次：头一片之后那几片的段，加映射每一片之下的段。
fn submissions_of_a_region_record_plan(plan: &RegionRecordPlanOnAGpu) -> usize {
    plan.chunks_of_the_pieces_after_the_heads.len()
        + plan
            .mapping_slices
            .iter()
            .map(|mapping_slice| mapping_slice.record_chunks.len())
            .sum::<usize>()
}

/// 选段打分那一段一种切法要交几次：记录表每一片之下的段。
fn submissions_of_a_live_slot_plan(plan: &[RecordSliceWithItsChunks]) -> usize {
    plan.iter()
        .map(|record_slice| record_slice.segment_chunks.len())
        .sum()
}

impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
    /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：最近交来的那一组点到的一张没开的卡重开到上限都没开成、换设置时量一张卡的代价出错了，
    /// 或参与的卡因显存上限一张都不剩，就交回是哪一种；分得到交回 `None`（切换点到的卡还在开时照的是交来之前那一组）。
    /// 两批之间那一问答换上了时带着它（[`LatestSettingsHandedIn::PutInEffect`]），核心层照它判停，不等头一批。
    /// 告知一张卡都不参与之后（切到 `cpu`）没有生效的那一组，核心层不问它。
    ///
    /// # Panics
    /// 参与的卡一张都不剩，却既没有开不成、量代价出错，也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，换上的那一组点的卡都开着，
    /// 换上之后没参与的只有后两种原因；参与的卡为空而这三样都没有的只有一种——告知一张卡都不参与（[`Self::use_no_gpu_from_the_next_batch`]）之后、
    /// 再交来的一组换上之前。这期间核心层生效的是 `cpu`：交来的一组还在开卡时两批之间那一问在走到这里之前答还在开卡，答开不成时这一组不生效、
    /// 核心层生效的照旧是 `cpu`（`singlefs_core::background_compute::BackgroundComputeOfThisMount` 只在实现答换上了那一刻换上 `gpu` 那一组），
    /// 所以核心层不为生效的那一组问它、不走 GPU 那一路交活。
    #[must_use]
    pub fn batch_not_split_over_the_gpus(&self) -> Option<BatchNotSplitOverTheGpus> {
        if let Some(not_opened) = &self.not_opened_after_the_reopenings_at_the_last_switch {
            return Some(
                BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(
                    not_opened.clone(),
                ),
            );
        }
        if let Some(not_measured) = &self.cost_not_measured_at_the_last_switch {
            return Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                device: not_measured.device.clone(),
                cause: not_measured.cause.clone(),
                pci_state_when_judged_unusable: not_measured.pci_state_when_judged_unusable.clone(),
            });
        }
        if !self.participating.is_empty() {
            return None;
        }
        assert!(
            !self.left_out_under_their_memory_limit.is_empty(),
            "参与的卡一张都不剩只有两种：换设置时量代价出错（上面交回了），或这组设置点的卡都因显存上限不用"
        );
        Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(
            self.left_out_under_their_memory_limit.clone(),
        ))
    }

    /// 一批里 `entries` 那几条按卡的顺序填满这一批还用的参与的卡、各卡切好（`cut` 切一张卡分到的那几条，照这张卡交活之前那一刻的地方；
    /// `submissions_of_the_plan` 数一种切法要交几次，填满照它判）：
    /// 这一批因空闲显存不用的卡（`gpus_short_of_free_memory_in_this_batch`，它在本机里的位置连同报的那一条，按不用的先后）不分；交活之前先等这张卡上前一次超时的那次提交
    /// （等满上限算这一批这张卡超时一次、再等，`waiting_for_the_gpu::outcome_waited_for_again_after_timeouts`），切不出最小的一片（显存上限扣掉已占的）的卡
    /// 这一批不分给它（`gpus_of_this_host::plans_of_a_batch_filling_the_gpus_in_order`）。交回的各卡分到的条目在 `entries` 里、
    /// 按卡的次序相连。没有条目交回空。
    ///
    /// # Errors
    /// 换设置时量代价出错、参与的卡因显存上限一张都不剩、这一批还用的卡都因空闲显存不用了（[`OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory`]，
    /// 连同这一次因显存上限不用的）、交活之前那一等出错、一张卡切不出而装不下的是绑定上限。
    fn plans_of_a_batch<Plan>(
        &mut self,
        entries: Range<usize>,
        gpus_short_of_free_memory_in_this_batch: &[(
            usize,
            GpuShortOfFreeMemoryForTheSmallestPiece,
        )],
        submissions_of_each_card: &mut [SubmissionsOfACardInTheBatch],
        cut: impl Fn(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
        submissions_of_the_plan: impl Fn(&Plan) -> usize,
    ) -> Result<Vec<PlanOfAGpuForTheBatch<Plan>>, OutcomesOfTheBatchNotComputed> {
        match self.batch_not_split_over_the_gpus() {
            Some(BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(
                GpuNotOpenedAfterTheReopenings {
                    device,
                    causes_of_each_opening,
                    pci_state_when_judged_unusable,
                },
            )) => {
                // 判它开不成那一刻读过了：每一批带的都是那一份，不重读。
                return Err(OutcomesOfTheBatchNotComputed::GpuFailed {
                    device,
                    cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                        causes_of_each_opening,
                    },
                    pci_state_when_judged_unusable,
                });
            }
            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                device,
                cause,
                pci_state_when_judged_unusable,
            }) => {
                // 量代价出错那一刻读过了（原因不是判卡用不了的那一种时没读）：每一批带的都是那一份，不重读。
                return Err(OutcomesOfTheBatchNotComputed::GpuFailed {
                    device,
                    cause,
                    pci_state_when_judged_unusable,
                });
            }
            Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
                if entries.is_empty() {
                    return Ok(Vec::new());
                }
                return Err(OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(
                    gpus_left_out,
                ));
            }
            None => {}
        }
        if entries.is_empty() {
            return Ok(Vec::new());
        }
        let gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece> =
            gpus_short_of_free_memory_in_this_batch
                .iter()
                .map(|(_position, short)| short.clone())
                .collect();
        let positions_short_of_free_memory: Vec<usize> = gpus_short_of_free_memory_in_this_batch
            .iter()
            .map(|(position, _short)| *position)
            .collect();
        let participating_in_this_batch = self
            .participating
            .clone()
            .leaving_out(|position| positions_short_of_free_memory.contains(&position));
        if participating_in_this_batch.is_empty() {
            return Err(
                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
                    gpus_short_of_free_memory,
                    gpus_left_out_under_their_memory_limit: Vec::new(),
                },
            );
        }
        let pci_state_reader = &self.pci_state_reader;
        let cards_on_this_host = &mut self.cards_on_this_host;
        let first_entry = entries.start;
        let shifted_by_the_first_entry =
            |relative: Range<usize>| (relative.start + first_entry)..(relative.end + first_entry);
        plans_of_a_batch_filling_the_gpus_in_order(
            &participating_in_this_batch,
            entries.len(),
            |position| {
                let card = &mut cards_on_this_host[position];
                let device = card.discrete_gpu().identifier.clone();
                let room = outcome_waited_for_again_after_timeouts(
                    &mut *card,
                    &mut submissions_of_each_card[position].timeouts,
                    Card::room_once_the_submission_running_past_its_wait_completed,
                )
                .map_err(|cause| BatchNotPlannedOverTheGpus::GpuFailed {
                    device: device.clone(),
                    cause,
                })?;
                let memory_account = card.budget().memory_account();
                Ok(RoomOfAGpuForTheBatch {
                    device,
                    room,
                    memory_limit: memory_account.memory_limit(),
                    bytes_already_held_on_the_device: memory_account.bytes_held(),
                })
            },
            |relative, room| cut(shifted_by_the_first_entry(relative), room),
            submissions_of_the_plan,
        )
        .map(|plans| {
            plans
                .into_iter()
                .map(|plan| PlanOfAGpuForTheBatch {
                    position_on_this_host: plan.position_on_this_host,
                    entries: shifted_by_the_first_entry(plan.entries),
                    room: plan.room,
                    plan: plan.plan,
                })
                .collect()
        })
        .map_err(|not_planned| match not_planned {
            BatchNotPlannedOverTheGpus::GpuFailed { device, cause } => {
                OutcomesOfTheBatchNotComputed::GpuFailed {
                    pci_state_when_judged_unusable: pci_state_reader
                        .pci_state_if_the_cause_judges_the_gpu_unusable(&device, &cause),
                    device,
                    cause,
                }
            }
            BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)
                if gpus_short_of_free_memory.is_empty() =>
            {
                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out)
            }
            BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
                    gpus_short_of_free_memory,
                    gpus_left_out_under_their_memory_limit: gpus_left_out,
                }
            }
            BatchNotPlannedOverTheGpus::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
                OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(
                    SmallestPieceNotHeldOnTheGpu {
                        device: smallest_piece.device,
                        entry_position: smallest_piece.entry_position + first_entry,
                        memory_limit: smallest_piece.memory_limit,
                        smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
                    },
                )
            }
        })
    }

    /// 一段（`segment`）的一次调用照三段共用的那一套交给参与的卡：按卡的顺序填满、切（[`Self::plans_of_a_batch`]，`submissions_of_the_plan` 数一种切法交几次），
    /// 再逐张卡照切法交
    /// （`run_the_plan_on_a_card`：这一段自己的交法，交它这张卡、分到的那几条、切法与这一批这张卡的那一份，每一段套重交）。
    /// 建缓冲显存不足（[`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]）就把这张卡这一次的地方压到建不成的那一段的一半、照新切法从头整段重交它分到的那几条
    /// （已交成的片的结果不用，第 4 条「这一段在这张卡上已做完的片不留」）
    /// （第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）；压到连最小的一片都切不出，这一批就不用这张卡
    /// （下一批照设置重新起），它分到的那几条（一条都没交回）重新分给这一批还用的参与的卡、照它们的地方切、交；
    /// 都不用了交回 [`OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory`]。各卡交回的按条目的次序接起来。
    /// 交成了几次、几片、重交了几次、重切了几次、哪几张卡分到了活，记进这一段的那一份（出错也记，[`Self::take_the_submissions_of_the_batch`]）。
    ///
    /// 重切那一圈：每一轮要么交回、要么这张卡的显存那一项严格变小（建不成的那一段是按此刻的界切的，不比界大，压到它的一半），压到切不出就停，所以有界。
    /// 重分那一圈：每次把一段条目放回去，就多一张这一批不用的卡（不再分给它），参与的卡有限，所以至多比参与的卡数多一轮。
    fn outcomes_of_a_batch_over_the_cards<Plan, Outcome>(
        &mut self,
        segment: GpuComputedSegment,
        entry_count: usize,
        cut: impl Fn(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
        submissions_of_the_plan: impl Fn(&Plan) -> usize,
        mut run_the_plan_on_a_card: impl FnMut(
            &mut Card,
            Range<usize>,
            &Plan,
            &mut SubmissionsOfACardInTheBatch,
        ) -> Result<Vec<Outcome>, GpuFailureCause>,
    ) -> Result<Vec<Outcome>, OutcomesOfTheBatchNotComputed> {
        let mut submissions_of_each_card: Vec<SubmissionsOfACardInTheBatch> = self
            .cards_on_this_host
            .iter()
            .map(|_card| SubmissionsOfACardInTheBatch::none_yet())
            .collect();
        let mut gpus_short_of_free_memory_in_this_batch: Vec<(
            usize,
            GpuShortOfFreeMemoryForTheSmallestPiece,
        )> = Vec::new();
        let mut outcomes_from_the_first_entry: Vec<(usize, Vec<Outcome>)> = Vec::new();
        let mut devices_given_work: Vec<GpuDeviceIdentifier> = Vec::new();
        let mut recuts_after_running_out_of_memory: u64 = 0;
        // 一张待分的条目段的栈，起头只有整批那一段（不是 0 到 entry_count 这几个数）：装不下最小一片的卡交回的那几条再压回来。
        let mut entries_to_hand_out: Vec<Range<usize>> = std::iter::once(0..entry_count).collect();
        let mut failure = None;
        'handing_out: while let Some(entries) = entries_to_hand_out.pop() {
            let plans = match self.plans_of_a_batch(
                entries,
                &gpus_short_of_free_memory_in_this_batch,
                &mut submissions_of_each_card,
                &cut,
                &submissions_of_the_plan,
            ) {
                Ok(plans) => plans,
                Err(not_planned) => {
                    failure = Some(not_planned);
                    break 'handing_out;
                }
            };
            for planned in plans {
                let card = &mut self.cards_on_this_host[planned.position_on_this_host];
                let device = card.discrete_gpu().identifier.clone();
                devices_given_work.push(device.clone());
                let submissions_of_the_card =
                    &mut submissions_of_each_card[planned.position_on_this_host];
                let mut room = planned.room;
                let mut plan = planned.plan;
                let outcomes_of_the_card = loop {
                    match run_the_plan_on_a_card(
                        &mut *card,
                        planned.entries.clone(),
                        &plan,
                        &mut *submissions_of_the_card,
                    ) {
                        Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                            bytes_held_by_the_chunk,
                            reason: _,
                        }) => {
                            recuts_after_running_out_of_memory += 1;
                            room = room.with_the_memory_limit_at_most(GpuMemoryInBytes(
                                singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_memory_cap_after_allocation_failure(bytes_held_by_the_chunk.0),
                            ));
                            // 重切不留已做完的片：这张卡分到的那几条照新切法从头整段重交。
                            match cut(planned.entries.clone(), room) {
                                Ok(recut) => plan = recut,
                                Err(smallest_piece) => {
                                    break OutcomesOfACardInTheBatch::ShortOfFreeMemory(
                                        GpuShortOfFreeMemoryForTheSmallestPiece {
                                            device: device.clone(),
                                            bytes_of_the_last_chunk_not_created:
                                                bytes_held_by_the_chunk,
                                            smallest_piece_bytes: smallest_piece
                                                .smallest_piece_bytes,
                                        },
                                    )
                                }
                            }
                        }
                        Ok(of_the_card) => break OutcomesOfACardInTheBatch::Computed(of_the_card),
                        Err(cause) => break OutcomesOfACardInTheBatch::Failed(cause),
                    }
                };
                match outcomes_of_the_card {
                    OutcomesOfACardInTheBatch::Computed(of_the_card) => {
                        outcomes_from_the_first_entry.push((planned.entries.start, of_the_card));
                    }
                    OutcomesOfACardInTheBatch::ShortOfFreeMemory(short) => {
                        gpus_short_of_free_memory_in_this_batch
                            .push((planned.position_on_this_host, short));
                        entries_to_hand_out.push(planned.entries);
                    }
                    OutcomesOfACardInTheBatch::Failed(cause) => {
                        failure = Some(OutcomesOfTheBatchNotComputed::GpuFailed {
                            pci_state_when_judged_unusable: self
                                .pci_state_reader
                                .pci_state_if_the_cause_judges_the_gpu_unusable(&device, &cause),
                            device,
                            cause,
                        });
                        break 'handing_out;
                    }
                }
            }
        }
        self.note_the_submissions_of_the_segment(
            segment,
            &submissions_of_each_card,
            devices_given_work,
            recuts_after_running_out_of_memory,
        );
        if let Some(failure) = failure {
            return Err(failure);
        }
        outcomes_from_the_first_entry.sort_by_key(|(first_entry, _outcomes)| *first_entry);
        let mut outcomes = Vec::with_capacity(entry_count);
        for (first_entry, of_the_card) in outcomes_from_the_first_entry {
            assert_eq!(
                first_entry,
                outcomes.len(),
                "各卡交回的条目按次序相连、不重不漏：重分的那几条接在原来的位置上"
            );
            outcomes.extend(of_the_card);
        }
        assert_eq!(
            outcomes.len(),
            entry_count,
            "每条都有两份结果，按条目的次序接回"
        );
        Ok(outcomes)
    }

    /// 把一段这一批在各卡上交的记进这一段的那一份（同一段一批里算了几次的加起来）。
    fn note_the_submissions_of_the_segment(
        &mut self,
        segment: GpuComputedSegment,
        submissions_of_each_card: &[SubmissionsOfACardInTheBatch],
        devices_given_work: Vec<GpuDeviceIdentifier>,
        recuts_after_running_out_of_memory: u64,
    ) {
        let submissions_of_the_segment = GpuSubmissionsOfOneSegment {
            submissions_completed: submissions_of_each_card
                .iter()
                .map(|of_the_card| of_the_card.submissions_completed)
                .sum(),
            pieces_submitted: submissions_of_each_card
                .iter()
                .map(|of_the_card| of_the_card.pieces_submitted)
                .sum(),
            resubmissions_after_timeouts: submissions_of_each_card
                .iter()
                .map(|of_the_card| u64::from(of_the_card.timeouts.resubmissions()))
                .sum(),
            recuts_after_running_out_of_memory,
            devices_given_work,
        };
        self.submissions_since_the_last_take
            .of_the_segment_mut(segment)
            .add(&submissions_of_the_segment);
    }

    /// 一批单元交两份核各算一遍，交回每条两份的结果（按交进来的次序），不比对、不动计数：对拍与 [`Self::verify_unit_checksums`] 都用它。
    /// 按卡的顺序填满各卡（每张卡一段相连的条目），每张卡上再按此刻剩下的显存切段，单独成一段都装不下的单元切成几片、逐片交、
    /// 各片的值两份各按自己的合法合成整单元的（`checksum_kernels::checksum_outcomes_of_the_units_cut_into_pieces`）；分活、切段、超时、重切照
    /// [`Self::outcomes_of_a_batch_over_the_cards`]。同一个条目键在这一批里出现两次，一片都不交给卡、交回重复的那个键（卡出错与重复键不同时报）。
    ///
    /// # Errors
    /// [`ChecksumBatchFailed`] 的三种。
    pub fn both_kernel_outcomes_of_the_batch<EntryKey: Eq + std::hash::Hash>(
        &mut self,
        units: &[UnitChecksumToVerify<'_, EntryKey>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
        if let Some(repeated) =
            first_entry_key_repeated_among(units.iter().map(|unit| &unit.entry_key))
        {
            return Err(ChecksumBatchFailed::SameEntryKeyTwiceInOneBatch(repeated));
        }
        let unit_lengths: Vec<InputLengthOnTheHostInBytes> = units
            .iter()
            .map(|unit| InputLengthOnTheHostInBytes::of_this_many_bytes(unit.unit_bytes.len()))
            .collect();
        self.outcomes_of_a_batch_over_the_cards(
            GpuComputedSegment::UnitChecksumVerification,
            units.len(),
            |entries, room| {
                pieces_cut_to_fit_on_the_gpu(&unit_lengths[entries], CHECKSUM_CHUNK_FOOTPRINT, room)
            },
            Vec::len,
            |card, entries, chunks_of_pieces, submissions_of_the_card| {
                checksum_outcomes_of_the_units_cut_into_pieces(
                    &units[entries],
                    chunks_of_pieces,
                    |piece_units| {
                        submissions_of_the_card.outcome_of_a_chunk(
                            &mut *card,
                            piece_units.len(),
                            |card_handed_the_chunk| {
                                card_handed_the_chunk
                                    .both_kernel_outcomes_of_a_checksum_chunk(piece_units)
                            },
                        )
                    },
                )
            },
        )
        .map_err(checksum_batch_failed_of)
    }

    /// 扫描里的校验和核对一批（第 8 条）：两份核各算一遍、逐条比对；连续不确定到 N 轮的条目交核心层的 CRC-32C 现判。
    /// 交回逐条的判定与确认、不确定、交 CPU 判三个计数。GPU 出错（含这一批这张卡第 4 次等超过上限）时交回出错，`uncertainty` 不动。
    ///
    /// # Errors
    /// [`ChecksumBatchFailed`] 的三种。
    pub fn verify_unit_checksums<EntryKey: Eq + std::hash::Hash + Clone>(
        &mut self,
        units: &[UnitChecksumToVerify<'_, EntryKey>],
        uncertainty: &mut ConsecutiveUncertaintyCounts<EntryKey>,
    ) -> Result<ReconciledBatch<ChecksumKernelOutcome>, ChecksumBatchFailed> {
        let both_kernel_outcomes = self.both_kernel_outcomes_of_the_batch(units);
        reconciled_unit_checksums(units, both_kernel_outcomes, uncertainty)
    }

    /// 区域扫描里一批记录交两份核各分一次类，交回每条两份的分类（按交进来的次序），不比对、不动计数：对拍与
    /// [`Self::classify_region_records`] 都用它。`records` 都是区域扫描会去读的那一种（`region_record_kernels` 文件头），
    /// `mapping_entries` 是这一版的中央映射条目（与核心层区域扫描建按身份索引的是同一份）。
    /// 按卡的顺序填满各卡，每张卡上再按此刻剩下的显存切段（整份映射每段都占一份；连同一条记录装不下时映射按落点切成几片逐片交，
    /// 最大那条记录连同一个落点上的条目还装不下时单元切成头一片与之后的几片，`region_record_kernels::region_record_plan_on_a_gpu`）；
    /// 分活、切段、超时、重切照 [`Self::outcomes_of_a_batch_over_the_cards`]。同一个条目键在这一批里出现两次，一片都不交给卡、交回重复的那个键（先于映射与记录的别的判）。
    ///
    /// # Errors
    /// [`RegionRecordBatchFailed`] 的四种。
    ///
    /// # Panics
    /// 有一条记录不是区域扫描会去读的那一种（`region_record_kernels::record_is_one_the_region_scan_reads`）：调用方先按 R 筛过。
    pub fn both_kernel_classifications_of_the_region_records<EntryKey: Eq + std::hash::Hash>(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, EntryKey>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, RegionRecordBatchFailed> {
        if let Some(repeated) =
            first_entry_key_repeated_among(records.iter().map(|record| &record.entry_key))
        {
            return Err(RegionRecordBatchFailed::SameEntryKeyTwiceInOneBatch(
                repeated,
            ));
        }
        assert!(
            records
                .iter()
                .all(|record| record_is_one_the_region_scan_reads(region, &record.record)),
            "交进来的记录要在 R 那块盘上、未释放、起点在 R 里、不罩出 R"
        );
        let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(mapping_entries)
            .map_err(RegionRecordBatchFailed::MappingKeyLongerThanTheKernelsHold)?;
        self.outcomes_of_a_batch_over_the_cards(
            GpuComputedSegment::RegionRecordClassification,
            records.len(),
            |entries, room| {
                region_record_plan_on_a_gpu(
                    &records[entries],
                    mapping_entries,
                    &mapping_tables,
                    room,
                )
            },
            submissions_of_a_region_record_plan,
            |card, entries, plan, submissions_of_the_card| {
                region_record_classifications_of_the_plan(
                    &records[entries],
                    plan,
                    |chunk_pieces, mapping_slice| {
                        submissions_of_the_card.outcome_of_a_chunk(
                            &mut *card,
                            chunk_pieces.len(),
                            |card_handed_the_chunk| {
                                card_handed_the_chunk.both_kernel_outcomes_of_a_region_record_chunk(
                                    chunk_pieces,
                                    mapping_slice.map(|slice| &slice.tables),
                                )
                            },
                        )
                    },
                )
            },
        )
        .map_err(region_record_batch_failed_of)
    }

    /// 区域扫描里的单元头解析与映射反查一批（第 1、8 条）：两份核各分一次类、逐条比对；连续不确定到 N 轮的记录交核心层的区域扫描现判
    /// （`region_record_kernels::cpu_reference_classification`，按身份的索引只在头一次要它时建）。交回逐条的判定与三个计数。
    /// GPU 出错或这一批交不成时交回出错，`uncertainty` 不动。
    ///
    /// # Errors
    /// [`RegionRecordBatchFailed`] 的四种。
    ///
    /// # Panics
    /// 同 [`Self::both_kernel_classifications_of_the_region_records`]。
    pub fn classify_region_records<EntryKey: Eq + std::hash::Hash + Clone>(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, EntryKey>],
        mapping_entries: &[CentralMappingEntry],
        uncertainty: &mut ConsecutiveUncertaintyCounts<EntryKey>,
    ) -> Result<ReconciledBatch<AllocationInRegion>, RegionRecordBatchFailed> {
        let both_kernel_classifications = self.both_kernel_classifications_of_the_region_records(
            region,
            records,
            mapping_entries,
        );
        reconciled_region_record_classifications(
            region,
            records,
            mapping_entries,
            both_kernel_classifications,
            uncertainty,
        )
    }

    /// 选段打分一批交两份核各数一遍，交回每段两份的活槽数（按 `segments` 的次序，逐条走遍记录那一份在前），不比对、不动计数：对拍与
    /// [`Self::count_live_slots_per_cluster_segment`] 都用它。`segments` 是 `unit_area` 那块盘上的整段（核心层按单元区给的条目键）。
    /// 按卡的顺序填满各卡，每张卡上再按此刻剩下的显存切段（整份记录表每段都占一份；连同一个段装不下时记录表按记录切成几片、各片数出的逐段相加，
    /// `live_slot_kernels::live_slot_plan_on_a_gpu`）；分活、切段、超时、重切照 [`Self::outcomes_of_a_batch_over_the_cards`]。
    /// 同一个段在这一批里出现两次，一片都不交给卡、交回重复的那个段。
    ///
    /// # Errors
    /// [`LiveSlotBatchFailed`] 的三种。
    ///
    /// # Panics
    /// 有一个段不在 `unit_area` 那块盘上。
    pub fn both_kernel_live_slots_of_the_cluster_segments(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<(u64, u64)>, LiveSlotBatchFailed> {
        if let Some(repeated) = first_entry_key_repeated_among(segments.iter()) {
            return Err(LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch(repeated));
        }
        assert!(
            segments
                .iter()
                .all(|segment| segment.device == unit_area.device),
            "交进来的段都在单元区那块盘上"
        );
        self.outcomes_of_a_batch_over_the_cards(
            GpuComputedSegment::LiveSlotsPerClusterSegment,
            segments.len(),
            |entries, room| {
                live_slot_plan_on_a_gpu(
                    AllocationRecordCountOfATable::of_this_many_records(allocation_records.len()),
                    ClusterSegmentCountOfAGpu::of_this_many_segments(entries.len()),
                    room,
                )
            },
            |plan: &Vec<RecordSliceWithItsChunks>| submissions_of_a_live_slot_plan(plan),
            |card, entries, plan, submissions_of_the_card| {
                live_slots_summed_over_the_record_slices(
                    allocation_records,
                    &segments[entries],
                    plan,
                    |chunk_segments, _records_of_the_slice, record_tables| {
                        submissions_of_the_card.outcome_of_a_chunk(
                            &mut *card,
                            chunk_segments.len(),
                            |card_handed_the_chunk| {
                                card_handed_the_chunk.both_kernel_live_slots_of_a_chunk(
                                    chunk_segments,
                                    record_tables,
                                    unit_area,
                                )
                            },
                        )
                    },
                )
            },
        )
        .map_err(live_slot_batch_failed_of)
    }

    /// 选段打分一批（第 1、8 条）：两份核各数一遍每段的活槽数、逐段比对；连续不确定到 N 轮的段交核心层的
    /// `live_slots_per_cluster_segment_by_the_records` 现判（`live_slot_kernels::cpu_reference_live_slots`）。交回逐段的判定与三个计数。
    /// GPU 出错或这一批交不成时交回出错，`uncertainty` 不动。
    ///
    /// # Errors
    /// [`LiveSlotBatchFailed`] 的三种。
    ///
    /// # Panics
    /// `segments` 不是 `unit_area` 的整段按起点升序一个不漏（`live_slot_kernels::reconciled_live_slots_of_the_segments`）。
    pub fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
        uncertainty: &mut ConsecutiveUncertaintyCounts<ClusterSegmentEntryKey>,
    ) -> Result<ReconciledBatch<u64>, LiveSlotBatchFailed> {
        let both_kernel_live_slots = self.both_kernel_live_slots_of_the_cluster_segments(
            allocation_records,
            unit_area,
            segments,
        );
        reconciled_live_slots_of_the_segments(
            allocation_records,
            unit_area,
            segments,
            both_kernel_live_slots,
            uncertainty,
        )
    }
}
