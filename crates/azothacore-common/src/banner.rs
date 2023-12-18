use tracing::info;

use crate::GIT_VERSION;

const BANNER: &str = r#"
 █████╗ ███████╗ ██████╗ ████████╗██╗  ██╗ █████╗        
██╔══██╗╚══███╔╝██╔═══██╗╚══██╔══╝██║  ██║██╔══██╗       
███████║  ███╔╝ ██║   ██║   ██║   ███████║███████║       
██╔══██║ ███╔╝  ██║   ██║   ██║   ██╔══██║██╔══██║       
██║  ██║███████╗╚██████╔╝   ██║   ██║  ██║██║  ██║       
╚═╝  ╚═╝╚══════╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝       
                              
                         ██████╗ ██████╗ ██████╗ ███████╗
                        ██╔════╝██╔═══██╗██╔══██╗██╔════╝
                        ██║     ██║   ██║██████╔╝█████╗  
                        ██║     ██║   ██║██╔══██╗██╔══╝  
                        ╚██████╗╚██████╔╝██║  ██║███████╗
                         ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
                        
     Azothacore X.X.X  -  Adapted from AzerothCore (www.azerothcore.org) / TrinityCore (www.trinitycore.org)

"#;

pub fn azotha_banner_show<F>(application_name: &str, log_extra_info: F)
where
    F: Fn(),
{
    info!(
        target: "server",
        r#"{GIT_VERSION} ({application_name})
    <Ctrl-C> to stop.
    {BANNER}"#
    );

    log_extra_info();
}
