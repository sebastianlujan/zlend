import { Check } from "lucide-react";
import { LIFECYCLE_STEPS, STATUS_ORDER, type VaultStatus } from "../../types/vault.ts";

interface StepperProps {
  currentStatus: VaultStatus;
  className?: string;
}

export function Stepper({ currentStatus, className = "" }: StepperProps) {
  const currentIdx = STATUS_ORDER[currentStatus] ?? 0;

  return (
    <div className={`flex items-center gap-0 ${className}`}>
      {LIFECYCLE_STEPS.map((step, i) => {
        const isCompleted = i < currentIdx;
        const isActive = i === currentIdx;

        return (
          <div key={step.key} className="flex flex-1 items-center">
            <div className="flex flex-col items-center gap-1.5">
              <div
                className={`flex h-8 w-8 items-center justify-center rounded-full border-2 transition-all duration-300 ${
                  isCompleted
                    ? "border-primary-500 bg-primary-500 text-white"
                    : isActive
                      ? "border-primary-400 bg-primary-900/50 text-primary-400 shadow-[0_0_12px_rgba(255,57,74,0.3)]"
                      : "border-surface-600 bg-surface-800 text-surface-500"
                }`}
              >
                {isCompleted ? (
                  <Check size={14} strokeWidth={3} />
                ) : (
                  <span className="text-xs font-bold">{i + 1}</span>
                )}
              </div>
              <span
                className={`text-[10px] font-medium tracking-wide ${
                  isCompleted
                    ? "text-primary-400"
                    : isActive
                      ? "text-white"
                      : "text-surface-500"
                }`}
              >
                {step.label}
              </span>
            </div>

            {i < LIFECYCLE_STEPS.length - 1 && (
              <div className="mx-1 mb-5 h-0.5 flex-1">
                <div
                  className={`h-full rounded-full transition-colors duration-300 ${
                    i < currentIdx ? "bg-primary-500" : "bg-surface-700"
                  }`}
                />
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

interface MiniStepperProps {
  currentStatus: VaultStatus;
  className?: string;
}

export function MiniStepper({ currentStatus, className = "" }: MiniStepperProps) {
  const currentIdx = STATUS_ORDER[currentStatus] ?? 0;

  return (
    <div className={`flex items-center gap-1 ${className}`}>
      {LIFECYCLE_STEPS.map((step, i) => {
        const isCompleted = i < currentIdx;
        const isActive = i === currentIdx;

        return (
          <div key={step.key} className="flex items-center">
            <div
              className={`h-1.5 w-1.5 rounded-full transition-colors duration-200 ${
                isCompleted
                  ? "bg-primary-500"
                  : isActive
                    ? "bg-primary-400 shadow-[0_0_4px_rgba(255,57,74,0.5)]"
                    : "bg-surface-600"
              }`}
              title={step.label}
            />
            {i < LIFECYCLE_STEPS.length - 1 && (
              <div
                className={`mx-0.5 h-px w-2 ${
                  i < currentIdx ? "bg-primary-500/60" : "bg-surface-700"
                }`}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}
