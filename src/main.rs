#[cfg(feature = "gui")]
mod app;
#[cfg(feature = "cli")]
mod cli;
mod game;
mod random_stream;
mod randomization;

fn main(){
    env_logger::init();

    #[cfg(feature = "gui")]
    app::run();

    #[cfg(feature = "cli")]
    cli::run();
}

// enforce exactly one of these is enabled

#[cfg(all(feature = "cli", feature = "gui"))]
compile_error!("Features 'cli' and 'gui' cannot be enabled at the same time.");

#[cfg(not(any(feature = "cli", feature = "gui")))]
compile_error!("Either feature 'cli' or 'gui' must be enabled.");
