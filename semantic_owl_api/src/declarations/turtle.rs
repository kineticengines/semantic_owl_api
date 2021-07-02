//! Turtle module defines representaion of turtle documents
use serde::{Deserialize, Serialize};

use std::{collections::VecDeque, iter::FromIterator};

/// StatementKind used to map turtke parse results
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum StatementKind {
  // e.g -> ###  http://www.ontologyrepository.com/CommonCoreOntologies/Bent
  Comment,

  // e.g -> @base <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/QualityOntology> .
  BasePrefix,

  // e.g -> @prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .
  NormPrefix,

  // e.g -> cco:is_curated_in_ontology "http://www.ontologyrepository.com/CommonCoreOntologies/Mid/QualityOntology"^^xsd:anyURI ;
  PartOf,

  // e.g -> rdfs:label "Vermilion"@en .
  StatementWithTerminator,

  Whitespace,

  // e.g -> .
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurtleHeaderItem {
  // determines whether the header item is a `base` or not
  pub is_base: bool,

  // the prefix namespace. Example; skos, owl, rdfs, xsd,umls.
  // this will be absent for base header items
  pub prefix_namespace: Option<String>,

  // the URL where the prefix points to. Example <http://www.w3.org/2004/02/skos/core#>
  pub prefix_iri: Option<String>,

  // the raw item string
  pub raw_header: Option<String>,
}

impl TurtleHeaderItem {
  pub fn new(
    is_base: bool,
    prefix_namespace: Option<String>,
    prefix_iri: Option<String>,
    raw_header: Option<String>,
  ) -> TurtleHeaderItem {
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurtleBodyItem {
  pub subject: Option<String>,
  pub predicate_object: VecDeque<TurtlePredicateObject>,
}

/// TurtlePredicateObject is a combination of predicate and object retrieved
/// from a turtle statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurtlePredicateObject {
  pub raw_predicate_object: Option<String>,

  // represents the predicate part of a turtle triple
  // predicate can be prefixed or be an URI
  pub predicate: Option<String>,

  // indicates whether the predicate is a URI or not
  pub predicate_is_url: bool,

  // represents the object part of a turtle triple
  // object can be prefixed or be an URI
  pub object: Option<String>,

  // indicates whether the object is a URI or not
  pub object_is_url: bool,
}

/// TurtleDocument is the composition of an entire turtle document. It is the sum of turle headers and body items.
/// A turtle document can be very large. This struct is used to represent such a document
/// as a summation of it'document atomic structures
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TurtleDocument {
  pub headers: VecDeque<TurtleHeaderItem>,
  pub body: VecDeque<TurtleBodyItem>,
}

impl TurtleDocument {
  pub fn new() -> TurtleDocument {
    let headers: VecDeque<TurtleHeaderItem> = VecDeque::new();
    let body: VecDeque<TurtleBodyItem> = VecDeque::new();
    Self { headers, body }
  }

  /// base_iri returns the IRI of the base prefix
  /// example
  /// `@base <http://example.org/> .` returns Option of `<http://example.org/>`
  pub fn base_iri<'a>(&'a self) -> Option<String> {
    let base: VecDeque<TurtleHeaderItem> = self.headers.iter().filter(|x| x.is_base).collect();
    match base.len() {
      1 => {
        let h = base[0].clone();
        let raw = h.prefix_iri;
        let b = {
          match raw {
            Some(x) => {
              let r = &x;
              let r = r.strip_prefix('<')?;
              let r = r.strip_suffix('>')?;
              Some(String::from(r))
            }
            None => None,
          }
        };
        b
      }
      _ => None,
    }
  }
}

impl<'a> FromIterator<&'a TurtleHeaderItem> for VecDeque<TurtleHeaderItem> {
  fn from_iter<T: IntoIterator<Item = &'a TurtleHeaderItem>>(iter: T) -> Self {
    let mut headers: VecDeque<TurtleHeaderItem> = VecDeque::new();
    for h in iter {
      let i = h.clone();
      headers.push_back(i);
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
      Some(String::from("@base")),
      Some(String::from("<http://example.org/>")),
      Some(String::from("@base <http://example.org/> .")),
    );
    document.headers.push_back(header);
    let iri0 = document.base_iri();
    let iri1 = document.base_iri();
    assert_eq!(iri0, iri1);
  }

  #[test]
  fn should_return_base_prefix1() {
    let mut document = TurtleDocument::new();
    let header = TurtleHeaderItem::new(
      false,
      Some(String::from("@base")),
      Some(String::from("<http://example.org/>")),
      Some(String::from("@base <http://example.org/> .")),
    );
    document.headers.push_back(header);
    let iri0 = document.base_iri();
    assert_eq!(iri0, None);
  }
}
