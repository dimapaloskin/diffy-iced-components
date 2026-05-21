#[macro_export]
macro_rules! group {
  ($($child:expr),* $(,)?) => {
    $crate::widgets::group::group(vec![
      $($crate::widgets::group::group_item($child)),*
    ])
  };
}
