//! Pure rules for the skill board: no egui, no I/O. Everything the UI needs to
//! DECIDE lives here so it is unit-tested; the board only draws and forwards
//! clicks. Fixes two gaps in the old tab: (1) adding a rank with 0 points
//! available, (2) removing a point that silently locks a higher tier that still
//! has points.

/// One skill in a tree: current grade and its max.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub grade: i32,
    pub max: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tier {
    pub level: i32,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    pub tiers: Vec<Tier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deny {
    /// Tier needs `need` points in this tree, has `have`.
    Locked {
        need: i32,
        have: i32,
    },
    Maxed,
    NoPoints,
    Empty,
    /// Removing would drop the tree below what a HIGHER tier `tier_level` (which holds points) needs.
    WouldLock {
        tier_level: i32,
        need: i32,
    },
    /// Tier 1 needs a point in the class action skill first.
    NeedsActionSkill,
    /// The action-skill point is held by tier 1.
    ActionSkillHeld,
}

impl Deny {
    /// One line for the details panel / tooltip. Never colour-only (spec 2.3).
    pub fn message(self) -> String {
        match self {
            Deny::Locked { need, have } => {
                format!("Locked: needs {need} points in this tree ({have} invested)")
            }
            Deny::Maxed => "Already at max rank".into(),
            Deny::NoPoints => "No skill points available".into(),
            Deny::Empty => "Already at rank 0".into(),
            Deny::WouldLock { tier_level, need } => format!(
                "Can't remove: tier {} would lock (needs {need} points)",
                tier_level + 1
            ),
            Deny::NeedsActionSkill => "Put a point in your action skill first".into(),
            Deny::ActionSkillHeld => "Can't remove: tier 1 still has points in this class".into(),
        }
    }
}

/// Whether a tier is open. Tier 1 requires the action skill; tiers 2+ require
/// `5 * (tier-1)` points in the tree.
pub fn tier_open(total: i32, level: i32, tier_index: usize, action_ready: bool) -> bool {
    if tier_index == 0 {
        action_ready
    } else {
        unlocked(total, level)
    }
}

/// Whether the action-skill point may be cleared: not while tier 1 (any tree of
/// the class) still holds a rank.
pub fn action_clear_ok(trees: &[Tree]) -> bool {
    trees.iter().all(|t| {
        t.tiers
            .first()
            .is_none_or(|tier| tier.nodes.iter().all(|n| n.grade <= 0))
    })
}

pub fn needed(level: i32) -> i32 {
    5 * level
}
pub fn unlocked(total: i32, level: i32) -> bool {
    total >= needed(level)
}
pub fn total(t: &Tree) -> i32 {
    t.tiers
        .iter()
        .flat_map(|x| &x.nodes)
        .map(|n| n.grade.max(0))
        .sum()
}

pub fn can_add(
    t: &Tree,
    tier: usize,
    node: usize,
    available: i32,
    enforce_budget: bool,
    action_ready: bool,
) -> Result<(), Deny> {
    let tr = &t.tiers[tier];
    let n = tr.nodes[node];
    let have = total(t);
    if tier == 0 && !action_ready {
        return Err(Deny::NeedsActionSkill);
    }
    if !unlocked(have, tr.level) {
        return Err(Deny::Locked {
            need: needed(tr.level),
            have,
        });
    }
    if n.grade >= n.max {
        return Err(Deny::Maxed);
    }
    if enforce_budget && available <= 0 {
        return Err(Deny::NoPoints);
    }
    Ok(())
}

pub fn can_remove(t: &Tree, tier: usize, node: usize) -> Result<(), Deny> {
    if t.tiers[tier].nodes[node].grade <= 0 {
        return Err(Deny::Empty);
    }
    let after = total(t) - 1;
    for (i, tr) in t.tiers.iter().enumerate() {
        let points: i32 =
            tr.nodes.iter().map(|n| n.grade.max(0)).sum::<i32>() - if i == tier { 1 } else { 0 };
        if i > tier && points > 0 && !unlocked(after, tr.level) {
            return Err(Deny::WouldLock {
                tier_level: tr.level,
                need: needed(tr.level),
            });
        }
    }
    Ok(())
}

/// Grid slot (0..3) for each node of a tier. Uses the generated `col` when the
/// node carries one (the source page's real layout), otherwise falls back to
/// the count rule (single centred, pair flanked).
pub fn slots(cols: &[Option<u32>]) -> Vec<usize> {
    if !cols.is_empty() && cols.iter().all(Option::is_some) {
        return cols.iter().map(|c| c.unwrap() as usize).collect();
    }
    match cols.len() {
        1 => vec![1],
        2 => vec![0, 2],
        n => (0..n.min(3)).collect(),
    }
}

/// Points refunded by a respec (everything invested).
pub fn respec_refund(trees: &[Tree]) -> i32 {
    trees.iter().map(total).sum()
}

// ---- Build string: "LS2:<class>:<digits per tree separated by '-'>" -------------------------------
// One digit per node in data order (grades are 0..=5). Short, pasteable, checked against shape.

pub fn encode(class: &str, trees: &[Tree]) -> String {
    let parts: Vec<String> = trees
        .iter()
        .map(|t| {
            t.tiers
                .iter()
                .flat_map(|x| &x.nodes)
                .map(|n| char::from(b'0' + n.grade.clamp(0, 9) as u8))
                .collect()
        })
        .collect();
    format!("LS2:{class}:{}", parts.join("-"))
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuildError {
    Format,
    WrongClass,
    Shape,
    TooHigh,
}

/// Returns grades per tree (flat, data order). Does not apply them; caller validates with `can_*`.
pub fn decode(s: &str, class: &str, trees: &[Tree]) -> Result<Vec<Vec<i32>>, BuildError> {
    let rest = s.trim().strip_prefix("LS2:").ok_or(BuildError::Format)?;
    let (c, digits) = rest.split_once(':').ok_or(BuildError::Format)?;
    if !c.eq_ignore_ascii_case(class) {
        return Err(BuildError::WrongClass);
    }
    let groups: Vec<&str> = digits.split('-').collect();
    if groups.len() != trees.len() {
        return Err(BuildError::Shape);
    }
    let mut out = vec![];
    for (g, t) in groups.iter().zip(trees) {
        let maxes: Vec<i32> = t
            .tiers
            .iter()
            .flat_map(|x| &x.nodes)
            .map(|n| n.max)
            .collect();
        if g.len() != maxes.len() || !g.bytes().all(|b| b.is_ascii_digit()) {
            return Err(BuildError::Shape);
        }
        let grades: Vec<i32> = g.bytes().map(|b| (b - b'0') as i32).collect();
        if grades.iter().zip(&maxes).any(|(g, m)| g > m) {
            return Err(BuildError::TooHigh);
        }
        out.push(grades);
    }
    Ok(out)
}

/// True when a decoded build obeys the tier rule (grades placed tier by tier).
pub fn build_is_valid(t: &Tree, grades: &[i32]) -> bool {
    let mut it = grades.iter();
    let mut running = 0;
    // Tiers unlock on the tree TOTAL, so check each tier with points against the final total.
    let mut with_points = vec![];
    for tr in &t.tiers {
        let sum: i32 = tr.nodes.iter().map(|_| *it.next().unwrap_or(&0)).sum();
        running += sum;
        if sum > 0 {
            with_points.push(tr.level);
        }
    }
    with_points.iter().all(|l| unlocked(running, *l))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tree(grades: &[&[i32]]) -> Tree {
        Tree {
            tiers: grades
                .iter()
                .enumerate()
                .map(|(i, g)| Tier {
                    level: i as i32,
                    nodes: g.iter().map(|&x| Node { grade: x, max: 5 }).collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn tier_gate_matches_old_rule() {
        let t = tree(&[&[4, 0], &[0, 0]]);
        assert_eq!(
            can_add(&t, 1, 0, 9, true, true),
            Err(Deny::Locked { need: 5, have: 4 })
        );
        let t = tree(&[&[5, 0], &[0, 0]]);
        assert_eq!(can_add(&t, 1, 0, 9, true, true), Ok(()));
    }
    #[test]
    fn budget_and_max() {
        let t = tree(&[&[5, 0]]);
        assert_eq!(can_add(&t, 0, 0, 3, true, true), Err(Deny::Maxed));
        assert_eq!(can_add(&t, 0, 1, 0, true, true), Err(Deny::NoPoints));
        assert_eq!(can_add(&t, 0, 1, 0, false, true), Ok(())); // "ignore budget" editor mode
    }
    #[test]
    fn action_skill_gates_tier_one() {
        // Tier 1 (index 0) needs the action skill; tiers 2+ keep the point gate.
        let t = tree(&[&[0, 0], &[0, 0]]);
        assert_eq!(
            can_add(&t, 0, 0, 9, true, false),
            Err(Deny::NeedsActionSkill)
        );
        assert!(can_add(&t, 0, 0, 9, true, true).is_ok());
        assert!(!tier_open(0, 0, 0, false));
        assert!(tier_open(0, 0, 0, true));
        assert!(!tier_open(3, 1, 1, true), "tier 2 still needs 5 points");
        assert!(
            tier_open(5, 1, 1, false),
            "tier 2 is not gated on the action skill"
        );

        // The action point can't be cleared while tier 1 holds ranks.
        assert!(!action_clear_ok(&[tree(&[&[1, 0]])]));
        assert!(action_clear_ok(&[tree(&[&[0, 0], &[1, 0]])]));
        assert!(action_clear_ok(&[tree(&[&[0, 0]])]));
    }
    #[test]
    fn removal_that_would_lock_a_higher_tier_is_denied() {
        // total 10 unlocks tier 2 (needs 10). Removing from tier 0 drops to 9 and would lock it.
        let t = tree(&[&[5, 0], &[3, 0], &[2, 0]]);
        assert_eq!(
            can_remove(&t, 0, 0),
            Err(Deny::WouldLock {
                tier_level: 2,
                need: 10
            })
        );
        // Removing from the top tier itself is always allowed (nothing above it depends on it).
        assert_eq!(can_remove(&t, 2, 0), Ok(()));
        // Spare points above the gate: total 12, removing one leaves 11 >= 10.
        let t = tree(&[&[5, 0], &[3, 0], &[4, 0]]);
        assert_eq!(can_remove(&t, 0, 0), Ok(()));
        assert_eq!(can_remove(&tree(&[&[0, 0]]), 0, 0), Err(Deny::Empty));
    }
    #[test]
    fn columns_win_over_the_count_rule() {
        // Generated columns are used verbatim: adjacent pair (0,1), not flanks.
        assert_eq!(slots(&[Some(0), Some(1)]), vec![0, 1]);
        assert_eq!(slots(&[Some(1)]), vec![1]);
        assert_eq!(slots(&[Some(0), Some(2)]), vec![0, 2]);
        // Absent columns fall back to the count rule (single centre, pair flank).
        assert_eq!(slots(&[None]), vec![1]);
        assert_eq!(slots(&[None, None]), vec![0, 2]);
        assert_eq!(slots(&[None, None, None]), vec![0, 1, 2]);
    }
    #[test]
    fn build_string_round_trips_and_rejects_bad_input() {
        let trees = vec![tree(&[&[5, 0], &[2, 0]]), tree(&[&[1, 1]])];
        let s = encode("Soldier", &trees);
        assert_eq!(s, "LS2:Soldier:5020-11");
        let g = decode(&s, "soldier", &trees).unwrap();
        assert_eq!(g, vec![vec![5, 0, 2, 0], vec![1, 1]]);
        assert!(build_is_valid(&trees[0], &g[0]));
        assert_eq!(decode("nope", "Soldier", &trees), Err(BuildError::Format));
        assert_eq!(decode(&s, "Siren", &trees), Err(BuildError::WrongClass));
        assert_eq!(
            decode("LS2:Soldier:5020", "Soldier", &trees),
            Err(BuildError::Shape)
        );
        assert_eq!(
            decode("LS2:Soldier:9020-11", "Soldier", &trees),
            Err(BuildError::TooHigh)
        );
        // 2 points in tier 1 with only 0 in tier 0 is not a legal build.
        assert!(!build_is_valid(&trees[0], &[0, 0, 2, 0]));
    }
    #[test]
    fn deny_messages_are_plain_text_and_never_colour_only() {
        assert_eq!(
            Deny::Locked { need: 5, have: 4 }.message(),
            "Locked: needs 5 points in this tree (4 invested)"
        );
        assert_eq!(Deny::NoPoints.message(), "No skill points available");
        assert_eq!(Deny::Maxed.message(), "Already at max rank");
        assert_eq!(Deny::Empty.message(), "Already at rank 0");
        assert_eq!(
            Deny::WouldLock {
                tier_level: 2,
                need: 10
            }
            .message(),
            "Can't remove: tier 3 would lock (needs 10 points)"
        );
    }

    #[test]
    fn respec_refunds_everything() {
        assert_eq!(
            respec_refund(&[tree(&[&[5, 0], &[2, 0]]), tree(&[&[1, 1]])]),
            9
        );
    }
}
