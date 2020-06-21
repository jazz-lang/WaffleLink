use super::*;

impl<'a> FullCodegenTranslator<'a> {
    pub fn load_constant(&mut self, ix: u32) -> Value {
        let c = self.bc_module.value().constants.value().at(ix as _);
        if c.is_cell() {
            unimplemented!()
        } else {
            self.builder
                .ins()
                .iconst(types::I64, unsafe { c.u.as_int64 })
        }
    }
    pub fn load_register(&mut self, ix: u8) -> Value {
        let mut flags = MemFlags::new();
        flags.set_notrap();
        let regs = self
            .builder
            .ins()
            .load(types::I64, flags, self.callframe, 0);
        let reg = self.builder.ins().load(types::I64, flags, regs, ix as i32);
        reg
    }
}
