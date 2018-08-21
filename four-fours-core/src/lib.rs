#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simplelog;

pub use self::lib_gen::*;
pub use self::application_context::ApplicationContext;

#[macro_use]
pub(crate) mod ui;

mod native;
mod event;
mod presenter;
mod application_context;
mod util;

mod lib_gen;
