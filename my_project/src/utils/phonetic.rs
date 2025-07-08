/// 🔠 Normalise les lettres arabes (alif, ya, waw...) + enlève les diacritiques
pub fn normalize_arabic_letters(input: &str) -> String {
    input
        .replace('أ', "ا")
        .replace('إ', "ا")
        .replace('آ', "ا")
        .replace('ى', "ي")
        .replace('ئ', "ي")
        .replace('ؤ', "و")
        .replace('ة', "ه")
        .replace('ء', "")
        .replace('َ', "")
        .replace('ً', "")
        .replace('ُ', "")
        .replace('ٌ', "")
        .replace('ِ', "")
        .replace('ٍ', "")
        .replace('ْ', "")
        .replace('ّ', "")
}

/// 🔊 Encode un nom arabe avec un Soundex personnalisé (Aramix Soundex)
pub fn get_code(name: &str) -> String {
    let mut code = String::new();
    let mut last_digit = '0';

    if let Some(first_letter) = name.chars().next() {
        code.push(first_letter);
    }

    for c in name.chars().skip(1) {
        let digit = match c {
            'ب' | 'ف' => '1',
            'ج' | 'ك' | 'ق' => '2',
            'د' | 'ت' | 'ض' => '3',
            'ر' | 'ل' | 'ن' => '4',
            'س' | 'ش' | 'ز' => '5',
            'ط' | 'ظ' | 'ص' => '6',
            'ع' | 'غ' | 'ح' => '7',
            'خ' | 'ه' => '8',
            'م' | 'و' => '9',
            _ => '0',
        };

        if digit != '0' && digit != last_digit {
            code.push(digit);
            last_digit = digit;
        }
    }

    code.truncate(4);
    code
}

/// 🎯 Point d’entrée unique : normalisation + Soundex
pub fn aramix_soundex(name: &str) -> String {
    let normalized = normalize_arabic_letters(name);
    get_code(&normalized)
}
