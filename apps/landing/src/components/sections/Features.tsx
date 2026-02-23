import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";
import { useSplitTextHover } from "../../hooks/useSplitTextHover";

const features = [
  {
    title: "Noir + Ultrahonk",
    description: "ZK proofs generated client-side in Noir, verified on-chain by the Ultrahonk proving system from Aztec's Barretenberg.",
    tag: "ZK Proofs",
    animation: "slideLeft" as const,
  },
  {
    title: "Aave V3 Integration",
    description: "No forked lending pool. OGBank plugs directly into Aave V3's existing deployment on Avalanche.",
    tag: "Infrastructure",
    animation: "slideRight" as const,
  },
  {
    title: "ZIP-32 Key Derivation",
    description: "Deterministic address derivation from ZCash's hierarchical wallet standard. Same inputs always produce the same OGBank Unit.",
    tag: "Cryptography",
    animation: "slideLeft" as const,
  },
  {
    title: "Nullifier Protection",
    description: "Each lock cycle creates a unique nullifier. Prevents replay attacks and double-collateralization across cycles.",
    tag: "Security",
    animation: "slideRight" as const,
  },
];

export function Features() {
  const headingRef = useSplitTextHover<HTMLHeadingElement>();

  return (
    <SectionWrapper>
      <div>
        <div className="text-center mb-16">
          <Badge>Technical Foundation</Badge>
          <h2 ref={headingRef} className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            Built on Proven Primitives
          </h2>
        </div>

        <div className="grid md:grid-cols-2 gap-8">
          {features.map((feat) => (
            <FeatureItem key={feat.title} feat={feat} />
          ))}
        </div>
      </div>
    </SectionWrapper>
  );
}

function FeatureItem({ feat }: { feat: (typeof features)[number] }) {
  const ref = useScrollAnimation<HTMLDivElement>({
    animation: feat.animation,
  });

  return (
    <div
      ref={ref}
      className="flex gap-4 p-6 rounded-xl border border-surface-800/50 hover:border-primary-700/40 hover:-translate-y-1 hover:shadow-lg hover:shadow-primary-900/10 transition-all duration-300"
    >
      <div>
        <span className="text-xs font-mono text-primary-500 uppercase tracking-wider">
          {feat.tag}
        </span>
        <h3 className="mt-1 text-xl font-semibold text-white">
          {feat.title}
        </h3>
        <p className="mt-2 text-surface-400 text-sm leading-relaxed">
          {feat.description}
        </p>
      </div>
    </div>
  );
}
