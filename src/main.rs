#![forbid(unsafe_code)]

mod cli;
mod commands;
mod config;
mod error;
mod formatting;
mod input;
mod telegram;

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;
use crate::config::Config;
use crate::error::AppError;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => return report_error(error),
    };

    let client = match telegram::TelegramClient::new(config, cli.proxy.as_deref()).await {
        Ok(client) => client,
        Err(error) => return report_error(error),
    };

    match commands::execute(&client, cli.command).await {
        Ok(response) => {
            if let Err(error) = write_raw_stdout(&response.raw_body) {
                return report_error(error);
            }

            if response.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => report_error(error),
    }
}

fn write_raw_stdout(body: &[u8]) -> Result<(), AppError> {
    let mut stdout = io::stdout().lock();
    match stdout.write_all(body) {
        Ok(()) => stdout.flush().map_err(|_| AppError::OutputWrite),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(_) => Err(AppError::OutputWrite),
    }
}

fn report_error(error: AppError) -> ExitCode {
    eprintln!("tgpush: {error}");
    ExitCode::from(error.exit_code())
}
