export const content = {
  nav: {
    links: [
      { label: "Problem", href: "#problem" },
      { label: "How It Works", href: "#solution" },
      { label: "Under the Hood", href: "#how-it-works" },
      { label: "Market", href: "#market" },
      { label: "Team", href: "#team" },
    ],
    cta: { label: "Read the Docs", href: "#cta" },
  },

  hero: {
    headline: "Access DeFi. Keep your ZEC.",
    subheadline:
      "Lock your ZCash. Get liquidity on Avalanche. Privacy preserved.",
    cta: { label: "See How It Works", href: "#solution" },
    secondaryCta: { label: "Read the Docs", href: "#cta" },
  },

  problem: {
    sectionLabel: "The Landscape",
    title: "Your ZEC Has Privacy. It Has Almost Nothing Else.",
    subtitle:
      "Yield, swaps, farming, governance — all happening on Avalanche. Your ZEC can't touch any of it. Until now.",

    zecInventory: {
      title: "What Your ZEC Gives You",
      have: [
        { item: "Privacy", detail: "Shielded transactions" },
        { item: "Self-Custody", detail: "Your keys, your coins" },
        { item: "Censorship Resistance", detail: "Decentralized network" },
        { item: "Sound Money", detail: "21M cap, proof-of-work" },
      ],
      missing: [
        "Yield",
        "Swaps",
        "Lending",
        "Farming",
        "Liquidity Pools",
        "Governance",
        "Payments",
        "Cross-chain Access",
      ],
      note: "4 strengths. 8 blind spots.",
    },

    withStables: {
      title: "Lock ZEC. Get Stables. Access Everything.",
      alone: {
        label: "ZEC Alone",
        items: [
          "Privacy",
          "Shielded Transactions",
          "Self-Custody",
          "Censorship Resistance",
        ],
      },
      unlocked: {
        label: "ZEC + Stables on Avalanche",
        items: [
          "Yield (Aave, Benqi)",
          "Swaps (Trader Joe)",
          "Liquidity Pools",
          "Farming",
          "Governance",
          "Payments",
          "Cross-chain Bridges",
          "Full Avalanche DeFi",
        ],
      },
      closingLine: "Your ZEC stays locked. Your stables open every door on Avalanche.",
    },

  },

  solution: {
    sectionLabel: "How It Works",
    title: "Lock. Unlock. Access.",
    subtitle:
      "Your ZEC stays private. Your liquidity moves freely.",
    steps: [
      {
        number: "01",
        title: "Lock",
        description:
          "Send ZEC to your OGBank escrow. Secured by the protocol, verified by your viewing key.",
      },
      {
        number: "02",
        title: "Unlock Liquidity",
        description:
          "A ZK proof is generated in your browser. USDC arrives on Avalanche — ready for any DeFi protocol.",
      },
      {
        number: "03",
        title: "Return",
        description:
          "Done with DeFi? Return the USDC. Your private ZEC is released. Only when you choose.",
      },
    ],
  },

  howItWorks: {
    sectionLabel: "Under the Hood",
    title: "Zero-Knowledge. Full Access.",
    subtitle:
      "From shielded ZEC to USDC on Aave — without exposing a single byte.",
    pipeline: [
      {
        id: "deposit",
        step: "01",
        label: "Deposit ZEC",
        network: "Zcash",
        description: "Shielded escrow. Amount hidden.",
        color: "primary" as const,
      },
      {
        id: "proof",
        step: "02",
        label: "Generate Proof",
        network: "Browser",
        description: "Ultrahonk ZK proof. Client-side.",
        color: "accent" as const,
      },
      {
        id: "verify",
        step: "03",
        label: "Verify On-Chain",
        network: "Avalanche",
        description: "Smart contract verifies. Trustless.",
        color: "primary" as const,
      },
      {
        id: "borrow",
        step: "04",
        label: "Borrow USDC",
        network: "Aave V3",
        description: "Proven collateral. Instant liquidity.",
        color: "accent" as const,
      },
    ],
    highlights: [
      { label: "ZK-Verified Solvency" },
      { label: "MEV-Proof Positions" },
      { label: "Identity Unlinkability" },
      { label: "Escrow Custody" },
    ],
  },

  market: {
    sectionLabel: "Market Opportunity",
    title: "Untapped Capital, Waiting for Access",
    source: {
      label: "Source: Blockworks Analytics",
      href: "https://blockworks.com/analytics/zcash",
    },
    stats: [
      {
        value: "5.1M",
        unit: "ZEC",
        label: "In shielded pools",
        subtext: "~$255M, up 5x in 2 years",
      },
      {
        value: "$0",
        unit: "",
        label: "DeFi access for shielded ZEC",
        subtext: "OGBank changes this",
      },
      {
        value: "100%",
        unit: "",
        label: "ZEC stays on ZCash",
        subtext: "No wrapping, no synthetic tokens",
      },
      {
        value: "10-12K",
        unit: "",
        label: "Weekly shielded transactions",
        subtext: "Active, engaged users",
      },
    ],
  },

  trust: {
    sectionLabel: "Trust Model",
    title: "Transparent About Trade-Offs",
    description:
      "OGBank uses two keys derived from ZCash's ZIP-32 key tree. Think of it like a bank escrow: you deposit your valuables, the bank holds the vault key, and you hold a window to see inside.",
    keys: [
      {
        name: "Viewing Key",
        holder: "You",
        description:
          "See the balance at the escrow address. Verify your deposit at any time. Cannot move funds.",
      },
      {
        name: "Spending Key",
        holder: "Protocol",
        description:
          "Can move ZEC in/out of escrow. Required to return collateral after repayment.",
      },
    ],
    privacyTable: [
      { data: "Your ZCash address", visibility: "Hidden from Avalanche" },
      { data: "ZEC deposit amount", visibility: "Only you and the protocol" },
      { data: "USDC borrow amount", visibility: "Public on Avalanche" },
      {
        data: "ZCash-Avalanche link",
        visibility: "Only the protocol knows",
      },
    ],
  },

  team: {
    sectionLabel: "Team",
    title: "Built by Builders",
    description:
      "OGBank is built by a two-person team. Both members are fullstack — shared ownership of product, smart contracts, cryptography, frontend, and research.",
    members: [
      {
        name: "Franco",
        role: "Co-founder",
        focus:
          "Smart contract architecture, ZK circuit design, protocol security",
      },
      {
        name: "Seba",
        role: "Co-founder",
        focus:
          "ZCash integration, viewing key derivation, relayer service design",
      },
    ],
  },

  cta: {
    title: "Explore OGBank",
    subtitle:
      "OGBank is in active development. Dive into the technical docs, verify the design, or join the conversation.",
    links: [
      { label: "Technical Docs", href: "#", icon: "docs" },
      { label: "GitHub", href: "#", icon: "github" },
      { label: "Join Community", href: "#", icon: "community" },
    ],
  },

  footer: {
    tagline: "DeFi access for ZCash. Privacy preserved.",
    copyright: `${new Date().getFullYear()} OGBank. All rights reserved.`,
    links: [
      { label: "Documentation", href: "#" },
      { label: "GitHub", href: "#" },
      { label: "Community", href: "#" },
    ],
  },
} as const;

export type Content = typeof content;
