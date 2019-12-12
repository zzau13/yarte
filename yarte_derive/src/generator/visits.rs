#![allow(clippy::cognitive_complexity)]

use std::{fmt::Write, str};

use quote::quote;
use syn::{
    punctuated::Punctuated,
    visit::{self, Visit},
    PathSegment,
};

use super::{identifier::is_tuple_index, EWrite, Generator, On};

macro_rules! visit_attrs {
    ($_self:ident, $attrs:ident) => {
        for it in $attrs {
            $_self.visit_attribute(it)
        }
    };
}

macro_rules! visit_punctuated {
    ($_self:ident, $ele:expr, $method:ident) => {
        for el in Punctuated::pairs($ele) {
            let it = el.value();
            let punc = el.punct();
            $_self.$method(it);
            $_self.buf_t.write(&quote!(#punc));
        }
    };
}

impl<'a> Visit<'a> for Generator<'a> {
    fn visit_arm(
        &mut self,
        syn::Arm {
            attrs,
            pat,
            guard,
            fat_arrow_token,
            body,
            comma,
        }: &'a syn::Arm,
    ) {
        visit_attrs!(self, attrs);

        self.scp.push_scope(vec![]);
        self.visit_pat(pat);
        if let Some((_, expr)) = guard {
            self.buf_t.write(&"if ");
            self.visit_expr(expr);
        }
        self.buf_t.write(&quote!(#fat_arrow_token));
        self.visit_expr(body);
        self.buf_t.writeln(&quote!(#comma));
        self.scp.pop();
    }

    fn visit_attribute(&mut self, _i: &'a syn::Attribute) {
        panic!("Not available attributes in a template expression");
    }

    fn visit_bin_op(&mut self, i: &'a syn::BinOp) {
        write!(self.buf_t, " {} ", quote!(#i)).unwrap();
    }

    fn visit_block(&mut self, i: &'a syn::Block) {
        self.scp.push_scope(vec![]);
        self.buf_t.write(&" { ");
        visit::visit_block(self, i);
        self.buf_t.write(&" }");
        self.scp.pop();
    }

    fn visit_expr_array(&mut self, syn::ExprArray { attrs, elems, .. }: &'a syn::ExprArray) {
        visit_attrs!(self, attrs);
        self.buf_t.push('[');
        visit_punctuated!(self, elems, visit_expr);
        self.buf_t.push(']');
    }

    fn visit_expr_assign(
        &mut self,
        syn::ExprAssign {
            attrs,
            left,
            eq_token,
            right,
        }: &'a syn::ExprAssign,
    ) {
        visit_attrs!(self, attrs);
        let ident: &str = &quote!(#left).to_string();
        if let Some(ident) = self.scp.get_by(ident) {
            self.buf_t.write(&ident);
            self.buf_t.write(&quote!( #eq_token ));
            self.visit_expr(right);
        } else {
            panic!("Not exist `{}` in current scope", ident);
        };
    }

    fn visit_expr_assign_op(
        &mut self,
        syn::ExprAssignOp {
            attrs,
            left,
            op,
            right,
        }: &'a syn::ExprAssignOp,
    ) {
        visit_attrs!(self, attrs);
        let ident: &str = &quote!(#left).to_string();
        if let Some(ident) = self.scp.get_by(ident) {
            self.buf_t.write(&ident);
            self.buf_t.write(&quote!( #op ));
            self.visit_expr(right);
        } else {
            panic!("Not exist `{}` in current scope", ident);
        };
    }

    fn visit_expr_async(&mut self, _i: &'a syn::ExprAsync) {
        panic!("Not available async in a template expression");
    }

    fn visit_expr_block(&mut self, i: &'a syn::ExprBlock) {
        visit::visit_expr_block(self, i);
    }

    fn visit_expr_box(
        &mut self,
        syn::ExprBox {
            attrs,
            box_token,
            expr,
        }: &'a syn::ExprBox,
    ) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&quote!(#box_token));
        self.visit_expr(expr);
    }

    fn visit_expr_break(
        &mut self,
        syn::ExprBreak {
            attrs,
            break_token,
            label,
            expr,
        }: &'a syn::ExprBreak,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, "{} ", quote!(#break_token #label)).unwrap();
        if let Some(expr) = expr {
            self.visit_expr(expr)
        }
    }

    fn visit_expr_call(
        &mut self,
        syn::ExprCall {
            attrs, func, args, ..
        }: &'a syn::ExprCall,
    ) {
        visit_attrs!(self, attrs);
        let ident: &str = &quote!(#func).to_string();
        if let Some(ident) = self.scp.get_by(ident) {
            write!(self.buf_t, "{}(", ident).unwrap();
        } else {
            write!(self.buf_t, "{}(", ident).unwrap();
        }
        visit_punctuated!(self, args, visit_expr);
        self.buf_t.push(')');
    }

    fn visit_expr_cast(
        &mut self,
        syn::ExprCast {
            attrs,
            expr,
            as_token,
            ty,
        }: &'a syn::ExprCast,
    ) {
        visit_attrs!(self, attrs);
        self.visit_expr(expr);
        write!(self.buf_t, " {} ", quote!(#as_token #ty)).unwrap();
    }

    fn visit_expr_closure(
        &mut self,
        syn::ExprClosure {
            attrs,
            asyncness,
            movability,
            capture,
            inputs,
            output,
            body,
            ..
        }: &'a syn::ExprClosure,
    ) {
        visit_attrs!(self, attrs);
        if let Some(..) = asyncness {
            panic!("Not available async in template expression");
        };

        write!(self.buf_t, "{} |", quote!(#asyncness #movability #capture)).unwrap();
        self.scp.push_scope(vec![]);
        visit_punctuated!(self, inputs, visit_pat);
        self.buf_t.write(&"| ");
        self.buf_t.write(&quote!(#output));
        self.visit_expr(body);
        self.scp.pop();
    }

    fn visit_expr_continue(&mut self, i: &'a syn::ExprContinue) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_expr_field(
        &mut self,
        syn::ExprField {
            attrs,
            base,
            member,
            ..
        }: &'a syn::ExprField,
    ) {
        visit_attrs!(self, attrs);

        self.visit_expr(base);
        write!(self.buf_t, ".{}", quote!(#member)).unwrap();
    }

    fn visit_expr_for_loop(
        &mut self,
        syn::ExprForLoop {
            attrs,
            label,
            for_token,
            pat,
            expr,
            body,
            ..
        }: &'a syn::ExprForLoop,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} ", &quote!(#label #for_token)).unwrap();
        self.scp.push_scope(vec![]);
        self.visit_pat(pat);
        let last = self.scp.pops();
        self.buf_t.write(&" in ");
        self.visit_expr(expr);
        self.scp.push_scope(last);
        self.visit_block(body);
        self.scp.pop();
    }

    fn visit_expr_if(
        &mut self,
        syn::ExprIf {
            attrs,
            cond,
            then_branch,
            else_branch,
            ..
        }: &'a syn::ExprIf,
    ) {
        visit_attrs!(self, attrs);

        self.buf_t.write(&" if ");
        self.scp.push_scope(vec![]);

        self.visit_expr(cond);

        self.visit_block(then_branch);
        self.scp.pop();

        if let Some((_, it)) = else_branch {
            self.buf_t.write(&" else");
            self.visit_expr(it);
        };
    }

    fn visit_expr_index(
        &mut self,
        syn::ExprIndex {
            attrs, expr, index, ..
        }: &'a syn::ExprIndex,
    ) {
        visit_attrs!(self, attrs);
        self.visit_expr(expr);
        self.buf_t.write(&quote!([#index]));
    }

    fn visit_expr_let(
        &mut self,
        syn::ExprLet {
            attrs, expr, pat, ..
        }: &'a syn::ExprLet,
    ) {
        visit_attrs!(self, attrs);

        self.buf_t.write(&"let ");

        self.scp.push_scope(vec![]);
        self.visit_pat(pat);
        let last = self.scp.pops();

        self.buf_t.push(' ');
        self.buf_t.push('=');

        self.visit_expr(expr);
        self.scp.extend(last);
    }

    fn visit_expr_loop(
        &mut self,
        syn::ExprLoop {
            attrs,
            label,
            loop_token,
            body,
        }: &'a syn::ExprLoop,
    ) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&quote!(#label #loop_token));
        self.visit_block(body);
    }

    fn visit_expr_match(
        &mut self,
        syn::ExprMatch {
            attrs,
            match_token,
            expr,
            arms,
            ..
        }: &'a syn::ExprMatch,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} ", quote!(#match_token)).unwrap();
        self.visit_expr(expr);
        self.buf_t.push('{');
        for i in arms {
            self.visit_arm(i);
        }
        self.buf_t.push('}');
    }

    fn visit_expr_method_call(
        &mut self,
        syn::ExprMethodCall {
            attrs,
            receiver,
            method,
            turbofish,
            args,
            ..
        }: &'a syn::ExprMethodCall,
    ) {
        visit_attrs!(self, attrs);
        self.visit_expr(receiver);
        write!(self.buf_t, ".{}(", quote!(#method#turbofish)).unwrap();
        visit_punctuated!(self, args, visit_expr);
        self.buf_t.push(')');
    }

    fn visit_expr_paren(&mut self, syn::ExprParen { attrs, expr, .. }: &'a syn::ExprParen) {
        visit_attrs!(self, attrs);
        self.buf_t.push('(');
        self.visit_expr(expr);
        self.buf_t.push(')');
    }

    fn visit_expr_path(&mut self, syn::ExprPath { attrs, qself, path }: &'a syn::ExprPath) {
        debug_assert!(!self.scp.is_empty() && !self.scp[0].is_empty());
        visit_attrs!(self, attrs);
        if qself.is_some() {
            panic!("Not available QSelf in a template expression");
        }

        macro_rules! writes {
            ($($t:tt)+) => {{
                return write!(self.buf_t, $($t)+).unwrap();
            }};
        }

        macro_rules! index_var {
            ($ident:expr, $j:expr) => {{
                let ident = $ident.as_bytes();
                if is_tuple_index(ident) {
                    writes!(
                        "{}.{}",
                        self.scp[$j][0],
                        str::from_utf8(&ident[1..]).unwrap()
                    );
                }
            }};
        }

        macro_rules! each_var {
            ($ident:expr, $j:expr) => {{
                debug_assert!(self.scp.get($j).is_some(), "{} {:?}", $j, self.scp);
                debug_assert!(!self.scp[$j].is_empty());
                match $ident {
                    "index0" => writes!("{}", self.scp[$j][1]),
                    "index" => writes!("({} + 1)", self.scp[$j][1]),
                    "first" => writes!("({} == 0)", self.scp[$j][1]),
                    "this" => return self.buf_t.write(&self.scp[$j][0]),
                    ident => {
                        index_var!(ident, $j);
                        writes!("{}.{}", self.scp[$j][0], ident);
                    }
                }
            }};
        }

        macro_rules! with_var {
            ($ident:expr, $j:expr) => {{
                debug_assert!(self.scp.get($j).is_some());
                debug_assert!(!self.scp[$j].is_empty());
                index_var!($ident, $j);
                writes!("{}.{}", self.scp[$j][0], $ident);
            }};
        }

        macro_rules! self_var {
            ($ident:ident) => {{
                index_var!($ident, 0);
                writes!("{}.{}", self.scp.root(), $ident);
            }};
        }

        macro_rules! partial_var {
            ($ident:ident, $on:expr) => {{
                if let Some((partial, level)) = &self.partial {
                    if *level == $on {
                        if let Some(expr) = partial.get($ident) {
                            return self.buf_t.write(&quote!(#expr));
                        }
                    }
                }
            }};
        }

        if path.segments.len() == 1 {
            let ident: &str = &path.segments[0].ident.to_string();

            // static or constant
            if ident.chars().all(|x| x.is_ascii_uppercase() || x.eq(&'_')) {
                return self.buf_t.write(&ident);
            }

            partial_var!(ident, self.on.len());

            if let Some(ident) = &self.scp.get_by(ident) {
                // in scope
                self.buf_t.write(ident);
            } else {
                // out scope
                if ident.eq("self") {
                    return self.buf_t.write(&self.scp.root());
                }

                match self.on.last() {
                    None => self_var!(ident),
                    Some(On::Each(j)) => each_var!(ident, *j),
                    Some(On::With(j)) => with_var!(ident, *j),
                };
            };
        } else if let Some((j, ref ident)) = is_super(&path.segments) {
            if self.on.is_empty() {
                panic!("use super at top");
            } else if self.on.len() == j {
                partial_var!(ident, j);
                self_var!(ident);
            } else if j < self.on.len() {
                partial_var!(ident, j);
                match self.on[self.on.len() - j - 1] {
                    On::Each(j) => each_var!(ident.as_str(), j),
                    On::With(j) => with_var!(ident, j),
                }
            } else {
                panic!("use super without parent")
            }
        } else {
            self.buf_t.write(&quote!(#path));
        }
    }

    fn visit_expr_reference(
        &mut self,
        syn::ExprReference {
            attrs,
            and_token,
            mutability,
            expr,
            ..
        }: &'a syn::ExprReference,
    ) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&quote!(#and_token #mutability));
        self.visit_expr(expr);
    }

    fn visit_expr_repeat(&mut self, i: &'a syn::ExprRepeat) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_expr_return(&mut self, syn::ExprReturn { attrs, expr, .. }: &'a syn::ExprReturn) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&"return ");
        if let Some(expr) = expr {
            self.visit_expr(expr);
        }
    }

    fn visit_expr_struct(
        &mut self,
        syn::ExprStruct {
            attrs,
            path,
            fields,
            dot2_token,
            rest,
            ..
        }: &'a syn::ExprStruct,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} {{", quote!(#path)).unwrap();
        visit_punctuated!(self, fields, visit_field_value);
        write!(self.buf_t, " {} }}", quote!(#dot2_token#rest)).unwrap();
    }

    fn visit_expr_try(&mut self, syn::ExprTry { attrs, expr, .. }: &'a syn::ExprTry) {
        visit_attrs!(self, attrs);
        self.visit_expr(expr);
        self.buf_t.push('?');
    }

    fn visit_expr_try_block(&mut self, _i: &'a syn::ExprTryBlock) {
        panic!("Not allowed try block expression in a template expression");
    }

    fn visit_expr_tuple(&mut self, syn::ExprTuple { attrs, elems, .. }: &'a syn::ExprTuple) {
        visit_attrs!(self, attrs);

        self.buf_t.push('(');
        visit_punctuated!(self, elems, visit_expr);
        self.buf_t.push(')');
    }

    fn visit_expr_unsafe(&mut self, syn::ExprUnsafe { attrs, block, .. }: &'a syn::ExprUnsafe) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&"unsafe ");
        self.visit_block(block);
    }

    fn visit_expr_while(
        &mut self,
        syn::ExprWhile {
            attrs,
            label,
            while_token,
            cond,
            body,
        }: &'a syn::ExprWhile,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} ", quote!(#label #while_token)).unwrap();
        self.visit_expr(cond);
        self.visit_block(body);
    }

    fn visit_expr_yield(&mut self, _i: &'a syn::ExprYield) {
        panic!("Not allowed yield expression in a template expression");
    }

    fn visit_field_value(
        &mut self,
        syn::FieldValue {
            attrs,
            member,
            colon_token,
            expr,
        }: &'a syn::FieldValue,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} ", quote!(#member #colon_token)).unwrap();
        self.visit_expr(expr)
    }

    fn visit_lit(&mut self, i: &'a syn::Lit) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_local(
        &mut self,
        syn::Local {
            attrs, pat, init, ..
        }: &'a syn::Local,
    ) {
        visit_attrs!(self, attrs);

        self.scp.push_scope(vec![]);

        self.buf_t.write(&"let ");

        self.visit_pat(pat);
        let scope = self.scp.pops();

        if let Some((_, expr)) = init {
            self.buf_t.push('=');
            self.visit_expr(expr);
        }
        self.buf_t.push(';');

        self.scp.extend(scope);
    }

    fn visit_macro(&mut self, i: &'a syn::Macro) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_pat_box(&mut self, syn::PatBox { pat, .. }: &'a syn::PatBox) {
        self.buf_t.write(&" box ");
        self.visit_pat(pat);
    }

    fn visit_pat_ident(
        &mut self,
        syn::PatIdent {
            attrs,
            by_ref,
            mutability,
            ident,
            subpat,
        }: &'a syn::PatIdent,
    ) {
        visit_attrs!(self, attrs);

        if subpat.is_some() {
            panic!("Subpat is not allowed");
        }

        let name = self.scp.push(&ident.to_string());
        let ident = syn::Ident::new(&name, ident.span());

        self.buf_t.write(&quote!(#by_ref #mutability #ident));
    }

    fn visit_pat_lit(&mut self, _i: &'a syn::PatLit) {
        panic!("Not allowed pat lit");
    }

    fn visit_pat_macro(&mut self, _i: &'a syn::PatMacro) {
        panic!("Not allowed pat macro");
    }

    fn visit_pat_path(&mut self, _i: &'a syn::PatPath) {
        panic!("Not allowed pat path");
    }

    fn visit_pat_range(&mut self, _i: &'a syn::PatRange) {
        panic!("Not allowed pat range");
    }

    fn visit_pat_rest(&mut self, syn::PatRest { attrs, dot2_token }: &'a syn::PatRest) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&quote!(#dot2_token));
    }

    fn visit_pat_slice(&mut self, syn::PatSlice { attrs, elems, .. }: &'a syn::PatSlice) {
        visit_attrs!(self, attrs);
        self.buf_t.push('[');
        visit_punctuated!(self, elems, visit_pat);
        self.buf_t.push(']');
    }

    fn visit_pat_struct(&mut self, _i: &'a syn::PatStruct) {
        panic!("Not available let struct decompose, use `with` helper instead");
    }

    fn visit_pat_tuple(&mut self, syn::PatTuple { attrs, elems, .. }: &'a syn::PatTuple) {
        visit_attrs!(self, attrs);
        self.buf_t.push('(');
        visit_punctuated!(self, elems, visit_pat);
        self.buf_t.push(')');
    }

    fn visit_pat_tuple_struct(
        &mut self,
        syn::PatTupleStruct { path, pat, .. }: &'a syn::PatTupleStruct,
    ) {
        self.buf_t.write(&quote!(#path));
        self.visit_pat_tuple(pat)
    }

    fn visit_pat_type(
        &mut self,
        syn::PatType {
            attrs,
            pat,
            colon_token,
            ty,
        }: &'a syn::PatType,
    ) {
        visit_attrs!(self, attrs);
        self.visit_pat(pat);
        self.buf_t.write(&quote!(#colon_token #ty));
    }

    fn visit_pat_wild(&mut self, i: &'a syn::PatWild) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_range_limits(&mut self, i: &'a syn::RangeLimits) {
        use syn::RangeLimits::*;
        match i {
            HalfOpen(i) => {
                self.buf_t.write(&quote!(#i));
            }
            Closed(i) => {
                self.buf_t.write(&quote!(#i));
            }
        }
    }

    fn visit_stmt(&mut self, i: &'a syn::Stmt) {
        use syn::Stmt::*;
        match i {
            Local(i) => {
                self.visit_local(i);
            }
            Item(i) => {
                self.visit_item(i);
            }
            Expr(i) => {
                self.visit_expr(i);
            }
            Semi(i, semi) => {
                self.visit_expr(i);
                self.buf_t.write(&quote!(#semi));
            }
        }
    }

    fn visit_un_op(&mut self, i: &'a syn::UnOp) {
        self.buf_t.write(&quote!(#i));
    }
}

pub(super) fn is_super<S>(i: &Punctuated<PathSegment, S>) -> Option<(usize, String)> {
    let idents: Vec<String> = Punctuated::pairs(i)
        .map(|x| x.value().ident.to_string())
        .collect();
    let len = idents.len();
    let ident = idents[len - 1].clone();
    let idents: &[String] = &idents[0..len - 1];

    if idents.iter().all(|x| x.eq("super")) {
        Some((idents.len(), ident))
    } else {
        None
    }
}
