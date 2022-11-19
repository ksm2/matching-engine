use std::error::Error;

mod api;
mod matcher;
mod model;

fn main() -> Result<(), Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(10)
        .build()?;

    let (tx, rx) = tokio::sync::mpsc::channel(32);

    let handle = rt.spawn(api::api(tx));

    matcher::matcher(&rt, rx);
    rt.block_on(handle)?;

    Ok(())
}
