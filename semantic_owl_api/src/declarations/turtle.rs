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

  // e.g -> mro:BFO_0000050 rdf:type owl:ObjectProperty ;
  PartOfPredicateListWithSubject,

  // e.g -> cco:is_curated_in_ontology "http://www.ontologyrepository.com/CommonCoreOntologies/Mid/QualityOntology"^^xsd:anyURI ;
  PartOfPredicateList,

  // e.g -> rdf:type owl:NamedIndividual ,
  PartOfObjectListWithPredicate,

  // e.g -> owl:NamedIndividual ,
  PartOfObjectList,

  // e.g -> "http://www.ontologyrepository.com/CommonCoreOntologies/Mid/QualityOntology"^^xsd:anyURI ,
  //     -> "http://www.ontologyrepository.com/CommonCoreOntologies/Mid/QualityOntology"^^xsd:anyURI ;
  PartOfObjectListAsLiteral,

  // e.g -> [ rdf:type owl:Restriction ;
  //      owl:onProperty cco:has_process_part ;
  //      owl:someValuesFrom cco:Velocity
  //      ] ;
  PartOfCollectionList,

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

  // determines the whether the header item is has a blank namespace
  // this is avaible for non-base only
  pub is_empty: bool,

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
    is_empty: bool,
    prefix_namespace: Option<String>,
    prefix_iri: Option<String>,
    raw_header: Option<String>,
  ) -> TurtleHeaderItem {
    Self {
      is_base,
      is_empty,
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
  pub predicate: VecDeque<TurtlePredicate>,
}

/// TurtlePredicate is a combination of predicate and object retrieved
/// from a turtle statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurtlePredicate {
  pub raw_predicate_object: Option<String>,

  // indicates whether the predicate is a IRI or not
  // if `true`, `predicate_as_iri` should not be `None`. Should be for example -> <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/2021-03-01/ExtendedRelationOntology>
  pub predicate_is_iri: bool,

  // the predicate statement as a IRI. Example -> <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/2021-03-01/ExtendedRelationOntology>
  // valid is `predicate_is_iri` is TRUE
  pub predicate_as_iri_or_literal: Option<String>,

  // indicates whether the predicate is a literal or not
  // if `true`, `predicate_as_literal` should not be `None`. Should be for example -> "http://www.ontologyrepository.com/CommonCoreOntologies/Mid/ExtendedRelationOntology"^^xsd:anyURI
  pub predicate_is_literal: bool,

  // the literal predicate statement as a IRI. Example -> "http://www.ontologyrepository.com/CommonCoreOntologies/Mid/ExtendedRelationOntology"^^xsd:anyURI
  // valid is `predicate_is_literal` is TRUE
  pub predicate_as_literal: Option<String>,

  // represents the namespace part of a prefixed, non-IRI predicate. Example -> `rdf`
  // valid is `predicate_is_iri` and `predicate_is_literal` are FALSE
  pub predicate_namespace: Option<String>,

  // represents the value from a namespaced, non-IRI predicate. Example -> owl:versionIRI , versionIRI is the `preodicate_namespace_value`
  pub predicate_namespace_value: Option<String>,

  // list of objects pointing to the same predicate and implicitly the same subject
  pub object: VecDeque<TurtleObject>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurtleObject {
  pub raw_object: Option<String>,

  // indicates whether the predicate is a IRI or not
  // if `true`, `object_as_iri` should not be `None`. Should be for example -> <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/2021-03-01/ExtendedRelationOntology>
  pub object_is_iri: bool,

  // the predicate statement as a IRI. Example -> <http://www.ontologyrepository.com/CommonCoreOntologies/Mid/2021-03-01/ExtendedRelationOntology>
  // valid is `object_is_iri` is TRUE
  pub object_as_iri: Option<String>,

  // indicates whether the predicate is a IRI or not
  // if `true`, `object_as_literal` should not be `None`. Should be for example -> "A Definition Source that consists of a formalized doctrine in which the term is authoritatively defined."@en
  pub object_is_literal: bool,

  // the literall object statement . Example -> "A Definition Source that consists of a formalized doctrine in which the term is authoritatively defined."@en
  // valid is `object_is_literal` is TRUE
  pub object_as_literal: Option<String>,

  // represents the namespace part of a prefixed, non-IRI predicate. Example -> `rdf`
  // valid is `object_is_iri` and `object_is_iri` are FALSE
  pub object_namespace: Option<String>,

  // represents the value from a namespaced, non-IRI predicate. Example -> owl:versionIRI , versionIRI is the `preodicate_namespace_value`
  pub object_namespace_value: Option<String>,
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
  pub fn base_iri(&self) -> Option<String> {
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
      false,
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
