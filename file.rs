file /cargo/registry/src/github.com-1ecc6299db9ec823/hashbrown-0.6.2/src/raw/mod.rs does not exist!
file /cargo/registry/src/github.com-1ecc6299db9ec823/hashbrown-0.6.2/src/map.rs does not exist!
file /cargo/registry/src/github.com-1ecc6299db9ec823/hashbrown-0.6.2/src/raw/bitmask.rs does not exist!
 pub fn run(mut frame: Frame) -> Result<Value, Value> {
 push    rbp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 sub     rsp, 408
 mov     r12, rsi
 mov     qword, ptr, [rsp, +, 192], rdi
     lea     rax, [rsi, +, 40]
     mov     qword, ptr, [rsp, +, 256], rax
     mov     qword, ptr, [rsp, +, 184], rsi
     jmp     .LBB187_4
.LBB187_1:
     mov     esi, 32
     mov     edx, 8
     mov     rdi, rbx
.LBB187_2:
     call    qword, ptr, [rip, +, __rust_dealloc@GOTPCREL]
.LBB187_3:
     mov     rax, qword, ptr, [rsp, +, 48]
     mov     qword, ptr, [rbp, +, 32], rax
     movdqu  xmm0, xmmword, ptr, [rsp, +, 16]
     movdqu  xmm1, xmmword, ptr, [rsp, +, 32]
     movdqu  xmmword, ptr, [rbp, +, 16], xmm1
     movdqu  xmmword, ptr, [rbp], xmm0
.LBB187_4:
     unsafe { &mut *self.raw } (src/common/ptr.rs:84)
     mov     r15, qword, ptr, [r12, +, 16]
.LBB187_5:
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rax, qword, ptr, [r15, +, 8]
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rcx, qword, ptr, [rax, +, 96]
 let ins = *bb.code.get_unchecked(frame.ip);
 mov     rax, qword, ptr, [r12, +, 72]
 let bb = code.get_unchecked(frame.bp);
 mov     rdx, qword, ptr, [r12, +, 80]
     intrinsics::offset(self, count) (libcore/ptr/const_ptr.rs:160)
     lea     rdx, [rdx, +, 2*rdx]
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rdi, qword, ptr, [rcx, +, 8*rdx]
 let ins = *bb.code.get_unchecked(frame.ip);
 lea     rcx, [rax, +, 2*rax]
 movzx   edx, byte, ptr, [rdi, +, 4*rcx]
 frame.ip += 1;
 add     rax, 1
 let ins = *bb.code.get_unchecked(frame.ip);
 movzx   esi, byte, ptr, [rdi, +, 4*rcx, +, 1]
 movzx   ebp, word, ptr, [rdi, +, 4*rcx, +, 2]
 movzx   ebx, bp
 mov     r13d, dword, ptr, [rdi, +, 4*rcx, +, 4]
 mov     ecx, dword, ptr, [rdi, +, 4*rcx, +, 8]
 frame.ip += 1;
 mov     qword, ptr, [r12, +, 72], rax
 let ins = *bb.code.get_unchecked(frame.ip);
 cmp     rdx, 55
 Star(r) => {
 ja      .LBB187_4
 lea     rax, [rip, +, .LJTI187_0]
 mov     rdi, rax
 movsxd  rax, dword, ptr, [rax, +, 4*rdx]
 add     rax, rdi
 jmp     rax
.LBB187_7:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rdx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rdx
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rdi, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts(self.as_ptr(), self.len) } (liballoc/vec.rs:1966)
     mov     rax, qword, ptr, [rdi, +, 56]
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     cmp     rax, rbx
     jbe     .LBB187_945
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rdi, +, 40]
     self.func.func_value_unchecked().constants[ix as usize] (src/runtime/frame.rs:119)
     mov     rax, qword, ptr, [rax, +, 8*rbx]
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rcx, qword, ptr, [rdi, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rcx, r13
     jbe     .LBB187_943
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rcx, qword, ptr, [rdi, +, 8]
 if let FeedBack::Cache(attrs, offset, misses) = feedback {
 lea     rdi, [4*r13]
 add     rdi, r13
 cmp     word, ptr, [rcx, +, 8*rdi], 1
 jne     .LBB187_4
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rbx, -562949953421312
     add     rbx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rbx
     jne     .LBB187_508
     cmp     rdx, 10
     ja      .LBB187_13
     mov     ebx, 1029
     bt      rbx, rdx
     jb      .LBB187_508
.LBB187_13:
     lea     rax, [rcx, +, 8*rdi]
     add     rax, 8
     mov     rdi, qword, ptr, [rax]
.LBB187_14:
     self.inner.as_ptr() == other.inner.as_ptr() (src/arc.rs:75)
     cmp     qword, ptr, [rdx, +, 48], rdi
 if object.attributes.ptr_eq(attrs) {
 je      .LBB187_32
 while let Some(object) = obj {
 cmp     qword, ptr, [rdx, +, 16], 1
 obj = object.prototype;
 mov     rdx, qword, ptr, [rdx, +, 24]
 while let Some(object) = obj {
 je      .LBB187_14
 jmp     .LBB187_247
.LBB187_16:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let value = *frame.r(function);
 mov     r15, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 200], r15
 let this = frame.rax;
 mov     r13, qword, ptr, [r12]
     Vec { buf: RawVec::NEW, len: 0 } (liballoc/vec.rs:324)
     mov     qword, ptr, [rsp, +, 16], 8
     pxor    xmm0, xmm0
     lea     rax, [rsp, +, 24]
     movdqu  xmmword, ptr, [rax], xmm0
     fn lt(&self, other: &$t) -> bool { (*self) < (*other) } (libcore/cmp.rs:1136)
     test    bp, bp
     if self.start < self.end { (libcore/iter/range.rs:211)
     je      .LBB187_20
.LBB187_17:
     if self.len == 0 { (liballoc/vec.rs:1225)
     mov     rax, qword, ptr, [r12, +, 56]
     test    rax, rax
     if self.len == 0 { (liballoc/vec.rs:1225)
     je      .LBB187_20
     self.len -= 1; (liballoc/vec.rs:1229)
     lea     rcx, [rax, -, 1]
     mov     qword, ptr, [r12, +, 56], rcx
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rcx, qword, ptr, [r12, +, 40]
     Some(ptr::read(self.as_ptr().add(self.len()))) (liballoc/vec.rs:1230)
     mov     rsi, qword, ptr, [rcx, +, 8*rax, -, 8]
     lea     rdi, [rsp, +, 16]
 v.push(val);
 call    alloc::vec::Vec<T>::push
     fn lt(&self, other: &$t) -> bool { (*self) < (*other) } (libcore/cmp.rs:1136)
     add     bp, -1
     if self.start < self.end { (libcore/iter/range.rs:211)
     jne     .LBB187_17
.LBB187_20:
 v
 mov     rax, qword, ptr, [rsp, +, 32]
 mov     qword, ptr, [rsp, +, 224], rax
 mov     rax, qword, ptr, [rsp, +, 16]
 mov     qword, ptr, [rsp, +, 208], rax
 mov     rax, qword, ptr, [rsp, +, 24]
 mov     qword, ptr, [rsp, +, 216], rax
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    r15, rax
     jne     .LBB187_923
     cmp     r15, 10
     ja      .LBB187_23
     mov     eax, 1029
     bt      rax, r15
     jb      .LBB187_923
.LBB187_23:
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     qword, ptr, [r15], 3
 if cell.is_function() {
 jne     .LBB187_930
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rbp, qword, ptr, [r15, +, 8]
 let args = local_data().allocate_array(arguments, &mut frame);
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process10local_data17h1a97fc2e6905fb89E@GOTPCREL]
 let args = local_data().allocate_array(arguments, &mut frame);
 mov     rcx, qword, ptr, [rsp, +, 224]
 mov     qword, ptr, [rsp, +, 32], rcx
 movdqa  xmm0, xmmword, ptr, [rsp, +, 208]
 movdqa  xmmword, ptr, [rsp, +, 16], xmm0
 mov     r14b, 1
 lea     rsi, [rsp, +, 16]
 let args = local_data().allocate_array(arguments, &mut frame);
 mov     rdi, rax
 mov     rdx, r12
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process9LocalData14allocate_array17h40b996dd3de8a21fE@GOTPCREL]
 mov     rbx, rax
 FunctionCode::Bytecode(_) => {
 cmp     qword, ptr, [rbp, +, 88], 1
 jne     .LBB187_563
 x if x >= 10_000 => {
 cmp     qword, ptr, [rbp, +, 32], 9999
 ja      .LBB187_4
 local_data().frames.push(frame);
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process10local_data17h1a97fc2e6905fb89E@GOTPCREL]
 local_data().frames.push(frame);
 mov     rcx, qword, ptr, [r12, +, 112]
 mov     qword, ptr, [rsp, +, 128], rcx
 movups  xmm0, xmmword, ptr, [r12, +, 96]
 movaps  xmmword, ptr, [rsp, +, 112], xmm0
 movups  xmm0, xmmword, ptr, [r12, +, 80]
 movaps  xmmword, ptr, [rsp, +, 96], xmm0
 movups  xmm0, xmmword, ptr, [r12, +, 64]
 movaps  xmmword, ptr, [rsp, +, 80], xmm0
 movdqu  xmm0, xmmword, ptr, [r12]
 movdqu  xmm1, xmmword, ptr, [r12, +, 16]
 movups  xmm2, xmmword, ptr, [r12, +, 32]
 movups  xmm3, xmmword, ptr, [r12, +, 48]
 movaps  xmmword, ptr, [rsp, +, 64], xmm3
 movaps  xmmword, ptr, [rsp, +, 48], xmm2
 movdqa  xmmword, ptr, [rsp, +, 32], xmm1
 movdqa  xmmword, ptr, [rsp, +, 16], xmm0
 xor     r14d, r14d
 lea     rsi, [rsp, +, 16]
 local_data().frames.push(frame);
 mov     rdi, rax
 call    alloc::vec::Vec<T>::push
 frame = Frame::new(this, func.module);
 mov     rdx, qword, ptr, [rbp, +, 120]
 xor     r14d, r14d
 lea     rdi, [rsp, +, 16]
 frame = Frame::new(this, func.module);
 mov     rsi, r13
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5frame5Frame3new17hbbd4a28f571e6db5E@GOTPCREL]
 frame = Frame::new(this, func.module);
 mov     rax, qword, ptr, [rsp, +, 128]
 mov     qword, ptr, [r12, +, 112], rax
 movups  xmm0, xmmword, ptr, [rsp, +, 112]
 movups  xmmword, ptr, [r12, +, 96], xmm0
 movups  xmm0, xmmword, ptr, [rsp, +, 96]
 movups  xmmword, ptr, [r12, +, 80], xmm0
 movups  xmm0, xmmword, ptr, [rsp, +, 80]
 movups  xmmword, ptr, [r12, +, 64], xmm0
 movdqu  xmm0, xmmword, ptr, [rsp, +, 16]
 movdqu  xmm1, xmmword, ptr, [rsp, +, 32]
 movups  xmm2, xmmword, ptr, [rsp, +, 48]
 movups  xmm3, xmmword, ptr, [rsp, +, 64]
 movups  xmmword, ptr, [r12, +, 48], xmm3
 movups  xmmword, ptr, [r12, +, 32], xmm2
 movdqu  xmmword, ptr, [r12, +, 16], xmm1
 movdqu  xmmword, ptr, [r12], xmm0
 frame.func = cell;
 mov     qword, ptr, [r12, +, 16], r15
 frame.arguments = args;
 mov     qword, ptr, [r12, +, 32], rbx
 jmp     .LBB187_5
.LBB187_32:
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     mov     rdx, qword, ptr, [rdx, +, 32]
     mov     ecx, 10
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     cmp     rdx, 8
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     jb      .LBB187_36
     Some(val) => val, (libcore/option.rs:387)
     and     rdx, -8
     je      .LBB187_944
     mov     eax, dword, ptr, [rax, -, 4]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     cmp     eax, dword, ptr, [rdx, +, 16]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     jae     .LBB187_36
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rcx, qword, ptr, [rdx]
     unsafe { *self.slots.as_ref().unwrap().get_unchecked(offset as usize) } (src/runtime/cell.rs:89)
     mov     rcx, qword, ptr, [rcx, +, 8*rax]
.LBB187_36:
 frame.rax = object.direct(*offset);
 mov     qword, ptr, [r12], rcx
 jmp     .LBB187_5
.LBB187_37:
     if self.len == 0 { (liballoc/vec.rs:1225)
     mov     rax, qword, ptr, [r12, +, 56]
     test    rax, rax
     if self.len == 0 { (liballoc/vec.rs:1225)
     je      .LBB187_250
     self.len -= 1; (liballoc/vec.rs:1229)
     lea     rcx, [rax, -, 1]
     mov     qword, ptr, [r12, +, 56], rcx
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rcx, qword, ptr, [r12, +, 40]
     Some(ptr::read(self.as_ptr().add(self.len()))) (liballoc/vec.rs:1230)
     mov     rax, qword, ptr, [rcx, +, 8*rax, -, 8]
 frame.rax = val;
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_39:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let val = *frame.r(r);
 mov     rsi, qword, ptr, [rax, +, rsi]
 mov     rdi, qword, ptr, [rsp, +, 256]
     self.stack.push(val); (src/runtime/frame.rs:127)
     call    alloc::vec::Vec<T>::push
     jmp     .LBB187_4
.LBB187_40:
 frame.rax = Value::from(VTag::Undefined);
 mov     qword, ptr, [r12], 10
 jmp     .LBB187_4
.LBB187_41:
     if self.len == 0 { (liballoc/vec.rs:1225)
     mov     rax, qword, ptr, [r12, +, 56]
     test    rax, rax
     if self.len == 0 { (liballoc/vec.rs:1225)
     je      .LBB187_251
     self.len -= 1; (liballoc/vec.rs:1229)
     lea     rcx, [rax, -, 1]
     mov     qword, ptr, [r12, +, 56], rcx
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rcx, qword, ptr, [r12, +, 40]
     Some(ptr::read(self.as_ptr().add(self.len()))) (liballoc/vec.rs:1230)
     mov     rax, qword, ptr, [rcx, +, 8*rax, -, 8]
     jmp     .LBB187_252
.LBB187_43:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rax, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rax
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts(self.as_ptr(), self.len) } (liballoc/vec.rs:1966)
     mov     rsi, qword, ptr, [rax, +, 56]
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     cmp     rsi, rbx
     jbe     .LBB187_953
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rax, +, 40]
     self.func.func_value_unchecked().constants[ix as usize] (src/runtime/frame.rs:119)
     mov     rsi, qword, ptr, [rax, +, 8*rbx]
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.lookup(key, &mut slot);
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6lookup17h35588436e8a30a8cE@GOTPCREL]
.LBB187_45:
 mov     rax, qword, ptr, [rsp, +, 24]
 test    rax, rax
 je      .LBB187_62
 mov     rax, qword, ptr, [rax]
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_47:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rdx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rdx
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rdi, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rax, qword, ptr, [rdi, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rax, rcx
     jbe     .LBB187_963
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rax, qword, ptr, [rdi, +, 8]
 if let FeedBack::Cache(attrs, offset, misses) = feedback {
 lea     rdi, [rcx, +, 4*rcx]
 cmp     word, ptr, [rax, +, 8*rdi], 1
 jne     .LBB187_4
 movabs  rbx, -562949953421312
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     lea     rbp, [rbx, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rbp
     jne     .LBB187_52
     cmp     rdx, 10
     ja      .LBB187_511
     mov     ebp, 1029
     bt      rbp, rdx
     jae     .LBB187_511
.LBB187_52:
     or      r13, rbx
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.lookup(key, &mut slot);
 mov     rsi, r13
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6lookup17h35588436e8a30a8cE@GOTPCREL]
     if self.value.is_null() { (src/runtime/cell.rs:377)
     mov     rax, qword, ptr, [rsp, +, 24]
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     test    rax, rax
     if self.value.is_null() { (src/runtime/cell.rs:377)
     je      .LBB187_395
     *self.value (src/runtime/cell.rs:384)
     mov     rax, qword, ptr, [rax]
 frame.rax = slot.value();
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_55:
 let src1 = frame.rax;
 mov     rbx, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let src2 = *frame.r(src2);
 mov     rsi, qword, ptr, [rax, +, rsi]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     movabs  rax, -562949953421312
     lea     rbp, [rax, -, 1]
     cmp     rbx, rbp
 if src1.is_int32() && src2.is_int32() {
 jbe     .LBB187_206
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
 if src1.is_int32() && src2.is_int32() {
 jbe     .LBB187_207
 frame.rax = Value::new_int(src1.as_int32() & src2.as_int32());
 and     ebx, esi
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      rbx, rax
 frame.rax = Value::new_int(src1.as_int32() & src2.as_int32());
 mov     qword, ptr, [r12], rbx
 xor     eax, eax
 xor     ecx, ecx
 jmp     .LBB187_800
.LBB187_58:
 lda_by_idx(&mut frame, base_r, idx_r, fdbk);
 mov     rdi, r12
 mov     edx, r13d
 call    jlight::interpreter::run::lda_by_idx
 jmp     .LBB187_4
.LBB187_59:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base);
 mov     rcx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rcx
     unsafe { &mut *self.regs.offset(i as _).raw } (src/runtime/frame.rs:115)
     movzx   ecx, bl
 let val = *frame.r(val);
 mov     rsi, qword, ptr, [rax, +, rcx]
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.insert(Symbol::new_value(val), &mut slot);
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6insert17h849017bfd7b28e34E@GOTPCREL]
     if self.value.is_null() { (src/runtime/cell.rs:370)
     mov     rax, qword, ptr, [rsp, +, 24]
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     test    rax, rax
     if self.value.is_null() { (src/runtime/cell.rs:370)
     je      .LBB187_62
 slot.store(frame.rax);
 mov     rcx, qword, ptr, [r12]
     *self.value = val; (src/runtime/cell.rs:373)
     mov     qword, ptr, [rax], rcx
 frame.rax = slot.value();
 mov     qword, ptr, [r12], rcx
 jmp     .LBB187_4
.LBB187_62:
 mov     rax, qword, ptr, [rsp, +, 32]
 test    rax, rax
 mov     ecx, 10
 cmovne  rcx, rax
 mov     qword, ptr, [r12], rcx
 jmp     .LBB187_4
.LBB187_63:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rax, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rax
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      r13, rax
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.insert(key, &mut slot);
 mov     rsi, r13
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6insert17h849017bfd7b28e34E@GOTPCREL]
.LBB187_64:
 mov     rax, qword, ptr, [rsp, +, 24]
 test    rax, rax
 je      .LBB187_4
 mov     rcx, qword, ptr, [r12]
 mov     qword, ptr, [rax], rcx
 jmp     .LBB187_4
.LBB187_66:
 let val = frame.rax;
 mov     rbx, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let shift = *frame.r(rhs);
 mov     rsi, qword, ptr, [rax, +, rsi]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     movabs  rax, -562949953421312
     lea     rbp, [rax, -, 1]
     cmp     rbx, rbp
 if val.is_int32() && shift.is_int32() {
 jbe     .LBB187_209
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
 if val.is_int32() && shift.is_int32() {
 jbe     .LBB187_210
 frame.rax = Value::new_int(val.as_int32() >> (shift.as_int32() & 0x1f));
 mov     ecx, esi
 sar     ebx, cl
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      rbx, rax
 frame.rax = Value::new_int(val.as_int32() >> (shift.as_int32() & 0x1f));
 mov     qword, ptr, [r12], rbx
 xor     eax, eax
 xor     ecx, ecx
 jmp     .LBB187_819
.LBB187_69:
     unsafe { &mut *self.regs.offset(i as _).raw } (src/runtime/frame.rs:115)
     movzx   eax, bl
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rcx, qword, ptr, [r12, +, 8]
 let value = *frame.r(r1);
 mov     rax, qword, ptr, [rcx, +, rax]
 *frame.r(r0) = value;
 mov     qword, ptr, [rcx, +, rsi], rax
 jmp     .LBB187_4
.LBB187_70:
     if self.len == self.buf.capacity() { (liballoc/vec.rs:1200)
     mov     rbp, qword, ptr, [r12, +, 104]
     cmp     rbp, qword, ptr, [r12, +, 96]
     if self.len == self.buf.capacity() { (liballoc/vec.rs:1200)
     jne     .LBB187_212
     let (a, b) = intrinsics::add_with_overflow(self as $ActualT, rhs as $ActualT); (libcore/num/mod.rs:3632)
     mov     rax, rbp
     inc     rax
     Some(v) => Ok(v), (libcore/option.rs:540)
     je      .LBB187_970
     let double_cap = self.cap * 2; (liballoc/raw_vec.rs:511)
     mov     rcx, rbp
     add     rcx, rbp
     Ordering::Less | Ordering::Equal => v2, (libcore/cmp.rs:1016)
     cmp     rcx, rax
     cmova   rax, rcx
     mov     ecx, 4
     let (a, b) = intrinsics::mul_with_overflow(self as $ActualT, rhs as $ActualT); (libcore/num/mod.rs:3688)
     mul     rcx
     Ok(t) => Ok(t), (libcore/result.rs:611)
     jo      .LBB187_970
     mov     r14, rax
     if mem::size_of::<T>() == 0 || self.cap == 0 { (liballoc/raw_vec.rs:200)
     test    rbp, rbp
     if mem::size_of::<T>() == 0 || self.cap == 0 { (liballoc/raw_vec.rs:200)
     je      .LBB187_260
     let memory = if let Some((ptr, old_layout)) = self.current_memory() { (liballoc/raw_vec.rs:524)
     mov     rax, qword, ptr, [r12, +, 88]
     let memory = if let Some((ptr, old_layout)) = self.current_memory() { (liballoc/raw_vec.rs:524)
     test    rax, rax
     je      .LBB187_260
     lea     rsi, [4*rbp]
     if size == new_size { (liballoc/alloc.rs:205)
     cmp     rsi, r14
     if size == new_size { (liballoc/alloc.rs:205)
     jne     .LBB187_76
     Ok(t) => Ok(t), (libcore/result.rs:611)
     test    rax, rax
     jne     .LBB187_555
     jmp     .LBB187_957
.LBB187_78:
 let val = frame.rax;
 mov     r14, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let shift = *frame.r(rhs);
 mov     rbx, qword, ptr, [rax, +, rsi]
 movabs  rax, -562949953421312
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     lea     rcx, [rax, -, 1]
     cmp     r14, rcx
     mov     qword, ptr, [rsp, +, 8], rcx
 if val.is_int32() && shift.is_int32() {
 jbe     .LBB187_213
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rcx
 if val.is_int32() && shift.is_int32() {
 jbe     .LBB187_272
 frame.rax = Value::new_int(val.as_int32() << (shift.as_int32() & 0x1f));
 mov     ecx, ebx
 shl     r14d, cl
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      r14, rax
 frame.rax = Value::new_int(val.as_int32() << (shift.as_int32() & 0x1f));
 mov     qword, ptr, [r12], r14
 xor     eax, eax
 xor     ecx, ecx
 jmp     .LBB187_838
.LBB187_81:
 lda_by_id(&mut frame, base_r, key_r, fdbk);
 mov     rdi, r12
 mov     edx, ebp
 mov     ecx, r13d
 call    jlight::interpreter::run::lda_by_id
 jmp     .LBB187_4
.LBB187_82:
 let lhs = frame.rax;
 mov     rbx, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let rhs = *frame.r(rhs);
 mov     rbp, qword, ptr, [rax, +, rsi]
 movabs  rax, -562949953421312
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     lea     r14, [rax, -, 1]
     cmp     rbx, r14
 if lhs.is_int32() && rhs.is_int32() && rhs.as_int32() != 0 {
 jbe     .LBB187_215
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
 if lhs.is_int32() && rhs.is_int32() && rhs.as_int32() != 0 {
 jbe     .LBB187_257
 if lhs.is_int32() && rhs.is_int32() && rhs.as_int32() != 0 {
 test    ebp, ebp
 if lhs.is_int32() && rhs.is_int32() && rhs.as_int32() != 0 {
 je      .LBB187_257
 frame.rax = Value::new_int(lhs.as_int32() % rhs.as_int32());
 cmp     ebx, -2147483648
 jne     .LBB187_87
 cmp     ebp, -1
 je      .LBB187_991
.LBB187_87:
 mov     eax, ebx
 cdq
 idiv    ebp
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      rdx, rax
 frame.rax = Value::new_int(lhs.as_int32() % rhs.as_int32());
 mov     qword, ptr, [r12], rdx
 xor     edx, edx
 xor     eax, eax
 xor     esi, esi
 jmp     .LBB187_781
.LBB187_88:
 let arguments = frame.arguments;
 mov     rax, qword, ptr, [r12, +, 32]
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rcx, -562949953421312
     add     rcx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rax, rcx
     jne     .LBB187_972
     cmp     rax, 10
     ja      .LBB187_91
     mov     ecx, 1029
     bt      rcx, rax
     jb      .LBB187_972
.LBB187_91:
 if let CellValue::Array(ref array) = arguments.as_cell().value {
 cmp     qword, ptr, [rax], 1
 jne     .LBB187_973
 frame.rax = array
 mov     rcx, qword, ptr, [rax, +, 8]
 mov     eax, 10
     if self < slice.len() { unsafe { Some(self.get_unchecked(slice)) } } else { None } (libcore/slice/mod.rs:2850)
     cmp     qword, ptr, [rcx, +, 16], rbx
     Some(x) => Some(f(x)), (libcore/option.rs:458)
     jbe     .LBB187_94
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rcx]
     Some(x) => Some(f(x)), (libcore/option.rs:458)
     mov     rax, qword, ptr, [rax, +, 8*rbx]
.LBB187_94:
 frame.rax = array
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_95:
 let src1 = frame.rax;
 mov     rbp, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let src2 = *frame.r(src2);
 mov     rbx, qword, ptr, [rax, +, rsi]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     movabs  rax, -562949953421312
     lea     rdx, [rax, -, 1]
     cmp     rbp, rdx
 if src1.is_int32() && src2.is_int32() {
 jbe     .LBB187_217
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
 if src1.is_int32() && src2.is_int32() {
 jbe     .LBB187_218
 frame.rax = Value::new_int(src1.as_int32() ^ src2.as_int32());
 xor     ebx, ebp
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      rbx, rax
 frame.rax = Value::new_int(src1.as_int32() ^ src2.as_int32());
 mov     qword, ptr, [r12], rbx
 xor     eax, eax
 xor     ecx, ecx
 jmp     .LBB187_857
.LBB187_98:
 frame.rax = frame.arguments;
 mov     rax, qword, ptr, [r12, +, 32]
 frame.rax = frame.arguments;
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_99:
 let lhs = frame.rax;
 mov     rdi, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let rhs = *frame.r(rhs);
 mov     rdx, qword, ptr, [rax, +, rsi]
 movabs  rcx, -562949953421312
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     lea     rax, [rcx, -, 1]
     cmp     rdi, rax
     if self.is_int32() { (src/runtime/value.rs:343)
     jbe     .LBB187_220
     return self.as_int32() as _; (src/runtime/value.rs:344)
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, edi
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_289
.LBB187_433:
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rcx, rdx
     movabs  rsi, -1125899906842624
     cmp     rcx, rsi
     if self.is_double() { (src/runtime/value.rs:346)
     jae     .LBB187_435
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm1, rcx
     jmp     .LBB187_439
.LBB187_101:
 let acc = frame.rax;
 mov     rax, qword, ptr, [r12]
 jmp     .LBB187_252
.LBB187_102:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let value = *frame.r(r);
 mov     rax, qword, ptr, [rax, +, rsi]
 frame.rax = value;
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_103:
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      r13, rax
 frame.rax = Value::new_int(x);
 mov     qword, ptr, [r12], r13
 jmp     .LBB187_4
.LBB187_104:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rdx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rdx
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rdi, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rax, qword, ptr, [rdi, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rax, rcx
     jbe     .LBB187_997
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rax, qword, ptr, [rdi, +, 8]
 if let FeedBack::Cache(attrs, offset, misses) = feedback {
 lea     rdi, [rcx, +, 4*rcx]
 cmp     word, ptr, [rax, +, 8*rdi], 1
 jne     .LBB187_4
 movabs  rbx, -562949953421312
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     lea     rbp, [rbx, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rbp
     jne     .LBB187_109
     cmp     rdx, 10
     ja      .LBB187_513
     mov     ebp, 1029
     bt      rbp, rdx
     jae     .LBB187_513
.LBB187_109:
     or      r13, rbx
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.lookup(key, &mut slot);
 mov     rsi, r13
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6lookup17h35588436e8a30a8cE@GOTPCREL]
     if self.value.is_null() { (src/runtime/cell.rs:377)
     mov     rax, qword, ptr, [rsp, +, 24]
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     test    rax, rax
     if self.value.is_null() { (src/runtime/cell.rs:377)
     je      .LBB187_396
     *self.value (src/runtime/cell.rs:384)
     mov     rax, qword, ptr, [rax]
 frame.rax = slot.value();
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_112:
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts(self.as_ptr(), self.len) } (liballoc/vec.rs:1966)
     mov     rsi, qword, ptr, [rax, +, 56]
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     cmp     rsi, rbx
     jbe     .LBB187_974
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rax, +, 40]
     self.func.func_value_unchecked().constants[ix as usize] (src/runtime/frame.rs:119)
     mov     rbx, qword, ptr, [rax, +, 8*rbx]
 local_data()
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process10local_data17h1a97fc2e6905fb89E@GOTPCREL]
 mov     r14, rax
 mov     qword, ptr, [rsp, +, 232], rbx
     mov     rax, qword, ptr, [rax, +, 144]
     mov     rcx, qword, ptr, [r14, +, 152]
     self.state.v0 = self.k0 ^ 0x736f6d6570736575; (libcore/hash/sip.rs:215)
     movq    xmm0, rax
     pshufd  xmm0, xmm0, 68
     pxor    xmm0, xmmword, ptr, [rip, +, .LCPI187_1]
     self.state.v1 = self.k1 ^ 0x646f72616e646f6d; (libcore/hash/sip.rs:216)
     movq    xmm1, rcx
     pshufd  xmm1, xmm1, 68
     pxor    xmm1, xmmword, ptr, [rip, +, .LCPI187_2]
 .insert(Symbol::new_value(key), frame.rax);
 mov     rbp, qword, ptr, [r12]
     DefaultHasher(SipHasher13::new_with_keys(self.k0, self.k1)) (libstd/collections/hash/map.rs:2504)
     mov     qword, ptr, [rsp, +, 16], rax
     mov     qword, ptr, [rsp, +, 24], rcx
     mov     qword, ptr, [rsp, +, 32], 0
     movdqu  xmmword, ptr, [rsp, +, 40], xmm0
     movdqu  xmmword, ptr, [rsp, +, 56], xmm1
     pxor    xmm0, xmm0
     lea     rax, [rsp, +, 24]
     movdqu  xmmword, ptr, [rax, +, 48], xmm0
     lea     rsi, [rsp, +, 16]
     mov     rdi, rbx
     call    <jlight::runtime::symbol::Symbol as core::hash::Hash>::hash
     mov     qword, ptr, [rsp, +, 8], rbp
     mov     qword, ptr, [rsp, +, 368], rbx
     let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail; (libcore/hash/sip.rs:308)
     mov     rsi, qword, ptr, [rsp, +, 32]
     let mut state = self.state; (libcore/hash/sip.rs:306)
     mov     rbp, qword, ptr, [rsp, +, 56]
     let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail; (libcore/hash/sip.rs:308)
     shl     rsi, 56
     or      rsi, qword, ptr, [rsp, +, 72]
     mov     rax, qword, ptr, [rsp, +, 64]
     mov     rcx, qword, ptr, [rsp, +, 40]
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 13
     state.v3 ^= b; (libcore/hash/sip.rs:310)
     xor     rax, rsi
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rcx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rcx, 32
     mov     rdx, qword, ptr, [rsp, +, 48]
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 16
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rdx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 21
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 17
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rcx
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rdx, 32
     state.v0 ^= b; (libcore/hash/sip.rs:312)
     xor     rcx, rsi
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 13
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rcx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rcx, 32
     state.v2 ^= 0xff; (libcore/hash/sip.rs:314)
     xor     rdx, 255
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 16
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rdx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 21
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 17
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rdx, 32
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rcx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 13
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rcx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rcx, 32
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 16
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rdx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 21
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 17
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rdx, 32
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rcx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 13
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 16
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rdx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 21
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rcx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 17
 lea     rcx, [r14, +, 144]
 mov     qword, ptr, [rsp, +, 344], rcx
     state.v0 ^ state.v1 ^ state.v2 ^ state.v3 (libcore/hash/sip.rs:317)
     xor     rbp, rdx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rdx, 32
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     state.v0 ^ state.v1 ^ state.v2 ^ state.v3 (libcore/hash/sip.rs:317)
     xor     rbp, rax
     mov     rax, r14
     add     rax, 160
     mov     qword, ptr, [rsp, +, 336], rax
     mov     rcx, qword, ptr, [r14, +, 160]
     mov     rdx, qword, ptr, [r14, +, 168]
     mov     rax, rbp
     shr     rax, 57
     mov     qword, ptr, [rsp, +, 352], rax
     movd    xmm0, eax
     punpcklbw xmm0, xmm0
     pshuflw xmm0, xmm0, 224
     pshufd  xmm0, xmm0, 0
     mov     qword, ptr, [rsp, +, 360], r14
     mov     rsi, qword, ptr, [r14, +, 176]
     xor     eax, eax
     mov     rdi, rbp
     mov     qword, ptr, [rsp, +, 288], rdx
     movdqa  xmmword, ptr, [rsp, +, 384], xmm0
.LBB187_116:
     mov     r12, rdi
     and     r12, rcx
     lea     rdi, [rax, +, r12]
     add     rdi, 16
     mov     qword, ptr, [rsp, +, 320], rdi
     add     rax, 16
     mov     qword, ptr, [rsp, +, 176], rax
     copy_nonoverlapping(src, dst, count) (libcore/intrinsics.rs:1986)
     movdqu  xmm1, xmmword, ptr, [rdx, +, r12]
     movdqa  xmmword, ptr, [rsp, +, 304], xmm1
     pmovmskb(a.as_i8x16()) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1401)
     pcmpeqb xmm0, xmm1
     pmovmskb r15d, xmm0
.LBB187_117:
     test    r15w, r15w
     je      .LBB187_120
     bsf     ax, r15w
     movzx   ebx, ax
     add     rbx, r12
     mov     r13, rcx
     and     rbx, rcx
     shl     rbx, 4
     mov     r14, rsi
     add     rsi, rbx
     lea     rdi, [rsp, +, 232]
     call    qword, ptr, [rip, +, _ZN72_$LT$jlight..runtime..symbol..Symbol$u20$as$u20$core..cmp..PartialEq$GT$2eq17hc8747ce9d1deae42E@GOTPCREL]
     lea     ecx, [r15, -, 1]
     and     ecx, r15d
     mov     r15d, ecx
     test    al, al
     mov     rcx, r13
     mov     rsi, r14
     jne     .LBB187_203
     jmp     .LBB187_117
.LBB187_120:
     transmute::<i8x16, _>(simd_eq(a.as_i8x16(), b.as_i8x16())) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:820)
     pcmpeqd xmm0, xmm0
     movdqa  xmm1, xmmword, ptr, [rsp, +, 304]
     pmovmskb(a.as_i8x16()) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1401)
     pcmpeqb xmm1, xmm0
     pmovmskb eax, xmm1
     test    ax, ax
     mov     r12, qword, ptr, [rsp, +, 184]
     mov     rdx, qword, ptr, [rsp, +, 288]
     movdqa  xmm0, xmmword, ptr, [rsp, +, 384]
     mov     rax, qword, ptr, [rsp, +, 176]
     mov     rdi, qword, ptr, [rsp, +, 320]
     je      .LBB187_116
     mov     rax, qword, ptr, [rsp, +, 344]
     mov     qword, ptr, [rsp, +, 144], rax
     mov     r14, qword, ptr, [rsp, +, 360]
     mov     rcx, qword, ptr, [r14, +, 160]
     mov     rax, qword, ptr, [r14, +, 168]
     xor     esi, esi
     mov     rdi, rbp
     mov     r15, qword, ptr, [rsp, +, 8]
.LBB187_122:
     mov     rdx, rdi
     and     rdx, rcx
     copy_nonoverlapping(src, dst, count) (libcore/intrinsics.rs:1986)
     movdqu  xmm0, xmmword, ptr, [rax, +, rdx]
     pmovmskb(a.as_i8x16()) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1401)
     pmovmskb ebx, xmm0
     lea     rdi, [rsi, +, rdx]
     add     rdi, 16
     add     rsi, 16
     test    bx, bx
     je      .LBB187_122
     bsf     si, bx
     movzx   esi, si
     add     rdx, rsi
     and     rdx, rcx
     mov     bl, byte, ptr, [rax, +, rdx]
     test    bl, bl
     jns     .LBB187_915
     and     bl, 1
     mov     r13, qword, ptr, [rsp, +, 368]
     je      .LBB187_130
.LBB187_125:
     cmp     qword, ptr, [r14, +, 184], 0
     jne     .LBB187_130
     lea     rdi, [rsp, +, 16]
     lea     rdx, [rsp, +, 144]
     mov     rsi, qword, ptr, [rsp, +, 336]
     call    hashbrown::raw::RawTable<T>::reserve_rehash
     mov     rcx, qword, ptr, [r14, +, 160]
     mov     rax, qword, ptr, [r14, +, 168]
     xor     esi, esi
.LBB187_128:
     mov     rdx, rbp
     and     rdx, rcx
     copy_nonoverlapping(src, dst, count) (libcore/intrinsics.rs:1986)
     movdqu  xmm0, xmmword, ptr, [rax, +, rdx]
     pmovmskb(a.as_i8x16()) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1401)
     pmovmskb edi, xmm0
     lea     rbp, [rsi, +, rdx]
     add     rbp, 16
     add     rsi, 16
     test    di, di
     je      .LBB187_128
     bsf     si, di
     movzx   esi, si
     add     rdx, rsi
     and     rdx, rcx
     cmp     byte, ptr, [rax, +, rdx], 0
     jns     .LBB187_916
.LBB187_130:
     mov     rsi, qword, ptr, [r14, +, 176]
     mov     rdi, rdx
     shl     rdi, 4
     movzx   ebp, bl
     sub     qword, ptr, [r14, +, 184], rbp
     intrinsics::wrapping_sub(self, rhs) (libcore/num/mod.rs:3343)
     lea     rbp, [rdx, -, 16]
     and     rbp, rcx
     mov     rcx, qword, ptr, [rsp, +, 352]
     mov     byte, ptr, [rax, +, rdx], cl
     mov     byte, ptr, [rbp, +, rax, +, 16], cl
     intrinsics::move_val_init(&mut *dst, src) (libcore/ptr/mod.rs:817)
     mov     qword, ptr, [rsi, +, rdi], r13
     mov     qword, ptr, [rsi, +, rdi, +, 8], r15
     add     qword, ptr, [r14, +, 192], 1
     jmp     .LBB187_4
.LBB187_131:
 let lhs = frame.rax;
 mov     r14, qword, ptr, [r12]
 mov     qword, ptr, [rsp, +, 376], r14
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let rhs = *frame.r(rhs);
 mov     rbx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 200], rbx
 movabs  rbp, -562949953421312
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     lea     rax, [rbp, -, 1]
     cmp     r14, rax
     mov     qword, ptr, [rsp, +, 8], rax
 if lhs.is_int32()
 jbe     .LBB187_222
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rax
 if lhs.is_int32()
 jbe     .LBB187_290
 mov     eax, 2147483647
 && !(lhs.as_int32() > std::i32::MAX - rhs.as_int32())
 sub     eax, ebx
 && !(lhs.as_int32() > std::i32::MAX - rhs.as_int32())
 cmp     eax, r14d
 if lhs.is_int32()
 jl      .LBB187_291
 frame.rax = Value::new_int(lhs.as_int32() + rhs.as_int32());
 add     ebx, r14d
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     or      rbx, rbp
 frame.rax = Value::new_int(lhs.as_int32() + rhs.as_int32());
 mov     qword, ptr, [r12], rbx
 xor     edx, edx
 xor     ecx, ecx
 xor     esi, esi
 jmp     .LBB187_744
.LBB187_135:
 let src1 = frame.rax;
 mov     rbp, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let src2 = *frame.r(src2);
 mov     rbx, qword, ptr, [rax, +, rsi]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     movabs  rax, -562949953421312
     lea     rdx, [rax, -, 1]
     cmp     rbp, rdx
 if src1.is_int32() && src2.is_int32() {
 jbe     .LBB187_226
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
 if src1.is_int32() && src2.is_int32() {
 jbe     .LBB187_227
 frame.rax = Value::new_int(src1.as_int32() | src2.as_int32());
 or      ebx, ebp
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      rbx, rax
 frame.rax = Value::new_int(src1.as_int32() | src2.as_int32());
 mov     qword, ptr, [r12], rbx
 xor     eax, eax
 xor     ecx, ecx
 jmp     .LBB187_876
.LBB187_138:
 let lhs = frame.rax;
 mov     rbx, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let rhs = *frame.r(rhs);
 mov     rcx, qword, ptr, [rax, +, rsi]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     movabs  rax, -562949953421312
     lea     r8, [rax, -, 1]
     cmp     rbx, r8
 _ if lhs.is_int32() && rhs.is_int32() => {
 jbe     .LBB187_229
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
 _ if lhs.is_int32() && rhs.is_int32() => {
 jbe     .LBB187_315
     let (a, b) = intrinsics::sub_with_overflow(self as $ActualT, rhs as $ActualT); (libcore/num/mod.rs:1634)
     sub     ebx, ecx
 (x, false) => {
 jo      .LBB187_142
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     mov     eax, ebx
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rcx, -562949953421312
     or      rax, rcx
 frame.rax = Value::new_int(x);
 mov     qword, ptr, [r12], rax
.LBB187_142:
 xor     edi, edi
 xor     eax, eax
 xor     ecx, ecx
 jmp     .LBB187_656
.LBB187_143:
 sta_by_id(&mut frame, base_r, key_r, fdbk)
 mov     rdi, r12
 mov     edx, ebp
 mov     ecx, r13d
 call    jlight::interpreter::run::sta_by_id
 jmp     .LBB187_4
.LBB187_144:
 frame.rax = Value::from(VTag::Null);
 mov     qword, ptr, [r12], 2
 jmp     .LBB187_4
.LBB187_145:
 let val = frame.rax;
 mov     rbx, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let shift = *frame.r(rhs);
 mov     rsi, qword, ptr, [rax, +, rsi]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     movabs  rax, -562949953421312
     lea     rbp, [rax, -, 1]
     cmp     rbx, rbp
 if val.is_int32() && shift.is_int32() {
 jbe     .LBB187_235
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
 if val.is_int32() && shift.is_int32() {
 jbe     .LBB187_236
 frame.rax = Value::new_int(val.as_int32() >> (shift.as_int32() & 0x1f));
 mov     ecx, esi
 sar     ebx, cl
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      rbx, rax
 frame.rax = Value::new_int(val.as_int32() >> (shift.as_int32() & 0x1f));
 mov     qword, ptr, [r12], rbx
 xor     eax, eax
 xor     ecx, ecx
 jmp     .LBB187_898
.LBB187_148:
 let lhs = frame.rax;
 mov     rbx, qword, ptr, [r12]
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let rhs = *frame.r(rhs);
 mov     rcx, qword, ptr, [rax, +, rsi]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     movabs  rax, -562949953421312
     lea     r8, [rax, -, 1]
     cmp     rbx, r8
 _ if lhs.is_int32() && rhs.is_int32() => {
 jbe     .LBB187_238
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
 _ if lhs.is_int32() && rhs.is_int32() => {
 jbe     .LBB187_324
     let (a, b) = intrinsics::mul_with_overflow(self as $ActualT, rhs as $ActualT); (libcore/num/mod.rs:1660)
     imul    ecx, ebx
 (x, false) => {
 jo      .LBB187_152
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     mov     eax, ecx
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rcx, -562949953421312
     or      rax, rcx
 frame.rax = Value::new_int(x);
 mov     qword, ptr, [r12], rax
.LBB187_152:
 xor     edi, edi
 xor     eax, eax
 xor     ecx, ecx
 jmp     .LBB187_672
.LBB187_153:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base);
 mov     rcx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rcx
     unsafe { &mut *self.regs.offset(i as _).raw } (src/runtime/frame.rs:115)
     movzx   ecx, bl
 let val = *frame.r(val);
 mov     rsi, qword, ptr, [rax, +, rcx]
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.lookup(Symbol::new_value(val), &mut slot);
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6lookup17h35588436e8a30a8cE@GOTPCREL]
 jmp     .LBB187_45
.LBB187_154:
 let val = frame.rax;
 mov     rsi, qword, ptr, [r12]
 mov     rdi, qword, ptr, [rsp, +, 256]
     self.stack.push(val); (src/runtime/frame.rs:127)
     call    alloc::vec::Vec<T>::push
     jmp     .LBB187_4
.LBB187_155:
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rcx, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts(self.as_ptr(), self.len) } (liballoc/vec.rs:1966)
     mov     rax, qword, ptr, [rcx, +, 56]
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     cmp     rax, rbx
     jbe     .LBB187_975
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rax, qword, ptr, [rcx, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rax, r13
     movabs  rbx, -562949953421312
     jbe     .LBB187_1011
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rax, qword, ptr, [rcx, +, 8]
 if let FeedBack::Cache(attrs, offset, misses) = feedback {
 lea     rcx, [4*r13]
 add     rcx, r13
 cmp     word, ptr, [rax, +, 8*rcx], 1
 jne     .LBB187_1012
 mov     rdx, qword, ptr, [r12, +, 8]
 mov     rdi, qword, ptr, [rdx, +, rsi]
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     lea     rdx, [rbx, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdi, rdx
     jne     .LBB187_954
 mov     r8, qword, ptr, [r12]
     cmp     rdi, 10
     ja      .LBB187_161
     mov     ebx, 1029
     bt      rbx, rdi
     jb      .LBB187_954
.LBB187_161:
     lea     rbx, [rax, +, 8*rcx]
     add     rbx, 8
 if base.as_cell().attributes.ptr_eq(attrs) {
 mov     rdx, qword, ptr, [rdi, +, 48]
     self.inner.as_ptr() == other.inner.as_ptr() (src/arc.rs:75)
     cmp     rdx, qword, ptr, [rbx]
 if base.as_cell().attributes.ptr_eq(attrs) {
 je      .LBB187_253
 lea     rax, [rax, +, 8*rcx]
 add     rax, 2
 *misses += 1;
 add     word, ptr, [rax], 1
 sta_by_id(&mut frame, base_r, key_r, fdbk);
 mov     rdi, r12
 mov     edx, ebp
 mov     ecx, r13d
 call    jlight::interpreter::run::sta_by_id
 jmp     .LBB187_4
.LBB187_163:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rdx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rdx
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rdi, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts(self.as_ptr(), self.len) } (liballoc/vec.rs:1966)
     mov     rax, qword, ptr, [rdi, +, 56]
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     cmp     rax, rbx
     jbe     .LBB187_955
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rdi, +, 40]
     self.func.func_value_unchecked().constants[ix as usize] (src/runtime/frame.rs:119)
     mov     rax, qword, ptr, [rax, +, 8*rbx]
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rcx, qword, ptr, [rdi, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rcx, r13
     jbe     .LBB187_956
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rcx, qword, ptr, [rdi, +, 8]
 if let FeedBack::Cache(attrs, offset, misses) = feedback {
 lea     rdi, [4*r13]
 add     rdi, r13
 cmp     word, ptr, [rcx, +, 8*rdi], 1
 jne     .LBB187_4
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rbx, -562949953421312
     add     rbx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rbx
     jne     .LBB187_169
     cmp     rdx, 10
     ja      .LBB187_515
     mov     ebx, 1029
     bt      rbx, rdx
     jae     .LBB187_515
.LBB187_169:
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.lookup(Symbol::new_value(key), &mut slot);
 mov     rsi, rax
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6lookup17h35588436e8a30a8cE@GOTPCREL]
     if self.value.is_null() { (src/runtime/cell.rs:377)
     mov     rax, qword, ptr, [rsp, +, 24]
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     test    rax, rax
     if self.value.is_null() { (src/runtime/cell.rs:377)
     je      .LBB187_397
     *self.value (src/runtime/cell.rs:384)
     mov     rax, qword, ptr, [rax]
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_172:
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts(self.as_ptr(), self.len) } (liballoc/vec.rs:1966)
     mov     rsi, qword, ptr, [rax, +, 56]
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     cmp     rsi, rbx
     jbe     .LBB187_1005
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rax, +, 40]
     self.func.func_value_unchecked().constants[ix as usize] (src/runtime/frame.rs:119)
     mov     rbp, qword, ptr, [rax, +, 8*rbx]
 let global = local_data().globals.get(&Symbol::new_value(key));
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process10local_data17h1a97fc2e6905fb89E@GOTPCREL]
 mov     rbx, rax
 let global = local_data().globals.get(&Symbol::new_value(key));
 mov     qword, ptr, [rsp, +, 144], rbp
     mov     rax, qword, ptr, [rax, +, 144]
     mov     rcx, qword, ptr, [rbx, +, 152]
     self.state.v0 = self.k0 ^ 0x736f6d6570736575; (libcore/hash/sip.rs:215)
     movq    xmm0, rax
     pshufd  xmm0, xmm0, 68
     pxor    xmm0, xmmword, ptr, [rip, +, .LCPI187_1]
     self.state.v1 = self.k1 ^ 0x646f72616e646f6d; (libcore/hash/sip.rs:216)
     movq    xmm1, rcx
     pshufd  xmm1, xmm1, 68
     pxor    xmm1, xmmword, ptr, [rip, +, .LCPI187_2]
     DefaultHasher(SipHasher13::new_with_keys(self.k0, self.k1)) (libstd/collections/hash/map.rs:2504)
     mov     qword, ptr, [rsp, +, 16], rax
     mov     qword, ptr, [rsp, +, 24], rcx
     mov     qword, ptr, [rsp, +, 32], 0
     movdqu  xmmword, ptr, [rsp, +, 40], xmm0
     movdqu  xmmword, ptr, [rsp, +, 56], xmm1
     pxor    xmm0, xmm0
     lea     rax, [rsp, +, 24]
     movdqu  xmmword, ptr, [rax, +, 48], xmm0
     lea     rsi, [rsp, +, 16]
     mov     rdi, rbp
     call    <jlight::runtime::symbol::Symbol as core::hash::Hash>::hash
     let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail; (libcore/hash/sip.rs:308)
     mov     rsi, qword, ptr, [rsp, +, 32]
     let mut state = self.state; (libcore/hash/sip.rs:306)
     mov     rbp, qword, ptr, [rsp, +, 56]
     let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail; (libcore/hash/sip.rs:308)
     shl     rsi, 56
     or      rsi, qword, ptr, [rsp, +, 72]
     mov     rax, qword, ptr, [rsp, +, 64]
     mov     rdx, qword, ptr, [rsp, +, 40]
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 13
     state.v3 ^= b; (libcore/hash/sip.rs:310)
     xor     rax, rsi
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rdx, 32
     mov     rcx, qword, ptr, [rsp, +, 48]
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 16
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rcx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 21
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 17
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rdx
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rcx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rcx, 32
     state.v0 ^= b; (libcore/hash/sip.rs:312)
     xor     rdx, rsi
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 13
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rdx, 32
     state.v2 ^= 0xff; (libcore/hash/sip.rs:314)
     xor     rcx, 255
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 16
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rcx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 21
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 17
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rcx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rcx, 32
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rdx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 13
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rdx, 32
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 16
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rcx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 21
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 17
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rcx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rcx, 32
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rdx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rdx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 13
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rax
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 16
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rax, rcx
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rax, 21
     intrinsics::wrapping_add(self, rhs) (libcore/num/mod.rs:3320)
     add     rcx, rbp
     intrinsics::rotate_left(self, n as $SelfT) (libcore/num/mod.rs:2704)
     rol     rbp, 17
     mov     rdx, rcx
     rol     rdx, 32
     state.v0 ^ state.v1 ^ state.v2 ^ state.v3 (libcore/hash/sip.rs:317)
     xor     rbp, rcx
     ($state:expr) => {{ compress!($state.v0, $state.v1, $state.v2, $state.v3) }}; (libcore/hash/sip.rs:85)
     xor     rbp, rdx
     state.v0 ^ state.v1 ^ state.v2 ^ state.v3 (libcore/hash/sip.rs:317)
     xor     rbp, rax
     mov     rcx, qword, ptr, [rbx, +, 160]
     mov     rdx, qword, ptr, [rbx, +, 168]
     mov     rax, rbp
     shr     rax, 57
     movd    xmm0, eax
     punpcklbw xmm0, xmm0
     pshuflw xmm0, xmm0, 224
     pshufd  xmm0, xmm0, 0
     mov     rsi, qword, ptr, [rbx, +, 176]
     xor     eax, eax
     mov     qword, ptr, [rsp, +, 304], rdx
     movdqa  xmmword, ptr, [rsp, +, 288], xmm0
.LBB187_176:
     mov     r13, rbp
     and     r13, rcx
     lea     rbp, [rax, +, r13]
     add     rbp, 16
     add     rax, 16
     mov     qword, ptr, [rsp, +, 176], rax
     copy_nonoverlapping(src, dst, count) (libcore/intrinsics.rs:1986)
     movdqu  xmm1, xmmword, ptr, [rdx, +, r13]
     movdqa  xmmword, ptr, [rsp, +, 320], xmm1
     pmovmskb(a.as_i8x16()) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1401)
     pcmpeqb xmm0, xmm1
     pmovmskb r14d, xmm0
.LBB187_177:
     test    r14w, r14w
     je      .LBB187_180
     bsf     ax, r14w
     movzx   r12d, ax
     add     r12, r13
     mov     qword, ptr, [rsp, +, 8], rcx
     and     r12, rcx
     shl     r12, 4
     mov     r15, rsi
     lea     rbx, [rsi, +, r12]
     lea     rdi, [rsp, +, 144]
     mov     rsi, rbx
     call    qword, ptr, [rip, +, _ZN72_$LT$jlight..runtime..symbol..Symbol$u20$as$u20$core..cmp..PartialEq$GT$2eq17hc8747ce9d1deae42E@GOTPCREL]
     lea     ecx, [r14, -, 1]
     and     ecx, r14d
     mov     r14d, ecx
     test    al, al
     mov     rcx, qword, ptr, [rsp, +, 8]
     mov     rsi, r15
     jne     .LBB187_204
     jmp     .LBB187_177
.LBB187_180:
     transmute::<i8x16, _>(simd_eq(a.as_i8x16(), b.as_i8x16())) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:820)
     pcmpeqd xmm0, xmm0
     movdqa  xmm1, xmmword, ptr, [rsp, +, 320]
     pmovmskb(a.as_i8x16()) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1401)
     pcmpeqb xmm1, xmm0
     pmovmskb eax, xmm1
     test    ax, ax
     mov     rdx, qword, ptr, [rsp, +, 304]
     movdqa  xmm0, xmmword, ptr, [rsp, +, 288]
     mov     rax, qword, ptr, [rsp, +, 176]
     jne     .LBB187_248
     jmp     .LBB187_176
.LBB187_181:
 StaByIdx(base_r, key_r, fdbk) => sta_by_idx(&mut frame, base_r, key_r, fdbk),
 mov     rdi, r12
 mov     edx, r13d
 call    jlight::interpreter::run::sta_by_idx
 jmp     .LBB187_4
.LBB187_182:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rdx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rdx
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rdi, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rax, qword, ptr, [rdi, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rax, rcx
     jbe     .LBB187_1013
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rax, qword, ptr, [rdi, +, 8]
 if let FeedBack::Cache(attrs, offset, misses) = feedback {
 lea     rdi, [rcx, +, 4*rcx]
 cmp     word, ptr, [rax, +, 8*rdi], 1
 jne     .LBB187_4
 movabs  rbx, -562949953421312
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     lea     rbp, [rbx, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rbp
     jne     .LBB187_187
     cmp     rdx, 10
     ja      .LBB187_518
     mov     ebp, 1029
     bt      rbp, rdx
     jae     .LBB187_518
.LBB187_187:
     or      r13, rbx
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.insert(key, &mut slot);
 mov     rsi, r13
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6insert17h849017bfd7b28e34E@GOTPCREL]
 jmp     .LBB187_64
.LBB187_188:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rax, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rax
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts(self.as_ptr(), self.len) } (liballoc/vec.rs:1966)
     mov     rsi, qword, ptr, [rax, +, 56]
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     cmp     rsi, rbx
     jbe     .LBB187_1014
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rax, +, 40]
     self.func.func_value_unchecked().constants[ix as usize] (src/runtime/frame.rs:119)
     mov     rsi, qword, ptr, [rax, +, 8*rbx]
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.insert(key, &mut slot);
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6insert17h849017bfd7b28e34E@GOTPCREL]
 jmp     .LBB187_64
.LBB187_190:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rdx, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rdx
     CellValue::Function(f) => f, (src/runtime/cell.rs:316)
     mov     rdi, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts(self.as_ptr(), self.len) } (liballoc/vec.rs:1966)
     mov     rax, qword, ptr, [rdi, +, 56]
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     cmp     rax, rbx
     jbe     .LBB187_998
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rdi, +, 40]
     self.func.func_value_unchecked().constants[ix as usize] (src/runtime/frame.rs:119)
     mov     rax, qword, ptr, [rax, +, 8*rbx]
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rcx, qword, ptr, [rdi, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rcx, r13
     movabs  rbx, -562949953421312
     jbe     .LBB187_1001
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rcx, qword, ptr, [rdi, +, 8]
 if let FeedBack::Cache(attrs, offset, misses) = feedback {
 lea     rdi, [4*r13]
 add     rdi, r13
 cmp     word, ptr, [rcx, +, 8*rdi], 1
 jne     .LBB187_1006
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     add     rbx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rbx
     jne     .LBB187_196
     cmp     rdx, 10
     ja      .LBB187_462
     mov     ebx, 1029
     bt      rbx, rdx
     jae     .LBB187_462
.LBB187_196:
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.lookup(Symbol::new_value(key), &mut slot);
 mov     rsi, rax
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6lookup17h35588436e8a30a8cE@GOTPCREL]
     if self.value.is_null() { (src/runtime/cell.rs:377)
     mov     rax, qword, ptr, [rsp, +, 24]
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     test    rax, rax
     if self.value.is_null() { (src/runtime/cell.rs:377)
     je      .LBB187_258
     *self.value (src/runtime/cell.rs:384)
     mov     rax, qword, ptr, [rax]
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_199:
     if self.is_null_or_undefined() { (src/runtime/value.rs:326)
     mov     rax, qword, ptr, [r12]
     unsafe { (self.u.as_int64 & !Self::UNDEFINED_TAG as i64) == Self::VALUE_NULL as _ } (src/runtime/value.rs:168)
     mov     rcx, rax
     and     rcx, -9
     cmp     rcx, 2
     if self.is_null_or_undefined() { (src/runtime/value.rs:326)
     jne     .LBB187_244
.LBB187_200:
 frame.bp = if_false as _;
 movzx   eax, r13w
 frame.bp = if_false as _;
 mov     qword, ptr, [r12, +, 80], rax
 frame.ip = 0;
 mov     qword, ptr, [r12, +, 72], 0
 jmp     .LBB187_4
.LBB187_201:
 let this = frame.this;
 mov     rax, qword, ptr, [r12, +, 24]
 frame.rax = this;
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_202:
     raw: (self.raw as isize + x) as *mut T, (src/common/ptr.rs:35)
     mov     rax, qword, ptr, [r12, +, 8]
 let mut base = *frame.r(base_r);
 mov     rax, qword, ptr, [rax, +, rsi]
 mov     qword, ptr, [rsp, +, 144], rax
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      r13, rax
     Self { (src/runtime/cell.rs:362)
     pxor    xmm0, xmm0
     movdqa  xmmword, ptr, [rsp, +, 16], xmm0
     mov     qword, ptr, [rsp, +, 32], 0
     mov     dword, ptr, [rsp, +, 40], -1
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.lookup(key, &mut slot);
 mov     rsi, r13
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6lookup17h35588436e8a30a8cE@GOTPCREL]
 jmp     .LBB187_45
.LBB187_203:
 mov     rax, qword, ptr, [rsp, +, 8]
     copy_nonoverlapping(src, dst, count) (libcore/intrinsics.rs:1986)
     mov     qword, ptr, [rsi, +, rbx, +, 8], rax
     mov     r12, qword, ptr, [rsp, +, 184]
     jmp     .LBB187_4
.LBB187_204:
     Some(x) => Some(f(x)), (libcore/option.rs:458)
     test    rbx, rbx
     je      .LBB187_248
     Some(x) => Some(f(x)), (libcore/option.rs:458)
     mov     rax, qword, ptr, [rsi, +, r12, +, 8]
     jmp     .LBB187_249
.LBB187_206:
     xor     r14d, r14d
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbx, rax
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_264
.LBB187_207:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rbp
     mov     qword, ptr, [rsp, +, 8], rsi
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_262
     movabs  rax, -562949953421312
     add     rax, rbx
     movq    xmm0, rax
     jmp     .LBB187_263
.LBB187_209:
     xor     r14d, r14d
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbx, rax
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_269
.LBB187_210:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rbp
     mov     qword, ptr, [rsp, +, 8], rsi
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_267
     movabs  rax, -562949953421312
     add     rax, rbx
     movq    xmm0, rax
     jmp     .LBB187_268
.LBB187_212:
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rax, qword, ptr, [r12, +, 88]
     jmp     .LBB187_557
.LBB187_213:
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     add     rax, r14
     movabs  rcx, -1125899906842624
     cmp     rax, rcx
     if self.is_double() { (src/runtime/value.rs:346)
     jae     .LBB187_273
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rax
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
 let val = val.to_number().floor() as i32;
 cvttsd2si ebp, xmm0
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, qword, ptr, [rsp, +, 8]
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_276
     jmp     .LBB187_401
.LBB187_215:
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     add     rax, rbx
     movabs  rcx, -1125899906842624
     cmp     rax, rcx
     if self.is_double() { (src/runtime/value.rs:346)
     jae     .LBB187_277
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_280
     jmp     .LBB187_417
.LBB187_217:
     xor     r14d, r14d
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbp, rax
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_283
.LBB187_218:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, rdx
     mov     qword, ptr, [rsp, +, 8], rdx
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_281
     movabs  rax, -562949953421312
     add     rax, rbp
     movq    xmm0, rax
     jmp     .LBB187_282
.LBB187_220:
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     add     rcx, rdi
     movabs  rsi, -1125899906842624
     cmp     rcx, rsi
     if self.is_double() { (src/runtime/value.rs:346)
     jae     .LBB187_286
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_289
     jmp     .LBB187_433
.LBB187_222:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     r14, rax
 if lhs.is_number() && rhs.is_number() {
 jbe     .LBB187_295
 movabs  rax, 562949953421311
 cmp     rbx, rax
 jbe     .LBB187_295
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     lea     rax, [r14, +, rbp]
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm1, rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, qword, ptr, [rsp, +, 8]
     if self.is_int32() { (src/runtime/value.rs:343)
     jbe     .LBB187_292
.LBB187_225:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
     jmp     .LBB187_293
.LBB187_226:
     xor     r14d, r14d
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbp, rax
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_312
.LBB187_227:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, rdx
     mov     qword, ptr, [rsp, +, 8], rdx
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_310
     movabs  rax, -562949953421312
     add     rax, rbp
     movq    xmm0, rax
     jmp     .LBB187_311
.LBB187_229:
     movabs  rax, 562949953421311
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     cmp     rbx, rax
 if lhs.is_number() && rhs.is_number() {
 jbe     .LBB187_231
 cmp     rcx, rax
 ja      .LBB187_316
.LBB187_231:
 movabs  rdx, 9221683186994511872
 mov     qword, ptr, [r12], rdx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     cmp     rbx, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     ja      .LBB187_387
     mov     al, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_455
     cmp     rbx, 10
     jne     .LBB187_520
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_456
     jmp     .LBB187_522
.LBB187_235:
     xor     r14d, r14d
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbx, rax
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_321
.LBB187_236:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rbp
     mov     qword, ptr, [rsp, +, 8], rsi
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_319
     movabs  rax, -562949953421312
     add     rax, rbx
     movq    xmm0, rax
     jmp     .LBB187_320
.LBB187_238:
     movabs  rax, 562949953421311
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     cmp     rbx, rax
 if lhs.is_number() && rhs.is_number() {
 jbe     .LBB187_240
 cmp     rcx, rax
 ja      .LBB187_325
.LBB187_240:
 movabs  rdx, 9221683186994511872
 mov     qword, ptr, [r12], rdx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     cmp     rbx, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     ja      .LBB187_393
     mov     al, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_458
     cmp     rbx, 10
     jne     .LBB187_536
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_459
     jmp     .LBB187_538
.LBB187_244:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rax, rcx
     if self.is_number() { (src/runtime/value.rs:329)
     jbe     .LBB187_328
     movabs  rdx, -562949953421312
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     lea     rcx, [rdx, -, 1]
     cmp     rax, rcx
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_449
     add     rax, rdx
     movq    xmm0, rax
 if c {
 ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_0]
 jne     .LBB187_200
 jnp     .LBB187_450
 jmp     .LBB187_200
.LBB187_247:
 if let FeedBack::Cache(attrs, offset, misses) = feedback {
 lea     rax, [4*r13]
 add     rax, r13
 *misses += 1;
 add     word, ptr, [rcx, +, 8*rax, +, 2], 1
 lda_by_id(&mut frame, base_r, key_r, fdbk);
 mov     rdi, r12
 mov     edx, ebp
 mov     ecx, r13d
 call    jlight::interpreter::run::lda_by_id
 jmp     .LBB187_4
.LBB187_248:
 mov     eax, 10
.LBB187_249:
 mov     r12, qword, ptr, [rsp, +, 184]
 frame.rax = global.copied().unwrap_or(Value::from(VTag::Undefined));
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_250:
 mov     eax, 10
 frame.rax = val;
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_251:
 mov     eax, 10
.LBB187_252:
 mov     rcx, qword, ptr, [r12, +, 8]
 mov     qword, ptr, [rcx, +, rsi], rax
 jmp     .LBB187_4
.LBB187_253:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     mov     rdx, rdi
     or      rdx, 8
     cmp     rdx, 10
     je      .LBB187_992
     if self.slots.is_null() { (src/runtime/cell.rs:326)
     mov     rsi, qword, ptr, [rdi, +, 32]
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     cmp     rsi, 8
     if self.slots.is_null() { (src/runtime/cell.rs:326)
     jb      .LBB187_4
     Some(val) => val, (libcore/option.rs:387)
     and     rsi, -8
     je      .LBB187_993
 lea     rax, [rax, +, 8*rcx]
 add     rax, 4
 mov     eax, dword, ptr, [rax]
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rcx, qword, ptr, [rsi]
     *self (src/runtime/cell.rs:330)
     mov     qword, ptr, [rcx, +, 8*rax], r8
     jmp     .LBB187_4
.LBB187_257:
     return self.as_int32() as _; (src/runtime/value.rs:344)
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_280
.LBB187_417:
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rax, rbp
     movabs  rcx, -1125899906842624
     cmp     rax, rcx
     if self.is_double() { (src/runtime/value.rs:346)
     jae     .LBB187_419
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm1, rax
     jmp     .LBB187_423
.LBB187_258:
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     mov     rcx, qword, ptr, [rsp, +, 32]
     unsafe { self.u.as_int64 == Self::VALUE_EMPTY as _ } (src/runtime/value.rs:139)
     test    rcx, rcx
     mov     eax, 10
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     cmovne  rax, rcx
.LBB187_259:
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_260:
     if size == 0 { (liballoc/alloc.rs:170)
     test    r14, r14
     if size == 0 { (liballoc/alloc.rs:170)
     je      .LBB187_460
     __rust_alloc(layout.size(), layout.align()) (liballoc/alloc.rs:80)
     mov     esi, 4
     mov     rdi, r14
     call    qword, ptr, [rip, +, __rust_alloc@GOTPCREL]
     Ok(t) => Ok(t), (libcore/result.rs:611)
     test    rax, rax
     jne     .LBB187_556
     jmp     .LBB187_957
.LBB187_262:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
.LBB187_263:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si r14d, xmm0
     mov     rsi, qword, ptr, [rsp, +, 8]
.LBB187_264:
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rsi, rcx
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_335
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     mov     qword, ptr, [rsp, +, 8], rsi
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_333
     movabs  rax, -562949953421312
     add     rax, rsi
     movq    xmm0, rax
     jmp     .LBB187_334
.LBB187_267:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
.LBB187_268:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si r14d, xmm0
     mov     rsi, qword, ptr, [rsp, +, 8]
.LBB187_269:
     xor     ecx, ecx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rsi, rax
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_345
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     mov     qword, ptr, [rsp, +, 8], rsi
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_343
     movabs  rax, -562949953421312
     add     rax, rsi
     movq    xmm0, rax
     jmp     .LBB187_344
.LBB187_272:
     mov     ebp, r14d
     jmp     .LBB187_401
.LBB187_273:
     pxor    xmm0, xmm0
     if self.is_true() { (src/runtime/value.rs:354)
     cmp     r14, 6
     je      .LBB187_274
     movq    xmm0, qword, ptr, [rip, +, .LCPI187_4]
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     r14, 7
     if self.is_true() { (src/runtime/value.rs:354)
     jne     .LBB187_275
.LBB187_400:
     movq    xmm0, qword, ptr, [rip, +, .LCPI187_0]
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
 let val = val.to_number().floor() as i32;
 cvttsd2si ebp, xmm0
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, qword, ptr, [rsp, +, 8]
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_276
.LBB187_401:
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rax, rbx
     movabs  rcx, -1125899906842624
     cmp     rax, rcx
     if self.is_double() { (src/runtime/value.rs:346)
     jae     .LBB187_403
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rax
     jmp     .LBB187_407
.LBB187_274:
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     r14, 7
     if self.is_true() { (src/runtime/value.rs:354)
     je      .LBB187_400
.LBB187_275:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
 let val = val.to_number().floor() as i32;
 cvttsd2si ebp, xmm0
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, qword, ptr, [rsp, +, 8]
     if self.is_int32() { (src/runtime/value.rs:343)
     jbe     .LBB187_401
.LBB187_276:
     return self.as_int32() as _; (src/runtime/value.rs:344)
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
     jmp     .LBB187_407
.LBB187_403:
     pxor    xmm0, xmm0
     if self.is_true() { (src/runtime/value.rs:354)
     cmp     rbx, 6
     je      .LBB187_405
     movq    xmm0, qword, ptr, [rip, +, .LCPI187_4]
.LBB187_405:
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     rbx, 7
     if self.is_true() { (src/runtime/value.rs:354)
     jne     .LBB187_407
     movq    xmm0, qword, ptr, [rip, +, .LCPI187_0]
.LBB187_407:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
 let shift = shift.to_number().floor() as i32;
 cvttsd2si ecx, xmm0
 frame.rax = Value::new_int(val << (shift & 0x1f));
 shl     ebp, cl
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      rbp, rax
 frame.rax = Value::new_int(val << (shift & 0x1f));
 mov     qword, ptr, [r12], rbp
 mov     rdx, qword, ptr, [rsp, +, 8]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     r14, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_409
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_624
.LBB187_826:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rbx, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_829
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rbx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_832
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     rbx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     jmp     .LBB187_838
.LBB187_409:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     r14, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_412
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     r14, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, r14
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_477
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     r14, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, r14
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_624
     jmp     .LBB187_826
.LBB187_277:
     pxor    xmm0, xmm0
     if self.is_true() { (src/runtime/value.rs:354)
     cmp     rbx, 6
     je      .LBB187_278
     movq    xmm0, qword, ptr, [rip, +, .LCPI187_4]
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     rbx, 7
     if self.is_true() { (src/runtime/value.rs:354)
     jne     .LBB187_279
.LBB187_416:
     movq    xmm0, qword, ptr, [rip, +, .LCPI187_0]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_280
     jmp     .LBB187_417
.LBB187_278:
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     rbx, 7
     if self.is_true() { (src/runtime/value.rs:354)
     je      .LBB187_416
.LBB187_279:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:343)
     jbe     .LBB187_417
.LBB187_280:
     return self.as_int32() as _; (src/runtime/value.rs:344)
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, ebp
.LBB187_423:
 frame.rax = Value::from(lhs.to_number() % rhs.to_number());
 call    qword, ptr, [rip, +, fmod@GOTPCREL]
     let as_i32 = x as i32; (src/runtime/value.rs:522)
     cvttsd2si eax, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, eax
     mov     rcx, rax
     movabs  rdx, -562949953421312
     or      rcx, rdx
     ucomisd xmm0, xmm1
     movq    rdx, xmm0
     movabs  rsi, 562949953421311
     lea     rdx, [rsi, +, rdx, +, 1]
     cmovne  rcx, rdx
     cmovp   rcx, rdx
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     test    eax, eax
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmovne  rcx, rdx
     pxor    xmm1, xmm1
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     ucomisd xmm1, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmova   rcx, rdx
 frame.rax = Value::from(lhs.to_number() % rhs.to_number());
 mov     qword, ptr, [r12], rcx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_425
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_612
.LBB187_751:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rdx, 562949953421311
     cmp     rbp, rdx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_754
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rdx, -562949953421312
     add     rbp, rdx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbp
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_757
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rdx, 9223372036854775807
     and     rbp, rdx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbp
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   dl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     dl, dl
     or      dl, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_765
     jmp     .LBB187_769
.LBB187_425:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     cmp     rbx, rsi
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_428
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_469
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_751
.LBB187_612:
     xor     edx, edx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_765
.LBB187_769:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rsi, 562949953421311
     cmp     rcx, rsi
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_772
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rsi, -562949953421312
     add     rcx, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_775
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rsi, 9223372036854775807
     and     rcx, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   sil
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     sil, sil
     or      sil, 1
     jmp     .LBB187_781
.LBB187_281:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebp
.LBB187_282:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si r14d, xmm0
     mov     rdx, qword, ptr, [rsp, +, 8]
.LBB187_283:
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rbx, rcx
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_355
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     mov     qword, ptr, [rsp, +, 8], rdx
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_353
     movabs  rax, -562949953421312
     add     rax, rbx
     movq    xmm0, rax
     jmp     .LBB187_354
.LBB187_435:
     pxor    xmm1, xmm1
     if self.is_true() { (src/runtime/value.rs:354)
     cmp     rdx, 6
     je      .LBB187_437
     movq    xmm1, qword, ptr, [rip, +, .LCPI187_4]
.LBB187_437:
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     rdx, 7
     if self.is_true() { (src/runtime/value.rs:354)
     jne     .LBB187_439
     movq    xmm1, qword, ptr, [rip, +, .LCPI187_0]
     jmp     .LBB187_439
.LBB187_286:
     pxor    xmm0, xmm0
     cmp     rdi, 6
     je      .LBB187_287
     movq    xmm0, qword, ptr, [rip, +, .LCPI187_4]
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     rdi, 7
     if self.is_true() { (src/runtime/value.rs:354)
     jne     .LBB187_288
.LBB187_432:
     movq    xmm0, qword, ptr, [rip, +, .LCPI187_0]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_289
     jmp     .LBB187_433
.LBB187_287:
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     rdi, 7
     if self.is_true() { (src/runtime/value.rs:354)
     je      .LBB187_432
.LBB187_288:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:343)
     jbe     .LBB187_433
.LBB187_289:
     return self.as_int32() as _; (src/runtime/value.rs:344)
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, edx
.LBB187_439:
 frame.rax = Value::from(lhs.to_number() / rhs.to_number());
 divsd   xmm0, xmm1
     let as_i32 = x as i32; (src/runtime/value.rs:522)
     cvttsd2si esi, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, esi
     mov     rcx, rsi
     movabs  rbp, -562949953421312
     or      rcx, rbp
     ucomisd xmm0, xmm1
     movq    rbp, xmm0
     movabs  rbx, 562949953421311
     lea     rbp, [rbx, +, rbp, +, 1]
     cmovne  rcx, rbp
     cmovp   rcx, rbp
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     test    esi, esi
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmovne  rcx, rbp
     pxor    xmm1, xmm1
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     ucomisd xmm1, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmova   rcx, rbp
 frame.rax = Value::from(lhs.to_number() / rhs.to_number());
 mov     qword, ptr, [r12], rcx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdi, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_441
     xor     esi, esi
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_603
.LBB187_676:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rdi, 562949953421311
     cmp     rdx, rdi
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_679
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rdi, -562949953421312
     add     rdx, rdi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rdx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_682
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rdi, 9223372036854775807
     and     rdx, rdi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rdx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   dil
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     dil, dil
     or      dil, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_690
     jmp     .LBB187_694
.LBB187_441:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     cmp     rdi, rbx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_444
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rsi, -562949953421312
     add     rdi, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rdi
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_461
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rsi, 9223372036854775807
     and     rdi, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rdi
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   sil
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     sil, sil
     or      sil, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_676
.LBB187_603:
     xor     edi, edi
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_690
.LBB187_694:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rcx, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_697
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rcx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_700
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rcx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     jmp     .LBB187_707
.LBB187_444:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rdi, 2
     je      .LBB187_464
     cmp     rdi, 10
     jne     .LBB187_465
     mov     sil, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_603
     jmp     .LBB187_676
.LBB187_679:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rdx, 2
     je      .LBB187_683
     cmp     rdx, 10
     jne     .LBB187_684
     mov     dil, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_690
     jmp     .LBB187_694
.LBB187_697:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rcx, 2
     je      .LBB187_701
     cmp     rcx, 10
     jne     .LBB187_702
     mov     al, 10
     jmp     .LBB187_707
.LBB187_290:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbx, rax
 if lhs.is_number() && rhs.is_number() {
 jbe     .LBB187_295
.LBB187_291:
     return self.as_int32() as _; (src/runtime/value.rs:344)
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, r14d
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, qword, ptr, [rsp, +, 8]
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_225
.LBB187_292:
     lea     rax, [rbx, +, rbp]
     movq    xmm0, rax
.LBB187_293:
 frame.rax = Value::from(lhs.to_number() + rhs.to_number());
 addsd   xmm0, xmm1
     let as_i32 = x as i32; (src/runtime/value.rs:522)
     cvttsd2si ecx, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, ecx
     mov     rax, rcx
     or      rax, rbp
     ucomisd xmm0, xmm1
     movq    rdx, xmm0
     movabs  rsi, 562949953421311
     lea     rdx, [rsi, +, rdx, +, 1]
     cmovne  rax, rdx
     cmovp   rax, rdx
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     test    ecx, ecx
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmovne  rax, rdx
     pxor    xmm1, xmm1
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     ucomisd xmm1, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmova   rax, rdx
 mov     qword, ptr, [r12], rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     r14, qword, ptr, [rsp, +, 8]
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_307
.LBB187_294:
     xor     ecx, ecx
     mov     rdi, qword, ptr, [rsp, +, 8]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_608
.LBB187_714:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rdx, 562949953421311
     cmp     rbx, rdx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_717
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     add     rbx, rbp
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_720
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rdx, 9223372036854775807
     and     rbx, rdx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   dl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     dl, dl
     or      dl, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rax, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_728
     jmp     .LBB187_732
.LBB187_295:
 frame.rax = local_data().allocate_string(
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process10local_data17h1a97fc2e6905fb89E@GOTPCREL]
 mov     qword, ptr, [rsp, +, 176], rax
 lea     rbp, [rsp, +, 208]
 lea     rsi, [rsp, +, 376]
 format!("{}{}", lhs.to_string(), rhs.to_string()),
 mov     rdi, rbp
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value9to_string17hd631185dab7314e7E@GOTPCREL]
 lea     r15, [rsp, +, 232]
 lea     rsi, [rsp, +, 200]
 format!("{}{}", lhs.to_string(), rhs.to_string()),
 mov     rdi, r15
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value9to_string17hd631185dab7314e7E@GOTPCREL]
 format!("{}{}", lhs.to_string(), rhs.to_string()),
 mov     qword, ptr, [rsp, +, 144], rbp
 lea     rax, [rip, +, _ZN60_$LT$alloc..string..String$u20$as$u20$core..fmt..Display$GT$3fmt17h517b077bfbda8f02E]
 mov     qword, ptr, [rsp, +, 152], rax
 mov     qword, ptr, [rsp, +, 160], r15
 mov     qword, ptr, [rsp, +, 168], rax
     Arguments { pieces, fmt: None, args } (libcore/fmt/mod.rs:328)
     lea     rax, [rip, +, .L__unnamed_90]
     mov     qword, ptr, [rsp, +, 16], rax
     mov     qword, ptr, [rsp, +, 24], 2
     mov     qword, ptr, [rsp, +, 32], 0
     lea     rax, [rsp, +, 144]
     mov     qword, ptr, [rsp, +, 48], rax
     mov     qword, ptr, [rsp, +, 56], 2
     lea     rdi, [rsp, +, 264]
     lea     rsi, [rsp, +, 16]
 format!("{}{}", lhs.to_string(), rhs.to_string()),
 call    qword, ptr, [rip, +, _ZN5alloc3fmt6format17hf6896c61c4aa13beE@GOTPCREL]
     pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) { (libcore/ptr/mod.rs:180)
     mov     rdi, qword, ptr, [rsp, +, 232]
     if let Some((ptr, layout)) = self.current_memory() { (liballoc/raw_vec.rs:594)
     test    rdi, rdi
     if mem::size_of::<T>() == 0 || self.cap == 0 { (liballoc/raw_vec.rs:200)
     je      .LBB187_302
     mov     rsi, qword, ptr, [rsp, +, 240]
     test    rsi, rsi
     je      .LBB187_302
     __rust_dealloc(ptr, layout.size(), layout.align()) (liballoc/alloc.rs:102)
     mov     edx, 1
     call    qword, ptr, [rip, +, __rust_dealloc@GOTPCREL]
.LBB187_302:
     pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) { (libcore/ptr/mod.rs:180)
     mov     rdi, qword, ptr, [rsp, +, 208]
     if let Some((ptr, layout)) = self.current_memory() { (liballoc/raw_vec.rs:594)
     test    rdi, rdi
     if mem::size_of::<T>() == 0 || self.cap == 0 { (liballoc/raw_vec.rs:200)
     je      .LBB187_305
     mov     rsi, qword, ptr, [rsp, +, 216]
     test    rsi, rsi
     je      .LBB187_305
     __rust_dealloc(ptr, layout.size(), layout.align()) (liballoc/alloc.rs:102)
     mov     edx, 1
     call    qword, ptr, [rip, +, __rust_dealloc@GOTPCREL]
.LBB187_305:
 format!("{}{}", lhs.to_string(), rhs.to_string()),
 movdqu  xmm0, xmmword, ptr, [rsp, +, 264]
 movdqa  xmmword, ptr, [rsp, +, 16], xmm0
 mov     rax, qword, ptr, [rsp, +, 280]
 mov     qword, ptr, [rsp, +, 32], rax
 lea     rsi, [rsp, +, 16]
 mov     rdi, qword, ptr, [rsp, +, 176]
 frame.rax = local_data().allocate_string(
 mov     rdx, r12
 call    jlight::runtime::process::LocalData::allocate_string
 movabs  rbp, -562949953421312
 mov     qword, ptr, [r12], rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     r14, qword, ptr, [rsp, +, 8]
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_294
.LBB187_307:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     r14, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_330
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     add     r14, rbp
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, r14
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_468
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     r14, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, r14
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     mov     rdi, qword, ptr, [rsp, +, 8]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_714
.LBB187_608:
     xor     edx, edx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rax, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_728
.LBB187_732:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rsi, 562949953421311
     cmp     rax, rsi
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_735
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     add     rax, rbp
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rax
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_738
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rsi, 9223372036854775807
     and     rax, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rax
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   sil
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     sil, sil
     or      sil, 1
     jmp     .LBB187_744
.LBB187_310:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebp
.LBB187_311:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si r14d, xmm0
     mov     rdx, qword, ptr, [rsp, +, 8]
.LBB187_312:
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rbx, rcx
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_365
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     mov     qword, ptr, [rsp, +, 8], rdx
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_363
     movabs  rax, -562949953421312
     add     rax, rbx
     movq    xmm0, rax
     jmp     .LBB187_364
.LBB187_315:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rcx, rax
 if lhs.is_number() && rhs.is_number() {
 jbe     .LBB187_447
.LBB187_316:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, r8
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_383
     movabs  rax, -562949953421312
     add     rax, rbx
     movq    xmm0, rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:343)
     jbe     .LBB187_384
.LBB187_318:
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, ecx
     movabs  rsi, -562949953421312
     jmp     .LBB187_385
.LBB187_319:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
.LBB187_320:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si r14d, xmm0
     mov     rsi, qword, ptr, [rsp, +, 8]
.LBB187_321:
     xor     ecx, ecx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rsi, rax
     if self.is_number() { (src/runtime/value.rs:214)
     jbe     .LBB187_375
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     mov     qword, ptr, [rsp, +, 8], rsi
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_373
     movabs  rax, -562949953421312
     add     rax, rsi
     movq    xmm0, rax
     jmp     .LBB187_374
.LBB187_324:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rcx, rax
 if lhs.is_number() && rhs.is_number() {
 jbe     .LBB187_448
.LBB187_325:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, r8
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_389
     movabs  rax, -562949953421312
     add     rax, rbx
     movq    xmm0, rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:343)
     jbe     .LBB187_390
.LBB187_327:
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, ecx
     movabs  rsi, -562949953421312
     jmp     .LBB187_391
.LBB187_328:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rcx, rax
     and     rcx, -2
     cmp     rcx, 6
     if self.is_bool() { (src/runtime/value.rs:332)
     jne     .LBB187_451
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     rax, 7
 if c {
 jne     .LBB187_200
 jmp     .LBB187_450
.LBB187_330:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     r14, 2
     je      .LBB187_470
     cmp     r14, 10
     jne     .LBB187_471
     mov     cl, 10
     mov     rdi, qword, ptr, [rsp, +, 8]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_608
     jmp     .LBB187_714
.LBB187_717:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_721
     cmp     rbx, 10
     jne     .LBB187_722
     mov     dl, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rax, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_728
     jmp     .LBB187_732
.LBB187_735:
     mov     sil, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rax, 2
     je      .LBB187_744
     cmp     rax, 10
     jne     .LBB187_739
     mov     sil, 10
     jmp     .LBB187_744
.LBB187_419:
     pxor    xmm1, xmm1
     if self.is_true() { (src/runtime/value.rs:354)
     cmp     rbp, 6
     je      .LBB187_421
     movq    xmm1, qword, ptr, [rip, +, .LCPI187_4]
.LBB187_421:
     unsafe { self.u.as_int64 == other.u.as_int64 } (src/runtime/value.rs:532)
     cmp     rbp, 7
     if self.is_true() { (src/runtime/value.rs:354)
     jne     .LBB187_423
     movq    xmm1, qword, ptr, [rip, +, .LCPI187_0]
     jmp     .LBB187_423
.LBB187_428:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_476
     cmp     rbx, 10
     jne     .LBB187_479
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_612
     jmp     .LBB187_751
.LBB187_754:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbp, 2
     je      .LBB187_758
     cmp     rbp, 10
     jne     .LBB187_759
     mov     dl, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_765
     jmp     .LBB187_769
.LBB187_772:
     mov     sil, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rcx, 2
     je      .LBB187_781
     cmp     rcx, 10
     jne     .LBB187_776
     mov     sil, 10
     jmp     .LBB187_781
.LBB187_333:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, esi
.LBB187_334:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si eax, xmm0
     mov     rsi, qword, ptr, [rsp, +, 8]
.LBB187_335:
 frame.rax = Value::new_int(src1.to_int32() & src2.to_int32() as i32);
 and     eax, r14d
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rcx, -562949953421312
     or      rax, rcx
 frame.rax = Value::new_int(src1.to_int32() & src2.to_int32() as i32);
 mov     qword, ptr, [r12], rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_337
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_616
.LBB187_788:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rsi, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_791
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rsi, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rsi
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_794
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     rsi, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rsi
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     jmp     .LBB187_800
.LBB187_337:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbx, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_340
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_474
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_616
     jmp     .LBB187_788
.LBB187_340:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_484
     cmp     rbx, 10
     jne     .LBB187_485
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_616
     jmp     .LBB187_788
.LBB187_791:
     mov     cl, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rsi, 2
     je      .LBB187_800
     cmp     rsi, 10
     jne     .LBB187_795
     mov     cl, 10
     jmp     .LBB187_800
.LBB187_343:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, esi
.LBB187_344:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si ecx, xmm0
     mov     rsi, qword, ptr, [rsp, +, 8]
.LBB187_345:
 frame.rax = Value::new_int(val >> (shift & 0x1f));
 sar     r14d, cl
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      r14, rax
 frame.rax = Value::new_int(val >> (shift & 0x1f));
 mov     qword, ptr, [r12], r14
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_347
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_620
.LBB187_807:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rsi, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_810
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rsi, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rsi
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_813
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     rsi, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rsi
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     jmp     .LBB187_819
.LBB187_347:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbx, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_350
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_475
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_620
     jmp     .LBB187_807
.LBB187_350:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_488
     cmp     rbx, 10
     jne     .LBB187_491
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_620
     jmp     .LBB187_807
.LBB187_810:
     mov     cl, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rsi, 2
     je      .LBB187_819
     cmp     rsi, 10
     jne     .LBB187_814
     mov     cl, 10
     jmp     .LBB187_819
.LBB187_412:
     cmp     r14, 2
     je      .LBB187_489
     cmp     r14, 10
     jne     .LBB187_494
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_624
     jmp     .LBB187_826
.LBB187_829:
     mov     cl, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_838
     cmp     rbx, 10
     jne     .LBB187_833
     mov     cl, 10
     jmp     .LBB187_838
.LBB187_353:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
.LBB187_354:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si eax, xmm0
     mov     rdx, qword, ptr, [rsp, +, 8]
.LBB187_355:
 frame.rax = Value::new_int(src1.to_int32() ^ src2.to_int32() as i32);
 xor     eax, r14d
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rcx, -562949953421312
     or      rax, rcx
 frame.rax = Value::new_int(src1.to_int32() ^ src2.to_int32() as i32);
 mov     qword, ptr, [r12], rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_357
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_628
.LBB187_845:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rbx, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_848
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rbx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_851
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     rbx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     jmp     .LBB187_857
.LBB187_357:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbp, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_360
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rbp, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbp
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_478
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rbp, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbp
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_628
     jmp     .LBB187_845
.LBB187_360:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbp, 2
     je      .LBB187_490
     cmp     rbp, 10
     jne     .LBB187_499
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_628
     jmp     .LBB187_845
.LBB187_848:
     mov     cl, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_857
     cmp     rbx, 10
     jne     .LBB187_852
     mov     cl, 10
     jmp     .LBB187_857
.LBB187_363:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
.LBB187_364:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si eax, xmm0
     mov     rdx, qword, ptr, [rsp, +, 8]
.LBB187_365:
 frame.rax = Value::new_int(src1.to_int32() | src2.to_int32() as i32);
 or      eax, r14d
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rcx, -562949953421312
     or      rax, rcx
 frame.rax = Value::new_int(src1.to_int32() | src2.to_int32() as i32);
 mov     qword, ptr, [r12], rax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_367
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_632
.LBB187_864:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rbx, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_867
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rbx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_870
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     rbx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     jmp     .LBB187_876
.LBB187_367:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbp, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_370
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rbp, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbp
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_482
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rbp, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbp
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_632
     jmp     .LBB187_864
.LBB187_370:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbp, 2
     je      .LBB187_497
     cmp     rbp, 10
     jne     .LBB187_502
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_632
     jmp     .LBB187_864
.LBB187_867:
     mov     cl, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_876
     cmp     rbx, 10
     jne     .LBB187_871
     mov     cl, 10
     jmp     .LBB187_876
.LBB187_373:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, esi
.LBB187_374:
     unsafe { intrinsics::floorf64(self) } (libstd/f64.rs:50)
     call    qword, ptr, [rip, +, floor@GOTPCREL]
     self.to_number().floor() as i32 (src/runtime/value.rs:215)
     cvttsd2si ecx, xmm0
     mov     rsi, qword, ptr, [rsp, +, 8]
.LBB187_375:
 frame.rax = Value::new_int(((val as u32) >> (shift as u32 & 0x1f)) as i32);
 shr     r14d, cl
     as_int64: Self::NUMBER_TAG | unsafe { std::mem::transmute::<i32, u32>(x) as i64 }, (src/runtime/value.rs:128)
     movabs  rax, -562949953421312
     or      r14, rax
 frame.rax = Value::new_int(((val as u32) >> (shift as u32 & 0x1f)) as i32);
 mov     qword, ptr, [r12], r14
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_377
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_636
.LBB187_886:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rsi, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_889
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rsi, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rsi
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_892
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     rsi, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rsi
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     jmp     .LBB187_898
.LBB187_377:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rax, 562949953421311
     cmp     rbx, rax
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_380
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_483
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_636
     jmp     .LBB187_886
.LBB187_380:
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rbx, 2
     je      .LBB187_498
     cmp     rbx, 10
     jne     .LBB187_505
     mov     al, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_636
     jmp     .LBB187_886
.LBB187_889:
     mov     cl, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rsi, 2
     je      .LBB187_898
     cmp     rsi, 10
     jne     .LBB187_893
     mov     cl, 10
     jmp     .LBB187_898
.LBB187_383:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_318
.LBB187_384:
     movabs  rsi, -562949953421312
     lea     rax, [rcx, +, rsi]
     movq    xmm1, rax
.LBB187_385:
 frame.rax = Value::from(lhs.to_number() + rhs.to_number());
 addsd   xmm0, xmm1
     let as_i32 = x as i32; (src/runtime/value.rs:522)
     cvttsd2si eax, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, eax
     mov     rdx, rax
     or      rdx, rsi
     ucomisd xmm0, xmm1
     movq    rbp, xmm0
     movabs  rsi, 562949953421311
     lea     rbp, [rsi, +, rbp, +, 1]
     cmovne  rdx, rbp
     cmovp   rdx, rbp
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     test    eax, eax
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmovne  rdx, rbp
     pxor    xmm1, xmm1
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     ucomisd xmm1, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmova   rdx, rbp
 mov     qword, ptr, [r12], rdx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_387
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_456
     jmp     .LBB187_522
.LBB187_387:
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_454
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_456
.LBB187_522:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rsi, 562949953421311
     cmp     rcx, rsi
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_525
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rsi, -562949953421312
     add     rcx, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_530
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rsi, 9223372036854775807
     and     rcx, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   dil
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     dil, dil
     or      dil, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_535
     jmp     .LBB187_644
.LBB187_389:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, ebx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:343)
     ja      .LBB187_327
.LBB187_390:
     movabs  rsi, -562949953421312
     lea     rax, [rcx, +, rsi]
     movq    xmm1, rax
.LBB187_391:
 frame.rax = Value::from(lhs.to_number() + rhs.to_number());
 addsd   xmm0, xmm1
     let as_i32 = x as i32; (src/runtime/value.rs:522)
     cvttsd2si eax, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     xorps   xmm1, xmm1
     cvtsi2sd xmm1, eax
     mov     rdx, rax
     or      rdx, rsi
     ucomisd xmm0, xmm1
     movq    rbp, xmm0
     movabs  rsi, 562949953421311
     lea     rbp, [rsi, +, rbp, +, 1]
     cmovne  rdx, rbp
     cmovp   rdx, rbp
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     test    eax, eax
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmovne  rdx, rbp
     pxor    xmm1, xmm1
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     ucomisd xmm1, xmm0
     if as_i32 as f64 != x || !(as_i32 == 0 && signbit!(x)) { (src/runtime/value.rs:523)
     cmova   rdx, rbp
 mov     qword, ptr, [r12], rdx
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_393
     xor     eax, eax
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_459
     jmp     .LBB187_538
.LBB187_393:
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rax, -562949953421312
     add     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_457
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rax, 9223372036854775807
     and     rbx, rax
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rbx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   al
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     al, al
     or      al, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_459
.LBB187_538:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rsi, 562949953421311
     cmp     rcx, rsi
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_541
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rsi, -562949953421312
     add     rcx, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_546
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rsi, 9223372036854775807
     and     rcx, rsi
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rcx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   dil
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     dil, dil
     or      dil, 1
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_551
     jmp     .LBB187_660
.LBB187_395:
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     mov     rcx, qword, ptr, [rsp, +, 32]
     unsafe { self.u.as_int64 == Self::VALUE_EMPTY as _ } (src/runtime/value.rs:139)
     test    rcx, rcx
     mov     eax, 10
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     cmovne  rax, rcx
 frame.rax = slot.value();
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_396:
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     mov     rcx, qword, ptr, [rsp, +, 32]
     unsafe { self.u.as_int64 == Self::VALUE_EMPTY as _ } (src/runtime/value.rs:139)
     test    rcx, rcx
     mov     eax, 10
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     cmovne  rax, rcx
 frame.rax = slot.value();
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_397:
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     mov     rcx, qword, ptr, [rsp, +, 32]
     unsafe { self.u.as_int64 == Self::VALUE_EMPTY as _ } (src/runtime/value.rs:139)
     test    rcx, rcx
     mov     eax, 10
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     cmovne  rax, rcx
.LBB187_398:
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_447:
 movabs  rdx, 9221683186994511872
 mov     qword, ptr, [r12], rdx
 xor     eax, eax
.LBB187_525:
 mov     dil, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rcx, 2
     je      .LBB187_534
     cmp     rcx, 10
     jne     .LBB187_528
     mov     dil, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_535
     jmp     .LBB187_644
.LBB187_448:
     movabs  rdx, 9221683186994511872
     mov     qword, ptr, [r12], rdx
     xor     eax, eax
.LBB187_541:
     mov     dil, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rcx, 2
     je      .LBB187_550
     cmp     rcx, 10
     jne     .LBB187_544
     mov     dil, 10
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_551
     jmp     .LBB187_660
.LBB187_449:
     xorps   xmm0, xmm0
     cvtsi2sd xmm0, eax
 if c {
 ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_0]
 jne     .LBB187_200
 jp      .LBB187_200
.LBB187_450:
 mov     qword, ptr, [r12, +, 80], rbx
 mov     qword, ptr, [r12, +, 72], 0
 jmp     .LBB187_4
.LBB187_451:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rcx, -562949953421312
     add     rcx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rax, rcx
     jne     .LBB187_200
     cmp     rax, 10
     ja      .LBB187_450
     mov     ecx, 1029
     bt      rcx, rax
     jb      .LBB187_200
     jmp     .LBB187_450
.LBB187_454:
     mov     al, 2
.LBB187_455:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_522
.LBB187_456:
     xor     edi, edi
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_535
.LBB187_644:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rdx, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_647
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rdx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rdx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_650
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     rdx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rdx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     jmp     .LBB187_656
.LBB187_647:
     mov     cl, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rdx, 2
     je      .LBB187_656
     cmp     rdx, 10
     jne     .LBB187_651
     mov     cl, 10
     jmp     .LBB187_656
.LBB187_457:
     mov     al, 2
.LBB187_458:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_538
.LBB187_459:
     xor     edi, edi
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_551
.LBB187_660:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG) != 0 } (src/runtime/value.rs:179)
     movabs  rcx, 562949953421311
     cmp     rdx, rcx
     } else if self.is_number() { (src/runtime/value.rs:367)
     jbe     .LBB187_663
     !self.is_int32() && self.is_number() (src/runtime/value.rs:183)
     movabs  rcx, -562949953421312
     add     rdx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rdx
     self != self (libcore/num/f64.rs:238)
     ucomisd xmm0, xmm0
     if self.to_number().is_nan() { (src/runtime/value.rs:368)
     jp      .LBB187_666
     f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff) (libcore/num/f64.rs:246)
     movabs  rcx, 9223372036854775807
     and     rdx, rcx
     unsafe { mem::transmute(v) } (libcore/num/f64.rs:567)
     movq    xmm0, rdx
     self.abs_private() == Self::INFINITY (libcore/num/f64.rs:267)
     ucomisd xmm0, qword, ptr, [rip, +, .LCPI187_3]
     setae   cl
     } else if self.to_number().is_infinite() { (src/runtime/value.rs:370)
     add     cl, cl
     or      cl, 1
     jmp     .LBB187_672
.LBB187_663:
     mov     cl, 11
     } else if self.is_null() { (src/runtime/value.rs:375)
     cmp     rdx, 2
     je      .LBB187_672
     cmp     rdx, 10
     jne     .LBB187_667
     mov     cl, 10
     jmp     .LBB187_672
.LBB187_76:
     ReallocPlacement::MayMove if layout.size() == 0 => { (liballoc/alloc.rs:211)
     test    rsi, rsi
     je      .LBB187_552
     __rust_realloc(ptr, layout.size(), layout.align(), new_size) (liballoc/alloc.rs:124)
     mov     edx, 4
     mov     rdi, rax
     mov     rcx, r14
     call    qword, ptr, [rip, +, __rust_realloc@GOTPCREL]
     Ok(t) => Ok(t), (libcore/result.rs:611)
     test    rax, rax
     jne     .LBB187_555
     jmp     .LBB187_957
.LBB187_461:
     mov     sil, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_603
     jmp     .LBB187_676
.LBB187_682:
     mov     dil, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_690
     jmp     .LBB187_694
.LBB187_700:
     mov     al, 2
     jmp     .LBB187_707
.LBB187_528:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rsi, rcx
     and     rsi, -2
     cmp     rsi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_531
     mov     dil, 4
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_535
     jmp     .LBB187_644
.LBB187_544:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rsi, rcx
     and     rsi, -2
     cmp     rsi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_547
     mov     dil, 4
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_551
     jmp     .LBB187_660
.LBB187_462:
 lea     rax, [rcx, +, 8*rdi]
 add     rax, 8
 if base.as_cell().attributes.ptr_eq(attrs) {
 mov     rbx, qword, ptr, [rdx, +, 48]
     self.inner.as_ptr() == other.inner.as_ptr() (src/arc.rs:75)
     cmp     rbx, qword, ptr, [rax]
 if base.as_cell().attributes.ptr_eq(attrs) {
 je      .LBB187_558
 lea     rax, [rcx, +, 8*rdi]
 add     rax, 2
 *misses += 1;
 add     word, ptr, [rax], 1
 lda_by_id(&mut frame, base_r, key_r, fdbk);
 mov     rdi, r12
 mov     edx, ebp
 mov     ecx, r13d
 call    jlight::interpreter::run::lda_by_id
 jmp     .LBB187_4
.LBB187_465:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rbp, rdi
     and     rbp, -2
     mov     sil, 4
     cmp     rbp, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_602
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rsi, -562949953421312
     add     rsi, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdi, rsi
     je      .LBB187_600
.LBB187_464:
     mov     sil, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_603
     jmp     .LBB187_676
.LBB187_684:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rbp, rdx
     and     rbp, -2
     mov     dil, 4
     cmp     rbp, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_689
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdi, -562949953421312
     add     rdi, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rdi
     je      .LBB187_687
.LBB187_683:
     mov     dil, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_690
     jmp     .LBB187_694
.LBB187_702:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, rcx
     and     rdx, -2
     mov     al, 4
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_707
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rcx, rax
     je      .LBB187_705
.LBB187_701:
     mov     al, 11
     jmp     .LBB187_707
.LBB187_468:
     mov     cl, 2
     mov     rdi, qword, ptr, [rsp, +, 8]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_608
     jmp     .LBB187_714
.LBB187_720:
     mov     dl, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rax, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_728
     jmp     .LBB187_732
.LBB187_738:
     mov     sil, 2
     jmp     .LBB187_744
.LBB187_469:
     mov     al, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_612
     jmp     .LBB187_751
.LBB187_757:
     mov     dl, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_765
     jmp     .LBB187_769
.LBB187_775:
     mov     sil, 2
     jmp     .LBB187_781
.LBB187_471:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, r14
     and     rdx, -2
     mov     cl, 4
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_607
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     lea     rcx, [rbp, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    r14, rcx
     je      .LBB187_605
.LBB187_470:
     mov     cl, 11
     mov     rdi, qword, ptr, [rsp, +, 8]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_608
     jmp     .LBB187_714
.LBB187_722:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rsi, rbx
     and     rsi, -2
     mov     dl, 4
     cmp     rsi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_727
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     lea     rdx, [rbp, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rdx
     je      .LBB187_725
.LBB187_721:
     mov     dl, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rax, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_728
     jmp     .LBB187_732
.LBB187_739:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdi, rax
     and     rdi, -2
     cmp     rdi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_741
     mov     sil, 4
     jmp     .LBB187_744
.LBB187_474:
     mov     al, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_616
     jmp     .LBB187_788
.LBB187_794:
     mov     cl, 2
     jmp     .LBB187_800
.LBB187_475:
     mov     al, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_620
     jmp     .LBB187_807
.LBB187_813:
     mov     cl, 2
     jmp     .LBB187_819
.LBB187_477:
     mov     al, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_624
     jmp     .LBB187_826
.LBB187_832:
     mov     cl, 2
     jmp     .LBB187_838
.LBB187_478:
     mov     al, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_628
     jmp     .LBB187_845
.LBB187_851:
     mov     cl, 2
     jmp     .LBB187_857
.LBB187_479:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, rbx
     and     rdx, -2
     mov     al, 4
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_611
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rax
     je      .LBB187_609
.LBB187_476:
     mov     al, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_612
     jmp     .LBB187_751
.LBB187_759:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rsi, rbp
     and     rsi, -2
     mov     dl, 4
     cmp     rsi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_764
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdx, -562949953421312
     add     rdx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbp, rdx
     je      .LBB187_762
.LBB187_758:
     mov     dl, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_765
     jmp     .LBB187_769
.LBB187_776:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdi, rcx
     and     rdi, -2
     cmp     rdi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_778
     mov     sil, 4
     jmp     .LBB187_781
.LBB187_482:
     mov     al, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_632
     jmp     .LBB187_864
.LBB187_870:
     mov     cl, 2
     jmp     .LBB187_876
.LBB187_650:
     mov     cl, 2
     jmp     .LBB187_656
.LBB187_483:
     mov     al, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_636
     jmp     .LBB187_886
.LBB187_892:
     mov     cl, 2
     jmp     .LBB187_898
.LBB187_666:
     mov     cl, 2
     jmp     .LBB187_672
.LBB187_485:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rcx, rbx
     and     rcx, -2
     mov     al, 4
     cmp     rcx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_615
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rax
     je      .LBB187_613
.LBB187_484:
     mov     al, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_616
     jmp     .LBB187_788
.LBB187_795:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, rsi
     and     rdx, -2
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_797
     mov     cl, 4
     jmp     .LBB187_800
.LBB187_491:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rcx, rbx
     and     rcx, -2
     mov     al, 4
     cmp     rcx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_619
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rax
     je      .LBB187_617
.LBB187_488:
     mov     al, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_620
     jmp     .LBB187_807
.LBB187_814:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, rsi
     and     rdx, -2
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_816
     mov     cl, 4
     jmp     .LBB187_819
.LBB187_494:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rcx, r14
     and     rcx, -2
     mov     al, 4
     cmp     rcx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_623
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    r14, rax
     je      .LBB187_621
.LBB187_489:
     mov     al, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_624
     jmp     .LBB187_826
.LBB187_833:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, rbx
     and     rdx, -2
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_835
     mov     cl, 4
     jmp     .LBB187_838
.LBB187_499:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rcx, rbp
     and     rcx, -2
     mov     al, 4
     cmp     rcx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_627
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbp, rax
     je      .LBB187_625
.LBB187_490:
     mov     al, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_628
     jmp     .LBB187_845
.LBB187_852:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, rbx
     and     rdx, -2
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_854
     mov     cl, 4
     jmp     .LBB187_857
.LBB187_502:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rcx, rbp
     and     rcx, -2
     mov     al, 4
     cmp     rcx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_631
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbp, rax
     je      .LBB187_629
.LBB187_497:
     mov     al, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_632
     jmp     .LBB187_864
.LBB187_871:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, rbx
     and     rdx, -2
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_873
     mov     cl, 4
     jmp     .LBB187_876
.LBB187_651:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rsi, rdx
     and     rsi, -2
     cmp     rsi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_653
     mov     cl, 4
     jmp     .LBB187_656
.LBB187_505:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rcx, rbx
     and     rcx, -2
     mov     al, 4
     cmp     rcx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     je      .LBB187_635
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rax, -562949953421312
     add     rax, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rax
     je      .LBB187_633
.LBB187_498:
     mov     al, 11
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_636
     jmp     .LBB187_886
.LBB187_893:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdx, rsi
     and     rdx, -2
     cmp     rdx, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_895
     mov     cl, 4
     jmp     .LBB187_898
.LBB187_667:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rsi, rdx
     and     rsi, -2
     cmp     rsi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_669
     mov     cl, 4
     jmp     .LBB187_672
.LBB187_508:
     lea     rdi, [rsp, +, 144]
     lea     rdx, [rsp, +, 16]
 base.lookup(Symbol::new_value(key), &mut slot);
 mov     rsi, rax
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value6lookup17h35588436e8a30a8cE@GOTPCREL]
     if self.value.is_null() { (src/runtime/cell.rs:377)
     mov     rax, qword, ptr, [rsp, +, 24]
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     test    rax, rax
     if self.value.is_null() { (src/runtime/cell.rs:377)
     je      .LBB187_571
     *self.value (src/runtime/cell.rs:384)
     mov     rax, qword, ptr, [rax]
 frame.rax = slot.value();
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_530:
 mov     dil, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_535
     jmp     .LBB187_644
.LBB187_546:
     mov     dil, 2
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_551
     jmp     .LBB187_660
.LBB187_511:
 lea     rbp, [rax, +, 8*rdi]
 add     rbp, 8
 if base.as_cell().attributes.ptr_eq(attrs) {
 mov     rbx, qword, ptr, [rdx, +, 48]
     self.inner.as_ptr() == other.inner.as_ptr() (src/arc.rs:75)
     cmp     rbx, qword, ptr, [rbp]
 if base.as_cell().attributes.ptr_eq(attrs) {
 je      .LBB187_584
 lea     rax, [rax, +, 8*rdi]
 add     rax, 2
 *misses += 1;
 add     word, ptr, [rax], 1
 lda_by_idx(&mut frame, base_r, idx_r, fdbk);
 mov     rdi, r12
 mov     edx, r13d
 call    jlight::interpreter::run::lda_by_idx
 jmp     .LBB187_4
.LBB187_513:
 lea     rbp, [rax, +, 8*rdi]
 add     rbp, 8
 if base.as_cell().attributes.ptr_eq(attrs) {
 mov     rbx, qword, ptr, [rdx, +, 48]
     self.inner.as_ptr() == other.inner.as_ptr() (src/arc.rs:75)
     cmp     rbx, qword, ptr, [rbp]
 if base.as_cell().attributes.ptr_eq(attrs) {
 je      .LBB187_590
 lea     rax, [rax, +, 8*rdi]
 add     rax, 2
 *misses += 1;
 add     word, ptr, [rax], 1
 lda_by_idx(&mut frame, base_r, idx_r, fdbk);
 mov     rdi, r12
 mov     edx, r13d
 call    jlight::interpreter::run::lda_by_idx
 jmp     .LBB187_4
.LBB187_531:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rsi, -562949953421312
     add     rsi, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rcx, rsi
     jne     .LBB187_534
     cmp     rcx, 10
     ja      .LBB187_641
     mov     esi, 1029
     bt      rsi, rcx
     jae     .LBB187_641
.LBB187_534:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_644
.LBB187_535:
     xor     ecx, ecx
.LBB187_656:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   edx, dil
 shl     edx, 8
 movzx   eax, al
 or      eax, edx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     byte, ptr, [rsp, +, 35], cl
 mov     word, ptr, [rsp, +, 33], ax
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_979
.LBB187_547:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rsi, -562949953421312
     add     rsi, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rcx, rsi
     jne     .LBB187_550
     cmp     rcx, 10
     ja      .LBB187_657
     mov     esi, 1029
     bt      rsi, rcx
     jae     .LBB187_657
.LBB187_550:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_660
.LBB187_551:
     xor     ecx, ecx
.LBB187_672:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   edx, dil
 shl     edx, 8
 movzx   eax, al
 or      eax, edx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     byte, ptr, [rsp, +, 35], cl
 mov     word, ptr, [rsp, +, 33], ax
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_994
.LBB187_515:
 lea     rax, [rcx, +, 8*rdi]
 add     rax, 2
 if let Some(proto) = base.as_cell().prototype {
 cmp     qword, ptr, [rdx, +, 16], 1
 jne     .LBB187_570
 lea     r8, [rcx, +, 8*rdi]
 add     r8, 8
 if let Some(proto) = base.as_cell().prototype {
 mov     rdx, qword, ptr, [rdx, +, 24]
 if proto.attributes.ptr_eq(attrs) {
 mov     rbx, qword, ptr, [rdx, +, 48]
     self.inner.as_ptr() == other.inner.as_ptr() (src/arc.rs:75)
     cmp     rbx, qword, ptr, [r8]
 if proto.attributes.ptr_eq(attrs) {
 je      .LBB187_637
 *misses += 1;
 add     word, ptr, [rax], 1
 lda_by_id(&mut frame, base_r, key_r, fdbk);
 mov     rdi, r12
 mov     edx, ebp
 mov     ecx, r13d
 call    jlight::interpreter::run::lda_by_id
 jmp     .LBB187_4
.LBB187_518:
 lea     rbp, [rax, +, 8*rdi]
 add     rbp, 8
 if base.as_cell().attributes.ptr_eq(attrs) {
 mov     rbx, qword, ptr, [rdx, +, 48]
     self.inner.as_ptr() == other.inner.as_ptr() (src/arc.rs:75)
     cmp     rbx, qword, ptr, [rbp]
 if base.as_cell().attributes.ptr_eq(attrs) {
 je      .LBB187_596
 lea     rax, [rax, +, 8*rdi]
 add     rax, 2
 *misses += 1;
 add     word, ptr, [rax], 1
 sta_by_idx(&mut frame, base_r, idx_r, fdbk);
 mov     rdi, r12
 mov     edx, r13d
 call    jlight::interpreter::run::sta_by_idx
 jmp     .LBB187_4
.LBB187_520:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdi, rbx
     and     rdi, -2
     cmp     rdi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_572
     mov     al, 4
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_456
     jmp     .LBB187_522
.LBB187_536:
     unsafe { (self.u.as_int64 & !1) == Self::VALUE_FALSE as _ } (src/runtime/value.rs:164)
     mov     rdi, rbx
     and     rdi, -2
     cmp     rdi, 6
     } else if self.is_bool() { (src/runtime/value.rs:379)
     jne     .LBB187_578
     mov     al, 4
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_459
     jmp     .LBB187_538
.LBB187_552:
     if size == 0 { (liballoc/alloc.rs:170)
     test    r14, r14
     if size == 0 { (liballoc/alloc.rs:170)
     je      .LBB187_460
     __rust_alloc(layout.size(), layout.align()) (liballoc/alloc.rs:80)
     mov     esi, 4
     mov     rdi, r14
     call    qword, ptr, [rip, +, __rust_alloc@GOTPCREL]
     Ok(t) => Ok(t), (libcore/result.rs:611)
     test    rax, rax
     je      .LBB187_957
.LBB187_555:
     let end = self.as_mut_ptr().add(self.len); (liballoc/vec.rs:1204)
     mov     rbp, qword, ptr, [r12, +, 104]
     jmp     .LBB187_556
.LBB187_460:
     mov     eax, 4
.LBB187_556:
     self.ptr = memory.ptr.cast().into(); (liballoc/raw_vec.rs:472)
     mov     qword, ptr, [r12, +, 88], rax
     excess / mem::size_of::<T>() (liballoc/raw_vec.rs:468)
     shr     r14, 2
     self.cap = Self::capacity_from_bytes(memory.size); (liballoc/raw_vec.rs:473)
     mov     qword, ptr, [r12, +, 96], r14
.LBB187_557:
     intrinsics::move_val_init(&mut *dst, src) (libcore/ptr/mod.rs:817)
     mov     dword, ptr, [rax, +, 4*rbp], ebx
     self.len += 1; (liballoc/vec.rs:1206)
     add     qword, ptr, [r12, +, 104], 1
     jmp     .LBB187_4
.LBB187_741:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     lea     rdi, [rbp, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rax, rdi
     jne     .LBB187_744
     cmp     rax, 10
     ja      .LBB187_745
     mov     edi, 1029
     bt      rdi, rax
     jb      .LBB187_744
.LBB187_745:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdi, qword, ptr, [rax]
     mov     sil, 5
     cmp     rdi, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_744
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rax, 8
     cmp     rax, 10
     je      .LBB187_980
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdi, 3
     sete    sil
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     sil, sil
     add     sil, 6
     jmp     .LBB187_744
.LBB187_778:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdi, -562949953421312
     add     rdi, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rcx, rdi
     jne     .LBB187_781
     cmp     rcx, 10
     ja      .LBB187_782
     mov     edi, 1029
     bt      rdi, rcx
     jb      .LBB187_781
.LBB187_782:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdi, qword, ptr, [rcx]
     mov     sil, 5
     cmp     rdi, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_781
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rcx, 8
     cmp     rcx, 10
     je      .LBB187_1000
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdi, 3
     sete    sil
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     sil, sil
     add     sil, 6
     jmp     .LBB187_781
.LBB187_558:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     mov     rax, rdx
     or      rax, 8
     cmp     rax, 10
     je      .LBB187_971
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     mov     rdx, qword, ptr, [rdx, +, 32]
     mov     eax, 10
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     cmp     rdx, 8
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     jb      .LBB187_259
     Some(val) => val, (libcore/option.rs:387)
     and     rdx, -8
     je      .LBB187_964
     lea     rcx, [rcx, +, 8*rdi]
     add     rcx, 4
     mov     ecx, dword, ptr, [rcx]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     cmp     ecx, dword, ptr, [rdx, +, 16]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     jae     .LBB187_259
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rdx]
     unsafe { *self.slots.as_ref().unwrap().get_unchecked(offset as usize) } (src/runtime/cell.rs:89)
     mov     rax, qword, ptr, [rax, +, 8*rcx]
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_797:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdx, -562949953421312
     add     rdx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rsi, rdx
     jne     .LBB187_800
     cmp     rsi, 10
     ja      .LBB187_801
     mov     edx, 1029
     bt      rdx, rsi
     jb      .LBB187_800
.LBB187_801:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [rsi]
     mov     cl, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_800
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rsi, 8
     cmp     rsi, 10
     je      .LBB187_982
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     jmp     .LBB187_800
.LBB187_816:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdx, -562949953421312
     add     rdx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rsi, rdx
     jne     .LBB187_819
     cmp     rsi, 10
     ja      .LBB187_820
     mov     edx, 1029
     bt      rdx, rsi
     jb      .LBB187_819
.LBB187_820:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [rsi]
     mov     cl, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_819
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rsi, 8
     cmp     rsi, 10
     je      .LBB187_1002
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     jmp     .LBB187_819
.LBB187_835:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdx, -562949953421312
     add     rdx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rdx
     jne     .LBB187_838
     cmp     rbx, 10
     ja      .LBB187_839
     mov     edx, 1029
     bt      rdx, rbx
     jb      .LBB187_838
.LBB187_839:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [rbx]
     mov     cl, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_838
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_985
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     jmp     .LBB187_838
.LBB187_854:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdx, -562949953421312
     add     rdx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rdx
     jne     .LBB187_857
     cmp     rbx, 10
     ja      .LBB187_858
     mov     edx, 1029
     bt      rdx, rbx
     jb      .LBB187_857
.LBB187_858:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [rbx]
     mov     cl, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_857
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_986
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     jmp     .LBB187_857
.LBB187_873:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdx, -562949953421312
     add     rdx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rdx
     jne     .LBB187_876
     cmp     rbx, 10
     ja      .LBB187_877
     mov     edx, 1029
     bt      rdx, rbx
     jb      .LBB187_876
.LBB187_877:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [rbx]
     mov     cl, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_876
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_1003
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     jmp     .LBB187_876
.LBB187_653:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rsi, -562949953421312
     add     rsi, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rsi
     jne     .LBB187_656
     cmp     rdx, 10
     ja      .LBB187_880
     mov     esi, 1029
     bt      rsi, rdx
     jb      .LBB187_656
.LBB187_880:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rsi, qword, ptr, [rdx]
     mov     cl, 5
     cmp     rsi, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_656
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rdx, 8
     cmp     rdx, 10
     je      .LBB187_990
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rsi, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     jmp     .LBB187_656
.LBB187_895:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rdx, -562949953421312
     add     rdx, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rsi, rdx
     jne     .LBB187_898
     cmp     rsi, 10
     ja      .LBB187_909
     mov     edx, 1029
     bt      rdx, rsi
     jb      .LBB187_898
.LBB187_909:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [rsi]
     mov     cl, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_898
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rsi, 8
     cmp     rsi, 10
     je      .LBB187_1004
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     jmp     .LBB187_898
.LBB187_669:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rsi, -562949953421312
     add     rsi, 2
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rdx, rsi
     jne     .LBB187_672
     cmp     rdx, 10
     ja      .LBB187_912
     mov     esi, 1029
     bt      rsi, rdx
     jb      .LBB187_672
.LBB187_912:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rsi, qword, ptr, [rdx]
     mov     cl, 5
     cmp     rsi, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_672
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rdx, 8
     cmp     rdx, 10
     je      .LBB187_951
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rsi, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     jmp     .LBB187_672
.LBB187_563:
 FunctionCode::Native(fun) => {
 mov     r14, qword, ptr, [rbp, +, 96]
 let mut f = Frame::native_frame(frame.rax, args, func.module);
 mov     r15, qword, ptr, [r12]
 let mut f = Frame::native_frame(frame.rax, args, func.module);
 mov     rbp, qword, ptr, [rbp, +, 120]
     Vec { buf: RawVec::NEW, len: 0 } (liballoc/vec.rs:324)
     mov     qword, ptr, [rsp, +, 144], 4
     pxor    xmm0, xmm0
     lea     rax, [rsp, +, 152]
     movdqu  xmmword, ptr, [rax], xmm0
     let result = unsafe { PAGE_SIZE_BITS }; (src/common/mem.rs:19)
     mov     rcx, qword, ptr, [rip, +, _ZN6jlight6common3mem14PAGE_SIZE_BITS17hd4aab45525d7d8ceE]
     if result != 0 { (src/common/mem.rs:21)
     test    rcx, rcx
     if result != 0 { (src/common/mem.rs:21)
     jne     .LBB187_566
     init_page_size(); (src/common/mem.rs:25)
     call    jlight::common::mem::init_page_size
     unsafe { PAGE_SIZE_BITS } (src/common/mem.rs:27)
     mov     rcx, qword, ptr, [rip, +, _ZN6jlight6common3mem14PAGE_SIZE_BITS17hd4aab45525d7d8ceE]
.LBB187_566:
     mov     esi, 1
     ((val + (1 << align) - 1) >> align) << align (src/common/mem.rs:151)
     shl     rsi, cl
     ((val + (1 << align) - 1) >> align) << align (src/common/mem.rs:151)
     add     rsi, 2047
     ((val + (1 << align) - 1) >> align) << align (src/common/mem.rs:151)
     shr     rsi, cl
     shl     rsi, cl
     libc::mmap( (src/common/mem.rs:320)
     mov     edi, 0
     mov     edx, 3
     mov     ecx, 34
     mov     r8d, -1
     xor     r9d, r9d
     call    qword, ptr, [rip, +, mmap@GOTPCREL]
     if ptr == libc::MAP_FAILED { (src/common/mem.rs:330)
     cmp     rax, -1
     if ptr == libc::MAP_FAILED { (src/common/mem.rs:330)
     je      .LBB187_965
     Self { (src/runtime/frame.rs:64)
     movabs  rcx, -562949953421312
     mov     qword, ptr, [rsp, +, 16], rcx
     mov     qword, ptr, [rsp, +, 24], rax
     mov     qword, ptr, [rsp, +, 32], 0
     mov     qword, ptr, [rsp, +, 40], r15
     mov     qword, ptr, [rsp, +, 48], rbx
     mov     qword, ptr, [rsp, +, 56], 8
     pxor    xmm0, xmm0
     lea     rax, [rsp, +, 24]
     mov     rcx, rax
     movdqu  xmmword, ptr, [rax, +, 40], xmm0
     mov     qword, ptr, [rsp, +, 80], rbp
     movdqu  xmmword, ptr, [rax, +, 64], xmm0
     mov     rax, qword, ptr, [rsp, +, 160]
     mov     qword, ptr, [rcx, +, 96], rax
     movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
     movdqu  xmmword, ptr, [rcx, +, 80], xmm0
     mov     byte, ptr, [rsp, +, 128], 0
     lea     rdi, [rsp, +, 16]
 let value = fun(&mut f)?;
 call    r14
 let value = fun(&mut f)?;
 cmp     rax, 1
 je      .LBB187_941
 frame.rax = value;
 mov     qword, ptr, [r12], rdx
 lea     rdi, [rsp, +, 16]
 }
 call    core::ptr::drop_in_place
 jmp     .LBB187_4
.LBB187_570:
 *misses += 1;
 add     word, ptr, [rax], 1
 lda_by_id(&mut frame, base_r, key_r, fdbk);
 mov     rdi, r12
 mov     edx, ebp
 mov     ecx, r13d
 call    jlight::interpreter::run::lda_by_id
 jmp     .LBB187_4
.LBB187_571:
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     mov     rcx, qword, ptr, [rsp, +, 32]
     unsafe { self.u.as_int64 == Self::VALUE_EMPTY as _ } (src/runtime/value.rs:139)
     test    rcx, rcx
     mov     eax, 10
     if self.value_c.is_empty() { (src/runtime/cell.rs:378)
     cmovne  rax, rcx
 frame.rax = slot.value();
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_572:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rsi, -562949953421312
     lea     rdi, [rsi, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rdi
     jne     .LBB187_455
     cmp     rbx, 10
     ja      .LBB187_575
     mov     edi, 1029
     bt      rdi, rbx
     jb      .LBB187_455
.LBB187_575:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rbp, qword, ptr, [rbx]
     mov     al, 5
     cmp     rbp, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_455
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_983
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rbp, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_456
     jmp     .LBB187_522
.LBB187_578:
     let result = unsafe { self.u.as_int64 & Self::NOT_CELL_MASK as i64 }; (src/runtime/value.rs:174)
     movabs  rsi, -562949953421312
     lea     rdi, [rsi, +, 2]
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     test    rbx, rdi
     jne     .LBB187_458
     cmp     rbx, 10
     ja      .LBB187_581
     mov     edi, 1029
     bt      rdi, rbx
     jb      .LBB187_458
.LBB187_581:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rbp, qword, ptr, [rbx]
     mov     al, 5
     cmp     rbp, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_458
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_988
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rbp, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_459
     jmp     .LBB187_538
.LBB187_584:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     mov     rcx, rdx
     or      rcx, 8
     cmp     rcx, 10
     je      .LBB187_976
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     mov     rdx, qword, ptr, [rdx, +, 32]
     mov     ecx, 10
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     cmp     rdx, 8
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     jb      .LBB187_589
     Some(val) => val, (libcore/option.rs:387)
     and     rdx, -8
     je      .LBB187_977
     lea     rax, [rax, +, 8*rdi]
     add     rax, 4
     mov     eax, dword, ptr, [rax]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     cmp     eax, dword, ptr, [rdx, +, 16]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     jae     .LBB187_589
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rcx, qword, ptr, [rdx]
     unsafe { *self.slots.as_ref().unwrap().get_unchecked(offset as usize) } (src/runtime/cell.rs:89)
     mov     rcx, qword, ptr, [rcx, +, 8*rax]
.LBB187_589:
 frame.rax = base.as_cell().direct(*offset);
 mov     qword, ptr, [r12], rcx
 jmp     .LBB187_4
.LBB187_590:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     mov     rcx, rdx
     or      rcx, 8
     cmp     rcx, 10
     je      .LBB187_966
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     mov     rdx, qword, ptr, [rdx, +, 32]
     mov     ecx, 10
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     cmp     rdx, 8
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     jb      .LBB187_595
     Some(val) => val, (libcore/option.rs:387)
     and     rdx, -8
     je      .LBB187_958
     lea     rax, [rax, +, 8*rdi]
     add     rax, 4
     mov     eax, dword, ptr, [rax]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     cmp     eax, dword, ptr, [rdx, +, 16]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     jae     .LBB187_595
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rcx, qword, ptr, [rdx]
     unsafe { *self.slots.as_ref().unwrap().get_unchecked(offset as usize) } (src/runtime/cell.rs:89)
     mov     rcx, qword, ptr, [rcx, +, 8*rax]
.LBB187_595:
 frame.rax = base.as_cell().direct(*offset);
 mov     qword, ptr, [r12], rcx
 jmp     .LBB187_4
.LBB187_596:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     mov     rcx, rdx
     or      rcx, 8
     cmp     rcx, 10
     je      .LBB187_996
     if self.slots.is_null() { (src/runtime/cell.rs:326)
     mov     rcx, qword, ptr, [rdx, +, 32]
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     cmp     rcx, 8
     if self.slots.is_null() { (src/runtime/cell.rs:326)
     jb      .LBB187_4
     Some(val) => val, (libcore/option.rs:387)
     and     rcx, -8
     je      .LBB187_995
 lea     rax, [rax, +, 8*rdi]
 add     rax, 4
 mov     eax, dword, ptr, [rax]
 mov     rdx, qword, ptr, [r12]
     let ptr = self.buf.ptr(); (liballoc/vec.rs:850)
     mov     rcx, qword, ptr, [rcx]
     *self (src/runtime/cell.rs:330)
     mov     qword, ptr, [rcx, +, 8*rax], rdx
     jmp     .LBB187_4
.LBB187_600:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rdi, 10
     ja      .LBB187_673
     mov     esi, 1029
     bt      rsi, rdi
     mov     sil, 11
     jae     .LBB187_673
.LBB187_602:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_676
     jmp     .LBB187_603
.LBB187_687:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rdx, 10
     ja      .LBB187_691
     mov     edi, 1029
     bt      rdi, rdx
     mov     dil, 11
     jae     .LBB187_691
.LBB187_689:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_694
.LBB187_690:
     xor     eax, eax
.LBB187_707:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   ecx, dil
 shl     ecx, 8
 movzx   edx, sil
 or      edx, ecx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     byte, ptr, [rsp, +, 35], al
 mov     word, ptr, [rsp, +, 33], dx
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_968
.LBB187_705:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rcx, 10
     ja      .LBB187_708
     mov     eax, 1029
     bt      rax, rcx
     mov     al, 11
     jb      .LBB187_707
.LBB187_708:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [rcx]
     mov     al, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_707
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rcx, 8
     cmp     rcx, 10
     je      .LBB187_1007
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     jmp     .LBB187_707
.LBB187_605:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     r14, 10
     ja      .LBB187_711
     mov     ecx, 1029
     bt      rcx, r14
     mov     cl, 11
     jae     .LBB187_711
.LBB187_607:
     mov     rdi, qword, ptr, [rsp, +, 8]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_714
     jmp     .LBB187_608
.LBB187_725:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rbx, 10
     ja      .LBB187_729
     mov     edx, 1029
     bt      rdx, rbx
     mov     dl, 11
     jae     .LBB187_729
.LBB187_727:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rax, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_732
.LBB187_728:
     xor     esi, esi
.LBB187_744:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   eax, dl
 shl     eax, 8
 movzx   ecx, cl
 or      ecx, eax
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     byte, ptr, [rsp, +, 35], sil
 mov     word, ptr, [rsp, +, 33], cx
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     unsafe { &mut *self.raw } (src/common/ptr.rs:84)
     mov     rax, qword, ptr, [r12, +, 16]
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [rax, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_1015
.LBB187_609:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rbx, 10
     ja      .LBB187_748
     mov     eax, 1029
     bt      rax, rbx
     mov     al, 11
     jae     .LBB187_748
.LBB187_611:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_751
     jmp     .LBB187_612
.LBB187_762:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rbp, 10
     ja      .LBB187_766
     mov     edx, 1029
     bt      rdx, rbp
     mov     dl, 11
     jae     .LBB187_766
.LBB187_764:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_769
.LBB187_765:
     xor     esi, esi
.LBB187_781:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   ecx, dl
 shl     ecx, 8
 movzx   eax, al
 or      eax, ecx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     byte, ptr, [rsp, +, 35], sil
 mov     word, ptr, [rsp, +, 33], ax
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_981
.LBB187_613:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rbx, 10
     ja      .LBB187_785
     mov     eax, 1029
     bt      rax, rbx
     mov     al, 11
     jae     .LBB187_785
.LBB187_615:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_788
.LBB187_616:
     xor     ecx, ecx
.LBB187_800:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   ecx, cl
 shl     ecx, 8
 movzx   eax, al
 or      eax, ecx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     word, ptr, [rsp, +, 33], ax
 mov     byte, ptr, [rsp, +, 35], 0
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_950
.LBB187_617:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rbx, 10
     ja      .LBB187_804
     mov     eax, 1029
     bt      rax, rbx
     mov     al, 11
     jae     .LBB187_804
.LBB187_619:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_807
.LBB187_620:
     xor     ecx, ecx
.LBB187_819:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   ecx, cl
 shl     ecx, 8
 movzx   eax, al
 or      eax, ecx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     word, ptr, [rsp, +, 33], ax
 mov     byte, ptr, [rsp, +, 35], 0
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_1017
.LBB187_621:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     r14, 10
     ja      .LBB187_823
     mov     eax, 1029
     bt      rax, r14
     mov     al, 11
     jae     .LBB187_823
.LBB187_623:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_826
.LBB187_624:
     xor     ecx, ecx
.LBB187_838:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   ecx, cl
 shl     ecx, 8
 movzx   eax, al
 or      eax, ecx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     word, ptr, [rsp, +, 33], ax
 mov     byte, ptr, [rsp, +, 35], 0
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_952
.LBB187_625:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rbp, 10
     ja      .LBB187_842
     mov     eax, 1029
     bt      rax, rbp
     mov     al, 11
     jae     .LBB187_842
.LBB187_627:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_845
.LBB187_628:
     xor     ecx, ecx
.LBB187_857:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   ecx, cl
 shl     ecx, 8
 movzx   eax, al
 or      eax, ecx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     word, ptr, [rsp, +, 33], ax
 mov     byte, ptr, [rsp, +, 35], 0
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_962
.LBB187_629:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rbp, 10
     ja      .LBB187_861
     mov     eax, 1029
     bt      rax, rbp
     mov     al, 11
     jae     .LBB187_861
.LBB187_631:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_864
.LBB187_632:
     xor     ecx, ecx
.LBB187_876:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   ecx, cl
 shl     ecx, 8
 movzx   eax, al
 or      eax, ecx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     word, ptr, [rsp, +, 33], ax
 mov     byte, ptr, [rsp, +, 35], 0
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     ja      .LBB187_899
     jmp     .LBB187_1019
.LBB187_633:
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     cmp     rbx, 10
     ja      .LBB187_883
     mov     eax, 1029
     bt      rax, rbx
     mov     al, 11
     jae     .LBB187_883
.LBB187_635:
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     jbe     .LBB187_886
.LBB187_636:
     xor     ecx, ecx
.LBB187_898:
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 movzx   ecx, cl
 shl     ecx, 8
 movzx   eax, al
 or      eax, ecx
 FeedBack::TypeInfo(smallvec::SmallVec::from_buf([
 mov     qword, ptr, [rsp, +, 24], 3
 mov     byte, ptr, [rsp, +, 32], 0
 mov     word, ptr, [rsp, +, 33], ax
 mov     byte, ptr, [rsp, +, 35], 0
 mov     eax, dword, ptr, [rsp, +, 160]
 lea     rcx, [rsp, +, 24]
 mov     dword, ptr, [rcx, +, 28], eax
 movdqu  xmm0, xmmword, ptr, [rsp, +, 144]
 movdqu  xmmword, ptr, [rcx, +, 12], xmm0
 mov     word, ptr, [rsp, +, 16], 3
     CellValue::Function(f) => f, (src/runtime/cell.rs:310)
     mov     rax, qword, ptr, [r15, +, 8]
     unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) } (liballoc/vec.rs:1973)
     mov     rsi, qword, ptr, [rax, +, 24]
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     cmp     rsi, r13
     jbe     .LBB187_1010
.LBB187_899:
 mov     rax, qword, ptr, [rax, +, 8]
 lea     rcx, [4*r13]
 add     rcx, r13
 lea     rbp, [rax, +, 8*rcx]
 movzx   edx, word, ptr, [rax, +, 8*rcx]
 test    rdx, rdx
 je      .LBB187_3
 cmp     rdx, 2
 je      .LBB187_3
 cmp     rdx, 1
 jne     .LBB187_907
 mov     rdx, qword, ptr, [rax, +, 8*rcx, +, 8]
 lock    sub, qword, ptr, [rdx, +, 24], 1
 jne     .LBB187_3
 lea     rax, [rax, +, 8*rcx]
 add     rax, 8
 mov     rbx, qword, ptr, [rax]
 mov     rsi, qword, ptr, [rbx, +, 8]
 test    rsi, rsi
 je      .LBB187_1
 mov     rdi, qword, ptr, [rbx]
 test    rdi, rdi
 je      .LBB187_1
 shl     rsi, 4
 je      .LBB187_1
 mov     edx, 8
 call    qword, ptr, [rip, +, __rust_dealloc@GOTPCREL]
 jmp     .LBB187_1
.LBB187_907:
 mov     rsi, qword, ptr, [rax, +, 8*rcx, +, 8]
 cmp     rsi, 3
 jbe     .LBB187_3
 mov     rdi, qword, ptr, [rax, +, 8*rcx, +, 24]
 mov     edx, 1
 jmp     .LBB187_2
.LBB187_637:
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     mov     rdx, qword, ptr, [rdx, +, 32]
     mov     eax, 10
     (self as *mut u8) == null_mut() (libcore/ptr/mut_ptr.rs:30)
     cmp     rdx, 8
     if self.slots.is_null() { (src/runtime/cell.rs:83)
     jb      .LBB187_398
     Some(val) => val, (libcore/option.rs:387)
     and     rdx, -8
     je      .LBB187_978
     lea     rcx, [rcx, +, 8*rdi]
     add     rcx, 4
     mov     ecx, dword, ptr, [rcx]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     cmp     ecx, dword, ptr, [rdx, +, 16]
     if offset >= self.slots.as_ref().unwrap().len() as u32 { (src/runtime/cell.rs:86)
     jae     .LBB187_398
     let ptr = self.buf.ptr(); (liballoc/vec.rs:814)
     mov     rax, qword, ptr, [rdx]
     unsafe { *self.slots.as_ref().unwrap().get_unchecked(offset as usize) } (src/runtime/cell.rs:89)
     mov     rax, qword, ptr, [rax, +, 8*rcx]
 mov     qword, ptr, [r12], rax
 jmp     .LBB187_4
.LBB187_641:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rbp, qword, ptr, [rcx]
     mov     dil, 5
     cmp     rbp, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_534
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rcx, 8
     cmp     rcx, 10
     je      .LBB187_984
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rbp, 3
     sete    dil
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     dil, dil
     add     dil, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_535
     jmp     .LBB187_644
.LBB187_657:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rbp, qword, ptr, [rcx]
     mov     dil, 5
     cmp     rbp, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_550
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rcx, 8
     cmp     rcx, 10
     je      .LBB187_989
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rbp, 3
     sete    dil
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     dil, dil
     add     dil, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, r8
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_551
     jmp     .LBB187_660
.LBB187_673:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rbp, qword, ptr, [rdi]
     mov     sil, 5
     cmp     rbp, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_602
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rdi, 8
     cmp     rdi, 10
     je      .LBB187_999
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rbp, 3
     sete    sil
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     sil, sil
     add     sil, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rdx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_603
     jmp     .LBB187_676
.LBB187_691:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rbp, qword, ptr, [rdx]
     mov     dil, 5
     cmp     rbp, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_689
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rdx, 8
     cmp     rdx, 10
     je      .LBB187_967
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rbp, 3
     sete    dil
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     dil, dil
     add     dil, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, rax
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_690
     jmp     .LBB187_694
.LBB187_711:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [r14]
     mov     cl, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_607
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      r14, 8
     cmp     r14, 10
     je      .LBB187_987
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    cl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     cl, cl
     add     cl, 6
     mov     rdi, qword, ptr, [rsp, +, 8]
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_608
     jmp     .LBB187_714
.LBB187_729:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rsi, qword, ptr, [rbx]
     mov     dl, 5
     cmp     rsi, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_727
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_949
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rsi, 3
     sete    dl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     dl, dl
     add     dl, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rax, rdi
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_728
     jmp     .LBB187_732
.LBB187_748:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rdx, qword, ptr, [rbx]
     mov     al, 5
     cmp     rdx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_611
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_969
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rdx, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbp, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_612
     jmp     .LBB187_751
.LBB187_766:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rsi, qword, ptr, [rbp]
     mov     dl, 5
     cmp     rsi, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_764
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbp, 8
     cmp     rbp, 10
     je      .LBB187_1008
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rsi, 3
     sete    dl
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     dl, dl
     add     dl, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rcx, r14
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_765
     jmp     .LBB187_769
.LBB187_785:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rcx, qword, ptr, [rbx]
     mov     al, 5
     cmp     rcx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_615
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_959
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rcx, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_616
     jmp     .LBB187_788
.LBB187_804:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rcx, qword, ptr, [rbx]
     mov     al, 5
     cmp     rcx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_619
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_1016
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rcx, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_620
     jmp     .LBB187_807
.LBB187_823:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rcx, qword, ptr, [r14]
     mov     al, 5
     cmp     rcx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_623
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      r14, 8
     cmp     r14, 10
     je      .LBB187_960
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rcx, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_624
     jmp     .LBB187_826
.LBB187_842:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rcx, qword, ptr, [rbp]
     mov     al, 5
     cmp     rcx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_627
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbp, 8
     cmp     rbp, 10
     je      .LBB187_961
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rcx, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_628
     jmp     .LBB187_845
.LBB187_861:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rcx, qword, ptr, [rbp]
     mov     al, 5
     cmp     rcx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_631
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbp, 8
     cmp     rbp, 10
     je      .LBB187_1018
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rcx, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rbx, rdx
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_632
     jmp     .LBB187_864
.LBB187_883:
     CellValue::String(_) => true, (src/runtime/cell.rs:227)
     mov     rcx, qword, ptr, [rbx]
     mov     al, 5
     cmp     rcx, 2
     if self.as_cell().is_string() { (src/runtime/value.rs:382)
     je      .LBB187_635
     result == 0 && !self.is_empty() && !self.is_null_or_undefined() (src/runtime/value.rs:175)
     or      rbx, 8
     cmp     rbx, 10
     je      .LBB187_1009
     CellValue::Function(_) => true, (src/runtime/cell.rs:233)
     cmp     rcx, 3
     sete    al
     } else if self.as_cell().is_function() { (src/runtime/value.rs:386)
     add     al, al
     add     al, 6
     unsafe { (self.u.as_int64 & Self::NUMBER_TAG as i64) == Self::NUMBER_TAG as i64 } (src/runtime/value.rs:187)
     cmp     rsi, rbp
     if self.is_int32() { (src/runtime/value.rs:365)
     ja      .LBB187_636
     jmp     .LBB187_886
.LBB187_915:
     *mem_addr (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1204)
     movdqa  xmm0, xmmword, ptr, [rax]
     pmovmskb(a.as_i8x16()) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1401)
     pmovmskb edx, xmm0
     bsf     dx, dx
     movzx   edx, dx
     mov     bl, byte, ptr, [rax, +, rdx]
     and     bl, 1
     mov     r13, qword, ptr, [rsp, +, 368]
     jne     .LBB187_125
     jmp     .LBB187_130
.LBB187_916:
     *mem_addr (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1204)
     movdqa  xmm0, xmmword, ptr, [rax]
     pmovmskb(a.as_i8x16()) (libcore/../stdarch/crates/core_arch/src/x86/sse2.rs:1401)
     pmovmskb edx, xmm0
     bsf     dx, dx
     movzx   edx, dx
     jmp     .LBB187_130
.LBB187_917:
 Return => return Ok(frame.rax),
 mov     rax, qword, ptr, [r12]
 mov     rcx, qword, ptr, [rsp, +, 192]
 Return => return Ok(frame.rax),
 mov     qword, ptr, [rcx, +, 8], rax
 mov     qword, ptr, [rcx], 0
.LBB187_918:
 }
 mov     rdi, r12
 call    core::ptr::drop_in_place
 mov     rax, qword, ptr, [rsp, +, 192]
 jmp     .LBB187_922
.LBB187_919:
 let err = frame.rax;
 mov     rbx, qword, ptr, [r12]
 mov     r14b, 1
 local_data().frames.push(frame);
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process10local_data17h1a97fc2e6905fb89E@GOTPCREL]
 local_data().frames.push(frame);
 mov     rcx, qword, ptr, [r12, +, 112]
 mov     qword, ptr, [rsp, +, 128], rcx
 movups  xmm0, xmmword, ptr, [r12, +, 96]
 movaps  xmmword, ptr, [rsp, +, 112], xmm0
 movups  xmm0, xmmword, ptr, [r12, +, 80]
 movaps  xmmword, ptr, [rsp, +, 96], xmm0
 movups  xmm0, xmmword, ptr, [r12, +, 64]
 movaps  xmmword, ptr, [rsp, +, 80], xmm0
 movdqu  xmm0, xmmword, ptr, [r12]
 movdqu  xmm1, xmmword, ptr, [r12, +, 16]
 movups  xmm2, xmmword, ptr, [r12, +, 32]
 movups  xmm3, xmmword, ptr, [r12, +, 48]
 movaps  xmmword, ptr, [rsp, +, 64], xmm3
 movaps  xmmword, ptr, [rsp, +, 48], xmm2
 movdqa  xmmword, ptr, [rsp, +, 32], xmm1
 movdqa  xmmword, ptr, [rsp, +, 16], xmm0
 xor     r14d, r14d
 lea     rsi, [rsp, +, 16]
 local_data().frames.push(frame);
 mov     rdi, rax
 call    alloc::vec::Vec<T>::push
 mov     rax, qword, ptr, [rsp, +, 192]
 return Err(err);
 mov     qword, ptr, [rax, +, 8], rbx
 mov     qword, ptr, [rax], 1
.LBB187_922:
 }
 add     rsp, 408
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret
.LBB187_923:
 return Err(local_data().allocate_string(
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process10local_data17h1a97fc2e6905fb89E@GOTPCREL]
 mov     rbx, rax
 lea     rbp, [rsp, +, 144]
 lea     rsi, [rsp, +, 200]
 format!("{} is not a function", value.to_string()),
 mov     rdi, rbp
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value9to_string17hd631185dab7314e7E@GOTPCREL]
 format!("{} is not a function", value.to_string()),
 mov     qword, ptr, [rsp, +, 264], rbp
 lea     rax, [rip, +, _ZN60_$LT$alloc..string..String$u20$as$u20$core..fmt..Display$GT$3fmt17h517b077bfbda8f02E]
 mov     qword, ptr, [rsp, +, 272], rax
     Arguments { pieces, fmt: None, args } (libcore/fmt/mod.rs:328)
     lea     rax, [rip, +, .L__unnamed_91]
     mov     qword, ptr, [rsp, +, 16], rax
     mov     qword, ptr, [rsp, +, 24], 2
     mov     qword, ptr, [rsp, +, 32], 0
     lea     rax, [rsp, +, 264]
     mov     qword, ptr, [rsp, +, 48], rax
     mov     qword, ptr, [rsp, +, 56], 1
     lea     rdi, [rsp, +, 232]
     lea     rsi, [rsp, +, 16]
 format!("{} is not a function", value.to_string()),
 call    qword, ptr, [rip, +, _ZN5alloc3fmt6format17hf6896c61c4aa13beE@GOTPCREL]
     pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) { (libcore/ptr/mod.rs:180)
     mov     rdi, qword, ptr, [rsp, +, 144]
     if let Some((ptr, layout)) = self.current_memory() { (liballoc/raw_vec.rs:594)
     test    rdi, rdi
     if mem::size_of::<T>() == 0 || self.cap == 0 { (liballoc/raw_vec.rs:200)
     je      .LBB187_929
     mov     rsi, qword, ptr, [rsp, +, 152]
     test    rsi, rsi
     je      .LBB187_929
     __rust_dealloc(ptr, layout.size(), layout.align()) (liballoc/alloc.rs:102)
     mov     edx, 1
     call    qword, ptr, [rip, +, __rust_dealloc@GOTPCREL]
.LBB187_929:
     lea     rsi, [rsp, +, 16]
 format!("{} is not a function", value.to_string()),
 movdqu  xmm0, xmmword, ptr, [rsp, +, 232]
 movdqa  xmmword, ptr, [rsp, +, 16], xmm0
 mov     rax, qword, ptr, [rsp, +, 248]
 mov     qword, ptr, [rsp, +, 32], rax
 return Err(local_data().allocate_string(
 mov     rdi, rbx
 mov     rdx, r12
 call    jlight::runtime::process::LocalData::allocate_string
 jmp     .LBB187_937
.LBB187_930:
 return Err(local_data().allocate_string(
 call    qword, ptr, [rip, +, _ZN6jlight7runtime7process10local_data17h1a97fc2e6905fb89E@GOTPCREL]
 mov     rbx, rax
 lea     rbp, [rsp, +, 144]
 lea     rsi, [rsp, +, 200]
 format!("{} is not a function", value.to_string()),
 mov     rdi, rbp
 call    qword, ptr, [rip, +, _ZN6jlight7runtime5value5Value9to_string17hd631185dab7314e7E@GOTPCREL]
 format!("{} is not a function", value.to_string()),
 mov     qword, ptr, [rsp, +, 264], rbp
 lea     rax, [rip, +, _ZN60_$LT$alloc..string..String$u20$as$u20$core..fmt..Display$GT$3fmt17h517b077bfbda8f02E]
 mov     qword, ptr, [rsp, +, 272], rax
     Arguments { pieces, fmt: None, args } (libcore/fmt/mod.rs:328)
     lea     rax, [rip, +, .L__unnamed_91]
     mov     qword, ptr, [rsp, +, 16], rax
     mov     qword, ptr, [rsp, +, 24], 2
     mov     qword, ptr, [rsp, +, 32], 0
     lea     rax, [rsp, +, 264]
     mov     qword, ptr, [rsp, +, 48], rax
     mov     qword, ptr, [rsp, +, 56], 1
     lea     rdi, [rsp, +, 232]
     lea     rsi, [rsp, +, 16]
 format!("{} is not a function", value.to_string()),
 call    qword, ptr, [rip, +, _ZN5alloc3fmt6format17hf6896c61c4aa13beE@GOTPCREL]
     pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) { (libcore/ptr/mod.rs:180)
     mov     rdi, qword, ptr, [rsp, +, 144]
     if let Some((ptr, layout)) = self.current_memory() { (liballoc/raw_vec.rs:594)
     test    rdi, rdi
     if mem::size_of::<T>() == 0 || self.cap == 0 { (liballoc/raw_vec.rs:200)
     je      .LBB187_936
     mov     rsi, qword, ptr, [rsp, +, 152]
     test    rsi, rsi
     je      .LBB187_936
     __rust_dealloc(ptr, layout.size(), layout.align()) (liballoc/alloc.rs:102)
     mov     edx, 1
     call    qword, ptr, [rip, +, __rust_dealloc@GOTPCREL]
.LBB187_936:
     lea     rsi, [rsp, +, 16]
 format!("{} is not a function", value.to_string()),
 movdqu  xmm0, xmmword, ptr, [rsp, +, 232]
 movdqa  xmmword, ptr, [rsp, +, 16], xmm0
 mov     rax, qword, ptr, [rsp, +, 248]
 mov     qword, ptr, [rsp, +, 32], rax
 return Err(local_data().allocate_string(
 mov     rdi, rbx
 mov     rdx, r12
 call    jlight::runtime::process::LocalData::allocate_string
.LBB187_937:
 mov     rcx, qword, ptr, [rsp, +, 192]
 mov     qword, ptr, [rcx, +, 8], rax
 mov     qword, ptr, [rcx], 1
     pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) { (libcore/ptr/mod.rs:180)
     mov     rsi, qword, ptr, [rsp, +, 216]
     if mem::size_of::<T>() == 0 || self.cap == 0 { (liballoc/raw_vec.rs:200)
     test    rsi, rsi
     if mem::size_of::<T>() == 0 || self.cap == 0 { (liballoc/raw_vec.rs:200)
     je      .LBB187_918
     mov     rdi, qword, ptr, [rsp, +, 208]
     if let Some((ptr, layout)) = self.current_memory() { (liballoc/raw_vec.rs:594)
     test    rdi, rdi
     je      .LBB187_918
     shl     rsi, 3
     test    rsi, rsi
     je      .LBB187_918
     __rust_dealloc(ptr, layout.size(), layout.align()) (liballoc/alloc.rs:102)
     mov     edx, 8
     call    qword, ptr, [rip, +, __rust_dealloc@GOTPCREL]
     jmp     .LBB187_918
.LBB187_941:
     mov     rax, qword, ptr, [rsp, +, 192]
     Err(v) (libcore/result.rs:1558)
     mov     qword, ptr, [rax, +, 8], rdx
     mov     qword, ptr, [rax], 1
     lea     rdi, [rsp, +, 16]
 }
 call    core::ptr::drop_in_place
 jmp     .LBB187_918
.LBB187_942:
 mov     r14b, 1
 LdaChainIdx { .. } => unimplemented!(),
 lea     rdi, [rip, +, .L__unnamed_92]
 lea     rdx, [rip, +, .L__unnamed_93]
 mov     esi, 15
 call    std::panicking::begin_panic
 jmp     .LBB187_948
.LBB187_943:
 mov     r14b, 1
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_94]
     mov     rdi, r13
     mov     rsi, rcx
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_944:
     mov     r14b, 1
     $crate::panicking::panic($msg) (libcore/macros/mod.rs:10)
     lea     rdi, [rip, +, .L__unnamed_15]
     lea     rdx, [rip, +, .L__unnamed_95]
     mov     esi, 43
     call    qword, ptr, [rip, +, _ZN4core9panicking5panic17h02171c407fa1462fE@GOTPCREL]
     jmp     .LBB187_948
.LBB187_945:
     mov     r14b, 1
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     lea     rdx, [rip, +, .L__unnamed_96]
     mov     rdi, rbx
     mov     rsi, rax
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_946:
     mov     r14b, 1
 StaGlobalDirect(_) => unimplemented!(),
 lea     rdi, [rip, +, .L__unnamed_92]
 lea     rdx, [rip, +, .L__unnamed_97]
 mov     esi, 15
 call    std::panicking::begin_panic
 jmp     .LBB187_948
.LBB187_947:
 mov     r14b, 1
 LdaGlobalDirect(_) => unimplemented!(),
 lea     rdi, [rip, +, .L__unnamed_92]
 lea     rdx, [rip, +, .L__unnamed_98]
 mov     esi, 15
 call    std::panicking::begin_panic
.LBB187_948:
 ud2
.LBB187_951:
 mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_950:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_101]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_949:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_952:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_102]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_953:
     mov     r14b, 1
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     lea     rdx, [rip, +, .L__unnamed_96]
     mov     rdi, rbx
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_954:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_955:
     mov     r14b, 1
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     lea     rdx, [rip, +, .L__unnamed_96]
     mov     rdi, rbx
     mov     rsi, rax
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_956:
     mov     r14b, 1
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_103]
     mov     rdi, r13
     mov     rsi, rcx
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_957:
     Err(AllocError { layout, .. }) => handle_alloc_error(layout), (liballoc/raw_vec.rs:345)
     mov     esi, 4
     mov     rdi, r14
     call    qword, ptr, [rip, +, _ZN5alloc5alloc18handle_alloc_error17h2ec1e7edc0ea879eE@GOTPCREL]
     ud2
.LBB187_958:
     mov     r14b, 1
     $crate::panicking::panic($msg) (libcore/macros/mod.rs:10)
     lea     rdi, [rip, +, .L__unnamed_15]
     lea     rdx, [rip, +, .L__unnamed_95]
     mov     esi, 43
     call    qword, ptr, [rip, +, _ZN4core9panicking5panic17h02171c407fa1462fE@GOTPCREL]
     jmp     .LBB187_948
.LBB187_962:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_104]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_959:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_960:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_961:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_963:
     mov     r14b, 1
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_105]
     mov     rdi, rcx
     mov     rsi, rax
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_964:
     mov     r14b, 1
     $crate::panicking::panic($msg) (libcore/macros/mod.rs:10)
     lea     rdi, [rip, +, .L__unnamed_15]
     lea     rdx, [rip, +, .L__unnamed_95]
     mov     esi, 43
     call    qword, ptr, [rip, +, _ZN4core9panicking5panic17h02171c407fa1462fE@GOTPCREL]
     jmp     .LBB187_948
.LBB187_965:
     panic!("committing memory with mmap() failed"); (src/common/mem.rs:331)
     lea     rdi, [rip, +, .L__unnamed_46]
     lea     rdx, [rip, +, .L__unnamed_47]
     mov     esi, 36
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_966:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_968:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_106]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_967:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_969:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_970:
     mov     r14b, 1
     Err(CapacityOverflow) => capacity_overflow(), (liballoc/raw_vec.rs:344)
     call    qword, ptr, [rip, +, _ZN5alloc7raw_vec17capacity_overflow17hd937678e9b14b783E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_971:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_972:
     mov     r14b, 1
 panic!("Arguments is not an array");
 lea     rdi, [rip, +, .L__unnamed_107]
 lea     rdx, [rip, +, .L__unnamed_108]
 mov     esi, 25
 call    std::panicking::begin_panic
 jmp     .LBB187_948
.LBB187_973:
 mov     r14b, 1
 panic!("Arguments is not an array");
 lea     rdi, [rip, +, .L__unnamed_107]
 lea     rdx, [rip, +, .L__unnamed_109]
 mov     esi, 25
 call    std::panicking::begin_panic
 jmp     .LBB187_948
.LBB187_974:
 mov     r14b, 1
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     lea     rdx, [rip, +, .L__unnamed_96]
     mov     rdi, rbx
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_975:
     mov     r14b, 1
     lea     rdx, [rip, +, .L__unnamed_96]
     mov     rdi, rbx
     mov     rsi, rax
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_979:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_110]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_980:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_982:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_976:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_977:
     mov     r14b, 1
     $crate::panicking::panic($msg) (libcore/macros/mod.rs:10)
     lea     rdi, [rip, +, .L__unnamed_15]
     lea     rdx, [rip, +, .L__unnamed_95]
     mov     esi, 43
     call    qword, ptr, [rip, +, _ZN4core9panicking5panic17h02171c407fa1462fE@GOTPCREL]
     jmp     .LBB187_948
.LBB187_981:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_111]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_978:
     mov     r14b, 1
     $crate::panicking::panic($msg) (libcore/macros/mod.rs:10)
     lea     rdi, [rip, +, .L__unnamed_15]
     lea     rdx, [rip, +, .L__unnamed_95]
     mov     esi, 43
     call    qword, ptr, [rip, +, _ZN4core9panicking5panic17h02171c407fa1462fE@GOTPCREL]
     jmp     .LBB187_948
.LBB187_985:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_986:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_983:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_984:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_987:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_990:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_988:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_989:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_991:
     mov     r14b, 1
 frame.rax = Value::new_int(lhs.as_int32() % rhs.as_int32());
 lea     rdi, [rip, +, str.3]
 lea     rdx, [rip, +, .L__unnamed_112]
 mov     esi, 48
 call    qword, ptr, [rip, +, _ZN4core9panicking5panic17h02171c407fa1462fE@GOTPCREL]
 jmp     .LBB187_948
.LBB187_992:
 mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_993:
     mov     r14b, 1
     $crate::panicking::panic($msg) (libcore/macros/mod.rs:10)
     lea     rdi, [rip, +, .L__unnamed_15]
     lea     rdx, [rip, +, .L__unnamed_113]
     mov     esi, 43
     call    qword, ptr, [rip, +, _ZN4core9panicking5panic17h02171c407fa1462fE@GOTPCREL]
     jmp     .LBB187_948
.LBB187_994:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_114]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_995:
     mov     r14b, 1
     $crate::panicking::panic($msg) (libcore/macros/mod.rs:10)
     lea     rdi, [rip, +, .L__unnamed_15]
     lea     rdx, [rip, +, .L__unnamed_113]
     mov     esi, 43
     call    qword, ptr, [rip, +, _ZN4core9panicking5panic17h02171c407fa1462fE@GOTPCREL]
     jmp     .LBB187_948
.LBB187_996:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_997:
     mov     r14b, 1
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_115]
     mov     rdi, rcx
     mov     rsi, rax
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_998:
     mov     r14b, 1
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     lea     rdx, [rip, +, .L__unnamed_96]
     mov     rdi, rbx
     mov     rsi, rax
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1000:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_999:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1001:
     mov     r14b, 1
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_116]
     mov     rdi, r13
     mov     rsi, rcx
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1002:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1003:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1004:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1005:
     mov     r14b, 1
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     lea     rdx, [rip, +, .L__unnamed_96]
     mov     rdi, rbx
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1006:
     mov     r14b, 1
 unreachable!();
 lea     rdi, [rip, +, .L__unnamed_117]
 lea     rdx, [rip, +, .L__unnamed_118]
 mov     esi, 40
 call    std::panicking::begin_panic
 jmp     .LBB187_948
.LBB187_1007:
 mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1010:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_119]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1008:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1009:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1011:
     mov     r14b, 1
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_120]
     mov     rdi, r13
     mov     rsi, rax
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1012:
     mov     r14b, 1
 unreachable!();
 lea     rdi, [rip, +, .L__unnamed_117]
 lea     rdx, [rip, +, .L__unnamed_121]
 mov     esi, 40
 call    std::panicking::begin_panic
 jmp     .LBB187_948
.LBB187_1013:
 mov     r14b, 1
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_122]
     mov     rdi, rcx
     mov     rsi, rax
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1014:
     mov     r14b, 1
     &(*slice)[self] (libcore/slice/mod.rs:2871)
     lea     rdx, [rip, +, .L__unnamed_96]
     mov     rdi, rbx
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1015:
     &mut (*slice)[self] (libcore/slice/mod.rs:2877)
     lea     rdx, [rip, +, .L__unnamed_123]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1017:
     lea     rdx, [rip, +, .L__unnamed_124]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1019:
     lea     rdx, [rip, +, .L__unnamed_125]
     mov     rdi, r13
     call    qword, ptr, [rip, +, _ZN4core9panicking18panic_bounds_check17ha0668dcff6357ef4E@GOTPCREL]
     jmp     .LBB187_948
.LBB187_1016:
     mov     r14b, 1
     assert!(self.is_cell()); (src/runtime/value.rs:200)
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1018:
     mov     r14b, 1
     lea     rdi, [rip, +, .L__unnamed_99]
     lea     rdx, [rip, +, .L__unnamed_100]
     mov     esi, 32
     call    std::panicking::begin_panic
     jmp     .LBB187_948
.LBB187_1020:
     jmp     .LBB187_1032
.LBB187_1021:
     jmp     .LBB187_1032
.LBB187_1022:
     jmp     .LBB187_1032
.LBB187_1023:
     jmp     .LBB187_1032
.LBB187_1024:
     jmp     .LBB187_1032
.LBB187_1025:
     jmp     .LBB187_1032
.LBB187_1026:
     jmp     .LBB187_1032
.LBB187_1027:
     jmp     .LBB187_1032
.LBB187_1028:
     jmp     .LBB187_1035
.LBB187_1029:
     jmp     .LBB187_1032
.LBB187_1030:
     jmp     .LBB187_1032
.LBB187_1031:
.LBB187_1032:
     mov     rbx, rax
     lea     rdi, [rsp, +, 16]
     call    core::ptr::drop_in_place
     jmp     .LBB187_1054
.LBB187_1033:
     mov     rbx, rax
     mov     r14b, 1
 }
 test    r14b, r14b
 jne     .LBB187_1054
 jmp     .LBB187_1055
.LBB187_1034:
.LBB187_1035:
 mov     rbx, rax
 lea     rdi, [rsp, +, 144]
     } (src/runtime/frame.rs:83)
     call    core::ptr::drop_in_place
     jmp     .LBB187_1054
.LBB187_1036:
     mov     rbx, rax
     mov     r14b, 1
 }
 test    r14b, r14b
 jne     .LBB187_1054
 jmp     .LBB187_1055
.LBB187_1037:
 mov     rbx, rax
 lea     rdi, [rsp, +, 16]
 }
 call    core::ptr::drop_in_place
 jmp     .LBB187_1054
.LBB187_1038:
 jmp     .LBB187_1040
.LBB187_1039:
.LBB187_1040:
 mov     rbx, rax
 lea     rdi, [rsp, +, 144]
 call    core::ptr::drop_in_place
 lea     rdi, [rsp, +, 208]
 jmp     .LBB187_1052
.LBB187_1041:
 mov     rbx, rax
 lea     rdi, [rsp, +, 208]
 jmp     .LBB187_1052
.LBB187_1042:
 mov     rbx, rax
 lea     rdi, [rsp, +, 232]
 format!("{}{}", lhs.to_string(), rhs.to_string()),
 call    core::ptr::drop_in_place
 jmp     .LBB187_1044
.LBB187_1043:
 mov     rbx, rax
.LBB187_1044:
 lea     rdi, [rsp, +, 208]
 call    core::ptr::drop_in_place
 jmp     .LBB187_1054
.LBB187_1045:
 mov     rbx, rax
 lea     rdi, [rsp, +, 208]
 jmp     .LBB187_1052
.LBB187_1046:
 mov     rbx, rax
 mov     r14b, 1
 }
 test    r14b, r14b
 jne     .LBB187_1054
 jmp     .LBB187_1055
.LBB187_1047:
 mov     rbx, rax
 mov     r14b, 1
 test    r14b, r14b
 jne     .LBB187_1054
 jmp     .LBB187_1055
.LBB187_1048:
 mov     rbx, rax
 test    r14b, r14b
 jne     .LBB187_1054
 jmp     .LBB187_1055
.LBB187_1049:
 mov     rbx, rax
 test    r14b, r14b
 jne     .LBB187_1054
 jmp     .LBB187_1055
.LBB187_1051:
 mov     rbx, rax
 lea     rdi, [rsp, +, 16]
.LBB187_1052:
 call    core::ptr::drop_in_place
 jmp     .LBB187_1054
.LBB187_1053:
 mov     rbx, rax
 mov     r14b, 1
 test    r14b, r14b
 je      .LBB187_1055
.LBB187_1054:
 mov     rdi, qword, ptr, [rsp, +, 184]
 call    core::ptr::drop_in_place
.LBB187_1055:
 mov     rdi, rbx
 call    _Unwind_Resume
 ud2
