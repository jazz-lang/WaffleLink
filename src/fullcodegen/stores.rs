use super::*;
impl<'a> FullCodegenTranslator<'a> {
    pub fn store_reg(&mut self, r: u8, val: Value) {
        let mut flags = MemFlags::new();
        flags.set_notrap();
        let ptr = self
            .builder
            .ins()
            .load(types::I64, flags, self.callframe, 0);
        self.builder.ins().store(flags, val, ptr, r as i32);
    }
}
