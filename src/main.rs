use rand::seq::SliceRandom;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{self, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const CHAMPIONS_JSON: &str = include_str!("../champions.json");
const AUGMENTS_JSON: &str = include_str!("../augments.json");
const SYNERGIES_JSON: &str = include_str!("../synergies.json");

const LEVELS: [u32; 4] = [3, 7, 11, 15];
const CHOICES_PER_PICK: usize = 3;
const CARD_WIDTH: usize = 32;
const CARD_GAP: usize = 3;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[38;2;120;220;120m";
const CYAN: &str = "\x1b[38;2;120;200;230m";

#[derive(Debug, Deserialize, Clone)]
struct Champion {
    name: String,
    title: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Augment {
    tier: String,
    name: String,
    description: String,
    #[serde(default)]
    #[allow(dead_code)]
    image: String,
    #[serde(default)]
    #[allow(dead_code)]
    exclusive: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct Stage {
    threshold: usize,
    effect: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Synergy {
    name: String,
    #[allow(dead_code)]
    base: String,
    augments: Vec<String>,
    stages: Vec<Stage>,
}

struct Borders {
    tl: &'static str,
    tr: &'static str,
    bl: &'static str,
    br: &'static str,
    h: &'static str,
    v: &'static str,
    div_l: &'static str,
    div_r: &'static str,
}

const SILVER_B: Borders = Borders {
    tl: "╭", tr: "╮", bl: "╰", br: "╯", h: "─", v: "│", div_l: "├", div_r: "┤",
};
const GOLD_B: Borders = Borders {
    tl: "┏", tr: "┓", bl: "┗", br: "┛", h: "━", v: "┃", div_l: "┠", div_r: "┨",
};
const PRISM_B: Borders = Borders {
    tl: "╔", tr: "╗", bl: "╚", br: "╝", h: "═", v: "║", div_l: "╟", div_r: "╢",
};

fn borders_for(tier: &str) -> &'static Borders {
    match tier {
        "silver" => &SILVER_B,
        "gold" => &GOLD_B,
        "prism" => &PRISM_B,
        _ => &SILVER_B,
    }
}

fn tier_label(tier: &str) -> &'static str {
    match tier {
        "silver" => "실버",
        "gold" => "골드",
        "prism" => "프리즘",
        _ => "?",
    }
}

fn tier_color(tier: &str) -> &'static str {
    match tier {
        "silver" => "\x1b[38;2;200;200;210m",
        "gold" => "\x1b[38;2;255;200;75m",
        "prism" => "\x1b[38;2;230;130;230m",
        _ => "",
    }
}

const ICON_SILVER: [&str; 4] = [
    "     ▟████▙     ",
    "    ▟██████▙    ",
    "    ▜██████▛    ",
    "     ▜████▛     ",
];

const ICON_GOLD: [&str; 4] = [
    "       ▄▄▄▄     ",
    "     ▟██████▙   ",
    "     ▜██████▛   ",
    "       ▀▀▀▀     ",
];

const ICON_PRISM: [&str; 4] = [
    "     ▄▟▙▄▟▙▄    ",
    "    ▟██▜█▛██▙   ",
    "    ▜██▟█▙██▛   ",
    "     ▀▜▛▀▜▛▀    ",
];

fn icon_for(tier: &str) -> &'static [&'static str; 4] {
    match tier {
        "silver" => &ICON_SILVER,
        "gold" => &ICON_GOLD,
        "prism" => &ICON_PRISM,
        _ => &ICON_SILVER,
    }
}

fn random_tier(rng: &mut impl Rng) -> &'static str {
    let r: f64 = rng.gen_range(0.0..1.0);
    if r < 0.50 { "silver" } else if r < 0.85 { "gold" } else { "prism" }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for n in chars.by_ref() {
                if n.is_ascii_alphabetic() { break; }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn vw(s: &str) -> usize {
    UnicodeWidthStr::width(strip_ansi(s).as_str())
}

fn pad_right(s: &str, total: usize) -> String {
    let w = vw(s);
    if w >= total { s.to_string() } else { format!("{}{}", s, " ".repeat(total - w)) }
}

fn center_in(s: &str, total: usize) -> String {
    let w = vw(s);
    if w >= total { return s.to_string(); }
    let left = (total - w) / 2;
    let right = total - w - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

fn wrap_text(s: &str, max_width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut cur_w = 0usize;
    for word in s.split_whitespace() {
        let w = UnicodeWidthStr::width(word);
        if w > max_width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                cur_w = 0;
            }
            let mut buf = String::new();
            let mut bw = 0usize;
            for ch in word.chars() {
                let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
                if bw + cw > max_width {
                    lines.push(std::mem::take(&mut buf));
                    bw = 0;
                }
                buf.push(ch);
                bw += cw;
            }
            if !buf.is_empty() { current = buf; cur_w = bw; }
            continue;
        }
        let needed = if current.is_empty() { w } else { cur_w + 1 + w };
        if needed > max_width {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            cur_w = w;
        } else {
            if !current.is_empty() { current.push(' '); cur_w += 1; }
            current.push_str(word);
            cur_w += w;
        }
    }
    if !current.is_empty() { lines.push(current); }
    lines
}

fn synergies_for_augment<'a>(synergies: &'a [Synergy], aug_name: &str) -> Vec<&'a str> {
    synergies.iter()
        .filter(|s| s.augments.iter().any(|n| n == aug_name))
        .map(|s| s.name.as_str())
        .collect()
}

fn is_transition_augment(name: &str) -> bool {
    name.starts_with("전환:") || name.starts_with("전환 :")
}

fn card_blank(bd: &Borders, tc: &str, inner: usize) -> String {
    format!("{}{}{}{}{}{}{}", tc, bd.v, RESET, " ".repeat(inner), tc, bd.v, RESET)
}

fn card_row(bd: &Borders, tc: &str, inner: usize, content: &str) -> String {
    let padded = pad_right(content, inner);
    format!("{}{}{}{}{}{}{}", tc, bd.v, RESET, padded, tc, bd.v, RESET)
}

fn build_card(
    slot: usize,
    aug: &Augment,
    reroll_used: bool,
    synergies: &[Synergy],
) -> Vec<String> {
    let inner = CARD_WIDTH - 2;
    let bd = borders_for(&aug.tier);
    let tc = tier_color(&aug.tier);
    let icon = icon_for(&aug.tier);

    let mut lines: Vec<String> = Vec::new();

    // Slot number indicator above card
    let slot_label = format!("{}[ {} ]{}", BOLD, slot, RESET);
    lines.push(center_in(&slot_label, CARD_WIDTH));

    // Top border
    lines.push(format!("{}{}{}{}{}", tc, bd.tl, bd.h.repeat(inner), bd.tr, RESET));

    // Blank
    lines.push(card_blank(bd, tc, inner));

    // Icon (4 lines, centered)
    for icon_line in icon {
        let colored = format!("{}{}{}", tc, icon_line, RESET);
        let centered = center_in(&colored, inner);
        lines.push(card_row(bd, tc, inner, &centered));
    }

    // Blank
    lines.push(card_blank(bd, tc, inner));

    // Tier label
    let label = format!("{}─ {} ─{}", tc, tier_label(&aug.tier), RESET);
    lines.push(card_row(bd, tc, inner, &center_in(&label, inner)));

    // Blank
    lines.push(card_blank(bd, tc, inner));

    // Divider
    lines.push(format!("{}{}{}{}{}", tc, bd.div_l, bd.h.repeat(inner), bd.div_r, RESET));

    // Blank
    lines.push(card_blank(bd, tc, inner));

    // Name (wrapped, centered, bold)
    let name_lines = wrap_text(&aug.name, inner - 2);
    for nl in name_lines {
        let colored = format!("{}{}{}", BOLD, nl, RESET);
        lines.push(card_row(bd, tc, inner, &center_in(&colored, inner)));
    }

    // Blank
    lines.push(card_blank(bd, tc, inner));

    // Divider
    lines.push(format!("{}{}{}{}{}", tc, bd.div_l, bd.h.repeat(inner), bd.div_r, RESET));

    // Blank
    lines.push(card_blank(bd, tc, inner));

    // Description (wrapped, left-aligned with padding)
    let desc_inner = inner.saturating_sub(4);
    let desc_lines = wrap_text(&aug.description, desc_inner);
    for dl in desc_lines {
        let colored = format!("{}{}{}", DIM, dl, RESET);
        lines.push(card_row(bd, tc, inner, &format!("  {}", pad_right(&colored, desc_inner + 2))));
    }

    // Synergy
    let syns = synergies_for_augment(synergies, &aug.name);
    if !syns.is_empty() {
        lines.push(card_blank(bd, tc, inner));
        let prefix = if is_transition_augment(&aug.name) { "전환 : " } else { "" };
        let body = format!("◆ 시너지: {}{}", prefix, syns.join(", "));
        let tag_lines = wrap_text(&body, desc_inner);
        for tl in tag_lines {
            let colored = if let Some(idx) = tl.find("전환 : ") {
                let (before, rest) = tl.split_at(idx);
                let after = &rest["전환 : ".len()..];
                format!(
                    "{}{}{}전환 : {}{}{}",
                    GREEN, before, CYAN, GREEN, after, RESET
                )
            } else {
                format!("{}{}{}", GREEN, tl, RESET)
            };
            lines.push(card_row(bd, tc, inner, &format!("  {}", pad_right(&colored, desc_inner + 2))));
        }
    }

    // Blank
    lines.push(card_blank(bd, tc, inner));

    // Bottom border
    lines.push(format!("{}{}{}{}{}", tc, bd.bl, bd.h.repeat(inner), bd.br, RESET));

    // Reroll button below card
    let btn = if reroll_used {
        format!("{}╰─ 리롤 사용 ─╯{}", DIM, RESET)
    } else {
        format!("{}╭─ r{}: 리롤 ─╮{}", tc, slot, RESET)
    };
    lines.push(center_in(&btn, CARD_WIDTH));

    lines
}

fn stack_cards_horizontally(cards: Vec<Vec<String>>) -> Vec<String> {
    let max_h = cards.iter().map(|c| c.len()).max().unwrap_or(0);
    let gap = " ".repeat(CARD_GAP);
    let mut out: Vec<String> = Vec::with_capacity(max_h);
    for i in 0..max_h {
        let mut row = String::new();
        for (idx, card) in cards.iter().enumerate() {
            if idx > 0 {
                row.push_str(&gap);
            }
            if i < card.len() {
                row.push_str(&pad_right(&card[i], CARD_WIDTH));
            } else {
                row.push_str(&" ".repeat(CARD_WIDTH));
            }
        }
        out.push(row);
    }
    out
}

fn normalize_card_heights(cards: &mut [Vec<String>], slots: &[Augment]) {
    // Card layout: [slot_label, top_border, ...body..., bottom_border, reroll_button]
    // Pad body (insert blank lines at len()-2) to match the tallest card.
    let max_len = cards.iter().map(|c| c.len()).max().unwrap_or(0);
    let inner = CARD_WIDTH - 2;
    for (i, card) in cards.iter_mut().enumerate() {
        let bd = borders_for(&slots[i].tier);
        let tc = tier_color(&slots[i].tier);
        let blank = card_blank(bd, tc, inner);
        while card.len() < max_len {
            let insert_at = card.len() - 2;
            card.insert(insert_at, blank.clone());
        }
    }
}

fn print_three_cards(slots: &[Augment], reroll_used: &[bool], synergies: &[Synergy]) {
    let mut cards: Vec<Vec<String>> = slots
        .iter()
        .enumerate()
        .map(|(i, a)| build_card(i + 1, a, reroll_used[i], synergies))
        .collect();
    normalize_card_heights(&mut cards, slots);
    for line in stack_cards_horizontally(cards) {
        println!("{}", line);
    }
}

const PANEL_WIDTH: usize = CARD_WIDTH * CHOICES_PER_PICK + CARD_GAP * (CHOICES_PER_PICK - 1);

fn synergy_bar(s: &Synergy, count: usize) -> String {
    let max = s.stages.last().map(|x| x.threshold).unwrap_or(0);
    let mut bar = String::new();
    for i in 1..=max {
        if i <= count {
            bar.push('█');
        } else {
            bar.push('░');
        }
    }
    bar
}

fn print_accumulated_synergies(synergies: &[Synergy], picked: &[Augment]) {
    let progress = synergy_progress(synergies, picked);
    let inner = PANEL_WIDTH - 2;

    let title = " 누적 시너지 ";
    let title_w = vw(title);
    let title_left = inner.saturating_sub(title_w) / 2;
    let title_right = inner - title_w - title_left;
    println!(
        "{}╭{}{}{}{}{}{}{}╮{}",
        DIM,
        "─".repeat(title_left),
        BOLD,
        title,
        RESET,
        DIM,
        "─".repeat(title_right),
        DIM,
        RESET
    );

    if progress.is_empty() {
        let msg = "아직 누적된 시너지 없음";
        let padded = center_in(&format!("{}{}{}", DIM, msg, RESET), inner);
        println!("{}│{}{}{}│{}", DIM, RESET, padded, DIM, RESET);
    } else {
        for (s, c) in progress {
            let max_th = s.stages.last().map(|x| x.threshold).unwrap_or(0);
            let bar = synergy_bar(s, c);
            let active = current_stage(s, c);
            let status = if let Some(st) = active {
                format!("{}✓ {}단계 활성{}", GREEN, st.threshold, RESET)
            } else {
                let next = s
                    .stages
                    .iter()
                    .find(|st| st.threshold > c)
                    .map(|x| x.threshold)
                    .unwrap_or(max_th);
                format!("{}다음 {}개 필요{}", DIM, next - c, RESET)
            };
            let name_padded = pad_right(&format!("{}{}{}", BOLD, s.name, RESET), 18);
            let bar_colored = if active.is_some() {
                format!("{}{}{}", GREEN, bar, RESET)
            } else {
                format!("{}{}{}", DIM, bar, RESET)
            };
            let bar_padded = pad_right(&bar_colored, 6);
            let progress_text = format!("{}({}/{}){}", DIM, c, max_th, RESET);
            let line = format!(
                "  ◆ {}  {}  {}  {}",
                name_padded, bar_padded, progress_text, status
            );
            println!("{}│{}{}{}│{}", DIM, RESET, pad_right(&line, inner), DIM, RESET);
        }
    }
    println!("{}╰{}╯{}", DIM, "─".repeat(inner), RESET);
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().ok();
    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_err() {
        std::process::exit(0);
    }
    buf.trim().to_lowercase()
}

fn synergy_progress<'a>(synergies: &'a [Synergy], picked: &[Augment]) -> Vec<(&'a Synergy, usize)> {
    synergies.iter()
        .map(|s| {
            let count = picked.iter()
                .filter(|p| s.augments.iter().any(|n| n == &p.name))
                .count();
            (s, count)
        })
        .filter(|(_, c)| *c > 0)
        .collect()
}

fn current_stage(syn: &Synergy, count: usize) -> Option<&Stage> {
    syn.stages.iter().rev().find(|st| st.threshold <= count)
}

fn print_synergy_status(synergies: &[Synergy], picked: &[Augment]) {
    let progress = synergy_progress(synergies, picked);
    if progress.is_empty() { return; }
    println!();
    println!("    {}━━━ 시너지 진행 ━━━{}", DIM, RESET);
    for (s, c) in progress {
        let max_th = s.stages.last().map(|x| x.threshold).unwrap_or(0);
        let active = current_stage(s, c);
        let (col, label) = if let Some(st) = active {
            (GREEN, format!("✓ {}단계 활성", st.threshold))
        } else {
            let next = s.stages.iter().find(|st| st.threshold > c).map(|x| x.threshold).unwrap_or(max_th);
            (DIM, format!("다음 {}개", next - c))
        };
        println!(
            "    ◆ {}{}{}  ({}/{})  {}{}{}",
            BOLD, s.name, RESET, c, max_th, col, label, RESET
        );
    }
}

fn pick_random_of_tier(
    tier: &str,
    augments: &[Augment],
    seen: &HashSet<String>,
    excluded: &HashSet<String>,
    rng: &mut impl Rng,
) -> Option<Augment> {
    let avail: Vec<&Augment> = augments.iter()
        .filter(|a| a.tier == tier && !seen.contains(&a.name) && !excluded.contains(&a.name))
        .filter(|a| !is_transition_augment(&a.name) && a.name != "판도라의 상자")
        .collect();
    avail.choose(rng).map(|&a| a.clone())
}

/// Returns true if this augment had a special effect that already populated picked.
fn handle_special(
    aug: &Augment,
    picked: &mut Vec<Augment>,
    seen: &mut HashSet<String>,
    augments: &[Augment],
    rng: &mut impl Rng,
) -> bool {
    match aug.name.as_str() {
        "전환: 골드" => {
            seen.insert(aug.name.clone());
            picked.push(aug.clone());
            let excluded: HashSet<String> = picked.iter().map(|p| p.name.clone()).collect();
            if let Some(new_aug) = pick_random_of_tier("gold", augments, seen, &excluded, rng) {
                println!("\n    {}✦ '전환: 골드' 발동 → 무작위 골드: {}{}{}{}",
                    CYAN, BOLD, new_aug.name, RESET, RESET);
                seen.insert(new_aug.name.clone());
                picked.push(new_aug);
            }
            true
        }
        "전환: 프리즘" => {
            seen.insert(aug.name.clone());
            picked.push(aug.clone());
            let excluded: HashSet<String> = picked.iter().map(|p| p.name.clone()).collect();
            if let Some(new_aug) = pick_random_of_tier("prism", augments, seen, &excluded, rng) {
                println!("\n    {}✦ '전환: 프리즘' 발동 → 무작위 프리즘: {}{}{}{}",
                    CYAN, BOLD, new_aug.name, RESET, RESET);
                seen.insert(new_aug.name.clone());
                picked.push(new_aug);
            }
            true
        }
        "전환: 혼돈" => {
            seen.insert(aug.name.clone());
            picked.push(aug.clone());
            println!("\n    {}✦ '전환: 혼돈' 발동 → 무작위 증강 2개 획득{}", CYAN, RESET);
            for _ in 0..2 {
                let tier = random_tier(rng);
                let excluded: HashSet<String> = picked.iter().map(|p| p.name.clone()).collect();
                if let Some(new_aug) = pick_random_of_tier(tier, augments, seen, &excluded, rng) {
                    println!("        + {}[{}]{} {}{}{}",
                        tier_color(&new_aug.tier), tier_label(&new_aug.tier), RESET,
                        BOLD, new_aug.name, RESET);
                    seen.insert(new_aug.name.clone());
                    picked.push(new_aug);
                }
            }
            true
        }
        "판도라의 상자" => {
            seen.insert(aug.name.clone());
            let n_prev = picked.len();
            println!("\n    {}✦ '판도라의 상자' 발동 → 기존 {}개 증강이 무작위 프리즘으로 전환!{}",
                CYAN, n_prev, RESET);
            for prev in picked.iter() {
                seen.insert(prev.name.clone());
            }
            picked.clear();
            let mut already: HashSet<String> = HashSet::new();
            for _ in 0..n_prev {
                if let Some(new_aug) = pick_random_of_tier("prism", augments, seen, &already, rng) {
                    println!("        ◇ {}{}{}", BOLD, new_aug.name, RESET);
                    already.insert(new_aug.name.clone());
                    seen.insert(new_aug.name.clone());
                    picked.push(new_aug);
                }
            }
            picked.push(aug.clone());
            true
        }
        _ => false,
    }
}

fn print_header(title: &str) {
    let inner = 64;
    let pad = inner - vw(title);
    let lpad = pad / 2;
    let rpad = pad - lpad;
    println!("{}╔{}╗{}", BOLD, "═".repeat(inner), RESET);
    println!("{}║{}{}{}{}║{}", BOLD, " ".repeat(lpad), title, " ".repeat(rpad), BOLD, RESET);
    println!("{}╚{}╝{}", BOLD, "═".repeat(inner), RESET);
}

fn main() {
    let champions: Vec<Champion> =
        serde_json::from_str(CHAMPIONS_JSON).expect("champions.json 파싱 실패");
    let augments: Vec<Augment> =
        serde_json::from_str(AUGMENTS_JSON).expect("augments.json 파싱 실패");
    let synergies: Vec<Synergy> =
        serde_json::from_str(SYNERGIES_JSON).expect("synergies.json 파싱 실패");

    let mut rng = rand::thread_rng();

    println!();
    print_header("무작위 총력전: 아수라장   (ARAM Brawl)");

    let champ = champions.choose(&mut rng).expect("챔피언 풀 비어있음");
    println!();
    println!("    ▸ 챔피언:  {}{}{}", BOLD, champ.name, RESET);
    println!("               {}{}{}", DIM, champ.title, RESET);

    let mut seen: HashSet<String> = HashSet::new();
    let mut picked: Vec<Augment> = Vec::with_capacity(8);

    for (round_idx, &level) in LEVELS.iter().enumerate() {
        let tier = random_tier(&mut rng);
        println!();
        println!(
            "    {}━━━ 라운드 {}/4   레벨 {}   ::  {}{}{} 증강 ━━━{}",
            BOLD, round_idx + 1, level,
            tier_color(tier), tier_label(tier), RESET, RESET
        );

        let initial_pool: Vec<&Augment> = augments.iter()
            .filter(|a| a.tier == tier && !seen.contains(&a.name))
            .collect();

        let mut slots: Vec<Augment> = initial_pool
            .choose_multiple(&mut rng, CHOICES_PER_PICK)
            .map(|a| (*a).clone())
            .collect();

        if slots.len() < CHOICES_PER_PICK {
            println!("    {}이 등급에 남은 증강이 부족합니다. 게임 종료.{}", DIM, RESET);
            return;
        }

        let mut reroll_used = [false; CHOICES_PER_PICK];
        let chosen;

        loop {
            println!();
            print_accumulated_synergies(&synergies, &picked);
            println!();
            print_three_cards(&slots, &reroll_used, &synergies);

            let inp = read_input("\n    선택 1·2·3 / 리롤 r1·r2·r3 / 종료 q: ");
            if inp == "q" {
                println!("\n    종료합니다.");
                return;
            }

            if let Ok(n) = inp.parse::<usize>() {
                if (1..=CHOICES_PER_PICK).contains(&n) {
                    chosen = slots[n - 1].clone();
                    for a in &slots { seen.insert(a.name.clone()); }
                    break;
                }
            }

            if let Some(rest) = inp.strip_prefix('r') {
                if let Ok(idx) = rest.parse::<usize>() {
                    if (1..=CHOICES_PER_PICK).contains(&idx) {
                        let i = idx - 1;
                        if reroll_used[i] {
                            println!("    {}이미 리롤한 슬롯입니다.{}", DIM, RESET);
                            continue;
                        }
                        seen.insert(slots[i].name.clone());
                        let pool2: Vec<&Augment> = augments.iter()
                            .filter(|a| a.tier == tier && !seen.contains(&a.name)
                                && !slots.iter().enumerate().any(|(j, x)| j != i && x.name == a.name))
                            .collect();
                        if let Some(&new_a) = pool2.choose(&mut rng) {
                            slots[i] = new_a.clone();
                            reroll_used[i] = true;
                        } else {
                            println!("    {}이 등급의 풀이 비었습니다.{}", DIM, RESET);
                        }
                        continue;
                    }
                }
            }
            println!("    {}잘못된 입력. 예: 1, 2, 3, r1, r2, r3, q{}", DIM, RESET);
        }

        println!(
            "\n    → {}[{}]{} {}{}{} 선택!",
            tier_color(&chosen.tier), tier_label(&chosen.tier), RESET,
            BOLD, chosen.name, RESET
        );

        if !handle_special(&chosen, &mut picked, &mut seen, &augments, &mut rng) {
            picked.push(chosen);
        }

        print_synergy_status(&synergies, &picked);
    }

    println!();
    print_header("최종 빌드");

    println!();
    println!("    챔피언:  {}{}{}", BOLD, champ.name, RESET);
    println!("             {}{}{}", DIM, champ.title, RESET);
    println!();
    println!("    증강:");
    for (i, a) in picked.iter().enumerate() {
        println!(
            "      {}{}. [{}{}{}]{}  {}{}{}",
            BOLD, i + 1,
            tier_color(&a.tier), tier_label(&a.tier), RESET, RESET,
            BOLD, a.name, RESET
        );
        for dl in wrap_text(&a.description, 58) {
            println!("         {}{}{}", DIM, dl, RESET);
        }
    }

    let progress = synergy_progress(&synergies, &picked);
    if !progress.is_empty() {
        println!();
        println!("    시너지:");
        for (s, c) in progress {
            let max_th = s.stages.last().map(|x| x.threshold).unwrap_or(0);
            if let Some(active) = current_stage(s, c) {
                println!(
                    "      {}◆ {}{}{}  ({}/{}, {}단계 활성){}",
                    GREEN, BOLD, s.name, RESET, c, max_th, active.threshold, RESET
                );
                for el in wrap_text(&active.effect, 58) {
                    println!("         {}{}{}", DIM, el, RESET);
                }
            } else {
                println!(
                    "      {}◇ {} ({}/{}, 미활성){}",
                    DIM, s.name, c, max_th, RESET
                );
            }
        }
    }
    println!();
}
