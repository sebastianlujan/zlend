import { useState, useCallback } from "react";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

const features = [
  {
    title: "Noir + Ultrahonk",
    description:
      "ZK proofs generated client-side in Noir, verified on-chain by the Ultrahonk proving system from Aztec's Barretenberg.",
    tag: "ZK Proofs",
  },
  {
    title: "Aave V3 Integration",
    description:
      "No forked lending pool. OGBank plugs directly into Aave V3's existing deployment on Avalanche.",
    tag: "Infrastructure",
  },
  {
    title: "ZIP-32 Key Derivation",
    description:
      "Deterministic address derivation from ZCash's hierarchical wallet standard. Same inputs always produce the same OGBank Unit.",
    tag: "Cryptography",
  },
  {
    title: "Nullifier Protection",
    description:
      "Each lock cycle creates a unique nullifier. Prevents replay attacks and double-collateralization across cycles.",
    tag: "Security",
  },
];

export function Features() {
  const gridRef = useScrollAnimation<HTMLDivElement>({
    animation: "fadeUp",
    childSelector: "[data-feature-box]",
    staggerDelay: 0.1,
  });

  return (
    <SectionWrapper>
      <div>
        <div className="text-center mb-16">
          <Badge>Technical Foundation</Badge>
          <h2 className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            Built on Proven Primitives
          </h2>
        </div>

        <div ref={gridRef} className="grid md:grid-cols-2 gap-5">
          {features.map((feat) => (
            <FeatureBox key={feat.title} feat={feat} />
          ))}
        </div>
      </div>
    </SectionWrapper>
  );
}

function FeatureBox({ feat }: { feat: (typeof features)[number] }) {
  const [open, setOpen] = useState(false);

  const toggle = useCallback(() => {
    setOpen((prev) => !prev);
  }, []);

  return (
    <div
      data-feature-box
      className="group relative rounded-xl border border-surface-800/50 bg-surface-900/60 backdrop-blur-sm h-[240px] overflow-hidden transition-colors duration-300 hover:border-primary-700/40 cursor-pointer"
      onClick={toggle}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          toggle();
        }
      }}
      role="button"
      tabIndex={0}
      aria-expanded={open}
    >
      {/* Resting state — tag + title */}
      <div className="absolute inset-0 p-8 flex flex-col justify-between">
        <span className="text-xs font-mono text-primary-500/70 uppercase tracking-wider">
          {feat.tag}
        </span>

        <div>
          <h3 className="text-2xl md:text-3xl font-bold text-white font-mono leading-tight">
            {feat.title}
          </h3>
          <span className="inline-block mt-3 text-xs font-mono text-surface-500 transition-opacity duration-200 group-hover:text-surface-300">
            {">"} details_
          </span>
        </div>
      </div>

      {/* Overlay — fades in on hover (desktop) or tap (mobile) */}
      <div
        className={`absolute inset-0 bg-surface-900/95 backdrop-blur-sm p-8 flex flex-col justify-end transition-opacity duration-200 ${
          open ? "opacity-100" : "opacity-0 pointer-events-none md:group-hover:opacity-100 md:group-hover:pointer-events-auto"
        }`}
      >
        <span className="text-xs font-mono text-primary-500 uppercase tracking-wider mb-3">
          {feat.tag}
        </span>
        <h3 className="text-xl font-bold text-white font-mono leading-tight">
          {feat.title}
        </h3>
        <p className="mt-3 text-sm text-surface-300 font-mono leading-relaxed">
          {feat.description}
        </p>
        <span className="inline-block mt-4 text-xs font-mono text-surface-500">
          {">"} close_
        </span>
      </div>
    </div>
  );
}
