use super::record::RequestLogRecord;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Configuration for Iceberg logging.
#[derive(Debug, Clone)]
pub struct IcebergConfig {
    pub catalog_uri: String,
    pub warehouse: String,
    pub r2_endpoint: String,
    pub r2_access_key: String,
    pub r2_secret_key: String,
    pub r2_bucket: String,
    pub batch_size: usize,
    pub flush_interval: Duration,
}

impl IcebergConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Option<Self> {
        Some(Self {
            catalog_uri: std::env::var("ICEBERG_CATALOG_URI").ok()?,
            warehouse: std::env::var("ICEBERG_WAREHOUSE").ok()?,
            r2_endpoint: std::env::var("R2_ENDPOINT").ok()?,
            r2_access_key: std::env::var("R2_ACCESS_KEY_ID").ok()?,
            r2_secret_key: std::env::var("R2_SECRET_ACCESS_KEY").ok()?,
            r2_bucket: std::env::var("R2_BUCKET").ok()?,
            batch_size: std::env::var("ICEBERG_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            flush_interval: Duration::from_secs(
                std::env::var("ICEBERG_FLUSH_INTERVAL_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60),
            ),
        })
    }
}

/// Logger that writes request logs to Iceberg tables via R2 Data Catalog.
pub struct IcebergLogger {
    batch: Vec<RequestLogRecord>,
    batch_size: usize,
    flush_interval: Duration,
    last_flush: Instant,
    flush_tx: mpsc::Sender<Vec<RequestLogRecord>>,
}

impl IcebergLogger {
    /// Create a new Iceberg logger.
    ///
    /// Note: This is a stub implementation. Full Iceberg integration requires:
    /// - Connecting to R2 Data Catalog via REST Catalog API
    /// - Creating/loading Iceberg tables
    /// - Writing Parquet files to R2
    /// - Committing to Iceberg metadata
    pub async fn new(config: IcebergConfig) -> anyhow::Result<Self> {
        let (flush_tx, flush_rx) = mpsc::channel(16);

        // Spawn background flush task
        tokio::spawn(Self::flush_task(flush_rx, config.clone()));

        Ok(Self {
            batch: Vec::with_capacity(config.batch_size),
            batch_size: config.batch_size,
            flush_interval: config.flush_interval,
            last_flush: Instant::now(),
            flush_tx,
        })
    }

    /// Log a request record.
    pub fn log(&mut self, record: RequestLogRecord) {
        self.batch.push(record);

        if self.should_flush() {
            self.trigger_flush();
        }
    }

    fn should_flush(&self) -> bool {
        self.batch.len() >= self.batch_size || self.last_flush.elapsed() > self.flush_interval
    }

    fn trigger_flush(&mut self) {
        if self.batch.is_empty() {
            return;
        }

        let batch = std::mem::take(&mut self.batch);
        self.batch = Vec::with_capacity(self.batch_size);
        self.last_flush = Instant::now();

        // Non-blocking send to background task
        let _ = self.flush_tx.try_send(batch);
    }

    async fn flush_task(mut rx: mpsc::Receiver<Vec<RequestLogRecord>>, _config: IcebergConfig) {
        while let Some(batch) = rx.recv().await {
            // TODO: Implement actual Iceberg write
            // This would involve:
            // 1. Convert records to Arrow RecordBatch
            // 2. Write Parquet file to R2
            // 3. Commit to Iceberg table
            tracing::debug!("Would write {} records to Iceberg", batch.len());
        }
    }

    /// Flush any pending records.
    pub fn flush(&mut self) {
        self.trigger_flush();
    }
}

/// No-op logger for when Iceberg is not configured.
pub struct NoOpLogger;

impl NoOpLogger {
    pub fn log(&self, _record: RequestLogRecord) {
        // No-op
    }
}
