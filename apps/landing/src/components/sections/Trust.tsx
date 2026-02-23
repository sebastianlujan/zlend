import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { Card } from "../ui/Card";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function Trust() {
  const { trust } = content;
  const ref = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    animation: "fadeUp",
  });
  return (
    <SectionWrapper>
      <div ref={ref}>
        <div className="text-center mb-16" data-animate>
          <Badge>{trust.sectionLabel}</Badge>
          <h2 className="section-heading mt-4 text-4xl md:text-5xl font-bold text-white">
            {trust.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto">
            {trust.description}
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6 mb-12">
          {trust.keys.map((key) => (
            <Card key={key.name} className="text-center">
              <div data-animate>
                <span className="text-xs font-mono uppercase tracking-wider text-primary-500">
                  {key.holder} holds this
                </span>
                <h3 className="mt-2 text-2xl font-bold text-white">
                  {key.name}
                </h3>
                <p className="mt-3 text-surface-400">{key.description}</p>
              </div>
            </Card>
          ))}
        </div>

        <div data-animate>
          <h3 className="text-xl font-semibold text-white mb-4 text-center">
            Privacy Summary
          </h3>
          <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="border-b border-surface-700">
                  <th className="py-3 px-4 text-sm font-medium text-surface-400">
                    Data
                  </th>
                  <th className="py-3 px-4 text-sm font-medium text-surface-400">
                    Who Can See It
                  </th>
                </tr>
              </thead>
              <tbody>
                {trust.privacyTable.map((row) => (
                  <tr
                    key={row.data}
                    className="border-b border-surface-800/50 hover:bg-surface-800/50 transition-colors duration-200 cursor-default"
                  >
                    <td className="py-3 px-4 text-surface-200">{row.data}</td>
                    <td className="py-3 px-4 text-surface-400">
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
