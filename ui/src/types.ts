export type PlayerId = "a" | "b";
export type Mode = "hotseat" | "vs-bot" | "watch";
export type First = "coin" | "a" | "b";

export type NeutralAction =
  | { mulligan: { player: PlayerId; swap: [boolean, boolean, boolean, boolean] } }
  | { play: { player: PlayerId; hand_pos: number; card: string } }
  | {
      attack: {
        player: PlayerId;
        attacker_slot: number;
        target: { slot: number } | "leader";
      };
    }
  | { evolve: { player: PlayerId; slot: number; super: boolean } }
  | { engage: { player: PlayerId; slot: number } }
  | { fuse: { player: PlayerId; host_pos: number; partner_pos: number[] } }
  | { bonus_pp: { player: PlayerId } }
  | { choose: { player: PlayerId; option: ChooseOption } }
  | { confirm: { player: PlayerId } }
  | { end_turn: { player: PlayerId } };

export type ChooseOption =
  | { card: string }
  | { slot: number; player?: PlayerId }
  | "leader"
  | { mode: number };

export type CatalogEntry = {
  name: string;
  cost: number | null;
  class: string | null;
  kind: string | null;
  attack: number | null;
  defense: number | null;
  card: string;
  banner: string;
  evoCard: string;
  evoBanner: string;
  specificEffects: string[];
  specificEffectTypes: string[];
};

export type CardText = {
  id: string;
  name: string;
  text: string;
  kind: string;
  cost: number | null;
  attack?: number | null;
  defense?: number | null;
  modes?: string[];
};

export type CardInstance = {
  id: number;
  card: string;
  name: string;
  kind: string;
  class: string;
  cost: number;
  base_cost: number;
  attack: number;
  defense: number;
  max_defense: number;
  evolved: boolean;
  super_evolved: boolean;
  traits: string[];
  printed_tags: string[];
  granted: unknown;
  flags: {
    was_fused: boolean;
    fused_kinds: string[];
    ambush_active: boolean;
    summoning_sick: boolean;
    attacked_this_turn: boolean;
    attacks_left: number;
    engaged_this_turn: boolean;
    fused_this_turn: boolean;
    enhanced: boolean;
  };
  vars: Record<string, number>;
  skybound: number;
  countdown: number | null;
  spellboost_count: number;
  tribes: string[];
};

export type CrestInstance = {
  id: string;
  countdown: number | null;
  faith: boolean;
  granted_order: number;
};

export type BonusPp = {
  early_charge: boolean;
  late_charge: boolean;
  active: boolean;
  locked: boolean;
};

export type GateKind =
  | "enhance"
  | "accelerate"
  | "crystallize"
  | "necromancy"
  | "rally"
  | "combo"
  | "earth_rite"
  | "overflow"
  | "spellboost";

export type GateInfo = {
  kind: GateKind | string;
  label?: string;
  need: number;
  have: number;
  met: boolean;
};

export type HandCardInfo = {
  pos: number;
  id: string;
  base_cost: number;
  cost: number | null;
  form: "normal" | "enhance" | "accelerate" | "crystallize" | null;
  playable: boolean;
  gates: GateInfo[];
};

export type BoardCardInfo = {
  slot: number;
  id: string;
  can_attack: boolean;
  can_attack_leader: boolean;
  rush_only: boolean;
  evolved: boolean;
  super_evolved: boolean;
  gates: GateInfo[];
};

export type PlayerState = {
  leader_defense: number;
  leader_max: number;
  pp: number;
  pp_max: number;
  bonus_pp: BonusPp;
  ep: number;
  sep: number;
  evolves_used: number;
  evolved_this_turn: boolean;
  shadows: number;
  combo: number;
  earth: number;
  faith: number;
  rally: number;
  crests: CrestInstance[];
  hand: CardInstance[];
  field: Array<CardInstance | null>;
  deck: CardInstance[];
  cemetery: CardInstance[];
  banished: CardInstance[];
  destroyed_history: Array<{ card: string; from_field: boolean }>;
  played_this_turn: string[];
  turns_taken: number;
  is_second: boolean;
  leader_mods: Array<{ damage_cap?: number | null }>;
};

export type TargetOpt =
  | { slot: number; player: PlayerId }
  | { leader: PlayerId }
  | { hand: { player: PlayerId; pos: number } }
  | { deck: { player: PlayerId; id: number } }
  | { card: string }
  | { mode: number };

export type ChoiceNode =
  | { targets: { options: TargetOpt[]; remaining: number } }
  | { modes: { options: number[]; picked: number[]; remaining: number } }
  | { cards: { options: string[]; remaining: number } }
  | { fuse_partners: { host: number; options: number[]; picked: number[] } }
  | { multi_pick: { options: TargetOpt[]; picked: number[]; remaining: number } };

export type PhaseFull =
  | "main"
  | "combat"
  | "end"
  | "terminal"
  | { mulligan: { player: PlayerId } }
  | { choice: { player: PlayerId; node: ChoiceNode } };

export type FullState = {
  players: { a: PlayerState; b: PlayerState };
  turn: number;
  active: PlayerId;
  first: PlayerId;
  phase: PhaseFull;
  winner: PlayerId | null;
};

export type EngineEvent = Record<string, unknown>;

export type DeckManifestEntry = {
  file: string;
  id: string;
  label: string;
  category: string;
};

export type PositionLog = {
  v: 1;
  kind: "replay-log";
  seed: string;
  deckA: Record<string, number>;
  deckB: Record<string, number>;
  deckAId: string;
  deckBId: string;
  first: First;
  actions: NeutralAction[];
};

export type SessionConfig = {
  seed: bigint;
  deckA: Record<string, number>;
  deckB: Record<string, number>;
  deckAId: string;
  deckBId: string;
  first: First;
  mode: Mode;
  humanSide: PlayerId;
  hideBotHand: boolean;
  policyA: string;
  policyB: string;
};
