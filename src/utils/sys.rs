#[macro_export]
macro_rules! fatal {
    () => {
        $crate::utils::sys::fatal(String::from("FATAL"))
    };
    ($($arg:tt)*) => {{
        $crate::utils::sys::fatal(std::format!($($arg)*))
    }};
}

pub fn fatal(msg: String) -> ! {
    {
        let mut msg = msg.as_str();
        if let Some(i) = msg.find('\n') {
            let (s, _) = msg.split_at(i);
            msg = s;
        }

        let span = tracing::span!(tracing::Level::ERROR, "sys_fatal");
        let _enter = span.enter();
        tracing::error!("{msg}");
    }

    if let Err(e) = std::io::Write::write_fmt(
        &mut std::io::stderr(),
        std::format_args!("\nFATAL: {msg}\n"),
    ) {
        panic!("failed printing to stderr: {e}");
    }

    std::process::exit(1)
}
