use super::*;
#[tokio::test]
async fn negotiation_and_concurrent_request_languages_are_isolated() {
  assert_eq!(negotiate_language("fr, en;q=0.5,zh-CN;q=0.9"), "zh");
  assert_eq!(negotiate_language("zh;q=0, en-US"), "en");
  assert_eq!(negotiate_language("zh;q=NaN, en"), "en");
  let a = tokio::spawn(with_language("zh", async {
    tokio::task::yield_now().await;
    current_language()
  }));
  let b = tokio::spawn(with_language("en", async {
    tokio::task::yield_now().await;
    current_language()
  }));
  assert_eq!(a.await.unwrap(), "zh");
  assert_eq!(b.await.unwrap(), "en");
  assert_eq!(current_language(), "en");
}
