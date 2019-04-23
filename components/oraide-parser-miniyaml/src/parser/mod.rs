// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # `parser`
//!
//! This module consists of 3 sub-modules:
//!
//! - `tokenizer`
//!     - input: text
//!     - output: collection of `Token`s
//! - `nodeizer`
//!     - input: collection of `Token`s
//!     - output: collection of `Node`s
//! - `treeizer`
//!     - input: collection of `Node`s
//!     - output: a `Tree`
//!
//! It also contains types used by the previously-mentioned sub-modules
//! and other components of this project such as `Token`, `Node`, `Tree`, etc.
//!

mod tokenizer;
mod nodeizer;
mod treeizer;

pub use tokenizer::{
    Token,
    TokenKind,
    Tokenizer,
    TokenCollectionExts,
};

pub use nodeizer::{
    Node,
    Nodeizer,
};

pub use treeizer::{
    Tree,
    Treeizer,
    IndentLevelDelta,
    Arena,
    ArenaNodeId,
};