use std::path::Path;

use azothacore_rs::{
    common::{banner, configuration::S_CONFIG_MGR, utils::create_pid_file},
    logging::init_logging,
    modules::{self, REGISTERED_MODULES},
    receive_signal_and_run_expr,
    server::{
        game::{scripting::ScriptMgr, world::S_WORLD},
        shared::shared_defines::{ServerProcessType, THIS_SERVER_PROCESS},
    },
    short_curcuit_unix_signal_unwrap,
    AZOTHA_CORE_CONFIG,
    CONF_DIR,
};
use clap::Parser;
use num_bigint::RandBigInt;
use rand::rngs::OsRng;
use tokio::task::{self, JoinHandle};
use tracing::{error, info};

#[cfg(target_os = "windows")]
fn signal_handler() -> JoinHandle<Result<(), std::io::Error>> {
    task::spawn(async {
        use tokio::signal::windows::ctrl_break;
        let mut sig_break = ctrl_break()?;
        receive_signal_and_run_expr!(
            S_WORLD.write().stop_now(1),
            "SIGBREAK" => sig_break
        );
    })
    .instrument(info_span!("signal_handler"))
}

#[cfg(target_os = "linux")]
fn signal_handler() -> JoinHandle<Result<(), std::io::Error>> {
    use tracing::{info_span, Instrument};

    task::spawn(
        async {
            use tokio::signal::unix::SignalKind;
            let mut sig_interrupt = short_curcuit_unix_signal_unwrap!(SignalKind::interrupt());
            let mut sig_terminate = short_curcuit_unix_signal_unwrap!(SignalKind::terminate());
            let mut sig_quit = short_curcuit_unix_signal_unwrap!(SignalKind::quit());
            receive_signal_and_run_expr!(
                S_WORLD.write().stop_now(1),
                "SIGINT" => sig_interrupt
                "SIGTERM" => sig_terminate
                "SIGQUIT" => sig_quit
            );
            Ok(())
        }
        .instrument(info_span!("signal_handler")),
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _wg = init_logging();
    {
        let mut p = THIS_SERVER_PROCESS.write();
        *p = Some(ServerProcessType::Worldserver);
    }
    let vm = ConsoleArgs::parse();
    {
        let mut s_config_mgr_w = S_CONFIG_MGR.write();
        s_config_mgr_w.set_dry_run(vm.dry_run);
        s_config_mgr_w.configure(&vm.config, REGISTERED_MODULES.map(String::from));
        s_config_mgr_w.load_app_configs()?;
    }
    // TODO: Setup logging. Original code below
    // // Init all logs
    // sLog->RegisterAppender<AppenderDB>();
    // // If logs are supposed to be handled async then we need to pass the IoContext into the Log singleton
    // sLog->Initialize(sConfigMgr->GetOption<bool>("Log.Async.Enable", false) ? ioContext.get() : nullptr);

    banner::azotha_banner_show(
        "worldserver-daemon",
        Some(|| info!("> Using configuration file       {}", S_CONFIG_MGR.read().get_filename().as_str())),
    );
    // Seed the OsRng here.
    // That way it won't auto-seed when calling OsRng and slow down the first world login
    OsRng.gen_bigint(16 * 8);

    // worldserver PID file creation
    if let Some(pid_file) = &S_CONFIG_MGR.read().world().PidFile {
        let pid = create_pid_file(pid_file)?;
        error!("Daemon PID: {}", pid);
    }
    let signal_handler = signal_handler();

    // // TODO: Follow thread pool based model? from the original core code
    // // Start the Boost based thread pool
    // int numThreads = sConfigMgr->GetOption<int32>("ThreadPool", 1);
    // std::shared_ptr<std::vector<std::thread>> threadPool(new std::vector<std::thread>(), [ioContext](std::vector<std::thread>* del)
    // {
    //     ioContext->stop();
    //     for (std::thread& thr : *del)
    //         thr.join();
    //     delete del;
    // });
    // if (numThreads < 1)
    // {
    //     numThreads = 1;
    // }
    // for (int i = 0; i < numThreads; ++i)
    // {
    //     threadPool->push_back(std::thread([ioContext]()
    //     {
    //         ioContext->run();
    //     }));
    // }

    // // TODO: Implement process priority?
    // // Set process priority according to configuration settings
    // SetProcessPriority("server.worldserver", sConfigMgr->GetOption<int32>(CONFIG_PROCESSOR_AFFINITY, 0), sConfigMgr->GetOption<bool>(CONFIG_HIGH_PRIORITY, false));

    // Loading the modules/scripts before configs as the hooks are required!
    ScriptMgr::initialise()?;

    S_CONFIG_MGR.write().load_modules_configs(false, true)?;

    start_db()?;

    signal_handler.await??;

    ScriptMgr::unload();
    info!("TERMINATING!");
    Ok(())
}

fn start_db() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: hiro@13mar2023: StartDB()
    modules::REGISTERED_MODULES
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ConsoleArgs {
    /// Dry run
    #[arg(short, long = "dry-run")]
    dry_run: bool,
    /// use <arg> as configuration file
    #[arg(short, long, default_value_t = Path::new(CONF_DIR).join(AZOTHA_CORE_CONFIG).to_str().unwrap().to_string())]
    config:  String,
    #[arg(short, long, default_value_t = String::new())]
    service: String,
}
