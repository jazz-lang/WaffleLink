/*
*   Copyright (c) 2020 Adel Prokurov
*   All rights reserved.

*   Licensed under the Apache License, Version 2.0 (the "License");
*   you may not use this file except in compliance with the License.
*   You may obtain a copy of the License at

*   http://www.apache.org/licenses/LICENSE-2.0

*   Unless required by applicable law or agreed to in writing, software
*   distributed under the License is distributed on an "AS IS" BASIS,
*   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*   See the License for the specific language governing permissions and
*   limitations under the License.
*/

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(unix)]
use nix::unistd::alarm as sig_alarm;

use libc;
#[cfg(not(windows))]
use libc::{SIG_DFL, SIG_ERR, SIG_IGN};

#[cfg(windows)]
const SIG_DFL: libc::sighandler_t = 0;
#[cfg(windows)]
const SIG_IGN: libc::sighandler_t = 1;
#[cfg(windows)]
const SIG_ERR: libc::sighandler_t = !0;
