use std::collections::BTreeSet;

pub struct DatabaseLoader {
    update_flags: u32,
    modules_list: BTreeSet<String>,
    auto_setup:   bool,
}

// pub fn configure<Iter: IntoIterator<Item = String>>(&mut self, init_file_name: &str, list_of_modules: Iter)
