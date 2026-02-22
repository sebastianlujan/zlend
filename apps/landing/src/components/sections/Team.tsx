import { content } from "../../data/content";
import { Badge } from "../ui/Badge";
import { Card } from "../ui/Card";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function Team() {
  const { team } = content;
  const ref = useScrollAnimation<HTMLDivElement>({ childSelector: "[data-animate]" });

  return (
    <SectionWrapper id="team" className="bg-surface-900/30">
      <div ref={ref}>
        <div className="text-center mb-16" data-animate>
          <Badge>{team.sectionLabel}</Badge>
          <h2 className="mt-4 text-4xl md:text-5xl font-bold text-white">
            {team.title}
          </h2>
          <p className="mt-4 text-lg text-surface-400 max-w-2xl mx-auto">
            {team.description}
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6 max-w-2xl mx-auto">
          {team.members.map((member) => (
            <Card key={member.name}>
              <div data-animate>
                <div className="w-14 h-14 rounded-full bg-primary-900/50 border border-primary-700/30 flex items-center justify-center mb-4">
                  <span className="text-xl font-bold text-primary-300">
                    {member.name[0]}
                  </span>
                </div>
                <h3 className="text-xl font-bold text-white">{member.name}</h3>
                <p className="text-sm text-primary-400 mt-1">{member.role}</p>
                <p className="text-surface-400 text-sm mt-3 leading-relaxed">
                  {member.focus}
                </p>
              </div>
            </Card>
          ))}
        </div>
      </div>
    </SectionWrapper>
  );
}
