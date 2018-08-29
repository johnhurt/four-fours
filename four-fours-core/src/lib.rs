#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate itertools;

pub use self::lib_gen::*;
pub use self::application_context::ApplicationContext;

#[macro_use]
pub(crate) mod ui;

#[macro_use]
mod model;

mod application_context;
mod event;
mod native;
mod presenter;
mod util;

mod lib_gen;
