// src/utils/matching.rs

use strsim::{jaro, levenshtein};
use crate::utils::linked_list::VariationNode;
use crate::utils::normalization::{normalize_arabic, remove_diacritics, standardize_prefixes};
use crate::utils::phonetic::aramix_soundex;

/// 🔠 Compare two strings with plain Jaro + normalized Levenshtein, plus a capped 20% Soundex bonus
pub fn score_pair_with_soundex(s1: &str, s2: &str) -> f64 {
    // 1) Normalize both inputs
    let norm1 = standardize_prefixes(&normalize_arabic(&remove_diacritics(s1)));
    let norm2 = standardize_prefixes(&normalize_arabic(&remove_diacritics(s2)));

    // 2) Compute plain Jaro (no prefix‐boost) and normalized Levenshtein
    let j = jaro(&norm1, &norm2);
    let lev = 1.0 - (levenshtein(&norm1, &norm2)
        .min(norm1.len()) as f64
        / norm1.len().max(1) as f64);

    // 3) Combine Jaro+Lev into 80% of the score
    let base_score = ((j + lev) / 2.0) * 0.8;

    // 4) Add a flat 20% bonus if Soundex codes match
    let bonus = if aramix_soundex(&norm1) == aramix_soundex(&norm2) {
        0.2
    } else {
        0.0
    };

    // 5) Final score, capped at 1.0
    (base_score + bonus).min(1.0)
}

/// Helper: average of phonetic match (0/1) and plain Jaro
pub fn combo(a: &str, b: &str) -> f32 {
    let norm_a = standardize_prefixes(&normalize_arabic(&remove_diacritics(a)));
    let norm_b = standardize_prefixes(&normalize_arabic(&remove_diacritics(b)));

    let p = (aramix_soundex(&norm_a) == aramix_soundex(&norm_b)) as u8 as f32;
    let j = jaro(&norm_a, &norm_b) as f32;
    (p + j) / 2.0
}

/// 🎯 Return the best score against the base string *and* all its variations
pub fn best_score_against_variations(
    input: &str,
    base: &str,
    variations: &Option<Box<VariationNode>>,
) -> f64 {
    let mut best = score_pair_with_soundex(input, base);
    let mut current = variations;
    while let Some(var) = current {
        let s = score_pair_with_soundex(input, &var.variation);
        if s > best {
            best = s;
        }
        current = &var.next_variation;
    }
    best
}

/// 🎯 Compute the weighted full‐record score
pub fn calculate_full_score(
    input_names: (&str, &str, &str, &str, &str, &str),
    target_names: (&str, &str, &str, &str, &str, &str),
    variations: (
        &Option<Box<VariationNode>>, &Option<Box<VariationNode>>, &Option<Box<VariationNode>>,
        &Option<Box<VariationNode>>, &Option<Box<VariationNode>>, &Option<Box<VariationNode>>,
    ),
    dob1: Option<(u32, u32, u32)>,
    dob2: Option<(u32, u32, u32)>,
    place1: &str,
    place2: &str,
    _sex1: u8,
    _sex2: u8,
) -> f64 {
    let (in_fn, in_ln, in_fa, in_gd, in_ml, in_m) = input_names;
    let (t_fn,  t_ln,  t_fa,  t_gd,  lt_ml,  t_m ) = target_names;

    // Normalize fields once
    let norm = |s: &str| standardize_prefixes(&normalize_arabic(&remove_diacritics(s)));
    let in_fn_norm = norm(in_fn);
    let in_ln_norm = norm(in_ln);
    let in_fa_norm = norm(in_fa);
    let in_gd_norm = norm(in_gd);
    let in_m_norm  = norm(in_m);
    let place1_norm = norm(place1);

    let t_fn_norm = norm(t_fn);
    let t_ln_norm = norm(t_ln);
    let t_fa_norm = norm(t_fa);
    let t_gd_norm = norm(t_gd);
    let t_m_norm  = norm(t_m);
    let place2_norm = norm(place2);

    // Weighted scoring
    let mut score = 0.0;
    let mut total = 0.0;

    // First name (35%)
    score += combo(&in_fn_norm, &t_fn_norm) as f64 * 0.35;
    total += 0.35;

    // Last name (30%)
    score += combo(&in_ln_norm, &t_ln_norm) as f64 * 0.30;
    total += 0.30;

    // Father name (10%)
    score += jaro(&in_fa_norm, &t_fa_norm) * 0.10;
    total += 0.10;

    // Grandfather name (5%)
    score += jaro(&in_gd_norm, &t_gd_norm) * 0.05;
    total += 0.05;

    // Mother name (5%)
    score += jaro(&in_m_norm, &t_m_norm) * 0.05;
    total += 0.05;

    // DOB exact match (10%)
    if let (Some(d1), Some(d2)) = (dob1, dob2) {
        score += (d1 == d2) as u8 as f64 * 0.10;
    }
    total += 0.10;

    // Place of birth (5%)
    score += jaro(&place1_norm, &place2_norm) * 0.05;
    total += 0.05;

    score / total
}

/// Pre‐filter candidates by sex, decade window, and phonetic last‐name
pub fn should_consider_candidate(
    input: &(
        &str, &str, &str, &str, &str, &str,
        Option<(u32, u32, u32)>, u8, &str
    ),
    candidate: &(
        &str, &str, &str, &str, &str, &str,
        Option<(u32, u32, u32)>, u8, &str
    ),
) -> bool {
    let (_, in_ln, _, _, _, _, in_dob, in_sex, _) = input;
    let (_, cand_ln, _, _, _, _, cand_dob, cand_sex, _) = candidate;

    // 1) Sex must match
    if in_sex != cand_sex {
        return false;
    }

    // 2) Birth-year within ±10 years
    if let (Some((_,_,y1)), Some((_,_,y2))) = (*in_dob, *cand_dob) {
        if (y1 as i32 - y2 as i32).abs() > 10 {
            return false;
        }
    }

    // 3) Last-name Soundex must match
    let norm_last = |s: &str| standardize_prefixes(&normalize_arabic(&remove_diacritics(s)));
    if aramix_soundex(&norm_last(in_ln)) != aramix_soundex(&norm_last(cand_ln)) {
        return false;
    }

    true
}
