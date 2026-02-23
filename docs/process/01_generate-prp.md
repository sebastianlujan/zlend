# Create PRP

## Feature file: $ARGUMENTS

Generate a complete PRP for a OGBank protocol feature. Read the feature file first to understand what needs to be designed and implemented.

OGBank is a privacy-preserving lending protocol on Avalanche that uses ZCash shielded UTXOs as collateral to borrow ERC-20 tokens via Aave V3, verified by Ultrahonk zero-knowledge proofs. The project is in research/design phase — there is no deployed code yet.

The AI agent only gets the context you append to the PRP. Assume the agent has access to the codebase and web search — include or reference all research findings in the PRP.

## MANDATORY FIRST STEP

**ALWAYS** read the existing documentation to understand the protocol:

- **`docs/technical/`** — Architecture, protocol spec, smart contracts, privacy model, ZCash integration, resolved research
- **`docs/product/`** — Product overview, problem/solution, user personas, user journeys, market data, success metrics
- **`docs/research/`** — Deep-dive investigations (Kohaku codebase, data requirements, adapter design)

Key documents to review:
- `docs/technical/00_overview.md` — Architecture diagram, key concepts, full stack
- `docs/technical/02_protocol.md` — OGBank Units, key derivation, full user flow
- `docs/technical/03_contracts.md` — Contract architecture (OGBankContract, Ultrahonk Verifier, Aave V3)
- `docs/technical/04_privacy-model.md` — Privacy guarantees, relayer model, threat model
- `docs/product/01_overview.md` — Product overview and vision

## Research Process

### 1. Review Existing Documentation (MANDATORY)
- Read all relevant docs in `docs/technical/` for the feature's domain
- Check `docs/research/` for deep-dives that apply
- Review `docs/product/` for user context and success metrics
- Identify gaps — what's documented vs. what the feature needs

### 2. Codebase Analysis
- Check `contracts/` for existing Solidity code and patterns
- Check `circuits/` for existing Noir circuits
- Review test patterns in `test/`
- Note established conventions (naming, structure, patterns)

### 3. External Research (only after reviewing existing docs)
- Protocol documentation (Aave V3, ZCash ZIP-32, Noir/Barretenberg)
- Similar implementations in other privacy protocols
- Security considerations and known attack vectors
- Include specific URLs to documentation sections
- Save findings to `research/` if broadly useful

### 4. User Clarification (if needed)
- Which protocol phase does this feature target (MVP vs Phase 2+)?
- Integration requirements with existing components?
- Privacy constraints and threat model considerations?

## PRP Sections

Using `PRPs/templates/prp_base.md` as template, the PRP must include:

### 1. Feature Context
- Which phase: MVP or Phase 2+
- Where it fits in the protocol architecture (reference `docs/technical/01_architecture.md`)
- Dependencies on other protocol components
- Reference to relevant product docs (`docs/product/`)

### 2. Functional Requirements
- Specific behavior referencing technical docs
- User flow (reference `docs/product/06_user-journey.md` for existing flows)
- Privacy requirements (reference `docs/technical/04_privacy-model.md`)
- On-chain vs off-chain responsibilities

### 3. Technical Specification
- **Smart contracts**: New contracts or modifications to OGBankContract (reference `docs/technical/03_contracts.md`)
- **ZK circuits**: Noir circuit design, public/private inputs, proof structure
- **ZCash integration**: ZIP-32 derivation, viewing keys, UTXO handling (reference `docs/technical/05_zcash-integration.md`)
- **Aave V3 integration**: Supply/borrow/repay interactions
- **Relayer**: Transaction submission and privacy preservation

### 4. Security & Privacy Considerations
- Threat model impact (reference `docs/technical/04_privacy-model.md`)
- Nullifier handling and replay prevention
- Relayer trust assumptions
- Potential attack vectors specific to this feature

### 5. Dependencies & Implementation Sequence
- What must exist before this feature can be built
- Ordered list of implementation tasks
- Integration points with existing components

### 6. Success Criteria
- Measurable outcomes referencing `docs/product/09_metrics.md`
- Protocol-level acceptance criteria
- Privacy guarantees that must hold

### 7. Testing Strategy
- **Unit tests**: Solidity tests with Foundry (`forge test`)
- **Fork tests**: Against Aave V3 on Avalanche fork
- **ZK proof verification**: Noir circuit tests, proof generation/verification
- **Integration tests**: End-to-end flow from UTXO lock to borrow
- **Privacy tests**: Verify no information leakage

## Validation Gates (Must be Executable)

```bash
# Compile contracts
forge build

# Run Solidity tests
forge test

# Compile Noir circuits (if applicable)
nargo compile

# Run Noir tests (if applicable)
nargo test

# Verify proof generation (if applicable)
nargo prove && nargo verify
```

### Required Compliance
- [ ] Follows OGBank architecture patterns (`docs/technical/01_architecture.md`)
- [ ] Respects privacy model (`docs/technical/04_privacy-model.md`)
- [ ] Nullifier handling prevents replay attacks
- [ ] ZK proofs verify correctly on-chain
- [ ] Aave V3 integration follows existing patterns
- [ ] Relayer interactions preserve privacy
- [ ] No information leakage between ZCash and Avalanche identities

---

**CRITICAL**: After researching the docs and exploring the codebase, **think hard** about the protocol implications before writing the PRP.

## Output

Save as: `PRPs/{feature-name}.md`

## Quality Checklist
- [ ] All relevant docs in `docs/technical/` reviewed
- [ ] Product context from `docs/product/` referenced
- [ ] Existing research in `docs/research/` checked
- [ ] Feature phase clearly identified (MVP vs Phase 2+)
- [ ] Smart contract changes specified with Solidity interfaces
- [ ] ZK circuit design documented (inputs, constraints, outputs)
- [ ] Privacy implications analyzed against threat model
- [ ] Dependencies and implementation sequence defined
- [ ] Success criteria reference `docs/product/09_metrics.md`
- [ ] Testing strategy covers contracts, circuits, and integration
- [ ] Validation gates are executable (Foundry, Nargo)
- [ ] No references to web app patterns (Next.js, React, npm, etc.)
- [ ] Clear implementation path for a blockchain protocol

Score the PRP on a scale of 1-10 (confidence level to succeed in one-pass implementation).

**Goal**: One-pass implementation success through comprehensive context that respects OGBank's architecture, privacy model, and protocol design.
