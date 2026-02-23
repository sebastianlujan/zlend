export function PipelineConnector({
  orientation = "horizontal",
  color = "primary",
  className = "",
}: {
  orientation?: "horizontal" | "vertical";
  color?: "primary" | "accent";
  className?: string;
}) {
  const strokeColor =
    color === "primary" ? "rgba(232, 65, 66, 0.4)" : "rgba(5, 138, 255, 0.4)";
  const particleFill =
    color === "primary" ? "#E84142" : "#058AFF";

  if (orientation === "vertical") {
    return (
      <div className={`flex justify-center ${className}`} data-pipeline-connector>
        <svg
          width="2"
          height="60"
          viewBox="0 0 2 60"
          className="pipeline-connector overflow-visible"
        >
          <path
            d="M1 0 L1 60"
            stroke={strokeColor}
            strokeWidth="1.5"
            strokeDasharray="4 4"
            fill="none"
            data-connector-path
          />
          <circle
            r="3"
            cx="1"
            cy="0"
            fill={particleFill}
            opacity="0"
            className="pipeline-particle"
            data-connector-particle
          />
        </svg>
      </div>
    );
  }

  return (
    <div className={`flex-1 flex items-center min-w-[40px] ${className}`} data-pipeline-connector>
      <svg
        width="100%"
        height="2"
        viewBox="0 0 100 2"
        preserveAspectRatio="none"
        className="pipeline-connector overflow-visible"
      >
        <path
          d="M0 1 L100 1"
          stroke={strokeColor}
          strokeWidth="1.5"
          strokeDasharray="4 4"
          fill="none"
          data-connector-path
        />
        <circle
          r="3"
          cx="0"
          cy="1"
          fill={particleFill}
          opacity="0"
          className="pipeline-particle"
          data-connector-particle
        />
      </svg>
    </div>
  );
}
