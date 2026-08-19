use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use time::OffsetDateTime;

use gathr_infra_oidc::{verify, JwkSet, OidcError, Provider};

const TEST_ONLY_SIGNING_KEY: &str = "\
-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDd18LbJki6AmkM
Gz5DAssxxaexuvbF8oD+TRLhJ7vPnEfNVFbrcL3io3x3B4hk6a7CeGNEsBCPz5h/
F9+XdIUvDAqjLtptLNBPo19mubrYoz0bvU6UXZbLHgcQwMnyXEsbpJHfllqNx0qI
AnzCO37iJqI/+QgMDajwQr+UodOwkA0l4fS9xJvf7rLpIKIqM3quE1Un6hXEGth9
c5G4HtFT1hQ2bdiHnGuUFHRVxgjOPDrXSe+BgUuK9vTcmZKhcEBPQuCN712r6mVj
63VdVbPntxE0iOSMnU1lZhgF/c903tKqhxkZJCejlbDe08oHOFtewuYkRGDqL7UU
tD23NVIZAgMBAAECggEALr5Ut3qEnnT9j2h6IQKIsA8Wu5NKhgEH/w1cagbGQwOJ
+ZDI08koqYWLJ0V2DtVSVnWpbQKwKq0yU61DFpWdroEaCLJJofFsXytsYafG3/jd
Wmf0E7z4lgnYsUX+B0w9IzD076itoewZHzlu8jo5DQvY6iPd9zQ1oZJe7cF/alc7
j+IwIaDskfOeVtPiipUlDG83TAj2OwxNlbTWBoWCDrEHu1VTYZg8o8rIcaelnjRN
xsmxGuI6oJr52Sv7i9osgLt1fihWIq1UdBoFV7uj2f1UQoZ17Mf74xFwiiwQ4sYJ
Hq3DjwesRa3qU8X34biTBhB7RPoQQqd700KJO6yO5QKBgQDv13R1e/v30N89oqFl
VybPMaFQ6HgxPewzIWFmWi0n44U1/4d+ZHS1uf6aLJWy4eXvQrJeeeYjACPjqw2b
VwbJV/Zftm4B/voX0JCJFwz6GWqu2eN96oGV+6a1HwlPIEmRQsGk2iACIqYGeAZX
xEPIadAhCm+RLOrUQpTY4IT8iwKBgQDsyeGRNK0IfCNhujYFjVTdu8421cnVxGgR
H+MDGCxYLiGfrUlVzP6f2JNts6b1lKaa3lR+6CPMExv9X30VrbXIHIGvu66WQeNS
1SPWIB0E9+VOBRhxfg1kaiY+9oQstKIWETwwjAC602k961Rp+wRa69Yj3KXLCScF
qWxzKkPMawKBgQDojicUC6YjgloT729DTiAJYvwh8Wcph/tREJranbGPLeNqmlyM
x2cLFk+yumxkyPkk560AQn5NjcM+7AsDhnzEGhCAeels8gkZsleTNdCVaLOy2v7k
htAj16Crmm5yVMJAoVQWPpIuv93wA81SqLF40HDIuM+5Gq6QJgchy4HnRQKBgDYj
Ujdo28b82voVIRTT43tu3Q8cgjEl3sVLjRPACyN+KKHhdMrxO6ZAVjTWxm2Ptyfh
fWAmo0iddeDQXEEAnndKTTxopNol5luh77edUAU6yGq2L4iDUXyL6IKsyjuOcSCq
gCB5YhFVFNLbY0l34t8G3McGQ8HQLePVgL40A5xRAoGAbi96j5rqsZY0cpa2DwW9
60VOodW5PBFYpH3iZbbAxfi1CDHnfFt3DwIcODS5tKrBspxXItRXZcVuBpbZ7Z4E
smTPAbPMinBMdNFVVd5G9WGgloLGdg6fXrDkZs67sv2QJoloaH/OkNVkJ2nYjF26
PP2TOq5or7lyDzCyzi31oqI=
-----END PRIVATE KEY-----
";

const TEST_ONLY_MODULUS: &str = "3dfC2yZIugJpDBs-QwLLMcWnsbr2xfKA_k0S4Se7z5xHzVRW63C94qN8dweIZOmuwnhjRLAQj8-Yfxffl3SFLwwKoy7abSzQT6NfZrm62KM9G71OlF2Wyx4HEMDJ8lxLG6SR35ZajcdKiAJ8wjt-4iaiP_kIDA2o8EK_lKHTsJANJeH0vcSb3-6y6SCiKjN6rhNVJ-oVxBrYfXORuB7RU9YUNm3Yh5xrlBR0VcYIzjw610nvgYFLivb03JmSoXBAT0Lgje9dq-plY-t1XVWz57cRNIjkjJ1NZWYYBf3PdN7SqocZGSQno5Ww3tPKBzhbXsLmJERg6i-1FLQ9tzVSGQ";

const KEY_ID: &str = "test-key";

#[derive(Serialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<String>,
}

fn keys() -> JwkSet {
    serde_json::from_value(serde_json::json!({
        "keys": [{ "kid": KEY_ID, "n": TEST_ONLY_MODULUS, "e": "AQAB" }]
    }))
    .expect("the fixture key set should parse")
}

fn sign(claims: &Claims) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KEY_ID.to_owned());
    encode(
        &header,
        claims,
        &EncodingKey::from_rsa_pem(TEST_ONLY_SIGNING_KEY.as_bytes())
            .expect("the fixture key should load"),
    )
    .expect("signing should succeed")
}

fn google_claims() -> Claims {
    Claims {
        sub: "google-subject-1".to_owned(),
        iss: "https://accounts.google.com".to_owned(),
        aud: "gathr-ios-client".to_owned(),
        exp: (OffsetDateTime::now_utc().unix_timestamp()) + 600,
        email: Some("amara@example.com".to_owned()),
        name: Some("Amara Chukwu".to_owned()),
        nonce: None,
    }
}

fn audiences() -> Vec<String> {
    vec!["gathr-ios-client".to_owned()]
}

#[test]
fn a_well_formed_google_token_yields_the_subject_name_and_email() {
    let token = sign(&google_claims());
    let identity = verify(&token, &keys(), Provider::Google, &audiences(), None)
        .expect("a valid token should verify");

    assert_eq!(identity.subject, "google-subject-1");
    assert_eq!(identity.name.as_deref(), Some("Amara Chukwu"));
    assert_eq!(identity.email.as_deref(), Some("amara@example.com"));
}

#[test]
fn a_token_minted_for_another_app_is_rejected() {
    let mut claims = google_claims();
    claims.aud = "some-other-app".to_owned();
    let token = sign(&claims);

    assert!(matches!(
        verify(&token, &keys(), Provider::Google, &audiences(), None),
        Err(OidcError::InvalidToken(_))
    ));
}

#[test]
fn an_expired_token_is_rejected() {
    let mut claims = google_claims();
    claims.exp = OffsetDateTime::now_utc().unix_timestamp() - 3_600;
    let token = sign(&claims);

    assert!(matches!(
        verify(&token, &keys(), Provider::Google, &audiences(), None),
        Err(OidcError::InvalidToken(_))
    ));
}

#[test]
fn a_google_token_presented_as_apple_is_rejected_on_issuer() {
    let token = sign(&google_claims());

    assert!(matches!(
        verify(&token, &keys(), Provider::Apple, &audiences(), None),
        Err(OidcError::InvalidToken(_))
    ));
}

