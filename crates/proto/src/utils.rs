use std::fmt;
use tracing::{field, Span};

pub(crate) trait Record: Sized {
  fn record_field(self, field: &'static str) -> Self;

  #[inline]
  fn record_request(self) -> Self {
    self.record_field("request")
  }

  #[inline]
  fn record_response(self) -> Self {
    self.record_field("response")
  }
}

impl<T: fmt::Debug> Record for T {
  #[inline]
  fn record_field(self, field: &'static str) -> Self {
    Span::current().record(field, &field::debug(&self));
    self
  }
}

#[inline]
pub(crate) fn record_request<T: fmt::Debug>(request: T) -> T {
  request.record_request()
}

// #[inline]
// fn record_response<T: fmt::Debug>(response: T) -> T {
//   response.record_response()
// }
