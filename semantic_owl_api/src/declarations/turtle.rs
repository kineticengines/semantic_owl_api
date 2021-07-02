//! Turtle module defines representaion of turtle documents

use std::{collections::VecDeque, iter::FromIterator};

/// StatementKind used to map turtke parse results
#[derive(Debug, PartialEq)]
pub enum StatementKind {
  Comment,
  BasePrefix,
  NormPrefix,
  PartOf,
  StatementWithTerminator,
  Whitespace,
  Terminator,
  NotATurtle,
  None,
}

/// TurtleHeaderItem represents the header part of a Turtle document.
/// An example of such a header in a Turtle document is as:
/// ```ttl
/// @prefix skos: <http://www.w3.org/2004/02/skos/core#> .
/// @prefix owl:  <http://www.w3.org/2002/07/owl#> .
/// @prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .
/// @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
/// @prefix umls: <http://bioportal.bioontology.org/ontologies/umls/> .
/// ```
/// Often a base is provided. If the base is absent, it will be inferred in the
/// document
///
/// If the header is present, it will be of the form (an example)
/// ```ttl
/// @base <http://example.org/> .
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurtleHeaderItem<'document, R> {
  // determines whether the header item is a `base` or not
  pub is_base: bool,

  // the prefix namespace. Example; skos, owl, rdfs, xsd,umls.
  // this will be absent for base header items
  pub prefix_namespace: &'document str,

  // the URL where the prefix points to. Example <http://www.w3.org/2004/02/skos/core#>
  pub prefix_iri: &'document str,

  // the raw item string
  pub raw_header: R,
}

impl<'document, R> TurtleHeaderItem<'document, R> {
  pub fn new(
    is_base: bool,
    prefix_namespace: &'document str,
    prefix_iri: &'document str,
    raw_header: R,
  ) -> TurtleHeaderItem<'document, R> {
    Self {
      is_base,
      prefix_namespace,
      prefix_iri,
      raw_header,
    }
  }
}

/// TurtleBodyItem is a statement in a turtle document.
/// Simple example of a turtle statement:
/// ```ttl
/// <http://purl.bioontology.org/ontology/UATC/>
///     a owl:Ontology ;
///     rdfs:comment "RDF Version of the UMLS ontology ATC; converted with the UMLS2RDF tool (https://github.com/ncbo/umls2rdf), developed by the NCBO project." ;
///     rdfs:label "ATC" ;
///     owl:imports <http://www.w3.org/2004/02/skos/core> ;
///     owl:versionInfo "2020ab" .
///```
///
#[derive(Debug, Clone, PartialEq)]
pub struct TurtleBodyItem<'document> {
  pub subject: &'document str,
  pub predicate_object: VecDeque<TurtlePredicateObject<'document>>,
}

/// TurtlePredicateObject is a combination of predicate and object retrieved
/// from a turtle statement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurtlePredicateObject<'document> {
  pub raw_predicate_object: &'document str,

  // represents the predicate part of a turtle triple
  // predicate can be prefixed or be an URI
  pub predicate: &'document str,

  // indicates whether the predicate is a URI or not
  pub predicate_is_url: bool,

  // represents the object part of a turtle triple
  // object can be prefixed or be an URI
  pub object: &'document str,

  // indicates whether the object is a URI or not
  pub object_is_url: bool,
}

/// TurtleDocument is the composition of an entire turtle document. It is the sum of turle headers and body items.
/// A turtle document can be very large. This struct is used to represent such a document
/// as a summation of it'document atomic structures
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TurtleDocument<'document, R> {
  pub headers: VecDeque<TurtleHeaderItem<'document, R>>,
  pub body: VecDeque<TurtleBodyItem<'document>>,
}

impl<'document, R> TurtleDocument<'document, R>
where
  R: Copy,
{
  pub fn new() -> TurtleDocument<'document, R> {
    let headers: VecDeque<TurtleHeaderItem<'document, R>> = VecDeque::new();
    let body: VecDeque<TurtleBodyItem<'document>> = VecDeque::new();
    TurtleDocument { headers, body }
  }

  /// base_iri returns the IRI of the base prefix
  /// example
  /// `@base <http://example.org/> .` returns Option of `<http://example.org/>`
  fn base_iri(&'document self) -> Option<&'document str> {
    let base: VecDeque<TurtleHeaderItem<'document, R>> =
      self.headers.iter().filter(|x| x.is_base).collect();
    match base.len() {
      1 => {
        let raw = base[0].prefix_iri;
        let raw = raw.strip_prefix('<')?;
        let raw = raw.strip_suffix('>')?;
        Some(raw)
      }
      _ => None,
    }
  }
}

impl<'document, R: 'document> FromIterator<&'document TurtleHeaderItem<'document, R>>
  for VecDeque<TurtleHeaderItem<'document, R>>
where
  R: Copy,
{
  fn from_iter<T: IntoIterator<Item = &'document TurtleHeaderItem<'document, R>>>(iter: T) -> Self {
    let mut headers: VecDeque<TurtleHeaderItem<'document, R>> = VecDeque::new();
    for h in iter {
      headers.push_back(*h);
    }
    headers
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_return_base_prefix0() {
    let mut document = TurtleDocument::new();
    let header = TurtleHeaderItem::new(
      true,
      "@base",
      "<http://example.org/>",
      "@base <http://example.org/> .",
    );
    document.headers.push_back(header);
    let iri0 = document.base_iri();
    let iri1 = document.base_iri();
    assert_eq!(Some(iri0), Some(iri1));
  }

  #[test]
  fn should_return_base_prefix1() {
    let mut document = TurtleDocument::new();
    let header = TurtleHeaderItem::new(
      true,
      "@base",
      "http://example.org/",
      "@base <http://example.org/> .",
    );
    document.headers.push_back(header);
    let iri0 = document.base_iri();
    assert_eq!(iri0, None);
  }

  #[test]
  fn should_return_base_prefix2() {
    let mut document = TurtleDocument::new();
    let header = TurtleHeaderItem::new(
      false,
      "@base",
      "<http://example.org/>",
      "@base <http://example.org/> .",
    );
    document.headers.push_back(header);
    let iri0 = document.base_iri();
    assert_eq!(iri0, None);
  }
}
