use chrono::{Duration, TimeZone, Utc};
use chrono_tz::America::Phoenix;
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[allow(non_upper_case_globals)]
#[derive(RustEmbed)]
#[folder = "config/"]
struct EmbeddedAsset;

#[derive(Deserialize)]
struct SecretsConfigFile {
    base_url: String,
    company_id: String,
    email: String,
}

lazy_static! {
    static ref CONFIG: SecretsConfigFile = {
        serde_ini::from_str(
            &String::from_utf8(EmbeddedAsset::get("mystudio.ini").unwrap().data.to_vec())
                .unwrap()
                .to_string(),
        )
        .unwrap()
    };
}

pub async fn load_students() -> Result<Vec<crate::api::Student>, LoadStudentsError> {
    let client = reqwest::Client::new();
    let mut students_out = vec![];

    let GenerateStudioAttendanceTokenOut { msg: token } = client
        .post(format!("{}/generateStudioAttendanceToken", CONFIG.base_url))
        .json(&GenerateStudioAttendanceTokenIn {
            company_id: CONFIG.company_id.clone(),
            email: CONFIG.email.clone(),
            from_page: "attendance".to_string(),
        })
        .send()
        .await?
        .json::<GenerateStudioAttendanceTokenOut>()
        .await?;

    let AllParticipantsOut { student_detail } = client
        .post(format!("{}/allParticipants", CONFIG.base_url))
        .json(&AllParticipantsIn {
            company_id: CONFIG.company_id.clone(),
            email: CONFIG.email.clone(),
            from: "attendance".to_string(),
            from_page: "attendance".to_string(),
            program_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            token: token.clone(),
        })
        .send()
        .await?
        .json::<AllParticipantsOut>()
        .await?;

    for (_category, students) in student_detail {
        for student in students {
            let GetAvailableClassDetailsOut { class_details } = client
                .post(format!("{}/getAvailableClassDetails", CONFIG.base_url))
                .json(&GetAvailableClassDetailsIn {
                    token: token.clone(),
                    company_id: CONFIG.company_id.clone(),
                    email: CONFIG.email.clone(),
                    user_login_type: "".to_string(),
                    from: "attendance".to_string(),
                    from_page: "attendance".to_string(),
                    participant_id: student.participant_id.clone(),
                    student_id: student.student_id.clone(),
                    reg_id: student.membership_registration_id.clone(),
                    reg_id_type: "M".to_string(),
                    selected_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
                    student_view: "Y".to_string(),
                    field_type: "membership".to_string(),
                })
                .send()
                .await?
                .json::<GetAvailableClassDetailsOut>()
                .await?;

            let mut num_hours = 0;
            let mut checkin_times = vec![];
            for ClassDetail {
                checkin_status,
                checkin_time_utc,
            } in class_details
            {
                if checkin_status == "Cancel check in" {
                    num_hours += 1;
                    let time = Utc
                        .datetime_from_str(&checkin_time_utc, "%Y-%m-%d %H:%M:%S")?
                        .with_timezone(&Phoenix);
                    checkin_times.push(time);
                }
            }

            if !checkin_times.is_empty() {
                checkin_times.sort();
                let start = checkin_times.first().unwrap().clone();
                let end = checkin_times.first().unwrap().clone() + Duration::hours(num_hours);
                let name = format!(
                    "{} {}",
                    student.participant_first_name, student.participant_last_name
                );

                students_out.push(crate::api::Student {
                    name,
                    belt: student.rank_name,
                    time_start_dt: checkin_times.first().unwrap().with_timezone(&Utc).clone(),
                    time_start: start.format("%I:%M %P").to_string(),
                    time_end: end.format("%I:%M %P").to_string(),
                });
            }
        }
    }

    students_out.sort_by(|a, b| a.time_start_dt.cmp(&b.time_start_dt));
    Ok(students_out)
}

#[derive(Error, Debug)]
pub enum LoadStudentsError {
    #[error("Request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Failed to parse date: {0}")]
    InvalidDate(#[from] chrono::ParseError),
}

#[derive(Serialize, Clone, Debug)]
struct GenerateStudioAttendanceTokenIn {
    company_id: String,
    email: String,
    from_page: String,
}

#[derive(Deserialize, Clone, Debug)]
struct GenerateStudioAttendanceTokenOut {
    msg: String,
}

#[derive(Serialize, Clone, Debug)]
struct AllParticipantsIn {
    company_id: String,
    email: String,
    from: String,
    from_page: String,
    program_date: String,
    token: String,
}

#[derive(Deserialize, Clone, Debug)]
struct StudentDetails {
    student_id: String,
    membership_registration_id: String,
    participant_id: String,
    participant_first_name: String,
    participant_last_name: String,
    rank_name: String,
}

#[derive(Deserialize, Clone, Debug)]
struct AllParticipantsOut {
    student_detail: HashMap<String, Vec<StudentDetails>>,
}

#[derive(Serialize, Clone, Debug)]
struct GetAvailableClassDetailsIn {
    company_id: String,
    token: String,
    email: String,
    user_login_type: String,
    from: String,
    from_page: String,
    participant_id: String,
    student_id: String,
    reg_id: String,
    reg_id_type: String,
    selected_date: String,
    student_view: String,
    #[serde(rename = "type")]
    field_type: String,
}

#[derive(Deserialize, Clone, Debug)]
struct ClassDetail {
    checkin_status: String,
    #[serde(rename = "att_checkin_datetime")]
    checkin_time_utc: String,
}

#[derive(Deserialize, Clone, Debug)]
struct GetAvailableClassDetailsOut {
    class_details: Vec<ClassDetail>,
}
