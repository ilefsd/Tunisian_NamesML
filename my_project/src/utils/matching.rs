// src/utils/matching.rs

use strsim::{jaro, levenshtein};
use crate::utils::linked_list::VariationNode;
use crate::utils::normalization::{normalize_arabic, remove_diacritics, standardize_prefixes};
use crate::utils::phonetic::aramix_soundex;

/// 🔠 Compare two already normalized strings with plain Jaro + normalized Levenshtein, plus a capped 20% Soundex bonus.
/// Soundex comparison uses its own normalization via `aramix_soundex`.
pub fn score_pair_with_soundex(norm_s1: &str, norm_s2: &str) -> f64 {
    // 1) Strings are assumed to be pre-normalized for Jaro/Levenshtein.
    // 2) Compute plain Jaro (no prefix‐boost) and normalized Levenshtein
    let j = jaro(norm_s1, norm_s2);
    let lev = 1.0 - (levenshtein(norm_s1, norm_s2)
        .min(norm_s1.len()) as f64
        / norm_s1.len().max(1) as f64);

    // 3) Combine Jaro+Lev into 80% of the score
    let base_score = ((j + lev) / 2.0) * 0.8;

    // 4) Add a flat 20% bonus if Soundex codes match.
    // `aramix_soundex` performs its own internal normalization suitable for phonetic coding.
    let bonus = if aramix_soundex(norm_s1) == aramix_soundex(norm_s2) {
        0.2
    } else {
        0.0
    };

    // 5) Final score, capped at 1.0
    (base_score + bonus).min(1.0)
}

/// Helper: average of phonetic match (0/1) and plain Jaro.
/// Assumes input strings `norm_a` and `norm_b` are pre-normalized for Jaro.
/// `aramix_soundex` handles its own normalization for the phonetic part.
pub fn combo(norm_a: &str, norm_b: &str) -> f32 {
    let p = (aramix_soundex(norm_a) == aramix_soundex(norm_b)) as u8 as f32;
    let j = jaro(norm_a, norm_b) as f32;
    (p + j) / 2.0
}

/// 🎯 Return the best score against the base string *and* all its variations.
/// `norm_input` is the pre-normalized input string from the request.
/// `norm_base` is the pre-normalized base string from the IdentityNode.
/// `variations` contain raw strings that need normalization before comparison.
pub fn best_score_against_variations(
    norm_input: &str, // Pre-normalized input string
    norm_base: &str,  // Pre-normalized base string from IdentityNode
    variations: &Option<Box<VariationNode>>,
) -> f64 {
    let mut best = score_pair_with_soundex(norm_input, norm_base);
    let mut current_variation_node = variations;
    while let Some(var_node) = current_variation_node {
        // Normalize the raw variation string before comparing
        let norm_variation = standardize_prefixes(&normalize_arabic(&remove_diacritics(&var_node.variation)));
        let s = score_pair_with_soundex(norm_input, &norm_variation);
        if s > best {
            best = s;
        }
        current_variation_node = &var_node.next_variation;
    }
    best
}

/// 🎯 Compute the weighted full‐record score.
/// Assumes `input_names` and `place1` are pre-normalized.
/// Assumes `target_names` and `place2` (from IdentityNode) are already normalized by the loader.
pub fn calculate_full_score(
    // These are pre-normalized strings from the input request
    input_norm_names: (&str, &str, &str, &str, &str, &str),
    // These are already normalized strings from the IdentityNode
    target_norm_names: (&str, &str, &str, &str, &str, &str),
    _variations: ( // Variations are handled by best_score_against_variations, not directly here
                   &Option<Box<VariationNode>>, &Option<Box<VariationNode>>, &Option<Box<VariationNode>>,
                   &Option<Box<VariationNode>>, &Option<Box<VariationNode>>, &Option<Box<VariationNode>>,
    ),
    dob1: Option<(u32, u32, u32)>,
    dob2: Option<(u32, u32, u32)>,
    // Pre-normalized place from input request
    place1_norm: &str,
    // Already normalized place from IdentityNode
    place2_norm: &str,
    _sex1: u8, // Sex doesn't require string normalization
    _sex2: u8,
) -> f64 {
    let (in_fn_norm, in_ln_norm, in_fa_norm, in_gd_norm, _in_ml_norm, in_m_norm) = input_norm_names;
    let (t_fn_norm,  t_ln_norm,  t_fa_norm,  t_gd_norm,  _lt_ml_norm,  t_m_norm ) = target_norm_names;

    // Fields are now assumed to be pre-normalized where necessary.
    // No more internal `norm = |s: &str| ...` calls for these inputs.

    // Weighted scoring
    let mut score = 0.0;
    let mut total = 0.0;

    // First name (35%) - uses combo, which expects normalized inputs
    score += combo(in_fn_norm, t_fn_norm) as f64 * 0.35;
    total += 0.35;

    // Last name (30%) - uses combo
    score += combo(in_ln_norm, t_ln_norm) as f64 * 0.30;
    total += 0.30;

    // Father name (10%) - uses jaro directly with normalized inputs
    score += jaro(in_fa_norm, t_fa_norm) * 0.10;
    total += 0.10;

    // Grandfather name (5%) - uses jaro
    score += jaro(in_gd_norm, t_gd_norm) * 0.05;
    total += 0.05;

    // Mother name (5%) - uses jaro
    score += jaro(in_m_norm, t_m_norm) * 0.05;
    total += 0.05;

    // DOB exact match (10%)
    if let (Some(d1), Some(d2)) = (dob1, dob2) {
        score += (d1 == d2) as u8 as f64 * 0.10;
    }
    total += 0.10;

    // Place of birth (5%) - uses jaro with normalized inputs
    score += jaro(place1_norm, place2_norm) * 0.05;
    total += 0.05;

    score / total
}

/// Pre‐filter candidates by sex, decade window, and phonetic last‐name.
/// `input_norm_ln` is the pre-normalized last name from the request.
/// `candidate_norm_ln` is the pre-normalized last name from the IdentityNode.
pub fn should_consider_candidate(
    input_details: &( // Contains pre-normalized last name
                      &str, &str, &str, &str, &str, &str, // other names not used by this function directly for filtering
                      Option<(u32, u32, u32)>, u8, &str // dob, sex, place (place not used for filtering)
    ),
    candidate_details: &( // Contains pre-normalized last name
                          &str, &str, &str, &str, &str, &str, // other names
                          Option<(u32, u32, u32)>, u8, &str // dob, sex, place
    ),
) -> bool {
    // Parameter names changed to reflect they are expected to be normalized for string fields
    let (_, input_norm_ln, _, _, _, _, in_dob, in_sex, _) = input_details;
    let (_, candidate_norm_ln, _, _, _, _, cand_dob, cand_sex, _) = candidate_details;

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

    // 3) Last-name Soundex must match.
    // `aramix_soundex` handles its own normalization.
    // The input strings `input_norm_ln` and `candidate_norm_ln` are passed directly.
    if aramix_soundex(input_norm_ln) != aramix_soundex(candidate_norm_ln) {
        return false;
    }

    true
}
