//! 几份单测共用的帮手。

use super::*;

pub(super) fn bytes_write(device: u32, offset: u64, bytes: Vec<u8>) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(device),
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset: DeviceOffsetInBytes(offset),
        contents: WrittenContents::Bytes(bytes),
    }
}

pub(super) fn neutral(
    device: u32,
    kind: RecordedOperationKind,
    offset: u64,
    length: u64,
    payload_start: Option<usize>,
) -> NeutralOperation {
    NeutralOperation {
        device,
        kind,
        offset,
        length,
        payload_start,
    }
}

/// 同时判的线程数：对拍里「几条线程」那一趟与连续段的单测用的，不是产品路径的线程数（产品路径 [`sampling_threads_of_this_run`]）。
pub(super) const SEVERAL_THREADS: NonZeroUsize = NonZeroUsize::new(4).expect("4 不是 0");

/// 线程对拍每批取几个取样态：够几条线程同时领到不同的连续段，一趟不跑太久。
const SAMPLED_STATES_OF_EACH_BATCH_IN_THE_PAIRING: usize = 12;

/// O2 两段的单测替身：录下的盘按引用存在内存里（判段从这里读回），判交给 `judge`；数录与判各几次。
pub(super) struct PoolCheckerTablesKeptInMemory<'judge> {
    pools: Mutex<BTreeMap<[u8; DIGEST_BYTES], MemoryPool>>,
    pub(super) judge: &'judge PoolCheckerRedNamesOfAGivenImage<'judge>,
    pub(super) tables_recorded: AtomicUsize,
    pub(super) tables_judged: AtomicUsize,
}

impl<'judge> PoolCheckerTablesKeptInMemory<'judge> {
    pub(super) fn judging_with(judge: &'judge PoolCheckerRedNamesOfAGivenImage<'judge>) -> Self {
        Self {
            pools: Mutex::new(BTreeMap::new()),
            judge,
            tables_recorded: AtomicUsize::new(0),
            tables_judged: AtomicUsize::new(0),
        }
    }

    pub(super) fn tables_recorded(&self) -> usize {
        self.tables_recorded.load(Ordering::SeqCst)
    }

    pub(super) fn tables_judged(&self) -> usize {
        self.tables_judged.load(Ordering::SeqCst)
    }
}

impl PoolCheckerTablesOfTheTwoStages for PoolCheckerTablesKeptInMemory<'_> {
    fn record_a_table(&self, table_name: &str, pool: &MemoryPool) -> RecordedTableReference {
        self.tables_recorded.fetch_add(1, Ordering::SeqCst);
        let mut message = table_name.as_bytes().to_vec();
        message.extend_from_slice(&memory_pool_digest(pool));
        let table = RecordedTableReference {
            table_name: table_name.to_string(),
            digest: sha256_digest(&message),
        };
        self.pools
            .lock()
            .expect("这把锁里不 panic")
            .insert(table.digest, pool.clone());
        table
    }

    fn judge_a_recorded_table(
        &self,
        table: &RecordedTableReference,
    ) -> Result<Vec<String>, String> {
        self.tables_judged.fetch_add(1, Ordering::SeqCst);
        let pool = self
            .pools
            .lock()
            .expect("这把锁里不 panic")
            .get(&table.digest)
            .cloned()
            .ok_or_else(|| format!("{} 没录过：先跑录入段", table.table_name))?;
        Ok((self.judge)(&table.table_name, &pool))
    }
}

/// 新池新建文件流的几样输入（写法同 `export_stream`），取样每批只留序号最大的 [`SAMPLED_STATES_OF_EACH_BATCH_IN_THE_PAIRING`] 个：
/// 批 1 留着全持久态（整条流最大的序号），按序号排与按（批号、序号）排的次序不同，对拍看得出结果排没排好。
pub(super) struct StreamToSampleInThePairing {
    pub(super) stream: PreparedStream,
    pub(super) tables: CrashStateEnumerationTables,
    pub(super) facts: FlowFacts,
    pub(super) torn_image_index_by_write: BTreeMap<usize, usize>,
    pub(super) sampled: SampledOrdinals,
}

impl StreamToSampleInThePairing {
    pub(super) fn new_pool_file_creation() -> Self {
        let stream = prepare_stream(
            StreamName::NewPoolFileCreation,
            RunSwitches::default(),
            true,
        );
        let tables = crash_state_enumeration_tables(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let facts = extract_flow_facts(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let torn_image_index_by_write: BTreeMap<usize, usize> = tables
            .torn_images
            .iter()
            .map(|row| (row.write_index, row.torn_image_index))
            .collect();
        let segment_has_a_torn_write: Vec<bool> = stream
            .segments
            .iter()
            .map(|segment| {
                segment
                    .iter()
                    .any(|write_index| tables.is_tearable_by_write[*write_index])
            })
            .collect();
        let mut sampled = sample_ordinals(
            &tables.state_ranges_by_segment,
            tables.state_count,
            &segment_has_a_torn_write,
            &|ordinal| {
                landing_vector_of(
                    &facts.persisted_writes_of_state(ordinal),
                    stream.writes.len(),
                    &torn_image_index_by_write,
                )
            },
        );
        for batch in &mut sampled.batches {
            *batch = batch
                .iter()
                .rev()
                .copied()
                .take(SAMPLED_STATES_OF_EACH_BATCH_IN_THE_PAIRING)
                .collect();
        }
        Self {
            stream,
            tables,
            facts,
            torn_image_index_by_write,
            sampled,
        }
    }

    pub(super) fn observations_on(
        &self,
        threads: NonZeroUsize,
        pool_checker_table: &PoolCheckerTableOfASampledState<'_>,
    ) -> Result<Vec<StateObservation>, ObservationsUpToACrashInAState> {
        observe_sampled_states(
            &self.stream,
            &self.tables,
            &self.facts,
            &self.torn_image_index_by_write,
            &self.sampled,
            RunSwitches::default(),
            threads,
            pool_checker_table,
        )
    }

    /// 录入段：取样、O2 要判的盘录进 `tables`（产品的录法 [`pool_checker_table_of_a_sampled_state`]）；实现自崩交回那一句。
    pub(super) fn observations_recorded_in(
        &self,
        threads: NonZeroUsize,
        tables: &dyn PoolCheckerTablesOfTheTwoStages,
    ) -> Result<Vec<StateObservation>, String> {
        self.observations_on(
            threads,
            &|stream_name, ordinal, image_digest, image, pool_checker_control| {
                pool_checker_table_of_a_sampled_state(
                    tables,
                    stream_name,
                    ordinal,
                    image_digest,
                    image,
                    pool_checker_control,
                )
            },
        )
        .map_err(|crashed| crashed.crash)
    }

    /// 录入段录完、判段判完的观测（判段与录入段同样开 `threads` 条线程）。
    pub(super) fn observations_recorded_and_judged_in(
        &self,
        threads: NonZeroUsize,
        tables: &dyn PoolCheckerTablesOfTheTwoStages,
    ) -> Vec<StateObservation> {
        let mut observations = self
            .observations_recorded_in(threads, tables)
            .expect("这条流上实现不 panic");
        match judge_the_recorded_pool_checker_tables(
            &mut observations,
            tables,
            threads,
            RunSwitches::default(),
        ) {
            Ok(JudgedTablesOfAStream::AllJudged) => observations,
            Ok(JudgedTablesOfAStream::EngineFailedInAState { what }) => {
                panic!("这条流上判器不失败：{what}")
            }
            Err(refusal) => panic!("录下的盘都读得回：{refusal}"),
        }
    }

    /// 这一份观测写出的每个文件：名字与去掉耗时之后的摘要（`E7RESULT name=file` 行报的那一个，`states.tsv`、`oracle-verdicts.tsv` 的行序进摘要）。
    fn files_written_from(
        &self,
        observations: &[StateObservation],
        threads: NonZeroUsize,
    ) -> Vec<(String, String)> {
        let directory = std::env::temp_dir().join(format!(
            "e171-sampled-states-pairing-{}-{threads}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&directory)
            .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
        let files = write_stream_files(
            &directory,
            &self.stream,
            &self.tables,
            observations,
            self.tables.state_count,
            0,
        );
        std::fs::remove_dir_all(&directory)
            .unwrap_or_else(|error| panic!("删 {}：{error}", directory.display()));
        files
            .into_iter()
            .map(|file| (file.name, file.sha256_without_timing.unwrap_or(file.sha256)))
            .collect()
    }

    /// 一条线程与几条线程各录、各判一遍：观测按（批号、序号）排好，写出的文件逐个相同。
    pub(super) fn assert_several_threads_write_the_same_files_as_one_thread(
        &self,
        tables: &dyn PoolCheckerTablesOfTheTwoStages,
    ) {
        let on_one_thread = self.observations_recorded_and_judged_in(NonZeroUsize::MIN, tables);
        let on_several_threads = self.observations_recorded_and_judged_in(SEVERAL_THREADS, tables);
        for observations in [&on_one_thread, &on_several_threads] {
            assert!(
                observations
                    .windows(2)
                    .all(|pair| (pair[0].batch, pair[0].ordinal)
                        < (pair[1].batch, pair[1].ordinal)),
                "观测按（批号、序号）严格升序"
            );
        }
        assert_eq!(
            self.files_written_from(&on_several_threads, SEVERAL_THREADS),
            self.files_written_from(&on_one_thread, NonZeroUsize::MIN),
            "几条线程与一条线程写出的文件（去掉耗时之后）逐个相同"
        );
    }
}

/// 不上卡的给定盘 O2 名单：同一张盘直接判（线程对拍比的是分线程，不是两段；两段的那一条在带 gpu 特性的对拍里）。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn pool_checker_red_names_of_a_given_image_on_the_cpu(
    _table_name: &str,
    pool: &MemoryPool,
) -> Vec<String> {
    violated_invariant_names(&check_pool_image(pool))
}

/// 造出来的 E170 坏状态每条流各几行带补丁（取已知坏镜像表开头几条的改法）。
pub(super) const PATCHED_E170_BAD_ROWS_OF_EACH_STREAM: usize = 4;

/// 造出来的 E170 坏状态每条流各几行不带补丁、落在同一张没落任何写的底态上（几条线程同时判到同一张底态）。
pub(super) const UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM: usize = 2;

/// 第 `row_position` 行造出来的 E170 坏状态的行号。
pub(super) fn e170_bad_row_identifier(row_position: usize) -> String {
    format!("row-{row_position}")
}

/// `known-bad-images` 模式走一趟留下的：退出码或 panic 的消息、结果行的全部字节、输出目录里每个文件的全部字节。
#[derive(Debug, PartialEq, Eq)]
pub(super) struct KnownBadImagesModeOutcome {
    pub(super) ending: Result<i32, String>,
    pub(super) result_lines: String,
    pub(super) files: BTreeMap<String, Vec<u8>>,
}

/// 临时目录里一份 `known-bad-images` 模式要读的导出：两条流的目录都放新池新建文件流的中性格式（模式只按落地向量重建，不看它是哪条流），
/// `states.tsv` 一行全持久态，E170 坏状态由已知坏镜像表开头几条的补丁与不带补丁的几行组成，每行的两个摘要与 E170 池级 checker 那一列由这里现算。
pub(super) struct DumpDirectoryForKnownBadImages {
    pub(super) directory: PathBuf,
    pub(super) known_bad_entry_count: usize,
}

impl DumpDirectoryForKnownBadImages {
    pub(super) fn new(scenario: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "e171-known-bad-images-{scenario}-{}",
            std::process::id()
        ));
        let stream = prepare_stream(
            StreamName::NewPoolFileCreation,
            RunSwitches::default(),
            true,
        );
        let tables = crash_state_enumeration_tables(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let mut known_bad_entry_count = 0;
        for name in STREAM_NAMES {
            let stream_directory = directory.join(name.directory_name());
            std::fs::create_dir_all(&stream_directory)
                .unwrap_or_else(|error| panic!("建 {}：{error}", stream_directory.display()));
            write_stream_files(
                &stream_directory,
                &stream,
                &tables,
                &[],
                tables.state_count,
                0,
            );
            known_bad_entry_count =
                Self::write_the_full_state_and_the_e170_bad_rows(&stream_directory);
        }
        Self {
            directory,
            known_bad_entry_count,
        }
    }

    /// 交回已知坏镜像表的条数。
    fn write_the_full_state_and_the_e170_bad_rows(stream_directory: &Path) -> usize {
        let neutral = read_neutral_stream(stream_directory);
        let write_count = neutral
            .operations
            .iter()
            .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
            .count();
        let all_persisted = "2".repeat(write_count);
        let none_landed = "0".repeat(write_count);
        let clean = rebuilt_memory_pool(&neutral, &all_persisted);
        std::fs::write(
            stream_directory.join(STATES_FILE),
            format!(
                "0\t0\t0\t1\t{all_persisted}\t{}\n",
                sha256_text(&memory_pool_digest(&clean))
            ),
        )
        .unwrap_or_else(|error| panic!("写 {STATES_FILE}：{error}"));
        let entries = known_bad_images(&clean);
        let patches_of_the_entry = |entry_index: usize| {
            let mut bad = clean.clone();
            (entries[entry_index].1)(&mut bad);
            sector_difference_as_patches(&clean, &bad)
        };
        let rows: Vec<(&str, Vec<BytePatch>)> = (0..PATCHED_E170_BAD_ROWS_OF_EACH_STREAM)
            .map(|entry_index| (all_persisted.as_str(), patches_of_the_entry(entry_index)))
            .chain(
                (0..UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM)
                    .map(|_row| (none_landed.as_str(), Vec::new())),
            )
            .collect();
        let mut states_text = String::new();
        let mut verdicts_text = String::new();
        let mut patches_file = Vec::new();
        for (row_position, (landing_vector, patches)) in rows.iter().enumerate() {
            let mut pool = rebuilt_memory_pool(&neutral, landing_vector);
            let base_digest = sha256_text(&memory_pool_digest(&pool));
            apply_patches(&mut pool, patches);
            let row_identifier = e170_bad_row_identifier(row_position);
            states_text.push_str(&format!(
                "{row_identifier}\t{landing_vector}\t{base_digest}\t{}\t{}\n",
                sha256_text(&memory_pool_digest(&pool)),
                patch_list_text(patches, &mut patches_file)
            ));
            let mut verdict_fields = vec!["-".to_string(); 10];
            verdict_fields[6] = list_text(&pool_checker_red_names_of_a_given_image_on_the_cpu(
                &row_identifier,
                &pool,
            ));
            verdicts_text.push_str(&verdict_fields.join("\t"));
            verdicts_text.push('\n');
        }
        for (file_name, contents) in [
            ("bad-states.tsv", states_text.into_bytes()),
            ("bad-patches.bin", patches_file),
            ("bad-impl-verdicts.tsv", verdicts_text.into_bytes()),
        ] {
            std::fs::write(stream_directory.join(file_name), contents)
                .unwrap_or_else(|error| panic!("写 {file_name}：{error}"));
        }
        entries.len()
    }

    pub(super) fn command_line_on(
        &self,
        threads: NonZeroUsize,
        output_directory: PathBuf,
    ) -> KnownBadImagesCommandLine {
        KnownBadImagesCommandLine {
            dump_directory: self.directory.clone(),
            output_directory,
            with_e170_bad_rows: true,
            switches: RunSwitches {
                sampling_threads: Some(threads),
                ..RunSwitches::default()
            },
        }
    }

    /// 两段先后跑一趟，O2 两段交内存里的替身（判交给 `pool_checker_red_names`）。
    pub(super) fn run_on(
        &self,
        threads: NonZeroUsize,
        pool_checker_red_names: &PoolCheckerRedNamesOfAGivenImage<'_>,
    ) -> KnownBadImagesModeOutcome {
        self.run_on_with_the_tables(
            threads,
            &PoolCheckerTablesKeptInMemory::judging_with(pool_checker_red_names),
        )
    }

    pub(super) fn run_on_with_the_tables(
        &self,
        threads: NonZeroUsize,
        tables: &dyn PoolCheckerTablesOfTheTwoStages,
    ) -> KnownBadImagesModeOutcome {
        let output_directory = self.directory.join(format!("output-on-{threads}-threads"));
        let KnownBadImagesModeOutcome {
            ending,
            result_lines,
            ..
        } = self.run_stage_on(
            RunStage::RecordThenJudge,
            threads,
            &output_directory,
            tables,
        );
        KnownBadImagesModeOutcome {
            ending,
            result_lines,
            files: Self::read_and_remove(&output_directory),
        }
    }

    /// 照 `stage` 跑一趟，交回退出码（或 panic 的消息、拒绝的那一句）与结果行；输出目录留着（[`Self::read_and_remove`] 读）。
    pub(super) fn run_stage_on(
        &self,
        stage: RunStage,
        threads: NonZeroUsize,
        output_directory: &Path,
        tables: &dyn PoolCheckerTablesOfTheTwoStages,
    ) -> KnownBadImagesModeOutcome {
        let command_line = self.command_line_on(threads, output_directory.to_path_buf());
        let mut result_lines = Vec::new();
        let ending = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_known_bad_images(
                &command_line,
                stage,
                &KNOWN_BAD_IMAGES_RUN_ARGUMENTS.map(str::to_string),
                tables,
                &mut result_lines,
            )
        }))
        .map_err(|payload| panic_text(&*payload))
        .and_then(|ran| ran);
        KnownBadImagesModeOutcome {
            ending,
            result_lines: String::from_utf8(result_lines).expect("结果行是 UTF-8"),
            files: BTreeMap::new(),
        }
    }

    /// 输出目录里每个文件的全部字节（没有这个目录是空的），读完删掉这个目录。
    pub(super) fn read_and_remove(output_directory: &Path) -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        if output_directory.exists() {
            for entry in std::fs::read_dir(output_directory)
                .unwrap_or_else(|error| panic!("读 {}：{error}", output_directory.display()))
            {
                let path = entry.expect("读得了输出目录的一项").path();
                let contents = std::fs::read(&path)
                    .unwrap_or_else(|error| panic!("读 {}：{error}", path.display()));
                files.insert(
                    path.file_name()
                        .expect("输出目录里的文件有名字")
                        .to_string_lossy()
                        .into_owned(),
                    contents,
                );
            }
            std::fs::remove_dir_all(output_directory)
                .unwrap_or_else(|error| panic!("删 {}：{error}", output_directory.display()));
        }
        files
    }
}

impl Drop for DumpDirectoryForKnownBadImages {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.directory) {
            eprintln!("删 {}：{error}", self.directory.display());
        }
    }
}

/// 坏镜像模式这几条单测的参数（进交接；两段核是不是同一条命令）。
const KNOWN_BAD_IMAGES_RUN_ARGUMENTS: [&str; 1] = ["known-bad-images"];
