/** Visible bot-policy labels (values unchanged). Shared by UI and Playwright tests. */

export const POLICY_STRONG_H0 = "h0:nodes=6000";
export const POLICY_CHEAT_H0 = "h0:nodes=6000,info=all";
export const CUSTOM_SPEC_MENU_VALUE = "__custom_spec__";

export function formatThousands(n: number): string {
  return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, " ");
}

export function fullStrengthPolicyLabel(spec: string, seesHand: boolean): string {
  const m = spec.match(/nodes=(\d+)/);
  const base = seesHand ? "Full strength, sees your hand" : "Full strength";
  if (!m) return base;
  return `${base} — ${formatThousands(Number(m[1]))} nodes`;
}

export function servedFullStrengthTitle(spec: string): string {
  return `The full-strength bot from py/serve.py: ${spec}. A PC's local server may run another version; its badge shows which.`;
}

export function policyOptionLabel(
  value: string,
  servedStrong: string,
  servedCheat: string,
): string {
  switch (value) {
    case "random":
      return "Random moves";
    case "first-legal":
      return "First legal move";
    case "h0":
      return "Quick — 2 000 nodes";
    case POLICY_STRONG_H0:
      return "Medium — 6 000 nodes";
    case POLICY_CHEAT_H0:
      return "Medium, sees your hand — 6 000 nodes";
    case servedStrong:
      return fullStrengthPolicyLabel(servedStrong, false);
    case servedCheat:
      return fullStrengthPolicyLabel(servedCheat, true);
    case CUSTOM_SPEC_MENU_VALUE:
      return "Custom spec…";
    default:
      if (value.startsWith("h0:")) return `Custom: ${value}`;
      return value;
  }
}
