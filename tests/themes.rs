/*

    # How this test works:

    1) generate a spantrace with `test_capture`

    2) convert the spantrace to a string

    3) load stored spantrace control to compare to spantrace string (stored in the path of `control_file_path` below)

    4) if `control_file_path` doesn't exist, generate corresponding file in the current working directory and request the user to fix the issue (see below)

    5) extract ANSI escaping sequences (of control and current spantrace)

    6) compare if the current spantrace and the control contains the same ANSI escape sequences

    7) If not, fail and show the full strings of the control and the current spantrace

    # Re-generating the control

    If the control spantrace is lost and/or it needs to be re-generated, do the following:

    1) Checkout the `color_spantrace` version from Git that you want to test against

    3) Add this test file to '/tests'

    4) If `control_file_path` exist, delete it

    5) If you now run this test, it will generate a test control file in the current working directory

    6) copy this file to `control_file_path` (see instructions that are shown)

*/

use ansi_parser::{AnsiParser, AnsiSequence, Output};
use std::{fs, path::Path};
use tracing::instrument;
use tracing_error::ErrorLayer;
use tracing_error::SpanTrace;
use tracing_subscriber::{prelude::*, registry::Registry};

#[instrument]
fn test_capture(x: u8) -> SpanTrace {
    #[allow(clippy::if_same_then_else)]
    if x == 42 {
        SpanTrace::capture()
    } else {
        SpanTrace::capture()
    }
}

#[test]
fn test_backwards_compatibility() {
    std::env::set_var("RUST_LIB_BACKTRACE", "full");
    Registry::default().with(ErrorLayer::default()).init();

    let spantrace = test_capture(42);
    let colored_spantrace = format!("{}", colorz_spantrace::colorize(&spantrace));

    let control_file_name = "theme_control.txt";
    let control_file_path = ["tests/data/", control_file_name].concat();

    // If `control_file_path` is missing, save corresponding file to current working directory, and panic with the request to move these files to `control_file_path`, and to commit them to Git. Being explicit (instead of saving directly to `control_file_path` to make sure `control_file_path` is committed to Git. These files anyway should never be missing.

    if !Path::new(&control_file_path).is_file() {
        std::fs::write(control_file_name, &colored_spantrace)
            .expect("\n\nError saving `colored_spanntrace` to a file");
        panic!("Required test data missing! Fix this, by moving '{}' to '{}', and commit it to Git.\n\nNote: '{0}' was just generated in the current working directory.\n\n", control_file_name, control_file_path);
    }

    // `unwrap` should never fail with files generated by this test
    let colored_spantrace_control =
        String::from_utf8(fs::read(control_file_path).unwrap()).unwrap();

    fn get_ansi(s: &str) -> impl Iterator<Item = AnsiSequence> + '_ {
        s.ansi_parse().filter_map(|x| {
            if let Output::Escape(ansi) = x {
                Some(ansi)
            } else {
                None
            }
        })
    }

    let colored_spantrace_ansi = get_ansi(&colored_spantrace);
    let colored_spantrace_control_ansi = get_ansi(&colored_spantrace_control);

    assert!(
        colored_spantrace_ansi.eq(colored_spantrace_control_ansi),
        "\x1b[0mANSI escape sequences are not identical to control!\n\nCONTROL:\n\n{}\n\n\n\n{:?}\n\nCURRENT:\n\n{}\n\n\n\n{:?}\n\n", &colored_spantrace_control, &colored_spantrace_control, &colored_spantrace, &colored_spantrace
        // `\x1b[0m` clears previous ANSI escape sequences
    );
}
