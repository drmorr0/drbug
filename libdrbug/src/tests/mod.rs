mod breakpoint_test;
mod memory_test;
mod process_test;
mod register_test;
mod util;

use assertables::*;
use rstest::*;

use crate::Empty;
use crate::prelude::*;

const HELLO_PATH: &str = "../target/debug/hello";
const LOOP_PATH: &str = "../target/debug/loop";
const MEMORY_PATH: &str = "../target/debug/memory";
const READ_TEST_BINARY: &str = "../target/asm/reg_read";
const WRITE_TEST_BINARY: &str = "../target/asm/reg_write";
