use std::{collections::BTreeMap, io, path::Path};

use flagset::FlagSet;
use tracing::subscriber::set_global_default;
use tracing_appender::non_blocking as tanb;
use tracing_subscriber::{
    filter::{self as tsfil},
    fmt::{self as tsfmt},
    prelude::*,
    {self as ts},
};

use crate::configuration::{LogAppender, LogFlags, LogLevel, LogLoggerConfig};

pub struct LogGuard {
    _guards: Vec<tanb::WorkerGuard>,
}

pub struct ConsoleWriter {
    stdout: tanb::NonBlocking,
    stderr: tanb::NonBlocking,
}

/// A lock on either stdout or stderr, depending on the verbosity level
/// of the event being written.
pub enum ConsoleWriterLock {
    Stdout(tanb::NonBlocking),
    Stderr(tanb::NonBlocking),
}

impl io::Write for ConsoleWriterLock {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            ConsoleWriterLock::Stdout(lock) => lock.write(buf),
            ConsoleWriterLock::Stderr(lock) => lock.write(buf),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match self {
            ConsoleWriterLock::Stdout(lock) => lock.write_all(buf),
            ConsoleWriterLock::Stderr(lock) => lock.write_all(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            ConsoleWriterLock::Stdout(lock) => lock.flush(),
            ConsoleWriterLock::Stderr(lock) => lock.flush(),
        }
    }
}

impl tsfmt::MakeWriter<'_> for ConsoleWriter {
    type Writer = ConsoleWriterLock;

    fn make_writer(&self) -> Self::Writer {
        // We must have an implementation of `make_writer` that makes
        // a "default" writer without any configuring metadata. Let's
        // just return stdout in that case.
        ConsoleWriterLock::Stdout(self.stdout.clone())
    }

    fn make_writer_for(&self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        // Here's where we can implement our special behavior. We'll
        // check if the metadata's verbosity level is WARN or ERROR,
        // and return stderr in that case.
        if meta.level() < &tracing::Level::WARN {
            return ConsoleWriterLock::Stderr(self.stderr.clone());
        }

        // Otherwise, we'll return stdout.
        ConsoleWriterLock::Stdout(self.stdout.clone())
    }
}

pub struct ConsoleWriterOrNonBlocking(Result<ConsoleWriter, tanb::NonBlocking>);

impl tsfmt::MakeWriter<'_> for ConsoleWriterOrNonBlocking {
    type Writer = Box<dyn io::Write>;

    fn make_writer(&'_ self) -> Self::Writer {
        match &self.0 {
            Err(e) => Box::new(e.make_writer()),
            Ok(o) => Box::new(o.make_writer()),
        }
    }

    fn make_writer_for(&'_ self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        match &self.0 {
            Err(e) => Box::new(e.make_writer_for(meta)),
            Ok(o) => Box::new(o.make_writer_for(meta)),
        }
    }
}

struct ProcessedAppenderPart {
    make_writer:        ConsoleWriterOrNonBlocking,
    f_guard:            Vec<tanb::WorkerGuard>,
    name:               String,
    appender_min_level: Option<tracing::Level>,
    appender_max_level: Option<tracing::Level>,
    flags:              FlagSet<LogFlags>,
    is_console:         bool,
}

fn construct_appender_parts<P>(logs_dir: P, a: &LogAppender) -> ProcessedAppenderPart
where
    P: AsRef<Path>,
{
    match a {
        LogAppender::Console {
            // colours,
            flags,
            min_level,
            max_level,
            name,
        } => {
            let (stdout, stdout_g) = tracing_appender::non_blocking(io::stdout());
            let (stderr, stderr_g) = tracing_appender::non_blocking(io::stderr());
            ProcessedAppenderPart {
                make_writer:        ConsoleWriterOrNonBlocking(Ok(ConsoleWriter { stdout, stderr })),
                f_guard:            vec![stdout_g, stderr_g],
                name:               name.clone(),
                appender_min_level: (*min_level).into(),
                appender_max_level: (*max_level).into(),
                flags:              *flags,
                is_console:         true,
            }
        },
        LogAppender::File {
            file,
            name,
            min_level,
            max_level,
            flags,
            // mode,
        } => {
            // let dest_log_file_name = logs_dir.as_ref().join(name);
            let f = if flags.contains(LogFlags::AppendFileTimestamps) {
                tracing_appender::rolling::daily(logs_dir, file)
            } else {
                tracing_appender::rolling::never(logs_dir, file)
            };
            let (w, g) = tracing_appender::non_blocking(f);
            ProcessedAppenderPart {
                make_writer:        ConsoleWriterOrNonBlocking(Err(w)),
                f_guard:            vec![g],
                name:               name.clone(),
                appender_min_level: (*min_level).into(),
                appender_max_level: (*max_level).into(),
                flags:              *flags,
                is_console:         false,
            }
        },
    }
}

const LOGGER_ROOT: &str = "root";

fn is_target_in_logger_targets(m: &tracing::Metadata, logger_target: &str) -> bool {
    let mut t = m.target();
    loop {
        if logger_target == LOGGER_ROOT || logger_target == t {
            // always allow logger if the logger target is root, or the logger target
            // matches the target's name
            return true;
        }
        t = match t.rfind("::") {
            None => {
                return false;
            },
            Some(i) => &t[..i],
        }
    }
}

fn default_server_appenders() -> Vec<LogAppender> {
    vec![
        LogAppender::Console {
            name:      String::from("Console"),
            min_level: LogLevel::Trace,
            max_level: LogLevel::Error,
            flags:     LogFlags::AddLogLevel | LogFlags::AddLogFilter,
            // colours: vec![
            //     (Fatal, Red),
            //     (Error, Lred),
            //     (Warning, Brown),
            //     (Info, Green),
            //     (Debug, Cyan),
            //     (Trace, Magenta),
            // ],
        },
        LogAppender::File {
            name:      String::from("Server"),
            min_level: LogLevel::Trace,
            max_level: LogLevel::Error,
            flags:     LogFlags::AddLogLevel | LogFlags::AddLogFilter | LogFlags::AddLogTimestamps, // TruncateFile.into(),
            file:      String::from("Server.log"),
        },
    ]
}

fn default_server_loggers() -> Vec<LogLoggerConfig> {
    vec![LogLoggerConfig {
        name:      String::from("root"),
        min_level: LogLevel::Trace,
        max_level: LogLevel::Error,
        appenders: vec![String::from("Console"), String::from("Server")],
    }]
}

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// Then register the subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_logging<P: AsRef<Path>>(logs_dir: P, appenders: &[LogAppender], loggers: &[LogLoggerConfig]) -> LogGuard {
    let defaults = (default_server_appenders(), default_server_loggers());
    let (appenders, loggers) = if appenders.is_empty() || loggers.is_empty() {
        eprintln!("Using default configurations. Creating default loggers");
        (defaults.0.as_slice(), defaults.1.as_slice())
    } else {
        (appenders, loggers)
    };

    let mut layers = vec![];
    let mut guards = vec![];
    for a in appenders {
        let ProcessedAppenderPart {
            make_writer,
            mut f_guard,
            name,
            appender_min_level,
            appender_max_level,
            flags,
            is_console,
        } = construct_appender_parts(&logs_dir, a);
        let appender_logger_targets = loggers
            .iter()
            .filter_map(|logger_cfg| {
                if logger_cfg.appenders.contains(&name) {
                    let min_level: Option<tracing::Level> = logger_cfg.min_level.into();
                    let max_level: Option<tracing::Level> = logger_cfg.max_level.into();
                    Some((
                        logger_cfg.name.clone(),
                        (tsfil::LevelFilter::from(min_level), tsfil::LevelFilter::from(max_level)),
                    ))
                } else {
                    None
                }
            })
            .collect::<BTreeMap<_, _>>();

        let filter_fn = tsfil::filter_fn(move |m| {
            let appender_max_level = tsfil::LevelFilter::from(appender_max_level);
            let appender_min_level = tsfil::LevelFilter::from(appender_min_level);
            if !(appender_max_level..=appender_min_level).contains(m.level()) {
                return false;
            }

            appender_logger_targets
                .iter()
                .any(|(target, (target_min_level, target_max_level))| {
                    if !(*target_max_level..=*target_min_level).contains(m.level()) {
                        return false;
                    }
                    is_target_in_logger_targets(m, target)
                })
        });

        let layer = tsfmt::Layer::new()
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_ansi(is_console)
            .with_target(flags.contains(LogFlags::AddLogFilter))
            .with_level(flags.contains(LogFlags::AddLogLevel))
            .with_writer(make_writer);
        let layer = if !flags.contains(LogFlags::AddLogTimestamps) {
            layer.without_time().boxed()
        } else {
            layer.boxed()
        };
        layers.push(layer.with_filter(filter_fn));
        guards.append(&mut f_guard);
    }
    let subscriber = ts::Registry::default().with(layers).with(console_subscriber::spawn());
    set_global_default(subscriber).expect("Failed to set subscriber");
    LogGuard { _guards: guards }
}