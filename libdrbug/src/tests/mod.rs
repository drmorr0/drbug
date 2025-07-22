mod breakpoint_test;
mod process_test;
mod register_test;

use assertables::*;
use rstest::*;

use crate::prelude::*;

const LOOP_PATH: &str = "../target/debug/loop";
const WRITE_TEST_BINARY: &str = "../target/asm/reg_write";
const READ_TEST_BINARY: &str = "../target/asm/reg_read";
