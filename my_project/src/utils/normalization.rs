// 📌 Enhanced Normalization for Arabic Names
use regex::Regex;

/// Extract potential named entities (name, location, date) from input
pub fn extract_named_entities(text: &str) -> (Option<String>, Option<String>, Option<String>) {
    let name_regex = Regex::new(r"(?i)\b([A-Za-z\u0600-\u06FF]+)\b").unwrap(); // Arabic & Latin words
    let date_regex = Regex::new(r"\b(\d{2}/\d{2}/\d{4})\b").unwrap(); // Dates in dd/mm/yyyy format
    let location_keywords = [
        "تونس", "صفاقس", "بنزرت", "سوسة", "نابل",
        "أريانة", "منوبة", "بن عروس", "المنستير", "المهدية",
        "الحمامات", "القيروان", "زغوان", "مدنين", "تطاوين",
        "توزر", "قبلي", "قفصة", "قابس", "جندوبة",
        "الكاف", "باجة", "سليانة", "القصرين"
    ]; // Tunisian locations

    let mut name = None;
    let mut date = None;
    let mut place = None;

    for cap in name_regex.captures_iter(text) {
        let entity = cap[0].to_string();
        if location_keywords.contains(&entity.as_str()) {
            place = Some(entity.clone());
        } else {
            name = Some(entity);
        }
    }

    if let Some(date_match) = date_regex.captures(text) {
        date = Some(date_match[0].to_string());
    }

    (name, date, place)
}

pub fn normalize_arabic(text: &str) -> String {
    text.replace("ة", "ه")  // Convert "ة" to "ه"
        .replace("ى", "ي")  // Convert "ى" to "ي"
        .replace("أ", "ا")  // Normalize Hamza variations
        .replace("إ", "ا")  // Normalize Hamza variations
        .replace("ؤ", "و")  // Convert Hamza-on-Waw to Waw
        .replace("ئ", "ي")  // Convert Hamza-on-Ya to Ya
        .replace("ﻷ", "لا")  // Normalize Lam-Alef ligature
        .replace("ﻵ", "لا")  // Normalize Lam-Alef ligature
        .replace("ﻹ", "لا")  // Normalize Lam-Alef ligature
        .replace("ﻻ", "لا")  // Normalize Lam-Alef ligature
}

pub fn remove_diacritics(text: &str) -> String {
    let diacritics = ["َ", "ً", "ُ", "ٌ", "ِ", "ٍ", "ّ", "ْ"];
    let mut result = text.to_string();
    for dia in diacritics.iter() {
        result = result.replace(dia, "");
    }
    result
}

pub fn standardize_prefixes(text: &str) -> String {
    let prefixes = ["ال", "بن", "ابن", "بنت", "أبو", "أم"];
    let mut normalized = text.to_string();

    for prefix in prefixes.iter() {
        if normalized.starts_with(prefix) {
            normalized = normalized[prefix.len()..].to_string();
            break; // Remove only the first matching prefix
        }
    }

    normalized
}
