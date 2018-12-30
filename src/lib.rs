
#[cfg(feature = "parallel")]
pub use self::async_dispatcher::AsyncDispatcher;
pub use self::builder::DispatcherBuilder;
pub use self::dispatcher::Dispatcher;
#[cfg(feature = "parallel")]
pub use self::par_seq::{Par, ParSeq, RunWithPool, Seq};

#[cfg(feature = "parallel")]
mod async_dispatcher;
mod builder;
mod dispatcher;
mod stage;
mod util;
