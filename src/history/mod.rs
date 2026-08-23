//! 历史会话读取。子模块契约（精确签名）见 docs/CONTRACT.md §2：
//! - claude: normalize_path / all_sessions / transcript(project, session_id)
//! - codex:  all_sessions / transcript(session_id)

pub mod claude;
pub mod codex;

/// 生成 UUID 字符串（v7=true 时高位为毫秒时间戳，与 codex 会话 id 风格一致；
/// 否则 v4 风格）。熵源：系统时钟纳秒 + 进程计数器（本地唯一性足够）。
pub fn new_uuid(v7: bool) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static CTR: AtomicU64 = AtomicU64::new(0);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let ms = now.as_millis() as u64;
    let mut seed = now.as_nanos() as u64 ^ CTR.fetch_add(1, Ordering::Relaxed).wrapping_mul(0x9E3779B97F4A7C15);
    let mut rnd = || {
        // xorshift64*
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed.wrapping_mul(0x2545F4914F6CDD1D)
    };
    let (hi, lo) = if v7 {
        let hi = (ms << 16) | 0x7000 | (rnd() & 0x0FFF);
        let lo = (0b10u64 << 62) | (rnd() & 0x3FFF_FFFF_FFFF_FFFF);
        (hi, lo)
    } else {
        let hi = (rnd() & 0xFFFF_FFFF_FFFF_0FFF) | 0x4000;
        let lo = (0b10u64 << 62) | (rnd() & 0x3FFF_FFFF_FFFF_FFFF);
        (hi, lo)
    };
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (hi >> 32) as u32,
        ((hi >> 16) & 0xFFFF) as u16,
        (hi & 0xFFFF) as u16,
        ((lo >> 48) & 0xFFFF) as u16,
        lo & 0xFFFF_FFFF_FFFF
    )
}

/// 当前 UTC 时间：(ISO8601 毫秒字符串, (年,月,日,时,分,秒))。
pub fn now_utc_parts() -> (String, (i64, u32, u32, u32, u32, u32)) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs() as i64;
    let ms = now.subsec_millis();
    let days = secs.div_euclid(86400);
    let sod = secs.rem_euclid(86400);
    let (h, mi, s) = ((sod / 3600) as u32, ((sod % 3600) / 60) as u32, (sod % 60) as u32);
    // Howard Hinnant civil_from_days
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if m <= 2 { y + 1 } else { y };
    let iso = format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{ms:03}Z");
    (iso, (y, m, d, h, mi, s))
}
