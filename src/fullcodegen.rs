use cranelift::prelude::*;
use cranelift_module::Module;
use cranelift_module::{DataContext, Linkage};
use cranelift_simplejit::*;
pub struct FullCodegen {
    pub module: Module<SimpleJITBackend>,
    pub builder_ctx: FunctionBuilderContext,
    pub ctx: codegen::Context,
    pub data_ctx: DataContext,
}

impl FullCodegen {
    pub fn new() -> Self {
        let mut flags_builder = cranelift::codegen::settings::builder();
        flags_builder.set("opt_level", "speed_and_size").unwrap();
        flags_builder.set("use_colocated_libcalls", "true").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder.finish(settings::Flags::new(flags_builder));
        let builder = SimpleJITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = Module::new(builder);
        Self {
            builder_ctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
        }
    }
}
