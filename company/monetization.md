# GUAC Monetisation Strategy

## Business Model at a Glance

GUAC is an **open-core** company. The CLI and local memory engine are MIT-licensed and free forever. Revenue comes from managed services, team collaboration, and enterprise features built on top of that core.

| Layer | Offering | Monetisation |
|-------|----------|--------------|
| Core | `guac` CLI, local Git memory, KG, compression | Free / open source |
| Cloud | Hosted memory sync, backups, team sharing | Subscription |
| Enterprise | SSO, audit, private deployment, support | Custom contracts |
| Services | Memory architecture reviews, onboarding | Professional services |

---

## Pricing Tiers

### Developer — Free

- Self-hosted `guac` CLI
- Unlimited local memory repositories
- Full knowledge graph, branching, compression
- Community support (GitHub Discussions)

**Goal:** Maximize adoption, build the default memory tool for AI engineers.

### Team — $49 per seat per month

- Cloud-hosted memory replicas
- Shared team knowledge graphs
- Real-time sync across devices
- Compression analytics dashboard
- Slack/email support
- 99.9% SLA

**Target:** Startups and product teams shipping AI characters or agents.

### Enterprise — Custom

- On-premise or VPC deployment
- SSO (SAML/OIDC) and SCIM provisioning
- Audit logs and compliance exports
- Custom retention and compression policies
- Dedicated success engineer
- Guaranteed response times

**Target:** Regulated industries, scale-ups with internal AI platforms, gaming studios.

---

## Revenue Streams

1. **Seat-based subscriptions** — Team and Enterprise tiers.
2. **Usage overages** — Cloud API calls, storage, and hosted compression beyond included quotas.
3. **Professional services** — Memory architecture audits, migration from vector DBs, custom character-engine integrations.
4. **Licensing** — White-label GUAC memory engine for OEMs building their own AI platforms.
5. **Training & certification** — Future offering for teams adopting memory-native AI practices.

---

## Unit Economics (Year 1 Assumptions)

| Metric | Assumption |
|--------|------------|
| Team ARPU | $49/month × 12 = $588/year |
| Gross margin (cloud) | ~75% at scale; lower in year 1 due to infrastructure ramp |
| Sales cycle (Enterprise) | 3–6 months |
| Free-to-paid conversion | 3–5% of active developer accounts |
| Target year-1 paying customers | 100 Team accounts + 5 Enterprise deals |
| Target year-1 ARR | $150K–$300K |

---

## Pricing Principles

- **Free must be genuinely useful.** A solo developer should never need to pay for local memory.
- **Pay for collaboration and trust.** Team sync, sharing, and compliance are the upgrade triggers.
- **No per-token pricing.** Avoid coupling revenue to model usage; monetise memory infrastructure, not inference.
- **Transparent public pricing.** Enterprise is the only custom tier.

---

## Path to $1M ARR

| Stage | Timeline | Focus |
|-------|----------|-------|
| Seed traction | 0–6 mo | 1,000+ CLI installs, 50 active Team trials |
| Product-market fit | 6–12 mo | 100 paying Team customers, first Enterprise logos |
| Scale | 12–24 mo | Self-serve expansion, channel partners, enterprise sales |
| Platform | 24–36 mo | OEM licensing, marketplace, certification revenue |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Users stay on free forever | Make team sync and hosted compression clearly valuable; limit free cloud storage. |
| Groq pricing changes | Abstract inference layer so customers can bring their own Groq/OpenAI/local model. |
| Enterprise sales slow | Build bottoms-up adoption; team plans create internal champions. |
| Competition from vector DBs | Position on determinism and auditability, not just retrieval. |
