use serde::{Deserialize, Serialize};
use std::{
    io::{self, Write},
};
use rayon::prelude::*;

pub mod utils;
use utils::{
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
struct FieldScore {
    field: String,
    score: f64,
}

#[derive(Debug, Serialize)]
struct MatchResult {
    matched_identity: IdentityNodeSummary,
    total_score:      f64,
    breakdown:        Vec<FieldScore>,
}

/// A small serde‐friendly copy of IdentityNode for output
#[derive(Debug, Serialize)]
struct IdentityNodeSummary {
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

#[tokio::main]
async fn main() {
    // 1) Read user input first
    println!("▶ Enter the identity to match:");
    let input = read_identity_from_stdin();

    // 2) Compute the decade key
    let gen = input
        .dob
        .map(|(_, _, y)| generation_key(y as i32))
        .unwrap_or_else(|| generation_key(0));
    println!("🔍 Loading records for generation {}…", gen);

    // 3) Load only that slice from Postgres
    let records: Vec<IdentityNode> = load_identities_by_generation(gen).await;
    if records.is_empty() {
        println!("⚠️  No records found for generation {}.", gen);
        return;
    }
    println!("✅ Loaded {} records.", records.len());

    // 4) Pre-filter
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
    println!("✅ {} candidates after pre-filter.", candidates.len());
    if candidates.is_empty() {
        println!("No candidates passed the pre-filter. Try lowering your filter criteria.");
        return;
    }

    // 5) Score & sort
    let mut scored: Vec<MatchResult> = candidates
        .par_iter()
        .map(|id| {
            let mut breakdown = Vec::new();
            // name fields
            let fields = [
                ("الاسم الأول",   &input.first_name,    &id.first_name,    &id.first_name_variations),
                ("اسم العائلة",   &input.last_name,     &id.last_name,     &id.last_name_variations),
                ("اسم الأب",      &input.father_name,   &id.father_name,   &id.father_name_variations),
                ("اسم الجد",      &input.grandfather_name, &id.grandfather_name, &id.grandfather_name_variations),
                ("اسم عائلة الأم",&input.mother_last_name,&id.mother_last_name,&id.mother_last_name_variations),
                ("اسم الأم",      &input.mother_name,    &id.mother_name,    &id.mother_name_variations),
            ];
            for (label, inp, base, vars) in fields {
                let raw = best_score_against_variations(inp, base, vars) * 100.0_f64;
                breakdown.push(FieldScore { field: label.to_string(), score: raw.round() });
            }
            // DOB
            let dob_score: f64 = if let (Some((d1,m1,y1)), Some((d2,m2,y2)))=(input.dob,id.dob) {
                let mut s: f64 = 0.0;
                if d1==d2 { s+=1.0/3.0 }
                if m1==m2 { s+=1.0/3.0 }
                if y1==y2 { s+=1.0/3.0 }
                (s * 100.0_f64).round()
            } else { 0.0 };
            breakdown.push(FieldScore { field: "تاريخ الميلاد".into(), score: dob_score });
            // place
            let place_score = (score_pair_with_soundex(&input.place_of_birth,&id.place_of_birth)*100.0_f64).round();
            breakdown.push(FieldScore { field: "مكان الولادة".into(), score: place_score });
            // sex
            let sex_score = if input.sex==id.sex { 100.0 } else { 0.0 };
            breakdown.push(FieldScore { field: "الجنس".into(), score: sex_score });
            // total
            let raw_total = calculate_full_score(
                (
                    &input.first_name,&input.last_name,&input.father_name,
                    &input.grandfather_name,&input.mother_last_name,&input.mother_name
                ),
                (
                    &id.first_name,&id.last_name,&id.father_name,
                    &id.grandfather_name,&id.mother_last_name,&id.mother_name
                ),
                (
                    &id.first_name_variations,&id.last_name_variations,
                    &id.father_name_variations,&id.grandfather_name_variations,
                    &id.mother_last_name_variations,&id.mother_name_variations
                ),
                input.dob,id.dob,&input.place_of_birth,&id.place_of_birth,input.sex,id.sex
            ) * 100.0_f64;
            let total_score = raw_total.round();

            // build summary
            let dob_tuple = id.dob.unwrap_or((0,0,0));
            let summary = IdentityNodeSummary {
                first_name:      id.first_name.clone(),
                last_name:       id.last_name.clone(),
                father_name:     id.father_name.clone(),
                grandfather_name:id.grandfather_name.clone(),
                mother_last_name:id.mother_last_name.clone(),
                mother_name:     id.mother_name.clone(),
                dob:             dob_tuple,
                sex:             id.sex,
                place_of_birth:  id.place_of_birth.clone(),
            };

            MatchResult { matched_identity: summary, total_score, breakdown }
        })
        .collect();
    // sort descending
    scored.sort_by(|a,b| b.total_score.partial_cmp(&a.total_score).unwrap());

    // 6) Print top-3
    println!("\n▶ Top 3 matches:");
    for (i, m) in scored.into_iter().take(3).enumerate() {
        println!("Match #{} → {}%", i+1, m.total_score);
        for fs in &m.breakdown {
            println!("  {:<15} : {:>5.1}%", fs.field, fs.score);
        }
    }
}

/// Read the identity interactively
fn read_identity_from_stdin() -> InputIdentity {
    fn ask(prompt: &str) -> String {
        print!("{:>15}: ", prompt);
        io::stdout().flush().unwrap();
        let mut buf = String::new();
        io::stdin().read_line(&mut buf).unwrap();
        buf.trim().to_string()
    }

    let first = ask("first_name");
    let last = ask("last_name");
    let father = ask("father_name");
    let grand = ask("grandfather_name");
    let ml = ask("mother_last_name");
    let mom = ask("mother_name");

    let day = ask("dob day").parse().ok();
    let month = ask("dob month").parse().ok();
    let year = ask("dob year").parse().ok();
    let sex = ask("sex (1=M,2=F)").parse().unwrap_or(0);
    let place = ask("place_of_birth");

    InputIdentity {
        first_name: first.clone(),
        last_name: last.clone(),
        father_name: father.clone(),
        grandfather_name: grand.clone(),
        mother_last_name: ml.clone(),
        mother_name: mom.clone(),
        dob: match (day, month, year) {
            (Some(d), Some(m), Some(y)) => Some((d, m, y)),
            _ => None,
        },
        sex,
        place_of_birth: place.clone(),
    }
}