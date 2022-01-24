use std::pin::Pin;

use cgmath::*;
use futures::Future;

pub type P2 = Point2<f64>;
pub type V2 = Vector2<f64>;

pub type Zed = i8;

pub type R = Rad<f64>;

// just like BoxFuture but not send, because this will
// be used in the single threaded browser environment
pub type SharedFuture<T> = Pin<Box<dyn Future<Output = T>>>;
pub trait SharedFutureExt: Future {}
