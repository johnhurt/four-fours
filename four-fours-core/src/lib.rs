#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate getset;
#[macro_use] extern crate nom;

extern crate statrs;

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
mod math;

mod lib_gen;
