//! Event output: unit ids, random picks, resolve markers, choice sources.

use arena_engine::event::{Event, EventSource, EventTarget};
use arena_engine::state::SourceRef;
use arena_engine::{
    apply, legal_actions, new_game, policy_rng, Action, First, GameConfig, GameRng, Illegal, Phase,
    Pick, PickChose, PickWhat, PlayerId,
};

mod common;
use common::*;

fn slot_id(st: &arena_engine::State, p: PlayerId, slot: u8) -> u32 {
    st.field_inst(p, slot).expect("field").id
}

#[test]
fn compaction_destroy_same_slot_different_ids() {
    let db = load_db();
    let mut st = started(&db, 42);
    let opp = PlayerId::B;
    let a0 = put_field(&db, &mut st, opp, "10011210");
    let a1 = put_field(&db, &mut st, opp, "10011210");
    let id0 = slot_id(&st, opp, a0);
    let id1 = slot_id(&st, opp, a1);
    if let Some(c) = st.field_inst_mut(opp, a0) {
        c.countdown = Some(1);
    }
    if let Some(c) = st.field_inst_mut(opp, a1) {
        c.countdown = Some(1);
    }
    let events = apply(&db, &mut st, Action::EndTurn).expect("A ends");
    let destroys: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Destroy {
                slot, player, id, ..
            } => Some((slot.0, *player, *id)),
            _ => None,
        })
        .collect();
    assert_eq!(destroys.len(), 2, "both amulets expire: {destroys:?}");
    assert_eq!(destroys[0].0, destroys[1].0, "same slot after compact");
    assert_ne!(destroys[0].2, destroys[1].2, "different instance ids");
    assert_eq!(destroys[0].1, PlayerId::B);
    assert_eq!(destroys[1].1, PlayerId::B);
    assert!(destroys.iter().any(|(_, _, id)| *id == id0));
    assert!(destroys.iter().any(|(_, _, id)| *id == id1));
}

#[test]
fn damage_ids_on_area_hit() {
    let db = load_db();
    let mut st = started(&db, 43);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let f0 = put_field(&db, &mut st, opp, "88001110");
    let f1 = put_field(&db, &mut st, opp, "88001320");
    let id0 = slot_id(&st, opp, f0);
    let id1 = slot_id(&st, opp, f1);
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10753310");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("split damage spell");
    let slot_damages: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                target: EventTarget::Slot(p, s),
                unit,
                ..
            } => Some((p, s.0, *unit)),
            _ => None,
        })
        .collect();
    assert!(
        slot_damages.len() >= 2,
        "hits multiple followers: {slot_damages:?}"
    );
    for (_, _, uid) in &slot_damages {
        assert!(uid.is_some());
    }
    let ids: Vec<u32> = slot_damages.iter().filter_map(|(_, _, u)| *u).collect();
    assert!(ids.contains(&id0));
    assert!(ids.contains(&id1));
    let leader_hits = events.iter().any(|e| {
        matches!(
            e,
            Event::Damage {
                target: EventTarget::Leader(_),
                unit: None,
                ..
            }
        )
    });
    assert!(!leader_hits, "split damage to followers only");
}

#[test]
fn random_pick_before_damage() {
    let db = load_db();
    let mut st = started(&db, 44);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10011210");
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001320");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "90011110");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("pixie enter");
    let picks: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::RandomPick { what, unit, .. } if *what == PickWhat::RandomTarget => Some(*unit),
            _ => None,
        })
        .collect();
    assert_eq!(picks.len(), 1);
    let dmg_id = events.iter().find_map(|e| match e {
        Event::Damage {
            target: EventTarget::Slot(_, _),
            unit,
            ..
        } => *unit,
        _ => None,
    });
    assert_eq!(picks[0], dmg_id);
}

#[test]
fn random_distinct_picks() {
    let db = load_db();
    let mut st = started(&db, 441);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    put_field(&db, &mut st, opp, "88001320");
    put_field(&db, &mut st, opp, "88001200");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10811110");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("random distinct destroy");
    let picks: Vec<u32> = events
        .iter()
        .filter_map(|e| match e {
            Event::RandomPick { what, unit, .. } if *what == PickWhat::RandomTarget => *unit,
            _ => None,
        })
        .collect();
    assert_eq!(picks.len(), 2);
    assert_ne!(picks[0], picks[1]);
}

#[test]
fn highest_unique_emits_no_random_pick() {
    let db = load_db();
    let mut st = started(&db, 45);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10503310");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("unique highest destroy");
    let picks = events
        .iter()
        .filter(|e| matches!(e, Event::RandomPick { .. }))
        .count();
    assert_eq!(
        picks, 0,
        "single highest candidate must not emit random_pick"
    );
    assert!(
        events.iter().any(|e| matches!(e, Event::Destroy { .. })),
        "destroy still happens"
    );
}

#[test]
fn highest_tie_emits_one_random_pick() {
    let db = load_db();
    let mut st = started(&db, 46);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001200");
    put_field(&db, &mut st, opp, "88001200");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10503310");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("highest tie destroy");
    let picks: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::RandomPick { what, .. } if *what == PickWhat::RandomTarget => Some(()),
            _ => None,
        })
        .collect();
    assert_eq!(
        picks.len(),
        1,
        "tied highest must emit exactly one random_pick"
    );
}

#[test]
fn random_pick_leader() {
    let db = load_db();
    let mut st = started(&db, 47);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001200");
    let slot = put_field(&db, &mut st, me, "10524110");
    if let Some(c) = st.field_inst_mut(me, slot) {
        c.evolved = true;
    }
    st.phase = Phase::Main;
    st.active = me;
    let scripted: Vec<Pick> = (0..8)
        .map(|_| Pick {
            what: PickWhat::RandomTarget,
            among: None,
            chose: PickChose::Id("leader:b".into()),
        })
        .collect();
    st.rng = GameRng::scripted(scripted, 47);
    let events = apply(&db, &mut st, Action::EndTurn).expect("oluon evolved eot");
    let pick = events.iter().find_map(|e| match e {
        Event::RandomPick {
            target: arena_engine::TargetOpt::Leader { player },
            ..
        } => Some(*player),
        _ => None,
    });
    assert_eq!(pick, Some(opp), "random_pick must target enemy leader");
    let leader_dmg = events.iter().find_map(|e| match e {
        Event::Damage {
            target: EventTarget::Leader(p),
            ..
        } => Some(*p),
        _ => None,
    });
    assert_eq!(leader_dmg, Some(opp), "next damage must hit that leader");
}

#[test]
fn scripted_rng_still_emits_random_pick() {
    let db = load_db();
    let mut st = started(&db, 48);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, me, "10011210");
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "90011110");
    st.rng = GameRng::scripted(
        vec![Pick {
            what: PickWhat::RandomTarget,
            among: None,
            chose: PickChose::Slot { slot: 0 },
        }],
        47,
    );
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("scripted pick");
    assert!(
        events.iter().any(
            |e| matches!(e, Event::RandomPick { what, .. } if *what == PickWhat::RandomTarget)
        ),
        "random_pick under scripted RNG"
    );
    assert!(st.picks.is_empty(), "scripted emit list stays empty");
}

#[test]
fn resolve_spell_before_damage() {
    let db = load_db();
    let mut st = started(&db, 481);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10753310");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("spell");
    let dmg_idx = events
        .iter()
        .position(|e| matches!(e, Event::Damage { .. }))
        .expect("damage");
    let resolve_idx = events
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::Resolve {
                    source: EventSource::Ref(SourceRef::Spell { .. }),
                    ..
                }
            )
        })
        .expect("spell resolve");
    assert!(resolve_idx < dmg_idx);
}

#[test]
fn resolve_field_on_fanfare_play() {
    let db = load_db();
    let mut st = started(&db, 49);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10002210");
    let uid = st.player(me).hand[h as usize].id;
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("fanfare amulet");
    let resolve_idx = events.iter().position(|e| {
        matches!(
            e,
            Event::Resolve {
                source: EventSource::Ref(SourceRef::Field { player, id }),
                ..
            } if *player == me && *id == uid
        )
    });
    assert!(resolve_idx.is_some(), "field resolve for fanfare");
}

#[test]
fn last_words_restore_follows_spell_resolve() {
    let db = load_db();
    let mut st = started(&db, 51);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let col = put_field(&db, &mut st, opp, "10952110");
    let atk = put_field(&db, &mut st, me, "10872110");
    st.phase = Phase::Main;
    st.active = me;
    let events = apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: arena_engine::Slot(atk),
            target: arena_engine::AttackTarget::Slot(arena_engine::Slot(col)),
        },
    )
    .expect("kill colonel");
    let spell = events.iter().position(|e| {
        matches!(
            e,
            Event::Resolve {
                source: EventSource::Ref(SourceRef::Spell { player, card }),
                ..
            } if *player == opp && card.as_str() == "10952110"
        )
    });
    let restore = events.iter().position(|e| {
        matches!(
            e,
            Event::Restore {
                target: EventTarget::Leader(p),
                ..
            } if *p == opp
        )
    });
    assert!(spell.is_some(), "spell resolve");
    assert!(restore.is_some(), "restore to bot leader");
    assert!(
        spell.unwrap() < restore.unwrap(),
        "spell resolve must precede restore for cue arrows"
    );
}

#[test]
fn last_words_spell_after_destroy() {
    let db = load_db();
    let mut st = started(&db, 51);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let col = put_field(&db, &mut st, opp, "10952110");
    let col_id = slot_id(&st, opp, col);
    let atk = put_field(&db, &mut st, me, "88001320");
    if let Some(f) = st.field_inst_mut(me, atk) {
        f.attack = 10;
    }
    st.phase = Phase::Main;
    st.active = me;
    let events = apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: arena_engine::Slot(atk),
            target: arena_engine::AttackTarget::Slot(arena_engine::Slot(col)),
        },
    )
    .expect("kill colonel");
    let destroy_pos = events
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::Destroy { id, .. } if *id == col_id
            )
        })
        .expect("destroy colonel");
    let lw_resolve = events.iter().position(|e| {
        matches!(
            e,
            Event::Resolve {
                source: EventSource::Ref(SourceRef::Spell { player, card }),
                ..
            } if *player == opp && card.as_str() == "10952110"
        )
    });
    assert!(lw_resolve.is_some());
    assert!(lw_resolve.unwrap() > destroy_pos);
}

#[test]
fn nested_single_resolve_marker() {
    let db = load_db();
    let mut st = started(&db, 52);
    let me = PlayerId::A;
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "10503310");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("seq list");
    let resolves: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, Event::Resolve { .. }))
        .collect();
    let spell_resolves = resolves
        .iter()
        .filter(|e| {
            matches!(
                e,
                Event::Resolve {
                    source: EventSource::Ref(SourceRef::Spell { .. }),
                    ..
                }
            )
        })
        .count();
    assert_eq!(spell_resolves, 1, "one marker for nested spell ops");
}

#[test]
fn combat_resolve_before_damage() {
    let db = load_db();
    let mut st = started(&db, 53);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let atk = put_field(&db, &mut st, me, "88001320");
    let atk_id = slot_id(&st, me, atk);
    put_field(&db, &mut st, opp, "88001110");
    st.phase = Phase::Main;
    st.active = me;
    let events = apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: arena_engine::Slot(atk),
            target: arena_engine::AttackTarget::Leader,
        },
    )
    .expect("attack leader");
    let combat_idx = events
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::Resolve {
                    source: EventSource::Combat { player, id },
                    ..
                } if *player == me && *id == atk_id
            )
        })
        .expect("combat resolve");
    let dmg_idx = events
        .iter()
        .position(|e| matches!(e, Event::Damage { .. }))
        .expect("damage");
    assert!(combat_idx < dmg_idx);
}

#[test]
fn strike_then_combat_markers() {
    let db = load_db();
    let mut st = started(&db, 54);
    let me = PlayerId::A;
    let opp = PlayerId::B;
    let atk = put_field(&db, &mut st, me, "10011130");
    let atk_id = slot_id(&st, me, atk);
    put_field(&db, &mut st, opp, "88001110");
    st.phase = Phase::Main;
    st.active = me;
    let events = apply(
        &db,
        &mut st,
        Action::Attack {
            attacker: arena_engine::Slot(atk),
            target: arena_engine::AttackTarget::Slot(arena_engine::Slot(0)),
        },
    )
    .expect("strike attack");
    let markers: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Resolve { source, .. } => Some(source.clone()),
            _ => None,
        })
        .collect();
    assert!(
        markers.iter().any(|s| matches!(
            s,
            EventSource::Ref(SourceRef::Field { player, id })
                if *player == me && *id == atk_id
        )),
        "strike field marker"
    );
    assert!(
        markers.iter().any(|s| matches!(
            s,
            EventSource::Combat { player, id } if *player == me && *id == atk_id
        )),
        "combat marker after strike"
    );
    let field_pos = markers.iter().position(|s| {
        matches!(
            s,
            EventSource::Ref(SourceRef::Field { player, id })
                if *player == me && *id == atk_id
        )
    });
    let combat_pos = markers.iter().position(
        |s| matches!(s, EventSource::Combat { player, id } if *player == me && *id == atk_id),
    );
    assert!(field_pos.unwrap() < combat_pos.unwrap());
}

#[test]
fn resumed_choose_has_resolve_first() {
    let db = load_db();
    let mut st = started(&db, 55);
    let me = PlayerId::A;
    put_field(&db, &mut st, PlayerId::B, "88001110");
    put_field(&db, &mut st, PlayerId::B, "88001320");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001300");
    apply(&db, &mut st, Action::Play { hand: h }).expect("target spell");
    let events = apply(&db, &mut st, Action::Choose(0)).expect("pick target");
    let resolve_idx = events
        .iter()
        .position(|e| matches!(e, Event::Resolve { .. }))
        .expect("resolve on resumed pick");
    let destroy_idx = events
        .iter()
        .position(|e| matches!(e, Event::Destroy { .. }))
        .unwrap_or(events.len());
    assert!(resolve_idx < destroy_idx);
}

#[test]
fn choice_offered_sources() {
    let db = load_db();
    let me = PlayerId::A;
    let opp = PlayerId::B;

    // Play-time spell pick.
    let mut st = started(&db, 56);
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001300");
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("play-time pick spell");
    let offered = events
        .iter()
        .find_map(|e| match e {
            Event::ChoiceOffered { source, .. } => *source,
            _ => None,
        })
        .expect("choice_offered");
    assert!(matches!(offered, SourceRef::Spell { .. }));

    // Fanfare target choice on the played unit.
    let mut st = started(&db, 561);
    put_field(&db, &mut st, opp, "88001110");
    give_pp(&mut st, me, 10, 10);
    st.player_mut(me).hand.clear();
    let h = put_hand(&db, &mut st, me, "88001210");
    let uid = st.player(me).hand[h as usize].id;
    let events = apply(&db, &mut st, Action::Play { hand: h }).expect("fanfare pick follower");
    let offered = events
        .iter()
        .find_map(|e| match e {
            Event::ChoiceOffered { source, .. } => *source,
            _ => None,
        })
        .expect("fanfare choice_offered");
    assert!(matches!(
        offered,
        SourceRef::Field { player, id } if player == me && id == uid
    ));

    // Evolve ability target choice on the evolving unit.
    let mut st = started(&db, 562);
    put_field(&db, &mut st, opp, "88001110");
    let slot = put_field(&db, &mut st, me, "10622110");
    let uid = slot_id(&st, me, slot);
    skip_to_player_turn(&db, &mut st, me, 5);
    st.player_mut(me).ep = 2;
    let events = apply(
        &db,
        &mut st,
        Action::Evolve {
            slot: arena_engine::Slot(slot),
            super_evolve: false,
        },
    )
    .expect("evolve pick");
    let offered = events
        .iter()
        .find_map(|e| match e {
            Event::ChoiceOffered { source, .. } => *source,
            _ => None,
        })
        .expect("evolve choice_offered");
    assert!(matches!(
        offered,
        SourceRef::Field { player, id } if player == me && id == uid
    ));

    // Fuse partners: no source.
    let mut st = started(&db, 563);
    st.player_mut(me).hand.clear();
    put_hand(&db, &mut st, me, "90071210");
    put_hand(&db, &mut st, me, "90071220");
    let events = apply(&db, &mut st, Action::Fuse { host: 0 }).expect("fuse partners");
    let offered = events
        .iter()
        .find_map(|e| match e {
            Event::ChoiceOffered { source, .. } => Some(source.is_none()),
            _ => None,
        })
        .expect("fuse choice_offered");
    assert!(offered);
}

#[test]
fn countdown_destroy_has_owner_and_id() {
    let db = load_db();
    let mut st = started(&db, 59);
    let opp = PlayerId::B;
    let slot = put_field(&db, &mut st, opp, "10011210");
    let uid = slot_id(&st, opp, slot);
    if let Some(c) = st.field_inst_mut(opp, slot) {
        c.countdown = Some(1);
    }
    let events = apply(&db, &mut st, Action::EndTurn).expect("tick on B start");
    let destroy = events
        .iter()
        .find_map(|e| match e {
            Event::Destroy { player, id, .. } => Some((*player, *id)),
            _ => None,
        })
        .expect("countdown destroy");
    assert_eq!(destroy, (opp, uid));
}

fn check_event_contract(events: &[Event]) -> Option<String> {
    let mut last_resolve: Option<usize> = None;
    for (i, e) in events.iter().enumerate() {
        if let Event::Resolve { .. } = e {
            last_resolve = Some(i);
        }
        match e {
            Event::Damage { unit, target, .. } | Event::Restore { unit, target, .. } => {
                if last_resolve.is_none() {
                    return Some(format!("missing resolve before event {i}"));
                }
                if let EventTarget::Slot(_, _) = target {
                    if unit.is_none() {
                        return Some(format!("slot damage/restore without id at {i}"));
                    }
                }
            }
            Event::RandomPick { unit, target, .. } => {
                if last_resolve.is_none() {
                    return Some(format!("missing resolve before random_pick at {i}"));
                }
                if let arena_engine::TargetOpt::Slot { .. } = target {
                    if unit.is_none() {
                        return Some(format!("random_pick slot without id at {i}"));
                    }
                }
            }
            Event::Destroy { id, player, .. } => {
                if *id == 0 {
                    return Some(format!("destroy without id at {i}"));
                }
                let _ = player;
            }
            _ => {}
        }
    }
    None
}

fn meta_stems() -> Vec<String> {
    let text =
        std::fs::read_to_string(repo_root().join("oracle/decks/POOLS.json")).expect("POOLS.json");
    let pools: serde_json::Value = serde_json::from_str(&text).expect("pools json");
    pools["meta"]
        .as_array()
        .expect("meta pool")
        .iter()
        .map(|v| v.as_str().expect("deck stem").to_string())
        .collect()
}

#[derive(Default, Debug)]
struct EventSoakCounts {
    random_pick: u64,
    ability_damage: u64,
    restore: u64,
    resolve: u64,
    destroy: u64,
}

fn tally_step_events(events: &[Event], counts: &mut EventSoakCounts) -> Option<String> {
    if let Some(msg) = check_event_contract(events) {
        return Some(msg);
    }
    let mut last_resolve_combat = false;
    for e in events {
        if let Event::Resolve { source, .. } = e {
            counts.resolve += 1;
            last_resolve_combat = matches!(source, EventSource::Combat { .. });
            continue;
        }
        match e {
            Event::RandomPick { .. } => counts.random_pick += 1,
            Event::Restore { .. } => counts.restore += 1,
            Event::Destroy { .. } => counts.destroy += 1,
            Event::Damage { .. } if !last_resolve_combat => counts.ability_damage += 1,
            _ => {}
        }
    }
    None
}

#[test]
fn property_resolve_and_ids_over_soak() {
    let per_pair: u32 = std::env::var("ARENA_EVENT_SOAK_GAMES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    let db = load_db();
    let stems = meta_stems();
    assert!(!stems.is_empty());
    let decks: Vec<Vec<arena_engine::CardId>> = stems
        .iter()
        .map(|s| load_deck_file(repo_root().join(format!("oracle/decks/{s}.json"))))
        .collect();
    let mut counts = EventSoakCounts::default();
    let mut games = 0u32;
    for (i, da) in decks.iter().enumerate() {
        for (j, dbk) in decks.iter().enumerate() {
            if i == j {
                continue;
            }
            for g in 0..per_pair {
                let seed = 90_000 + (i as u64) * 1_000 + (j as u64) * 10 + u64::from(g);
                let mut state = new_game(
                    &db,
                    GameConfig {
                        seed,
                        deck_a: da.clone(),
                        deck_b: dbk.clone(),
                        first: First::A,
                        opening_hands: None,
                    },
                )
                .expect("new_game");
                let mut policy = policy_rng(seed);
                apply(
                    &db,
                    &mut state,
                    Action::MulliganConfirm { swap: [false; 4] },
                )
                .unwrap();
                apply(
                    &db,
                    &mut state,
                    Action::MulliganConfirm { swap: [false; 4] },
                )
                .unwrap();
                let mut actions = 0u32;
                while state.winner.is_none()
                    && !matches!(state.phase, Phase::Terminal)
                    && actions < 500
                {
                    let legal = legal_actions(&db, &state);
                    if legal.is_empty() {
                        break;
                    }
                    let idx = policy.gen_range(legal.len() as u32) as usize;
                    let act = legal[idx].clone();
                    let events = apply(&db, &mut state, act).unwrap_or_else(|e: Illegal| {
                        panic!("game {games} action failed: {e:?}");
                    });
                    if let Some(msg) = tally_step_events(&events, &mut counts) {
                        panic!("game {games} step {actions}: {msg}");
                    }
                    actions += 1;
                }
                games += 1;
            }
        }
    }
    eprintln!(
        "event_soak {games} games: random_pick={} ability_damage={} restore={} resolve={} destroy={}",
        counts.random_pick,
        counts.ability_damage,
        counts.restore,
        counts.resolve,
        counts.destroy
    );
    assert!(
        games >= 300,
        "expected >=300 meta deck-pair games, got {games}"
    );
    assert!(counts.random_pick >= 100, "random_pick count too low");
    assert!(counts.ability_damage >= 100, "ability damage count too low");
}
