1. For candidate C: Instance 1, checkpoint 10, transaction 5 writes a code 1 unit at location X. The write fails and retries at location Y with the same write order (instance 1, transaction 5). Both units have birth tree T, checkpoint 10, and write order (1,5). The map key for both is identical, causing a conflict.

2. For candidate A: A code 2 index node key includes tree id, level, lower key bound, and birth checkpoint. The header stores tree id and birth checkpoint but not level or lower key bound. During a scan, these missing values cannot be retrieved, making exact key reconstruction impossible.

3. For candidate G: The key requires a version number stored in a persistent table. The pointer only contains birth device, slot, and checkpoint. Entering the map requires an extra read from the version table to obtain the full key.

4. For candidate C: Same as Q1. Instance 1, checkpoint 10, transaction 5 writes a code 1 unit at location X. Write fails and retries at location Y with same write order. Both units share birth tree T, checkpoint 10, and write order (1,5), resulting in duplicate keys.

5. The per-slot version table must live on disk. If the version number wraps from 65535 to 0 and the same slot is reused in the same checkpoint, but checkpoint numbers are unique and never reused, this case is impossible. However, if the system crashes during checkpoint creation and the checkpoint number is reused (contradicting F5), two units could share the key. But since published checkpoint numbers are never reused, no valid case exists.

6. Birth location (device 4, slot 6, checkpoint 8) plus a 2 byte version number stored in the unit header. This key avoids external table reads (better than G on Q3) and prevents duplicate keys from retries (better than C on Q1). It also reconstructs correctly during scans (better than A on Q2).
