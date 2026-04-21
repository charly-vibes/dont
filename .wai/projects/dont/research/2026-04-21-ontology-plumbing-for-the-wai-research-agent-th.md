# Ontology plumbing for the WAI research agent

The semantic-web ecosystem already has a near-complete analog to the OpenAPI stack, and most of the high-value pieces are free, actively maintained, and MCP-wrappable today. **For WAI, the single fastest path to useful ontology coverage is to plug in EBI's official OLS4 MCP endpoint, the OpenAlex REST API, the Wikidata SPARQL endpoint, and the Python Ontology Access Kit (OAK)** — together these four cover biomedical, cross-domain, and scholarly knowledge with zero hosting costs beyond an MCP client. The wider landscape adds breadth (BioPortal, AgroPortal, TIB) and depth (OntoGPT, BioCypher, GraphRAG) but the backbone is small. Everything below is mapped to the API-world analogy the user asked for, and flagged with maturity + integration effort.

## How the ontology stack maps to the API world

Think of it as five layers, each with a direct API-world counterpart:

| API world | Ontology world |
|---|---|
| OpenAPI / JSON Schema spec | **OWL / SHACL / LinkML / JSON-LD** (definition + validation) |
| Swagger Editor | **Protégé, WebProtégé, TopBraid Composer** |
| API gateway / REST endpoint | **SPARQL endpoint / triple store** (Fuseki, Oxigraph, Stardog, GraphDB) |
| RapidAPI / APIs.guru directory | **OLS4, BioPortal, OBO Foundry, LOV, Bioregistry, FAIRsharing** |
| Tool-calling SDK / MCP server | **OAK, OntoGPT, sparql-llm, BioMCP, OLS-MCP** |

The crucial 2025–2026 development is that **MCP has become the dominant plugin surface**: EBI publishes an official MCP endpoint for OLS4, and community MCP servers now exist for BioPortal, Wikidata, PubMed, Semantic Scholar, UniProt, GraphDB, Jena, Oxigraph, and generic SPARQL. For a plugin-based agent like WAI this collapses integration effort to configuration rather than coding.

## The specification layer: formats that define ontologies

**OWL 2** remains the W3C standard for expressive ontologies — analogous to a rigorous OpenAPI spec with Description Logic semantics. Its four profiles (DL, EL, QL, RL) trade expressivity for tractability; **OWL 2 EL** is what powers SNOMED CT and Gene Ontology reasoning at scale. OWL is heavy for most agent use-cases because it needs a Java or Python reasoner (HermiT, ELK, Pellet, owlready2) — treat it as read-only input, not something an LLM should author directly.

**SHACL** is the Shapes Constraint Language and is effectively **the JSON Schema of RDF**. It validates a graph against shape definitions and has become the preferred modeling layer even where OWL used to dominate (TopQuadrant has publicly pivoted from OWL to SHACL). For WAI, **pySHACL** is the go-to Python validator and can run in a sidecar to check any RDF that the LLM emits before it lands in a knowledge store. **ShEx** is a competing grammar-style validator used primarily by Wikidata EntitySchemas; SHACL has won everywhere else.

**LinkML** is the single most important format to highlight for a developer-friendly workflow. It is **YAML-based and compiles to JSON Schema, SHACL, OWL, ShEx, SQL DDL, Pydantic, GraphQL, Protobuf, and TypeScript** from one source. This is as close to "Swagger YAML for ontologies" as exists today. It is Apache-2.0, production-stable, and used by NIH Bridge2AI, the Biolink model, and the National Microbiome Data Collaborative. LLMs handle YAML fluently, which makes LinkML ideal as WAI's internal schema format for ontology plugins — compile once, get Pydantic models for Python tools, JSON Schema for MCP tool signatures, and SHACL for validation, all from the same file.

**SKOS** is the lightweight vocabulary for taxonomies and thesauri (MeSH, AGROVOC, EuroVoc use it). Think of it as "an enum catalog API": `prefLabel`, `altLabel`, `broader`, `narrower`. Plenty of auto-tagging use cases need nothing heavier. **JSON-LD** is the JSON serialization of linked data and the natural wire format between LLMs and knowledge sources — it is already how schema.org exposes 45M+ websites' structured data. Alongside these, **Turtle** is the de facto human-readable RDF authoring format and every triple store parses it.

## The query layer: triple stores and SPARQL endpoints as API gateways

**SPARQL 1.1** over HTTP is the query protocol. Every triple store speaks it, result formats are JSON/XML/CSV, and the `SERVICE` keyword enables federated queries across endpoints. For WAI the most relevant deployment options are:

**Oxigraph** (Rust, Apache-2.0) is a single-binary SPARQL 1.1 engine with Python, JS/WASM, and Rust bindings. **Because it runs in the browser via WASM, it is the best fit for WAI's GitHub-Pages client-side model** — no backend needed, just ship oxigraph.wasm and load ontology dumps locally. **Comunica**, a TypeScript modular query engine, complements this by federating SPARQL across remote endpoints from the browser with 89% SPARQL 1.1 test-suite coverage.

For server-side deployments, **Apache Jena Fuseki** is the canonical open-source SPARQL server (a Jena MCP server already exists on PulseMCP), **Stardog** and **Ontotext GraphDB** dominate the commercial/enterprise tier with native SHACL and GraphQL, and both have MCP integrations. **Blazegraph is effectively abandoned** (Amazon acquired it, Wikidata is actively migrating off it per Wikimedia phab T206560) — do not pick it for new work. **Virtuoso** still powers DBpedia and UniProt at production scale but is heavyweight. **GraphQL-over-RDF** via HyperGraphQL or Stardog/Ontotext lets teams expose ontology data through the GraphQL idiom many frontend developers already know.

On the Python side, **rdflib** is the universal library (parsers for every RDF serialization, local SPARQL engine), **owlready2** offers Pythonic OWL class access with bundled HermiT/Pellet reasoners and scales to 1B+ triples via SQLite. These are building blocks that any ontology-aware WAI plugin will lean on.

## The registry layer: where to discover ontologies

The ontology marketplaces split cleanly by domain, and critically, **most of the major scientific portals speak the same BioPortal-compatible REST API shape** — one client library unlocks a dozen registries.

**OLS4 at EMBL-EBI** (`ebi.ac.uk/ols4`) is the highest-leverage single integration. It hosts hundreds of biomedical and chemistry ontologies (GO, HPO, CHEBI, MONDO, UBERON, ENVO, EFO, CL, and more), has an open REST API with no authentication, and **exposes a native production MCP endpoint at `https://www.ebi.ac.uk/ols4/api/mcp`** — rare among major scientific infrastructure. Community MCP wrappers like `seandavi/ols-mcp-server` provide stdio transport for local agents.

**BioPortal** (NCBO, Stanford) is the broadest biomedical repository: **1,549 ontologies, 15.3M terms, and over 100M cross-ontology mappings** as of the 2025 *NAR* paper, including licensed vocabularies like SNOMED CT and NCIT that OLS doesn't fully mirror. It needs a free API key but offers REST, SPARQL (beta), term search, an **Annotator** service (free text → ontology terms), and a **Recommender** (which ontologies fit my data?). Two independent MCP servers exist: `ncbo/bioportal-mcp` (official NCBO) and `Augmented-Nature/BioOntology-MCP-Server`.

The **OntoPortal Alliance** is the open-source federation framework behind BioPortal. Because sibling portals — **AgroPortal** (agronomy), **EcoPortal** (ecology), **EarthPortal** (Earth systems), **MatPortal** (materials), **IndustryPortal** (manufacturing), **BiodivPortal** (biodiversity), **MedPortal** (Chinese biomed) — all share the same API, a single OntoPortal-aware WAI plugin automatically works across every domain. Federation across all these portals went live in December 2024, so one API call can now reach the whole alliance.

**OBO Foundry** is the curated consortium defining the gold-standard open biomedical ontologies and their interoperability principles (~250 ontologies tracked). It is not itself an API but a registry published as JSON-LD at `obofoundry.org/registry/ontologies.jsonld`; resolution goes through `purl.obolibrary.org`.

**Bioregistry** (`bioregistry.io`) is the meta-registry that stitches everything together. It integrates ~1,500 prefixes from OBO Foundry, OLS, BioPortal, MIRIAM, Wikidata, and 25 other sources, and **answers the essential disambiguation question "is `chebi` the same as `CHEBI` or `obo:CHEBI`?"** that breaks most naive LLM pipelines. It has an OpenAPI-documented REST API (no auth), a `pip install bioregistry` Python SDK, and a Docker image. An MCP wrapper would be trivial and is probably the single most valuable 30-minute addition to WAI.

**TIB Terminology Service** (`terminology.tib.eu`) is a German NFDI fork of OLS with 220+ ontologies skewed toward chemistry, engineering, and materials — OLS-compatible API so the same client works. **Linked Open Vocabularies (LOV)** catalogs ~750 general-purpose RDF vocabularies (FOAF, DC, SKOS, schema.org, PROV) with a free SPARQL endpoint at `lov.linkeddata.es/dataset/lov/sparql` — essential outside biomedical. **prefix.cc** is a zero-friction lookup for RDF namespace prefixes. **FAIRsharing** tracks ~4,000 standards/databases/policies but requires JWT auth and uses a CC BY-SA license that complicates redistribution. **UMLS** remains the gold standard for SNOMED↔ICD↔MeSH crosswalks but requires accepting the NLM license — gate it behind explicit user consent.

## LLM-to-ontology integration frameworks

Beyond raw endpoints, a distinct tier of tools is built specifically to bridge ontologies and LLMs.

**OAK (Ontology Access Kit)** by Chris Mungall's team is the library every ontology-aware WAI plugin should sit on. It provides a uniform Python adapter API across local OBO/OWL/SQLite, OLS, BioPortal, Ubergraph SPARQL, and others (`from oaklib import get_adapter; get_adapter("sqlite:obo:hp")`), covering search, ancestors, descendants, SSSOM mappings, text annotation, semsimian similarity, and lexical indexing. Ships with auto-download of 50+ OBO ontology SQLite dumps. Apache-2.0, actively maintained.

**OntoGPT** and its **SPIRES** method (Monarch Initiative, *Bioinformatics* 2024) is the canonical zero-shot ontology-grounded extraction pipeline: provide a LinkML schema, pass text, and get back structured YAML where every entity is grounded to an OBO CURIE. Works with any LiteLLM-supported model. The pattern — schema-as-prompt + ontology-grounding — is exactly what WAI's "extract structured knowledge from a paper" node should do.

**ROBOT** is the Java CLI and library for OBO release automation (reason, relax, extract MIREOT modules, validate). Less relevant for live querying, essential if WAI ever builds custom sub-ontologies. **CurateGPT** adds term-curation workflows atop ChromaDB vector search plus an LLM.

For knowledge-graph construction, **BioCypher** (Apache-2.0, *Nature Biotech* 2023) harmonizes heterogeneous biomedical sources to the Biolink model and emits Neo4j/ArangoDB/RDF; its sibling **BioChatter** is a chat interface on BioCypher KGs. **LlamaIndex PropertyGraphIndex** with `SchemaLLMPathExtractor` is the schema-constrained modern replacement for the older KnowledgeGraphIndex and works with Neo4j, Memgraph, FalkorDB, or in-memory. **Neo4j's `neo4j-graphrag-python`** and the `llm-graph-builder` React+FastAPI app are production-grade document-to-KG pipelines. **Microsoft GraphRAG** (~30k stars) answers "global" corpus-wide questions that pure vector RAG cannot — at significant indexing cost; the newer LazyGraphRAG variant cuts that cost substantially. **txtai** offers a lightweight all-in-one embeddings + semantic graph + agent framework without external DB dependencies. **PyKEEN** trains 40+ knowledge-graph embedding models for link prediction on top of any of these.

The **`sib-swiss/sparql-llm`** library (and its hosted MCP at `chat.expasy.org/mcp`) is the most sophisticated NL-to-SPARQL tool available: it retrieves similar SHACL-annotated SPARQL examples via embeddings, injects them into the prompt, generates SPARQL, **validates the generated query against the endpoint's VoID schema, and auto-corrects** — a published pattern (arXiv 2512.14277) that substantially reduces hallucinated queries against UniProt, Rhea, Bgee, and similar life-science endpoints.

## MCP servers available today

The MCP ecosystem turned one year old on 2025-11-25 with a major spec refresh adding the Tasks abstraction. Ontology-adjacent MCP servers worth knowing:

- **OLS4 MCP** — hosted by EBI at `ebi.ac.uk/ols4/api/mcp` (the only clearly institutional one); also `seandavi/ols-mcp-server` for stdio.
- **BioPortal MCP** — `ncbo/bioportal-mcp` (official) and `Augmented-Nature/BioOntology-MCP-Server` (community).
- **BioMCP** (`genomoncology/biomcp`) — unified PubMed/ClinicalTrials.gov/MyVariant MCP.
- **sparql-llm MCP** — NL-to-SPARQL for life-science endpoints with VoID-based validation.
- **mcp-rdf-explorer** (`emekaokoye`) — generic SPARQL + Turtle explorer.
- **open-ontologies** (`fabio-rovai`) — Rust single-binary, Oxigraph-backed, 42 tools including native OWL-DL reasoning and SHACL validation.
- **Wikidata MCPs** — `zzaebok/mcp-wikidata`, `joelgombin/mcp-wikidata`.
- **GraphDB MCP** and **Stardog MCP** from the vendors.
- **PubMed, Semantic Scholar, OpenAlex MCPs** — multiple community implementations.

The **BioContextAI Registry** at `biocontext.ai/registry` is a meta-directory of biomedical MCP servers — directly useful to WAI users who want to discover domain-specific tools.

## The domain ontologies worth wiring in

Biomedical and life sciences dominate by sheer maturity and infrastructure. **Gene Ontology** (~45,000 terms, GO Consortium) is the functional-annotation gold standard — but note that **GO is deprecating its public SPARQL endpoint**, so plan around `api.geneontology.org` or local Blazegraph journals. **ChEBI** covers ~200,000 bio-relevant chemicals; **SNOMED CT** has 350,000+ clinical concepts but requires national licenses; **MeSH** (public domain) indexes PubMed via a free SPARQL endpoint at `id.nlm.nih.gov/mesh/sparql`; **HPO** (~17,000 classes) is the rare-disease phenotyping standard; **Mondo** harmonizes DOID/OMIM/Orphanet/ICD/NCIT/SNOMED/MeSH into ~25,000 disease classes; **UBERON** and **Cell Ontology** cover cross-species anatomy and cell types; **NCBI Taxonomy** lists 2.5M+ organisms; **UniProt's SPARQL endpoint** is the largest public biology RDF store at **230+ billion triples**.

For physical, earth and materials sciences, **ENVO** covers environments, **CHEMINF** covers chemical identifiers, **EMMO** is the European materials top-level ontology, and **SWEET** is the 6,000-concept Earth-science ontology maintained by ESIP.

Cross-domain, **Wikidata** is indispensable — 110M+ items, CC0, queryable via `query.wikidata.org/sparql` with a 60-second timeout that forces agents to write narrow queries with `LIMIT`. **DBpedia** is partially eclipsed by Wikidata but still useful where Wikipedia article structure matters. **schema.org** (with **Bioschemas** extension) is the right format for *outputs* — marking up research datasets and papers so they appear in Google Dataset Search and FAIR registries.

For scholarly metadata, **OpenAlex** is the top-priority integration for any research workflow: **250M works, 95M authors, CC0, no API key, 100k-calls/day polite pool, and a ready-made MCP server**. It has effectively replaced Microsoft Academic Graph. Pair it with **Crossref** (authoritative DOI metadata) and **Semantic Scholar** (paper embeddings and TLDRs). The **SPAR ontologies** (FaBiO, CiTO) provide the vocabulary for emitting citation metadata as RDF; the **Computer Science Ontology (CSO)** provides 14,000 CS topics for classifying CS literature.

## Integration patterns and recommended WAI architecture

Seven integration patterns cover the design space, with very different cost/benefit profiles:

| Pattern | Latency | Offline | Effort | Best for |
|---|---|---|---|---|
| Direct SPARQL plugin | 100ms–10s | No | Low–Med | Wikidata, DBpedia, UniProt, MeSH |
| REST API wrapper | 50–500ms | No | Low | OLS, BioPortal, OpenAlex, Crossref, PubChem |
| MCP server reuse | Same as API | Depends | Very low | OLS4, BioMCP, sparql-llm, Wikidata |
| Local OBO/OWL via OAK | <50ms | Yes | Medium | Stable mid-size OBOs (GO, HPO, CL, UBERON, ChEBI, Mondo) |
| Embedding semantic search | <50ms | Yes | Med–High | Fuzzy term grounding across large vocabularies |
| Entity linking (scispaCy/OAK) | 100ms–2s | Partial | High | Grounding free-text mentions to IRIs |
| SHACL validation | Local, fast | Yes | Medium | Validating outputs against schemas/profiles |

Given WAI's static GitHub-Pages hosting model, the cleanest concrete architecture is a three-ring design. The **inner ring runs in-browser**: Oxigraph-WASM as a local triple store, Comunica for federated SPARQL to Wikidata/DBpedia/user endpoints, rdf-validate-shacl for JS-side validation, and pre-computed embedding indexes for semantic term search. The **middle ring is thin MCP clients** pointing at EBI's OLS4 MCP, OpenAlex, BioMCP, sparql-llm, and Wikidata MCP — zero infrastructure, immediate coverage of biomedical, scholarly, life-science, and cross-domain knowledge. The **outer ring is an optional Python sidecar** wrapping OAK, OntoGPT, and pySHACL when a user wants offline OBO querying, LLM-extraction grounding, or SHACL validation beyond what the browser can do.

**Authoring happens in LinkML YAML**: one schema file per plugin compiles to Pydantic (Python tools), JSON Schema (MCP tool signatures), SHACL (validation), and OWL (reasoning when needed) — avoiding duplication and keeping everything LLM-editable.

## The eight integrations to do first

Ranked by value per effort for a general scientific research workflow:

1. **OpenAlex REST + MCP** — highest-value single integration; covers scholarly search/citations/topics with CC0 data, no key, 1–2 days to wrap.
2. **EBI OLS4 via its official MCP endpoint** — one hook unlocks 250+ biomedical ontologies with production-grade uptime and no API key.
3. **Wikidata SPARQL plugin with canonical query templates** — universal cross-domain grounding layer; ship named templates with `LIMIT` guards to stay under the 60-second timeout.
4. **Bioregistry Python SDK embedded in WAI** — disambiguates prefixes and CURIEs across all other plugins; prevents the most common class of LLM hallucination in ontology workflows.
5. **UniProt SPARQL with a curated query library** — the 230B-triple reference resource for protein/gene/taxonomy questions.
6. **PubChem PUG-REST + ChEBI via OLS** — chemistry coverage for drug discovery and cheminformatics.
7. **OAK with a bundled OBO ontology pack** — offline, low-latency GO/HPO/CL/UBERON/Mondo/ChEBI/ENVO for hot-path entity linking and graph traversal.
8. **schema.org + Bioschemas + pySHACL** — for emitting structured, FAIR-compliant research outputs rather than consuming knowledge.

## Risks and caveats to surface in the WAI UI

**Licensing varies sharply**: SNOMED CT, UMLS, OMIM, and DrugBank need user-obtained licenses and cannot be bundled with WAI; FAIRsharing is CC BY-SA which complicates redistribution; most OBO ontologies are CC BY or CC0. Plugin metadata should carry license fields and gate licensed sources behind user consent. **Query reliability varies**: Wikidata enforces a 60-second processing budget per minute per UA+IP, GO is sunsetting its public SPARQL endpoint, and `purl.org` has a history of flakiness (prefer `w3id.org` or direct obolibrary PURLs). **MCP quality varies**: the OLS4 MCP is clearly institutional; most other community MCPs are single-maintainer prototypes — vet before depending on them. **LLM-generated SPARQL hallucinates prefixes and properties frequently** — always round-trip through Bioregistry for CURIE normalization and pySHACL for validation before committing outputs to a store. **Embedding-based semantic search is fuzzy** — pair it with a deterministic ID resolver before any fact is reported as true.

## What this means for WAI

The ontology ecosystem has quietly built exactly what a plugin-based AI research tool needs: a full OpenAPI-analogous stack from authoring (LinkML/SHACL) through registries (OLS4/BioPortal/Bioregistry) to tool-calling (MCP). **The gap that remained in 2024 — a standard protocol for LLM agents to reach all of this — closed in 2025 when MCP servers proliferated and EBI shipped a native MCP endpoint.** For WAI, the implication is that "ontology plugins" should not be built from scratch. They should be thin wrappers over existing MCP servers (OLS4, OpenAlex, BioMCP, Wikidata, sparql-llm) plus a small in-browser RDF runtime (Oxigraph-WASM + Comunica) plus a Python sidecar only where offline OBO reasoning is needed. Doing this in the order listed above gets a scientist-ready tool with billion-triple knowledge access in a week or two, not a quarter.
