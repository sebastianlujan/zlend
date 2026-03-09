import { useAccount } from "wagmi";
import { Card } from "../ui/Card.tsx";
import { Badge } from "../ui/Badge.tsx";
import { Button } from "../ui/Button.tsx";
import { Spinner } from "../ui/Spinner.tsx";
import { Alert } from "../ui/Alert.tsx";
import { VAULT_STATUS, type Vault } from "../../types/vault.ts";
import { useProofGeneration } from "../../hooks/useProofGeneration.ts";
import type { Hex } from "viem";

interface ProofGenerationCardProps {
  vault: Vault;
  onProofGenerated: (proofData: { proof: Hex; publicInputs: Hex[] }) => void;
}

export function ProofGenerationCard({
  vault,
  onProofGenerated,
}: ProofGenerationCardProps) {
  const { address } = useAccount();
  const { generateBorrow, isGenerating, error } = useProofGeneration();

  const isReady = vault.status === VAULT_STATUS.PROOF_READY;
  const hasSecrets = !!vault.userSecret && !!vault.nonce;

  const handleGenerate = async () => {
    if (!vault.userSecret || !vault.nonce || !address) return;

    const zecSatoshis = Math.floor(
      parseFloat(vault.zecAmount) * 1e8,
    ).toString();

    const result = await generateBorrow({
      userSecret: vault.userSecret,
      value: zecSatoshis,
      nonce: vault.nonce,
      threshold: zecSatoshis,
      recipient: address,
    });

    if (result) {
      onProofGenerated(result);
    }
  };

  return (
    <Card>
      <div className="flex items-center gap-2">
        <p className="text-xs font-medium uppercase tracking-wider text-surface-400">
          Generate ZK Proof
        </p>
        <Badge variant="info">Real Proof</Badge>
      </div>

      <div className="mt-4 flex items-center gap-3">
        <span className="text-sm text-surface-300">
          Status:{" "}
          {isReady ? (
            <span className="font-medium text-green-400">Ready</span>
          ) : (
            <span className="text-surface-500">Not generated</span>
          )}
        </span>
      </div>

      {!isReady && (
        <>
          {!hasSecrets && (
            <div className="mt-3">
              <Alert variant="warning">
                This vault was created before automatic secret generation.
                Please create a new vault to generate proofs.
              </Alert>
            </div>
          )}

          {error && (
            <div className="mt-3">
              <Alert variant="danger">{error}</Alert>
            </div>
          )}

          {hasSecrets && (
            <div className="mt-4">
              <Button
                size="sm"
                onClick={handleGenerate}
                disabled={isGenerating}
              >
                {isGenerating && <Spinner size="sm" />}
                <span className={isGenerating ? "ml-2" : ""}>
                  {isGenerating ? "Generating proof…" : "Generate ZK Proof"}
                </span>
              </Button>
            </div>
          )}
        </>
      )}
    </Card>
  );
}
