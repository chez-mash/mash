fn main() {
    std::process::exit(mash_cli::with_main_stack(|| {
        mash_cli::main_entry(env!("MASH_BUILD_REVISION"))
    }));
}
