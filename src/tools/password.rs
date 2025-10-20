use crate::tool::{Output, Tool};
use anyhow::{bail, Result};
use clap::{Command, CommandFactory, Parser};
use rand::{Rng, rngs::OsRng, seq::SliceRandom};

#[derive(Parser, Debug)]
#[command(
    name = "password",
    about = "Generate secure passwords with various options",
    long_about = "Generate cryptographically secure passwords with customizable options.\n\
                  Supports random character-based passwords and memorable passphrases."
)]
pub struct PasswordTool {
    /// Length of the password to generate
    #[arg(long, short, default_value = "16")]
    length: usize,

    /// Number of passwords to generate
    #[arg(long, short = 'n', default_value = "1")]
    count: usize,

    /// Include uppercase letters (A-Z)
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    uppercase: bool,

    /// Include lowercase letters (a-z)
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    lowercase: bool,

    /// Include numbers (0-9)
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    numbers: bool,

    /// Include symbols (!@#$%^&*-_+=)
    #[arg(long, default_value = "true", action = clap::ArgAction::Set)]
    symbols: bool,

    /// Exclude ambiguous characters (0, O, l, I, 1, etc.)
    #[arg(long, default_value = "false")]
    no_ambiguous: bool,

    /// Generate memorable passphrase instead of random characters
    /// (e.g., "correct-horse-battery-staple")
    #[arg(long, conflicts_with_all = ["length", "uppercase", "lowercase", "numbers", "symbols", "no_ambiguous"])]
    memorable: bool,

    /// Number of words in the passphrase (only with --memorable)
    #[arg(long, default_value = "4", requires = "memorable")]
    words: usize,

    /// Separator for passphrase words (only with --memorable)
    #[arg(long, default_value = "-", requires = "memorable")]
    separator: String,

    /// Capitalize first letter of each word in passphrase (only with --memorable)
    #[arg(long, default_value = "false", requires = "memorable")]
    capitalize: bool,
}

impl Tool for PasswordTool {
    fn cli() -> Command {
        PasswordTool::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        if self.memorable {
            generate_memorable_passwords(self.count, self.words, &self.separator, self.capitalize)
        } else {
            generate_random_passwords(
                self.length,
                self.count,
                self.uppercase,
                self.lowercase,
                self.numbers,
                self.symbols,
                self.no_ambiguous,
            )
        }
    }
}

fn generate_random_passwords(
    length: usize,
    count: usize,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
    no_ambiguous: bool,
) -> Result<Option<Output>> {
    if length == 0 {
        bail!("Password length must be greater than 0");
    }

    if count == 0 {
        bail!("Count must be greater than 0");
    }

    // Build character set
    let mut charset = String::new();

    if uppercase {
        if no_ambiguous {
            charset.push_str("ABCDEFGHJKLMNPQRSTUVWXYZ"); // Exclude I, O
        } else {
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
    }

    if lowercase {
        if no_ambiguous {
            charset.push_str("abcdefghijkmnopqrstuvwxyz"); // Exclude l
        } else {
            charset.push_str("abcdefghijklmnopqrstuvwxyz");
        }
    }

    if numbers {
        if no_ambiguous {
            charset.push_str("23456789"); // Exclude 0, 1
        } else {
            charset.push_str("0123456789");
        }
    }

    if symbols {
        charset.push_str("!@#$%^&*-_+=");
    }

    if charset.is_empty() {
        bail!("At least one character set must be enabled");
    }

    let mut rng = OsRng;
    let charset_chars: Vec<char> = charset.chars().collect();
    let mut passwords = Vec::new();

    for _ in 0..count {
        let password = generate_secure_password(&mut rng, &charset_chars, length, uppercase, lowercase, numbers, symbols);
        let strength = calculate_strength(&password, length);
        
        passwords.push(serde_json::json!({
            "password": password,
            "length": length,
            "strength": strength,
            "entropy_bits": calculate_entropy(charset_chars.len(), length),
        }));
    }

    if count == 1 {
        Ok(Some(Output::JsonValue(passwords[0].clone())))
    } else {
        Ok(Some(Output::JsonValue(serde_json::json!(passwords))))
    }
}

fn generate_secure_password(
    rng: &mut OsRng,
    charset_chars: &[char],
    length: usize,
    has_uppercase: bool,
    has_lowercase: bool,
    has_numbers: bool,
    has_symbols: bool,
) -> String {
    // Generate password ensuring at least one character from each enabled set
    let mut password: Vec<char> = Vec::with_capacity(length);
    
    // Collect required character sets
    let mut required_chars = Vec::new();
    
    if has_uppercase {
        let uppercase: Vec<char> = charset_chars.iter()
            .filter(|c| c.is_uppercase())
            .copied()
            .collect();
        if !uppercase.is_empty() {
            required_chars.push(*uppercase.choose(rng).unwrap());
        }
    }
    
    if has_lowercase {
        let lowercase: Vec<char> = charset_chars.iter()
            .filter(|c| c.is_lowercase())
            .copied()
            .collect();
        if !lowercase.is_empty() {
            required_chars.push(*lowercase.choose(rng).unwrap());
        }
    }
    
    if has_numbers {
        let numbers: Vec<char> = charset_chars.iter()
            .filter(|c| c.is_numeric())
            .copied()
            .collect();
        if !numbers.is_empty() {
            required_chars.push(*numbers.choose(rng).unwrap());
        }
    }
    
    if has_symbols {
        let symbols: Vec<char> = charset_chars.iter()
            .filter(|c| !c.is_alphanumeric())
            .copied()
            .collect();
        if !symbols.is_empty() {
            required_chars.push(*symbols.choose(rng).unwrap());
        }
    }
    
    // Add required characters first
    password.extend(required_chars.iter());
    
    // Fill the rest randomly
    while password.len() < length {
        password.push(charset_chars[rng.gen_range(0..charset_chars.len())]);
    }
    
    // Shuffle to avoid predictable patterns
    password.shuffle(rng);
    
    password.into_iter().collect()
}

fn generate_memorable_passwords(
    count: usize,
    words: usize,
    separator: &str,
    capitalize: bool,
) -> Result<Option<Output>> {
    if words == 0 {
        bail!("Number of words must be greater than 0");
    }

    if count == 0 {
        bail!("Count must be greater than 0");
    }

    let mut rng = OsRng;
    let mut passwords = Vec::new();

    for _ in 0..count {
        let mut selected_words = Vec::new();

        for _ in 0..words {
            let word = WORDLIST.choose(&mut rng).unwrap();
            let word = if capitalize {
                capitalize_first(word)
            } else {
                word.to_string()
            };
            selected_words.push(word);
        }

        let passphrase = selected_words.join(separator);
        let strength = if words >= 6 {
            "very-strong"
        } else if words >= 5 {
            "strong"
        } else if words >= 4 {
            "good"
        } else {
            "moderate"
        };

        passwords.push(serde_json::json!({
            "password": passphrase,
            "type": "passphrase",
            "word_count": words,
            "strength": strength,
            "entropy_bits": calculate_entropy(WORDLIST.len(), words),
        }));
    }

    if count == 1 {
        Ok(Some(Output::JsonValue(passwords[0].clone())))
    } else {
        Ok(Some(Output::JsonValue(serde_json::json!(passwords))))
    }
}

fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn calculate_strength(password: &str, length: usize) -> &'static str {
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    let variety_count = [has_lower, has_upper, has_digit, has_symbol]
        .iter()
        .filter(|&&x| x)
        .count();

    match (length, variety_count) {
        (l, v) if l >= 16 && v >= 4 => "very-strong",
        (l, v) if l >= 12 && v >= 3 => "strong",
        (l, v) if l >= 11 && v >= 3 => "good",
        (l, v) if l >= 8 && v >= 3 => "moderate",
        (l, v) if l >= 8 && v >= 2 => "moderate",
        _ => "weak",
    }
}

fn calculate_entropy(charset_size: usize, length: usize) -> f64 {
    (charset_size as f64).log2() * length as f64
}

// EFF's short wordlist for passphrases - commonly used words that are easy to remember
const WORDLIST: &[&str] = &[
    "able", "acid", "aged", "also", "area", "army", "away", "baby", "back", "ball",
    "band", "bank", "base", "bath", "bear", "beat", "been", "beer", "bell", "belt",
    "best", "bill", "bird", "blow", "blue", "boat", "body", "bomb", "bond", "bone",
    "book", "boom", "born", "boss", "both", "bowl", "bulk", "burn", "bush", "busy",
    "cafe", "cage", "cake", "call", "calm", "came", "camp", "card", "care", "cart",
    "case", "cash", "cast", "cell", "chat", "chef", "chip", "city", "clay", "clip",
    "club", "coal", "coat", "code", "cold", "come", "cook", "cool", "cope", "copy",
    "cord", "core", "cost", "crab", "crew", "crop", "dark", "data", "date", "dawn",
    "days", "dead", "deal", "dean", "dear", "debt", "deep", "desk", "dial", "diet",
    "disc", "disk", "dock", "door", "dose", "down", "drag", "draw", "drew", "drop",
    "drug", "drum", "duck", "duke", "dust", "duty", "each", "earl", "earn", "ease",
    "east", "easy", "echo", "edge", "else", "even", "ever", "evil", "exit", "face",
    "fact", "fail", "fair", "fall", "fame", "farm", "fast", "fate", "fear", "feed",
    "feel", "feet", "fell", "felt", "file", "fill", "film", "find", "fine", "fire",
    "firm", "fish", "five", "flat", "flow", "folk", "food", "foot", "ford", "form",
    "fort", "four", "free", "from", "fuel", "full", "fund", "gain", "game", "gate",
    "gave", "gear", "gene", "gift", "girl", "give", "glad", "glen", "goal", "goes",
    "gold", "golf", "gone", "good", "gray", "grew", "grey", "grow", "gulf", "hair",
    "half", "hall", "hand", "hang", "hard", "harm", "hate", "have", "head", "hear",
    "heat", "held", "hell", "help", "hero", "high", "hill", "hire", "hold", "hole",
    "holy", "home", "hope", "horn", "host", "hour", "huge", "hung", "hunt", "hurt",
    "idea", "inch", "into", "iron", "isle", "item", "jack", "jane", "jazz", "john",
    "join", "jump", "june", "jury", "just", "keen", "keep", "kent", "kept", "kick",
    "kill", "kind", "king", "knee", "knew", "know", "lack", "lady", "laid", "lake",
    "land", "lane", "last", "late", "lead", "left", "lend", "lens", "less", "lied",
    "life", "lift", "like", "line", "link", "list", "live", "load", "loan", "lock",
    "logo", "long", "look", "lord", "lose", "loss", "lost", "love", "luck", "made",
    "mail", "main", "make", "male", "mall", "many", "mark", "mass", "matt", "meal",
    "mean", "meat", "meet", "menu", "mere", "mike", "mile", "milk", "mill", "mind",
    "mine", "miss", "mode", "mood", "moon", "more", "most", "move", "much", "must",
    "myth", "name", "navy", "near", "neck", "need", "news", "next", "nice", "nick",
    "nine", "none", "nose", "note", "okay", "once", "only", "onto", "open", "oral",
    "over", "pace", "pack", "page", "paid", "pain", "pair", "palm", "park", "part",
    "pass", "past", "path", "paul", "peak", "pick", "pile", "pine", "pink", "pipe",
    "plan", "play", "plot", "plug", "plus", "poem", "poet", "poll", "pool", "poor",
    "pope", "port", "post", "pour", "pray", "prep", "prev", "prey", "quit", "race",
    "rail", "rain", "rank", "rare", "rate", "read", "real", "rear", "rely", "rent",
    "rest", "rice", "rich", "ride", "ring", "rise", "risk", "road", "rock", "rode",
    "role", "roll", "roof", "room", "root", "rope", "rose", "rule", "rush", "ruth",
    "safe", "sage", "said", "sail", "sake", "sale", "salt", "same", "sand", "save",
    "seat", "seed", "seek", "seem", "seen", "self", "sell", "send", "sent", "sept",
    "ship", "shop", "shot", "show", "shut", "side", "sign", "sing", "site", "size",
    "skin", "slip", "slow", "snow", "soft", "soil", "sold", "sole", "some", "song",
    "soon", "sort", "soul", "spot", "star", "stay", "stem", "step", "stop", "such",
    "suit", "sure", "take", "tale", "talk", "tall", "tank", "tape", "task", "team",
    "tech", "tell", "tend", "term", "test", "text", "than", "that", "thee", "them",
    "then", "they", "thin", "this", "thus", "tide", "tied", "tier", "tile", "till",
    "time", "tiny", "tire", "told", "toll", "tone", "tony", "took", "tool", "tops",
    "torn", "tour", "town", "tree", "trip", "true", "tube", "tune", "turn", "twin",
    "type", "unit", "upon", "used", "user", "vary", "vast", "very", "vice", "view",
    "vote", "wage", "wait", "wake", "walk", "wall", "want", "ward", "warm", "warn",
    "wash", "wave", "ways", "weak", "wear", "week", "well", "went", "were", "west",
    "what", "when", "whom", "wide", "wife", "wild", "will", "wind", "wine", "wing",
    "wire", "wise", "wish", "with", "wood", "word", "wore", "work", "worn", "wrap",
    "yard", "year", "your", "zero", "zone",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_default_password() {
        let tool = PasswordTool {
            length: 16,
            count: 1,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            no_ambiguous: false,
            memorable: false,
            words: 4,
            separator: "-".to_string(),
            capitalize: false,
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let password = val["password"].as_str().unwrap();
        assert_eq!(password.len(), 16);
        assert!(val["strength"].as_str().is_some());
        assert!(val["entropy_bits"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_generate_multiple_passwords() {
        let tool = PasswordTool {
            length: 12,
            count: 5,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            no_ambiguous: false,
            memorable: false,
            words: 4,
            separator: "-".to_string(),
            capitalize: false,
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let passwords = val.as_array().unwrap();
        assert_eq!(passwords.len(), 5);
        
        // Check all passwords are unique
        let unique_passwords: std::collections::HashSet<_> = passwords
            .iter()
            .map(|p| p["password"].as_str().unwrap())
            .collect();
        assert_eq!(unique_passwords.len(), 5);
    }

    #[test]
    fn test_generate_no_ambiguous() {
        let tool = PasswordTool {
            length: 20,
            count: 1,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: false,
            no_ambiguous: true,
            memorable: false,
            words: 4,
            separator: "-".to_string(),
            capitalize: false,
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let password = val["password"].as_str().unwrap();
        
        // Check no ambiguous characters
        assert!(!password.contains('0'));
        assert!(!password.contains('O'));
        assert!(!password.contains('l'));
        assert!(!password.contains('I'));
        assert!(!password.contains('1'));
    }

    #[test]
    fn test_generate_only_lowercase() {
        let tool = PasswordTool {
            length: 10,
            count: 1,
            uppercase: false,
            lowercase: true,
            numbers: false,
            symbols: false,
            no_ambiguous: false,
            memorable: false,
            words: 4,
            separator: "-".to_string(),
            capitalize: false,
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let password = val["password"].as_str().unwrap();
        assert!(password.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_generate_no_character_sets() {
        let tool = PasswordTool {
            length: 10,
            count: 1,
            uppercase: false,
            lowercase: false,
            numbers: false,
            symbols: false,
            no_ambiguous: false,
            memorable: false,
            words: 4,
            separator: "-".to_string(),
            capitalize: false,
        };

        let result = tool.execute();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string().to_lowercase();
        assert!(error_msg.contains("at least one") || error_msg.contains("character set"));
    }

    #[test]
    fn test_generate_memorable_passphrase() {
        let tool = PasswordTool {
            length: 16,
            count: 1,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            no_ambiguous: false,
            memorable: true,
            words: 4,
            separator: "-".to_string(),
            capitalize: false,
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let password = val["password"].as_str().unwrap();
        assert_eq!(val["type"], "passphrase");
        assert_eq!(val["word_count"], 4);
        
        let word_count = password.split('-').count();
        assert_eq!(word_count, 4);
    }

    #[test]
    fn test_generate_memorable_with_custom_separator() {
        let tool = PasswordTool {
            length: 16,
            count: 1,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            no_ambiguous: false,
            memorable: true,
            words: 3,
            separator: "_".to_string(),
            capitalize: false,
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let password = val["password"].as_str().unwrap();
        assert!(password.contains('_'));
        assert_eq!(password.split('_').count(), 3);
    }

    #[test]
    fn test_generate_memorable_capitalized() {
        let tool = PasswordTool {
            length: 16,
            count: 1,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            no_ambiguous: false,
            memorable: true,
            words: 4,
            separator: "-".to_string(),
            capitalize: true,
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let password = val["password"].as_str().unwrap();
        let words: Vec<&str> = password.split('-').collect();
        
        // Each word should start with uppercase
        for word in words {
            assert!(word.chars().next().unwrap().is_uppercase());
        }
    }

    #[test]
    fn test_zero_length_error() {
        let tool = PasswordTool {
            length: 0,
            count: 1,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            no_ambiguous: false,
            memorable: false,
            words: 4,
            separator: "-".to_string(),
            capitalize: false,
        };

        let result = tool.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_strength() {
        assert_eq!(calculate_strength("abcdefgh", 8), "weak");
        assert_eq!(calculate_strength("Abcdefgh", 8), "moderate");
        assert_eq!(calculate_strength("Abcdef12", 8), "moderate");
        assert_eq!(calculate_strength("Abcdef12!@#", 11), "good");
        assert_eq!(calculate_strength("Abcdef12!@#$", 12), "strong");
        assert_eq!(calculate_strength("Abcdef12!@#$%^&*", 16), "very-strong");
    }

    #[test]
    fn test_calculate_entropy() {
        let entropy = calculate_entropy(26, 8); // lowercase only, 8 chars
        assert!(entropy > 0.0);
        
        let entropy2 = calculate_entropy(62, 16); // alphanumeric, 16 chars
        assert!(entropy2 > entropy);
    }

    #[test]
    fn test_password_has_required_character_types() {
        let tool = PasswordTool {
            length: 20,
            count: 1,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            no_ambiguous: false,
            memorable: false,
            words: 4,
            separator: "-".to_string(),
            capitalize: false,
        };

        let result = tool.execute().unwrap().unwrap();
        let Output::JsonValue(val) = result else {
            panic!("Expected JsonValue output");
        };

        let password = val["password"].as_str().unwrap();
        
        // Should have at least one of each type
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_numeric()));
        assert!(password.chars().any(|c| !c.is_alphanumeric()));
    }
}

