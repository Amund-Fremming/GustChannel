use tracing::level_filters::LevelFilter;
use tracing_subscriber::FmtSubscriber;

pub(crate) fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::DEBUG)
        .finish();

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        println!("Tracing error: {}", e);
    };
}
