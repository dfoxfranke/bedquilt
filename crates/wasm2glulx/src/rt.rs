use crate::common::*;
use glulx_asm::concise::*;

use bytes::{BufMut, BytesMut};
pub struct RuntimeLabels {
    pub swap: Label,
    pub swaps: Label,
    pub memload64: Label,
    pub memload32: Label,
    pub memload16: Label,
    pub memload8: Label,
    pub memstore64: Label,
    pub memstore32: Label,
    pub memstore16: Label,
    pub memstore8: Label,
    pub swaparray: Label,
    pub swapunistr: Label,
    pub divu: Label,
    pub remu: Label,
    pub rotl: Label,
    pub rotr: Label,
    pub clz: Label,
    pub ctz: Label,
    pub popcnt: Label,
    pub eqz: Label,
    pub eq: Label,
    pub ne: Label,
    pub lt: Label,
    pub ltu: Label,
    pub gt: Label,
    pub gtu: Label,
    pub le: Label,
    pub leu: Label,
    pub ge: Label,
    pub geu: Label,
    pub add64: Label,
    pub sub64: Label,
    pub mul64: Label,
    pub and64: Label,
    pub or64: Label,
    pub xor64: Label,
    pub shl64: Label,
    pub shr64: Label,
    pub shru64: Label,
    pub rotl64: Label,
    pub rotr64: Label,
    pub eqz64: Label,
    pub eq64: Label,
    pub ne64: Label,
    pub lt64: Label,
    pub ltu64: Label,
    pub gt64: Label,
    pub gtu64: Label,
    pub le64: Label,
    pub leu64: Label,
    pub ge64: Label,
    pub geu64: Label,
    pub clz64: Label,
    pub ctz64: Label,
    pub popcnt64: Label,
    pub trapjump: Label,
    pub table_init: Label,
    pub data_init: Label,
}

impl RuntimeLabels {
    pub fn new(gen: &mut LabelGenerator) -> Self {
        RuntimeLabels {
            swap: gen.gen("rt_swap"),
            swaps: gen.gen("rt_swaps"),
            memload64: gen.gen("rt_memload64"),
            memload32: gen.gen("rt_memload32"),
            memload16: gen.gen("rt_memload16"),
            memload8: gen.gen("rt_memload8"),
            memstore64: gen.gen("rt_memload64"),
            memstore32: gen.gen("rt_memload32"),
            memstore16: gen.gen("rt_memload16"),
            memstore8: gen.gen("rt_memload8"),
            swaparray: gen.gen("rt_swaparray"),
            swapunistr: gen.gen("rt_swapunistr"),
            divu: gen.gen("rt_divu"),
            remu: gen.gen("rt_remu"),
            rotl: gen.gen("rt_rotl"),
            rotr: gen.gen("rt_rotr"),
            clz: gen.gen("rt_clz"),
            ctz: gen.gen("rt_ctz"),
            popcnt: gen.gen("rt_popcnt"),
            eqz: gen.gen("rt_eqz"),
            eq: gen.gen("rt_eq"),
            ne: gen.gen("rt_ne"),
            lt: gen.gen("rt_lt"),
            ltu: gen.gen("rt_ltu"),
            gt: gen.gen("rt_gt"),
            gtu: gen.gen("rt_gtu"),
            le: gen.gen("rt_le"),
            leu: gen.gen("rt_leu"),
            ge: gen.gen("rt_ge"),
            geu: gen.gen("rt_geu"),
            add64: gen.gen("rt_add64"),
            sub64: gen.gen("rt_sub64"),
            mul64: gen.gen("rt_mul64"),
            and64: gen.gen("rt_and64"),
            or64: gen.gen("rt_or64"),
            xor64: gen.gen("rt_xor64"),
            shl64: gen.gen("rt_shl64"),
            shr64: gen.gen("rt_shr64"),
            shru64: gen.gen("rt_shru64"),
            rotl64: gen.gen("rt_rotl64"),
            rotr64: gen.gen("rt_rotr64"),
            eqz64: gen.gen("rt_eqz64"),
            eq64: gen.gen("rt_eq64"),
            ne64: gen.gen("rt_ne64"),
            lt64: gen.gen("rt_lt64"),
            ltu64: gen.gen("rt_ltu64"),
            gt64: gen.gen("rt_gt64"),
            gtu64: gen.gen("rt_gtu64"),
            le64: gen.gen("rt_le64"),
            leu64: gen.gen("rt_leu64"),
            ge64: gen.gen("rt_ge64"),
            geu64: gen.gen("rt_geu64"),
            clz64: gen.gen("rt_clz64"),
            ctz64: gen.gen("rt_ctz64"),
            popcnt64: gen.gen("rt_popcnt64"),
            trapjump: gen.gen("rt_trampjump"),
            table_init: gen.gen("rt_table_init"),
            data_init: gen.gen("rt_data_init"),
        }
    }
}

fn gen_swap(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.swap),
        fnhead_local(1),
        shiftl(lloc(0), imm(16), push()),
        ushiftr(lloc(0), imm(16), push()),
        bitor(pop(), pop(), sloc(0)),
        bitand(lloc(0), uimm(0xff00ff00), push()),
        ushiftr(pop(), imm(8), push()),
        bitand(lloc(0), uimm(0x00ff00ff), push()),
        shiftl(pop(), imm(8), push()),
        bitor(pop(), pop(), push()),
        ret(pop())
    );
}

fn gen_swaps(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.swaps),
        fnhead_local(1),
        bitand(lloc(0), uimm(0xff00ff00), push()),
        ushiftr(pop(), imm(8), push()),
        bitand(lloc(0), uimm(0x00ff00ff), push()),
        shiftl(pop(), imm(8), push()),
        bitor(pop(), pop(), push()),
        ret(pop())
    );
}

fn gen_memload64(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.memload64),
        fnhead_local(1),
        add(lloc(0), imm(1), push()),
        aload(
            pop(),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            push()
        ),
        callfi(
            imml(ctx.rt.swap),
            pop(),
            storel(ctx.layout.hi_return().addr)
        ),
        aload(
            lloc(0),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            push()
        ),
        tailcall(imml(ctx.rt.swap), imm(1)),
    )
}

fn gen_memload32(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.memload32),
        fnhead_local(1),
        aload(
            lloc(0),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            push()
        ),
        tailcall(imml(ctx.rt.swap), imm(1)),
    );
}

fn gen_memload16(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.memload16),
        fnhead_local(1),
        aloads(
            lloc(0),
            imml_off_shift(ctx.layout.memory().addr, 0, 1),
            push()
        ),
        tailcall(imml(ctx.rt.swaps), imm(1)),
    );
}

fn gen_memload8(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.memload8),
        fnhead_local(1),
        aloadb(lloc(0), imml(ctx.layout.memory().addr), push()),
        ret(pop()),
    );
}

fn gen_memstore64(ctx: &mut Context) {
    let addr = 0;
    let val_hi = 1;
    let val_lo = 2;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memstore64),
        fnhead_local(3),
        callfi(imml(ctx.rt.swap), lloc(val_lo), push()),
        astore(
            lloc(addr),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            pop()
        ),
        callfi(imml(ctx.rt.swap), lloc(val_hi), push()),
        add(lloc(addr), imm(1), push()),
        astore(pop(), imml_off_shift(ctx.layout.memory().addr, 0, 2), pop()),
        ret(imm(0)),
    );
}

fn gen_memstore32(ctx: &mut Context) {
    let addr = 0;
    let val = 1;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memstore32),
        fnhead_local(2),
        callfi(imml(ctx.rt.swap), lloc(val), push()),
        astore(
            lloc(addr),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            pop()
        ),
        ret(imm(0)),
    );
}

fn gen_memstore16(ctx: &mut Context) {
    let addr = 0;
    let val = 1;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memstore16),
        fnhead_local(2),
        callfi(imml(ctx.rt.swaps), lloc(val), push()),
        astores(
            lloc(addr),
            imml_off_shift(ctx.layout.memory().addr, 0, 1),
            pop()
        ),
        ret(imm(0)),
    );
}

fn gen_memstore8(ctx: &mut Context) {
    let addr = 0;
    let val = 1;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.memstore8),
        fnhead_local(2),
        astore(lloc(addr), imml(ctx.layout.memory().addr), lloc(val)),
        ret(imm(0)),
    );
}

fn gen_swaparray(ctx: &mut Context) {
    let arraybase = 0;
    let arraylen = 1;

    let loop_head = ctx.gen.gen("swaparray_loop_head");
    let loop_end = ctx.gen.gen("swaparray_loop_end");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.swaparray),
        fnhead_local(3),
        label(loop_head),
        jz(lloc(arraylen), loop_end),
        aload(
            lloc(arraybase),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            push()
        ),
        callfi(imml(ctx.rt.swap), pop(), push()),
        astore(
            lloc(arraybase),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            pop()
        ),
        add(lloc(arraybase), imm(4), sloc(arraybase)),
        sub(lloc(arraylen), imm(1), sloc(arraylen)),
        jump(loop_head),
        label(loop_end),
        ret(imm(0)),
    );
}

fn gen_swapunistr(ctx: &mut Context) {
    let arraybase = 0;
    let curword = 1;

    let loop_head = ctx.gen.gen("swapunistr_loop_head");
    let loop_end = ctx.gen.gen("swapunistr_loop_end");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.swapunistr),
        fnhead_local(2),
        label(loop_head),
        aload(
            lloc(arraybase),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            sloc(curword)
        ),
        jz(lloc(curword), loop_end),
        callfi(imml(ctx.rt.swap), lloc(curword), push()),
        astore(
            lloc(arraybase),
            imml_off_shift(ctx.layout.memory().addr, 0, 2),
            pop()
        ),
        add(lloc(arraybase), imm(4), sloc(arraybase)),
        jump(loop_head),
        label(loop_end),
        ret(imm(0)),
    );
}

fn gen_divu(ctx: &mut Context) {
    let divs = ctx.gen.gen("divu_divs");
    let div1 = ctx.gen.gen("divu_div1");
    let dont_add1 = ctx.gen.gen("divu_dontadd1");

    let n = 0; // numerator
    let d = 1; // denominator
    let n_lo = 2; // n & 0x7fffffff
    let hi_quot = 3; // 0x7fffffff / d
    let hi_rem = 4; // 0x7fffffff % d
    let lo_quot = 5; // n_lo / d
    let lo_rem = 6; // n_lo % d

    push_all!(
        ctx.rom_items,
        label(ctx.rt.divu),
        fnhead_local(7),
        // If d > n, quotient is 0
        jgtu_ret(lloc(d), lloc(n), false),
        // If d fills 32 bits, getting here from previous test means n does too.
        // So the quotient must be 1.
        jlt_ret(lloc(d), imm(0), true),
        // d is at most 31 bits. If n also fits in 31 bits, just do signed division.
        jge(lloc(n), imm(0), divs),
        // Treat division by 1 as a special case so that afterward we can assume
        // 1 / d = 0 and 1 % d = 1.
        jeq(lloc(d), imm(1), div1),
        // We have 32-bit n, sub-32-bit d. This is the hard case. Break up n =
        // (n & 0x7fffffff + 0x7fffffff + 1). Take the sum of the quotients,
        // then add 1 if the sum of the remainders > 1.
        bitand(lloc(n), imm(0x7fffffff), sloc(n_lo)),
        div(imm(0x7fffffff), lloc(d), sloc(hi_quot)),
        modulo(imm(0x7fffffff), lloc(d), sloc(hi_rem)),
        div(lloc(n_lo), lloc(d), sloc(lo_quot)),
        modulo(lloc(n_lo), lloc(d), sloc(lo_rem)),
        // Push the sum of the two quotients...
        add(lloc(hi_quot), lloc(lo_quot), push()),
        // ...then push the sum of the three remainders
        add(lloc(hi_rem), lloc(lo_rem), push()),
        add(pop(), imm(1), push()),
        // If the remainder sum >= n, add 1 to the quotient sum, otherwise
        // don't. Either way, that's our result.
        jltu(pop(), lloc(n), dont_add1),
        add(pop(), imm(1), push()),
        label(dont_add1),
        ret(pop()),
        label(divs),
        // Jump here for the 31-bit signed division case.
        div(lloc(n), lloc(d), push()),
        ret(pop()),
        // Jump here for the division-by-1 case.
        label(div1),
        ret(lloc(n))
    );
}

fn gen_remu(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.remu),
        fnhead_local(2),
        callfii(imml(ctx.rt.divu), lloc(0), lloc(1), push()),
        mul(pop(), lloc(1), push()),
        sub(lloc(0), pop(), push()),
        ret(pop())
    )
}

fn gen_rotl(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.rotl),
        fnhead_local(2),
        bitand(lloc(1), imm(0x1f), sloc(1)),
        shiftl(lloc(0), lloc(1), push()),
        sub(imm(32), lloc(1), push()),
        ushiftr(lloc(0), pop(), push()),
        bitor(pop(), pop(), push()),
        ret(pop()),
    )
}

fn gen_rotr(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.rotr),
        fnhead_local(2),
        bitand(lloc(1), imm(0x1f), sloc(1)),
        ushiftr(lloc(0), lloc(1), push()),
        sub(imm(32), lloc(1), push()),
        shiftl(lloc(0), pop(), push()),
        bitor(pop(), pop(), push()),
        ret(pop()),
    )
}

fn gen_clz(ctx: &mut Context) {
    let lead8 = ctx.gen.gen("clz_lead8");
    let lead16 = ctx.gen.gen("clz_lead16");
    let lead24 = ctx.gen.gen("clz_lead24");

    let clz_table = ctx.gen.gen("clz_table");
    let mut table_bytes = BytesMut::with_capacity(256);

    for x in 0u8..=255 {
        table_bytes.put_u8(
            x.leading_zeros()
                .try_into()
                .expect("leading zero count of a u8 should fit in a u8"),
        );
    }

    push_all!(
        ctx.rom_items,
        label(ctx.rt.clz),
        fnhead_local(2),
        ushiftr(lloc(0), imm(24), sloc(1)),
        jz(lloc(1), lead8),
        aloadb(imml(clz_table), lloc(1), push()),
        ret(pop()),
        label(lead8),
        ushiftr(lloc(0), imm(16), sloc(1)),
        jz(lloc(1), lead16),
        aloadb(imml(clz_table), lloc(1), push()),
        add(pop(), imm(8), push()),
        ret(pop()),
        label(lead16),
        ushiftr(lloc(0), imm(8), sloc(1)),
        jz(lloc(1), lead24),
        aloadb(imml(clz_table), lloc(1), push()),
        add(pop(), imm(16), push()),
        ret(pop()),
        label(lead24),
        aloadb(imml(clz_table), lloc(0), push()),
        add(pop(), imm(24), push()),
        ret(pop()),
        label(clz_table),
        blob(table_bytes.freeze()),
    )
}

fn gen_ctz(ctx: &mut Context) {
    let trail8 = ctx.gen.gen("ctz_trail8");
    let trail16 = ctx.gen.gen("ctz_trail16");
    let trail24 = ctx.gen.gen("ctz_trail24");

    let ctz_table = ctx.gen.gen("ctz_table");
    let mut table_bytes = BytesMut::with_capacity(256);

    for x in 0u8..=255 {
        table_bytes.put_u8(
            x.trailing_zeros()
                .try_into()
                .expect("trailing zero count of a u8 should fit in a u8"),
        );
    }

    push_all!(
        ctx.rom_items,
        label(ctx.rt.ctz),
        fnhead_local(2),
        bitand(lloc(0), imm(0xff), sloc(1)),
        jz(lloc(1), trail8),
        aloadb(imml(ctz_table), lloc(1), push()),
        ret(pop()),
        label(trail8),
        ushiftr(lloc(0), imm(8), push()),
        bitand(pop(), imm(0xff), sloc(1)),
        jz(lloc(1), trail16),
        aloadb(imml(ctz_table), lloc(1), push()),
        add(pop(), imm(8), push()),
        ret(pop()),
        label(trail16),
        ushiftr(lloc(0), imm(16), push()),
        bitand(pop(), imm(0xff), sloc(1)),
        jz(lloc(1), trail24),
        aloadb(imml(ctz_table), lloc(1), push()),
        add(pop(), imm(16), push()),
        ret(pop()),
        label(trail24),
        ushiftr(lloc(0), imm(24), push()),
        aloadb(imml(ctz_table), pop(), push()),
        add(pop(), imm(24), push()),
        ret(pop()),
        label(ctz_table),
        blob(table_bytes.freeze()),
    );
}

fn gen_popcnt(ctx: &mut Context) {
    let popcnt_table = ctx.gen.gen("popcnt_table");
    let mut table_bytes = BytesMut::with_capacity(256);

    for x in 0u8..=255 {
        table_bytes.put_u8(
            x.count_ones()
                .try_into()
                .expect("popcnt of a u8 should fit in a u8"),
        );
    }

    push_all!(
        ctx.rom_items,
        label(ctx.rt.popcnt),
        fnhead_local(1),
        bitand(lloc(0), imm(0xff), push()),
        aloadb(imml(popcnt_table), pop(), push()),
        ushiftr(lloc(0), imm(8), push()),
        bitand(pop(), imm(0xff), push()),
        aloadb(imml(popcnt_table), pop(), push()),
        add(pop(), pop(), push()),
        ushiftr(lloc(0), imm(16), push()),
        bitand(pop(), imm(0xff), push()),
        aloadb(imml(popcnt_table), pop(), push()),
        add(pop(), pop(), push()),
        ushiftr(lloc(0), imm(24), push()),
        aloadb(imml(popcnt_table), pop(), push()),
        add(pop(), pop(), push()),
        ret(pop()),
        label(popcnt_table),
        blob(table_bytes.freeze()),
    );
}

fn gen_eqz(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.eqz),
        fnhead_local(1),
        jz_ret(lloc(0), true),
        ret(imm(0))
    )
}

fn gen_eq(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.eq),
        fnhead_local(2),
        jeq_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_ne(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.ne),
        fnhead_local(2),
        jne_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_lt(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.lt),
        fnhead_local(2),
        jlt_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_ltu(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.ltu),
        fnhead_local(2),
        jltu_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_le(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.le),
        fnhead_local(2),
        jle_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_leu(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.leu),
        fnhead_local(2),
        jleu_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_gt(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.gt),
        fnhead_local(2),
        jgt_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_gtu(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.gtu),
        fnhead_local(2),
        jgtu_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_ge(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.ge),
        fnhead_local(2),
        jge_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_geu(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.geu),
        fnhead_local(2),
        jgeu_ret(lloc(0), lloc(1), true),
        ret(imm(0))
    )
}

fn gen_add64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;
    let sum_lo = 4;
    let sum_hi = 5;

    let nocarry = ctx.gen.gen("add64_nocarry");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.add64),
        fnhead_local(6),
        add(lloc(x_lo), lloc(y_lo), sloc(sum_lo)),
        add(lloc(x_hi), lloc(y_hi), sloc(sum_hi)),
        jgeu(lloc(sum_lo), lloc(x_lo), nocarry),
        add(lloc(sum_hi), imm(1), sloc(sum_hi)),
        label(nocarry),
        copy(lloc(sum_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(sum_lo)),
    );
}

fn gen_sub64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;
    let diff_lo = 4;
    let diff_hi = 5;

    let noborrow = ctx.gen.gen("sub64_noborrow");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.sub64),
        fnhead_local(6),
        sub(lloc(x_lo), lloc(y_lo), sloc(diff_lo)),
        sub(lloc(x_hi), lloc(y_hi), sloc(diff_lo)),
        jleu(lloc(diff_lo), lloc(x_lo), noborrow),
        sub(lloc(diff_hi), imm(1), sloc(diff_hi)),
        label(noborrow),
        copy(lloc(diff_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(diff_lo)),
    );
}

fn gen_mul64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    let x_lohi = 4;
    let x_lolo = 5;
    let y_lohi = 6;
    let y_lolo = 7;

    let out_hi = 8;
    let out_lo = 9;

    let tmp = 10;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.mul64),
        fnhead_local(11),
        ushiftr(lloc(x_lo), imm(16), sloc(x_lohi)),
        bitand(lloc(x_lo), imm(0xffff), sloc(x_lolo)),
        ushiftr(lloc(y_lo), imm(16), sloc(y_lohi)),
        bitand(lloc(y_lo), imm(0xffff), sloc(y_lolo)),
        mul(lloc(x_lolo), lloc(y_lolo), sloc(out_lo)),
        ushiftr(lloc(out_lo), imm(16), sloc(tmp)),
        bitand(lloc(out_lo), imm(0xffff), sloc(out_lo)),
        mul(lloc(x_lohi), lloc(y_lolo), push()),
        add(lloc(tmp), pop(), sloc(tmp)),
        bitand(lloc(tmp), imm(0xffff), push()),
        shiftl(pop(), imm(16), push()),
        add(lloc(out_lo), pop(), sloc(out_lo)),
        ushiftr(lloc(tmp), imm(16), sloc(out_hi)),
        ushiftr(lloc(out_lo), imm(16), sloc(tmp)),
        bitand(lloc(out_lo), imm(0xffff), sloc(out_lo)),
        mul(lloc(y_lohi), lloc(x_lolo), push()),
        add(lloc(tmp), pop(), sloc(tmp)),
        bitand(lloc(tmp), imm(0xffff), push()),
        shiftl(pop(), imm(16), push()),
        add(lloc(out_lo), pop(), sloc(out_lo)),
        ushiftr(lloc(tmp), imm(16), push()),
        add(lloc(out_hi), pop(), sloc(out_hi)),
        mul(lloc(x_lohi), lloc(y_lohi), push()),
        add(lloc(out_hi), pop(), sloc(out_hi)),
        mul(lloc(x_hi), lloc(y_lo), push()),
        mul(lloc(x_lo), lloc(y_hi), push()),
        add(pop(), pop(), push()),
        add(lloc(out_hi), pop(), sloc(out_hi)),
        copy(lloc(out_hi), storel(ctx.layout.hi_return().addr)),
        ret(lloc(out_lo)),
    )
}

fn gen_and64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.and64),
        fnhead_local(4),
        bitand(lloc(x_hi), lloc(y_hi), storel(ctx.layout.hi_return().addr)),
        bitand(lloc(x_lo), lloc(y_lo), push()),
        ret(pop())
    );
}

fn gen_or64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.or64),
        fnhead_local(4),
        bitor(lloc(x_hi), lloc(y_hi), storel(ctx.layout.hi_return().addr)),
        bitor(lloc(x_lo), lloc(y_lo), push()),
        ret(pop())
    );
}

fn gen_xor64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.xor64),
        fnhead_local(4),
        bitxor(lloc(x_hi), lloc(y_hi), storel(ctx.layout.hi_return().addr)),
        bitxor(lloc(x_lo), lloc(y_lo), push()),
        ret(pop())
    );
}

fn gen_shl64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let r = 2;

    let shift32 = ctx.gen.gen("shl64_shift32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.shl64),
        fnhead_local(3),
        jgeu(lloc(r), imm(32), shift32),
        shiftl(lloc(x_hi), lloc(r), sloc(x_hi)),
        sub(imm(32), lloc(r), push()),
        ushiftr(lloc(x_lo), pop(), push()),
        bitor(lloc(x_hi), pop(), storel(ctx.layout.hi_return().addr)),
        shiftl(lloc(x_lo), lloc(r), push()),
        ret(pop()),
        label(shift32),
        sub(lloc(r), imm(32), push()),
        shiftl(lloc(x_lo), pop(), storel(ctx.layout.hi_return().addr)),
        ret(imm(0)),
    )
}

fn gen_shr64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let r = 2;

    let shift32 = ctx.gen.gen("shr64_shift32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.shr64),
        fnhead_local(3),
        jgeu(lloc(r), imm(32), shift32),
        sshiftr(lloc(x_hi), lloc(r), storel(ctx.layout.hi_return().addr)),
        ushiftr(lloc(x_lo), lloc(r), sloc(x_lo)),
        sub(imm(32), lloc(r), push()),
        shiftl(lloc(x_hi), pop(), push()),
        bitor(lloc(x_lo), pop(), push()),
        ret(pop()),
        label(shift32),
        sshiftr(lloc(x_hi), imm(31), push()),
        copy(pop(), storel(ctx.layout.hi_return().addr)),
        sub(lloc(r), imm(32), push()),
        sshiftr(lloc(x_hi), pop(), push()),
        ret(pop()),
    )
}

fn gen_shru64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let r = 2;

    let shift32 = ctx.gen.gen("shru64_shift32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.shru64),
        fnhead_local(3),
        jgeu(lloc(r), imm(32), shift32),
        ushiftr(lloc(x_hi), lloc(r), storel(ctx.layout.hi_return().addr)),
        ushiftr(lloc(x_lo), lloc(r), sloc(x_lo)),
        sub(imm(32), lloc(r), push()),
        shiftl(lloc(x_hi), pop(), push()),
        bitor(lloc(x_lo), pop(), push()),
        ret(pop()),
        label(shift32),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        sub(lloc(r), imm(32), push()),
        ushiftr(lloc(x_hi), pop(), push()),
        ret(pop()),
    )
}

fn gen_rotl64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let r = 2;
    let x_hi_shifted = 3;
    let x_lo_shifted = 4;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.rotl64),
        fnhead_local(5),
        bitand(lloc(r), imm(0x3f), sloc(r)),
        callfiii(
            imml(ctx.rt.shl64),
            lloc(x_hi),
            lloc(x_lo),
            lloc(r),
            sloc(x_lo_shifted)
        ),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi_shifted)),
        sub(imm(64), lloc(r), push()),
        callfiii(imml(ctx.rt.shru64), lloc(x_hi), lloc(x_lo), pop(), push()),
        bitor(
            lloc(x_hi_shifted),
            derefl(ctx.layout.hi_return().addr),
            storel(ctx.layout.hi_return().addr)
        ),
        bitor(lloc(x_lo_shifted), pop(), push()),
        ret(pop()),
    );
}

fn gen_rotr64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let r = 2;
    let x_hi_shifted = 3;
    let x_lo_shifted = 4;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.rotr64),
        fnhead_local(5),
        bitand(lloc(r), imm(0x3f), sloc(r)),
        callfiii(
            imml(ctx.rt.shru64),
            lloc(x_hi),
            lloc(x_lo),
            lloc(r),
            sloc(x_lo_shifted)
        ),
        copy(derefl(ctx.layout.hi_return().addr), sloc(x_hi_shifted)),
        sub(imm(64), lloc(r), push()),
        callfiii(imml(ctx.rt.shl64), lloc(x_hi), lloc(x_lo), pop(), push()),
        bitor(
            lloc(x_hi_shifted),
            derefl(ctx.layout.hi_return().addr),
            storel(ctx.layout.hi_return().addr)
        ),
        bitor(lloc(x_lo_shifted), pop(), push()),
        ret(pop()),
    );
}

fn gen_eqz64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.eqz64),
        fnhead_local(2),
        jnz_ret(lloc(x_hi), false),
        jnz_ret(lloc(x_lo), false),
        ret(imm(1))
    );
}

fn gen_eq64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.eq64),
        fnhead_local(4),
        jne_ret(lloc(x_hi), lloc(y_hi), false),
        jne_ret(lloc(x_lo), lloc(y_lo), false),
        ret(imm(1))
    );
}

fn gen_ne64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.ne64),
        fnhead_local(4),
        jne_ret(lloc(x_hi), lloc(y_hi), true),
        jne_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0))
    );
}

fn gen_lt64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.lt64),
        fnhead_local(4),
        jlt_ret(lloc(x_hi), lloc(y_hi), true),
        jlt_ret(lloc(y_hi), lloc(x_hi), false),
        jlt_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_ltu64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.ltu64),
        fnhead_local(4),
        jltu_ret(lloc(x_hi), lloc(y_hi), true),
        jltu_ret(lloc(y_hi), lloc(x_hi), false),
        jltu_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_gt64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.gt64),
        fnhead_local(4),
        jgt_ret(lloc(x_hi), lloc(y_hi), true),
        jgt_ret(lloc(y_hi), lloc(x_hi), false),
        jgt_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_gtu64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.gtu64),
        fnhead_local(4),
        jgtu_ret(lloc(x_hi), lloc(y_hi), true),
        jgtu_ret(lloc(y_hi), lloc(x_hi), false),
        jgtu_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_le64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.le64),
        fnhead_local(4),
        jlt_ret(lloc(x_hi), lloc(y_hi), true),
        jlt_ret(lloc(y_hi), lloc(x_hi), false),
        jle_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_leu64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.leu64),
        fnhead_local(4),
        jltu_ret(lloc(x_hi), lloc(y_hi), true),
        jltu_ret(lloc(y_hi), lloc(x_hi), false),
        jleu_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_ge64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.ge64),
        fnhead_local(4),
        jgt_ret(lloc(x_hi), lloc(y_hi), true),
        jgt_ret(lloc(y_hi), lloc(x_hi), false),
        jge_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_geu64(ctx: &mut Context) {
    let x_hi = 0;
    let x_lo = 1;
    let y_hi = 2;
    let y_lo = 3;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.geu64),
        fnhead_local(4),
        jgtu_ret(lloc(x_hi), lloc(y_hi), true),
        jgtu_ret(lloc(y_hi), lloc(x_hi), false),
        jgeu_ret(lloc(x_lo), lloc(y_lo), true),
        ret(imm(0)),
    );
}

fn gen_clz64(ctx: &mut Context) {
    let hi = 0;
    let lo = 1;
    let hi_clz = 2;

    let hi32 = ctx.gen.gen("clz64_hi32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.clz64),
        fnhead_local(3),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        callfi(imml(ctx.rt.clz), lloc(hi), sloc(hi_clz)),
        jeq(lloc(hi_clz), imm(32), hi32),
        ret(lloc(hi_clz)),
        label(hi32),
        callfi(imml(ctx.rt.clz), lloc(lo), push()),
        add(imm(32), pop(), push()),
        ret(pop()),
    )
}

fn gen_ctz64(ctx: &mut Context) {
    let hi = 0;
    let lo = 1;
    let lo_clz = 2;

    let lo32 = ctx.gen.gen("ctz64_lo32");

    push_all!(
        ctx.rom_items,
        label(ctx.rt.ctz64),
        fnhead_local(3),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        callfi(imml(ctx.rt.ctz), lloc(lo), sloc(lo_clz)),
        jeq(lloc(lo_clz), imm(32), lo32),
        ret(lloc(lo_clz)),
        label(lo32),
        callfi(imml(ctx.rt.ctz), lloc(hi), push()),
        add(imm(32), pop(), push()),
        ret(pop()),
    )
}

fn gen_popcnt64(ctx: &mut Context) {
    let hi = 0;
    let lo = 1;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.popcnt64),
        fnhead_local(2),
        copy(imm(0), storel(ctx.layout.hi_return().addr)),
        callfi(imml(ctx.rt.popcnt), lloc(hi), push()),
        callfi(imml(ctx.rt.popcnt), lloc(lo), push()),
        add(pop(), pop(), push()),
        ret(pop()),
    )
}

fn gen_trap(ctx: &mut Context) {
    push_all!(
        ctx.rom_items,
        label(ctx.rt.trapjump),
        debugtrap(derefl(ctx.layout.trap().code)),
        quit(),
    )
}

fn gen_table_init(ctx: &mut Context) {
    let n = 6;
    let elem_offset = 5;
    let table_offset = 4;
    let table_addr = 3;
    let table_size = 2;
    let elem_addr = 1;
    let elem_size = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.table_init),
        fnhead_local(7),
        copy(
            uimm(TrapCode::OutOfBoundsTableAccess.into()),
            storel(ctx.layout.trap().code)
        ),
        jgtu(lloc(elem_offset), lloc(elem_size), ctx.rt.trapjump),
        sub(lloc(elem_size), lloc(elem_offset), push()),
        jgtu(lloc(n), pop(), ctx.rt.trapjump),
        jgtu(lloc(table_offset), lloc(table_size), ctx.rt.trapjump),
        sub(lloc(table_size), lloc(table_offset), push()),
        jgtu(lloc(n), pop(), ctx.rt.trapjump),
        shiftl(lloc(table_offset), imm(2), push()),
        add(pop(), lloc(table_addr), push()),
        shiftl(lloc(elem_offset), imm(2), push()),
        add(pop(), lloc(elem_addr), push()),
        shiftl(lloc(n), imm(2), push()),
        mcopy(pop(), pop(), pop()),
        ret(imm(0)),
    )
}

fn gen_data_init(ctx: &mut Context) {
    let n = 4;
    let data_offset = 3;
    let mem_offset = 2;
    let data_addr = 1;
    let data_size = 0;

    push_all!(
        ctx.rom_items,
        label(ctx.rt.data_init),
        fnhead_local(5),
        copy(
            uimm(TrapCode::OutOfBoundsMemoryAccess.into()),
            storel(ctx.layout.trap().code)
        ),
        jgtu(lloc(data_offset), lloc(data_size), ctx.rt.trapjump),
        sub(lloc(data_size), lloc(data_offset), push()),
        jgtu(lloc(n), pop(), ctx.rt.trapjump),
        jgtu(
            lloc(mem_offset),
            derefl(ctx.layout.memory().cur_size),
            ctx.rt.trapjump
        ),
        sub(
            derefl(ctx.layout.memory().cur_size),
            lloc(mem_offset),
            push()
        ),
        jgtu(lloc(n), pop(), ctx.rt.trapjump),
        add(lloc(mem_offset), imml(ctx.layout.memory().addr), push()),
        add(lloc(data_offset), lloc(data_addr), push()),
        mcopy(lloc(n), pop(), pop()),
        ret(imm(0)),
    )
}

pub fn gen_rt(ctx: &mut Context) {
    gen_swap(ctx);
    gen_swaps(ctx);
    gen_memload64(ctx);
    gen_memload32(ctx);
    gen_memload16(ctx);
    gen_memload8(ctx);
    gen_memstore64(ctx);
    gen_memstore32(ctx);
    gen_memstore16(ctx);
    gen_memstore8(ctx);
    gen_swaparray(ctx);
    gen_swapunistr(ctx);
    gen_divu(ctx);
    gen_remu(ctx);
    gen_rotl(ctx);
    gen_rotr(ctx);
    gen_clz(ctx);
    gen_ctz(ctx);
    gen_popcnt(ctx);
    gen_eqz(ctx);
    gen_eq(ctx);
    gen_ne(ctx);
    gen_lt(ctx);
    gen_ltu(ctx);
    gen_gt(ctx);
    gen_gtu(ctx);
    gen_le(ctx);
    gen_leu(ctx);
    gen_ge(ctx);
    gen_geu(ctx);
    gen_add64(ctx);
    gen_sub64(ctx);
    gen_mul64(ctx);
    gen_and64(ctx);
    gen_or64(ctx);
    gen_xor64(ctx);
    gen_shl64(ctx);
    gen_shr64(ctx);
    gen_shru64(ctx);
    gen_rotl64(ctx);
    gen_rotr64(ctx);
    gen_eqz64(ctx);
    gen_eq64(ctx);
    gen_ne64(ctx);
    gen_lt64(ctx);
    gen_ltu64(ctx);
    gen_gt64(ctx);
    gen_gtu64(ctx);
    gen_le64(ctx);
    gen_leu64(ctx);
    gen_ge64(ctx);
    gen_geu64(ctx);
    gen_clz64(ctx);
    gen_ctz64(ctx);
    gen_popcnt64(ctx);
    gen_trap(ctx);
    gen_table_init(ctx);
    gen_data_init(ctx);
}
