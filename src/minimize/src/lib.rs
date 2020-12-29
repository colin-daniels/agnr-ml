/* ************************************************************************ **
** This file is part of rsp2, and is licensed under EITHER the MIT license  **
** or the Apache 2.0 license, at your option.                               **
**                                                                          **
**     http://www.apache.org/licenses/LICENSE-2.0                           **
**     http://opensource.org/licenses/MIT                                   **
**                                                                          **
** Be aware that not all of rsp2 is provided under this permissive license, **
** and that the project as a whole is licensed under the GPL 3.0.           **
** ************************************************************************ */

#[cfg(test)] #[macro_use] extern crate rsp2_assert_close;
#[macro_use] extern crate rsp2_util_macros;

#[macro_use] extern crate serde_derive;
#[cfg(test)] #[macro_use] extern crate serde_json;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
#[cfg(test)] #[macro_use] extern crate itertools;

pub mod test;

pub(crate) mod util;
pub mod stop_condition;
pub mod cg;
pub mod strong_ls;
pub mod hager_ls;
pub use crate::cg::cg_descent;
pub use crate::hager_ls::linesearch;
pub mod exact_ls;
pub use crate::exact_ls::linesearch as exact_ls;
pub mod numerical;
