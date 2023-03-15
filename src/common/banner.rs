use tracing::info;

use crate::GIT_VERSION;

const BANNER: &str = r#"
  █████╗ ███████╗███████╗██████╗  ██████╗ ████████╗██╗  ██╗
  ██╔══██╗╚══███╔╝██╔════╝██╔══██╗██╔═══██╗╚══██╔══╝██║  ██║
  ███████║  ███╔╝ █████╗  ██████╔╝██║   ██║   ██║   ███████║
  ██╔══██║ ███╔╝  ██╔══╝  ██╔══██╗██║   ██║   ██║   ██╔══██║
  ██║  ██║███████╗███████╗██║  ██║╚██████╔╝   ██║   ██║  ██║
  ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝
                                 ██████╗ ██████╗ ██████╗ ███████╗
                                ██╔════╝██╔═══██╗██╔══██╗██╔════╝
                                ██║     ██║   ██║██████╔╝█████╗
                                ██║     ██║   ██║██╔══██╗██╔══╝
                                ╚██████╗╚██████╔╝██║  ██║███████╗
                                 ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
     Azothacore X.X.X  -  Adapted from Azerothcore - www.azerothcore.org

"#;

pub fn azotha_banner_show(application_name: &str, log_extra_info: Option<impl Fn()>) {
    info!("{} ({})", GIT_VERSION, application_name);
    info!("<Ctrl-C> to stop.\n");
    info!("{}", BANNER);

    if let Some(lei) = log_extra_info {
        lei();
    }
}
