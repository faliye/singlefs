"""task_server.py 拆出的一份：库（Store：每个线程一条连接、写在事务里、读回核签名）（总说明在 task_server.py 文件头）。"""
import os
import task_server_lib as lib
import contextlib
import hmac
import json
import secrets
import sqlite3
import threading
import time
from task_server_constants import PAUSE_META_NAME, RESTART_SURVIVING_META_PREFIX, keyed_lock


class StoreRefused(Exception):
    """Store.change / claim_attempt 判出这一次不能写（行校验不过、状态不对、取消了）：str 是为什么，kind 是哪一种。"""

    def __init__(self, kind, why):
        super().__init__(why)
        self.kind = kind


class Store:
    """库（task_server_lib.py 文件头「库」）：每个线程一条连接，写都在 write_lock 里、BEGIN IMMEDIATE（write_transaction：出事先回滚再抛，连接不留在写事务里）。
    任务的状态转换（task_server_lib.py 文件头「库」的「状态转换」）：change 在一个事务里读当前那一行、核校验、判 guard、只并进给的几栏再写，
    不拿调用方手里的旧字典整行覆盖；结了局的任务不回到别的状态。claim_attempt 把「判 queued 且没取消、分尝试号、插尝试、改任务」放进同一个事务。
    task_lock 是每条任务一把锁：起单元与取消都拿它，取消要么在起之前（没起），要么等起完再停、回读。
    锁表 task_locks 只留此刻有人拿着或在等的那几把（{任务号: [锁, 拿着与在等的个数]}）：个数回到 0 就删，任务有了结局、没人再拿它的锁之后表里没有它；
    表的大小不随跑过的任务数涨。弄坏开关 task-locks-kept 下照改前一把都不删。"""
    # build_slots：每棵树每台那一组编译目录（task_server_lib.py 文件头「编译目录」）；一行一个、command_key 那一列放树根（照它选同一棵树的组），删了的整行删掉
    SIGNED_TABLES = ("tasks", "attempts", "history", "build_slots")
    LOG_TABLES = ("operations", "alerts")
    # (task_id, number) 与 (parent_id, role, shard_index) 两条业务唯一（TS-02、TS-03）：照正文里的那几栏建索引；task_id 是 0 的尝试（自证记进库的那几次）不归任务、不判
    UNIQUE_INDEXES = (
        "CREATE UNIQUE INDEX IF NOT EXISTS attempts_task_number ON attempts(task_id, json_extract(record, '$.number')) WHERE task_id > 0",
        "CREATE UNIQUE INDEX IF NOT EXISTS tasks_child_identity ON tasks(json_extract(record, '$.parent_id'), json_extract(record, '$.role'), "
        "json_extract(record, '$.shard_index')) WHERE json_extract(record, '$.parent_id') > 0",
    )
    # 查得勤的几种照列建索引（task_server_lib.py 文件头「库」的索引）：按关联列 task_id 取一条任务的尝试与一件的子任务（attempts_of、children_of），
    # 按命令键取结束了的尝试（recorded_measurements、上次运行时长、占盘预测），按会话取任务；没有它们每一次都是整表扫。弄坏开关 lookup-indexes-skipped 下不建（建过的照留）
    LOOKUP_INDEXES = (
        "CREATE INDEX IF NOT EXISTS attempts_task ON attempts(task_id)",
        "CREATE INDEX IF NOT EXISTS tasks_task ON tasks(task_id)",
        "CREATE INDEX IF NOT EXISTS attempts_key_state ON attempts(command_key, state, id)",
        "CREATE INDEX IF NOT EXISTS tasks_session ON tasks(session, state)",
    )

    def __init__(self, path, key, environment):
        self.path = path
        self.key = key
        self.environment = environment
        self.local = threading.local()
        self.write_lock = threading.Lock()
        self.task_locks = {}
        self.task_locks_guard = threading.Lock()
        self.write_generation = 0   # 本进程写过几行（write_row、delete_row、delete_attempt 各涨一）：running_attempts 的那一份照它与 PRAGMA data_version 作废
        self.tag = ""
        with self.write_lock:
            self.initialize()

    def connection(self):
        connection = getattr(self.local, "connection", None)
        if connection is None:
            connection = sqlite3.connect(self.path, timeout=30, isolation_level=None, check_same_thread=True)
            connection.execute("PRAGMA journal_mode=WAL")
            connection.execute("PRAGMA busy_timeout=30000")
            self.local.connection = connection
        return connection

    def initialize(self):
        connection = self.connection()
        connection.execute("BEGIN IMMEDIATE")
        connection.execute("CREATE TABLE IF NOT EXISTS meta(name TEXT PRIMARY KEY, value TEXT NOT NULL)")
        for table in self.SIGNED_TABLES:
            connection.execute(f"CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, task_id INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL, "
                               "session TEXT NOT NULL, command_key TEXT NOT NULL, record TEXT NOT NULL, signature TEXT NOT NULL)")
            connection.execute(f"CREATE INDEX IF NOT EXISTS {table}_state ON {table}(state)")
        for table in self.LOG_TABLES:
            connection.execute(f"CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT, record TEXT NOT NULL, signature TEXT NOT NULL)")
        for statement in self.UNIQUE_INDEXES:
            try:
                connection.execute(statement)
            except sqlite3.IntegrityError as error:
                connection.execute("ROLLBACK")
                raise lib.Unavailable(f"库 {self.path} 里已有违反业务唯一的行（同一任务同一次号的两次尝试，或同一父任务同一步同一片的两个子任务）：{error}",
                                      "停着服务时 python3 task-server/control clear-queue（先备份库）清掉任务与尝试，再起；这一版不认这种库") from None
        if not lib.broken("lookup-indexes-skipped", self.environment):
            for statement in self.LOOKUP_INDEXES:
                connection.execute(statement)
        row = connection.execute("SELECT value FROM meta WHERE name = 'tag'").fetchone()
        if row is None:
            self.tag = secrets.token_hex(4)
            connection.execute("INSERT INTO meta(name, value) VALUES('tag', ?)", (self.tag,))
        else:
            self.tag = row[0]
        connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES('health', '0')")
        connection.execute("COMMIT")
        for suffix in ("", "-wal", "-shm"):
            with contextlib.suppress(FileNotFoundError):
                os.chmod(self.path + suffix, 0o600)

    def signature(self, table, row_id, state, session, command_key, record_text):
        return lib.row_signature(self.key, table, row_id, state, session, command_key, record_text)

    def verified(self, table, row):
        """row 是 (id, state, session, command_key, record, signature, task_id)：校验列对得上、正文与几列对得上才交 True。
        决定关联的那一列 task_id（attempts_of、children_of 照它挑）不在签名里，读的时候与正文的 task_id 比，对不上就是改过（TS-05）；
        子任务的 parent_id 另要与 task_id 相同。弄坏开关 association-column-unchecked 下不比（改前的样子）。"""
        if lib.broken("signature-unchecked", self.environment):
            return True
        row_id, state, session, command_key, record_text, given, column_task_id = row
        if not hmac.compare_digest(given or "", self.signature(table, row_id, state, session, command_key, record_text)):
            return False
        try:
            record = json.loads(record_text)
        except ValueError:
            return False
        if not (isinstance(record, dict) and record.get("id") == row_id and record.get("state", "") == state and record.get("session", "") == session
                and record.get("command_key", "") == command_key):
            return False
        if table in self.SIGNED_TABLES and not lib.broken("association-column-unchecked", self.environment):
            if (record.get("task_id") or 0) != (column_task_id or 0):
                return False
            if table == "tasks" and record.get("parent_id") and record.get("parent_id") != record.get("task_id"):
                return False
        return True

    def decoded(self, table, row):
        try:
            record = json.loads(row[4])
        except ValueError:
            record = {}
        if not isinstance(record, dict):
            record = {}
        record["id"] = row[0]   # 校验不过的行正文里的 id 不可信：一律取行号，照它写回、收尾都落在这一行上
        return record, self.verified(table, row)

    @contextlib.contextmanager
    def write_transaction(self):
        """一个写事务（调用方拿着 write_lock）：BEGIN IMMEDIATE，with 里那一段做完 COMMIT；那一段或 COMMIT 抛了任何异常（SQLITE_BUSY、盘满、IO 错、StoreRefused、
        KeyboardInterrupt 都算）先 ROLLBACK（SQLite 已自己回滚了的再回滚报错，不盖住原来那个异常）再照原样抛。连接不留在没结束的写事务里：
        留着的话这一条线程下回 BEGIN 报「事务里不能再开事务」，别的连接等满 busy_timeout 报 locked，整个服务写不进库。
        弄坏开关 store-transaction-left-open 下出事不回滚。"""
        connection = self.connection()
        connection.execute("BEGIN IMMEDIATE")
        try:
            yield connection
            connection.execute("COMMIT")
        except BaseException:
            if not lib.broken("store-transaction-left-open", self.environment):
                with contextlib.suppress(sqlite3.Error):
                    connection.execute("ROLLBACK")
            raise

    def insert(self, table, record):
        with self.write_lock, self.write_transaction() as connection:
            if table in self.SIGNED_TABLES:
                cursor = connection.execute(f"INSERT INTO {table}(task_id, state, session, command_key, record, signature) VALUES(?, '', '', '', '{{}}', '')",
                                            (record.get("task_id", 0),))
            else:
                cursor = connection.execute(f"INSERT INTO {table}(record, signature) VALUES('{{}}', '')")
            record["id"] = cursor.lastrowid
            self.write_row(connection, table, record)
        return record

    def update(self, table, record, force=False):
        """整行写（尝试、钉值这类行）。任务行改用 change，尝试行只改一两栏的（记编译目录已删这类）同样用 change；这里另守两条：库里已结了局的任务不让写回别的状态，
        库里已是 ended 的尝试不让写回别的状态（HTTP 线程手里先读的那份 running 的字典，在对账记了 ended、结局与峰值之后整行写回，会把它们抹掉、尝试永远算在跑），
        抛 StoreRefused("finished") 与 StoreRefused("ended")；弄坏开关 attempt-row-overwritten 下尝试那一条不守
        （force 只给弄坏开关 cancel-race-unguarded 照改前整行覆盖的那一支）。"""
        with self.write_lock, self.write_transaction() as connection:
            if table == "tasks" and record.get("state") != "finished" and not force:
                current = connection.execute("SELECT state FROM tasks WHERE id = ?", (record["id"],)).fetchone()
                if current is not None and current[0] == "finished":
                    raise StoreRefused("finished", f"任务 {record['id']} 已结了局，不写回 {record.get('state')}")
            if (table == "attempts" and record.get("state") != "ended" and not force
                    and not lib.broken("attempt-row-overwritten", self.environment)):
                current = connection.execute("SELECT state FROM attempts WHERE id = ?", (record["id"],)).fetchone()
                if current is not None and current[0] == "ended":
                    raise StoreRefused("ended", f"尝试 {record['id']} 已 ended，不写回 {record.get('state')}")
            self.write_row(connection, table, record)

    def read_row(self, connection, table, row_id):
        """事务里读一行：→ (正文, 校验过没有)；不在交 (None, False)。"""
        row = connection.execute(f"SELECT id, state, session, command_key, record, signature, task_id FROM {table} WHERE id = ?", (row_id,)).fetchone()
        return self.decoded(table, row) if row is not None else (None, False)

    def change(self, table, row_id, changes, guard=None, reopen=False):
        """在一个事务里读当前那一行、核校验、判 guard（交空串放行，交一句就不写、抛 StoreRefused("guard", 那一句)）、只并进 changes 那几栏、写回；→ 写好的正文。
        行不在抛 StoreRefused("missing")，校验不过抛 StoreRefused("tampered")；结了局的任务要写回别的状态（reopen 没给）抛 StoreRefused("finished")。
        changes 可以是函数（拿当前正文、交要并进的几栏）。"""
        with self.write_lock, self.write_transaction() as connection:
            record, valid = self.read_row(connection, table, row_id)
            if record is None:
                raise StoreRefused("missing", f"{table} 里没有 {row_id}")
            if not valid:
                raise StoreRefused("tampered", f"{table} 的 {row_id} 校验不过")
            why = guard(record) if guard is not None else ""
            if why:
                raise StoreRefused("guard", why)
            fields = changes(record) if callable(changes) else changes
            was_finished = record.get("state") == "finished"
            record.update(fields)
            if table == "tasks" and was_finished and record.get("state") != "finished" and not reopen:
                raise StoreRefused("finished", f"任务 {row_id} 已结了局，不回到 {record.get('state')}")
            self.write_row(connection, table, record)
        return record

    def claim_attempt(self, task_id, number, attempt, task_changes, between=None, split=False):
        """起一次尝试的那一个事务（TS-01、TS-03）：读任务、判 queued 且没取消、号是 attempt_count + 1、父任务（有的话）还没结局，插尝试、改任务，一起提交。
        父任务记了 cancel_requested（整件正在取消）的子任务同样不起。
        判不过抛 StoreRefused（kind 是 missing、tampered、cancelled、state、number、parent-finished、parent-cancelling）；between 在插了尝试、改任务之前调（自证的崩溃点）。
        split（弄坏开关 attempt-claim-split）照改前分两次提交：先插尝试、提交，再改任务。→ (尝试正文, 改过的任务正文)。"""
        with self.write_lock:
            connection = self.connection()
            connection.execute("BEGIN IMMEDIATE")
            try:
                task, valid = self.read_row(connection, "tasks", task_id)
                if task is None:
                    raise StoreRefused("missing", f"没有任务 {task_id}")
                if not valid:
                    raise StoreRefused("tampered", f"任务 {task_id} 的记录校验不过")
                if task.get("cancel_requested"):
                    raise StoreRefused("cancelled", f"任务 {task_id} 判放得下之后、起之前被取消了")
                if task.get("state") != "queued":
                    raise StoreRefused("state", f"任务 {task_id} 此刻是 {task.get('state')}，不是 queued")
                if int(task.get("attempt_count") or 0) + 1 != number:
                    raise StoreRefused("number", f"任务 {task_id} 已起过 {task.get('attempt_count')} 次，这一次的号 {number} 对不上")
                parent_id = task.get("parent_id")
                if parent_id:
                    parent, parent_valid = self.read_row(connection, "tasks", parent_id)
                    if parent is None or not parent_valid or parent.get("state") == "finished":
                        raise StoreRefused("parent-finished", f"子任务 {task_id} 的父任务 {parent_id} 已结了局（或记录校验不过），不起")
                    # 整件正在取消（cancel_sharded 先给父任务记 cancel_requested，再逐个停、撤子任务）：这时还排着的子任务不起，
                    # 不在停兄弟片腾出内存的那一会儿被排队线程起出来（admission 审核 F10）。弄坏开关 parent-cancel-unchecked-at-claim 下不判
                    if parent.get("cancel_requested") and not lib.broken("parent-cancel-unchecked-at-claim", self.environment):
                        raise StoreRefused("parent-cancelling", f"子任务 {task_id} 的父任务 {parent_id} 整件正在取消，不起")
                cursor = connection.execute("INSERT INTO attempts(task_id, state, session, command_key, record, signature) VALUES(?, '', '', '', '{}', '')", (task_id,))
                attempt = dict(attempt, id=cursor.lastrowid, task_id=task_id, number=number)
                self.write_row(connection, "attempts", attempt)
                if split:
                    connection.execute("COMMIT")
                    connection.execute("BEGIN IMMEDIATE")
                if between is not None:
                    between()
                task.update(task_changes, attempt_count=number)
                self.write_row(connection, "tasks", task)
                connection.execute("COMMIT")
            except BaseException:
                with contextlib.suppress(sqlite3.Error):
                    connection.execute("ROLLBACK")
                raise
        return attempt, task

    @contextlib.contextmanager
    def task_lock(self, task_id):
        """拿着这条任务的那把锁跑 with 里那一段（起单元、取消、恢复时续起都拿它）：锁表 task_locks 照 keyed_lock 管，最后一个放下时删掉那一项。
        弄坏开关 task-locks-kept 下放下也不删。"""
        with keyed_lock(self.task_locks, self.task_locks_guard, task_id, keep=lib.broken("task-locks-kept", self.environment)):
            yield

    def write_row(self, connection, table, record):
        """写一行、重签。签名表连关联列 task_id 一起照正文写：重签的那一行关联列与正文对得上（verified 的 TS-05 那一判），核得过。
        弄坏开关 tampered-rows-left-unrepaired 下不写关联列，关联列被改过的行重签之后照样核不过。
        每写一行 write_generation 涨一（running_attempts 的那一份照它作废）。"""
        self.write_generation += 1
        text = lib.canonical_json(record)
        if table in self.SIGNED_TABLES:
            state, session, command_key = record.get("state", ""), record.get("session", ""), record.get("command_key", "")
            signature = self.signature(table, record["id"], state, session, command_key, text)
            if lib.broken("tampered-rows-left-unrepaired", self.environment):
                connection.execute(f"UPDATE {table} SET state = ?, session = ?, command_key = ?, record = ?, signature = ? WHERE id = ?",
                                   (state, session, command_key, text, signature, record["id"]))
            else:
                connection.execute(f"UPDATE {table} SET task_id = ?, state = ?, session = ?, command_key = ?, record = ?, signature = ? WHERE id = ?",
                                   (record.get("task_id") or 0, state, session, command_key, text, signature, record["id"]))
        else:
            connection.execute(f"UPDATE {table} SET record = ?, signature = ? WHERE id = ?",
                               (text, self.signature(table, record["id"], "", "", "", text), record["id"]))

    def delete_row(self, table, row_id):
        with self.write_lock:
            self.write_generation += 1
            with self.write_transaction() as connection:
                connection.execute(f"DELETE FROM {table} WHERE id = ?", (row_id,))

    def count(self, table, state):
        """这张表里这个 state 的行数（/health 用：只数、不解、不核校验列）。"""
        return self.connection().execute(f"SELECT COUNT(*) FROM {table} WHERE state = ?", (state,)).fetchone()[0]

    def count_outside_session(self, table, state, session):
        """这张表里这个 state、会话列不是 session 的行数（/health 的「别的会话在跑 M 个」：没有会话的人手任务也是别的会话）。"""
        return self.connection().execute(f"SELECT COUNT(*) FROM {table} WHERE state = ? AND session != ?", (state, session)).fetchone()[0]

    def delete_attempt(self, attempt_id):
        with self.write_lock:
            self.write_generation += 1
            with self.write_transaction() as connection:
                connection.execute("DELETE FROM attempts WHERE id = ?", (attempt_id,))

    def select(self, table, where="1", parameters=(), order="id"):
        rows = self.connection().execute(f"SELECT id, state, session, command_key, record, signature, task_id FROM {table} WHERE {where} ORDER BY {order}",
                                         parameters).fetchall()
        return [self.decoded(table, row) for row in rows]

    def task(self, task_id):
        found = self.select("tasks", "id = ?", (task_id,))
        return found[0] if found else None

    def attempts_of(self, task_id):
        return self.select("attempts", "task_id = ?", (task_id,))

    def running_attempts(self):
        """此刻在起与在跑（state 是 starting 或 running）而校验过的尝试的正文：内存与盘的记账、CPU 闸的线程、被挤掉的那一句都只取这一份
        （task_server_lib.py 文件头「库」：核不过的行不认）。核不过的不在里面，由对账记 ended（Scheduler.end_tampered_attempt）。
        弄坏开关 running-attempts-unverified 下连校验不过的一起交。
        同一轮里各处要读几百遍（task_server_lib.py 文件头「库」的在跑尝试读一遍）：每个线程照自己那条连接记一份，库没变过（本进程没写过——write_generation 没涨——
        别的连接也没提交过——PRAGMA data_version 没变）就交那一份的拷贝（每条一份浅拷贝：调用方改了不写回的不染到下一次），变过才现读。
        弄坏开关 running-attempts-reread 下每次现读。"""
        unverified = lib.broken("running-attempts-unverified", self.environment)
        if lib.broken("running-attempts-reread", self.environment):
            return [record for record, valid in self.select("attempts", "state IN ('starting', 'running')") if valid or unverified]
        stamp = (self.connection().execute("PRAGMA data_version").fetchone()[0], self.write_generation)
        cached = getattr(self.local, "running_cache", None)
        if cached is None or cached[0] != stamp:
            cached = (stamp, [record for record, valid in self.select("attempts", "state IN ('starting', 'running')") if valid or unverified])
            self.local.running_cache = cached
        return [dict(record) for record in cached[1]]

    def health_write(self):
        """库写不写得进：试写一行 meta（自证的 TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE 指的文件在时当失败）。交 (写进了没有, 原因)。"""
        failing = lib.selftest_override("TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE", self.environment)
        if failing and os.path.exists(failing):
            return False, "自证造的写不进"
        try:
            with self.write_lock, self.write_transaction() as connection:
                connection.execute("UPDATE meta SET value = ? WHERE name = 'health'", (str(time.time()),))
        except sqlite3.Error as error:
            return False, f"{type(error).__name__}: {error}"
        return True, ""

    # ── 暂停态（文件头 ⑤「暂停」）：meta 表里名为 pause 的一行，正文是 {reason, since, allow_session, signature}；没有这一行就是没暂停 ──
    def pause_signature(self, reason, since, allow_session):
        """签名盖住原因、起点和放行的会话（弄坏开关 pause-allow-unsigned 下放行的会话不进签名，改它改不出校验不过）。"""
        body = {"reason": reason, "since": since}
        if not lib.broken("pause-allow-unsigned", self.environment):
            body["allow_session"] = allow_session
        return self.signature("meta", 0, PAUSE_META_NAME, "", PAUSE_META_NAME, lib.canonical_json(body))

    def pause_record(self):
        """库里记的暂停态：没暂停交 None，暂停着交 {reason, since, allow_session, valid}；签名对不上（绕过服务改过）或读不出的按暂停算、不放行谁，valid 为 False。"""
        row = self.connection().execute("SELECT value FROM meta WHERE name = ?", (PAUSE_META_NAME,)).fetchone()
        if row is None:
            return None
        try:
            stored = json.loads(row[0])
            reason, since, allow_session, given = stored["reason"], stored["since"], stored["allow_session"], stored["signature"]
            valid = (isinstance(reason, str) and isinstance(since, (int, float)) and isinstance(allow_session, str) and isinstance(given, str)
                     and hmac.compare_digest(given, self.pause_signature(reason, since, allow_session)))
        except (ValueError, KeyError, TypeError):
            valid = False
        if not valid:
            return {"reason": "暂停记录校验不过（库被绕过服务改过），按暂停算", "since": 0.0, "allow_session": "", "valid": False}
        return {"reason": reason, "since": since, "allow_session": allow_session, "valid": True}

    def write_pause(self, reason, since, allow_session):
        """记暂停态（放行的会话连同原因、起点一起签名）；换放行的会话也是整行重写。弄坏开关 pause-allow-memory-only 下放行的会话不写进库。"""
        if lib.broken("pause-allow-memory-only", self.environment):
            allow_session = ""
        value = lib.canonical_json({"reason": reason, "since": since, "allow_session": allow_session, "signature": self.pause_signature(reason, since, allow_session)})
        with self.write_lock, self.write_transaction() as connection:
            connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES(?, ?)", (PAUSE_META_NAME, value))

    def clear_pause(self):
        with self.write_lock, self.write_transaction() as connection:
            connection.execute("DELETE FROM meta WHERE name = ?", (PAUSE_META_NAME,))

    # ── 换服务不丢的调度判断（RESTART_SURVIVING_STATES）：meta 表里名为 restart-surviving:<名> 的一行，正文 {state, signature}；没有这一行就是没记过 ──
    def restart_surviving_signature(self, name, state):
        meta_name = RESTART_SURVIVING_META_PREFIX + name
        return self.signature("meta", 0, meta_name, "", meta_name, lib.canonical_json(state))

    def restart_surviving_state(self, name):
        """库里记的这一份 → (正文, 认不认)：没记过交 (None, True)；签名对得上交 (正文, True)；签名对不上（绕过服务改过）或读不出交 (None, False)。
        弄坏开关 restart-surviving-signature-unchecked 下签名对不上的也认。"""
        row = self.connection().execute("SELECT value FROM meta WHERE name = ?", (RESTART_SURVIVING_META_PREFIX + name,)).fetchone()
        if row is None:
            return None, True
        try:
            stored = json.loads(row[0])
            state, given = stored["state"], stored["signature"]
            valid = ((isinstance(given, str) and hmac.compare_digest(given, self.restart_surviving_signature(name, state)))
                     or lib.broken("restart-surviving-signature-unchecked", self.environment))
        except (ValueError, KeyError, TypeError):
            return None, False
        return (state, True) if valid else (None, False)

    def write_restart_surviving_state(self, name, state):
        """整行重写这一份（state 是能写成 JSON 的值，连同签名）。"""
        value = lib.canonical_json({"state": state, "signature": self.restart_surviving_signature(name, state)})
        with self.write_lock, self.write_transaction() as connection:
            connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES(?, ?)", (RESTART_SURVIVING_META_PREFIX + name, value))
