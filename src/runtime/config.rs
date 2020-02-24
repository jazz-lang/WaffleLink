use structopt::StructOpt;

#[derive(StructOpt, Clone)]
#[structopt(name = "waffle")]
pub struct Config {
    #[structopt(name = "FILE", parse(from_os_str))]
    pub main_name: Option<std::path::PathBuf>,
    #[structopt(
        long = "perm-size",
        help = "Permanent heap size (default 1024 * 8 * 2)",
        default_value = "32768"
    )]
    pub perm_size: usize,
    #[structopt(
        long = "young-size",
        help = "Young space size (default 1024 * 8 * 4)",
        default_value = "32000"
    )]
    pub young_size: usize,
    #[structopt(
        long = "old-size",
        help = "Old space size (default 1024 * 8 * 2)",
        default_value = "128000"
    )]
    pub old_size: usize,
    #[structopt(
        long = "heap-size",
        help = "Heap size (default 1024 * 8 * 4)",
        default_value = "16384"
    )]
    pub heap_size: usize,
    #[structopt(
        long = "young-threshold",
        help = "Young GC Threshold (default 8192)",
        default_value = "32768"
    )]
    pub young_threshold: usize,
    #[structopt(
        long = "mature-threshold",
        help = "Mature GC Threshold (default 8192)",
        default_value = "16384"
    )]
    pub mature_threshold: usize,
    #[structopt(long = "blocking", help = "Number of blocking scheduler threads")]
    pub blocking: Option<usize>,
    #[structopt(long = "primary", help = "Number of primary scheduler threads")]
    pub primary: Option<usize>,
    #[structopt(long = "GC workers threads")]
    pub gc_workers: Option<usize>,
    #[structopt(
        long = "gc",
        help = "GC Variant to use for process heap garbage collection.",
        default_value = "generational mark-sweep"
    )]
    pub gc: crate::heap::GCVariant,
    #[structopt(
        long = "jit-fn-min-hotness",
        help = "Minimal number of calls before JITing function",
        default_value = "10"
    )]
    pub min_hotness: usize,
    #[structopt(
        long = "jit-loop-min-hotness",
        help = "Minimal number of loop cycles before JITing loop",
        default_value = "100"
    )]
    pub loop_min_hotness: usize,
}

impl Default for Config {
    fn default() -> Self {
        /*Self {
            main_name: None,
            perm_size: 2 * 512 * 1024,
            young_size: 4 * 512 * 1024,
            old_size: 2 * 512 * 1024,
            heap_size: 4 * 512 * 1024,
            gc_threshold: 2 * 1024,
            blocking: None,
            primary: None,
            gc_workers: None,
            gc: crate::heap::GCVariant::GenerationalSemispace,
        }*/
        // TODO: Do we really need clone there?
        CONFIG.read().clone()
    }
}

use parking_lot::RwLock;

lazy_static::lazy_static!(
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::from_args());
);
