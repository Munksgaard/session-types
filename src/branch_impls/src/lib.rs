#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

use syntax::codemap::Span;
use syntax::ptr::P;
use syntax::ast::{TokenTree, Item};
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacEager};
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::util::small_vector::SmallVector;
use syntax::ast::Lit_;
use rustc_plugin::Registry;

fn commalist(n: u8, before: &str, after: &str) -> String {
    let mut buf = "".to_string();
    for i in 1..n {
        buf.push_str(&format!("{}B{}{}, ", before, i, after));
    }
    buf.push_str(&format!("{}B{}{}", before, n, after));

    return buf;
}

fn branch_struct(n: u8) -> String {
    let mut buf = format!("pub enum Branch{}<", n);
    buf.push_str(&commalist(n, "", ""));
    buf.push_str("> { ");
    for i in 1..n+1 {
        buf.push_str(&format!("B{}(B{}), ", i, i));
    }

    buf.push_str(" }");

    return buf;
}

fn hasdual_choose_impl(n: u8) -> String {
    let mut buf = "unsafe impl <".to_string();
    buf.push_str(&commalist(n, "", ": HasDual"));
    buf.push_str("> HasDual for Choose<(");
    buf.push_str(&commalist(n, "", ""));
    buf.push_str(")> { type Dual = Offer<(");
    buf.push_str(&commalist(n, "", "::Dual"));
    buf.push_str(")>; }");

    return buf;
}

fn hasdual_offer_impl(n: u8) -> String {
    let mut buf = "unsafe impl <".to_string();
    buf.push_str(&commalist(n, "", ": HasDual"));
    buf.push_str("> HasDual for Offer<(");
    buf.push_str(&commalist(n, "", ""));
    buf.push_str(")> { type Dual = Choose<(");
    buf.push_str(&commalist(n, "", "::Dual"));
    buf.push_str(")>; }");

    return buf;
}

fn choose_impl(n: u8) -> String {
    let mut buf = "impl <E, ".to_string();
    buf.push_str(&commalist(n, "", ""));
    buf.push_str("> Chan<E, Choose<(");
    buf.push_str(&commalist(n, "", ""));
    buf.push_str(")>> { ");

    for i in 1..n+1 {
        buf.push_str("#[must_use] ");
        buf.push_str(&format!("pub fn sel{}(self) -> Chan<E, B{}> {{ ", i, i));
        buf.push_str(&format!("unsafe {{ write_chan(&self, {}); transmute(self) }} }} ", i));
    }

    buf.push_str("}");

    return buf;
}

fn offer_impl(n: u8) -> String {
    let mut buf = "impl <E, ".to_string();
    buf.push_str(&commalist(n, "", ""));
    buf.push_str("> Chan<E, Offer<(");
    buf.push_str(&commalist(n, "", ""));
    buf.push_str(&format!(")>> {{ #[must_use] pub fn offer(self) -> Branch{}<", n));
    buf.push_str(&commalist(n, "Chan<E, ", ">"));
    buf.push_str("> { unsafe { match read_chan(&self) { ");

    for i in 1..(n+1) {
        buf.push_str(&format!("{}u8 => Branch{}::B{}(transmute(self)), ", i, n, i));
    }

    buf.push_str("_ => unreachable!() ");
    buf.push_str("} } } }");

    return buf;
}

fn impl_branch(cx: &mut ExtCtxt, n: u8) -> SmallVector<P<Item>> {
    let mut v = SmallVector::zero();

    v.push(cx.parse_item(branch_struct(n)));
    v.push(cx.parse_item(hasdual_choose_impl(n)));
    v.push(cx.parse_item(hasdual_offer_impl(n)));
    v.push(cx.parse_item(offer_impl(n)));
    v.push(cx.parse_item(choose_impl(n)));

    return v;
}

fn branch_impls(cx: &mut ExtCtxt, sp: Span, tts: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(tts);
    let n = match parser.parse_lit() {
        Ok(lit) => match lit.node {
            Lit_::LitInt(n, _) => n,
            _ => {
                cx.span_err(lit.span, "Expected literal integer");
                return DummyResult::any(sp);
            }
        },
        Err(mut e) => {
            e.emit();
            return DummyResult::any(sp);
        }
    } as u8;

    let mut v = SmallVector::zero();

    for i in 2..n+1 {
        v.push_all(impl_branch(cx, i));
    }

    return MacEager::items(v);
}
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("branch_impls", branch_impls);}
