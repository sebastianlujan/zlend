import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

const features = [
  {
    title: "Noir + Ultrahonk",
    description: "ZK proofs generated client-side in Noir, verified on-chain by the Ultrahonk proving system from Aztec's Barretenberg.",
    tag: "ZK Proofs",
  },
  {
    title: "Aave V3 Integration",
    description: "No forked lending pool. ZLend plugs directly into Aave V3's existing deployment on Avalanche.",
    tag: "Infrastructure",
  },
  {
    title: "ZIP-32 Key Derivation",
    description: "Deterministic address derivation from ZCash's hierarchical wallet standard. Same inputs always produce the same ZLend Unit.",
    tag: "Cryptography",
  },
  {
    title: "Nullifier Protection",
    description: "Each borrow cycle creates a unique nullifier. Prevents replay attacks and double-collateralization across cycles.",
    tag: "Security",
  },
];

export function Features() {
  const ref = useScrollAnimation<HTMLDivElement>({ childSelector: "[data-animate]" });

  return (
    <SectionWrapper>
      <div ref={ref}>
        <div className="text-center mb-16" data-animate>
          <Badge>Technical Foundation</Badge>
          <h2 className="mt-4 text-4xl md:text-5xl font-bold text-white">
            Built on Proven Primitives
          </h2>
        </div>

        <div className="grid md:grid-cols-2 gap-8">
          {features.map((feat) => (
            <div
              key={feat.title}
              className="flex gap-4 p-6 rounded-xl border border-surface-800/50 hover:border-primary-700/30 transition-colors"
              data-animate
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
          ))}
        </div>
      </div>
    </SectionWrapper>
  );
}
