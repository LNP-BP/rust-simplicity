// Simplicity Policy Compiler
// Written in 2020 by
//     Sanket Kanjalkar <sanket1729@gmail.com>
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Policy Compiler
//! Compile a policy to Simplicity Program
//! Currently the policy compilation is one to one mapping
//! between policy fragment and a simplicity program.

use super::ast::Policy;
use crate::bitcoin_hashes::Hash;
use crate::bititer::BitIter;
use crate::core::term::DagTerm;
use crate::core::types::pow2_types;
use crate::extension::bitcoin::BtcNode;
use crate::extension::jets::JetsNode::{
    Adder32, EqV256, EqV32, LessThanV32, SchnorrAssert, Sha256,
};
use crate::miniscript::MiniscriptKey;
use crate::Error;
use crate::PubkeyKey32;
use crate::Value;

use std::rc::Rc;

/// Scribe progra: for any value of a Simplicity type b :B, the constant function
/// from A -> B can be realized by a Simplicity expression called scribe.  
/// Refer to 3.4 section of the Tech Report for details.
/// This returns a list of untyped nodes.
pub fn scribe<Ext>(b: Value) -> DagTerm<(), Ext> {
    match b {
        Value::Unit => DagTerm::Unit,
        Value::SumL(l) => {
            let l = scribe(*l);
            DagTerm::InjL(Rc::new(l))
        }
        Value::SumR(r) => {
            let r = scribe(*r);
            DagTerm::InjL(Rc::new(r))
        }
        Value::Prod(l, r) => {
            let l = scribe(*l);
            let r = scribe(*r);
            DagTerm::Pair(Rc::new(l), Rc::new(r))
        }
    }
}

/// constant function that returns false
pub fn zero<Ext>() -> DagTerm<(), Ext> {
    scribe(Value::sum_l(Value::Unit))
}

/// constant function that returns true
pub fn one<Ext>() -> DagTerm<(), Ext> {
    scribe(Value::sum_r(Value::Unit))
}

/// Cond program: The combinator to branch based on the value of a
/// bit using case and drop. The first argument is the
/// then clause and the second argument is the else clause
/// [[cond st]] <0, a> = [[s]](a); [[cond st]] <1, a> = [[t]](a)
pub fn cond<Ext>(s: Rc<DagTerm<(), Ext>>, t: Rc<DagTerm<(), Ext>>) -> DagTerm<(), Ext> {
    DagTerm::Case(Rc::new(DagTerm::Drop(s)), Rc::new(DagTerm::Drop(t)))
}

/// Convert a single bit into u2 by pre-padding zeros
fn u1_to_u2<Ext>(s: Rc<DagTerm<(), Ext>>) -> DagTerm<(), Ext> {
    DagTerm::Pair(Rc::new(scribe(Value::u1(0))), s)
}

/// Convert a single bit into u4 by pre-padding zeros
fn u1_to_u4<Ext>(s: Rc<DagTerm<(), Ext>>) -> DagTerm<(), Ext> {
    DagTerm::Pair(Rc::new(scribe(Value::u2(0))), Rc::new(u1_to_u2(s)))
}

/// Convert a single bit into u8 by pre-padding zeros
fn u1_to_u8<Ext>(s: Rc<DagTerm<(), Ext>>) -> DagTerm<(), Ext> {
    DagTerm::Pair(Rc::new(scribe(Value::u4(0))), Rc::new(u1_to_u4(s)))
}

/// Convert a single bit into u16 by pre-padding zeros
fn u1_to_u16<Ext>(s: Rc<DagTerm<(), Ext>>) -> DagTerm<(), Ext> {
    DagTerm::Pair(Rc::new(scribe(Value::u8(0))), Rc::new(u1_to_u8(s)))
}

/// Convert a single bit into u32 by pre-padding zeros
fn u1_to_u32<Ext>(s: Rc<DagTerm<(), Ext>>) -> DagTerm<(), Ext> {
    DagTerm::Pair(Rc::new(scribe(Value::u16(0))), Rc::new(u1_to_u16(s)))
}

/// Compile the desired policy into a bitcoin simplicity program
pub fn compile<Pk: MiniscriptKey + PubkeyKey32>(
    pol: &Policy<Pk>,
) -> Result<DagTerm<(), BtcNode>, Error> {
    let two_pow_256 = pow2_types()[9].clone();
    let frag = match pol {
        Policy::Unsatisfiable => unimplemented!(), //lookup  fail
        Policy::Trivial => DagTerm::Unit,
        Policy::Key(ref pk) => {
            let pk_value = Value::from_bits_and_type(
                &mut BitIter::from(pk.to_32_byte_pubkey().to_vec().into_iter()),
                &two_pow_256,
            )?;
            let scribe_pk = scribe(pk_value);
            let pk_sig_pair = DagTerm::Pair(Rc::new(scribe_pk), Rc::new(DagTerm::Witness(())));
            DagTerm::Comp(Rc::new(pk_sig_pair), Rc::new(DagTerm::Jet(SchnorrAssert)))
        }
        Policy::Sha256(ref h) => {
            let hash_value = Value::from_bits_and_type(
                &mut BitIter::from(h.into_inner().to_vec().into_iter()),
                &two_pow_256,
            )?;
            // scribe target hash
            let scribe_hash = scribe(hash_value);
            // compute the preimage hash. An implicit contraint on the len=32 is enfored
            // by the typesystem.
            let computed_hash =
                DagTerm::Comp(Rc::new(DagTerm::Witness(())), Rc::new(DagTerm::Jet(Sha256)));
            // Check eq256 here
            let pair = DagTerm::Pair(Rc::new(scribe_hash), Rc::new(computed_hash));
            DagTerm::Comp(Rc::new(pair), Rc::new(DagTerm::Jet(EqV256)))
        }
        Policy::After(n) => {
            let cltv = DagTerm::Ext(BtcNode::LockTime);
            let n_value = Value::u32(*n);
            let scribe_n = scribe(n_value);
            let pair = DagTerm::Pair(Rc::new(scribe_n), Rc::new(cltv));
            DagTerm::Comp(Rc::new(pair), Rc::new(DagTerm::Jet(LessThanV32)))
        }
        Policy::Older(n) => {
            let csv = DagTerm::Ext(BtcNode::CurrentSequence);
            let n_value = Value::u32(*n);
            let scribe_n = scribe(n_value);
            let pair = DagTerm::Pair(Rc::new(scribe_n), Rc::new(csv));
            DagTerm::Comp(Rc::new(pair), Rc::new(DagTerm::Jet(LessThanV32)))
        }
        Policy::Threshold(k, ref subs) => {
            assert!(subs.len() >= 2, "Threshold must have numbre of subs >=2");
            let child = compile(&subs[0])?;
            // selector denotes a bit that specifies whether the first child should be executed.
            let selector = Rc::new(DagTerm::Witness(()));
            // The case condition that for the current child
            let case_term = cond(Rc::new(child), Rc::new(DagTerm::Unit));
            let mut acc = DagTerm::Comp(Rc::clone(&selector), Rc::new(case_term));
            let mut sum = u1_to_u32(selector);
            for sub in &subs[1..] {
                let child = compile(sub)?;
                let selector = Rc::new(DagTerm::Witness(()));
                let case_term = cond(Rc::new(child), Rc::new(DagTerm::Unit));

                let curr_term = DagTerm::Comp(Rc::clone(&selector), Rc::new(case_term));
                let selector_u32 = u1_to_u32(selector);

                acc = DagTerm::Comp(Rc::new(acc), Rc::new(curr_term));
                let full_sum = DagTerm::Comp(
                    Rc::new(DagTerm::Pair(Rc::new(sum), Rc::new(selector_u32))),
                    Rc::new(DagTerm::Jet(Adder32)),
                );
                // Discard the overflow bit.
                // NOTE: This *assumes* that the threshold would be have 2**32 branches.
                // FIXME: enforce this in policy specification.
                sum = DagTerm::Drop(Rc::new(full_sum));
            }
            let scribe_k = scribe(Value::u32(*k as u32));
            DagTerm::Comp(
                Rc::new(DagTerm::Pair(Rc::new(scribe_k), Rc::new(sum))),
                Rc::new(DagTerm::Jet(EqV32)),
            )
        }
        Policy::And(ref subs) => {
            assert!(subs.len() == 2);
            let l = compile(&subs[0])?;
            let r = compile(&subs[1])?;
            DagTerm::Comp(Rc::new(l), Rc::new(r))
        }
        Policy::Or(ref subs) => {
            assert!(subs.len() == 2);
            let l = compile(&subs[0])?;
            let r = compile(&subs[1])?;
            let case_term = cond(Rc::new(l), Rc::new(r));
            DagTerm::Comp(Rc::new(DagTerm::Witness(())), Rc::new(case_term))
        }
    };
    Ok(frag)
}
