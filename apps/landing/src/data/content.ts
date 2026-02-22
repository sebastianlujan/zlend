export const content = {
  nav: {
    links: [
      { label: "Problem", href: "#problem" },
      { label: "Solution", href: "#solution" },
      { label: "How It Works", href: "#how-it-works" },
      { label: "Market", href: "#market" },
      { label: "Team", href: "#team" },
    ],
    cta: { label: "Read the Docs", href: "#cta" },
  },

  hero: {
    headline: "Private Lending on Avalanche",
    subheadline:
      "Borrow USDC using your ZCash as collateral — without anyone seeing what you own.",
    cta: { label: "Learn More", href: "#problem" },
    secondaryCta: { label: "Read the Docs", href: "#cta" },
  },

  problem: {
    sectionLabel: "The Landscape",
    title: "DeFi Is Growing. Where Does Your ZEC Fit?",
    subtitle:
      "You hold ZEC because privacy matters to you. Meanwhile, DeFi lending keeps expanding. What would it look like to participate — without giving that up?",

    glassHouse: {
      title: "How DeFi Lending Works Today",
      position: {
        wallet: "0x7a3f...8291",
        collateral: "142.5 ETH",
        debt: "$185,000 USDC",
        health: "1.23",
        liquidation: "$1,298",
      },
      note: "Every position is fully transparent",
      observations: [
        "Collateral, debt, and liquidation prices are on-chain",
        "Anyone with a block explorer can see every position",
        "This is how it works — for everyone, on every protocol",
      ],
    },

    idleCapital: {
      title: "Meanwhile, 5.1M ZEC Sits Idle",
      earning: {
        label: "ETH in DeFi",
        apy: "4.2",
        items: ["Lending", "Borrowing", "Earning yield"],
      },
      idle: {
        label: "ZEC in Shielded Pools",
        apy: "0.00",
        items: ["No lending yet", "No borrowing yet", "No yield yet"],
      },
      stat: "What if your ZEC could work in DeFi — without compromising what makes it valuable?",
    },

  },

  solution: {
    sectionLabel: "The Solution",
    title: "ZLend: Where Privacy Meets DeFi",
    subtitle:
      "Use zero-knowledge proofs to verify your ZCash collateral without revealing what you own.",
    steps: [
      {
        number: "01",
        title: "Deposit",
        description:
          "Send ZEC to your ZLend escrow address. You get a viewing key to verify your balance at any time.",
      },
      {
        number: "02",
        title: "Prove & Borrow",
        description:
          "Your browser generates a ZK proof of your deposit. Submit it to borrow USDC from Aave V3 on Avalanche.",
      },
      {
        number: "03",
        title: "Repay & Claim",
        description:
          "Repay USDC on Avalanche. Prove repayment with a ZK proof. Get your ZEC back automatically.",
      },
    ],
  },

  howItWorks: {
    sectionLabel: "How It Works",
    title: "Zero-Knowledge Verified Lending",
    features: [
      {
        title: "Cross-Chain Private Collateral",
        description:
          "ZCash used as collateral on Avalanche without revealing balances or identity.",
      },
      {
        title: "ZK-Verified Solvency",
        description:
          "Ultrahonk proofs verify your collateral exists and is sufficient — without disclosing the amount.",
      },
      {
        title: "Built on Aave V3",
        description:
          "Battle-tested lending infrastructure. ZLend doesn't compete with Aave — it feeds it.",
      },
      {
        title: "MEV Protection",
        description:
          "Hidden positions can't be front-run or targeted for liquidation by MEV bots.",
      },
      {
        title: "Identity Unlinkability",
        description:
          "Nobody can connect your ZCash deposits to your Avalanche borrows. The relayer breaks the on-chain link.",
      },
      {
        title: "Escrow-Based Custody",
        description:
          "Protocol holds ZCash during loan with on-chain proof of obligation to return it after repayment.",
      },
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
        subtext: "ZLend changes this",
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
      "ZLend uses two keys derived from ZCash's ZIP-32 key tree. Think of it like a bank escrow: you deposit your valuables, the bank holds the vault key, and you hold a window to see inside.",
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
      "ZLend is built by a two-person team. Both members are fullstack — shared ownership of product, smart contracts, cryptography, frontend, and research.",
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
    title: "Explore ZLend",
    subtitle:
      "ZLend is in active development. Dive into the technical docs, verify the design, or join the conversation.",
    links: [
      { label: "Technical Docs", href: "#", icon: "docs" },
      { label: "GitHub", href: "#", icon: "github" },
      { label: "Join Community", href: "#", icon: "community" },
    ],
  },

  footer: {
    tagline: "Privacy-preserving lending on Avalanche.",
    copyright: `${new Date().getFullYear()} ZLend. All rights reserved.`,
    links: [
      { label: "Documentation", href: "#" },
      { label: "GitHub", href: "#" },
      { label: "Community", href: "#" },
    ],
  },
} as const;

export type Content = typeof content;
