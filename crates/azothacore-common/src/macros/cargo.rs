#[macro_export]
macro_rules! workspace_dir {
    () => {{
        let output = std::process::Command::new(env!("CARGO"))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;
        let cargo_path = std::path::Path::new(std::str::from_utf8(&output).unwrap().trim());
        cargo_path.parent().unwrap().to_path_buf()
    }};
}

/// Adds the sript modules via two different
#[macro_export]
macro_rules! add_script_modules {
    ( INCLUDE_MOD; $($module_name:tt);* ) => {
        $(
            mod $module_name;
        )*

        add_script_modules!($($module_name);*);
    };
    ( $($module_name:tt);* ) => {
        #[doc = "Runs through a run of init functions, returning early if at the first script that fails to register"]
        pub fn add_scripts() -> azothacore_common::AzResult<()> {
            $(
                $module_name::init()?;
            )*
            Ok(())
        }

        pub fn scripts() -> Vec<String> {
            vec![
                $(
                    String::from(stringify!($module_name)),
                )*
            ]
        }
    };
}
