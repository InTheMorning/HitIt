fn main() -> std::process::ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    // ADR 0096 Decision 6: a failed move stops the app before it reads the
    // configuration.
    if let Err(error) = hitit::config::user_data_move::move_previous_user_data() {
        eprintln!("Error: {error:#}");
        return std::process::ExitCode::FAILURE;
    }
    if args.is_empty() {
        return if hitit::app::run_app() {
            std::process::ExitCode::SUCCESS
        } else {
            std::process::ExitCode::FAILURE
        };
    }

    #[cfg(debug_assertions)]
    if args.first().is_some_and(|arg| arg == "startup-fixture") {
        return match hitit::startup::fixture::run_cli(&args[1..]) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Error: {error:#}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    if let Err(error) = hitit::cli::run(&args) {
        eprintln!("Error: {error:#}");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
