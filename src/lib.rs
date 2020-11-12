mod fnbox;
use fnbox::FnBox;

type Job = Box<dyn FnBox + Send + 'static>;

mod message;

mod worker;

pub mod thread_pool;
