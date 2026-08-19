use gathr_application::otp::{self, Channel, Delivery};
use gathr_application::AppError;
use gathr_infra_db::Db;

async fn db() -> Db {
    let _ = dotenvy::from_filename("../../.env");
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run these tests");
    gathr_infra_db::connect(&url, 4)
        .await
        .expect("postgres must be reachable")
}

#[tokio::test]
async fn a_development_build_hands_the_code_back_instead_of_sending_it() {
    let db = db().await;

    let challenge = otp::request(
        &db,
        Channel::Email,
        "amara@example.com",
        Delivery {
            email: None,
            reveal_instead_of_sending: true,
        },
    )
    .await
    .expect("a development request should succeed without any mail provider");

    let code = challenge
        .code_for_development
        .expect("a development build must reveal the code");
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|character| character.is_ascii_digit()));
}

