use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls, Row};
use crate::utils::linked_list::IdentityNode;
use crate::utils::normalization::{normalize_arabic, remove_diacritics, standardize_prefixes};

/// Group birth years into decades (e.g. 1985 → 1980)
pub fn generation_key(year: i32) -> i32 {
    (year / 10) * 10
}

/// Load *only* the identities for a given decade (e.g. 1980s → 1980)
pub async fn load_identities_by_generation(gen: i32) -> Vec<IdentityNode> {
    println!("🔍 Connecting to PostgreSQL to load generation {}…", gen);

    // 1) Setup BB8 pool
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost port=5432 user=postgres password=9155 dbname=tunisian_citizens",
        NoTls,
    ).expect("Invalid connection string");

    let pool: Pool<PostgresConnectionManager<NoTls>> = Pool::builder()
        .max_size(10)
        .build(manager)
        .await
        .expect("Failed to build pool");

    let conn = pool.get().await.expect("Failed to get connection");

    // 2) Fetch only that decade
    let sql = r#"
        SELECT
            الاسم, اسم_العائلة, اسم_الأب, اسم_الجد,
            اسم_عائلة_الأم, اسم_الأم,
            يوم_الميلاد, شهر_الميلاد, سنة_الميلاد,
            الجنس, مكان_الولادة
        FROM tunisian_citizens
        WHERE (سنة_الميلاد / 10) * 10 = $1
    "#;
    println!("🔎 Executing decade query…");
    let rows: Vec<Row> = conn
        .query(sql, &[&gen])
        .await
        .expect("Query failed");

    println!("✅ {} rows in generation {}", rows.len(), gen);

    // 3) Parse & normalize into a flat Vec<IdentityNode>
    let normalize = |s: &str| {
        let s = remove_diacritics(s);
        let s = normalize_arabic(&s);
        standardize_prefixes(&s)
    };

    rows.into_iter().filter_map(|row| {
        // extract, skip row if any required field is missing
        let first    = row.try_get::<_, String>("الاسم").ok()?;
        let last     = row.try_get::<_, String>("اسم_العائلة").ok()?;
        let father   = row.try_get::<_, String>("اسم_الأب").ok()?;
        let grandpa  = row.try_get::<_, String>("اسم_الجد").ok()?;
        let mom_last = row.try_get::<_, String>("اسم_عائلة_الأم").ok()?;
        let mom      = row.try_get::<_, String>("اسم_الأم").ok()?;
        let day      = row.try_get::<_, i32>("يوم_الميلاد").ok()? as u32;
        let mon      = row.try_get::<_, i32>("شهر_الميلاد").ok()? as u32;
        let year     = row.try_get::<_, i32>("سنة_الميلاد").ok()? as u32;
        let gender   = row.try_get::<_, String>("الجنس").ok()?;
        let place    = row.try_get::<_, String>("مكان_الولادة").ok()?;

        // map gender to u8
        let sex = match gender.as_str() {
            "1" | "ذكر"   => 1,
            "2" | "أنثى"  => 2,
            _             => 0,
        };

        // normalized bases
        let base_first      = normalize(&first);
        let base_last       = normalize(&last);
        let base_father     = normalize(&father);
        let base_grandpa    = normalize(&grandpa);
        let base_mom_last   = normalize(&mom_last);
        let base_mom        = normalize(&mom);

        // build IdentityNode
        Some(IdentityNode {
            first_name:      base_first,
            last_name:       base_last,
            father_name:     base_father,
            grandfather_name: base_grandpa,
            mother_last_name: base_mom_last,
            mother_name:     base_mom,
            dob:             Some((day, mon, year)),
            sex,
            place_of_birth:  place.clone(),

            // single‐entry variation lists: just the raw original text
            first_name_variations:      Some(Box::new(crate::utils::linked_list::VariationNode { variation: first, next_variation: None })),
            last_name_variations:       Some(Box::new(crate::utils::linked_list::VariationNode { variation: last, next_variation: None })),
            father_name_variations:     Some(Box::new(crate::utils::linked_list::VariationNode { variation: father, next_variation: None })),
            grandfather_name_variations:Some(Box::new(crate::utils::linked_list::VariationNode { variation: grandpa, next_variation: None })),
            mother_last_name_variations:Some(Box::new(crate::utils::linked_list::VariationNode { variation: mom_last, next_variation: None })),
            mother_name_variations:     Some(Box::new(crate::utils::linked_list::VariationNode { variation: mom, next_variation: None })),

            next_identity: None,
        })
    }).collect()
}
