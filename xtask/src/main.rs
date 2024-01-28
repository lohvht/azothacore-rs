fn main() -> Result<(), anyhow::Error> {
    // Prerequisite for running the `cargo xtask coverage`
    // 1. cargo install grcov
    xtaskops::tasks::main()
}
