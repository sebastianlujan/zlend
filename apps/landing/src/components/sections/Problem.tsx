import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function Problem() {
  const { problem } = content;
  const introRef = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
  });
  const glassHouseRef = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    staggerDelay: 120,
  });
  const idleCapitalRef = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
  });
  const { glassHouse, idleCapital } = problem;
  const positionRows = [
    { label: "Wallet", value: glassHouse.position.wallet },
    { label: "Collateral", value: glassHouse.position.collateral },
    { label: "Debt", value: glassHouse.position.debt },
    { label: "Health Factor", value: glassHouse.position.health },
    { label: "Liquidation", value: glassHouse.position.liquidation },
  ];

  return (
    <SectionWrapper id="problem">
      {/* Intro */}
      <div ref={introRef} className="text-center mb-24" data-animate>
        <Badge>{problem.sectionLabel}</Badge>
        <h2 className="mt-4 text-4xl md:text-5xl font-bold text-white">
          {problem.title}
        </h2>
        <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto leading-relaxed">
          {problem.subtitle}
        </p>
      </div>

      {/* Scene 1: How DeFi lending works */}
      <div ref={glassHouseRef} className="mb-32">
        <h3
          className="text-2xl md:text-3xl font-bold text-white mb-8 text-center"
          data-animate
        >
          {glassHouse.title}
        </h3>

        <div
          className="mx-auto max-w-md rounded-xl border border-surface-700/50 bg-surface-900/80 p-6"
          data-animate
        >
          <div className="flex items-center justify-between mb-5 pb-3 border-b border-surface-700/50">
            <span className="text-xs font-mono uppercase tracking-wider text-surface-500">
              Lending Position
            </span>
            <span className="text-[10px] font-mono text-surface-600">
              #48291
            </span>
          </div>

          <div className="space-y-3">
            {positionRows.map((row) => (
              <div
                key={row.label}
                className="flex justify-between items-center"
                data-animate
              >
                <span className="text-sm text-surface-500">{row.label}</span>
                <span className="text-sm font-mono text-white">
                  {row.value}
                </span>
              </div>
            ))}
          </div>

          <div
            className="mt-5 pt-3 border-t border-surface-700/50 flex items-center gap-2"
            data-animate
          >
            <svg
              className="w-3.5 h-3.5 text-surface-500 shrink-0"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.64 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.64 0-8.573-3.007-9.963-7.178z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
            </svg>
            <span className="text-xs text-surface-500">
              {glassHouse.note}
            </span>
          </div>
        </div>

        <div className="mt-10 max-w-lg mx-auto space-y-3">
          {glassHouse.observations.map((o) => (
            <p
              key={o}
              className="text-surface-400 text-sm leading-relaxed pl-5 border-l-2 border-surface-700/30"
              data-animate
            >
              {o}
            </p>
          ))}
        </div>
      </div>

      {/* Scene 2: Idle Capital */}
      <div ref={idleCapitalRef}>
        <h3
          className="text-2xl md:text-3xl font-bold text-white mb-8 text-center"
          data-animate
        >
          {idleCapital.title}
        </h3>

        <div className="grid md:grid-cols-2 gap-6 max-w-2xl mx-auto">
          {/* ETH side */}
          <div
            className="rounded-xl border border-primary-500/20 bg-surface-900/80 p-6"
            data-animate
          >
            <div className="text-xs font-mono uppercase tracking-wider text-primary-400 mb-4">
              {idleCapital.earning.label}
            </div>
            <div className="text-4xl font-bold font-mono text-white mb-4">
              {idleCapital.earning.apy}
              <span className="text-lg text-surface-400 ml-1">% APY</span>
            </div>
            <ul className="space-y-2">
              {idleCapital.earning.items.map((item) => (
                <li
                  key={item}
                  className="flex items-center gap-2 text-sm text-surface-300"
                >
                  <span className="w-1 h-1 rounded-full bg-primary-400 shrink-0" />
                  {item}
                </li>
              ))}
            </ul>
          </div>

          {/* ZEC side */}
          <div
            className="rounded-xl border border-surface-700/50 bg-surface-900/50 p-6"
            data-animate
          >
            <div className="text-xs font-mono uppercase tracking-wider text-surface-500 mb-4">
              {idleCapital.idle.label}
            </div>
            <div className="text-4xl font-bold font-mono text-surface-600 mb-4">
              {idleCapital.idle.apy}
              <span className="text-lg text-surface-600 ml-1">% APY</span>
            </div>
            <ul className="space-y-2">
              {idleCapital.idle.items.map((item) => (
                <li
                  key={item}
                  className="flex items-center gap-2 text-sm text-surface-500"
                >
                  <span className="w-1 h-1 rounded-full bg-surface-600 shrink-0" />
                  {item}
                </li>
              ))}
            </ul>
          </div>
        </div>

        <p
          className="mt-10 text-center text-lg text-surface-300 italic max-w-xl mx-auto"
          data-animate
        >
          {idleCapital.stat}
        </p>
      </div>
    </SectionWrapper>
  );
}
