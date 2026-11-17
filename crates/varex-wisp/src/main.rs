// crates/varex-wisp/src/main.rs
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use varex_core::{metrics_defs, Config};
use varex_wisp::{router, AppState};

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Parser, Debug)]
#[command(version, about = "Varex interception proxy server")]
struct Args {
    #[arg(short, long, env = "VAREX_CONFIG")] config: Option<PathBuf>,
    #[arg(long, env = "VAREX_BIND")] bind: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut cfg = match &args.config { Some(p) => Config::load(p)?, None => Config::default() };
    if let Some(b) = args.bind { cfg.server.bind = b; }
    cfg.validate()?;
    init_tracing(&cfg);

    // The external `metrics` crate is unshadowed now that the local
    // module is `metrics_defs`, so this resolves correctly.
    let recorder = metrics_exporter_prometheus::PrometheusBuilder::new().build_recorder();
    metrics_defs::describe_all();
    let handle = recorder.handle();
    metrics::set_global_recorder(recorder)?;

    let state = Arc::new(AppState::new(cfg.clone())?);
    let metrics_path = cfg.observability.metrics_path.clone();
    let app = router(state).route(
        &metrics_path,
        axum::routing::get(move || { let h = handle.clone(); async move { h.render() } }),
    );

    let addr: SocketAddr = cfg.server.bind.parse()?;
    tracing::info!(%addr, "varex-server listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal()).await?;
    Ok(())
}

fn init_tracing(cfg: &Config) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("varex=info,varex_core=info,varex_wisp=info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    if cfg.observability.json_logs { builder.json().init(); } else { builder.init(); }
}

async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("ctrl-c") };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("sigterm").recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("shutdown");
}
