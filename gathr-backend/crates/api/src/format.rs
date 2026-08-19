use gathr_domain::Category;
use time::format_description::BorrowedFormatItem;
use time::{macros::format_description, OffsetDateTime, UtcOffset};

const DAY_AND_DATE: &[BorrowedFormatItem<'_>] =
    format_description!("[weekday repr:short], [month repr:short] [day padding:none]");
const CLOCK: &[BorrowedFormatItem<'_>] =
    format_description!("[hour repr:12 padding:none]:[minute] [period]");

pub fn in_zone(instant: OffsetDateTime, timezone: &str) -> OffsetDateTime {
    let offset = match timezone {
        "Africa/Lagos" => UtcOffset::from_hms(1, 0, 0).unwrap_or(UtcOffset::UTC),
        _ => UtcOffset::UTC,
    };
    instant.to_offset(offset)
}

pub fn long_when(instant: OffsetDateTime, timezone: &str) -> String {
    let local = in_zone(instant, timezone);
    let date = local.format(DAY_AND_DATE).unwrap_or_default();
    let clock = local.format(CLOCK).unwrap_or_default();
    format!("{date} · {clock}")
}

pub fn category_label(category: Category) -> &'static str {
    match category {
        Category::Birthday => "BIRTHDAY",
        Category::Party => "PARTY",
        Category::Meetup => "MEETUP",
        Category::Dinner => "DINNER",
        Category::GameNight => "GAME NIGHT",
        Category::Wedding => "WEDDING",
        Category::Other => "EVENT",
    }
}

pub fn category_tint(category: Category) -> &'static str {
    match category {
        Category::Birthday => "#ff375f",
        Category::Party => "#af52de",
        Category::Meetup => "#0a84ff",
        Category::Dinner => "#ff9500",
        Category::GameNight => "#30d158",
        Category::Wedding => "#ff2d55",
        Category::Other => "#8e8e93",
    }
}

