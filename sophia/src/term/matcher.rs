//! This crate defines generic traits and default implementations for *matchers*,
//! objects that can be used to match zero, one or several terms.
//!
//! # Usage
//!
//! This is how triple matchers can be used to retrieve any subject of type
//! `s:City` or `s:Country`.
//!
//! ```
//! # use sophia::graph::{*, inmem::LightGraph};
//! # use sophia::triple::Triple;
//! use sophia::ns::{Namespace, rdf};
//! use sophia::term::matcher::ANY;
//!
//! # let mut graph = LightGraph::new();
//! let s = Namespace::new("http://schema.org/").unwrap();
//! let city = s.get("City").unwrap();
//! let country = s.get("Country").unwrap();
//!
//! for t in graph.triples_matching(&ANY, &rdf::type_, &[city, country]) {
//!     let t = t.unwrap();
//!     println!("{} was found", t.s());
//! }
//! ```
//!
//! For more kinds of matchers,
//! check [`TermMarcher`'s ](trait.TermMatcher.html#implementors) and
//! [`GraphNameMatcher`'s implementors lists](trait.GraphNameMatcher.html#implementors).
//!
//! For methods using matchers, see for example
//! [`Graph::triples_matching`](../../graph/trait.Graph.html#method.triples_matching),
//! [`MutableGraph::remove_matching`](../../graph/trait.MutableGraph.html#method.remove_matching),
//! [`MutableGraph::retain_matching`](../../graph/trait.MutableGraph.html#method.retain_matching),
//! [`Dataset::quads_matching`](../../dataset/trait.Dataset.html#method.quads_matching),
//! [`MutableDataset::remove_matching`](../../dataset/trait.MutableDataset.html#method.remove_matching),
//! [`MutableDataset::retain_matching`](../../dataset/trait.MutableDataset.html#method.retain_matching).
//!

use super::*;

pub use super::_graph_name_matcher::*;

/// Generic trait for matching [term]s.
///
/// [term]: ../enum.Term.html
///
pub trait TermMatcher {
    type TermData: TermData;
    /// If this matcher matches only one term, return this term, else `None`.
    fn constant(&self) -> Option<&Term<Self::TermData>>;

    /// Check whether this matcher matches `t`.
    fn matches<T>(&self, t: &Term<T>) -> bool
    where
        T: TermData;
}

/// A universal matcher: it matches any term or graph name (even the default graph).
pub const ANY: AnyTerm = AnyTerm {};

/// The type of the [`ANY`](constant.ANY.html) singleton matcher.
pub struct AnyTerm {}

impl TermMatcher for AnyTerm {
    type TermData = &'static str;
    fn constant(&self) -> Option<&Term<Self::TermData>> {
        None
    }
    fn matches<T>(&self, _t: &Term<T>) -> bool
    where
        T: TermData,
    {
        true
    }
}

/// A matcher matching either any term, or only a specific one.
pub enum AnyOrExactly<T> {
    Any,
    Exactly(T),
}

impl<T> From<Option<T>> for AnyOrExactly<T> {
    fn from(other: Option<T>) -> AnyOrExactly<T> {
        match other {
            None => AnyOrExactly::Any,
            Some(t) => AnyOrExactly::Exactly(t),
        }
    }
}

impl<U> TermMatcher for AnyOrExactly<Term<U>>
where
    U: TermData,
{
    type TermData = U;
    fn constant(&self) -> Option<&Term<Self::TermData>> {
        match self {
            AnyOrExactly::Any => None,
            AnyOrExactly::Exactly(t) => Some(t),
        }
    }
    fn matches<T>(&self, t: &Term<T>) -> bool
    where
        T: TermData,
    {
        match self {
            AnyOrExactly::Any => true,
            AnyOrExactly::Exactly(tself) => tself == t,
        }
    }
}

impl<U> TermMatcher for Term<U>
where
    U: TermData,
{
    type TermData = U;
    fn constant(&self) -> Option<&Term<Self::TermData>> {
        Some(self)
    }
    fn matches<T>(&self, t: &Term<T>) -> bool
    where
        T: TermData,
    {
        t == self
    }
}

impl<U> TermMatcher for [Term<U>]
where
    U: TermData,
{
    type TermData = U;
    fn constant(&self) -> Option<&Term<Self::TermData>> {
        if self.len() == 1 {
            Some(&self[0])
        } else {
            None
        }
    }
    fn matches<T>(&self, t: &Term<T>) -> bool
    where
        T: TermData,
    {
        for term in self {
            if t == term {
                return true;
            }
        }
        false
    }
}

// Also impl'ing on arrays for improving DX
// (&[T;N] is not automatically cast to &[T] in generic functions...).
macro_rules! impl_for_array {
    ($n: expr) => {
        impl<U> TermMatcher for [Term<U>; $n]
        where
            U: TermData,
        {
            type TermData = U;
            fn constant(&self) -> Option<&Term<Self::TermData>> {
                None
            }
            fn matches<T>(&self, t: &Term<T>) -> bool
            where
                T: TermData,
            {
                for term in self {
                    if t == term {
                        return true;
                    }
                }
                false
            }
        }
    };
}
impl_for_array!(2);
impl_for_array!(3);
impl_for_array!(4);
impl_for_array!(5);
impl_for_array!(6);
impl_for_array!(7);
impl_for_array!(8);
impl_for_array!(9);
impl_for_array!(10);
impl_for_array!(11);
impl_for_array!(12);

impl<F: Fn(&RefTerm) -> bool> TermMatcher for F {
    type TermData = &'static str;
    fn constant(&self) -> Option<&Term<Self::TermData>> {
        None
    }
    fn matches<T>(&self, t: &Term<T>) -> bool
    where
        T: TermData,
    {
        self(&RefTerm::from(t))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_term_as_matcher() {
        let m = BoxTerm::new_iri("http://champin.net/#pa").unwrap();
        // comparing to a term using a different term data, and differently cut,
        // to make the test less obvious
        let t1 = RcTerm::new_iri_suffixed("http://champin.net/#", "pa").unwrap();
        let t2 = RcTerm::new_iri("http://example.org/").unwrap();

        let mc = TermMatcher::constant(&m);
        assert!(mc.is_some());
        assert_eq!(mc.unwrap(), &t1);
        assert!(TermMatcher::matches(&m, &t1));
        assert!(!TermMatcher::matches(&m, &t2));
    }

    #[test]
    fn test_any_as_matcher() {
        let m = ANY;
        // comparing to a term using a different term data, and differently cut,
        // to make the test less obvious
        let t1 = RcTerm::new_iri_suffixed("http://champin.net/#", "pa").unwrap();

        let mc = TermMatcher::constant(&m);
        assert!(mc.is_none());
        assert!(TermMatcher::matches(&m, &t1));
    }

    #[test]
    fn test_aoe_any_as_matcher() {
        let m = AnyOrExactly::<BoxTerm>::Any;
        // comparing to a term using a different term data, and differently cut,
        // to make the test less obvious
        let t1 = RcTerm::new_iri_suffixed("http://champin.net/#", "pa").unwrap();

        let mc = TermMatcher::constant(&m);
        assert!(mc.is_none());
        assert!(TermMatcher::matches(&m, &t1));
    }

    #[test]
    fn test_aoe_exactly_as_matcher() {
        let m = AnyOrExactly::Exactly(BoxTerm::new_iri("http://champin.net/#pa").unwrap());
        // comparing to a term using a different term data, and differently cut,
        // to make the test less obvious
        let t1 = RcTerm::new_iri_suffixed("http://champin.net/#", "pa").unwrap();
        let t2 = RcTerm::new_iri("http://example.org/").unwrap();

        let mc = TermMatcher::constant(&m);
        assert!(mc.is_some());
        assert_eq!(mc.unwrap(), &t1);
        assert!(TermMatcher::matches(&m, &t1));
        assert!(!TermMatcher::matches(&m, &t2));
    }

    #[test]
    fn test_array2_as_matcher() {
        let m = [
            BoxTerm::new_iri("http://champin.net/#pa").unwrap(),
            BoxTerm::new_iri("http://example.org/").unwrap(),
        ];
        // comparing to a term using a different term data, and differently cut,
        // to make the test less obvious
        let t1 = RcTerm::new_iri_suffixed("http://champin.net/#", "pa").unwrap();
        let t2 = RcTerm::new_iri("http://example.org/").unwrap();
        let t3 = RcTerm::new_iri("http://example.org/other").unwrap();

        let mc = TermMatcher::constant(&m);
        assert!(mc.is_none());
        assert!(TermMatcher::matches(&m[..], &t1));
        assert!(TermMatcher::matches(&m[..], &t2));
        assert!(!TermMatcher::matches(&m[..], &t3));
    }

    #[test]
    fn test_array1_as_matcher() {
        let m = [BoxTerm::new_iri("http://champin.net/#pa").unwrap()];
        // comparing to a term using a different term data, and differently cut,
        // to make the test less obvious
        let t1 = RcTerm::new_iri_suffixed("http://champin.net/#", "pa").unwrap();
        let t2 = RcTerm::new_iri("http://example.org/").unwrap();

        let mc = TermMatcher::constant(&m[..]);
        assert!(mc.is_some());
        assert_eq!(mc.unwrap(), &t1);
        assert!(TermMatcher::matches(&m[..], &t1));
        assert!(!TermMatcher::matches(&m[..], &t2));
    }

    #[test]
    fn test_array0_as_matcher() {
        let m: [BoxTerm; 0] = [];
        // comparing to a term using a different term data, and differently cut,
        // to make the test less obvious
        let t1 = RcTerm::new_iri_suffixed("http://champin.net/#", "pa").unwrap();

        let mc = TermMatcher::constant(&m[..]);
        assert!(mc.is_none());
        assert!(!TermMatcher::matches(&m[..], &t1));
    }

    #[test]
    fn test_func_as_matcher() {
        let t1 = RcTerm::new_iri_suffixed("http://champin.net/#", "pa").unwrap();
        let t2 = RcTerm::new_iri("http://example.org/").unwrap();

        let m = |t: &RefTerm| t.value().starts_with("http://champin");
        assert!(TermMatcher::constant(&m).is_none());
        assert!(TermMatcher::matches(&m, &t1));
        assert!(!TermMatcher::matches(&m, &t2));
    }
}
