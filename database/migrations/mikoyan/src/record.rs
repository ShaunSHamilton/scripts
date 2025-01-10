use mongodb::{
    self,
    bson::{self, oid::ObjectId},
};
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum NOption<T> {
    Some(T),
    #[default]
    Null,
    #[allow(dead_code)]
    Undefined,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub about: String,
    pub accepted_privacy_terms: bool,
    pub completed_challenges: Vec<CompletedChallenge>,
    pub completed_exams: Vec<CompletedExam>,
    pub current_challenge_id: NOption<String>,
    pub donation_emails: Vec<String>,
    pub email: String,
    #[serde(rename = "emailAuthLinkTTL")]
    pub email_auth_link_ttl: NOption<bson::DateTime>,
    pub email_verified: bool,
    #[serde(rename = "emailVerifyTTL")]
    pub email_verify_ttl: NOption<bson::DateTime>,
    pub external_id: NOption<String>,
    pub github_profile: String,
    pub is_2018_data_vis_cert: bool,
    pub is_2018_full_stack_cert: bool,
    pub is_apis_microservices_cert: bool,
    pub is_back_end_cert: bool,
    pub is_banned: bool,
    pub is_cheater: bool,
    pub is_classroom_account: bool,
    pub is_college_algebra_py_cert_v8: bool,
    pub is_data_analysis_py_cert_v7: bool,
    pub is_data_vis_cert: bool,
    pub is_donating: bool,
    pub is_foundational_c_sharp_cert_v8: bool,
    pub is_front_end_cert: bool,
    pub is_front_end_libs_cert: bool,
    pub is_full_stack_cert: bool,
    pub is_honest: bool,
    pub is_infosec_cert_v7: bool,
    // #[serde(rename = "isInfosecQACert")]
    pub is_infosec_qa_cert: bool,
    pub is_js_algo_data_struct_cert: bool,
    pub is_js_algo_data_struct_cert_v8: bool,
    pub is_machine_learning_py_cert_v7: bool,
    // #[serde(rename = "isQACertV7")]
    pub is_qa_cert_v7: bool,
    pub is_relational_database_cert_v8: bool,
    pub is_resp_web_design_cert: bool,
    pub is_sci_comp_py_cert_v7: bool,
    pub keyboard_shortcuts: bool,
    #[serde(rename = "lastUpdatedAtInMS")]
    pub last_updated_at_in_ms: u64,
    pub linkedin: String,
    pub location: String,
    pub name: String,
    pub needs_moderation: bool,
    pub new_email: NOption<String>,
    pub partially_completed_challenges: Vec<PartiallyCompletedChallenge>,
    pub picture: String,
    pub portfolio: Vec<Portfolio>,
    #[serde(rename = "profileUI")]
    pub profile_ui: ProfileUI,
    pub progress_timestamps: Vec<u64>,
    pub rand: f64,
    pub saved_challenges: Vec<SavedChallenge>,
    pub send_quincy_email: bool,
    pub theme: String,
    pub twitter: String,
    pub unsubscribe_id: String,
    pub username: String,
    pub username_display: String,
    pub website: String,
    pub years_top_contributor: Vec<u32>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletedChallenge {
    pub challenge_type: NOption<u32>,
    pub completed_date: u64,
    pub files: Vec<File>,
    pub github_link: NOption<String>,
    pub id: String,
    pub is_manually_approved: NOption<bool>,
    pub solution: NOption<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletedExam {
    pub challenge_type: u32,
    pub completed_date: u64,
    pub exam_results: ExamResults,
    pub id: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExamResults {
    pub exam_time_in_seconds: u32,
    pub number_of_correct_answers: u32,
    pub number_of_questions_in_exam: u32,
    pub passed: bool,
    pub passing_percent: f64,
    pub percent_correct: f64,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PartiallyCompletedChallenge {
    pub completed_date: u64,
    pub id: String,
}

#[derive(Debug, Serialize, Default, PartialEq)]
pub struct Portfolio {
    pub description: String,
    pub id: String,
    pub image: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUI {
    pub is_locked: bool,
    pub show_about: bool,
    pub show_certs: bool,
    pub show_donation: bool,
    pub show_heat_map: bool,
    pub show_location: bool,
    pub show_name: bool,
    pub show_points: bool,
    pub show_portfolio: bool,
    pub show_time_line: bool,
}

impl Default for ProfileUI {
    fn default() -> Self {
        Self {
            is_locked: true,
            show_about: Default::default(),
            show_certs: Default::default(),
            show_donation: Default::default(),
            show_heat_map: Default::default(),
            show_location: Default::default(),
            show_name: Default::default(),
            show_points: Default::default(),
            show_portfolio: Default::default(),
            show_time_line: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SavedChallenge {
    pub challenge_type: u32,
    pub files: Vec<File>,
    pub id: String,
    pub last_saved_date: u64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct File {
    pub contents: String,
    pub ext: String,
    pub key: String,
    pub name: String,
    pub path: String,
}
