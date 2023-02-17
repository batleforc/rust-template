use regex::Regex;

pub fn validate_email(email: String) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    re.is_match(&email)
}

// validate password
// 1. length >= 3
// 2. length <= 20
// 3. at least one number
// 4. at least one lowercase letter
// 5. at least one uppercase letter
pub fn validate_password(password: String) -> bool {
    if password.len() < 3 || password.len() > 20 {
        return false;
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return false;
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return false;
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return false;
    }
    return true;
}

// validate name and surname
// 1. length >= 2
pub fn validate_name(name: String) -> bool {
    let re = Regex::new(r"^[a-zA-Z-\s]{2,}$").unwrap();
    re.is_match(&name)
}
