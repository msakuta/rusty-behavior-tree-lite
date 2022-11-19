//! Our custom implementation of symbol.
//!
//! Copied and modified from https://github.com/remexre/symbol-rs
//! to conform to modern Rust and drop support for no-std

use ::once_cell::sync::{Lazy, OnceCell};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::mem::{forget, transmute};
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Mutex;

static SYMBOL_HEAP: Lazy<Mutex<BTreeSet<&'static str>>> = Lazy::new(|| Mutex::new(BTreeSet::new()));

/// An interned string with O(1) equality.
#[allow(clippy::derive_hash_xor_eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Clone, Copy, Eq, Hash)]
pub struct Symbol {
    s: &'static str,
}

impl Symbol {
    /// Retrieves the address of the backing string.
    pub fn addr(self) -> usize {
        self.s.as_ptr() as usize
    }

    /// Retrieves the string from the Symbol.
    pub fn as_str(self) -> &'static str {
        self.s
    }

    /// Generates a new symbol with a name of the form `G#n`, where `n` is some positive integer.
    pub fn gensym() -> Symbol {
        static COUNTER: OnceCell<AtomicUsize> = OnceCell::with_value(AtomicUsize::new(0));
        let counter = COUNTER.get().unwrap();

        let n = if let Ok(mut heap) = SYMBOL_HEAP.lock() {
            loop {
                let n = leak_string(format!(
                    "G#{}",
                    counter.fetch_add(1, AtomicOrdering::SeqCst)
                ));
                if heap.insert(n) {
                    break n;
                }
            }
        } else {
            panic!("SYMBOL_HEAP is not initialized");
        };

        Symbol::from(n)
    }
}

impl Debug for Symbol {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        Debug::fmt(self.s, fmt)
    }
}

impl Deref for Symbol {
    type Target = str;
    fn deref(&self) -> &str {
        self.s
    }
}

impl Display for Symbol {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str(self.s)
    }
}

impl<S: AsRef<str>> From<S> for Symbol {
    fn from(s: S) -> Symbol {
        let s = s.as_ref();
        {
            let mut heap = SYMBOL_HEAP.lock().unwrap();
            if heap.get(s).is_none() {
                heap.insert(leak_string(s.to_owned()));
            }
        }
        let s = {
            let heap = SYMBOL_HEAP.lock().unwrap();
            *heap.get(s).unwrap()
        };
        Symbol { s }
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        let l = self.addr();
        let r = other.addr();
        l.cmp(&r)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: AsRef<str>> PartialEq<S> for Symbol {
    fn eq(&self, other: &S) -> bool {
        self.partial_cmp(&other.as_ref()) == Some(Ordering::Equal)
    }
}

impl<S: AsRef<str>> PartialOrd<S> for Symbol {
    fn partial_cmp(&self, other: &S) -> Option<Ordering> {
        self.s.partial_cmp(other.as_ref())
    }
}

#[cfg(feature = "gc")]
impl ::gc::Finalize for Symbol {
    fn finalize(&self) {}
}

#[cfg(feature = "gc")]
unsafe impl ::gc::Trace for Symbol {
    unsafe_empty_trace!();
}

#[cfg(feature = "radix_trie")]
impl radix_trie::TrieKey for Symbol {
    fn encode_bytes(&self) -> Vec<u8> {
        self.as_str().encode_bytes()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Symbol {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Symbol, D::Error> {
        String::deserialize(de).map(Symbol::from)
    }
}

fn leak_string(s: String) -> &'static str {
    let out = unsafe { transmute(&s as &str) };
    forget(s);
    out
}
