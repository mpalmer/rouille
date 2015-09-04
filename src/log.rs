use backtrace;
use std::io::Write;
use std::thread;

use time;

use Request;

/// RAII guard that ensures that a log entry corresponding to a request will be written.
///
/// # Example
///
/// ```no_run
/// rouille::start_server("localhost:80", move |request| {
///     let _entry = rouille::LogEntry::start(std::io::stdout(), request);
///
///     // process the request here
///
/// # panic!()
/// }); // <-- the log entry is written at the end of this block
/// ```
///
pub struct LogEntry<W> where W: Write {
    line: String,
    output: W,
    start_time: u64,
}

impl<'a, W> LogEntry<W> where W: Write {
    /// Starts a `LogEntry`.
    pub fn start(output: W, rq: &Request) -> LogEntry<W> {
        LogEntry {
            line: format!("GET {}", rq.url()),       // TODO: 
            output: output,
            start_time: time::precise_time_ns(),
        }
    }
}

impl<W> Drop for LogEntry<W> where W: Write {
    fn drop(&mut self) {
        write!(self.output, "{} - ", self.line).unwrap();

        if thread::panicking() {
            writeln!(self.output, " - PANIC!").unwrap();

            let mut frame_num = 0;

            backtrace::trace(&mut |frame| {
                let ip = frame.ip();
                frame_num += 1;

                backtrace::resolve(ip, &mut |symbol| {
                    let name = String::from_utf8(symbol.name()
                                                       .unwrap_or(&b"<unknown>"[..])
                                                       .to_owned())
                                       .unwrap_or_else(|_| "<not-utf8>".to_owned());
                    let filename = String::from_utf8(symbol.filename()
                                                           .unwrap_or(&b"<unknown>"[..])
                                                           .to_owned())
                                           .unwrap_or_else(|_| "<not-utf8>".to_owned());
                    let line = symbol.lineno().map(|l| l.to_string())
                                              .unwrap_or_else(|| "??".to_owned());

                    writeln!(self.output, "{:>#4} - {:p} - {}\n       {}:{}",
                             frame_num, ip, name, filename, line).unwrap();
                });

                true
            });

        } else {
            let elapsed = time::precise_time_ns() - self.start_time;
            format_time(self.output.by_ref(), elapsed);
        }

        writeln!(self.output, "").unwrap();
    }
}

fn format_time<W>(mut out: W, time: u64) where W: Write {
    if time < 1_000 {
        write!(out, "{}ns", time).unwrap()
    } else if time < 1_000_000 {
        write!(out, "{:.1}us", time as f64 / 1_000.0).unwrap()
    } else if time < 1_000_000_000 {
        write!(out, "{:.1}ms", time as f64 / 1_000_000.0).unwrap()
    } else {
        write!(out, "{:.1}s", time as f64 / 1_000_000_000.0).unwrap()
    }
}
