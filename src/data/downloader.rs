use crate::data::db::get_latest_multi_asset_timestamp;
use crate::data::kline::Kline;
use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BinanceKlineTuple(
    i64,    // 0: Open time
    String, // 1: Open
    String, // 2: High
    String, // 3: Low
    String, // 4: Close
    String, // 5: Volume
    i64,    // 6: Close time
    String, // 7: Quote asset volume
    i64,    // 8: Number of trades
    String, // 9: Taker buy base asset volume
    String, // 10: Taker buy quote asset volume
    String, // 11: Ignore
);

// --- Estructuras para deserialización de Yahoo Finance API v8 ---
#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: YahooChartContent,
}

#[derive(Debug, Deserialize)]
struct YahooChartContent {
    result: Option<Vec<YahooChartResult>>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct YahooChartResult {
    timestamp: Option<Vec<i64>>,
    indicators: YahooIndicators,
}

#[derive(Debug, Deserialize)]
struct YahooIndicators {
    quote: Option<Vec<YahooQuote>>,
}

#[derive(Debug, Deserialize)]
struct YahooQuote {
    open: Option<Vec<Option<f64>>>,
    high: Option<Vec<Option<f64>>>,
    low: Option<Vec<Option<f64>>>,
    close: Option<Vec<Option<f64>>>,
    volume: Option<Vec<Option<f64>>>,
}

/// Descarga velas históricas reales de S&P 500 (SPY u otros tickers bursátiles) desde Yahoo Finance API
pub async fn download_yahoo_finance_klines(
    client: &Client,
    symbol: &str,
    range: &str,     // ej. "10y", "5y", "max"
    interval: &str,  // ej. "1d", "1wk"
) -> Result<Vec<Kline>, Box<dyn Error + Send + Sync>> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval={}&range={}",
        symbol, interval, range
    );

    let res = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .send()
        .await?;

    let text = res.text().await?;
    let chart_resp: YahooChartResponse = serde_json::from_str(&text)?;

    if let Some(err) = chart_resp.chart.error {
        return Err(format!("Yahoo Finance devolvió error: {:?}", err).into());
    }

    let results = chart_resp
        .chart
        .result
        .ok_or("No se encontraron resultados en Yahoo Finance")?;
    let first_result = results
        .first()
        .ok_or("Array de resultados vacío en Yahoo Finance")?;

    let timestamps = first_result
        .timestamp
        .as_ref()
        .ok_or("No se encontraron timestamps en la respuesta de Yahoo Finance")?;

    let quote = first_result
        .indicators
        .quote
        .as_ref()
        .and_then(|q| q.first())
        .ok_or("No se encontraron indicadores de cotización en Yahoo Finance")?;

    let opens = quote.open.as_ref();
    let highs = quote.high.as_ref();
    let lows = quote.low.as_ref();
    let closes = quote.close.as_ref();
    let volumes = quote.volume.as_ref();

    let mut klines = Vec::with_capacity(timestamps.len());
    let now_ms = Utc::now().timestamp_millis();

    for (i, &sec_ts) in timestamps.iter().enumerate() {
        let ts_ms = sec_ts * 1000;
        if ts_ms > now_ms {
            continue; // Excluir velas en curso
        }

        let close_opt = closes.and_then(|v| v.get(i).copied().flatten());
        if let Some(close) = close_opt {
            if close <= 0.0 || close.is_nan() {
                continue;
            }

            let open = opens
                .and_then(|v| v.get(i).copied().flatten())
                .unwrap_or(close);
            let high = highs
                .and_then(|v| v.get(i).copied().flatten())
                .unwrap_or(close.max(open));
            let low = lows
                .and_then(|v| v.get(i).copied().flatten())
                .unwrap_or(close.min(open));
            let volume = volumes
                .and_then(|v| v.get(i).copied().flatten())
                .unwrap_or(0.0);

            klines.push(Kline {
                timestamp: ts_ms,
                open,
                high,
                low,
                close,
                volume,
            });
        }
    }

    Ok(klines)
}

/// Descarga un único bloque de hasta 1000 velas desde Binance API con balanceo de endpoints y reintentos.
pub async fn download_binance_klines(
    client: &Client,
    symbol: &str,
    interval: &str,
    start_time: Option<i64>,
    limit: u32,
) -> Result<(Vec<Kline>, usize), Box<dyn Error + Send + Sync>> {
    let base_urls = [
        "https://api.binance.com",
        "https://api1.binance.com",
        "https://api2.binance.com",
        "https://api3.binance.com",
        "https://api-gcp.binance.com",
    ];

    let mut last_err = None;

    for &base_url in &base_urls {
        let mut url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&limit={}",
            base_url, symbol, interval, limit
        );

        if let Some(st) = start_time {
            url = format!("{}&startTime={}", url, st);
        }

        match client.get(&url).send().await {
            Ok(res) => match res.text().await {
                Ok(text_res) => match serde_json::from_str::<Vec<BinanceKlineTuple>>(&text_res) {
                    Ok(data) => {
                        let raw_len = data.len();
                        let now_ms = Utc::now().timestamp_millis();
                        let klines: Vec<Kline> = data
                            .into_iter()
                            .filter(|t| t.6 <= now_ms)
                            .map(|t| Kline {
                                timestamp: t.0,
                                open: t.1.parse().unwrap_or(0.0),
                                high: t.2.parse().unwrap_or(0.0),
                                low: t.3.parse().unwrap_or(0.0),
                                close: t.4.parse().unwrap_or(0.0),
                                volume: t.5.parse().unwrap_or(0.0),
                            })
                            .collect();
                        return Ok((klines, raw_len));
                    }
                    Err(e) => {
                        last_err = Some(format!(
                            "Error deserializando JSON Binance ({}): {}",
                            base_url, e
                        ))
                    }
                },
                Err(e) => {
                    last_err = Some(format!("Error leyendo respuesta de {}: {}", base_url, e))
                }
            },
            Err(e) => last_err = Some(format!("Error conectando a {}: {}", base_url, e)),
        }

        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    Err(last_err
        .unwrap_or_else(|| "Todos los endpoints de Binance fallaron".to_string())
        .into())
}

/// Descarga de forma PAGINADA y COMPLETA todas las velas disponibles en Binance desde su génesis
pub async fn download_all_historical_binance_klines(
    conn: &mut Connection,
    client: &Client,
    symbol: &str,
    timeframe: &str,
    from_scratch: bool,
) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let api_interval = match timeframe {
        "1D" | "1d" => "1d",
        "4H" | "4h" => "4h",
        "1H" | "1h" => "1h",
        "15M" | "15m" => "15m",
        "5M" | "5m" => "5m",
        "1M" | "1m" => "1m",
        other => other,
    };

    let mut current_ts = if from_scratch {
        1i64
    } else {
        match get_latest_multi_asset_timestamp(conn, symbol, timeframe)? {
            Some(ts) => ts + 1,
            None => 1i64,
        }
    };

    let mode_str = if current_ts == 1 {
        "GÉNESIS (Historial Completo desde inicio de cotización)"
    } else {
        "INCREMENTAL (Desde última vela cerrada)"
    };

    println!("\n  🚀 Iniciando Descarga Paginada para {} [{}]:", symbol, timeframe);
    println!("     Modo: {}", mode_str);

    let mut total_downloaded = 0usize;
    let mut batch_count = 0usize;

    loop {
        batch_count += 1;

        let (klines, raw_len) = download_binance_klines(
            client,
            symbol,
            api_interval,
            Some(current_ts),
            1000,
        )
        .await?;

        if klines.is_empty() {
            println!("  ✅ No hay más velas pendientes para {}.", symbol);
            break;
        }

        let batch_len = klines.len();
        let first_ts = klines.first().map(|k| k.timestamp).unwrap_or(0);
        let last_ts = klines.last().map(|k| k.timestamp).unwrap_or(0);

        insert_multi_asset_klines(conn, symbol, timeframe, &klines)?;
        total_downloaded += batch_len;

        let start_date = Utc
            .timestamp_millis_opt(first_ts)
            .single()
            .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let end_date = Utc
            .timestamp_millis_opt(last_ts)
            .single()
            .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "  📥 [Lote {:>2}] +{:>4} velas | Rango: {} → {} | Total Acumulado: {:>5}",
            batch_count, batch_len, start_date, end_date, total_downloaded
        );

        if raw_len < 1000 {
            println!("\n  🎉 ¡Sincronización completa alcanzada! Todas las velas históricas están guardadas.");
            break;
        }

        current_ts = last_ts + 1;
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    Ok(total_downloaded)
}

/// Inserta o actualiza un lote de velas multiactivo en SQLite
pub fn insert_multi_asset_klines(
    conn: &mut Connection,
    symbol: &str,
    timeframe: &str,
    klines: &[Kline],
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS multi_asset_klines (
            symbol TEXT NOT NULL,
            timeframe TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL,
            PRIMARY KEY (symbol, timeframe, timestamp)
        )",
        [],
    )?;

    let tx = conn.transaction()?;
    let mut inserted = 0;

    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO multi_asset_klines 
             (symbol, timeframe, timestamp, open, high, low, close, volume) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        for k in klines {
            stmt.execute(params![
                symbol,
                timeframe,
                k.timestamp,
                k.open,
                k.high,
                k.low,
                k.close,
                k.volume
            ])?;
            inserted += 1;
        }
    }

    tx.commit()?;
    Ok(inserted)
}
