import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function TrustCompact() {
  const { trust } = content;
  const ref = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    animation: "fadeUp",
  });

  return (
    <SectionWrapper id="trust-compact">
      <div ref={ref}>
        <div className="text-center mb-12" data-animate>
          <Badge>{trust.sectionLabel}</Badge>
          <h2 className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            {trust.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto">
            {trust.description}
          </p>
        </div>

        <div className="max-w-3xl mx-auto" data-animate>
          <h3 className="text-lg font-semibold text-white mb-4 text-center font-mono">
            {"// privacy_summary"}
          </h3>
          <div className="overflow-x-auto rounded-xl border border-surface-700/50 bg-surface-900/50 backdrop-blur-sm">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="border-b border-surface-700">
                  <th className="py-3 px-5 text-sm font-medium text-surface-400 font-mono">
                    Data
                  </th>
                  <th className="py-3 px-5 text-sm font-medium text-surface-400 font-mono">
                    Visibility
                  </th>
                </tr>
              </thead>
              <tbody>
                {trust.privacyTable.map((row) => (
                  <tr
                    key={row.data}
                    className="border-b border-surface-800/50 hover:bg-surface-800/30 transition-colors duration-200"
                  >
                    <td className="py-3 px-5 text-surface-200 font-mono text-sm">
                      {row.data}
                    </td>
                    <td className="py-3 px-5 text-surface-400 text-sm">
                      {row.visibility}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </SectionWrapper>
  );
}
