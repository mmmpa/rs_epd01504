use tokio;

pub async fn delay_for(n: u64) {
    tokio::time::delay_for(tokio::time::Duration::from_millis(n)).await;
}
