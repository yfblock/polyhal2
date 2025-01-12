use log::Log;
use polyhal2_debug::println;

pub struct LogImpl;

impl Log for LogImpl {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        use log::Level;

        let file = record.module_path();
        let line = record.line();
        let color_code = match record.level() {
            Level::Error => 31u8, // Red
            Level::Warn => 93,    // BrightYellow
            Level::Info => 34,    // Blue
            Level::Debug => 32,   // Green
            Level::Trace => 90,   // BrightBlack
        };
        println!(
            "\u{1B}[{}m\
                [{}] <{}:{}> {}\
                \u{1B}[0m",
            color_code,
            record.level(),
            file.unwrap(),
            line.unwrap(),
            record.args()
        );
    }

    fn flush(&self) {}
}
