# GUAC R&D Strategy

## Research Pillars

GUAC’s competitive moat comes from treating memory as a systems problem, not a model problem. Our R&D is organised around four pillars:

### 1. Memory Systems Architecture
- Deterministic retrieval from structured knowledge graphs.
- Addressable filesystem memory with path-based queries.
- Git-native persistence: branching, merging, diffing, and rollback at conversation scale.
- Conflict resolution for merged memory branches.

### 2. Temporal Compression
- Hierarchical summarisation of conversation history.
- Lossy-but-controllable compression with user-defined retention policies.
- Evaluation metrics: recall@k, faithfulness, token savings.
- Long-term goal: automatically maintain a living "narrative" of every relationship.

### 3. Low-Latency Inference Orchestration
- Optimise the end-to-end pipeline: retrieval, assembly, Groq inference, commit.
- Target <50ms p99 for typical queries.
- Cache hot knowledge-graph paths and recent memory windows.
- Abstract inference so users can plug in local, OpenAI, or other providers.

### 4. Character & Identity Consistency
- Immutable core + mutable memory model.
- Personality drift detection and correction.
- Multi-character memory isolation and shared world-state.
- Emotional salience scoring alongside importance/novelty/recency/repetition.

---

## Technical Roadmap

### v0.1 — Foundation (now)
- Local CLI with Git-backed memory.
- YAML knowledge graph and filesystem addressing.
- Rule-based classifier, Groq reasoning, memory scoring.
- Temporal compression (level-1 summaries).
- 42-test suite and static website.

### v0.2 — Collaboration (3–6 months)
- Team knowledge graphs with merge semantics.
- Cloud sync daemon for memory repositories.
- Web dashboard for memory inspection and search.
- Memory provider SDK for LangChain/LlamaIndex.
- Improved compression with multi-level summaries.

### v0.5 — Platform (6–12 months)
- Hosted GUAC Cloud with Team and Enterprise tiers.
- REST/gRPC memory API.
- Real-time memory streaming for multi-user agents.
- Compression evaluation harness and benchmarks.
- Character marketplace / template sharing.

### v1.0 — Infrastructure (12–24 months)
- Distributed memory replication.
- Pluggable inference backends with latency-based routing.
- Enterprise security: SSO, audit, private VPC.
- Memory query language (MQL) for complex graph + history queries.
- Research publications on temporal compression and character consistency.

---

## Open Source vs. Commercial Boundaries

| Open Source (MIT) | Commercial |
|-------------------|------------|
| `guac` CLI | GUAC Cloud hosting |
| Local memory engine | Team sync & collaboration |
| Knowledge graph core | Web dashboard |
| Compression algorithms | Enterprise SSO/audit |
| Character engine | Managed support |
| Test suite & docs | Professional services |

**Principle:** Anything needed to run GUAC locally stays open. Anything that requires our infrastructure or enterprise compliance is commercial.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Groq API changes or downtime | Medium | High | Abstract inference layer; support multiple providers. |
| Compression loses critical facts | Medium | High | Faithfulness evals, user-tunable thresholds, preserved summaries. |
| Git does not scale to massive memory repos | Medium | Medium | Implement sharding, pack optimization, optional remote storage. |
| Competition from incumbent vector DBs | High | Medium | Differentiate on determinism, auditability, and character use cases. |
| Talent scarcity in Rust + AI systems | Medium | High | Build in public; strong docs and contributor community. |

---

## Hiring & Funding Implications

### Key Hires (next 12 months)
1. **Rust systems engineer** — Git internals, performance, distributed systems.
2. **Applied AI researcher** — compression, memory evaluation, character consistency.
3. **Founding product engineer** — Cloud dashboard, APIs, integrations.
4. **Developer advocate** — Content, community, partnerships.

### Funding Uses
- 50% engineering (core + cloud).
- 20% research (compression, evaluation, publications).
- 20% go-to-market (content, community, sales).
- 10% operations and legal.

---

## Research Partnerships

- Collaborate with academic labs on memory evaluation benchmarks.
- Publish datasets of compressed conversation histories.
- Sponsor open challenges: "Best compression with >95% factual recall."

---

## Success Metrics

| Pillar | Metric | Target (12 mo) |
|--------|--------|----------------|
| Memory systems | End-to-end latency p99 | <50ms |
| Compression | Token reduction vs. full history | >80% |
| Compression | Factual recall@5 | >95% |
| Inference | Provider abstraction coverage | 3+ backends |
| Character | Personality drift rate | <2% per 1K turns |
