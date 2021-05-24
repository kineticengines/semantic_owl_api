//! Turtle module provides mechanisms to parse and manipulate turtle documents

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
#[derive(Debug, Clone, PartialEq)]
pub struct TurtleHeaderItem<'a> {
    // determines whether the header item is a `base` or not
    pub is_base: bool,

    // the prefix unit. Example; skos, owl, rdfs, xsd,umls.
    // this will be absent for base header items
    pub prefix_name: &'a str,

    // the URL where the prefix points to. Example <http://www.w3.org/2004/02/skos/core#>
    pub prefix_iri: &'a str,

    // the raw item string
    pub raw_header: &'a str,
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
pub struct TurtleBodyItem<'a> {
    pub subject: &'a str,
    pub predicate_object: Vec<TurtlePredicateObject<'a>>,
}

/// TurtlePredicateObject is a combination of predicate and object retrieved
/// from a turtle statement.
#[derive(Debug, Clone, PartialEq)]
pub struct TurtlePredicateObject<'a> {
    pub raw_predicate_object: &'a str,

    // represents the predicate part of a turtle triple
    // predicate can be prefixed or be an URI
    pub predicate: &'a str,

    // indicates whether the predicate is a URI or not
    pub predicate_is_url: bool,

    // represents the object part of a turtle triple
    // object can be prefixed or be an URI
    pub object: &'a str,

    // indicates whether the object is a URI or not
    pub object_is_url: bool,
}

/// TurleSignature is the composition of an entire turtle document. It is the sum of turle headers and body items.
/// A turtle document can be very large. This struct is used to represent such a document
/// as a summation of it'a atomic structures
#[derive(Debug, Clone, PartialEq)]
pub struct TurtleSignature<'a> {
    pub headers: Vec<TurtleHeaderItem<'a>>,
    pub body: Vec<TurtleBodyItem<'a>>,
}
