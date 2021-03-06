/// Prints to stderr, and then exits the process with a return code of 1.
///
/// Based off of https://github.com/rust-lang/rfcs/issues/1078.
#[macro_export]
macro_rules! exit_with_err {
    ($($arg:tt)*) => (
        {
            use std::io::prelude::*;

            if let Err(e) = write!(&mut ::std::io::stderr(), "{}\n", format_args!($($arg)*)) {
                panic!("Failed to write to stderr.\
                    \nOriginal error output: {}\
                    \nSecondary error writing to stderr: {}", format!($($arg)*), e);
            }

            ::std::process::exit(1);
        }
    )
}
