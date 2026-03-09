import { Barretenberg, UltraHonkBackend } from "@aztec/bb.js";
import { Noir } from "@noir-lang/noir_js";
import initNoirC from "@noir-lang/noirc_abi";
import initACVM from "@noir-lang/acvm_js";
import acvm from "@noir-lang/acvm_js/web/acvm_js_bg.wasm?url";
import noirc from "@noir-lang/noirc_abi/web/noirc_abi_wasm_bg.wasm?url";
import circuit from "../../../../packages/circuits/target/circuits.json";
import type { Hex } from "viem";

// --- WASM & Barretenberg singleton ---

let wasmInitialized = false;

async function initWasm(): Promise<void> {
  if (wasmInitialized) {
    console.log("[noir] WASM already initialized, skipping");
    return;
  }
  console.log("[noir] Initializing WASM modules (ACVM + noirc_abi)...");
  console.time("[noir] WASM init");
  await Promise.all([initACVM(fetch(acvm)), initNoirC(fetch(noirc))]);
  wasmInitialized = true;
  console.timeEnd("[noir] WASM init");
}

let bbInstance: Barretenberg | null = null;

async function getBarretenberg(): Promise<Barretenberg> {
  if (bbInstance) {
    console.log("[noir] Barretenberg already initialized, reusing");
    return bbInstance;
  }
  console.log("[noir] Creating Barretenberg instance (threads: 1)...");
  console.time("[noir] Barretenberg.new()");
  bbInstance = await Barretenberg.new();
  console.timeEnd("[noir] Barretenberg.new()");
  return bbInstance;
}

// --- Field element helpers ---

function hexToBytes(hex: string): Uint8Array {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  const padded = clean.padStart(64, "0");
  const bytes = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    bytes[i] = parseInt(padded.substring(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

function u64ToFieldBytes(value: bigint): Uint8Array {
  const bytes = new Uint8Array(32);
  let v = value;
  for (let i = 31; i >= 24; i--) {
    bytes[i] = Number(v & 0xffn);
    v >>= 8n;
  }
  return bytes;
}

function bytesToHex(bytes: Uint8Array): string {
  return (
    "0x" +
    Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("")
  );
}

// --- Poseidon2 hash (matches Noir's Poseidon2::hash sponge construction) ---

async function poseidon2Hash(inputs: Uint8Array[]): Promise<Uint8Array> {
  console.log(
    "[noir] poseidon2Hash inputs:",
    inputs.map((b) => bytesToHex(b)),
  );
  const api = await getBarretenberg();
  const { hash } = await api.poseidon2Hash({ inputs });
  console.log("[noir] poseidon2Hash result:", bytesToHex(hash));
  return hash;
}

// --- Public types ---

export interface BorrowProofInputs {
  userSecret: string; // hex (64 chars, no 0x prefix)
  value: string; // decimal string (satoshis)
  nonce: string; // hex (64 chars, no 0x prefix)
  threshold: string; // decimal string
  recipient: string; // ethereum address
}

export interface AuthProofInputs {
  userSecret: string;
  nonce: string;
  borrowNullifier: string; // hex from vault
  recipient: string;
}

export interface ProofResult {
  proof: Uint8Array;
  publicInputs: string[];
}

// --- Proof generation ---

export async function generateBorrowProof(
  inputs: BorrowProofInputs,
): Promise<ProofResult> {
  console.group("[noir] generateBorrowProof (mode 0)");
  console.log("[noir] Raw inputs:", {
    userSecret: inputs.userSecret,
    value: inputs.value,
    nonce: inputs.nonce,
    threshold: inputs.threshold,
    recipient: inputs.recipient,
  });

  console.log("[noir] Step 1: Init WASM");
  await initWasm();

  console.log("[noir] Step 2: Convert inputs to field bytes");
  const secretBytes = hexToBytes(inputs.userSecret);
  const nonceBytes = hexToBytes(inputs.nonce);
  const valueBytes = u64ToFieldBytes(BigInt(inputs.value));
  console.log("[noir] secretBytes:", bytesToHex(secretBytes));
  console.log("[noir] nonceBytes:", bytesToHex(nonceBytes));
  console.log("[noir] valueBytes:", bytesToHex(valueBytes));

  console.log("[noir] Step 3: Compute Poseidon2 hashes");
  console.log("[noir] Computing commitment_hash = H(secret, value, nonce)...");
  const commitmentHash = bytesToHex(
    await poseidon2Hash([secretBytes, valueBytes, nonceBytes]),
  );
  console.log("[noir] commitment_hash:", commitmentHash);

  console.log("[noir] Computing nullifier = H(secret, nonce)...");
  const nullifier = bytesToHex(
    await poseidon2Hash([secretBytes, nonceBytes]),
  );
  console.log("[noir] nullifier:", nullifier);

  const userSecretField = "0x" + inputs.userSecret;
  const nonceField = "0x" + inputs.nonce;

  const circuitInputs = {
    user_secret: userSecretField,
    value: inputs.value,
    nonce: nonceField,
    mode: "0",
    commitment_hash: commitmentHash,
    nullifier: nullifier,
    threshold: inputs.threshold,
    borrow_nullifier: "0",
    repay_nullifier: "0",
    recipient: inputs.recipient,
  };
  console.log("[noir] Step 4: Execute Noir circuit with inputs:", {
    ...circuitInputs,
  });

  const noir = new Noir(circuit as never);
  console.time("[noir] noir.execute()");
  const { witness } = await noir.execute(circuitInputs);
  console.timeEnd("[noir] noir.execute()");
  console.log("[noir] Witness generated, size:", witness.length, "bytes");

  console.log("[noir] Step 5: Generate UltraHonk proof (verifierTarget: evm)");
  const api = await getBarretenberg();
  const backend = new UltraHonkBackend(circuit.bytecode, api);
  console.time("[noir] backend.generateProof()");
  const { proof, publicInputs } = await backend.generateProof(witness, {
    verifierTarget: "evm",
  });
  console.timeEnd("[noir] backend.generateProof()");
  console.log("[noir] Proof generated:", {
    proofSize: proof.length,
    publicInputsCount: publicInputs.length,
    publicInputs,
  });
  console.groupEnd();

  return { proof, publicInputs };
}

export async function generateAuthProof(
  inputs: AuthProofInputs,
): Promise<ProofResult> {
  console.group("[noir] generateAuthProof (mode 1)");
  console.log("[noir] Raw inputs:", {
    userSecret: inputs.userSecret,
    nonce: inputs.nonce,
    borrowNullifier: inputs.borrowNullifier,
    recipient: inputs.recipient,
  });

  console.log("[noir] Step 1: Init WASM");
  await initWasm();

  console.log("[noir] Step 2: Convert inputs to field bytes");
  const secretBytes = hexToBytes(inputs.userSecret);
  const borrowNullifierBytes = hexToBytes(inputs.borrowNullifier);

  console.log("[noir] Step 3: Compute repay_nullifier = H(secret, borrow_nullifier)");
  const repayNullifier = bytesToHex(
    await poseidon2Hash([secretBytes, borrowNullifierBytes]),
  );
  console.log("[noir] repay_nullifier:", repayNullifier);

  const userSecretField = "0x" + inputs.userSecret;
  const nonceField = "0x" + inputs.nonce;

  const circuitInputs = {
    user_secret: userSecretField,
    value: "0",
    nonce: nonceField,
    mode: "1",
    commitment_hash: "0",
    nullifier: "0",
    threshold: "0",
    borrow_nullifier: inputs.borrowNullifier,
    repay_nullifier: repayNullifier,
    recipient: inputs.recipient,
  };
  console.log("[noir] Step 4: Execute Noir circuit with inputs:", {
    ...circuitInputs,
  });

  const noir = new Noir(circuit as never);
  console.time("[noir] noir.execute()");
  const { witness } = await noir.execute(circuitInputs);
  console.timeEnd("[noir] noir.execute()");
  console.log("[noir] Witness generated, size:", witness.length, "bytes");

  console.log("[noir] Step 5: Generate UltraHonk proof (verifierTarget: evm)");
  const api = await getBarretenberg();
  const backend = new UltraHonkBackend(circuit.bytecode, api);
  console.time("[noir] backend.generateProof()");
  const { proof, publicInputs } = await backend.generateProof(witness, {
    verifierTarget: "evm",
  });
  console.timeEnd("[noir] backend.generateProof()");
  console.log("[noir] Proof generated:", {
    proofSize: proof.length,
    publicInputsCount: publicInputs.length,
    publicInputs,
  });
  console.groupEnd();

  return { proof, publicInputs };
}

// --- Solidity formatting ---

export function formatProofForSolidity(proof: Uint8Array): Hex {
  return `0x${Array.from(proof)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("")}` as Hex;
}

export function formatPublicInputsForSolidity(
  publicInputs: string[],
): Hex[] {
  return publicInputs.map((pi) => {
    const hex = BigInt(pi).toString(16).padStart(64, "0");
    return `0x${hex}` as Hex;
  });
}
