// this module is transparently re-exported by its parent `term`
//
// Implement the Display trait for Term, using the Turtle family of syntax.

use std::fmt;

use crate::*;

impl<T> fmt::Display for Term<T>
where
    T: TermData,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_term(f, self)
    }
}

/// Write a single RDF term into `w` using the N-Triples syntax.
fn write_term<T, W>(w: &mut W, t: &Term<T>) -> fmt::Result
where
    T: TermData,
    W: fmt::Write,
{
    use self::Term::*;
    match t {
        Iri(iri) => {
            iri.write_fmt(w)?;
        }
        BNode(bn) => {
            bn.write_fmt(w)?;
        }
        Literal(lit) => {
            lit.write_fmt(w)?;
        }
        Variable(var) => {
            var.write_fmt(w)?;
        }
    };
    Ok(())
}

#[cfg(test)]
pub(crate) mod test {
    use crate::literal::AsLiteral;
    use crate::ns::*;
    use crate::*;
    use lazy_static::lazy_static;

    lazy_static! {
        pub(crate) static ref NT_TERMS: Vec<(StaticTerm, &'static str)> = vec![
            (
                StaticTerm::new_iri("http://example.org/foo/bar").unwrap(),
                r"<http://example.org/foo/bar>",
            ),
            (
                StaticTerm::new_iri_suffixed("http://example.org/foo/", "bar").unwrap(),
                r"<http://example.org/foo/bar>",
            ),
            (
                // IRI with non ascii term
                StaticTerm::new_iri("http://example.org/hé/\u{10000}/").unwrap(),
                "<http://example.org/hé/\u{10000}/>",
            ),
            (
                // BNode nice
                StaticTerm::new_bnode("foo_bar.baz").unwrap(),
                r"_:foo_bar.baz",
            ),
            (
                StaticTerm::new_literal_lang("chat", "fr-FR").unwrap(),
                r#""chat"@fr-FR"#,
            ),
            (
                "chat".as_term(),
                r#""chat""#,
            ),
            (
                StaticTerm::new_literal_dt("42", xsd::integer).unwrap(),
                r#""42"^^<http://www.w3.org/2001/XMLSchema#integer>"#,
            ),
            (
                " \n \r \\ \" hello world".as_term(),
                r#"" \n \r \\ \" hello world""#,
            ),
            (
                // Literal with non-ascii characters
                "é \u{10000}".as_term(),
                // in canonical form, non-ascii characters are NOT escaped in literals
                "\"é \u{10000}\"",
            )
        ];
    }

    #[test]
    fn terms() {
        for (term, expected) in NT_TERMS.iter() {
            let got = format!("{}", term);
            assert_eq!(&got, expected);
        }
    }
}
