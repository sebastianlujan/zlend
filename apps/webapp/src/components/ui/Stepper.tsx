import { LIFECYCLE_STEPS, type VaultStatus } from "../../types/vault.ts";

interface StepperProps {
  currentStatus?: VaultStatus;
}

const STATUS_ORDER = LIFECYCLE_STEPS.map((s) => s.key);

export function Stepper({ currentStatus }: StepperProps) {
  const currentIndex = currentStatus
    ? STATUS_ORDER.indexOf(currentStatus)
    : -1;

  return (
    <div className="flex items-center gap-1">
      {LIFECYCLE_STEPS.map((step, i) => {
        const isCompleted = i < currentIndex;
        const isCurrent = i === currentIndex;
        const isLocked = i > currentIndex;

        return (
          <div key={step.key} className="flex items-center">
            {i > 0 && (
              <div
                className={`mx-1 h-px w-6 sm:w-10 ${
                  isCompleted ? "bg-primary-500" : "bg-surface-700 border-dashed border-t border-surface-600"
                }`}
              />
            )}
            <div className="flex flex-col items-center gap-1">
              <div
                className={`flex h-6 w-6 items-center justify-center rounded-full text-xs font-medium ${
                  isCompleted
                    ? "bg-primary-500 text-white"
                    : isCurrent
                      ? "border-2 border-primary-500 text-primary-400 animate-pulse"
                      : "border border-surface-600 text-surface-500"
                }`}
              >
                {isCompleted ? "\u2713" : i + 1}
              </div>
              <span
                className={`text-[10px] ${
                  isCurrent
                    ? "font-medium text-white"
                    : isLocked
                      ? "text-surface-500"
                      : "text-surface-400"
                }`}
              >
                {step.label}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
