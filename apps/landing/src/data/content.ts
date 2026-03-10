export const content = {
  nav: {
    links: [
      { label: "Home", href: "/" },
      { label: "Technology", href: "/technology" },
    ],
    cta: { label: "Read the Docs", href: "/technology" },
  },

  hero: {
    headline: "Access DeFi. Keep your ZEC",
    subheadline:
      "Lock your ZCash. Get liquidity on Avalanche. Privacy preserved.",
    cta: { label: "Go to App", href: "https://app.ogbank.vercel.app/" },
    secondaryCta: { label: "Read the Docs", href: "/technology" },
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
        { item: "Sound Money", detail: "21M cap, proof-of-work" },
      ],
      opportunities: [
        "Yield",
        "Swaps",
        "Farming",
        "Liquidity Pools",
        "Governance",
      ],
      note: "5.1M ZEC shielded. $0 earning yield.",
    },

    withStables: {
      title: "Lock ZEC. Get Stables. Access Everything.",
      alone: {
        label: "ZEC Alone",
        items: ["Privacy", "Self-Custody", "Sound Money"],
      },
      unlocked: {
        label: "ZEC + Stables on Avalanche",
        items: [
          "Yield (Aave, Benqi)",
          "Swaps (Trader Joe)",
          "Farming",
          "Liquidity Pools",
          "Governance",
        ],
      },
      closingLine: "Your ZEC stays locked. Your stables open every door on Avalanche. Every day without access is yield left on the table.",
    },

  },

  solution: {
    sectionLabel: "The Solution",
    title: "Lock. Unlock. Access.",
    subtitle: "Your ZEC stays private. Your liquidity moves freely.",
    worlds: {
      zcash: { label: "Private World", chain: "Zcash" },
      avalanche: { label: "DeFi World", chain: "Avalanche" },
    },
    bridge: {
      label: "OGBank",
      subtitle: "The bridge between privacy and DeFi",
    },
    steps: [
      { number: "01", title: "Lock", description: "Send ZEC to your OGBank escrow. Secured by the protocol, verified by your viewing key.", side: "zcash" as const },
      { number: "02", title: "Unlock Liquidity", description: "A ZK proof is generated in your browser. AUSD arrives on Avalanche — ready for any DeFi protocol.", side: "bridge" as const },
      { number: "03", title: "Return", description: "Done with DeFi? Return the AUSD. Your private ZEC is released. Only when you choose.", side: "avalanche" as const },
    ],
  },

  howItWorks: {
    sectionLabel: "Under the Hood",
    title: "Zero-Knowledge. Full Access.",
    subtitle: "From shielded ZEC to AUSD on Aave — without exposing a single byte.",
    layers: [
      { id: "deposit", step: "01", label: "Deposit ZEC", network: "Zcash", color: "gold" as const, description: "Shielded escrow. Amount hidden.", detail: "ZEC sent to a ZIP-32 derived address. Viewing key issued. Amount visible only to you and the protocol.", terminalLine: "zcash-cli z_sendmany ..." },
      { id: "proof", step: "02", label: "Generate Proof", network: "Browser", color: "accent" as const, description: "UltraHonk ZK proof. Client-side.", detail: "Noir circuit compiled to ACIR. UltraHonk prover generates a SNARK in your browser. No server.", terminalLine: "nargo prove --circuit deposit.nr" },
      { id: "verify", step: "03", label: "Verify On-Chain", network: "Avalanche", color: "primary" as const, description: "Smart contract verifies. Trustless.", detail: "OGBank contract receives the proof. UltraHonk verifier checks validity. Nullifier stored.", terminalLine: "cast call $OGBANK \"verify(bytes)\" $PROOF" },
      { id: "borrow", step: "04", label: "Borrow AUSD", network: "Aave V3", color: "accent" as const, description: "Proven collateral. Instant liquidity.", detail: "OGBank calls Aave V3 Pool.borrow(). AUSD sent to your wallet. No new lending pool.", terminalLine: "Pool.borrow(AUSD, amount, 2, 0, user)" },
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
        subtext: "Up 5x in 2 years",
        typedSubvalue: "~$255M",
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
      { data: "AUSD borrow amount", visibility: "Public on Avalanche" },
      {
        data: "ZCash-Avalanche link",
        visibility: "Only the protocol knows",
      },
    ],
  },

  narrativeBridges: [
    { prompt: "> Your ZEC is private. But is it productive? _" },
    { prompt: "> What if you didn't have to choose? _" },
    { prompt: "> But how does it actually work? _" },
  ],

  cta: {
    title: "> Ready to unlock your ZEC? _",
    subtitle:
      "OGBank is in active development. Dive into the technical docs, verify the design, or join the conversation.",
    terminalCommand: "$ ogbank init --network mainnet",
    links: [
      { label: "Technical Docs", href: "#", icon: "docs" },
      { label: "GitHub", href: "#", icon: "github" },
      { label: "Join Community", href: "#", icon: "community" },
    ],
  },

  chapters: [
    { label: "Intro", sectionId: "hero" },
    { label: "Landscape", sectionId: "problem" },
    { label: "Market", sectionId: "market" },
    { label: "Solution", sectionId: "solution" },
    { label: "How", sectionId: "how-it-works" },
    { label: "Trust", sectionId: "trust-compact" },
    { label: "Start", sectionId: "cta" },
  ],

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
