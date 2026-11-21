//! 几张卡上的后台 GPU 的构造、对外报的几样，与两批之间换设置：交来新的一组、切到 `cpu`、换上切换以来开成的卡、关掉不再点名的卡。

use super::*;

impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
    /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显（连同没参与而没开的）、没点名而没收进清单的那几张、
    /// 开卡那一组参与的卡与因显存上限不用的卡（`gpus_of_this_host::gpus_opened_at_the_mount` 交回的），进程里泄漏的那一份（丢它时记进去），
    /// 卡清单每次答之前现查本机在的独显的那一样（产品带着开卡用的那一份进程级 Vulkan 实例），切换点到没开的卡时开它一次的那一步，
    /// 开不成时怎么重开（产品是 [`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]），与判一张卡用不了那一刻读它的 PCI 状态的那一份
    /// （产品是 [`PciStateReaderOfTheGpus::of_this_host`]）。测试拿它把替身卡装进来。
    ///
    /// # Panics
    /// 开成的卡与 `discrete_gpus_after_the_opening.opened` 张数不等。
    #[must_use]
    #[allow(
        clippy::too_many_arguments,
        reason = "挂载那一刻开卡之后的各样各是一个参数，测试照它逐样装替身；捆成一个结构体只是把同样几个字段挪个地方"
    )]
    pub fn of_the_cards_opened_at_the_mount(
        cards_on_this_host: Vec<Card>,
        discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
        unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
        participating: ParticipatingGpusOnThisHost,
        left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
        memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
        discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
        opening_of_a_gpu_at_a_switch: OpeningOfAGpuAtASwitch<Card>,
        reopenings_of_a_gpu_not_opened: ReopeningsOfAGpuNotOpened,
        pci_state_reader: PciStateReaderOfTheGpus,
    ) -> Self {
        assert_eq!(
            cards_on_this_host.len(),
            discrete_gpus_after_the_opening.opened.len(),
            "开成的卡与开卡之后的独显清单一一对应"
        );
        let devices_named_by_the_settings_in_effect = discrete_gpus_after_the_opening
            .opened
            .iter()
            .map(|discrete_gpu| discrete_gpu.identifier.clone())
            .collect();
        Self {
            cards_on_this_host,
            discrete_gpus_after_the_opening,
            unnamed_discrete_gpus_left_off_the_list,
            participating,
            left_out_under_their_memory_limit,
            not_opened_after_the_reopenings_at_the_last_switch: None,
            cost_not_measured_at_the_last_switch: None,
            opening_of_a_gpu_at_a_switch: Arc::new(Mutex::new(opening_of_a_gpu_at_a_switch)),
            reopenings_of_a_gpu_not_opened,
            latest_handing_in: HandingInNumber(0),
            devices_named_by_the_settings_in_effect,
            settings_waiting_for_the_opening: None,
            opening_at_a_switch: None,
            start_of_the_opening_thread: StartOfTheOpeningThread::ByTheStandardLibrary,
            failed_openings_in_this_mount: FailedOpeningsOfEachGpuInThisMount::default(),
            gpus_leaked_when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch::with_none_leaked(),
            submissions_since_the_last_take: GpuSubmissionsOfTheBatch::default(),
            gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
            memory_leaked_in_this_process,
            discrete_gpus_present_now,
            pci_state_reader,
            caller_stopping_seen_by_the_waits_of_the_batches:
                CallerStoppingSeenByTheWait::NotReadByThisWait,
        }
    }

    /// 收下调用方在停的标记（核心层建把手时交一次，`BackgroundGpuCompute::give_up_waiting_for_the_cards_once_the_caller_is_stopping` 经
    /// `crate::background_gpu_compute::BothKernelsOfTheThreeSegments` 转来）：从此每张开着的卡等一批的结果（交一段之后等它、冲掉待写那一等、
    /// 交活与重交之前等前一次还在跑的那次）都拆成短等、每一轮醒来读它，置了就放弃那一等（第 4 条「卸载时不等还在跑的提交」）；
    /// 之后切换开成、收进来的卡同样（[`Self::take_in_the_cards_opened_at_a_switch`]）。开卡那几等（挂载那一刻、切换开卡线程上）不读它。
    pub fn give_up_waiting_for_the_cards_once_the_caller_is_stopping(
        &mut self,
        caller_is_stopping: CallerIsStoppingTheBackgroundGpu,
    ) {
        self.caller_stopping_seen_by_the_waits_of_the_batches =
            CallerStoppingSeenByTheWait::ReadBetweenShortWaits(caller_is_stopping);
        for card in &mut self.cards_on_this_host {
            card.budget_mut()
                .read_the_caller_stopping_in_the_waits_of_the_batches(
                    &self.caller_stopping_seen_by_the_waits_of_the_batches,
                );
        }
    }

    /// 此刻开着的卡（挂载那一刻开的参与的卡按总线地址排，为切换开的依次排在后面；换上一组时不再点名的已关掉），参与与否都列。
    #[must_use]
    pub fn gpus_opened(&self) -> Vec<GpuDeviceIdentifier> {
        self.cards_on_this_host
            .iter()
            .map(|card| card.discrete_gpu().identifier.clone())
            .collect()
    }

    /// 给了 `background_gpu_devices` 时没点名、交不成一张卡（缺总线地址、缺设备本地堆、地址重复）而没收进卡清单的独显，各带原因；没有是空。
    #[must_use]
    pub fn unnamed_discrete_gpus_left_off_the_list(&self) -> &[DiscreteGpuListRefused] {
        &self.unnamed_discrete_gpus_left_off_the_list
    }

    /// 此刻这组设置点了而因显存上限连最小的一片都装不下、不用的卡，按参与的次序，各带上限与至少要多少；没有是空。
    /// 与 [`Self::participating_gpus`] 一起看：参与的卡一张都不剩时三段每一批都交回「参与的卡一张都不剩」（[`Self::batch_not_split_over_the_gpus`]）。
    #[must_use]
    pub fn gpus_left_out_under_their_memory_limit(&self) -> &[GpuLeftOutUnderItsMemoryLimit] {
        &self.left_out_under_their_memory_limit
    }

    /// 丢掉它那一刻泄漏了哪几张卡（还没丢是 `None`）：调用方拿着这一份，丢了之后读（[`GpusLeakedWhenTheBackgroundGpuIsDropped`]）。
    #[must_use]
    pub fn gpus_leaked_when_dropped(&self) -> GpusLeakedWhenTheBackgroundGpuIsDropped {
        self.gpus_leaked_when_dropped.clone()
    }

    /// 挂着切换关卡时泄漏了哪几张卡（有提交还在跑的那张整张泄漏、不析构，同卸载的规矩），按关的先后累计：调用方拿着这一份随时读
    /// （[`GpusLeakedWhenClosedAtASwitch`]，`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    #[must_use]
    pub fn gpus_leaked_when_closed_at_a_switch(&self) -> GpusLeakedWhenClosedAtASwitch {
        self.gpus_leaked_when_closed_at_a_switch.clone()
    }

    /// 这一次挂载里判卡用不了时一共读了几次 PCI 状态（切换开卡线程上读的也算；`.claude/rules/fs-design.md` 五条硬要求第 4 条：看得出读了没有）。
    #[must_use]
    pub fn pci_state_readings_taken(&self) -> u64 {
        self.pci_state_reader.readings_taken()
    }

    /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：接下来 `refusals` 次起切换开卡那条线程都照起不了报，强制进入「起不了开卡线程」那一支。
    pub fn refuse_to_start_the_opening_thread_for_tests(&mut self, refusals: u32) {
        self.start_of_the_opening_thread = StartOfTheOpeningThread::RefusedForTests {
            refusals_left: refusals,
        };
    }

    /// 上一次交回以来三段各自在卡上是怎么交的（交成了几次、几片、重交了几次、显存不足重切了几次、分到活的是哪几张卡），交回之后清零。
    pub fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        std::mem::take(&mut self.submissions_since_the_last_take)
    }

    /// 此刻参与的卡，按参与的次序，带此刻设置给的显存上限与量出的一次提交的主机 CPU 时间。
    ///
    /// # Panics
    /// 一张参与的卡没量过代价：开卡或换设置让它参与的那一刻量过，没量成的不参与。
    #[must_use]
    pub fn participating_gpus(&self) -> Vec<ParticipatingGpuReport> {
        self.participating
            .positions_on_this_host()
            .iter()
            .map(|position| {
                let card = &self.cards_on_this_host[*position];
                ParticipatingGpuReport {
                    identifier: card.discrete_gpu().identifier.clone(),
                    name: card.discrete_gpu().name.clone(),
                    memory_limit: card.budget().memory_account().memory_limit(),
                    measured_cost: card
                        .measured_cost()
                        .expect("参与的卡都量过代价：开卡或换设置让它参与的那一刻量的"),
                }
            })
            .collect()
    }

    /// 从下一批起照核心层交来的这一组跑（D26 已定项 13 第 1–3 条；核心层只在两批之间交）：参与的卡换成它点的那几张、按它的次序，
    /// 每张卡的显存上限与等的上限换成它给的。开着的卡不重开，各卡的账照旧记着。这一次交来记一个新的次序号。
    /// 它点的卡都开着，就当场换上（[`Self::put_the_settings_in_effect`]：不再点名的卡关掉）。它点到没开的卡（挂载时没开的，或之前切换、切到 `cpu` 关掉的），
    /// 照第 4 条「切换请求照收，在会话锁外开新卡」：这一刻不开卡，在另一条线程上按点的次序逐张开（开不成就隔一段时间重开，
    /// [`Self::of_the_cards_opened_at_the_mount`] 给的重开规矩），开着的这段时间照旧照交来之前那一组跑；开完了在之后的两批之间换上
    /// （[`Self::put_the_gpus_opened_since_the_switch_in_effect`]）。它点到的没开的卡里有一张在这一次挂载里已连着试满
    /// （[`ReopeningsOfAGpuNotOpened::openings_at_most`] 次都没开成）：不再开，当场记成开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]），
    /// 此刻生效那一组之外开着的卡关掉、照旧照它跑。上一次切换的卡还在开时又交来一组，记下最近这一组，等那一次开完再看。
    /// 上一次切换留下的「重开到上限都没开成」在这一刻清掉（下一次换设置解除它）。
    pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.latest_handing_in = self.latest_handing_in.next();
        self.not_opened_after_the_reopenings_at_the_last_switch = None;
        let discrete_gpus_to_open = self.discrete_gpus_named_and_not_opened(settings);
        if discrete_gpus_to_open.is_empty() {
            self.settings_waiting_for_the_opening = None;
            self.put_the_settings_in_effect(settings);
            return;
        }
        if let Some(used_up) =
            self.gpu_whose_openings_in_this_mount_are_used_up(&discrete_gpus_to_open)
        {
            self.settings_waiting_for_the_opening = None;
            self.close_the_cards_outside_the_settings_in_effect();
            self.not_opened_after_the_reopenings_at_the_last_switch = Some(used_up);
            return;
        }
        self.settings_waiting_for_the_opening = Some(SettingsWaitingForTheOpening {
            handing_in_number: self.latest_handing_in,
            settings: settings.clone(),
        });
        if self.opening_at_a_switch.is_none() {
            self.start_the_opening_at_a_switch(
                self.latest_handing_in,
                discrete_gpus_to_open,
                settings.wait_limit(),
                Vec::new(),
            );
        }
    }

    /// 从下一批起一张卡都不参与（核心层切到 `cpu` 时交来；第 2 条「从 gpu 切到 cpu……关掉全部开着的卡、丢掉还在开的那一组」）：这一次也记一个新的次序号，
    /// 在会话锁外开着的那一次叫停（开卡线程不再开下一次，开出来的没人收、在那条线程上照常丢），已送回还没取走的开卡结局里开成的卡收下再关，
    /// 还在等开卡的那一组丢掉，开着的卡全部关掉（有提交还在跑的那张整张泄漏，[`Self::close_the_cards`]），「重开到上限都没开成」与「换设置时量代价出错」清掉。
    /// 之后再交来一组，它点的卡都当没开的卡照重开的规矩开。
    pub fn use_no_gpu_from_the_next_batch(&mut self) {
        self.latest_handing_in = self.latest_handing_in.next();
        self.settings_waiting_for_the_opening = None;
        self.not_opened_after_the_reopenings_at_the_last_switch = None;
        self.cost_not_measured_at_the_last_switch = None;
        if let Some(opening) = self.opening_at_a_switch.take() {
            opening.call_off();
            let cards_opened_and_not_taken = match opening {
                OpeningAtASwitch::UnderWay {
                    outcome_receiver, ..
                } => outcome_receiver
                    .try_recv()
                    .map_or_else(|_nothing_sent| Vec::new(), |outcome| outcome.opened),
                OpeningAtASwitch::EndedWithoutAThread { outcome, .. } => outcome.opened,
                OpeningAtASwitch::ThreadNotStarted { .. } => Vec::new(),
            };
            self.take_in_the_cards_opened_at_a_switch(cards_opened_and_not_taken);
        }
        self.devices_named_by_the_settings_in_effect.clear();
        self.left_out_under_their_memory_limit.clear();
        self.close_the_cards(|_device| true);
    }

    /// 两批之间（核心层每一批开头与整理两个入口判停没停之前，经 `BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）：
    /// 会话锁外为切换开卡的那一次开完了就在这一刻收下它的结局（[`Self::take_in_the_outcome_of_the_opening`]），起不了开卡线程的到了间隔就再起
    /// （那一次之后又交来了别的一组的，为最近那一组起，不为被盖掉的那一组起；最近没有在等开卡的一组就不起）；
    /// 然后交回最近交来的那一组此刻怎样了（[`LatestSettingsHandedIn`]）。`wait` 是 [`WaitForTheOpeningAtASwitch::NotAtAll`] 时还在开就不等，
    /// [`WaitForTheOpeningAtASwitch::UntilItEnds`] 时一直等到为最近交来的那一组开卡开完。
    ///
    /// # Panics
    /// 开卡线程没送结局就断开了：开一次卡里的 panic 在那条线程上就接住（[`opening_with_its_panic_caught`]），断开只剩这个包自己的代码 panic 一种，照原样再 panic。
    pub fn put_the_gpus_opened_since_the_switch_in_effect(
        &mut self,
        wait: WaitForTheOpeningAtASwitch,
    ) -> LatestSettingsHandedIn {
        let mut round: u64 = 0;
        while let Some(opening) = self.opening_at_a_switch.take() {
            round += 1;
            // 照进度打一行（第 1、2、4、8… 轮）：等得久的能从日志看出在第几轮、等的是哪一步，空转的轮数一路翻倍也看得出来
            if round.is_power_of_two() {
                eprintln!(
                    "BACKGROUND_GPU_SWITCH_OPENING_PROGRESS round={round} waiting={} step={}",
                    matches!(wait, WaitForTheOpeningAtASwitch::UntilItEnds),
                    match &opening {
                        OpeningAtASwitch::EndedWithoutAThread { .. } => "ended-without-a-thread",
                        OpeningAtASwitch::ThreadNotStarted { .. } => "thread-not-started",
                        OpeningAtASwitch::UnderWay { .. } => "under-way",
                    }
                );
            }
            match opening {
                OpeningAtASwitch::EndedWithoutAThread {
                    handing_in_number,
                    outcome,
                } => self.take_in_the_outcome_of_the_opening(handing_in_number, outcome),
                OpeningAtASwitch::ThreadNotStarted {
                    handing_in_number,
                    discrete_gpus_to_open,
                    wait_limit,
                    causes_of_each_start,
                    not_started_at,
                } => {
                    let interval_left = self
                        .reopenings_of_a_gpu_not_opened
                        .interval
                        .saturating_sub(not_started_at.elapsed());
                    if !interval_left.is_zero() {
                        match wait {
                            WaitForTheOpeningAtASwitch::NotAtAll => {
                                self.opening_at_a_switch =
                                    Some(OpeningAtASwitch::ThreadNotStarted {
                                        handing_in_number,
                                        discrete_gpus_to_open,
                                        wait_limit,
                                        causes_of_each_start,
                                        not_started_at,
                                    });
                                break;
                            }
                            WaitForTheOpeningAtASwitch::UntilItEnds => {
                                std::thread::sleep(interval_left);
                            }
                        }
                    }
                    let latest_waiting_handing_in_number = self
                        .settings_waiting_for_the_opening
                        .as_ref()
                        .map(|waiting| waiting.handing_in_number);
                    if latest_waiting_handing_in_number == Some(handing_in_number) {
                        self.start_the_opening_at_a_switch(
                            handing_in_number,
                            discrete_gpus_to_open,
                            wait_limit,
                            causes_of_each_start,
                        );
                    } else {
                        self.start_the_opening_for_the_settings_waiting();
                    }
                }
                OpeningAtASwitch::UnderWay {
                    handing_in_number,
                    outcome_receiver,
                    opening_thread,
                    called_off,
                } => {
                    let received = match wait {
                        WaitForTheOpeningAtASwitch::NotAtAll => outcome_receiver.try_recv(),
                        WaitForTheOpeningAtASwitch::UntilItEnds => outcome_receiver
                            .recv()
                            .map_err(|_disconnected| TryRecvError::Disconnected),
                    };
                    match received {
                        Ok(outcome) => {
                            self.take_in_the_outcome_of_the_opening(handing_in_number, outcome);
                        }
                        Err(TryRecvError::Empty) => {
                            self.opening_at_a_switch = Some(OpeningAtASwitch::UnderWay {
                                handing_in_number,
                                outcome_receiver,
                                opening_thread,
                                called_off,
                            });
                            break;
                        }
                        Err(TryRecvError::Disconnected) => {
                            std::panic::resume_unwind(opening_thread.join().expect_err(
                                "开卡线程没送结局就断开只有一种：它 panic 了（正常结束一定送了，送的时候这一头还拿着收的那一端）",
                            ))
                        }
                    }
                }
            }
        }
        self.latest_settings_handed_in()
    }

    /// 最近交来的那一组此刻怎样了：还在等开卡是 [`LatestSettingsHandedIn::StillOpeningItsCards`]；为它开卡重开到上限都没开成是
    /// [`LatestSettingsHandedIn::NotOpened`]；别的是换上了，连同这组设置下一批分不分得到卡上。
    fn latest_settings_handed_in(&self) -> LatestSettingsHandedIn {
        if self.settings_waiting_for_the_opening.is_some() {
            return LatestSettingsHandedIn::StillOpeningItsCards;
        }
        if let Some(not_opened) = &self.not_opened_after_the_reopenings_at_the_last_switch {
            return LatestSettingsHandedIn::NotOpened(not_opened.clone());
        }
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: self.batch_not_split_over_the_gpus(),
        }
    }

    /// 收下切换开卡开成的卡：接到开着的卡末尾，卡清单里从没开的那几张挪到开着的那几张；收进来那一刻起它等一批的结果照开着的卡读调用方在停的标记
    /// （在开卡线程上开它、量它代价的那几等不读：那时它还没收进来）。
    fn take_in_the_cards_opened_at_a_switch(&mut self, cards_opened: Vec<Card>) {
        for mut card in cards_opened {
            card.budget_mut()
                .read_the_caller_stopping_in_the_waits_of_the_batches(
                    &self.caller_stopping_seen_by_the_waits_of_the_batches,
                );
            let identifier = card.discrete_gpu().identifier.clone();
            self.discrete_gpus_after_the_opening
                .note_the_gpu_opened_at_a_switch(&identifier);
            assert!(
                !self
                    .discrete_gpus_after_the_opening
                    .not_opened_for_not_taking_part
                    .iter()
                    .any(|discrete_gpu| discrete_gpu.identifier == identifier),
                "切换开成的卡记账之后不该还在没开的那几张里（不然再交同一组会为它一遍遍重开、永远换不上）：{identifier:?}"
            );
            self.cards_on_this_host.push(card);
        }
    }

    /// 收下为第 `handing_in_number` 次交来的设置开卡的结局：开成的卡先收下（[`Self::take_in_the_cards_opened_at_a_switch`]），再照最近交来、在等的那一组办——
    /// 开卡为的就是它（次序号相同）、有一张重开到上限都没开成：此刻生效那一组之外开着的卡全部关掉（连同为更早几次交来的设置开成的），记下那一张，
    /// 照旧照此刻生效那一组跑，直到下一次换设置；它点的卡都开着了：换上它（不再点名的卡关掉）；它还点着没开的卡（开卡为的是更早一次交来的，
    /// 那一次开不成的不报给它）：此刻生效那一组与它都没点的开着的卡当场关掉（为被盖掉的那一组开成的），再为它开一次
    /// （[`Self::start_the_opening_for_the_settings_waiting`]）。没有在等的那一组（之后交来的一组不用开卡、已换上，或告知过一张卡都不参与）：
    /// 此刻生效那一组之外开着的卡全部关掉。
    fn take_in_the_outcome_of_the_opening(
        &mut self,
        handing_in_number: HandingInNumber,
        outcome: OutcomeOfTheOpeningAtASwitch<Card>,
    ) {
        self.take_in_the_cards_opened_at_a_switch(outcome.opened);
        let Some(settings_waiting) = self.settings_waiting_for_the_opening.as_ref() else {
            self.close_the_cards_outside_the_settings_in_effect();
            return;
        };
        if settings_waiting.handing_in_number == handing_in_number {
            if let Some(not_opened) = outcome.not_opened {
                self.settings_waiting_for_the_opening = None;
                self.close_the_cards_outside_the_settings_in_effect();
                self.not_opened_after_the_reopenings_at_the_last_switch = Some(not_opened);
                return;
            }
        }
        let devices_named_by_the_settings_in_effect_or_waiting: Vec<GpuDeviceIdentifier> = self
            .devices_named_by_the_settings_in_effect
            .iter()
            .cloned()
            .chain(
                settings_waiting
                    .settings
                    .participating_devices()
                    .iter()
                    .map(|participating| participating.device.clone()),
            )
            .collect();
        self.close_the_cards(|device| {
            !devices_named_by_the_settings_in_effect_or_waiting.contains(device)
        });
        self.start_the_opening_for_the_settings_waiting();
    }

    /// 为最近交来、在等开卡的那一组（有的话）接着办：它点的卡都开着了就换上；还点着没开的卡，其中有一张这一次挂载里已连着试满就记成开不成
    /// （不再开，此刻生效那一组之外开着的卡关掉），不然为它起开卡（起线程之前没有起不了的原因）。没有在等的那一组就什么都不做。
    /// 起不了开卡线程、到了间隔而之后又交来了别的一组时，也走这一步为最近那一组起。
    fn start_the_opening_for_the_settings_waiting(&mut self) {
        let Some(settings_waiting) = self.settings_waiting_for_the_opening.take() else {
            return;
        };
        let discrete_gpus_to_open =
            self.discrete_gpus_named_and_not_opened(&settings_waiting.settings);
        if discrete_gpus_to_open.is_empty() {
            self.put_the_settings_in_effect(&settings_waiting.settings);
            return;
        }
        if let Some(used_up) =
            self.gpu_whose_openings_in_this_mount_are_used_up(&discrete_gpus_to_open)
        {
            self.close_the_cards_outside_the_settings_in_effect();
            self.not_opened_after_the_reopenings_at_the_last_switch = Some(used_up);
            return;
        }
        let waiting_handing_in_number = settings_waiting.handing_in_number;
        let wait_limit = settings_waiting.settings.wait_limit();
        self.settings_waiting_for_the_opening = Some(settings_waiting);
        self.start_the_opening_at_a_switch(
            waiting_handing_in_number,
            discrete_gpus_to_open,
            wait_limit,
            Vec::new(),
        );
    }

    /// `discrete_gpus_to_open` 里头一张在这一次挂载里已连着试满（开不成的次数到了 [`ReopeningsOfAGpuNotOpened::openings_at_most`]）的卡，
    /// 记成它开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]，只一个原因），带判它开不成这一刻读的它的 PCI 状态；都没试满交回 `None`（不读）。
    fn gpu_whose_openings_in_this_mount_are_used_up(
        &self,
        discrete_gpus_to_open: &[(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)],
    ) -> Option<GpuNotOpenedAfterTheReopenings> {
        discrete_gpus_to_open
            .iter()
            .find(|(discrete_gpu, _memory_limit)| {
                self.failed_openings_in_this_mount
                    .openings_left_in_this_mount_of(
                        &discrete_gpu.identifier,
                        self.reopenings_of_a_gpu_not_opened,
                    )
                    == 0
            })
            .map(
                |(discrete_gpu, _memory_limit)| GpuNotOpenedAfterTheReopenings {
                    device: discrete_gpu.identifier.clone(),
                    causes_of_each_opening: vec![openings_in_this_mount_used_up(
                        &self.failed_openings_in_this_mount,
                        &discrete_gpu.identifier,
                    )],
                    pci_state_when_judged_unusable: self
                        .pci_state_reader
                        .pci_state_of_the_gpu(&discrete_gpu.identifier),
                },
            )
    }

    /// 关掉此刻生效那一组没点的开着的卡（[`Self::close_the_cards`]）。
    fn close_the_cards_outside_the_settings_in_effect(&mut self) {
        let devices_named_by_the_settings_in_effect =
            self.devices_named_by_the_settings_in_effect.clone();
        self.close_the_cards(|device| !devices_named_by_the_settings_in_effect.contains(device));
    }

    /// 这一组点到、此刻没开的卡（按点的次序），各带这一组给它的显存上限。
    fn discrete_gpus_named_and_not_opened(
        &self,
        settings: &AdmittedGpuSettings,
    ) -> Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)> {
        settings
            .participating_devices()
            .iter()
            .filter_map(|participating| {
                self.discrete_gpus_after_the_opening
                    .not_opened_for_not_taking_part
                    .iter()
                    .find(|discrete_gpu| discrete_gpu.identifier == participating.device)
                    .map(|discrete_gpu| (discrete_gpu.clone(), participating.memory_limit))
            })
            .collect()
    }

    /// 起切换开卡那条线程（[`StartOfTheOpeningThread`]）：产品照 std 起；只供测试的那一种照剩下的次数报起不了。
    fn opening_thread_started(
        &mut self,
        body_of_the_thread: impl FnOnce() + Send + 'static,
    ) -> std::io::Result<JoinHandle<()>> {
        match &mut self.start_of_the_opening_thread {
            StartOfTheOpeningThread::ByTheStandardLibrary
            | StartOfTheOpeningThread::RefusedForTests { refusals_left: 0 } => {
                std::thread::Builder::new()
                    .name("singlefs-opening-gpus-at-a-switch".to_string())
                    .spawn(body_of_the_thread)
            }
            StartOfTheOpeningThread::RefusedForTests { refusals_left } => {
                *refusals_left -= 1;
                Err(std::io::Error::other("只供测试：照起不了切换开卡线程报"))
            }
        }
    }

    /// 在另一条线程上为第 `handing_in_number` 次交来的设置开 `discrete_gpus_to_open` 那几张（按给的次序逐张开，开一张同挂载那一刻开卡那一步，
    /// [`gpus_opened_at_a_switch`]；开不成就照重开的规矩重开，每次重开之前在那条线程上睡一个间隔；有一张重开到上限都没开成，后面的就不再开）。
    /// 不等它：调用即返回。`causes_of_each_start` 是之前几次起不了这条线程的原因。起不了那条线程，就算头一张要开的卡这一次没开成
    /// （[`GpuFailureCause::OpeningThreadNotStarted`]，记进它这一次挂载里连着开不成的次数）：它还没试满就隔重开的间隔、在之后的两批之间再起
    /// （[`OpeningAtASwitch::ThreadNotStarted`]），试满了结局就定成头一张开不成。
    fn start_the_opening_at_a_switch(
        &mut self,
        handing_in_number: HandingInNumber,
        discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
        wait_limit: Duration,
        causes_of_each_start: Vec<GpuFailureCause>,
    ) {
        let opening_of_a_gpu_at_a_switch = Arc::clone(&self.opening_of_a_gpu_at_a_switch);
        let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
        let failed_openings_in_this_mount = self.failed_openings_in_this_mount.clone();
        let pci_state_reader = self.pci_state_reader.clone();
        let reopenings = self.reopenings_of_a_gpu_not_opened;
        let called_off = Arc::new(AtomicBool::new(false));
        let called_off_seen_by_the_thread = Arc::clone(&called_off);
        let discrete_gpus_opened_by_the_thread = discrete_gpus_to_open.clone();
        let causes_of_each_start_seen_by_the_thread = causes_of_each_start.clone();
        let (outcome_sender, outcome_receiver) = channel();
        let started = self.opening_thread_started(move || {
            let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(
                "开卡那一步只在开卡线程上锁着用，开一次卡里的 panic 在锁里就接住了：锁没被毒化",
            );
            let outcome = gpus_opened_at_a_switch(
                &mut *opening_of_a_gpu,
                discrete_gpus_opened_by_the_thread,
                wait_limit,
                reopenings,
                causes_of_each_start_seen_by_the_thread,
                &called_off_seen_by_the_thread,
                &memory_leaked_in_this_process,
                &failed_openings_in_this_mount,
                &pci_state_reader,
            );
            drop(opening_of_a_gpu);
            // 送不出去只有一种：后台 GPU 已经丢了（卸载）或切到了 `cpu`，开出来的卡没人要，在这里照常丢。
            let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);
        });
        let error = match started {
            Ok(opening_thread) => {
                self.opening_at_a_switch = Some(OpeningAtASwitch::UnderWay {
                    handing_in_number,
                    outcome_receiver,
                    opening_thread,
                    called_off,
                });
                return;
            }
            Err(error) => error,
        };
        let first_device_to_open = discrete_gpus_to_open
            .first()
            .map(|(discrete_gpu, _memory_limit)| discrete_gpu.identifier.clone())
            .expect("只为点到没开的卡的那一组开卡：至少一张");
        let mut causes_of_each_start = causes_of_each_start;
        causes_of_each_start.push(GpuFailureCause::OpeningThreadNotStarted {
            reason: format!("起不了切换开卡的那条线程：{error}"),
        });
        self.failed_openings_in_this_mount
            .note_a_failed_opening(&first_device_to_open);
        if self
            .failed_openings_in_this_mount
            .failed_openings_of(&first_device_to_open)
            < reopenings.openings_at_most()
        {
            self.opening_at_a_switch = Some(OpeningAtASwitch::ThreadNotStarted {
                handing_in_number,
                discrete_gpus_to_open,
                wait_limit,
                causes_of_each_start,
                not_started_at: Instant::now(),
            });
            return;
        }
        self.opening_at_a_switch = Some(OpeningAtASwitch::EndedWithoutAThread {
            handing_in_number,
            outcome: OutcomeOfTheOpeningAtASwitch {
                opened: Vec::new(),
                not_opened: Some(GpuNotOpenedAfterTheReopenings {
                    pci_state_when_judged_unusable: self
                        .pci_state_reader
                        .pci_state_of_the_gpu(&first_device_to_open),
                    device: first_device_to_open,
                    causes_of_each_opening: causes_of_each_start,
                }),
            },
        });
    }

    /// 关掉 `is_closed` 判成要关的开着的卡（第 2 条「切换之后不再参与的卡，切换生效时关掉」）：它从开着的那几张挪回没开的那几张（卡清单照列，
    /// 之后再点它就重开），设备丢掉；上面还记着一次等超过上限、没在 GPU 上做完的提交的那张不丢、整张泄漏，这一刻占着的字节记进这个进程的泄漏账
    /// （同卸载，`gpus_of_this_host::gpus_dropped_leaking_those_still_running`），并累计进对外读得到的那一份
    /// （[`Self::gpus_leaked_when_closed_at_a_switch`]）。参与的卡照它们的标识在剩下的卡里重找位置，次序不变。
    fn close_the_cards(&mut self, is_closed: impl Fn(&GpuDeviceIdentifier) -> bool) {
        let participating_devices: Vec<GpuDeviceIdentifier> = self
            .participating
            .positions_on_this_host()
            .iter()
            .map(|position| {
                self.cards_on_this_host[*position]
                    .discrete_gpu()
                    .identifier
                    .clone()
            })
            .collect();
        let mut cards_kept_open = Vec::new();
        let mut cards_closed = Vec::new();
        for card in std::mem::take(&mut self.cards_on_this_host) {
            if is_closed(&card.discrete_gpu().identifier) {
                self.discrete_gpus_after_the_opening
                    .note_the_gpu_closed_at_a_switch(&card.discrete_gpu().identifier);
                cards_closed.push(card);
            } else {
                cards_kept_open.push(card);
            }
        }
        self.cards_on_this_host = cards_kept_open;
        let gpus_leaked = gpus_dropped_leaking_those_still_running(
            cards_closed,
            card_has_a_submission_running_past_its_wait,
            card_leaked_when_dropped,
        );
        self.memory_leaked_in_this_process
            .note_the_gpus_leaked_when_dropped(&gpus_leaked);
        self.gpus_leaked_when_closed_at_a_switch
            .note_the_gpus_leaked(&gpus_leaked);
        self.participating = ParticipatingGpusOnThisHost::of_the_positions(
            participating_devices
                .iter()
                .filter_map(|device| {
                    self.cards_on_this_host
                        .iter()
                        .position(|card| card.discrete_gpu().identifier == *device)
                })
                .collect(),
        );
    }

    /// 换上 `settings`（它点的卡都开着）：不再点名的开着的卡关掉（[`Self::close_the_cards`]），参与的卡换成它点的那几张、按它的次序，
    /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。
    /// 显存上限（扣掉卡上已占的）连最小的一片都装不下的卡不用、报出来（[`Self::gpus_left_out_under_their_memory_limit`]）；开卡时因显存上限没量代价、
    /// 或上一组设置因显存上限晾下而这一组给了装得下的上限的卡，先等账上还在跑的那次提交做完、放账，再照新上限量一次（量不量照旧、量出的代价不用来分活），
    /// 等不到或量出错的这张卡不参与，之后每一批报成它的 GPU 出错（[`BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch`]，出错的原因是判卡用不了的那一种时
    /// 带出错那一刻读的它的 PCI 状态），直到下一次换设置。
    /// 请回来的卡照参与的次序重新参与分活。
    fn put_the_settings_in_effect(&mut self, settings: &AdmittedGpuSettings) {
        let devices_named_by_the_settings: Vec<GpuDeviceIdentifier> = settings
            .participating_devices()
            .iter()
            .map(|participating| participating.device.clone())
            .collect();
        self.close_the_cards(|device| !devices_named_by_the_settings.contains(device));
        self.devices_named_by_the_settings_in_effect = devices_named_by_the_settings;
        let identifiers_on_this_host = self.gpus_opened();
        let participating_by_the_settings =
            ParticipatingGpusOnThisHost::of_the_settings(settings, &identifiers_on_this_host);
        for card in &mut self.cards_on_this_host {
            let memory_limit = memory_limit_of_the_device_under_the_settings(
                settings,
                &card.discrete_gpu().identifier,
            );
            card.budget_mut()
                .use_the_settings_from_the_next_batch(memory_limit, settings.wait_limit());
        }
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
        let devices_left_out_under_the_settings_before: Vec<GpuDeviceIdentifier> = self
            .left_out_under_their_memory_limit
            .iter()
            .map(|left_out| left_out.device.clone())
            .collect();
        let mut left_out_under_their_memory_limit = Vec::new();
        let mut cost_not_measured = None;
        let mut positions_left_out = Vec::new();
        for position in participating_by_the_settings.positions_on_this_host() {
            let card = &mut self.cards_on_this_host[*position];
            if let Some(left_out) = gpu_left_out_under_its_memory_limit(
                &card.discrete_gpu().identifier,
                card.budget().memory_account().memory_limit(),
                smallest_piece_bytes,
                card.budget().memory_account().bytes_held(),
            ) {
                left_out_under_their_memory_limit.push(left_out);
                positions_left_out.push(*position);
                continue;
            }
            // 第 3 条「挂着切换给了它装得下的上限，从下一批起照新上限按卡的顺序重新参与分活」：上一组设置因显存上限晾下的卡请回来时照新上限再量一次代价
            // （参与的卡都报得出量出的代价，[`Self::participating_gpus`]），量出的代价不用来分活，量不成照旧不参与、报出来。量之前先在这一组给的等的上限之内等账上还在跑的那次提交（前一次等超过上限、或前一次量代价超时留下的）
            // 做完、放账（同交活之前那一等）：量代价那一段要记账，账上记着还在跑的提交时不交；等不到就照量代价出错办。
            let is_brought_back_after_being_left_out = devices_left_out_under_the_settings_before
                .contains(&card.discrete_gpu().identifier);
            if card.measured_cost().is_none() || is_brought_back_after_being_left_out {
                let measured = card
                    .room_once_the_submission_running_past_its_wait_completed()
                    .and_then(|_room_left| card.measure_the_cost_of_one_submission());
                if let Err(cause) = measured {
                    positions_left_out.push(*position);
                    // 量代价那几次提交出的错与一批里的同样判：卡当场报出错、等超时用完是判它用不了的时刻，当场读一次（D26（后台整理与放置回收） 已定项 13 第 4 条）；
                    // 只报头一张出错的卡，后面的不读。
                    if cost_not_measured.is_none() {
                        let device = card.discrete_gpu().identifier.clone();
                        let pci_state_when_judged_unusable = self
                            .pci_state_reader
                            .pci_state_if_the_cause_judges_the_gpu_unusable(&device, &cause);
                        cost_not_measured = Some(GpuWhoseCostWasNotMeasuredAtTheSwitch {
                            device,
                            cause,
                            pci_state_when_judged_unusable,
                        });
                    }
                }
            }
        }
        self.participating = participating_by_the_settings
            .leaving_out(|position| positions_left_out.contains(&position));
        self.left_out_under_their_memory_limit = left_out_under_their_memory_limit;
        self.cost_not_measured_at_the_last_switch = cost_not_measured;
    }
}
