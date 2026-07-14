use nanospinner::Spinner;
use std::future::Future;

pub async fn with_spinner<F, T>(msg: &str, f: F) -> anyhow::Result<T>
where
    F: Future<Output = anyhow::Result<T>>,
{
    let spinner = Spinner::new(&format!("{}...", msg)).start();
    let result = f.await;
    spinner.stop();
    result
}
