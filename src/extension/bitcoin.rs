// Rust Simplicity Library
// Written in 2020 by
//   Andrew Poelstra <apoelstra@blockstream.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Bitcoin Extensions
//!
//! Extensions to the Simplicity language to allow use on the Bitcoin
//! blockchain
//!

use std::{fmt, io};

use bititer::BitIter;
use super::TypeName;
use Error;
use cmr::Cmr;
use encode;

/// Set of new Simplicity nodes enabled by the Bitcoin extension
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Node {
    Version,
    LockTime,
    InputsHash,
    OutputsHash,
    NumInputs,
    TotalInputValue,
    CurrentPrevOutpoint,
    CurrentValue,
    CurrentSequence,
    CurrentIndex,
    InputPrevOutpoint,
    InputValue,
    InputSequence,
    NumOutputs,
    TotalOutputValue,
    OutputValue,
    OutputScriptHash,
    ScriptCMR,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Node::Version => "version",
            Node::LockTime => "locktime",
            Node::InputsHash => "inputshash",
            Node::OutputsHash => "outputshash",
            Node::NumInputs => "numinputs",
            Node::TotalInputValue =>  "totalinputvalue",
            Node::CurrentPrevOutpoint => "currentprevoutpoint",
            Node::CurrentValue => "currentvalue",
            Node::CurrentSequence => "currentsequence",
            Node::CurrentIndex => "currentindex",
            Node::InputPrevOutpoint => "inputprevoutpoint",
            Node::InputValue => "inputvalue",
            Node::InputSequence => "inputsequence",
            Node::NumOutputs => "numoutputs",
            Node::TotalOutputValue => "totaloutputvalue",
            Node::OutputValue => "outputvalue",
            Node::OutputScriptHash => "outputscripthash",
            Node::ScriptCMR => "scriptcmr",
        })
    }
}

/// Decode a natural number according to section 7.2.1
/// of the Simplicity whitepaper. Assumes that a 10 prefix
/// has already been read
pub fn decode_node<I: Iterator<Item = u8>>(
    iter: &mut BitIter<I>,
) -> Result<Node, Error> {
    let code = match iter.read_bits_be(4) {
        Some(code) => code,
        None => return Err(Error::EndOfStream),
    };
    match code {
        0 => match iter.next() {
            Some(false) => Ok(Node::Version),
            Some(true) => Ok(Node::LockTime),
            None => Err(Error::EndOfStream),
        },
        1 => Ok(Node::InputsHash),
        2 => Ok(Node::OutputsHash),
        3 => Ok(Node::NumInputs),
        4 => Ok(Node::TotalInputValue),
        5 => Ok(Node::CurrentPrevOutpoint),
        6 => Ok(Node::CurrentValue),
        7 => Ok(Node::CurrentSequence),
        8 => match iter.next() {
            Some(false) => Ok(Node::CurrentIndex),
            Some(true) => Ok(Node::InputPrevOutpoint),
            None => Err(Error::EndOfStream),
        },
        9 => Ok(Node::InputValue),
        10 => Ok(Node::InputSequence),
        11 => Ok(Node::NumOutputs),
        12 => Ok(Node::TotalOutputValue),
        13 => Ok(Node::OutputValue),
        14 => Ok(Node::OutputScriptHash),
        15 => Ok(Node::ScriptCMR),
        _ => unreachable!(),
    }
}

impl Node {
    /// Name of the source type for this node
    pub fn source_type(&self) -> TypeName {
        match *self {
            Node::Version
                | Node::LockTime
                | Node::InputsHash
                | Node::OutputsHash
                | Node::NumInputs
                | Node::TotalInputValue
                | Node::CurrentPrevOutpoint
                | Node::CurrentValue
                | Node::CurrentSequence
                | Node::CurrentIndex => TypeName::One,
            Node::InputPrevOutpoint
                | Node::InputValue
                | Node::InputSequence => TypeName::Word32,
            Node::NumOutputs
                | Node::TotalOutputValue => TypeName::One,
            Node::OutputValue
                | Node::OutputScriptHash => TypeName::Word32,
            Node::ScriptCMR => TypeName::One,
        }
    }

    /// Name of the target type for this node
    pub fn target_type(&self) -> TypeName {
        match *self {
            Node::Version => TypeName::Word32,
            Node::LockTime => TypeName::Word32,
            Node::InputsHash => TypeName::Word256,
            Node::OutputsHash => TypeName::Word256,
            Node::NumInputs => TypeName::Word32,
            Node::TotalInputValue => TypeName::Word64,
            Node::CurrentPrevOutpoint => TypeName::Word256Word32,
            Node::CurrentValue => TypeName::Word64,
            Node::CurrentSequence => TypeName::Word32,
            Node::CurrentIndex => TypeName::Word32,
            Node::InputPrevOutpoint => TypeName::SWord256Word32,
            Node::InputValue => TypeName::SWord64,
            Node::InputSequence => TypeName::SWord32,
            Node::NumOutputs => TypeName::Word32,
            Node::TotalOutputValue => TypeName::Word64,
            Node::OutputValue => TypeName::SWord64,
            Node::OutputScriptHash => TypeName::SWord256,
            Node::ScriptCMR => TypeName::Word256,
        }
    }

    /// CMR for this node
    pub fn cmr(&self) -> Cmr {
        match *self {
            Node::Version => Cmr::new(b"SimplicityPrimitiveBitcoin\x1fversion"),
            Node::LockTime => Cmr::new(b"SimplicityPrimitiveBitcoin\x1flockTime"),
            Node::InputsHash => Cmr::new(b"SimplicityPrimitiveBitcoin\x1finputsHash"),
            Node::OutputsHash => Cmr::new(b"SimplicityPrimitiveBitcoinx1foutputsHash"),
            Node::NumInputs => Cmr::new(b"SimplicityPrimitiveBitcoinx1fnumInputs"),
            Node::TotalInputValue => Cmr::new(b"SimplicityPrimitiveBitcoinx1ftotalInputValue"),
            Node::CurrentPrevOutpoint => Cmr::new(b"SimplicityPrimitiveBitcoinx1fcurrentPrevOutpoint"),
            Node::CurrentValue => Cmr::new(b"SimplicityPrimitiveBitcoinx1fcurrentValue"),
            Node::CurrentSequence => Cmr::new(b"SimplicityPrimitiveBitcoinx1fcurrentSequence"),
            Node::CurrentIndex => Cmr::new(b"SimplicityPrimitiveBitcoinx1fcurrentIndex"),
            Node::InputPrevOutpoint => Cmr::new(b"SimplicityPrimitiveBitcoinx1finputPrevOutpoint"),
            Node::InputValue => Cmr::new(b"SimplicityPrimitiveBitcoinx1finputValue"),
            Node::InputSequence => Cmr::new(b"SimplicityPrimitiveBitcoinx1finputSequence"),
            Node::NumOutputs => Cmr::new(b"SimplicityPrimitiveBitcoinx1fnumOutputs"),
            Node::TotalOutputValue => Cmr::new(b"SimplicityPrimitiveBitcoinx1ftotalOutputValue"),
            Node::OutputValue => Cmr::new(b"SimplicityPrimitiveBitcoinx1foutputValue"),
            Node::OutputScriptHash => Cmr::new(b"SimplicityPrimitiveBitcoinx1foutputScriptHash"),
            Node::ScriptCMR => Cmr::new(b"SimplicityPrimitiveBitcoinx1fscriptCMR"),
        }
    }

    /// Encode the node into a bitstream
    pub fn encode_node<W: encode::BitWrite>(&self, w: &mut W) -> io::Result<usize> {
        match *self {
            Node::Version => w.write_u8(64 + 0, 7),
            Node::LockTime => w.write_u8(64 + 1, 7),
            Node::InputsHash => w.write_u8(32 + 1, 6),
            Node::OutputsHash => w.write_u8(32 + 2, 6),
            Node::NumInputs => w.write_u8(32 + 3, 6),
            Node::TotalInputValue => w.write_u8(32 + 4, 6),
            Node::CurrentPrevOutpoint => w.write_u8(32 + 5, 6),
            Node::CurrentValue => w.write_u8(32 + 6, 6),
            Node::CurrentSequence => w.write_u8(32 + 7, 6),
            Node::CurrentIndex => w.write_u8(64 + 16, 7),
            Node::InputPrevOutpoint => w.write_u8(64 + 17, 7),
            Node::InputValue => w.write_u8(32 + 9, 6),
            Node::InputSequence => w.write_u8(32 + 10, 6),
            Node::NumOutputs => w.write_u8(32 + 11, 6),
            Node::TotalOutputValue => w.write_u8(32 + 12, 6),
            Node::OutputValue => w.write_u8(32 + 13, 6),
            Node::OutputScriptHash => w.write_u8(32 + 14, 6),
            Node::ScriptCMR => w.write_u8(32 + 15, 6),
        }
    }
}

