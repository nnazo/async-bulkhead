use super::*;
use assert_matches::assert_matches;
use tokio1 as tokio;

async fn two_calls_test_helper(bulkhead: Bulkhead) -> Result<(), BulkheadError> {
    let bulkhead_clone = bulkhead.clone();
    let handle = tokio::spawn(async move {
        let sleep_fut = tokio::time::sleep(Duration::from_millis(50));
        bulkhead_clone.limit(sleep_fut).await
    });
    let result_fut = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        bulkhead.limit(async {}).await
    };
    let (result, _) = tokio::join!(result_fut, handle);
    result
}

#[tokio::test]
pub async fn times_out() {
    let bulkhead = Bulkhead::builder().max_concurrent_calls(1).build();
    let result = two_calls_test_helper(bulkhead).await;
    assert_matches!(result, Err(BulkheadError::Timeout(_)));
}

#[tokio::test]
pub async fn doesnt_time_out() {
    let bulkhead = Bulkhead::default();
    let result = bulkhead.limit(async {}).await;
    assert_matches!(result, Ok(_));
}

#[tokio::test]
pub async fn doesnt_time_out_long() {
    let bulkhead = Bulkhead::builder()
        .max_concurrent_calls(1)
        .max_wait_duration(Duration::from_secs(2))
        .build();
    let result = two_calls_test_helper(bulkhead).await;
    assert_matches!(result, Ok(_));
}
