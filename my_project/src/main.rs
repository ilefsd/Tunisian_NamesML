// src/main.rs

use axum::{
    routing::post,
    extract::Json,
    http::StatusCode,
    Router, middleware as axum_middleware,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use rayon::prelude::*;

pub mod utils;
pub mod models;
pub mod db;
pub mod handlers;
pub mod middleware;

use crate::utils::{
    loader::{load_identities_by_generation, generation_key},
    matching::{
        should_consider_candidate,
        best_score_against_variations,
        score_pair_with_soundex,
        calculate_full_score,
    },
    normalization::{normalize_arabic, remove_diacritics, standardize_prefixes}, // Added for input normalization
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
    let pool = db::create_pool().await;
    db::init_db(&pool).await;

    // Public routes
    let public_routes = Router::new()
        .route("/api/register", post(handlers::register))
        .route("/api/login", post(handlers::login));
    

    // Protected routes
    let protected_routes = Router::new()
        .route("/match", post(match_identity))
        .route(
            "/api/usage/:user_id",
            axum::routing::get(handlers::get_api_usage),
        )
        .route(
            "/api/users",
            post(handlers::create_user).get(handlers::get_users),
        )
        .route(
            "/api/users/:id",
            axum::routing::put(handlers::update_user).delete(handlers::delete_user),
        )
        .route_layer(axum_middleware::from_fn(middleware::auth));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(axum_middleware::from_fn_with_state(
            pool.clone(),
            middleware::track_api_usage,
        ))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running on http://{}", addr);

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
    // --- Normalize input strings once ---
    let normalize_fn = |s: &str| standardize_prefixes(&normalize_arabic(&remove_diacritics(s)));

    let norm_input_first_name = normalize_fn(&input.first_name);
    let norm_input_last_name = normalize_fn(&input.last_name);
    let norm_input_father_name = normalize_fn(&input.father_name);
    let norm_input_grandfather_name = normalize_fn(&input.grandfather_name);
    let norm_input_mother_last_name = normalize_fn(&input.mother_last_name);
    let norm_input_mother_name = normalize_fn(&input.mother_name);
    let norm_input_place_of_birth = normalize_fn(&input.place_of_birth);
    // --- End of input normalization ---

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

    // 3) Pre-filter using normalized input
    let candidates: Vec<&IdentityNode> = records
        .iter()
        .filter(|id_node| { // Renamed `id` to `id_node` to avoid conflict if we destructure input later
            should_consider_candidate(
                &( // Pass normalized input fields
                   &norm_input_first_name,
                   &norm_input_last_name,
                   &norm_input_father_name,
                   &norm_input_grandfather_name,
                   &norm_input_mother_last_name,
                   &norm_input_mother_name,
                   input.dob, // DOB, sex are not strings, no normalization needed here
                   input.sex,
                   &norm_input_place_of_birth,
                ),
                &( // IdentityNode fields are already normalized by loader (for base names)
                   &id_node.first_name,
                   &id_node.last_name,
                   &id_node.father_name,
                   &id_node.grandfather_name,
                   &id_node.mother_last_name,
                   &id_node.mother_name,
                   id_node.dob,
                   id_node.sex,
                   &id_node.place_of_birth, // place_of_birth in IdentityNode is raw, but loader normalizes it for storage.
                   // For should_consider_candidate, it expects normalized if used,
                   // but current implementation only uses last_name for soundex.
                   // Let's assume id_node.place_of_birth is the normalized version as per loader.rs for consistency with other name fields.
                   // If id_node.place_of_birth was raw, it would need normalization here or inside should_consider_candidate.
                   // Given loader.rs normalizes all text fields it extracts for the IdentityNode main fields, this should be fine.
                ),
            )
        })
        .collect();
    println!("✅ {} candidates after pre-filter", candidates.len());
    if candidates.is_empty() {
        println!("⚠️ All records filtered out; returning empty result.");
        return (StatusCode::OK, Json(vec![]));
    }

    // 4) Score & sort using normalized input
    println!("▶ Scoring {} candidates in parallel…", candidates.len());
    let mut results: Vec<MatchResult> = candidates
        .par_iter()
        .map(|id_node| { // Renamed `id` to `id_node`
            let mut breakdown = Vec::new();

            // Name fields - use normalized input
            let fields_to_score = [
                ("الاسم الأول",    &norm_input_first_name,    &id_node.first_name,    &id_node.first_name_variations),
                ("اسم العائلة",    &norm_input_last_name,     &id_node.last_name,     &id_node.last_name_variations),
                ("اسم الأب",       &norm_input_father_name,   &id_node.father_name,   &id_node.father_name_variations),
                ("اسم الجد",       &norm_input_grandfather_name, &id_node.grandfather_name, &id_node.grandfather_name_variations),
                ("اسم عائلة الأم", &norm_input_mother_last_name, &id_node.mother_last_name, &id_node.mother_last_name_variations),
                ("اسم الأم",       &norm_input_mother_name,    &id_node.mother_name,    &id_node.mother_name_variations),
            ];
            for (label, norm_inp_field, id_base_field, id_vars) in fields_to_score {
                // best_score_against_variations expects normalized input and normalized base,
                // and handles normalization of raw variations internally.
                let raw_score = best_score_against_variations(norm_inp_field, id_base_field, id_vars) * 100.0_f64;
                breakdown.push(FieldScore { field: label.to_string(), score: raw_score.round() });
            }

            // DOB
            let dob_score: f64 = if let (Some((d1,m1,y1)), Some((d2,m2,y2))) = (input.dob, id_node.dob) {
                let mut s: f64 = 0.0;
                if d1==d2 { s+=1.0/3.0 }
                if m1==m2 { s+=1.0/3.0 }
                if y1==y2 { s+=1.0/3.0 }
                (s * 100.0_f64).round()
            } else { 0.0 };
            breakdown.push(FieldScore { field: "تاريخ الميلاد".into(), score: dob_score });

            // Place - use normalized input and normalized IdentityNode.place_of_birth
            // score_pair_with_soundex expects both inputs to be pre-normalized for Jaro/Lev,
            // and handles Soundex internal normalization.
            let place_score = (score_pair_with_soundex(&norm_input_place_of_birth, &id_node.place_of_birth) * 100.0_f64).round();
            breakdown.push(FieldScore { field: "مكان الولادة".into(), score: place_score });

            // Sex
            let sex_score = if input.sex == id_node.sex { 100.0 } else { 0.0 };
            breakdown.push(FieldScore { field: "الجنس".into(), score: sex_score });

            // Total - use normalized inputs
            let raw_total = calculate_full_score(
                ( // Normalized input names
                  &norm_input_first_name,
                  &norm_input_last_name,
                  &norm_input_father_name,
                  &norm_input_grandfather_name,
                  &norm_input_mother_last_name,
                  &norm_input_mother_name,
                ),
                ( // Normalized IdentityNode names (already normalized by loader)
                  &id_node.first_name,
                  &id_node.last_name,
                  &id_node.father_name,
                  &id_node.grandfather_name,
                  &id_node.mother_last_name,
                  &id_node.mother_name,
                ),
                ( // Variations (calculate_full_score doesn't use these directly, best_score_against_variations does)
                  &id_node.first_name_variations,
                  &id_node.last_name_variations,
                  &id_node.father_name_variations,
                  &id_node.grandfather_name_variations,
                  &id_node.mother_last_name_variations,
                  &id_node.mother_name_variations,
                ),
                input.dob,
                id_node.dob,
                &norm_input_place_of_birth, // Normalized input place
                &id_node.place_of_birth,   // Normalized IdentityNode place
                input.sex,
                id_node.sex,
            ) * 100.0_f64;
            let total_score = raw_total.round();

            let dob_tuple = id_node.dob.unwrap_or((0,0,0)); // id_node here
            let record = IdentityRecord {
                first_name:       id_node.first_name.clone(),
                last_name:        id_node.last_name.clone(),
                father_name:      id_node.father_name.clone(),
                grandfather_name: id_node.grandfather_name.clone(),
                mother_last_name: id_node.mother_last_name.clone(),
                mother_name:      id_node.mother_name.clone(),
                dob:              dob_tuple,
                sex:              id_node.sex,
                place_of_birth:   id_node.place_of_birth.clone(),
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
        .take(3) // Changed from 1 to 3
        .collect();
    println!("✅ Returning up to {} match(es) ≥ 75%.", filtered.len());

    (StatusCode::OK, Json(filtered))
}
