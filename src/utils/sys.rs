use std::future::Future;

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

#[cfg(unix)]
pub fn shutdown_signal(
) -> std::io::Result<impl Future<Output = ()> + Send + 'static> {
    use tokio::signal::unix::{signal, SignalKind};

    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;

    Ok(async move {
        tokio::select! {
            _ = interrupt.recv() => {
                tracing::info!(target: "sys_signals", "received SIGINT");
            }
            _ = terminate.recv() => {
                tracing::info!(target: "sys_signals", "received SIGTERM");
            }
        }
    })
}

#[cfg(not(unix))]
pub fn shutdown_signal(
) -> std::io::Result<impl Future<Output = ()> + Send + 'static> {
    use futures_util::FutureExt;

    Ok(tokio::signal::ctrl_c().map(|res| match res {
        Ok(_) => {
            tracing::info!(target: "sys_signals", "received CTRL_C");
        }
        Err(_) => {
            tracing::error!(target: "sys_signals", "failed to create CTRL_C signal receiver");
        }
    }))
}
