use anyhow::Result;
use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::Opts;

#[derive(Clone)]
pub struct Metrics {
    pub api_count: Counter,
    pub api_2xx: Counter,
    pub api_4xx: Counter,
    pub api_5xx: Counter,

    pub cache_hit: Counter,
    pub cache_miss: Counter,

    pub req_elapsed: Histogram,

    pub job_cleanup: Counter,
    pub job_upload: Counter,

    pub batch_inserts: Counter,
}

#[tracing::instrument]
pub fn setup(env: &str) -> Result<Metrics> {
    let r = prometheus::default_registry();

    let api_count = Counter::with_opts(Opts::new("api_count", "Count of API requests"))?;
    let api_2xx = Counter::with_opts(Opts::new("api_2xx", "Api 2XX request count"))?;
    let api_4xx = Counter::with_opts(Opts::new("api_4xx", "Api 4XX request count"))?;
    let api_5xx = Counter::with_opts(Opts::new("api_5xx", "Api 5XX request count"))?;
    r.register(Box::new(api_count.clone()))?;
    r.register(Box::new(api_2xx.clone()))?;
    r.register(Box::new(api_4xx.clone()))?;
    r.register(Box::new(api_5xx.clone()))?;

    let cache_hit = Counter::with_opts(Opts::new("cache_hit", "Cache hit count"))?;
    let cache_miss = Counter::with_opts(Opts::new("cache_miss", "Cache miss count"))?;
    r.register(Box::new(cache_hit.clone()))?;
    r.register(Box::new(cache_miss.clone()))?;

    let req_elapsed =
        Histogram::with_opts(HistogramOpts::new("req_elapsed", "Request elapsed time"))?;
    r.register(Box::new(req_elapsed.clone()))?;

    let job_cleanup = Counter::with_opts(Opts::new(
        "job_cleanup",
        "How many times the cleanup job ran",
    ))?;
    r.register(Box::new(job_cleanup.clone()))?;
    let job_upload =
        Counter::with_opts(Opts::new("job_upload", "How many times the upload job ran"))?;
    r.register(Box::new(job_upload.clone()))?;
    let batch_inserts = Counter::with_opts(Opts::new(
        "batch_inserts",
        "How many times the batch_inserts were performed",
    ))?;
    r.register(Box::new(batch_inserts.clone()))?;

    tracing::info!("Metrics setup finished");

    Ok(Metrics {
        api_count,
        api_2xx,
        api_4xx,
        api_5xx,
        cache_hit,
        cache_miss,
        req_elapsed,
        job_cleanup,
        job_upload,
        batch_inserts,
    })
}
