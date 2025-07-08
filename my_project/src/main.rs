// src/main.rs

use axum::{
    routing::post,
    extract::Json,
    http::StatusCode,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use rayon::prelude::*;

pub mod utils;

use crate::utils::{
    loader::{load_identities_by_generation, generation_key},
    matching::{
        should_consider_candidate,
        best_score_against_variations,
        score_pair_with_soundex,
        calculate_full_score,
    },
    linked_list::IdentityNode,
};

#[derive(Debug, Deserialize)]
struct InputIdentity {
    first_name:       String,
    last_name:        String,
    father_name:      String,
    grandfather_name: String,
    mother_last_name: String,
    mother_name:      String,
    dob:              Option<(u32, u32, u32)>,
    sex:              u8,
    place_of_birth:   String,
}

#[derive(Debug, Serialize)]
struct IdentityRecord {
    first_name:       String,
    last_name:        String,
    father_name:      String,
    grandfather_name: String,
    mother_last_name: String,
    mother_name:      String,
    dob:              (u32, u32, u32),
    sex:              u8,
    place_of_birth:   String,
}

#[derive(Debug, Serialize)]
struct FieldScore {
    field: String,
    score: f64,
}

#[derive(Debug, Serialize)]
struct MatchResult {
    matched_identity: IdentityRecord,
    total_score:      f64,
    breakdown:        Vec<FieldScore>,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/match", post(match_identity));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running on http://{}", addr);

    // Use TcpListener so we can reuse your existing pattern
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn match_identity(
    Json(input): Json<InputIdentity>,
) -> (StatusCode, Json<Vec<MatchResult>>) {
    // 1) Compute decade key
    let gen = input
        .dob
        .map(|(_, _, y)| generation_key(y as i32))
        .unwrap_or_else(|| generation_key(0));
    println!("🔍 Computed generation key: {}", gen);

    // 2) Load only that decade
    println!("🔍 Connecting to PostgreSQL to load generation {}…", gen);
    let records: Vec<IdentityNode> = load_identities_by_generation(gen).await;
    println!("✅ {} rows in generation {}", records.len(), gen);
    if records.is_empty() {
        println!("⚠️ No records found for generation {}; aborting.", gen);
        return (StatusCode::NOT_FOUND, Json(vec![]));
    }

    // 3) Pre-filter
    let candidates: Vec<&IdentityNode> = records
        .iter()
        .filter(|id| {
            should_consider_candidate(
                &(
                    &input.first_name,
                    &input.last_name,
                    &input.father_name,
                    &input.grandfather_name,
                    &input.mother_last_name,
                    &input.mother_name,
                    input.dob,
                    input.sex,
                    &input.place_of_birth,
                ),
                &(
                    &id.first_name,
                    &id.last_name,
                    &id.father_name,
                    &id.grandfather_name,
                    &id.mother_last_name,
                    &id.mother_name,
                    id.dob,
                    id.sex,
                    &id.place_of_birth,
                ),
            )
        })
        .collect();
    println!("✅ {} candidates after pre-filter", candidates.len());
    if candidates.is_empty() {
        println!("⚠️ All records filtered out; returning empty result.");
        return (StatusCode::OK, Json(vec![]));
    }

    // 4) Score & sort
    println!("▶ Scoring {} candidates in parallel…", candidates.len());
    let mut results: Vec<MatchResult> = candidates
        .par_iter()
        .map(|id| {
            let mut breakdown = Vec::new();

            // Name fields
            let fields = [
                ("الاسم الأول",    &input.first_name,    &id.first_name,    &id.first_name_variations),
                ("اسم العائلة",    &input.last_name,     &id.last_name,     &id.last_name_variations),
                ("اسم الأب",       &input.father_name,   &id.father_name,   &id.father_name_variations),
                ("اسم الجد",       &input.grandfather_name, &id.grandfather_name, &id.grandfather_name_variations),
                ("اسم عائلة الأم", &input.mother_last_name, &id.mother_last_name, &id.mother_last_name_variations),
                ("اسم الأم",       &input.mother_name,    &id.mother_name,    &id.mother_name_variations),
            ];
            for (label, inp, base, vars) in fields {
                let raw = best_score_against_variations(inp, base, vars) * 100.0_f64;
                breakdown.push(FieldScore { field: label.to_string(), score: raw.round() });
            }

            // DOB
            let dob_score: f64 = if let (Some((d1,m1,y1)), Some((d2,m2,y2))) = (input.dob, id.dob) {
                let mut s: f64 = 0.0;
                if d1==d2 { s+=1.0/3.0 }
                if m1==m2 { s+=1.0/3.0 }
                if y1==y2 { s+=1.0/3.0 }
                (s * 100.0_f64).round()
            } else { 0.0 };
            breakdown.push(FieldScore { field: "تاريخ الميلاد".into(), score: dob_score });

            // Place
            let place_score = (score_pair_with_soundex(&input.place_of_birth, &id.place_of_birth) * 100.0_f64).round();
            breakdown.push(FieldScore { field: "مكان الولادة".into(), score: place_score });

            // Sex
            let sex_score = if input.sex == id.sex { 100.0 } else { 0.0 };
            breakdown.push(FieldScore { field: "الجنس".into(), score: sex_score });

            // Total
            let raw_total = calculate_full_score(
                (
                    &input.first_name,
                    &input.last_name,
                    &input.father_name,
                    &input.grandfather_name,
                    &input.mother_last_name,
                    &input.mother_name,
                ),
                (
                    &id.first_name,
                    &id.last_name,
                    &id.father_name,
                    &id.grandfather_name,
                    &id.mother_last_name,
                    &id.mother_name,
                ),
                (
                    &id.first_name_variations,
                    &id.last_name_variations,
                    &id.father_name_variations,
                    &id.grandfather_name_variations,
                    &id.mother_last_name_variations,
                    &id.mother_name_variations,
                ),
                input.dob,
                id.dob,
                &input.place_of_birth,
                &id.place_of_birth,
                input.sex,
                id.sex,
            ) * 100.0_f64;
            let total_score = raw_total.round();

            let dob_tuple = id.dob.unwrap_or((0,0,0));
            let record = IdentityRecord {
                first_name:       id.first_name.clone(),
                last_name:        id.last_name.clone(),
                father_name:      id.father_name.clone(),
                grandfather_name: id.grandfather_name.clone(),
                mother_last_name: id.mother_last_name.clone(),
                mother_name:      id.mother_name.clone(),
                dob:              dob_tuple,
                sex:              id.sex,
                place_of_birth:   id.place_of_birth.clone(),
            };

            MatchResult { matched_identity: record, total_score, breakdown }
        })
        .collect();

    // *** Sort by descending total_score so take(1) is the highest match ***
    results.sort_unstable_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());

    println!("✅ Scoring done ({} results).", results.len());

    // 5) Threshold & return top-1
    let filtered: Vec<MatchResult> = results
        .into_iter()
        .filter(|r| r.total_score >= 75.0)
        .take(1)
        .collect();
    println!("✅ Returning {} match(es) ≥ 75%.", filtered.len());

    (StatusCode::OK, Json(filtered))
}
