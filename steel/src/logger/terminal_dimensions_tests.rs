//! Process-isolated terminal tests driven by run-terminal-dimensions-tests.py.

use super::{terminal_dimensions, terminal_height, terminal_width};
use std::io::{self, BufRead, Write};

#[test]
#[ignore = "requires the isolated PTY/pipe driver and its stdin protocol"]
fn dimensions_follow_real_terminal_descriptors() -> io::Result<()> {
    let input = io::stdin();
    let mut output = io::stdout();
    let mut saw_command = false;
    for (step, line) in input.lock().lines().enumerate() {
        let line = line?;
        let fields: Vec<_> = line.split_whitespace().collect();
        match fields.as_slice() {
            ["window", columns, rows, width, height] => {
                let columns = columns.parse::<u16>().expect("driver columns");
                let rows = rows.parse::<u16>().expect("driver rows");
                assert_eq!(terminal_dimensions()?, (columns, rows));
                assert_eq!(
                    terminal_width(),
                    width.parse::<usize>().expect("driver width")
                );
                assert_eq!(
                    terminal_height(),
                    height.parse::<usize>().expect("driver height")
                );
            }
            ["error"] => {
                assert!(terminal_dimensions().is_err());
                assert_eq!((terminal_width(), terminal_height()), (80, 30));
            }
            _ => panic!("invalid terminal test protocol: {line:?}"),
        }
        writeln!(output, "STEEL_TERMINAL_OK {step}")?;
        output.flush()?;
        saw_command = true;
    }
    assert!(
        saw_command,
        "requires the isolated terminal driver protocol"
    );
    Ok(())
}
