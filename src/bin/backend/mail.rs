use lettre::{
    message::{MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use once_cell::sync::Lazy;
use tempusalert_be::parse_env_var::parse_env_var;

const SMTP_USER: Lazy<String> = Lazy::new(|| parse_env_var("SMTP_USER"));
const SMTP_PASSWORD: Lazy<String> = Lazy::new(|| parse_env_var("SMTP_PASSWORD"));
const SMTP_HOSTNAME: Lazy<String> = Lazy::new(|| parse_env_var("SMTP_HOSTNAME"));

pub fn send_mail(receiver_email: String, title: String, body: String) -> Option<()> {
    let email = Message::builder()
        .from(
            format!("Tempusalert <noreply@tempusalert.com>")
                .parse()
                .ok()?,
        )
        .to(format!("Receiver <{}>", receiver_email).parse().ok()?)
        .subject(title)
        .multipart(MultiPart::alternative().singlepart(SinglePart::html(body)))
        .ok()?;

    let creds = Credentials::new(SMTP_USER.to_owned(), SMTP_PASSWORD.to_string());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(SMTP_HOSTNAME.as_str())
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => Some(()),
        Err(_) => None,
    }
}
