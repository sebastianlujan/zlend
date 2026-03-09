import { useState, useCallback } from "react";
import {
  generateBorrowProof,
  generateAuthProof,
  formatProofForSolidity,
  formatPublicInputsForSolidity,
  type BorrowProofInputs,
  type AuthProofInputs,
} from "../lib/noir.ts";
import type { Hex } from "viem";

interface FormattedProof {
  proof: Hex;
  publicInputs: Hex[];
}

export function useProofGeneration() {
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const generateBorrow = useCallback(
    async (inputs: BorrowProofInputs): Promise<FormattedProof | null> => {
      setIsGenerating(true);
      setError(null);
      try {
        const { proof, publicInputs } = await generateBorrowProof(inputs);
        return {
          proof: formatProofForSolidity(proof),
          publicInputs: formatPublicInputsForSolidity(publicInputs),
        };
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        console.error("Borrow proof generation failed:", e);
        return null;
      } finally {
        setIsGenerating(false);
      }
    },
    [],
  );

  const generateAuth = useCallback(
    async (inputs: AuthProofInputs): Promise<FormattedProof | null> => {
      setIsGenerating(true);
      setError(null);
      try {
        const { proof, publicInputs } = await generateAuthProof(inputs);
        return {
          proof: formatProofForSolidity(proof),
          publicInputs: formatPublicInputsForSolidity(publicInputs),
        };
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        console.error("Auth proof generation failed:", e);
        return null;
      } finally {
        setIsGenerating(false);
      }
    },
    [],
  );

  return { generateBorrow, generateAuth, isGenerating, error };
}
