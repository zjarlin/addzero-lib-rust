use crate::{KiroAuthSupportError, KiroAuthSupportResult};
use az_derive_aliases::{apply, impl_default, plain_copy_eq, plain_default_copy_eq, plain_eq};
use ring::rand::{SecureRandom, SystemRandom};

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const DEFAULT_SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

const MALE_NAMES: &[&str] = &[
    "James",
    "John",
    "Robert",
    "Michael",
    "William",
    "David",
    "Richard",
    "Joseph",
    "Thomas",
    "Charles",
    "Christopher",
    "Daniel",
    "Matthew",
    "Anthony",
    "Donald",
    "Mark",
    "Paul",
    "Steven",
    "Andrew",
    "Kenneth",
    "Joshua",
    "Kevin",
    "Brian",
    "George",
    "Edward",
    "Ronald",
    "Timothy",
    "Jason",
    "Jeffrey",
    "Ryan",
    "Jacob",
    "Gary",
    "Nicholas",
    "Eric",
    "Jonathan",
    "Stephen",
    "Larry",
    "Justin",
    "Scott",
    "Brandon",
    "Benjamin",
    "Samuel",
    "Raymond",
    "Gregory",
    "Frank",
    "Alexander",
    "Patrick",
    "Jack",
    "Dennis",
    "Jerry",
];

const FEMALE_NAMES: &[&str] = &[
    "Mary",
    "Patricia",
    "Jennifer",
    "Linda",
    "Barbara",
    "Elizabeth",
    "Susan",
    "Jessica",
    "Sarah",
    "Karen",
    "Lisa",
    "Nancy",
    "Betty",
    "Margaret",
    "Sandra",
    "Ashley",
    "Kimberly",
    "Emily",
    "Donna",
    "Michelle",
    "Dorothy",
    "Carol",
    "Amanda",
    "Melissa",
    "Deborah",
    "Stephanie",
    "Rebecca",
    "Sharon",
    "Laura",
    "Cynthia",
    "Kathleen",
    "Amy",
    "Shirley",
    "Angela",
    "Helen",
    "Anna",
    "Brenda",
    "Pamela",
    "Nicole",
    "Emma",
    "Samantha",
    "Katherine",
    "Christine",
    "Debra",
    "Rachel",
    "Catherine",
    "Carolyn",
    "Janet",
    "Ruth",
    "Maria",
];

const SURNAMES: &[&str] = &[
    "Smith",
    "Johnson",
    "Williams",
    "Brown",
    "Jones",
    "Garcia",
    "Miller",
    "Davis",
    "Rodriguez",
    "Martinez",
    "Hernandez",
    "Lopez",
    "Gonzalez",
    "Wilson",
    "Anderson",
    "Thomas",
    "Taylor",
    "Moore",
    "Jackson",
    "Martin",
    "Lee",
    "Perez",
    "Thompson",
    "White",
    "Harris",
    "Sanchez",
    "Clark",
    "Ramirez",
    "Lewis",
    "Robinson",
    "Walker",
    "Young",
    "Allen",
    "King",
    "Wright",
    "Scott",
    "Torres",
    "Nguyen",
    "Hill",
    "Flores",
    "Green",
    "Adams",
    "Nelson",
    "Baker",
    "Hall",
    "Rivera",
    "Campbell",
    "Mitchell",
    "Carter",
    "Roberts",
];

/// Gender preference for generated English first names.
#[apply(plain_default_copy_eq)]
pub enum NameGender {
    /// Pick from the male-name pool.
    Male,
    /// Pick from the female-name pool.
    Female,
    /// Pick from the combined pool.
    #[default]
    Random,
}

/// Options for local English name generation.
#[apply(plain_copy_eq)]
pub struct EnglishNameOptions {
    /// Whether to include a surname.
    pub full_name: bool,
    /// First-name pool preference.
    pub gender: NameGender,
}

impl_default!(EnglishNameOptions => EnglishNameOptions {
    full_name: true,
    gender: NameGender::Random,
});

/// Generated English name parts.
#[apply(plain_eq)]
pub struct EnglishName {
    /// First name.
    pub first_name: String,
    /// Optional surname.
    pub last_name: Option<String>,
}

impl EnglishName {
    /// Returns the display form used in registration pages.
    #[must_use]
    pub fn display_name(&self) -> String {
        match self.last_name.as_deref() {
            Some(last_name) => format!("{} {last_name}", self.first_name),
            None => self.first_name.clone(),
        }
    }
}

/// Password generation policy matching Kiro/AWS Builder ID complexity needs.
#[apply(plain_eq)]
pub struct PasswordPolicy {
    /// Requested length. Values are clamped to `8..=64`.
    pub length: usize,
    /// Allowed special symbols.
    pub symbols: String,
}

impl_default!(PasswordPolicy => PasswordPolicy {
    length: 12,
    symbols: String::from_utf8(DEFAULT_SYMBOLS.to_vec())
        .expect("default symbols should be ASCII"),
});

/// Generates a random English name from the embedded first-name and surname pools.
pub fn generate_english_name(options: EnglishNameOptions) -> KiroAuthSupportResult<EnglishName> {
    let first_pool = match options.gender {
        NameGender::Male => MALE_NAMES,
        NameGender::Female => FEMALE_NAMES,
        NameGender::Random => {
            return generate_random_gender_name(options.full_name);
        }
    };
    let first_name = choose(first_pool)?.to_owned();
    let last_name = if options.full_name {
        Some(choose(SURNAMES)?.to_owned())
    } else {
        None
    };
    Ok(EnglishName {
        first_name,
        last_name,
    })
}

/// Generates a password with lowercase, uppercase, digit, and symbol coverage.
pub fn generate_password(policy: PasswordPolicy) -> KiroAuthSupportResult<String> {
    let length = policy.length.clamp(8, 64);
    let symbols = policy.symbols.as_bytes();
    if symbols.is_empty() {
        return Err(KiroAuthSupportError::InvalidConfig(
            "symbols cannot be empty".to_owned(),
        ));
    }

    let mut chars = Vec::with_capacity(length);
    chars.push(random_char(LOWER)?);
    chars.push(random_char(UPPER)?);
    chars.push(random_char(DIGITS)?);
    chars.push(random_char(symbols)?);

    let mut all = Vec::new();
    all.extend_from_slice(LOWER);
    all.extend_from_slice(UPPER);
    all.extend_from_slice(DIGITS);
    all.extend_from_slice(symbols);

    while chars.len() < length {
        chars.push(random_char(&all)?);
    }

    shuffle(&mut chars)?;
    String::from_utf8(chars).map_err(|_| {
        KiroAuthSupportError::InvalidResponse("generated password was not UTF-8".to_owned())
    })
}

fn generate_random_gender_name(full_name: bool) -> KiroAuthSupportResult<EnglishName> {
    let total = MALE_NAMES.len() + FEMALE_NAMES.len();
    let index = random_index(total)?;
    let first_name = if index < MALE_NAMES.len() {
        MALE_NAMES[index]
    } else {
        FEMALE_NAMES[index - MALE_NAMES.len()]
    }
    .to_owned();
    let last_name = if full_name {
        Some(choose(SURNAMES)?.to_owned())
    } else {
        None
    };
    Ok(EnglishName {
        first_name,
        last_name,
    })
}

fn choose<'a>(pool: &'a [&str]) -> KiroAuthSupportResult<&'a str> {
    Ok(pool[random_index(pool.len())?])
}

fn random_char(alphabet: &[u8]) -> KiroAuthSupportResult<u8> {
    Ok(alphabet[random_index(alphabet.len())?])
}

fn random_index(upper_bound: usize) -> KiroAuthSupportResult<usize> {
    if upper_bound == 0 {
        return Err(KiroAuthSupportError::InvalidConfig(
            "random choice pool cannot be empty".to_owned(),
        ));
    }
    let bytes = random_bytes::<8>()?;
    let value = u64::from_be_bytes(bytes);
    Ok((value as usize) % upper_bound)
}

fn random_bytes<const N: usize>() -> KiroAuthSupportResult<[u8; N]> {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; N];
    rng.fill(&mut bytes)
        .map_err(|_| KiroAuthSupportError::Crypto)?;
    Ok(bytes)
}

fn shuffle(values: &mut [u8]) -> KiroAuthSupportResult<()> {
    if values.len() < 2 {
        return Ok(());
    }

    for i in (1..values.len()).rev() {
        let j = random_index(i + 1)?;
        values.swap(i, j);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        EnglishNameOptions, NameGender, PasswordPolicy, generate_english_name, generate_password,
    };

    #[test]
    fn password_has_required_character_classes() {
        let password = generate_password(PasswordPolicy {
            length: 20,
            symbols: "!".to_owned(),
        })
        .expect("password");

        assert_eq!(password.len(), 20);
        assert!(password.chars().any(|ch| ch.is_ascii_lowercase()));
        assert!(password.chars().any(|ch| ch.is_ascii_uppercase()));
        assert!(password.chars().any(|ch| ch.is_ascii_digit()));
        assert!(password.contains('!'));
    }

    #[test]
    fn name_generation_can_emit_full_female_name() {
        let name = generate_english_name(EnglishNameOptions {
            full_name: true,
            gender: NameGender::Female,
        })
        .expect("name");

        assert!(name.last_name.is_some());
        assert!(name.display_name().contains(' '));
    }
}
