use super::kline::Kline;
use super::timeframe::{normalize_timeframe, tf_to_duration_millis};
use chrono::{DateTime, TimeZone, Utc};
pub use rusqlite::{params, Connection, Result};
use std::time::Duration;

pub fn init_candles_db(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "synchronous", "NORMAL");
    conn.busy_timeout(Duration::from_secs(60))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS klines (
            timeframe TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL,
            PRIMARY KEY (timeframe, timestamp)
        )",
        [],
    )?;
    Ok(conn)
}

pub fn get_all_klines_closed_only(conn: &Connection, tf: &str) -> Result<Vec<Kline>> {
    let tf_clean = normalize_timeframe(tf);
    let tf_duration = tf_to_duration_millis(&tf_clean);
    let now_ts = Utc::now().timestamp_millis();

    let mut stmt = conn.prepare(
        "SELECT timestamp, open, high, low, close, volume 
         FROM klines 
         WHERE timeframe = ?1 
         ORDER BY timestamp ASC",
    )?;

    let klines_iter = stmt.query_map(params![tf_clean], |row| {
        Ok(Kline {
            timestamp: row.get(0)?,
            open: row.get(1)?,
            high: row.get(2)?,
            low: row.get(3)?,
            close: row.get(4)?,
            volume: row.get(5)?,
        })
    })?;

    let mut klines = Vec::new();
    for k in klines_iter {
        let kline = k?;
        // Excluir velas en curso que no han cerrado todavía
        if kline.timestamp + tf_duration <= now_ts {
            klines.push(kline);
        }
    }

    Ok(klines)
}

pub fn get_latest_multi_asset_timestamp(
    conn: &Connection,
    symbol: &str,
    timeframe: &str,
) -> Result<Option<i64>> {
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='multi_asset_klines'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if !table_exists {
        return Ok(None);
    }

    let mut stmt = conn.prepare(
        "SELECT MAX(timestamp) FROM multi_asset_klines WHERE symbol = ?1 AND timeframe = ?2",
    )?;
    let res = stmt.query_row(params![symbol, timeframe], |r| r.get::<_, Option<i64>>(0))?;
    Ok(res)
}

pub fn print_db_summary(conn: &Connection) -> Result<()> {
    println!("\n  📊 Resumen de Velas en Base de Datos (candles.db):");

    // 1. Tabla klines (BTCUSDT nativo)
    let klines_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='klines'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    let mut count_total = 0;

    if klines_exists {
        println!("  ── Tabla: klines (BTCUSDT Primario) ──");
        let mut stmt = conn.prepare(
            "SELECT timeframe, COUNT(*), MIN(timestamp), MAX(timestamp) 
             FROM klines 
             GROUP BY timeframe 
             ORDER BY timeframe",
        )?;

        let mut rows = stmt.query([])?;
        let mut has_rows = false;

        while let Some(row) = rows.next()? {
            has_rows = true;
            let tf: String = row.get(0)?;
            let count_raw: i64 = row.get(1)?;
            let count = count_raw as usize;
            let min_ts: i64 = row.get(2)?;
            let max_ts: i64 = row.get(3)?;
            count_total += count;

            let start_str = Utc
                .timestamp_millis_opt(min_ts)
                .single()
                .map(|dt: DateTime<Utc>| dt.format("%d-%m-%Y").to_string())
                .unwrap_or_else(|| "N/A".to_string());
            let end_str = Utc
                .timestamp_millis_opt(max_ts)
                .single()
                .map(|dt: DateTime<Utc>| dt.format("%d-%m-%Y").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            println!(
                "   • {:<4} | {:>7} velas | Rango: {} → {}",
                tf, count, start_str, end_str
            );
        }

        if !has_rows {
            println!("   (Sin velas registradas)");
        }
    }

    // 2. Tabla multi_asset_klines (PAXGUSDT, SP500, etc.)
    let multi_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='multi_asset_klines'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if multi_exists {
        println!("\n  ── Tabla: multi_asset_klines (Multiactivo / PAXG / SP500) ──");
        let mut stmt = conn.prepare(
            "SELECT symbol, timeframe, COUNT(*), MIN(timestamp), MAX(timestamp) 
             FROM multi_asset_klines 
             GROUP BY symbol, timeframe 
             ORDER BY symbol, timeframe",
        )?;

        let mut rows = stmt.query([])?;
        let mut has_rows = false;

        while let Some(row) = rows.next()? {
            has_rows = true;
            let sym: String = row.get(0)?;
            let tf: String = row.get(1)?;
            let count_raw: i64 = row.get(2)?;
            let count = count_raw as usize;
            let min_ts: i64 = row.get(3)?;
            let max_ts: i64 = row.get(4)?;
            count_total += count;

            let start_str = Utc
                .timestamp_millis_opt(min_ts)
                .single()
                .map(|dt: DateTime<Utc>| dt.format("%d-%m-%Y").to_string())
                .unwrap_or_else(|| "N/A".to_string());
            let end_str = Utc
                .timestamp_millis_opt(max_ts)
                .single()
                .map(|dt: DateTime<Utc>| dt.format("%d-%m-%Y").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            println!(
                "   • {:<10} [{:<3}] | {:>7} velas | Rango: {} → {}",
                sym, tf, count, start_str, end_str
            );
        }

        if !has_rows {
            println!("   (Aún no se han descargado velas multiactivo)");
        }
    }

    if count_total == 0 {
        println!("   ⚠️ La base de datos está vacía.");
    }
    println!();
    Ok(())
}
