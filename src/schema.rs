use iri_rs::{Iri, iri};

/// The `rdfs:Resource` IRI, <http://www.w3.org/2000/01/rdf-schema#Resource>.
pub const RDFS_RESOURCE: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#Resource");
/// The `rdfs:Class` IRI, <http://www.w3.org/2000/01/rdf-schema#Class>.
pub const RDFS_CLASS: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#Class");
/// The `rdfs:Literal` IRI, <http://www.w3.org/2000/01/rdf-schema#Literal>.
pub const RDFS_LITERAL: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#Literal");
/// The `rdfs:Datatype` IRI, <http://www.w3.org/2000/01/rdf-schema#Datatype>.
pub const RDFS_DATATYPE: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#Datatype");
/// The `rdfs:Container` IRI, <http://www.w3.org/2000/01/rdf-schema#Container>.
pub const RDFS_CONTAINER: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#Container");
/// The `rdfs:ContainerMembershipProperty` IRI, <http://www.w3.org/2000/01/rdf-schema#ContainerMembershipProperty>.
pub const RDFS_CONTAINER_MEMBERSHIP_PROPERTY: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#ContainerMembershipProperty");

/// The `rdfs:range` IRI, <http://www.w3.org/2000/01/rdf-schema#range>.
pub const RDFS_RANGE: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#range");
/// The `rdfs:domain` IRI, <http://www.w3.org/2000/01/rdf-schema#domain>.
pub const RDFS_DOMAIN: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#domain");
/// The `rdfs:subClassOf` IRI, <http://www.w3.org/2000/01/rdf-schema#subClassOf>.
pub const RDFS_SUB_CLASS_OF: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#subClassOf");
/// The `rdfs:subPropertyOf` IRI, <http://www.w3.org/2000/01/rdf-schema#subPropertyOf>.
pub const RDFS_SUB_PROPERTY_OF: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#subPropertyOf");
/// The `rdfs:label` IRI, <http://www.w3.org/2000/01/rdf-schema#label>.
pub const RDFS_LABEL: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#label");
/// The `rdfs:comment` IRI, <http://www.w3.org/2000/01/rdf-schema#comment>.
pub const RDFS_COMMENT: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#comment");
/// The `rdfs:member` IRI, <http://www.w3.org/2000/01/rdf-schema#member>.
pub const RDFS_MEMBER: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#member");
/// The `rdfs:seeAlso` IRI, <http://www.w3.org/2000/01/rdf-schema#seeAlso>.
pub const RDFS_SEE_ALSO: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#seeAlso");
/// The `rdfs:isDefinedBy` IRI, <http://www.w3.org/2000/01/rdf-schema#isDefinedBy>.
pub const RDFS_IS_DEFINED_BY: Iri<&'static str> = iri!("http://www.w3.org/2000/01/rdf-schema#isDefinedBy");

/// The `rdf:langString` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#langString>.
pub const RDF_LANG_STRING: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString");
/// `rdf:dirLangString` — RDF 1.2 directional language-tagged literal datatype.
pub const RDF_DIR_LANG_STRING: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString");
/// `rdf:reifies` — RDF 1.2 annotation predicate linking a reifier to its
/// reified triple term.
pub const RDF_REIFIES: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies");
/// `rdf:TripleTerm` — RDF 1.2 class of reified triple terms.
pub const RDF_TRIPLE_TERM: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#TripleTerm");
/// The `rdf:HTML` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#HTML>.
pub const RDF_HTML: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#HTML");
/// The `rdf:XMLLiteral` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#XMLLiteral>.
pub const RDF_XML_LITERAL: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#XMLLiteral");
/// The `rdf:JSON` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON>.
pub const RDF_JSON: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#JSON");
/// The `rdf:Property` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#Property>.
pub const RDF_PROPERTY: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property");
/// The `rdf:Bag` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#Bag>.
pub const RDF_BAG: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#Bag");
/// The `rdf:Seq` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#Seq>.
pub const RDF_SEQ: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#Seq");
/// The `rdf:Alt` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#Alt>.
pub const RDF_ALT: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#Alt");
/// The `rdf:List` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#List>.
pub const RDF_LIST: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#List");
/// The `rdf:Statement` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement>.
pub const RDF_STATEMENT: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement");

/// The `rdf:type` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#type>.
pub const RDF_TYPE: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#type");
/// The `rdf:first` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#first>.
pub const RDF_FIRST: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#first");
/// The `rdf:rest` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>.
pub const RDF_REST: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#rest");
/// The `rdf:subject` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#subject>.
pub const RDF_SUBJECT: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#subject");
/// The `rdf:predicate` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate>.
pub const RDF_PREDICATE: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate");
/// The `rdf:object` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#object>.
pub const RDF_OBJECT: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#object");
/// The `rdf:value` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#value>.
pub const RDF_VALUE: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#value");

/// The `rdf:nil` IRI, <http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>.
pub const RDF_NIL: Iri<&'static str> = iri!("http://www.w3.org/1999/02/22-rdf-syntax-ns#nil");
