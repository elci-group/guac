# GUAC — Git + Groq Augmented Cognition

GUAC is a hybrid conversational intelligence architecture that combines the deterministic persistence of Git with the ultra-low-latency inference of Groq.

> Models should not be responsible for remembering things.  
> Models should be responsible for reasoning over memory.

## Core Idea

GUAC treats memory as a first-class system component:

- **Knowledge Graph** — structured YAML facts under `memory/kg/`
- **Git-Native Memory** — every turn is a commit
- **Addressable Filesystem** — `/memory/projects/firebac/` resolves to a path
- **Temporal Compression** — old conversations are recursively summarized
- **Groq Reasoning Layer** — low-latency inference over assembled context

## Quick Start

```bash
cd /home/sal/GUAC
cp .env.example .env
# Edit .env and add your GROQ_API_KEY
# Or copy guac.toml.example to guac.toml for file-based configuration
cargo build --release

# Initialize a memory repository
./target/release/guac init

# Start chatting
./target/release/guac chat

# Set a knowledge-graph fact
./target/release/guac kg set user.name Rory
./target/release/guac kg set projects "[Marina, FireBAC, MRMR]"

# Branch a conversation
./target/release/guac branch create firebac-warfare
./target/release/guac chat --branch firebac-warfare

# Compress old memory
./target/release/guac compress

# Filesystem memory addressing
./target/release/guac fs list /memory/
./target/release/guac fs read /memory/kg/core.yaml
./target/release/guac fs write /memory/projects/firebac/warfare.yaml "tactics mode"

# Show git history
./target/release/guac status
```

## Architecture

```text
User Input
    │
    ▼
Intent Classifier ──▶ Memory Router
                           │
           ┌───────────────┴───────────────┐
           ▼                               ▼
  Knowledge Graph Lookup        Conversation History Search
           │                               │
           └───────────────┬───────────────┘
                           ▼
                   Context Assembler
                           │
                           ▼
                      Groq LLM
                           │
                           ▼
              Response + Memory Delta
                           │
                           ▼
                     Git Commit
```

## Memory Scoring

Every message receives:

- **Importance** — length, questions, emphasis
- **Novelty** — Jaccard distance from recent messages
- **Recency** — exponential decay with age
- **Repetition** — similarity to previous messages

Combined as `M = αI + βN + γR + δP`. Low-scoring messages are pruned during compression.

## Filesystem Memory

Addresses like `/memory/projects/firebac/warfare` map directly to `memory/projects/firebac/warfare.yaml`.

Queries become path lookups instead of vector searches.

## Testing

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

The suite includes:
- Unit tests for the knowledge graph, memory scoring, Git operations, classifier, router, assembler, character engine, filesystem memory, and compression
- Mocked HTTP tests for the Groq client via `mockito`
- End-to-end CLI integration tests using temporary directories

## Website & Company

- **Landing page:** open [`website/index.html`](website/index.html) in a browser.
- **Site kit:** see [`website/site-kit/`](website/site-kit/) for tokens and components.
- **Brand book:** [`company/brand.md`](company/brand.md)
- **Monetisation strategy:** [`company/monetization.md`](company/monetization.md)
- **Marketing strategy:** [`company/marketing.md`](company/marketing.md)
- **Brand aesthetic system:** [`company/brand-aesthetic.md`](company/brand-aesthetic.md)
- **Launch kit:** [`company/launch-kit.md`](company/launch-kit.md)
- **R&D strategy:** [`company/rd.md`](company/rd.md)
- **Enterprise-grade transformation plan:** [`company/enterprise-plan.md`](company/enterprise-plan.md)
- **Press kit:** [`website/press.html`](website/press.html)
- **Blog:** [`website/blog.html`](website/blog.html)
- **Docs landing:** [`website/docs.html`](website/docs.html)

## Requirements

- Rust 1.76+
- Git
- A Groq API key (optional; the CLI falls back to local echo mode without it)

## License

MIT
